//! `POST /shutdown` — authenticated graceful-shutdown trigger handler.
//!
//! Returns HTTP 501 (stub) until S-005 implementation wires the actual shutdown logic.
//! The handler is registered on the authenticated router (`build_server`) so that the
//! dual-accept auth middleware (ADR-0005) enforces authentication before this stub is reached.
//!
//! # Behavioral contract
//!
//! BC-2.01.004 (Graceful Shutdown — 10-Second Drain). See story S-005.
//!
//! # Architecture constraints (S-005)
//!
//! - `POST /shutdown` requires valid auth (canonical OR alias per ADR-0005 dual-accept).
//! - On success (HTTP 200), transitions `AppMode` to `ShuttingDown` and returns
//!   `{"status":"shutting_down"}`.
//! - MUST NOT call `std::process::exit` directly — use `lifecycle::exit_with(DaemonExit::…)`.
//! - All hook POST requests arriving after `AppMode::ShuttingDown` is set return HTTP 503
//!   with `Retry-After: 10` and body `{"error":"daemon_shutting_down"}` (BC-2.01.004 PC-2).
//!
//! # Red Gate
//!
//! This stub returns HTTP 501 Not Implemented. All tests for S-005 shutdown behavior
//! MUST FAIL against this stub to satisfy the Red Gate requirement (TDD strict mode).

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::state::DaemonState;

/// Stub handler for `POST /shutdown`.
///
/// Returns HTTP 501 Not Implemented until the S-005 implementation replaces this stub
/// with the actual graceful-shutdown logic (AppMode transition, drain coordination,
/// tokio signal wiring, `lifecycle::exit_with(DaemonExit::Graceful)`).
///
/// Authentication is enforced by the auth middleware layer on the authenticated router —
/// this handler is only reached after the dual-accept protocol succeeds.
pub async fn post_shutdown(State(_state): State<Arc<DaemonState>>) -> impl IntoResponse {
    // Stub: implementation MUST return HTTP 200 + {"status":"shutting_down"} and initiate
    // AppMode transition, drain, and lifecycle::exit_with(DaemonExit::Graceful).
    // Returning 501 here ensures the Red Gate fails as required by TDD strict mode.
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "not_implemented"})),
    )
}
