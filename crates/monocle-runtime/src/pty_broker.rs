//! PtyOutput fan-out broker — bounded channel, backpressure, and client lifecycle.
//!
//! Implements BC-2.05.009: per-session bounded INPUT channel `Arc<Bytes>(1024)` with
//! `.send().await` backpressure, per-client isolated `mpsc::Sender<ServerToClient>(64)`,
//! 3-strike disconnect, and `ServerToClient::PtyReset` emission on broker task drop.
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
use std::sync::atomic::AtomicU64;
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
    #[allow(clippy::todo)]
    pub fn register_client(&mut self, _id: String) -> mpsc::Receiver<ServerToClient> {
        todo!()
    }

    /// Remove a TUI client from the active set.
    ///
    /// Drops the per-client `Sender`, closing the channel. Any subsequent send to this
    /// client will fail with `SendError::Closed`, which the broker catches during fan-out.
    #[allow(clippy::todo)]
    pub fn unregister_client(&mut self, _id: &str) {
        todo!()
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
    #[allow(clippy::todo)]
    pub fn fan_out(&mut self, _session_id: &str, _frame: Arc<Bytes>) {
        todo!()
    }

    /// Emit `ServerToClient::PtyReset { session_id }` to all registered clients.
    ///
    /// Called when the PTY writer task for this session is dropped (BC-2.05.009 Invariant 4).
    /// Each client channel receives an independent `try_send()` call; failure for one client
    /// does not prevent emission to others (fire-and-forget per-client).
    #[allow(clippy::todo)]
    pub fn emit_pty_reset(&mut self, _session_id: &str) {
        todo!()
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
    /// Emits `ServerToClient::PtyReset` to all clients before the task exits (on normal
    /// termination or when the INPUT channel is closed by the PTY reader dropping its sender).
    #[allow(clippy::todo)]
    pub fn spawn_event_loop(
        &mut self,
        _hook_rx: mpsc::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        todo!()
    }
}
