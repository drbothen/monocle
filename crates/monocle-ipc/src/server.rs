//! Daemon-side UDS connection accept loop and per-client task spawner (S-022).
//!
//! This module implements the server side of the IPC connection lifecycle described
//! in BC-2.05.002. Key responsibilities:
//!
//! - Accept incoming TUI client connections from an already-bound [`tokio::net::UnixListener`].
//! - Spawn a dedicated Tokio task per client (per-client send loop + fan-out subscriber).
//! - Send one [`crate::types::ServerToClient::InitialState`] message as the FIRST message
//!   on every new connection (BC-2.05.002 postcondition PC-2, invariant 1).
//! - Register the client in the fan-out subscriber list before any incremental event can
//!   be missed (BC-2.05.002 invariant 3 — no gap window).
//! - Remove clients from the subscriber list on clean EOF or send error (AC-001).
//!
//! # S-022 Stub Status
//!
//! All function bodies in this module are `todo!()` stubs. This module is the stub
//! skeleton for Step 3 (failing tests) and Step 4 (implementation). The `clippy::todo`
//! and dead-code lints are suppressed at module scope because stubs are intentionally
//! incomplete by design.
#![allow(clippy::todo, dead_code, unused_variables)]

use std::sync::Arc;

use tokio::net::UnixListener;
use tokio::sync::mpsc;

use crate::error::IpcError;
use crate::types::ServerToClient;

/// Bounded channel capacity for per-client outbound message queues.
///
/// Matches the capacity used in [`crate::uds::UdsTransport`] fan-out channels.
/// Each connected client gets a dedicated `mpsc::channel(CLIENT_CHANNEL_CAPACITY)`.
pub(crate) const CLIENT_CHANNEL_CAPACITY: usize = 64;

/// Handle for the fan-out subscriber list shared across the accept loop and all per-client tasks.
///
/// The outer `Arc<tokio::sync::Mutex<...>>` matches the pattern established in
/// [`crate::uds::UdsTransport::subscribers`] so both types can share the same list.
pub type SubscriberList = Arc<tokio::sync::Mutex<Vec<mpsc::Sender<ServerToClient>>>>;

/// Entry point for the daemon connection accept loop (BC-2.05.002 postcondition PC-1).
///
/// Accepts incoming TUI client connections from `listener` and spawns a dedicated Tokio task
/// per client via [`spawn_client_task`]. Runs until the listener is closed (daemon shutdown).
///
/// # Parameters
///
/// - `listener`: The already-bound `UnixListener` from `monocle-ipc`'s bind step (S-021).
/// - `state`: Shared daemon state; used to take the `InitialState` snapshot and look up the
///   pending-decision registry.
/// - `subscribers`: Shared fan-out subscriber list to which new client senders are added.
///
/// # Contract
///
/// - Each accepted connection spawns exactly one Tokio task via `tokio::spawn`.
/// - The task is responsible for (a) registering the client sender in `subscribers`,
///   (b) sending the `InitialState` message as the first message, and (c) removing the
///   sender on disconnect.
/// - This function itself does not block; it loops on `listener.accept().await`.
pub async fn run_accept_loop(
    _listener: UnixListener,
    _state: Arc<monocle_runtime_state_placeholder::DaemonState>,
    _subscribers: SubscriberList,
) {
    todo!("S-022: run_accept_loop — accept loop + per-client task spawner")
}

