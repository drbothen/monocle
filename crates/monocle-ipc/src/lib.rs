#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! IPC message types and transport for monocle daemon-TUI communication (S-021).
//!
//! # Architecture
//!
//! This crate implements the Unix domain socket transport layer for the monocle daemon-TUI
//! IPC protocol. Key components:
//!
//! - [`transport::Transport`] — the abstract transport trait (Phase 4 extensibility point).
//! - [`uds::UdsTransport`] — the sole Phase 1 `Transport` implementor (UDS-only, no shared memory).
//! - [`framing`] — 4-byte little-endian length-prefix framing with 256 KiB limit.
//! - [`types`] — `ServerToClient`, `ClientToServer`, `PermissionPromptPayload`, `IpcError`.
//!
//! # Phase 1 Constraints (BC-2.05.008)
//!
//! `#![forbid(unsafe_code)]` is the first crate attribute. No shared-memory primitives
//! are used or permitted in this crate — UDS only. The `cargo deny` configuration and
//! semgrep CI checks enforce this at build time. See `deny.toml` for the banned crate list.
//!
//! `UdsTransport` is the sole `Transport` implementor in this crate during Phase 1. Phase 4
//! `ShmTransport` will live in a separate `monocle-ipc-shm` crate.

pub mod error;
/// Transport-level events (S-023, BC-2.05.007): `TransportEvent::Disconnected` for SOQ-3.
pub mod events;
pub mod framing;
/// Reconnection loop with exponential backoff (S-023, BC-2.05.006).
pub mod reconnect;
/// Server-side connection accept loop and per-client task spawner (S-022, BC-2.05.002).
pub mod server;
pub mod transport;
pub mod types;
pub mod uds;
