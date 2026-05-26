//! Daemon shared state types.
//!
//! Provides [`AppMode`] and [`DaemonState`], the central shared-state struct threaded
//! through all axum handlers via `State(state): State<Arc<DaemonState>>`.
//!
//! Populated by S-002 (healthz endpoint). Extended by S-003 (auth token, lock-file,
//! last-hook timestamps, TUI attachment flag), S-004 (lock-file path write),
//! S-005 (hook-ingestion channels).

use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};
use std::time::Instant;

use tokio::sync::watch;

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

/// Per-hook-type timestamps for the most recent invocation of each hook endpoint.
///
/// All fields are `Option<String>` — `None` means the hook has not fired since daemon
/// start. When set, the string is an ISO 8601 UTC timestamp with mandatory millisecond
/// precision (`YYYY-MM-DDTHH:MM:SS.sssZ`), per BC-2.01.002 PC-1 sub-bullet `last_hook_ts`
/// and EC-044.
///
/// Stored inside `DaemonState.last_hook_ts` under an `RwLock` so that hook-receiver
/// tasks (S-005) can write concurrently while the `/status` handler reads.
#[derive(Debug, Default, Clone)]
pub struct LastHookTimestamps {
    /// Timestamp of most recent `POST /hooks/pre-tool-use` invocation.
    pub pre_tool_use: Option<String>,
    /// Timestamp of most recent `POST /hooks/notification` invocation.
    pub notification: Option<String>,
    /// Timestamp of most recent `POST /hooks/stop` invocation.
    pub stop: Option<String>,
    /// Timestamp of most recent `POST /hooks/session-start` invocation.
    pub session_start: Option<String>,
    /// Timestamp of most recent `POST /hooks/prompt-submit` invocation.
    pub prompt_submit: Option<String>,
}

/// Central shared state threaded through all axum handlers.
///
/// All fields requiring interior mutability use `RwLock` (sync for read-heavy paths)
/// or `AtomicBool` for single-flag state. The struct itself is `Send + Sync` and wrapped
/// in `Arc<DaemonState>` for sharing across handler tasks.
///
/// # Architecture
///
/// Per SS-daemon-lifecycle.md v1.0.33 §Health and Status Endpoints:
/// - `mode` drives the 200/503 split in `GET /healthz`.
/// - `start_time` is read-only after construction; `Instant` is `Copy`.
/// - `auth_token` holds the raw 64-hex-char secret; NEVER compared with `==` (use
///   `constant_time_eq::constant_time_eq` on both canonical and alias auth paths, NFR-010).
/// - `lock_file_path` is written once by S-004 during daemon startup; read by `/status`.
/// - `last_hook_ts` is updated by S-005 hook-receiver tasks on each hook invocation.
/// - `tui_attached` is an atomic flag set when a TUI client establishes a session.
/// - `shutdown_tx` / `shutdown_rx`: graceful-shutdown notification channel (S-005).
///   The `POST /shutdown` handler sends `true`; `run_server` wires a clone of `shutdown_rx`
///   into `axum::serve(...).with_graceful_shutdown(...)` so the HTTP server stops accepting
///   new connections once shutdown is signalled (BC-2.01.004 PC-5).
#[derive(Debug)]
pub struct DaemonState {
    /// Current operating mode of the daemon.
    ///
    /// Written during graceful shutdown (BC-2.01.004); read by every health/status handler.
    pub mode: RwLock<AppMode>,

    /// Monotonic timestamp recorded at daemon startup.
    ///
    /// Used to compute `uptime_sec` in the `/healthz` and `/status` response bodies.
    pub start_time: Instant,

    /// Hook-receiver supervisor watch channel (AC-002).
    ///
    /// `None` — no hook-receiver has been wired yet (normal in S-002; S-005 wires it).
    ///   Treated as healthy.
    /// `Some(rx)` where `rx.borrow().is_ok()` — hook-receiver is alive and healthy.
    /// `Some(rx)` where `rx.borrow().is_err()` — hook-receiver task exited abnormally;
    ///   supervisor set the channel to `Err` on termination detection via
    ///   `JoinHandle::is_finished()`. The healthz handler returns HTTP 503 in this case.
    pub hook_receiver_status: Option<watch::Receiver<Result<(), String>>>,

