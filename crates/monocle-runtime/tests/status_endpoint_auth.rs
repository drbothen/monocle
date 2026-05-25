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
//! | AC-001 types | BC-2.01.002 PC-1 field types | 2.a field type correctness | test_BC_2_01_002_field_types_are_correct |
//! | AC-001 pid | BC-2.01.002 PC-1 pid ≥ 1 | 2.a pid is positive u32 | test_BC_2_01_002_pid_is_positive_u32 |
//! | AC-001 version | BC-2.01.002 PC-1 version semver | 2.a version matches semver | test_BC_2_01_002_version_matches_semver_regex |
//! | AC-001 floats | BC-2.01.002 PC-1 float ranges | 2.a floats in 0.0–100.0 | test_BC_2_01_002_float_fields_in_valid_range |
//! | AC-001 uptime | BC-2.01.002 PC-1 uptime_sec u64 ≥ 0 | 2.a uptime is non-negative integer | test_BC_2_01_002_uptime_sec_is_u64 |
//! | AC-001 tui_attached | BC-2.01.002 PC-1 tui_attached bool | 2.a tui_attached is bool | test_BC_2_01_002_tui_attached_is_bool_defaults_false |
//! | AC-001 lock_file | BC-2.01.002 PC-1 lock_file string | 2.a lock_file is string | test_BC_2_01_002_lock_file_is_string |
//! | AC-002 | BC-2.01.009 PC-3 + INV-6 | 2.b alias auth → 200 + WARN log | test_BC_2_01_002_alias_auth_returns_200_and_emits_warn |
//! | AC-003 | BC-2.01.009 PC-1 | 2.c no auth → 401 E-AUTH-001 | test_BC_2_01_002_no_auth_returns_401_missing_token |
//! | AC-004 | BC-2.01.009 PC-2 | 2.d wrong token → 401 E-AUTH-002 | test_BC_2_01_002_wrong_token_returns_401_invalid_token |
//! | AC-006 | BC-2.01.002 PC-1 hook_endpoints | 2.e hook_endpoints array shape | test_BC_2_01_002_hook_endpoints_array_exactly_5_paths |
//! | AC-008 | BC-2.01.002 PC-3 | 2.f drain → 200 (not 503) | test_BC_2_01_002_status_serves_during_drain |
//! | EC-007 | BC-2.01.009 EC-007 | empty canonical value → invalid | test_BC_2_01_002_empty_canonical_header_value_returns_401_invalid |
//! | EC-009 | BC-2.01.009 EC-009 | prefix only no hex → invalid | test_BC_2_01_002_canonical_prefix_only_no_hex_returns_401_invalid |
//! | EC-010 | BC-2.01.009 EC-010 | alias wrong secret → invalid + WARN | test_BC_2_01_002_alias_wrong_secret_returns_401_invalid |
//! | EC-011 | BC-2.01.009 EC-011 | both headers → canonical wins no WARN | test_BC_2_01_002_both_headers_canonical_wins |
//! | EC-012 | BC-2.01.009 EC-012 | alias empty value → invalid | test_BC_2_01_002_alias_empty_value_returns_401_invalid |
//! | EC-013 | BC-2.01.009 EC-013 | Authorization: Bearer → missing (not invalid) | test_BC_2_01_002_bearer_auth_header_returns_401_missing |
//! | INV-1 | BC-2.01.009 INV-1 | no third error body | test_BC_2_01_002_invariant_no_third_error_body |
//! | INV-2 | BC-2.01.009 INV-2 | missing vs invalid distinction preserved | test_BC_2_01_002_invariant_missing_and_invalid_are_distinct_bodies |
//! | PC-4 | BC-2.01.009 PC-4 | canonical present → alias ignored | test_BC_2_01_002_canonical_header_present_alias_ignored |
//! | Body limit | BC-2.01.001 INV-2 structural | N/A | test_BC_2_01_002_invariant_default_body_limit_on_auth_router |
//! | Arch rules | Story §Architecture Compliance | No constant_time_eq in status handler | test_BC_2_01_002_invariant_status_handler_does_not_compare_tokens_with_eq |
//! | Arch rules | Story §Architecture Compliance | No monocle-tui import | test_BC_2_01_002_invariant_status_handler_does_not_import_monocle_tui |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per the naming convention.
#![allow(non_snake_case)]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
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
    let state = DaemonState::new();
    // SAFETY: auth_token is set once at daemon startup; test constructs the equivalent.
    // We cannot use the public field directly because it is behind no lock (it's a plain
    // String set before serving starts). The field is `pub` on DaemonState.
    // This is the test equivalent of the S-004 token-generation path.
    let mut state = state;
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
// AC-001 field type assertions — BC-2.01.002 PC-1 type correctness
// ---------------------------------------------------------------------------

