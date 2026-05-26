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
//! `build_server` returns the composed `Router` for in-process testing via
//! `tower::ServiceExt::oneshot`. For production daemon startup, use `run_server`, which
//! wires graceful shutdown via `axum::serve(...).with_graceful_shutdown(...)`:
//!
//! ```ignore
//! let state = Arc::new(DaemonState::new());
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
//! run_server(state, listener).await?;
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
use tokio::net::TcpListener;

use crate::auth::auth_middleware;
use crate::body_limit::body_size_limit_middleware;
use crate::handlers::hooks::{
    post_hook_notification, post_hook_pre_tool_use, post_hook_prompt_submit, post_hook_session_start,
    post_hook_stop,
};
use crate::handlers::shutdown::post_shutdown;
use crate::handlers::status::get_status;
use crate::router::unauthenticated_router;
use crate::state::{AppMode, DaemonState};

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

/// Run the monocle daemon HTTP server with graceful shutdown wired to the state channel.
///
/// Calls `axum::serve(listener, build_server(state))` and attaches a
/// `with_graceful_shutdown` future that resolves when either:
///
/// 1. `state.shutdown_rx` receives `true` — triggered by a successful `POST /shutdown`
///    (BC-2.01.004 PC-1 / PC-5), or
/// 2. SIGTERM or SIGINT is received via `tokio::signal` — the OS signal handler sends
///    `true` on `state.shutdown_tx` before this function sees it, OR the signal future
///    resolves directly (whichever fires first).
///
/// When the shutdown future resolves, axum stops accepting new connections and waits for
/// all in-flight requests to complete (the 10-second drain window from BC-2.01.004 INV-1
/// is enforced at the `main` call-site via `tokio::time::timeout`).
///
/// # Errors
///
/// Returns `std::io::Error` if `axum::serve` fails (e.g., listener I/O error).
///
/// # Signal wiring
///
/// SIGTERM and SIGINT are handled here to guarantee that OS-initiated shutdowns also go
/// through the `shutdown_tx` channel so that `AppMode` transitions to `ShuttingDown`
/// before the server stops accepting connections (BC-2.01.004 PC-1 + PC-5). Callers at
/// the `main` level MUST NOT install competing signal handlers.
pub async fn run_server(state: Arc<DaemonState>, listener: TcpListener) -> std::io::Result<()> {
    let app = build_server(Arc::clone(&state));

    // Clone the receiver so we can watch for the shutdown signal from POST /shutdown.
    let mut shutdown_rx = state.shutdown_rx.clone();
    // Clone the sender so OS signals can also trigger the shutdown channel.
    let shutdown_tx = state.shutdown_tx.clone();
    // Clone state into the closure so OS signal branches can set AppMode::ShuttingDown
    // (BC-2.01.004 PC-1 requires AppMode transitions to ShuttingDown for ALL shutdown
    // triggers, including SIGTERM and SIGINT — not only POST /shutdown).
    let signal_state = Arc::clone(&state);

    let shutdown_signal = async move {
        // Wait for the first of: watch channel sends true, SIGTERM, or SIGINT.
        tokio::select! {
            // POST /shutdown (or another sender) signalled shutdown.
            // The post_shutdown handler already sets AppMode::ShuttingDown before sending;
            // no additional mode write needed here.
            result = shutdown_rx.changed() => {
                if let Ok(()) = result {
                    // Value is now true (shutdown triggered by handler).
                    tracing::info!("shutdown signal received via watch channel");
                }
            }
            // OS SIGTERM (systemd/k8s graceful stop).
            _ = async {
                #[cfg(unix)]
                {
                    use tokio::signal::unix::{signal, SignalKind};
                    if let Ok(mut sigterm) = signal(SignalKind::terminate()) {
                        sigterm.recv().await;
                    } else {
                        // If signal registration fails (rare), park forever — the watch
                        // channel or SIGINT branch will handle shutdown.
                        std::future::pending::<()>().await;
                    }
                }
                #[cfg(not(unix))]
                std::future::pending::<()>().await;
            } => {
                tracing::info!("SIGTERM received — initiating graceful shutdown");
                // BC-2.01.004 PC-1: AppMode transitions to ShuttingDown immediately for
                // ALL shutdown triggers, including OS signals.
                *signal_state.mode.write().unwrap_or_else(|e| e.into_inner()) =
                    AppMode::ShuttingDown;
                let _ = shutdown_tx.send(true);
            }
            // SIGINT (Ctrl-C / second Ctrl-C during drain).
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("SIGINT received — initiating graceful shutdown");
                // BC-2.01.004 PC-1: AppMode transitions to ShuttingDown immediately for
                // ALL shutdown triggers, including OS signals.
                *signal_state.mode.write().unwrap_or_else(|e| e.into_inner()) =
                    AppMode::ShuttingDown;
                let _ = shutdown_tx.send(true);
            }
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
}
