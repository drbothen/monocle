//! Runtime directory resolution and daemon lifecycle (exit-code taxonomy) for the monocle daemon.
//!
//! Populated by S-006 (Lock File Atomic Lifecycle) and S-005 (Graceful Shutdown).
//!
//! # XDG / Platform Resolution Order
//!
//! 1. `MONOCLE_RUNTIME_DIR` environment variable (highest priority; used in tests and
//!    container deployments).
//! 2. `directories::ProjectDirs::from("", "", "monocle")`:
//!    - `runtime_dir()` — returns `Some` on Linux (XDG_RUNTIME_DIR), `None` on macOS.
//!    - Fallback: `data_local_dir()` — always set on all supported platforms.
//! 3. If `ProjectDirs::from("", "", "monocle")` returns `None` (e.g., `$HOME` is unset):
//!    `Err(DaemonStartError::RuntimeDirUnresolvable)`.
//!
//! # Security
//!
//! The runtime directory MUST be created with mode `0o700` (owner-only read/write/execute)
//! to prevent other local users from reading the lock file and extracting the auth token.
//! `ensure_runtime_dir` enforces this via `DirBuilder::new().mode(0o700)`.
//!
//! # Daemon Exit-Code Taxonomy (BC-2.01.004 PC-8)
//!
//! [`DaemonExit`] encodes the 5-code POSIX exit taxonomy for the monocle daemon.
//! [`exit_with`] is the **sole call-site** for `std::process::exit` in the entire codebase.
//! No handler, task, or signal-callback may call `std::process::exit` directly.
//! This invariant is enforced by SS-conventions-anti-patterns.md §"No `std::process::exit`
//! in handler code" and verified by a structural source-grep test.

use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use crate::errors::DaemonStartError;
use crate::types::{CheckpointReadResult, RecoveryCheckpoint};

/// Resolve the monocle daemon runtime directory path.
///
/// Checks `MONOCLE_RUNTIME_DIR` first, then falls back to
/// `directories::ProjectDirs::from("", "", "monocle")`.
/// On macOS, `runtime_dir()` returns `None`; the fallback is `data_local_dir()`.
///
/// # Errors
///
/// Returns [`DaemonStartError::RuntimeDirUnresolvable`] when both the environment
/// variable is absent AND `ProjectDirs::from("", "", "monocle")` returns `None` (which
/// happens when `$HOME` is unset or the platform provides no home directory equivalent).
pub fn resolve_runtime_dir() -> Result<PathBuf, DaemonStartError> {
    // Priority 1: MONOCLE_RUNTIME_DIR env var (non-empty).
    if let Ok(dir) = std::env::var("MONOCLE_RUNTIME_DIR") {
        if !dir.is_empty() {
            tracing::info!(
                dir = %dir,
                "runtime_dir from MONOCLE_RUNTIME_DIR env var"
            );
            return Ok(PathBuf::from(dir));
        }
    }

    // Priority 2: platform ProjectDirs.
    //
    // Before delegating to `directories::ProjectDirs::from()`, verify that the
    // environment provides a usable home directory via env vars. On macOS,
    // `dirs-sys` has a `getpwuid_r` fallback that can resolve a home directory
    // even when `HOME` is not set — but in that case the path is not under the
    // user's control and cannot be trusted in a headless/container deployment.
    //
    // BC-2.01.005 PC-2d / EC-059: when HOME and all XDG paths are absent from the
    // environment, return Err(RuntimeDirUnresolvable) rather than silently using
    // a path derived from the password database.
    //
    // We check the same variables that `directories` / `dirs-sys` inspect on
    // each platform so that `temp_env::with_vars` in tests can reliably trigger
    // the unresolvable path.
    #[cfg(target_os = "macos")]
    let home_available = std::env::var_os("HOME")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    #[cfg(not(target_os = "macos"))]
    let home_available = {
        // Linux / Unix: HOME or XDG_DATA_HOME are the primary env vars.
        let has_home = std::env::var_os("HOME")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let has_xdg = std::env::var_os("XDG_DATA_HOME")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        has_home || has_xdg
    };

    if !home_available {
        tracing::warn!(
            "HOME env var is not set; cannot resolve platform runtime dir (BC-2.01.005 PC-2d)"
        );
        return Err(DaemonStartError::RuntimeDirUnresolvable);
    }

    let proj = directories::ProjectDirs::from("", "", "monocle")
        .ok_or(DaemonStartError::RuntimeDirUnresolvable)?;

    // runtime_dir() is Some on Linux (XDG_RUNTIME_DIR), None on macOS.
    if let Some(rd) = proj.runtime_dir() {
        // BC-2.04.006 PC-6: log Level 2 selection before returning.
        tracing::info!("runtime_dir from ProjectDirs::runtime_dir()");
        Ok(rd.to_path_buf())
    } else {
        // BC-2.04.006 PC-9: log Level 3 selection with platform inline in message.
        tracing::info!(
            "runtime_dir fallback to data_local_dir (platform: {})",
            std::env::consts::OS
        );
        Ok(proj.data_local_dir().to_path_buf())
    }
}

