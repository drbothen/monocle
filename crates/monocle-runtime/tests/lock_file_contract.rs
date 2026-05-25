//! Integration tests for BC-2.01.010: Lock File Contract Version Field.
//!
//! Covers ACs 010-013: contract_version first key, missing contract_version,
//! contract_version as string instead of integer, and unknown contract_version values.
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause | VP |
//! |---------------|----|-----------|----|
//! | test_BC_2_01_010_contract_version_equals_1_and_is_first_key | AC-010 | BC-2.01.010 PC-1, PC-2 | VP-010 |
//! | test_BC_2_01_010_app_field_equals_monocle | AC-010 | BC-2.01.010 PC-3 | VP-010 |
//! | test_BC_2_01_010_unknown_contract_version_treated_as_stale | AC-010 | BC-2.01.010 PC-4, EC-010 | VP-010 |
//! | test_BC_2_01_010_missing_contract_version_treated_as_stale | AC-011, AC-013 | BC-2.01.010 EC-012 | VP-010 |
//! | test_BC_2_01_010_string_contract_version_handled_gracefully | AC-012 | BC-2.01.010 EC-011 | VP-010 |
//! | test_BC_2_01_010_contract_version_key_absent_entirely_treated_as_stale | AC-013 | BC-2.01.010 EC-012 | VP-010 |
//! | test_BC_2_01_010_invariant_contract_version_first_via_raw_scan | AC-010 | BC-2.01.010 INV-1 | VP-010 |
//!
//! # Red Gate
//!
//! Tests that call DaemonLock::acquire MUST FAIL against stubs (unimplemented!() panic).
//! Tests that exercise the stale-lock path via written JSON files + acquire() will also
//! fail at the unimplemented!() boundary.

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per naming convention.
#![allow(non_snake_case)]

use monocle_runtime::errors::DaemonStartError;
use monocle_runtime::lock::DaemonLock;

/// Test port for lock acquisition.
const TEST_PORT: u16 = 39_002;

/// Write test-fixture JSON content to a path atomically via tempfile::NamedTempFile + persist.
///
/// The project-wide semgrep/clippy anti-pattern rule bans `std::fs::write` blanket in all code,
/// including tests. Using `NamedTempFile + persist` is the same atomic pattern required by
/// production code, so this helper is also semantically correct for test setup.
fn write_test_fixture_to(dir: &std::path::Path, filename: &str, content: &str) {
    use std::io::Write;
    let mut tmp =
        tempfile::NamedTempFile::new_in(dir).expect("create NamedTempFile in test runtime dir");
    tmp.write_all(content.as_bytes())
        .expect("write test fixture content");
    let dest = dir.join(filename);
    tmp.persist(&dest).expect("persist test fixture atomically");
}

/// A PID that will never be alive (far above kernel PID max on all supported platforms).
const DEAD_PID: i32 = i32::MAX - 1;

// ---------------------------------------------------------------------------
// AC-010 — contract_version first key, value 1
// ---------------------------------------------------------------------------

/// BC-2.01.010 PC-1 + PC-2: After a clean acquire(), the lock file JSON must have
/// `contract_version` as the FIRST key with value `1`.
///
/// VP-010 requires verification via `serde_json::Value::Object` iteration — however,
/// since serde_json without `preserve_order` feature does not guarantee iteration order
/// of `serde_json::Map`, this test verifies ordering via raw string position scanning
/// (which reflects the actual serialization byte order) AND verifies the value via
/// parsed JSON.
///
/// Traces to BC-2.01.010 PC-1, PC-2, AC-010, VP-010.
#[test]
fn test_BC_2_01_010_contract_version_equals_1_and_is_first_key() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir (BC-2.01.010 PC-1 precondition)");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");

    // --- Verify contract_version value is 1 ---
    let json: serde_json::Value = serde_json::from_str(&raw).expect("lock file must be valid JSON");
    let cv = json
        .get("contract_version")
        .and_then(|v| v.as_u64())
        .expect("contract_version must be a non-negative integer (BC-2.01.010 PC-2)");
    assert_eq!(
        cv, 1u64,
        "contract_version must equal 1 for all Phase 1 daemons (BC-2.01.010 PC-2 / AC-010); \
        got: {cv}"
    );

    // --- Verify contract_version appears FIRST in raw JSON ---
    let cv_pos = raw
        .find("\"contract_version\"")
        .expect("\"contract_version\" key must appear in lock file JSON");

    let other_keys = [
        "\"pid\"",
        "\"port\"",
        "\"authToken\"",
        "\"startTimeUtc\"",
        "\"app\"",
        "\"version\"",
    ];
    for key in other_keys {
        if let Some(pos) = raw.find(key) {
            assert!(
                cv_pos < pos,
                "\"contract_version\" must appear before {key} in raw JSON \
                (BC-2.01.010 PC-2 / AC-010 / INV-1): cv at byte {cv_pos}, {key} at byte {pos}. \
                Ordered struct serialization (not HashMap) is required."
            );
        }
    }

    drop(lock);
}

