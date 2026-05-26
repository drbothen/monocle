//! Hook endpoint handlers for the monocle daemon (`POST /hooks/*`).
//!
//! S-005 registers the 5 canonical hook routes with shutdown-aware stubs that gate on
//! `AppMode::ShuttingDown`. Full hook ingestion logic (event persistence, ring buffer,
//! session tracking) is implemented by S-009.
//!
//! # Behavioral contract
//!
//! BC-2.01.004 PC-2: While `AppMode::ShuttingDown`, all `POST /hooks/*` requests MUST
//! return HTTP 503 with `{"error":"daemon_shutting_down"}` and `Retry-After: 10`.
//!
//! S-009 extends these stubs with full hook handling. When `AppMode::Running`, the stub
//! returns HTTP 501 (not yet implemented); S-009 replaces that branch with real logic.
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

/// Stub handler for `POST /hooks/pre-tool-use`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// S-009 replaces the non-drain branch with real pre-tool-use hook ingestion logic.
pub async fn post_hook_pre_tool_use(State(state): State<Arc<DaemonState>>) -> Response {
    with_shutdown_gate(&state, || {
        // S-009 stub: not yet implemented outside of drain guard.
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({"error": "not_implemented"})),
        )
            .into_response()
    })
}

/// Stub handler for `POST /hooks/notification`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// S-009 replaces the non-drain branch with real notification hook ingestion logic.
pub async fn post_hook_notification(State(state): State<Arc<DaemonState>>) -> Response {
    with_shutdown_gate(&state, || {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({"error": "not_implemented"})),
        )
            .into_response()
    })
}

/// Stub handler for `POST /hooks/stop`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// S-009 replaces the non-drain branch with real stop hook ingestion logic.
pub async fn post_hook_stop(State(state): State<Arc<DaemonState>>) -> Response {
    with_shutdown_gate(&state, || {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({"error": "not_implemented"})),
        )
            .into_response()
    })
}

/// Stub handler for `POST /hooks/session-start`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// S-009 replaces the non-drain branch with real session-start hook ingestion logic.
pub async fn post_hook_session_start(State(state): State<Arc<DaemonState>>) -> Response {
    with_shutdown_gate(&state, || {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({"error": "not_implemented"})),
        )
            .into_response()
    })
}

/// Stub handler for `POST /hooks/prompt-submit`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// S-009 replaces the non-drain branch with real prompt-submit hook ingestion logic.
pub async fn post_hook_prompt_submit(State(state): State<Arc<DaemonState>>) -> Response {
    with_shutdown_gate(&state, || {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({"error": "not_implemented"})),
        )
            .into_response()
    })
}
