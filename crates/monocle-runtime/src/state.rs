//! Daemon shared state types.
//!
//! Provides [`AppMode`] and [`DaemonState`], the central shared-state struct threaded
//! through all axum handlers via `State(state): State<Arc<DaemonState>>`.
//!
//! Populated by S-002 (healthz endpoint). Extended by S-003 (auth token field),
//! S-004 (lock-file field), S-005 (hook-ingestion channels).

use std::sync::RwLock;
use std::time::Instant;

/// Operating mode of the monocle daemon.
///
/// Drives the 200/503 split in `GET /healthz` (BC-2.01.001 postcondition 1/2)
/// and the graceful-shutdown gate in BC-2.01.004.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    /// Normal operating mode — daemon is accepting hook events.
    Running,
    /// Graceful-shutdown in progress — daemon is draining, not accepting new connections.
    ShuttingDown,
}

/// Central shared state threaded through all axum handlers.
///
/// All fields requiring interior mutability use `RwLock` (or `tokio::sync::RwLock`
/// for async-write paths). The struct itself is `Send + Sync` and wrapped in
/// `Arc<DaemonState>` for sharing across handler tasks.
///
/// # Architecture
///
/// Per SS-daemon-lifecycle.md v1.0.33 §Health and Status Endpoints:
/// - `mode` drives the 200/503 split in `GET /healthz`.
/// - `start_time` is read-only after construction; `Instant` is `Copy`.
pub struct DaemonState {
    /// Current operating mode of the daemon.
    ///
    /// Written during graceful shutdown (BC-2.01.004); read by every health/status handler.
    pub mode: RwLock<AppMode>,

    /// Monotonic timestamp recorded at daemon startup.
    ///
    /// Used to compute `uptime_sec` in the `/healthz` and `/status` response bodies.
    pub start_time: Instant,
}
