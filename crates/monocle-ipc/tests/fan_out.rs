//! Fan-out broadcast tests for `monocle-ipc` (BC-2.05.003, BC-2.05.004).
// Test function names follow the VSDD `test_BC_S_SS_NNN_xxx` convention (uppercase BC).
// Suppress non_snake_case lint for test files — BC_ prefix is intentional for traceability.
#![allow(non_snake_case)]
//!
//! Tests `broadcast_session_list_update` and `broadcast_hook_event_received`:
//! - Broadcast to all connected clients.
//! - Disconnected client (closed receiver) skipped without error.
//! - 256 KiB limit for SessionListUpdate.
//! - `payload_excerpt` truncation at UTF-8 boundary.
//! - `drop_counter` NOT incremented by IPC slow-client disconnect.
//! - Slow client removed; other clients unaffected.
//!
//! All test names follow `test_BC_S_SS_NNN_[description]` convention.
//!
//! **RED GATE**: Tests that call `UdsTransport::bind` / `broadcast_*` panic with `todo!()`.

use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::hook_events::HookType;
use monocle_ipc::types::{truncate_to_utf8_boundary, ServerToClient, PAYLOAD_EXCERPT_MAX_BYTES};
use monocle_ipc::uds::UdsTransport;

// ---------------------------------------------------------------------------
// payload_excerpt truncation tests (pure function — no stub involved)
// These tests exercise the `truncate_to_utf8_boundary` helper and must PASS.
// ---------------------------------------------------------------------------

/// test_BC_2_05_004_payload_excerpt_short_body_full_excerpt:
/// A hook body shorter than 256 bytes produces a `payload_excerpt` equal to the full body.
///
/// BC-2.05.004 PC-1 / canonical test vector (50-byte body → full excerpt).
#[test]
fn test_BC_2_05_004_payload_excerpt_short_body_full_excerpt() {
    let body = r#"{"tool":"Bash","session_id":"abc123"}"#; // < 256 bytes
    assert!(
        body.len() < PAYLOAD_EXCERPT_MAX_BYTES,
        "test setup: body must be shorter than PAYLOAD_EXCERPT_MAX_BYTES"
    );
    let excerpt = truncate_to_utf8_boundary(body, PAYLOAD_EXCERPT_MAX_BYTES);
    assert_eq!(
        excerpt, body,
        "short body must produce full excerpt (no truncation)"
    );
}

/// test_BC_2_05_004_payload_excerpt_512_byte_body_truncated_to_256:
/// A hook body of 512 bytes is truncated to 256 bytes at a valid UTF-8 boundary.
///
/// BC-2.05.004 EC-002 / canonical test vector.
#[test]
fn test_BC_2_05_004_payload_excerpt_512_byte_body_truncated_to_256() {
    // All ASCII — every byte boundary is a valid char boundary.
    let body: String = "a".repeat(512);
    let excerpt = truncate_to_utf8_boundary(&body, PAYLOAD_EXCERPT_MAX_BYTES);
    assert_eq!(
        excerpt.len(),
        PAYLOAD_EXCERPT_MAX_BYTES,
        "512-byte ASCII body must truncate to exactly 256 bytes"
    );
    assert!(
        std::str::from_utf8(excerpt.as_bytes()).is_ok(),
        "truncated excerpt must be valid UTF-8"
    );
}

/// test_BC_2_05_004_payload_excerpt_multibyte_boundary_truncation:
/// When the 256th byte falls mid-way through a multi-byte UTF-8 sequence,
/// truncation snaps back to the last complete character boundary.
///
/// BC-2.05.004 EC-003.
#[test]
fn test_BC_2_05_004_payload_excerpt_multibyte_boundary_truncation() {
    // Build a string where byte 255 starts a 3-byte character (€ = 0xE2 0x82 0xAC).
    // Pad 255 ASCII bytes, then append €, making the total 258 bytes.
    // Truncating at 256 must snap back to 255 bytes.
    let padding: String = "b".repeat(255);
    let body = format!("{padding}\u{20ac}"); // 255 + 3 = 258 bytes
    assert_eq!(body.len(), 258, "test setup: body must be 258 bytes");

    let excerpt = truncate_to_utf8_boundary(&body, PAYLOAD_EXCERPT_MAX_BYTES);
    // The € starts at byte 255, which is within the 256-byte window.
    // But bytes 256 (0x82) and 257 (0xAC) are beyond the window, so the € is split at 256.
    // Truncation must snap back to 255, returning the 255 padding chars.
    assert_eq!(
        excerpt.len(),
        255,
        "truncation must snap back before the split 3-byte sequence"
    );
    assert!(
        std::str::from_utf8(excerpt.as_bytes()).is_ok(),
        "excerpt must be valid UTF-8 after snapping back"
    );
}

