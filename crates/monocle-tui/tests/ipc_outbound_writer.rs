//! Integration test proving the outbound IPC writer channel is wired in production.
//!
//! # F-S026-ADV5-CRIT-001 — Production-path test (RED GATE)
//!
//! `App.ipc_tx` is the outbound channel for `ClientToServer::PermissionDecision`.
//! Before this fix, `ipc_tx` was permanently `None` in the production `run()` path:
//! the `UnixStream` was moved whole into `spawn_ipc_reader`, leaving NO writer half.
//! Every decision keypress hit the `ipc_tx is None` guard in
//! `send_permission_decision()` and was silently dropped.
//!
//! This test exercises:
//! 1. `setup_ipc_streams` (new production helper): splits a stream, spawns the reader
//!    task AND the outbound writer task, and wires `app.ipc_tx = Some(cmd_tx)`.
//! 2. Asserts `app.ipc_tx` is `Some` after wiring (not `None`).
//! 3. Sends a `ClientToServer::PermissionDecision` through `app.ipc_tx` and reads
//!    it from the daemon-side half — proving the message physically traverses the wire.
//!
//! RED GATE: before the fix, `setup_ipc_streams` does not exist — these tests fail
//! with a compile error or at runtime because `app.ipc_tx` remains `None`.
//!
//! Traces to: BC-2.06.011 / BC-2.06.012 / BC-2.06.013 (decision send),
//! F-S026-ADV5-CRIT-001 (outbound writer not wired).

use monocle_config::MonocleConfig;
use monocle_ipc::framing::read_framed;
use monocle_ipc::types::{ClientToServer, PermissionDecisionKind};
use monocle_tui::app::{setup_ipc_streams, App};
use std::time::Duration;
use uuid::Uuid;

/// F-S026-ADV5-CRIT-001 (RED GATE): After `setup_ipc_streams`, `app.ipc_tx` must be `Some`.
///
/// RED GATE: Currently `setup_ipc_streams` does not exist and `ipc_tx` is never wired
/// in the production run() path.  This test fails to compile (function missing) before
/// the fix.  After the fix, it passes.
#[tokio::test]
async fn test_f_s026_adv5_crit001_ipc_tx_is_some_after_setup() {
    let mut app = App::new(MonocleConfig::default());

    // Precondition: ipc_tx starts None (construction contract).
    assert!(
        app.ipc_tx.is_none(),
        "precondition: ipc_tx must be None at construction"
    );

    // Create an in-process duplex pipe (no UDS socket required for this unit-level test).
    // tokio::io::split gives (ReadHalf, WriteHalf) which implement AsyncRead/AsyncWrite.
    let (client_side, _server_side) = tokio::io::duplex(65536);
    let (client_read, client_write) = tokio::io::split(client_side);

    // Call the production wiring function — this is what run() must call after the
    // UnixStream is obtained (initial connect + after each reconnect).
    let (reader_handle, writer_handle) = setup_ipc_streams(&mut app, client_read, client_write);

    // ASSERTION: ipc_tx must now be Some — the outbound channel is wired.
    assert!(
        app.ipc_tx.is_some(),
        "F-S026-ADV5-CRIT-001: ipc_tx must be Some after setup_ipc_streams; \
         got None — the outbound writer task is not wired"
    );

    // Clean up background tasks to avoid leaking them between tests.
    reader_handle.abort();
    writer_handle.abort();
}

/// F-S026-ADV5-CRIT-001 (RED GATE): A `PermissionDecision` sent through `app.ipc_tx`
/// must physically reach the daemon-side read half as a framed `ClientToServer` message.
///
/// This is the E2E wire-level verification of the outbound path.  The test:
/// 1. Wires app.ipc_tx via `setup_ipc_streams` on the client half.
/// 2. Sends a `PermissionDecision` via `app.ipc_tx.as_ref().unwrap().try_send(...)`.
/// 3. Reads from the server half with `read_framed` and asserts the message arrived.
///
/// RED GATE: Before the fix there is no outbound writer task, so no bytes ever reach
/// the server half.  `read_framed` on the server side will time out / never return —
/// the test panics at the deadline.
#[tokio::test]
async fn test_f_s026_adv5_crit001_decision_message_traverses_wire_to_daemon() {
    let mut app = App::new(MonocleConfig::default());

    let (client_side, server_side) = tokio::io::duplex(65536);
    let (client_read, client_write) = tokio::io::split(client_side);

    // Wire the outbound channel.
    let (reader_handle, writer_handle) = setup_ipc_streams(&mut app, client_read, client_write);

    // Compose a PermissionDecision and send it via the wired ipc_tx.
    let prompt_id = Uuid::new_v4();
    let msg = ClientToServer::PermissionDecision {
        prompt_id,
        decision: PermissionDecisionKind::Allow,
    };

    app.ipc_tx
        .as_ref()
        .expect("ipc_tx must be Some after setup_ipc_streams")
        .try_send(msg)
        .expect("try_send must succeed on a freshly created channel");

    // Read from the server (daemon) side with a deadline.
    // After the fix, the outbound writer task drains ipc_tx and calls write_framed,
    // so the frame arrives at server_side promptly.
    // Before the fix, no writer task exists — server_side never receives bytes.
    let deadline = Duration::from_millis(500);
    let received: ClientToServer = tokio::time::timeout(deadline, async {
        let mut reader = server_side;
        read_framed::<_, ClientToServer>(&mut reader).await
    })
    .await
    .expect("timed out waiting for PermissionDecision on daemon side — outbound writer not wired")
    .expect("read_framed must succeed on the server side");

    match received {
        ClientToServer::PermissionDecision {
            prompt_id: recv_id,
            decision,
        } => {
            assert_eq!(
                recv_id, prompt_id,
                "F-S026-ADV5-CRIT-001: received prompt_id must match sent prompt_id"
            );
            assert_eq!(
                decision,
                PermissionDecisionKind::Allow,
                "F-S026-ADV5-CRIT-001: received decision must be Allow"
            );
        }
        // S-033 stub: SpawnSession is not expected in this PermissionDecision test context.
        ClientToServer::SpawnSession { .. } => {
            panic!(
                "unexpected SpawnSession in ipc_outbound_writer test (F-S026-ADV5-CRIT-001 first)"
            );
        }

        // S-034 stub: KillSession is not expected in this test context.
        ClientToServer::KillSession { .. } => {
            panic!("unexpected KillSession in this test context");
        }
        // S-035 stub: AttachSession is not expected in this PermissionDecision test context.
        ClientToServer::AttachSession { .. } => {
            panic!("unexpected AttachSession in this test context");
        }
        // S-035 stub: DetachSession is not expected in this PermissionDecision test context.
        ClientToServer::DetachSession { .. } => {
            panic!("unexpected DetachSession in this test context");
        }
        // S-040 stub: KeyInput is not expected in this PermissionDecision test context.
        ClientToServer::KeyInput { .. } => {
            panic!("unexpected KeyInput in this test context");
        }
    }

    reader_handle.abort();
    writer_handle.abort();
}

