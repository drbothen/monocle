---
document_type: architecture-section
level: L3
section: "engine-module"
slug: "engine-module-trait-stability"
subsystem: "core"
version: "1.1.6"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-13T20:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md
input-hash: "[live-state]"
traces_to: "vision authority restoration per human Q-15-1; round-14 adversary N1/N2; SS-forward-compatibility lines 95-97 veto honored; F-FC-I003 adversary finding; vision §EngineModule lines 111-128; brief v1.4.7 §Harness plane; v1.1.1 round-16 fixes: N16-1 dirs→directories::ProjectDirs; N16-2 ClaudeCodeModule::new; N16-3 EngineMetadata claim clarified; N16-4 exe_path+ppid in ProcessSnapshot; v1.1.2 round-19 fixes: F-R18-1 ProjectDirs→BaseDirs::home_dir().join(.claude); F-R18-2 ClaudeCodeModule::new rustdoc; F-R18-4 BC-ENGINE-002 exe_path=None wording; v1.1.3 round-20 fixes: F-R20-1 metadata/enrich Result<_,EngineMetadataError> typed error; F-R20-3 url-crate rustdoc removed; v1.1.4 round-22 fixes: F-R22-1/2 vision-verbatim vs vision-spirit-aligned provenance precision; F-R22-3 BC-ENGINE-002-ERR HomeUnresolvable error-path test spec with temp-env isolation; v1.1.5 round-23 micro-fix: BC-ENGINE-002-ERR added to Phase 1 PRD BC Pre-Staging table (3→4 engine BCs); v1.1.6 round-24 fixes: F-R24-adv-1 BC-ENGINE-002-ERR enrich() half split to async_with_vars (temp-env async_closure feature; ^0.3 pin); F-R24-adv-3 env-var unset list corrected to HOME+USERPROFILE+HOMEDRIVE+HOMEPATH (removed irrelevant XDG_* entries)"
project: monocle
---

# Architecture: EngineModule Trait Stability (SS-engine-module)

## [Section Content]

## §Purpose

This artifact locks the Phase 1 stability contract for the `EngineModule` trait in
`monocle-core`. The `EngineModule` trait is the Harness-plane abstraction: every AI
coding harness (Claude Code in Phase 1, CodeMachine in Phase 4, third-party plugins
in Phase 3+) implements this trait to integrate with the monocle daemon.

Analogous to `FactoryAdapter` (SS-core-types-and-abi.md §FactoryAdapter Trait),
`EngineModule` is defined in `monocle-core` so that `monocle-runtime` and
`monocle-tui` can depend on it without a cross-crate cycle. Phase 3 promotes the
trait to a WASM ABI via `monocle-plugin-sdk`; the Phase 1 static built-in
implementation is `ClaudeCodeModule`.

**Sealing policy:** `EngineModule` and `FactoryAdapter` are NOT sealed. Per
SS-forward-compatibility.md lines 95–97: "Do not apply the Sealed pattern to
`EngineModule` or `FactoryAdapter`." These traits exist to be implemented by
third-party code (that is their purpose); sealing would defeat Phase 3 plugin SDK
extensibility. The sealed-trait pattern applies only to internal traits that are
`pub` for technical reasons but must not be implemented by downstream code.
`EngineModule` is NOT in that category.

---

## §EngineModule Trait Signature

The signature below is the authoritative Phase 1 contract. The five trait methods have
different provenance with respect to `domain-monocle-vision-synthesis.md` §EngineModule
lines 111–128:

- **Vision-verbatim** (`id`, `detect`, `on_hook`): Signatures match the vision sketch
  exactly. `id() -> &'static str`; `detect(&self, proc: &ProcessSnapshot) -> bool`;
  `on_hook(&self, event: HookEvent) -> HookResponse`. No deviation from the vision text.

