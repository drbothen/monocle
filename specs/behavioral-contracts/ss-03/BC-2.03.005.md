---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-engine-module-v2-delta.md, architecture/SS-session-manager.md]
input-hash: "38bc71b"
traces_to: prd.md
origin: greenfield
subsystem: SS-03
capability: CAP-003
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.03.005: ClaudeCodeModule.spawn_recipe() — Happy-Path Recipe Assembly

## Description

`ClaudeCodeModule::spawn_recipe()` is the implementation of the `EngineModule::spawn_recipe()`
trait method for the Claude Code harness. Given a valid `SpawnOptions`, it resolves the
`claude` binary on `PATH`, constructs a `SpawnRecipe` with `--settings <hooks_settings_path>`
in the args, injects `MONOCLE_SESSION_ID` into the environment, and sets the working directory
to `opts.project_root`. The returned recipe is consumed by `SessionManager` to spawn a
`monocle-session-host` process.

## Preconditions

1. The `claude` binary is locatable via `which::which("claude")` on the current `PATH`.
2. `opts.hooks_settings_path` is a valid UTF-8 path (can be converted via `.to_str()`).
3. `opts.hooks_settings_path` does not contain embedded null bytes.
4. `opts.session_id` is a non-empty string (UUID rendered as string).
5. `opts.project_root` is an absolute path to the session's working directory.

## Postconditions

1. Returns `Ok(SpawnRecipe)` where:
   - `recipe.binary` is the absolute `PathBuf` returned by `which::which("claude")` — the
     resolved path of the `claude` binary on `PATH` (e.g., `/usr/local/bin/claude`).
   - `recipe.args` is `["--settings", <hooks_settings_path as UTF-8 string>]` — exactly
     two elements in this order. No additional args are added.
   - `recipe.env` contains the key `"MONOCLE_SESSION_ID"` mapped to `opts.session_id`.
     If `opts.ccr_base_url` is `None`, `recipe.env` contains exactly one key
     (`"MONOCLE_SESSION_ID"`). If `opts.ccr_base_url` is `Some(_)`, `recipe.env` also
     contains `"ANTHROPIC_BASE_URL"` (see BC-2.03.006).
   - `recipe.cwd` equals `opts.project_root`.
2. The returned `SpawnRecipe` is fully populated — no `None` or empty fields.
3. `spawn_recipe()` is synchronous (non-async). It performs one filesystem lookup
   (`which::which`) and one UTF-8 conversion; no I/O beyond `PATH` resolution.
4. `spawn_recipe()` does NOT write any files. The `hooks_settings_path` is passed through
   as a CLI arg string; the file itself is written by the daemon before `spawn_recipe()` is
   called (existing OQ-02 mechanism).

## Invariants

1. The `--settings <hooks_settings_path>` argument is the hook injection mechanism.
   `SessionManager` MUST write `hooks-settings.json` to `opts.hooks_settings_path` BEFORE
   calling `spawn_recipe()`. The file at that path contains the 5-endpoint hook configuration
   with `lock.app = 'monocle'` filter.
2. `MONOCLE_SESSION_ID` is always injected regardless of CCR configuration. It enables
   correlation of hook events to specific sessions in the daemon's session registry.
3. The environment map (`recipe.env`) is MERGED with the child process's inherited
   environment by `monocle-session-host` at spawn time — it does NOT replace the entire
   environment. Variables not in `recipe.env` are inherited unchanged.
4. `recipe.cwd` MUST be the git worktree root for the project (claude-squad A.1
   worktree-per-task pattern). The caller (`SessionManager`) is responsible for resolving
   the correct worktree path before calling `spawn_recipe()`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-100 | `which::which("claude")` succeeds; `opts.ccr_base_url` is `None` | Returns `Ok(SpawnRecipe)` with `env = {"MONOCLE_SESSION_ID": opts.session_id}`; no `ANTHROPIC_BASE_URL` key in `recipe.env` |
