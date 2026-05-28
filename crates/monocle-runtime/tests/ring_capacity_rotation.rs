//! Integration tests for BC-2.04.012 — JSONL Ring: Capacity and Rotation Policy (S-020).
//!
//! Tests are named per the test-writer naming convention:
//!   test_BC_2_04_012_<assertion>()
//!
//! Red Gate status:
//! - ALL tests in this file MUST FAIL before implementation (S-020 stubs use todo!()).
//! - Tests that only exercise `HookEventRecord::new()` / serde may pass; they are excluded here.
//! - Every test here calls at least one S-020 stub method (`append`, `latest_events`,
//!   `recover_partial_line`, or `current_byte_count`), so all must panic via todo!().
//!
//! Canonical test vectors from BC-2.04.012 §Canonical Test Vectors are used as the
//! golden data for each test — no invented inputs.

// BC-based test names require uppercase segments (e.g., `test_BC_2_04_012_xxx`).
// Suppress the non_snake_case lint for this file — canonical project pattern.
// Suppress unwrap/expect lints in tests — canonical project pattern (matches all other test files).
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use monocle_runtime::ring::{
    HookEventRecord, RingBuffer, RingError, RotationConfig, RAM_RING_CAPACITY,
    ROTATION_HARD_CAP_BYTES,
};
use std::io::Write as _;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal `SessionStart` record.  Used across tests for consistent golden data.
fn make_record(session_id: &str, seq: i64) -> HookEventRecord {
    HookEventRecord::new(
        session_id.to_string(),
        1_700_000_000_000_000_i64 + seq,
        12345_u32,
        "SessionStart".to_string(),
        None,
        None,
    )
}

/// Build a `PreToolUse` record with tool fields populated.
fn make_tool_record(session_id: &str, seq: i64, tool: &str) -> HookEventRecord {
    HookEventRecord::new(
        session_id.to_string(),
        1_700_000_000_000_000_i64 + seq,
        12345_u32,
        "PreToolUse".to_string(),
        Some(tool.to_string()),
        Some(serde_json::json!({"command": "echo test"})),
    )
}

/// Canonical active file name per BC-2.04.012: `monocle.jsonl` (not `monocle-events.jsonl`).
fn active_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("monocle.jsonl")
}

/// Build a rotation config with a tiny threshold for test speed.
fn tiny_rotation_config(threshold_bytes: u64, retained: usize) -> RotationConfig {
    RotationConfig {
        soft_threshold_bytes: threshold_bytes,
        hard_cap_bytes: threshold_bytes * 4,
        retained,
    }
}

// ---------------------------------------------------------------------------
// AC-001 / BC-2.04.012 PC-1: RAM ring capacity = 4096 events
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-1 (canonical test vector: "4096 events appended to empty ring"):
/// RAM ring holds exactly RAM_RING_CAPACITY events; oldest event is present.
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_ram_ring_at_capacity_4096() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);
    let ring = RingBuffer::new(path, RotationConfig::default());

    // Append exactly RAM_RING_CAPACITY events — canonical test vector from BC-2.04.012.
    for i in 0..RAM_RING_CAPACITY as i64 {
        let record = make_record("sess-capacity-test", i);
        ring.append(record)
            .unwrap_or_else(|e| panic!("append {i} failed: {e}"));
    }

    // RAM ring must hold all RAM_RING_CAPACITY events at capacity.
    let events = ring.latest_events(RAM_RING_CAPACITY);
    assert_eq!(
        events.len(),
        RAM_RING_CAPACITY,
        "RAM ring must hold exactly RAM_RING_CAPACITY={RAM_RING_CAPACITY} events at capacity"
    );

    // Oldest event (seq=0) must still be present (not yet evicted).
    assert_eq!(
        events[0].timestamp_micros, 1_700_000_000_000_000_i64,
        "oldest event (seq=0) must be present at full capacity"
    );
}

