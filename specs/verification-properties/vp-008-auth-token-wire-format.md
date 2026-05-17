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
source_bc: BC-2.01.008
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

# VP-008: Auth Token — Wire Format + Constant-Time Comparison (FC-06)

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-AUTH-001 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

The lock-file `authToken` field is a 64-character lowercase hex string
(`^[0-9a-f]{64}$`) sourced from `rand::rngs::OsRng` (NOT `thread_rng`). The
wire-format token presented in `X-Monocle-Authorization` is
`"monocle-v1:" ++ authToken` (74 characters total). `validate_auth_token`
returns `true` iff the prefix is `monocle-v1:` AND the post-prefix hex
equals `expected_secret` byte-for-byte under `constant_time_eq` comparison
(NOT `==` on `&str`/`String`). Defense-in-depth: the containing
`<runtime_dir>` is `0o700` (per VP-005) and the lock file itself is
`0o600`. Fuzz target asserts NO panic and NO `true` for any input differing
from the expected secret.

## Source Contract

- **BC (primary):** BC-2.01.008 — Auth Token Wire Format (FC-06).
- **BCs (partial coverage):** BC-2.01.005 (lock-file mode + runtime-dir
  defense-in-depth pairing).
- **Postcondition/Invariant:** 64-hex token format from `OsRng`;
  `monocle-v1:`-prefixed wire format; `constant_time_eq` comparison
  (source-grep negative assertion on raw `==`); defense-in-depth with
  `0o700` runtime dir + `0o600` lock file.
- **Traces to (historical):** BC-AUTH-001 (SS-daemon-lifecycle.md §Start
  Sequence).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test (axum 0.8 test client) | Bounded — finite probe set | Lock-file token regex; wire-format round-trip; bit-flip rejection; missing-prefix rejection |
| Fuzz (auxiliary) | cargo-fuzz / libFuzzer | Bounded byte-sequence universe | Arbitrary bytes never produce `true` except the exact expected secret; no daemon panic |
| Source-grep (structural) | ripgrep | N/A — static | `constant_time_eq::constant_time_eq` used in `auth.rs`; no raw `==` on hex secret strings |

## Mechanism

Integration test (primary; harness at
`monocle-runtime/tests/auth_token_lifecycle.rs` — files in `<crate>/tests/`
are cargo integration tests; PRD v1.25 §7 RTM Test Type column labels this
BC `Integration`); fuzz (auxiliary). The harness asserts the lock-file
`authToken` regex, performs round-trip valid/invalid auth probes against
the test server, asserts bit-flip rejection (single-byte-flipped hex →
401), and runs the source-grep on `monocle-runtime/src/auth.rs`. The fuzz
target sweeps arbitrary `X-Monocle-Authorization` values and asserts
panic-free behavior with `true` only for the exact expected secret.

## Pre-conditions

- Daemon has completed start sequence and written `monocle.lock`.
- `constant_time_eq ^0.3` is the project pin (per SS-deps-pin-manifest.md
  v1.1.17).
- `rand::rngs::OsRng` is the entropy source (not `thread_rng`).

## Post-conditions

1. `lock.authToken` matches `^[0-9a-f]{64}$` (exact length 64, lowercase
   hex only).
2. Presenting `monocle-v1:<lock.authToken>` to `/status` returns HTTP 200.
3. Presenting `monocle-v1:<lock.authToken with one byte flipped>` returns
   HTTP 401.
4. Presenting `<lock.authToken>` WITHOUT the `monocle-v1:` prefix returns
   HTTP 401.
5. The auth middleware's secret comparison uses `constant_time_eq`; this is
   verified by source-grep against `monocle-runtime/src/auth.rs` ensuring no
   `==` on the hex secret string appears outside `constant_time_eq`.
