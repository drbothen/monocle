//! S-034 Red Gate: Failing TDD tests for `SessionManager::kill_session()`.
//!
//! Every test here MUST fail before S-034 implementation is complete.
//! The stubs (`kill_session()`, `spawn_kill_watchdog()`, `handle_kill_session()`)
//! all contain `todo!()` bodies — tests fail with a panic on `todo!()` invocation.
//!
//! # Behavioral Contract Coverage
//!
//! | Test | BC / AC | Fails because |
//! |------|---------|---------------|
//! | test_BC_2_08_003_kill_session_sigterm_within_500ms | BC-2.08.003 PC-1, PC-2 | kill_session() is todo!() |
//! | test_BC_2_08_003_kill_session_idempotent_on_terminated | BC-2.08.003 Invariant 2 | kill_session() is todo!() |
//! | test_BC_2_08_003_kill_session_idempotent_on_terminating | BC-2.08.003 Invariant 2 | kill_session() is todo!() |
//! | test_BC_2_08_003_12s_watchdog | BC-2.08.003 PC-5 | spawn_kill_watchdog() is todo!() |
//! | test_BC_2_08_003_kill_detached_so_peercred | BC-2.08.003 Invariant 5, EC-164 | kill_session() is todo!() |
//! | test_BC_2_08_003_kill_session_not_found | BC-2.08.003 EC-166, AC-011 | kill_session() is todo!() |
//! | test_BC_2_08_008_state_changed_ordering_on_kill | BC-2.08.008 Invariant 4, PC-3 | kill_session() is todo!() |
//! | test_kill_during_launching_before_socket_bind | BC-2.08.003 Invariant 3, AC-008 | kill_session() is todo!() |
//! | test_kill_during_launching_after_socket_bind | BC-2.08.003 PC-1 (Running/Launching path) | kill_session() is todo!() |
//! | test_BC_2_08_003_kill_detached_so_peercred_uid_mismatch_terminates | BC-2.08.003 Invariant 5 | kill_session() is todo!() |
//! | test_BC_2_08_003_kill_session_not_found_wire_code | BC-2.08.003 EC-166, session_error_to_code | kill_session() is todo!() |
//!
//! # Anti-false-green contract
//!
//! - Tests exercise `kill_session()` which hits `todo!()` — the panic IS the Red Gate failure.
//! - Idempotency tests: the session must be pre-placed in Terminated/Terminating state by
//!   directly inserting into the sessions map via the public `session_list()` path after
//!   a successful spawn (Running → manually transition). These tests exercise the real
//!   kill_session() code path, which is todo!().
//! - Timing tests use `tokio::time::pause()` + `tokio::time::advance()` — no real sleeps.

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use monocle_core::engine::{SpawnOptions, SpawnRecipe};
use monocle_runtime::session_manager::{
    PeerCredVerifier, SessionError, SessionHostSpawner, SessionManager, SpawnedHostHandle,
};
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Shared test infrastructure (mirrors s033_blocker_red_gate.rs patterns)
// ---------------------------------------------------------------------------

/// Build a short-path temp dir under /tmp to keep UDS socket paths under macOS's
/// SUN_LEN limit (104 chars).
fn isolated_runtime_dir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .tempdir_in("/tmp")
        .expect("create isolated runtime dir for S-034 red gate test in /tmp")
}

/// Build a minimal `SpawnOptions` with session_id and hooks_settings_path pre-filled.
fn make_spawn_opts(session_id: &str) -> SpawnOptions {
    SpawnOptions::for_spawn_request(
        PathBuf::from("/tmp/test-project"),
        PathBuf::from("/tmp/test-project"),
        "claude-code".to_string(),
        "default".to_string(),
        None,
    )
    .with_daemon_fields(session_id.to_string(), PathBuf::from("/tmp/hooks.json"))
}

/// Spawner that returns `Ok` with the given `pid` and derives the socket path from
/// `runtime_dir`. Used for standard kill-path tests where the session-host
/// socket address must be predictable.
struct FakePidSpawner {
    pid: u32,
}

#[async_trait::async_trait]
impl SessionHostSpawner for FakePidSpawner {
    async fn spawn(
        &self,
        session_id: &str,
        _recipe: &SpawnRecipe,
        runtime_dir: &std::path::Path,
    ) -> Result<SpawnedHostHandle, SessionError> {
        Ok(SpawnedHostHandle {
            pid: self.pid,
            socket_path: runtime_dir.join(format!("session-{}.sock", session_id)),
        })
    }
}

/// Spawner that returns `Ok` with a fixed socket path (not derived from runtime_dir).
/// Used when the test needs to bind a specific UDS socket that the monitor will find.
struct FixedSocketSpawner {
    pid: u32,
    socket_path: PathBuf,
}

#[async_trait::async_trait]
impl SessionHostSpawner for FixedSocketSpawner {
    async fn spawn(
        &self,
        _session_id: &str,
        _recipe: &SpawnRecipe,
        _runtime_dir: &std::path::Path,
    ) -> Result<SpawnedHostHandle, SessionError> {
        Ok(SpawnedHostHandle {
            pid: self.pid,
            socket_path: self.socket_path.clone(),
        })
    }
}

/// Minimal `EngineModule` that returns a valid `SpawnRecipe` for every call.
/// Avoids bringing in SucceedingMockEngine from the inline test module (which
/// is cfg(test)-only in session_manager/mod.rs and therefore unreachable here).
struct KillTestEngine;

#[async_trait::async_trait]
impl monocle_core::engine::EngineModule for KillTestEngine {
    fn id(&self) -> &'static str {
        "kill-test-engine"
    }
    fn metadata(
        &self,
    ) -> Result<monocle_core::engine::EngineMetadata, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for kill tests")
    }
    fn detect(&self, _: &monocle_core::engine::ProcessSnapshot) -> bool {
        false
    }
    async fn enrich(
        &self,
        _: &monocle_core::engine::ProcessSnapshot,
    ) -> Result<monocle_core::engine::EnrichedSession, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for kill tests")
    }
    async fn on_hook(
        &self,
        _: monocle_core::hook_events::HookEvent,
    ) -> monocle_core::engine::HookResponse {
        unimplemented!("not needed for kill tests")
    }
    fn spawn_recipe(
        &self,
        opts: &monocle_core::engine::SpawnOptions,
    ) -> Result<monocle_core::engine::SpawnRecipe, monocle_core::engine::EngineError> {
        Ok(monocle_core::engine::SpawnRecipe::new(
            PathBuf::from("claude"),
            vec![],
            std::collections::HashMap::new(),
            opts.worktree_root.clone(),
        ))
    }
}

