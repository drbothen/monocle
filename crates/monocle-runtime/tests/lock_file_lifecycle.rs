//! Integration tests for BC-2.01.005: Lock File Atomic Lifecycle.
//! Also covers BC-2.01.008 (Auth Token Wire Format) for the token generation
//! aspect exercised in AC-014.
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause | VP |
//! |---------------|----|-----------|----|
//! | test_BC_2_01_005_clean_start_creates_lock_file | AC-001 | BC-2.01.005 PC-3 | VP-005 |
//! | test_BC_2_01_005_lock_file_mode_is_0o600 | AC-001 | BC-2.01.005 PC-3, INV-3 | VP-005 |
//! | test_BC_2_01_005_lock_file_path_is_in_runtime_dir | AC-001 | BC-2.01.005 PC-3 | VP-005 |
//! | test_BC_2_01_005_json_has_7_fields_correct_types | AC-002 | BC-2.01.005 PC-4 | VP-005 |
//! | test_BC_2_01_005_json_field_contract_version_is_first | AC-002, AC-010 | BC-2.01.005 PC-4, BC-2.01.010 PC-2 | VP-005, VP-010 |
//! | test_BC_2_01_005_json_pid_field_is_current_process | AC-002 | BC-2.01.005 PC-4 | VP-005 |
//! | test_BC_2_01_005_json_app_field_is_monocle | AC-002 | BC-2.01.005 PC-4, BC-2.01.010 PC-3 | VP-005, VP-010 |
//! | test_BC_2_01_005_json_start_time_is_iso8601 | AC-002 | BC-2.01.005 PC-4 | VP-005 |
//! | test_BC_2_01_005_live_pid_conflict_returns_error | AC-003 | BC-2.01.005 PC-1 | VP-005 |
//! | test_BC_2_01_005_stale_pid_cleaned_up | AC-004 | BC-2.01.005 PC-2 | VP-005 |
//! | test_BC_2_01_005_stale_pid_new_lock_acquired | AC-004 | BC-2.01.005 PC-2, PC-3 | VP-005 |
//! | test_BC_2_01_005_release_removes_lock_file | AC-005 | BC-2.01.005 PC-6 | VP-005 |
//! | test_BC_2_01_005_release_removes_sock_file | AC-005 | BC-2.01.005 PC-7 | VP-005 |
//! | test_BC_2_01_005_runtime_dir_created_with_0o700 | AC-006 | BC-2.01.005 PC-8 | VP-005 |
//! | test_BC_2_01_005_env_override_monocle_runtime_dir | AC-007 | BC-2.01.005 PC-2a, EC-058 | VP-005 |
//! | test_BC_2_01_005_env_override_empty_string_falls_through | AC-007 | BC-2.01.005 EC-060 | VP-005 |
//! | test_BC_2_01_005_runtimedirunresolvable_when_no_home | AC-009 | BC-2.01.005 PC-2d, EC-059 | VP-005 |
//! | test_BC_2_01_008_auth_token_is_64_hex | AC-014 | BC-2.01.008 PC-1 | VP-005 |
//! | test_BC_2_01_008_auth_token_matches_regex | AC-014 | BC-2.01.008 PC-1 | VP-005 |
//! | test_BC_2_01_008_generate_session_token_format | AC-014 | BC-2.01.008 PC-1 | VP-005 |
//! | test_BC_2_01_008_generate_session_token_is_random | AC-014 | BC-2.01.008 PC-1 | VP-005 |
//! | test_BC_2_01_005_acquire_rejects_port_zero | — | F-S006-ADV1-004 | — |
//!
//! # Red Gate
//!
//! ALL tests in this file MUST FAIL against the stubs (unimplemented!() panics
//! in lifecycle::resolve_runtime_dir, lifecycle::ensure_runtime_dir,
//! lock::DaemonLock::acquire, lock::DaemonLock::release, auth::generate_session_token).
//! The tests that call these functions will panic; tests using macOS-specific
//! ProjectDirs mocking will fail at the unimplemented!() boundary.

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per naming convention.
#![allow(non_snake_case)]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use monocle_runtime::auth::generate_session_token;
use monocle_runtime::errors::DaemonStartError;
use monocle_runtime::lifecycle::{ensure_runtime_dir, resolve_runtime_dir};
use monocle_runtime::lock::DaemonLock;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Create an isolated temp directory and return it as a `TempDir` (drops = cleanup).
fn make_temp_runtime_dir() -> TempDir {
    tempfile::tempdir().expect("create temp runtime dir for test")
}

