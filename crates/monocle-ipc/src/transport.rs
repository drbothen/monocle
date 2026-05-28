//! The `Transport` trait — the Phase 4 extensibility point for IPC transport.
//!
//! Defined in `monocle-ipc` per BC-2.05.008 PC-5/PC-6. The trait has exactly two methods:
//! `send_message` and `recv_message`. `UdsTransport` is the sole Phase 1 implementor.
//!
//! When Phase 4 arrives, `ShmTransport` will implement this trait in a new `monocle-ipc-shm`
//! crate. No changes to this file are required for Phase 4 transport addition.

use async_trait::async_trait;

use crate::error::IpcError;
use crate::types::{ClientToServer, ServerToClient};

/// Async IPC transport abstraction for daemon-TUI communication.
///
/// # Phase 1 Constraint (BC-2.05.008 PC-5/PC-6)
///
/// [`crate::uds::UdsTransport`] is the only `Transport` implementor in `monocle-ipc`
/// during Phase 1. This trait provides the abstraction point for a future Phase 4
/// `ShmTransport` in `monocle-ipc-shm`.
///
/// # Method Stability
///
/// The `send_message` / `recv_message` signatures are stable for Phase 4. Any change
/// to these signatures is a breaking ABI change requiring a new BC revision and ADR.
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Send a `ServerToClient` message to the connected TUI client.
    ///
    /// # Errors
    ///
    /// - [`IpcError::SerializeError`] — if JSON serialization fails.
    /// - [`IpcError::MessageTooLarge`] — if the serialized message exceeds 256 KiB.
    /// - [`IpcError::IoError`] — if writing to the socket fails.
    /// - [`IpcError::Disconnected`] — if the client has closed the connection.
    async fn send_message(&mut self, msg: &ServerToClient) -> Result<(), IpcError>;

    /// Receive the next `ClientToServer` message from the connected TUI client.
    ///
    /// Blocks until a complete framed message is available.
    ///
    /// # Errors
    ///
    /// - [`IpcError::Disconnected`] — if the client closed the connection (EOF).
    /// - [`IpcError::MessageTooLarge`] — if the received message exceeds 256 KiB.
    /// - [`IpcError::IoError`] — if reading from the socket fails.
    /// - [`IpcError::SerializeError`] — if the payload is not valid JSON.
    async fn recv_message(&mut self) -> Result<ClientToServer, IpcError>;
}
