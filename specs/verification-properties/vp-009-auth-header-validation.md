---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
traces_to: prd.md
source_bc: BC-2.01.009
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

# VP-009: Auth Header Validation — Two-Body Taxonomy (`missing_auth_token` vs `invalid_auth_token`)

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-AUTH-002 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

The auth middleware's `AuthError` enum has exactly TWO variants: `Missing`
(absent `X-Monocle-Authorization` header → HTTP 401 +
`{"error":"missing_auth_token"}`) and `Invalid` (any value-present failure
including wrong prefix, malformed format, length mismatch, or wrong secret
→ HTTP 401 + `{"error":"invalid_auth_token"}`). The retired v1.0 body
`{"error":"invalid_auth_token_format"}` (per architect commit 2db408f
disposition (c)) MUST NOT appear in any Phase 1 daemon response. `Authorization:
Bearer <anything>` without `X-Monocle-Authorization` is treated as absent
header. All value-present failure modes return the same body intentionally,
preventing a timing- or body-oracle.

## Source Contract

- **BC (primary):** BC-2.01.009 — Auth Header Validation (Missing and
  Invalid Token).
- **Postcondition/Invariant:** two-variant `AuthError` enum; exact body
  taxonomy per probe; uniform application across all 3 authenticated
  route classes (`/hooks/*`, `/status`, `/shutdown`); Bearer-fallback
  rejection; retired-body absence.
- **Traces to (historical):** BC-AUTH-002 (PRD v1.25 §BC-AUTH-002;
  SS-daemon-lifecycle.md v1.0.25 §Start Sequence; architect adjudication
  commit 2db408f — disposition (c) collapsed error taxonomy; F-R62-4
  back-propagation closure landed in arch v1.0.9 commit 8bf3759).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — 7-probe matrix | Two-body taxonomy across all probe categories; positive control |
| Fuzz (auxiliary) | cargo-fuzz / libFuzzer | Bounded byte-sequence universe | Arbitrary `X-Monocle-Authorization` values + absent-header case; no panic; retired body never appears |
| Source assertion (structural) | type system | N/A — compile-time | `AuthError` enum has exactly 2 variants (Missing, Invalid) |

## Mechanism