/// Write test-fixture JSON content to a path atomically via tempfile::NamedTempFile + persist.
///
/// This helper is required because the project-wide semgrep/clippy anti-pattern rule bans
/// `std::fs::write` even in test code (it is a blanket lint, not scoped to production code).
/// Using `NamedTempFile + persist` is also the pattern the production code must use, so
/// this helper exercises the same writeback path.
///
/// The file is persisted INTO the `dir` directory so that the persist rename is atomic
/// on the same filesystem (POSIX rename(2) atomicity guarantee).
fn write_test_fixture_to(dir: &std::path::Path, filename: &str, content: &str) {
    use std::io::Write;
    let mut tmp =
        tempfile::NamedTempFile::new_in(dir).expect("create NamedTempFile in test runtime dir");
    tmp.write_all(content.as_bytes())
        .expect("write test fixture content to NamedTempFile");
    let dest = dir.join(filename);
    tmp.persist(&dest)
        .expect("persist test fixture file atomically");
}

/// Write an empty marker file at `dir/filename` (used to simulate socket existence).
fn write_empty_marker(dir: &std::path::Path, filename: &str) {
    use std::io::Write;
    let mut tmp =
        tempfile::NamedTempFile::new_in(dir).expect("create NamedTempFile for empty marker");
    tmp.write_all(b"")
        .expect("write empty content to NamedTempFile");
    let dest = dir.join(filename);
    tmp.persist(&dest)
        .expect("persist empty marker file atomically");
}

/// A PID that should never exist. Using `i32::MAX - 1` avoids PID zero and PID 1.
/// The kernel wraps at a low value (default 32768 on Linux), so `i32::MAX - 1` is
/// safe as a "definitely dead" sentinel in tests. On macOS the max pid is 99998;
/// `i32::MAX - 1` still exceeds that and kill() will return ESRCH.
const DEAD_PID: i32 = i32::MAX - 1;

/// Test port — arbitrary high port unlikely to be in use during tests.
const TEST_PORT: u16 = 39_001;

// ---------------------------------------------------------------------------
// AC-001 — Clean start: lock file created atomically, mode 0o600
// ---------------------------------------------------------------------------

/// BC-2.01.005 PC-3: On a clean start (no pre-existing lock file), acquire() creates
/// `<runtime_dir>/monocle.lock` with mode 0o600 (owner-read-write only).
///
/// Counter-example guarded: a direct std::fs::write would not set the mode to 0o600
/// by default; tempfile::persist writes to a temp file first, but the final file mode
/// must also be 0o600, not the default umask-derived mode.
///
/// Traces to BC-2.01.005 PC-3, AC-001, VP-005.
#[test]
fn test_BC_2_01_005_clean_start_creates_lock_file() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let result = DaemonLock::acquire(runtime_dir, TEST_PORT);
    let (lock, _token) = result.expect(
        "DaemonLock::acquire on a clean runtime dir must succeed (BC-2.01.005 PC-3 clean start)",
    );

    let lock_path = runtime_dir.join("monocle.lock");
    assert!(
        lock_path.exists(),
        "monocle.lock must exist at <runtime_dir>/monocle.lock after acquire \
        (BC-2.01.005 PC-3); path: {}",
        lock_path.display()
    );

    // Keep lock alive until after the assertion.
    drop(lock);
}

/// BC-2.01.005 PC-3 + INV-3: Lock file mode must be exactly 0o600
/// (owner read+write, no group, no other).
///
/// Assertion pattern per S-006 story: `metadata.permissions().mode() & 0o777 == 0o600`
/// (NOT `metadata.mode() == 0o600` which includes file-type bits in the high bits).
///
/// Traces to BC-2.01.005 PC-3, INV-3, AC-001, VP-005.
#[test]
fn test_BC_2_01_005_lock_file_mode_is_0o600() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    let metadata = lock_path.metadata().expect("lock file must be stat-able");

    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "lock file mode must be 0o600 (owner-read-write only, no group/other bits set) \
        per BC-2.01.005 PC-3 / INV-3 / AC-001; \
        assertion: `metadata.permissions().mode() & 0o777 == 0o600`; \
        got mode 0o{mode:o}. \
        Counter-example: std::fs::write with default umask might produce 0o644."
    );

    drop(lock);
}

/// BC-2.01.005 PC-3: Lock file must be created at `<runtime_dir>/monocle.lock` exactly.
///
/// Traces to BC-2.01.005 PC-3, AC-001, VP-005.
#[test]
fn test_BC_2_01_005_lock_file_path_is_in_runtime_dir() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    // The canonical lock path.
    let expected_lock_path = runtime_dir.join("monocle.lock");
    assert!(
        expected_lock_path.exists(),
        "lock file must be at <runtime_dir>/monocle.lock per BC-2.01.005 PC-3; \
        expected: {}",
        expected_lock_path.display()
    );

    // Verify no lock file exists at other locations (e.g., current working dir).
    let cwd_lock = PathBuf::from("monocle.lock");
    assert!(
        !cwd_lock.exists(),
        "lock file must NOT be written to the current working directory — \
        it belongs at <runtime_dir>/monocle.lock per BC-2.01.005 PC-3"
    );

    drop(lock);
}

