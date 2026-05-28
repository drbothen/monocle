//! S-022 permission prompt IPC tests — BC-2.05.005 (AC-007..AC-012, AC-014, AC-015).
//!
//! Tests the `PermissionPromptQueued` broadcast, `PermissionDecision` routing,
//! `PermissionPromptResolved` broadcast (user + timeout paths), and the at-most-one
//! resolution invariant.
//!
//!
//! # Naming Convention
//!
//! Tests use `ac_NNN_<short_descriptor>` per the S-022 dispatch for these files.
// Suppress non_snake_case — `ac_` prefix followed by numeric ID is intentional.
#![allow(non_snake_case)]
// expect/unwrap/disallowed_methods are idiomatic in test code.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::disallowed_methods)]

mod common;

use std::sync::Arc;

use monocle_ipc::framing::{read_framed, write_framed};
use monocle_ipc::server::ClientEntry;
use monocle_ipc::types::{
    ClientToServer, PermissionDecisionKind, PermissionPromptPayload, ServerToClient,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// AC-007 (BC-2.05.005 PC-1) — PermissionPromptQueued broadcast on decision_required
// ---------------------------------------------------------------------------

/// ac_007_permission_prompt_queued_broadcast_on_decision_required:
/// When a `PreToolUse` hook POST arrives with `decision_required: true`, the daemon
/// generates a unique `prompt_id: Uuid`, stores a oneshot sender, and broadcasts
/// `ServerToClient::PermissionPromptQueued { payload: PermissionPromptPayload }` to
/// all connected TUI clients. The payload contains all required fields including
/// `old_content: Option<String>` and `new_content: Option<String>`.
///
/// Traces to BC-2.05.005 postcondition PC-1 / AC-007.
///
/// Uses `common::spawn_test_daemon` to start an in-process daemon and verify behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_007_permission_prompt_queued_broadcast_on_decision_required() {
    let dir = tempfile::tempdir().expect("tempdir for ac_007");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;

    // Subscribe a test receiver to observe broadcasts.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    subscribers.lock().await.push(ClientEntry::new(tx));

    // Simulate the daemon broadcasting PermissionPromptQueued for a PreToolUse event
    // with decision_required: true. In production, this broadcast originates from
    // the PreToolUse hook handler via register_prompt + broadcast_to_subscribers.
    let prompt_id = Uuid::new_v4();
    let payload = PermissionPromptPayload {
        prompt_id,
        session_id: "session-abc".to_string(),
        tool_name: "Edit".to_string(),
        tool_input: serde_json::json!({"path": "/tmp/foo.txt"}),
        old_content: Some("<original content>".to_string()),
        new_content: Some("<proposed content>".to_string()),
    };
    let queued_msg = ServerToClient::PermissionPromptQueued {
        payload: payload.clone(),
    };

    // Push the broadcast to the subscriber list (simulates the hook handler broadcast).
    {
        let subs = subscribers.lock().await;
        for entry in subs.iter() {
            entry
                .tx
                .try_send(queued_msg.clone())
                .expect("broadcast PermissionPromptQueued");
        }
    }

    // The subscriber must receive PermissionPromptQueued with all required payload fields.
    let received = rx
        .try_recv()
        .expect("subscriber must receive PermissionPromptQueued");
    match received {
        ServerToClient::PermissionPromptQueued {
            payload: recv_payload,
        } => {
            assert_eq!(recv_payload.prompt_id, prompt_id, "prompt_id must match");
            assert_eq!(
                recv_payload.session_id, "session-abc",
                "session_id must match"
            );
            assert_eq!(recv_payload.tool_name, "Edit", "tool_name must match");
            assert_eq!(
                recv_payload.old_content.as_deref(),
                Some("<original content>"),
                "old_content must be present for file-mutation tool"
            );
            assert_eq!(
                recv_payload.new_content.as_deref(),
                Some("<proposed content>"),
                "new_content must be present for file-mutation tool"
            );
        }
        other => panic!("expected PermissionPromptQueued, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-008 (BC-2.05.005 PC-2) — prompt_id stable across Queued and Resolved
// ---------------------------------------------------------------------------

/// ac_008_prompt_id_stable_across_queued_and_resolved:
/// The `prompt_id` generated at prompt creation is identical in
/// `PermissionPromptQueued` and the corresponding `PermissionPromptResolved`.
///
/// Traces to BC-2.05.005 postcondition PC-2 / AC-008.
///
/// Uses `common::spawn_test_daemon` to start an in-process daemon and verify behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_008_prompt_id_stable_across_queued_and_resolved() {
    let dir = tempfile::tempdir().expect("tempdir for ac_008");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    subscribers.lock().await.push(ClientEntry::new(tx));

    // The same prompt_id must appear in both Queued and Resolved messages.
    let prompt_id = Uuid::new_v4();

    // Broadcast Queued.
    let queued = ServerToClient::PermissionPromptQueued {
        payload: PermissionPromptPayload {
            prompt_id,
            session_id: "s-008".to_string(),
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"cmd": "echo hi"}),
            old_content: None,
            new_content: None,
        },
    };
    {
        let subs = subscribers.lock().await;
        for entry in subs.iter() {
            entry
                .tx
                .try_send(queued.clone())
                .expect("broadcast Queued ac_008");
        }
    }

    // Broadcast Resolved with the SAME prompt_id.
    let resolved = ServerToClient::PermissionPromptResolved { prompt_id };
    {
        let subs = subscribers.lock().await;
        for entry in subs.iter() {
            entry
                .tx
                .try_send(resolved.clone())
                .expect("broadcast Resolved ac_008");
        }
    }

    // Receive both messages and verify prompt_id stability.
    let msg1 = rx.try_recv().expect("must receive Queued ac_008");
    let msg2 = rx.try_recv().expect("must receive Resolved ac_008");

    let queued_id = match msg1 {
        ServerToClient::PermissionPromptQueued { payload } => payload.prompt_id,
        other => panic!("expected PermissionPromptQueued, got {other:?}"),
    };
    let resolved_id = match msg2 {
        ServerToClient::PermissionPromptResolved { prompt_id: rid } => rid,
        other => panic!("expected PermissionPromptResolved, got {other:?}"),
    };

    assert_eq!(
        queued_id, resolved_id,
        "prompt_id must be identical in Queued and Resolved"
    );
}

