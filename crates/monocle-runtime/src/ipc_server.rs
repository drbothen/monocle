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
                    // S-034: KillSession handler (BC-2.08.003 §IPC handler arm)
                    Ok(ClientToServer::KillSession { session_id }) => {
                        handle_kill_session(session_id, &tx, &state).await;
                    }
                    // S-035: AttachSession handler (BC-2.08.007 §attach_session)
                    Ok(ClientToServer::AttachSession { session_id }) => {
                        handle_attach_session(session_id, &tx, &state).await;
                    }
                    // S-035: DetachSession handler (BC-2.08.007 §detach_session)
                    Ok(ClientToServer::DetachSession { session_id }) => {
                        handle_detach_session(session_id, &tx, &state).await;
                    }
                    // S-040: KeyInput handler (BC-2.09.002 — keyboard/paste forwarding)
                    // Forward bytes to the session-host via SessionManager::send_key_input().
                    Ok(ClientToServer::KeyInput { session_id, bytes }) => {
                        handle_key_input(session_id, bytes, &tx, &state).await;
                    }
                    // S-042: ResizePane handler (BC-2.09.006 — PTY resize after 50ms debounce)
                    // Forward resize to the session-host via SessionManager::resize_session().
                    Ok(ClientToServer::ResizePane { session_id, rows, cols }) => {
                        handle_resize_pane(session_id, rows, cols, &tx, &state).await;
                    }
                    // S-047: RenameSession handler (BC-2.05.010 RenameSession PC-4a)
                    // Updates session display_name via SessionManager::rename_session().
                    Ok(ClientToServer::RenameSession { session_id, new_name }) => {
                        handle_rename_session(session_id, new_name, &tx, &state).await;
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

    // SessionManager owns the canonical hooks-settings.json path (S-038 single-writer mandate).
    // The path passed to with_daemon_fields() is immediately overwritten by spawn_session()
    // from self.hooks_settings_path — so derive an empty placeholder here.
    let hooks_settings_path = std::path::PathBuf::new();

    // Step 1: generate session_id via the injectable seam (EC-152 / Ruling F).
    // Production: state.session_id_gen is UuidV4Generator → uuid::Uuid::new_v4().to_string().
    // Tests: may inject SequencedIdGenerator to force deterministic collision sequences.
    let session_id = state.session_id_gen.next_id();

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
    //
    // IMPORTANT: The lock guard from `sm.lock().await` lives for the duration of the
    // entire `match` scrutinee expression. To avoid a self-deadlock when the collision
    // arm tries to re-acquire the same mutex, we must store the result in a let binding
    // first (which drops the guard), then match on the stored result.
    let spawn_result = sm.lock().await.spawn_session(opts.clone()).await;
    match spawn_result {
        Ok(_) => {
            // Success: spawn_session publishes SessionStateChanged{Launching} + SessionListUpdate.
        }
        Err(SessionError::SessionIdCollision { .. }) => {
            // EC-152: first collision — regenerate via the seam and retry once.
            let new_id = state.session_id_gen.next_id();

            // Send a second SpawnAck with the regenerated ID before retry.
            let _ = client_tx
                .send(ServerToClient::SpawnAck {
                    session_id: new_id.clone(),
                })
                .await;

            let opts2 = opts.with_daemon_fields(new_id, hooks_settings_path);
            // Lock is already released (guard dropped after spawn_result was bound above).
            let retry_result = sm.lock().await.spawn_session(opts2).await;
            match retry_result {
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
/// Exposes the private IPC handler to tests in `session_manager/mod.rs` and
/// integration tests (tests/ directory) that exercise:
/// - BLOCKER-001 regression guard (no panic from todo!())
/// - AC-001/AC-012: SpawnAck ordering before SessionStateChanged{Launching}
/// - EC-152: UUID collision retry in the IPC handler
///
/// Available under both `cfg(test)` (unit tests) and `feature = "test-utils"`
/// (integration tests linked via dev-dependency with test-utils feature).
///
/// NEVER call this from production code. The cfg guard enforces that.
#[cfg(any(test, feature = "test-utils"))]
pub async fn handle_spawn_session_pub(
    opts: monocle_core::engine::SpawnOptions,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    handle_spawn_session(opts, client_tx, state).await
}

// ---------------------------------------------------------------------------
// S-034: KillSession IPC handler
// ---------------------------------------------------------------------------

/// Handle a `ClientToServer::KillSession` message from a TUI client.
///
/// IPC handler steps (BC-2.08.003 §IPC handler arm — S-034):
/// 1. Retrieve session_manager from daemon state.
/// 2. Call `session_manager.kill_session(session_id)`.
/// 3. On error: send `ServerToClient::Error { code, message }` to requesting client.
/// 4. On success: `kill_session()` has already emitted `SessionStateChanged{Terminating}` +
///    `SessionListUpdate` to all clients under the sessions mutex (BC-2.08.008 invariant 4).
async fn handle_kill_session(
    session_id: String,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    use crate::session_manager::{session_error_to_code, IpcOp};

    let sm = match state.session_manager.as_ref() {
        Some(sm) => sm,
        None => {
            tracing::error!("handle_kill_session: session_manager is None (daemon wiring bug)");
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: "invalid_request".to_string(),
                    message: "session_manager not initialized".to_string(),
                })
                .await;
            return;
        }
    };

    let kill_result = sm.lock().await.kill_session(&session_id).await;
    match kill_result {
        Ok(()) => {
            // kill_session() emitted SessionStateChanged{Terminating} + SessionListUpdate to all
            // clients (BC-2.08.008 Invariant 4). No additional response to requesting client.
        }
        Err(e) => {
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::Kill, &e).to_string(),
                    message: e.to_string(),
                })
                .await;
        }
    }
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

