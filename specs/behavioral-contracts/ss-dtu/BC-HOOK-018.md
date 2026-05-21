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

# BC-HOOK-018: Per-Hook Fallback Semantics Matrix When No Alive Server Found

## Description

This BC consolidates the authoritative fallback matrix for all five hooks when no
alive server is found. The matrix shows which hooks are fail-open (stdin echoed)
vs fail-closed (silent drop). This is the canonical reference contract;
BC-HOOK-001 and BC-HOOK-002 document the individual fail-open and fail-closed behaviors.

## Preconditions

1. No alive lock file found (lock file directory missing, all lock files have dead PIDs, or directory is empty).

## Postconditions

The following matrix governs fallback behavior:

| Hook Type | Fallback Behavior | stdout Output | HTTP Request |
|-----------|------------------|---------------|-------------|
| PreToolUse | Fail-open | stdin echoed verbatim | None |
| Notification | Fail-closed | None | None |
| Stop | Fail-closed | None | None |
| SessionStart | Fail-closed | None | None |
| UserPromptSubmit | Fail-closed | None | None |

## Invariants

1. PreToolUse is the ONLY hook that echoes stdin when no server is found.
2. The four fail-closed hooks produce no observable output when no server is found.
3. This asymmetry is load-bearing for Claude Code UX: if monocle is down, Claude Code continues to function because PreToolUse "allows" tool execution via the stdin echo.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All hooks fire in sequence with no server | PreToolUse echoes stdin; all others produce no output |
| EC-002 | Server comes up between Notification (dropped) and Stop (dropped) | Stop and subsequent hooks will still find no server if discovery runs before the lock file is written |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| No server; PreToolUse stdin = `{"tool_name":"Bash","tool_input":{}}` | stdout = `{"tool_name":"Bash","tool_input":{}}` | happy-path |
| No server; Notification stdin = `{"notification_type":"permission_prompt"}` | No stdout | happy-path |
| No server; Stop stdin = `{"stop_reason":"normal"}` | No stdout | happy-path |
| No server; SessionStart stdin = `{"session_id":"abc"}` | No stdout | happy-path |
| No server; UserPromptSubmit stdin = `{"session_id":"abc"}` | No stdout | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone fallback matrix matches the 5-row spec above exactly | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the per-hook fallback matrix defines the complete behavior of the hook protocol when the daemon lifecycle is interrupted (no running daemon) |
| L2 Domain Invariants | DI-001 (tee invariant — fail-closed on 4 hooks means those events are not ingested when daemon is absent; this is the lifecycle boundary of DI-001) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-018 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (PreToolUse fail-open `if(!srvPort){console.log(d);return;}`); hooks.go:35,38,41,44 (fail-closed `if(!srvPort)return;`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-018 (gene-source: deep-hooks-r1 §5 BC-HOOK-018) |
| Test name | test_BC_HOOK_018_per_hook_fallback_semantics_matrix |

## Related BCs

- [BC-HOOK-001] — depends on: BC-HOOK-001 covers PreToolUse fail-open in detail
- [BC-HOOK-002] — depends on: BC-HOOK-002 covers the other 4 hooks fail-closed in detail

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (PreToolUse fallback differs from hooks.go:35,38,41,44 — asymmetry documented here).
- This BC is the consolidating reference; BC-HOOK-001/002 document the individual behaviors.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