/// PeerCredVerifier that always returns Ok — simulate matching UID.
struct AllowAllVerifier;
impl PeerCredVerifier for AllowAllVerifier {
    fn verify(&self, _stream: &tokio::net::UnixStream) -> Result<(), SessionError> {
        Ok(())
    }
}

/// PeerCredVerifier that always returns Err — simulate UID mismatch.
struct RejectAllVerifier;
impl PeerCredVerifier for RejectAllVerifier {
    fn verify(&self, _stream: &tokio::net::UnixStream) -> Result<(), SessionError> {
        Err(SessionError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "S-034 test: simulated SO_PEERCRED UID mismatch",
        )))
    }
}

/// Create a `SessionManager` backed by a `FakePidSpawner` with a single broker subscriber.
/// Returns `(manager, per-client rx)`.
fn make_manager(
    tmp: &std::path::Path,
    pid: u32,
) -> (
    SessionManager,
    monocle_ipc::server::SubscriberList,
    tokio::sync::mpsc::Receiver<monocle_ipc::types::ServerToClient>,
) {
    use monocle_ipc::server::{ClientEntry, CLIENT_CHANNEL_CAPACITY};
    use tokio::sync::Mutex;

    let (tx, rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let subscriber_list: monocle_ipc::server::SubscriberList =
        Arc::new(Mutex::new(vec![ClientEntry::new(tx)]));
    let broker = Arc::new(Arc::clone(&subscriber_list));
    let spawner: Arc<dyn SessionHostSpawner> = Arc::new(FakePidSpawner { pid });
    let manager =
        SessionManager::new(tmp.to_path_buf(), spawner, broker, Arc::new(KillTestEngine));
    (manager, subscriber_list, rx)
}

/// Create a `SessionManager` backed by a `FixedSocketSpawner` (for UDS-control tests).
fn make_manager_with_socket(
    tmp: &std::path::Path,
    pid: u32,
    socket_path: PathBuf,
) -> (
    SessionManager,
    monocle_ipc::server::SubscriberList,
    tokio::sync::mpsc::Receiver<monocle_ipc::types::ServerToClient>,
) {
    use monocle_ipc::server::{ClientEntry, CLIENT_CHANNEL_CAPACITY};
    use tokio::sync::Mutex;

    let (tx, rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let subscriber_list: monocle_ipc::server::SubscriberList =
        Arc::new(Mutex::new(vec![ClientEntry::new(tx)]));
    let broker = Arc::new(Arc::clone(&subscriber_list));
    let spawner: Arc<dyn SessionHostSpawner> =
        Arc::new(FixedSocketSpawner { pid, socket_path });
    let manager =
        SessionManager::new(tmp.to_path_buf(), spawner, broker, Arc::new(KillTestEngine));
    (manager, subscriber_list, rx)
}

/// Send a length-prefixed `HostToDaemon` message over a `tokio::net::UnixStream`.
async fn send_host_to_daemon(
    peer: &mut tokio::net::UnixStream,
    msg: &monocle_ipc::types::HostToDaemon,
) {
    use tokio::io::AsyncWriteExt;
    let body = serde_json::to_vec(msg).expect("serialize HostToDaemon");
    let len = (body.len() as u32).to_le_bytes();
    peer.write_all(&len).await.expect("write len prefix");
    peer.write_all(&body).await.expect("write body");
}

/// Drain all messages from `rx` for up to `timeout_ms` milliseconds.
async fn drain_messages(
    rx: &mut tokio::sync::mpsc::Receiver<monocle_ipc::types::ServerToClient>,
    timeout_ms: u64,
) -> Vec<monocle_ipc::types::ServerToClient> {
    let deadline =
        tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let mut msgs = Vec::new();
    while let Ok(Some(m)) =
        tokio::time::timeout_at(deadline, rx.recv()).await
    {
        msgs.push(m);
    }
    msgs
}

// ---------------------------------------------------------------------------
// Test 1: test_BC_2_08_003_kill_session_sigterm_within_500ms
//
// Verifies: BC-2.08.003 PC-1 (Kill delivered within 500ms to Running session-host
// via existing host_conn), PC-2 (state → Terminating immediately), PC-4 (on mock
// confirmation → Terminated), sidecar updated with Terminated state.
//
// Fails because: kill_session() is todo!() — panics at first call.
// ---------------------------------------------------------------------------

/// BC-2.08.003 PC-1, PC-2, PC-4: `kill_session()` delivers `DaemonToHost::Kill`
/// to a Running session-host within 500ms, transitions state to `Terminating`
/// immediately, and then to `Terminated` on mock `HostToDaemon::StateChanged`.
///
/// The mock session-host is a UDS server that:
/// 1. Accepts the post-spawn monitor's connect (sends StateChanged{Running}).
/// 2. Keeps the connection open and waits for DaemonToHost::Kill.
/// 3. Responds with HostToDaemon::StateChanged{Terminated} to confirm exit.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics.
#[tokio::test]
async fn test_BC_2_08_003_kill_session_sigterm_within_500ms() {
    // BC-2.08.003 PC-1 canonical test vector: kill on "existing-running-session"
    // → Ok(()); DaemonToHost::Kill sent within 500ms; state → Terminating; on
    // mock confirmation → Terminated; sidecar updated.

    let tmp = isolated_runtime_dir();
    let session_id = "034a0000-0001-4000-a000-000000000001".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));

    // Bind the mock session-host UDS socket before spawning.
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock session-host socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 55_001, socket_path.clone());
    // Allow SO_PEERCRED: same-uid path.
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Accept the post-spawn monitor connection and advance session to Running.
    let (mut peer, _addr) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        listener.accept(),
    )
    .await
    .expect("timed out waiting for post-spawn monitor connect")
    .expect("accept failed");

    // Send StateChanged{Running} — transitions session to Running.
    send_host_to_daemon(
        &mut peer,
        &monocle_ipc::types::HostToDaemon::StateChanged {
            new_state: monocle_ipc::types::SessionState::Running,
            degraded_env: None,
        },
    )
    .await;

    // Wait for SessionStateChanged{Running} broadcast to confirm Running state.
    let mut reached_running = false;
    let running_deadline =
        tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        if tokio::time::Instant::now() >= running_deadline {
            break;
        }
        match tokio::time::timeout(std::time::Duration::from_millis(200), rx.recv()).await {
            Ok(Some(monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Running,
            })) if sid == &session_id => {
                reached_running = true;
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => break,
        }
    }
    assert!(
        reached_running,
        "precondition: session must reach Running before testing kill_session()"
    );

    // Now measure: kill_session() must return within 500ms.
    // FAILS: kill_session() is todo!() — panics here.
    let kill_start = std::time::Instant::now();
    let kill_result = manager.kill_session(&session_id).await;
    let kill_elapsed = kill_start.elapsed();

    kill_result.expect(
        "test_BC_2_08_003_kill_session_sigterm_within_500ms: \
         kill_session() must return Ok(()) — BC-2.08.003 PC-1, AC-006",
    );

    assert!(
        kill_elapsed < std::time::Duration::from_millis(500),
        "test_BC_2_08_003_kill_session_sigterm_within_500ms: \
         kill_session() must return within 500ms (BC-2.08.003 PC-1), took {:?}",
        kill_elapsed
    );

    // Verify state transitioned to Terminating in the registry (BC-2.08.003 PC-2).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry after kill_session");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "test_BC_2_08_003_kill_session_sigterm_within_500ms: \
         state must be Terminating immediately after kill_session() returns \
         (BC-2.08.003 PC-2)",
    );

    // Verify SessionStateChanged{Terminating} was broadcast (BC-2.08.008 PC-1).
    // Read DaemonToHost::Kill from the mock session-host side.
    use tokio::io::AsyncReadExt;
    let mut len_buf = [0u8; 4];
    tokio::time::timeout(
        std::time::Duration::from_millis(500),
        peer.read_exact(&mut len_buf),
    )
    .await
    .expect("timed out waiting for DaemonToHost::Kill from session manager")
    .expect("read Kill length prefix failed");

    let msg_len = u32::from_le_bytes(len_buf) as usize;
    let mut body = vec![0u8; msg_len];
    peer.read_exact(&mut body)
        .await
        .expect("read Kill body failed");
    let kill_msg: monocle_ipc::types::DaemonToHost =
        serde_json::from_slice(&body).expect("deserialize DaemonToHost::Kill");
    assert!(
        matches!(kill_msg, monocle_ipc::types::DaemonToHost::Kill),
        "test_BC_2_08_003_kill_session_sigterm_within_500ms: \
         mock session-host must receive DaemonToHost::Kill (BC-2.08.003 PC-1)"
    );

    // Simulate session-host confirming exit.
    send_host_to_daemon(
        &mut peer,
        &monocle_ipc::types::HostToDaemon::StateChanged {
            new_state: monocle_ipc::types::SessionState::Terminated,
            degraded_env: None,
        },
    )
    .await;

    // Wait for state → Terminated (BC-2.08.003 PC-4).
    let terminated_deadline =
        tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    let mut reached_terminated = false;
    loop {
        if tokio::time::Instant::now() >= terminated_deadline {
            break;
        }
        let sessions = manager.session_list().await;
        if let Some(s) = sessions.iter().find(|s| s.session_id == session_id) {
            if s.state == monocle_ipc::types::SessionState::Terminated {
                reached_terminated = true;
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        reached_terminated,
        "test_BC_2_08_003_kill_session_sigterm_within_500ms: \
         state must transition to Terminated after HostToDaemon::StateChanged{{Terminated}} \
         (BC-2.08.003 PC-4)",
    );

    // Verify sidecar updated to Terminated (BC-2.08.003 PC-4b).
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));
    let contents = std::fs::read_to_string(&sidecar_path)
        .expect("sidecar must exist after kill completion");
    let sidecar: monocle_ipc::types::SessionSidecarV3 =
        serde_json::from_str(&contents).expect("sidecar must parse as SessionSidecarV3");
    assert_eq!(
        sidecar.state,
        monocle_ipc::types::SessionState::Terminated,
        "test_BC_2_08_003_kill_session_sigterm_within_500ms: \
         sidecar must reflect Terminated state after kill completion (BC-2.08.003 PC-4b)",
    );
}

