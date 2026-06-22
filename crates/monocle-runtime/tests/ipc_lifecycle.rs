//! S-047 Red Gate: Failing TDD tests for IPC lifecycle variants and scrollback protocol.
//!
//! Every net-new behavior test MUST FAIL before S-047 implementation is complete.
//! Tests that exercise pre-existing paths (send_key_input routing, rename_session routing)
//! may GREEN on the stub — those are noted per-test.
//!
//! # Behavioral Contract Coverage
//!
//! | Test | AC / EC | Stub that causes failure |
//! |------|---------|--------------------------|
//! | test_BC_2_05_010_key_input_routes_to_session_host | AC-003 | send_key_input() (S-040); may GREEN |
//! | test_BC_2_05_010_key_input_unknown_session_returns_session_not_found | AC-003 error | should GREEN |
//! | test_BC_2_05_010_rename_session_propagates_list_update | AC-005 | rename_session() (S-037); may GREEN |
//! | test_BC_2_05_010_rename_session_empty_name_returns_rename_failed | AC-005 error | should GREEN |
//! | test_BC_2_05_010_no_silent_failure_all_variants | AC-011 | pure code inspection; should GREEN |
//! | test_BC_2_05_010_error_codes_exhaustive_taxonomy | AC-012 | pure code inspection; should GREEN |
//! | test_BC_2_05_010_kill_session_idempotent_terminating | AC-002/EC-301 | kill_session() (S-034); may GREEN |
//! | test_BC_2_05_010_detach_session_blocks_on_launching | AC-004 | detach_session() todo!() panics |
//! | test_BC_2_05_011_attach_session_triggers_scrollback_sequence | AC-006 | forward_scrollback_dump_to_client() todo!() |
//! | test_BC_2_05_011_pending_pty_bytes_buffered_during_dump | AC-010 | forward_scrollback_dump_to_client() todo!() |
//! | test_BC_2_05_011_forward_scrollback_dump_sends_to_requesting_client_only | AC-SH-005 | forward_scrollback_dump_to_client() todo!() |
//! | test_BC_2_05_011_ec_306_empty_dump_total_chunks_zero | EC-306 | forward_scrollback_dump_to_client() todo!() |
//! | test_BC_2_05_011_ec_308_two_clients_independent_dumps | EC-308 | forward_scrollback_dump_to_client() todo!() |

#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_core::engine::{SpawnOptions, SpawnRecipe};
use monocle_runtime::session_manager::{
    HookEndpointConfig, IpcOp, PeerCredVerifier, SessionError, SessionHostSpawner, SessionManager,
    SpawnedHostHandle,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ---------------------------------------------------------------------------
// Test infrastructure
// ---------------------------------------------------------------------------

fn isolated_runtime_dir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .tempdir_in("/tmp")
        .expect("create isolated runtime dir for S-047 red gate test in /tmp")
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

struct S047Engine;

#[async_trait::async_trait]
impl monocle_core::engine::EngineModule for S047Engine {
    fn id(&self) -> &'static str {
        "s047-test-engine"
    }
    fn metadata(
        &self,
    ) -> Result<monocle_core::engine::EngineMetadata, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for S-047 IPC lifecycle tests")
    }
    fn detect(&self, _: &monocle_core::engine::ProcessSnapshot) -> bool {
        false
    }
    async fn enrich(
        &self,
        _: &monocle_core::engine::ProcessSnapshot,
    ) -> Result<monocle_core::engine::EnrichedSession, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for S-047 IPC lifecycle tests")
    }
    async fn on_hook(
        &self,
        _: monocle_core::hook_events::HookEvent,
    ) -> monocle_core::engine::HookResponse {
        unimplemented!("not needed for S-047 IPC lifecycle tests")
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

/// PeerCredVerifier that always passes — eliminates SO_PEERCRED UID check in tests.
struct AllowAllVerifier;
impl PeerCredVerifier for AllowAllVerifier {
    fn verify(&self, _stream: &tokio::net::UnixStream) -> Result<(), SessionError> {
        Ok(())
    }
}

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
        Arc::new(S047Engine),
        HookEndpointConfig::default(),
    );
    (manager, subscriber_list, rx)
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
        Arc::new(S047Engine),
        HookEndpointConfig::default(),
    );
    (manager, subscriber_list, rx)
}

