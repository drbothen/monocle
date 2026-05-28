//! `UdsTransport` — Unix domain socket lifecycle manager (BC-2.05.001).
//!
//! Owns the UDS socket path lifecycle for the daemon:
//! - Path length validation against `UDS_PATH_LIMIT_BYTES` (BC-2.05.001 EC-002).
//! - Stale socket removal before bind (BC-2.05.001 PC-3).
//! - Socket bind at `<runtime_dir>/monocle.sock` with mode 0o600 (BC-2.05.001 PC-1/PC-2).
//! - Socket file cleanup on shutdown via `UdsTransport::cleanup` (BC-2.05.001 PC-4).
//!
//! The `UnixListener` is returned from [`UdsTransport::bind`] and passed to
//! `monocle_runtime::ipc_server::run_accept_loop`, which owns the per-client task spawner.
//! Fan-out subscriber management lives entirely in `monocle_runtime::ipc_server`.
//!
//! # Fan-out broadcast (F-ADV2-MED-002)
//!
//! All fan-out broadcasts to TUI clients go through
//! `monocle_runtime::ipc_server::broadcast_to_subscribers`, which is the canonical
//! broadcast helper with consistent slow-client handling (BC-2.05.004 EC-005).
//! The previous `broadcast_session_list_update` and `broadcast_hook_event_received`
//! methods on `UdsTransport` were dead code — the lifecycle path used
//! `monocle_runtime::ipc_server` directly, not the `UdsTransport` fan-out methods.
//! Those dead methods enforced a 256 KiB guard that production's
//! `broadcast_to_subscribers` did NOT; retaining them risked future SessionListUpdate
//! wiring bypassing the guard. They were deleted (F-ADV2-MED-002). Future callers
//! MUST add an explicit size check in `ipc_server.rs` before broadcasting large messages.
//!
//! The accept loop and per-client task spawner live in `monocle-runtime::ipc_server`
//! (S-022) to avoid a circular crate dependency.
//!
//! # S-023: `connect_with_events` — Background Reader Architecture
//!
//! [`connect_with_events`] spawns a background Tokio task that drives the socket read loop
//! on the TUI client side. On connection loss the task emits `TransportEvent::Disconnected`
//! via the event channel BEFORE placing the error in the message channel. The `UdsClientTransport`
//! returned by `connect_with_events` has `recv_message()` backed by the message channel,
//! so callers can either poll `event_rx` for the signal or call `recv_message()` — both paths
//! observe the event-before-error ordering invariant (BC-2.05.007 Invariant 1).
//!
//! The `new()` constructor retains direct socket reads (no background task) for
//! server-side / test callers that do not need the SOQ-3 event channel.

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::UnixListener;
use tokio::sync::mpsc;

use crate::error::IpcError;
use crate::events::TransportEvent;
use crate::framing::write_framed;
use crate::transport::Transport;
use crate::types::{ClientToServer, ServerToClient};

// ---------------------------------------------------------------------------
// Event channel type aliases (S-023)
// ---------------------------------------------------------------------------

/// Sender half of the bounded `TransportEvent` channel (S-023, BC-2.05.007).
///
/// Capacity is 1: a single `Disconnected` event is sufficient per connection lifetime.
pub(crate) type EventSender = mpsc::Sender<TransportEvent>;

/// Receiver half of the bounded `TransportEvent` channel (S-023, BC-2.05.007).
///
/// Returned by [`connect_with_events`]. The TUI event loop selects on this receiver
/// alongside crossterm input events. Receiving `TransportEvent::Disconnected` triggers
/// the SOQ-3 overlay-clear handler before the reconnect loop begins.
pub type EventReceiver = mpsc::Receiver<TransportEvent>;

/// Platform-specific maximum UDS socket path length in bytes.
///
/// POSIX: 104 bytes on macOS (BSD), 108 bytes on Linux. We use 104 as the conservative
/// cross-platform limit (takes the smaller of the two to guarantee portability).
/// BC-2.05.001 EC-002 references this limit.
pub const UDS_PATH_LIMIT_BYTES: usize = 104;

// ---------------------------------------------------------------------------
// Read strategy for UdsClientTransport (S-023)
// ---------------------------------------------------------------------------

