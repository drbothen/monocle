//! Reconnection loop with exponential backoff for the TUI-to-daemon IPC connection
//! (S-023, BC-2.05.006).
//!
//! Entry point: [`reconnect`]. Called by the TUI event loop after the SOQ-3 overlay-clear
//! handler has run (BC-2.05.007). The reconnect loop:
//!
//! 1. Re-reads `<runtime_dir>/monocle.lock` after each failed attempt to discover a
//!    restarted daemon with a new PID, port, or authToken (BC-2.05.006 PC-3).
//! 2. Retries with exponential backoff: 250ms → 500ms → 1000ms → 2000ms cap (PC-4).
//! 3. Times out after 5 seconds from first disconnect detection (PC-5).
//! 4. On success: returns the connected [`crate::uds::UdsClientTransport`] so the caller
//!    can receive the fresh `InitialState` push (BC-2.05.002).
//! 5. On timeout: returns [`crate::error::IpcError::ReconnectTimeout`] and the caller
//!    enters offline mode (5-second lock-file poll).
//!
//! # Backoff Schedule (BC-2.05.006 PC-4)
//!
//! | Attempt | Pre-retry wait |
//! |---------|---------------|
//! | 1       | 250ms         |
//! | 2       | 500ms         |
//! | 3       | 1000ms        |
//! | 4+      | 2000ms (cap)  |
//!
//! # Offline Mode (BC-2.05.006 PC-5)
//!
//! After `IpcError::ReconnectTimeout`, the TUI polls
//! `<runtime_dir>/monocle.lock` every 5 seconds. When a new lock file is detected, it
//! re-enters the reconnect loop with a fresh `BackoffState` (backoff resets to 250ms).

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::IpcError;
use crate::uds::UdsClientTransport;

// ---------------------------------------------------------------------------
// Backoff constants (BC-2.05.006 PC-4)
// ---------------------------------------------------------------------------

/// First retry wait: 250ms (BC-2.05.006 PC-4 Attempt 1).
pub const BACKOFF_INITIAL_MS: u64 = 250;

/// Backoff cap: 2000ms (BC-2.05.006 PC-4 Attempt 4+).
pub const BACKOFF_CAP_MS: u64 = 2_000;

/// Total reconnect window before transitioning to offline mode (BC-2.05.006 PC-5).
pub const RECONNECT_WINDOW_SECS: u64 = 5;

/// Offline mode lock-file poll interval (BC-2.05.006 PC-5).
///
/// Distinct from the auto-start 100ms poll interval (S-019). The offline poll is
/// intentionally coarser: the TUI is in passive mode and low-frequency polling is
/// sufficient to discover a restarted daemon.
pub const OFFLINE_POLL_INTERVAL_SECS: u64 = 5;

// ---------------------------------------------------------------------------
// BackoffState
// ---------------------------------------------------------------------------

/// Mutable state for the exponential backoff loop (BC-2.05.006 PC-4).
///
/// Tracks the current retry attempt number. [`BackoffState::next_delay`] computes the
/// correct pre-retry wait duration for each attempt and increments the counter.
///
/// # Reset
///
/// A `BackoffState` is per-reconnect-session. When the TUI enters offline mode and
/// later detects a new lock file, it creates a fresh `BackoffState::new()` (backoff resets
/// to 250ms — BC-2.05.006 PC-5: "re-enters the reconnect loop from the beginning").
#[derive(Debug, Clone)]
pub struct BackoffState {
    /// Number of reconnect attempts made so far (0-indexed).
    attempt: u32,
}

impl BackoffState {
    /// Create a fresh `BackoffState` for the start of a reconnect session.
    ///
    /// `attempt` starts at 0; the first call to [`Self::next_delay`] returns the
    /// Attempt 1 delay (250ms) and increments to 1.
    pub fn new() -> Self {
        Self { attempt: 0 }
    }

    /// Return the pre-retry delay for the current attempt and advance the counter.
    ///
    /// # Backoff schedule (BC-2.05.006 PC-4)
    ///
    /// | `attempt` before call | Returned delay |
    /// |----------------------|----------------|
    /// | 0                    | 250ms          |
    /// | 1                    | 500ms          |
    /// | 2                    | 1000ms         |
    /// | 3+                   | 2000ms (cap)   |
    pub fn next_delay(&mut self) -> Duration {
        todo!("S-023: compute exponential backoff delay for attempt {}", self.attempt)
    }
}

impl Default for BackoffState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Public reconnect entry point
// ---------------------------------------------------------------------------

