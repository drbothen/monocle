//! Integration tests for AC-014: daemon subprocess survives parent exit (detachment).
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause | Notes |
//! |---------------|----|-----------|-------|
//! | test_ac_014_daemon_subprocess_survives_parent_exit | AC-014 | BC-2.04.004 INV-2 | detachment |
//! | test_ac_014_daemon_survives_terminal_session_end | AC-014 | BC-2.04.004 INV-2 | SIGHUP independence |
//! | test_ac_014_daemon_not_child_of_calling_shell | AC-014 | BC-2.04.004 PC-5 | process group |
//!
//! # Red Gate Status
//!
//! ALL tests in this file MUST FAIL against the current stub: `cmd_daemon_start()` is
//! implemented as `todo!("S-017: implement daemon start")`. The todo!() panic causes the
//! `monocle daemon start` process to exit 101 without ever spawning a daemon subprocess.
//!
//! Tests in this file assert that a daemon subprocess is running after the `monocle daemon
//! start` caller exits — this assertion fails because no subprocess was spawned.
//!
//! # Test Strategy
//!
//! To test daemon detachment:
//! 1. Invoke `monocle daemon start` with a controlled runtime directory.
//! 2. Wait for `daemon start` to exit successfully (exit 0).
//! 3. Check that the PID recorded in the lock file is still alive.
//! 4. Verify the daemon process is NOT a child of the test process (its PPID is 1 or init,
//!    not the test process PID).
//!
//! With todo!() stub: step 2 fails because `daemon start` exits 101, not 0.
//! The PID check in step 3 therefore cannot be reached.
//!
//! # Double-Fork / setsid Requirement
//!
//! BC-2.04.004 INV-2 requires that the daemon subprocess survives terminal session end.
//! On Unix, this requires either:
//! - Double-fork: grandchild process is reparented to init/systemd.
//! - `setsid()`: creates a new session; the process is no longer in the calling terminal's
//!   process group and does not receive SIGHUP on terminal close.
//!
//! The test verifies this by:
//! - Sending SIGHUP to the daemon PID after start exits.
//! - Checking that the daemon is still alive 100ms later.
//!
//! A non-detached daemon would terminate on SIGHUP if its process group inherits the
//! terminal; a properly detached daemon ignores SIGHUP or has a new session.

// Test files: unwrap/expect are idiomatic assertion amplification in tests, not production.
#![allow(clippy::unwrap_used, clippy::expect_used)]
// Non-snake-case names encode BC IDs per project naming convention.
#![allow(non_snake_case)]

use assert_cmd::Command;
use predicates::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Invoke `monocle daemon start` with the given runtime directory and wait for exit.
/// Returns the `assert_cmd::Output` for exit code inspection.
fn run_daemon_start(runtime_dir: &std::path::Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("monocle").expect("monocle binary must be buildable");
    cmd.arg("daemon")
        .arg("start")
        .env("MONOCLE_RUNTIME_DIR", runtime_dir.as_os_str())
        .timeout(std::time::Duration::from_secs(15));
    cmd.assert()
}

/// Read the PID from the lock file at `runtime_dir/monocle.lock`.
/// Returns the PID if the file exists and is parseable, None otherwise.
fn read_lock_pid(runtime_dir: &std::path::Path) -> Option<i32> {
    let lock_path = runtime_dir.join("monocle.lock");
    let contents = std::fs::read_to_string(&lock_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    json.get("pid").and_then(|v| v.as_i64()).map(|p| p as i32)
}

/// Check whether `pid` is alive using `kill(pid, 0)`.
/// Returns `true` if the process exists, `false` if not.
fn pid_is_alive(pid: i32) -> bool {
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(pid),
        None, // signal 0: liveness check only
    )
    .is_ok()
}

// ---------------------------------------------------------------------------
// AC-014: Daemon subprocess survives parent exit
// ---------------------------------------------------------------------------

