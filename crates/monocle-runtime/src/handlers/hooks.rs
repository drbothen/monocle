//! Hook endpoint handlers for the monocle daemon (`POST /hooks/*`).
//!
//! S-005 registers the 5 canonical hook routes with shutdown-aware stubs that gate on
//! `AppMode::ShuttingDown`. S-009 extends these handlers with full hook ingestion logic:
//! JSON body acceptance, `HookEventRecord` construction, and `RingBuffer::push()` writes.
//!
//! # Behavioral contract
//!
//! BC-2.01.004 PC-2: While `AppMode::ShuttingDown`, all `POST /hooks/*` requests MUST
//! return HTTP 503 with `{"error":"daemon_shutting_down"}` and `Retry-After: 10`.
//!
//! BC-2.01.002 PC-1 / AC-010b: When `AppMode::Running`, each handler MUST:
//! 1. Accept the JSON hook body from Claude Code (or monocle-aware tool).
//! 2. Construct a `HookEventRecord` and call `RingBuffer::push()` (DI-001: write BEFORE
//!    constructing the HTTP response).
//! 3. Return HTTP 200 with body `{"status":"ok"}`.
//!
//! # Route ownership
//!
//! Five canonical hook endpoints (BC-2.01.004 PC-2, domain-monocle-vision-synthesis.md):
//! - `POST /hooks/pre-tool-use`
//! - `POST /hooks/notification`
//! - `POST /hooks/stop`
//! - `POST /hooks/session-start`
//! - `POST /hooks/prompt-submit`

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::state::{AppMode, DaemonState};

/// Construct a 503 `Retry-After: 10` / `{"error":"daemon_shutting_down"}` response.
///
/// Shared across all 5 hook stubs (BC-2.01.004 PC-2). The `Retry-After: 10` header value
/// is normative and MUST be the exact integer string `"10"` per VP-004 §Post-conditions item 2.
fn drain_response() -> Response {
    let mut response = (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "daemon_shutting_down"})),
    )
        .into_response();
    response
        .headers_mut()
        .insert("Retry-After", HeaderValue::from_static("10"));
    response
}

/// Gate: if `AppMode::ShuttingDown`, return `drain_response()`; otherwise delegate to `f`.
///
/// Used by all 5 hook stubs to enforce BC-2.01.004 PC-2 without repetition.
///
/// A poisoned `RwLock<AppMode>` is treated as `ShuttingDown` — the same degraded-mode
/// convention used by `get_healthz` (BC-2.01.001 safety invariant).
fn with_shutdown_gate<F>(state: &DaemonState, f: F) -> Response
where
    F: FnOnce() -> Response,
{
    let is_shutting_down = match state.mode.read() {
        Ok(mode) => *mode == AppMode::ShuttingDown,
        Err(_poisoned) => {
            tracing::warn!("RwLock<AppMode> poisoned in hook handler; treating as ShuttingDown");
            true
        }
    };

    if is_shutting_down {
        drain_response()
    } else {
        f()
    }
}

/// Handler for `POST /hooks/pre-tool-use`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"PreToolUse"`. The `tool_name` and `tool_input`
/// fields are populated from the request body when present.
#[allow(clippy::todo)]
pub async fn post_hook_pre_tool_use(
    State(state): State<Arc<DaemonState>>,
    Json(_body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        todo!(
            "S-009: PreToolUse — construct HookEventRecord(hook_type=PreToolUse, \
            tool_name from body, tool_input from body), call state.ring.push(), \
            return 200 {{\"status\":\"ok\"}}"
        )
    })
}

/// Handler for `POST /hooks/notification`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"Notification"`.
#[allow(clippy::todo)]
pub async fn post_hook_notification(
    State(state): State<Arc<DaemonState>>,
    Json(_body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        todo!(
            "S-009: Notification — construct HookEventRecord(hook_type=Notification), \
            call state.ring.push(), return 200 {{\"status\":\"ok\"}}"
        )
    })
}

/// Handler for `POST /hooks/stop`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"Stop"`.
#[allow(clippy::todo)]
pub async fn post_hook_stop(
    State(state): State<Arc<DaemonState>>,
    Json(_body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        todo!(
            "S-009: Stop — construct HookEventRecord(hook_type=Stop), \
            call state.ring.push(), return 200 {{\"status\":\"ok\"}}"
        )
    })
}

/// Handler for `POST /hooks/session-start`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"SessionStart"`.
#[allow(clippy::todo)]
pub async fn post_hook_session_start(
    State(state): State<Arc<DaemonState>>,
    Json(_body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        todo!(
            "S-009: SessionStart — construct HookEventRecord(hook_type=SessionStart), \
            call state.ring.push(), return 200 {{\"status\":\"ok\"}}"
        )
    })
}

/// Handler for `POST /hooks/prompt-submit`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"UserPromptSubmit"`.
#[allow(clippy::todo)]
pub async fn post_hook_prompt_submit(
    State(state): State<Arc<DaemonState>>,
    Json(_body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        todo!(
            "S-009: UserPromptSubmit — construct HookEventRecord(hook_type=UserPromptSubmit), \
            call state.ring.push(), return 200 {{\"status\":\"ok\"}}"
        )
    })
}