Integration test (primary; harness at
`monocle-runtime/tests/auth_header_rejection.rs` — files in
`<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test Type
column labels this BC `Integration`); fuzz (auxiliary). The harness
performs all 7 probes in the Probe Matrix below against the same axum test
server and asserts the exact response code + body shape. The fuzz target
sweeps arbitrary `X-Monocle-Authorization` byte sequences (including the
absent-header case via `Option<Vec<u8>>`) and asserts the two-body taxonomy
and the absence of the retired body.

## Pre-conditions

- Daemon is running with a valid `monocle-v1:` secret in the lock file.
- Authenticated test client has access to the secret for the positive
  control (probe 7).
- The auth middleware's `AuthError` enum is defined as exactly:
  ```rust
  pub enum AuthError {
      Missing,  // → HTTP 401 {"error":"missing_auth_token"}
      Invalid,  // → HTTP 401 {"error":"invalid_auth_token"}
  }
  ```
  No third variant exists.

## Post-conditions

The 7-probe taxonomy table below (Probe Matrix) is the post-condition set.
Each row is a deterministic single-body assertion.

## Counter-examples

1. Auth middleware accepts `Authorization: Bearer` as a fallback path —
   probe 5 would return 200; the integration test must assert 401 +
   `missing_auth_token`.
2. Auth middleware uses `presented.contains("monocle-v1:")` instead of
   `strip_prefix("monocle-v1:")` — probe `X-Monocle-Authorization:
   junk-monocle-v1:abc` would be accepted; the integration test asserts
   strict `strip_prefix` behavior (returns 401 + `invalid_auth_token`
   for any value not starting with the literal prefix).
3. Auth middleware returns the retired `invalid_auth_token_format` body
   for probe 2/3/4 — fails the exact-body assertion (the retired taxonomy
   is forbidden post-2db408f).
4. Auth middleware returns `invalid_auth_token` for probe 1 (absent header
   treated as invalid) — fails the missing-vs-invalid distinction; the
   structural precondition (header absence) must produce the
   diagnostic-friendly `missing_auth_token` body.
5. Auth middleware returns `missing_auth_token` for probe 6
   (correct-format wrong-secret) — fails the value-present unification;
   secret mismatch must produce `invalid_auth_token`, not
   `missing_auth_token` (an attacker probing the secret space must not
   learn that their format was correct).

## Probe Matrix

| Probe | Header | Expected status | Expected body |
|-------|--------|-----------------|---------------|
| 9.1 | (no `X-Monocle-Authorization` header) | 401 | `{"error":"missing_auth_token"}` |
| 9.2 | `X-Monocle-Authorization: deadbeef...64chars` (bare token, no prefix) | 401 | `{"error":"invalid_auth_token"}` |
| 9.3 | `X-Monocle-Authorization: monocle-v2:deadbeef...64chars` (wrong version prefix) | 401 | `{"error":"invalid_auth_token"}` |
| 9.4 | `X-Monocle-Authorization: monocle-v1:` (prefix only, no hex suffix) | 401 | `{"error":"invalid_auth_token"}` |
| 9.5 | `Authorization: Bearer fake-token` with no `X-Monocle-Authorization` (wrong header name) | 401 | `{"error":"missing_auth_token"}` |
| 9.6 | `X-Monocle-Authorization: monocle-v1:<wrong-64-hex>` (correct format, wrong secret) | 401 | `{"error":"invalid_auth_token"}` |
| 9.7 | `X-Monocle-Authorization: monocle-v1:<correct-64-hex>` (positive control) | 200 | (route's normal body) |

**Cross-property reciprocations (SE-15d / Extension 16 backfill sweep):**

- **Cross-property with VP-002 §Mechanical property item 4 +
  §Post-condition 2** (auth-header rejection on `/status`): VP-002
  asserts `/status` without an auth header returns HTTP 401 +
  `missing_auth_token` and with a malformed header returns HTTP 401 +
  `invalid_auth_token`; this VP asserts the same two-body taxonomy
  applies uniformly across all 3 authenticated route classes
  (`/hooks/*`, `/status`, `/shutdown`).
- **Cross-property with VP-004 §Post-condition 7** (`/shutdown`
  authentication): VP-004 asserts `POST /shutdown` without an
  auth header returns HTTP 401 + `missing_auth_token`; this VP asserts
  the same body-taxonomy applies (probe 9.1 of the matrix above with the
  `/shutdown` route as the target).

**Fuzz harness:** the `fuzz_auth_token_validation` target shared with
VP-008 is updated to assert the post-2db408f two-body taxonomy. The
fuzzer constructs arbitrary byte sequences as the `X-Monocle-Authorization`
value (including the absent-header case via `Option<Vec<u8>>`) and asserts:

- No panic.
- If header is absent: response body is exactly
  `{"error":"missing_auth_token"}`.
- If header is present but token validation fails for any reason: response
  body is exactly `{"error":"invalid_auth_token"}`.
- Response body is NEVER `{"error":"invalid_auth_token_format"}` (the
  retired body — fuzz harness asserts this body string never appears in
  any response).
- The fuzzer should never produce an input that returns 200 except for
  the exact expected secret with the `monocle-v1:` prefix.

## Harness Location

- `monocle-runtime/tests/auth_header_rejection.rs` (integration)
- `fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz, shared with VP-008)
- Test name: `test_BC_AUTH_002_auth_header_validation_all_failure_modes`
  (per PRD v1.25 §BC-AUTH-002, Verification subsection — to be migrated to
  `test_BC_2_01_009_auth_header_validation_all_failure_modes`).

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-AUTH-002 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.009.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 §Start
  Sequence (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.009 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-002 (`/status` auth probes); VP-004 (`/shutdown` auth
  probe); VP-008 (token wire format + `constant_time_eq`).
