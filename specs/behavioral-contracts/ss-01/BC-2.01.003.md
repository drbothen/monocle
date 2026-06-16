---
document_type: behavioral-contract
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:02:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "17f5b4f"
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

# Behavioral Contract BC-2.01.003: Body Size Limit (256 KiB, HTTP 413)

## Description

The monocle daemon enforces a 256 KiB (262,144 byte) request body size ceiling on all
authenticated endpoints via axum's `DefaultBodyLimit::max(256 * 1024)` layer applied at
router construction time. Requests exceeding the limit receive HTTP 413 with a structured
error body. This bounds worst-case daemon memory exposure per connection to
`concurrent_requests_max × 256KiB`.

## Preconditions

1. The monocle daemon is running.
2. A request arrives at any of the 5 hook POST endpoints (`/hooks/pre-tool-use`, `/hooks/notification`, `/hooks/stop`, `/hooks/session-start`, `/hooks/prompt-submit`) or at `/status` with a request body exceeding 262,144 bytes.

## Postconditions

1. The daemon returns HTTP 413 Payload Too Large with body `{"error":"payload_too_large","limit_bytes":262144}`.
2. The limit is enforced via axum's `DefaultBodyLimit::max(256 * 1024)` layer applied at router construction time on the authenticated router.
3. `/healthz` (unauthenticated, no body) is NOT subject to the limit — it is registered on the unauthenticated router which has no body-limit layer.
4. The limit applies to the request body. Response bodies from `/status` are not bounded by this contract.

## Invariants

1. The 256 KiB ceiling accommodates 5× the 99th-percentile expected payload from Claude Code's `Notification` hook (diff output, stack traces, tool output summaries typically 1–50 KiB).
2. The worst-case daemon memory exposure per connection is bounded to `concurrent_requests_max × 256KiB`.
3. `DefaultBodyLimit::max(256 * 1024)` must be explicitly added — axum 0.8 does NOT apply a default body limit.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-045 | Request body is exactly 262,145 bytes | HTTP 413 (limit is strictly exclusive — `> limit` triggers rejection; body of exactly N=262,144 bytes returns HTTP 200) |
| EC-046 | Request body is 262,143 bytes | HTTP 200 (within limit) |
| EC-047 | `POST /shutdown` (authenticated admin endpoint) with oversized body | Also subject to the body limit (defense-in-depth); shutdown payload is typically empty or a few bytes |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Hook POST with 262,145 byte body | HTTP 413 `{"error":"payload_too_large","limit_bytes":262144}` | error |
| Hook POST with 262,143 byte body | HTTP 200 (hook processed) | edge-case |
| Hook POST with ~1 KiB body | HTTP 200 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-003 | POST to any hook endpoint with body > 262,144 bytes returns HTTP 413 with exact error body | integration |
| VP-003 | POST to any hook endpoint with body = 262,143 bytes returns HTTP 200 | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the memory-protection contract for hook event ingestion, bounding daemon memory exposure during hook processing |
| L2 Domain Invariants | DI-001 (every hook event must be written to the JSONL ring before any acknowledgement is returned — the 256 KiB body size limit protects the ring write path from runaway memory pressure that would cause ring write failures or OOM before the ack is issued); DI-002 (applies to authenticated hook endpoints — the body limit is enforced on the authenticated router, which is the router governed by the DI-002 lock-file auth contract) |
| Architecture Module | monocle-runtime (daemon binary, HTTP server) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.33 §Body Size Limit |
| Brief Section | §Success Criteria (hook receiver body size limit row — target `{"error":"payload_too_large","limit_bytes":262144}`) |
| Test File | `monocle-runtime/tests/body_size_limit.rs` |
| Test Name | `test_BC_DAEMON_003_body_size_limit_413_on_excess` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-DAEMON-003 |

## Related BCs (Recommended)

- [BC-2.01.001] — composes with: `/healthz` is NOT subject to this limit (unauthenticated router has no body-limit layer)
- [BC-2.01.007] — related to: ring buffer records can approach 256 KiB in size (per BC-2.01.007 EC-002); this limit bounds the ingestion path

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#body-size-limit` — axum `DefaultBodyLimit::max(256 * 1024)` layer placement on authenticated router

## Story Anchor (Recommended)