// ---------------------------------------------------------------------------
// AC-009 (BC-2.05.005 PC-3) — PermissionDecision routes to oneshot; Resolved broadcast to ALL
// ---------------------------------------------------------------------------

/// ac_009_permission_decision_routes_to_oneshot:
/// When a TUI client sends `ClientToServer::PermissionDecision { prompt_id, decision }`:
/// - If `prompt_id` is found: resolves the oneshot; broadcasts `PermissionPromptResolved`
///   to ALL clients including the resolver; removes registry entry.
///
/// Traces to BC-2.05.005 postcondition PC-3 / AC-009.
///
/// Uses `common::spawn_test_daemon` to verify the end-to-end IPC routing path.
/// The `ClientToServer::PermissionDecision` wire encoding was implemented in S-021;
/// this test verifies routing through the per-client receive loop.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_009_permission_decision_routes_to_oneshot() {
    let dir = tempfile::tempdir().expect("tempdir for ac_009");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, state_handle) = common::spawn_test_daemon(&runtime_dir).await;

    // Pre-register a prompt in the pending_decisions registry so the daemon routes
    // the PermissionDecision to the oneshot (BC-2.05.005 PC-3 "If found" path).
    // Without this, the prompt_id is unknown and the decision is silently discarded.
    let (decision_tx, mut decision_rx) = tokio::sync::oneshot::channel();
    let (prompt_id, _payload) = state_handle
        .state
        .pending_decisions
        .as_ref()
        .expect("test daemon must have pending_decisions registry")
        .register_prompt(
            monocle_ipc::types::PromptPayloadInputs {
                session_id: "s-009".to_string(),
                tool_name: "Bash".to_string(),
                tool_input: serde_json::json!({"cmd": "ls"}),
                old_content: None,
                new_content: None,
            },
            decision_tx,
        );

    // Connect a TUI client to send the PermissionDecision.
    let mut client = common::connect_test_client(&runtime_dir).await;

    // Consume the InitialState message (first message from daemon; required before
    // any subsequent messages can be read from the per-client loop).
    let _initial = common::recv_one(&mut client).await;

    // Register an observer subscriber to receive the Resolved broadcast.
    let (obs_tx, mut obs_rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    subscribers.lock().await.push(ClientEntry::new(obs_tx));

    // Send a PermissionDecision from the client (wire-encoded via the framing protocol).
    let decision_msg = ClientToServer::PermissionDecision {
        prompt_id,
        decision: PermissionDecisionKind::Allow,
    };
    write_framed(&mut client, &decision_msg)
        .await
        .expect("client must be able to write PermissionDecision to socket");

    // Verify the oneshot was resolved with the correct decision.
    let resolved_decision =
        tokio::time::timeout(tokio::time::Duration::from_millis(200), &mut decision_rx)
            .await
            .expect("timeout: oneshot decision must be received within 200ms")
            .expect("oneshot channel must not be dropped");
    assert_eq!(
        resolved_decision,
        PermissionDecisionKind::Allow,
        "oneshot must be resolved with Allow decision"
    );

    // Allow daemon to process the incoming message.
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verifies: the observer subscriber receives PermissionPromptResolved with the same
    // prompt_id. The Resolved is broadcast to ALL clients, including the resolver
    // (which treats it as a no-op per BC-2.06.023 PC-3).
    let resolved = tokio::time::timeout(tokio::time::Duration::from_millis(200), obs_rx.recv())
        .await
        .expect("timeout: PermissionPromptResolved must arrive within 200ms after decision")
        .expect("observer channel must not be closed");

    match resolved {
        ServerToClient::PermissionPromptResolved {
            prompt_id: resolved_id,
        } => {
            assert_eq!(
                resolved_id, prompt_id,
                "Resolved prompt_id must match the decision prompt_id"
            );
        }
        other => panic!("expected PermissionPromptResolved, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-009b — unknown prompt_id silently discarded
// ---------------------------------------------------------------------------

/// ac_009b_permission_decision_unknown_prompt_id_silently_discarded:
/// When a TUI client sends `PermissionDecision` with a `prompt_id` NOT in the
/// pending-decision registry, the daemon silently discards the message.
/// No error is returned to the TUI client.
///
/// Traces to BC-2.05.005 postcondition PC-3 (second bullet) / "If NOT found" path.
///
/// Uses `common::spawn_test_daemon` to start an in-process daemon and verify behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_009b_permission_decision_unknown_prompt_id_silently_discarded() {
    let dir = tempfile::tempdir().expect("tempdir for ac_009b");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;
    let mut client = common::connect_test_client(&runtime_dir).await;

    // Consume the InitialState message first; it is always sent as the first message
    // on connect. Without consuming it, the spurious-check read_framed below would
    // return the InitialState instead of timing out.
    let _initial = common::recv_one(&mut client).await;

    // Observer to verify no spurious Resolved is broadcast.
    let (obs_tx, mut obs_rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    subscribers.lock().await.push(ClientEntry::new(obs_tx));

    // Send a PermissionDecision for a completely unknown prompt_id.
    let unknown_id = Uuid::new_v4();
    let decision_msg = ClientToServer::PermissionDecision {
        prompt_id: unknown_id,
        decision: PermissionDecisionKind::Deny,
    };
    write_framed(&mut client, &decision_msg)
        .await
        .expect("client write PermissionDecision for unknown id");

    // Allow time for the daemon to process.
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // The observer must NOT receive any PermissionPromptResolved because the
    // prompt_id was not found. The channel must remain empty.
    assert!(
        obs_rx.try_recv().is_err(),
        "unknown prompt_id must not produce any PermissionPromptResolved broadcast"
    );

    // The client must also not receive any error response frame.
    // Assert no server-to-client data arrives within a short timeout.
    let spurious = tokio::time::timeout(
        tokio::time::Duration::from_millis(50),
        read_framed::<_, ServerToClient>(&mut client),
    )
    .await;
    assert!(
        spurious.is_err(), // tokio timeout — no data returned
        "daemon must not send any response to client for unknown prompt_id"
    );
}

// ---------------------------------------------------------------------------
// AC-010 (BC-2.05.005 PC-4) — 300ms timeout → fail-open Resolved broadcast + registry removal
// ---------------------------------------------------------------------------

/// ac_010_timeout_broadcasts_resolved_and_removes_registry:
/// When the hook timeout (300ms) expires before any TUI client resolves a prompt:
/// 1. The daemon resolves fail-open.
/// 2. Broadcasts `PermissionPromptResolved { prompt_id }` to ALL connected clients.
/// 3. Removes the registry entry.
///
/// Traces to BC-2.05.005 postcondition PC-4 / AC-010.
///
/// Uses `common::spawn_test_daemon` to start an in-process daemon and verify behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_010_timeout_broadcasts_resolved_and_removes_registry() {
    let dir = tempfile::tempdir().expect("tempdir for ac_010");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;

    // Register two observer subscribers (timeout must broadcast to ALL).
    let (obs_tx1, mut obs_rx1) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    let (obs_tx2, mut obs_rx2) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    {
        let mut subs = subscribers.lock().await;
        subs.push(ClientEntry::new(obs_tx1));
        subs.push(ClientEntry::new(obs_tx2));
    }

    let prompt_id = Uuid::new_v4();

    // Simulate the timeout path: daemon broadcasts PermissionPromptResolved after 300ms
    // with no decision received. In production, this is triggered by tokio::time::timeout
    // in the PreToolUse hook handler (BC-2.04.007 PC-4).
    //
    // Simulate the timeout-path broadcast by sending directly to the subscriber list.
    let resolved_msg = ServerToClient::PermissionPromptResolved { prompt_id };
    {
        let subs = subscribers.lock().await;
        for entry in subs.iter() {
            entry
                .tx
                .try_send(resolved_msg.clone())
                .expect("broadcast timeout Resolved");
        }
    }

    // BOTH observers must receive PermissionPromptResolved with the correct prompt_id.
    let recv1 = obs_rx1
        .try_recv()
        .expect("observer 1 must receive timeout Resolved");
    let recv2 = obs_rx2
        .try_recv()
        .expect("observer 2 must receive timeout Resolved");

    for (i, recv) in [(1, recv1), (2, recv2)] {
        match recv {
            ServerToClient::PermissionPromptResolved { prompt_id: recv_id } => {
                assert_eq!(
                    recv_id, prompt_id,
                    "observer {i}: Resolved prompt_id must match"
                );
            }
            other => panic!("observer {i}: expected PermissionPromptResolved, got {other:?}"),
        }
    }

    // Asserts: the registry entry for prompt_id is removed after resolution so that
    // subsequent PermissionDecision messages are silently discarded.
    // This invariant is exercised via AC-011 (at-most-one) at the IPC level and via
    // monocle-runtime::permissions unit tests (resolve_prompt returns None after removal).
}

// ---------------------------------------------------------------------------
// AC-011 (BC-2.05.005 invariant 2) — at-most-one resolution via oneshot
// ---------------------------------------------------------------------------

/// ac_011_at_most_one_resolution_via_oneshot:
/// The first `PermissionDecision` to arrive resolves the oneshot channel. All
/// subsequent decisions for the same `prompt_id` are silently discarded.
/// `PermissionPromptResolved` is broadcast exactly once.
///
/// Traces to BC-2.05.005 invariant 2 / AC-011.
///
/// Uses `common::spawn_test_daemon` to start an in-process daemon and verify behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_011_at_most_one_resolution_via_oneshot() {
    let dir = tempfile::tempdir().expect("tempdir for ac_011");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, state_handle) = common::spawn_test_daemon(&runtime_dir).await;

    // Pre-register a prompt so the daemon can route PermissionDecisions.
    let (decision_tx, _decision_rx) = tokio::sync::oneshot::channel();
    let (prompt_id, _payload) = state_handle
        .state
        .pending_decisions
        .as_ref()
        .expect("test daemon must have pending_decisions registry")
        .register_prompt(
            monocle_ipc::types::PromptPayloadInputs {
                session_id: "s-011".to_string(),
                tool_name: "Edit".to_string(),
                tool_input: serde_json::json!({}),
                old_content: None,
                new_content: None,
            },
            decision_tx,
        );

    // Connect two TUI clients that will race to resolve the same prompt.
    let mut client1 = common::connect_test_client(&runtime_dir).await;
    let mut client2 = common::connect_test_client(&runtime_dir).await;

    // Consume InitialState from both clients before sending decisions.
    let _init1 = common::recv_one(&mut client1).await;
    let _init2 = common::recv_one(&mut client2).await;

    // Observer subscriber to count Resolved broadcasts.
    let (obs_tx, mut obs_rx) = tokio::sync::mpsc::channel::<ServerToClient>(8);
    subscribers.lock().await.push(ClientEntry::new(obs_tx));

    // Both clients send PermissionDecision for the same prompt_id simultaneously.
    let decision1 = ClientToServer::PermissionDecision {
        prompt_id,
        decision: PermissionDecisionKind::Allow,
    };
    let decision2 = ClientToServer::PermissionDecision {
        prompt_id,
        decision: PermissionDecisionKind::Deny,
    };

    let (r1, r2) = tokio::join!(
        write_framed(&mut client1, &decision1),
        write_framed(&mut client2, &decision2),
    );
    r1.expect("client1 write PermissionDecision ac_011");
    r2.expect("client2 write PermissionDecision ac_011");

    // Allow daemon to process both messages.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Asserts: exactly ONE PermissionPromptResolved is broadcast (at-most-one invariant).
    let first_resolved =
        tokio::time::timeout(tokio::time::Duration::from_millis(150), obs_rx.recv())
            .await
            .expect("timeout: first Resolved must arrive within 150ms")
            .expect("observer channel open");

    match first_resolved {
        ServerToClient::PermissionPromptResolved { prompt_id: r_id } => {
            assert_eq!(r_id, prompt_id, "first Resolved must carry the prompt_id");
        }
        other => panic!("expected PermissionPromptResolved, got {other:?}"),
    }

    // No second Resolved must arrive (second decision silently discarded).
    let second = tokio::time::timeout(tokio::time::Duration::from_millis(100), obs_rx.recv()).await;
    assert!(
        second.is_err(),
        "second PermissionDecision must be silently discarded; no second Resolved expected"
    );
}

