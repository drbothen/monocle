---
document_type: behavioral-contract
level: L3
version: "1.0.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-20T21:00:00Z
phase: 1a
inputs:
  - {path: .factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r2.md, version: "r2"}
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

# BC-HOOK-040: Go Map Iteration Randomness Causes Byte Non-Determinism; Rust Struct Serialization Is Stable

## Description

In the gene source (Go), `buildHooksMap()` returns `map[string]any`. Go's `encoding/json`
serializes maps with keys in alphabetical order (documented stdlib behavior), producing a
deterministic key order. The Rust port using `serde_json` with a struct (not `HashMap`)
produces stable output in struct field declaration order (using `#[serde(rename = ...)]`).
Using a `HashMap` in Rust would produce non-deterministic output.

## Preconditions

1. `WriteHooksSettingsFile` is called; the hook map is being serialized.

## Postconditions (gene-source Go behavior):

1. Go's `map[string]any` is serialized with alphabetically sorted keys.
2. Output hook key order: `Notification` → `PreToolUse` → `SessionStart` → `Stop` → `UserPromptSubmit`.
3. Inner object keys (`hooks`, `matcher` per `HookEntry`) also sorted alphabetically.

## Postconditions (Rust port with struct):

1. Rust struct with named fields and `#[serde(rename = "PreToolUse")]` etc. serializes in field declaration order.
2. Output is byte-stable across runs.
3. Claude Code's JSON parser is order-insensitive; key order does not affect functionality.

## Invariants

1. The Rust port MUST use a struct (not `HashMap`) for `HookMap` to guarantee byte-stable output.
2. Byte order does not affect Claude Code's `--settings` parsing, but CI tests that compare against a golden file need deterministic output.
3. The exact key order in the Rust output is implementation-defined (struct declaration order); it may differ from Go's alphabetical order — this is acceptable as both are valid JSON.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Rust port uses `HashMap<String, Vec<MatcherEntry>>` | Key order is random per run; CI golden-file comparison may flap |
| EC-002 | Rust port uses struct with fixed declaration order | Key order is stable; no CI flap |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Serialize HookMap twice | Same key order both times (struct determinism) | lint |
| Parse with Claude Code `--settings` | Both orderings are valid JSON; Claude Code accepts either | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone produces byte-stable hooks-settings.json (struct-based serialization, not HashMap) | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — byte-stable serialization is a correctness property for test infrastructure; CI golden-file comparisons require deterministic output |
| L2 Domain Invariants | None directly (key ordering is a serialization detail, not a domain invariant) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-040 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:92-99 (`map[string]any` — Go alphabetical JSON ordering) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-040 (gene-source: deep-hooks-r2 §3 BC-HOOK-040) |
| Test name | test_BC_HOOK_040_struct_based_stable_serialization |

## Related BCs

- [BC-HOOK-008] — depends on: BC-HOOK-008 covers encoding options; this BC covers key ordering stability

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:92-99 (`map[string]any` — Go stdlib JSON sorts keys alphabetically).
- Rust port note: use `struct` not `HashMap` for deterministic serialization.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
