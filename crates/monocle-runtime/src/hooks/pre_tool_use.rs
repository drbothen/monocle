//! `POST /hooks/pre-tool-use` handler (S-018, BC-2.04.007).
//!
//! # Contract summary
//!
//! - Deserializes request body into `HookEnvelope` → HTTP 422 on failure.
//! - Constructs `HookEvent::PreToolUse` and calls `engine.on_hook().await`.
//! - On `HookDecision::Defer`: creates `oneshot::channel`, pushes
//!   `PermissionPromptQueued` IPC to TUI clients, awaits decision.
//! - Entire handler body wrapped in `tokio::time::timeout(300ms, ...)`.
//! - On timeout: returns HTTP 200 `{"decision":"allow","reason":"timeout"}` (fail-open).
//! - After decision: `try_send` to event bus (non-blocking, drop on full).
//! - `DaemonState.ring.append(record)` — best-effort, WARN on failure.
//! - HTTP 200 returned regardless of bus saturation or ring failure.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::hooks::HookEnvelope;
use crate::state::{AppMode, DaemonState};

/// Timeout budget for the PreToolUse handler (BC-2.04.007 PC-4).
///
/// 300ms is absolute — no code path may delay the HTTP response beyond this.
const PRE_TOOL_USE_TIMEOUT_MS: u64 = 300;

/// Handler for `POST /hooks/pre-tool-use`.
///
/// BC-2.04.007: 300ms timeout, EngineModule dispatch, Defer support via oneshot,
/// event bus try_send, ring append. Fail-open on timeout.
///
/// BC-2.01.004 PC-2: during `AppMode::ShuttingDown`, returns HTTP 503.
#[allow(clippy::todo)]
pub async fn post_hook_pre_tool_use(
    State(state): State<Arc<DaemonState>>,
    body: axum::body::Bytes,
) -> Response {
    // Shutdown gate (BC-2.01.004 PC-2).
    let is_shutting_down = match state.mode.read() {
        Ok(mode) => *mode == AppMode::ShuttingDown,
        Err(_) => {
            tracing::warn!(
                "RwLock<AppMode> poisoned in pre_tool_use handler; treating as ShuttingDown"
            );
            true
        }
    };
    if is_shutting_down {
        return crate::handlers::hooks::drain_response_pub();
    }

    // Deserialize HookEnvelope — return HTTP 422 on failure (BC-2.04.007 PC-1).
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

    // Wrap entire handler logic in 300ms timeout (BC-2.04.007 PC-4, invariant 1).
    let result = tokio::time::timeout(
        Duration::from_millis(PRE_TOOL_USE_TIMEOUT_MS),
        handle_pre_tool_use_inner(Arc::clone(&state), envelope),
    )
    .await;

    match result {
        Ok(response) => response,
        Err(_timeout) => {
            // Fail-open on timeout (BC-2.04.007 PC-4, PC-7, F-P1D2-001).
            tracing::warn!(
                "pre_tool_use handler timeout after {}ms; returning fail-open allow",
                PRE_TOOL_USE_TIMEOUT_MS
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({"decision": "allow", "reason": "timeout"})),
            )
                .into_response()
        }
    }
}

/// Inner handler logic (runs inside the 300ms timeout budget).
async fn handle_pre_tool_use_inner(state: Arc<DaemonState>, envelope: HookEnvelope) -> Response {
    use crate::engine::ClaudeCodeModule;
    use crate::event_bus::try_publish_event;
    use crate::ring::HookEventRecord;
    use crate::types::EventBusHookEvent;
    use monocle_core::engine::{EngineModule as _, HookDecision};
    use monocle_core::hook_events::HookEvent;

    // BC-2.04.007 PC-2: look up or create session entry.
    if let Some(registry) = &state.session_registry {
        registry.get_or_create(&envelope.session_id);
    }

    // Construct HookEvent::PreToolUse from envelope fields.
    // Using serde_json round-trip because PreToolUseEvent is #[non_exhaustive] (no external
    // struct-literal construction allowed outside monocle-core).
    let hook_event: HookEvent = match serde_json::from_value(serde_json::json!({
        "PreToolUse": {
            "tool_name": envelope.tool_name.clone().unwrap_or_default(),
            "tool_input": envelope.tool_input.clone().unwrap_or(serde_json::Value::Null),
            "session_id": envelope.session_id.clone(),
            "pid": envelope.pid
        }
    })) {
        Ok(e) => e,
        Err(err) => {
            tracing::error!(
                session_id = %envelope.session_id,
                error = %err,
                "failed to construct PreToolUse HookEvent; returning fail-open allow"
            );
            return (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"decision": "allow"})),
            )
                .into_response();
        }
    };

    // Clone event for bus publish (before dispatch consumes it).
    let bus_event_payload = hook_event.clone();

    // Artificial delay for integration tests exercising the 300ms timeout path.
    // hook_delay_ms is None in production; Some(ms) only in tests.
    if let Some(delay_ms) = state.hook_delay_ms {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    // BC-2.04.007 PC-3: dispatch to EngineModule.
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

    // BC-2.04.007 PC-3: handle HookDecision.
    let http_response = match &response.decision {
        HookDecision::Allow => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"decision": "allow"})),
        )
            .into_response(),
        HookDecision::Block => {
            let reason = response
                .diagnostic
                .as_deref()
                .unwrap_or("blocked by policy")
                .to_owned();
            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"decision": "block", "reason": reason})),
            )
                .into_response()
        }
        HookDecision::Defer => {
            // BC-2.04.007 invariant 5: Defer → push PermissionPromptQueued IPC (S-022 scope).
            // Phase 1: no TUI clients yet; await a oneshot that will never fire, which means
            // the 300ms outer timeout wrapping this function will fire, returning fail-open allow.
            // This correctly documents the Defer path and lets the timeout test fire.
            let (_tx, rx) = tokio::sync::oneshot::channel::<HookDecision>();
            tracing::info!(
                session_id = %envelope.session_id,
                "PreToolUse Defer: awaiting permission prompt resolution (S-022 will wire TUI IPC)"
            );
            // Await the oneshot. The outer 300ms timeout will fire before this resolves.
            let _ = rx.await;
            // This line is unreachable in Phase 1; the outer timeout catches it.
            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"decision": "allow", "reason": "defer-resolved"})),
            )
                .into_response()
        }
        // Wildcard for #[non_exhaustive] — fail-open (EC-031).
        _ => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"decision": "allow"})),
        )
            .into_response(),
    };

    // BC-2.04.007 PC-5 / BC-2.04.011 PC-3: try_send to event bus (non-blocking, best-effort).
    let received_at = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    let bus_event = EventBusHookEvent::new(bus_event_payload, received_at);
    try_publish_event(&state, bus_event);

    // BC-2.04.007 PC-6 / AC-019: ring append — best-effort, WARN on failure.
    let now_micros = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0);
    let record = HookEventRecord::new(
        envelope.session_id.clone(),
        now_micros,
        envelope.pid,
        "PreToolUse".to_owned(),
        envelope.tool_name.clone(),
        envelope.tool_input.clone(),
    );
    if let Some(ring) = &state.ring {
        if let Err(e) = ring.append(record) {
            tracing::warn!(
                session_id = %envelope.session_id,
                error = %e,
                "ring append failed for PreToolUse (best-effort AC-019)"
            );
        }
    } else {
        tracing::warn!(
            session_id = %envelope.session_id,
            "ring=None; PreToolUse event not persisted (best-effort AC-019)"
        );
    }

    http_response
}
