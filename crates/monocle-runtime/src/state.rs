//! Daemon shared state types.
//!
//! Provides [`AppMode`] and [`DaemonState`], the central shared-state struct threaded
//! through all axum handlers via `State(state): State<Arc<DaemonState>>`.
//!
//! Populated by S-002 (healthz endpoint). Extended by S-003 (auth token, lock-file,
//! last-hook timestamps, TUI attachment flag), S-004 (lock-file path write),
//! S-005 (hook-ingestion channels), S-018 (event bus, drop counter, session registry).

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

use tokio::sync::watch;

use crate::ring::{RingBuffer, RingShutdownNotify};

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
/// - `tui_attached_count` is an `AtomicUsize` incremented on TUI connect, decremented on
///   disconnect. The `/status` handler exposes `tui_attached: count > 0` (BC-2.01.002 PC-1).
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

    /// Absolute path to the companion Unix socket file (same stem as lock file, `.sock`).
    ///
    /// Written once during daemon startup alongside `lock_file_path`. Used by the
    /// `POST /shutdown` fallback path to remove the socket file without recomputing the
    /// path from `lock_file_path` (I-004: store, don't recompute). Empty string means
    /// the socket path has not been set yet.
    pub sock_file_path: String,

    /// Most-recent invocation timestamp for each hook endpoint type.
    ///
    /// Protected by a `std::sync::RwLock` (sync, not tokio) so that multiple Tokio worker
    /// threads can read concurrently while a hook-receiver task holds the write guard
    /// transiently to update a single field.
    pub last_hook_ts: RwLock<LastHookTimestamps>,

    /// JSONL ring buffer writer for hook event records (BC-2.01.007, S-008, S-009).
    ///
    /// `None` — ring buffer has not been initialized (normal in tests and before XDG path
    ///   resolution completes during daemon startup). Hook handlers MUST log WARN and
    ///   return 200 when this is `None` (AC-005: ring write is best-effort).
    /// `Some(ring)` — ring buffer is live; handlers call `ring.append(record)`. On append
    ///   failure the same best-effort policy applies (log WARN, return 200).
    ///
    /// Wrapped in `Arc` so the `DaemonState` can be cloned into axum's `State` extractor
    /// while the ring buffer remains shared (no double-buffering).
    ///
    /// Wrapped in `std::sync::Mutex` so the shutdown tail can take the Arc out (setting
    /// `None`) from the shared `Arc<DaemonState>` as a belt-and-suspenders backstop
    /// (CRITICAL-1 fix — SS-daemon-wiring-impl.md §Fix Addendum Round 2 CRITICAL-1).
    ///
    /// # Shutdown close mechanism
    ///
    /// The deterministic close/drain mechanism is the explicit `ring_flush_shutdown` Notify
    /// (H1 fix): the shutdown tail calls `ring_flush_shutdown.notify_one()` BEFORE dropping
    /// the ring Arc, which signals the flush task to drain all pending records and exit.
    /// The subsequent `drop(ring_arc)` is a harmless belt-and-suspenders backstop — it does
    /// NOT by itself close `write_tx` while any hook handler still holds a cloned Arc (hook
    /// handlers lock the Mutex, clone the Arc out, and drop the guard BEFORE calling append,
    /// so they may hold a live Arc clone past the DaemonState's Arc drop).
    /// The `begin_shutdown` AtomicBool (on `RingBuffer`) converts any post-shutdown append
    /// attempt (abnormal force-close window) into a loud, logged rejection rather than a
    /// silent enqueue-then-loss.
    ///
    /// Hook handlers MUST lock, clone the `Arc` out, drop the guard, THEN call
    /// `ring.append()` so no `std::sync::MutexGuard` is held across an `.await` point
    /// (clippy::await_holding_lock would fire otherwise; CRITICAL-1 constraint).
    pub ring: Mutex<Option<Arc<RingBuffer>>>,

    /// JoinHandle for the ring buffer flush task (BC-2.04.012 PC-4).
    ///
    /// Stored so the shutdown tail can await it after closing the write channel.
    /// `None` until `daemon_start_sequence` spawns the task at step 4b.
    /// Taken (set to `None`) during the shutdown drain sequence so no double-await occurs.
    pub ring_flush_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,

    /// Shutdown notify for the ring flush task (H1 fix — SS-daemon-wiring-impl.md §Round 3 H1).
    ///
    /// The shutdown tail calls `ring_flush_shutdown.notify_one()` BEFORE dropping the ring Arc
    /// and BEFORE awaiting the flush JoinHandle. This signals the flush task to drain all
    /// pending records via `try_recv()` and exit deterministically, independent of
    /// `Arc<RingBuffer>` refcount.
    ///
    /// `None` in the unit-test constructor (`DaemonState::new()`) — no flush task is spawned
    /// there. `Some(notify)` after `daemon_start_sequence` wires the flush task at step 4b.
    ///
    /// The shutdown tail checks `if let Some(notify) = state.ring_flush_shutdown.as_ref()`
    /// so a `None` value (test path) is a no-op.
    pub ring_flush_shutdown: Option<RingShutdownNotify>,

    /// Count of TUI clients currently attached to this daemon session (F-ADV2-HIGH-003).
    ///
    /// Incremented by 1 (`fetch_add`) when a TUI client connects and registers its
    /// subscriber channel; decremented by 1 (`fetch_sub`) when the same client disconnects
    /// and its subscriber is removed. The `/status` handler reads this as a `bool` via
    /// `tui_attached_count.load(...) > 0` (BC-2.01.002 PC-1: `tui_attached` field).
    ///
    /// Using `AtomicUsize` instead of `AtomicBool` prevents flicker when multiple TUI
    /// clients connect concurrently — a plain bool would oscillate between `false` and
    /// `true` as each connect increments and the corresponding disconnect later decrements.
    ///
    /// `Ordering::SeqCst` on both write and read: per-client tasks run on different Tokio
    /// worker threads; `SeqCst` ensures the increment and decrement are globally ordered
    /// relative to any `/status` read that follows on another thread.
    pub tui_attached_count: AtomicUsize,

    /// Force-exit signal set by a second authenticated `POST /shutdown` during drain (EC-050).
    ///
    /// `false` on construction. Set to `true` by the shutdown handler when a second
    /// `POST /shutdown` arrives while `AppMode` is already `ShuttingDown`. The main
    /// run-loop reads this flag after the drain completes and calls
    /// `exit_with(DaemonExit::AdminForceStop)` (exit code 2) instead of
    /// `exit_with(DaemonExit::Graceful)` (exit code 0) when it is `true`.
    ///
    /// `Ordering::SeqCst` is used on both write and read: the write happens in the
    /// HTTP handler task; the read happens in the main loop task. SeqCst provides the
    /// strongest cross-thread ordering guarantee, ruling out any reordering that could
    /// cause the main loop to read `false` after the handler has written `true`.
    pub force_exit: AtomicBool,

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

    // -------------------------------------------------------------------------
    // S-018 fields: bounded event bus, drop counter, session registry
    // -------------------------------------------------------------------------
    /// Bounded event bus sender (BC-2.04.011 PC-1, S-018).
    ///
    /// `None` — event bus not yet initialized (daemon startup or test stub without event bus).
    ///   Hook handlers log WARN and discard the event when this is `None`.
    /// `Some(tx)` — live `mpsc::channel(4096)` sender.
    ///   Hook handlers MUST call `tx.try_send()` (non-blocking); blocking `.send().await`
    ///   is forbidden (BC-2.04.011 invariant 2).
    ///
    /// Wrapped in `Arc` to allow cheap clones across axum handler tasks.
    pub event_bus_tx: Option<Arc<crate::types::EventBusTx>>,

    /// Drop counter for the event bus (BC-2.04.011 PC-2, PC-3a, S-018).
    ///
    /// `None` — counter not yet initialized (startup or test stub).
    /// `Some(counter)` — incremented by 1 on each `TrySendError::Full` (Ordering::Relaxed).
    ///   NEVER reset during a daemon run (BC-2.04.011 PC-2, invariant 3).
    ///
    /// Ordering::Relaxed is sufficient: this counter is for monitoring only.
    pub drop_counter: Option<Arc<AtomicU64>>,

    /// Session registry for tracking active and stopped Claude Code sessions (S-018).
    ///
    /// `None` — registry not yet initialized (startup or test stub without session tracking).
    /// `Some(registry)` — live registry. Hook handlers look up or create entries.
    ///
    /// BC-2.04.007 PC-2, BC-2.04.008 PC-2, BC-2.04.009 PC-2: each handler looks up or
    /// creates a `SessionEntry` keyed by `HookEnvelope.session_id`. The Stop handler
    /// additionally marks the session as `Stopped`.
    pub session_registry: Option<Arc<crate::hooks::SessionRegistry>>,

    // -------------------------------------------------------------------------
    // S-022 fields: pending-decision registry + IPC subscriber list
    // -------------------------------------------------------------------------
    /// Pending-decision registry for in-flight permission prompts (S-022, BC-2.05.005).
    ///
    /// `None` — registry not yet initialized (daemon startup or test stub without IPC).
    /// `Some(registry)` — live registry. The `PreToolUse` Defer path calls
    ///   `registry.register_prompt(payload, sender)` to create the entry and obtain a
    ///   stable `prompt_id`. The per-client IPC task calls `registry.resolve_prompt(...)`
    ///   when a `ClientToServer::PermissionDecision` arrives. The timeout path calls
    ///   `registry.remove_timed_out_prompt(prompt_id)` before broadcasting
    ///   `PermissionPromptResolved`.
    ///
    /// Wrapped in `Arc` to allow the registry to be shared across axum handler tasks
    /// and the per-client IPC task without cloning the registry struct.
    pub pending_decisions: Option<crate::permissions::SharedPendingDecisionRegistry>,

    /// Fan-out subscriber list for broadcasting IPC messages to connected TUI clients (S-022).
    ///
    /// `None` — IPC subscriber list not yet initialized (daemon startup or test stub).
    /// `Some(list)` — live list. The `PreToolUse` Defer path broadcasts
    ///   `ServerToClient::PermissionPromptQueued` through this list.
    ///   The `handle_permission_decision` path broadcasts `PermissionPromptResolved`
    ///   through the same list.
    ///
    /// Arc-wrapped to share the same list across axum handler tasks and the accept loop.
    pub ipc_subscribers: Option<monocle_ipc::server::SubscriberList>,

    /// UDS transport lifecycle manager (S-022, BC-2.05.001).
    ///
    /// `None` — UDS socket not yet bound (daemon startup or test stub without IPC).
    /// `Some(transport)` — bound and active. `transport.cleanup()` removes the socket file
    ///   on graceful shutdown (BC-2.05.001 PC-4).
    ///
    /// This field ensures the UDS path-length check (BC-2.05.001 EC-002) is enforced
    /// before any socket is bound: `UdsTransport::bind` validates the path and returns
    /// `IpcError::PathTooLong` if the limit is exceeded, which `daemon_start_sequence`
    /// maps to `DaemonStartError::UdsPathTooLong`.
    pub uds_transport: Option<monocle_ipc::uds::UdsTransport>,

    // -------------------------------------------------------------------------
    // Test-only fields: engine decision injection
    //
    // These fields are Option<_> (zero cost when None) and are NEVER set by
    // production code. They exist solely so integration tests can inject specific
    // engine decisions and artificial delays without coupling tests to a mock
    // engine type or a cargo feature flag.
    //
    // Production code: these fields remain None for the entire daemon lifetime.
    //
    // hook_decision_override: when Some((decision, diagnostic)), hook handlers
    //   short-circuit the real ClaudeCodeModule call and return this decision.
    //
    // hook_delay_ms: when Some(ms), hook handlers sleep for ms milliseconds
    //   inside the 300ms timeout budget, reliably triggering the timeout path.
    // -------------------------------------------------------------------------
    /// Engine decision override for integration tests.
    ///
    /// `None` — use the real `ClaudeCodeModule::on_hook()` (production default).
    /// `Some((decision, diagnostic))` — return this decision without calling `on_hook()`.
    ///
    /// NEVER set by production code. Only tests assign a `Some(_)` value.
    pub hook_decision_override: Option<(monocle_core::engine::HookDecision, Option<String>)>,

    /// Artificial delay override for integration tests (inner handler, inside 300ms timeout).
    ///
    /// `None` — no delay (production default).
    /// `Some(ms)` — sleep `ms` milliseconds inside the inner handler body, before the decision
    ///   is returned. Use a value > 300 to reliably trigger the 300ms outer timeout path.
    ///
    /// NEVER set by production code. Only unit tests assign a `Some(_)` value.
    pub hook_delay_ms: Option<u64>,

    /// Artificial delay override for E2E tests (outer handler, before 300ms timeout budget).
    ///
    /// `None` — no delay (production default).
    /// `Some(ms)` — sleep `ms` milliseconds at the outer handler level, BEFORE the 300ms inner
    ///   timeout budget starts. Used by the E2E drain-timeout test (AC-E2E-007) to create a
    ///   genuinely long in-flight HTTP request that holds axum's graceful-shutdown drain open.
    ///   Set via the `MONOCLE_HOOK_DELAY_MS` env var (read at daemon startup in
    ///   `daemon_start_sequence`). Never set by production deployments.
    pub hook_outer_delay_ms: Option<u64>,
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
            sock_file_path: String::new(),
            last_hook_ts: RwLock::new(LastHookTimestamps::default()),
            ring: Mutex::new(None),
            ring_flush_handle: Mutex::new(None),
            ring_flush_shutdown: None,
            tui_attached_count: AtomicUsize::new(0),
            force_exit: AtomicBool::new(false),
            daemon_lock: Mutex::new(None),
            shutdown_tx,
            shutdown_rx,
            event_bus_tx: None,
            drop_counter: None,
            session_registry: None,
            pending_decisions: None,
            ipc_subscribers: None,
            uds_transport: None,
            hook_decision_override: None,
            hook_delay_ms: None,
            hook_outer_delay_ms: None,
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

