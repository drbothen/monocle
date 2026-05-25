//! Daemon lock file lifecycle: acquire, detect stale, release.
//!
//! Populated by S-006 (Lock File Atomic Lifecycle).
//!
//! # Contract Summary
//!
//! - [`DaemonLock::acquire`] atomically writes a lock file into the runtime directory.
//!   It checks for an existing lock file, validates PID liveness, removes stale locks,
//!   generates an auth token, and persists the lock via `tempfile::persist`.
//! - [`DaemonLock::release`] removes both the lock file and the Unix socket path.
//! - Lock file content is canonically ordered JSON per [`LockFileContent`].
//!
//! # Security
//!
//! - Auth token generated via `crate::auth::generate_session_token()` — 32 bytes of
//!   `OsRng` output hex-encoded to 64 chars (BC-2.03.001, NFR-010).
//! - Lock file written via `tempfile::persist` — atomic, no partial-write window.
//! - Runtime directory MUST be `0o700`; lock file inherits directory ACL protection.
//!
//! # Anti-patterns enforced (SS-conventions-anti-patterns.md)
//!
//! - NEVER `std::fs::write` — use `tempfile::persist`.
//! - NEVER `std::fs::create_dir_all` — use `DirBuilder::new().mode(0o700)`.
//! - NEVER `println!` — use `tracing`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::DaemonStartError;

/// Canonical JSON content of the monocle daemon lock file.
///
/// Field ordering in the serialized JSON is significant for human readability:
/// `contract_version` MUST appear first so that readers can detect incompatible
/// versions before parsing the remaining fields. `serde_json` with the `preserve_order`
/// feature (or use of an ordered map) is required to guarantee insertion order.
///
/// # Wire format
///
/// ```json
/// {
///   "contract_version": 1,
///   "pid": 12345,
///   "port": 9_001,
///   "authToken": "<64-hex-chars>",
///   "startTimeUtc": "2026-05-25T10:00:00.000Z",
///   "app": "monocle",
///   "version": "0.1.0"
/// }
/// ```
///
/// `authToken` and `startTimeUtc` use camelCase per BC-2.03.001 PC-1 schema.
#[derive(Debug, Serialize, Deserialize)]
pub struct LockFileContent {
    /// Schema version — always `1` for this revision. MUST be the first key.
    pub contract_version: u32,
    /// PID of the live daemon process that created this lock.
    pub pid: i32,
    /// TCP port the daemon's HTTP server is bound to.
    pub port: u16,
    /// 64-hex-char auth token for this daemon session (BC-2.01.009, BC-2.03.001).
    ///
    /// Clients must include this in the `X-Monocle-Authorization` header with the
    /// `monocle-v1:` prefix, or in `X-Claude-Code-Ide-Authorization` without prefix.
    #[serde(rename = "authToken")]
    pub auth_token: String,
    /// ISO 8601 UTC timestamp with millisecond precision recording daemon start time.
    ///
    /// Format: `YYYY-MM-DDTHH:MM:SS.sssZ` (e.g., `"2026-05-25T10:00:00.000Z"`).
    #[serde(rename = "startTimeUtc")]
    pub start_time_utc: String,
    /// Application identifier — always `"monocle"`.
    pub app: String,
    /// Semantic version of the daemon binary from `CARGO_PKG_VERSION`.
    pub version: String,
}

/// RAII guard for the daemon lock file.
///
/// Constructed by [`DaemonLock::acquire`]; released by [`DaemonLock::release`].
/// Does NOT implement `Drop`-based release — the daemon must call `release()` explicitly
/// during graceful shutdown so that the `Result` from `fs::remove_file` can be surfaced
/// to the tracing layer. Silent `Drop` panics on `io::Error` are not acceptable.
#[derive(Debug)]
// Fields are populated by the implementer (S-006). Stubs leave bodies as unimplemented!().
#[allow(dead_code)]
pub struct DaemonLock {
    /// Absolute path to the lock file on disk.
    pub(crate) path: PathBuf,
    /// Absolute path to the Unix socket file (removed on release alongside the lock file).
    pub(crate) sock_path: PathBuf,
}

impl DaemonLock {
    /// Acquire the daemon lock file atomically.
    ///
    /// # Algorithm
    ///
    /// 1. Read existing lock file at `<runtime_dir>/monocle.lock` (if present).
    /// 2. If the recorded PID is live (`nix::sys::signal::kill(pid, None)` succeeds),
    ///    return `Err(DaemonStartError::LockFileConflict { pid })`.
    /// 3. If the PID is stale (process not found), remove the existing lock file.
    /// 4. Generate a fresh auth token via [`crate::auth::generate_session_token`].
    /// 5. Serialize [`LockFileContent`] to JSON and persist via `tempfile::NamedTempFile`
    ///    + `tempfile::persist` into the runtime directory.
    /// 6. Return `(DaemonLock, auth_token_string)`.
    ///
    /// # Errors
    ///
    /// - [`DaemonStartError::LockFileConflict`] — live process holds the lock.
    /// - [`DaemonStartError::LockFileWriteFailure`] — I/O error writing/persisting the lock.
    pub fn acquire(_runtime_dir: &Path, _port: u16) -> Result<(Self, String), DaemonStartError> {
        unimplemented!("S-006: DaemonLock::acquire — read existing lock, check PID liveness, remove stale, write atomic lock via tempfile::persist")
    }

    /// Release the daemon lock by removing the lock file and socket file from disk.
    ///
    /// Both files are removed with `std::fs::remove_file`. Errors from either removal
    /// are propagated; if the lock file removal succeeds but the socket removal fails,
    /// the socket error is returned (the lock file is already gone).
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if either file removal fails. Callers should log the
    /// error via `tracing::error!` and proceed with shutdown regardless.
    pub fn release(&self) -> std::io::Result<()> {
        unimplemented!("S-006: DaemonLock::release — remove lock file and sock file")
    }
}
