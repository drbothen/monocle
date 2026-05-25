//! Error types for the monocle daemon startup and lock-file lifecycle.
//!
//! Populated by S-006 (Lock File Atomic Lifecycle).
//!
//! All error variants conform to the production-grade error taxonomy requirement in
//! SS-conventions-anti-patterns.md: every public API must expose a typed error enum
//! derived via `thiserror::Error`.

use thiserror::Error;

/// Errors that can occur during daemon startup — runtime directory resolution, lock
/// file acquisition, and related filesystem operations.
///
/// Returned by [`crate::lifecycle::resolve_runtime_dir`],
/// [`crate::lifecycle::ensure_runtime_dir`], and [`crate::lock::DaemonLock::acquire`].
#[derive(Debug, Error)]
pub enum DaemonStartError {
    /// The runtime directory could not be resolved from either the
    /// `MONOCLE_RUNTIME_DIR` environment variable or `directories::ProjectDirs`.
    ///
    /// This can occur on platforms where `ProjectDirs::new(...)` returns `None`
    /// (e.g., when `$HOME` is unset on Unix) and `MONOCLE_RUNTIME_DIR` is also absent.
    #[error(
        "cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path"
    )]
    RuntimeDirUnresolvable,

    /// A live process already holds the lock file at the path.
    ///
    /// `pid` is the PID recorded in the existing lock file. The caller must surface
    /// this to the user and exit without creating a second daemon instance (BC-2.03.001).
    #[error("daemon already running with PID {pid}; lock file conflict")]
    LockFileConflict {
        /// PID of the existing live daemon process recorded in the lock file.
        pid: i32,
    },

    /// The lock file could not be written or persisted atomically.
    ///
    /// Wraps the underlying `std::io::Error` from the `tempfile::persist` call or the
    /// temporary-file creation step.
    #[error("failed to write lock file: {0}")]
    LockFileWriteFailure(#[from] std::io::Error),

    /// The runtime directory could not be created.
    ///
    /// Wraps the underlying `std::io::Error` from the `DirBuilder` call.
    /// Note: this variant is NOT covered by the `#[from] std::io::Error` on
    /// `LockFileWriteFailure`; callers in `ensure_runtime_dir` construct this variant
    /// directly to distinguish the error site.
    #[error("failed to create runtime directory: {0}")]
    RuntimeDirCreateFailure(std::io::Error),
}
