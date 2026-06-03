//! Killer Scenario Integration Tests — BC-2.06.022
//!
//! End-to-end integration tests for the permission prompt lifecycle:
//! daemon → IPC → TUI overlay → user key → IPC write-back → daemon.
//!
//! # Behavioral Contract
//!
//! BC-2.06.022: Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve.
//! Every test drives the REAL production handler chain — NOT isolated helpers.
//!
//! # Production path exercised
//!
//! Every async test drives at minimum:
//!   - `setup_ipc_streams_with_rx` (wires app.ipc_tx + returns inbound_rx from real socket)
//!   - `spawn_ipc_reader` (real reader task that drains the UDS socket)
//!   - `handle_server_message` (real inbound dispatch router)
//!   - `on_permission_prompt_queued` (IPC receive → overlay_stack push + mode transition)
//!   - `dispatch_key_event` (key → PermissionDecision send via ipc_tx)
//!   - `on_permission_prompt_resolved` (reached THROUGH handle_server_message dispatch)
//!   - `on_transport_event(Disconnected)` (for AC-004 disconnect tests; reached from
//!     the Err arm of inbound_rx — mirroring the run() loop's Ok(Err(e)) dispatch path)
//!
//! # Test isolation (AC-007 / BC-2.06.022 INV-1)
//!
//! Each test uses its own `tempfile::TempDir` for a unique UDS socket path.
//! No shared mutable state across test functions.
//!
//! # Red Gate
//!
//! These tests exercise the ALREADY-IMPLEMENTED production handlers from S-026/S-027.
//! They are regression-coverage tests for BC-2.06.022. If any test passes without
//! exercising the real production path, that is a wiring gap (the canonical defect
//! class for EPIC-06).
//!
//! `#![allow(non_snake_case)]` is required because the factory-mandated test naming
//! convention uses uppercase BC identifiers: `test_BC_S_SS_NNN_...`.
#![allow(non_snake_case)]

use monocle_config::MonocleConfig;
use monocle_core::tui::binding::{KeyCode, KeyEvent, KeyModifiers};
use monocle_core::tui::state::{AppMode, FocusSnapshot};
use monocle_ipc::framing::{read_framed, write_framed};
use monocle_ipc::types::{
    ClientToServer, PermissionDecisionKind, PermissionPromptPayload, ServerToClient,
};
use monocle_tui::app::{
    build_builtin_binding_layers, dispatch_key_event, handle_server_message,
    on_permission_prompt_queued, on_transport_event, setup_ipc_streams_with_rx, App, KeyOutcome,
    TransportEvent,
};
use monocle_tui::ui::sessions_panel::SessionsPanelState;
use std::time::Duration;
use tokio::net::{UnixListener, UnixStream};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn no_mod() -> KeyModifiers {
    KeyModifiers::default()
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: no_mod(),
    }
}

/// Build a `PermissionPromptPayload` for a Bash tool invocation.
///
/// Used as the canonical test vector (BC-2.06.022 Canonical Test Vectors §KS-003).
fn bash_payload(prompt_id: Uuid) -> PermissionPromptPayload {
    PermissionPromptPayload {
        prompt_id,
        session_id: "sess-killer-001".into(),
        tool_name: "Bash".into(),
        tool_input: serde_json::json!({"command": "rm -rf /tmp/test"}),
        old_content: None,
        new_content: None,
    }
}

/// Build a `PermissionPromptPayload` for an Edit tool invocation.
///
/// Carries `old_content` / `new_content` so `payload_to_modal` produces
/// `ToolPayload::Edit { .. }` — required for AC-006 diff render assertion.
/// Canonical test vector per AC-006 (BC-2.06.022 PC-6).
fn edit_payload(prompt_id: Uuid) -> PermissionPromptPayload {
    PermissionPromptPayload {
        prompt_id,
        session_id: "sess-killer-002".into(),
        tool_name: "Edit".into(),
        tool_input: serde_json::json!({"path": "/tmp/test.txt"}),
        old_content: Some("hello\n".into()),
        new_content: Some("hello world\n".into()),
    }
}

