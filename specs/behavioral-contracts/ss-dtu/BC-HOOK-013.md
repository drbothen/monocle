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

# BC-HOOK-013: Hook URL Host Is 127.0.0.1; Port Resolved at Each Invocation via Lock-File Scan

## Description

Each hook invocation independently re-scans the lock file directory (`~/.monocle/run/` or
`MONOCLE_RUNTIME_DIR`) for alive lock files. The host is hardcoded `127.0.0.1`. The port
is NOT stored in hooks-settings.json — it is read fresh from the alive lock file's filename
on every hook invocation. This stateless design means daemon restarts automatically redirect
hook traffic to the new port without re-issuing the settings file.

## Preconditions

1. A hooks-settings.json file exists (written once at session launch).
2. A hook is invoked by Claude Code (stdin is piped).

## Postconditions

1. The hook JS re-scans the lock file directory on every invocation.
2. `hostname` is always `'127.0.0.1'`.
3. `port` is the integer value extracted from the filename of the highest-port alive lock file.
4. `srvToken` is the `authToken` field from the same lock file as `srvPort`.
5. If no alive lock file exists: `srvPort` remains `null`; fail-open/fail-closed per hook type.

## Invariants

1. The hooks-settings.json file NEVER contains a static port value. Port is always dynamic.
2. Each hook invocation is a fresh `node -e "..."` process; `srvPort` is local state — no cross-invocation caching.
3. Port and token are always sourced from the same lock file (atomicity: if the lock file is valid, port and token are consistent).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Daemon restarts between two consecutive hook invocations | First invocation uses old port; second invocation re-scans and finds new port |
| EC-002 | Lock file directory is inaccessible (permissions) | `readdirSync` throws; caught by outer try/catch; srvPort = null; fail-open/closed |
| EC-003 | Lock file is readable but port from filename is NaN (non-numeric filename) | BC-HOOK-034 handling applies: NaN comparison skips that lock file |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Lock file `7860.lock` alive; hook invocation | POST to `http://127.0.0.1:7860/...` | happy-path |
| Daemon restarts on port 7861; next hook invocation | POST to `http://127.0.0.1:7861/...` (new port discovered) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone re-scans lock files per invocation; selects highest-port alive server | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — dynamic port discovery is the mechanism that maintains hook-to-daemon connectivity across daemon restarts, a core lifecycle management capability |
| L2 Domain Invariants | DI-002 (lock file precondition — this BC is the consumer-side implementation of DI-002: hooks can only find the daemon port by reading a valid lock file) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-013 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:26-27 (`let srvPort=null,srvToken=null;`; `if(best){srvPort=best.port;srvToken=best.lock.authToken;}`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-013 (gene-source: deep-hooks-r1 §5 BC-HOOK-013) |
| Test name | test_BC_HOOK_013_port_resolved_per_invocation_from_lock_file |

## Related BCs

- [BC-HOOK-005] — composes with: BC-HOOK-005 covers the 127.0.0.1 host and dynamic port at request-construction level
- [BC-HOOK-015] — composes with: BC-HOOK-015 covers token resolution from the same lock file
- [BC-HOOK-025] — composes with: BC-HOOK-025 covers the restart-resilience sequence that this per-invocation scan enables
- [BC-HOOK-026] — composes with: BC-HOOK-026 covers the statelessness (no caching between invocations)

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:26-27 (`let srvPort=null,srvToken=null;`; `if(best)...`); hooks.go:13-20 (lock-dir scan loop).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
