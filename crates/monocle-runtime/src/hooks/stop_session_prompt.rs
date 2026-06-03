//! `POST /hooks/stop`, `POST /hooks/session-start`, `POST /hooks/prompt-submit` handlers
//! (S-018, BC-2.04.009).
//!
//! # Contract summary (all three handlers)
//!
//! - Deserializes request body into `HookEnvelope` → HTTP 422 on failure.
//! - Constructs the appropriate `HookEvent` variant and calls `engine.on_hook().await`.
//! - `HookDecision::Defer` is NOT supported — treated as `Allow` + WARN log.
//! - Entire handler body wrapped in `tokio::time::timeout(300ms, ...)`.
//! - On timeout: returns HTTP 200 `{"decision":"allow"}` (fail-open) + WARN log.
//! - After decision: `try_send` to event bus + `ring.append()` (both best-effort).
//!
//! # Stop-specific
//!
//! After `on_hook()` returns, the Stop handler transitions the session to
//! `SessionState::Stopped` in the registry (BC-2.04.009 PC-2).
//! Response is always `{"decision":"allow"}` — monocle cannot block session termination.
//!
//! # SessionStart-specific
//!
//! Response is always `{"decision":"allow"}` — session start is informational.
//!
//! # PromptSubmit-specific
//!
//! - `HookDecision::Allow` → `{"decision":"allow"}`.
//! - `HookDecision::Block` → `{"decision":"block","reason":"<reason>"}` where reason
//!   comes from `HookResponse.diagnostic`. (BC-2.04.009 PC-7).
//! - Timeout → `{"decision":"allow"}` (fail-open, BC-2.04.009 PC-7).

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::hooks::HookEnvelope;
use crate::state::{AppMode, DaemonState};

/// Timeout budget for Stop, SessionStart, PromptSubmit handlers (BC-2.04.009 PC-4).
///
/// 300ms — same as PreToolUse. Absolute budget; fail-open on expiry.
const STOP_SESSION_PROMPT_TIMEOUT_MS: u64 = 300;

