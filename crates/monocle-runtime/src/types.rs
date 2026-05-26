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

impl RecoveryCheckpoint {
    /// Validate BC-2.01.006 INV-1 field constraints.
    ///
    /// Invariants enforced:
    /// - `pid >= 1`: PID 0 is not a valid process identifier on POSIX systems.
    /// - `last_app_mode` is non-empty: the mode string must name the actual AppMode.
    /// - `shutdown_utc` matches `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`:
    ///   ISO 8601 UTC with mandatory 3-digit milliseconds.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.pid >= 1, "pid must be >= 1, got {}", self.pid);
        anyhow::ensure!(
            !self.last_app_mode.is_empty(),
            "last_app_mode must be non-empty"
        );
        // VP-006 regex: YYYY-MM-DDTHH:MM:SS.sssZ (mandatory millisecond precision).
        // Compiled once via OnceLock to avoid per-call allocation and to satisfy
        // the clippy::expect_used lint (the pattern is a hardcoded constant; if it
        // fails to compile the binary is fatally broken at startup, not at runtime).
        static SHUTDOWN_UTC_RE: std::sync::OnceLock<regex_lite::Regex> =
            std::sync::OnceLock::new();
        let re = SHUTDOWN_UTC_RE.get_or_init(|| {
            regex_lite::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$")
                .unwrap_or_else(|e| {
                    // This branch is unreachable in practice — the pattern is a compile-time
                    // constant. Panic here is intentional: a broken regex means the binary
                    // should not start, not silently accept invalid timestamps.
                    panic!("shutdown_utc validation regex failed to compile: {e}")
                })
        });
        anyhow::ensure!(
            re.is_match(&self.shutdown_utc),
            "shutdown_utc must match ISO 8601 millisecond format YYYY-MM-DDTHH:MM:SS.sssZ, got '{}'",
            self.shutdown_utc
        );
        Ok(())
    }
}

/// Result of reading a recovery checkpoint file.
///
/// Three-state return from [`crate::lifecycle::read_recovery_checkpoint`] that allows
/// callers to distinguish between a missing file (clean boot) and a malformed file
/// (crash mid-write or corruption) — required by EC-054 for differentiated log messages.
pub enum CheckpointReadResult {
    /// File exists and contains a valid, field-validated checkpoint.
    Valid(RecoveryCheckpoint),
    /// File exists but is malformed (truncated, invalid JSON, or fails INV-1 validation).
    Malformed,
    /// File does not exist (clean boot / graceful shutdown).
    Absent,
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

impl std::fmt::Display for ShutdownReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Graceful => write!(f, "graceful"),
            Self::Signal => write!(f, "signal"),
            Self::Forced => write!(f, "forced"),
        }
    }
}
