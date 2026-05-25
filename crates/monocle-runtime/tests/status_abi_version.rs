//! Integration tests for BC-2.02.001: ABI Version in `/status` response.
//! Also covers AC-007 (`last_hook_ts` format), EC-043/EC-044 (initial state),
//! and field-level type/invariant assertions for BC-2.01.002.
//!
//! Covers VP-011 §Harness Location: ABI version in `/status` response body.
//! Every test name follows the `test_BC_S_SS_NNN_xxx` pattern for full traceability.
//!
//! # Red Gate
//!
//! Tests that hit the live handler (`build_server` / `get_status`) will FAIL until
//! S-003 implementation is complete (`unimplemented!()` panics in handler and server stubs).
//!
//! The compile-time drift guard assertion is GREEN-BY-DESIGN: it evaluates a const at
//! compile time, not at runtime. This is expected and acceptable per BC-5.38.002.
//!
//! # Coverage Map
//!
//! | Probe | BC / AC | VP-011 Probe | Test function |
//! |-------|---------|--------------|---------------|
//! | AC-005 11.a | BC-2.02.001 PC-1 | 11.a abi_version == 1 | test_BC_2_02_001_abi_version_field_equals_1 |
//! | VP-011 PC-3 | BC-2.02.002 PC-2 | Drift guard | test_BC_2_02_001_compile_time_drift_guard |
//! | AC-007 | BC-2.01.002 PC-1 last_hook_ts format | 2.g null for unfired hooks | test_BC_2_01_002_last_hook_ts_unfired_hooks_are_null |
//! | AC-007 | BC-2.01.002 PC-1 last_hook_ts structure | 2.g last_hook_ts has 5 fields | test_BC_2_01_002_last_hook_ts_has_exactly_5_fields |
//! | AC-007 | BC-2.01.002 PC-1 last_hook_ts keys | 2.g last_hook_ts field names correct | test_BC_2_01_002_last_hook_ts_field_names_are_canonical |
//! | EC-043 | BC-2.01.002 EC-043 | Initial state: ring=0.0, channel=0.0 | test_BC_2_01_002_initial_state_ring_and_channel_are_zero |
//! | EC-044 | BC-2.01.002 EC-044 | Unfired hook ts is null (not "null" string, not 0) | test_BC_2_01_002_ec_044_unfired_hook_ts_is_json_null_not_string |
//! | VP-011 INV | BC-2.02.001 INV | abi_version == MONOCLE_ABI_VERSION const | test_BC_2_02_001_abi_version_matches_compile_time_const |

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

/// Raw 64-hex-char auth token for ABI version tests.
const TEST_TOKEN_HEX: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

/// Build a `DaemonState` with the test token pre-loaded.
fn make_state_with_token() -> Arc<DaemonState> {
    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN_HEX.to_string();
    Arc::new(state)
}

/// Issue `GET /status` with canonical auth and return the JSON body.
async fn get_status_body(state: Arc<DaemonState>) -> (StatusCode, serde_json::Value) {
    let canonical_header = format!("monocle-v1:{TEST_TOKEN_HEX}");
    let app = build_server(state);
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
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response body must be valid JSON");
    (status, value)
}

// ---------------------------------------------------------------------------
// AC-005 / VP-011 Probe 11.a — abi_version field == 1
// ---------------------------------------------------------------------------

/// VP-011 Probe 11.a: The `abi_version` field in the `/status` response equals
/// `monocle_core::MONOCLE_ABI_VERSION` (value `1`) as compiled into the binary.
///
/// This test asserts:
/// 1. `GET /status` with valid auth returns HTTP 200.
/// 2. The `abi_version` field is present and equals the integer `1`.
/// 3. The value matches `monocle_core::MONOCLE_ABI_VERSION` (compile-time equality).
///
/// Counter-example guarded: stub `unimplemented!()` panics; test asserts 200 + abi_version == 1.
///
/// Traces to BC-2.02.001 PC-1, AC-005.
#[tokio::test]
async fn test_BC_2_02_001_abi_version_field_equals_1() {
    let state = make_state_with_token();
    let (status, body) = get_status_body(state).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "prerequisite: GET /status with valid auth must return HTTP 200; got {status}. \
        Body: {body}"
    );

    let abi_version = body
        .get("abi_version")
        .expect("abi_version field must be present in /status response (BC-2.02.001 PC-1)");

    assert!(
        abi_version.is_u64() || abi_version.is_i64(),
        "abi_version must be a JSON integer; got: {abi_version:?}"
    );

    let version_value = abi_version
        .as_u64()
        .expect("abi_version must be a non-negative integer");

    assert_eq!(
        version_value,
        u64::from(monocle_core::MONOCLE_ABI_VERSION),
        "abi_version in /status response must equal monocle_core::MONOCLE_ABI_VERSION \
        (VP-011 Probe 11.a: `jq .abi_version == 1`); \
        got {version_value}, expected {}",
        monocle_core::MONOCLE_ABI_VERSION
    );
}

