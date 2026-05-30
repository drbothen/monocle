---
document_type: behavioral-contract
level: L3
version: "1.0.1"
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

# BC-HOOK-002: Non-PreToolUse Hooks Fail-Closed (No Server Found)

## Description

When the monocle daemon is unreachable, the Notification, Stop, SessionStart, and
UserPromptSubmit hook processes MUST exit silently without echoing stdin or making
any HTTP request. This fail-closed behavior is intentional: these four hooks are
observability/state signals, not gates for Claude Code operation. Dropping them
when monocle is down is safe; the tool execution is never blocked.

## Preconditions

1. No alive monocle daemon lock file exists.
2. A hook invocation for one of: Notification, Stop, SessionStart, or UserPromptSubmit
   is triggered by Claude Code.

## Postconditions

1. The hook process reads all stdin to buffer `d`.
2. Lock-file discovery finds no alive server: `srvPort` remains `null`.
3. `if(!srvPort) return;` executes — the function returns immediately.
4. No `console.log(d)` is called (stdin is NOT echoed to stdout).
5. No HTTP request is attempted.
6. Hook process exits cleanly.
7. Claude Code receives no response from the hook (the hook produces no stdout), which is acceptable for non-blocking hook types.

## Invariants

1. Exactly four hook types use fail-closed (not five): Notification, Stop, SessionStart, UserPromptSubmit.
2. PreToolUse MUST NOT use fail-closed — it uses fail-open (BC-HOOK-001).
3. The asymmetry is architectural: fail-closed on observability hooks never blocks Claude Code; fail-open on PreToolUse ensures tool execution proceeds.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No lock files; Notification hook triggered | Return immediately; no stdout; no HTTP |
| EC-002 | No lock files; Stop hook triggered | Return immediately; no stdout; no HTTP |
| EC-003 | No lock files; SessionStart hook triggered | Return immediately; no stdout; no HTTP |
| EC-004 | No lock files; UserPromptSubmit hook triggered | Return immediately; no stdout; no HTTP |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| No lock files; Notification stdin = `{"notification_type":"permission_prompt","message":"Allow Bash?"}` | No stdout; no HTTP call | happy-path |
| No lock files; Stop stdin = `{"stop_reason":"normal","session_id":"abc-123"}` | No stdout; no HTTP call | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone respects fail-closed semantics for Notification/Stop/SessionStart/UserPromptSubmit when no server is found | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — this BC defines the fail-closed fallback for observability hooks, part of the hook event ingestion lifecycle |
| L2 Domain Invariants | DI-001 (tee invariant — fail-closed on these hook types is the zero-ingestion boundary; no event is written when daemon is absent, which is within the tee invariant's scope) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-018 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:35,38,41,44 (`if(!srvPort)return;`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-002 (gene-source: any-context Pass 3) |
| Test name | test_BC_HOOK_002_non_pretooluse_fail_closed_no_server |

## Related BCs

- [BC-HOOK-001] — composes with: BC-HOOK-001 covers the PreToolUse fail-open counterpart
- [BC-HOOK-018] — supersedes: BC-HOOK-018 is the consolidated matrix of all per-hook fallback semantics
- [BC-HOOK-033] — composes with: BC-HOOK-033 covers the malformed-stdin variant of fail-closed

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach` — hook fidelity requirements and gene-source derivation

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:35, 38, 41, 44 (`if(!srvPort)return;` — identical pattern for 4 hooks).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