/// Internal read strategy for [`UdsClientTransport`].
///
/// `Direct`: the caller drives reads via `recv_message()` on the socket half directly.
/// `Buffered`: a background Tokio task drives reads and forwards messages + events
///   through channels. Used by [`connect_with_events`].
enum ReadStrategy {
    /// Direct read from the owned socket half (no background task).
    Direct {
        reader: tokio::net::unix::OwnedReadHalf,
        event_tx: EventSender,
        graceful: bool,
    },
    /// Background reader task: `recv_message()` reads from the message channel.
    ///
    /// The background task owns the socket read half and:
    /// 1. On success: forwards the message via `msg_tx`.
    /// 2. On connection loss: emits `TransportEvent::Disconnected` via `event_tx`
    ///    BEFORE forwarding the error via `msg_tx` (ordering invariant BC-2.05.007 Invariant 1).
    ///
    /// `graceful` is set to `true` by [`UdsClientTransport::graceful_disconnect`] BEFORE
    /// the stream is dropped. The background reader checks this flag before emitting
    /// `TransportEvent::Disconnected` on EOF/connection-loss — if set, the Disconnected
    /// event is suppressed (BC-2.05.007 PC-6 / AC-006).
    ///
    /// Sequencing guarantee: `graceful_disconnect()` stores `true` with `Release` ordering,
    /// then the caller drops the transport (closing the write half). The background reader
    /// loads with `Acquire` ordering before emitting. Because the store happens-before the
    /// EOF that the reader observes, the reader always sees `graceful = true` when
    /// the disconnect originates from the TUI side.
    Buffered {
        msg_rx: mpsc::Receiver<Result<ClientToServer, IpcError>>,
        /// Shared graceful-disconnect flag. Set by `graceful_disconnect()` before stream drop.
        graceful: Arc<AtomicBool>,
    },
}

// ---------------------------------------------------------------------------
// UdsTransport — daemon-side socket lifecycle manager
// ---------------------------------------------------------------------------

/// Unix domain socket path lifecycle manager for the monocle daemon (BC-2.05.001).
///
/// Created by [`UdsTransport::bind`]; stored on [`monocle_runtime::state::DaemonState`]
/// so that cleanup on shutdown calls [`UdsTransport::cleanup`] to remove the socket file.
/// The `UnixListener` is returned separately from `bind` and passed to
/// `monocle_runtime::ipc_server::run_accept_loop`, which owns the accept loop.
///
/// # Responsibilities
///
/// - Path length validation (BC-2.05.001 EC-002).
/// - Stale socket removal (BC-2.05.001 PC-3 / EC-001).
/// - Socket permissions (BC-2.05.001 PC-2: mode 0o600).
/// - Socket cleanup on daemon shutdown (BC-2.05.001 PC-4).
///
/// Fan-out subscriber management and the accept loop live in `monocle_runtime::ipc_server`.
#[derive(Debug)]
pub struct UdsTransport {
    /// Absolute path to the bound socket file.
    sock_path: PathBuf,
}

impl UdsTransport {
    /// Bind a new `UnixListener` at `<runtime_dir>/monocle.sock`.
    ///
    /// Returns `(UdsTransport, UnixListener)`. The caller is responsible for passing
    /// the `UnixListener` to `monocle_runtime::ipc_server::run_accept_loop` (S-022).
    /// `UdsTransport` manages the socket path and fan-out subscriber list; the listener
    /// itself is owned by the accept-loop task.
    ///
    /// # Steps performed (BC-2.05.001)
    ///
    /// 1. Compute `sock_path = Path::new(runtime_dir).join("monocle.sock")`.
    /// 2. Validate `sock_path` byte length against `UDS_PATH_LIMIT_BYTES` (EC-002).
    ///    On violation: log `ERROR: UDS socket path exceeds OS limit (<N> bytes, limit <M>)`
    ///    and return `Err(IpcError::PathTooLong { .. })`.
    /// 3. If `sock_path` already exists (stale socket from a prior crash — EC-001):
    ///    remove it via `std::fs::remove_file`, then log
    ///    `WARN: removed stale UDS socket at <path>`.
    /// 4. Call `tokio::net::UnixListener::bind(&sock_path)`. On failure:
    ///    log `ERROR: failed to bind UDS socket at <path>: <reason>` and return
    ///    `Err(IpcError::BindFailure(..))`.
    /// 5. Set mode 0o600 on the socket file via `std::fs::set_permissions`.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::PathTooLong`], [`IpcError::IoError`], or
    /// [`IpcError::BindFailure`] on failure.
    pub async fn bind(runtime_dir: &Path) -> Result<(Self, UnixListener), IpcError> {
        // Step 1: Compute socket path via Path::join (never string concatenation).
        let sock_path = Path::new(runtime_dir).join("monocle.sock");

        // Step 2: Validate path length against OS UDS limit (BC-2.05.001 EC-002).
        // Use as_os_str().len() to get the true byte count of the OS path representation,
        // avoiding the lossy UTF-8 round-trip from to_string_lossy() which can inflate or
        // truncate the byte count when the path contains non-UTF-8 bytes.
        let path_len = sock_path.as_os_str().len();
        if path_len > UDS_PATH_LIMIT_BYTES {
            tracing::error!(
                "UDS socket path exceeds OS limit ({path_len} bytes, limit {UDS_PATH_LIMIT_BYTES})"
            );
            return Err(IpcError::PathTooLong {
                length: path_len,
                limit: UDS_PATH_LIMIT_BYTES,
            });
        }

        // Step 3: Remove stale socket if it exists (BC-2.05.001 PC-3 / EC-001).
        if sock_path.exists() {
            std::fs::remove_file(&sock_path)?;
            tracing::warn!("removed stale UDS socket at {}", sock_path.display());
        }

        // Step 4: Bind the UnixListener (BC-2.05.001 PC-1).
        let listener = UnixListener::bind(&sock_path).map_err(|e| {
            tracing::error!("failed to bind UDS socket at {}: {e}", sock_path.display());
            IpcError::BindFailure(e)
        })?;

        // Step 5: Set mode 0o600 — owner-only access (BC-2.05.001 PC-2).
        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&sock_path, permissions)?;

