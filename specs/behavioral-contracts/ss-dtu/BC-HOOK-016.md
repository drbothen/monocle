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
  - {path: .factory/specs/architecture/adr/ADR-0005.md, version: "1.0.2"}
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

# BC-HOOK-016: Auth Header Name Is X-Claude-Code-Ide-Authorization (Hardcoded in Hook Source)

## Description

All five hook HTTP requests send the auth token in the header
`X-Claude-Code-Ide-Authorization`. This header name is hardcoded in the Claude Code
hook source (`hooks.go:31` and identical at lines 35, 38, 41, 44). It is NOT
`X-Monocle-Authorization` (monocle's canonical header). The DTU clone MUST use
`X-Claude-Code-Ide-Authorization` to exercise the monocle daemon's ADR-0005
compatibility alias code path. A separate unit test exercises the canonical
`X-Monocle-Authorization` path.

## Preconditions

1. An alive monocle daemon lock file is found with valid port and authToken.
2. A hook HTTP POST is being constructed.

## Postconditions

1. The HTTP request includes the header `X-Claude-Code-Ide-Authorization: <raw-token>`.
2. The raw token is the verbatim `authToken` string from the lock file (no prefix added by the hook).
3. No `X-Monocle-Authorization` header is sent by the hook.
4. The monocle daemon accepts the request via its ADR-0005 compatibility alias code path.

## Invariants

1. The header name is hardcoded — not configurable via env var or hooks-settings.json field.
2. The DTU clone tests the alias path (`X-Claude-Code-Ide-Authorization`); separate unit tests test the canonical path (`X-Monocle-Authorization`).
3. Both headers must be tested to verify ADR-0005 dual-accept is correctly implemented in the daemon.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Daemon only accepts `X-Monocle-Authorization` (ADR-0005 dual-accept not implemented) | 401 response; hook swallows (fire-and-forget); event not ingested |
| EC-002 | Both headers present on a single request (hypothetical future) | ADR-0005: canonical `X-Monocle-Authorization` takes priority; alias generates WARN-level log |
| EC-003 | Hook sends raw token `abc123...` (64 hex chars) without `monocle-v1:` prefix | Daemon's alias code path accepts raw token directly (no prefix required for alias header) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| DTU clone POST to `/hooks/pre-tool-use` with raw token `abc...` | Header `X-Claude-Code-Ide-Authorization: abc...` present on wire | happy-path |
| Daemon receives `X-Claude-Code-Ide-Authorization: abc...` | Daemon accepts via alias path (ADR-0005); HTTP 200 | happy-path |
| Daemon receives only `X-Monocle-Authorization: monocle-v1:abc...` | Daemon accepts via canonical path | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone sends X-Claude-Code-Ide-Authorization header on all hook POSTs | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the auth header name is the wire identity of hook requests arriving at the daemon's ingestion endpoints; testing the alias path (BC-HOOK-016) is required by ADR-0005 |
| L2 Domain Invariants | DI-005 (auth validation — the daemon MUST validate the auth header; this BC specifies which header name real Claude Code sends, which must be accepted per DI-005 + ADR-0005 dual-accept) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Auth Header (lines 101-110, auth header column rationale); ADR-0005 v1.0.2; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-016 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (`headers:{'X-Claude-Code-Ide-Authorization':srvToken}`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-016 (gene-source: deep-hooks-r1 §5 BC-HOOK-016) |
| Test name | test_BC_HOOK_016_auth_header_x_claude_code_ide_authorization |

## Related BCs

- [BC-HOOK-015] — depends on: BC-HOOK-015 covers token extraction; this BC covers the header name used to transmit it
- [BC-2.01.009] — composes with: BC-2.01.009 covers the daemon's auth validation accepting both headers per ADR-0005

## Architecture Anchors

- `specs/dtu-assessment.md#auth-header`
- `specs/architecture/adr/ADR-0005.md` — dual-accept auth header specification

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (`X-Claude-Code-Ide-Authorization` header — hardcoded, identical at lines 35, 38, 41, 44).
- dtu-assessment.md §Auth header column rationale (lines 101-110) cited as canonical authority.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
