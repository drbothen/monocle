---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-20T21:00:00Z
phase: 1a
inputs:
  - {path: .factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-3-behavioral-contracts.md, version: "pass-3"}
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

# BC-HOOK-005: Hook HTTP Request Target is 127.0.0.1 with Port from Lock File

## Description

All hook HTTP requests target `hostname: '127.0.0.1'` with port dynamically resolved
from the alive lock file at invocation time. The hostname is hardcoded in the hook
script source; the port is NOT in the hooks-settings.json file — it is discovered
fresh at each hook invocation by scanning the lock file directory.

## Preconditions

1. An alive lock file exists with a valid `port` and `authToken`.
2. The PID in the lock file responds to `process.kill(pid, 0)`.

## Postconditions

1. `hostname` is `'127.0.0.1'` — no DNS resolution, no IPv6, no localhost aliasing.
2. `port` is the integer parsed from the alive lock file's filename (e.g., `"7860.lock"` → port `7860`).
3. The HTTP request is sent to `http://127.0.0.1:<port>/<path>`.
4. The port is resolved at invocation time — the hooks-settings.json file contains no port value.

## Invariants

1. The hook MUST NOT hardcode a port. Port discovery is always dynamic.
2. `127.0.0.1` is the only valid target hostname — the daemon only binds on loopback.
3. If multiple alive lock files exist, the highest-port alive server is selected (BC-HOOK-013).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Daemon restarts on a new port before next hook invocation | Hook discovers new port from new lock file; hooks-settings.json is unchanged |
| EC-002 | Two alive lock files (e.g., port 7860 and port 7861) | Higher port (7861) is selected per BC-HOOK-013 highest-port-wins |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Lock file at `7860.lock` with live PID; PreToolUse hook | HTTP POST to `http://127.0.0.1:7860/hooks/pre-tool-use` | happy-path |
| Two lock files: `7860.lock` (live PID), `7861.lock` (live PID) | HTTP POST to `http://127.0.0.1:7861/...` (highest wins) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone targets 127.0.0.1:<dynamic-port> for all hook POSTs | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — dynamic port discovery from the lock file is the core mechanism that connects hook scripts to the daemon across restarts |
| L2 Domain Invariants | DI-002 (lock file precondition — the lock file must be present and readable before hooks can discover the port; this BC operationalizes the consumer side of DI-002) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-013 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (`hostname:'127.0.0.1', port:srvPort`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-005 (gene-source: any-context Pass 3) |
| Test name | test_BC_HOOK_005_hook_target_loopback_dynamic_port |

## Related BCs

- [BC-HOOK-013] — depends on: port resolution algorithm is specified in BC-HOOK-013
- [BC-HOOK-025] — composes with: restart-resilience sequence uses this dynamic port discovery

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (`hostname:'127.0.0.1', port:srvPort`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