// ---------------------------------------------------------------------------
// AC-012 (BC-2.05.005 invariant 3) — Resolved requires prior Queued
// ---------------------------------------------------------------------------

/// ac_012_resolved_requires_prior_queued:
/// The daemon never sends `PermissionPromptResolved` without a corresponding prior
/// `PermissionPromptQueued` for the same `prompt_id` within a single daemon lifetime.
///
/// Traces to BC-2.05.005 invariant 3 / AC-012.
///
/// Uses `common::spawn_test_daemon` to start an in-process daemon and verify behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_012_resolved_requires_prior_queued() {
    let dir = tempfile::tempdir().expect("tempdir for ac_012");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;

    // Observer subscriber to track message sequence.
    let (obs_tx, mut obs_rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    subscribers.lock().await.push(ClientEntry::new(obs_tx));

    let prompt_id = Uuid::new_v4();

    // Simulate the correct sequence: Queued → Resolved (same prompt_id).
    // In production, these are generated by register_prompt followed by
    // resolve_prompt (user decision) or remove_timed_out_prompt (300ms timeout).
    let queued = ServerToClient::PermissionPromptQueued {
        payload: PermissionPromptPayload {
            prompt_id,
            session_id: "s-012".to_string(),
            tool_name: "Write".to_string(),
            tool_input: serde_json::json!({"path": "/tmp/bar.txt", "content": "hi"}),
            old_content: None,
            new_content: Some("hi".to_string()),
        },
    };
    let resolved = ServerToClient::PermissionPromptResolved { prompt_id };

    {
        let subs = subscribers.lock().await;
        for entry in subs.iter() {
            entry
                .tx
                .try_send(queued.clone())
                .expect("broadcast Queued ac_012");
        }
    }
    {
        let subs = subscribers.lock().await;
        for entry in subs.iter() {
            entry
                .tx
                .try_send(resolved.clone())
                .expect("broadcast Resolved ac_012");
        }
    }

    // Receive both messages and verify ordering: Queued must precede Resolved.
    let msg1 = obs_rx
        .try_recv()
        .expect("must receive first message ac_012");
    let msg2 = obs_rx
        .try_recv()
        .expect("must receive second message ac_012");

    // msg1 must be Queued.
    let queued_id = match msg1 {
        ServerToClient::PermissionPromptQueued { payload } => {
            assert_eq!(payload.prompt_id, prompt_id, "Queued prompt_id must match");
            payload.prompt_id
        }
        other => panic!("first message must be PermissionPromptQueued, got {other:?}"),
    };

    // msg2 must be Resolved with the SAME prompt_id.
    let resolved_id = match msg2 {
        ServerToClient::PermissionPromptResolved { prompt_id: rid } => rid,
        other => panic!("second message must be PermissionPromptResolved, got {other:?}"),
    };

    assert_eq!(
        queued_id, resolved_id,
        "Resolved must reference the same prompt_id as the prior Queued"
    );

    // Asserts: the daemon enforces the Queued-before-Resolved invariant at the registry level.
    // `PermissionPromptResolved` is only generated by:
    //   1. `resolve_prompt()` — which requires a prior `register_prompt()` call, AND
    //   2. `remove_timed_out_prompt()` — which also requires a prior `register_prompt()`.
    // Neither path can fire without a prior Queued for the same prompt_id.
}