/// Spawn a dedicated Tokio task for a single accepted TUI client connection.
///
/// # Responsibilities (AC-001, AC-002)
///
/// 1. Create a bounded `mpsc::channel(CLIENT_CHANNEL_CAPACITY)` for this client.
/// 2. Take the `InitialState` snapshot from `state` via `snapshot_initial_state`.
/// 3. Register the sender in `subscribers` (before any event can be missed — AC-006).
/// 4. Send the `InitialState` message as the FIRST message to the client (AC-002).
/// 5. Enter the per-client receive loop: read `ClientToServer` messages and dispatch them.
/// 6. On EOF or send error: remove this client's sender from `subscribers`; log DEBUG.
///
/// # Parameters
///
/// - `stream`: The accepted `tokio::net::UnixStream` for this client.
/// - `state`: Shared daemon state (for snapshot and permission decision routing).
/// - `subscribers`: Shared fan-out list; the new sender is appended before `InitialState` send.
pub async fn spawn_client_task(
    _stream: tokio::net::UnixStream,
    _state: Arc<monocle_runtime_state_placeholder::DaemonState>,
    _subscribers: SubscriberList,
) {
    todo!("S-022: spawn_client_task — InitialState send + per-client receive loop")
}

/// Send the `InitialState` snapshot message to a newly connected TUI client.
///
/// # Contract (BC-2.05.002 PC-2, PC-4)
///
/// - Calls `snapshot_initial_state(state)` to build the payload.
/// - Serializes the resulting `ServerToClient::InitialState { .. }` message.
/// - If the serialized payload exceeds 256 KiB: logs
///   `ERROR: InitialState for client exceeds 256 KiB limit (<N> bytes)`,
///   closes the connection by returning `Err(IpcError::MessageTooLarge)`.
/// - Writes the framed message to `writer` using `crate::framing::write_framed`.
///
/// # Returns
///
/// `Ok(())` on success; `Err(IpcError)` on serialization failure, size overflow, or I/O error.
pub async fn send_initial_state(
    _writer: &mut tokio::net::unix::OwnedWriteHalf,
    _state: &monocle_runtime_state_placeholder::DaemonState,
) -> Result<(), IpcError> {
    todo!("S-022: send_initial_state — snapshot + 256 KiB guard + write_framed")
}

/// Add a client sender to the shared fan-out subscriber list.
///
/// # Contract (AC-006 — no gap window)
///
/// The sender MUST be registered in `subscribers` BEFORE `InitialState` is sent.
/// Registering before send ensures that any incremental events published between
/// snapshot time and first message delivery are queued in the client's channel,
/// not dropped.
pub async fn register_subscriber(
    _subscribers: &SubscriberList,
    _sender: mpsc::Sender<ServerToClient>,
) {
    todo!("S-022: register_subscriber — append sender to shared subscriber list")
}

/// Remove a client sender from the shared fan-out subscriber list.
///
/// Called when the per-client task detects EOF or a send error on the client channel.
/// Logs at DEBUG level: `DEBUG: TUI client disconnected; removed from subscriber list`.
///
/// # Implementation note
///
/// Identify the sender to remove by pointer equality (`Arc::ptr_eq` or `Sender::same_channel`)
/// rather than index, to avoid a TOCTOU race with concurrent subscriber additions.
pub async fn remove_subscriber(
    _subscribers: &SubscriberList,
    _sender: &mpsc::Sender<ServerToClient>,
) {
    todo!("S-022: remove_subscriber — remove sender from subscriber list on disconnect")
}

// ---------------------------------------------------------------------------
// Placeholder module to allow monocle-ipc stubs to compile without a full
// dependency on monocle-runtime. The implementer will replace this with an
// actual `use monocle_runtime::state::DaemonState` import once the
// monocle-runtime dependency is wired into monocle-ipc/Cargo.toml (S-022 task).
// ---------------------------------------------------------------------------

/// Placeholder types for `DaemonState` until monocle-runtime is added as a
/// monocle-ipc dependency. The implementer MUST replace this with the real import.
///
/// This module is intentionally minimal — it provides only the surface needed for
/// the stub signatures to compile. No logic lives here.
#[doc(hidden)]
pub mod monocle_runtime_state_placeholder {
    /// Opaque placeholder for `monocle_runtime::state::DaemonState`.
    ///
    /// Replaced by the real `DaemonState` when the implementer wires the
    /// `monocle-runtime` dependency into `monocle-ipc/Cargo.toml`.
    pub struct DaemonState {
        _private: (),
    }
}