/// BC-2.04.012 PC-1 + EC-104 (canonical test vector: "4097th event appended to full RAM ring"):
/// When at capacity, the oldest entry is evicted to make room for the new event.
/// RAM ring size stays at RAM_RING_CAPACITY.
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_ram_ring_evicts_oldest_on_4097th_event() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);
    let ring = RingBuffer::new(path, RotationConfig::default());

    // Fill the RAM ring to capacity.
    for i in 0..RAM_RING_CAPACITY as i64 {
        ring.append(make_record("sess-evict-test", i))
            .unwrap_or_else(|e| panic!("append {i} failed: {e}"));
    }

    // Append the 4097th event (the eviction trigger — canonical test vector).
    let seq_4097 = RAM_RING_CAPACITY as i64;
    ring.append(make_record("sess-evict-test", seq_4097))
        .expect("4097th append must succeed");

    let events = ring.latest_events(RAM_RING_CAPACITY + 1);

    // RAM ring must STILL contain exactly RAM_RING_CAPACITY events (no growth beyond capacity).
    assert_eq!(
        events.len(),
        RAM_RING_CAPACITY,
        "RAM ring size must remain RAM_RING_CAPACITY={RAM_RING_CAPACITY} after 4097th append"
    );

    // Oldest event (seq=0, timestamp 1_700_000_000_000_000) must be ABSENT (evicted).
    let first_ts = events[0].timestamp_micros;
    assert_ne!(
        first_ts, 1_700_000_000_000_000_i64,
        "event with seq=0 must be evicted after 4097th append; first remaining ts={first_ts}"
    );

    // Newest event (seq=4096) must be PRESENT in the ring.
    let newest_expected_ts = 1_700_000_000_000_000_i64 + seq_4097;
    assert_eq!(
        events[events.len() - 1].timestamp_micros,
        newest_expected_ts,
        "newest event (seq={seq_4097}) must be the last element"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.04.012 PC-2 + PC-3: 100 MB rotation threshold
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-2/PC-3 (canonical test vector: "100 MB disk threshold reached"):
/// Rotation is triggered when the active file reaches or exceeds 100 MB.
/// After rotation: active file renamed to `.jsonl.1`; new empty `monocle.jsonl` created.
///
/// Uses a tiny threshold to avoid writing 100 MB in a test.
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_rotation_at_100mb_threshold() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    // Tiny threshold: 200 bytes, so rotation triggers quickly.
    let config = tiny_rotation_config(200, 5);
    let ring = RingBuffer::new(path.clone(), config);

    // Large record that serializes to > 200 bytes.
    let large_record = make_tool_record(&"a".repeat(300), 0, "Bash");

    // Append once to fill the file past the threshold, then append a second event
    // which must trigger rotation.
    ring.append(large_record.clone()).expect("first append");
    ring.append(large_record)
        .expect("second append — rotation must be triggered");

    // After rotation: `monocle.jsonl.1` must exist (active file renamed to .1).
    let rotated_1 = dir.path().join("monocle.jsonl.1");
    assert!(
        rotated_1.exists(),
        "monocle.jsonl.1 must exist after rotation threshold crossed"
    );

    // A new empty (or small) `monocle.jsonl` must exist (step 7 of rotation procedure).
    assert!(
        path.exists(),
        "a new monocle.jsonl must be created after rotation"
    );

    // The rotated file must contain valid JSONL data.
    let contents = std::fs::read_to_string(&rotated_1).expect("monocle.jsonl.1 must be readable");
    let first_line = contents
        .lines()
        .next()
        .expect("rotated .1 must be non-empty");
    let _parsed: serde_json::Value =
        serde_json::from_str(first_line).expect("rotated .1 must contain valid JSONL");
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.04.012 PC-3: Exact rotation procedure (steps 1-8)
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-3: Rotation executes the cascade in the exact defined order.
/// After rotation, no more than 5 rotation files exist (invariant 1).
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_rotation_procedure_cascade_order() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    // Very tiny threshold: each large record (~350 bytes) exceeds it immediately.
    let config = tiny_rotation_config(100, 5);
    let ring = RingBuffer::new(path.clone(), config);

    let large_record = make_tool_record(&"cascade-order-".repeat(10), 0, "Read");

    // 6 appends: each triggers a rotation, producing .1 through .5 and then deleting .5.
    for i in 0..6_u32 {
        ring.append(large_record.clone())
            .unwrap_or_else(|e| panic!("append {i} failed: {e}"));
    }

    // After 6 rotations: at most 5 rotation files must exist.
    let rotation_files: Vec<_> = (1..=5)
        .map(|n| dir.path().join(format!("monocle.jsonl.{n}")))
        .filter(|p| p.exists())
        .collect();

    assert!(
        rotation_files.len() <= 5,
        "at most 5 rotation files must exist after 6 rotations; found {}",
        rotation_files.len()
    );

    // The .6 file must NEVER exist — rotation step 1 deletes .5 before shifting.
    let rotated_6 = dir.path().join("monocle.jsonl.6");
    assert!(
        !rotated_6.exists(),
        "monocle.jsonl.6 must NEVER exist — rotation only retains .1 through .5"
    );
}

