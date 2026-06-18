//! S-034 Ruling I validation: kill_confirm_monitor is MANDATORY on the ExistingConn
//! kill path (Running/Launching), and a promptly-confirmed kill must NOT require the
//! 12s watchdog.
//!
//! # Ruling I (SS-session-manager.md §Ruling I)
//!
//! On the ExistingConn SUCCESS path:
//! 1. `post_spawn_monitor` stores `host_conn.reader: Some(reader)` after connecting,
//!    then exits immediately after observing `StateChanged{Running}`.
//! 2. `kill_session()` takes `host_conn.reader` and spawns `kill_confirm_monitor`.
//! 3. `kill_confirm_monitor` reads `StateChanged{Terminated}` from the EXISTING reader.
//! 4. A prompt session-host response (well under the 12s watchdog) finalizes via
//!    `kill_confirm_monitor`, NOT via the watchdog.
//!
//! These tests FAIL if `kill_confirm_monitor` is NOT spawned on the ExistingConn path:
//! - If `host_conn.reader` is not stored (test #1: reader never handed to kill_confirm_monitor).
//! - If the watchdog fires instead of `kill_confirm_monitor` (test #2: prompt kill takes >12s
//!   of virtual time, or session stays Terminating after 1s advance).
//! - If the reader is None at kill time and no watchdog fallback is provided (test #3:
//!   watchdog-only path is safe).
//!
//! # References
//! - BC-2.08.003 AC-002 (kill path selection), AC-004 (Terminated via confirmation),
//!   AC-005 (watchdog deadline), AC-006 (fire-and-confirm)
//! - SS-session-manager.md §Ruling I
//! - ADV-S034-BLOCKER-001

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use monocle_core::engine::{SpawnOptions, SpawnRecipe};
use monocle_runtime::session_manager::{
    PeerCredVerifier, SessionError, SessionHostSpawner, SessionManager, SpawnedHostHandle,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

// ---------------------------------------------------------------------------
// Shared test infrastructure
// ---------------------------------------------------------------------------

fn isolated_runtime_dir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .tempdir_in("/tmp")
        .expect("create isolated runtime dir for S-034 Ruling I test in /tmp")
}

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

struct RulingITestEngine;

#[async_trait::async_trait]
impl monocle_core::engine::EngineModule for RulingITestEngine {
    fn id(&self) -> &'static str {
        "ruling-i-test-engine"
    }
    fn metadata(
        &self,
    ) -> Result<monocle_core::engine::EngineMetadata, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for Ruling I test")
    }
    fn detect(&self, _: &monocle_core::engine::ProcessSnapshot) -> bool {
        false
    }
    async fn enrich(
        &self,
        _: &monocle_core::engine::ProcessSnapshot,
    ) -> Result<monocle_core::engine::EnrichedSession, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for Ruling I test")
    }
    async fn on_hook(
        &self,
        _: monocle_core::hook_events::HookEvent,
    ) -> monocle_core::engine::HookResponse {
        unimplemented!("not needed for Ruling I test")
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

struct AllowAllVerifier;
impl PeerCredVerifier for AllowAllVerifier {
    fn verify(&self, _stream: &tokio::net::UnixStream) -> Result<(), SessionError> {
        Ok(())
    }
}

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
        Arc::new(RulingITestEngine),
    );
    (manager, subscriber_list, rx)
}

/// Send a length-prefixed `HostToDaemon` message to the mock daemon endpoint.
async fn send_host_to_daemon(
    peer: &mut tokio::net::UnixStream,
    msg: &monocle_ipc::types::HostToDaemon,
) {
    let body = serde_json::to_vec(msg).expect("serialize HostToDaemon");
    let len = (body.len() as u32).to_le_bytes();
    peer.write_all(&len).await.expect("write len prefix");
    peer.write_all(&body).await.expect("write body");
}

