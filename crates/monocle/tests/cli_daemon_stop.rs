//! Integration tests for BC-2.04.005: `monocle daemon stop` CLI subcommand.
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause | Notes |
//! |---------------|----|-----------|-------|
//! | test_ac_009_stop_running_daemon_exits_zero | AC-009 | BC-2.04.005 PC-1..PC-4 | happy path |
//! | test_ac_009_stop_no_stdout_on_success | AC-009 | BC-2.04.005 INV-3 | no stdout invariant |
//! | test_ac_009_stop_no_stderr_on_success | AC-009 | BC-2.04.005 INV-4 | no stderr on success |
//! | test_ac_010_stop_no_lock_file_exits_1_with_stderr | AC-010 | BC-2.04.005 PC-5 | absent lock file |
//! | test_ac_010_stop_no_lock_file_stderr_format | AC-010 | BC-2.04.005 PC-5 | stderr message format |
//! | test_ac_011_stop_stale_lock_exits_1_with_stderr | AC-011 | BC-2.04.005 PC-6 | stale lock |
//! | test_ac_011_stop_stale_lock_stderr_format | AC-011 | BC-2.04.005 PC-6 | stderr message format |
//! | test_ac_012_stop_timeout_exits_2_with_stderr | AC-012 | BC-2.04.005 PC-7 | timeout |
//! | test_ac_012_stop_timeout_stderr_format | AC-012 | BC-2.04.005 PC-7 | stderr message format |
//! | test_ac_013_no_sigkill_ever_sent | AC-013 | BC-2.04.005 INV-1 | SIGKILL invariant |
//! | test_ac_009_malformed_lock_file_exits_1 | AC-009 | BC-2.04.005 EC-2.04.005-06 | malformed lock |
//! | test_ac_009_malformed_lock_file_stderr_format | AC-009 | BC-2.04.005 EC-2.04.005-06 | malformed lock stderr |
//! | test_ac_008_exit_code_70_runtime_dir_unresolvable | AC-009 | BC-2.04.005 EC-2.04.005-07, PC-8 | exit code 70 |
//!
//! # Red Gate Status
//!
//! ALL tests in this file MUST FAIL against the current stub: `cmd_daemon_stop()` is
//! implemented as `todo!("S-018: implement daemon stop")`, which causes the process to
//! abort with a non-zero exit code (101 on Rust panic). Tests that assert exit code 0
//! will fail because actual exit is 101. Tests that assert specific stderr content will
//! fail because actual stderr contains the todo!() panic message.
//!
//! # Test Strategy
//!
//! Tests invoke the `monocle` binary via `assert_cmd::Command::cargo_bin("monocle")`.
//! Since `cmd_daemon_stop()` is `todo!()`, any invocation of `monocle daemon stop`
//! aborts the process at the `todo!()` call site. Tests assert on the expected
//! post-implementation behavior, which differs from the current panic behavior.

// Test files: unwrap/expect are idiomatic assertion amplification in tests, not production.
#![allow(clippy::unwrap_used, clippy::expect_used)]
// Non-snake-case names encode BC IDs per project naming convention.
#![allow(non_snake_case)]

use assert_cmd::Command;
use predicates::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return a `Command` for `monocle daemon stop` with `MONOCLE_RUNTIME_DIR` set to
/// `runtime_dir`. Provides a controlled runtime environment for each test.
fn stop_cmd(runtime_dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("monocle").expect("monocle binary must be buildable");
    cmd.arg("daemon")
        .arg("stop")
        .env("MONOCLE_RUNTIME_DIR", runtime_dir.as_os_str());
    cmd
}