// ---------------------------------------------------------------------------
// AC-002 — JSON schema: 7 fields, correct types, correct order
// ---------------------------------------------------------------------------

/// BC-2.01.005 PC-4: Lock file JSON has exactly 7 fields with the correct types.
///
/// Schema per AC-002:
/// - contract_version: integer
/// - pid: integer
/// - port: integer
/// - authToken: string (64-hex)
/// - startTimeUtc: string (ISO 8601)
/// - app: string ("monocle")
/// - version: string (semver)
///
/// Traces to BC-2.01.005 PC-4, BC-2.01.010 PC-1, AC-002, VP-005.
#[test]
fn test_BC_2_01_005_json_has_7_fields_correct_types() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("lock file must be valid JSON");

    let obj = json
        .as_object()
        .expect("lock file JSON must be a JSON object (not array, not primitive)");

    assert_eq!(
        obj.len(),
        7,
        "lock file JSON must have EXACTLY 7 fields per BC-2.01.005 PC-4 / AC-002; \
        got {} field(s): {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );

    // contract_version: integer
    let cv = obj
        .get("contract_version")
        .expect("contract_version field must be present");
    assert!(
        cv.is_u64() || cv.is_i64(),
        "contract_version must be a JSON integer; got: {cv:?} (BC-2.01.005 PC-4 / AC-002)"
    );

    // pid: integer
    let pid_val = obj.get("pid").expect("pid field must be present");
    assert!(
        pid_val.is_i64() || pid_val.is_u64(),
        "pid must be a JSON integer; got: {pid_val:?} (BC-2.01.005 PC-4 / AC-002)"
    );

    // port: integer
    let port_val = obj.get("port").expect("port field must be present");
    assert!(
        port_val.is_u64(),
        "port must be a JSON non-negative integer; got: {port_val:?} (BC-2.01.005 PC-4 / AC-002)"
    );

    // authToken: string
    let auth_val = obj
        .get("authToken")
        .expect("authToken field must be present");
    assert!(
        auth_val.is_string(),
        "authToken must be a JSON string; got: {auth_val:?} (BC-2.01.005 PC-4 / AC-002)"
    );

    // startTimeUtc: string
    let ts_val = obj
        .get("startTimeUtc")
        .expect("startTimeUtc field must be present");
    assert!(
        ts_val.is_string(),
        "startTimeUtc must be a JSON string; got: {ts_val:?} (BC-2.01.005 PC-4 / AC-002)"
    );

    // app: string
    let app_val = obj.get("app").expect("app field must be present");
    assert!(
        app_val.is_string(),
        "app must be a JSON string; got: {app_val:?} (BC-2.01.005 PC-4 / AC-002)"
    );

    // version: string
    let ver_val = obj.get("version").expect("version field must be present");
    assert!(
        ver_val.is_string(),
        "version must be a JSON string; got: {ver_val:?} (BC-2.01.005 PC-4 / AC-002)"
    );

    drop(lock);
}

/// BC-2.01.005 PC-4 + BC-2.01.010 PC-2: `contract_version` must be the FIRST key in the
/// JSON object.
///
/// Key order is verified via raw string scanning (not by parsing, which may reorder).
/// The raw JSON text must have "contract_version" appearing before any other key names.
///
/// Traces to BC-2.01.005 PC-4, BC-2.01.010 PC-2, AC-002, AC-010, VP-005, VP-010.
#[test]
fn test_BC_2_01_005_json_field_contract_version_is_first() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");

    // Find positions of "contract_version" and every other field key in the raw JSON.
    let cv_pos = raw
        .find("\"contract_version\"")
        .expect("\"contract_version\" must appear in lock file JSON (BC-2.01.010 PC-2)");

    let other_keys = [
        "\"pid\"",
        "\"port\"",
        "\"authToken\"",
        "\"startTimeUtc\"",
        "\"app\"",
        "\"version\"",
    ];
    for key in other_keys {
        if let Some(key_pos) = raw.find(key) {
            assert!(
                cv_pos < key_pos,
                "\"contract_version\" must appear BEFORE \"{key}\" in the raw JSON text \
                (BC-2.01.005 PC-4 / BC-2.01.010 PC-2 / AC-002): contract_version at byte \
                {cv_pos}, {key} at byte {key_pos}. \
                Counter-example: HashMap-based serialization does NOT guarantee insertion order."
            );
        }
    }

    // Verify contract_version value is integer 1.
    let json: serde_json::Value = serde_json::from_str(&raw).expect("lock file must be valid JSON");
    let cv_val = json
        .get("contract_version")
        .and_then(|v| v.as_u64())
        .expect("contract_version must be a non-negative integer");
    assert_eq!(
        cv_val, 1,
        "contract_version must equal 1 for Phase 1 daemons (BC-2.01.010 PC-2 / AC-010); \
        got: {cv_val}"
    );

    drop(lock);
}

