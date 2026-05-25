//! Full axum server construction for the monocle daemon.
//!
//! Merges the unauthenticated router (healthz) with the authenticated router
//! (status + hook endpoints). The authenticated router has:
//! - Auth middleware (`auth_middleware` from `auth.rs`, ADR-0005 dual-accept).
//! - `DefaultBodyLimit::max(262144)` (256 KiB, per SS-daemon-lifecycle.md v1.0.33 §Body Size Limit).
//!
//! # Usage
//!
//! `build_server` is the single entry point for daemon startup (S-002/S-003 wiring):
//!
//! ```ignore
//! let state = Arc::new(DaemonState::new());
//! let app = build_server(Arc::clone(&state));
//! axum::serve(listener, app).await?;
//! ```
//!
//! # Router split
//!
//! Two routers are constructed separately and then merged via axum `Router::merge`:
//! - Unauthenticated: `GET /healthz` (no auth, no body limit).
//! - Authenticated: `GET /status` (auth middleware + 256 KiB body limit).
//!   S-009 will extend the authenticated router with 5 hook POST routes.

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::get;
use axum::{middleware, Router};

use crate::auth::auth_middleware;
use crate::handlers::status::get_status;
use crate::router::unauthenticated_router;
use crate::state::DaemonState;

/// Construct the full axum router for the monocle daemon.
///
/// Merges the unauthenticated router (hosts `GET /healthz` only, no body limit, no auth)
/// with the authenticated router (hosts `GET /status`, with auth middleware and
/// `DefaultBodyLimit::max(262144)` per SS-daemon-lifecycle.md v1.0.33 §Body Size Limit).
///
/// # Postconditions
///
/// - `GET /healthz` is reachable without auth and without body limit.
/// - `GET /status` is behind the auth middleware (BC-2.01.009 dual-accept, ADR-0005).
/// - `DefaultBodyLimit::max(262144)` applies to the authenticated router only
///   (BC-2.01.001 Invariant 2 / BC-2.01.002 §Architecture Compliance).
pub fn build_server(state: Arc<DaemonState>) -> Router {
    let unauth_routes = unauthenticated_router(Arc::clone(&state));

    let auth_routes = Router::new()
        .route("/status", get(get_status))
        .layer(DefaultBodyLimit::max(262144))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            auth_middleware,
        ))
        .with_state(Arc::clone(&state));

    unauth_routes.merge(auth_routes)
}