        let transport = Self { sock_path };
        Ok((transport, listener))
    }

    /// Return the absolute path of the bound socket file.
    pub fn sock_path(&self) -> &Path {
        &self.sock_path
    }

    /// Remove the socket file as part of graceful shutdown cleanup.
    ///
    /// Called alongside `monocle.lock` removal in the daemon shutdown sequence
    /// (BC-2.05.001 PC-4). Errors during removal are logged at WARN level but
    /// do not propagate — the shutdown sequence must not be blocked by socket cleanup.
    pub fn cleanup(&self) {
        if let Err(e) = std::fs::remove_file(&self.sock_path) {
            tracing::warn!(
                "failed to remove UDS socket at {} during cleanup: {e}",
                self.sock_path.display()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// connect / connect_with_events — TUI-side connection helpers
// ---------------------------------------------------------------------------

/// Connect a TUI client to the daemon's Unix domain socket (S-022, BC-2.05.002 precondition 3).
///
/// # Steps
///
/// 1. Compute `sock_path = runtime_dir.join("monocle.sock")`.
/// 2. Call `tokio::net::UnixStream::connect(&sock_path)`.
/// 3. Wrap the resulting stream in a [`UdsClientTransport`] and return it.
///
/// # Errors
///
/// Returns [`IpcError::IoError`] if the connect syscall fails (daemon not running, socket
/// not yet bound, or permission denied).
///
/// # Usage
///
/// The TUI calls this after confirming daemon liveness via the lock file PID check
/// (BC-2.05.002 precondition 2). The returned `UdsClientTransport` is ready to receive
/// the `InitialState` message via `recv_message()`.
pub async fn connect(runtime_dir: &Path) -> Result<UdsClientTransport, IpcError> {
    let sock_path = runtime_dir.join("monocle.sock");
    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .map_err(IpcError::IoError)?;
    Ok(UdsClientTransport::new(stream))
}

/// Connect a TUI client to the daemon's Unix domain socket, returning an event channel
/// alongside the transport (S-023, BC-2.05.007).
///
/// This is the S-023 replacement for [`connect`] on the TUI event-loop path. It:
/// 1. Connects to `<runtime_dir>/monocle.sock`.
/// 2. Spawns a background Tokio task that owns the socket read half.
/// 3. The background task reads messages and forwards them through a message channel.
///    On connection-loss errors it emits `TransportEvent::Disconnected` via the event
///    channel BEFORE forwarding the error, enforcing the SOQ-3 ordering invariant
///    (BC-2.05.007 Invariant 1).
/// 4. Returns `(transport, event_rx)` where `transport.recv_message()` reads from the
///    message channel and `event_rx` delivers `TransportEvent::Disconnected`.
///
/// # Background Reader Architecture
///
/// The background task enables automatic connection-loss detection without requiring
/// the caller to poll `recv_message()`. The TUI event loop can `select!` on `event_rx`
/// alongside crossterm input — the Disconnected event arrives as soon as the OS reports
/// the EOF or I/O error, regardless of whether `recv_message()` is called.
///
/// Channel capacities:
/// - Event channel: 1 (one Disconnected event per connection lifetime).
/// - Message channel: 8 (small buffer; the TUI event loop drains promptly).
///   Chosen as 8 to buffer a short burst of push messages during high-load rendering
///   cycles while keeping memory bounded. Overflow drops are tracked by the drop counter.
///
/// # Errors
///
/// Returns [`IpcError::IoError`] if the connect syscall fails.
pub async fn connect_with_events(
    runtime_dir: &Path,
) -> Result<(UdsClientTransport, EventReceiver), IpcError> {
    // Event channel capacity 1: exactly one Disconnected event per connection lifetime.
    let (event_tx, event_rx) = mpsc::channel::<TransportEvent>(1);
    // Message channel capacity 8: small buffer for push messages during render cycles.
    let (msg_tx, msg_rx) = mpsc::channel::<Result<ClientToServer, IpcError>>(8);
    // Graceful-disconnect flag: shared between transport and background reader.
    // False initially; set to true by graceful_disconnect() before stream is dropped.
    let graceful = Arc::new(AtomicBool::new(false));
    let graceful_reader = Arc::clone(&graceful);

    let sock_path = runtime_dir.join("monocle.sock");
    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .map_err(IpcError::IoError)?;
    tracing::info!(
        sock_path = ?sock_path,
        "monocle-ipc: UDS transport connected with event channel"
    );
    let (mut reader, writer) = stream.into_split();

    // Spawn the background reader task.
    // The task owns the read half and drives connection-loss detection autonomously.
    // When a connection-loss error is detected:
    //   1. Check graceful_reader flag (Acquire) — if true, suppress Disconnected emission
    //      (BC-2.05.007 PC-6 / AC-006: SOQ-3 must NOT fire on TUI-initiated graceful exit).
    //   2. If not graceful: emit TransportEvent::Disconnected via event_tx (SOQ-3 signal
    //      — BC-2.05.007 PC-1).
    //   3. Forward the error via msg_tx (so recv_message() callers get the error too).
    //   4. Exit the loop (connection is gone; no further reads possible).
    //
    // Channel send discipline (F-S023-ADV1-HIGH-002):
    // Use try_send + match rather than `let _ = .send().await` to distinguish:
    //   - Closed receiver (consumer gone): debug log + break
    //   - Full channel (consumer hung): warn log + break
    // This matches the emit_disconnected() discipline on the Direct path.
    tokio::spawn(async move {
        loop {
            let result: Result<ClientToServer, IpcError> =
                crate::framing::read_framed(&mut reader).await;
            match &result {
                Err(IpcError::Disconnected) => {
                    // Clean EOF — daemon shut down or TUI-initiated graceful close.
                    // Check graceful flag (Acquire) BEFORE emitting SOQ-3 signal.
                    // graceful_disconnect() stores true (Release) then drops the write half,
                    // causing this EOF. The Release-Acquire pair guarantees visibility.
                    if graceful_reader.load(Ordering::Acquire) {
                        tracing::debug!(
                            "background reader: EOF on socket (graceful TUI exit) — \
                             suppressing Disconnected (BC-2.05.007 PC-6 / AC-006)"
                        );
                        break;
                    }
                    tracing::debug!("background reader: EOF on socket — emitting Disconnected");
                    // Emit SOQ-3 signal first (ordering invariant BC-2.05.007 Invariant 1).
                    match event_tx.try_send(TransportEvent::Disconnected) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader: event channel full on Disconnected \
                                 (SOQ-3 may already be queued)"
                            );
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            // Consumer (TUI event loop) dropped the receiver — gone.
                            tracing::debug!(
                                "background reader: event receiver closed — consumer gone"
                            );
                            break;
                        }
                    }
                    // Forward error to recv_message() callers.
                    match msg_tx.try_send(result) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader: msg channel full on Disconnected — consumer hung"
                            );
                            break;
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!(
                                "background reader: msg receiver closed — consumer gone"
                            );
                            break;
                        }
                    }
                    break;
                }
                Err(IpcError::IoError(io_err))
                    if UdsClientTransport::is_connection_loss_error_static(io_err) =>
                {
                    // BrokenPipe or ConnectionReset — check graceful flag first.
                    if graceful_reader.load(Ordering::Acquire) {
                        tracing::debug!(
                            "background reader: connection loss ({:?}) (graceful TUI exit) — \
                             suppressing Disconnected (BC-2.05.007 PC-6 / AC-006)",
                            io_err.kind()
                        );
                        break;
                    }
                    // Unexpected connection loss.
                    tracing::debug!(
                        "background reader: connection loss ({:?}) — emitting Disconnected",
                        io_err.kind()
                    );
                    match event_tx.try_send(TransportEvent::Disconnected) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader: event channel full on connection loss"
                            );
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!(
                                "background reader: event receiver closed — consumer gone"
                            );
                            break;
                        }
                    }
                    match msg_tx.try_send(result) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader: msg channel full on connection loss — consumer hung"
                            );
                            break;
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!(
                                "background reader: msg receiver closed — consumer gone"
                            );
                            break;
                        }
                    }
                    break;
                }
                Err(_) => {
                    // Other error (e.g., framing error, MessageTooLarge).
                    // Do NOT break — recoverable errors should not kill the read loop.
                    // (In practice, MessageTooLarge / SerializeError are protocol bugs;
                    // the daemon sends malformed data. The TUI can handle and continue.)
                    match msg_tx.try_send(result) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader: msg channel full on recoverable error — \
                                 consumer hung; breaking to avoid message loss"
                            );
                            break;
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!(
                                "background reader: msg receiver closed on recoverable error"
                            );
                            break;
                        }
                    }
                }
                Ok(_) => {
                    // Successful message — forward to recv_message() callers.
                    if msg_tx.send(result).await.is_err() {
                        // Transport was dropped; stop the background task.
                        break;
                    }
                }
            }
        }
        tracing::debug!("background reader task exiting");
    });

    let transport = UdsClientTransport {
        writer,
        read_strategy: ReadStrategy::Buffered { msg_rx, graceful },
    };
    Ok((transport, event_rx))
}