/// BC-2.01.005 PC-4: The `pid` field must equal the current process PID.
///
/// The daemon records the PID of the process that created the lock file.
///
/// Traces to BC-2.01.005 PC-4, AC-002, VP-005.
#[test]
fn test_BC_2_01_005_json_pid_field_is_current_process() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");

    let recorded_pid = json
        .get("pid")
        .and_then(|v| v.as_i64())
        .expect("pid field must be a JSON integer");

    let current_pid = i64::from(std::process::id());
    assert_eq!(
        recorded_pid, current_pid,
        "lock file pid field must equal the current process PID \
        (BC-2.01.005 PC-4 / AC-002); got pid={recorded_pid}, expected={current_pid}"
    );

    drop(lock);
}

/// BC-2.01.005 PC-4 + BC-2.01.010 PC-3: The `port` field must equal the port passed
/// to acquire(), and the `app` field must equal `"monocle"` exactly.
///
/// Traces to BC-2.01.005 PC-4, BC-2.01.010 PC-3, AC-002, VP-005, VP-010.
#[test]
fn test_BC_2_01_005_json_app_field_is_monocle() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");

    // port must match what was passed to acquire()
    let recorded_port = json
        .get("port")
        .and_then(|v| v.as_u64())
        .expect("port field must be a non-negative integer");
    assert_eq!(
        recorded_port,
        u64::from(TEST_PORT),
        "lock file port field must equal the port passed to DaemonLock::acquire \
        (BC-2.01.005 PC-4 / AC-002); got {recorded_port}, expected {TEST_PORT}"
    );

    // app must be "monocle" (BC-2.01.010 PC-3)
    let app_val = json
        .get("app")
        .and_then(|v| v.as_str())
        .expect("app field must be a JSON string");
    assert_eq!(
        app_val, "monocle",
        "app field must be the literal string \"monocle\" per BC-2.01.010 PC-3 / AC-002; \
        got: {app_val:?}. Future hook-discovery tooling filters by this field."
    );

    drop(lock);
}

/// BC-2.01.005 PC-4: The `startTimeUtc` field must be a valid ISO 8601 UTC timestamp
/// with mandatory millisecond precision (`YYYY-MM-DDTHH:MM:SS.sssZ`).
///
/// Traces to BC-2.01.005 PC-4, AC-002, VP-005.
#[test]
fn test_BC_2_01_005_json_start_time_is_iso8601() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");

    let ts = json
        .get("startTimeUtc")
        .and_then(|v| v.as_str())
        .expect("startTimeUtc must be a JSON string");

    // Mandatory ISO 8601 UTC with millisecond precision: YYYY-MM-DDTHH:MM:SS.sssZ
    let iso8601_ms_re = regex_lite::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$")
        .expect("valid regex");

    assert!(
        iso8601_ms_re.is_match(ts),
        "startTimeUtc must match ISO 8601 UTC with mandatory millisecond precision \
        `YYYY-MM-DDTHH:MM:SS.sssZ` per BC-2.01.005 PC-4 / AC-002 / \
        SS-daemon-lifecycle.md v1.0.33; got: {ts:?}. \
        Format: chrono `\"%Y-%m-%dT%H:%M:%S%.3fZ\"`."
    );

    assert!(
        ts.ends_with('Z'),
        "startTimeUtc must end with 'Z' (UTC mandatory, no offset); got: {ts:?}"
    );

    drop(lock);
}

// ---------------------------------------------------------------------------
// AC-003 — Live pid conflict → Err(LockFileConflict)
// ---------------------------------------------------------------------------

/// BC-2.01.005 PC-1: If a lock file exists at startup with a live PID, acquire() must
/// return Err(DaemonStartError::LockFileConflict { pid }).
///
/// We use the current process PID (std::process::id()) as a guaranteed-live PID —
/// the test process itself is running, so kill(pid, 0) will succeed.
///
/// Traces to BC-2.01.005 PC-1, AC-003, EC-003 (implicit), VP-005.
#[test]
fn test_BC_2_01_005_live_pid_conflict_returns_error() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    // Write a lock file recording the CURRENT process PID — guaranteed to be live.
    let current_pid = std::process::id();
    let fake_lock = serde_json::json!({
        "contract_version": 1,
        "pid": current_pid,
        "port": 9000,
        "authToken": "a".repeat(64),
        "startTimeUtc": "2026-05-25T10:00:00.000Z",
        "app": "monocle",
        "version": "0.1.0"
    });
    write_test_fixture_to(
        runtime_dir,
        "monocle.lock",
        &serde_json::to_string(&fake_lock).unwrap(),
    );

    let result = DaemonLock::acquire(runtime_dir, TEST_PORT);

    assert!(
        matches!(result, Err(DaemonStartError::LockFileConflict { pid }) if pid == current_pid as i32),
        "DaemonLock::acquire must return Err(LockFileConflict {{ pid }}) when a live process \
        holds the lock file (BC-2.01.005 PC-1 / AC-003 / E-LOCK-001); \
        got: {result:?}. The live PID was {current_pid}."
    );
}