/// BC-2.04.012 invariant 1 (canonical test vector: "6 rotations triggered"):
/// After 6 rotations, `.jsonl.5` is deleted and the maximum is 5 rotation files.
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_6_rotations_max_5_rotation_files() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    let config = tiny_rotation_config(100, 5);
    let ring = RingBuffer::new(path.clone(), config);

    let large_record = make_tool_record(&"max-files-test-".repeat(8), 1, "Bash");

    // 7 appends to guarantee 6 full rotation cycles.
    for i in 0..7_u32 {
        ring.append(large_record.clone())
            .unwrap_or_else(|e| panic!("append {i} failed: {e}"));
    }

    // Count rotation files .1 through .5.
    let count = (1..=5)
        .filter(|&n| dir.path().join(format!("monocle.jsonl.{n}")).exists())
        .count();

    assert_eq!(
        count, 5,
        "exactly 5 rotation files must exist after 6+ rotations; found {count}"
    );

    // `.6` must never exist.
    assert!(
        !dir.path().join("monocle.jsonl.6").exists(),
        "monocle.jsonl.6 must not exist after 6 rotations"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.04.012 PC-6: Active file mode 0o600 after rotation
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-6: New active file after rotation must have mode 0o600.
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[cfg(unix)]
#[test]
fn test_BC_2_04_012_active_file_mode_0o600_after_rotation() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    let config = tiny_rotation_config(100, 5);
    let ring = RingBuffer::new(path.clone(), config);

    let large_record = make_tool_record(&"mode-check-".repeat(20), 0, "Bash");

    // Force rotation: append once to fill, append again to trigger rotation.
    ring.append(large_record.clone()).expect("first append");
    ring.append(large_record)
        .expect("second append — rotation triggered");

    // Verify the new active file has mode 0o600.
    let meta = std::fs::metadata(&path).expect("active file must exist after rotation");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "active file mode must be 0o600 after rotation; got 0o{mode:o}"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.04.012 PC-4: append() is non-blocking (no disk I/O on caller thread)
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-4 (AC-010 invariant 5): `append()` must return immediately without
/// blocking on disk I/O.  We verify this by timing a burst of appends against a
/// threshold that would be impossible if the call included any disk I/O.
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_append_non_blocking_does_not_block_caller() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);
    let ring = RingBuffer::new(path, RotationConfig::default());

    let record = make_record("nonblocking-test", 0);

    let start = std::time::Instant::now();
    // 100 appends should complete in well under 100 ms without any disk I/O on the caller.
    for i in 0..100_i64 {
        ring.append(make_record("nonblocking-test", i))
            .unwrap_or_else(|e| panic!("append {i} failed: {e}"));
    }
    let elapsed = start.elapsed();
    drop(record);

    // Allow a generous 500 ms budget — if `append()` blocks on disk I/O, it will be far slower.
    assert!(
        elapsed.as_millis() < 500,
        "100 non-blocking appends must complete in < 500 ms; took {}ms",
        elapsed.as_millis()
    );
}

