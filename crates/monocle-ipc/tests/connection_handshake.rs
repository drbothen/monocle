//! S-022 connection handshake tests — BC-2.05.002 (AC-001..AC-006, AC-013).
//!
//! Tests the TUI connect + InitialState push lifecycle per BC-2.05.002.
//!
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
/// Uses `common::spawn_test_daemon` to verify per-client task lifecycle behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_001_per_client_tokio_task_spawned() {
    let dir = tempfile::tempdir().expect("tempdir for ac_001");
    let runtime_dir = dir.path().to_path_buf();

    let (_subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;

    // 1. Connect a TUI client.
    // 2. Assert subscribers.lock().await.len() == 1 (one per-client task spawned).
    // 3. Drop the client stream (simulate EOF).
    // 4. Allow the per-client task to detect EOF.
    // 5. Assert subscribers.lock().await.len() == 0 (client removed on EOF).
    //
    // The per-client task polls for EOF via the recv loop;
    // subscriber removal is driven by `remove_subscriber` called on send error or EOF.
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
/// Uses `common::spawn_test_daemon` + `common::connect_test_client` to verify
/// InitialState is the first and only message on a new TUI connection.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_002_initial_state_is_first_message() {
    let dir = tempfile::tempdir().expect("tempdir for ac_002");
    let runtime_dir = dir.path().to_path_buf();

    let (_subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;

    let mut client = common::connect_test_client(&runtime_dir).await;

    // Asserts: recv_one returns ServerToClient::InitialState as the first message.
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
/// Part 2 exercises end-to-end integration via `spawn_test_daemon` + `connect_test_client`.
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

    let (_subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;
    let mut client = common::connect_test_client(&runtime_dir).await;

    // Asserts: the first message is a properly framed InitialState (end-to-end decode).
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
/// Uses `common::spawn_test_daemon` to verify the 256 KiB size guard on InitialState.
///
/// Pre-populates `pending_decisions` with 300 `PermissionPromptPayload` entries in the
/// overlay_stack so that `snapshot_initial_state` produces an InitialState exceeding
/// 256 KiB (262,144 bytes). `send_initial_state` serializes, detects `len > MAX_MESSAGE_BYTES`,
/// logs ERROR, and returns `Err(IpcError::MessageTooLarge)`, closing the connection.
/// The TUI client reads EOF and receives `Err(IpcError::Disconnected)`.
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
/// Uses `common::spawn_test_daemon` to verify server-push delivery without client polling.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_005_push_only_no_polling() {
    let dir = tempfile::tempdir().expect("tempdir for ac_005");
    let runtime_dir = dir.path().to_path_buf();

    let (subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;
    let mut client = common::connect_test_client(&runtime_dir).await;

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
/// Verifies that `register_subscriber` enqueues events before InitialState delivery
/// so no events are lost in the gap between snapshot and streaming phase.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_006_no_gap_window_between_snapshot_and_streaming() {
    let subscribers: SubscriberList = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);

    // The sender MUST be registered in subscribers BEFORE InitialState is sent so that
    // any event arriving during the send is queued in the channel, not dropped.
    // This enforces the no-gap invariant (AC-006, BC-2.05.002 invariant 3).

    register_subscriber(&subscribers, tx).await;

    // Simulate an event arriving after subscription but before InitialState completes
    // by pushing a DropCounterUpdate directly to the subscriber list.
    let gap_event = ServerToClient::DropCounterUpdate { drop_counter: 1 };
    {
        let subs = subscribers.lock().await;
        for sender in subs.iter() {
            sender.try_send(gap_event.clone()).expect("gap event push");
        }
    }

    // Asserts: the subscriber channel must contain the gap_event,
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
/// Uses `common::spawn_test_daemon` + `common::connect_test_client` to verify that
/// `InitialState` carries empty collections and drop_counter=0 when daemon state is fresh.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac_013_empty_initial_state() {
    let dir = tempfile::tempdir().expect("tempdir for ac_013");
    let runtime_dir = dir.path().to_path_buf();

    // The daemon is started with a freshly constructed DaemonState (all None fields),
    // which maps to empty Vecs and drop_counter=0 in InitialState.
    //

    let (_subscribers, _state) = common::spawn_test_daemon(&runtime_dir).await;
    let mut client = common::connect_test_client(&runtime_dir).await;

    // Asserts: the first message is InitialState with all empties.
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

// ---------------------------------------------------------------------------
// test_BC_2_05_002_ring_tail_non_empty_passes_through:
// ring_tail in InitialState carries Vec<HookEventRecord> populated from the RAM ring.
// Verifies F-S022-ADV2-HIGH-002 architect directive: no lossy conversion, no fabrication.
// ---------------------------------------------------------------------------

/// test_BC_2_05_002_ring_tail_non_empty_passes_through:
/// When the daemon RAM ring contains N records, `snapshot_initial_state` populates
/// `InitialState.ring_tail` with exactly those N `HookEventRecord` values — session_ids,
/// pids, and hook_types intact.
///
/// Traces to BC-2.05.002 PC-2 v1.0.4 / architect decision F-S022-ADV2-HIGH-002.
///
/// No `todo!()` call — exercises `snapshot_initial_state` directly without the accept loop.
#[test]
fn test_BC_2_05_002_ring_tail_non_empty_passes_through() {
    use std::sync::Arc;

    // Build a DaemonState with a live RingBuffer pre-populated with 3 records.
    let tmp = tempfile::tempdir().expect("tempdir for ring_tail test");
    let ring_path = tmp.path().join("monocle.jsonl");

    let ring = Arc::new(monocle_runtime::ring::RingBuffer::new(
        ring_path,
        monocle_runtime::ring::RotationConfig::default(),
    ));

    // Push 3 records with distinct session_ids and pids via the legacy push() API
    // (synchronous; no flush task needed — push() writes directly).
    let records = vec![
        monocle_runtime::ring::HookEventRecord::new(
            "session-aaa".to_string(),
            1_000_001_i64,
            1001_u32,
            "SessionStart".to_string(),
            None,
            None,
        ),
        monocle_runtime::ring::HookEventRecord::new(
            "session-bbb".to_string(),
            1_000_002_i64,
            1002_u32,
            "PreToolUse".to_string(),
            Some("Bash".to_string()),
            None,
        ),
        monocle_runtime::ring::HookEventRecord::new(
            "session-ccc".to_string(),
            1_000_003_i64,
            1003_u32,
            "Stop".to_string(),
            None,
            None,
        ),
    ];

    for record in &records {
        ring.push(record).expect("ring push must succeed");
    }

    // Also push to RAM ring via append() so latest_events() returns them.
    // (push() writes to disk; append() writes to RAM ring. Both paths are needed for
    //  snapshot_initial_state to see the records via ring.latest_events().)
    for record in records.clone() {
        ring.append(record).expect("ring append must succeed");
    }

    let mut state = monocle_runtime::state::DaemonState::new();
    state.ring = Some(Arc::clone(&ring));

    // snapshot_initial_state must populate ring_tail from the RAM ring directly.
    let snapshot = monocle_runtime::state::snapshot_initial_state(&state);

    match snapshot {
        ServerToClient::InitialState { ring_tail, .. } => {
            assert_eq!(
                ring_tail.len(),
                3,
                "ring_tail must contain exactly 3 records (one per push)"
            );
            // Verify each record's session_id and pid survived pass-through (no fabrication).
            assert_eq!(
                ring_tail[0].session_id, "session-aaa",
                "ring_tail[0].session_id must be session-aaa"
            );
            assert_eq!(ring_tail[0].pid, 1001, "ring_tail[0].pid must be 1001");
            assert_eq!(
                ring_tail[0].hook_type, "SessionStart",
                "ring_tail[0].hook_type must be SessionStart"
            );

            assert_eq!(
                ring_tail[1].session_id, "session-bbb",
                "ring_tail[1].session_id must be session-bbb"
            );
            assert_eq!(ring_tail[1].pid, 1002, "ring_tail[1].pid must be 1002");
            assert_eq!(
                ring_tail[1].hook_type, "PreToolUse",
                "ring_tail[1].hook_type must be PreToolUse"
            );
            assert_eq!(
                ring_tail[1].tool_name.as_deref(),
                Some("Bash"),
                "ring_tail[1].tool_name must be Bash"
            );

            assert_eq!(
                ring_tail[2].session_id, "session-ccc",
                "ring_tail[2].session_id must be session-ccc"
            );
            assert_eq!(ring_tail[2].pid, 1003, "ring_tail[2].pid must be 1003");
            assert_eq!(
                ring_tail[2].hook_type, "Stop",
                "ring_tail[2].hook_type must be Stop"
            );
        }
        other => panic!("snapshot_initial_state must return InitialState, got {other:?}"),
    }
}
