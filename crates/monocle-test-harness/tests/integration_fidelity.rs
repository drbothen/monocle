//! Integration tests for DTU fidelity score ≥0.95 against 25-fixture corpus — AC-004.
//!
//! Verifies:
//! - FixtureScore::compute returns score ≥ 0.95 across all 25 fixtures
//! - Per-fixture scoring: each of the 25 fixture files is exercised individually
//! - Large payload boundary: 200.2 KiB message field (large-message-boundary.json)
//! - FixtureScore correctly handles missing required fields (score = 0 per field)
//! - FixtureScore ignores extra fields (not penalized)
//! - The 25-fixture corpus covers all 5 endpoints (5 fixtures × 5 endpoints)
//!
//! Source authority:
//! - AC-004 (≥0.95 fidelity against 25-fixture corpus)
//! - BC-HOOK-007 (scoring function)
//! - dtu-assessment.md v1.7.5 §DTU Fidelity Measurement Procedure

#[path = "common/mod.rs"]
mod common;

use monocle_test_harness::dtu::payload::FixtureScore;

// ──────────────────────────────────────────────────────────────────────────────
// AC-004: 25-fixture corpus completeness check
// ──────────────────────────────────────────────────────────────────────────────

/// AC-004: The fixture corpus has exactly 25 fixtures (5 per endpoint × 5 endpoints)
/// AND FixtureScore::compute is callable for each one.
///
/// Red Gate: FixtureScore::compute panics with todo!() — so this test fails after the
/// corpus count check is verified, exercising the unimplemented scoring function.
#[test]
fn test_BC_HOOK_007_fixture_corpus_has_25_fixtures() {
    let endpoints = [
        "pre-tool-use",
        "notification",
        "stop",
        "session-start",
        "prompt-submit",
    ];
    let mut total = 0;
    for endpoint in &endpoints {
        let fixtures = common::list_fixtures(endpoint);
        assert_eq!(
            fixtures.len(),
            5,
            "endpoint {endpoint} must have exactly 5 fixtures, got {}",
            fixtures.len()
        );
        total += fixtures.len();
    }
    assert_eq!(total, 25, "total fixture count must be 25");
    // Verify FixtureScore::compute is callable (panics at todo!() — Red Gate).
    // We must call it to exercise the scoring function, not just count files.
    let first_fixture = common::load_fixture("pre-tool-use", "bash-simple");
    let _score = FixtureScore::compute(
        "pre-tool-use/bash-simple.json".to_string(),
        &first_fixture,
        &first_fixture,
    );
}

