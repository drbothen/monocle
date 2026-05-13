---
document_type: architecture-core-types
level: L3
section: "core"
slug: "types-and-abi"
subsystem: "core"
version: "1.0"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-12T23:59:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-forward-compatibility.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-permissions-phase1.md
  - /Users/jmagady/Dev/monocle/.factory/planning/oq-research.md
  - /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md
input-hash: "[live-state]"
traces_to: "FC-02 + FC-03 + FC-04 + FC-05 from forward-compat scan 9618502; human authorization to lock pre-Phase-1"
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

**BC-ABI-001:** Every monocle binary exposes `abi_version: 1` in the `/status`
JSON response body. The field is present and equals `MONOCLE_ABI_VERSION` as
compiled into that binary. Verification: integration test asserts
`GET /status | jq .abi_version == 1`.

**BC-ABI-002:** `monocle-core` exports `MONOCLE_ABI_VERSION` as a `pub const u32`
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

Rationale: Phase 4 brief §Phase Plan notes "revisit PostToolUse endpoint need at
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

#### `Phase1Permission` — Exemption

`Phase1Permission` (defined in `SS-permissions-phase1.md`) is **exhaustive by
explicit design**. `#[non_exhaustive]` is FORBIDDEN on this enum. Rationale:
the TUI permission dispatcher must handle every variant at compile time;
exhaustiveness is a compile-time correctness invariant. Phase 3 adds a categorically
distinct `monocle-plugin-sdk::PluginPermission` enum rather than extending
`Phase1Permission`. This exemption is documented in SS-permissions-phase1.md §Decision
and is not subject to the general non-exhaustive default established below.

### Non-Exhaustive Inner Structs

Event variant payload structs also carry `#[non_exhaustive]` to allow adding new
fields in future phases without breaking construction sites:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionStartEvent {
    pub cwd: String,
    pub transcript_path: String,
    pub session_id: String,
    pub pid: u32,
}

// ... similar for UserPromptSubmitEvent, PreToolUseEvent, NotificationEvent, StopEvent
```

Phase 2 may add fields to `NotificationEvent` (e.g., `parent_message_id` for
trigger-trace) without a breaking change, because `#[non_exhaustive]` prevents
exhaustive struct literal construction in downstream code.

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

**BC-TYPES-001:** Every `pub` enum in `monocle-core` carries `#[non_exhaustive]`
unless an ADR documents the exhaustiveness requirement. At Phase 1 PRD dispatch,
the exemptions are: `Phase1Permission` (ADR exemption per SS-permissions-phase1.md).
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
/// The fields here are the minimum required by the Phase 1 Workflow panel.
/// Phase 3 extends this via the WASM adapter API; fields are non-exhaustive
/// to allow extension without breaking Phase 1 consumers.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FactoryState {
    /// Current pipeline phase (e.g., "phase-1", "pre-phase-1-architecture").
    pub current_phase: String,
    /// Human-readable summary of the current pipeline status.
    pub status_summary: String,
    /// Raw content of the state file, retained for display in the TUI panel.
    pub raw_content: String,
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
/// The trait is sealed for Phase 1: only `monocle-core` defines implementations.
/// Phase 3 relaxes this via an `unsafe impl FactoryAdapter for SdkAdapter`
/// mechanism in `monocle-plugin-sdk`, documented in SS-permissions-phase3.md.
pub trait FactoryAdapter: Send + Sync + private::Sealed {
    /// Detect whether the project at `workspace_root` uses this factory pattern.
    ///
    /// Returns `Some(FactoryDetection)` if detected; `None` if this adapter does
    /// not recognize the workspace layout.
    ///
    /// This method is called once at daemon startup and at TUI attach time.
    /// It must be fast (no network I/O; filesystem stat only).
    fn detect(workspace_root: &Path) -> Option<FactoryDetection>
    where
        Self: Sized;

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
    /// Default implementation returns `crate::MONOCLE_ABI_VERSION`. Overriding
    /// this method is forbidden in Phase 1 implementations (the Sealed pattern
    /// prevents external impls; Phase 3 SDK adapters use the default).
    fn abi_version(&self) -> u32 {
        crate::MONOCLE_ABI_VERSION
    }
}

