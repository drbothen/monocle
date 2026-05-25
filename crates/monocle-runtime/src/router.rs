//! Axum router construction for the monocle daemon.
//!
//! Two distinct routers per SS-daemon-lifecycle.md v1.0.33 §Body Size Limit:
//! - [`unauthenticated_router`] — hosts `GET /healthz` only; no body limit; no auth.
//! - Authenticated router (S-003+) — hosts hook endpoints and admin endpoints;
//!   enforces `DefaultBodyLimit::max(256 * 1024)` and the dual-accept auth middleware.
//!
//! The two routers are merged by the daemon entry-point (S-002/S-003 wiring).

use std::sync::Arc;

use axum::Router;

use crate::state::DaemonState;

/// Construct the unauthenticated axum router.
///
/// Registers:
/// - `GET /healthz` → [`crate::handlers::healthz::get_healthz`]
///
/// No `DefaultBodyLimit` is applied (BC-2.01.001 postcondition 4).
/// No auth middleware is applied (BC-2.01.001 postcondition 3, invariant 2).
pub fn unauthenticated_router(state: Arc<DaemonState>) -> Router {
    let _ = state;
    unimplemented!("S-002: unauthenticated_router — Router::new().route(\"/healthz\", get(get_healthz)).with_state(state)")
}
