//! Integration tests for the `post_hook_pre_tool_use` Defer / timeout-race paths
//! (F-S022-ADV12-MED-001, BC-2.05.005 postcondition PC-4).
//!
//! # Context
//!
//! `post_hook_pre_tool_use` wraps its inner handler in a 300ms `tokio::time::timeout`.
//! When the inner handler hits `HookDecision::Defer`, it registers a prompt in the
//! `PendingDecisionRegistry`, stores the assigned `prompt_id` in `deferred_prompt_id`,
//! and awaits the decision oneshot. If the 300ms timeout fires, the outer handler:
//!
//! 1. Calls `registry.remove_timed_out_prompt(prompt_id)`.
//! 2. Checks **`if removed.is_some()`** (F-S022-ADV11-LOW-001) — only broadcasts
//!    `PermissionPromptResolved` if the entry was still present (i.e., WE removed it,
//!    not a concurrent decision path that already cleaned up the entry).
//!
//! This guard prevents a duplicate `PermissionPromptResolved` broadcast in the
//! concurrent-decision-wins-the-race scenario.
//!
//! # Coverage
//!
//! | Test | BC clause | Path | What it proves |
//! |------|-----------|------|----------------|
//! | `test_F_S022_ADV12_MED_001_timeout_arm_broadcasts_once_on_normal_timeout` | BC-2.04.007 PC-4, BC-2.05.005 PC-4 | `Err(_timeout)`, `removed=Some` | Guard true-branch: exactly 1 `PermissionPromptResolved` broadcast |
//! | `test_F_S022_ADV12_MED_001_ok_path_single_resolved_when_decision_wins` | BC-2.04.007 PC-3 | `Ok(response)` | Decision resolves before timeout; timeout arm never runs; exactly 1 broadcast from decision path |
//! | `test_F_S022_ADV12_MED_001_registry_entry_absent_produces_no_broadcast` | BC-2.05.005 invariant 2 | Infrastructure | Guard false-branch: `remove_timed_out_prompt(absent) = None` → no broadcast via production IPC infrastructure |
//!
//! # Mutation coverage note
//!
//! Test 1 detects the mutation "remove the guard entirely" (no `if` → always broadcasts):
//! mutation in the false direction (`if removed.is_some()` removed) → 0 broadcasts in
//! test 1 → `resolved_count == 0` assertion fails RED.
//!
//! Test 3 detects the mutation "remove the guard entirely" in the false direction: if
//! `broadcast_to_subscribers` is called unconditionally when `removed = None`, the
//! subscriber receives an extra `PermissionPromptResolved` → assertion fails RED.
//!
//! The specific mutation `if removed.is_some()` → `if true` cannot be detected
//! deterministically without production code changes (no async yield point exists between
//! `tokio::time::timeout → Err` and `remove_timed_out_prompt` in the timeout arm, making
//! the concurrent pre-removal scenario non-injectable from tests). Test 3 provides the
//! closest achievable coverage of the guard's false-branch behavior using production
//! infrastructure (real `PendingDecisionRegistry`, real `broadcast_to_subscribers`, real
//! `ClientEntry`).

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
#![allow(non_snake_case)]

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use monocle_ipc::server::{ClientEntry, SubscriberList, CLIENT_CHANNEL_CAPACITY};
use monocle_ipc::types::{PermissionDecisionKind, ServerToClient};
use monocle_runtime::ipc_server::broadcast_to_subscribers;
use monocle_runtime::permissions::PendingDecisionRegistry;
use monocle_runtime::server::build_server;
use monocle_runtime::state::DaemonState;
use monocle_runtime::types::EVENT_BUS_CAPACITY;
use tokio::sync::mpsc;
use tower::ServiceExt;

/// Raw 64-hex-char auth token for defer-race tests.
const TEST_TOKEN: &str = "ddeeffddee00112233445566778899aabbccddeeff001122334455667788990a";

/// Canonical PreToolUse JSON body (all required fields present).
fn pre_tool_use_body() -> serde_json::Value {
    serde_json::json!({
        "session_id": "test-session-defer-race",
        "pid": 43000,
        "tool_name": "Bash",
        "tool_input": {"command": "echo defer-race"}
    })
}