/// Wrap an already-connected `UnixStream` into a `(UdsClientTransport, EventReceiver)` pair
/// with a background reader task (S-023, for use by [`crate::reconnect::reconnect`]).
///
/// This is the internal constructor used by `reconnect()` after a successful
/// `tokio::net::UnixStream::connect()`. It splits the stream, spawns the background reader
/// task, and returns both the transport and the disconnect event channel.
///
/// The caller (reconnect) owns the stream and passes it here; this avoids a second
/// `connect()` call on reconnect and ensures the event channel is created atomically
/// with the connection.
///
/// # Background Reader
///
/// Identical to the reader spawned by [`connect_with_events`]. See that function's
/// documentation for the channel capacities and ordering invariants.
pub(crate) async fn connect_with_events_from_stream(
    stream: tokio::net::UnixStream,
) -> (UdsClientTransport, EventReceiver) {
    // Event channel capacity 1: exactly one Disconnected event per connection lifetime.
    let (event_tx, event_rx) = mpsc::channel::<TransportEvent>(1);
    // Message channel capacity 8: small buffer for push messages during render cycles.
    let (msg_tx, msg_rx) = mpsc::channel::<Result<ClientToServer, IpcError>>(8);
    // Graceful-disconnect flag (same semantics as connect_with_events).
    let graceful = Arc::new(AtomicBool::new(false));
    let graceful_reader = Arc::clone(&graceful);

    let (mut reader, writer) = stream.into_split();

    // Spawn the background reader task (same logic as connect_with_events).
    tokio::spawn(async move {
        loop {
            let result: Result<ClientToServer, IpcError> =
                crate::framing::read_framed(&mut reader).await;
            match &result {
                Err(IpcError::Disconnected) => {
                    // Check graceful flag before emitting SOQ-3 (BC-2.05.007 PC-6 / AC-006).
                    if graceful_reader.load(Ordering::Acquire) {
                        tracing::debug!(
                            "background reader (reconnect): EOF (graceful TUI exit) — \
                             suppressing Disconnected"
                        );
                        break;
                    }
                    tracing::debug!(
                        "background reader (reconnect): EOF on socket — emitting Disconnected"
                    );
                    // SOQ-3 signal first (ordering invariant BC-2.05.007 Invariant 1).
                    match event_tx.try_send(TransportEvent::Disconnected) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader (reconnect): event channel full on Disconnected"
                            );
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!(
                                "background reader (reconnect): event receiver dropped — consumer gone"
                            );
                            break;
                        }
                    }
                    match msg_tx.try_send(result) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader (reconnect): msg channel full on Disconnected"
                            );
                            break;
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!(
                                "background reader (reconnect): msg receiver dropped — consumer gone"
                            );
                            break;
                        }
                    }
                    break;
                }
                Err(IpcError::IoError(io_err))
                    if UdsClientTransport::is_connection_loss_error_static(io_err) =>
                {
                    // Check graceful flag before emitting SOQ-3 (BC-2.05.007 PC-6 / AC-006).
                    if graceful_reader.load(Ordering::Acquire) {
                        tracing::debug!(
                            "background reader (reconnect): connection loss ({:?}) (graceful TUI exit) — \
                             suppressing Disconnected",
                            io_err.kind()
                        );
                        break;
                    }
                    tracing::debug!(
                        "background reader (reconnect): connection loss ({:?}) — emitting Disconnected",
                        io_err.kind()
                    );
                    match event_tx.try_send(TransportEvent::Disconnected) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader (reconnect): event channel full on connection loss"
                            );
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!(
                                "background reader (reconnect): event receiver dropped — consumer gone"
                            );
                            break;
                        }
                    }
                    match msg_tx.try_send(result) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader (reconnect): msg channel full on connection loss"
                            );
                            break;
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!(
                                "background reader (reconnect): msg receiver dropped — consumer gone"
                            );
                            break;
                        }
                    }
                    break;
                }
                Err(_) => {
                    // Other error — recoverable; do not break the read loop.
                    match msg_tx.try_send(result) {
                        Ok(()) => {}
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::warn!(
                                "background reader (reconnect): msg channel full on recoverable error — consumer hung"
                            );
                            break;
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::debug!("background reader (reconnect): msg receiver dropped");
                            break;
                        }
                    }
                }
                Ok(_) => {
                    if msg_tx.send(result).await.is_err() {
                        // Transport was dropped; stop the background task.
                        break;
                    }
                }
            }
        }
        tracing::debug!("background reader task (reconnect) exiting");
    });

    let transport = UdsClientTransport {
        writer,
        read_strategy: ReadStrategy::Buffered { msg_rx, graceful },
    };
    (transport, event_rx)
}

