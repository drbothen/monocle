//! Daemon-side UDS connection accept loop and per-client task spawner (S-022).
//!
//! Implements the server side of the IPC connection lifecycle described in BC-2.05.002.
//!
//! Lives in `monocle-runtime` (not `monocle-ipc`) to avoid a circular crate dependency:
//! `monocle-runtime` depends on `monocle-ipc`, and this module needs both
//! `monocle_ipc` types and `monocle_runtime::state::DaemonState`.
//!
//! # Responsibilities
//!
//! - Accept incoming TUI client connections from an already-bound `UnixListener`.
//! - Spawn a dedicated Tokio task per client (per-client send loop + fan-out subscriber).
//! - Send one `ServerToClient::InitialState` message as the FIRST message on every new
//!   connection (BC-2.05.002 postcondition PC-2, invariant 1).
//! - Register the client in the fan-out subscriber list BEFORE sending `InitialState`,
//!   so no events are lost between snapshot time and streaming phase (AC-006, invariant 3).
//! - Process incoming `ClientToServer::PermissionDecision` messages from each client and
//!   route them to the pending-decision registry (BC-2.05.005 postcondition PC-3).
//! - Remove clients from the subscriber list on clean EOF or send error (AC-001).

use std::sync::Arc;

use tokio::net::UnixListener;

use monocle_ipc::error::IpcError;
use monocle_ipc::framing::{write_framed, MAX_MESSAGE_BYTES};
use monocle_ipc::server::{register_subscriber, remove_subscriber, SubscriberList};

/// Per-client outbound channel capacity — matches `monocle_ipc::server::CLIENT_CHANNEL_CAPACITY`.
const CLIENT_CHANNEL_CAPACITY: usize = monocle_ipc::server::CLIENT_CHANNEL_CAPACITY;
use monocle_ipc::types::{ClientToServer, ServerToClient};

use crate::state::{snapshot_initial_state, DaemonState};

/// Entry point for the daemon UDS connection accept loop (BC-2.05.002 postcondition PC-1).
///
/// Accepts incoming TUI client connections from `listener` and spawns a dedicated Tokio task
/// per client. Runs until the listener is closed (daemon shutdown).
///
/// # Parameters
///
/// - `listener`: The already-bound `UnixListener` from the daemon bind step (S-021).
/// - `state`: Shared daemon state; used to take the `InitialState` snapshot and look up the
///   pending-decision registry.
/// - `subscribers`: Shared fan-out subscriber list to which new client senders are added.
pub async fn run_accept_loop(
    listener: UnixListener,
    state: Arc<DaemonState>,
    subscribers: SubscriberList,
) {
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let state = Arc::clone(&state);
                let subscribers = Arc::clone(&subscribers);
                tokio::spawn(async move {
                    spawn_client_task(stream, state, subscribers).await;
                });
            }
            Err(e) => {
                tracing::error!("UDS accept error: {e}");
                break;
            }
        }
    }
}

/// Spawn a dedicated Tokio task for a single accepted TUI client connection.
///
/// # Responsibilities (AC-001, AC-002, AC-006)
///
/// 1. Create a bounded `mpsc::channel(CLIENT_CHANNEL_CAPACITY)` for this client.
/// 2. Register the sender in `subscribers` BEFORE taking the snapshot (AC-006 no-gap).
/// 3. Take the `InitialState` snapshot from `state` via `snapshot_initial_state`.
/// 4. Send the `InitialState` message as the FIRST message to the client (AC-002).
/// 5. Enter the per-client receive loop: read `ClientToServer` messages and dispatch them.
/// 6. On EOF or send error: remove this client's sender from `subscribers`; log DEBUG.
async fn spawn_client_task(
    stream: tokio::net::UnixStream,
    state: Arc<DaemonState>,
    subscribers: SubscriberList,
) {
    let (mut read_half, mut write_half) = stream.into_split();

    // Step 1: Create per-client bounded channel.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);

    // Step 2: Register in subscribers BEFORE snapshot (AC-006 — no gap window).
    // Any event published after registration is queued in this channel, not lost.
    register_subscriber(&subscribers, tx.clone()).await;

    // Step 3 + 4: Take snapshot and send InitialState as the first message.
    match send_initial_state(&mut write_half, &state).await {
        Ok(()) => {}
        Err(e) => {
            tracing::error!("failed to send InitialState to client: {e}");
            remove_subscriber(&subscribers, &tx).await;
            return;
        }
    }

    // Step 5: Concurrent per-client send loop + receive loop.
    // The send loop drains the mpsc channel to the client's write half.
    // The receive loop reads ClientToServer messages and dispatches them.
    //
    // Use tokio::select! to run both concurrently. When either loop exits
    // (EOF, error, or channel closed), cleanup and return.

    loop {
        tokio::select! {
            // Outbound: drain queued ServerToClient messages to the client.
            msg = rx.recv() => {
                match msg {
                    Some(msg) => {
                        if let Err(e) = write_framed(&mut write_half, &msg).await {
                            tracing::debug!("TUI client send error: {e}; closing connection");
                            break;
                        }
                    }
                    None => {
                        // Channel closed (daemon shutting down).
                        break;
                    }
                }
            }

            // Inbound: read ClientToServer messages from the client.
            result = monocle_ipc::uds::read_framed_from_stream(&mut read_half) => {
                match result {
                    Ok(ClientToServer::PermissionDecision { prompt_id, decision }) => {
                        handle_permission_decision(
                            prompt_id,
                            decision,
                            &state,
                            &subscribers,
                        ).await;
                    }
                    Err(IpcError::Disconnected) => {
                        tracing::debug!("TUI client disconnected (EOF)");
                        break;
                    }
                    Err(e) => {
                        tracing::warn!("TUI client recv error: {e}; closing connection");
                        break;
                    }
                }
            }
        }
    }

    // Step 6: Remove client from fan-out subscriber list on disconnect.
    remove_subscriber(&subscribers, &tx).await;
}

