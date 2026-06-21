//! TDD test suite for BC-2.09.006 daemon leg: ResizePane IPC dispatch, zero-dimension
//! clamp, resize_session DaemonToHost::Resize forwarding, and EC-238 session-host-dead
//! WARN-drop policy.
//!
//! BC clause → test mapping:
//!
//!   Postcondition 4 (AC-013)  → test_BC_2_09_006_daemon_resizepane_routes_to_resize_session
//!   EC-239 daemon clamp (AC-014) → test_BC_2_09_006_daemon_zero_dim_clamp_rows_forwarded
//!                                   test_BC_2_09_006_daemon_zero_dim_clamp_cols_forwarded
//!   Postcondition 5 (AC-015)  → test_BC_2_09_006_resize_session_forwards_daemon_to_host_resize
//!   EC-238 (AC-016)           → test_BC_2_09_006_ec238_session_host_dead_warn_drop
//!   AC-013 SessionNotFound    → test_BC_2_09_006_daemon_session_not_found_returns_err
//!
//! All tests MUST FAIL before implementation: `resize_session()` is `todo!()` and panics
//! on every call path. Each test will panic with the todo!() message.

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use monocle_ipc::types::DaemonToHost;
use monocle_runtime::session_manager::{
    HookEndpointConfig, SessionError, SessionHostSpawner, SessionManager, SpawnedHostHandle,
};
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