// ---------------------------------------------------------------------------
// AC-004 — Stale lock (dead pid) → cleaned up, new lock acquired
// ---------------------------------------------------------------------------

/// BC-2.01.005 PC-2: If a lock file exists with a dead PID, the lock file is removed
/// before the new lock file is written.
///
/// We use DEAD_PID (`i32::MAX - 1`) as a PID that will never be alive on any system —
/// the kernel's PID space is far below i32::MAX.
///
/// Traces to BC-2.01.005 PC-2, AC-004, EC-004 (stale lock), VP-005.
#[test]
fn test_BC_2_01_005_stale_pid_cleaned_up() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    // Write a stale lock file with a dead PID.
    let fake_lock = serde_json::json!({
        "contract_version": 1,
        "pid": DEAD_PID,
        "port": 9000,
        "authToken": "b".repeat(64),
        "startTimeUtc": "2026-05-01T00:00:00.000Z",
        "app": "monocle",
        "version": "0.1.0"
    });
    let lock_path = runtime_dir.join("monocle.lock");
    write_test_fixture_to(
        runtime_dir,
        "monocle.lock",
        &serde_json::to_string(&fake_lock).unwrap(),
    );

    // Record the inode / content before acquire to detect it was replaced.
    let old_content = std::fs::read_to_string(&lock_path).expect("read old lock file");

    // acquire() must succeed, replacing the stale lock.
    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT).expect(
        "DaemonLock::acquire must succeed when the existing lock has a dead PID \
            (BC-2.01.005 PC-2 / AC-004 stale cleanup)",
    );

    // The new lock file must have different content (new pid, new token).
    let new_content =
        std::fs::read_to_string(&lock_path).expect("new lock file must exist after stale cleanup");

    assert_ne!(
        old_content, new_content,
        "Lock file content must change after stale cleanup — a new lock file \
        with the current PID and fresh token must have been written \
        (BC-2.01.005 PC-2 / AC-004)"
    );

    // The new pid in the file must NOT be the dead pid.
    let new_json: serde_json::Value = serde_json::from_str(&new_content).expect("valid JSON");
    let new_pid = new_json
        .get("pid")
        .and_then(|v| v.as_i64())
        .expect("pid field must be present");
    assert_ne!(
        new_pid,
        i64::from(DEAD_PID),
        "New lock file pid must not be the dead pid sentinel ({DEAD_PID}); got {new_pid}"
    );

    drop(lock);
}

/// BC-2.01.005 PC-2 + PC-3: After stale cleanup, a new valid lock file must be created
/// with the current PID and a fresh auth token.
///
/// Traces to BC-2.01.005 PC-2, PC-3, AC-004, VP-005.
#[test]
fn test_BC_2_01_005_stale_pid_new_lock_acquired() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let fake_lock = serde_json::json!({
        "contract_version": 1,
        "pid": DEAD_PID,
        "port": 9000,
        "authToken": "c".repeat(64),
        "startTimeUtc": "2026-01-01T00:00:00.000Z",
        "app": "monocle",
        "version": "0.1.0"
    });
    let lock_path = runtime_dir.join("monocle.lock");
    write_test_fixture_to(
        runtime_dir,
        "monocle.lock",
        &serde_json::to_string(&fake_lock).unwrap(),
    );

    let (lock, token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed after stale cleanup");

    // The returned token must be present.
    assert_eq!(
        token.len(),
        64,
        "auth token returned by acquire() must be 64 characters (BC-2.01.008 PC-1); \
        got {} chars: {token:?}",
        token.len()
    );

    // Lock file must exist and contain the new current-process PID.
    let new_content = std::fs::read_to_string(&lock_path)
        .expect("lock file must exist after stale cleanup + new acquire");
    let new_json: serde_json::Value = serde_json::from_str(&new_content).expect("valid JSON");
    let new_pid = new_json
        .get("pid")
        .and_then(|v| v.as_i64())
        .expect("pid must be present");
    let current_pid = i64::from(std::process::id());
    assert_eq!(
        new_pid, current_pid,
        "New lock file pid must be the current process PID ({current_pid}) after stale cleanup; \
        got {new_pid} (BC-2.01.005 PC-2+3 / AC-004)"
    );

    drop(lock);
}

// ---------------------------------------------------------------------------
// AC-005 — release() removes lock + sock files
// ---------------------------------------------------------------------------

/// BC-2.01.005 PC-6: On release(), the lock file `<runtime_dir>/monocle.lock` must be removed.
///
/// Traces to BC-2.01.005 PC-6, AC-005, VP-005.
#[test]
fn test_BC_2_01_005_release_removes_lock_file() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    assert!(lock_path.exists(), "lock file must exist before release()");

    lock.release().expect("release() must not return an error");

    assert!(
        !lock_path.exists(),
        "monocle.lock must NOT exist after DaemonLock::release() \
        (BC-2.01.005 PC-6 / AC-005); path: {}",
        lock_path.display()
    );
}