S-TBD — Implement authenticated router with DefaultBodyLimit layer (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-003-body-size-limit.md` — VP-003 body size limit integration tests

## §Trace v1.0.1

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-001 ... ; DI-002 ...`
  - DI-001 mapping: The 256 KiB limit guards the ring write path — without it, oversized payloads could exhaust memory during the ring write that must complete before the ack. DI-002 mapping: The limit is applied to the authenticated router, which is the same router governed by the lock-file auth contract.
- F-R105-9 (SE-17c-d body-scope grep): Related BCs references `[BC-2.01.007]` and `[BC-2.01.001]` — canonical form. Inline prose references `BC-RING-001 EC-002` in a descriptive context — this is an EC reference label, not a stale BC cross-reference (the EC ID EC-002 is defined within BC-2.01.007). 0 stale BC IDs in non-historical body prose. 0 stale VP IDs in body prose. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T11:30:00Z (v1.0).

## §Trace v1.0.2

**F-R106-12 MED — Stale BC-RING-001 EC-002 parenthetical in Related BCs** (2026-05-17T22:40:00Z):
- F-R106-12: Related BCs section contained `(per BC-RING-001 EC-002)`. The prior §Trace v1.0.1 classified this as "an EC reference label, not a stale BC cross-reference" and marked it NO-OP. However, adversary R106 flagged it as a stale old-form BC ID parenthetical at MEDIUM severity. Production-grade resolution: canonicalize to `BC-2.01.007 EC-002` since `BC-RING-001` is the old ID for `BC-2.01.007` per BC-INDEX §Renumbering Map.
- **SE-17f Related BCs before/after:**
  - Before: `[BC-2.01.007] — related to: ring buffer records can approach 256 KiB in size (per BC-RING-001 EC-002); this limit bounds the ingestion path`
  - After: `[BC-2.01.007] — related to: ring buffer records can approach 256 KiB in size (per BC-2.01.007 EC-002); this limit bounds the ingestion path`
  - Rationale: EC-002 is defined within BC-2.01.007 (the renumbered BC-RING-001); the old-form ID BC-RING-001 is renumbering noise in body prose; canonical form `BC-2.01.007 EC-002` is self-documenting and consistent with BC-INDEX. Note: the prior §Trace v1.0.1 observation that "the EC ID EC-002 is defined within BC-2.01.007" confirmed the EC is valid — only the BC prefix needed canonicalization.
- SE-17c-d body-scope grep: `BC-RING-001 EC-002` in Related BCs was the only stale old-form reference in non-historical body prose. §Trace v1.0.1 historical text retains the prior reasoning inline (audit trail). 0 stale VP IDs. 0 other stale BC IDs.
- SE-16d monotonicity PASS: 2026-05-17T22:40:00Z > prior 2026-05-17T18:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R107-2 CRITICAL — Architecture Source pin refresh v1.0.25 → v1.0.30** (2026-05-17T23:30:00Z):
- F-R107-2: Sibling-layer cascade miss from Round 5D (VPs swept but BCs not). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.25 §Body Size Limit`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.30 §Body Size Limit`
  - Canonical version per architect 5E commit 03a4c57 post-R106 closure.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T22:40:00Z (v1.0.2).

## §Trace v1.0.4

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.0.30 → v1.0.32; F-R109-14 MED — §Trace reordered ascending** (2026-05-18T05:02:00Z):
- F-R109-4: Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32 (Round 8A). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.30 §Body Size Limit`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.32 §Body Size Limit`
- F-R109-14: §Trace blocks were descending (v1.0.3, v1.0.2, v1.0.1). Reordered to ascending (v1.0.1 → v1.0.3 → v1.0.4). Content of each section preserved verbatim; only insertion order corrected.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:02:00Z > prior 2026-05-17T23:30:00Z (v1.0.3). ARITHMETICALLY TRUE: 2026-05-18T05:02:00Z > 2026-05-17T23:30:00Z PASS.

## §Trace v1.0.5

**GAP-PHASE2-R06-1 closure — Architecture Source pin SS-daemon-lifecycle v1.0.32 → v1.0.33** (2026-05-19T12:02:00Z):
- GAP-PHASE2-R06-1: architect commit `2d43127` bumped SS-daemon-lifecycle.md v1.0.32 → v1.0.33 (Ring Buffer Rotation Policy added). BC ledger Architecture Source cell was not cascaded in that commit.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.32 §Body Size Limit`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.33 §Body Size Limit`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:02:00Z > prior 2026-05-18T05:02:00Z (v1.0.4). ARITHMETICALLY TRUE: PASS.
