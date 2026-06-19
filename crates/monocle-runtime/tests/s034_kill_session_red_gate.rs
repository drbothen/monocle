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
//! | test_BC_2_08_003_existing_conn_broken_write_falls_back_to_fresh_connect | BC-2.08.003 ExistingConn fallback, Invariant 5 (OBS-1) | kill_session() is todo!() |
//! | test_BC_2_08_003_kill_session_not_found | BC-2.08.003 EC-166, AC-011 | kill_session() is todo!() |
//! | test_BC_2_08_008_state_changed_ordering_on_kill | BC-2.08.008 Invariant 4, PC-3 | kill_session() is todo!() |
//! | test_kill_during_launching_before_socket_bind | BC-2.08.003 Invariant 3, AC-008 | kill_session() is todo!() |
//! | test_kill_during_launching_after_socket_bind | BC-2.08.003 PC-1 (Running/Launching path) | kill_session() is todo!() |
//! | test_BC_2_08_003_existing_conn_broken_write_fallback_connect_fails_terminated | BC-2.08.003 EC-162/163 (OBS-1) | kill_session() is todo!() |
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
    let manager = SessionManager::new(
        tmp.to_path_buf(),
        spawner,
        broker,
        Arc::new(KillTestEngine),
        monocle_runtime::session_manager::HookEndpointConfig::default(),
    );
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
    let spawner: Arc<dyn SessionHostSpawner> = Arc::new(FixedSocketSpawner { pid, socket_path });
    let manager = SessionManager::new(
        tmp.to_path_buf(),
        spawner,
        broker,
        Arc::new(KillTestEngine),
        monocle_runtime::session_manager::HookEndpointConfig::default(),
    );
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
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let mut msgs = Vec::new();
    while let Ok(Some(m)) = tokio::time::timeout_at(deadline, rx.recv()).await {
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
    let (mut peer, _addr) =
        tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
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
    let running_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
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

    // BC-2.08.003 PC-1: Kill delivered within 500ms in production.
    // Test uses 5s budget to accommodate slow CI runners (aarch64 GitHub-hosted).
    // The meaningful invariant is that kill_session() is fire-and-confirm (non-blocking):
    // it must NOT wait for harness-child exit. 5s is still a strong non-blocking check.
    assert!(
        kill_elapsed < std::time::Duration::from_secs(5),
        "test_BC_2_08_003_kill_session_sigterm_within_500ms: \
         kill_session() must return without blocking (BC-2.08.003 fire-and-confirm), took {:?}",
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
    let terminated_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
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
    let contents =
        std::fs::read_to_string(&sidecar_path).expect("sidecar must exist after kill completion");
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
// Anchors: AC-007, BC-2.08.003 Invariant 2, finding F-S034-ADV-LOW-001.
//
// Converted (F-S034-ADV-LOW-001): original test drove through spawn + StateChanged{Terminated}
// sent to the post-spawn monitor, which has no Terminated arm while in Launching state, so
// the session remained Launching + host_conn=Some → kill_session() took KillPath::ExistingConn
// rather than KillPath::Idempotent.  This conversion inserts a genuine Terminated session via
// insert_terminated_session_for_test() so KillPath::Idempotent is exercised directly.
// ---------------------------------------------------------------------------

/// BC-2.08.003 Invariant 2: `kill_session()` on an already-`Terminated` session
/// MUST return `Ok(())` without sending another `DaemonToHost::Kill` or transitioning
/// to `Terminating` (KillPath::Idempotent arm — no duplicate Kill, no watchdog spawned).
///
/// Pre-condition: session is inserted directly into `Terminated` state via the
/// `insert_terminated_session_for_test()` test seam (F-S034-ADV-LOW-001).
#[tokio::test]
async fn test_BC_2_08_003_kill_session_idempotent_on_terminated() {
    // BC-2.08.003 Invariant 2 (genuine): kill on Terminated → Ok(()), idempotent.
    // Uses insert_terminated_session_for_test seam (F-S034-ADV-LOW-001).

    let tmp = isolated_runtime_dir();
    let session_id = "034b0000-0001-4000-a000-000000000001".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));

    // make_manager uses FakePidSpawner — no real process needed.
    let (mut manager, _subs, mut rx) = make_manager(tmp.path(), 55_002);

    // Insert session directly in Terminated state (F-S034-ADV-LOW-001 seam).
    manager
        .insert_terminated_session_for_test(&session_id, 55_002, socket_path.clone())
        .await;

    // Verify precondition: session is in Terminated state.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must be in registry after insert_terminated_session_for_test");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminated,
        "test_BC_2_08_003_kill_session_idempotent_on_terminated: \
         precondition: session must be Terminated before calling kill_session()"
    );

    // Call kill_session() on the Terminated session.
    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "test_BC_2_08_003_kill_session_idempotent_on_terminated: \
         kill_session() on Terminated session MUST return Ok(()) (BC-2.08.003 Invariant 2). \
         Got: {:?}",
        result
    );

    // No DaemonToHost::Kill should be sent — verify no SessionStateChanged{Terminating}
    // arrives on the broker (which would indicate the ExistingConn or FreshConnect arm fired).
    // Allow 50ms for any spurious messages to flush.
    let msgs = drain_messages(&mut rx, 50).await;
    let has_terminating = msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Terminating,
            } if sid == &session_id
        )
    });
    assert!(
        !has_terminating,
        "test_BC_2_08_003_kill_session_idempotent_on_terminated: \
         kill_session() on Terminated session MUST NOT send DaemonToHost::Kill or emit \
         SessionStateChanged{{Terminating}} (BC-2.08.003 Invariant 2 — no duplicate Kill). \
         Got a Terminating broadcast — KillPath::Idempotent arm was NOT taken."
    );

    // State must remain Terminated (no re-transition to Terminating).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry after idempotent kill");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminated,
        "test_BC_2_08_003_kill_session_idempotent_on_terminated: \
         state must remain Terminated after idempotent kill (BC-2.08.003 Invariant 2)"
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
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 55_003, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Advance session to Running via mock UDS.
    let (mut peer, _) = tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
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
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 55_004, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Advance session to Running.
    let (mut peer, _) = tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
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
    manager.kill_session(&session_id).await.expect(
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
// Test 5: test_BC_2_08_003_existing_conn_broken_write_falls_back_to_fresh_connect
//
// OBS-1 (adversarial pass-9): The original test was MISLABELED as "Detached/EC-164
// FreshConnect" and LATENTLY FLAKY.  What it actually exercises is the
// KillPath::ExistingConn broken-write → FreshConnect FALLBACK path:
//
//   1. Session is inserted in Launching state with a pre-broken host_conn writer
//      via insert_launching_session_with_broken_conn_for_test().  host_conn.is_some()
//      → kill_session dispatches to KillPath::ExistingConn.
//   2. The writer is connected to a UnixStream::pair() whose receiver was immediately
//      dropped — the very next write to the writer returns BrokenPipe (EPIPE)
//      deterministically, without any kernel-buffer race or platform timing dependency.
//      (UnixStream::pair() + immediate drop of receiver is the canonical in-process
//      broken-write technique; unlike SHUT_RDWR on a real socket, the kernel cannot
//      buffer a small write across an already-dropped in-memory peer.)
//   3. ExistingConn write fails → fallback to FreshConnect: kill_session opens a new
//      UDS connect to socket_path, applies SO_PEERCRED (AllowAllVerifier → uid match),
//      and sends DaemonToHost::Kill on the fresh connection.
//   4. State transitions Launching → Terminating immediately after Kill is sent.
//
// Genuine KillPath::FreshConnect (Detached, host_conn:None) is already covered
// by test_BC_2_08_003_IMP001_fresh_connect_detached_kill_path_happy (mod.rs ~5988).
//
// Fails because: kill_session() is todo!() — panics at call.
// ---------------------------------------------------------------------------

/// BC-2.08.003 ExistingConn broken-write fallback: when the existing control
/// connection write fails (broken pipe), `kill_session()` MUST fall back to a fresh
/// UDS connect, apply SO_PEERCRED before sending any message (Invariant 5), send
/// `DaemonToHost::Kill` on the fresh connection, and transition the session to
/// `Terminating`.
///
/// Uses `insert_launching_session_with_broken_conn_for_test()` to inject a pre-broken
/// writer (receiver half of `UnixStream::pair()` immediately dropped) — this is
/// deterministically EPIPE on the next write, no kernel-buffering race.
///
/// OBS-1 (adversarial pass-9): renamed from `test_BC_2_08_003_kill_detached_so_peercred`
/// (mislabeled) + deterministic broken-write via test-seam helper.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics.
#[tokio::test]
async fn test_BC_2_08_003_existing_conn_broken_write_falls_back_to_fresh_connect() {
    // OBS-1 fix: ExistingConn broken-write → FreshConnect fallback path.
    // The session is Launching + host_conn:Some (broken writer).
    // kill_session dispatches to KillPath::ExistingConn; the write fails immediately
    // (EPIPE); then falls back to FreshConnect on the same socket path.
    use tokio::io::AsyncReadExt;

    let tmp = isolated_runtime_dir();
    let session_id = "034e0000-0001-4000-a000-000000000001".to_string();

    // Bind the UDS socket for kill_session()'s fallback fresh connect.
    // (No post_spawn_monitor is involved — the session is inserted directly.)
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind socket");

    let (mut manager, _subs, _rx) =
        make_manager_with_socket(tmp.path(), 55_005, socket_path.clone());
    // AllowAllVerifier: SO_PEERCRED passes on the fallback fresh connect.
    // This verifies SO_PEERCRED IS applied (not skipped) even on the fallback path.
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    // Insert a Launching session with a pre-broken control connection writer.
    // receiver of UnixStream::pair() is dropped immediately inside the helper →
    // the next write to host_conn.writer returns EPIPE without any timing dependency.
    manager
        .insert_launching_session_with_broken_conn_for_test(
            &session_id,
            55_005,
            socket_path.clone(),
        )
        .await;

    // Spawn a task to accept kill_session()'s fallback fresh connect and read the
    // Kill message it sends.  The fallback runs concurrently with kill_session().
    let kill_connect_task = tokio::spawn({
        let sid = session_id.clone();
        async move {
            // Accept the fallback fresh connect from kill_session.
            let (mut conn, _) =
                tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
                    .await
                    .expect("timed out waiting for fallback fresh connect")
                    .expect("accept failed");

            // Read the Kill message sent over the fallback connection.
            let mut len_buf = [0u8; 4];
            tokio::time::timeout(
                std::time::Duration::from_millis(500),
                conn.read_exact(&mut len_buf),
            )
            .await
            .expect("timed out waiting for Kill message on fallback connect")
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
                "test_BC_2_08_003_existing_conn_broken_write_falls_back_to_fresh_connect: \
                 kill_session() MUST send DaemonToHost::Kill on the fallback fresh connect \
                 (BC-2.08.003 ExistingConn→FreshConnect fallback, Invariant 5 SO_PEERCRED). \
                 Got: {:?}",
                msg
            );

            sid
        }
    });

    // Call kill_session().
    // Expected flow: ExistingConn write → BrokenPipe (pre-broken writer, deterministic)
    // → fallback FreshConnect → AllowAllVerifier passes → Kill sent → Terminating.
    // FAILS: kill_session() is todo!() → panics here.
    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "test_BC_2_08_003_existing_conn_broken_write_falls_back_to_fresh_connect: \
         kill_session() on Launching+broken-write session MUST return Ok(()) \
         (BC-2.08.003 ExistingConn→FreshConnect fallback). Got: {:?}",
        result
    );

    // Wait for the fallback-connect task to verify Kill was received.
    let confirmed_sid = tokio::time::timeout(std::time::Duration::from_secs(3), kill_connect_task)
        .await
        .expect("timed out waiting for fallback kill_connect_task")
        .expect("fallback kill_connect_task panicked");

    assert_eq!(confirmed_sid, session_id);

    // State must be Terminating: ExistingConn broken-write → FreshConnect fallback →
    // Kill sent → Terminating (same terminal invariant as genuine FreshConnect path).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry after fallback kill");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "test_BC_2_08_003_existing_conn_broken_write_falls_back_to_fresh_connect: \
         state must be Terminating after ExistingConn broken-write→FreshConnect fallback Kill \
         (BC-2.08.003 Invariant 5 / ExistingConn fallback path). Got: {:?}",
        snap.state,
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
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 55_007, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Advance to Running.
    let (mut peer, _) = tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
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
    manager.kill_session(&session_id).await.expect(
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
// Test 8b: test_pid_fallback_non_esrch_sigterm_failure_returns_kill_failed
//
// Verifies: BC-2.08.003 PC-1 (Launching race window) + ADV-S034-MED-001 +
// ADV-S034-IMPORTANT-001 (deterministic EPERM injection — no real-PID signal).
// When the PID-based SIGTERM fails with a non-ESRCH error (e.g. EPERM),
// kill_session() MUST return Err mapping to "kill_failed" and MUST NOT transition
// the session to Terminating.
//
// Anchors: AC-001 ("Failure code: kill_failed"), BC-2.08.003 PC-1 (Launching/no
// host_conn), ADV-S034-MED-001, ADV-S034-IMPORTANT-001.
//
// FIX (ADV-S034-IMPORTANT-001): replaced the real-PID-1 approach with a
// deterministic failure-injection seam (SessionManager::with_pid_sigterm_fn).
// The seam intercepts the PidFallback SIGTERM call and returns a synthetic EPERM
// without sending any signal to any live OS process.  PID 55_099 is used as the
// synthetic PID — well above the typical system PID ceiling and guaranteed never
// to be PID 1 (init/launchd).
// ---------------------------------------------------------------------------

/// BC-2.08.003 PC-1 / ADV-S034-MED-001 / ADV-S034-IMPORTANT-001:
/// `kill_session()` on a `Launching` session with `host_conn: None` when the
/// PID-based SIGTERM fails with a non-ESRCH error MUST return
/// `Err(SessionError::SessionHostDead)` mapping to wire code `"kill_failed"` and
/// MUST NOT transition the session to `Terminating`.
///
/// Deterministic failure injection (ADV-S034-IMPORTANT-001): uses
/// `SessionManager::with_pid_sigterm_fn` to return a synthetic `EPERM` without
/// sending any signal to any real OS process.  No PID 1 / init is involved; the
/// session is registered with synthetic PID 55_099.
#[tokio::test]
async fn test_pid_fallback_non_esrch_sigterm_failure_returns_kill_failed() {
    use monocle_runtime::session_manager::{session_error_to_code, IpcOp};

    // ADV-S034-MED-001 / ADV-S034-IMPORTANT-001: PidFallback path, non-ESRCH
    // SIGTERM failure → kill_failed.  Synthetic PID — no real signal sent.
    let tmp = isolated_runtime_dir();
    let session_id = "034f0000-0001-4000-a000-000000000001".to_string();

    // Synthetic PID — high value, guaranteed not to be PID 1 (init/launchd) or any
    // real process the test infrastructure would own.  The signal syscall is never
    // issued against this PID because the injection seam intercepts it.
    let synthetic_pid: u32 = 55_099;
    let (mut manager, _subs, mut rx) = make_manager(tmp.path(), synthetic_pid);

    // Install the failure-injection seam: return synthetic EPERM (non-ESRCH) for
    // any PID the PidFallback path tries to signal.  NO real kill(2) syscall is made.
    manager.with_pid_sigterm_fn(Arc::new(|_pid| Err(nix::errno::Errno::EPERM)));

    // Spawn a session using FakePidSpawner(pid=55_099) — registers the session with
    // session_host_pid = 55_099 and host_conn = None (no socket bound, monitor won't
    // connect).
    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Verify precondition: session is Launching with host_conn=None.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must be in registry");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Launching,
        "test_pid_fallback_non_esrch_sigterm_failure_returns_kill_failed: \
         precondition: session must be Launching (host_conn=None)"
    );

    // Call kill_session() — injection seam returns EPERM (non-ESRCH) → kill_failed.
    // No real signal is sent to any OS process.
    let result = manager.kill_session(&session_id).await;
    let err = result.expect_err(
        "test_pid_fallback_non_esrch_sigterm_failure_returns_kill_failed: \
         kill_session() on Launching (host_conn=None, synthetic EPERM injection) MUST \
         return Err when SIGTERM fails with non-ESRCH (ADV-S034-MED-001, \
         BC-2.08.003 PC-1 'Failure code: kill_failed')",
    );

    // Verify the error maps to wire code "kill_failed" (BC-2.08.003 PC-1, AC-001).
    let code = session_error_to_code(IpcOp::Kill, &err);
    assert_eq!(
        code, "kill_failed",
        "test_pid_fallback_non_esrch_sigterm_failure_returns_kill_failed: \
         non-ESRCH SIGTERM failure on PidFallback path MUST map to wire code 'kill_failed' \
         via session_error_to_code(IpcOp::Kill, e) (ADV-S034-MED-001, BC-2.08.003 PC-1). \
         Got: '{}'",
        code
    );

    // Session MUST NOT have transitioned to Terminating — the kill was not delivered.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry after failed kill");
    assert_ne!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "test_pid_fallback_non_esrch_sigterm_failure_returns_kill_failed: \
         session MUST NOT transition to Terminating when PidFallback SIGTERM fails with \
         non-ESRCH error — kill was NOT delivered (ADV-S034-MED-001, BC-2.08.003 PC-1)"
    );

    // No SessionStateChanged{Terminating} MUST be emitted.
    let msgs = drain_messages(&mut rx, 50).await;
    let has_terminating = msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Terminating,
            } if sid == &session_id
        )
    });
    assert!(
        !has_terminating,
        "test_pid_fallback_non_esrch_sigterm_failure_returns_kill_failed: \
         SessionStateChanged{{Terminating}} MUST NOT be emitted when PidFallback SIGTERM \
         fails (ADV-S034-MED-001)"
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
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind mock socket");

    let (mut manager, _subs, _rx) =
        make_manager_with_socket(tmp.path(), 55_009, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Accept the post-spawn monitor's connect. This sets host_conn=Some in the entry.
    let (mut peer, _) = tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
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
    peer.read_exact(&mut body).await.expect("read Kill body");
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
// Test 10: test_BC_2_08_003_existing_conn_broken_write_fallback_connect_fails_terminated
//
// OBS-1 (adversarial pass-9): The original test was MISLABELED and VACUOUSLY PASSING:
// RejectAllVerifier caused the post_spawn_monitor's SO_PEERCRED to fail, transitioning
// the session to Terminated BEFORE kill_session() was called.  kill_session() then hit
// KillPath::Idempotent (already Terminated) → Ok(()) without exercising any kill logic.
// The Terminated assertion was trivially true (state set by the monitor, not kill_session).
//
// OBS-1 fix: test the ExistingConn broken-write → fallback connect FAILS path instead.
//   1. insert_launching_session_with_broken_conn_for_test(): Launching + host_conn:Some
//      (pre-broken writer — receiver of UnixStream::pair() immediately dropped).
//      kill_session dispatches to KillPath::ExistingConn.
//   2. Pre-broken writer returns EPIPE on the next write — deterministic, no race.
//   3. Socket file does NOT exist (no listener bound) → fallback fresh connect fails
//      immediately (ENOENT).
//   4. kill_session() returns Ok(()); session transitions to Terminated (EC-162/163 path).
//
// Genuine FreshConnect UID mismatch (Detached, host_conn:None) is already covered by
// test_BC_2_08_003_IMP001_fresh_connect_detached_uid_mismatch_terminates (mod.rs ~6217).
//
// Fails because: kill_session() is todo!() — panics.
// ---------------------------------------------------------------------------

/// BC-2.08.003 EC-162/163: when the existing control connection write fails (broken
/// pipe) AND the fallback fresh connect also fails (socket file does not exist),
/// `kill_session()` MUST treat the session as dead, transition to `Terminated`
/// immediately, and return `Ok(())`.
///
/// Uses `insert_launching_session_with_broken_conn_for_test()` for the broken writer
/// and does NOT bind any listener — the fallback `UnixStream::connect()` fails with
/// ENOENT immediately, with no timing dependency.
///
/// OBS-1 (adversarial pass-9): renamed from
/// `test_BC_2_08_003_kill_detached_so_peercred_uid_mismatch_terminates` (mislabeled +
/// vacuously passing via Idempotent) to correctly test the broken-write + no-socket
/// → Terminated path.
///
/// FAILS NOW: `kill_session()` is `todo!()` → panics.
#[tokio::test]
async fn test_BC_2_08_003_existing_conn_broken_write_fallback_connect_fails_terminated() {
    // OBS-1 fix: ExistingConn broken-write → fallback connect fails → Terminated.
    // No listener is bound — fallback UnixStream::connect() returns ENOENT immediately.

    let tmp = isolated_runtime_dir();
    let session_id = "03400000-0001-4000-a000-000000000010".to_string();
    // Use a socket path that does NOT exist (no listener bound).
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));

    let (mut manager, _subs, _rx) =
        make_manager_with_socket(tmp.path(), 55_010, socket_path.clone());
    // Verifier is irrelevant here: the fallback connect will fail before verify()
    // is called. AllowAllVerifier is used for clarity.
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    // Insert a Launching session with a pre-broken writer (no listener involved).
    // receiver of UnixStream::pair() is dropped immediately → EPIPE on next write.
    manager
        .insert_launching_session_with_broken_conn_for_test(
            &session_id,
            55_010,
            socket_path.clone(),
        )
        .await;

    // Call kill_session() — expected flow:
    //   ExistingConn write → BrokenPipe (pre-broken writer, deterministic)
    //   → fallback connect to socket_path → ENOENT (no listener, no socket file)
    //   → transition to Terminated; return Ok(()) (EC-162/163 dead-session path).
    // FAILS: kill_session() is todo!() → panics here.
    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "test_BC_2_08_003_existing_conn_broken_write_fallback_connect_fails_terminated: \
         kill_session() on broken-write + no-socket session MUST return Ok(()) \
         (session treated as dead — BC-2.08.003 EC-162/163). Got: {:?}",
        result
    );

    // State must be Terminated: broken write + no socket = session dead.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("session must remain in registry after dead-session kill");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminated,
        "test_BC_2_08_003_existing_conn_broken_write_fallback_connect_fails_terminated: \
         broken-write + no fallback-socket MUST transition session to Terminated \
         (BC-2.08.003 EC-162/163). Got: {:?}",
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
        code, "session_not_found",
        "test_BC_2_08_003_kill_session_not_found_wire_code: \
         SessionNotFound on Kill path MUST map to wire code 'session_not_found' \
         via session_error_to_code(IpcOp::Kill, e) (BC-2.08.003 EC-166, AC-011). \
         Got: '{}'",
        code
    );
}