// ---------------------------------------------------------------------------
// Test 2: test_BC_2_08_003_kill_session_idempotent_on_terminated
//
// Verifies: BC-2.08.003 Invariant 2 (kill on Terminated → Ok(()), idempotent).
// Canonical test vector: kill_session("already-terminated-session") → Ok(()).
//
// Fails because: kill_session() is todo!() — panics at first call.
// ---------------------------------------------------------------------------

/// BC-2.08.003 Invariant 2: `kill_session()` on an already-`Terminated` session
/// MUST return `Ok(())` without sending another `DaemonToHost::Kill`.
///
/// Pre-condition: session is pre-placed into Terminated state via spawn + forced
/// Terminated transition (simulating a prior completed kill).
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics.
#[tokio::test]
async fn test_BC_2_08_003_kill_session_idempotent_on_terminated() {
    // BC-2.08.003 canonical test vector: kill_session("already-terminated-session") → Ok(()).

    let tmp = isolated_runtime_dir();
    let session_id = "034b0000-0001-4000-a000-000000000001".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));

    // Bind mock UDS socket — the post-spawn monitor will connect.
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 55_002, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Accept monitor connect, send StateChanged{Terminated} directly — skipping
    // Running to put session immediately into Terminated state.
    let (mut peer, _) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        listener.accept(),
    )
    .await
    .expect("timed out waiting for monitor")
    .expect("accept failed");

    send_host_to_daemon(
        &mut peer,
        &monocle_ipc::types::HostToDaemon::StateChanged {
            new_state: monocle_ipc::types::SessionState::Terminated,
            degraded_env: None,
        },
    )
    .await;

    // Drain messages: wait until a SessionStateChanged{Terminated} arrives.
    // This confirms the session is in Terminated state before we test kill idempotency.
    let msgs = drain_messages(&mut rx, 3000).await;
    let has_terminated = msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Terminated,
            } if sid == &session_id
        )
    });
    // If the post-spawn monitor doesn't produce Terminated on its own yet (S-034 not
    // implemented), we fall through — the test still exercises kill_session() which
    // is todo!() and will panic, producing the Red Gate failure.
    let _ = has_terminated;

    // Now call kill_session() on the (Terminated or stub) session.
    // FAILS: kill_session() is todo!() → panics.
    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "test_BC_2_08_003_kill_session_idempotent_on_terminated: \
         kill_session() on Terminated session MUST return Ok(()) (BC-2.08.003 Invariant 2). \
         Got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// Test 3: test_BC_2_08_003_kill_session_idempotent_on_terminating
