//! Lock file reader — extracts `authToken`, `port`, and `app` fields from the
//! monocle daemon lock file at `<runtime_dir>/monocle.lock`.
//!
//! Source authority:
//! - SS-daemon-lifecycle.md v1.0.33 §Start Sequence (JSON template lines 491-512)
//! - BC-HOOK-013 (lock file scan algorithm)
//! - BC-HOOK-015 (token extraction from lock file)
//! - BC-HOOK-016 (auth header name — X-Claude-Code-Ide-Authorization)
//! - BC-HOOK-024 (app field filter — monocle-only)
//!
//! Lock file schema (contract_version == 1):
//! ```json
//! {
//!   "contract_version": 1,
//!   "pid": <N>,
//!   "port": <N>,
//!   "authToken": "<64-char hex>",
//!   "startTimeUtc": "<YYYY-MM-DDTHH:MM:SS.sssZ>",
//!   "app": "monocle",
//!   "version": "<semver>"
//! }
//! ```

use std::path::PathBuf;

use serde::Deserialize;
use thiserror::Error;

/// Errors from lock file discovery and parsing.
///
/// BC-HOOK-001 (fail-open when no server), BC-HOOK-013 (lock file scan),
/// BC-HOOK-015 (token extraction)
#[derive(Debug, Error)]
pub enum LockReadError {
    /// No alive monocle lock file was found.
    /// Callers MUST treat this as fail-open (BC-HOOK-001).
    #[error("no alive monocle lock file found at {search_path}")]
    NoAliveLock {
        /// Path that was searched.
        search_path: PathBuf,
    },

    /// The lock file exists but has an unrecognized contract_version.
    /// Per SS-daemon-lifecycle.md §Start Sequence: version != 1 triggers graceful skip.
    #[error("lock file contract_version mismatch: expected 1, got {found}")]
    ContractVersionMismatch {
        /// The contract_version value found in the lock file.
        found: u64,
    },

    /// The lock file JSON is syntactically invalid or missing required fields.
    #[error("lock file parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Filesystem I/O error during lock file read.
    #[error("lock file I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The lock file's PID is not alive (stale lock file).
    /// Callers MUST treat this as fail-open (BC-HOOK-001, BC-HOOK-003).
    #[error("lock file PID {pid} is not alive (stale lock)")]
    StaleLock {
        /// The PID found in the lock file.
        pid: u32,
    },

    /// The lock file's `app` field identifies a non-monocle application.
    /// Per BC-HOOK-024: monocle MUST filter out non-monocle lock files.
    #[error("lock file app field is non-monocle: {app}")]
    NonMonocleLock {
        /// The `app` field value found in the lock file.
        app: String,
    },
}

/// The connection details extracted from an alive monocle lock file.
///
/// BC-HOOK-013 (lock file scan result), BC-HOOK-015 (token extraction),
/// BC-HOOK-016 (auth header value = raw authToken, no prefix)
#[derive(Debug, Clone)]
pub struct LockFileInfo {
    /// TCP port the monocle daemon is listening on.
    pub port: u16,
    /// Raw 64-hex auth token from the lock file `authToken` field.
    /// Sent verbatim in `X-Claude-Code-Ide-Authorization` per BC-HOOK-016.
    /// No `monocle-v1:` prefix — that is the canonical header form; the alias header
    /// form used by real Claude Code hook scripts carries the raw token only.
    pub auth_token: String,
    /// PID of the daemon process.
    pub pid: u32,
}

/// Intermediate serde struct for deserializing the lock file JSON.
///
/// Kept private; callers only see `LockFileInfo`.
/// BC-HOOK-040 (struct-based serde derivation, not HashMap)
#[derive(Debug, Deserialize)]
struct LockFileJson {
    contract_version: u64,
    pid: u32,
    port: u16,
    #[serde(rename = "authToken")]
    auth_token: String,
    /// Optional per BC-HOOK-024: absent app field is treated as monocle (backward compat).
    app: Option<String>,
}