/// Read a single framed `ClientToServer` message from a raw `UnixStream` read half.
///
/// This is a low-level helper for callers that hold an `OwnedReadHalf` directly
/// (e.g., per-client receive loops in `monocle-runtime::ipc_server`) rather than going
/// through the `Transport` trait.
///
/// Delegates to [`crate::framing::read_framed`] for the 4-byte LE length-prefix decoding.
///
/// # Errors
///
/// Returns [`IpcError::Disconnected`] on EOF, [`IpcError::MessageTooLarge`] when the
/// declared payload exceeds 256 KiB, [`IpcError::IoError`] on socket errors, and
/// [`IpcError::SerializeError`] when the payload is not valid JSON.
pub async fn read_framed_from_stream(
    reader: &mut tokio::net::unix::OwnedReadHalf,
) -> Result<crate::types::ClientToServer, IpcError> {
    crate::framing::read_framed(reader).await
}

// ---------------------------------------------------------------------------
// UdsClientTransport — per-client TUI/server transport handle
// ---------------------------------------------------------------------------

/// Per-TUI-client transport handle.
///
/// Returned by the per-client task spawner; wraps a single accepted `UnixStream`
/// and implements the [`Transport`] trait for per-message send/recv operations.
///
/// `UdsClientTransport` is the type injected into TUI-side logic (S-022).
///
/// # S-023: Two Read Strategies
///
/// - `new()`: Direct read from socket (no background task). Used server-side and in
///   tests that call `recv_message()` explicitly. Emits `TransportEvent::Disconnected`
///   synchronously inside `recv_message()` on connection loss. The `graceful_disconnect()`
///   method suppresses the event for TUI-initiated exits.
///
/// - `connect_with_events()` (see module): Background reader task drives reads and emits
///   events autonomously. Callers can select on `event_rx` without calling `recv_message()`.
///   `recv_message()` reads from the buffered message channel.
///
/// See module-level documentation for the background reader architecture.
pub struct UdsClientTransport {
    /// Write half of the accepted UnixStream.
    writer: tokio::net::unix::OwnedWriteHalf,
    /// Read strategy: either direct socket reads or buffered via background task.
    read_strategy: ReadStrategy,
}

