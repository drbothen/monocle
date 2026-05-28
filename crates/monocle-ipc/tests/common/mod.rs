//! Common test harness helpers for S-022 integration tests.
//!
//! Provides helpers for spawning a test daemon accept loop and connecting a test
//! TUI client. These helpers are now fully implemented.
#![allow(dead_code)]

use std::path::Path;
use std::sync::Arc;

use monocle_ipc::framing::read_framed;
use monocle_ipc::server::SubscriberList;
use monocle_ipc::types::ServerToClient;

/// Opaque daemon handle returned by `spawn_test_daemon`.
///
/// Holds the accept-loop join handle and provides test utilities for
/// pre-populating DaemonState before a client connects.
pub struct TestDaemonHandle {
    /// The shared DaemonState used by the accept loop.
    /// Tests can inject state (e.g. oversized ring_tail, pending decisions) via this handle.
    pub state: Arc<monocle_runtime::state::DaemonState>,
    /// JoinHandle for the accept loop task. Dropped when TestDaemonHandle is dropped.
    _accept_task: tokio::task::JoinHandle<()>,
}

/// Spawn a test daemon accept loop on a socket at `runtime_dir/monocle.sock`.
///
/// Returns `(subscriber_list, daemon_handle)` so tests can:
/// - Push messages via the subscriber list to simulate daemon broadcasts.
/// - Assert subscriber counts after client connect/disconnect events.
/// - Pre-populate `daemon_handle.state` before clients connect (e.g. register pending
///   prompts via `state.pending_decisions`).
///
/// The returned `DaemonState` is initialized with a live `pending_decisions` registry
/// so that `handle_permission_decision` routes decisions correctly in tests.
pub async fn spawn_test_daemon(runtime_dir: &Path) -> (SubscriberList, TestDaemonHandle) {
    let listener = tokio::net::UnixListener::bind(runtime_dir.join("monocle.sock"))
        .expect("spawn_test_daemon: bind UnixListener");

    let mut state = monocle_runtime::state::DaemonState::new();
    // Initialize the pending-decisions registry so PermissionDecision routing works in tests.
    state.pending_decisions = Some(std::sync::Arc::new(
        monocle_runtime::permissions::PendingDecisionRegistry::new(),
    ));
    let state = Arc::new(state);
    let subscribers: SubscriberList = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let accept_state = Arc::clone(&state);
    let accept_subscribers = Arc::clone(&subscribers);

    let accept_task = tokio::spawn(async move {
        monocle_runtime::ipc_server::run_accept_loop(listener, accept_state, accept_subscribers)
            .await;
    });

    let handle = TestDaemonHandle {
        state,
        _accept_task: accept_task,
    };

    (subscribers, handle)
}

/// Connect a TUI test client to the daemon socket at `runtime_dir/monocle.sock`.
///
/// Returns a raw `tokio::net::UnixStream` on which tests can call `write_framed`
/// and `read_framed` directly. This is the TUI-side connection: reads `ServerToClient`,
/// writes `ClientToServer`.
///
/// Uses `tokio::net::UnixStream::connect` directly — bypasses the server-perspective
/// `Transport` trait which has the inverse I/O direction.
pub async fn connect_test_client(runtime_dir: &Path) -> tokio::net::UnixStream {
    // Retry briefly — the accept loop may need a moment to bind.
    for attempt in 0..20 {
        match tokio::net::UnixStream::connect(runtime_dir.join("monocle.sock")).await {
            Ok(stream) => return stream,
            Err(_) if attempt < 19 => {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            Err(e) => panic!("connect_test_client: failed to connect after 20 attempts: {e}"),
        }
    }
    unreachable!()
}

/// Receive the next `ServerToClient` message from a connected TUI-side stream.
///
/// Uses `read_framed` directly on the `UnixStream` (TUI receives `ServerToClient`).
///
/// Panics if the message cannot be received (connection closed, framing error).
pub async fn recv_one(stream: &mut tokio::net::UnixStream) -> ServerToClient {
    read_framed(stream)
        .await
        .expect("recv_one: must receive a ServerToClient message from daemon")
}