    /// Raw 64-hex-char auth token for this daemon session (BC-2.01.009, ADR-0005).
    ///
    /// Stored WITHOUT the `monocle-v1:` prefix — that prefix is a wire-format concern only.
    /// Comparison MUST use `constant_time_eq::constant_time_eq`; NEVER `==` (NFR-010).
    ///
    /// Initialized to an empty string at construction; written exactly once during daemon
    /// startup by the token-generation path (S-004). An empty token means auth has not
    /// been initialized — the auth middleware detects this via an `is_empty()` guard placed
    /// before any header extraction and rejects ALL authenticated requests with HTTP 401
    /// `{"error":"invalid_auth_token"}`. This prevents the empty-credential bypass where
    /// `constant_time_eq("", "")` would return `true` against an unset token.
    pub auth_token: String,

    /// Absolute path to the daemon's lock file on disk.
    ///
    /// Written once during daemon startup (S-004); read by the `/status` handler and
    /// surfaced in the `lock_file` response field (BC-2.01.002 PC-1). Empty string means
    /// the lock file has not been written yet.
    pub lock_file_path: String,

    /// Most-recent invocation timestamp for each hook endpoint type.
    ///
    /// Protected by a `std::sync::RwLock` (sync, not tokio) so that multiple Tokio worker
    /// threads can read concurrently while a hook-receiver task holds the write guard
    /// transiently to update a single field.
    pub last_hook_ts: RwLock<LastHookTimestamps>,

    /// Whether a TUI client is currently attached to this daemon session.
    ///
    /// Set to `true` when a TUI session is established; `false` when the TUI disconnects.
    /// Read by the `/status` handler and surfaced as `tui_attached` (BC-2.01.002 PC-1).
    ///
    /// `Ordering::Relaxed` is sufficient: this flag is informational and is read only in
    /// the `/status` handler which already tolerates eventual consistency for floating-point
    /// fill percentages.
    pub tui_attached: AtomicBool,

    /// RAII guard for the daemon lock file (BC-2.01.004 PC-7).
    ///
    /// `Some(lock)` — the lock file is held on disk. The graceful-shutdown handler takes
    /// this value (leaving `None`) and calls `lock.release()` before signalling process exit.
    ///
    /// `None` — either the daemon has not yet acquired the lock (startup), or the lock has
    /// already been released (shutdown path). The `POST /shutdown` handler is a no-op on
    /// the lock when this is `None`.
    ///
    /// Protected by a `std::sync::Mutex` (sync, not tokio) because `DaemonLock` is not
    /// `Send` to async task boundaries and the critical section is purely a take-and-drop.
    pub daemon_lock: Mutex<Option<crate::lock::DaemonLock>>,

    /// Graceful-shutdown notification sender (BC-2.01.004 PC-5, S-005).
    ///
    /// The `POST /shutdown` handler sends `true` after setting `AppMode::ShuttingDown` and
    /// releasing the lock file. `run_server` clones `shutdown_rx` and passes it to
    /// `axum::serve(...).with_graceful_shutdown(...)` so the HTTP listener stops accepting
    /// new connections once the signal fires.
    ///
    /// Sending `false` has no semantic meaning — callers MUST send `true` to trigger shutdown.
    /// The sender is never dropped before shutdown completes because `DaemonState` outlives
    /// the server task.
    pub shutdown_tx: watch::Sender<bool>,

    /// Graceful-shutdown notification receiver (BC-2.01.004 PC-5, S-005).
    ///
    /// Cloned by `run_server` and passed into `with_graceful_shutdown`. Additional clones
    /// may be held by SIGTERM/SIGINT signal tasks to trigger the same shutdown path from
    /// OS signals. Sending on `shutdown_tx` wakes all receivers simultaneously.
    pub shutdown_rx: watch::Receiver<bool>,
}

impl DaemonState {
    /// Creates a new `DaemonState` with `AppMode::Running` and the current time.
    ///
    /// `hook_receiver_status` is initialized to `None` — no hook-receiver is wired at
    /// construction time. S-005 wires the channel once the hook-receiver task is spawned.
    ///
    /// `auth_token` and `lock_file_path` are empty strings. S-004 writes both during
    /// daemon startup after generating the token and claiming the lock file.
    ///
    /// `shutdown_tx` / `shutdown_rx` are initialized with `false` (not shutdown). The
    /// `POST /shutdown` handler sends `true` to trigger graceful shutdown.
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            mode: RwLock::new(AppMode::Running),
            start_time: Instant::now(),
            hook_receiver_status: None,
            auth_token: String::new(),
            lock_file_path: String::new(),
            last_hook_ts: RwLock::new(LastHookTimestamps::default()),
            tui_attached: AtomicBool::new(false),
            daemon_lock: Mutex::new(None),
            shutdown_tx,
            shutdown_rx,
        }
    }
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn _check() {
        assert_send_sync::<DaemonState>();
    }
};