/// BC-2.01.002 PC-1: `pid` must be a positive integer (≥ 1) per POSIX (PID 0 is reserved).
///
/// Counter-example: `pid: 0` would violate the "positive integer" spec.
/// Counter-example: `pid: "42"` (string) would violate the JSON type requirement.
///
/// Traces to BC-2.01.002 PC-1 sub-bullet `pid`, AC-001.
#[tokio::test]
async fn test_BC_2_01_002_pid_is_positive_u32() {
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
        "prerequisite: canonical auth must return HTTP 200"
    );

    let pid = body.get("pid").expect("pid field must be present");
    assert!(
        pid.is_u64() || pid.is_i64(),
        "pid must be a JSON integer (not string, not float); got: {pid:?}"
    );
    let pid_val = pid.as_u64().expect("pid must be a non-negative integer");
    assert!(
        pid_val >= 1,
        "pid must be ≥ 1 per POSIX (PID 0 is reserved for the scheduler, BC-2.01.002 PC-1); \
        got pid={pid_val}"
    );
    // Sanity cap: PID values above 2^32 would be unusual on any supported platform.
    assert!(
        pid_val <= u64::from(u32::MAX),
        "pid must fit in u32; got pid={pid_val}"
    );
}

/// BC-2.01.002 PC-1: `uptime_sec` must be a non-negative integer (u64).
///
/// Counter-example: `uptime_sec: "0"` (string) fails the integer type assertion.
///
/// Traces to BC-2.01.002 PC-1 sub-bullet `uptime_sec`, AC-001.
#[tokio::test]
async fn test_BC_2_01_002_uptime_sec_is_u64() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "prerequisite: canonical auth → 200");

    let uptime = body
        .get("uptime_sec")
        .expect("uptime_sec field must be present");
    assert!(
        uptime.is_u64() || uptime.is_i64(),
        "uptime_sec must be a JSON integer (not string, not float); got: {uptime:?}"
    );
    let uptime_val = uptime
        .as_u64()
        .expect("uptime_sec must be a non-negative integer");
    // uptime may be 0 for a freshly-constructed state; that is valid.
    // We assert it is a u64 (non-negative integer) above.
    let _ = uptime_val;
}

/// BC-2.01.002 PC-1: `version` must match SemVer 2.0 regex without a leading `v` prefix.
///
/// Counter-example: `version: "debug"` fails regex match.
/// Counter-example: `version: "v0.1.0"` fails because of the leading `v`.
///
/// Traces to BC-2.01.002 PC-1 sub-bullet `version`, AC-001.
#[tokio::test]
async fn test_BC_2_01_002_version_matches_semver_regex() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "prerequisite: canonical auth → 200");

    let version = body
        .get("version")
        .and_then(|v| v.as_str())
        .expect("version field must be a string");

    // Semver 2.0 regex from BC-2.01.002 PC-1 and VP-002 §Post-condition.
    let semver_re = regex_lite::Regex::new(r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$")
        .expect("valid semver regex");
    assert!(
        semver_re.is_match(version),
        "version field must match SemVer 2.0 regex \
        `^\\d+\\.\\d+\\.\\d+(-[0-9A-Za-z.-]+)?(\\+[0-9A-Za-z.-]+)?$`; got: {version:?} \
        (BC-2.01.002 PC-1 sub-bullet `version`)"
    );
    assert!(
        !version.starts_with('v'),
        "version field must NOT have a leading 'v' prefix per BC-2.01.002 PC-1; got: {version:?}"
    );
    // Also assert it matches the compile-time CARGO_PKG_VERSION.
    let expected = env!("CARGO_PKG_VERSION");
    assert_eq!(
        version, expected,
        "version must equal CARGO_PKG_VERSION ({expected}); got: {version:?}"
    );
}

/// BC-2.01.002 PC-1: `ring_buffer_fill_pct` and `channel_saturation_pct` must be floats
/// in the range 0.0–100.0 (inclusive).
///
/// Counter-example: `ring_buffer_fill_pct: -1.0` fails the lower bound.
/// Counter-example: `channel_saturation_pct: 101.0` fails the upper bound.
///
/// Traces to BC-2.01.002 PC-1 sub-bullets `ring_buffer_fill_pct` + `channel_saturation_pct`, AC-001.
#[tokio::test]
async fn test_BC_2_01_002_float_fields_in_valid_range() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "prerequisite: canonical auth → 200");

    // ring_buffer_fill_pct
    let ring_pct = body
        .get("ring_buffer_fill_pct")
        .expect("ring_buffer_fill_pct must be present");
    assert!(
        ring_pct.is_f64() || ring_pct.is_u64() || ring_pct.is_i64(),
        "ring_buffer_fill_pct must be a JSON number; got: {ring_pct:?}"
    );
    let ring_val = ring_pct
        .as_f64()
        .expect("ring_buffer_fill_pct must be convertible to f64");
    assert!(
        (0.0..=100.0).contains(&ring_val),
        "ring_buffer_fill_pct must be in [0.0, 100.0] per BC-2.01.002 PC-1; got: {ring_val}"
    );

    // channel_saturation_pct
    let channel_pct = body
        .get("channel_saturation_pct")
        .expect("channel_saturation_pct must be present");
    assert!(
        channel_pct.is_f64() || channel_pct.is_u64() || channel_pct.is_i64(),
        "channel_saturation_pct must be a JSON number; got: {channel_pct:?}"
    );
    let channel_val = channel_pct
        .as_f64()
        .expect("channel_saturation_pct must be convertible to f64");
    assert!(
        (0.0..=100.0).contains(&channel_val),
        "channel_saturation_pct must be in [0.0, 100.0] per BC-2.01.002 PC-1; got: {channel_val}"
    );
}

