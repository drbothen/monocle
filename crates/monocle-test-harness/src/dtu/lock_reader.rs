//! Lock file reader — extracts `authToken`, `port`, and `app` fields from the
// Stub module: all function bodies are `todo!()` per TDD Red Gate.
// The `todo` lint is suppressed at file scope; this allow is intentional and
// will be removed as each function is implemented in the TDD implementation step.
#![allow(clippy::todo)]
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
pub fn read_lock_file(_lock_path: &std::path::Path) -> Result<LockFileInfo, LockReadError> {
    todo!("S-DTU-001 implementation pending; BC-HOOK-013, BC-HOOK-015, BC-HOOK-024")
}

/// Derive the base URL for hook POSTs from a lock file info.
///
/// Default: `http://127.0.0.1:<port>` per dtu-assessment.md §Environment Variable Overrides.
/// Override: `MONOCLE_HOOK_ENDPOINT_BASE` env var if set (AC-006).
pub fn derive_endpoint_base(_info: &LockFileInfo) -> String {
    todo!("S-DTU-001 implementation pending; AC-006, BC-HOOK-013")
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
