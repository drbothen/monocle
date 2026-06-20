// S-036 TDD Test Suite — SessionManager::rediscover_sessions()
//
// Derived from:
//   BC-2.08.002  (session-host survives graceful daemon restart; setsid; InitialState
//                 after restart includes re-discovered sessions with scrollback)
//   BC-2.08.004  (rediscover_sessions: all alive sessions visible within 5s; UDS bind
//                 blocked until complete; all SessionState variants handled;
//                 RediscoveryReport shape; schema_version 1/2/3 accepted; 4+ skipped)
//
// Test naming: test_BC_S_SS_NNN_<assertion_name>
//
// RED GATE REQUIREMENT: every test MUST FAIL against the current todo!() stub in
// `rediscover_sessions()`. The todo!() panics before any assertion fires.
//
// Socket path note (macOS SUN_LEN=104): socket files for mock session-hosts must be
// placed at short /tmp paths. Sidecar files live in tempdir; sidecar socket_path field
// points to the short /tmp socket path.

#![allow(
    clippy::too_many_lines,
    non_snake_case,
    clippy::io_other_error,
    clippy::while_let_loop
)]

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::{mpsc, Mutex};

use monocle_core::engine::{EngineError, SpawnOptions};
use monocle_ipc::server::{ClientEntry, SubscriberList, CLIENT_CHANNEL_CAPACITY};
use monocle_ipc::types::{HostToDaemon, ServerToClient, SessionState};

use super::{
    FakePeerCredVerifier, HookEndpointConfig, MockSessionHostSpawner, RediscoveryError,
    SessionManager,
};

// ---------------------------------------------------------------------------
// Local engine mock (mirrors SucceedingMockEngine in the parent tests module)
// ---------------------------------------------------------------------------

struct SucceedingMockEngineRediscovery;

#[async_trait::async_trait]
impl monocle_core::engine::EngineModule for SucceedingMockEngineRediscovery {
    fn id(&self) -> &'static str {
        "mock-engine-rediscovery"
    }

    fn metadata(
        &self,
    ) -> Result<monocle_core::engine::EngineMetadata, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for rediscovery tests")
    }

    fn detect(&self, _proc: &monocle_core::engine::ProcessSnapshot) -> bool {
        false
    }

    async fn enrich(
        &self,
        _proc: &monocle_core::engine::ProcessSnapshot,
    ) -> Result<monocle_core::engine::EnrichedSession, monocle_core::engine::EngineMetadataError>
    {
        unimplemented!("not needed for rediscovery tests")
    }

    async fn on_hook(
        &self,
        _event: monocle_core::hook_events::HookEvent,
    ) -> monocle_core::engine::HookResponse {
        unimplemented!("not needed for rediscovery tests")
    }

    fn spawn_recipe(
        &self,
        opts: &SpawnOptions,
    ) -> Result<monocle_core::engine::SpawnRecipe, EngineError> {
        Ok(monocle_core::engine::SpawnRecipe::new(
            PathBuf::from("claude"),
            vec!["--dangerously-skip-permissions".to_string()],
            std::collections::HashMap::new(),
            opts.worktree_root.clone(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Wrap an inner `SubscriberList` in a second `Arc` to produce the broker type
/// expected by `SessionManager::new`.
fn make_broker(subs: &SubscriberList) -> Arc<SubscriberList> {
    Arc::new(Arc::clone(subs))
}

/// Build a `SessionManager` with a `FakePeerCredVerifier` and a
/// `MockSessionHostSpawner` that never spawns OS processes.
fn make_rediscovery_manager(
    runtime_dir: &Path,
    peercred_allow: bool,
) -> (
    SessionManager,
    SubscriberList,
    mpsc::Receiver<ServerToClient>,
) {
    let (tx, rx) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let entry = ClientEntry::new(tx);
    let subs: SubscriberList = Arc::new(Mutex::new(vec![entry]));
    let broker = make_broker(&subs);

    let spawner = Arc::new(MockSessionHostSpawner {
        spawn_result: None,
        fake_pid: 0,
    });
    let engine: Arc<dyn monocle_core::engine::EngineModule> =
        Arc::new(SucceedingMockEngineRediscovery);

    let mut manager = SessionManager::new(
        runtime_dir.to_path_buf(),
        spawner,
        broker,
        engine,
        HookEndpointConfig::default(),
    );
    manager.with_peer_cred_verifier(Arc::new(FakePeerCredVerifier {
        allow: peercred_allow,
    }));

    (manager, subs, rx)
}

/// Generate a short socket path in `/tmp` safe for macOS SUN_LEN (104 bytes).
///
/// Pattern: `/tmp/s036-<short_tag>-<nanos>.sock`
/// Max length ≈ 5 + 12 + 1 + 10 + 5 = ~33 bytes — safely under 104.
fn short_socket_path(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    PathBuf::from(format!("/tmp/s036-{}-{}.sock", tag, nanos))
}

/// Write a schema_version 3 sidecar JSON file.
///
/// `socket_path` is the actual OS path for the session-host UDS socket.
/// This may differ from `runtime_dir/session-<id>.sock` when the path would
/// exceed macOS SUN_LEN (104 bytes).
fn write_sidecar_v3(
    runtime_dir: &Path,
    session_id: &str,
    state: &str,
    pid: u32,
    socket_path: &Path,
    kill_deadline_unix_ms: Option<u64>,
) -> PathBuf {
    let sidecar = serde_json::json!({
        "schema_version": 3u32,
        "session_id": session_id,
        "pid": pid,
        "socket_path": socket_path.to_string_lossy(),
        "child_pid": serde_json::Value::Null,
        "state": state,
        "project_root": "/tmp/test-project",
        "cwd": "/tmp/test-cwd",
        "harness_id": "claude-code",
        "profile_id": "default",
        "started_at": chrono::Utc::now().to_rfc3339(),
        "display_name": format!("claude-code — test-project ({})", &session_id[..8]),
        "pty_rows": 24u16,
        "pty_cols": 80u16,
        "kill_deadline_unix_ms": match kill_deadline_unix_ms {
            Some(ms) => serde_json::Value::Number(ms.into()),
            None => serde_json::Value::Null,
        },
    });
    let sidecar_path = runtime_dir.join(format!("session-{}.json", session_id));
    let mut f = std::fs::File::create(&sidecar_path)
        .unwrap_or_else(|e| panic!("write_sidecar_v3: create {sidecar_path:?}: {e}"));
    f.write_all(&serde_json::to_vec_pretty(&sidecar).expect("write_sidecar_v3: serialize"))
        .expect("write_sidecar_v3: write");
    sidecar_path
}

/// Write a schema_version 1 sidecar (legacy — no `cwd`, no `kill_deadline_unix_ms`).
///
/// BC-2.08.004 PC-1: schema_version 1 → `cwd = project_root`; fully accepted.
fn write_sidecar_v1_legacy(
    runtime_dir: &Path,
    session_id: &str,
    state: &str,
    pid: u32,
    socket_path: &Path,
) -> PathBuf {
    // Deliberately omit `cwd` and `kill_deadline_unix_ms`.
    let sidecar = serde_json::json!({
        "schema_version": 1u32,
        "session_id": session_id,
        "pid": pid,
        "socket_path": socket_path.to_string_lossy(),
        "child_pid": serde_json::Value::Null,
        "state": state,
        "project_root": "/tmp/test-project-v1",
        "harness_id": "claude-code",
        "profile_id": "default",
        "started_at": chrono::Utc::now().to_rfc3339(),
        "display_name": "claude-code — test-project-v1",
        "pty_rows": 24u16,
        "pty_cols": 80u16,
    });
    let sidecar_path = runtime_dir.join(format!("session-{}.json", session_id));
    let mut f = std::fs::File::create(&sidecar_path)
        .unwrap_or_else(|e| panic!("write_sidecar_v1: create {sidecar_path:?}: {e}"));
    f.write_all(&serde_json::to_vec_pretty(&sidecar).expect("write_sidecar_v1: serialize"))
        .expect("write_sidecar_v1: write");
    sidecar_path
}

/// Write a sidecar with `schema_version: 4` (unknown future version).
fn write_sidecar_future_version(
    runtime_dir: &Path,
    session_id: &str,
    pid: u32,
    socket_path: &Path,
) -> PathBuf {
    let sidecar = serde_json::json!({
        "schema_version": 4u32,
        "session_id": session_id,
        "pid": pid,
        "socket_path": socket_path.to_string_lossy(),
        "child_pid": serde_json::Value::Null,
        "state": "Running",
        "project_root": "/tmp/test-project",
        "cwd": "/tmp/test-cwd",
        "harness_id": "claude-code",
        "profile_id": "default",
        "started_at": chrono::Utc::now().to_rfc3339(),
        "display_name": "claude-code — test-project",
        "pty_rows": 24u16,
        "pty_cols": 80u16,
        "kill_deadline_unix_ms": serde_json::Value::Null,
    });
    let sidecar_path = runtime_dir.join(format!("session-{}.json", session_id));
    let mut f = std::fs::File::create(&sidecar_path)
        .unwrap_or_else(|e| panic!("write_sidecar_future: create {sidecar_path:?}: {e}"));
    f.write_all(&serde_json::to_vec_pretty(&sidecar).expect("write_sidecar_future: serialize"))
        .expect("write_sidecar_future: write");
    sidecar_path
}

/// Spawn a mock session-host on `socket_path` that responds to `DaemonToHost::Attach`
/// with the chunked scrollback protocol.
///
/// Protocol: accept → consume Attach frame → send ScrollbackChunk(seq=0) →
/// send ScrollbackDumpComplete → hold open briefly.
fn spawn_mock_session_host_attach(socket_path: PathBuf) -> mpsc::Receiver<()> {
    let (done_tx, done_rx) = mpsc::channel::<()>(1);
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&socket_path);
        let listener = UnixListener::bind(&socket_path)
            .unwrap_or_else(|e| panic!("mock_host_attach: bind {:?}: {e}", socket_path));

        let (mut stream, _) = listener.accept().await.expect("mock_host_attach: accept");

        // Consume DaemonToHost::Attach.
        let mut len_buf = [0u8; 4];
        stream
            .read_exact(&mut len_buf)
            .await
            .expect("mock_host_attach: read Attach len");
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut body = vec![0u8; len];
        stream
            .read_exact(&mut body)
            .await
            .expect("mock_host_attach: read Attach body");

        // Reply: ScrollbackChunk(0) + ScrollbackDumpComplete.
        let chunk = HostToDaemon::ScrollbackChunk {
            rows: vec![],
            chunk_seq: 0,
        };
        send_lp_frame(&mut stream, &chunk).await;

        let complete = HostToDaemon::ScrollbackDumpComplete {
            total_chunks: 1,
            cursor_row: 0,
            cursor_col: 0,
            pty_rows: 24,
            pty_cols: 80,
        };
        send_lp_frame(&mut stream, &complete).await;

        let _ = done_tx.send(()).await;
        // Hold stream open.
        tokio::time::sleep(Duration::from_millis(300)).await;
    });
    done_rx
}

/// Spawn a mock session-host that accepts but never responds (stuck/non-responsive).
fn spawn_mock_session_host_silent(socket_path: PathBuf) -> mpsc::Receiver<()> {
    let (accepted_tx, accepted_rx) = mpsc::channel::<()>(1);
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&socket_path);
        let listener = UnixListener::bind(&socket_path)
            .unwrap_or_else(|e| panic!("mock_host_silent: bind {:?}: {e}", socket_path));
        let (_stream, _) = listener.accept().await.expect("mock_host_silent: accept");
        let _ = accepted_tx.send(()).await;
        // Never respond.
        tokio::time::sleep(Duration::from_secs(60)).await;
    });
    accepted_rx
}

/// Send a single length-prefixed JSON frame (4-byte LE u32 + JSON body).
async fn send_lp_frame(stream: &mut tokio::net::UnixStream, msg: &HostToDaemon) {
    let bytes = serde_json::to_vec(msg).expect("send_lp_frame: serialize");
    let len = (bytes.len() as u32).to_le_bytes();
    stream
        .write_all(&len)
        .await
        .expect("send_lp_frame: write len");
    stream
        .write_all(&bytes)
        .await
        .expect("send_lp_frame: write body");
    stream.flush().await.expect("send_lp_frame: flush");
}

/// Current Unix epoch in milliseconds.
fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ===========================================================================
// AC-004 / BC-2.08.004 PC-2b Running (re-attach → found_alive: 1)
// ===========================================================================

/// AC-004: sidecar `state: "Running"` + alive mock session-host →
/// Attach sent, ScrollbackDumpComplete received, `SessionEntry{Running}` registered,
/// `found_alive: 1`.
///
/// BC-2.08.004 postcondition 2b (Running arm).
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_running_session_reregistered() {
    let tmp = tempfile::tempdir().expect("AC-004: tempdir");
    let session_id = "00000000-0036-4000-a001-000000000001";

    let socket_path = short_socket_path("ac004-run");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(tmp.path(), session_id, "Running", 0, &socket_path, None);

    let mut done_rx = spawn_mock_session_host_attach(socket_path.clone());

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here with "not yet implemented".
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-004: rediscover_sessions must return Ok");

    let _ = tokio::time::timeout(Duration::from_secs(6), done_rx.recv()).await;

    assert_eq!(
        report.found_alive, 1,
        "AC-004: Running + alive mock → found_alive=1; got {}",
        report.found_alive
    );
    assert_eq!(
        report.found_dead, 0,
        "AC-004: found_dead=0; got {}",
        report.found_dead
    );
    assert!(
        report.errors.is_empty(),
        "AC-004: errors=[]; got {:?}",
        report.errors
    );

    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("AC-004: session must be in registry");
    assert_eq!(
        snap.state,
        SessionState::Running,
        "AC-004: re-discovered session state must be Running; got {:?}",
        snap.state
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// AC-008 / BC-2.08.004 PC-2c (Dead PID: GC immediately)
// ===========================================================================

/// AC-008: sidecar with dead PID → sidecar deleted; no `SessionEntry`; `found_dead: 1`.
///
/// BC-2.08.004 postcondition 2c.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_dead_pid_gc() {
    let tmp = tempfile::tempdir().expect("AC-008: tempdir");
    let session_id = "00000000-0036-4000-a001-000000000003";

    let mut child = std::process::Command::new("true")
        .spawn()
        .expect("AC-008: spawn 'true'");
    let dead_pid = child.id();
    let _ = child.wait(); // reap to avoid zombie; process exits immediately

    let socket_path = short_socket_path("ac008-dead");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Running",
        dead_pid,
        &socket_path,
        None,
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-008: rediscover_sessions must return Ok");

    assert_eq!(
        report.found_dead, 1,
        "AC-008: dead PID → found_dead=1; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "AC-008: found_alive=0; got {}",
        report.found_alive
    );
    assert!(
        !sidecar_path.exists(),
        "AC-008: dead sidecar must be deleted; still at {:?}",
        sidecar_path
    );
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "AC-008: dead session MUST NOT appear in registry"
    );
}

// ===========================================================================
// AC-010 / BC-2.08.004 PC-5 (Corrupt sidecar: WARN; delete; CorruptSidecar error)
// ===========================================================================

/// AC-010: corrupt JSON sidecar → WARN; sidecar deleted; `errors` has 1
/// `CorruptSidecar`; function returns Ok; other sessions unaffected.
///
/// BC-2.08.004 postcondition 5.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_corrupt_sidecar() {
    let tmp = tempfile::tempdir().expect("AC-010: tempdir");
    let corrupt_id = "00000000-0036-4000-a001-000000000004";

    let corrupt_path = tmp.path().join(format!("session-{}.json", corrupt_id));
    {
        let mut f = std::fs::File::create(&corrupt_path).expect("AC-010: create corrupt sidecar");
        f.write_all(b"this is not valid JSON {{{{{ corrupt $$$$")
            .expect("AC-010: write corrupt");
    }

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-010: rediscover_sessions must return Ok (corrupt is non-fatal)");

    assert_eq!(
        report.errors.len(),
        1,
        "AC-010: errors must have 1 CorruptSidecar entry; got {:?}",
        report.errors
    );
    assert!(
        matches!(&report.errors[0], RediscoveryError::CorruptSidecar { .. }),
        "AC-010: error must be CorruptSidecar; got {:?}",
        report.errors[0]
    );
    assert!(
        !corrupt_path.exists(),
        "AC-010: corrupt sidecar must be deleted; still at {:?}",
        corrupt_path
    );
    assert_eq!(
        report.found_alive, 0,
        "AC-010: found_alive=0; got {}",
        report.found_alive
    );
}

// ===========================================================================
// AC-003 / BC-2.08.004 PC-1 (schema_version 1 legacy: cwd = project_root)
// ===========================================================================