/// BC-2.04.004 INV-2 + PC-5: The daemon subprocess continues running after the foreground
/// `monocle daemon start` caller exits. The daemon must be detached from the calling shell
/// (double-fork or setsid equivalent).
///
/// Test sequence:
/// 1. Invoke `monocle daemon start` → expect exit 0.
/// 2. Read PID from lock file.
/// 3. Assert the PID is still alive after the caller exited.
///
/// RED GATE: This test will FAIL at step 1 because `cmd_daemon_start()` is `todo!()`,
/// producing exit 101 instead of 0. With exit 101, no lock file is written and no
/// subprocess is spawned. Steps 2 and 3 are unreachable.
///
/// Traces to BC-2.04.004 INV-2, PC-5, AC-014.
#[test]
fn test_ac_014_daemon_subprocess_survives_parent_exit() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Step 1: Invoke `monocle daemon start` — expect exit 0.
    run_daemon_start(tmp.path()).success(); // WILL FAIL: todo!() produces exit 101

    // Step 2: Read PID from lock file written by the daemon subprocess.
    let daemon_pid = read_lock_pid(tmp.path()).expect(
        "lock file must exist and contain a valid pid after successful `daemon start` \
        (BC-2.04.004 PC-3/PC-4 — lock file appearance is the completion signal); \
        this assertion is unreachable with todo!() stub (step 1 fails first)",
    );

    // Step 3: Assert the daemon PID is still alive after the caller exited.
    assert!(
        pid_is_alive(daemon_pid),
        "daemon process (PID {daemon_pid}) must still be alive after `monocle daemon start` exits \
        (BC-2.04.004 INV-2 / PC-5 / AC-014); \
        the daemon must be detached via double-fork or setsid so it is NOT a child of \
        the calling process. If the daemon dies immediately after the caller exits, \
        the detachment was not properly implemented."
    );

    // Cleanup: send SIGTERM to the daemon subprocess (test cleanup only).
    // This is NOT part of the behavioral assertion — we just don't want to leave
    // orphaned processes after the test.
    let _ = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(daemon_pid),
        nix::sys::signal::Signal::SIGTERM,
    );
}

/// BC-2.04.004 INV-2: The daemon survives terminal session end (SIGHUP independence).
///
/// After `monocle daemon start` exits, we send SIGHUP to the daemon PID and verify it
/// is still alive 200ms later. A non-detached daemon (one that did not call setsid) would
/// terminate on SIGHUP when the terminal closes; a properly detached daemon either:
/// - Has SIG_IGN on SIGHUP (explicitly set during daemon initialization), OR
/// - Is in a new session (setsid) and has no controlling terminal to receive SIGHUP from.
///
/// RED GATE: This test will FAIL at the `success()` assertion because todo!() exits 101.
///
/// Traces to BC-2.04.004 INV-2, AC-014.
#[test]
fn test_ac_014_daemon_survives_terminal_session_end() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Step 1: Start the daemon.
    run_daemon_start(tmp.path()).success(); // WILL FAIL: todo!() produces exit 101

    let daemon_pid = read_lock_pid(tmp.path()).expect(
        "lock file must exist after successful daemon start; \
        unreachable with todo!() stub",
    );

    // Step 2: Send SIGHUP to simulate terminal session end.
    // A properly detached daemon (setsid / double-fork) ignores or handles this.
    let sighup_result = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(daemon_pid),
        nix::sys::signal::Signal::SIGHUP,
    );
    assert!(
        sighup_result.is_ok(),
        "SIGHUP delivery must succeed to a live daemon PID (PID {daemon_pid}); got: {sighup_result:?}"
    );

    // Step 3: Wait 200ms and verify the daemon is still alive.
    // A detached daemon ignores SIGHUP; a non-detached daemon exits on it.
    std::thread::sleep(std::time::Duration::from_millis(200));

    assert!(
        pid_is_alive(daemon_pid),
        "daemon process (PID {daemon_pid}) must remain alive after receiving SIGHUP \
        (BC-2.04.004 INV-2 / AC-014). \
        A daemon that dies on SIGHUP is not properly detached from its terminal. \
        The implementation must call setsid() or double-fork so the daemon has no \
        controlling terminal to receive SIGHUP from."
    );

    // Cleanup.
    let _ = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(daemon_pid),
        nix::sys::signal::Signal::SIGTERM,
    );
}