/// Send the `InitialState` snapshot message to a newly connected TUI client.
///
/// # Contract (BC-2.05.002 PC-2, PC-4)
///
/// - Calls `snapshot_initial_state(state)` to build the payload.
/// - Serializes the resulting `ServerToClient::InitialState { .. }` message.
/// - If the serialized payload exceeds 256 KiB: logs
///   `ERROR: InitialState for client exceeds 256 KiB limit (<N> bytes)`,
///   and returns `Err(IpcError::MessageTooLarge)` which causes the caller to close the
///   connection.
/// - Writes the framed message to `writer` using `monocle_ipc::framing::write_framed`.
///
/// # Returns
///
/// `Ok(())` on success; `Err(IpcError)` on serialization failure, size overflow, or I/O error.
pub async fn send_initial_state(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    state: &DaemonState,
) -> Result<(), IpcError> {
    let msg = snapshot_initial_state(state);

    // 256 KiB guard (BC-2.05.002 PC-4, AC-004).
    // Serialize first to measure size; if within limit, write_framed serializes again.
    // Double-serialize is acceptable: this path is called once per client connect, not
    // in a hot loop.
    let serialized = serde_json::to_vec(&msg)?;
    if serialized.len() > MAX_MESSAGE_BYTES {
        tracing::error!(
            "InitialState for client exceeds 256 KiB limit ({} bytes)",
            serialized.len()
        );
        return Err(IpcError::MessageTooLarge);
    }

    write_framed(writer, &msg).await
}

/// Broadcast a `ServerToClient` message to all connected TUI clients.
///
/// Implements the drain-and-retain pattern (slow-client disconnect):
/// - For each sender: calls `try_send(msg.clone())`.
/// - On `TrySendError::Full` (slow client): logs WARN and removes the sender.
/// - On `TrySendError::Closed` (disconnected): silently removes the sender.
///
/// This is the canonical broadcast helper; all broadcast sites MUST use it to ensure
/// consistent slow-client handling (BC-2.05.004 EC-005).
pub(crate) async fn broadcast_to_subscribers(subscribers: &SubscriberList, msg: ServerToClient) {
    let mut subs = subscribers.lock().await;
    let mut live: Vec<tokio::sync::mpsc::Sender<ServerToClient>> = Vec::with_capacity(subs.len());

    for sender in subs.drain(..) {
        match sender.try_send(msg.clone()) {
            Ok(()) => live.push(sender),
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                // Slow client — send buffer full. Disconnect and log WARN (BC-2.05.004 EC-005).
                tracing::warn!("removed slow TUI client during broadcast (send buffer full)");
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                // Disconnected client — silently remove.
            }
        }
    }

    *subs = live;
}

/// Route a `PermissionDecision` from a TUI client to the pending-decision registry.
///
/// # Contract (BC-2.05.005 postcondition PC-3)
///
/// - If `prompt_id` is found: resolves the oneshot; broadcasts `PermissionPromptResolved`
///   to ALL connected TUI clients (including the resolver).
/// - If `prompt_id` is NOT found: silently discarded (no-op, no error to client).
async fn handle_permission_decision(
    prompt_id: uuid::Uuid,
    decision: monocle_ipc::types::PermissionDecisionKind,
    state: &DaemonState,
    subscribers: &SubscriberList,
) {
    let registry = match state.pending_decisions.as_ref() {
        Some(r) => r,
        None => {
            tracing::debug!(
                "PermissionDecision for {prompt_id} ignored: pending_decisions registry not initialized"
            );
            return;
        }
    };

    match registry.resolve_prompt(prompt_id, decision) {
        Some(_payload) => {
            // First resolution: broadcast PermissionPromptResolved to ALL clients.
            let resolved_msg = ServerToClient::PermissionPromptResolved { prompt_id };
            broadcast_to_subscribers(subscribers, resolved_msg).await;
            tracing::debug!("PermissionDecision for {prompt_id} resolved; Resolved broadcast sent");
        }
        None => {
            // Second resolution attempt or already timed out: silently discard.
            tracing::debug!(
                "PermissionDecision for {prompt_id} silently discarded (not in registry)"
            );
        }
    }
}
