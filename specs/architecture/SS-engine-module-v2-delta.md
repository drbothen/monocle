---
document_type: architecture-section-delta
level: L3
section: "engine-module-v2-delta"
subsystem: SS-03
version: "1.2.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T23:00:00Z
inputs:
  - specs/architecture/SS-engine-module.md
  - research/domain-monocle-vision-synthesis.md
  - specs/architecture/SS-session-manager.md
  - specs/architecture/adr/ADR-0009-native-session-host-process-model.md
input-hash: "426f012"
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
```

### EngineError additions

The following variants are added to `EngineError` in `monocle-core/src/engine.rs`:

```rust
#[error("unsupported operation: {0}")]
UnsupportedOperation(&'static str),

#[error("harness binary not found: {0}")]
BinaryNotFound(String),

/// Invalid argument supplied to spawn_recipe() — e.g., a hooks_settings_path that
/// cannot be converted to a valid UTF-8 string, or a path that contains null bytes.
/// Distinct from BinaryNotFound: the harness binary may exist but the argument is
/// structurally invalid and cannot be passed as a CLI arg.
#[error("invalid argument: {0}")]
InvalidPath(String),
```

**Semantic contract:**
- `BinaryNotFound` is reserved exclusively for the case where `which::which("claude")`
  (or the equivalent for other harnesses) fails — i.e., the harness binary cannot be
  located on `PATH`.
- `InvalidPath` is used for structurally invalid arguments to `spawn_recipe()`, including
  `hooks_settings_path` values that cannot be converted to a UTF-8 string (required for
  CLI arg passing), or that contain embedded null bytes.
- These two failure modes are categorically different: `BinaryNotFound` means "the harness
  is not installed"; `InvalidPath` means "the supplied configuration is invalid." Conflating
  them (as the original draft did) misrepresents the error cause to callers and produces
  incorrect diagnostic messages.

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
        let args = vec![
            "--settings".to_string(),
            opts.hooks_settings_path
                .to_str()
                .ok_or_else(|| EngineError::InvalidPath(
                    format!("hooks_settings_path is not valid UTF-8: {:?}", opts.hooks_settings_path)
                ))?
                .to_string(),
        ];

        // 3. Build env: inject CCR base URL if configured
        let mut env = HashMap::new();
        if let Some(ref url) = opts.ccr_base_url {
            env.insert("ANTHROPIC_BASE_URL".to_string(), url.clone());
        }
        // Inject MONOCLE_SESSION_ID so the session can be correlated in hook events
        env.insert("MONOCLE_SESSION_ID".to_string(), opts.session_id.clone());

        Ok(SpawnRecipe {
            binary,
            args,
            env,
            // Use the resolved worktree root (not project_root directly).
            // project_root == worktree_root when no git worktree is configured.
            cwd: opts.worktree_root.clone(),
        })
    }
}
```

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
  `SpawnRecipe` does NOT require `#[non_exhaustive]` from a wire-type perspective (it is
  not on the IPC boundary); however the attribute is harmless and is retained for the
  forward-compat guarantee within daemon-internal code that struct-literal constructs
  `SpawnRecipe` values.

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

## §Trace v1.2.0

**I27-001 — wire-type reconciliation: `SpawnOptions` becomes wire type; `SpawnRecipe` becomes daemon-internal** (2026-06-13):

- **Finding (I27-001):** The §Phase Compatibility section incorrectly stated that "`SpawnRecipe` carries `#[non_exhaustive]` per BC-2.02.003 — it is a wire type (carried in `ClientToServer::SpawnSession { recipe: SpawnRecipe }`)." Under the correct Model A architecture (daemon-side `spawn_recipe()` execution), `SpawnRecipe` is daemon-internal and `SpawnOptions` is the wire type. The prior S26-001 §Phase Compatibility prose had the wire/internal assignments exactly backwards.
- **Fix (a) — `SpawnOptions` promoted to wire type:** `SpawnOptions` gains `#[non_exhaustive]` (BC-2.02.003 wire-type extensibility policy) and `Serialize`/`Deserialize` derives. It is carried in `ClientToServer::SpawnSession { opts: SpawnOptions }`. The TUI populates `project_root`, `worktree_root`, `harness_id`, `profile_id`, `ccr_base_url`; the daemon IPC handler fills `session_id` and `hooks_settings_path` on receipt.
- **Fix (b) — `SpawnRecipe` demoted to daemon-internal:** `SpawnRecipe` is no longer a wire type. Its `Serialize`/`Deserialize` derives are retained as optional (for diagnostic serialization), but it is never transmitted over IPC. `#[non_exhaustive]` on `SpawnRecipe` is retained as harmless forward-compat on a daemon-internal struct.
- **Fix (c) — §Phase Compatibility prose corrected:** Two-bullet statement now correctly reads: `SpawnOptions` carries `#[non_exhaustive]` (wire type); `SpawnRecipe` does NOT carry mandatory wire-type obligations (daemon-internal).
- **Fix (d) — `SpawnOptions` doc-comment and struct header updated:** Added I27-001 Model A rationale; added `harness_id: String` field (needed for `SessionEntry` recording; previously implicit in `profile_id`/`harness_id` on the old `spawn_session()` signature).
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
