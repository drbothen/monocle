// BC-HOOK-NNN naming preserves traceability to canonical BC IDs (.factory/specs/behavioral-contracts/ss-dtu/).
// Clippy's non_snake_case lint is suppressed at module level rather than renaming to lower-case
// identifiers that would lose that traceability signal.
#![allow(non_snake_case)]
//! Integration tests for the 5 DTU clone hook endpoints — AC-001.
//!
//! Verifies:
//! - POST is accepted on each of the 5 canonical paths
//! - GET / other methods return 405 Method Not Allowed
//! - Path strings exactly match dtu-assessment.md §Endpoint matrix
//!
//! Source authority:
//! - AC-001 (5 endpoints implemented)
//! - BC-HOOK-007 (exactly five hook types)
//! - BC-HOOK-019 (monocle canonical endpoints, not gene-source paths)
//! - dtu-assessment.md §Endpoint matrix (implemented against v1.7.5 at S-DTU-001 authoring time)

#[path = "common/mod.rs"]
mod common;

use axum::http::StatusCode;
use monocle_test_harness::dtu::endpoints::{build_router, paths};

// ──────────────────────────────────────────────────────────────────────────────
// AC-001 / BC-HOOK-007: POST accepted on all 5 canonical paths
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-007 / AC-001: POST /hooks/pre-tool-use is registered and accepted.
///
/// test_BC_HOOK_007_endpoint_pre_tool_use_accepts_post
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_endpoint_pre_tool_use_accepts_post() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    });
    // Route must be registered; handler processes the request (BC-HOOK-007 AC-001).
    let status = common::post_json(router, paths::PRE_TOOL_USE, &body).await;
    assert_ne!(status, StatusCode::NOT_FOUND, "route must exist");
}

/// BC-HOOK-007 / AC-001: POST /hooks/notification is registered and accepted.
///
/// test_BC_HOOK_007_endpoint_notification_accepts_post
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_endpoint_notification_accepts_post() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "notification_type": "permission_prompt",
        "tool_name": "Bash",
        "tool_input": {},
        "message": "Allow Bash?"
    });
    let status = common::post_json(router, paths::NOTIFICATION, &body).await;
    assert_ne!(status, StatusCode::NOT_FOUND);
}

/// BC-HOOK-007 / AC-001: POST /hooks/stop is registered and accepted.
///
/// test_BC_HOOK_007_endpoint_stop_accepts_post
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_endpoint_stop_accepts_post() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "stop_reason": "end_turn"
    });
    let status = common::post_json(router, paths::STOP, &body).await;
    assert_ne!(status, StatusCode::NOT_FOUND);
}

/// BC-HOOK-007 / AC-001: POST /hooks/session-start is registered and accepted.
///
/// test_BC_HOOK_007_endpoint_session_start_accepts_post
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_endpoint_session_start_accepts_post() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "cwd": "/tmp/project",
        "transcript_path": "/home/user/.claude/transcript.json"
    });
    let status = common::post_json(router, paths::SESSION_START, &body).await;
    assert_ne!(status, StatusCode::NOT_FOUND);
}

/// BC-HOOK-007 / AC-001: POST /hooks/prompt-submit is registered and accepted.
///
/// test_BC_HOOK_007_endpoint_prompt_submit_accepts_post
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_endpoint_prompt_submit_accepts_post() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "prompt": "Please refactor the main function"
    });
    let status = common::post_json(router, paths::PROMPT_SUBMIT, &body).await;
    assert_ne!(status, StatusCode::NOT_FOUND);
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-007: Method enforcement — GET must return 405 on all 5 paths
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-007: GET /hooks/pre-tool-use returns 405 Method Not Allowed.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_get_pre_tool_use_returns_405() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let status = common::get(router, paths::PRE_TOOL_USE).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

/// BC-HOOK-007: GET /hooks/notification returns 405 Method Not Allowed.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_get_notification_returns_405() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let status = common::get(router, paths::NOTIFICATION).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

/// BC-HOOK-007: GET /hooks/stop returns 405 Method Not Allowed.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_get_stop_returns_405() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let status = common::get(router, paths::STOP).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

/// BC-HOOK-007: GET /hooks/session-start returns 405 Method Not Allowed.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_get_session_start_returns_405() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let status = common::get(router, paths::SESSION_START).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

/// BC-HOOK-007: GET /hooks/prompt-submit returns 405 Method Not Allowed.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_get_prompt_submit_returns_405() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let status = common::get(router, paths::PROMPT_SUBMIT).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-007: No PostToolUse endpoint exists in Phase 1
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-007: /hooks/post-tool-use does NOT exist (returns 404).
/// PostToolUse is explicitly absent from Phase 1 per the spec.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_post_tool_use_absent_returns_404() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body =
        serde_json::json!({"session_id": "x", "pid": 1, "tool_name": "Bash", "tool_input": {}});
    let status = common::post_json(router, "/hooks/post-tool-use", &body).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-019: Monocle canonical paths are NOT the gene-source paths
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-019: Gene-source path /notify does NOT exist (monocle uses /hooks/notification).
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_019_monocle_canonical_endpoints_not_gene_source() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let body = serde_json::json!({});
    // Gene-source paths must NOT be registered
    let s1 = common::post_json(router.clone(), "/notify", &body).await;
    let s2 = common::post_json(router.clone(), "/stop", &body).await;
    let s3 = common::post_json(router.clone(), "/session-start", &body).await;
    let s4 = common::post_json(router.clone(), "/prompt-submit", &body).await;
    let s5 = common::post_json(router, "/pre-tool-use", &body).await;
    assert_eq!(s1, StatusCode::NOT_FOUND, "/notify must not exist");
    assert_eq!(s2, StatusCode::NOT_FOUND, "/stop must not exist");
    assert_eq!(s3, StatusCode::NOT_FOUND, "/session-start must not exist");
    assert_eq!(s4, StatusCode::NOT_FOUND, "/prompt-submit must not exist");
    assert_eq!(s5, StatusCode::NOT_FOUND, "/pre-tool-use must not exist");
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-007: DELETE also returns 405 (not just GET)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-007: DELETE /hooks/pre-tool-use returns 405 (only POST accepted).
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_delete_pre_tool_use_returns_405() {
    let state = common::make_test_clone_state();
    let router = build_router(state);
    let status = common::delete(router, paths::PRE_TOOL_USE).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}