/// Ensure the runtime directory exists with mode `0o700`.
///
/// Creates the directory (and any missing parent directories) using
/// `DirBuilder::new().mode(0o700).recursive(true).create(path)`.
///
/// # Errors
///
/// Returns [`DaemonStartError::RuntimeDirCreateFailure`] wrapping the underlying
/// `std::io::Error` if directory creation fails.
///
/// # Platform note
///
/// The `mode()` method requires `std::os::unix::fs::DirBuilderExt`, which is only
/// available on Unix targets. Windows is not a supported platform for S-006.
pub fn ensure_runtime_dir(path: &Path) -> Result<(), DaemonStartError> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(path)
        .map_err(DaemonStartError::RuntimeDirCreateFailure)
}

// ---------------------------------------------------------------------------
// Daemon exit-code taxonomy (BC-2.01.004 PC-8, S-005)
// ---------------------------------------------------------------------------

/// The reason the monocle daemon is terminating.
///
/// Encodes the 5-code POSIX exit taxonomy from BC-2.01.004 PC-8. Every daemon termination
/// path MUST pass through [`exit_with`], which is the **sole call-site** for
/// `std::process::exit` in the codebase (SS-conventions-anti-patterns.md §"No
/// `std::process::exit` in handler code").
///
/// # POSIX 128+N convention
///
/// Signal-induced exits follow the POSIX 128+N convention (128 + signal number):
/// - SIGINT = signal 2 → exit code 130 (128+2)
/// - SIGTERM = signal 15 → exit code 143 (128+15)
///
/// External monitoring systems (systemd `Restart=on-failure`, k8s
/// `terminationGracePeriodSeconds`, CI status parsers) **MUST** use exit code 143
/// (not 130) to detect SIGTERM hard-kill during drain. Exit 130 encodes SIGINT
/// (Ctrl-C second press), not SIGTERM (BC-2.01.004 INV-4).
///
/// # Non-POSIX-128+N codes
///
/// - Exit `2` (AdminForceStop) is a monocle-specific code outside the POSIX 128+N
///   range (which starts at 129). It is distinct from startup-failure exit 1 so that
///   monitoring systems can distinguish operator-initiated force-stop from daemon
///   startup failure.
/// - Exit `1` (StartupFailure) is the conventional failure exit code; it covers all
///   cases where the daemon could not start (runtime directory unresolvable, port bind
///   failure, existing live lock file).
/// - Exit `0` (Graceful) means all in-flight requests completed within the 10-second
///   drain window. The lock file is removed and the UDS socket is closed before this
///   exit fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonExit {
    /// All in-flight requests completed within the 10-second drain window.
    ///
    /// Lock file and UDS socket removed before exit. Exit code `0`.
    Graceful,

    /// Daemon failed to start (runtime directory unresolvable, port bind failure,
    /// or existing live lock file). Exit code `1`.
    StartupFailure,

    /// A second authenticated `POST /shutdown` was received while a drain was already
    /// in progress. Monocle-specific programmatic code (outside POSIX 128+N range).
    /// Exit code `2`.
    AdminForceStop,

    /// A second SIGINT (signal 2, Ctrl-C) was received while a drain was in progress.
    /// POSIX convention 128+2. Exit code `130`.
    SigintDuringDrain,

    /// A second SIGTERM (signal 15) was received while a drain was in progress.
    /// POSIX convention 128+15. Exit code `143`.
    ///
    /// Note: the drain timeout expiring WITHOUT a second SIGTERM is NOT this variant —
    /// drain-timeout-forced-shutdown exits with `DaemonExit::Graceful` (exit code 0)
    /// per story spec line 171: "drain-timeout-forced-shutdown exits 0". This variant
    /// is exclusively for when a second SIGTERM arrives before the drain window closes.
    SigtermDuringDrain,
}

impl DaemonExit {
    /// Map this exit reason to the OS exit code.
    ///
    /// | Variant | Code | Rationale |
    /// |---------|------|-----------|
    /// | `Graceful` | 0 | Clean drain |
    /// | `StartupFailure` | 1 | Startup error |
    /// | `AdminForceStop` | 2 | Second POST /shutdown during drain |
    /// | `SigintDuringDrain` | 130 | POSIX 128+2 (SIGINT=2) |
    /// | `SigtermDuringDrain` | 143 | POSIX 128+15 (SIGTERM=15) |
    pub fn to_exit_code(self) -> i32 {
        match self {
            DaemonExit::Graceful => 0,
            DaemonExit::StartupFailure => 1,
            DaemonExit::AdminForceStop => 2,
            DaemonExit::SigintDuringDrain => 130,
            DaemonExit::SigtermDuringDrain => 143,
        }
    }
}