/// Spawn a `MockDaemon` on the provided socket path.
///
/// The MockDaemon:
///   1. Listens on a `UnixListener` at `sock_path`.
///   2. Accepts one TUI client connection.
///   3. Sends `ServerToClient::InitialState { overlay_stack, ... }` immediately.
///   4. Sends any additional `ServerToClient` messages enqueued via `to_send_tx`.
///   5. Reads `ClientToServer` messages from the TUI and forwards them via `received_tx`.
///   6. Exits when the TUI closes the connection or `to_send_tx` is dropped.
///
/// Returns a tuple of:
///   - `to_send_tx`: sender for `ServerToClient` messages to push to the TUI.
///   - `received_rx`: receiver for `ClientToServer` messages from the TUI.
///   - `task_handle`: `JoinHandle` — caller must `.await` or `.abort()` to clean up.
///   - `ready_rx`: fires once the listener is bound (prevents connect-before-listen race).
///
/// # Socket isolation (AC-007)
///
/// Each call to `spawn_mock_daemon` must receive a unique `sock_path` from the
/// caller's own `tempfile::TempDir`. Never reuse paths across tests.
///
/// # Synchronization
///
/// The caller waits for `ready_rx` to fire before proceeding with TUI setup.
/// This prevents the TUI from attempting to connect before the listener is bound.
async fn spawn_mock_daemon(
    sock_path: std::path::PathBuf,
    initial_overlay: Vec<PermissionPromptPayload>,
) -> (
    tokio::sync::mpsc::Sender<ServerToClient>,
    tokio::sync::mpsc::Receiver<ClientToServer>,
    tokio::task::JoinHandle<()>,
    tokio::sync::oneshot::Receiver<()>, // ready signal: listener bound
) {
    let (to_send_tx, mut to_send_rx) = tokio::sync::mpsc::channel::<ServerToClient>(32);
    let (received_tx, received_rx) = tokio::sync::mpsc::channel::<ClientToServer>(32);
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();

    let handle = tokio::spawn(async move {
        // Bind the UDS listener BEFORE signalling ready.
        let listener = match UnixListener::bind(&sock_path) {
            Ok(l) => l,
            Err(e) => panic!("MockDaemon: failed to bind UDS listener: {e}"),
        };

        // Signal that the listener is bound and accepting connections.
        let _ = ready_tx.send(());

        // Accept the single TUI client connection.
        let (mut stream, _addr) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => panic!("MockDaemon: accept failed: {e}"),
        };

        // Send InitialState immediately after accepting.
        let initial_state = ServerToClient::InitialState {
            sessions: vec![],
            ring_tail: vec![],
            overlay_stack: initial_overlay,
            drop_counter: 0,
        };
        if let Err(e) = write_framed(&mut stream, &initial_state).await {
            panic!("MockDaemon: failed to send InitialState: {e}");
        }

        // Concurrently:
        //   a) Forward any `to_send_rx` messages to the TUI client.
        //   b) Read `ClientToServer` messages from the TUI and forward to `received_tx`.
        //
        // Use `into_split()` (OwnedReadHalf / OwnedWriteHalf) rather than `tokio::io::split`.
        //
        // Rationale for into_split vs tokio::io::split (AC-004 correctness):
        //   - `tokio::io::split` shares the underlying AsyncFd via Arc<Mutex<...>>.
        //     Dropping the write half does NOT send a FIN to the peer — the read half
        //     still holds a reference to the same fd, so the socket stays open.
        //   - `into_split()` gives independently-owned halves. Dropping `OwnedWriteHalf`
        //     calls `shutdown(Shutdown::Write)` on the underlying UnixStream, sending
        //     a FIN half-close to the client even while the read half is still live.
        //     The App's `spawn_ipc_reader` observes EOF and forwards
        //     `Err(IpcError::Disconnected)` to `inbound_rx` — the required AC-004 signal.
        let (mut read_half, mut write_half) = stream.into_split();

        let send_task = tokio::spawn(async move {
            while let Some(msg) = to_send_rx.recv().await {
                if write_framed(&mut write_half, &msg).await.is_err() {
                    // TUI disconnected — stop sending.
                    break;
                }
            }
            // When to_send_rx is exhausted (sender dropped), write_half (OwnedWriteHalf)
            // is dropped here.  OwnedWriteHalf::drop() calls shutdown(Shutdown::Write),
            // sending a FIN to the client.  The App's spawn_ipc_reader observes EOF and
            // forwards Err(IpcError::Disconnected) to inbound_rx — the AC-004 signal.
        });

        let recv_task = tokio::spawn(async move {
            // Read ClientToServer messages until the TUI closes the connection.
            while let Ok(msg) = read_framed::<_, ClientToServer>(&mut read_half).await {
                if received_tx.send(msg).await.is_err() {
                    // Receiver dropped — stop.
                    break;
                }
            }
        });

        // Wait for both tasks to finish (they exit when the TUI disconnects).
        let _ = tokio::join!(send_task, recv_task);
    });

    (to_send_tx, received_rx, handle, ready_rx)
}

/// Connect the TUI `App` to a MockDaemon via a real UDS socket.
///
/// Connects `UnixStream` to `sock_path`, reads the `InitialState` frame
/// to populate `app`, then calls `setup_ipc_streams_with_rx` to wire `app.ipc_tx`
/// and obtain the real inbound receiver.
///
/// Returns `(reader_handle, writer_handle, inbound_rx)` — the caller MUST:
///   - Drain `inbound_rx` via `handle_server_message` or `on_transport_event`
///     (mirroring the run() event loop's IPC drain path).
///   - Abort both task handles on teardown.
///
/// # Inbound dispatch contract
///
/// The returned `inbound_rx` carries `Result<ServerToClient, IpcError>`:
///   - `Ok(msg)` → dispatch via `handle_server_message(&mut app, msg)`.
///   - `Err(_)` → dispatch via `on_transport_event(&mut app, TransportEvent::Disconnected)`.
///
/// This mirrors the run() loop's `Ok(Ok(msg))` / `Ok(Err(e))` arms exactly.
async fn connect_app_to_mock_daemon(
    app: &mut App,
    sock_path: &std::path::Path,
) -> (
    tokio::task::JoinHandle<()>,
    tokio::task::JoinHandle<()>,
    tokio::sync::mpsc::Receiver<Result<ServerToClient, monocle_ipc::error::IpcError>>,
) {
    let mut stream = match UnixStream::connect(sock_path).await {
        Ok(s) => s,
        Err(e) => panic!("connect_app_to_mock_daemon: failed to connect to UDS: {e}"),
    };

    // Read InitialState and apply it to the app (BC-2.05.002 Invariant 1).
    let initial_frame = match read_framed::<_, ServerToClient>(&mut stream).await {
        Ok(f) => f,
        Err(e) => panic!("connect_app_to_mock_daemon: failed to read InitialState: {e}"),
    };
    match initial_frame {
        ServerToClient::InitialState {
            sessions,
            ring_tail,
            overlay_stack,
            drop_counter,
        } => {
            monocle_tui::app::on_initial_state(
                app,
                sessions,
                ring_tail,
                overlay_stack,
                drop_counter,
            );
        }
        other => panic!(
            "connect_app_to_mock_daemon: expected InitialState, got {:?}",
            other
        ),
    }

    // Wire the real IPC streams via setup_ipc_streams_with_rx.
    //
    // This is the production-grade seam (implementer commit befc415):
    //   - setup_ipc_streams_with_rx returns the real inbound_rx so tests can drain it.
    //   - spawn_ipc_reader (wired internally) reads ServerToClient frames from the UDS
    //     socket and forwards them via inbound_rx.
    //   - spawn_ipc_writer (wired internally) writes ClientToServer frames to the UDS
    //     socket from app.ipc_tx (set to Some by setup_ipc_streams_with_rx).
    //
    // Tests drain inbound_rx explicitly with tokio::time::timeout + recv() and dispatch
    // via handle_server_message (Ok arm) or on_transport_event (Err arm) — mirroring
    // the run() loop's IPC drain path exactly.
    let (read_half, write_half) = stream.into_split();
    let (rh, wh, inbound_rx) = setup_ipc_streams_with_rx(app, read_half, write_half);

    (rh, wh, inbound_rx)
}

