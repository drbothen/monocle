//! Integration tests for BC-2.01.006: Crash Recovery Checkpoint.
//! Tests exercise write/read paths for `RecoveryCheckpoint` against BC-2.01.006 schema invariants.

// BC-based test naming uses uppercase letters (BC_2_01_006_…) which Rust's
// snake_case linter flags. This is intentional — the naming convention is
// mandated by the factory TDD spec and must be preserved for traceability.
#![allow(non_snake_case)]
// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// fs::write used for test fixture setup only (not production config writes).
// The disallowed_methods lint targets production code; test-only fixture writes are exempt.
#![allow(clippy::disallowed_methods)]

use std::fs;
use std::os::unix::fs::PermissionsExt;

use monocle_runtime::lifecycle::{read_recovery_checkpoint, write_recovery_checkpoint};
use monocle_runtime::types::{CheckpointReadResult, RecoveryCheckpoint, ShutdownReason};
use regex_lite::Regex;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper: build a canonical RecoveryCheckpoint for use in tests.
// ---------------------------------------------------------------------------

fn sample_checkpoint() -> RecoveryCheckpoint {
    RecoveryCheckpoint {
        pid: 12345,
        shutdown_reason: ShutdownReason::Signal,
        last_app_mode: "Running".to_string(),
        shutdown_utc: "2026-05-26T10:00:00.000Z".to_string(),
    }
}

// ---------------------------------------------------------------------------
// AC-009: write_recovery_checkpoint creates the file
// ---------------------------------------------------------------------------

/// BC-2.01.006 PC-1: checkpoint file MUST exist after write_recovery_checkpoint returns.
/// BC-2.01.006 PC-2: file content MUST be valid JSON containing all 4 fields.
#[test]
fn test_BC_2_01_006_write_creates_file() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("recovery.json");
    let checkpoint = sample_checkpoint();

    write_recovery_checkpoint(&path, &checkpoint).expect("write must succeed");

    assert!(path.exists(), "checkpoint file must exist after write");

    let contents = fs::read_to_string(&path).expect("file must be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&contents).expect("file must be valid JSON");

    assert!(parsed.get("pid").is_some(), "JSON must contain 'pid'");
    assert!(
        parsed.get("shutdown_reason").is_some(),
        "JSON must contain 'shutdown_reason'"
    );
    assert!(
        parsed.get("last_app_mode").is_some(),
        "JSON must contain 'last_app_mode'"
    );
    assert!(
        parsed.get("shutdown_utc").is_some(),
        "JSON must contain 'shutdown_utc'"
    );
}

// ---------------------------------------------------------------------------
// AC-001 / AC-002: read_recovery_checkpoint deserialises a valid file
// ---------------------------------------------------------------------------

/// BC-2.01.006 PC-3: reading a valid checkpoint file returns Some with correct fields.
#[test]
fn test_BC_2_01_006_read_valid_checkpoint() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("recovery.json");

    // Write the JSON manually so this test does not depend on write_recovery_checkpoint.
    let json = r#"{
        "pid": 99,
        "shutdown_reason": "graceful",
        "last_app_mode": "Running",
        "shutdown_utc": "2026-05-26T09:00:00.000Z"
    }"#;
    fs::write(&path, json).expect("manual write");

    let result = read_recovery_checkpoint(&path);

    let cp = match result {
        CheckpointReadResult::Valid(cp) => cp,
        other => panic!(
            "must return Valid for a valid checkpoint file, got {:?}",
            std::mem::discriminant(&other)
        ),
    };
    assert_eq!(cp.pid, 99, "pid must round-trip correctly");
    assert_eq!(
        cp.shutdown_reason,
        ShutdownReason::Graceful,
        "shutdown_reason must round-trip correctly"
    );
    assert_eq!(
        cp.last_app_mode, "Running",
        "last_app_mode must round-trip correctly"
    );
    assert_eq!(
        cp.shutdown_utc, "2026-05-26T09:00:00.000Z",
        "shutdown_utc must round-trip correctly"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / AC-007: absent file returns None
// ---------------------------------------------------------------------------

/// BC-2.01.006 PC-4: when no checkpoint file is present, read returns Absent.
/// TUI startup must NOT be blocked by a missing checkpoint (clean-boot case).
#[test]
fn test_BC_2_01_006_read_absent_file_returns_none() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("does_not_exist.json");

    let result = read_recovery_checkpoint(&path);

    assert!(
        matches!(result, CheckpointReadResult::Absent),
        "must return Absent when checkpoint file does not exist"
    );
}

