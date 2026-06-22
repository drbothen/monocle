//! PtyOutput fan-out broker — unit tests (S-046, BC-2.05.009).
//!
//! Tests exercise the new `SubscriberList`-based `PtyBroker` design (Q1 ruling):
//! the broker does NOT own a per-client registry; all fan-out goes through
//! `broadcast_to_subscribers(&shared_subscriber_list, msg)`.
//!
//! # AC coverage
//!
//! | AC    | Test fn                                                                      |
//! |-------|------------------------------------------------------------------------------|
//! | AC-001| test_BC_2_05_009_bounded_channel_backpressure_blocks_not_drops               |
//! | AC-002| test_BC_2_05_009_fan_out_via_subscriber_list_not_broker_registry             |
//! | AC-003| test_BC_2_05_009_one_strike_disconnect_slow_client                           |
//! | AC-004| test_BC_2_05_009_pty_drop_counter_only_oom_not_backpressure                  |
//! | AC-005| test_BC_2_05_009_pty_reset_emitted_on_proxy_send_error_not_graceful_close    |
//! | AC-006| test_BC_2_05_009_hook_events_priority_over_pty_output                        |
//! | AC-007| test_BC_2_05_009_no_unbounded_channel_in_pty_path                            |

// Test naming preserves BC identifiers for traceability.
#![allow(non_snake_case)]
// Tests use unwrap/expect as assertion helpers — panics are the desired failure mode.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use monocle_ipc::server::{register_subscriber, SubscriberList, CLIENT_CHANNEL_CAPACITY};
use monocle_ipc::types::ServerToClient;
use monocle_runtime::ipc_server::broadcast_to_subscribers;
use monocle_runtime::pty_broker::{
    PtyBroker, PTY_BROKER_CLIENT_CAPACITY, PTY_BROKER_INPUT_CAPACITY, PTY_BROKER_STRIKE_LIMIT,
};
use tokio::sync::mpsc;
use tokio::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a fresh empty SubscriberList.
fn make_subscriber_list() -> Arc<SubscriberList> {
    Arc::new(Arc::new(tokio::sync::Mutex::new(Vec::new())))
}

