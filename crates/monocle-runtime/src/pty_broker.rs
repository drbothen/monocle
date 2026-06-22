//! PtyOutput fan-out broker — bounded INPUT channel and biased event loop.
//!
//! Implements BC-2.05.009 §PtyBroker integration (Q1 ruling): the broker owns a bounded
//! INPUT channel (`Arc<Bytes>`, capacity 1024) and an `Arc<SubscriberList>` reference.
//! It does NOT own any per-client registry. Fan-out goes through
//! `crate::ipc_server::broadcast_to_subscribers(&self.subscriber_list, msg)` — the same
//! `SubscriberList` used by all daemon-to-TUI fan-out paths (hook events, session state
//! changes, permission prompts).
//!
//! The biased `select!` guarantees hook/control events are processed before PTY frames
//! when both arms are ready simultaneously (BC-2.05.009 Invariant 6 / ADR-0010).
//!
//! # PtyReset triggers (BC-2.05.009 Invariant 4)
//!
//! `ServerToClient::PtyReset` is broadcast on exactly two conditions:
//! 1. The session-host sends `HostToDaemon::PtyReset` — handled in the proxy task.
//! 2. The proxy task's `tx.send(frame).await` returns `Err(_)` — handled in the proxy task.
//!
//! The broker event loop MUST NOT emit `PtyReset` when `input_rx.recv()` returns `None`.
//! An `input_rx` close is the normal graceful session-exit path; emitting `PtyReset` there
//! would spuriously corrupt TUI state.

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use bytes::Bytes;
use tokio::sync::mpsc;

use monocle_ipc::server::SubscriberList;
use monocle_ipc::types::ServerToClient;

// ---------------------------------------------------------------------------
// Capacity constants (BC-2.05.009 PC-2, ADR-0010 §channel capacity 1024)
// ---------------------------------------------------------------------------

/// Capacity of the broker's INPUT channel (proxy task → broker).
///
/// Raw PTY frame bytes arrive here from the PTY reader `spawn_blocking` thread.
/// The `.send().await` call on the proxy-task side blocks when this channel is full,
/// propagating backpressure all the way to the PTY read syscall (BC-2.05.009 Invariant 3).
pub const PTY_BROKER_INPUT_CAPACITY: usize = 1024;

/// Capacity of each per-client outbound channel (mirrors `monocle_ipc::server::CLIENT_CHANNEL_CAPACITY`).
///
/// Exposed here so tests can reference it without importing `monocle_ipc::server` directly.
/// Canonical value is 64 per SS-ipc.md §TUI IPC Read Loop Pattern
/// (`64 × 256 KiB = 16 MiB` maximum in-flight per client).
pub const PTY_BROKER_CLIENT_CAPACITY: usize = monocle_ipc::server::CLIENT_CHANNEL_CAPACITY;

/// Strike limit constant — **test anchor only; encodes no production behavior.**
///
/// The production 1-strike disconnect is implemented directly inside
/// `broadcast_to_subscribers` (in `monocle_ipc`). This constant is exported purely so
/// tests can assert the value has not drifted (BC-2.05.009 Invariant 3b Q1 ruling).
/// No production code path reads or branches on this constant.
pub const PTY_BROKER_STRIKE_LIMIT: u8 = 1;

// ---------------------------------------------------------------------------
// PtyBroker
// ---------------------------------------------------------------------------

/// Per-session PTY output fan-out broker.
///
/// Receives raw PTY frame bytes via its bounded INPUT channel (`Arc<Bytes>`, capacity 1024)
/// and fans them out to all registered TUI clients by calling
/// `broadcast_to_subscribers(&self.subscriber_list, ServerToClient::PtyOutput { .. })`.
///
/// The broker does NOT own a per-client registry. All per-client channels are managed by
/// the `SubscriberList` (populated by `register_subscriber` on IPC connect; drained by
/// `remove_subscriber` on disconnect).
pub struct PtyBroker {
    /// Session identifier (UUID string) for this broker instance.
    pub session_id: String,

    /// INPUT channel sender — the proxy task calls `.send().await` here.
    ///
    /// Capacity 1024 (BC-2.05.009 PC-2). `Arc<Bytes>` is the item type at this layer only.
    /// The broker event loop wraps frames into `ServerToClient::PtyOutput` before fan-out.
    pub input_tx: mpsc::Sender<Arc<Bytes>>,

    /// INPUT channel receiver — drained by the broker event-loop task.
    ///
    /// Moved into the event-loop task on `spawn_event_loop`. `None` after that call.
    pub input_rx: Option<mpsc::Receiver<Arc<Bytes>>>,

    /// Daemon-global PTY drop counter (BC-2.05.009 PC-3).
    ///
    /// Incremented ONLY in the proxy task when `tx.send(frame).await` returns `Err(_)`
    /// (INPUT channel receiver closed while session is still live — OOM-level condition).
    /// NOT incremented on normal backpressure or when the INPUT channel closes gracefully.
    pub pty_drop_counter: Arc<AtomicU64>,

