//! Integration tests for DTU fidelity score ≥0.95 against 25-fixture corpus — AC-004.
//!
// BC-based test function names use BC_S_SS_NNN_ uppercase pattern per factory convention.
// expect_used: test helpers use .expect() / .unwrap_or_else in test-only code; this is
// intentional — panics in test code surface failures clearly.
#![allow(non_snake_case)]
#![allow(clippy::expect_used)]
//!
//! Each test:
//!   1. Starts a mock daemon on a random loopback port.
//!   2. Builds a CloneState pointing at that mock daemon.
//!   3. Drives the clone router via tower::ServiceExt (in-process, no real TCP for clone side).
//!   4. Waits (≤500ms) for the mock daemon to receive what the clone forwarded.
//!   5. Scores the received wire bytes against the fixture using FixtureScore::compute.
//!   6. Asserts score ≥ 0.95.
//!
//! This design catches real implementation gaps:
//! - If the clone drops a field in re-serialization, the score drops below 0.95.
//! - If the clone filters a notification it should forward, the daemon receives nothing.
//! - If the mock daemon never receives a POST (fire-and-forget fails), the test fails.
//!
//! For `non-permission-dropped`: the clone MUST NOT forward to the daemon (BC-HOOK-034
//! client-side filter). This fixture's test asserts the daemon receives nothing.
//!
//! Source authority:
//! - AC-004 (≥0.95 fidelity against 25-fixture corpus)
//! - BC-HOOK-007 (scoring function)
//! - dtu-assessment.md §DTU Fidelity Measurement Procedure (implemented against v1.7.5 at S-DTU-001 authoring time)

#[path = "common/mod.rs"]
mod common;

use monocle_test_harness::dtu::{endpoints::build_router, payload::FixtureScore};

// ──────────────────────────────────────────────────────────────────────────────
// AC-004: 25-fixture corpus completeness check (non-fidelity — stays sync)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-004: The fixture corpus has exactly 25 fixtures (5 per endpoint × 5 endpoints).
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
}

/// AC-004: All 25 fixture files are valid parseable JSON.
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
            // load_fixture panics on parse error — reaching here means valid JSON.
            let _v = common::load_fixture(endpoint, &name);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Fidelity test helper: drive clone → capture daemon receive → score
// ──────────────────────────────────────────────────────────────────────────────

