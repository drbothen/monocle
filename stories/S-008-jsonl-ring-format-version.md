---
document_type: story
story_id: S-008
epic_id: EPIC-01
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 3
tdd_mode: strict
priority: P0
depends_on: [S-006]
blocks: []
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.007]
verification_properties: [VP-007]
estimated_days: 2
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

### AC-002 (traces to BC-2.01.007 postcondition 2 — 7-field canonical schema)
Each record contains exactly these 7 fields in order:
`format_version, session_id, timestamp_micros, pid, hook_type, tool_name, tool_input`.
`tool_name` and `tool_input` are `null` for hook types that do not carry tool context.

### AC-003 (traces to BC-2.01.007 postcondition 3 — ring is hybrid RAM + async flush)
The ring buffer maintains records in RAM up to the configured capacity limit. Async flush
to `<runtime_dir>/monocle-ring.jsonl` is triggered when the RAM buffer reaches 80%
capacity or on a 5-second timer (whichever comes first).

### AC-004 (traces to BC-2.01.007 postcondition 4 — tee invariant DI-001)
Every hook event received by the daemon is written to the JSONL ring BEFORE any
acknowledgement (HTTP 200 response) is returned to the harness. DI-001 enforcement.

### AC-005 (traces to BC-2.01.007 edge case EC-003 — flush failure is degraded, not broken)
If the async flush to disk fails (e.g., disk full), the daemon logs
`WARN: ring buffer flush failed: <io-error>` (E-RING-001) but continues accepting hook
events into the RAM ring. No hook acknowledgement is withheld due to flush failure.

### AC-006 (traces to BC-2.01.007 invariant 1 — ring rotation policy)
The JSONL ring file is rotated when it exceeds the configured size limit (default 50 MB).
Old records beyond the limit are discarded (newest-wins rotation). No ring file exceeds
2× the size limit at any point.

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
  - Fields in order: `format_version: u32`, `session_id: String`, `timestamp_micros: i64`,
    `pid: u32`, `hook_type: String`, `tool_name: Option<String>`, `tool_input: Option<serde_json::Value>`
  - Use `indexmap::IndexMap` or manual struct ordering to guarantee `format_version` is first
- [ ] Implement `RingBuffer` struct in `monocle-runtime/src/ring.rs`
  - RAM buffer: `VecDeque<HookEventRecord>` with capacity limit
  - Async flush task via `tokio::spawn` background task
  - Flush trigger: 80% capacity OR 5-second timer (`tokio::time::interval`)
- [ ] Hook handlers write to ring BEFORE returning HTTP 200 (DI-001)
- [ ] Rotation policy: when `monocle-ring.jsonl` > 50 MB, rotate (rename to `.jsonl.bak`, start fresh)
- [ ] E-RING-001 error handling: log WARN on flush failure, continue accepting events
- [ ] Integration tests `monocle-runtime/tests/jsonl_ring.rs`:
  - Record written → `format_version` is first key in JSON output
  - All 7 fields present; `tool_name`/`tool_input` null for non-tool events
  - DI-001: hook POST response returns AFTER ring write
  - Flush failure → WARN log + ring continues accepting events
  - Rotation at 50 MB threshold

## Previous Story Intelligence

S-006 (Wave 2): `runtime_dir` resolution established. `<runtime_dir>` path available.
The JSONL ring file lives at `<runtime_dir>/monocle-ring.jsonl` — same runtime dir pattern.
S-009 (Wave 2) will add hook handlers that call into `RingBuffer::push()` — this story
stubs those hooks or provides the ring API that S-009 will call into.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.32 §JSONL Ring Buffer:
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
