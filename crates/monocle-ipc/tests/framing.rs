//! Framing tests for `monocle-ipc` (BC-2.05.008 PC-5/PC-6).
// Test function names follow the VSDD `test_BC_S_SS_NNN_xxx` convention (uppercase BC).
// Suppress non_snake_case lint for test files — BC_ prefix is intentional for traceability.
#![allow(non_snake_case)]
//!
//! Tests the 4-byte LE length-prefix framing protocol.
//!
//! # Test naming convention
//!
//! All tests follow `test_BC_S_SS_NNN_[description]` per VSDD TDD naming policy.
//! The framing logic itself is not BC-tagged (it's a protocol constraint from SS-ipc §Framing),
//! but the 256 KiB limit is BC-2.05.003 PC-3 / BC-2.05.004 invariant 1.

use monocle_ipc::framing::{read_framed, write_framed, MAX_MESSAGE_BYTES};

/// Helper: serialize a value through write_framed into a Vec<u8>.
async fn frame_to_bytes<T: serde::Serialize>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    write_framed(&mut buf, value)
        .await
        .expect("write_framed should succeed");
    buf
}

/// Helper: deserialize from a framed byte slice via read_framed.
async fn deframe_from_bytes<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    let mut cursor = std::io::Cursor::new(bytes);
    read_framed(&mut cursor)
        .await
        .expect("read_framed should succeed")
}

/// test_BC_2_05_003_framing_length_prefix_encodes_correctly:
/// The 4-byte LE prefix in the wire output matches the payload length.
#[tokio::test]
async fn test_BC_2_05_003_framing_length_prefix_encodes_correctly() {
    // Use a simple JSON value so the payload length is predictable.
    let payload = serde_json::json!({"hello": "world"});
    let bytes = frame_to_bytes(&payload).await;

    // First 4 bytes must be the LE u32 payload length.
    assert!(bytes.len() >= 4, "framed output must have at least 4 bytes");
    let declared_len = u32::from_le_bytes(bytes[..4].try_into().expect("4 bytes")) as usize;
    let actual_payload_len = bytes.len() - 4;
    assert_eq!(
        declared_len, actual_payload_len,
        "declared length ({declared_len}) must equal actual payload length ({actual_payload_len})"
    );
}

/// test_BC_2_05_003_framing_roundtrip_preserves_value:
/// A value serialized by write_framed and deserialized by read_framed is equal to the original.
#[tokio::test]
async fn test_BC_2_05_003_framing_roundtrip_preserves_value() {
    let original = serde_json::json!({"key": "value", "num": 42, "nested": {"x": true}});
    let bytes = frame_to_bytes(&original).await;
    let decoded: serde_json::Value = deframe_from_bytes(&bytes).await;
    assert_eq!(
        original, decoded,
        "roundtrip must preserve the original value"
    );
}

/// test_BC_2_05_003_framing_empty_object_roundtrip:
/// An empty JSON object can be framed and unframed.
#[tokio::test]
async fn test_BC_2_05_003_framing_empty_object_roundtrip() {
    let original: serde_json::Value = serde_json::json!({});
    let bytes = frame_to_bytes(&original).await;
    let decoded: serde_json::Value = deframe_from_bytes(&bytes).await;
    assert_eq!(original, decoded);
}

/// test_BC_2_05_003_framing_rejects_payload_exceeding_256_kib:
/// write_framed returns IpcError::MessageTooLarge when the serialized payload > 262,144 bytes.
///
/// This exercises BC-2.05.003 PC-3 (256 KiB limit) and BC-2.05.004 invariant 1.
#[tokio::test]
async fn test_BC_2_05_003_framing_rejects_payload_exceeding_256_kib() {
    // Construct a JSON value that exceeds MAX_MESSAGE_BYTES when serialized.
    // A string of 300,000 'a' characters will be ~300,002 bytes in JSON (with quotes).
    let large_string: String = "a".repeat(300_000);
    let payload = serde_json::json!({"data": large_string});

    let mut buf = Vec::new();
    let result = write_framed(&mut buf, &payload).await;

    assert!(
        result.is_err(),
        "write_framed must return an error for payloads > {MAX_MESSAGE_BYTES} bytes"
    );
    match result.unwrap_err() {
        monocle_ipc::error::IpcError::MessageTooLarge => {}
        other => panic!("expected IpcError::MessageTooLarge, got {other:?}"),
    }
}

