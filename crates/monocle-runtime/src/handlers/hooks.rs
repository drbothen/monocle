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
//!
//! # Ring buffer write policy (AC-005)
//!
//! The ring buffer write is best-effort. If `state.ring` is `None` (not yet configured)
//! or `ring.push()` returns an error, the handler logs at WARN and still returns HTTP 200.
//! Hook event ingestion MUST NOT block or fail the HTTP response.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::ring::HookEventRecord;
use crate::state::{AppMode, DaemonState};

/// Extract `session_id` string from the JSON body; fall back to `"unknown"` if absent/malformed.
///
/// Used by all 5 hook handlers — `session_id` is present in all canonical hook payloads
/// (BC-2.01.008 PC-1). The fallback string prevents a missing key from being a hard error.
fn extract_session_id(body: &serde_json::Value) -> String {
    body.get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_owned()
}

/// Extract `pid` u32 from the JSON body; fall back to `0` if absent/malformed.
///
/// The originating harness process ID (BC-2.01.008 PC-1). Zero is a sentinel for "unknown PID".
fn extract_pid(body: &serde_json::Value) -> u32 {
    body.get("pid")
        .and_then(|v| v.as_u64())
        .map(|n| n as u32)
        .unwrap_or(0)
}

/// Current Unix epoch timestamp in microseconds.
///
/// Derived from `std::time::SystemTime` to avoid a chrono dependency in the handler path.
/// `i64` per `HookEventRecord` schema (SS-core-types-and-abi.md §HookEventRecord).
fn now_micros() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

/// Write a `HookEventRecord` to the ring buffer from `state`, per AC-005 best-effort policy.
///
/// - If `state.ring` is `None`, log WARN and return immediately (ring not yet initialised).
/// - If `ring.push()` fails, log WARN and return immediately (E-RING-001 degraded mode).
/// - On success, no log is emitted (hot path).
///
/// DI-001: callers MUST invoke this BEFORE constructing any HTTP response.
fn ring_push_best_effort(state: &DaemonState, record: &HookEventRecord) {
    match &state.ring {
        None => {
            tracing::warn!(
                hook_type = %record.hook_type,
                "ring buffer not initialised; hook event dropped (AC-005 best-effort)"
            );
        }
        Some(ring) => {
            if let Err(e) = ring.push(record) {
                tracing::warn!(
                    error = %e,
                    hook_type = %record.hook_type,
                    "E-RING-001: ring buffer push failed; hook event dropped (AC-005 best-effort)"
                );
            }
        }
    }
}

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
pub async fn post_hook_pre_tool_use(
    State(state): State<Arc<DaemonState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        // Extract tool-specific fields — PreToolUse carries tool_name + tool_input.
        let tool_name = body
            .get("tool_name")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let tool_input = body.get("tool_input").cloned();

        let record = HookEventRecord::new(
            extract_session_id(&body),
            now_micros(),
            extract_pid(&body),
            "PreToolUse".to_owned(),
            tool_name,
            tool_input,
        );
        // DI-001: ring write BEFORE constructing the HTTP response.
        ring_push_best_effort(&state, &record);

        (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
    })
}

/// Handler for `POST /hooks/notification`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"Notification"`.
pub async fn post_hook_notification(
    State(state): State<Arc<DaemonState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        let record = HookEventRecord::new(
            extract_session_id(&body),
            now_micros(),
            extract_pid(&body),
            "Notification".to_owned(),
            None,
            None,
        );
        // DI-001: ring write BEFORE constructing the HTTP response.
        ring_push_best_effort(&state, &record);

        (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
    })
}

/// Handler for `POST /hooks/stop`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"Stop"`.
pub async fn post_hook_stop(
    State(state): State<Arc<DaemonState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        let record = HookEventRecord::new(
            extract_session_id(&body),
            now_micros(),
            extract_pid(&body),
            "Stop".to_owned(),
            None,
            None,
        );
        // DI-001: ring write BEFORE constructing the HTTP response.
        ring_push_best_effort(&state, &record);

        (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
    })
}

/// Handler for `POST /hooks/session-start`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"SessionStart"`.
pub async fn post_hook_session_start(
    State(state): State<Arc<DaemonState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        let record = HookEventRecord::new(
            extract_session_id(&body),
            now_micros(),
            extract_pid(&body),
            "SessionStart".to_owned(),
            None,
            None,
        );
        // DI-001: ring write BEFORE constructing the HTTP response.
        ring_push_best_effort(&state, &record);

        (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
    })
}

/// Handler for `POST /hooks/prompt-submit`.
///
/// BC-2.01.004 PC-2: returns HTTP 503 during drain.
/// AC-010b: when Running, deserializes JSON body, writes `HookEventRecord` to ring buffer,
/// returns HTTP 200 `{"status":"ok"}`.
///
/// The `hook_type` for this endpoint is `"UserPromptSubmit"`.
pub async fn post_hook_prompt_submit(
    State(state): State<Arc<DaemonState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    with_shutdown_gate(&state, || {
        let record = HookEventRecord::new(
            extract_session_id(&body),
            now_micros(),
            extract_pid(&body),
            "UserPromptSubmit".to_owned(),
            None,
            None,
        );
        // DI-001: ring write BEFORE constructing the HTTP response.
        ring_push_best_effort(&state, &record);

        (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
    })
}
