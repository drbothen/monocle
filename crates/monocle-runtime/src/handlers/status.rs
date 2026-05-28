//! `GET /status` — authenticated daemon state snapshot handler.
//!
//! Returns a JSON object with 10 fields reflecting the current runtime state of the
//! monocle daemon. Requires a valid auth header per BC-2.01.009 / ADR-0005 (enforced
//! by the auth middleware layer on the authenticated router, not by this handler).
//!
//! # Behavioral contract
//!
//! BC-2.01.002 (Status Endpoint — Authenticated Daemon State). See story S-003.
//! BC-2.02.001 (ABI Version in /status). S-010 provides `MONOCLE_ABI_VERSION`.
//!
//! # Architecture constraints
//!
//! - This handler MUST NOT import from `monocle-tui` (not a Phase 1 crate).
//! - This handler MUST NOT perform token comparison with `==`; auth is enforced
//!   upstream by the auth middleware in `auth.rs`.
//! - Registered on the AUTHENTICATED router only (`server::build_server`).
//! - `/status` serves during graceful shutdown drain window (AC-008, BC-2.01.002 PC-3):
//!   it does NOT return 503 on `AppMode::ShuttingDown`. It is read-only and exempt from
//!   the drain-503 rule.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use crate::state::DaemonState;

/// Per-hook-type ISO 8601 timestamps for the most-recent invocation of each hook endpoint.
///
/// Fields map 1-to-1 to the 5 canonical hook endpoints from BC-2.01.002 PC-1 sub-bullet
/// `last_hook_ts` and BC-2.01.008 PC-4:
/// - `/hooks/pre-tool-use`
/// - `/hooks/notification`
/// - `/hooks/stop`
/// - `/hooks/session-start`
/// - `/hooks/prompt-submit`
///
/// A `None` value serializes as JSON `null` (not the string `"null"` and not `0`), per
/// BC-2.01.002 PC-1 sub-bullet `last_hook_ts` and EC-044.
///
/// Non-null values use ISO 8601 UTC with mandatory millisecond precision:
/// `YYYY-MM-DDTHH:MM:SS.sssZ` (chrono format string `"%Y-%m-%dT%H:%M:%S%.3fZ"`),
/// per SS-daemon-lifecycle.md v1.0.33 §Status Endpoint.
#[derive(Debug, Serialize)]
pub struct LastHookTs {
    /// Timestamp of most recent `POST /hooks/pre-tool-use`.
    pub pre_tool_use: Option<String>,
    /// Timestamp of most recent `POST /hooks/notification`.
    pub notification: Option<String>,
    /// Timestamp of most recent `POST /hooks/stop`.
    pub stop: Option<String>,
    /// Timestamp of most recent `POST /hooks/session-start`.
    pub session_start: Option<String>,
    /// Timestamp of most recent `POST /hooks/prompt-submit`.
    pub prompt_submit: Option<String>,
}

