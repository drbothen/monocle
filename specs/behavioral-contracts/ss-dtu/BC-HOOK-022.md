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

# BC-HOOK-022: Notification Timeout Is 2000ms; Other Four Hooks Are 300ms

## Description

The Notification hook uses a `timeout: 2000` (2 seconds) on its HTTP request while
the other four hooks (PreToolUse, Stop, SessionStart, UserPromptSubmit) use
`timeout: 300` (300 milliseconds). Notification's longer timeout is because it
drives the permission-prompt UI flow: the server may need time to capture ANSI output
and dispatch a permission popup before responding. The other four are observability
signals with no UI requirement.

## Preconditions

1. An HTTP POST request is being constructed for a hook invocation.

## Postconditions

1. Notification: `timeout: 2000` set on the `http.request` options.
2. PreToolUse: `timeout: 300` set on the `http.request` options.
3. Stop: `timeout: 300` set on the `http.request` options.
4. SessionStart: `timeout: 300` set on the `http.request` options.
5. UserPromptSubmit: `timeout: 300` set on the `http.request` options.
6. When the timeout fires: `req.destroy()` is called; socket is closed; the hook continues (fire-and-forget).

## Invariants

1. The 300ms timeout is load-bearing for Claude Code performance: a monocle daemon handler that exceeds 300ms will cause the PreToolUse hook to timeout and destroy the connection. The daemon handler MUST return in well under 300ms.
2. The 2000ms timeout for Notification reflects the UI latency budget for the permission-prompt flow.
3. Timeout values are hardcoded constants — not configurable via env var or hooks-settings.json.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Daemon handler for PreToolUse takes 400ms | `req.destroy()` fires at 300ms; hook swallows; event lost |
| EC-002 | Daemon handler for Notification takes 2100ms | `req.destroy()` fires at 2000ms; hook swallows; permission event lost |
| EC-003 | Daemon is under heavy load; handler takes 290ms | Under the 300ms threshold; event delivered |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Notification POST constructed | `timeout` option is 2000 | lint |
| PreToolUse POST constructed | `timeout` option is 300 | lint |
| Stop POST constructed | `timeout` option is 300 | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone uses 2000ms timeout for Notification and 300ms for all other hooks | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — hook timeout values define the latency budget for the daemon's hook ingestion pipeline; the 300ms/2000ms split is a lifecycle performance constraint |
| L2 Domain Invariants | None directly (timeout values are implementation constraints, not domain invariants) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-022 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:35 (`timeout:2000` Notification); hooks.go:31,38,41,44 (`timeout:300` others) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-022 (gene-source: deep-hooks-r1 §6 BC-HOOK-022) |
| Test name | test_BC_HOOK_022_notification_timeout_2000ms_others_300ms |

## Related BCs

- [BC-HOOK-004] — depends on: BC-HOOK-004 covers the fire-and-forget pattern; timeout is part of that pattern

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:35 (`timeout:2000`); hooks.go:31,38,41,44 (`timeout:300`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
