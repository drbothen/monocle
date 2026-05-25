//! Integration tests for BC-2.02.001: ABI Version in `/status` response.
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
    let has_in_const_context = main_src
        .lines()
        .any(|line| {
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