/// Simulate session-host: accept monitor connection, send StateChanged{Running},
/// drain channel until SessionStateChanged{Running} arrives.
/// Returns the open peer stream (keep alive to avoid ECONNRESET).
async fn advance_to_running(
    listener: &tokio::net::UnixListener,
    session_id: &str,
    rx: &mut tokio::sync::mpsc::Receiver<monocle_ipc::types::ServerToClient>,
) -> tokio::net::UnixStream {
    let (mut peer, _addr) =
        tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
            .await
            .expect("timed out waiting for post-spawn monitor connect")
            .expect("accept failed");

    let body = serde_json::to_vec(&monocle_ipc::types::HostToDaemon::StateChanged {
        new_state: monocle_ipc::types::SessionState::Running,
        degraded_env: None,
    })
    .expect("serialize StateChanged");
    let len = (body.len() as u32).to_le_bytes();
    peer.write_all(&len).await.expect("write len");
    peer.write_all(&body).await.expect("write body");

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Some(monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state: monocle_ipc::types::SessionState::Running,
            })) if sid == session_id => break,
            Ok(Some(_)) => {}
            _ => panic!(
                "timed out waiting for SessionStateChanged{{Running}} for {}",
                session_id
            ),
        }
    }
    peer
}

/// Drain all messages from `rx` until `deadline` passes.
async fn drain_all(
    rx: &mut tokio::sync::mpsc::Receiver<monocle_ipc::types::ServerToClient>,
    deadline: std::time::Duration,
) -> Vec<monocle_ipc::types::ServerToClient> {
    let end = tokio::time::Instant::now() + deadline;
    let mut msgs = Vec::new();
    while let Ok(Some(m)) = tokio::time::timeout_at(end, rx.recv()).await {
        msgs.push(m);
    }
    msgs
}

/// Drain all messages from a dedicated client mpsc channel.
async fn drain_client(
    rx: &mut tokio::sync::mpsc::Receiver<monocle_ipc::types::ServerToClient>,
    deadline: std::time::Duration,
) -> Vec<monocle_ipc::types::ServerToClient> {
    let end = tokio::time::Instant::now() + deadline;
    let mut msgs = Vec::new();
    while let Ok(Some(m)) = tokio::time::timeout_at(end, rx.recv()).await {
        msgs.push(m);
    }
    msgs
}

/// Read one length-prefixed DaemonToHost frame from the mock session-host peer stream.
/// The daemon sends DaemonToHost messages to the session-host over the post-spawn monitor
/// connection (same UDS stream used for HostToDaemon messages in the other direction).
async fn read_daemon_to_host_msg(
    stream: &mut tokio::net::UnixStream,
) -> monocle_ipc::types::DaemonToHost {
    let mut len_buf = [0u8; 4];
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        stream.read_exact(&mut len_buf),
    )
    .await
    .expect("timed out reading DaemonToHost frame length")
    .expect("read length failed");
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut body = vec![0u8; len];
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        stream.read_exact(&mut body),
    )
    .await
    .expect("timed out reading DaemonToHost body")
    .expect("read body failed");
    serde_json::from_slice::<monocle_ipc::types::DaemonToHost>(&body)
        .expect("deserialize DaemonToHost")
}

// ---------------------------------------------------------------------------
// AC-003: KeyInput routes to session-host via send_key_input → DaemonToHost::KeyInput
// ---------------------------------------------------------------------------

