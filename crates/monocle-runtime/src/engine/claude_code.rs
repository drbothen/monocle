//! `ClaudeCodeModule` — `EngineModule` implementation for Claude Code (S-015).
//!
//! Implements BC-2.03.002 (strict-basename detect), BC-2.03.003 (HomeUnresolvable
//! fail-fast), and BC-2.03.004 (inherent methods: hook_paths, spawn, preflight).
//! Satisfies VP-020, VP-021, VP-022.

use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use monocle_core::engine::{
    EngineMetadata, EngineMetadataError, EngineModule, EnrichedSession, HookDecision, HookResponse,
    ProcessSnapshot,
};
use monocle_core::hook_events::HookEvent;
use monocle_core::hook_events::HookType;

// ---------------------------------------------------------------------------
// Supporting types for inherent methods (BC-2.03.004)
// SS-engine-module.md v1.1.20 §Struct-level inherent operations (lines 740-902)
// ---------------------------------------------------------------------------

/// Arguments for spawning a new Claude Code session.
///
/// `#[non_exhaustive]` prevents external struct-literal construction (E0639).
/// Use `SpawnArgs::new(project_root)` + builder methods.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SpawnArgs {
    /// The project root directory for this session.
    pub project_root: PathBuf,
    /// Optional worktree path override (claude-squad isolation pattern).
    /// `None` means Claude Code opens in `project_root` directly.
    pub worktree: Option<PathBuf>,
    /// Environment variable overrides for the spawned subprocess.
    /// Empty map means no overrides; Claude Code inherits the daemon's environment.
    pub env_overrides: HashMap<String, String>,
}

impl SpawnArgs {
    /// Construct a `SpawnArgs` for a session rooted at `project_root`.
    ///
    /// `worktree` defaults to `None`. `env_overrides` defaults to empty.
    /// Use `with_worktree` and `with_env_override` for optional fields.
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            worktree: None,
            env_overrides: HashMap::new(),
        }
    }

    /// Set the worktree path override for this spawn.
    pub fn with_worktree(mut self, worktree: PathBuf) -> Self {
        self.worktree = Some(worktree);
        self
    }

    /// Add a single environment variable override for the spawned subprocess.
    pub fn with_env_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_overrides.insert(key.into(), value.into());
        self
    }
}

/// Handle to a successfully spawned Claude Code session.
///
/// `#[non_exhaustive]` prevents external struct-literal construction (E0639).
/// Use `SessionHandle::new(pid, session_id, hook_base_url)`.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SessionHandle {
    /// PID of the spawned `claude` subprocess.
    pub pid: u32,
    /// Session UUID assigned by Claude Code itself.
    pub session_id: String,
    /// The hook base URL this session's hook scripts POST to.
    pub hook_base_url: String,
}

impl SessionHandle {
    /// Construct a `SessionHandle` from the spawn result.
    ///
    /// All three fields are required and come from the spawn result.
    /// Field order matches struct declaration order.
    pub fn new(pid: u32, session_id: String, hook_base_url: String) -> Self {
        Self {
            pid,
            session_id,
            hook_base_url,
        }
    }
}

/// Version information returned by a successful preflight check.
///
/// `#[non_exhaustive]` prevents external struct-literal construction (E0639).
/// Use `EngineVersion::new(version, binary_path)`.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct EngineVersion {
    /// Version string from `claude --version` (e.g., `"2.0.0"`).
    pub version: String,
    /// Absolute path to the `claude` binary as resolved by `which`.
    pub binary_path: PathBuf,
}

impl EngineVersion {
    /// Construct an `EngineVersion` from a successful preflight result.
    ///
    /// Both fields are required and come from `claude --version` + `which claude`.
    /// Field order matches struct declaration order.
    pub fn new(version: String, binary_path: PathBuf) -> Self {
        Self {
            version,
            binary_path,
        }
    }
}