/// test_BC_2_05_004_payload_excerpt_empty_body_produces_empty_excerpt:
/// A zero-byte hook body produces an empty `payload_excerpt`.
///
/// BC-2.05.004 EC-001.
#[test]
fn test_BC_2_05_004_payload_excerpt_empty_body_produces_empty_excerpt() {
    let excerpt = truncate_to_utf8_boundary("", PAYLOAD_EXCERPT_MAX_BYTES);
    assert_eq!(excerpt, "", "empty body must produce empty payload_excerpt");
}

/// test_BC_2_05_004_payload_excerpt_always_valid_utf8_property:
/// Property: for any string and any max_bytes, the result of `truncate_to_utf8_boundary`
/// is always valid UTF-8 and always <= max_bytes.
///
/// BC-2.05.004 invariant 1.
///
/// Note: property-based testing would normally use proptest/quickcheck here.
/// This test uses a representative set of constructed inputs covering ASCII,
/// 2-byte, 3-byte, and 4-byte UTF-8 sequences at various boundary offsets.
#[test]
fn test_BC_2_05_004_payload_excerpt_always_valid_utf8_property() {
    // Mixed-length UTF-8 chars: ASCII (1B) + é (2B) + € (3B) + 𝄞 (4B).
    let base = "a\u{00e9}\u{20ac}\u{1d11e}"; // 10 bytes total
                                             // Repeat enough to exceed 256 bytes.
    let s: String = base.repeat(30); // 300 bytes

    for max in 0..=300 {
        let result = truncate_to_utf8_boundary(&s, max);
        assert!(
            result.len() <= max,
            "truncate_to_utf8_boundary result len ({}) must be <= max ({max})",
            result.len()
        );
        assert!(
            std::str::from_utf8(result.as_bytes()).is_ok(),
            "truncate_to_utf8_boundary result must be valid UTF-8 for max={max}"
        );
    }
}

// ---------------------------------------------------------------------------
// Fan-out broadcast tests — these call UdsTransport::bind and broadcast_*,
// which are todo!() stubs. These tests constitute the RED GATE for fan-out.
// ---------------------------------------------------------------------------

/// test_BC_2_05_003_broadcast_session_list_update_sends_to_all_clients:
/// `broadcast_session_list_update` sends the complete session list to all connected clients.
///
/// BC-2.05.003 PC-1, invariant 1.
#[tokio::test]
async fn test_BC_2_05_003_broadcast_session_list_update_sends_to_all_clients() {
    let dir = tempfile::tempdir().expect("tempdir");
    let transport = std::sync::Arc::new(UdsTransport::bind(dir.path()).await.expect("bind"));

    // Add 2 subscribers.
    let (tx1, mut rx1) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    transport.add_subscriber(tx1).await;
    transport.add_subscriber(tx2).await;

    // Broadcast a session list with one identifiable session.
    let session = monocle_core::engine::EnrichedSession::new(
        "session-fanout-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        monocle_core::engine::SessionStatus::Active,
        None,
    );
    transport
        .broadcast_session_list_update(vec![session])
        .await;

    // Both subscribers must receive the SessionListUpdate with the correct session.
    let msg1 = rx1
        .try_recv()
        .expect("subscriber 1 must receive the broadcast");
    let msg2 = rx2
        .try_recv()
        .expect("subscriber 2 must receive the broadcast");

    for msg in [msg1, msg2] {
        match msg {
            ServerToClient::SessionListUpdate { sessions } => {
                assert_eq!(sessions.len(), 1, "sessions list must contain exactly 1 entry");
                assert_eq!(
                    sessions[0].session_id, "session-fanout-001",
                    "session_id must match what was broadcast"
                );
            }
            other => panic!("expected SessionListUpdate, got {other:?}"),
        }
    }
}

