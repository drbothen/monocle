//! DTU clone server — axum HTTP server lifecycle and shared state.
// Stub module: `start_server` body is `todo!()` per TDD Red Gate.
// The `todo` lint is suppressed at file scope; this allow is intentional and
// will be removed when the server is implemented in the TDD implementation step.
#![allow(clippy::todo)]
//!
//! Provides `CloneState` (shared across handlers) and `start_server`
//! (binds the axum router to a port and begins serving).
//!
//! Source authority:
//! - AC-005 (Rust binary, dtu-claude-code-hooks-v1)
//! - AC-006 (MONOCLE_HOOK_ENDPOINT_BASE and MONOCLE_NO_AUTOSTART env vars)
//! - dtu-assessment.md v1.7.5 §Packaging Decision §Environment Variable Overrides
//! - BC-HOOK-013 (lock file discovery), BC-HOOK-015 (token extraction)

use std::net::SocketAddr;

use anyhow::Result;

use crate::dtu::lock_reader::LockFileInfo;

/// Shared state passed to all 5 axum hook endpoint handlers.
///
/// Contains the resolved daemon connection info (read from the monocle lock file
/// per BC-HOOK-015) and the reqwest HTTP client for proxying POSTs.
///
/// BC-HOOK-013 (lock file scan), BC-HOOK-015 (token extraction),
/// BC-HOOK-016 (auth header value)
#[derive(Debug, Clone)]
pub struct CloneState {
    /// HTTP client used to POST synthesized payloads to the monocle daemon.
    /// Initialized once and shared across all handler invocations.
    pub client: reqwest::Client,
    /// Resolved monocle daemon connection details from the lock file.
    pub daemon: LockFileInfo,
    /// Base URL for daemon hook POSTs, e.g. `http://127.0.0.1:7860`.
    /// Derived from `daemon.port` unless `MONOCLE_HOOK_ENDPOINT_BASE` overrides.
    /// AC-006 (env var override)
    pub endpoint_base: String,
}

/// Start the DTU clone HTTP server on the given address.
///
/// The server binds to `addr`, registers the 5 hook endpoint routes, and starts
/// serving. This function does not return until the server shuts down.
///
/// AC-005 (binary artifact), AC-006 (MONOCLE_NO_AUTOSTART)
pub async fn start_server(_addr: SocketAddr, _state: CloneState) -> Result<()> {
    todo!("S-DTU-001 implementation pending; AC-005, AC-006")
}

/// Write the hooks-settings.json file atomically to `runtime_dir` using `tempfile::persist`.
///
/// Returns the path to the written file.
///
/// BC-HOOK-009 (path + mode), BC-HOOK-010 (single file per runtimeDir),
/// BC-HOOK-011 (no cleanup), BC-HOOK-039 (atomic write), BC-HOOK-040 (struct not HashMap),
/// BC-HOOK-041 (filename assertion)
pub fn write_hooks_settings_file(_runtime_dir: &std::path::Path) -> Result<std::path::PathBuf> {
    todo!("S-DTU-001 implementation pending; BC-HOOK-009, BC-HOOK-039, BC-HOOK-040, BC-HOOK-041")
}