/// BC-2.01.010 PC-3: The `app` field must equal the exact string `"monocle"`.
///
/// Future hook-discovery tooling filters by this field name.
///
/// Traces to BC-2.01.010 PC-3, AC-010, VP-010.
#[test]
fn test_BC_2_01_010_app_field_equals_monocle() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT).expect("acquire must succeed");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");

    let app = json
        .get("app")
        .and_then(|v| v.as_str())
        .expect("app field must be a JSON string");

    assert_eq!(
        app, "monocle",
        "app field must be the exact string \"monocle\" per BC-2.01.010 PC-3 / AC-010; \
        got: {app:?}. No variants (e.g., \"Monocle\", \"MONOCLE\") are acceptable."
    );

    drop(lock);
}

/// BC-2.01.010 PC-4 + EC-010: When acquire() encounters an existing lock file with an
/// unrecognized `contract_version` (e.g., a hypothetical future Phase 4 format), it must
/// treat the file as stale and proceed with startup.
///
/// An unrecognized `contract_version` MUST NOT cause a panic or a crash. The lock file
/// should be cleaned up and a new one written.
///
/// Traces to BC-2.01.010 PC-4, EC-010, AC-010, VP-010.
#[test]
fn test_BC_2_01_010_unknown_contract_version_treated_as_stale() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let runtime_dir = tmp.path();

    // Write a lock file with a future unknown contract_version.
    let future_lock = serde_json::json!({
        "contract_version": 99,
        "pid": DEAD_PID,
        "port": 9000,
        "authToken": "d".repeat(64),
        "startTimeUtc": "2030-01-01T00:00:00.000Z",
        "app": "monocle",
        "version": "99.0.0"
    });
    let lock_path = runtime_dir.join("monocle.lock");
    write_test_fixture_to(
        runtime_dir,
        "monocle.lock",
        &serde_json::to_string(&future_lock).unwrap(),
    );

    // acquire() must succeed — unknown contract_version is treated as stale.
    let result = DaemonLock::acquire(runtime_dir, TEST_PORT);
    assert!(
        result.is_ok(),
        "DaemonLock::acquire must succeed when an existing lock file has an unrecognized \
        contract_version (99) — treat as stale per BC-2.01.010 PC-4 / EC-010 / AC-010; \
        got: {result:?}. Counter-example: panicking or returning an error would prevent \
        daemon startup when the lock file format has evolved."
    );

    // After cleanup, the new lock file must have contract_version == 1.
    let (lock, _token) = result.unwrap();
    let new_raw =
        std::fs::read_to_string(&lock_path).expect("new lock file must exist after stale cleanup");
    let new_json: serde_json::Value = serde_json::from_str(&new_raw).expect("valid JSON");
    let new_cv = new_json
        .get("contract_version")
        .and_then(|v| v.as_u64())
        .expect("contract_version must be in new lock file");
    assert_eq!(
        new_cv, 1u64,
        "New lock file after unknown-version cleanup must have contract_version == 1 \
        (BC-2.01.010 PC-2 / AC-010); got: {new_cv}"
    );

    drop(lock);
}

// ---------------------------------------------------------------------------
// AC-011 + AC-013 — Missing contract_version treated as stale
// ---------------------------------------------------------------------------