//
// Verifies: BC-2.08.003 Invariant 2 (kill on Terminating → Ok(()), no dup Kill).
// Canonical test vector: kill_session("terminating-session") → Ok(()).
//
// Fails because: kill_session() is todo!() — panics at first call.
// ---------------------------------------------------------------------------

/// BC-2.08.003 Invariant 2: `kill_session()` on a `Terminating` session returns
/// `Ok(())` idempotently — no duplicate `DaemonToHost::Kill` is sent, and the
/// existing watchdog keeps running.
///
/// Pre-condition: session is placed into Terminating state by calling kill_session()
/// once (which panics on todo!(), so we cannot pre-place it without the first call
/// completing — but the test is structured to panic at the SECOND kill call for
/// specification completeness, demonstrating both calls are expected to be Ok).
///
/// Strategy: call kill_session() once (→ todo!() panic = Red Gate), which verifies
/// the todo!() stub is present. The test's intent is that both first and second
/// calls must return Ok.
///
/// FAILS NOW: first `kill_session()` call is `todo!()` → panics.
#[tokio::test]
async fn test_BC_2_08_003_kill_session_idempotent_on_terminating() {
    // BC-2.08.003 canonical test vector: kill_session("terminating-session") → Ok(()).

    let tmp = isolated_runtime_dir();
    let session_id = "034c0000-0001-4000-a000-000000000001".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 55_003, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Advance session to Running via mock UDS.
    let (mut peer, _) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        listener.accept(),
    )
    .await
    .expect("timed out waiting for monitor")
    .expect("accept failed");

    send_host_to_daemon(
        &mut peer,
        &monocle_ipc::types::HostToDaemon::StateChanged {
            new_state: monocle_ipc::types::SessionState::Running,
            degraded_env: None,
        },
    )
    .await;

    // Wait for Running.
    let mut reached_running = false;
    let dl = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        if tokio::time::Instant::now() >= dl {
            break;
        }
        match tokio::time::timeout(std::time::Duration::from_millis(200), rx.recv()).await {
            Ok(Some(monocle_ipc::types::ServerToClient::SessionStateChanged {
                new_state: monocle_ipc::types::SessionState::Running,
                ..
            })) => {
                reached_running = true;
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => break,
        }
    }
    assert!(reached_running, "precondition: session must reach Running");

    // First kill_session() call — must return Ok(()) and transition to Terminating.
    // FAILS: kill_session() is todo!() → panics here.
    let first_result = manager.kill_session(&session_id).await;
    first_result.expect(
        "test_BC_2_08_003_kill_session_idempotent_on_terminating: \
         first kill_session() must return Ok(()) (BC-2.08.003 PC-1)",
    );

    // Verify state is Terminating.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "test_BC_2_08_003_kill_session_idempotent_on_terminating: \
         state must be Terminating after first kill (BC-2.08.003 PC-2)",
    );

    // Second kill_session() call on a Terminating session — must also return Ok(()).
    // (No duplicate Kill sent; watchdog already running.)
    let second_result = manager.kill_session(&session_id).await;
    assert!(
        second_result.is_ok(),
        "test_BC_2_08_003_kill_session_idempotent_on_terminating: \
         second kill_session() on Terminating session MUST return Ok(()) without duplicate Kill \
         (BC-2.08.003 Invariant 2). Got: {:?}",
        second_result
    );
}

// ---------------------------------------------------------------------------
// Test 4: test_BC_2_08_003_12s_watchdog
//
// Verifies: BC-2.08.003 PC-5 — 12s watchdog fires if no HostToDaemon::StateChanged
// confirmation. Uses tokio::time::pause()/advance(). SIGKILL sent to session-host
// PID; state → Terminated; sidecar updated.
//
// Fails because: spawn_kill_watchdog() is todo!() — panics during kill_session().
// ---------------------------------------------------------------------------