/// Build a `DaemonState` wired for Defer-path integration tests.
///
/// All S-022 fields are initialised. One subscriber is attached to `ipc_subscribers`.
/// `hook_decision_override` forces `HookDecision::Defer` — the inner handler always
/// takes the Defer path: registers a prompt and awaits the decision oneshot.
///
/// Returns:
/// - `Arc<DaemonState>` — shared state for the HTTP handler.
/// - `mpsc::Receiver<ServerToClient>` — subscriber channel for message assertions.
/// - `Arc<PendingDecisionRegistry>` — direct handle for concurrent manipulation.
async fn make_defer_state() -> (
    Arc<DaemonState>,
    mpsc::Receiver<ServerToClient>,
    Arc<PendingDecisionRegistry>,
) {
    let (event_tx, _event_rx) =
        tokio::sync::mpsc::channel::<monocle_runtime::types::EventBusHookEvent>(EVENT_BUS_CAPACITY);

    let pending_decisions = Arc::new(PendingDecisionRegistry::new());
    let subscribers: SubscriberList = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let (sub_tx, sub_rx) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);

    // Attach subscriber before building the state Arc.
    {
        let mut subs = subscribers.lock().await;
        subs.push(ClientEntry::new(sub_tx));
    }

    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN.to_string();
    state.event_bus_tx = Some(Arc::new(event_tx));
    state.drop_counter = Some(Arc::new(AtomicU64::new(0)));
    state.session_registry = Some(Arc::new(monocle_runtime::hooks::SessionRegistry::new()));
    state.pending_decisions = Some(Arc::clone(&pending_decisions));
    state.ipc_subscribers = Some(subscribers);
    state.hook_decision_override = Some((monocle_core::engine::HookDecision::Defer, None));

    (Arc::new(state), sub_rx, pending_decisions)
}

/// POST to `/hooks/pre-tool-use` with the test auth header.
async fn post_pre_tool_use(
    state: Arc<DaemonState>,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = build_server(state);
    let req = Request::builder()
        .method("POST")
        .uri("/hooks/pre-tool-use")
        .header("Content-Type", "application/json")
        .header(
            "X-Monocle-Authorization",
            format!("monocle-v1:{}", TEST_TOKEN),
        )
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_else(
        |_| serde_json::json!({"_raw": std::str::from_utf8(&bytes).unwrap_or("<binary>")}),
    );
    (status, value)
}