/// Spawn the `test-stubborn-daemon` binary as a surrogate "daemon" process that ignores
/// SIGTERM. Waits for the ready signal before returning to eliminate the SIGTERM race
/// window between `spawn()` and SIG_IGN installation.
///
/// This binary is the preferred surrogate for stop-timeout tests because:
/// - It installs SIG_IGN for SIGTERM atomically on startup (no race window).
/// - It signals readiness by writing `b'R'` to stdout AFTER installing SIG_IGN.
/// - It has NO child processes (eliminates orphan process accumulation between tests).
/// - It exits cleanly on SIGKILL (test cleanup).
///
/// Returns the `Child` handle. stdout is consumed (piped and drained for the ready byte).
fn spawn_stubborn_daemon() -> std::process::Child {
    use std::io::Read as _;
    let bin_path = assert_cmd::cargo::cargo_bin("test-stubborn-daemon");
    let mut child = std::process::Command::new(bin_path)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn test-stubborn-daemon");

    // Read the single ready byte. This blocks until the child has installed SIG_IGN,
    // eliminating the race window where SIGTERM could kill the process before it ignores it.
    let mut stdout = child.stdout.take().expect("stdout must be piped");
    let mut ready = [0u8; 1];
    stdout
        .read_exact(&mut ready)
        .expect("read ready byte from test-stubborn-daemon");
    assert_eq!(
        ready[0], b'R',
        "expected 'R' ready signal from test-stubborn-daemon"
    );

    child
}

/// Write a lock file fixture to `dir/monocle.lock` with the given PID and port.
/// Uses `NamedTempFile + persist` per the project anti-pattern rule against naked
/// `std::fs::write`.
fn write_lock_fixture(dir: &std::path::Path, pid: i32, port: u16) {
    use std::io::Write as _;

    let content = serde_json::json!({
        "contract_version": 1,
        "pid": pid,
        "port": port,
        "authToken": "b".repeat(64),
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

/// Write a malformed lock file with an invalid JSON structure to `dir/monocle.lock`.
fn write_malformed_lock(dir: &std::path::Path) {
    use std::io::Write as _;

    let mut tmp = tempfile::NamedTempFile::new_in(dir).expect("create NamedTempFile");
    tmp.write_all(b"{ not valid json at all !")
        .expect("write malformed lock content");
    tmp.persist(dir.join("monocle.lock"))
        .expect("persist malformed lock fixture");
}

/// A PID that will never be alive: far above the kernel's maximum PID on any platform.
const DEAD_PID: i32 = i32::MAX - 1;

// ---------------------------------------------------------------------------
// AC-009: Happy path — daemon running, exits 0, no stdout, no stderr
// ---------------------------------------------------------------------------

/// BC-2.04.005 PC-1..PC-4: `monocle daemon stop` with a live daemon (current process PID)
/// must send SIGTERM and poll until exit, then exit 0 with no stdout.
///
/// We use the current test process PID to represent a "live daemon". The actual SIGTERM
/// will be sent to the test process itself — however, since SIGTERM handling in a Rust
/// test process is non-trivial, this test primarily verifies the exit code behavior.
/// The full happy-path integration test requires a real running daemon subprocess; that
/// level of test belongs to S-017/S-018. This test verifies that when a valid lock file
/// exists with a live PID, the command eventually exits 0.
///
/// For Red Gate purposes: todo!() panics with exit 101, not 0. Assertion fails.
///
/// NOTE: Sending SIGTERM to the current test process would kill the test runner. To avoid
/// this, we use a subprocess as the "daemon" PID target. We spawn `sleep infinity` as a
/// surrogate long-running process, write its PID to the lock file, then invoke stop.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 PC-1, PC-2, PC-3, PC-4, AC-009.
#[test]
fn test_ac_009_stop_running_daemon_exits_zero() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Spawn a surrogate "daemon" process that we can safely SIGTERM.
    // `sleep 30` is harmless and will exit quickly on SIGTERM.
    let mut surrogate = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("spawn sleep surrogate process");

    let surrogate_pid = surrogate.id() as i32;
    write_lock_fixture(tmp.path(), surrogate_pid, 39_200);

    // Invoke `monocle daemon stop` — this should send SIGTERM to the surrogate and poll.
    let assert_result = stop_cmd(tmp.path())
        .timeout(std::time::Duration::from_secs(20))
        .assert()
        .code(0); // WILL FAIL: todo!() produces 101

    // Reap the surrogate process (may already be dead from SIGTERM if stop worked).
    let _ = surrogate.wait();
    let _ = assert_result;
}

/// BC-2.04.005 INV-3: On success (exit 0), no bytes are written to stdout.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 INV-3, AC-009.
#[test]
fn test_ac_009_stop_no_stdout_on_success() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    let mut surrogate = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("spawn sleep surrogate");
    let surrogate_pid = surrogate.id() as i32;
    write_lock_fixture(tmp.path(), surrogate_pid, 39_201);

    stop_cmd(tmp.path())
        .timeout(std::time::Duration::from_secs(20))
        .assert()
        .code(0) // WILL FAIL: todo!() produces 101
        .stdout(predicate::str::is_empty());

    let _ = surrogate.wait();
}

/// BC-2.04.005 INV-4: On success (exit 0), no bytes are written to stderr.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 INV-4, AC-009.
#[test]
fn test_ac_009_stop_no_stderr_on_success() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    let mut surrogate = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("spawn sleep surrogate");
    let surrogate_pid = surrogate.id() as i32;
    write_lock_fixture(tmp.path(), surrogate_pid, 39_202);

    stop_cmd(tmp.path())
        .timeout(std::time::Duration::from_secs(20))
        .assert()
        .code(0) // WILL FAIL: todo!() produces 101
        .stderr(predicate::str::is_empty());

    let _ = surrogate.wait();
}