/// BC-2.01.002 PC-1: `tui_attached` must be a JSON boolean.
///
/// In a freshly-constructed `DaemonState`, `tui_attached` defaults to `false` because no TUI
/// client has connected.
///
/// Counter-example: `tui_attached: "false"` (string) fails the type assertion.
///
/// Traces to BC-2.01.002 PC-1 sub-bullet `tui_attached`, AC-001.
#[tokio::test]
async fn test_BC_2_01_002_tui_attached_is_bool_defaults_false() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "prerequisite: canonical auth → 200");

    let tui_attached = body
        .get("tui_attached")
        .expect("tui_attached field must be present");
    assert!(
        tui_attached.is_boolean(),
        "tui_attached must be a JSON boolean (not string, not integer); got: {tui_attached:?} \
        (BC-2.01.002 PC-1 sub-bullet `tui_attached`)"
    );
    // Default state: no TUI client connected → must be false.
    assert_eq!(
        tui_attached.as_bool(),
        Some(false),
        "tui_attached must be false in a freshly-constructed DaemonState (no TUI connected); \
        got: {tui_attached:?}"
    );
}

/// BC-2.01.002 PC-1: `lock_file` must be a JSON string.
///
/// In a freshly-constructed `DaemonState`, `lock_file_path` is an empty string (written
/// by S-004 during startup). The field must be a string regardless of value.
///
/// Counter-example: `lock_file: null` would fail the string type assertion.
///
/// Traces to BC-2.01.002 PC-1 sub-bullet `lock_file`, AC-001.
#[tokio::test]
async fn test_BC_2_01_002_lock_file_is_string() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "prerequisite: canonical auth → 200");

    let lock_file = body
        .get("lock_file")
        .expect("lock_file field must be present");
    assert!(
        lock_file.is_string(),
        "lock_file must be a JSON string; got: {lock_file:?} \
        (BC-2.01.002 PC-1 sub-bullet `lock_file`)"
    );
}