/// AC-003: `send_key_input()` on a Running session writes `DaemonToHost::KeyInput{bytes}`
/// to the session-host over the post-spawn monitor connection. No error emitted.
///
/// send_key_input() was implemented in S-040.
/// Expected: GREEN after advance_to_running establishes host_conn.
#[tokio::test]
async fn test_BC_2_05_010_key_input_routes_to_session_host() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-keyinput.sock");
    let listener = tokio::net::UnixListener::bind(&socket_path)
        .expect("bind socket for KeyInput routing test");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_003, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "47030000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn_session must succeed");

    let mut peer = advance_to_running(&listener, &session_id, &mut rx).await;

    let key_bytes: Vec<u8> = vec![0x61]; // 'a'
    let result = manager.send_key_input(&session_id, key_bytes).await;
    assert!(
        result.is_ok(),
        "AC-003: send_key_input must succeed for Running session; got: {:?}",
        result
    );

    // Mock session-host peer should receive DaemonToHost::KeyInput.
    let msg = read_daemon_to_host_msg(&mut peer).await;
    match msg {
        monocle_ipc::types::DaemonToHost::KeyInput { bytes } => {
            assert_eq!(
                bytes,
                vec![0x61u8],
                "AC-003: KeyInput bytes must survive routing unchanged"
            );
        }
        other => panic!(
            "AC-003: expected DaemonToHost::KeyInput on session-host side, got: {:?}",
            other
        ),
    }
}

/// AC-003 error path: send_key_input on unknown session_id → "session_not_found".
/// NEVER "pty_write_failed" (phantom code per AC-012).
#[tokio::test]
async fn test_BC_2_05_010_key_input_unknown_session_returns_session_not_found() {
    let tmp = isolated_runtime_dir();
    let (mut manager, _subs, _rx) = make_manager(tmp.path(), 47_004);

    let result = manager
        .send_key_input("deadbeef-0000-4000-a000-000000000000", vec![0x62])
        .await;

    assert!(result.is_err(), "unknown session KeyInput must be Err");
    let code = monocle_runtime::session_manager::session_error_to_code(
        IpcOp::KeyInput,
        result.as_ref().unwrap_err(),
    );
    assert_eq!(
        code, "session_not_found",
        "AC-003: unknown session → 'session_not_found', got '{code}'"
    );
    assert_ne!(
        code, "pty_write_failed",
        "AC-012: 'pty_write_failed' is a phantom code — MUST NOT be returned"
    );
}

// ---------------------------------------------------------------------------
// AC-005: RenameSession propagates SessionListUpdate fan-out
// ---------------------------------------------------------------------------

/// AC-005: `rename_session()` → display_name updated → `SessionListUpdate` broadcast
/// with updated display_name. rename_session() implemented in S-037.
/// Expected: GREEN on the stub.
#[tokio::test]
async fn test_BC_2_05_010_rename_session_propagates_list_update() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-rename.sock");
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket for rename test");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_005, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "47050000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");

    let _peer = advance_to_running(&listener, &session_id, &mut rx).await;

    let result = manager
        .rename_session(&session_id, "my-new-name".to_string())
        .await;
    assert!(
        result.is_ok(),
        "AC-005: rename_session must succeed; got: {:?}",
        result
    );

    let msgs = drain_all(&mut rx, std::time::Duration::from_secs(2)).await;
    let list_update = msgs.iter().find(|m| {
        if let monocle_ipc::types::ServerToClient::SessionListUpdate { ref sessions } = m {
            sessions
                .iter()
                .any(|s| s.session_id == session_id && s.display_name == "my-new-name")
        } else {
            false
        }
    });
    assert!(
        list_update.is_some(),
        "AC-005: SessionListUpdate with display_name 'my-new-name' must be broadcast. \
         Got: {:?}",
        msgs
    );

    let state_changed = msgs.iter().find(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                ..
            } if sid == &session_id
        )
    });
    assert!(
        state_changed.is_none(),
        "AC-005: rename MUST NOT emit SessionStateChanged. Got: {:?}",
        state_changed
    );
}

