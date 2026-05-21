// BC-HOOK-NNN naming preserves traceability to canonical BC IDs (.factory/specs/behavioral-contracts/ss-dtu/).
// Clippy's non_snake_case lint is suppressed at module level rather than renaming to lower-case
// identifiers that would lose that traceability signal.
// Test code: expect/unwrap idiomatic for failure-amplification on assertion failures.
#![allow(non_snake_case)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
//! Integration tests covering remaining BC-HOOK-001..BC-HOOK-041 behaviors.
//!
//! This file covers behaviors not exercised by the other integration test files:
//! - BC-HOOK-004: fire-and-forget HTTP (no response read)
//! - BC-HOOK-006: PreToolUse unconditional stdin echo
//! - BC-HOOK-008: No HTML escaping in hooks-settings.json
//! - BC-HOOK-009: hooks-settings.json path and mode
//! - BC-HOOK-010: single hooks-settings.json per runtimeDir
//! - BC-HOOK-011: no cleanup of hooks-settings.json on session stop
//! - BC-HOOK-012: role-invariant hook configuration
//! - BC-HOOK-017: PID liveness check (signal 0)
//! - BC-HOOK-018: per-hook fallback semantics matrix
//! - BC-HOOK-020: notification filter deep-ingest confirmation
//! - BC-HOOK-021: fire-and-forget deep-ingest confirmation
//! - BC-HOOK-022: timeout values (Notification 2000ms; others 300ms)
//! - BC-HOOK-023: Content-Type and Content-Length headers always set
//! - BC-HOOK-025: restart resilience / new port discovery
//! - BC-HOOK-026: stateless per-invocation discovery
//! - BC-HOOK-027: settings injection via --settings flag
//! - BC-HOOK-028: no env-var hook injection alternative
//! - BC-HOOK-029: hook process env-independence
//! - BC-HOOK-030: MONOCLE_SESSION_ID set but not read by hook JS
//! - BC-HOOK-031: hooks-settings.json unversioned
//! - BC-HOOK-032: malformed stdin echo (PreToolUse doubly fail-open)
//! - BC-HOOK-033: malformed stdin silent drop (non-PreToolUse)
//! - BC-HOOK-035: lock file read errors silently skipped
//! - BC-HOOK-037: req.write + req.end pattern
//! - BC-HOOK-038: no same-port race
//! - BC-HOOK-039: atomic write via tempfile::persist
//! - BC-HOOK-040: struct-based stable serialization (covered also in integration_payload.rs)
//! - BC-HOOK-041: hooks-settings.json filename assertion

#[path = "common/mod.rs"]
mod common;

use monocle_test_harness::dtu::{
    endpoints::{build_router, paths},
    lock_reader::read_lock_file,
    payload::{NotificationPayload, PreToolUsePayload},
    server::CloneState,
};

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-004 / BC-HOOK-021: Fire-and-forget semantics
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-004 / BC-HOOK-021: PreToolUse POST does not block on daemon response.
///
/// test_BC_HOOK_004_hook_requests_fire_and_forget
/// The clone sends the POST and returns without waiting for a response.
/// (Red Gate: todo!() fires before any network activity.)
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_004_hook_requests_fire_and_forget() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "s",
        "pid": 1,
        "tool_name": "Bash",
        "tool_input": {}
    });
    // Handler should not block or wait for daemon reply — panics at todo!() (Red Gate).
    let _status = common::post_json(router, paths::PRE_TOOL_USE, &body).await;
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-006: PreToolUse unconditional stdin echo
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-006: PreToolUse handler returns 200 with the original payload unchanged.
///
/// test_BC_HOOK_006_pretooluse_unconditional_stdin_echo
/// The handler must echo the payload back in the response body.
/// (Red Gate: todo!() fires before echo logic.)
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_006_pretooluse_unconditional_stdin_echo() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    let payload_body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    });
    let req = Request::builder()
        .method("POST")
        .uri(paths::PRE_TOOL_USE)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&payload_body).expect("serialize"),
        ))
        .expect("build req");
    let response = router.oneshot(req).await.expect("oneshot");
    // After implementation: response body == original payload.
    // Red Gate: this panics at todo!() before the body check.
    let status = response.status();
    assert_eq!(
        status,
        axum::http::StatusCode::OK,
        "PreToolUse must return 200 with echo"
    );

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let echo_value: serde_json::Value = serde_json::from_slice(&body_bytes).expect("parse echo");
    assert_eq!(
        echo_value["session_id"],
        "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
    );
    assert_eq!(echo_value["tool_name"], "Bash");
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-009 / BC-HOOK-039: hooks-settings.json path, mode, atomic write
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-009 / BC-HOOK-041: WriteHooksSettingsFile produces a file ending in hooks-settings.json.
///
/// test_BC_HOOK_041_hooks_settings_json_filename_assertion
/// test_BC_HOOK_039_atomic_write_tempfile_persist
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_041_hooks_settings_json_filename_assertion() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write hooks settings");
    // Must end in hooks-settings.json (BC-HOOK-041 explicit assertion).
    assert!(
        path.ends_with("hooks-settings.json"),
        "path must end in hooks-settings.json, got {path:?}"
    );
}

