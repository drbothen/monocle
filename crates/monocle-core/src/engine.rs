/// EngineModule trait and supporting types (BC-2.03.001; S-014).
///
/// Tests in `engine_module_surface.rs` (VP-019 AST audit suite) verify structural
/// conformance to BC-2.03.001 and SS-engine-module.md v1.1.27.
///
/// S-033: adds `EngineError`, `SpawnOptions`, `SpawnRecipe`, and `spawn_recipe()` default
/// method per BC-2.03.008 and SS-engine-module-v2-delta.md §spawn_recipe-new-trait-method.
use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};

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

    /// Return the recipe needed to spawn a session under monocle's daemon.
    ///
    /// Default impl returns `Err(EngineError::UnsupportedOperation("spawn_recipe"))`.
    /// Only engines that support monocle-controlled session spawning implement this.
    /// Phase 1: `ClaudeCodeModule` override is added by S-045.
    ///
    /// BC-2.03.008 PC-1: a no-override `EngineModule` MUST return this exact error.
    /// BC-2.03.008 PC-3: `session_error_to_code()` maps `UnsupportedOperation` to
    /// `"spawn_unsupported"` (not `"invalid_request"`) — F-P44-IMP-001.
    fn spawn_recipe(&self, _opts: &SpawnOptions) -> Result<SpawnRecipe, EngineError> {
        Err(EngineError::UnsupportedOperation("spawn_recipe"))
    }
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
///
/// `Serialize + Deserialize` are required for IPC wire format: `EnrichedSession` is
/// embedded in `ServerToClient::SessionListUpdate` and `ServerToClient::InitialState`
/// (S-021 BC-2.05.003). `serde_json::to_vec` is called on the containing `ServerToClient`
/// enum by `monocle-ipc::framing::write_framed`.
///
/// All four display fields (`project_name`, `started_at`, `token_count`, `cost_usd`) are
/// specified in SS-engine-module.md v1.1.27 and BC-2.06.005 PC-2. Phase 1 daemon populates
/// these with zero-value defaults (`None`/`0`) until a richer enrichment story provides real
/// values. The TUI renders `None`/`0` as `"—"` per BC-2.06.005 Invariant 3.
///
/// `#[non_exhaustive]` per ADR-0006. All construction via `new()`.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnrichedSession {
    /// Engine-specific session identifier.
    pub session_id: String,
    /// Harness type identifier (e.g., "claude-code").
    pub harness_type: String,
    /// Absolute path to the engine-specific transcript file, if known.
    pub transcript_path: Option<PathBuf>,
    /// Absolute path to the engine-specific config file, if known.
    pub config_path: Option<PathBuf>,
    /// Session lifecycle status.
    pub status: SessionStatus,
    /// Timestamp of the last received hook event, in microseconds since the Unix epoch (UTC).
    /// `None` means no hook events have been received for this session yet.
    /// `Some(0)` is the Unix epoch, NOT a sentinel — using `0` as "no events" is forbidden.
    pub last_event_micros: Option<i64>,
    /// Human-readable project name derived from the transcript directory name
    /// (the immediate parent directory of `transcript_path`).
    /// `None` when `transcript_path` is unknown or parsing fails.
    /// Phase 1 daemon populates this during `enrich()`; zero-value is `None`.
    pub project_name: Option<String>,
    /// UTC timestamp of the first `SessionStart` hook event for this session.
    /// `None` until the daemon receives the first hook event carrying session-start data.
    /// The TUI computes uptime as `now - started_at` at render time.
    /// Phase 1 daemon defaults to `None`; populated when enrichment reads start time.
    pub started_at: Option<DateTime<Utc>>,
    /// Cumulative input + output token count reported by the harness hook stream.
    /// Defaults to `0` when the daemon has not yet received token-count hook data.
    /// Phase 1: zero is the sentinel-free default (the TUI renders `0` as `"0"` per
    /// BC-2.06.005 PC-2 — `format_token_count(0)` returns `"0"`, not `"—"`).
    pub token_count: u64,
    /// Cumulative cost in USD as reported by the harness hook stream.
    /// `None` when the daemon has not received cost data for this session.
    /// `Some(0.0)` is a valid zero-cost session — not a sentinel.
    pub cost_usd: Option<f64>,

    /// Human-readable display name for the harness engine that owns this session.
    ///
    /// Populated by the daemon from `EngineMetadata::display_name` during session
    /// enrichment (e.g., `"Claude Code"` for the `claude-code` harness).
    ///
    /// Used by the TUI sessions filter (BC-2.06.006 PC-3 OR-match: sessions are
    /// included when `project_name` OR `display_name` fuzzy-matches the query).
    /// Sourced from the daemon's IPC wire copy — NOT from a hardcoded TUI-side lookup
    /// table — so that third-party harness engines not listed in the TUI's map can
    /// also be found by their display name.
    ///
    /// Defaults to `""` (empty) for sessions enriched before this field was added;
    /// the TUI sessions filter treats an empty `display_name` as no additional match surface
    /// (project_name OR-match still applies).
    pub display_name: String,
}