// ---------------------------------------------------------------------------
// S-035: AttachSession IPC handler
// ---------------------------------------------------------------------------

/// Handle a `ClientToServer::AttachSession` message from a TUI client.
///
/// IPC handler steps (BC-2.08.007 §attach_session — S-035):
/// 1. Retrieve session_manager from daemon state.
/// 2. Call `session_manager.attach_session(session_id)`.
/// 3. On error: send `ServerToClient::Error { code, message }` to requesting client.
/// 4. On success: `attach_session()` has already emitted `SessionStateChanged{Running}` +
///    `SessionListUpdate` to all clients under the sessions mutex (BC-2.08.008 Invariant 4).
///
/// The `session_manager is None` branch is an unreachable-in-production daemon-wiring guard
/// (the daemon always initializes session_manager before starting the IPC listener).
async fn handle_attach_session(
    session_id: String,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    use crate::session_manager::{session_error_to_code, IpcOp};

    let sm = match state.session_manager.as_ref() {
        Some(sm) => sm,
        None => {
            tracing::error!("handle_attach_session: session_manager is None (daemon wiring bug)");
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: "invalid_request".to_string(),
                    message: "session_manager not initialized".to_string(),
                })
                .await;
            return;
        }
    };

    let attach_result = sm.lock().await.attach_session(&session_id).await;
    match attach_result {
        Ok(()) => {
            // attach_session() emitted SessionStateChanged{Running} + SessionListUpdate to all
            // clients (BC-2.08.008 Invariant 4). No additional response to requesting client.
        }
        Err(e) => {
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::Attach, &e).to_string(),
                    message: e.to_string(),
                })
                .await;
        }
    }
}

// ---------------------------------------------------------------------------
// S-035: DetachSession IPC handler
// ---------------------------------------------------------------------------

/// Handle a `ClientToServer::DetachSession` message from a TUI client.
///
/// IPC handler steps (BC-2.08.007 §detach_session — S-035):
/// 1. Retrieve session_manager from daemon state.
/// 2. Call `session_manager.detach_session(session_id)`.
/// 3. On error: send `ServerToClient::Error { code, message }` to requesting client.
/// 4. On success: `detach_session()` has already emitted `SessionStateChanged{Detached}` +
///    `SessionListUpdate` to all clients under the sessions mutex (BC-2.08.008 Invariant 4).
///
/// The `session_manager is None` branch is an unreachable-in-production daemon-wiring guard
/// (the daemon always initializes session_manager before starting the IPC listener).
async fn handle_detach_session(
    session_id: String,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    use crate::session_manager::{session_error_to_code, IpcOp};

    let sm = match state.session_manager.as_ref() {
        Some(sm) => sm,
        None => {
            tracing::error!("handle_detach_session: session_manager is None (daemon wiring bug)");
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: "invalid_request".to_string(),
                    message: "session_manager not initialized".to_string(),
                })
                .await;
            return;
        }
    };

    let detach_result = sm.lock().await.detach_session(&session_id).await;
    match detach_result {
        Ok(()) => {
            // detach_session() emitted SessionStateChanged{Detached} + SessionListUpdate to all
            // clients (BC-2.08.008 Invariant 4). No additional response to requesting client.
        }
        Err(e) => {
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::Detach, &e).to_string(),
                    message: e.to_string(),
                })
                .await;
        }
    }
}