/// Handler for `POST /hooks/stop`.
///
/// BC-2.04.009: 300ms timeout, no Defer, transitions session to Stopped,
/// always returns HTTP 200 `{"decision":"allow"}`.
///
/// BC-2.01.004 PC-2: during `AppMode::ShuttingDown`, returns HTTP 503.
#[allow(clippy::todo)]
pub async fn post_hook_stop(
    State(state): State<Arc<DaemonState>>,
    body: axum::body::Bytes,
) -> Response {
    // Shutdown gate (BC-2.01.004 PC-2).
    let is_shutting_down = match state.mode.read() {
        Ok(mode) => *mode == AppMode::ShuttingDown,
        Err(_) => {
            tracing::warn!("RwLock<AppMode> poisoned in stop handler; treating as ShuttingDown");
            true
        }
    };
    if is_shutting_down {
        return crate::handlers::hooks::drain_response_pub();
    }

    // Deserialize HookEnvelope (BC-2.04.009 PC-1).
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

    let result = tokio::time::timeout(
        Duration::from_millis(STOP_SESSION_PROMPT_TIMEOUT_MS),
        handle_stop_inner(Arc::clone(&state), envelope),
    )
    .await;

    match result {
        Ok(response) => response,
        Err(_timeout) => {
            tracing::warn!(
                session_id = %session_id,
                "Stop handler timeout (session_id={})",
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

/// Handler for `POST /hooks/session-start`.
///
/// BC-2.04.009: 300ms timeout, no Defer, always returns HTTP 200 `{"decision":"allow"}`.
///
/// BC-2.01.004 PC-2: during `AppMode::ShuttingDown`, returns HTTP 503.
#[allow(clippy::todo)]
pub async fn post_hook_session_start(
    State(state): State<Arc<DaemonState>>,
    body: axum::body::Bytes,
) -> Response {
    // Shutdown gate (BC-2.01.004 PC-2).
    let is_shutting_down = match state.mode.read() {
        Ok(mode) => *mode == AppMode::ShuttingDown,
        Err(_) => {
            tracing::warn!(
                "RwLock<AppMode> poisoned in session_start handler; treating as ShuttingDown"
            );
            true
        }
    };
    if is_shutting_down {
        return crate::handlers::hooks::drain_response_pub();
    }

    // Deserialize HookEnvelope (BC-2.04.009 PC-1).
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

    let result = tokio::time::timeout(
        Duration::from_millis(STOP_SESSION_PROMPT_TIMEOUT_MS),
        handle_session_start_inner(Arc::clone(&state), envelope),
    )
    .await;

    match result {
        Ok(response) => response,
        Err(_timeout) => {
            tracing::warn!(
                session_id = %session_id,
                "SessionStart handler timeout (session_id={})",
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

/// Handler for `POST /hooks/prompt-submit`.
///
/// BC-2.04.009: 300ms timeout, no Defer, Block response includes reason from diagnostic,
/// timeout is fail-open allow.
///
/// BC-2.01.004 PC-2: during `AppMode::ShuttingDown`, returns HTTP 503.
#[allow(clippy::todo)]
pub async fn post_hook_prompt_submit(
    State(state): State<Arc<DaemonState>>,
    body: axum::body::Bytes,
) -> Response {
    // Shutdown gate (BC-2.01.004 PC-2).
    let is_shutting_down = match state.mode.read() {
        Ok(mode) => *mode == AppMode::ShuttingDown,
        Err(_) => {
            tracing::warn!(
                "RwLock<AppMode> poisoned in prompt_submit handler; treating as ShuttingDown"
            );
            true
        }
    };
    if is_shutting_down {
        return crate::handlers::hooks::drain_response_pub();
    }

    // Deserialize HookEnvelope (BC-2.04.009 PC-1).
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

    let result = tokio::time::timeout(
        Duration::from_millis(STOP_SESSION_PROMPT_TIMEOUT_MS),
        handle_prompt_submit_inner(Arc::clone(&state), envelope),
    )
    .await;

    match result {
        Ok(response) => response,
        Err(_timeout) => {
            tracing::warn!(
                session_id = %session_id,
                "PromptSubmit handler timeout (session_id={})",
                session_id
            );
            // Fail-open (BC-2.04.009 PC-4, PC-7, EC-086).
            (
                StatusCode::OK,
                Json(serde_json::json!({"decision": "allow"})),
            )
                .into_response()
        }
    }
}

/// Inner Stop handler logic (runs inside the 300ms timeout budget).
async fn handle_stop_inner(state: Arc<DaemonState>, envelope: HookEnvelope) -> Response {
    use crate::engine::ClaudeCodeModule;
    use crate::event_bus::try_publish_event;
    use crate::ring::HookEventRecord;
    use crate::types::EventBusHookEvent;
    use monocle_core::engine::{EngineModule as _, HookDecision};
    use monocle_core::hook_events::HookEvent;

    // Construct HookEvent::Stop from envelope fields.
    // Using serde_json round-trip because StopEvent is #[non_exhaustive].
    let hook_event: HookEvent = match serde_json::from_value(serde_json::json!({
        "Stop": {
            "stop_reason": envelope.stop_reason.clone().unwrap_or_else(|| "unknown".to_owned()),
            "session_id": envelope.session_id.clone(),
            "pid": envelope.pid
        }
    })) {
        Ok(e) => e,
        Err(err) => {
            tracing::error!(
                session_id = %envelope.session_id,
                error = %err,
                "failed to construct Stop HookEvent; returning allow"
            );
            return (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"decision": "allow"})),
            )
                .into_response();
        }
    };

    let bus_event_payload = hook_event.clone();

    // Artificial delay for integration tests exercising the 300ms timeout path.
    // hook_delay_ms is None in production; Some(ms) only in tests.
    if let Some(delay_ms) = state.hook_delay_ms {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    // BC-2.04.009 PC-3: dispatch to EngineModule.
    // hook_decision_override is None in production; Some((decision, diagnostic)) in tests
    // that need to exercise the Defer path (Phase 1 ClaudeCodeModule always returns Allow).
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

    // BC-2.04.009 invariant 2, AC-017: Defer → treat as Allow with WARN.
    // Stop cannot be deferred or blocked — monocle cannot prevent session termination.
    if response.decision == HookDecision::Defer {
        tracing::warn!(
            session_id = %envelope.session_id,
            "invalid Defer on Stop hook; treating as Allow (BC-2.04.009 invariant 2, AC-017)"
        );
    }

    // BC-2.04.009 PC-2, AC-016: transition session to Stopped after on_hook() returns.
    if let Some(registry) = &state.session_registry {
        registry.mark_stopped(&envelope.session_id);
    }

    // BC-2.04.009 PC-5: try_send to event bus (non-blocking, best-effort).
    let received_at = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    let bus_event = EventBusHookEvent::new(bus_event_payload, received_at);
    try_publish_event(&state, bus_event);

    // Ring append — best-effort.
    // DI-001 best-effort: ring append runs after decision but before function return.
    // Under timeout cancellation, this append may be skipped (BC-2.04.009 best-effort caveat).
    let now_micros = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0);
    let record = HookEventRecord::new(
        envelope.session_id.clone(),
        now_micros,
        envelope.pid,
        "Stop".to_owned(),
        None,
        None,
    );
    // Lock, clone Arc out, drop guard BEFORE calling append (CRITICAL-1 constraint:
    // no MutexGuard held across any await point — SS-daemon-wiring-impl.md Round 2).
    let ring_arc = state.ring.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(ring) = ring_arc.as_ref() {
        if let Err(e) = ring.append(record) {
            tracing::warn!(
                session_id = %envelope.session_id,
                error = %e,
                "ring append failed for Stop (best-effort)"
            );
        }
    } else {
        tracing::warn!(
            session_id = %envelope.session_id,
            "ring=None; Stop event not persisted (best-effort)"
        );
    }

    // BC-2.04.009 PC-7: Stop always returns allow (monocle cannot block session termination).
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({"decision": "allow"})),
    )
        .into_response()
}

/// Inner SessionStart handler logic (runs inside the 300ms timeout budget).
async fn handle_session_start_inner(state: Arc<DaemonState>, envelope: HookEnvelope) -> Response {
    use crate::engine::ClaudeCodeModule;
    use crate::event_bus::try_publish_event;
    use crate::ring::HookEventRecord;
    use crate::types::EventBusHookEvent;
    use monocle_core::engine::{EngineModule as _, HookDecision};
    use monocle_core::hook_events::HookEvent;

    // BC-2.04.009 PC-2: look up or create session entry.
    if let Some(registry) = &state.session_registry {
        registry.get_or_create(&envelope.session_id);
    }

    // Construct HookEvent::SessionStart from envelope fields.
    let hook_event: HookEvent = match serde_json::from_value(serde_json::json!({
        "SessionStart": {
            "cwd": envelope.cwd.clone().unwrap_or_default(),
            "transcript_path": envelope.transcript_path.clone().unwrap_or_default(),
            "session_id": envelope.session_id.clone(),
            "pid": envelope.pid
        }
    })) {
        Ok(e) => e,
        Err(err) => {
            tracing::error!(
                session_id = %envelope.session_id,
                error = %err,
                "failed to construct SessionStart HookEvent; returning allow"
            );
            return (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"decision": "allow"})),
            )
                .into_response();
        }
    };

    let bus_event_payload = hook_event.clone();

    // Artificial delay for integration tests exercising the 300ms timeout path.
    // hook_delay_ms is None in production; Some(ms) only in tests.
    if let Some(delay_ms) = state.hook_delay_ms {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    // BC-2.04.009 PC-3: dispatch to EngineModule.
    // hook_decision_override is None in production; Some((decision, diagnostic)) in tests
    // that need to exercise the Defer path (Phase 1 ClaudeCodeModule always returns Allow).
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

    // BC-2.04.009 invariant 2, AC-017, EC-085: Defer → treat as Allow with WARN.
    if response.decision == HookDecision::Defer {
        tracing::warn!(
            session_id = %envelope.session_id,
            "invalid Defer on SessionStart hook; treating as Allow (BC-2.04.009 AC-017, EC-085)"
        );
    }

    // BC-2.04.009 PC-5: try_send to event bus.
    let received_at = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    let bus_event = EventBusHookEvent::new(bus_event_payload, received_at);
    try_publish_event(&state, bus_event);

    // Ring append — best-effort.
    // DI-001 best-effort: ring append runs after decision but before function return.
    // Under timeout cancellation, this append may be skipped (BC-2.04.009 best-effort caveat).
    let now_micros = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0);
    let record = HookEventRecord::new(
        envelope.session_id.clone(),
        now_micros,
        envelope.pid,
        "SessionStart".to_owned(),
        None,
        None,
    );
    // Lock, clone Arc out, drop guard BEFORE calling append (CRITICAL-1 constraint:
    // no MutexGuard held across any await point — SS-daemon-wiring-impl.md Round 2).
    let ring_arc_ss = state.ring.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(ring) = ring_arc_ss.as_ref() {
        if let Err(e) = ring.append(record) {
            tracing::warn!(
                session_id = %envelope.session_id,
                error = %e,
                "ring append failed for SessionStart (best-effort)"
            );
        }
    } else {
        tracing::warn!(
            session_id = %envelope.session_id,
            "ring=None; SessionStart event not persisted (best-effort)"
        );
    }

    // BC-2.04.009 PC-7: SessionStart always returns allow (informational).
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({"decision": "allow"})),
    )
        .into_response()
}

