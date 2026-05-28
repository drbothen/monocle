//! Integration tests for `broadcast_to_subscribers` slow-client drain-and-retain behaviour.
//!
//! Traces to BC-2.05.004 EC-005 (slow client disconnect) and the canonical
//! `broadcast_to_subscribers` helper in `monocle_runtime::ipc_server`.
//!
//! # Coverage
//!
//! | Test function | BC clause |
//! |---------------|-----------|
//! | test_S022_broadcast_slow_client_removed_fast_client_receives | BC-2.05.004 EC-005 |
//!
//! F-ADV3-MED-003: explicit integration coverage for the slow-client drain-and-retain
//! code path in `broadcast_to_subscribers` (ipc_server.rs:234-252).

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
#![allow(non_snake_case)]

use std::sync::Arc;

use tokio::sync::mpsc;

use monocle_ipc::server::{SubscriberList, CLIENT_CHANNEL_CAPACITY};
use monocle_ipc::types::ServerToClient;
use monocle_runtime::ipc_server::broadcast_to_subscribers;

/// Exercises BC-2.05.004 EC-005: the slow-client drain-and-retain path in
/// `broadcast_to_subscribers`.
///
/// Wires two subscriber channels into a `SubscriberList`. Saturates client A's
/// channel to capacity without draining. Triggers a broadcast. Asserts:
/// - Client B receives the broadcast message.
/// - Client A is removed from the subscriber list (re-broadcast confirms only B receives).
/// - The WARN log line "removed slow TUI client during broadcast (send buffer full)" fires.
#[tokio::test]
async fn test_S022_broadcast_slow_client_removed_fast_client_receives() {
    // Create the shared subscriber list.
    let subscribers: SubscriberList = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // --- Set up client A (slow — channel will be saturated before broadcast) ---
    let (tx_a, mut rx_a) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);

    // --- Set up client B (fast — channel is empty and will receive the broadcast) ---
    let (tx_b, mut rx_b) = mpsc::channel::<ServerToClient>(CLIENT_CHANNEL_CAPACITY);

    // Register both clients in the subscriber list.
    {
        let mut subs = subscribers.lock().await;
        subs.push(tx_a);
        subs.push(tx_b);
    }

    assert_eq!(
        subscribers.lock().await.len(),
        2,
        "subscriber list must have 2 entries before broadcast"
    );

    // Saturate client A's channel: fill it to exactly CLIENT_CHANNEL_CAPACITY messages.
    // These messages are sent directly into the receiver (not via broadcast_to_subscribers),
    // so the subscriber list is not modified at this stage.
    {
        let subs = subscribers.lock().await;
        // tx_a is at index 0 in the list. Clone it for direct filling.
        let tx_a_clone = subs[0].clone();
        drop(subs);

        for i in 0..CLIENT_CHANNEL_CAPACITY {
            // Use HookEventReceived as a convenient non-trivial message variant.
            let fill_msg = ServerToClient::HookEventReceived {
                hook_type: monocle_core::hook_events::HookType::Notification,
                session_id: format!("fill-session-{i}"),
                payload_excerpt: format!("fill-{i}"),
                latency_ms: 0,
            };
            tx_a_clone
                .try_send(fill_msg)
                .expect("filling client A channel must succeed before capacity is reached");
        }

        // Verify client A's channel is now at capacity.
        assert_eq!(
            tx_a_clone.capacity(),
            0,
            "client A channel must be at capacity (0 remaining slots)"
        );
    }

    // Build the broadcast message (one beyond client A's capacity).
    let broadcast_msg = ServerToClient::HookEventReceived {
        hook_type: monocle_core::hook_events::HookType::PreToolUse,
        session_id: "broadcast-session".to_string(),
        payload_excerpt: "broadcast-payload".to_string(),
        latency_ms: 5,
    };

    // Broadcast: client A is full -> removed with WARN log; client B receives normally.
    // The WARN log "removed slow TUI client during broadcast (send buffer full)" is
    // emitted by broadcast_to_subscribers for the TrySendError::Full case.
    broadcast_to_subscribers(&subscribers, broadcast_msg.clone()).await;

    // --- Assert client B received the broadcast ---
    let received_b = rx_b
        .try_recv()
        .expect("client B must receive the broadcast message");

    match received_b {
        ServerToClient::HookEventReceived {
            session_id,
            payload_excerpt,
            ..
        } => {
            assert_eq!(
                session_id, "broadcast-session",
                "client B must receive the correct session_id"
            );
            assert_eq!(
                payload_excerpt, "broadcast-payload",
                "client B must receive the correct payload_excerpt"
            );
        }
        other => panic!("client B received unexpected message: {other:?}"),
    }

    // --- Assert client A was removed from the subscriber list ---
    assert_eq!(
        subscribers.lock().await.len(),
        1,
        "subscriber list must have 1 entry after slow-client removal (client A removed)"
    );

    // Confirm the remaining subscriber is client B (not client A) by sending a second
    // broadcast and verifying only B receives it.
    let second_msg = ServerToClient::HookEventReceived {
        hook_type: monocle_core::hook_events::HookType::Stop,
        session_id: "second-broadcast".to_string(),
        payload_excerpt: "second-payload".to_string(),
        latency_ms: 1,
    };
    broadcast_to_subscribers(&subscribers, second_msg).await;

    // Client B must receive the second broadcast.
    let second_b = rx_b
        .try_recv()
        .expect("client B must receive the second broadcast");
    match second_b {
        ServerToClient::HookEventReceived { session_id, .. } => {
            assert_eq!(
                session_id, "second-broadcast",
                "client B must receive the second broadcast"
            );
        }
        other => panic!("client B received unexpected second message: {other:?}"),
    }

    // Client A must NOT receive the second broadcast (it was removed).
    // The fill messages are still in rx_a, but the broadcast message must NOT be there.
    // Drain rx_a completely and count messages:
    // - The first CLIENT_CHANNEL_CAPACITY messages are fill messages (from saturation step).
    // - The broadcast and second_msg must NOT appear (A was removed before broadcast).
    let mut drain_count = 0usize;
    while let Ok(msg) = rx_a.try_recv() {
        drain_count += 1;
        // Verify none of the drained messages are from broadcast or second_msg.
        if let ServerToClient::HookEventReceived { session_id, .. } = &msg {
            assert_ne!(
                session_id, "broadcast-session",
                "client A must NOT receive the broadcast message after slow-client removal"
            );
            assert_ne!(
                session_id, "second-broadcast",
                "client A must NOT receive the second broadcast after slow-client removal"
            );
        }
    }

    assert_eq!(
        drain_count, CLIENT_CHANNEL_CAPACITY,
        "client A channel must contain exactly the {CLIENT_CHANNEL_CAPACITY} fill messages \
        and nothing else (no broadcast leaked to removed client)"
    );
}