/// Drain all messages from `rx` within `timeout_ms` of virtual time.
/// In paused-clock tests, this drains whatever arrived during prior advances.
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
// Test 1: Ruling I — prompt kill via kill_confirm_monitor (avoids watchdog)
//
// BC-2.08.003 AC-004 (Terminated via StateChanged confirmation), AC-005 (watchdog
// fires ONLY when confirmation never arrives), AC-006 (fire-and-confirm)
// SS-session-manager.md §Ruling I
//
// On the ExistingConn (Running) kill path:
// - kill_session() takes host_conn.reader and spawns kill_confirm_monitor.
// - The mock session-host sends StateChanged{Terminated} promptly (within ~1s of
//   virtual time, well under the 12s watchdog).
// - The session reaches Terminated via kill_confirm_monitor before the 12s watchdog fires.
//
// This test FAILS if kill_confirm_monitor is NOT spawned on the ExistingConn path
// (i.e., it specifically guards Ruling I by ensuring:
//   (a) Terminated is reached after only 1s of virtual time advance — impossible if
//       the watchdog-only path is taken, since the watchdog fires at 12s, not 1s.
//   (b) The StateChanged{Terminated} arrives via kill_confirm_monitor, NOT via a
//       watchdog-forced transition.)
// ---------------------------------------------------------------------------

