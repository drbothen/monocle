//! `POST /shutdown` — authenticated graceful-shutdown trigger handler.
//!
//! # Behavioral contract
//!
//! BC-2.01.004 (Graceful Shutdown — 10-Second Drain).
//!
//! # Architecture constraints (S-005)
//!
//! - `POST /shutdown` requires valid auth (canonical OR alias per ADR-0005 dual-accept).
//! - On first call (AppMode::Running → ShuttingDown): transitions `AppMode` to `ShuttingDown`,
//!   releases the daemon lock file, and returns HTTP 200 `{"status":"shutting_down"}`.
//! - On second call (AppMode already ShuttingDown — EC-050): acknowledges with HTTP 200
//!   `{"status":"shutting_down"}` without further state change.
//! - MUST NOT call `std::process::exit` directly — use `lifecycle::exit_with(DaemonExit::…)`.
//! - All hook POST requests arriving after `AppMode::ShuttingDown` is set return HTTP 503
//!   with `Retry-After: 10` and body `{"error":"daemon_shutting_down"}` (BC-2.01.004 PC-2).

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::state::{AppMode, DaemonState};

/// Handler for `POST /shutdown`.
///
/// First call: transitions `AppMode` from `Running` to `ShuttingDown`, releases the daemon
/// lock file (BC-2.01.004 PC-7), and returns HTTP 200 `{"status":"shutting_down"}`.
///
/// Subsequent calls while already in `ShuttingDown` mode (EC-050): acknowledges with the
/// same HTTP 200 `{"status":"shutting_down"}` response without further state change.
///
/// Authentication is enforced by the auth middleware layer on the authenticated router —
/// this handler is only reached after the dual-accept protocol succeeds (ADR-0005).
pub async fn post_shutdown(State(state): State<Arc<DaemonState>>) -> impl IntoResponse {
    // Transition AppMode to ShuttingDown (idempotent — no-op if already ShuttingDown).
    //
    // A poisoned write lock means a handler panicked while holding it — the daemon state
    // is unrecoverable. Proceed with shutdown anyway (best-effort): log the condition
    // and continue to release the lock file and return 200.
    match state.mode.write() {
        Ok(mut mode) => {
            *mode = AppMode::ShuttingDown;
        }
        Err(poisoned) => {
            tracing::error!(
                "RwLock<AppMode> poisoned; daemon state unrecoverable. \
                Proceeding with shutdown (best-effort) to release lock file."
            );
            // Recover the guard and overwrite to ShuttingDown regardless.
            let mut mode = poisoned.into_inner();
            *mode = AppMode::ShuttingDown;
        }
    }

    // Release the daemon lock file if it has not yet been released.
    //
    // BC-2.01.004 PC-7: lock file MUST be absent after graceful shutdown completes.
    // SS-conventions-anti-patterns.md: NEVER call std::process::exit here.
    //
    // Two release paths:
    //
    // 1. RAII guard path (production binary): take the DaemonLock out of daemon_lock,
    //    call lock.release() — removes both .lock and .sock files.
    //
    // 2. Path-based fallback (test / startup-before-lock-wired path): if daemon_lock is None
    //    but lock_file_path is set, remove the file by path directly. This satisfies the
    //    integration test which acquires a real lock but stores only the path in state, not
    //    the RAII guard.
    let lock = match state.daemon_lock.lock() {
        Ok(mut guard) => guard.take(),
        Err(poisoned) => {
            tracing::error!(
                "daemon_lock Mutex poisoned during graceful shutdown; \
                attempting path-based lock file removal as fallback."
            );
            // Recover from poisoning and take the inner value.
            poisoned.into_inner().take()
        }
    };

    if let Some(lock) = lock {
        // Primary path: release via RAII guard (removes .lock + .sock).
        if let Err(e) = lock.release() {
            tracing::warn!(
                error = %e,
                "lock file release failed during graceful shutdown (BC-2.01.004 PC-7); \
                next daemon start may need to clean up stale lock"
            );
        }
    } else if !state.lock_file_path.is_empty() {
        // Fallback path: release by removing the files at the recorded path.
        // Used when the binary has not yet wired the DaemonLock into state, or in tests.
        // Both the lock file and the companion .sock file are removed (BC-2.01.004 PC-7,
        // BC-2.01.005 PC-6 lock+sock removal invariant).
        let lock_path = std::path::Path::new(&state.lock_file_path);
        if let Err(e) = std::fs::remove_file(lock_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    path = %lock_path.display(),
                    error = %e,
                    "lock file path-based removal failed during graceful shutdown \
                    (BC-2.01.004 PC-7)"
                );
            }
        }
        // Remove the companion .sock file (same stem, .sock extension).
        let sock_path = lock_path.with_extension("sock");
        if let Err(e) = std::fs::remove_file(&sock_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    path = %sock_path.display(),
                    error = %e,
                    "sock file path-based removal failed during graceful shutdown \
                    (BC-2.01.005 PC-6)"
                );
            }
        }
    }

    // Signal the server to stop accepting new connections (BC-2.01.004 PC-5).
    //
    // `send()` fails only if all receivers have been dropped, which means the server
    // task is already gone — a harmless no-op in that case.
    let _ = state.shutdown_tx.send(true);

    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "shutting_down"})),
    )
}
