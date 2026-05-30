---
document_type: behavioral-contract
level: L3
version: "1.0.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-20T21:00:00Z
phase: 1a
inputs:
  - {path: .factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md, version: "r1"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
input-hash: "[live-state]"
traces_to: prd.md
origin: gene-transfusion
subsystem: SS-01
capability: CAP-001
dtu_service: claude-code-hook-protocol
gene_source: any-context-lazyclaude/internal/core/config/hooks.go
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

# BC-HOOK-012: Hook Configuration Is Identical Across All Session Types (PM, Worker, Plain)

## Description

The `buildHooksMap()` function takes no Role or SessionType parameter — it returns
the same hook configuration regardless of whether the session is a plain session,
a PM (project manager), or a Worker. All sessions get the same 5 hook commands
pointing to the same lock-discovered server. There is no role-based hook skipping.

## Preconditions

1. Any type of monocle session is launched (plain, PM, Worker, or future role).

## Postconditions

1. `buildHooksMap()` returns the same 5-entry map regardless of session role.
2. The hooks-settings.json content is byte-identical for PM, Worker, and plain sessions launched in the same runtimeDir.
3. No hook type is conditionally omitted based on session role.

## Invariants

1. The Rust port SHOULD use a `&'static [HookEntry]` constant for the hook configuration — it never varies.
2. Role-based hook filtering is NOT a Phase 1 feature. If introduced in Phase 3+, it requires both a new `buildHooksMap` variant AND BC updates.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | PM session launches | All 5 hooks injected identically to plain session |
| EC-002 | Worker session launches | All 5 hooks injected identically to plain session |
| EC-003 | Future session type with no explicit role config | Falls through to same default 5-hook config |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Plain session hooks-settings.json | 5 hooks; all present | happy-path |
| PM session hooks-settings.json | Byte-identical to plain session file | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone produces identical hook config for all session types | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — uniform hook configuration across session types ensures all sessions participate equally in the daemon's hook event ingestion lifecycle |
| L2 Domain Invariants | None directly (uniform configuration is an implementation choice, not a domain invariant) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-012 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:78-99 (`buildHooksMap` is parameterless) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-012 (gene-source: deep-hooks-r1 §4 BC-HOOK-012) |
| Test name | test_BC_HOOK_012_hook_config_identical_all_session_types |

## Related BCs

- [BC-HOOK-007] — depends on: BC-HOOK-007 defines the hook type set that is uniformly applied

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:78-99 (`buildHooksMap()` is parameterless — confirmed at file:line precision).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
