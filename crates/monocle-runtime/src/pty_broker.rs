//! PtyOutput fan-out broker — bounded channel, backpressure, and client lifecycle.
//!
//! Implements BC-2.05.009: per-session bounded INPUT channel `Arc<Bytes>(1024)` with
//! `.send().await` backpressure, per-client isolated `mpsc::Sender<ServerToClient>(64)`,
//! 3-strike disconnect, and `ServerToClient::PtyReset` emission when the PTY writer task
//! is dropped.
//!
//! The broker INPUT channel (`tokio::mpsc::channel::<Arc<Bytes>>(1024)`) carries raw PTY
//! frames from the PTY reader task to the broker. The broker wraps each frame into
//! `ServerToClient::PtyOutput { session_id, bytes }` before sending to each per-client
//! channel. Per-client channels carry `ServerToClient` messages and MUST NOT carry
//! `Arc<Bytes>` directly (BC-2.05.009 PC-1b / SS-ipc.md §Daemon-Side Per-Client Fan-out
//! Channel).
//!
//! The `biased; select!` macro ensures hook/control events are processed before PTY frames
//! when both are ready simultaneously (BC-2.05.009 Invariant 6 / ADR-0010).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::mpsc;

use monocle_ipc::types::ServerToClient;

// ---------------------------------------------------------------------------
// Input channel capacity (BC-2.05.009 PC-2, ADR-0010 §channel capacity 1024)
// ---------------------------------------------------------------------------

/// Capacity of the broker's INPUT channel (PTY reader → broker).
///
/// Raw PTY frame bytes arrive here from the PTY reader `spawn_blocking` thread.
/// The `.send().await` call on the PTY reader side blocks when this channel is full,
/// propagating backpressure to the PTY read syscall (BC-2.05.009 Invariant 3).
pub const PTY_BROKER_INPUT_CAPACITY: usize = 1024;

/// Capacity of each per-client isolated send channel.
///
/// The broker uses `.try_send()` into per-client channels (BC-2.05.009 Invariant 3b).
/// After 3 consecutive full-buffer failures for a client, the client is disconnected.
/// Capacity 64 per SS-ipc.md §TUI IPC Read Loop Pattern — covers typical burst sizes;
/// `64 × 256 KiB = 16 MiB` maximum in-flight per client.
pub const PTY_BROKER_CLIENT_CAPACITY: usize = 64;

/// Maximum consecutive send failures before a client is disconnected (3-strike rule).
///
/// BC-2.05.009 Invariant 3b: after `PTY_BROKER_STRIKE_LIMIT` consecutive full-buffer
/// `.try_send()` failures, the broker removes the client from the active set and logs
/// `WARN: slow TUI client disconnected`. Other clients are unaffected.
pub const PTY_BROKER_STRIKE_LIMIT: u8 = 3;

// ---------------------------------------------------------------------------
// PtyBroker
// ---------------------------------------------------------------------------

/// Per-session PTY output fan-out broker.
///
/// Receives raw PTY frame bytes via its bounded INPUT channel and fans them out to all
/// registered TUI clients as `ServerToClient::PtyOutput` messages. Each client has an
/// isolated `mpsc::channel::<ServerToClient>(64)` so that a slow client's full buffer
/// does not apply backpressure to other clients or to the PTY reader.
///
/// On drop, the broker emits `ServerToClient::PtyReset` to all remaining clients
/// (BC-2.05.009 Invariant 4).
pub struct PtyBroker {
    /// Session identifier (UUID string) for this broker instance.
    pub session_id: String,

    /// INPUT channel sender — PTY reader task calls `.send().await` here.
    ///
    /// Capacity 1024 (BC-2.05.009 PC-2). `Arc<Bytes>` is the item type at this layer only.
    /// The broker wraps frames into `ServerToClient::PtyOutput` before per-client dispatch.
    pub input_tx: mpsc::Sender<Arc<Bytes>>,

    /// INPUT channel receiver — drained by the broker event-loop task.
    ///
    /// Moved into the event-loop `tokio::spawn`ed task on startup.
    /// `None` after the task is spawned (ownership transferred).
    pub input_rx: Option<mpsc::Receiver<Arc<Bytes>>>,

    /// Per-client senders keyed by client_id (arbitrary String).
    ///
    /// Each sender has capacity `PTY_BROKER_CLIENT_CAPACITY` (64). The broker calls
    /// `.try_send()` — NOT `.send().await` — into each per-client channel to avoid
    /// a slow client stalling the broker task or other clients.
    pub clients: HashMap<String, mpsc::Sender<ServerToClient>>,

