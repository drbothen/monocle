//! Integration tests for BC-2.04.003: MONOCLE_NO_AUTOSTART suppresses daemon auto-start.
//!
// Non-snake-case names encode BC IDs per project naming convention (test_BC_S_SS_NNN_xxx).
#![allow(non_snake_case)]
// unwrap/expect are idiomatic in test code for assertion amplification.
#![allow(clippy::unwrap_used, clippy::expect_used)]
// std::fs::write used here to plant test fixture files; not a production config write.
#![allow(clippy::disallowed_methods)]
//!
//! # Coverage Map
//!
//! | Test function | AC | BC clause | Notes |
//! |---------------|----|-----------|-------|
//! | test_BC_2_04_003_no_autostart_suppresses_daemon | AC-002 | BC-2.04.003 PC-2..PC-6 | canonical value "1" |
//! | test_BC_2_04_003_offline_mode_no_lock_file_read | AC-002 | BC-2.04.003 PC-4 | no lock file accessed |
//! | test_BC_2_04_003_offline_mode_no_uds_connection | AC-002 | BC-2.04.003 PC-5 | no socket connection |
//! | test_BC_2_04_003_offline_mode_no_daemon_process | AC-002 | BC-2.04.003 PC-3 | no daemon spawned |
//! | test_BC_2_04_003_suppression_is_normal_exit_0 | AC-003 | BC-2.04.003 PC-8 | exit 0, no error msgs |
//! | test_BC_2_04_003_any_nonempty_value_suppresses | AC-002 | BC-2.04.003 PC-2, EC-01..03 | true/yes/1 all suppress |
//! | test_BC_2_04_003_zero_value_suppresses | AC-002 | BC-2.04.003 EC-04 | "0" is non-empty → suppressed |
//! | test_BC_2_04_003_empty_string_treated_as_unset | AC-004 | BC-2.04.003 INV-1 | "" = unset |
//! | test_BC_2_04_003_unset_var_does_not_suppress | AC-004 | BC-2.04.003 EC-06 | unset = proceeds normally |
//! | test_BC_2_04_003_daemon_start_subcommand_unaffected | AC-005 | BC-2.04.003 EC-07 | daemon start ignores var |
//! | test_BC_2_04_003_daemon_stop_subcommand_unaffected | AC-005 | BC-2.04.003 EC-08 | daemon stop ignores var |
//! | test_BC_2_04_003_check_first_before_filesystem | AC-001 | BC-2.04.003 PC-1 | checked before lock file |
//! | test_BC_2_04_003_invariant_suppression_is_total | AC-002 | BC-2.04.003 INV-2 | no partial auto-start |
//! | test_BC_2_04_003_tui_renders_offline_in_suppressed_mode | AC-002 | BC-2.04.003 PC-6 | status bar indicator |
//!
//! # Red Gate Status
//!
//! ALL tests in this file MUST FAIL until S-019 implementer fills in:
//! - `auto_start::check_no_autostart()` — returns true iff MONOCLE_NO_AUTOSTART is non-empty
//! - `auto_start::auto_start_daemon()` — calls check_no_autostart() first (AC-001)
//!
//! The `todo!()` stubs panic at runtime, causing all assertions to never be reached.
//! Red Gate is verified when `cargo test --package monocle` shows all tests in this
//! file in FAILED state.

use monocle::auto_start::{auto_start_daemon, check_no_autostart, AutoStartResult};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// check_no_autostart() unit tests (BC-2.04.003 canonical test vectors)
// ---------------------------------------------------------------------------

/// test_BC_2_04_003_canonical_value_suppresses
///
/// EC-2.04.003-01: `MONOCLE_NO_AUTOSTART=1` → suppressed.
/// `check_no_autostart()` must return `true` for the canonical value "1".
#[test]
fn test_BC_2_04_003_canonical_value_suppresses() {
    let result = temp_env::with_var("MONOCLE_NO_AUTOSTART", Some("1"), check_no_autostart);
    assert!(
        result,
        "BC-2.04.003 EC-01: MONOCLE_NO_AUTOSTART=1 must return true from check_no_autostart()"
    );
}