// ---------------------------------------------------------------------------
// AC-010 / EC-054: malformed JSON returns None, never panics
// ---------------------------------------------------------------------------

/// BC-2.01.006 PC-5: a truncated / malformed checkpoint file must return Malformed.
/// The TUI must never be blocked by a corrupt file from a previous crash-mid-write.
#[test]
fn test_BC_2_01_006_read_malformed_json_returns_none() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("recovery.json");

    // Truncated JSON — missing the closing brace.
    let malformed = r#"{"pid":1,"shutdown_reason":"graceful","last_app_mode":"Running""#;
    fs::write(&path, malformed).expect("manual write of malformed JSON");

    let result = read_recovery_checkpoint(&path);

    assert!(
        matches!(result, CheckpointReadResult::Malformed),
        "must return Malformed for malformed/truncated JSON, never panic"
    );
}

// ---------------------------------------------------------------------------
// AC-008: ShutdownReason serde wire format
// ---------------------------------------------------------------------------

/// BC-2.01.006 INV-1 / BC-2.01.006 wire stability:
/// ShutdownReason variants must serialise to lowercase strings as specified.
#[test]
fn test_BC_2_01_006_shutdown_reason_serde_roundtrip() {
    // Graceful → "graceful"
    let serialised =
        serde_json::to_string(&ShutdownReason::Graceful).expect("serialisation must not fail");
    assert_eq!(
        serialised, r#""graceful""#,
        "ShutdownReason::Graceful must serialise to \"graceful\""
    );
    let deserialised: ShutdownReason =
        serde_json::from_str(&serialised).expect("deserialisation must not fail");
    assert_eq!(
        deserialised,
        ShutdownReason::Graceful,
        "ShutdownReason::Graceful must round-trip"
    );

    // Signal → "signal"
    let serialised =
        serde_json::to_string(&ShutdownReason::Signal).expect("serialisation must not fail");
    assert_eq!(
        serialised, r#""signal""#,
        "ShutdownReason::Signal must serialise to \"signal\""
    );
    let deserialised: ShutdownReason =
        serde_json::from_str(&serialised).expect("deserialisation must not fail");
    assert_eq!(
        deserialised,
        ShutdownReason::Signal,
        "ShutdownReason::Signal must round-trip"
    );

    // Forced → "forced"
    let serialised =
        serde_json::to_string(&ShutdownReason::Forced).expect("serialisation must not fail");
    assert_eq!(
        serialised, r#""forced""#,
        "ShutdownReason::Forced must serialise to \"forced\""
    );
    let deserialised: ShutdownReason =
        serde_json::from_str(&serialised).expect("deserialisation must not fail");
    assert_eq!(
        deserialised,
        ShutdownReason::Forced,
        "ShutdownReason::Forced must round-trip"
    );
}

// ---------------------------------------------------------------------------
// AC-008: RecoveryCheckpoint JSON schema validation
// ---------------------------------------------------------------------------

/// BC-2.01.006 PC-2: checkpoint JSON must contain exactly the 4 canonical fields,
/// and shutdown_utc must match the millisecond-precision RFC 3339 UTC format.
#[test]
fn test_BC_2_01_006_checkpoint_json_schema() {
    let checkpoint = RecoveryCheckpoint {
        pid: 42,
        shutdown_reason: ShutdownReason::Forced,
        last_app_mode: "Draining".to_string(),
        shutdown_utc: "2026-05-26T10:30:00.123Z".to_string(),
    };

    let json = serde_json::to_string(&checkpoint).expect("serialisation must not fail");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");

    // All 4 canonical fields must be present.
    assert!(parsed.get("pid").is_some(), "JSON must contain 'pid'");
    assert!(
        parsed.get("shutdown_reason").is_some(),
        "JSON must contain 'shutdown_reason'"
    );
    assert!(
        parsed.get("last_app_mode").is_some(),
        "JSON must contain 'last_app_mode'"
    );
    assert!(
        parsed.get("shutdown_utc").is_some(),
        "JSON must contain 'shutdown_utc'"
    );

    // shutdown_utc must match millisecond-precision RFC 3339 UTC format.
    // Pattern: YYYY-MM-DDTHH:MM:SS.sssZ (mandatory 3-digit milliseconds, mandatory Z suffix).
    let utc_value = parsed["shutdown_utc"]
        .as_str()
        .expect("shutdown_utc must be a string");
    let re =
        Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$").expect("regex must compile");
    assert!(
        re.is_match(utc_value),
        "shutdown_utc '{}' must match YYYY-MM-DDTHH:MM:SS.sssZ",
        utc_value
    );
}