// ---------------------------------------------------------------------------
// Short timeout constant for inbound receive assertions.
// ---------------------------------------------------------------------------

/// Timeout for receiving a single inbound IPC message in tests (500ms).
///
/// Used in tokio::time::timeout() guards when waiting for the MockDaemon to
/// deliver a ServerToClient frame through the real UDS socket → spawn_ipc_reader
/// → inbound_rx pipeline.
const SHORT_TIMEOUT: Duration = Duration::from_millis(500);

// ---------------------------------------------------------------------------
// AC-002 / BC-2.06.022 PC-2 — killer_scenario_accept (full 8-step happy path)
//
// Production path (REAL socket through REAL dispatch):
//   on_permission_prompt_queued → dispatch_key_event('y') → send_permission_decision
//   → ipc_tx → spawn_ipc_writer → UDS wire → MockDaemon receives ClientToServer::PermissionDecision
//   → MockDaemon sends ServerToClient::PermissionPromptResolved → spawn_ipc_reader → inbound_rx
//   → handle_server_message → on_permission_prompt_resolved → overlay_stack.retain() → Dashboard
// ---------------------------------------------------------------------------

/// BC-2.06.022 PC-2 / AC-002: Full E2E happy path — Bash prompt queued, `y` accepted,
/// daemon receives `PermissionDecision { Allow }` over the UDS socket, daemon sends
/// `PermissionPromptResolved`, the real `handle_server_message` dispatches to
/// `on_permission_prompt_resolved`, TUI stack empties, mode collapses to Dashboard.
///
/// This test drives the REAL inbound dispatch path:
///   MockDaemon writes PermissionPromptResolved → spawn_ipc_reader reads it from socket
///   → inbound_rx.recv() → handle_server_message → on_permission_prompt_resolved
///
/// Production symbols exercised:
///   - `setup_ipc_streams_with_rx`: wires app.ipc_tx + returns real inbound_rx
///   - `spawn_ipc_reader`: reads ServerToClient frames off the real UDS socket
///   - `handle_server_message`: routes PermissionPromptResolved to on_permission_prompt_resolved
///   - `on_permission_prompt_resolved`: retain() + Dashboard collapse (reached THROUGH dispatch)
#[tokio::test]
async fn test_BC_2_06_022_killer_scenario_accept() {
    // AC-007: unique socket path per test.
    let dir = tempfile::TempDir::new().expect("TempDir::new");
    let sock_path = dir.path().join("monocle_ks_accept.sock");

    let pid = Uuid::new_v4();

    // Step 1: spawn MockDaemon with one queued Bash prompt (P1).
    let (to_send_tx, mut received_rx, daemon_handle, ready_rx) =
        spawn_mock_daemon(sock_path.clone(), vec![bash_payload(pid)]).await;

    // Wait for the daemon listener to be ready (prevents connect-before-listen race).
    ready_rx.await.expect("MockDaemon ready signal");

    // Step 2: connect TUI App to MockDaemon and receive InitialState.
    let mut app = App::new(MonocleConfig::default());
    let (rh, wh, mut inbound_rx) = connect_app_to_mock_daemon(&mut app, &sock_path).await;

    // Step 3: assert TUI is in Overlay mode with overlay_stack.len() == 1.
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.022 PC-2 step 3: app.mode must be Overlay after InitialState with one prompt"
    );
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.022 PC-2 step 3: overlay_stack.len() must be 1 after InitialState"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid,
        "BC-2.06.022 PC-2: front modal must have the correct prompt_id"
    );

    // Step 4: inject `y` key — drives dispatch_key_event → send_permission_decision(Allow).
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    let outcome = dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('y')),
        &layers,
        &mut sessions_state,
    );
    assert_eq!(
        outcome,
        KeyOutcome::Continue,
        "BC-2.06.022 PC-2: `y` must not quit"
    );

    // Step 5: assert the decision physically arrived at the MockDaemon over UDS.
    // No sleep() — tokio timeout as the synchronization mechanism.
    let received = tokio::time::timeout(SHORT_TIMEOUT, received_rx.recv())
        .await
        .expect("timed out waiting for PermissionDecision at MockDaemon — outbound IPC not wired")
        .expect("MockDaemon received_rx channel closed unexpectedly");

    match received {
        ClientToServer::PermissionDecision {
            prompt_id,
            decision,
        } => {
            assert_eq!(
                prompt_id, pid,
                "BC-2.06.022 PC-2 step 5: PermissionDecision prompt_id must match the queued prompt"
            );
            assert_eq!(
                decision,
                PermissionDecisionKind::Allow,
                "BC-2.06.022 PC-2 step 5: `y` must send Allow (Accept-Once) decision"
            );
        }
    }

    // Step 6: Modal must still be in overlay_stack — not popped on send (BC-2.06.023).
    // The overlay is retained until PermissionPromptResolved arrives from the daemon.
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.022 PC-2 step 6 pre-condition: modal stays in stack until PermissionPromptResolved"
    );

    // Step 7: MockDaemon sends PermissionPromptResolved over the REAL UDS socket.
    //
    // The production inbound path:
    //   MockDaemon.to_send_tx → send_task → write_framed → UDS socket
    //   → spawn_ipc_reader → inbound_rx → handle_server_message → on_permission_prompt_resolved
    //
    // This exercises handle_server_message and on_permission_prompt_resolved through
    // real socket I/O — NOT direct function injection.
    to_send_tx
        .send(ServerToClient::PermissionPromptResolved { prompt_id: pid })
        .await
        .expect("step 7: to_send_tx.send failed — MockDaemon send task dead");

    // Drain inbound_rx: wait for spawn_ipc_reader to deliver the frame.
    let inbound_msg = tokio::time::timeout(SHORT_TIMEOUT, inbound_rx.recv())
        .await
        .expect(
            "step 7: timed out waiting for PermissionPromptResolved from inbound_rx \
                 — spawn_ipc_reader did not deliver the frame",
        )
        .expect("inbound_rx channel closed unexpectedly");

    // Dispatch through the REAL router — exercises handle_server_message →
    // on_permission_prompt_resolved (reached THROUGH dispatch, not called directly).
    let inbound_ok = inbound_msg.expect("step 7: inbound_rx contained Err (unexpected disconnect)");
    handle_server_message(&mut app, inbound_ok).unwrap();

    // Step 8: assert overlay_stack is empty and mode is Dashboard.
    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.022 PC-2 step 8: overlay_stack must be empty after PermissionPromptResolved \
         dispatched through handle_server_message"
    );
    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "BC-2.06.022 PC-2 step 8: AppMode must collapse to Dashboard {{ Sessions }} \
         per BC-2.06.001 empty-stack invariant"
    );

    // Clean up background tasks.
    drop(to_send_tx);
    rh.abort();
    wh.abort();
    let _ = tokio::time::timeout(Duration::from_millis(200), daemon_handle).await;
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.06.022 PC-3 — killer_scenario_multi_prompt (two-prompt FIFO stacking)
//
// Production path (REAL socket through REAL dispatch):
//   on_permission_prompt_queued x2 (FIFO) → stack len 2 →
//   dispatch_key_event('n') → PermissionDecision(Deny) for P1 (outbound over UDS) →
//   MockDaemon sends PermissionPromptResolved(P1) (inbound over UDS) →
//   handle_server_message → on_permission_prompt_resolved(P1) → len 1 →
//   dispatch_key_event('y') → PermissionDecision(Allow) for P2 (outbound over UDS) →
//   MockDaemon sends PermissionPromptResolved(P2) (inbound over UDS) →
//   handle_server_message → on_permission_prompt_resolved(P2) → empty → Dashboard
// ---------------------------------------------------------------------------