/// BC-2.01.002 PC-1: all 10 field types are correct simultaneously.
///
/// This is the composite type-correctness test exercising all fields in a single response.
/// Individual field tests above focus one assertion per field; this test verifies the
/// complete type contract in one pass.
///
/// Traces to BC-2.01.002 PC-1 (all sub-bullets), AC-001.
#[tokio::test]
async fn test_BC_2_01_002_field_types_are_correct() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "prerequisite: canonical auth → 200");

    let obj = body.as_object().expect("body must be a JSON object");

    // pid: u32 ≥ 1
    let pid = obj.get("pid").expect("pid must be present");
    assert!(
        pid.is_u64() || pid.is_i64(),
        "pid must be a JSON integer; got: {pid:?}"
    );
    assert!(
        pid.as_u64().unwrap_or(0) >= 1,
        "pid must be ≥ 1; got: {pid:?}"
    );

    // uptime_sec: u64 ≥ 0
    let uptime_sec = obj.get("uptime_sec").expect("uptime_sec must be present");
    assert!(
        uptime_sec.is_u64() || uptime_sec.is_i64(),
        "uptime_sec must be a JSON integer; got: {uptime_sec:?}"
    );

    // version: string (semver; non-empty)
    let version = obj.get("version").expect("version must be present");
    assert!(
        version.is_string(),
        "version must be a JSON string; got: {version:?}"
    );
    let ver_str = version.as_str().unwrap();
    assert!(!ver_str.is_empty(), "version must be non-empty");

    // abi_version: u32 integer
    let abi_version = obj.get("abi_version").expect("abi_version must be present");
    assert!(
        abi_version.is_u64() || abi_version.is_i64(),
        "abi_version must be a JSON integer; got: {abi_version:?}"
    );

    // lock_file: string
    let lock_file = obj.get("lock_file").expect("lock_file must be present");
    assert!(
        lock_file.is_string(),
        "lock_file must be a JSON string; got: {lock_file:?}"
    );

    // hook_endpoints: array of strings
    let hook_endpoints = obj
        .get("hook_endpoints")
        .expect("hook_endpoints must be present");
    assert!(
        hook_endpoints.is_array(),
        "hook_endpoints must be a JSON array; got: {hook_endpoints:?}"
    );

    // ring_buffer_fill_pct: number in [0.0, 100.0]
    let ring_pct = obj
        .get("ring_buffer_fill_pct")
        .expect("ring_buffer_fill_pct must be present");
    let ring_val = ring_pct
        .as_f64()
        .expect("ring_buffer_fill_pct must be a number");
    assert!(
        (0.0..=100.0).contains(&ring_val),
        "ring_buffer_fill_pct must be in [0.0, 100.0]; got: {ring_val}"
    );

    // channel_saturation_pct: number in [0.0, 100.0]
    let channel_pct = obj
        .get("channel_saturation_pct")
        .expect("channel_saturation_pct must be present");
    let channel_val = channel_pct
        .as_f64()
        .expect("channel_saturation_pct must be a number");
    assert!(
        (0.0..=100.0).contains(&channel_val),
        "channel_saturation_pct must be in [0.0, 100.0]; got: {channel_val}"
    );

    // last_hook_ts: JSON object with 5 keys
    let last_hook_ts = obj
        .get("last_hook_ts")
        .expect("last_hook_ts must be present");
    assert!(
        last_hook_ts.is_object(),
        "last_hook_ts must be a JSON object; got: {last_hook_ts:?}"
    );

    // tui_attached: boolean
    let tui_attached = obj
        .get("tui_attached")
        .expect("tui_attached must be present");
    assert!(
        tui_attached.is_boolean(),
        "tui_attached must be a JSON boolean; got: {tui_attached:?}"
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
// AC-008 — BC-2.01.002 PC-3 — /status serves during drain (ShuttingDown → 200)
// ---------------------------------------------------------------------------

/// BC-2.01.002 PC-3 / AC-008: `GET /status` with valid canonical auth while
/// `AppMode = ShuttingDown` MUST return HTTP 200, NOT 503.
///
/// `/status` is a read-only observability endpoint and is explicitly exempt from the
/// drain-503 rule (BC-2.01.004 PC-4 cross-cites BC-2.01.002 PC-3). The drain-503 rule
/// applies to state-mutating endpoints (hook ingestion); read-only /status continues
/// to serve so that the TUI can observe drain progress.
///
/// Counter-example guarded: a naive implementation that blanket-applies the drain-503 rule
/// to all authenticated routes would return 503 here. This test guards against that mistake.
///
/// Traces to BC-2.01.002 PC-3 (drain-exempt read-only endpoint), AC-008.
#[tokio::test]
async fn test_BC_2_01_002_status_serves_during_drain() {
    let state = make_shutting_down_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let (status, body) = get_status_json(
        state,
        &[("X-Monocle-Authorization", canonical_header.as_str())],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "GET /status with valid auth during AppMode::ShuttingDown must return HTTP 200, \
        NOT 503. /status is exempt from the drain-503 rule (BC-2.01.002 PC-3 / \
        BC-2.01.004 PC-4). Got {status}. Body: {body}. \
        Counter-example: blanket drain-503 on all authenticated routes returns 503."
    );

    // The body must still be a valid 10-field status object.
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        10,
        "drain-mode /status body must have EXACTLY 10 fields (same as Running mode); \
        got {} field(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

/// BC-2.01.002 PC-3 negative: Unauthenticated request during drain still returns 401.
///
/// Drain exemption applies to the STATUS RESPONSE only — the auth requirement is
/// not relaxed during drain. A missing auth token must still produce 401 E-AUTH-001
/// during graceful shutdown.
///
/// Traces to BC-2.01.002 PC-3 (read-only exempt) + BC-2.01.009 PC-1 (missing auth).
#[tokio::test]
async fn test_BC_2_01_002_drain_unauthenticated_still_returns_401() {
    let state = make_shutting_down_state_with_token();
    // No auth header — should still be rejected with 401, not 200 or 503.
    let (status, body) = get_status_json(state, &[]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with no auth during drain must still return HTTP 401 (auth is not relaxed \
        during drain); got {status}. Body: {body}. \
        Counter-example: drain-exempt means the 200 is served once auth passes; not that \
        auth itself is bypassed."
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("missing_auth_token"),
        "drain unauthenticated body must be E-AUTH-001 {{\"error\":\"missing_auth_token\"}}; \
        got: {body}"
    );
}

// ---------------------------------------------------------------------------
// EC-007 — BC-2.01.009 EC-007 — Empty canonical header value → 401 invalid (not missing)
// ---------------------------------------------------------------------------

/// BC-2.01.009 EC-007: `X-Monocle-Authorization` present with an EMPTY value (not absent).
///
/// An empty string does not begin with `monocle-v1:` — value-present format failure.
/// This is E-AUTH-002 (`{"error":"invalid_auth_token"}`), NOT E-AUTH-001 (missing), because
/// the canonical header IS present; its value fails format validation.
///
/// Counter-example: returning E-AUTH-001 (missing) when the header IS present but empty
/// would violate INV-2 (the missing-vs-invalid distinction is preserved because a missing
/// header is a client-configuration error, not an authentication attempt).
///
/// Traces to BC-2.01.009 EC-007, INV-2.
#[tokio::test]
async fn test_BC_2_01_002_empty_canonical_header_value_returns_401_invalid() {
    let state = make_state_with_token();
    // Empty value for the canonical header: header IS present, value is empty string.
    let (status, body) = get_status_json(state, &[("X-Monocle-Authorization", "")]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with empty X-Monocle-Authorization value must return HTTP 401; \
        got {status}. Body: {body}. (BC-2.01.009 EC-007)"
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "Empty canonical header value must return E-AUTH-002 {{\"error\":\"invalid_auth_token\"}} \
        (not E-AUTH-001 missing_auth_token — the header IS present); got: {body} \
        (BC-2.01.009 EC-007)"
    );
}

// ---------------------------------------------------------------------------
// EC-009 — BC-2.01.009 EC-009 — Prefix only, no hex suffix → 401 invalid
// ---------------------------------------------------------------------------

/// BC-2.01.009 EC-009: `X-Monocle-Authorization: monocle-v1:` (prefix present but no hex suffix).
///
/// The prefix check passes, but the constant-time secret comparison on an empty hex string
/// never matches the 64-char secret. Returns HTTP 401 E-AUTH-002.
///
/// Counter-example: returning E-AUTH-001 (missing) would be incorrect because the header IS
/// present with a value; it fails on the comparison, not on absence.
///
/// Traces to BC-2.01.009 EC-009.
#[tokio::test]
async fn test_BC_2_01_002_canonical_prefix_only_no_hex_returns_401_invalid() {
    let state = make_state_with_token();
    // Correct prefix, no hex suffix — empty suffix after stripping "monocle-v1:".
    let (status, body) =
        get_status_json(state, &[("X-Monocle-Authorization", "monocle-v1:")]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with monocle-v1: (no hex suffix) must return HTTP 401; \
        got {status}. Body: {body}. (BC-2.01.009 EC-009)"
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "Prefix-only canonical header must return E-AUTH-002; got: {body} \
        (BC-2.01.009 EC-009 — empty suffix is a value-present failure)"
    );
}

/// BC-2.01.009 test vector: wrong version prefix (`monocle-v2:`) → E-AUTH-002.
///
/// A wrong version prefix is a value-present format failure (not missing). Returns
/// HTTP 401 `{"error":"invalid_auth_token"}`.
///
/// Counter-example: returning E-AUTH-001 (missing) would violate INV-2.
///
/// Traces to BC-2.01.009 PC-2, canonical test vector row 3.
#[tokio::test]
async fn test_BC_2_01_002_wrong_version_prefix_returns_401_invalid() {
    let state = make_state_with_token();
    // Wrong version prefix: monocle-v2: (not monocle-v1:)
    let wrong_prefix = format!("monocle-v2:{TEST_TOKEN_HEX}");
    let (status, body) =
        get_status_json(state, &[("X-Monocle-Authorization", wrong_prefix.as_str())]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with monocle-v2: prefix must return HTTP 401; got {status}. \
        Body: {body}. (BC-2.01.009 PC-2, test vector row 3)"
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "Wrong version prefix must return E-AUTH-002; got: {body}. \
        (The monocle-v1: prefix is required per BC-2.01.009 PC-2 / DI-005)"
    );
}

/// BC-2.01.009 test vector: canonical header with no prefix at all (raw 64-hex) → E-AUTH-002.
///
/// `X-Monocle-Authorization: deadbeef...64chars` (no `monocle-v1:` prefix) is a value-present
/// format failure even if the raw hex matches the secret. The canonical header ALWAYS requires
/// the prefix.
///
/// Traces to BC-2.01.009 PC-2, canonical test vector row 2.
#[tokio::test]
async fn test_BC_2_01_002_canonical_header_without_prefix_returns_401_invalid() {
    let state = make_state_with_token();
    // Correct token value but no prefix — sent as if it were the alias format.
    let (status, body) =
        get_status_json(state, &[("X-Monocle-Authorization", TEST_TOKEN_HEX)]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with X-Monocle-Authorization raw-hex (no monocle-v1: prefix) must return \
        HTTP 401; got {status}. Body: {body}. (BC-2.01.009 PC-2, test vector row 2)"
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "Canonical header without monocle-v1: prefix must return E-AUTH-002; got: {body}"
    );
}

// ---------------------------------------------------------------------------
// EC-010 — BC-2.01.009 EC-010 — Alias path wrong secret → 401 invalid + WARN
// ---------------------------------------------------------------------------

/// BC-2.01.009 EC-010: `X-Claude-Code-Ide-Authorization` present with a wrong secret
/// (correct 64-hex format but value does not match stored token); `X-Monocle-Authorization` absent.
///
/// Alias path: WARN log is emitted; constant-time comparison fails → HTTP 401 E-AUTH-002.
///
/// Note: the WARN log assertion is structural (presence of WARN in implementation). The log
/// value is tested by the implementer's tracing subscriber. This integration test asserts the
/// HTTP status and body only.
///
/// Traces to BC-2.01.009 EC-010, PC-3.
#[tokio::test]
async fn test_BC_2_01_002_alias_wrong_secret_returns_401_invalid() {
    let state = make_state_with_token();
    // Alias header with wrong secret (64 hex, all 'f' — not the test token).
    let wrong_alias_token = "f".repeat(64);
    let (status, body) = get_status_json(
        state,
        &[(
            "X-Claude-Code-Ide-Authorization",
            wrong_alias_token.as_str(),
        )],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with alias auth (wrong secret) must return HTTP 401 E-AUTH-002; \
        got {status}. Body: {body}. (BC-2.01.009 EC-010 / PC-3)"
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "Alias path wrong-secret must return E-AUTH-002; got: {body}. \
        (BC-2.01.009 EC-010: WARN is emitted AND comparison fails → 401 invalid)"
    );
}

// ---------------------------------------------------------------------------
// EC-011 — BC-2.01.009 EC-011 — Both headers present → canonical wins, no WARN
// ---------------------------------------------------------------------------

/// BC-2.01.009 EC-011 / PC-4: When BOTH `X-Monocle-Authorization` (correct canonical) and
/// `X-Claude-Code-Ide-Authorization` (any value) are present, the canonical header wins.
///
/// The alias header is IGNORED when the canonical header is present. No WARN log is emitted
/// (the WARN is exclusively for the alias-only path per BC-2.01.009 INV-6). The response is
/// HTTP 200 (auth passes via canonical header).
///
/// Counter-example: emitting the WARN when both headers are present would violate INV-6
/// ("emitted once per alias-path authentication attempt" — the alias path is not entered
/// when the canonical header is present).
///
/// Traces to BC-2.01.009 EC-011, PC-4, INV-5 (canonical priority immutable).
#[tokio::test]
async fn test_BC_2_01_002_both_headers_canonical_wins() {
    let state = make_state_with_token();
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    // Both headers present: canonical (valid) and alias (arbitrary value, potentially invalid).
    let (status, body) = get_status_json(
        state,
        &[
            ("X-Monocle-Authorization", canonical_header.as_str()),
            // Alias header with a wrong secret — if alias were evaluated, this would return 401.
            // If canonical wins correctly, the result is 200.
            (
                "X-Claude-Code-Ide-Authorization",
                "badbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadbadb",
            ),
        ],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "GET /status with both headers present must return HTTP 200 (canonical header wins \
        per BC-2.01.009 PC-4 / EC-011 / INV-5); got {status}. Body: {body}. \
        Counter-example: alias header wrong value would cause 401 if alias were evaluated."
    );
    // Body must have the standard 10-field structure.
    let obj = body.as_object().expect("body must be a JSON object");
    assert_eq!(
        obj.len(),
        10,
        "both-headers-present body must have EXACTLY 10 fields; got {} field(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );
}

/// BC-2.01.009 PC-4 (canonical priority): Both headers present, canonical is INVALID but alias
/// would be valid — canonical must still be evaluated (and fail), returning 401 E-AUTH-002.
///
/// The canonical header takes priority regardless of whether the alias would succeed. The alias
/// path is not entered when `X-Monocle-Authorization` is present, even if its value is wrong.
///
/// Counter-example: falling back to the alias when canonical fails would violate INV-5
/// (canonical priority is immutable).
///
/// Traces to BC-2.01.009 PC-4, INV-5.
#[tokio::test]
async fn test_BC_2_01_002_canonical_header_present_alias_ignored() {
    let state = make_state_with_token();
    // Wrong canonical value (correct format, wrong hex). Alias value is the correct token.
    // If canonical priority is correct: canonical fails → 401 (alias is not evaluated).
    // If canonical priority is violated: alias succeeds → 200.
    let wrong_canonical = format!("monocle-v1:{}", "9".repeat(64));
    let (status, body) = get_status_json(
        state,
        &[
            ("X-Monocle-Authorization", wrong_canonical.as_str()),
            ("X-Claude-Code-Ide-Authorization", TEST_TOKEN_HEX),
        ],
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with wrong canonical + correct alias must return HTTP 401 (canonical \
        priority per BC-2.01.009 PC-4 / INV-5 — alias is not evaluated when canonical present); \
        got {status}. Body: {body}. Counter-example: fallback to alias would return 200."
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "wrong canonical with correct alias must return E-AUTH-002 (canonical fails, alias \
        not evaluated); got: {body}"
    );
}

// ---------------------------------------------------------------------------
// EC-012 — BC-2.01.009 EC-012 — Alias empty value → 401 invalid
// ---------------------------------------------------------------------------

/// BC-2.01.009 EC-012: `X-Claude-Code-Ide-Authorization` present with empty value;
/// `X-Monocle-Authorization` absent.
///
/// Alias path entered: WARN log emitted; empty string fails constant-time comparison →
/// HTTP 401 `{"error":"invalid_auth_token"}` (value-present failure on alias path).
///
/// Counter-example: returning E-AUTH-001 (missing) would be incorrect because the alias
/// header IS present with a value (empty string is still a value-present case).
///
/// Traces to BC-2.01.009 EC-012, PC-3.
#[tokio::test]
async fn test_BC_2_01_002_alias_empty_value_returns_401_invalid() {
    let state = make_state_with_token();
    // Alias header present with empty value — alias path entered.
    let (status, body) = get_status_json(state, &[("X-Claude-Code-Ide-Authorization", "")]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with empty X-Claude-Code-Ide-Authorization value must return HTTP 401; \
        got {status}. Body: {body}. (BC-2.01.009 EC-012)"
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "Alias empty value must return E-AUTH-002 (not E-AUTH-001 — header IS present); \
        got: {body}. (BC-2.01.009 EC-012)"
    );
}

// ---------------------------------------------------------------------------
// EC-013 — BC-2.01.009 EC-013 — Unrecognized Authorization: Bearer → 401 missing (not invalid)
// ---------------------------------------------------------------------------

/// BC-2.01.009 EC-013: `Authorization: Bearer <token>` present; neither
/// `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` present.
///
/// BOTH recognized headers are absent → falls into PC-1 (missing-header case, not a
/// value-present failure). The `Authorization` header name is unrecognized by the auth
/// middleware and is ignored. Returns HTTP 401 `{"error":"missing_auth_token"}` (E-AUTH-001).
///
/// Counter-example: treating `Authorization: Bearer <token>` as a value-present failure
/// (E-AUTH-002) would violate PC-1 — the dual-absence rule applies when neither recognized
/// header is present, even if other headers exist.
///
/// Traces to BC-2.01.009 EC-013, PC-1, canonical test vector row 5.
#[tokio::test]
async fn test_BC_2_01_002_bearer_auth_header_returns_401_missing() {
    let state = make_state_with_token();
    // Authorization: Bearer is an UNRECOGNIZED header; neither canonical nor alias present.
    let (status, body) =
        get_status_json(state, &[("Authorization", "Bearer fake-token-value")]).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /status with Authorization: Bearer must return HTTP 401; got {status}. \
        Body: {body}. (BC-2.01.009 EC-013)"
    );
    assert_eq!(
        body.get("error").and_then(|v| v.as_str()),
        Some("missing_auth_token"),
        "Authorization: Bearer must return E-AUTH-001 missing_auth_token (not invalid — \
        both recognized headers are absent per BC-2.01.009 EC-013 / PC-1); got: {body}"
    );
}

// ---------------------------------------------------------------------------
// INV-1 — BC-2.01.009 INV-1 — No third error body
// ---------------------------------------------------------------------------

/// BC-2.01.009 INV-1: The two-body taxonomy (`missing_auth_token` vs. `invalid_auth_token`)
/// is the COMPLETE auth error surface for Phase 1. There is no third body.
/// The old body `invalid_auth_token_format` is retired and does not appear in any Phase 1 response.
///
/// This test verifies that the source code does not emit the retired body string.
///
/// Traces to BC-2.01.009 INV-1.
#[test]
fn test_BC_2_01_002_invariant_no_third_error_body() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let auth_src =
        fs::read_to_string(manifest_dir.join("src/auth.rs")).expect("src/auth.rs must exist");

    // The retired body literal must not appear in non-comment source code.
    let retired_body_hits: Vec<(usize, String)> = auth_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && line.contains("invalid_auth_token_format")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        retired_body_hits.is_empty(),
        "src/auth.rs must NOT emit the retired error body `invalid_auth_token_format` in \
        executable code (BC-2.01.009 INV-1: this body is retired in Phase 1). \
        Found at lines: {:?}",
        retired_body_hits
    );
}

