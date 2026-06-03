---
document_type: story
level: L4
story_id: S-008
epic_id: EPIC-01
version: "1.4"
status: done
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
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.007.md, version: "1.0.6"}
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

Test oracle: `assert!(serde_json::to_string(&record).unwrap().starts_with(r#"{"format_version":1,"#))`
This verbatim assertion must appear in the test body for the AC-001 test case.

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

Cross-reference: `SS-core-types-and-abi.md v1.2.13 §HookEventRecord` is the canonical struct
declaration source. Note that `timestamp_micros` is typed `i64` (signed) per that section —
verify signedness matches BC-INDEX field declaration before writing the struct.

### AC-003 (traces to SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer L539 — post-batch flush; BC-2.01.007 postcondition 2 + postcondition 3 for RING_FORMAT_VERSION const usage)
Each record is written via `tempfile::persist` for atomic durability; flush after every batch
per SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer (line 694). No in-RAM VecDeque ring with
capacity-threshold triggering exists in the canonical architecture — the flush model is
post-batch `tempfile::persist` only. Any RAM-ring hybrid model would require an architect ADR
before introduction and is out of scope for this story.

### AC-004 (traces to BC-2.01.007 postcondition 4 — tee invariant DI-001)
Every hook event received by the daemon is written to the JSONL ring BEFORE any
acknowledgement (HTTP 200 response) is returned to the harness. DI-001 enforcement.

Ordering oracle: Test asserts ordering via `mpsc::channel` probe in test harness:
`ring.push(record).await` MUST complete BEFORE `Response::new(200)` is constructed
in the hook handler. The channel probe verifies sequencing — ring write signal received
before response construction signal.

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

### AC-007 (traces to SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer Rotation Policy L675-719 + BC-2.01.007 v1.0.6 edge case EC-002 — ring rotation policy)
The JSONL ring file is rotated when it exceeds the configured size limit. Parameters per
SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer Rotation Policy (canonical source of truth,
lines 675-719): default rotation threshold is 50 MB per active file (soft trigger, checked on
each flush batch); absolute per-file cap is 100 MB (hard upper bound, rotation mandatory);
retention is 5 rotated files (cascade: `.1` newest through `.5` oldest, oldest deleted first
per L691); total disk ceiling is 500 MB rotated + up to 100 MB active = 600 MB worst-case.
No ring file exceeds 100 MB at any point.
EC-002 (very large tool_input up to 256 KiB, per BC-2.01.007 v1.0.6 EC-002 re-anchored to
this rotation policy section) requires the rotation logic to handle lines approaching 256 KiB
without truncation. (BC-2.01.007 INV-1 governs `serde_json` struct-field-order preservation —
it is NOT the ring rotation clause. The canonical rotation policy lives in
SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer Rotation Policy L675-719, not in PRD §OQ-06.)

Testability: Test harness injects `RotationConfig { soft_threshold_bytes: 1024, hard_cap_bytes: 4096, retained: 5 }`;
assert rotation behavior via `metadata().len()` after each flush — match the architectural pattern
at SS-daemon-lifecycle.md L694.

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
  - Atomic flush via `tempfile::persist` after each batch (SS-daemon-lifecycle.md L694)
  - No in-RAM VecDeque capacity threshold; flush is post-batch, not timer-triggered
- [ ] Hook handlers write to ring BEFORE returning HTTP 200 (DI-001)
- [ ] Rotation policy (SS-daemon-lifecycle.md L675-719): when `monocle-events.jsonl` exceeds 50 MiB,
  rotate using .1...5 cascade: `monocle-events.jsonl.4 → monocle-events.jsonl.5` (drop oldest),
  `monocle-events.jsonl.3 → monocle-events.jsonl.4`, ...,
  `monocle-events.jsonl → monocle-events.jsonl.1`; then start fresh `monocle-events.jsonl`
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
The JSONL ring file lives at `<runtime_dir>/monocle-events.jsonl` — same runtime dir pattern.
S-009 (Wave 3) CONSUMES `RingBuffer::push()` from this story (Decision 1: S-008 → S-009 edge).
S-008 is the PRODUCER of the RingBuffer push API; S-009 wires hook handlers to call into it.
S-008 MUST deliver the full `RingBuffer` API surface that S-009 depends on — no stubbing of
the push() method. This story is a hard blocker for S-009; the Wave 3 gate requires S-008
green before S-009 can be dispatched.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.33 §JSONL Ring Buffer (L539 flush location, L675-719 Rotation Policy):
- `format_version` MUST be first key — use ordered struct, not HashMap
- Post-batch atomic flush via `tempfile::persist` (L694) — NEVER synchronous disk write on hook response path
- Tee invariant (DI-001): ring write before HTTP 200