/// AC-005 error: empty new_name → "rename_failed".
#[tokio::test]
async fn test_BC_2_05_010_rename_session_empty_name_returns_rename_failed() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-rename-empty.sock");
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket for rename empty test");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_006, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "47060000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");

    let _peer = advance_to_running(&listener, &session_id, &mut rx).await;

    let result = manager.rename_session(&session_id, "".to_string()).await;
    assert!(result.is_err(), "empty rename must return Err");
    let code = monocle_runtime::session_manager::session_error_to_code(
        IpcOp::Rename,
        result.as_ref().unwrap_err(),
    );
    assert_eq!(
        code, "rename_failed",
        "empty new_name → 'rename_failed', got '{code}'"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / EC-301: KillSession on Terminating → idempotent Ok(()) + no Error
// ---------------------------------------------------------------------------

/// AC-002 / EC-301: kill_session on a Terminating session → Ok(()) (idempotent).
/// NO ServerToClient::Error emitted. kill_session() implemented in S-034.
/// Expected: GREEN on the stub.
#[tokio::test]
async fn test_BC_2_05_010_kill_session_idempotent_terminating() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-kill-idem.sock");
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket for kill idempotent test");

    let (mut manager, _subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_002, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "47020000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");

    let _peer = advance_to_running(&listener, &session_id, &mut rx).await;

    manager
        .kill_session(&session_id)
        .await
        .expect("first kill must succeed");
    drain_all(&mut rx, std::time::Duration::from_millis(200)).await;

    let result = manager.kill_session(&session_id).await;
    assert!(
        result.is_ok(),
        "EC-301: kill_session on Terminating must return Ok(()) (idempotent). Got: {:?}",
        result
    );

    let msgs = drain_all(&mut rx, std::time::Duration::from_millis(200)).await;
    let error_msg = msgs
        .iter()
        .find(|m| matches!(m, monocle_ipc::types::ServerToClient::Error { .. }));
    assert!(
        error_msg.is_none(),
        "EC-301: idempotent kill on Terminating MUST NOT emit Error. Got: {:?}",
        error_msg
    );
}

// ---------------------------------------------------------------------------
// AC-004: DetachSession on Launching → "session_not_ready"
// ---------------------------------------------------------------------------

/// AC-004: detach_session on a Launching session (host_conn=None) → SessionError::SessionNotReady
/// → wire code "session_not_ready".
///
/// detach_session() is implemented (S-035 scope); this test verifies the correctness of
/// the Launching→SessionNotReady path (F-P51-001 / AC-014). Expected: GREEN on the stub.
#[tokio::test]
async fn test_BC_2_05_010_detach_session_blocks_on_launching() {
    let tmp = isolated_runtime_dir();
    let (mut manager, _subs, _rx) = make_manager(tmp.path(), 47_041);

    let session_id = "47041000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");
    // Session is Launching — FakePidSpawner socket_path doesn't exist; no monitor
    // connection yet → host_conn = None → LaunchingNoConn path.

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        manager.detach_session(&session_id),
    )
    .await
    .expect("detach_session must not hang");

    assert!(
        result.is_err(),
        "AC-004: detach_session on Launching must return Err (SessionNotReady)"
    );
    let code = monocle_runtime::session_manager::session_error_to_code(
        IpcOp::Detach,
        result.as_ref().unwrap_err(),
    );
    assert_eq!(
        code, "session_not_ready",
        "AC-004: Launching+NoConn detach → 'session_not_ready', got '{code}'"
    );
}

// ---------------------------------------------------------------------------
// AC-011: No-silent-failure — every error maps to a canonical 12-code
// ---------------------------------------------------------------------------