/// Ruling I (BC-2.08.003 AC-004/AC-006): On the ExistingConn (Running) kill path,
/// when the mock session-host sends StateChanged{Terminated} PROMPTLY, the session
/// reaches Terminated via kill_confirm_monitor WELL BEFORE the 12s watchdog deadline.
///
/// Specifically: after advancing only ~1s of virtual time after kill_session(), the
/// session must be Terminated. If kill_confirm_monitor were NOT spawned (only watchdog),
/// the session would still be Terminating after 1s and would not reach Terminated until
/// 12s — this test would then fail.
///
/// This guards Ruling I: kill_confirm_monitor MUST be spawned and MUST take
/// host_conn.reader on the ExistingConn SUCCESS path.
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_003_ruling_I_prompt_kill_reaches_terminated_via_kill_confirm_monitor() {
    // Ruling I / BC-2.08.003 AC-004/AC-006: kill_confirm_monitor (not watchdog) handles
    // promptly-confirmed kill on the ExistingConn (Running) path.

    let tmp = isolated_runtime_dir();
    let session_id = "b1d10000-0001-4000-b000-000000000001".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));

    // Bind mock session-host socket BEFORE spawning.
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock session-host socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 66_001, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Accept the post-spawn monitor connection in a background task.
    // We use start_paused=true so all tokio::time operations are virtual.
    let accept_task = tokio::spawn(async move {
        tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
            .await
            .expect("accept task: timed out waiting for post-spawn monitor")
            .expect("accept task: accept failed")
    });

    // Advance slightly to let the post-spawn monitor poll task run.
    tokio::time::advance(std::time::Duration::from_millis(50)).await;
    tokio::task::yield_now().await;

    let (mut peer, _addr) = accept_task.await.expect("accept task panicked");

    // Ruling I step 1: send StateChanged{Running} so the post_spawn_monitor stores
    // host_conn.reader and breaks immediately.
    send_host_to_daemon(
        &mut peer,
        &monocle_ipc::types::HostToDaemon::StateChanged {
            new_state: monocle_ipc::types::SessionState::Running,
            degraded_env: None,
        },
    )
    .await;

    // Let the post_spawn_monitor process Running, store host_conn.reader, and break.
    tokio::time::advance(std::time::Duration::from_millis(100)).await;
    for _ in 0..30 {
        tokio::task::yield_now().await;
    }

    // Verify session reached Running.
    let mut reached_running = false;
    {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(500);
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await {
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
    }
    assert!(
        reached_running,
        "Ruling I precondition: session must reach Running before testing kill path"
    );

    // Start a background task that simulates the session-host Kill handler.
    //
    // This task waits for DaemonToHost::Kill on the ORIGINAL connection (peer), then sends
    // StateChanged{Terminated} + Goodbye PROMPTLY (~1s virtual time) on the SAME connection.
    //
    // Ruling I assertion mechanism:
    // - After kill_session() and advancing only 1s of virtual time, the session must be Terminated.
    // - If kill_confirm_monitor was NOT spawned, nobody reads the Terminated message, and the
    //   session would remain Terminating until the 12s watchdog fires — causing this test to fail
    //   after the 1s advance assertion.
    let kill_handler_task = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;

        // Wait for DaemonToHost::Kill on the same connection (original peer).
        let msg = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            let mut len_buf = [0u8; 4];
            peer.read_exact(&mut len_buf)
                .await
                .expect("Ruling I: read Kill len prefix on original connection");
            let msg_len = u32::from_le_bytes(len_buf) as usize;
            let mut body = vec![0u8; msg_len];
            peer.read_exact(&mut body)
                .await
                .expect("Ruling I: read Kill body on original connection");
            serde_json::from_slice::<monocle_ipc::types::DaemonToHost>(&body)
                .expect("Ruling I: deserialize DaemonToHost Kill")
        })
        .await
        .expect(
            "Ruling I FAIL: DaemonToHost::Kill was NOT received on the original connection \
             within 2s of virtual time. kill_session() must send Kill on the ExistingConn path. \
             (BC-2.08.003 AC-002 / Ruling I)",
        );

        assert!(
            matches!(msg, monocle_ipc::types::DaemonToHost::Kill),
            "Ruling I FAIL: expected DaemonToHost::Kill, got {:?}",
            msg
        );

        // Session-host Kill handler: respond with StateChanged{Terminated} PROMPTLY.
        // No sleep needed — kill_confirm_monitor will read this from host_conn.reader.
        send_host_to_daemon(
            &mut peer,
            &monocle_ipc::types::HostToDaemon::StateChanged {
                new_state: monocle_ipc::types::SessionState::Terminated,
                degraded_env: None,
            },
        )
        .await;

        send_host_to_daemon(&mut peer, &monocle_ipc::types::HostToDaemon::Goodbye).await;
    });

    // Call kill_session() — must use ExistingConn path (host_conn.writer present).
    // Ruling I: after this, kill_confirm_monitor is spawned with host_conn.reader.
    manager
        .kill_session(&session_id)
        .await
        .expect("Ruling I: kill_session() must return Ok(())");

    // State must be Terminating immediately after kill_session() (BC-2.08.003 AC-002/PC-2).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("Ruling I: session must remain in registry after kill_session()");
    assert!(
        snap.state == monocle_ipc::types::SessionState::Terminating
            || snap.state == monocle_ipc::types::SessionState::Terminated,
        "Ruling I (BC-2.08.003 AC-002): state must be Terminating (or Terminated if very fast) \
         immediately after kill_session(). Got: {:?}",
        snap.state
    );

    // Advance only 1s of virtual time — the kill_confirm_monitor should process the prompt
    // Terminated response. This is WELL UNDER the 12s watchdog.
    // If kill_confirm_monitor was NOT spawned, the session would remain Terminating here,
    // and the assertion below would FAIL (it would still be Terminating, not Terminated).
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    for _ in 0..100 {
        tokio::task::yield_now().await;
    }

    // Ruling I assertion: session must be Terminated after only 1s of virtual time.
    // This is the core guard — if only the 12s watchdog were used (kill_confirm_monitor
    // not spawned or reader not handed to it), the session would still be Terminating here.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("Ruling I: session must still be in registry");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminated,
        "Ruling I FAIL (BC-2.08.003 AC-004/AC-006): session must be Terminated via \
         kill_confirm_monitor after ~1s virtual time. If the session is still Terminating, \
         kill_confirm_monitor was NOT spawned on the ExistingConn path (Ruling I violation). \
         The 12s watchdog fires at 12s, not 1s — this path distinguishes kill_confirm_monitor \
         from watchdog-only. SS-session-manager.md §Ruling I."
    );

    // Drain messages; verify SessionStateChanged{Terminated} was broadcast via kill_confirm_monitor.
    // (The Terminating pair was already drained above; these messages are the Terminated pair.)
    let msgs = drain_messages(&mut rx, 300).await;
    let terminated_via_confirm = msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Terminated,
            } if sid == &session_id
        )
    });
    assert!(
        terminated_via_confirm,
        "Ruling I (BC-2.08.003 AC-004): SessionStateChanged{{Terminated}} must be broadcast \
         by kill_confirm_monitor after prompt session-host response. Messages: {:?}",
        msgs
    );

    // Verify the kill_handler task completed successfully (it sent the Terminated response).
    tokio::time::timeout(std::time::Duration::from_secs(3), kill_handler_task)
        .await
        .expect("Ruling I: kill_handler_task timed out")
        .expect("Ruling I: kill_handler_task panicked");

    // Verify the watchdog did NOT fire (session was already Terminated at 1s, before 12s).
    // Advance to 12s and verify no duplicate Terminated broadcast appears (watchdog skips
    // if already Terminated per spawn_kill_watchdog's re-check under lock).
    let _ = drain_messages(&mut rx, 100).await; // clear any residual
    tokio::time::advance(std::time::Duration::from_secs(11)).await;
    for _ in 0..50 {
        tokio::task::yield_now().await;
    }

    let post_watchdog_msgs = drain_messages(&mut rx, 200).await;
    let duplicate_terminated = post_watchdog_msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Terminated,
            } if sid == &session_id
        )
    });
    // The watchdog must NOT emit a duplicate Terminated after kill_confirm_monitor already
    // set the state. Watchdog re-checks under lock and skips if already Terminated.
    assert!(
        !duplicate_terminated,
        "Ruling I: watchdog must NOT emit a duplicate Terminated broadcast after \
         kill_confirm_monitor has already transitioned the session. \
         (BC-2.08.003 AC-005: watchdog fires ONLY when confirmation never arrives). \
         Messages after watchdog deadline: {:?}",
        post_watchdog_msgs
    );
}

