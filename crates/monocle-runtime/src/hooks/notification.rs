//! `POST /hooks/notification` handler (S-018, BC-2.04.008).
//!
//! # Contract summary
//!
//! - Deserializes request body into `HookEnvelope` → HTTP 422 on failure.
//! - Constructs `HookEvent::Notification` and calls `engine.on_hook().await`.
//! - `HookDecision::Defer` is NOT supported — treated as `Allow` + WARN log.
//! - Entire handler body wrapped in `tokio::time::timeout(2000ms, ...)`.
//! - On timeout: returns HTTP 200 `{"decision":"allow"}` + WARN log.
//! - After decision: `try_send` to event bus (non-blocking, drop on full).
//! - `DaemonState.ring.append(record)` — best-effort, WARN on failure.
//! - Response is ALWAYS HTTP 200 `{"decision":"allow"}` (PC-7, invariant 4).

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::hooks::HookEnvelope;
use crate::state::{AppMode, DaemonState};

/// Timeout budget for the Notification handler (BC-2.04.008 PC-4).
///
/// 2000ms — seven times PreToolUse budget, for heavier TUI rendering updates.
const NOTIFICATION_TIMEOUT_MS: u64 = 2000;

/// Handler for `POST /hooks/notification`.
///
/// BC-2.04.008: 2000ms timeout, no Defer, always returns HTTP 200 `{"decision":"allow"}`.
///
/// BC-2.01.004 PC-2: during `AppMode::ShuttingDown`, returns HTTP 503.
#[allow(clippy::todo)]
pub async fn post_hook_notification(
    State(state): State<Arc<DaemonState>>,
    body: axum::body::Bytes,
) -> Response {
    // Shutdown gate (BC-2.01.004 PC-2).
    let is_shutting_down = match state.mode.read() {
        Ok(mode) => *mode == AppMode::ShuttingDown,
        Err(_) => {
            tracing::warn!(
                "RwLock<AppMode> poisoned in notification handler; treating as ShuttingDown"
            );
            true
        }
    };
    if is_shutting_down {
        return crate::handlers::hooks::drain_response_pub();
    }

    // Deserialize HookEnvelope — return HTTP 422 on failure (BC-2.04.008 PC-1).
    let envelope: HookEnvelope = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({
                    "error": "invalid_body",
                    "message": err.to_string()
                })),
            )
                .into_response();
        }
    };

    let session_id = envelope.session_id.clone();

    // Wrap entire handler logic in 2000ms timeout (BC-2.04.008 PC-4, invariant 1).
    let result = tokio::time::timeout(
        Duration::from_millis(NOTIFICATION_TIMEOUT_MS),
        handle_notification_inner(Arc::clone(&state), envelope),
    )
    .await;

    match result {
        Ok(response) => response,
        Err(_timeout) => {
            // Notification timeout — always allow (BC-2.04.008 PC-4, PC-7).
            tracing::warn!(
                session_id = %session_id,
                "notification handler timeout (session_id={})",
                session_id
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({"decision": "allow"})),
            )
                .into_response()
        }
    }
}

/// Inner handler logic (runs inside the 2000ms timeout budget).
async fn handle_notification_inner(state: Arc<DaemonState>, envelope: HookEnvelope) -> Response {
    use crate::engine::ClaudeCodeModule;
    use crate::event_bus::try_publish_event;
    use crate::ring::HookEventRecord;
    use crate::types::EventBusHookEvent;
    use monocle_core::engine::{EngineModule as _, HookDecision};
    use monocle_core::hook_events::HookEvent;

    // BC-2.04.008 PC-2: look up or create session entry.
    if let Some(registry) = &state.session_registry {
        registry.get_or_create(&envelope.session_id);
    }

    // Construct HookEvent::Notification from envelope fields.
    // Using serde_json round-trip because NotificationEvent is #[non_exhaustive].
    let hook_event: HookEvent = match serde_json::from_value(serde_json::json!({
        "Notification": {
            "notification_type": envelope.notification_type.clone().unwrap_or_default(),
            "tool_name": envelope.tool_name.clone().unwrap_or_default(),
            "tool_input": envelope.tool_input.clone().unwrap_or(serde_json::Value::Null),
            "message": envelope.message.clone().unwrap_or_default(),
            "session_id": envelope.session_id.clone(),
            "pid": envelope.pid
        }
    })) {
        Ok(e) => e,
        Err(err) => {
            tracing::error!(
                session_id = %envelope.session_id,
                error = %err,
                "failed to construct Notification HookEvent; returning allow"
            );
            // Notification always returns allow (PC-7), even on construction failure.
            return (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"decision": "allow"})),
            )
                .into_response();
        }
    };

    let bus_event_payload = hook_event.clone();

    // Artificial delay for integration tests exercising the 2000ms timeout path.
    // hook_delay_ms is None in production; Some(ms) only in tests.
    if let Some(delay_ms) = state.hook_delay_ms {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    // BC-2.04.008 PC-3: dispatch to EngineModule.
    // hook_decision_override is None in production; Some((decision, diagnostic)) in tests
    // that need to exercise the Block or Defer paths (Phase 1 ClaudeCodeModule always
    // returns Allow, so tests that need Block/Defer inject the decision here).
    let response = if let Some((ref decision, ref diagnostic)) = state.hook_decision_override {
        let base = monocle_core::engine::HookResponse::new(decision.clone());
        if let Some(ref diag) = diagnostic {
            base.with_diagnostic(diag.clone())
        } else {
            base
        }
    } else {
        let engine = ClaudeCodeModule::new(String::new());
        engine.on_hook(hook_event).await
    };

    // BC-2.04.008 PC-3, invariant 4: response to Claude Code is ALWAYS allow.
    // Internal Block decision is handled (future: could log/filter) but not surfaced.
    // BC-2.04.008 invariant 2, EC-077: Defer → treat as Allow with WARN.
    match &response.decision {
        HookDecision::Defer => {
            tracing::warn!(
                session_id = %envelope.session_id,
                "invalid Defer on Notification hook; treating as Allow (BC-2.04.008 invariant 2, EC-077)"
            );
        }
        HookDecision::Block => {
            // Internal Block — not surfaced to Claude Code (invariant 4).
            tracing::debug!(
                session_id = %envelope.session_id,
                "Notification Block decision (internal); returning allow to Claude Code (invariant 4)"
            );
        }
        _ => {}
    }

    // BC-2.04.008 PC-5 / BC-2.04.011 PC-3: try_send to event bus (non-blocking, best-effort).
    let received_at = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    let bus_event = EventBusHookEvent::new(bus_event_payload, received_at);
    try_publish_event(&state, bus_event);

    // Ring append — best-effort, WARN on failure.
    // DI-001 best-effort: ring append runs after decision but before function return.
    // Under timeout cancellation, this append may be skipped (BC-2.04.008 invariant 4 best-effort caveat).
    let now_micros = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0);
    let record = HookEventRecord::new(
        envelope.session_id.clone(),
        now_micros,
        envelope.pid,
        "Notification".to_owned(),
        envelope.tool_name.clone(),
        envelope.tool_input.clone(),
    );
    if let Some(ring) = &state.ring {
        if let Err(e) = ring.append(record) {
            tracing::warn!(
                session_id = %envelope.session_id,
                error = %e,
                "ring append failed for Notification (best-effort)"
            );
        }
    } else {
        tracing::warn!(
            session_id = %envelope.session_id,
            "ring=None; Notification event not persisted (best-effort)"
        );
    }

    // BC-2.04.008 PC-7, invariant 4: ALWAYS return allow to Claude Code.
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({"decision": "allow"})),
    )
        .into_response()
}
