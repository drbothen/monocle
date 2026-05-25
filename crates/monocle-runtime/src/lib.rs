//! monocle-runtime — Runtime daemon library.
//!
//! Stub created in S-001. Populated by S-002 (daemon lifecycle), S-003 (auth),
//! S-004 (lock file), S-005 (hook ingestion), S-006 (status endpoint), S-008 (ring),
//! S-009 (hook routes), S-015 (XDG path resolution).

#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// HTTP request handlers, organized by endpoint.
pub mod handlers;
/// Axum router construction — unauthenticated and authenticated router split.
pub mod router;
/// Daemon shared state types: [`state::AppMode`] and [`state::DaemonState`].
pub mod state;
