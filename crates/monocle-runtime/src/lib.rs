//! monocle-runtime — Runtime daemon library.
//!
//! Stub created in S-001. Populated by S-002 (daemon lifecycle), S-003 (auth),
//! S-004 (lock file), S-005 (hook ingestion), S-006 (lock file atomic lifecycle),
//! S-008 (ring), S-009 (hook routes), S-015 (XDG path resolution).

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
/// Error types for daemon startup and lock-file lifecycle (S-006).
pub mod errors;
/// HTTP request handlers, organized by endpoint.
pub mod handlers;
/// Runtime directory resolution, daemon exit-code taxonomy, and crash recovery
/// checkpoint I/O:
/// [`lifecycle::resolve_runtime_dir`], [`lifecycle::ensure_runtime_dir`] (S-006),
/// [`lifecycle::DaemonExit`], [`lifecycle::exit_with`] (S-005),
/// [`lifecycle::write_recovery_checkpoint`], [`lifecycle::read_recovery_checkpoint`] (S-007).
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
pub mod types;