/// Reconnect to the daemon with exponential backoff (BC-2.05.006).
///
/// Called by the TUI event loop **after** the SOQ-3 overlay-clear handler has completed
/// (BC-2.05.007 Invariant 1: clear happens before reconnect loop begins).
///
/// # Arguments
///
/// - `runtime_dir`: the runtime directory containing `monocle.lock` and `monocle.sock`.
/// - `backoff`: mutable backoff state. Pass `BackoffState::new()` for the initial
///   reconnect attempt. Reuse across retries within the same reconnect session so the
///   delay schedule advances correctly.
///
/// # Returns
///
/// - `Ok(UdsClientTransport)` — a fresh connected transport. The caller MUST await a
///   `ServerToClient::InitialState` push to rebuild TUI state (BC-2.05.002).
/// - `Err(IpcError::ReconnectTimeout)` — the 5-second window was exhausted without a
///   successful connection. The TUI MUST enter offline mode (passive observe-only;
///   poll lock file every 5 seconds).
///
/// # Reconnect Loop Invariants
///
/// 1. Lock file is re-read after EACH failed attempt (not only on window expiry).
/// 2. If the lock file changes between retries (new `pid`, `port`, `authToken`),
///    subsequent attempts use the updated socket path.
/// 3. The reconnect loop never starts before the SOQ-3 clear completes (enforced by
///    the caller contract, not inside this function — the function assumes SOQ-3 has run).
///
/// # Logging
///
/// - `DEBUG` on each reconnect attempt (with attempt number and runtime_dir).
/// - `INFO` on successful reconnect.
/// - `WARN` on 5-second timeout and offline mode entry.
///
/// # Errors
///
/// Returns [`IpcError::ReconnectTimeout`] when no connection succeeds within 5 seconds.
pub async fn reconnect(
    runtime_dir: &Path,
    backoff: &mut BackoffState,
) -> Result<UdsClientTransport, IpcError> {
    todo!(
        "S-023: implement reconnect loop with exponential backoff for runtime_dir={:?}",
        runtime_dir
    )
}

// ---------------------------------------------------------------------------
// Lock-file re-read helper
// ---------------------------------------------------------------------------

/// Re-read `<runtime_dir>/monocle.lock` and return the socket path for the next attempt.
///
/// Called after each failed reconnect attempt (BC-2.05.006 PC-3). If the lock file has
/// changed since the last read (new `pid`, `port`, or `authToken`), the returned
/// `PathBuf` reflects the updated socket path, enabling the TUI to discover a restarted
/// daemon that may be bound to a different port.
///
/// # Errors
///
/// Returns [`IpcError::IoError`] if the lock file cannot be read.
/// Returns [`IpcError::SerializeError`] if the lock file JSON is malformed.
#[allow(dead_code)]
pub(crate) async fn read_lock_file_sock_path(
    runtime_dir: &Path,
) -> Result<PathBuf, IpcError> {
    todo!(
        "S-023: read monocle.lock from {:?} and return resolved socket path",
        runtime_dir
    )
}

// ---------------------------------------------------------------------------
// Offline poll helper
// ---------------------------------------------------------------------------

/// Poll `<runtime_dir>/monocle.lock` every [`OFFLINE_POLL_INTERVAL_SECS`] seconds until
/// a new lock file is detected, then return the runtime directory for a fresh reconnect
/// loop.
///
/// Called when [`reconnect`] returns `IpcError::ReconnectTimeout`. The TUI enters passive
/// mode and this function drives the 5-second lock-file poll. When it returns, the caller
/// MUST call [`reconnect`] again with a fresh [`BackoffState::new()`] (backoff resets).
///
/// # Lock-file change detection
///
/// A "new lock file" is one whose `pid` field differs from the last observed `pid`. The
/// PID is the stable discriminant: the same daemon process cannot crash and restart without
/// changing its PID.
///
/// # Cancellation
///
/// The returned future is cancel-safe: dropping it before it resolves leaves no
/// background tasks running. The TUI event loop may use `tokio::select!` to cancel if the
/// user quits.
///
/// # Errors
///
/// This function does not return errors — transient lock-file read failures are logged
/// at `DEBUG` level and the poll continues.
#[allow(dead_code)]
pub(crate) async fn poll_for_new_daemon(runtime_dir: &Path) {
    todo!(
        "S-023: poll {:?}/monocle.lock every {}s for new daemon PID",
        runtime_dir,
        OFFLINE_POLL_INTERVAL_SECS
    )
}
