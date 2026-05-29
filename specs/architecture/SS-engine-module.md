---
document_type: architecture-section
level: L3
section: "engine-module"
slug: "engine-module-trait-stability"
subsystem: SS-03
version: "1.1.24"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-26T14:00:00Z
inputs: [research/domain-monocle-vision-synthesis.md, product-brief.md, SS-core-types-and-abi.md]
input-hash: "3734014"
traces_to: architecture/ARCH-INDEX.md
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
SS-forward-compatibility.md §Item P3-1 — Verdict on Sealed: "Do not apply the Sealed
pattern to `EngineModule` or `FactoryAdapter`." These traits exist to be implemented by
third-party code (that is their purpose); sealing would defeat Phase 3 plugin SDK
extensibility. The sealed-trait pattern applies only to internal traits that are
`pub` for technical reasons but must not be implemented by downstream code.
`EngineModule` is NOT in that category.

---

## §EngineModule Trait Signature

The signature below is the authoritative Phase 1 contract. The five trait methods have
different provenance with respect to `domain-monocle-vision-synthesis.md` §EngineModule
(original sketch; non-authoritative — this document supersedes it per CLAUDE.md §Architectural Authority):

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

/// Implemented by each AI coding harness adapter.
///
/// Phase 1 ships one built-in: `ClaudeCodeModule` (statically compiled into
/// `monocle-runtime`). Phase 4 adds `CodeMachineModule`. Phase 3+ allows
/// third-party WASM plugins implementing this trait via `monocle-plugin-sdk`.
///
/// The trait is OPEN — third-party crates may implement it. This is intentional:
/// it is the mechanism by which the Phase 3 plugin SDK exposes harness extensibility.
/// See SS-forward-compatibility.md §Item P3-1 — Verdict on Sealed.
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

impl EngineMetadata {
    /// Construct an `EngineMetadata` instance.
    ///
    /// All four fields are required — no field has a semantically valid default.
    /// `display_name` and `icon` are always known at module compilation time.
    /// `config_paths` must contain at least the primary config root for the
    /// engine; an empty Vec would silently disable config-path-dependent features
    /// (TUI display, DTU validator, transcript watcher). `hook_schema_version`
    /// must reflect the hook protocol version this engine speaks.
    ///
    /// Field order matches struct declaration order.
    ///
    /// # Rationale — constructor required for `#[non_exhaustive]`
    ///
    /// `EngineMetadata` carries `#[non_exhaustive]`. Per Rust E0639, struct literal
    /// construction (`EngineMetadata { display_name: ..., ... }`) is forbidden outside
    /// the defining crate (`monocle-core`). `ClaudeCodeModule` is defined in
    /// `monocle-runtime` (an external crate), so a constructor in `monocle-core` is
    /// the only legal construction path for downstream implementors. A builder pattern
    /// would add complexity without benefit for four semantically-required fields;
    /// this single constructor is the production-grade default.
    pub fn new(
        display_name: &'static str,
        icon: char,
        config_paths: Vec<PathBuf>,
        hook_schema_version: u32,
    ) -> Self {
        Self { display_name, icon, config_paths, hook_schema_version }
    }
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

impl ProcessSnapshot {
    /// Construct a `ProcessSnapshot` with the minimum viable signal set for `detect()`.
    ///
    /// `ppid`, `working_dir`, and `env` default to `None` / empty — they are not
    /// required for `detect()` to function. Callers that need those fields for
    /// `enrich()` (e.g., test fixtures that exercise the enrichment path) use
    /// `with_full_context` instead.
    ///
    /// # Field choices
    ///
    /// - `pid`: always available from OS process enumeration; required.
    /// - `exe_path`: the canonical `detect()` signal; `None` when process exits
    ///   before the path is resolved (daemon handles this gracefully).
    /// - `cmdline`: retained for `enrich()` (reading `CLAUDE_SESSION_ID`); always
    ///   available from OS enumeration (may be empty for exited processes).
    /// - `start_time_secs`: required for session deduplication across pid-reuse cycles.
    ///
    /// # Rationale — two constructors vs builder
    ///
    /// `ProcessSnapshot` has seven fields with a natural two-tier access pattern:
    /// detection needs only `pid`/`exe_path`/`cmdline`/`start_time_secs`; enrichment
    /// additionally needs `ppid`/`working_dir`/`env`. A builder pattern would add
    /// complexity (a `ProcessSnapshotBuilder` type and `.build()` step) for no
    /// ergonomic gain in a type that is always constructed from OS data where all
    /// fields are known at construction time. Two constructors with clear documentation
    /// of which tier each serves is the production-grade default.
    ///
    /// # Rationale — no `Default` impl
    ///
    /// `pid: 0` has no valid meaning (PID 0 is the idle/swapper process, never a
    /// Claude Code session). `start_time_secs: 0` is the Unix epoch, semantically
    /// wrong for any real process. A `Default` impl would produce a silently-invalid
    /// snapshot that passes `detect()` only by accident. No `Default` is provided.
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

    /// Construct a `ProcessSnapshot` with the full context needed by `enrich()`.
    ///
    /// Use this constructor in production process-scan code (where all fields are
    /// populated from the OS) and in test fixtures that exercise the enrichment path
    /// (e.g., BC-2.03.003 async half). For fixtures that only exercise `detect()`,
    /// `new` is sufficient.
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
/// Returned by `EngineModule::enrich`. May perform I/O (transcript path
/// resolution, harness config file reads) — it runs off the hot path.
///
/// **Phase 1 TUI fields (added for SS-06 Sessions Panel rendering, BC-2.06.005):**
/// `project_name`, `started_at`, `token_count`, and `cost_usd` are the Phase 1 set
/// of TUI display fields accumulated by the daemon and included in `SessionListUpdate`
/// IPC pushes. `project_name` is derived from the transcript directory name (parent
/// of `transcript_path`, if known). `token_count` and `cost_usd` are accumulated from
/// hook event metadata as events arrive. `phase_tag` was considered but removed —
/// it requires `FactoryAdapter` integration not available in Phase 1.
/// `uptime` is computable from `started_at` at render time; no dedicated field is
/// needed on the struct.
///
/// **Serde requirement:** All types that appear in `ServerToClient` or `ClientToServer`
/// IPC message variants MUST derive `Serialize, Deserialize` for IPC transport.
/// `EnrichedSession` is included in `SessionListUpdate` IPC pushes; both derives are
/// required. The `chrono` workspace dep MUST be declared with `features = ["serde"]`
/// so that `started_at: Option<chrono::DateTime<chrono::Utc>>` round-trips correctly.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    ///
    /// `None` means the session has been enriched but no hook events have been
    /// received yet (e.g., a session detected immediately after process spawn, before
    /// the first `SessionStart` hook POST arrives at the daemon). `Some(t)` carries
    /// the microseconds-since-epoch timestamp of the most recent hook event.
    ///
    /// **Contract for consumers:**
    /// - TUI session-list age column: display `"—"` for `None`; compute age as
    ///   `now - t` for `Some(t)`. MUST NOT treat `0` as a sentinel — that is the
    ///   Unix epoch (1970-01-01), semantically wrong for any real session event.
    /// - Idle-session reaper: treat `None` as "no events yet; do not reap based on
    ///   event timestamp." Reaping a `None`-epoch session requires a separate policy
    ///   (e.g., time-since-process-start). MUST NOT coerce `None` to `0` and compare
    ///   against a reap threshold — that would wrongly classify new sessions as 56
    ///   years idle.
    pub last_event_micros: Option<i64>,
    /// Human-readable project name derived from the transcript directory name.
    ///
    /// Typically the parent directory name of `transcript_path` (e.g., a transcript
    /// at `~/.claude/projects/my-project/session.jsonl` yields
    /// `project_name: Some("my-project")`). `None` when `transcript_path` is unknown
    /// at enrichment time. The TUI Sessions panel renders `"—"` for `None` in the
    /// Project column.
    pub project_name: Option<String>,
    /// UTC timestamp when this session started (when the `SessionStart` hook was first
    /// received by the daemon, or when the process was first detected).
    ///
    /// `None` at initial enrichment time if no `SessionStart` hook has arrived yet.
    /// The daemon sets this to `Some(t)` on receipt of the first `SessionStart` hook
    /// event for this session. The TUI computes `uptime = now - started_at` at render
    /// time; no separate `uptime` field is required on the struct.
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Cumulative token count accumulated from hook event metadata for this session.
    ///
    /// Starts at `0` at enrichment time. The daemon increments this field each time a
    /// hook event carrying token-usage metadata arrives for this session. The TUI
    /// Sessions panel renders this as a human-formatted string (e.g., `"142k"`).
    pub token_count: u64,
    /// Cumulative cost estimate in USD accumulated from hook event metadata.
    ///
    /// `None` when no cost metadata has been received (e.g., the harness does not
    /// emit cost data). `Some(v)` carries the running total cost in USD. The TUI
    /// Sessions panel renders `"$0.83"` for `Some(0.83)` and `"—"` for `None`.
    pub cost_usd: Option<f64>,
}

impl EnrichedSession {
    /// Construct an `EnrichedSession` instance.
    ///
    /// `transcript_path` and `config_path` are `Option<PathBuf>` because they may
    /// be legitimately unknown at enrichment time (e.g., a session whose config file
    /// has not been read yet passes `None`; the TUI handles `None` by displaying
    /// `"—"`).
    ///
    /// `last_event_micros` is `Option<i64>`. Pass `None` at initial enrichment time
    /// (no hook events received yet). The daemon updates this field to `Some(t)` when
    /// the first hook event for this session arrives. See field-level rustdoc for the
    /// full consumer contract.
    ///
    /// Field order matches struct declaration order.
    ///
    /// # Rationale — no `Default` impl
    ///
    /// An `EnrichedSession` with `session_id: ""` and `harness_type: ""` is
    /// semantically invalid — it would be indistinguishable from a session that
    /// failed enrichment. The TUI and transcript watcher consume `EnrichedSession`
    /// values as authoritative; a silently-wrong default would cause observable
    /// misbehavior (wrong session ID in display, wrong harness_type in routing).
    /// No `Default` is provided to prevent accidental use of the zero-value.
    ///
    /// # Rationale — constructor required for `#[non_exhaustive]`
    ///
    /// `EnrichedSession` carries `#[non_exhaustive]`. Per Rust E0639, struct literal
    /// construction is forbidden outside `monocle-core`. `ClaudeCodeModule::enrich`
    /// is in `monocle-runtime`, so this constructor is the only legal construction path.
    ///
    /// # Rationale — `Option<i64>` vs sentinel `0`
    ///
    /// `0i64` is the Unix epoch (1970-01-01T00:00:00Z). Using it as a sentinel for
    /// "no hook events yet" would cause the TUI age column to display "56 years idle"
    /// and would cause any idle-session reaper comparing `(now - last_event_micros)`
    /// against a threshold to wrongly classify new sessions as stale. The `Option`
    /// type makes the "not yet received" state unambiguous and enforces explicit
    /// handling at every consumer. This is the same reasoning that motivated
    /// `ProcessSnapshot::start_time_secs` to have no `Default` impl (see §Supporting
    /// Types — ProcessSnapshot rationale). The two cases require consistent treatment:
    /// there is no valid zero-value timestamp for any field on these structs.
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
            project_name: None,
            started_at: None,
            token_count: 0,
            cost_usd: None,
        }
    }
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
///
/// **Serde requirement:** `HookResponse` appears on the IPC wire (daemon → TUI push);
/// all types in IPC message variants MUST derive `Serialize, Deserialize`.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