// ---------------------------------------------------------------------------
// Crash recovery checkpoint I/O (BC-2.01.006, S-007)
// ---------------------------------------------------------------------------

/// Write a crash recovery checkpoint to `path` atomically via `tempfile::persist`.
///
/// The checkpoint is serialised to JSON and written to a temporary file in the same
/// directory as `path`, then renamed atomically so readers never observe a partial file.
/// BC-2.01.006 postcondition: the checkpoint file MUST be present after the daemon
/// writes it, even if the process is subsequently killed.
///
/// # Errors
///
/// Returns an error if:
/// - The parent directory of `path` is inaccessible.
/// - JSON serialisation of `checkpoint` fails.
/// - The `tempfile::persist` call fails (e.g., cross-device rename on Linux).
pub fn write_recovery_checkpoint(
    path: &Path,
    checkpoint: &RecoveryCheckpoint,
) -> anyhow::Result<()> {
    // Validate BC-2.01.006 INV-1 field constraints before writing.
    checkpoint.validate()?;

    let json = serde_json::to_string(checkpoint)?;

    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "checkpoint path has no parent directory: {}",
            path.display()
        )
    })?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(json.as_bytes())?;

    // Set 0o600 before persisting so the final file is always owner-only.
    std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o600))?;

    tmp.persist(path).map_err(|e| {
        anyhow::anyhow!(
            "failed to persist checkpoint to {}: {}",
            path.display(),
            e.error
        )
    })?;

    Ok(())
}

/// Read a recovery checkpoint from `path`.
///
/// Returns a [`CheckpointReadResult`] distinguishing three states required by EC-054:
/// - [`CheckpointReadResult::Valid`] — file present, valid JSON, INV-1 fields satisfied.
/// - [`CheckpointReadResult::Malformed`] — file present but truncated, invalid JSON, or
///   INV-1 field validation failure (e.g., `pid = 0`, empty `last_app_mode`, bad timestamp).
/// - [`CheckpointReadResult::Absent`] — file does not exist (clean boot / graceful shutdown).
///
/// TUI startup is never blocked: malformed and absent both indicate no recovery needed,
/// but the distinction allows differentiated log messages at the call site.
pub fn read_recovery_checkpoint(path: &Path) -> CheckpointReadResult {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return CheckpointReadResult::Absent,
        Err(_) => return CheckpointReadResult::Malformed,
    };
    match serde_json::from_str::<RecoveryCheckpoint>(&contents) {
        Ok(cp) if cp.validate().is_ok() => CheckpointReadResult::Valid(cp),
        _ => CheckpointReadResult::Malformed,
    }
}

// ---------------------------------------------------------------------------
// Daemon start sequence (S-017, BC-2.04.001 + BC-2.04.010)
// ---------------------------------------------------------------------------

/// Canonical JSON content for the S-017 lock file (BC-2.04.001 step 8).
///
/// This struct uses a STRING `contract_version` field (`"monocle-lock-v1"`) per
/// BC-2.04.001 PC-8, which differs from the legacy [`crate::lock::LockFileContent`]
/// that uses an integer `contract_version: u32`. The two formats are intentionally
/// separate to allow the S-006 reader (integer format) to remain compatible while
/// the S-017 start sequence emits the canonical string format.
///
/// # Code duplication note
///
/// `StartSequenceLockContent` (this struct) and `crate::lock::LockFileContent` coexist
/// intentionally: they serve different schemas. The S-006 `LockFileContent` uses integer
/// `contract_version: u32` (legacy). This struct uses string `contract_version` per the
/// canonical S-017 BC-2.04.001 PC-8 wire format. The two must not be merged.
///
/// # Wire format (BC-2.04.001 PC-15 — exactly 5 fields)
///
/// ```json
/// {
///   "pid": 12345,
///   "port": 9001,
///   "token": "monocle-v1:<64-hex>",
///   "contract_version": "monocle-lock-v1",
///   "started_at": "2026-05-27T10:00:00.000Z"
/// }
/// ```
///
/// Note: `pid` is `i32` for consistency with nix PID types.
/// `token` stores the full `monocle-v1:<64-hex>` wire token.
/// `started_at` is an ISO 8601 UTC timestamp with millisecond precision.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StartSequenceLockContent {
    /// PID of the daemon process. `i32` for consistency with nix::unistd::Pid.
    pid: i32,
    /// TCP port the HTTP server is bound to.
    port: u16,
    /// Wire-format token: `monocle-v1:<64-hex>`.
    token: String,
    /// Schema version string — always `"monocle-lock-v1"`.
    contract_version: String,
    /// ISO 8601 UTC timestamp (millisecond precision) recording when the daemon started.
    started_at: String,
}