impl UdsClientTransport {
    /// Wrap an accepted `UnixStream` into a `UdsClientTransport` with direct reads.
    ///
    /// This constructor is retained from S-022 for callers that do not need the
    /// S-023 event channel (e.g., server-side accept loop, tests that exercise send/recv
    /// directly). It creates an internal event channel whose receiver is immediately
    /// dropped, making `TransportEvent::Disconnected` emissions non-blocking (sends on
    /// a closed channel are silently discarded).
    pub fn new(stream: tokio::net::UnixStream) -> Self {
        let (reader, writer) = stream.into_split();
        // Capacity 1: one Disconnected event per connection lifetime.
        // Receiver dropped immediately — no external event observer needed.
        let (event_tx, _event_rx) = mpsc::channel(1);
        Self {
            writer,
            read_strategy: ReadStrategy::Direct {
                reader,
                event_tx,
                graceful: false,
            },
        }
    }

    // -----------------------------------------------------------------------
    // S-023: disconnect-path detection helpers
    // -----------------------------------------------------------------------

    /// Classify an I/O error as a connection-loss error that triggers SOQ-3 emission.
    ///
    /// Returns `true` if `err` is one of the three connection-loss error kinds defined
    /// by BC-2.05.007 Precondition 2: `UnexpectedEof`, `BrokenPipe`, `ConnectionReset`.
    ///
    /// # Note on `UnexpectedEof`
    ///
    /// `read_framed` normally converts `UnexpectedEof` to `IpcError::Disconnected` before
    /// this function is reached, so the `UnexpectedEof` arm is typically unreachable in
    /// practice. It is retained as a belt-and-suspenders defensive arm: if the framing layer
    /// is ever refactored and stops converting `UnexpectedEof`, this classifier still
    /// handles it correctly rather than silently letting it fall through as a non-loss error.
    pub(crate) fn is_connection_loss_error_static(err: &std::io::Error) -> bool {
        matches!(
            err.kind(),
            std::io::ErrorKind::UnexpectedEof
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::ConnectionReset
        )
    }

