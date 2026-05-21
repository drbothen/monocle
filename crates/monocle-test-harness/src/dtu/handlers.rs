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
    payload::NotificationPayload,
    server::CloneState,
};

/// Fire-and-forget POST to the daemon with auth header per BC-HOOK-004 / BC-HOOK-016.
///
/// Passes raw body bytes to preserve all fields including unknown future extensions.
/// BC-HOOK-001 fail-open semantics: errors are silently absorbed.
/// BC-HOOK-016: auth header sent verbatim from lock file.
fn spawn_daemon_post(state: &CloneState, path: &'static str, body_bytes: Vec<u8>) {
    let url = format!("{}{}", state.endpoint_base, path);
    let client = state.client.clone();
    let token = state.daemon.auth_token.clone();
    tokio::spawn(async move {
        let result = client
            .post(&url)
            .header(AUTH_HEADER_ALIAS, token)
            .header("content-type", "application/json")
            .body(body_bytes)
            .send()
            .await;
        if let Err(e) = result {
            tracing::warn!(
                error = ?e,
                path = path,
                "daemon POST failed (fail-open per BC-HOOK-001)"
            );
        }
    });
}

/// Handler for `POST /hooks/pre-tool-use`.
///
/// BC-HOOK-006: Unconditional stdin echo — returns 200 with the original payload as JSON body.
/// BC-HOOK-032: Must return 200 even for malformed JSON (doubly fail-open).
///   axum's Json extractor rejects malformed JSON; we use raw Bytes extraction.
///
/// Extra-field pass-through (CRIT-4): raw bytes are forwarded to daemon WITHOUT
/// re-serialization through a typed struct. This preserves unknown future fields.
///
/// BC-HOOK-001 (fail-open), BC-HOOK-006 (echo), BC-HOOK-007 (schema),
/// BC-HOOK-016 (auth header), BC-HOOK-032 (malformed JSON → still 200),
/// AC-001, AC-002, AC-003
pub async fn handle_pre_tool_use(
    State(state): State<CloneState>,
    body: Bytes,
) -> impl IntoResponse {
    // Forward raw bytes to daemon — no re-serialization (preserves extra fields).
    // BC-HOOK-032: even if parse fails we still forward raw bytes.
    spawn_daemon_post(&state, paths::PRE_TOOL_USE, body.to_vec());

    // BC-HOOK-006: echo the original raw payload back as response.
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        body.to_vec(),
    )
        .into_response()
}

/// Handler for `POST /hooks/notification`.
///
/// BC-HOOK-034 filter: only `notification_type == "permission_prompt"` payloads
/// reach the daemon wire. All other types are dropped with HTTP 200 (fire-and-forget,
/// fail-open per BC-HOOK-003).
/// BC-HOOK-033: Non-PreToolUse hooks silently drop malformed JSON (return 200).
///
/// Extra-field pass-through (CRIT-4): raw bytes are forwarded to daemon when
/// the filter passes — only `notification_type` is inspected for routing decisions.
///
/// BC-HOOK-003 (fail-closed non-PreToolUse), BC-HOOK-007 (schema),
/// BC-HOOK-016 (auth header), BC-HOOK-033 (malformed → 200 silent drop),
/// BC-HOOK-034 (filter), AC-001, AC-002, AC-003
pub async fn handle_notification(
    State(state): State<CloneState>,
    body: Bytes,
) -> impl IntoResponse {
    // BC-HOOK-033: silently drop malformed JSON — return 200 without forwarding.
    // We only need notification_type for the filter (BC-HOOK-034).
    let payload: NotificationPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                error = ?e,
                endpoint = "notification",
                "malformed JSON — silently dropping per BC-HOOK-033"
            );
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
        tracing::debug!(
            notification_type = %payload.notification_type,
            "forwarding notification to daemon (BC-HOOK-034 filter pass)"
        );
        // Forward raw bytes to daemon — preserves all fields including extras.
        spawn_daemon_post(&state, paths::NOTIFICATION, body.to_vec());
        return (
            StatusCode::OK,
            [("content-type", "application/json")],
            body.to_vec(),
        )
            .into_response();
    }

    // Non-permission_prompt: silently drop, return 200 with payload echo.
    tracing::debug!(
        notification_type = %payload.notification_type,
        "dropping notification (not permission_prompt per BC-HOOK-034)"
    );
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        body.to_vec(),
    )
        .into_response()
}

/// Handler for `POST /hooks/stop`.
///
/// BC-HOOK-033: malformed JSON → silent drop → 200.
///
/// Extra-field pass-through (CRIT-4): raw bytes are forwarded to daemon
/// without re-serialization through a typed struct.
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), BC-HOOK-033,
/// AC-001, AC-002, AC-003
pub async fn handle_stop(State(state): State<CloneState>, body: Bytes) -> impl IntoResponse {
    // BC-HOOK-033: validate JSON is parseable (silently drop malformed).
    // We do a type-check parse but forward the original raw bytes.
    if serde_json::from_slice::<serde_json::Value>(&body).is_err() {
        tracing::warn!(
            endpoint = "stop",
            "malformed JSON — silently dropping per BC-HOOK-033"
        );
        return (
            StatusCode::OK,
            [("content-type", "application/json")],
            body.to_vec(),
        )
            .into_response();
    }
    // Forward raw bytes — preserves all fields.
    spawn_daemon_post(&state, paths::STOP, body.to_vec());
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        body.to_vec(),
    )
        .into_response()
}

/// Handler for `POST /hooks/session-start`.
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), BC-HOOK-033,
/// AC-001, AC-002, AC-003
///
/// Extra-field pass-through (CRIT-4): raw bytes are forwarded to daemon
/// without re-serialization through a typed struct.
pub async fn handle_session_start(
    State(state): State<CloneState>,
    body: Bytes,
) -> impl IntoResponse {
    // BC-HOOK-033: validate JSON is parseable (silently drop malformed).
    if serde_json::from_slice::<serde_json::Value>(&body).is_err() {
        tracing::warn!(
            endpoint = "session-start",
            "malformed JSON — silently dropping per BC-HOOK-033"
        );
        return (
            StatusCode::OK,
            [("content-type", "application/json")],
            body.to_vec(),
        )
            .into_response();
    }
    // Forward raw bytes — preserves all fields including future extensions.
    spawn_daemon_post(&state, paths::SESSION_START, body.to_vec());
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        body.to_vec(),
    )
        .into_response()
}

/// Handler for `POST /hooks/prompt-submit`.
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), BC-HOOK-033,
/// AC-001, AC-002, AC-003
///
/// Extra-field pass-through (CRIT-4): raw bytes are forwarded to daemon
/// without re-serialization through a typed struct.
pub async fn handle_prompt_submit(
    State(state): State<CloneState>,
    body: Bytes,
) -> impl IntoResponse {
    // BC-HOOK-033: validate JSON is parseable (silently drop malformed).
    if serde_json::from_slice::<serde_json::Value>(&body).is_err() {
        tracing::warn!(
            endpoint = "prompt-submit",
            "malformed JSON — silently dropping per BC-HOOK-033"
        );
        return (
            StatusCode::OK,
            [("content-type", "application/json")],
            body.to_vec(),
        )
            .into_response();
    }
    // Forward raw bytes — preserves all fields including future extensions.
    spawn_daemon_post(&state, paths::PROMPT_SUBMIT, body.to_vec());
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        body.to_vec(),
    )
        .into_response()
}
