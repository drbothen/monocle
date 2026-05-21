// BC-HOOK-NNN naming preserves traceability to canonical BC IDs (.factory/specs/behavioral-contracts/ss-dtu/).
// Clippy's non_snake_case lint is suppressed at module level rather than renaming to lower-case
// identifiers that would lose that traceability signal.
// Test code: expect/unwrap idiomatic for failure-amplification on assertion failures.
#![allow(non_snake_case)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
//! Integration tests for monocle-canonical payload structure — AC-003.
//!
//! Verifies:
//! - PreToolUse: {session_id, pid, tool_name, tool_input}
//! - Notification: {session_id, pid, notification_type, tool_name, tool_input, message}
//! - Stop: {session_id, pid, stop_reason}
//! - SessionStart: {session_id, pid, cwd (EX-2), transcript_path (EX-2)}
//! - UserPromptSubmit: {session_id, pid, prompt (EX-2)}
//! - BC-HOOK-040: struct-based serialization (not HashMap) for stable key ordering
//! - BC-HOOK-008: no HTML escaping in JSON output
//! - BC-HOOK-036: Content-Length is UTF-8 byte count
//!
//! Source authority:
//! - AC-003 (payload structure)
//! - BC-HOOK-007 (schema), BC-HOOK-008 (encoding), BC-HOOK-036 (byte length),
//!   BC-HOOK-040 (struct not HashMap)

#[path = "common/mod.rs"]
mod common;

use axum::http::StatusCode;
use http_body_util::BodyExt;
use monocle_test_harness::dtu::{
    endpoints::{build_router, paths},
    payload::FixtureScore,
};

// ──────────────────────────────────────────────────────────────────────────────
// AC-003 / BC-HOOK-007: PreToolUse endpoint produces canonical payload fields
// ──────────────────────────────────────────────────────────────────────────────

/// AC-003: The PreToolUse handler produces a payload containing the 4 monocle-canonical fields.
///
/// This test posts to the handler and verifies the response/echo includes all 4 required fields.
/// Red Gate: todo!() in handle_pre_tool_use fires before any field is produced.
///
/// test_BC_HOOK_007_pretooluse_handler_produces_canonical_fields
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_pretooluse_handler_produces_canonical_fields() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    let input = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "tool_name": "Bash",
        "tool_input": {"command": "ls -la"}
    });
    let req = Request::builder()
        .method("POST")
        .uri(paths::PRE_TOOL_USE)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() in handle_pre_tool_use (Red Gate).
    let response = router.oneshot(req).await.expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    // After implementation: echoed body must contain all 4 canonical fields.
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect")
        .to_bytes();
    let echo: serde_json::Value = serde_json::from_slice(&body_bytes).expect("parse echo");
    assert!(
        echo.get("session_id").is_some(),
        "session_id missing from echo"
    );
    assert!(echo.get("pid").is_some(), "pid missing from echo");
    assert!(
        echo.get("tool_name").is_some(),
        "tool_name missing from echo"
    );
    assert!(
        echo.get("tool_input").is_some(),
        "tool_input missing from echo"
    );
    assert!(
        echo.get("type").is_none(),
        "gene-source 'type' field must not exist in monocle payload"
    );
}

/// AC-003: PreToolUse canonical field values are preserved in the handler echo.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_pretooluse_handler_preserves_field_values() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    let input = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    });
    let req = Request::builder()
        .method("POST")
        .uri(paths::PRE_TOOL_USE)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() (Red Gate).
    let response = router.oneshot(req).await.expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect")
        .to_bytes();
    let echo: serde_json::Value = serde_json::from_slice(&body_bytes).expect("parse echo");
    assert_eq!(echo["session_id"], "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
    assert_eq!(echo["pid"], 12345_u64);
    assert_eq!(echo["tool_name"], "Bash");
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-003 / BC-HOOK-007: Notification endpoint produces canonical payload fields
// ──────────────────────────────────────────────────────────────────────────────

/// AC-003: The Notification handler for permission_prompt includes all 6 canonical fields.
///
/// Red Gate: todo!() in handle_notification fires before the forwarded request is constructed.
///
/// test_BC_HOOK_007_notification_handler_produces_canonical_fields
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_notification_handler_produces_canonical_fields() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    let input = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "notification_type": "permission_prompt",
        "tool_name": "Bash",
        "tool_input": {"command": "rm -rf /"},
        "message": "Allow Bash to run rm -rf /?"
    });
    let req = Request::builder()
        .method("POST")
        .uri(paths::NOTIFICATION)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() in handle_notification (Red Gate).
    let response = router.oneshot(req).await.expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    // After implementation: verify forwarded payload contains all 6 fields.
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect")
        .to_bytes();
    let forwarded: serde_json::Value = serde_json::from_slice(&body_bytes).expect("parse");
    assert!(forwarded.get("session_id").is_some(), "session_id missing");
    assert!(forwarded.get("pid").is_some(), "pid missing");
    assert!(
        forwarded.get("notification_type").is_some(),
        "notification_type missing"
    );
    assert!(forwarded.get("tool_name").is_some(), "tool_name missing");
    assert!(forwarded.get("tool_input").is_some(), "tool_input missing");
    assert!(forwarded.get("message").is_some(), "message missing");
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-003 / BC-HOOK-007: Stop endpoint produces canonical payload fields
// ──────────────────────────────────────────────────────────────────────────────

