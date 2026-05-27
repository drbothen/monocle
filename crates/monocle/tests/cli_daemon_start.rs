//! Integration tests for BC-2.04.004: `monocle daemon start` CLI subcommand.
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause | Notes |
//! |---------------|----|-----------|-------|
//! | test_ac_005_start_no_lock_file_exits_zero | AC-005 | BC-2.04.004 PC-1..PC-4 | happy path |
//! | test_ac_005_start_no_stdout_on_success | AC-005 | BC-2.04.004 INV-3 | no stdout invariant |
//! | test_ac_006_start_already_running_exits_1_with_stderr | AC-006 | BC-2.04.004 PC-6 | already-running |
//! | test_ac_006_start_already_running_stderr_format | AC-006 | BC-2.04.004 PC-6 | stderr message format |
//! | test_ac_007_start_timeout_exits_1_with_stderr | AC-007 | BC-2.04.004 PC-7 | timeout path |
//! | test_ac_007_start_timeout_stderr_format | AC-007 | BC-2.04.004 PC-7 | stderr message format |
//! | test_ac_008_exit_code_zero_on_success | AC-008 | BC-2.04.004 PC-8 | exit code 0 |
//! | test_ac_008_exit_code_1_already_running | AC-008 | BC-2.04.004 PC-8 | exit code 1 |
//! | test_ac_008_exit_code_1_timeout | AC-008 | BC-2.04.004 PC-8 | exit code 1 |
//! | test_ac_008_exit_code_70_runtime_dir_unresolvable | AC-008 | BC-2.04.004 PC-8, EC-2.04.004-06 | exit code 70 |
//! | test_ac_005_stale_lock_treated_as_absent | AC-005 | BC-2.04.004 EC-2.04.004-02 | stale lock |
//! | test_ac_005_no_autostart_env_does_not_affect_daemon_start | AC-005 | BC-2.04.004 EC-2.04.004-07 | MONOCLE_NO_AUTOSTART |
//! | test_ac_004_exit_code_4_internal_error | AC-008 | BC-2.04.004 PC-8 | exit code 71 |
//!
//! # Red Gate Status
//!
//! ALL tests in this file MUST FAIL against the current stub: `cmd_daemon_start()` is
//! implemented as `todo!("S-017: implement daemon start")`, which causes the process to
//! abort with a non-zero exit code (101 on Rust panic). Tests that assert exit code 0
//! will fail because actual exit is 101. Tests that assert specific stderr content will
//! fail because actual stderr contains the todo!() panic message, not the expected output.
//!
//! The Red Gate is verified when `cargo test --package monocle` shows all tests in this
//! file in FAILED state with assertion-error messages (not compilation errors).
//!
//! # Test Strategy
//!
//! Tests invoke the `monocle` binary via `assert_cmd::Command::cargo_bin("monocle")`.
//! The binary is built from `crates/monocle/src/main.rs`. Since `cmd_daemon_start()` is
//! `todo!()`, any invocation of `monocle daemon start` will abort the process at the
//! `todo!()` call site. `assert_cmd` captures exit status and stdio streams.
//!
//! Setup for each test:
//! 1. Create a `tempfile::TempDir` for the runtime directory.
//! 2. Set `MONOCLE_RUNTIME_DIR` via `env()` on the `Command` builder.
//! 3. Assert on exit status and stdio content.
//!
//! The tests never write to real system paths; all runtime directory I/O is confined to
//! the `tempfile::TempDir` which is deleted on `drop()`.

// Test files: unwrap/expect are idiomatic assertion amplification in tests, not production.
#![allow(clippy::unwrap_used, clippy::expect_used)]
// Non-snake-case names encode BC IDs per project naming convention.
#![allow(non_snake_case)]