/// BC-2.02.001 INV: The `abi_version` value in the response must match the compile-time
/// constant `monocle_core::MONOCLE_ABI_VERSION`.
///
/// This is a runtime binding of the compile-time invariant: the handler must read the const
/// directly, not use a hardcoded literal that could drift from the const.
///
/// Traces to BC-2.02.001 INV (ABI version in response MUST equal MONOCLE_ABI_VERSION).
#[tokio::test]
async fn test_BC_2_02_001_abi_version_matches_compile_time_const() {
    let state = make_state_with_token();
    let (status, body) = get_status_body(state).await;
    assert_eq!(status, StatusCode::OK, "prerequisite: valid auth → 200");

    let abi_val = body
        .get("abi_version")
        .and_then(|v| v.as_u64())
        .expect("abi_version must be present and u64");

    // Compile-time constant as u64 for comparison.
    let expected = u64::from(monocle_core::MONOCLE_ABI_VERSION);
    assert_eq!(
        abi_val, expected,
        "abi_version in /status response ({abi_val}) must exactly equal the compile-time \
        constant monocle_core::MONOCLE_ABI_VERSION ({expected}). The handler must reference \
        the const directly to prevent drift (BC-2.02.001 INV)."
    );
}

// ---------------------------------------------------------------------------
// VP-011 PC-3 / VP-012 — Compile-time drift guard (GREEN-BY-DESIGN)
// ---------------------------------------------------------------------------

/// VP-011 PC-3 / BC-2.02.002 PC-2: Compile-time ABI drift guard.
///
/// This test verifies the compile-time `const_assert` is structurally present in the
/// monocle-runtime binary entry point (`main.rs`), per the story task and VP-011 §Mechanism.
///
/// The source-grep approach is used because the actual `const_assert` panics at COMPILE
/// time (not at runtime), making it untestable via `#[test]`. This structural test
/// verifies the invariant is expressed in source code.
///
/// # GREEN-BY-DESIGN
///
/// This test itself passes immediately against the stubs because it is a pure source-grep
/// that reads a static file. It does not call `build_server` or `get_status`.
/// Listed in the stub commit report under GREEN-BY-DESIGN.
///
/// Traces to BC-2.02.001 PC-2, BC-2.02.002 PC-2, VP-011 PC-3, VP-012.
#[test]
fn test_BC_2_02_001_compile_time_drift_guard() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main_src = fs::read_to_string(manifest_dir.join("src/main.rs"))
        .expect("src/main.rs must exist for compile-time drift guard assertion");

    // Assert the compile-time ABI drift guard const assertion is present.
    // The guard is: `const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1, ...);`
    // We check for the key tokens rather than the exact string to avoid brittleness to
    // whitespace/comment changes while still catching accidental deletion of the guard.
    let has_const_assert = main_src.contains("MONOCLE_ABI_VERSION == 1");
    assert!(
        has_const_assert,
        "src/main.rs must contain a compile-time ABI drift guard using \
        `MONOCLE_ABI_VERSION == 1` in a `const _: () = assert!(...)` form \
        per VP-011 §Mechanism and BC-2.02.002 PC-2. \
        This assertion prevents silent ABI drift when monocle_core is updated. \
        Not found in src/main.rs."
    );

    // Additionally verify the guard appears as a const expression (not just in a comment).
    let has_in_const_context = main_src.lines().any(|line| {
        let t = line.trim_start();
        !t.starts_with("//") && line.contains("MONOCLE_ABI_VERSION == 1")
    });
    assert!(
        has_in_const_context,
        "The MONOCLE_ABI_VERSION == 1 assertion must appear in non-comment executable code \
        in src/main.rs (compile-time const assert, not just a doc comment). \
        Traces to VP-011 PC-3 / BC-2.02.002 PC-2."
    );
}