/// Execute the full 13-step daemon start sequence (BC-2.04.001).
///
/// # Return value
///
/// Returns `Ok(Arc<DaemonState>)` on success. The returned `DaemonState` is fully
/// wired: it holds the ring buffer, event bus, session registry, pending-decisions
/// registry, auth token, daemon lock, and the spawned UDS accept loop join handle.
///
/// # SOQ-2 Commit-Point Invariant
///
/// Steps are executed in strict order. The SOQ-2 invariant requires:
/// - Step 3 (port bind) BEFORE Step 8 (lock file write) BEFORE Step 9 (hooks-settings.json write).
///
/// If any step fails, subsequent steps are NOT executed. Steps 1–7 are pre-commit-point;
/// a failure there leaves the filesystem clean. Post-Step-8 failures MUST clean up the
/// lock file (BC-2.04.001 invariant 6).
///
/// # Steps
///
/// 1. Resolve runtime dir and call `ensure_runtime_dir` (mode 0o700).
/// 2. Fail-fast stale lock check: read lock file if present, send signal 0 to check PID
///    liveness. If live, return `LockFileConflict`. If stale, remove and continue.
/// 3. Bind `TcpListener` on `127.0.0.1:0` and record the allocated port.
/// 4. Construct `RingBuffer` with 100 MiB × 5 rotations, Arc-wrapped.
/// 5. Create bounded `mpsc::channel::<EventBusHookEvent>(4096)`, drop counter `AtomicU64`.
/// 6. Register `ClaudeCodeModule` + `VsddFactoryAdapter` in `EngineModuleRegistry`.
/// 7. Generate session token: `OsRng` → 32 bytes → 64 hex lowercase (no `monocle-v1:` prefix).
/// 8. **SOQ-2 commit point**: write lock file via `write_lock_file`, mode 0o600.
/// 9. Write `hooks-settings.json` via `write_hooks_settings`, mode 0o600.
/// 10. Remove stale UDS socket at `<runtime_dir>/monocle.sock`, bind tokio `UnixListener`,
///     mode 0o600. Spawn `run_accept_loop` as a Tokio task.
/// 11. Init crash recovery checkpoint infrastructure (stateless; verifies path is writable).
/// 12. Signal startup complete.
///
/// # Errors
///
/// Returns [`DaemonStartError`] on any step failure. Post-step-8 failures trigger
/// lock file cleanup before returning (invariant 6 of BC-2.04.001).
pub async fn daemon_start_sequence(
    runtime_dir: &Path,
) -> Result<std::sync::Arc<crate::state::DaemonState>, DaemonStartError> {
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    use tokio::net::TcpListener;

    // Step 1: Ensure runtime dir exists with mode 0o700.
    ensure_runtime_dir(runtime_dir)?;

    // Step 2: Fail-fast stale lock check — detect a live daemon before any expensive
    // operations (BC-2.04.001 step 2). Read the lock file if present, check PID liveness
    // via signal 0. If live: return LockFileConflict. If stale: remove and continue.
    // This is a fail-fast optimization; write_lock_file also checks (defense in depth).
    {
        let lock_path = runtime_dir.join("monocle.lock");
        if lock_path.exists() {
            let raw = match std::fs::read_to_string(&lock_path) {
                Ok(s) => s,
                Err(_) => {
                    tracing::warn!(
                        path = %lock_path.display(),
                        "daemon_start_sequence: step 2 — lock file exists but cannot be read; \
                         treating as stale and removing"
                    );
                    let _ = std::fs::remove_file(&lock_path);
                    String::new()
                }
            };
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(pid) = json.get("pid").and_then(|v| v.as_i64()) {
                    if let Ok(pid_i32) = i32::try_from(pid) {
                        let nix_pid = nix::unistd::Pid::from_raw(pid_i32);
                        if nix::sys::signal::kill(nix_pid, None).is_ok() {
                            tracing::warn!(
                                pid = pid_i32,
                                "daemon_start_sequence: step 2 — live daemon detected (LockFileConflict)"
                            );
                            return Err(DaemonStartError::LockFileConflict { pid: pid_i32 });
                        }
                        // Stale lock: process not found — remove it.
                        tracing::info!(
                            pid = pid_i32,
                            path = %lock_path.display(),
                            "daemon_start_sequence: step 2 — stale lock detected; removing"
                        );
                        let _ = std::fs::remove_file(&lock_path);
                    }
                }
            }
        }
    }

    // Step 3: Bind TcpListener on 127.0.0.1:0 — record OS-assigned port.
    // Done before any file write (SOQ-2 invariant: port bind precedes commit point).
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(DaemonStartError::PortBindFailure)?;
    let port = listener
        .local_addr()
        .map_err(DaemonStartError::PortBindFailure)?
        .port();

    // Step 4: Construct RingBuffer with 100 MiB hard cap × 5 rotations.
    // Filename: monocle-events.jsonl (established by S-008 ring buffer tests).
    let ring_path = runtime_dir.join("monocle-events.jsonl");
    let ring_config = crate::ring::RotationConfig {
        soft_threshold_bytes: 50 * 1024 * 1024,
        hard_cap_bytes: 100 * 1024 * 1024,
        retained: 5,
    };
    let ring: Arc<crate::ring::RingBuffer> =
        Arc::new(crate::ring::RingBuffer::new(ring_path, ring_config));

    // Step 5: Create bounded mpsc channel for event bus (4096 slots), drop counter AtomicU64.
    let (event_tx, _event_rx) = tokio::sync::mpsc::channel::<crate::types::EventBusHookEvent>(
        crate::types::EVENT_BUS_CAPACITY,
    );
    let drop_counter: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));

    // Step 6: Register engine modules.
    let mut registry = crate::types::EngineModuleRegistry::new();
    registry.register_claude_code();
    registry.register_vsdd_factory();
    debug_assert!(
        registry.is_complete(),
        "engine module registry must be complete after step 6"
    );

    // Step 7: Generate session token — 32 bytes OsRng → 64 hex lowercase (no monocle-v1: prefix).
    let auth_token = crate::auth::generate_session_token();

    // Step 8: SOQ-2 commit point — write lock file atomically.
    let lock_result = write_lock_file(runtime_dir, port, &auth_token);
    let (daemon_lock, auth_token) = match lock_result {
        Ok(pair) => pair,
        Err(e) => {
            // Pre-commit-point failure: no cleanup needed (lock file was not written).
            tracing::error!(
                error = %e,
                "daemon_start_sequence: step 8 (write_lock_file) failed; aborting"
            );
            return Err(e);
        }
    };

    // Step 9: Write hooks-settings.json AFTER lock file (SOQ-2).
    // INV-6: on failure, remove lock file before returning.
    if let Err(e) = write_hooks_settings(runtime_dir, port, &auth_token) {
        tracing::error!(
            error = %e,
            "daemon_start_sequence: step 9 (write_hooks_settings) failed; removing lock file (INV-6)"
        );
        if let Err(cleanup_err) = daemon_lock.release() {
            tracing::error!(
                error = %cleanup_err,
                "daemon_start_sequence: lock file cleanup failed after step 9 failure"
            );
        }
        return Err(e);
    }

    // Step 10: Remove stale UDS socket, bind tokio UnixListener (mode 0o600),
    // build DaemonState with all wired components, and spawn run_accept_loop.
    //
    // The tokio UnixListener is retained for the accept loop — it is NOT dropped after bind.
    // The socket file remains on disk as required by BC-2.04.001 PC-13.
    let uds_listener = {
        let sock_path = runtime_dir.join("monocle.sock");
        if sock_path.exists() {
            if let Err(e) = std::fs::remove_file(&sock_path) {
                tracing::warn!(
                    path = %sock_path.display(),
                    error = %e,
                    "daemon_start_sequence: failed to remove stale socket (step 10)"
                );
                // INV-6: post-step-8 failure — remove lock file before returning.
                if let Err(cleanup_err) = daemon_lock.release() {
                    tracing::error!(
                        error = %cleanup_err,
                        "daemon_start_sequence: lock file cleanup failed after step 10 \
                         (stale socket removal) failure"
                    );
                }
                return Err(DaemonStartError::UdsBindFailure(e));
            }
        }

        match tokio::net::UnixListener::bind(&sock_path) {
            Ok(uds) => {
                // Set mode 0o600 — owner-only access (BC-2.05.001 PC-2).
                if let Err(e) =
                    std::fs::set_permissions(&sock_path, std::fs::Permissions::from_mode(0o600))
                {
                    tracing::error!(
                        path = %sock_path.display(),
                        error = %e,
                        "daemon_start_sequence: failed to set socket permissions (step 10)"
                    );
                    // INV-6: remove lock file before returning.
                    if let Err(cleanup_err) = daemon_lock.release() {
                        tracing::error!(
                            error = %cleanup_err,
                            "daemon_start_sequence: lock file cleanup failed after step 10 \
                             (socket permission) failure"
                        );
                    }
                    return Err(DaemonStartError::UdsBindFailure(e));
                }
                uds
            }
            Err(e) => {
                tracing::error!(
                    path = %sock_path.display(),
                    error = %e,
                    "daemon_start_sequence: UDS bind failed (step 10)"
                );
                // INV-6: remove lock file before returning.
                if let Err(cleanup_err) = daemon_lock.release() {
                    tracing::error!(
                        error = %cleanup_err,
                        "daemon_start_sequence: lock file cleanup failed after step 10 \
                         (UDS bind) failure"
                    );
                }
                return Err(DaemonStartError::UdsBindFailure(e));
            }
        }
    };

    // Build the fully-wired DaemonState (BC-2.04.001 all steps assembled).
    //
    // pending_decisions is initialised here so the PreToolUse Defer path can
    // register prompts without a None-guard failure (BC-2.05.005 PC-1).
    //
    // ipc_subscribers is constructed before DaemonState so the Arc can be cloned
    // for the accept-loop spawn without reaching into Option<> after construction.
    let ipc_subscribers: monocle_ipc::server::SubscriberList =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let sock_file_path = runtime_dir
        .join("monocle.sock")
        .to_string_lossy()
        .into_owned();
    let lock_file_path = runtime_dir
        .join("monocle.lock")
        .to_string_lossy()
        .into_owned();
    let daemon_state = Arc::new(crate::state::DaemonState {
        mode: std::sync::RwLock::new(crate::state::AppMode::Running),
        start_time: std::time::Instant::now(),
        hook_receiver_status: None,
        auth_token,
        lock_file_path,
        sock_file_path,
        last_hook_ts: std::sync::RwLock::new(crate::state::LastHookTimestamps::default()),
        ring: Some(ring),
        tui_attached: std::sync::atomic::AtomicBool::new(false),
        force_exit: std::sync::atomic::AtomicBool::new(false),
        daemon_lock: std::sync::Mutex::new(Some(daemon_lock)),
        shutdown_tx,
        shutdown_rx,
        event_bus_tx: Some(Arc::new(event_tx)),
        drop_counter: Some(drop_counter),
        session_registry: Some(Arc::new(crate::hooks::SessionRegistry::new())),
        pending_decisions: Some(Arc::new(crate::permissions::PendingDecisionRegistry::new())),
        ipc_subscribers: Some(Arc::clone(&ipc_subscribers)),
        hook_decision_override: None,
        hook_delay_ms: None,
    });

    // Spawn the UDS accept loop (BC-2.05.002 PC-1).
    // Clone shutdown_rx and ipc_subscribers before moving into the spawned task.
    // The accept loop races accept() against the shutdown watch receiver so that
    // graceful shutdown terminates the loop cleanly within ~1 scheduler tick
    // instead of blocking on the next accept() call (F-ADV2-HIGH-001).
    let accept_shutdown_rx = daemon_state.shutdown_rx.clone();
    let accept_state = Arc::clone(&daemon_state);
    let accept_subscribers = Arc::clone(&ipc_subscribers);
    tokio::spawn(async move {
        crate::ipc_server::run_accept_loop(
            uds_listener,
            accept_state,
            accept_subscribers,
            accept_shutdown_rx,
        )
        .await;
    });

    // Step 11: Crash recovery checkpoint infrastructure is initialized implicitly —
    // write_recovery_checkpoint and read_recovery_checkpoint are stateless functions
    // operating on the runtime dir path. No additional init required.
    tracing::debug!(
        runtime_dir = %runtime_dir.display(),
        "daemon_start_sequence: crash recovery checkpoint infrastructure ready (step 11)"
    );

    // Step 12: Startup complete. Log and return.
    tracing::info!(
        runtime_dir = %runtime_dir.display(),
        port = port,
        "daemon_start_sequence: all steps complete (BC-2.04.001)"
    );

    Ok(daemon_state)
}