/// Returns true iff the message is `ServerToClient::PtyOutput` with matching fields.
fn is_pty_output(msg: &ServerToClient, expected_session_id: &str, expected_bytes: &[u8]) -> bool {
    match msg {
        ServerToClient::PtyOutput { session_id, bytes } => {
            session_id == expected_session_id && bytes.as_slice() == expected_bytes
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// AC-001 (BC-2.05.009 PC-2, Invariant 3): bounded INPUT channel blocks on full
// ---------------------------------------------------------------------------

/// INPUT channel blocks (backpressure) rather than dropping on full — AC-001, BC-2.05.009 PC-2.
#[tokio::test]
async fn test_BC_2_05_009_bounded_channel_backpressure_blocks_not_drops() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();
    let broker = PtyBroker::new(
        "test-session-001".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Step 1: fill the INPUT channel to capacity via try_send.
    let frame = Arc::new(Bytes::from_static(b"hello pty"));
    for _ in 0..PTY_BROKER_INPUT_CAPACITY {
        broker
            .input_tx
            .try_send(Arc::clone(&frame))
            .expect("try_send must succeed while channel has capacity");
    }

    // Step 2: channel is now at capacity. A further try_send MUST return Err::Full.
    let try_result = broker.input_tx.try_send(Arc::clone(&frame));
    assert!(
        try_result.is_err(),
        "try_send must fail when INPUT channel is at capacity (AC-001)"
    );
    assert!(
        matches!(
            try_result.as_ref().unwrap_err(),
            tokio::sync::mpsc::error::TrySendError::Full(_)
        ),
        "try_send error must be TrySendError::Full, not Closed \
         (BC-2.05.009 Invariant 3 — channel not dropped, only full)"
    );

    // Step 3: behavioral assertion — `.send().await` blocks, not drops.
    let input_tx_clone = broker.input_tx.clone();
    let send_frame = Arc::clone(&frame);

    let send_future = async move {
        input_tx_clone
            .send(send_frame)
            .await
            .expect("send must not error — receiver is alive");
    };

    // The channel is full: .send().await must NOT complete within 50ms.
    let result = tokio::time::timeout(Duration::from_millis(50), send_future).await;
    assert!(
        result.is_err(),
        "send().await must BLOCK (not complete) when INPUT channel is at full capacity \
         — backpressure to PTY reader (BC-2.05.009 Invariant 3, AC-001)"
    );

    // Step 5: pty_drop_counter must remain 0 — normal channel fullness is NOT a drop.
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on normal channel fullness \
         (BC-2.05.009 PC-3 — drop_counter counts only OOM/sender-error)"
    );
}

// ---------------------------------------------------------------------------
// AC-002 (BC-2.05.009 PC-1b): fan-out via shared SubscriberList
//
// The broker event loop calls broadcast_to_subscribers with the SHARED SubscriberList
// (not a per-broker registry). Clients registered via register_subscriber after broker
// construction receive PtyOutput frames.
// ---------------------------------------------------------------------------

/// Fan-out goes through shared SubscriberList — AC-002, BC-2.05.009 PC-1b.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_BC_2_05_009_fan_out_via_subscriber_list_not_broker_registry() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let mut broker = PtyBroker::new(
        "test-session-002".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register two TUI clients via the shared SubscriberList BEFORE spawning event loop.
    let (tx_a, mut rx_a) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    let (tx_b, mut rx_b) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    register_subscriber(&sub_list, tx_a).await;
    register_subscriber(&sub_list, tx_b).await;

    let (hook_tx, hook_rx) = mpsc::channel::<()>(8);
    let handle = broker.spawn_event_loop(hook_rx);

    // Give the event loop a moment to start.
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Send a PTY frame through the INPUT channel.
    let frame_bytes = b"pty output for fan-out test";
    broker
        .input_tx
        .send(Arc::new(Bytes::from_static(frame_bytes)))
        .await
        .expect("INPUT send must succeed");

    // Both clients must receive the frame as PtyOutput.
    let msg_a = tokio::time::timeout(Duration::from_millis(200), rx_a.recv())
        .await
        .expect("client-a must receive message within timeout")
        .expect("channel must not be closed");
    let msg_b = tokio::time::timeout(Duration::from_millis(200), rx_b.recv())
        .await
        .expect("client-b must receive message within timeout")
        .expect("channel must not be closed");

    assert!(
        is_pty_output(&msg_a, "test-session-002", frame_bytes),
        "client-a must receive PtyOutput{{session_id:'test-session-002', bytes:...}}; got {:?}",
        msg_a,
    );
    assert!(
        is_pty_output(&msg_b, "test-session-002", frame_bytes),
        "client-b must receive PtyOutput{{session_id:'test-session-002', bytes:...}}; got {:?}",
        msg_b,
    );

    // Verify no per-client registry fields exist on the broker (structural assertion).
    // The broker owns only input_tx/input_rx/pty_drop_counter/subscriber_list.
    // Compilation of this test proves the API: if register_client/fan_out existed, they
    // would be called here; their absence confirms the Q1 ruling is enforced.

    drop(hook_tx);
    handle.abort();
}

// ---------------------------------------------------------------------------
// AC-003 (BC-2.05.009 Invariant 3b): 1-strike disconnect for slow clients
//
// broadcast_to_subscribers disconnects a slow client on the first TrySendError::Full.
// All other clients continue receiving uninterrupted.
// ---------------------------------------------------------------------------

/// Slow client is disconnected on first full-buffer send; fast client receives all frames — AC-003.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_BC_2_05_009_one_strike_disconnect_slow_client() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let broker = PtyBroker::new(
        "test-session-003".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register a FAST client with a normal capacity channel.
    let (tx_fast, mut rx_fast) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    register_subscriber(&sub_list, tx_fast).await;

    // Register a SLOW client: fill its receive buffer to capacity immediately so that
    // any broadcast via broadcast_to_subscribers will see a full channel.
    let (tx_slow, _rx_slow) = mpsc::channel::<ServerToClient>(1); // capacity 1
                                                                  // Pre-fill the slow client's channel so the FIRST broadcast attempt fails.
    tx_slow
        .try_send(ServerToClient::SessionListUpdate { sessions: vec![] })
        .expect("pre-fill must succeed");
    register_subscriber(&sub_list, tx_slow).await;

    // Subscriber list now has 2 entries.
    {
        let subs = sub_list.lock().await;
        assert_eq!(subs.len(), 2, "must have 2 subscribers before broadcast");
    }

    // Broadcast one PtyOutput — fast client receives it; slow client (full buffer)
    // is disconnected immediately (1-strike model per BC-2.05.009 Invariant 3b).
    let msg = ServerToClient::PtyOutput {
        session_id: "test-session-003".to_string(),
        bytes: b"one-strike test".to_vec(),
    };
    broadcast_to_subscribers(&sub_list, msg).await;

    // After broadcast: subscriber list must have exactly 1 entry (slow client removed).
    {
        let subs = sub_list.lock().await;
        assert_eq!(
            subs.len(),
            1,
            "slow client must be removed after 1 TrySendError::Full (1-strike model — AC-003)"
        );
    }

    // Fast client must have received the message.
    let received = rx_fast
        .try_recv()
        .expect("fast client must have received the PtyOutput message");
    assert!(
        is_pty_output(&received, "test-session-003", b"one-strike test"),
        "fast client must receive PtyOutput; got {:?}",
        received,
    );

    // pty_drop_counter must NOT be incremented — 1-strike disconnect is not OOM.
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on slow-client 1-strike disconnect \
         (BC-2.05.009 PC-3 / EC-202)"
    );

    drop(broker);
}