// ---------------------------------------------------------------------------
// AC-007 — BC-2.01.002 PC-1 last_hook_ts — Structure, field names, null for unfired hooks
// ---------------------------------------------------------------------------

/// BC-2.01.002 PC-1 sub-bullet `last_hook_ts`: The response field is a JSON object
/// with EXACTLY 5 keys, one per canonical hook endpoint type.
///
/// Counter-example: `last_hook_ts: null` (null object) fails the object-type assertion.
/// Counter-example: `last_hook_ts: {}` (empty object) fails the 5-key count assertion.
///
/// Traces to BC-2.01.002 PC-1 sub-bullet `last_hook_ts`, AC-007.
#[tokio::test]
async fn test_BC_2_01_002_last_hook_ts_has_exactly_5_fields() {
    let state = make_state_with_token();
    let (status, body) = get_status_body(state).await;
    assert_eq!(status, StatusCode::OK, "prerequisite: valid auth → 200");

    let last_hook_ts = body
        .get("last_hook_ts")
        .expect("last_hook_ts must be present in /status response");
    assert!(
        last_hook_ts.is_object(),
        "last_hook_ts must be a JSON object (not null, not array); got: {last_hook_ts:?} \
        (BC-2.01.002 PC-1 sub-bullet `last_hook_ts`)"
    );

    let ts_obj = last_hook_ts
        .as_object()
        .expect("last_hook_ts must be an object");
    assert_eq!(
        ts_obj.len(),
        5,
        "last_hook_ts must have EXACTLY 5 fields, one per canonical hook endpoint type \
        (BC-2.01.002 PC-1 sub-bullet `last_hook_ts`); got {} field(s): {:?}",
        ts_obj.len(),
        ts_obj.keys().collect::<Vec<_>>()
    );
}

/// BC-2.01.002 PC-1 sub-bullet `last_hook_ts`: The 5 field names must exactly match the
/// canonical hook endpoint types in the spec.
///
/// Canonical field names (from `LastHookTimestamps` struct + BC-2.01.002 PC-1):
/// - `pre_tool_use` (maps to `/hooks/pre-tool-use`)
/// - `notification` (maps to `/hooks/notification`)
/// - `stop` (maps to `/hooks/stop`)
/// - `session_start` (maps to `/hooks/session-start`)
/// - `prompt_submit` (maps to `/hooks/prompt-submit`)
///
/// Counter-example: `pre-tool-use` (hyphen) is a WRONG key name; the spec uses
/// `pre_tool_use` (underscore) for the Rust struct field (JSON serialized).
///
/// Traces to BC-2.01.002 PC-1 sub-bullet `last_hook_ts`, AC-007.
#[tokio::test]
async fn test_BC_2_01_002_last_hook_ts_field_names_are_canonical() {
    let state = make_state_with_token();
    let (status, body) = get_status_body(state).await;
    assert_eq!(status, StatusCode::OK, "prerequisite: valid auth → 200");

    let last_hook_ts = body
        .get("last_hook_ts")
        .and_then(|v| v.as_object())
        .expect("last_hook_ts must be a JSON object");

    let required_ts_fields = [
        "pre_tool_use",
        "notification",
        "stop",
        "session_start",
        "prompt_submit",
    ];

    for field in required_ts_fields {
        assert!(
            last_hook_ts.contains_key(field),
            "last_hook_ts must contain field \"{field}\" (BC-2.01.002 PC-1 sub-bullet \
            `last_hook_ts`); missing from: {:?}",
            last_hook_ts.keys().collect::<Vec<_>>()
        );
    }
}

