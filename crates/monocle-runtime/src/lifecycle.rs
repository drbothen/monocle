//! Runtime directory resolution and daemon lifecycle (exit-code taxonomy) for the monocle daemon.
//!
//! Populated by S-006 (Lock File Atomic Lifecycle) and S-005 (Graceful Shutdown).
//!
//! # XDG / Platform Resolution Order
//!
//! 1. `MONOCLE_RUNTIME_DIR` environment variable (highest priority; used in tests and
//!    container deployments).
//! 2. `directories::ProjectDirs::new("monocle", "monocle", "monocle")`:
//!    - `runtime_dir()` — returns `None` on macOS (no XDG_RUNTIME_DIR equivalent).
//!    - Fallback: `data_local_dir()` — always set on all supported platforms.
//! 3. If `ProjectDirs::new(...)` returns `None` (e.g., `$HOME` is unset):
//!    `Err(DaemonStartError::RuntimeDirUnresolvable)`.
//!
//! # Security
//!
//! The runtime directory MUST be created with mode `0o700` (owner-only read/write/execute)
//! to prevent other local users from reading the lock file and extracting the auth token.
//! `ensure_runtime_dir` enforces this via `DirBuilder::new().mode(0o700)`.
//!
//! # Daemon Exit-Code Taxonomy (BC-2.01.004 PC-8)
//!
//! [`DaemonExit`] encodes the 5-code POSIX exit taxonomy for the monocle daemon.
//! [`exit_with`] is the **sole call-site** for `std::process::exit` in the entire codebase.
//! No handler, task, or signal-callback may call `std::process::exit` directly.
//! This invariant is enforced by SS-conventions-anti-patterns.md §"No `std::process::exit`
//! in handler code" and verified by a structural source-grep test.

use std::path::{Path, PathBuf};

use crate::errors::DaemonStartError;

/// Resolve the monocle daemon runtime directory path.
///
/// Checks `MONOCLE_RUNTIME_DIR` first, then falls back to
/// `directories::ProjectDirs::new("monocle", "monocle", "monocle")`.
/// On macOS, `runtime_dir()` returns `None`; the fallback is `data_local_dir()`.
///
/// # Errors
///
/// Returns [`DaemonStartError::RuntimeDirUnresolvable`] when both the environment
/// variable is absent AND `ProjectDirs::new(...)` returns `None` (which happens when
/// `$HOME` is unset or the platform provides no home directory equivalent).
pub fn resolve_runtime_dir() -> Result<PathBuf, DaemonStartError> {
    // Priority 1: MONOCLE_RUNTIME_DIR env var (non-empty).
    if let Ok(dir) = std::env::var("MONOCLE_RUNTIME_DIR") {
        if !dir.is_empty() {
            tracing::info!(
                dir = %dir,
                "runtime_dir from MONOCLE_RUNTIME_DIR env var"
            );
            return Ok(PathBuf::from(dir));
        }
    }

    // Priority 2: platform ProjectDirs.
    //
    // Before delegating to `directories::ProjectDirs::from()`, verify that the
    // environment provides a usable home directory via env vars. On macOS,
    // `dirs-sys` has a `getpwuid_r` fallback that can resolve a home directory
    // even when `HOME` is not set — but in that case the path is not under the
    // user's control and cannot be trusted in a headless/container deployment.
    //
    // BC-2.01.005 PC-2d / EC-059: when HOME and all XDG paths are absent from the
    // environment, return Err(RuntimeDirUnresolvable) rather than silently using
    // a path derived from the password database.
    //
    // We check the same variables that `directories` / `dirs-sys` inspect on
    // each platform so that `temp_env::with_vars` in tests can reliably trigger
    // the unresolvable path.
    #[cfg(target_os = "macos")]
    let home_available = std::env::var_os("HOME")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    #[cfg(not(target_os = "macos"))]
    let home_available = {
        // Linux / Unix: HOME or XDG_DATA_HOME are the primary env vars.
        let has_home = std::env::var_os("HOME")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let has_xdg = std::env::var_os("XDG_DATA_HOME")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        has_home || has_xdg
    };

    if !home_available {
        tracing::warn!(
            "HOME env var is not set; cannot resolve platform runtime dir (BC-2.01.005 PC-2d)"
        );
        return Err(DaemonStartError::RuntimeDirUnresolvable);
    }

    let proj = directories::ProjectDirs::from("", "monocle", "monocle")
        .ok_or(DaemonStartError::RuntimeDirUnresolvable)?;

    // runtime_dir() is Some on Linux (XDG_RUNTIME_DIR), None on macOS.
    if let Some(rd) = proj.runtime_dir() {
        Ok(rd.to_path_buf())
    } else {
        tracing::info!(
            platform = std::env::consts::OS,
            "runtime_dir fallback to data_local_dir"
        );
        Ok(proj.data_local_dir().to_path_buf())
    }
}

/// Ensure the runtime directory exists with mode `0o700`.
///
/// Creates the directory (and any missing parent directories) using
/// `DirBuilder::new().mode(0o700).recursive(true).create(path)`.
///
/// # Errors
///
/// Returns [`DaemonStartError::RuntimeDirCreateFailure`] wrapping the underlying
/// `std::io::Error` if directory creation fails.
///
/// # Platform note
///
/// The `mode()` method requires `std::os::unix::fs::DirBuilderExt`, which is only
/// available on Unix targets. Windows is not a supported platform for S-006.
pub fn ensure_runtime_dir(path: &Path) -> Result<(), DaemonStartError> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(path)
        .map_err(DaemonStartError::RuntimeDirCreateFailure)
}