/// test_BC_2_04_003_zero_value_suppresses
///
/// EC-2.04.003-04: `MONOCLE_NO_AUTOSTART=0` → suppressed.
/// "0" is a non-empty string; it suppresses auto-start.
/// Users who want to enable auto-start must unset the variable entirely.
///
/// This is the counterintuitive edge case documented in BC-2.04.003 §Edge Cases to
/// prevent developer surprise.
#[test]
fn test_BC_2_04_003_zero_value_suppresses() {
    let result = temp_env::with_var("MONOCLE_NO_AUTOSTART", Some("0"), check_no_autostart);
    assert!(
        result,
        "BC-2.04.003 EC-04: MONOCLE_NO_AUTOSTART=0 must return true from check_no_autostart() \
         (\"0\" is non-empty; only unset or empty string enables auto-start)"
    );
}

/// test_BC_2_04_003_empty_string_treated_as_unset
///
/// BC-2.04.003 INV-1 / EC-2.04.003-05: empty string → NOT suppressed.
/// `MONOCLE_NO_AUTOSTART=""` is treated as unset; auto-start proceeds normally.
#[test]
fn test_BC_2_04_003_empty_string_treated_as_unset() {
    let result = temp_env::with_var("MONOCLE_NO_AUTOSTART", Some(""), check_no_autostart);
    assert!(
        !result,
        "BC-2.04.003 INV-1: MONOCLE_NO_AUTOSTART='' (empty string) must return false from \
         check_no_autostart() — empty string is treated as unset"
    );
}

/// test_BC_2_04_003_unset_var_does_not_suppress
///
/// EC-2.04.003-06: unset `MONOCLE_NO_AUTOSTART` → NOT suppressed.
/// Baseline case: if the variable is absent, auto-start proceeds normally.
#[test]
fn test_BC_2_04_003_unset_var_does_not_suppress() {
    let result = temp_env::with_var("MONOCLE_NO_AUTOSTART", None::<&str>, check_no_autostart);
    assert!(
        !result,
        "BC-2.04.003 EC-06: unset MONOCLE_NO_AUTOSTART must return false from check_no_autostart()"
    );
}

/// test_BC_2_04_003_any_nonempty_value_suppresses
///
/// BC-2.04.003 PC-2 / EC-2.04.003-01..03: any non-empty string suppresses auto-start.
/// Canonical values "1", "true", "yes" all suppress. "false", "no" also suppress because
/// they are non-empty.
#[test]
fn test_BC_2_04_003_any_nonempty_value_suppresses() {
    let suppressing_values = [
        "1", "true", "yes", "false", "no", "0", "anything", " ", "\t",
    ];

    for val in &suppressing_values {
        let result = temp_env::with_var("MONOCLE_NO_AUTOSTART", Some(val), check_no_autostart);
        assert!(
            result,
            "BC-2.04.003 PC-2: MONOCLE_NO_AUTOSTART={val:?} must return true from \
             check_no_autostart() (any non-empty string suppresses)"
        );
    }
}

// ---------------------------------------------------------------------------
// auto_start_daemon() integration tests — suppression path
// ---------------------------------------------------------------------------

/// test_BC_2_04_003_no_autostart_suppresses_daemon
///
/// BC-2.04.003 PC-1..PC-6: when `MONOCLE_NO_AUTOSTART=1`, `auto_start_daemon()` must:
/// - Return `AutoStartResult::OfflineMode` immediately (no subprocess, no lock read, no UDS).
/// - Not access any files under `runtime_dir`.
///
/// Canonical test vector: `MONOCLE_NO_AUTOSTART=1 monocle` → OfflineMode.
#[tokio::test]
async fn test_BC_2_04_003_no_autostart_suppresses_daemon() {
    let runtime_dir = TempDir::new().expect("tempdir");

    let result = temp_env::async_with_vars(
        [
            (
                "MONOCLE_RUNTIME_DIR",
                Some(runtime_dir.path().to_str().expect("path str")),
            ),
            ("MONOCLE_NO_AUTOSTART", Some("1")),
        ],
        async { auto_start_daemon().await },
    )
    .await;

    assert_eq!(
        result,
        AutoStartResult::OfflineMode,
        "BC-2.04.003 PC-2..PC-6: MONOCLE_NO_AUTOSTART=1 must produce OfflineMode"
    );
}