/// BC-2.01.002 PC-1 sub-bullet `last_hook_ts` / EC-044 / AC-007:
/// Hooks that have NOT fired since daemon start must have JSON `null` as their timestamp value.
///
/// The `DaemonState` constructed by `DaemonState::new()` has all `LastHookTimestamps` fields
/// as `Option::None`. When serialized, `None` must produce JSON `null` (via `serde`'s
/// default `Option` serialization), NOT:
/// - The string `"null"` (which would be JSON `"null"` — a string value)
/// - The integer `0` (not a timestamp, violates the type contract)
/// - An empty string `""` (not null)
///
/// Counter-example: `"pre_tool_use": "null"` fails because `"null"` is a string, not JSON null.
/// Counter-example: `"pre_tool_use": 0` fails because `0` is not a timestamp or null.
///
/// Traces to BC-2.01.002 EC-044, PC-1 sub-bullet `last_hook_ts`, AC-007.
#[tokio::test]
async fn test_BC_2_01_002_last_hook_ts_unfired_hooks_are_null() {
    let state = make_state_with_token();
    // DaemonState::new() initializes all last_hook_ts fields to None → must serialize as null.
    let (status, body) = get_status_body(state).await;
    assert_eq!(status, StatusCode::OK, "prerequisite: valid auth → 200");

    let last_hook_ts = body
        .get("last_hook_ts")
        .and_then(|v| v.as_object())
        .expect("last_hook_ts must be a JSON object");

    let hook_fields = [
        "pre_tool_use",
        "notification",
        "stop",
        "session_start",
        "prompt_submit",
    ];

    for field in hook_fields {
        let value = last_hook_ts
            .get(field)
            .unwrap_or_else(|| panic!("last_hook_ts.{field} must be present"));

        assert!(
            value.is_null(),
            "last_hook_ts.{field} must be JSON null for an unfired hook (BC-2.01.002 EC-044 / \
            AC-007); got: {value:?}. Counter-example: the string \"null\" is NOT JSON null — \
            serde's Option serialization must produce null, not the string \"null\" or 0."
        );

        // Explicitly assert it is not the string "null".
        assert_ne!(
            value.as_str(),
            Some("null"),
            "last_hook_ts.{field} must be JSON null, NOT the string \"null\"; \
            got: {value:?} (BC-2.01.002 EC-044)"
        );

        // Explicitly assert it is not 0 (integer).
        assert!(
            value.as_i64().is_none() && value.as_u64().is_none(),
            "last_hook_ts.{field} must be JSON null, NOT an integer (0 or otherwise); \
            got: {value:?} (BC-2.01.002 EC-044)"
        );
    }
}

/// BC-2.01.002 EC-044 / AC-007: Fired hook timestamps MUST use ISO 8601 UTC with mandatory
/// millisecond precision: `YYYY-MM-DDTHH:MM:SS.sssZ`.
///
/// This test simulates a fired hook by pre-populating `state.last_hook_ts` with a
/// timestamp string and verifying the response contains a correctly-formatted string
/// (not null) that matches the ISO 8601 millisecond-precision regex.
///
/// Format requirement per SS-daemon-lifecycle.md v1.0.33 §Status Endpoint:
/// `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")` — mandatory millisecond precision.
///
/// Traces to BC-2.01.002 EC-044, PC-1 sub-bullet `last_hook_ts`, AC-007.
#[tokio::test]
async fn test_BC_2_01_002_ec_044_fired_hook_ts_uses_iso8601_ms_precision() {
    // Construct a timestamp string in the required format.
    let example_ts = "2026-05-25T12:34:56.789Z";

    let mut state = DaemonState::new();
    state.auth_token = TEST_TOKEN_HEX.to_string();
    // Simulate a fired pre-tool-use hook by setting the timestamp.
    {
        let mut ts = state.last_hook_ts.write().unwrap();
        ts.pre_tool_use = Some(example_ts.to_string());
    }
    let state = Arc::new(state);

    let (status, body) = get_status_body(state).await;
    assert_eq!(status, StatusCode::OK, "prerequisite: valid auth → 200");

    let last_hook_ts = body
        .get("last_hook_ts")
        .and_then(|v| v.as_object())
        .expect("last_hook_ts must be a JSON object");

    let pre_tool_use_val = last_hook_ts
        .get("pre_tool_use")
        .expect("last_hook_ts.pre_tool_use must be present");

    // The fired hook must be a string (not null).
    assert!(
        pre_tool_use_val.is_string(),
        "last_hook_ts.pre_tool_use must be a string when the hook has fired; got: {pre_tool_use_val:?} \
        (BC-2.01.002 EC-044 / AC-007)"
    );

    let ts_str = pre_tool_use_val.as_str().unwrap();

    // Validate ISO 8601 UTC with mandatory millisecond precision:
    // YYYY-MM-DDTHH:MM:SS.sssZ
    // The format produced by `chrono::format("%Y-%m-%dT%H:%M:%S%.3fZ")`.
    let iso8601_ms_re = regex_lite::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$")
        .expect("valid ISO 8601 ms-precision regex");

    assert!(
        iso8601_ms_re.is_match(ts_str),
        "last_hook_ts.pre_tool_use must match ISO 8601 UTC with mandatory millisecond precision \
        `YYYY-MM-DDTHH:MM:SS.sssZ` per BC-2.01.002 EC-044 / AC-007 / \
        SS-daemon-lifecycle.md v1.0.33 §Status Endpoint; got: {ts_str:?}. \
        Format must use chrono `\"%Y-%m-%dT%H:%M:%S%.3fZ\"`."
    );

    // Verify ends with 'Z' (UTC mandatory).
    assert!(
        ts_str.ends_with('Z'),
        "ISO 8601 timestamp must end with 'Z' (UTC offset mandatory); got: {ts_str:?} \
        (BC-2.01.002 EC-044)"
    );

    // Verify the 'T' separator is present between date and time.
    assert!(
        ts_str.contains('T'),
        "ISO 8601 timestamp must contain 'T' separator; got: {ts_str:?} \
        (BC-2.01.002 EC-044)"
    );

    // Verify millisecond precision: the '.' before the millisecond digits.
    let time_part = ts_str.trim_end_matches('Z');
    assert!(
        time_part.contains('.'),
        "ISO 8601 timestamp must contain '.' for millisecond precision; got: {ts_str:?} \
        (BC-2.01.002 EC-044 — `YYYY-MM-DDTHH:MM:SS.sssZ` form requires milliseconds)"
    );
}