// ---------------------------------------------------------------------------
// AC-013 / BC-2.09.006 PC-4 — daemon routes ClientToServer::ResizePane to resize_session()
//
// RED GATE: resize_session() is todo!() — panics on every call.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_daemon_resizepane_routes_to_resize_session
///
/// AC-013 / BC-2.09.006 PC-4:
///   `resize_session(session_id, rows, cols)` is called by `handle_resize_pane()` when
///   `ClientToServer::ResizePane` arrives at the daemon. For a Running session with an
///   established host_conn, `resize_session()` must return `Ok(())`.
///
///   RED GATE: `resize_session()` is `todo!()` — panics.
#[tokio::test]
async fn test_BC_2_09_006_daemon_resizepane_routes_to_resize_session() {
    let tmp = isolated_runtime_dir();

    let (mut manager, session_id, _host_stream) =
        make_manager_with_running_session(tmp.path()).await;

    // RED GATE: todo!() panics here before any forwarding occurs.
    let result = manager.resize_session(&session_id, 30, 100).await;

    assert!(
        result.is_ok(),
        "BC-2.09.006 AC-013 PC-4: resize_session() must return Ok(()) for a Running session \
         with an established host_conn — got Err: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// AC-014 / EC-239 — zero-dimension clamp (defense-in-depth at ipc_server layer)
//
// The ipc_server clamps rows.max(1) and cols.max(1) before calling resize_session.
// These tests exercise resize_session() with the post-clamped values (rows=1, cols=1)
// and verify the correct Resize frame is forwarded.
//
// RED GATE: resize_session() is todo!() — panics.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_daemon_zero_dim_clamp_rows_forwarded
///
/// AC-014 / BC-2.09.006 EC-239:
///   The IPC handler clamps `rows=0` to `rows=1` before calling `resize_session()`.
///   This test calls `resize_session(session_id, 1, 80)` (the post-clamp value) and
///   asserts `DaemonToHost::Resize { rows: 1, cols: 80 }` is forwarded to the session-host.
///
///   RED GATE: `resize_session()` is `todo!()` — panics.
#[tokio::test]
async fn test_BC_2_09_006_daemon_zero_dim_clamp_rows_forwarded() {
    let tmp = isolated_runtime_dir();

    let (mut manager, session_id, mut host_stream) =
        make_manager_with_running_session(tmp.path()).await;

    // RED GATE: todo!() panics here.
    manager.resize_session(&session_id, 1, 80).await.expect(
        "BC-2.09.006 AC-014 EC-239: resize_session(1, 80) must return Ok(()) — \
             rows=1 is the clamped value from rows=0",
    );

    let msg = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        read_daemon_to_host(&mut host_stream),
    )
    .await
    .expect(
        "BC-2.09.006 AC-014 EC-239: DaemonToHost::Resize must arrive within 200ms \
         after resize_session(1, 80)",
    );

    match msg {
        DaemonToHost::Resize { rows, cols } => {
            assert_eq!(
                rows, 1,
                "BC-2.09.006 AC-014 EC-239: forwarded rows must be 1 (clamped from 0)"
            );
            assert_eq!(
                cols, 80,
                "BC-2.09.006 AC-014 EC-239: forwarded cols must be 80 (unchanged)"
            );
        }
        other => panic!(
            "BC-2.09.006 AC-014 EC-239: expected DaemonToHost::Resize, got {:?}",
            other
        ),
    }
}

/// test_BC_2_09_006_daemon_zero_dim_clamp_cols_forwarded
///
/// AC-014 / BC-2.09.006 EC-239:
///   The IPC handler clamps `cols=0` to `cols=1` before calling `resize_session()`.
///   This test calls `resize_session(session_id, 30, 1)` (the post-clamp value) and
///   asserts `DaemonToHost::Resize { rows: 30, cols: 1 }` is forwarded.
///
///   RED GATE: `resize_session()` is `todo!()` — panics.
#[tokio::test]
async fn test_BC_2_09_006_daemon_zero_dim_clamp_cols_forwarded() {
    let tmp = isolated_runtime_dir();

    let (mut manager, session_id, mut host_stream) =
        make_manager_with_running_session(tmp.path()).await;

    // RED GATE: todo!() panics here.
    manager.resize_session(&session_id, 30, 1).await.expect(
        "BC-2.09.006 AC-014 EC-239: resize_session(30, 1) must return Ok(()) — \
             cols=1 is the clamped value from cols=0",
    );

    let msg = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        read_daemon_to_host(&mut host_stream),
    )
    .await
    .expect(
        "BC-2.09.006 AC-014 EC-239: DaemonToHost::Resize must arrive within 200ms \
         after resize_session(30, 1)",
    );

    match msg {
        DaemonToHost::Resize { rows, cols } => {
            assert_eq!(
                rows, 30,
                "BC-2.09.006 AC-014 EC-239: forwarded rows must be 30 (unchanged)"
            );
            assert_eq!(
                cols, 1,
                "BC-2.09.006 AC-014 EC-239: forwarded cols must be 1 (clamped from 0)"
            );
        }
        other => panic!(
            "BC-2.09.006 AC-014 EC-239: expected DaemonToHost::Resize, got {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// AC-015 / BC-2.09.006 PC-5 — resize_session() serializes and forwards DaemonToHost::Resize
//
// Canonical test vector: 24×80 → 30×100.
//
// RED GATE: resize_session() is todo!() — panics; no frame is ever sent.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_resize_session_forwards_daemon_to_host_resize
///
/// AC-015 / BC-2.09.006 PC-5 (canonical test vector row 1):
///   `resize_session(session_id, 30, 100)` serializes `DaemonToHost::Resize { rows: 30,
///   cols: 100 }` and writes it to the session-host via `host_conn.writer`.
///
///   Input: pane resizes from 24×80 to 30×100.
///   Expected: `DaemonToHost::Resize { rows: 30, cols: 100 }` received on session-host stream.
///
///   RED GATE: `resize_session()` is `todo!()` — panics; no frame arrives on `host_stream`.
#[tokio::test]
async fn test_BC_2_09_006_resize_session_forwards_daemon_to_host_resize() {
    let tmp = isolated_runtime_dir();

    let (mut manager, session_id, mut host_stream) =
        make_manager_with_running_session(tmp.path()).await;

    // RED GATE: todo!() panics here; after implementation the Resize frame arrives.
    manager.resize_session(&session_id, 30, 100).await.expect(
        "BC-2.09.006 AC-015 PC-5: resize_session() must return Ok(()) for a Running \
             session with an established host_conn",
    );

    // Assert DaemonToHost::Resize { rows: 30, cols: 100 } received on host stream.
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
// AC-016 / EC-238 — session-host dead mid-resize → Err returned, no TUI error sent
//
// When the host connection is dead (remote side closed), resize_session() must return
// Err(SessionHostDead{..}) or Err(Io(..)).
//
// RED GATE (two layers):
//   1. resize_session() is todo!() — panics before any IO path is reached.
//   2. After todo!() is removed, the Err variant check will catch any incorrect
//      error propagation (e.g., returning Ok() or a wrong error variant).
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_ec238_session_host_dead_warn_drop
///
/// AC-016 / BC-2.09.006 EC-238:
///   When the session-host control connection is dead (remote side closed), `resize_session()`
///   must return `Err(SessionHostDead { .. })` or `Err(Io(..))` — never `Ok(())`.
///   The IPC handler (`handle_resize_pane`) WARN-drops this error — no `ServerToClient::Error`
///   is sent to the TUI client.
///
///   RED GATE: `resize_session()` is `todo!()` — panics.
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

    // RED GATE: todo!() panics here; after implementation returns Err(SessionHostDead) or Io.
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
// AC-013 / SessionNotFound — WARN-drop for unknown session_id
//
// RED GATE: resize_session() is todo!() — panics before reaching the session lookup.
// ---------------------------------------------------------------------------

/// test_BC_2_09_006_daemon_session_not_found_returns_err
///
/// AC-013 / BC-2.09.006 PC-4:
///   When `resize_session()` is called for a session_id not in the registry,
///   it must return `Err(SessionError::SessionNotFound { .. })`.
///   The IPC handler (`handle_resize_pane`) WARN-drops this per the ResizePane carve-out —
///   no `ServerToClient::Error` is sent to the TUI.
///
///   RED GATE: `resize_session()` is `todo!()` — panics before the session lookup.
#[tokio::test]
async fn test_BC_2_09_006_daemon_session_not_found_returns_err() {
    let tmp = isolated_runtime_dir();

    let (mut manager, _running_id, _host_stream) =
        make_manager_with_running_session(tmp.path()).await;

    let nonexistent_id = "ffffffff-ffff-4fff-bfff-ffffffffffff";

    // RED GATE: todo!() panics here; after implementation returns SessionNotFound.
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