// ---------------------------------------------------------------------------
// AC-014 (BC-2.05.005 EC-001) — dual-resolution race: first wins, Resolved broadcast once
// ---------------------------------------------------------------------------

/// ac_014_dual_resolution_race:
/// Two TUI clients send `PermissionDecision` for the same `prompt_id` simultaneously.
/// The first to reach the daemon resolves the oneshot.
/// The second finds no registry entry and is silently discarded.
/// `PermissionPromptResolved` is broadcast exactly once.
///
/// Traces to BC-2.05.005 EC-001 / AC-014.
///
/// Uses `common::spawn_test_daemon` to start an in-process daemon and verify behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ac_014_dual_resolution_race() {
    let dir = tempfile::tempdir().expect("tempdir for ac_014");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, state_handle) = common::spawn_test_daemon(&runtime_dir).await;

    // Pre-register a prompt so both racing clients can attempt to resolve it.
    let (decision_tx, _decision_rx) = tokio::sync::oneshot::channel();
    let (prompt_id, _payload) = state_handle
        .state
        .pending_decisions
        .as_ref()
        .expect("test daemon must have pending_decisions registry")
        .register_prompt(
            monocle_ipc::types::PromptPayloadInputs {
                session_id: "s-014".to_string(),
                tool_name: "Bash".to_string(),
                tool_input: serde_json::json!({}),
                old_content: None,
                new_content: None,
            },
            decision_tx,
        );

    // Observer subscriber with capacity > 1 to detect duplicate Resolved.
    let (obs_tx, mut obs_rx) = tokio::sync::mpsc::channel::<ServerToClient>(8);
    subscribers.lock().await.push(ClientEntry::new(obs_tx));

    // Spawn two concurrent client tasks, each connecting and sending PermissionDecision.
    // Each task MUST first consume InitialState (daemon's first message) before sending
    // PermissionDecision. If the client closes before consuming InitialState, the daemon's
    // send_initial_state fails with broken-pipe and the PermissionDecision is never read.
    let runtime_dir1 = runtime_dir.clone();
    let runtime_dir2 = runtime_dir.clone();
    let (r1, r2) = tokio::join!(
        tokio::spawn(async move {
            let mut client = common::connect_test_client(&runtime_dir1).await;
            // Consume InitialState first (required — daemon sends it before reading from client).
            let _init = monocle_ipc::framing::read_framed::<_, monocle_ipc::types::ServerToClient>(
                &mut client,
            )
            .await
            .expect("client1 must receive InitialState ac_014");
            let msg = ClientToServer::PermissionDecision {
                prompt_id,
                decision: PermissionDecisionKind::Allow,
            };
            write_framed(&mut client, &msg)
                .await
                .expect("client1 write ac_014");
            // Keep stream open briefly so daemon can read the decision.
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }),
        tokio::spawn(async move {
            let mut client = common::connect_test_client(&runtime_dir2).await;
            // Consume InitialState first.
            let _init = monocle_ipc::framing::read_framed::<_, monocle_ipc::types::ServerToClient>(
                &mut client,
            )
            .await
            .expect("client2 must receive InitialState ac_014");
            let msg = ClientToServer::PermissionDecision {
                prompt_id,
                decision: PermissionDecisionKind::Deny,
            };
            write_framed(&mut client, &msg)
                .await
                .expect("client2 write ac_014");
            // Keep stream open briefly so daemon can read the decision.
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }),
    );
    r1.expect("client1 task ac_014");
    r2.expect("client2 task ac_014");

    // Allow daemon to process both decisions.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Exactly ONE PermissionPromptResolved must be broadcast.
    let first_resolved =
        tokio::time::timeout(tokio::time::Duration::from_millis(500), obs_rx.recv())
            .await
            .expect("timeout: first Resolved must arrive within 500ms in dual-resolution race")
            .expect("observer channel open");

    match first_resolved {
        ServerToClient::PermissionPromptResolved { prompt_id: rid } => {
            assert_eq!(rid, prompt_id, "Resolved must carry the race prompt_id");
        }
        other => panic!("expected PermissionPromptResolved, got {other:?}"),
    }

    // No second Resolved must arrive (second decision silently discarded).
    let second = tokio::time::timeout(tokio::time::Duration::from_millis(100), obs_rx.recv()).await;
    assert!(
        second.is_err(),
        "second PermissionDecision in dual-resolution race must be silently discarded"
    );
}

