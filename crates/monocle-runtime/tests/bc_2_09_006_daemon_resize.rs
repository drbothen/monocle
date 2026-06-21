//! TDD test suite for BC-2.09.006 daemon leg: ResizePane IPC dispatch, zero-dimension
//! clamp, resize_session DaemonToHost::Resize forwarding, and EC-238 session-host-dead
//! WARN-drop policy.
//!
//! # Test architecture
//!
//! Tests are split into two layers:
//!
//! ## Layer 1: session_manager::resize_session() (EXISTING, CORRECT)
//!
//! These tests call `resize_session()` directly and verify the forwarding contract.
//! They represent the plumbing layer: does the session manager send the right bytes
//! to the right host?
//!
//!   AC-013 / PC-4 → test_BC_2_09_006_daemon_resizepane_routes_to_resize_session
//!   AC-015 / PC-5 → test_BC_2_09_006_resize_session_forwards_daemon_to_host_resize
//!   EC-238 / AC-016 → test_BC_2_09_006_ec238_session_host_dead_warn_drop (error path)
//!   AC-013 NotFound → test_BC_2_09_006_daemon_session_not_found_returns_err
//!
//! ## Layer 2: ipc_server::handle_resize_pane() (NEW — handler boundary)
//!
//! These tests call `handle_resize_pane_pub` — the PUBLIC TEST SEAM around the private
//! `handle_resize_pane()` IPC handler. They verify:
//!   - Zero-dimension clamping happens BEFORE calling resize_session (AC-014 / EC-239).
//!     The clamp is at the handler boundary: rows.max(1), cols.max(1). The pre-clamped
//!     zero values (rows=0, cols=0) must NEVER reach resize_session.
//!   - All error paths from resize_session are WARN-dropped — no ServerToClient::Error
//!     is sent to the TUI client for any resize failure (AC-013/AC-016 WARN-drop carve-out).
//!
//! HIGH-002 / AC-013 warn-drop → test_BC_2_09_006_handler_session_not_found_warn_drop
//! HIGH-003 / AC-014 zero-rows → test_BC_2_09_006_handler_zero_dim_rows_clamp_no_error
//! HIGH-003 / AC-014 zero-cols → test_BC_2_09_006_handler_zero_dim_cols_clamp_no_error
//! HIGH-004 / AC-016 host-dead → test_BC_2_09_006_handler_session_host_dead_warn_drop
//! AC-013 mgr-none → test_BC_2_09_006_handler_session_manager_none_warn_drop
//!
//! # Red Gate (ALL tests in this file)
//!
//! Layer 1 tests: `resize_session()` is `todo!()` — panics on every call path.
//! Layer 2 tests: `resize_session()` is `todo!()` AND `handle_resize_pane` currently
//!   DOES send `ServerToClient::Error` on failure (wrong behavior). After `todo!()` is
//!   replaced with real implementation, the WARN-drop assertions in Layer 2 tests will
//!   fail because the current `handle_resize_pane` sends Error for every failure path.
//!
//! # Existing zero-dim clamp tests (RETIRED — replaced by handler-level tests)
//!
//! The previous `test_BC_2_09_006_daemon_zero_dim_clamp_rows_forwarded` and
//! `_cols_forwarded` were TAUTOLOGICAL: they called `resize_session(id, 1, 80)` and
//! `resize_session(id, 30, 1)` — pre-clamped values that bypass the clamp logic. The
//! clamping happens at the handler boundary (`handle_resize_pane`), not in `resize_session`.
//! Replaced by `test_BC_2_09_006_handler_zero_dim_rows_clamp_no_error` and
//! `_cols_clamp_no_error` which call `handle_resize_pane_pub` with rows=0 / cols=0.

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use monocle_ipc::types::{DaemonToHost, ServerToClient};
use monocle_runtime::ipc_server::handle_resize_pane_pub;
use monocle_runtime::session_manager::{
    HookEndpointConfig, SessionError, SessionHostSpawner, SessionManager, SpawnedHostHandle,
};
use monocle_runtime::state::DaemonState;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Shared test infrastructure
// ---------------------------------------------------------------------------

/// Build a short-path temp dir under /tmp to keep UDS socket paths under macOS's
/// SUN_LEN limit (104 chars).
fn isolated_runtime_dir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .tempdir_in("/tmp")
        .expect("create isolated runtime dir under /tmp for daemon resize tests")
}

