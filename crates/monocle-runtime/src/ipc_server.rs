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

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::net::UnixListener;

use monocle_ipc::error::IpcError;
use monocle_ipc::framing::{write_framed, write_framed_bytes, MAX_MESSAGE_BYTES};
use monocle_ipc::server::{register_subscriber, remove_subscriber, SubscriberList};

/// Per-client outbound channel capacity — matches `monocle_ipc::server::CLIENT_CHANNEL_CAPACITY`.
const CLIENT_CHANNEL_CAPACITY: usize = monocle_ipc::server::CLIENT_CHANNEL_CAPACITY;
use monocle_ipc::types::{ClientToServer, ServerToClient};

use crate::state::{snapshot_initial_state, DaemonState};

/// Entry point for the daemon UDS connection accept loop (BC-2.05.002 postcondition PC-1).
///
/// Accepts incoming TUI client connections from `listener` and spawns a dedicated Tokio task
/// per client. Runs until the listener is closed (daemon shutdown) OR a shutdown signal is
/// received on `shutdown_rx`.
///
/// # Parameters
///
/// - `listener`: The already-bound `UnixListener` from the daemon bind step (S-021).
/// - `state`: Shared daemon state; used to take the `InitialState` snapshot and look up the
///   pending-decision registry.
/// - `subscribers`: Shared fan-out subscriber list to which new client senders are added.
/// - `mut shutdown_rx`: Watch receiver cloned from `DaemonState.shutdown_rx`. When the value
///   changes to `true` (graceful shutdown), the accept loop terminates cleanly. In-flight
///   per-client tasks are NOT aborted — they are allowed to complete naturally (draining
///   any queued messages before closing their socket).
pub async fn run_accept_loop(
    listener: UnixListener,
    state: Arc<DaemonState>,
    subscribers: SubscriberList,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            // Bias toward shutdown check so a pending shutdown signal
            // is never delayed by a queued accept event.
            biased;

            // Shutdown signal received — terminate accept loop cleanly.
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::debug!("run_accept_loop: shutdown signal received; stopping accept loop");
                    break;
                }
                // Value changed to false (reset) — not a shutdown; continue.
            }

            // New TUI client connection arrived.
            accept_result = listener.accept() => {
                match accept_result {
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
    }
    tracing::debug!("run_accept_loop: exited");
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
    // The returned `disconnect_notify` is triggered by `broadcast_to_subscribers` when
    // this client's channel is found full (slow-client disconnect, BC-2.05.004 EC-005).
    let disconnect_notify = register_subscriber(&subscribers, tx.clone()).await;

    // Increment TUI attachment count now that the client is registered.
    // BC-2.01.002 PC-1: tui_attached reports true when count > 0.
    state.tui_attached_count.fetch_add(1, Ordering::SeqCst);

    // Step 3 + 4: Take snapshot and send InitialState as the first message.
    match send_initial_state(&mut write_half, &state).await {
        Ok(()) => {}
        Err(e) => {
            tracing::error!("failed to send InitialState to client: {e}");
            remove_subscriber(&subscribers, &tx).await;
            state.tui_attached_count.fetch_sub(1, Ordering::SeqCst);
            return;
        }
    }

    // Step 5: Concurrent per-client send loop + receive loop + slow-disconnect signal.
    // The send loop drains the mpsc channel to the client's write half.
    // The receive loop reads ClientToServer messages and dispatches them.
    // The disconnect branch exits when broadcast_to_subscribers fires the slow-disconnect
    // signal (TrySendError::Full, BC-2.05.004 EC-005).
    //
    // Use tokio::select! to run all three concurrently. When any branch exits
    // (EOF, error, channel closed, or slow-disconnect signal), cleanup and return.

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
                    // S-033: SpawnSession handler (BC-2.08.001 §IPC handler pattern)
                    Ok(ClientToServer::SpawnSession { opts }) => {
                        handle_spawn_session(opts, &tx, &state).await;
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

            // Slow-disconnect signal: broadcast_to_subscribers removed this client from the
            // fan-out list because its channel was full (BC-2.05.004 EC-005). Break out of
            // the loop here so that write_half and read_half are dropped, closing the UDS
            // socket. The slow client will observe EOF on its read side.
            _ = disconnect_notify.notified() => {
                tracing::debug!("TUI client slow-disconnect signal received; closing connection");
                break;
            }
        }
    }

    // Step 6: Remove client from fan-out subscriber list on disconnect;
    // decrement TUI attachment count (BC-2.01.002 PC-1 — F-ADV2-HIGH-003).
    remove_subscriber(&subscribers, &tx).await;
    state.tui_attached_count.fetch_sub(1, Ordering::SeqCst);
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

    // Serialize once (F-ADV2-MED-004): use the payload for both the 256 KiB guard check
    // and the framed write. The previous double-serialize pattern (serialize → size check →
    // write_framed serializes again) wasted CPU and risked byte-count divergence.
    // BC-2.05.002 PC-4, AC-004: ERROR log must include the byte count.
    let payload = serde_json::to_vec(&msg)?;
    if payload.len() > MAX_MESSAGE_BYTES {
        tracing::error!(
            "InitialState for client exceeds 256 KiB limit ({} bytes)",
            payload.len()
        );
        return Err(IpcError::MessageTooLarge);
    }

    write_framed_bytes(writer, &payload).await
}

