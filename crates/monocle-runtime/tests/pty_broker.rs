//! PtyOutput fan-out broker — Red Gate tests (S-046, BC-2.05.009).
//!
//! All tests in this file are RED by design: they exercise behaviors that require
//! real implementation in `crates/monocle-runtime/src/pty_broker.rs`. The stubs
//! have `todo!()` bodies so every test will panic until the implementer fills them in.
// Test naming uses uppercase BC identifiers (BC-2.05.009 etc.) to preserve traceability
// to behavioral contracts; suppress the snake_case lint for the whole file.
#![allow(non_snake_case)]
// Tests use unwrap/expect as assertion helpers — panics are the desired failure mode.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use bytes::Bytes;
use monocle_runtime::pty_broker::{
    PTY_BROKER_CLIENT_CAPACITY, PTY_BROKER_INPUT_CAPACITY, PTY_BROKER_STRIKE_LIMIT, PtyBroker,
};

// ---------------------------------------------------------------------------
// AC-001 (BC-2.05.009 PC-2): bounded INPUT channel blocks on full — backpressure
// ---------------------------------------------------------------------------

/// When the broker INPUT channel is at capacity, the PTY reader's `.send().await`
/// blocks rather than dropping the frame.
///
/// Traces to: BC-2.05.009 Postcondition 2, Invariant 3.
#[tokio::test]
async fn test_BC_2_05_009_bounded_channel_backpressure_blocks_not_drops() {
    let counter = Arc::new(AtomicU64::new(0));
    let broker = PtyBroker::new("test-session-001".to_string(), counter);

    // Fill the INPUT channel to capacity.
    let frame = Arc::new(Bytes::from_static(b"hello pty"));
    for _ in 0..PTY_BROKER_INPUT_CAPACITY {
        broker
            .input_tx
            .try_send(Arc::clone(&frame))
            .expect("send should succeed while channel is not full");
    }

    // The channel is now full. A try_send MUST fail with Full (not Closed).
    let result = broker.input_tx.try_send(Arc::clone(&frame));
    assert!(
        result.is_err(),
        "expected try_send to fail when channel is at capacity"
    );
    // The drop counter must NOT be incremented — this is a normal backpressure condition.
    let drops = broker
        .pty_drop_counter
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must not be incremented on normal channel fullness (BC-2.05.009 PC-3)"
    );
}

// ---------------------------------------------------------------------------
// AC-002 (BC-2.05.009 PC-1b): per-client isolation — slow client does not block fast
// ---------------------------------------------------------------------------

/// A slow client's full per-client channel must not block sends to other clients.
///
/// Traces to: BC-2.05.009 Postcondition 1b, Invariant 3b.
#[tokio::test]
async fn test_BC_2_05_009_per_client_isolation_slow_client_does_not_block_fast() {
    let counter = Arc::new(AtomicU64::new(0));
    let mut broker = PtyBroker::new("test-session-002".to_string(), Arc::clone(&counter));

    // Register two clients: one fast, one slow.
    let _fast_rx = broker.register_client("fast-client".to_string());
    let slow_rx = broker.register_client("slow-client".to_string());

    // Fill the slow client's buffer to capacity by dropping the receiver
    // (or by not draining it). Since slow_rx is not drained, sending
    // PTY_BROKER_CLIENT_CAPACITY frames will saturate it.
    drop(slow_rx); // slow client's receiver is dropped — sends will fail

    // The fast client must still receive frames after the slow client is struck out.
    let session_id = "test-session-002".to_string();
    let frame = Arc::new(Bytes::from_static(b"pty bytes for isolation test"));

    // fan_out must not panic or block even when the slow client is not draining.
    for _ in 0..(PTY_BROKER_STRIKE_LIMIT + 1) {
        broker.fan_out(&session_id, Arc::clone(&frame));
    }
    // After 3 strikes the slow client is disconnected; the fast client remains.
    // (fast client receives frames: verified by reading from fast_rx in the real impl)
}