// ---------------------------------------------------------------------------
// AC-010: Lock file absent — exits 1 with stderr error message
// ---------------------------------------------------------------------------

/// BC-2.04.005 PC-5: If the lock file does not exist, the command must write
/// `error: no lock file found; daemon may not be running` to stderr and exit 1.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 PC-5, EC-2.04.005-01, AC-010.
#[test]
fn test_ac_010_stop_no_lock_file_exits_1_with_stderr() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Pre-condition: no lock file exists.
    assert!(
        !tmp.path().join("monocle.lock").exists(),
        "test pre-condition: lock file must not exist"
    );

    stop_cmd(tmp.path()).assert().code(1); // WILL FAIL: todo!() produces 101
}

/// BC-2.04.005 PC-5: Verify the exact stderr message when lock file is absent.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 PC-5, AC-010.
#[test]
fn test_ac_010_stop_no_lock_file_stderr_format() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    stop_cmd(tmp.path())
        .assert()
        .code(1) // WILL FAIL: todo!() produces 101
        .stderr(predicate::str::contains(
            "error: no lock file found; daemon may not be running",
        )); // WILL FAIL: todo!() panic message instead
}

// ---------------------------------------------------------------------------
// AC-011: Stale lock file (PID not alive) — exits 1 with stderr error message
// ---------------------------------------------------------------------------

/// BC-2.04.005 PC-6: If the lock file exists but the PID is not alive, the command must
/// write `error: daemon not running (stale lock file?)` to stderr and exit 1.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 PC-6, EC-2.04.005-02, AC-011.
#[test]
fn test_ac_011_stop_stale_lock_exits_1_with_stderr() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Write a lock file with a dead PID.
    write_lock_fixture(tmp.path(), DEAD_PID, 39_203);

    stop_cmd(tmp.path()).assert().code(1); // WILL FAIL: todo!() produces 101
}

/// BC-2.04.005 PC-6: Verify the exact stderr message for a stale lock file.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 PC-6, AC-011.
#[test]
fn test_ac_011_stop_stale_lock_stderr_format() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    write_lock_fixture(tmp.path(), DEAD_PID, 39_204);

    stop_cmd(tmp.path())
        .assert()
        .code(1) // WILL FAIL: todo!() produces 101
        .stderr(predicate::str::contains(
            "error: daemon not running (stale lock file?)",
        )); // WILL FAIL: todo!() panic message instead
}

// ---------------------------------------------------------------------------
// AC-012: Timeout — daemon did not exit within 15 s, exits 2
// ---------------------------------------------------------------------------

/// BC-2.04.005 PC-7: When the daemon does not exit within 15 seconds, the command must
/// write `error: daemon did not exit within 15 s; it may still be draining` to stderr
/// and exit 2. The daemon is NOT killed.
///
/// # Test Design
///
/// We use the `test-stubborn-daemon` binary (a Rust test helper compiled alongside
/// monocle) as the surrogate "daemon". It installs SIG_IGN for SIGTERM on startup,
/// has no child processes (eliminating orphan accumulation between tests), and sleeps
/// for 60 seconds before exiting normally.
///
/// The stop timeout is shortened to 2 seconds via `MONOCLE_STOP_TIMEOUT_SECS=2` to
/// keep CI fast. The error message in stderr always reads "15 s" (per the BC contract)
/// regardless of the configured timeout.
///
/// Traces to BC-2.04.005 PC-7, EC-2.04.005-04, AC-012.
#[test]
fn test_ac_012_stop_timeout_exits_2_with_stderr() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Spawn the test-stubborn-daemon: ignores SIGTERM, no child processes.
    let mut stubborn = spawn_stubborn_daemon();

    let stubborn_pid = stubborn.id() as i32;
    write_lock_fixture(tmp.path(), stubborn_pid, 39_205);

    // MONOCLE_STOP_TIMEOUT_SECS=2 shortens the poll window to 2 seconds for CI speed.
    // The error message always says "15 s" per the BC contract.
    stop_cmd(tmp.path())
        .env("MONOCLE_STOP_TIMEOUT_SECS", "2")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .code(2);

    let _ = stubborn.kill();
    let _ = stubborn.wait();
}