/// BC-2.06.022 PC-3 / AC-003: Two-prompt FIFO stacking and sequential resolution.
///
/// Exercises:
///   - `on_permission_prompt_queued` x2 → FIFO stack (P1 at front, P2 at back)
///   - `dispatch_key_event('n')` → Deny P1 (front), IPC write-back to MockDaemon
///   - MockDaemon sends `PermissionPromptResolved(P1)` over real UDS socket
///   - `handle_server_message` dispatches to `on_permission_prompt_resolved(P1)` → stack len 1
///   - `dispatch_key_event('y')` → Allow P2 (now front), IPC write-back
///   - MockDaemon sends `PermissionPromptResolved(P2)` over real UDS socket
///   - `handle_server_message` dispatches to `on_permission_prompt_resolved(P2)` → empty → Dashboard
///
/// Production symbols exercised (inbound):
///   - `setup_ipc_streams_with_rx` → real inbound_rx from spawn_ipc_reader
///   - `handle_server_message` → routes PermissionPromptResolved to on_permission_prompt_resolved
///   - `on_permission_prompt_resolved` reached THROUGH handle_server_message dispatch (not directly)
///
/// Non-vacuity: if the FIFO ordering is reversed (P2 at front), the first `n`
/// would resolve P2 instead of P1 and the second decision would target P1 — the
/// decision/resolve pairing assertions would fail.
#[tokio::test]
async fn test_BC_2_06_022_killer_scenario_multi_prompt() {
    // AC-007: unique socket path per test.
    let dir = tempfile::TempDir::new().expect("TempDir::new");
    let sock_path = dir.path().join("monocle_ks_multi.sock");

    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();

    // Step 1: Daemon emits two PermissionPromptQueued (both in InitialState for simplicity).
    let (to_send_tx, mut received_rx, daemon_handle, ready_rx) = spawn_mock_daemon(
        sock_path.clone(),
        vec![bash_payload(pid1), bash_payload(pid2)],
    )
    .await;
    ready_rx.await.expect("MockDaemon ready");

    let mut app = App::new(MonocleConfig::default());
    let (rh, wh, mut inbound_rx) = connect_app_to_mock_daemon(&mut app, &sock_path).await;

    // Step 2: assert stack len == 2 and P1 is at front (FIFO).
    assert_eq!(
        app.overlay_stack.len(),
        2,
        "BC-2.06.022 PC-3 step 2: overlay_stack must have 2 prompts after InitialState"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid1,
        "BC-2.06.022 PC-3 step 2: P1 must be at front (FIFO insertion order)"
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.022 PC-3 step 2: mode must be Overlay"
    );

    // Step 3: inject `n` → reject P1 (front).
    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();
    let outcome = dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('n')),
        &layers,
        &mut sessions_state,
    );
    assert_eq!(
        outcome,
        KeyOutcome::Continue,
        "BC-2.06.022 PC-3: `n` must not quit"
    );

    // Step 4: verify MockDaemon received Deny for P1 over UDS.
    let received1 = tokio::time::timeout(SHORT_TIMEOUT, received_rx.recv())
        .await
        .expect("timed out waiting for first PermissionDecision at MockDaemon")
        .expect("received_rx closed");
    match received1 {
        ClientToServer::PermissionDecision {
            prompt_id,
            decision,
        } => {
            assert_eq!(
                prompt_id, pid1,
                "BC-2.06.022 PC-3: first decision must target P1 (front)"
            );
            assert_eq!(
                decision,
                PermissionDecisionKind::Deny,
                "BC-2.06.022 PC-3: `n` must send Deny"
            );
        }
    }

    // Step 5: MockDaemon sends PermissionPromptResolved for P1 over the REAL UDS socket.
    // Then drain inbound_rx and dispatch through the REAL handle_server_message router.
    to_send_tx
        .send(ServerToClient::PermissionPromptResolved { prompt_id: pid1 })
        .await
        .expect("step 5: to_send_tx.send failed");

    let inbound1 = tokio::time::timeout(SHORT_TIMEOUT, inbound_rx.recv())
        .await
        .expect("step 5: timed out waiting for PermissionPromptResolved(P1) from inbound_rx")
        .expect("inbound_rx closed");
    handle_server_message(
        &mut app,
        inbound1.expect("step 5: unexpected disconnect in inbound_rx"),
    )
    .unwrap();

    // Stack must now have len == 1 (P2 remains at front), mode stays Overlay.
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.022 PC-3 step 5: stack must have len 1 after P1 resolved via handle_server_message"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid2,
        "BC-2.06.022 PC-3 step 5: P2 must now be at front"
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.022 PC-3 step 5: mode must remain Overlay while stack is non-empty"
    );

    // Step 6: inject `y` → accept P2 (now front).
    let outcome2 = dispatch_key_event(
        &mut app,
        &key(KeyCode::Char('y')),
        &layers,
        &mut sessions_state,
    );
    assert_eq!(
        outcome2,
        KeyOutcome::Continue,
        "BC-2.06.022 PC-3: second `y` must not quit"
    );

    // Step 7: verify MockDaemon received Allow for P2.
    let received2 = tokio::time::timeout(SHORT_TIMEOUT, received_rx.recv())
        .await
        .expect("timed out waiting for second PermissionDecision at MockDaemon")
        .expect("received_rx closed");
    match received2 {
        ClientToServer::PermissionDecision {
            prompt_id,
            decision,
        } => {
            assert_eq!(
                prompt_id, pid2,
                "BC-2.06.022 PC-3: second decision must target P2"
            );
            assert_eq!(
                decision,
                PermissionDecisionKind::Allow,
                "BC-2.06.022 PC-3: `y` must send Allow"
            );
        }
    }

    // Step 8: MockDaemon sends PermissionPromptResolved for P2 over the REAL UDS socket.
    to_send_tx
        .send(ServerToClient::PermissionPromptResolved { prompt_id: pid2 })
        .await
        .expect("step 8: to_send_tx.send failed");

    let inbound2 = tokio::time::timeout(SHORT_TIMEOUT, inbound_rx.recv())
        .await
        .expect("step 8: timed out waiting for PermissionPromptResolved(P2) from inbound_rx")
        .expect("inbound_rx closed");
    handle_server_message(
        &mut app,
        inbound2.expect("step 8: unexpected disconnect in inbound_rx"),
    )
    .unwrap();

    // Stack must be empty; mode must collapse to Dashboard.
    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.022 PC-3 step 8: overlay_stack must be empty after P2 resolved via handle_server_message"
    );
    assert!(
        matches!(app.mode, AppMode::Dashboard { .. }),
        "BC-2.06.022 PC-3 step 8: mode must collapse to Dashboard after last prompt resolved"
    );

    drop(to_send_tx);
    rh.abort();
    wh.abort();
    let _ = tokio::time::timeout(Duration::from_millis(200), daemon_handle).await;
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.06.022 PC-4 — killer_scenario_disconnect (disconnect clears overlay)
//
// Production path (REAL socket closure through REAL dispatch):
//   on_permission_prompt_queued x2 → stack len 2, Overlay mode →
//   MockDaemon drops to_send_tx (write half closes → EOF on socket) →
//   spawn_ipc_reader observes EOF → forwards Err(IpcError::Disconnected) to inbound_rx →
//   test drains inbound_rx (Err arm) → on_transport_event(Disconnected) →
//   overlay_stack.clear() → Dashboard
//
// Architecture rule (S-029 line 147-148): MockDaemon closes the socket to trigger this.
// ---------------------------------------------------------------------------

