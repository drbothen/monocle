---
document_type: behavioral-contract
level: L3
version: "1.2.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-engine-module-v2-delta.md, architecture/SS-ipc.md, architecture/SS-session-manager.md]
input-hash: "7ef03d0"
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

# Behavioral Contract BC-2.03.007: spawn_recipe() Error Cases — BinaryNotFound and InvalidPath

## Description

`ClaudeCodeModule::spawn_recipe()` returns typed errors for two distinct failure modes:
`EngineError::BinaryNotFound` when `which::which("claude")` cannot locate the harness
binary on `PATH`, and `EngineError::InvalidPath` when `opts.hooks_settings_path` cannot
be converted to a valid UTF-8 string (required for CLI arg passing). These two error modes
are categorically distinct — `BinaryNotFound` means the harness is not installed;
`InvalidPath` means the supplied configuration is structurally invalid. The distinction
matters for diagnostic accuracy.

## Preconditions

1. `ClaudeCodeModule` is instantiated.
2. `spawn_recipe()` is called with a `SpawnOptions` value.

## Postconditions

### BinaryNotFound path

1. When `which::which("claude")` fails (the `claude` binary is not found on `PATH`),
   `spawn_recipe()` returns `Err(EngineError::BinaryNotFound("claude".into()))`.
2. The `BinaryNotFound` variant carries the string `"claude"` identifying which binary
   was not found. The error message format is: `"harness binary not found: claude"`.
3. On receiving `BinaryNotFound`, the error propagates as `EngineError::BinaryNotFound` →
   `SessionError::EngineError` → `ServerToClient::Error { code: "binary_not_found", message }`.
   The TUI MUST display the fixed banner: `"claude binary not found — is Claude Code installed
   and on PATH?"`. The session spawn fails; no `monocle-session-host` process is started.
   Canonical code: `"binary_not_found"` (SS-ipc.md v1.24.0 §ServerToClient::Error taxonomy;
   SS-session-manager.md v2.6.1 §session_error_to_code spawn-path arms).
4. `BinaryNotFound` is NOT returned for any other failure mode. It is reserved exclusively
   for `which::which` failures.

### InvalidPath path

5. `spawn_recipe()` applies a **two-pronged check** to `opts.hooks_settings_path`:
   - **Prong 1 — non-UTF-8:** `opts.hooks_settings_path.to_str()` returns `None` (path bytes
     are not valid UTF-8). `to_str()` cannot detect null bytes because null (`\x00`) is valid
     UTF-8 and `to_str()` returns `Some` for paths containing null bytes.
   - **Prong 2 — embedded null byte:** After a successful `to_str()`, an explicit scan via
     `path_str.as_bytes().contains(&0)` detects any embedded null byte and returns
     `Err(EngineError::InvalidPath(...))`. This explicit scan is required because `to_str()`
     alone cannot detect null bytes.
   In either prong, `spawn_recipe()` returns
   `Err(EngineError::InvalidPath(format!("hooks_settings_path is not valid UTF-8: {:?}", opts.hooks_settings_path)))`.
6. The `InvalidPath` variant carries a descriptive message including the problematic path
   rendered via `{:?}` formatting. The error message format is:
   `"invalid argument: hooks_settings_path is not valid UTF-8: <path_debug>"`.
   Both prongs produce the same `EngineError::InvalidPath` variant and IPC error code
   (`"invalid_spawn_arg"`); they are treated identically at the wire level.
7. On receiving `InvalidPath`, the error propagates as `EngineError::InvalidPath` →
   `SessionError::EngineError` → `ServerToClient::Error { code: "invalid_spawn_arg", message }`.
   The TUI MUST display the fixed banner: `"Session spawn failed: invalid hooks settings path
   (non-UTF-8)"`. The session spawn fails; no `monocle-session-host` process is started.
   Canonical code: `"invalid_spawn_arg"` (SS-ipc.md v1.24.0 §ServerToClient::Error taxonomy;
   SS-session-manager.md v2.6.1 §session_error_to_code spawn-path arms).
8. `InvalidPath` is NOT used for binary-not-found. The two variants MUST NOT be conflated.

## Invariants

1. **Semantic separation is mandatory:** `BinaryNotFound` = harness not installed (which failure).
   `InvalidPath` = argument structurally invalid (non-UTF-8 bytes OR embedded null byte). The
   two-pronged detection mechanism (non-UTF-8 via `to_str()` returning `None`; embedded null
   via explicit `as_bytes().contains(&0)` scan) is required because `to_str()` alone cannot
   detect null bytes — null (`\x00`) is valid UTF-8. The original draft of this BC incorrectly
   proposed using `BinaryNotFound` for both cases. SS-engine-module-v2-delta.md §Trace v1.0.1
   (IMP-5) corrected this; this BC encodes the corrected taxonomy. SS-engine-module-v2-delta.md
   §spawn_recipe() further corrected the detection mechanism at v1.4.1 authoring time (C34-001); this BC v1.2.2
   encodes that correction.