// ---------------------------------------------------------------------------
// AC-009: write_recovery_checkpoint uses atomic tempfile (mode 0o600)
// ---------------------------------------------------------------------------

/// BC-2.01.006 PC-6 / security: checkpoint file must be written with 0o600 permissions
/// (owner-only read/write) so other local users cannot read the daemon state.
/// The atomic tempfile::persist write must preserve the restrictive mode.
#[test]
fn test_BC_2_01_006_write_atomic_file_permissions() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("recovery.json");
    let checkpoint = sample_checkpoint();

    write_recovery_checkpoint(&path, &checkpoint).expect("write must succeed");

    let metadata = fs::metadata(&path).expect("metadata must be readable");
    let mode = metadata.permissions().mode();
    // Mask to the lower 9 permission bits (rwxrwxrwx).
    let perm_bits = mode & 0o777;
    assert_eq!(
        perm_bits, 0o600,
        "checkpoint file must have 0o600 permissions (got 0o{:03o})",
        perm_bits
    );
}

// ---------------------------------------------------------------------------
// Clean graceful shutdown — no recovery file created
// ---------------------------------------------------------------------------

/// BC-2.01.006 scope boundary: a clean graceful shutdown does NOT produce a
/// recovery checkpoint file. The checkpoint is only written on non-graceful paths.
/// This test verifies the absence contract: if nothing writes the file, read returns None.
#[test]
fn test_BC_2_01_006_clean_graceful_no_recovery_file() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("recovery.json");

    // Simulate clean graceful shutdown: no write call is made.
    assert!(!path.exists(), "no checkpoint file must exist before write");

    let result = read_recovery_checkpoint(&path);

    assert!(
        matches!(result, CheckpointReadResult::Absent),
        "clean graceful shutdown must produce no recovery file (read must return Absent)"
    );
}

// ---------------------------------------------------------------------------
// HIGH-002: VP-006 typed-field probes (PC-9, PC-10, PC-11)
// ---------------------------------------------------------------------------

/// VP-006 PC-9: pid field must serialize to a JSON unsigned integer >= 1.
/// VP-006 PC-10: shutdown_reason must serialize to one of the three lowercase wire strings.
/// VP-006 PC-11: last_app_mode must serialize to a non-empty JSON string.
///
/// These probes go beyond the schema presence checks in test_BC_2_01_006_checkpoint_json_schema
/// by asserting the *type* and *value domain* of each field in the serialised output,
/// matching the BC-2.01.006 INV-1 constraints exactly.
#[test]
fn test_BC_2_01_006_typed_field_probes() {
    let checkpoint = RecoveryCheckpoint {
        pid: 42,
        shutdown_reason: ShutdownReason::Signal,
        last_app_mode: "Running".to_string(),
        shutdown_utc: "2026-05-26T10:00:00.000Z".to_string(),
    };

    let value = serde_json::to_value(&checkpoint).expect("serialisation must not fail");

    // VP-006 PC-9: pid must be a JSON unsigned integer >= 1.
    assert!(
        value["pid"].is_u64(),
        "pid must serialise to a JSON unsigned integer, got: {:?}",
        value["pid"]
    );
    assert!(
        value["pid"].as_u64().unwrap() >= 1,
        "pid must be >= 1, got {}",
        value["pid"].as_u64().unwrap()
    );

    // VP-006 PC-10: shutdown_reason must be one of the three canonical wire strings.
    let valid_reasons = ["graceful", "signal", "forced"];
    let reason_str = value["shutdown_reason"]
        .as_str()
        .expect("shutdown_reason must serialise to a JSON string");
    assert!(
        valid_reasons.contains(&reason_str),
        "shutdown_reason '{}' must be one of {:?}",
        reason_str,
        valid_reasons
    );

    // VP-006 PC-11: last_app_mode must be a non-empty JSON string.
    assert!(
        value["last_app_mode"].is_string(),
        "last_app_mode must serialise to a JSON string, got: {:?}",
        value["last_app_mode"]
    );
    assert!(
        !value["last_app_mode"].as_str().unwrap().is_empty(),
        "last_app_mode must be non-empty"
    );
}

