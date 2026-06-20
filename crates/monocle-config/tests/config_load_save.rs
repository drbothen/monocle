//! Integration tests for config load/save (atomic write).
//!
//! Covers:
//! - BC-2.07.001: Atomic write via `tempfile::persist` — `write_config()`.
//! - BC-2.07.003: Load resilience — `load_config()` with missing/corrupted/valid files.
//! - Story AC-003..AC-006, AC-010, AC-011.
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause |
//! |---------------|----|-----------|
//! | test_BC_2_07_003_missing_config_returns_default | AC-003 | BC-2.07.003 PC-1, PC-2 |
//! | test_BC_2_07_003_corrupted_config_returns_default | AC-003 | BC-2.07.003 PC-7, PC-9 |
//! | test_BC_2_07_003_zero_byte_config_returns_default | — | BC-2.07.003 EC-089 |
//! | test_BC_2_07_003_valid_json_not_object_returns_default | — | BC-2.07.003 EC-090 |
//! | test_BC_2_07_003_missing_schema_version_returns_default | — | BC-2.07.003 EC-091, BC-2.07.002 EC-084 |
//! | test_BC_2_07_003_valid_config_round_trips | AC-003 | BC-2.07.003 PC-11, PC-4 |
//! | test_BC_2_07_001_write_config_creates_file | AC-005 | BC-2.07.001 PC-3..PC-5 |
//! | test_BC_2_07_001_write_config_creates_parent_dir | AC-006 | BC-2.07.001 PC-1, EC-070 |
//! | test_BC_2_07_001_write_config_round_trip | AC-005 | BC-2.07.001 canonical test vector |
//! | test_BC_2_07_001_write_config_no_partial_write | AC-011 | BC-2.07.001 INV-1, INV-2 |
//! | test_BC_2_07_001_write_config_json_is_valid | AC-005 | BC-2.07.001 PC-3 |
//! | test_BC_2_07_002_schema_version_mismatch_error | AC-004 | story AC-004 |
//! | test_BC_2_07_003_invariant_no_panic_on_any_input | — | BC-2.07.003 INV-1, VP |
//! | test_BC_2_07_003_corrupted_config_emits_warn | — | BC-2.07.003 PC-9 (log assertion) |

// Tests use expect/unwrap as assertion amplification — not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// BC-ID-encoded test names.
#![allow(non_snake_case)]
// catch_unwind + todo!() panics in stubs.
#![allow(clippy::panic)]

use monocle_config::{load_config, write_config, ConfigError, HarnessProfile, MonocleConfig};
use std::io::Write as _;
use std::panic::AssertUnwindSafe;
use tempfile::tempdir;
use tracing_test::traced_test;

// ---------------------------------------------------------------------------
// Helper: write test fixture atomically (per project semgrep rule — no naked fs::write)
// ---------------------------------------------------------------------------

/// Write test fixture content atomically via NamedTempFile + persist.
///
/// The semgrep rule `monocle-no-direct-config-write` forbids `std::fs::write`
/// in the monocle-config crate. We replicate the atomic pattern in tests too
/// so that the test setup is consistent with production behavior.
fn write_fixture(dir: &std::path::Path, filename: &str, content: &str) {
    let mut tmp = tempfile::NamedTempFile::new_in(dir).expect("create NamedTempFile in test dir");
    tmp.write_all(content.as_bytes())
        .expect("write test fixture");
    let dest = dir.join(filename);
    tmp.persist(&dest).expect("persist test fixture atomically");
}

// ---------------------------------------------------------------------------
// BC-2.07.003 — load_config() resilience
// ---------------------------------------------------------------------------

/// BC-2.07.003 PC-1/PC-2 + EC-088: When the config file is absent (first run),
/// `load_config()` must return `Ok(MonocleConfig::default())` without error or
/// logging.
///
/// This is the nominal first-run path: no file → default config → no panic.
#[test]
fn test_BC_2_07_003_missing_config_returns_default() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    // Verify the file really doesn't exist.
    assert!(
        !config_path.exists(),
        "config.json must not exist for this test"
    );

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&config_path)));

    let result = result.unwrap_or_else(|_| {
        panic!(
            "load_config() panicked (likely todo!()) on missing file — \
            Red Gate: implementation not yet written (BC-2.07.003 PC-1)"
        )
    });

    assert!(
        result.is_ok(),
        "load_config() must return Ok(default) when config file is absent \
        (BC-2.07.003 PC-1/PC-2 / EC-088 / story AC-003); got Err: {:?}",
        result.err()
    );

    let config = result.expect("must be Ok");
    assert_eq!(
        config,
        MonocleConfig::default(),
        "load_config() must return MonocleConfig::default() when config file is absent \
        (BC-2.07.003 PC-2 / story AC-003)"
    );
}

