---
document_type: story
level: L4
story_id: S-045
epic_id: EPIC-03
version: "1.1"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 5
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-015, S-033]
blocks: []
target_module: monocle-runtime
subsystems: [SS-03]
behavioral_contracts: [BC-2.03.005, BC-2.03.006, BC-2.03.007]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.005.md, version: "1.1.5"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.006.md, version: "1.1.2"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.007.md, version: "1.2.5"}
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.008.md, version: "1.0.8"}
    # BC-2.03.008 input retained for cross-reference; BC-2.03.008 is owned by S-033 (re-anchored Pass-7). This story verifies non-override behavior indirectly via the compile-check task.
  - {path: .factory/specs/architecture/SS-engine-module-v2-delta.md, version: "1.6.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.6.1"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.24.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.03.005 (spawn_recipe happy path), BC-2.03.006 (CCR base URL injection), BC-2.03.007 (BinaryNotFound/InvalidPath errors). BC-2.03.008 (default UnsupportedOperation trait impl) is anchored to S-033 (trait method + EngineError defined there)."
# BC status: BC-2.03.005/006/007 non-empty; status draft pending Phase-2 adversarial convergence gate. BC-2.03.008 re-anchored to S-033 (Pass-7 fix).
---

# S-045: ClaudeCodeModule::spawn_recipe() — Happy Path, CCR Injection, Error Cases, and Default Trait Impl

## Narrative

As the monocle daemon, I want `ClaudeCodeModule::spawn_recipe()` to resolve the `claude`
binary on PATH, build a `SpawnRecipe` with `--settings <hooks_settings_path>` and
`MONOCLE_SESSION_ID` in the env (plus optional `ANTHROPIC_BASE_URL` when CCR is configured),
set `cwd` to `opts.worktree_root`, and return typed errors (`BinaryNotFound` or `InvalidPath`)
for the two distinct failure modes — so that `SessionManager::spawn_session()` can delegate
recipe assembly to the engine module cleanly and receive actionable IPC error codes when the
spawn path fails.

## Acceptance Criteria

### AC-001 (traces to BC-2.03.005 postcondition 1 — happy-path recipe fields)

When `ClaudeCodeModule::spawn_recipe(opts)` is called with a valid `SpawnOptions` and `claude`
is on `PATH`:
- `recipe.binary` is the absolute `PathBuf` returned by `which::which("claude")`.
- `recipe.args` is exactly `["--settings", <hooks_settings_path_as_utf8_str>]` (two elements).
- `recipe.env` contains `"MONOCLE_SESSION_ID"` mapped to `opts.session_id`.
- `recipe.cwd` equals `opts.worktree_root`.
- The returned `SpawnRecipe` has no `None` or empty fields.

### AC-002 (traces to BC-2.03.005 postcondition 3 — synchronous, no I/O beyond PATH lookup)

`spawn_recipe()` is synchronous (non-async). It performs exactly one filesystem lookup
(`which::which("claude")`) and one UTF-8 conversion. It does NOT write any files.

### AC-003 (traces to BC-2.03.006 postcondition 1 — CCR base URL injection when Some)

When `opts.ccr_base_url` is `Some(url)`:
- `recipe.env` contains `"ANTHROPIC_BASE_URL"` mapped to `url` verbatim (no normalization).
- `recipe.env` also contains `"MONOCLE_SESSION_ID"` per AC-001.
- No URL validation is performed by `spawn_recipe()`.

### AC-004 (traces to BC-2.03.006 postcondition 3 — ANTHROPIC_BASE_URL absent when None)

When `opts.ccr_base_url` is `None`:
- `recipe.env` does NOT contain the key `"ANTHROPIC_BASE_URL"`.
- `recipe.env` contains only `"MONOCLE_SESSION_ID"`.

### AC-005 (traces to BC-2.03.007 postcondition 1–4 — BinaryNotFound when which fails)

When `which::which("claude")` fails (binary not on PATH):
- `spawn_recipe()` returns `Err(EngineError::BinaryNotFound("claude".into()))`.
- The error message format is `"harness binary not found: claude"`.
- This error propagates as `EngineError::BinaryNotFound` → `SessionError::EngineError` →
  `ServerToClient::Error { code: "binary_not_found", message }` per BC-2.03.007 PC-3.
- `BinaryNotFound` is NOT returned for any other failure mode.

### AC-006 (traces to BC-2.03.007 postcondition 5–8 — two-pronged InvalidPath check)

When `opts.hooks_settings_path` fails the two-pronged UTF-8 / null-byte check:
- Prong 1: `opts.hooks_settings_path.to_str()` returns `None` (non-UTF-8 bytes) →
  `Err(EngineError::InvalidPath(...))`.
- Prong 2: `to_str()` returns `Some(path_str)` but `path_str.as_bytes().contains(&0)` is true
  (embedded null byte — valid UTF-8 but OS-rejected) → `Err(EngineError::InvalidPath(...))`.
- Both prongs produce the same `EngineError::InvalidPath` variant and IPC error code
  `"invalid_spawn_arg"`.
- `InvalidPath` is NOT returned for binary-not-found failures; the two variants are not conflated.

### AC-007 (traces to BC-2.03.007 invariant 2 — binary checked before path)

When both `which::which("claude")` fails AND `hooks_settings_path` is invalid:
- `spawn_recipe()` returns `Err(EngineError::BinaryNotFound("claude"))` — binary check is first.
- Arg validation is never reached when the binary lookup fails (early return).

### AC-008 (traces to BC-2.03.005 invariant 1 — hooks_settings_path is CLI arg, not written by spawn_recipe)

`spawn_recipe()` does NOT write `hooks-settings.json`. The `hooks_settings_path` is passed
through as a CLI arg string (`--settings <path>`). The file is written by the daemon BEFORE
`spawn_recipe()` is called (existing OQ-02 mechanism).

### AC-009 (traces to BC-2.03.005 invariant 3 — env MERGED, not replaced)

The environment map (`recipe.env`) is an OVERLAY on the child process's inherited environment
at spawn time. Variables not in `recipe.env` are inherited unchanged. `spawn_recipe()` itself
returns the overlay map only; the merge is performed by `monocle-session-host` at spawn time.

## Tasks

- [ ] **Verify (do NOT re-add):** S-033 added `spawn_recipe(&self, opts: &SpawnOptions) -> Result<SpawnRecipe, EngineError>` default method + `EngineError` enum (`BinaryNotFound(String)`, `InvalidPath(String)`, `UnsupportedOperation(&'static str)`) to `monocle-core/src/engine.rs`. Compile-check that `use monocle_core::engine::{EngineModule, EngineError, SpawnOptions, SpawnRecipe};` resolves before proceeding. If not found, S-033 has not landed — block and surface to orchestrator.
- [ ] Implement `ClaudeCodeModule::spawn_recipe()` in `monocle-runtime/src/engine/claude_code.rs`:
      - Step 1: `which::which("claude")` — on failure return `Err(EngineError::BinaryNotFound("claude"))`.
      - Step 2: two-pronged path check — Prong 1: `opts.hooks_settings_path.to_str()` returns `None`
        → `Err(EngineError::InvalidPath(...))`. Prong 2: `path_str.as_bytes().contains(&0)` fires
        → `Err(EngineError::InvalidPath(...))`.
      - Step 3: Build `recipe.env` with `MONOCLE_SESSION_ID` always present; add `ANTHROPIC_BASE_URL`
        when `opts.ccr_base_url` is `Some`.
      - Step 4: Set `recipe.cwd = opts.worktree_root.clone()`.
      - Step 5: Return `Ok(SpawnRecipe { binary, args: ["--settings", path_str], env, cwd })`.
- [ ] Add `SessionError::EngineError(#[from] EngineError)` variant to the `SessionError` enum in
      `monocle-runtime/src/session_manager/mod.rs` (canonical location per S-033; if not already
      present from S-033, add it here — do NOT create a separate error module).
- [ ] Add `"binary_not_found"` and `"invalid_spawn_arg"` arms to `session_error_to_code()` for
      `IpcOp::Spawn` (per BC-2.03.007 PC-3/PC-7).
- [ ] Write unit tests in `monocle-runtime/tests/spawn_recipe.rs`:
      - `test_BC_2_03_005_spawn_recipe_happy_path_binary_args_env_cwd` (AC-001/002/009)
      - `test_BC_2_03_005_spawn_recipe_ccr_none_env_monocle_id_only` (AC-004)
      - `test_BC_2_03_006_spawn_recipe_ccr_base_url_injected` (AC-003)
      - `test_BC_2_03_007_spawn_recipe_binary_not_found` (AC-005)
      - `test_BC_2_03_007_spawn_recipe_invalid_path_non_utf8` (AC-006 Prong 1)
      - `test_BC_2_03_007_spawn_recipe_invalid_path_null_byte` (AC-006 Prong 2)
      - `test_BC_2_03_007_spawn_recipe_binary_checked_first` (AC-007)
      - Note: `test_BC_2_03_008_default_spawn_recipe_unsupported_operation` is owned by S-033 (BC-2.03.008 re-anchored per Pass-7 fix). Do NOT add it here.

## Previous Story Intelligence

S-014 defined `EngineModule` trait in `monocle-core/src/engine.rs`. S-015 implemented
`ClaudeCodeModule` with detect/enrich/metadata/on_hook/hook_paths. S-033 (Wave 8, before
this story) added the `spawn_recipe()` DEFAULT TRAIT METHOD + `EngineError` enum to
`monocle-core/src/engine.rs` (BC-2.03.008), and `spawn_session()` calls
`engine_module.spawn_recipe(&opts)?` as its FIRST step. This story (S-045) adds ONLY the
`ClaudeCodeModule::spawn_recipe()` concrete OVERRIDE (BC-2.03.005/006/007). The trait
method already exists (from S-033); this is the implementation-side addition. The
`SessionError::EngineError` bridge added in S-033 may already be present — verify before
re-adding.

`S-015` note: that story used `temp-env ^0.3` with `features = ["async_closure"]` for the
`HomeUnresolvable` path tests. This story's tests do NOT require env manipulation (they mock
`which` or compile-test the path). Keep test isolation patterns consistent.

