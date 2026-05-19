---
document_type: behavioral-contract
level: L3
version: "1.0.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:06:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "76564f0"
traces_to: prd.md
origin: greenfield
subsystem: SS-01
capability: CAP-001
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

# Behavioral Contract BC-2.01.007: JSONL Ring Format Version (FC-01)

## Description

Every JSONL line written to the monocle ring buffer begins with `"format_version":1` as
its first JSON key, guaranteed by field declaration order in `HookEventRecord` combined
with `serde_json`'s struct-field-order preservation. A module-level const
`RING_FORMAT_VERSION: u32 = 1` in `monocle-runtime::ring` is the single source of truth;
all `HookEventRecord::new(...)` call sites pass this const, not a literal integer. This
forward-compatibility contract (FC-01) enables Phase 2 trigger-trace readers to validate
the format version before deserializing remaining fields.

## Preconditions

1. The monocle daemon is running and has received at least one hook event.
2. `--persistent-events` flag is set OR the drain path is exercised during graceful shutdown.
3. The JSONL ring buffer serializes `HookEventRecord` instances via `serde_json::to_string`.

## Postconditions

1. Every JSONL line written to the ring buffer begins with `{"format_version":1,` — the `format_version` key is the first key in the JSON object.
2. The `format_version` value is always `1` for all Phase 1-origin records. No record written by Phase 1 code has any other value.
3. The module-level const `RING_FORMAT_VERSION: u32 = 1` in `monocle-runtime::ring` is the single source of truth for the format version value. All `HookEventRecord::new(...)` call sites MUST pass `RING_FORMAT_VERSION`, not a literal integer.
4. `HookEventRecord` is defined in `monocle-runtime::ring` (NOT `monocle-core`) with the fields declared in declaration order: `format_version: u32`, `session_id: String`, `timestamp_micros: i64`, `pid: u32`, `hook_type: String`, `tool_name: Option<String>`, `tool_input: Option<serde_json::Value>`.
5. `HookEventRecord` carries `#[non_exhaustive]` and provides `pub fn new(session_id, timestamp_micros, pid, hook_type, tool_name, tool_input) -> Self` constructor.

## Invariants

1. `serde_json` preserves struct field declaration order for plain Rust structs (no `#[serde(rename)]` reordering). `format_version` being the first declared field guarantees it serializes first.
2. `RING_FORMAT_VERSION` is never modified without: bumping the version value, updating `HookEventRecord` field layout documentation, and adding a Phase 2 ingestor capable of reading both versions.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Hook types with no tool context (`SessionStart`, `UserPromptSubmit`, `Stop`) — `tool_name` and `tool_input` are `None` | Serialized record omits these two fields entirely via `#[serde(skip_serializing_if = "Option::is_none")]`; `format_version` is always present; Phase 2 readers MUST tolerate both absence-of-field and explicit-null-field semantically; Phase 1 emitters MUST emit absence (no explicit null) |
| EC-002 | Very large `tool_input` values (up to 256 KiB per BC-2.01.003) | JSONL line may approach 256 KiB in length; ring buffer rotation (50 MB rotation threshold, 100 MB × 5 cap per SS-daemon-lifecycle.md §JSONL Ring Buffer Rotation Policy) must handle lines of this length without truncation |
| EC-003 | Ring buffer file truncated mid-line (e.g., crash during write) | Phase 2 ring readers MUST handle incomplete trailing lines by ignoring them (standard JSONL reader robustness requirement); the `format_version` first-key contract applies only to complete lines |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `HookEventRecord::new(session_id, t, pid, "PreToolUse".into(), Some("Bash".into()), Some(json!({"command":"cargo test"})))` | `serde_json::to_string` result begins with `{"format_version":1,` | happy-path |
| `HookEventRecord::new(session_id, t, pid, "SessionStart".into(), None, None)` | Result begins with `{"format_version":1,`; `tool_name` and `tool_input` fields absent | edge-case |
| `HookEventRecord::new(session_id, t, pid, "Stop".into(), None, None)` | Result begins with `{"format_version":1,`; `tool_name` and `tool_input` fields absent | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-007 | Every `HookEventRecord` serialized via `serde_json::to_string` begins with `{"format_version":1,` | integration |
| VP-007 | `HookEventRecord` for hook types with no tool context omits `tool_name` and `tool_input` fields (not explicit null) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the JSONL ring format versioning for hook event records, which is the persistence layer of hook ingestion |
| L2 Domain Invariants | DI-001 (every hook event must be written to the JSONL ring before any acknowledgement is returned — this BC defines the JSONL ring write format and enforces that every serialized HookEventRecord is committed to the ring on each hook POST, which is the write operation that DI-001 requires complete before ack); DI-004 (all public wire types must carry a version discriminant as their first field — format_version as the first key in every JSONL record directly implements DI-004 for the ring wire format, enabling Phase 2 readers to detect format evolution without parsing the full record) |
| Architecture Module | monocle-runtime (ring buffer) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.33 §Drain |
| Forward Compat Contract | FC-01 (JSONL ring format versioning) |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — JSONL ring format versioning) |
| Test File | `monocle-runtime/tests/jsonl_ring.rs` |
| Test Name | `test_BC_RING_001_format_version_first_key` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-RING-001 |

## Related BCs (Recommended)