// ---------------------------------------------------------------------------
// S-022: InitialState snapshot (BC-2.05.002 PC-2, AC-002)
// ---------------------------------------------------------------------------

/// Produce an `InitialState` snapshot from the current `DaemonState`.
///
/// # Contract (BC-2.05.002 §Postconditions PC-2, AC-002)
///
/// The snapshot captures, at the moment of the call:
/// - `sessions`: clone of the current session roster from `state.session_registry`.
/// - `ring_tail`: last N events from the RAM ring as `Vec<HookEventRecord>` — the native
///   ring storage type (architect decision F-S022-ADV2-HIGH-002, ADR-0006).
///   No type conversion or field fabrication is performed.
/// - `overlay_stack`: clone of all currently-pending permission prompt payloads from
///   `state.pending_decisions`.
/// - `drop_counter`: current value of `state.drop_counter`.
///
/// When any of the optional fields is `None` (registry / ring / counter not yet
/// initialized), the corresponding `InitialState` field is an empty Vec / 0.
///
/// # ring_tail type rationale
///
/// `InitialState.ring_tail` is typed `Vec<HookEventRecord>` per BC-2.05.002 PC-2.
/// The previous `Vec<HookEvent>` typing required converting `HookEventRecord` → `HookEvent`,
/// which involved fabricating absent fields (cwd, transcript_path, prompt, stop_reason)
/// with empty-string defaults — silently incorrect data. The correct fix is to match the
/// IPC type to the ring's native storage type (F-S022-ADV2-HIGH-002).
///
/// # 256 KiB guard
///
/// The 256 KiB size check is performed by the caller after serialization. This function
/// only constructs the struct; it does not serialize.
///
/// # Returns
///
/// A `monocle_ipc::types::ServerToClient::InitialState { sessions, ring_tail, overlay_stack, drop_counter }`
/// variant, ready for framing.
pub fn snapshot_initial_state(state: &DaemonState) -> monocle_ipc::types::ServerToClient {
    use std::sync::atomic::Ordering;

    // Sessions: snapshot all current sessions from session_registry.
    // Maps SessionEntry → EnrichedSession via SessionRegistry::snapshot_enriched_sessions.
    let sessions: Vec<monocle_core::engine::EnrichedSession> = state
        .session_registry
        .as_ref()
        .map(|reg| reg.snapshot_enriched_sessions())
        .unwrap_or_default();

    // ring_tail: last RING_TAIL_N events from the RAM ring as Vec<HookEventRecord>.
    // Pass-through — no type conversion, no field fabrication (architect decision
    // F-S022-ADV2-HIGH-002, BC-2.05.002 PC-2, ADR-0006).
    //
    // Lock, clone Arc out, drop guard — do NOT hold the guard across the latest_events call.
    // This follows the CRITICAL-1 constraint: no Mutex guard held across any await point.
    // snapshot_initial_state is a sync function so there is no await, but the pattern is
    // consistent with the hook handlers and prevents accidental future regression.
    const RING_TAIL_N: usize = 50;
    let ring_tail: Vec<monocle_ipc::types::HookEventRecord> = {
        let ring_arc = state.ring.lock().unwrap_or_else(|e| e.into_inner()).clone();
        ring_arc
            .as_ref()
            .map(|ring| ring.latest_events(RING_TAIL_N))
            .unwrap_or_default()
    };

    // overlay_stack: clone all pending prompt payloads from pending_decisions registry.
    let overlay_stack: Vec<monocle_ipc::types::PermissionPromptPayload> = state
        .pending_decisions
        .as_ref()
        .map(|reg| reg.snapshot_payloads())
        .unwrap_or_default();

    // drop_counter: read the atomic counter value (Relaxed ordering — monitoring only).
    let drop_counter: u64 = state
        .drop_counter
        .as_ref()
        .map(|c| c.load(Ordering::Relaxed))
        .unwrap_or(0);

    monocle_ipc::types::ServerToClient::InitialState {
        sessions,
        ring_tail,
        overlay_stack,
        drop_counter,
    }
}
