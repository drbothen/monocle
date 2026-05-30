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
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
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

# BC-HOOK-019: Gene-Source Endpoint Matrix (PreToolUse and Notification Share /notify via type Field)

## Description

In the gene source (any-context-lazyclaude), PreToolUse and Notification both POST
to the same `/notify` endpoint. The server discriminates on the presence or absence
of the `type: 'tool_info'` field. NOTE: monocle uses SEPARATE endpoints
(`/hooks/pre-tool-use` and `/hooks/notification`) per the dtu-assessment.md endpoint
matrix. This BC documents the gene-source behavior for provenance; the monocle
implementation MUST use the monocle-canonical paths, not the gene-source paths.

## Preconditions

1. Gene-source behavior: applicable to any-context-lazyclaude implementation.
2. Monocle behavior: separate endpoints used per dtu-assessment.md §Endpoint Matrix.

## Postconditions (gene-source behavior — for DTU provenance only)

1. PreToolUse POSTs to `/notify` with body `{type: 'tool_info', pid, tool_name, tool_input}`.
2. Notification POSTs to `/notify` with body `{pid, tool_name, tool_input, message}` — NO `type` field.
3. Server at `/notify` dispatches based on presence of `type` field: `'tool_info'` → PreToolUse handling; no type / `'permission_prompt'` → Notification handling.

## Postconditions (monocle-canonical behavior — NORMATIVE for implementation)

1. PreToolUse POSTs to `/hooks/pre-tool-use` with monocle-canonical body: `{session_id, pid, tool_name, tool_input}`.
2. Notification POSTs to `/hooks/notification` with monocle-canonical body: `{session_id, pid, notification_type, tool_name, tool_input, message}`.
3. The monocle daemon has SEPARATE handlers for `/hooks/pre-tool-use` and `/hooks/notification`.

## Invariants

1. The gene-source shared-endpoint design is NOT replicated in monocle. monocle uses separate endpoints for clarity.
2. The `type: 'tool_info'` discriminator field is a gene-source artifact NOT present in the monocle-canonical body schema.
3. The dtu-assessment.md §Endpoint Matrix monocle-canonical column is the authoritative source for monocle path names and body fields.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DTU clone accidentally uses gene-source `/notify` path | Daemon returns 404 (no `/notify` endpoint); test fails; clone must use monocle-canonical paths |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| DTU clone PreToolUse POST | Path is `/hooks/pre-tool-use` (NOT `/notify`) | lint |
| DTU clone Notification POST | Path is `/hooks/notification` (NOT `/notify`) | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | DTU clone uses monocle-canonical paths for all 5 endpoints | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the endpoint path mapping is a core element of the hook event ingestion protocol; the monocle-canonical paths are the normative contract for all hook delivery to the daemon |
| L2 Domain Invariants | DI-001 (tee invariant — hook events must reach the daemon's endpoints; using the wrong path means events are not ingested, violating DI-001) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md §Endpoint Matrix (monocle-canonical column); semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §BC-HOOK-019 (gene-source provenance) |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:31 (`path:'/notify'` for PreToolUse); hooks.go:35 (`path:'/notify'` for Notification with no type field) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-019 (gene-source: deep-hooks-r1 §6 BC-HOOK-019) |
| Test name | test_BC_HOOK_019_monocle_canonical_endpoints_not_gene_source |

## Related BCs

- [BC-HOOK-007] — depends on: BC-HOOK-007 establishes the 5 hook types; this BC covers the URL paths
- [BC-HOOK-020] — composes with: BC-HOOK-020 covers the notification_type filter on the Notification hook

## Architecture Anchors

- `specs/dtu-assessment.md#endpoint-matrix` — authoritative monocle-canonical vs gene-source path comparison

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:31 (`path:'/notify'` PreToolUse); hooks.go:35 (`path:'/notify'` Notification — shared path, no type field).
- NORMATIVE NOTE: monocle uses separate paths per dtu-assessment.md endpoint matrix.
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
## §Trace v1.0.1

**POL-11 version-pin remediation — dtu-assessment Architecture Source version-free** (2026-05-30):
- Architecture Source table row: `dtu-assessment.md v1.7.5 §...` → `dtu-assessment.md §...` (Option 2, version-free; per ADR-0007 §Decision — navigation pointer to canonical source, permanently prevents re-staling).
- Version bump: 1.0.0 → 1.0.1.
- SE-16d PASS: 2026-05-30 >= 2026-05-20T21:00:00Z (patch; no normative content change).
