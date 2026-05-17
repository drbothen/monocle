---
document_type: architecture-section
level: L3
section: "core-types-and-abi"
slug: "types-and-abi"
subsystem: SS-02
version: "1.2.11"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-17T17:00:00Z
inputs: [product-brief.md, research/domain-monocle-vision-synthesis.md, SS-forward-compatibility.md, SS-deps-pin-manifest.md, SS-daemon-lifecycle.md, SS-permissions-phase1.md, planning/oq-research.md, semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md]
input-hash: "7f2e572"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: Core Types and ABI Stability Surface

## [Section Content]

## §Purpose

This artifact locks the Phase 1 stability contracts for `monocle-core`'s public API
and the cross-host wire format. Its purpose is to ensure that Phase 2, 3, and 4
evolution does not require breaking changes to Phase 1 consumers.

Phase 1 consumers of `monocle-core` include: the daemon binary (`monocle-runtime`),
the TUI binary (`monocle-tui`), and, prospectively, the Phase 3 plugin SDK
(`monocle-plugin-sdk`) and Phase 4 federation layer (`monocle-ipc`). Every
commitment made in this artifact is binding. Changes to any surface defined here
require an ADR. The phrase "breaking change" is defined concretely in
§Forward Compatibility Guarantees.

Covers FC-02 (`#[non_exhaustive]` enum policy), FC-03 (ABI version constant),
FC-04 (`FactoryAdapter` trait), and FC-05 (prost wire schemas).

---

## §ABI Version Constant (FC-03 resolution)

### Declaration

```rust
// monocle-core/src/abi.rs

/// ABI version for monocle-core's public interface.
///
/// This constant is used by the Phase 3 plugin SDK to refuse loading plugins
/// compiled against an incompatible host ABI, and by the Phase 4 federation
/// layer to validate peer-daemon compatibility before establishing a session.
///
/// Increment only via ADR. A change to this constant is a BREAKING change.
pub const MONOCLE_ABI_VERSION: u32 = 1;
```

`monocle-core::abi` is a dedicated submodule (`monocle-core/src/abi.rs`). It
re-exports from `monocle-core/src/lib.rs` via `pub use abi::MONOCLE_ABI_VERSION;`
so callers can write `monocle_core::MONOCLE_ABI_VERSION` without qualifying the
submodule path.

### Exposure Requirements

Every monocle binary (daemon, TUI) MUST expose `MONOCLE_ABI_VERSION` via the
`/status` HTTP endpoint (see SS-daemon-lifecycle.md §Health and Status Endpoints):

```json
{
  "abi_version": 1,
  ...
}
```

The Phase 3 plugin SDK embeds this value in the WIT component interface definition.
A plugin binary compiled against `MONOCLE_ABI_VERSION = 1` will refuse to load
against a host exposing `MONOCLE_ABI_VERSION = 2` unless an explicit compatibility
shim ships — the shim is Phase 5+ scope and requires its own ADR.

The Phase 4 federation handshake includes `abi_version` in the capability exchange
message. A remote daemon running a different ABI version responds with HTTP 409
Conflict to federation establishment requests if no compatibility shim is registered.

### Behavioral Contracts

**BC-2.02.001:** Every monocle binary exposes `abi_version: 1` in the `/status`
JSON response body. The field is present and equals `MONOCLE_ABI_VERSION` as
compiled into that binary. Verification: integration test asserts
`GET /status | jq .abi_version == 1`.

**BC-2.02.002:** `monocle-core` exports `MONOCLE_ABI_VERSION` as a `pub const u32`
at the crate root (`monocle_core::MONOCLE_ABI_VERSION`). Downstream crates may
compile-time-assert against it:

```rust
const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1,
    "ABI version mismatch — check monocle-core version");
```

Verification: compile-time assertion in `monocle-plugin-sdk/src/lib.rs` (added
during Phase 3 story); lint test in `monocle-core/tests/abi_stability.rs` asserting
the constant is exactly `1` and publicly accessible.

---

## §Enum Extensibility — `#[non_exhaustive]` Markers (FC-02 resolution)

### Mandatory Non-Exhaustive Enums

The following Phase 1 `monocle-core` enums MUST carry `#[non_exhaustive]`:

#### `HookType`

The canonical 5-variant hook event type enum:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HookType {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    Notification,
    Stop,
}
```

Rationale: Phase 4 brief §Scope (Phase 4 — PostToolUse revisit note) notes "revisit PostToolUse endpoint need at
this point." If Anthropic expands the Claude Code hook endpoint matrix (e.g., adds
`PostToolUse`), Phase 4 can add a variant without breaking match sites in Phase 1
consumers. `#[non_exhaustive]` requires all external `match` blocks to include a
wildcard arm, enforced by the compiler.

#### `HookEvent`

The unified hook event carrying the actual event payload:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HookEvent {
    SessionStart(SessionStartEvent),
    UserPromptSubmit(UserPromptSubmitEvent),
    PreToolUse(PreToolUseEvent),
    Notification(NotificationEvent),
    Stop(StopEvent),
}
```

`#[non_exhaustive]` permits adding new variants (new hook types) and new fields to
existing variant structs (via `#[non_exhaustive]` on the inner event structs as
well — see §Non-Exhaustive Inner Structs below). Phase 4 federation may introduce a
`FederatedEvent` variant that wraps a remote peer's event for local display.

#### Exhaustive Enums — Forbidden List (F-FC-C001 resolution)

The following Phase 1 enums are **exhaustive by explicit design**. `#[non_exhaustive]`
is FORBIDDEN on each. Both are documented in ADR-0004.

**`Phase1Permission`** (defined in `SS-permissions-phase1.md`): exhaustive because
the TUI permission dispatcher must handle every variant at compile time;
exhaustiveness is a compile-time correctness invariant. Phase 3 adds a categorically
distinct `monocle-plugin-sdk::PluginPermission` enum rather than extending
`Phase1Permission`. Adding a variant requires an ADR covering the new Claude Code
permission semantic, full match-arm coverage across all dispatch sites, and
security-reviewer sign-off.

**`ClaudeCodeTool`** (defined in `SS-permissions-phase1.md`): exhaustive because
this enum mirrors Claude Code's tool list exactly. Phase 1 defines fifteen named
variants (`Bash`, `Read`, `Write`, `Edit`, `MultiEdit`, `Glob`, `Grep`, `LS`,
`WebFetch`, `WebSearch`, `TodoRead`, `TodoWrite`, `NotebookRead`, `NotebookEdit`,
`Task`) plus the `Unknown(String)` catch-all. New tools are added by Anthropic as a
product decision; monocle's mapping enum must track Claude Code's set deliberately.
Each new tool addition requires an explicit ADR when Claude Code ships it, covering
monocle's intended permission dispatch behavior for that tool. The `Unknown(String)`
catch-all variant is the runtime safety net for tools added between monocle releases;
it IS the exhaustion escape hatch and is intentional.

These two enums are the complete Phase 1 exhaustive-enum forbidden list.
The exemptions are recorded in ADR-0004.

### Non-Exhaustive Inner Structs (F-FC-I002 resolution — all 5 fully specified)

All five event variant payload structs are fully specified below. No placeholders.
Gene-source: BC-HOOK-007 canonical hook-endpoint-body matrix (any-context-lazyclaude
deep-hooks pass). EX-2 architect extensions add `cwd`, `transcript_path`, and `prompt`
beyond the gene-source body fields.

