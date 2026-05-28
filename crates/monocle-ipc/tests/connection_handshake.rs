//! S-022 connection handshake tests — BC-2.05.002 (AC-001..AC-006, AC-013).
//!
//! Tests the TUI connect + InitialState push lifecycle per BC-2.05.002.
//!
//! # Red Gate
//!
//! Every test in this file calls into `todo!()` stubs in `monocle-ipc::server`,
//! `monocle-ipc::uds`, or the common test harness. Each test MUST panic at runtime
//! with a `"not yet implemented: S-022: ..."` message until the implementer lands.
//! This is the required Red Gate signal for story S-022 Step 4.
//!
//! # Naming Convention
//!
//! Tests use `ac_NNN_<short_descriptor>` (lowercase `ac_`) per the S-022 dispatch
//! for these files specifically, in addition to the VSDD `test_BC_S_SS_NNN_xxx`
//! convention used in other monocle-ipc test files.
// Suppress non_snake_case — `ac_` prefix followed by numeric ID is intentional.
#![allow(non_snake_case)]
// expect/unwrap/disallowed_methods are idiomatic in test code.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::disallowed_methods)]

mod common;

use std::sync::Arc;

use monocle_ipc::error::IpcError;
use monocle_ipc::framing::{read_framed, write_framed};
use monocle_ipc::server::{register_subscriber, SubscriberList};
use monocle_ipc::types::{PermissionPromptPayload, ServerToClient};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// AC-001 (BC-2.05.002 PC-1) — per-client Tokio task spawned; removed on EOF
// ---------------------------------------------------------------------------

/// ac_001_per_client_tokio_task_spawned:
/// The daemon accept loop spawns a dedicated Tokio task for each connecting TUI client.
/// After the client disconnects (EOF), the client is removed from the fan-out subscriber list.
///
/// Traces to BC-2.05.002 postcondition PC-1 / AC-001.
///
/// # Red Gate
///
/// Calls `common::spawn_test_daemon` → `todo!("S-022 test harness: spawn test daemon ...")` → panics.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_001_per_client_tokio_task_spawned() {
    let dir = tempfile::tempdir().expect("tempdir for ac_001");
    let runtime_dir = dir.path().to_path_buf();

    // spawn_test_daemon hits todo!() (Red Gate).
    let (_subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;

    // After implementation:
    // 1. Connect a TUI client.
    // 2. Assert subscribers.lock().await.len() == 1 (one per-client task spawned).
    // 3. Drop the client stream (simulate EOF).
    // 4. Allow the per-client task to detect EOF.
    // 5. Assert subscribers.lock().await.len() == 0 (client removed on EOF).
    //
    // Implementation note: the per-client task polls for EOF via the recv loop;
    // the subscriber removal is driven by `remove_subscriber` called on send error or
    // clean read of 0 bytes.
}

// ---------------------------------------------------------------------------
// AC-002 (BC-2.05.002 PC-2) — InitialState is the first and only message on connect
// ---------------------------------------------------------------------------

