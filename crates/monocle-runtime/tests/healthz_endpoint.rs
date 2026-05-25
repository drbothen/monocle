//! Integration tests for BC-2.01.001: Healthz Endpoint (Unauthenticated Liveness Probe).
//!
//! Every test name follows the `test_BC_S_SS_NNN_xxx` pattern for full traceability.
//!
//! # Red Gate
//!
//! All tests in this file MUST FAIL before S-002 implementation is complete.
//! The stubs return HTTP 500 via `unimplemented!()` / `StatusCode::INTERNAL_SERVER_ERROR`.
//! The Red Gate is verified by running `cargo test --workspace` after writing these tests.
//!
//! # Coverage Map
//!
//! | Probe | BC-2.01.001 Clause | VP-001 Probe Matrix | Test function |
//! |-------|-------------------|---------------------|---------------|
//! | AC-001 / Probe 1.a | Postcondition 1 | 1.a (normal, no auth) | test_BC_2_01_001_normal_mode_returns_200_alive |
//! | AC-001 body shape | Postcondition 1 | 1.a (3 keys exact) | test_BC_2_01_001_response_body_has_exactly_three_keys |
//! | AC-001 uptime | Postcondition 1 | 1.a (uptime_sec integer ≥ 0) | test_BC_2_01_001_uptime_sec_is_integer_gte_zero |
//! | AC-001 version | Postcondition 1 | 1.f (semver regex) | test_BC_2_01_001_version_matches_semver_regex |
//! | AC-001 version cargo | Postcondition 1 | Post-condition 2 (equals CARGO_PKG_VERSION) | test_BC_2_01_001_version_equals_cargo_pkg_version |
//! | AC-002 / Probe 1.d | Postcondition 2 | 1.d (ShuttingDown → 503) | test_BC_2_01_001_shutting_down_mode_returns_503 |
//! | AC-002 body shape | Postcondition 2 | 1.d (1 key exact (BC literal authoritative over VP wording)) | test_BC_2_01_001_shutting_down_body_has_exactly_one_key |
//! | AC-003 / Probe 1.a | Postcondition 3 | 1.a (no auth → 200 not 401) | test_BC_2_01_001_no_auth_header_returns_200_not_401 |
//! | AC-003 / Probe 1.b | Postcondition 3 | 1.b (valid auth header → 200) | test_BC_2_01_001_valid_auth_header_is_ignored_returns_200 |
//! | AC-003 / Probe 1.c | Postcondition 3 | 1.c (garbage auth header → 200) | test_BC_2_01_001_garbage_auth_header_is_ignored_returns_200 |
//! | AC-004 | Postcondition 4 | 1.e (no DefaultBodyLimit on unauth router) | test_BC_2_01_001_large_body_returns_200_not_413 |
//! | AC-005 / Probe 1.e | Invariant 2 | 1.e (source-grep: DefaultBodyLimit on auth only) | test_BC_2_01_001_invariant_default_body_limit_on_auth_router_only |
//! | AC-006 | EC-040 (100ms timing) | N/A | test_BC_2_01_001_response_within_100ms |
//! | Arch Rule: Forbidden Deps | Story §Architecture Compliance Rules "Forbidden Dependencies" (constant_time_eq auth-only) | N/A | test_BC_2_01_001_invariant_healthz_does_not_import_constant_time_eq |
//! | Arch Rule: Forbidden Deps | Story §Architecture Compliance Rules "Forbidden Dependencies" (monocle-tui forbidden) | N/A | test_BC_2_01_001_invariant_healthz_does_not_import_monocle_tui |
//! | VP-001 proptest auxiliary | VP-001 proptest auxiliary (semver regex) | N/A | test_BC_2_01_001_invariant_semver_regex_shape |
//! | BC-2.01.001 defensive | BC-2.01.001 defensive handling (lock poisoning → graceful degradation to ShuttingDown) | N/A | test_BC_2_01_001_poisoned_lock_returns_503 |
//!
//! # Proptest (VP-001 auxiliary: semver-regex property across random tokens)
//!
//! The semver regex is verified via proptest in `test_BC_2_01_001_invariant_semver_regex_shape`.

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per the naming convention.
#![allow(non_snake_case)]

