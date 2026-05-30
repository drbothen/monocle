//! IPC error taxonomy for monocle-ipc (BC-2.05.008).
//!
//! `IpcError` is the canonical error type for all monocle-ipc operations:
//! framing, serialization, socket I/O, and connection lifecycle.

/// Errors that can occur during IPC operations.
///
/// Used as the `Err` variant for [`crate::transport::Transport`] methods,
/// [`crate::framing::write_framed`], and [`crate::framing::read_framed`].
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// The serialized message payload exceeds the 256 KiB (262,144 byte) framing limit.
    ///
    /// Raised by `write_framed` / `broadcast_*` methods when the serialized JSON message
    /// is larger than `MAX_MESSAGE_BYTES` (262,144).  The message is NOT sent.
    #[error("IPC message exceeds 256 KiB limit")]
    MessageTooLarge,

    /// JSON serialization or deserialization failed.
    ///
    /// Wraps the underlying `serde_json::Error` for context-preserving error chains.
    #[error("IPC serialization error: {0}")]
    SerializeError(#[from] serde_json::Error),

    /// An I/O error occurred on the underlying socket or file.
    ///
    /// Wraps `std::io::Error`. Common causes: socket closed unexpectedly, filesystem
    /// error during socket bind/remove, or OS-level send buffer exhaustion.
    #[error("IPC I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The IPC connection has been closed by the remote peer.
    ///
    /// Returned by `recv_message` when the remote peer closes the connection cleanly
    /// (EOF on the socket read path). Callers should treat this as a graceful disconnect
    /// and remove the peer from any fan-out subscriber list.
    #[error("IPC connection disconnected")]
    Disconnected,

    /// The computed UDS socket path exceeds the OS-imposed path length limit.
    ///
    /// POSIX UDS path limit is 104 bytes on macOS / 108 bytes on Linux.
    /// Raised by `UdsTransport::bind` during path validation before any bind attempt.
    /// Logged as `ERROR: UDS socket path exceeds OS limit (<N> bytes, limit <M>)`.
    #[error("UDS socket path exceeds OS limit ({length} bytes, limit {limit})")]
    PathTooLong {
        /// Actual byte length of the computed path.
        length: usize,
        /// Platform-specific maximum UDS path length in bytes.
        limit: usize,
    },

    /// `UnixListener::bind` failed after stale-socket removal.
    ///
    /// Wraps the underlying `std::io::Error`. Logged as
    /// `ERROR: failed to bind UDS socket at <path>: <reason>`.
    ///
    /// # Extension note
    ///
    /// This variant is not in the BC-2.05.008 base set but is required for operational
    /// correctness: without it, bind failures collapse into the generic `IoError` variant
    /// and lose the specific context needed for diagnostics. Added as an operational
    /// extension per the production-grade default.
    #[error("UDS bind failed: {0}")]
    BindFailure(std::io::Error),

    /// The 5-second reconnect window was exhausted without a successful connection.
    ///
    /// Returned by [`crate::reconnect::reconnect`] when no connection to the daemon
    /// could be established within [`crate::reconnect::RECONNECT_WINDOW_SECS`] seconds
    /// of the initial disconnect detection (BC-2.05.006 PC-5).
    ///
    /// On receipt the TUI MUST:
    /// 1. Render `[daemon: offline]` in the status bar.
    /// 2. Enter passive observe-only mode (no IPC push messages received).
    /// 3. Poll `<runtime_dir>/monocle.lock` every
    ///    [`crate::reconnect::OFFLINE_POLL_INTERVAL_SECS`] seconds.
    /// 4. Re-enter the reconnect loop when a new lock file is detected.
    #[error("daemon reconnect timed out after 5 seconds — entering offline mode")]
    ReconnectTimeout,
}