2. `spawn_recipe()` checks binary existence FIRST (via `which::which`); if that succeeds,
   it then checks UTF-8 path validity. The order of checks is: (1) binary, (2) args validity.
   Early return on first failure.
3. Neither error variant is retried by the caller. Both are terminal failures requiring user
   intervention (install the binary, or fix the daemon's hooks-settings path generation).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-108 | `claude` binary is on PATH but not executable (permission denied) | `which::which` succeeds (returns path); execution permission failure occurs later at spawn time (in `monocle-session-host`); `spawn_recipe()` returns `Ok(recipe)` — it does not check executability |
| EC-109 | `hooks_settings_path` contains embedded null bytes (not just non-UTF-8) | `to_str()` returns `Some(path_str)` because null (`\x00`) is valid UTF-8 — `to_str()` alone CANNOT detect null bytes; the explicit scan `path_str.as_bytes().contains(&0)` detects the null and returns `Err(EngineError::InvalidPath(...))`; treated identically to non-UTF-8 at the IPC wire-code level (`"invalid_spawn_arg"`) |
| EC-110 | `hooks_settings_path` is valid UTF-8 but the file does not exist at that path | `spawn_recipe()` returns `Ok(recipe)` — it does not check file existence; the `--settings <path>` arg is passed through; `claude` will fail to read the settings file at runtime |
| EC-111 | Both `which::which` fails AND `hooks_settings_path` is invalid UTF-8 | Returns `Err(EngineError::BinaryNotFound("claude"))` — binary check is first; arg validation is never reached |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `claude` not on PATH | `Err(EngineError::BinaryNotFound("claude"))` | error |
| `hooks_settings_path` with `\xFF\xFE` non-UTF-8 bytes | `Err(EngineError::InvalidPath("hooks_settings_path is not valid UTF-8: ..."))` | error |
| `claude` not on PATH AND invalid hooks path | `Err(EngineError::BinaryNotFound("claude"))` — binary checked first | error |
| `claude` on PATH AND valid UTF-8 hooks path | `Ok(SpawnRecipe {...})` — no error | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `which::which` failure → `BinaryNotFound("claude")`; never `InvalidPath` | unit |
| VP-TBD | Non-UTF-8 `hooks_settings_path` → `InvalidPath`; never `BinaryNotFound` | unit |
| VP-TBD | Binary check precedes arg check (both fail: BinaryNotFound returned) | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC defines the error taxonomy for spawn_recipe(), which is a method on the ClaudeCodeModule adapter; typed errors are essential for diagnostic accuracy in the engine abstraction layer |
| L2 Domain Invariants | DI-006 (EngineModule implementations must be stateless — error variants carry no shared state; both errors are pure value returns) |
| Architecture Module | monocle-runtime (ClaudeCodeModule — `monocle-runtime/src/engine/claude_code.rs`); monocle-core (`EngineError` type) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module-v2-delta.md v1.6.0 §EngineError (new in v1A) + §Semantic contract (IMP-5 InvalidPath correction) + §spawn_recipe() two-pronged null-byte detection (C34-001) + §Phase Compatibility (I27-001 Model A: spawn_recipe() called daemon-side inside spawn_session()); SS-ipc.md v1.24.0 §ServerToClient::Error taxonomy (codes `"binary_not_found"` and `"invalid_spawn_arg"` — I12-001); SS-session-manager.md v2.6.1 §session_error_to_code spawn-path arms (EngineError bridge — I12-001; Model A reachability confirmed — I27-001) |
| Test Name | test_BC_2_03_007_spawn_recipe_binary_not_found_and_invalid_path |

## Related BCs

- [BC-2.03.005] — composes with: this BC covers the error branches of the same method
- [BC-2.03.001] — depends on: EngineError is the new error type returned by spawn_recipe() in the EngineModule trait (new in v1A; defined in SS-engine-module-v2-delta.md §EngineError (new in v1A))

## Architecture Anchors

- `architecture/SS-engine-module-v2-delta.md#engineerror-new-in-v1a` — BinaryNotFound and InvalidPath variant definitions
- `architecture/SS-engine-module-v2-delta.md#semantic-contract` — IMP-5 semantic separation ruling

## Story Anchor

S-045 — Same story as BC-2.03.005 (error handling in spawn_recipe())

## VP Anchors

VP-TBD — spawn_recipe() error path unit tests (filled after VP creation)

## §Trace v1.2.3

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-045** (2026-06-15):
- Story Anchor filled from Phase-2 Burst C story decomposition (clusters with BC-2.03.005). No behavioral content changed.

## §Trace v1.2.2

**C34-001 — Corrected null-byte detection mechanism; arch-source pin v1.4.0→v1.4.1** (2026-06-13 / D-276):

- **Root cause:** The old EC-109 text claimed "`to_str()` returns `None` (null bytes invalidate
  path conversion)". This is false: null (`\x00`) is valid UTF-8, so `to_str()` returns `Some`
  for paths with embedded null bytes and cannot detect them. The correct mechanism
  (established by SS-engine-module-v2-delta.md v1.5.0 §spawn_recipe() C34-001 fix) is a
  two-pronged check: Prong 1 = `to_str()` returns `None` (genuine non-UTF-8); Prong 2 =
  explicit `path_str.as_bytes().contains(&0)` scan (null bytes, valid UTF-8 but OS-rejected).