/// Handle a `ClientToServer::ResizePane` message from a TUI client.
///
/// Routes the resize request to `SessionManager::resize_session()`, which forwards
/// `DaemonToHost::Resize { rows, cols }` to the session-host. The session-host calls
/// `pty.resize()` and `parser.set_size()`, causing the harness child to receive `SIGWINCH`.
///
/// Mirrors the `handle_key_input` dispatch pattern (S-040). Called after the TUI's 50ms
/// debounce window expires (BC-2.09.006 postcondition 2).
///
/// # Zero-dimension clamp (BC-2.09.006 AC-014 / EC-239 / BC-2.05.010 Inv-5)
///
/// `rows` and `cols` are clamped to a minimum of 1 at the handler boundary before being
/// passed to `resize_session`. Pre-clamp zero values must NOT reach the session-host.
/// A `tracing::warn!` is emitted when clamping occurs.
///
/// # WARN-drop carve-out (BC-2.09.006 AC-013/AC-016 / BC-2.05.010 Inv-6)
///
/// ALL error paths from `resize_session` — including `session_manager is None`,
/// `SessionNotFound`, `SessionNotReady`, and `SessionHostDead` / IO errors — are
/// WARN-dropped: the error is logged at WARN level and the handler returns without
/// sending `ServerToClient::Error` to the client. This is an explicit carve-out
/// from the general error-propagation policy for resize messages.
///
/// Rationale: a resize failure is benign (the session may have terminated mid-resize);
/// propagating an error frame to the TUI would require the TUI to handle it and could
/// cause spurious error overlays during normal session teardown.
async fn handle_resize_pane(
    session_id: String,
    rows: u16,
    cols: u16,
    // WARN-drop carve-out (BC-2.09.006 AC-013/AC-016 / BC-2.05.010 Inv-6): this handler
    // never sends ServerToClient::Error for resize failures. The parameter is kept in the
    // signature to mirror the handle_key_input dispatch pattern and to allow future callers
    // to pass a client_tx without API breakage if the carve-out policy changes.
    _client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    // Validate session_id is UUID format before any log emission (CWE-532 mitigation).
    // Rejects client-supplied strings that are not valid UUIDs before they can appear in
    // any structured log field. Matches the uuid::Uuid::parse_str pattern used throughout
    // session_manager (see resize_session, kill_session, etc.).
    if uuid::Uuid::parse_str(&session_id).is_err() {
        tracing::warn!(
            session_id_len = session_id.len(),
            "ResizePane: invalid session_id (not UUID format) — dropped"
        );
        return;
    }

    // HIGH-003 / AC-014 / EC-239 / BC-2.05.010 Inv-5: zero-dimension clamp.
    // Clamp rows and cols to minimum 1 BEFORE calling resize_session.
    // Pre-clamp zeros must never reach the session-host PTY.
    let rows = if rows == 0 {
        tracing::warn!(
            session_id = %session_id,
            "handle_resize_pane: rows=0 clamped to rows=1 (EC-239 / BC-2.05.010 Inv-5)"
        );
        1u16
    } else {
        rows
    };
    let cols = if cols == 0 {
        tracing::warn!(
            session_id = %session_id,
            "handle_resize_pane: cols=0 clamped to cols=1 (EC-239 / BC-2.05.010 Inv-5)"
        );
        1u16
    } else {
        cols
    };

    // HIGH-002 / AC-013 / BC-2.05.010 Inv-6: WARN-drop when session_manager is None.
    // NO ServerToClient::Error is sent for any resize failure path.
    let sm = match state.session_manager.as_ref() {
        Some(sm) => sm,
        None => {
            tracing::warn!(
                session_id = %session_id,
                "handle_resize_pane: session_manager is None (daemon wiring bug) — \
                 ResizePane WARN-dropped per BC-2.09.006 AC-013/BC-2.05.010 Inv-6"
            );
            // WARN-drop: no ServerToClient::Error sent (ResizePane carve-out).
            return;
        }
    };

    let result = sm
        .lock()
        .await
        .resize_session(&session_id, rows, cols)
        .await;
    match result {
        Ok(()) => {
            // Resize forwarded successfully; no response needed (fire-and-continue).
            tracing::trace!(
                session_id = %session_id,
                rows,
                cols,
                "handle_resize_pane: resize_session forwarded to session-host"
            );
        }
        Err(e) => {
            // HIGH-002 / AC-013 / AC-016 / BC-2.05.010 Inv-6: WARN-drop carve-out.
            // All resize_session errors (SessionNotFound, SessionNotReady, SessionHostDead,
            // Io) are logged at WARN and silently discarded — no ServerToClient::Error.
            tracing::warn!(
                session_id = %session_id,
                rows,
                cols,
                error = %e,
                "handle_resize_pane: resize_session failed — WARN-dropped \
                 (BC-2.09.006 AC-013/AC-016 / BC-2.05.010 Inv-6)"
            );
            // WARN-drop: no ServerToClient::Error sent (ResizePane carve-out).
        }
    }
}