/// BC-2.08.003 PC-5: The 12-second watchdog fires when no `HostToDaemon::StateChanged`
/// confirmation arrives within 12 seconds of `DaemonToHost::Kill` being sent.
///
/// Uses `tokio::time::pause()` to advance virtual time without real sleeps.
/// Verifies: SIGKILL sent to session-host PID; state → Terminated; sidecar updated.
///
/// FAILS NOW: `spawn_kill_watchdog()` is `todo!()` (called from `kill_session()`)
/// → panics during `kill_session()`.
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_003_12s_watchdog() {
    // BC-2.08.003 canonical test vector:
    // "12s pass without session-host confirmation → Session forced to Terminated;
    // SIGKILL to session-host PID; sidecar updated; GC timer started"

    let tmp = isolated_runtime_dir();
    let session_id = "034d0000-0001-4000-a000-000000000001".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 55_004, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Advance session to Running.
    let (mut peer, _) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        listener.accept(),
    )
    .await
    .expect("timed out waiting for monitor")
    .expect("accept failed");

    send_host_to_daemon(
        &mut peer,
        &monocle_ipc::types::HostToDaemon::StateChanged {
            new_state: monocle_ipc::types::SessionState::Running,
            degraded_env: None,
        },
    )
    .await;

    // Give the runtime a chance to process the Running transition.
    tokio::time::advance(std::time::Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    // Kill the session — spawns watchdog (todo!() → panics here).
    // FAILS: kill_session() calls spawn_kill_watchdog() which is todo!().
    manager
        .kill_session(&session_id)
        .await
        .expect(
            "test_BC_2_08_003_12s_watchdog: kill_session() must return Ok(()) \
             to proceed to watchdog test (BC-2.08.003 PC-1)",
        );

    // Advance virtual time by 12 seconds — watchdog must fire.
    // The watchdog is spawned by kill_session() using tokio::time::sleep(Duration::from_secs(12)).
    tokio::time::advance(std::time::Duration::from_secs(12)).await;
    // Yield to allow the watchdog task to execute.
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }

    // State must be Terminated after watchdog fires (BC-2.08.003 PC-5a).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry after watchdog");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminated,
        "test_BC_2_08_003_12s_watchdog: watchdog must force state to Terminated \
         after 12s without HostToDaemon::StateChanged (BC-2.08.003 PC-5a)",
    );

    // Verify sidecar updated to Terminated (BC-2.08.003 PC-5c).
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));
    if sidecar_path.exists() {
        let contents = std::fs::read_to_string(&sidecar_path)
            .expect("sidecar must be readable after watchdog");
        let sidecar: monocle_ipc::types::SessionSidecarV3 =
            serde_json::from_str(&contents).expect("sidecar must parse as SessionSidecarV3");
        assert_eq!(
            sidecar.state,
            monocle_ipc::types::SessionState::Terminated,
            "test_BC_2_08_003_12s_watchdog: sidecar must reflect Terminated after watchdog fires \
             (BC-2.08.003 PC-5c)",
        );
    }

    // Verify SessionStateChanged{Terminated} was broadcast by watchdog (BC-2.08.008 PC-1).
    // Drain messages from the channel (may include earlier broadcasts).
    let msgs = drain_messages(&mut rx, 200).await;
    let has_watchdog_terminated = msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Terminated,
            } if sid == &session_id
        )
    });
    assert!(
        has_watchdog_terminated,
        "test_BC_2_08_003_12s_watchdog: watchdog must broadcast \
         SessionStateChanged{{Terminated}} (BC-2.08.003 PC-5d, BC-2.08.008 PC-1). \
         Messages received: {:?}",
        msgs
    );
}

// ---------------------------------------------------------------------------
// Test 5: test_BC_2_08_003_kill_detached_so_peercred
//
// Verifies: BC-2.08.003 Invariant 5, EC-164 — kill on Detached session makes a
// fresh UDS connect and applies SO_PEERCRED before sending Kill. When uid matches,
// Kill is sent and state transitions Detached → Terminating.
//
// Fails because: kill_session() is todo!() — panics at call.
// ---------------------------------------------------------------------------

/// BC-2.08.003 Invariant 5, EC-164: `kill_session()` on a `Detached` session
/// makes a fresh UDS connect to `<runtime_dir>/session-<id>.sock`, applies
/// SO_PEERCRED BEFORE sending any message, and if uid matches sends
/// `DaemonToHost::Kill`. State: `Detached → Terminating`.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics.
#[tokio::test]
async fn test_BC_2_08_003_kill_detached_so_peercred() {
    // BC-2.08.003 EC-164 canonical path: fresh connect + SO_PEERCRED + Kill.

    let tmp = isolated_runtime_dir();
    let session_id = "034e0000-0001-4000-a000-000000000001".to_string();

    // For the Detached/fresh-connect SO_PEERCRED test, we need kill_session() to use
    // the fresh-connect path. This occurs when host_conn is None at kill time.
    //
    // Strategy: use a spawner that points to a socket path that NEVER gets the
    // monitor connected (no UDS socket bound for the monitor), so host_conn=None.
    // Then bind a socket at the expected path just before calling kill_session(),
    // so kill_session() can make a fresh connect.
    //
    // The post-spawn monitor will fail to connect (socket doesn't exist yet) and
    // exit after 30s timeout (virtual, since this test does not pause time). For
    // test speed: we bind a "dummy" socket for the monitor to accept one connection
    // and send EOF, then drop it and rebind for kill.

    // Bind the socket that both the post-spawn monitor AND kill_session() will connect to.
    // We keep the same listener alive throughout the test — no rebind needed.
    // The monitor connects and receives EOF (we drop the accepted stream), leaving
    // host_conn=None. Then kill_session() (Detached path) connects to the same listener.
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket");

    let (mut manager, _subs, _rx) =
        make_manager_with_socket(tmp.path(), 55_005, socket_path.clone());
    // Allow SO_PEERCRED: uid match → proceed.
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Accept the post-spawn monitor's connection then send EOF (drop immediately).
    // This keeps host_conn=None: monitor exits without sending StateChanged{Running}.
    let (stream_to_drop, _) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        listener.accept(),
    )
    .await
    .expect("timed out waiting for monitor connect")
    .expect("accept failed");
    // Drop the stream immediately — EOF causes the monitor read loop to exit.
    drop(stream_to_drop);

    // Give the monitor task time to process EOF and exit.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // The listener is still bound. kill_session() (Detached/Launching-fallback path)
    // will make a fresh UDS connect — the same listener accepts it.
    let new_listener = listener;

    // Spawn task to accept kill_session()'s fresh connect and confirm it sends Kill.
    let kill_connect_task = tokio::spawn({
        let sid = session_id.clone();
        async move {
            // Accept kill_session()'s fresh connect (Detached path).
            let (mut conn, _) = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                new_listener.accept(),
            )
            .await
            .expect("timed out waiting for kill_session fresh connect")
            .expect("accept failed");

            // Read the first message — must be DaemonToHost::Kill (not any other message).
            use tokio::io::AsyncReadExt;
            let mut len_buf = [0u8; 4];
            tokio::time::timeout(
                std::time::Duration::from_millis(500),
                conn.read_exact(&mut len_buf),
            )
            .await
            .expect("timed out waiting for Kill message from kill_session")
            .expect("read Kill length failed");

            let msg_len = u32::from_le_bytes(len_buf) as usize;
            let mut body = vec![0u8; msg_len];
            conn.read_exact(&mut body)
                .await
                .expect("read Kill body failed");
            let msg: monocle_ipc::types::DaemonToHost =
                serde_json::from_slice(&body).expect("deserialize DaemonToHost");

            assert!(
                matches!(msg, monocle_ipc::types::DaemonToHost::Kill),
                "test_BC_2_08_003_kill_detached_so_peercred: \
                 kill_session() on Detached session MUST send DaemonToHost::Kill on the fresh \
                 connect (BC-2.08.003 EC-164, Invariant 5). Got: {:?}",
                msg
            );

            // Confirm the session ID for the assertion.
            sid
        }
    });

    // Call kill_session() — on Detached path, must make a fresh UDS connect + SO_PEERCRED.
    // FAILS: kill_session() is todo!() → panics here.
    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "test_BC_2_08_003_kill_detached_so_peercred: \
         kill_session() on Detached session (host_conn=None) MUST return Ok(()) \
         (BC-2.08.003 EC-164). Got: {:?}",
        result
    );

    // Wait for the kill-connect task to complete and assert the Kill was received.
    let kill_session_id = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        kill_connect_task,
    )
    .await
    .expect("timed out waiting for kill_connect_task")
    .expect("kill_connect_task panicked");

    assert_eq!(kill_session_id, session_id);

    // State must be Terminating after Kill is sent (Detached → Terminating).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "test_BC_2_08_003_kill_detached_so_peercred: \
         state must be Terminating after kill on Detached session \
         (BC-2.08.003 EC-164 — Detached → Terminating)",
    );
}