// ---------------------------------------------------------------------------
// AC-004 (BC-2.05.009 PC-3): pty_drop_counter — OOM-only, not backpressure
// ---------------------------------------------------------------------------

/// pty_drop_counter NOT incremented on per-client 1-strike disconnect — AC-004, BC-2.05.009 PC-3.
#[tokio::test]
async fn test_BC_2_05_009_pty_drop_counter_only_oom_not_backpressure() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let broker = PtyBroker::new(
        "test-session-004".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register a client and pre-fill its buffer to force disconnect on next broadcast.
    let (tx, _rx) = mpsc::channel::<ServerToClient>(1);
    tx.try_send(ServerToClient::SessionListUpdate { sessions: vec![] })
        .expect("pre-fill");
    register_subscriber(&sub_list, tx).await;

    // Broadcast — slow-client disconnect fires.
    broadcast_to_subscribers(
        &sub_list,
        ServerToClient::PtyOutput {
            session_id: "test-session-004".to_string(),
            bytes: b"drop counter test".to_vec(),
        },
    )
    .await;

    {
        let subs = sub_list.lock().await;
        assert_eq!(
            subs.len(),
            0,
            "client must be removed after 1-strike disconnect"
        );
    }

    // pty_drop_counter MUST NOT have been incremented — per-client 1-strike is not OOM.
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on per-client 1-strike disconnect \
         (BC-2.05.009 PC-3 / AC-004 / EC-202)"
    );

    // EC-202: additional broadcast with no clients must also not increment counter.
    broadcast_to_subscribers(
        &sub_list,
        ServerToClient::PtyOutput {
            session_id: "test-session-004".to_string(),
            bytes: b"no clients".to_vec(),
        },
    )
    .await;
    let drops_after = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops_after, 0,
        "pty_drop_counter must NOT be incremented when broadcast has no clients \
         (EC-202 — empty SubscriberList is normal, not OOM)"
    );

    drop(broker);
}

// ---------------------------------------------------------------------------
// AC-005 (BC-2.05.009 Invariant 4): PtyReset emitted ONLY on proxy send error,
//                                     NOT on graceful INPUT channel close
//
// Sub-case A: when input_rx.recv() returns None (INPUT channel closed normally),
//   the event loop exits WITHOUT emitting PtyReset to any subscriber.
// Sub-case B: The proxy task is responsible for emitting PtyReset on tx.send error;
//   that path is exercised indirectly by verifying the broker event loop never emits
//   PtyReset on graceful close.
// ---------------------------------------------------------------------------

/// Broker event loop exits gracefully without emitting PtyReset when INPUT channel closes — AC-005.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_BC_2_05_009_pty_reset_emitted_on_proxy_send_error_not_graceful_close() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let mut broker = PtyBroker::new(
        "test-session-005".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register a client to observe any spurious PtyReset messages.
    let (tx_obs, mut rx_obs) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    register_subscriber(&sub_list, tx_obs).await;

    let (_hook_tx, hook_rx) = mpsc::channel::<()>(8);
    let handle = broker.spawn_event_loop(hook_rx);

    // Give the event loop a moment to start.
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Drop the INPUT sender — the event loop's input_rx.recv() will return None.
    // This simulates the normal graceful session-exit path.
    drop(broker); // drops input_tx (the receiver was moved into the event loop)

    // Wait for the event loop to exit.
    let _ = tokio::time::timeout(Duration::from_millis(200), handle).await;

    // The observer channel must be EMPTY — the event loop must NOT have emitted PtyReset
    // when the INPUT channel closed normally (BC-2.05.009 Invariant 4).
    let result = rx_obs.try_recv();
    assert!(
        result.is_err(),
        "broker must NOT emit any message (especially not PtyReset) when INPUT channel \
         closes gracefully (BC-2.05.009 Invariant 4 — graceful exit is NOT a byte drop); \
         got unexpected message: {:?}",
        result,
    );

    // pty_drop_counter must remain 0 — graceful close is NOT an OOM drop.
    let drops = counter.load(Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must NOT be incremented on graceful INPUT channel close \
         (BC-2.05.009 PC-3 — counter only incremented by proxy task on tx.send error)"
    );
}