/// Read and validate the monocle daemon lock file.
///
/// Implements the lock-file discovery and validation logic per:
/// - SS-daemon-lifecycle.md v1.0.33 §Start Sequence lines 491-512 (schema)
/// - BC-HOOK-013 (scan algorithm)
/// - BC-HOOK-015 (token extraction)
/// - BC-HOOK-024 (app filter — `if (lk.app && lk.app !== 'monocle') continue`)
///
/// Returns `LockFileInfo` on success, or `LockReadError` with the specific failure mode.
/// Callers that receive `NoAliveLock` or `StaleLock` MUST fail-open per BC-HOOK-001.
pub fn read_lock_file(lock_path: &std::path::Path) -> Result<LockFileInfo, LockReadError> {
    // Step 1: Read the file — surfaces Io and NoAliveLock errors.
    let content = std::fs::read_to_string(lock_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            LockReadError::NoAliveLock {
                search_path: lock_path.to_path_buf(),
            }
        } else {
            LockReadError::Io(e)
        }
    })?;

    // Step 2: Parse JSON — surfaces ParseError for malformed / empty content.
    let lk: LockFileJson = serde_json::from_str(&content)?;

    // Step 3: contract_version check — BC-HOOK-013.
    if lk.contract_version != 1 {
        return Err(LockReadError::ContractVersionMismatch {
            found: lk.contract_version,
        });
    }

    // Step 4: app field filter per BC-HOOK-024.
    // Rule: `if (lk.app && lk.app !== 'monocle') continue`
    // Absent app field (None) passes (backwards-compat). Non-monocle string rejects.
    if let Some(ref app) = lk.app {
        if app != "monocle" {
            return Err(LockReadError::NonMonocleLock { app: app.clone() });
        }
    }

    // Step 5: port validation — port 0 is invalid.
    // BC-HOOK-013 requires a valid listening port.
    if lk.port == 0 {
        return Err(LockReadError::NoAliveLock {
            search_path: lock_path.to_path_buf(),
        });
    }

    // Step 6: PID liveness check (signal 0) per BC-HOOK-013 / BC-HOOK-017.
    // kill(pid, 0) on Unix: Ok => process alive; Err(ESRCH) => dead.
    let pid = lk.pid;
    if !is_pid_alive(pid) {
        return Err(LockReadError::StaleLock { pid });
    }

    Ok(LockFileInfo {
        port: lk.port,
        auth_token: lk.auth_token,
        pid: lk.pid,
    })
}

/// Check whether a PID is currently alive via a safe proc-filesystem or platform check.
///
/// BC-HOOK-017: PID liveness check must not produce false negatives.
/// Returns `true` if the process exists, `false` if it is definitely dead.
fn is_pid_alive(pid: u32) -> bool {
    // u32::MAX cannot be a valid PID on any real OS; always dead.
    if pid == u32::MAX {
        return false;
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, /proc/<pid>/status exists iff the process is alive.
        // This is a read-only proc-filesystem probe — safe, no signal needed.
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, use `proc_pidpath` via sysctl-based /proc equivalent.
        // The portable safe alternative: attempt to open /proc/<pid>/status
        // (not available on macOS), so instead spawn `kill -0 <pid>` via Command.
        // BC-HOOK-017 mandates liveness check; Command::new is safe (no unsafe block).
        let output = std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output();
        match output {
            Ok(o) => {
                o.status.success() || {
                    // Exit code 1 with "Operation not permitted" means process exists
                    // but we lack permission — still alive (EPERM case).
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    stderr.contains("Operation not permitted")
                }
            }
            Err(_) => {
                // Could not run kill — conservatively treat as alive.
                true
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        // On other platforms (Windows, FreeBSD, etc.) we cannot cheaply check liveness
        // without unsafe. Conservatively treat as alive to avoid false stale-lock
        // rejections. BC-HOOK-001 (fail-open) takes precedence over stale-lock detection.
        let _ = pid;
        true
    }
}

/// Derive the base URL for hook POSTs from a lock file info.
///
/// Default: `http://127.0.0.1:<port>` per dtu-assessment.md §Environment Variable Overrides.
/// Override: `MONOCLE_HOOK_ENDPOINT_BASE` env var if set (AC-006).
pub fn derive_endpoint_base(info: &LockFileInfo) -> String {
    // BC-HOOK-014: MONOCLE_HOOK_ENDPOINT_BASE takes precedence over port-derived default.
    if let Ok(base) = std::env::var("MONOCLE_HOOK_ENDPOINT_BASE") {
        if !base.is_empty() {
            return base;
        }
    }
    // BC-HOOK-005: default target is 127.0.0.1 (loopback) with the lock file port.
    format!("http://127.0.0.1:{}", info.port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_read_error_variants_are_accessible() {
        // Compile-time verification that all error variants are visible.
        // Real behavioral tests in test_bc_hook_*.rs per Step 3.
        let _e = LockReadError::NoAliveLock {
            search_path: PathBuf::from("/tmp/test.lock"),
        };
        let _e2 = LockReadError::ContractVersionMismatch { found: 2 };
        let _e3 = LockReadError::StaleLock { pid: 999 };
        let _e4 = LockReadError::NonMonocleLock {
            app: "vscode".to_string(),
        };
    }
}