/// Write `hooks-settings.json` atomically into the runtime directory (BC-2.04.010).
///
/// Creates a `NamedTempFile` in `runtime_dir`, serializes the [`crate::types::HooksSettings`]
/// struct to pretty-printed JSON via `serde_json::to_writer_pretty`, sets mode 0o600, then
/// persists the file to `<runtime_dir>/hooks-settings.json` via `tempfile::persist`.
///
/// # Schema
///
/// The generated JSON has:
/// - 4 active hooks: `PreToolUse`, `Notification`, `Stop`, `UserPromptSubmit`
/// - `PostToolUse`: empty array `[]` (JC-2 — PostToolUse absent from Phase 1)
/// - `PreCompact`: empty array `[]` (not in canonical 5 endpoints)
/// - `SessionStart`: absent entirely (JC-2 closure)
///
/// Each active hook entry contains a single `command` item invoking the monocle hook
/// bridge script at `http://127.0.0.1:<port>/hooks/<hook-path>` with the
/// `monocle-v1:<auth_token>` wire token in the `X-Monocle-Authorization` header.
///
/// # SOQ-2 Invariant
///
/// This function MUST only be called AFTER `write_lock_file` has succeeded
/// (BC-2.04.001 step 9 follows step 8). Callers in `daemon_start_sequence` enforce this.
///
/// # Errors
///
/// Returns [`DaemonStartError::HooksSettingsWriteFailure`] on I/O failure.
pub fn write_hooks_settings(
    runtime_dir: &Path,
    port: u16,
    auth_token: &str,
) -> Result<(), DaemonStartError> {
    use crate::types::{HookEntry, HooksMap, HooksSettings};

    // Build the wire token with monocle-v1: prefix for the Authorization header.
    let wire_token = format!("monocle-v1:{auth_token}");

    // Build the curl command for a given hook endpoint path.
    let make_command = |hook_path: &str| -> String {
        format!(
            "curl -s -X POST http://127.0.0.1:{port}/hooks/{hook_path} \
            -H 'Content-Type: application/json' \
            -H 'X-Monocle-Authorization: {wire_token}' \
            -d @-"
        )
    };

    let settings = HooksSettings {
        hooks: HooksMap {
            pre_tool_use: vec![HookEntry::command_all(make_command("pre-tool-use"))],
            post_tool_use: vec![],
            notification: vec![HookEntry::command_all(make_command("notification"))],
            stop: vec![HookEntry::command_all(make_command("stop"))],
            user_prompt_submit: vec![HookEntry::command_all(make_command("prompt-submit"))],
            pre_compact: vec![],
        },
    };

    let hs_path = runtime_dir.join("hooks-settings.json");

    let mut tmp = tempfile::NamedTempFile::new_in(runtime_dir)
        .map_err(DaemonStartError::HooksSettingsWriteFailure)?;

    serde_json::to_writer_pretty(&mut tmp, &settings).map_err(|e| {
        DaemonStartError::HooksSettingsWriteFailure(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("JSON serialize failed: {e}"),
        ))
    })?;

    tmp.flush()
        .map_err(DaemonStartError::HooksSettingsWriteFailure)?;

    tmp.persist(&hs_path)
        .map_err(|e| DaemonStartError::HooksSettingsWriteFailure(e.error))?;

    std::fs::set_permissions(&hs_path, std::fs::Permissions::from_mode(0o600))
        .map_err(DaemonStartError::HooksSettingsWriteFailure)?;

    tracing::info!(
        path = %hs_path.display(),
        port = port,
        "hooks-settings.json written (BC-2.04.010)"
    );

    Ok(())
}

