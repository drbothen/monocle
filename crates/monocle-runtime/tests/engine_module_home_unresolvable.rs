//! Integration tests for `HomeUnresolvable` error path — VP-021 (S-015).
//!
//! Covers AC-005 and AC-006:
//! - AC-005 / BC-2.03.003 PC-1: `metadata()` returns `Err(HomeUnresolvable)` when all
//!   four home-env vars are unset (HOME, USERPROFILE, HOMEDRIVE, HOMEPATH).
//! - AC-005 / BC-2.03.003 PC-1: `enrich()` returns `Err(HomeUnresolvable)` under the
//!   same conditions.
//! - AC-006 / BC-2.03.003 PC-2: `metadata()` and `enrich()` emit the `E-ENG-001` log
//!   line when returning `HomeUnresolvable`.
//!
//! Uses `temp-env 0.3` `async_with_vars` for hermetic env isolation.
//! The four vars unset are: HOME, USERPROFILE, HOMEDRIVE, HOMEPATH.
//! XDG_DATA_HOME / XDG_CONFIG_HOME are NOT consulted by `BaseDirs::home_dir()`;
//! clearing them is not required (BC-2.03.003 Invariant 3).
//!
//! # Log Capture (AC-006 / tracing-test)
//!
//! AC-006 assertions use `tracing-test 0.2` with `#[traced_test]`.  Integration tests
//! in tests/ are compiled as a separate crate from the implementation, so the
//! `no-env-filter` feature is required to capture logs emitted by `monocle-runtime`.
//! Without it the subscriber only captures the test crate's own spans — not the
//! `tracing::error!` calls inside `ClaudeCodeModule::metadata()` and `::enrich()`.
//!
//! # Red Gate
//!
//! Both tests in this file MUST FAIL until `metadata()` and `enrich()` are implemented
//! with `directories::BaseDirs::new()` — currently `todo!()` stubs.
//!
//! # Coverage Map
//!
//! | BC Clause | AC | Test function |
//! |-----------|----|---------------|
//! | BC-2.03.003 PC-1 | AC-005 | test_BC_2_03_003_metadata_home_unresolvable |
//! | BC-2.03.003 PC-1 | AC-005 | test_BC_2_03_003_enrich_home_unresolvable |
//! | BC-2.03.003 PC-2 | AC-006 | test_BC_2_03_003_metadata_home_unresolvable (logs_contain) |
//! | BC-2.03.003 PC-2 | AC-006 | test_BC_2_03_003_enrich_home_unresolvable (logs_contain) |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per the naming convention.
#![allow(non_snake_case)]

use std::path::PathBuf;

use monocle_core::engine::{EngineMetadataError, EngineModule, ProcessSnapshot};
use monocle_runtime::engine::ClaudeCodeModule;
use tracing_test::traced_test;

// ---------------------------------------------------------------------------
// Helper: the four home env vars that BaseDirs::new() consults on all platforms.
// Unset all four to guarantee BaseDirs::new() returns None.
// ---------------------------------------------------------------------------
const HOME_ENV_VARS_UNSET: [(&str, Option<&str>); 4] = [
    ("HOME", None),
    ("USERPROFILE", None),
    ("HOMEDRIVE", None),
    ("HOMEPATH", None),
];

/// Construct a minimal ProcessSnapshot for enrich() tests.
fn minimal_proc() -> ProcessSnapshot {
    ProcessSnapshot::new(
        12345,
        Some(PathBuf::from("/usr/local/bin/claude")),
        vec!["claude".to_string()],
        0,
    )
}

/// Construct a `ClaudeCodeModule` with a dummy base URL.
fn module() -> ClaudeCodeModule {
    ClaudeCodeModule::new("http://127.0.0.1:7891".to_string())
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.03.003 PC-1: metadata() returns Err(HomeUnresolvable)
// ---------------------------------------------------------------------------

/// AC-005 / BC-2.03.003 PC-1 + AC-006 / BC-2.03.003 PC-2: When HOME, USERPROFILE,
/// HOMEDRIVE, and HOMEPATH are all unset, `metadata()` must return
/// `Err(EngineMetadataError::HomeUnresolvable)` with no default path substitution,
/// AND must emit the `E-ENG-001` log line at error level.
///
/// Uses `temp_env::async_with_vars` for hermetic env isolation — restores original values
/// after the closure, even on panic.
///
/// AC-006 log assertion uses `tracing-test` `logs_contain()` which is injected by
/// `#[traced_test]`. The `no-env-filter` feature is required because integration tests
/// in tests/ are a separate crate from the implementation; without it only the test
/// crate's own logs are captured, not the `tracing::error!` calls in ClaudeCodeModule.
#[tokio::test]
#[traced_test]
async fn test_BC_2_03_003_metadata_home_unresolvable() {
    temp_env::async_with_vars(HOME_ENV_VARS_UNSET, async {
        let m = module();
        let result = m.metadata();
        assert!(
            result.is_err(),
            "metadata() must return Err when home directory is unresolvable; got Ok"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, EngineMetadataError::HomeUnresolvable),
            "metadata() must return Err(HomeUnresolvable) per BC-2.03.003 PC-1; got {:?}",
            err
        );
    })
    .await;

    // AC-006 / BC-2.03.003 PC-2: the E-ENG-001 log must have been emitted.
    // This asserts on the exact message string emitted by claude_code.rs.
    assert!(
        logs_contain("E-ENG-001: platform home directory unresolvable"),
        "metadata() must emit E-ENG-001 log when returning HomeUnresolvable per BC-2.03.003 PC-2"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.03.003 PC-1: enrich() returns Err(HomeUnresolvable)
// ---------------------------------------------------------------------------

/// AC-005 / BC-2.03.003 PC-1 + AC-006 / BC-2.03.003 PC-2: When HOME, USERPROFILE,
/// HOMEDRIVE, and HOMEPATH are all unset, `enrich()` must return
/// `Err(EngineMetadataError::HomeUnresolvable)` with no default path substitution,
/// AND must emit the `E-ENG-001` log line at error level.
///
/// Uses `temp_env::async_with_vars` for hermetic env isolation — restores original values
/// after the closure, even on panic.
///
/// AC-006 log assertion uses `tracing-test` `logs_contain()` which is injected by
/// `#[traced_test]`. The `no-env-filter` feature is required because integration tests
/// in tests/ are a separate crate from the implementation; without it only the test
/// crate's own logs are captured, not the `tracing::error!` calls in ClaudeCodeModule.
#[tokio::test]
#[traced_test]
async fn test_BC_2_03_003_enrich_home_unresolvable() {
    temp_env::async_with_vars(HOME_ENV_VARS_UNSET, async {
        let m = module();
        let proc = minimal_proc();
        let result = m.enrich(&proc).await;
        assert!(
            result.is_err(),
            "enrich() must return Err when home directory is unresolvable; got Ok"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, EngineMetadataError::HomeUnresolvable),
            "enrich() must return Err(HomeUnresolvable) per BC-2.03.003 PC-1; got {:?}",
            err
        );
    })
    .await;

    // AC-006 / BC-2.03.003 PC-2: the E-ENG-001 log must have been emitted.
    // This asserts on the exact message string emitted by claude_code.rs.
    assert!(
        logs_contain("E-ENG-001: platform home directory unresolvable"),
        "enrich() must emit E-ENG-001 log when returning HomeUnresolvable per BC-2.03.003 PC-2"
    );
}