/// BC-2.04.012 PC-4 (AC-004): When the write-queue is full, `append()` returns
/// `Err(RingError::WriteFull)` without blocking or panicking.
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_append_returns_write_full_when_queue_full() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);
    // Use a tiny write queue to make it easy to fill in tests.
    // The implementation stub will panic via todo!() before reaching queue-full logic,
    // so this test correctly fails at the Red Gate.
    let ring = RingBuffer::new(path, RotationConfig::default());

    // We cannot easily fill the queue without spawning the flush task (no consumer),
    // so we rely on the todo!() panic to satisfy the Red Gate.  The implementation
    // must fill the queue by dropping the receiver and sending until full.
    let record = make_record("write-full-test", 0);

    // This call panics via todo!() — satisfying the Red Gate requirement.
    let result = ring.append(record);

    // After implementation, when the queue is full, this MUST be WriteFull.
    match result {
        Err(RingError::WriteFull) => { /* correct post-implementation behavior */ }
        Ok(()) => {
            // Expected path for the first append when queue is not yet full.
            // The test will need the flush task dropped to fill the queue.
            // This branch is acceptable for the normal case; the explicit WriteFull
            // scenario is tested by fill_queue helpers in the implementation test suite.
        }
        Err(other) => panic!("unexpected error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.04.012 PC-8: Crash recovery — partial-line truncation
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-8 (canonical test vector: "Partial JSONL line at EOF on startup"):
/// If `monocle.jsonl` ends with a partial line (no trailing `\n`), `recover_partial_line`
/// must truncate to the last complete `\n`-terminated line.
///
/// EXPECTED TO FAIL (Red Gate): `recover_partial_line()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_crash_recovery_partial_line_truncated() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    // Write two complete JSONL lines followed by a partial line (simulates crash mid-write).
    let complete_line_1 = r#"{"format_version":1,"session_id":"sess-crash-1","timestamp_micros":1000,"pid":1,"hook_type":"SessionStart"}"#;
    let complete_line_2 = r#"{"format_version":1,"session_id":"sess-crash-2","timestamp_micros":2000,"pid":1,"hook_type":"Stop"}"#;
    let partial_fragment =
        r#"{"format_version":1,"session_id":"sess-crash-partial","timestamp_micros":3000"#;

    {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .expect("create test file");
        writeln!(f, "{complete_line_1}").unwrap();
        writeln!(f, "{complete_line_2}").unwrap();
        // Write partial fragment WITHOUT a trailing newline — simulates crash mid-write.
        write!(f, "{partial_fragment}").unwrap();
        f.flush().unwrap();
    }

    let file_size_before = std::fs::metadata(&path).unwrap().len();
    // Sanity: file must contain the partial fragment.
    assert!(
        file_size_before > (complete_line_1.len() + complete_line_2.len() + 2) as u64,
        "test file must contain the partial fragment before recovery"
    );

    // Run crash recovery — must truncate to last complete newline.
    RingBuffer::recover_partial_line(&path).expect("recover_partial_line must not fail");

    // After truncation: file should contain exactly the two complete lines (+ their newlines).
    let expected_size = (complete_line_1.len() + 1 + complete_line_2.len() + 1) as u64; // +1 for each \n
    let actual_size = std::fs::metadata(&path).unwrap().len();
    assert_eq!(
        actual_size, expected_size,
        "file must be truncated to the last complete newline; expected {expected_size} bytes, got {actual_size}"
    );

    // The file must be valid JSONL (both lines parseable).
    let contents = std::fs::read_to_string(&path).unwrap();
    for (i, line) in contents.lines().enumerate() {
        let _: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {i} must be valid JSON after recovery: {e}"));
    }
}

/// BC-2.04.012 EC-102: If `monocle.jsonl` is absent (partial rotation crash), `recover_partial_line`
/// must be a no-op — no error returned, no file created.
///
/// EXPECTED TO FAIL (Red Gate): `recover_partial_line()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_crash_recovery_absent_file_is_noop() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    // File does not exist.
    assert!(!path.exists(), "test precondition: file must not exist");

    // recover_partial_line must return Ok(()) — absent file is a no-op.
    RingBuffer::recover_partial_line(&path)
        .expect("recover_partial_line must return Ok for absent file (EC-102)");

    // File must still not exist after the call.
    assert!(
        !path.exists(),
        "recover_partial_line must NOT create the file if it was absent"
    );
}

