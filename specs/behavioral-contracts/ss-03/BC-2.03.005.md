---
document_type: behavioral-contract
level: L3
version: "1.1.11"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-engine-module-v2-delta.md, architecture/SS-session-manager.md]
input-hash: "a7357a2"
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
to `opts.worktree_root` (the resolved git worktree root; equal to `project_root` only when no
per-session worktree applies). The returned recipe is consumed by `SessionManager` to spawn a
`monocle-session-host` process.

## Preconditions

1. The `claude` binary is locatable via `which::which("claude")` on the current `PATH`.
2. `opts.hooks_settings_path` is a valid UTF-8 path (can be converted via `.to_str()`).
3. `opts.hooks_settings_path` does not contain embedded null bytes.
4. `opts.session_id` is a non-empty string (UUID rendered as string).
5. `opts.worktree_root` is an absolute, validated path to the harness child's working
   directory — the resolved git worktree root (or `project_root` when no worktree applies,
   per the three-rule algorithm in SS-session-manager.md §SpawnOptions.worktree_root).

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
   - `recipe.cwd` equals `opts.worktree_root` — the resolved worktree root path (which equals
     `opts.project_root` when no git worktree is configured or when the project is not a git
     repo; see three-rule algorithm in SS-session-manager.md §SpawnOptions.worktree_root).
     NEVER hardcoded to `opts.project_root` directly.
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
4. `recipe.cwd` MUST be `opts.worktree_root` — the resolved worktree root path from
   `SpawnOptions` (claude-squad A.1 worktree-per-session pattern). The caller
   (`SessionManager` via `SessionCreation` wizard) resolves the worktree path in Step 3
   (WorktreeConfirm) before calling `spawn_recipe()`. `spawn_recipe()` uses the pre-resolved
   path verbatim — it does NOT perform its own worktree resolution.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-100 | `which::which("claude")` succeeds; `opts.ccr_base_url` is `None` | Returns `Ok(SpawnRecipe)` with `env = {"MONOCLE_SESSION_ID": opts.session_id}`; no `ANTHROPIC_BASE_URL` key in `recipe.env` |