/// Inner PromptSubmit handler logic (runs inside the 300ms timeout budget).
async fn handle_prompt_submit_inner(state: Arc<DaemonState>, envelope: HookEnvelope) -> Response {
    use crate::engine::ClaudeCodeModule;
    use crate::event_bus::try_publish_event;
    use crate::ring::HookEventRecord;
    use crate::types::EventBusHookEvent;
    use monocle_core::engine::{EngineModule as _, HookDecision};
    use monocle_core::hook_events::HookEvent;

    // BC-2.04.009 PC-2: look up or create session entry.
    if let Some(registry) = &state.session_registry {
        registry.get_or_create(&envelope.session_id);
    }

    // Construct HookEvent::UserPromptSubmit from envelope fields.
    let hook_event: HookEvent = match serde_json::from_value(serde_json::json!({
        "UserPromptSubmit": {
            "prompt": envelope.prompt.clone().unwrap_or_default(),
            "session_id": envelope.session_id.clone(),
            "pid": envelope.pid
        }
    })) {
        Ok(e) => e,
        Err(err) => {
            tracing::error!(
                session_id = %envelope.session_id,
                error = %err,
                "failed to construct UserPromptSubmit HookEvent; returning allow"
            );
            return (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"decision": "allow"})),
            )
                .into_response();
        }
    };

    let bus_event_payload = hook_event.clone();

    // Artificial delay for integration tests exercising the 300ms timeout path.
    // hook_delay_ms is None in production; Some(ms) only in tests.
    if let Some(delay_ms) = state.hook_delay_ms {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    // BC-2.04.009 PC-3: dispatch to EngineModule.
    // hook_decision_override is None in production; Some((decision, diagnostic)) in tests
    // that need to exercise the Block path (Phase 1 ClaudeCodeModule always returns Allow).
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

    // BC-2.04.009 PC-7, AC-018: build HTTP response based on decision.
    // No Defer support — Defer treated as Allow with WARN.
    let http_response = match &response.decision {
        HookDecision::Allow => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"decision": "allow"})),
        )
            .into_response(),
        HookDecision::Block => {
            // BC-2.04.009 invariant 4: Block unit variant; reason from HookResponse.diagnostic.
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
            tracing::warn!(
                session_id = %envelope.session_id,
                "invalid Defer on PromptSubmit hook; treating as Allow (BC-2.04.009 invariant 2)"
            );
            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"decision": "allow"})),
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

    // BC-2.04.009 PC-5: try_send to event bus.
    let received_at = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    let bus_event = EventBusHookEvent::new(bus_event_payload, received_at);
    try_publish_event(&state, bus_event);

    // Ring append — best-effort.
    // DI-001 best-effort: ring append runs after decision but before function return.
    // Under timeout cancellation, this append may be skipped (BC-2.04.009 best-effort caveat).
    let now_micros = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0);
    let record = HookEventRecord::new(
        envelope.session_id.clone(),
        now_micros,
        envelope.pid,
        "UserPromptSubmit".to_owned(),
        None,
        None,
    );
    // Lock, clone Arc out, drop guard BEFORE calling append (CRITICAL-1 constraint:
    // no MutexGuard held across any await point — SS-daemon-wiring-impl.md Round 2).
    let ring_arc_ps = state.ring.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(ring) = ring_arc_ps.as_ref() {
        if let Err(e) = ring.append(record) {
            tracing::warn!(
                session_id = %envelope.session_id,
                error = %e,
                "ring append failed for PromptSubmit (best-effort)"
            );
        }
    } else {
        tracing::warn!(
            session_id = %envelope.session_id,
            "ring=None; PromptSubmit event not persisted (best-effort)"
        );
    }

    http_response
}