- **EC-109 rewritten:** Old: "`to_str()` returns `None` (null bytes invalidate path conversion)".
  New: "`to_str()` returns `Some(path_str)` because null is valid UTF-8 — explicit scan
  `path_str.as_bytes().contains(&0)` detects the null and returns `Err(InvalidPath(...))`;
  treated identically to non-UTF-8 at the `"invalid_spawn_arg"` wire-code level."

- **Postcondition 5 rewritten:** Now documents both prongs with explicit statement that
  `to_str()` alone cannot detect null bytes. Same `EngineError::InvalidPath` variant and
  `"invalid_spawn_arg"` IPC code for both prongs (wire-level identity preserved).

- **Postcondition 6 extended:** Added clarification that both prongs produce the same variant
  and IPC code, so they are treated identically at the wire level.

- **Invariant 1 updated:** Extended to document the two-pronged detection mechanism and cite
  the C34-001 correction in SS-engine-module-v2-delta.md v1.5.0.

- **Architecture Source pin:** v1.4.0 → v1.4.1 (architect's C34-001 bump); §spawn_recipe()
  two-pronged null-byte detection added to citation.

- No behavioral outcome changed (InvalidPath → invalid_spawn_arg wire code unchanged). Only
  the mechanism description was wrong. Patch bump only.

## §Trace v1.2.1

**Pass-32 — IMP-002 + IMP-003: dead anchor corrected; stale "extension" framing removed** (2026-06-13):

- **IMP-002 (Architecture Anchors):** The anchor `#engineerror-additions` was a dead cross-reference
  — the heading it targeted was renamed in Pass-31 from "### EngineError additions" to
  "### EngineError (new in v1A)". Updated to the current GitHub slug `#engineerror-new-in-v1a`.
  Verified against SS-engine-module-v2-delta.md line 210: `### EngineError (new in v1A)`.

- **IMP-003 (Related BCs):** The BC-2.03.001 dependency description incorrectly framed
  EngineError as "an extension of the error taxonomy used in the EngineModule trait". This
  contradicts the ratified Pass-31 decision (SS-engine-module-v2-delta.md §Trace v1.4.0 /
  §EngineError (new in v1A)): EngineError is a NEW type introduced in v1A, not an extension
  of the pre-existing EngineModule error taxonomy. Reworded to describe EngineError as the
  new error type returned by `spawn_recipe()` in the EngineModule trait (new in v1A).

- No behavioral content changed (Preconditions, Postconditions, Invariants, Edge Cases, Test
  Vectors unchanged). Patch bump only.

## §Trace v1.2.0

**I27-001 (Model A) — reachability confirmation: PC-3/PC-7 propagation chain verified under daemon-side spawn_recipe()** (2026-06-13):

- **Reachability under Model A:** Under the I27-001 Model A adjudication,
  `ClientToServer::SpawnSession { opts: SpawnOptions }` is the wire message. The daemon's
  IPC handler calls `SessionManager::spawn_session(opts)`, which calls
  `engine_module.spawn_recipe(&opts)?` as its FIRST step (before any OS process is spawned).
  `BinaryNotFound` and `InvalidPath` errors produced by `spawn_recipe()` propagate via `?`
  as `SessionError::EngineError(...)` and are mapped by `session_error_to_code(IpcOp::Spawn,
  &e)` to `"binary_not_found"` / `"invalid_spawn_arg"` respectively. This confirms PC-3 and
  PC-7 are FULLY REACHABLE in Model A — the propagation chains they specify are the live paths.
- **Content verified correct:** No residual Model B (TUI-builds-recipe) assumptions found in
  PC-3, PC-7, Invariants, or Edge Cases. The BC's description of `spawn_recipe()` as a method
  on `ClaudeCodeModule` returning typed errors is independent of which caller invokes it.
- **Architecture Source updated:** Version pins advanced to current canonical versions:
  SS-engine-module-v2-delta.md `v1.1.1` → `v1.2.0`; SS-ipc.md `v1.18.0` → `v1.19.0`;
  SS-session-manager.md `v1.9.0` → `v2.0.0`. SS-engine-module-v2-delta.md v1.2.0 citation
  updated to include §Phase Compatibility reference (which documents Model A wire-type
  assignments). <!-- version-pin-historical: §Trace I27-001 record; prior pins v1.1.1/v1.18.0/v1.9.0 in §Trace v1.1.0 are preserved as historical annotations -->
- No behavioral content changed (PC-3/PC-7/Invariants/Edge Cases unchanged).
- Version bump: 1.1.0 → 1.2.0 (minor: Architecture Source version citations updated;
  reachability confirmation added to §Trace).

## §Trace v1.1.0

**I12-001 — PC-3/PC-7 now cite canonical IPC error codes; Architecture Source extended** (2026-06-04):
- Finding (I12-001): BC-2.03.007 PC-3 and PC-7 specified the correct user-facing banners but did
  not cite the canonical `ServerToClient::Error` codes through which those banners are delivered.
  The architect extended the `ServerToClient::Error` taxonomy (SS-ipc.md v1.17.0, 8→10 codes) <!-- version-pin-historical: §Trace I12-001 record; v1.17.0 is SS-ipc at Pass-12 fix time -->
  and added the `SessionError::EngineError` bridge (SS-session-manager.md v1.8.0
  `session_error_to_code()`) so that both distinct user messages are now satisfiable via the
  IPC wire protocol.
- PC-3 updated: `BinaryNotFound` propagation chain cited explicitly —
  `EngineError::BinaryNotFound` → `SessionError::EngineError` →
  `ServerToClient::Error { code: "binary_not_found", message }`. Canonical code
  `"binary_not_found"` cited with source.
- PC-7 updated: `InvalidPath` propagation chain cited explicitly —
  `EngineError::InvalidPath` → `SessionError::EngineError` →
  `ServerToClient::Error { code: "invalid_spawn_arg", message }`. Canonical code
  `"invalid_spawn_arg"` cited with source.
- Architecture Source row extended to cite SS-ipc.md v1.17.0 §ServerToClient::Error taxonomy <!-- version-pin-historical: §Trace I12-001 record; v1.17.0 is SS-ipc at Pass-12 fix time -->
  AND SS-session-manager.md v1.8.0 §session_error_to_code spawn-path arms. <!-- version-pin-historical: §Trace I12-001 record; v1.8.0 is SS-session-manager at Pass-12 fix time -->
- inputs frontmatter extended to include SS-ipc.md and SS-session-manager.md.
- Cross-reference verification confirmed:
  - SS-ipc.md v1.17.0 lines 396-397: `"binary_not_found"` and `"invalid_spawn_arg"` present <!-- version-pin-historical: §Trace cross-reference verification at Pass-12; v1.17.0 is SS-ipc at that time -->
    in `ServerToClient::Error` taxonomy table.
  - SS-session-manager.md v1.8.0 lines 411, 471-473: `SessionError::EngineError(#[from] <!-- version-pin-historical: §Trace cross-reference verification at Pass-12; v1.8.0 is SS-session-manager at that time -->
    EngineError)` variant and `session_error_to_code()` match arms confirmed present.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.03.007 authored for SS-03 as part of the v1A control-center pivot BC burst.
- Title corrected per SS-engine-module-v2-delta.md §Trace v1.0.1 (IMP-5): original architect
  proposal was "spawn_recipe() with non-UTF-8 hooks_settings_path returns BinaryNotFound error"
  — this is wrong. The corrected title covers both BinaryNotFound AND InvalidPath in one BC.
  The architect's IMP-5 fix established that InvalidPath is a distinct variant; this BC encodes
  the full error taxonomy for spawn_recipe().
- Design decision (in-scope): Consolidated architect's proposed BC-2.03.007 (non-UTF-8 → InvalidPath)
  and implicit BinaryNotFound error coverage into a single BC covering the full spawn_recipe() error
  taxonomy. Rationale: both errors are preconditions of the same method; splitting them would create
  two BCs covering two exit arms of one function, harming test vector organization.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).

## §Trace v1.2.4

**Phase-2 Pass-1 fix burst — SS-ipc v1.24.0 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source SS-ipc pin v1.23.2 → v1.24.0 (3 occurrences). Plain version-pin refresh — §Daemon-Side Per-Client Fan-out Channel added to SS-ipc.md v1.24.0; pre-existing §Transport and §Lifecycle section anchors in this BC are unaffected.
- SE-16d monotonicity: v1.2.4 timestamp >= v1.2.3. PASS.

## §Trace v1.2.5

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.2.5 timestamp >= v1.2.4. PASS.
