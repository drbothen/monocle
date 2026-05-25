//! Runtime directory resolution for the monocle daemon.
//!
//! Populated by S-006 (Lock File Atomic Lifecycle).
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
