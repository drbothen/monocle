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

# BC-HOOK-004: Hook HTTP Requests Are Fire-and-Forget (Response Ignored)

## Description

Every hook HTTP POST is fire-and-forget: the hook script writes the body, calls
`req.end()`, and does NOT read the HTTP response. The server can return any status
code (200, 401, 500, etc.) and the hook will not observe it. Error events on the
request are swallowed via `req.on('error', () => {})`. Timeout events destroy
the socket via `req.on('timeout', () => { req.destroy(); })`. This applies to all
five hook types.

## Preconditions

1. An alive monocle daemon is running (lock file found with live PID).
2. The hook script constructs a valid HTTP POST body.

## Postconditions

1. `req.write(body)` is called with the JSON body.
2. `req.end()` is called — write-side is closed.
3. No response body or status code is read from the connection.
4. `req.on('error', () => {})` is registered — network errors are swallowed silently.
5. `req.on('timeout', () => { req.destroy(); })` is registered — timeout destroys the socket.
6. The hook process exits after `req.end()` (with stdin echo for PreToolUse, or immediately for others).

## Invariants

1. Fire-and-forget applies to ALL five hook types without exception.
2. The monocle daemon's response body and status code are diagnostic-only — no hook logic depends on them.
3. At-most-once delivery: if the TCP connection fails or times out, the event is silently lost.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Server returns HTTP 401 (wrong token) | Hook swallows the response; no retry; event silently dropped |
| EC-002 | Server returns HTTP 500 (internal error) | Same: response swallowed; event silently dropped |
| EC-003 | Network timeout fires before response received | `req.destroy()` called; socket closed; process exits |
| EC-004 | TCP connection refused (port not listening) | `req.on('error')` fires; `() => {}` swallows; process exits |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Server returns 200 OK | Hook exits normally; no stdout change | happy-path |
| Server returns 401 | Hook exits normally (response ignored) | edge-case |
| Simulated TCP timeout | Hook calls req.destroy(); exits normally | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone hook scripts do not read response body or status code | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — fire-and-forget delivery semantics are fundamental to hook event ingestion; the daemon must handle each incoming POST idempotently without expecting acknowledgement |
| L2 Domain Invariants | DI-001 (tee invariant — at-most-once delivery is the lower bound; DI-001 requires ring write BEFORE ack, which is compatible with the hook not reading the ack at all) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-021 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31,35,38,41,44 (`req.on('error',()=>{});req.on('timeout',()=>{req.destroy()});req.write(body);req.end();`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-004 (gene-source: any-context Pass 3) |
| Test name | test_BC_HOOK_004_hook_requests_fire_and_forget |

## Related BCs

- [BC-HOOK-021] — supersedes: BC-HOOK-021 is the deep-ingest confirmation with exact source evidence
- [BC-HOOK-022] — composes with: timeout values per hook type (300ms vs 2000ms) govern when req.destroy() fires

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31,35,38,41,44 (fire-and-forget pattern identical across all 5 hooks).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
