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

# BC-HOOK-038: Two-Server-Same-Port Race Is Structurally Impossible (Lock-After-Bind Ordering)

## Description

The "highest port wins" tie-break in lock file discovery never has to handle two
live servers at the same port because TCP port binding is atomic at the kernel level:
only one process can bind a given port; the other gets EADDRINUSE. The daemon writes
the lock file AFTER binding, so `<port>.lock` always corresponds to a bound server
(or a stale entry from a dead one). Stale entries are caught by PID liveness check.

## Preconditions

1. Two monocle daemon instances attempt to bind the same port simultaneously.

## Postconditions

1. Kernel TCP: one succeeds with `bind(<port>)`, the other gets EADDRINUSE.
2. The losing daemon returns an error and exits WITHOUT writing a lock file.
3. Only the winning daemon writes `<port>.lock`.
4. At any point in time, `<port>.lock` corresponds to either: (a) the currently bound daemon at that port, or (b) a stale entry from a dead daemon (which PID liveness skips).

## Invariants

1. The lock-after-bind ordering is `net.Listen` → `lock.Write` — never reversed.
2. Two live daemons at the same port is structurally impossible, not just unlikely.
3. The "highest port wins" tie-break is therefore selecting between daemons on DIFFERENT ports — there is no tie.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Theoretical: two locks at same port (impossible in practice) | Stale lock is always caught by PID liveness check; winner is the one with a live PID |
| EC-002 | Lock file with port from filename X; daemon actually bound to port Y (programmer error) | PID liveness passes; srvPort = X; HTTP POST to port X fails; hook timeout or error; event dropped |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Concurrent daemon starts | Only one succeeds bind; only one lock file written | integration |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone: lock file always corresponds to the bound port (no same-port ties possible) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the lock-after-bind ordering is a daemon lifecycle invariant that ensures lock files are always valid references to bound TCP ports |
| L2 Domain Invariants | DI-002 (lock file precondition — lock-after-bind ordering ensures the lock file represents a fully initialized daemon; this BC confirms the structural impossibility of a lock file that doesn't correspond to a valid bound port) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-038 |
| Gene Source | any-context-lazyclaude/internal/server/server.go:123-143 (`net.Listen` before `lock.Write`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-038 (gene-source: deep-hooks-r2 §2 BC-HOOK-038) |
| Test name | test_BC_HOOK_038_no_same_port_race |

## Related BCs

- [BC-HOOK-013] — depends on: BC-HOOK-013 covers the highest-port-wins algorithm; this BC confirms that ties are impossible

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: server.go:123-143 (`net.Listen` first, `lock.Write` second — confirmed atomic ordering).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