/// BC-2.07.003 PC-7/PC-9 + EC-089: When config file contains corrupted (truncated) JSON,
/// `load_config()` must emit `tracing::warn!` and return `Ok(MonocleConfig::default())`.
///
/// The parse error is swallowed — NOT propagated to the caller.
/// This test verifies only the return value; log capture is done in a separate test.
#[test]
fn test_BC_2_07_003_corrupted_config_returns_default() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    // Write deliberately corrupted JSON (truncated).
    write_fixture(tmp.path(), "config.json", r#"{"not_valid_json": "#);

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "load_config() panicked (likely todo!()) on corrupted file — \
            Red Gate: implementation not yet written (BC-2.07.003 PC-7)"
        )
    });

    assert!(
        result.is_ok(),
        "load_config() must return Ok(default) when config file is corrupted JSON \
        (BC-2.07.003 PC-7/PC-9 / story AC-003); got Err: {:?}",
        result.err()
    );

    let config = result.expect("must be Ok");
    assert_eq!(
        config,
        MonocleConfig::default(),
        "load_config() must return MonocleConfig::default() on corrupted JSON \
        (BC-2.07.003 PC-9)"
    );
}

/// BC-2.07.003 EC-089: Zero-byte config file must be treated as corrupted (parse failure).
/// Returns `Ok(MonocleConfig::default())` with a `tracing::warn!` emitted.
#[test]
fn test_BC_2_07_003_zero_byte_config_returns_default() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    // Write zero bytes — empty file produces an EOF parse error.
    write_fixture(tmp.path(), "config.json", "");

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "load_config() panicked (likely todo!()) on zero-byte file — \
            Red Gate: implementation not yet written (BC-2.07.003 EC-089)"
        )
    });

    assert!(
        result.is_ok(),
        "load_config() must return Ok(default) for zero-byte config file \
        (BC-2.07.003 EC-089); got Err: {:?}",
        result.err()
    );

    let config = result.expect("must be Ok");
    assert_eq!(
        config,
        MonocleConfig::default(),
        "zero-byte config must return MonocleConfig::default() (BC-2.07.003 EC-089)"
    );
}

/// BC-2.07.003 EC-090: A valid JSON value that is NOT a JSON object (e.g., `[]` or
/// `"string"`) must be treated as a parse failure → `Ok(MonocleConfig::default())`.
#[test]
fn test_BC_2_07_003_valid_json_not_object_returns_default() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    // Write a JSON array — valid JSON but not a MonocleConfig object.
    write_fixture(tmp.path(), "config.json", "[1, 2, 3]");

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "load_config() panicked (likely todo!()) on non-object JSON — \
            Red Gate: implementation not yet written (BC-2.07.003 EC-090)"
        )
    });

    assert!(
        result.is_ok(),
        "load_config() must return Ok(default) for valid-JSON-but-not-object config \
        (BC-2.07.003 EC-090); got Err: {:?}",
        result.err()
    );
}

/// BC-2.07.003 EC-091 / BC-2.07.002 EC-084: A JSON file with a valid object but no
/// `schema_version` field must fail parse (since `schema_version: u32` has no default)
/// and be treated as Case 3 → `Ok(MonocleConfig::default())`.
#[test]
fn test_BC_2_07_003_missing_schema_version_returns_default() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    // Empty JSON object — missing schema_version (required field).
    write_fixture(tmp.path(), "config.json", "{}");

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "load_config() panicked (likely todo!()) on missing schema_version — \
            Red Gate: implementation not yet written (BC-2.07.003 EC-091)"
        )
    });

    assert!(
        result.is_ok(),
        "load_config() must return Ok(default) when schema_version is missing from JSON \
        (BC-2.07.003 EC-091 / BC-2.07.002 EC-084); got Err: {:?}",
        result.err()
    );
}