/// AC-003 (v1): sidecar without `cwd` field (schema_version 1) → accepted;
/// `cwd` defaults to `project_root`; proceeds as Running re-discovery.
///
/// BC-2.08.004 postcondition 1 (schema_version 1 arm).
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_schema_v1_legacy() {
    let tmp = tempfile::tempdir().expect("AC-003-v1: tempdir");
    let session_id = "00000000-0036-4000-a001-000000000005";

    let socket_path = short_socket_path("ac003-v1");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v1_legacy(tmp.path(), session_id, "Running", 0, &socket_path);
    let mut done_rx = spawn_mock_session_host_attach(socket_path.clone());

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-003-v1: rediscover_sessions must return Ok");

    let _ = tokio::time::timeout(Duration::from_secs(6), done_rx.recv()).await;

    assert_eq!(
        report.found_alive, 1,
        "AC-003 (v1): schema_version 1 accepted → found_alive=1; got {}",
        report.found_alive
    );
    assert!(
        report.errors.is_empty(),
        "AC-003 (v1): errors=[]; got {:?}",
        report.errors
    );

    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("AC-003 (v1): session must be in registry");
    assert_eq!(
        snap.cwd, "/tmp/test-project-v1",
        "AC-003 (v1): cwd must equal project_root for schema_version 1; got '{}'",
        snap.cwd
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// AC-003 / BC-2.08.004 PC-1 (schema_version 4 future: WARN; delete; skip)
// ===========================================================================

/// AC-003 (v4): unknown future schema_version → WARN; sidecar deleted as orphan;
/// `found_dead` NOT incremented; `errors` has 1 `UnknownSchemaVersion{version:4}`.
///
/// BC-2.08.004 postcondition 1 (schema_version > 3 arm).
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_schema_v4_future() {
    let tmp = tempfile::tempdir().expect("AC-003-v4: tempdir");
    let session_id = "00000000-0036-4000-a001-000000000006";

    let socket_path = short_socket_path("ac003-v4");
    write_sidecar_future_version(tmp.path(), session_id, 0, &socket_path);
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-003-v4: rediscover_sessions must return Ok");

    assert!(
        !sidecar_path.exists(),
        "AC-003 (v4): schema_version 4 sidecar must be deleted; still at {:?}",
        sidecar_path
    );
    assert_eq!(
        report.found_dead, 0,
        "AC-003 (v4): found_dead NOT incremented for schema skip; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "AC-003 (v4): found_alive=0; got {}",
        report.found_alive
    );
    assert_eq!(
        report.errors.len(),
        1,
        "AC-003 (v4): errors has 1 UnknownSchemaVersion entry; got {:?}",
        report.errors
    );
    assert!(
        matches!(
            &report.errors[0],
            RediscoveryError::UnknownSchemaVersion { version: 4, .. }
        ),
        "AC-003 (v4): error must be UnknownSchemaVersion{{version:4}}; got {:?}",
        report.errors[0]
    );
}

// ===========================================================================
// AC-006 / AC-014 / BC-2.08.004 PC-2b Terminating (elapsed → SIGKILL)
// ===========================================================================

/// AC-006 / AC-014: sidecar `state: "Terminating"` with elapsed deadline →
/// immediate SIGKILL (via seam); sidecar deleted; no `SessionEntry`; `found_dead: 1`.
///
/// BC-2.08.004 postcondition 2b (Terminating — elapsed sub-branch) + Invariant 7.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_elapsed_deadline() {
    let tmp = tempfile::tempdir().expect("AC-006-elapsed: tempdir");
    let session_id = "00000000-0036-4000-a001-000000000007";

    let elapsed_ms = unix_now_ms().saturating_sub(1_000);
    let socket_path = short_socket_path("ac006-elapsed");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(),
        &socket_path,
        Some(elapsed_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    let (sigkill_tx, mut sigkill_rx) = mpsc::channel::<u32>(4);
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);
    manager.with_pid_sigkill_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigkill_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-006-elapsed: rediscover_sessions must return Ok");

    let sigkill_pid = tokio::time::timeout(Duration::from_millis(200), sigkill_rx.recv())
        .await
        .expect("AC-006 / AC-014: SIGKILL must fire within 200ms for elapsed deadline")
        .expect("AC-006 / AC-014: sigkill channel closed");
    assert_eq!(
        sigkill_pid,
        std::process::id(),
        "AC-006 / AC-014: SIGKILL must target the session-host PID"
    );
    assert!(
        !sidecar_path.exists(),
        "AC-006 / AC-014: sidecar must be deleted; still at {:?}",
        sidecar_path
    );
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "AC-006 / AC-014: elapsed Terminating MUST NOT appear in registry"
    );
    assert_eq!(
        report.found_dead, 1,
        "AC-006 / AC-014: elapsed Terminating → found_dead=1; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "AC-006 / AC-014: found_alive=0; got {}",
        report.found_alive
    );
}

// ===========================================================================
// AC-006 / BC-2.08.004 PC-2b Terminating (NOT elapsed → watchdog + found_alive)
// ===========================================================================

/// AC-006 (not elapsed): sidecar `state: "Terminating"` with future deadline →
/// Kill fire-and-forget; `SessionEntry{Terminating}` registered; background watchdog;
/// `rediscover_sessions()` returns immediately; `found_alive: 1`.
///
/// BC-2.08.004 postcondition 2b (Terminating — not-elapsed sub-branch) + Invariant 2.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_not_elapsed_deadline() {
    let tmp = tempfile::tempdir().expect("AC-006-not-elapsed: tempdir");
    let session_id = "00000000-0036-4000-a001-000000000008";

    let future_ms = unix_now_ms() + 10_000;
    let socket_path = short_socket_path("ac006-future");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(),
        &socket_path,
        Some(future_ms),
    );

    let (kill_received_tx, mut kill_received_rx) = mpsc::channel::<()>(1);
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("AC-006-not-elapsed: mock bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut body = vec![0u8; len];
                let _ = stream.read_exact(&mut body).await;
                let _ = kill_received_tx.send(()).await;
            }
            // Hold open.
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    let start = std::time::Instant::now();

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-006-not-elapsed: rediscover_sessions must return Ok");

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "AC-006: must return before Terminating watchdog fires; took {:?}",
        elapsed
    );

    let kill_ok = tokio::time::timeout(Duration::from_secs(3), kill_received_rx.recv()).await;
    assert!(
        kill_ok.is_ok() && kill_ok.unwrap().is_some(),
        "AC-006: Kill must be sent fire-and-forget to Terminating session-host"
    );

    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("AC-006: Terminating session must be in registry");
    assert_eq!(
        snap.state,
        SessionState::Terminating,
        "AC-006: state must be Terminating; got {:?}",
        snap.state
    );
    assert_eq!(
        report.found_alive, 1,
        "AC-006: Terminating + future deadline → found_alive=1; got {}",
        report.found_alive
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// AC-012 / BC-2.08.004 PC-7 + Invariant 3 (parallel: 8 sessions ≤ 5s)
// ===========================================================================

/// AC-012: 8 mock session-hosts probed concurrently via `tokio::join_all`;
/// wall-clock ≤ 5s (each mock has 100ms simulated latency).
///
/// Uses `start_paused = true` for time auto-advance.
///
/// BC-2.08.004 postcondition 7 + Invariant 3 (parallel attach required).
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_004_rediscovery_parallelism_8_sessions() {
    let tmp = tempfile::tempdir().expect("AC-012: tempdir");

    // Use short unique socket paths (macOS SUN_LEN constraint).
    let base_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let session_ids: Vec<String> = (0..8u8)
        .map(|i| format!("00000000-0036-4012-b001-{:012}", i))
        .collect();

    for (i, id) in session_ids.iter().enumerate() {
        let socket_path = PathBuf::from(format!("/tmp/s036-p8-{}-{}.sock", i, base_nanos));
        let _ = std::fs::remove_file(&socket_path);
        write_sidecar_v3(tmp.path(), id, "Running", 0, &socket_path, None);
        let sock = socket_path.clone();
        tokio::spawn(async move {
            let _ = std::fs::remove_file(&sock);
            let listener = UnixListener::bind(&sock).expect("AC-012: mock bind");
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut len_buf = [0u8; 4];
                let _ = stream.read_exact(&mut len_buf).await;
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut body = vec![0u8; len];
                let _ = stream.read_exact(&mut body).await;
                tokio::time::sleep(Duration::from_millis(100)).await;
                let chunk = HostToDaemon::ScrollbackChunk {
                    rows: vec![],
                    chunk_seq: 0,
                };
                send_lp_frame(&mut stream, &chunk).await;
                let complete = HostToDaemon::ScrollbackDumpComplete {
                    total_chunks: 1,
                    cursor_row: 0,
                    cursor_col: 0,
                    pty_rows: 24,
                    pty_cols: 80,
                };
                send_lp_frame(&mut stream, &complete).await;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    tokio::time::sleep(Duration::from_millis(10)).await;

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    let wall_start = std::time::Instant::now();

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-012: rediscover_sessions must return Ok for 8 sessions");

    let wall_elapsed = wall_start.elapsed();

    assert_eq!(
        report.found_alive, 8,
        "AC-012: all 8 sessions must be found alive; got {}",
        report.found_alive
    );
    assert_eq!(
        report.found_dead, 0,
        "AC-012: found_dead=0; got {}",
        report.found_dead
    );
    assert!(
        wall_elapsed < Duration::from_secs(5),
        "AC-012: 8 parallel probes must complete < 5s; took {:?}",
        wall_elapsed
    );

    for i in 0..8usize {
        let _ = std::fs::remove_file(PathBuf::from(format!(
            "/tmp/s036-p8-{}-{}.sock",
            i, base_nanos
        )));
    }
}

// ===========================================================================
// AC-007 (Terminated: GC; unknown state: WARN + delete)
// ===========================================================================

/// AC-007 (Terminated): crash-leftover sidecar → deleted; no `SessionEntry`;
/// `found_dead: 1`.
///
/// BC-2.08.004 postcondition 2b (Terminated arm).
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminated_state_gc() {
    let tmp = tempfile::tempdir().expect("AC-007-terminated: tempdir");
    let session_id = "00000000-0036-4000-a001-000000000009";

    let socket_path = short_socket_path("ac007-term");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminated",
        std::process::id(),
        &socket_path,
        None,
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-007: rediscover_sessions must return Ok");

    assert!(
        !sidecar_path.exists(),
        "AC-007: Terminated sidecar must be deleted; still at {:?}",
        sidecar_path
    );
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "AC-007: Terminated session MUST NOT appear in registry"
    );
    assert_eq!(
        report.found_dead, 1,
        "AC-007: Terminated → found_dead=1; got {}",
        report.found_dead
    );
}

/// AC-007 (unknown state): `state: "UnknownFutureState"` → WARN; sidecar deleted;
/// errors non-empty; no `SessionEntry`.
///
/// BC-2.08.004 postcondition 2b (unknown state arm).
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_unknown_state_string_warn_delete() {
    let tmp = tempfile::tempdir().expect("AC-007-unknown: tempdir");
    let session_id = "00000000-0036-4000-a001-00000000000a";

    let socket_path = short_socket_path("ac007-unk");
    let sidecar = serde_json::json!({
        "schema_version": 3u32,
        "session_id": session_id,
        "pid": std::process::id(),
        "socket_path": socket_path.to_string_lossy(),
        "child_pid": serde_json::Value::Null,
        "state": "UnknownFutureState",
        "project_root": "/tmp/test-project",
        "cwd": "/tmp/test-cwd",
        "harness_id": "claude-code",
        "profile_id": "default",
        "started_at": chrono::Utc::now().to_rfc3339(),
        "display_name": "test",
        "pty_rows": 24u16,
        "pty_cols": 80u16,
        "kill_deadline_unix_ms": serde_json::Value::Null,
    });
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));
    {
        let mut f = std::fs::File::create(&sidecar_path).expect("AC-007-unknown: create sidecar");
        f.write_all(&serde_json::to_vec_pretty(&sidecar).expect("AC-007-unknown: serialize"))
            .expect("AC-007-unknown: write");
    }

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-007-unknown: rediscover_sessions must return Ok");

    assert!(
        !sidecar_path.exists(),
        "AC-007-unknown: unknown-state sidecar must be deleted; still at {:?}",
        sidecar_path
    );
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "AC-007-unknown: unknown-state session MUST NOT appear in registry"
    );
    assert!(
        !report.errors.is_empty(),
        "AC-007-unknown: unknown state must produce at least 1 error in the report"
    );
}

// ===========================================================================
// AC-009 / BC-2.08.004 PC-4 (RediscoveryReport shape)
// ===========================================================================

/// AC-009: `rediscover_sessions()` returns `Ok(RediscoveryReport)` with correct
/// `found_alive`, `found_dead`, and `errors` for a concrete 2-session scenario.
///
/// BC-2.08.004 postcondition 4.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_report_shape_mixed() {
    let tmp = tempfile::tempdir().expect("AC-009: tempdir");

    let id_alive = "00000000-0036-4000-a001-00000000000b";
    let socket_alive = short_socket_path("ac009-alive");
    let _ = std::fs::remove_file(&socket_alive);
    write_sidecar_v3(tmp.path(), id_alive, "Running", 0, &socket_alive, None);
    let mut done_alive = spawn_mock_session_host_attach(socket_alive.clone());

    let id_dead = "00000000-0036-4000-a001-00000000000c";
    let mut child = std::process::Command::new("true")
        .spawn()
        .expect("AC-009: spawn true");
    let dead_pid = child.id();
    let _ = child.wait(); // reap to avoid zombie; process exits immediately
    let socket_dead = short_socket_path("ac009-dead");
    write_sidecar_v3(tmp.path(), id_dead, "Running", dead_pid, &socket_dead, None);

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("AC-009: rediscover_sessions must return Ok");

    let _ = tokio::time::timeout(Duration::from_secs(6), done_alive.recv()).await;

    assert_eq!(
        report.found_alive, 1,
        "AC-009: found_alive=1; got {}",
        report.found_alive
    );
    assert_eq!(
        report.found_dead, 1,
        "AC-009: found_dead=1; got {}",
        report.found_dead
    );
    assert!(
        report.errors.is_empty(),
        "AC-009: errors=[]; got {:?}",
        report.errors
    );

    let _ = std::fs::remove_file(&socket_alive);
}

// ===========================================================================
// EC-167 / BC-2.08.004 EC-167 (empty runtime_dir → all-zeros)
// ===========================================================================

/// EC-167: empty `runtime_dir` → `RediscoveryReport { found_alive: 0, found_dead: 0, errors: [] }`.
///
/// BC-2.08.004 edge case EC-167.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_empty_runtime_dir() {
    let tmp = tempfile::tempdir().expect("EC-167: tempdir");
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("EC-167: rediscover_sessions must return Ok for empty dir");

    assert_eq!(
        report.found_alive, 0,
        "EC-167: found_alive=0; got {}",
        report.found_alive
    );
    assert_eq!(
        report.found_dead, 0,
        "EC-167: found_dead=0; got {}",
        report.found_dead
    );
    assert!(
        report.errors.is_empty(),
        "EC-167: errors=[]; got {:?}",
        report.errors
    );
}

// ===========================================================================
// EC-170 / BC-2.08.004 EC-170 (unreadable runtime_dir → RuntimeDirUnreadable)
// ===========================================================================

/// EC-170: `runtime_dir` does not exist → `Ok(RediscoveryReport { errors: [RuntimeDirUnreadable] })`.
///
/// BC-2.08.004 edge case EC-170 + postcondition 6.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_unreadable_runtime_dir() {
    let nonexistent = PathBuf::from("/tmp/monocle-test-s036-nonexistent-runtime-ec170");
    let _ = std::fs::remove_dir_all(&nonexistent);

    let (tx, _rx) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let entry = ClientEntry::new(tx);
    let subs: SubscriberList = Arc::new(Mutex::new(vec![entry]));
    let broker = make_broker(&subs);
    let spawner = Arc::new(MockSessionHostSpawner {
        spawn_result: None,
        fake_pid: 0,
    });
    let engine: Arc<dyn monocle_core::engine::EngineModule> =
        Arc::new(SucceedingMockEngineRediscovery);
    let mut manager = SessionManager::new(
        nonexistent,
        spawner,
        broker,
        engine,
        HookEndpointConfig::default(),
    );
    manager.with_peer_cred_verifier(Arc::new(FakePeerCredVerifier { allow: true }));

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("EC-170: rediscover_sessions must return Ok (never Err)");

    let has_unreadable = report
        .errors
        .iter()
        .any(|e| matches!(e, RediscoveryError::RuntimeDirUnreadable { .. }));
    assert!(
        has_unreadable,
        "EC-170: errors must contain RuntimeDirUnreadable; got {:?}",
        report.errors
    );
    assert_eq!(
        report.found_alive, 0,
        "EC-170: found_alive=0; got {}",
        report.found_alive
    );
    assert_eq!(
        report.found_dead, 0,
        "EC-170: found_dead=0; got {}",
        report.found_dead
    );
}

// ===========================================================================
// EC-155 (alive PID, socket missing → SIGTERM + GC)
// ===========================================================================

/// EC-155: session-host alive (`kill(pid, None)` OK) but socket file missing →
/// connect fails; SIGTERM (via seam); sidecar deleted; no `SessionEntry`.
///
/// BC-2.08.002 edge case EC-155.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_alive_pid_socket_missing() {
    let tmp = tempfile::tempdir().expect("EC-155: tempdir");
    let session_id = "00000000-0036-4000-a001-00000000000d";

    // Socket path: NOT created — simulates deleted socket.
    let socket_path = short_socket_path("ec155-missing");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Running",
        std::process::id(),
        &socket_path,
        None,
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    let (sigterm_tx, mut sigterm_rx) = mpsc::channel::<u32>(4);
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);
    manager.with_pid_sigterm_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigterm_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("EC-155: rediscover_sessions must return Ok");

    let sigterm_pid = tokio::time::timeout(Duration::from_millis(500), sigterm_rx.recv())
        .await
        .expect("EC-155: SIGTERM must be sent within 500ms")
        .expect("EC-155: sigterm channel closed");
    assert_eq!(
        sigterm_pid,
        std::process::id(),
        "EC-155: SIGTERM must target the session-host PID"
    );
    assert!(
        !sidecar_path.exists(),
        "EC-155: sidecar must be deleted; still at {:?}",
        sidecar_path
    );
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "EC-155: alive-PID/missing-socket MUST NOT appear in registry"
    );
    assert_eq!(
        report.found_dead, 1,
        "EC-155: alive-PID/missing-socket → found_dead=1; got {}",
        report.found_dead
    );
}

// ===========================================================================
// EC-156 (alive PID, non-responsive within 5s → SIGTERM + GC)
// ===========================================================================

