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

/// Internal result of inspecting an existing lock file.
///
/// Used by [`DaemonLock::check_existing_lock`] to communicate the disposition
/// to [`DaemonLock::acquire`].
enum ExistingLockStatus {
    /// The lock file records a live process — acquisition must fail.
    LiveConflict { pid: i32 },
    /// The lock file is stale (dead pid, unknown format, unreadable) — remove and proceed.
    Stale,
}

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
    pub fn acquire(runtime_dir: &Path, port: u16) -> Result<(Self, String), DaemonStartError> {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let lock_path = runtime_dir.join("monocle.lock");
        let sock_path = runtime_dir.join("monocle.sock");

        // --- Step 1-4: Check for existing lock file and handle stale/live scenarios ---
        if lock_path.exists() {
            match Self::check_existing_lock(&lock_path) {
                ExistingLockStatus::LiveConflict { pid } => {
                    return Err(DaemonStartError::LockFileConflict { pid });
                }
                ExistingLockStatus::Stale => {
                    // Dead pid or unrecognized format — remove stale lock and proceed.
                    if let Err(e) = std::fs::remove_file(&lock_path) {
                        tracing::warn!(
                            path = %lock_path.display(),
                            error = %e,
                            "failed to remove stale lock file; proceeding anyway"
                        );
                    }
                }
            }
        }

        // --- Step 5: Generate fresh auth token ---
        let auth_token = crate::auth::generate_session_token();

        // --- Step 6: Build LockFileContent with all 7 fields ---
        let now = chrono::Utc::now();
        let start_time_utc = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        // std::process::id() returns u32; on all supported POSIX platforms the
        // kernel PID namespace fits within i32::MAX (Linux: 4_194_304; macOS: 99_999).
        // If the conversion fails, we cannot write a valid lock file — treat as I/O error.
        let pid_i32 = i32::try_from(std::process::id()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "current process PID does not fit in i32 (unexpected on supported platforms)",
            )
        })?;

        let content = LockFileContent {
            contract_version: 1,
            pid: pid_i32,
            port,
            auth_token: auth_token.clone(),
            start_time_utc,
            app: "monocle".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // --- Steps 7-8: Serialize to JSON and persist atomically ---
        let json_bytes = serde_json::to_string_pretty(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("JSON serialize failed: {e}"),
            )
        })?;

        let mut tmp = tempfile::NamedTempFile::new_in(runtime_dir)?;
        tmp.write_all(json_bytes.as_bytes())?;

        // --- Step 9: Persist atomically (rename into place) ---
        // tempfile::persist returns PersistError (not io::Error); extract the inner io::Error.
        tmp.persist(&lock_path).map_err(|e| e.error)?;

        // --- Step 10: Set file permissions to 0o600 ---
        std::fs::set_permissions(&lock_path, std::fs::Permissions::from_mode(0o600))?;

        tracing::info!(
            path = %lock_path.display(),
            port = port,
            "daemon lock file acquired"
        );

        Ok((
            Self {
                path: lock_path,
                sock_path,
            },
            auth_token,
        ))
    }

    /// Internal helper: inspect an existing lock file and determine if it represents
    /// a live pid conflict, or a stale (dead pid / unknown format) that should be removed.
    fn check_existing_lock(lock_path: &Path) -> ExistingLockStatus {
        // Read and parse the existing lock file.
        let raw = match std::fs::read_to_string(lock_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    path = %lock_path.display(),
                    error = %e,
                    "could not read existing lock file; treating as stale"
                );
                return ExistingLockStatus::Stale;
            }
        };

        let json: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    path = %lock_path.display(),
                    error = %e,
                    "existing lock file is not valid JSON; treating as stale"
                );
                return ExistingLockStatus::Stale;
            }
        };

        // --- Validate contract_version ---
        let cv = match json.get("contract_version") {
            None => {
                // Missing contract_version — pre-Phase-1 format, treat as stale (BC-2.01.010 EC-012).
                tracing::warn!(
                    path = %lock_path.display(),
                    "existing lock file missing contract_version key (pre-Phase-1 format); treating as stale (E-LOCK-002)"
                );
                return ExistingLockStatus::Stale;
            }
            Some(v) => v,
        };

        let version_int: u64 = if let Some(n) = cv.as_u64() {
            // Canonical: integer contract_version.
            n
        } else if let Some(s) = cv.as_str() {
            // BC-2.01.010 EC-011: string contract_version — attempt coercion.
            match s.parse::<u64>() {
                Ok(n) => {
                    tracing::warn!(
                        path = %lock_path.display(),
                        version_str = s,
                        "existing lock file has contract_version as string (coercion); \
                        treating as version {n} (E-LOCK-002)"
                    );
                    n
                }
                Err(_) => {
                    tracing::warn!(
                        path = %lock_path.display(),
                        version_str = s,
                        "existing lock file has non-numeric string contract_version; treating as stale (E-LOCK-002)"
                    );
                    return ExistingLockStatus::Stale;
                }
            }
        } else {
            // Other types (null, bool, array, object) — treat as stale.
            tracing::warn!(
                path = %lock_path.display(),
                "existing lock file has contract_version with unsupported type; treating as stale (E-LOCK-002)"
            );
            return ExistingLockStatus::Stale;
        };

        // BC-2.01.010 PC-4 / EC-010: unknown (future) contract_version — treat as stale.
        if version_int != 1 {
            tracing::warn!(
                path = %lock_path.display(),
                contract_version = version_int,
                "existing lock file has unrecognized contract_version; treating as stale (E-LOCK-002)"
            );
            return ExistingLockStatus::Stale;
        }

        // --- version == 1: check PID liveness ---
        let pid = match json.get("pid").and_then(|v| v.as_i64()) {
            Some(p) => p,
            None => {
                tracing::warn!(
                    path = %lock_path.display(),
                    "existing lock file missing or invalid pid field; treating as stale"
                );
                return ExistingLockStatus::Stale;
            }
        };

        let pid_i32 = match i32::try_from(pid) {
            Ok(p) => p,
            Err(_) => {
                tracing::warn!(
                    path = %lock_path.display(),
                    pid = pid,
                    "existing lock file pid does not fit in i32; treating as stale"
                );
                return ExistingLockStatus::Stale;
            }
        };

        // Send signal 0 to check if the process is alive (nix::sys::signal::kill).
        let nix_pid = nix::unistd::Pid::from_raw(pid_i32);
        match nix::sys::signal::kill(nix_pid, None) {
            Ok(()) => {
                // Process is alive — return conflict.
                tracing::info!(
                    path = %lock_path.display(),
                    pid = pid_i32,
                    "existing lock file has live PID; returning LockFileConflict (E-LOCK-001)"
                );
                ExistingLockStatus::LiveConflict { pid: pid_i32 }
            }
            Err(nix::errno::Errno::ESRCH) => {
                // ESRCH: no such process — stale lock.
                tracing::warn!(
                    path = %lock_path.display(),
                    pid = pid_i32,
                    "existing lock file has dead PID (ESRCH); removing stale lock (E-LOCK-004)"
                );
                ExistingLockStatus::Stale
            }
            Err(e) => {
                // Other error (e.g., EPERM: process exists but we lack permission to signal it).
                // Treat as live to be safe — we cannot verify staleness.
                tracing::warn!(
                    path = %lock_path.display(),
                    pid = pid_i32,
                    error = %e,
                    "kill(pid, 0) returned unexpected error; treating as live conflict for safety"
                );
                ExistingLockStatus::LiveConflict { pid: pid_i32 }
            }
        }
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
        // Remove lock file.
        std::fs::remove_file(&self.path).map_err(|e| {
            tracing::error!(
                path = %self.path.display(),
                error = %e,
                "failed to remove lock file on release"
            );
            e
        })?;

        // Remove socket file. If it doesn't exist (e.g., daemon crashed before binding),
        // treat NotFound as success to avoid masking the successful lock file removal.
        match std::fs::remove_file(&self.sock_path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!(
                    path = %self.sock_path.display(),
                    "sock file not found on release (not an error)"
                );
            }
            Err(e) => {
                tracing::error!(
                    path = %self.sock_path.display(),
                    error = %e,
                    "failed to remove sock file on release"
                );
                return Err(e);
            }
        }

        tracing::info!(
            lock_path = %self.path.display(),
            sock_path = %self.sock_path.display(),
            "daemon lock released"
        );

        Ok(())
    }
}
