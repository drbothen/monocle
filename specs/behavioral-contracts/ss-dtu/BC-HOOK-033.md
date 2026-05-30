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

# BC-HOOK-033: Malformed Stdin JSON Silently Drops Hook for Non-PreToolUse Hooks

## Description

For Notification, Stop, SessionStart, and UserPromptSubmit: if Claude Code pipes
malformed JSON on stdin, `JSON.parse(d)` throws. The outer `try{}catch{}` catches
the exception. There is NO `console.log(d)` at end of block for these four hooks.
The hook process exits silently — no stdout, no HTTP call. This is fail-closed on
parse error for all observability hooks.

## Preconditions

1. Any of: Notification, Stop, SessionStart, or UserPromptSubmit hook is invoked.
2. Claude Code pipes malformed JSON on stdin.

## Postconditions

1. Stdin is read into `d`.
2. `JSON.parse(d)` throws.
3. Outer `catch{}` executes.
4. Function returns (implicit exit from catch).
5. No stdout output.
6. No HTTP POST attempted.
7. Hook process exits cleanly.

## Invariants

1. The asymmetry is intentional: PreToolUse is fail-open on parse error (BC-HOOK-032); others are fail-closed.
2. Observability hooks silently dropping on malformed stdin is safe — these hooks are state signals, not gates.
3. There is no logging, no stderr output, and no way to distinguish "parse error" from "notification_type filter dropped it" from the outside.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Stop hook; stdin is empty string | `JSON.parse('')` throws; hook silently drops |
| EC-002 | Notification hook; stdin is valid JSON but notification_type is not permission_prompt | BC-HOOK-020's filter runs INSIDE the try block; hook drops via explicit `return` (not error) |
| EC-003 | Notification hook; stdin is malformed JSON | JSON.parse fails; catch fires; hook drops silently |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Stop hook; stdin = `not json` | No stdout; no HTTP call | edge-case |
| Notification hook; stdin = `not json` | No stdout; no HTTP call | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone non-PreToolUse hooks silently drop on malformed stdin | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — silent drop on malformed stdin for observability hooks is the fail-closed lifecycle boundary; these events are not ingested |
| L2 Domain Invariants | DI-001 (tee invariant — malformed-stdin events never reach the daemon and are never written to the ring; DI-001 applies only to events that successfully reach the daemon) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-033 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:35,38,41,44 (`}catch{}})"` — no trailing `console.log(d)` for non-PreToolUse hooks) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-033 (gene-source: deep-hooks-r2 §2 BC-HOOK-033) |
| Test name | test_BC_HOOK_033_non_pretooluse_silent_drop_malformed_json |

## Related BCs

- [BC-HOOK-032] — composes with: BC-HOOK-032 covers the opposite (PreToolUse echoes on malformed stdin)
- [BC-HOOK-002] — composes with: BC-HOOK-002 covers the general fail-closed pattern; this BC covers the specific malformed-stdin case

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:35,38,41,44 (character-level — `}catch{}})"` ends the hook block with no `console.log(d)`; confirmed in r2 §1).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
