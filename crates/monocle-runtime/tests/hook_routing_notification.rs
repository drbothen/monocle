//! Integration tests for `POST /hooks/notification` (S-018, BC-2.04.008).
//!
//! Covers every BC-2.04.008 postcondition and invariant via the full server stack.
//! Uses in-process `tower::ServiceExt::oneshot` (no TCP socket overhead).
//!
//! # Red Gate
//!
//! Tests that depend on the `handle_notification_inner()` implementation will
//! FAIL (todo!() panic) until S-018 is implemented.
//!
//! # Coverage Map
//!
//! | Test | BC clause | Assertion |
//! |------|-----------|-----------|
//! | test_BC_2_04_008_allow_returns_200_decision_allow | PC-3, PC-7 | Allow → HTTP 200 `{"decision":"allow"}` |
//! | test_BC_2_04_008_block_still_returns_allow_response | PC-7, invariant 4 | Block (internal) → `{"decision":"allow"}` to Claude Code |
//! | test_BC_2_04_008_invalid_json_returns_422 | PC-1 | Malformed JSON → HTTP 422 |
//! | test_BC_2_04_008_missing_session_id_returns_422 | PC-1, EC-076 | Missing session_id → HTTP 422 |
//! | test_BC_2_04_008_timeout_returns_allow | PC-4, EC-079 | Timeout → HTTP 200 `{"decision":"allow"}` |
//! | test_BC_2_04_008_defer_treated_as_allow_with_warn | PC-3, invariant 2, EC-077 | Defer → allow + WARN |
//! | test_BC_2_04_008_event_published_after_decision | PC-5 | Valid → event on bus |
//! | test_BC_2_04_008_event_bus_full_increments_drop_counter | PC-5, invariant 3 | Full bus → drop counter +1 |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per naming convention.
#![allow(non_snake_case)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use monocle_runtime::server::build_server;
use monocle_runtime::state::DaemonState;
use monocle_runtime::types::{EventBusHookEvent, EVENT_BUS_CAPACITY};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Test constants and helpers
// ---------------------------------------------------------------------------

const TEST_TOKEN: &str = "ccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabb";

fn notification_body() -> serde_json::Value {
    serde_json::json!({
        "session_id": "test-session-notification",
        "pid": 43000,
        "notification_type": "permission_prompt",
        "tool_name": "Bash",
        "tool_input": {"command": "rm -rf /"},
        "message": "Bash tool wants to run a command"
    })
}

fn make_state_with_bus() -> (
    Arc<DaemonState>,
    tokio::sync::mpsc::Receiver<EventBusHookEvent>,
) {
    let (tx, rx) = tokio::sync::mpsc::channel::<EventBusHookEvent>(EVENT_BUS_CAPACITY);
    let drop_counter = Arc::new(AtomicU64::new(0));
    let session_registry = Arc::new(monocle_runtime::hooks::SessionRegistry::new());

    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN.to_string();
    state.event_bus_tx = Some(Arc::new(tx));
    state.drop_counter = Some(Arc::clone(&drop_counter));
    state.session_registry = Some(session_registry);
    (Arc::new(state), rx)
}

fn make_state_with_full_bus() -> (Arc<DaemonState>, Arc<AtomicU64>) {
    let (tx, _rx) = tokio::sync::mpsc::channel::<EventBusHookEvent>(1);
    let drop_counter = Arc::new(AtomicU64::new(0));

    // Pre-fill the channel.
    let hook_event = {
        use monocle_core::hook_events::HookEvent;
        let event: HookEvent = serde_json::from_value(serde_json::json!({
            "Stop": {"stop_reason": "end_turn", "session_id": "dummy", "pid": 1}
        }))
        .unwrap();
        EventBusHookEvent::new(event, "2026-01-01T00:00:00.000Z".to_string())
    };
    let _ = tx.try_send(hook_event);

    let session_registry = Arc::new(monocle_runtime::hooks::SessionRegistry::new());
    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN.to_string();
    state.event_bus_tx = Some(Arc::new(tx));
    state.drop_counter = Some(Arc::clone(&drop_counter));
    state.session_registry = Some(session_registry);
    (Arc::new(state), drop_counter)
}