/// BC-2.01.005 PC-7: On release(), the socket file `<runtime_dir>/monocle.sock` must be removed.
///
/// We pre-create the sock file to simulate a running daemon that has bound its socket.
///
/// Traces to BC-2.01.005 PC-7, AC-005, VP-005.
#[test]
fn test_BC_2_01_005_release_removes_sock_file() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    // Simulate the daemon having created its socket file.
    let sock_path = runtime_dir.join("monocle.sock");
    write_empty_marker(runtime_dir, "monocle.sock");
    assert!(sock_path.exists(), "sock file must exist before release()");

    lock.release().expect("release() must not return an error");

    assert!(
        !sock_path.exists(),
        "monocle.sock must NOT exist after DaemonLock::release() \
        (BC-2.01.005 PC-7 / AC-005); path: {}",
        sock_path.display()
    );
}

// ---------------------------------------------------------------------------
// AC-006 — Runtime dir created with mode 0o700
// ---------------------------------------------------------------------------

/// BC-2.01.005 PC-8: If `<runtime_dir>` does not exist at start, ensure_runtime_dir()
/// creates it with mode 0o700 (owner-only access).
///
/// Assertion: `dir.metadata().permissions().mode() & 0o777 == 0o700`.
/// (NOT metadata.mode() == 0o700, which includes file-type bits.)
///
/// Traces to BC-2.01.005 PC-8, AC-006, NFR-012, VP-005.
#[test]
fn test_BC_2_01_005_runtime_dir_created_with_0o700() {
    // Use a unique subdirectory that does NOT yet exist.
    let base = tempfile::tempdir().expect("create outer temp dir");
    let new_dir = base.path().join("monocle-runtime-test");
    assert!(
        !new_dir.exists(),
        "test pre-condition: runtime dir must NOT exist before ensure_runtime_dir"
    );

    ensure_runtime_dir(&new_dir).expect("ensure_runtime_dir must succeed creating a new directory");

    assert!(
        new_dir.exists(),
        "runtime dir must exist after ensure_runtime_dir"
    );
    assert!(new_dir.is_dir(), "runtime dir path must be a directory");

    let metadata = new_dir.metadata().expect("stat runtime dir");
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o700,
        "runtime dir must have mode 0o700 (owner-only: rwx------) per \
        BC-2.01.005 PC-8 / AC-006 / NFR-012; \
        assertion: `metadata.permissions().mode() & 0o777 == 0o700`; \
        got mode 0o{mode:o}. \
        Counter-example: std::fs::create_dir_all honors umask (typically 0o755)."
    );
}

/// BC-2.01.005 PC-8: ensure_runtime_dir creates intermediate directories (recursive)
/// and all created segments must be accessible (not necessarily all 0o700 internally,
/// but the final leaf must be 0o700).
///
/// Traces to BC-2.01.005 PC-8, AC-006, VP-005.
#[test]
fn test_BC_2_01_005_runtime_dir_created_recursively() {
    let base = tempfile::tempdir().expect("create outer temp dir");
    let nested = base.path().join("a").join("b").join("c");
    assert!(
        !nested.exists(),
        "test pre-condition: nested dir must not exist"
    );

    ensure_runtime_dir(&nested)
        .expect("ensure_runtime_dir must succeed creating nested directories");

    assert!(
        nested.exists(),
        "nested runtime dir must exist after ensure_runtime_dir"
    );

    let mode = nested
        .metadata()
        .expect("stat nested dir")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o700,
        "leaf runtime dir must have mode 0o700 even when created recursively; \
        got mode 0o{mode:o}"
    );
}

// ---------------------------------------------------------------------------
// AC-007 — MONOCLE_RUNTIME_DIR env override
// ---------------------------------------------------------------------------

/// BC-2.01.005 PC-2a + EC-058: When `MONOCLE_RUNTIME_DIR` is set to a non-empty path,
/// resolve_runtime_dir() returns that path verbatim.
///
/// Traces to BC-2.01.005 PC-2a, EC-058, AC-007, VP-005.
#[test]
fn test_BC_2_01_005_env_override_monocle_runtime_dir() {
    let tmp = make_temp_runtime_dir();
    let expected = tmp.path().to_path_buf();
    let expected_str = expected.to_str().expect("temp path is valid UTF-8");

    temp_env::with_vars([("MONOCLE_RUNTIME_DIR", Some(expected_str))], || {
        let resolved = resolve_runtime_dir()
            .expect("resolve_runtime_dir must succeed when MONOCLE_RUNTIME_DIR is set");
        assert_eq!(
            resolved, expected,
            "resolve_runtime_dir must return the MONOCLE_RUNTIME_DIR path verbatim \
                (BC-2.01.005 PC-2a / EC-058 / AC-007); \
                got: {resolved:?}, expected: {expected:?}"
        );
    });
}

