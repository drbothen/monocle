---
document_type: behavioral-contract
level: L3
version: "1.0.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:13:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "76564f0"
traces_to: prd.md
origin: greenfield
subsystem: SS-03
capability: CAP-003
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.03.004: ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight)

## Description

`ClaudeCodeModule` exposes three inherent methods NOT in the `EngineModule` trait:
`hook_paths()`, `spawn()`, and `preflight()`. `hook_paths()` returns a
`HashMap<HookType, String>` with exactly 5 entries matching the canonical Phase 1 endpoint
set (JC-2 parity: `PostToolUse` is omitted). `spawn()` and `preflight()` are Phase 1 stubs
(`todo!()`) with binding signatures. ABI version is read from
`monocle_core::MONOCLE_ABI_VERSION` at call sites — no `abi_version` method on any trait.

## Preconditions

1. `ClaudeCodeModule` is instantiated via `ClaudeCodeModule::new("http://127.0.0.1:7891".into())`.

## Postconditions

1. `ClaudeCodeModule::hook_paths()` is an inherent method (NOT a trait method) that returns `HashMap<HookType, String>` with exactly 5 entries:
   - `HookType::SessionStart` → `"/hooks/session-start"`
   - `HookType::UserPromptSubmit` → `"/hooks/prompt-submit"`
   - `HookType::PreToolUse` → `"/hooks/pre-tool-use"`
   - `HookType::Notification` → `"/hooks/notification"`
   - `HookType::Stop` → `"/hooks/stop"`
2. `ClaudeCodeModule::spawn(args: SpawnArgs) -> Result<SessionHandle, SpawnError>` is an inherent async method. Phase 1 implementation is `todo!()` — the signature is binding; the implementation is stubbed.
3. `ClaudeCodeModule::preflight() -> Result<EngineVersion, PreflightError>` is an inherent async method. Phase 1 implementation is `todo!()` — the signature is binding.
4. The ABI version is read as `monocle_core::MONOCLE_ABI_VERSION` at call sites. No `abi_version` method appears on any trait.
5. These three methods (`hook_paths`, `spawn`, `preflight`) are NOT in the `EngineModule` trait. They are engine-specific operational methods on the concrete struct.

## Invariants

1. The 5 hook path strings exactly match the canonical endpoint set from brief §Scope (§In Scope sub-bullets for hook endpoints): `PostToolUse` is NOT included (JC-2 gene-source parity).
2. Path strings begin with `/` (relative to the daemon's base URL).
3. The `hook_paths()` method is synchronous — it returns a static mapping. No I/O, no async.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-038 | `spawn()` called in Phase 1 with a valid `SpawnArgs` | Returns `todo!()` panic — intentional; Phase 1 daemon receives hook POSTs from externally-started sessions, does not spawn them |
| EC-039 | `preflight()` called at daemon startup before accepting hook registrations | Returns `todo!()` panic in Phase 1 stub; Phase 1 story replaces the stub with `which claude` + `claude --version` checks |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| `hook_paths().len()` | `5` | happy-path |
| `hook_paths()[HookType::PreToolUse]` | `"/hooks/pre-tool-use"` | happy-path |
| `hook_paths()[HookType::Stop]` | `"/hooks/stop"` | happy-path |
| `hook_paths()[HookType::UserPromptSubmit]` | `"/hooks/prompt-submit"` | happy-path |
| `hook_paths()[HookType::SessionStart]` | `"/hooks/session-start"` | happy-path |
| `hook_paths()[HookType::Notification]` | `"/hooks/notification"` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | `module.hook_paths().len() == 5` and each `HookType` maps to the exact path string from the canonical table | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC governs the hook path routing and operational method surface of the ClaudeCodeModule adapter named in CAP-003 |
| L2 Domain Invariants | DI-006 (every EngineModule implementation must be stateless with respect to process detection — hook_paths() is synchronous, performs no I/O, and returns a static mapping from a pure in-memory HashMap construction; Invariant 3 explicitly states "no I/O, no async" for this method, satisfying DI-006's stateless detection requirement); DI-007 (monocle must not write to any file owned by a harness or factory workflow system — hook_paths() returns read-only routing strings and writes to no files; spawn() and preflight() are stubs in Phase 1, preventing any write path from being exercised against harness-owned paths) |
| Architecture Module | monocle-core (EngineModule trait, EnrichedSession, HookEvent types); monocle-runtime (ClaudeCodeModule implementation — monocle-runtime/src/engine/claude_code.rs) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module.md v1.1.20 §Struct-level inherent operations |
| FC | JC-2 (5-endpoint parity, PostToolUse omitted) |
| Brief Section | §Scope (§In Scope sub-bullets for hook endpoints — 5 canonical endpoints) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-ENGINE-003 |
| Test name | test_BC_ENGINE_003_claude_module_hook_paths_five_entries |

## Related BCs (Recommended)

- [BC-2.03.002] — composes with: inherent methods belong to the ClaudeCodeModule struct defined in BC-2.03.002
- [BC-2.01.001] — composes with: hook_paths() values match the daemon HTTP endpoint paths registered by BC-2.01 contracts

## Architecture Anchors (Recommended)

- `architecture/SS-engine-module.md#struct-level-inherent-operations` — hook_paths() endpoint map, spawn/preflight stub policy, JC-2 PostToolUse omission rationale

## Story Anchor (Recommended)

S-TBD — Implement ClaudeCodeModule inherent methods with hook_paths map (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-022-claude-code-module-inherent-methods.md` — VP-022 ClaudeCodeModule inherent methods integration test

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-003 per ARCH-INDEX is authoritative source`
  - After: `DI-006 ... ; DI-007 ...`
  - DI-006 mapping: hook_paths() is the zero-side-effect method: synchronous, no I/O, returns a static map. It is the purest expression of DI-006 on this struct. DI-007 mapping: hook_paths() returns strings; spawn() and preflight() are stubs in Phase 1; no write path to harness-owned files exists in this BC's scope.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).

## §Trace v1.0.4

**GAP-PHASE2-R06-3 closure — Architecture Module pin updated per ARCH-INDEX v1.0.11 cascade (trait/impl split clarification)** (2026-05-19T12:13:00Z):
- GAP-PHASE2-R06-3: ARCH-INDEX v1.0.10 → v1.0.11 corrected SS-03 split (EngineModule trait in monocle-core, ClaudeCodeModule implementation in monocle-runtime). BC Architecture Module cell still read pre-correction text `monocle-core (EngineModule trait, ClaudeCodeModule adapter)`.
  - SE-17f BEFORE: `monocle-core (EngineModule trait, ClaudeCodeModule adapter) per ARCH-INDEX Subsystem Registry SS-03`
  - SE-17f AFTER: `monocle-core (EngineModule trait, EnrichedSession, HookEvent types); monocle-runtime (ClaudeCodeModule implementation — monocle-runtime/src/engine/claude_code.rs) per ARCH-INDEX Subsystem Registry SS-03`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:13:00Z > prior 2026-05-18T05:21:00Z (v1.0.3). ARITHMETICALLY TRUE: PASS.

## §Trace v1.0.3

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.1.15 → v1.1.20** (2026-05-18T05:21:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.1.15 → v1.1.20); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-engine-module.md v1.1.15 §Struct-level inherent operations`
  - SE-17f AFTER: `SS-engine-module.md v1.1.20 §Struct-level inherent operations`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:21:00Z > prior 2026-05-17T18:00:00Z (v1.0.2). ARITHMETICALLY TRUE: 2026-05-18T05:21:00Z > 2026-05-17T18:00:00Z PASS.
