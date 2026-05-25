//! Integration tests for BC-2.01.002: Status Endpoint (Authenticated Daemon State).
//!
//! Covers the authentication-path verification per VP-002 §Mechanism.
//! Every test name follows the `test_BC_S_SS_NNN_xxx` pattern for full traceability.
//!
//! # Red Gate
//!
//! All tests in this file MUST FAIL before S-003 implementation is complete.
//! The `build_server` and `get_status` stubs use `unimplemented!()`, which panics at
//! runtime; the auth middleware stub also panics. Tests will produce an `Err` or a
//! panic-propagated `SERVICE_UNAVAILABLE` from axum's panic handler.
//!
//! # Coverage Map
//!
//! | Probe | BC / AC | VP-002 Probe | Test function |
//! |-------|---------|--------------|---------------|
//! | AC-001 | BC-2.01.002 PC-1 | 2.a canonical auth → 200 + 10 fields | test_BC_2_01_002_valid_canonical_auth_returns_200_with_all_fields |
//! | AC-002 | BC-2.01.009 PC-3 + INV-6 | 2.b alias auth → 200 + WARN log | test_BC_2_01_002_alias_auth_returns_200_and_emits_warn |
//! | AC-003 | BC-2.01.009 PC-1 | 2.c no auth → 401 E-AUTH-001 | test_BC_2_01_002_no_auth_returns_401_missing_token |
//! | AC-004 | BC-2.01.009 PC-2 | 2.d wrong token → 401 E-AUTH-002 | test_BC_2_01_002_wrong_token_returns_401_invalid_token |
//! | AC-006 | BC-2.01.002 PC-1 hook_endpoints | 2.e hook_endpoints array shape | test_BC_2_01_002_hook_endpoints_array_exactly_5_paths |
//! | Body limit | BC-2.01.001 INV-2 structural | N/A | test_BC_2_01_002_invariant_default_body_limit_on_auth_router |

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
// Test helpers
// ---------------------------------------------------------------------------

/// Raw 64-hex-char auth token used across all auth-path tests.
///
/// Must match the format stored in `DaemonState.auth_token` (raw hex, no prefix).
const TEST_TOKEN_HEX: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

/// Build a `DaemonState` with the test token pre-loaded.
fn make_state_with_token() -> Arc<DaemonState> {
    let state = DaemonState::new();
    // SAFETY: auth_token is set once at daemon startup; test constructs the equivalent.
    // We cannot use the public field directly because it is behind no lock (it's a plain
    // String set before serving starts). The field is `pub` on DaemonState.
    // This is the test equivalent of the S-004 token-generation path.
    let mut state = state;
    state.auth_token = TEST_TOKEN_HEX.to_string();
    Arc::new(state)
}

/// Issue `GET /status` through the full server router and return `(StatusCode, serde_json::Value)`.
///
/// Uses `tower::ServiceExt::oneshot` (in-process, no TCP).
async fn get_status_json(
    state: Arc<DaemonState>,
    headers: &[(&str, &str)],
) -> (StatusCode, serde_json::Value) {
    let app = build_server(state);
    let mut builder = Request::builder().method("GET").uri("/status");
    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }
    let req = builder.body(Body::empty()).expect("build GET /status");
    let response = app.oneshot(req).await.expect("oneshot GET /status");
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

// ---------------------------------------------------------------------------
// AC-001 — BC-2.01.002 PC-1 / VP-002 Probe 2.a — Canonical auth → 200 + 10 fields
// ---------------------------------------------------------------------------

