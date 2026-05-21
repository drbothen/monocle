---
document_type: behavioral-contract
level: L3
version: "1.0.0"
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

# BC-HOOK-031: Hooks-Settings.json Is Unversioned — No Schema Version Field

## Description

The hooks-settings.json file has no `version`, `apiVersion`, or `schemaVersion` field.
The schema is implicitly defined by what Claude Code's `--settings` parser expects.
Forward-compatibility is handled by content evolution (add new hook types as Claude Code
adds them), not by schema versioning. The file is a derived artifact; its content is
always regenerated from the current hook command constants.

## Preconditions

1. `WriteHooksSettingsFile` is called.

## Postconditions

1. The output JSON has exactly one top-level key: `"hooks"`.
2. No `"version"`, `"apiVersion"`, `"schemaVersion"`, or similar versioning fields are present.
3. Claude Code parses the file expecting this exact structure.

## Invariants

1. The hooks-settings.json schema is implicitly versioned by Claude Code's `--settings` consumer.
2. If Claude Code adds a new required field, `WriteHooksSettingsFile` must be updated to include it.
3. Old hooks-settings.json files on disk (from prior monocle versions) will be overwritten on the next session launch — stale files are never a problem because the content is deterministic and regenerated.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Old hooks-settings.json from prior monocle version has different schema | Overwritten at next session launch; new version's schema always wins |
| EC-002 | Future Claude Code version requires a `version` field | monocle must update `WriteHooksSettingsFile` to add it; no self-healing |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Parse hooks-settings.json top-level keys | Exactly `["hooks"]` | lint |
| Check for version field | Not present | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | Hooks-settings.json has exactly one top-level key ("hooks"); no version field | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the unversioned hooks-settings.json schema is a lifecycle simplification; schema evolution is handled by code updates, not embedded versioning |
| L2 Domain Invariants | None directly (schema versioning is a forward-compatibility concern, not a domain invariant) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-031 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:49-52 (only `"hooks"` key in top-level map; no version field) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-031 (gene-source: deep-hooks-r1 §10 BC-HOOK-031) |
| Test name | test_BC_HOOK_031_hooks_settings_json_unversioned |

## Related BCs

- [BC-HOOK-007] — depends on: BC-HOOK-007 covers the key set; this BC covers the absence of a version key

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:49-52 (only `"hooks"` key confirmed; hooks_test.go:34-42 asserts `parsed["hooks"]` only — no version assertion exists or is needed).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