```rust
// monocle-core/src/hook_events.rs

/// SessionStart hook — fired when a Claude Code session begins.
/// Gene-source body fields: pid, session_id.
/// EX-2 extension adds: cwd, transcript_path.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionStartEvent {
    /// Working directory of the spawned Claude Code session.
    pub cwd: String,
    /// Absolute path to Claude Code's session transcript file.
    pub transcript_path: String,
    /// Claude Code's own session UUID (not monocle's internal session ID).
    pub session_id: String,
    /// PID of the Claude Code subprocess (process.ppid in the hook JS).
    pub pid: u32,
}

/// UserPromptSubmit hook — fired when the user submits a prompt.
/// Gene-source body fields: pid, session_id.
/// EX-2 extension adds: prompt text.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPromptSubmitEvent {
    /// The submitted prompt text. May be large; bounded by BC-2.01.003 (256 KiB).
    pub prompt: String,
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
}

/// PreToolUse hook — fired before Claude Code executes a tool.
/// Gene-source body fields: type ('tool_info'), pid, tool_name, tool_input.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreToolUseEvent {
    /// Name of the tool about to be invoked (e.g., "Bash", "Edit", "Write").
    pub tool_name: String,
    /// JSON-encoded tool input arguments. Stored as serde_json::Value to avoid
    /// double-deserialization; the TUI renders this in the permission overlay.
    pub tool_input: serde_json::Value,
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
}

/// Notification hook — fired for permission prompts and assistant messages.
/// Gene-source body fields: pid, tool_name, tool_input, message.
/// Gene-source applies a client-side filter (notification_type == 'permission_prompt')
/// before POSTing; monocle preserves that filter in hook-install JS.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NotificationEvent {
    /// "permission_prompt" or "assistant_message". Always "permission_prompt"
    /// for Phase 1 due to the client-side filter. Retained as String
    /// (not enum) for Phase 2 forward compatibility when the filter may relax.
    pub notification_type: String,
    /// Tool name; populated for permission_prompt type. Empty string otherwise.
    pub tool_name: String,
    /// JSON tool input; populated for permission_prompt type.
    pub tool_input: serde_json::Value,
    /// Human-readable notification body. May be large; bounded by BC-2.01.003.
    pub message: String,
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
}

/// Stop hook — fired when a Claude Code session ends.
/// Gene-source body fields: pid, stop_reason, session_id.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StopEvent {
    /// Reason the session ended. Known values: "end_turn" | "max_tokens" |
    /// "tool_use" | "error". Stored as String for forward compatibility.
    pub stop_reason: String,
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
}
```

**Field reconciliation: `session_id` and `pid` in Rust structs vs proto `HookEnvelope`.**
Both fields appear in the Rust inner event structs AND in the proto `HookEnvelope`
(envelope fields 2 and 4). This is intentional. Rust structs carry them because the
daemon's HTTP handlers deserialize the JSON POST body directly into the event struct;
the proto envelope is not in the HTTP deserialization path. The proto `HookEnvelope`
carries them as envelope-level routing fields for Phase 4 federation. Inner proto
event messages do NOT re-declare `session_id` or `pid` — they are envelope-level.
The invariant `envelope.session_id == event.session_id` holds for all Phase 1 messages.

Phase 2 may add fields to any event struct without a breaking change, because
`#[non_exhaustive]` prevents exhaustive struct literal construction in downstream code.

### General Rule for All Other Phase 1 Public Enums

`#[non_exhaustive]` is the default for every `pub` enum in `monocle-core`.
Non-exhaustive markers are removed ONLY if:

1. An ADR documents why exhaustive matching is required for correctness (as
   `Phase1Permission` demonstrates), AND
2. The ADR records the Phase 3/4 extension strategy if the enum's semantics
   require future extension (e.g., a separate parallel enum rather than variant
   addition).

This rule is enforced by a `clippy` lint configuration: a custom project-level
deny list of `#[allow(non_exhaustive_omitted_patterns)]` is forbidden in monocle
source files (see SS-conventions-anti-patterns.md).

### Behavioral Contract

**BC-2.02.003:** Every `pub` enum in `monocle-core` carries `#[non_exhaustive]`
unless an ADR documents the exhaustiveness requirement. At Phase 1 PRD dispatch,
the exhaustive-enum forbidden list contains exactly two entries: `Phase1Permission`
and `ClaudeCodeTool` (both documented in ADR-0004). Any future exemption requires
a new ADR before the exemption is valid.
Verification: `cargo clippy` with a project-local lint that checks public enums for
the attribute; CI enforces this via the `--deny warnings` flag.

---

## §FactoryAdapter Trait (FC-04 resolution — CRITICAL)

### Module Location

The `FactoryAdapter` trait is defined in `monocle-core::factory`. It is not defined
in `monocle-workflow` because Phase 1 consumers of the trait span multiple crates
(`monocle-runtime` uses it for factory detection, `monocle-tui` uses it for the
Workflow panel display), and `monocle-workflow` does not exist until Phase 3. Placing
the trait in `monocle-core` gives it the widest possible Phase 1 visibility without
creating circular dependencies.

### Trait Signature

