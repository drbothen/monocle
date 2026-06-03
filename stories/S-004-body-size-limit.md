---
document_type: story
level: L4
story_id: S-004
epic_id: EPIC-01
version: "1.1"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 2
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001, S-003]
blocks: [S-009]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.003]
verification_properties: [VP-003]
estimated_days: 1
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.003.md, version: "1.0.5"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-003-body-size-limit.md, version: "1.0.14"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
  - {path: .factory/specs/prd-supplements/nfr-catalog.md, version: "1.7"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.01.003 (Body Size Limit); verifies VP-003; covers EC-045; addresses NFR-005, E-DAEMON-001."
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
A POST to any of the 5 hook endpoints with a body of exactly 262,144 bytes (256 KiB)
returns HTTP status ≠ 413 (canonical: HTTP 200 per BC-2.01.003 canonical test vectors).

### AC-003 (traces to BC-2.01.003 postcondition 3 — unauthenticated endpoints exempt)
`GET /healthz` (no body) returns 200, AND the `/healthz` route definition contains no
`DefaultBodyLimit` layer (source-grep assertion per VP-003 Proof Method line 67: grep
`monocle-runtime/src/server.rs` for `DefaultBodyLimit` — must appear on the
`auth_router` only, not on the unauthenticated router).

### AC-004 (traces to BC-2.01.003 invariant 1 — exact error body)
The 413 response body is EXACTLY `{"error":"payload_too_large","limit_bytes":262144}`.
No extra fields; no different key names. `Content-Type: application/json`.

### AC-005 (traces to BC-2.01.003 edge case EC-045 — cross-route coverage)
POST a 262,145-byte body to `/status` with valid auth → HTTP 413 with exact body
(cross-route coverage per VP-003 PC-4). GET `/healthz` with any body size → not 413
(per BC-2.01.003 PC-3). Note: `/status` is on the AUTHENTICATED router per
SS-daemon-lifecycle.md line 169 and IS subject to the body limit per VP-003 PC-4 (line 98).

### AC-006 (traces to BC-2.01.003 structural invariant — single DefaultBodyLimit layer placement)
Source-grep assertion: `DefaultBodyLimit::max(256 * 1024)` appears exactly ONCE in
`monocle-runtime/src/server.rs`, on the `auth_router`. This locks down layer placement
and prevents accidental drift (per VP-003 line 67 Proof Method).

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

- [ ] Apply `axum::extract::DefaultBodyLimit::max(256 * 1024)` on the authenticated router layer
  in `monocle-runtime/src/server.rs` (canonical source file per VP-003 lines 67 and 86)
- [ ] Implement custom 413 error handler that returns `{"error":"payload_too_large","limit_bytes":262144}`
  with `Content-Type: application/json` response header
- [ ] Add integration tests `monocle-runtime/tests/body_size_limit.rs`
  with canonical test function name `test_BC_DAEMON_003_body_size_limit_413_on_excess`
  (per BC-2.01.003 Traceability line 89):
  - POST 262,145 bytes to `/hooks/pre-tool-use` → 413 with exact body
  - POST 262,144 bytes to `/hooks/pre-tool-use` → NOT 413 (HTTP 200)
  - POST 262,145 bytes to `/status` with valid auth → 413 (cross-route coverage per VP-003 PC-4)
  - GET `/healthz` (no body) → 200 and source-grep confirms no DefaultBodyLimit on unauthenticated router
  - Source-grep: `DefaultBodyLimit::max(256 * 1024)` appears exactly once in `server.rs` on auth_router
- [ ] Verify VP-003 probe: fuzz test with random body sizes around the 262,144-byte boundary

## Previous Story Intelligence

S-001 (Wave 1): Workspace initialized. axum 0.8.9 pinned. S-001 establishes the
`monocle-runtime` workspace crate skeleton.
S-003 (Wave 2, co-dependency): The authenticated router construction lives in S-003
(auth-ownership decision per e485814). S-004 applies the `DefaultBodyLimit::max(256 * 1024)`
layer to S-003's authenticated router in `server.rs`. This story must come after S-003
to merge the body-limit layer into the auth router structure S-003 creates.
This story is small — it is a single axum `Layer` configuration on the authenticated router.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.33 §Body Size Limit (lines 118-176):
- `DefaultBodyLimit::max(256 * 1024)` applied to authenticated router ONLY (line 171)
- `DefaultBodyLimit` is NOT applied to the unauthenticated router
- Implementation file: `monocle-runtime/src/server.rs` (per VP-003 lines 67 and 86)
- Error taxonomy: `error-taxonomy.md §E-DAEMON-001` (line 41): `payload_too_large` body

**Forbidden Dependencies:**
- Body limit middleware MUST NOT be imported in the unauthenticated router setup
- Implementation file MUST NOT be `router.rs` — canonical target is `server.rs` (VP authority)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| axum | =0.8.9 | DefaultBodyLimit layer |
| serde_json | =1.0.149 | 413 error response body |

## File Structure Requirements

Files to modify:
- `monocle-runtime/src/server.rs` — add `DefaultBodyLimit::max(256 * 1024)` layer to authenticated router
  (canonical source file per VP-003; NOT router.rs)
- `monocle-runtime/src/handlers/errors.rs` — custom 413 rejection handler (create)
- `monocle-runtime/tests/body_size_limit.rs` — integration test (create)

## §Trace v1.1

**Phase 3.B Batch 2 — spec-reviewer remediation** (2026-05-20):
- F-C-01 [CRIT]: AC-005 rewritten — router-membership reversed per BC-2.01.003 PC-3 + VP-003 PC-4:
  `/status` IS on the authenticated router and IS subject to the body limit; AC-005 now asserts
  POST 262,145 bytes to `/status` → 413 (not the previous incorrect claim that /status is exempt).
- F-B-01 + F-B-02 [MED]: EC-002 → EC-045 throughout (frontmatter traces_to + AC-005 header);
  EC-002 belongs to BC-2.01.007 (ring buffer), not BC-2.01.003.
- F-C-02 [HIGH]: AC-002 oracle tightened — "returns HTTP status ≠ 413 (canonical: HTTP 200)".
- F-C-03 [MED]: AC-003 reframed — GET /healthz (no body) + source-grep structural assertion per VP-003.
- F-C-04 [LOW]: AC-006 added — source-grep `DefaultBodyLimit::max(256 * 1024)` appears exactly once in server.rs.
- F-D-01 [MED]: File path router.rs → server.rs throughout (VP-003 is the verification authority).
- F-D-02 [LOW]: Canonical literal `256 * 1024` used throughout (not `262_144`) for source-grep determinism.
- F-D-03 [LOW]: Canonical test function name `test_BC_DAEMON_003_body_size_limit_413_on_excess` surfaced in Tasks.
- F-D-04 [LOW]: `Content-Type: application/json` added to 413 rejection response task.
- F-B-03 [LOW]: error-taxonomy anchor `§E-DAEMON-001 (line 41)` added.
- F-B-04 [LOW]: SS-daemon-lifecycle anchor `lines 118-176 §Body Size Limit` added.
- F-E-01 [HIGH]: S-003 added to depends_on (auth-ownership decision: authenticated router built by S-003).
- F-E-02 [LOW]: Previous Story Intelligence expanded with S-001 workspace crate note.
