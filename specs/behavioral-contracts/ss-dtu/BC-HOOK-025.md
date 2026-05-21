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

# BC-HOOK-025: After Daemon Restart, First Hook Invocation Re-Discovers New Port; Events During Restart Window Are Dropped

## Description

When the monocle daemon restarts on a new port, hook events behave as follows:
- Window 1 (before restart): hooks find old server, deliver normally.
- Window 2 (restart window — between old server stop and new lock file written): hooks find no alive server; events dropped per BC-HOOK-018 (fail-open/closed).
- Window 3 (new server up, new lock file written): first hook invocation after restart re-scans and discovers the new port; events resume delivery.

No producer-side re-issuance of hooks-settings.json is needed — the stateless per-invocation scan handles all three windows automatically.

## Preconditions

1. Daemon restarts (old process dies; new process starts on a different port).
2. New lock file is written at `<runtimeDir>/<new-port>.lock`.

## Postconditions

1. Hook invocations during Window 2 (after old PID is dead, before new lock written): `srvPort` = null; BC-HOOK-018 fallback applies.
2. First hook invocation after new lock file appears (Window 3): `srvPort` = new port; delivery resumes.
3. No hooks-settings.json update is required between restarts.
4. Old lock file is removed by daemon's `CleanAllExcept(newPort)` call at startup; new lock file is the only alive monocle lock.

## Invariants

1. The restart-resilience is a consequence of per-invocation stateless discovery (BC-HOOK-026) combined with PID liveness checking (BC-HOOK-017).
2. The only data structure that changes is the filesystem (`~/.monocle/run/`). No in-process state needs updating.
3. Events dropped in Window 2 are permanently lost. There is no buffering or replay.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Daemon restarts so fast that no hook invocation occurs in Window 2 | Hooks never see a "no server" state; seamless delivery |
| EC-002 | Multiple rapid restarts (daemon loops) | Each new lock file at a new port; hooks always pick up the most recent alive lock |
| EC-003 | Daemon crashes without writing new lock file | Window 2 extends indefinitely; all hooks fail-open/closed until daemon is manually restarted |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Daemon running on port 7860; restart begins | PreToolUse during restart window echoes stdin (fail-open) | edge-case |
| Daemon restarts on port 7861 | First PreToolUse after restart POSTs to 7861 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone rediscovers new port after daemon restart without settings file update | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — restart resilience is a core daemon lifecycle property; the hook protocol must remain functional across daemon restarts |
| L2 Domain Invariants | DI-002 (lock file precondition — restart resilience works through the lock file mechanism: new lock file appears → hooks discover new port; this is the dynamic lifecycle of DI-002) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-025 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:13-27 (per-invocation stateless discovery) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-025 (gene-source: deep-hooks-r1 §7 BC-HOOK-025) |
| Test name | test_BC_HOOK_025_restart_resilience_new_port_discovery |

## Related BCs

- [BC-HOOK-013] — depends on: BC-HOOK-013 covers port resolution; this BC covers the multi-invocation temporal behavior
- [BC-HOOK-026] — depends on: BC-HOOK-026 covers the stateless discovery that makes restart resilience work

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:13-27 (entire discovery block runs per invocation with no cached state).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