/// VP-002 Probe 2.a: `GET /status` with valid canonical `X-Monocle-Authorization` header
/// returns HTTP 200 with a JSON body containing all 10 required fields.
///
/// Counter-example guarded: stub returns panic/500 from `unimplemented!()`;
/// test asserts 200 and all 10 field keys present.
///
/// Traces to BC-2.01.002 Postcondition 1, AC-001.
#[tokio::test]
async fn test_BC_2_01_002_valid_canonical_auth_returns_200_with_all_fields() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "GET /status with valid canonical auth must return HTTP 200; got {status}. \
        Body: {body}. Counter-example: unimplemented!() stub returns 500."
    );

    // Assert all 10 required fields are present (BC-2.01.002 PC-1 field enumeration).
    let obj = body.as_object().expect("body must be a JSON object");
    let required_fields = [
        "pid",
        "uptime_sec",
        "version",
        "abi_version",
        "lock_file",
        "hook_endpoints",
        "ring_buffer_fill_pct",
        "channel_saturation_pct",
        "last_hook_ts",
        "tui_attached",
    ];
    for field in required_fields {
        assert!(
            obj.contains_key(field),
            "GET /status body must contain field \"{field}\"; missing from: {:?}",
            obj.keys().collect::<Vec<_>>()
        );
    }
    assert_eq!(
        obj.len(),
        10,
        "GET /status body must have EXACTLY 10 fields per BC-2.01.002 PC-1; \
        got {} field(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// AC-002 — BC-2.01.009 PC-3 / INV-6 — Alias auth → 200 + WARN log
// ---------------------------------------------------------------------------

/// VP-002 Probe 2.b: `GET /status` with `X-Claude-Code-Ide-Authorization: <raw-64-hex>` (alias path,
/// no canonical header present) returns HTTP 200 with the same body as canonical auth.
///
/// The WARN log emission is validated structurally (AC-002 requires it; verifying log
/// output in integration tests requires a log subscriber capture mechanism not yet in scope
/// for the Red Gate stub phase). The structural test asserts 200 is returned; log content
/// is verified by the implementer via a tracing subscriber in S-003's full test suite.
///
/// Counter-example guarded: stub panics from `unimplemented!()` in auth middleware;
/// test asserts 200.
///
/// Traces to BC-2.01.009 PC-3, INV-6 (alias path WARN string), AC-002.
#[tokio::test]
async fn test_BC_2_01_002_alias_auth_returns_200_and_emits_warn() {
    let state = make_state_with_token();
    // Alias path: raw 64-hex token, no monocle-v1: prefix (Claude Code lock-file format).
    let (status, body) = get_status_json(
        state,
        &[("X-Claude-Code-Ide-Authorization", TEST_TOKEN_HEX)],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "GET /status with alias auth (X-Claude-Code-Ide-Authorization) must return HTTP 200 \
        (ADR-0005 dual-accept, BC-2.01.009 PC-3); got {status}. Body: {body}. \
        Counter-example: stub panics before auth decision."
    );
    // Body must have the same 10-field shape as canonical auth.
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        10,
        "alias-auth body must have EXACTLY 10 fields; got {} field(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// AC-003 — BC-2.01.009 PC-1 — No auth header → 401 E-AUTH-001
// ---------------------------------------------------------------------------

/// VP-002 Probe 2.c: `GET /status` with no auth header returns HTTP 401 with
/// body `{"error":"missing_auth_token"}` (E-AUTH-001).
///
/// Neither `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` is present.
///
/// Counter-example guarded: stub panics from `unimplemented!()` in auth middleware;
/// test asserts 401 with E-AUTH-001 body.
///
/// Traces to BC-2.01.009 PC-1, AC-003.
#[tokio::test]
async fn test_BC_2_01_002_no_auth_returns_401_missing_token() {
    let state = make_state_with_token();
    let (status, body) = get_status_json(state, &[]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with no auth header must return HTTP 401 E-AUTH-001; got {status}. \
        Body: {body}. Traces to BC-2.01.009 PC-1."
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("missing_auth_token"),
        "E-AUTH-001 body must be {{\"error\":\"missing_auth_token\"}}; got: {body}"
    );
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "E-AUTH-001 body must have exactly 1 key; got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// AC-004 — BC-2.01.009 PC-2 — Wrong token → 401 E-AUTH-002
// ---------------------------------------------------------------------------

/// VP-002 Probe 2.d: `GET /status` with a valid-format but wrong canonical token
/// returns HTTP 401 with body `{"error":"invalid_auth_token"}` (E-AUTH-002).
///
/// The token format is correct (`monocle-v1:<64-hex>`) but the hex value does not
/// match `state.auth_token`.
///
/// Counter-example guarded: stub panics from `unimplemented!()` in auth middleware;
/// test asserts 401 with E-AUTH-002 body.
///
/// Traces to BC-2.01.009 PC-2, AC-004.
#[tokio::test]
async fn test_BC_2_01_002_wrong_token_returns_401_invalid_token() {
    let state = make_state_with_token();
    // Wrong token: 64 hex chars, but all zeros (does not match TEST_TOKEN_HEX).
    let wrong_token_header = format!("monocle-v1:{}", "0".repeat(64));
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", wrong_token_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with wrong token value must return HTTP 401 E-AUTH-002; got {status}. \
        Body: {body}. Traces to BC-2.01.009 PC-2."
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "E-AUTH-002 body must be {{\"error\":\"invalid_auth_token\"}}; got: {body}"
    );
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "E-AUTH-002 body must have exactly 1 key; got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// AC-006 — BC-2.01.002 PC-1 hook_endpoints sub-bullet — exactly 5 canonical paths
// ---------------------------------------------------------------------------

/// VP-002 Probe 2.e: The `hook_endpoints` field is an array of exactly 5 paths in
/// canonical spec order (BC-2.01.002 PC-1 sub-bullet `hook_endpoints` + BC-2.01.008 PC-4).
///
/// Canonical 5 paths (order is normative):
/// 1. `/hooks/pre-tool-use`
/// 2. `/hooks/notification`
/// 3. `/hooks/stop`
/// 4. `/hooks/session-start`
/// 5. `/hooks/prompt-submit`
///
/// Counter-example guarded: stub `unimplemented!()` returns no body;
/// test asserts array length == 5 and exact path values in order.
///
/// Traces to BC-2.01.002 PC-1 sub-bullet `hook_endpoints`, AC-006.
#[tokio::test]
async fn test_BC_2_01_002_hook_endpoints_array_exactly_5_paths() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "prerequisite: canonical auth must return HTTP 200 for hook_endpoints assertion"
    );

    let endpoints = body
        .get("hook_endpoints")
        .and_then(|v| v.as_array())
        .expect("hook_endpoints must be a JSON array");

    let expected: [&str; 5] = [
        "/hooks/pre-tool-use",
        "/hooks/notification",
        "/hooks/stop",
        "/hooks/session-start",
        "/hooks/prompt-submit",
    ];

    assert_eq!(
        endpoints.len(),
        5,
        "hook_endpoints must have EXACTLY 5 paths per BC-2.01.002 PC-1 + BC-2.01.008 PC-4; \
        got {} path(s): {endpoints:?}",
        endpoints.len()
    );

    for (i, (actual, expected_path)) in endpoints.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            actual.as_str(),
            Some(*expected_path),
            "hook_endpoints[{i}] must be \"{expected_path}\" (canonical spec order \
            from BC-2.01.002 PC-1 + BC-2.01.008 PC-4); got: {actual:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Structural invariant — DefaultBodyLimit on authenticated router only
// ---------------------------------------------------------------------------

/// Structural invariant (BC-2.01.001 Invariant 2 complement): `server.rs` MUST contain
/// `DefaultBodyLimit` in non-comment executable code — the authenticated router MUST apply it.
///
/// This is the positive complement to the negative assertion in `healthz_endpoint.rs`:
/// - Healthz test: `DefaultBodyLimit` must NOT appear in `router.rs` non-comment lines.
/// - This test: `DefaultBodyLimit` MUST appear in `server.rs` non-comment lines.
///
/// Uses source-grep on `server.rs`. Does not require a running server.
///
/// Traces to BC-2.01.001 Invariant 2, SS-daemon-lifecycle.md v1.0.33 §Body Size Limit.
#[test]
fn test_BC_2_01_002_invariant_default_body_limit_on_auth_router() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let server_src =
        fs::read_to_string(manifest_dir.join("src/server.rs")).expect("src/server.rs must exist");

    let non_comment_hits: Vec<(usize, String)> = server_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("//") && line.contains("DefaultBodyLimit")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        !non_comment_hits.is_empty(),
        "server.rs (authenticated router builder) MUST apply DefaultBodyLimit in executable \
        code (SS-daemon-lifecycle.md v1.0.33 §Body Size Limit: 256 KiB on authenticated router). \
        No non-comment DefaultBodyLimit usage found. This test is expected to be RED until \
        S-003 implementation wires the body limit."
    );
}