/// Write the daemon lock file (BC-2.04.001 step 8, SOQ-2 commit point).
///
/// Uses a [`StartSequenceLockContent`] struct with string `contract_version` field
/// (`"monocle-lock-v1"`) per BC-2.04.001 PC-8, distinct from the legacy
/// [`crate::lock::LockFileContent`] that uses `contract_version: u32`.
///
/// # Lock file schema (BC-2.04.001 PC-15 — exactly 5 fields)
///
/// ```json
/// {
///   "pid": 12345,
///   "port": 9001,
///   "token": "monocle-v1:<64-hex>",
///   "contract_version": "monocle-lock-v1",
///   "started_at": "2026-05-27T10:00:00.000Z"
/// }
/// ```
///
/// - `token`: wire-format token WITH the `monocle-v1:` prefix.
/// - `contract_version`: string `"monocle-lock-v1"` (BC-2.04.001 PC-8).
/// - `pid`: `i32` for consistency with nix::unistd::Pid (safe — modern OS PIDs fit in i32).
///
/// # Errors
///
/// Returns [`DaemonStartError`] on I/O failure or live-PID conflict.
pub fn write_lock_file(
    runtime_dir: &Path,
    port: u16,
    auth_token: &str,
) -> Result<(crate::lock::DaemonLock, String), DaemonStartError> {
    use std::io::Write as _;

    let lock_path = runtime_dir.join("monocle.lock");
    let sock_path = runtime_dir.join("monocle.sock");

    // Defense-in-depth check for an existing live lock (primary check is step 2
    // in daemon_start_sequence; this is a second guard at the write call-site).
    if lock_path.exists() {
        // Read and check for live PID conflicts via nix::sys::signal::kill.
        let raw = std::fs::read_to_string(&lock_path).unwrap_or_else(|_| {
            tracing::warn!(
                path = %lock_path.display(),
                "write_lock_file: lock file exists but cannot be read; treating as stale"
            );
            String::new()
        });
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(pid) = json.get("pid").and_then(|v| v.as_i64()) {
                if let Ok(pid_i32) = i32::try_from(pid) {
                    let nix_pid = nix::unistd::Pid::from_raw(pid_i32);
                    if nix::sys::signal::kill(nix_pid, None).is_ok() {
                        return Err(DaemonStartError::LockFileConflict { pid: pid_i32 });
                    }
                }
            }
        }
        // Stale lock — remove it before writing fresh one.
        let _ = std::fs::remove_file(&lock_path);
    }

    // Build the ISO 8601 UTC started_at timestamp.
    let started_at = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    // Build the wire token with the monocle-v1: prefix.
    let wire_token = format!("monocle-v1:{auth_token}");

    let content = StartSequenceLockContent {
        // i32 cast is safe: modern OS PIDs always fit in i32 (max 4194304 on Linux).
        pid: std::process::id() as i32,
        port,
        token: wire_token,
        contract_version: "monocle-lock-v1".to_string(),
        started_at: started_at.clone(),
    };

    // Serialize and persist atomically via tempfile.
    let mut tmp = tempfile::NamedTempFile::new_in(runtime_dir)?;
    serde_json::to_writer_pretty(&mut tmp, &content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("JSON serialize lock file failed: {e}"),
        )
    })?;
    tmp.flush()?;
    tmp.persist(&lock_path).map_err(|e| e.error)?;

    // Set 0o600 permissions on the lock file.
    std::fs::set_permissions(&lock_path, std::fs::Permissions::from_mode(0o600))?;

    tracing::info!(
        path = %lock_path.display(),
        port = port,
        "lock file written at SOQ-2 commit point (BC-2.04.001 step 8)"
    );

    Ok((
        crate::lock::DaemonLock {
            path: lock_path,
            sock_path,
        },
        auth_token.to_string(),
    ))
}

