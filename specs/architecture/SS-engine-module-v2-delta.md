---
document_type: architecture-section-delta
level: L3
section: "engine-module-v2-delta"
subsystem: SS-03
version: "1.6.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T23:00:00Z
inputs:
  - specs/architecture/SS-engine-module.md
  - research/domain-monocle-vision-synthesis.md
  - specs/architecture/SS-session-manager.md
  - specs/architecture/adr/ADR-0009-native-session-host-process-model.md
input-hash: "cd35c15"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# EngineModule v2 Delta (SS-03 extension)

## Purpose

This document specifies the changes to `SS-engine-module.md` (SS-03) required by the
control-center pivot. It is a delta, not a replacement — all existing Phase-1 contracts
in SS-engine-module.md (v1.1.26) remain in effect. The implementer applies both documents.

**When SS-engine-module.md is updated to incorporate these changes, this delta document's
version becomes SUPERSEDED and should be marked as such in ARCH-INDEX.**

---

<a id="spawn_recipe-new-trait-method"></a>
## spawn_recipe() — new trait method

The following method is added to the `EngineModule` trait in `monocle-core/src/engine.rs`:

```rust
/// Return the recipe needed to spawn a session under monocle's daemon.
///
/// Default impl returns `Err(EngineError::UnsupportedOperation("spawn_recipe"))`.
/// Only engines that support monocle-controlled session spawning implement this method.
/// Phase 1: `ClaudeCodeModule` implements this. `CodeMachineModule` returns the default.
///
/// Lifecycle operations (spawn, kill, attach, detach, resize) live in `SessionManager`,
/// not on this trait. `EngineModule` provides the RECIPE only — what binary to run,
/// with what args and env, in what working directory.
fn spawn_recipe(
    &self,
    opts: &SpawnOptions,
) -> Result<SpawnRecipe, EngineError> {
    Err(EngineError::UnsupportedOperation("spawn_recipe"))
}
```

### SpawnRecipe and SpawnOptions types

These types are added to `monocle-core/src/engine.rs` alongside the trait:

```rust
/// The spawn recipe produced by an EngineModule.
/// SessionManager uses this to build the monocle-session-host command line.
/// All fields MUST be set by the implementing module (no Optional fields except where noted).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRecipe {
    /// Absolute path to the harness binary.
    pub binary: PathBuf,
    /// CLI arguments (e.g., ["--settings", "/tmp/monocle-hooks-abc.json"]).
    /// The hooks_settings_path from SpawnOptions MUST be passed here as --settings.
    pub args: Vec<String>,
    /// Environment variables to OVERLAY on top of the session-host process's inherited env.
    /// The session-host CommandBuilder inherits the session-host process env first, then
    /// overlays these fields. They do NOT replace the base env (PATH/HOME must be preserved).
    /// Keys present here will override any matching key in the inherited env.
    pub env: HashMap<String, String>,
    /// Working directory for the harness child process.
    /// Populated from SpawnOptions.worktree_root — the resolved git worktree path
    /// (or project_root if no worktree applies). See SpawnOptions.worktree_root for rules.
    /// NEVER hardcoded to project_root.
    pub cwd: PathBuf,
}

impl SpawnRecipe {
    /// ADR-0006 constructor: required because `SpawnRecipe` is `#[non_exhaustive]` and
    /// constructed cross-crate inside `ClaudeCodeModule::spawn_recipe()`, which lives in
    /// `monocle-runtime`. `SpawnRecipe` is defined in `monocle-core`; `monocle-runtime`
    /// depends on `monocle-core` as an external crate, so E0639 applies. All 4 fields are
    /// required positional parameters (no optional fields on `SpawnRecipe`).
    ///
    /// # Construction path
    /// - `monocle-runtime` (`src/engine/claude_code.rs`, `ClaudeCodeModule::spawn_recipe()`):
    ///   the ONLY production construction site. Daemon-internal; never transmitted over IPC.
    /// - `monocle-runtime/tests/`: integration test binaries that exercise `spawn_recipe()`
    ///   outcomes call `new(...)` for assertion fixtures.
    ///
    /// # ADR-0006 criteria
    /// (1) Internal workspace scope: `monocle-core` and `monocle-runtime` are both workspace
    ///     crates, never published to crates.io.
    /// (2) External protocol anchor: field additions (e.g., new harness CLI flags) arise from
    ///     Claude Code version bumps requiring coordinated BC revisions; not organic refactoring.
    /// (3) All 4 fields are required positional parameters.
    pub fn new(binary: PathBuf, args: Vec<String>, env: HashMap<String, String>, cwd: PathBuf) -> Self {
        Self { binary, args, env, cwd }
    }
}

/// Options passed from the TUI via `ClientToServer::SpawnSession { opts }` to the daemon,
/// and then from the daemon's IPC handler to `SessionManager::spawn_session(opts)` and
/// from there to `EngineModule::spawn_recipe(&opts)`.
///
/// I27-001 (Model A): `SpawnOptions` is the IPC wire type. `SpawnRecipe` is daemon-internal.
/// The TUI populates `project_root`, `worktree_root`, `harness_id`, `profile_id`, and
/// `ccr_base_url`. The daemon IPC handler fills `session_id` and `hooks_settings_path` on
/// receipt before calling `SessionManager::spawn_session(opts)`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnOptions {
    /// Project root directory (user-selected in the wizard; used for display grouping).
    pub project_root: PathBuf,
    /// Working directory for the harness child process (resolved git worktree root or
    /// project_root). Set by SessionCreation wizard Step 3 (WorktreeConfirm) per resolution
    /// rules in SS-session-manager.md §SpawnOptions.worktree_root. The EngineModule MUST
    /// use this as SpawnRecipe.cwd — NOT project_root.
    pub worktree_root: PathBuf,
    /// Harness identifier selected by the user (e.g., "claude-code", "codemachine").
    pub harness_id: String,
    /// Harness profile ID selected by the user in the SessionCreation wizard.
    pub profile_id: String,
    /// Pre-generated session UUID.
    pub session_id: String,
    /// Path where the daemon has already written the shared hooks-settings.json.
    /// The EngineModule MUST include "--settings <hooks_settings_path>" in the returned recipe args.
    pub hooks_settings_path: PathBuf,
    /// If CCR is detected and a base URL is configured, this carries the URL.
    /// The EngineModule MUST inject this as `ANTHROPIC_BASE_URL` in `env` if present.
    pub ccr_base_url: Option<String>,
}