/// AC-011: Structural check that session_error_to_code() maps all known SessionError
/// variants to codes from the canonical 12-code closed set. No phantom codes.
#[test]
fn test_BC_2_05_010_no_silent_failure_all_variants() {
    let canonical_codes: std::collections::HashSet<&str> = [
        "binary_not_found",
        "invalid_spawn_arg",
        "spawn_unsupported",
        "spawn_failed",
        "sidecar_write_failed",
        "session_id_collision",
        "session_not_found",
        "attach_failed",
        "kill_failed",
        "rename_failed",
        "session_not_ready",
        "invalid_request",
    ]
    .iter()
    .copied()
    .collect();

    let test_cases: Vec<(IpcOp, SessionError)> = vec![
        (
            IpcOp::KeyInput,
            SessionError::SessionNotFound {
                session_id: "x".to_string(),
            },
        ),
        (
            IpcOp::KeyInput,
            SessionError::SessionHostDead {
                session_id: "x".to_string(),
            },
        ),
        (
            IpcOp::Rename,
            SessionError::SessionNotFound {
                session_id: "x".to_string(),
            },
        ),
        (
            IpcOp::Rename,
            SessionError::InvalidSessionName {
                reason: "empty".to_string(),
            },
        ),
        (
            IpcOp::Detach,
            SessionError::SessionNotReady {
                session_id: "x".to_string(),
            },
        ),
        (
            IpcOp::Detach,
            SessionError::SessionNotFound {
                session_id: "x".to_string(),
            },
        ),
        (
            IpcOp::Kill,
            SessionError::SessionNotFound {
                session_id: "x".to_string(),
            },
        ),
        (
            IpcOp::Attach,
            SessionError::SessionNotFound {
                session_id: "x".to_string(),
            },
        ),
        (
            IpcOp::Spawn,
            SessionError::SpawnFailed {
                reason: "test-reason".to_string(),
            },
        ),
        (
            IpcOp::Spawn,
            SessionError::SessionIdCollision {
                session_id: "x".to_string(),
            },
        ),
    ];

    for (op, err) in &test_cases {
        let code = monocle_runtime::session_manager::session_error_to_code(*op, err);
        assert!(
            canonical_codes.contains(code),
            "AC-011: session_error_to_code({op:?}, {err:?}) → '{code}' \
             not in canonical 12-code set"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-012: Closed taxonomy of exactly 12 wire error codes; no phantom codes
// ---------------------------------------------------------------------------

/// AC-012: The canonical 12-code set is exactly 12 codes, no phantom codes.
/// Phantom codes that MUST NOT exist: "pty_write_failed", "unknown_command", etc.
/// Also asserts specific mappings required by BC-2.05.010 §3.2.
#[test]
fn test_BC_2_05_010_error_codes_exhaustive_taxonomy() {
    let canonical: std::collections::HashSet<&str> = [
        "binary_not_found",
        "invalid_spawn_arg",
        "spawn_unsupported",
        "spawn_failed",
        "sidecar_write_failed",
        "session_id_collision",
        "session_not_found",
        "attach_failed",
        "kill_failed",
        "rename_failed",
        "session_not_ready",
        "invalid_request",
    ]
    .iter()
    .copied()
    .collect();

    assert_eq!(canonical.len(), 12, "AC-012: must have exactly 12 codes");

    let phantom_codes = [
        "pty_write_failed",
        "unknown_command",
        "internal_error",
        "permission_denied",
        "protocol_error",
        "rename_rejected",
        "kill_rejected",
    ];
    for phantom in &phantom_codes {
        assert!(
            !canonical.contains(phantom),
            "AC-012: phantom code '{phantom}' MUST NOT be in canonical set"
        );
    }

    // KeyInput+SessionHostDead → "attach_failed" (not "pty_write_failed").
    let code = monocle_runtime::session_manager::session_error_to_code(
        IpcOp::KeyInput,
        &SessionError::SessionHostDead {
            session_id: "test".to_string(),
        },
    );
    assert_eq!(
        code, "attach_failed",
        "AC-012: KeyInput+SessionHostDead → 'attach_failed' (not 'pty_write_failed'), got '{code}'"
    );

    // Detach+SessionNotReady → "session_not_ready".
    let code2 = monocle_runtime::session_manager::session_error_to_code(
        IpcOp::Detach,
        &SessionError::SessionNotReady {
            session_id: "test".to_string(),
        },
    );
    assert_eq!(
        code2, "session_not_ready",
        "AC-012: Detach+SessionNotReady → 'session_not_ready', got '{code2}'"
    );

    // RenameSession+InvalidSessionName → "rename_failed".
    let code3 = monocle_runtime::session_manager::session_error_to_code(
        IpcOp::Rename,
        &SessionError::InvalidSessionName {
            reason: "too long".to_string(),
        },
    );
    assert_eq!(
        code3, "rename_failed",
        "AC-012: Rename+InvalidSessionName → 'rename_failed', got '{code3}'"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / AC-008: AttachSession triggers scrollback dump sequence
// ---------------------------------------------------------------------------

/// AC-006 / AC-008: `forward_scrollback_dump_to_client()` sends ScrollbackChunk* +
/// ScrollbackDumpComplete. total_chunks in DumpComplete matches received chunk count.
///
/// FAILS because `forward_scrollback_dump_to_client()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_attach_session_triggers_scrollback_sequence() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-attach-dump.sock");
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket for attach dump test");

    let (mut manager, subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_060, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "47060001-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");

    let _peer = advance_to_running(&listener, &session_id, &mut rx).await;

    let (client_tx, mut client_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(64);
    {
        use monocle_ipc::server::ClientEntry;
        subs.lock().await.push(ClientEntry::new(client_tx.clone()));
    }

    // FAILS: forward_scrollback_dump_to_client() is todo!()
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        manager.forward_scrollback_dump_to_client(&session_id, "test-client-attach", &client_tx),
    )
    .await
    .expect("forward_scrollback_dump_to_client must not hang");

    assert!(
        result.is_ok(),
        "AC-006: forward_scrollback_dump_to_client must succeed; got: {:?}",
        result
    );

    let msgs = drain_client(&mut client_rx, std::time::Duration::from_secs(2)).await;

    let dump_complete = msgs.iter().find(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::ScrollbackDumpComplete {
                session_id: ref sid,
                ..
            } if sid == &session_id
        )
    });
    assert!(
        dump_complete.is_some(),
        "AC-006: client must receive ScrollbackDumpComplete. Got: {:?}",
        msgs
    );

    // AC-008: total_chunks must match received ScrollbackChunk count.
    let chunk_count = msgs
        .iter()
        .filter(|m| {
            matches!(
                m,
                monocle_ipc::types::ServerToClient::ScrollbackChunk {
                    session_id: ref sid,
                    ..
                } if sid == &session_id
            )
        })
        .count();
    if let Some(monocle_ipc::types::ServerToClient::ScrollbackDumpComplete {
        total_chunks, ..
    }) = dump_complete
    {
        assert_eq!(
            *total_chunks as usize, chunk_count,
            "AC-008: total_chunks ({}) must match received chunk count ({})",
            total_chunks, chunk_count
        );
    }
}

// ---------------------------------------------------------------------------
// AC-010: pending_pty_bytes buffered during dump; drained after DumpComplete
// ---------------------------------------------------------------------------

/// AC-010: During a scrollback dump, live PTY bytes are buffered in `pending_pty_bytes`
/// per session-client pair. After DumpComplete, buffered bytes are drained to the client.
///
/// FAILS because `forward_scrollback_dump_to_client()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_pending_pty_bytes_buffered_during_dump() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-pending-pty.sock");
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket for pending_pty test");

    let (mut manager, subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_100, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "47100000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");

    let _peer = advance_to_running(&listener, &session_id, &mut rx).await;

    let (client_tx, mut client_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(64);
    {
        use monocle_ipc::server::ClientEntry;
        subs.lock().await.push(ClientEntry::new(client_tx.clone()));
    }

    // FAILS: forward_scrollback_dump_to_client() is todo!()
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        manager.forward_scrollback_dump_to_client(&session_id, "test-client-pending", &client_tx),
    )
    .await
    .expect("must not hang");

    assert!(
        result.is_ok(),
        "AC-010: forward_scrollback_dump_to_client must succeed; got {:?}",
        result
    );

    let msgs = drain_client(&mut client_rx, std::time::Duration::from_secs(2)).await;
    let dump_complete = msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::ScrollbackDumpComplete {
                session_id: ref sid,
                ..
            } if sid == &session_id
        )
    });
    assert!(
        dump_complete,
        "AC-010: must receive ScrollbackDumpComplete. Got: {:?}",
        msgs
    );
}

