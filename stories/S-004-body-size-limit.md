---
document_type: story
story_id: S-004
epic_id: EPIC-01
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 2
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001]
blocks: [S-009]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.003]
verification_properties: [VP-003]
estimated_days: 1
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.11"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.003.md, version: "1.0.4"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-003-body-size-limit.md, version: "1.0.14"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.10"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.32"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.01.003 (Body Size Limit); verifies VP-003; covers EC-002; addresses NFR-005, E-DAEMON-001."
---

# S-004: Body Size Limit (256 KiB, HTTP 413)

## Narrative

As a daemon operator, I want all authenticated POST endpoints to reject request bodies
exceeding 256 KiB with HTTP 413, so that the daemon's memory exposure is bounded and
a misbehaving or adversarial hook client cannot cause unbounded RAM consumption.

## Acceptance Criteria

### AC-001 (traces to BC-2.01.003 postcondition 1)
A POST to any of the 5 hook endpoints with a body of exactly 262,145 bytes (256 KiB + 1)
returns HTTP 413 with body `{"error":"payload_too_large","limit_bytes":262144}`.

### AC-002 (traces to BC-2.01.003 postcondition 2)
A POST to any of the 5 hook endpoints with a body of exactly 262,144 bytes (256 KiB) is
accepted by the body-limit layer and proceeds to the handler (may return other errors, but
not 413).

### AC-003 (traces to BC-2.01.003 postcondition 3 — unauthenticated endpoints exempt)
`GET /healthz` does NOT have `DefaultBodyLimit` applied. Sending a 300 KiB body to
`/healthz` does not return 413.

### AC-004 (traces to BC-2.01.003 invariant 1 — exact error body)
The 413 response body is EXACTLY `{"error":"payload_too_large","limit_bytes":262144}`.
No extra fields; no different key names. `Content-Type: application/json`.

### AC-005 (traces to BC-2.01.003 edge case EC-002 — authenticated endpoints only)
The body size limit applies ONLY to the authenticated router. `/healthz` and `/status`
are on separate routers; confirm `/status` GET (no body) is unaffected.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~600 |
| BC-2.01.003.md | ~500 |
| VP-003 file | ~400 |
| axum DefaultBodyLimit docs (reference) | ~300 |
| Test file | ~400 |
| **Total estimate** | **~2,200** |

## Tasks

- [ ] Apply `axum::extract::DefaultBodyLimit::max(262_144)` on the authenticated router layer
- [ ] Implement custom 413 error handler that returns `{"error":"payload_too_large","limit_bytes":262144}`
- [ ] Add integration tests `monocle-runtime/tests/body_size_limit.rs`:
  - POST 262,145 bytes to `/hooks/pre-tool-use` → 413 with exact body
  - POST 262,144 bytes to `/hooks/pre-tool-use` → NOT 413
  - GET /healthz with large body → NOT 413
- [ ] Verify VP-003 probe: fuzz test with random body sizes around the 262,144-byte boundary

## Previous Story Intelligence

S-001 (Wave 1): Workspace initialized. axum 0.8.9 pinned.
This story is small — it is a single axum `Layer` configuration on the authenticated router.
The authenticated router is built in S-003, but this story can stub the hook POST routes
for testing purposes even before S-009 delivers the full hook handlers.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.32 §Body Size Limit:
- `DefaultBodyLimit::max(262_144)` applied to authenticated router ONLY
- `DefaultBodyLimit` is NOT applied to the unauthenticated router

**Forbidden Dependencies:**
- Body limit middleware MUST NOT be imported in the unauthenticated router setup

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| axum | =0.8.9 | DefaultBodyLimit layer |
| serde_json | =1.0.149 | 413 error response body |

## File Structure Requirements

Files to modify:
- `monocle-runtime/src/router.rs` — add `DefaultBodyLimit::max(262_144)` layer to authenticated router
- `monocle-runtime/src/handlers/errors.rs` — custom 413 rejection handler (create)
- `monocle-runtime/tests/body_size_limit.rs` — integration test (create)