/// BC-2.07.003 PC-11 / PC-4 (Case 4): A well-formed config file must deserialize
/// and return `Ok(config)` with all fields populated.
#[test]
fn test_BC_2_07_003_valid_config_round_trips() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    // Write a valid, complete config JSON.
    let original = MonocleConfig {
        schema_version: MonocleConfig::CURRENT_SCHEMA_VERSION,
        harness_profiles: vec![HarnessProfile {
            id: "cc".to_string(),
            display_name: "Claude Code".to_string(),
            binary_path: "/usr/local/bin/claude".to_string(),
            config_dir: None,
        }],
        ccr_path: Some("/usr/local/bin/ccr".to_string()),
        binding_overrides: serde_json::Value::Object(serde_json::Map::new()),
        project_profiles: std::collections::HashMap::new(),
        pty_scrollback_rows: None,
    };
    let json = serde_json::to_string_pretty(&original).expect("original config must serialize");
    write_fixture(tmp.path(), "config.json", &json);

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "load_config() panicked (likely todo!()) on valid config — \
            Red Gate: implementation not yet written (BC-2.07.003 PC-11)"
        )
    });

    assert!(
        result.is_ok(),
        "load_config() must return Ok(config) for a valid config file \
        (BC-2.07.003 PC-11 / Case 4); got Err: {:?}",
        result.err()
    );

    let loaded = result.expect("must be Ok");
    assert_eq!(
        loaded, original,
        "load_config() must return the exact deserialized config when the file is valid \
        (BC-2.07.003 PC-11 round-trip)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.001 — write_config() atomic write
// ---------------------------------------------------------------------------

/// BC-2.07.001 PC-3..PC-5: `write_config()` must create a valid config file at
/// the target path with correct JSON content.
#[test]
fn test_BC_2_07_001_write_config_creates_file() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    let config = MonocleConfig::default();

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| write_config(&config, &config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "write_config() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.001 PC-3)"
        )
    });

    assert!(
        result.is_ok(),
        "write_config() must succeed on a writable temp dir \
        (BC-2.07.001 PC-3..PC-5 / story AC-005); got Err: {:?}",
        result.err()
    );

    assert!(
        config_path.exists(),
        "config file must exist after write_config() (BC-2.07.001 PC-5)"
    );
}

/// BC-2.07.001 PC-1 / EC-070: `write_config()` must create the parent directory
/// (and all missing ancestors) if it does not exist.
///
/// Per story AC-006: `create_dir_all(parent)` is called before writing the tempfile.
#[test]
fn test_BC_2_07_001_write_config_creates_parent_dir() {
    let tmp = tempdir().expect("create temp dir");
    // Two-level nested path that does not exist yet.
    let nested_dir = tmp.path().join("level1").join("level2");
    let config_path = nested_dir.join("config.json");

    assert!(
        !nested_dir.exists(),
        "nested dir must not exist before test"
    );

    let config = MonocleConfig::default();

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| write_config(&config, &config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "write_config() panicked (likely todo!()) on missing parent dir — \
            Red Gate: implementation not yet written (BC-2.07.001 PC-1 / EC-070)"
        )
    });

    assert!(
        result.is_ok(),
        "write_config() must create parent directory when absent \
        (BC-2.07.001 PC-1 / EC-070 / story AC-006); got Err: {:?}",
        result.err()
    );

    assert!(
        nested_dir.exists(),
        "parent directory must exist after write_config() creates it \
        (BC-2.07.001 PC-1 / EC-070 / story AC-006)"
    );

    assert!(
        config_path.exists(),
        "config file must exist in the newly created directory \
        (BC-2.07.001 EC-070)"
    );
}

/// BC-2.07.001 canonical test vector: `write_config(&default_config)` followed by
/// `load_config()` must produce the original config (round-trip invariant).
#[test]
fn test_BC_2_07_001_write_config_round_trip() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");
    let original = MonocleConfig::default();

    // Step 1: write.
    let write_result =
        std::panic::catch_unwind(AssertUnwindSafe(|| write_config(&original, &config_path)));
    let write_result = write_result.unwrap_or_else(|_| {
        panic!(
            "write_config() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.001 canonical test vector)"
        )
    });
    assert!(
        write_result.is_ok(),
        "write_config() must succeed for round-trip test (BC-2.07.001); \
        got Err: {:?}",
        write_result.err()
    );

    // Step 2: load back.
    let load_result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&config_path)));
    let load_result = load_result.unwrap_or_else(|_| {
        panic!(
            "load_config() panicked (likely todo!()) after write — \
            Red Gate: implementation not yet written (BC-2.07.001 canonical test vector)"
        )
    });
    assert!(
        load_result.is_ok(),
        "load_config() must succeed after write_config() (BC-2.07.001 round-trip); \
        got Err: {:?}",
        load_result.err()
    );

    let loaded = load_result.expect("must be Ok");
    assert_eq!(
        loaded, original,
        "config loaded after write must equal original \
        (BC-2.07.001 canonical test vector: round-trip)"
    );
}