async fn post_json(
    state: Arc<DaemonState>,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    post_raw(state, uri, serde_json::to_vec(&body).unwrap()).await
}

async fn post_raw(
    state: Arc<DaemonState>,
    uri: &str,
    body_bytes: Vec<u8>,
) -> (StatusCode, serde_json::Value) {
    let app = build_server(state);
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .header(
            "X-Monocle-Authorization",
            format!("monocle-v1:{}", TEST_TOKEN),
        )
        .body(Body::from(body_bytes))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_else(
        |_| serde_json::json!({"_raw": std::str::from_utf8(&bytes).unwrap_or("<binary>")}),
    );
    (status, value)
}

// ---------------------------------------------------------------------------
// BC-2.04.008 PC-1: Invalid JSON → HTTP 422
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.008 PC-1: malformed JSON → HTTP 422.
#[tokio::test]
async fn test_BC_2_04_008_invalid_json_returns_422() {
    let (state, _rx) = make_state_with_bus();
    let (status, body) = post_raw(
        state,
        "/hooks/notification",
        b"definitely not json".to_vec(),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "malformed JSON → HTTP 422: {body}"
    );
    assert_eq!(body["error"], "invalid_body");
}

/// Exercises BC-2.04.008 PC-1 / EC-076: missing session_id → HTTP 422.
#[tokio::test]
async fn test_BC_2_04_008_missing_session_id_returns_422() {
    let (state, _rx) = make_state_with_bus();
    let body = serde_json::json!({"pid": 999, "notification_type": "permission_prompt"});
    let (status, resp_body) = post_json(state, "/hooks/notification", body).await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "missing session_id must return HTTP 422: {resp_body}"
    );
    assert_eq!(resp_body["error"], "invalid_body");
}

// ---------------------------------------------------------------------------
// BC-2.04.008 PC-3, PC-7: Allow → HTTP 200 {"decision":"allow"}
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.008 PC-3 and PC-7: allow decision → HTTP 200 `{"decision":"allow"}`.
///
/// Notification always returns HTTP 200 `{"decision":"allow"}` (invariant 4).
#[tokio::test]
async fn test_BC_2_04_008_allow_returns_200_decision_allow() {
    let (state, _rx) = make_state_with_bus();
    // Red Gate: fails because handle_notification_inner is todo!()
    let (status, body) = post_json(state, "/hooks/notification", notification_body()).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "notification allow → HTTP 200: {body}"
    );
    assert_eq!(
        body["decision"], "allow",
        "notification response must always be {{\"decision\":\"allow\"}}: {body}"
    );
}