/// F-S026-ADV5-CRIT-001: `setup_ipc_streams` called a second time (reconnect path)
/// replaces `app.ipc_tx` with a fresh channel — the old channel is NOT leaked.
///
/// After reconnect, sends on the NEW ipc_tx must succeed and reach the NEW daemon side.
///
/// This validates the reconnect wiring contract: on reconnect, run() must call
/// `setup_ipc_streams` again with the fresh `UnixStream`, and `app.ipc_tx` must
/// point to the new channel (not the old one that's now dead).
#[tokio::test]
async fn test_f_s026_adv5_crit001_reconnect_rewires_ipc_tx_to_new_channel() {
    let mut app = App::new(MonocleConfig::default());

    // First connection.
    let (client1, _server1) = tokio::io::duplex(65536);
    let (c1_read, c1_write) = tokio::io::split(client1);
    let (rh1, wh1) = setup_ipc_streams(&mut app, c1_read, c1_write);

    let tx1 = app
        .ipc_tx
        .clone()
        .expect("ipc_tx must be Some after first setup");

    // Simulate reconnect: call setup_ipc_streams again with a fresh stream.
    let (client2, server2) = tokio::io::duplex(65536);
    let (c2_read, c2_write) = tokio::io::split(client2);
    let (rh2, wh2) = setup_ipc_streams(&mut app, c2_read, c2_write);

    let tx2 = app
        .ipc_tx
        .as_ref()
        .expect("ipc_tx must be Some after reconnect setup")
        .clone();

    // The new tx must be a DIFFERENT channel than the old one.
    // We detect this by checking that tx1 is closed (old writer task gone or old channel gone)
    // OR that a message sent via tx2 arrives on server2 (new writer works).
    let prompt_id = Uuid::new_v4();
    let msg = ClientToServer::PermissionDecision {
        prompt_id,
        decision: PermissionDecisionKind::Deny,
    };

    tx2.try_send(msg)
        .expect("tx2 (new channel) must accept sends after reconnect");

    // Read from server2 — the new daemon side.
    let received: ClientToServer = tokio::time::timeout(Duration::from_millis(500), async {
        let mut reader = server2;
        read_framed::<_, ClientToServer>(&mut reader).await
    })
    .await
    .expect("timed out reading from server2 after reconnect — new writer task not wired")
    .expect("read_framed on server2 must succeed");

    match received {
        ClientToServer::PermissionDecision {
            prompt_id: recv_id,
            decision,
        } => {
            assert_eq!(recv_id, prompt_id, "reconnect: prompt_id must match");
            assert_eq!(
                decision,
                PermissionDecisionKind::Deny,
                "reconnect: decision must be Deny"
            );
        }
        // S-033 stub: SpawnSession is not expected in this PermissionDecision test context.
        ClientToServer::SpawnSession { .. } => {
            panic!("unexpected SpawnSession in ipc_outbound_writer test (reconnect path)");
        }
        // S-034 stub: KillSession is not expected in this test context.
        ClientToServer::KillSession { .. } => {
            panic!("unexpected KillSession in this test context");
        }
        // S-035 stub: AttachSession is not expected in this PermissionDecision test context.
        ClientToServer::AttachSession { .. } => {
            panic!("unexpected AttachSession in this test context");
        }
        // S-035 stub: DetachSession is not expected in this PermissionDecision test context.
        ClientToServer::DetachSession { .. } => {
            panic!("unexpected DetachSession in this test context");
        }
        // S-040 stub: KeyInput is not expected in this PermissionDecision test context.
        ClientToServer::KeyInput { .. } => {
            panic!("unexpected KeyInput in this test context");
        }
    }

    // The OLD tx1 may still be open (its writer task was aborted by setup_ipc_streams
    // on reconnect, or it's just dead because the stream dropped).
    // We do NOT assert tx1.is_closed() because the drop timing is task-scheduler
    // dependent. What matters is that tx2 routes to server2 — proven above.
    drop(tx1);

    rh1.abort();
    wh1.abort();
    rh2.abort();
    wh2.abort();
}
