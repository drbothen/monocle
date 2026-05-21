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

# BC-HOOK-024: Monocle Improvement — Lock File App Filter Added to Hook JS

## Description

The gene source (any-context-lazyclaude) hook JS does NOT filter by `lock.app` field —
it accepts any alive lock file at the highest port regardless of which application
wrote it. This is a P2 finding from hooks-r1: if another IDE tool (VS Code) writes a
lock file at a higher port, hooks would send POSTs to that tool's server.

monocle IMPROVES on this: the inline JS MUST add `if (lk.app && lk.app !== 'monocle') continue;`
to the lock-file enumeration loop. This closes the cross-IDE collision risk identified
in hooks-r1.

## Preconditions

1. Multiple lock files exist in the lock file directory, potentially from different applications.
2. A lock file from a non-monocle application (e.g., VS Code IDE integration) exists at a higher port.

## Postconditions

1. The hook JS filters lock files by `lk.app` field: only `lk.app === 'monocle'` or `lk.app` is absent/undefined passes the filter.
2. A non-monocle lock file at a higher port is SKIPPED.
3. The highest-port MONOCLE lock file is selected.
4. If no monocle lock file exists: `srvPort` remains null; per-hook fallback (fail-open/closed).

## Invariants

1. The monocle lock file MUST write `app: 'monocle'` (or equivalent Rust struct field) to enable this filter.
2. The filter logic: `if (lk.app && lk.app !== 'monocle') continue;` — this correctly allows absent app field (legacy compat) AND rejects non-monocle apps.
3. This is a monocle improvement over the gene source; it prevents cross-IDE collision.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Only monocle lock file exists | Filter passes; monocle port selected |
| EC-002 | VS Code lock file at port 9000; monocle lock file at port 7860 | VS Code lock filtered; monocle port 7860 selected |
| EC-003 | VS Code lock file at port 9000; no monocle lock file | All locks filtered; srvPort = null; fail-open/closed |
| EC-004 | Lock file with absent `app` field (legacy) | `!lk.app` is true; filter passes — backwards compatible |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Two lock files: monocle@7860, vscode@9000 | Hook POST to port 7860 (monocle) | happy-path |
| Only vscode lock file | srvPort = null; fail-open/closed | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone hook JS filters non-monocle lock files by app field | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the app-field filter is a correctness improvement that ensures hooks connect to the monocle daemon specifically, not arbitrary IDE integrations sharing the lock file directory |
| L2 Domain Invariants | DI-002 (lock file precondition — the app filter ensures that only a valid MONOCLE lock file is used to derive the connection target; without the filter, a non-monocle lock file would satisfy DI-002 incorrectly) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-024 (P2 finding: no app filter) |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:13-20 (NO `lock.app` check — identified as P2 finding; monocle improvement adds the filter) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-024 (gene-source: deep-hooks-r1 §7 BC-HOOK-024) |
| Test name | test_BC_HOOK_024_lock_app_filter_monocle_only |

## Related BCs

- [BC-HOOK-013] — depends on: BC-HOOK-013 covers the overall lock-file scan algorithm; this BC covers the app-field filter within that scan

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source P2 finding: hooks.go:13-20 has NO `lock.app` filter; monocle improvement adds `if (lk.app && lk.app !== 'monocle') continue;`.
- This is a deliberate monocle improvement over the gene source, not a replication.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
