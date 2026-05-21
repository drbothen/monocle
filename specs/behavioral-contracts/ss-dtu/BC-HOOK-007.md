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

# BC-HOOK-007: Exactly Five Hook Types Registered; PostToolUse Intentionally Absent

## Description

The Claude Code hook protocol registers exactly five hook types: PreToolUse,
Notification, Stop, SessionStart, and UserPromptSubmit. PostToolUse is explicitly
absent in Phase 1 — Claude Code's own documentation lists it as a potential hook
type, but lazyclaude (the gene source) does not implement it, and monocle follows
the same Phase 1 scope boundary. This is a deliberate design decision, not an
oversight.

## Preconditions

1. The hooks-settings.json file is produced by `WriteHooksSettingsFile`.

## Postconditions

1. The `hooks` object in hooks-settings.json contains exactly these five keys:
   - `"PreToolUse"`
   - `"Notification"`
   - `"Stop"`
   - `"SessionStart"`
   - `"UserPromptSubmit"`
2. No `"PostToolUse"` key is present.
3. No other undocumented keys are present.
4. All five keys are PascalCase — exact spelling as listed.

## Invariants

1. Phase 1 scope: PostToolUse requires both a producer-side hook entry AND a consumer-side HTTP handler. Neither exists in Phase 1.
2. Adding PostToolUse in Phase 2+ requires: (a) a new hook const, (b) a `buildHooksMap` entry, AND (c) a new `/hooks/post-tool-use` endpoint on the monocle daemon.
3. The monocle-canonical paths (`/hooks/pre-tool-use`, etc.) differ from the gene-source lazyclaude paths (`/notify`, `/stop`, etc.) per dtu-assessment.md §Endpoint Matrix.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Claude Code sends a PostToolUse hook invocation (future) | No hook script registered; Claude Code does not invoke the hook; monocle receives no POST |
| EC-002 | Hooks-settings.json parsed by a future Claude Code version that expects 6 hook types | Unknown keys are presumably ignored by Claude Code; existing 5 hooks still fire |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Parse hooks-settings.json `hooks` object | Exactly 5 keys: PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit | lint |
| Check for PostToolUse key | Key absent | lint |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-DTU-001 | Hooks-settings.json produced by DTU clone contains exactly 5 hook types with correct PascalCase keys | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per capabilities.md §CAP-001 — the 5-hook registry is the complete hook protocol surface that the daemon ingests; this BC establishes the authoritative count and names |
| L2 Domain Invariants | DI-001 (tee invariant — the 5-hook scope defines the complete set of events subject to the tee invariant; PostToolUse absence is an explicit scope boundary) |
| Architecture Module | crates/monocle-test-harness/src/dtu/ (DTU clone binary) per dtu-assessment.md §Packaging Decision |
| Architecture Source | dtu-assessment.md v1.7.5 §Endpoint Matrix; semport/any-context-lazyclaude-pass-B-deep-hooks-r1.md §3 (6-vs-5 hook clarification) |
| Gene Source | any-context-lazyclaude/internal/core/config/hooks.go:92-99 (`buildHooksMap()` — exactly 5 entries) |
| Stories | S-DTU-001 |
| Old ID (historical) | BC-HOOK-007 (gene-source: any-context deep-hooks-r1 §3) |
| Test name | test_BC_HOOK_007_exactly_five_hooks_registered |

## Related BCs

- [BC-HOOK-008] — composes with: BC-HOOK-008 covers the hooks-settings.json encoding; this BC defines the key set
- [BC-HOOK-019] — composes with: PreToolUse and Notification both POST to `/notify` (gene-source); monocle uses `/hooks/pre-tool-use` and `/hooks/notification`

## Architecture Anchors

- `specs/dtu-assessment.md#endpoint-matrix` — canonical 5-endpoint matrix with gene-source and monocle-canonical columns

## Story Anchor

S-DTU-001 — Claude Code Hook Protocol DTU Clone

## VP Anchors

- VP-DTU-001 (pending Phase 4 formal verification)

## §Trace v1.0.0

**Phase 3 TDD — BC-HOOK-001..041 initial authorship** (2026-05-20T21:00:00Z):
- Gene-source file:line: hooks.go:92-99 (`buildHooksMap()` — 5-entry map literal).
- Key dtu-assessment.md citation: endpoint matrix, §Clone Development Approach line 150 "5-endpoint matrix".
- Authored for S-DTU-001 DTU clone prerequisite gate.
- SE-16d monotonicity PASS: 2026-05-20T21:00:00Z is initial creation.