/// Drive the clone router with a fixture POST and return what the mock daemon received.
///
/// Returns `Some(received_json)` if the daemon received a POST within 500ms.
/// Returns `None` if the daemon received nothing (filtered or fire-and-forget failed).
///
/// The clone router is driven via `tower::ServiceExt::oneshot` in-process.
/// The `spawn_daemon_post` inside the handler fires a background `tokio::spawn`
/// which sends the POST to the mock daemon. We wait up to 500ms for it to arrive.
async fn drive_and_capture(
    endpoint_path: &'static str,
    fixture: &serde_json::Value,
) -> Option<serde_json::Value> {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    // Start mock daemon.
    let mut mock = common::start_mock_daemon().await;
    let state = common::make_clone_state_for_mock(mock.port);
    let router = build_router(state);

    // POST the fixture JSON to the clone router.
    let req = Request::builder()
        .method("POST")
        .uri(endpoint_path)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(fixture).expect("serialize fixture"),
        ))
        .expect("build req");

    // Drive in-process (this triggers the handler which calls spawn_daemon_post).
    let response = router.oneshot(req).await.expect("oneshot");
    // Handler must return 200 (fail-open / echo semantics).
    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "clone handler must return 200; got {}",
        response.status()
    );

    // Wait for the background spawn_daemon_post to POST to the mock daemon.
    // We give it up to 500ms. Fire-and-forget means no guarantee of ordering;
    // 500ms is conservative and avoids flakiness.
    let received =
        tokio::time::timeout(std::time::Duration::from_millis(500), mock.receiver.recv()).await;

    // Graceful shutdown of mock daemon.
    let _ = mock.shutdown_tx.send(());

    match received {
        Ok(Some(bytes)) => {
            // Parse the received bytes as JSON.
            serde_json::from_slice(&bytes).ok()
        }
        _ => None,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-004: Per-fixture fidelity scores ≥ 0.95
// ──────────────────────────────────────────────────────────────────────────────

/// AC-004: pre-tool-use/bash-simple — clone forwards correct payload to daemon (score ≥ 0.95).
///
/// Drives the clone router with the fixture body. Verifies the mock daemon receives
/// the forwarded bytes and that the fidelity score against the fixture is ≥ 0.95.
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_pre_tool_use_bash_simple() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("pre-tool-use", "bash-simple");
    let clone_output = drive_and_capture(paths::PRE_TOOL_USE, &fixture).await;
    let clone_json = clone_output.expect(
        "mock daemon must receive a POST for pre-tool-use/bash-simple — clone did not forward",
    );
    let score = FixtureScore::compute(
        "pre-tool-use/bash-simple.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: pre-tool-use/bash-multiline — clone forwards correct payload to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_pre_tool_use_bash_multiline() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("pre-tool-use", "bash-multiline");
    let clone_output = drive_and_capture(paths::PRE_TOOL_USE, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for pre-tool-use/bash-multiline");
    let score = FixtureScore::compute(
        "pre-tool-use/bash-multiline.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: pre-tool-use/read-file — clone forwards correct payload to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_pre_tool_use_read_file() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("pre-tool-use", "read-file");
    let clone_output = drive_and_capture(paths::PRE_TOOL_USE, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for pre-tool-use/read-file");
    let score = FixtureScore::compute(
        "pre-tool-use/read-file.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: pre-tool-use/write-file — clone forwards correct payload to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_pre_tool_use_write_file() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("pre-tool-use", "write-file");
    let clone_output = drive_and_capture(paths::PRE_TOOL_USE, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for pre-tool-use/write-file");
    let score = FixtureScore::compute(
        "pre-tool-use/write-file.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: pre-tool-use/unknown-tool — clone forwards correct payload to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_pre_tool_use_unknown_tool() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("pre-tool-use", "unknown-tool");
    let clone_output = drive_and_capture(paths::PRE_TOOL_USE, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for pre-tool-use/unknown-tool");
    let score = FixtureScore::compute(
        "pre-tool-use/unknown-tool.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: notification/permission-prompt-bash — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_notification_permission_prompt_bash() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("notification", "permission-prompt-bash");
    let clone_output = drive_and_capture(paths::NOTIFICATION, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for notification/permission-prompt-bash");
    let score = FixtureScore::compute(
        "notification/permission-prompt-bash.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: notification/permission-prompt-write — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_notification_permission_prompt_write() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("notification", "permission-prompt-write");
    let clone_output = drive_and_capture(paths::NOTIFICATION, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for notification/permission-prompt-write");
    let score = FixtureScore::compute(
        "notification/permission-prompt-write.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: notification/permission-prompt-web-fetch — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_notification_permission_prompt_web_fetch() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("notification", "permission-prompt-web-fetch");
    let clone_output = drive_and_capture(paths::NOTIFICATION, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for notification/permission-prompt-web-fetch");
    let score = FixtureScore::compute(
        "notification/permission-prompt-web-fetch.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004 / BC-HOOK-034: notification/non-permission-dropped — daemon must receive NOTHING.
///
/// The clone MUST NOT forward non-permission_prompt notifications to the daemon
/// (client-side filter per BC-HOOK-034). The mock daemon times out with no payload.
/// If the daemon receives a payload, this test fails — the filter is broken.
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_notification_non_permission_dropped() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("notification", "non-permission-dropped");
    // non-permission-dropped has notification_type == "assistant_message" — must NOT be forwarded.
    assert_eq!(
        fixture["notification_type"].as_str().unwrap_or(""),
        "assistant_message",
        "fixture must have notification_type=assistant_message to exercise filter"
    );
    let clone_output = drive_and_capture(paths::NOTIFICATION, &fixture).await;
    // BC-HOOK-034: daemon must receive NOTHING for non-permission_prompt notifications.
    assert!(
        clone_output.is_none(),
        "BC-HOOK-034: clone must NOT forward assistant_message notifications to daemon; \
         but daemon received: {clone_output:?}"
    );
}

/// AC-004 / BC-HOOK-036: notification/large-message-boundary — 200 KiB message forwarded (score ≥ 0.95).
///
/// Verifies the clone handles large payloads without truncation or corruption.
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_notification_large_message_boundary() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("notification", "large-message-boundary");
    // Verify the fixture has a large message field AND is permission_prompt type.
    let message_len = fixture["message"].as_str().map(|s| s.len()).unwrap_or(0);
    assert!(
        message_len >= 200 * 1024,
        "large-message-boundary.json message must be ≥ 200 KiB, got {message_len} bytes"
    );
    assert_eq!(
        fixture["notification_type"].as_str().unwrap_or(""),
        "permission_prompt",
        "large-message-boundary.json must have notification_type=permission_prompt \
         to exercise the 200 KiB wire boundary (MED-3 fix required if this fails)"
    );
    let clone_output = drive_and_capture(paths::NOTIFICATION, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for notification/large-message-boundary");
    let score = FixtureScore::compute(
        "notification/large-message-boundary.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: stop/stop-reason-normal — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_stop_stop_reason_normal() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("stop", "stop-reason-normal");
    let clone_output = drive_and_capture(paths::STOP, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for stop/stop-reason-normal");
    let score = FixtureScore::compute(
        "stop/stop-reason-normal.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: stop/stop-reason-error — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_stop_stop_reason_error() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("stop", "stop-reason-error");
    let clone_output = drive_and_capture(paths::STOP, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for stop/stop-reason-error");
    let score = FixtureScore::compute(
        "stop/stop-reason-error.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: stop/stop-reason-interrupt — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_stop_stop_reason_interrupt() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("stop", "stop-reason-interrupt");
    let clone_output = drive_and_capture(paths::STOP, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for stop/stop-reason-interrupt");
    let score = FixtureScore::compute(
        "stop/stop-reason-interrupt.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: stop/stop-reason-empty — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_stop_stop_reason_empty() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("stop", "stop-reason-empty");
    let clone_output = drive_and_capture(paths::STOP, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for stop/stop-reason-empty");
    let score = FixtureScore::compute(
        "stop/stop-reason-empty.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: stop/stop-reason-unknown — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_stop_stop_reason_unknown() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("stop", "stop-reason-unknown");
    let clone_output = drive_and_capture(paths::STOP, &fixture).await;
    let clone_json =
        clone_output.expect("mock daemon must receive a POST for stop/stop-reason-unknown");
    let score = FixtureScore::compute(
        "stop/stop-reason-unknown.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: session-start/session-start-basic — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_session_start_basic() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("session-start", "session-start-basic");
    let clone_output = drive_and_capture(paths::SESSION_START, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for session-start/session-start-basic");
    let score = FixtureScore::compute(
        "session-start/session-start-basic.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: session-start/session-start-uuid-v4 — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_session_start_uuid_v4() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("session-start", "session-start-uuid-v4");
    let clone_output = drive_and_capture(paths::SESSION_START, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for session-start/session-start-uuid-v4");
    let score = FixtureScore::compute(
        "session-start/session-start-uuid-v4.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: session-start/session-start-extra-fields — extra fields MUST be preserved.
///
/// The daemon must receive all fields including `extra_future_field` and
/// `another_future_field`. If the clone drops these fields (re-serialization gap),
/// the score drops below 0.95 and this test FAILS.
///
/// Red Gate: the handler re-serializes via SessionStartPayload which drops
/// unknown fields → score ≈ 0.667 < 0.95.
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_session_start_extra_fields_not_penalized() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("session-start", "session-start-extra-fields");
    // Verify the fixture actually has extra fields (test precondition).
    assert!(
        fixture.get("extra_future_field").is_some(),
        "fixture must contain extra_future_field for this test to be meaningful"
    );
    let clone_output = drive_and_capture(paths::SESSION_START, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for session-start/session-start-extra-fields");
    let score = FixtureScore::compute(
        "session-start/session-start-extra-fields.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "extra fields must not be dropped: fidelity must be ≥ 0.95, got {:.4}; \
         mismatched fields (dropped by re-serialization): {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: session-start/session-start-missing-session-id — empty session_id forwarded (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_session_start_missing_session_id() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("session-start", "session-start-missing-session-id");
    let clone_output = drive_and_capture(paths::SESSION_START, &fixture).await;
    let clone_json = clone_output.expect(
        "mock daemon must receive a POST for session-start/session-start-missing-session-id",
    );
    let score = FixtureScore::compute(
        "session-start/session-start-missing-session-id.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: session-start/session-start-null-pid — pid=0 forwarded (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_session_start_null_pid() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("session-start", "session-start-null-pid");
    let clone_output = drive_and_capture(paths::SESSION_START, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for session-start/session-start-null-pid");
    let score = FixtureScore::compute(
        "session-start/session-start-null-pid.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: prompt-submit/prompt-submit-basic — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_prompt_submit_basic() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-basic");
    let clone_output = drive_and_capture(paths::PROMPT_SUBMIT, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for prompt-submit/prompt-submit-basic");
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-basic.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: prompt-submit/prompt-submit-uuid-v4 — clone forwards to daemon (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_prompt_submit_uuid_v4() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-uuid-v4");
    let clone_output = drive_and_capture(paths::PROMPT_SUBMIT, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for prompt-submit/prompt-submit-uuid-v4");
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-uuid-v4.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: prompt-submit/prompt-submit-extra-fields — extra fields MUST be preserved.
///
/// Red Gate: the handler re-serializes via UserPromptSubmitPayload which drops
/// unknown fields → score < 0.95.
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_prompt_submit_extra_fields_not_penalized() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-extra-fields");
    // Verify fixture has extra fields.
    let has_extra = fixture
        .as_object()
        .map(|m| {
            m.keys()
                .any(|k| k.starts_with("extra") || k.starts_with("another"))
        })
        .unwrap_or(false);
    assert!(
        has_extra,
        "prompt-submit-extra-fields.json must contain extra fields for this test to be meaningful"
    );
    let clone_output = drive_and_capture(paths::PROMPT_SUBMIT, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for prompt-submit/prompt-submit-extra-fields");
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-extra-fields.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "extra fields must not be dropped: fidelity must be ≥ 0.95, got {:.4}; \
         mismatched fields (dropped by re-serialization): {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: prompt-submit/prompt-submit-missing-session-id — clone forwards (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_prompt_submit_missing_session_id() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-missing-session-id");
    let clone_output = drive_and_capture(paths::PROMPT_SUBMIT, &fixture).await;
    let clone_json = clone_output.expect(
        "mock daemon must receive a POST for prompt-submit/prompt-submit-missing-session-id",
    );
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-missing-session-id.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

/// AC-004: prompt-submit/prompt-submit-long-session-id — clone forwards (score ≥ 0.95).
#[tokio::test]
async fn test_BC_HOOK_007_fidelity_prompt_submit_long_session_id() {
    use monocle_test_harness::dtu::endpoints::paths;
    let fixture = common::load_fixture("prompt-submit", "prompt-submit-long-session-id");
    let clone_output = drive_and_capture(paths::PROMPT_SUBMIT, &fixture).await;
    let clone_json = clone_output
        .expect("mock daemon must receive a POST for prompt-submit/prompt-submit-long-session-id");
    let score = FixtureScore::compute(
        "prompt-submit/prompt-submit-long-session-id.json".to_string(),
        &fixture,
        &clone_json,
    );
    assert!(
        score.score >= 0.95,
        "fidelity must be ≥ 0.95, got {:.4}; mismatched: {:?}",
        score.score,
        score.mismatched_fields
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-004: Mean fidelity over all 25 fixtures ≥ 0.95 (aggregate gate)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-004: Mean fidelity score across all 25 fixtures is ≥ 0.95 (aggregate gate).
///
/// This is the gate that `cargo xtask dtu-fidelity` exits 0 on iff this passes.
///
/// Each fixture is driven through the clone router to a mock daemon; the daemon-received
/// bytes are scored. The aggregate mean must be ≥ 0.95.
///
/// Expected to FAIL until the implementer fixes extra-field pass-through and the
/// non-permission-dropped filter behavior.
#[tokio::test]
async fn test_BC_HOOK_007_mean_fidelity_over_25_fixtures_gte_095() {
    use monocle_test_harness::dtu::endpoints::paths;

    struct FixtureSpec {
        endpoint: &'static str,
        path: &'static str,
        name: &'static str,
        /// If true, the daemon must receive a POST. If false, filter test — scored separately.
        expect_forward: bool,
    }

    let fixtures: &[FixtureSpec] = &[
        FixtureSpec {
            endpoint: "pre-tool-use",
            path: paths::PRE_TOOL_USE,
            name: "bash-simple",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "pre-tool-use",
            path: paths::PRE_TOOL_USE,
            name: "bash-multiline",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "pre-tool-use",
            path: paths::PRE_TOOL_USE,
            name: "read-file",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "pre-tool-use",
            path: paths::PRE_TOOL_USE,
            name: "write-file",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "pre-tool-use",
            path: paths::PRE_TOOL_USE,
            name: "unknown-tool",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "notification",
            path: paths::NOTIFICATION,
            name: "permission-prompt-bash",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "notification",
            path: paths::NOTIFICATION,
            name: "permission-prompt-write",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "notification",
            path: paths::NOTIFICATION,
            name: "permission-prompt-web-fetch",
            expect_forward: true,
        },
        // non-permission-dropped: expect NO forward (BC-HOOK-034). Treated as score=1.0 IF correctly dropped.
        FixtureSpec {
            endpoint: "notification",
            path: paths::NOTIFICATION,
            name: "non-permission-dropped",
            expect_forward: false,
        },
        FixtureSpec {
            endpoint: "notification",
            path: paths::NOTIFICATION,
            name: "large-message-boundary",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "stop",
            path: paths::STOP,
            name: "stop-reason-normal",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "stop",
            path: paths::STOP,
            name: "stop-reason-error",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "stop",
            path: paths::STOP,
            name: "stop-reason-interrupt",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "stop",
            path: paths::STOP,
            name: "stop-reason-empty",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "stop",
            path: paths::STOP,
            name: "stop-reason-unknown",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "session-start",
            path: paths::SESSION_START,
            name: "session-start-basic",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "session-start",
            path: paths::SESSION_START,
            name: "session-start-uuid-v4",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "session-start",
            path: paths::SESSION_START,
            name: "session-start-extra-fields",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "session-start",
            path: paths::SESSION_START,
            name: "session-start-missing-session-id",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "session-start",
            path: paths::SESSION_START,
            name: "session-start-null-pid",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "prompt-submit",
            path: paths::PROMPT_SUBMIT,
            name: "prompt-submit-basic",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "prompt-submit",
            path: paths::PROMPT_SUBMIT,
            name: "prompt-submit-uuid-v4",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "prompt-submit",
            path: paths::PROMPT_SUBMIT,
            name: "prompt-submit-extra-fields",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "prompt-submit",
            path: paths::PROMPT_SUBMIT,
            name: "prompt-submit-missing-session-id",
            expect_forward: true,
        },
        FixtureSpec {
            endpoint: "prompt-submit",
            path: paths::PROMPT_SUBMIT,
            name: "prompt-submit-long-session-id",
            expect_forward: true,
        },
    ];

    let mut total_score = 0.0_f64;
    let mut count = 0_usize;
    let mut failures: Vec<String> = Vec::new();

    for spec in fixtures {
        let fixture = common::load_fixture(spec.endpoint, spec.name);
        let fixture_key = format!("{}/{}.json", spec.endpoint, spec.name);

        if !spec.expect_forward {
            // For filtered fixtures: correctness = daemon receives nothing.
            // Score as 1.0 iff filter works, 0.0 iff incorrectly forwarded.
            let received = drive_and_capture(spec.path, &fixture).await;
            let score = if received.is_none() { 1.0 } else { 0.0 };
            if score < 0.95 {
                failures.push(format!(
                    "{fixture_key}: filter broken, daemon received payload"
                ));
            }
            total_score += score;
            count += 1;
            continue;
        }

        let received = drive_and_capture(spec.path, &fixture).await;
        match received {
            None => {
                failures.push(format!(
                    "{fixture_key}: daemon received nothing (not forwarded)"
                ));
                // Score 0: nothing forwarded.
                count += 1;
            }
            Some(clone_json) => {
                let result = FixtureScore::compute(fixture_key.clone(), &fixture, &clone_json);
                if result.score < 0.95 {
                    failures.push(format!(
                        "{fixture_key}: score={:.4}, mismatched={:?}",
                        result.score, result.mismatched_fields
                    ));
                }
                total_score += result.score;
                count += 1;
            }
        }
    }

    assert_eq!(count, 25, "must score exactly 25 fixtures");
    let mean = total_score / count as f64;
    assert!(
        failures.is_empty() && mean >= 0.95,
        "mean fidelity = {mean:.4} < 0.95 (or individual failures):\n{}",
        failures.join("\n")
    );
}
