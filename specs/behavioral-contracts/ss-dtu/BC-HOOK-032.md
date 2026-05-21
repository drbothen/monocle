---
document_type: behavioral-contract
level: L3
version: "1.0.0"
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

# BC-HOOK-032: Malformed Stdin JSON Does NOT Prevent Stdin Echo for PreToolUse (Doubly Fail-Open)

## Description

If Claude Code pipes malformed JSON to the PreToolUse hook, `JSON.parse(d)` throws.
The outer `try{}catch{}` catches the exception. The `console.log(d)` at the END of
the outer block (after the catch) runs unconditionally. Stdin is echoed verbatim
even on parse failure. This is "doubly fail-open": both (a) no server found and
(b) malformed stdin converge on echoing stdin unchanged.

## Preconditions

1. PreToolUse hook is invoked.
2. Claude Code pipes malformed JSON (or any non-JSON bytes) on stdin.

## Postconditions

1. Stdin is read into `d` (raw bytes as string).
2. `JSON.parse(d)` throws.
3. The outer `catch{}` block executes.
4. `console.log(d)` runs AFTER the catch — it is outside the try/catch block.
5. Stdin `d` is echoed to stdout verbatim.
6. No HTTP POST is attempted.
7. Claude Code receives the echoed stdin and proceeds (assuming it can handle the original malformed data, which is its own concern).

## Invariants

1. The `console.log(d)` is deliberately placed OUTSIDE the try/catch for PreToolUse — this is what makes it run on all code paths.
2. For the other four hooks, there is NO `console.log(d)` at end — they are silently dropped on parse failure (BC-HOOK-033).
3. The malformed stdin echo is byte-for-byte — `d` is the raw buffer content, not re-serialized.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty stdin (EOF immediately) | `d = ''`; `JSON.parse('')` throws; `console.log('')` outputs empty line |
| EC-002 | Truncated JSON (`{"tool_name":`) | Parse error; raw truncated string echoed |
| EC-003 | Binary data on stdin | `d` contains binary-as-string; parse error; binary echoed back |
| EC-004 | Valid JSON (normal case) | `JSON.parse(d)` succeeds; HTTP POST attempted (if server alive); `console.log(d)` still runs at end |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| PreToolUse; stdin = `not json` | stdout = `not json` (verbatim echo) | edge-case |
| PreToolUse; stdin = `{"tool_name":"Bash"}` (valid) | stdout = `{"tool_name":"Bash"}` (after HTTP POST) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone PreToolUse echoes stdin on malformed JSON (doubly fail-open) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the doubly fail-open behavior is the strongest possible form of the PreToolUse lifecycle guarantee: Claude Code tool execution is never blocked regardless of hook input validity |
| L2 Domain Invariants | DI-001 (tee invariant boundary — malformed stdin means no event is ingested; the echo path exists to avoid blocking Claude Code, not to satisfy DI-001) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-032 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (`}catch{}console.log(d);` — `console.log(d)` is AFTER the catch, outside the try block) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-032 (gene-source: deep-hooks-r2 §2 BC-HOOK-032) |
| Test name | test_BC_HOOK_032_pretooluse_echo_on_malformed_json |

## Related BCs

- [BC-HOOK-006] — composes with: BC-HOOK-006 covers the unconditional echo; this BC covers the malformed-stdin edge case
- [BC-HOOK-033] — composes with: BC-HOOK-033 covers the opposite behavior for non-PreToolUse hooks on parse failure

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (`}catch{}console.log(d);` — character-level verification in r2 §1).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
