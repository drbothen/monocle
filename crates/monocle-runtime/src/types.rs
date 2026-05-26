//! Shared runtime types for the monocle daemon.
//!
//! Populated by S-007 (Crash Recovery Checkpoint). Types defined here are used by
//! [`crate::lifecycle`] for checkpoint write/read operations.

/// Crash recovery checkpoint written during the drain sequence.
///
/// Serialised to JSON via `tempfile::persist` so the TUI can detect unclean shutdowns
/// on next attach. BC-2.01.006 defines the postcondition that a checkpoint file MUST
/// exist after any non-graceful termination path.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecoveryCheckpoint {
    /// PID of the daemon process that wrote this checkpoint.
    pub pid: u32,
    /// Reason the daemon is shutting down when the checkpoint is written.
    pub shutdown_reason: ShutdownReason,
    /// String representation of the `AppMode` at shutdown time (e.g., `"Running"`).
    pub last_app_mode: String,
    /// RFC 3339 / ISO 8601 UTC timestamp when the checkpoint was written.
    ///
    /// Format: `YYYY-MM-DDTHH:MM:SS.sssZ` with mandatory millisecond precision,
    /// consistent with `LastHookTimestamps` field conventions (BC-2.01.002 PC-1).
    pub shutdown_utc: String,
}

/// Reason the daemon is shutting down — written into [`RecoveryCheckpoint`].
///
/// BC-2.01.006 invariant 1: every checkpoint MUST carry a `ShutdownReason`.
///
/// # Serialisation
///
/// `#[serde(rename_all = "lowercase")]` ensures the wire format is stable lowercase
/// strings (`"graceful"`, `"signal"`, `"forced"`), independent of any future Rust
/// variant renaming.
///
/// # Non-exhaustive
///
/// `#[non_exhaustive]` follows SS-conventions-anti-patterns.md §"Non-Exhaustive Enum
/// Policy" (S-011): any future addition of a shutdown reason variant (e.g., `Watchdog`)
/// is a non-breaking change for downstream crates that match on `ShutdownReason`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ShutdownReason {
    /// Daemon exited cleanly; all in-flight requests drained within the 10-second window.
    Graceful,
    /// Daemon received a POSIX signal (SIGINT or SIGTERM) that initiated shutdown.
    Signal,
    /// Drain timed out or a second shutdown request forced immediate termination.
    Forced,
}