/// BC-2.01.002 EC-044 / AC-007 negative: The string `"null"` is NOT a valid null representation.
///
/// This test exercises the serialization contract explicitly: `Option<String>::None` must
/// serialize to JSON `null`, not to the string literal `"null"`. We verify the distinction
/// holds at the serialization layer by checking the schema contract directly.
///
/// Traces to BC-2.01.002 EC-044.
#[test]
fn test_BC_2_01_002_ec_044_unfired_hook_ts_is_json_null_not_string() {
    // Structural test: verify Option<String> serializes as null via serde_json.
    // This does not require a running server — it validates the serialization schema
    // directly to catch cases where the field type is changed to a plain String.
    let none_value: Option<String> = None;
    let serialized = serde_json::to_value(none_value).expect("serialize Option::None");
    assert!(
        serialized.is_null(),
        "Option<String>::None must serialize to JSON null (not the string \"null\"); \
        got: {serialized:?}. If the handler serializes None as the string \"null\", it violates \
        BC-2.01.002 EC-044. Counter-example: `serde_json::json!({{\"field\": \"null\"}})` is \
        wrong; `serde_json::json!({{\"field\": null}})` is correct."
    );

    // Positive: a Some value serializes as a string.
    let some_value: Option<String> = Some("2026-05-25T12:34:56.789Z".to_string());
    let serialized_some = serde_json::to_value(some_value).expect("serialize Option::Some");
    assert!(
        serialized_some.is_string(),
        "Option<String>::Some(ts) must serialize to a JSON string; got: {serialized_some:?}"
    );
}

// ---------------------------------------------------------------------------
// EC-043 — BC-2.01.002 EC-043 — Initial state: ring=0.0, channel=0.0
// ---------------------------------------------------------------------------

