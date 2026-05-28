//! monocle — CLI entry point.
//!
//! Provides daemon lifecycle subcommands:
//! - `monocle daemon start` — check PID liveness, spawn detached daemon, poll for lock file,
//!   exit 0 on success or exit 1 on timeout / conflict / exit 70 on runtime-dir error.
//! - `monocle daemon stop`  — read lock file, send SIGTERM, poll for process exit,
//!   exit 0 on success or exit 1/2 on error / exit 70 on runtime-dir error.
//!
//! Implemented in S-016.
//!
//! TUI auto-start (S-019):
//! - No subcommand → TUI mode: `auto_start::auto_start_daemon()` is called before TUI init.
//! - `MONOCLE_NO_AUTOSTART` non-empty → offline mode, no daemon started.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

use std::process::Stdio;
use std::time::{Duration, Instant};

mod auto_start;

use clap::{Parser, Subcommand};

use crate::auto_start::auto_start_daemon;

/// monocle — AI coding harness session manager.
#[derive(Parser, Debug)]
#[command(name = "monocle", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Top-level subcommands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Daemon lifecycle management.
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

/// Daemon subcommands.
#[derive(Subcommand, Debug)]
enum DaemonAction {
    /// Start the monocle daemon.
    Start,
    /// Stop the monocle daemon.
    Stop,
}

// ---------------------------------------------------------------------------
// Exit code constants (BC-2.04.004 PC-8, BC-2.04.005 PC-8)
// ---------------------------------------------------------------------------

/// Exit code 0: success.
const EXIT_SUCCESS: i32 = 0;
/// Exit code 1: conflict (daemon already running, no lock file, stale lock, timeout-start).
const EXIT_CONFLICT: i32 = 1;
/// Exit code 2: stop timeout — daemon did not exit within 15 s.
const EXIT_STOP_TIMEOUT: i32 = 2;
/// Exit code 70: runtime directory unresolvable.
const EXIT_RUNTIME_DIR_UNRESOLVABLE: i32 = 70;
/// Exit code 71: internal error (daemon subprocess startup failure).
const EXIT_INTERNAL_ERROR: i32 = 71;

// ---------------------------------------------------------------------------
// Timing constants (with test-override env vars)
// ---------------------------------------------------------------------------

/// Default maximum time to wait for the lock file to appear after spawning the daemon.
///
/// Per BC-2.04.004 PC-7, production default is 10 seconds.
/// Override via `MONOCLE_START_TIMEOUT_SECS` for testing (e.g., set to 1 or 2).
const DAEMON_START_TIMEOUT_DEFAULT_SECS: u64 = 10;

/// Poll interval when waiting for the lock file to appear (100 ms per spec).
const DAEMON_START_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Default maximum time to wait for the daemon process to exit after SIGTERM.
///
/// Per BC-2.04.005 PC-7, production default is 15 seconds.
/// Override via `MONOCLE_STOP_TIMEOUT_SECS` for testing (e.g., set to 1 or 2).
const DAEMON_STOP_TIMEOUT_DEFAULT_SECS: u64 = 15;

/// Poll interval when waiting for the daemon process to exit (1 s per spec).
const DAEMON_STOP_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Resolve the daemon start timeout from `MONOCLE_START_TIMEOUT_SECS` or the default (10 s).
fn daemon_start_timeout() -> Duration {
    std::env::var("MONOCLE_START_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(DAEMON_START_TIMEOUT_DEFAULT_SECS))
}

/// Resolve the daemon stop timeout from `MONOCLE_STOP_TIMEOUT_SECS` or the default (15 s).
fn daemon_stop_timeout() -> Duration {
    std::env::var("MONOCLE_STOP_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(DAEMON_STOP_TIMEOUT_DEFAULT_SECS))
}

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Some(Commands::Daemon { action }) => match action {
            DaemonAction::Start => cmd_daemon_start(),
            DaemonAction::Stop => cmd_daemon_stop(),
        },
        None => {
            // TUI mode: run the daemon auto-start decision sequence (BC-2.04.002, BC-2.04.003).
            // auto_start_daemon() is async; build a tokio runtime to drive it.
            // The auto-start sequence calls std::process::exit(70) directly on runtime-dir failure.
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!(error = %e, "failed to create tokio runtime for TUI auto-start");
                    eprintln!("error: failed to initialize async runtime: {e}");
                    std::process::exit(EXIT_INTERNAL_ERROR);
                }
            };
            let result = rt.block_on(auto_start_daemon());
            // The TUI layer would use `result` to render initial state.
            // For S-019 scope: the auto-start verdict is determined; TUI rendering is out of scope.
            // Report the verdict and exit 0 (normal TUI exit — the TUI hasn't been built yet).
            match result {
                auto_start::AutoStartResult::Connected { port, token: _ } => {
                    tracing::info!(port = port, "TUI mode: daemon connected");
                }
                auto_start::AutoStartResult::OfflineMode => {
                    tracing::info!("TUI mode: [daemon: offline]");
                }
            }
            EXIT_SUCCESS
        }
    };

    std::process::exit(exit_code);
}

