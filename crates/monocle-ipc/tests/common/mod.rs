//! Common test harness helpers for S-022 integration tests.
//!
//! Provides helpers for spawning a test daemon accept loop and connecting a test
//! TUI client. Bodies are `todo!()` — the harness itself does not need to work
//! for the Red Gate phase; the `todo!()` panic is the Red Gate evidence.
//! These helpers compile cleanly and serve as documented call-sites for the implementer.
#![allow(dead_code, unused_variables, clippy::todo)]

use std::path::Path;

use monocle_ipc::framing::read_framed;
use monocle_ipc::server::SubscriberList;
use monocle_ipc::types::ServerToClient;

// ---------------------------------------------------------------------------
// Stub signature issues found during Red Gate test writing
//
// Issue 1: `monocle_ipc::server::monocle_runtime_state_placeholder::DaemonState`
//   has a private field `_private: ()` with no public constructor. This prevents
//   tests from constructing `DaemonState` directly outside the crate. The implementer
//   MUST either:
//     a) Replace the placeholder with `monocle_runtime::state::DaemonState` (which has
//        `DaemonState::new()` / `Default`), OR
//     b) Add `pub fn new() -> Self` or `impl Default` to the placeholder.
//   Until then, all tests route DaemonState construction through this helper.
//
// Issue 2: `monocle_ipc::uds::connect()` returns `UdsClientTransport`, which is
//   the SERVER-SIDE per-client transport implementing the server-perspective
//   `Transport` trait (`send_message` writes `ServerToClient`, `recv_message` reads
//   `ClientToServer`). For TUI-side use, the TUI needs to READ `ServerToClient` and
//   WRITE `ClientToServer`. `UdsClientTransport` does not implement `AsyncRead` or
//   `AsyncWrite` directly, making `write_framed`/`read_framed` unusable on it.
//   The implementer MUST either:
//     a) Expose the inner `UnixStream` read/write halves via a separate TUI-side type, OR
//     b) Add a `TuiTransport` type (or methods) that wraps the TUI-side IPC direction.
//   For tests, we use `tokio::net::UnixStream` directly (before `connect()` is wired).
// ---------------------------------------------------------------------------

/// Opaque daemon handle returned by `spawn_test_daemon`.
///
/// After implementation, this will hold the accept-loop join handle and bound path.
pub struct TestDaemonHandle {
    _marker: (),
}

/// Spawn a test daemon accept loop on a socket at `runtime_dir/monocle.sock`.
///
/// Returns `(subscriber_list, daemon_handle)` so tests can:
/// - Push messages via the subscriber list to simulate daemon broadcasts.
/// - Assert subscriber counts after client connect/disconnect events.
///
/// # Red Gate
///
/// Calling this helper panics via `todo!()`, which is the Red Gate evidence that
/// the implementation has not yet landed.
pub async fn spawn_test_daemon(runtime_dir: &Path) -> (SubscriberList, TestDaemonHandle) {
    todo!("S-022 test harness: spawn test daemon — wire run_accept_loop on the bound UDS socket")
}

/// Connect a TUI test client to the daemon socket at `runtime_dir/monocle.sock`.
///
/// Returns a raw `tokio::net::UnixStream` on which tests can call `write_framed`
/// and `read_framed` directly (bypassing the server-perspective `Transport` trait,
/// which has the wrong I/O direction for TUI-side use).
///
/// # Red Gate
///
/// Calling this helper panics via `todo!()`, which is the Red Gate evidence.
pub async fn connect_test_client(runtime_dir: &Path) -> tokio::net::UnixStream {
    todo!("S-022 test harness: connect test client — wire tokio::net::UnixStream::connect")
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