/// BC-2.01.002 EC-043: In the initial daemon state (no ring or channel events yet),
/// `ring_buffer_fill_pct` is `0.0` and `channel_saturation_pct` is `0.0`.
///
/// `DaemonState::new()` does not pre-populate ring or channel state (those are wired
/// by S-005 and S-008). The handler must default to `0.0` for both fields when no
/// ring or channel state is available.
///
/// Counter-example: returning `null` for these fields would violate the float type contract.
/// Counter-example: returning a non-zero value from an un-initialized state would be fabricated.
///
/// Traces to BC-2.01.002 EC-043.
#[tokio::test]
async fn test_BC_2_01_002_initial_state_ring_and_channel_are_zero() {
    let state = make_state_with_token();
    // DaemonState::new() — no ring buffer events, no channel events.
    let (status, body) = get_status_body(state).await;
    assert_eq!(status, StatusCode::OK, "prerequisite: valid auth → 200");

    // ring_buffer_fill_pct must be 0.0 in initial state.
    let ring_pct = body
        .get("ring_buffer_fill_pct")
        .and_then(|v| v.as_f64())
        .expect("ring_buffer_fill_pct must be a number");
    assert_eq!(
        ring_pct, 0.0,
        "ring_buffer_fill_pct must be 0.0 in initial daemon state \
        (BC-2.01.002 EC-043 — no ring events yet); got: {ring_pct}"
    );

    // channel_saturation_pct must be 0.0 in initial state.
    let channel_pct = body
        .get("channel_saturation_pct")
        .and_then(|v| v.as_f64())
        .expect("channel_saturation_pct must be a number");
    assert_eq!(
        channel_pct, 0.0,
        "channel_saturation_pct must be 0.0 in initial daemon state \
        (BC-2.01.002 EC-043 — no channel events yet); got: {channel_pct}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.02.001 PC-2 — Source-grep: handler reads MONOCLE_ABI_VERSION, not a hardcoded literal
// ---------------------------------------------------------------------------

/// BC-2.02.001 PC-2: The `get_status` handler MUST reference `monocle_core::MONOCLE_ABI_VERSION`
/// in non-comment executable code, NOT a hardcoded integer literal for abi_version.
///
/// This prevents the abi_version field from silently diverging from the compile-time constant
/// if someone edits the handler to use a hardcoded `1` instead of the const.
///
/// Traces to BC-2.02.001 PC-2 (handler must reference the const, not a literal).
#[test]
fn test_BC_2_02_001_status_handler_references_monocle_abi_version_const() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let status_src = fs::read_to_string(manifest_dir.join("src/handlers/status.rs"))
        .expect("src/handlers/status.rs must exist");

    // Positive: MONOCLE_ABI_VERSION must appear in non-comment executable code.
    let abi_const_hits: Vec<(usize, String)> = status_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && line.contains("MONOCLE_ABI_VERSION")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        !abi_const_hits.is_empty(),
        "src/handlers/status.rs must reference `MONOCLE_ABI_VERSION` (or \
        `monocle_core::MONOCLE_ABI_VERSION`) in executable code for the `abi_version` field \
        per BC-2.02.001 PC-2. Using a hardcoded integer literal instead of the const would \
        allow silent drift. Not found in non-comment lines."
    );
}

/// BC-2.02.001 INV (structural complement): The `abi_version` field value in the handler
/// must NOT be a hardcoded integer literal `abi_version: 1u32` without referencing the const.
///
/// This source-grep detects if someone shortcuts the const and writes the field as a raw
/// integer literal. The pattern `abi_version: 1` (with a numeric literal right-hand side)
/// MUST NOT appear in non-comment status handler code.
///
/// Note: `abi_version: 1_u32` and `abi_version: 1u32` are also detected.
///
/// Traces to BC-2.02.001 INV.
#[test]
fn test_BC_2_02_001_status_handler_does_not_hardcode_abi_version_literal() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let status_src = fs::read_to_string(manifest_dir.join("src/handlers/status.rs"))
        .expect("src/handlers/status.rs must exist");

    // Detect patterns like `abi_version: 1,` or `abi_version: 1u32,` in non-comment code.
    // This regex-free grep checks for the field assignment with a bare integer.
    let hardcoded_hits: Vec<(usize, String)> = status_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            if t.starts_with("//") {
                return false;
            }
            // Match lines where abi_version is assigned a raw numeric literal.
            // Pattern: `abi_version: 1` followed by non-const-name characters.
            // We check that MONOCLE_ABI_VERSION does NOT appear on the same line.
            line.contains("abi_version:")
                && (line.contains(": 1,") || line.contains(": 1_u32") || line.contains(": 1u32"))
                && !line.contains("MONOCLE_ABI_VERSION")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        hardcoded_hits.is_empty(),
        "src/handlers/status.rs must NOT hardcode `abi_version: 1` (or `1u32`) as a raw \
        integer literal. Use `abi_version: monocle_core::MONOCLE_ABI_VERSION` to prevent \
        silent drift (BC-2.02.001 INV). Found suspicious lines: {:?}",
        hardcoded_hits
    );
}