- [BC-2.01.003] — related to: ring buffer records can approach 256 KiB (BC-2.01.007 EC-002); BC-2.01.003 governs the ingestion-path body limit
- [BC-2.01.004] — composes with: ring buffer flush occurs during graceful shutdown drain (BC-2.01.004 Postcondition 6)
- [BC-2.01.010] — related to: lock file `contract_version` first-key convention parallels `format_version` first-key convention in the ring

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#drain` — ring buffer flush during graceful shutdown, JSONL persistence path
- `architecture/SS-forward-compatibility.md` — FC-01 contract (JSONL ring format versioning)

## Story Anchor (Recommended)

S-TBD — Implement HookEventRecord with format_version first-key guarantee (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-007-ring-format-version.md` — VP-007 JSONL ring format version integration tests

## §Trace v1.0.6

**GAP-PHASE2-R06-1 closure — Architecture Source pin SS-daemon-lifecycle v1.0.32 → v1.0.33** (2026-05-19T12:06:00Z):
- GAP-PHASE2-R06-1: architect commit `2d43127` bumped SS-daemon-lifecycle.md v1.0.32 → v1.0.33 (Ring Buffer Rotation Policy added). BC ledger Architecture Source cell was not cascaded in that commit.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.32 §Drain`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.33 §Drain`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:06:00Z > prior 2026-05-19T10:00:00Z (v1.0.5). ARITHMETICALLY TRUE: PASS.

## §Trace v1.0.5

**F-PHASE2-R05-05 — EC-002 rotation anchor re-pointed to canonical spec section** (2026-05-19T10:00:00Z):
- NORMATIVE (F-PHASE2-R05-05): EC-002 parenthetical updated.
  - SE-17c BEFORE: `ring buffer rotation (100 MB × 5 files per OQ-06)`
  - SE-17c AFTER: `ring buffer rotation (50 MB rotation threshold, 100 MB × 5 cap per SS-daemon-lifecycle.md §JSONL Ring Buffer Rotation Policy)`
  - Rationale: `PRD §OQ-06` does not exist as a PRD section anchor; OQ-NNN IDs are planning
    open-question IDs from `oq-research.md`, not PRD section references. The canonical rotation
    policy is now defined in `SS-daemon-lifecycle.md §JSONL Ring Buffer Rotation Policy`
    (added in v1.0.33 of that document). The BC's behavioral semantics are unchanged — EC-002
    describes what the rotation policy must tolerate (256 KiB lines); the specific policy
    parameters are an architecture concern, not a BC invariant. The parenthetical reference
    is updated to point to the canonical architecture spec. No behavioral change.
- SE-16d PASS: 2026-05-19T10:00:00Z > chain high-water 2026-05-17T22:50:00Z (monotonic).

## §Trace v1.0.2

**F-R106-12 MED — Stale BC-RING-001 EC-002 self-reference in Related BCs** (2026-05-17T22:50:00Z):
- F-R106-12: Related BCs section contained `(BC-RING-001 EC-002)` as a self-referencing parenthetical. BC-RING-001 is this file's own old ID (per BC-INDEX §Renumbering Map and this file's Traceability "Old ID (historical): BC-RING-001" row). Using the old form in active body prose constitutes a stale ID reference even for self-referential EC citations.
- **SE-17f Related BCs before/after:**
  - Before: `[BC-2.01.003] — related to: ring buffer records can approach 256 KiB (BC-RING-001 EC-002); BC-2.01.003 governs the ingestion-path body limit`
  - After: `[BC-2.01.003] — related to: ring buffer records can approach 256 KiB (BC-2.01.007 EC-002); BC-2.01.003 governs the ingestion-path body limit`
  - Rationale: EC-002 is defined in this file (BC-2.01.007); canonical self-reference is `BC-2.01.007 EC-002`. Applied consistently with the parallel fix in BC-2.01.003.
- SE-17c-d body-scope grep: `BC-RING-001 EC-002` in Related BCs was the only stale old-form reference in non-historical body prose. `BC-RING-001` in the Old ID (historical) Traceability row is preserved (correct; that is historical identity). 0 stale VP IDs. 0 other stale BC IDs.
- SE-16d monotonicity PASS: 2026-05-17T22:50:00Z > prior 2026-05-17T18:00:00Z (v1.0.1).

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-001 ... ; DI-004 ...`
  - DI-001 mapping: This BC governs the ring write itself — every HookEventRecord serialized to JSONL constitutes the ring write that DI-001 requires to complete before ack. DI-004 mapping: format_version as first key is the exact implementation of DI-004's "version discriminant as first field" requirement for the ring wire type.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T11:30:00Z (v1.0).

## §Trace v1.0.3

**F-R107-2 CRITICAL — Architecture Source pin refresh v1.0.25 → v1.0.30** (2026-05-17T23:30:00Z):
- F-R107-2: Sibling-layer cascade miss from Round 5D (VPs swept but BCs not). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.25 §Drain`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.30 §Drain`
  - Canonical version per architect 5E commit 03a4c57 post-R106 closure.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T22:50:00Z (v1.0.2).

## §Trace v1.0.4

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.0.30 → v1.0.32; F-R109-14 MED — §Trace reordered ascending** (2026-05-18T05:06:00Z):
- F-R109-4: Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32 (Round 8A). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.30 §Drain`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.32 §Drain`
- F-R109-14: §Trace blocks were descending (v1.0.3, v1.0.2, v1.0.1). Reordered to ascending (v1.0.1 → v1.0.3 → v1.0.4). Content of each section preserved verbatim; only insertion order corrected.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:06:00Z > prior 2026-05-17T23:30:00Z (v1.0.3). ARITHMETICALLY TRUE: 2026-05-18T05:06:00Z > 2026-05-17T23:30:00Z PASS.
