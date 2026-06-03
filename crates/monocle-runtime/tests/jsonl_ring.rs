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

    let obj = value
        .as_object()
        .expect("record must serialize as a JSON object");

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
    assert!(
        !contents.is_empty(),
        "ring file must not be empty after push"
    );
    assert!(contents.ends_with('\n'), "JSONL line must end with newline");

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
    ring.push(&large_record)
        .expect("second push — should trigger rotation");

    // After rotation, the .1 rotated file must exist.
    let rotated = dir.path().join("monocle-events.jsonl.1");
    assert!(
        rotated.exists(),
        "rotation file monocle-events.jsonl.1 must exist after threshold exceeded"
    );
}

// ---------------------------------------------------------------------------
// F-006: round-trip deserialization (VP-007 probe 7.c)
// ---------------------------------------------------------------------------

/// VP-007 probe 7.c: a serialized HookEventRecord must deserialize back to an
/// identical record with all fields intact.
#[test]
fn test_BC_RING_001_roundtrip_deserialization() {
    let record = HookEventRecord::new(
        "sess1".to_string(),
        1000_i64,
        42_u32,
        "SessionStart".to_string(),
        None,
        None,
    );
    let json = serde_json::to_string(&record).unwrap();
    let roundtrip: HookEventRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip.format_version, RING_FORMAT_VERSION);
    assert_eq!(roundtrip.session_id, "sess1");
    assert_eq!(roundtrip.timestamp_micros, 1000_i64);
    assert_eq!(roundtrip.pid, 42_u32);
    assert_eq!(roundtrip.hook_type, "SessionStart");
    assert!(
        roundtrip.tool_name.is_none(),
        "tool_name must remain None after roundtrip"
    );
    assert!(
        roundtrip.tool_input.is_none(),
        "tool_input must remain None after roundtrip"
    );
}

// ---------------------------------------------------------------------------
// F-007: UserPromptSubmit and Stop absence-of-field tests (VP-007 probes 7.e, 7.f)
// ---------------------------------------------------------------------------

/// VP-007 probe 7.e: UserPromptSubmit records must omit tool_name and tool_input entirely.
///
/// BC-2.01.007 EC-001: "Phase 1 emitters MUST emit absence (no explicit null)."
#[test]
fn test_BC_RING_001_user_prompt_submit_absent_tool_fields() {
    let record = HookEventRecord::new(
        "sess-ups-001".to_string(),
        1_700_000_002_000_000_i64,
        99_u32,
        "UserPromptSubmit".to_string(),
        None,
        None,
    );
    let json = serde_json::to_string(&record).unwrap();

    assert!(
        !json.contains("tool_name"),
        "tool_name must be ABSENT (not null) from UserPromptSubmit JSON, got: {json}"
    );
    assert!(
        !json.contains("tool_input"),
        "tool_input must be ABSENT (not null) from UserPromptSubmit JSON, got: {json}"
    );
}

/// VP-007 probe 7.f: Stop records must omit tool_name and tool_input entirely.
///
/// BC-2.01.007 EC-001: "Phase 1 emitters MUST emit absence (no explicit null)."
#[test]
fn test_BC_RING_001_stop_absent_tool_fields() {
    let record = HookEventRecord::new(
        "sess-stop-001".to_string(),
        1_700_000_003_000_000_i64,
        99_u32,
        "Stop".to_string(),
        None,
        None,
    );
    let json = serde_json::to_string(&record).unwrap();

    assert!(
        !json.contains("tool_name"),
        "tool_name must be ABSENT (not null) from Stop JSON, got: {json}"
    );
    assert!(
        !json.contains("tool_input"),
        "tool_input must be ABSENT (not null) from Stop JSON, got: {json}"
    );
}

// ---------------------------------------------------------------------------
// F-008: rotation cascade coverage
// ---------------------------------------------------------------------------

/// BC-2.01.007 EC-002 / SS-daemon-lifecycle v1.0.33 §JSONL Ring Buffer Rotation Policy:
/// Multiple sequential pushes that each exceed the soft threshold must produce a full
/// `.1`...`.{retained}` cascade and must NOT produce a `.{retained+1}` file.
///
/// retained=3 → files .1, .2, .3 must exist after 4+ pushes; .4 must NOT exist.
#[test]
fn test_BC_RING_001_rotation_cascade_multiple() {
    let dir = TempDir::new().expect("create tempdir");
    let path = ring_path(&dir);

    // Tiny thresholds: each record (~80 bytes) exceeds the soft threshold so
    // every push causes a rotation of whatever is already on disk.
    let config = RotationConfig {
        soft_threshold_bytes: 50,
        hard_cap_bytes: 200,
        retained: 3,
    };
    let ring = RingBuffer::new(path.clone(), config);

    // A record that serialises to > 50 bytes when written.
    let record = HookEventRecord::new(
        "cascade-session-abc123".to_string(),
        1_700_000_000_000_001_i64,
        1111_u32,
        "PreToolUse".to_string(),
        Some("Bash".to_string()),
        Some(serde_json::json!({"command": "echo hello"})),
    );

    // Push 4 records.  Rotation check fires on each push after the first write,
    // so after 4 pushes we expect the .1, .2, .3 slots all occupied.
    for i in 0_u32..4 {
        ring.push(&record)
            .unwrap_or_else(|e| panic!("push {i} failed: {e}"));
    }

    let rotated_1 = dir.path().join("monocle-events.jsonl.1");
    let rotated_2 = dir.path().join("monocle-events.jsonl.2");
    let rotated_3 = dir.path().join("monocle-events.jsonl.3");
    let rotated_4 = dir.path().join("monocle-events.jsonl.4");

    assert!(
        rotated_1.exists(),
        "monocle-events.jsonl.1 must exist after multiple rotations"
    );
    assert!(
        rotated_2.exists(),
        "monocle-events.jsonl.2 must exist after multiple rotations"
    );
    assert!(
        rotated_3.exists(),
        "monocle-events.jsonl.3 must exist after multiple rotations (retained=3)"
    );
    assert!(
        !rotated_4.exists(),
        "monocle-events.jsonl.4 must NOT exist — retained=3 caps at .3"
    );

    // The oldest retained file (.1 or .3, depending on cascade direction) must
    // contain valid JSONL data.
    let contents = std::fs::read_to_string(&rotated_1).expect("rotated .1 must be readable");
    let first_line = contents
        .lines()
        .next()
        .expect("rotated .1 must be non-empty");
    let _parsed: serde_json::Value =
        serde_json::from_str(first_line).expect("rotated .1 must contain valid JSONL");
}