```rust
// monocle-core/src/factory.rs

use std::path::{Path, PathBuf};
use std::pin::Pin;
use futures::Stream;

/// Information returned by a successful factory detection.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FactoryDetection {
    /// The factory type name (e.g., "VSDD Factory").
    pub display_name: String,
    /// Path to the root of the detected factory workspace.
    pub workspace_root: PathBuf,
    /// Path to the canonical state file for this factory.
    pub state_file: PathBuf,
}

/// A parsed, structured representation of the factory pipeline state.
///
/// This is the canonical 7-field struct from the vision (approved 2026-05-12).
/// Fields are non-exhaustive to allow Phase 3+ extension without breaking
/// Phase 1 consumers. The `raw_content` field is explicitly NOT included
/// per user red-line (vision-7-field-only; no hybrid fields).
///
/// `convergence` and `cycle` are `Option` because STATE.md files in early pipeline
/// stages legitimately lack a §Session Resume Checkpoint convergence block and a
/// `current_cycle:` frontmatter key. Consumers (Workflow TUI panel) MUST display
/// `"pending"` or `"—"` for `None` fields rather than synthesizing a placeholder string.
///
/// `custom_fields` uses `serde_yaml_ng::Value` (project pins `serde_yaml_ng 0.10` in
/// SS-deps-pin-manifest.md) because STATE.md is a YAML-frontmatter file; mapping
/// unmapped frontmatter values through a YAML-native type avoids lossy JSON round-trips
/// on structured YAML values (sequences, nested mappings).
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FactoryState {
    /// Current pipeline phase identifier (e.g., "phase-1-spec-crystallization").
    /// Populated from STATE.md frontmatter `phase:` key.
    pub phase: String,
    /// Workflow status string.
    /// Populated from STATE.md frontmatter `status:` key.
    ///
    /// Valid values matching STATE.md frontmatter `status:` field convention:
    /// - `"active"` — default workflow state; work in progress.
    /// - `"blocked"` — gate is waiting on input (human or agent).
    /// - `"converged"` — current cycle complete; all findings resolved.
    /// - `"draft"` — pre-completion; artifact in authoring.
    /// - `"complete"` — cycle archived; no further changes expected.
    ///
    /// Implementations may encounter project-specific values beyond this list;
    /// consumers MUST handle unknown values gracefully (do not panic on
    /// unrecognized status strings).
    pub status: String,
    /// What the orchestrator is waiting on, if anything.
    /// Populated from STATE.md frontmatter `awaiting:` key.
    pub awaiting: Option<String>,
    /// Structured list of issues blocking pipeline progress.
    /// Populated by parsing the STATE.md §"Blocking Issues" body table.
    pub blocking_issues: Vec<BlockingIssue>,
    /// Convergence round count and finding trajectory from the most recent
    /// §Session Resume Checkpoint in the STATE.md body.
    /// `None` if no §Session Resume Checkpoint block is present in STATE.md.
    pub convergence: Option<ConvergenceMetrics>,
    /// Current cycle identifier (e.g., "cycle-001").
    /// Populated from STATE.md frontmatter `current_cycle:` key.
    /// `None` if the `current_cycle:` key is absent from the frontmatter.
    pub cycle: Option<String>,
    /// Forward-compatibility escape hatch: any frontmatter keys not explicitly
    /// mapped above are collected here for Phase 3+ schema evolution.
    /// Uses `serde_yaml_ng::Value` (project pin `serde_yaml_ng 0.10`) because
    /// STATE.md frontmatter is YAML; YAML-native values avoid lossy JSON coercion.
    pub custom_fields: std::collections::HashMap<String, serde_yaml_ng::Value>,
}

/// A single issue blocking pipeline progress.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct BlockingIssue {
    /// Issue identifier (e.g., "B-1").
    pub id: String,
    /// Severity classification.
    pub severity: BlockingSeverity,
    /// Human-readable description of the blocking condition.
    pub description: String,
    /// The agent or human responsible for resolving this issue, if known.
    pub owner: Option<String>,
}

/// Severity classification for a blocking issue.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BlockingSeverity {
    Critical,
    Important,
    Advisory,
}

/// Convergence metrics extracted from the most recent session checkpoint.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ConvergenceMetrics {
    /// The current adversarial review round number.
    pub round: u32,
    /// Count of findings per severity level from the most recent pass.
    pub findings_by_severity: std::collections::HashMap<BlockingSeverity, u32>,
    /// True if the most recent pass produced zero Critical or Important findings.
    pub trajectory_clean: bool,
}

/// Error reading or parsing the factory state file.
#[derive(Debug, thiserror::Error)]
pub enum FactoryReadError {
    #[error("state file not found at {path}: {source}")]
    NotFound {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("state file could not be read: {0}")]
    Io(#[from] std::io::Error),
    #[error("state file is malformed: {reason}")]
    Malformed { reason: String },
}

/// Error subscribing to factory state change notifications.
#[derive(Debug, thiserror::Error)]
pub enum FactorySubscribeError {
    #[error("filesystem watcher could not be initialized: {0}")]
    WatcherInit(String),
    #[error("subscription not supported in this phase (Phase 1 returns empty stream)")]
    NotSupported,
}

/// A stream of state change events emitted when the factory state file changes.
///
/// Phase 1 implementations return an empty/never-resolving stream (the `notify 8`
/// watcher is not activated until Phase 3). Phase 3 implements the live stream
/// via `notify::RecommendedWatcher` in `monocle-workflow`.
pub type StateChangeStream =
    Pin<Box<dyn Stream<Item = FactoryState> + Send + 'static>>;

/// Trait implemented by every factory adapter monocle supports.
///
/// Phase 1 ships one implementation: `VsddFactoryAdapter` (statically bundled).
/// Phase 3 promotes `VsddFactoryAdapter` to a WASM-loadable module; the trait
/// signature is identical — the Phase 1 static bundle uses the same trait methods
/// that the Phase 3 WASM component will expose, so the host-side dispatch code
/// requires no changes at the Phase 3 boundary.
///
/// The trait is OPEN — third-party crates may implement it. This is intentional:
/// it is the mechanism by which the Phase 3 plugin SDK exposes factory adapter
/// extensibility. Per SS-forward-compatibility.md §Item P3-1 — Verdict on Sealed:
/// "Do not apply the Sealed pattern to
/// `EngineModule` or `FactoryAdapter`." Sealing would prevent Phase 3 WASM plugin
/// authors from implementing this trait, defeating its purpose.
pub trait FactoryAdapter: Send + Sync + 'static {
    /// Detect whether the project at `workspace_root` uses this factory pattern.
    ///
    /// Returns `Some(FactoryDetection)` if detected; `None` if this adapter does
    /// not recognize the workspace layout.
    ///
    /// This method is called once at daemon startup and at TUI attach time.
    /// It must be fast (no network I/O; filesystem stat only).
    ///
    /// The `where Self: Sized` bound means this method is NOT available on
    /// `dyn FactoryAdapter`. Use `matches` instead for dyn-dispatch contexts.
    fn detect(workspace_root: &Path) -> Option<FactoryDetection>
    where
        Self: Sized;

    /// Returns true if this adapter recognizes `workspace_root`.
    ///
    /// Equivalent to `Self::detect(workspace_root).is_some()` but callable on
    /// `dyn FactoryAdapter` (no `where Self: Sized` bound). Phase 3 plugin SDK
    /// calls this on dynamically dispatched plugin adapters to probe which adapter
    /// handles a given workspace, before calling `read_state`.
    fn matches(&self, workspace_root: &Path) -> bool;

    /// Path to the canonical state file for this factory.
    ///
    /// For `VsddFactoryAdapter`: `<workspace_root>/.factory/STATE.md`.
    fn state_file_path(&self) -> &Path;

    /// Read the current pipeline state from the canonical state file.
    ///
    /// Returns a structured `FactoryState` on success. Returns `Err` if the
    /// file is absent, unreadable, or does not conform to the expected format.
    ///
    /// This method performs synchronous filesystem I/O. Callers in async
    /// contexts MUST use `tokio::task::spawn_blocking`.
    fn read_state(&self) -> Result<FactoryState, FactoryReadError>;

    /// Subscribe to filesystem changes on the canonical state file.
    ///
    /// Returns a `StateChangeStream` that emits a new `FactoryState` on each
    /// change detected by the filesystem watcher.
    ///
    /// Phase 1 implementations MUST return a never-resolving empty stream
    /// (the `notify 8` watcher is not activated until Phase 3):
    ///
    /// ```rust
    /// fn subscribe(&self) -> Result<StateChangeStream, FactorySubscribeError> {
    ///     Ok(Box::pin(futures::stream::empty()))
    /// }
    /// ```
    ///
    /// Phase 3 provides a live stream via `notify::RecommendedWatcher`. The
    /// stream terminates when the watcher is dropped or the state file is
    /// permanently removed.
    fn subscribe(&self) -> Result<StateChangeStream, FactorySubscribeError>;

    /// The factory's human-readable display name.
    ///
    /// Used in the TUI Workflow panel header and in log messages.
    /// Example: "VSDD Factory".
    fn display_name(&self) -> &str;

    /// The ABI version this adapter was compiled against.
    ///
    /// Default implementation returns `crate::MONOCLE_ABI_VERSION`.
    /// Phase 3 SDK adapters use the default; if the host ABI version differs,
    /// the plugin loader refuses to activate the adapter. Phase 1 static
    /// implementations inherit the default without override.
    fn abi_version(&self) -> u32 {
        crate::MONOCLE_ABI_VERSION
    }
}
```

### Phase 1 Implementation: `VsddFactoryAdapter`

```rust
// monocle-core/src/factory.rs (continued)

/// The Phase 1 static implementation of `FactoryAdapter` for VSDD factory workspaces.
///
/// Detection criterion: the workspace contains `.factory/STATE.md` with a YAML
/// frontmatter block that includes `document_type: pipeline-state`. This is the
/// exact format written by `vsdd-factory:state-manager` — monocle's own
/// `.factory/STATE.md` satisfies this criterion (self-referential test per
/// brief §Success Criteria).
pub struct VsddFactoryAdapter {
    workspace_root: PathBuf,
    state_file: PathBuf,
}

impl VsddFactoryAdapter {
    /// Construct a `VsddFactoryAdapter` rooted at `workspace_root`.
    ///
    /// Derives the state file path as `<workspace_root>/.factory/STATE.md`.
    ///
    /// # Validation
    ///
    /// This constructor performs **no validation** on `workspace_root`. The
    /// adapter is intentionally lazy: existence and content validation happens
    /// in `detect()` (returns `None` for invalid workspaces) and `read_state()`
    /// (returns `Err(...)` for missing or malformed STATE.md). This separation
    /// keeps construction infallible and lets callers handle validation errors
    /// at the right operational point.
    ///
    /// Callers that need eager validation should call
    /// `VsddFactoryAdapter::detect(&workspace_root)` before invoking `new`.
    pub fn new(workspace_root: PathBuf) -> Self {
        let state_file = workspace_root.join(".factory").join("STATE.md");
        Self { workspace_root, state_file }
    }
}

impl FactoryAdapter for VsddFactoryAdapter {
    fn detect(workspace_root: &Path) -> Option<FactoryDetection> {
        let state_file = workspace_root.join(".factory").join("STATE.md");
        let content = std::fs::read_to_string(&state_file).ok()?;
        // Minimal YAML frontmatter check: look for the document_type field
        // without pulling in a full YAML parser at detection time.
        if content.contains("document_type: pipeline-state") {
            Some(FactoryDetection {
                display_name: "VSDD Factory".to_string(),
                workspace_root: workspace_root.to_path_buf(),
                state_file,
            })
        } else {
            None
        }
    }

