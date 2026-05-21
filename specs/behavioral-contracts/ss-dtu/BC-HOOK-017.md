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

# BC-HOOK-017: PID Liveness Check Uses process.kill(pid, 0) (POSIX-Only)

## Description

Lock file liveness is determined by sending signal 0 to the lock file's PID via
`process.kill(lk.pid, 0)`. On POSIX (Linux, macOS), signal 0 succeeds if the PID
exists and throws if the PID is dead — it does NOT actually send a signal. This is
a POSIX-specific mechanism: on Windows, signal 0 behavior differs. Since monocle
targets macOS + Linux (Phase 1 CI matrix per S-001), this is acceptable.

## Preconditions

1. A lock file exists with a `pid` field.
2. The lock file JSON is parseable.
3. Platform is POSIX (Linux or macOS).

## Postconditions

1. `process.kill(lk.pid, 0)` is called.
2. If the PID exists: no exception thrown; the lock file is considered alive.
3. If the PID is dead: exception thrown; caught by inner `try{}catch{}`; lock file is skipped.
4. No actual signal is delivered to the target process.

## Invariants

1. Signal 0 is a POSIX convention for "does this PID exist?" — not a signal delivery.
2. macOS and Linux both support `process.kill(pid, 0)` with the expected semantics.
3. Windows behavior is undefined for this check. monocle Phase 1 does not target Windows.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | PID is alive but belongs to a different process (zombie, PID reuse) | `process.kill(pid, 0)` succeeds (PID exists); lock file treated as alive; potentially incorrect — but lock file is owner-writable-only and the scenario is rare |
| EC-002 | PID field in lock file is 0 or negative | `process.kill(0, 0)` sends signal to process group; `process.kill(-1, 0)` is undefined; treated as failure by catch |
| EC-003 | Platform is Windows | `process.kill` may behave differently; Phase 1 does not support Windows |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Lock file with alive PID (current process's PID) | Lock file considered alive; port + token extracted | happy-path |
| Lock file with dead PID | Inner catch fires; lock file skipped | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone skips lock files with dead PIDs | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — PID liveness checking is part of the lock file coordination mechanism that enables hooks to distinguish alive from stale daemon instances |
| L2 Domain Invariants | DI-002 (lock file precondition — PID liveness is the check that determines whether a lock file represents a live daemon; dead-PID locks are stale and must not be used) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-017 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:19 (`process.kill(lk.pid,0)`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-017 (gene-source: deep-hooks-r1 §5 BC-HOOK-017) |
| Test name | test_BC_HOOK_017_pid_liveness_signal_zero |

## Related BCs

- [BC-HOOK-013] — depends on: BC-HOOK-013 covers the lock file scan loop; this BC covers the liveness check within that loop

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:19 (`process.kill(lk.pid,0)` inside inner try/catch).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
