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

# BC-HOOK-003: Notification Hook Filters on notification_type === 'permission_prompt'

## Description

The Notification hook script contains a client-side filter: it only fires an HTTP POST
to the monocle daemon when the Claude Code input carries `notification_type ===
'permission_prompt'`. All other notification types (idle, informational, etc.) are
silently dropped before any HTTP call. This filtering occurs in the inline JS hook
script, not on the server. The daemon has no knowledge of this pre-filter.

## Preconditions

1. An alive monocle daemon is running (lock file found with live PID).
2. Claude Code triggers a Notification hook and pipes JSON on stdin.

## Postconditions

1. If `i.notification_type === 'permission_prompt'`: HTTP POST is sent to `/hooks/notification`.
2. If `i.notification_type !== 'permission_prompt'`: `return` executes; no HTTP POST is sent; no stdout output.
3. The filter is evaluated BEFORE the HTTP request is constructed.
4. Claude Code receives no stdout in either case (Notification hook never echoes stdin).

## Invariants

1. The `notification_type` field is read from the Claude Code-supplied stdin JSON — the hook does NOT generate or modify this field.
2. The filter is client-side: the monocle daemon endpoint `/hooks/notification` only ever receives `permission_prompt` notifications; other types are invisible to the server.
3. Case-sensitive match: `'permission_prompt'` is lowercase with underscore.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `notification_type` is absent from stdin JSON | `i.notification_type` is `undefined`; `undefined !== 'permission_prompt'` is true; notification dropped |
| EC-002 | `notification_type` is `'Permission_Prompt'` (wrong case) | String comparison fails; notification dropped |
| EC-003 | `notification_type` is `'permission_prompt'` with a live server | HTTP POST sent to `/hooks/notification` |
| EC-004 | `notification_type` is `'idle'` | Dropped before HTTP call |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Server alive; stdin `{"notification_type":"permission_prompt","message":"Allow Bash?","tool_name":"Bash","tool_input":{"command":"ls"}}` | POST to `/hooks/notification` sent | happy-path |
| Server alive; stdin `{"notification_type":"idle"}` | No HTTP call; no stdout | edge-case |
| Server alive; stdin `{}` (no notification_type) | No HTTP call; no stdout | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone Notification handler only fires on permission_prompt notification_type | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — this BC defines the notification_type filter at the hook protocol level, which determines what reaches the daemon's ingestion endpoint |
| L2 Domain Invariants | DI-001 (tee invariant — the client-side filter is a pre-ingestion gate; only events that pass the filter reach the daemon and are subject to DI-001's ring-write obligation) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-020 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:35 (`if(i.notification_type!=='permission_prompt')return;`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-003 (gene-source: any-context Pass 3) |
| Test name | test_BC_HOOK_003_notification_filter_permission_prompt_only |

## Related BCs

- [BC-HOOK-020] — supersedes: BC-HOOK-020 is the deep-ingest confirmation of this filter with exact source evidence

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:35 (`if(i.notification_type!=='permission_prompt')return;`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
