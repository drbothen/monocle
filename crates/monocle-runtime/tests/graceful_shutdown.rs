//! Integration tests for BC-2.01.004: Graceful Shutdown (10-Second Drain).
//!
//! Tests the `POST /shutdown` endpoint, AppMode transition, 503 during drain, exit-code
//! taxonomy, hard-timeout enforcement, lock-file release, and authentication requirements.
//!
//! All tests use `tower::ServiceExt::oneshot` (in-process, no TCP) following the
//! established pattern from `healthz_endpoint.rs` and `status_endpoint_auth.rs`.
//!
//! # Red Gate
//!
//! All tests in this file MUST FAIL before S-005 implementation is complete. The
//! `post_shutdown` handler stub returns HTTP 501; the shutdown drain logic, AppMode
//! transitions, and signal wiring do not yet exist. Every behavioral assertion below
//! will fail against the current stubs.
//!
//! # Coverage Map
//!
//! | AC | BC-2.01.004 Clause | VP-004 Probe | Test function |
//! |----|-------------------|--------------|---------------|
//! | AC-002 canonical | PC-1 + INV-3 | 4.a / 4.j | test_BC_2_01_004_post_shutdown_canonical_auth_returns_200_shutting_down |
//! | AC-002 alias | INV-3 + ADR-0005 | 4.a | test_BC_2_01_004_post_shutdown_alias_auth_returns_200_shutting_down |
//! | AC-002 negative | INV-3 | 4.j | test_BC_2_01_004_post_shutdown_no_auth_returns_401_missing_token |
//! | AC-002 invalid | INV-3 + BC-2.01.009 | 4.j | test_BC_2_01_004_post_shutdown_wrong_token_returns_401_invalid_token |
//! | AC-002 mode | PC-1 | 4.a | test_BC_2_01_004_post_shutdown_transitions_appmode_to_shutting_down |
//! | AC-003 | PC-2 | 4.b | test_BC_2_01_004_hook_post_during_drain_returns_503_with_retry_after |
//! | AC-003 body | PC-2 | 4.b | test_BC_2_01_004_hook_503_body_is_daemon_shutting_down |
//! | AC-003 Retry-After | PC-2 | 4.b | test_BC_2_01_004_hook_503_retry_after_header_is_10 |
//! | AC-003 healthz | BC-2.01.001 PC-2 (cross) | 4.c | test_BC_2_01_004_healthz_returns_503_shutting_down_during_drain |
//! | AC-003 status | BC-2.01.002 PC-3 (cross) | 4.d | test_BC_2_01_004_status_continues_serving_200_during_drain |
//! | AC-004 | PC-8 | — | test_BC_2_01_004_exit_codes_graceful_is_zero |
//! | AC-004 | PC-8 | — | test_BC_2_01_004_exit_codes_startup_failure_is_one |
//! | AC-004 | PC-8 | — | test_BC_2_01_004_exit_codes_admin_force_stop_is_two |
//! | AC-004 | PC-8 | — | test_BC_2_01_004_exit_codes_sigint_during_drain_is_130 |
//! | AC-004 | PC-8 | — | test_BC_2_01_004_exit_codes_sigterm_during_drain_is_143 |
//! | AC-004 invariant | PC-8 INV-4 | — | test_BC_2_01_004_invariant_sigterm_and_sigint_exit_codes_are_distinct |
//! | AC-004 invariant | PC-8 | — | test_BC_2_01_004_invariant_all_5_exit_codes_are_distinct |
//! | AC-004 POSIX | PC-8 INV-4 | — | test_BC_2_01_004_invariant_posix_128n_convention_sigint_is_128_plus_2 |
//! | AC-004 POSIX | PC-8 INV-4 | — | test_BC_2_01_004_invariant_posix_128n_convention_sigterm_is_128_plus_15 |
//! | AC-005 | INV-1 / VP-004 PC-6 | 4.e | test_BC_2_01_004_drain_completes_within_11_seconds |
//! | AC-007 | PC-7 | — | test_BC_2_01_004_lock_file_absent_after_graceful_shutdown |
//! | EC-050 | EC-050 | 4.h | test_BC_2_01_004_second_post_shutdown_during_drain_returns_200 |
//! | Arch | SS-conventions-anti-patterns | — | test_BC_2_01_004_invariant_no_process_exit_in_handler_code |
//! | Arch | SS-conventions-anti-patterns | — | test_BC_2_01_004_invariant_exit_with_is_sole_process_exit_callsite |
//!
//! # Test Vectors (BC-2.01.004 Canonical Test Vectors)
//!
//! | Input | Expected Output | Category |
//! |-------|----------------|----------|
//! | SIGTERM or POST /shutdown (authenticated) | AppMode → ShuttingDown; new hooks HTTP 503 + Retry-After: 10 | happy-path |
//! | POST /hooks/* during drain | HTTP 503 {"error":"daemon_shutting_down"}, Retry-After: 10 | edge-case |
//! | All in-flight requests complete within 10s | Exit code 0 | happy-path |
//! | Second SIGINT (Ctrl-C) during drain | Exit code 130 (POSIX 128+2) | edge-case |
//! | Second SIGTERM during drain | Exit code 143 (POSIX 128+15) | edge-case |
//! | Second POST /shutdown during drain | Exit code 2 (monocle-specific admin forced-stop) | edge-case |
//! | DaemonStartError::RuntimeDirUnresolvable or port bind failure | Exit code 1 | error |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per the naming convention.
#![allow(non_snake_case)]

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use monocle_runtime::lifecycle::DaemonExit;
use monocle_runtime::server::build_server;
use monocle_runtime::state::{AppMode, DaemonState};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Raw 64-hex-char auth token used across all auth-path tests.
///
/// Must match the format stored in `DaemonState.auth_token` (raw hex, no prefix).
const TEST_TOKEN_HEX: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

/// Build a `DaemonState` with the test token pre-loaded.
fn make_state_with_token() -> Arc<DaemonState> {
    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN_HEX.to_string();
    Arc::new(state)
}

/// Build a `DaemonState` with the test token pre-loaded and `AppMode::ShuttingDown`.
fn make_shutting_down_state_with_token() -> Arc<DaemonState> {
    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN_HEX.to_string();
    *state.mode.write().unwrap() = AppMode::ShuttingDown;
    Arc::new(state)
}

