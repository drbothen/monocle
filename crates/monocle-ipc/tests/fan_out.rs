//! Fan-out broadcast tests for `monocle-ipc` (BC-2.05.003, BC-2.05.004).
//!
//! # Coverage after F-ADV2-MED-002
//!
//! `UdsTransport::broadcast_session_list_update` and
//! `UdsTransport::broadcast_hook_event_received` were dead code — the production
//! lifecycle path used `monocle_runtime::ipc_server::broadcast_to_subscribers`
//! directly, not the `UdsTransport` methods. Those methods have been deleted
//! (F-ADV2-MED-002). Tests that exercised them have been removed from this file.
//!
//! Retained: pure-function tests for `truncate_to_utf8_boundary` (no `UdsTransport`
//! dependency; these remain green and should stay in this suite).
//!
//! Fan-out broadcast behavior (slow client removal, 256 KiB guard, etc.) is now
//! covered by integration tests in `monocle-runtime/tests/` that exercise
//! `broadcast_to_subscribers` through the actual lifecycle path.
// Test function names follow the VSDD `test_BC_S_SS_NNN_xxx` convention (uppercase BC).
// Suppress non_snake_case lint for test files — BC_ prefix is intentional for traceability.
#![allow(non_snake_case)]
// expect/unwrap are idiomatic in test code for assertion amplification.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use monocle_ipc::types::{truncate_to_utf8_boundary, PAYLOAD_EXCERPT_MAX_BYTES};

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
