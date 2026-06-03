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

    // hook_outer_delay_ms: test-only delay at the outer handler level, BEFORE the 300ms
    // inner timeout budget. Set via MONOCLE_HOOK_DELAY_MS env var (read at daemon startup
    // ONLY when compiled with the `e2e-test-affordances` cargo feature).
    // Used by AC-E2E-007 to create a genuinely in-flight request that holds axum's
    // graceful-shutdown drain open for the drain-timeout test (C2 fix).
    // HIGH-1 fix: gated behind cfg(feature = "e2e-test-affordances") so production binaries
    // compiled without the feature never execute this code path (SS-daemon-wiring-impl.md
    // §Fix Addendum Round 2 HIGH-1).
    #[cfg(feature = "e2e-test-affordances")]
    if let Some(delay_ms) = state.hook_outer_delay_ms {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    // Track any deferred prompt_id registered during this handler invocation.
    // The inner handler writes to this cell when a Defer prompt is registered;
    // the timeout handler reads it to clean up the registry entry and broadcast
    // PermissionPromptResolved (BC-2.05.005 postcondition PC-4).
    // Arc<Mutex<Option<Uuid>>> is required here rather than Cell<Option<Uuid>>: the value
    // is shared across two async closures (inner handler + timeout handler) that may run
    // on different OS threads under tokio's work-stealing scheduler. Cell<T> is not Send,
    // so it cannot be moved into an async block that crosses an await point on a multi-
    // thread runtime. Mutex adds minimal overhead for a single write/read pair.
    let deferred_prompt_id: std::sync::Arc<std::sync::Mutex<Option<uuid::Uuid>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));

    // Wrap entire handler logic in 300ms timeout (BC-2.04.007 PC-4, invariant 1).
    let result = tokio::time::timeout(
        Duration::from_millis(PRE_TOOL_USE_TIMEOUT_MS),
        handle_pre_tool_use_inner(
            Arc::clone(&state),
            envelope,
            std::sync::Arc::clone(&deferred_prompt_id),
        ),
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

            // Step 6 (BC-2.05.005 PC-4): If a Defer prompt was registered, clean up
            // the registry and broadcast PermissionPromptResolved to all TUI clients.
            //
            // Read the prompt_id and immediately release the Mutex before any .await
            // (std::sync::MutexGuard is not Send across .await boundaries).
            let timed_out_prompt_id: Option<uuid::Uuid> =
                deferred_prompt_id.lock().ok().and_then(|guard| *guard);

            if let Some(prompt_id) = timed_out_prompt_id {
                // If a concurrent PermissionDecision arrives during the tokio::time::timeout
                // → Err window, handle_permission_decision will have already removed the
                // registry entry and broadcast PermissionPromptResolved. Only broadcast here
                // if WE removed the entry (Some return value). Per F-S022-ADV11-LOW-001.
                let removed = state
                    .pending_decisions
                    .as_ref()
                    .and_then(|registry| registry.remove_timed_out_prompt(prompt_id));
                if removed.is_some() {
                    // Broadcast PermissionPromptResolved so TUI clients dismiss the overlay.
                    if let Some(subscribers) = state.ipc_subscribers.as_ref() {
                        let resolved_msg =
                            monocle_ipc::types::ServerToClient::PermissionPromptResolved {
                                prompt_id,
                            };
                        crate::ipc_server::broadcast_to_subscribers(subscribers, resolved_msg)
                            .await;
                        tracing::info!(
                            %prompt_id,
                            "PreToolUse timeout: PermissionPromptResolved broadcast to TUI clients"
                        );
                    }
                }
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({"decision": "allow", "reason": "timeout"})),
            )
                .into_response()
        }
    }
}