| EC-101 | `which::which("claude")` succeeds; `opts.ccr_base_url` is `Some("http://localhost:8080")` | Returns `Ok(SpawnRecipe)` with `env = {"MONOCLE_SESSION_ID": opts.session_id, "ANTHROPIC_BASE_URL": "http://localhost:8080"}`; see BC-2.03.006 |
| EC-102 | `opts.worktree_root` resolves (via three-rule algorithm) to a path that exists on disk but is not a git repo (non-git project; three-rule fallback: `worktree_root = project_root`) | No error from `spawn_recipe()` — it uses `opts.worktree_root` as-is (which equals `opts.project_root` for non-git projects per the three-rule fallback); session-host is spawned in that directory; git worktree validation (if required) is caller's responsibility before passing `worktree_root` to `spawn_recipe()` |
| EC-103 | `which::which("claude")` fails (binary not on PATH) | Returns `Err(EngineError::BinaryNotFound("claude".into()))` — see BC-2.03.007 |
| EC-104 | `opts.hooks_settings_path` is non-UTF-8 (Prong 1: `to_str()` returns `None`) | Returns `Err(EngineError::InvalidPath(...))` — see BC-2.03.007 PC-5 Prong 1 (corrected: InvalidPath) |
| EC-104b | `opts.hooks_settings_path` is valid UTF-8 but contains an embedded null byte (Prong 2: explicit `as_bytes().contains(&0)` scan fires) | Returns `Err(EngineError::InvalidPath(...))` — `to_str()` alone CANNOT detect null bytes (null is valid UTF-8); the explicit null scan is required; same `EngineError::InvalidPath` variant and `"invalid_spawn_arg"` IPC code as EC-104 — see BC-2.03.007 PC-5 Prong 2 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SpawnOptions { session_id: "sess-001", hooks_settings_path: "/tmp/monocle-rt/hooks-settings.json", ccr_base_url: None, worktree_root: "/home/user/project/worktree-feature" }` with `claude` on PATH | `Ok(SpawnRecipe { binary: "/usr/local/bin/claude", args: ["--settings", "/tmp/monocle-rt/hooks-settings.json"], env: {"MONOCLE_SESSION_ID": "sess-001"}, cwd: "/home/user/project/worktree-feature" })` | happy-path |
| Same but `ccr_base_url: Some("http://127.0.0.1:8080")` | `Ok(SpawnRecipe { ..., env: {"MONOCLE_SESSION_ID": "sess-001", "ANTHROPIC_BASE_URL": "http://127.0.0.1:8080"} })` | happy-path |
| Non-git project: `worktree_root = project_root = "/home/user/project"` | `recipe.cwd = "/home/user/project"` (three-rule fallback; worktree_root equals project_root for non-git) | happy-path |
| `which::which("claude")` fails | `Err(EngineError::BinaryNotFound("claude"))` | error |
| `hooks_settings_path` with non-UTF-8 bytes | `Err(EngineError::InvalidPath(...))` | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `spawn_recipe()` returns `Ok(recipe)` with binary = `which("claude")`, args = `["--settings", path]`, `MONOCLE_SESSION_ID` in env, cwd = `opts.worktree_root` | unit |
| VP-TBD | `MONOCLE_SESSION_ID` is always present in `recipe.env` regardless of CCR config | unit |
| VP-TBD | `recipe.cwd` equals `opts.worktree_root` (not `opts.project_root`); for non-git projects the two are equal | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC defines the spawn recipe assembly for the ClaudeCodeModule adapter, which is the mechanism by which the engine abstraction enables monocle to launch Claude Code sessions |
| L2 Domain Invariants | DI-007 (monocle must not write to any file owned by a harness or factory workflow system — PC-4 explicitly states spawn_recipe() writes no files; the hooks-settings.json path is passed through as a CLI arg string only) |
| Architecture Module | monocle-runtime (ClaudeCodeModule implementation — `monocle-runtime/src/engine/claude_code.rs`) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module-v2-delta.md v1.6.0 §ClaudeCodeModule::spawn_recipe() implementation spec (two-pronged null-byte detection — C34-001); SS-session-manager.md v2.17.1 §SpawnRecipe integration with EngineModule |
| Stories | S-045 |
| Test Name | test_BC_2_03_005_spawn_recipe_happy_path_binary_args_env_cwd |

## Related BCs

- [BC-2.03.006] — composes with: CCR base URL injection into env (extends this BC's env map)
- [BC-2.03.007] — composes with: error cases when binary not found or path invalid
- [BC-2.03.008] — composes with: default trait impl for engines that do not support spawn

## Architecture Anchors

- `architecture/SS-engine-module-v2-delta.md#spawnrecipe-and-spawnoptions-types` — SpawnRecipe/SpawnOptions struct definitions
- `architecture/SS-engine-module-v2-delta.md#claudecodemodulespawn_recipe-implementation-spec` — implementation spec

## Story Anchor

S-045 — Implement ClaudeCodeModule::spawn_recipe() with binary resolution, --settings arg, MONOCLE_SESSION_ID env

## VP Anchors

VP-TBD — spawn_recipe() happy-path unit tests (filled after VP creation)


## §Trace 1.1.11

**SS-session-manager arch-source pin cascade v2.17.0→v2.17.1 (F-S042-ADV-MED-001 ownership-drift cleanup)** (2026-06-21):
- Architecture Source pin updated: SS-session-manager.md v2.17.0 → v2.17.1. No behavioral content changed.
- SE-16d monotonicity: 1.1.11 > 1.1.10. PASS.

## §Trace 1.1.10

**SS-session-manager arch-source pin cascade v2.16.0→v2.17.0** (2026-06-21):
- Architecture Source pin updated: SS-session-manager.md v2.16.0 → v2.17.0. No behavioral content changed.
- SE-16d monotonicity: 1.1.10 > 1.1.9. PASS.