From `architecture/SS-forward-compatibility.md` v1.2.19 §FC-01:
- `format_version` field placement at position 0 is the FC-01 forward-compatibility contract — readers MUST check this field before parsing remaining fields to enable non-breaking format evolution

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
| tokio | =1.52 | Async runtime for hook handler response path |
| serde | 1 | Serialize derive for HookEventRecord |
| serde_json | =1.0.149 | JSONL line serialization |
| tempfile | 3 | `tempfile::persist` for atomic flush (CLAUDE.md §Atomic writes) |
| tracing | 0.1 | WARN on flush failure (E-RING-001) |

## File Structure Requirements

Files to create:
- `monocle-runtime/src/ring.rs` — `HookEventRecord`, `RingBuffer`, async flush logic
- `monocle-runtime/tests/jsonl_ring.rs` — integration tests

Files to modify:
- `monocle-runtime/src/lib.rs` — add `pub mod ring;`

## Downstream Consumer Surface

Public API surface produced by this story for consumption by S-009:

```
RingBuffer::new(config: RotationConfig) -> Self
RingBuffer::push(record: HookEventRecord) -> Result<(), RingError>
HookEventRecord::new(session_id: String, timestamp_micros: i64, pid: u32, hook_type: String, tool_name: Option<String>, tool_input: Option<serde_json::Value>) -> Self
RING_FORMAT_VERSION: u32  // const
RotationConfig  // struct (fields: soft_threshold_bytes: u64, hard_cap_bytes: u64, retained: usize)
RingError       // enum
```

S-009 (auth token header validation) wires hook handler POST routes to call
`RingBuffer::push()` before returning HTTP 200 (DI-001 enforcement). This API surface
must be complete and non-stubbed before S-009 can be dispatched.

## §Trace v1.4

**Phase 3.B Batch 4 spec-reviewer remediation** (2026-05-20):
- F-D-01 (CRITICAL) closed: Ring filename corrected throughout — `monocle-ring.jsonl` → canonical `monocle-events.jsonl` (SS-daemon-lifecycle L539, L691, L699-708; error-taxonomy; BC-2.01.004 EC-049).
- F-D-02 (CRITICAL) closed: Fabricated 80%-capacity/5-second-timer hybrid-flush trigger DELETED from AC-003 and Tasks; aligned to canonical post-batch `tempfile::persist` (SS-daemon-lifecycle L694); note that a RAM-ring hybrid model requires an architect ADR and is out of scope.
- F-D-03 (MED) closed: Rotation cascade corrected from `.bak` to `.1...5` cascade per SS-daemon-lifecycle L691.
- F-D-04 (MED) closed: AC-002b cross-reference to SS-core-types-and-abi.md v1.2.13 §HookEventRecord added with `timestamp_micros: i64` signedness note.
- F-A-02 (MED) closed: `tempfile = "3"` added to Library & Framework Requirements table.
- F-B-01 (HIGH) closed: SS-daemon-lifecycle line anchors added to AC-003 (L539) and AC-007 (L675-719).
- F-B-02 (MED) closed: SS-forward-compatibility.md v1.2.19 §FC-01 reference added to Architecture Compliance Rules.
- F-C-01 (HIGH) closed: AC-001 oracle explicit — `serde_json::to_string(&record).unwrap().starts_with(r#"{"format_version":1,"#)` verbatim assertion.
- F-C-02 (HIGH) closed: AC-007 testability hook — `RotationConfig { soft_threshold_bytes: 1024, hard_cap_bytes: 4096, retained: 5 }` injection + `metadata().len()` assertion.
- F-C-03 (MED) closed: AC-004 ordering oracle — `mpsc::channel` probe verifying ring write before `Response::new(200)`.
- F-E-02 (MED) closed: Downstream Consumer Surface section added for S-009 (RingBuffer::new, push, HookEventRecord::new, RING_FORMAT_VERSION, RotationConfig, RingError).
- tokio row in Library table updated: removed "interval timer" wording (no timer-based flush in canonical model).
- version bumped 1.3 → 1.4.
