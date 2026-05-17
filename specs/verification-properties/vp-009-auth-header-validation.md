---
document_type: verification-property
level: L4
version: "1.0.1"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T18:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
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

## Proof Method

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

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: manual+fuzz
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_01_009() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/auth_header_rejection.rs` (integration)
- `fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz, shared with VP-008)
- Test name: `test_BC_AUTH_002_auth_header_validation_all_failure_modes`
  (per PRD v1.25 §BC-AUTH-002, Verification subsection — to be migrated to
  `test_BC_2_01_009_auth_header_validation_all_failure_modes`).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: manual+fuzz` per frontmatter; mechanism documented in `## Mechanism` section. |
| Tool support | Available | Tooling pinned in `architecture/SS-deps-pin-manifest.md`; no novel verification tooling required. |
| Estimated proof time | Within Phase-1 budget | `feasibility: feasible` per frontmatter. Coverage details in `## Proof Method` table; probe enumeration in `## Probe Matrix`. |

**Authoritative feasibility verdict:** `feasibility: feasible` per frontmatter (canonical machine-consumed field).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | v1.0.0 (cycle) | vsdd-factory:architect |
| Proof harness committed | pending Phase-3 implementation | vsdd-factory:formal-verifier |
| Proof first passed | pending Phase-6 formal hardening | vsdd-factory:formal-verifier |
| Locked (VERIFIED) | pending (`verification_lock: false`) | vsdd-factory:formal-verifier |

**Authoritative lifecycle state** (canonical machine-consumed fields in frontmatter):

| Field | Current Value |
|-------|---------------|
| `lifecycle_status` | `active` |
| `introduced` | `v1.0.0` |
| `verification_lock` | `false` |
| `proof_completed_date` | `null` |
| `modified` | `[]` |
| `deprecated` | `null` |
| `retired` | `null` |
| `withdrawn` | `null` |

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

---

## §Trace v1.0.1 — Audit R2 Residual RES-03: VP Heading Reconciliation to L4 Template

**Bump:** v1.0 → v1.0.1.
**Predecessor pin:** v1.0 (Dispatch 5a/5b commits 7326ff5 + e3824ec — VP monolith decomposition; Dispatch 7 commit 51e77cb — input-hash population).
**Scope of v1.0.1 (Option 3 hybrid — heading reconciliation; NO content removed):**

### Heading changes (NORMATIVE)

- **Renamed** `## Verification Method` → `## Proof Method` (template-strict per L4-verification-property-template.md §Proof Method). Content unchanged; identical Method/Tool/Bounded?/Coverage table preserved verbatim.
- **Added** `## Proof Harness Skeleton` (template-required §Proof Harness Skeleton). Section is a template-strict skeleton block that references the existing rich harness specification (`## Mechanism` for execution narrative, `## Probe Matrix` for probe enumeration, `## Harness Location` for file + test name). The L4 template's skeleton is a Rust code-block stub; monocle's pre-existing `## Mechanism` + `## Harness Location` exceed the template's stub fidelity by carrying execution narrative AND concrete file paths. The new `## Proof Harness Skeleton` section satisfies the template heading requirement without removing the richer Phase-1-specific content.
- **Added** `## Feasibility Assessment` (template-required §Feasibility Assessment). Section is a populated Factor/Assessment/Notes table derived from the `feasibility:` frontmatter field (canonical machine-consumed value) + the `## Proof Method` Bounded?/Coverage columns. The L4 template carries feasibility as both a frontmatter field and a body table; pre-v1.0.1 monocle VPs carried it only in frontmatter. v1.0.1 adds the body table without changing the authoritative frontmatter value.
- **Added** `## Lifecycle` (template-required §Lifecycle). Section is a populated Event/Date/Actor table + an authoritative lifecycle-state mirror of the DF-030 lifecycle frontmatter fields. The L4 template carries lifecycle as both frontmatter fields and a body table; pre-v1.0.1 monocle VPs carried it only in frontmatter. v1.0.1 adds the body table without changing the authoritative frontmatter values.

