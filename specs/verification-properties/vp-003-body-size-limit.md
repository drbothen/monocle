---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.01.003
module: monocle-runtime
proof_method: manual+fuzz
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-003: Body Size Limit — 256 KiB; HTTP 413 on Excess

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-003 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

POST requests to any authenticated route (the 5 hook endpoints
`/hooks/pre-tool-use`, `/hooks/notification`, `/hooks/stop`,
`/hooks/session-start`, `/hooks/prompt-submit`, and `/status`,
`/shutdown`) with a request body strictly exceeding 256 KiB
(262,144 bytes) are rejected with HTTP 413 and body
`{"error":"payload_too_large","limit_bytes":262144}`. Bodies up to and
including exactly 262,144 bytes succeed. The `DefaultBodyLimit::max(256 *
1024)` layer is mounted on the authenticated router only — `/healthz`
(unauthenticated, GET-only) is NOT subject to the limit. The source layer is
asserted by source-grep (exactly once, on `auth_router`).

## Source Contract

- **BC:** BC-2.01.003 — Body Size Limit (256 KiB, HTTP 413).
- **Postcondition/Invariant:** boundary semantics (`> N` rejected, `<= N`
  accepted, where `N = 262144`); exact error body shape; layer placement
  on authenticated router only; uniform cross-route behavior across all
  authenticated routes including `/status`.
- **Traces to (historical):** BC-DAEMON-003 (PRD v1.25 §BC-DAEMON-003;
  SS-daemon-lifecycle.md v1.0.25 §Body Size Limit).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — finite size set around boundary | Boundary semantics at N-1 / N / N+1 across 5 hook endpoints + `/status`; exact body shape on 413 |
| Fuzz (auxiliary) | cargo-fuzz / libFuzzer | Bounded body length 262_140..=262_150 | Boundary exploration; daemon-panic absence |
| Source-grep (structural) | ripgrep | N/A — static | `DefaultBodyLimit::max(256 * 1024)` appears exactly once in `monocle-runtime/src/server.rs` on `auth_router` only |

## Mechanism

Integration test (primary; harness at `monocle-runtime/tests/body_size_limit.rs`
— files in `<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM
Test Type column labels this BC `Integration`); fuzz (auxiliary — boundary
exploration). The integration harness constructs request bodies of sizes
262_143, 262_144, and 262_145 and asserts the corresponding HTTP responses
on each of the 5 hook endpoints + `/status`. The fuzz target sweeps
boundary-adjacent sizes (262_140..=262_150) to catch off-by-one regressions
and asserts daemon-panic absence.

## Pre-conditions

- Daemon running with a valid lock file.
- `axum::extract::DefaultBodyLimit::max(256 * 1024)` is the layer pinned
  to the authenticated router at construction time. The integration test
  asserts the layer is present via a `cargo expand` or source-grep
  inspection of `monocle-runtime/src/server.rs`.
- Test client holds the auth token for the positive controls.

## Post-conditions

1. POST 262,145-byte body to any of the 5 hook endpoints with valid auth →
   HTTP 413 with exact body
   `{"error":"payload_too_large","limit_bytes":262144}`.
2. POST 262,144-byte body (boundary) to any hook endpoint with valid auth →
   HTTP 200 (within limit; processed normally).
3. POST 262,143-byte body (one under) to any hook endpoint with valid auth →
   HTTP 200.
4. POST 262,145-byte body to `/status` with valid auth → HTTP 413
   (cross-route limit coverage). Cross-check VP-002 §Post-condition
   5 (`/status` route inherits the body-limit layer): VP-002 cites
   this VP as the cross-check for the `/status` route's 413 behavior;
   this VP asserts that 262,145-byte bodies on `/status` produce the
   same HTTP 413 response shape as on the 5 hook endpoints.
5. Source-grep asserts `DefaultBodyLimit::max(256 * 1024)` appears
   exactly once in `monocle-runtime/src/server.rs` and is applied to the
   authenticated router only (not the `/healthz` route).

## Counter-examples

1. `DefaultBodyLimit` layer omitted — 262,145-byte body returns HTTP 200
   (the request is processed, exposing unbounded memory); the test must
   assert 413.
2. `DefaultBodyLimit::max(256 * 1024)` applied to the unauthenticated
   router by mistake — `/healthz` would reject oversized bodies but
   `/healthz` is GET-only; benign drift but still wrong; the source-grep
   asserts the layer is on the authenticated router only.
3. Limit set to `262_144` instead of `256 * 1024` (off-by-one constant) —
   functionally identical (both equal 262,144) but the literal constant
   form `256 * 1024` is preferred for readability; the source-grep
   tolerates either form.
4. Error body returns `{"error":"too_large"}` (typo / non-canonical) —
   fails the exact-body assertion.

## Probe Matrix

| Probe | Setup | Expected status | Expected body |
|-------|-------|-----------------|---------------|
| 3.a | POST 262,143 bytes to any of 5 hook endpoints (valid auth) | 200 | normal route body |
| 3.b | POST 262,144 bytes (boundary) to any of 5 hook endpoints | 200 | normal route body |
| 3.c | POST 262,145 bytes to any of 5 hook endpoints | 413 | `{"error":"payload_too_large","limit_bytes":262144}` |
| 3.d | POST 262,145 bytes to `/status` (cross-route inheritance) | 413 | same body as 3.c |
| 3.e | Source-grep: `DefaultBodyLimit::max(256 * 1024)` placement | N/A | layer present on `auth_router` only (exactly once) |
| 3.f | Fuzz: body lengths 262_140..=262_150 sweep | per limit | 413 iff length > 262_144; 200 iff length ≤ 262_144; no daemon panic |

## Harness Location

- `monocle-runtime/tests/body_size_limit.rs` (integration)
- `fuzz/fuzz_targets/fuzz_body_size_boundary.rs` (fuzz, Phase 6 deliverable —
  `cargo fuzz add fuzz_body_size_boundary`)
- Test name: `test_BC_DAEMON_003_body_size_limit_413_on_excess` (per PRD
  v1.25 §BC-DAEMON-003, Verification subsection — to be migrated to
  `test_BC_2_01_003_body_size_limit_413_on_excess`).

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-DAEMON-003 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.003.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.003 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-002 (`/status` cross-route inheritance).
