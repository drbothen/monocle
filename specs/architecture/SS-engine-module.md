---
document_type: architecture-section
level: L3
section: "engine-module"
slug: "engine-module-trait-stability"
subsystem: "core"
version: "1.1"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-13T12:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md
input-hash: "[live-state]"
traces_to: "vision authority restoration per human Q-15-1; round-14 adversary N1/N2; SS-forward-compatibility lines 95-97 veto honored; F-FC-I003 adversary finding; vision §EngineModule lines 111-128; brief v1.4.7 §Harness plane"
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

The signature below is the authoritative Phase 1 contract. It matches
`domain-monocle-vision-synthesis.md` §EngineModule lines 111–128 exactly.

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
    fn metadata(&self) -> EngineMetadata;

    /// Detect whether a running process matches this engine's signature.
    /// Sync because process inspection is OS-level cheap; called per-process during scan.
    fn detect(&self, proc: &ProcessSnapshot) -> bool;

    /// Enrich a detected process snapshot with engine-specific context
    /// (session ID derivation, transcript path resolution, harness config introspection).
    async fn enrich(&self, proc: &ProcessSnapshot) -> EnrichedSession;

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
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    /// Process ID of the running process.
    pub pid: u32,
    /// Full command line including argv[0] and all arguments.
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

    fn metadata(&self) -> EngineMetadata {
        EngineMetadata {
            display_name: "Claude Code",
            icon: '●',
            config_paths: vec![
                dirs::home_dir().unwrap_or_default().join(".claude"),
                dirs::home_dir().unwrap_or_default().join(".claude.json"),
            ],
            hook_schema_version: 1,
        }
    }

    fn detect(&self, proc: &ProcessSnapshot) -> bool {
        proc.cmdline
            .first()
            .map(|arg0| arg0.ends_with("claude") || arg0.ends_with("claude.js"))
            .unwrap_or(false)
    }

    async fn enrich(&self, proc: &ProcessSnapshot) -> EnrichedSession {
        let session_id = proc
            .env
            .get("CLAUDE_SESSION_ID")
            .cloned()
            .unwrap_or_else(|| format!("pid-{}", proc.pid));

        let transcript_path = proc.working_dir.as_ref().map(|cwd| {
            // Standard Claude Code transcript layout:
            // ~/.claude/projects/<cwd-sha256-hex>/
            // Phase 1: placeholder path; full derivation in Phase 1 story.
            dirs::home_dir()
                .unwrap_or_default()
                .join(".claude")
                .join("projects")
                .join(format!("{}", cwd.display()))
        });

        EnrichedSession {
            session_id,
            harness_type: self.id().to_string(),
            transcript_path,
            config_path: Some(
                dirs::home_dir().unwrap_or_default().join(".claude"),
            ),
            status: SessionStatus::Active,
            last_event_micros: 0, // updated by on_hook
        }
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
    #[error("preflight check failed: {reason}")]
    Failed { reason: String },
}
```

The `todo!()` markers are intentional: these are Phase 1 spec artifacts.
The Phase 1 story for `monocle-runtime` initialization provides the full
implementation. These signatures are binding — the implementer must not alter them.

---

## §Behavioral Contracts

**BC-ENGINE-001:** The `EngineModule` trait is defined in `monocle-core::engine` with
the exact vision-aligned signature in §EngineModule Trait Signature (methods: `id`,
`metadata`, `detect`, `enrich`, `on_hook`). Supporting types `EngineMetadata`,
`ProcessSnapshot`, `EnrichedSession`, `SessionStatus`, `HookResponse`, `HookDecision`,
`DeferUntil` are co-located in `monocle-core::engine`. The trait carries NO sealed
bound. Verification: `cargo check` with the Phase 1 workspace; `rustdoc` confirms all
types are publicly accessible and the trait has no `private::Sealed` supertrait.

**BC-ENGINE-002:** `ClaudeCodeModule` (defined in `monocle-runtime::engine::claude_code`)
implements `EngineModule`. `id()` returns the string `"claude-code"`. `detect()` returns
`true` for any process whose `cmdline[0]` ends with `"claude"` or `"claude.js"`.
Verification: unit test in `monocle-runtime/tests/engine_module.rs` asserts
`module.id() == "claude-code"` and `module.detect(&snap) == true` for a synthetic
`ProcessSnapshot` with `cmdline[0] = "/usr/local/bin/claude"`.

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
| BC-ENGINE-001 | `EngineModule` trait defined in `monocle-core::engine` with vision-exact signature (detect/enrich/on_hook) and no sealed bound | §EngineModule Trait Signature |
| BC-ENGINE-002 | `ClaudeCodeModule` implements `EngineModule`; `id()` returns "claude-code"; `detect()` matches claude processes | §Phase 1 Implementation |
| BC-ENGINE-003 | `ClaudeCodeModule::hook_paths()` returns 5-path mapping; spawn/preflight as inherent struct methods; ABI version read from const | §Struct-level inherent operations |

**Total: 3 BCs pre-staged.** Product-owner MUST use these exact IDs when formalizing
contracts with postconditions and verification harness stubs.

---

## §Trace

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
- Vision §EngineModule lines 111–128 (authoritative trait signature)
- `SS-forward-compatibility.md` lines 95–97 (sealing veto)