use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use monocle_runtime::router::unauthenticated_router;
use monocle_runtime::state::{AppMode, DaemonState};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Construct a `DaemonState` in `Running` mode with a fresh start time.
fn make_running_state() -> Arc<DaemonState> {
    Arc::new(DaemonState {
        mode: std::sync::RwLock::new(AppMode::Running),
        start_time: Instant::now(),
        hook_receiver_status: None,
    })
}

/// Construct a `DaemonState` in `ShuttingDown` mode.
fn make_shutting_down_state() -> Arc<DaemonState> {
    Arc::new(DaemonState {
        mode: std::sync::RwLock::new(AppMode::ShuttingDown),
        start_time: Instant::now(),
        hook_receiver_status: None,
    })
}

/// Issue a `GET /healthz` request through the unauthenticated router and return
/// the `(StatusCode, serde_json::Value)` pair.
async fn get_healthz_json(
    state: Arc<DaemonState>,
    extra_headers: &[(&str, &str)],
) -> (StatusCode, serde_json::Value) {
    let router = unauthenticated_router(state);

    let mut builder = Request::builder().method("GET").uri("/healthz");
    for (k, v) in extra_headers {
        builder = builder.header(*k, *v);
    }
    let req = builder.body(Body::empty()).expect("build GET /healthz");

    let response = router.oneshot(req).await.expect("oneshot");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response must be valid JSON");
    (status, value)
}

/// Issue a `GET /healthz` with a body of `body_len` zero bytes.
async fn get_healthz_with_body(state: Arc<DaemonState>, body_len: usize) -> StatusCode {
    let router = unauthenticated_router(state);
    let body_bytes = vec![0u8; body_len];
    let req = Request::builder()
        .method("GET")
        .uri("/healthz")
        .body(Body::from(body_bytes))
        .expect("build GET /healthz with body");
    let response = router.oneshot(req).await.expect("oneshot");
    response.status()
}

// ---------------------------------------------------------------------------
// BC-2.01.001 Postcondition 1 / VP-001 Probe 1.a — Normal mode HTTP 200 alive
// ---------------------------------------------------------------------------

/// VP-001 Probe 1.a: Normal AppMode, no auth header → HTTP 200.
///
/// Exercises BC-2.01.001 Postcondition 1 (first clause: Running mode → HTTP 200).
/// Counter-example guarded: stub returns 500; test asserts 200.
#[tokio::test]
async fn test_BC_2_01_001_normal_mode_returns_200_alive() {
    let state = make_running_state();
    let (status, body) = get_healthz_json(state, &[]).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /healthz in Running mode must return HTTP 200; got {status}. Body: {body}"
    );
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("alive"),
        "body.status must equal \"alive\" in Running mode; got: {body}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.001 Postcondition 1 — 3-key body shape assertion
// ---------------------------------------------------------------------------

/// VP-001 Post-condition 2: response body has EXACTLY three keys in normal mode.
///
/// Counter-examples from VP-001 §Counter-examples:
/// - 1 key only (`{"status":"alive"}`) fails this test.
/// - 4 keys (`{"status":"alive","uptime_sec":N,"version":"v","drain":true}`) fails this test.
#[tokio::test]
async fn test_BC_2_01_001_response_body_has_exactly_three_keys() {
    let state = make_running_state();
    let (status, body) = get_healthz_json(state, &[]).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "prerequisite: HTTP 200 in Running mode"
    );
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        3,
        "body must have EXACTLY 3 keys (status, uptime_sec, version); got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
    assert!(obj.contains_key("status"), "body must contain \"status\"");
    assert!(
        obj.contains_key("uptime_sec"),
        "body must contain \"uptime_sec\""
    );
    assert!(obj.contains_key("version"), "body must contain \"version\"");
}

