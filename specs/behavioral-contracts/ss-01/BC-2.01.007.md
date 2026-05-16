---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T11:30:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
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
| EC-002 | Very large `tool_input` values (up to 256 KiB per BC-2.01.003) | JSONL line may approach 256 KiB in length; ring buffer rotation (100 MB × 5 files per OQ-06) must handle lines of this length without truncation |
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
| L2 Domain Invariants | N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source |
| Architecture Module | monocle-runtime (ring buffer) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.25 §Drain |
| Forward Compat Contract | FC-01 (JSONL ring format versioning) |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — JSONL ring format versioning) |
| Test File | `monocle-runtime/tests/jsonl_ring.rs` |
| Test Name | `test_BC_RING_001_format_version_first_key` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-RING-001 |

## Related BCs (Recommended)

- [BC-2.01.003] — related to: ring buffer records can approach 256 KiB (BC-RING-001 EC-002); BC-2.01.003 governs the ingestion-path body limit
- [BC-2.01.004] — composes with: ring buffer flush occurs during graceful shutdown drain (BC-2.01.004 Postcondition 6)
- [BC-2.01.010] — related to: lock file `contract_version` first-key convention parallels `format_version` first-key convention in the ring

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#drain` — ring buffer flush during graceful shutdown, JSONL persistence path
- `architecture/SS-forward-compatibility.md` — FC-01 contract (JSONL ring format versioning)

## Story Anchor (Recommended)

S-TBD — Implement HookEventRecord with format_version first-key guarantee (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-007-ring-format-version.md` — VP-007 JSONL ring format version integration tests