/// BC-2.07.001 PC-3: The written file must be valid JSON parseable by `serde_json`.
///
/// Also verifies: `schema_version: 1` appears in the written content.
#[test]
fn test_BC_2_07_001_write_config_json_is_valid() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");
    let config = MonocleConfig::default();

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| write_config(&config, &config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "write_config() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.001 PC-3)"
        )
    });
    result.expect("write_config must succeed (BC-2.07.001 PC-3)");

    // Read raw content and parse as JSON.
    let raw =
        std::fs::read_to_string(&config_path).expect("config file must be readable after write");
    let value: serde_json::Value =
        serde_json::from_str(&raw).expect("written config must be valid JSON (BC-2.07.001 PC-3)");

    assert_eq!(
        value.get("schema_version").and_then(|v| v.as_u64()),
        Some(1u64),
        "written config JSON must have schema_version: 1 \
        (BC-2.07.001 PC-3 / BC-2.07.002 INV-1)"
    );
}

/// BC-2.07.001 INV-1/INV-2 — no partial write observable.
///
/// Verifies that if write_config succeeds, the file at the target path is valid
/// and complete — never zero-byte or partially written. This is the core atomic
/// write invariant from BC-2.07.001.
///
/// Note: We cannot trivially test the "mid-write crash leaves old content intact"
/// invariant in unit tests without a fault injector. This test verifies the
/// observable postcondition: on success, the config file contains complete,
/// parseable JSON.
#[test]
fn test_BC_2_07_001_write_config_no_partial_write() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");
    let config = MonocleConfig {
        schema_version: MonocleConfig::CURRENT_SCHEMA_VERSION,
        harness_profiles: vec![
            HarnessProfile {
                id: "profile-a".to_string(),
                display_name: "Profile A".to_string(),
                binary_path: "/usr/bin/claude".to_string(),
                config_dir: None,
            },
            HarnessProfile {
                id: "profile-b".to_string(),
                display_name: "Profile B".to_string(),
                binary_path: "/usr/local/bin/claude".to_string(),
                config_dir: Some("/tmp/cfg".to_string()),
            },
        ],
        ccr_path: None,
        binding_overrides: serde_json::json!({"ctrl_p": "profile-picker"}),
        project_profiles: [("/home/user/proj".to_string(), "profile-a".to_string())]
            .into_iter()
            .collect(),
        pty_scrollback_rows: None,
    };

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| write_config(&config, &config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "write_config() panicked (likely todo!()) — \
            Red Gate: implementation not yet written (BC-2.07.001 INV-1)"
        )
    });
    result.expect("write_config must succeed (BC-2.07.001 INV-1)");

    // File must be non-empty and parseable.
    let metadata = std::fs::metadata(&config_path).expect("config file must exist");
    assert!(
        metadata.len() > 0,
        "config file must be non-empty after write_config() \
        (BC-2.07.001 INV-1 — no partial write)"
    );

    let raw = std::fs::read_to_string(&config_path).expect("must be readable");
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .expect("config file must be valid JSON after write (BC-2.07.001 INV-1)");
    assert!(
        parsed.is_object(),
        "config file must be a JSON object after write (BC-2.07.001 INV-1)"
    );
}

// ---------------------------------------------------------------------------
// Story AC-004 — schema_version mismatch error
// ---------------------------------------------------------------------------

