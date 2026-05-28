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
//! # Lock File Format and UDS Socket Path Convention
//!
//! The lock file is JSON with at minimum the following fields:
//! ```json
//! {
//!   "pid": 12345,
//!   "port": 9001,
//!   "authToken": "..."
//! }
//! ```
//!
//! The reconnect loop reads `(pid, port, authToken)` from the lock file as a
//! change-detection discriminant (BC-2.05.006 PC-3). The daemon's UDS socket path is
//! NOT stored in the lock file — the canonical path is always
//! `<runtime_dir>/monocle.sock` by convention (established by SS-ipc and SS-daemon-wiring).
//! This is the same path the daemon binds to in [`crate::uds::UdsTransport::bind`].
//! [`canonical_sock_path`] encodes this convention for all reconnect call sites.
//!
//! The `pid` field is the stable discriminant for detecting a new daemon in
//! [`poll_for_new_daemon`].

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::IpcError;
use crate::uds::{EventReceiver, UdsClientTransport};

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
///
/// # ADR-0006
///
/// `#[non_exhaustive]` is required per ADR-0006 for all public structs with a `new()`
/// constructor to prevent out-of-crate struct literal construction.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct BackoffState {
    /// Number of reconnect attempts made so far (0-indexed).
    attempt: u32,
    /// Consecutive lock-file read failures in the current reconnect session.
    ///
    /// Reset to 0 on the first successful lock-file read. Used for telemetry escalation
    /// (F-S023-ADV1-MED-002): WARN once after ≥2 consecutive failures; ERROR before
    /// returning `IpcError::ReconnectTimeout` when ALL lock reads in the session failed.
    lock_read_failures: u32,
}

impl BackoffState {
    /// Create a fresh `BackoffState` for the start of a reconnect session.
    ///
    /// `attempt` starts at 0; the first call to [`Self::next_delay`] returns the
    /// Attempt 1 delay (250ms) and increments to 1.
    pub fn new() -> Self {
        Self {
            attempt: 0,
            lock_read_failures: 0,
        }
    }

