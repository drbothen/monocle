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
//!
//! # Lock File Format
//!
//! The lock file is JSON with at minimum the following fields:
//! ```json
//! {
//!   "pid": 12345,
//!   "port": 9001,
//!   "authToken": "...",
//!   "socketPath": "/path/to/monocle.sock"
//! }
//! ```
//!
//! The `socketPath` field is used directly for reconnect attempts. The `pid` field
//! is used as the stable discriminant for detecting a new daemon in [`poll_for_new_daemon`].

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
        // Backoff schedule (BC-2.05.006 PC-4):
        // Attempt 0 → 250ms, Attempt 1 → 500ms, Attempt 2 → 1000ms, Attempt 3+ → 2000ms cap.
        let delay_ms = match self.attempt {
            0 => BACKOFF_INITIAL_MS,     // 250ms
            1 => BACKOFF_INITIAL_MS * 2, // 500ms
            2 => BACKOFF_INITIAL_MS * 4, // 1000ms
            _ => BACKOFF_CAP_MS,         // 2000ms cap
        };
        self.attempt = self.attempt.saturating_add(1);
        Duration::from_millis(delay_ms)
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
    let window = Duration::from_secs(RECONNECT_WINDOW_SECS);
    let deadline = tokio::time::Instant::now() + window;
    let mut attempt = 0u32;

    loop {
        // Read current socket path from lock file (BC-2.05.006 PC-3).
        // On first attempt OR after each failed attempt, re-read to discover daemon restart.
        let sock_path = match read_lock_file_sock_path(runtime_dir).await {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!(
                    attempt,
                    runtime_dir = %runtime_dir.display(),
                    error = %e,
                    "reconnect: lock file unavailable, will retry after backoff"
                );
                // No socket path — use default path as fallback so we can still
                // attempt the connection (the daemon may have started by the time sleep ends).
                runtime_dir.join("monocle.sock")
            }
        };

        // Check if we've already exceeded the window BEFORE the backoff sleep.
        // This handles the case where previous attempts consumed most of the 5s window.
        if tokio::time::Instant::now() >= deadline {
            tracing::warn!(
                runtime_dir = %runtime_dir.display(),
                "reconnect: 5-second window exhausted — entering offline mode (BC-2.05.006 PC-5)"
            );
            return Err(IpcError::ReconnectTimeout);
        }

        attempt = attempt.saturating_add(1);
        let delay = backoff.next_delay();

        tracing::debug!(
            attempt,
            runtime_dir = %runtime_dir.display(),
            delay_ms = delay.as_millis(),
            sock_path = %sock_path.display(),
            "reconnect: attempt {attempt} — waiting {delay_ms}ms before trying to connect",
            delay_ms = delay.as_millis()
        );

        // Wait for the backoff delay. tokio::time::sleep is compatible with mock time
        // (tokio::time::pause() + advance()) used in tests (BC-2.05.006 PC-4).
        // MUST NOT use std::thread::sleep here (blocks the Tokio runtime).
        tokio::time::sleep(delay).await;

        // Re-check deadline after sleep (the sleep itself may have consumed time).
        if tokio::time::Instant::now() >= deadline {
            tracing::warn!(
                runtime_dir = %runtime_dir.display(),
                "reconnect: 5-second window exhausted after backoff sleep — entering offline mode"
            );
            return Err(IpcError::ReconnectTimeout);
        }

        // Attempt connection.
        match tokio::net::UnixStream::connect(&sock_path).await {
            Ok(stream) => {
                tracing::info!(
                    attempt,
                    runtime_dir = %runtime_dir.display(),
                    sock_path = %sock_path.display(),
                    "reconnect: connection succeeded on attempt {attempt}"
                );
                return Ok(UdsClientTransport::new(stream));
            }
            Err(e) => {
                tracing::debug!(
                    attempt,
                    runtime_dir = %runtime_dir.display(),
                    error = %e,
                    "reconnect: connection attempt {attempt} failed — will re-read lock file"
                );
                // Re-read lock file before next attempt (BC-2.05.006 PC-3 — done at top of loop).
                // Continue loop for next attempt.
            }
        }
    }
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
/// # Lock file format
///
/// The lock file is JSON with a `"socketPath"` field containing the absolute path to the
/// daemon's UDS socket. If `"socketPath"` is absent, falls back to
/// `<runtime_dir>/monocle.sock` as the canonical default.
///
/// # Errors
///
/// Returns [`IpcError::IoError`] if the lock file cannot be read.
/// Returns [`IpcError::SerializeError`] if the lock file JSON is malformed.
pub async fn read_lock_file_sock_path(runtime_dir: &Path) -> Result<PathBuf, IpcError> {
    let lock_path = runtime_dir.join("monocle.lock");
    // Use tokio::fs for async I/O (does not block the runtime on large lock files).
    let contents = tokio::fs::read_to_string(&lock_path)
        .await
        .map_err(IpcError::IoError)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;
    // Extract the socketPath field if present; otherwise default to runtime_dir/monocle.sock.
    let sock_path = json
        .get("socketPath")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| runtime_dir.join("monocle.sock"));
    Ok(sock_path)
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
pub async fn poll_for_new_daemon(runtime_dir: &Path) {
    let lock_path = runtime_dir.join("monocle.lock");
    // Read the initial PID to detect when a NEW daemon starts.
    let initial_pid = read_lock_pid(&lock_path).await;
    tracing::debug!(
        runtime_dir = %runtime_dir.display(),
        initial_pid,
        "offline mode: polling for new daemon every {}s (BC-2.05.006 PC-5)",
        OFFLINE_POLL_INTERVAL_SECS
    );

    loop {
        // Poll interval is 5 seconds (BC-2.05.006 PC-5 — NOT the 100ms auto-start interval).
        tokio::time::sleep(Duration::from_secs(OFFLINE_POLL_INTERVAL_SECS)).await;

        let current_pid = read_lock_pid(&lock_path).await;
        if let (Some(initial), Some(current)) = (initial_pid, current_pid) {
            if current != initial {
                tracing::info!(
                    runtime_dir = %runtime_dir.display(),
                    old_pid = initial,
                    new_pid = current,
                    "offline mode: new daemon detected (PID changed) — re-entering reconnect loop"
                );
                return;
            }
        } else if initial_pid.is_none() && current_pid.is_some() {
            // Lock file appeared (no initial PID → new daemon).
            tracing::info!(
                runtime_dir = %runtime_dir.display(),
                new_pid = current_pid,
                "offline mode: lock file appeared — re-entering reconnect loop"
            );
            return;
        }
        tracing::debug!(
            runtime_dir = %runtime_dir.display(),
            current_pid,
            "offline mode: no new daemon detected, continuing poll"
        );
    }
}

/// Read the `pid` field from the lock file, returning `None` if not readable.
///
/// Used by [`poll_for_new_daemon`] for change detection.
async fn read_lock_pid(lock_path: &Path) -> Option<u64> {
    let contents = tokio::fs::read_to_string(lock_path).await.ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    json.get("pid").and_then(|v| v.as_u64())
}