    fn state_file_path(&self) -> &Path {
        &self.state_file
    }

    fn matches(&self, workspace_root: &Path) -> bool {
        Self::detect(workspace_root).is_some()
    }

    fn read_state(&self) -> Result<FactoryState, FactoryReadError> {
        let content = std::fs::read_to_string(&self.state_file).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FactoryReadError::NotFound {
                    path: self.state_file.clone(),
                    source: e,
                }
            } else {
                FactoryReadError::Io(e)
            }
        })?;

        // Parse STATE.md frontmatter fields. Field names match the canonical
        // STATE.md YAML frontmatter keys produced by vsdd-factory:state-manager:
        //   phase:         current pipeline phase identifier
        //   status:        workflow status string
        //   awaiting:      optional waiting-on string
        //   current_cycle: optional cycle identifier
        // Body-level fields (blocking_issues, convergence) are parsed from
        // the markdown body sections below the frontmatter.
        let phase = parse_frontmatter_field(&content, "phase")
            .unwrap_or_else(|| "unknown".to_string());
        let status = parse_frontmatter_field(&content, "status")
            .unwrap_or_else(|| "unknown".to_string());
        let awaiting = parse_frontmatter_field(&content, "awaiting");
        // cycle: None when current_cycle: key is absent — do NOT substitute "unknown".
        let cycle = parse_frontmatter_field(&content, "current_cycle");

        // Collect any additional frontmatter keys not explicitly mapped above.
        // This provides the forward-compat escape hatch for future STATE.md fields.
        let custom_fields = parse_frontmatter_extra_fields(&content,
            &["phase", "status", "awaiting", "current_cycle",
              "document_type", "level", "version", "producer",
              "timestamp", "inputs", "input-hash", "traces_to", "project",
              "mode", "current_step", "dtu_required", "dtu_assessment",
              "dtu_clones_built", "dtu_services"]);

        // Phase 1: blocking_issues are stub-populated (empty Vec).
        // convergence: None when §Session Resume Checkpoint is absent — do NOT
        // synthesize a zero-round placeholder. Full body parsing
        // (§Blocking Issues table, §Session Resume Checkpoint) is implemented in
        // the Phase 3 monocle-workflow crate where pulldown-cmark is available.
        // Phase 1 surfaces the frontmatter-derived fields which cover the
        // Workflow panel's primary display: phase, status, awaiting, cycle.
        let blocking_issues = Vec::new();
        let convergence: Option<ConvergenceMetrics> = None;

        Ok(FactoryState {
            phase,
            status,
            awaiting,
            blocking_issues,
            convergence,
            cycle,
            custom_fields,
        })
    }

    fn subscribe(&self) -> Result<StateChangeStream, FactorySubscribeError> {
        // Phase 1: return an empty stream. Phase 3 activates notify 8 here.
        Ok(Box::pin(futures::stream::empty()))
    }

    fn display_name(&self) -> &str {
        "VSDD Factory"
    }
}

/// Extract a scalar value from YAML frontmatter without a full YAML parse.
///
/// The frontmatter block MUST start on the FIRST LINE of the document (line 0
/// must be exactly `---`). This anchors the parser to genuine frontmatter and
/// prevents matching markdown horizontal rules `---` that appear in the body.
///
/// Returns the trimmed, unquoted value string for `key: value` lines, or `None`
/// in any of the following cases:
///
/// - The document does not begin with `---`.
/// - The key is absent in the frontmatter block.
/// - The line is a continuation line (leading whitespace — part of a prior block scalar).
/// - The value is empty after trimming (e.g., `key: ` with trailing space).
/// - The value begins with `[` (flow-style list — requires a full YAML parser to decode).
/// - The value begins with `|` or `>` (block scalar marker — multi-line; not parsed).
///
/// YAML quoted scalars are unquoted: surrounding double quotes or single quotes
/// are stripped so callers receive the semantic string value, not the YAML
/// encoding. Example: `awaiting: "round 18 validation chain"` returns
/// `Some("round 18 validation chain".to_string())` (without the quotes).
/// Only a single layer of quoting is stripped; nested quotes are not processed.
///
/// This function and `parse_frontmatter_extra_fields` share identical guard semantics.
/// BC-2.02.005's `Some(_) or None` assertion is genuinely discriminating: `None`
/// means "key absent or value unparseable as a simple scalar".
fn parse_frontmatter_field(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();
    // Frontmatter MUST open on the very first line.
    let first = lines.next()?;
    if first.trim() != "---" {
        return None; // Document does not start with a frontmatter marker.
    }
    for line in lines {
        if line.trim() == "---" {
            break; // End of frontmatter block.
        }
        // Skip continuation lines (block scalar body lines begin with whitespace).
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        if let Some(rest) = line.strip_prefix(&format!("{}: ", key)) {
            let value = rest.trim();
            // Return None for empty values — semantically distinct from "key absent".
            if value.is_empty() {
                return None;
            }
            // Return None for flow-style lists and block scalars — these require a
            // full YAML parser to decode correctly.
            if value.starts_with('[')
                || value.starts_with('|')
                || value.starts_with('>')
            {
                return None;
            }
            // Strip surrounding double quotes (YAML double-quoted scalar).
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            // Strip surrounding single quotes (YAML single-quoted scalar).
            let value = value
                .strip_prefix('\'')
                .and_then(|v| v.strip_suffix('\''))
                .unwrap_or(value);
            return Some(value.to_string());
        }
    }
    None
}

/// Collect frontmatter key-value pairs NOT in the `known_keys` list.
///
/// Used to populate `FactoryState::custom_fields` for forward-compat schema
/// evolution. Only single-line `key: value` pairs with scalar values are
/// collected. The following are explicitly skipped and NOT stored:
///
/// - Flow-style lists (`[...]`) — would require a full YAML parser to decode correctly.
/// - Block scalars (`|` or `>` folded/literal) — multi-line; parsing requires
///   consuming continuation lines which this function does not attempt.
/// - Continuation lines (indented lines that are part of a prior block scalar) —
///   identified by leading whitespace and skipped individually.
/// - Empty values — lines matching `key: ` with nothing after the space are skipped.
///
/// Values are unquoted (same rule as `parse_frontmatter_field`) and wrapped as
/// `serde_yaml_ng::Value::String`. For full YAML parsing semantics including
/// nested structures, downstream code should re-parse with
/// `serde_yaml_ng::from_str` on the raw frontmatter content.
fn parse_frontmatter_extra_fields(
    content: &str,
    known_keys: &[&str],
) -> std::collections::HashMap<String, serde_yaml_ng::Value> {
    let mut result = std::collections::HashMap::new();
    let mut lines = content.lines();
    let first = lines.next().unwrap_or("");
    if first.trim() != "---" {
        return result;
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        // Skip continuation lines (block scalar body lines begin with whitespace).
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        let Some(colon_pos) = line.find(": ") else {
            continue;
        };
        let k = line[..colon_pos].trim();
        if known_keys.contains(&k) {
            continue;
        }
        let value_str = line[colon_pos + 2..].trim();
        // Skip empty values.
        if value_str.is_empty() {
            continue;
        }
        // Skip flow-style lists and block scalars — these require a full YAML parser.
        if value_str.starts_with('[')
            || value_str.starts_with('|')
            || value_str.starts_with('>')
        {
            continue;
        }
        // Unquote the scalar value (same rule as parse_frontmatter_field).
        let unquoted = value_str
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| {
                value_str
                    .strip_prefix('\'')
                    .and_then(|v| v.strip_suffix('\''))
            })
            .unwrap_or(value_str);
        result.insert(k.to_string(), serde_yaml_ng::Value::String(unquoted.to_string()));
    }
    result
}
```

### Phase 3 Plugin SDK Integration

Because `FactoryAdapter` is an open trait (no sealed bound), the Phase 3
`monocle-plugin-sdk` crate can implement it directly without any feature-flag
escape hatch. A Phase 3 WASM adapter implementing `FactoryAdapter` simply:

```rust
// monocle-plugin-sdk/src/adapter.rs (Phase 3 only)