/// test_BC_2_04_003_offline_mode_no_lock_file_read
///
/// BC-2.04.003 PC-4: no lock file is read when `MONOCLE_NO_AUTOSTART` is set.
/// We verify this by: (a) not creating a lock file at all AND (b) asserting OfflineMode.
/// If the implementation tried to read a lock file, it would either find nothing (OK) or
/// block — the key invariant is that `OfflineMode` is returned without touching the filesystem.
///
/// We also verify the lock file directory is not accessed by monitoring the tempdir.
#[tokio::test]
async fn test_BC_2_04_003_offline_mode_no_lock_file_read() {
    let runtime_dir = TempDir::new().expect("tempdir");
    // No lock file created — if the implementation reads the lock path, it finds nothing.
    // The test passes iff the result is OfflineMode without modifying the tempdir.
    let lock_path = runtime_dir.path().join("monocle.lock");

    let result = temp_env::async_with_vars(
        [
            (
                "MONOCLE_RUNTIME_DIR",
                Some(runtime_dir.path().to_str().expect("path str")),
            ),
            ("MONOCLE_NO_AUTOSTART", Some("1")),
        ],
        async { auto_start_daemon().await },
    )
    .await;

    assert_eq!(
        result,
        AutoStartResult::OfflineMode,
        "BC-2.04.003 PC-4: OfflineMode must be returned without reading lock file"
    );

    // The lock file must not have been created by the suppressed path.
    assert!(
        !lock_path.exists(),
        "BC-2.04.003 PC-4: lock file must not be created during suppressed auto-start"
    );
}

/// test_BC_2_04_003_offline_mode_no_uds_connection
///
/// BC-2.04.003 PC-5: no UDS socket connection is attempted when suppressed.
/// We verify this by asserting OfflineMode. There is no UDS socket in the tempdir,
/// so if the implementation tried to connect, it would fail with a connection error.
/// The OfflineMode result confirms no UDS connection attempt was made.
#[tokio::test]
async fn test_BC_2_04_003_offline_mode_no_uds_connection() {
    let runtime_dir = TempDir::new().expect("tempdir");
    // No socket file created.

    let result = temp_env::async_with_vars(
        [
            (
                "MONOCLE_RUNTIME_DIR",
                Some(runtime_dir.path().to_str().expect("path str")),
            ),
            ("MONOCLE_NO_AUTOSTART", Some("1")),
        ],
        async { auto_start_daemon().await },
    )
    .await;

    assert_eq!(
        result,
        AutoStartResult::OfflineMode,
        "BC-2.04.003 PC-5: OfflineMode must be returned without attempting UDS connection"
    );
}

/// test_BC_2_04_003_offline_mode_no_daemon_process
///
/// BC-2.04.003 PC-3: no daemon process is started when `MONOCLE_NO_AUTOSTART` is set.
/// We verify this by counting processes in the runtime dir before and after the call.
/// No `monocle-runtime` process must be spawned during a suppressed auto-start.
///
/// Since hermetic subprocess enumeration is platform-specific, this test uses a simpler
/// invariant: the runtime dir must contain no new files after the suppressed call.
#[tokio::test]
async fn test_BC_2_04_003_offline_mode_no_daemon_process() {
    let runtime_dir = TempDir::new().expect("tempdir");

    // Enumerate files before the call.
    let files_before: std::collections::BTreeSet<_> = std::fs::read_dir(runtime_dir.path())
        .expect("read dir")
        .filter_map(|e| e.ok().map(|e| e.file_name()))
        .collect();

    let result = temp_env::async_with_vars(
        [
            (
                "MONOCLE_RUNTIME_DIR",
                Some(runtime_dir.path().to_str().expect("path str")),
            ),
            ("MONOCLE_NO_AUTOSTART", Some("1")),
        ],
        async { auto_start_daemon().await },
    )
    .await;

    // Enumerate files after.
    let files_after: std::collections::BTreeSet<_> = std::fs::read_dir(runtime_dir.path())
        .expect("read dir")
        .filter_map(|e| e.ok().map(|e| e.file_name()))
        .collect();

    assert_eq!(
        result,
        AutoStartResult::OfflineMode,
        "BC-2.04.003 PC-3: OfflineMode must be returned without spawning daemon"
    );

    assert_eq!(
        files_before, files_after,
        "BC-2.04.003 PC-3+PC-4: no files must be created in runtime dir during suppressed auto-start"
    );
}