// ---------------------------------------------------------------------------
// Daemon exit-code taxonomy (BC-2.01.004 PC-8, S-005)
// ---------------------------------------------------------------------------

/// The reason the monocle daemon is terminating.
///
/// Encodes the 5-code POSIX exit taxonomy from BC-2.01.004 PC-8. Every daemon termination
/// path MUST pass through [`exit_with`], which is the **sole call-site** for
/// `std::process::exit` in the codebase (SS-conventions-anti-patterns.md §"No
/// `std::process::exit` in handler code").
///
/// # POSIX 128+N convention
///
/// Signal-induced exits follow the POSIX 128+N convention (128 + signal number):
/// - SIGINT = signal 2 → exit code 130 (128+2)
/// - SIGTERM = signal 15 → exit code 143 (128+15)
///
/// External monitoring systems (systemd `Restart=on-failure`, k8s
/// `terminationGracePeriodSeconds`, CI status parsers) **MUST** use exit code 143
/// (not 130) to detect SIGTERM hard-kill during drain. Exit 130 encodes SIGINT
/// (Ctrl-C second press), not SIGTERM (BC-2.01.004 INV-4).
///
/// # Non-POSIX-128+N codes
///
/// - Exit `2` (AdminForceStop) is a monocle-specific code outside the POSIX 128+N
///   range (which starts at 129). It is distinct from startup-failure exit 1 so that
///   monitoring systems can distinguish operator-initiated force-stop from daemon
///   startup failure.
/// - Exit `1` (StartupFailure) is the conventional failure exit code; it covers all
///   cases where the daemon could not start (runtime directory unresolvable, port bind
///   failure, existing live lock file).
/// - Exit `0` (Graceful) means all in-flight requests completed within the 10-second
///   drain window. The lock file is removed and the UDS socket is closed before this
///   exit fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonExit {
    /// All in-flight requests completed within the 10-second drain window.
    ///
    /// Lock file and UDS socket removed before exit. Exit code `0`.
    Graceful,

    /// Daemon failed to start (runtime directory unresolvable, port bind failure,
    /// or existing live lock file). Exit code `1`.
    StartupFailure,

    /// A second authenticated `POST /shutdown` was received while a drain was already
    /// in progress. Monocle-specific programmatic code (outside POSIX 128+N range).
    /// Exit code `2`.
    AdminForceStop,

    /// A second SIGINT (signal 2, Ctrl-C) was received while a drain was in progress.
    /// POSIX convention 128+2. Exit code `130`.
    SigintDuringDrain,

    /// A second SIGTERM (signal 15) was received while a drain was in progress.
    /// POSIX convention 128+15. Exit code `143`.
    ///
    /// Note: the drain timeout expiring WITHOUT a second SIGTERM is NOT this variant —
    /// drain-timeout-forced-shutdown exits with `DaemonExit::Graceful` (exit code 0)
    /// per story spec line 171: "drain-timeout-forced-shutdown exits 0". This variant
    /// is exclusively for when a second SIGTERM arrives before the drain window closes.
    SigtermDuringDrain,
}

impl DaemonExit {
    /// Map this exit reason to the OS exit code.
    ///
    /// | Variant | Code | Rationale |
    /// |---------|------|-----------|
    /// | `Graceful` | 0 | Clean drain |
    /// | `StartupFailure` | 1 | Startup error |
    /// | `AdminForceStop` | 2 | Second POST /shutdown during drain |
    /// | `SigintDuringDrain` | 130 | POSIX 128+2 (SIGINT=2) |
    /// | `SigtermDuringDrain` | 143 | POSIX 128+15 (SIGTERM=15) |
    pub fn to_exit_code(self) -> i32 {
        match self {
            DaemonExit::Graceful => 0,
            DaemonExit::StartupFailure => 1,
            DaemonExit::AdminForceStop => 2,
            DaemonExit::SigintDuringDrain => 130,
            DaemonExit::SigtermDuringDrain => 143,
        }
    }
}

/// Terminate the daemon process with the exit code corresponding to `reason`.
///
/// This is the **sole call-site** for `std::process::exit` in the entire monocle codebase.
/// All daemon termination paths MUST call this function rather than calling
/// `std::process::exit` directly (SS-conventions-anti-patterns.md §"No
/// `std::process::exit` in handler code").
///
/// On a graceful exit (`DaemonExit::Graceful`), the S-005 implementation must invoke
/// `DaemonLock::release()` BEFORE calling this function to ensure the lock file and UDS
/// socket are removed from the filesystem (BC-2.01.004 PC-7).
///
/// # Stub note (S-005 Red Gate)
///
/// The S-005 implementation will wire `exit_with` as the final step of the graceful
/// shutdown sequence. During the Red Gate phase, the stub returns HTTP 501 from
/// `handlers::shutdown::post_shutdown` and this function is not called by any test path
/// that exercises the shutdown behavior.
///
/// # Returns
///
/// This function never returns (`-> !`). The process exits immediately.
pub fn exit_with(reason: DaemonExit) -> ! {
    let code = reason.to_exit_code();
    tracing::info!(
        exit_code = code,
        reason = ?reason,
        "daemon terminating"
    );
    std::process::exit(code)
}
