---
document_type: story-uncertainty-assessment
story_id: S-008
story_version: "1.3"
story_title: JSONL Ring Format Version (FC-01)
assessment_batch: BATCH-3
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: PASS_WITH_OBSERVATIONS
---

# Story Assessment: S-008

## Verdict

**PASS_WITH_OBSERVATIONS** — Story is implementable as written. Two MEDIUM and one LOW
observation. None block TDD start given the story's well-specified ring architecture.

## Summary

S-008 is the JSONL ring buffer story. The `HookEventRecord` struct specification is precise
(7 fields in declaration order, `#[non_exhaustive]`, `pub fn new()` constructor, `format_version: RING_FORMAT_VERSION` set internally). The hybrid RAM + async flush architecture is well-described. The AC-007 rotation policy citation is correctly re-anchored from BC-2.01.007 INV-1 to SS-daemon-lifecycle.md §JSONL Ring Buffer Rotation Policy. Two medium observations on the ring buffer capacity and `RING_FORMAT_VERSION` const placement.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | `tokio =1.52`, `serde =1`, `serde_json =1.0.149`, `tracing 0.1` all correctly specified. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S008-D2-01 | MEDIUM | AC-003 specifies a "configured capacity limit" for the RAM ring buffer but does not define the default capacity. SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer presumably specifies this default, but the story does not surface it. The implementer will need to read SS-daemon-lifecycle.md to find the value. The capacity should be stated in the story (e.g., "default 1000 records per RAM ring, configurable via `MONOCLE_RING_CAPACITY` env var"). |
| S008-D2-02 | MEDIUM | The `RING_FORMAT_VERSION` constant is referenced in AC-006 and AC-007 as the value set in `pub fn new()`. But the story does not declare WHERE this constant is defined — in `monocle-runtime/src/ring.rs` (private), in `monocle-core` (public), or in `monocle-runtime/src/lib.rs`. The constant must be accessible to the `pub fn new()` constructor. Given that `HookEventRecord` is in `monocle-runtime`, the const should be in `ring.rs` as `const RING_FORMAT_VERSION: u32 = 1;`. This should be explicit. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S008-D3-01 | LOW | S-008 must deliver the full `RingBuffer::push()` API before S-009 can be dispatched (per "S-008 is a hard blocker for S-009"). The push() method signature is not defined in the story. What are its parameters? Presumably `fn push(&self, record: HookEventRecord)` or `async fn push(&self, record: HookEventRecord)`. The test-writer needs this signature to write the failing test. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Test coverage is complete. The explicit test function name `test_BC_RING_001_format_version_first_key` is commendable. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter complete. inputs versioned correctly. |

## Research Queue

None.

## Recommended Fixes

1. S008-D2-01 (MEDIUM): Add explicit default RAM ring capacity value (read from SS-daemon-lifecycle.md). Routing: story-writer.
2. S008-D2-02 (MEDIUM): Specify `const RING_FORMAT_VERSION: u32 = 1;` declared in `monocle-runtime/src/ring.rs` and add to File Structure Requirements or Architecture Compliance Rules. Routing: story-writer.
3. S008-D3-01 (LOW): Add `RingBuffer::push()` method signature to File Structure Requirements or Tasks. Routing: story-writer.
