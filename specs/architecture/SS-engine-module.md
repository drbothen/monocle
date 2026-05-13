---
document_type: architecture-section
level: L3
section: "engine-module"
slug: "engine-module-trait-stability"
subsystem: "core"
version: "1.0"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-13T10:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md
input-hash: "[live-state]"
traces_to: "F-FC-I003 adversary finding; vision §EngineModule lines 111-128; brief v1.4.7 §Harness plane"
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

---

## §EngineModule Trait Signature

```rust
// monocle-core/src/engine.rs

use std::collections::HashMap;
use std::path::PathBuf;

/// Implemented by each AI coding harness adapter.
///
/// Phase 1 ships one built-in: `ClaudeCodeModule` (statically compiled into
/// `monocle-runtime`). Phase 4 adds `CodeMachineModule`. Phase 3+ allows
/// third-party WASM plugins implementing this trait via `monocle-plugin-sdk`.
///
/// The trait is sealed for Phase 1 (same escape-hatch pattern as FactoryAdapter;
/// see §Sealed Pattern Relaxation). Phase 3 relaxes via `plugin-sdk-escape-hatch`.
#[async_trait::async_trait]
pub trait EngineModule: Send + Sync + private::Sealed {
    /// Unique stable identifier for this harness adapter.
    /// Examples: "claude-code", "codemachine".
    /// MUST be stable across restarts — used as a session-roster key.
    fn id(&self) -> &str;

    /// Human-readable metadata surfaced in the Sessions panel header.
    fn metadata(&self) -> EngineMetadata;

    /// Hook protocol path mapping: hook event type to URL path.
    ///
    /// For `ClaudeCodeModule` in Phase 1:
    /// ```
    /// SessionStart     -> "/hooks/session-start"
    /// UserPromptSubmit -> "/hooks/prompt-submit"
    /// PreToolUse       -> "/hooks/pre-tool-use"
    /// Notification     -> "/hooks/notification"
    /// Stop             -> "/hooks/stop"
    /// ```
    ///
    /// Phase 4 federation modules may register additional paths.
    /// The daemon's axum router is built from the union of all registered modules'
    /// hook_paths at startup.
    fn hook_paths(&self) -> HashMap<HookType, String>;

    /// Spawn a new session under this engine.
    ///
    /// Returns a `SessionHandle` on success. The handle carries the pid, session
    /// UUID, and the axum-routable hook endpoints for this session. Phase 1:
    /// `ClaudeCodeModule::spawn` invokes `claude` on PATH with the worktree-
    /// isolation arguments derived from `SpawnArgs`.
    async fn spawn(&self, args: SpawnArgs) -> Result<SessionHandle, SpawnError>;

    /// Validate that the engine binary is available and version-compatible.
    ///
    /// Called at daemon startup before accepting hook registrations.
    /// For `ClaudeCodeModule`: checks `which claude`, parses `claude --version`,
    /// and verifies the version meets the minimum supported floor.
    ///
    /// Returns `Ok(EngineVersion)` on success or `Err(PreflightError)` with a
    /// diagnostic message suitable for display to the user.
    async fn preflight(&self) -> Result<EngineVersion, PreflightError>;

    /// ABI version this engine module was compiled against.
    ///
    /// Defaults to `crate::MONOCLE_ABI_VERSION`. Overriding this method is
    /// meaningless for Phase 1 built-ins (the sealed pattern prevents external
    /// impls). Phase 3 plugin SDK adapters use the default; if the host ABI
    /// version differs, the plugin loader refuses to activate the module.
    fn abi_version(&self) -> u32 {
        crate::MONOCLE_ABI_VERSION
    }
}

/// Sealing module — same pattern as factory::private.
/// Phase 3 escape hatch: `plugin-sdk-escape-hatch` feature enables
/// `__plugin_sdk_only::EngineModule` re-export for `monocle-plugin-sdk`.
mod private {
    pub trait Sealed {}
}

#[cfg(feature = "plugin-sdk-escape-hatch")]
pub mod __plugin_sdk_only {
    pub use super::private::Sealed;
    pub use super::EngineModule;
}

#[cfg(all(feature = "plugin-sdk-escape-hatch", feature = "__monocle-binary-build"))]
compile_error!(
    "plugin-sdk-escape-hatch must not be enabled in monocle binary builds. \
     This feature is for monocle-plugin-sdk only."
);
```

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
    /// Path to the harness's config directory (e.g., `~/.claude/`).
    pub config_path: PathBuf,
    /// JSON schema for hook payloads this harness emits.
    /// Used by the DTU validator to verify clone fidelity.
    pub hook_schema: &'static str,
}

/// Arguments for spawning a new harness session.
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

/// Handle to a successfully spawned harness session.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SessionHandle {
    /// PID of the spawned harness subprocess.
    pub pid: u32,
    /// Session UUID assigned by the harness (Claude Code's own session ID).
    pub session_id: String,
    /// The hook base URL this session's hook scripts POST to.
    pub hook_base_url: String,
}

/// Version information returned by a successful preflight check.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct EngineVersion {
    /// Version string from the harness binary (e.g., "2.0.0").
    pub version: String,
    /// Path to the harness binary as resolved by preflight.
    pub binary_path: PathBuf,
}

/// Error from `EngineModule::spawn`.
#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("harness binary not found: {0}")]
    BinaryNotFound(String),
    #[error("spawn failed: {0}")]
    SpawnFailed(String),
    #[error("worktree setup failed: {0}")]
    WorktreeFailed(String),
}

/// Error from `EngineModule::preflight`.
#[derive(Debug, thiserror::Error)]
pub enum PreflightError {
    #[error("harness binary not found on PATH: {binary}")]
    BinaryNotFound { binary: String },
    #[error("harness version {found} is below minimum supported {minimum}")]
    VersionTooOld { found: String, minimum: String },
    #[error("preflight check failed: {reason}")]
    Failed { reason: String },
}
```