## §Trace v1.1.9

**SS-session-manager v2.15.1 → v2.16.0 Architecture Source pin cascade (Ruling A errata)** (2026-06-21):
- Architecture Source pin updated. No behavioral content changed.
- SE-16d monotonicity: v1.1.9 > v1.1.8. PASS.

## §Trace v1.1.6

**SS-session-manager v2.13.0 → v2.14.0 Architecture Source pin cascade (F-S035-PASS5-MED-001)** (2026-06-19T00:00:00Z):
- Architecture Source pin: SS-session-manager.md v2.13.0 → v2.14.0 (v2.14.0 adds EC-188
  timeout → Terminated subpath (d) in the attach_session Detached cell of the action×state
  matrix — F-S035-PASS5-MED-001). No behavioral content changes to this BC.
- SE-16d monotonicity: v1.1.6 timestamp 2026-06-19 >= v1.1.5 timestamp auto. PASS.

## §Trace v1.1.4

**Burst-E D-305 — Story Anchor and Traceability Stories resolved: S-TBD → S-045** (2026-06-15):
- Story Anchor and Traceability §Stories row filled from Phase-2 Burst C story decomposition. No behavioral content changed.

## §Trace v1.1.3

**C34-001 — Add EC-104b for null-byte precondition violation; arch-source pin v1.4.0→v1.4.1** (2026-06-13 / D-276):

- **Context:** SS-engine-module-v2-delta.md v1.5.0 (C34-001) corrected the null-byte detection
  mechanism in `spawn_recipe()` to a two-pronged check. Prong 1: `to_str()` returns `None`
  for non-UTF-8 paths. Prong 2: explicit `path_str.as_bytes().contains(&0)` scan for embedded
  null bytes (required because null is valid UTF-8 and `to_str()` returns `Some` for paths
  containing null bytes).

- **EC-104 relabeled:** Prong 1 (non-UTF-8 → `to_str()` returns `None`) now explicitly
  cited as Prong 1 in the EC-104 description to match BC-2.03.007 PC-5 terminology.

- **EC-104b added:** New edge case covering Prong 2 — path is valid UTF-8 but contains an
  embedded null byte. `to_str()` returns `Some` (cannot detect the null); explicit scan
  `path_str.as_bytes().contains(&0)` fires and returns `Err(EngineError::InvalidPath(...))`.
  Same wire-code (`"invalid_spawn_arg"`) as EC-104. Ensures BC-2.03.005 and BC-2.03.007
  agree on the complete `InvalidPath` coverage.

- **Precondition 3** ("does not contain embedded null bytes") remains correct as a
  precondition — EC-104b documents the violation path for callers that fail to satisfy it.

- **Architecture Source pin:** v1.4.0 → v1.4.1; §spawn_recipe() two-pronged null-byte
  detection cited.

- No behavioral change to happy-path postconditions. Patch bump only.

## §Trace v1.1.2

**I13-001 + S13-001 — Complete worktree_root propagation (residual stale CWD-source locations)** (2026-06-04):
- Context: This is the 3rd remediation pass for the I2-002 cwd = project_root → worktree_root
  correction. Prior passes (v1.1.0, v1.1.1) each claimed completion but each missed one
  location. This pass exhaustively grepped every `project_root` and `cwd` occurrence and
  classified each before editing.
- I13-001 (IMPORTANT): VP table row 1 (line ~112) still asserted `cwd = project_root`.
  Fixed: changed to `cwd = \`opts.worktree_root\`` — now consistent with VP row 3, PC-1,
  Invariant 4, and Description.
- S13-001 (SUGGESTION): EC-102 described the CWD source as "it uses `opts.project_root`
  as-is". Fixed: rewritten to reference `opts.worktree_root` as the source (which equals
  `opts.project_root` for non-git projects per the three-rule fallback). The scenario now
  correctly models spawn_recipe() behavior: it always consumes `opts.worktree_root`; the
  caller resolves worktree_root before calling spawn_recipe().
