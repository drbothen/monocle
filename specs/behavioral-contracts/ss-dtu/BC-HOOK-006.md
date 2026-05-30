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

# BC-HOOK-006: PreToolUse Always Echoes Stdin to Stdout

## Description

The PreToolUse hook process ALWAYS writes its original stdin `d` to stdout via
`console.log(d)` as its final act — regardless of whether the HTTP POST succeeded,
failed, or was never attempted (no server). This unconditional echo is what allows
Claude Code to proceed with the tool invocation: Claude Code reads the hook's stdout
to determine whether to proceed; echoing stdin verbatim signals "allow".

## Preconditions

1. A PreToolUse hook invocation is triggered by Claude Code.
2. Claude Code pipes JSON on stdin.

## Postconditions

1. All stdin is buffered in `d`.
2. After all other hook logic (discovery, HTTP call or skip), `console.log(d)` is called.
3. `d` is written to stdout verbatim — no modification, no re-encoding.
4. The process exits after this final `console.log(d)`.
5. `console.log(d)` fires in ALL cases: server found + POST sent, server not found, parse error, or any other failure path.

## Invariants

1. The `console.log(d)` at the end of the PreToolUse hook body is OUTSIDE the try/catch block — it runs even if JSON parsing fails (see BC-HOOK-032).
2. For the other four hooks, NO unconditional stdout echo exists — they are echo-free.
3. The echo is the PreToolUse "allow" signal: Claude Code interprets an unchanged stdin echo as permission to proceed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Server found; POST succeeds; stdin is valid JSON | stdout = stdin verbatim after POST completes |
| EC-002 | No server found; stdin is valid JSON | stdout = stdin verbatim (fail-open path per BC-HOOK-001) |
| EC-003 | JSON parse fails on stdin | stdout = stdin verbatim (outer catch runs; `console.log(d)` still executes per BC-HOOK-032) |
| EC-004 | POST times out (req.destroy() called) | stdout = stdin verbatim after timeout handling |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Server alive; PreToolUse stdin = `{"tool_name":"Bash","tool_input":{"command":"ls"}}` | stdout = `{"tool_name":"Bash","tool_input":{"command":"ls"}}` | happy-path |
| No server; PreToolUse stdin = `{"tool_name":"Read","tool_input":{}}` | stdout = `{"tool_name":"Read","tool_input":{}}` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone PreToolUse handler unconditionally echoes stdin to stdout | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the unconditional stdin echo is the PreToolUse "allow" signal; it is the non-blocking interface between the hook protocol and Claude Code's tool execution lifecycle |
| L2 Domain Invariants | DI-001 (tee invariant — the echo is structurally separate from the ring-write obligation; even when the ring write fails, PreToolUse echoes stdin to avoid blocking Claude Code) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-018 (PreToolUse fallback) |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (trailing `}catch{}console.log(d);` — unconditional at end) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-006 (gene-source: any-context Pass 3) |
| Test name | test_BC_HOOK_006_pretooluse_unconditional_stdin_echo |

## Related BCs

- [BC-HOOK-001] — composes with: BC-HOOK-001 covers the no-server fail-open case; this BC covers the unconditional echo mechanism
- [BC-HOOK-032] — composes with: BC-HOOK-032 covers the malformed-stdin sub-case where echo still fires

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (`}catch{}console.log(d);` — outside catch, always executes).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