use monocle_core::factory::FactoryAdapter;

/// Phase 3 WASM adapter implementing FactoryAdapter for plugin-SDK-loaded adapters.
/// ABI version is checked at WASM component load time via MONOCLE_ABI_VERSION.
pub struct SdkAdapter { /* wasmtime Store, WIT bindings */ }

impl FactoryAdapter for SdkAdapter { /* ... */ }
```

No `plugin-sdk-escape-hatch` feature flag is needed or defined. No `mod private`
module exists in `monocle-core::factory`. This is simpler and more correct than
the feature-flag pattern: the trait is open because it is intended to be implemented
by third-party code. Openness is the right default for extension traits.

### Behavioral Contracts

**BC-2.02.004:** `FactoryAdapter` trait is defined in `monocle-core::factory`
with the exact signature above (including `StateChangeStream` type alias,
`FactoryDetection`, `FactoryState` (7-field canonical struct), `BlockingIssue`,
`BlockingSeverity`, `ConvergenceMetrics`, `FactoryReadError`, `FactorySubscribeError`
supporting types, and the `matches` dyn-dispatch method). The trait carries NO sealed
bound (`Send + Sync + 'static` only) — it is an open extension trait per
SS-forward-compatibility.md §Item P3-1 — Verdict on Sealed.
Verification: `cargo check` with the Phase 1 workspace; `rustdoc` output confirms
public trait surface including all supporting types, and confirms no `private::Sealed`
supertrait appears.

**BC-2.02.005:** `VsddFactoryAdapter` implements `FactoryAdapter`. A public
`VsddFactoryAdapter::new(workspace_root: PathBuf) -> Self` constructor is provided;
it derives `state_file = workspace_root.join(".factory/STATE.md")`. The `detect`
static method returns `Some(FactoryDetection)` when called against monocle's own
workspace root (the directory containing `.factory/STATE.md` with
`document_type: pipeline-state` frontmatter). This is the self-referential detection
test from brief §Success Criteria. When `read_state` encounters
absent optional fields, the returned `FactoryState` carries `None` rather than
placeholder strings: absent `current_cycle:` → `cycle: None`; absent §Session
Resume Checkpoint → `convergence: None`. Consumers (Workflow TUI panel) display
`"pending"` or `"—"` for `None` fields. Verification: integration test
`monocle-core/tests/factory_self_referential.rs` calls
`VsddFactoryAdapter::detect(workspace_root)` with the monocle repository root as
`workspace_root`; asserts `Some(_)` is returned with `display_name == "VSDD Factory"`;
also constructs via `VsddFactoryAdapter::new(workspace_root)` and calls `read_state()`,
asserting `cycle` is `None` or `Some(_)` (not a hardcoded `"unknown"` string).

---

## §Prost Wire Schemas (FC-05 resolution)

### Crate

Wire schemas live in `monocle-proto`. The crate declares `prost 0.14` (EXACT pin
per SS-deps-pin-manifest.md) and `prost-build` as a `[build-dependencies]` entry.
Phase 1 generates Rust types via `build.rs` but activates no wire path — the
protobuf types are compiled into the binary and available for Phase 4 without
any Phase 4 workspace changes to `monocle-proto`.

### Field Number Convention

Phase 1 reserves field numbers **1–99** for core fields stable across all phases.
Phase 4 federation additions MUST use field numbers **100–999**. Phase 5+ additions
MUST use field numbers **1000+**. This reservation prevents accidental field
number collisions when Phase 4 adds federation-specific fields alongside Phase 1
fields. Breaking changes to any field with number 1–99 require bumping
`schema_version` AND an ADR.

### Schema Definitions

```protobuf
// monocle-proto/proto/monocle/v1/hook_envelope.proto
syntax = "proto3";
package monocle.v1;

// HookEnvelope is the canonical wire message for every hook event.
// Phase 1 defines this schema; Phase 4 activates the wire path.
// Field numbers 1-99: stable Phase 1 core fields.
// Field numbers 100-999: reserved for Phase 4 federation additions.
// Field numbers 1000+: reserved for Phase 5+.
message HookEnvelope {
  uint32 schema_version = 1;  // Always 1 for Phase 1 messages.
  string session_id      = 2; // Claude Code session identifier.
  int64  timestamp_micros = 3; // Event timestamp, UTC microseconds since Unix epoch.
  uint32 pid             = 4;  // PID of the Claude Code process that fired the hook.

  oneof event {
    SessionStartEvent    session_start  = 10;
    UserPromptSubmitEvent prompt_submit = 11;
    PreToolUseEvent      pre_tool_use   = 12;
    NotificationEvent    notification   = 13;
    StopEvent            stop           = 14;
  }
}

// SessionStart hook — fired when a Claude Code session begins.
message SessionStartEvent {
  string cwd             = 1; // Working directory of the session.
  string transcript_path = 2; // Absolute path to Claude Code's session transcript.
}

// UserPromptSubmit hook — fired when the user submits a prompt.
message UserPromptSubmitEvent {
  string prompt = 1; // The submitted prompt text (may be truncated at 64KiB).
}

// PreToolUse hook — fired before Claude Code executes a tool.
// monocle's response (exit code + optional JSON output) determines
// whether Claude Code proceeds. Fail-open: non-response = proceed.
message PreToolUseEvent {
  string tool_name  = 1; // Name of the tool about to be invoked.
  bytes  tool_input = 2; // JSON-encoded tool input arguments (raw bytes).
}

// Notification hook — fired for assistant messages and permission prompts.
message NotificationEvent {
  string notification_type = 1; // "permission_prompt" or "assistant_message".
  string tool_name         = 2; // Populated when notification_type = "permission_prompt".
  bytes  tool_input        = 3; // JSON-encoded tool input; populated on permission prompts.
  string message           = 4; // Human-readable notification body (may be large; see BC-2.01.003).
}

// Stop hook — fired when a Claude Code session ends (agentic loop complete).
message StopEvent {
  string stop_reason = 1; // "end_turn" | "max_tokens" | "tool_use" | "error".
}
```

### Schema Evolution Rules

1. New fields added in Phase 4 MUST use field numbers 100–999. Example:
   `string peer_origin_host = 100;` in `HookEnvelope` for federation provenance.
2. Any change to a Phase 1 field (numbers 1–99) is a BREAKING change: bump
   `schema_version` AND produce an ADR.
3. Removing a Phase 1 field is forbidden. Mark deprecated fields with the
   `[deprecated = true]` protobuf option and retain the field number as reserved.
4. Phase 4 deserialization of Phase 1 messages (those with `schema_version = 1`)
   MUST succeed even if the Phase 4 receiver knows about additional fields
   (proto3 forward compatibility guarantee: unknown fields are preserved).

### Behavioral Contracts

**BC-2.02.006 (proto schema contract):** The `.proto` message definition declares
`schema_version` at proto field number 1 in `HookEnvelope`. This is a wire-format
contract — it governs the binary encoding on the wire and the field number visible
to every proto consumer regardless of language. Verification: `protoc --decode`
confirms field number 1 is `schema_version` in any encoded Phase 1 `HookEnvelope`
message; a `monocle-proto/tests/wire_field_order.rs` test round-trips a message and
asserts the field number assignment via prost-build's generated descriptor.

**BC-2.02.007 (Rust surface contract):** The prost-build-generated `HookEnvelope`
Rust struct exposes `pub schema_version: u32`. The value is `1` for all Phase 1-origin
messages. The generated Rust struct field order is an implementation detail of
prost-build and is NOT a behavioral contract. Verification: a unit test in
`monocle-proto/tests/schema_version.rs` constructs a `HookEnvelope` with
`schema_version: 1` and asserts `envelope.schema_version == 1`.