| EC-101 | `which::which("claude")` succeeds; `opts.ccr_base_url` is `Some("http://localhost:8080")` | Returns `Ok(SpawnRecipe)` with `env = {"MONOCLE_SESSION_ID": opts.session_id, "ANTHROPIC_BASE_URL": "http://localhost:8080"}`; see BC-2.03.006 |
| EC-102 | `opts.project_root` is a path that exists on disk but is not a git repo | No error from `spawn_recipe()` — it uses `opts.project_root` as-is; session-host is spawned in that directory; git worktree validation (if required) is caller's responsibility |
| EC-103 | `which::which("claude")` fails (binary not on PATH) | Returns `Err(EngineError::BinaryNotFound("claude".into()))` — see BC-2.03.007 |
| EC-104 | `opts.hooks_settings_path` is non-UTF-8 | Returns `Err(EngineError::InvalidPath(...))` — see BC-2.03.007 (corrected: InvalidPath) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SpawnOptions { session_id: "sess-001", hooks_settings_path: "/tmp/hooks.json", ccr_base_url: None, project_root: "/home/user/project" }` with `claude` on PATH | `Ok(SpawnRecipe { binary: "/usr/local/bin/claude", args: ["--settings", "/tmp/hooks.json"], env: {"MONOCLE_SESSION_ID": "sess-001"}, cwd: "/home/user/project" })` | happy-path |
| Same but `ccr_base_url: Some("http://127.0.0.1:8080")` | `Ok(SpawnRecipe { ..., env: {"MONOCLE_SESSION_ID": "sess-001", "ANTHROPIC_BASE_URL": "http://127.0.0.1:8080"} })` | happy-path |
| `which::which("claude")` fails | `Err(EngineError::BinaryNotFound("claude"))` | error |
| `hooks_settings_path` with non-UTF-8 bytes | `Err(EngineError::InvalidPath(...))` | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `spawn_recipe()` returns `Ok(recipe)` with binary = `which("claude")`, args = `["--settings", path]`, `MONOCLE_SESSION_ID` in env, cwd = project_root | unit |
| VP-TBD | `MONOCLE_SESSION_ID` is always present in `recipe.env` regardless of CCR config | unit |
| VP-TBD | `recipe.cwd` equals `opts.project_root` verbatim | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC defines the spawn recipe assembly for the ClaudeCodeModule adapter, which is the mechanism by which the engine abstraction enables monocle to launch Claude Code sessions |
| L2 Domain Invariants | DI-007 (monocle must not write to any file owned by a harness or factory workflow system — PC-4 explicitly states spawn_recipe() writes no files; the hooks-settings.json path is passed through as a CLI arg string only) |
| Architecture Module | monocle-runtime (ClaudeCodeModule implementation — `monocle-runtime/src/engine/claude_code.rs`) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module-v2-delta.md v1.0.1 §ClaudeCodeModule::spawn_recipe() implementation spec; SS-session-manager.md v1.2.0 §SpawnRecipe integration with EngineModule |
| Stories | S-TBD (filled by story-writer) |
| Test Name | test_BC_2_03_005_spawn_recipe_happy_path_binary_args_env_cwd |

## Related BCs

- [BC-2.03.006] — composes with: CCR base URL injection into env (extends this BC's env map)
- [BC-2.03.007] — composes with: error cases when binary not found or path invalid
- [BC-2.03.008] — composes with: default trait impl for engines that do not support spawn

## Architecture Anchors

- `architecture/SS-engine-module-v2-delta.md#spawnrecipe-and-spawnoptions-types` — SpawnRecipe/SpawnOptions struct definitions
- `architecture/SS-engine-module-v2-delta.md#claudecodemodulespawn_recipe-implementation-spec` — implementation spec

## Story Anchor

S-TBD — Implement ClaudeCodeModule::spawn_recipe() with binary resolution, --settings arg, MONOCLE_SESSION_ID env (filled by story-writer)

## VP Anchors

VP-TBD — spawn_recipe() happy-path unit tests (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.03.005 authored for SS-03 as part of the v1A control-center pivot BC burst.
- Covers: spawn_recipe() happy-path — binary resolution, --settings arg construction,
  MONOCLE_SESSION_ID injection, cwd from project_root.
- Architecture source: SS-engine-module-v2-delta.md v1.0.1 (IMP-5 InvalidPath fix applied).
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
