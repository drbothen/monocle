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

# BC-HOOK-020: Notification Client-Side Filter notification_type === 'permission_prompt' (Deep-Ingest Confirmation)

## Description

This BC provides deep-ingest confirmation (hooks-r1 file:line precision) of BC-HOOK-003's
notification_type filter. The filter `if(i.notification_type !== 'permission_prompt') return;`
is present at hooks.go:35 and is the ONLY place where Claude Code's `notification_type` field
is read in the hook protocol. The server has no knowledge of this pre-filter.

## Preconditions

1. A Notification hook invocation fires with any `notification_type` value.
2. Server is alive (srvPort is non-null).

## Postconditions

1. If `i.notification_type !== 'permission_prompt'`: return immediately; NO HTTP POST.
2. If `i.notification_type === 'permission_prompt'`: continue to HTTP POST.
3. The `notification_type` field is read from `i` (parsed stdin JSON), NOT injected by the hook.
4. The server endpoint only ever receives `permission_prompt` Notification POSTs.

## Invariants

1. This filter is the sole client-side gate on notification type. No server-side re-filtering occurs.
2. The filter is case-sensitive and type-exact: `'permission_prompt'` as a string value.
3. Cross-reference: `i.notification_type` is also the field name in the monocle-canonical Notification body schema (SS-core-types-and-abi.md v1.2.13).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `notification_type` is `null` | `null !== 'permission_prompt'` → dropped |
| EC-002 | `notification_type` is `undefined` (field absent) | `undefined !== 'permission_prompt'` → dropped |
| EC-003 | `notification_type` is `'PERMISSION_PROMPT'` (uppercase) | String inequality → dropped |
| EC-004 | `notification_type` is `'permission_prompt'` with extra whitespace | Strict inequality (whitespace matters) → dropped |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `{"notification_type":"permission_prompt","message":"Allow?"}` | POST sent | happy-path |
| `{"notification_type":"agent_turn_start"}` | No POST | edge-case |
| `{"notification_type":null}` | No POST | edge-case |
| `{}` (no notification_type) | No POST | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone Notification handler filters on exact 'permission_prompt' string with case sensitivity | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the Notification filter determines which events enter the daemon's ingestion pipeline; the permission_prompt filter is a critical selection gate for the permission-overlay lifecycle flow |
| L2 Domain Invariants | DI-001 (tee invariant — non-permission_prompt notifications are pre-filtered at the hook layer; they never reach the daemon and therefore are not subject to the ring-write obligation) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Clone Development Approach; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-020 |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:35 (`if(i.notification_type!=='permission_prompt')return;`) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-020 (gene-source: deep-hooks-r1 §6 BC-HOOK-020) |
| Test name | test_BC_HOOK_020_notification_filter_deep_ingest_confirmation |

## Related BCs

- [BC-HOOK-003] — supersedes: BC-HOOK-003 covers the same filter at pass-3 confidence; this BC provides r1 file:line precision

## Architecture Anchors

- `specs/dtu-assessment.md#clone-development-approach`

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:35 (verbatim `if(i.notification_type!=='permission_prompt')return;` — exact character-level evidence).
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