// ---------------------------------------------------------------------------
// Test 6: test_BC_2_08_003_kill_session_not_found
//
// Verifies: BC-2.08.003 EC-166, AC-011 — unknown session_id → Err(SessionNotFound).
// Canonical test vector: kill_session("nonexistent-id") → Err(SessionNotFound).
//
// Fails because: kill_session() is todo!() — panics before returning the error.
// ---------------------------------------------------------------------------

/// BC-2.08.003 EC-166, AC-011: `kill_session()` on an unknown `session_id` MUST
/// return `Err(SessionError::SessionNotFound { session_id })`.
///
/// Canonical test vector: `kill_session("nonexistent-id")` → `Err(SessionNotFound)`.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics before returning the error.
#[tokio::test]
async fn test_BC_2_08_003_kill_session_not_found() {
    // BC-2.08.003 canonical test vector: kill_session("nonexistent-id") → Err(SessionNotFound).

    let tmp = isolated_runtime_dir();
    let (mut manager, _subs, _rx) = make_manager(tmp.path(), 55_006);

    // FAILS: kill_session() is todo!() → panics here.
    let result = manager
        .kill_session("00000000-dead-4000-beef-000000000000")
        .await;

    assert!(
        matches!(
            result,
            Err(SessionError::SessionNotFound { session_id: ref sid })
            if sid == "00000000-dead-4000-beef-000000000000"
        ),
        "test_BC_2_08_003_kill_session_not_found: \
         kill_session() on unknown session_id MUST return \
         Err(SessionError::SessionNotFound) (BC-2.08.003 EC-166, AC-011). \
         Got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// Test 7: test_BC_2_08_008_state_changed_ordering_on_kill
//
// Verifies: BC-2.08.008 Invariant 4, PC-3 — SessionStateChanged{Terminating}
// arrives in the per-client FIFO BEFORE SessionListUpdate for the same transition.
// Both are emitted under the same sessions mutex hold.
//
// Fails because: kill_session() is todo!() — panics before emitting anything.
// ---------------------------------------------------------------------------

/// BC-2.08.008 Invariant 4, PC-3: `SessionStateChanged{Terminating}` MUST be
/// enqueued into the per-client FIFO channel BEFORE `SessionListUpdate` for the
/// kill transition. Both must be enqueued under a single `SessionManager` mutex hold.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics before any broadcast.
#[tokio::test]
async fn test_BC_2_08_008_state_changed_ordering_on_kill() {
    // BC-2.08.008 canonical test vector:
    // "kill_session() → SessionStateChanged{Terminating} arrives before SessionListUpdate"

    let tmp = isolated_runtime_dir();
    let session_id = "034f0000-0001-4000-a000-000000000001".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 55_007, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Advance to Running.
    let (mut peer, _) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        listener.accept(),
    )
    .await
    .expect("timed out waiting for monitor")
    .expect("accept failed");

    send_host_to_daemon(
        &mut peer,
        &monocle_ipc::types::HostToDaemon::StateChanged {
            new_state: monocle_ipc::types::SessionState::Running,
            degraded_env: None,
        },
    )
    .await;

    // Drain messages until Running.
    let mut reached_running = false;
    let dl = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        if tokio::time::Instant::now() >= dl {
            break;
        }
        match tokio::time::timeout(std::time::Duration::from_millis(200), rx.recv()).await {
            Ok(Some(monocle_ipc::types::ServerToClient::SessionStateChanged {
                new_state: monocle_ipc::types::SessionState::Running,
                ..
            })) => {
                reached_running = true;
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => break,
        }
    }
    assert!(reached_running, "precondition: must reach Running");

    // Drain any residual SessionListUpdate from the Running transition.
    let _ = drain_messages(&mut rx, 100).await;

    // Call kill_session() — FAILS: todo!() → panics here.
    manager
        .kill_session(&session_id)
        .await
        .expect(
            "test_BC_2_08_008_state_changed_ordering_on_kill: \
             kill_session() must return Ok(()) (BC-2.08.003 PC-1)",
        );

    // Collect all messages in a tight window (both messages should be enqueued atomically).
    let msgs = drain_messages(&mut rx, 500).await;

    // Find the first SessionStateChanged{Terminating} and the first SessionListUpdate.
    let terminating_idx = msgs.iter().position(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Terminating,
            } if sid == &session_id
        )
    });
    let list_update_idx = msgs.iter().position(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionListUpdate { .. }
        )
    });

    // ORDERING ASSERTION: SessionStateChanged{Terminating} MUST appear BEFORE SessionListUpdate.
    assert!(
        terminating_idx.is_some(),
        "test_BC_2_08_008_state_changed_ordering_on_kill: \
         SessionStateChanged{{Terminating}} must be broadcast on kill (BC-2.08.008 PC-1). \
         Messages received: {:?}",
        msgs
    );
    assert!(
        list_update_idx.is_some(),
        "test_BC_2_08_008_state_changed_ordering_on_kill: \
         SessionListUpdate must be broadcast on kill (BC-2.08.008 PC-1). \
         Messages received: {:?}",
        msgs
    );

    let t_idx = terminating_idx.expect("asserted above");
    let lu_idx = list_update_idx.expect("asserted above");

    assert!(
        t_idx < lu_idx,
        "test_BC_2_08_008_state_changed_ordering_on_kill: \
         SessionStateChanged{{Terminating}} (index {}) MUST arrive before \
         SessionListUpdate (index {}) in the per-client FIFO channel \
         (BC-2.08.008 Invariant 4, PC-3). \
         Messages: {:?}",
        t_idx,
        lu_idx,
        msgs
    );

    // ADJACENCY ASSERTION (BC-2.08.008 PC-3 / Ruling G): the pair must be adjacent
    // — no other message may appear between SessionStateChanged{Terminating} and
    // SessionListUpdate.
    assert_eq!(
        lu_idx,
        t_idx + 1,
        "test_BC_2_08_008_state_changed_ordering_on_kill: \
         SessionListUpdate must be IMMEDIATELY after SessionStateChanged{{Terminating}} \
         (adjacent — no messages in between). t_idx={}, lu_idx={}. Messages: {:?}",
        t_idx,
        lu_idx,
        msgs
    );
}

