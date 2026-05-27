//! CCR (Claude Code Router) detection logic.
//!
//! Per BC-2.07.006: two-step detection — explicit config path first,
//! then `which::which("ccr")` PATH fallback. No caching (INV-3: fresh
//! filesystem probe on every call). Returns `Option<PathBuf>`, never `Result`.

#![forbid(unsafe_code)]

use crate::config::MonocleConfig;
use std::path::PathBuf;

/// Detect the CCR binary path using a two-step algorithm.
///
/// **Step 1 — Explicit config path check (BC-2.07.006 PC-1):**
/// If `config.ccr_path` is `Some(path_str)`:
/// - Constructs `PathBuf::from(path_str)` and calls `path.is_file()`.
/// - If `true`: returns `Some(path)`. PATH search is NOT performed.
/// - If `false`: emits `tracing::warn!` and falls through to Step 2.
///   (configured path not existing is non-fatal — BC-2.07.006 PC-1d.)
///
/// **Step 2 — PATH search fallback (BC-2.07.006 PC-2):**
/// If `config.ccr_path` is `None` OR configured path did not resolve:
/// - Calls `which::which("ccr")`.
/// - Returns `Some(path)` on `Ok(path)`, `None` on `Err(_)` (not found on PATH).
///
/// **No caching:** Every call performs fresh filesystem probes (BC-2.07.006 INV-3).
/// This ensures a CCR binary installed while monocle is running is detected on the
/// next status bar refresh cycle.
///
/// **No panic, no `Err`:** All fallible operations return `bool` or `Result`;
/// neither causes a panic. All failure modes produce `None` (BC-2.07.006 INV-1, INV-2).
pub fn detect_ccr(config: &MonocleConfig) -> Option<PathBuf> {
    todo!("detect_ccr(): not yet implemented")
}
