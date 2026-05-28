//! Fan-out subscriber registry and primitive helpers for daemon-side IPC (S-022).
//!
//! This module exposes the minimal surface needed by both the `monocle-ipc` integration
//! tests and the `monocle-runtime::ipc_server` accept loop, without creating a circular
//! crate dependency.
//!
//! # Circular dependency note
//!
//! `monocle-runtime` depends on `monocle-ipc` (for `ServerToClient`, `ClientToServer`,
//! framing, etc.). If `monocle-ipc` also depended on `monocle-runtime` in production
//! code, a circular crate dependency would result. The accept loop and per-client task
//! spawner — which need `monocle_runtime::state::DaemonState` — therefore live in
//! `monocle-runtime::ipc_server`, not here.
//!
//! # Architecture decision for Issue 2 (UdsClientTransport bidirectionality)
//!
//! Option (c) from the dispatch: expose raw `tokio::net::UnixStream` access via the
//! `monocle-runtime` accept loop. The per-client receive path reads `ClientToServer`
//! from the stream's read half; the per-client send path writes `ServerToClient` to
//! the stream's write half. This minimises surface area — no new TUI-side transport
//! type is needed since tests use `tokio::net::UnixStream` directly.
//!
//! # S-022 implementation status
//!
//! This module is fully implemented. All `todo!()` stubs have been replaced.

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::types::ServerToClient;

/// Bounded channel capacity for per-client outbound message queues.
///
/// Matches the capacity used in [`crate::uds::UdsTransport`] fan-out channels.
/// Each connected client gets a dedicated `mpsc::channel(CLIENT_CHANNEL_CAPACITY)`.
pub const CLIENT_CHANNEL_CAPACITY: usize = 64;

/// Per-client entry in the fan-out subscriber list.
///
/// Each connected TUI client is represented by one `ClientEntry`. The `tx` field is
/// the outbound send half of the per-client bounded channel. The `disconnect` field
/// is a `Notify` that is triggered by `broadcast_to_subscribers` when the client's
/// channel is found full (slow-client disconnect per BC-2.05.004 EC-005). The
/// per-client task holds a clone of `disconnect` and selects on `notified()` to
/// detect the slow-disconnect signal and break out of its event loop, causing the
/// underlying UDS socket to close.
///
/// # BC-2.05.004 EC-005 contract
///
/// When `broadcast_to_subscribers` detects `TrySendError::Full`:
///
/// 1. Removes the entry from the subscriber list (no further broadcasts reach the client).
/// 2. Calls `disconnect.notify_one()` to signal the per-client task.
/// 3. Logs WARN.
///
/// The per-client task's `tokio::select!` exits on the notify, dropping the write half
/// of the UDS socket. This closes the connection — the slow client observes EOF on
/// its read side.
pub struct ClientEntry {
    /// Outbound channel sender for this client.
    pub tx: mpsc::Sender<ServerToClient>,
    /// Slow-disconnect signal. Triggered on `TrySendError::Full` during broadcast.
    pub disconnect: Arc<tokio::sync::Notify>,
}

impl std::fmt::Debug for ClientEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientEntry")
            .field("tx", &self.tx)
            .field("disconnect", &"<Notify>")
            .finish()
    }
}

impl ClientEntry {
    /// Create a new `ClientEntry` pairing `tx` with a fresh `Notify`.
    pub fn new(tx: mpsc::Sender<ServerToClient>) -> Self {
        Self {
            tx,
            disconnect: Arc::new(tokio::sync::Notify::new()),
        }
    }
}

/// Handle for the fan-out subscriber list shared across the accept loop and all per-client tasks.
///
/// The outer `Arc<tokio::sync::Mutex<...>>` matches the pattern established in
/// [`crate::uds::UdsTransport::subscribers`] so both types can share the same list.
pub type SubscriberList = Arc<tokio::sync::Mutex<Vec<ClientEntry>>>;

/// Add a client sender to the shared fan-out subscriber list.
///
/// Returns the `Arc<Notify>` disconnect signal so the caller (per-client task in
/// `monocle-runtime::ipc_server`) can select on it to detect slow-disconnect.
///
/// # Contract (AC-006 — no gap window)
///
/// The sender MUST be registered in `subscribers` BEFORE `InitialState` is sent.
/// Registering before send ensures that any incremental events published between
/// snapshot time and first message delivery are queued in the client's channel,
/// not dropped.
pub async fn register_subscriber(
    subscribers: &SubscriberList,
    sender: mpsc::Sender<ServerToClient>,
) -> Arc<tokio::sync::Notify> {
    let entry = ClientEntry::new(sender);
    let disconnect = Arc::clone(&entry.disconnect);
    let mut subs = subscribers.lock().await;
    subs.push(entry);
    disconnect
}

/// Remove a client entry from the shared fan-out subscriber list.
///
/// Called when the per-client task detects EOF or a send error on the client channel.
/// Logs at DEBUG level: `DEBUG: TUI client disconnected; removed from subscriber list`.
///
/// # Implementation note
///
/// Removes all entries whose `tx` channel is the same as `sender` using
/// `Sender::same_channel`. This is O(n) in the number of subscribers, which is
/// acceptable for Phase 1 (low single-digit subscriber counts expected).
pub async fn remove_subscriber(
    subscribers: &SubscriberList,
    sender: &mpsc::Sender<ServerToClient>,
) {
    let mut subs = subscribers.lock().await;
    subs.retain(|entry| !entry.tx.same_channel(sender));
    tracing::debug!("TUI client disconnected; removed from subscriber list");
}
