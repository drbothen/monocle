//! `GET /healthz` — unauthenticated liveness probe handler.
//!
//! Returns HTTP 200 `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}` when
//! the daemon is running normally, or HTTP 503 `{"status":"shutting_down"}` when
//! `AppMode` is `ShuttingDown` or the hook-receiver task has exited abnormally.
//!
//! # Behavioral contract
//!
//! BC-2.01.001 (Healthz Endpoint — Unauthenticated Liveness Probe). See story S-002.
//!
//! # Architecture constraints
//!
//! - This handler MUST NOT import `constant_time_eq` (auth path only).
//! - This handler MUST NOT import from `monocle-tui` (not a Phase 1 crate).
//! - Registered on the UNAUTHENTICATED router only (`router::unauthenticated_router`).

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;

use crate::state::{AppMode, DaemonState};

/// Unauthenticated liveness probe.
///
/// Postconditions (BC-2.01.001):
/// - Normal mode + hook-receiver alive → HTTP 200
///   `{"status":"alive","uptime_sec":<N>,"version":"<semver>"}`.
/// - `AppMode::ShuttingDown` OR hook-receiver abnormally exited → HTTP 503
///   `{"status":"shutting_down"}`.
/// - No `X-Monocle-Authorization` header required or inspected.
/// - No `DefaultBodyLimit` applied.
pub async fn get_healthz(State(state): State<Arc<DaemonState>>) -> Response {
    // A poisoned RwLock means a handler panicked while holding the write lock —
    // the daemon state is unrecoverable. Treat it as ShuttingDown to surface
    // the degraded condition to the TUI and monitoring infrastructure.
    let mode = state
        .mode
        .read()
        .map(|guard| guard.clone())
        .unwrap_or(AppMode::ShuttingDown);

    match mode {
        AppMode::Running => {
            let uptime_sec: u64 = state.start_time.elapsed().as_secs();
            let version: &str = env!("CARGO_PKG_VERSION");
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "alive",
                    "uptime_sec": uptime_sec,
                    "version": version,
                })),
            )
                .into_response()
        }
        AppMode::ShuttingDown => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "shutting_down",
            })),
        )
            .into_response(),
    }
}