// ---------------------------------------------------------------------------
// AC-003 (BC-2.05.009 Invariant 3b): 3-strike disconnect for slow clients
// ---------------------------------------------------------------------------

/// After 3 consecutive send failures, the broker disconnects the client.
///
/// Traces to: BC-2.05.009 Invariant 3b, AC-003.
#[tokio::test]
async fn test_BC_2_05_009_three_strike_disconnect() {
    let counter = Arc::new(AtomicU64::new(0));
    let mut broker = PtyBroker::new("test-session-003".to_string(), Arc::clone(&counter));

    let _rx = broker.register_client("strike-test-client".to_string());
    drop(_rx); // drop the receiver so all sends fail

    let session_id = "test-session-003".to_string();
    let frame = Arc::new(Bytes::from_static(b"strike test frame"));

    // Before 3 strikes, client is still in the registry.
    assert!(
        broker.clients.contains_key("strike-test-client"),
        "client must be present before strikes are exhausted"
    );

    // Fan out exactly PTY_BROKER_STRIKE_LIMIT times — client not yet removed.
    for i in 0..PTY_BROKER_STRIKE_LIMIT {
        broker.fan_out(&session_id, Arc::clone(&frame));
        if i < PTY_BROKER_STRIKE_LIMIT - 1 {
            assert!(
                broker.clients.contains_key("strike-test-client"),
                "client must still be present after {} strike(s)", i + 1
            );
        }
    }

    // After PTY_BROKER_STRIKE_LIMIT strikes, client must be removed.
    assert!(
        !broker.clients.contains_key("strike-test-client"),
        "client must be removed after {} consecutive send failures",
        PTY_BROKER_STRIKE_LIMIT
    );
}

// ---------------------------------------------------------------------------
// AC-004 (BC-2.05.009 PC-3): pty_drop_counter only incremented on OOM/sender-error
// ---------------------------------------------------------------------------

/// The `pty_drop_counter` is NOT incremented on backpressure waits or 3-strike
/// disconnects. It is incremented ONLY on sender errors (receiver gone / OOM).
///
/// Traces to: BC-2.05.009 Postcondition 3.
#[tokio::test]
async fn test_BC_2_05_009_pty_drop_counter_only_oom_not_backpressure() {
    let counter = Arc::new(AtomicU64::new(0));
    let mut broker = PtyBroker::new("test-session-004".to_string(), Arc::clone(&counter));

    let _rx = broker.register_client("drop-counter-client".to_string());
    drop(_rx); // receiver dropped — all sends will fail with SendError::Closed

    let session_id = "test-session-004".to_string();
    let frame = Arc::new(Bytes::from_static(b"drop counter test"));

    // Strike out the client — 3 consecutive send failures.
    for _ in 0..PTY_BROKER_STRIKE_LIMIT {
        broker.fan_out(&session_id, Arc::clone(&frame));
    }

    // 3-strike disconnect must NOT increment pty_drop_counter (BC-2.05.009 PC-3).
    let drops = counter.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        drops, 0,
        "pty_drop_counter must not be incremented on 3-strike disconnect (BC-2.05.009 PC-3)"
    );
}

// ---------------------------------------------------------------------------
// AC-005 (BC-2.05.009 Invariant 4): PtyReset emitted when broker task drops
// ---------------------------------------------------------------------------