/// BC-2.04.004 PC-5: The daemon's PPID (parent PID) must NOT be the calling shell.
///
/// After `monocle daemon start` exits, the daemon process must have been reparented to
/// init/systemd (PID 1 or near-init) — its PPID must not be the calling test process.
///
/// This is verified by reading `/proc/<pid>/status` on Linux or using `ps` on macOS.
///
/// RED GATE: This test will FAIL at the `success()` assertion because todo!() exits 101.
///
/// Traces to BC-2.04.004 PC-5, INV-2, AC-014.
#[test]
fn test_ac_014_daemon_not_child_of_calling_shell() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    // Step 1: Start the daemon.
    run_daemon_start(tmp.path()).success(); // WILL FAIL: todo!() produces exit 101

    let daemon_pid = read_lock_pid(tmp.path()).expect(
        "lock file must exist after successful daemon start; \
        unreachable with todo!() stub",
    );

    // Step 2: Read the daemon's parent PID.
    let test_process_pid = std::process::id() as i32;

    // Use `ps` to get the PPID of the daemon process — cross-platform approach.
    let ps_output = std::process::Command::new("ps")
        .args(["-o", "ppid=", "-p", &daemon_pid.to_string()])
        .output()
        .expect("ps command must be available to read daemon PPID");

    let ppid_str = std::str::from_utf8(&ps_output.stdout)
        .expect("ps output must be valid UTF-8")
        .trim();

    let daemon_ppid: i32 = ppid_str
        .parse()
        .unwrap_or_else(|_| panic!("ps ppid output must be a valid integer; got: {ppid_str:?}"));

    // Step 3: Assert the daemon's parent is NOT the test process.
    assert_ne!(
        daemon_ppid, test_process_pid,
        "daemon process (PID {daemon_pid}) must NOT be a child of the calling test process (PID {test_process_pid}). \
        After double-fork or setsid detachment, the daemon must be reparented to init/systemd. \
        Got daemon PPID = {daemon_ppid}. BC-2.04.004 PC-5 / INV-2 / AC-014."
    );

    // The daemon's PPID should be init (1) or a subreaper (small PID near 1 on systemd systems).
    // We do NOT assert an exact PPID of 1 because systemd subreapers may be used.
    // The assertion above (PPID != test process) is the necessary and sufficient check
    // for the double-fork/setsid requirement.

    // Cleanup.
    let _ = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(daemon_pid),
        nix::sys::signal::Signal::SIGTERM,
    );
}

// ---------------------------------------------------------------------------
// AC-014 additional: lock file existence after start exits confirms detachment timing
// ---------------------------------------------------------------------------

/// BC-2.04.004 PC-3/PC-4: The lock file must exist after `daemon start` exits 0,
/// confirming that the daemon subprocess has written the lock file before the caller returns.
///
/// This is a prerequisite for the detachment test: if no lock file exists, no subprocess
/// was spawned or the subprocess crashed before writing the lock.
///
/// RED GATE: This test will FAIL because `cmd_daemon_start()` is `todo!()`.
///
/// Traces to BC-2.04.004 PC-3, PC-4, INV-1, AC-014.
#[test]
fn test_ac_014_lock_file_exists_after_start_exits() {
    let tmp = tempfile::tempdir().expect("create temp runtime dir");

    run_daemon_start(tmp.path())
        .success() // WILL FAIL: todo!() produces exit 101
        .stdout(predicate::str::is_empty()); // no stdout on success

    // After exit 0, the lock file must exist.
    assert!(
        tmp.path().join("monocle.lock").exists(),
        "monocle.lock must exist at <runtime_dir>/monocle.lock after `daemon start` exits 0 \
        (BC-2.04.004 INV-1: lock file is the completion signal; \
        BC-2.04.004 PC-3/PC-4: caller polls for lock file and exits when it appears); \
        this assertion is unreachable with todo!() stub"
    );
}