/// BC-2.01.010 EC-012: When the existing lock file is missing the `contract_version` key
/// entirely (pre-Phase-1 format), acquire() must treat it as stale and proceed.
///
/// "Same treatment as EC-010: log WARN and proceed as if no lock file exists."
///
/// Traces to BC-2.01.010 EC-012, AC-011, AC-013, VP-010.
#[test]
fn test_BC_2_01_010_missing_contract_version_treated_as_stale() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let runtime_dir = tmp.path();

    // Write a lock file with no contract_version key at all.
    let old_format_lock = serde_json::json!({
        "pid": DEAD_PID,
        "port": 9000,
        "authToken": "e".repeat(64),
        "startTimeUtc": "2025-01-01T00:00:00.000Z",
        "app": "monocle",
        "version": "0.0.1"
    });
    let lock_path = runtime_dir.join("monocle.lock");
    write_test_fixture_to(
        runtime_dir,
        "monocle.lock",
        &serde_json::to_string(&old_format_lock).unwrap(),
    );

    // acquire() must succeed — missing contract_version treated as stale.
    let result = DaemonLock::acquire(runtime_dir, TEST_PORT);
    assert!(
        result.is_ok(),
        "DaemonLock::acquire must succeed when an existing lock file is missing \
        contract_version entirely — treat as stale per BC-2.01.010 EC-012 / AC-011 / AC-013; \
        got: {result:?}. No crash, no panic."
    );

    // The new lock file must have contract_version == 1 as the first key.
    let (lock, _token) = result.unwrap();
    let new_raw = std::fs::read_to_string(&lock_path)
        .expect("new lock file must exist after missing-version stale cleanup");
    let new_json: serde_json::Value = serde_json::from_str(&new_raw).expect("valid JSON");
    let new_cv = new_json
        .get("contract_version")
        .and_then(|v| v.as_u64())
        .expect("new lock file must have contract_version");
    assert_eq!(
        new_cv, 1u64,
        "New lock file must have contract_version == 1 after missing-version cleanup; \
        got: {new_cv}"
    );

    drop(lock);
}

// ---------------------------------------------------------------------------
// AC-012 — contract_version as string "1" → handled gracefully (no crash)
// ---------------------------------------------------------------------------

/// BC-2.01.010 EC-011: When the existing lock file has `contract_version` as a string
/// (e.g., `"1"` instead of integer `1`), the Phase 1 reader must handle this gracefully.
///
/// Acceptable behaviors (per AC-012):
/// 1. Coerce the string to integer and proceed as if it were a live/dead pid.
/// 2. Treat as stale (log E-LOCK-002) and proceed with a fresh lock.
///
/// NOT acceptable: panic, crash, or returning a non-LockFileConflict/LockFileWriteFailure error.
///
/// Traces to BC-2.01.010 EC-011, AC-012, VP-010.
#[test]
fn test_BC_2_01_010_string_contract_version_handled_gracefully() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let runtime_dir = tmp.path();

    // Write a lock file with contract_version as a STRING "1" (not integer 1).
    // Use a dead PID so that even if coercion succeeds and PID-liveness is checked,
    // the result is "stale" (proceed) rather than "live" (conflict).
    let bad_format_lock = serde_json::json!({
        "contract_version": "1",
        "pid": DEAD_PID,
        "port": 9000,
        "authToken": "f".repeat(64),
        "startTimeUtc": "2026-05-25T10:00:00.000Z",
        "app": "monocle",
        "version": "0.1.0"
    });
    let lock_path = runtime_dir.join("monocle.lock");
    write_test_fixture_to(
        runtime_dir,
        "monocle.lock",
        &serde_json::to_string(&bad_format_lock).unwrap(),
    );

    // acquire() must succeed without panicking — no crash on string contract_version.
    let result = DaemonLock::acquire(runtime_dir, TEST_PORT);

    // Must not be an unexpected error variant.
    match &result {
        Ok(_) => {
            // Acceptable: coerced or treated as stale, new lock written.
        }
        Err(DaemonStartError::LockFileConflict { pid }) => {
            // Acceptable ONLY if the coercion path resolved the pid to a live process.
            // With DEAD_PID this should not happen, but if it does it means the coercion
            // path checked pid liveness and found it alive — which is a logic error for
            // i32::MAX - 1 but not a format-handling failure.
            panic!(
                "Unexpected LockFileConflict with pid={pid} when the lock file had a \
                dead pid ({DEAD_PID}) — this indicates the string contract_version coercion \
                path produced a conflicting result. \
                BC-2.01.010 EC-011 / AC-012"
            );
        }
        Err(DaemonStartError::LockFileWriteFailure(_)) => {
            panic!(
                "DaemonLock::acquire returned LockFileWriteFailure on a string contract_version \
                lock file — the writer path should not be reached if the stale-cleanup path \
                fails due to a filesystem error; \
                BC-2.01.010 EC-011 / AC-012: {result:?}"
            );
        }
        Err(DaemonStartError::RuntimeDirUnresolvable) => {
            panic!(
                "DaemonLock::acquire returned RuntimeDirUnresolvable on a string \
                contract_version lock file — this error is not expected here; \
                BC-2.01.010 EC-011 / AC-012"
            );
        }
        Err(DaemonStartError::RuntimeDirCreateFailure(_)) => {
            panic!(
                "DaemonLock::acquire returned RuntimeDirCreateFailure on a string \
                contract_version lock file — this error is not expected here; \
                BC-2.01.010 EC-011 / AC-012"
            );
        }
    }

    // Main contract: no panic, no crash. If result is Ok, assert the new lock is valid.
    if let Ok((lock, _token)) = result {
        assert!(
            lock_path.exists(),
            "lock file must exist after successful acquire despite string contract_version \
            in old lock (BC-2.01.010 EC-011 / AC-012)"
        );
        drop(lock);
    }
}

