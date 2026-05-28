//! `UdsTransport` — Unix domain socket implementation of the `Transport` trait.
//!
//! Phase 1 sole implementor of [`crate::transport::Transport`]. Handles:
//! - Socket bind at `<runtime_dir>/monocle.sock` with mode 0o600 (BC-2.05.001 PC-1/PC-2).
//! - Stale socket removal before bind (BC-2.05.001 PC-3).
//! - UDS path length validation (BC-2.05.001 EC-002).
//! - Fan-out subscriber list (`Vec<Sender<ServerToClient>>`).
//! - `broadcast_session_list_update` / `broadcast_hook_event_received` with 256 KiB guard.
//! - Slow client disconnect (BC-2.05.004 EC-005).
//!
//! The accept loop and per-client task spawner live in `monocle-runtime::ipc_server`
//! (S-022) to avoid a circular crate dependency.

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::UnixListener;
use tokio::sync::mpsc;

use crate::error::IpcError;
use crate::framing::{write_framed, MAX_MESSAGE_BYTES};
use crate::transport::Transport;
use crate::types::{ClientToServer, ServerToClient};

/// Platform-specific maximum UDS socket path length in bytes.
///
/// POSIX: 104 bytes on macOS (BSD), 108 bytes on Linux. We use 104 as the conservative
/// cross-platform limit (takes the smaller of the two to guarantee portability).
/// BC-2.05.001 EC-002 references this limit.
pub const UDS_PATH_LIMIT_BYTES: usize = 104;

/// A single TUI client connection handle on the server side.
///
/// Each connected TUI client has a `Sender<ServerToClient>` in the fan-out list.
/// When the sender returns an error (full buffer / dropped receiver), the client
/// is removed from the list (AC-017, BC-2.05.004 EC-005).
type ClientSender = mpsc::Sender<ServerToClient>;

/// Unix domain socket transport for daemon-to-TUI IPC (Phase 1).
///
/// Binds a `UnixListener` at `<runtime_dir>/monocle.sock` and manages a fan-out
/// subscriber list of connected TUI clients.
///
/// # Construction
///
/// Use [`UdsTransport::bind`] to create a bound socket. The bound `UnixListener`
/// is transferred to `monocle_runtime::ipc_server::run_accept_loop` (S-022) which
/// owns the per-client task spawner. The returned `UdsTransport` manages the path
/// and fan-out subscriber list.
///
/// # Fan-out
///
/// [`UdsTransport::broadcast_session_list_update`] and
/// [`UdsTransport::broadcast_hook_event_received`] serialize and send messages to all
/// connected clients. Disconnected clients are silently removed from the subscriber list.
#[derive(Debug)]
pub struct UdsTransport {
    /// Absolute path to the bound socket file.
    sock_path: PathBuf,
    /// Fan-out subscriber list — one sender per connected TUI client.
    subscribers: Arc<tokio::sync::Mutex<Vec<ClientSender>>>,
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