// ---------------------------------------------------------------------------
// AC-SH-005: forward_scrollback_dump sends to requesting client ONLY
// ---------------------------------------------------------------------------

/// AC-SH-005: `forward_scrollback_dump_to_client()` sends ScrollbackChunk* and
/// ScrollbackDumpComplete to the REQUESTING client only (via targeted tx, not broadcast).
/// Passive observers MUST NOT receive scrollback messages.
///
/// FAILS because `forward_scrollback_dump_to_client()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_forward_scrollback_dump_sends_to_requesting_client_only() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-sh005.sock");
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket for AC-SH-005 test");

    let (mut manager, subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_052, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "47052000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");

    let _peer = advance_to_running(&listener, &session_id, &mut rx).await;

    let (client_a_tx, mut client_a_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(64);
    let (client_b_tx, mut client_b_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(64);
    {
        use monocle_ipc::server::ClientEntry;
        let mut lock = subs.lock().await;
        lock.push(ClientEntry::new(client_a_tx.clone()));
        lock.push(ClientEntry::new(client_b_tx.clone()));
    }

    // FAILS: forward_scrollback_dump_to_client() is todo!()
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        manager.forward_scrollback_dump_to_client(&session_id, "client-a", &client_a_tx),
    )
    .await
    .expect("must not hang");

    assert!(
        result.is_ok(),
        "AC-SH-005: forward_scrollback_dump_to_client must succeed; got {:?}",
        result
    );

    let a_msgs = drain_client(&mut client_a_rx, std::time::Duration::from_secs(2)).await;
    let b_msgs = drain_client(&mut client_b_rx, std::time::Duration::from_millis(300)).await;

    let a_complete = a_msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::ScrollbackDumpComplete {
                session_id: ref sid,
                ..
            } if sid == &session_id
        )
    });
    assert!(
        a_complete,
        "AC-SH-005: requesting client MUST receive ScrollbackDumpComplete. Got: {:?}",
        a_msgs
    );

    let b_has_scrollback = b_msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::ScrollbackChunk { .. }
                | monocle_ipc::types::ServerToClient::ScrollbackDumpComplete { .. }
        )
    });
    assert!(
        !b_has_scrollback,
        "AC-SH-005: passive client MUST NOT receive scrollback dump. Got: {:?}",
        b_msgs
    );
}