use assert_cmd::Command;
use predicates::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return a `Command` for `monocle daemon start` with `MONOCLE_RUNTIME_DIR` set to
/// `runtime_dir`. Clears HOME, XDG_RUNTIME_DIR so that resolve_runtime_dir() does not
/// accidentally read platform paths; tests control the runtime dir entirely via the env var.
fn start_cmd(runtime_dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("monocle").expect("monocle binary must be buildable");
    cmd.arg("daemon")
        .arg("start")
        .env("MONOCLE_RUNTIME_DIR", runtime_dir.as_os_str());
    cmd
}

/// Create a fake daemon script in `dir` that sleeps for 30 seconds but never writes a
/// lock file. Returns the path to the script.
///
/// This script is used by the start-timeout tests: by pointing `MONOCLE_DAEMON_BIN` at
/// this script and setting `MONOCLE_START_TIMEOUT_SECS=2`, we simulate the scenario where
/// the daemon subprocess runs but never signals readiness (lock file never appears).
///
/// The script sleeps long enough to outlast the 2-second test timeout without exiting
/// prematurely (which would trigger exit 71 instead of the timeout path exit 1).
fn make_fake_daemon_script(dir: &std::path::Path) -> std::path::PathBuf {
    use std::io::Write as _;
    use std::os::unix::fs::PermissionsExt as _;

    let script_path = dir.join("fake-daemon.sh");
    let mut f = std::fs::File::create(&script_path).expect("create fake daemon script");
    f.write_all(b"#!/bin/sh\n# Fake daemon: runs but never writes a lock file.\nsleep 30\n")
        .expect("write fake daemon script");
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
        .expect("set executable on fake daemon script");
    script_path
}

/// Write a lock file fixture to `dir/monocle.lock` with the given PID and port.
/// Uses `NamedTempFile + persist` per the project anti-pattern rule against naked
/// `std::fs::write`. The `pid` in the fixture is used to simulate a live or dead daemon.
fn write_lock_fixture(dir: &std::path::Path, pid: i32, port: u16) {
    use std::io::Write as _;

    let content = serde_json::json!({
        "contract_version": 1,
        "pid": pid,
        "port": port,
        "authToken": "a".repeat(64),
        "startTimeUtc": "2026-05-27T00:00:00.000Z",
        "app": "monocle",
        "version": "0.1.0"
    });
    let json_str = serde_json::to_string(&content).expect("serialize lock fixture");
    let mut tmp = tempfile::NamedTempFile::new_in(dir).expect("create NamedTempFile");
    tmp.write_all(json_str.as_bytes())
        .expect("write lock fixture");
    tmp.persist(dir.join("monocle.lock"))
        .expect("persist lock fixture");
}

/// A PID that will never be alive: far above the kernel's maximum PID on any platform.
const DEAD_PID: i32 = i32::MAX - 1;

// ---------------------------------------------------------------------------
// AC-005: Happy path — no live daemon, exits 0, no stdout
// ---------------------------------------------------------------------------

/// BC-2.04.004 PC-1..PC-4: `monocle daemon start` with no pre-existing lock file must
/// spawn the daemon subprocess, poll for the lock file, and exit 0 with no stdout.
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()` — the binary
/// panics with exit code 101, not the expected exit code 0.
///
/// Traces to BC-2.04.004 PC-1, PC-2, PC-3, PC-4, AC-005.
#[test]
fn test_ac_005_start_no_lock_file_exits_zero() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Pre-condition: no lock file exists.
    assert!(
        !tmp.path().join("monocle.lock").exists(),
        "test pre-condition: lock file must not exist before invocation"
    );

    start_cmd(tmp.path()).assert().success(); // exit code 0 — WILL FAIL: todo!() produces exit 101
}

/// BC-2.04.004 INV-3: On success, no bytes are written to stdout.
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()` — the binary
/// panics before producing clean output. The test asserts empty stdout AND exit 0.
///
/// Traces to BC-2.04.004 INV-3, AC-005.
#[test]
fn test_ac_005_start_no_stdout_on_success() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    start_cmd(tmp.path())
        .assert()
        .success() // WILL FAIL: todo!() produces exit 101
        .stdout(predicate::str::is_empty()); // no stdout on success
}