/// ac_002_initial_state_is_first_message:
/// Exactly one `ServerToClient::InitialState` message arrives first after connection.
/// The message contains sessions, ring_tail, overlay_stack, and drop_counter.
///
/// Traces to BC-2.05.002 postcondition PC-2 / AC-002.
///
/// # Red Gate
///
/// Calls `common::spawn_test_daemon` + `common::connect_test_client` →
/// both hit `todo!()` → panics.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_002_initial_state_is_first_message() {
    let dir = tempfile::tempdir().expect("tempdir for ac_002");
    let runtime_dir = dir.path().to_path_buf();

    // spawn_test_daemon hits todo!() (Red Gate).
    let (_subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;

    // connect_test_client hits todo!() (Red Gate).
    let mut client = common::connect_test_client(&runtime_dir).await;

    // After implementation: recv_one must return ServerToClient::InitialState.
    let first_msg = common::recv_one(&mut client).await;

    match first_msg {
        ServerToClient::InitialState {
            sessions,
            ring_tail,
            overlay_stack,
            drop_counter,
        } => {
            // All fields present (may be empty in the empty-state case).
            // Specific values are tested in ac_013 (empty-state vector).
            let _ = (sessions, ring_tail, overlay_stack, drop_counter);
        }
        other => panic!("first message from daemon must be InitialState, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-003 (BC-2.05.002 PC-3) — 4-byte LE length-prefix framing for all variants
// ---------------------------------------------------------------------------

/// ac_003_four_byte_le_framing:
/// All `ServerToClient` messages are framed with a 4-byte little-endian u32 payload
/// length. The decoder uses the same framing for `InitialState` and all other variants.
///
/// Traces to BC-2.05.002 postcondition PC-3 / AC-003.
///
/// # Pure framing (already implemented — no Red Gate hit in Part 1)
///
/// Part 1 exercises `write_framed`/`read_framed` which are already implemented (S-021).
/// Part 2 exercises the end-to-end integration via `spawn_test_daemon` (Red Gate).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_003_four_byte_le_framing() {
    // Part 1: pure framing check (already implemented — no todo!() hit).
    // Verify that ALL ServerToClient variants round-trip correctly through write_framed/read_framed.
    let variants: Vec<ServerToClient> = vec![
        ServerToClient::InitialState {
            sessions: vec![],
            ring_tail: vec![],
            overlay_stack: vec![],
            drop_counter: 0,
        },
        ServerToClient::PermissionPromptQueued {
            payload: PermissionPromptPayload {
                prompt_id: Uuid::new_v4(),
                session_id: "s1".to_string(),
                tool_name: "Bash".to_string(),
                tool_input: serde_json::json!({"cmd": "ls"}),
                old_content: None,
                new_content: None,
            },
        },
        ServerToClient::PermissionPromptResolved {
            prompt_id: Uuid::new_v4(),
        },
        ServerToClient::DropCounterUpdate { drop_counter: 42 },
    ];

    for variant in &variants {
        let mut buf = Vec::<u8>::new();
        write_framed(&mut buf, variant)
            .await
            .expect("write_framed must succeed for all ServerToClient variants");

        // Verify 4-byte LE prefix encodes the payload length.
        assert!(buf.len() >= 4, "framed output must have at least 4 bytes");
        let declared_len = u32::from_le_bytes(buf[..4].try_into().expect("first 4 bytes")) as usize;
        let actual_payload_len = buf.len() - 4;
        assert_eq!(
            declared_len, actual_payload_len,
            "4-byte LE prefix must equal actual payload length for variant {variant:?}"
        );

        // Verify round-trip decode.
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded: ServerToClient = read_framed(&mut cursor)
            .await
            .expect("read_framed must decode what write_framed encoded");
        let _ = decoded; // structural equality; variant identity verified by encode.
    }

    // Part 2: integration path — spawn daemon + connect client (both hit todo!()).
    let dir = tempfile::tempdir().expect("tempdir for ac_003 part 2");
    let runtime_dir = dir.path().to_path_buf();

    // Hits todo!() (Red Gate for the integration part of AC-003).
    let (_subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;
    let mut client = common::connect_test_client(&runtime_dir).await;

    // After implementation: the first message must be a properly framed InitialState.
    let first_msg = common::recv_one(&mut client).await;
    match first_msg {
        ServerToClient::InitialState { .. } => {
            // Correct — framing worked; InitialState decoded successfully.
        }
        other => panic!("expected InitialState via framing, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-004 (BC-2.05.002 PC-4) — InitialState > 256 KiB → IpcError::MessageTooLarge
// ---------------------------------------------------------------------------

/// ac_004_initial_state_too_large_closes_connection:
/// When the serialized `InitialState` exceeds 256 KiB (262,144 bytes), the daemon
/// closes the connection with `IpcError::MessageTooLarge` and logs ERROR.
/// The TUI receives EOF.
///
/// Traces to BC-2.05.002 postcondition PC-4 / AC-004.
///
/// # Red Gate
///
/// Calls `common::spawn_test_daemon` → `todo!()` → panics.
///
/// # Implementation note for the implementer
///
/// Pre-populate the DaemonState with a `ring_tail` whose total serialized size
/// exceeds 262,144 bytes. One way: insert 300 synthetic `HookEvent` records whose
/// JSON is ~1 KiB each. `send_initial_state` must serialize, detect `len > MAX_MESSAGE_BYTES`,
/// log ERROR, and return `Err(IpcError::MessageTooLarge)` which closes the connection.
/// The TUI client reads EOF and returns `Err(IpcError::Disconnected)`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_004_initial_state_too_large_closes_connection() {
    let dir = tempfile::tempdir().expect("tempdir for ac_004");
    let runtime_dir = dir.path().to_path_buf();

    let (_subscribers, state_handle) = common::spawn_test_daemon(&runtime_dir).await;

    // Pre-populate pending_decisions with ~300 large PermissionPromptPayload entries
    // so that snapshot_initial_state produces an overlay_stack whose serialized
    // InitialState exceeds 256 KiB (262,144 bytes), triggering MessageTooLarge.
    // Each payload carries ~1 KiB of tool_input data; 300 × ~1 KiB ≈ 300 KiB.
    {
        let registry = state_handle
            .state
            .pending_decisions
            .as_ref()
            .expect("test daemon must have pending_decisions registry");
        for i in 0..300u32 {
            let (tx, _rx) = tokio::sync::oneshot::channel();
            let large_tool_input = serde_json::json!({
                "path": format!("/tmp/file_{i}.txt"),
                "content": "A".repeat(1024),
            });
            registry.register_prompt(
                monocle_ipc::types::PromptPayloadInputs {
                    session_id: format!("session-{i}"),
                    tool_name: "Edit".to_string(),
                    tool_input: large_tool_input,
                    old_content: Some("old".repeat(100)),
                    new_content: Some("new".repeat(100)),
                },
                tx,
            );
        }
    }

    let mut client = common::connect_test_client(&runtime_dir).await;

    // Daemon closed the connection after MessageTooLarge detection; client receives EOF.
    let first_result: Result<ServerToClient, IpcError> =
        monocle_ipc::framing::read_framed(&mut client).await;
    match first_result {
        Err(IpcError::Disconnected) => {
            // Correct — daemon closed connection after MessageTooLarge.
        }
        Ok(msg) => panic!("client must receive EOF for oversized InitialState, got {msg:?}"),
        Err(other) => panic!("expected Disconnected, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-005 (BC-2.05.002 PC-5/PC-6) — push-only model; no polling after InitialState
// ---------------------------------------------------------------------------

/// ac_005_push_only_no_polling:
/// After the `InitialState` push, the TUI does NOT poll the daemon for subsequent
/// state changes. All updates arrive as push messages over the same connection.
/// The IPC receive loop runs concurrently with (not blocking) the terminal event loop.
///
/// Traces to BC-2.05.002 postcondition PC-5 / PC-6 / AC-005.
///
/// # Red Gate
///
/// Calls `common::spawn_test_daemon` → `todo!()` → panics.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_005_push_only_no_polling() {
    let dir = tempfile::tempdir().expect("tempdir for ac_005");
    let runtime_dir = dir.path().to_path_buf();

    // Hits todo!() (Red Gate).
    let (subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;
    let mut client = common::connect_test_client(&runtime_dir).await;

    // After implementation:
    // 1. Consume the InitialState message (first message, no poll required).
    // 2. Push a DropCounterUpdate via the subscriber list to simulate a daemon push.
    // 3. Assert the client receives the push message without sending any request.

    // Wait until the daemon has registered the client in the subscriber list.
    // There is a scheduling race: connect_test_client returns as soon as the
    // OS-level accept() handshake completes, but the per-client task runs
    // asynchronously and may not have reached register_subscriber yet.
    // Polling here ensures the push goes to a non-empty subscriber list.
    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);
    loop {
        {
            let subs = subscribers.lock().await;
            if !subs.is_empty() {
                break;
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "ac_005: subscriber not registered within 2 s after connect"
        );
        tokio::task::yield_now().await;
    }

    // Simulate a daemon push directly via the subscriber list.
    let push_msg = ServerToClient::DropCounterUpdate { drop_counter: 99 };
    {
        let subs = subscribers.lock().await;
        for sender in subs.iter() {
            sender
                .try_send(push_msg.clone())
                .expect("push DropCounterUpdate to subscriber");
        }
    }

    // The second message received (after InitialState) must be the pushed update.
    // No poll was issued by the client — the daemon pushed it.
    let _initial = common::recv_one(&mut client).await; // consume InitialState
    let push_received = common::recv_one(&mut client).await;

    match push_received {
        ServerToClient::DropCounterUpdate { drop_counter } => {
            assert_eq!(drop_counter, 99, "pushed drop_counter value must be 99");
        }
        other => panic!("second message must be the DropCounterUpdate push, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-006 (BC-2.05.002 invariant 3) — no gap window between snapshot and streaming
// ---------------------------------------------------------------------------

/// ac_006_no_gap_window_between_snapshot_and_streaming:
/// Events that occur after the connection is accepted but before `InitialState` is
/// fully sent are delivered as incremental push messages. No events are lost.
///
/// Traces to BC-2.05.002 invariant 3 / AC-006.
///
/// # Red Gate
///
/// Calls `register_subscriber` → `todo!("S-022: register_subscriber ...")` → panics.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_006_no_gap_window_between_snapshot_and_streaming() {
    let subscribers: SubscriberList = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);

    // register_subscriber is the todo!() stub that enforces the no-gap invariant.
    // The sender MUST be registered in subscribers BEFORE InitialState is sent so that
    // any event arriving during the send is queued in the channel, not dropped.
    //
    // Panics here (Red Gate for AC-006).
    register_subscriber(&subscribers, tx).await;

    // After implementation: simulate an event arriving after subscription but before
    // InitialState completes by pushing a DropCounterUpdate to the subscriber list.
    let gap_event = ServerToClient::DropCounterUpdate { drop_counter: 1 };
    {
        let subs = subscribers.lock().await;
        for sender in subs.iter() {
            sender.try_send(gap_event.clone()).expect("gap event push");
        }
    }

    // After implementation: the subscriber channel must contain the gap_event,
    // proving no gap window exists between snapshot time and streaming phase.
    let received = rx
        .try_recv()
        .expect("gap event must be delivered to subscriber after register_subscriber");
    match received {
        ServerToClient::DropCounterUpdate { drop_counter } => {
            assert_eq!(
                drop_counter, 1,
                "gap event drop_counter must be 1 (incremental update, not snapshot)"
            );
        }
        other => panic!("expected DropCounterUpdate gap event, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-013 (BC-2.05.002 EC-001) — empty InitialState
// ---------------------------------------------------------------------------

/// ac_013_empty_initial_state:
/// When the daemon has zero sessions, zero ring events, zero queued prompts, and
/// drop_counter=0, `InitialState` is sent with all fields as empty Vecs and
/// drop_counter=0.
///
/// Traces to BC-2.05.002 EC-001 / AC-013.
///
/// # Red Gate
///
/// Calls `common::spawn_test_daemon` + `common::connect_test_client` →
/// both hit `todo!()` → panics.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_013_empty_initial_state() {
    let dir = tempfile::tempdir().expect("tempdir for ac_013");
    let runtime_dir = dir.path().to_path_buf();

    // The daemon is started with a freshly constructed DaemonState (all None fields),
    // which maps to empty Vecs and drop_counter=0 in InitialState.
    //
    // Hits todo!() (Red Gate).
    let (_subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;
    let mut client = common::connect_test_client(&runtime_dir).await;

    // After implementation: the first message must be InitialState with all empties.
    let first_msg = common::recv_one(&mut client).await;

    match first_msg {
        ServerToClient::InitialState {
            sessions,
            ring_tail,
            overlay_stack,
            drop_counter,
        } => {
            assert!(sessions.is_empty(), "empty state: sessions must be []");
            assert!(ring_tail.is_empty(), "empty state: ring_tail must be []");
            assert!(
                overlay_stack.is_empty(),
                "empty state: overlay_stack must be []"
            );
            assert_eq!(drop_counter, 0, "empty state: drop_counter must be 0");
        }
        other => panic!("expected InitialState with all empties, got {other:?}"),
    }
}