### Project-specific extensions retained (INFORMATIONAL rationale per SE-17g)

The monocle VP body retains the following sections beyond the L4 template's minimum set; they encode Phase-1-specific verification discipline that emerged during the cycle-001 adversarial review chain:

- `## Mechanism` — execution narrative for the proof harness. Encodes the F-R88-5 discipline of separating test-type-class (Integration/Unit/AST-audit/Proptest) from harness-execution-narrative. Without this section the harness intent reduces to a one-cell `Method` column in `## Proof Method`, which proved insufficient during R85-R87 for adversary fresh-context comprehension.
- `## Pre-conditions` — precondition surface for the harness. Required for proof-harness reproducibility per F-R89/F-R90 work.
- `## Post-conditions` — assertion surface for the harness. The numbered postcondition format (1., 2., 3., …) emerged from R88-R91 BC↔VP round-trip discipline; flat unstructured postcondition prose proved insufficient for fresh-context BC→VP traceability.
- `## Counter-examples` — negative cases. Encodes mutation-test rationale per VP-013's `## Mutation-Test Rationale` extension; even VPs without an explicit mutation block carry counter-example enumeration to make the assertion surface adversary-reviewable.
- `## Probe Matrix` — probe enumeration table. The Probe ID column (e.g., `1.a`, `14.b`, `22.c`) provides direct probe→assertion traceability for the F-R89/F-R90 probe-enumeration discipline that emerged from BC↔VP round-trip cycles. Without this section the probe set is implicit in `## Mechanism` prose, which the adversary repeatedly flagged as insufficient.
- `## Harness Location` — direct test-path traceability (file path + test name). Provides implementer + test-writer agents in Phase 3 with explicit harness implementation targets, eliminating one round-trip during TDD red-gate setup.

### Authoritative cross-references

- **L4 template:** `templates/L4-verification-property-template.md` (canonical heading set: `## Property Statement`, `## Source Contract`, `## Proof Method`, `## Proof Harness Skeleton`, `## Feasibility Assessment`, `## Lifecycle`).
- **Audit R1:** `.factory/plans/template-compliance-audit-r1.md` (D-122 trigger — initial heading-name mismatch identification).
- **Audit R2:** `.factory/plans/template-compliance-audit-r2.md` RES-03 (residual heading-name mismatch on all 22 VP files; this v1.0.1 closes RES-03).

### Concurrent dispatches

- **architect RES-01+RES-04:** COMPLETE (commit 0af206a) — input-hash normalization + ARCH-INDEX Tokens column.
- **PO RES-02+RES-05:** COMPLETE (commit 1a09095) — BC VP anchor sweep + PRD §6/§7 column reconciliation.
- **FV RES-03:** this v1.0.1 (audit R2 residual closure for all 22 VP files).

### Content preservation verification

NO content removed. Heading renames in place. New `## Proof Harness Skeleton` / `## Feasibility Assessment` / `## Lifecycle` sections are derived from existing frontmatter fields + existing body sections; they add structure without changing the authoritative machine-consumed values (which remain in frontmatter per the L4 schema).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T18:00:00Z` >= chain high-water `2026-05-17T17:30:00Z` (PRD v1.26.1 — RES-05 concurrent dispatch). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: heading renames + new section additions (`## Proof Method`, `## Proof Harness Skeleton`, `## Feasibility Assessment`, `## Lifecycle`); frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: project-specific extension rationale (above subsection); audit cross-references; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Chose **Option 3 (hybrid)** — template-aligned form with documented project-specific extensions — over Option 1 (template-strict rename + reorganize, which would risk content drift on the Phase-1-emergent probe enumeration discipline) and Option 2 (extension-only documentation without template heading adoption, which would leave the audit RES-03 WARN unresolved). Option 3 satisfies the L4 template heading requirements (all 6 required headings present) AND preserves the Phase-1 verification discipline (probe matrices, mechanism narratives, harness locations) that the adversarial review chain hardened.