impl HookResponse {
    /// Construct a `HookResponse` with the given decision.
    ///
    /// `redirect_url` and `diagnostic` default to `None`, which is the correct
    /// Phase 1 state: no federation redirect (Phase 4+ only), no diagnostic
    /// string unless the engine has one to surface.
    ///
    /// Phase 1 callers that only need a decision use `new` directly:
    /// ```rust
    /// HookResponse::new(HookDecision::Allow)
    /// ```
    ///
    /// Callers that need to attach a diagnostic or redirect chain builder methods:
    /// ```rust
    /// HookResponse::new(HookDecision::Allow)
    ///     .with_diagnostic("allow: no policy match")
    ///
    /// // Phase 4 federation redirect:
    /// HookResponse::new(HookDecision::Allow)
    ///     .with_redirect("http://peer-daemon:7892")
    ///     .with_diagnostic("federated to peer")
    /// ```
    ///
    /// # Rationale — constructor required for `#[non_exhaustive]`
    ///
    /// `HookResponse` carries `#[non_exhaustive]`. Per Rust E0639, struct literal
    /// construction (`HookResponse { decision: ..., redirect_url: None, diagnostic: None }`)
    /// is forbidden outside `monocle-core`. `ClaudeCodeModule::on_hook` is defined in
    /// `monocle-runtime`, so this constructor is the only legal construction path.
    ///
    /// A single required argument (`decision`) with `None` defaults for the two
    /// optional Phase 4 fields is the production-grade choice: `decision` is always
    /// known at construction; the optional fields are always `None` in Phase 1.
    ///
    /// # Rationale — builder methods vs pub-field mutation
    ///
    /// `HookResponse` fields are `pub` for forward-compat read access by consumers
    /// outside `monocle-core`. However, direct field assignment (`resp.diagnostic = ...`)
    /// forces callers to declare `let mut`, bypasses future validation, and leaks
    /// mutability into caller code. Builder methods (`with_diagnostic`, `with_redirect`)
    /// consume `self` by value and return `Self`, enabling immutable construction with
    /// no `let mut` required. This is the idiomatic Rust builder pattern for structs with
    /// a small number of optional fields. Phase 4 federation logic that needs `redirect_url`
    /// uses `.with_redirect(...)` — the builder method is the canonical setter.
    pub fn new(decision: HookDecision) -> Self {
        Self { decision, redirect_url: None, diagnostic: None }
    }

    /// Attach a human-readable diagnostic string to this response.
    ///
    /// The diagnostic may be surfaced in the TUI status bar when the session
    /// detail panel is open, and appears in daemon debug logs. Accepts anything
    /// that converts `Into<String>` (string literals, `String`, `&str`).
    ///
    /// ```rust
    /// HookResponse::new(HookDecision::Allow).with_diagnostic("allow: no policy match")
    /// ```
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }

    /// Set the federation redirect URL for this response.
    ///
    /// Used in Phase 4 federation scenarios where the hook should be forwarded to
    /// a peer daemon. Always `None` in Phase 1 (`ClaudeCodeModule` never sets this).
    /// Phase 4 federation logic sets this to the peer daemon's hook base URL before
    /// returning the response.
    ///
    /// ```rust
    /// HookResponse::new(HookDecision::Allow)
    ///     .with_redirect("http://127.0.0.1:7892")
    ///     .with_diagnostic("federated to peer")
    /// ```
    pub fn with_redirect(mut self, url: impl Into<String>) -> Self {
        self.redirect_url = Some(url.into());
        self
    }
}

/// The action the daemon takes in response to a hook event.
///
/// Phase 1 canonical 3-variant unit enum (Wave 3 simplification, F-D-02/F-D-03):
/// - `Deny { reason: String }` was renamed to `Block` (unit variant). The block reason
///   is carried in `HookResponse.diagnostic: Option<String>`, not inside the enum variant.
/// - `Defer { until: DeferUntil }` was simplified to `Defer` (unit variant). Phase 1
///   defers unconditionally to user decision. The `DeferUntil` sub-enum was dropped in
///   Wave 3 (F-D-03). Phase 2+ may reintroduce conditional deferral.
/// - `Modify { event: HookEvent }` was dropped from Phase 1 scope (F-D-02). Phase 2+
///   may reintroduce event rewriting.
///
/// All types appearing in IPC message variants MUST derive `Serialize, Deserialize`.
/// `HookDecision` is carried inside `HookResponse` which appears on the IPC wire;
/// the serde derives below are required for IPC transport.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HookDecision {
    /// Proceed; the operation is permitted.
    Allow,
    /// Reject the operation; the daemon MUST NOT proceed.
    ///
    /// The block reason (if any) is carried in `HookResponse.diagnostic: Option<String>`.
    Block,
    /// Park the decision; the daemon waits for the permission overlay resolution.
    ///
    /// Phase 1: unconditional deferral to user decision. Phase 2+ may reintroduce
    /// `DeferUntil` sub-enum for conditional deferral; `#[non_exhaustive]` ensures
    /// match arms remain exhaustive without a SemVer-major bump.
    Defer,
}

// NOTE: `DeferUntil` (Phase 2 planned) — not implemented in Phase 1.
// The sub-enum was dropped in Wave 3 (F-D-03). Phase 2+ may reintroduce it as:
//   pub enum DeferUntil { UserDecision, NextHook, Timeout(Duration) }
// Do NOT add it here until the Phase 2 story for conditional deferral ships.
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
    EnrichedSession, EngineMetadata, EngineModule,
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
        Ok(EngineMetadata::new(
            "Claude Code",
            '●',
            vec![
                claude_config_root.clone(),
                claude_config_root.with_extension("json"), // ~/.claude.json
            ],
            1,
        ))
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

        Ok(EnrichedSession::new(
            session_id,
            self.id().to_string(),
            transcript_path,
            Some(claude_config_root),
            SessionStatus::Active,
            None, // last_event_micros: None = no hook events received yet; daemon sets Some(t) on first hook
        ))
    }

    async fn on_hook(&self, _event: HookEvent) -> HookResponse {
        // Phase 1: default fail-open policy per BC-HOOK-018
        // (gene-source: any-context-lazyclaude; attested in SS-permissions-phase1.md §Trace).
        // Full permission-overlay dispatch implemented in Phase 1 story
        // for monocle-runtime hook_handler.
        HookResponse::new(HookDecision::Allow)
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
    /// `None` means Claude Code opens in `project_root` directly (no worktree isolation).
    pub worktree: Option<PathBuf>,
    /// Environment variable overrides for the spawned subprocess.
    /// Empty map means no overrides; Claude Code inherits the daemon's environment.
    pub env_overrides: HashMap<String, String>,
}

impl SpawnArgs {
    /// Construct a `SpawnArgs` for a session rooted at `project_root`.
    ///
    /// `worktree` defaults to `None` (no worktree isolation). Use `with_worktree`
    /// to set the worktree path for claude-squad-style isolation.
    /// `env_overrides` defaults to empty (inherit daemon environment).
    /// Use `with_env_override` to add per-session environment overrides.
    ///
    /// # Rationale — constructor required for `#[non_exhaustive]`
    ///
    /// `SpawnArgs` carries `#[non_exhaustive]`. Per Rust E0639, struct literal
    /// construction is forbidden outside `monocle-runtime`. Integration tests in
    /// `monocle-runtime/tests/` compile as separate `[[test]]` binaries that link
    /// `monocle-runtime` as a dependency — they are external crates from the
    /// library's perspective and cannot construct `SpawnArgs` via struct literal.
    /// This constructor is the only legal construction path for test fixtures and
    /// any future Phase 4 federation code that spawns sessions on peer daemons.
    ///
    /// # Rationale — builder methods for optional fields
    ///
    /// `SpawnArgs` has one required field (`project_root`) and two optional fields
    /// with valid defaults (`None` and empty map respectively). A single mandatory
    /// argument in `new` with `with_*` builder methods for optional configuration
    /// is the production-grade Rust pattern for structs with mixed required/optional
    /// fields where all optionals have valid no-op defaults.
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            worktree: None,
            env_overrides: HashMap::new(),
        }
    }

    /// Set the worktree path override for this spawn.
    ///
    /// Used for claude-squad-style worktree isolation: `claude` is invoked with
    /// `worktree` as its working directory rather than `project_root`.
    /// Phase 1 TUI "spawn in new worktree" action sets this field.
    pub fn with_worktree(mut self, worktree: PathBuf) -> Self {
        self.worktree = Some(worktree);
        self
    }

    /// Add a single environment variable override for the spawned subprocess.
    ///
    /// Overrides an environment variable in the subprocess's environment without
    /// replacing the full environment. Multiple calls accumulate: each call inserts
    /// or replaces one entry in `env_overrides`.
    ///
    /// Phase 4 federation may use this to inject session-routing variables
    /// (e.g., `MONOCLE_PEER_TOKEN`) into the subprocess environment.
    pub fn with_env_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_overrides.insert(key.into(), value.into());
        self
    }
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

