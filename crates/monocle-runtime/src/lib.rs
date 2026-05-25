//! monocle-runtime — Runtime daemon library.
//!
//! Stub created in S-001. Populated by S-002 (daemon lifecycle), S-003 (auth),
//! S-004 (lock file), S-005 (hook ingestion), S-006 (status endpoint), S-008 (ring),
//! S-009 (hook routes), S-015 (XDG path resolution).

#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// Auth middleware implementing dual-accept header validation (ADR-0005, BC-2.01.009).
pub mod auth;
/// HTTP request handlers, organized by endpoint.
pub mod handlers;
/// Axum router construction — unauthenticated and authenticated router split.
pub mod router;
/// Full server construction: merges unauthenticated + authenticated routers.
pub mod server;
/// Daemon shared state types: [`state::AppMode`] and [`state::DaemonState`].
pub mod state;