/// Sealing module — prevents external crates from implementing `FactoryAdapter`
/// in Phase 1. Phase 3 `monocle-plugin-sdk` exposes a controlled relaxation.
mod private {
    pub trait Sealed {}
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
/// brief v1.4.6 §Phase 1 Success Criteria).
pub struct VsddFactoryAdapter {
    workspace_root: PathBuf,
    state_file: PathBuf,
}

impl private::Sealed for VsddFactoryAdapter {}

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

    fn read_state(&self) -> Result<FactoryState, FactoryReadError> {
        let raw_content = std::fs::read_to_string(&self.state_file).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FactoryReadError::NotFound {
                    path: self.state_file.clone(),
                    source: e,
                }
            } else {
                FactoryReadError::Io(e)
            }
        })?;

        // Extract the current phase from the STATE.md frontmatter.
        // The state file uses YAML frontmatter delimited by `---` lines.
        let current_phase = parse_frontmatter_field(&raw_content, "current_phase")
            .unwrap_or_else(|| "unknown".to_string());
        let status_summary = parse_frontmatter_field(&raw_content, "status")
            .unwrap_or_else(|| "unknown".to_string());

        Ok(FactoryState {
            current_phase,
            status_summary,
            raw_content,
        })
    }

    fn subscribe(&self) -> Result<StateChangeStream, FactorySubscribeError> {
        // Phase 1: return an empty stream. Phase 3 activates `notify 8` here.
        Ok(Box::pin(futures::stream::empty()))
    }

    fn display_name(&self) -> &str {
        "VSDD Factory"
    }
}