/// Inner handler logic (runs inside the 300ms timeout budget).
///
/// `deferred_prompt_id` is written with the registered prompt_id if the engine returns
/// `HookDecision::Defer`. The outer handler reads this cell after the timeout fires to
/// clean up the registry and broadcast `PermissionPromptResolved`.
async fn handle_pre_tool_use_inner(
    state: Arc<DaemonState>,
    envelope: HookEnvelope,
    deferred_prompt_id: std::sync::Arc<std::sync::Mutex<Option<uuid::Uuid>>>,
) -> Response {
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
            // BC-2.04.007 invariant 5: Defer → register prompt + broadcast PermissionPromptQueued
            // to all connected TUI clients; await the decision oneshot.
            // The outer 300ms timeout wrapping this function fires on no-decision.
            use monocle_ipc::types::{PermissionDecisionKind, PromptPayloadInputs};

            // Step 1: Build PromptPayloadInputs from envelope fields.
            // No prompt_id — the registry assigns it (F-ADV2-MED-001).
            let inputs = PromptPayloadInputs {
                session_id: envelope.session_id.clone(),
                tool_name: envelope.tool_name.clone().unwrap_or_default(),
                tool_input: envelope
                    .tool_input
                    .clone()
                    .unwrap_or(serde_json::Value::Null),
                old_content: None,
                new_content: None,
            };

            // Steps 2+3: Create decision oneshot; register prompt → obtain stable prompt_id
            // and complete payload (both returned by register_prompt).
            let defer_response = if let Some(registry) = state.pending_decisions.as_ref() {
                let (decision_tx, decision_rx) =
                    tokio::sync::oneshot::channel::<PermissionDecisionKind>();
                let (prompt_id, payload) = registry.register_prompt(inputs, decision_tx);

                // Record the prompt_id so the outer timeout handler can clean up.
                if let Ok(mut guard) = deferred_prompt_id.lock() {
                    *guard = Some(prompt_id);
                }

                // Step 4: Broadcast PermissionPromptQueued to all connected TUI clients.
                if let Some(subscribers) = state.ipc_subscribers.as_ref() {
                    let queued_msg = monocle_ipc::types::ServerToClient::PermissionPromptQueued {
                        payload: payload.clone(),
                    };
                    crate::ipc_server::broadcast_to_subscribers(subscribers, queued_msg).await;
                    tracing::info!(
                        session_id = %envelope.session_id,
                        %prompt_id,
                        "PreToolUse Defer: PermissionPromptQueued broadcast to TUI clients"
                    );
                } else {
                    tracing::warn!(
                        session_id = %envelope.session_id,
                        %prompt_id,
                        "PreToolUse Defer: ipc_subscribers=None; PermissionPromptQueued not broadcast"
                    );
                }

                // Step 5: Await decision_rx inside the existing outer 300ms timeout.
                match decision_rx.await {
                    Ok(PermissionDecisionKind::Allow) => (
                        axum::http::StatusCode::OK,
                        Json(serde_json::json!({"decision": "allow", "reason": "user-approved"})),
                    )
                        .into_response(),
                    Ok(PermissionDecisionKind::AcceptAlways) => {
                        // BC-2.06.012 PC-1: forward {"decision":"always"} to Claude Code hook
                        // protocol (SS-ipc §942). Persistent allow-pattern recording is a
                        // future story (no Phase-1 BC mandates monocle-side pattern persistence).
                        // The TUI sends AcceptAlways; the daemon forwards the wire decision.
                        (
                            axum::http::StatusCode::OK,
                            Json(serde_json::json!({"decision": "always", "reason": "user-approved-always"})),
                        )
                            .into_response()
                    }
                    Ok(PermissionDecisionKind::Deny) => (
                        axum::http::StatusCode::OK,
                        Json(serde_json::json!({"decision": "block", "reason": "user-denied"})),
                    )
                        .into_response(),
                    Err(_dropped) => {
                        // Oneshot sender dropped (e.g., daemon shutting down).
                        // Fail-open: allow.
                        tracing::warn!(
                            session_id = %envelope.session_id,
                            %prompt_id,
                            "PreToolUse Defer: decision oneshot dropped; failing open"
                        );
                        (
                                axum::http::StatusCode::OK,
                                Json(serde_json::json!({"decision": "allow", "reason": "oneshot-dropped"})),
                            )
                                .into_response()
                    }
                }
            } else {
                // pending_decisions=None (test stub without registry).
                // Await a never-resolving future so the outer 300ms timeout fires.
                tracing::warn!(
                    session_id = %envelope.session_id,
                    "PreToolUse Defer: pending_decisions=None; awaiting timeout (fail-open)"
                );
                let (_tx, rx) = tokio::sync::oneshot::channel::<()>();
                let _ = rx.await;
                // Unreachable — outer timeout fires first.
                (
                    axum::http::StatusCode::OK,
                    Json(serde_json::json!({"decision": "allow", "reason": "defer-no-registry"})),
                )
                    .into_response()
            };

            defer_response

            // Step 6 (timeout path): Handled by the outer timeout in `post_hook_pre_tool_use`.
            // When the 300ms timeout fires, it broadcasts PermissionPromptResolved and
            // removes the registry entry. See the timeout arm in `post_hook_pre_tool_use`.
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
    // Lock, clone Arc out, drop guard BEFORE calling append (which is async-safe but
    // the guard must not be held across any await point — CRITICAL-1 constraint).
    let ring_arc = state.ring.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(ring) = ring_arc.as_ref() {
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
