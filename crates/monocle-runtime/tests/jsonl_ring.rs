//! Integration tests for BC-2.01.007 — JSONL Ring Format Version (FC-01).
//!
//! Tests are named per the test-writer naming convention:
//!   test_BC_RING_001_<assertion>()
//!
//! Red Gate status:
//! - Tests that exercise only HookEventRecord::new() + serde: PASS on stubs (expected).
//! - Tests that call RingBuffer::push() or rotate_if_needed(): FAIL on todo!() stubs (required).

// BC-based test names require uppercase segments (e.g., `test_BC_RING_001_xxx`).
// Suppress the non_snake_case lint for this file — canonical project pattern per enum_audit.rs.
// Suppress unwrap/expect lints in tests — canonical project pattern (matches all other test files).
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use monocle_runtime::ring::{
    HookEventRecord, RingBuffer, RingError, RotationConfig, RING_FORMAT_VERSION,
};
use std::path::PathBuf;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal SessionStart record with no tool context.
fn session_start_record() -> HookEventRecord {
    HookEventRecord::new(
        "test-session-id-001".to_string(),
        1_700_000_000_000_000_i64,
        12345_u32,
        "SessionStart".to_string(),
        None,
        None,
    )
}

/// Build a PreToolUse record with both tool fields present.
fn pre_tool_use_record() -> HookEventRecord {
    HookEventRecord::new(
        "test-session-id-002".to_string(),
        1_700_000_001_000_000_i64,
        12345_u32,
        "PreToolUse".to_string(),
        Some("Bash".to_string()),
        Some(serde_json::json!({"command": "ls -la", "timeout": 30})),
    )
}

/// Canonical ring file path for a given runtime dir.
fn ring_path(dir: &TempDir) -> PathBuf {
    dir.path().join("monocle-events.jsonl")
}

// ---------------------------------------------------------------------------
// AC-001: format_version is always the first JSON key
// ---------------------------------------------------------------------------