/// JSON response body for `GET /status` (BC-2.01.002 Postcondition 1).
///
/// All 10 fields are required. Serialized as a flat JSON object.
///
/// Field specifications (VP-002 probe matrix, BC-2.01.002 PC-1):
///
/// | Field | Type | Notes |
/// |-------|------|-------|
/// | `pid` | `u32` | daemon process PID |
/// | `uptime_sec` | `u64` | seconds since daemon start |
/// | `version` | `String` | semver string from `CARGO_PKG_VERSION` |
/// | `abi_version` | `u32` | equals `monocle_core::MONOCLE_ABI_VERSION` (value 1) |
/// | `lock_file` | `String` | absolute path to daemon lock file |
/// | `hook_endpoints` | `Vec<String>` | exactly 5 canonical paths (BC-2.01.008 PC-4) |
/// | `ring_buffer_fill_pct` | `f64` | 0.0–100.0 |
/// | `channel_saturation_pct` | `f64` | 0.0–100.0 |
/// | `last_hook_ts` | `LastHookTs` | per-type nullable ISO 8601 timestamps |
/// | `tui_attached` | `bool` | TUI attachment state |
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    /// Daemon process ID.
    pub pid: u32,
    /// Seconds elapsed since daemon start.
    pub uptime_sec: u64,
    /// Crate version from `CARGO_PKG_VERSION`.
    pub version: String,
    /// ABI version from `monocle_core::MONOCLE_ABI_VERSION` (value `1` in Phase 1).
    pub abi_version: u32,
    /// Absolute path to the daemon's lock file.
    pub lock_file: String,
    /// Exactly 5 canonical hook endpoint paths (BC-2.01.002 PC-1 + BC-2.01.008 PC-4).
    pub hook_endpoints: Vec<String>,
    /// Ring buffer fill percentage (0.0–100.0).
    pub ring_buffer_fill_pct: f64,
    /// Channel saturation percentage (0.0–100.0).
    pub channel_saturation_pct: f64,
    /// Per-hook-type most-recent invocation timestamps.
    pub last_hook_ts: LastHookTs,
    /// Whether a TUI client is currently attached.
    pub tui_attached: bool,
}

/// Build the `LastHookTs` response struct from the shared `LastHookTimestamps` state.
///
/// Reads under a shared read lock and clones the `Option<String>` fields.
/// On lock poisoning (a handler panicked while holding the write lock), returns
/// an all-null `LastHookTs` and logs a warning — the daemon can still serve status.
fn build_last_hook_ts(state: &DaemonState) -> LastHookTs {
    match state.last_hook_ts.read() {
        Ok(ts) => LastHookTs {
            pre_tool_use: ts.pre_tool_use.clone(),
            notification: ts.notification.clone(),
            stop: ts.stop.clone(),
            session_start: ts.session_start.clone(),
            prompt_submit: ts.prompt_submit.clone(),
        },
        Err(_poisoned) => {
            tracing::warn!(
                "RwLock<LastHookTimestamps> poisoned; returning null timestamps in /status"
            );
            LastHookTs {
                pre_tool_use: None,
                notification: None,
                stop: None,
                session_start: None,
                prompt_submit: None,
            }
        }
    }
}

/// Authenticated daemon state snapshot handler for `GET /status`.
///
/// Returns HTTP 200 with a [`StatusResponse`] JSON body when the auth middleware
/// has already validated the request (BC-2.01.002 Postcondition 1).
///
/// This handler does NOT perform auth itself — the auth middleware in `auth.rs` is
/// applied at the router layer via `build_server`.
///
/// Serves during graceful shutdown drain window without returning 503 (AC-008,
/// BC-2.01.002 PC-3): the read-only status handler is exempt from the drain-503 rule.
pub async fn get_status(State(state): State<Arc<DaemonState>>) -> impl IntoResponse {
    let last_hook_ts = build_last_hook_ts(&state);
    let response = StatusResponse {
        pid: std::process::id(),
        uptime_sec: state.start_time.elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        abi_version: monocle_core::MONOCLE_ABI_VERSION,
        lock_file: state.lock_file_path.clone(),
        hook_endpoints: vec![
            "/hooks/pre-tool-use".to_string(),
            "/hooks/notification".to_string(),
            "/hooks/stop".to_string(),
            "/hooks/session-start".to_string(),
            "/hooks/prompt-submit".to_string(),
        ],
        ring_buffer_fill_pct: 0.0,
        channel_saturation_pct: 0.0,
        last_hook_ts,
        // tui_attached is true when at least one TUI client is connected
        // (count > 0). SeqCst ordering matches the write ordering in ipc_server.rs
        // (F-ADV2-HIGH-003: count-based to prevent flicker with multiple clients).
        tui_attached: state
            .tui_attached_count
            .load(std::sync::atomic::Ordering::SeqCst)
            > 0,
    };
    (StatusCode::OK, Json(response))
}