/// Extract a scalar value from YAML frontmatter without a full YAML parse.
///
/// Scans for lines of the form `key: value` between the opening `---` and
/// the closing `---` delimiter. Returns the trimmed value string, or `None`
/// if the key is absent or the frontmatter block is malformed.
fn parse_frontmatter_field(content: &str, key: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let mut frontmatter_started = false;
    for line in content.lines() {
        if line.trim() == "---" {
            if !frontmatter_started {
                frontmatter_started = true;
                in_frontmatter = true;
                continue;
            } else {
                break; // End of frontmatter block.
            }
        }
        if in_frontmatter {
            if let Some(rest) = line.strip_prefix(&format!("{}: ", key)) {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}
```

### Sealed Pattern Relaxation in Phase 3

Phase 3 `monocle-plugin-sdk` introduces a controlled relaxation. The SDK exposes
a `SdkAdapter` wrapper type that implements `FactoryAdapter` for WASM-loaded adapters:

```rust
// monocle-plugin-sdk/src/adapter.rs (Phase 3 only — not in Phase 1 workspace)
//
// SAFETY: `SdkAdapter` wraps a WASM component that has been validated by the
// wasmtime 44 component model linker against the canonical `FactoryAdapter` WIT
// interface. ABI version compatibility is checked at load time via
// `monocle_core::MONOCLE_ABI_VERSION`. The adapter is `Send + Sync` because
// the WASM component model enforces single-threaded execution within the
// guest; the host-side `SdkAdapter` manages the wasmtime `Store` under an
// internal `Arc<Mutex<Store<WasiCtx>>>`.
unsafe impl monocle_core::factory::private::Sealed for SdkAdapter {}
```

This is the ONLY legitimate external impl. The `unsafe` keyword signals the
intentional bypass of the sealed pattern with a documented safety argument.

### Behavioral Contracts

**BC-FACTORY-001:** `FactoryAdapter` trait is defined in `monocle-core::factory`
with the exact signature above (including `StateChangeStream` type alias,
`FactoryDetection`, `FactoryState`, `FactoryReadError`, `FactorySubscribeError`
supporting types, and the `private::Sealed` bound). Verification: `cargo check`
with the Phase 1 workspace; `rustdoc` output confirms public trait surface.

**BC-FACTORY-002:** `VsddFactoryAdapter` implements `FactoryAdapter`. Its `detect`
method returns `Some(FactoryDetection)` when called against monocle's own workspace
root (the directory containing `.factory/STATE.md` with
`document_type: pipeline-state` frontmatter). This is the self-referential detection
test from brief v1.4.6 §Phase 1 Success Criteria. Verification: integration test
`monocle-core/tests/factory_self_referential.rs` calls
`VsddFactoryAdapter::detect(workspace_root)` with the monocle repository root as
`workspace_root`; asserts `Some(_)` is returned with `display_name == "VSDD Factory"`.

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
  string message           = 4; // Human-readable notification body (may be large; see BC-DAEMON-003).
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

**BC-PROTO-001:** Every `HookEnvelope` wire message has `schema_version` as field
number 1, with value `1` for all Phase 1-origin messages. Verification: prost-build
generates `schema_version: u32` as the first field in the generated `HookEnvelope`
Rust struct; a unit test in `monocle-proto/tests/schema_version.rs` constructs a
`HookEnvelope` and asserts `schema_version == 1`.

**BC-PROTO-002:** The Phase 1 `HookEnvelope` schema is the canonical wire
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
| BC-ABI-001 | Every monocle binary exposes `abi_version: 1` in `/status` response | §ABI Version Constant |
| BC-ABI-002 | `monocle-core` exports `MONOCLE_ABI_VERSION` as pub const at crate root | §ABI Version Constant |
| BC-TYPES-001 | Every pub enum in `monocle-core` carries `#[non_exhaustive]` unless ADR exempts it | §Enum Extensibility |
| BC-FACTORY-001 | `FactoryAdapter` trait defined in `monocle-core::factory` with the signature in this artifact | §FactoryAdapter Trait |
| BC-FACTORY-002 | `VsddFactoryAdapter` passes self-referential detection test against monocle's own `.factory/` | §FactoryAdapter Trait |
| BC-PROTO-001 | Every HookEnvelope wire message defines `schema_version = 1` as field number 1 | §Prost Wire Schemas |
| BC-PROTO-002 | Phase 1 HookEnvelope schema is canonical wire representation; Phase 4 validates `schema_version` before deserializing | §Prost Wire Schemas |

**Total: 7 BCs pre-staged.** The product-owner MUST NOT renumber these BCs during
PRD authoring; the IDs above are anchor identifiers that cross-references in this
artifact and in SS-forward-compatibility.md rely upon.

---

## §Forward Compatibility Guarantees

Any change to the contracts defined in this artifact is a BREAKING change requiring
an ADR. The following operations are explicitly NOT breaking and do not require an ADR:

- Adding a new variant to any `#[non_exhaustive]` enum (except `Phase1Permission`
  which is exhaustive by ADR-exemption).
- Adding a new field to any `#[non_exhaustive]` struct.
- Adding a new proto field with a field number in the Phase 4 reserved range (100–999)
  or Phase 5+ range (1000+).
- Adding a new method to `FactoryAdapter` with a default implementation.

The following operations ARE breaking and require an ADR:

- Removing any variant from any enum.
- Removing any field from any struct or proto message.
- Changing the type of any existing field.
- Changing any existing proto field number.
- Modifying the `MONOCLE_ABI_VERSION` constant.
- Modifying the `detect`, `state_file_path`, `read_state`, `subscribe`,
  `display_name`, or `abi_version` method signatures on `FactoryAdapter`.
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

Cross-references:
- `SS-permissions-phase1.md` — `Phase1Permission` exhaustiveness exemption
  (see §Enum Extensibility §Mandatory Non-Exhaustive Enums §Phase1Permission)
- `SS-daemon-lifecycle.md` — `/status` endpoint BC-DAEMON-002 extended by
  BC-ABI-001 (add `abi_version` field)
- `SS-deps-pin-manifest.md` — `prost 0.14` EXACT pin; `futures` crate for
  `StateChangeStream` (add to Phase 1 pin manifest: `futures = "^0.3"`,
  caret pin acceptable — no untrusted-input deserialization path)
- `oq-research.md` OQ-03 (`VsddFactoryAdapter` as Phase 1 static bundle),
  OQ-07 (protobuf seams v1)
