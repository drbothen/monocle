//! Integration tests for `POST /hooks/stop`, `/session-start`, `/prompt-submit`
//! (S-018, BC-2.04.009).
//!
//! Covers all BC-2.04.009 postconditions and invariants via the full server stack.
//! Uses in-process `tower::ServiceExt::oneshot` (no TCP socket overhead).
//!
//! # Red Gate
//!
//! Tests that depend on inner handler implementations will FAIL (todo!() panic)
//! until S-018 implements those handlers.
//!
//! # Coverage Map
//!
//! | Test | BC clause | Assertion |
//! |------|-----------|-----------|
//! | test_BC_2_04_009_stop_returns_200_allow_session_marked_stopped | PC-2, PC-7, AC-016 | Stop → HTTP 200 allow + session = Stopped |
//! | test_BC_2_04_009_stop_unknown_session_creates_stopped_entry | EC-082 | Stop for unknown session → creates Stopped entry |
//! | test_BC_2_04_009_session_start_returns_200_allow | PC-7 | SessionStart → HTTP 200 allow |
//! | test_BC_2_04_009_prompt_submit_allow_returns_allow | PC-7 | PromptSubmit allow → allow |
//! | test_BC_2_04_009_prompt_submit_block_returns_block_with_reason | PC-7, AC-018 | PromptSubmit Block + diagnostic → block response |
//! | test_BC_2_04_009_prompt_submit_timeout_returns_allow | PC-4, EC-086 | PromptSubmit timeout → fail-open allow |
//! | test_BC_2_04_009_stop_invalid_json_returns_422 | PC-1 | Stop malformed JSON → HTTP 422 |
//! | test_BC_2_04_009_session_start_invalid_json_returns_422 | PC-1 | SessionStart malformed → HTTP 422 |
//! | test_BC_2_04_009_prompt_submit_invalid_json_returns_422 | PC-1 | PromptSubmit malformed → HTTP 422 |
//! | test_BC_2_04_009_defer_on_stop_treated_as_allow | PC-3, invariant 2, AC-017 | Stop Defer → allow + WARN |
//! | test_BC_2_04_009_defer_on_session_start_treated_as_allow | PC-3, AC-017, EC-085 | SessionStart Defer → allow + WARN |
//! | test_BC_2_04_009_event_published_after_stop | PC-5 | Stop → event on bus |
//! | test_BC_2_04_009_event_published_after_session_start | PC-5 | SessionStart → event on bus |
//! | test_BC_2_04_009_event_published_after_prompt_submit | PC-5 | PromptSubmit → event on bus |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per naming convention.
#![allow(non_snake_case)]

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use monocle_runtime::hooks::SessionState;
use monocle_runtime::server::build_server;
use monocle_runtime::state::DaemonState;
use monocle_runtime::types::{EventBusHookEvent, EVENT_BUS_CAPACITY};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Constants and helpers
// ---------------------------------------------------------------------------

const TEST_TOKEN: &str = "eeffeeffeeffeeffeeffeeffeeffeeffeeffeeffeeffeeffeeffeeffeeffaabb";

fn stop_body() -> serde_json::Value {
    serde_json::json!({
        "session_id": "test-session-stop",
        "pid": 44000,
        "stop_reason": "end_turn"
    })
}

fn session_start_body() -> serde_json::Value {
    serde_json::json!({
        "session_id": "test-session-start",
        "pid": 45000,
        "cwd": "/home/user/project",
        "transcript_path": "/home/user/.claude/transcripts/abc123.json"
    })
}