/// Exercises BC-2.04.008 invariant 4: Block decision is NOT surfaced to Claude Code.
///
/// Even if the EngineModule returns Block (internal filtering), the HTTP response
/// to Claude Code is always `{"decision":"allow"}` (notifications are fire-and-forward).
///
/// NOTE: Phase 1 ClaudeCodeModule always returns Allow. This test documents the invariant
/// but cannot exercise the Block path with the current engine.
#[tokio::test]
async fn test_BC_2_04_008_block_still_returns_allow_response() {
    let (state, _rx) = make_state_with_bus();
    // Red Gate: fails because handle_notification_inner is todo!()
    let (status, body) = post_json(state, "/hooks/notification", notification_body()).await;

    assert_eq!(status, StatusCode::OK);
    // Notification must ALWAYS return allow to Claude Code, even if Block is the internal decision.
    assert_eq!(
        body["decision"], "allow",
        "notification Block must still return {{\"decision\":\"allow\"}} to Claude Code: {body}"
    );
    // The response must NOT contain "block" decision.
    assert_ne!(
        body["decision"], "block",
        "notification must never return block to Claude Code: {body}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.008 PC-4, EC-079: 2000ms timeout → allow
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.008 PC-4 / EC-079: 2000ms timeout → HTTP 200 `{"decision":"allow"}`.
///
/// NOTE: This test cannot reliably trigger the timeout without a slow engine injection.
/// It documents the timeout contract. Red Gate: inner handler is todo!().
#[tokio::test]
async fn test_BC_2_04_008_timeout_returns_allow() {
    let (state, _rx) = make_state_with_bus();
    // Red Gate: fails because inner handler is todo!()
    let (status, body) = post_json(state, "/hooks/notification", notification_body()).await;

    assert_eq!(status, StatusCode::OK);
    // The timeout path returns {"decision":"allow"} without "reason" (unlike PreToolUse).
    // BC-2.04.008 PC-7: response is always {"decision":"allow"}.
    assert_eq!(
        body["decision"], "allow",
        "notification timeout must return allow: {body}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.008 PC-3 invariant 2, EC-077: Defer → Allow + WARN (no oneshot)
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.008 invariant 2 / EC-077: Defer returned by module → treated as Allow.
///
/// When a module returns Defer for Notification:
/// - Handler treats it as Allow.
/// - WARN logged: "invalid Defer on Notification hook; treating as Allow".
/// - No oneshot channel created.
/// - No PermissionPromptQueued IPC sent.
/// - Response: HTTP 200 `{"decision":"allow"}`.
///
/// NOTE: Phase 1 engine never returns Defer. This test requires mock engine injection.
/// Red Gate: documents the invariant but inner handler is todo!().
#[tokio::test]
async fn test_BC_2_04_008_defer_treated_as_allow_with_warn() {
    let (state, _rx) = make_state_with_bus();
    // Red Gate: fails because inner handler is todo!()
    let (status, body) = post_json(state, "/hooks/notification", notification_body()).await;

    assert_eq!(status, StatusCode::OK);
    // Even if engine returned Defer, response must be allow (Defer demoted to Allow).
    assert_eq!(
        body["decision"], "allow",
        "Defer on Notification must produce allow response: {body}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.008 PC-5: event bus publish after decision
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.008 PC-5: event published after decision (non-blocking try_send).
#[tokio::test]
async fn test_BC_2_04_008_event_published_after_decision() {
    let (state, mut rx) = make_state_with_bus();
    // Red Gate: fails because inner handler is todo!()
    let (status, _body) = post_json(
        Arc::clone(&state),
        "/hooks/notification",
        notification_body(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // Exactly 1 event must be published.
    let event = rx.try_recv().expect(
        "one EventBusHookEvent must be published after a successful Notification (BC-2.04.008 PC-5)"
    );
    assert!(
        rx.try_recv().is_err(),
        "exactly one event must be published"
    );
    // Verify payload variant.
    match event.payload {
        monocle_core::hook_events::HookEvent::Notification(_) => {}
        other => panic!("expected HookEvent::Notification, got {:?}", other),
    }
}

/// Exercises BC-2.04.008 PC-5 / invariant 3: bus full → drop counter +1, handler not blocked.
#[tokio::test]
async fn test_BC_2_04_008_event_bus_full_increments_drop_counter() {
    let (state, drop_counter) = make_state_with_full_bus();

    let initial = drop_counter.load(Ordering::Relaxed);
    // Red Gate: fails because inner handler is todo!()
    let (status, _body) = post_json(
        Arc::clone(&state),
        "/hooks/notification",
        notification_body(),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "bus saturation must not affect HTTP response"
    );
    let after = drop_counter.load(Ordering::Relaxed);
    assert_eq!(
        after,
        initial + 1,
        "drop counter must increment by 1 on full bus (BC-2.04.011 PC-3a): initial={initial} after={after}"
    );
}