/// BC-2.04.005 PC-7: Verify the exact stderr message for the timeout path.
///
/// Traces to BC-2.04.005 PC-7, AC-012.
#[test]
fn test_ac_012_stop_timeout_stderr_format() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    let mut stubborn = spawn_stubborn_daemon();

    let stubborn_pid = stubborn.id() as i32;
    write_lock_fixture(tmp.path(), stubborn_pid, 39_206);

    stop_cmd(tmp.path())
        .env("MONOCLE_STOP_TIMEOUT_SECS", "2")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "error: daemon did not exit within 15 s; it may still be draining",
        ));

    let _ = stubborn.kill();
    let _ = stubborn.wait();
}

/// BC-2.04.005 PC-7 + INV-5: After timeout, the daemon process must still be alive
/// (SIGKILL was NOT sent). Verify that the target PID remains alive after `daemon stop`
/// exits with code 2.
///
/// Traces to BC-2.04.005 PC-7, INV-1, INV-5, AC-012, AC-013.
#[test]
fn test_ac_012_stop_timeout_daemon_still_alive_no_sigkill() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    let mut stubborn = spawn_stubborn_daemon();

    let stubborn_pid = stubborn.id() as i32;
    write_lock_fixture(tmp.path(), stubborn_pid, 39_207);

    // Run stop with a 2-second timeout; expect exit 2.
    stop_cmd(tmp.path())
        .env("MONOCLE_STOP_TIMEOUT_SECS", "2")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .code(2);

    // After stop exits with code 2, verify the process is still alive.
    // `kill(pid, 0)` returns Ok(()) when the process exists, Err when it does not.
    let liveness_result = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(stubborn_pid),
        None, // signal 0: liveness check only
    );
    assert!(
        liveness_result.is_ok(),
        "daemon process (PID {}) must still be alive after `daemon stop` timeout (exit 2) — \
        SIGKILL was not sent per BC-2.04.005 INV-1 / AC-013; \
        kill(pid, 0) returned: {:?}",
        stubborn_pid,
        liveness_result
    );

    // Test cleanup: send SIGKILL manually (NOT what the stop command does).
    let _ = stubborn.kill();
    let _ = stubborn.wait();
}

// ---------------------------------------------------------------------------
// AC-013: No SIGKILL invariant
// ---------------------------------------------------------------------------

