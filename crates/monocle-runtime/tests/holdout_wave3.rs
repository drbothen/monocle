//! Holdout evaluation tests for Wave 3 (monocle-runtime).
//!
//! These tests exercise hidden acceptance scenarios that were NOT visible to
//! the implementation team. They validate behavioral contracts from the
//! outside, using only the public API surface.
//!
//! Scenarios covered:
//! - HS-W3-001: Crash Recovery Checkpoint Survives Daemon Restart
//! - HS-W3-002: JSONL Ring format_version Survives Rotation
//! - HS-W3-004: HomeUnresolvable Does Not Leak Partial Engine State
//! - HS-W3-006: Concurrent Body Limit + Auth Failure Ordering

// Test files: expect/unwrap are idiomatic assertion amplification.
#![allow(clippy::expect_used, clippy::unwrap_used)]
#![allow(non_snake_case)]
// fs::write used for test fixture inspection only.
#![allow(clippy::disallowed_methods)]

use std::sync::Arc;

use monocle_runtime::lifecycle::{read_recovery_checkpoint, write_recovery_checkpoint};
use monocle_runtime::ring::{HookEventRecord, RingBuffer, RotationConfig, RING_FORMAT_VERSION};
use monocle_runtime::state::DaemonState;
use monocle_runtime::types::{CheckpointReadResult, RecoveryCheckpoint, ShutdownReason};
use tempfile::TempDir;

// ═══════════════════════════════════════════════════════════════════════════
// HS-W3-001: Crash Recovery Checkpoint Survives Daemon Restart
// ═══════════════════════════════════════════════════════════════════════════

