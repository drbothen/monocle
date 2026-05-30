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

# BC-HOOK-041: Monocle Test Must Assert Canonical Filename hooks-settings.json

## Description

The gene source's hooks_test.go does NOT assert that the output file is named
`hooks-settings.json` — it only asserts the path is non-empty. A refactor that
changes the filename would not break the existing test. The monocle port MUST add
an explicit assertion `assert!(path.ends_with("hooks-settings.json"))` to its test
suite to prevent a naming regression.

## Preconditions

1. `WriteHooksSettingsFile` is called in a test context.
2. The returned path is checked.

## Postconditions

1. The returned path ends with `"hooks-settings.json"`.
2. The test asserts `path.ends_with("hooks-settings.json")` explicitly.
3. This assertion catches any future refactoring that changes the filename.

## Invariants

1. The filename `hooks-settings.json` is byte-compatible with the Claude Code `--settings` convention — the file name doesn't matter to Claude Code, but it matters for human readability and for the monocle codebase's internal consistency.
2. Adding this test assertion is a monocle improvement over the gene source's test gap (P2 finding in hooks-r2).
3. The assertion must use path suffix check, not full path check, to remain portable across different `runtimeDir` values in tests.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Refactoring changes filename to `monocle-hooks.json` | Test fails with assertion error `"monocle-hooks.json" does not end with "hooks-settings.json"` |
| EC-002 | runtimeDir is `/tmp/test-abc`; expected full path is `/tmp/test-abc/hooks-settings.json` | `path.ends_with("hooks-settings.json")` passes |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| WriteHooksSettingsFile returns `/tmp/test/hooks-settings.json` | Assertion passes | happy-path |
| WriteHooksSettingsFile changed to return `/tmp/test/monocle-hooks.json` | Assertion fails | regression |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | Monocle WriteHooksSettingsFile test includes `ends_with("hooks-settings.json")` assertion | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the canonical filename assertion prevents a class of regression where the hooks-settings.json path changes and Claude Code's --settings injection silently breaks |
| L2 Domain Invariants | None directly (test assertion is a correctness property, not a domain invariant) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-041 (P2 finding: hooks_test.go doesn't assert canonical filename) |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks_test.go:17-19 (`assert.NotEmpty(t, path)` — NO filename assertion) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-041 (gene-source: deep-hooks-r2 §4 BC-HOOK-041; P2 finding) |
| Test name | test_BC_HOOK_041_canonical_filename_assertion |

## Related BCs

- [BC-HOOK-009] — depends on: BC-HOOK-009 specifies the canonical filename; this BC specifies the test that must verify it

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks_test.go:17-19 (`assert.NotEmpty(t, path)` — no filename assertion; P2 gap identified in r2 §4).
- Monocle improvement: add `assert!(path.ends_with("hooks-settings.json"))` to test.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