6. **Cross-property with VP-005 Post-condition 9:** the auth token's
   containing `<runtime_dir>` is protected by `0o700` owner-only mode
   (defense-in-depth with this VP's auth-token in-band protections —
   `monocle-v1:`-prefixed wire format + `constant_time_eq` comparison +
   64-hex `OsRng` entropy — and out-of-band protections — `0o600`
   lock-file mode per VP-005 §Post-condition 1). The `0o700` runtime-dir
   mode is the outermost ring of the defense-in-depth layering
   protecting the `lock.authToken` field: even if the lock-file mode
   bits regress, the containing directory's owner-only mode prevents
   other OS users from stat-traversing or reading the file. Per VP-005
   §Post-condition 9 the `0o700` mode is asserted on the
   runtime-dir-creation path; this VP reciprocates the cross-property
   reference (Obs-R84-2 / SE-15d cross-property reciprocity closure).

## Counter-examples

1. Switch `constant_time_eq` to `String::eq` — would still pass functional
   tests but would lose the timing-oracle property; mitigated by the
   source-grep assertion in the harness.
2. Lock file written with `tempfile::persist` interrupted mid-write —
   partial token leaves a < 64-char hex; the regex match must reject.
3. Token generation via `rand::thread_rng()` instead of `OsRng` — passes
   the format regex but fails the entropy source check (verified by
   source-grep against `monocle-runtime/src/lock.rs`).
4. Adversary submits `monocle-v1:` + 64 chars of `0` (all-zero secret) —
   must be rejected because the real secret has 256 bits of entropy.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 8.a | Read `lock.authToken` from generated lock file | matches `^[0-9a-f]{64}$` |
| 8.b | Present `monocle-v1:<lock.authToken>` to `/status` | HTTP 200 |
| 8.c | Present `monocle-v1:<bit-flipped 64-hex>` to `/status` | HTTP 401 |
| 8.d | Present `<lock.authToken>` (no prefix) to `/status` | HTTP 401 |
| 8.e | Source-grep `monocle-runtime/src/auth.rs` for `constant_time_eq` | present; raw `==` on hex secret absent |
| 8.f | Source-grep `monocle-runtime/src/lock.rs` for `OsRng` | present; `thread_rng` absent in token-gen path |
| 8.g | Cross-property: containing `<runtime_dir>` mode | `0o700` (assertion in VP-005 5.e) |
| 8.h | Fuzz: arbitrary `X-Monocle-Authorization` bytes | no panic; `true` only for exact expected secret |

**Fuzz harness:** `cargo fuzz add fuzz_auth_token_validation`. The fuzz
target constructs arbitrary byte sequences as the
`X-Monocle-Authorization` value and runs `validate_auth_token(input,
expected)` against a fixed 64-char hex secret. The fuzzer should never
produce an input that returns `true` other than the exact expected secret
with the `monocle-v1:` prefix. The target asserts NO panic and NO `true`
return for any input differing from the expected secret.

## Harness Location

- `monocle-runtime/tests/auth_token_lifecycle.rs` (integration)
- `fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz)
- Test name: `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`
  (per PRD v1.25 §BC-AUTH-001, Verification subsection — to be migrated to
  `test_BC_2_01_008_lockfile_token_format_and_auth_round_trip`).

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
fn verify_bc_2_01_008() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/auth_token_lifecycle.rs` (integration)
- `fuzz/fuzz_targets/fuzz_auth_token_validation.rs` (fuzz)
- Test name: `test_BC_AUTH_001_lockfile_token_format_and_auth_round_trip`
  (per PRD v1.25 §BC-AUTH-001, Verification subsection — to be migrated to
  `test_BC_2_01_008_lockfile_token_format_and_auth_round_trip`).

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
- Predecessor: monolithic VP-AUTH-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.008.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 §Start
  Sequence (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.008 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.17.
- Cross-property: VP-005 §Post-condition 9 (runtime-dir `0o700`
  defense-in-depth pairing); VP-009 (auth header validation taxonomy).

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
- **SE-17c-d body-scope grep:** pre-§Trace-block body cites of `1.1.15` → 0 remaining; cites of `1.1.17` → 2 (Pre-conditions §subtle pin, References §Dependency pins).

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