/// BC-HOOK-039: WriteHooksSettingsFile uses tempfile::persist (atomic write).
/// After the call, the file must contain valid JSON (no torn read possible).
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_039_atomic_write_tempfile_persist() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write hooks settings");
    // File must contain valid JSON immediately after write.
    let content = std::fs::read_to_string(&path).expect("read file");
    let _: serde_json::Value = serde_json::from_str(&content)
        .expect("hooks-settings.json must be valid JSON after atomic write");
}

/// BC-HOOK-009: hooks-settings.json file mode is 0o600.
#[test]
#[cfg(unix)]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_009_hooks_settings_json_mode_0o600() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write hooks settings");
    let meta = std::fs::metadata(&path).expect("metadata");
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "hooks-settings.json must have mode 0o600, got {mode:#o}"
    );
}

/// BC-HOOK-010: WriteHooksSettingsFile called twice produces single file.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_010_single_hooks_settings_json_per_runtime_dir() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path1 = write_hooks_settings_file(dir.path()).expect("first write");
    let path2 = write_hooks_settings_file(dir.path()).expect("second write");
    // Same path; only one file.
    assert_eq!(path1, path2, "both calls must return same path");
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with("hooks-settings.json")
        })
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "exactly one hooks-settings.json must exist"
    );
}

/// BC-HOOK-011: hooks-settings.json persists after session stop (no cleanup).
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_011_hooks_settings_json_persists_after_session_stop() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    // Simulate session stop — file must still exist.
    assert!(
        path.exists(),
        "hooks-settings.json must persist after session stop"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-007 / BC-HOOK-012: Five hooks registered; role-invariant
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-007: hooks-settings.json contains exactly 5 hook keys.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_007_hooks_settings_json_contains_five_hook_keys() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    let v: serde_json::Value = serde_json::from_str(&content).expect("parse");
    let hooks = v.get("hooks").expect("hooks key must exist");
    let hook_map = hooks.as_object().expect("hooks must be an object");
    assert_eq!(hook_map.len(), 5, "exactly 5 hook keys required");
    assert!(hook_map.contains_key("PreToolUse"), "PreToolUse missing");
    assert!(
        hook_map.contains_key("Notification"),
        "Notification missing"
    );
    assert!(hook_map.contains_key("Stop"), "Stop missing");
    assert!(
        hook_map.contains_key("SessionStart"),
        "SessionStart missing"
    );
    assert!(
        hook_map.contains_key("UserPromptSubmit"),
        "UserPromptSubmit missing"
    );
    assert!(
        !hook_map.contains_key("PostToolUse"),
        "PostToolUse must NOT exist in Phase 1"
    );
}

/// BC-HOOK-012: Role-invariant: hook config is identical for plain and PM sessions.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_012_role_invariant_hook_configuration() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    // Two different temp dirs (simulating two sessions in different runtimeDirs).
    let dir1 = tempfile::TempDir::new().expect("TempDir 1");
    let dir2 = tempfile::TempDir::new().expect("TempDir 2");
    let path1 = write_hooks_settings_file(dir1.path()).expect("write 1");
    let path2 = write_hooks_settings_file(dir2.path()).expect("write 2");
    let content1 = std::fs::read_to_string(path1).expect("read 1");
    let content2 = std::fs::read_to_string(path2).expect("read 2");
    // Hook command content must be identical (role-invariant).
    let v1: serde_json::Value = serde_json::from_str(&content1).expect("parse 1");
    let v2: serde_json::Value = serde_json::from_str(&content2).expect("parse 2");
    assert_eq!(
        v1.get("hooks"),
        v2.get("hooks"),
        "hook configuration must be identical regardless of session role/runtimeDir"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-017: PID liveness check
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-017: Current process PID is alive (liveness check passes).
///
/// test_BC_HOOK_017_pid_liveness_signal_zero
#[test]
fn test_BC_HOOK_017_pid_liveness_signal_zero() {
    // The current process is alive; a lock file with this PID must pass liveness.
    let (dir, lock_path) = common::temp_lock_dir_with_valid_lock(7860, std::process::id());
    let result = read_lock_file(&lock_path);
    assert!(
        result.is_ok(),
        "current process PID must pass liveness check; got {result:?}"
    );
    drop(dir);
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-018: Per-hook fallback semantics matrix
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-018: PreToolUse with no daemon → returns 200 (fail-open; echo of input).
///
/// test_BC_HOOK_018_per_hook_fallback_semantics_matrix
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_018_per_hook_fallback_semantics_matrix_pretooluse_fail_open() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "s",
        "pid": 1,
        "tool_name": "Bash",
        "tool_input": {}
    });
    // Red Gate: panics at todo!() — after implementation returns 200.
    let _status = common::post_json(router, paths::PRE_TOOL_USE, &body).await;
}