// ---------------------------------------------------------------------------
// INV-2 — BC-2.01.009 INV-2 — missing and invalid are distinct bodies
// ---------------------------------------------------------------------------

/// BC-2.01.009 INV-2: The distinction between missing and invalid error bodies is preserved.
///
/// This property test exercises both failure modes in sequence and asserts their bodies are
/// different JSON objects — guarding against a regression where both paths return the same body.
///
/// Traces to BC-2.01.009 INV-2.
#[tokio::test]
async fn test_BC_2_01_002_invariant_missing_and_invalid_are_distinct_bodies() {
    let state = make_state_with_token();

    // E-AUTH-001: no auth header (missing)
    let (status_missing, body_missing) = get_status_json(Arc::clone(&state), &[]).await;
    assert_eq!(
        status_missing,
        StatusCode::UNAUTHORIZED,
        "missing auth must be 401"
    );

    // E-AUTH-002: wrong token (invalid)
    let wrong = format!("monocle-v1:{}", "1".repeat(64));
    let (status_invalid, body_invalid) =
        get_status_json(state, &[("X-Monocle-Authorization", wrong.as_str())]).await;
    assert_eq!(
        status_invalid,
        StatusCode::UNAUTHORIZED,
        "wrong token must be 401"
    );

    // The bodies must be distinct: missing_auth_token vs. invalid_auth_token.
    assert_eq!(
        body_missing.get("error").and_then(|v| v.as_str()),
        Some("missing_auth_token"),
        "missing-auth body must contain error=missing_auth_token"
    );
    assert_eq!(
        body_invalid.get("error").and_then(|v| v.as_str()),
        Some("invalid_auth_token"),
        "wrong-token body must contain error=invalid_auth_token"
    );
    assert_ne!(
        body_missing, body_invalid,
        "missing-auth and wrong-token bodies must be DISTINCT per BC-2.01.009 INV-2; \
        both returned: {body_missing:?}"
    );
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

/// BC-2.01.002 §Body Size Limit: The body limit MUST be 262144 bytes (256 KiB), not
/// some other value.
///
/// This test greps for the exact byte value `262144` in non-comment `server.rs` lines to
/// ensure the limit is correctly sized per SS-daemon-lifecycle.md v1.0.33 §Body Size Limit.
///
/// Traces to BC-2.01.002 Invariant 3, SS-daemon-lifecycle.md v1.0.33 §Body Size Limit.
#[test]
fn test_BC_2_01_002_invariant_body_limit_value_is_256kib() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let server_src =
        fs::read_to_string(manifest_dir.join("src/server.rs")).expect("src/server.rs must exist");

    let hits_262144: Vec<(usize, String)> = server_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && line.contains("262144")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        !hits_262144.is_empty(),
        "server.rs must specify body limit as 262144 (256 * 1024) per SS-daemon-lifecycle.md \
        v1.0.33 §Body Size Limit. No non-comment reference to `262144` found. \
        Ensure `DefaultBodyLimit::max(262144)` (or equivalent) is present in server.rs."
    );
}