// ---------------------------------------------------------------------------
// BC-2.01.001 Postcondition 1 — uptime_sec is integer ≥ 0
// ---------------------------------------------------------------------------

/// VP-001 Post-condition 2: `uptime_sec` is a JSON integer (not string) ≥ 0.
///
/// Counter-example: `"uptime_sec": "42"` (string) fails this test.
#[tokio::test]
async fn test_BC_2_01_001_uptime_sec_is_integer_gte_zero() {
    let state = make_running_state();
    let (status, body) = get_healthz_json(state, &[]).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "prerequisite: HTTP 200 in Running mode"
    );
    let uptime = body.get("uptime_sec").expect("uptime_sec must be present");
    assert!(
        uptime.is_i64() || uptime.is_u64(),
        "uptime_sec must be a JSON integer (not string, not float); got: {uptime:?}"
    );
    let n = uptime
        .as_u64()
        .expect("uptime_sec must be a non-negative integer");
    assert!(
        n <= u64::from(u32::MAX),
        "uptime_sec should be a reasonable value; got: {n}"
    );
    // The value may be 0 for a freshly constructed state; ≥ 0 is always true for u64.
    // We assert it can be obtained as u64 (non-negative integer), which is above.
    let _ = n; // suppress unused warning
}

// ---------------------------------------------------------------------------
// BC-2.01.001 Postcondition 1 — version matches semver regex
// ---------------------------------------------------------------------------

/// VP-001 Post-condition 7 + Probe 1.f: `version` field matches SemVer 2.0 regex.
///
/// The regex is: `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$`
/// No leading `v` prefix is permitted.
///
/// Counter-example: `version: "debug"` (build profile string) fails regex match.
/// Counter-example: `version: "v0.1.0"` (with `v` prefix) fails regex match.
#[tokio::test]
async fn test_BC_2_01_001_version_matches_semver_regex() {
    let state = make_running_state();
    let (status, body) = get_healthz_json(state, &[]).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "prerequisite: HTTP 200 in Running mode"
    );
    let version = body
        .get("version")
        .and_then(|v| v.as_str())
        .expect("version field must be a string");
    // Semver 2.0 regex from BC-2.01.001 Postcondition 1 and VP-001 §Post-condition 7.
    let semver_re = regex_lite::Regex::new(r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$")
        .expect("valid regex");
    assert!(
        semver_re.is_match(version),
        "version field must match SemVer 2.0 regex `^\\d+\\.\\d+\\.\\d+(-[0-9A-Za-z.-]+)?(\
        \\+[0-9A-Za-z.-]+)?$`; got: {version:?}"
    );
    assert!(
        !version.starts_with('v'),
        "version field must NOT have a leading 'v' prefix; got: {version:?}"
    );
}

/// VP-001 Post-condition 2: `version` must equal `env!("CARGO_PKG_VERSION")` at compile time.
#[tokio::test]
async fn test_BC_2_01_001_version_equals_cargo_pkg_version() {
    let state = make_running_state();
    let (status, body) = get_healthz_json(state, &[]).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "prerequisite: HTTP 200 in Running mode"
    );
    let version = body
        .get("version")
        .and_then(|v| v.as_str())
        .expect("version field must be a string");
    let expected = env!("CARGO_PKG_VERSION");
    assert_eq!(
        version, expected,
        "version must equal CARGO_PKG_VERSION ({expected}); got: {version:?}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.001 Postcondition 2 / VP-001 Probe 1.d — ShuttingDown mode HTTP 503
// ---------------------------------------------------------------------------

/// VP-001 Probe 1.d: `AppMode::ShuttingDown` → HTTP 503 with `{"status":"shutting_down"}`.
///
/// Exercises BC-2.01.001 Postcondition 2.
/// Counter-example: stub returns 500 (unimplemented); test asserts 503.
#[tokio::test]
async fn test_BC_2_01_001_shutting_down_mode_returns_503() {
    let state = make_shutting_down_state();
    let (status, body) = get_healthz_json(state, &[]).await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "GET /healthz in ShuttingDown mode must return HTTP 503; got {status}. Body: {body}"
    );
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("shutting_down"),
        "body.status must equal \"shutting_down\" in ShuttingDown mode; got: {body}"
    );
}

