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
  - {path: .factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r2.md, version: "r2"}
  - {path: .factory/specs/dtu-assessment.md, version: "1.7.5"}
input-hash: "[live-state]"
traces_to: prd.md
origin: gene-transfusion
subsystem: SS-01
capability: CAP-001
# DTU-specific fields
dtu_service: claude-code-hook-protocol
gene_source: any-context-lazyclaude/internal/core/config/hooks.go
# Lifecycle fields (DF-030)
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

# BC-HOOK-001: PreToolUse Hook Fail-Open Semantics (No Server Found)

## Description

When the monocle daemon is unreachable (no alive lock file found), the PreToolUse
hook process MUST echo its stdin unchanged to stdout so that Claude Code's tool
invocation proceeds unblocked. This is the foundational fail-open contract: if
monocle is down, Claude Code continues to function normally. The hook acts as a
transparent passthrough.

## Preconditions

1. No alive monocle daemon lock file exists at `~/.monocle/run/*.lock` (or the
   equivalent `MONOCLE_RUNTIME_DIR`-derived path).
2. A PreToolUse hook invocation is triggered by Claude Code, which pipes JSON on
   stdin.

## Postconditions

1. The hook process reads all stdin to a buffer `d`.
2. Lock-file discovery finds no alive server: `srvPort` remains `null`.
3. `console.log(d)` is called — stdin is echoed to stdout verbatim.
4. The hook process exits cleanly (exit code 0 implied).
5. Claude Code receives the echoed stdin and proceeds with the tool call.
6. No HTTP request is attempted.

## Invariants

1. PreToolUse fail-open is non-negotiable: Claude Code's tool call MUST NOT block
   even when monocle is unavailable.
2. The stdin echo happens regardless of whether the JSON in `d` is valid or
   malformed — it is byte-for-byte passthrough.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Lock file directory does not exist | `fs.readdirSync` throws; caught by outer try/catch; `srvPort` remains null; stdin echoed |
| EC-002 | Lock file directory exists but is empty (no .lock files) | Loop body never executes; `srvPort` remains null; stdin echoed |
| EC-003 | All .lock files have dead PIDs | All fail the `process.kill(pid, 0)` check; `srvPort` remains null; stdin echoed |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| No lock files present; PreToolUse stdin = `{"tool_name":"Bash","tool_input":{"command":"ls"}}` | stdout = `{"tool_name":"Bash","tool_input":{"command":"ls"}}` verbatim | happy-path |
| Dead-PID lock file only; stdin = `{"tool_name":"Read","tool_input":{"file_path":"/tmp/x"}}` | stdout = `{"tool_name":"Read","tool_input":{"file_path":"/tmp/x"}}` verbatim | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone respects PreToolUse fail-open semantics: no lock file → stdin echoed unchanged | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — this BC defines the fail-open semantics of PreToolUse, which is foundational to hook event ingestion and daemon lifecycle |
| L2 Domain Invariants | DI-001 (tee invariant — when daemon is unreachable, PreToolUse fail-open ensures Claude Code proceeds; the zero-event case is the boundary condition of DI-001) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-018 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (`if(!srvPort){console.log(d);return;}`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-001 (gene-source: any-context Pass 3) |
| Test name | test_BC_HOOK_001_pretooluse_fail_open_no_server |

## Related BCs

- [BC-HOOK-018] — supersedes: BC-HOOK-018 refines this contract with exact per-hook fallback semantics matrix
- [BC-HOOK-032] — composes with: BC-HOOK-032 covers the malformed-stdin variant of fail-open

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach` — hook fidelity requirements and gene-source derivation

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source: any-context-lazyclaude hooks.go pass-3 + r1 + r2 ingest rounds.
- Gene-source file:line: hooks.go:31 (`if(!srvPort){console.log(d);return;}` and trailing `console.log(d)`).
- Authored for S-DTU-001 DTU clone prerequisite gate (story status: draft → ready).
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation; no prior high-water.
