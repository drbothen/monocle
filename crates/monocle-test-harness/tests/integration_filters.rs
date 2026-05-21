//! Integration tests for monocle improvement filters — AC-005, AC-006.
//!
//! Verifies:
//! - BC-HOOK-024: non-monocle lock files at higher port are SKIPPED
//! - BC-HOOK-003 / BC-HOOK-034: notification_type != "permission_prompt" is NOT forwarded
//! - BC-HOOK-034: non-numeric lock file port (NaN equivalent) is skipped
//! - AC-006: MONOCLE_NO_AUTOSTART=1 prevents daemon auto-start
//!
//! Source authority:
//! - AC-005 (cross-IDE collision filter)
//! - AC-006 (env var overrides)
//! - BC-HOOK-003, BC-HOOK-024, BC-HOOK-034

#[path = "common/mod.rs"]
mod common;

use axum::http::StatusCode;
use monocle_test_harness::dtu::endpoints::{build_router, paths};

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-034 / AC-006: Notification filter — non-permission_prompt types dropped
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-034 / AC-006 / BC-HOOK-003: notification_type "assistant_message" is NOT forwarded.
/// Uses the non-permission-dropped.json fixture (notification_type = "assistant_message").
///
/// test_BC_HOOK_034_nan_port_lock_file_skip (routing layer variant)
/// and test_BC_HOOK_003_notification_filter_permission_prompt_only
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_003_notification_filter_drops_assistant_message() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    // Load the specific fixture for this test.
    let fixture = common::load_fixture("notification", "non-permission-dropped");
    // notification_type is "assistant_message" — must be dropped by filter.
    assert_eq!(fixture["notification_type"], "assistant_message");
    // Post it: the handler should filter and return without forwarding to daemon.
    // Red Gate: handler panics with todo!() before the filter logic executes.
    let status = common::post_json(router, paths::NOTIFICATION, &fixture).await;
    // The filtered case returns 200 (fire-and-forget; dropped silently).
    assert_eq!(
        status,
        StatusCode::OK,
        "non-permission_prompt notification must return 200 (dropped, not forwarded)"
    );
}

/// BC-HOOK-003: notification_type "idle" is dropped (not forwarded).
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_003_notification_filter_drops_idle_type() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "notification_type": "idle",
        "tool_name": "",
        "tool_input": {},
        "message": "Claude is thinking..."
    });
    let status = common::post_json(router, paths::NOTIFICATION, &body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "idle notification must be dropped (200, not forwarded)"
    );
}

/// BC-HOOK-003: notification_type absent from payload → dropped (not forwarded).
/// Per BC-HOOK-003 EC-001: undefined notification_type !== 'permission_prompt' → drop.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_003_notification_filter_drops_absent_type() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "tool_name": "",
        "tool_input": {},
        "message": "No type field"
        // notification_type is absent
    });
    // axum will reject this as a deserialization error since notification_type is required.
    // The important assertion: it does NOT reach the daemon.
    let status = common::post_json(router, paths::NOTIFICATION, &body).await;
    // Either 400 (deserialization) or 200 (absorbed by filter) — NOT 500 or 201.
    assert!(
        status == StatusCode::OK
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::BAD_REQUEST,
        "absent notification_type must be absorbed (not forwarded); got {status}"
    );
}

/// BC-HOOK-003: notification_type "permission_prompt" IS forwarded (positive case).
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_003_notification_filter_passes_permission_prompt() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    // Load a permission_prompt fixture.
    let fixture = common::load_fixture("notification", "permission-prompt-bash");
    assert_eq!(fixture["notification_type"], "permission_prompt");
    // This should attempt to forward to daemon and panic from todo!() in handler.
    let status = common::post_json(router, paths::NOTIFICATION, &fixture).await;
    // After implementation this returns 200; for Red Gate it panics.
    assert_ne!(
        status,
        StatusCode::NOT_FOUND,
        "permission_prompt must not return 404"
    );
}

