//! Axum handler functions for the 5 DTU clone hook endpoints.
//!
//! Each handler synthesizes the monocle-canonical POST payload and fires it at
//! the monocle daemon. Handlers are wired to axum routes by `endpoints.rs`.
//!
//! Source authority:
//! - AC-001 (5 endpoints implemented)
//! - AC-002 (X-Claude-Code-Ide-Authorization header on all POSTs)
//! - AC-003 (monocle-canonical payload structure)
//! - BC-HOOK-001..BC-HOOK-041

use axum::{body::Bytes, extract::State, http::StatusCode, response::IntoResponse};

use crate::dtu::{
    endpoints::{paths, AUTH_HEADER_ALIAS},
    payload::{
        NotificationPayload, PreToolUsePayload, SessionStartPayload, StopPayload,
        UserPromptSubmitPayload,
    },
    server::CloneState,
};

/// Serialize a value to JSON bytes without HTML escaping.
///
/// BC-HOOK-008: no HTML escaping of >, <, & characters.
/// serde_json's default serializer HTML-escapes; we use a raw formatter.
fn to_json_bytes_no_escape<T: serde::Serialize>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::new(&mut buf);
    serde::Serialize::serialize(value, &mut ser).unwrap_or_default();
    buf
}

/// Fire-and-forget POST to the daemon with auth header per BC-HOOK-004 / BC-HOOK-016.
///
/// Errors are silently absorbed — BC-HOOK-001 fail-open semantics.
fn spawn_daemon_post(state: &CloneState, path: &'static str, body_bytes: Vec<u8>) {
    let url = format!("{}{}", state.endpoint_base, path);
    let client = state.client.clone();
    let token = state.daemon.auth_token.clone();
    tokio::spawn(async move {
        let _ = client
            .post(&url)
            .header(AUTH_HEADER_ALIAS, token)
            .header("content-type", "application/json")
            .body(body_bytes)
            .send()
            .await;
    });
}

/// Handler for `POST /hooks/pre-tool-use`.
///
/// BC-HOOK-006: Unconditional stdin echo — returns 200 with the original payload as JSON body.
/// BC-HOOK-032: Must return 200 even for malformed JSON (doubly fail-open).
///   axum's Json extractor rejects malformed JSON; we use raw Bytes extraction
///   and try to deserialize, falling back to echoing raw bytes on parse failure.
///
/// BC-HOOK-001 (fail-open), BC-HOOK-006 (echo), BC-HOOK-007 (schema),
/// BC-HOOK-016 (auth header), BC-HOOK-032 (malformed JSON → still 200),
/// AC-001, AC-002, AC-003
pub async fn handle_pre_tool_use(
    State(state): State<CloneState>,
    body: Bytes,
) -> impl IntoResponse {
    // BC-HOOK-032: Try to parse as PreToolUsePayload; on failure, echo raw bytes with 200.
    match serde_json::from_slice::<PreToolUsePayload>(&body) {
        Ok(payload) => {
            // Fire-and-forget POST to daemon per BC-HOOK-004.
            let echo_bytes = to_json_bytes_no_escape(&payload);
            spawn_daemon_post(&state, paths::PRE_TOOL_USE, echo_bytes.clone());
            // BC-HOOK-006: echo original payload back as response.
            (
                StatusCode::OK,
                [("content-type", "application/json")],
                echo_bytes,
            )
                .into_response()
        }
        Err(_) => {
            // BC-HOOK-032: malformed JSON → echo raw stdin bytes with 200.
            // Fire-and-forget with raw bytes (daemon may reject, but we fail-open).
            spawn_daemon_post(&state, paths::PRE_TOOL_USE, body.to_vec());
            (
                StatusCode::OK,
                [("content-type", "application/json")],
                body.to_vec(),
            )
                .into_response()
        }
    }
}

/// Handler for `POST /hooks/notification`.
///
/// BC-HOOK-034 filter: only `notification_type == "permission_prompt"` payloads
/// reach the daemon wire. All other types are dropped with HTTP 200 (fire-and-forget,
/// fail-open per BC-HOOK-003).
/// BC-HOOK-033: Non-PreToolUse hooks silently drop malformed JSON (return 200).
///
/// BC-HOOK-003 (fail-closed non-PreToolUse), BC-HOOK-007 (schema),
/// BC-HOOK-016 (auth header), BC-HOOK-033 (malformed → 200 silent drop),
/// BC-HOOK-034 (filter), AC-001, AC-002, AC-003
pub async fn handle_notification(
    State(state): State<CloneState>,
    body: Bytes,
) -> impl IntoResponse {
    // BC-HOOK-033: silently drop malformed JSON — return 200 without forwarding.
    let payload: NotificationPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::OK,
                [("content-type", "application/json")],
                body.to_vec(),
            )
                .into_response();
        }
    };

    // BC-HOOK-034: case-sensitive filter — only "permission_prompt" is forwarded.
    if payload.notification_type == "permission_prompt" {
        let forward_bytes = to_json_bytes_no_escape(&payload);
        spawn_daemon_post(&state, paths::NOTIFICATION, forward_bytes.clone());
        return (
            StatusCode::OK,
            [("content-type", "application/json")],
            forward_bytes,
        )
            .into_response();
    }

    // Non-permission_prompt: silently drop, return 200 with payload echo.
    let echo_bytes = to_json_bytes_no_escape(&payload);
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        echo_bytes,
    )
        .into_response()
}

/// Handler for `POST /hooks/stop`.
///
/// BC-HOOK-033: malformed JSON → silent drop → 200.
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), BC-HOOK-033,
/// AC-001, AC-002, AC-003
pub async fn handle_stop(State(state): State<CloneState>, body: Bytes) -> impl IntoResponse {
    let payload: StopPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::OK,
                [("content-type", "application/json")],
                body.to_vec(),
            )
                .into_response();
        }
    };
    let forward_bytes = to_json_bytes_no_escape(&payload);
    spawn_daemon_post(&state, paths::STOP, forward_bytes.clone());
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        forward_bytes,
    )
        .into_response()
}

/// Handler for `POST /hooks/session-start`.
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), BC-HOOK-033,
/// AC-001, AC-002, AC-003
pub async fn handle_session_start(
    State(state): State<CloneState>,
    body: Bytes,
) -> impl IntoResponse {
    let payload: SessionStartPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::OK,
                [("content-type", "application/json")],
                body.to_vec(),
            )
                .into_response();
        }
    };
    let forward_bytes = to_json_bytes_no_escape(&payload);
    spawn_daemon_post(&state, paths::SESSION_START, forward_bytes.clone());
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        forward_bytes,
    )
        .into_response()
}

/// Handler for `POST /hooks/prompt-submit`.
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), BC-HOOK-033,
/// AC-001, AC-002, AC-003
pub async fn handle_prompt_submit(
    State(state): State<CloneState>,
    body: Bytes,
) -> impl IntoResponse {
    let payload: UserPromptSubmitPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::OK,
                [("content-type", "application/json")],
                body.to_vec(),
            )
                .into_response();
        }
    };
    let forward_bytes = to_json_bytes_no_escape(&payload);
    spawn_daemon_post(&state, paths::PROMPT_SUBMIT, forward_bytes.clone());
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        forward_bytes,
    )
        .into_response()
}