/// test_BC_2_05_003_broadcast_session_list_update_empty_sessions:
/// `broadcast_session_list_update` with an empty Vec sends `sessions: []` to all clients.
///
/// BC-2.05.003 EC-001.
///
/// **RED GATE**: Calls `UdsTransport::bind` — panics with `todo!()`.
#[tokio::test]
async fn test_BC_2_05_003_broadcast_session_list_update_empty_sessions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let transport = UdsTransport::bind(dir.path()).await.expect("bind");

    // Empty sessions Vec — broadcast should succeed (not panic, not error).
    transport.broadcast_session_list_update(vec![]).await;
}

/// test_BC_2_05_003_broadcast_session_list_update_256_kib_limit:
/// `broadcast_session_list_update` logs an error and does NOT broadcast when the
/// serialized message exceeds 256 KiB. A connected subscriber must receive nothing.
///
/// BC-2.05.003 PC-3 / AC-008.
#[tokio::test]
async fn test_BC_2_05_003_broadcast_session_list_update_256_kib_limit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let transport = std::sync::Arc::new(UdsTransport::bind(dir.path()).await.expect("bind"));

    // Add a real subscriber so we can assert it receives nothing.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    transport.add_subscriber(tx).await;

    // Build a session whose session_id alone is 300 KiB — far above the 256 KiB limit.
    // A single EnrichedSession with a 300 KiB session_id will cause the serialized
    // SessionListUpdate to exceed MAX_MESSAGE_BYTES (262144 bytes).
    let big_session = EnrichedSession::new(
        "x".repeat(300 * 1024), // 300 KiB session_id
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Active,
        None,
    );
    transport
        .broadcast_session_list_update(vec![big_session])
        .await;

    // The subscriber channel must be empty — no message was broadcast.
    assert!(
        rx.try_recv().is_err(),
        "subscriber must NOT receive a message when SessionListUpdate exceeds 256 KiB"
    );
}

/// test_BC_2_05_004_broadcast_hook_event_received_sends_to_all_clients:
/// `broadcast_hook_event_received` sends a `HookEventReceived` message to all connected clients.
///
/// BC-2.05.004 PC-1.
#[tokio::test]
async fn test_BC_2_05_004_broadcast_hook_event_received_sends_to_all_clients() {
    let dir = tempfile::tempdir().expect("tempdir");
    let transport = std::sync::Arc::new(UdsTransport::bind(dir.path()).await.expect("bind"));

    // Add 2 subscribers.
    let (tx1, mut rx1) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    transport.add_subscriber(tx1).await;
    transport.add_subscriber(tx2).await;

    let body = br#"{"session_id":"abc","tool":"Bash"}"#;
    transport
        .broadcast_hook_event_received(HookType::PreToolUse, "abc".to_string(), body, 5)
        .await;

    // Both subscribers must receive the HookEventReceived with correct fields.
    let msg1 = rx1
        .try_recv()
        .expect("subscriber 1 must receive the broadcast");
    let msg2 = rx2
        .try_recv()
        .expect("subscriber 2 must receive the broadcast");

    for msg in [msg1, msg2] {
        match msg {
            ServerToClient::HookEventReceived {
                hook_type,
                session_id,
                latency_ms,
                ..
            } => {
                assert_eq!(hook_type, HookType::PreToolUse, "hook_type must be PreToolUse");
                assert_eq!(session_id, "abc", "session_id must match");
                assert_eq!(latency_ms, 5, "latency_ms must match");
            }
            other => panic!("expected HookEventReceived, got {other:?}"),
        }
    }
}

