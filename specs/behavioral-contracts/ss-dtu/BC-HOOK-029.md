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

# BC-HOOK-029: Hook Process Reads Only os.homedir() from Environment — Env-Independent for All Other Vars

## Description

The hook inline JS reads ONLY `os.homedir()` from the environment (which uses `$HOME`
or `$USERPROFILE` on Windows). It does NOT read `MONOCLE_RUNTIME_DIR`, `MONOCLE_SESSION_ID`,
`CLAUDE_CODE_AUTO_CONNECT_IDE`, `ANTHROPIC_API_KEY`, or any other env var. The hook
logic is purely: parse stdin → scan lock dir → build HTTP request. No env var injection
by monocle affects the hook script's behavior except via the lock file filesystem.

## Preconditions

1. A hook process is executing.

## Postconditions

1. `process.env` is NOT read by the hook script (beyond `os.homedir()` which reads `$HOME`).
2. The hook process inherits the full Claude Code subprocess environment but ignores all env vars except `$HOME`.
3. The hook behavior is identical regardless of what env vars are set (as long as `$HOME` is set).

## Invariants

1. `$HOME` must be set for the hook to locate the lock directory. If `$HOME` is unset: `os.homedir()` returns an empty string or throws; lock dir scan fails; srvPort = null; fail-open/closed.
2. The hook's env-independence makes it portable across PM, Worker, and plain sessions without modification.
3. **Exception (monocle improvement):** BC-HOOK-014 specifies that monocle's inline JS SHOULD read `process.env.MONOCLE_RUNTIME_DIR` as an improvement over the gene source. If implemented, this would be the ONLY env var read by the hook JS beyond `$HOME`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `$HOME` is not set | `os.homedir()` returns empty string or platform default; lock dir scan may fail; srvPort = null |
| EC-002 | `MONOCLE_SESSION_ID` is set | Ignored by hook JS (not read) |
| EC-003 | `ANTHROPIC_API_KEY` is set | Ignored by hook JS (not read) |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Hook process with custom env vars | Behavior unchanged (only $HOME matters) | lint |
| Hook process with `$HOME=/tmp/test` | Lock dir scan at `/tmp/test/.monocle/run/` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone hook JS reads only os.homedir() from environment | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the hook JS's env-independence is a portability property that ensures all session types participate equally in hook event ingestion |
| L2 Domain Invariants | None directly (env-independence is a portability property) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-029 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:13-44 (zero `process.env.<X>` references; only `process.ppid`, `os.homedir()`, and `process.stdin` consulted) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-029 (gene-source: deep-hooks-r1 §9 BC-HOOK-029) |
| Test name | test_BC_HOOK_029_hook_env_independence |

## Related BCs

- [BC-HOOK-014] — composes with: BC-HOOK-014 covers the specific env-var asymmetry for the lock dir path; this BC covers the overall env-independence property

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:13-44 (full inline JS — zero `process.env` reads confirmed; only `process.ppid`, `os.homedir()`, `process.stdin`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