/// Test seam: public wrapper around [`handle_resize_pane`] for external integration tests.
///
/// Exposes `handle_resize_pane` to tests in `crates/monocle-runtime/tests/` that must
/// call the real IPC handler path (with zero-dim clamping, `session_manager` look-up,
/// and `ServerToClient::Error` / WARN-drop behavior) rather than calling `resize_session`
/// directly. Mirrors the `handle_spawn_session_pub` pattern.
///
/// AC-013 / AC-014 / AC-016 (BC-2.09.006): the WARN-drop policy is implemented in
/// `handle_resize_pane`, not in `resize_session`. Tests that want to verify zero-dim
/// clamping or WARN-drop semantics at the handler boundary MUST go through this seam.
///
/// Available under both `cfg(test)` (unit tests) and `feature = "test-utils"`
/// (integration tests linked via dev-dependency with test-utils feature).
///
/// NEVER call this from production code. The cfg guard enforces that.
#[cfg(any(test, feature = "test-utils"))]
pub async fn handle_resize_pane_pub(
    session_id: String,
    rows: u16,
    cols: u16,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    handle_resize_pane(session_id, rows, cols, client_tx, state).await
}

/// Handle a `ClientToServer::KeyInput` message from a TUI client.
///
/// Forwards the keyboard bytes to the session-host via `SessionManager::send_key_input()`,
/// which writes `DaemonToHost::KeyInput { bytes }` to the per-session control connection.
///
/// # Error behaviour (BC-2.09.002)
///
/// - `SessionNotFound` / `SessionNotReady`: logged at WARN; `ServerToClient::Error` sent
///   to the requesting client. The session may have terminated between the TUI sending
///   the `KeyInput` and the daemon processing it — this is a benign race.
/// - Write errors to the session-host: logged at WARN; propagated as `ServerToClient::Error`.
async fn handle_key_input(
    session_id: String,
    bytes: Vec<u8>,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    use crate::session_manager::{session_error_to_code, IpcOp};

    let sm = match state.session_manager.as_ref() {
        Some(sm) => sm,
        None => {
            tracing::error!("handle_key_input: session_manager is None (daemon wiring bug)");
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: "invalid_request".to_string(),
                    message: "session_manager not initialized".to_string(),
                })
                .await;
            return;
        }
    };

    let result = sm.lock().await.send_key_input(&session_id, bytes).await;
    match result {
        Ok(()) => {
            // Bytes forwarded successfully; no response needed (fire-and-continue).
        }
        Err(e) => {
            tracing::warn!(
                session_id = %session_id,
                error = %e,
                "handle_key_input: send_key_input failed"
            );
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::KeyInput, &e).to_string(),
                    message: e.to_string(),
                })
                .await;
        }
    }
}

// ---------------------------------------------------------------------------
// S-047: RenameSession IPC handler (BC-2.05.010 RenameSession PC-4a)
// ---------------------------------------------------------------------------

/// Handle a `ClientToServer::RenameSession` message from a TUI client.
///
/// IPC handler steps (BC-2.05.010 RenameSession PC-4a):
/// 1. Retrieve session_manager from daemon state.
/// 2. Call `session_manager.rename_session(session_id, new_name)`.
/// 3. On success: `rename_session()` updates `display_name` in the registry and
///    publishes `SessionListUpdate` to all clients (BC-2.08.005 / BC-2.08.008 PC-4a).
/// 4. On error: send `ServerToClient::Error { code: "rename_failed", message }`.
///
/// Rename is permitted when the session is in `Launching` or `Running` state.
/// On `Terminated` state, `rename_session()` returns `SessionError::SessionNotFound`
/// or a lifecycle error that maps to `"rename_failed"` per the 12-code taxonomy.
async fn handle_rename_session(
    session_id: String,
    new_name: String,
    client_tx: &tokio::sync::mpsc::Sender<ServerToClient>,
    state: &DaemonState,
) {
    use crate::session_manager::{session_error_to_code, IpcOp};

    let sm = match state.session_manager.as_ref() {
        Some(sm) => sm,
        None => {
            tracing::error!("handle_rename_session: session_manager is None (daemon wiring bug)");
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: "invalid_request".to_string(),
                    message: "session_manager not initialized".to_string(),
                })
                .await;
            return;
        }
    };

    let result = sm.lock().await.rename_session(&session_id, new_name).await;
    match result {
        Ok(()) => {
            // rename_session() emitted SessionListUpdate to all clients (BC-2.08.008 PC-4a).
            // No additional response to the requesting client.
        }
        Err(e) => {
            tracing::warn!(
                session_id = %session_id,
                error = %e,
                "handle_rename_session: rename_session failed"
            );
            let _ = client_tx
                .send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::Rename, &e).to_string(),
                    message: e.to_string(),
                })
                .await;
        }
    }
}