**BC-2.02.008:** The Phase 1 `HookEnvelope` schema is the canonical wire
representation for cross-host federation in Phase 4. Phase 4 federation nodes
check `schema_version` before deserializing event payloads. A node receiving a
message with an unrecognized `schema_version` MUST log a warning and skip the
message rather than crash (proto3 unknown-field semantics). Verification: Phase 4
integration test simulates a `schema_version = 0` message and asserts the receiver
skips without panic.

---

## §Phase 1 PRD BC Pre-Staging

The following behavioral contract IDs are pre-staged by this artifact for
formalization during `/vsdd-factory:create-prd`. The product-owner assigns full
preconditions, postconditions, evidence requirements, and verification harness
stubs during PRD authoring.

| BC ID | Description | Source Section |
|-------|-------------|----------------|
| BC-2.02.001 | Every monocle binary exposes `abi_version: 1` in `/status` response | §ABI Version Constant |
| BC-2.02.002 | `monocle-core` exports `MONOCLE_ABI_VERSION` as pub const at crate root | §ABI Version Constant |
| BC-2.02.003 | Every pub enum in `monocle-core` carries `#[non_exhaustive]` unless ADR exempts it (two current exemptions: Phase1Permission and ClaudeCodeTool per ADR-0004) | §Enum Extensibility |
| BC-2.02.004 | `FactoryAdapter` trait defined in `monocle-core::factory` with the 7-field FactoryState and all supporting types per this artifact | §FactoryAdapter Trait |
| BC-2.02.005 | `VsddFactoryAdapter::new(workspace_root)` public constructor; passes self-referential detection test; `read_state` returns `None` for absent optional fields (cycle, convergence) | §FactoryAdapter Trait |
| BC-2.02.006 | `.proto` message definition declares `schema_version` at proto field number 1 (wire-format contract) | §Prost Wire Schemas |
| BC-2.02.007 | Prost-build-generated `HookEnvelope` Rust struct exposes `pub schema_version: u32` with value `1` for Phase 1 messages (Rust surface contract) | §Prost Wire Schemas |
| BC-2.02.008 | Phase 1 HookEnvelope schema is canonical wire representation; Phase 4 validates `schema_version` before deserializing | §Prost Wire Schemas |
| BC-2.01.010 | Lock-file JSON includes `contract_version: u32 = 1` as the first key (see SS-daemon-lifecycle.md §Lock File Discovery Policy) | Covered in SS-daemon-lifecycle.md |

**Total: 8 BCs authored in this artifact; BC-2.01.010 cross-referenced from `SS-daemon-lifecycle.md`.**
The product-owner MUST NOT renumber these BCs during PRD authoring; the IDs above are
anchor identifiers that cross-references in this artifact and in SS-forward-compatibility.md
rely upon. The pre-split proto contract (retired per F-FC-O004) was divided into BC-2.02.006 (wire-format) and BC-2.02.007 (Rust
surface) to eliminate the wire-vs-Rust conflation identified in F-FC-O004.

Combined with SS-engine-module.md (BC-2.03.001, BC-2.03.002, BC-2.03.003,
BC-2.03.004 = 4 BCs) and SS-daemon-lifecycle.md (BC-2.01.007, BC-2.01.008, BC-2.01.009,
BC-2.01.010 = 4 BCs), the pre-Phase-1 pre-staged total is **16 BCs** across all
architecture artifacts. The authoritative enumeration with source references is in
SS-forward-compatibility.md §Cross-Phase Decisions Required closing paragraph.

---

## §Forward Compatibility Guarantees

Any change to the contracts defined in this artifact is a BREAKING change requiring
an ADR. The following operations are explicitly NOT breaking and do not require an ADR:

- Adding a new variant to any `#[non_exhaustive]` enum (except `Phase1Permission`
  which is exhaustive by ADR-exemption).
- Adding a new field to any `#[non_exhaustive]` struct.
- Adding a new proto field with a field number in the Phase 4 reserved range (100–999)
  or Phase 5+ range (1000+).
- Adding a new method to `FactoryAdapter` with a default implementation (the trait
  is open; existing impls are unaffected by new default methods).

The following operations ARE breaking and require an ADR:

- Removing any variant from any enum.
- Removing any field from any struct or proto message.
- Changing the type of any existing field.
- Changing any existing proto field number.
- Modifying the `MONOCLE_ABI_VERSION` constant.
- Modifying the `detect`, `state_file_path`, `read_state`, `subscribe`,
  `display_name`, `matches`, or `abi_version` method signatures on `FactoryAdapter`.
- Adding a non-default method to `FactoryAdapter` (breaks existing impls).

Phase 2–4 work that needs to extend Phase 1 contracts proceeds by:

1. Adding new fields via `#[non_exhaustive]` struct extension or proto field
   addition in the reserved range.
2. Adding new traits or types alongside Phase 1's (parallel extension, not
   modification).
3. NEVER modifying the existing surface of any item listed in this artifact.

---

## §Trace

Resolves FC-02, FC-03, FC-04 (CRITICAL), and FC-05 from the forward-compatibility
scan in commit 9618502. Human-authorized pre-Phase-1 lock-in.

v1.2.8 changes (round-56.1 F-R56-2 PG-5 historical-anchor framing — 2 sites):

- F-R56-2 RESOLVED (MEDIUM content — 2 sites in this file): §VsddFactoryAdapter Phase 1
  Implementation rustdoc and §BC-2.02.005 Traceability both cited
  `brief v1.4.6 §Success Criteria`. PG-5 §Historical-Anchor Framing Convention (codified
  in SS-conventions-anti-patterns.md v1.24 same burst): `brief v1.4.6` is neither current
  (brief is at v1.4.23) nor explicitly framed as historical. §Success Criteria is a stable
  brief section that has existed since v1.4.0; version qualifier adds no navigational value.
  Fix: option (c) — version qualifier dropped at both sites. Both now read
  `brief §Success Criteria`.

v1.2.7 changes (round-53.1 F-R53-adv-3/4 brief §-anchor mis-anchors):

- F-R53-adv-3 COMPANION (§Scope header): `HookType` rustdoc rationale cited `brief §Phase
  Plan` — no `## Phase Plan` heading exists in product-brief.md; the actual heading is
  `### Phase Plan Rationale`. Corrected to `brief §Scope (Phase 4 — PostToolUse revisit
  note)` using the PG-4 parenthetical-descriptor form, since the cited note is under the
  Phase 4 bold label within `## Scope`, not within `### Phase Plan Rationale`.

- F-R53-adv-4 RESOLVED (LOW — adversary finding R53): Two sites in `VsddFactoryAdapter`
  Phase 1 Implementation rustdoc and §BC-2.02.005 Traceability cited
  `brief v1.4.6 §Phase 1 Success Criteria`. Brief has no `## Phase 1 Success Criteria`
  heading; the actual heading is `## Success Criteria`. The Phase 1 row is unambiguous
  (brief §Phase 2 Exit Criteria is a separate heading). Corrected to `brief §Success
  Criteria` at both sites. PG-4 §-heading-existence compliance restored.

v1.2.6 changes (round-52.1 PG-3-TRACE-NEW-ENTRY sweep):

- F-R52-cons-1 COMPANION (PG-3-TRACE-NEW-ENTRY sweep): §Trace v1.2.5 entry referenced
  `§FactoryAdapter Trait rustdoc at L487` — bare intra-doc L-number in §Trace prose, a PG-3
  violation. The `at L487` token dropped; reference updated to `§FactoryAdapter Trait
  §Trait Signature rustdoc` using the actual `### Trait Signature` heading as position-free
  navigation anchor. The section heading is sufficient for reader navigation without a line
  number pinpoint.

v1.2.5 changes (round-51.1 PG-4 §-heading-existence sweep):