/// EC-156: session-host alive + socket present but never responds within 5s →
/// SIGTERM (via seam); sidecar deleted; no `SessionEntry`.
///
/// Uses `start_paused = true` for time auto-advance.
///
/// BC-2.08.002 edge case EC-156.
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_004_rediscovery_non_responsive_within_5s() {
    let tmp = tempfile::tempdir().expect("EC-156: tempdir");
    let session_id = "00000000-0036-4000-a001-00000000000e";

    let socket_path = short_socket_path("ec156-stuck");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Running",
        std::process::id(),
        &socket_path,
        None,
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    let _accepted_rx = spawn_mock_session_host_silent(socket_path.clone());
    tokio::time::sleep(Duration::from_millis(10)).await;

    let (sigterm_tx, mut sigterm_rx) = mpsc::channel::<u32>(4);
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);
    manager.with_pid_sigterm_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigterm_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: hits todo!() — test fails here. With paused time, 5s auto-advances.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("EC-156: rediscover_sessions must return Ok for non-responsive host");

    let sigterm_pid = tokio::time::timeout(Duration::from_secs(1), sigterm_rx.recv())
        .await
        .expect("EC-156: SIGTERM must be sent after 5s timeout")
        .expect("EC-156: sigterm channel closed");
    assert_eq!(
        sigterm_pid,
        std::process::id(),
        "EC-156: SIGTERM must target the session-host PID"
    );
    assert!(
        !sidecar_path.exists(),
        "EC-156: sidecar must be deleted; still at {:?}",
        sidecar_path
    );
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "EC-156: non-responsive MUST NOT appear in registry"
    );
    assert_eq!(
        report.found_dead, 1,
        "EC-156: non-responsive → found_dead=1; got {}",
        report.found_dead
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// EC-159 (mixed alive + dead)
// ===========================================================================

/// EC-159: 1 alive + 1 dead → `found_alive: 1, found_dead: 1`.
///
/// BC-2.08.004 canonical test vector (2 sidecars; 1 alive, 1 dead).
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_mixed_alive_dead_ec159() {
    let tmp = tempfile::tempdir().expect("EC-159: tempdir");

    let id_alive = "00000000-0036-4000-a001-00000000000f";
    let socket_alive = short_socket_path("ec159-alive");
    let _ = std::fs::remove_file(&socket_alive);
    write_sidecar_v3(tmp.path(), id_alive, "Running", 0, &socket_alive, None);
    let mut done_alive = spawn_mock_session_host_attach(socket_alive.clone());

    let id_dead = "00000000-0036-4000-a001-000000000010";
    let mut child = std::process::Command::new("true")
        .spawn()
        .expect("EC-159: spawn true");
    let dead_pid = child.id();
    let _ = child.wait(); // reap to avoid zombie; process exits immediately
    let socket_dead = short_socket_path("ec159-dead");
    write_sidecar_v3(tmp.path(), id_dead, "Running", dead_pid, &socket_dead, None);

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("EC-159: rediscover_sessions must return Ok");

    let _ = tokio::time::timeout(Duration::from_secs(6), done_alive.recv()).await;

    assert_eq!(
        report.found_alive, 1,
        "EC-159: found_alive=1; got {}",
        report.found_alive
    );
    assert_eq!(
        report.found_dead, 1,
        "EC-159: found_dead=1; got {}",
        report.found_dead
    );
    assert!(
        report.errors.is_empty(),
        "EC-159: errors=[]; got {:?}",
        report.errors
    );

    let sessions = manager.session_list().await;
    assert!(
        sessions.iter().any(|s| s.session_id == id_alive),
        "EC-159: alive session must be in registry"
    );
    assert!(
        !sessions.iter().any(|s| s.session_id == id_dead),
        "EC-159: dead session MUST NOT be in registry"
    );

    let _ = std::fs::remove_file(&socket_alive);
}

// ===========================================================================
// AC-011 / BC-2.08.004 PC-6 (UDS bind blocked until re-discovery)
// ===========================================================================

/// AC-011: `rediscover_sessions()` completes before the UDS socket file appears.
///
/// Unit-level verification: `rediscover_sessions()` does NOT bind `monocle.sock` —
/// that is step 10 in `daemon_start_sequence`, which runs after step 8b.
///
/// BC-2.08.004 postcondition 6 + Invariant 1.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_completes_before_uds_bind() {
    let tmp = tempfile::tempdir().expect("AC-011: tempdir");
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    let sock_path = tmp.path().join("monocle.sock");
    assert!(
        !sock_path.exists(),
        "AC-011: monocle.sock must not exist before call"
    );

    // RED GATE: hits todo!() — test fails here.
    let _report = manager
        .rediscover_sessions()
        .await
        .expect("AC-011: rediscover_sessions must return Ok");

    assert!(
        !sock_path.exists(),
        "AC-011: monocle.sock must not exist after rediscover_sessions() returns; \
         UDS bind (step 10) must not be triggered by rediscover_sessions()"
    );
}

// ===========================================================================
// AC-001 / AC-015 / BC-2.08.002 (Integration: session survives daemon restart)
// ===========================================================================

