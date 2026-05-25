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

use crate::state::DaemonState;

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
    let _ = state;
    // Stub: implementation reads AppMode, computes uptime_sec from start_time,
    // serializes JSON body, and returns the correct HTTP status code.
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}