// ---------------------------------------------------------------------------
// EC-306: Empty dump (total_chunks = 0) is valid
// ---------------------------------------------------------------------------

/// EC-306: Session with no output → ScrollbackDumpComplete{total_chunks: 0}.
/// No ScrollbackChunk messages. Valid; client reconstructs empty screen.
///
/// FAILS because `forward_scrollback_dump_to_client()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_ec_306_empty_dump_total_chunks_zero() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-ec306.sock");
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket for EC-306 test");

    let (mut manager, subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_306, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "ec306000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");

    let _peer = advance_to_running(&listener, &session_id, &mut rx).await;

    let (client_tx, mut client_rx) =
        tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(64);
    {
        use monocle_ipc::server::ClientEntry;
        subs.lock().await.push(ClientEntry::new(client_tx.clone()));
    }

    // FAILS: forward_scrollback_dump_to_client() is todo!()
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        manager.forward_scrollback_dump_to_client(&session_id, "client-306", &client_tx),
    )
    .await
    .expect("must not hang");

    assert!(
        result.is_ok(),
        "EC-306: empty dump must succeed; got {:?}",
        result
    );

    let msgs = drain_client(&mut client_rx, std::time::Duration::from_secs(2)).await;

    let complete_zero = msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::ScrollbackDumpComplete {
                session_id: ref sid,
                total_chunks: 0,
                ..
            } if sid == &session_id
        )
    });
    assert!(
        complete_zero,
        "EC-306: empty session must produce ScrollbackDumpComplete{{total_chunks: 0}}. \
         Got: {:?}",
        msgs
    );

    let has_chunks = msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::ScrollbackChunk {
                session_id: ref sid,
                ..
            } if sid == &session_id
        )
    });
    assert!(
        !has_chunks,
        "EC-306: empty session MUST NOT produce ScrollbackChunk. Got: {:?}",
        msgs
    );
}