/// BC-2.01.001 Postcondition 2 canonical body shape: exactly 1 key.
///
/// The canonical body literal from BC-2.01.001 PC-2 is `{"status":"shutting_down"}`.
/// That is a JSON object with exactly ONE key (`"status"`), whose value is `"shutting_down"`.
/// VP-001 §Property Statement describes this as "exactly two keys" — that annotation is
/// a wording error in the VP; the body literal `{"status":"shutting_down"}` has ONE key.
/// The BC body literal is authoritative per the CLAUDE.md Architecture Authority hierarchy.
///
/// Counter-example from VP-001 §Counter-examples item 5:
/// `{"status":"alive","uptime_sec":N,"version":"v","drain":true}` (4 keys) fails this test.
#[tokio::test]
async fn test_BC_2_01_001_shutting_down_body_has_exactly_one_key() {
    let state = make_shutting_down_state();
    let (status, body) = get_healthz_json(state, &[]).await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "prerequisite: HTTP 503 in ShuttingDown mode"
    );
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        1,
        "ShuttingDown body must have EXACTLY 1 key per BC-2.01.001 PC-2 literal \
        `{{\"status\":\"shutting_down\"}}`; got {} key(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
    assert!(
        obj.contains_key("status"),
        "ShuttingDown body must contain key \"status\"; got: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.001 Postcondition 3 / VP-001 Probes 1.a, 1.b, 1.c — Auth header ignored
// ---------------------------------------------------------------------------

/// VP-001 Probe 1.a (already covered above): GET /healthz with no auth header → 200, not 401.
///
/// Explicitly named test to document the no-auth → 200 contract.
#[tokio::test]
async fn test_BC_2_01_001_no_auth_header_returns_200_not_401() {
    let state = make_running_state();
    let req = Request::builder()
        .method("GET")
        .uri("/healthz")
        // Intentionally NO X-Monocle-Authorization header
        .body(Body::empty())
        .expect("build request");
    let router = unauthenticated_router(state);
    let response = router.oneshot(req).await.expect("oneshot");
    let status = response.status();
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /healthz with no auth header must NOT return 401; got {status}. \
        Indicates auth middleware incorrectly applied to unauthenticated router."
    );
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /healthz with no auth header must return 200; got {status}"
    );
}

/// VP-001 Probe 1.b: GET /healthz with a syntactically valid auth header → 200 (header ignored).
///
/// The unauthenticated router does NOT inspect `X-Monocle-Authorization`.
#[tokio::test]
async fn test_BC_2_01_001_valid_auth_header_is_ignored_returns_200() {
    let state = make_running_state();
    // A syntactically valid auth header (64-hex after monocle-v1: prefix).
    let valid_token = format!("monocle-v1:{}", "a".repeat(64));
    let (status, body) =
        get_healthz_json(state, &[("X-Monocle-Authorization", valid_token.as_str())]).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /healthz with valid auth header must return 200 (header ignored); \
        got {status}. Body: {body}"
    );
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("alive"),
        "body.status must be \"alive\" even when auth header present; got: {body}"
    );
}