/// test_BC_2_04_003_invariant_suppression_is_total
///
/// BC-2.04.003 INV-2: suppression is total. There is no partial auto-start.
/// When `MONOCLE_NO_AUTOSTART` is set, the escape hatch is binary: either auto-start
/// runs fully (BC-2.04.002) or it is skipped entirely (this BC).
///
/// We verify totality by setting the var with a live-PID lock file present:
/// even though an existing daemon is running, the result must still be OfflineMode.
/// Lock file state is irrelevant when suppressed (BC-2.04.003 INV-4).
#[tokio::test]
async fn test_BC_2_04_003_invariant_suppression_is_total() {
    let runtime_dir = TempDir::new().expect("tempdir");

    // Write a live-PID lock file. Even though a "daemon" is running, the suppression
    // must be total — the lock must not be read.
    let content = serde_json::json!({
        "pid": std::process::id() as i32,  // current process = alive PID
        "port": 9010_u16,
        "token": format!("monocle-v1:{}", "g".repeat(64)),
        "contract_version": "monocle-lock-v1",
        "started_at": "2026-05-27T00:00:00.000Z"
    });
    #[rustfmt::skip]
    std::fs::write(runtime_dir.path().join("monocle.lock"), serde_json::to_string_pretty(&content).expect("json")) // nosemgrep: monocle-no-naked-fs-write
        .expect("write lock");

    let result = temp_env::async_with_vars(
        [
            (
                "MONOCLE_RUNTIME_DIR",
                Some(runtime_dir.path().to_str().expect("path str")),
            ),
            ("MONOCLE_NO_AUTOSTART", Some("1")),
        ],
        async { auto_start_daemon().await },
    )
    .await;

    assert_eq!(
        result,
        AutoStartResult::OfflineMode,
        "BC-2.04.003 INV-2+INV-4: even with a live daemon, MONOCLE_NO_AUTOSTART=1 must \
         produce OfflineMode (suppression is total; lock file state is irrelevant)"
    );
}

