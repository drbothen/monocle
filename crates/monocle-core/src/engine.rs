/// EngineModule trait and supporting types (BC-2.03.001; S-014).
///
/// Tests in `engine_module_surface.rs` (VP-019 AST audit suite) verify structural
/// conformance to BC-2.03.001 and SS-engine-module.md v1.1.20.
use std::collections::HashMap;
use std::path::PathBuf;

use crate::hook_events::HookEvent;

/// Abstraction over an AI coding harness engine (Claude Code, CodeMachine, etc.).
///
/// # Why `#[async_trait]`?
///
/// native async fn in traits does not provide ergonomic dyn-compatibility on MSRV 1.86.
/// The `async_trait` macro desugars async methods to `Pin<Box<dyn Future>>`, which is the
/// only stable mechanism for `dyn EngineModule` trait objects on Rust 1.86. Once
/// `async fn` in dyn traits stabilises (tracked in RFC 3185), the macro can be removed
/// without changing the public API surface.
#[async_trait::async_trait]
pub trait EngineModule: Send + Sync + 'static {
    /// Stable identifier for this engine.
    fn id(&self) -> &'static str;

    /// Static metadata describing this engine.
    fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError>;

    /// Detect whether a running process matches this engine's signature.
    fn detect(&self, proc: &ProcessSnapshot) -> bool;

    /// Enrich a detected process snapshot with engine-specific context.
    async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>;

    /// Process an inbound hook event, returning the dispatch decision.
    async fn on_hook(&self, event: HookEvent) -> HookResponse;
}

/// Human-readable harness metadata.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct EngineMetadata {
    /// Display name shown in the Sessions panel header.
    pub display_name: &'static str,
    /// Single char icon shown next to session entries.
    pub icon: char,
    /// Canonical config file locations for this engine.
    pub config_paths: Vec<PathBuf>,
    /// Version of the hook protocol this engine speaks.
    pub hook_schema_version: u32,
}

impl EngineMetadata {
    /// Construct an `EngineMetadata` instance.
    pub fn new(
        display_name: &'static str,
        icon: char,
        config_paths: Vec<PathBuf>,
        hook_schema_version: u32,
    ) -> Self {
        Self {
            display_name,
            icon,
            config_paths,
            hook_schema_version,
        }
    }
}

/// A snapshot of a running process.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    /// Process ID of the running process.
    pub pid: u32,
    /// Parent process ID, if available.
    pub ppid: Option<u32>,
    /// Resolved binary path.
    pub exe_path: Option<PathBuf>,
    /// Full command line including argv[0] and all arguments.
    pub cmdline: Vec<String>,
    /// Working directory of the process, if accessible.
    pub working_dir: Option<PathBuf>,
    /// Subset of the process environment.
    pub env: HashMap<String, String>,
    /// Process start time as seconds since the Unix epoch (UTC).
    pub start_time_secs: i64,
}

impl ProcessSnapshot {
    /// Minimal constructor (detect-path fields only).
    pub fn new(
        pid: u32,
        exe_path: Option<PathBuf>,
        cmdline: Vec<String>,
        start_time_secs: i64,
    ) -> Self {
        Self {
            pid,
            ppid: None,
            exe_path,
            cmdline,
            working_dir: None,
            env: HashMap::new(),
            start_time_secs,
        }
    }

    /// Full-context constructor (detect + enrich fields).
    pub fn with_full_context(
        pid: u32,
        ppid: Option<u32>,
        exe_path: Option<PathBuf>,
        cmdline: Vec<String>,
        working_dir: Option<PathBuf>,
        env: HashMap<String, String>,
        start_time_secs: i64,
    ) -> Self {
        Self {
            pid,
            ppid,
            exe_path,
            cmdline,
            working_dir,
            env,
            start_time_secs,
        }
    }
}

