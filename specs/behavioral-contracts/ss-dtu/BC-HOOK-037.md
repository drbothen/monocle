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

# BC-HOOK-037: req.write(body) Followed by req.end() Sends Body and Closes Write-Side Immediately

## Description

The hook sends its POST body by calling `req.write(body)` followed immediately by
`req.end()`. This sends the body in a single write and signals EOF on the write side.
The hook process then returns from the `'end'` callback, effectively exiting. No
response is waited for (BC-HOOK-004/021). The TCP connection is half-closed after
`req.end()`.

## Preconditions

1. An HTTP POST request has been constructed with all required headers.
2. `body` is the JSON-serialized hook event payload.

## Postconditions

1. `req.write(body)` sends the body bytes to the server.
2. `req.end()` closes the write side of the TCP connection.
3. The callback (`process.stdin.on('end', ...)`) returns after `req.end()`.
4. No response is read — the connection is abandoned.
5. The node process exits after the callback returns (or after PreToolUse's `console.log(d)`).

## Invariants

1. `req.write()` + `req.end()` is the standard Node.js pattern for sending a request body and finalizing.
2. The server (monocle daemon) must handle half-closed TCP connections correctly — it receives the body, processes it, and can respond (but the hook won't read the response).
3. Body is sent in a single `req.write()` — there are no chunked writes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `req.write()` returns false (buffer full) | Drain event would be needed for flow control; hook doesn't wait — this is fire-and-forget; in practice, hook bodies are small (<4KB) and buffer overflow is not a concern |
| EC-002 | Server closes connection before hook writes body | `req.on('error')` fires; `()=>{}` swallows; hook exits |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Hook constructs and sends POST | `req.write(body)` + `req.end()` called; no `req.read()` called | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone uses req.write() + req.end() pattern with no response read | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the write+end pattern is the fire-and-forget delivery mechanism for hook events; the daemon must handle half-closed connections |
| L2 Domain Invariants | DI-001 (tee invariant — the write+end pattern is the delivery mechanism; the daemon receives the body and must complete the ring write before responding per DI-001) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r2.md §BC-HOOK-037 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (`req.write(body);req.end();`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-037 (gene-source: deep-hooks-r2 §2 BC-HOOK-037) |
| Test name | test_BC_HOOK_037_req_write_req_end_pattern |

## Related BCs

- [BC-HOOK-004] — composes with: BC-HOOK-004 covers the fire-and-forget semantics; this BC covers the specific write/end invocation pattern

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (`req.write(body);req.end();` — identical pattern at 35, 38, 41, 44).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