    /// Return the pre-retry delay for the current attempt and advance the counter.
    ///
    /// # Backoff schedule (BC-2.05.006 PC-4)
    ///
    /// | Attempt number (1-indexed, matches BC-2.05.006 AC-009) | Returned delay |
    /// |--------------------------------------------------------|----------------|
    /// | 1                                                      | 250ms          |
    /// | 2                                                      | 500ms          |
    /// | 3                                                      | 1000ms         |
    /// | 4+                                                     | 2000ms (cap)   |
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
/// - `Ok((UdsClientTransport, EventReceiver))` — a fresh connected transport and its
///   associated disconnect event channel. The receiver MUST be drained by the consumer
///   to observe `Disconnected` events on subsequent disconnects (2nd SOQ-3 cycle).
///   The caller MUST await a `ServerToClient::InitialState` push to rebuild TUI state
///   (BC-2.05.002).
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
/// 4. Each `connect` attempt is bounded by the remaining reconnect window via
///    `tokio::time::timeout` — a hung connect cannot exceed the 5-second deadline
///    (BC-2.05.006 PC-5 hard window requirement).
///
/// # Logging
///
/// - `DEBUG` on each reconnect attempt (with attempt number and runtime_dir).
/// - `INFO` on successful reconnect.
/// - `WARN` on 5-second timeout and offline mode entry.
/// - `WARN` after ≥2 consecutive lock-file read failures.
/// - `ERROR` before returning `ReconnectTimeout` when all lock reads in the session failed.
///
/// # Errors
///
/// Returns [`IpcError::ReconnectTimeout`] when no connection succeeds within 5 seconds.
pub async fn reconnect(
    runtime_dir: &Path,
    backoff: &mut BackoffState,
) -> Result<(UdsClientTransport, EventReceiver), IpcError> {
    let window = Duration::from_secs(RECONNECT_WINDOW_SECS);
    let deadline = tokio::time::Instant::now() + window;
    let mut attempt = 0u32;

    loop {
        // Re-read the lock file after each failed attempt to detect a restarted daemon
        // (BC-2.05.006 PC-3). The lock file provides the change-detection discriminant
        // (pid, port, authToken); the socket path is always the canonical convention path.
        let sock_path = match read_lock_discriminant(&runtime_dir.join("monocle.lock")).await {
            Some(_discriminant) => {
                // Successful read — reset consecutive failure counter.
                backoff.lock_read_failures = 0;
                canonical_sock_path(runtime_dir)
            }
            None => {
                backoff.lock_read_failures = backoff.lock_read_failures.saturating_add(1);
                if backoff.lock_read_failures == 2 {
                    tracing::warn!(
                        attempt,
                        runtime_dir = %runtime_dir.display(),
                        consecutive_failures = backoff.lock_read_failures,
                        "reconnect: repeated lock file read failures — daemon may have removed lock"
                    );
                } else {
                    tracing::debug!(
                        attempt,
                        runtime_dir = %runtime_dir.display(),
                        "reconnect: lock file unavailable, will retry after backoff"
                    );
                }
                // Lock file absent — use the canonical path anyway; the daemon may have
                // started by the time the backoff sleep completes.
                canonical_sock_path(runtime_dir)
            }
        };

        // Check if we've already exceeded the window BEFORE the backoff sleep.
        // This handles the case where previous attempts consumed most of the 5s window.
        if tokio::time::Instant::now() >= deadline {
            if backoff.lock_read_failures > 0 {
                tracing::error!(
                    runtime_dir = %runtime_dir.display(),
                    consecutive_lock_failures = backoff.lock_read_failures,
                    "reconnect: 5-second window exhausted with all lock reads failing — \
                     daemon likely removed lock file before shutdown"
                );
            } else {
                tracing::warn!(
                    runtime_dir = %runtime_dir.display(),
                    "reconnect: 5-second window exhausted — entering offline mode (BC-2.05.006 PC-5)"
                );
            }
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
            if backoff.lock_read_failures > 0 {
                tracing::error!(
                    runtime_dir = %runtime_dir.display(),
                    consecutive_lock_failures = backoff.lock_read_failures,
                    "reconnect: 5-second window exhausted after sleep with all lock reads failing"
                );
            } else {
                tracing::warn!(
                    runtime_dir = %runtime_dir.display(),
                    "reconnect: 5-second window exhausted after backoff sleep — entering offline mode"
                );
            }
            return Err(IpcError::ReconnectTimeout);
        }

        // Attempt connection — bounded by the remaining window (BC-2.05.006 PC-5 hard deadline).
        // A hung connect() on a path with no listener would otherwise exceed the 5-second window.
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let connect_result =
            tokio::time::timeout(remaining, tokio::net::UnixStream::connect(&sock_path)).await;

        match connect_result {
            Ok(Ok(stream)) => {
                tracing::info!(
                    attempt,
                    runtime_dir = %runtime_dir.display(),
                    sock_path = %sock_path.display(),
                    "reconnect: connection succeeded on attempt {attempt}"
                );
                // Use connect_with_events to get a fresh event channel for this connection.
                // The consumer MUST drain event_rx to observe Disconnected on subsequent
                // disconnects (2nd SOQ-3 cycle — AC-EC-004).
                let (transport, event_rx) =
                    crate::uds::connect_with_events_from_stream(stream).await;
                return Ok((transport, event_rx));
            }
            Ok(Err(e)) => {
                tracing::debug!(
                    attempt,
                    runtime_dir = %runtime_dir.display(),
                    error = %e,
                    "reconnect: connection attempt {attempt} failed — will re-read lock file"
                );
                // Re-read lock file before next attempt (BC-2.05.006 PC-3 — done at top of loop).
                // Continue loop for next attempt.
            }
            Err(_elapsed) => {
                // tokio::time::timeout elapsed — the connect() hung past the remaining window.
                tracing::warn!(
                    attempt,
                    runtime_dir = %runtime_dir.display(),
                    sock_path = %sock_path.display(),
                    "reconnect: connect() timed out within remaining window — entering offline mode"
                );
                return Err(IpcError::ReconnectTimeout);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Socket path convention helper
// ---------------------------------------------------------------------------

/// Return the canonical UDS socket path for the monocle daemon in the given runtime directory.
///
/// The daemon always binds at `<runtime_dir>/monocle.sock` (established by
/// [`crate::uds::UdsTransport::bind`] and codified in SS-ipc / SS-daemon-wiring).
/// The lock file does NOT contain a `socketPath` field — that field was a lock-file
/// shape mismatch removed in S-023. Reconnect call sites MUST use this helper rather
/// than constructing the path inline.
///
/// # Lock file and socket path separation
///
/// The lock file fields `(pid, port, authToken)` serve as a change-detection discriminant
/// for detecting daemon restarts (BC-2.05.006 PC-3). The socket path is a stable convention
/// that does not change between daemon restarts within the same runtime directory.
/// These concerns are intentionally separated.
pub fn canonical_sock_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("monocle.sock")
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
/// # Lock-file change detection (BC-2.05.006 PC-3 / EC-003)
///
/// A "new lock file" is one whose `(pid, port, auth_token)` tuple differs from the last
/// observed value. Using all three fields ensures detection of same-path daemon restarts
/// where only the `authToken` changed (e.g., daemon process kept the same PID after a
/// rapid restart edge case), matching the full set of change indicators in BC-2.05.006
/// PC-3 / EC-003.
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
    // Read the initial (pid, port, authToken) discriminant to detect when a NEW daemon starts.
    let initial_discriminant = read_lock_discriminant(&lock_path).await;
    tracing::debug!(
        runtime_dir = %runtime_dir.display(),
        initial_pid = initial_discriminant.as_ref().map(|(pid, _, _)| *pid),
        "offline mode: polling for new daemon every {}s (BC-2.05.006 PC-5)",
        OFFLINE_POLL_INTERVAL_SECS
    );

    loop {
        // Poll interval is 5 seconds (BC-2.05.006 PC-5 — NOT the 100ms auto-start interval).
        tokio::time::sleep(Duration::from_secs(OFFLINE_POLL_INTERVAL_SECS)).await;

        let current_discriminant = read_lock_discriminant(&lock_path).await;
        match (&initial_discriminant, &current_discriminant) {
            (Some(initial), Some(current)) if current != initial => {
                let (old_pid, old_port, _) = initial;
                let (new_pid, new_port, _) = current;
                tracing::info!(
                    runtime_dir = %runtime_dir.display(),
                    old_pid,
                    old_port,
                    new_pid,
                    new_port,
                    "offline mode: new daemon detected (pid/port/authToken changed) — \
                     re-entering reconnect loop"
                );
                return;
            }
            (None, Some((new_pid, new_port, _))) => {
                // Lock file appeared (no initial discriminant → new daemon).
                tracing::info!(
                    runtime_dir = %runtime_dir.display(),
                    new_pid,
                    new_port,
                    "offline mode: lock file appeared — re-entering reconnect loop"
                );
                return;
            }
            _ => {
                tracing::debug!(
                    runtime_dir = %runtime_dir.display(),
                    current_pid = current_discriminant.as_ref().map(|(pid, _, _)| *pid),
                    "offline mode: no new daemon detected, continuing poll"
                );
            }
        }
    }
}

/// Read the `(pid, port, authToken)` discriminant from the lock file.
///
/// Used by [`poll_for_new_daemon`] for change detection (BC-2.05.006 PC-3 / EC-003).
/// Returns `None` if the lock file is absent or malformed.
async fn read_lock_discriminant(lock_path: &Path) -> Option<(u64, u64, String)> {
    let contents = tokio::fs::read_to_string(lock_path).await.ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let pid = json.get("pid").and_then(|v| v.as_u64())?;
    let port = json.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
    let auth_token = json
        .get("authToken")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Some((pid, port, auth_token))
}