fn prompt_submit_body() -> serde_json::Value {
    serde_json::json!({
        "session_id": "test-session-prompt",
        "pid": 46000,
        "prompt": "Help me fix this bug"
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
    state.session_registry = Some(Arc::clone(&session_registry));
    (Arc::new(state), rx)
}

fn make_state_with_bus_and_registry() -> (
    Arc<DaemonState>,
    tokio::sync::mpsc::Receiver<EventBusHookEvent>,
    Arc<monocle_runtime::hooks::SessionRegistry>,
) {
    let (tx, rx) = tokio::sync::mpsc::channel::<EventBusHookEvent>(EVENT_BUS_CAPACITY);
    let drop_counter = Arc::new(AtomicU64::new(0));
    let session_registry = Arc::new(monocle_runtime::hooks::SessionRegistry::new());

    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN.to_string();
    state.event_bus_tx = Some(Arc::new(tx));
    state.drop_counter = Some(Arc::clone(&drop_counter));
    state.session_registry = Some(Arc::clone(&session_registry));
    (Arc::new(state), rx, session_registry)
}

/// Build a `DaemonState` with a Block decision override wired in.
///
/// Used by `test_BC_2_04_009_prompt_submit_block_returns_block_with_reason` to exercise
/// the Block path in `handle_prompt_submit_inner` without depending on a real engine
/// that returns Block (Phase 1 ClaudeCodeModule always returns Allow).
fn make_state_with_block_decision() -> (
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
    state.session_registry = Some(Arc::clone(&session_registry));
    state.hook_decision_override = Some((
        monocle_core::engine::HookDecision::Block,
        Some("policy-denied".to_owned()),
    ));
    (Arc::new(state), rx)
}

/// Build a `DaemonState` with a Defer decision override wired in.
///
/// Used to exercise the Defer-treated-as-Allow path in Stop and SessionStart handlers.
fn make_state_with_defer_decision() -> (
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
    state.session_registry = Some(Arc::clone(&session_registry));
    state.hook_decision_override = Some((monocle_core::engine::HookDecision::Defer, None));
    (Arc::new(state), rx)
}

/// Build a `DaemonState` with a 400ms artificial delay, reliably triggering the 300ms timeout.
fn make_state_with_slow_engine() -> (
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
    state.session_registry = Some(Arc::clone(&session_registry));
    // 400ms > 300ms timeout budget → reliably triggers the timeout path.
    state.hook_delay_ms = Some(400);
    (Arc::new(state), rx)
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
// BC-2.04.009 PC-1: Invalid JSON → HTTP 422 (all 3 endpoints)
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.009 PC-1 for Stop endpoint.
#[tokio::test]
async fn test_BC_2_04_009_stop_invalid_json_returns_422() {
    let (state, _rx) = make_state_with_bus();
    let (status, body) = post_raw(state, "/hooks/stop", b"not json".to_vec()).await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "Stop malformed JSON → HTTP 422: {body}"
    );
    assert_eq!(body["error"], "invalid_body");
}

/// Exercises BC-2.04.009 PC-1 for SessionStart endpoint.
#[tokio::test]
async fn test_BC_2_04_009_session_start_invalid_json_returns_422() {
    let (state, _rx) = make_state_with_bus();
    let (status, body) = post_raw(state, "/hooks/session-start", b"bad json".to_vec()).await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "SessionStart malformed JSON → HTTP 422: {body}"
    );
    assert_eq!(body["error"], "invalid_body");
}

/// Exercises BC-2.04.009 PC-1 for PromptSubmit endpoint.
#[tokio::test]
async fn test_BC_2_04_009_prompt_submit_invalid_json_returns_422() {
    let (state, _rx) = make_state_with_bus();
    let (status, body) = post_raw(state, "/hooks/prompt-submit", b"garbage".to_vec()).await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "PromptSubmit malformed JSON → HTTP 422: {body}"
    );
    assert_eq!(body["error"], "invalid_body");
}

// ---------------------------------------------------------------------------
// BC-2.04.009 PC-2, PC-7, AC-016: Stop → session marked Stopped
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.009 PC-2 / AC-016: Stop handler marks session as Stopped.
///
/// After `on_hook()` returns, the Stop handler must mark the session as
/// `SessionState::Stopped` in the registry. Response is always `{"decision":"allow"}`.
#[tokio::test]
async fn test_BC_2_04_009_stop_returns_200_allow_session_marked_stopped() {
    let (state, _rx, registry) = make_state_with_bus_and_registry();
    let session_id = "test-session-stop";

    // Red Gate: fails because handle_stop_inner is todo!()
    let (status, body) = post_json(Arc::clone(&state), "/hooks/stop", stop_body()).await;

    assert_eq!(status, StatusCode::OK, "Stop → HTTP 200: {body}");
    assert_eq!(
        body["decision"], "allow",
        "Stop response must always be allow: {body}"
    );

    // Session must be marked Stopped (BC-2.04.009 PC-2).
    let state_in_registry = registry.get_state(session_id);
    assert_eq!(
        state_in_registry,
        Some(SessionState::Stopped),
        "Stop handler must mark session as Stopped in registry (BC-2.04.009 PC-2)"
    );
}

