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

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::net::UnixListener;

use crate::error::IpcError;
use crate::framing::write_framed;
use crate::transport::Transport;
use crate::types::{ClientToServer, ServerToClient};

/// Platform-specific maximum UDS socket path length in bytes.
///
/// POSIX: 104 bytes on macOS (BSD), 108 bytes on Linux. We use 104 as the conservative
/// cross-platform limit (takes the smaller of the two to guarantee portability).
/// BC-2.05.001 EC-002 references this limit.
pub const UDS_PATH_LIMIT_BYTES: usize = 104;

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