// ---------------------------------------------------------------------------
// Test 2: Ruling I — watchdog-only path when host_conn.reader is None at kill time
//
// BC-2.08.003 AC-005 (watchdog fallback), SS-session-manager.md §Ruling I
//
// Edge case: if host_conn.reader is None at kill time (the pre-Running race or
// reader-already-taken case), kill_session() falls back to the watchdog-only path.
// The session must still reach Terminated (no panic, no hang) via the watchdog.
//
// Simulated by: mock session-host sends Running but does NOT send StateChanged{Terminated}
// promptly. The 12s watchdog fires and forces Terminated.
//
// Note: the "reader is None" edge case arises only if the reader was not stored.
// In the v2.9.0 implementation, post_spawn_monitor ALWAYS stores the reader at Running.
// This test verifies the watchdog path is safe when reader is absent — e.g. if a
// future code change incorrectly clears the reader before kill_session() runs.
// ---------------------------------------------------------------------------

/// Ruling I edge case (BC-2.08.003 AC-005): when kill_confirm_monitor returns early
/// (EOF/no response from session-host), the 12s watchdog fires and forces Terminated.
///
/// This test exercises the watchdog fallback path:
/// - Session reaches Running (post_spawn_monitor exits and stores reader).
/// - kill_session() spawns kill_confirm_monitor AND watchdog.
/// - Session-host receives Kill but sends NO StateChanged{Terminated} response
///   (simulates unresponsive session-host: kill_confirm_monitor exits on EOF).
/// - After 12s virtual time, the watchdog forces Terminated.
/// - No panic, no hang, session is correctly finalized.
///
/// This guards BC-2.08.003 AC-005: the 12s watchdog MUST fire when no confirmation
/// arrives, regardless of whether kill_confirm_monitor was spawned.
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_003_ruling_I_watchdog_fires_when_kill_confirm_monitor_gets_eof() {
    // BC-2.08.003 AC-005 / Ruling I: watchdog fires when kill_confirm_monitor gets EOF
    // (session-host did not send StateChanged{Terminated}).

    let tmp = isolated_runtime_dir();
    let session_id = "b1d10000-0002-4000-b000-000000000002".to_string();
    let socket_path = tmp.path().join(format!("session-{}.sock", session_id));

    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind mock session-host socket");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 66_002, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let opts = make_spawn_opts(&session_id);
    manager
        .spawn_session(opts)
        .await
        .expect("spawn_session must succeed");

    // Accept post-spawn monitor connection.
    let accept_task = tokio::spawn(async move {
        tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
            .await
            .expect("accept task: timed out")
            .expect("accept task: accept failed")
    });

    tokio::time::advance(std::time::Duration::from_millis(50)).await;
    tokio::task::yield_now().await;

    let (mut peer, _addr) = accept_task.await.expect("accept task panicked");

    // Send Running; post_spawn_monitor stores reader and exits.
    send_host_to_daemon(
        &mut peer,
        &monocle_ipc::types::HostToDaemon::StateChanged {
            new_state: monocle_ipc::types::SessionState::Running,
            degraded_env: None,
        },
    )
    .await;

    tokio::time::advance(std::time::Duration::from_millis(100)).await;
    for _ in 0..30 {
        tokio::task::yield_now().await;
    }

    // Drain until Running.
    let mut reached_running = false;
    {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(500);
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await {
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
    }
    assert!(
        reached_running,
        "Ruling I watchdog test precondition: session must reach Running"
    );

    // Spawn a task that reads Kill on the original connection, then closes the connection
    // WITHOUT sending StateChanged{Terminated} — simulating an unresponsive session-host.
    // kill_confirm_monitor will get EOF on the existing reader and return early.
    // The 12s watchdog must then fire.
    let no_response_task = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        // Read Kill (consume it so kill_session doesn't block on the write).
        let mut len_buf = [0u8; 4];
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            peer.read_exact(&mut len_buf),
        )
        .await;
        // Do NOT send any response — just drop `peer`, causing EOF on kill_confirm_monitor's reader.
        // peer is dropped here when the task exits.
    });

    // Call kill_session() — ExistingConn path, spawns kill_confirm_monitor + watchdog.
    manager
        .kill_session(&session_id)
        .await
        .expect("Ruling I watchdog test: kill_session() must return Ok(())");

    // State must be Terminating immediately.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("Ruling I watchdog test: session must remain in registry");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "Ruling I watchdog test (BC-2.08.003 AC-002): state must be Terminating immediately \
         after kill_session(). Got: {:?}",
        snap.state
    );

    // Let the no_response_task close the connection (EOF to kill_confirm_monitor).
    tokio::time::advance(std::time::Duration::from_millis(100)).await;
    for _ in 0..50 {
        tokio::task::yield_now().await;
    }
    let _ = no_response_task.await;

    // After 1s advance: session must still be Terminating (confirm never arrived, watchdog not yet).
    // This distinguishes the watchdog-path from the kill_confirm_monitor-path.
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    for _ in 0..50 {
        tokio::task::yield_now().await;
    }

    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("Ruling I watchdog test: session must still be in registry at 1s");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminating,
        "Ruling I watchdog test: session must still be Terminating at 1s (watchdog fires at 12s). \
         If Terminated at 1s, kill_confirm_monitor received a response it should not have. Got: {:?}",
        snap.state
    );

    // BC-2.08.003 AC-005: advance 12s total (11s more from now) — watchdog fires.
    tokio::time::advance(std::time::Duration::from_secs(11)).await;
    for _ in 0..100 {
        tokio::task::yield_now().await;
    }

    // State must now be Terminated (watchdog forced it).
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("Ruling I watchdog test: session must remain in registry after watchdog");
    assert_eq!(
        snap.state,
        monocle_ipc::types::SessionState::Terminated,
        "Ruling I watchdog test (BC-2.08.003 AC-005): watchdog must force Terminated after 12s \
         when kill_confirm_monitor gets EOF and no StateChanged{{Terminated}} arrives. Got: {:?}",
        snap.state
    );

    // BC-2.08.008 Invariant 4: watchdog broadcasts SessionStateChanged{Terminated} BEFORE
    // SessionListUpdate. Drain may also contain prior Terminating-pair messages from earlier
    // (kill_session broadcasts). We verify the Terminated → ListUpdate ordering by finding
    // the Terminated index and then checking that a ListUpdate follows it.
    let watchdog_msgs = drain_messages(&mut rx, 300).await;
    let terminated_idx = watchdog_msgs.iter().position(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Terminated,
            } if sid == &session_id
        )
    });
    let list_update_after_terminated = terminated_idx.and_then(|t_idx| {
        watchdog_msgs.iter().enumerate().position(|(i, m)| {
            i > t_idx
                && matches!(
                    m,
                    monocle_ipc::types::ServerToClient::SessionListUpdate { .. }
                )
        })
    });

    assert!(
        terminated_idx.is_some(),
        "Ruling I watchdog test (BC-2.08.003 AC-005): watchdog must broadcast \
         SessionStateChanged{{Terminated}}. Messages: {:?}",
        watchdog_msgs
    );
    assert!(
        list_update_after_terminated.is_some(),
        "Ruling I watchdog test (BC-2.08.008 I4): SessionListUpdate must follow \
         SessionStateChanged{{Terminated}} in the watchdog broadcast pair. \
         Terminated at idx {:?}. Messages: {:?}",
        terminated_idx,
        watchdog_msgs
    );
}