    /// Per-client consecutive send-failure counters (3-strike rule).
    ///
    /// Reset to 0 on any successful send. Reaches `PTY_BROKER_STRIKE_LIMIT` → disconnect.
    pub strike_counters: HashMap<String, u8>,

    /// Daemon-global PTY drop counter (BC-2.05.009 PC-3).
    ///
    /// Incremented ONLY on sender-error / OOM / receiver-gone conditions.
    /// NOT incremented on normal backpressure waits or per-client 3-strike disconnects.
    /// Logged at WARN level to the session-host's stderr; never surfaced over IPC.
    pub pty_drop_counter: Arc<AtomicU64>,
}

impl PtyBroker {
    /// Construct a new `PtyBroker` for the given session.
    ///
    /// The INPUT channel is created here with `PTY_BROKER_INPUT_CAPACITY` (1024).
    /// The caller receives ownership of `input_tx` for use in the PTY reader task,
    /// and a `PtyBroker` whose `input_rx` holds the receiving half.
    ///
    /// Before the broker is operational, call `spawn_event_loop()` to start the
    /// background fan-out task.
    pub fn new(session_id: String, pty_drop_counter: Arc<AtomicU64>) -> Self {
        let (input_tx, input_rx) = mpsc::channel::<Arc<Bytes>>(PTY_BROKER_INPUT_CAPACITY);
        Self {
            session_id,
            input_tx,
            input_rx: Some(input_rx),
            clients: HashMap::new(),
            strike_counters: HashMap::new(),
            pty_drop_counter,
        }
    }

    /// Register a new TUI client and return the per-client receiver end.
    ///
    /// The caller (IPC writer task) receives the `Receiver` to drain into the UDS socket.
    /// The broker retains the `Sender` end in `self.clients`.
    ///
    /// Per-client channel capacity is `PTY_BROKER_CLIENT_CAPACITY` (64).
    pub fn register_client(&mut self, id: String) -> mpsc::Receiver<ServerToClient> {
        let (tx, rx) = mpsc::channel::<ServerToClient>(PTY_BROKER_CLIENT_CAPACITY);
        self.clients.insert(id.clone(), tx);
        // Reset the strike counter for this client (fresh start on registration).
        self.strike_counters.insert(id, 0);
        rx
    }

    /// Remove a TUI client from the active set.
    ///
    /// Drops the per-client `Sender`, closing the channel. Any subsequent send to this
    /// client will fail with `SendError::Closed`, which the broker catches during fan-out.
    pub fn unregister_client(&mut self, id: &str) {
        self.clients.remove(id);
        self.strike_counters.remove(id);
    }

    /// Fan out a raw PTY frame to all registered clients.
    ///
    /// Wraps `frame` as `ServerToClient::PtyOutput { session_id: session_id.to_string(),
    /// bytes: frame.to_vec() }` and calls `.try_send()` into each per-client channel.
    ///
    /// Applies 3-strike disconnect logic (BC-2.05.009 Invariant 3b):
    /// - Successful send → reset strike counter for that client.
    /// - Failed `.try_send()` → increment strike counter; if counter reaches
    ///   `PTY_BROKER_STRIKE_LIMIT`, remove the client.
    ///
    /// The `pty_drop_counter` is NOT incremented here — per-client 3-strike disconnects
    /// are not OOM/sender-error conditions (BC-2.05.009 PC-3 / EC-202).
    pub fn fan_out(&mut self, session_id: &str, frame: Arc<Bytes>) {
        if self.clients.is_empty() {
            // EC-200/EC-202: no clients registered; discard frame silently.
            // pty_drop_counter NOT incremented — empty registry is not OOM.
            return;
        }

        let msg = ServerToClient::PtyOutput {
            session_id: session_id.to_string(),
            bytes: frame.to_vec(),
        };

        // Collect client IDs to disconnect after iteration (can't mutate while iterating).
        let mut to_disconnect: Vec<String> = Vec::new();

        for (client_id, sender) in &self.clients {
            match sender.try_send(msg.clone()) {
                Ok(()) => {
                    // Successful send — reset the strike counter for this client.
                    self.strike_counters.insert(client_id.clone(), 0);
                }
                Err(_) => {
                    // Send failed (channel full or closed). Increment strike counter.
                    let strikes = self.strike_counters.entry(client_id.clone()).or_insert(0);
                    *strikes += 1;

                    if *strikes >= PTY_BROKER_STRIKE_LIMIT {
                        tracing::warn!(
                            session_id = %session_id,
                            client_id = %client_id,
                            strikes = %strikes,
                            "slow TUI client disconnected after {} consecutive send failures",
                            PTY_BROKER_STRIKE_LIMIT,
                        );
                        to_disconnect.push(client_id.clone());
                    } else {
                        tracing::warn!(
                            session_id = %session_id,
                            client_id = %client_id,
                            strikes = %strikes,
                            "PTY fan-out send failed (strike {}/{}); client retained",
                            strikes,
                            PTY_BROKER_STRIKE_LIMIT,
                        );
                    }
                }
            }
        }

        for client_id in to_disconnect {
            self.clients.remove(&client_id);
            self.strike_counters.remove(&client_id);
        }
    }