/// VP-001 Probe 1.c: GET /healthz with garbage auth header → 200, not 401.
///
/// Unauthenticated router does not run auth middleware; garbage header is silently ignored.
#[tokio::test]
async fn test_BC_2_01_001_garbage_auth_header_is_ignored_returns_200() {
    let state = make_running_state();
    let (status, body) =
        get_healthz_json(state, &[("X-Monocle-Authorization", "not-a-valid-token")]).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /healthz with garbage auth header must return 200 (not 401); got {status}. \
        Body: {body}. Counter-example: being on the authenticated router would return 401."
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.001 Postcondition 4 / AC-004 — No DefaultBodyLimit on unauthenticated router
// ---------------------------------------------------------------------------

/// AC-004: Sending a body >256 KiB to `GET /healthz` must NOT produce HTTP 413.
///
/// `DefaultBodyLimit::max(256 * 1024)` is applied to the authenticated router only.
/// The unauthenticated router has no body limit.
/// A 1 MiB body (≈ 4x the authenticated limit) must succeed with HTTP 200.
#[tokio::test]
async fn test_BC_2_01_001_large_body_returns_200_not_413() {
    let state = make_running_state();
    // 1 MiB body (4× the 256 KiB authenticated limit)
    let status = get_healthz_with_body(state, 1_048_576).await;
    assert_ne!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "GET /healthz must NOT return 413 for large body; DefaultBodyLimit must NOT be \
        applied to the unauthenticated router (BC-2.01.001 PC-4)"
    );
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /healthz with 1 MiB body must return 200; got {status}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.001 Invariant 2 / AC-005 / VP-001 Probe 1.e — Structural: body limit on auth only
// ---------------------------------------------------------------------------

/// VP-001 Probe 1.e: Source-grep asserts `DefaultBodyLimit` does NOT appear in non-comment
/// lines of the unauthenticated router source (`router.rs`) or the healthz handler source.
///
/// This is a static structural invariant. It does NOT require a running server.
///
/// In S-002, the only router file is `src/router.rs`. We assert:
/// 1. The `unauthenticated_router` function source does NOT call `DefaultBodyLimit`
///    in non-comment, non-doc code lines.
/// 2. No DefaultBodyLimit usage appears in the healthz handler source code lines.
///
/// The authenticated router (S-003+) MUST declare DefaultBodyLimit. That assertion
/// lives in S-003's test file. Here we only assert the negative (it must NOT appear
/// as executable code in the unauthenticated router).
#[test]
fn test_BC_2_01_001_invariant_default_body_limit_on_auth_router_only() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let router_src =
        fs::read_to_string(manifest_dir.join("src/router.rs")).expect("src/router.rs must exist");
    let healthz_src = fs::read_to_string(manifest_dir.join("src/handlers/healthz.rs"))
        .expect("src/handlers/healthz.rs must exist");

    // Helper: collect non-comment lines that contain the target string.
    // "Comment" means lines where the first non-whitespace character(s) are `//`.
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

    // Assert DefaultBodyLimit does NOT appear in non-comment router.rs lines.
    let router_hits = non_comment_matches(&router_src, "DefaultBodyLimit");
    assert!(
        router_hits.is_empty(),
        "router.rs (unauthenticated router) must NOT use DefaultBodyLimit in executable \
        code; found non-comment references at lines: {:?}. The body limit belongs on the \
        authenticated router only (BC-2.01.001 Invariant 2 / VP-001 Probe 1.e).",
        router_hits
    );

    // Assert DefaultBodyLimit does NOT appear in non-comment healthz.rs lines.
    let healthz_hits = non_comment_matches(&healthz_src, "DefaultBodyLimit");
    assert!(
        healthz_hits.is_empty(),
        "handlers/healthz.rs must NOT use DefaultBodyLimit in executable code; \
        found non-comment references at lines: {:?}",
        healthz_hits
    );
}