/// Exercises VP-007: format_version must be the first serialized key in every HookEventRecord.
///
/// The verbatim oracle from AC-001 is reproduced here exactly.
#[test]
fn test_BC_RING_001_format_version_first_key() {
    let record = session_start_record();
    // AC-001 verbatim oracle — do NOT change this assertion.
    assert!(
        serde_json::to_string(&record)
            .unwrap()
            .starts_with(r#"{"format_version":1,"#),
        "format_version MUST be the first JSON key (BC-2.01.007 PC-1, FC-01)"
    );
}

// ---------------------------------------------------------------------------
// AC-002 (negative path): absent tool fields must not appear as null
// ---------------------------------------------------------------------------

/// VP-007: SessionStart records must omit tool_name and tool_input entirely — no explicit null.
///
/// BC-2.01.007 EC-001: "Phase 1 emitters MUST emit absence (no explicit null)."
#[test]
fn test_BC_RING_001_absent_tool_fields_not_null() {
    let record = session_start_record();
    let json = serde_json::to_string(&record).unwrap();

    assert!(
        !json.contains("tool_name"),
        "tool_name must be ABSENT from SessionStart JSON, got: {json}"
    );
    assert!(
        !json.contains("tool_input"),
        "tool_input must be ABSENT from SessionStart JSON, got: {json}"
    );
}

// ---------------------------------------------------------------------------
// AC-002 (positive path): present tool fields must appear in the JSON
// ---------------------------------------------------------------------------

/// Both tool_name and tool_input must be present when Some (BC-2.01.007 PC-4).
#[test]
fn test_BC_RING_001_present_tool_fields() {
    let record = pre_tool_use_record();
    let json = serde_json::to_string(&record).unwrap();

    assert!(
        json.contains("\"tool_name\""),
        "tool_name must be present for PreToolUse, got: {json}"
    );
    assert!(
        json.contains("\"tool_input\""),
        "tool_input must be present for PreToolUse, got: {json}"
    );
    assert!(
        json.contains("\"Bash\""),
        "tool_name value 'Bash' must appear in JSON, got: {json}"
    );
}

// ---------------------------------------------------------------------------
// AC-002b: 7-field canonical declaration order
// ---------------------------------------------------------------------------

/// BC-2.01.007 PC-4 + SS-core-types-and-abi.md v1.2.13 §HookEventRecord:
/// All 7 canonical fields must be present and the struct must serialize them
/// in declaration order (format_version first).
#[test]
fn test_BC_RING_001_7_field_declaration_order() {
    let record = pre_tool_use_record();
    let json_str = serde_json::to_string(&record).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let obj = value.as_object().expect("record must serialize as a JSON object");

    // All 7 canonical keys must be present.
    let required_keys = [
        "format_version",
        "session_id",
        "timestamp_micros",
        "pid",
        "hook_type",
        "tool_name",
        "tool_input",
    ];
    for key in &required_keys {
        assert!(
            obj.contains_key(*key),
            "canonical key '{key}' missing from JSON object; got keys: {:?}",
            obj.keys().collect::<Vec<_>>()
        );
    }

    // format_version must still be first key in the serialized string.
    assert!(
        json_str.starts_with(r#"{"format_version":1,"#),
        "format_version must be first key even with all 7 fields present"
    );

    // timestamp_micros is i64 (signed) per SS-core-types-and-abi.md v1.2.13.
    // Verify it round-trips as i64.
    let ts = obj["timestamp_micros"]
        .as_i64()
        .expect("timestamp_micros must deserialize as i64");
    assert_eq!(ts, 1_700_000_001_000_000_i64);
}

// ---------------------------------------------------------------------------
// AC-006: constructor is the only legal construction path
// ---------------------------------------------------------------------------

/// BC-2.01.007 PC-5: HookEventRecord::new() sets format_version = RING_FORMAT_VERSION.
/// #[non_exhaustive] enforces constructor-only access outside the crate (Rust E0639).
///
/// This test passes even with stubs because new() is implemented in the stub.
#[test]
fn test_BC_RING_001_non_exhaustive_constructor_only() {
    let record = HookEventRecord::new(
        "constructor-test-session".to_string(),
        42_i64,
        99_u32,
        "Stop".to_string(),
        None,
        None,
    );

    assert_eq!(
        record.format_version, RING_FORMAT_VERSION,
        "constructor must set format_version = RING_FORMAT_VERSION ({RING_FORMAT_VERSION})"
    );
    assert_eq!(RING_FORMAT_VERSION, 1_u32, "RING_FORMAT_VERSION must be 1");
    assert_eq!(record.session_id, "constructor-test-session");
    assert_eq!(record.timestamp_micros, 42_i64);
    assert_eq!(record.pid, 99_u32);
    assert_eq!(record.hook_type, "Stop");
    assert!(record.tool_name.is_none());
    assert!(record.tool_input.is_none());
}

// ---------------------------------------------------------------------------
// AC-003 + AC-004: push() writes a JSONL line atomically
// ---------------------------------------------------------------------------

/// BC-2.01.007 PC-2/PC-3 + SS-daemon-lifecycle.md L694:
/// push() must write a newline-terminated JSONL record to the ring file.
///
/// EXPECTED TO FAIL (Red Gate): push() body is todo!().
#[test]
fn test_BC_RING_001_push_writes_jsonl_line() {
    let dir = TempDir::new().expect("create tempdir");
    let path = ring_path(&dir);
    let config = RotationConfig::default();
    let ring = RingBuffer::new(path.clone(), config);

    let record = session_start_record();
    ring.push(&record).expect("push must succeed");

    let contents = std::fs::read_to_string(&path).expect("ring file must exist after push");
    // Must be non-empty and end with a newline (JSONL format).
    assert!(!contents.is_empty(), "ring file must not be empty after push");
    assert!(
        contents.ends_with('\n'),
        "JSONL line must end with newline"
    );

    // The written line must be valid JSON containing the record.
    let line = contents.trim_end_matches('\n');
    let value: serde_json::Value =
        serde_json::from_str(line).expect("written line must be valid JSON");
    assert_eq!(
        value["format_version"], 1,
        "written record must have format_version = 1"
    );
    assert_eq!(
        value["hook_type"], "SessionStart",
        "written record must preserve hook_type"
    );
    // Verify format_version is first key in the written line.
    assert!(
        line.starts_with(r#"{"format_version":1,"#),
        "written JSONL line must have format_version as first key"
    );
}

// ---------------------------------------------------------------------------
// AC-007: rotation at threshold
// ---------------------------------------------------------------------------

/// SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer Rotation Policy L675-719 + BC-2.01.007 EC-002:
/// Ring file is rotated when it exceeds the soft threshold.
/// Cascade: monocle-events.jsonl → monocle-events.jsonl.1, etc.
///
/// EXPECTED TO FAIL (Red Gate): push() body is todo!().
#[test]
fn test_BC_RING_001_rotation_at_threshold() {
    let dir = TempDir::new().expect("create tempdir");
    let path = ring_path(&dir);

    // Tiny thresholds so rotation triggers quickly.
    // AC-007 testability hook from story spec.
    let config = RotationConfig {
        soft_threshold_bytes: 100,
        hard_cap_bytes: 400,
        retained: 3,
    };
    let ring = RingBuffer::new(path.clone(), config);

    // Push enough records to exceed the 100-byte soft threshold.
    // A single serialized HookEventRecord with a long session_id will easily exceed 100 bytes.
    let large_record = HookEventRecord::new(
        "a".repeat(200),
        1_700_000_000_000_000_i64,
        12345_u32,
        "PreToolUse".to_string(),
        Some("Bash".to_string()),
        Some(serde_json::json!({"command": "ls"})),
    );

    // Push twice to ensure rotation is triggered (first push fills the file,
    // second push should trigger rotation check).
    ring.push(&large_record).expect("first push");
    ring.push(&large_record).expect("second push — should trigger rotation");

    // After rotation, the .1 rotated file must exist.
    let rotated = dir.path().join("monocle-events.jsonl.1");
    assert!(
        rotated.exists(),
        "rotation file monocle-events.jsonl.1 must exist after threshold exceeded"
    );
}

// ---------------------------------------------------------------------------
// AC-005: flush failure returns Err, does not panic
// ---------------------------------------------------------------------------

/// BC-2.01.004 EC-049 + BC-2.01.007 AC-005:
/// I/O failure during push must return Err(RingError::Io) and NOT panic.
/// The daemon must continue accepting events after a flush failure.
///
/// EXPECTED TO FAIL (Red Gate): push() body is todo!() which panics rather than returning Err.
#[test]
fn test_BC_RING_001_flush_failure_degraded_not_broken() {
    // Point the ring at a non-existent directory to force I/O failure.
    let path = PathBuf::from("/nonexistent-dir-s008-test/monocle-events.jsonl");
    let config = RotationConfig::default();
    let ring = RingBuffer::new(path, config);

    let record = session_start_record();
    let result = ring.push(&record);

    assert!(
        result.is_err(),
        "push to non-existent directory must return Err, not panic"
    );
    match result.unwrap_err() {
        RingError::Io(_) => { /* correct error variant */ }
        other => panic!("expected RingError::Io, got: {other:?}"),
    }

    // Calling push again after a failure must not panic (degraded, not broken).
    let second_result = ring.push(&record);
    assert!(
        second_result.is_err(),
        "second push after failure must still return Err, not panic"
    );
}