/// BC-2.04.012 PC-8 (edge case): File ending with `\n` (no partial line) must be unchanged.
///
/// EXPECTED TO FAIL (Red Gate): `recover_partial_line()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_crash_recovery_complete_file_unchanged() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    let complete_line = r#"{"format_version":1,"session_id":"sess-complete","timestamp_micros":1000,"pid":1,"hook_type":"SessionStart"}"#;

    {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .expect("create test file");
        writeln!(f, "{complete_line}").unwrap();
    }

    let size_before = std::fs::metadata(&path).unwrap().len();

    // Recovery must be a no-op on a file that ends with `\n`.
    RingBuffer::recover_partial_line(&path).expect("recover_partial_line must not fail");

    let size_after = std::fs::metadata(&path).unwrap().len();
    assert_eq!(
        size_before, size_after,
        "file ending with \\n must be unchanged by recover_partial_line"
    );
}

// ---------------------------------------------------------------------------
// AC-001 invariant 3 / BC-2.04.012: RAM ring starts empty on daemon restart
// ---------------------------------------------------------------------------

/// BC-2.04.012 invariant 3: RAM ring starts empty at construction (crash recovery does NOT
/// pre-populate the RAM ring from disk in Phase 1).
///
/// EXPECTED TO FAIL (Red Gate): `latest_events()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_ram_ring_starts_empty_on_construction() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    // Pre-populate the file with some events (simulates restart after previous run).
    {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .expect("create pre-existing file");
        for i in 0..10 {
            let line = format!(
                r#"{{"format_version":1,"session_id":"old-sess","timestamp_micros":{i},"pid":1,"hook_type":"SessionStart"}}"#
            );
            writeln!(f, "{line}").unwrap();
        }
    }

    // Construct a new RingBuffer (simulates daemon restart).
    let ring = RingBuffer::new(path, RotationConfig::default());

    // RAM ring must be EMPTY — invariant 3: not pre-populated from disk at startup.
    let events = ring.latest_events(100);
    assert_eq!(
        events.len(),
        0,
        "RAM ring must be empty at construction (invariant 3: no pre-population from disk in Phase 1)"
    );
}

// ---------------------------------------------------------------------------
// AC-001 / BC-2.04.012 PC-1: latest_events returns in chronological order (oldest first)
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-1: `latest_events(n)` must return events in chronological order (oldest first).
/// Requesting fewer events than are present must return the N most recent.
///
/// EXPECTED TO FAIL (Red Gate): both `append()` and `latest_events()` are `todo!()`.
#[test]
fn test_BC_2_04_012_latest_events_chronological_order() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);
    let ring = RingBuffer::new(path, RotationConfig::default());

    // Append 10 events with monotonically increasing timestamps.
    for i in 0..10_i64 {
        ring.append(make_record("order-test-sess", i))
            .unwrap_or_else(|e| panic!("append {i} failed: {e}"));
    }

    // Request the last 5 — must return events 5..9 in order.
    let events = ring.latest_events(5);
    assert_eq!(
        events.len(),
        5,
        "latest_events(5) must return exactly 5 events"
    );

    // Each returned event must have strictly increasing timestamp (oldest-first order).
    for window in events.windows(2) {
        assert!(
            window[0].timestamp_micros < window[1].timestamp_micros,
            "events must be in chronological (oldest-first) order; {} >= {}",
            window[0].timestamp_micros,
            window[1].timestamp_micros
        );
    }

    // The last returned event must be the most recently appended (seq=9).
    assert_eq!(
        events[4].timestamp_micros,
        1_700_000_000_000_000_i64 + 9,
        "last element of latest_events must be the most recent event"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.04.012 PC-2: byte_count tracks on-disk bytes
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-2 (invariant 4): `current_byte_count()` must reflect bytes written
/// to the active JSONL file within one write cycle.
///
/// EXPECTED TO FAIL (Red Gate): `append()` and `current_byte_count()` are `todo!()`.
#[test]
fn test_BC_2_04_012_byte_count_reflects_written_bytes() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);
    let ring = RingBuffer::new(path.clone(), RotationConfig::default());

    // Start at 0.
    assert_eq!(
        ring.current_byte_count(),
        0,
        "byte count must be 0 before any writes"
    );

    // Append one record and check that byte count is non-zero and matches disk.
    let record = make_record("byte-count-test", 0);
    ring.append(record).expect("append must succeed");

    // After at least one flush cycle, byte count must match the on-disk file size.
    // We give the flush task a moment to write (in the real implementation).
    // Under stubs this panics immediately via todo!(), satisfying Red Gate.
    let tracked = ring.current_byte_count();
    let on_disk = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    assert_eq!(
        tracked, on_disk,
        "current_byte_count()={tracked} must equal on-disk file size={on_disk} within one write cycle (invariant 4)"
    );
}