/// BC-2.01.007 EC-002 / SS-daemon-lifecycle §JSONL Ring Buffer Rotation Policy:
/// With retained=2, three or more rotations must delete the oldest segment so
/// that `.3` is absent while `.1` and `.2` are present.
#[test]
fn test_BC_RING_001_rotation_deletes_oldest() {
    let dir = TempDir::new().expect("create tempdir");
    let path = ring_path(&dir);

    let config = RotationConfig {
        soft_threshold_bytes: 50,
        hard_cap_bytes: 200,
        retained: 2,
    };
    let ring = RingBuffer::new(path.clone(), config);

    let record = HookEventRecord::new(
        "delete-oldest-session-xyz987".to_string(),
        1_700_000_000_000_002_i64,
        2222_u32,
        "PostToolUse".to_string(),
        Some("Read".to_string()),
        Some(serde_json::json!({"path": "/tmp/test.txt"})),
    );

    // Push 4 records to trigger 3+ rotations with retained=2.
    for i in 0_u32..4 {
        ring.push(&record)
            .unwrap_or_else(|e| panic!("push {i} failed: {e}"));
    }

    let rotated_1 = dir.path().join("monocle-events.jsonl.1");
    let rotated_2 = dir.path().join("monocle-events.jsonl.2");
    let rotated_3 = dir.path().join("monocle-events.jsonl.3");

    assert!(
        rotated_1.exists(),
        "monocle-events.jsonl.1 must exist (retained=2)"
    );
    assert!(
        rotated_2.exists(),
        "monocle-events.jsonl.2 must exist (retained=2)"
    );
    assert!(
        !rotated_3.exists(),
        "monocle-events.jsonl.3 must NOT exist — oldest deleted at retained=2"
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

// ---------------------------------------------------------------------------
// MED-002: post-shutdown enqueue guard — begin_shutdown() → append() returns Err
// ---------------------------------------------------------------------------

/// MED-002: once `begin_shutdown()` is called, `append()` must return
/// `Err(RingError::Shutdown)` immediately without enqueueing the record.
///
/// This guards the force-close post-timeout window where a still-live handler holding
/// its own `Arc<RingBuffer>` clone could silently enqueue a record after the flush task
/// has drained and exited (E-RING-007 taxonomy).
///
/// The test also verifies:
/// - `append()` succeeds BEFORE `begin_shutdown()` (pre-condition: ring is live).
/// - After `begin_shutdown()`, append returns `Err(RingError::Shutdown)`, not a panic.
/// - The RAM ring is NOT updated on rejected appends (record count does not increase).
#[test]
fn test_ring_begin_shutdown_rejects_append() {
    let dir = TempDir::new().expect("create tempdir");
    let path = ring_path(&dir);
    let config = RotationConfig::default();
    let ring = RingBuffer::new(path, config);

    // Pre-condition: append succeeds before shutdown.
    let record = session_start_record();
    ring.append(record.clone())
        .expect("append must succeed before begin_shutdown");

    // Verify the record landed in the RAM ring (latest_events returns 1 item).
    let events_before = ring.latest_events(10);
    assert_eq!(
        events_before.len(),
        1,
        "RAM ring must have 1 event after successful append"
    );

    // Signal shutdown.
    ring.begin_shutdown();

    // Post-condition: append must fail with RingError::Shutdown.
    let result = ring.append(record.clone());
    assert!(
        result.is_err(),
        "append after begin_shutdown must return Err, not Ok"
    );
    match result.unwrap_err() {
        RingError::Shutdown => { /* correct variant — E-RING-007 */ }
        other => panic!("expected RingError::Shutdown, got: {other:?}"),
    }

    // Verify the RAM ring was NOT updated on the rejected append (still 1 event).
    let events_after = ring.latest_events(10);
    assert_eq!(
        events_after.len(),
        1,
        "RAM ring must NOT be updated after a post-shutdown rejected append"
    );

    // Second rejected append must also return Shutdown (idempotent flag).
    let result2 = ring.append(record);
    assert!(
        matches!(result2.unwrap_err(), RingError::Shutdown),
        "subsequent appends after begin_shutdown must all return RingError::Shutdown"
    );
}