/// AC-001 / AC-015 (integration): session persists across simulated daemon restart;
/// appears in `session_list()` with state Running after `rediscover_sessions()`.
///
/// BC-2.08.002 postconditions 1-7 (integration path).
#[tokio::test]
async fn test_BC_2_08_002_session_survives_daemon_graceful_restart() {
    let tmp = tempfile::tempdir().expect("AC-001/AC-015: tempdir");
    let session_id = "00000000-0036-4002-c001-000000000001";

    // Step 1: Write sidecar as daemon A left it (Running, before shutdown).
    let socket_path = short_socket_path("bc002-restart");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(tmp.path(), session_id, "Running", 0, &socket_path, None);

    // Step 2: Daemon A gone — no manager A.

    // Step 3: Mock session-host alive after restart.
    let mut done_rx = spawn_mock_session_host_attach(socket_path.clone());

    // Step 4: Daemon B with same runtime_dir.
    let (mut manager_b, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: hits todo!() — test fails here.
    let report = manager_b
        .rediscover_sessions()
        .await
        .expect("AC-001/AC-015: rediscover_sessions must return Ok after simulated restart");

    let _ = tokio::time::timeout(Duration::from_secs(6), done_rx.recv()).await;

    assert_eq!(
        report.found_alive, 1,
        "AC-001/AC-015: session rediscovered; found_alive=1; got {}",
        report.found_alive
    );
    assert_eq!(
        report.found_dead, 0,
        "AC-001/AC-015: found_dead=0; got {}",
        report.found_dead
    );

    let sessions = manager_b.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("AC-001/AC-015: session must be in InitialState after restart");
    assert_eq!(
        snap.state,
        SessionState::Running,
        "AC-001/AC-015: re-discovered session must be Running; got {:?}",
        snap.state
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// ADVERSARIAL PASS 1 CORRECTIONS — RED-GATE TESTS
//
// These tests encode spec behaviour surfaced by the adversarial review and
// must FAIL against the current implementation.  Each comment identifies
// the finding ID.
// ===========================================================================

// ---------------------------------------------------------------------------
// MED-002 — Detached path MUST connect + verify SO_PEERCRED (no Attach sent)
// ---------------------------------------------------------------------------

/// MED-002 (corrected): sidecar `state: "Detached"` with alive PID.
///
/// Per BC-2.08.004 PC-2b Detached, the daemon MUST:
///   1. Open a UDS connection to the session-host socket.
///   2. Verify SO_PEERCRED peer uid matches daemon uid.
///   3. Register `SessionEntry{Detached, host_conn: None}`.
///   4. NOT send `DaemonToHost::Attach`.
///   5. NOT emit `SessionStateChanged`.
///
/// The current implementation skips the UDS connect entirely — it registers
/// Detached sessions without connecting, so the "connection IS accepted"
/// assertion will FAIL.
///
/// This replaces the weaker over-assertion in
/// `test_BC_2_08_004_rediscovery_detached_no_attach_sent` which checked
/// "no connection made".  That test was wrong: the spec mandates a connect +
/// peercred verify before registration.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_detached_peercred_verified_no_attach() {
    let tmp = tempfile::tempdir().expect("MED-002: tempdir");
    let session_id = "00000000-0036-4000-a002-000000000001";

    let socket_path = short_socket_path("med002-det");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(tmp.path(), session_id, "Detached", 0, &socket_path, None);

    // Track: (a) whether a connection was accepted; (b) whether DaemonToHost::Attach
    // was received.
    let (connect_tx, mut connect_rx) = mpsc::channel::<()>(4);
    let (attach_tx, mut attach_rx) = mpsc::channel::<()>(4);
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("MED-002: mock bind");
        loop {
            match tokio::time::timeout(Duration::from_secs(5), listener.accept()).await {
                Ok(Ok((mut stream, _))) => {
                    let _ = connect_tx.send(()).await;
                    // Try to read a message — if Attach is sent, capture it.
                    let mut len_buf = [0u8; 4];
                    if stream.read_exact(&mut len_buf).await.is_ok() {
                        let len = u32::from_le_bytes(len_buf) as usize;
                        if len > 0 && len <= 65536 {
                            let mut body = vec![0u8; len];
                            if stream.read_exact(&mut body).await.is_ok() {
                                if let Ok(msg) = serde_json::from_slice::<
                                    monocle_ipc::types::DaemonToHost,
                                >(&body)
                                {
                                    if matches!(msg, monocle_ipc::types::DaemonToHost::Attach) {
                                        let _ = attach_tx.send(()).await;
                                    }
                                }
                            }
                        }
                    }
                    // Hold stream open briefly so the daemon's peercred verify
                    // can complete before the mock drops the connection.
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                _ => break,
            }
        }
    });
    // Give the mock time to bind before rediscover_sessions is called.
    tokio::time::sleep(Duration::from_millis(30)).await;

    let (mut manager, _subs, mut rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: current impl registers Detached without connecting —
    // the connection assertion below will FAIL.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("MED-002: rediscover_sessions must return Ok");

    // Assert 1: a UDS connection WAS accepted (peercred-verify step requires connect).
    let got_connect = tokio::time::timeout(Duration::from_millis(500), connect_rx.recv()).await;
    assert!(
        got_connect.is_ok() && got_connect.unwrap().is_some(),
        "MED-002: Detached re-discovery MUST connect to session-host socket for \
         SO_PEERCRED verification (BC-2.08.004 PC-2b Detached). \
         Current impl skips the connect."
    );

    // Assert 2: no DaemonToHost::Attach was sent over that connection.
    let got_attach = tokio::time::timeout(Duration::from_millis(200), attach_rx.recv()).await;
    assert!(
        got_attach.is_err() || got_attach.unwrap().is_none(),
        "MED-002: Detached re-discovery MUST NOT send DaemonToHost::Attach"
    );

    // Assert 3: SessionEntry{Detached} is registered.
    let sessions = manager.session_list().await;
    let snap = sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .expect("MED-002: Detached session must be in registry after peercred-verified connect");
    assert_eq!(
        snap.state,
        SessionState::Detached,
        "MED-002: state must be Detached; got {:?}",
        snap.state
    );

    // Assert 4: no SessionStateChanged emitted (F-P47-001).
    let drain_deadline = tokio::time::Instant::now() + Duration::from_millis(200);
    let mut state_changed_found = false;
    loop {
        match tokio::time::timeout_at(drain_deadline, rx.recv()).await {
            Ok(Some(ServerToClient::SessionStateChanged {
                session_id: ref sid,
                ..
            })) if sid == session_id => {
                state_changed_found = true;
                break;
            }
            Ok(Some(_)) => continue,
            _ => break,
        }
    }
    assert!(
        !state_changed_found,
        "MED-002: re-discovery of Detached MUST NOT emit SessionStateChanged"
    );

    assert_eq!(
        report.found_alive, 1,
        "MED-002: Detached + peercred OK → found_alive=1; got {}",
        report.found_alive
    );

    let _ = std::fs::remove_file(&socket_path);
}

/// MED-002 (peercred mismatch on Detached path): SO_PEERCRED fails →
/// WARN, non-responsive treatment (SIGTERM + delete sidecar + skip).
/// No SessionEntry registered.
///
/// BC-2.08.004 PC-2b Detached: "Verify SO_PEERCRED; if uid matches: register...
/// [implied: if mismatch: same non-responsive treatment as Running/Launching]".
/// AC-005: "If mismatch: same non-responsive treatment as above."
///
/// Current impl does no peercred check on Detached path — this test will
/// FAIL because the session will be registered despite mismatch.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_detached_peercred_mismatch_no_entry() {
    let tmp = tempfile::tempdir().expect("MED-002-mismatch: tempdir");
    let session_id = "00000000-0036-4000-a002-000000000002";

    let socket_path = short_socket_path("med002-mis");
    let _ = std::fs::remove_file(&socket_path);
    // PID = current process (alive); peercred verifier will DENY.
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Detached",
        std::process::id(),
        &socket_path,
        None,
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Mock listener: just accept; FakePeerCredVerifier(allow=false) will deny
    // before any message is exchanged.
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("MED-002-mismatch: mock bind");
        if let Ok((_stream, _)) = listener.accept().await {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (sigterm_tx, mut sigterm_rx) = mpsc::channel::<u32>(4);
    // FakePeerCredVerifier allow=false → deny.
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), false);
    manager.with_pid_sigterm_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigterm_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: current impl never connects for Detached — peercred check is
    // never reached, so the session is registered anyway despite allow=false.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("MED-002-mismatch: rediscover_sessions must return Ok");

    // Assert 1: SIGTERM sent for peercred-mismatched Detached session.
    let sigterm_pid = tokio::time::timeout(Duration::from_millis(500), sigterm_rx.recv()).await;
    assert!(
        sigterm_pid.is_ok() && sigterm_pid.unwrap().is_some(),
        "MED-002-mismatch: peercred mismatch on Detached path MUST send SIGTERM \
         (non-responsive treatment per AC-005 / BC-2.08.004 PC-2b Detached)"
    );

    // Assert 2: sidecar deleted.
    assert!(
        !sidecar_path.exists(),
        "MED-002-mismatch: peercred-mismatch Detached sidecar must be deleted; \
         still at {:?}",
        sidecar_path
    );

    // Assert 3: no SessionEntry registered.
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "MED-002-mismatch: peercred-mismatch Detached MUST NOT appear in registry"
    );

    // Assert 4: found_dead incremented (non-responsive treatment).
    assert_eq!(
        report.found_dead, 1,
        "MED-002-mismatch: peercred-mismatch Detached → found_dead=1; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "MED-002-mismatch: found_alive=0; got {}",
        report.found_alive
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ---------------------------------------------------------------------------
// HIGH-001 — Parallelism test that genuinely distinguishes concurrent from
//             sequential (1000ms per mock; sequential would exceed 5s)
// ---------------------------------------------------------------------------

/// HIGH-001 (strengthened parallelism): 8 session-hosts each with 1000ms
/// post-Attach latency.
///
/// Sequential execution: 8 × 1000ms = 8000ms >> 5s budget.
/// Concurrent execution: ~1000ms (all probes run in parallel via join_all).
///
/// Wall-clock bound: < 2000ms (impossible to satisfy with a sequential loop).
///
/// BC-2.08.004 postcondition 7 + Invariant 3.
///
/// The weaker version (`test_BC_2_08_004_rediscovery_parallelism_8_sessions`)
/// used 100ms mocks with `start_paused = true`; paused-time auto-advance
/// makes even a sequential loop pass that test trivially.  This test uses
/// real wall-clock measurement to distinguish.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_parallelism_8_sessions_sequential_would_exceed_5s() {
    let tmp = tempfile::tempdir().expect("HIGH-001: tempdir");

    let base_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let session_ids: Vec<String> = (0..8u8)
        .map(|i| format!("00000000-0036-4001-b001-{:012}", i))
        .collect();

    for (i, id) in session_ids.iter().enumerate() {
        let socket_path =
            std::path::PathBuf::from(format!("/tmp/s036-hi1-{}-{}.sock", i, base_nanos));
        let _ = std::fs::remove_file(&socket_path);
        write_sidecar_v3(tmp.path(), id, "Running", 0, &socket_path, None);
        let sock = socket_path.clone();
        tokio::spawn(async move {
            let _ = std::fs::remove_file(&sock);
            let listener = UnixListener::bind(&sock).expect("HIGH-001: mock bind");
            if let Ok((mut stream, _)) = listener.accept().await {
                // Consume Attach frame.
                let mut len_buf = [0u8; 4];
                let _ = stream.read_exact(&mut len_buf).await;
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut body = vec![0u8; len];
                let _ = stream.read_exact(&mut body).await;

                // 1000ms latency — sequential total would be 8000ms >> 5s budget.
                tokio::time::sleep(Duration::from_millis(1000)).await;

                let chunk = HostToDaemon::ScrollbackChunk {
                    rows: vec![],
                    chunk_seq: 0,
                };
                send_lp_frame(&mut stream, &chunk).await;
                let complete = HostToDaemon::ScrollbackDumpComplete {
                    total_chunks: 1,
                    cursor_row: 0,
                    cursor_col: 0,
                    pty_rows: 24,
                    pty_cols: 80,
                };
                send_lp_frame(&mut stream, &complete).await;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    // Give mock tasks time to bind before rediscover_sessions is called.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    let wall_start = std::time::Instant::now();

    // RED GATE: if rediscover_sessions() probes sequentially, each 1000ms mock
    // is awaited in turn → total ≥ 8000ms → the 2000ms assertion FAILS.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("HIGH-001: rediscover_sessions must return Ok for 8 sessions");

    let wall_elapsed = wall_start.elapsed();

    assert_eq!(
        report.found_alive, 8,
        "HIGH-001: all 8 sessions must be found alive; got {}",
        report.found_alive
    );
    assert_eq!(
        report.found_dead, 0,
        "HIGH-001: found_dead=0; got {}",
        report.found_dead
    );

    // Primary assertion: < 2s.  Parallel ~1s; sequential ~8s.
    assert!(
        wall_elapsed < Duration::from_millis(2000),
        "HIGH-001: 8 × 1000ms mocks MUST complete < 2s when probed in parallel \
         (sequential would take ~8s). Took {:?}. \
         Impl may be probing sequentially.",
        wall_elapsed
    );

    for i in 0..8usize {
        let _ = std::fs::remove_file(std::path::PathBuf::from(format!(
            "/tmp/s036-hi1-{}-{}.sock",
            i, base_nanos
        )));
    }
}

// ---------------------------------------------------------------------------
// BLOCKER-001 — Terminating path MUST verify SO_PEERCRED before sending Kill
// ---------------------------------------------------------------------------

/// BLOCKER-001: Terminating sidecar, NOT-elapsed deadline, alive PID,
/// SO_PEERCRED MISMATCH on the Kill connect.
///
/// Per BC-2.08.004 PC-2b Terminating: "Verify SO_PEERCRED; if uid matches:
/// (i) Check kill_deadline...".  If uid does NOT match, the daemon must NOT
/// send `DaemonToHost::Kill`, must apply non-responsive treatment
/// (SIGTERM + delete sidecar), and must NOT register a SessionEntry and
/// must NOT spawn a background watchdog.
///
/// Current impl skips SO_PEERCRED on the Terminating arm — it connects and
/// sends Kill regardless.  This test will FAIL because Kill IS sent (or the
/// session IS registered) despite the peercred mismatch.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_peercred_mismatch_no_kill() {
    let tmp = tempfile::tempdir().expect("BLOCKER-001: tempdir");
    let session_id = "00000000-0036-4000-a003-000000000001";

    let future_ms = unix_now_ms() + 10_000;
    let socket_path = short_socket_path("bl001-mismatch");
    // PID = current process (alive).
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(),
        &socket_path,
        Some(future_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Track: (a) whether Kill message arrived; (b) whether any connection occurred.
    let (kill_received_tx, mut kill_received_rx) = mpsc::channel::<()>(4);
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("BLOCKER-001: mock bind");
        loop {
            match tokio::time::timeout(Duration::from_secs(5), listener.accept()).await {
                Ok(Ok((mut stream, _))) => {
                    let mut len_buf = [0u8; 4];
                    if stream.read_exact(&mut len_buf).await.is_ok() {
                        let len = u32::from_le_bytes(len_buf) as usize;
                        if len > 0 && len <= 65536 {
                            let mut body = vec![0u8; len];
                            if stream.read_exact(&mut body).await.is_ok() {
                                if let Ok(msg) = serde_json::from_slice::<
                                    monocle_ipc::types::DaemonToHost,
                                >(&body)
                                {
                                    if matches!(msg, monocle_ipc::types::DaemonToHost::Kill) {
                                        let _ = kill_received_tx.send(()).await;
                                    }
                                }
                            }
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                _ => break,
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (sigterm_tx, mut sigterm_rx) = mpsc::channel::<u32>(4);
    // FakePeerCredVerifier allow=false → mismatch.
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), false);
    manager.with_pid_sigterm_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigterm_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: current impl skips SO_PEERCRED on Terminating path —
    // Kill will be sent anyway, and the session will be registered.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("BLOCKER-001: rediscover_sessions must return Ok");

    // Assert 1: no DaemonToHost::Kill was sent (peercred mismatch → skip Kill).
    let got_kill = tokio::time::timeout(Duration::from_millis(500), kill_received_rx.recv()).await;
    assert!(
        got_kill.is_err() || got_kill.unwrap().is_none(),
        "BLOCKER-001: peercred mismatch on Terminating path MUST NOT send \
         DaemonToHost::Kill (BC-2.08.004 PC-2b Terminating)"
    );

    // Assert 2: SIGTERM sent (non-responsive treatment).
    let sigterm_pid = tokio::time::timeout(Duration::from_millis(500), sigterm_rx.recv()).await;
    assert!(
        sigterm_pid.is_ok() && sigterm_pid.unwrap().is_some(),
        "BLOCKER-001: peercred mismatch on Terminating MUST send SIGTERM \
         (non-responsive treatment)"
    );

    // Assert 3: sidecar deleted.
    assert!(
        !sidecar_path.exists(),
        "BLOCKER-001: peercred-mismatch Terminating sidecar must be deleted; \
         still at {:?}",
        sidecar_path
    );

    // Assert 4: no SessionEntry registered.
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "BLOCKER-001: peercred-mismatch Terminating MUST NOT appear in registry"
    );

    // Assert 5: found_dead=1, found_alive=0 (no watchdog path entered).
    assert_eq!(
        report.found_dead, 1,
        "BLOCKER-001: peercred-mismatch Terminating → found_dead=1; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "BLOCKER-001: found_alive=0; got {}",
        report.found_alive
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ---------------------------------------------------------------------------
// BLOCKER-002 — Terminating path MUST probe liveness before watchdog path
// ---------------------------------------------------------------------------

/// BLOCKER-002: Terminating sidecar with a DEAD PID and a FUTURE
/// kill_deadline.
///
/// Per BC-2.08.004 PC-2b step (a): "probe liveness; if alive: state handling".
/// A dead PID must be GC'd immediately: sidecar deleted, `found_dead`
/// incremented, NO SessionEntry registered, NO background watchdog spawned.
///
/// Current implementation skips the liveness probe in the Terminating arm
/// and goes directly to the deadline check.  This test will FAIL because
/// the impl will either register a SessionEntry or attempt to connect to a
/// non-existent socket before failing.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_dead_pid_gc() {
    let tmp = tempfile::tempdir().expect("BLOCKER-002: tempdir");
    let session_id = "00000000-0036-4000-a003-000000000002";

    // Spawn a real short-lived process and reap it so the PID is definitively dead.
    let mut child = std::process::Command::new("true")
        .spawn()
        .expect("BLOCKER-002: spawn 'true'");
    let dead_pid = child.id();
    let _ = child.wait();

    let future_ms = unix_now_ms() + 10_000;
    let socket_path = short_socket_path("bl002-deadpid");
    // No socket bound — correct impl never reaches connect() because liveness
    // probe fires first and detects the dead PID.
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        dead_pid,
        &socket_path,
        Some(future_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: current impl skips liveness probe for Terminating.
    // With future_ms not elapsed and no socket, it will attempt connect (fail),
    // register SessionEntry{Terminating} + spawn watchdog.
    // The assertions below (no SessionEntry, found_dead=1) FAIL.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("BLOCKER-002: rediscover_sessions must return Ok");

    // Assert 1: sidecar deleted.
    assert!(
        !sidecar_path.exists(),
        "BLOCKER-002: dead-PID Terminating sidecar must be deleted; \
         still at {:?}",
        sidecar_path
    );

    // Assert 2: no SessionEntry registered.
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "BLOCKER-002: dead-PID Terminating MUST NOT appear in registry \
         (BC-2.08.004 PC-2b step a: liveness probe must precede state handling)"
    );

    // Assert 3: found_dead incremented.
    assert_eq!(
        report.found_dead, 1,
        "BLOCKER-002: dead-PID Terminating → found_dead=1; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "BLOCKER-002: found_alive=0; got {}",
        report.found_alive
    );
}

// ---------------------------------------------------------------------------
// HIGH-002 — Terminating watchdog MUST emit SessionStateChanged{Terminated}
//             + SessionListUpdate via broker on completion
// ---------------------------------------------------------------------------

/// HIGH-002 (deadline-elapsed path): Terminating sidecar, NOT-elapsed
/// deadline, alive PID, SO_PEERCRED OK → watchdog spawned.
/// Deadline elapses (virtual time advance) → watchdog fires SIGKILL, GCs
/// sidecar, AND emits `SessionStateChanged{Terminated}` + `SessionListUpdate`
/// via the broker.
///
/// Per BC-2.08.004 Invariant 1: "they only mutate their own session's state
/// asynchronously via the broker".
/// Per SS-daemon-wiring §3b emission table.
///
/// Current watchdog does NOT publish to the broker.
/// The broker-emission assertions will FAIL.
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_004_rediscovery_terminating_watchdog_deadline_emits_broker() {
    let tmp = tempfile::tempdir().expect("HIGH-002-deadline: tempdir");
    let session_id = "00000000-0036-4000-a004-000000000001";

    // Short future deadline: 500ms from now (virtual).
    let future_ms = unix_now_ms() + 500;
    let socket_path = short_socket_path("hi002-wdog");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(),
        &socket_path,
        Some(future_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Mock session-host: accept the Kill fire-and-forget connect, then hold open.
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("HIGH-002-deadline: mock bind");
        if let Ok((_stream, _)) = listener.accept().await {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(10)).await;

    let (sigkill_tx, mut sigkill_rx) = mpsc::channel::<u32>(4);
    let (mut manager, _subs, mut rx) = make_rediscovery_manager(tmp.path(), true);
    manager.with_pid_sigkill_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigkill_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // Run rediscover_sessions — watchdog spawned, returns immediately.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("HIGH-002-deadline: rediscover_sessions must return Ok");

    assert_eq!(
        report.found_alive, 1,
        "HIGH-002-deadline: Terminating + future deadline → found_alive=1; got {}",
        report.found_alive
    );

    // Advance virtual time past the deadline to fire the watchdog.
    tokio::time::advance(Duration::from_millis(600)).await;
    // Yield multiple times to allow the watchdog task to run.
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;

    // Assert A: SIGKILL was fired by the watchdog.
    let sigkill_pid = tokio::time::timeout(Duration::from_millis(500), sigkill_rx.recv()).await;
    assert!(
        sigkill_pid.is_ok() && sigkill_pid.unwrap().is_some(),
        "HIGH-002-deadline: watchdog must fire SIGKILL after deadline elapses"
    );

    // Allow settling.
    tokio::task::yield_now().await;

    // Assert B: sidecar deleted.
    assert!(
        !sidecar_path.exists(),
        "HIGH-002-deadline: watchdog must delete sidecar on SIGKILL; \
         still at {:?}",
        sidecar_path
    );

    // Assert C + D: SessionStateChanged{Terminated} + SessionListUpdate published.
    // RED GATE: current watchdog does not publish to broker → FAIL.
    let drain_deadline = tokio::time::Instant::now() + Duration::from_millis(200);
    let mut state_changed_terminated = false;
    let mut list_update_found = false;
    loop {
        match tokio::time::timeout_at(drain_deadline, rx.recv()).await {
            Ok(Some(ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state,
                ..
            })) if sid == session_id => {
                if new_state == SessionState::Terminated {
                    state_changed_terminated = true;
                }
            }
            Ok(Some(ServerToClient::SessionListUpdate { .. })) => {
                list_update_found = true;
            }
            Ok(Some(_)) => continue,
            _ => break,
        }
    }
    assert!(
        state_changed_terminated,
        "HIGH-002-deadline: watchdog MUST emit SessionStateChanged{{Terminated}} \
         to broker (BC-2.08.004 Invariant 1 + SS-daemon-wiring §3b)"
    );
    assert!(
        list_update_found,
        "HIGH-002-deadline: watchdog MUST emit SessionListUpdate to broker \
         after SIGKILL (SS-daemon-wiring §3b)"
    );

    let _ = std::fs::remove_file(&socket_path);
}

/// HIGH-002 (StateChanged::Terminated message path): Terminating sidecar,
/// NOT-elapsed deadline, alive PID, SO_PEERCRED OK → watchdog spawned.
/// Mock session-host sends `HostToDaemon::StateChanged` with new_state
/// Terminated → watchdog GCs sidecar AND emits `SessionStateChanged{Terminated}`
/// + `SessionListUpdate` via the broker.
///
/// BC-2.08.004 PC-2b Terminating step (iv): "If Terminated received → GC sidecar".
/// Invariant 1: "mutate their own session's state asynchronously via the broker".
///
/// Current watchdog does only GC + log — no broker emission → FAIL.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_watchdog_terminated_msg_emits_broker() {
    let tmp = tempfile::tempdir().expect("HIGH-002-msg: tempdir");
    let session_id = "00000000-0036-4000-a004-000000000002";

    let future_ms = unix_now_ms() + 30_000; // deadline far in the future
    let socket_path = short_socket_path("hi002-msg");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(),
        &socket_path,
        Some(future_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Mock session-host: accept Kill connect, then send StateChanged::Terminated
    // after a short delay (simulating graceful exit).
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("HIGH-002-msg: mock bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            // Read and discard the Kill message (fire-and-forget from daemon).
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut body = vec![0u8; len];
                let _ = stream.read_exact(&mut body).await;
            }

            // Reply: HostToDaemon::StateChanged { new_state: Terminated }.
            tokio::time::sleep(Duration::from_millis(100)).await;
            let terminated_msg = HostToDaemon::StateChanged {
                new_state: SessionState::Terminated,
                degraded_env: None,
            };
            let bytes = serde_json::to_vec(&terminated_msg).expect("HIGH-002-msg: serialize");
            let len_bytes = (bytes.len() as u32).to_le_bytes();
            let _ = stream.write_all(&len_bytes).await;
            let _ = stream.write_all(&bytes).await;
            let _ = stream.flush().await;
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (mut manager, _subs, mut rx) = make_rediscovery_manager(tmp.path(), true);

    // Run rediscover_sessions — watchdog spawned.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("HIGH-002-msg: rediscover_sessions must return Ok");

    assert_eq!(
        report.found_alive, 1,
        "HIGH-002-msg: Terminating + future deadline → found_alive=1; got {}",
        report.found_alive
    );

    // Wait for the watchdog to receive the Terminated message and process it.
    tokio::time::sleep(Duration::from_millis(800)).await;

    // Assert A: sidecar GC'd.
    assert!(
        !sidecar_path.exists(),
        "HIGH-002-msg: watchdog must GC sidecar on StateChanged::Terminated; \
         still at {:?}",
        sidecar_path
    );

    // Assert B + C: broker emissions.
    // RED GATE: current watchdog does not emit to broker → FAIL.
    let drain_deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    let mut state_changed_terminated = false;
    let mut list_update_found = false;
    loop {
        match tokio::time::timeout_at(drain_deadline, rx.recv()).await {
            Ok(Some(ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state,
                ..
            })) if sid == session_id => {
                if new_state == SessionState::Terminated {
                    state_changed_terminated = true;
                }
            }
            Ok(Some(ServerToClient::SessionListUpdate { .. })) => {
                list_update_found = true;
            }
            Ok(Some(_)) => continue,
            _ => break,
        }
    }
    assert!(
        state_changed_terminated,
        "HIGH-002-msg: watchdog MUST emit SessionStateChanged{{Terminated}} to \
         broker when session-host sends StateChanged::Terminated \
         (BC-2.08.004 Invariant 1)"
    );
    assert!(
        list_update_found,
        "HIGH-002-msg: watchdog MUST emit SessionListUpdate after \
         StateChanged::Terminated receipt (SS-daemon-wiring §3b)"
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// ADVERSARIAL PASS 2 CORRECTIONS — RED-GATE TESTS
//
// These tests encode spec behaviour surfaced by adversarial-pass-2 and must
// FAIL against the current implementation.  Each comment identifies the
// finding ID.
// ===========================================================================

// ---------------------------------------------------------------------------
// BLOCKER-001 (pass-2) — Terminating with null kill_deadline_unix_ms must
//                        take NOT-elapsed path with a NEW 12-second window
// ---------------------------------------------------------------------------

/// BLOCKER-001 (pass-2): Terminating sidecar with `kill_deadline_unix_ms = null`
/// (i.e., `None` / absent field) + alive PID + SO_PEERCRED OK.
///
/// Per AC-006 and BC-2.08.004 PC-2b: when `kill_deadline_unix_ms` is absent or
/// null, the daemon MUST assign a fresh 12-second window from `Instant::now()` and
/// take the NOT-elapsed watchdog path — NOT the immediate-SIGKILL path.
///
/// Current impl maps `None → unwrap_or(0)` which collapses to `0 <= current_unix_ms`
/// → elapsed → immediate SIGKILL.  This test will FAIL because SIGKILL fires and the
/// session is NOT registered as Terminating.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_null_deadline_new_window() {
    let tmp = tempfile::tempdir().expect("pass2-BLOCKER-001: tempdir");
    let session_id = "00000000-0036-4000-a005-000000000001";

    // Sidecar with kill_deadline_unix_ms = None (null in JSON).
    let socket_path = short_socket_path("bl001p2-null");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(), // alive PID
        &socket_path,
        None, // <-- key: null deadline
    );

    // Mock session-host: accept the Kill connect and hold open indefinitely.
    // Correct impl sends Kill fire-and-forget then spawns watchdog; we just
    // need a listening socket to let the connect succeed.
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("pass2-BLOCKER-001: mock bind");
        if let Ok((_stream, _)) = listener.accept().await {
            // Hold open long enough for assertions to run.
            tokio::time::sleep(Duration::from_secs(20)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Track whether SIGKILL fires.
    let (sigkill_tx, mut sigkill_rx) = mpsc::channel::<u32>(4);
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);
    manager.with_pid_sigkill_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigkill_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: current impl unwrap_or(0) → elapsed → SIGKILL fires immediately.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass2-BLOCKER-001: rediscover_sessions must return Ok");

    // Assert 1: SIGKILL must NOT fire during rediscover_sessions() itself.
    // The null deadline must produce a fresh window → NOT-elapsed path.
    let got_sigkill = tokio::time::timeout(Duration::from_millis(300), sigkill_rx.recv()).await;
    assert!(
        got_sigkill.is_err() || got_sigkill.unwrap().is_none(),
        "pass2-BLOCKER-001: null kill_deadline_unix_ms MUST take the NOT-elapsed \
         watchdog path (fresh 12s window); SIGKILL must NOT fire immediately. \
         Current impl unwrap_or(0) causes immediate SIGKILL. \
         AC-006 / BC-2.08.004 PC-2b."
    );

    // Assert 2: session must be registered as Terminating (watchdog path taken).
    let sessions = manager.session_list().await;
    assert!(
        sessions.iter().any(|s| s.session_id == session_id),
        "pass2-BLOCKER-001: null-deadline Terminating MUST be registered in registry \
         (watchdog path). Current impl GCs immediately."
    );

    // Assert 3: found_alive=1 (watchdog path → counted as alive).
    assert_eq!(
        report.found_alive, 1,
        "pass2-BLOCKER-001: null deadline → found_alive=1 (watchdog path); got {}",
        report.found_alive
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ---------------------------------------------------------------------------
// BLOCKER-002 (pass-2) — Terminating watchdog: socket close without Terminated
//                        msg before deadline must still fire SIGKILL + GC + broker
// ---------------------------------------------------------------------------

/// BLOCKER-002 (pass-2): Terminating sidecar, NOT-elapsed deadline, alive PID,
/// SO_PEERCRED OK → watchdog spawned.  The mock session-host CLOSES the Kill
/// socket WITHOUT sending `HostToDaemon::StateChanged{Terminated}`, BEFORE the
/// deadline.  At the deadline, the watchdog must still fire SIGKILL + GC sidecar +
/// emit `SessionStateChanged{Terminated}` then `SessionListUpdate` via broker.
///
/// Per BC-2.08.004 PC-2b Terminating step (iv).
///
/// Current impl: when the socket closes, `read_exact` returns Err → loop breaks →
/// `terminated_by_msg` future completes with `false` → the `got_terminated` select!
/// arm resolves with `got_terminated == false` → the arm does nothing and exits
/// ("fall through to deadline path" comment but no code follows) — SIGKILL and
/// broker emissions are skipped entirely.
///
/// This test uses real wall-clock time with a short 300ms deadline to make the
/// socket-close race deterministic: the mock drops the stream immediately after
/// consuming the Kill frame (no sleep), so `terminated_by_msg` resolves with
/// `false` before the deadline timer fires.  After the deadline elapses, the
/// watchdog must STILL fire SIGKILL + broker.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_watchdog_socket_close_before_terminated() {
    let tmp = tempfile::tempdir().expect("pass2-BLOCKER-002: tempdir");
    let session_id = "00000000-0036-4000-a005-000000000002";

    // Short future deadline: 600ms from now (real wall-clock).
    let future_ms = unix_now_ms() + 600;
    let socket_path = short_socket_path("bl002p2-close");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(), // alive PID
        &socket_path,
        Some(future_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Mock session-host: accept, read Kill frame, then DROP the stream IMMEDIATELY
    // (no sleep) — ensures the socket-close resolves before the 600ms deadline.
    // The watchdog's `terminated_by_msg` future will complete with `false` early,
    // and the `got_terminated` select! arm fires.  The bug: nothing happens after.
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("pass2-BLOCKER-002: mock bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            // Read and discard the Kill frame.
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut body = vec![0u8; len];
                let _ = stream.read_exact(&mut body).await;
            }
            // Drop immediately — socket closed, NO Terminated sent.
            // stream goes out of scope here.
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (sigkill_tx, mut sigkill_rx) = mpsc::channel::<u32>(4);
    let (mut manager, _subs, mut rx) = make_rediscovery_manager(tmp.path(), true);
    manager.with_pid_sigkill_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigkill_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // Run rediscover_sessions — watchdog spawned, returns immediately (found_alive=1).
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass2-BLOCKER-002: rediscover_sessions must return Ok");

    assert_eq!(
        report.found_alive, 1,
        "pass2-BLOCKER-002: Terminating + future deadline → found_alive=1; got {}",
        report.found_alive
    );

    // Wait for the deadline to elapse and the watchdog to react (600ms + margin).
    // In the CORRECT impl, SIGKILL fires here.  In the BUGGY impl, the watchdog
    // already exited when the socket closed, so SIGKILL never fires.
    let got_sigkill = tokio::time::timeout(Duration::from_millis(1200), sigkill_rx.recv()).await;
    assert!(
        got_sigkill.is_ok() && got_sigkill.unwrap().is_some(),
        "pass2-BLOCKER-002: watchdog must fire SIGKILL at deadline even when the \
         session-host socket closes before sending Terminated. \
         Current impl: when socket closes early, terminated_by_msg completes with \
         false → got_terminated arm exits without scheduling deadline SIGKILL. \
         BC-2.08.004 PC-2b Terminating step (iv)."
    );

    tokio::task::yield_now().await;
    tokio::task::yield_now().await;

    // Assert B: sidecar GC'd.
    assert!(
        !sidecar_path.exists(),
        "pass2-BLOCKER-002: watchdog must delete sidecar on SIGKILL; still at {:?}",
        sidecar_path
    );

    // Assert C + D: broker emissions — SessionStateChanged{Terminated} then
    // SessionListUpdate.  RED GATE: current impl never reaches this code.
    let drain_deadline = tokio::time::Instant::now() + Duration::from_millis(300);
    let mut state_changed_terminated = false;
    let mut list_update_found = false;
    loop {
        match tokio::time::timeout_at(drain_deadline, rx.recv()).await {
            Ok(Some(monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state,
                ..
            })) if sid == session_id => {
                if new_state == monocle_ipc::types::SessionState::Terminated {
                    state_changed_terminated = true;
                }
            }
            Ok(Some(monocle_ipc::types::ServerToClient::SessionListUpdate { .. })) => {
                list_update_found = true;
            }
            Ok(Some(_)) => continue,
            _ => break,
        }
    }
    assert!(
        state_changed_terminated,
        "pass2-BLOCKER-002: watchdog must emit SessionStateChanged{{Terminated}} to \
         broker when deadline fires after socket-close-without-Terminated. \
         BC-2.08.004 PC-2b + SS-daemon-wiring §3b."
    );
    assert!(
        list_update_found,
        "pass2-BLOCKER-002: watchdog must emit SessionListUpdate to broker after \
         SIGKILL at deadline. SS-daemon-wiring §3b."
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ---------------------------------------------------------------------------
// HIGH-001 (pass-2) — SO_PEERCRED pid mismatch (same uid, different pid) rejected
// ---------------------------------------------------------------------------

/// HIGH-001 (pass-2): A session-host socket whose PEER pid does NOT match the
/// sidecar's recorded pid (same uid, different pid — stale sidecar / PID-reuse /
/// spoof attempt).  Daemon MUST reject: log WARN, NOT register the session,
/// SIGTERM the sidecar pid, delete the sidecar.
///
/// Per SS-session-manager.md §Per-session UDS security item 2.
///
/// IMPLEMENTER NOTE: The current `FakePeerCredVerifier` only accepts a boolean
/// `allow` flag and cannot express "same uid, different pid".  This test uses a
/// `FakePeerCredVerifierWithPidMismatch` struct defined in this module (test-only).
/// For this test to be COMPLETE, the implementer must:
///   1. Add a `peer_pid` field to the peer-cred result in `PeerCredVerifier::verify()`.
///   2. Expose the sidecar's recorded pid to the verifier call site.
///   3. Compare peer_pid against sidecar pid and reject on mismatch.
///
/// Until that seam exists, `FakePeerCredVerifierWithPidMismatch::verify()` returns
/// `Ok(())` (uid matches) so the current implementation DOES register the session.
/// The "session NOT registered" assertion will FAIL, proving the pid-mismatch
/// check is absent.
struct FakePeerCredVerifierWithPidMismatch;

impl super::PeerCredVerifier for FakePeerCredVerifierWithPidMismatch {
    fn verify(&self, _stream: &tokio::net::UnixStream) -> Result<(), super::SessionError> {
        // UID check passes (matching uid) — the PID mismatch is in verify_with_sidecar_pid.
        Ok(())
    }

    fn verify_with_sidecar_pid(
        &self,
        _stream: &tokio::net::UnixStream,
        _sidecar_pid: u32,
    ) -> Result<(), super::PeerCredMismatch> {
        // Simulates a peer whose actual pid differs from the sidecar-recorded pid
        // (matching uid, mismatched pid — stale sidecar / PID-reuse / spoof attempt).
        // Per SS-session-manager §Per-session UDS security item 2.
        Err(super::PeerCredMismatch {
            peer_pid: None,
            reason: "FakePeerCredVerifierWithPidMismatch: simulated PID mismatch".into(),
        })
    }
}

#[tokio::test]
async fn test_BC_2_08_004_rediscovery_peercred_pid_mismatch_rejected() {
    let tmp = tempfile::tempdir().expect("pass2-HIGH-001: tempdir");
    let session_id = "00000000-0036-4000-a005-000000000003";

    // Sidecar records pid = 100 (almost certainly not a real running process).
    let sidecar_pid: u32 = 100;
    let socket_path = short_socket_path("hi001p2-pidmism");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Running",
        sidecar_pid,
        &socket_path,
        None,
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Spawn a mock session-host whose actual peer pid is std::process::id(),
    // which differs from sidecar_pid=100 on any realistic system.
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("pass2-HIGH-001: mock bind");
        if let Ok((_stream, _)) = listener.accept().await {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (sigterm_tx, mut sigterm_rx) = mpsc::channel::<u32>(4);

    // Build manager manually to inject FakePeerCredVerifierWithPidMismatch.
    let (tx, _rx_ch) = mpsc::channel::<monocle_ipc::types::ServerToClient>(
        monocle_ipc::server::CLIENT_CHANNEL_CAPACITY,
    );
    let entry = monocle_ipc::server::ClientEntry::new(tx);
    let subs: monocle_ipc::server::SubscriberList = Arc::new(tokio::sync::Mutex::new(vec![entry]));
    let broker = Arc::new(Arc::clone(&subs));
    let spawner = Arc::new(super::MockSessionHostSpawner {
        spawn_result: None,
        fake_pid: 0,
    });
    let engine: Arc<dyn monocle_core::engine::EngineModule> =
        Arc::new(SucceedingMockEngineRediscovery);
    let mut manager = super::SessionManager::new(
        tmp.path().to_path_buf(),
        spawner,
        broker,
        engine,
        super::HookEndpointConfig::default(),
    );
    // IMPLEMENTER: extend PeerCredVerifier to expose peer_pid; update
    // FakePeerCredVerifierWithPidMismatch to return Err on pid mismatch.
    manager.with_peer_cred_verifier(Arc::new(FakePeerCredVerifierWithPidMismatch));
    manager.with_pid_sigterm_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigterm_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: current impl has no pid-mismatch check — session will be registered.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass2-HIGH-001: rediscover_sessions must return Ok");

    // Assert 1: session must NOT be registered (pid mismatch → reject).
    // FAILS NOW: current impl registers it because pid is not checked.
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "pass2-HIGH-001: session-host with PEER pid != sidecar pid MUST NOT be \
         registered (SS-session-manager.md §Per-session UDS security item 2). \
         Current impl has no pid-mismatch check — session is registered via \
         uid-only check (FakePeerCredVerifierWithPidMismatch returns Ok)."
    );

    // Assert 2: SIGTERM sent for the sidecar pid.
    // NOTE: This assertion may not be reached if Assert 1 fires first.
    let got_sigterm = tokio::time::timeout(Duration::from_millis(500), sigterm_rx.recv()).await;
    assert!(
        got_sigterm.is_ok() && got_sigterm.unwrap().is_some(),
        "pass2-HIGH-001: pid-mismatch detection must send SIGTERM to the sidecar pid"
    );

    // Assert 3: sidecar deleted.
    assert!(
        !sidecar_path.exists(),
        "pass2-HIGH-001: sidecar must be deleted on pid-mismatch; still at {:?}",
        sidecar_path
    );

    // Assert 4: found_dead=1 (mismatch → non-responsive treatment).
    assert_eq!(
        report.found_dead, 1,
        "pass2-HIGH-001: pid-mismatch → found_dead=1; got {}",
        report.found_dead
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// ADVERSARIAL PASS 3 CORRECTIONS — RED-GATE TESTS
//
// These tests encode security-perimeter gaps surfaced by adversarial-pass-3
// and must FAIL against the current implementation.  Each comment identifies
// the finding ID.
// ===========================================================================

// ---------------------------------------------------------------------------
// Seam support: FakePeerCredVerifierWithPeerPid
//
// This verifier simulates "uid matches, peer pid is distinct from sidecar pid"
// — i.e., a PID-reuse / spoof scenario where the connecting process belongs to
// the correct uid but is NOT the sidecar-recorded session-host.
//
// IMPLEMENTER CONTRACT:
//   The `verify_with_sidecar_pid` method must return a rich error type that
//   exposes the socket peer pid so the call site can SIGTERM both pids.
//
//   Exact trait method signature the implementer must provide:
//
//     fn verify_with_sidecar_pid(
//         &self,
//         stream: &tokio::net::UnixStream,
//         sidecar_pid: u32,
//     ) -> Result<(), super::PeerCredMismatch>
//
//   where `super::PeerCredMismatch` is defined as:
//
//     pub struct PeerCredMismatch {
//         pub peer_pid: Option<u32>,
//         pub reason: String,
//     }
//     impl std::fmt::Display for PeerCredMismatch { ... }
//
//   `RealPeerCredVerifier::verify_with_sidecar_pid` must populate
//   `PeerCredMismatch::peer_pid` from `stream.peer_cred()?.pid()` so that the
//   call site can SIGTERM both pids on a mismatch.
//
//   The default `verify_with_sidecar_pid` in the trait must NOT silently
//   delegate to `verify()` ignoring `sidecar_pid` — it must call the full
//   pid-aware check.  `FakePeerCredVerifier` must override
//   `verify_with_sidecar_pid` to honour the pid argument:
//   - `allow == true`:  return Ok(()) regardless of pid (UID+PID both pass).
//   - `allow == false`: return Err (UID fails, never reaches pid check).
//
//   `FakePeerCredVerifierWithPeerPid` below performs the proper cross-check:
//   UID always passes; pid check compares `injected_peer_pid` vs `sidecar_pid`.
// ---------------------------------------------------------------------------

/// Test-only verifier: uid always matches; pid check compares an injected
/// "socket peer pid" against the sidecar's recorded pid.
///
/// Used for HIGH-001 (pass-3) and HIGH-002 (pass-3) to exercise the
/// mandatory per-session PID cross-check without forking a real process
/// (SS-session-manager §Per-session UDS security item 2).
#[cfg(any(test, feature = "test-utils"))]
struct FakePeerCredVerifierWithPeerPid {
    /// The pid the verifier will claim SO_PEERCRED reports (the "socket peer pid").
    /// When this does NOT equal `sidecar_pid`, `verify_with_sidecar_pid` must
    /// return Err carrying `peer_pid = Some(injected_peer_pid)`.
    pub injected_peer_pid: u32,
}

#[cfg(any(test, feature = "test-utils"))]
impl super::PeerCredVerifier for FakePeerCredVerifierWithPeerPid {
    fn verify(&self, _stream: &tokio::net::UnixStream) -> Result<(), super::SessionError> {
        // UID check: always passes (simulates correct uid ownership).
        Ok(())
    }

    fn verify_with_sidecar_pid(
        &self,
        _stream: &tokio::net::UnixStream,
        sidecar_pid: u32,
    ) -> Result<(), super::PeerCredMismatch> {
        // PID cross-check: compare injected peer_pid against sidecar_pid.
        // Return PeerCredMismatch carrying peer_pid so the call site can
        // SIGTERM BOTH the sidecar pid and the socket peer pid (HIGH-002 fix).
        if self.injected_peer_pid != sidecar_pid {
            Err(super::PeerCredMismatch {
                peer_pid: Some(self.injected_peer_pid),
                reason: format!(
                    "FakePeerCredVerifierWithPeerPid: peer_pid={} != sidecar_pid={} \
                     (PID cross-check mismatch)",
                    self.injected_peer_pid, sidecar_pid
                ),
            })
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// HIGH-001 (pass-3): Detached path MUST use verify_with_sidecar_pid (not uid-only verify)
// ---------------------------------------------------------------------------

/// HIGH-001 (pass-3): Detached sidecar, alive PID, uid matches but socket PEER pid
/// != sidecar.pid → daemon MUST NOT register the session; MUST WARN; MUST SIGTERM;
/// MUST delete sidecar.
///
/// Current impl calls `verify()` (uid-only) on the Detached path — it passes
/// because uid matches, so the session IS registered despite the pid mismatch.
/// The "NOT registered" assertion will FAIL.
///
/// The fix: the Detached connect path must call `verify_with_sidecar_pid(stream, data.pid)`
/// instead of `verify(stream)`.
///
/// SS-session-manager §Per-session UDS security item 2 + BC-2.08.004 AC-004/005.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_detached_pid_mismatch_rejected() {
    let tmp = tempfile::tempdir().expect("pass3-HIGH-001-detached: tempdir");
    let session_id = "00000000-0036-4000-a006-000000000001";

    let socket_path = short_socket_path("p3hi001-det");
    let _ = std::fs::remove_file(&socket_path);

    // PID 9999 is almost certainly dead, but the sidecar state is "Detached"
    // and the Detached path calls kill(pid, None) for a liveness check.
    // We need an ALIVE pid for the liveness check to pass so the code reaches
    // the UDS connect + verify step.  Use pid=0 (a magic value that the
    // implementation maps to "always alive" for mock tests, as seen in AC-004),
    // or use our own pid.  Use std::process::id() for the sidecar pid so the
    // liveness probe passes, but configure the fake verifier to inject a
    // DIFFERENT peer_pid.
    let alive_sidecar_pid = std::process::id();
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Detached",
        alive_sidecar_pid,
        &socket_path,
        None,
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Peer pid injected by the fake verifier: current process pid + 1 (distinct
    // from alive_sidecar_pid, creating a mismatch).
    let injected_peer_pid = alive_sidecar_pid.wrapping_add(1);

    // Mock session-host: just accept and hold open (verifier fires before any msg).
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("pass3-HIGH-001-detached: mock bind");
        if let Ok((_stream, _)) = listener.accept().await {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (sigterm_tx, mut sigterm_rx) = mpsc::channel::<u32>(8);
    let (tx, _rx_ch) = mpsc::channel::<monocle_ipc::types::ServerToClient>(
        monocle_ipc::server::CLIENT_CHANNEL_CAPACITY,
    );
    let entry = monocle_ipc::server::ClientEntry::new(tx);
    let subs: monocle_ipc::server::SubscriberList = Arc::new(tokio::sync::Mutex::new(vec![entry]));
    let broker = Arc::new(Arc::clone(&subs));
    let spawner = Arc::new(super::MockSessionHostSpawner {
        spawn_result: None,
        fake_pid: 0,
    });
    let engine: Arc<dyn monocle_core::engine::EngineModule> =
        Arc::new(SucceedingMockEngineRediscovery);
    let mut manager = super::SessionManager::new(
        tmp.path().to_path_buf(),
        spawner,
        broker,
        engine,
        super::HookEndpointConfig::default(),
    );
    manager.with_peer_cred_verifier(Arc::new(FakePeerCredVerifierWithPeerPid {
        injected_peer_pid,
    }));
    let sigterm_tx_clone = sigterm_tx.clone();
    manager.with_pid_sigterm_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigterm_tx_clone.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: current Detached path calls verify() which returns Ok(())
    // from this verifier (uid matches).  The pid mismatch is never checked.
    // The session IS registered → "NOT registered" assertion FAILS.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass3-HIGH-001-detached: rediscover_sessions must return Ok");

    // Assert 1: session MUST NOT be registered (pid mismatch → reject).
    // FAILS NOW: Detached path uses uid-only verify() → session is registered.
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "pass3-HIGH-001-detached: Detached session with peer_pid != sidecar.pid \
         MUST NOT be registered (SS-session-manager §Per-session UDS security \
         item 2).  Current impl uses uid-only verify() on the Detached path — \
         pid cross-check is skipped entirely."
    );

    // Assert 2: SIGTERM sent (non-responsive treatment for pid mismatch).
    let got_sigterm = tokio::time::timeout(Duration::from_millis(500), sigterm_rx.recv()).await;
    assert!(
        got_sigterm.is_ok() && got_sigterm.unwrap().is_some(),
        "pass3-HIGH-001-detached: pid mismatch on Detached path MUST send SIGTERM \
         (belt-and-suspenders; SS-session-manager §Per-session UDS security item 2)"
    );

    // Assert 3: sidecar deleted.
    assert!(
        !sidecar_path.exists(),
        "pass3-HIGH-001-detached: sidecar must be deleted on pid-mismatch; \
         still at {:?}",
        sidecar_path
    );

    // Assert 4: found_dead=1, found_alive=0.
    assert_eq!(
        report.found_dead, 1,
        "pass3-HIGH-001-detached: pid-mismatch Detached → found_dead=1; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "pass3-HIGH-001-detached: found_alive=0; got {}",
        report.found_alive
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ---------------------------------------------------------------------------
// HIGH-001 (pass-3, Terminating arm): Terminating path MUST use
//   verify_with_sidecar_pid (not uid-only verify)
// ---------------------------------------------------------------------------

/// HIGH-001 (pass-3, Terminating): Terminating sidecar, NOT-elapsed deadline,
/// alive PID, uid matches but socket PEER pid != sidecar.pid → daemon MUST NOT
/// send Kill, MUST NOT register, MUST NOT spawn watchdog; MUST WARN, SIGTERM,
/// delete sidecar.
///
/// Current impl calls `verify()` (uid-only) on the Terminating not-elapsed path.
/// This verifier returns Ok for uid → Kill IS sent and the session IS registered.
/// The assertions below FAIL.
///
/// The fix: the Terminating connect path must call
/// `verify_with_sidecar_pid(stream, data.pid)` instead of `verify(stream)`.
///
/// SS-session-manager §Per-session UDS security item 2 + BC-2.08.004 AC-006/008.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_pid_mismatch_no_kill() {
    let tmp = tempfile::tempdir().expect("pass3-HIGH-001-terminating: tempdir");
    let session_id = "00000000-0036-4000-a006-000000000002";

    let future_ms = unix_now_ms() + 10_000;
    let alive_sidecar_pid = std::process::id();
    // injected_peer_pid is distinct from alive_sidecar_pid → mismatch.
    let injected_peer_pid = alive_sidecar_pid.wrapping_add(2);

    let socket_path = short_socket_path("p3hi001-term");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        alive_sidecar_pid,
        &socket_path,
        Some(future_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Track: Kill message and SIGTERM.
    let (kill_received_tx, mut kill_received_rx) = mpsc::channel::<()>(4);
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener =
            UnixListener::bind(&sock_clone).expect("pass3-HIGH-001-terminating: mock bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                let len = u32::from_le_bytes(len_buf) as usize;
                if len > 0 && len <= 65536 {
                    let mut body = vec![0u8; len];
                    if stream.read_exact(&mut body).await.is_ok() {
                        if let Ok(msg) =
                            serde_json::from_slice::<monocle_ipc::types::DaemonToHost>(&body)
                        {
                            if matches!(msg, monocle_ipc::types::DaemonToHost::Kill) {
                                let _ = kill_received_tx.send(()).await;
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (sigterm_tx, mut sigterm_rx) = mpsc::channel::<u32>(8);
    let (tx, _rx_ch) = mpsc::channel::<monocle_ipc::types::ServerToClient>(
        monocle_ipc::server::CLIENT_CHANNEL_CAPACITY,
    );
    let entry = monocle_ipc::server::ClientEntry::new(tx);
    let subs: monocle_ipc::server::SubscriberList = Arc::new(tokio::sync::Mutex::new(vec![entry]));
    let broker = Arc::new(Arc::clone(&subs));
    let spawner = Arc::new(super::MockSessionHostSpawner {
        spawn_result: None,
        fake_pid: 0,
    });
    let engine: Arc<dyn monocle_core::engine::EngineModule> =
        Arc::new(SucceedingMockEngineRediscovery);
    let mut manager = super::SessionManager::new(
        tmp.path().to_path_buf(),
        spawner,
        broker,
        engine,
        super::HookEndpointConfig::default(),
    );
    manager.with_peer_cred_verifier(Arc::new(FakePeerCredVerifierWithPeerPid {
        injected_peer_pid,
    }));
    let sigterm_tx_clone = sigterm_tx.clone();
    manager.with_pid_sigterm_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigterm_tx_clone.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: current Terminating not-elapsed path calls verify() which
    // returns Ok(()) from this verifier.  Kill IS sent; session IS registered.
    // The "no Kill" and "NOT registered" assertions FAIL.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass3-HIGH-001-terminating: rediscover_sessions must return Ok");

    // Assert 1: Kill MUST NOT be sent (pid mismatch → reject before Kill).
    // FAILS NOW: current impl uses uid-only verify() → Kill IS sent.
    let got_kill = tokio::time::timeout(Duration::from_millis(400), kill_received_rx.recv()).await;
    assert!(
        got_kill.is_err() || got_kill.unwrap().is_none(),
        "pass3-HIGH-001-terminating: Terminating session with peer_pid != sidecar.pid \
         MUST NOT send DaemonToHost::Kill (SS-session-manager §Per-session UDS \
         security item 2).  Current impl uses uid-only verify() → Kill IS sent."
    );

    // Assert 2: SIGTERM sent (non-responsive treatment).
    let got_sigterm = tokio::time::timeout(Duration::from_millis(500), sigterm_rx.recv()).await;
    assert!(
        got_sigterm.is_ok() && got_sigterm.unwrap().is_some(),
        "pass3-HIGH-001-terminating: pid mismatch on Terminating path MUST send \
         SIGTERM (belt-and-suspenders; SS-session-manager §Per-session UDS security item 2)"
    );

    // Assert 3: sidecar deleted.
    assert!(
        !sidecar_path.exists(),
        "pass3-HIGH-001-terminating: sidecar must be deleted on pid-mismatch; \
         still at {:?}",
        sidecar_path
    );

    // Assert 4: no SessionEntry registered.
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "pass3-HIGH-001-terminating: pid-mismatch Terminating MUST NOT appear in \
         registry; watchdog MUST NOT be spawned"
    );

    // Assert 5: found_dead=1, found_alive=0 (no watchdog path entered).
    assert_eq!(
        report.found_dead, 1,
        "pass3-HIGH-001-terminating: pid-mismatch Terminating → found_dead=1; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "pass3-HIGH-001-terminating: found_alive=0; got {}",
        report.found_alive
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ---------------------------------------------------------------------------
// HIGH-002 (pass-3): SIGTERM must target BOTH sidecar pid AND socket peer pid
// ---------------------------------------------------------------------------

/// HIGH-002 (pass-3): On a PID mismatch (any connect path), the daemon MUST
/// SIGTERM BOTH the sidecar's recorded pid AND the socket's peer pid.
///
/// Per SS-session-manager §Per-session UDS security item 2: "SIGTERM BOTH the
/// sidecar pid AND the socket peer pid (belt-and-suspenders)".
///
/// This test exercises the Running/Launching probe path (which calls
/// `verify_with_sidecar_pid`) with an injected peer_pid distinct from
/// sidecar_pid.  It asserts that SIGTERM is sent to BOTH pids.
///
/// IMPLEMENTER CONTRACT: `verify_with_sidecar_pid` must return a
/// `PeerCredMismatch { peer_pid: Some(N), reason: ... }` error so the call
/// site has the peer pid available to SIGTERM it.  The call site must then
/// send SIGTERM to BOTH `data.pid` (sidecar) and `mismatch.peer_pid`
/// (socket peer).
///
/// Current impl: `verify_with_sidecar_pid` returns `Result<(), SessionError>`
/// — the SessionError carries no peer_pid.  The call site SIGTERMs only
/// `data.pid`.  The "peer pid also receives SIGTERM" assertion FAILS.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_pid_mismatch_sigterms_both_pids() {
    let tmp = tempfile::tempdir().expect("pass3-HIGH-002: tempdir");
    let session_id = "00000000-0036-4000-a006-000000000003";

    // Use a sidecar_pid that is alive for the liveness probe (std::process::id())
    // but give the fake verifier a DIFFERENT injected_peer_pid.
    let alive_sidecar_pid = std::process::id();
    // Choose a peer_pid that is distinct and unlikely to be a real running process
    // so it doesn't interfere with test cleanup.
    let injected_peer_pid = alive_sidecar_pid.wrapping_add(7777);

    let socket_path = short_socket_path("p3hi002-both");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Running",
        alive_sidecar_pid,
        &socket_path,
        None,
    );

    // Mock session-host: accept and hold open (verifier fires immediately on connect).
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener = UnixListener::bind(&sock_clone).expect("pass3-HIGH-002: mock bind");
        if let Ok((_stream, _)) = listener.accept().await {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Capture ALL pids that receive SIGTERM (not just one).
    let sigterm_pids: Arc<tokio::sync::Mutex<Vec<u32>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let sigterm_pids_clone = Arc::clone(&sigterm_pids);
    let (sigterm_done_tx, mut sigterm_done_rx) = mpsc::channel::<()>(8);

    let (tx, _rx_ch) = mpsc::channel::<monocle_ipc::types::ServerToClient>(
        monocle_ipc::server::CLIENT_CHANNEL_CAPACITY,
    );
    let entry = monocle_ipc::server::ClientEntry::new(tx);
    let subs: monocle_ipc::server::SubscriberList = Arc::new(tokio::sync::Mutex::new(vec![entry]));
    let broker = Arc::new(Arc::clone(&subs));
    let spawner = Arc::new(super::MockSessionHostSpawner {
        spawn_result: None,
        fake_pid: 0,
    });
    let engine: Arc<dyn monocle_core::engine::EngineModule> =
        Arc::new(SucceedingMockEngineRediscovery);
    let mut manager = super::SessionManager::new(
        tmp.path().to_path_buf(),
        spawner,
        broker,
        engine,
        super::HookEndpointConfig::default(),
    );
    manager.with_peer_cred_verifier(Arc::new(FakePeerCredVerifierWithPeerPid {
        injected_peer_pid,
    }));
    manager.with_pid_sigterm_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let pids = Arc::clone(&sigterm_pids_clone);
        let done_tx = sigterm_done_tx.clone();
        let pid_raw = pid.as_raw() as u32;
        // Use try_lock to avoid blocking inside the sync closure.
        if let Ok(mut guard) = pids.try_lock() {
            guard.push(pid_raw);
        }
        let _ = done_tx.try_send(());
        Ok(())
    }));

    // RED GATE: current impl on the Running probe path, when verify_with_sidecar_pid
    // returns Err, SIGTERMs only data.pid (sidecar pid).  The peer pid is not carried
    // in the error — there is no second SIGTERM.  The "peer pid SIGTERMed" assertion FAILS.
    manager
        .rediscover_sessions()
        .await
        .expect("pass3-HIGH-002: rediscover_sessions must return Ok");

    // Wait for at least one SIGTERM to arrive (the sidecar SIGTERM should always fire).
    let _ = tokio::time::timeout(Duration::from_millis(500), sigterm_done_rx.recv()).await;
    // Brief settle to catch the second SIGTERM if it fires separately.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let captured: Vec<u32> = sigterm_pids.lock().await.clone();

    // Assert A: sidecar pid received SIGTERM.
    assert!(
        captured.contains(&alive_sidecar_pid),
        "pass3-HIGH-002: sidecar pid={} MUST receive SIGTERM on PID mismatch; \
         captured SIGTERMs: {:?}",
        alive_sidecar_pid,
        captured
    );

    // Assert B: socket peer pid ALSO received SIGTERM (belt-and-suspenders).
    // FAILS NOW: current impl does not carry peer_pid from the error; only
    // sidecar pid is SIGTERMed.
    assert!(
        captured.contains(&injected_peer_pid),
        "pass3-HIGH-002: socket peer_pid={} MUST ALSO receive SIGTERM on PID mismatch \
         (SS-session-manager §Per-session UDS security item 2 belt-and-suspenders). \
         Current impl: PeerCredMismatch carries no peer_pid → only sidecar pid \
         is SIGTERMed.  captured SIGTERMs: {:?}",
        injected_peer_pid,
        captured
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ---------------------------------------------------------------------------
// MED-002 (pass-3): dead-PID GC during rediscovery MUST emit broker events
// ---------------------------------------------------------------------------

/// MED-002 (pass-3): A sidecar whose recorded PID is dead (kill(pid, None)
/// returns ESRCH) must be GC'd AND the daemon must emit
/// `SessionStateChanged{Terminated}` then `SessionListUpdate` via the broker.
///
/// Per SS-daemon-wiring §3b emission table: "Re-discovery GC (dead session)
/// → any → Terminated → emit SessionStateChanged{Terminated} then
/// SessionListUpdate."
///
/// Current impl in the dead-PID inline path (Running/Launching at line ~4521
/// and Detached at line ~4553): GC + found_dead only; NO broker emission.
/// The broker-emission assertions FAIL.
///
/// The Terminating elapsed path already has broker emission (added in pass-2
/// HIGH-003); this test targets the Running/Launching and Detached dead-PID
/// paths which still lack it.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_dead_pid_emits_terminated() {
    let tmp = tempfile::tempdir().expect("pass3-MED-002: tempdir");
    let session_id = "00000000-0036-4000-a006-000000000004";

    // Spawn a real short-lived process and reap it so the PID is definitively dead.
    let mut child = std::process::Command::new("true")
        .spawn()
        .expect("pass3-MED-002: spawn 'true'");
    let dead_pid = child.id();
    let _ = child.wait(); // reap to avoid zombie; exits immediately

    let socket_path = short_socket_path("p3med002-dead");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Running",
        dead_pid,
        &socket_path,
        None,
    );

    // Wire a broker subscriber BEFORE calling rediscover_sessions().
    let (mut manager, _subs, mut rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: current dead-PID path does GC + found_dead, no broker emission.
    // The broker-assertion below FAILS.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass3-MED-002: rediscover_sessions must return Ok");

    // Assert A: GC basics (should pass even before the fix).
    assert_eq!(
        report.found_dead, 1,
        "pass3-MED-002: dead PID → found_dead=1; got {}",
        report.found_dead
    );
    assert_eq!(
        report.found_alive, 0,
        "pass3-MED-002: found_alive=0; got {}",
        report.found_alive
    );
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "pass3-MED-002: dead session MUST NOT appear in registry"
    );

    // Assert B + C: broker MUST emit SessionStateChanged{Terminated} then
    // SessionListUpdate for dead-PID GC paths.
    // RED GATE: current impl does not call broadcast_to_subscribers in the
    // dead-PID path → both assertions FAIL.
    let drain_deadline = tokio::time::Instant::now() + Duration::from_millis(300);
    let mut state_changed_terminated = false;
    let mut list_update_found = false;
    let mut ordering_violation = false;
    loop {
        match tokio::time::timeout_at(drain_deadline, rx.recv()).await {
            Ok(Some(monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state,
                ..
            })) if sid == session_id => {
                if new_state == monocle_ipc::types::SessionState::Terminated {
                    state_changed_terminated = true;
                    if list_update_found {
                        ordering_violation = true;
                    }
                }
            }
            Ok(Some(monocle_ipc::types::ServerToClient::SessionListUpdate { .. })) => {
                list_update_found = true;
            }
            Ok(Some(_)) => continue,
            _ => break,
        }
    }
    assert!(
        state_changed_terminated,
        "pass3-MED-002: dead-PID GC MUST emit SessionStateChanged{{Terminated}} to \
         broker (SS-daemon-wiring §3b).  Current impl skips broker emission on \
         Running/Launching and Detached dead-PID paths."
    );
    assert!(
        list_update_found,
        "pass3-MED-002: dead-PID GC MUST emit SessionListUpdate to broker \
         after SessionStateChanged{{Terminated}} (SS-daemon-wiring §3b)"
    );
    assert!(
        !ordering_violation,
        "pass3-MED-002: SessionStateChanged{{Terminated}} MUST precede \
         SessionListUpdate (SS-daemon-wiring §3b ordering constraint)"
    );
}

// ---------------------------------------------------------------------------
// MED-001 coverage: verify that FakePeerCredVerifier STILL passes the happy
//   path on Detached and Terminating with peer pid == sidecar pid
// ---------------------------------------------------------------------------

/// MED-001 (pass-3, Detached happy path): When `FakePeerCredVerifierWithPeerPid`
/// injects a peer_pid that MATCHES the sidecar pid, the Detached session MUST
/// be registered (verify_with_sidecar_pid returns Ok).
///
/// This test ensures no false-green: the pid cross-check is genuinely exercised
/// (not silently bypassed) on the Detached path.  If `verify_with_sidecar_pid`
/// defaults to `verify()` (ignoring sidecar_pid), this test passes vacuously
/// — the implementer must call the overriding method, not the default.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_detached_pid_match_registers() {
    let tmp = tempfile::tempdir().expect("pass3-MED-001-detached-happy: tempdir");
    let session_id = "00000000-0036-4000-a006-000000000005";

    let alive_sidecar_pid = std::process::id();
    // injected_peer_pid == sidecar_pid → cross-check PASSES.
    let injected_peer_pid = alive_sidecar_pid;

    let socket_path = short_socket_path("p3med001-det-ok");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Detached",
        alive_sidecar_pid,
        &socket_path,
        None,
    );

    // Mock session-host: accept and hold open.
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener =
            UnixListener::bind(&sock_clone).expect("pass3-MED-001-detached-happy: mock bind");
        if let Ok((_stream, _)) = listener.accept().await {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (tx, _rx_ch) = mpsc::channel::<monocle_ipc::types::ServerToClient>(
        monocle_ipc::server::CLIENT_CHANNEL_CAPACITY,
    );
    let entry = monocle_ipc::server::ClientEntry::new(tx);
    let subs: monocle_ipc::server::SubscriberList = Arc::new(tokio::sync::Mutex::new(vec![entry]));
    let broker = Arc::new(Arc::clone(&subs));
    let spawner = Arc::new(super::MockSessionHostSpawner {
        spawn_result: None,
        fake_pid: 0,
    });
    let engine: Arc<dyn monocle_core::engine::EngineModule> =
        Arc::new(SucceedingMockEngineRediscovery);
    let mut manager = super::SessionManager::new(
        tmp.path().to_path_buf(),
        spawner,
        broker,
        engine,
        super::HookEndpointConfig::default(),
    );
    manager.with_peer_cred_verifier(Arc::new(FakePeerCredVerifierWithPeerPid {
        injected_peer_pid,
    }));

    // Once the Detached path calls verify_with_sidecar_pid(stream, alive_sidecar_pid),
    // the verifier compares injected_peer_pid (== alive_sidecar_pid) → Ok → register.
    // Until the fix (Detached calls verify instead of verify_with_sidecar_pid), the
    // session is registered via verify() → Ok.
    // After the fix, verify_with_sidecar_pid is called → Ok (pids match) → still registers.
    // Either way this test PASSES — it exists to confirm no regression on the happy path.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass3-MED-001-detached-happy: rediscover_sessions must return Ok");

    assert_eq!(
        report.found_alive, 1,
        "pass3-MED-001-detached-happy: Detached + peer_pid==sidecar_pid → \
         found_alive=1; got {}",
        report.found_alive
    );
    assert!(
        manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "pass3-MED-001-detached-happy: Detached session with matching pids MUST \
         be registered"
    );

    let _ = std::fs::remove_file(&socket_path);
}

/// MED-001 (pass-3, Terminating happy path): When `FakePeerCredVerifierWithPeerPid`
/// injects a peer_pid that MATCHES the sidecar pid, the Terminating not-elapsed
/// session MUST be registered with Kill sent (verify_with_sidecar_pid returns Ok).
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_pid_match_kill_sent() {
    let tmp = tempfile::tempdir().expect("pass3-MED-001-terminating-happy: tempdir");
    let session_id = "00000000-0036-4000-a006-000000000006";

    let future_ms = unix_now_ms() + 10_000;
    let alive_sidecar_pid = std::process::id();
    // injected_peer_pid == sidecar_pid → cross-check PASSES → Kill sent.
    let injected_peer_pid = alive_sidecar_pid;

    let socket_path = short_socket_path("p3med001-term-ok");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        alive_sidecar_pid,
        &socket_path,
        Some(future_ms),
    );

    let (kill_received_tx, mut kill_received_rx) = mpsc::channel::<()>(4);
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener =
            UnixListener::bind(&sock_clone).expect("pass3-MED-001-terminating-happy: mock bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                let len = u32::from_le_bytes(len_buf) as usize;
                if len > 0 && len <= 65536 {
                    let mut body = vec![0u8; len];
                    if stream.read_exact(&mut body).await.is_ok() {
                        if let Ok(msg) =
                            serde_json::from_slice::<monocle_ipc::types::DaemonToHost>(&body)
                        {
                            if matches!(msg, monocle_ipc::types::DaemonToHost::Kill) {
                                let _ = kill_received_tx.send(()).await;
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(20)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    let (tx, _rx_ch) = mpsc::channel::<monocle_ipc::types::ServerToClient>(
        monocle_ipc::server::CLIENT_CHANNEL_CAPACITY,
    );
    let entry = monocle_ipc::server::ClientEntry::new(tx);
    let subs: monocle_ipc::server::SubscriberList = Arc::new(tokio::sync::Mutex::new(vec![entry]));
    let broker = Arc::new(Arc::clone(&subs));
    let spawner = Arc::new(super::MockSessionHostSpawner {
        spawn_result: None,
        fake_pid: 0,
    });
    let engine: Arc<dyn monocle_core::engine::EngineModule> =
        Arc::new(SucceedingMockEngineRediscovery);
    let mut manager = super::SessionManager::new(
        tmp.path().to_path_buf(),
        spawner,
        broker,
        engine,
        super::HookEndpointConfig::default(),
    );
    manager.with_peer_cred_verifier(Arc::new(FakePeerCredVerifierWithPeerPid {
        injected_peer_pid,
    }));

    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass3-MED-001-terminating-happy: rediscover_sessions must return Ok");

    // Kill must be sent (pid match → proceed to Kill path).
    let got_kill = tokio::time::timeout(Duration::from_millis(500), kill_received_rx.recv()).await;
    assert!(
        got_kill.is_ok() && got_kill.unwrap().is_some(),
        "pass3-MED-001-terminating-happy: Terminating + peer_pid==sidecar_pid \
         MUST send Kill (pid check passes → normal Terminating path)"
    );

    assert_eq!(
        report.found_alive, 1,
        "pass3-MED-001-terminating-happy: Terminating + future deadline + pid match \
         → found_alive=1; got {}",
        report.found_alive
    );
    assert!(
        manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "pass3-MED-001-terminating-happy: Terminating session with matching pids \
         MUST be registered"
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ---------------------------------------------------------------------------
// HIGH-002 (pass-2) — Dead PID must also delete the orphaned socket file
// ---------------------------------------------------------------------------

/// HIGH-002 (pass-2): Dead-PID sidecar WITH an orphaned socket file present in
/// the runtime_dir → re-discovery must delete BOTH the sidecar JSON AND the
/// orphaned socket file.
///
/// Per BC-2.08.004 PC-2c and Invariant 5: "On dead-PID GC, delete both sidecar
/// and orphaned socket file."
///
/// Current impl deletes only the sidecar JSON in the dead-PID path; it does NOT
/// delete the orphaned socket file.  The socket-file assertion below will FAIL.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_dead_pid_deletes_orphan_socket() {
    let tmp = tempfile::tempdir().expect("pass2-HIGH-002: tempdir");
    let session_id = "00000000-0036-4000-a005-000000000004";

    // Spawn a real short-lived process and reap it so the PID is definitively dead.
    let mut child = std::process::Command::new("true")
        .spawn()
        .expect("pass2-HIGH-002: spawn 'true'");
    let dead_pid = child.id();
    let _ = child.wait(); // reap to avoid zombie; process exits immediately

    // Create the sidecar JSON with a socket path that we will also create as a
    // regular file to simulate an orphaned socket inode.
    let socket_path = short_socket_path("hi002p2-orphsock");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Running",
        dead_pid,
        &socket_path,
        None,
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Create an orphaned socket file at the path recorded in the sidecar.
    // std::fs::remove_file handles both regular files and socket files on
    // Linux/macOS via unlink(2), so a regular file suffices here.
    // Use File::create (not std::fs::write) to avoid the disallowed-methods lint.
    {
        let _ =
            std::fs::File::create(&socket_path).expect("pass2-HIGH-002: create orphan socket file");
    }
    assert!(
        socket_path.exists(),
        "pass2-HIGH-002: orphaned socket file must exist before rediscovery"
    );

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // RED GATE: current dead-PID path removes sidecar but not the socket file.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass2-HIGH-002: rediscovery must return Ok");

    // Assert 1: sidecar JSON deleted.
    assert!(
        !sidecar_path.exists(),
        "pass2-HIGH-002: sidecar JSON must be deleted for dead PID; \
         still at {:?}",
        sidecar_path
    );

    // Assert 2: orphaned socket file also deleted.
    // FAILS NOW: current impl only removes the sidecar, not the socket file.
    assert!(
        !socket_path.exists(),
        "pass2-HIGH-002: orphaned socket file must be deleted for dead PID \
         (BC-2.08.004 PC-2c + Invariant 5); still at {:?}. \
         Current impl removes only the sidecar JSON.",
        socket_path
    );

    // Assert 3: session NOT in registry.
    assert!(
        !manager
            .session_list()
            .await
            .iter()
            .any(|s| s.session_id == session_id),
        "pass2-HIGH-002: dead session MUST NOT appear in registry"
    );

    // Assert 4: found_dead=1.
    assert_eq!(
        report.found_dead, 1,
        "pass2-HIGH-002: dead PID → found_dead=1; got {}",
        report.found_dead
    );
}

// ---------------------------------------------------------------------------
// HIGH-003 (pass-2) — Elapsed Terminating path must emit broker events
// ---------------------------------------------------------------------------

/// HIGH-003 (pass-2): Terminating sidecar with ELAPSED absolute `kill_deadline_unix_ms`
/// (deadline is in the past) + alive PID → immediate SIGKILL + GC + MUST emit
/// `SessionStateChanged{Terminated}` then `SessionListUpdate` via broker.
/// Ordering: `SessionStateChanged` MUST precede `SessionListUpdate`.
///
/// Per BC-2.08.004 EC-173 and SS-daemon-wiring §3b emission table.
///
/// Current impl in the elapsed inline path: SIGKILL + sidecar delete +
/// `report.found_dead += 1`, but does NOT call `broadcast_to_subscribers` for
/// either message.  The broker-emission assertions below will FAIL.
#[tokio::test]
async fn test_BC_2_08_004_rediscovery_terminating_elapsed_emits_broker() {
    let tmp = tempfile::tempdir().expect("pass2-HIGH-003: tempdir");
    let session_id = "00000000-0036-4000-a005-000000000005";

    // Elapsed deadline: well in the past (1 minute ago).
    let elapsed_ms = unix_now_ms().saturating_sub(60_000);
    let socket_path = short_socket_path("hi003p2-elapsed");
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(), // alive PID so SIGKILL path fires
        &socket_path,
        Some(elapsed_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    let (sigkill_tx, mut sigkill_rx) = mpsc::channel::<u32>(4);
    // Subscribe to broker BEFORE calling rediscover_sessions().
    // make_rediscovery_manager wires one subscriber into the broker.
    let (mut manager, _subs, mut rx) = make_rediscovery_manager(tmp.path(), true);
    manager.with_pid_sigkill_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigkill_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // RED GATE: current elapsed inline path does not emit broker messages.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass2-HIGH-003: rediscover_sessions must return Ok");

    // Assert A: SIGKILL fired.
    let got_sigkill = tokio::time::timeout(Duration::from_millis(300), sigkill_rx.recv()).await;
    assert!(
        got_sigkill.is_ok() && got_sigkill.unwrap().is_some(),
        "pass2-HIGH-003: SIGKILL must fire for elapsed Terminating deadline"
    );

    // Assert B: sidecar deleted.
    assert!(
        !sidecar_path.exists(),
        "pass2-HIGH-003: sidecar must be deleted on elapsed SIGKILL path; \
         still at {:?}",
        sidecar_path
    );

    // Assert C: found_dead=1.
    assert_eq!(
        report.found_dead, 1,
        "pass2-HIGH-003: elapsed Terminating → found_dead=1; got {}",
        report.found_dead
    );

    // Assert D + E: broker must emit SessionStateChanged{Terminated} then
    // SessionListUpdate in that order.
    // RED GATE: current impl does not call broadcast_to_subscribers here → FAIL.
    let drain_deadline = tokio::time::Instant::now() + Duration::from_millis(300);
    let mut state_changed_terminated = false;
    let mut list_update_found = false;
    let mut ordering_violation = false;
    loop {
        match tokio::time::timeout_at(drain_deadline, rx.recv()).await {
            Ok(Some(monocle_ipc::types::ServerToClient::SessionStateChanged {
                session_id: ref sid,
                new_state,
                ..
            })) if sid == session_id => {
                if new_state == monocle_ipc::types::SessionState::Terminated {
                    state_changed_terminated = true;
                    if list_update_found {
                        // SessionListUpdate arrived before SessionStateChanged — ordering violation.
                        ordering_violation = true;
                    }
                }
            }
            Ok(Some(monocle_ipc::types::ServerToClient::SessionListUpdate { .. })) => {
                list_update_found = true;
            }
            Ok(Some(_)) => continue,
            _ => break,
        }
    }
    assert!(
        state_changed_terminated,
        "pass2-HIGH-003: elapsed-deadline SIGKILL path MUST emit \
         SessionStateChanged{{Terminated}} to broker \
         (BC-2.08.004 EC-173 + SS-daemon-wiring §3b). \
         Current impl skips broker emission in the elapsed inline path."
    );
    assert!(
        list_update_found,
        "pass2-HIGH-003: elapsed-deadline SIGKILL path MUST emit SessionListUpdate \
         to broker (SS-daemon-wiring §3b)"
    );
    assert!(
        !ordering_violation,
        "pass2-HIGH-003: SessionStateChanged{{Terminated}} MUST precede SessionListUpdate \
         (BC-2.08.008 Invariant 4)"
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// ADVERSARIAL PASS 4 CORRECTIONS — RED-GATE TESTS
//
// These tests encode spec behaviour surfaced by adversarial-pass-4 and must
// FAIL against the current implementation.  Each comment identifies the
// finding ID.
// ===========================================================================

// ---------------------------------------------------------------------------
// HIGH-001 (pass-4) — Registry leak: watchdog Terminated transition MUST call
//                      spawn_gc_task so stale Terminated entry is evicted
// ---------------------------------------------------------------------------

/// HIGH-001 (pass-4): Terminating sidecar, NOT-elapsed deadline, alive PID,
/// SO_PEERCRED OK → `rediscover_sessions()` registers `SessionEntry{Terminating}`
/// and spawns the background watchdog.  The mock session-host then sends
/// `HostToDaemon::StateChanged{Terminated}` → watchdog drives the entry to
/// `Terminated`.  After the 10-second GC grace period (BC-2.08.005 PC-1) the
/// entry MUST be removed from the registry.
///
/// The current watchdog (`emit_terminated` closure, mod.rs ~5016-5071) sets
/// `entry.state = Terminated` and broadcasts `SessionStateChanged` + `SessionListUpdate`
/// but NEVER calls `SessionManager::spawn_gc_task`.  The `spawn_gc_task` doc
/// comment explicitly lists "re-discovery alive-then-dead paths" as a required
/// call site.  Because the GC task is never spawned the stale `Terminated` entry
/// lingers in `self.sessions` forever and is visible in every subsequent
/// `session_list()` call.
///
/// This test covers both the `StateChanged::Terminated` watchdog arm AND the
/// deadline-elapsed SIGKILL arm (sharing the same `emit_terminated` closure).
/// Two sub-cases:
///   Sub-case A (StateChanged path): mock session-host sends StateChanged::Terminated.
///   Sub-case B (deadline path): virtual-time advance fires the deadline arm.
///
/// For clarity this test exercises sub-case A (the StateChanged message path)
/// using real async timing with a paused clock.  Sub-case B is structurally
/// identical and the same `spawn_gc_task` call site fixes both.
///
/// RED GATE:
/// - After `emit_terminated` + advance(10s) + yield the entry STILL appears in
///   `session_list()` because `spawn_gc_task` was never called.
/// - The "MUST NOT appear in registry" assertion FAILS.
/// - The "sidecar deleted" assertion may also fail if `spawn_gc_task`'s
///   sidecar-delete is the only removal for this path.
///
/// BC reference: BC-2.08.005 (GC after Terminated) + BC-2.08.004 PC-2b
/// (Terminating watchdog transition).  `spawn_gc_task` doc:
/// "Called at every Terminated transition point: ... Re-discovery alive-then-dead paths."
///
/// Implementer note: call `SessionManager::spawn_gc_task(session_id, sidecar_path,
/// socket_path, sessions_arc.clone(), broker_arc.clone())` from both the
/// `terminated_by_msg` arm and the `_ = tokio::time::sleep_until(kill_deadline_tokio)`
/// arm of the watchdog `tokio::select!`, immediately after `emit_terminated(...)`.
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_004_rediscovery_watchdog_terminated_entry_gcd() {
    let tmp = tempfile::tempdir().expect("pass4-HIGH-001: tempdir");
    let session_id = "00000000-0036-4000-a007-000000000001";

    // Deadline well in the future (30s): ensures the watchdog stays alive until
    // the mock session-host sends StateChanged::Terminated (sub-case A).
    let future_ms = unix_now_ms() + 30_000;
    let socket_path = short_socket_path("p4hi001-wdog");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(), // alive PID, SO_PEERCRED allow=true
        &socket_path,
        Some(future_ms),
    );
    let sidecar_path = tmp.path().join(format!("session-{}.json", session_id));

    // Mock session-host: accept the Kill fire-and-forget connect, read Kill frame,
    // then after a short delay send StateChanged{Terminated} so the watchdog's
    // `terminated_by_msg` arm fires.
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener =
            UnixListener::bind(&sock_clone).expect("pass4-HIGH-001: mock session-host bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            // Consume the Kill message the daemon sends fire-and-forget.
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                let len = u32::from_le_bytes(len_buf) as usize;
                if len <= 65536 {
                    let mut body = vec![0u8; len];
                    let _ = stream.read_exact(&mut body).await;
                }
            }
            // Short pause so the watchdog's select! is parked, then send Terminated.
            tokio::time::sleep(Duration::from_millis(100)).await;
            let terminated_msg = HostToDaemon::StateChanged {
                new_state: monocle_ipc::types::SessionState::Terminated,
                degraded_env: None,
            };
            let bytes =
                serde_json::to_vec(&terminated_msg).expect("pass4-HIGH-001: serialize terminated");
            let len_bytes = (bytes.len() as u32).to_le_bytes();
            let _ = stream.write_all(&len_bytes).await;
            let _ = stream.write_all(&bytes).await;
            let _ = stream.flush().await;
            // Hold stream open so the watchdog has time to process the message.
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
    // Yield so the mock task has time to bind before rediscover_sessions is called.
    tokio::time::sleep(Duration::from_millis(10)).await;

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // Phase 1: rediscover_sessions registers Terminating entry + spawns watchdog.
    let report = manager
        .rediscover_sessions()
        .await
        .expect("pass4-HIGH-001: rediscover_sessions must return Ok");

    assert_eq!(
        report.found_alive, 1,
        "pass4-HIGH-001: Terminating + future deadline → found_alive=1; got {}",
        report.found_alive
    );

    // Verify the entry is present and Terminating immediately after rediscovery.
    let sessions_before = manager.session_list().await;
    assert!(
        sessions_before.iter().any(|s| s.session_id == session_id),
        "pass4-HIGH-001: Terminating entry MUST be present right after rediscovery"
    );

    // Phase 2: advance virtual time so the mock's 100ms sleep elapses and the
    // watchdog receives StateChanged::Terminated.
    tokio::time::advance(Duration::from_millis(200)).await;
    // Yield multiple times to let the watchdog task run to completion.
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }

    // Phase 3: advance virtual time past the 10-second GC grace period.
    // If spawn_gc_task was called, the GC task will fire here and remove the entry.
    tokio::time::advance(Duration::from_secs(11)).await;
    // Yield again to let the GC task run.
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }

    // Assert 1: sidecar MUST be deleted.
    // The watchdog's emit_terminated closure already deletes the sidecar inline
    // (std::fs::remove_file in the closure), so this assertion exercises whether
    // the inline deletion ran.
    assert!(
        !sidecar_path.exists(),
        "pass4-HIGH-001: sidecar must be deleted after watchdog Terminated transition; \
         still at {:?}",
        sidecar_path
    );

    // Assert 2 (RED GATE — will FAIL against current impl):
    // After the 10-second GC window the entry MUST NOT appear in the registry.
    // Current impl: `emit_terminated` never calls `spawn_gc_task`, so the
    // entry remains in `self.sessions` as a stale Terminated/Stopped entry.
    let sessions_after = manager.session_list().await;
    assert!(
        !sessions_after.iter().any(|s| s.session_id == session_id),
        "pass4-HIGH-001: Terminated entry MUST be removed from registry after 10s GC \
         grace period (BC-2.08.005 PC-1 + spawn_gc_task contract). \
         Current impl: watchdog `emit_terminated` closure does NOT call \
         `spawn_gc_task`, so the stale Terminated entry lingers in self.sessions \
         indefinitely and is visible in every session_list() call. \
         Implementer fix: call `SessionManager::spawn_gc_task(sid, sidecar_path, \
         socket_path, sessions_arc.clone(), broker_arc.clone())` from both arms \
         of the watchdog tokio::select! immediately after `emit_terminated(...)`. \
         This mirrors the live kill_session GC wiring."
    );

    let _ = std::fs::remove_file(&socket_path);
}

// ===========================================================================
// MED-001 — Terminating watchdog GC grace MUST be measured from the Terminated
//           TRANSITION, not from rediscover_sessions() call time (T0).
//
// BC-2.08.005 PC-1: "10-second GC grace period begins at the Terminated transition."
//
// Bug: `watchdog_gc_deadline = T0 + 10s` is precomputed in the SYNCHRONOUS body
// of rediscover_sessions() (mod.rs ~5248).  Both select! arms then call
// `sleep_until(watchdog_gc_deadline)`.  When the Terminated transition fires
// AFTER T0+10s (e.g. at T0+12s via null-deadline fresh-12s-window), the
// precomputed gc_deadline is already in the past → the GC task fires immediately
// → 0 seconds of grace instead of 10.
//
// Test strategy (deadline / SIGKILL arm):
//   - null kill_deadline → fresh 12s window → Terminated at T0+12s
//   - advance to T0+12s to fire the transition
//   - at T0+12s+5s (well past T0+10s but only 5s into grace) entry MUST still exist
//   - at T0+12s+10s (full grace) entry MUST be removed
//
// Test strategy (terminated_by_msg arm):
//   - deadline far in future (T0+30s), mock sends StateChanged::Terminated at T0+7s
//   - advance to T0+7s to fire the msg-arm Terminated transition
//   - at T0+10.2s (past buggy T0+10s gc_deadline, but 6.8s before correct T0+17s)
//     entry MUST still exist
//   - at T0+17.3s entry MUST be removed
// ===========================================================================

/// MED-001 (deadline / SIGKILL arm): Terminating sidecar with NULL kill_deadline
/// → fresh 12s window → SIGKILL fires at T0+12s → Terminated transition.
/// GC grace MUST begin from T0+12s (the transition), NOT from T0.
///
/// Assertion (a): at T0+12s+5s the entry is STILL present in the registry
///   (grace has not yet expired — would fail with buggy impl because T0+10s < T0+12s).
/// Assertion (b): at T0+12s+10s the entry is REMOVED from the registry
///   (full 10s grace from the transition has elapsed).
///
/// RED GATE: buggy impl precomputes gc_deadline=T0+10s; GC fires immediately when
/// the transition happens at T0+12s (deadline already past) → entry removed before
/// assertion (a) is checked → assertion (a) fails with "entry must still be present".
///
/// BC-2.08.005 PC-1.
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_004_rediscovery_watchdog_gc_grace_from_transition_deadline_arm() {
    let tmp = tempfile::tempdir().expect("MED-001-deadline: tempdir");
    let session_id = "00000000-0036-4000-a008-000000000001";

    // NULL kill_deadline → fresh 12s window from T0 → Terminated transition at T0+12s.
    let socket_path = short_socket_path("med001-dl");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(), // alive PID; SO_PEERCRED allow=true
        &socket_path,
        None, // null deadline → fresh 12s window
    );

    // Mock session-host: accept the Kill connect, then hold open (never sends Terminated).
    // The deadline arm fires after the 12s window.
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener =
            UnixListener::bind(&sock_clone).expect("MED-001-deadline: mock session-host bind");
        if let Ok((_stream, _)) = listener.accept().await {
            // Hold open for longer than the test lasts; deadline arm will fire.
            tokio::time::sleep(Duration::from_secs(120)).await;
        }
    });
    // Yield so the mock task binds the socket before rediscover_sessions connects.
    tokio::time::sleep(Duration::from_millis(10)).await;

    let (sigkill_tx, mut sigkill_rx) = mpsc::channel::<u32>(4);
    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);
    manager.with_pid_sigkill_fn(Arc::new(move |pid: nix::unistd::Pid| {
        let _ = sigkill_tx.try_send(pid.as_raw() as u32);
        Ok(())
    }));

    // T0: rediscover_sessions() — registers Terminating entry, spawns watchdog,
    // precomputes watchdog_gc_deadline = T0+10s (the BUG).
    let report = manager
        .rediscover_sessions()
        .await
        .expect("MED-001-deadline: rediscover_sessions must return Ok");

    assert_eq!(
        report.found_alive, 1,
        "MED-001-deadline: Terminating + null deadline → found_alive=1; got {}",
        report.found_alive
    );

    // Verify entry is present immediately after rediscovery.
    let sessions_initial = manager.session_list().await;
    assert!(
        sessions_initial.iter().any(|s| s.session_id == session_id),
        "MED-001-deadline: Terminating entry MUST be present immediately after rediscovery"
    );

    // Advance to T0+12s: the 12s fresh window elapses → SIGKILL fires → Terminated transition.
    tokio::time::advance(Duration::from_secs(12)).await;
    // Drain the executor so the watchdog's deadline arm runs through emit_terminated
    // and spawns the GC sub-task.
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }

    // Confirm SIGKILL was fired (the Terminated transition happened).
    let got_sigkill = tokio::time::timeout(Duration::from_millis(200), sigkill_rx.recv()).await;
    assert!(
        got_sigkill.is_ok() && got_sigkill.unwrap().is_some(),
        "MED-001-deadline: watchdog MUST fire SIGKILL at the 12s deadline"
    );

    // Yield again to let the GC sub-task be spawned and registered in the timer wheel.
    for _ in 0..6 {
        tokio::task::yield_now().await;
    }

    // -----------------------------------------------------------------------
    // Assertion (a): at T0+12s+5s the entry MUST still be present.
    //
    // Correct impl: GC deadline = (T0+12s)+10s = T0+22s → not yet elapsed.
    // Buggy impl:   GC deadline = T0+10s → already 2s in the past when the
    //               transition fires at T0+12s → GC fires immediately → entry
    //               already removed → assertion FAILS.
    // -----------------------------------------------------------------------
    tokio::time::advance(Duration::from_secs(5)).await;
    for _ in 0..6 {
        tokio::task::yield_now().await;
    }

    let sessions_mid = manager.session_list().await;
    assert!(
        sessions_mid.iter().any(|s| s.session_id == session_id),
        "MED-001-deadline: [assertion a] entry MUST still be present 5s after \
         Terminated transition (grace not yet expired). \
         BUG: watchdog_gc_deadline precomputed at T0+10s; transition fires at T0+12s; \
         T0+10s is already past → GC fires immediately on spawn → 0s grace. \
         FIX: compute gc_deadline AFTER emit_terminated, not in rediscover_sessions \
         sync body. Use spawn_gc_task() at transition time (mirrors kill_session \
         ~mod.rs 2770)."
    );

    // -----------------------------------------------------------------------
    // Assertion (b): at T0+12s+10s the entry MUST be removed.
    //
    // Correct impl: GC deadline = T0+22s; we are now at T0+12s+10s = T0+22s.
    // -----------------------------------------------------------------------
    // Advance the remaining 5s to reach T0+22s total.
    tokio::time::advance(Duration::from_secs(5)).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }

    let sessions_after = manager.session_list().await;
    assert!(
        !sessions_after.iter().any(|s| s.session_id == session_id),
        "MED-001-deadline: [assertion b] entry MUST be removed after full 10s GC grace \
         from the Terminated transition (BC-2.08.005 PC-1)"
    );

    let _ = std::fs::remove_file(&socket_path);
}

/// MED-001 (terminated_by_msg arm): Terminating sidecar, deadline far in future
/// (T0+30s), mock session-host sends StateChanged::Terminated at T0+1.5s.
/// GC grace MUST begin from T0+1.5s (msg-arm transition), NOT from T0.
///
/// The key scenario: with the bug gc_deadline=T0+10s (precomputed at T0),
/// the GC fires at T0+10s — which is only 8.5s after the T0+1.5s transition,
/// not 10s.  The correct gc_deadline is T0+1.5s+10s = T0+11.5s.
///
/// Assertion (a): at T0+10.2s the entry is STILL present.
///   Correct gc_deadline = T0+11.5s → grace not yet expired.
///   Buggy gc_deadline = T0+10s → GC fires at T0+10s (8.5s after transition)
///   → entry already removed → assertion FAILS.
///
/// Assertion (b): at T0+11.7s the entry MUST be removed.
///
/// BC-2.08.005 PC-1.
#[tokio::test(start_paused = true)]
async fn test_BC_2_08_004_rediscovery_watchdog_gc_grace_from_transition_msg_arm() {
    let tmp = tempfile::tempdir().expect("MED-001-msg: tempdir");
    let session_id = "00000000-0036-4000-a008-000000000002";

    // Deadline far in the future (T0+30s virtual) so the msg arm fires first.
    let future_ms = unix_now_ms() + 30_000;
    let socket_path = short_socket_path("med001-msg");
    let _ = std::fs::remove_file(&socket_path);
    write_sidecar_v3(
        tmp.path(),
        session_id,
        "Terminating",
        std::process::id(),
        &socket_path,
        Some(future_ms),
    );

    // Mock session-host: accept Kill connect, consume Kill frame, then after
    // a 1.5s delay (virtual time) send StateChanged::Terminated.
    // The GC grace must begin from T0+1.5s.  With the bug, gc_deadline=T0+10s
    // fires at T0+10s (8.5s of effective grace, not 10s).
    let sock_clone = socket_path.clone();
    tokio::spawn(async move {
        let _ = std::fs::remove_file(&sock_clone);
        let listener =
            UnixListener::bind(&sock_clone).expect("MED-001-msg: mock session-host bind");
        if let Ok((mut stream, _)) = listener.accept().await {
            // Consume the Kill frame the daemon sends fire-and-forget.
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                let len = u32::from_le_bytes(len_buf) as usize;
                if len <= 65536 {
                    let mut body = vec![0u8; len];
                    let _ = stream.read_exact(&mut body).await;
                }
            }
            // Session exits 1.5s after re-discovery.
            tokio::time::sleep(Duration::from_millis(1_500)).await;
            let terminated_msg = HostToDaemon::StateChanged {
                new_state: monocle_ipc::types::SessionState::Terminated,
                degraded_env: None,
            };
            let bytes =
                serde_json::to_vec(&terminated_msg).expect("MED-001-msg: serialize terminated");
            let len_bytes = (bytes.len() as u32).to_le_bytes();
            let _ = stream.write_all(&len_bytes).await;
            let _ = stream.write_all(&bytes).await;
            let _ = stream.flush().await;
            // Hold stream open so the watchdog reader stays alive.
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(10)).await;

    let (mut manager, _subs, _rx) = make_rediscovery_manager(tmp.path(), true);

    // T0: rediscover_sessions() — registers Terminating, spawns watchdog.
    // watchdog_gc_deadline = T0+10s (the BUG: should be computed at transition time).
    let report = manager
        .rediscover_sessions()
        .await
        .expect("MED-001-msg: rediscover_sessions must return Ok");

    assert_eq!(
        report.found_alive, 1,
        "MED-001-msg: Terminating + future deadline → found_alive=1; got {}",
        report.found_alive
    );

    // Advance to T0+1.6s: mock's 1.5s sleep fires, Terminated message written to socket.
    // Yield multiple times to drain the executor so the watchdog reads the msg,
    // calls emit_terminated, and spawns the GC sub-task.
    tokio::time::advance(Duration::from_millis(1_600)).await;
    for _ in 0..16 {
        tokio::task::yield_now().await;
    }

    // Confirm the entry transitioned to Terminated (msg arm fired).
    let sessions_post_transition = manager.session_list().await;
    assert!(
        sessions_post_transition
            .iter()
            .any(|s| s.session_id == session_id),
        "MED-001-msg: entry MUST still be in registry immediately after Terminated \
         transition (grace period just started at T0+1.5s)"
    );

    // -----------------------------------------------------------------------
    // Advance to T0+10.2s.  This is PAST the buggy precomputed gc_deadline
    // (T0+10s) but BEFORE the correct gc_deadline (T0+1.5s+10s = T0+11.5s).
    //
    // Correct impl: gc_deadline = T0+11.5s → GC has NOT fired → entry present.
    // Buggy impl:   gc_deadline = T0+10s → GC fires at T0+10s → entry removed.
    //
    // Assertion (a): entry MUST still be present at T0+10.2s.
    // RED GATE: buggy impl removes entry at T0+10s → assertion FAILS.
    // -----------------------------------------------------------------------
    tokio::time::advance(Duration::from_millis(8_600)).await; // now at ~T0+10.2s
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }

    let sessions_mid = manager.session_list().await;
    assert!(
        sessions_mid.iter().any(|s| s.session_id == session_id),
        "MED-001-msg: [assertion a] entry MUST still be present at T0+10.2s \
         (Terminated at T0+1.5s; correct gc_deadline=T0+11.5s; 1.3s remain). \
         BUG: watchdog_gc_deadline precomputed at T0+10s → GC fires at T0+10s \
         (only 8.5s after T0+1.5s transition, not the required 10s grace). \
         FIX: compute gc_deadline AFTER emit_terminated in the msg arm, \
         i.e. call spawn_gc_task() at transition time (mirrors kill_session ~mod.rs 2770)."
    );

    // -----------------------------------------------------------------------
    // Assertion (b): at T0+11.7s the entry MUST be removed.
    // Advance the remaining ~1.5s to T0+11.7s.
    // -----------------------------------------------------------------------
    tokio::time::advance(Duration::from_millis(1_500)).await; // now at ~T0+11.7s
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }

    let sessions_after = manager.session_list().await;
    assert!(
        !sessions_after.iter().any(|s| s.session_id == session_id),
        "MED-001-msg: [assertion b] entry MUST be removed after 10s GC grace from \
         the Terminated transition at T0+1.5s; correct gc_deadline=T0+11.5s \
         (BC-2.08.005 PC-1)"
    );

    let _ = std::fs::remove_file(&socket_path);
}