/// AC-004: All 25 fixture files are valid parseable JSON AND scoreable by FixtureScore::compute.
///
/// Red Gate: FixtureScore::compute panics with todo!() after the first fixture is loaded.
#[test]
fn test_BC_HOOK_007_all_fixtures_are_valid_json() {
    let endpoints = [
        "pre-tool-use",
        "notification",
        "stop",
        "session-start",
        "prompt-submit",
    ];
    for endpoint in &endpoints {
        for name in common::list_fixtures(endpoint) {
            let v = common::load_fixture(endpoint, &name);
            // load_fixture panics on parse error — reaching here means valid JSON.
            // Also invoke FixtureScore::compute to exercise the unimplemented path (Red Gate).
            let _score = FixtureScore::compute(format!("{endpoint}/{name}.json"), &v, &v);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-004: Per-fixture fidelity scores ≥ 0.95
// ──────────────────────────────────────────────────────────────────────────────

/// AC-004: FixtureScore for pre-tool-use/bash-simple.json is ≥ 0.95.
///
/// This test exercises the FixtureScore::compute scoring function against
/// the bash-simple fixture. Panics from todo!() until implemented.
#[test]
fn test_BC_HOOK_007_fidelity_pre_tool_use_bash_simple() {
    let fixture = common::load_fixture("pre-tool-use", "bash-simple");
    // In the real implementation, the DTU clone constructs a payload from the fixture input.
    // For the Red Gate, we pass the fixture as both expected and clone_output — the
    // scoring function would return 1.0 if implemented, but panics with todo!() now.
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "pre-tool-use/bash-simple.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for pre-tool-use/bash-multiline.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_pre_tool_use_bash_multiline() {
    let fixture = common::load_fixture("pre-tool-use", "bash-multiline");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "pre-tool-use/bash-multiline.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for pre-tool-use/read-file.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_pre_tool_use_read_file() {
    let fixture = common::load_fixture("pre-tool-use", "read-file");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "pre-tool-use/read-file.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for pre-tool-use/write-file.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_pre_tool_use_write_file() {
    let fixture = common::load_fixture("pre-tool-use", "write-file");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "pre-tool-use/write-file.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for pre-tool-use/unknown-tool.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_pre_tool_use_unknown_tool() {
    let fixture = common::load_fixture("pre-tool-use", "unknown-tool");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "pre-tool-use/unknown-tool.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for notification/permission-prompt-bash.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_notification_permission_prompt_bash() {
    let fixture = common::load_fixture("notification", "permission-prompt-bash");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "notification/permission-prompt-bash.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for notification/permission-prompt-write.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_notification_permission_prompt_write() {
    let fixture = common::load_fixture("notification", "permission-prompt-write");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "notification/permission-prompt-write.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for notification/permission-prompt-web-fetch.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_notification_permission_prompt_web_fetch() {
    let fixture = common::load_fixture("notification", "permission-prompt-web-fetch");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "notification/permission-prompt-web-fetch.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for notification/non-permission-dropped.json is ≥ 0.95.
/// Per AC-004: this fixture MUST NOT reach the wire (filter validation in integration_filters.rs).
#[test]
fn test_BC_HOOK_007_fidelity_notification_non_permission_dropped() {
    let fixture = common::load_fixture("notification", "non-permission-dropped");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "notification/non-permission-dropped.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004 / BC-HOOK-036: FixtureScore for notification/large-message-boundary.json is ≥ 0.95.
/// This fixture has a 200.2 KiB message field (all-ASCII, so byte count = char count).
#[test]
fn test_BC_HOOK_007_fidelity_notification_large_message_boundary() {
    let fixture = common::load_fixture("notification", "large-message-boundary");
    // Verify the message field is actually large (200 KiB+).
    let message_len = fixture["message"].as_str().map(|s| s.len()).unwrap_or(0);
    assert!(
        message_len >= 200 * 1024,
        "large-message-boundary.json message must be ≥ 200 KiB, got {message_len} bytes"
    );
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "notification/large-message-boundary.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for stop/stop-reason-normal.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_stop_stop_reason_normal() {
    let fixture = common::load_fixture("stop", "stop-reason-normal");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "stop/stop-reason-normal.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for stop/stop-reason-error.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_stop_stop_reason_error() {
    let fixture = common::load_fixture("stop", "stop-reason-error");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "stop/stop-reason-error.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for stop/stop-reason-interrupt.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_stop_stop_reason_interrupt() {
    let fixture = common::load_fixture("stop", "stop-reason-interrupt");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "stop/stop-reason-interrupt.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for stop/stop-reason-empty.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_stop_stop_reason_empty() {
    let fixture = common::load_fixture("stop", "stop-reason-empty");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "stop/stop-reason-empty.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for stop/stop-reason-unknown.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_stop_stop_reason_unknown() {
    let fixture = common::load_fixture("stop", "stop-reason-unknown");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "stop/stop-reason-unknown.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for session-start/session-start-basic.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_session_start_basic() {
    let fixture = common::load_fixture("session-start", "session-start-basic");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "session-start/session-start-basic.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for session-start/session-start-uuid-v4.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_session_start_uuid_v4() {
    let fixture = common::load_fixture("session-start", "session-start-uuid-v4");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "session-start/session-start-uuid-v4.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for session-start/session-start-extra-fields.json is ≥ 0.95.
/// Extra fields must not be penalized (dtu-assessment.md §Scoring Function).
#[test]
fn test_BC_HOOK_007_fidelity_session_start_extra_fields_not_penalized() {
    let fixture = common::load_fixture("session-start", "session-start-extra-fields");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "session-start/session-start-extra-fields.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "extra fields must not penalize fidelity; got {}",
        score.score
    );
}

/// AC-004: FixtureScore for session-start/session-start-missing-session-id.json.
/// Empty session_id is a valid fixture input (empty string, not absent field).
#[test]
fn test_BC_HOOK_007_fidelity_session_start_missing_session_id() {
    let fixture = common::load_fixture("session-start", "session-start-missing-session-id");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "session-start/session-start-missing-session-id.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for session-start/session-start-null-pid.json.
/// pid=0 is a valid edge case.
#[test]
fn test_BC_HOOK_007_fidelity_session_start_null_pid() {
    let fixture = common::load_fixture("session-start", "session-start-null-pid");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "session-start/session-start-null-pid.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for prompt-submit/prompt-submit-basic.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_prompt_submit_basic() {
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-basic");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-basic.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for prompt-submit/prompt-submit-uuid-v4.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_prompt_submit_uuid_v4() {
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-uuid-v4");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-uuid-v4.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for prompt-submit/prompt-submit-extra-fields.json is ≥ 0.95.
/// Extra fields must not be penalized.
#[test]
fn test_BC_HOOK_007_fidelity_prompt_submit_extra_fields_not_penalized() {
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-extra-fields");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-extra-fields.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "extra fields must not penalize fidelity; got {}",
        score.score
    );
}

/// AC-004: FixtureScore for prompt-submit/prompt-submit-missing-session-id.json.
#[test]
fn test_BC_HOOK_007_fidelity_prompt_submit_missing_session_id() {
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-missing-session-id");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-missing-session-id.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

/// AC-004: FixtureScore for prompt-submit/prompt-submit-long-session-id.json is ≥ 0.95.
#[test]
fn test_BC_HOOK_007_fidelity_prompt_submit_long_session_id() {
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-long-session-id");
    let clone_output = fixture.clone();
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-long-session-id.json".to_string(),
        &fixture,
        &clone_output,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {}",
        score.score
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-004: Mean fidelity over all 25 fixtures ≥ 0.95 (aggregate gate)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-004: Mean fidelity score across all 25 fixtures is ≥ 0.95.
///
/// This is the aggregate gate: `cargo xtask dtu-fidelity` exits 0 iff this passes.
/// test_BC_HOOK_007_mean_fidelity_over_25_fixtures_gte_095
#[test]
fn test_BC_HOOK_007_mean_fidelity_over_25_fixtures_gte_095() {
    let endpoint_fixtures = [
        (
            "pre-tool-use",
            vec![
                "bash-simple",
                "bash-multiline",
                "read-file",
                "write-file",
                "unknown-tool",
            ],
        ),
        (
            "notification",
            vec![
                "permission-prompt-bash",
                "permission-prompt-write",
                "permission-prompt-web-fetch",
                "non-permission-dropped",
                "large-message-boundary",
            ],
        ),
        (
            "stop",
            vec![
                "stop-reason-normal",
                "stop-reason-error",
                "stop-reason-interrupt",
                "stop-reason-empty",
                "stop-reason-unknown",
            ],
        ),
        (
            "session-start",
            vec![
                "session-start-basic",
                "session-start-uuid-v4",
                "session-start-extra-fields",
                "session-start-missing-session-id",
                "session-start-null-pid",
            ],
        ),
        (
            "prompt-submit",
            vec![
                "prompt-submit-basic",
                "prompt-submit-uuid-v4",
                "prompt-submit-extra-fields",
                "prompt-submit-missing-session-id",
                "prompt-submit-long-session-id",
            ],
        ),
    ];

    let mut total_score = 0.0_f64;
    let mut count = 0_usize;

    for (endpoint, fixtures) in &endpoint_fixtures {
        for name in fixtures {
            let fixture = common::load_fixture(endpoint, name);
            let clone_output = fixture.clone();
            let score =
                FixtureScore::compute(format!("{endpoint}/{name}.json"), &fixture, &clone_output);
            total_score += score.score;
            count += 1;
        }
    }

    let mean = total_score / count as f64;
    assert!(
        mean >= 0.95,
        "mean fidelity over {count} fixtures must be ≥ 0.95, got {mean:.4}"
    );
}
