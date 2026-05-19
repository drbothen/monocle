---
document_type: story
story_id: S-008
epic_id: EPIC-01
version: "1.3"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 3
tdd_mode: strict
priority: P0
depends_on: [S-006]
blocks: [S-009]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.007]
verification_properties: [VP-007]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.12"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.007.md, version: "1.0.5"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-007-ring-format-version.md, version: "1.0.14"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.01.007 (JSONL Ring Format Version FC-01); verifies VP-007; covers EC-001, EC-002, EC-003; addresses DI-001, DI-004."
---

# S-008: JSONL Ring Format Version (FC-01)

## Narrative

As a Phase 2 trigger-trace consumer, I want each JSONL hook event record written to the
ring buffer to have `format_version` as its first JSON key, so that future readers can
detect format evolution and skip or upgrade records without parsing the full record.

## Acceptance Criteria

### AC-001 (traces to BC-2.01.007 postcondition 1 — format_version is first key)
Every `HookEventRecord` written to the JSONL ring has `format_version: 1` as its first
JSON key. The key order is enforced in serialization — not relying on HashMap ordering.

### AC-002 (traces to BC-2.01.007 postcondition 4 + EC-001 — field omission for absent tool context; NOT null)
`HookEventRecord` fields `tool_name: Option<String>` and `tool_input: Option<serde_json::Value>` carry
`#[serde(skip_serializing_if = "Option::is_none")]`. For hook types with no tool context
(`SessionStart`, `UserPromptSubmit`, `Stop`): serialized record OMITS `tool_name` and `tool_input`
fields entirely — NOT emitting `null` values (BC-2.01.007 EC-001 verbatim: "Phase 1 emitters MUST
emit absence (no explicit null)"). Phase 2 readers MUST tolerate both absence-of-field and
explicit-null-field semantically. VP-007 verifies that `HookEventRecord::new(session_id, t, pid,
"SessionStart".into(), None, None)` serializes without `tool_name` or `tool_input` keys.

### AC-002b (traces to BC-2.01.007 postcondition 4 — 7-field canonical declaration order)
`HookEventRecord` struct declares fields in this exact order:
`format_version: u32`, `session_id: String`, `timestamp_micros: i64`, `pid: u32`,
`hook_type: String`, `tool_name: Option<String>`, `tool_input: Option<serde_json::Value>`
(BC-2.01.007 PC-4). This declaration order ensures `format_version` serializes first via
`serde_json`'s struct-field-order preservation (BC-2.01.007 invariant 1).

### AC-003 (traces to SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer — ring is hybrid RAM + async flush; BC-2.01.007 postcondition 2 + postcondition 3 for RING_FORMAT_VERSION const usage)
The ring buffer maintains records in RAM up to the configured capacity limit. Async flush
to `<runtime_dir>/monocle-ring.jsonl` is triggered when the RAM buffer reaches 80%
capacity or on a 5-second timer (whichever comes first). The hybrid-RAM-async-flush
architecture is specified in SS-daemon-lifecycle.md §JSONL Ring Buffer (not in BC-2.01.007
which only governs format_version placement and RING_FORMAT_VERSION const; no BC clause
covers the hybrid-ring capacity threshold — this is an architectural constraint).

### AC-004 (traces to BC-2.01.007 postcondition 4 — tee invariant DI-001)
Every hook event received by the daemon is written to the JSONL ring BEFORE any
acknowledgement (HTTP 200 response) is returned to the harness. DI-001 enforcement.

### AC-005 (traces to BC-2.01.004 edge case EC-049 — flush failure is degraded, not broken)
If the async flush to disk fails (e.g., disk full), the daemon logs
`WARN: ring buffer flush failed: <io-error>` (E-RING-001) but continues accepting hook
events into the RAM ring. No hook acknowledgement is withheld due to flush failure.
(BC-2.01.004 EC-049 "Ring buffer flush fails during drain" is the canonical EC locus.
BC-2.01.007 EC-003 is "ring buffer file truncated mid-line (crash)" — a reader-side
robustness concern, not the flush-failure writer concern. Re-anchored from EC-003 → EC-049.)

### AC-006 (traces to BC-2.01.007 postcondition 5 — #[non_exhaustive] + pub fn new() constructor)
`HookEventRecord` carries `#[non_exhaustive]` attribute AND provides a public constructor:
`pub fn new(session_id: String, timestamp_micros: i64, pid: u32, hook_type: String, tool_name: Option<String>, tool_input: Option<serde_json::Value>) -> Self`
(BC-2.01.007 PC-5 verbatim). External callers MUST construct `HookEventRecord` via `new()` —
struct literal construction outside `monocle-runtime::ring` is forbidden by `#[non_exhaustive]`
(Rust E0639). The `format_version` field is set to `RING_FORMAT_VERSION` inside the constructor.

### AC-007 (traces to SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer Rotation Policy + BC-2.01.007 v1.0.5 edge case EC-002 — ring rotation policy)
The JSONL ring file is rotated when it exceeds the configured size limit. Parameters per
SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer Rotation Policy (canonical source of truth):
default rotation threshold is 50 MB per active file (soft trigger, checked on each flush batch);
absolute per-file cap is 100 MB (hard upper bound, rotation mandatory); retention is 5 rotated
files; total disk ceiling is 500 MB rotated + up to 100 MB active = 600 MB worst-case;
newest-wins rotation (oldest deleted first). No ring file exceeds 100 MB at any point.
EC-002 (very large tool_input up to 256 KiB, per BC-2.01.007 v1.0.5 EC-002 re-anchored to
this rotation policy section) requires the rotation logic to handle lines approaching 256 KiB
without truncation. (BC-2.01.007 INV-1 governs `serde_json` struct-field-order preservation —
it is NOT the ring rotation clause. The canonical rotation policy lives in
SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer Rotation Policy, not in PRD §OQ-06.)

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~800 |
| BC-2.01.007.md | ~700 |
| VP-007 file | ~500 |
| SS-daemon-lifecycle.md (ring buffer section, ~100 lines) | ~1,500 |
| SS-core-types-and-abi.md (HookEventRecord struct) | ~500 |
| Test file | ~700 |
| **Total estimate** | **~4,700** |

## Tasks

- [ ] Define `HookEventRecord` struct in `monocle-runtime/src/ring.rs` with `serde::Serialize`
  - Apply `#[non_exhaustive]` to `HookEventRecord` (BC-2.01.007 PC-5)
  - Fields in declaration order: `format_version: u32`, `session_id: String`, `timestamp_micros: i64`,
    `pid: u32`, `hook_type: String`, `tool_name: Option<String>`, `tool_input: Option<serde_json::Value>`
  - `tool_name` and `tool_input` fields: add `#[serde(skip_serializing_if = "Option::is_none")]` (EC-001)
  - Implement `pub fn new(session_id, timestamp_micros, pid, hook_type, tool_name, tool_input) -> Self`
    that sets `format_version: RING_FORMAT_VERSION` internally (BC-2.01.007 PC-5)
  - Struct field ordering (not HashMap) guarantees `format_version` first via serde_json (invariant 1)
- [ ] Implement `RingBuffer` struct in `monocle-runtime/src/ring.rs`
  - RAM buffer: `VecDeque<HookEventRecord>` with capacity limit
  - Async flush task via `tokio::spawn` background task
  - Flush trigger: 80% capacity OR 5-second timer (`tokio::time::interval`)
- [ ] Hook handlers write to ring BEFORE returning HTTP 200 (DI-001)
- [ ] Rotation policy: when `monocle-ring.jsonl` > 50 MB, rotate (rename to `.jsonl.bak`, start fresh)
- [ ] E-RING-001 error handling: log WARN on flush failure, continue accepting events
- [ ] Integration tests `monocle-runtime/tests/jsonl_ring.rs` (test name: `test_BC_RING_001_format_version_first_key`):
  - Record written → `format_version` is first key in JSON output (VP-007)
  - `SessionStart` record → `tool_name` and `tool_input` fields ABSENT (not null) in JSON output (EC-001)
  - `PreToolUse` record → both `tool_name` and `tool_input` present (non-None path)
  - DI-001: hook POST response returns AFTER ring write (AC-004)
  - Flush failure → WARN log E-RING-001 + ring continues accepting events (AC-005)
  - Rotation at 50 MB threshold (AC-007)
  - `HookEventRecord::new()` is the only legal construction path (AC-006; #[non_exhaustive] enforces at compile time)

## Previous Story Intelligence

S-006 (Wave 2): `runtime_dir` resolution established. `<runtime_dir>` path available.
The JSONL ring file lives at `<runtime_dir>/monocle-ring.jsonl` — same runtime dir pattern.
S-009 (Wave 3) CONSUMES `RingBuffer::push()` from this story (Decision 1: S-008 → S-009 edge).
S-008 is the PRODUCER of the RingBuffer push API; S-009 wires hook handlers to call into it.
S-008 MUST deliver the full `RingBuffer` API surface that S-009 depends on — no stubbing of
the push() method. This story is a hard blocker for S-009; the Wave 3 gate requires S-008
green before S-009 can be dispatched.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.33 §JSONL Ring Buffer:
- `format_version` MUST be first key — use ordered struct, not HashMap
- Async flush — NEVER synchronous disk write on hook response path
- Tee invariant (DI-001): ring write before HTTP 200

From `architecture/SS-core-types-and-abi.md` v1.2.13 §HookEventRecord:
- `HookEventRecord` is the canonical 7-field struct
- `#[derive(serde::Serialize)]` with explicit field ordering

**Forbidden Dependencies:**
- `monocle-runtime/src/ring.rs` MUST NOT import from `monocle-tui`
- Synchronous disk I/O on the hook event response path is FORBIDDEN
- `HashMap` for record serialization is FORBIDDEN (field order not guaranteed)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| tokio | =1.52 | Async flush task, interval timer |
| serde | 1 | Serialize derive for HookEventRecord |
| serde_json | =1.0.149 | JSONL line serialization |
| tracing | 0.1 | WARN on flush failure (E-RING-001) |

## File Structure Requirements

Files to create:
- `monocle-runtime/src/ring.rs` — `HookEventRecord`, `RingBuffer`, async flush logic
- `monocle-runtime/tests/jsonl_ring.rs` — integration tests

Files to modify:
- `monocle-runtime/src/lib.rs` — add `pub mod ring;`