- F-R51-adv-1 COMPANION (PG-4 sweep): §FactoryAdapter Trait §Trait Signature rustdoc cited
  `SS-forward-compatibility.md §Analysis — Sealed trait §Item P3-1 — Verdict on Sealed`.
  `§Analysis — Sealed trait` has no heading in SS-forward-compatibility.md; it is a bold
  paragraph label within `#### Item P3-1`. The F-R48-adv-2 §Trace entry (v1.2.4) recorded
  the removal of the line-number pinpoint but inadvertently left `§Analysis — Sealed trait`
  as a residual prefix. Corrected to `§Item P3-1 — Verdict on Sealed` — the actual heading
  uniquely identified by prefix `Item P3-1`. Chain-resolvability confirmed: reader navigates
  to `#### Item P3-1` in SS-forward-compatibility.md and reads the Verdict on Sealed bold
  paragraph directly.

v1.2.4 changes (round-49 F-R48-adv-2 root-cause fix — PG-3 all-prose expansion):

- F-R48-adv-2 RESOLVED (LOW process-gap): Two cross-doc L-number pinpoints converted
  to position-free section references per expanded PG-3 rule (now covers all spec
  prose, not only §Trace):
  (1) §FactoryAdapter Trait rustdoc (§Phase 3 Forward-Compatibility Analysis inline):
  "Per SS-forward-compatibility.md §Analysis — Sealed trait (lines 95–97)" →
  "Per SS-forward-compatibility.md §Item P3-1 — Verdict on Sealed".
  (2) §BC-2.02.004 Traceability prose: "open extension trait per
  SS-forward-compatibility.md lines 95–97" → "per SS-forward-compatibility.md
  §Item P3-1 — Verdict on Sealed".

v1.2.3 fixes (round-20 fix F-R20-2):
- F-R20-2 RESOLVED (MEDIUM): `parse_frontmatter_field` lacked three of the four
  safety guards present in its sibling `parse_frontmatter_extra_fields`. The v1.2.2
  fix (F-R18-3) applied the guards only to `parse_frontmatter_extra_fields`. Consequences
  before this fix: (1) `phase: |` block-scalar marker returned `Some("|")`, violating the
  rustdoc contract that promises `None` for block scalars; (2) `current_cycle: ` (empty
  value) returned `Some("")`, semantically indistinct from an absent key and defeating
  BC-2.02.005's `Some(_) or None` assertion; (3) `awaiting: [a, b]` flow-list returned
  `Some("[a, b]")` rather than `None`. Fix: four guards added to `parse_frontmatter_field`
  matching `parse_frontmatter_extra_fields` exactly: (a) skip continuation lines (leading
  whitespace); (b) return `None` for empty value_str; (c) return `None` for values
  starting with `[` (flow-style list); (d) return `None` for values starting with `|`
  or `>` (block scalar). Rustdoc updated to enumerate all `None`-returning cases
  explicitly. `Some(_) or None` is now genuinely discriminating.

v1.2.2 fixes (round-19 fixes F-R18-2/F-R18-3):
- F-R18-2 RESOLVED: `VsddFactoryAdapter::new` rustdoc expanded with an explicit
  `# Validation` section documenting the no-validation-at-construction contract:
  `workspace_root` is accepted without existence or content checks; validation
  is deferred to `detect()` (returns `None` for invalid workspaces) and
  `read_state()` (returns `Err(...)` for missing or malformed STATE.md).
- F-R18-3 RESOLVED: Two bugs in the frontmatter parser functions corrected.
  Bug 1 (`parse_frontmatter_field`): naive `strip_prefix` + `trim` left surrounding
  YAML double- or single-quotes in the returned string (e.g., `awaiting: "round 18
  validation chain..."` returned the value WITH quotes). Fixed by adding explicit
  `strip_prefix('"')` / `strip_suffix('"')` and single-quote equivalents after
  the initial trim. Bug 2 (`parse_frontmatter_extra_fields`): rustdoc claimed
  flow-style lists and block scalars were "silently skipped" but the implementation
  stored them verbatim as `Value::String` (wrong). Fixed by adding explicit guards
  that `continue` on values starting with `[`, `|`, or `>`, and on lines with
  leading whitespace (block scalar continuation lines). Rustdoc updated to describe
  actual behavior: only single-line scalar key-value pairs are extracted; skipped
  types are enumerated; downstream code needing full YAML semantics should use
  `serde_yaml_ng::from_str` on the raw frontmatter.

v1.2.1 fixes (round-16 adversary N16-2/N16-5/N16-6/N16-8):
- N16-2 RESOLVED: `VsddFactoryAdapter::new(workspace_root: PathBuf) -> Self`
  public constructor added as an inherent `impl VsddFactoryAdapter` block.
  BC-2.02.005 updated to require the constructor and verify it.
- N16-5 RESOLVED: FactoryAdapter divergence from vision §FactoryAdapter documented
  in §FactoryAdapter vs vision divergence subsection (below), per human Q-16-5
  authorization retaining architect's design over vision sketch.
- N16-6 RESOLVED: `FactoryState.convergence` changed from `ConvergenceMetrics` to
  `Option<ConvergenceMetrics>`; `FactoryState.cycle` changed from `String` to
  `Option<String>`; `FactoryState.custom_fields` type changed from
  `HashMap<String, serde_json::Value>` to `HashMap<String, serde_yaml_ng::Value>`
  (project pin `serde_yaml_ng 0.10`). `read_state` updated: absent `current_cycle:`
  yields `cycle: None` (not `"unknown"`); absent §Session Resume Checkpoint yields
  `convergence: None` (not zero-round stub). `parse_frontmatter_extra_fields`
  return type updated to `serde_yaml_ng::Value`. BC-2.02.005 updated to require
  None semantics and the constructor. BC pre-staging table row for BC-2.02.005
  updated accordingly.
- N16-8 RESOLVED: BC footer changed from "9 BCs pre-staged" to "8 BCs authored in
  this artifact; BC-2.01.010 cross-referenced from SS-daemon-lifecycle.md."
  Grand total 15 = 8 (SS-core) + 4 (SS-daemon) + 3 (SS-engine) confirmed unchanged.

### FactoryAdapter vs vision divergence (intentional, human-authorized)

Vision §FactoryAdapter sketches four methods: `fn id() -> &'static str`,
`fn detect(&self, project_root: &Path) -> bool`,
`async fn read_state(&self, project_root: &Path) -> Result<FactoryState>`,
`async fn on_change(...)`.

The current spec departs from this sketch in the following ways, each an intentional
improvement authorized by the human per Q-16-5:

- `detect` is split into a `where Self: Sized` static method returning
  `Option<FactoryDetection>` (carries detection metadata — `display_name`,
  `workspace_root`, `state_file` — not just a bool) and a `matches(&self,
  workspace_root) -> bool` dyn-dispatch-safe method. The static variant is
  called at daemon startup for efficient type-driven dispatch; the instance
  variant is callable on `dyn FactoryAdapter` in Phase 3 plugin SDK scenarios.
- `read_state` is synchronous (`fn read_state(&self) -> Result<FactoryState, ...>`
  with no async, no `project_root` argument). Rationale: filesystem reads are
  synchronous OS operations. Callers in async contexts use
  `tokio::task::spawn_blocking`. This is more honest than an `async fn` that
  internally does a synchronous read; the sync signature makes the blocking
  nature explicit at the call site.
- `on_change` (callback pattern) is replaced by `subscribe(&self) ->
  Result<StateChangeStream, ...>` (stream pattern). A stream is more composable
  than a callback: it supports backpressure, cancellation, and multi-consumer fan-out
  without requiring the adapter to manage a callback registry.
- Accessors `state_file_path()`, `display_name()`, `abi_version()` are added to
  support TUI panel display and Phase 3 plugin SDK version negotiation without
  requiring the `FactoryDetection` struct to be re-queried.
- `id()` is absorbed into `display_name() -> &str` (non-static, instance method)
  because the display name already serves as the identity discriminant in all
  Phase 1 contexts; a separate `id()` method would be redundant.