/// Broadcast a `ServerToClient` message to all connected TUI clients.
///
/// Implements the drain-and-retain pattern (slow-client disconnect):
/// - For each entry: calls `try_send(msg.clone())` on `entry.tx`.
/// - On `TrySendError::Full` (slow client): removes the entry, logs WARN, and fires
///   `entry.disconnect.notify_one()` so the per-client task's `select!` branch exits,
///   closing the UDS socket (BC-2.05.004 EC-005).
/// - On `TrySendError::Closed` (disconnected): silently removes the entry.
///
/// This is the canonical broadcast helper; all broadcast sites MUST use it to ensure
/// consistent slow-client handling (BC-2.05.004 EC-005).
pub async fn broadcast_to_subscribers(subscribers: &SubscriberList, msg: ServerToClient) {
    use monocle_ipc::server::ClientEntry;

    let mut subs = subscribers.lock().await;
    let mut live: Vec<ClientEntry> = Vec::with_capacity(subs.len());

    for entry in subs.drain(..) {
        match entry.tx.try_send(msg.clone()) {
            Ok(()) => live.push(entry),
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                // Slow client — send buffer full. Signal per-client task to close connection,
                // then log WARN (BC-2.05.004 EC-005).
                entry.disconnect.notify_one();
                tracing::warn!("removed slow TUI client during broadcast (send buffer full)");
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                // Disconnected client — silently remove.
            }
        }
    }

    *subs = live;
}

// ---------------------------------------------------------------------------
// S-033: SpawnSession IPC handler
// ---------------------------------------------------------------------------

/// Handle a `ClientToServer::SpawnSession` message from a TUI client.
///
/// IPC handler canonical steps (BC-2.08.001 §IPC handler pattern, F-P41-IMP-001):
/// 1. Generate UUID v4 → session_id.
/// 2. Send `ServerToClient::SpawnAck { session_id }` to requesting client ONLY.
/// 3. Fill daemon-owned fields via `opts.with_daemon_fields(session_id, hooks_settings_path)`.
/// 4. Call `spawn_session(opts)` on the session_manager.
/// 5. On error: send `ServerToClient::Error { code, message }` to requesting client.
///
/// EC-152: on first `SessionIdCollision`, regenerate UUID once, send a second SpawnAck with
/// the new ID, then retry. On second collision, send `Error{code:"session_id_collision"}`.
///
/// `spawn_session()` itself publishes `SessionStateChanged{Launching}` + `SessionListUpdate`
/// to the broker on success (BC-2.08.001 PC-5, BC-2.08.008 Invariant 4).
async fn handle_spawn_session(
    opts: monocle_core::engine::SpawnOptions,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    use crate::session_manager::{session_error_to_code, IpcOp, SessionError};

    // Derive hooks_settings_path from lock_file_path parent + "hooks-settings.json".
    // If lock_file_path is empty (test path), fall back to temp_dir.
    let hooks_settings_path = if state.lock_file_path.is_empty() {
        std::env::temp_dir().join("hooks-settings.json")
    } else {
        std::path::Path::new(&state.lock_file_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"))
            .join("hooks-settings.json")
    };

    // Step 1: generate UUID v4 for session_id.
    let session_id = uuid::Uuid::new_v4().to_string();

    // Step 2 (AC-012): send SpawnAck BEFORE spawn_session() — must be the first message.
    let _ = client_tx
        .send(ServerToClient::SpawnAck {
            session_id: session_id.clone(),
        })
        .await;

    // Step 3: fill daemon-owned fields.
    let opts = opts.with_daemon_fields(session_id.clone(), hooks_settings_path.clone());

    // Retrieve session_manager (must be Some after MED-011 wiring).
    let sm = match state.session_manager.as_ref() {
        Some(sm) => sm,
        None => {
            tracing::error!("handle_spawn_session: session_manager is None (daemon wiring bug)");
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: "invalid_request".to_string(),
                    message: "session_manager not initialized".to_string(),
                })
                .await;
            return;
        }
    };

    // Step 4: call spawn_session().
    match sm.lock().await.spawn_session(opts.clone()).await {
        Ok(_) => {
            // Success: spawn_session publishes SessionStateChanged{Launching} + SessionListUpdate.
        }
        Err(SessionError::SessionIdCollision { .. }) => {
            // EC-152: first collision — regenerate UUID once and retry.
            let new_id = uuid::Uuid::new_v4().to_string();

            // Send a second SpawnAck with the regenerated ID before retry.
            let _ = client_tx
                .send(ServerToClient::SpawnAck {
                    session_id: new_id.clone(),
                })
                .await;

            let opts2 = opts.with_daemon_fields(new_id, hooks_settings_path);
            match sm.lock().await.spawn_session(opts2).await {
                Ok(_) => {}
                Err(e) => {
                    // EC-152: second collision → send error.
                    let _ = client_tx
                        .send(ServerToClient::Error {
                            code: session_error_to_code(IpcOp::Spawn, &e).to_string(),
                            message: e.to_string(),
                        })
                        .await;
                }
            }
        }
        Err(e) => {
            // Step 5: on error, send Error to requesting client.
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::Spawn, &e).to_string(),
                    message: e.to_string(),
                })
                .await;
        }
    }
}

/// Test-only public wrapper for `handle_spawn_session`.
///
/// Exposes the private IPC handler to tests in `session_manager/mod.rs` that exercise:
/// - BLOCKER-001 regression guard (no panic from todo!())
/// - AC-001/AC-012: SpawnAck ordering before SessionStateChanged{Launching}
/// - EC-152: UUID collision retry in the IPC handler
///
/// NEVER call this from production code. The `#[cfg(test)]` guard enforces that.
#[cfg(test)]
pub async fn handle_spawn_session_pub(
    opts: monocle_core::engine::SpawnOptions,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    handle_spawn_session(opts, client_tx, state).await
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
