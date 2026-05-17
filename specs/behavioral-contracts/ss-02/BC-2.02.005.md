---
document_type: behavioral-contract
level: L3
version: "1.0.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T05:14:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "8eabccc"
traces_to: prd.md
origin: greenfield
subsystem: SS-02
capability: CAP-002
# Lifecycle fields (DF-030)
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

# Behavioral Contract BC-2.02.005: VsddFactoryAdapter Implementation

## Description

`VsddFactoryAdapter` is the Phase 1 implementation of `FactoryAdapter` that detects and
reads VSDD factory project state from `.factory/STATE.md`. Detection is triggered by
`document_type: pipeline-state` in the file's YAML frontmatter. Absent optional fields
(`current_cycle`, convergence checkpoint) produce `None` — never the string `"unknown"`.
The `parse_frontmatter_field` function applies four guards to handle YAML edge cases.

## Preconditions

1. `monocle-core` compiles.
2. `VsddFactoryAdapter` implements `FactoryAdapter`.

## Postconditions

1. `VsddFactoryAdapter::new(workspace_root: PathBuf) -> Self` is a public constructor. It derives `state_file = workspace_root.join(".factory").join("STATE.md")`. No validation is performed at construction time — validation is deferred to `detect()` and `read_state()`.
2. `VsddFactoryAdapter::detect(workspace_root)` returns `Some(FactoryDetection)` when called against monocle's own workspace root — the directory containing `.factory/STATE.md` with `document_type: pipeline-state` in YAML frontmatter.
3. `read_state()` returns `None` for absent optional fields: absent `current_cycle:` → `cycle: None`; absent §Session Resume Checkpoint → `convergence: None`. Consumers MUST NOT receive `"unknown"` as a placeholder for absent optional fields.
4. `parse_frontmatter_field` and `parse_frontmatter_extra_fields` apply these guards: skip continuation lines (leading whitespace); return `None` for empty values; return `None` for flow-style list values (beginning with `[`); return `None` for block scalar markers (beginning with `|` or `>`). YAML quoted scalars are unquoted (single and double quotes stripped).

## Invariants

1. The detection criterion is `document_type: pipeline-state` in the YAML frontmatter of `.factory/STATE.md`. No other file or field is required for detection.
2. `display_name()` returns `"VSDD Factory"` — the exact string used in TUI display.
3. `subscribe()` returns `Ok(Box::pin(futures::stream::empty()))` in Phase 1.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-021 | STATE.md file with `document_type: pipeline-state` in the document body (not frontmatter) | `parse_frontmatter_field` checks that the document begins with `---` on the FIRST line before scanning for the key; body occurrences are not detected |
| EC-022 | STATE.md file with `awaiting: "round 18 validation chain"` (YAML double-quoted value) | `parse_frontmatter_field` strips surrounding double quotes and returns `Some("round 18 validation chain")` |
| EC-023 | STATE.md file with `blocking_issues: []` (YAML flow-style list) | `parse_frontmatter_extra_fields` skips this; `blocking_issues` in `FactoryState` is populated by Phase 3 body parsing; Phase 1 always returns `Vec::new()` |
| EC-061 | STATE.md frontmatter with `current_cycle: ""` (present-but-empty quoted value) | Parses to `state.cycle == None`, NOT `Some("".into())`; the empty-value guard returns `None` regardless of key presence or absence |

## Canonical Test Vectors

| Scenario | Input | Expected Output | Category |
|----------|-------|----------------|----------|
| Self-referential detection | `VsddFactoryAdapter::detect(monocle_repo_root)` | `Some(FactoryDetection { display_name: "VSDD Factory" })` | happy-path |
| read_state — absent current_cycle | STATE.md with no `current_cycle:` key | `FactoryState { cycle: None, ... }` | edge-case |
| read_state — present cycle | STATE.md with `current_cycle: "cycle-001"` | `FactoryState { cycle: Some("cycle-001"), ... }` | happy-path |
| read_state — quoted awaiting | STATE.md with `awaiting: "human GO"` | `FactoryState { awaiting: Some("human GO"), ... }` | edge-case |
| Nonexistent workspace | `VsddFactoryAdapter::detect("/tmp/not-a-factory")` | `None` | edge-case |
| Present-but-empty cycle | STATE.md with `current_cycle: ""` | `FactoryState { cycle: None, ... }` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-015 | `VsddFactoryAdapter::detect(monocle_repo_root)` returns `Some(_)` with `display_name == "VSDD Factory"`; `read_state()` returns `cycle` as `None` or `Some(_)`, never `"unknown"` | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability §SS-02 |
| Capability Anchor Justification | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability — this BC implements the factory-state abstraction component of CAP-002 for the VSDD factory format |
| L2 Domain Invariants | DI-007 (monocle must not write to any file owned by a harness or factory workflow system — VsddFactoryAdapter reads .factory/STATE.md via detect() and read_state() and must never write to it; Invariant 1 (detection criterion) and Invariant 3 (subscribe returns empty stream) enforce that no write path exists in Phase 1; the observe-only constraint is a core invariant of this implementation) |
| Architecture Module | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) per ARCH-INDEX Subsystem Registry SS-02 |
| Architecture Source | SS-core-types-and-abi.md v1.2.13 §FactoryAdapter Trait §Phase 1 Implementation: VsddFactoryAdapter |
| Brief Section | §Success Criteria (factory pattern detection row — "Detection succeeds on monocle's own `.factory/`") |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-FACTORY-002 |
| Test name | test_BC_FACTORY_002_vsdd_adapter_self_referential_detection |

## Related BCs (Recommended)

- [BC-2.02.004] — depends on: VsddFactoryAdapter implements the FactoryAdapter trait defined here
- [BC-2.01.002] — composes with: /status endpoint consumes FactoryState output from this adapter

## Architecture Anchors (Recommended)

- `architecture/SS-core-types-and-abi.md#factoryadapter-trait` — §Phase 1 Implementation: VsddFactoryAdapter, frontmatter parser guards, self-referential detection criterion

## Story Anchor (Recommended)

S-TBD — Implement VsddFactoryAdapter with frontmatter parser and self-referential detection (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-015-vsdd-factory-adapter.md` — VP-015 VsddFactoryAdapter self-referential integration test

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-002 per ARCH-INDEX is authoritative source`
  - After: `DI-007 ...`
  - DI-007 mapping: VsddFactoryAdapter is the concrete observe-only reader of .factory/STATE.md. It calls no write methods. Invariant 3 (subscribe returns empty stream) ensures no live watcher writes are triggered. This is the primary DI-007 enforcer for the factory state abstraction layer.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T12:00:00Z (v1.0).

## §Trace v1.0.2

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.2.8 → v1.2.13** (2026-05-18T05:14:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.2.8 → v1.2.13); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait §Phase 1 Implementation: VsddFactoryAdapter`
  - SE-17f AFTER: `SS-core-types-and-abi.md v1.2.13 §FactoryAdapter Trait §Phase 1 Implementation: VsddFactoryAdapter`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:14:00Z > prior 2026-05-17T18:00:00Z (v1.0.1). ARITHMETICALLY TRUE: 2026-05-18T05:14:00Z > 2026-05-17T18:00:00Z PASS.