        let transport = Self {
            sock_path,
            subscribers: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        };
        Ok((transport, listener))
    }

    /// Add a subscriber sender to the fan-out list (test helper and accept-loop wiring).
    ///
    /// Production callers: `monocle_runtime::ipc_server::register_subscriber`.
    /// Test callers: fan_out integration tests that inject senders directly.
    pub async fn add_subscriber(&self, sender: ClientSender) {
        self.subscribers.lock().await.push(sender);
    }

    /// Return the number of currently registered subscribers (test helper).
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.lock().await.len()
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

    /// Broadcast `ServerToClient::SessionListUpdate` to all connected TUI clients.
    ///
    /// # Contract (BC-2.05.003)
    ///
    /// - The `sessions` Vec contains the **complete current session list** (not a diff).
    /// - If the serialized message exceeds 256 KiB:
    ///   log `ERROR: SessionListUpdate exceeds 256 KiB; cannot broadcast` and return.
    ///   No message is sent to any client.
    /// - Disconnected clients (closed receiver) are silently removed from the subscriber list.
    /// - The `drop_counter` is NOT incremented by IPC send failures (AC-010).
    pub async fn broadcast_session_list_update(
        &self,
        sessions: Vec<monocle_core::engine::EnrichedSession>,
    ) {
        let msg = ServerToClient::SessionListUpdate { sessions };

        // 256 KiB guard (BC-2.05.003 PC-3).
        // Serialize once to measure size; the message itself is passed to fan_out_message
        // to avoid a redundant deserialize step.
        let serialized = match serde_json::to_vec(&msg) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("SessionListUpdate serialization failed: {e}");
                return;
            }
        };
        if serialized.len() > MAX_MESSAGE_BYTES {
            tracing::error!("SessionListUpdate exceeds 256 KiB; cannot broadcast");
            return;
        }

        self.fan_out_message(&msg).await;
    }

    /// Broadcast `ServerToClient::HookEventReceived` to all connected TUI clients.
    ///
    /// # Contract (BC-2.05.004)
    ///
    /// - `payload_excerpt` is constructed from `body_bytes` by:
    ///   1. Interpreting `body_bytes` as UTF-8 (losslessly, via `String::from_utf8_lossy`).
    ///   2. Calling [`crate::types::truncate_to_utf8_boundary`] with `max_bytes = 256`.
    /// - `latency_ms` is passed through from the HTTP handler's measurement.
    /// - Slow clients (send buffer full): removed from subscriber list; log
    ///   `WARN: removed slow TUI client (send buffer full)`. Other clients unaffected.
    /// - The `drop_counter` is NOT incremented on IPC slow-client disconnect (AC-010).
    pub async fn broadcast_hook_event_received(
        &self,
        hook_type: monocle_core::hook_events::HookType,
        session_id: String,
        body_bytes: &[u8],
        latency_ms: u64,
    ) {
        use crate::types::{truncate_to_utf8_boundary, PAYLOAD_EXCERPT_MAX_BYTES};

        let body_str = String::from_utf8_lossy(body_bytes);
        let payload_excerpt =
            truncate_to_utf8_boundary(&body_str, PAYLOAD_EXCERPT_MAX_BYTES).to_owned();

        let msg = ServerToClient::HookEventReceived {
            hook_type,
            session_id,
            payload_excerpt,
            latency_ms,
        };

        self.fan_out_message(&msg).await;
    }

    /// Fan-out a `ServerToClient` message to all connected TUI clients.
    ///
    /// Clones the message into each subscriber's bounded channel. Subscribers whose
    /// channels are full (slow clients) or whose receivers are dropped (disconnected
    /// clients) are removed from the list after the broadcast pass.
    ///
    /// The `drop_counter` is NOT touched here (BC-2.05.004 PC-4).
    async fn fan_out_message(&self, msg: &ServerToClient) {
        let mut subs = self.subscribers.lock().await;
        let mut live: Vec<ClientSender> = Vec::with_capacity(subs.len());

        for sender in subs.drain(..) {
            match sender.try_send(msg.clone()) {
                Ok(()) => live.push(sender),
                Err(mpsc::error::TrySendError::Full(_)) => {
                    // Slow client — send buffer full. Disconnect and log WARN (BC-2.05.004 EC-005).
                    tracing::warn!("removed slow TUI client (send buffer full)");
                    // sender is dropped here, which closes the channel.
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    // Disconnected client — silently remove from subscriber list.
                }
            }
        }

        *subs = live;
    }
}

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

/// Per-TUI-client transport handle.
///
/// Returned by the per-client task spawner; wraps a single accepted `UnixStream`
/// and implements the [`Transport`] trait for per-message send/recv operations.
///
/// `UdsClientTransport` is the type injected into TUI-side logic (S-022).
pub struct UdsClientTransport {
    /// Split read half of the accepted UnixStream.
    reader: tokio::net::unix::OwnedReadHalf,
    /// Split write half of the accepted UnixStream.
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl UdsClientTransport {
    /// Wrap an accepted `UnixStream` into a `UdsClientTransport`.
    pub fn new(stream: tokio::net::UnixStream) -> Self {
        let (reader, writer) = stream.into_split();
        Self { reader, writer }
    }
}

#[async_trait]
impl Transport for UdsClientTransport {
    async fn send_message(&mut self, msg: &ServerToClient) -> Result<(), IpcError> {
        write_framed(&mut self.writer, msg).await
    }

    async fn recv_message(&mut self) -> Result<ClientToServer, IpcError> {
        crate::framing::read_framed(&mut self.reader).await
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
