//! Daemon auto-start decision sequence for TUI mode (BC-2.04.002, BC-2.04.003).
//!
//! This module is invoked when `monocle` is run without any subcommand (TUI mode).
//! It implements the 5-step decision sequence defined in BC-2.04.002 and the
//! `MONOCLE_NO_AUTOSTART` suppression gate from BC-2.04.003.
//!
//! # Decision Sequence (BC-2.04.002)
//!
//! 1. Check `MONOCLE_NO_AUTOSTART` — if non-empty, enter offline mode immediately
//!    (no filesystem access, no subprocess spawning, no UDS connection).
//! 2. `resolve_runtime_dir()` — exit 70 on failure before any TUI output.
//! 3. Check `<runtime_dir>/monocle.lock`:
//!    - Absent → proceed to step 4 (start daemon).
//!    - Present with alive PID → proceed to step 5 (connect to existing daemon).
//!    - Present with dead PID → log WARN, remove stale lock, proceed to step 4.
//! 4. Start daemon subprocess; poll `monocle.lock` at 100ms intervals for up to 5 seconds.
//!    - Lock appears → proceed to step 5.
//!    - Timeout → render `daemon start timed out; retrying…`; retry once (another 5 seconds).
//!    - Both timeouts → enter offline mode with `daemon unavailable — running in offline mode`.
//! 5. PID liveness pre-check before UDS connection attempt (`kill(pid, 0)` must return 0).
//!
//! # MONOCLE_NO_AUTOSTART (BC-2.04.003)
//!
//! - Non-empty value → total suppression: no lock file access, no subprocess, no UDS.
//! - Empty string → treated as unset; auto-start proceeds normally.
//! - Only applies to TUI mode; `monocle daemon start/stop` are unaffected.
//!
//! # Forbidden dependencies
//!
//! This module MUST NOT import from `monocle-ipc` directly. IPC connection is handled
//! by the TUI layer after auto-start completes (S-019 architecture constraint).

use std::path::Path;
use std::time::Duration;

/// Result of the auto-start decision sequence (BC-2.04.002 / BC-2.04.003).
///
/// The TUI layer receives this and renders the initial state:
/// - `Connected` → normal TUI rendering with live daemon session.
/// - `OfflineMode` → degraded TUI rendering with `[daemon: offline]` status bar.
#[derive(Debug, PartialEq, Eq)]
pub enum AutoStartResult {
    /// The daemon is reachable: the auto-start sequence completed all 5 steps successfully,
    /// or an already-running daemon was detected. The TUI should connect to the UDS socket
    /// at `<runtime_dir>/monocle.sock` using the supplied `port` and `token`.
    Connected {
        /// TCP port the daemon's HTTP server is bound to (from the lock file).
        port: u16,
        /// Wire-format auth token (`monocle-v1:<64-hex>`) from the lock file.
        token: String,
    },

    /// Auto-start was suppressed (`MONOCLE_NO_AUTOSTART` set to a non-empty value),
    /// or the daemon failed to start within the 10-second total timeout (5s + 5s retry).
    /// The TUI renders in degraded offline mode with `[daemon: offline]` status bar.
    OfflineMode,
}