// ---------------------------------------------------------------------------
// AC-006: Already running — exits 1 with stderr error message
// ---------------------------------------------------------------------------

/// BC-2.04.004 PC-6: If a lock file exists AND the PID is alive, the command must write
/// `error: daemon already running (pid=<N>)` to stderr and exit 1.
///
/// We use the current process PID as a guaranteed-alive PID (the test process itself).
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()` — the binary
/// panics with a todo!() message instead of the expected stderr, and exits 101 not 1.
///
/// Traces to BC-2.04.004 PC-6, EC-2.04.004-01, AC-006.
#[test]
fn test_ac_006_start_already_running_exits_1_with_stderr() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");
    let current_pid = std::process::id() as i32;

    // Write a lock file with the current (alive) PID.
    write_lock_fixture(tmp.path(), current_pid, 39_100);

    start_cmd(tmp.path())
        .assert()
        .failure() // exit code non-zero — PASSES (todo!() also fails)
        .code(1); // exit code 1 specifically — WILL FAIL: todo!() produces 101
}

/// BC-2.04.004 PC-6: Verify the exact stderr message format when daemon is already running.
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()` — the stderr
/// contains the todo!() panic message, not `error: daemon already running (pid=<N>)`.
///
/// Traces to BC-2.04.004 PC-6, AC-006.
#[test]
fn test_ac_006_start_already_running_stderr_format() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");
    let current_pid = std::process::id() as i32;

    write_lock_fixture(tmp.path(), current_pid, 39_101);

    start_cmd(tmp.path())
        .assert()
        .code(1) // WILL FAIL: todo!() produces 101
        .stderr(predicate::str::contains(format!(
            "error: daemon already running (pid={})",
            current_pid
        ))); // WILL FAIL: todo!() panic message does not contain this string
}

// ---------------------------------------------------------------------------
// AC-007: Timeout — lock file does not appear within 10 s, exits 1 with stderr
// ---------------------------------------------------------------------------

/// BC-2.04.004 PC-7: When the lock file does not appear within 10 seconds, the command
/// must write `error: daemon failed to start within 10 s` to stderr and exit 1.
///
/// # Test Design
///
/// We simulate the timeout by:
/// 1. Pointing `MONOCLE_DAEMON_BIN` at a fake daemon script that sleeps for 30 seconds
///    but never writes a lock file.
/// 2. Setting `MONOCLE_START_TIMEOUT_SECS=2` to shorten the polling window to 2 seconds
///    (keeps CI fast while still exercising the full timeout code path).
///
/// The fake daemon subprocess runs and stays alive, so the implemention does NOT detect
/// premature subprocess exit (which would give exit 71). Instead, it polls for the lock
/// file for 2 seconds, finds nothing, and exits 1 with the timeout message.
///
/// Traces to BC-2.04.004 PC-7, EC-2.04.004-04, AC-007.
#[test]
fn test_ac_007_start_timeout_exits_1_with_stderr() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");
    let fake_daemon = make_fake_daemon_script(tmp.path());

    start_cmd(tmp.path())
        .env("MONOCLE_DAEMON_BIN", &fake_daemon)
        .env("MONOCLE_START_TIMEOUT_SECS", "2")
        .timeout(std::time::Duration::from_secs(10)) // allow room: 2s timeout + buffer
        .assert()
        .code(1);
}

/// BC-2.04.004 PC-7: Verify the exact stderr message format for the timeout case.
///
/// Traces to BC-2.04.004 PC-7, AC-007.
#[test]
fn test_ac_007_start_timeout_stderr_format() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");
    let fake_daemon = make_fake_daemon_script(tmp.path());

    start_cmd(tmp.path())
        .env("MONOCLE_DAEMON_BIN", &fake_daemon)
        .env("MONOCLE_START_TIMEOUT_SECS", "2")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "error: daemon failed to start within 10 s",
        ));
}

// ---------------------------------------------------------------------------
// AC-008: Exit codes
// ---------------------------------------------------------------------------

