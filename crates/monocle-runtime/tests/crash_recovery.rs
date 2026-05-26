//! Integration tests for BC-2.01.006: Crash Recovery Checkpoint.
//!
//! Every test in this file MUST compile and MUST fail with the `todo!()` stubs
//! in `monocle_runtime::lifecycle`. These are the Red Gate tests for S-007.
//!
//! Test naming follows the BC-based convention:
//!   `test_BC_2_01_006_<assertion_name>()`

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
use monocle_runtime::types::{RecoveryCheckpoint, ShutdownReason};
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

    assert!(
        result.is_some(),
        "must return Some for a valid checkpoint file"
    );
    let cp = result.unwrap();
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

/// BC-2.01.006 PC-4: when no checkpoint file is present, read returns None.
/// TUI startup must NOT be blocked by a missing checkpoint (clean-boot case).
#[test]
fn test_BC_2_01_006_read_absent_file_returns_none() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("does_not_exist.json");

    let result = read_recovery_checkpoint(&path);

    assert!(
        result.is_none(),
        "must return None when checkpoint file does not exist"
    );
}

// ---------------------------------------------------------------------------
// AC-010 / EC-054: malformed JSON returns None, never panics
// ---------------------------------------------------------------------------

/// BC-2.01.006 PC-5: a truncated / malformed checkpoint file must silently return None.
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
        result.is_none(),
        "must return None for malformed/truncated JSON, never panic"
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
        result.is_none(),
        "clean graceful shutdown must produce no recovery file (read must return None)"
    );
}
