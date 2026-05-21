//! Axum handler functions for the 5 DTU clone hook endpoints.
// Stub module: all handler bodies are `todo!()` per TDD Red Gate.
// The `todo` lint is suppressed at file scope; this allow is intentional and
// will be removed as each handler is implemented in the TDD implementation step.
#![allow(clippy::todo)]
//!
//! Each handler synthesizes the monocle-canonical POST payload and fires it at
//! the monocle daemon. Handlers are wired to axum routes by `server.rs`.
//!
//! Source authority:
//! - AC-001 (5 endpoints implemented)
//! - AC-002 (X-Claude-Code-Ide-Authorization header on all POSTs)
//! - AC-003 (monocle-canonical payload structure)
//! - dtu-assessment.md v1.7.5 §Endpoint matrix
//! - BC-HOOK-001..BC-HOOK-041

use axum::{extract::State, http::StatusCode, Json};

use crate::dtu::{
    payload::{
        NotificationPayload, PreToolUsePayload, SessionStartPayload, StopPayload,
        UserPromptSubmitPayload,
    },
    server::CloneState,
};

/// Handler for `POST /hooks/pre-tool-use`.
///
/// Synthesizes a `PreToolUsePayload` with monocle-canonical fields, then fires a
/// POST to the daemon's `/hooks/pre-tool-use` endpoint with header
/// `X-Claude-Code-Ide-Authorization: <raw-token>`.
///
/// BC-HOOK-001 (fail-open), BC-HOOK-007 (schema), BC-HOOK-016 (auth header),
/// AC-001, AC-002, AC-003
#[allow(unused_variables)]
pub async fn handle_pre_tool_use(
    State(state): State<CloneState>,
    Json(payload): Json<PreToolUsePayload>,
) -> StatusCode {
    todo!("S-DTU-001 implementation pending; BC-HOOK-007, BC-HOOK-016")
}

/// Handler for `POST /hooks/notification`.
///
/// Synthesizes a `NotificationPayload`. Per BC-HOOK-034, the clone applies the
/// client-side filter: only `notification_type == "permission_prompt"` payloads
/// reach the wire; other notification types are dropped with HTTP 200 (fire-and-forget).
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), BC-HOOK-034 (filter),
/// AC-001, AC-002, AC-003
#[allow(unused_variables)]
pub async fn handle_notification(
    State(state): State<CloneState>,
    Json(payload): Json<NotificationPayload>,
) -> StatusCode {
    todo!("S-DTU-001 implementation pending; BC-HOOK-007, BC-HOOK-016, BC-HOOK-034")
}

/// Handler for `POST /hooks/stop`.
///
/// Synthesizes a `StopPayload` and fires it to the daemon's `/hooks/stop` endpoint.
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), AC-001, AC-002, AC-003
#[allow(unused_variables)]
pub async fn handle_stop(
    State(state): State<CloneState>,
    Json(payload): Json<StopPayload>,
) -> StatusCode {
    todo!("S-DTU-001 implementation pending; BC-HOOK-007, BC-HOOK-016")
}

/// Handler for `POST /hooks/session-start`.
///
/// Synthesizes a `SessionStartPayload` including EX-2 fields (`cwd`, `transcript_path`).
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), AC-001, AC-002, AC-003
#[allow(unused_variables)]
pub async fn handle_session_start(
    State(state): State<CloneState>,
    Json(payload): Json<SessionStartPayload>,
) -> StatusCode {
    todo!("S-DTU-001 implementation pending; BC-HOOK-007, BC-HOOK-016")
}

/// Handler for `POST /hooks/prompt-submit`.
///
/// Synthesizes a `UserPromptSubmitPayload` including EX-2 field (`prompt`).
///
/// BC-HOOK-007 (schema), BC-HOOK-016 (auth header), AC-001, AC-002, AC-003
#[allow(unused_variables)]
pub async fn handle_prompt_submit(
    State(state): State<CloneState>,
    Json(payload): Json<UserPromptSubmitPayload>,
) -> StatusCode {
    todo!("S-DTU-001 implementation pending; BC-HOOK-007, BC-HOOK-016")
}