/// BC-2.04.005 INV-1: `monocle daemon stop` NEVER sends SIGKILL. This is an absolute
/// invariant. This test verifies the invariant by checking that the source code does not
/// contain any `SIGKILL` send path accessible from `cmd_daemon_stop()`.
///
/// This is a structural test that inspects the source (via compile-time string grep)
/// rather than a runtime test. The absence of SIGKILL in the stop source is the contract.
///
/// Since this test does not invoke the binary, it cannot be broken by todo!() panics —
/// it will PASS trivially because the current stub does not send SIGKILL.
///
/// However, we pair it with a runtime assertion: the process targetted for stop must
/// remain alive after receiving SIGTERM but before the 15-second window expires
/// (demonstrating that stop does not escalate to SIGKILL prematurely).
///
/// RED GATE: This specific test PASSES (structural check). The runtime companion tests
/// (test_ac_012_stop_timeout_daemon_still_alive_no_sigkill) carry the Red Gate burden.
///
/// Traces to BC-2.04.005 INV-1, AC-013.
#[test]
fn test_ac_013_no_sigkill_ever_sent() {
    // Structural assertion: read the stop command source and verify SIGKILL is not referenced.
    // This test reads the source file directly as a string and checks for forbidden patterns.
    // Note: `nix::sys::signal::Signal::SIGKILL` would be the correct way to send SIGKILL.

    let main_src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("read monocle/src/main.rs for structural SIGKILL audit");

    // BC-2.04.005 INV-1: the stop command must NOT contain a SIGKILL send.
    // Acceptable patterns:
    // - SIGTERM references are fine (that is the only signal stop sends)
    // - SIGKILL references in COMMENTS (describing what we do NOT do) are fine
    // - SIGKILL references in VARIABLE NAMES or code that sends it are FORBIDDEN
    //
    // We check for the code pattern that would actually send SIGKILL:
    // `Signal::SIGKILL` or `kill(pid, SIGKILL)` as a functional call.
    // Note: a comment saying "we do not send SIGKILL" is a valid documentation pattern.
    //
    // Currently the stub does not contain this pattern. After S-018 implements stop,
    // this test will continue to pass as a sentinel that SIGKILL was not added.
    //
    // The test WILL PASS in Red Gate because the todo!() stub does not contain SIGKILL code.
    // After implementation, it should continue to pass because implementation must not
    // add SIGKILL.
    assert!(
        !main_src.contains("kill(") || {
            // The kill() call is only allowed for SIGTERM, liveness (signal 0), or similar.
            // We do NOT allow SIGKILL in the code (only in comments).
            // A crude but effective check: if "SIGKILL" appears as a code token (not in a comment),
            // the test fails.
            //
            // Strip Rust line comments (//) and block comments (/* */) for the check.
            let no_line_comments: String = main_src
                .lines()
                .map(|line| {
                    if let Some(pos) = line.find("//") {
                        &line[..pos]
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            !no_line_comments.contains("SIGKILL")
        },
        "monocle/src/main.rs must NOT contain SIGKILL in executable code \
        (only in comments is acceptable). BC-2.04.005 INV-1 / AC-013: \
        `daemon stop` must never send SIGKILL. Found SIGKILL reference in non-comment code."
    );
}

// ---------------------------------------------------------------------------
// AC-009 edge case: malformed lock file exits 1 with stderr
// ---------------------------------------------------------------------------

/// BC-2.04.005 EC-2.04.005-06: A malformed lock file (invalid JSON) must cause exit 1
/// with `error: malformed lock file; daemon may not be running` on stderr.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 EC-2.04.005-06, AC-009.
#[test]
fn test_ac_009_malformed_lock_file_exits_1() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    write_malformed_lock(tmp.path());

    stop_cmd(tmp.path()).assert().code(1); // WILL FAIL: todo!() produces 101
}

/// BC-2.04.005 EC-2.04.005-06: Verify the exact stderr message for a malformed lock file.
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 EC-2.04.005-06, AC-009.
#[test]
fn test_ac_009_malformed_lock_file_stderr_format() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    write_malformed_lock(tmp.path());

    stop_cmd(tmp.path())
        .assert()
        .code(1) // WILL FAIL: todo!() produces 101
        .stderr(predicate::str::contains(
            "error: malformed lock file; daemon may not be running",
        )); // WILL FAIL: todo!() panic message instead
}

// ---------------------------------------------------------------------------
// AC-009 edge case: RuntimeDirUnresolvable exits 70 (consistent with start)
// ---------------------------------------------------------------------------

/// BC-2.04.005 EC-2.04.005-07 + PC-8: When runtime directory resolution fails,
/// `daemon stop` must exit 70 (consistent with the exit code table for RuntimeDirUnresolvable).
///
/// RED GATE: This test will FAIL because `cmd_daemon_stop()` is `todo!()`.
///
/// Traces to BC-2.04.005 EC-2.04.005-07, PC-8, AC-009.
#[test]
fn test_ac_009_exit_code_70_runtime_dir_unresolvable() {
    let mut cmd = Command::cargo_bin("monocle").expect("monocle binary must be buildable");
    cmd.arg("daemon")
        .arg("stop")
        .env_remove("MONOCLE_RUNTIME_DIR")
        .env_remove("HOME")
        .env_remove("XDG_RUNTIME_DIR")
        .env_remove("XDG_DATA_HOME")
        .env_remove("USERPROFILE");

    cmd.assert().code(70); // WILL FAIL: todo!() produces 101
}