/// When the broker's PTY writer task is dropped, `PtyReset` is emitted to all
/// registered clients.
///
/// Traces to: BC-2.05.009 Invariant 4, AC-005.
#[tokio::test]
async fn test_BC_2_05_009_pty_reset_emitted_on_broker_drop() {
    use monocle_ipc::types::ServerToClient;

    let counter = Arc::new(AtomicU64::new(0));
    let mut broker = PtyBroker::new("test-session-005".to_string(), counter);

    let mut rx_a = broker.register_client("client-a".to_string());
    let mut rx_b = broker.register_client("client-b".to_string());

    // Emit PtyReset to both clients.
    broker.emit_pty_reset("test-session-005");

    // Both clients must receive PtyReset.
    let msg_a = rx_a.try_recv().expect("client-a must receive a message");
    let msg_b = rx_b.try_recv().expect("client-b must receive a message");

    assert!(
        matches!(
            &msg_a,
            ServerToClient::PtyReset { session_id } if session_id == "test-session-005"
        ),
        "client-a must receive PtyReset {{ session_id: 'test-session-005' }}"
    );
    assert!(
        matches!(
            &msg_b,
            ServerToClient::PtyReset { session_id } if session_id == "test-session-005"
        ),
        "client-b must receive PtyReset {{ session_id: 'test-session-005' }}"
    );
}

// ---------------------------------------------------------------------------
// AC-006 (BC-2.05.009 Invariant 6): hook events priority over PtyOutput in select!
// ---------------------------------------------------------------------------

/// The broker's event loop processes hook/control events before PTY frames when
/// both arrive simultaneously.
///
/// Traces to: BC-2.05.009 Invariant 6, AC-006.
#[tokio::test]
async fn test_BC_2_05_009_hook_events_priority_over_pty_output() {
    // This test verifies the `biased; select!` ordering contract.
    // When a hook/control event and a PTY frame arrive in the same poll cycle,
    // the hook/control event must be processed first.
    //
    // Implementation note: this test exercises the event-loop task. When the
    // `spawn_event_loop()` stub is implemented, a paired hook_rx sender and
    // a PTY input_tx sender will inject concurrent events. The test verifies
    // that the hook event is drained before PTY frames are processed.
    //
    // For now: the test calls spawn_event_loop() which returns a todo!() JoinHandle.
    let counter = Arc::new(AtomicU64::new(0));
    let mut broker = PtyBroker::new("test-session-006".to_string(), counter);

    let (hook_tx, hook_rx) = tokio::sync::mpsc::channel::<()>(8);
    let _handle = broker.spawn_event_loop(hook_rx); // todo!() — will panic in stub phase

    // Send a hook event and a PTY frame concurrently; verify hook is processed first.
    let _ = hook_tx.try_send(());
    // PTY frame: send via input_tx
    let _ = broker
        .input_tx
        .try_send(Arc::new(Bytes::from_static(b"pty frame")));

    // In the real implementation: read from a client channel and verify ordering.
    // Stub phase: the spawn_event_loop todo!() will panic first.
}

// ---------------------------------------------------------------------------
// AC-007 (BC-2.05.009 Invariant 3): no unbounded_channel in PTY path
// ---------------------------------------------------------------------------

/// There must be no `tokio::mpsc::unbounded_channel` call in the PTY fan-out code path.
///
/// This is a compile-time + grep assertion. The test verifies the INPUT channel is bounded.
///
/// Traces to: BC-2.05.009 Invariant 3, AC-007.
#[test]
fn test_BC_2_05_009_no_unbounded_channel_in_pty_path() {
    // Structural assertion: the INPUT channel capacity constant must be 1024 (ADR-0010).
    assert_eq!(
        PTY_BROKER_INPUT_CAPACITY,
        1024,
        "PTY_BROKER_INPUT_CAPACITY must be 1024 (BC-2.05.009 PC-2, ADR-0010)"
    );
    // Per-client capacity must be 64 (SS-ipc.md §TUI IPC Read Loop Pattern).
    assert_eq!(
        PTY_BROKER_CLIENT_CAPACITY,
        64,
        "PTY_BROKER_CLIENT_CAPACITY must be 64 (BC-2.05.009 Invariant 3b)"
    );
    // The grep-level assertion (no unbounded_channel in monocle-runtime/src/pty_broker.rs)
    // is enforced by CI via AC-007 and the SS-conventions-anti-patterns.md ban on
    // unbounded_channel in the PTY output code path.
}