impl SessionHandle {
    /// Construct a `SessionHandle` from the spawn result.
    ///
    /// All three fields are required — each is populated from the spawn result and
    /// cannot be deferred. `pid` comes from the spawned subprocess. `session_id`
    /// is read from the `CLAUDE_SESSION_ID` environment variable that Claude Code
    /// injects into its own hook scripts. `hook_base_url` is the base URL where
    /// Claude Code hook scripts POST events, copied from `ClaudeCodeModule::hook_base_url`.
    ///
    /// Field order matches struct declaration order.
    ///
    /// # Rationale — constructor required for `#[non_exhaustive]`
    ///
    /// `SessionHandle` carries `#[non_exhaustive]`. Integration tests in
    /// `monocle-runtime/tests/` are separate `[[test]]` binaries and cannot
    /// construct `SessionHandle` via struct literal (E0639). This constructor
    /// is the only legal construction path for test fixtures that exercise
    /// spawn-result processing (e.g., session-handle-to-enriched-session flow).
    pub fn new(pid: u32, session_id: String, hook_base_url: String) -> Self {
        Self { pid, session_id, hook_base_url }
    }
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

impl EngineVersion {
    /// Construct an `EngineVersion` from a successful preflight result.
    ///
    /// Both fields are required — they come from parsing `claude --version` output
    /// and resolving the binary path via `which claude`. Neither has a valid default.
    ///
    /// Field order matches struct declaration order.
    ///
    /// # Rationale — constructor required for `#[non_exhaustive]`
    ///
    /// `EngineVersion` carries `#[non_exhaustive]`. Integration tests in
    /// `monocle-runtime/tests/` are separate `[[test]]` binaries and cannot
    /// construct `EngineVersion` via struct literal (E0639). This constructor
    /// is the only legal construction path for test fixtures that mock preflight
    /// results (e.g., tests that assert a minimum version requirement rejection).
    pub fn new(version: String, binary_path: PathBuf) -> Self {
        Self { version, binary_path }
    }
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

**BC-2.03.001:** The `EngineModule` trait is defined in `monocle-core::engine` with
the exact vision-aligned signature in §EngineModule Trait Signature (methods: `id`,
`metadata`, `detect`, `enrich`, `on_hook`). Method return types are:
`id() -> &'static str`; `metadata() -> Result<EngineMetadata, EngineMetadataError>`;
`detect() -> bool`; `enrich() -> Result<EnrichedSession, EngineMetadataError>`;
`on_hook() -> HookResponse`. The `Result`-returning methods MUST NOT substitute a
default path for `EngineMetadataError::HomeUnresolvable`; daemon initialization MUST
fail fast with a diagnostic (no silent-fallback contract, CLAUDE.md SOUL #4).
Supporting types `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`, `SessionStatus`,
`HookResponse`, `HookDecision`, `EngineMetadataError` are co-located in
`monocle-core::engine`. `DeferUntil` is NOT present in Phase 1 — it was dropped in Wave 3
(F-D-03); Phase 2+ may reintroduce it. `HookDecision` is a 3-variant unit enum:
`Allow`, `Block`, `Defer` (Wave 3 simplification, F-D-02/F-D-03). The trait carries NO sealed bound.
`EnrichedSession::last_event_micros` is `Option<i64>`: `None` = no hook events received
yet; `Some(t)` = microseconds since epoch of most recent hook event. Consumers MUST
distinguish `None` from any numeric value — treating `0` as a sentinel is forbidden
(the Unix epoch 1970-01-01 is not a valid last-event timestamp for any real session).
Verification: `cargo check` with the Phase 1 workspace; `rustdoc` confirms all types
are publicly accessible and the trait has no `private::Sealed` supertrait.

**BC-2.03.002:** `ClaudeCodeModule` (defined in `monocle-runtime::engine::claude_code`)
implements `EngineModule`. A public `ClaudeCodeModule::new(hook_base_url: String) -> Self`
constructor is provided. `id()` returns the string `"claude-code"`. `detect()` returns
`true` for any process whose `exe_path` has a final basename component equal to `"claude"`
or `"claude.js"` (strict basename match on the resolved binary path; NOT a suffix match
on `cmdline[0]`, which would produce false positives for `claude-squad`, `claudio`, etc.).
Verification: unit test in `monocle-runtime/tests/engine_module.rs` constructs a module via
`ClaudeCodeModule::new("http://127.0.0.1:7891".into())`, asserts `module.id() == "claude-code"`,
and tests `detect()` with three synthetic `ProcessSnapshot` instances constructed via
`ProcessSnapshot::new(pid, exe_path, cmdline, start_time_secs)`:
(a) `ProcessSnapshot::new(12345, Some(PathBuf::from("/usr/local/bin/claude")), vec![], 1_700_000_000)` → asserts `detect()` returns `true`;
(b) `ProcessSnapshot::new(12346, Some(PathBuf::from("/usr/local/bin/claude-squad")), vec![], 1_700_000_000)` → asserts `detect()` returns `false`;
(c) `ProcessSnapshot::new(12347, None, vec!["claude".to_string()], 1_700_000_000)` → asserts `detect()` returns `false` (exe_path=None regardless of cmdline contents).
Note: `detect()` consults ONLY `exe_path`; `cmdline` is preserved for engine-specific use in `enrich()` (e.g., reading `CLAUDE_SESSION_ID`) but is NOT used for engine identification — this avoids false positives from processes such as `claude-squad`, `claudio`, and `claude-code-router` that may place `"claude"` in `cmdline[0]`.

**BC-2.03.003:** `ClaudeCodeModule::metadata()` and `ClaudeCodeModule::enrich()` MUST
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
   Construct a synthetic `ProcessSnapshot` using `ProcessSnapshot::new(...)` with the
   same pid and exe_path values used in the `detect()` test case (a), plus explicitly
   specified cmdline and start_time_secs. All seven fields are accounted for (four via
   `new`, three defaulted to `None`/empty by the constructor — satisfying F-R26-adv-5).
   In a separate `#[tokio::test]` (or within an `async` block):
   ```rust
   // Construct snapshot via ProcessSnapshot::new — struct literal is forbidden
   // outside monocle-core (E0639 applies to #[non_exhaustive] structs).
   // ProcessSnapshot::new sets ppid=None, working_dir=None, env=HashMap::new().
   let snapshot = ProcessSnapshot::new(
       12345,                                              // pid
       Some(PathBuf::from("/usr/local/bin/claude")),       // exe_path
       vec![],                                             // cmdline (empty for this test)
       1_700_000_000,                                      // start_time_secs
   );
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

The test is placed in `monocle-runtime/tests/engine_module.rs` alongside the BC-2.03.002
`detect()` tests. `temp-env` is a `[dev-dependencies]` entry with `features = ["async_closure"]`;
it does not appear in the production binary. The Phase 1 implementer MUST NOT use `#[serial]`
as a substitute for `temp-env` — serialisation mitigates the race but does not guarantee
cleanup on panic. The implementer MUST NOT use `tokio::runtime::Handle::current().block_on()`
inside the sync closure as a workaround — that pattern induces a nested-runtime panic under
`#[tokio::test]` and is explicitly forbidden.

**BC-2.03.004:** `ClaudeCodeModule::hook_paths()` returns exactly 5 entries, one per
`HookType` variant, with the exact path strings in §Struct-level inherent operations.
`ClaudeCodeModule::spawn()` and `ClaudeCodeModule::preflight()` are inherent methods on
the struct (NOT trait methods). The ABI version is read as
`monocle_core::MONOCLE_ABI_VERSION` at the call site; no `abi_version` method appears
on any trait. Verification: unit test in `monocle-runtime/tests/engine_module.rs`
asserts `module.hook_paths().len() == 5` with the exact path string for each `HookType`.

---

## §Cross-Crate Constructor Audit

Every `#[non_exhaustive]` struct in the monocle workspace requires a `pub fn new(...)` (or
equivalent constructor) when it is constructed from any crate OTHER than the defining crate.
This requirement arises from Rust E0639: struct literal construction (`Foo { field: val }`) is
forbidden outside the defining crate for `#[non_exhaustive]` types. The restriction applies
to integration test binaries (`monocle-runtime/tests/*.rs`) because each `tests/*.rs` file is
compiled as a separate `[[test]]` binary that LINKS the library as an external dependency — it
is NOT part of the library crate. Therefore E0639 applies to test fixtures exactly as it
applies to production code in downstream crates.

**Invariant:** Every architect spec change that adds `#[non_exhaustive]` to a struct MUST
update this table and add a constructor if any cross-crate construction site exists or is
anticipated. The table is committed atomically with the struct definition — never retroactively.

This table covers ALL `#[non_exhaustive]` structs in the monocle workspace, regardless of which
spec file defines them. It is the single authoritative list. Enums with `#[non_exhaustive]` are
governed by BC-2.02.003 and ADR-0004 (separate concern — match pattern completeness vs struct
literal construction); they do not appear here. `#[non_exhaustive]` on a struct that is NEVER
constructed via struct literal (only via serde deserialization) still needs the attribute for
forward-compat field extension — the "Constructor present?" column records the construction path.

**CI enforcement:** A semgrep rule (`monocle-non-exhaustive-struct-audit-completeness`) and a
CI Python script validate that every `#[non_exhaustive]`-annotated `pub struct` appearing in
the Rust source is listed between the HTML delimiters below. See SS-conventions-anti-patterns.md
§Semgrep Rules for the rule definition and §Semgrep Coverage Hardening for the fixture corpus.

### Audit Table (Phase 1 baseline)

<!-- BEGIN: Cross-Crate Constructor Audit Table -->
| Struct | Defining crate | Source spec | Construction path | Constructor present? | Notes |
|--------|---------------|-------------|-------------------|---------------------|-------|
| `EngineMetadata` | `monocle-core` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime::engine::claude_code` (metadata()); test fixtures | Yes (`new(display_name, icon, config_paths, hook_schema_version)`, v1.1.7) | All 4 fields required |
| `ProcessSnapshot` | `monocle-core` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime::engine::claude_code` (detect/enrich path); `monocle-runtime/tests/engine_module.rs` | Yes (two: `new(pid, exe_path, cmdline, start_time_secs)` + `with_full_context(...)`, v1.1.7) | Two-tier: detect-only vs enrich |
| `EnrichedSession` | `monocle-core` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime::engine::claude_code` (enrich()); `monocle-runtime/tests/` | Yes (`new(session_id, harness_type, transcript_path, config_path, status, last_event_micros: Option<i64>)`, v1.1.8) | `last_event_micros: Option<i64>` — None on initial enrich |
| `HookResponse` | `monocle-core` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime::engine::claude_code` (on_hook()); `monocle-runtime/tests/` | Yes (`new(decision)` + `.with_diagnostic()` + `.with_redirect()`, v1.1.8) | Builder pattern for optional fields |
| `SpawnArgs` | `monocle-runtime` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime/tests/` (tests compile as separate `[[test]]` binaries) | Yes (`new(project_root)` + `.with_worktree()` + `.with_env_override()`, v1.1.8) | Builder for optional fields |
| `SessionHandle` | `monocle-runtime` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime/tests/` (separate `[[test]]` binaries) | Yes (`new(pid, session_id, hook_base_url)`, v1.1.8) | All 3 fields required |
| `EngineVersion` | `monocle-runtime` | SS-engine-module.md | struct-literal (cross-crate): `monocle-runtime/tests/` (separate `[[test]]` binaries) | Yes (`new(version, binary_path)`, v1.1.8) | All 2 fields required |
| `HookEventRecord` | `monocle-runtime` | SS-daemon-lifecycle.md | struct-literal (cross-crate): `monocle-runtime/tests/jsonl_ring.rs` (separate `[[test]]` binary) | Yes (`new(session_id, timestamp_micros, pid, hook_type, tool_name, tool_input)`, v1.0.5); `RING_FORMAT_VERSION: u32 = 1` const | `format_version` always `RING_FORMAT_VERSION`; Phase 2 field evolution requires `#[non_exhaustive]` to avoid SemVer-major break |
| `SessionStartEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only: axum handlers call `serde_json::from_slice::<HookEvent>(&body)`; serde's `Deserialize` impl constructs internally within `monocle-core` — E0639 does not apply | No constructor required | Forward-compat: `#[non_exhaustive]` allows Phase 2+ field additions without breaking `Deserialize` impls in downstream crates. Enforce: if `Deserialize` is ever removed, re-audit. |
| `UserPromptSubmitEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only (same as `SessionStartEvent`) | No constructor required | See `SessionStartEvent` note |
| `PreToolUseEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only (same as `SessionStartEvent`) | No constructor required | See `SessionStartEvent` note |
| `NotificationEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only (same as `SessionStartEvent`) | No constructor required | See `SessionStartEvent` note |
| `StopEvent` | `monocle-core` | SS-core-types-and-abi.md | serde-deserialize-only (same as `SessionStartEvent`) | No constructor required | See `SessionStartEvent` note |
| `FactoryDetection` | `monocle-core` | SS-core-types-and-abi.md | intra-crate only (Phase 1): `VsddFactoryAdapter::detect()` constructs via struct literal WITHIN `monocle-core::factory` — E0639 does not apply. Phase 3 WASM adapters implementing `FactoryAdapter::detect()` will construct cross-crate. | No constructor yet — add before Phase 3 when first cross-crate construction site materializes | Production-grade note: `#[non_exhaustive]` on `FactoryDetection` allows Phase 3+ field additions (e.g., adapter priority, schema version) without breaking existing `detect()` callers. |
| `FactoryState` | `monocle-core` | SS-core-types-and-abi.md | intra-crate only (Phase 1): `VsddFactoryAdapter::read_state()` constructs via struct literal WITHIN `monocle-core::factory` — E0639 does not apply. Phase 2 body-parser in `monocle-workflow` will construct cross-crate. | No constructor yet — add before Phase 2 `monocle-workflow` body-parser implementation | `blocking_issues` and `convergence` are Phase 1 stubs (empty Vec / None) constructed inline. Phase 2 adds body parsing in `monocle-workflow` — that is a cross-crate construction site requiring a constructor. |
| `BlockingIssue` | `monocle-core` | SS-core-types-and-abi.md | intra-crate only (Phase 1): not constructed in Phase 1 (blocking_issues Vec is always empty). Phase 2 body-parser in `monocle-workflow` will construct cross-crate. | No constructor yet — add before Phase 2 `monocle-workflow` body-parser implementation | Phase 2 table parser populates `Vec<BlockingIssue>` — that is the first cross-crate construction site. |
| `ConvergenceMetrics` | `monocle-core` | SS-core-types-and-abi.md | intra-crate only (Phase 1): not constructed in Phase 1 (convergence is always None). Phase 2 body-parser in `monocle-workflow` will construct cross-crate. | No constructor yet — add before Phase 2 `monocle-workflow` body-parser implementation | Phase 2 §Session Resume Checkpoint parser populates `Option<ConvergenceMetrics>` — that is the first cross-crate construction site. |
| `App` | `monocle-tui` | SS-tui.md | struct-literal (cross-crate): `monocle-tui/tests/startup_connect.rs` (17+ call sites: lines 96, 188, 231, 275, 295, 310, 329, 351, 380, 403, 557, 637, 680, 743, 793, 841, 902, 1001 and others); `monocle-tui/tests/sessions_panel.rs` (6 call sites: lines 52, 61, 136, 408, 424, 501) — each `[[test]]` binary links monocle-tui as external; E0639 applies | Yes (`new(config: MonocleConfig) -> Self`, v1.1.23) | Top-level TUI state aggregator constructed once per binary entry point (main.rs + each integration test binary). Fields: `mode`, `config`, `sessions`, `drop_counter`, `overlay_stack`, `status_message`, `event_ring`. `#[non_exhaustive]` provides forward-compat for S-026 (overlay state additions), S-027 (event ring rendering fields), and S-028 (filter state) without breaking the constructor call sites. Sweep: `TransportEvent` (app.rs:44) is an `enum` — exempt per §Cross-Crate Constructor Audit introductory note (lines 1153-1154). No other `#[non_exhaustive] pub struct` present in monocle-tui src as of v1.1.23 sweep. |
| `EventBusHookEvent` | `monocle-runtime` | SS-engine-module.md (§S-017 types) | struct-literal (cross-crate): `monocle-runtime/tests/event_bus.rs:51` (1 call site); `monocle-runtime/tests/hook_routing_pre_tool_use.rs:103` (1 call site); `monocle-runtime/tests/hook_routing_notification.rs:144` (1 call site); `monocle-runtime/tests/daemon_start_sequence.rs:731` (1 call site) — each `[[test]]` binary links monocle-runtime as external; E0639 applies. Intra-crate production construction sites in `src/hooks/pre_tool_use.rs:349`, `src/hooks/notification.rs:186`, `src/hooks/stop_session_prompt.rs:313/427/572` also call `EventBusHookEvent::new`. | Yes (`new(payload: HookEvent, received_at: String) -> Self`, unknown — pre-existing per Pass 16 sweep) | Internal routing wrapper: pairs the deserialized hook payload with reception timestamp for event-bus transit (BC-2.04.001 step 5). `#[non_exhaustive]` allows Phase 2+ routing metadata fields (e.g., priority, origin daemon ID for federation) to be added without breaking the integration test construction sites. ADR-0006 criteria satisfied: (1) internal workspace scope, (2) field evolution tied to intentional protocol expansions (Phase 4 federation, not organic refactoring), (3) both required fields present as positional parameters. Earlier-wave struct hidden by check_audit_table.py false-green since commit 184f7d4; surfaced by devops-engineer fix at 390d04d. |
| `EngineModuleRegistry` | `monocle-runtime` | SS-engine-module.md (§S-017 types) | struct-literal (cross-crate): `monocle-runtime/tests/daemon_start_sequence.rs:951` (1 call site: `EngineModuleRegistry::new()`) — `[[test]]` binary links monocle-runtime as external; E0639 applies. Intra-crate production construction site in `src/lifecycle.rs:467` (`crate::types::EngineModuleRegistry::new()`). | Yes (`new() -> Self`, unknown — pre-existing per Pass 16 sweep) | Tracks which Phase 1 engine modules have been registered during daemon start sequence (BC-2.04.001 step 6). Also implements `Default` (delegates to `new()`), providing `EngineModuleRegistry::default()` as an alias. `#[non_exhaustive]` allows Phase 3 WASM plugin registry fields to be added when the plugin SDK ships without breaking the daemon_start_sequence test that calls `new()`. ADR-0006 criteria satisfied: (1) internal workspace scope, (2) field evolution tied to intentional Phase 3 plugin SDK expansion, (3) zero-value booleans are semantically meaningful defaults (both modules start unregistered). Earlier-wave struct hidden by check_audit_table.py false-green since commit 184f7d4; surfaced by devops-engineer fix at 390d04d. |
<!-- END: Cross-Crate Constructor Audit Table -->

**Serde-deserialize-only enforcement note:** The `Deserialize` derive on each HookEvent inner
struct is the construction gate. If `Deserialize` is ever removed from any of the five inner
structs (`SessionStartEvent`, `UserPromptSubmitEvent`, `PreToolUseEvent`, `NotificationEvent`,
`StopEvent`), the construction path changes from serde-internal to struct-literal — at which
point E0639 applies and a constructor MUST be added before removing `Deserialize`. Any PR that
removes `Deserialize` from these structs must include a constructor addition in the same commit.

**Future audit maintenance:** The CI `monocle-non-exhaustive-struct-audit-completeness` semgrep
rule (SS-conventions-anti-patterns.md §Semgrep Rules) scans for `#[non_exhaustive]` on `pub struct`
definitions and verifies each struct name appears in the delimiter-bounded block (the HTML BEGIN/END
marker pair defined in SS-conventions-anti-patterns.md §Semgrep Coverage Hardening that wraps the
audit table rows above). A Python CI script reads the delimiter-bounded block and compares the set
of struct names against semgrep matches; CI fails if any struct is missing from the table. This
prevents the class of defect that motivated F-R30-1 (audit table claimed completeness while 10
structs were missing).

---

## §Phase 1 PRD BC Pre-Staging

| BC ID | Description | Source Section |
|-------|-------------|----------------|
| BC-2.03.001 | `EngineModule` trait defined in `monocle-core::engine` with vision-exact signature (id/detect/on_hook) and no sealed bound; `metadata()` returns `Result<EngineMetadata, EngineMetadataError>` (vision-spirit-aligned elaboration); `enrich()` returns `Result<EnrichedSession, EngineMetadataError>` (vision-spirit-aligned elaboration); no-silent-fallback contract enforced on `HomeUnresolvable` | §EngineModule Trait Signature |
| BC-2.03.002 | `ClaudeCodeModule::new(hook_base_url)` public constructor; implements `EngineModule`; `id()` returns "claude-code"; `detect()` performs strict basename match on `exe_path` (not cmdline) | §Phase 1 Implementation |
| BC-2.03.003 | `ClaudeCodeModule::metadata()` and `enrich()` MUST return `Err(EngineMetadataError::HomeUnresolvable)` when `BaseDirs::new()` returns `None`; no-silent-fallback contract enforced via test in `monocle-runtime/tests/engine_module.rs` with `temp-env ^0.3` (features=["async_closure"]) — `with_vars` for sync `metadata()` half, `async_with_vars` for async `enrich()` half; clears HOME, USERPROFILE, HOMEDRIVE, HOMEPATH | §Behavioral Contracts (BC-2.03.003) |
| BC-2.03.004 | `ClaudeCodeModule::hook_paths()` returns 5-path mapping; spawn/preflight as inherent struct methods; ABI version read from const | §Struct-level inherent operations |

**Total: 4 BCs pre-staged.** Product-owner MUST use these exact IDs when formalizing
contracts with postconditions and verification harness stubs.

---

## §Trace

v1.1.15 changes (round-51.1 F-R51-adv-1 PG-4 §-heading-existence sweep):

- F-R51-adv-1 RESOLVED (MEDIUM — PG-4 §-heading-existence mis-anchor, two sites in this
  file): The R49 fix burst introduced `SS-permissions-phase1.md §Option A` at two sites —
  the on_hook inline comment body and the §Trace v1.1.14 F-R48-adv-3 entry. No heading
  named "Option A" exists in SS-permissions-phase1.md; the text appears only as inline prose
  within §Trace and §Status. Both sites corrected to `SS-permissions-phase1.md §Trace` —
  the actual heading under which the BC-HOOK-018 attestation and Q-A-permission-enum Option A
  resolution reside. Chain-resolvability of F-R48-adv-3 fix preserved: reader navigates to
  §Trace in SS-permissions-phase1.md and reads the attestation directly.

- PG-4 (§-heading-existence sweep companion): §HookEventRecord mis-anchor in §Trace
  v1.1.8 F-R28-4 entry corrected — `SS-daemon-lifecycle.md §HookEventRecord` had no heading;
  `HookEventRecord` struct is defined in prose under `### Drain (10-Second Timeout)`. Citation
  updated to `§Drain (HookEventRecord struct, introduced at v1.0.5)` per PG-4 correct form
  (closest enclosing heading + position-free description).

v1.1.14 changes (round-49 F-R48-adv-2 + F-R48-adv-3 root-cause fixes):

- F-R48-adv-2 RESOLVED (LOW process-gap — PG-3 all-prose expansion): Four cross-doc
  L-number pinpoints converted to position-free section references per the expanded
  PG-3 rule (now covers all spec prose, not only §Trace):
  (1) §Purpose: "SS-forward-compatibility.md lines 95–97" → "SS-forward-compatibility.md
  §Item P3-1 — Verdict on Sealed".
  (2) §EngineModule Trait Signature rustdoc (formerly L92): "(lines 95–97)" →
  position-free "(§Item P3-1 — Verdict on Sealed)".
  (3) §Trace v1.1 entry: "SS-forward-compatibility.md lines 95–97 veto honored" →
  "SS-forward-compatibility.md §Item P3-1 — Verdict on Sealed veto honored".
  (4) §Trace Cross-references block: "lines 95–97 (sealing veto)" →
  "§Item P3-1 — Verdict on Sealed (sealing veto)".
  Additionally: §EngineModule Trait Signature main-body prose (formerly L52): vision
  "lines 111–128" → position-free "§EngineModule" (non-authoritative parenthetical
  moved to same line for clarity). §Trace Cross-references vision entry updated to
  match: "lines 111–128" → position-free section reference.

- F-R48-adv-3 RESOLVED (LOW — phantom-ID prevention, Option A chosen): Inline code
  comment in §Phase 1 Implementation: `ClaudeCodeModule` (on_hook method body) cited
  "BC-HOOK-018" without a gene-source qualifier. Option A: added two-line inline
  gene-source qualifier "(gene-source: any-context-lazyclaude; attested in
  SS-permissions-phase1.md §Trace)" to make the citation chain-resolvable without
  per-line grep lookup. Option A chosen over B (soften phantom-ID grep) because: the
  line-grep prevention pattern is correct for unattested citations; BC-HOOK-018 is
  attested, so the pattern would not flag it once the chain-resolution qualifier is
  present; B adds grep complexity for no marginal benefit. Uniform application scope:
  BC-HOOK-018 is the only inline BC-HOOK-NNN citation in this file; no sibling sites
  required the same treatment. [§Option A citation corrected to §Trace in v1.1.15
  per F-R51-adv-1 PG-4 sweep — §Option A has no heading in SS-permissions-phase1.md;
  BC-HOOK-018 attestation resides in its §Trace heading.]

v1.1.13 changes (round-47.3 PG-3 §Trace-prose sub-rule — position-free conversion):

- §Trace v1.1.12 L-number pinpoints converted to position-free section references per
  PG-3 §Trace-prose authoring sub-rule (codified in SS-conventions-anti-patterns.md v1.17
  §Cross-Section Directional Reference Convention). Affected: `paragraph at L1137` → `§Future
  audit maintenance paragraph`; `delimiter block L1108-L1128` → `HTML-delimited §Cross-Crate
  Constructor Audit table block`. Also removed stale version pin in the PG-3 rule citation
  (v1.16 → bare section name, since the convention persists across versions). No behavioral
  change; navigation accuracy fix only.

v1.1.12 changes (round-48 PG-3 sweep directional fix):

- Directional typo in §Future audit maintenance RESOLVED (LOW — PG-3 sweep finding): the
  §Future audit maintenance paragraph described the HTML BEGIN/END delimiter pair "that wraps
  the audit table rows below." The HTML-delimited §Cross-Crate Constructor Audit table block
  is above that paragraph, not below. Corrected "below" to "above". No content change;
  navigational accuracy fix only. Root cause: the phrase was introduced when the paragraph
  immediately followed the table; subsequent additions between the table and this paragraph
  inverted the positional truth without updating the directional qualifier. Caught by the PG-3
  mandatory sweep (see SS-conventions-anti-patterns.md §Cross-Section Directional Reference
  Convention).

v1.1.11 changes (round-41 fixes F-R40-2 MEDIUM):
- F-R40-2 RESOLVED (MEDIUM — adversary finding): Two current-pointer citations in the v1.1.8
  §Trace block referenced `SS-daemon-lifecycle.md v1.0.5` without qualification. The phrasing
  "See SS-daemon-lifecycle.md v1.0.5" and "(v1.0.5)" in a trace entry describing F-R28-2 and
  F-R28-4 reads as a current pointer — directing readers to v1.0.5 to understand the current
  state of `HookEventRecord`. However, SS-daemon-lifecycle.md was bumped to v1.0.6 in a
  subsequent round (F-R30-2) which added `#[non_exhaustive]` to `HookEventRecord`. A reader
  consulting v1.0.5 (per the old citation) would see the struct definition without
  `#[non_exhaustive]`, missing a materially important attribute. Fix: both citations rewritten
  as historical pinpoints clearly distinguishing when HookEventRecord was first defined (v1.0.5)
  from what the current version adds (v1.0.6 adds `#[non_exhaustive]`). Historical narrative
  meaning preserved: F-R28-2 and F-R28-4 context is unchanged; the temporal precision is added.

v1.1.10 changes (round-35 fix F-R34-1 CRITICAL):
- F-R34-1 RESOLVED (CRITICAL — adversary finding): §Trace prose in the v1.1.9 entry (at
  the lines describing the HTML delimiter addition in F-R30-1) quoted the audit-table delimiter
  strings verbatim in backticks. The SS-conventions-anti-patterns.md v1.8 F-R34-1 fix specifies
  that the `check_audit_table.py` duplicate-delimiter detection MUST use line-anchored regex
  (not substring search) precisely to exclude prose mentions like these. However, as defense in
  depth, the convention also prohibits verbatim quoting of the delimiter strings in §Trace or any
  spec narrative — the line-anchored regex is the first defense; the no-verbatim-quoting convention
  is the second. Fix: the two verbatim delimiter quotes in the v1.1.9 §Trace entry and the
  abbreviated form in the F-R30-3 entry are replaced with name-references: "HTML BEGIN/END
  delimiter markers (defined in SS-conventions-anti-patterns.md §Semgrep Coverage Hardening)."
  Historical meaning is preserved — readers understand that HTML comment markers wrap the table;
  the verbatim strings are no longer embedded in the spec file.

v1.1.9 changes (round-30 fixes F-R30-1 HIGH / F-R30-3 MEDIUM):
- F-R30-1 RESOLVED (HIGH — adversary finding): §Cross-Crate Constructor Audit table claimed
  "every `#[non_exhaustive]` struct in the monocle workspace" but enumerated only 7 structs.
  Fresh grep across all spec files (`grep -rn "#\[non_exhaustive\]" .factory/specs/`) revealed
  17 `#[non_exhaustive]` structs across four spec documents. Missing structs:
  (1) `HookEventRecord` (SS-daemon-lifecycle.md, monocle-runtime) — had a constructor but was
  omitted from the table; now in row 8.
  (2) Five HookEvent inner structs (`SessionStartEvent`, `UserPromptSubmitEvent`, `PreToolUseEvent`,
  `NotificationEvent`, `StopEvent`) from SS-core-types-and-abi.md — previously in a separate
  "HookEvent Inner Struct Audit" sub-section; now merged into the main table (rows 9–13) with
  "serde-deserialize-only" construction path column value. The serde-deserialize-only reasoning
  is preserved in the "Serde-deserialize-only enforcement note" paragraph after the table.
  (3) Four factory structs from SS-core-types-and-abi.md (`FactoryDetection`, `FactoryState`,
  `BlockingIssue`, `ConvergenceMetrics`) — never appeared in any audit. Added as rows 14–17.
  Phase 1 construction is intra-crate only (within `monocle-core::factory`); no cross-crate
  construction in Phase 1, so no constructor required now. Phase 2 (`monocle-workflow` body
  parser for `blocking_issues` and `convergence`) and Phase 3 (WASM adapters implementing
  `FactoryAdapter::detect()`) will require constructors — the table notes record the trigger.
  Structure change: the separate "HookEvent Inner Struct Audit" sub-section is replaced by the
  "Serde-deserialize-only enforcement note" paragraph; all struct entries are now in one table.
  HTML BEGIN/END delimiter markers (defined in SS-conventions-anti-patterns.md §Semgrep Coverage
  Hardening) wrap the table rows for CI machine-parsing by the Step 3 Python script.
  Table column "Source spec" added to make the audit cross-document navigable.
  Table column "Construction path" replaces the merged "Cross-crate construction sites" +
  "`#[non_exhaustive]`?" columns; the latter was always "Yes" (the table lists only `#[non_exhaustive]`
  structs) and is now conveyed by the section title and invariant statement.
  Total structs in audit table: 17.
- F-R30-3 RESOLVED (MEDIUM process-gap — adversary finding): The audit-table invariant was a
  passive policy ("MUST update this table") with no machine enforcement. F-R30-1 demonstrated
  the policy was violated — 10 structs were missing while the table claimed completeness. Fix:
  (1) HTML BEGIN/END delimiter markers (defined in SS-conventions-anti-patterns.md §Semgrep
  Coverage Hardening) wrap the audit table, enabling a CI Python script to extract the declared
  struct names by parsing between the delimiters.
  (2) A new semgrep rule `monocle-non-exhaustive-struct-audit-completeness` is specified in
  SS-conventions-anti-patterns.md v1.6 §Semgrep Rules. The rule matches `#[non_exhaustive]` on
  `pub struct` definitions in monocle crate sources. The CI Python script (devops-engineer Phase 1
  deliverable) cross-references semgrep match output against the delimiter-bounded table to fail
  if any struct is absent. (3) A fixture file `semgrep-fixtures/non_exhaustive_struct.rs` is
  added to the POL-11 fixture corpus (SS-conventions-anti-patterns.md §Semgrep Coverage Hardening).
  Mechanism choice: semgrep + CI Python script over a Rust `syn`-based integration test. Rationale:
  semgrep is already in the CI pipeline (4 existing rules); adding a 5th rule keeps enforcement
  homogeneous. The Python script requires no compilation and fits in the existing two-step
  (fixture-corpus → production-scan) CI pattern. A `syn`-based test binary would add a new
  dev-dependency (`syn` with all-features), CI build time, and a new test crate — more overhead
  for the same coverage. The delimiter approach is language-agnostic and works even if the spec
  is the source of truth (which it is in pre-Phase-1).

v1.1.8 changes (round-29 fixes F-R28-1 HIGH / F-R28-2 HIGH / F-R28-3 MEDIUM / F-R28-5 LOW):
- F-R28-1 RESOLVED (HIGH — adversary finding): `EnrichedSession::last_event_micros` changed
  from `i64` to `Option<i64>`. The prior type accepted `0` as a construction argument in the
  `enrich()` call site comment, using Unix epoch (1970-01-01T00:00:00Z) as a sentinel for
  "no events yet" — the exact semantic-smell the architect rejected for `ProcessSnapshot`'s
  `start_time_secs` (no `Default` impl, `0` is semantically wrong for any real process). Fix:
  (1) `EnrichedSession::last_event_micros` field type changed to `Option<i64>` with expanded
  field-level rustdoc documenting the `None`/"no events yet" vs `Some(t)`/"last event time"
  contract and explicit consumer guidance (TUI display `"—"` for `None`; reaper must not
  coerce `None` to `0`). (2) `EnrichedSession::new(...)` `last_event_micros` parameter type
  changed to `Option<i64>`. (3) `enrich()` call site updated from `0` to `None` with a
  comment: "None = no hook events received yet; daemon sets Some(t) on first hook." (4)
  Constructor rustdoc updated with the Option rationale (same epoch-sentinel reasoning as
  ProcessSnapshot). (5) BC-2.03.001 updated to state the `Option<i64>` contract and
  sentinel-is-forbidden rule.
- F-R28-2 RESOLVED (HIGH — adversary finding): Three additional `#[non_exhaustive]` structs
  in `monocle-runtime` (`SpawnArgs`, `SessionHandle`, `EngineVersion`) lacked constructors.
  Integration tests in `monocle-runtime/tests/*.rs` compile as separate `[[test]]` binaries
  (external crates from the library's perspective); E0639 applies. Architect's round-27 audit
  claimed completeness for 4 structs but missed 3 additional structs in the SAME crate. Fix:
  (1) `SpawnArgs::new(project_root: PathBuf) -> Self` added with `with_worktree(PathBuf)` and
  `with_env_override(key, value)` builder methods — builder pattern because `project_root` is
  the only required field and both remaining fields have valid empty/None defaults. (2)
  `SessionHandle::new(pid: u32, session_id: String, hook_base_url: String) -> Self` added —
  single constructor because all 3 fields are always known at spawn time. (3)
  `EngineVersion::new(version: String, binary_path: PathBuf) -> Self` added — single
  constructor because both fields come from the preflight parse result. (4) A new §Cross-Crate
  Constructor Audit table added listing all 7 `#[non_exhaustive]` structs with their crate,
  construction sites, and constructor status. (5) HookEvent inner structs audited: serde-
  deserialize-only construction — no cross-crate struct literal, no constructor required.
  (6) Also defines `HookEventRecord` (see F-R28-4 below) — the ring buffer serialization
  struct referenced in SS-daemon-lifecycle.md BC-2.01.007 (introduced at SS-daemon-lifecycle.md
  v1.0.5; `#[non_exhaustive]` attribute added in v1.0.6 per F-R30-2).
- F-R28-3 RESOLVED (MEDIUM — adversary finding): `HookResponse` rustdoc documented the
  canonical setter pattern as pub-field mutation (`let mut resp = HookResponse::new(...);
  resp.diagnostic = Some(...)`), forcing `let mut` and bypassing encapsulation. Fix:
  `with_diagnostic(impl Into<String>) -> Self` and `with_redirect(impl Into<String>) -> Self`
  builder methods added to `impl HookResponse`. Both consume `self` by value and return
  `Self` (consuming builder pattern — no `&mut self`). The rustdoc now shows the builder-
  chain form: `HookResponse::new(decision).with_diagnostic("...")`. The pub-field mutation
  example is removed from the rustdoc; pub visibility is retained for read access by
  consumers outside `monocle-core`, but the WRITE path is the builder method.
- F-R28-4 RESOLVED (MEDIUM — folded into F-R28-2 above): `HookEventRecord` was first defined in
  SS-daemon-lifecycle.md §Drain (HookEventRecord struct, introduced at v1.0.5). The type is
  the concrete struct pushed onto the JSONL ring buffer, and its definition is in the
  daemon-lifecycle spec because the ring buffer is a daemon-lifecycle artifact.
  Cross-reference: §Behavioral Contracts BC-2.01.007 in SS-daemon-lifecycle.md now references
  `HookEventRecord` via a defined type, not a phantom reference.
- F-R28-5 RESOLVED (LOW — adversary finding): The v1.1.5 trace block had a supersession
  annotation incorrectly implying its content was no longer applicable. v1.1.5's sole change
  (adding BC-2.03.003 to the Pre-Staging table) is still current and correct in v1.1.8.
  Fix: annotation rewritten to "v1.1.5 content remains current; subsequent versions add new
  content but do not supersede this entry."

v1.1.7 changes (round-27 fixes F-R26-adv-1 CRITICAL / F-R26-adv-5 LOW / F-R26-2 MEDIUM):
- F-R26-adv-1 RESOLVED (CRITICAL — adversary finding): `EngineMetadata`, `ProcessSnapshot`,
  `EnrichedSession`, and `HookResponse` are all `#[non_exhaustive]` structs defined in
  `monocle-core`. The production code in `ClaudeCodeModule` (in `monocle-runtime`, an
  external crate from `monocle-core`'s perspective) previously constructed all four via
  struct literal syntax — forbidden by Rust E0639: "`#[non_exhaustive]` prevents struct
  literal construction outside the defining crate." An implementer following the prior spec
  would hit E0639 compile errors on first `cargo build`. Fix: added `pub fn new(...)` and
  (for `ProcessSnapshot`) `pub fn with_full_context(...)` inherent constructors to all four
  structs within `monocle-core`. Updated all four call sites in `ClaudeCodeModule`:
  `metadata()` → `EngineMetadata::new(...)`; `enrich()` → `EnrichedSession::new(...)`;
  `on_hook()` → `HookResponse::new(HookDecision::Allow)`. `HookResponse::new` takes only
  `decision` (the one required field) and defaults `redirect_url` and `diagnostic` to
  `None` (correct Phase 1 state). Constructor design rationale for each struct is documented
  in their respective `impl` block rustdocs inline in §Supporting Types.
- F-R26-adv-5 RESOLVED (LOW — folded into F-R26-adv-1): The BC-2.03.003 async half
  test spec previously stated "construct a synthetic `ProcessSnapshot` with the same field
  values used in the detect() test cases (pid, exe_path, empty cmdline/env)" — an incomplete
  specification that left 3 of 7 field values unspecified and left the constructor form
  implicit. Fixed in the same edit: the test spec now shows the complete `ProcessSnapshot::new`
  call with all four positional arguments (`pid=12345`, `exe_path=Some(...)`, `cmdline=vec![]`,
  `start_time_secs=1_700_000_000`) and a comment noting that the three remaining fields
  (`ppid`, `working_dir`, `env`) are defaulted by the constructor.
- F-R26-2 RESOLVED (MEDIUM — consistency finding): v1.1.4 trace block referenced `temp-env
  ^0.2` and the XDG_* env-var list, both superseded in v1.1.6. v1.1.5 trace block was
  superseded by v1.1.6 (behavioral changes) and v1.1.7 (constructor changes). Fix: supersession
  annotations added to both the v1.1.4 and v1.1.5 trace entries identifying exactly what
  content was superseded and by which version.
- Additional in-scope finding: `HookResponse` audited as an additional E0639-affected struct
  (same `#[non_exhaustive]` cross-crate struct literal pattern; constructor added in same burst).

v1.1.6 changes (round-24 fixes F-R24-adv-1 + F-R24-adv-3):
- F-R24-adv-1 RESOLVED (MEDIUM — adversary finding): BC-2.03.003 verification block
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
**NOTE: v1.1.5 content remains current. Subsequent versions (v1.1.6, v1.1.7, v1.1.8) add
new content (test-spec corrections, constructors, builder methods) but do NOT supersede
v1.1.5's contribution, which was a cross-reference table consistency fix — the BC-2.03.003
Pre-Staging row added here is still present and correct in the current version.**
- BC-2.03.003 ADDED to §Phase 1 PRD BC Pre-Staging table (between BC-2.03.002 and
  BC-2.03.004, preserving numerical order). Prior commit 563b573 added this BC to
  §Behavioral Contracts but missed the pre-staging cross-reference table. Total updated
  from "3 BCs pre-staged" to "4 BCs pre-staged". No behavioral content changed; this is
  a cross-reference consistency fix only. Downstream documents updated in same burst:
  SS-core-types-and-abi.md (BC count 3→4 for engine BCs, global total 15→16),
  SS-forward-compatibility.md (BC-2.03.003 row added, table intro 15→16),
  product-brief.md (BC list and count updated 15→16).

v1.1.4 changes (round-22 fixes F-R22-1/F-R22-2/F-R22-3):
**NOTE: Superseded by v1.1.5 (BC-2.03.003 added to Pre-Staging table; cross-ref
consistency fix) and v1.1.6 (test-spec async/sync split; temp-env ^0.2 → ^0.3; env-var
list HOME+USERPROFILE+HOMEDRIVE+HOMEPATH; XDG_* removed). The v1.1.4 temp-env pin
(^0.2) and XDG_* env-var list in this entry are SUPERSEDED — implementers MUST follow
the v1.1.6 (and later v1.1.7) specifications.**
- F-R22-1/F-R22-2 RESOLVED (MEDIUM — adversary finding): §EngineModule Trait Signature
  opening paragraph previously claimed all five methods match the vision "exactly."
  This was imprecise: `id`, `detect`, and `on_hook` are vision-verbatim; `metadata`
  and `enrich` are vision-spirit-aligned elaborations (Result-wrapped return types).
  Fix: paragraph rewritten to enumerate the two provenance categories with explicit
  rationale. The vision is confirmed non-authoritative for this surface per CLAUDE.md
  §Architectural Authority. Implementers reading the vision sketch after this fix will
  see a clear statement that the Result signatures defined here supersede the infallible
  vision sketch. BC-2.03.001 Pre-Staging table row corrected: "(detect/enrich/on_hook)"
  changed to "(id/detect/on_hook)" for the vision-verbatim claim; `metadata()` and
  `enrich()` explicitly marked as vision-spirit-aligned elaborations. The vision document
  was NOT edited (per authority decision in this fix burst — the vision is human-approved
  verbatim; the architecture document is the canonical source for Phase 1 signatures).
- F-R22-3 RESOLVED (MEDIUM — adversary finding): BC-2.03.002 had no test specification
  for the `HomeUnresolvable` error paths in `metadata()` and `enrich()`. New sibling BC
  BC-2.03.003 added specifying the full test in
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
  operating on a wrong relative path. BC-2.03.001 updated to document the Result return
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
- F-R18-4 RESOLVED: BC-2.03.002 test case (c) reworded from
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
  constructor added to the inherent `impl ClaudeCodeModule` block. BC-2.03.002
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
  BC-2.03.002 verification updated with three cases: true positive, false positive
  guard (`claude-squad`), and `exe_path=None` guard.

v1.1 changes (human Q-15-1, round-14 adversary N1/N2):
- N1 RESOLVED: trait signature restored to vision-exact (detect/enrich/on_hook).
  Removed hook_paths/spawn/preflight/abi_version from trait surface; moved to
  `ClaudeCodeModule` inherent methods per vision authority.
- N2 RESOLVED: sealed pattern removed entirely. `EngineModule` trait is open.
  SS-forward-compatibility.md §Item P3-1 — Verdict on Sealed veto honored. No `mod private`, no
  `Sealed` supertrait, no `plugin-sdk-escape-hatch` feature flag on this trait.
- Supporting types fully specified: `EngineMetadata`, `ProcessSnapshot`,
  `EnrichedSession`, `SessionStatus`, `HookResponse`, `HookDecision`, `DeferUntil`.
- `SessionStatus`, `HookDecision`, `DeferUntil` carry `#[non_exhaustive]` per
  BC-2.02.003.
- BC-2.03.004 added to capture the inherent-methods contract.

Cross-references:
- `SS-core-types-and-abi.md` — `FactoryAdapter`, `HookType` enum, `HookEvent` variants
- `SS-daemon-lifecycle.md` — daemon startup sequence (ClaudeCodeModule::preflight
  called at step 1 before lock-file write)
- `SS-deps-pin-manifest.md` — `async-trait = "^0.1"` (Phase 1 Pin Manifest)
- Vision `domain-monocle-vision-synthesis.md` §EngineModule (original sketch; non-authoritative for Phase 1 signatures — this document supersedes it per CLAUDE.md §Architectural Authority)
- `SS-forward-compatibility.md` §Item P3-1 — Verdict on Sealed (sealing veto)

**§Trace v1.1.16** (2026-05-17T11:00:00Z) — Template compliance Dispatch 1:
- NORMATIVE: `subsystem` corrected from `"core"` → `SS-03` (canonical SS-NN format per
  ARCH-INDEX.md Subsystem Registry; `"core"` pre-dated ARCH-INDEX existence and was ambiguous
  between SS-02 and SS-03; SS-03 is the correct assignment for the EngineModule trait).
- NORMATIVE: `traces_to` corrected to `architecture/ARCH-INDEX.md` (was long trace-history
  string; ARCH-INDEX.md created in this dispatch).
- NORMATIVE: `timestamp` bumped to 2026-05-17T11:00:00Z (>= chain high-water 2026-05-17T10:30:00Z;
  SE-16d PASS).
- INFORMATIONAL: `document_type` already `architecture-section` — no change required (audit §7
  confirmed PASS for engine-module document_type).
- INFORMATIONAL: Version bump 1.1.15 → 1.1.16 records structural fix; no content changes.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §7 (SS-engine-module).
- SE-17g classification: all citations above NORMATIVE or INFORMATIONAL as labeled.

**§Trace v1.1.18** (2026-05-17T17:00:00Z) — F-R105-8 BC ID canonicalization (T-128h):
- NORMATIVE: All stale pre-renumbering BC IDs replaced with canonical BC-2.SS.NNN forms
  per BC-INDEX.md v1.1 §Renumbering Map (canonical at T-128h dispatch time
  2026-05-17T17:00:00Z; current canonical advances over time per F-R107-8
  historical-pin discipline).
  Finding: F-R105-8 MED.
- SE-17c BEFORE: 31 lines / 33 occurrences with stale BC IDs (all old-form ENGINE/RING/TYPES prefixes).
- Replacements by canonical new ID (old-form identity in BC-INDEX §Renumbering Map):
  BC-2.01.007 [old: RING-001]: 2 occurrences (cross-ref to SS-01)
  BC-2.02.003 [old: TYPES-001]: 2 occurrences (cross-ref to SS-02)
  BC-2.03.001 [old: ENGINE-001]: 5 occurrences
  BC-2.03.003 [old: ENGINE-002-ERR]: 12 occurrences (replaced before ENGINE-002 to avoid partial-match corruption)
  BC-2.03.002 [old: ENGINE-002]: 8 occurrences
  BC-2.03.004 [old: ENGINE-003]: 4 occurrences
- SE-17d AFTER: 0 lines with stale BC IDs in normative body (SE-17g PASS — see ARCH-INDEX §Trace v1.0.3).
- SE-17f PASS: sampled mapping verified — §EngineModule Trait Signature BC-2.03.001,
  §Phase 1 Implementation BC-2.03.002, BC-2.03.003 §Behavioral Contracts, BC-2.03.004 inherent methods.
- SE-16d PASS: 2026-05-17T17:00:00Z >= chain high-water 2026-05-17T16:30:00Z.
- No retired BCs discovered. All 31 stale-ID lines resolved to active BCs in BC-INDEX v1.1.

**§Trace v1.1.19** (2026-05-17T23:00:00Z) — F-R107-8 historical-pin clarification (Round 6D):
- INFORMATIONAL: §Trace v1.1.18 BC-INDEX cite `v1.1 Renumbering Map` expanded to explicit
  historical-pin form: `v1.1 §Renumbering Map (canonical at T-128h dispatch time
  2026-05-17T17:00:00Z; current canonical advances over time per F-R107-8
  historical-pin discipline)`.
  Purpose: prevent future fresh-context audits from re-flagging the historical pin as stale;
  the cite records what was canonical at the time of the T-128h canonicalization sweep,
  not a live version claim. Finding: F-R107-8 architect part.
- SE-16d PASS: 2026-05-17T23:00:00Z > chain high-water 2026-05-17T22:00:00Z (monotonic).

**§Trace v1.1.20** (2026-05-18T01:00:00Z) — F-R108-1 + F-R108-9 historical-pin + frontmatter correction (Round 7C):
- NORMATIVE (F-R108-1 CRITICAL): Two "current canonical BC-INDEX is v1.4 per F-R107-2 closure"
  occurrences removed per O-R108-3 codification. Live-version claims in historical-pin §Trace
  notes become false when BC-INDEX advances. Replaced with "current canonical advances over
  time per F-R107-8 historical-pin discipline" in §Trace v1.1.18 body (1 occurrence) and
  §Trace v1.1.19 historical-pin expansion prose (1 occurrence). This is a normative content
  change: two live-version claim strings were removed from the document body.
- NORMATIVE (F-R108-9 HIGH): frontmatter `timestamp` corrected from 2026-05-17T17:00:00Z to
  2026-05-18T01:00:00Z. Prior timestamp lagged the latest §Trace entry (v1.1.19 at 23:00:00Z);
  SE-16b violation. Version bump v1.1.19 → v1.1.20 applied in Round 8A (F-R109-1) to
  reconcile frontmatter with the §Trace version number this entry already claimed.
- SE-17c BEFORE: "current canonical BC-INDEX is v1.4 per F-R107-2 closure" (2 occurrences).
- SE-17c AFTER: "current canonical advances over time per F-R107-8 historical-pin discipline".
- SE-16d PASS: 2026-05-18T01:00:00Z > chain high-water 2026-05-17T23:00:00Z (monotonic).

**§Trace v1.1.20-R109** (2026-05-18T05:00:00Z) — F-R109-1 + F-R109-8 frontmatter version reconciliation (Round 8A):
- NORMATIVE (F-R109-1 CRITICAL): frontmatter `version` bumped from "1.1.19" to "1.1.20" to
  match the §Trace v1.1.20 entry already present in the document body since Round 7C. The Round
  7C dispatch wrote §Trace v1.1.20 but withheld the frontmatter bump per cross-dispatch
  coordination directive (targeting current PO 7B SS pin). This created a fabrication-class
  defect: frontmatter claimed v1.1.19 while §Trace body documented v1.1.20 as the current state.
- NORMATIVE (F-R109-8 HIGH): §Trace v1.1.20 body rewritten to remove "No version bump —
  content unchanged; timestamp-only correction" self-contradiction. The F-R108-1 removal of two
  live-version claim strings IS normative content change. The body now accurately states that the
  version bump v1.1.19 → v1.1.20 was applied in Round 8A to reconcile frontmatter with §Trace.
- SE-17c BEFORE: "No version bump — content unchanged; timestamp-only correction".
- SE-17c AFTER: "Version bump v1.1.19 → v1.1.20 applied in Round 8A (F-R109-1) to reconcile
  frontmatter with the §Trace version number this entry already claimed."
- SE-16d PASS: 2026-05-18T05:00:00Z > chain high-water 2026-05-18T01:00:00Z (monotonic; corrected from erroneous 2026-05-17T04:30:00Z per F-R110-1).

**§Trace v1.1.20-R110** (2026-05-18T05:30:00Z) — F-R110-1 timestamp correction (Round 9A):
- NORMATIVE (F-R110-1 CRITICAL): frontmatter `timestamp` and §Trace v1.1.20-R109 header corrected
  from "2026-05-17T04:30:00Z" to "2026-05-18T05:00:00Z". Round 8A used wrong date; SE-16d chain
  regressed below prior §Trace v1.1.20 entry at 2026-05-18T01:00:00Z. Arithmetic correction
  applied across all 5 affected files in parallel Round 9A burst.
- SE-16d PASS: 2026-05-18T05:30:00Z > chain high-water 2026-05-18T05:00:00Z (monotonic).

**§Trace v1.1.21** (2026-05-26T12:00:00Z) — F-P13-001: expand `EnrichedSession` with Phase 1 TUI fields:
- NORMATIVE (F-P13-001 CRITICAL): Added four new fields to `EnrichedSession` required by the
  SS-06 Sessions Panel (BC-2.06.005): `project_name: Option<String>`, `started_at: Option<chrono::DateTime<chrono::Utc>>`,
  `token_count: u64`, `cost_usd: Option<f64>`. Added doc-comment block on the struct explaining
  Phase 1 TUI field provenance and that `phase_tag` was excluded (requires FactoryAdapter, not
  available in Phase 1) and `uptime` is computed from `started_at` at render time (no field needed).
- NORMATIVE: `EnrichedSession::new` constructor updated to initialize the four new fields to their
  zero/None defaults (`project_name: None`, `started_at: None`, `token_count: 0`, `cost_usd: None`).
  The struct carries `#[non_exhaustive]`; the constructor is the only legal construction path for
  external crates, so the zero-default initialization is the correct pattern — callers set the
  fields via daemon update paths rather than at construction time.
- NORMATIVE (deps impact): `started_at: Option<chrono::DateTime<chrono::Utc>>` introduces `chrono`
  as a new direct dependency of `monocle-core`. The current `SS-deps-pin-manifest.md` Phase 1 Pin
  Manifest table lists `chrono 0.4` as a dependency of `monocle-runtime` only (dep graph edge
  `runtime → chrono`). The dep manifest must be updated to add `core → chrono` and expand the
  chrono row Role column to include: "Phase 1 TUI field `EnrichedSession::started_at`
  (`monocle-core/src/engine.rs`) for session uptime display (BC-2.06.005)." The feature flag
  `features = ["serde"]` should be added to the `chrono` workspace dep declaration so that
  `EnrichedSession` (which derives `serde::Serialize + serde::Deserialize`) can round-trip the
  `started_at` field over the IPC wire. This deps-manifest update is the responsibility of the
  architect role when next editing SS-deps-pin-manifest.md.
- SE-16d PASS: 2026-05-26T12:00:00Z > chain high-water 2026-05-18T05:30:00Z (monotonic).

**§Trace v1.1.22** (2026-05-26T14:00:00Z) — F-P14-001/003/005/006: HookDecision simplification + serde derives (Wave 3 drift repair):

- NORMATIVE (F-P14-001/005 HIGH): `HookDecision` enum replaced with the Wave 3 implementation
  shape. `Deny { reason: String }` → `Block` (unit variant); block reason is now carried in
  `HookResponse.diagnostic: Option<String>`. `Defer { until: DeferUntil }` → `Defer` (unit
  variant); Phase 1 defers unconditionally to user decision. `Modify { event: HookEvent }`
  removed (F-D-02 Phase 1 scope drop). Serde derives `serde::Serialize, serde::Deserialize`
  added to `HookDecision` (IPC wire requirement).

- NORMATIVE (F-P14-006 HIGH): `DeferUntil` enum definition removed from §Supporting Types.
  Replaced with a `// NOTE` comment documenting Phase 2+ planned reintroduction. Removed
  `DeferUntil` from BC-2.03.001 co-location list. Removed `std::time::Duration` import
  from the trait code block (was only needed for `DeferUntil::Timeout`). Removed `DeferUntil`
  from the ClaudeCodeModule use statement.

- NORMATIVE (F-P14-003 HIGH): `EnrichedSession` derive updated from `#[derive(Debug, Clone)]`
  to `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`. `HookResponse` derive
  updated to add `serde::Serialize, serde::Deserialize`. IPC wire requirement note added to
  both structs: "All types that appear in `ServerToClient` or `ClientToServer` message variants
  MUST derive `Serialize, Deserialize` for IPC transport." `SessionStatus` and `HookDecision`
  already carried serde derives (SessionStatus since v1.1.21 / implementation; HookDecision
  added in this burst).

- INFORMATIONAL: The implementation in `monocle-core/src/engine.rs` already reflects the
  Wave 3 simplified shape (3-variant unit `HookDecision`, no `DeferUntil`). The SS-06 TUI
  stories will add the four `EnrichedSession` TUI fields and serde derives to the implementation;
  the spec leads. `EngineMetadata` and `ProcessSnapshot` do not appear on the IPC wire in
  Phase 1 (they are daemon-internal); serde derives for these are deferred to the Phase 2+
  story that introduces IPC serialization of metadata payloads.

- SE-16d PASS: 2026-05-26T14:00:00Z > chain high-water 2026-05-26T12:00:00Z (monotonic).

**§Trace v1.1.23** (2026-05-28T00:00:00Z) — F-S025-ADV16-MED-001 closure: monocle-tui::App added to Cross-Crate Constructor Audit Table:

- NORMATIVE (F-S025-ADV16-MED-001 MED): Added `App` struct row to §Cross-Crate Constructor Audit
  Table. `App` is defined at `crates/monocle-tui/src/app.rs:123` with `#[non_exhaustive]` and
  `pub fn new(config: MonocleConfig) -> Self` at line 158. It is constructed cross-crate from 17+
  call sites in `monocle-tui/tests/startup_connect.rs` and 6 call sites in
  `monocle-tui/tests/sessions_panel.rs`. Each `[[test]]` binary links monocle-tui as an external
  crate — E0639 applies. ADR-0006 criteria satisfied: (1) internal workspace scope, (2) `App`
  is the top-level TUI state aggregator whose field set evolves with new stories (S-026 overlay
  state, S-027 event ring rendering, S-028 sessions filter) — not an organic refactoring target,
  (3) all required fields present as constructor initialization in the body. `#[non_exhaustive]`
  forward-compat rationale: prevents breakage at the 20+ test call sites when new optional fields
  are added to `App` in downstream stories.
- NORMATIVE (completeness sweep): `TransportEvent` (app.rs:44) is an `enum` — exempt per audit
  table introductory note (this doc lines 1153-1154) and ADR-0004/BC-2.02.003 match-pattern
  completeness governance. No other `#[non_exhaustive] pub struct` exists in monocle-tui src at
  the time of this sweep. Sweep scope: `grep -rn non_exhaustive crates/monocle-tui/src/` — three
  hits: TransportEvent (enum, exempt), App struct (now listed), and a comment at
  `ui/sessions_panel.rs:401` (not a type annotation).
- INFORMATIONAL: CI semgrep rule `monocle-non-exhaustive-struct-audit-completeness` coverage
  investigation routed to devops-engineer in parallel — outcome will be documented in a subsequent
  §Trace entry if a rule scope update is required.
- SE-16d PASS: 2026-05-28T00:00:00Z > chain high-water 2026-05-26T14:00:00Z (monotonic).

**§Trace v1.1.24** (2026-05-28T01:00:00Z) — F-S025-ADV16-MED-001 round-2 [process-gap] closure: added `EventBusHookEvent` and `EngineModuleRegistry` rows discovered via devops-engineer CI rule investigation:

- NORMATIVE (F-S025-ADV16-MED-001 round-2 MED): Added `EventBusHookEvent` and `EngineModuleRegistry`
  rows to §Cross-Crate Constructor Audit Table. Both structs are defined in
  `monocle-runtime/src/types.rs` (introduced in S-017, Wave 4) with `#[non_exhaustive]` and
  `pub fn new(...)` constructors. Both are constructed cross-crate from integration test binaries
  that link `monocle-runtime` as an external crate — E0639 applies at those call sites.
- NORMATIVE (process-gap root cause): Both structs were present since Wave 4 (S-017 delivery) but
  hidden by a false-green in `scripts/check_audit_table.py` since commit 184f7d4. The script's
  `parse_semgrep_json()` read struct names from `metavars.$NAME.abstract_content`, but semgrep OSS
  1.156.0 (CI version) does not populate metavars for `pattern-either` rules. Result: the script
  silently returned an empty struct set → "0 production structs declared — PASS" regardless of
  how many `#[non_exhaustive]` structs existed. Devops-engineer fix at commit 390d04d added a
  message-field regex fallback and a safety assertion, which correctly failed with 2 of 3 missing
  rows (App already added in v1.1.23 in the same PR cycle).
- NORMATIVE (completeness sweep — v1.1.24 scope): Wide grep `non_exhaustive` across all workspace
  crates confirms exactly 3 `#[non_exhaustive] pub struct` instances existed during the
  S-025 PR cycle: `App` (monocle-tui, added v1.1.23), `EventBusHookEvent` (monocle-runtime, added
  this entry), `EngineModuleRegistry` (monocle-runtime, added this entry). The grep also surfaced
  `HookEventRecord` in `monocle-ipc/src/types.rs` — this struct exists in the audit table already
  (row with `monocle-runtime` origin pre-relocation), so the table is not missing a row; however
  the crate column for `HookEventRecord` is stale (shows `monocle-runtime`; correct is
  `monocle-ipc` post S-022 relocation). This stale-crate correction is a separate finding — not a
  missing row — and is outside the scope of this burst. Routing: architect or consistency-validator
  in a future story.
- NORMATIVE: Total `#[non_exhaustive] pub struct` count in workspace: `App`, `EventBusHookEvent`,
  `EngineModuleRegistry`, and the 5 HookEvent inner structs + `EngineMetadata`, `ProcessSnapshot`,
  `EnrichedSession`, `HookResponse`, `SpawnArgs`, `SessionHandle`, `EngineVersion`,
  `HookEventRecord`, `FactoryDetection`, `FactoryState`, `BlockingIssue`, `ConvergenceMetrics` =
  19 rows total now in the audit table (17 pre-S-025 + App + EventBusHookEvent + EngineModuleRegistry).
- SE-16d PASS: 2026-05-28T01:00:00Z > chain high-water 2026-05-28T00:00:00Z (monotonic).