/// BC-2.01.005 EC-060: When `MONOCLE_RUNTIME_DIR` is set to an empty string, it must
/// be treated as unset — the daemon falls through to platform-default resolution.
///
/// Empty-string check: `if set and non-empty` (BC-2.01.005 PC-2a).
/// The resolved path must NOT be an empty PathBuf and must NOT equal "".
///
/// Traces to BC-2.01.005 EC-060, AC-007 (negative), VP-005.
#[test]
fn test_BC_2_01_005_env_override_empty_string_falls_through() {
    temp_env::with_vars([("MONOCLE_RUNTIME_DIR", Some(""))], || {
        // With empty MONOCLE_RUNTIME_DIR, resolve_runtime_dir falls through to
        // platform-default (ProjectDirs). On a CI system where HOME is set this
        // returns Ok with a platform path; on a system where HOME is unset it may
        // return Err(RuntimeDirUnresolvable). Either is acceptable — what is NOT
        // acceptable is returning the empty path "" itself.
        match resolve_runtime_dir() {
            Ok(path) => {
                assert!(
                    !path.as_os_str().is_empty(),
                    "resolve_runtime_dir must NOT return an empty path when \
                        MONOCLE_RUNTIME_DIR=\"\" — it must treat empty as unset and \
                        fall through to platform-default resolution \
                        (BC-2.01.005 EC-060 / AC-007); got empty path"
                );
            }
            Err(DaemonStartError::RuntimeDirUnresolvable) => {
                // Acceptable: CI environment without HOME set falls through to
                // RuntimeDirUnresolvable after the empty-string bypass. This is the
                // correct behavior — NOT returning a path of "".
            }
            Err(other) => {
                panic!(
                    "resolve_runtime_dir with MONOCLE_RUNTIME_DIR=\"\" must return \
                        Ok(platform_path) or Err(RuntimeDirUnresolvable); \
                        got unexpected error: {other:?}"
                );
            }
        }
    });
}

// ---------------------------------------------------------------------------
// AC-009 — RuntimeDirUnresolvable when HOME is absent
// ---------------------------------------------------------------------------

/// BC-2.01.005 PC-2d + EC-059: When both MONOCLE_RUNTIME_DIR and HOME are unset,
/// resolve_runtime_dir() must return Err(DaemonStartError::RuntimeDirUnresolvable).
///
/// We clear HOME (and XDG_RUNTIME_DIR, XDG_DATA_HOME) to force ProjectDirs::new() to
/// return None.
///
/// Traces to BC-2.01.005 PC-2d, EC-059, AC-009, VP-005.
#[test]
fn test_BC_2_01_005_runtimedirunresolvable_when_no_home() {
    temp_env::with_vars(
        [
            ("MONOCLE_RUNTIME_DIR", None::<&str>),
            ("HOME", None::<&str>),
            ("XDG_RUNTIME_DIR", None::<&str>),
            ("XDG_DATA_HOME", None::<&str>),
            // macOS: also clear USERPROFILE in case of fallback
            ("USERPROFILE", None::<&str>),
        ],
        || {
            let result = resolve_runtime_dir();
            assert!(
                matches!(result, Err(DaemonStartError::RuntimeDirUnresolvable)),
                "resolve_runtime_dir must return Err(RuntimeDirUnresolvable) when both \
                MONOCLE_RUNTIME_DIR and HOME/XDG paths are unset \
                (BC-2.01.005 PC-2d / EC-059 / AC-009 / E-LOCK-003); got: {result:?}. \
                The daemon must fail fast with an actionable error message, not panic."
            );
        },
    );
}

// ---------------------------------------------------------------------------
// AC-014 — authToken is 64-hex, matches /^[0-9a-f]{64}$/
// ---------------------------------------------------------------------------

/// BC-2.01.008 PC-1: The `authToken` field in the lock file must be exactly 64 lowercase
/// hex characters, matching `/^[0-9a-f]{64}$/`.
///
/// Traces to BC-2.01.008 PC-1, AC-014, VP-005.
#[test]
fn test_BC_2_01_008_auth_token_is_64_hex() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");

    let auth_token = json
        .get("authToken")
        .and_then(|v| v.as_str())
        .expect("authToken field must be a JSON string");

    assert_eq!(
        auth_token.len(),
        64,
        "authToken must be exactly 64 characters long (BC-2.01.008 PC-1 / AC-014); \
        got {} chars: {auth_token:?}",
        auth_token.len()
    );

    drop(lock);
}