/// BC-2.06.022 PC-4 / AC-004: Disconnect while two prompts are queued clears the
/// overlay stack and collapses to Dashboard.
///
/// Exercises the REAL socket-closure disconnect path:
///   1. MockDaemon drops `to_send_tx` → its write task exits → socket write half closes.
///   2. `spawn_ipc_reader` (the real inbound reader task, wired by `setup_ipc_streams_with_rx`)
///      observes EOF on the UDS socket and forwards `Err(IpcError::Disconnected)` to `inbound_rx`.
///   3. Test drains `inbound_rx` and reaches the `Err` arm.
///   4. `on_transport_event(&mut app, TransportEvent::Disconnected)` is called — mirroring
///      the run() loop's `Ok(Err(e))` dispatch path exactly.
///   5. `on_transport_event` clears `overlay_stack` and transitions to Dashboard.
///
/// Production symbols exercised (inbound disconnect):
///   - `setup_ipc_streams_with_rx` → real spawn_ipc_reader wired to the socket
///   - `spawn_ipc_reader` → observes EOF, forwards Err(IpcError::Disconnected) to inbound_rx
///   - `on_transport_event(Disconnected)` → reached from the Err arm of inbound_rx drain
///     (mirrors the run() loop's Ok(Err(e)) → on_transport_event dispatch path)
///
/// Non-vacuity: if on_transport_event does NOT clear the overlay, the post-disconnect
/// stack-empty assertion fails. If the socket closure does NOT propagate to inbound_rx,
/// the timeout fires and the test panics before reaching the assertion.
#[tokio::test]
async fn test_BC_2_06_022_killer_scenario_disconnect() {
    // AC-007: unique socket path per test.
    let dir = tempfile::TempDir::new().expect("TempDir::new");
    let sock_path = dir.path().join("monocle_ks_disconnect.sock");

    let pid1 = Uuid::new_v4();
    let pid2 = Uuid::new_v4();

    // Step 1: Spawn daemon with 2 prompts, connect TUI.
    let (to_send_tx, _received_rx, daemon_handle, ready_rx) = spawn_mock_daemon(
        sock_path.clone(),
        vec![bash_payload(pid1), bash_payload(pid2)],
    )
    .await;
    ready_rx.await.expect("MockDaemon ready");

    let mut app = App::new(MonocleConfig::default());
    let (rh, wh, mut inbound_rx) = connect_app_to_mock_daemon(&mut app, &sock_path).await;

    // Precondition: 2 prompts in stack, Overlay mode.
    assert_eq!(
        app.overlay_stack.len(),
        2,
        "BC-2.06.022 PC-4 precondition: 2 prompts queued"
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.022 PC-4 precondition: in Overlay mode"
    );

    // Step 2: MockDaemon closes its socket write half by dropping to_send_tx.
    //
    // Dropping to_send_tx causes the MockDaemon's send_task to exit (its to_send_rx
    // is now exhausted), which drops the write_half of the MockDaemon's stream.
    // The App's spawn_ipc_reader observes EOF on the UDS socket and forwards
    // Err(IpcError::Disconnected) to inbound_rx.
    drop(to_send_tx);

    // Step 3: Drain inbound_rx — wait for spawn_ipc_reader to deliver the disconnect signal.
    //
    // The Err arm mirrors the run() loop's Ok(Err(e)) dispatch path:
    //   Ok(Err(e)) => on_transport_event(&mut app, TransportEvent::Disconnected)
    //
    // We use a longer timeout here because EOF propagation may require the MockDaemon's
    // send_task to observe the channel close and exit first.
    let disconnect_signal = tokio::time::timeout(Duration::from_millis(1000), inbound_rx.recv())
        .await
        .expect(
            "BC-2.06.022 PC-4 step 3: timed out waiting for disconnect signal from inbound_rx \
                 — spawn_ipc_reader did not detect socket EOF",
        )
        .expect(
            "inbound_rx channel itself closed unexpectedly (reader task exited without sending)",
        );

    assert!(
        disconnect_signal.is_err(),
        "BC-2.06.022 PC-4 step 3: inbound_rx must carry Err (disconnect) after MockDaemon \
         socket closure, got Ok (unexpected message)"
    );

    // Step 4: Dispatch through the REAL on_transport_event handler.
    // This mirrors the run() loop's Ok(Err(e)) arm exactly.
    on_transport_event(&mut app, TransportEvent::Disconnected);

    // Step 5: Assert overlay_stack is cleared.
    assert!(
        app.overlay_stack.is_empty(),
        "BC-2.06.022 PC-4 step 5: overlay_stack must be empty after TransportEvent::Disconnected \
         dispatched from real socket EOF (BC-2.06.016 PC-1 / BC-2.06.022 PC-4)"
    );

    // Step 6: Assert mode is Dashboard.
    assert!(
        matches!(
            app.mode,
            AppMode::Dashboard {
                focused: FocusSnapshot::Sessions
            }
        ),
        "BC-2.06.022 PC-4 step 6: mode must collapse to Dashboard {{ Sessions }} after disconnect"
    );

    rh.abort();
    wh.abort();
    let _ = tokio::time::timeout(Duration::from_millis(200), daemon_handle).await;
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.06.022 PC-5 — killer_scenario_esc_no_reject (Esc is identity)
//
// Production path (MockDaemon form):
//   on_permission_prompt_queued → Overlay mode, stack len 1 →
//   dispatch_key_event(Esc) x3 → NO IPC send, mode stays Overlay, stack stays len 1 →
//   assert MockDaemon received_rx stays EMPTY within SHORT_TIMEOUT
// ---------------------------------------------------------------------------

/// BC-2.06.022 PC-5 / AC-005: Pressing Esc three times in Overlay mode is an identity
/// no-op — stack unchanged, mode stays Overlay, NO `PermissionDecision` sent.
///
/// Exercises (MockDaemon form — real UDS socket, outbound I/O observed at daemon):
///   - `setup_ipc_streams_with_rx` → app.ipc_tx wired to the real UDS socket
///   - `dispatch_key_event(Esc)` x3 → resolves to `Action::Esc` which is identity
///     in Overlay mode (BC-2.06.014 PC-1 / transition(Overlay, Esc) = Overlay)
///   - MockDaemon `received_rx` must stay EMPTY — no ClientToServer frame arrives
///
/// Non-vacuity: if Esc were to send a decision (wrong) or pop the stack (wrong),
/// the stack-len or mode assertion would fail. If any IPC message is sent, the
/// SHORT_TIMEOUT wait on received_rx.recv() would succeed and the assertion panics.
#[tokio::test]
async fn test_BC_2_06_022_killer_scenario_esc_no_reject() {
    // AC-007: unique socket path per test.
    let dir = tempfile::TempDir::new().expect("TempDir::new");
    let sock_path = dir.path().join("monocle_ks_esc.sock");

    let pid = Uuid::new_v4();

    // Spawn MockDaemon with one prompt; connect TUI.
    let (to_send_tx, mut received_rx, daemon_handle, ready_rx) =
        spawn_mock_daemon(sock_path.clone(), vec![bash_payload(pid)]).await;
    ready_rx.await.expect("MockDaemon ready");

    let mut app = App::new(MonocleConfig::default());
    let (rh, wh, _inbound_rx) = connect_app_to_mock_daemon(&mut app, &sock_path).await;

    let layers = build_builtin_binding_layers();
    let mut sessions_state = SessionsPanelState::default();

    // Precondition: in Overlay mode, stack len 1.
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.022 PC-5 precondition: one prompt in stack"
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.022 PC-5 precondition: in Overlay mode"
    );

    // Step 1: inject Esc three times.
    for i in 0..3 {
        let outcome =
            dispatch_key_event(&mut app, &key(KeyCode::Esc), &layers, &mut sessions_state);
        assert_eq!(
            outcome,
            KeyOutcome::Continue,
            "BC-2.06.022 PC-5: Esc press {} must not quit",
            i + 1
        );
    }

    // Step 2: assert stack unchanged (len 1) and mode still Overlay.
    assert_eq!(
        app.overlay_stack.len(),
        1,
        "BC-2.06.022 PC-5 step 2: overlay_stack must remain len 1 after 3x Esc \
         (Esc is identity in Overlay mode per BC-2.06.014 PC-1)"
    );
    assert!(
        matches!(app.mode, AppMode::Overlay { .. }),
        "BC-2.06.022 PC-5 step 2: mode must remain Overlay after 3x Esc"
    );
    assert_eq!(
        app.overlay_stack.front().unwrap().prompt_id,
        pid,
        "BC-2.06.022 PC-5 step 2: front modal must be unchanged"
    );

    // Step 3: assert NO PermissionDecision arrived at the MockDaemon.
    //
    // Use tokio::time::timeout: if received_rx.recv() returns within SHORT_TIMEOUT,
    // a decision was wrongly sent — assert failure. If the timeout fires (Err), the
    // channel is empty — correct behavior.
    let no_decision = tokio::time::timeout(SHORT_TIMEOUT, received_rx.recv()).await;
    assert!(
        no_decision.is_err(),
        "BC-2.06.022 PC-5 step 3: Esc must NOT send any PermissionDecision to MockDaemon \
         (BC-2.06.014 PC-1: Esc is identity, no IPC send)"
    );

    drop(to_send_tx);
    rh.abort();
    wh.abort();
    let _ = tokio::time::timeout(Duration::from_millis(200), daemon_handle).await;
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.06.022 PC-6 — killer_scenario_edit_diff (diff rendered in TestBackend)
//
// Production path:
//   on_permission_prompt_queued with Edit payload → payload_to_modal produces
//   ToolPayload::Edit { old_content, new_content, path } →
//   render_overlay_widget → render_edit_payload → similar LINE diff →
//   TestBackend Buffer contains "+" line (green) and "hello world" text
// ---------------------------------------------------------------------------

/// BC-2.06.022 PC-6 / AC-006: Edit payload renders a unified diff in TestBackend.
///
/// Canonical test vector (AC-006): `old_content: "hello\n"`, `new_content: "hello world\n"`,
/// `path: "/tmp/test.txt"`. `similar::TextDiff::from_lines` produces a LINE-level diff:
///   - Delete line: `"hello\n"` (old)
///   - Insert line: `"hello world\n"` (new)
///
/// The rendered buffer must contain:
///   - `"hello world"` text (the inserted line content).
///   - `"hello"` text (the unchanged prefix in context or in the inserted line).
///   - A `"+"` symbol colored `Color::Green` (addition line prefix per BC-2.06.010 PC-2).
///
/// Exercises:
///   - `on_permission_prompt_queued` with Edit payload →
///     `payload_to_modal` → `ToolPayload::Edit { old_content, new_content, path }`
///   - `render_overlay_widget` → `render_edit_payload` → `similar` LINE diff
///   - `ratatui::backend::TestBackend` for headless render — no real terminal
///
/// Non-vacuity: if `render_edit_payload` renders nothing (blank body), neither the
/// `"+"` symbol check nor the `"hello world"` text check would pass. If `payload_to_modal`
/// falls back to `ToolPayload::Generic` (wrong path), the diff path is not exercised.
#[test]
fn test_BC_2_06_022_killer_scenario_edit_diff() {
    use monocle_tui::ui::overlay_widget::render_overlay_widget;
    use ratatui::{backend::TestBackend, layout::Rect, style::Color, Terminal};

    let pid = Uuid::new_v4();
    let mut app = App::new(MonocleConfig::default());

    // Step 1: queue an Edit prompt via on_permission_prompt_queued.
    on_permission_prompt_queued(&mut app, edit_payload(pid));

    // Assert payload_to_modal produced ToolPayload::Edit (not Generic).
    // The edit_payload fixture has old_content=Some("hello\n"), new_content=Some("hello world\n"),
    // and tool_input contains "path". Per payload_to_modal: has_content=true, path present
    // → ToolPayload::Edit { .. }.
    let modal = app
        .overlay_stack
        .front()
        .expect("BC-2.06.022 PC-6: overlay_stack must have one modal");
    assert!(
        matches!(
            &modal.tool_payload,
            monocle_core::tui::state::ToolPayload::Edit { .. }
        ),
        "BC-2.06.022 PC-6: payload_to_modal must produce ToolPayload::Edit for Edit payload \
         with non-None old/new_content and 'path' key in tool_input"
    );

    // Step 2: render via TestBackend — no real terminal required.
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("Terminal::new");

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            // render_overlay_widget renders the full overlay including the diff body.
            render_overlay_widget(modal, app.overlay_stack.len(), area, frame);
        })
        .expect("terminal.draw must succeed with TestBackend");

    // Step 3: inspect the rendered buffer.
    let buffer = terminal.backend().buffer().clone();
    let area = buffer.area();

    // Collect all cell symbols into a single string for text searches.
    let full_text: String = (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buffer[(x, y)].symbol().to_string())
        .collect();

    // Assert "hello world" appears in the rendered buffer (the inserted line content).
    // similar::TextDiff::from_lines produces a LINE diff: the new line is "hello world\n"
    // rendered with a "+" prefix at the start of the line.
    assert!(
        full_text.contains("hello world"),
        "BC-2.06.022 PC-6 step 3: rendered buffer must contain 'hello world' \
         (the inserted line from the similar LINE diff of new_content)"
    );

    // Assert "hello" (context — present within the inserted "hello world" line) appears.
    assert!(
        full_text.contains("hello"),
        "BC-2.06.022 PC-6 step 3: rendered buffer must contain 'hello' \
         (prefix of the inserted 'hello world' line)"
    );

    // Assert a "+" character appears in the buffer (the addition prefix from the LINE diff).
    assert!(
        full_text.contains('+'),
        "BC-2.06.022 PC-6 step 3: rendered buffer must contain '+' \
         (addition line prefix in similar LINE diff for 'hello world\\n')"
    );

    // Assert the "+" cell is colored green (Color::Green) per BC-2.06.010 PC-2 color coding.
    // Walk cells to find the first "+" symbol and verify its fg color.
    let plus_cell_color = (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .find(|&(x, y)| buffer[(x, y)].symbol() == "+")
        .map(|(x, y)| buffer[(x, y)].style().fg);

    assert!(
        plus_cell_color.is_some(),
        "BC-2.06.022 PC-6 step 3: '+' cell not found in buffer — diff not rendered"
    );
    assert_eq!(
        plus_cell_color.unwrap(),
        Some(Color::Green),
        "BC-2.06.022 PC-6 step 3: '+' (addition) cell must be Color::Green \
         (BC-2.06.010 PC-2: addition lines are rendered green)"
    );
}

