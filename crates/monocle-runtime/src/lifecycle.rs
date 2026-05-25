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
    unimplemented!("S-006: resolve_runtime_dir — check MONOCLE_RUNTIME_DIR, then ProjectDirs::runtime_dir() / data_local_dir()")
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
pub fn ensure_runtime_dir(_path: &Path) -> Result<(), DaemonStartError> {
    unimplemented!("S-006: ensure_runtime_dir — DirBuilder::new().mode(0o700).recursive(true).create(path)")
}