## Architecture Compliance Rules

From `architecture/SS-engine-module-v2-delta.md v1.6.0`:
- `spawn_recipe()` is a NEW trait method on `EngineModule` with a DEFAULT impl
  (non-breaking addition; BC-2.03.008 owned by S-033 which defines the trait method + EngineError).
- `EngineError` is a NEW type introduced in v1A — NOT an extension of `EngineMetadataError`.
- The two-pronged null-byte detection is mandatory: `to_str()` alone cannot detect null bytes.
- `SpawnRecipe` is daemon-internal — never crosses the IPC wire. Only `SpawnOptions` is on the wire.
- `recipe.cwd` MUST be `opts.worktree_root`, NOT `opts.project_root` directly.
- Model A (I27-001): `spawn_recipe()` is called INSIDE `spawn_session()` as its first step.
  The TUI sends `SpawnOptions`; the daemon builds `SpawnRecipe` daemon-side.

From `architecture/SS-ipc.md v1.24.0`:
- Wire codes `"binary_not_found"` and `"invalid_spawn_arg"` must be in the `ServerToClient::Error`
  taxonomy.

**Forbidden dependencies**: `monocle-core` MUST NOT depend on `which`. The `which::which()` call
is in `monocle-runtime` (ClaudeCodeModule implementation). If `monocle-core` gains a dependency on
`which`, the build MUST fail.