/// Structural invariant: healthz.rs must NOT import `constant_time_eq` (auth path only).
///
/// From story Architecture Compliance Rules: "This handler MUST NOT import `constant_time_eq`."
/// We scan non-comment lines only to avoid matching the doc-comment constraint itself.
#[test]
fn test_BC_2_01_001_invariant_healthz_does_not_import_constant_time_eq() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let healthz_src = fs::read_to_string(manifest_dir.join("src/handlers/healthz.rs"))
        .expect("src/handlers/healthz.rs must exist");
    // Scan only non-comment, non-doc lines for actual import usage.
    let non_comment_hits: Vec<(usize, &str)> = healthz_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && line.contains("constant_time_eq")
        })
        .map(|(i, line)| (i + 1, line))
        .collect();
    assert!(
        non_comment_hits.is_empty(),
        "handlers/healthz.rs must NOT import or use constant_time_eq in executable code \
        (auth path only); found non-comment references at lines: {:?}",
        non_comment_hits
    );
}

/// Structural invariant: healthz.rs must NOT import from monocle-tui (not a Phase 1 crate).
/// We scan non-comment lines only to avoid matching the doc-comment constraint itself.
#[test]
fn test_BC_2_01_001_invariant_healthz_does_not_import_monocle_tui() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let healthz_src = fs::read_to_string(manifest_dir.join("src/handlers/healthz.rs"))
        .expect("src/handlers/healthz.rs must exist");
    // Scan only non-comment, non-doc lines for actual import usage.
    let non_comment_hits: Vec<(usize, String)> = healthz_src
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
        "handlers/healthz.rs must NOT import from monocle-tui (not a Phase 1 crate); \
        found non-comment references at lines: {:?}",
        non_comment_hits
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.001 EC-040 / AC-006 — Response within 100ms under no load
// ---------------------------------------------------------------------------

/// AC-006 (EC-040): `GET /healthz` in normal mode completes within 100ms under no load.
///
/// Daemon-side liveness observable by the TUI hung-daemon detection path:
/// if `/healthz` does not respond within 100ms from a live daemon, the TUI
/// initiates recovery (EC-040). The daemon must guarantee response latency < 100ms.
///
/// This test binds a real TCP listener on port 0 (OS-assigned) and issues a real
/// HTTP request over localhost to exercise the full axum `serve` stack.
/// Pattern: VP-001 L63 "axum::serve on tokio::net::TcpListener bound to 127.0.0.1:0".
#[tokio::test]
async fn test_BC_2_01_001_response_within_100ms() {
    use tokio::net::TcpListener;
    use tokio::time::{timeout, Duration, Instant as TokioInstant};

    let state = make_running_state();
    let router = unauthenticated_router(state);

    // Bind to port 0 — OS assigns an ephemeral port.
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind TcpListener to 127.0.0.1:0");
    let addr = listener.local_addr().expect("get local addr");

    // Spawn the axum server in the background.
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("axum serve");
    });

    // Build a reqwest client and measure the round-trip latency.
    let client = reqwest::Client::new();
    let url = format!("http://{addr}/healthz");

    let start = TokioInstant::now();
    let result = timeout(Duration::from_millis(100), client.get(&url).send()).await;
    let elapsed = start.elapsed();

    let response = result
        .expect("GET /healthz must complete within 100ms (timeout exceeded)")
        .expect("HTTP request must succeed");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "GET /healthz within 100ms must return 200; got {}",
        response.status()
    );
    assert!(
        elapsed < Duration::from_millis(100),
        "GET /healthz must respond within 100ms under no load (EC-040 TUI hung-daemon \
        detection boundary); elapsed: {elapsed:?}"
    );
}

// ---------------------------------------------------------------------------
// VP-001 auxiliary proptest — semver regex property across random token forms
// ---------------------------------------------------------------------------