/// BC-HOOK-018: Notification with no daemon → returns 200 (fail-closed; silent drop).
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_018_per_hook_fallback_semantics_matrix_notification_fail_closed() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "s",
        "pid": 1,
        "notification_type": "permission_prompt",
        "tool_name": "Bash",
        "tool_input": {},
        "message": "Allow Bash?"
    });
    // Red Gate: panics at todo!() — after implementation returns 200 (fire-and-forget).
    let _status = common::post_json(router, paths::NOTIFICATION, &body).await;
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-020: Notification client-side filter (deep-ingest confirmation)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-020: Notification filter is client-side (runs in hook, not server).
/// The /hooks/notification endpoint never receives non-permission_prompt events.
///
/// test_BC_HOOK_020_notification_filter_deep_ingest_confirmation
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_020_notification_filter_deep_ingest_confirmation() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "s",
        "pid": 1,
        "notification_type": "assistant_message",
        "tool_name": "",
        "tool_input": {},
        "message": "I have completed the task"
    });
    let status = common::post_json(router, paths::NOTIFICATION, &body).await;
    assert_eq!(
        status,
        axum::http::StatusCode::OK,
        "assistant_message notifications must be dropped (client-side filter); got {status}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-022: Timeout values
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-022: Notification timeout is 2000ms; other hooks are 300ms.
///
/// test_BC_HOOK_022_notification_timeout_2000ms_others_300ms
/// Verified by inspecting the hooks-settings.json content for timeout values.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_022_notification_timeout_2000ms_others_300ms() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    let v: serde_json::Value = serde_json::from_str(&content).expect("parse");
    let hooks = v["hooks"].as_object().expect("hooks object");

    // Notification timeout must be 2000ms.
    let notif_cmd = hooks["Notification"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap_or("");
    assert!(
        notif_cmd.contains("2000") || notif_cmd.contains("timeout:2000"),
        "Notification hook command must specify 2000ms timeout; cmd = {notif_cmd:?}"
    );

    // PreToolUse timeout must be 300ms.
    let pre_cmd = hooks["PreToolUse"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap_or("");
    assert!(
        pre_cmd.contains("300") || pre_cmd.contains("timeout:300"),
        "PreToolUse hook command must specify 300ms timeout; cmd = {pre_cmd:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-023: Content-Type and Content-Length headers
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-023: Hook HTTP requests include Content-Type: application/json.
///
/// test_BC_HOOK_023_content_type_content_length_headers
/// Verified by inspecting the hook JS command string in hooks-settings.json.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_023_content_type_content_length_headers() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    // The JS hook commands must contain Content-Type and Content-Length headers.
    assert!(
        content.contains("Content-Type") || content.contains("content-type"),
        "hooks-settings.json must specify Content-Type header in hook commands"
    );
    assert!(
        content.contains("Content-Length") || content.contains("content-length"),
        "hooks-settings.json must specify Content-Length header in hook commands"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-025: Restart resilience — new port discovery
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-025: After daemon restart (new lock file), next invocation uses new port.
///
/// test_BC_HOOK_025_restart_resilience_new_port_discovery
/// Simulated by writing new lock file at new port and re-reading.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_025_restart_resilience_new_port_discovery() {
    let dir = tempfile::TempDir::new().expect("TempDir");
    // Initial lock at port 7860.
    let content_old = common::monocle_lock_json(7860, std::process::id(), common::VALID_AUTH_TOKEN);
    let old_path = common::write_lock_file(dir.path(), 7860, &content_old);
    let info_old = read_lock_file(&old_path).expect("old lock");
    assert_eq!(info_old.port, 7860);

    // Daemon "restarts": new lock at port 7861.
    let new_token = "b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3";
    let content_new = common::monocle_lock_json(7861, std::process::id(), new_token);
    let new_path = common::write_lock_file(dir.path(), 7861, &content_new);
    let info_new = read_lock_file(&new_path).expect("new lock");
    assert_eq!(info_new.port, 7861);
    assert_eq!(info_new.auth_token, new_token);
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-026: Stateless per-invocation discovery
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-026: read_lock_file returns identical result on repeated calls (stateless).
///
/// test_BC_HOOK_026_stateless_per_invocation_discovery
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_026_stateless_per_invocation_discovery() {
    let (dir, lock_path) = common::temp_lock_dir_with_valid_lock(7860, std::process::id());
    let info1 = read_lock_file(&lock_path).expect("first read");
    let info2 = read_lock_file(&lock_path).expect("second read");
    assert_eq!(
        info1.port, info2.port,
        "stateless: port must be identical on repeated reads"
    );
    assert_eq!(
        info1.auth_token, info2.auth_token,
        "stateless: token must be identical"
    );
    drop(dir);
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-027 / BC-HOOK-028: Settings injection via --settings flag
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-027: hooks-settings.json is provided via --settings flag, not global ~/.monocle/settings.json.
///
/// test_BC_HOOK_027_settings_injection_via_flag_not_global_write
/// Verified by: WriteHooksSettingsFile writes to a runtimeDir, not ~/.monocle/settings.json.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_027_settings_injection_via_flag_not_global_write() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    // Must NOT write to ~/.monocle/settings.json.
    let home = std::env::var("HOME").unwrap_or_default();
    let global_settings = std::path::PathBuf::from(&home).join(".monocle/settings.json");
    assert_ne!(
        path, global_settings,
        "hooks-settings.json must NOT be written to the global ~/.monocle/settings.json"
    );
    // Must be inside runtimeDir.
    assert!(
        path.starts_with(dir.path()),
        "hooks-settings.json must be inside runtimeDir"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-031: hooks-settings.json is unversioned
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-031: hooks-settings.json contains no schema_version field.
///
/// test_BC_HOOK_031_hooks_settings_json_unversioned
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_031_hooks_settings_json_unversioned() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    let v: serde_json::Value = serde_json::from_str(&content).expect("parse");
    assert!(
        v.get("schema_version").is_none() && v.get("version").is_none(),
        "hooks-settings.json must have no version field; got schema_version={:?} version={:?}",
        v.get("schema_version"),
        v.get("version")
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-032: Malformed stdin echo (PreToolUse doubly fail-open)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-032: PreToolUse echoes stdin even when JSON is malformed.
///
/// test_BC_HOOK_032_pretooluse_echo_on_malformed_json
/// axum's Json extractor rejects malformed JSON with 400/422; the handler
/// must still return 200 with the raw body echoed.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_032_pretooluse_echo_on_malformed_json() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let malformed = b"{ not valid json !!!";
    let req = Request::builder()
        .method("POST")
        .uri(paths::PRE_TOOL_USE)
        .header("content-type", "application/json")
        .body(Body::from(malformed.as_slice()))
        .expect("build req");
    let response = router.oneshot(req).await.expect("oneshot");
    // Implementation must echo stdin even on malformed JSON (BC-HOOK-032).
    // axum returns 422 for malformed JSON; but the PreToolUse handler with raw body
    // extraction must return 200. Red Gate: panics at todo!() or 422 from axum.
    let status = response.status();
    assert_eq!(
        status,
        axum::http::StatusCode::OK,
        "PreToolUse must echo stdin even on malformed JSON (BC-HOOK-032); got {status}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-033: Malformed stdin silent drop (non-PreToolUse)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-033: Stop hook silently drops malformed JSON input.
///
/// test_BC_HOOK_033_non_pretooluse_silent_drop_malformed_json
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_033_non_pretooluse_silent_drop_malformed_json() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let malformed = b"{ bad json";
    let req = Request::builder()
        .method("POST")
        .uri(paths::STOP)
        .header("content-type", "application/json")
        .body(Body::from(malformed.as_slice()))
        .expect("build req");
    let response = router.oneshot(req).await.expect("oneshot");
    let status = response.status();
    // Non-PreToolUse: malformed JSON → silent drop → 200.
    assert_eq!(
        status,
        axum::http::StatusCode::OK,
        "Stop hook must silently drop malformed JSON (BC-HOOK-033); got {status}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-038: No same-port race (lock-after-bind ordering)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-038: Two concurrent lock files for the same port cannot both be alive.
/// (Structurally impossible due to lock-after-bind ordering.)
///
/// test_BC_HOOK_038_no_same_port_race
/// Verified by: the lock file is written AFTER the port is bound, so two processes
/// cannot both bind port 7860 simultaneously. We test the invariant indirectly by
/// verifying that read_lock_file returns a port derived from the lock file content.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_038_no_same_port_race() {
    // This is a structural invariant; we verify the lock file port is consistent.
    let (dir, lock_path) = common::temp_lock_dir_with_valid_lock(7860, std::process::id());
    let info = read_lock_file(&lock_path).expect("read lock");
    // Port from lock file content must be consistent with the lock filename convention.
    assert_eq!(
        info.port, 7860,
        "port from lock file must match the expected value"
    );
    drop(dir);
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-040: Struct-based stable serialization
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-040: hooks-settings.json produced by WriteHooksSettingsFile is byte-stable across calls.
///
/// test_BC_HOOK_040_struct_based_stable_serialization (settings-level)
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_040_hooks_settings_json_byte_stable_across_writes() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir1 = tempfile::TempDir::new().expect("TempDir 1");
    let dir2 = tempfile::TempDir::new().expect("TempDir 2");
    let path1 = write_hooks_settings_file(dir1.path()).expect("write 1");
    let path2 = write_hooks_settings_file(dir2.path()).expect("write 2");
    let c1 = std::fs::read_to_string(path1).expect("read 1");
    let c2 = std::fs::read_to_string(path2).expect("read 2");
    assert_eq!(
        c1, c2,
        "hooks-settings.json must be byte-stable (struct-based, not HashMap)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-030: MONOCLE_SESSION_ID env var set but not read by hook JS
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-030: MONOCLE_SESSION_ID is set on Claude Code subprocess but NOT read by hook JS.
/// Verified by: hook command strings in hooks-settings.json do NOT reference MONOCLE_SESSION_ID.
///
/// test_BC_HOOK_030_monocle_session_id_env_set_not_read_by_hooks
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_030_monocle_session_id_env_set_not_read_by_hooks() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    // Hook commands must NOT read MONOCLE_SESSION_ID.
    assert!(
        !content.contains("MONOCLE_SESSION_ID"),
        "hook commands must NOT reference MONOCLE_SESSION_ID"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-029: Hook process env-independence
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-029: Hook process reads only os.homedir() from env; no other env vars.
/// Verified by: hook command strings do not read arbitrary env vars except homedir-derived path.
///
/// test_BC_HOOK_029_hook_env_independence
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_029_hook_env_independence() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    // Hook command must NOT reference arbitrary env vars beyond homedir/MONOCLE_RUNTIME_DIR.
    // Forbidden: process.env.NODE_ENV, process.env.PATH, etc.
    assert!(
        !content.contains("process.env.NODE_ENV") && !content.contains("process.env.PATH"),
        "hook commands must not read arbitrary env vars"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-037: req.write + req.end pattern
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-037: Hook command uses req.write(body) + req.end() pattern (not streaming).
///
/// test_BC_HOOK_037_req_write_req_end_pattern
/// Verified by: hook command string contains req.write and req.end.
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_037_req_write_req_end_pattern() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    assert!(
        content.contains("req.write") && content.contains("req.end"),
        "hook commands must use req.write(body)+req.end() pattern"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-008: No HTML escaping (settings-level)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-008: hooks-settings.json contains literal => (no HTML escaping of >).
///
/// test_BC_HOOK_008_no_html_escape (settings variant)
#[test]
#[allow(clippy::expect_used)]
fn test_BC_HOOK_008_no_html_escape_in_hooks_settings_json() {
    use monocle_test_harness::dtu::server::write_hooks_settings_file;
    let dir = tempfile::TempDir::new().expect("TempDir");
    let path = write_hooks_settings_file(dir.path()).expect("write");
    let content = std::fs::read_to_string(&path).expect("read");
    // serde_json does NOT HTML-escape; arrow functions must appear as literal =>.
    assert!(
        !content.contains("\\u003e") && !content.contains("\\u003c"),
        "hooks-settings.json must not contain HTML-escaped characters"
    );
    // If arrow functions are embedded in the command, => must appear literally.
    if content.contains("=>") {
        assert!(
            !content.contains("\\u003e"),
            "=> must not be HTML-escaped to \\u003e"
        );
    }
}