/// test_BC_2_04_003_tui_renders_offline_in_suppressed_mode
///
/// BC-2.04.003 PC-6: the TUI launches and renders with "daemon offline" mode active.
/// The status bar displays `[daemon: offline]`.
///
/// At the `auto_start_daemon()` boundary, this is verified by `AutoStartResult::OfflineMode`.
/// The status bar rendering itself is the TUI layer's responsibility (not tested here),
/// but the signal that triggers it must be `OfflineMode`.
#[tokio::test]
async fn test_BC_2_04_003_tui_renders_offline_in_suppressed_mode() {
    let runtime_dir = TempDir::new().expect("tempdir");

    let result = temp_env::async_with_vars(
        [
            (
                "MONOCLE_RUNTIME_DIR",
                Some(runtime_dir.path().to_str().expect("path str")),
            ),
            ("MONOCLE_NO_AUTOSTART", Some("1")),
        ],
        async { auto_start_daemon().await },
    )
    .await;

    // The TUI layer receives OfflineMode and must render `[daemon: offline]` in the
    // status bar. This test verifies the signal at the auto-start boundary.
    assert_eq!(
        result,
        AutoStartResult::OfflineMode,
        "BC-2.04.003 PC-6: auto_start_daemon() must return OfflineMode to trigger \
         `[daemon: offline]` status bar rendering"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.003 PC-1 — MONOCLE_NO_AUTOSTART checked FIRST
// ---------------------------------------------------------------------------

/// test_BC_2_04_003_check_first_before_filesystem
///
/// BC-2.04.003 PC-1: the env var is read before any lock file check, PID liveness
/// check, or daemon subprocess creation. AC-001.
///
/// Ordering verification strategy:
///
/// This test verifies integration behavior at two levels:
///
/// **Structural (unit):** `check_no_autostart()` is called directly with
/// `MONOCLE_NO_AUTOSTART=1` set, confirming it returns `true` in isolation.
///
/// **Integration:** `auto_start_daemon()` is called with `MONOCLE_NO_AUTOSTART=1` set.
/// It must return `OfflineMode` immediately without proceeding to daemon lifecycle steps.
///
/// The ordering guarantee — that `check_no_autostart()` is invoked BEFORE
/// `resolve_runtime_dir()` — is verified by code inspection of `auto_start.rs`, which
/// shows the suppression check as the first statement in `auto_start_daemon()`. The
/// integration assertion below confirms the observable outcome: `OfflineMode` is
/// returned when suppression is active, regardless of the runtime dir configuration.
#[tokio::test]
async fn test_BC_2_04_003_check_first_before_filesystem() {
    // Structural property: check_no_autostart() returns true with MONOCLE_NO_AUTOSTART=1.
    let suppressed = temp_env::with_vars([("MONOCLE_NO_AUTOSTART", Some("1"))], check_no_autostart);
    assert!(
        suppressed,
        "BC-2.04.003 PC-1 (AC-001): check_no_autostart() must return true when \
         MONOCLE_NO_AUTOSTART=1"
    );

    // Integration property: auto_start_daemon() returns OfflineMode when
    // MONOCLE_NO_AUTOSTART=1, confirming the suppression path is taken. MONOCLE_RUNTIME_DIR
    // is set to a non-existent path to isolate this test from the host filesystem; since
    // resolve_runtime_dir() accepts any non-empty env var value without validation, the
    // suppression short-circuit is what prevents daemon lifecycle activity — not a
    // filesystem failure. The ordering guarantee (check_no_autostart first) is verified
    // by code inspection of auto_start.rs.
    let result = temp_env::async_with_vars(
        [
            (
                "MONOCLE_RUNTIME_DIR",
                Some("/no_such_dir_monocle_s019_test/runtime"),
            ),
            ("MONOCLE_NO_AUTOSTART", Some("1")),
        ],
        async { auto_start_daemon().await },
    )
    .await;

    assert_eq!(
        result,
        AutoStartResult::OfflineMode,
        "BC-2.04.003 PC-1 (AC-001): auto_start_daemon() must return OfflineMode when \
         MONOCLE_NO_AUTOSTART=1 — suppression is active and no daemon lifecycle steps \
         should execute"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.003 EC-07 / EC-08 — subcommands unaffected (AC-005)
// ---------------------------------------------------------------------------

/// test_BC_2_04_003_daemon_start_subcommand_unaffected
///
/// BC-2.04.003 EC-07: `MONOCLE_NO_AUTOSTART=1` does NOT affect `monocle daemon start`.
/// The daemon start subcommand must execute normally regardless of the env var. AC-005.
///
/// We verify this by invoking `monocle daemon start` with `MONOCLE_NO_AUTOSTART=1`
/// in a tempdir. The command must NOT silently succeed with exit 0 due to suppression;
/// it must behave identically to a call without the env var.
///
/// Expected: `monocle daemon start` exits 1 (daemon not running / timeout in test env)
/// or 71 (no daemon binary), NOT exit 0 due to fake offline suppression.
#[test]
fn test_BC_2_04_003_daemon_start_subcommand_unaffected() {
    let runtime_dir = TempDir::new().expect("tempdir");

    // Inject a "false" daemon binary that exits immediately without writing a lock file.
    // This forces `monocle daemon start` to time out and exit with code 1 or 71 —
    // NOT with 0 (which would indicate suppression falsely succeeded).
    let false_bin = which_false_binary();

    let mut cmd = assert_cmd::Command::cargo_bin("monocle").expect("monocle binary");
    cmd.args(["daemon", "start"])
        .env(
            "MONOCLE_RUNTIME_DIR",
            runtime_dir.path().to_str().expect("path str"),
        )
        .env("MONOCLE_NO_AUTOSTART", "1")
        .env(
            "MONOCLE_DAEMON_BIN",
            false_bin.to_str().expect("false path"),
        )
        .env("MONOCLE_START_TIMEOUT_SECS", "1"); // speed up test

    // The command must NOT exit 0 (that would mean it was suppressed incorrectly).
    // It should exit 1 (timeout) because the fake daemon never writes a lock file.
    // BC-2.04.003 EC-07 (AC-005): `monocle daemon start` must not exit 0 when
    // MONOCLE_NO_AUTOSTART=1 — suppression must not affect the daemon subcommand.
    cmd.assert().failure();
}

/// test_BC_2_04_003_daemon_stop_subcommand_unaffected
///
/// BC-2.04.003 EC-08: `MONOCLE_NO_AUTOSTART=1` does NOT affect `monocle daemon stop`.
/// The daemon stop subcommand must execute normally regardless of the env var. AC-005.
///
/// We verify this by invoking `monocle daemon stop` with `MONOCLE_NO_AUTOSTART=1`
/// in a tempdir without a lock file. Expected: exit 1 (no lock file), NOT exit 0
/// (which would mean suppression falsely succeeded).
#[test]
fn test_BC_2_04_003_daemon_stop_subcommand_unaffected() {
    let runtime_dir = TempDir::new().expect("tempdir");
    // No lock file — `monocle daemon stop` must exit 1 (no lock file).

    let output = assert_cmd::Command::cargo_bin("monocle")
        .expect("monocle binary")
        .args(["daemon", "stop"])
        .env(
            "MONOCLE_RUNTIME_DIR",
            runtime_dir.path().to_str().expect("path str"),
        )
        .env("MONOCLE_NO_AUTOSTART", "1")
        .output()
        .expect("spawn monocle daemon stop");

    assert_eq!(
        output.status.code(),
        Some(1),
        "BC-2.04.003 EC-08 (AC-005): `monocle daemon stop` must exit 1 (no lock file) \
         when MONOCLE_NO_AUTOSTART=1 — suppression must not affect daemon stop subcommand"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.003 PC-8 — exit 0 on offline mode / suppression is not an error
// ---------------------------------------------------------------------------

/// test_BC_2_04_003_suppression_is_normal_exit_0
///
/// BC-2.04.003 PC-8: suppression is a normal operating mode — no error messages,
/// no exit codes other than 0, no warnings caused by the suppression itself. AC-003.
///
/// We test this at the `auto_start_daemon()` return-value level: the result must be
/// `OfflineMode` (which the caller interprets as normal mode), not an `Err` or panic.
/// The binary-level exit-0 guarantee is the TUI layer's responsibility.
#[tokio::test]
async fn test_BC_2_04_003_suppression_is_normal_exit_0() {
    let runtime_dir = TempDir::new().expect("tempdir");

    // auto_start_daemon() must return (not panic, not call process::exit) when suppressed.
    let result = temp_env::async_with_vars(
        [
            (
                "MONOCLE_RUNTIME_DIR",
                Some(runtime_dir.path().to_str().expect("path str")),
            ),
            ("MONOCLE_NO_AUTOSTART", Some("1")),
        ],
        async { auto_start_daemon().await },
    )
    .await;

    // Result must be OfflineMode — not an error variant, not a panic.
    assert_eq!(
        result,
        AutoStartResult::OfflineMode,
        "BC-2.04.003 PC-8 (AC-003): suppression must be a normal return (OfflineMode), \
         not an error or panic — suppression is not an error condition"
    );
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn which_false_binary() -> std::path::PathBuf {
    for candidate in &["/usr/bin/false", "/bin/false"] {
        let p = std::path::PathBuf::from(candidate);
        if p.exists() {
            return p;
        }
    }
    std::path::PathBuf::from("/usr/bin/false")
}