The human authorized retaining this design per Q-16-5. Vision §FactoryAdapter is
preserved as historical intent but is not a binding contract for Phase 1 implementation.
Downstream code written against vision §FactoryAdapter sketch must be updated to use
the `FactoryAdapter` trait signature above; no compatibility shim is provided.

v1.2 fixes (human Q-15-1, round-14 adversary N2/N9):
- N2 RESOLVED: `FactoryAdapter` sealed pattern removed entirely. Trait bound changed
  from `Send + Sync + private::Sealed` to `Send + Sync + 'static`. `mod private`,
  `Sealed` marker, `__plugin_sdk_only` re-export, `compile_error!` guard, and
  `plugin-sdk-escape-hatch` feature references all removed from spec. Phase 3 plugin
  SDK implements the open trait directly — no escape hatch needed.
- N9 RESOLVED: `FactoryState.status` field doc now enumerates the five canonical
  valid values ("active", "blocked", "converged", "draft", "complete") that align
  with STATE.md frontmatter convention, with an explicit note that consumers must
  handle unknown values gracefully.
- BC-2.02.004 updated to remove `private::Sealed` reference.
- BC pre-staging count note updated to reflect cross-artifact total.

v1.1 fixes (adversary fresh-pass F-FC-C001/C002/C003/I001/I002/O002/O003/O004):
- F-FC-C001: exhaustive-enum forbidden list now unambiguously lists both
  `Phase1Permission` AND `ClaudeCodeTool`; refers to ADR-0004.
- F-FC-C002: `unsafe impl private::Sealed` replaced with feature-flag-gated
  `__plugin_sdk_only` escape hatch that compiles correctly.
- F-FC-C003: `read_state` parser now uses actual STATE.md field names:
  `phase:`, `status:`, `awaiting:`, `current_cycle:`.
- F-FC-I001: `FactoryState` restored to canonical 7-field vision struct with
  `BlockingIssue`, `BlockingSeverity`, `ConvergenceMetrics` supporting types.
  No `raw_content` field (user red-line: pure vision-7-field-only).
- F-FC-I002: All 5 HookEvent inner-variant structs fully specified; no placeholders.
- F-FC-O002: `matches(&self, workspace_root: &Path) -> bool` added to trait for
  dyn-dispatch. `detect` retains `where Self: Sized`.
- F-FC-O003: `parse_frontmatter_field` now anchors `---` to first line of document
  only, preventing false matches on markdown horizontal rules in body.
- F-FC-O004: The pre-split proto contract (retired; now BC-2.02.006 wire-format + BC-2.02.007 Rust surface);
  wire-vs-Rust conflation resolved. Historical old-ID record in BC-INDEX.md §Renumbering Map.

Cross-references:
- `SS-permissions-phase1.md` — `Phase1Permission` and `ClaudeCodeTool` definitions
- `ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md` — exemption rationale
- `SS-daemon-lifecycle.md` — `/status` endpoint BC-2.01.002 extended by
  BC-2.02.001 (`abi_version` field); lock-file `contract_version` (BC-2.01.010)
- `SS-engine-module.md` — EngineModule trait (companion artifact, same fix burst)
- `SS-deps-pin-manifest.md` — `prost 0.14` EXACT pin; `futures` caret pin;
  `serde_json` for tool_input fields
- `oq-research.md` OQ-03 (`VsddFactoryAdapter` as Phase 1 static bundle),
  OQ-07 (protobuf seams v1)

**§Trace v1.2.9** (2026-05-17T11:00:00Z) — Template compliance Dispatch 1:
- NORMATIVE: `document_type` corrected from `architecture-core-types` → `architecture-section`
  per audit §5 (SS-core-types-and-abi.md L1 verdict: FAIL; wrong document_type).
- NORMATIVE: `section` field corrected from `"core"` → `"core-types-and-abi"` (full section name
  per template; `"core"` was a partial identifier per audit §5 WARN).
- NORMATIVE: `subsystem` field corrected from `"core"` → `SS-02` (canonical SS-NN format per
  ARCH-INDEX.md Subsystem Registry; `"core"` pre-dated ARCH-INDEX existence).
- NORMATIVE: `traces_to` corrected to `architecture/ARCH-INDEX.md` (was long trace-history
  string; ARCH-INDEX.md now created in this dispatch).
- NORMATIVE: `timestamp` bumped to 2026-05-17T11:00:00Z (>= chain high-water 2026-05-17T10:30:00Z;
  SE-16d PASS).
- INFORMATIONAL: Version bump 1.2.8 → 1.2.9 records structural fix; no content changes.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §6 (SS-core-types-and-abi).
- SE-17g classification: all citations above NORMATIVE or INFORMATIONAL as labeled.

**§Trace v1.2.11** (2026-05-17T17:00:00Z) — F-R105-8 BC ID canonicalization (T-128h):
- NORMATIVE: All stale pre-renumbering BC IDs replaced with canonical BC-2.SS.NNN forms
  per BC-INDEX.md v1.1 Renumbering Map. Finding: F-R105-8 MED.
- SE-17c BEFORE: 39 lines / 46 occurrences with stale BC IDs (all old-form ABI/TYPES/FACTORY/PROTO/ENGINE prefixes).
- Replacements by canonical new ID (old-form identity in BC-INDEX §Renumbering Map):
  BC-2.01.002 [old: DAEMON-002]: 1 occurrence (cross-ref to SS-01)
  BC-2.01.003 [old: DAEMON-003]: 3 occurrences (cross-ref to SS-01)
  BC-2.01.007 [old: RING-001]: 1 occurrence (cross-ref to SS-01)
  BC-2.01.008 [old: AUTH-001]: 1 occurrence (cross-ref to SS-01)
  BC-2.01.009 [old: AUTH-002]: 1 occurrence (cross-ref to SS-01)
  BC-2.01.010 [old: LOCK-001]: 5 occurrences (cross-ref to SS-01)
  BC-2.02.001 [old: ABI-001]: 3 occurrences
  BC-2.02.002 [old: ABI-002]: 2 occurrences
  BC-2.02.003 [old: TYPES-001]: 2 occurrences
  BC-2.02.004 [old: FACTORY-001]: 4 occurrences
  BC-2.02.005 [old: FACTORY-002]: 9 occurrences
  BC-2.02.006 [old: PROTO-001a]: 4 occurrences
  BC-2.02.007 [old: PROTO-001b]: 4 occurrences
  BC-2.02.008 [old: PROTO-002]: 2 occurrences
  BC-2.03.001 [old: ENGINE-001]: 1 occurrence (cross-ref to SS-03)
  BC-2.03.003 [old: ENGINE-002-ERR]: 1 occurrence (cross-ref to SS-03)
  BC-2.03.002 [old: ENGINE-002]: 1 occurrence (cross-ref to SS-03)
  BC-2.03.004 [old: ENGINE-003]: 1 occurrence (cross-ref to SS-03)
- DISCOVERED: PROTO-001 (bare, pre-split) — 2 occurrences in historical §Trace prose.
  This ID is retired (split into BC-2.02.006 + BC-2.02.007 per F-FC-O004); it has no canonical
  new-form entry in BC-INDEX §Renumbering Map (only the a/b split variants are listed). Action:
  rewritten to descriptive prose removing the stale ID; historical record delegated to
  BC-INDEX.md §Renumbering Map (the append-only authority). Not a live BC reference.
- SE-17d AFTER: 0 lines with stale BC IDs in normative body (SE-17g PASS — see ARCH-INDEX §Trace v1.0.3).
- SE-17f PASS: sampled mapping verified — §ABI Version BC-2.02.001/BC-2.02.002, §Enum
  Extensibility BC-2.02.003, §FactoryAdapter BC-2.02.004/BC-2.02.005, §Prost Wire BC-2.02.006/BC-2.02.007.
- SE-16d PASS: 2026-05-17T17:00:00Z >= chain high-water 2026-05-17T16:30:00Z.
- Retired BC discovered: PROTO-001 (bare pre-split form, historical only). Surfaced above.
  Not in BC-INDEX Renumbering Map as standalone entry; treated as retired-by-split, no new ID needed.