- §Trace v1.1.0 false attestation corrected: the claim "VP: updated from project_root to
  worktree_root" is annotated to acknowledge it was incomplete (only VP row 3 was fixed;
  VP row 1 was missed).
- S13-002 adjudication (NO CHANGE): BC-2.03.005 Invariant 3 uses "MERGED". BC-2.03.006
  §Trace v1.1.0 explicitly ratifies: "Invariant 3 was already correct ('MERGED with the
  child process's inherited environment') — no change needed there". "MERGED" is the
  deliberately-ratified phrasing; "OVERLAY" is the term used in BC-2.03.006's Description
  for the overall semantics. Both terms are correct in context. No change to Invariant 3.
- Proof: post-edit grep for CWD-source `project_root` occurrences → zero stale occurrences
  remain (all remaining `project_root` references are legitimate SpawnOptions field references
  explaining the three-rule fallback relationship).
- Scope: coherence/propagation completion; no behavioral change. Version bumped as patch 1.1.1→1.1.2.

## §Trace v1.1.1

**I12-002 — Description partial-fix regression corrected** (2026-06-04):
- Finding (I12-002): The §Description prose said "sets the working directory to
  `opts.project_root`" — this contradicts the v1.1.0 worktree_root correction applied to
  Precondition 5, PC-1 cwd, Invariant 4, the canonical test vectors, and the VP. The
  §Trace v1.1.0 changelog enumerated those fix targets but missed the Description, leaving a
  partial-fix regression: an implementer reading the lead normative prose would set
  `cwd = project_root` and silently re-break worktree-per-session (claude-squad A.1 gene).
- Fix: Description updated — "sets the working directory to `opts.worktree_root` (the resolved
  git worktree root; equal to `project_root` only when no per-session worktree applies)" —
  mirroring the exact phrasing and intent of PC-1 and Invariant 4 in this file.
- Scope: wording-coherence correction only; no behavioral change. Version bumped as patch.

## §Trace v1.1.0

**Architect-delegated BC edit — cwd = worktree_root, not project_root (I2-002)** (2026-06-03):
- I2-002 finding: BC-2.03.005 set `recipe.cwd = opts.project_root` (incorrect). The
  architecture (SS-session-manager.md v1.5.0 §SpawnRecipe integration) specifies that
  `recipe.cwd` is populated from `SpawnOptions.worktree_root` — the resolved worktree root
  path, which equals `project_root` only when no git worktree is configured (three-rule
  algorithm). For git repos with worktrees, `cwd` is the worktree root, not `project_root`.
- Precondition 5: changed from `opts.project_root` to `opts.worktree_root`.
- PC-1 cwd field: changed from `opts.project_root` to `opts.worktree_root` with explanation.
- Invariant 4: rewritten — `recipe.cwd = opts.worktree_root`; caller resolves at wizard Step 3.
- Canonical test vectors: updated to use `worktree_root` field and show non-git fallback.
- VP: **partially** updated — VP row 3 updated from `project_root` to `worktree_root`; VP row 1
  was missed (residual stale CWD-source `project_root` — corrected in v1.1.2 per I13-001).
  NOTE: The original attestation "VP: updated from project_root to worktree_root" was
  INCOMPLETE — VP row 1 still said `cwd = project_root` after this pass. Corrected in v1.1.2.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.03.005 authored for SS-03 as part of the v1A control-center pivot BC burst.
- Covers: spawn_recipe() happy-path — binary resolution, --settings arg construction,
  MONOCLE_SESSION_ID injection, cwd from project_root.
- Architecture source: SS-engine-module-v2-delta.md v1.1.0 (IMP-5 InvalidPath fix applied). <!-- version-pin-historical: §Trace initial-production record; v1.1.0 is the spec version at BC authoring time -->
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).

## §Trace v1.1.5

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.1.5 timestamp >= v1.1.4. PASS.