/// A spawner that always panics — we never invoke spawning in resize tests.
struct NeverSpawner;

#[async_trait::async_trait]
impl SessionHostSpawner for NeverSpawner {
    async fn spawn(
        &self,
        _session_id: &str,
        _recipe: &monocle_core::engine::SpawnRecipe,
        _runtime_dir: &std::path::Path,
    ) -> Result<SpawnedHostHandle, SessionError> {
        panic!("NeverSpawner: spawn() must never be called in resize_session tests")
    }
}

/// Minimal `EngineModule` that panics on every call — only used in manager construction
/// where the actual engine is never invoked by resize_session.
struct ResizeTestEngine;

#[async_trait::async_trait]
impl monocle_core::engine::EngineModule for ResizeTestEngine {
    fn id(&self) -> &'static str {
        "resize-test-engine"
    }
    fn metadata(
        &self,
    ) -> Result<monocle_core::engine::EngineMetadata, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for resize_session tests")
    }
    fn detect(&self, _: &monocle_core::engine::ProcessSnapshot) -> bool {
        false
    }
    async fn enrich(
        &self,
        _: &monocle_core::engine::ProcessSnapshot,
    ) -> Result<monocle_core::engine::EnrichedSession, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for resize_session tests")
    }
    async fn on_hook(
        &self,
        _: monocle_core::hook_events::HookEvent,
    ) -> monocle_core::engine::HookResponse {
        unimplemented!("not needed for resize_session tests")
    }
}