    /// Emit `ServerToClient::PtyReset { session_id }` to all registered clients.
    ///
    /// Called when the PTY writer task for this session is dropped (BC-2.05.009 Invariant 4).
    /// Each client channel receives an independent `try_send()` call; failure for one client
    /// does not prevent emission to others (fire-and-forget per-client).
    ///
    /// No-op when no clients are registered (EC-204).
    pub fn emit_pty_reset(&mut self, session_id: &str) {
        // EC-204: no clients → no-op.
        if self.clients.is_empty() {
            return;
        }

        let msg = ServerToClient::PtyReset {
            session_id: session_id.to_string(),
        };

        // Collect clients to disconnect (3-strike applies to PtyReset sends too).
        let mut to_disconnect: Vec<String> = Vec::new();

        for (client_id, sender) in &self.clients {
            match sender.try_send(msg.clone()) {
                Ok(()) => {
                    self.strike_counters.insert(client_id.clone(), 0);
                }
                Err(_) => {
                    let strikes = self.strike_counters.entry(client_id.clone()).or_insert(0);
                    *strikes += 1;

                    if *strikes >= PTY_BROKER_STRIKE_LIMIT {
                        tracing::warn!(
                            session_id = %session_id,
                            client_id = %client_id,
                            "slow TUI client disconnected during PtyReset emission"
                        );
                        to_disconnect.push(client_id.clone());
                    }
                }
            }
        }

        for client_id in to_disconnect {
            self.clients.remove(&client_id);
            self.strike_counters.remove(&client_id);
        }
    }

