---
document_type: architecture-section-delta
level: L3
section: "engine-module-v2-delta"
subsystem: SS-03
version: "1.0.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T23:00:00Z
inputs:
  - specs/architecture/SS-engine-module.md
  - research/domain-monocle-vision-synthesis.md
  - specs/architecture/SS-session-manager.md
  - specs/architecture/adr/ADR-0009-native-session-host-process-model.md
input-hash: "3e4fab8"
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRecipe {
    /// Absolute path to the harness binary.
    pub binary: PathBuf,
    /// CLI arguments (e.g., ["--settings", "/tmp/monocle-hooks-abc.json"]).
    /// The hooks_settings_path from SpawnOptions MUST be passed here as --settings.
    pub args: Vec<String>,
    /// Environment variables to inject (CCR ANTHROPIC_BASE_URL, MONOCLE_SESSION_ID, etc.).
    /// These are MERGED with the harness binary's current environment; they do not replace it.
    pub env: HashMap<String, String>,
    /// Working directory for the harness child process.
    /// MUST be the git worktree root for the project (claude-squad A.1 worktree-per-task).
    pub cwd: PathBuf,
}

/// Options passed from SessionManager to EngineModule::spawn_recipe().
#[derive(Debug, Clone)]
pub struct SpawnOptions {
    /// Project root directory (used as cwd if no worktree is configured).
    pub project_root: PathBuf,
    /// Harness profile ID selected by the user in the SessionCreation wizard.
    pub profile_id: String,
    /// Pre-generated session UUID.
    pub session_id: String,
    /// Path where the daemon has already written the per-session hooks-settings.json.
    /// The EngineModule MUST include "--settings <hooks_settings_path>" in the returned recipe args.
    pub hooks_settings_path: PathBuf,
    /// If CCR is detected and a base URL is configured, this carries the URL.
    /// The EngineModule MUST inject this as `ANTHROPIC_BASE_URL` in `env` if present.
    pub ccr_base_url: Option<String>,
}
```

### EngineError additions

The following variant is added to `EngineError` in `monocle-core/src/engine.rs`:

```rust
#[error("unsupported operation: {0}")]
UnsupportedOperation(&'static str),

#[error("harness binary not found: {0}")]
BinaryNotFound(String),
```

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
                .ok_or_else(|| EngineError::BinaryNotFound("invalid hooks_settings_path".into()))?
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
            cwd: opts.project_root.clone(),
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
- `SpawnRecipe` and `SpawnOptions` carry `#[non_exhaustive]` where applicable per BC-2.02.003.

---

## Behavioral Contracts (additions to BC-2.03.*)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.03.005 | ClaudeCodeModule.spawn_recipe(): returns binary path, --settings arg, MONOCLE_SESSION_ID env | P0 |
| BC-2.03.006 | ClaudeCodeModule.spawn_recipe(): injects ANTHROPIC_BASE_URL when ccr_base_url present | P0 |
| BC-2.03.007 | spawn_recipe() with invalid hooks_settings_path returns BinaryNotFound error | P1 |
| BC-2.03.008 | CodeMachineModule.spawn_recipe() returns UnsupportedOperation (v1 boundary) | P1 |

BC IDs are proposals; product-owner assigns canonical IDs in the PRD delta.

---

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- Engine module v2 delta authored for v1A architecture delta.
- `spawn_recipe()` method specified; Q-2 resolution confirmed.
- `SpawnRecipe` and `SpawnOptions` types defined.
- `ClaudeCodeModule::spawn_recipe()` implementation spec written.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