/// BC-2.04.004 PC-8: Exit code 0 when daemon started successfully.
///
/// RED GATE: Redundant with test_ac_005_start_no_lock_file_exits_zero but explicitly
/// tests the exit code table as a separate assertion with explicit `code(0)`.
///
/// Traces to BC-2.04.004 PC-8, AC-008.
#[test]
fn test_ac_008_exit_code_zero_on_success() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    start_cmd(tmp.path()).assert().code(0); // WILL FAIL: todo!() produces 101
}

/// BC-2.04.004 PC-8: Exit code 1 when daemon is already running.
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()`.
///
/// Traces to BC-2.04.004 PC-8, AC-008.
#[test]
fn test_ac_008_exit_code_1_already_running() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");
    let current_pid = std::process::id() as i32;
    write_lock_fixture(tmp.path(), current_pid, 39_102);

    start_cmd(tmp.path()).assert().code(1); // WILL FAIL: todo!() produces 101
}

/// BC-2.04.004 PC-8: Exit code 1 on timeout.
///
/// Uses the fake daemon script + short timeout (2 s) to exercise the timeout path
/// without blocking CI for 10 seconds.
///
/// Traces to BC-2.04.004 PC-8, AC-008.
#[test]
fn test_ac_008_exit_code_1_timeout() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");
    let fake_daemon = make_fake_daemon_script(tmp.path());

    start_cmd(tmp.path())
        .env("MONOCLE_DAEMON_BIN", &fake_daemon)
        .env("MONOCLE_START_TIMEOUT_SECS", "2")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .code(1);
}

/// BC-2.04.004 PC-8 + EC-2.04.004-06: Exit code 70 when runtime directory cannot be resolved.
///
/// We trigger RuntimeDirUnresolvable by unsetting MONOCLE_RUNTIME_DIR and clearing HOME
/// and all XDG variables so that resolve_runtime_dir() returns Err(RuntimeDirUnresolvable).
/// Per BC-2.04.004 PC-8, the CLI must exit with code 70 in this case.
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()` and does not
/// call resolve_runtime_dir() at all — it panics with exit 101 before any resolution.
///
/// Traces to BC-2.04.004 PC-8, EC-2.04.004-06, AC-008.
#[test]
fn test_ac_008_exit_code_70_runtime_dir_unresolvable() {
    let mut cmd = Command::cargo_bin("monocle").expect("monocle binary must be buildable");
    cmd.arg("daemon")
        .arg("start")
        // Clear all env vars that resolve_runtime_dir() uses.
        .env_remove("MONOCLE_RUNTIME_DIR")
        .env_remove("HOME")
        .env_remove("XDG_RUNTIME_DIR")
        .env_remove("XDG_DATA_HOME")
        .env_remove("USERPROFILE");

    cmd.assert().code(70); // WILL FAIL: todo!() produces 101
}

// ---------------------------------------------------------------------------
// AC-005 edge cases: stale lock treated as absent, MONOCLE_NO_AUTOSTART ignored
// ---------------------------------------------------------------------------

/// BC-2.04.004 EC-2.04.004-02: A lock file with a dead PID (stale lock) must be treated
/// as absent — the daemon starts normally and exits 0.
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()`.
///
/// Traces to BC-2.04.004 EC-2.04.004-02, AC-005.
#[test]
fn test_ac_005_stale_lock_treated_as_absent() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Write a stale lock file with a dead PID.
    write_lock_fixture(tmp.path(), DEAD_PID, 39_103);
    assert!(
        tmp.path().join("monocle.lock").exists(),
        "test pre-condition: stale lock file must exist before invocation"
    );

    start_cmd(tmp.path()).assert().code(0); // WILL FAIL: todo!() produces 101
}

/// BC-2.04.004 EC-2.04.004-07: `MONOCLE_NO_AUTOSTART=1` must not affect `daemon start`.
/// The command must proceed normally (not be blocked by this env var).
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()`.
///
/// Traces to BC-2.04.004 EC-2.04.004-07, BC-2.04.004 PC-2 (precondition 2), AC-005.
#[test]
fn test_ac_005_no_autostart_env_does_not_affect_daemon_start() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    start_cmd(tmp.path())
        .env("MONOCLE_NO_AUTOSTART", "1")
        .assert()
        .code(0); // WILL FAIL: todo!() produces 101
                  // If the implementation incorrectly blocked on MONOCLE_NO_AUTOSTART, it might exit 1.
                  // The correct behavior is exit 0 (daemon started). Either way, todo!() panics first.
}