/// VP-001 Proof Method §proptest (auxiliary): The `version` field returned by the handler
/// always matches the semver regex for any valid CARGO_PKG_VERSION form.
///
/// The implementation uses a compile-time `env!("CARGO_PKG_VERSION")` value.
/// We verify the regex directly against known-valid semver forms and known-invalid forms
/// to ensure the regex itself is correctly specified.
///
/// Note: proptest requires the `proptest` crate. Since this is not yet in workspace
/// dependencies (it would be added in Phase 6 formal hardening), we encode the
/// property test using a hand-enumerated boundary corpus sufficient for Red Gate.
/// The proptest crate dependency is tracked in the VP-001 proof method; Phase 6
/// formal-verifier will upgrade to proptest!{} macros.
#[test]
fn test_BC_2_01_001_invariant_semver_regex_shape() {
    let semver_re = regex_lite::Regex::new(r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$")
        .expect("valid semver regex");

    // Valid semver forms — all must match
    let valid_forms = [
        "0.1.0",
        "1.0.0",
        "0.0.1",
        "1.2.3",
        "10.20.30",
        "0.1.0-alpha",
        "0.1.0-alpha.1",
        "0.1.0-0.3.7",
        "0.1.0-x.7.z.92",
        "1.0.0-rc.1",
        "1.0.0+build.1",
        "1.0.0-beta+exp.sha.5114f85",
    ];
    for form in valid_forms {
        assert!(
            semver_re.is_match(form),
            "Expected semver regex to match valid form {form:?}"
        );
    }

    // Invalid forms — none must match
    let invalid_forms = [
        "v0.1.0",  // leading v prefix (forbidden by BC-2.01.001 PC-1)
        "v1.0.0",  // leading v prefix
        "debug",   // build profile string (counter-example from VP-001 §Counter-examples 4)
        "0.1",     // missing patch
        "1.2.3.4", // four components
        "1.0.0-",  // trailing hyphen only
        "1.0.0+",  // trailing plus only
        "",        // empty
    ];
    for form in invalid_forms {
        assert!(
            !semver_re.is_match(form),
            "Expected semver regex NOT to match invalid form {form:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// BC-2.01.001 defensive handling — RwLock poisoning → graceful degradation
// ---------------------------------------------------------------------------

/// BC-2.01.001 defensive handling: when the `mode` RwLock is poisoned (write guard held
/// by a panicking thread), the handler must NOT propagate the panic to the caller.
/// Instead it must degrade gracefully to `{"status":"shutting_down"}` with HTTP 503,
/// treating a poisoned lock as equivalent to `AppMode::ShuttingDown` (safe sentinel).
///
/// Poisoning technique: spawn a thread that acquires the write guard and then panics.
/// After `join()`, the lock is poisoned. Any subsequent `read().unwrap()` will itself
/// panic — the implementation must use `read().unwrap_or_else(|e| e.into_inner())` or
/// an equivalent poison-tolerant read to uphold this contract.
#[tokio::test]
async fn test_BC_2_01_001_poisoned_lock_returns_503() {
    use std::time::Instant;

    // Build a DaemonState whose mode lock we will poison.
    let state = Arc::new(DaemonState {
        mode: std::sync::RwLock::new(AppMode::Running),
        start_time: Instant::now(),
        hook_receiver_status: None,
    });

    // Poison the lock by spawning an OS thread that panics while holding the write guard.
    let state_clone = Arc::clone(&state);
    let handle = std::thread::spawn(move || {
        let _guard = state_clone.mode.write().unwrap();
        panic!("deliberate lock poisoning for test_BC_2_01_001_poisoned_lock_returns_503");
    });
    // Wait for the thread to complete (and panic). The join error is expected.
    let _ = handle.join();
    // At this point, state.mode is poisoned. Any .unwrap() on read() would panic.

    let router = unauthenticated_router(state);
    let req = Request::builder()
        .method("GET")
        .uri("/healthz")
        .body(Body::empty())
        .expect("build GET /healthz");

    let response = router.oneshot(req).await.expect("oneshot must not propagate panic");
    let status = response.status();

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let body: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response must be valid JSON even on poisoned lock");

    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "poisoned RwLock must degrade to HTTP 503 (ShuttingDown sentinel); got {status}. \
        Body: {body}. Counter-example: propagating PoisonError as HTTP 500 violates graceful \
        degradation contract of BC-2.01.001."
    );
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("shutting_down"),
        "poisoned lock body must contain {{\"status\":\"shutting_down\"}}; got: {body}"
    );
}
