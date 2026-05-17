---
document_type: verification-property
level: L4
version: "1.0.2"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T19:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.02.001
module: monocle-runtime
proof_method: integration-test
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

# VP-011: ABI Version in `/status` Endpoint

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-ABI-001 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

A `GET /status` request with a valid `X-Monocle-Authorization` header returns
HTTP 200 with a JSON body whose top-level `abi_version` key has the integer
value `1` (equal to `monocle_core::MONOCLE_ABI_VERSION` as compiled into the
daemon binary).

## Source Contract

- **BC:** BC-2.02.001 — ABI Version (`/status` Endpoint Response).
- **Postcondition/Invariant:** BC-2.02.001 Postcondition asserting
  `abi_version: 1` integer in `/status` JSON body; cross-property with
  VP-012 §Post-condition 1 (compile-time `const _: () =
  assert!(MONOCLE_ABI_VERSION == 1)` in the binary crate ensures drift
  between binary and constant is impossible).
- **Traces to (historical):** BC-ABI-001 (SS-core-types-and-abi.md
  §ABI Version Constant; PRD v1.25 §BC-ABI-001 Verification subsection).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — finite probe set | Status-code + body-shape across authenticated probe; integer-equality assertion on `abi_version` field |

## Mechanism

Integration test (harness at `monocle-runtime/tests/status_abi_version.rs`
— files in `<crate>/tests/` are cargo integration tests per Rust
convention; PRD v1.25 §7 RTM Test Type column labels this BC
`Integration`). The harness constructs an axum test server, makes an
authenticated `GET /status` request, parses the JSON body, and asserts
`body["abi_version"] == 1` (integer comparison, not string).

## Pre-conditions

- Daemon running with a valid lock file.
- Authenticated client holds the lock-file secret.
- `axum 0.8` is the project pin (per SS-deps-pin-manifest.md v1.1.17).

## Post-conditions

1. HTTP 200 status code on authenticated `GET /status`.
2. Response body parsed as JSON has key `abi_version` with integer value `1`.
3. The value `1` equals `monocle_core::MONOCLE_ABI_VERSION` at compile time
   (compile-time `const _: () = assert!(MONOCLE_ABI_VERSION == 1)` in the
   binary crate ensures drift between binary and constant is impossible;
   cross-property with VP-012).

## Counter-examples

1. `/status` handler hardcodes `"abi_version": 2` — must fail the literal
   integer comparison.
2. `MONOCLE_ABI_VERSION` raised to `2` without updating the status handler —
   the compile-time assert catches drift; without the assert, the
   integration test would still catch the runtime mismatch.

## Probe Matrix

| Probe | Setup | Expected status | Expected body shape |
|-------|-------|-----------------|---------------------|
| 11.a | Authenticated `GET /status` | 200 | JSON contains `abi_version: 1` (integer) |
| 11.b | Drift simulation: handler returns `abi_version: 2` | 200 | Test FAILS integer-equality assertion |
| 11.c | Drift simulation: `MONOCLE_ABI_VERSION` raised to `2` without handler update | compile error | Compile-time `const _:` assert traps the drift |

## Harness Location

- `monocle-runtime/tests/status_abi_version.rs` (integration test)
- Test name: `test_BC_ABI_001_status_endpoint_returns_abi_version_1` (per
  PRD v1.25 §BC-ABI-001, Verification subsection — to be migrated to
  `test_BC_2_02_001_status_endpoint_returns_abi_version_1` post BC
  renumber propagation into source).

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: integration-test
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_02_001() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/status_abi_version.rs` (integration test)
- Test name: `test_BC_ABI_001_status_endpoint_returns_abi_version_1` (per
  PRD v1.25 §BC-ABI-001, Verification subsection — to be migrated to
  `test_BC_2_02_001_status_endpoint_returns_abi_version_1` post BC
  renumber propagation into source).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: integration-test` per frontmatter; mechanism documented in `## Mechanism` section. |
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

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-ABI-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.001.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §ABI Version Constant.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.001 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.17.
- Cross-VP: VP-012 (compile-time const equals `1`).

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

---

## §Trace v1.0.2 — F-R105-7 MED: Manifest Pin Refresh v1.1.15 → v1.1.17

**Bump:** v1.0.1 → v1.0.2.
**Predecessor pin:** v1.0.1 (commit 4090d0b — RES-03 VP heading reconciliation).
**Scope of v1.0.2 (NORMATIVE — stale-pin refresh; NO content cascade):**

### Change set (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** body cited `SS-deps-pin-manifest.md v1.1.15` at 2 locations (pre-edit grep).
  - **After:** body cites `SS-deps-pin-manifest.md v1.1.17` at the same 2 locations (post-edit grep).
- **SE-17c-d body-scope grep:** pre-§Trace-block body cites of `1.1.15` → 0 remaining; cites of `1.1.17` → 2 (Pre-conditions §axum pin, References §Dependency pins).

### Rationale

Architect confirmed (T-128d, commit 0d0c64b) the manifest delta v1.1.15 → v1.1.17 is **STRUCTURAL ONLY** — version-number swap with no content cascade. Therefore the only required downstream action across VP files is the pin-citation refresh; no substantive change to the VP property statement, proof method, mechanism, pre-conditions, post-conditions, counter-examples, probe matrix, or harness location.

### Authoritative cross-references

- **Manifest:** `architecture/SS-deps-pin-manifest.md` v1.1.17 (commit 0d0c64b — T-128d §Trace reconciliation).
- **R105 closure chain:** F-R105-7 MED — manifest pin refresh sweep across 14 pin-citing VP files (the other 8 VP files do not cite the manifest pin and are unchanged in this T-128g dispatch).
- **Concurrent dispatch:** T-128j FV portion — VP-014 title sync to VP-INDEX canonical + VP-007 sister-VP reference reconciliation `VP-TYPES-001` → `VP-013`.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T19:30:00Z` >= chain high-water `2026-05-17T19:00:00Z` (nfr-catalog.md). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: pin refresh `v1.1.15` → `v1.1.17`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Refreshed the pin citation in-scope of T-128g rather than deferring or routing through a parallel dispatch. The architect's structural-only delta classification (T-128d) authorized this mechanical citation refresh without requiring per-VP content review. No tech-debt entries created.