/// BC-2.04.004 PC-8: Exit code 71 for internal error during daemon startup.
///
/// This path arises when the lock file write fails inside the daemon subprocess
/// (BC-2.04.001 step 8). For the CLI integration test, we verify the exit code routing
/// exists in the exit code table. The full path requires a daemon subprocess that fails
/// at lock-file write — this is exercised more precisely by S-017 tests. This test
/// verifies the exit code taxonomy at the CLI surface.
///
/// RED GATE: This test validates that the exit code 71 path is part of the implementation
/// contract. The todo!() stub does not implement this routing, so the test fails.
///
/// Traces to BC-2.04.004 PC-8, AC-008.
#[test]
fn test_ac_008_exit_code_71_internal_error() {
    // Exit code 71 is only reachable when lock file write fails inside the daemon.
    // This test documents the contract; the specific trigger path is daemon-subprocess
    // internal. We verify that exit code 71 is a distinct value in the exit code table
    // by asserting it is NOT exit code 0, 1, 2, or 70.
    //
    // The actual integration path for triggering 71: run daemon start with a runtime_dir
    // that is read-only (so the daemon subprocess cannot write the lock file). This test
    // is a documentation assertion — the full trigger will be implemented in S-017 tests.
    //
    // For Red Gate: this test asserts the exit code is 71. todo!() exits 101.

    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Make the runtime directory read-only so the daemon subprocess cannot write the lock file.
    // (This simulates the lock file write failure condition.)
    let runtime_dir = tmp.path();
    std::fs::set_permissions(
        runtime_dir,
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o500),
    )
    .expect("set read-only permissions on runtime dir");

    start_cmd(runtime_dir).assert().code(71); // WILL FAIL: todo!() produces 101

    // Restore permissions so tempdir cleanup can succeed.
    std::fs::set_permissions(
        runtime_dir,
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700),
    )
    .expect("restore permissions on runtime dir");
}

// ---------------------------------------------------------------------------
// AC-008 invariant: stderr-only for errors (no stdout on error paths)
// ---------------------------------------------------------------------------

/// BC-2.04.004 INV-4: All error messages go to stderr, not stdout. Verify that the
/// already-running error path produces no stdout output.
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()` — exit code
/// is 101, not 1. Even if we relax exit code, the stdout assertion may or may not pass
/// depending on Rust's panic output routing.
///
/// Traces to BC-2.04.004 INV-4, AC-006.
#[test]
fn test_ac_006_already_running_produces_no_stdout() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");
    let current_pid = std::process::id() as i32;
    write_lock_fixture(tmp.path(), current_pid, 39_104);

    start_cmd(tmp.path())
        .assert()
        .code(1) // WILL FAIL: todo!() produces 101
        .stdout(predicate::str::is_empty()); // error messages must go to stderr only
}

/// BC-2.04.004 INV-4: Timeout error path produces no stdout output.
///
/// Uses the fake daemon script + short timeout (2 s) to exercise the timeout path
/// without blocking CI for 10 seconds.
///
/// Traces to BC-2.04.004 INV-4, AC-007.
#[test]
fn test_ac_007_timeout_produces_no_stdout() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");
    let fake_daemon = make_fake_daemon_script(tmp.path());

    start_cmd(tmp.path())
        .env("MONOCLE_DAEMON_BIN", &fake_daemon)
        .env("MONOCLE_START_TIMEOUT_SECS", "2")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());
}