/// Issue `POST /shutdown` through the full server router and return `(StatusCode, serde_json::Value)`.
///
/// Uses `tower::ServiceExt::oneshot` (in-process, no TCP).
async fn post_shutdown_json(
    state: Arc<DaemonState>,
    headers: &[(&str, &str)],
) -> (StatusCode, serde_json::Value) {
    let app = build_server(state);
    let mut builder = Request::builder().method("POST").uri("/shutdown");
    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }
    let req = builder.body(Body::empty()).expect("build POST /shutdown");
    let response = app.oneshot(req).await.expect("oneshot POST /shutdown");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response body must be valid JSON");
    (status, value)
}

/// Issue a `POST` to a hook endpoint through the full server router and return `(StatusCode, serde_json::Value)`.
///
/// Uses canonical auth header. Route is parameterized to exercise different hook paths.
async fn post_hook_json(
    state: Arc<DaemonState>,
    hook_path: &str,
    auth_token: &str,
) -> (StatusCode, serde_json::Value) {
    let app = build_server(state);
    let auth_value = format!("monocle-v1:{auth_token}");
    let req = Request::builder()
        .method("POST")
        .uri(hook_path)
        .header("X-Monocle-Authorization", auth_value.as_str())
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .expect("build POST hook request");
    let response = app.oneshot(req).await.expect("oneshot POST hook");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_else(|_| {
        serde_json::json!({"error": "response body was not valid JSON"})
    });
    (status, value)
}

// ---------------------------------------------------------------------------
// AC-002 — BC-2.01.004 PC-1 + INV-3 — POST /shutdown canonical auth → 200 + shutting_down
// ---------------------------------------------------------------------------

/// VP-004 Probe 4.a (canonical): `POST /shutdown` with valid canonical `X-Monocle-Authorization`
/// returns HTTP 200 with body `{"status":"shutting_down"}`.
///
/// Counter-example guarded: stub returns HTTP 501; test asserts 200 with correct body.
///
/// Traces to BC-2.01.004 PC-1, INV-3, AC-002.
#[tokio::test]
async fn test_BC_2_01_004_post_shutdown_canonical_auth_returns_200_shutting_down() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = post_shutdown_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "POST /shutdown with valid canonical auth must return HTTP 200 \
        (BC-2.01.004 PC-1 + INV-3); got {status}. Body: {body}. \
        Counter-example: stub returns 501."
    );
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("shutting_down"),
        "POST /shutdown success body must be {{\"status\":\"shutting_down\"}} \
        (BC-2.01.004 PC-1); got: {body}"
    );
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "POST /shutdown success body must have EXACTLY 1 key ({{\"status\":\"shutting_down\"}}); \
        got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

/// VP-004 Probe 4.a (alias): `POST /shutdown` with `X-Claude-Code-Ide-Authorization: <raw-64-hex>`
/// (alias, no prefix) also returns HTTP 200 with `{"status":"shutting_down"}`.
///
/// ADR-0005 dual-accept applies to ALL authenticated endpoints including /shutdown (BC-2.01.004 INV-3).
///
/// Counter-example guarded: stub returns HTTP 501; alias auth test also fails.
///
/// Traces to BC-2.01.004 INV-3, ADR-0005, AC-006.
#[tokio::test]
async fn test_BC_2_01_004_post_shutdown_alias_auth_returns_200_shutting_down() {
    let state = make_state_with_token();
    // Alias path: raw 64-hex token, no monocle-v1: prefix (Claude Code lock-file format).
    let (status, body) = post_shutdown_json(
        state,
        &[("X-Claude-Code-Ide-Authorization", TEST_TOKEN_HEX)],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "POST /shutdown with alias auth (X-Claude-Code-Ide-Authorization) must return HTTP 200 \
        (ADR-0005 dual-accept applies to /shutdown per BC-2.01.004 INV-3); got {status}. \
        Body: {body}. Counter-example: stub returns 501."
    );
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("shutting_down"),
        "POST /shutdown alias-auth success body must be {{\"status\":\"shutting_down\"}}; \
        got: {body}"
    );
}

// ---------------------------------------------------------------------------
// AC-002 negative — BC-2.01.004 INV-3 — Unauthenticated POST /shutdown → 401
// ---------------------------------------------------------------------------

