//! HTTP request handlers for the monocle daemon.
//!
//! Organized by endpoint. Each sub-module contains handler functions registered
//! on either the unauthenticated or authenticated axum router.
//!
//! - [`healthz`] — `GET /healthz` unauthenticated liveness probe (BC-2.01.001, S-002)
//! - [`status`] — `GET /status` authenticated daemon state snapshot (BC-2.01.002, S-003)

pub mod healthz;
pub mod status;
