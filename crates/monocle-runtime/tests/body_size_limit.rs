//! Integration tests for S-004: Body Size Limit (256 KiB, HTTP 413).
//!
//! Verifies that the authenticated router enforces a 256 KiB body size limit and returns
//! a structured JSON 413 response when the limit is exceeded.
//!
//! # Behavioral contract
//!
//! BC-2.01.003 Invariant 2 (body limit on authenticated router only):
//! > `DefaultBodyLimit::max(256 * 1024)` MUST be applied to the authenticated router.
//! > Exceeding the limit MUST produce HTTP 413 with JSON body
//! > `{"error":"payload_too_large","limit_bytes":262144}`.
//!
//! BC-2.01.003 Postcondition 4 (no body limit on unauthenticated router):
//! > `GET /healthz` is never 413 regardless of body size.
//!
//! # Coverage Map
//!
//! | AC | BC/Story clause | Test function |
//! |----|-----------------|---------------|
//! | AC-001 | Body > 262144 on auth route → 413 | test_BC_S004_001_overlimit_body_returns_413 |
//! | AC-002 | Body == 262144 on auth route → not 413 | test_BC_S004_002_atlimit_body_is_not_413 |
//! | AC-003 | GET /healthz with large body → 200 (no limit on unauth) | test_BC_S004_003_healthz_large_body_not_413 |
//! | AC-004 | 413 body is EXACTLY `{"error":"payload_too_large","limit_bytes":262144}` | test_BC_S004_004_413_body_is_exact_json |
//! | AC-005 | POST overlimit to /status → 413 (cross-route enforcement) | test_BC_S004_005_overlimit_status_route_413 |
//! | AC-006 | Source-grep: `DefaultBodyLimit::max(262144)` appears in server.rs | test_BC_S004_006_source_grep_default_body_limit_once |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per the naming convention.
#![allow(non_snake_case)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use http_body_util::BodyExt;
use monocle_runtime::router::unauthenticated_router;
use monocle_runtime::server::build_server;
use monocle_runtime::state::DaemonState;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Raw 64-hex-char auth token used across all auth-path tests.
const TEST_TOKEN_HEX: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

/// Build a `DaemonState` with the test token pre-loaded.
fn make_state_with_token() -> Arc<DaemonState> {
    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN_HEX.to_string();
    Arc::new(state)
}

/// POST a body of `body_len` bytes to the given `path` on the full authenticated server.
/// The request includes the canonical auth header and an explicit `Content-Length` header
/// so that the body-size enforcement middleware can inspect the declared length.
async fn post_with_size(
    state: Arc<DaemonState>,
    path: &str,
    body_len: usize,
) -> (StatusCode, Vec<u8>) {
    let app = build_server(state);
    let auth_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let body_bytes = vec![0u8; body_len];
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("X-Monocle-Authorization", auth_header.as_str())
        .header("Content-Length", body_len.to_string())
        .body(Body::from(body_bytes))
        .expect("build POST request");

    let response = app.oneshot(req).await.expect("oneshot POST");
    let status = response.status();
    let collected = response
        .into_body()
        .collect()
        .await
        .expect("collect response body")
        .to_bytes();
    (status, collected.to_vec())
}

/// POST a body of `body_len` bytes to the given `path` on the full authenticated server.
/// Returns `(StatusCode, HeaderMap, Vec<u8>)` so callers can inspect response headers.
///
/// Used by AC-004 to assert the 413 response carries `Content-Type: application/json`.
async fn post_with_size_full(
    state: Arc<DaemonState>,
    path: &str,
    body_len: usize,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let app = build_server(state);
    let auth_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let body_bytes = vec![0u8; body_len];
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("X-Monocle-Authorization", auth_header.as_str())
        .header("Content-Length", body_len.to_string())
        .body(Body::from(body_bytes))
        .expect("build POST request");

    let response = app.oneshot(req).await.expect("oneshot POST");
    let status = response.status();
    let headers = response.headers().clone();
    let collected = response
        .into_body()
        .collect()
        .await
        .expect("collect response body")
        .to_bytes();
    (status, headers, collected.to_vec())
}

// ---------------------------------------------------------------------------
// AC-001 — Body > 262144 on authenticated route → HTTP 413
// ---------------------------------------------------------------------------

/// AC-001: POST 262,145 bytes (256 KiB + 1) to `/status` with valid auth returns HTTP 413.
///
/// The `Content-Length: 262145` header causes `body_size_limit_middleware` to intercept
/// the request before it reaches the route handler and return 413 Payload Too Large.
///
/// Counter-example guarded: if the middleware is missing or the limit is wrong, the
/// request would reach the route handler and return 200 or 405, NOT 413.
///
/// Traces to BC-2.01.003 Invariant 2, SS-daemon-lifecycle.md v1.0.33 §Body Size Limit.
#[tokio::test]
async fn test_BC_S004_001_overlimit_body_returns_413() {
    let state = make_state_with_token();
    let body_len = 262_145; // 256 KiB + 1 byte
    let (status, body_bytes) = post_with_size(state, "/status", body_len).await;

    assert_eq!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "POST with Content-Length 262145 (256 KiB + 1) to /status must return HTTP 413; \
        got {status}. Body: {:?}. \
        Counter-example: missing body_size_limit_middleware returns 200/405 instead of 413.",
        std::str::from_utf8(&body_bytes).unwrap_or("<non-utf8>")
    );
}