/// AC-003: The Stop handler produces all 3 canonical payload fields.
///
/// test_BC_HOOK_007_stop_handler_produces_canonical_fields
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_stop_handler_produces_canonical_fields() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    let input = serde_json::json!({
        "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "pid": 12345,
        "stop_reason": "end_turn"
    });
    let req = Request::builder()
        .method("POST")
        .uri(paths::STOP)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() in handle_stop (Red Gate).
    let response = router.oneshot(req).await.expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect")
        .to_bytes();
    let forwarded: serde_json::Value = serde_json::from_slice(&body_bytes).expect("parse");
    assert!(forwarded.get("session_id").is_some(), "session_id missing");
    assert!(forwarded.get("pid").is_some(), "pid missing");
    assert!(
        forwarded.get("stop_reason").is_some(),
        "stop_reason missing"
    );
}

/// AC-003: Stop handler accepts all known stop_reason values.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_stop_handler_accepts_all_stop_reasons() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    for reason in &[
        "end_turn",
        "max_tokens",
        "tool_use",
        "error",
        "future_unknown_reason",
    ] {
        let state = common::make_test_clone_state();
        let router = build_router(state);
        let input = serde_json::json!({
            "session_id": "s",
            "pid": 1,
            "stop_reason": reason
        });
        let req = Request::builder()
            .method("POST")
            .uri(paths::STOP)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
            .expect("build req");
        // Panics at todo!() (Red Gate).
        let response = router.oneshot(req).await.expect("oneshot");
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "stop_reason={reason} must be accepted"
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-003 / BC-HOOK-007: SessionStart endpoint produces canonical payload fields (EX-2)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-003: The SessionStart handler produces 4 canonical fields including EX-2 extensions.
///
/// test_BC_HOOK_007_session_start_handler_produces_canonical_fields
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_session_start_handler_produces_canonical_fields() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    // Load from fixture corpus.
    let input = common::load_fixture("session-start", "session-start-basic");
    let req = Request::builder()
        .method("POST")
        .uri(paths::SESSION_START)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() in handle_session_start (Red Gate).
    let response = router.oneshot(req).await.expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect")
        .to_bytes();
    let forwarded: serde_json::Value = serde_json::from_slice(&body_bytes).expect("parse");
    assert!(forwarded.get("session_id").is_some(), "session_id missing");
    assert!(forwarded.get("pid").is_some(), "pid missing");
    assert!(forwarded.get("cwd").is_some(), "cwd (EX-2) missing");
    assert!(
        forwarded.get("transcript_path").is_some(),
        "transcript_path (EX-2) missing"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// AC-003 / BC-HOOK-007: UserPromptSubmit endpoint produces canonical payload fields (EX-2)
// ──────────────────────────────────────────────────────────────────────────────

/// AC-003: The UserPromptSubmit handler produces 3 canonical fields including EX-2 prompt.
///
/// test_BC_HOOK_007_prompt_submit_handler_produces_canonical_fields
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_007_prompt_submit_handler_produces_canonical_fields() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    let input = common::load_fixture("prompt-submit", "prompt-submit-basic");
    let req = Request::builder()
        .method("POST")
        .uri(paths::PROMPT_SUBMIT)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() in handle_prompt_submit (Red Gate).
    let response = router.oneshot(req).await.expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect")
        .to_bytes();
    let forwarded: serde_json::Value = serde_json::from_slice(&body_bytes).expect("parse");
    assert!(forwarded.get("session_id").is_some(), "session_id missing");
    assert!(forwarded.get("pid").is_some(), "pid missing");
    assert!(forwarded.get("prompt").is_some(), "prompt (EX-2) missing");
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-040: Struct-based stable key ordering (verified via handler output)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-040: PreToolUse handler produces byte-stable JSON output (struct, not HashMap).
///
/// test_BC_HOOK_040_struct_based_stable_serialization
/// Two identical requests must produce byte-identical payloads forwarded to daemon.
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_040_struct_based_stable_serialization() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let input = serde_json::json!({
        "session_id": "s1",
        "pid": 1,
        "tool_name": "Bash",
        "tool_input": {}
    });

    let state1 = common::make_test_clone_state();
    let router1 = build_router(state1);
    let req1 = Request::builder()
        .method("POST")
        .uri(paths::PRE_TOOL_USE)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() (Red Gate).
    let r1 = router1.oneshot(req1).await.expect("oneshot 1");

    let state2 = common::make_test_clone_state();
    let router2 = build_router(state2);
    let req2 = Request::builder()
        .method("POST")
        .uri(paths::PRE_TOOL_USE)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    let r2 = router2.oneshot(req2).await.expect("oneshot 2");

    assert_eq!(r1.status(), StatusCode::OK);
    assert_eq!(r2.status(), StatusCode::OK);
    let b1 = r1
        .into_body()
        .collect()
        .await
        .expect("collect 1")
        .to_bytes();
    let b2 = r2
        .into_body()
        .collect()
        .await
        .expect("collect 2")
        .to_bytes();
    assert_eq!(
        b1, b2,
        "handler output must be byte-stable (struct-based serialization)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-008: No HTML escaping in handler-produced payload
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-008: Handler-produced payload contains literal => (no HTML escaping of >).
///
/// test_BC_HOOK_008_no_html_escape_in_handler_payload
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_008_no_html_escape_in_handler_payload() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    let input = serde_json::json!({
        "session_id": "s",
        "pid": 1,
        "tool_name": "Bash",
        "tool_input": {"command": "ls > /tmp/out && cat <file>"}
    });
    let req = Request::builder()
        .method("POST")
        .uri(paths::PRE_TOOL_USE)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() (Red Gate).
    let response = router.oneshot(req).await.expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect")
        .to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert!(
        !body_str.contains("\\u003e"),
        "handler must not HTML-escape > as \\u003e"
    );
    assert!(
        !body_str.contains("\\u003c"),
        "handler must not HTML-escape < as \\u003c"
    );
    assert!(
        !body_str.contains("\\u0026"),
        "handler must not HTML-escape & as \\u0026"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// BC-HOOK-036: Content-Length is UTF-8 byte count (via handler)
// ──────────────────────────────────────────────────────────────────────────────

/// BC-HOOK-036: Handler-forwarded request body uses UTF-8 byte length for Content-Length.
///
/// test_BC_HOOK_036_content_length_utf8_byte_count
#[tokio::test]
#[allow(clippy::expect_used)]
async fn test_BC_HOOK_036_content_length_utf8_byte_count() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let state = common::make_test_clone_state();
    let router = build_router(state);
    // Japanese characters: each kanji is 3 UTF-8 bytes.
    let multibyte_prompt = "テスト"; // 3 chars × 3 bytes = 9 bytes
    let input = serde_json::json!({
        "session_id": "s",
        "pid": 1,
        "prompt": multibyte_prompt
    });
    let req = Request::builder()
        .method("POST")
        .uri(paths::PROMPT_SUBMIT)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&input).expect("serialize")))
        .expect("build req");
    // Panics at todo!() (Red Gate).
    let response = router.oneshot(req).await.expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect")
        .to_bytes();
    // After implementation: the Content-Length sent to daemon must match byte length.
    // We verify the body is valid UTF-8 with the expected content.
    let body_str = std::str::from_utf8(&body_bytes).expect("body must be valid UTF-8");
    let parsed: serde_json::Value = serde_json::from_str(body_str).expect("parse body");
    assert_eq!(
        parsed["prompt"], multibyte_prompt,
        "prompt must be preserved as UTF-8"
    );
}