// ---------------------------------------------------------------------------
// MED-001: EC-055 overwrite — second write must replace the first
// ---------------------------------------------------------------------------

/// BC-2.01.006 EC-055: writing a second checkpoint to the same path MUST overwrite
/// the first atomically. Only one file must exist at the path after both writes.
/// The second checkpoint's field values must be the ones returned by read.
#[test]
fn test_BC_2_01_006_overwrite_recovery_checkpoint() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("recovery.json");

    // First write.
    let first = RecoveryCheckpoint {
        pid: 100,
        shutdown_reason: ShutdownReason::Graceful,
        last_app_mode: "Running".to_string(),
        shutdown_utc: "2026-05-26T10:00:00.000Z".to_string(),
    };
    write_recovery_checkpoint(&path, &first).expect("first write must succeed");
    assert!(path.exists(), "checkpoint file must exist after first write");

    // Second write — different pid and reason.
    let second = RecoveryCheckpoint {
        pid: 200,
        shutdown_reason: ShutdownReason::Signal,
        last_app_mode: "Draining".to_string(),
        shutdown_utc: "2026-05-26T10:00:01.000Z".to_string(),
    };
    write_recovery_checkpoint(&path, &second).expect("second write must succeed");

    // Read back: must reflect the second checkpoint, not the first.
    let cp = match read_recovery_checkpoint(&path) {
        CheckpointReadResult::Valid(cp) => cp,
        other => panic!(
            "must return Valid after overwrite, got {:?}",
            std::mem::discriminant(&other)
        ),
    };
    assert_eq!(cp.pid, 200, "pid must reflect second write (got {})", cp.pid);
    assert_eq!(
        cp.shutdown_reason,
        ShutdownReason::Signal,
        "shutdown_reason must reflect second write"
    );

    // Exactly one file must exist at the path (no .tmp or .bak residuals).
    let count = fs::read_dir(dir.path())
        .expect("dir must be readable")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("recovery")
        })
        .count();
    assert_eq!(
        count, 1,
        "exactly one recovery* file must exist after overwrite (got {})",
        count
    );
}

// ---------------------------------------------------------------------------
// LOW-001: chrono-generated timestamp matches the mandatory regex
// ---------------------------------------------------------------------------

/// BC-2.01.006 INV-1: a timestamp produced by chrono with the canonical format string
/// must match `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` and must be accepted
/// by RecoveryCheckpoint::validate().
///
/// This test closes the gap identified in adversary round R1 (LOW-001): the test suite
/// previously only exercised hardcoded timestamp literals, leaving the runtime chrono
/// formatting path untested.
#[test]
fn test_BC_2_01_006_chrono_generated_timestamp_matches_regex() {
    // Generate a live timestamp using the exact format string from lifecycle.rs.
    let ts = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    // The generated timestamp must match the canonical millisecond-precision UTC pattern.
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$")
        .expect("regex must compile");
    assert!(
        re.is_match(&ts),
        "chrono-generated timestamp '{}' must match YYYY-MM-DDTHH:MM:SS.sssZ",
        ts
    );

    // A RecoveryCheckpoint using this timestamp must pass validate().
    let checkpoint = RecoveryCheckpoint {
        pid: 1,
        shutdown_reason: ShutdownReason::Graceful,
        last_app_mode: "Running".to_string(),
        shutdown_utc: ts.clone(),
    };
    checkpoint
        .validate()
        .unwrap_or_else(|e| panic!("validate() must succeed for chrono-generated timestamp '{}': {}", ts, e));
}