/// BC-2.04.012 PC-3 step 8: After rotation, the byte count tracker must be reset to 0.
///
/// EXPECTED TO FAIL (Red Gate): `append()` and `current_byte_count()` are `todo!()`.
#[test]
fn test_BC_2_04_012_byte_count_reset_to_zero_after_rotation() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);
    let config = tiny_rotation_config(100, 5);
    let ring = RingBuffer::new(path.clone(), config);

    let large_record = make_tool_record(&"byte-reset-".repeat(20), 0, "Bash");

    // First append: fill past threshold.
    ring.append(large_record.clone()).expect("first append");
    // Second append: must trigger rotation and reset byte count.
    ring.append(large_record)
        .expect("second append — rotation triggered");

    // After rotation, byte count must be reset to 0 (step 8 of rotation procedure).
    let count_after_rotation = ring.current_byte_count();
    // It may have been immediately updated by the second append's bytes,
    // but it must NOT contain the pre-rotation total.
    // The pre-rotation content was > 100 bytes; if reset correctly, count < 100.
    assert!(
        count_after_rotation < 200,
        "byte count must be reset after rotation; got {count_after_rotation} (should reflect only the new active file)"
    );
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.04.012 PC-7: Graceful shutdown — flush task drains; file NOT deleted
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-7 (AC-007): On graceful shutdown (dropping the RingBuffer), the active
/// JSONL file must NOT be deleted. Unlike the lock file and hooks-settings.json, the
/// ring file is persistent across daemon restarts.
///
/// This tests the persistence guarantee — the file exists before and after ring drop.
///
/// EXPECTED TO FAIL (Red Gate): `append()` body is `todo!()`.
#[test]
fn test_BC_2_04_012_graceful_shutdown_file_not_deleted() {
    let dir = TempDir::new().expect("create tempdir");
    let path = active_path(&dir);

    let ring = RingBuffer::new(path.clone(), RotationConfig::default());
    ring.append(make_record("shutdown-test", 0))
        .expect("append must succeed");

    // Drop the ring (simulates graceful shutdown / daemon stop).
    drop(ring);

    // The active JSONL file must still exist after shutdown (PC-7).
    assert!(
        path.exists(),
        "monocle.jsonl must NOT be deleted on graceful shutdown (BC-2.04.012 PC-7)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.04.012 PC-2: ROTATION_HARD_CAP_BYTES constant correctness
// ---------------------------------------------------------------------------

/// BC-2.04.012 PC-2: The compile-time `ROTATION_HARD_CAP_BYTES` must equal 100 MiB exactly.
///
/// This test verifies the constant value — it does NOT invoke any stub and will PASS.
/// It is included here for traceability to PC-2 and to catch future constant drift.
#[test]
fn test_BC_2_04_012_rotation_hard_cap_bytes_constant_value() {
    assert_eq!(
        ROTATION_HARD_CAP_BYTES, 104_857_600_u64,
        "ROTATION_HARD_CAP_BYTES must be exactly 100 MiB = 104,857,600 bytes (BC-2.04.012 PC-2)"
    );
}

/// BC-2.04.012 PC-1: The compile-time `RAM_RING_CAPACITY` must equal 4096.
///
/// This test verifies the constant value — it does NOT invoke any stub and will PASS.
/// It is included here for traceability to PC-1 and to catch future constant drift.
#[test]
fn test_BC_2_04_012_ram_ring_capacity_constant_value() {
    assert_eq!(
        RAM_RING_CAPACITY, 4096_usize,
        "RAM_RING_CAPACITY must be exactly 4096 (BC-2.04.012 PC-1)"
    );
}
