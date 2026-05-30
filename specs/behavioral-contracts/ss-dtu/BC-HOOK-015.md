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

# BC-HOOK-015: Auth Token Resolved at Each Invocation from Lock File authToken Field

## Description

The auth token (`srvToken`) is read from the alive lock file's `authToken` JSON field on
every hook invocation. The token is NOT embedded in the hooks-settings.json file —
it is discovered dynamically alongside the port. Token rotation (daemon restart) is
automatically handled because the next invocation re-reads the new lock file with
the new token.

## Preconditions

1. An alive lock file exists with valid `port` (from filename) and `authToken` (from content).
2. The lock file JSON is parseable.

## Postconditions

1. `srvToken = best.lock.authToken` — the raw auth token string from the lock file.
2. The token is passed verbatim in the `X-Claude-Code-Ide-Authorization` header (BC-HOOK-016).
3. No prefix is added — the raw token is sent as-is.
4. If the token rotates (daemon restart + new lock file), the next hook invocation automatically uses the new token.

## Invariants

1. The auth token is sourced from the same lock file as the port (atomic: valid lock = valid port + valid token).
2. The hooks-settings.json contains NO static token value — tokens are runtime-ephemeral.
3. Token rotation is implicit: the daemon writes a new token to the new lock file on restart; hooks automatically pick it up on the next invocation scan.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Lock file exists but `authToken` field is absent | `best.lock.authToken` is `undefined`; request sends `X-Claude-Code-Ide-Authorization: undefined` (string) — daemon rejects with 401 |
| EC-002 | Token rotated between two hook invocations (daemon restart) | First hook uses old token (old lock); second hook re-scans, finds new lock, uses new token |
| EC-003 | `authToken` is an empty string | Empty token sent; daemon rejects with 401 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Lock file with `authToken: "abc123def456..."` | Header `X-Claude-Code-Ide-Authorization: abc123def456...` | happy-path |
| Token rotated; new lock file with `authToken: "xyz789..."` | Next invocation sends `X-Claude-Code-Ide-Authorization: xyz789...` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone reads authToken from lock file and sends it in X-Claude-Code-Ide-Authorization header | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — dynamic token resolution from the lock file is the mechanism that maintains authenticated hook-to-daemon connectivity across token rotations, a core lifecycle management capability |
| L2 Domain Invariants | DI-003 (token write order — the token is written to the lock file AFTER the port is bound; this BC is the consumer-side complement: hooks only read the token after it has been written) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-015; ADR-0005 v1.0.2 (auth header dual-accept) |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:27 (`srvToken=best.lock.authToken`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-015 (gene-source: deep-hooks-r1 §5 BC-HOOK-015) |
| Test name | test_BC_HOOK_015_auth_token_from_lock_file_per_invocation |

## Related BCs

- [BC-HOOK-013] — depends on: BC-HOOK-013 covers the lock file scan; this BC covers the token extraction from the selected lock
- [BC-HOOK-016] — composes with: BC-HOOK-016 covers the specific header name used to transmit the token

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`
- `specs/architecture/adr/ADR-0005.md` — auth header dual-accept; X-Claude-Code-Ide-Authorization alias

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:27 (`srvToken=best.lock.authToken`); lock.go:44-49 (`AuthToken: token`).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