    /// Shared daemon-wide subscriber list.
    ///
    /// The canonical per-client registry, populated by `register_subscriber` on IPC connect
    /// and drained by `remove_subscriber` on disconnect. The broker event loop calls
    /// `broadcast_to_subscribers` with this list — the same list used for all other
    /// daemon-to-TUI fan-out paths.
    pub subscriber_list: Arc<SubscriberList>,
}

impl PtyBroker {
    /// Construct a new `PtyBroker` for the given session.
    ///
    /// The INPUT channel is created here with `PTY_BROKER_INPUT_CAPACITY` (1024).
    /// `subscriber_list` is the daemon's shared `Arc<SubscriberList>` — it MUST NOT be
    /// a cloned snapshot; it must be the live shared instance so that clients connecting
    /// after this broker is created are visible to the event loop.
    ///
    /// Call `spawn_event_loop()` to start the background fan-out task.
    pub fn new(
        session_id: String,
        pty_drop_counter: Arc<AtomicU64>,
        subscriber_list: Arc<SubscriberList>,
    ) -> Self {
        let (input_tx, input_rx) = mpsc::channel::<Arc<Bytes>>(PTY_BROKER_INPUT_CAPACITY);
        Self {
            session_id,
            input_tx,
            input_rx: Some(input_rx),
            pty_drop_counter,
            subscriber_list,
        }
    }

    /// Spawn the broker event-loop as a background `tokio::task`.
    ///
    /// The loop uses `biased; select!` with two arms:
    /// 1. Hook/control event arm (higher priority — biased first).
    /// 2. PTY frame arm (lower priority).
    ///
    /// Returns a `JoinHandle<()>` — the caller stores this to abort the task on session
    /// teardown. The INPUT receiver is moved into the spawned task; `self.input_rx` is
    /// set to `None` after this call.
    ///
    /// When `input_rx.recv()` returns `None` (graceful session exit — proxy task exited
    /// and dropped its sender), the event loop simply returns WITHOUT emitting `PtyReset`
    /// (BC-2.05.009 Invariant 4). `PtyReset` is emitted only by the proxy task on
    /// `tx.send().await` error or on `HostToDaemon::PtyReset` receipt.
    pub fn spawn_event_loop(
        &mut self,
        mut hook_rx: mpsc::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        let Some(mut input_rx) = self.input_rx.take() else {
            tracing::error!(
                session_id = %self.session_id,
                "PtyBroker::spawn_event_loop called after INPUT receiver already moved; \
                 returning no-op task — this is a programming error"
            );
            return tokio::spawn(async {});
        };

        let session_id = self.session_id.clone();
        let subscriber_list = Arc::clone(&self.subscriber_list);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;

                    // Arm 1 (higher priority): hook/control event channel.
                    // Biased ensures this arm is checked before the PTY frame arm when
                    // both are ready simultaneously (BC-2.05.009 Invariant 6 / ADR-0010).
                    hook_event = hook_rx.recv() => {
                        match hook_event {
                            Some(()) => {
                                tracing::debug!(
                                    session_id = %session_id,
                                    "broker: hook/control event processed"
                                );
                            }
                            None => {
                                // Hook channel closed — no more hook events.
                                // Continue processing PTY frames until the INPUT channel closes.
                                tracing::debug!(
                                    session_id = %session_id,
                                    "broker: hook/control channel closed; switching to PTY-only loop"
                                );
                                break;
                            }
                        }
                    }

                    // Arm 2 (lower priority): PTY frame channel.
                    pty_frame = input_rx.recv() => {
                        match pty_frame {
                            Some(frame) => {
                                let msg = ServerToClient::PtyOutput {
                                    session_id: session_id.clone(),
                                    bytes: frame.to_vec(),
                                };
                                crate::ipc_server::broadcast_to_subscribers(
                                    &subscriber_list,
                                    msg,
                                )
                                .await;
                            }
                            None => {
                                // INPUT channel closed — normal graceful session exit.
                                // The proxy task exited and dropped its sender. Do NOT emit
                                // PtyReset: input_rx close is NOT a byte drop (BC-2.05.009
                                // Invariant 4).
                                tracing::debug!(
                                    session_id = %session_id,
                                    "broker: INPUT channel closed (graceful session exit); \
                                     exiting event loop without emitting PtyReset"
                                );
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
                        let msg = ServerToClient::PtyOutput {
                            session_id: session_id.clone(),
                            bytes: frame.to_vec(),
                        };
                        crate::ipc_server::broadcast_to_subscribers(&subscriber_list, msg).await;
                    }
                    None => {
                        // INPUT channel closed — normal graceful session exit.
                        // Do NOT emit PtyReset (BC-2.05.009 Invariant 4).
                        tracing::debug!(
                            session_id = %session_id,
                            "broker: INPUT channel closed (PTY-only loop, graceful session exit); \
                             exiting without emitting PtyReset"
                        );
                        return;
                    }
                }
            }
        })
    }
}