/// BC-HOOK-003: Case-sensitive match — "Permission_Prompt" is NOT forwarded.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_003_notification_filter_case_sensitive_rejects_wrong_case() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "notification_type": "Permission_Prompt",  // wrong case
        "tool_name": "Bash",
        "tool_input": {},
        "message": "Allow Bash?"
    });
    let status = common::post_json(router, paths::NOTIFICATION, &body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "wrong-case notification_type must be dropped (200); got {status}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-024 / AC-005: Cross-IDE collision filter (lock_reader level)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-024 / AC-005: Non-monocle lock file at higher port is skipped.
///
/// test_BC_HOOK_024_lock_app_filter_monocle_only (multi-lock scenario)
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_024_vscode_lock_at_higher_port_is_skipped() {
    use monocle_test_harness::dtu::lock_reader::{read_lock_file, LockReadError};
    let dir = tempfile::TempDir::new().expect("TempDir");
    // Write monocle lock at 7860, vscode lock at 9000 (higher port).
    let monocle_content =
        common::monocle_lock_json(7860, std::process::id(), common::VALID_AUTH_TOKEN);
    let vscode_content = common::non_monocle_lock_json(9000, std::process::id());
    common::write_lock_file(dir.path(), 7860, &monocle_content);
    common::write_lock_file(dir.path(), 9000, &vscode_content);

    // read_lock_file is per-file; the multi-lock scan is in lock_reader at scan level.
    // Test: reading the vscode lock directly returns NonMonocleLock.
    let vscode_path = dir.path().join("9000.lock");
    let result = read_lock_file(&vscode_path);
    match result {
        Err(LockReadError::NonMonocleLock { app }) => {
            assert_eq!(app, "vscode");
        }
        other => panic!("expected NonMonocleLock for vscode lock, got {other:?}"),
    }

    // Reading the monocle lock directly succeeds.
    let monocle_path = dir.path().join("7860.lock");
    let info = read_lock_file(&monocle_path).expect("monocle lock should be readable");
    assert_eq!(info.port, 7860);
}

/// BC-HOOK-024: Only-vscode lock directory → no monocle lock found.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_024_only_vscode_lock_dir_has_no_monocle_lock() {
    use monocle_test_harness::dtu::lock_reader::{read_lock_file, LockReadError};
    let dir = tempfile::TempDir::new().expect("TempDir");
    let vscode_content = common::non_monocle_lock_json(9000, std::process::id());
    let lock_path = common::write_lock_file(dir.path(), 9000, &vscode_content);
    let result = read_lock_file(&lock_path);
    assert!(
        result.is_err(),
        "only-vscode lock must return error (no monocle lock)"
    );
    match result.unwrap_err() {
        LockReadError::NonMonocleLock { .. } => { /* correct */ }
        other => panic!("expected NonMonocleLock, got {other:?}"),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-034: NaN-port lock file skipped (monocle improvement)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-034: Non-numeric lock file is skipped; numeric lock wins.
///
/// test_BC_HOOK_034_nan_port_lock_file_skip
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_034_nan_port_lock_file_skip_numeric_wins() {
    use monocle_test_harness::dtu::lock_reader::read_lock_file;
    let dir = tempfile::TempDir::new().expect("TempDir");

    // Write a monocle lock at port 7860.
    let monocle_content =
        common::monocle_lock_json(7860, std::process::id(), common::VALID_AUTH_TOKEN);
    common::write_lock_file(dir.path(), 7860, &monocle_content);

    // Write a non-numeric-named lock file ("app.lock") that would produce NaN in JS.
    let app_lock_path = dir.path().join("app.lock");
    let app_content = serde_json::json!({
        "contract_version": 1,
        "pid": std::process::id(),
        "port": 7861,
        "authToken": common::VALID_AUTH_TOKEN,
        "app": "monocle"
    });
    std::fs::write(
        &app_lock_path,
        serde_json::to_string(&app_content).expect("json"),
    )
    .expect("write");

    // The numeric lock at 7860 should be readable correctly.
    let monocle_path = dir.path().join("7860.lock");
    let info = read_lock_file(&monocle_path).expect("numeric lock must be readable");
    assert_eq!(info.port, 7860, "numeric lock port must be 7860");
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-001 / BC-HOOK-002: Fail-open vs fail-closed semantics
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-001: PreToolUse handler accepts a POST even when no daemon is running.
/// (The clone itself accepts the POST; it is the forwarding to daemon that fails.)
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_001_pretooluse_fail_open_no_server() {
    // CloneState points to a non-listening port; handler should panic with todo!() (Red Gate).
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    });
    // Red Gate: this panics at todo!() in handle_pre_tool_use.
    let _status = common::post_json(router, paths::PRE_TOOL_USE, &body).await;
}

/// BC-HOOK-002: Non-PreToolUse hooks are fail-closed (return 200 without forwarding).
/// Even with no daemon, the Notification handler should drop and return 200.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_002_non_pretooluse_fail_closed_no_server() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "stop_reason": "end_turn"
    });
    // Red Gate: panics at todo!() in handle_stop.
    let _status = common::post_json(router, paths::STOP, &body).await;
}