/// Exercises BC-2.04.009 PC-2 / EC-082: Stop for unknown session → creates entry in Stopped state.
#[tokio::test]
async fn test_BC_2_04_009_stop_unknown_session_creates_stopped_entry() {
    let (state, _rx, registry) = make_state_with_bus_and_registry();
    let unknown_session_id = "never-seen-before-session";

    let body = serde_json::json!({
        "session_id": unknown_session_id,
        "pid": 99999,
        "stop_reason": "error"
    });

    // Red Gate: fails because handle_stop_inner is todo!()
    let (status, resp_body) = post_json(Arc::clone(&state), "/hooks/stop", body).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Stop for unknown session → HTTP 200: {resp_body}"
    );
    assert_eq!(resp_body["decision"], "allow");

    // Registry must have a Stopped entry for this previously-unknown session.
    let session_state = registry.get_state(unknown_session_id);
    assert_eq!(
        session_state,
        Some(SessionState::Stopped),
        "Stop for unknown session must create Stopped entry (EC-082)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.009 PC-7: SessionStart → always allow
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.009 PC-7 for SessionStart: always returns `{"decision":"allow"}`.
#[tokio::test]
async fn test_BC_2_04_009_session_start_returns_200_allow() {
    let (state, _rx) = make_state_with_bus();
    // Red Gate: fails because handle_session_start_inner is todo!()
    let (status, body) = post_json(state, "/hooks/session-start", session_start_body()).await;

    assert_eq!(status, StatusCode::OK, "SessionStart → HTTP 200: {body}");
    assert_eq!(
        body["decision"], "allow",
        "SessionStart must always return allow: {body}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.009 PC-7, AC-018: PromptSubmit allow/block/timeout
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.009 PC-7: PromptSubmit Allow → HTTP 200 `{"decision":"allow"}`.
#[tokio::test]
async fn test_BC_2_04_009_prompt_submit_allow_returns_allow() {
    let (state, _rx) = make_state_with_bus();
    // Red Gate: fails because handle_prompt_submit_inner is todo!()
    let (status, body) = post_json(state, "/hooks/prompt-submit", prompt_submit_body()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["decision"], "allow",
        "PromptSubmit allow → allow: {body}"
    );
}

/// Exercises BC-2.04.009 PC-7 / AC-018: PromptSubmit Block → `{"decision":"block","reason":"X"}`.
///
/// Block reason comes from `HookResponse.diagnostic`, not from an enum variant field.
/// `HookDecision::Block` is a unit variant (BC-2.04.009 invariant 4 / BC-2.04.007 F-P1D11-001).
///
/// NOTE: Phase 1 ClaudeCodeModule always returns Allow. Red Gate assertion.
#[tokio::test]
async fn test_BC_2_04_009_prompt_submit_block_returns_block_with_reason() {
    let (state, _rx) = make_state_with_block_decision();
    let (status, body) = post_json(state, "/hooks/prompt-submit", prompt_submit_body()).await;

    assert_eq!(status, StatusCode::OK);
    // Red Gate: expects block, Phase 1 engine returns allow.
    assert_eq!(
        body["decision"], "block",
        "PromptSubmit Block must return {{\"decision\":\"block\"}}: {body}"
    );
    assert!(
        body["reason"].is_string(),
        "Block response must include reason from HookResponse.diagnostic: {body}"
    );
}

/// Exercises BC-2.04.009 PC-4 / EC-086: PromptSubmit timeout → fail-open allow.
///
/// Injects a 400ms artificial delay (> 300ms budget) to reliably trigger the timeout path.
/// NOTE: This test takes ~300ms real time (waits for the timeout budget to expire).
#[tokio::test]
async fn test_BC_2_04_009_prompt_submit_timeout_returns_allow() {
    let (state, _rx) = make_state_with_slow_engine();
    let (status, body) = post_json(state, "/hooks/prompt-submit", prompt_submit_body()).await;

    assert_eq!(status, StatusCode::OK);
    // Timeout produces {"decision":"allow"} without "reason":"timeout" (unlike PreToolUse).
    // BC-2.04.009 PC-7 / EC-086.
    assert_eq!(
        body["decision"], "allow",
        "PromptSubmit timeout must produce allow (fail-open): {body}"
    );
}

/// Exercises BC-2.04.009 PC-4: Stop timeout → fail-open allow.
///
/// Injects a 400ms artificial delay (> 300ms budget) to reliably trigger the timeout path.
/// NOTE: This test takes ~300ms real time (waits for the timeout budget to expire).
#[tokio::test]
async fn test_BC_2_04_009_stop_timeout_returns_allow() {
    let (state, _rx) = make_state_with_slow_engine();
    let (status, body) = post_json(state, "/hooks/stop", stop_body()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["decision"], "allow",
        "Stop timeout must produce allow (fail-open): {body}"
    );
}

/// Exercises BC-2.04.009 PC-4: SessionStart timeout → fail-open allow.
///
/// Injects a 400ms artificial delay (> 300ms budget) to reliably trigger the timeout path.
/// NOTE: This test takes ~300ms real time (waits for the timeout budget to expire).
#[tokio::test]
async fn test_BC_2_04_009_session_start_timeout_returns_allow() {
    let (state, _rx) = make_state_with_slow_engine();
    let (status, body) = post_json(state, "/hooks/session-start", session_start_body()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["decision"], "allow",
        "SessionStart timeout must produce allow (fail-open): {body}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.009 PC-3, invariant 2, AC-017: Defer → Allow + WARN
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.009 invariant 2 / AC-017: Defer on Stop → treated as Allow + WARN.
///
/// Injects a Defer decision override to exercise the code path directly.
/// No oneshot channel created; no PermissionPromptQueued IPC sent.
#[tokio::test]
async fn test_BC_2_04_009_defer_on_stop_treated_as_allow() {
    let (state, _rx) = make_state_with_defer_decision();
    let (status, body) = post_json(state, "/hooks/stop", stop_body()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["decision"], "allow",
        "Defer on Stop must produce allow (AC-017): {body}"
    );
}

/// Exercises BC-2.04.009 invariant 2 / AC-017 / EC-085: Defer on SessionStart → Allow + WARN.
///
/// Injects a Defer decision override to exercise the code path directly.
#[tokio::test]
async fn test_BC_2_04_009_defer_on_session_start_treated_as_allow() {
    let (state, _rx) = make_state_with_defer_decision();
    let (status, body) = post_json(state, "/hooks/session-start", session_start_body()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["decision"], "allow",
        "Defer on SessionStart must produce allow (AC-017, EC-085): {body}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.009 PC-5: event published after decision
// ---------------------------------------------------------------------------

/// Exercises BC-2.04.009 PC-5: Stop event published after decision.
#[tokio::test]
async fn test_BC_2_04_009_event_published_after_stop() {
    let (state, mut rx) = make_state_with_bus();
    // Red Gate: fails because inner handler is todo!()
    let (status, _body) = post_json(Arc::clone(&state), "/hooks/stop", stop_body()).await;

    assert_eq!(status, StatusCode::OK);
    let event = rx
        .try_recv()
        .expect("one event must be published after Stop (BC-2.04.009 PC-5)");
    assert!(
        rx.try_recv().is_err(),
        "exactly one event must be published"
    );
    match event.payload {
        monocle_core::hook_events::HookEvent::Stop(_) => {}
        other => panic!("expected HookEvent::Stop, got {:?}", other),
    }
}

/// Exercises BC-2.04.009 PC-5: SessionStart event published after decision.
#[tokio::test]
async fn test_BC_2_04_009_event_published_after_session_start() {
    let (state, mut rx) = make_state_with_bus();
    // Red Gate: fails because inner handler is todo!()
    let (status, _body) = post_json(
        Arc::clone(&state),
        "/hooks/session-start",
        session_start_body(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let event = rx
        .try_recv()
        .expect("one event must be published after SessionStart (BC-2.04.009 PC-5)");
    assert!(
        rx.try_recv().is_err(),
        "exactly one event must be published"
    );
    match event.payload {
        monocle_core::hook_events::HookEvent::SessionStart(_) => {}
        other => panic!("expected HookEvent::SessionStart, got {:?}", other),
    }
}

/// Exercises BC-2.04.009 PC-5: PromptSubmit event published after decision.
#[tokio::test]
async fn test_BC_2_04_009_event_published_after_prompt_submit() {
    let (state, mut rx) = make_state_with_bus();
    // Red Gate: fails because inner handler is todo!()
    let (status, _body) = post_json(
        Arc::clone(&state),
        "/hooks/prompt-submit",
        prompt_submit_body(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let event = rx
        .try_recv()
        .expect("one event must be published after PromptSubmit (BC-2.04.009 PC-5)");
    assert!(
        rx.try_recv().is_err(),
        "exactly one event must be published"
    );
    match event.payload {
        monocle_core::hook_events::HookEvent::UserPromptSubmit(_) => {}
        other => panic!("expected HookEvent::UserPromptSubmit, got {:?}", other),
    }
}
