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
        // VP-006: `shutdown_utc` must match `YYYY-MM-DDTHH:MM:SS.sssZ` with mandatory
        // millisecond precision (exactly 3 fractional digits). Validated via:
        //   1. Structural length + positional checks using `std` only — no regex crate.
        //      chrono's `%.3f` is permissive (accepts 0-N digits), so we enforce the
        //      exact `.NNN` format structurally before parsing.
        //   2. chrono parse to verify the date/time values are real (e.g., month 13 rejected).
        //
        // chrono is already a workspace production dependency; regex-lite is dev-only per
        // SS-deps-pin-manifest.md v1.2.1 §Trace and must not be used in production code.
        //
        // Expected layout (24 chars): YYYY-MM-DDTHH:MM:SS.MMMZ
        //   [0..4]  year digits
        //   [4]     '-'
        //   [5..7]  month digits
        //   [7]     '-'
        //   [8..10] day digits
        //   [10]    'T'
        //   [11..13] hour digits
        //   [13]    ':'
        //   [14..16] minute digits
        //   [16]    ':'
        //   [17..19] second digits
        //   [19]    '.'
        //   [20..23] millisecond digits (exactly 3)
        //   [23]    'Z'
        let s = &self.shutdown_utc;
        let structurally_valid = s.len() == 24
            && s.as_bytes().get(4) == Some(&b'-')
            && s.as_bytes().get(7) == Some(&b'-')
            && s.as_bytes().get(10) == Some(&b'T')
            && s.as_bytes().get(13) == Some(&b':')
            && s.as_bytes().get(16) == Some(&b':')
            && s.as_bytes().get(19) == Some(&b'.')
            && s.as_bytes().get(23) == Some(&b'Z')
            && s[0..4].bytes().all(|b| b.is_ascii_digit())
            && s[5..7].bytes().all(|b| b.is_ascii_digit())
            && s[8..10].bytes().all(|b| b.is_ascii_digit())
            && s[11..13].bytes().all(|b| b.is_ascii_digit())
            && s[14..16].bytes().all(|b| b.is_ascii_digit())
            && s[17..19].bytes().all(|b| b.is_ascii_digit())
            && s[20..23].bytes().all(|b| b.is_ascii_digit());
        anyhow::ensure!(
            structurally_valid,
            "shutdown_utc '{s}' must match ISO 8601 millisecond format YYYY-MM-DDTHH:MM:SS.sssZ \
             (exactly 3 fractional digits, mandatory Z suffix)",
        );
        // Parse with chrono to validate that the date/time values are real
        // (e.g., month 13, hour 25, or minute 61 are rejected).
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3fZ").map_err(|e| {
            anyhow::anyhow!("shutdown_utc '{s}' contains an invalid date/time value: {e}",)
        })?;
        Ok(())
    }
}

/// Result of reading a recovery checkpoint file.
///
/// Three-state return from [`crate::lifecycle::read_recovery_checkpoint`] that allows
/// callers to distinguish between a missing file (clean boot) and a malformed file
/// (crash mid-write or corruption) — required by EC-054 for differentiated log messages.
///
/// # Non-exhaustive
///
/// `#[non_exhaustive]` follows SS-conventions-anti-patterns.md §"Non-Exhaustive Enum Policy"
/// (S-011): a future additional state (e.g., `PermissionDenied` for an unreadable-but-existing
/// file) is a non-breaking change for downstream crates that pattern-match on this type.
#[derive(Debug)]
#[non_exhaustive]
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

// ---------------------------------------------------------------------------
// S-017 types: daemon start sequence, event bus, hooks-settings
// ---------------------------------------------------------------------------

/// A hook event that has been received from a Claude Code session and enqueued
/// on the daemon-internal event bus (S-017, BC-2.04.001 step 5).
///
/// This is an internal routing type distinct from `monocle_core::hook_events::HookEvent`
/// (which carries the deserialized payload). `EventBusHookEvent` wraps the raw event
/// plus routing metadata for the ring buffer and session registry.
///
/// `#[non_exhaustive]` follows SS-conventions-anti-patterns.md §"Non-Exhaustive Enum
/// Policy": new fields can be added without breaking match sites in Phase 2+.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct EventBusHookEvent {
    /// The deserialized hook payload from the HTTP body.
    pub payload: monocle_core::hook_events::HookEvent,
    /// ISO 8601 UTC timestamp (millisecond precision) when the event was received.
    pub received_at: String,
}

impl EventBusHookEvent {
    /// Construct a new `EventBusHookEvent` with the given payload and reception timestamp.
    pub fn new(payload: monocle_core::hook_events::HookEvent, received_at: String) -> Self {
        Self {
            payload,
            received_at,
        }
    }
}

/// Bounded event bus channel sender for `EventBusHookEvent` (S-017, BC-2.04.001 step 5).
///
/// Created by `tokio::sync::mpsc::channel::<EventBusHookEvent>(EVENT_BUS_CAPACITY)`.
/// The channel capacity MUST be `EVENT_BUS_CAPACITY` (4096). Using an unbounded channel
/// is forbidden by SS-conventions-anti-patterns.md §"No unbounded channels".
pub type EventBusTx = tokio::sync::mpsc::Sender<EventBusHookEvent>;

/// Bounded event bus channel receiver for `EventBusHookEvent` (S-017, BC-2.04.001 step 5).
///
/// Symmetric to [`EventBusTx`]. Stored in `DaemonState` or consumed by the ring-buffer
/// writer task.
pub type EventBusRx = tokio::sync::mpsc::Receiver<EventBusHookEvent>;

/// Capacity of the daemon-internal hook event bus channel (S-017, BC-2.04.001 step 5).
///
/// BC-2.04.001 PC-5 specifies exactly 4096 slots. This constant MUST be used at the
/// call-site of `tokio::sync::mpsc::channel` to guarantee compliance. The drop counter
/// `AtomicU64` is incremented on each send-failure due to the channel being at capacity.
pub const EVENT_BUS_CAPACITY: usize = 4096;

/// Registry of active engine modules registered with the daemon (S-017, BC-2.04.001 step 6).
///
/// Phase 1 always registers exactly two modules:
/// 1. `ClaudeCodeModule` — the primary harness engine.
/// 2. `VsddFactoryAdapter` — factory awareness plane.
///
/// `#[non_exhaustive]` prevents external struct-literal construction so new fields
/// can be added in Phase 3 (wasmtime plugin registry) without breaking callers.
#[non_exhaustive]
#[derive(Debug)]
pub struct EngineModuleRegistry {
    /// Whether the Claude Code engine module has been registered.
    pub claude_code_registered: bool,
    /// Whether the VSDD factory adapter has been registered.
    pub vsdd_factory_registered: bool,
}

impl EngineModuleRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self {
            claude_code_registered: false,
            vsdd_factory_registered: false,
        }
    }

    /// Register the Claude Code engine module.
    pub fn register_claude_code(&mut self) {
        self.claude_code_registered = true;
    }

    /// Register the VSDD factory adapter.
    pub fn register_vsdd_factory(&mut self) {
        self.vsdd_factory_registered = true;
    }

    /// Returns `true` if all required Phase 1 modules are registered.
    pub fn is_complete(&self) -> bool {
        self.claude_code_registered && self.vsdd_factory_registered
    }
}

impl Default for EngineModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