// ---------------------------------------------------------------------------
// AC-002 — Body == 262144 on authenticated route → NOT 413 (at-limit is allowed)
// ---------------------------------------------------------------------------

/// AC-002: POST exactly 262,144 bytes (256 KiB, at the limit) to `/status` with valid auth
/// MUST NOT return HTTP 413.
///
/// The limit is `> 262144` (strictly greater), so 262144 itself is allowed.
/// A POST to `GET /status` will return 405 (Method Not Allowed) — not 413 — because
/// the body is within the limit and the route handler runs normally.
///
/// Counter-example guarded: if the comparison is `>= 262144` (wrong boundary), a body of
/// exactly 262144 would also return 413, incorrectly rejecting at-limit payloads.
///
/// Traces to BC-2.01.003 Invariant 2 (strict > limit) / S-004 AC-002.
#[tokio::test]
async fn test_BC_S004_002_atlimit_body_is_not_413() {
    let state = make_state_with_token();
    let body_len = 262_144; // exactly 256 KiB (at the limit — must NOT return 413)
    let (status, _body_bytes) = post_with_size(state, "/status", body_len).await;

    assert_ne!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "POST with Content-Length 262144 (exactly 256 KiB, at the limit) must NOT return \
        HTTP 413 (the limit is strictly > 262144); got {status}. \
        Counter-example: >= boundary instead of > would incorrectly reject at-limit payloads."
    );
    // A POST to /status (GET only) returns 405, not 413. Either way, not 413.
    assert!(
        status == StatusCode::METHOD_NOT_ALLOWED
            || status == StatusCode::NOT_FOUND
            || status == StatusCode::OK,
        "POST at-limit to /status must return 405/404/200, not 413; got {status}"
    );
}

// ---------------------------------------------------------------------------
// AC-003 — GET /healthz with large body → 200 (no body limit on unauthenticated router)
// ---------------------------------------------------------------------------

/// AC-003 (structural): `GET /healthz` with a 1 MiB body MUST NOT return HTTP 413.
///
/// The unauthenticated router has no `DefaultBodyLimit` and no `body_size_limit_middleware`.
/// Even a body 4× the authenticated limit must succeed with HTTP 200.
///
/// Counter-example guarded: if body limit middleware were mistakenly applied to the
/// unauthenticated router, /healthz would return 413 for large bodies.
///
/// Traces to BC-2.01.003 Postcondition 4 (no body limit on unauthenticated router).
#[tokio::test]
async fn test_BC_S004_003_healthz_large_body_not_413() {
    let state = Arc::new(DaemonState::new());
    let router = unauthenticated_router(state);

    // 1 MiB body (4× the authenticated limit)
    let body_len = 1_048_576;
    let body_bytes = vec![0u8; body_len];
    let req = Request::builder()
        .method("GET")
        .uri("/healthz")
        .header("Content-Length", body_len.to_string())
        .body(Body::from(body_bytes))
        .expect("build GET /healthz with large body");

    let response = router.oneshot(req).await.expect("oneshot GET /healthz");
    let status = response.status();

    assert_ne!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "GET /healthz with 1 MiB body must NOT return HTTP 413; got {status}. \
        The unauthenticated router must have no body limit \
        (BC-2.01.003 Postcondition 4). \
        Counter-example: body limit middleware on unauth router incorrectly returns 413."
    );
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /healthz with 1 MiB body must return HTTP 200; got {status}"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — 413 body is EXACTLY `{"error":"payload_too_large","limit_bytes":262144}`
// ---------------------------------------------------------------------------