/// Error from `ClaudeCodeModule::spawn` (BC-2.03.004 PC-2).
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    /// `claude` binary was not found on PATH.
    #[error("claude binary not found: {0}")]
    BinaryNotFound(String),
    /// OS-level spawn failed.
    #[error("spawn failed: {0}")]
    SpawnFailed(String),
    /// Worktree setup failed before spawn could proceed.
    #[error("worktree setup failed: {0}")]
    WorktreeFailed(String),
}

/// Error from `ClaudeCodeModule::preflight` (BC-2.03.004 PC-3).
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PreflightError {
    /// `claude` binary was not found on PATH.
    #[error("claude binary not found on PATH: {binary}")]
    BinaryNotFound {
        /// The name that was searched for on PATH.
        binary: String,
    },
    /// Installed Claude Code version is below the minimum supported floor.
    #[error("claude version {found} is below minimum supported {minimum}")]
    VersionTooOld {
        /// The version string returned by `claude --version`.
        found: String,
        /// The minimum version required by this daemon build.
        minimum: String,
    },
    /// Hook base URL passed to `ClaudeCodeModule::new` is syntactically invalid.
    /// Deferred validation surfaces here (construction is infallible per BC-2.03.002 PC-2).
    #[error("hook base URL is invalid: {url}: {reason}")]
    InvalidHookUrl {
        /// The invalid URL string.
        url: String,
        /// Human-readable parse error.
        reason: String,
    },
    /// Preflight check failed for a reason not covered by the above variants.
    #[error("preflight check failed: {reason}")]
    Failed {
        /// Human-readable failure description.
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Home directory availability guard
// ---------------------------------------------------------------------------

/// Returns `true` iff the process environment contains a non-empty home-directory
/// variable that `directories::BaseDirs::new()` would trust on this platform.
///
/// On macOS, `dirs-sys` has a `getpwuid_r` fallback that resolves a home directory
/// even when `HOME` is not set — but that path is not under the user's env control
/// and cannot be trusted in headless / container deployments (BC-2.03.003 Invariant 3;
/// same guard applied in `lifecycle::resolve_runtime_dir` for S-006 BC-2.01.005 PC-2d).
///
/// The four variables below are exactly the ones unset by the
/// `HOME_ENV_VARS_UNSET` constant in the `engine_module_home_unresolvable` test suite.
fn home_env_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows: USERPROFILE, or HOMEDRIVE+HOMEPATH together.
        // Linux/Unix: HOME.
        let home = std::env::var_os("HOME")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let userprofile = std::env::var_os("USERPROFILE")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let homedrive = std::env::var_os("HOMEDRIVE")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let homepath = std::env::var_os("HOMEPATH")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        home || userprofile || (homedrive && homepath)
    }
}

// ---------------------------------------------------------------------------
// ClaudeCodeModule
// ---------------------------------------------------------------------------

/// Claude Code engine module.
///
/// Implements `EngineModule` for the Claude Code harness. This is the Phase 1
/// canonical reference implementation described in SS-engine-module.md v1.1.20
/// §Phase 1 Implementation: ClaudeCodeModule (lines 545-657).
///
/// # Detection strategy
///
/// `detect()` uses **strict basename matching** on `proc.exe_path`: only the exact
/// basenames `"claude"` and `"claude.js"` return `true`. All look-alike processes
/// (`claude-squad`, `claudio`, `claude-code`, `Claude`, etc.) return `false`.
/// This is a pure function — no I/O, no environment lookups (BC-2.03.001 PC-6; DI-006).
///
/// # Home directory resolution
///
/// `metadata()` and `enrich()` call `directories::BaseDirs::new()`. If it returns
/// `None` (broken passwd, systemd `User=` unit without `Environment=HOME`,
/// hardened sandbox), both methods return `Err(EngineMetadataError::HomeUnresolvable)`
/// immediately — no default path substitution (BC-2.03.003 PC-1; BC-2.03.001 PC-5).
///
/// # Hook paths
///
/// `hook_paths()` returns the canonical 5-entry mapping per BC-2.03.004 PC-1.
/// `PostToolUse` is intentionally absent (JC-2 / BC-2.03.004 invariant 1).
pub struct ClaudeCodeModule {
    // Used by spawn() and preflight() — both are todo!() stubs in Phase 1.
    #[allow(dead_code)]
    hook_base_url: String,
}