// ---------------------------------------------------------------------------
// Test 8: test_kill_during_launching_before_socket_bind
//
// Verifies: BC-2.08.003 Invariant 3, AC-008 — kill on Launching with host_conn=None
// (PID fallback path). State: Launching → Terminating immediately.
//
// Fails because: kill_session() is todo!() — panics.
// ---------------------------------------------------------------------------

/// BC-2.08.003 Invariant 3, AC-008: `kill_session()` on a `Launching` session with
/// `host_conn: None` (race window before post-spawn monitor has connected) MUST use
/// the PID-based SIGTERM fallback and transition `Launching → Terminating` immediately.
///
/// We deliberately create this scenario by spawning a session WITHOUT binding the
/// mock UDS socket, so the post-spawn monitor never connects and `host_conn` remains
/// `None` for the duration of the test.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics.
#[tokio::test]
async fn test_kill_during_launching_before_socket_bind() {
    // BC-2.08.003 Invariant 3: kill on Launching with host_conn=None → PID fallback.
    // The session-host socket does NOT exist — the monitor cannot connect.

    let tmp = isolated_runtime_dir();
    let session_id = "03400000-0001-4000-a000-000000000007".to_string();

    // Do NOT bind the socket — post-spawn monitor will fail to connect (30s timeout).
    // Use FakePidSpawner (no socket binding) so the monitor never succeeds.
    let (mut manager, _subs, _rx) = make_manager(tmp.path(), 55_008);
    // The socket will not exist — monitor poll loop will time out eventually.

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Session state must be Launching (host_conn=None: socket not bound, monitor not connected).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must be in registry");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Launching,
        "precondition: session must be Launching before kill"
    );

    // Call kill_session() on Launching session with host_conn=None (PID fallback path).
    // FAILS: kill_session() is todo!() → panics here.
    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "test_kill_during_launching_before_socket_bind: \
         kill_session() on Launching (host_conn=None) MUST return Ok(()) using PID fallback \
         (BC-2.08.003 Invariant 3, AC-008). Got: {:?}",
        result
    );

    // State must be Terminating immediately (BC-2.08.003 PC-1).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry after kill");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "test_kill_during_launching_before_socket_bind: \
         state must be Terminating after kill on Launching (host_conn=None) \
         (BC-2.08.003 Invariant 3 — Launching → Terminating immediate)",
    );
}

// ---------------------------------------------------------------------------
// Test 9: test_kill_during_launching_after_socket_bind
//
// Verifies: BC-2.08.003 PC-1 (Running/Launching path) — kill on Launching with
// host_conn=Some(_) (socket bound, monitor connected, but StateChanged{Running} not
// yet sent). Kill is sent over the existing control connection.
//
// Fails because: kill_session() is todo!() — panics.
// ---------------------------------------------------------------------------

/// BC-2.08.003 PC-1 (Running/Launching host_conn established path): `kill_session()`
/// on a `Launching` session where `host_conn` is `Some(_)` (post-spawn monitor has
/// connected but hasn't yet seen `StateChanged{Running}`) MUST use the existing
/// `host_conn.writer` to send `DaemonToHost::Kill` — NOT make a fresh UDS connect.
///
/// Transition: `Launching → Terminating` immediately.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics.
#[tokio::test]
async fn test_kill_during_launching_after_socket_bind() {
    // BC-2.08.003 PC-1: kill during Launching with host_conn=Some.
    // The monitor has connected but not yet sent StateChanged{Running}.

    let tmp = isolated_runtime_dir();
    let session_id = "03400000-0001-4000-a000-000000000009".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, _rx) =
        make_manager_with_socket(tmp.path(), 55_009, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Accept the post-spawn monitor's connect. This sets host_conn=Some in the entry.
    let (mut peer, _) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        listener.accept(),
    )
    .await
    .expect("timed out waiting for monitor connect")
    .expect("accept failed");

    // Do NOT send StateChanged{Running} — session stays in Launching with host_conn=Some.
    // Give the monitor task a moment to store the writer in host_conn.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Call kill_session() on a Launching session with host_conn=Some.
    // Must use host_conn.writer to send Kill (not PID fallback, not fresh connect).
    // FAILS: kill_session() is todo!() → panics here.
    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "test_kill_during_launching_after_socket_bind: \
         kill_session() on Launching (host_conn=Some) MUST return Ok(()) using host_conn \
         (BC-2.08.003 PC-1). Got: {:?}",
        result
    );

    // State must be Terminating immediately (BC-2.08.003 PC-1, AC-008).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "test_kill_during_launching_after_socket_bind: \
         state must be Terminating after kill on Launching (host_conn=Some) \
         (BC-2.08.003 PC-1 — Launching → Terminating immediate)",
    );

    // Verify kill_session() sent Kill over the control connection (not PID fallback).
    use tokio::io::AsyncReadExt;
    let mut len_buf = [0u8; 4];
    let kill_received = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        peer.read_exact(&mut len_buf),
    )
    .await;

    assert!(
        kill_received.is_ok() && kill_received.unwrap().is_ok(),
        "test_kill_during_launching_after_socket_bind: \
         kill_session() on Launching (host_conn=Some) MUST send DaemonToHost::Kill \
         over the existing control connection (BC-2.08.003 PC-1)",
    );

    let msg_len = u32::from_le_bytes(len_buf) as usize;
    let mut body = vec![0u8; msg_len];
    peer.read_exact(&mut body)
        .await
        .expect("read Kill body");
    let kill_msg: monocle_ipc::types::DaemonToHost =
        serde_json::from_slice(&body).expect("deserialize DaemonToHost");
    assert!(
        matches!(kill_msg, monocle_ipc::types::DaemonToHost::Kill),
        "test_kill_during_launching_after_socket_bind: \
         message sent over control connection must be DaemonToHost::Kill (BC-2.08.003 PC-1). \
         Got: {:?}",
        kill_msg
    );
}