- **Vision-spirit-aligned** (`metadata`, `enrich`): The vision sketch declares these
  infallible (`-> EngineMetadata` and `-> EnrichedSession`). Phase 1 wraps both return
  types in `Result<_, EngineMetadataError>` to honour CLAUDE.md SOUL #4 (no silent
  fallback for the unresolvable platform home directory case — see §EngineMetadataError).
  The `Ok` variant recovers the vision's return shape exactly; the `Err` variant adds the
  `HomeUnresolvable` failure path that the vision sketch left implicit. The vision is
  non-authoritative for this surface per CLAUDE.md §Architectural Authority ("the LATER,
  MORE-SPECIFIC artifact wins"); SS-engine-module.md is both later and more specific.
  Implementers MUST use the Result forms defined here, not the infallible vision sketch.

`EngineMetadata` is additionally a vision-spirit-aligned elaboration at the struct level:
`config_paths: Vec<PathBuf>` supports multi-path Claude Code config (e.g.,
`~/.claude/CLAUDE.md` plus a per-project `.claude/CLAUDE.md`); `hook_schema_version: u32`
enables Phase 4 federation peer-version negotiation. Both fields are forward-compatible
elaborations of vision's single-path/string-schema-name fields; downstream code that
needs vision's exact shape can call `.first()` on the Vec and format the u32 as a string.

The trait carries no sealed bound (vision-verbatim; see §Purpose).

```rust
// monocle-core/src/engine.rs

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Implemented by each AI coding harness adapter.
///
/// Phase 1 ships one built-in: `ClaudeCodeModule` (statically compiled into
/// `monocle-runtime`). Phase 4 adds `CodeMachineModule`. Phase 3+ allows
/// third-party WASM plugins implementing this trait via `monocle-plugin-sdk`.
///
/// The trait is OPEN — third-party crates may implement it. This is intentional:
/// it is the mechanism by which the Phase 3 plugin SDK exposes harness extensibility.
/// See SS-forward-compatibility.md §Analysis — Sealed trait (lines 95–97).
#[async_trait::async_trait]
pub trait EngineModule: Send + Sync + 'static {
    /// Stable identifier for this engine (e.g., "claude-code", "codemachine").
    ///
    /// MUST be stable across restarts — used as a session-roster key.
    fn id(&self) -> &'static str;

    /// Static metadata describing this engine for UI display + config.
    ///
    /// Returns `Err(EngineMetadataError::HomeUnresolvable)` when the platform
    /// home directory cannot be resolved (e.g., `$HOME` unset in a systemd
    /// `User=` unit, broken passwd entry, or sandboxed runtime). Callers MUST
    /// propagate this error rather than substituting a default path; daemon
    /// initialization MUST fail fast with a diagnostic message rather than
    /// operating on a relative path that downstream code treats as absolute
    /// (silent-failure violation per CLAUDE.md SOUL #4).
    fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError>;

    /// Detect whether a running process matches this engine's signature.
    /// Sync because process inspection is OS-level cheap; called per-process during scan.
    fn detect(&self, proc: &ProcessSnapshot) -> bool;

    /// Enrich a detected process snapshot with engine-specific context
    /// (session ID derivation, transcript path resolution, harness config introspection).
    ///
    /// Returns `Err(EngineMetadataError::HomeUnresolvable)` when the platform
    /// home directory cannot be resolved. The daemon MUST surface this as a
    /// session-enrichment failure rather than substituting a relative path.
    async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>;

    /// Process an inbound hook event, returning the dispatch decision.
    async fn on_hook(&self, event: HookEvent) -> HookResponse;
}
```

The `HookEvent` type is defined in `monocle-core/src/hook_events.rs` as documented in
SS-core-types-and-abi.md §Non-Exhaustive Inner Structs. Reference it; do not re-declare.

---

## §Supporting Types

```rust
// monocle-core/src/engine.rs (continued)

/// Human-readable harness metadata surfaced in the Sessions panel.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct EngineMetadata {
    /// Display name shown in the Sessions panel header (e.g., "Claude Code").
    pub display_name: &'static str,
    /// Single char icon shown next to session entries (e.g., '●').
    pub icon: char,
    /// Canonical config file locations for this engine.
    /// Multiple entries are allowed (e.g., both `~/.claude/` and `~/.claude.json`).
    pub config_paths: Vec<PathBuf>,
    /// Version of the hook protocol this engine speaks.
    /// Used by the DTU validator to select the correct clone fidelity level.
    pub hook_schema_version: u32,
}

/// A snapshot of a running process, captured by OS-level process enumeration.
///
/// `detect` receives one of these per observed process on each scan cycle.
/// All fields are cheap to populate (no I/O beyond a single `/proc` stat read
/// or equivalent on non-Linux platforms).
///
/// Detection rule: `ClaudeCodeModule::detect` performs a STRICT basename match
/// on `exe_path` (NOT `cmdline[0]`). This avoids false positives from processes
/// named `claude-squad`, `claudio`, or `claude-code-router` that share a prefix
/// with the `claude` binary. `cmdline` is retained for engine-specific
/// environment-aware logic in `enrich()` (e.g., reading `CLAUDE_SESSION_ID`).
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    /// Process ID of the running process.
    pub pid: u32,
    /// Parent process ID, if available.
    /// Populated from `/proc/<pid>/status` (Linux) or platform-equivalent.
    /// Used for process-tree analysis (e.g., detecting claude launched inside tmux).
    pub ppid: Option<u32>,
    /// Resolved binary path via `/proc/<pid>/exe` readlink (Linux) or
    /// platform-equivalent (`sysctl KERN_PROC_PATHNAME` on macOS,
    /// `GetModuleFileNameEx` on Windows).
    ///
    /// This is the canonical field for `detect()`. Strict basename match on
    /// this field is required; `cmdline[0]` is unreliable (may be a wrapper script
    /// or symlink with a different name). `None` if the process has exited before
    /// the daemon could resolve the path.
    pub exe_path: Option<PathBuf>,
    /// Full command line including argv[0] and all arguments.
    /// Retained for `enrich()` use (e.g., reading `CLAUDE_SESSION_ID` from env).
    /// MUST NOT be used as the primary detection signal — use `exe_path` instead.
    pub cmdline: Vec<String>,
    /// Working directory of the process, if accessible.
    pub working_dir: Option<PathBuf>,
    /// Subset of the process environment: only variables matching
    /// engine-specific prefix patterns (e.g., `CLAUDE_*`, `ANTHROPIC_*`).
    /// The daemon applies a prefix allowlist before populating this field;
    /// the full environment is never copied.
    pub env: HashMap<String, String>,
    /// Process start time as seconds since the Unix epoch (UTC).
    pub start_time_secs: i64,
}

/// A process snapshot enriched with engine-specific context.
///
/// Returned by `EngineModule::enrich`. May perform I/O (transcript path
/// resolution, harness config file reads) — it runs off the hot path.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct EnrichedSession {
    /// Engine-specific session identifier (e.g., Claude Code's own session UUID).
    pub session_id: String,
    /// Harness type identifier; MUST equal `EngineModule::id()` for the
    /// module that produced this enriched session.
    pub harness_type: String,
    /// Absolute path to the engine-specific transcript file, if known.
    pub transcript_path: Option<PathBuf>,
    /// Absolute path to the engine-specific config file in use for this session.
    pub config_path: Option<PathBuf>,
    /// Session lifecycle status.
    pub status: SessionStatus,
    /// Timestamp of the most recent hook event for this session,
    /// as microseconds since the Unix epoch (UTC).
    pub last_event_micros: i64,
}

/// Session lifecycle status as observed by the engine module.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionStatus {
    /// Agentic loop running; tool calls in progress.
    Active,
    /// No agentic activity; waiting for user prompt.
    Idle,
    /// Hook response pending; agentic loop paused awaiting user decision.
    WaitingOnPermission,
    /// Graceful shutdown initiated; final `Stop` hook fired.
    Stopping,
    /// Session has ended; no further hooks expected.
    Stopped,
}

/// The dispatch decision returned by `EngineModule::on_hook`.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct HookResponse {
    /// The decision the daemon should act on.
    pub decision: HookDecision,
    /// If the engine requests hook redirect, the target URL.
    /// Used in Phase 4 federation scenarios where the hook should be
    /// forwarded to a peer daemon. `None` in Phase 1.
    pub redirect_url: Option<String>,
    /// Human-readable explanation for status bar display or debug logging.
    /// May be surfaced in the TUI status bar when the user has the session
    /// detail panel open.
    pub diagnostic: Option<String>,
}

/// The action the daemon takes in response to a hook event.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum HookDecision {
    /// Proceed; Claude Code's hook receives a passing response.
    Allow,
    /// Abort; Claude Code's hook receives a failing response.
    Deny {
        /// Human-readable reason surfaced to the user in the TUI overlay.
        reason: String,
    },
    /// Park the hook response until a future condition is met.
    Defer {
        until: DeferUntil,
    },
    /// The engine rewrote the event before dispatch; the daemon should
    /// use the modified event for subsequent processing.
    Modify {
        /// The rewritten event. Phase 1: unused (ClaudeCodeModule never rewrites).
        event: HookEvent,
    },
}

/// Specifies when a deferred hook response should be resolved.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum DeferUntil {
    /// Park until the user acts via the TUI permission overlay.
    UserDecision,
    /// Park until the next hook event arrives for this session.
    NextHook,
    /// Park until the specified duration elapses (daemon-side timeout).
    Timeout(Duration),
}
```

---

## §Phase 1 Implementation: `ClaudeCodeModule`

Phase 1 ships exactly one `EngineModule` implementation: `ClaudeCodeModule`.
It is defined in `monocle-runtime` (NOT `monocle-core`), because it depends on
runtime facilities (`tokio`, process spawning, filesystem I/O) that are not part
of the zero-dependency `monocle-core` crate.

### Trait implementation

```rust
// monocle-runtime/src/engine/claude_code.rs

use monocle_core::engine::{
    DeferUntil, EnrichedSession, EngineMetadata, EngineModule,
    HookDecision, HookResponse, ProcessSnapshot, SessionStatus,
};
use monocle_core::hook_events::HookEvent;

/// Phase 1 built-in EngineModule for Claude Code harness integration.
///
/// Detection: session processes are identified by the `claude` binary name
/// appearing in the process cmdline. Enrichment: resolves the session UUID
/// from the CLAUDE_SESSION_ID environment variable captured in the process
/// snapshot, and constructs the transcript path from the standard
/// `~/.claude/projects/<cwd-hash>/` layout.
pub struct ClaudeCodeModule {
    /// Hook base URL where this daemon is listening (set at daemon start).
    hook_base_url: String,
}

#[async_trait::async_trait]
impl EngineModule for ClaudeCodeModule {
    fn id(&self) -> &'static str {
        "claude-code"
    }

    fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError> {
        // Claude Code is XDG-non-conforming: it uses `~/.claude/` on every platform
        // (Linux, macOS, Windows), NOT XDG-conforming paths such as
        // `~/.config/claude-code/` or `~/Library/Application Support/...`.
        // `directories::BaseDirs::home_dir()` (pinned at `directories 6` in SS-deps)
        // provides the platform home directory without XDG path transformation,
        // which is exactly the right primitive here. `ProjectDirs` is wrong for this
        // use case because it applies XDG path transforms.
        //
        // `BaseDirs::new()` returns `None` when the platform home directory is
        // unresolvable (e.g., `$HOME` unset in a systemd `User=` unit). We MUST
        // fail fast with a typed error rather than substituting a relative path:
        // downstream callers treat config_paths entries as absolute, and a relative
        // `.claude` path would silently point to the wrong directory (SOUL #4).
        let base_dirs = directories::BaseDirs::new()
            .ok_or(EngineMetadataError::HomeUnresolvable)?;
        let claude_config_root = base_dirs.home_dir().join(".claude");
        Ok(EngineMetadata {
            display_name: "Claude Code",
            icon: '●',
            config_paths: vec![
                claude_config_root.clone(),
                claude_config_root.with_extension("json"), // ~/.claude.json
            ],
            hook_schema_version: 1,
        })
    }

    fn detect(&self, proc: &ProcessSnapshot) -> bool {
        // Strict basename match on the RESOLVED exe path (not cmdline[0]).
        // This avoids false positives from `claude-squad`, `claudio`,
        // `claude-code-router`, and other binaries that share a name prefix.
        proc.exe_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|name| name == "claude" || name == "claude.js")
            .unwrap_or(false)
    }

    async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError> {
        let session_id = proc
            .env
            .get("CLAUDE_SESSION_ID")
            .cloned()
            .unwrap_or_else(|| format!("pid-{}", proc.pid));

        // Resolve Claude Code's config root via `directories::BaseDirs` (no `dirs` crate).
        // Claude Code uses `~/.claude/` on every platform; `BaseDirs::home_dir()`
        // gives the platform home directory without XDG path transforms.
        //
        // `BaseDirs::new()` returns `None` when the platform home directory is
        // unresolvable. We MUST fail fast with a typed error: the paths returned in
        // `EnrichedSession` (transcript_path, config_path) are treated as absolute by
        // downstream callers (transcript watcher, DTU validator, TUI display). A
        // relative fallback would silently misdirect all path-dependent operations.
        let base_dirs = directories::BaseDirs::new()
            .ok_or(EngineMetadataError::HomeUnresolvable)?;
        let claude_config_root = base_dirs.home_dir().join(".claude");

        let transcript_path = proc.working_dir.as_ref().map(|cwd| {
            // Standard Claude Code transcript layout:
            // ~/.claude/projects/<cwd-sha256-hex>/
            // Phase 1: placeholder path; full SHA-256 derivation in Phase 1 story.
            claude_config_root
                .join("projects")
                .join(format!("{}", cwd.display()))
        });

        Ok(EnrichedSession {
            session_id,
            harness_type: self.id().to_string(),
            transcript_path,
            config_path: Some(claude_config_root),
            status: SessionStatus::Active,
            last_event_micros: 0, // updated by on_hook
        })
    }

    async fn on_hook(&self, _event: HookEvent) -> HookResponse {
        // Phase 1: default fail-open policy per BC-HOOK-018.
        // Full permission-overlay dispatch implemented in Phase 1 story
        // for monocle-runtime hook_handler.
        HookResponse {
            decision: HookDecision::Allow,
            redirect_url: None,
            diagnostic: None,
        }
    }
}
```

### Struct-level inherent operations (NOT trait methods)

`ClaudeCodeModule` exposes engine-specific operational methods as inherent (struct)
methods, not as part of the `EngineModule` trait. The trait defines the
engine-agnostic observation and hook-dispatch surface. Operational concerns that are
specific to the Claude Code integration — hook path routing, subprocess spawning, and
binary preflight — belong on the struct.

The daemon dispatches to these methods by holding a concrete `ClaudeCodeModule`
where the concrete type is known (Phase 1), and via downcasting from `dyn EngineModule`
in Phase 3 federation scenarios where the concrete type must be recovered (rare).
`ABI_VERSION` is read as `monocle_core::MONOCLE_ABI_VERSION` directly by plugin
loaders; no `abi_version` method on any trait is necessary.

```rust
// monocle-runtime/src/engine/claude_code.rs (continued)

use monocle_core::hook_events::HookType;
use std::collections::HashMap;

impl ClaudeCodeModule {
    /// Construct a new `ClaudeCodeModule` with the given hook base URL.
    ///
    /// Called once at daemon startup after the HTTP listener is bound.
    /// `hook_base_url` is the base URL where Claude Code hook scripts POST
    /// (e.g., `"http://127.0.0.1:7891"`). The path segments are appended by
    /// `hook_paths()`.
    ///
    /// # Validation
    ///
    /// `hook_base_url` is **NOT validated as a URL** at construction time —
    /// construction is infallible. The URL is validated when the module registers
    /// hook endpoints with the daemon at startup; malformed URLs surface as
    /// `PreflightError::InvalidHookUrl` from `preflight()`. Callers who want
    /// eager URL validation before invoking `new` may do so with any URL parsing
    /// strategy they choose; the spec does not mandate a specific crate.
    pub fn new(hook_base_url: String) -> Self {
        Self { hook_base_url }
    }

    /// Hook protocol path mapping: hook event type to URL path segment.
    ///
    /// The daemon's axum router is built from this mapping at startup.
    /// Phase 1 registers the 5 canonical Claude Code hook endpoints.
    /// Phase 4 federation modules may register additional paths via their
    /// own `hook_paths()` inherent method; the router accumulates all.
    pub fn hook_paths(&self) -> HashMap<HookType, String> {
        use HookType::*;
        [
            (SessionStart,      "/hooks/session-start".into()),
            (UserPromptSubmit,  "/hooks/prompt-submit".into()),
            (PreToolUse,        "/hooks/pre-tool-use".into()),
            (Notification,      "/hooks/notification".into()),
            (Stop,              "/hooks/stop".into()),
        ]
        .into()
    }

    /// Spawn a new Claude Code session.
    ///
    /// Invokes `claude` on PATH with the worktree-isolation arguments
    /// derived from `SpawnArgs`. Returns a `SessionHandle` on success.
    /// Full implementation in the Phase 1 monocle-runtime initialization story.
    pub async fn spawn(&self, args: SpawnArgs) -> Result<SessionHandle, SpawnError> {
        todo!("Phase 1 ClaudeCodeModule::spawn implementation")
    }

    /// Validate that the Claude Code binary is available and version-compatible.
    ///
    /// Called at daemon startup before accepting hook registrations.
    /// Checks `which claude`, parses `claude --version`, and verifies the
    /// version meets the minimum supported floor.
    ///
    /// Returns `Ok(EngineVersion)` on success or `Err(PreflightError)` with
    /// a diagnostic message suitable for display in the TUI status bar.
    pub async fn preflight(&self) -> Result<EngineVersion, PreflightError> {
        todo!("Phase 1 ClaudeCodeModule::preflight implementation")
    }
}

/// Arguments for spawning a new Claude Code session.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SpawnArgs {
    /// The project root directory for this session.
    pub project_root: PathBuf,
    /// Optional worktree path override (claude-squad isolation pattern).
    pub worktree: Option<PathBuf>,
    /// Environment variable overrides for the spawned subprocess.
    pub env_overrides: HashMap<String, String>,
}

/// Handle to a successfully spawned Claude Code session.
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

/// Version information returned by a successful preflight check.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct EngineVersion {
    /// Version string from `claude --version` (e.g., "2.0.0").
    pub version: String,
    /// Absolute path to the `claude` binary as resolved by `which`.
    pub binary_path: PathBuf,
}

/// Error from `ClaudeCodeModule::spawn`.
#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("claude binary not found: {0}")]
    BinaryNotFound(String),
    #[error("spawn failed: {0}")]
    SpawnFailed(String),
    #[error("worktree setup failed: {0}")]
    WorktreeFailed(String),
}

/// Error from `ClaudeCodeModule::preflight`.
#[derive(Debug, thiserror::Error)]
pub enum PreflightError {
    #[error("claude binary not found on PATH: {binary}")]
    BinaryNotFound { binary: String },
    #[error("claude version {found} is below minimum supported {minimum}")]
    VersionTooOld { found: String, minimum: String },
    /// Hook base URL is syntactically invalid. Detected at preflight time when the
    /// daemon attempts to parse the URL before binding axum routes. The URL was
    /// accepted by `ClaudeCodeModule::new` without validation (construction is
    /// infallible); this error surfaces the deferred validation failure.
    #[error("hook base URL is invalid: {url}: {reason}")]
    InvalidHookUrl { url: String, reason: String },
    #[error("preflight check failed: {reason}")]
    Failed { reason: String },
}

/// Error from `EngineModule::metadata` and `EngineModule::enrich`.
///
/// Both methods call `directories::BaseDirs::new()` to resolve the platform home
/// directory. In correctly configured environments `BaseDirs::new()` always succeeds.
/// In certain edge cases — a systemd `User=` unit with no `Environment=HOME` directive,
/// a broken passwd entry, or a hardened sandbox that unsets `$HOME` — it returns `None`.
/// Substituting a relative path in that case would constitute a silent failure (CLAUDE.md
/// SOUL #4): downstream callers (TUI display, DTU validator, transcript watcher) treat
/// the returned paths as absolute. Daemon initialization MUST fail fast with this error
/// and surface a diagnostic message to the operator.
#[derive(Debug, thiserror::Error)]
pub enum EngineMetadataError {
    /// Platform home directory could not be resolved (e.g., `$HOME` unset
    /// in a systemd `User=` unit with no `Environment=HOME`, broken passwd
    /// entry, or sandboxed runtime). Daemon initialization MUST fail fast
    /// with this error rather than substituting a relative path that
    /// downstream callers will treat as absolute (silent-failure violation
    /// per CLAUDE.md SOUL #4).
    #[error("platform home directory unresolvable (BaseDirs::new() returned None)")]
    HomeUnresolvable,
}
```

The `todo!()` markers are intentional: these are Phase 1 spec artifacts.
The Phase 1 story for `monocle-runtime` initialization provides the full
implementation. These signatures are binding — the implementer must not alter them.

---

## §Behavioral Contracts

**BC-ENGINE-001:** The `EngineModule` trait is defined in `monocle-core::engine` with
the exact vision-aligned signature in §EngineModule Trait Signature (methods: `id`,
`metadata`, `detect`, `enrich`, `on_hook`). Method return types are:
`id() -> &'static str`; `metadata() -> Result<EngineMetadata, EngineMetadataError>`;
`detect() -> bool`; `enrich() -> Result<EnrichedSession, EngineMetadataError>`;
`on_hook() -> HookResponse`. The `Result`-returning methods MUST NOT substitute a
default path for `EngineMetadataError::HomeUnresolvable`; daemon initialization MUST
fail fast with a diagnostic (no silent-fallback contract, CLAUDE.md SOUL #4).
Supporting types `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`, `SessionStatus`,
`HookResponse`, `HookDecision`, `DeferUntil`, `EngineMetadataError` are co-located in
`monocle-core::engine`. The trait carries NO sealed bound. Verification: `cargo check`
with the Phase 1 workspace; `rustdoc` confirms all types are publicly accessible and the
trait has no `private::Sealed` supertrait.

**BC-ENGINE-002:** `ClaudeCodeModule` (defined in `monocle-runtime::engine::claude_code`)
implements `EngineModule`. A public `ClaudeCodeModule::new(hook_base_url: String) -> Self`
constructor is provided. `id()` returns the string `"claude-code"`. `detect()` returns
`true` for any process whose `exe_path` has a final basename component equal to `"claude"`
or `"claude.js"` (strict basename match on the resolved binary path; NOT a suffix match
on `cmdline[0]`, which would produce false positives for `claude-squad`, `claudio`, etc.).
Verification: unit test in `monocle-runtime/tests/engine_module.rs` constructs a module via
`ClaudeCodeModule::new("http://127.0.0.1:7891".into())`, asserts `module.id() == "claude-code"`,
and tests `detect()` with: (a) a synthetic `ProcessSnapshot` with `exe_path = Some(PathBuf::from("/usr/local/bin/claude"))` → asserts `true`; (b) `exe_path = Some(PathBuf::from("/usr/local/bin/claude-squad"))` → asserts `false`; (c) `exe_path = None` (regardless of cmdline contents) → asserts `false`. Note: `detect()` consults ONLY `exe_path`; `cmdline` is preserved for engine-specific use in `enrich()` (e.g., reading `CLAUDE_SESSION_ID`) but is NOT used for engine identification — this avoids false positives from processes such as `claude-squad`, `claudio`, and `claude-code-router` that may place `"claude"` in `cmdline[0]`.

**BC-ENGINE-002-ERR:** `ClaudeCodeModule::metadata()` and `ClaudeCodeModule::enrich()` MUST
return `Err(EngineMetadataError::HomeUnresolvable)` when the platform home directory is
unresolvable (i.e., `directories::BaseDirs::new()` returns `None`). This contract enforces
the no-silent-fallback guarantee of CLAUDE.md SOUL #4: neither method may substitute a
relative path for an unresolvable home directory. Verification: test in
`monocle-runtime/tests/engine_module.rs` with the following specification:

1. Construct `ClaudeCodeModule::new("http://127.0.0.1:7891".into())`.
2. **Env-isolation strategy — two closures, one sync, one async:**
   `temp-env ^0.3` (feature `async_closure`) is pinned in SS-deps-pin-manifest.md
   `[dev-dependencies]`. It exposes two relevant APIs:
   - `temp_env::with_vars` — synchronous closure: `with_vars(kvs, FnOnce() -> R) -> R`
   - `temp_env::async_with_vars` — async closure (requires `features = ["async_closure"]`):
     `async_with_vars(kvs, impl Future<Output = R>) -> R`
   Because `metadata()` is synchronous and `enrich()` is `async`, they require different
   wrappers and MUST NOT be co-located inside the same `with_vars` call (which accepts
   a synchronous closure only and does not support `.await` inside it).
   **Env-isolation rationale:** `temp-env` restores all modified variables on drop — safe
   for multi-threaded test runs because no `std::env::set_var` / `remove_var` call outlives
   the closure scope. Manual `std::env::remove_var` without `temp-env` is unsafe in
   multi-threaded Rust test harnesses: the global environment is shared state across all
   threads in the same process, and a race between a test clearing `HOME` and another test
   reading it produces non-deterministic failures. `serial_test` serialises test execution
   to avoid the race but still leaves the environment mutated if a test panics before
   cleanup; `temp-env` is superior because it uses RAII cleanup on both normal and panic
   exit paths.
3. **Variables to clear — four env vars (corrected from prior v1.1.4 list):**
   Clear exactly these four variables, each with `None::<&str>` to explicitly unset (not
   empty-string):
   - `HOME` — Linux/macOS home resolution
   - `USERPROFILE` — Windows primary home resolution
   - `HOMEDRIVE` — Windows legacy home prefix (combined with `HOMEPATH`)
   - `HOMEPATH` — Windows legacy home path (combined with `HOMEDRIVE`)
   Use the explicit form `[("HOME", None::<&str>), ("USERPROFILE", None::<&str>),
   ("HOMEDRIVE", None::<&str>), ("HOMEPATH", None::<&str>)]` so the unset-vs-empty-string
   distinction is unambiguous in the test source.
   **Why XDG_* were removed:** The prior v1.1.4 specification listed `XDG_DATA_HOME`,
   `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, and `XDG_RUNTIME_DIR`. These are NOT consulted
   by `directories::BaseDirs::home_dir()` in `directories 6`; they affect `data_dir()`,
   `config_dir()`, `cache_dir()`, and `runtime_dir()` respectively. Clearing them does not
   affect `BaseDirs::new()` null-vs-Some result. They were documentation noise that
   misleads implementers about what `BaseDirs::new()` actually checks. Removed.
   **Windows CI caveat:** On Windows, `BaseDirs` may also fall back to `FOLDERID_Profile`
   (resolved via `SHGetKnownFolderPath` Windows COM call) regardless of env-var state.
   GitHub Actions Windows runners typically have a registered user SID, so `home_dir()`
   MAY succeed even with all four vars cleared. The test on Windows CI is therefore
   best-effort for the `None` path; the contract is fully deterministic on Linux/macOS
   where the four env vars are the only resolution mechanism.
4. **Sync half — `metadata()` test (use `temp_env::with_vars`):**
   ```rust
   temp_env::with_vars(
       [("HOME", None::<&str>), ("USERPROFILE", None::<&str>),
        ("HOMEDRIVE", None::<&str>), ("HOMEPATH", None::<&str>)],
       || {
           assert!(module.metadata().is_err());
           assert!(matches!(
               module.metadata().unwrap_err(),
               EngineMetadataError::HomeUnresolvable
           ));
       },
   );
   ```
5. **Async half — `enrich()` test (use `temp_env::async_with_vars`):**
   Construct a synthetic `ProcessSnapshot` with the same field values used in the
   `detect()` test cases (pid, exe_path, empty cmdline/env). In a separate
   `#[tokio::test]` (or within an `async` block):
   ```rust
   temp_env::async_with_vars(
       [("HOME", None::<&str>), ("USERPROFILE", None::<&str>),
        ("HOMEDRIVE", None::<&str>), ("HOMEPATH", None::<&str>)],
       async {
           assert!(module.enrich(&snapshot).await.is_err());
           assert!(matches!(
               module.enrich(&snapshot).await.unwrap_err(),
               EngineMetadataError::HomeUnresolvable
           ));
       },
   ).await;
   ```

The test is placed in `monocle-runtime/tests/engine_module.rs` alongside the BC-ENGINE-002
`detect()` tests. `temp-env` is a `[dev-dependencies]` entry with `features = ["async_closure"]`;
it does not appear in the production binary. The Phase 1 implementer MUST NOT use `#[serial]`
as a substitute for `temp-env` — serialisation mitigates the race but does not guarantee
cleanup on panic. The implementer MUST NOT use `tokio::runtime::Handle::current().block_on()`
inside the sync closure as a workaround — that pattern induces a nested-runtime panic under
`#[tokio::test]` and is explicitly forbidden.

**BC-ENGINE-003:** `ClaudeCodeModule::hook_paths()` returns exactly 5 entries, one per
`HookType` variant, with the exact path strings in §Struct-level inherent operations.
`ClaudeCodeModule::spawn()` and `ClaudeCodeModule::preflight()` are inherent methods on
the struct (NOT trait methods). The ABI version is read as
`monocle_core::MONOCLE_ABI_VERSION` at the call site; no `abi_version` method appears
on any trait. Verification: unit test in `monocle-runtime/tests/engine_module.rs`
asserts `module.hook_paths().len() == 5` with the exact path string for each `HookType`.

---

## §Phase 1 PRD BC Pre-Staging

| BC ID | Description | Source Section |
|-------|-------------|----------------|
| BC-ENGINE-001 | `EngineModule` trait defined in `monocle-core::engine` with vision-exact signature (id/detect/on_hook) and no sealed bound; `metadata()` returns `Result<EngineMetadata, EngineMetadataError>` (vision-spirit-aligned elaboration); `enrich()` returns `Result<EnrichedSession, EngineMetadataError>` (vision-spirit-aligned elaboration); no-silent-fallback contract enforced on `HomeUnresolvable` | §EngineModule Trait Signature |
| BC-ENGINE-002 | `ClaudeCodeModule::new(hook_base_url)` public constructor; implements `EngineModule`; `id()` returns "claude-code"; `detect()` performs strict basename match on `exe_path` (not cmdline) | §Phase 1 Implementation |
| BC-ENGINE-002-ERR | `ClaudeCodeModule::metadata()` and `enrich()` MUST return `Err(EngineMetadataError::HomeUnresolvable)` when `BaseDirs::new()` returns `None`; no-silent-fallback contract enforced via test in `monocle-runtime/tests/engine_module.rs` with `temp-env ^0.3` (features=["async_closure"]) — `with_vars` for sync `metadata()` half, `async_with_vars` for async `enrich()` half; clears HOME, USERPROFILE, HOMEDRIVE, HOMEPATH | §Behavioral Contracts (BC-ENGINE-002-ERR) |
| BC-ENGINE-003 | `ClaudeCodeModule::hook_paths()` returns 5-path mapping; spawn/preflight as inherent struct methods; ABI version read from const | §Struct-level inherent operations |

**Total: 4 BCs pre-staged.** Product-owner MUST use these exact IDs when formalizing
contracts with postconditions and verification harness stubs.

---

## §Trace

v1.1.6 changes (round-24 fixes F-R24-adv-1 + F-R24-adv-3):
- F-R24-adv-1 RESOLVED (MEDIUM — adversary finding): BC-ENGINE-002-ERR verification block
  previously called `temp_env::with_vars` (synchronous closure) and then used `.await`
  inside that closure for the `enrich()` assertion — an uncompilable pattern because
  `with_vars` accepts a synchronous `FnOnce` only. Fix: the test specification is now split
  into two halves. The sync half (`metadata()`) uses `temp_env::with_vars`. The async half
  (`enrich()`) uses `temp_env::async_with_vars`, which is available in `temp-env 0.3+`
  behind the `async_closure` feature flag. Verification: crates.io API confirmed
  `temp-env 0.3.6` (latest in 0.3.x line, published 2023-09-24, not yanked); source
  inspection of `vmx/temp-env` confirmed `async_with_vars` is gated on `features =
  ["async_closure"]` with signature `pub async fn async_with_vars<K,V,F,R>(kvs, F) -> R
  where F: Future<Output=R> + IntoFuture<Output=R>`. SS-deps-pin-manifest.md bumped from
  `^0.2` to `{ version = "^0.3", features = ["async_closure"] }` in the same burst
  (v1.1.7). The fallback path (block_on inside sync closure) was explicitly rejected:
  `Handle::current().block_on()` panics under `#[tokio::test]` due to nested-runtime
  prohibition; this prohibition is now documented in the test spec.
- F-R24-adv-3 RESOLVED (MEDIUM — adversary finding): The env-var list cleared to force
  `BaseDirs::new()` to return `None` was incorrect on two axes. (1) Missing Windows legacy
  vars: `HOMEDRIVE` and `HOMEPATH` (combined, they provide the Windows legacy home path
  used by `directories 6` BaseDirs on Windows); clearing only `USERPROFILE` allows
  `HOMEDRIVE`+`HOMEPATH` fallback to succeed, causing a false-pass on Windows CI.
  (2) XDG_* irrelevant: `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, and
  `XDG_RUNTIME_DIR` are NOT consulted by `BaseDirs::home_dir()` in `directories 6`; they
  affect `data_dir()`/`config_dir()`/`cache_dir()`/`runtime_dir()` only. Listing them
  created documentation noise that misleads implementers. Fix: corrected list is exactly
  four variables — `HOME`, `USERPROFILE`, `HOMEDRIVE`, `HOMEPATH` — each with
  `None::<&str>` to unset (not empty-string). XDG_* entries removed. Windows CI caveat
  documented: `FOLDERID_Profile` COM fallback may resolve home_dir regardless of env state
  on runners with a registered user SID; Linux/macOS path is fully deterministic.

v1.1.5 changes (round-23 micro-fix):
- BC-ENGINE-002-ERR ADDED to §Phase 1 PRD BC Pre-Staging table (between BC-ENGINE-002 and
  BC-ENGINE-003, preserving numerical order). Prior commit 563b573 added this BC to
  §Behavioral Contracts but missed the pre-staging cross-reference table. Total updated
  from "3 BCs pre-staged" to "4 BCs pre-staged". No behavioral content changed; this is
  a cross-reference consistency fix only. Downstream documents updated in same burst:
  SS-core-types-and-abi.md (BC count 3→4 for engine BCs, global total 15→16),
  SS-forward-compatibility.md (BC-ENGINE-002-ERR row added, table intro 15→16),
  product-brief.md (BC list and count updated 15→16).

v1.1.4 changes (round-22 fixes F-R22-1/F-R22-2/F-R22-3):
- F-R22-1/F-R22-2 RESOLVED (MEDIUM — adversary finding): §EngineModule Trait Signature
  opening paragraph previously claimed all five methods match the vision "exactly."
  This was imprecise: `id`, `detect`, and `on_hook` are vision-verbatim; `metadata`
  and `enrich` are vision-spirit-aligned elaborations (Result-wrapped return types).
  Fix: paragraph rewritten to enumerate the two provenance categories with explicit
  rationale. The vision is confirmed non-authoritative for this surface per CLAUDE.md
  §Architectural Authority. Implementers reading the vision sketch after this fix will
  see a clear statement that the Result signatures defined here supersede the infallible
  vision sketch. BC-ENGINE-001 Pre-Staging table row corrected: "(detect/enrich/on_hook)"
  changed to "(id/detect/on_hook)" for the vision-verbatim claim; `metadata()` and
  `enrich()` explicitly marked as vision-spirit-aligned elaborations. The vision document
  was NOT edited (per authority decision in this fix burst — the vision is human-approved
  verbatim; the architecture document is the canonical source for Phase 1 signatures).
- F-R22-3 RESOLVED (MEDIUM — adversary finding): BC-ENGINE-002 had no test specification
  for the `HomeUnresolvable` error paths in `metadata()` and `enrich()`. New sibling BC
  BC-ENGINE-002-ERR added specifying the full test in
  `monocle-runtime/tests/engine_module.rs`. Test isolation strategy: `temp-env = "^0.2"`
  (new `[dev-dependencies]` pin in SS-deps-pin-manifest.md v1.1.6). `temp-env` uses RAII
  cleanup (automatic on both normal and panic exit) making it safe for multi-threaded
  Rust test harnesses; `std::env::remove_var` without RAII is a data-race in multi-threaded
  tests; `serial_test` mitigates the race but lacks cleanup-on-panic guarantees.
  Variables cleared: HOME, USERPROFILE, XDG_DATA_HOME, XDG_CONFIG_HOME, XDG_CACHE_HOME,
  XDG_RUNTIME_DIR (all env vars that could allow `BaseDirs::new()` to succeed).

v1.1.3 changes (round-20 fixes F-R20-1/F-R20-3):
- F-R20-1 RESOLVED (MEDIUM): silent fallback `unwrap_or_else(|| PathBuf::from(".claude"))`
  eliminated from both `metadata()` and `enrich()`. Root cause: the F-R18-1 fix
  correctly replaced `ProjectDirs` with `BaseDirs::new()` but introduced
  `.unwrap_or_else(|| PathBuf::from(".claude"))` for the `None` case — a relative
  path fallback that downstream callers (TUI display, DTU validator, transcript watcher)
  treat as absolute. This constitutes a silent-failure violation (CLAUDE.md SOUL #4).
  Fix: `EngineMetadataError` enum added with `HomeUnresolvable` variant. `metadata()`
  return type changed from `EngineMetadata` to `Result<EngineMetadata, EngineMetadataError>`;
  `enrich()` return type changed from `EnrichedSession` to
  `Result<EnrichedSession, EngineMetadataError>`. Both implementations use
  `BaseDirs::new().ok_or(EngineMetadataError::HomeUnresolvable)?` — daemon initialization
  must fail fast with a typed error and operator-visible diagnostic rather than silently
  operating on a wrong relative path. BC-ENGINE-001 updated to document the Result return
  types and the no-silent-fallback contract.
- F-R20-3 RESOLVED (LOW): `ClaudeCodeModule::new` rustdoc previously recommended
  `Url::parse(&hook_base_url)` from the `url` crate for eager validation. The `url` crate
  is not pinned in SS-deps-pin-manifest.md; recommending it in spec rustdoc would cause
  implementers to pull an unpinned transitive. Fix: crate recommendation removed. Rustdoc
  now describes the infallible-construction / fail-at-preflight contract without mandating
  a specific validation crate.

v1.1.2 changes (round-19 fixes F-R18-1/F-R18-2/F-R18-4):
- F-R18-1 RESOLVED (CRITICAL): both `ProjectDirs::from("com", "anthropic", "claude-code")`
  call sites in `metadata()` and `enrich()` replaced with
  `BaseDirs::new().map(|b| b.home_dir().join(".claude"))`. Root cause: round-17's
  N16-1 fix correctly removed the `dirs` crate but chose `ProjectDirs` which applies
  XDG-conforming path transforms (`~/Library/Application Support/...` on macOS,
  `~/.config/...` on Linux) — wrong for Claude Code which is XDG-non-conforming and
  uses `~/.claude/` on every platform. `directories::BaseDirs::home_dir()` returns
  the platform home directory without XDG transforms; `.join(".claude")` then
  constructs the correct path on all platforms. `directories 6` remains the pinned
  crate (no new dep); `BaseDirs` and `ProjectDirs` are both in that crate.
- F-R18-2 RESOLVED: `ClaudeCodeModule::new` rustdoc expanded with a `# Validation`
  section explicitly documenting the no-validation-at-construction contract:
  `hook_base_url` is accepted without URL parsing; malformed URLs surface as
  `PreflightError::InvalidHookUrl` from `preflight()`. `PreflightError::InvalidHookUrl`
  variant added to the enum (was missing; the rustdoc referenced it but the enum
  did not define it).
- F-R18-4 RESOLVED: BC-ENGINE-002 test case (c) reworded from
  `exe_path = None, cmdline[0] = "claude"` to `exe_path = None (regardless of
  cmdline contents)`. Explicit note added: `detect()` consults ONLY `exe_path`;
  `cmdline` is used only in `enrich()` for session enrichment, not for engine
  identification.

v1.1.1 changes (round-16 adversary N16-1/N16-2/N16-3/N16-4):
- N16-1 RESOLVED: all four `dirs::home_dir()` calls replaced with
  `directories::ProjectDirs::from("com", "anthropic", "claude-code")` (pinned as
  `directories 6` in SS-deps-pin-manifest.md). No `dirs` crate introduced or referenced.
  Affected: `metadata()` (2 calls) and `enrich()` (2 calls).
  (NOTE: N16-1 was partially wrong — ProjectDirs resolves XDG paths, not `~/.claude/`.
  Corrected in v1.1.2 by F-R18-1 above.)
- N16-2 RESOLVED: `ClaudeCodeModule::new(hook_base_url: String) -> Self` public
  constructor added to the inherent `impl ClaudeCodeModule` block. BC-ENGINE-002
  updated to require the constructor and test it explicitly.
- N16-3 RESOLVED: claim "matches vision exactly" replaced with precise text
  distinguishing which parts match verbatim (method signatures) vs which parts are
  vision-spirit-aligned elaborations (`EngineMetadata` fields). Rationale for both
  elaborations documented inline.
- N16-4 RESOLVED: `ProcessSnapshot` gains `ppid: Option<u32>` (parent PID for
  process-tree analysis) and `exe_path: Option<PathBuf>` (resolved binary path via
  `/proc/<pid>/exe` readlink on Linux, platform-equivalent on macOS/Windows).
  `detect()` rewritten to perform strict basename match on `exe_path` instead of
  suffix match on `cmdline[0]`. Detection rule, false-positive avoidance, and `None`
  semantics documented in `ProcessSnapshot` doc-comment and `detect()` body comment.
  BC-ENGINE-002 verification updated with three cases: true positive, false positive
  guard (`claude-squad`), and `exe_path=None` guard.

v1.1 changes (human Q-15-1, round-14 adversary N1/N2):
- N1 RESOLVED: trait signature restored to vision-exact (detect/enrich/on_hook).
  Removed hook_paths/spawn/preflight/abi_version from trait surface; moved to
  `ClaudeCodeModule` inherent methods per vision authority.
- N2 RESOLVED: sealed pattern removed entirely. `EngineModule` trait is open.
  SS-forward-compatibility.md lines 95–97 veto honored. No `mod private`, no
  `Sealed` supertrait, no `plugin-sdk-escape-hatch` feature flag on this trait.
- Supporting types fully specified: `EngineMetadata`, `ProcessSnapshot`,
  `EnrichedSession`, `SessionStatus`, `HookResponse`, `HookDecision`, `DeferUntil`.
- `SessionStatus`, `HookDecision`, `DeferUntil` carry `#[non_exhaustive]` per
  BC-TYPES-001.
- BC-ENGINE-003 added to capture the inherent-methods contract.

Cross-references:
- `SS-core-types-and-abi.md` — `FactoryAdapter`, `HookType` enum, `HookEvent` variants
- `SS-daemon-lifecycle.md` — daemon startup sequence (ClaudeCodeModule::preflight
  called at step 1 before lock-file write)
- `SS-deps-pin-manifest.md` — `async-trait = "^0.1"` (Phase 1 Pin Manifest)
- Vision §EngineModule lines 111–128 (original sketch; non-authoritative for Phase 1 signatures — this document supersedes it per CLAUDE.md §Architectural Authority)
- `SS-forward-compatibility.md` lines 95–97 (sealing veto)