// ---------------------------------------------------------------------------
// `monocle daemon start`
// ---------------------------------------------------------------------------

/// Start the monocle daemon.
///
/// Resolves the runtime directory, checks for an existing live daemon, spawns the
/// daemon subprocess, polls for the lock file, and returns an exit code.
///
/// # Exit codes
///
/// | Code | Condition |
/// |------|-----------|
/// | 0    | Daemon started and lock file appeared within 10 s |
/// | 1    | Daemon already running (live PID in lock file) |
/// | 1    | Lock file did not appear within 10 s (timeout) |
/// | 70   | Runtime directory could not be resolved |
/// | 71   | Daemon subprocess exited before writing the lock file |
fn cmd_daemon_start() -> i32 {
    // Step 1: Resolve the runtime directory.
    let runtime_dir = match monocle_runtime::lifecycle::resolve_runtime_dir() {
        Ok(dir) => dir,
        Err(_) => {
            // EC-2.04.004-06: RuntimeDirUnresolvable → exit 70.
            return EXIT_RUNTIME_DIR_UNRESOLVABLE;
        }
    };

    let lock_path = runtime_dir.join("monocle.lock");

    // Step 2: Check for a live daemon (pre-spawn conflict detection).
    // Read and parse the existing lock file (if any) to check PID liveness.
    // - Absent lock file / stale lock / malformed lock → proceed to spawn (treated as absent).
    // - Live PID in lock file → report conflict and exit 1.
    if lock_path.exists() {
        if let Some(pid) = read_lock_pid_if_live(&lock_path) {
            eprintln!("error: daemon already running (pid={})", pid);
            return EXIT_CONFLICT;
        }
        // Stale or malformed — remove it so the daemon subprocess can write fresh lock.
        // If removal fails, the daemon subprocess will handle it via DaemonLock::acquire.
        let _ = std::fs::remove_file(&lock_path);
    }

    // Step 3: Find the monocle-runtime binary (sibling of current executable).
    let daemon_bin = match find_daemon_binary() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "failed to locate monocle-runtime binary");
            eprintln!("error: could not locate daemon binary: {}", e);
            return EXIT_INTERNAL_ERROR;
        }
    };

    // Step 4: Spawn the daemon subprocess.
    // - Redirect stdin/stdout/stderr to /dev/null (daemon is fully detached from CLI I/O).
    // - Set MONOCLE_RUNTIME_DIR so the daemon uses the same resolved directory.
    // - The daemon binary itself calls setsid() at startup to detach from the terminal.
    let child_result = std::process::Command::new(&daemon_bin)
        .env("MONOCLE_RUNTIME_DIR", runtime_dir.as_os_str())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    let mut child = match child_result {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "failed to spawn daemon subprocess");
            eprintln!("error: could not spawn daemon: {}", e);
            return EXIT_INTERNAL_ERROR;
        }
    };

    // Step 5: Poll for the lock file to appear (100 ms intervals, configurable timeout).
    // Also check whether the daemon subprocess exited prematurely.
    // The timeout is configurable via MONOCLE_START_TIMEOUT_SECS (for testing) and
    // defaults to 10 seconds in production (BC-2.04.004 PC-7).
    let deadline = Instant::now() + daemon_start_timeout();

    loop {
        // Lock file appeared → daemon is ready.
        if lock_path.exists() {
            // Drop the child handle — the OS reparents the daemon to init.
            // The daemon continues running after this function returns.
            drop(child);
            return EXIT_SUCCESS;
        }

        // Check for premature daemon exit.
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Daemon exited before writing the lock file → internal error.
                return EXIT_INTERNAL_ERROR;
            }
            Ok(None) => {
                // Daemon still running — continue polling.
            }
            Err(e) => {
                tracing::error!(error = %e, "error checking daemon subprocess status");
                return EXIT_INTERNAL_ERROR;
            }
        }

        // Check timeout.
        if Instant::now() >= deadline {
            // Kill the orphaned subprocess (it failed to write lock within the timeout).
            let _ = nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(child.id() as i32),
                nix::sys::signal::Signal::SIGTERM,
            );
            eprintln!("error: daemon failed to start within 10 s");
            return EXIT_CONFLICT;
        }

        std::thread::sleep(DAEMON_START_POLL_INTERVAL);
    }
}