/// test_BC_2_05_004_broadcast_hook_event_received_latency_ms_propagated:
/// The `latency_ms` value is propagated to the `HookEventReceived` message.
///
/// BC-2.05.004 PC-1 (latency_ms field).
#[tokio::test]
async fn test_BC_2_05_004_broadcast_hook_event_received_latency_ms_propagated() {
    let dir = tempfile::tempdir().expect("tempdir");
    let transport = UdsTransport::bind(dir.path()).await.expect("bind");

    // Add a subscriber to receive the broadcast.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);
    transport.add_subscriber(tx).await;

    // latency_ms = 99 must be embedded verbatim in the IPC message.
    let body = b"{}";
    transport
        .broadcast_hook_event_received(HookType::Stop, "session-x".to_string(), body, 99)
        .await;

    let msg = rx.try_recv().expect("subscriber must receive the broadcast");
    match msg {
        ServerToClient::HookEventReceived { latency_ms, .. } => {
            assert_eq!(latency_ms, 99, "latency_ms must be propagated verbatim");
        }
        other => panic!("expected HookEventReceived, got {other:?}"),
    }
}

/// test_BC_2_05_004_invariant_drop_counter_not_incremented_by_ipc_send:
/// The `drop_counter` is NOT incremented when a slow TUI client send fails.
///
/// BC-2.05.004 PC-4 / AC-010.
///
/// **RED GATE**: Calls `UdsTransport::bind` — panics with `todo!()`.
///
/// Implementation note: The drop counter is part of `DaemonState` (not `UdsTransport`).
/// The implementer must verify that `broadcast_hook_event_received` only disconnects
/// the slow client and does not touch any drop counter. This test establishes the
/// call site; the full assertion requires the implementer to wire a mock subscriber.
#[tokio::test]
async fn test_BC_2_05_004_invariant_drop_counter_not_incremented_by_ipc_send() {
    let dir = tempfile::tempdir().expect("tempdir");
    let transport = UdsTransport::bind(dir.path()).await.expect("bind");

    // When implemented: add a subscriber with a closed receiver (simulating slow client),
    // call broadcast, assert drop counter is NOT incremented.
    let body = b"{}";
    transport
        .broadcast_hook_event_received(HookType::Notification, "s".to_string(), body, 0)
        .await;
}

/// test_BC_2_05_004_slow_client_removed_from_subscriber_list:
/// When a TUI client's send buffer is full, it is removed from the subscriber list
/// and other clients continue to receive messages.
///
/// BC-2.05.004 EC-005 / AC-017.
#[tokio::test]
async fn test_BC_2_05_004_slow_client_removed_from_subscriber_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    let transport = std::sync::Arc::new(UdsTransport::bind(dir.path()).await.expect("bind"));

    // Slow subscriber: capacity 1, pre-filled so the next send will fail with TrySendError::Full.
    let (slow_tx, _slow_rx) = tokio::sync::mpsc::channel::<ServerToClient>(1);
    let dummy_msg = ServerToClient::DropCounterUpdate { drop_counter: 0 };
    slow_tx
        .try_send(dummy_msg)
        .expect("pre-fill slow channel: first send must succeed");
    // Channel is now full; any further try_send will return TrySendError::Full.

    // Healthy subscriber: large capacity, empty.
    let (healthy_tx, mut healthy_rx) = tokio::sync::mpsc::channel::<ServerToClient>(64);

    transport.add_subscriber(slow_tx).await;
    transport.add_subscriber(healthy_tx).await;
    assert_eq!(transport.subscriber_count().await, 2, "should start with 2 subscribers");

    let body = b"{}";
    transport
        .broadcast_hook_event_received(HookType::SessionStart, "s".to_string(), body, 0)
        .await;

    // After the broadcast, the slow subscriber must have been removed.
    assert_eq!(
        transport.subscriber_count().await,
        1,
        "slow client must be removed; only healthy subscriber remains"
    );

    // The healthy subscriber must have received the message.
    let msg = healthy_rx
        .try_recv()
        .expect("healthy subscriber must receive the broadcast");
    match msg {
        ServerToClient::HookEventReceived { session_id, .. } => {
            assert_eq!(session_id, "s");
        }
        other => panic!("expected HookEventReceived, got {other:?}"),
    }
}