/// A process snapshot enriched with engine-specific context.
///
/// `last_event_micros` is `Option<i64>` per BC-2.03.001 PC-4 and AC-004:
/// `None` means no hook events have been received yet. `Some(0)` is the Unix epoch
/// (1970-01-01T00:00:00Z), NOT a sentinel — `0` as a sentinel is forbidden.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct EnrichedSession {
    /// Engine-specific session identifier.
    pub session_id: String,
    /// Harness type identifier.
    pub harness_type: String,
    /// Absolute path to the engine-specific transcript file, if known.
    pub transcript_path: Option<PathBuf>,
    /// Absolute path to the engine-specific config file, if known.
    pub config_path: Option<PathBuf>,
    /// Session lifecycle status.
    pub status: SessionStatus,
    /// Timestamp of the last received hook event, in microseconds since the Unix epoch (UTC).
    /// `None` means no hook events have been received for this session yet.
    pub last_event_micros: Option<i64>,
}

impl EnrichedSession {
    /// Construct an `EnrichedSession`.
    ///
    /// Pass `None` for `last_event_micros` when no hook events have been received yet.
    /// Pass `Some(timestamp_micros)` for an active session with a known last-event time.
    /// Note: `Some(0)` is the Unix epoch (1970-01-01T00:00:00Z), NOT a sentinel — using
    /// `0` as a "no events" sentinel is forbidden per BC-2.03.001 PC-4.
    pub fn new(
        session_id: String,
        harness_type: String,
        transcript_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        status: SessionStatus,
        last_event_micros: Option<i64>,
    ) -> Self {
        Self {
            session_id,
            harness_type,
            transcript_path,
            config_path,
            status,
            last_event_micros,
        }
    }
}

/// Session lifecycle status.
///
/// Canonical 5 variants per SS-engine-module.md v1.1.20 (F-D-01):
/// `Active`, `Idle`, `WaitingOnPermission`, `Stopping`, `Stopped`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionStatus {
    /// Agentic loop running.
    Active,
    /// Waiting for user prompt.
    Idle,
    /// Permission overlay is raised; agentic loop is paused waiting for the human decision.
    WaitingOnPermission,
    /// Graceful shutdown in progress; session is draining in-flight operations.
    Stopping,
    /// Session has ended.
    Stopped,
}

/// The dispatch decision returned by `on_hook`.
///
/// Canonical 3-field struct per SS-engine-module.md v1.1.20 (F-D-02):
/// `decision`, `redirect_url: Option<String>`, `diagnostic: Option<String>`.
/// `DeferUntil` is NOT part of this type (F-D-03 ghost type dropped).
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct HookResponse {
    /// The decision the daemon should act on.
    pub decision: HookDecision,
    /// Optional URL to redirect the Claude Code process to when decision is `Defer`.
    pub redirect_url: Option<String>,
    /// Optional human-readable diagnostic message surfaced in the permission overlay.
    pub diagnostic: Option<String>,
}

impl HookResponse {
    /// Construct a `HookResponse` with the given decision and no redirect or diagnostic.
    pub fn new(decision: HookDecision) -> Self {
        Self {
            decision,
            redirect_url: None,
            diagnostic: None,
        }
    }

    /// Attach a diagnostic message to this response (builder pattern).
    pub fn with_diagnostic(self, diagnostic: impl Into<String>) -> Self {
        Self {
            diagnostic: Some(diagnostic.into()),
            ..self
        }
    }

    /// Attach a redirect URL to this response (builder pattern).
    pub fn with_redirect(self, url: impl Into<String>) -> Self {
        Self {
            redirect_url: Some(url.into()),
            ..self
        }
    }
}

/// The action the daemon takes in response to a hook event.
///
/// Canonical 3 variants per SS-engine-module.md v1.1.20 (F-D-02): `Allow`, `Block`, `Defer`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookDecision {
    /// Proceed; the operation is permitted.
    Allow,
    /// Reject the operation; the daemon MUST NOT proceed.
    Block,
    /// Park the decision; the daemon waits for the permission overlay resolution.
    Defer,
}

/// Error from `metadata()` and `enrich()`.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EngineMetadataError {
    /// Platform home directory could not be resolved.
    #[error("platform home directory unresolvable (BaseDirs::new() returned None)")]
    HomeUnresolvable,
}