/// HS-W3-001: Write a checkpoint with shutdown_reason = "signal", verify the
/// file exists with exactly 4 fields, verify field constraints, call detect()
/// (read), and then delete + re-detect.
#[test]
fn test_HS_W3_001_crash_recovery_checkpoint_survives_restart() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("monocle.recovery.json");

    // Step 1: Write a checkpoint with shutdown_reason = Signal
    let checkpoint = RecoveryCheckpoint {
        pid: std::process::id(),
        shutdown_reason: ShutdownReason::Signal,
        last_app_mode: "Running".to_string(),
        shutdown_utc: chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
    };

    write_recovery_checkpoint(&path, &checkpoint)
        .expect("write_recovery_checkpoint must succeed");

    // Step 2: Verify the file exists
    assert!(
        path.exists(),
        "HS-W3-001: monocle.recovery.json must exist after write"
    );

    // Step 3: Read the file and verify exactly 4 fields
    let contents = std::fs::read_to_string(&path).expect("file must be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&contents).expect("file must be valid JSON");
    let obj = parsed.as_object().expect("must be a JSON object");

    assert_eq!(
        obj.len(),
        4,
        "HS-W3-001: checkpoint JSON must have exactly 4 fields; got {} fields: {:?}",
        obj.len(),
        obj.keys().collect::<Vec<_>>()
    );

    // Verify the 4 field names
    assert!(obj.contains_key("pid"), "must contain 'pid'");
    assert!(
        obj.contains_key("shutdown_reason"),
        "must contain 'shutdown_reason'"
    );
    assert!(
        obj.contains_key("last_app_mode"),
        "must contain 'last_app_mode'"
    );
    assert!(
        obj.contains_key("shutdown_utc"),
        "must contain 'shutdown_utc'"
    );

    // Step 4: Verify pid >= 1
    let pid_val = obj["pid"].as_u64().expect("pid must be an integer");
    assert!(
        pid_val >= 1,
        "HS-W3-001: pid must be >= 1, got {}",
        pid_val
    );

    // Step 5: Verify shutdown_reason == "signal"
    let reason = obj["shutdown_reason"]
        .as_str()
        .expect("shutdown_reason must be a string");
    assert_eq!(
        reason, "signal",
        "HS-W3-001: shutdown_reason must be 'signal', got '{}'",
        reason
    );

    // Step 6: Verify shutdown_utc matches the regex
    let utc = obj["shutdown_utc"]
        .as_str()
        .expect("shutdown_utc must be a string");
    let re = regex_lite::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$")
        .expect("regex must compile");
    assert!(
        re.is_match(utc),
        "HS-W3-001: shutdown_utc '{}' must match YYYY-MM-DDTHH:MM:SS.sssZ",
        utc
    );

    // Step 7: Call read_recovery_checkpoint (detect) — must return Valid
    let result = read_recovery_checkpoint(&path);
    match &result {
        CheckpointReadResult::Valid(cp) => {
            assert_eq!(
                cp.shutdown_reason,
                ShutdownReason::Signal,
                "HS-W3-001: detected checkpoint must have shutdown_reason == Signal"
            );
            assert!(
                cp.pid >= 1,
                "HS-W3-001: detected checkpoint pid must be >= 1"
            );
        }
        other => panic!(
            "HS-W3-001: read_recovery_checkpoint must return Valid, got {:?}",
            std::mem::discriminant(other)
        ),
    }

    // Step 8: Delete the file
    std::fs::remove_file(&path).expect("must be able to delete checkpoint file");

    // Step 9: Call detect again — must return Absent (None equivalent)
    let result_after_delete = read_recovery_checkpoint(&path);
    assert!(
        matches!(result_after_delete, CheckpointReadResult::Absent),
        "HS-W3-001: after deletion, read_recovery_checkpoint must return Absent (None)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// HS-W3-002: JSONL Ring format_version Survives Rotation
// ═══════════════════════════════════════════════════════════════════════════

/// HS-W3-002: Create a RingBuffer with a small rotation threshold, push enough
/// events to trigger rotation, then verify that records written after rotation
/// still contain format_version as the first key.
#[test]
fn test_HS_W3_002_ring_format_version_survives_rotation() {
    let dir = TempDir::new().expect("tempdir");
    let ring_path = dir.path().join("monocle-events.jsonl");

    // Use a threshold that allows a few records before rotation.
    // A single HookEventRecord serialized is roughly 150-200 bytes.
    // Set threshold at 500 bytes to trigger after ~3 records.
    let config = RotationConfig {
        soft_threshold_bytes: 500,
        hard_cap_bytes: 1000,
        retained: 3,
    };

    let ring = RingBuffer::new(ring_path.clone(), config);

    // Phase 1: Push enough events to trigger at least one rotation
    for i in 0..10 {
        let record = HookEventRecord::new(
            format!("session-{i}"),
            1_000_000 + i64::from(i),
            42,
            "PreToolUse".to_string(),
            Some("Write".to_string()),
            None,
        );
        ring.push(&record).expect("push must succeed");
    }

    // Verify rotation happened (rotated file .1 should exist)
    let rotated_1 = dir.path().join("monocle-events.jsonl.1");
    assert!(
        rotated_1.exists(),
        "HS-W3-002: rotation must produce monocle-events.jsonl.1"
    );

    // Phase 2: Read the rotated file .1 and check format_version in each record
    let rotated_contents =
        std::fs::read_to_string(&rotated_1).expect("rotated file must be readable");

    for (line_idx, line) in rotated_contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record: serde_json::Value =
            serde_json::from_str(line).expect("each line must be valid JSON");

        // Verify format_version is present and correct
        assert_eq!(
            record.get("format_version").and_then(|v| v.as_u64()),
            Some(u64::from(RING_FORMAT_VERSION)),
            "HS-W3-002: line {} in rotated file must have format_version = {}",
            line_idx,
            RING_FORMAT_VERSION
        );

        // Verify format_version is the FIRST key
        assert!(
            line.starts_with(r#"{"format_version":"#),
            "HS-W3-002: line {}: format_version must be the FIRST key. \
             Line starts with: {:?}",
            line_idx,
            &line[..line.len().min(60)]
        );
    }

    // Phase 3: Read the current active file (if it exists) and check the same
    if ring_path.exists() {
        let active_contents =
            std::fs::read_to_string(&ring_path).expect("active file must be readable");
        for (line_idx, line) in active_contents.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let record: serde_json::Value =
                serde_json::from_str(line).expect("each line must be valid JSON");

            assert_eq!(
                record.get("format_version").and_then(|v| v.as_u64()),
                Some(u64::from(RING_FORMAT_VERSION)),
                "HS-W3-002: active file line {} must have format_version = {}",
                line_idx,
                RING_FORMAT_VERSION
            );

            assert!(
                line.starts_with(r#"{"format_version":"#),
                "HS-W3-002: active file line {}: format_version must be the FIRST key. \
                 Line starts with: {:?}",
                line_idx,
                &line[..line.len().min(60)]
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HS-W3-004: HomeUnresolvable Does Not Leak Partial Engine State
// ═══════════════════════════════════════════════════════════════════════════

/// HS-W3-004: Unset HOME, USERPROFILE, HOMEDRIVE, HOMEPATH, call metadata() —
/// must return Err(HomeUnresolvable), no partial EngineMetadata, and emit E-ENG-001.
#[tokio::test]
#[tracing_test::traced_test]
async fn test_HS_W3_004_home_unresolvable_no_partial_state() {
    use monocle_core::engine::{EngineMetadataError, EngineModule};
    use monocle_runtime::engine::ClaudeCodeModule;

    let env_vars: [(&str, Option<&str>); 4] = [
        ("HOME", None),
        ("USERPROFILE", None),
        ("HOMEDRIVE", None),
        ("HOMEPATH", None),
    ];

    temp_env::async_with_vars(env_vars, async {
        let module = ClaudeCodeModule::new("http://127.0.0.1:7891".to_string());
        let result = module.metadata();

        // Must return Err, not Ok with partial data
        assert!(
            result.is_err(),
            "HS-W3-004: metadata() must return Err(HomeUnresolvable) when HOME is unset; \
             got Ok — this would leak partial engine state"
        );

        let err = result.unwrap_err();
        assert!(
            matches!(err, EngineMetadataError::HomeUnresolvable),
            "HS-W3-004: error must be HomeUnresolvable, got {:?}",
            err
        );
    })
    .await;

    // Verify E-ENG-001 log message
    assert!(
        logs_contain("E-ENG-001"),
        "HS-W3-004: metadata() must emit E-ENG-001 log message when HOME is unset"
    );
    assert!(
        logs_contain("platform home directory unresolvable"),
        "HS-W3-004: E-ENG-001 log must contain 'platform home directory unresolvable'"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// HS-W3-006: Concurrent Body Limit + Auth Failure Ordering
// ═══════════════════════════════════════════════════════════════════════════

/// HS-W3-006: Send a POST to /hooks/pre-tool-use with body > 256 KiB AND no auth header.
/// The holdout scenario expects HTTP 413 (body limit checked BEFORE auth).
/// We observe the actual behavior to determine whether the implementation
/// checks body size or auth first.
#[tokio::test]
async fn test_HS_W3_006_body_limit_vs_auth_ordering() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use monocle_runtime::server::build_server;
    use tower::ServiceExt;

    // Build server with a real auth token
    let mut state = DaemonState::new();
    state.auth_token =
        "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string();
    let state = Arc::new(state);
    let app = build_server(state);

    // Build request: body > 256 KiB, NO auth header
    let body_len: usize = 262_145; // 256 KiB + 1 byte
    let body_bytes = vec![0u8; body_len];
    let req = Request::builder()
        .method("POST")
        .uri("/hooks/pre-tool-use")
        .header("Content-Length", body_len.to_string())
        // Deliberately NOT setting any auth header
        .body(Body::from(body_bytes))
        .expect("build POST request");

    let response = app.oneshot(req).await.expect("oneshot POST");
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let body_str = String::from_utf8_lossy(&body);

    eprintln!(
        "HS-W3-006: status={}, body={}",
        status, body_str
    );

    // The holdout scenario expects HTTP 413 (body limit checked BEFORE auth).
    // Middleware ordering: body_size_limit_middleware (outermost) → auth_middleware → handler.
    // Oversized payloads are rejected at 413 before any auth work (defense-in-depth).
    assert_eq!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "HS-W3-006: body limit middleware must run before auth middleware. \
         Oversized + unauthenticated request must return 413, not 401. Got: {}",
        status
    );
}