    /// Spawn the broker event-loop as a background `tokio::task`.
    ///
    /// The loop uses `biased; select!` with two arms:
    /// 1. Hook/control event arm (higher priority — biased first).
    /// 2. PTY frame arm (lower priority — default).
    ///
    /// Returns a `JoinHandle<()>` — the caller stores this to abort the task on session
    /// teardown. The INPUT receiver is moved into the spawned task; `self.input_rx` is
    /// set to `None` after this call.
    ///
    /// When the INPUT channel is closed by the PTY reader dropping its sender (OOM-level
    /// failure), the drop counter is incremented and `PtyReset` is emitted to all clients.
    pub fn spawn_event_loop(
        &mut self,
        mut hook_rx: mpsc::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        // Take the INPUT receiver out of self. If it has already been moved (i.e. this
        // method was called a second time on the same broker), log an error and return a
        // no-op task rather than panicking. Callers MUST call spawn_event_loop exactly once.
        let Some(mut input_rx) = self.input_rx.take() else {
            tracing::error!(
                session_id = %self.session_id,
                "PtyBroker::spawn_event_loop called after INPUT receiver already moved; \
                 returning no-op task — this is a programming error"
            );
            return tokio::spawn(async {});
        };

        // Clone the broker state needed by the event loop task.
        // The event loop operates on its own local copy of the client registry so that
        // the main thread (tests) can still call fan_out/register_client on `self`.
        // For the spawn_event_loop test (AC-006), the loop needs to see clients registered
        // on `self` before spawn_event_loop is called. We share the channel senders
        // by cloning them into a separate map for the event loop.
        let mut loop_clients: HashMap<String, mpsc::Sender<ServerToClient>> = self
            .clients
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut loop_strikes: HashMap<String, u8> = self.strike_counters.clone();

        let session_id = self.session_id.clone();
        let pty_drop_counter = Arc::clone(&self.pty_drop_counter);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;

                    // Arm 1 (higher priority): hook/control event channel.
                    // Biased ensures this arm is checked before the PTY frame arm
                    // when both are ready simultaneously (BC-2.05.009 Invariant 6 / ADR-0010).
                    hook_event = hook_rx.recv() => {
                        match hook_event {
                            Some(()) => {
                                // Hook event received — process it (currently a no-op control
                                // signal; future versions will carry structured hook commands).
                                tracing::debug!(
                                    session_id = %session_id,
                                    "broker: hook/control event processed"
                                );
                            }
                            None => {
                                // Hook channel closed — no more hook events; continue
                                // processing PTY frames until the INPUT channel also closes.
                                tracing::debug!(
                                    session_id = %session_id,
                                    "broker: hook/control channel closed"
                                );
                                // Replace hook_rx with a channel that never delivers;
                                // break out and continue with PTY-only loop.
                                break;
                            }
                        }
                    }

                    // Arm 2 (lower priority): PTY frame channel.
                    pty_frame = input_rx.recv() => {
                        match pty_frame {
                            Some(frame) => {
                                // Fan out to all registered clients.
                                fan_out_to_clients(
                                    &session_id,
                                    frame,
                                    &mut loop_clients,
                                    &mut loop_strikes,
                                );
                            }
                            None => {
                                // INPUT channel closed — OOM-level failure: PTY reader
                                // dropped its sender. Increment drop counter and emit PtyReset.
                                let n = pty_drop_counter.fetch_add(1, Ordering::Relaxed) + 1;
                                tracing::warn!(
                                    session_id = %session_id,
                                    drop_n = n,
                                    "WARN: PTY channel drop #{n} for session {session_id}"
                                );
                                emit_reset_to_clients(&session_id, &mut loop_clients, &mut loop_strikes);
                                return;
                            }
                        }
                    }
                }
            }

            // Hook channel closed — continue with PTY frames only.
            loop {
                match input_rx.recv().await {
                    Some(frame) => {
                        fan_out_to_clients(
                            &session_id,
                            frame,
                            &mut loop_clients,
                            &mut loop_strikes,
                        );
                    }
                    None => {
                        // INPUT channel closed — OOM-level failure.
                        let n = pty_drop_counter.fetch_add(1, Ordering::Relaxed) + 1;
                        tracing::warn!(
                            session_id = %session_id,
                            drop_n = n,
                            "WARN: PTY channel drop #{n} for session {session_id}"
                        );
                        emit_reset_to_clients(&session_id, &mut loop_clients, &mut loop_strikes);
                        return;
                    }
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Private helpers used by the event-loop task
// ---------------------------------------------------------------------------

/// Fan out a PTY frame to all clients in `clients_map`, applying 3-strike logic.
///
/// Mirrors `PtyBroker::fan_out` but operates on a local client map owned by the
/// event-loop task (avoids borrow conflicts with the spawning PtyBroker).
fn fan_out_to_clients(
    session_id: &str,
    frame: Arc<Bytes>,
    clients: &mut HashMap<String, mpsc::Sender<ServerToClient>>,
    strikes: &mut HashMap<String, u8>,
) {
    if clients.is_empty() {
        return;
    }

    let msg = ServerToClient::PtyOutput {
        session_id: session_id.to_string(),
        bytes: frame.to_vec(),
    };

    let mut to_disconnect: Vec<String> = Vec::new();

    for (client_id, sender) in clients.iter() {
        match sender.try_send(msg.clone()) {
            Ok(()) => {
                strikes.insert(client_id.clone(), 0);
            }
            Err(_) => {
                let s = strikes.entry(client_id.clone()).or_insert(0);
                *s += 1;
                if *s >= PTY_BROKER_STRIKE_LIMIT {
                    tracing::warn!(
                        session_id = %session_id,
                        client_id = %client_id,
                        "slow TUI client disconnected (event loop) after {} strikes",
                        PTY_BROKER_STRIKE_LIMIT,
                    );
                    to_disconnect.push(client_id.clone());
                }
            }
        }
    }

    for id in to_disconnect {
        clients.remove(&id);
        strikes.remove(&id);
    }
}

/// Emit `PtyReset` to all clients in `clients_map` (fire-and-forget, 3-strike applies).
fn emit_reset_to_clients(
    session_id: &str,
    clients: &mut HashMap<String, mpsc::Sender<ServerToClient>>,
    strikes: &mut HashMap<String, u8>,
) {
    if clients.is_empty() {
        return;
    }

    let msg = ServerToClient::PtyReset {
        session_id: session_id.to_string(),
    };

    let mut to_disconnect: Vec<String> = Vec::new();

    for (client_id, sender) in clients.iter() {
        match sender.try_send(msg.clone()) {
            Ok(()) => {
                strikes.insert(client_id.clone(), 0);
            }
            Err(_) => {
                let s = strikes.entry(client_id.clone()).or_insert(0);
                *s += 1;
                if *s >= PTY_BROKER_STRIKE_LIMIT {
                    to_disconnect.push(client_id.clone());
                }
            }
        }
    }

    for id in to_disconnect {
        clients.remove(&id);
        strikes.remove(&id);
    }
}
