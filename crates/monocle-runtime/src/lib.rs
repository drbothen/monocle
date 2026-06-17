//! monocle-runtime — Runtime daemon library.
//!
//! Stub created in S-001. Populated by S-002 (daemon lifecycle), S-003 (auth),
//! S-004 (lock file), S-005 (hook ingestion), S-006 (lock file atomic lifecycle),
//! S-008 (ring), S-009 (hook routes), S-015 (XDG path resolution),
//! S-017 (daemon start sequence + hooks-settings.json),
//! S-018 (hook routing + bounded event bus + drop counter).

#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// Auth middleware implementing dual-accept header validation (ADR-0005, BC-2.01.009).
/// Extended by S-006 to include [`auth::generate_session_token`].
pub mod auth;
/// Body-size enforcement middleware for the authenticated router (S-004).
///
/// Provides [`body_limit::body_size_limit_middleware`] — a `from_fn` middleware that
/// returns HTTP 413 with a JSON body when `Content-Length` exceeds 256 KiB.
pub mod body_limit;
/// Engine module implementations: [`engine::ClaudeCodeModule`] and supporting types
/// (`SpawnArgs`, `SessionHandle`, `EngineVersion`, `SpawnError`, `PreflightError`) (S-015).
pub mod engine;
/// Error types for daemon startup and lock-file lifecycle (S-006).
pub mod errors;
/// HTTP request handlers, organized by endpoint.
pub mod handlers;
/// Runtime directory resolution, daemon exit-code taxonomy, and crash recovery
/// checkpoint I/O:
/// [`lifecycle::resolve_runtime_dir`], [`lifecycle::ensure_runtime_dir`] (S-006),
/// [`lifecycle::DaemonExit`], [`lifecycle::exit_with`] (S-005),
/// [`lifecycle::write_recovery_checkpoint`], [`lifecycle::read_recovery_checkpoint`] (S-007).
/// Extended by S-017 with [`lifecycle::daemon_start_sequence`],
/// [`lifecycle::write_hooks_settings`], [`lifecycle::write_lock_file`],
/// and [`lifecycle::remove_hooks_settings`].
pub mod lifecycle;
/// Daemon lock file lifecycle: acquire, detect stale, release (S-006).
pub mod lock;
/// JSONL ring buffer writer for hook event records: [`ring::HookEventRecord`],
/// [`ring::RingBuffer`], [`ring::RotationConfig`], [`ring::RingError`], and
/// [`ring::RING_FORMAT_VERSION`] (BC-2.01.007, S-008).
pub mod ring;
/// Axum router construction — unauthenticated and authenticated router split.
pub mod router;
/// Full server construction: merges unauthenticated + authenticated routers.
pub mod server;
/// Daemon shared state types: [`state::AppMode`] and [`state::DaemonState`].
pub mod state;
/// Shared runtime types: [`types::RecoveryCheckpoint`] and [`types::ShutdownReason`] (S-007).
/// Extended by S-017 with [`types::EventBusHookEvent`], [`types::EventBusTx`],
/// [`types::EventBusRx`], [`types::EVENT_BUS_CAPACITY`], [`types::HooksSettings`],
/// [`types::HooksMap`], [`types::HookEntry`], [`types::HookCommand`],
/// and [`types::EngineModuleRegistry`].
pub mod types;

/// Bounded event bus fan-out task and drop counter utilities (S-018, BC-2.04.011).
///
/// Provides [`event_bus::event_bus_fan_out_task`] (fan-out from channel to TUI clients),
/// [`event_bus::drop_counter_debounce_task`] (100ms DropCounterUpdate debounce), and
/// [`event_bus::try_publish_event`] (canonical try_send helper used by all hook handlers).
pub mod event_bus;

/// Pending-decision registry for in-flight permission prompts (S-022, BC-2.05.005).
///
/// Provides [`permissions::PendingDecisionRegistry`] with `register_prompt`,
/// `resolve_prompt`, `remove_timed_out_prompt`, and `snapshot_payloads` methods.
/// Also exports [`permissions::SharedPendingDecisionRegistry`] (Arc-wrapped handle used
/// as the type of `DaemonState.pending_decisions`).
pub mod permissions;

/// UDS connection accept loop and per-client task spawner (S-022, BC-2.05.002).
///
/// Lives in `monocle-runtime` (not `monocle-ipc`) to avoid a circular crate dependency:
/// `monocle-runtime` depends on `monocle-ipc`, and the accept loop needs both
/// `monocle_ipc` types and `monocle_runtime::state::DaemonState`.
///
/// Provides [`ipc_server::run_accept_loop`] and [`ipc_server::send_initial_state`].
pub mod ipc_server;

/// Hook endpoint handler modules for S-018 (BC-2.04.007, BC-2.04.008, BC-2.04.009).
///
/// Sub-modules:
/// - [`hooks::pre_tool_use`]: `POST /hooks/pre-tool-use` (300ms timeout, Defer support)
/// - [`hooks::notification`]: `POST /hooks/notification` (2000ms timeout, no Defer)
/// - [`hooks::stop_session_prompt`]: `POST /hooks/stop`, `/session-start`, `/prompt-submit`
///
/// Shared types:
/// - [`hooks::HookEnvelope`]: deserialized JSON body (all 5 hook types share this)
/// - [`hooks::SessionRegistry`]: session lifecycle state tracker
/// - [`hooks::SessionState`]: Active / Stopped
pub mod hooks;

/// Session Manager — daemon-side coordinator for session-host processes (S-033).
///
/// Implements BC-2.08.001 (spawn_session), BC-2.08.008 (SessionStateChanged broadcast),
/// and BC-2.03.008 (spawn_recipe default + EngineError bridge).
///
/// Key exports:
/// - [`session_manager::SessionManager`]: main coordinator struct.
/// - [`session_manager::SessionError`]: error taxonomy.
/// - [`session_manager::IpcOp`]: operation context for error code mapping.
/// - [`session_manager::session_error_to_code`]: SessionError → wire code mapping.
/// - [`session_manager::SessionHostSpawner`]: spawner trait (test seam).
/// - [`session_manager::RealSessionHostSpawner`]: production spawner.
/// - [`session_manager::SpawnedHostHandle`]: spawn result type.
pub mod session_manager;