impl ClaudeCodeModule {
    /// Infallible constructor — URL validation is deferred to `preflight()`.
    ///
    /// `hook_base_url` is the base URL where Claude Code hook scripts POST
    /// (e.g., `"http://127.0.0.1:7891"`). Path segments are appended by `hook_paths()`.
    /// The URL is NOT validated here per BC-2.03.002 PC-2.
    pub fn new(hook_base_url: String) -> Self {
        Self { hook_base_url }
    }

    /// Hook protocol path mapping: hook type to URL path segment.
    ///
    /// Returns exactly 5 entries per BC-2.03.004 PC-1. `PostToolUse` is intentionally
    /// absent (JC-2 / BC-2.03.004 invariant 1). VP-022 asserts `len() == 5`.
    pub fn hook_paths(&self) -> HashMap<HookType, String> {
        use HookType::*;
        [
            (SessionStart, "/hooks/session-start".into()),
            (UserPromptSubmit, "/hooks/prompt-submit".into()),
            (PreToolUse, "/hooks/pre-tool-use".into()),
            (Notification, "/hooks/notification".into()),
            (Stop, "/hooks/stop".into()),
        ]
        .into()
    }

    /// Spawn a new Claude Code session (BC-2.03.004 PC-2).
    ///
    /// Phase 1 implementation is `todo!()` — the signature is binding.
    /// EC-038: calling `spawn()` in Phase 1 panics with `todo!()` — intentional.
    #[allow(clippy::todo)]
    pub async fn spawn(&self, _args: SpawnArgs) -> Result<SessionHandle, SpawnError> {
        todo!("S-015: ClaudeCodeModule::spawn — Phase 1 stub")
    }

    /// Validate that the Claude Code binary is available and version-compatible
    /// (BC-2.03.004 PC-3).
    ///
    /// Phase 1 implementation is `todo!()` — the signature is binding.
    /// EC-039: calling `preflight()` in Phase 1 panics with `todo!()` — intentional.
    /// Phase 3 story replaces the stub with `which claude` + `claude --version` checks.
    #[allow(clippy::todo)]
    pub async fn preflight(&self) -> Result<EngineVersion, PreflightError> {
        todo!("S-015: ClaudeCodeModule::preflight — Phase 1 stub")
    }
}