impl EnrichedSession {
    /// Construct an `EnrichedSession`.
    ///
    /// Pass `None` for `last_event_micros` when no hook events have been received yet.
    /// Pass `Some(timestamp_micros)` for an active session with a known last-event time.
    /// Note: `Some(0)` is the Unix epoch (1970-01-01T00:00:00Z), NOT a sentinel — using
    /// `0` as a "no events" sentinel is forbidden per BC-2.03.001 PC-4.
    ///
    /// Pass `None, None, 0, None` for `project_name`, `started_at`, `token_count`, `cost_usd`
    /// in Phase 1 until richer enrichment populates them.
    ///
    /// `display_name` is populated from `EngineMetadata::display_name` (e.g., `"Claude Code"`).
    /// Pass `""` for legacy callers that do not yet have the display name.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        harness_type: String,
        transcript_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        status: SessionStatus,
        last_event_micros: Option<i64>,
        project_name: Option<String>,
        started_at: Option<DateTime<Utc>>,
        token_count: u64,
        cost_usd: Option<f64>,
    ) -> Self {
        Self {
            session_id,
            harness_type,
            transcript_path,
            config_path,
            status,
            last_event_micros,
            project_name,
            started_at,
            token_count,
            cost_usd,
            // Default to empty — callers that do not have a display_name can pass ""
            // explicitly via new_with_display_name; this keeps existing call sites stable.
            display_name: String::new(),
        }
    }

    /// Construct an `EnrichedSession` with an explicit `display_name`.
    ///
    /// BC-2.06.006 PC-3 / ADV Pass-2: the daemon populates `display_name` from
    /// `EngineMetadata::display_name` during enrichment. This constructor is the
    /// canonical path for daemon-side session construction and for tests that verify
    /// display-name filter matching.
    ///
    /// `display_name` is the human-readable engine name (e.g., `"Claude Code"`).
    /// It is used by the TUI sessions filter as an OR-match target alongside `project_name`.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_display_name(
        session_id: String,
        harness_type: String,
        transcript_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        status: SessionStatus,
        last_event_micros: Option<i64>,
        project_name: Option<String>,
        started_at: Option<DateTime<Utc>>,
        token_count: u64,
        cost_usd: Option<f64>,
        display_name: String,
    ) -> Self {
        Self {
            session_id,
            harness_type,
            transcript_path,
            config_path,
            status,
            last_event_micros,
            project_name,
            started_at,
            token_count,
            cost_usd,
            display_name,
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

// ---------------------------------------------------------------------------
// S-033: spawn_recipe() types — BC-2.03.008 + SS-engine-module-v2-delta.md
// ---------------------------------------------------------------------------

/// Errors from `EngineModule::spawn_recipe()`.
///
/// `#[non_exhaustive]` for Phase 3 WASM engine forward-compatibility.
/// The inner match in `session_error_to_code()` MUST include a `_ => "invalid_request"`
/// fallback because this type is cross-crate and non-exhaustive.
///
/// Phase 1 variants: `BinaryNotFound`, `InvalidPath`, `UnsupportedOperation`.
/// The `UnsupportedOperation` arm MUST appear explicitly in `session_error_to_code()` BEFORE
/// the `_ =>` fallback — omitting it is the F-P44-IMP-001 regression pattern.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// The harness binary was not found on PATH (e.g., `which::which("claude")` failed).
    /// Wire code: `"binary_not_found"`.
    #[error("binary not found: {0}")]
    BinaryNotFound(String),

    /// An argument to spawn_recipe() is invalid (e.g., non-UTF-8 or null-byte in path).
    /// Wire code: `"invalid_spawn_arg"`.
    #[error("invalid path argument: {0}")]
    InvalidPath(String),

    /// The EngineModule does not support monocle-controlled spawning.
    /// Default impl of `spawn_recipe()` returns this variant (BC-2.03.008 PC-1).
    /// Wire code: `"spawn_unsupported"` (F-P44-IMP-001).
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(&'static str),
}

/// The spawn recipe produced by an `EngineModule`.
///
/// `SessionManager` uses this to build the `monocle-session-host` command line.
/// Daemon-internal — never transmitted over IPC directly.
/// All fields MUST be set by the implementing module.
///
/// `#[non_exhaustive]` per ADR-0006: field additions (new harness CLI flags) arise from
/// Claude Code version bumps requiring coordinated BC revisions.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpawnRecipe {
    /// Absolute path to the harness binary.
    pub binary: PathBuf,
    /// CLI arguments (e.g., `["--settings", "/tmp/monocle-hooks-abc.json"]`).
    /// The `hooks_settings_path` from `SpawnOptions` MUST be passed here as `--settings`.
    pub args: Vec<String>,
    /// Environment variables to OVERLAY on top of the session-host process's inherited env.
    /// Keys present here override any matching key in the inherited env.
    /// PATH/HOME are preserved from the inherited base env.
    pub env: HashMap<String, String>,
    /// Working directory for the harness child process.
    /// Populated from `SpawnOptions.worktree_root`. NEVER hardcoded to `project_root`.
    pub cwd: PathBuf,
}

impl SpawnRecipe {
    /// ADR-0006 constructor: required because `SpawnRecipe` is `#[non_exhaustive]` and
    /// constructed cross-crate inside `ClaudeCodeModule::spawn_recipe()` in `monocle-runtime`.
    pub fn new(
        binary: PathBuf,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: PathBuf,
    ) -> Self {
        Self {
            binary,
            args,
            env,
            cwd,
        }
    }
}

/// Options passed from the TUI via `ClientToServer::SpawnSession { opts }` to the daemon,
/// and then from the IPC handler to `SessionManager::spawn_session(opts)` and from there
/// to `EngineModule::spawn_recipe(&opts)`.
///
/// I27-001 (Model A): `SpawnOptions` is the IPC wire type. `SpawnRecipe` is daemon-internal.
/// The TUI populates `project_root`, `worktree_root`, `harness_id`, `profile_id`, and
/// `ccr_base_url`. The daemon IPC handler fills `session_id` and `hooks_settings_path` on
/// receipt before calling `SessionManager::spawn_session(opts)`.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpawnOptions {
    /// Project root directory (user-selected in the wizard; used for display grouping).
    pub project_root: PathBuf,
    /// Working directory for the harness child process (resolved git worktree root or
    /// `project_root`).
    pub worktree_root: PathBuf,
    /// Harness identifier selected by the user (e.g., `"claude-code"`, `"codemachine"`).
    pub harness_id: String,
    /// Harness profile ID selected by the user in the SessionCreation wizard.
    pub profile_id: String,
    /// Pre-generated session UUID (filled by daemon IPC handler, never by TUI).
    pub session_id: String,
    /// Path where the daemon has already written the shared hooks-settings.json.
    /// The EngineModule MUST include `"--settings <hooks_settings_path>"` in recipe args.
    pub hooks_settings_path: PathBuf,
    /// If CCR is detected and a base URL is configured, this carries the URL.
    /// The EngineModule MUST inject this as `ANTHROPIC_BASE_URL` in `env` if present.
    pub ccr_base_url: Option<String>,
}

impl SpawnOptions {
    /// TUI-side constructor (ADR-0006): populates the 5 TUI-owned fields.
    /// The two daemon-owned fields (`session_id`, `hooks_settings_path`) are initialized
    /// to placeholder values; the daemon ALWAYS overwrites them via `with_daemon_fields()`
    /// before calling `spawn_session()`.
    pub fn for_spawn_request(
        project_root: PathBuf,
        worktree_root: PathBuf,
        harness_id: String,
        profile_id: String,
        ccr_base_url: Option<String>,
    ) -> Self {
        Self {
            project_root,
            worktree_root,
            harness_id,
            profile_id,
            ccr_base_url,
            session_id: String::new(),
            hooks_settings_path: PathBuf::new(),
        }
    }

    /// Daemon-side consuming builder: fills the two daemon-owned fields.
    /// Called in the IPC handler immediately upon receipt of `ClientToServer::SpawnSession`,
    /// BEFORE passing `SpawnOptions` to `spawn_session()`.
    pub fn with_daemon_fields(self, session_id: String, hooks_settings_path: PathBuf) -> Self {
        Self {
            session_id,
            hooks_settings_path,
            ..self
        }
    }
}