/// BC-2.01.008 PC-1: The `authToken` field must match `/^[0-9a-f]{64}$/` — lowercase hex only.
///
/// Traces to BC-2.01.008 PC-1, AC-014, VP-005.
#[test]
fn test_BC_2_01_008_auth_token_matches_regex() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let (lock, _token) = DaemonLock::acquire(runtime_dir, TEST_PORT)
        .expect("acquire must succeed on clean runtime dir");

    let lock_path = runtime_dir.join("monocle.lock");
    let raw = std::fs::read_to_string(&lock_path).expect("lock file must be readable");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");

    let auth_token = json
        .get("authToken")
        .and_then(|v| v.as_str())
        .expect("authToken must be a JSON string");

    let hex_re = regex_lite::Regex::new(r"^[0-9a-f]{64}$").expect("valid hex regex");

    assert!(
        hex_re.is_match(auth_token),
        "authToken must match /^[0-9a-f]{{64}}$/ (BC-2.01.008 PC-1 / AC-014); \
        got: {auth_token:?}. \
        The token must be 32 bytes from OsRng, lowercase hex-encoded. \
        No uppercase letters, no prefix, no suffix."
    );

    // Verify returned token from acquire() matches the file content.
    // (Both should be the same value written at acquire time.)
    let (lock2, returned_token) = {
        let tmp2 = make_temp_runtime_dir();
        DaemonLock::acquire(tmp2.path(), TEST_PORT)
            .map(|(l, t)| {
                let path = tmp2.path().join("monocle.lock");
                let raw2 = std::fs::read_to_string(&path).expect("read");
                let j: serde_json::Value = serde_json::from_str(&raw2).expect("valid");
                let file_token = j
                    .get("authToken")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();
                assert_eq!(
                    t, file_token,
                    "token returned by acquire() must match authToken written to lock file \
                    (BC-2.01.008 PC-1 / AC-014)"
                );
                (l, t)
            })
            .expect("second acquire must succeed")
    };
    drop(lock2);
    drop(returned_token);

    drop(lock);
}

/// BC-2.01.008 PC-1: generate_session_token() must produce a 64-char lowercase hex string.
///
/// The token is 32 bytes from OsRng, hex-encoded.
///
/// Traces to BC-2.01.008 PC-1, AC-014, VP-005.
#[test]
fn test_BC_2_01_008_generate_session_token_format() {
    let token = generate_session_token();

    assert_eq!(
        token.len(), 64,
        "generate_session_token() must produce a 64-character string (32 bytes × 2 hex chars/byte) \
        per BC-2.01.008 PC-1 / AC-014; got {} chars: {token:?}",
        token.len()
    );

    let hex_re = regex_lite::Regex::new(r"^[0-9a-f]{64}$").expect("valid hex regex");

    assert!(
        hex_re.is_match(&token),
        "generate_session_token() output must match /^[0-9a-f]{{64}}$/ \
        (lowercase hex only, no prefix, no suffix) per BC-2.01.008 PC-1 / AC-014; \
        got: {token:?}. The entropy source must be OsRng (not thread_rng)."
    );
}

/// BC-2.01.008 PC-1 + BC-2.01.008 INV-3: generate_session_token() must produce different
/// values on each call (randomness invariant).
///
/// Statistical confidence: two 256-bit values collide with probability 2^-256.
/// If they collide, the CSPRNG is broken, which is a catastrophic failure.
///
/// Traces to BC-2.01.008 PC-1, INV-3, AC-014, VP-005.
#[test]
fn test_BC_2_01_008_generate_session_token_is_random() {
    let token_a = generate_session_token();
    let token_b = generate_session_token();

    assert_ne!(
        token_a, token_b,
        "generate_session_token() must produce unique tokens on each call — \
        two identical 64-hex tokens indicates the CSPRNG is broken or the function \
        is returning a constant (BC-2.01.008 INV-3 / AC-014). \
        token_a: {token_a:?}"
    );
}

// ---------------------------------------------------------------------------
// Port range validation (F-S006-ADV1-004)
// ---------------------------------------------------------------------------

/// acquire() must reject port 0 with LockFileWriteFailure(InvalidInput).
///
/// Ports 0–1023 are reserved/privileged. Accepting port 0 would produce a lock
/// file with an unusable port value. The valid range is 1024–65535.
///
/// Traces to F-S006-ADV1-004 (adversary finding Pass 1).
#[test]
fn test_BC_2_01_005_acquire_rejects_port_zero() {
    let tmp = make_temp_runtime_dir();
    let runtime_dir = tmp.path();

    let result = DaemonLock::acquire(runtime_dir, 0);

    match result {
        Err(DaemonStartError::LockFileWriteFailure(io_err)) => {
            assert_eq!(
                io_err.kind(),
                std::io::ErrorKind::InvalidInput,
                "port 0 rejection must use ErrorKind::InvalidInput; got: {:?}",
                io_err.kind()
            );
        }
        other => panic!(
            "expected Err(LockFileWriteFailure(InvalidInput)) for port=0, got: {:?}",
            other
        ),
    }
}