#[async_trait]
impl EngineModule for ClaudeCodeModule {
    /// Returns `"claude-code"` — stable engine identifier (BC-2.03.002 PC-3; AC-004).
    fn id(&self) -> &'static str {
        "claude-code"
    }

    /// Static metadata describing the Claude Code engine.
    ///
    /// Returns `Err(HomeUnresolvable)` when `BaseDirs::new()` returns `None` —
    /// fail-fast with no default path substitution (BC-2.03.003 PC-1).
    /// Logs `E-ENG-001` on `HomeUnresolvable` per BC-2.03.003 PC-2.
    fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError> {
        // On macOS, `dirs-sys` has a `getpwuid_r` fallback that resolves a home
        // directory even when HOME is not set — but in that case the path is not
        // under the user's env control and cannot be trusted in a headless/container
        // deployment (same guard used in lifecycle::resolve_runtime_dir for S-006).
        // We check HOME (+ Windows equivalents) before delegating to BaseDirs::new()
        // so that temp_env::async_with_vars can reliably trigger HomeUnresolvable.
        if !home_env_available() {
            tracing::error!(
                "E-ENG-001: platform home directory unresolvable (BaseDirs::new() returned None)"
            );
            return Err(EngineMetadataError::HomeUnresolvable);
        }

        let base_dirs = directories::BaseDirs::new().ok_or_else(|| {
            tracing::error!(
                "E-ENG-001: platform home directory unresolvable (BaseDirs::new() returned None)"
            );
            EngineMetadataError::HomeUnresolvable
        })?;

        // Claude Code stores its config under ~/.claude/
        let config_paths = vec![base_dirs.home_dir().join(".claude").join("settings.json")];

        Ok(EngineMetadata::new(
            "Claude Code",
            '\u{25C6}', // ◆
            config_paths,
            1,
        ))
    }

    /// Strict basename detection — returns `true` iff `exe_path.file_name()` is
    /// exactly `"claude"` or `"claude.js"` (BC-2.03.002 PC-4; AC-001).
    ///
    /// Returns `false` for `None` exe_path (AC-002; BC-2.03.002 PC-5).
    /// Pure function — no I/O, no env lookups, no shared-state mutation (AC-010;
    /// BC-2.03.001 PC-6; DI-006).
    fn detect(&self, proc: &ProcessSnapshot) -> bool {
        proc.exe_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|name| name == "claude" || name == "claude.js")
            .unwrap_or(false)
    }

    /// Enrich a detected process snapshot with Claude Code-specific context.
    ///
    /// Returns `Err(HomeUnresolvable)` when `BaseDirs::new()` returns `None` —
    /// fail-fast with no default path substitution (BC-2.03.003 PC-1; AC-005).
    /// Logs `E-ENG-001` on `HomeUnresolvable` per BC-2.03.003 PC-2 (AC-006).
    async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError> {
        // Same macOS getpwuid_r guard as in metadata() — see inline comment there.
        if !home_env_available() {
            tracing::error!(
                "E-ENG-001: platform home directory unresolvable (BaseDirs::new() returned None)"
            );
            return Err(EngineMetadataError::HomeUnresolvable);
        }

        let base_dirs = directories::BaseDirs::new().ok_or_else(|| {
            tracing::error!(
                "E-ENG-001: platform home directory unresolvable (BaseDirs::new() returned None)"
            );
            EngineMetadataError::HomeUnresolvable
        })?;

        // Claude Code stores transcripts under ~/.claude/projects/<cwd-hash>/
        // and config under ~/.claude/settings.json.
        // Working dir from the process determines the project context.
        let claude_dir = base_dirs.home_dir().join(".claude");
        let config_path = Some(claude_dir.join("settings.json"));

        // Derive a session_id from the process pid — no hook events have arrived yet.
        let session_id = format!("claude-{}", proc.pid);

        Ok(EnrichedSession::new(
            session_id,
            "claude-code".to_string(),
            None, // transcript_path: no transcript path until hook events arrive
            config_path,
            monocle_core::engine::SessionStatus::Idle,
            None, // last_event_micros: no hook events received yet
        ))
    }

    /// Process an inbound hook event.
    ///
    /// Phase 1: returns `HookResponse::new(HookDecision::Allow)` for ALL variants,
    /// including unknown future variants via the wildcard arm (EC-031; AC-010; AC-011;
    /// BC-2.03.001 EC-031). Per-variant dispatch is Phase 2+ scope (permission-overlay
    /// story). SS-engine-module.md line 650: Phase 1 unconditionally returns `Allow`.
    async fn on_hook(&self, event: HookEvent) -> HookResponse {
        match event {
            HookEvent::SessionStart(_)
            | HookEvent::UserPromptSubmit(_)
            | HookEvent::PreToolUse(_)
            | HookEvent::Notification(_)
            | HookEvent::Stop(_) => HookResponse::new(HookDecision::Allow),
            // Wildcard arm required by #[non_exhaustive] — fail-open for Phase 4 variants.
            _ => HookResponse::new(HookDecision::Allow),
        }
    }
}