/// Remove `hooks-settings.json` from the runtime directory on graceful shutdown.
///
/// Called by the graceful shutdown path after `DaemonLock::release()` completes
/// and before `exit_with(DaemonExit::Graceful)` is called (BC-2.04.010 PC-5).
///
/// Removing hooks-settings.json ensures Claude Code stops posting hooks to the
/// (now-dead) daemon endpoint. If the file does not exist (e.g., daemon crashed
/// before step 9), this function returns `Ok(())` (idempotent).
///
/// # Error handling (BC-2.04.010 PC-5)
///
/// Errors are logged at WARN level and swallowed — shutdown MUST continue even if
/// the file cannot be removed (e.g., permission denied). This function always
/// returns `Ok(())`.
pub fn remove_hooks_settings(runtime_dir: &Path) -> std::io::Result<()> {
    let hs_path = runtime_dir.join("hooks-settings.json");
    match std::fs::remove_file(&hs_path) {
        Ok(()) => {
            tracing::info!(
                path = %hs_path.display(),
                "hooks-settings.json removed on graceful shutdown (BC-2.04.010 PC-5)"
            );
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(
                path = %hs_path.display(),
                "hooks-settings.json not found on shutdown removal (already absent)"
            );
        }
        Err(e) => {
            // BC-2.04.010 PC-5: log at WARN and continue — shutdown must not be blocked
            // by a hooks-settings.json removal failure.
            tracing::warn!(
                path = %hs_path.display(),
                error = %e,
                "failed to remove hooks-settings.json on shutdown; continuing (BC-2.04.010 PC-5)"
            );
        }
    }
    Ok(())
}

/// Terminate the daemon process with the exit code corresponding to `reason`.
///
/// This is the **sole call-site** for `std::process::exit` in the entire monocle codebase.
/// All daemon termination paths MUST call this function rather than calling
/// `std::process::exit` directly (SS-conventions-anti-patterns.md §"No
/// `std::process::exit` in handler code").
///
/// On a graceful exit (`DaemonExit::Graceful`), the S-005 implementation must invoke
/// `DaemonLock::release()` BEFORE calling this function to ensure the lock file and UDS
/// socket are removed from the filesystem (BC-2.01.004 PC-7).
///
/// # Stub note (S-005 Red Gate)
///
/// The S-005 implementation will wire `exit_with` as the final step of the graceful
/// shutdown sequence. During the Red Gate phase, the stub returns HTTP 501 from
/// `handlers::shutdown::post_shutdown` and this function is not called by any test path
/// that exercises the shutdown behavior.
///
/// # Returns
///
/// This function never returns (`-> !`). The process exits immediately.
pub fn exit_with(reason: DaemonExit) -> ! {
    let code = reason.to_exit_code();
    tracing::info!(
        exit_code = code,
        reason = ?reason,
        "daemon terminating"
    );
    std::process::exit(code)
}