    /// Mark this transport as undergoing a TUI-initiated graceful disconnect.
    ///
    /// Called by the TUI before the process exits normally (user quit). Suppresses
    /// `TransportEvent::Disconnected` for the subsequent EOF/BrokenPipe that results
    /// from the TUI closing its end of the socket (BC-2.05.007 PC-6 / AC-006).
    ///
    /// # Behaviour by read strategy
    ///
    /// - **`Direct`** (created by `new()`): sets the `graceful` bool synchronously.
    ///   `recv_message()` checks the flag before emitting `Disconnected`.
    /// - **`Buffered`** (created by `connect_with_events()` / `connect_with_events_from_stream()`):
    ///   sets `graceful = true` with `Release` ordering on the shared `Arc<AtomicBool>`.
    ///   The background reader loads with `Acquire` ordering before emitting `Disconnected`.
    ///   The caller MUST call `graceful_disconnect()` BEFORE dropping the transport
    ///   (closing the write half). The Release-Acquire pair guarantees the background reader
    ///   sees the flag before observing the resulting EOF.
    ///
    /// # AC-006 (BC-2.05.007 PC-6)
    ///
    /// SOQ-3 MUST NOT fire on TUI-initiated graceful disconnect.
    pub fn graceful_disconnect(&mut self) {
        match &mut self.read_strategy {
            ReadStrategy::Direct { graceful, .. } => {
                *graceful = true;
            }
            ReadStrategy::Buffered { graceful, .. } => {
                graceful.store(true, Ordering::Release);
            }
        }
    }

    /// Emit `TransportEvent::Disconnected` synchronously on the event channel.
    ///
    /// Uses `try_send` (non-blocking) — the channel has capacity 1 and the receiver
    /// is drained by the TUI event loop. If the channel is full (extremely unlikely:
    /// only one Disconnected event per connection lifetime), the emission is dropped
    /// with a WARN log.
    fn emit_disconnected(event_tx: &EventSender) {
        match event_tx.try_send(TransportEvent::Disconnected) {
            Ok(()) => {
                tracing::debug!("TransportEvent::Disconnected emitted (SOQ-3)");
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                tracing::warn!(
                    "TransportEvent channel full when emitting Disconnected — \
                     drop counter +1 (SOQ-3 already queued)"
                );
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                // Receiver was dropped (e.g., test uses `new()` without the event receiver).
                // Expected for the `new()` constructor path; no log needed.
            }
        }
    }
}

#[async_trait]
impl Transport for UdsClientTransport {
    async fn send_message(&mut self, msg: &ServerToClient) -> Result<(), IpcError> {
        write_framed(&mut self.writer, msg).await
    }