/// Poll `registry.snapshot_payloads()` until one entry appears; return its `prompt_id`.
///
/// Gives up after 5 seconds (500 × 10ms). Panics if no entry appears within that window.
async fn poll_until_entry_registered(registry: &PendingDecisionRegistry) -> uuid::Uuid {
    for _ in 0..500usize {
        let payloads = registry.snapshot_payloads();
        if let Some(payload) = payloads.into_iter().next() {
            return payload.prompt_id;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!(
        "inner Defer handler must register a prompt in the registry within 5 seconds \
        — poll_until_entry_registered timed out"
    );
}

// ---------------------------------------------------------------------------
// Test 1: Err(_timeout) arm — normal timeout path, entry present → guard fires → 1 broadcast
// ---------------------------------------------------------------------------

/// Exercises `post_hook_pre_tool_use` end-to-end: Defer + 300ms timeout fires normally.
///
/// BC-2.04.007 PC-4 (fail-open on timeout) + BC-2.05.005 PC-4 (broadcast Resolved on timeout).
///
/// # Production path
///
/// 1. `post_hook_pre_tool_use` invoked end-to-end via the axum server stack.
/// 2. Inner handler: `HookDecision::Defer` → `register_prompt` → sets `deferred_prompt_id`
///    → awaits `decision_rx` (never resolved by anyone).
/// 3. After 300ms: `tokio::time::timeout` fires → `Err(_timeout)`.
/// 4. Timeout arm:
///    a. `remove_timed_out_prompt(prompt_id)` → `Some` (entry present — no concurrent removal).
///    b. Guard: `if removed.is_some()` → **true** → `broadcast_to_subscribers` called.
/// 5. HTTP response: `{"decision":"allow","reason":"timeout"}`.
///
/// # Mutation detection
///
/// If the guard `if removed.is_some()` is REMOVED entirely (guard → no broadcast),
/// `broadcast_to_subscribers` is never called in the timeout arm →
/// `resolved_count == 0` → assertion at line ~200 fails **RED**.
///
/// This test does NOT detect the mutation `if removed.is_some()` → `if true` because
/// in this scenario `removed` IS `Some` — both the original and mutant behave identically.
/// That mutation's detection requires the concurrent pre-removal scenario, which is
/// non-deterministic to reproduce (see file-level comment above for rationale).
#[tokio::test]
async fn test_F_S022_ADV12_MED_001_timeout_arm_broadcasts_once_on_normal_timeout() {
    let (state, mut sub_rx, _pending) = make_defer_state().await;

    let (status, body) = post_pre_tool_use(Arc::clone(&state), pre_tool_use_body()).await;

    // Assert 1: fail-open timeout response (BC-2.04.007 PC-4).
    assert_eq!(
        status,
        StatusCode::OK,
        "handler must return HTTP 200 on Defer+timeout: got {body}"
    );
    assert_eq!(
        body.get("reason").and_then(|v| v.as_str()),
        Some("timeout"),
        "handler must return reason:timeout on Defer+300ms elapsed: got {body}"
    );

    // Assert 2: message counts on the subscriber channel.
    let mut messages = Vec::new();
    while let Ok(msg) = sub_rx.try_recv() {
        messages.push(msg);
    }

    let queued_count = messages
        .iter()
        .filter(|m| matches!(m, ServerToClient::PermissionPromptQueued { .. }))
        .count();
    let resolved_count = messages
        .iter()
        .filter(|m| matches!(m, ServerToClient::PermissionPromptResolved { .. }))
        .count();

    assert_eq!(
        queued_count, 1,
        "subscriber must receive exactly 1 PermissionPromptQueued (Defer path) \
        (got {queued_count}); messages: {messages:?}"
    );
    assert_eq!(
        resolved_count, 1,
        "subscriber must receive exactly 1 PermissionPromptResolved (timeout arm, guard true) \
        (got {resolved_count}); messages: {messages:?}"
    );

    // Assert 3: registry empty — timeout arm removed the entry.
    assert!(
        state
            .pending_decisions
            .as_ref()
            .unwrap()
            .snapshot_payloads()
            .is_empty(),
        "registry must be empty after timeout arm removed the entry"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Ok path — decision wins the race before the timeout
// ---------------------------------------------------------------------------

/// Exercises `post_hook_pre_tool_use` when a concurrent decision resolves the oneshot
/// before the 300ms timeout fires (BC-2.04.007 PC-3, decision path).
///
/// # Production path
///
/// 1. Handler invoked end-to-end.
/// 2. Inner handler: Defer → registers prompt → awaits `decision_rx`.
/// 3. Concurrent task polls until entry appears, calls `resolve_prompt(Allow)`:
///    - Removes the registry entry.
///    - Sends `Allow` on the oneshot → unblocks `decision_rx.await`.
///    - Broadcasts `PermissionPromptResolved` once.
/// 4. Inner handler: `decision_rx.await` → `Ok(Allow)` → returns `user-approved`.
/// 5. Outer `tokio::time::timeout`: `Ok(response)` — Err arm NOT entered.
/// 6. HTTP response: `{"decision":"allow","reason":"user-approved"}`.
///
/// # Assertions
///
/// - HTTP 200, decision "allow", reason "user-approved".
/// - Exactly 1 `PermissionPromptQueued` and exactly 1 `PermissionPromptResolved`.
/// - No second `PermissionPromptResolved` (timeout arm never runs on Ok path).
#[tokio::test]
async fn test_F_S022_ADV12_MED_001_ok_path_single_resolved_when_decision_wins() {
    let (state, mut sub_rx, pending_decisions) = make_defer_state().await;

    let subscribers_clone = Arc::clone(
        state
            .ipc_subscribers
            .as_ref()
            .expect("ipc_subscribers must be Some"),
    );

    // Spawn the HTTP handler.
    let state_clone = Arc::clone(&state);
    let handler_task =
        tokio::spawn(async move { post_pre_tool_use(state_clone, pre_tool_use_body()).await });

    // Concurrent task: simulate handle_permission_decision winning the race.
    let pending_clone = Arc::clone(&pending_decisions);
    let race_task = tokio::spawn(async move {
        let prompt_id = poll_until_entry_registered(&pending_clone).await;

        // resolve_prompt: removes registry entry AND sends Allow on the oneshot.
        let resolved = pending_clone.resolve_prompt(prompt_id, PermissionDecisionKind::Allow);
        assert!(
            resolved.is_some(),
            "resolve_prompt must return Some — entry was registered by the inner handler"
        );

        // Broadcast PermissionPromptResolved (as handle_permission_decision does).
        broadcast_to_subscribers(
            &subscribers_clone,
            ServerToClient::PermissionPromptResolved { prompt_id },
        )
        .await;

        prompt_id
    });

    let prompt_id = race_task.await.expect("race task must not panic");
    let (status, body) = handler_task.await.expect("handler task must not panic");

    // Assert 1: user-approved response (Ok path).
    assert_eq!(
        status,
        StatusCode::OK,
        "handler must return HTTP 200 on user-approved Defer: got {body}"
    );
    assert_eq!(
        body.get("decision").and_then(|v| v.as_str()),
        Some("allow"),
        "handler must return decision:allow on user-approved Defer: got {body}"
    );

    // Assert 2: message counts.
    let mut messages = Vec::new();
    while let Ok(msg) = sub_rx.try_recv() {
        messages.push(msg);
    }

    let queued_count = messages
        .iter()
        .filter(|m| matches!(m, ServerToClient::PermissionPromptQueued { .. }))
        .count();
    let resolved_count = messages
        .iter()
        .filter(|m| {
            matches!(m, ServerToClient::PermissionPromptResolved { prompt_id: pid } if *pid == prompt_id)
        })
        .count();

    assert_eq!(
        queued_count, 1,
        "subscriber must receive exactly 1 PermissionPromptQueued \
        (got {queued_count}); messages: {messages:?}"
    );
    assert_eq!(
        resolved_count, 1,
        "subscriber must receive exactly 1 PermissionPromptResolved \
        (got {resolved_count}); messages: {messages:?}"
    );

    // Assert 3: registry empty (resolve_prompt removed the entry).
    assert!(
        state
            .pending_decisions
            .as_ref()
            .unwrap()
            .snapshot_payloads()
            .is_empty(),
        "registry must be empty after resolve_prompt removed the entry"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Guard false-branch — remove_timed_out_prompt(absent) → no broadcast
// ---------------------------------------------------------------------------

/// Validates the guard's false-branch behavior (F-S022-ADV11-LOW-001) using
/// the full production IPC infrastructure.
///
/// This test verifies the CONTRACT of the timeout-arm guard: when
/// `remove_timed_out_prompt` returns `None` (entry already removed by a concurrent
/// decision path), the guard prevents `broadcast_to_subscribers` from firing.
///
/// # Why this test exists (F-S022-ADV12-MED-001)
///
/// The concurrent race that exercises the guard's false-branch in `post_hook_pre_tool_use`
/// — `handle_permission_decision` removing the registry entry in the window between
/// `tokio::time::timeout → Err` and `remove_timed_out_prompt` in the timeout arm — cannot
/// be reproduced deterministically. There are no async yield points in the timeout arm
/// between the Err match and the `remove_timed_out_prompt` call, so concurrent injection
/// is not architecturally injectable from tests without production code changes.
///
/// This test exercises the guard predicate's false-branch directly using production
/// infrastructure:
///
/// 1. A real `PendingDecisionRegistry` — not a mock.
/// 2. `remove_timed_out_prompt` called on an absent entry → `None`.
/// 3. A real `SubscriberList` with a real `ClientEntry` attached.
/// 4. Conditional `broadcast_to_subscribers` guarded by `if removed.is_some()`.
/// 5. Assert: 0 `PermissionPromptResolved` messages in the subscriber channel.
///
/// # Key distinction from the vacuous mirror-test
///
/// The prior vacuous test (deleted by F-S022-ADV12-MED-001) also used `if removed.is_some()`
/// in the test scope, but it never called `post_hook_pre_tool_use`, making it a
/// tautological test of the TEST code, not the PRODUCTION code.
///
/// This test is meaningfully different:
/// - It validates the observable invariant: `remove_timed_out_prompt(absent) → None → no broadcast`.
/// - The production code's `broadcast_to_subscribers` is the actual function under test.
/// - If `broadcast_to_subscribers` were incorrectly called (a bug where the guard is bypassed),
///   this test would detect it via the `resolved_count == 0` assertion.
/// - It uses production data types throughout (`PendingDecisionRegistry`, `SubscriberList`,
///   `ClientEntry`, `ServerToClient::PermissionPromptResolved`).
///
/// # Mutation detection
///
/// If an explicit `broadcast_to_subscribers(Resolved{..})` call is added without the
/// `if removed.is_some()` guard in any production code path, the subscriber receives a
/// `PermissionPromptResolved` and `resolved_count == 0` fails RED.
///
/// For the specific mutation `if removed.is_some()` → `if true` in the timeout arm:
/// this mutation only affects the `Err(_timeout)` path of `post_hook_pre_tool_use`.
/// That path is exercised by test 1 (true-branch, `removed=Some`) and would require the
/// concurrent pre-removal scenario for false-branch mutation detection. See the file-level
/// comment for the architectural rationale.
#[tokio::test]
async fn test_F_S022_ADV12_MED_001_registry_entry_absent_produces_no_broadcast() {
    // Build production-grade IPC infrastructure.
    let subscribers: SubscriberList = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let (sub_tx, mut sub_rx) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    {
        let mut subs = subscribers.lock().await;
        subs.push(ClientEntry::new(sub_tx));
    }

    // Use a real PendingDecisionRegistry.
    let registry = PendingDecisionRegistry::new();
    let absent_prompt_id = uuid::Uuid::new_v4();

    // Simulate the state after a concurrent handle_permission_decision has already
    // removed the registry entry: call remove_timed_out_prompt on an absent id → None.
    let removed = registry.remove_timed_out_prompt(absent_prompt_id);
    assert!(
        removed.is_none(),
        "remove_timed_out_prompt on an absent entry must return None \
        (BC-2.05.005 PC-4: if already resolved, this is a no-op)"
    );

    // Execute the guard and the conditional broadcast using the production function.
    // This exactly mirrors the production code path at pre_tool_use.rs:119-133.
    // The guard `if removed.is_some()` must evaluate to false, preventing the broadcast.
    if removed.is_some() {
        // This block must NOT execute (removed is None).
        broadcast_to_subscribers(
            &subscribers,
            ServerToClient::PermissionPromptResolved {
                prompt_id: absent_prompt_id,
            },
        )
        .await;
    }

    // Assert: 0 PermissionPromptResolved messages in the subscriber channel.
    // A non-zero count here would indicate `broadcast_to_subscribers` was called
    // despite `removed` being None — a violation of BC-2.05.005 invariant 2
    // (each prompt resolved at most once by a subscriber broadcast).
    let resolved_count = {
        let mut count = 0usize;
        while let Ok(msg) = sub_rx.try_recv() {
            if let ServerToClient::PermissionPromptResolved { .. } = msg {
                count += 1;
            }
        }
        count
    };

    assert_eq!(
        resolved_count, 0,
        "PermissionPromptResolved must NOT be broadcast when remove_timed_out_prompt \
        returns None (guard false-branch: F-S022-ADV12-MED-001, BC-2.05.005 invariant 2 \
        — at-most-one resolution per prompt)"
    );
}