// ---------------------------------------------------------------------------
// AC-013 — contract_version key missing entirely → treated as stale
// ---------------------------------------------------------------------------

/// BC-2.01.010 EC-012 (explicit): Same as EC-011 but exercises the path where
/// `contract_version` is entirely absent (not even present as a wrong type).
///
/// This is a duplicate angle on AC-013, testing a DIFFERENT input format than
/// test_BC_2_01_010_missing_contract_version_treated_as_stale (which tests with
/// a realistic pre-Phase-1 format that includes other fields in a different order).
///
/// This test uses a MINIMAL lock file with just a pid — the most stripped-down
/// form of a missing-contract_version file.
///
/// Traces to BC-2.01.010 EC-012, AC-013, VP-010.
#[test]
fn test_BC_2_01_010_contract_version_key_absent_entirely_treated_as_stale() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let runtime_dir = tmp.path();

    // Minimal lock file with no contract_version at all.
    let minimal_lock = serde_json::json!({
        "pid": DEAD_PID
    });
    let lock_path = runtime_dir.join("monocle.lock");
    write_test_fixture_to(
        runtime_dir,
        "monocle.lock",
        &serde_json::to_string(&minimal_lock).unwrap(),
    );

    let result = DaemonLock::acquire(runtime_dir, TEST_PORT);

    assert!(
        result.is_ok(),
        "DaemonLock::acquire must succeed when an existing lock file has no \
        contract_version key — treat as stale per BC-2.01.010 EC-012 / AC-013; \
        got: {result:?}"
    );

    let (lock, _token) = result.unwrap();

    // New lock file must have contract_version == 1.
    let new_raw = std::fs::read_to_string(&lock_path)
        .expect("new lock file must exist after minimal-no-version stale cleanup");
    let new_json: serde_json::Value = serde_json::from_str(&new_raw).expect("valid JSON");

    assert!(
        new_json.get("contract_version").is_some(),
        "New lock file must include contract_version key (BC-2.01.010 PC-2 / AC-013); \
        lock file after cleanup: {new_json}"
    );

    let cv = new_json
        .get("contract_version")
        .and_then(|v| v.as_u64())
        .expect("contract_version must be a non-negative integer");
    assert_eq!(
        cv, 1u64,
        "New lock file contract_version must equal 1 after absent-version cleanup; \
        got: {cv}"
    );

    drop(lock);
}

// ---------------------------------------------------------------------------
// BC-2.01.010 INV-1 — Raw string scan verifies first-key ordering convention
// ---------------------------------------------------------------------------

/// BC-2.01.010 INV-1: The `contract_version` first-key convention parallels the
/// `format_version` convention in BC-2.01.007 JSONL ring. Both formats put the version
/// sentinel first so readers can validate before deserializing remaining fields.
///
/// This test acquires a fresh lock and verifies via raw byte position that
/// "contract_version" appears before every other key in the serialized JSON text.
/// This catches the case where a HashMap-backed serializer is used (which does not
/// preserve insertion order).
///
/// Traces to BC-2.01.010 INV-1, AC-010, VP-010.
#[test]
fn test_BC_2_01_010_invariant_contract_version_first_via_raw_scan() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT).expect("acquire must succeed");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");

    let cv_pos = raw
        .find("\"contract_version\"")
        .expect("contract_version must appear in lock file JSON raw text");

    // All other required keys must appear AFTER contract_version in the raw JSON.
    let required_later_keys = [
        "\"pid\"",
        "\"port\"",
        "\"authToken\"",
        "\"startTimeUtc\"",
        "\"app\"",
        "\"version\"",
    ];

    for key in required_later_keys {
        let key_pos = raw.find(key).unwrap_or_else(|| {
            panic!(
                "Required key {key} must appear in lock file JSON (BC-2.01.010 PC-1 / INV-1); \
                raw content: {raw:?}"
            )
        });
        assert!(
            cv_pos < key_pos,
            "\"contract_version\" (byte {cv_pos}) must appear BEFORE {key} (byte {key_pos}) \
            in raw JSON serialization per BC-2.01.010 INV-1 / AC-010. \
            Counter-example: a struct with fields in wrong order, or a HashMap-backed \
            serde serializer, would violate this invariant."
        );
    }

    drop(lock);
}
