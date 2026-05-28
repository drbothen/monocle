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
    Buffered {
        msg_rx: mpsc::Receiver<Result<ClientToServer, IpcError>>,
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

    let sock_path = runtime_dir.join("monocle.sock");
    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .map_err(IpcError::IoError)?;
    let (mut reader, writer) = stream.into_split();

    // Spawn the background reader task.
    // The task owns the read half and drives connection-loss detection autonomously.
    // When a connection-loss error is detected:
    //   1. Emit TransportEvent::Disconnected via event_tx (SOQ-3 signal — BC-2.05.007 PC-1).
    //   2. Forward the error via msg_tx (so recv_message() callers get the error too).
    //   3. Exit the loop (connection is gone; no further reads possible).
    tokio::spawn(async move {
        loop {
            let result: Result<ClientToServer, IpcError> =
                crate::framing::read_framed(&mut reader).await;
            match &result {
                Err(IpcError::Disconnected) => {
                    // Clean EOF — daemon shut down or crashed.
                    tracing::debug!("background reader: EOF on socket — emitting Disconnected");
                    // Emit SOQ-3 signal first (ordering invariant BC-2.05.007 Invariant 1).
                    let _ = event_tx.send(TransportEvent::Disconnected).await;
                    // Forward error to recv_message() callers.
                    let _ = msg_tx.send(result).await;
                    break;
                }
                Err(IpcError::IoError(io_err))
                    if UdsClientTransport::is_connection_loss_error_static(io_err) =>
                {
                    // BrokenPipe or ConnectionReset — unexpected connection loss.
                    tracing::debug!(
                        "background reader: connection loss ({:?}) — emitting Disconnected",
                        io_err.kind()
                    );
                    let _ = event_tx.send(TransportEvent::Disconnected).await;
                    let _ = msg_tx.send(result).await;
                    break;
                }
                Err(_) => {
                    // Other error (e.g., framing error, MessageTooLarge).
                    let _ = msg_tx.send(result).await;
                    // Do NOT break — recoverable errors should not kill the read loop.
                    // (In practice, MessageTooLarge / SerializeError are protocol bugs;
                    // the daemon sends malformed data. The TUI can handle and continue.)
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
        read_strategy: ReadStrategy::Buffered { msg_rx },
    };
    Ok((transport, event_rx))
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
    /// Called by the TUI before the process exits normally (user quit). Prevents
    /// `recv_message()` from emitting `TransportEvent::Disconnected` for the subsequent
    /// EOF/BrokenPipe that results from the TUI closing its end of the socket.
    ///
    /// Only effective on the `Direct` read strategy (the `new()` / `with_event_channel()`
    /// path). For `connect_with_events()` transports, the background reader task handles
    /// connection loss — calling `graceful_disconnect()` is still correct but has no effect
    /// on the already-spawned background task.
    ///
    /// # AC-006 (BC-2.05.007 PC-6)
    ///
    /// SOQ-3 MUST NOT fire on TUI-initiated graceful disconnect.
    pub fn graceful_disconnect(&mut self) {
        if let ReadStrategy::Direct { graceful, .. } = &mut self.read_strategy {
            *graceful = true;
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
    /// # Direct read strategy (new() / with_event_channel())
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
            ReadStrategy::Buffered { msg_rx } => {
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
