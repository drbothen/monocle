---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T12:00:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "03a845a"
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
| L2 Domain Invariants | N/A — no domain-spec/invariants.md exists; CAP-003 per ARCH-INDEX is authoritative source |
| Architecture Module | monocle-core (EngineModule trait, ClaudeCodeModule adapter) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module.md v1.1.15 §Struct-level inherent operations |
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

- `verification-properties/vp-022-claude-module-hook-paths.md` — VP-022 hook_paths five-entry integration test