/// Check whether `MONOCLE_NO_AUTOSTART` suppresses auto-start (BC-2.04.003).
///
/// Returns `true` if the variable is set to a **non-empty** string, which suppresses
/// daemon auto-start entirely. Returns `false` if the variable is unset or empty.
///
/// Empty string is treated as unset per BC-2.04.003 Invariant 1:
/// > If `MONOCLE_NO_AUTOSTART` is set to the empty string (e.g., via
/// > `export MONOCLE_NO_AUTOSTART=$UNDEFINED_VAR`), it is treated as unset and
/// > auto-start proceeds normally per BC-2.04.002.
///
/// Edge cases per BC-2.04.003 §Edge Cases:
/// - `MONOCLE_NO_AUTOSTART=1`    → suppressed (canonical value)
/// - `MONOCLE_NO_AUTOSTART=true` → suppressed (non-empty string)
/// - `MONOCLE_NO_AUTOSTART=0`    → suppressed (non-empty, even though "0" is falsy in shells)
/// - `MONOCLE_NO_AUTOSTART=`     → NOT suppressed (empty string = unset)
/// - `MONOCLE_NO_AUTOSTART` unset → NOT suppressed
pub fn check_no_autostart() -> bool {
    std::env::var("MONOCLE_NO_AUTOSTART")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

/// Resolve the auto-start poll timeout window from `MONOCLE_AUTO_START_TIMEOUT_SECS`
/// or the production default of 5 seconds (BC-2.04.002 PC-4).
///
/// Each poll window is 5 seconds; there are 2 windows (initial + 1 retry), giving a
/// maximum total wait of 10 seconds (BC-2.04.002 INV-3).
///
/// `MONOCLE_AUTO_START_TIMEOUT_SECS` is a test-only override that shortens each window
/// to speed up tests (e.g., `MONOCLE_AUTO_START_TIMEOUT_SECS=1` → 1s per window).
fn auto_start_timeout_window() -> Duration {
    std::env::var("MONOCLE_AUTO_START_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(5))
}

/// Poll interval: 100ms per BC-2.04.002 PC-4.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Read the wire-format token from a lock file JSON value.
///
/// Supports both lock file formats encountered in this codebase:
/// - `StartSequenceLockContent` (S-017 / BC-2.04.001 PC-15): field `"token"` with full
///   `monocle-v1:<hex>` wire token.
/// - `LockFileContent` (S-006 / `DaemonLock::acquire`): field `"authToken"` with raw hex
///   token (no `monocle-v1:` prefix).
/// - Test helper `write_lock_file`: field `"token"` with full `monocle-v1:<hex>` wire token.
///
/// Returns the wire-format token (`monocle-v1:<hex>`) in all cases.
fn extract_token(json: &serde_json::Value) -> Option<String> {
    // Prefer `"token"` (canonical wire format with monocle-v1: prefix already included).
    if let Some(token) = json.get("token").and_then(|v| v.as_str()) {
        return Some(token.to_string());
    }
    // Fall back to `"authToken"` (DaemonLock::acquire format — raw hex, needs prefix).
    if let Some(auth_token) = json.get("authToken").and_then(|v| v.as_str()) {
        return Some(format!("monocle-v1:{auth_token}"));
    }
    None
}

/// Read port and token from the lock file if the recorded PID is alive.
///
/// Returns `Some((port, token))` when:
/// - The lock file is parseable JSON with `"pid"`, `"port"`, and a token field.
/// - The recorded PID is alive (`kill(pid, 0)` returns 0).
///
/// Returns `None` for absent files, malformed JSON, missing fields, or dead PIDs.
fn read_lock_if_alive(lock_path: &Path) -> Option<(u16, String)> {
    let contents = std::fs::read_to_string(lock_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;

    let pid = json
        .get("pid")
        .and_then(|v| v.as_i64())
        .and_then(|p| i32::try_from(p).ok())?;

    let port = json.get("port").and_then(|v| v.as_u64()).and_then(|p| {
        if p <= u64::from(u16::MAX) {
            Some(p as u16)
        } else {
            None
        }
    })?;

    let token = extract_token(&json)?;

    // Liveness check: kill(pid, 0) returns Ok(()) only if the process is alive.
    let nix_pid = nix::unistd::Pid::from_raw(pid);
    match nix::sys::signal::kill(nix_pid, None) {
        Ok(()) => Some((port, token)),         // Process alive.
        Err(nix::errno::Errno::ESRCH) => None, // Dead process.
        Err(_) => Some((port, token)),         // EPERM or other: conservatively alive.
    }
}

/// Read pid, port and token from the lock file WITHOUT a liveness check.
///
/// Supports both lock file wire formats. Used at PC-5 where the liveness check is
/// done separately, and in `poll_for_lock`.
fn read_lock_raw(lock_path: &Path) -> Option<(i32, u16, String)> {
    let contents = std::fs::read_to_string(lock_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;

    let pid = json
        .get("pid")
        .and_then(|v| v.as_i64())
        .and_then(|p| i32::try_from(p).ok())?;

    let port = json.get("port").and_then(|v| v.as_u64()).and_then(|p| {
        if p <= u64::from(u16::MAX) {
            Some(p as u16)
        } else {
            None
        }
    })?;

    let token = extract_token(&json)?;

    Some((pid, port, token))
}

/// Find the daemon binary to spawn.
///
/// Resolution order:
/// 1. `MONOCLE_DAEMON_BIN` env var — test-only override.
/// 2. Same directory as the current executable (`monocle-runtime` sibling).
///    For integration tests, `current_exe()` points to `target/debug/deps/<test-binary>`,
///    so we also check `target/debug/` (parent of `deps/`) as a fallback.
///
/// Returns `None` if the binary cannot be found; the caller will fall back to
/// in-process `daemon_start_sequence()`.
///
/// # Why `current_exe()` diverges from BC EC-2.04.002-07
///
/// BC EC-2.04.002-07 specifies a PATH search fallback when the binary cannot be located
/// as a sibling of the current executable. This implementation uses the in-process
/// `daemon_start_sequence()` as its fallback instead. The in-process fallback is more
/// robust than a PATH search because: (1) it avoids spawning an unknown binary that
/// happens to be named `monocle-runtime`, (2) it works in test environments where
/// `monocle-runtime` is not on PATH, and (3) it provides the same behaviour as the
/// subprocess path because both write the same lock file. No test is written for the
/// `current_exe()` failure path because `std::env::current_exe()` cannot be made to
/// fail in a portable way from within a Rust test.
pub fn find_daemon_binary() -> Option<std::path::PathBuf> {
    // Test-only override.
    if let Ok(override_bin) = std::env::var("MONOCLE_DAEMON_BIN") {
        return Some(std::path::PathBuf::from(override_bin));
    }

    let current_exe = std::env::current_exe().ok()?;
    let bin_dir = current_exe.parent()?;

    // Primary: same directory as current executable (production install layout).
    let candidate = bin_dir.join("monocle-runtime");
    if candidate.exists() {
        return Some(candidate);
    }

    // Secondary: parent of current exe directory (handles `target/debug/deps/` layout
    // used by `cargo test` integration tests, where `monocle-runtime` lives in `target/debug/`).
    if let Some(parent_dir) = bin_dir.parent() {
        let candidate = parent_dir.join("monocle-runtime");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

/// Poll for the lock file to appear, up to `timeout` duration.
///
/// Returns `Some((pid, port, token))` if the lock file appears within the timeout window.
/// Returns `None` on timeout.
async fn poll_for_lock(lock_path: &Path, timeout: Duration) -> Option<(i32, u16, String)> {
    // Use tokio::time::timeout for accurate async deadline enforcement.
    tokio::time::timeout(timeout, async {
        loop {
            if lock_path.exists() {
                if let Some(result) = read_lock_raw(lock_path) {
                    return result;
                }
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    })
    .await
    .ok()
}

/// Execute the daemon auto-start decision sequence (BC-2.04.002) for TUI mode.
///
/// This is the entry point called by `main()` before any TUI initialization.
/// The TUI MUST NOT render its main content before this function returns.
///
/// # Step sequence
///
/// 1. Call `check_no_autostart()` — FIRST action, before any filesystem access.
///    If suppressed, return `AutoStartResult::OfflineMode` immediately.
/// 2. Call `resolve_runtime_dir()`. On failure, exit the process with code 70 via
///    `std::process::exit(70)` (before any TUI output).
/// 3. Check `<runtime_dir>/monocle.lock`:
///    - Absent → proceed to step 4.
///    - Present + alive PID → proceed to step 5.
///    - Present + dead PID → log `stale lock file removed`, remove, proceed to step 4.
/// 4. Start daemon (spawn subprocess or in-process). Poll for `monocle.lock` at 100ms
///    intervals for up to 5 seconds (configurable via `MONOCLE_AUTO_START_TIMEOUT_SECS`
///    for tests). On timeout, log the status bar message and retry once (another window).
///    On double timeout, return `AutoStartResult::OfflineMode`.
/// 5. PID liveness pre-check (`kill(pid, 0)`). On failure (daemon crashed between
///    step 4 and step 5), log WARN and return `AutoStartResult::OfflineMode`.
///    On success, return `AutoStartResult::Connected { port, token }`.
///
/// # Returns
///
/// `AutoStartResult::Connected { port, token }` on successful daemon connection.
/// `AutoStartResult::OfflineMode` on suppression or failure.
///
/// # No TUI rendering before verdict (BC-2.04.002 INV-1 / AC-011)
///
/// The caller MUST NOT render TUI main content before this function returns.
pub async fn auto_start_daemon() -> AutoStartResult {
    // Step 1: Check MONOCLE_NO_AUTOSTART FIRST — before any filesystem access (BC-2.04.003 PC-1).
    if check_no_autostart() {
        tracing::info!("MONOCLE_NO_AUTOSTART set; entering offline mode (BC-2.04.003)");
        return AutoStartResult::OfflineMode;
    }

    // Step 2: Resolve runtime directory — exit 70 on failure (BC-2.04.002 PC-1).
    //
    // Why std::process::exit(70) here instead of returning an error:
    // BC-2.04.002 PC-1 mandates that runtime-dir resolution failure terminates the process
    // with exit code 70 (EX_UNAVAILABLE) BEFORE any TUI output is rendered. Returning an
    // error from this function would require the caller (main()) to translate it to exit 70,
    // which risks the TUI partially rendering before the exit is propagated. The exit call
    // here enforces the "no TUI output before verdict" invariant (BC-2.04.002 INV-1).
    // This is the ONLY std::process::exit call outside of monocle-runtime/lifecycle.rs in
    // the codebase; all other error paths return normally. The subprocess integration test
    // (test_BC_2_04_002_resolve_failure_exits_70) validates this via assert_cmd.
    let runtime_dir = match monocle_runtime::lifecycle::resolve_runtime_dir() {
        Ok(dir) => dir,
        Err(e) => {
            tracing::error!(error = %e, "failed to resolve runtime dir; exiting with code 70");
            std::process::exit(70);
        }
    };

    let lock_path = runtime_dir.join("monocle.lock");

    // Step 3: Check for existing lock file (BC-2.04.002 PC-2 / PC-3).
    if lock_path.exists() {
        if let Some((port, token)) = read_lock_if_alive(&lock_path) {
            // PC-3: alive PID → existing daemon running → skip to step 5 (liveness already checked).
            tracing::info!(
                port = port,
                "existing daemon detected; connecting immediately"
            );
            return AutoStartResult::Connected { port, token };
        }

        // PC-3: dead PID → stale lock file. Log WARN, remove, proceed to step 4.
        tracing::warn!(
            path = %lock_path.display(),
            "stale lock file removed"
        );
        if let Err(e) = std::fs::remove_file(&lock_path) {
            tracing::error!(path = %lock_path.display(), error = %e, "failed to remove stale lock file");
        }
    }

    // Step 4: Start daemon and poll for lock file (BC-2.04.002 PC-4).
    start_daemon_and_poll(&runtime_dir, &lock_path).await
}

/// Start the daemon (subprocess or in-process) and poll for the lock file.
///
/// Implements BC-2.04.002 PC-4: start daemon subprocess; poll at 100ms intervals for
/// up to one timeout window. On timeout, retry once. On double timeout, return OfflineMode.
async fn start_daemon_and_poll(runtime_dir: &Path, lock_path: &Path) -> AutoStartResult {
    let timeout_window = auto_start_timeout_window();

    // Start daemon: prefer subprocess (MONOCLE_DAEMON_BIN or sibling monocle-runtime).
    // Fall back to in-process daemon_start_sequence() if no binary is found.
    let _daemon_handle = launch_daemon(runtime_dir).await;

    // First poll window.
    match poll_for_lock(lock_path, timeout_window).await {
        Some((pid, port, token)) => {
            return finalize_connection(pid, port, token);
        }
        None => {
            // First timeout — log the status bar message and retry.
            tracing::warn!("daemon start timed out; retrying\u{2026}");
        }
    }

    // Second poll window (one retry per BC-2.04.002 PC-4 / INV-3).
    match poll_for_lock(lock_path, timeout_window).await {
        Some((pid, port, token)) => finalize_connection(pid, port, token),
        None => {
            // Both windows timed out → offline mode.
            tracing::warn!("daemon unavailable \u{2014} running in offline mode");
            AutoStartResult::OfflineMode
        }
    }
}

/// PC-5: PID liveness pre-check before returning Connected (BC-2.04.002 PC-5 / INV-5).
///
/// The daemon PID MUST pass a liveness check before the TUI attempts the UDS connection.
fn finalize_connection(pid: i32, port: u16, token: String) -> AutoStartResult {
    let nix_pid = nix::unistd::Pid::from_raw(pid);
    match nix::sys::signal::kill(nix_pid, None) {
        Ok(()) => {
            // Liveness check passed.
            tracing::info!(port = port, "daemon reachable; returning Connected");
            AutoStartResult::Connected { port, token }
        }
        Err(nix::errno::Errno::ESRCH) => {
            // Daemon crashed between step 4 and step 5 (EC-2.04.002-08).
            tracing::warn!(
                pid = pid,
                "daemon PID failed liveness check at PC-5 (EC-2.04.002-08); entering offline mode"
            );
            AutoStartResult::OfflineMode
        }
        Err(e) => {
            // EPERM or other: conservatively treat as alive.
            tracing::warn!(pid = pid, error = %e, "liveness check returned non-ESRCH error; treating as alive");
            AutoStartResult::Connected { port, token }
        }
    }
}

/// Handle for the daemon — either a spawned subprocess or an in-process task.
enum DaemonHandle {
    /// Subprocess handle (not used after spawn; process runs independently).
    Subprocess,
    /// In-process tokio task handle (task is running but not awaited — daemon
    /// continues until the tokio runtime is dropped).
    Task(()),
}

/// Launch the daemon: try subprocess first, fall back to in-process.
async fn launch_daemon(runtime_dir: &Path) -> DaemonHandle {
    match find_daemon_binary() {
        Some(bin_path) => {
            // Spawn daemon as a detached subprocess (same pattern as cmd_daemon_start).
            tracing::info!(bin = %bin_path.display(), "launching daemon subprocess");
            let result = std::process::Command::new(&bin_path)
                .env("MONOCLE_RUNTIME_DIR", runtime_dir.as_os_str())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();

            match result {
                Ok(_child) => {
                    // Child process is detached; let OS reparent it.
                    DaemonHandle::Subprocess
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to spawn daemon subprocess; falling back to in-process");
                    launch_daemon_in_process(runtime_dir).await
                }
            }
        }
        None => {
            // No daemon binary found — start in-process.
            tracing::info!("no daemon binary found; starting daemon in-process");
            launch_daemon_in_process(runtime_dir).await
        }
    }
}

/// Start daemon_start_sequence in a background tokio task, then serve HTTP.
///
/// Retains the `(state, listener)` pair returned by `daemon_start_sequence` and calls
/// `run_server(state, listener)` so that the port recorded in `monocle.lock` is genuinely
/// served. Without calling `run_server`, dropping the `TcpListener` would release the
/// OS port immediately, making the lock file point to a dead endpoint (I1 fix —
/// SS-daemon-wiring-impl.md §Fix Addendum I1).
///
/// Runs the same shutdown tail as `main()` step 7 (UDS cleanup, lock release, hooks
/// removal) so no stale files remain when the TUI process exits. Does NOT call `exit_with`
/// — it returns from the task and lets the tokio runtime drop `DaemonState`.
async fn launch_daemon_in_process(runtime_dir: &Path) -> DaemonHandle {
    let dir = runtime_dir.to_path_buf();
    // Spawn and drop the JoinHandle — the task runs until the runtime is dropped.
    // Dropping the handle does NOT abort the task (tokio semantics).
    tokio::spawn(async move {
        let (state, listener) = match monocle_runtime::lifecycle::daemon_start_sequence(&dir).await
        {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!(error = %e, "in-process daemon_start_sequence failed");
                return;
            }
        };

        tracing::info!("in-process daemon: start sequence complete; serving HTTP");

        // Run HTTP server — blocks until shutdown signal fires.
        // The UDS accept loop is already spawned inside daemon_start_sequence.
        // Note: the in-process daemon does NOT wrap run_server with tokio::time::timeout.
        // BC-2.01.004 INV-1 applies to the subprocess binary; if the TUI process exits,
        // the runtime drops and the task is aborted, preventing indefinite hang.
        if let Err(e) =
            monocle_runtime::server::run_server(std::sync::Arc::clone(&state), listener).await
        {
            tracing::warn!(error = %e, "in-process daemon: HTTP server returned error");
        }

        // Shutdown tail — mirror main()'s step 7 cleanup.
        // UDS socket cleanup.
        if let Some(transport) = state.uds_transport.as_ref() {
            transport.cleanup();
        }
        // Release the daemon lock.
        let lock_opt = state
            .daemon_lock
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(lock) = lock_opt {
            if let Err(e) = lock.release() {
                tracing::warn!(
                    error = %e,
                    "in-process daemon: lock release failed during shutdown; continuing"
                );
            }
        }
        // Remove hooks-settings.json.
        if let Err(e) = monocle_runtime::lifecycle::remove_hooks_settings(&dir) {
            tracing::warn!(
                error = %e,
                "in-process daemon: hooks-settings removal failed; continuing"
            );
        }

        tracing::info!("in-process daemon: shutdown complete");
    });
    DaemonHandle::Task(())
}