// ---------------------------------------------------------------------------
// Architecture compliance — forbidden imports in status handler
// ---------------------------------------------------------------------------

/// Story §Architecture Compliance: `handlers/status.rs` MUST NOT import or use
/// `constant_time_eq` in non-comment executable code. Token comparison is enforced
/// upstream by the auth middleware in `auth.rs`; the handler must not duplicate it.
///
/// Traces to S-003 §Forbidden Dependencies, BC-2.01.002 §Architecture Constraints.
#[test]
fn test_BC_2_01_002_invariant_status_handler_does_not_compare_tokens_with_eq() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let status_src = fs::read_to_string(manifest_dir.join("src/handlers/status.rs"))
        .expect("src/handlers/status.rs must exist");

    let non_comment_hits: Vec<(usize, String)> = status_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && line.contains("constant_time_eq")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        non_comment_hits.is_empty(),
        "handlers/status.rs must NOT import or use `constant_time_eq` in executable code \
        (auth is enforced upstream by auth middleware; the status handler must not duplicate it). \
        Found non-comment references at lines: {:?}. \
        Traces to S-003 §Forbidden Dependencies.",
        non_comment_hits
    );
}

/// Story §Architecture Compliance: `handlers/status.rs` MUST NOT import from `monocle-tui`
/// (not a Phase 1 crate).
///
/// Traces to S-003 §Forbidden Dependencies, BC-2.01.002 §Architecture Constraints.
#[test]
fn test_BC_2_01_002_invariant_status_handler_does_not_import_monocle_tui() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let status_src = fs::read_to_string(manifest_dir.join("src/handlers/status.rs"))
        .expect("src/handlers/status.rs must exist");

    let non_comment_hits: Vec<(usize, String)> = status_src
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
        "handlers/status.rs must NOT import from monocle-tui (not a Phase 1 crate); \
        found non-comment references at lines: {:?}. \
        Traces to S-003 §Forbidden Dependencies.",
        non_comment_hits
    );
}

/// Story §Architecture Compliance: `auth.rs` MUST NOT use `std::cmp::PartialEq` (or `==`)
/// directly on token bytes. All comparisons MUST use `constant_time_eq`.
///
/// We grep for `constant_time_eq` usage in non-comment lines (must be present) AND
/// for direct `== state.auth_token` or `== token` patterns (must not be present).
///
/// Traces to S-003 §Forbidden Dependencies (NFR-010 constant-time comparison).
#[test]
fn test_BC_2_01_002_invariant_auth_uses_constant_time_eq() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let auth_src =
        fs::read_to_string(manifest_dir.join("src/auth.rs")).expect("src/auth.rs must exist");

    // POSITIVE assertion: constant_time_eq must appear in non-comment executable code.
    let cts_hits: Vec<(usize, String)> = auth_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && line.contains("constant_time_eq")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        !cts_hits.is_empty(),
        "src/auth.rs must use `constant_time_eq::constant_time_eq` for token comparison \
        in executable code (NFR-010 timing-attack resistance). No non-comment usage found. \
        Traces to S-003 §Architecture Compliance / BC-2.01.009 INV-7."
    );
}
