/// EngineModule trait and supporting types (stub — implementation by S-014).
///
/// This module contains compilable stubs. Tests in `engine_module_surface.rs` will
/// catch structural deviations from BC-2.03.001 and SS-engine-module.md v1.1.20.
use std::collections::HashMap;
use std::path::PathBuf;

use crate::hook_events::HookEvent;

/// Stub trait — missing production-grade rustdoc.
///
/// TODO-STUB: Add the canonical rationale text explaining why the `async_trait` macro
/// is required. The text must appear verbatim in the rustdoc per AC-007 / BC-2.03.001 INV-3.
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
        Self { display_name, icon, config_paths, hook_schema_version }
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
        Self { pid, ppid, exe_path, cmdline, working_dir, env, start_time_secs }
    }
}

/// A process snapshot enriched with engine-specific context.
///
/// STUB: `last_event_micros` is `i64` — canonical is `Option<i64>` (AC-004 / probe 19.e).
/// The VP-019 AST audit test will detect and fail on this field type.
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
    /// STUB: bare `i64` — must be `Option<i64>` per BC-2.03.001 PC-4 and AC-004.
    pub last_event_micros: i64,
}

impl EnrichedSession {
    /// Constructor stub.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        harness_type: String,
        transcript_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        status: SessionStatus,
        last_event_micros: i64,
    ) -> Self {
        Self { session_id, harness_type, transcript_path, config_path, status, last_event_micros }
    }
}

/// Session lifecycle status.
///
/// STUB: only 3 variants. Canonical is 5: Active, Idle, WaitingOnPermission, Stopping, Stopped
/// per SS-engine-module.md v1.1.20 (F-D-01). AC-003 test will detect missing variants.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionStatus {
    /// Agentic loop running.
    Active,
    /// Waiting for user prompt.
    Idle,
    /// Session has ended.
    Stopped,
}

/// The dispatch decision returned by `on_hook`.
///
/// STUB: missing `redirect_url` and `diagnostic` fields. Canonical per SS-engine-module.md
/// v1.1.20 (F-D-02): `decision`, `redirect_url: Option<String>`, `diagnostic: Option<String>`.
/// AC-003 test will detect absent fields.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct HookResponse {
    /// The decision the daemon should act on.
    pub decision: HookDecision,
}

impl HookResponse {
    /// Construct a `HookResponse` with the given decision.
    pub fn new(decision: HookDecision) -> Self {
        Self { decision }
    }

    /// Stub — `with_diagnostic` builder (no-op until fields are added).
    pub fn with_diagnostic(self, _diagnostic: impl Into<String>) -> Self {
        self
    }

    /// Stub — `with_redirect` builder (no-op until fields are added).
    pub fn with_redirect(self, _url: impl Into<String>) -> Self {
        self
    }
}

/// The action the daemon takes in response to a hook event.
///
/// STUB: missing `Block` variant. Canonical: `Allow, Block, Defer`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookDecision {
    /// Proceed.
    Allow,
    /// Park the decision.
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