/// Story AC-004 (traced to BC-2.07.002 INV-1): When the on-disk `schema_version`
/// does not match `MonocleConfig::CURRENT_SCHEMA_VERSION`, `load_config()` returns
/// `Err(ConfigError::SchemaMismatch { found, expected })`.
///
/// No silent migration in Phase 3. The caller receives the error and must handle it.
///
/// Note: BC-2.07.003 specifies parse failures return Ok(default). Schema version
/// mismatch is a distinct, higher-level check that happens AFTER successful parse
/// (the JSON is valid but the version is wrong). The story AC-004 specifies this
/// is an Err, not a swallowed parse error.
#[test]
fn test_BC_2_07_002_schema_version_mismatch_error() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    // Write a config with a future schema_version.
    let future_json = serde_json::json!({
        "schema_version": 99,
        "harness_profiles": [],
        "binding_overrides": {}
    });
    write_fixture(
        tmp.path(),
        "config.json",
        &serde_json::to_string(&future_json).unwrap(),
    );

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&config_path)));
    let result = result.unwrap_or_else(|_| {
        panic!(
            "load_config() panicked (likely todo!()) on schema version mismatch — \
            Red Gate: implementation not yet written (story AC-004)"
        )
    });

    assert!(
        result.is_err(),
        "load_config() must return Err(ConfigError::SchemaMismatch) when schema_version \
        does not match CURRENT_SCHEMA_VERSION (story AC-004). Got Ok."
    );

    match result.expect_err("must be Err") {
        ConfigError::SchemaMismatch { found, expected } => {
            assert_eq!(
                found, 99,
                "SchemaMismatch.found must be the on-disk version (99) per story AC-004"
            );
            assert_eq!(
                expected,
                MonocleConfig::CURRENT_SCHEMA_VERSION,
                "SchemaMismatch.expected must be CURRENT_SCHEMA_VERSION per story AC-004"
            );
        }
        other => panic!(
            "load_config() must return ConfigError::SchemaMismatch for future schema version, \
            got: {other:?} (story AC-004)"
        ),
    }
}

// ---------------------------------------------------------------------------
// BC-2.07.003 INV-1 — load_config() never panics
// ---------------------------------------------------------------------------

/// BC-2.07.003 INV-1: `load_config()` must not panic under any combination of
/// filesystem conditions. We test several inputs in one table-driven test to
/// verify the no-panic contract across all cases.
#[test]
fn test_BC_2_07_003_invariant_no_panic_on_any_input() {
    let test_cases: &[(&str, &str)] = &[
        ("absent", ""), // We'll handle this specially below.
        ("empty", ""),
        ("truncated", r#"{"not_valid_json": "#),
        ("json_array", "[1, 2, 3]"),
        ("json_string", r#""hello""#),
        ("json_null", "null"),
        ("empty_object", "{}"),
        (
            "valid_future_version",
            r#"{"schema_version":99,"harness_profiles":[]}"#,
        ),
    ];

    let tmp = tempdir().expect("create temp dir");

    // Test absent file.
    let absent_path = tmp.path().join("nonexistent.json");
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&absent_path)));
    assert!(
        result.is_ok(),
        "load_config() must not panic on absent file (BC-2.07.003 INV-1)"
    );

    // Test each fixture.
    for (name, content) in test_cases {
        if *name == "absent" {
            continue; // handled above
        }
        let fixture_path = tmp.path().join(format!("{name}.json"));
        write_fixture(tmp.path(), &format!("{name}.json"), content);
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| load_config(&fixture_path)));
        assert!(
            result.is_ok(),
            "load_config() must not panic for test case '{name}' \
            (BC-2.07.003 INV-1 no-panic contract)"
        );
        // The inner result may be Ok or Err but must not panic.
    }
}

// ---------------------------------------------------------------------------
// BC-2.07.003 PC-9 — tracing::warn! emitted on parse failure
// ---------------------------------------------------------------------------

/// BC-2.07.003 PC-9: When `load_config()` encounters a parse failure (Case 3),
/// it MUST emit a `tracing::warn!` log message before returning the default config.
///
/// Uses `tracing_test::traced_test` to capture log output and assert the warn was
/// emitted with the expected log target "monocle_config".
#[test]
#[traced_test]
fn test_BC_2_07_003_corrupted_config_emits_warn() {
    let tmp = tempdir().expect("create temp dir");
    let config_path = tmp.path().join("config.json");

    // Write deliberately corrupted JSON — will trigger Case 3 parse failure.
    write_fixture(tmp.path(), "config.json", r#"{"not_valid_json": "#);

    let result = load_config(&config_path);
    assert!(
        result.is_ok(),
        "load_config() must return Ok(default) on parse failure (BC-2.07.003 PC-9); \
        got Err: {:?}",
        result.err()
    );

    // Assert that a warn-level log was emitted containing the expected substring.
    // BC-2.07.003 PC-9: "a tracing::warn! MUST be emitted" on parse failure.
    assert!(
        logs_contain("monocle-config: failed to parse config file"),
        "load_config() must emit tracing::warn! containing 'monocle-config: failed to parse \
        config file' on parse failure (BC-2.07.003 PC-9)"
    );
}