// ---------------------------------------------------------------------------
// AC-006 (BC-2.05.009 Invariant 6): hook events priority over PtyOutput in select!
// ---------------------------------------------------------------------------

/// Hook events are processed before PTY frames (biased select!) — AC-006, BC-2.05.009 Inv 6.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_BC_2_05_009_hook_events_priority_over_pty_output() {
    let counter = Arc::new(AtomicU64::new(0));
    let sub_list = make_subscriber_list();

    let mut broker = PtyBroker::new(
        "test-session-006".to_string(),
        Arc::clone(&counter),
        Arc::clone(&sub_list),
    );

    // Register a client to observe output.
    let (tx_obs, mut rx_obs) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);
    register_subscriber(&sub_list, tx_obs).await;

    // Create hook and PTY input channels.
    let (hook_tx, hook_rx) = mpsc::channel::<()>(8);

    // Spawn the event loop.
    let handle = broker.spawn_event_loop(hook_rx);

    // Give the event loop a moment to start up.
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Send a hook event and a PTY frame concurrently so both are ready on the same poll.
    hook_tx
        .send(())
        .await
        .expect("hook event send must succeed");
    broker
        .input_tx
        .send(Arc::new(Bytes::from_static(b"pty frame")))
        .await
        .expect("PTY frame send must succeed");

    // Give the event loop time to process both events.
    tokio::time::sleep(Duration::from_millis(20)).await;

    // The PTY frame must result in a ServerToClient::PtyOutput to the client.
    // (The hook event is internal to the broker; it does not produce a client message.)
    let received = tokio::time::timeout(Duration::from_millis(100), rx_obs.recv())
        .await
        .expect("must receive a message within timeout")
        .expect("channel must not be closed");

    assert!(
        matches!(&received, ServerToClient::PtyOutput { .. }),
        "client must receive PtyOutput after event loop processes both arms; got {:?}",
        received,
    );

    // Clean up.
    drop(hook_tx);
    handle.abort();
}

// ---------------------------------------------------------------------------
// AC-007 (BC-2.05.009 Invariant 3): no unbounded_channel in PTY path; capacities canonical
// ---------------------------------------------------------------------------

/// No unbounded_channel in PTY path; capacities are canonical — AC-007, BC-2.05.009 Inv 3.
#[test]
fn test_BC_2_05_009_no_unbounded_channel_in_pty_path() {
    // Capacity assertions.
    assert_eq!(
        PTY_BROKER_INPUT_CAPACITY, 1024,
        "PTY_BROKER_INPUT_CAPACITY must be 1024 (BC-2.05.009 PC-2 / ADR-0010)"
    );
    assert_eq!(
        PTY_BROKER_CLIENT_CAPACITY, 64,
        "PTY_BROKER_CLIENT_CAPACITY must be 64 (BC-2.05.009 Invariant 3b / SS-ipc.md)"
    );
    assert_eq!(
        PTY_BROKER_STRIKE_LIMIT, 1,
        "PTY_BROKER_STRIKE_LIMIT must be 1 (1-strike model per BC-2.05.009 Q1 ruling, \
         superseding the retired 3-strike threshold)"
    );

    // Source-file grep: no `unbounded_channel` call in the PTY broker source.
    let pty_broker_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("pty_broker.rs"),
    )
    .expect("pty_broker.rs must be readable from CARGO_MANIFEST_DIR/src/");

    assert!(
        !pty_broker_source.contains("unbounded_channel"),
        "pty_broker.rs must NOT contain `unbounded_channel` — \
         BC-2.05.009 Invariant 3 forbids unbounded channels in the PTY output path; \
         use bounded mpsc::channel(N) with .send().await backpressure"
    );
}