/// test_BC_2_05_003_framing_accepts_payload_exactly_at_256_kib:
/// write_framed succeeds for a payload that serializes to exactly MAX_MESSAGE_BYTES bytes.
#[tokio::test]
async fn test_BC_2_05_003_framing_accepts_payload_exactly_at_256_kib() {
    // Build a raw JSON bytes vec of exactly MAX_MESSAGE_BYTES valid JSON.
    // Use a JSON string padded to bring the total to exactly MAX_MESSAGE_BYTES bytes.
    // JSON string encoding: `"` + content + `"` = content.len() + 2 bytes.
    // We want total JSON = MAX_MESSAGE_BYTES bytes:
    //   {"d":"<padding>"} = 7 + padding_len + 1 = 8 + padding_len
    //   => padding_len = MAX_MESSAGE_BYTES - 8
    let padding_len = MAX_MESSAGE_BYTES - 8;
    let padding: String = "a".repeat(padding_len);
    let payload = serde_json::json!({"d": padding});

    // Sanity-check our arithmetic.
    let serialized = serde_json::to_vec(&payload).expect("serialize");
    assert_eq!(
        serialized.len(),
        MAX_MESSAGE_BYTES,
        "test setup: payload must be exactly MAX_MESSAGE_BYTES bytes"
    );

    let mut buf = Vec::new();
    let result = write_framed(&mut buf, &payload).await;
    assert!(
        result.is_ok(),
        "write_framed must succeed for payload == MAX_MESSAGE_BYTES bytes"
    );
}

/// test_BC_2_05_003_framing_read_rejects_declared_length_exceeding_256_kib:
/// read_framed returns IpcError::MessageTooLarge when the length prefix declares > 262,144 bytes.
#[tokio::test]
async fn test_BC_2_05_003_framing_read_rejects_declared_length_exceeding_256_kib() {
    // Craft a byte stream with a length prefix of MAX_MESSAGE_BYTES + 1.
    let declared_len = (MAX_MESSAGE_BYTES + 1) as u32;
    let mut bytes = declared_len.to_le_bytes().to_vec();
    // No actual payload bytes needed — read_framed should reject at length validation.
    // But we need enough bytes or we'll get a Disconnected error first, not MessageTooLarge.
    // So we add the actual bytes too (even though they'll never be read past the check).
    bytes.extend_from_slice(&vec![0u8; MAX_MESSAGE_BYTES + 1]);

    let mut cursor = std::io::Cursor::new(&bytes);
    let result: Result<serde_json::Value, _> = read_framed(&mut cursor).await;

    assert!(
        result.is_err(),
        "read_framed must return an error when declared length > MAX_MESSAGE_BYTES"
    );
    match result.unwrap_err() {
        monocle_ipc::error::IpcError::MessageTooLarge => {}
        other => panic!("expected IpcError::MessageTooLarge, got {other:?}"),
    }
}

/// test_BC_2_05_003_framing_read_returns_disconnected_on_eof:
/// read_framed returns IpcError::Disconnected when the reader returns EOF immediately.
#[tokio::test]
async fn test_BC_2_05_003_framing_read_returns_disconnected_on_eof() {
    let empty: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&empty);
    let result: Result<serde_json::Value, _> = read_framed(&mut cursor).await;

    assert!(
        result.is_err(),
        "read_framed on empty input must return an error"
    );
    match result.unwrap_err() {
        monocle_ipc::error::IpcError::Disconnected => {}
        other => panic!("expected IpcError::Disconnected, got {other:?}"),
    }
}

/// test_BC_2_05_003_framing_le_byte_order:
/// The length prefix is little-endian: a 5-byte payload encodes as [0x05, 0x00, 0x00, 0x00].
#[tokio::test]
async fn test_BC_2_05_003_framing_le_byte_order() {
    // Use a known-length payload: `"hi!"` in JSON = 5 bytes (with quotes).
    let payload = "hi!";
    let mut buf = Vec::new();
    write_framed(&mut buf, &payload)
        .await
        .expect("write_framed");

    // JSON-serialized "hi!" = `"hi!"` = 5 bytes.
    let json_bytes = serde_json::to_vec(&payload).expect("serialize");
    let expected_len = json_bytes.len() as u32;

    let actual_prefix = u32::from_le_bytes(buf[..4].try_into().expect("4 bytes"));
    assert_eq!(
        actual_prefix, expected_len,
        "4-byte prefix must equal payload length in little-endian byte order"
    );
    // Verify LE byte order by checking the raw bytes.
    let expected_bytes = expected_len.to_le_bytes();
    assert_eq!(
        &buf[..4],
        &expected_bytes,
        "first 4 bytes must be payload length in LE byte order"
    );
}

/// test_BC_2_05_003_framing_multiple_messages_sequential:
/// Multiple framed messages can be written and read in sequence from the same stream.
#[tokio::test]
async fn test_BC_2_05_003_framing_multiple_messages_sequential() {
    let mut buf = Vec::new();
    write_framed(&mut buf, &serde_json::json!({"seq": 1}))
        .await
        .expect("write msg1");
    write_framed(&mut buf, &serde_json::json!({"seq": 2}))
        .await
        .expect("write msg2");
    write_framed(&mut buf, &serde_json::json!({"seq": 3}))
        .await
        .expect("write msg3");

    let mut cursor = std::io::Cursor::new(&buf);
    let msg1: serde_json::Value = read_framed(&mut cursor).await.expect("read msg1");
    let msg2: serde_json::Value = read_framed(&mut cursor).await.expect("read msg2");
    let msg3: serde_json::Value = read_framed(&mut cursor).await.expect("read msg3");

    assert_eq!(msg1["seq"], 1);
    assert_eq!(msg2["seq"], 2);
    assert_eq!(msg3["seq"], 3);
}
