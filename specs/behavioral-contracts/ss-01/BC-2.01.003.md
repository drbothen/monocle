---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T11:30:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "03a845a"
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
| L2 Domain Invariants | N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source |
| Architecture Module | monocle-runtime (daemon binary, HTTP server) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.25 §Body Size Limit |
| Brief Section | §Success Criteria (hook receiver body size limit row — target `{"error":"payload_too_large","limit_bytes":262144}`) |
| Test File | `monocle-runtime/tests/body_size_limit.rs` |
| Test Name | `test_BC_DAEMON_003_body_size_limit_413_on_excess` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-DAEMON-003 |

## Related BCs (Recommended)

- [BC-2.01.001] — composes with: `/healthz` is NOT subject to this limit (unauthenticated router has no body-limit layer)
- [BC-2.01.007] — related to: ring buffer records can approach 256 KiB in size (per BC-RING-001 EC-002); this limit bounds the ingestion path

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#body-size-limit` — axum `DefaultBodyLimit::max(256 * 1024)` layer placement on authenticated router

## Story Anchor (Recommended)

S-TBD — Implement authenticated router with DefaultBodyLimit layer (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-003-body-size-limit.md` — VP-003 body size limit integration tests
