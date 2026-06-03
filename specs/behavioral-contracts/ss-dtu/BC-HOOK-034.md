---
document_type: behavioral-contract
level: L3
version: "1.0.2"
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

# BC-HOOK-034: parseInt Filename Parsing Handles Non-Numeric Lock Files via NaN Comparison

## Description

Lock file filenames are parsed by `parseInt(f, 10)` to extract the port number.
For non-numerically-named lock files (e.g., `vscode-abc.lock`), `parseInt` returns
`NaN`. Due to ECMAScript NaN comparison semantics (`NaN > anything` === `false`),
a NaN-port lock file can only become `best` if it is the FIRST alive lock processed.
Subsequent numeric-port locks will NOT displace a NaN-port best (since `p > NaN`
is always false). The monocle improvement is to add `if (isNaN(p)) continue;` to
defensively skip non-numeric lock files.

## Preconditions

1. The lock file directory contains at least one lock file with a non-numeric filename prefix (e.g., `vscode-abc.lock` or `app.lock`).

## Postconditions (gene-source behavior):

1. `parseInt("vscode-abc.lock", 10)` → `NaN`.
2. If `best` is null and PID is alive: `best = {lock: lk, port: NaN}`.
3. `srvPort = NaN` → `if(!srvPort)` is true (NaN is falsy) → fail-open/closed triggered.
4. Effective result: non-numeric lock file leads to fail-open/closed behavior (same as no server).

## Postconditions (monocle improvement):

1. `if (isNaN(p)) continue;` is added to the enumeration loop.
2. Non-numeric lock files are skipped entirely — the loop continues to numeric-named locks.
3. The best candidate is always a numerically-ported lock file.

## Invariants

1. Lock files in `~/.monocle/run/` are expected to be named `<port>.lock` (numeric). Non-numeric filenames are treated as non-monocle files.
2. The monocle improvement (`isNaN` skip) is correct and should be implemented.
3. NaN comparison: `NaN > 5` → false; `5 > NaN` → false. Only `best === null` causes NaN-port selection in the gene source.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Non-numeric lock file only (`app.lock`) | Gene source: srvPort = NaN → fail-open/closed; Monocle: skipped → srvPort = null → fail-open/closed |
| EC-002 | Non-numeric lock file at port "NaN" followed by numeric `7860.lock` | Monocle (with isNaN skip): `7860.lock` selected; numeric lock wins |
| EC-003 | Only numeric lock files | Normal behavior; isNaN skip never fires |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Lock dir contains `app.lock` (alive PID) and `7860.lock` (alive PID) | monocle: PORT=7860 selected; `app.lock` skipped | edge-case |
| Lock dir contains only `app.lock` | srvPort = null; fail-open/closed | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone hook JS skips non-numeric lock files via isNaN check | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the NaN-handling during lock file enumeration is a defensive correctness property of the lock file discovery mechanism |
| L2 Domain Invariants | DI-002 (lock file precondition — the isNaN defensive skip ensures that only valid port-numbered lock files are used to derive the daemon connection; malformed filenames do not produce invalid port values) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-034 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:18-19 (`parseInt(f, 10)` + NaN comparison behavior) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-034 (gene-source: deep-hooks-r2 §2 BC-HOOK-034; P3 minor finding) |
| Test name | test_BC_HOOK_034_nan_port_lock_file_skip |

## Related BCs

- [BC-HOOK-013] — depends on: BC-HOOK-013 covers the lock file scan loop; this BC covers the NaN edge case within that scan

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:18-19 (`parseInt(f, 10)` and `p > best.port` comparison; NaN semantics).
- Monocle improvement: add `if (isNaN(p)) continue;` to defensively skip non-numeric lock files.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.2

**LOW-005: frontmatter typo `decorated_by` → `deprecated_by`** (2026-06-03):
- Field name corrected: `decorated_by: null` → `deprecated_by: null`.
- Pure frontmatter spelling fix; zero normative or behavioral content change.
- Version bump: 1.0.1 → 1.0.2 (minimal bump per convention; typo corrections in frontmatter fields are versioned to maintain registry atomicity).
- SE-16d PASS: 2026-06-03 >= 2026-05-30 (monotonicity satisfied).

## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