/// Build a minimal `SessionManager` and inject a Running session backed by a UDS stream pair.
///
/// Returns `(manager, session_id, host_stream)` where:
/// - `manager` has a Running session for `session_id`
/// - `host_stream` is the host side of the UDS pair; reading it yields `DaemonToHost` frames
async fn make_manager_with_running_session(
    tmp: &std::path::Path,
) -> (SessionManager, String, tokio::net::UnixStream) {
    use monocle_ipc::server::{ClientEntry, CLIENT_CHANNEL_CAPACITY};

    let (tx, _rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let subscriber_list: monocle_ipc::server::SubscriberList =
        Arc::new(Mutex::new(vec![ClientEntry::new(tx)]));
    let broker = Arc::new(Arc::clone(&subscriber_list));
    let spawner: Arc<dyn SessionHostSpawner> = Arc::new(NeverSpawner);

    let manager = SessionManager::new(
        tmp.to_path_buf(),
        spawner,
        broker,
        Arc::new(ResizeTestEngine),
        HookEndpointConfig::default(),
    );

    // Create a UDS stream pair: daemon-side (write_half passed to manager) + host-side reader.
    let (daemon_stream, host_stream) = tokio::net::UnixStream::pair()
        .expect("test setup: create UDS stream pair for resize_session test");
    let (_, write_half) = daemon_stream.into_split();

    let session_id = "00000042-0000-4000-8042-000000000042".to_string();

    manager
        .insert_running_session_for_test(
            &session_id,
            42_042,
            tmp.join("session-test.sock"),
            write_half,
        )
        .await;

    (manager, session_id, host_stream)
}

/// Build a minimal `DaemonState` whose `session_manager` contains a Running session.
///
/// Returns `(state, session_id, host_stream)`.
async fn make_daemon_state_with_running_session(
    tmp: &std::path::Path,
) -> (DaemonState, String, tokio::net::UnixStream) {
    use monocle_ipc::server::{ClientEntry, CLIENT_CHANNEL_CAPACITY};

    let (tx, _rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let subscriber_list: monocle_ipc::server::SubscriberList =
        Arc::new(Mutex::new(vec![ClientEntry::new(tx)]));
    let broker = Arc::new(Arc::clone(&subscriber_list));
    let spawner: Arc<dyn SessionHostSpawner> = Arc::new(NeverSpawner);

    let manager = SessionManager::new(
        tmp.to_path_buf(),
        spawner,
        broker,
        Arc::new(ResizeTestEngine),
        HookEndpointConfig::default(),
    );

    let (daemon_stream, host_stream) = tokio::net::UnixStream::pair()
        .expect("test setup: create UDS stream pair for daemon-state handler tests");
    let (_, write_half) = daemon_stream.into_split();

    let session_id = "00000042-0000-4000-8042-000000000042".to_string();

    manager
        .insert_running_session_for_test(
            &session_id,
            42_042,
            tmp.join("session-test.sock"),
            write_half,
        )
        .await;

    let mut state = DaemonState::new();
    state.session_manager = Some(tokio::sync::Mutex::new(manager));

    (state, session_id, host_stream)
}

/// Read one `DaemonToHost` frame from a stream (4-byte LE length prefix + JSON body).
async fn read_daemon_to_host(stream: &mut tokio::net::UnixStream) -> DaemonToHost {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .expect("read DaemonToHost frame: read length failed");
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut body = vec![0u8; len];
    stream
        .read_exact(&mut body)
        .await
        .expect("read DaemonToHost frame: read body failed");
    serde_json::from_slice::<DaemonToHost>(&body)
        .expect("read DaemonToHost frame: deserialize failed")
}

/// Drain all pending ServerToClient messages from an mpsc receiver without blocking.
fn drain_server_msgs(rx: &mut tokio::sync::mpsc::Receiver<ServerToClient>) -> Vec<ServerToClient> {
    let mut out = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        out.push(msg);
    }
    out
}

// ===========================================================================
// LAYER 1: session_manager::resize_session() — forwarding contract tests
//
// These tests exercise resize_session() directly. They do NOT verify handler-
// level concerns (clamping, WARN-drop) — those are in Layer 2 below.
//
// Red Gate: resize_session() is todo!() — panics on every call path.
// ===========================================================================

// ---------------------------------------------------------------------------
// AC-013 / BC-2.09.006 PC-4 — resize_session() called by handle_resize_pane
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_daemon_resizepane_routes_to_resize_session
///
/// AC-013 / BC-2.09.006 PC-4:
///   `resize_session(session_id, rows, cols)` is the gateway from daemon to session-host.
///   For a Running session with an established host_conn, it must return `Ok(())`.
///
///   Red Gate: `resize_session()` is `todo!()` — panics.
#[tokio::test]
async fn test_BC_2_09_006_daemon_resizepane_routes_to_resize_session() {
    let tmp = isolated_runtime_dir();

    let (mut manager, session_id, _host_stream) =
        make_manager_with_running_session(tmp.path()).await;

    // Red Gate: todo!() panics here before any forwarding occurs.
    let result = manager.resize_session(&session_id, 30, 100).await;

    assert!(
        result.is_ok(),
        "BC-2.09.006 AC-013 PC-4: resize_session() must return Ok(()) for a Running session \
         with an established host_conn — got Err: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// AC-015 / BC-2.09.006 PC-5 — resize_session forwards DaemonToHost::Resize
// (canonical test vector 1)
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_resize_session_forwards_daemon_to_host_resize
///
/// AC-015 / BC-2.09.006 PC-5 (canonical test vector row 1):
///   `resize_session(session_id, 30, 100)` serializes `DaemonToHost::Resize { rows: 30,
///   cols: 100 }` and writes it to the session-host via `host_conn.writer`.
///
///   Input: pane resizes from 24x80 to 30x100.
///   Expected: `DaemonToHost::Resize { rows: 30, cols: 100 }` received on session-host stream.
///
///   Red Gate: `resize_session()` is `todo!()` — panics; no frame arrives on `host_stream`.
#[tokio::test]
async fn test_BC_2_09_006_resize_session_forwards_daemon_to_host_resize() {
    let tmp = isolated_runtime_dir();

    let (mut manager, session_id, mut host_stream) =
        make_manager_with_running_session(tmp.path()).await;

    // Red Gate: todo!() panics here; after implementation the Resize frame arrives.
    manager.resize_session(&session_id, 30, 100).await.expect(
        "BC-2.09.006 AC-015 PC-5: resize_session() must return Ok(()) for a Running \
             session with an established host_conn",
    );

    let msg = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        read_daemon_to_host(&mut host_stream),
    )
    .await
    .expect(
        "BC-2.09.006 AC-015 PC-5: DaemonToHost::Resize { rows: 30, cols: 100 } must arrive \
         within 200ms after resize_session() returns Ok(())",
    );

    match msg {
        DaemonToHost::Resize { rows, cols } => {
            assert_eq!(
                rows, 30,
                "BC-2.09.006 AC-015 PC-5 canonical vector: DaemonToHost::Resize.rows must be 30"
            );
            assert_eq!(
                cols, 100,
                "BC-2.09.006 AC-015 PC-5 canonical vector: DaemonToHost::Resize.cols must be 100"
            );
        }
        other => panic!(
            "BC-2.09.006 AC-015 PC-5: expected DaemonToHost::Resize, got {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// EC-238 / AC-016 — session-host dead: resize_session returns Err
//
// The IPC handler (handle_resize_pane) WARN-drops this error — see Layer 2 tests
// below for the handler-level WARN-drop assertion.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_ec238_session_host_dead_warn_drop
///
/// AC-016 / BC-2.09.006 EC-238:
///   When the session-host control connection is dead (remote side closed),
///   `resize_session()` must return `Err(SessionHostDead { .. })` or `Err(Io(..))`.
///   The IPC handler (`handle_resize_pane`) WARN-drops this — no `ServerToClient::Error`
///   is sent to the TUI client. The handler-level assertion is in Layer 2.
///
///   Red Gate: `resize_session()` is `todo!()` — panics.
#[tokio::test]
async fn test_BC_2_09_006_ec238_session_host_dead_warn_drop() {
    let tmp = isolated_runtime_dir();

    // Create a stream pair, then drop the host side to simulate a dead connection.
    let (daemon_stream, host_stream) =
        tokio::net::UnixStream::pair().expect("test setup EC-238: create UDS pair");
    let (_, write_half) = daemon_stream.into_split();
    drop(host_stream); // host side closed — writes will get BrokenPipe/EOF

    // Yield to allow the OS to propagate the close.
    tokio::task::yield_now().await;

    use monocle_ipc::server::{ClientEntry, CLIENT_CHANNEL_CAPACITY};
    let (tx, _rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let subscriber_list: monocle_ipc::server::SubscriberList =
        Arc::new(Mutex::new(vec![ClientEntry::new(tx)]));
    let broker = Arc::new(Arc::clone(&subscriber_list));
    let spawner: Arc<dyn SessionHostSpawner> = Arc::new(NeverSpawner);

    let mut manager = SessionManager::new(
        tmp.path().to_path_buf(),
        spawner,
        broker,
        Arc::new(ResizeTestEngine),
        HookEndpointConfig::default(),
    );

    let session_id = "deadbeef-dead-4ead-beef-deadbeefbeef".to_string();
    manager
        .insert_running_session_for_test(
            &session_id,
            99_999,
            tmp.path().join("session-dead.sock"),
            write_half,
        )
        .await;

    // Red Gate: todo!() panics here; after implementation returns Err(SessionHostDead) or Io.
    let result = manager.resize_session(&session_id, 30, 100).await;

    assert!(
        result.is_err(),
        "BC-2.09.006 AC-016 EC-238: resize_session() with a dead host connection must return \
         Err — got Ok unexpectedly"
    );

    match &result {
        Err(SessionError::SessionHostDead { .. }) | Err(SessionError::Io(_)) => {
            // Correct per AC-016: WARN-drop variants.
        }
        Err(other) => panic!(
            "BC-2.09.006 AC-016 EC-238: expected SessionHostDead or Io, got: {:?}",
            other
        ),
        Ok(()) => unreachable!("asserted is_err above"),
    }
}

// ---------------------------------------------------------------------------
// AC-013 / SessionNotFound — returns Err for unknown session_id
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_daemon_session_not_found_returns_err
///
/// AC-013 / BC-2.09.006 PC-4:
///   When `resize_session()` is called for a session_id not in the registry,
///   it must return `Err(SessionError::SessionNotFound { .. })`.
///   The IPC handler WARN-drops this — handler-level assertion in Layer 2.
///
///   Red Gate: `resize_session()` is `todo!()` — panics before the session lookup.
#[tokio::test]
async fn test_BC_2_09_006_daemon_session_not_found_returns_err() {
    let tmp = isolated_runtime_dir();

    let (mut manager, _running_id, _host_stream) =
        make_manager_with_running_session(tmp.path()).await;

    let nonexistent_id = "ffffffff-ffff-4fff-bfff-ffffffffffff";

    // Red Gate: todo!() panics here; after implementation returns SessionNotFound.
    let result = manager.resize_session(nonexistent_id, 30, 100).await;

    assert!(
        result.is_err(),
        "BC-2.09.006 AC-013: resize_session() for a non-existent session must return \
         Err(SessionNotFound) — got Ok"
    );

    match &result {
        Err(SessionError::SessionNotFound { .. }) => {
            // Correct: WARN-dropped by handle_resize_pane per AC-013.
        }
        Err(other) => panic!(
            "BC-2.09.006 AC-013: expected SessionNotFound, got: {:?}",
            other
        ),
        Ok(()) => unreachable!("asserted is_err above"),
    }
}

// ===========================================================================
// LAYER 2: ipc_server::handle_resize_pane() — handler boundary tests
//
// These tests call `handle_resize_pane_pub` to verify:
//   - Zero-dim clamping at the handler boundary (AC-014 / EC-239)
//   - WARN-drop for all error paths (AC-013/AC-016 ResizePane carve-out)
//
// Red Gate (two layers):
//   1. resize_session() is todo!() — panics before any handler logic completes.
//   2. After todo!() is removed: current handle_resize_pane sends ServerToClient::Error
//      for failure paths (HIGH-002/003/004). The WARN-drop assertions will fail.
//
// For zero-dim tests: the current handler does NOT clamp before calling resize_session.
// Even after todo!() is replaced, rows=0/cols=0 will reach resize_session unclamped.
// The handler must clamp rows.max(1)/cols.max(1) BEFORE the call.
// ===========================================================================

// ---------------------------------------------------------------------------
// HIGH-003 / AC-014 / EC-239 — zero-dimension clamping AT THE HANDLER BOUNDARY
//
// The clamp `rows.max(1)` / `cols.max(1)` happens in `handle_resize_pane`,
// NOT in `resize_session`. The test passes raw zero values (rows=0 or cols=0)
// and asserts the handler clamps them before forwarding DaemonToHost::Resize.
//
// Red Gate:
//   - resize_session() is todo!() — panics before any forwarding.
//   - After todo!() removed: handle_resize_pane does not yet clamp — rows=0/cols=0
//     are forwarded unclamped, causing the DaemonToHost::Resize assertion to fail
//     (rows would be 0, not 1).
//   - The WARN-drop assertion also catches any ServerToClient::Error sent by the
//     current handler, which sends Error on failure.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_handler_zero_dim_rows_clamp_no_error
///
/// HIGH-003 / AC-014 / BC-2.09.006 EC-239:
///   `handle_resize_pane_pub(session_id, rows=0, cols=80, client_tx, state)` must:
///     1. Clamp rows=0 to rows=1 BEFORE calling resize_session.
///     2. Forward `DaemonToHost::Resize { rows: 1, cols: 80 }` to the session-host.
///     3. NOT send any `ServerToClient::Error` to the client channel (WARN-drop policy).
///     4. Emit a `tracing::warn!` on the clamped-from-zero path (not assertable in unit test).
///
///   Red Gate (layer 1): resize_session() is todo!() — panics.
///   Red Gate (layer 2): current handler does not clamp; rows=0 propagates unclamped.
///   Red Gate (layer 3): current handler sends ServerToClient::Error on failure.
#[tokio::test]
async fn test_BC_2_09_006_handler_zero_dim_rows_clamp_no_error() {
    let tmp = isolated_runtime_dir();

    let (state, session_id, mut host_stream) =
        make_daemon_state_with_running_session(tmp.path()).await;

    // Wire the client channel to receive handler responses.
    let (client_tx, mut client_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(16);

    // Act: call handler with rows=0 (zero-dim pre-clamp value — the TUI guard may have failed).
    // Red Gate (layer 1): resize_session() todo!() panics.
    // Red Gate (layer 2): handler does not clamp rows=0 before the call.
    handle_resize_pane_pub(
        session_id.clone(),
        0, // rows=0 — must be clamped to 1 by the handler
        80,
        &client_tx,
        &state,
    )
    .await;

    // Assert 1: NO ServerToClient::Error was sent (AC-013/AC-014 WARN-drop carve-out).
    // Red Gate (layer 3): current handler sends Error on all failure paths.
    let client_msgs = drain_server_msgs(&mut client_rx);
    let error_msgs: Vec<_> = client_msgs
        .iter()
        .filter(|m| matches!(m, ServerToClient::Error { .. }))
        .collect();
    assert!(
        error_msgs.is_empty(),
        "HIGH-003 / BC-2.09.006 AC-014: handle_resize_pane must NOT send ServerToClient::Error \
         for zero-dim rows=0 (WARN-drop carve-out) — got {} Error messages: {:?}",
        error_msgs.len(),
        error_msgs
    );

    // Assert 2: DaemonToHost::Resize { rows: 1, cols: 80 } forwarded with clamped rows.
    // Red Gate: handler does not clamp; rows=0 propagates or todo!() panics.
    let msg = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        read_daemon_to_host(&mut host_stream),
    )
    .await
    .expect(
        "BC-2.09.006 AC-014: DaemonToHost::Resize must arrive within 200ms after \
         handle_resize_pane clamps rows=0 to rows=1",
    );

    match msg {
        DaemonToHost::Resize { rows, cols } => {
            assert_eq!(
                rows, 1,
                "BC-2.09.006 AC-014 EC-239: handler must clamp rows=0 to rows=1 — \
                 got rows={rows}. Clamp `rows.max(1)` is absent in handle_resize_pane."
            );
            assert_eq!(
                cols, 80,
                "BC-2.09.006 AC-014 EC-239: cols must be 80 (unchanged) — got cols={cols}"
            );
        }
        other => panic!(
            "BC-2.09.006 AC-014: expected DaemonToHost::Resize, got {:?}",
            other
        ),
    }
}

/// test_BC_2_09_006_handler_zero_dim_cols_clamp_no_error
///
/// HIGH-003 / AC-014 / BC-2.09.006 EC-239:
///   `handle_resize_pane_pub(session_id, rows=30, cols=0, client_tx, state)` must:
///     1. Clamp cols=0 to cols=1 BEFORE calling resize_session.
///     2. Forward `DaemonToHost::Resize { rows: 30, cols: 1 }` to the session-host.
///     3. NOT send any `ServerToClient::Error` (WARN-drop carve-out).
///
///   Red Gate (layer 1): resize_session() todo!() panics.
///   Red Gate (layer 2): current handler does not clamp; cols=0 propagates unclamped.
///   Red Gate (layer 3): current handler sends Error on failure.
#[tokio::test]
async fn test_BC_2_09_006_handler_zero_dim_cols_clamp_no_error() {
    let tmp = isolated_runtime_dir();

    let (state, session_id, mut host_stream) =
        make_daemon_state_with_running_session(tmp.path()).await;

    let (client_tx, mut client_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(16);

    // Act: call handler with cols=0 (zero-dim pre-clamp value).
    handle_resize_pane_pub(
        session_id.clone(),
        30,
        0, // cols=0 — must be clamped to 1 by the handler
        &client_tx,
        &state,
    )
    .await;

    // Assert 1: NO ServerToClient::Error (WARN-drop carve-out).
    let client_msgs = drain_server_msgs(&mut client_rx);
    let error_msgs: Vec<_> = client_msgs
        .iter()
        .filter(|m| matches!(m, ServerToClient::Error { .. }))
        .collect();
    assert!(
        error_msgs.is_empty(),
        "HIGH-003 / BC-2.09.006 AC-014: handle_resize_pane must NOT send ServerToClient::Error \
         for zero-dim cols=0 (WARN-drop carve-out) — got {} Error messages: {:?}",
        error_msgs.len(),
        error_msgs
    );

    // Assert 2: DaemonToHost::Resize { rows: 30, cols: 1 } forwarded with clamped cols.
    let msg = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        read_daemon_to_host(&mut host_stream),
    )
    .await
    .expect(
        "BC-2.09.006 AC-014: DaemonToHost::Resize must arrive within 200ms after \
         handle_resize_pane clamps cols=0 to cols=1",
    );

    match msg {
        DaemonToHost::Resize { rows, cols } => {
            assert_eq!(
                rows, 30,
                "BC-2.09.006 AC-014 EC-239: rows must be 30 (unchanged) — got rows={rows}"
            );
            assert_eq!(
                cols, 1,
                "BC-2.09.006 AC-014 EC-239: handler must clamp cols=0 to cols=1 — \
                 got cols={cols}. Clamp `cols.max(1)` is absent in handle_resize_pane."
            );
        }
        other => panic!(
            "BC-2.09.006 AC-014: expected DaemonToHost::Resize, got {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// HIGH-002 / AC-013 — SessionNotFound: WARN-drop, NO ServerToClient::Error
//
// When resize_session returns SessionNotFound, handle_resize_pane must WARN-drop
// the error and send NO ServerToClient::Error to the TUI client.
//
// Red Gate (layer 1): resize_session() todo!() — panics.
// Red Gate (layer 2): current handle_resize_pane sends ServerToClient::Error
//   on ALL error paths (lines 688-694 in ipc_server.rs). The assertion fails.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_handler_session_not_found_warn_drop
///
/// HIGH-002 / AC-013 / BC-2.09.006 PC-4 (ResizePane WARN-drop carve-out):
///   When `handle_resize_pane` is called for a session_id not in the registry,
///   `resize_session` returns `Err(SessionNotFound)`. The handler MUST:
///     1. Emit `tracing::warn!` (not assertable here).
///     2. Return without sending `ServerToClient::Error` to the client channel.
///
///   Proves BC-2.09.006 AC-013: "All transport errors from resize_session() are
///   WARN-dropped — no ServerToClient::Error response is sent to the TUI for
///   resize failures."
///
///   Red Gate (layer 1): resize_session() todo!() panics.
///   Red Gate (layer 2): current handler sends Error on failure — assertion fails.
#[tokio::test]
async fn test_BC_2_09_006_handler_session_not_found_warn_drop() {
    let _tmp = isolated_runtime_dir();
    let state = DaemonState::new(); // No sessions inserted.

    let (client_tx, mut client_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(16);

    let nonexistent_id = "ffffffff-ffff-4fff-bfff-ffffffffffff".to_string();

    // Act: call handler for a non-existent session.
    // Red Gate (layer 1): resize_session() todo!() panics.
    // Red Gate (layer 2): current handler sends ServerToClient::Error.
    handle_resize_pane_pub(nonexistent_id, 30, 100, &client_tx, &state).await;

    // Assert: NO ServerToClient::Error was sent to the TUI client.
    // Red Gate: current handle_resize_pane sends Error on all error paths.
    let msgs = drain_server_msgs(&mut client_rx);
    let error_msgs: Vec<_> = msgs
        .iter()
        .filter(|m| matches!(m, ServerToClient::Error { .. }))
        .collect();

    assert!(
        error_msgs.is_empty(),
        "HIGH-002 / BC-2.09.006 AC-013: handle_resize_pane MUST NOT send ServerToClient::Error \
         for SessionNotFound (WARN-drop carve-out per AC-013) — got {} Error message(s): {:?}. \
         Current handle_resize_pane sends Error for all resize failures; the WARN-drop carve-out \
         must be implemented.",
        error_msgs.len(),
        error_msgs
    );
}

// ---------------------------------------------------------------------------
// HIGH-004 / AC-016 — SessionHostDead: WARN-drop, NO ServerToClient::Error
//
// When resize_session returns SessionHostDead or Io (dead host connection),
// handle_resize_pane must WARN-drop and send NO ServerToClient::Error.
//
// Red Gate (layer 1): resize_session() todo!() — panics.
// Red Gate (layer 2): current handle_resize_pane sends ServerToClient::Error
//   on ALL error paths. The assertion fails after todo!() is removed.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_handler_session_host_dead_warn_drop
///
/// HIGH-004 / AC-016 / BC-2.09.006 EC-238 (ResizePane WARN-drop carve-out):
///   When the session-host connection is dead and `resize_session` returns
///   `Err(SessionHostDead)` or `Err(Io)`, the handler MUST:
///     1. Emit `tracing::warn!` (not assertable here).
///     2. Return without sending `ServerToClient::Error` to the client channel.
///
///   Proves BC-2.09.006 AC-016: "If resize_session() returns Err(SessionHostDead)
///   or any IO error, the IPC handler emits tracing::warn! and returns without
///   sending ServerToClient::Error to the TUI."
///
///   Red Gate (layer 1): resize_session() todo!() panics.
///   Red Gate (layer 2): current handler sends Error on failure — assertion fails.
#[tokio::test]
async fn test_BC_2_09_006_handler_session_host_dead_warn_drop() {
    let tmp = isolated_runtime_dir();

    // Create a dead host connection: close the host side immediately.
    let (daemon_stream, host_stream_dead) =
        tokio::net::UnixStream::pair().expect("test setup EC-238: create UDS pair");
    let (_, write_half) = daemon_stream.into_split();
    drop(host_stream_dead); // close host side — any write will get BrokenPipe/EOF

    tokio::task::yield_now().await; // let OS propagate the close

    use monocle_ipc::server::{ClientEntry, CLIENT_CHANNEL_CAPACITY};
    let (tx, _rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let subscriber_list: monocle_ipc::server::SubscriberList =
        Arc::new(Mutex::new(vec![ClientEntry::new(tx)]));
    let broker = Arc::new(Arc::clone(&subscriber_list));
    let spawner: Arc<dyn SessionHostSpawner> = Arc::new(NeverSpawner);

    let manager = SessionManager::new(
        tmp.path().to_path_buf(),
        spawner,
        broker,
        Arc::new(ResizeTestEngine),
        HookEndpointConfig::default(),
    );

    let session_id = "deadbeef-dead-4ead-beef-deadbeefbeef".to_string();
    manager
        .insert_running_session_for_test(
            &session_id,
            99_999,
            tmp.path().join("session-dead.sock"),
            write_half,
        )
        .await;

    let mut state = DaemonState::new();
    state.session_manager = Some(tokio::sync::Mutex::new(manager));

    let (client_tx, mut client_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(16);

    // Act: call handler with a dead host connection.
    // Red Gate (layer 1): resize_session() todo!() panics.
    // Red Gate (layer 2): current handler sends ServerToClient::Error on failure.
    handle_resize_pane_pub(session_id, 30, 100, &client_tx, &state).await;

    // Assert: NO ServerToClient::Error was sent.
    // Red Gate: current handle_resize_pane sends Error — assertion fails.
    let msgs = drain_server_msgs(&mut client_rx);
    let error_msgs: Vec<_> = msgs
        .iter()
        .filter(|m| matches!(m, ServerToClient::Error { .. }))
        .collect();

    assert!(
        error_msgs.is_empty(),
        "HIGH-004 / BC-2.09.006 AC-016: handle_resize_pane MUST NOT send ServerToClient::Error \
         when session-host is dead (WARN-drop carve-out per AC-016) — got {} Error message(s): \
         {:?}. Current handle_resize_pane sends Error for all resize failures.",
        error_msgs.len(),
        error_msgs
    );
}

// ---------------------------------------------------------------------------
// AC-013 — session_manager is None: WARN-drop, NO ServerToClient::Error
//
// When DaemonState::session_manager is None (daemon wiring bug), the current
// handle_resize_pane sends ServerToClient::Error at lines 655-660. This is
// wrong: even for the None case, resize failures must be WARN-dropped.
//
// Red Gate: current handler sends Error for the None case.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_handler_session_manager_none_warn_drop
///
/// AC-013 / BC-2.09.006 PC-4 (ResizePane WARN-drop carve-out — session_manager None):
///   When `DaemonState::session_manager` is `None` (a daemon initialization bug),
///   `handle_resize_pane` currently logs an error and sends `ServerToClient::Error`.
///   Per AC-013, the ResizePane WARN-drop carve-out applies: NO `ServerToClient::Error`
///   must be sent for any resize failure path, including this one. The handler must
///   emit `tracing::warn!` (or `tracing::error!`) and return without sending Error.
///
///   Red Gate: current handler sends ServerToClient::Error for the None case.
#[tokio::test]
async fn test_BC_2_09_006_handler_session_manager_none_warn_drop() {
    // Build a DaemonState with session_manager = None to hit the "daemon wiring bug" path.
    let mut state = DaemonState::new();
    state.session_manager = None;

    let (client_tx, mut client_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(16);

    let session_id = "aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa".to_string();

    // Act: call handler when session_manager is None.
    handle_resize_pane_pub(session_id, 30, 100, &client_tx, &state).await;

    // Assert: NO ServerToClient::Error sent (WARN-drop carve-out applies to ALL resize paths).
    let msgs = drain_server_msgs(&mut client_rx);
    let error_msgs: Vec<_> = msgs
        .iter()
        .filter(|m| matches!(m, ServerToClient::Error { .. }))
        .collect();

    assert!(
        error_msgs.is_empty(),
        "AC-013 / BC-2.09.006: handle_resize_pane MUST NOT send ServerToClient::Error when \
         session_manager is None (WARN-drop carve-out applies to all resize failure paths) — \
         got {} Error message(s): {:?}. The None path at lines 655-660 of ipc_server.rs \
         currently sends Error; it must be changed to WARN-drop.",
        error_msgs.len(),
        error_msgs
    );
}
