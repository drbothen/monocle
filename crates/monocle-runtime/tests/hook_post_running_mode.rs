//! Integration tests for `POST /hooks/pre-tool-use` in Running mode (S-009 adversary R2).
//!
//! Tests the hook POST endpoints through the full server stack: auth middleware, body-limit
//! middleware, and handler — using in-process `tower::ServiceExt::oneshot` (no TCP).
//!
//! # Contract
//!
//! Exercises BC-2.01.002 PC-1 / AC-010b (hook POST → 200 `{"status":"ok"}` when Running)
//! and BC-2.01.009 PC-1 (missing auth → 401 E-AUTH-001).
//!
//! # Coverage
//!
//! | Test | BC / AC | Assertion |
//! |------|---------|-----------|
//! | test_hook_pre_tool_use_running_canonical_auth_returns_200 | BC-2.01.002 AC-010b | POST /hooks/pre-tool-use with valid canonical auth + JSON body → HTTP 200 `{"status":"ok"}` |
//! | test_hook_pre_tool_use_unauthenticated_returns_401 | BC-2.01.009 PC-1 | POST /hooks/pre-tool-use with no auth header → HTTP 401 E-AUTH-001 |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per the naming convention.
#![allow(non_snake_case)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use monocle_runtime::server::build_server;
use monocle_runtime::state::DaemonState;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Test constants and helpers
// ---------------------------------------------------------------------------

/// Raw 64-hex-char auth token used across all hook POST tests.
const HOOK_TEST_TOKEN: &str = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

/// Build a `DaemonState` in `AppMode::Running` with the test token pre-loaded.
fn make_running_state() -> Arc<DaemonState> {
    let mut state = DaemonState::new();
    state.auth_token = HOOK_TEST_TOKEN.to_string();
    // Default mode is Running — no mode override needed.
    Arc::new(state)
}

/// Issue `POST /<uri>` through the full server router with an optional auth header and JSON body.
///
/// Returns `(StatusCode, serde_json::Value)`.
async fn post_hook_json(
    state: Arc<DaemonState>,
    uri: &str,
    auth_headers: &[(&str, &str)],
    body_json: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = build_server(state);
    let body_bytes = serde_json::to_vec(&body_json).expect("serialize JSON body");

    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json");
    for (k, v) in auth_headers {
        builder = builder.header(*k, *v);
    }
    let req = builder
        .body(Body::from(body_bytes))
        .expect("build POST request");

    let response = app.oneshot(req).await.expect("oneshot POST");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body bytes")
        .to_bytes();
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response body must be valid JSON");
    (status, value)
}

// ---------------------------------------------------------------------------
// AC-010b — BC-2.01.002 PC-1 — POST /hooks/pre-tool-use Running mode → 200 {"status":"ok"}
// ---------------------------------------------------------------------------

/// BC-2.01.002 AC-010b: `POST /hooks/pre-tool-use` with valid canonical auth and a JSON body
/// in `AppMode::Running` returns HTTP 200 with body `{"status":"ok"}`.
///
/// This verifies the full stack in Running mode:
/// 1. Auth middleware passes (canonical `X-Monocle-Authorization: monocle-v1:<64-hex>`).
/// 2. Body-limit middleware passes (small JSON body, well within 256 KiB).
/// 3. `post_hook_pre_tool_use` handler is reached and returns the hook-ok response.
///
/// Traces to BC-2.01.002 PC-1 / AC-010b, BC-2.01.009 PC-2.
#[tokio::test]
async fn test_hook_pre_tool_use_running_canonical_auth_returns_200() {
    let state = make_running_state();
    let canonical_header = format!("monocle-v1:{HOOK_TEST_TOKEN}");
    let body = serde_json::json!({
        "session_id": "test-session-abc",
        "tool_name": "Write",
        "tool_input": {"path": "/tmp/test.txt", "content": "hello"}
    });

    let (status, resp_body) = post_hook_json(
        state,
        "/hooks/pre-tool-use",
        &[("X-Monocle-Authorization", canonical_header.as_str())],
        body,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "POST /hooks/pre-tool-use with valid canonical auth in Running mode must return HTTP 200; \
        got {status}. Body: {resp_body}. \
        Traces to BC-2.01.002 AC-010b."
    );
    assert_eq!(
        resp_body.get("status").and_then(|v| v.as_str()),
        Some("ok"),
        "POST /hooks/pre-tool-use 200 body must be {{\"status\":\"ok\"}}; got: {resp_body}. \
        Traces to BC-2.01.002 AC-010b."
    );
    let obj = resp_body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "POST /hooks/pre-tool-use 200 body must have exactly 1 key; \
        got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.009 PC-1 — POST /hooks/pre-tool-use unauthenticated → 401 E-AUTH-001
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-1: `POST /hooks/pre-tool-use` with NO auth header returns HTTP 401
/// with body `{"error":"missing_auth_token"}` (E-AUTH-001).
///
/// The auth middleware runs first and rejects the request before the hook handler is reached.
/// Neither `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` is present.
///
/// Traces to BC-2.01.009 PC-1 / E-AUTH-001.
#[tokio::test]
async fn test_hook_pre_tool_use_unauthenticated_returns_401() {
    let state = make_running_state();
    let body = serde_json::json!({
        "session_id": "test-session-xyz",
        "tool_name": "Read"
    });

    let (status, resp_body) =
        post_hook_json(state, "/hooks/pre-tool-use", &[], body).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "POST /hooks/pre-tool-use with no auth header must return HTTP 401 E-AUTH-001; \
        got {status}. Body: {resp_body}. \
        Traces to BC-2.01.009 PC-1."
    );
    assert_eq!(
        resp_body.get("error").and_then(|v| v.as_str()),
        Some("missing_auth_token"),
        "Unauthenticated POST /hooks/pre-tool-use must return E-AUTH-001 body \
        {{\"error\":\"missing_auth_token\"}}; got: {resp_body}. \
        Traces to BC-2.01.009 PC-1."
    );
    let obj = resp_body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "E-AUTH-001 body must have exactly 1 key; got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}