impl SpawnOptions {
    /// ADR-0006 TUI-side constructor: required because `SpawnOptions` is `#[non_exhaustive]`
    /// and constructed cross-crate by `monocle-tui` when the user confirms a session in the
    /// SessionCreation wizard. The TUI populates exactly 5 fields; the daemon fills the
    /// remaining 2 (`session_id`, `hooks_settings_path`) upon IPC receipt via
    /// `with_daemon_fields()`. Per Rust E0639, struct-literal construction is forbidden
    /// outside the defining crate for `#[non_exhaustive]` types.
    ///
    /// The two daemon-owned fields are initialized to documented placeholder values:
    /// - `session_id: String::new()` — empty; always overwritten by daemon before use.
    /// - `hooks_settings_path: PathBuf::new()` — empty; always overwritten by daemon before use.
    /// These placeholders are never observable by production code because the daemon always
    /// calls `with_daemon_fields()` before passing `SpawnOptions` to `spawn_session()`.
    ///
    /// # Construction path
    /// - `monocle-tui` (`src/ui/session_creation.rs`): SessionCreation wizard Step 4
    ///   (Confirm) calls `SpawnOptions::for_spawn_request(...)` and sends the result in
    ///   `ClientToServer::SpawnSession { opts }`.
    /// - `monocle-ipc/tests/` and `monocle-tui/tests/`: integration test binaries call
    ///   `for_spawn_request(...)` for test fixture construction.
    ///
    /// # ADR-0006 criteria
    /// (1) Internal workspace scope: `monocle-core` is a workspace crate, never published.
    /// (2) External protocol anchor: `SpawnOptions` is the `ClientToServer::SpawnSession`
    ///     wire payload; field additions require coordinated BC revisions (I27-001 Model A).
    /// (3) All required fields are positional parameters; daemon-owned fields use documented
    ///     placeholder values (empty strings/paths) because they are ALWAYS overwritten before
    ///     production use — not arbitrary defaults that could silently propagate.
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
            // Daemon-owned fields: placeholder values; always overwritten by daemon
            // via with_daemon_fields() before spawn_session() is called.
            session_id: String::new(),
            hooks_settings_path: PathBuf::new(),
        }
    }

    /// ADR-0006 daemon-side consuming builder: fills the two daemon-owned fields on receipt
    /// of `ClientToServer::SpawnSession { opts }`. The daemon calls this immediately upon
    /// receipt, BEFORE passing the completed `SpawnOptions` to `spawn_session()`. This
    /// replaces the E0639-violating `SpawnOptions { session_id: ..., hooks_settings_path: ...,
    /// ..opts }` functional-update pattern (C30-001).
    ///
    /// # Why a consuming builder (not &mut self)?
    /// The IPC handler receives `opts` by value (from serde deserialization of the wire
    /// payload). A consuming builder avoids cloning and makes the "fill then pass" pattern
    /// natural: `let opts = opts.with_daemon_fields(uuid, path); spawn_session(opts).await`.
    ///
    /// # Construction path
    /// - `monocle-runtime` (daemon IPC handler, `src/ipc_handler.rs`): called immediately
    ///   on the deserialized `opts` from `ClientToServer::SpawnSession { opts }`.
    pub fn with_daemon_fields(mut self, session_id: String, hooks_settings_path: PathBuf) -> Self {
        self.session_id = session_id;
        self.hooks_settings_path = hooks_settings_path;
        self
    }
}
```

### EngineError (new in v1A)

`EngineError` is a **new type introduced by the v1A control-center pivot**. No base
`EngineError` exists in the current source (`crates/monocle-runtime/src/engine/claude_code.rs`
has `SpawnError`; `monocle-core/src/engine.rs` has `EngineMetadataError` and `PreflightError`).
This delta introduces `EngineError` as the return type for the new `spawn_recipe()` trait method.

**Canonical location:** `monocle_core::engine::EngineError` (co-located with `EngineModule` in
`monocle-core/src/engine.rs`). This is the correct owner because: (a) `EngineModule::spawn_recipe()`
is defined in `monocle-core`; (b) the `SessionError::EngineError` bridge in SS-session-manager.md
cites `monocle_core::engine::EngineError`; (c) keeping the error with its trait avoids a cross-crate
import cycle.

**Relationship to existing error types (no subsumption):**

| Error type | Location | Trait method | Status |
|------------|----------|-------------|--------|
| `EngineError` | `monocle-core::engine` | `spawn_recipe()` | NEW — introduced by v1A |
| `EngineMetadataError` | `monocle-core::engine` | `metadata()` + `enrich()` | Existing Phase 1 |
| `SpawnError` | `monocle-runtime::engine::claude_code` | `spawn()` (legacy) | Existing Phase 1 (OLD spawn path) |
| `PreflightError` | `monocle-runtime::engine::claude_code` | `preflight()` | Existing Phase 1 |

`EngineError` does NOT subsume `SpawnError`. They serve different paths: `SpawnError` is the
error type for the legacy `ClaudeCodeModule::spawn()` inherent method (Phase 1, pre-pivot, still
present for backward compat in Wave 7). `EngineError` is the error type for the new
`EngineModule::spawn_recipe()` TRAIT method (v1A control-center). An implementer seeing both
types should understand: `spawn()` → `SpawnError`; `spawn_recipe()` → `EngineError`. They
will coexist during the transition wave; `spawn()` may be deprecated in a future wave once
all callers migrate to the `spawn_recipe()` + `SessionManager` path.

**`#[non_exhaustive]` decision — YES (recommended):**
`EngineError` carries `#[non_exhaustive]` for Phase 3 forward-compatibility: future WASM engine
modules (Phase 3 plugin SDK) may return additional error variants from their `spawn_recipe()`
implementations (e.g., `WasmModuleError(String)`, `PluginPermissionDenied`). Without
`#[non_exhaustive]`, adding a variant would require a SemVer-major bump and break every
downstream match arm. With `#[non_exhaustive]`, callers need a `_ =>` arm — which is already
present in `session_error_to_code()` (the `_ => "invalid_request"` catch-all in the inner
`EngineError` match). The `#[non_exhaustive]` attribute on an ENUM does NOT require an ADR-0006
constructor row (variant construction within the defining crate `monocle-core` is unrestricted;
`#[non_exhaustive]` only blocks exhaustive pattern-matching downstream). `EngineError` therefore
does NOT appear in the §Cross-Crate Constructor Audit Table — it is an enum, not a struct, and
the audit table explicitly covers only structs (see SS-engine-module.md §Cross-Crate Constructor
Audit introductory note: "Enums with `#[non_exhaustive]` are governed by BC-2.02.003 and ADR-0004
(separate concern — match pattern completeness vs struct literal construction); they do not
appear here.").

**Wire-code count note (updated F-P50-001):**
The `ServerToClient::Error.code` WIRE CODE SET (SS-ipc.md §ServerToClient::Error taxonomy) grew
from 10 to 11 codes in v1.22.0 of SS-ipc.md (F-P44-IMP-001), then to 12 codes in v1.23.0
(F-P50-001: `"session_not_ready"` added). The `EngineError` enum has exactly 3
variants (Phase 1). The 12-code taxonomy is the IPC wire layer; `EngineError` is the daemon-internal
error representation. These are distinct concepts: `EngineError` produces 3 of the 12 wire codes when
unwrapped in `session_error_to_code()`:
- `BinaryNotFound` → `"binary_not_found"`
- `InvalidPath` → `"invalid_spawn_arg"`
- `UnsupportedOperation` → `"spawn_unsupported"` (F-P44-IMP-001 — was `"invalid_request"` catch-all)

Unknown truly-novel future variants → `"invalid_request"` mandatory forward-compat fallback.

**Complete canonical declaration:**

```rust
/// Error returned by `EngineModule::spawn_recipe()`.
///
/// Introduced by the v1A control-center pivot. Distinct from:
/// - `EngineMetadataError` — returned by `metadata()` and `enrich()`.
/// - `SpawnError` — returned by the legacy `ClaudeCodeModule::spawn()` inherent method.
/// - `PreflightError` — returned by `preflight()`.
///
/// `#[non_exhaustive]` for Phase 3 forward-compatibility: future WASM engine modules
/// may return additional error variants. Downstream match arms MUST have a `_ =>` catch-all.
/// The `session_error_to_code()` inner match in `monocle-runtime` already satisfies this.
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum EngineError {
    /// The operation requested via `spawn_recipe()` is not supported by this engine module.
    /// Default impl returns this variant with `"spawn_recipe"` as the operation name.
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(&'static str),

    /// The harness binary could not be located on PATH.
    /// Produced when `which::which("claude")` (or equivalent) fails.
    /// RESERVED for binary-not-found-on-PATH only — do not use for argument validation failures.
    #[error("harness binary not found: {0}")]
    BinaryNotFound(String),

    /// A structurally invalid argument was supplied to `spawn_recipe()`.
    ///
    /// Detected via a TWO-PRONGED explicit check in `spawn_recipe()` step 2:
    ///   1. `Path::to_str()` returns `None` → path is not valid UTF-8 → `InvalidPath`.
    ///   2. Explicit byte scan: `path_str.as_bytes().contains(&0)` → path contains an
    ///      embedded null byte → `InvalidPath`.
    ///
    /// The null-byte check MUST be explicit because null bytes (U+0000) ARE valid UTF-8;
    /// `Path::to_str()` returns `Some(...)` for null-containing paths, so without prong (2)
    /// a null-byte path would pass through `to_str()` and produce a `NulError` at
    /// `CString`/`execve` construction, mapping to `SessionError::SpawnFailed` with wire
    /// code `"spawn_failed"` — defeating the diagnostic-separation guarantee of BC-2.03.007
    /// Invariant 1 (`"invalid_spawn_arg"` vs `"spawn_failed"` must be categorically distinct).
    ///
    /// DISTINCT from `BinaryNotFound`: the harness binary may exist but the argument is
    /// structurally invalid and cannot be passed as a CLI arg.
    #[error("invalid argument: {0}")]
    InvalidPath(String),
}
```

<a id="semantic-contract"></a>
**Semantic contract:**
- `BinaryNotFound` is reserved exclusively for the case where `which::which("claude")`
  (or the equivalent for other harnesses) fails — i.e., the harness binary cannot be
  located on `PATH`.
- `InvalidPath` is used for structurally invalid arguments to `spawn_recipe()`. Detection
  uses an EXPLICIT TWO-PRONGED check in step 2 of the implementation spec:
  - **Prong 1 (non-UTF-8):** `opts.hooks_settings_path.to_str()` returns `None` → the path
    cannot be represented as a valid UTF-8 string → `InvalidPath` is returned immediately.
  - **Prong 2 (embedded null byte):** After prong 1 succeeds and yields `path_str: &str`,
    an explicit byte scan `path_str.as_bytes().contains(&0)` checks for embedded null bytes.
    If true → `InvalidPath` is returned.
  Prong 2 is MANDATORY because null bytes (U+0000) ARE valid UTF-8; `to_str()` returns
  `Some(...)` for null-containing paths. Without prong 2, a null-byte path silently escapes
  to `CString`/`execve`, producing a `NulError` → `SessionError::SpawnFailed` → wire code
  `"spawn_failed"`, which violates BC-2.03.007 Invariant 1 (diagnostic-separation guarantee).
- `UnsupportedOperation` is the default-impl sentinel: the default `spawn_recipe()` trait
  implementation returns `Err(EngineError::UnsupportedOperation("spawn_recipe"))`. Engine
  modules that do not support monocle-controlled spawning return this variant. The ProfilePicker
  applies BEST-EFFORT capability filtering to avoid surfacing spawn-capable entries for
  harnesses known not to support it, but this filtering is not a hard invariant: a harness's
  spawn capability may be unknown until spawn-time, may change after profile selection, or a
  future/WASM engine may reach this path before filtering is established for it. The daemon
  MUST therefore surface a distinct, user-visible error (`"spawn_unsupported"` wire code;
  fixed banner `"Session spawn not supported for this harness"`) when `UnsupportedOperation`
  occurs — EC-112 is a reachable defensive path (F-P44-IMP-001). See BC-2.03.008 PC-3.
- These three failure modes are categorically distinct. `BinaryNotFound` means "the harness
  is not installed"; `InvalidPath` means "the supplied configuration is invalid";
  `UnsupportedOperation` means "this engine does not implement spawn_recipe."

---

## ClaudeCodeModule::spawn_recipe() implementation spec

The implementation lives in `monocle-runtime/src/engine/claude_code.rs`.

```rust
impl EngineModule for ClaudeCodeModule {
    // ... existing methods unchanged ...

    fn spawn_recipe(&self, opts: &SpawnOptions) -> Result<SpawnRecipe, EngineError> {
        // 1. Locate the claude binary via which::which("claude")
        let binary = which::which("claude")
            .map_err(|_| EngineError::BinaryNotFound("claude".into()))?;

        // 2. Build args: --settings <hooks_settings_path>
        //
        // TWO-PRONGED InvalidPath guard (must be explicit — both conditions required):
        //
        // Prong 1: non-UTF-8 path. Path::to_str() returns None for non-UTF-8 byte sequences.
        let path_str = opts.hooks_settings_path
            .to_str()
            .ok_or_else(|| EngineError::InvalidPath(
                format!("hooks_settings_path is not valid UTF-8: {:?}", opts.hooks_settings_path)
            ))?;
        //
        // Prong 2: embedded null byte. U+0000 IS valid UTF-8, so to_str() returns Some(...)
        // for null-containing paths. Without this explicit scan, a null-byte path silently
        // reaches CString/execve, producing NulError → SessionError::SpawnFailed →
        // wire code "spawn_failed", violating BC-2.03.007 Invariant 1 (diagnostic separation).
        if path_str.as_bytes().contains(&0) {
            return Err(EngineError::InvalidPath(
                format!("hooks_settings_path contains an embedded null byte: {:?}", opts.hooks_settings_path)
            ));
        }
        let args = vec![
            "--settings".to_string(),
            path_str.to_string(),
        ];

        // 3. Build env: inject CCR base URL if configured
        let mut env = HashMap::new();
        if let Some(ref url) = opts.ccr_base_url {
            env.insert("ANTHROPIC_BASE_URL".to_string(), url.clone());
        }
        // Inject MONOCLE_SESSION_ID so the session can be correlated in hook events
        env.insert("MONOCLE_SESSION_ID".to_string(), opts.session_id.clone());

        Ok(SpawnRecipe::new(
            binary,
            args,
            env,
            // Use the resolved worktree root (not project_root directly).
            // project_root == worktree_root when no git worktree is configured.
            opts.worktree_root.clone(),
        ))
    }
}
```

<a id="hook-auto-injection-invariant"></a>
**Hook auto-injection invariant:** The `--settings <hooks_settings_path>` argument is the
hook injection mechanism. The hooks-settings.json at that path contains:
```json
{
  "hooks": { /* per the 5-endpoint canonical set, all pointing at the daemon HTTP port */ },
  "lock": { "app": "monocle" }
}
```
The `lock.app = 'monocle'` filter ensures only monocle-launched sessions trigger the
monocle hook endpoint. No user manual configuration required.

---

## ProcessSnapshot.spawned_by_monocle field (detection reconciliation)

With the control-center pivot, sessions can be either:
- **Monocle-launched:** spawned by SessionManager with `--settings` arg and `MONOCLE_SESSION_ID` env.
- **Externally-launched:** the user ran `claude` elsewhere; monocle detects them via `detect()`.

The existing `detect()` method (SS-engine-module.md §ClaudeCodeModule Implementation — `detect()`)
uses `ProcessSnapshot.exe_path` basename matching. This remains correct and unchanged.

A new optional field is added to `EnrichedSession`:

```rust
pub struct EnrichedSession {
    // ... existing fields ...
    /// True if this session was launched by monocle's SessionManager.
    /// False (or None for existing sessions pre-v1A) if externally launched.
    pub spawned_by_monocle: Option<bool>,
}
```

This field is populated by the daemon when a session is added to the registry:
- `Some(true)` for sessions spawned via `SessionManager::spawn_session()`.
- `Some(false)` for sessions discovered via `detect()`.
- `None` for sessions in the registry before this field was added (forward-compat).

The TUI renders a small indicator in the sessions panel (`[M]` badge or similar) for
monocle-launched sessions to distinguish them from externally-monitored ones.

---

## Phase Compatibility

All Phase-1 `EngineModule` behavioral contracts (BC-2.03.*) remain in effect:
- `spawn_recipe()` is a new trait method with a default `Err` impl — it does NOT break
  existing implementations.
- The `#[non_exhaustive]` policy (ADR-0004) does not apply to trait methods; adding a
  method with a default impl is non-breaking for existing trait objects.
- `SpawnOptions` carries `#[non_exhaustive]` per BC-2.02.003 — it is a wire type (carried in
  `ClientToServer::SpawnSession { opts: SpawnOptions }` across the UDS IPC boundary, per
  I27-001 Model A resolution). `SpawnOptions` gains `Serialize`/`Deserialize` derives to
  support wire transmission. The TUI populates `project_root`, `worktree_root`, `harness_id`,
  `profile_id`, and `ccr_base_url`; the daemon IPC handler fills `session_id` and
  `hooks_settings_path` on receipt before passing to `SessionManager::spawn_session(opts)`.
- `SpawnRecipe` is DAEMON-INTERNAL after I27-001 (Model A). It is built by
  `engine_module.spawn_recipe(&opts)` inside `SessionManager::spawn_session()` and is
  never transmitted over IPC. The `Serialize`/`Deserialize` derives on `SpawnRecipe` are
  retained for potential diagnostic serialization but carry no wire-protocol obligation.
  `SpawnRecipe` is defined in `monocle-core` and constructed in `monocle-runtime`
  (`ClaudeCodeModule::spawn_recipe()`); this is cross-crate construction, so E0639 applies
  and `SpawnRecipe::new(binary, args, env, cwd) -> Self` is REQUIRED (C30-001 / ADR-0006).
  The implementation spec above uses `SpawnRecipe::new(...)` — not a struct literal — for
  exactly this reason. The `#[non_exhaustive]` attribute additionally provides forward-compat
  for future recipe field additions (e.g., resource limits, process group flags) without
  breaking the `ClaudeCodeModule` call site.

---

## Behavioral Contracts (additions to BC-2.03.*)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.03.005 | ClaudeCodeModule.spawn_recipe(): returns binary path, --settings arg, MONOCLE_SESSION_ID env | P0 |
| BC-2.03.006 | ClaudeCodeModule.spawn_recipe(): injects ANTHROPIC_BASE_URL when ccr_base_url present | P0 |
| BC-2.03.007 | spawn_recipe() with non-UTF-8 hooks_settings_path returns InvalidPath error | P1 |
| BC-2.03.008 | CodeMachineModule.spawn_recipe() returns UnsupportedOperation (v1 boundary) | P1 |

BC IDs are proposals; product-owner assigns canonical IDs in the PRD delta.

---

## §Trace v1.6.0

**F-P50-001 — Wire-code count propagated from 11 to 12 codes; SS-ipc.md pin updated v1.22.0 → v1.23.0** (2026-06-14):

- **Finding (F-P50-001, sibling-propagation sweep):** SS-ipc.md v1.23.0 (F-P50-001) added `"session_not_ready"` as the 12th wire code, extending the IPC error taxonomy from 11 to 12 codes. The `§Wire-code count note` in this document (§EngineError section) still stated "11 codes" (referencing SS-ipc.md v1.22.0), making the live-body count stale.
- **Fix (a) — §Wire-code count note updated:** Label changed from "updated F-P44-IMP-001" to "updated F-P50-001". Count updated: "11 codes in v1.22.0 (F-P44-IMP-001), then to 12 codes in v1.23.0 (F-P50-001: `"session_not_ready"` added)". Taxonomy descriptor updated from "11-code" to "12-code" in both the taxonomy-description sentence and the EngineError-produces-3-of-N sentence.
- **Fix (b) — §Trace v1.5.0 historical annotation:** The §Trace v1.5.0 wire-code-count bullet annotated with "[Subsequently updated to 12-code taxonomy by F-P50-001 — see §Trace v1.6.0]" to prevent future confusion between the historical 11-code state (v1.5.0 moment) and the current 12-code state.
- **Note on EngineError scope:** `EngineError` itself has not changed — it still has 3 variants producing 3 wire codes. The taxonomy extension (`"session_not_ready"`) is driven by `SessionError::SessionNotReady` in SS-session-manager.md v2.5.0, which is NOT a bridge through `EngineError`. The EngineError-to-wire-code relationship is unchanged; only the total taxonomy count changed.
- **Semver: minor (v1.5.0 → v1.6.0)** — wire-code count and SS-ipc.md pin updated; no behavioral change to `EngineError` itself. Minor (not patch) because the taxonomy pin references a normative IPC contract document at a new version.
- **Registry bump required:** `version-pin-registry.yaml` entry for `SS-engine-module-v2-delta.md` must be updated to v1.6.0 by state-manager in the same factory-artifacts commit as this spec file bump (REGISTRY ATOMICITY rule).

---

## §Trace v1.5.0

**F-P44-IMP-001 — `UnsupportedOperation` semantic contract reconciled: best-effort filtering + EC-112 reachable defensive path** (2026-06-14):

- **Finding (F-P44-IMP-001, IMPORTANT / secondary):** SS-engine-module-v2-delta.md §semantic-contract
  (lines ~333-334) stated: "callers should not treat it as a user-visible error (the TUI wizard only
  surfaces `spawn_recipe()` for harnesses that support it)." This contradicted BC-2.03.008 PC-3 /
  EC-112, which both mandate a REACHABLE path that surfaces the distinct user banner
  `"Session spawn not supported for this harness"`.
- **Decision:** ProfilePicker capability filtering is BEST-EFFORT, not a hard invariant. A harness's
  spawn capability may be unknown until spawn-time, may change after profile selection (config
  hot-reload, CodeMachine update), or a future/WASM engine may reach this path before filtering is
  established for it. The "wizard only surfaces spawn_recipe() for harnesses that support it" claim
  overstated the guarantee. The daemon must surface a distinct, deliverable error on
  `UnsupportedOperation` regardless. EC-112 is a reachable defensive path.
- **Fix — §semantic-contract updated:** `UnsupportedOperation` bullet rewritten to:
  (a) describe ProfilePicker filtering as best-effort; (b) enumerate the scenarios that make EC-112
  reachable at spawn-time despite filtering; (c) state that the daemon MUST surface
  `"spawn_unsupported"` wire code + fixed banner `"Session spawn not supported for this harness"`.
  Cross-reference to BC-2.03.008 PC-3 and F-P44-IMP-001 added.
- **Wire-code count updated (original):** The "10-code" note in §EngineError (new in v1A) updated to reflect
  the 11-code taxonomy (SS-ipc.md v1.22.0) and the corrected mapping:
  `EngineError` now produces 3 of the 11 wire codes (`BinaryNotFound`→`"binary_not_found"`;
  `InvalidPath`→`"invalid_spawn_arg"`; `UnsupportedOperation`→`"spawn_unsupported"`).
  [Subsequently updated to 12-code taxonomy by F-P50-001 — see §Trace v1.6.0.]
- Semver: minor (v1.4.1 → v1.5.0) — normative semantic-contract change for `UnsupportedOperation`;
  EC-112 reachability stance established.

## §Trace v1.4.1

**C34-001 — InvalidPath null-byte detection was factually impossible via to_str() alone** (2026-06-13, D-276):

- **Finding (C34-001 CRITICAL):** The `spawn_recipe()` implementation spec (step 2) detected `InvalidPath` ONLY via `opts.hooks_settings_path.to_str().ok_or_else(|| EngineError::InvalidPath(...))`. The `InvalidPath` doc-comment and §Semantic contract both claimed this variant ALSO covers paths "that contain embedded null bytes." This claim was factually wrong: null bytes (U+0000) ARE valid UTF-8, so `Path::to_str()` returns `Some(...)` for null-containing paths — it does NOT return `None`. A null-byte path would therefore pass through `to_str()` and only fail later at `CString`/`execve`/`portable_pty::CommandBuilder` construction as a `NulError`, which maps to `SessionError::SpawnFailed` with wire code `"spawn_failed"`, NOT `EngineError::InvalidPath` with wire code `"invalid_spawn_arg"`. This defeats BC-2.03.007 Invariant 1 (diagnostic-separation guarantee: `"invalid_spawn_arg"` vs `"spawn_failed"` must be categorically distinct).
- **Fix — Explicit two-pronged InvalidPath check in spawn_recipe() step 2:**
  - Prong 1 (existing, correct): `opts.hooks_settings_path.to_str().ok_or_else(|| EngineError::InvalidPath("...not valid UTF-8..."))?` — handles non-UTF-8 byte sequences.
  - Prong 2 (new, required): After prong 1 yields `path_str: &str`, an explicit byte scan `if path_str.as_bytes().contains(&0) { return Err(EngineError::InvalidPath("...embedded null byte...")); }` — handles paths that are valid UTF-8 but contain a null byte. The scan must precede any use of `path_str` as a CLI argument.
- **Doc-comment corrected:** `InvalidPath` variant doc-comment rewritten to state the TWO-PRONGED detection mechanism explicitly, explain WHY null bytes cannot be caught by `to_str()`, and document the consequence of omitting prong 2 (NulError → SpawnFailed → "spawn_failed" — BC-2.03.007 Invariant 1 violation).
- **§Semantic contract corrected:** `InvalidPath` bullet rewritten to describe both prongs explicitly, with the rationale for mandatory prong 2.
- **Semver: patch (v1.4.0 → v1.4.1)** — the InvalidPath contract for null bytes was always the INTENDED behavior (BC-2.03.007 Invariant 1 requires it); only the implementation mechanism was wrong. This is a correctness fix to the detection path, not a behavioral addition. If the implementing engineer had followed v1.4.0 literally, null-byte paths would have mapped to the wrong wire code — hence patch (not minor).
- **BC sweep required (no story cascade — all v1A):** BC-2.03.005, BC-2.03.006, BC-2.03.007, BC-2.03.008, BC-2.08.001, BC-2.08.006 — all cite `SS-engine-module-v2-delta.md v1.4.0` in Architecture Source; product-owner must sweep to v1.4.1.
- **Registry bump required:** `version-pin-registry.yaml` entry for `SS-engine-module-v2-delta.md` must be updated from v1.4.0 → v1.4.1 by state-manager (per REGISTRY ATOMICITY rule — registry and spec file bump are one atomic factory-artifacts commit).

---

## §Trace v1.4.0

**P31-CRIT-001 — `EngineError` declared completely as a canonical enum (Pass-31)** (2026-06-13):

- **Finding (P31-CRIT-001 CRITICAL):** `EngineError` was referenced at `Result<SpawnRecipe, EngineError>` (trait signature) and `SessionError::EngineError(#[from] monocle_core::engine::EngineError)` (bridge) but never formally declared as a `pub enum`. The `### EngineError additions` section mis-framed the type as "variants added to an existing EngineError" — but no base `EngineError` exists in the current source. An implementer reading the spec would not know the enum header, derives, `#[non_exhaustive]` decision, or relationship to other error types.
- **Fix — Full canonical `pub enum EngineError` declaration:** Replaced the fragment section with a complete declaration including: (1) enum header with `#[non_exhaustive]` + `#[derive(Debug, Clone, thiserror::Error)]`; (2) all 3 variants with `#[error("...")]` messages; (3) per-variant doc-comments; (4) semantic contract for each variant. Section renamed from "### EngineError additions" to "### EngineError (new in v1A)" to accurately reflect that this is a new type, not an extension.
- **Design decisions documented:** (a) Canonical location: `monocle_core::engine` (co-located with `EngineModule` trait); (b) `#[non_exhaustive]`: YES, for Phase 3 WASM engine forward-compat; downstream match arms need `_ =>` catch-all (already present in `session_error_to_code()`); (c) Audit table: EngineError is an ENUM — exempt from §Cross-Crate Constructor Audit Table per existing policy; (d) No ADR-0006 row needed (variant construction within defining crate is unrestricted by `#[non_exhaustive]`).
- **Relationship to existing error types clarified:** `EngineError` (spawn_recipe) is INDEPENDENT of `SpawnError` (legacy spawn()), `EngineMetadataError` (metadata/enrich), `PreflightError` (preflight). Disambiguation table added; no subsumption between them.
- **"10-code" conflation resolved:** The phrase referred to the 10 IPC wire codes in `ServerToClient::Error.code` (SS-ipc.md), NOT the `EngineError` variant count. `EngineError` has 3 variants (Phase 1). The 10-code taxonomy is the wire layer; `EngineError` is daemon-internal. `BinaryNotFound` → `"binary_not_found"`; `InvalidPath` → `"invalid_spawn_arg"`; future variants → `"invalid_request"` catch-all.
- Semver: minor (v1.3.0 → v1.4.0) — normative type declaration added (was incomplete fragment); no behavioral change to existing specs (variant semantics unchanged).

## §Trace v1.3.0

**C30-001 — ADR-0006 constructor gap: `SpawnOptions` and `SpawnRecipe` lacked public constructors despite cross-crate construction** (2026-06-13):

- **Finding (C30-001 CRITICAL — SpawnOptions):** `SpawnOptions` is `#[non_exhaustive]` and constructed cross-crate by `monocle-tui` (SessionCreation wizard) and by `monocle-runtime` (daemon IPC handler fills `session_id` and `hooks_settings_path` on receipt). No `pub fn new(...)` or builder existed. The daemon IPC handler sample in SS-daemon-wiring-v2-delta.md §3 used `SpawnOptions { session_id: ..., hooks_settings_path: ..., ..opts }` — the functional-update (`..opts`) on a `#[non_exhaustive]` struct from an external crate is E0639 (Rust E0639 applies to functional-record-update syntax as well as struct literals for `#[non_exhaustive]` types outside their defining crate).
- **Finding (C30-001 CRITICAL — SpawnRecipe):** `SpawnRecipe` is `#[non_exhaustive]` and defined in `monocle-core`, constructed cross-crate in `monocle-runtime` (`ClaudeCodeModule::spawn_recipe()`). The implementation spec used a struct literal `SpawnRecipe { binary, args, env, cwd }` — E0639 from `monocle-runtime`'s perspective. No `pub fn new(...)` existed.
- **Fix — `SpawnOptions::for_spawn_request(project_root, worktree_root, harness_id, profile_id, ccr_base_url) -> Self`:** TUI-side constructor. Daemon-owned fields (`session_id`, `hooks_settings_path`) are initialized to documented placeholder values (`String::new()`, `PathBuf::new()`) — always overwritten by daemon before use. This is NOT a Default-substitution for required fields: the fields ARE populated at production-time by `with_daemon_fields()`; the placeholder communicates "not yet populated" in specs and tests.
- **Fix — `SpawnOptions::with_daemon_fields(self, session_id: String, hooks_settings_path: PathBuf) -> Self`:** Daemon-side consuming builder. Replaces the `..opts` functional-update pattern in SS-daemon-wiring-v2-delta.md §3 sample (C30-001 root cause). The daemon IPC handler now calls `let opts = opts.with_daemon_fields(uuid, state.hooks_settings_path.clone()); spawn_session(opts).await`.
- **Fix — `SpawnRecipe::new(binary, args, env, cwd) -> Self`:** Four-field positional constructor. The `ClaudeCodeModule::spawn_recipe()` implementation spec updated to use `SpawnRecipe::new(...)` instead of the struct literal.
- **Fix — §Phase Compatibility prose corrected:** The prior text said `SpawnRecipe`'s `#[non_exhaustive]` "is harmless" for "daemon-internal code that struct-literal constructs SpawnRecipe values." This was wrong: the construction is CROSS-CRATE (monocle-runtime → monocle-core), so E0639 DOES apply and `SpawnRecipe::new()` is required. Prose updated to state the cross-crate relationship and E0639 applicability explicitly.
- **Canonical constructor spec:** `SpawnOptions::for_spawn_request()` and `with_daemon_fields()` are the normative patterns for all spec-level samples and test fixtures. The byte-for-byte consistent `SpawnOptions` struct definition in SS-session-manager.md §SpawnOptions is updated with identical `impl SpawnOptions` block (same constructor bodies, same doc-comments; see SS-session-manager.md §Trace v2.1.0).
- **Audit table:** `SpawnOptions` and `SpawnRecipe` added to `SS-engine-module.md §Cross-Crate Constructor Audit Table` (v1.1.26 → v1.1.27).
- Semver: minor (v1.2.0 → v1.3.0) — additive constructor additions + Phase Compatibility prose fix; no behavioral change.

## §Trace v1.2.0

**I27-001 — wire-type reconciliation: `SpawnOptions` becomes wire type; `SpawnRecipe` becomes daemon-internal** (2026-06-13):

- **Finding (I27-001):** The §Phase Compatibility section incorrectly stated that "`SpawnRecipe` carries `#[non_exhaustive]` per BC-2.02.003 — it is a wire type (carried in `ClientToServer::SpawnSession { recipe: SpawnRecipe }`)." Under the correct Model A architecture (daemon-side `spawn_recipe()` execution), `SpawnRecipe` is daemon-internal and `SpawnOptions` is the wire type. The prior S26-001 §Phase Compatibility prose had the wire/internal assignments exactly backwards.
- **Fix (a) — `SpawnOptions` promoted to wire type:** `SpawnOptions` gains `#[non_exhaustive]` (BC-2.02.003 wire-type extensibility policy) and `Serialize`/`Deserialize` derives. It is carried in `ClientToServer::SpawnSession { opts: SpawnOptions }`. The TUI populates `project_root`, `worktree_root`, `harness_id`, `profile_id`, `ccr_base_url`; the daemon IPC handler fills `session_id` and `hooks_settings_path` on receipt.
- **Fix (b) — `SpawnRecipe` demoted to daemon-internal:** `SpawnRecipe` is no longer a wire type. Its `Serialize`/`Deserialize` derives are retained as optional (for diagnostic serialization), but it is never transmitted over IPC. `#[non_exhaustive]` on `SpawnRecipe` is retained as harmless forward-compat on a daemon-internal struct.
- **Fix (c) — §Phase Compatibility prose corrected:** Two-bullet statement now correctly reads: `SpawnOptions` carries `#[non_exhaustive]` (wire type); `SpawnRecipe` does NOT carry mandatory wire-type obligations (daemon-internal).
- **Fix (d) — `SpawnOptions` doc-comment and struct header updated:** Added I27-001 Model A rationale; added `harness_id: String` field (needed for `SessionEntry` recording; previously implicit in `profile_id`/`harness_id` on the old `spawn_session()` signature).
- **Errata (C29-001, 2026-06-13) — Fix(d) struct-body edit inadvertently omitted at authoring:** The `harness_id: String` field attested in Fix(d) above was missing from the `SpawnOptions` struct body when this delta was first authored; only the doc-comment and prose reflected the field. The struct body is now corrected to match: `pub harness_id: String,` inserted after `worktree_root` and before `profile_id`, consistent with SS-session-manager.md §SpawnOptions field order. No semantic change; the published v1.2.0 meaning is unchanged.
- Semver: minor (v1.1.1 → v1.2.0) — normative wire-type assignment change; `SpawnOptions` derive change.

## §Trace v1.1.1

**S26-001 — `SpawnRecipe` missing `#[non_exhaustive]` (exhaustive wire-type class sweep)** (2026-06-13):

- **Finding (S26-001 adversarial suggestion / in-scope fix):** `SpawnRecipe` struct was declared
  `#[derive(Debug, Clone, Serialize, Deserialize)]` without `#[non_exhaustive]`. This contradicted:
  (a) the §Phase Compatibility prose in this file ("SpawnRecipe … carr[ies] `#[non_exhaustive]`
  where applicable per BC-2.02.003"), and (b) SS-ipc.md §Message Types blanket policy ("All public
  enums and message structs carry `#[non_exhaustive]` per the SS-02 extensibility policy"). As a
  wire type carried in `ClientToServer::SpawnSession { recipe: SpawnRecipe }` over the shared UDS,
  `SpawnRecipe` must carry `#[non_exhaustive]` to permit additive field evolution without breaking
  existing struct-literal constructors at callers.
- **Fix — `#[non_exhaustive]` added above `#[derive(...)]` on `SpawnRecipe`:** Attribute placed
  above the derive line per Rust convention (attribute order: outer attributes ordered
  non_exhaustive → derive).
- **§Phase Compatibility prose narrowed:** The vague "where applicable" language is replaced with
  an explicit two-bullet statement: `SpawnRecipe` carries `#[non_exhaustive]` (wire type);
  `SpawnOptions` does NOT (daemon-internal, no Serialize/Deserialize, never on any IPC message).
- **`SpawnOptions` determination:** `SpawnOptions` derives `#[derive(Debug, Clone)]` only — no
  `Serialize`/`Deserialize`. It is NOT placed on any IPC message in the in-scope docs. It is
  daemon-internal: constructed by `SessionManager`, passed to `EngineModule::spawn_recipe()`, and
  consumed entirely within the daemon process. The `#[non_exhaustive]` policy (BC-2.02.003)
  applies to wire types; `SpawnOptions` is definitively NOT a wire type. No attribute change needed.
- Semver: patch (v1.1.0 → v1.1.1) — attribute addition + prose narrowing; no behavioral change.

## §Trace v1.1.0

**I2-002 worktree-per-session operationalized** (2026-06-03):
- `SpawnOptions.worktree_root: PathBuf` added (was absent; cwd was incorrectly set from
  `project_root`). The new field carries the resolved git worktree path (or `project_root`
  when no worktree applies). The three-rule resolution spec is in SS-session-manager.md
  §SpawnOptions.worktree_root.
- `SpawnRecipe.cwd` doc-comment corrected: "populated from SpawnOptions.worktree_root"
  (was "MUST be git worktree root" without specifying how to get it).
- `ClaudeCodeModule::spawn_recipe()` implementation updated: `cwd: opts.worktree_root.clone()`
  (was `opts.project_root.clone()` — the root bug that caused the missing worktree field).
- `SpawnRecipe.env` doc-comment updated: "OVERLAY on top of inherited env" (was "MERGED
  with current environment" — the session-host startup step 4 spec in SS-session-manager
  defines the inheritance semantics; this doc now matches).
- BC sync required (product-owner): BC-2.03.005 PC-1 ("cwd is worktree root" needs
  SpawnOptions.worktree_root to be cited as the source), BC-2.03.006 (env overlay semantics),
  BC-2.08.001 (spawn recipe assembled from worktree root), BC-2.08.007 ("cwd is project root"
  in title must be revised to "cwd is resolved worktree root or project_root").

## §Trace v1.0.1

**IMP-5 EngineError taxonomy fix — InvalidPath variant** (2026-06-03):
- Added `EngineError::InvalidPath(String)` variant to the error taxonomy. The original
  draft erroneously used `BinaryNotFound` for both "harness not found on PATH" and
  "hooks_settings_path cannot be converted to UTF-8 string." These are categorically
  different failure modes; conflating them produces misleading diagnostics.
- `BinaryNotFound` is now reserved exclusively for the "harness binary not found on PATH"
  case (which::which failure). `InvalidPath` is used for structurally invalid arguments
  to spawn_recipe(), including non-UTF-8 path values.
- ClaudeCodeModule::spawn_recipe() implementation spec updated: the to_str() unwrap now
  returns `InvalidPath` instead of `BinaryNotFound`.
- BC-2.03.007 proposal title corrected: "returns BinaryNotFound error" → "returns
  InvalidPath error". PO will author the canonical BC text.
- Semantic contract section added to distinguish the two error variants.

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- Engine module v2 delta authored for v1A architecture delta.
- `spawn_recipe()` method specified; Q-2 resolution confirmed.
- `SpawnRecipe` and `SpawnOptions` types defined.
- `ClaudeCodeModule::spawn_recipe()` implementation spec written.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