## Library and Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| `which` | `^7` | `which::which("claude")` in `ClaudeCodeModule::spawn_recipe()` — `monocle-runtime` dep only |
| `thiserror` | `^2` | `EngineError` type derivation in `monocle-core` |
| `serde` | `^1` (features = ["derive"]) | `SpawnRecipe` and `SpawnOptions` serialization |

No new dependencies added. `which` is already in `monocle-runtime` (BC-2.07.006 CCR detection).
`EngineError` is a new type in `monocle-core` — does NOT require any new dep.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/monocle-core/src/engine.rs` | MODIFY | Add `spawn_recipe()` default method to `EngineModule` trait; add `EngineError` enum |
| `crates/monocle-runtime/src/engine/claude_code.rs` | MODIFY | Implement `ClaudeCodeModule::spawn_recipe()` |
| `crates/monocle-runtime/src/session_manager/mod.rs` | MODIFY (if needed) | Ensure `SessionError::EngineError(#[from] EngineError)` variant present (S-033 defined `SessionError` here; this is the canonical location) |
| `crates/monocle-runtime/tests/spawn_recipe.rs` | CREATE | Unit tests for all AC-001..AC-009 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-100 | `which::which("claude")` succeeds; `opts.ccr_base_url` is `None` | `recipe.env = {"MONOCLE_SESSION_ID": opts.session_id}` — no `ANTHROPIC_BASE_URL` key |
| EC-101 | `opts.ccr_base_url` is `Some("http://localhost:8080")` | `recipe.env` contains both `MONOCLE_SESSION_ID` and `ANTHROPIC_BASE_URL` |
| EC-102 | Non-git project: `worktree_root = project_root` (three-rule fallback) | `recipe.cwd = opts.worktree_root` (which equals `project_root`) — no error |
| EC-104 | `hooks_settings_path` has `\xFF\xFE` non-UTF-8 bytes (Prong 1) | `Err(EngineError::InvalidPath(...))` — `"invalid_spawn_arg"` wire code |
| EC-104b | `hooks_settings_path` is valid UTF-8 but contains embedded `\x00` null (Prong 2) | `Err(EngineError::InvalidPath(...))` — `to_str()` returns `Some` but explicit scan fires |
| EC-108 | `claude` binary is on PATH but not executable | `spawn_recipe()` returns `Ok(recipe)` — executability is NOT checked here; failure at spawn time |
| EC-112 | `CodeMachineModule.spawn_recipe()` called | Default impl returns `Err(EngineError::UnsupportedOperation("spawn_recipe"))` |

## Token Budget Estimate

| Category | Estimate |
|----------|----------|
| Story spec (this file) | ~4 000 tokens |
| BC files (3 BCs: BC-2.03.005/006/007; BC-2.03.008 cross-reference only) | ~5 000 tokens |
| Architecture sections (SS-engine-module-v2-delta, SS-session-manager, SS-ipc excerpts) | ~3 000 tokens |
| Existing code context (engine.rs, claude_code.rs, session_manager/mod.rs) | ~3 000 tokens |
| Test file to write | ~2 000 tokens |
| **Total estimated** | **~18 000 tokens** |

Well within the 20–30% context window constraint. Story does not need splitting.

## Dependency Justification

- S-045 depends on S-015 because `ClaudeCodeModule` and the `EngineModule` trait it extends
  are implemented in S-015; adding `ClaudeCodeModule::spawn_recipe()` is an additive override
  on that struct, which must exist first.
- S-045 depends on S-033 because S-033 delivers BC-2.03.008 — the `spawn_recipe()` default
  trait method + `EngineError` enum in `monocle-core/src/engine.rs`. S-033 compiles and passes
  unit tests via the default `Err(UnsupportedOperation)` impl. S-045 adds the `ClaudeCodeModule`
  concrete override (BC-2.03.005/006/007) that S-033's end-to-end integration tests require.
  Wave-8 intra-wave order: S-033 BEFORE S-045. S-045 blocks nothing directly because all
  downstream stories (S-034..S-038, S-047, S-048) depend on S-033 (not S-045 directly).

## Subsystem Anchor Justification

SS-03 owns this story's scope because `ClaudeCodeModule::spawn_recipe()` is the Phase 1
implementation of the engine abstraction's spawn method — exactly the `ClaudeCodeModule` adapter
that SS-03 governs per ARCH-INDEX Subsystem Registry SS-03 (monocle-runtime, ClaudeCodeModule).