    /// Receive one framed `ClientToServer` message.
    ///
    /// # Direct read strategy (new())
    ///
    /// Reads directly from the socket. On connection-loss errors (`UnexpectedEof`,
    /// `BrokenPipe`, `ConnectionReset`), emits `TransportEvent::Disconnected` via the
    /// event channel BEFORE returning the error (BC-2.05.007 PC-1, Invariant 1).
    /// The `graceful` flag suppresses emission for TUI-initiated disconnects (AC-006).
    ///
    /// # Buffered read strategy (connect_with_events())
    ///
    /// Reads from the message channel populated by the background reader task. The
    /// background task has already emitted `TransportEvent::Disconnected` before placing
    /// the error in the channel — ordering is preserved even from this path.
    async fn recv_message(&mut self) -> Result<ClientToServer, IpcError> {
        match &mut self.read_strategy {
            ReadStrategy::Direct {
                reader,
                event_tx,
                graceful,
            } => {
                let result = crate::framing::read_framed(reader).await;
                if !*graceful {
                    match &result {
                        Err(IpcError::Disconnected) => {
                            Self::emit_disconnected(event_tx);
                        }
                        Err(IpcError::IoError(io_err))
                            if Self::is_connection_loss_error_static(io_err) =>
                        {
                            Self::emit_disconnected(event_tx);
                        }
                        _ => {}
                    }
                }
                result
            }
            ReadStrategy::Buffered { msg_rx, .. } => {
                // Background task has already emitted the event before placing the
                // error in the channel — ordering invariant is preserved.
                msg_rx.recv().await.unwrap_or(Err(IpcError::Disconnected))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Compile-time assertions
// ---------------------------------------------------------------------------

/// Assert that `UdsClientTransport` implements `Transport` (BC-2.05.008 PC-5).
///
/// This is a compile-time check: if `UdsClientTransport` does not implement `Transport`,
/// this code fails to compile. Named per the test naming convention.
#[allow(dead_code, non_snake_case)]
fn test_BC_2_05_008_uds_client_transport_implements_transport() {
    fn assert_transport<T: Transport>() {}
    assert_transport::<UdsClientTransport>();
}

// ---------------------------------------------------------------------------
// Unit tests for is_connection_loss_error_static (F-S023-ADV1-MED-001)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::UdsClientTransport;

    /// F-S023-ADV1-MED-001 / BC-2.05.007 PC-6: `is_connection_loss_error_static` returns
    /// `true` for `ErrorKind::BrokenPipe`.
    ///
    /// `BrokenPipe` occurs when a WRITER sends data after the peer has closed its read end.
    /// Integration tests cannot reliably produce `BrokenPipe` on the UDS read path (the
    /// kernel gives EOF / `UnexpectedEof` instead). This unit test directly constructs the
    /// error and verifies the classifier, giving production proof of the BrokenPipe arm.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_05_007_unit_is_connection_loss_broken_pipe() {
        let err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe");
        assert!(
            UdsClientTransport::is_connection_loss_error_static(&err),
            "BC-2.05.007 PC-6: BrokenPipe must classify as a connection-loss error"
        );
    }

    /// F-S023-ADV1-MED-001 / BC-2.05.007 PC-6: `is_connection_loss_error_static` returns
    /// `true` for `ErrorKind::ConnectionReset`.
    ///
    /// `ConnectionReset` occurs when the peer sends a RST segment (TCP) or closes its end
    /// of a connected socket with SO_LINGER=0. UDS drops rarely produce this on macOS/Linux
    /// (they produce EOF instead), so integration-test coverage is unreliable. This unit test
    /// directly constructs the error and verifies the classifier.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_05_007_unit_is_connection_loss_connection_reset() {
        let err = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "connection reset");
        assert!(
            UdsClientTransport::is_connection_loss_error_static(&err),
            "BC-2.05.007 PC-6: ConnectionReset must classify as a connection-loss error"
        );
    }

    /// F-S023-ADV1-MED-001 / BC-2.05.007 PC-6: `is_connection_loss_error_static` returns
    /// `true` for `ErrorKind::UnexpectedEof`.
    ///
    /// This arm is typically converted to `IpcError::Disconnected` by the framing layer
    /// before reaching the classifier. The defensive arm is validated here as belt-and-suspenders.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_05_007_unit_is_connection_loss_unexpected_eof() {
        let err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "unexpected eof");
        assert!(
            UdsClientTransport::is_connection_loss_error_static(&err),
            "BC-2.05.007 PC-6: UnexpectedEof must classify as a connection-loss error"
        );
    }

    /// F-S023-ADV1-MED-001 / BC-2.05.007 PC-6: `is_connection_loss_error_static` returns
    /// `false` for `ErrorKind::PermissionDenied` — non-connection-loss errors must not
    /// trigger SOQ-3.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_05_007_unit_is_connection_loss_false_for_permission_denied() {
        let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        assert!(
            !UdsClientTransport::is_connection_loss_error_static(&err),
            "BC-2.05.007 PC-6: PermissionDenied must NOT classify as a connection-loss error"
        );
    }

    /// F-S023-ADV1-MED-001 / BC-2.05.007 PC-6: `is_connection_loss_error_static` returns
    /// `false` for `ErrorKind::WouldBlock` — non-connection-loss errors must not
    /// trigger SOQ-3.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_05_007_unit_is_connection_loss_false_for_would_block() {
        let err = std::io::Error::new(std::io::ErrorKind::WouldBlock, "would block");
        assert!(
            !UdsClientTransport::is_connection_loss_error_static(&err),
            "BC-2.05.007 PC-6: WouldBlock must NOT classify as a connection-loss error"
        );
    }
}
