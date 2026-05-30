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

# BC-HOOK-030: MONOCLE_SESSION_ID Env Var Is Set on Claude Code Subprocess but NOT Read by Hook JS

## Description

monocle sets `MONOCLE_SESSION_ID` in the Claude Code subprocess environment (as
`LAZYCLAUDE_SESSION_ID` in the gene source). This env var is available to user's
custom slash commands, codex plugins, and other tools that run inside the Claude Code
session. However, the hook JS itself does NOT read it — hooks use `process.ppid` for
PID correlation and read the session_id from the daemon lock file or hook body JSON,
not from this env var.

## Preconditions

1. A monocle session is being launched.

## Postconditions

1. `MONOCLE_SESSION_ID=<session-id>` is set in the Claude Code subprocess environment.
2. The hook JS does NOT read `MONOCLE_SESSION_ID` (zero `process.env.MONOCLE_SESSION_ID` references in hook code).
3. The env var is available to any code running inside the Claude Code session.

## Invariants

1. `MONOCLE_SESSION_ID` is for OUT-OF-PROCESS consumers — tools running inside Claude Code that want to identify the owning monocle session.
2. The hook JS identifies sessions via `process.ppid` (for PID correlation) and the daemon-provided session ID, NOT via this env var.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Custom user slash command reads `$MONOCLE_SESSION_ID` | Returns the monocle session ID; hook JS behavior unchanged |
| EC-002 | Hook JS accidentally reads `process.env.MONOCLE_SESSION_ID` | Bug (not expected); hook behavior would be env-dependent, violating BC-HOOK-029 |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Inspect Claude Code subprocess env | `MONOCLE_SESSION_ID` is set | lint |
| Hook JS source code | Zero `process.env.MONOCLE_SESSION_ID` references | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone sets MONOCLE_SESSION_ID in subprocess env; hook JS does not read it | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — MONOCLE_SESSION_ID is part of the session launch lifecycle; its presence in the subprocess env enables session identification for downstream tools |
| L2 Domain Invariants | None directly (env var injection is an implementation detail of session launch) |
| Architecture Module | monocle-runtime (daemon binary, session launcher) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-030 |
| Gene Source | any-context-lazyclaude/internal/session/manager.go:854-855 (set `LAZYCLAUDE_SESSION_ID`); hooks.go (not read) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-030 (gene-source: deep-hooks-r1 §9 BC-HOOK-030) |
| Test name | test_BC_HOOK_030_monocle_session_id_env_set_not_read_by_hooks |

## Related BCs

- [BC-HOOK-029] — composes with: BC-HOOK-029 covers the hook JS's env-independence; this BC covers the specific MONOCLE_SESSION_ID env var that is set but not read

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: manager.go:854-855 (`LAZYCLAUDE_SESSION_ID` set); hooks.go (zero references confirmed by hooks-r1 §9).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