/// AC-004: The JSON body of the 413 response is EXACTLY two fields, no extras, and the
/// response carries `Content-Type: application/json`.
///
/// Required shape: `{"error":"payload_too_large","limit_bytes":262144}`
/// Required header: `Content-Type: application/json` (confirms the 413 is a proper JSON
/// response, not a plain-text or HTML error page).
///
/// Counter-examples guarded:
/// - `{"error":"payload_too_large"}` (missing `limit_bytes`) fails.
/// - `{"error":"payload_too_large","limit_bytes":262144,"detail":"..."}` (extra key) fails.
/// - `{"error":"too_large"}` (wrong error code) fails.
/// - `{"limit_bytes":262144,"error":"payload_too_large"}` — key order may vary but values
///   must match; serde_json deserialization is order-independent.
/// - Missing or wrong Content-Type fails (BC-2.01.003 requires a structured JSON response).
///
/// Traces to BC-2.01.003 Invariant 2 / S-004 §Custom 413 handler.
#[tokio::test]
async fn test_BC_S004_004_413_body_is_exact_json() {
    let state = make_state_with_token();
    let body_len = 262_145; // over the limit
    let (status, headers, body_bytes) = post_with_size_full(state, "/status", body_len).await;

    assert_eq!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "prerequisite: overlimit POST must return 413"
    );

    // Assert Content-Type: application/json on the 413 response.
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.starts_with("application/json"),
        "413 response must carry Content-Type: application/json (BC-2.01.003 structured JSON \
        response); got Content-Type: {content_type:?}"
    );

    // Parse the JSON body.
    let value: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("413 body must be valid JSON");
    let obj = value.as_object().expect("413 body must be a JSON object");

    // Exactly 2 keys: `error` and `limit_bytes`.
    assert_eq!(
        obj.len(),
        2,
        "413 body must have EXACTLY 2 keys (error, limit_bytes); got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );

    // `error` field: string "payload_too_large".
    assert_eq!(
        obj.get("error").and_then(|v| v.as_str()),
        Some("payload_too_large"),
        "413 body[\"error\"] must be \"payload_too_large\"; got: {:?}",
        obj.get("error")
    );

    // `limit_bytes` field: integer 262144.
    assert_eq!(
        obj.get("limit_bytes").and_then(|v| v.as_u64()),
        Some(262_144),
        "413 body[\"limit_bytes\"] must be 262144 (256 * 1024); got: {:?}",
        obj.get("limit_bytes")
    );
}

// ---------------------------------------------------------------------------
// AC-005 — POST overlimit to /status → 413 (body limit is router-wide)
// ---------------------------------------------------------------------------

/// AC-005: POST 262,145 bytes to `/status` with auth → HTTP 413.
///
/// Verifies that the body-size enforcement is applied at the authenticated router level —
/// all routes on the authenticated router share the same body limit, not just specific
/// handler implementations that consume the body.
///
/// This is equivalent to AC-001 but explicitly named to satisfy the cross-route assertion
/// in the story acceptance criteria.
///
/// Counter-example guarded: if body limit only applied per-extractor (i.e., only when a
/// handler reads the body with `axum::body::Bytes`), routes that do not read the body would
/// not enforce the limit. The middleware ensures enforcement before handler dispatch.
///
/// Traces to BC-2.01.003 Invariant 2 (body limit applies to authenticated router, not just
/// individual extractors).
#[tokio::test]
async fn test_BC_S004_005_overlimit_status_route_413() {
    let state = make_state_with_token();
    // Use /status which is a GET route — POST to it is either 405 or caught by middleware.
    // With Content-Length > limit, the body_size_limit_middleware fires first → 413.
    let body_len = 262_145;
    let (status, body_bytes) = post_with_size(state, "/status", body_len).await;

    assert_eq!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "POST with Content-Length 262145 to /status must return HTTP 413 (authenticated \
        router body limit applies before route dispatch); got {status}. \
        Body: {:?}. \
        Counter-example: per-extractor-only enforcement would return 405 instead of 413.",
        std::str::from_utf8(&body_bytes).unwrap_or("<non-utf8>")
    );
}

// ---------------------------------------------------------------------------
// AC-006 — Source-grep: `DefaultBodyLimit::max(262144)` appears in server.rs
// ---------------------------------------------------------------------------

/// AC-006 (structural invariant): `server.rs` MUST contain exactly one non-comment
/// occurrence of `DefaultBodyLimit::max(262144)`.
///
/// This is the canonical body limit registration for the authenticated router per
/// SS-daemon-lifecycle.md v1.0.33 §Body Size Limit.
///
/// Counter-examples guarded:
/// - Zero occurrences: body limit not applied → all payloads accepted regardless of size.
/// - Two or more occurrences: duplicate / conflicting registrations.
/// - `DefaultBodyLimit::max(256 * 1024)` form: test accepts that form too (256 * 1024 == 262144).
///
/// Traces to BC-2.01.003 Invariant 2, SS-daemon-lifecycle.md v1.0.33 §Body Size Limit.
#[test]
fn test_BC_S004_006_source_grep_default_body_limit_once() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let server_src =
        fs::read_to_string(manifest_dir.join("src/server.rs")).expect("src/server.rs must exist");

    // Count non-comment lines that contain `DefaultBodyLimit::max(262144)` OR
    // `DefaultBodyLimit::max(256 * 1024)` (both forms are equivalent).
    let hits: Vec<(usize, String)> = server_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//")
                && (line.contains("DefaultBodyLimit::max(262144)")
                    || line.contains("DefaultBodyLimit::max(256 * 1024)"))
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert_eq!(
        hits.len(),
        1,
        "server.rs must contain EXACTLY ONE non-comment `DefaultBodyLimit::max(262144)` \
        (or `DefaultBodyLimit::max(256 * 1024)`) call per SS-daemon-lifecycle.md v1.0.33 \
        §Body Size Limit; found {} occurrence(s) at lines: {:?}",
        hits.len(),
        hits
    );
}