// ---------------------------------------------------------------------------
// AC-007 (implicit) — test_isolation
//
// Each test above uses its own tempfile::TempDir socket path.
// This explicit test proves the isolation contract: two concurrent test
// instances with different socket paths do NOT interfere.
// ---------------------------------------------------------------------------

/// BC-2.06.022 INV-1 / AC-007: Test isolation — two tests with separate TempDir
/// socket paths can run in the same process without interference.
///
/// This test spins up TWO independent MockDaemon+App pairs simultaneously and
/// verifies they route decisions to the correct daemon without cross-contamination.
/// If socket paths were shared, one of the two `connect` calls would fail or the
/// wrong daemon would receive the decision.
///
/// Exercises: `tempfile::TempDir` isolation and parallel-safe socket binding.
#[tokio::test]
async fn test_BC_2_06_022_killer_scenario_isolation_parallel_safe() {
    let dir_a = tempfile::TempDir::new().expect("TempDir A");
    let dir_b = tempfile::TempDir::new().expect("TempDir B");
    let sock_a = dir_a.path().join("monocle_isolation_a.sock");
    let sock_b = dir_b.path().join("monocle_isolation_b.sock");

    let pid_a = Uuid::new_v4();
    let pid_b = Uuid::new_v4();

    // Start two independent MockDaemons.
    let (_send_a, mut recv_a, daemon_a, ready_a) =
        spawn_mock_daemon(sock_a.clone(), vec![bash_payload(pid_a)]).await;
    let (_send_b, mut recv_b, daemon_b, ready_b) =
        spawn_mock_daemon(sock_b.clone(), vec![bash_payload(pid_b)]).await;
    ready_a.await.expect("Daemon A ready");
    ready_b.await.expect("Daemon B ready");

    // Connect two independent Apps.
    let mut app_a = App::new(MonocleConfig::default());
    let mut app_b = App::new(MonocleConfig::default());
    let (rh_a, wh_a, _) = connect_app_to_mock_daemon(&mut app_a, &sock_a).await;
    let (rh_b, wh_b, _) = connect_app_to_mock_daemon(&mut app_b, &sock_b).await;

    let layers = build_builtin_binding_layers();
    let mut ss_a = SessionsPanelState::default();
    let mut ss_b = SessionsPanelState::default();

    // Send 'y' on App A — should reach Daemon A only.
    dispatch_key_event(&mut app_a, &key(KeyCode::Char('y')), &layers, &mut ss_a);
    // Send 'n' on App B — should reach Daemon B only.
    dispatch_key_event(&mut app_b, &key(KeyCode::Char('n')), &layers, &mut ss_b);

    // Verify Daemon A received Allow for pid_a.
    let msg_a = tokio::time::timeout(SHORT_TIMEOUT, recv_a.recv())
        .await
        .expect("timed out on Daemon A recv")
        .expect("Daemon A recv_rx closed");
    match msg_a {
        ClientToServer::PermissionDecision {
            prompt_id,
            decision,
        } => {
            assert_eq!(
                prompt_id, pid_a,
                "isolation: Daemon A must receive decision for pid_a, not pid_b"
            );
            assert_eq!(
                decision,
                PermissionDecisionKind::Allow,
                "isolation: App A `y` must route Allow to Daemon A"
            );
        }
    }

    // Verify Daemon B received Deny for pid_b (not contaminated by App A's Allow).
    let msg_b = tokio::time::timeout(SHORT_TIMEOUT, recv_b.recv())
        .await
        .expect("timed out on Daemon B recv")
        .expect("Daemon B recv_rx closed");
    match msg_b {
        ClientToServer::PermissionDecision {
            prompt_id,
            decision,
        } => {
            assert_eq!(
                prompt_id, pid_b,
                "isolation: Daemon B must receive decision for pid_b, not pid_a"
            );
            assert_eq!(
                decision,
                PermissionDecisionKind::Deny,
                "isolation: App B `n` must route Deny to Daemon B"
            );
        }
    }

    rh_a.abort();
    wh_a.abort();
    rh_b.abort();
    wh_b.abort();
    let _ = tokio::time::timeout(Duration::from_millis(200), daemon_a).await;
    let _ = tokio::time::timeout(Duration::from_millis(200), daemon_b).await;
}