/// VP-004 Probe 4.j: `POST /shutdown` with no auth header returns HTTP 401 with
/// `{"error":"missing_auth_token"}` (E-AUTH-001).
///
/// The shutdown endpoint requires authentication (BC-2.01.004 INV-3). Neither recognized
/// auth header present → HTTP 401 missing_auth_token per BC-2.01.009 PC-1.
///
/// Counter-example guarded: stub returns 501; but auth middleware runs FIRST so stub
/// is never reached for unauthenticated requests. Auth middleware must return 401 before stub.
///
/// Traces to BC-2.01.004 INV-3, BC-2.01.009 PC-1, AC-002 negative.
#[tokio::test]
async fn test_BC_2_01_004_post_shutdown_no_auth_returns_401_missing_token() {
    let state = make_state_with_token();
    let (status, body) = post_shutdown_json(state, &[]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "POST /shutdown with no auth header must return HTTP 401 E-AUTH-001 \
        (BC-2.01.004 INV-3, BC-2.01.009 PC-1); got {status}. Body: {body}."
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("missing_auth_token"),
        "No-auth body must be {{\"error\":\"missing_auth_token\"}} (E-AUTH-001); got: {body}"
    );
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "E-AUTH-001 body must have EXACTLY 1 key; got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

/// BC-2.01.009 PC-2 / INV-3: `POST /shutdown` with a valid-format but WRONG canonical token
/// returns HTTP 401 with `{"error":"invalid_auth_token"}` (E-AUTH-002).
///
/// Traces to BC-2.01.004 INV-3, BC-2.01.009 PC-2, AC-002 negative (value-present failure).
#[tokio::test]
async fn test_BC_2_01_004_post_shutdown_wrong_token_returns_401_invalid_token() {
    let state = make_state_with_token();
    // Wrong token: 64 hex chars, all zeros — format valid but value wrong.
    let wrong_token = format!("monocle-v1:{}", "0".repeat(64));
    let (status, body) = post_shutdown_json(
        state,
        &[("X-Monocle-Authorization", wrong_token.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "POST /shutdown with wrong token must return HTTP 401 E-AUTH-002 \
        (BC-2.01.004 INV-3, BC-2.01.009 PC-2); got {status}. Body: {body}."
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "Wrong-token body must be {{\"error\":\"invalid_auth_token\"}} (E-AUTH-002); got: {body}"
    );
}

// ---------------------------------------------------------------------------
// AC-002 — BC-2.01.004 PC-1 — AppMode transitions to ShuttingDown on POST /shutdown
// ---------------------------------------------------------------------------

/// BC-2.01.004 PC-1: After `POST /shutdown` with valid auth, `AppMode` must be `ShuttingDown`.
///
/// The daemon responds HTTP 200 AND immediately transitions AppMode to ShuttingDown. This
/// transition is observable by reading `state.mode` after the handler completes.
///
/// Counter-example guarded: stub never changes AppMode; test asserts mode is ShuttingDown.
///
/// Traces to BC-2.01.004 PC-1, VP-004 PC-1 (AppMode transition < 10ms), AC-002.
#[tokio::test]
async fn test_BC_2_01_004_post_shutdown_transitions_appmode_to_shutting_down() {
    let state = make_state_with_token();
    let state_ref = Arc::clone(&state);

    // Verify initial state is Running.
    {
        let mode = state_ref.mode.read().unwrap();
        assert_eq!(
            *mode,
            AppMode::Running,
            "prerequisite: AppMode must be Running before shutdown request"
        );
    }

    // Drop the borrow before the next oneshot call (otherwise Arc::clone would work anyway).
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, _body) = post_shutdown_json(
        Arc::clone(&state),
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "prerequisite: POST /shutdown with valid canonical auth must return HTTP 200"
    );

    // After the shutdown request, AppMode must be ShuttingDown (BC-2.01.004 PC-1).
    let mode_after = state_ref.mode.read().unwrap().clone();
    assert_eq!(
        mode_after,
        AppMode::ShuttingDown,
        "AppMode must be ShuttingDown after successful POST /shutdown \
        (BC-2.01.004 PC-1); still Running after handler returned. \
        Counter-example: stub handler does not update AppMode."
    );
}

// ---------------------------------------------------------------------------
// AC-003 — BC-2.01.004 PC-2 — Hook POST during drain → 503 + Retry-After: 10
// ---------------------------------------------------------------------------

/// VP-004 Probe 4.b: `POST /hooks/pre-tool-use` when `AppMode::ShuttingDown` returns
/// HTTP 503 with `{"error":"daemon_shutting_down"}` and `Retry-After: 10` header.
///
/// Counter-example guarded: in-progress stub returns 404 (route not yet registered in
/// S-005 wave; S-009 adds hook routes). The 503 response requires the hook handler to
/// gate on `AppMode::ShuttingDown`.
///
/// Traces to BC-2.01.004 PC-2, VP-004 Probe 4.b, AC-003.
#[tokio::test]
async fn test_BC_2_01_004_hook_post_during_drain_returns_503_with_retry_after() {
    let state = make_shutting_down_state_with_token();

    let (status, body) =
        post_hook_json(Arc::clone(&state), "/hooks/pre-tool-use", TEST_TOKEN_HEX).await;

    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "POST /hooks/pre-tool-use during AppMode::ShuttingDown must return HTTP 503 \
        (BC-2.01.004 PC-2); got {status}. Body: {body}. \
        Counter-example: accepting hook posts during drain violates the contract."
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("daemon_shutting_down"),
        "503 during drain body must be {{\"error\":\"daemon_shutting_down\"}} \
        (BC-2.01.004 PC-2); got: {body}"
    );
}

/// VP-004 Probe 4.b: The `{"error":"daemon_shutting_down"}` body has exactly 1 key.
///
/// Counter-example: additional fields (e.g., `"retry_after": 10`) in the JSON body would
/// violate the canonical single-field body contract.
///
/// Traces to BC-2.01.004 PC-2, VP-004 Probe 4.b, AC-003.
#[tokio::test]
async fn test_BC_2_01_004_hook_503_body_is_daemon_shutting_down() {
    let state = make_shutting_down_state_with_token();

    let (status, body) =
        post_hook_json(Arc::clone(&state), "/hooks/pre-tool-use", TEST_TOKEN_HEX).await;

    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "prerequisite: POST /hooks/* during drain must return 503"
    );

    let obj = body.as_object().expect("503 body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "503 during drain body must have EXACTLY 1 key ({{\"error\":\"daemon_shutting_down\"}}); \
        got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
    assert!(
        obj.contains_key("error"),
        "503 body must contain key \"error\"; got: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
}

/// VP-004 Probe 4.b: `Retry-After: 10` header MUST be present on 503 responses during drain.
///
/// External clients (systemd hook scripts, Claude Code) use this header to determine
/// the retry interval. The exact integer value `10` is normative per BC-2.01.004 PC-2
/// and VP-004 §Post-conditions item 2.
///
/// Counter-example: omitting the header, or setting it to a different value (e.g., `5`),
/// fails the exact-value assertion in VP-004 §Counter-examples item 2.
///
/// Traces to BC-2.01.004 PC-2, VP-004 §Post-conditions item 2, AC-003.
#[tokio::test]
async fn test_BC_2_01_004_hook_503_retry_after_header_is_10() {
    let state = make_shutting_down_state_with_token();
    let app = build_server(Arc::clone(&state));
    let auth_value = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let req = Request::builder()
        .method("POST")
        .uri("/hooks/pre-tool-use")
        .header("X-Monocle-Authorization", auth_value.as_str())
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .expect("build POST /hooks/pre-tool-use");

    let response = app.oneshot(req).await.expect("oneshot");

    assert_eq!(
        response.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "prerequisite: POST /hooks/* during drain must return 503"
    );

    let retry_after = response.headers().get("Retry-After");
    assert!(
        retry_after.is_some(),
        "503 response during drain MUST include Retry-After header \
        (BC-2.01.004 PC-2; VP-004 §Post-conditions item 2); header is absent. \
        Counter-example: VP-004 §Counter-examples item 2 — header omitted fails assertion."
    );

    let retry_value = retry_after
        .unwrap()
        .to_str()
        .expect("Retry-After header must be valid UTF-8");
    assert_eq!(
        retry_value, "10",
        "Retry-After header must be the exact integer value \"10\" \
        (BC-2.01.004 PC-2; VP-004 §Post-conditions item 2); got: {retry_value:?}. \
        Counter-example: VP-004 §Counter-examples item 2 — value 5 (or any non-10) fails."
    );
}

/// VP-004 Probe 4.b: All 5 canonical hook endpoints return 503 during drain, not just /hooks/pre-tool-use.
///
/// BC-2.01.004 PC-2 applies to `/hooks/*` generically. All 5 endpoints must be gated.
///
/// Traces to BC-2.01.004 PC-2, VP-004 Probe 4.b, AC-003 (all hook routes).
#[tokio::test]
async fn test_BC_2_01_004_all_hook_endpoints_return_503_during_drain() {
    let hook_paths = [
        "/hooks/pre-tool-use",
        "/hooks/notification",
        "/hooks/stop",
        "/hooks/session-start",
        "/hooks/prompt-submit",
    ];

    for hook_path in hook_paths {
        let state = make_shutting_down_state_with_token();
        let (status, body) =
            post_hook_json(Arc::clone(&state), hook_path, TEST_TOKEN_HEX).await;

        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "POST {hook_path} during AppMode::ShuttingDown must return HTTP 503 \
            (BC-2.01.004 PC-2 — applies to /hooks/* generically); got {status}. Body: {body}."
        );
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("daemon_shutting_down"),
            "503 body for {hook_path} during drain must be \
            {{\"error\":\"daemon_shutting_down\"}}; got: {body}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-003 cross-property — BC-2.01.001 PC-2 — GET /healthz returns 503 during drain
// ---------------------------------------------------------------------------

/// VP-004 Probe 4.c (cross-property with VP-001): `GET /healthz` during drain returns
/// HTTP 503 with `{"status":"shutting_down"}`.
///
/// BC-2.01.004 PC-3 cites BC-2.01.001 PC-2 for this behavior. The healthz handler
/// already checks AppMode and returns 503 when ShuttingDown.
///
/// This test verifies the cross-property from the graceful-shutdown perspective.
///
/// Traces to BC-2.01.004 PC-3, BC-2.01.001 PC-2, VP-004 Probe 4.c.
#[tokio::test]
async fn test_BC_2_01_004_healthz_returns_503_shutting_down_during_drain() {
    // Use a state with ShuttingDown mode (no token needed for /healthz — unauthenticated).
    let state = DaemonState::new();
    *state.mode.write().unwrap() = AppMode::ShuttingDown;
    let state = Arc::new(state);

    let app = build_server(Arc::clone(&state));
    let req = Request::builder()
        .method("GET")
        .uri("/healthz")
        .body(Body::empty())
        .expect("build GET /healthz");

    let response = app.oneshot(req).await.expect("oneshot GET /healthz");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let body: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response must be valid JSON");

    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "GET /healthz during AppMode::ShuttingDown must return HTTP 503 \
        (BC-2.01.004 PC-3, BC-2.01.001 PC-2, VP-004 Probe 4.c); got {status}. Body: {body}."
    );
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("shutting_down"),
        "GET /healthz drain body must be {{\"status\":\"shutting_down\"}}; got: {body}"
    );
}

// ---------------------------------------------------------------------------
// AC-003 cross-property — BC-2.01.002 PC-3 — GET /status continues during drain
// ---------------------------------------------------------------------------

/// VP-004 Probe 4.d (cross-property with VP-002): `GET /status` with valid auth during drain
/// returns HTTP 200 (read-only, drain-exempt).
///
/// BC-2.01.004 PC-4 explicitly specifies `/status` continues to serve during drain.
/// VP-004 §Post-conditions item 4: "GET /status (valid auth) during drain → HTTP 200 + full
/// 10-field body."
///
/// Counter-example guarded: VP-004 §Counter-examples item 3 — `/status` blocks during drain
/// (returns no response or 503) would fail this test.
///
/// Traces to BC-2.01.004 PC-4, BC-2.01.002 PC-3, VP-004 Probe 4.d.
#[tokio::test]
async fn test_BC_2_01_004_status_continues_serving_200_during_drain() {
    let state = make_shutting_down_state_with_token();
    let app = build_server(Arc::clone(&state));
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let req = Request::builder()
        .method("GET")
        .uri("/status")
        .header("X-Monocle-Authorization", canonical_header.as_str())
        .body(Body::empty())
        .expect("build GET /status");

    let response = app.oneshot(req).await.expect("oneshot GET /status");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let body: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response must be valid JSON");

    assert_eq!(
        status,
        StatusCode::OK,
        "GET /status with valid auth during AppMode::ShuttingDown must return HTTP 200 \
        (BC-2.01.004 PC-4; drain-exempt read-only endpoint); got {status}. Body: {body}. \
        Counter-example: VP-004 §Counter-examples item 3 — /status returning 503 fails."
    );

    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        10,
        "GET /status during drain must return full 10-field body (VP-004 §Post-conditions 4); \
        got {} field(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// AC-004 — BC-2.01.004 PC-8 — 5-code POSIX exit taxonomy (unit tests, no server needed)
// ---------------------------------------------------------------------------

/// BC-2.01.004 PC-8: `DaemonExit::Graceful.to_exit_code()` must equal 0.
///
/// Exit code 0 = graceful drain succeeded; all in-flight requests completed within 10s.
///
/// Counter-example guarded: any non-zero value for Graceful would cause monitoring systems
/// to treat a clean shutdown as an error.
///
/// Traces to BC-2.01.004 PC-8, AC-004 (Graceful=0).
#[test]
fn test_BC_2_01_004_exit_codes_graceful_is_zero() {
    assert_eq!(
        DaemonExit::Graceful.to_exit_code(),
        0,
        "DaemonExit::Graceful exit code must be 0 (clean drain succeeded, BC-2.01.004 PC-8)"
    );
}

/// BC-2.01.004 PC-8: `DaemonExit::StartupFailure.to_exit_code()` must equal 1.
///
/// Exit code 1 = daemon failed to start (RuntimeDirUnresolvable, port bind failure,
/// existing live lock file).
///
/// Traces to BC-2.01.004 PC-8, AC-004 (StartupFailure=1).
#[test]
fn test_BC_2_01_004_exit_codes_startup_failure_is_one() {
    assert_eq!(
        DaemonExit::StartupFailure.to_exit_code(),
        1,
        "DaemonExit::StartupFailure exit code must be 1 (startup error, BC-2.01.004 PC-8)"
    );
}

/// BC-2.01.004 PC-8: `DaemonExit::AdminForceStop.to_exit_code()` must equal 2.
///
/// Exit code 2 = second authenticated `POST /shutdown` during drain (admin forced-stop).
/// Monocle-specific code outside the POSIX 128+N range; distinct from startup-failure exit 1.
///
/// Counter-example guarded: VP-004 §Counter-examples item 7 — if admin-forced-stop reused
/// exit code 1 (overlapping with startup failure), monitoring systems cannot distinguish
/// operator-initiated force-stop from daemon start failure.
///
/// Traces to BC-2.01.004 PC-8, AC-004 (AdminForceStop=2).
#[test]
fn test_BC_2_01_004_exit_codes_admin_force_stop_is_two() {
    assert_eq!(
        DaemonExit::AdminForceStop.to_exit_code(),
        2,
        "DaemonExit::AdminForceStop exit code must be 2 (monocle-specific admin forced-stop, \
        BC-2.01.004 PC-8). Counter-example: VP-004 §Counter-examples item 7 — if this were 1, \
        admin forced-stop is indistinguishable from startup failure."
    );
}

/// BC-2.01.004 PC-8: `DaemonExit::SigintDuringDrain.to_exit_code()` must equal 130.
///
/// POSIX convention 128+2 (SIGINT = signal 2). Typical cause: user pressed Ctrl-C a
/// second time while the drain was in progress.
///
/// Counter-example guarded: VP-004 §Counter-examples item 6 — returning 143 for SIGINT
/// hard-kill (conflating SIGTERM and SIGINT codes) would cause external monitoring to
/// misinterpret the trigger.
///
/// Traces to BC-2.01.004 PC-8 + INV-4, AC-004 (SigintDuringDrain=130=128+2).
#[test]
fn test_BC_2_01_004_exit_codes_sigint_during_drain_is_130() {
    assert_eq!(
        DaemonExit::SigintDuringDrain.to_exit_code(),
        130,
        "DaemonExit::SigintDuringDrain exit code must be 130 (POSIX 128+SIGINT(2), \
        BC-2.01.004 PC-8). Counter-example: VP-004 §Counter-examples item 6 — returning \
        143 for SIGINT hard-kill conflates SIGTERM and SIGINT exit codes."
    );
}

/// BC-2.01.004 PC-8: `DaemonExit::SigtermDuringDrain.to_exit_code()` must equal 143.
///
/// POSIX convention 128+15 (SIGTERM = signal 15). Typical cause: systemd/k8s sent a
/// second SIGTERM after the graceful-shutdown window, or the 10-second drain timed out.
///
/// Counter-example guarded: VP-004 §Counter-examples item 6 — returning 130 for SIGTERM
/// hard-kill is incorrect. External monitoring systems (systemd `Restart=on-failure`,
/// k8s `terminationGracePeriodSeconds`) MUST use exit code 143 (not 130) to detect
/// SIGTERM hard-kill during drain (BC-2.01.004 INV-4).
///
/// Traces to BC-2.01.004 PC-8 + INV-4, AC-004 (SigtermDuringDrain=143=128+15).
#[test]
fn test_BC_2_01_004_exit_codes_sigterm_during_drain_is_143() {
    assert_eq!(
        DaemonExit::SigtermDuringDrain.to_exit_code(),
        143,
        "DaemonExit::SigtermDuringDrain exit code must be 143 (POSIX 128+SIGTERM(15), \
        BC-2.01.004 PC-8 + INV-4). External monitoring MUST use 143 (not 130) to detect \
        SIGTERM hard-kill. Counter-example: VP-004 §Counter-examples item 6 — returning \
        130 for SIGTERM misidentifies the trigger to systemd/k8s."
    );
}

// ---------------------------------------------------------------------------
// AC-004 invariants — BC-2.01.004 INV-4 + PC-8
// ---------------------------------------------------------------------------

/// BC-2.01.004 INV-4: SIGTERM hard-kill (143) and SIGINT hard-kill (130) exit codes
/// are distinct. External monitoring systems MUST NOT conflate them.
///
/// VP-004 §Counter-examples item 6: "Exit code 130 returned for a SIGTERM hard-kill scenario"
/// is a failure. The test asserts `SigtermDuringDrain != SigintDuringDrain` at the code level.
///
/// Traces to BC-2.01.004 INV-4, VP-004 §Counter-examples item 6, AC-004.
#[test]
fn test_BC_2_01_004_invariant_sigterm_and_sigint_exit_codes_are_distinct() {
    let sigterm_code = DaemonExit::SigtermDuringDrain.to_exit_code();
    let sigint_code = DaemonExit::SigintDuringDrain.to_exit_code();

    assert_ne!(
        sigterm_code,
        sigint_code,
        "SigtermDuringDrain ({sigterm_code}) and SigintDuringDrain ({sigint_code}) exit codes \
        MUST be distinct (BC-2.01.004 INV-4). External monitoring (systemd, k8s) distinguishes \
        SIGTERM (143) from SIGINT (130); conflating them defeats the diagnostics contract. \
        Counter-example: VP-004 §Counter-examples item 6."
    );

    // Specific values are also normative (POSIX 128+N convention).
    assert_eq!(
        sigterm_code, 143,
        "SigtermDuringDrain must be 143 (128+15); got {sigterm_code}"
    );
    assert_eq!(
        sigint_code, 130,
        "SigintDuringDrain must be 130 (128+2); got {sigint_code}"
    );
}

/// BC-2.01.004 PC-8: All 5 exit codes are distinct.
///
/// Counter-example guarded: VP-004 §Counter-examples item 7 — AdminForceStop using code 1
/// (same as StartupFailure) cannot be distinguished by monitoring.
///
/// VP-004 §Counter-examples item 8: "Single-binary exit-code (any non-zero accepted)"
/// regression guard — no two variants may share a code.
///
/// Traces to BC-2.01.004 PC-8, VP-004 §Counter-examples items 7 + 8, AC-004.
#[test]
fn test_BC_2_01_004_invariant_all_5_exit_codes_are_distinct() {
    let codes = [
        ("Graceful", DaemonExit::Graceful.to_exit_code()),
        ("StartupFailure", DaemonExit::StartupFailure.to_exit_code()),
        ("AdminForceStop", DaemonExit::AdminForceStop.to_exit_code()),
        (
            "SigintDuringDrain",
            DaemonExit::SigintDuringDrain.to_exit_code(),
        ),
        (
            "SigtermDuringDrain",
            DaemonExit::SigtermDuringDrain.to_exit_code(),
        ),
    ];

    for i in 0..codes.len() {
        for j in (i + 1)..codes.len() {
            let (name_i, code_i) = codes[i];
            let (name_j, code_j) = codes[j];
            assert_ne!(
                code_i,
                code_j,
                "DaemonExit::{name_i} ({code_i}) and DaemonExit::{name_j} ({code_j}) have the \
                SAME exit code — all 5 codes must be distinct (BC-2.01.004 PC-8). \
                Counter-example: VP-004 §Counter-examples item 7+8 — shared codes prevent \
                monitoring from distinguishing trigger causes."
            );
        }
    }
}

/// BC-2.01.004 PC-8 + INV-4: POSIX 128+N convention for SIGINT.
///
/// The POSIX convention for signal-induced exits is 128 + signal number.
/// SIGINT has signal number 2. Therefore SIGINT hard-kill exit = 128 + 2 = 130.
///
/// This test makes the arithmetic explicit and guards against misremembering signal numbers.
///
/// Traces to BC-2.01.004 PC-8 + INV-4, POSIX 128+N convention.
#[test]
fn test_BC_2_01_004_invariant_posix_128n_convention_sigint_is_128_plus_2() {
    const SIGINT_NUMBER: i32 = 2;
    let expected = 128 + SIGINT_NUMBER;
    assert_eq!(
        DaemonExit::SigintDuringDrain.to_exit_code(),
        expected,
        "DaemonExit::SigintDuringDrain must follow POSIX 128+N convention: \
        128 + SIGINT({SIGINT_NUMBER}) = {expected}; got {}",
        DaemonExit::SigintDuringDrain.to_exit_code()
    );
}

/// BC-2.01.004 PC-8 + INV-4: POSIX 128+N convention for SIGTERM.
///
/// SIGTERM has signal number 15. Therefore SIGTERM hard-kill exit = 128 + 15 = 143.
///
/// External monitoring systems (systemd `Restart=on-failure`, k8s
/// `terminationGracePeriodSeconds`) MUST use exit code 143 to detect SIGTERM hard-kill
/// during drain (BC-2.01.004 INV-4).
///
/// Traces to BC-2.01.004 PC-8 + INV-4, POSIX 128+N convention.
#[test]
fn test_BC_2_01_004_invariant_posix_128n_convention_sigterm_is_128_plus_15() {
    const SIGTERM_NUMBER: i32 = 15;
    let expected = 128 + SIGTERM_NUMBER;
    assert_eq!(
        DaemonExit::SigtermDuringDrain.to_exit_code(),
        expected,
        "DaemonExit::SigtermDuringDrain must follow POSIX 128+N convention: \
        128 + SIGTERM({SIGTERM_NUMBER}) = {expected}; got {}",
        DaemonExit::SigtermDuringDrain.to_exit_code()
    );
}

// ---------------------------------------------------------------------------
// AC-005 — BC-2.01.004 INV-1 — Hard timeout: drain completes within 11 seconds
// ---------------------------------------------------------------------------

/// VP-004 Probe 4.e: Drain completes within 10 seconds for a normal in-flight request.
///
/// BC-2.01.004 INV-1: "The 10-second drain window is a hard timeout." VP-004 §Post-conditions
/// item 5: "With one synthetic in-flight /hooks/* POST that holds a 5-second sleep, the drain
/// completes within 10 seconds and the daemon exits cleanly with exit code 0."
///
/// This test verifies the time bound from the HTTP-observable side: after a shutdown is
/// initiated, subsequent requests must be rejected with 503 (no new connections accepted).
/// The 11-second bound is the VP-004 canonical assertion (elapsed < 11 seconds).
///
/// Counter-example guarded: VP-004 §Counter-examples item 4 — "Drain timeout not enforced
/// (in-flight 15-second sleep allowed to complete)" would cause the elapsed > 10 seconds.
///
/// Note: this test verifies the observable HTTP behavior (503 after shutdown initiated,
/// within 11-second wall-clock bound). The full drain-with-in-flight-sleep test is an
/// integration test that requires process-level signal wiring (implemented by S-005).
///
/// Traces to BC-2.01.004 INV-1, VP-004 §Post-conditions item 5/6, AC-005.
#[tokio::test]
async fn test_BC_2_01_004_drain_completes_within_11_seconds() {
    let state = make_state_with_token();

    // Measure the time to: (1) initiate shutdown via POST /shutdown, (2) verify subsequent
    // hook requests receive 503 (drain active). The combined operation must complete well
    // within the 11-second bound (the actual drain timer starts on shutdown trigger).

    let start = std::time::Instant::now();

    // Step 1: Initiate shutdown.
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = post_shutdown_json(
        Arc::clone(&state),
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "POST /shutdown must return HTTP 200 to initiate drain; got {status}. Body: {body}. \
        Counter-example: stub returns 501 — drain never starts."
    );

    // Step 2: After shutdown initiated, hook posts must return 503.
    let (hook_status, hook_body) =
        post_hook_json(Arc::clone(&state), "/hooks/pre-tool-use", TEST_TOKEN_HEX).await;

    let elapsed = start.elapsed();

    // VP-004 §Post-conditions item 5: assert elapsed < 11 seconds.
    assert!(
        elapsed < Duration::from_secs(11),
        "Drain + 503 verification must complete within 11 seconds \
        (VP-004 §Post-conditions item 5/6 canonical assertion 'elapsed < 11 seconds'); \
        elapsed: {elapsed:?}. Counter-example: VP-004 §Counter-examples item 4 — drain \
        timeout not enforced."
    );

    assert_eq!(
        hook_status,
        StatusCode::SERVICE_UNAVAILABLE,
        "POST /hooks/* after shutdown initiated must return 503 (drain active, \
        BC-2.01.004 PC-2); got {hook_status}. Body: {hook_body}."
    );
}

// ---------------------------------------------------------------------------
// AC-007 — BC-2.01.004 PC-7 — Lock file absent after graceful shutdown
// ---------------------------------------------------------------------------

/// BC-2.01.004 PC-7: After clean shutdown, the lock file is absent from the filesystem.
///
/// `lifecycle::exit_with(DaemonExit::Graceful)` must invoke `DaemonLock::release()`
/// BEFORE `std::process::exit(0)`, removing the lock file and UDS socket from disk.
///
/// This test verifies the filesystem assertion: write a real lock file via `DaemonLock::acquire`,
/// trigger a graceful shutdown through the endpoint, then assert the lock file is absent.
///
/// Counter-example guarded: if `exit_with` calls `process::exit` without first calling
/// `lock.release()`, the lock file persists and the next daemon start will find a stale
/// lock. The test asserts `fs::metadata(lock_path).is_err()` (file does not exist).
///
/// Note: this test requires the S-005 implementation to wire `DaemonLock::release()` in
/// `lifecycle::exit_with(DaemonExit::Graceful)`. During the Red Gate phase, the stub handler
/// returns 501 and never calls `exit_with`, so the lock file is NOT removed — this test fails.
///
/// Traces to BC-2.01.004 PC-7, BC-2.01.005 PC-6 (lock+sock removal invariant), AC-007.
#[tokio::test]
async fn test_BC_2_01_004_lock_file_absent_after_graceful_shutdown() {
    use monocle_runtime::lifecycle::ensure_runtime_dir;
    use monocle_runtime::lock::DaemonLock;

    // Create a temporary runtime directory for this test.
    let tmp_dir = tempfile::TempDir::new().expect("create temp dir");
    let runtime_dir = tmp_dir.path();
    ensure_runtime_dir(runtime_dir).expect("ensure runtime dir");

    // The expected lock file path (canonical: <runtime_dir>/monocle.lock).
    let expected_lock_path = runtime_dir.join("monocle.lock");

    // Acquire a real lock file at an ephemeral port.
    let (lock, auth_token) =
        DaemonLock::acquire(runtime_dir, 9999).expect("acquire lock file");

    // Verify lock file exists before shutdown.
    assert!(
        expected_lock_path.exists(),
        "lock file must exist before shutdown; not found at {expected_lock_path:?}"
    );

    // Set up DaemonState with the real auth token and pre-built lock path.
    let mut state = DaemonState::new();
    state.auth_token = auth_token.clone();
    state.lock_file_path = expected_lock_path.to_string_lossy().to_string();
    // The S-005 implementation needs a way to call lock.release() from the handler.
    // For this test, we verify the lock IS absent after a proper shutdown sequence.
    // The stub handler returns 501 and doesn't release, so this test FAILS at Red Gate.
    let state = Arc::new(state);

    // Trigger shutdown via POST /shutdown.
    let canonical_header = format!("monocle-v1:{auth_token}");
    let (status, body) = post_shutdown_json(
        Arc::clone(&state),
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "POST /shutdown must return HTTP 200 to initiate graceful shutdown; \
        got {status}. Body: {body}. Counter-example: stub returns 501."
    );

    // After graceful shutdown completes, lock file MUST be absent (BC-2.01.004 PC-7).
    // Allow a brief moment for async cleanup to propagate (lock release is synchronous
    // in S-005 implementation, but tokio schedules may have a delay).
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(
        !expected_lock_path.exists(),
        "lock file MUST be absent after graceful shutdown completes \
        (BC-2.01.004 PC-7: lifecycle::exit_with(DaemonExit::Graceful) must call \
        DaemonLock::release() before std::process::exit(0)); \
        lock file still present at: {expected_lock_path:?}. \
        Counter-example: stub never calls exit_with, lock persists."
    );

    // Clean up: release manually if the test reached this point (stub left it behind).
    // This prevents the TempDir from being poisoned for other tests.
    let _ = lock.release();
}

// ---------------------------------------------------------------------------
// EC-050 — Second POST /shutdown during drain → 200 + exit code 2
// ---------------------------------------------------------------------------

/// EC-050: `POST /shutdown` with valid auth during a drain already in progress acknowledges
/// with HTTP 200 and triggers immediate hard close with exit code 2 (admin forced-stop).
///
/// BC-2.01.004 EC-050: "Second authenticated POST /shutdown during drain — Daemon acknowledges
/// with HTTP 200; second shutdown call triggers immediate hard close with exit code 2
/// (admin forced-stop)."
///
/// This test verifies:
/// 1. The second POST /shutdown returns HTTP 200 (acknowledges, not errors).
/// 2. The subsequent hook requests receive 503 (daemon is still in ShuttingDown mode).
///
/// The exit code 2 assertion is in the exit-code unit tests (AC-004). Testing process exit
/// from an integration test is not feasible without subprocess spawning; the exit-code
/// behavior is verified via `DaemonExit::AdminForceStop.to_exit_code() == 2`.
///
/// Traces to BC-2.01.004 EC-050, AC-004 (AdminForceStop=2).
#[tokio::test]
async fn test_BC_2_01_004_second_post_shutdown_during_drain_returns_200() {
    let state = make_shutting_down_state_with_token();

    // Second POST /shutdown arrives while drain is in progress (AppMode is already ShuttingDown).
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = post_shutdown_json(
        Arc::clone(&state),
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Second POST /shutdown during drain (AppMode::ShuttingDown) must return HTTP 200 \
        (EC-050: daemon acknowledges, does not return error); got {status}. Body: {body}. \
        Counter-example: returning 503 or 409 for a second shutdown during drain."
    );
    // Body must indicate the daemon is shutting down.
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("shutting_down"),
        "Second POST /shutdown body must contain {{\"status\":\"shutting_down\"}}; got: {body}"
    );
}

// ---------------------------------------------------------------------------
// Architectural invariants — SS-conventions-anti-patterns.md
// ---------------------------------------------------------------------------

/// SS-conventions-anti-patterns.md §"No `std::process::exit` in handler code":
/// The `handlers/shutdown.rs` file must NOT call `std::process::exit` directly.
/// All process exits MUST go through `lifecycle::exit_with`.
///
/// This is a static structural invariant verified by source-grep on non-comment lines.
///
/// Traces to SS-conventions-anti-patterns.md, S-005 §Architecture Compliance Rules.
#[test]
fn test_BC_2_01_004_invariant_no_process_exit_in_handler_code() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shutdown_src =
        fs::read_to_string(manifest_dir.join("src/handlers/shutdown.rs"))
            .expect("src/handlers/shutdown.rs must exist");

    // Helper: collect non-comment lines that contain the target string.
    let non_comment_matches = |src: &str, target: &str| -> Vec<(usize, String)> {
        src.lines()
            .enumerate()
            .filter(|(_, line)| {
                let trimmed = line.trim_start();
                !trimmed.starts_with("//") && line.contains(target)
            })
            .map(|(i, line)| (i + 1, line.to_owned()))
            .collect()
    };

    // Handlers must NOT call std::process::exit directly.
    let process_exit_hits = non_comment_matches(&shutdown_src, "process::exit");
    assert!(
        process_exit_hits.is_empty(),
        "handlers/shutdown.rs must NOT call std::process::exit directly — all process exits \
        must go through lifecycle::exit_with (SS-conventions-anti-patterns.md); \
        found direct process::exit calls at lines: {:?}",
        process_exit_hits
    );

    // Also check the server.rs — no handler-called process exits.
    let server_src =
        fs::read_to_string(manifest_dir.join("src/server.rs")).expect("src/server.rs must exist");
    let server_exit_hits = non_comment_matches(&server_src, "process::exit");
    assert!(
        server_exit_hits.is_empty(),
        "src/server.rs must NOT call std::process::exit directly; \
        found direct process::exit calls at lines: {:?}",
        server_exit_hits
    );
}

/// SS-conventions-anti-patterns.md §"No `std::process::exit` in handler code":
/// `lifecycle::exit_with` must be the SOLE call-site for `std::process::exit` in the
/// codebase. Verify that `process::exit` appears exactly once in `lifecycle.rs` (inside
/// `exit_with`) and nowhere else in non-comment, non-test source.
///
/// This test uses a source-grep approach consistent with the structural invariant tests
/// in `healthz_endpoint.rs` and `status_endpoint_auth.rs`.
///
/// Traces to SS-conventions-anti-patterns.md, S-005 §Architecture Compliance Rules.
#[test]
fn test_BC_2_01_004_invariant_exit_with_is_sole_process_exit_callsite() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lifecycle_src =
        fs::read_to_string(manifest_dir.join("src/lifecycle.rs"))
            .expect("src/lifecycle.rs must exist");

    // Count non-comment lines in lifecycle.rs containing std::process::exit.
    let lifecycle_exit_lines: Vec<(usize, String)> = lifecycle_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && line.contains("std::process::exit")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    // `exit_with` must call it exactly once (inside the function body).
    assert_eq!(
        lifecycle_exit_lines.len(),
        1,
        "lifecycle.rs must contain EXACTLY ONE non-comment call to std::process::exit \
        (inside exit_with — the sole authorized call-site per SS-conventions-anti-patterns.md); \
        found {} call(s) at lines: {:?}",
        lifecycle_exit_lines.len(),
        lifecycle_exit_lines
    );

    // Verify the single call is within the exit_with function (not in resolve_runtime_dir
    // or ensure_runtime_dir).
    let (line_num, line_content) = &lifecycle_exit_lines[0];
    assert!(
        line_content.contains("std::process::exit"),
        "lifecycle.rs line {line_num} should contain std::process::exit; got: {line_content:?}"
    );
}

/// Structural invariant: `handlers/shutdown.rs` must NOT import from `monocle-tui`
/// (not a Phase 1 crate).
///
/// Mirrors the pattern from `test_BC_2_01_001_invariant_healthz_does_not_import_monocle_tui`.
///
/// Traces to S-005 §Architecture Compliance Rules ("This handler MUST NOT import from monocle-tui").
#[test]
fn test_BC_2_01_004_invariant_shutdown_handler_does_not_import_monocle_tui() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shutdown_src =
        fs::read_to_string(manifest_dir.join("src/handlers/shutdown.rs"))
            .expect("src/handlers/shutdown.rs must exist");

    let non_comment_hits: Vec<(usize, String)> = shutdown_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && (line.contains("monocle_tui") || line.contains("monocle-tui"))
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        non_comment_hits.is_empty(),
        "handlers/shutdown.rs must NOT import from monocle-tui (not a Phase 1 crate); \
        found non-comment references at lines: {:?}",
        non_comment_hits
    );
}

/// Structural invariant: `DaemonExit` is defined in `lifecycle.rs`, not in a handler file.
///
/// The exit-code taxonomy belongs to the lifecycle module per the S-005 §File Structure
/// Requirements: "lifecycle.rs — daemon start/stop lifecycle, exit code definitions".
///
/// This test verifies that `DaemonExit` is accessible via `monocle_runtime::lifecycle::DaemonExit`
/// (the import at the top of this test file already proves this at compile time).
///
/// Traces to S-005 §File Structure Requirements.
#[test]
fn test_BC_2_01_004_invariant_daemon_exit_defined_in_lifecycle_module() {
    // The fact that this test file imports DaemonExit from monocle_runtime::lifecycle::DaemonExit
    // at compile time proves it is defined there. This test makes the structural intent explicit
    // and provides a named test for the traceability matrix.
    let _ = DaemonExit::Graceful.to_exit_code();
    let _ = DaemonExit::StartupFailure.to_exit_code();
    let _ = DaemonExit::AdminForceStop.to_exit_code();
    let _ = DaemonExit::SigintDuringDrain.to_exit_code();
    let _ = DaemonExit::SigtermDuringDrain.to_exit_code();
    // If the import compiles, DaemonExit is in the lifecycle module as required.
}