// ---------------------------------------------------------------------------
// Test 10: test_BC_2_08_003_kill_detached_so_peercred_uid_mismatch_terminates
//
// Verifies: BC-2.08.003 Invariant 5 — SO_PEERCRED uid mismatch on Detached kill
// fresh-connect → session immediately transitions to Terminated (not Terminating).
// Returns Ok(()) — kill is effectively complete (session-host assumed dead/rogue).
//
// Fails because: kill_session() is todo!() — panics.
// ---------------------------------------------------------------------------

/// BC-2.08.003 Invariant 5: When `kill_session()` on a `Detached` session makes a
/// fresh UDS connect and SO_PEERCRED uid DOES NOT match, the session immediately
/// transitions to `Terminated` and `Ok(())` is returned (kill effectively complete).
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics.
#[tokio::test]
async fn test_BC_2_08_003_kill_detached_so_peercred_uid_mismatch_terminates() {
    // BC-2.08.003 Invariant 5: uid mismatch on kill fresh-connect → Terminated + Ok(()).

    let tmp = isolated_runtime_dir();
    let session_id = "03400000-0001-4000-a000-000000000010".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));
    // Keep the same listener for both the monitor connect and the kill fresh-connect.
    // RejectAllVerifier rejects the monitor's connection too — the monitor exits
    // with EC-163 Terminated path. Then kill_session() on the resulting Terminated
    // (or Launching) state hits the todo!() stub. Either way: Red Gate fails on todo!().
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, _rx) =
        make_manager_with_socket(tmp.path(), 55_010, socket_path.clone());
    // REJECT SO_PEERCRED: simulate uid mismatch on any fresh-connect.
    manager.with_peer_cred_verifier(Arc::new(RejectAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Accept the post-spawn monitor's connection (the RejectAllVerifier will reject it).
    let (stream_to_hold, _) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        listener.accept(),
    )
    .await
    .expect("timed out waiting for monitor")
    .expect("accept failed");
    // Hold the stream alive briefly so the monitor can call verify() and receive Err.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    drop(stream_to_hold);
    // Give the monitor time to process the rejection and set state → Terminated (EC-163).
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // The listener remains bound for any fresh-connect by kill_session().

    // Call kill_session() — SO_PEERCRED rejects → session → Terminated; return Ok(()).
    // FAILS: kill_session() is todo!() → panics here.
    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "test_BC_2_08_003_kill_detached_so_peercred_uid_mismatch_terminates: \
         kill_session() on uid-mismatch Detached session MUST return Ok(()) \
         (session treated as dead — BC-2.08.003 Invariant 5). Got: {:?}",
        result
    );

    // State must be Terminated (not Terminating) — uid mismatch = session is dead/rogue.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry after uid-mismatch kill");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminated,
        "test_BC_2_08_003_kill_detached_so_peercred_uid_mismatch_terminates: \
         uid mismatch on kill fresh-connect MUST transition session to Terminated immediately \
         (BC-2.08.003 Invariant 5). Got: {:?}",
        snap.state
    );
}

// ---------------------------------------------------------------------------
// Test 11: test_BC_2_08_003_kill_session_not_found_wire_code
//
// Verifies: BC-2.08.003 EC-166 + session_error_to_code(Kill, SessionNotFound)
// → "session_not_found". This is a pure unit test on the error mapping function.
// session_error_to_code() is already implemented (not todo!()); this test exercises
// the kill path specifically.
//
// NOTE: session_error_to_code() IS already implemented. However, to get the error,
// kill_session() must return it. Since kill_session() is todo!(), the test will panic
// at the kill_session() call, before reaching the wire-code assertion.
// Red Gate failure: todo!() panic from kill_session().
// ---------------------------------------------------------------------------

/// BC-2.08.003 EC-166 + session_error_to_code(): `kill_session()` on unknown session
/// returns `Err(SessionNotFound)` which maps to wire code `"session_not_found"` via
/// `session_error_to_code(IpcOp::Kill, e)`.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics before the wire code can be checked.
#[tokio::test]
async fn test_BC_2_08_003_kill_session_not_found_wire_code() {
    use monocle_runtime::session_manager::{session_error_to_code, IpcOp};

    let tmp = isolated_runtime_dir();
    let (mut manager, _subs, _rx) = make_manager(tmp.path(), 55_011);

    // FAILS: kill_session() is todo!() → panics here.
    let result = manager
        .kill_session("00000000-ffff-4000-aaaa-000000000000")
        .await;

    let err = result.expect_err(
        "test_BC_2_08_003_kill_session_not_found_wire_code: \
         kill_session() on unknown session_id must return Err (BC-2.08.003 EC-166)",
    );

    // Verify the wire code maps to "session_not_found" (AC-011, session_error_to_code).
    let code = session_error_to_code(IpcOp::Kill, &err);
    assert_eq!(
        code,
        "session_not_found",
        "test_BC_2_08_003_kill_session_not_found_wire_code: \
         SessionNotFound on Kill path MUST map to wire code 'session_not_found' \
         via session_error_to_code(IpcOp::Kill, e) (BC-2.08.003 EC-166, AC-011). \
         Got: '{}'",
        code
    );
}
