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

# BC-HOOK-021: All HTTP Requests Are Fire-and-Forget (Deep-Ingest Confirmation)

## Description

Deep-ingest confirmation (hooks-r1 file:line precision) of BC-HOOK-004. The
fire-and-forget pattern — `req.on('error',()=>{})`, `req.on('timeout',()=>{req.destroy()})`,
`req.write(body)`, `req.end()`, no response read — is identical across ALL five hook types
(hooks.go:31, 35, 38, 41, 44).

## Preconditions

1. A hook HTTP request is being sent (server is alive).

## Postconditions

1. `req.on('error', () => {})` is registered — network errors are swallowed.
2. `req.on('timeout', () => { req.destroy(); })` is registered — timeouts destroy the socket.
3. `req.write(body)` sends the body.
4. `req.end()` closes the write side.
5. No response status code or body is read.
6. The hook process does NOT wait for a response before exiting.

## Invariants

1. Fire-and-forget is the only delivery semantic for hook events. No retry, no ack, no response inspection.
2. At-most-once delivery: if the request fails (network, timeout, server error), the event is permanently lost.
3. This implies the monocle daemon MUST NOT reply with data the hook needs — the hook ignores all responses.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Server responds with 200 | Response ignored; hook exits normally |
| EC-002 | Server responds with 503 | Response ignored; hook exits normally |
| EC-003 | Network error before response | `req.on('error')` fires; `()=>{}` swallows; hook exits normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Hook POST with server alive; server returns 200 | Hook exits 0; no stdout except PreToolUse echo | happy-path |
| Hook POST with server alive; server returns 500 | Same: response ignored; hook exits 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone does not read HTTP response body or status code from any hook POST | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — fire-and-forget delivery is the protocol-level delivery guarantee for hook events entering the daemon's ingestion pipeline |
| L2 Domain Invariants | DI-001 (tee invariant — fire-and-forget means events may be lost on error; DI-001's ring-write-before-ack obligation is therefore a server-side guarantee, not a hook-side guarantee) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-021 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31,35,38,41,44 (identical fire-and-forget pattern) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-021 (gene-source: deep-hooks-r1 §6 BC-HOOK-021) |
| Test name | test_BC_HOOK_021_fire_and_forget_deep_ingest |

## Related BCs

- [BC-HOOK-004] — supersedes: BC-HOOK-004 covers fire-and-forget at pass-3 confidence; this BC provides r1 file:line precision

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31,35,38,41,44 (all 5 hooks: `req.on('error',()=>{});req.on('timeout',()=>{req.destroy()});`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