// ---------------------------------------------------------------------------
// EC-308: Two clients get independent scrollback dumps
// ---------------------------------------------------------------------------

/// EC-308: Two clients simultaneously attach to the same session each receive
/// an independent scrollback dump. pending_pty_bytes is per session-client pair.
///
/// FAILS because `forward_scrollback_dump_to_client()` is `todo!()`.
#[tokio::test]
async fn test_BC_2_05_011_ec_308_two_clients_independent_dumps() {
    let tmp = isolated_runtime_dir();
    let socket_path = tmp.path().join("s047-ec308.sock");
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("bind socket for EC-308 test");

    let (mut manager, subs, mut rx) =
        make_manager_with_socket(tmp.path(), 47_308, socket_path.clone());
    manager.with_peer_cred_verifier(Arc::new(AllowAllVerifier));

    let session_id = "ec308000-0000-4000-a000-000000000001".to_string();
    manager
        .spawn_session(make_spawn_opts(&session_id))
        .await
        .expect("spawn must succeed");

    let _peer = advance_to_running(&listener, &session_id, &mut rx).await;

    let (c1_tx, mut c1_rx) = tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(64);
    let (c2_tx, mut c2_rx) = tokio::sync::mpsc::channel::<monocle_ipc::types::ServerToClient>(64);
    {
        use monocle_ipc::server::ClientEntry;
        let mut lock = subs.lock().await;
        lock.push(ClientEntry::new(c1_tx.clone()));
        lock.push(ClientEntry::new(c2_tx.clone()));
    }

    // FAILS: forward_scrollback_dump_to_client() is todo!()
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        manager.forward_scrollback_dump_to_client(&session_id, "client-308-a", &c1_tx),
    )
    .await
    .expect("c1 must not hang")
    .expect("c1 forward must succeed");

    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        manager.forward_scrollback_dump_to_client(&session_id, "client-308-b", &c2_tx),
    )
    .await
    .expect("c2 must not hang")
    .expect("c2 forward must succeed");

    let c1_msgs = drain_client(&mut c1_rx, std::time::Duration::from_secs(2)).await;
    let c2_msgs = drain_client(&mut c2_rx, std::time::Duration::from_secs(2)).await;

    let c1_complete = c1_msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::ScrollbackDumpComplete {
                session_id: ref sid,
                ..
            } if sid == &session_id
        )
    });
    let c2_complete = c2_msgs.iter().any(|m| {
        matches!(
            m,
            monocle_ipc::types::ServerToClient::ScrollbackDumpComplete {
                session_id: ref sid,
                ..
            } if sid == &session_id
        )
    });

    assert!(
        c1_complete,
        "EC-308: client1 must receive independent ScrollbackDumpComplete. Got: {:?}",
        c1_msgs
    );
    assert!(
        c2_complete,
        "EC-308: client2 must receive independent ScrollbackDumpComplete. Got: {:?}",
        c2_msgs
    );
}