---

## §Phase 1 Implementation: `ClaudeCodeModule`

Phase 1 ships exactly one `EngineModule` implementation: `ClaudeCodeModule`.

```rust
// monocle-runtime/src/engine/claude_code.rs

/// Phase 1 built-in EngineModule for Claude Code harness integration.
///
/// Detection: session processes are identified by `claude` binary name in the
/// process snapshot. Preflight: `which claude` + `claude --version` >= MINIMUM.
pub struct ClaudeCodeModule {
    /// Hook base URL where this daemon is listening (set at daemon start).
    hook_base_url: String,
}

impl monocle_core::engine::private::Sealed for ClaudeCodeModule {}

#[async_trait::async_trait]
impl monocle_core::engine::EngineModule for ClaudeCodeModule {
    fn id(&self) -> &str { "claude-code" }

    fn metadata(&self) -> EngineMetadata {
        EngineMetadata {
            display_name: "Claude Code",
            icon: '●',
            config_path: dirs::home_dir()
                .unwrap_or_default()
                .join(".claude"),
            hook_schema: include_str!("../../../monocle-proto/proto/monocle/v1/hook_envelope.proto"),
        }
    }

    fn hook_paths(&self) -> HashMap<HookType, String> {
        use HookType::*;
        [
            (SessionStart,     "/hooks/session-start".into()),
            (UserPromptSubmit, "/hooks/prompt-submit".into()),
            (PreToolUse,       "/hooks/pre-tool-use".into()),
            (Notification,     "/hooks/notification".into()),
            (Stop,             "/hooks/stop".into()),
        ].into()
    }

    async fn spawn(&self, args: SpawnArgs) -> Result<SessionHandle, SpawnError> {
        // Phase 1: invokes `claude` with worktree-isolation env vars.
        // Full implementation in monocle-runtime Phase 1 story.
        todo!("Phase 1 ClaudeCodeModule::spawn implementation")
    }

    async fn preflight(&self) -> Result<EngineVersion, PreflightError> {
        // Phase 1: `which claude` + `claude --version` parse.
        // Full implementation in monocle-runtime Phase 1 story.
        todo!("Phase 1 ClaudeCodeModule::preflight implementation")
    }
}
```

The `todo!()` markers are intentional: `ClaudeCodeModule` is a Phase 1 spec artifact.
The Phase 1 story for `monocle-runtime` initialization provides the full implementation.
These signatures are binding — the implementer must not change them.

---

## §Sealed Pattern Relaxation

Identical to `FactoryAdapter` (SS-core-types-and-abi.md §Sealed Pattern Relaxation).
The `plugin-sdk-escape-hatch` cargo feature enables `__plugin_sdk_only::EngineModule`
re-export in `monocle-core`. `monocle-plugin-sdk` consumes this to allow third-party
`EngineModule` implementations in WASM guest plugins. The feature is excluded from
all binary crate dependency declarations.

---

## §Behavioral Contracts

**BC-ENGINE-001:** The `EngineModule` trait is defined in `monocle-core::engine` with
the exact signature specified in this artifact (including `EngineMetadata`, `SpawnArgs`,
`SessionHandle`, `EngineVersion`, `SpawnError`, `PreflightError` supporting types, the
`private::Sealed` bound, and the `async_trait` annotation). Verification: `cargo check`
with the Phase 1 workspace; `rustdoc` confirms all types are publicly accessible.

**BC-ENGINE-002:** `ClaudeCodeModule` implements `EngineModule` with `id()` returning
`"claude-code"` and `hook_paths()` returning the exact 5-path mapping above (one path
per `HookType` variant). Verification: unit test in
`monocle-runtime/tests/engine_module.rs` asserts `module.id() == "claude-code"` and
`module.hook_paths().len() == 5` with the exact path strings.

---

## §Phase 1 PRD BC Pre-Staging

| BC ID | Description | Source Section |
|-------|-------------|----------------|
| BC-ENGINE-001 | `EngineModule` trait defined in `monocle-core::engine` with the signature in this artifact | §EngineModule Trait Signature |
| BC-ENGINE-002 | `ClaudeCodeModule` implements `EngineModule`; `id()` returns "claude-code"; `hook_paths()` returns 5-path mapping | §Phase 1 Implementation |

**Total: 2 BCs pre-staged.** IDs are reserved; product-owner MUST use these exact IDs
when formalizing contracts with postconditions and verification harness stubs.

---

## §Trace

Resolves F-FC-I003 from adversary fresh-pass (commit 4f5d4ff fix burst). Source:
vision §EngineModule definition. Human-authorized pre-Phase-1 lock-in.

This artifact is separate from SS-core-types-and-abi.md because that file was already
approximately 700 lines; introducing the full `EngineModule` spec there would exceed
the 800-1,200 token target for section files (architecture-section-template.md §Split
guidance: "split further if a section exceeds 1,500 tokens").

Cross-references:
- `SS-core-types-and-abi.md` — `FactoryAdapter` and `HookType` enum (used by `hook_paths`)
- `SS-daemon-lifecycle.md` — daemon startup sequence (preflight is called during startup)
- `SS-deps-pin-manifest.md` — `async-trait` crate (add to Phase 1 pin table:
  `async-trait = "^0.1"`, caret pin; widely used, no untrusted-input path)
- Vision §EngineModule and §Five Planes §Harness plane