// ---------------------------------------------------------------------------
// `monocle daemon stop`
// ---------------------------------------------------------------------------

/// Stop the monocle daemon.
///
/// Reads the lock file for the daemon's PID, sends SIGTERM, and polls for the
/// daemon process to exit within 15 seconds.
///
/// # Exit codes
///
/// | Code | Condition |
/// |------|-----------|
/// | 0    | Daemon received SIGTERM and exited within 15 s |
/// | 1    | Lock file absent |
/// | 1    | Lock file malformed (invalid JSON or missing `pid` field) |
/// | 1    | Daemon not running (stale lock file) |
/// | 2    | Daemon did not exit within 15 s |
/// | 70   | Runtime directory could not be resolved |
fn cmd_daemon_stop() -> i32 {
    // Step 1: Resolve the runtime directory.
    let runtime_dir = match monocle_runtime::lifecycle::resolve_runtime_dir() {
        Ok(dir) => dir,
        Err(_) => {
            return EXIT_RUNTIME_DIR_UNRESOLVABLE;
        }
    };

    let lock_path = runtime_dir.join("monocle.lock");

    // Step 2: Read the lock file.
    let lock_contents = match std::fs::read_to_string(&lock_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("error: no lock file found; daemon may not be running");
            return EXIT_CONFLICT;
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to read lock file");
            eprintln!("error: no lock file found; daemon may not be running");
            return EXIT_CONFLICT;
        }
    };

    // Step 3: Parse the lock file JSON to extract the PID.
    let json: serde_json::Value = match serde_json::from_str(&lock_contents) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("error: malformed lock file; daemon may not be running");
            return EXIT_CONFLICT;
        }
    };

    let pid = match json
        .get("pid")
        .and_then(|v| v.as_i64())
        .and_then(|p| i32::try_from(p).ok())
    {
        Some(p) => p,
        None => {
            eprintln!("error: malformed lock file; daemon may not be running");
            return EXIT_CONFLICT;
        }
    };

    // Step 4: Verify the daemon is alive.
    let nix_pid = nix::unistd::Pid::from_raw(pid);
    match nix::sys::signal::kill(nix_pid, None) {
        Ok(()) => {
            // Process is alive — proceed to SIGTERM.
        }
        Err(nix::errno::Errno::ESRCH) => {
            eprintln!("error: daemon not running (stale lock file?)");
            return EXIT_CONFLICT;
        }
        Err(_) => {
            // EPERM or other error: process may exist. Treat as alive and attempt SIGTERM.
            // (Conservative — prefer not to report stale lock when process may be alive.)
        }
    }

    // Step 5: Send SIGTERM to the daemon.
    // BC-2.04.005 INV-1: NEVER send SIGKILL. SIGTERM only.
    if let Err(e) = nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGTERM) {
        tracing::error!(pid = pid, error = %e, "failed to send SIGTERM to daemon");
        eprintln!("error: failed to signal daemon (pid={}): {}", pid, e);
        return EXIT_CONFLICT;
    }

    // Step 6: Poll for the daemon process to exit (1 s intervals, configurable timeout).
    // The timeout is configurable via MONOCLE_STOP_TIMEOUT_SECS (for testing) and
    // defaults to 15 seconds in production (BC-2.04.005 PC-7).
    let deadline = Instant::now() + daemon_stop_timeout();

    loop {
        std::thread::sleep(DAEMON_STOP_POLL_INTERVAL);

        // Check if the process has exited or become a zombie (effectively dead).
        // Note: kill(pid, None) returns Ok(()) for zombie processes because zombies
        // still occupy a slot in the process table. We use `process_is_gone` which
        // treats zombies as dead (they cannot be signalled meaningfully and will be
        // reaped by their parent / init shortly).
        if process_is_gone(pid) {
            // Process exited (or is zombie, which is effectively dead) — daemon stopped.
            return EXIT_SUCCESS;
        }

        if Instant::now() >= deadline {
            // BC-2.04.005 PC-7: Timeout — do NOT send SIGKILL. Report to user.
            // BC-2.04.005 INV-1: SIGKILL is forbidden under any circumstances.
            // The stderr message always says "15 s" per the BC contract
            // (regardless of MONOCLE_STOP_TIMEOUT_SECS, which is test-only).
            eprintln!("error: daemon did not exit within 15 s; it may still be draining");
            return EXIT_STOP_TIMEOUT;
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check if a process has exited, treating zombie processes as gone.
///
/// `kill(pid, None)` (signal 0) returns `Ok(())` for zombie processes because zombies
/// still occupy a slot in the process table. This function uses `ps` to inspect the
/// process state character: a state of `Z` (zombie) means the process has effectively
/// terminated (its parent just hasn't reaped it yet). An absent PID (ps returns no
/// output or ESRCH) also means the process is gone.
///
/// Returns `true` if the process has exited (ESRCH) or is a zombie (state Z).
/// Returns `false` if the process is alive and in any non-zombie state.
///
/// This is necessary for `daemon stop` tests where the surrogate process is a zombie
/// of the test runner process (the test runner doesn't call wait() until after stop exits).
fn process_is_gone(pid: i32) -> bool {
    // First, fast-path via kill(0): ESRCH means definitively gone.
    match nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None) {
        Err(nix::errno::Errno::ESRCH) => return true,
        Err(_) => {} // Other errors (EPERM etc): check ps for zombie state.
        Ok(()) => {} // Process exists (may be zombie): check ps for zombie state.
    }

    // Slow-path: use ps to check if the process is in zombie state.
    // `ps -o state= -p <pid>` prints the process state character: 'Z' = zombie.
    // If ps returns no output, the process no longer exists.
    let output = match std::process::Command::new("ps")
        .args(["-o", "state=", "-p", &pid.to_string()])
        .output()
    {
        Ok(o) => o,
        Err(_) => return false, // ps unavailable — assume alive (conservative).
    };

    // ps exits non-zero when the process doesn't exist.
    if !output.status.success() {
        return true; // PID not found by ps — process is gone.
    }

    // Process state: 'Z' means zombie (effectively dead, awaiting reap).
    let state = output.stdout.first().copied().unwrap_or(0);
    state == b'Z'
}

/// Read the PID from `lock_path` if the recorded process is currently alive.
///
/// Returns `Some(pid)` if the lock file is parseable JSON with a `pid` field that
/// corresponds to a live process. Returns `None` for absent, malformed, or stale locks.
fn read_lock_pid_if_live(lock_path: &std::path::Path) -> Option<i32> {
    let contents = std::fs::read_to_string(lock_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let pid = json
        .get("pid")
        .and_then(|v| v.as_i64())
        .and_then(|p| i32::try_from(p).ok())?;

    // Use kill(pid, None) to check liveness without sending a signal.
    let nix_pid = nix::unistd::Pid::from_raw(pid);
    match nix::sys::signal::kill(nix_pid, None) {
        Ok(()) => Some(pid),                   // Process alive.
        Err(nix::errno::Errno::ESRCH) => None, // Dead process.
        Err(_) => Some(pid),                   // EPERM or other: assume alive (conservative).
    }
}

/// Find the `monocle-runtime` binary to spawn as the daemon subprocess.
///
/// Delegates to `auto_start::find_daemon_binary()` (the shared canonical implementation)
/// and converts the `Option<PathBuf>` result to `Result<PathBuf, anyhow::Error>` for use
/// in `cmd_daemon_start()`. The shared function also includes the `deps/` parent-dir
/// fallback required for integration-test layout.
fn find_daemon_binary() -> Result<std::path::PathBuf, anyhow::Error> {
    auto_start::find_daemon_binary().ok_or_else(|| {
        anyhow::anyhow!(
            "monocle-runtime binary not found; ensure it is installed alongside monocle"
        )
    })
}
