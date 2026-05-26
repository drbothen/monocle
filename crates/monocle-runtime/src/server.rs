//! Full axum server construction for the monocle daemon.
//!
//! Merges the unauthenticated router (healthz) with the authenticated router
//! (status + hook endpoints). The authenticated router has:
//! - Auth middleware (`auth_middleware` from `auth.rs`, ADR-0005 dual-accept).
//! - `DefaultBodyLimit::max(262144)` (256 KiB, per SS-daemon-lifecycle.md v1.0.33 §Body Size Limit).
//! - `body_size_limit_middleware` (S-004): custom JSON 413 handler for oversized requests with
//!   `Content-Length > 262144`. Returns `{"error":"payload_too_large","limit_bytes":262144}`.
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
//! - Authenticated: `GET /status` (auth middleware + 256 KiB body limit + custom 413 handler).
//!   S-009 will extend the authenticated router with 5 hook POST routes.
//!
//! # Middleware layer ordering on the authenticated router
//!
//! Layers are applied outermost-first (last `.layer()` call = outermost = runs first):
//! ```text
//! Request flow: auth_middleware → body_size_limit_middleware → DefaultBodyLimit → handler
//! ```
//! Auth runs first to reject unauthenticated requests cheaply.
//! Body size check runs after auth: only authenticated requests have their body size evaluated.
//! `DefaultBodyLimit` signals extractors to enforce the 256 KiB limit when reading the body.

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::{middleware, Router};

use crate::auth::auth_middleware;
use crate::body_limit::body_size_limit_middleware;
use crate::handlers::hooks::{
    post_hook_notification, post_hook_pre_tool_use, post_hook_prompt_submit, post_hook_session_start,
    post_hook_stop,
};
use crate::handlers::shutdown::post_shutdown;
use crate::handlers::status::get_status;
use crate::router::unauthenticated_router;
use crate::state::DaemonState;

/// Construct the full axum router for the monocle daemon.
///
/// Merges the unauthenticated router (hosts `GET /healthz` only, no body limit, no auth)
/// with the authenticated router (hosts `GET /status`, with auth middleware, body-size
/// enforcement, and `DefaultBodyLimit::max(262144)` per SS-daemon-lifecycle.md v1.0.33
/// §Body Size Limit).
///
/// # Postconditions
///
/// - `GET /healthz` is reachable without auth and without body limit.
/// - `GET /status` is behind the auth middleware (BC-2.01.009 dual-accept, ADR-0005).
/// - `DefaultBodyLimit::max(262144)` applies to the authenticated router only
///   (BC-2.01.003 Invariant 2 / BC-2.01.002 §Architecture Compliance).
/// - Requests with `Content-Length > 262144` on the authenticated router return
///   HTTP 413 with JSON body `{"error":"payload_too_large","limit_bytes":262144}`
///   (S-004 custom 413 handler, via `body_size_limit_middleware`).
pub fn build_server(state: Arc<DaemonState>) -> Router {
    let unauth_routes = unauthenticated_router(Arc::clone(&state));

    let auth_routes = Router::new()
        .route("/status", get(get_status))
        // POST /shutdown: authenticated graceful-shutdown trigger (BC-2.01.004, S-005).
        // Registered on the authenticated router so the dual-accept auth middleware runs
        // before the handler is reached (BC-2.01.004 INV-3 + ADR-0005).
        .route("/shutdown", post(post_shutdown))
        // Hook routes: 5 canonical endpoints (BC-2.01.004 PC-2, S-005 + S-009).
        // S-005 registers shutdown-aware stubs that gate on AppMode::ShuttingDown.
        // S-009 replaces the non-drain branch with full hook ingestion logic.
        .route("/hooks/pre-tool-use", post(post_hook_pre_tool_use))
        .route("/hooks/notification", post(post_hook_notification))
        .route("/hooks/stop", post(post_hook_stop))
        .route("/hooks/session-start", post(post_hook_session_start))
        .route("/hooks/prompt-submit", post(post_hook_prompt_submit))
        // DefaultBodyLimit signals extractors (Bytes, Json, Form) to enforce 256 KiB.
        // Must appear innermost (first .layer() call) so it wraps the routes directly.
        .layer(DefaultBodyLimit::max(262144))
        // body_size_limit_middleware: custom JSON 413 for Content-Length-bearing oversized
        // requests. Runs after auth (auth is outermost, this is middle layer).
        .layer(middleware::from_fn(body_size_limit_middleware))
        // auth_middleware: outermost layer — runs first, rejects unauthenticated requests
        // before body size or route dispatch.
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            auth_middleware,
        ))
        .with_state(Arc::clone(&state));

    unauth_routes.merge(auth_routes)
}