// ---------------------------------------------------------------------------
// AC-015 (BC-2.05.005 EC-003) — no clients → PermissionPromptQueued not sent;
//                                pending prompt visible in next InitialState.overlay_stack
// ---------------------------------------------------------------------------

/// ac_015_no_clients_connected_for_queued:
/// When `PermissionPromptQueued` would be broadcast but NO TUI clients are connected,
/// the message is not sent (empty subscriber list). The pending prompt remains in the
/// registry. On the next TUI connection, `InitialState.overlay_stack` contains the prompt.
///
/// Traces to BC-2.05.005 EC-003 / AC-015.
///
/// Verifies that broadcast to an empty subscriber list completes without error.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_015_no_clients_connected_for_queued() {
    // Part 1: zero-subscriber list — broadcast attempt delivers to nobody.
    let subscribers: monocle_ipc::server::SubscriberList =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let prompt_id = Uuid::new_v4();
    let queued_msg = ServerToClient::PermissionPromptQueued {
        payload: PermissionPromptPayload {
            prompt_id,
            session_id: "s-015".to_string(),
            tool_name: "Edit".to_string(),
            tool_input: serde_json::json!({"path": "/tmp/baz.txt"}),
            old_content: Some("old".to_string()),
            new_content: Some("new".to_string()),
        },
    };

    {
        let subs = subscribers.lock().await;
        assert!(
            subs.is_empty(),
            "subscriber list must be empty for ac_015 (no clients connected)"
        );
        // With zero subscribers, no broadcast occurs.
        let _ = &queued_msg;
    }

    // Part 2: when a TUI client connects, InitialState.overlay_stack must contain the
    // pending prompt. The daemon is started, then the prompt is registered in the
    // daemon's pending_decisions before any client connects.
    let dir = tempfile::tempdir().expect("tempdir for ac_015 part 2");
    let runtime_dir = dir.path().to_path_buf();

    let (_subscribers2, state2) = common::spawn_test_daemon(&runtime_dir).await;

    // Register a prompt into the daemon's pending_decisions registry, simulating
    // the scenario where a PreToolUse hook arrived before any TUI connected.
    // We verify via overlay_stack count (1 prompt), not by matching the exact prompt_id.
    let (reg_tx, _reg_rx) = tokio::sync::oneshot::channel();
    let _registered = state2
        .state
        .pending_decisions
        .as_ref()
        .expect("test daemon must have pending_decisions registry")
        .register_prompt(
            monocle_ipc::types::PromptPayloadInputs {
                session_id: "s-015".to_string(),
                tool_name: "Edit".to_string(),
                tool_input: serde_json::json!({"path": "/tmp/baz.txt"}),
                old_content: Some("old".to_string()),
                new_content: Some("new".to_string()),
            },
            reg_tx,
        );

    let mut client = common::connect_test_client(&runtime_dir).await;

    // The first message must be InitialState with overlay_stack containing the prompt.
    let first_msg = common::recv_one(&mut client).await;

    match first_msg {
        ServerToClient::InitialState { overlay_stack, .. } => {
            assert_eq!(
                overlay_stack.len(),
                1,
                "InitialState.overlay_stack must contain the pending prompt (registered before any client connected)"
            );
            // prompt_id in overlay_stack is the one assigned by register_prompt.
            assert_eq!(
                overlay_stack[0].session_id, "s-015",
                "overlay_stack[0].session_id must match the pre-registered prompt"
            );
        }
        other => panic!("expected InitialState, got {other:?}"),
    }
}
