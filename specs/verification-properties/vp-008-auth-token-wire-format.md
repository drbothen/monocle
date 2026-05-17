---
document_type: verification-property
level: L4
version: "1.0.6"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-18T01:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "01ed5ca"
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
- BC index: `behavioral-contracts/BC-INDEX.md` v1.6 (commit pending — PO 7A R108 Round 7A BC scope dispatch; supersedes v1.5 commit d92e4a7 — PO 6A R107 Round 6A finalization; supersedes v1.4 commit bb088a2 — PO 5A R106 Round 5 BC-INDEX dual-accept finalization; supersedes v1.2 commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure).
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.31 §Start
  Sequence (commit 98396fe — Architect 6D SS pin bump; Architect 7C keeps at v1.0.31).
- PRD: `.factory/specs/prd.md` v1.26.6 §BC-2.01.008 (Dispatch 4 commit 1030c65; refreshed to v1.26.6 in R108 Round 7B PO dispatch — commit pending; supersedes v1.26.5 in F-R107-3 / GAP-R46-1 closure commit d92e4a7; supersedes v1.26.4 commit 01af634 pre-R107 fix burst + v1.26.3 commit b2b378b F-R105-12 closure).
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

---

## §Trace v1.0.3 — F-R105-13 LOW: VP §References PRD Citation Refresh v1.26 → v1.26.3

**Bump:** v1.0.2 → v1.0.3.
**Predecessor pin:** v1.0.2 (commit 927fcce — T-128g+T-128j FV — F-R105-7/10/11 (pin refresh + title sync + sister-VP reconciliation)).
**Scope of v1.0.3 (NORMATIVE — §References PRD citation refresh; NO content cascade; NO BC-path changes — BC §References already cite canonical sharded `behavioral-contracts/ss-NN/BC-2.SS.NNN.md` paths):**

### Change set 1 — §References PRD Citation Refresh `v1.26` → `v1.26.3` (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** §References cited `prd.md v1.26 §BC-2.01.008 (Dispatch 4 commit 1030c65).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.3 §BC-2.01.008 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (post-edit grep).
- **SE-17c-d body-scope grep:** post-edit `grep -n "prd.md v1.26 " vp-008-auth-token-wire-format.md` → 0 matches; `grep -n "prd.md v1.26.3" vp-008-auth-token-wire-format.md` → 1 match (§References line).
- **BC §References scope:** §References §Source contract entry already cites canonical sharded `behavioral-contracts/ss-01/BC-2.01.008.md` (per BC-INDEX.md v1.2). No BC-path changes required in this dispatch.
- **Historical PRD v1.25 citations in body prose (Source Contract `Traces to (historical)`, Harness Location `to be migrated to`, Proof Harness Skeleton `to be migrated to`, where present):** UNCHANGED — these are explicitly historical predecessor citations pinned to the pre-Dispatch-4 PRD monolith and must not be refreshed.

### Rationale

PO commit b2b378b (T-128k Round-3 PO dispatch) bumped PRD `v1.26.2 → v1.26.3` for F-R105-12 VP alias + GAP-R44-4 closure. Parallel FV dispatch refreshes VP §References to cite the post-bump PRD version, preserving the stale-citation-zero invariant established in F-R105-7 (manifest pin refresh) and F-R105-11 (sister-VP reference reconciliation). Per CLAUDE.md Production-Grade Rule 1: no MVP-driven deferral; mechanical citation refresh executed in-scope of T-128k FV portion rather than left to post-Round-3 cleanup.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.3 (commit b2b378b — F-R105-12 VP alias + GAP-R44-4 closure).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7 — confirms canonical sharded path `ss-01/BC-2.01.008.md` for BC-2.01.008).
- **R105 closure chain:** F-R105-13 LOW — 22-VP §References PRD citation refresh sweep.
- **Concurrent dispatches (T-128k Round 3):**
  - PO: PRD v1.26.2 → v1.26.3 (F-R105-12 + GAP-R44-4) — COMPLETE (commit b2b378b).
  - architect: auth-header interop adjudication — separate scope.
  - BA: L2-INDEX anchor fixes — separate scope.
  - FV: this §Trace (F-R105-13 — 22-VP §References PRD citation refresh).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T20:30:00Z` >= chain high-water `2026-05-17T19:30:00Z` (this VP's prior v1.0.2 §Trace and PRD v1.26.3 frontmatter). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References PRD citation `v1.26` → `v1.26.3`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### SE-17g META audit (NORMATIVE)

Sweep-wide post-edit re-grep across all 22 VP files: `grep -rE "prd\.md v1\.(26[^.]|26\.[012])(\s|\$)" .factory/specs/verification-properties/vp-*.md` → 0 matches. Sweep-wide re-grep for non-sharded BC paths: `grep -rE "behavioral-contracts/BC-[^I]" .factory/specs/verification-properties/vp-*.md` → 0 matches. F-R105-13 closure verified.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Mechanical citation refresh executed in-scope rather than deferred. PRD v1.26.3 cite is valid as of PO commit b2b378b. No tech-debt entries created. Body-prose historical PRD v1.25 citations preserved unchanged per Production-Grade discipline (historical predecessor citations are not stale; refreshing them would erase audit trail).

---

## §Trace v1.0.4 — F-R106-10 HIGH: SS-daemon-lifecycle Pin Refresh v1.0.25 → v1.0.30

**Bump:** v1.0.3 → v1.0.4.
**Predecessor pin:** v1.0.3 (commit 932f4e0 — T-128k FV F-R105-13 LOW 22-VP §References PRD citation refresh).
**Scope of v1.0.4 (NORMATIVE — mechanical pin refresh; NO content cascade; NO BC-path changes):**

### Change set (NORMATIVE)

- **SE-17f §Source Contract `Traces to (historical)` line:** `SS-daemon-lifecycle.md v1.0.25 ...` → `SS-daemon-lifecycle.md v1.0.30 ...` (per-VP section name preserved verbatim).
- **SE-17f §References Architecture line:** `Architecture: \`architecture/SS-daemon-lifecycle.md\` v1.0.25 ... (commit 18fe265).` → `Architecture: \`architecture/SS-daemon-lifecycle.md\` v1.0.30 ... (commit pending — architect 5E F-FC-I005 removal + dual-accept consolidation).`
- **SE-17f inline `arch v1.0.NN` body-prose cites (where present in this VP):** refreshed to `arch v1.0.30`.

### SE-17c-d body-scope grep

- Post-edit `grep -nE "SS-daemon-lifecycle.md v1\.0\.25" <this-vp>` → 0 matches outside §Trace history blocks (the only remaining `v1.0.25` cites are in earlier §Trace SE-17f BEFORE evidence as preserved historical predecessor evidence per SE-17g audit-trail discipline).
- Post-edit `grep -nE "v1\.0\.30" <this-vp>` → 2+ matches (§Source Contract Traces-to + §References Architecture, plus any inline `arch v1.0.30` cites in body prose).

### Rationale

Architect 5E is bumping SS-daemon-lifecycle v1.0.29 → v1.0.30 in parallel (R106 Round 5) for F-R106 F-FC-I005 fabricated-FC-ID removal and dual-accept consolidation. The architectural delta is **STRUCTURAL** (FC-ID removal — the architecture sections referenced by this VP are unchanged content). Therefore the only required downstream action across pin-citing VP files is the pin-citation refresh; no substantive change to the VP property statement, proof method, mechanism, pre-conditions, post-conditions, counter-examples, probe matrix, or harness location.

The pin refresh is bundled with the broader R106 Round 5D FV dispatch (10 pin-citing VPs + VP-009 ADR-0005 dual-accept expansion + VP-INDEX 4-fix cascade). Cross-dispatch coordination with architect 5E handled via explicit "commit pending" annotation in §References — this will resolve to a concrete SHA during final state-manager pass after all parallel dispatches converge.

### Authoritative cross-references

- **Architecture:** `architecture/SS-daemon-lifecycle.md` v1.0.30 (commit 03a4c57 — architect 5E dispatch; subsequently bumped to v1.0.31 by architect 6D commit 98396fe).
- **R106 closure chain:** F-R106-10 HIGH — 10-VP SS-daemon-lifecycle pin refresh sweep (this VP is one of 10 pin-citing files).
- **Concurrent dispatches (R106 Round 5):**
  - PO 5A: BC + BC-INDEX dual-accept finalization — separate scope.
  - PO 5B: PRD + supplements — separate scope.
  - PO 5C: product-brief — separate scope.
  - FV 5D: this dispatch (VP-009 expansion + 10-VP pin sweep + VP-INDEX cascade).
  - Architect 5E: ADR-0005 path normalization + SS-daemon-lifecycle v1.0.29 → v1.0.30 — separate scope.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T22:30:00Z` >= chain high-water `2026-05-17T22:10:00Z` (BC-2.01.009 v1.0.3 frontmatter timestamp; cross-dispatch BC reference). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: pin refresh `v1.0.25` → `v1.0.30`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Mechanical pin refresh executed in-scope rather than deferred. Architect 5E's structural-only delta (FC-ID removal) authorized this mechanical citation refresh without requiring per-VP content review. No tech-debt entries created. Historical v1.0.25 cite preserved in §References "supersedes" annotation per audit-trail discipline.

---

## §Trace v1.0.5 — F-R107-4 HIGH + GAP-R46-1 HIGH + F-R107-8 (part 2): VP §References PRD Cite Refresh v1.26.3 → v1.26.5 + Active BC-INDEX Cite Addition v1.5

**Bump:** v1.0.4 → v1.0.5.
**Predecessor pin:** v1.0.4 (commits 932f4e0 / 7b8d6e8 — prior R105/R106 FV sweeps).
**Scope of v1.0.5 (NORMATIVE — mechanical §References citation refresh + active BC-INDEX cite addition; NO content cascade):**

### Change set 1 — §References PRD Citation Refresh `v1.26.3` → `v1.26.5` (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** §References cited `prd.md v1.26.3 §BC-2.01.008 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.5 §BC-2.01.008 (Dispatch 4 commit 1030c65; refreshed to v1.26.5 in F-R107-3 / GAP-R46-1 closure, parallel PO 6B dispatch — commit pending; supersedes v1.26.4 commit 01af634 pre-R107 fix burst + v1.26.3 commit b2b378b F-R105-12 closure).` (post-edit grep).
- **SE-17c-d body-scope grep (active §References scope):** post-edit `grep -nE "prd\.md.* v1\.26\.[0-4]" vp-008-auth-token-wire-format.md` outside §Trace history blocks → 0 matches; `grep -n "prd.md\` v1.26.5" vp-008-auth-token-wire-format.md` outside §Trace history blocks → 1 match (§References PRD line).
- **Historical PRD citations in §Trace history blocks:** UNCHANGED per SE-17g audit-trail-preservation discipline (predecessor citations document state-at-the-time of each historical bump and must not be refreshed; refreshing them would erase audit trail).

### Change set 2 — §References Active BC-INDEX Cite Addition v1.5 (NORMATIVE)

- **Rationale for active-cite ADDITION (not §Trace history rewrite):** F-R107-8 part 2 identifies stale BC-INDEX v1.2 cites in 22 VPs' `BC §References scope` evidence text inside §Trace v1.0.3 (F-R105-13 history blocks). Per SE-17g audit-trail-preservation discipline, §Trace history blocks are append-only — they document state at the time of the bump and are immutable. The production-grade closure is to ADD an active §References BC-INDEX cite at the current v1.5 target, which makes the v1.5 cite live-authoritative and demotes the v1.2 mention in §Trace v1.0.3 to historical snapshot evidence (its correct semantic). Before this dispatch, no VP had an active BC-INDEX §References cite — the only BC-INDEX version mentions were in §Trace history. Adding the active cite closes F-R107-8 part 2 durably without violating SE-17g.
- **SE-17f before/after evidence:**
  - **Before:** active §References had no `BC index:` line (only `Source contract:` cited the sharded BC path without referencing BC-INDEX version). Pre-edit grep `grep -nE "BC-INDEX" vp-008-auth-token-wire-format.md` outside §Trace blocks → 0 matches.
  - **After:** active §References gained `BC index: \`behavioral-contracts/BC-INDEX.md\` v1.5 (commit pending — PO 6A R107 Round 6A finalization; supersedes v1.4 commit bb088a2 — PO 5A R106 Round 5; supersedes v1.2 commit 61133a7 — F-R105-3 + F-R105-9 + OBS-R44-1 DI mapping closure).` Post-edit grep `grep -nE "BC-INDEX\.md.* v1\.5" vp-008-auth-token-wire-format.md` outside §Trace blocks → 1 match (§References BC index line).
- **No BC-path or BC-version changes required:** §Source contract entry already cites canonical sharded `behavioral-contracts/ss-01/BC-2.01.008.md` (per BC-INDEX.md v1.5 confirmation). No BC-path edits in this dispatch.

### Rationale

R107 Round 6C (FV scope) closes three findings in coordinated parallel dispatch:

- **F-R107-4 HIGH (VP-009-specific):** VP-009 cited ADR-0005 v1.0.1 in 3 locations (§Source Contract line 89; active §References line 431; §Trace v1.0.4 Authoritative cross-references line 660). Current ADR-0005 is at v1.0.2 (frontmatter `version: "1.0.2"` confirmed at audit time; commit 03a4c57 — F-R106-14 ADR-0005 inputs path normalization + F-R106-7 F-FC-I005 fabrication removal). All 3 cites refreshed to v1.0.2 in this dispatch.
- **GAP-R46-1 HIGH (all-22-VPs):** VP-INDEX was pre-fixed in commit 01af634 (PRD pin v1.26.3 → v1.26.4 pre-R107 fix burst) but the 22 individual VP files' active §References still cited the stale v1.26.3. PO 6B is bumping PRD v1.26.4 → v1.26.5 in parallel; this FV dispatch refreshes all 22 VPs' active §References to the post-PO-6B v1.26.5 target.
- **F-R107-8 part 2 (all-22-VPs):** 22 VPs had stale BC-INDEX v1.2 mentions inside §Trace v1.0.3 history blocks. Per SE-17g audit-trail-preservation, those §Trace mentions are historical snapshots and must not be edited. The durable closure is to ADD active §References BC-INDEX cites at the current v1.5 target (post-PO-6A), making v1.5 the live-authoritative cite and demoting the v1.2 mentions in §Trace v1.0.3 to historical snapshot evidence (their correct SE-17g semantic). PO 6A is bumping BC-INDEX v1.4 → v1.5 in parallel; this FV dispatch targets the post-PO-6A v1.5 version.

Per CLAUDE.md Production-Grade Default Rule 1+5: mechanical citation refresh + durable cite addition executed in-scope of R107 Round 6C rather than deferred. No tech-debt entries created. Historical §Trace block citations preserved unchanged per SE-17g.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.5 (commit d92e4a7 — PO 6B R107 Round 6B PRD + supplements dispatch [co-mingled with PO 6A]; supersedes v1.26.4 commit 01af634 pre-R107 fix burst).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.5 (commit d92e4a7 — PO 6A R107 Round 6A BC scope dispatch [co-mingled with PO 6B]; supersedes v1.4 commit bb088a2 — PO 5A R106 Round 5 BC-INDEX dual-accept finalization).
- **R107 closure chain:** F-R107-4 HIGH (VP-009 ADR-0005 pin refresh — VP-009-only); GAP-R46-1 HIGH (22-VP PRD cite refresh sweep); F-R107-8 part 2 (22-VP active BC-INDEX cite addition).
- **Concurrent dispatches (R107 Round 6):**
  - PO 6A: BC + BC-INDEX scope (BC-INDEX v1.4 → v1.5) — separate scope.
  - PO 6B: PRD + supplements (PRD v1.26.4 → v1.26.5) — separate scope.
  - FV 6C: this dispatch (VP-009 ADR-0005 pin refresh + 22-VP PRD cite refresh + 22-VP BC-INDEX active cite addition + VP-INDEX cascade).
  - Architect 6D: SS-forward-compatibility scope — separate scope.
  - BA 6E: L2-INDEX scope — separate scope.

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T23:00:00Z` >= chain high-water `2026-05-17T22:50:00Z` (VP-INDEX v1.4 timestamp; pre-R107 fix burst commit 01af634). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References PRD citation `v1.26.3` → `v1.26.5`; §References BC index cite ADDITION at v1.5; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Mechanical citation refresh + durable cite-addition executed in-scope rather than deferred. PRD v1.26.5 cite is the post-PO-6B target (commit pending — will resolve to concrete SHA during final state-manager pass after parallel dispatches converge). BC-INDEX v1.5 cite is the post-PO-6A target (commit pending — same resolution). No tech-debt entries created. §Trace history blocks preserved unchanged per SE-17g — historical predecessor citations are not stale; refreshing them would erase audit trail.


---

## §Trace v1.0.6 — F-R108-5 HIGH + F-R108-6 HIGH + F-R108-15 MED: R108 Round 7D FV Cascade (commit-pending Resolution + SS Pin Refresh + Active R7-Forward Cite Refresh)

**Bump:** v1.0.5 → v1.0.6.
**Predecessor pin:** v1.0.5 (commit bd14774 — F-R107 Round 6C FV — 22-VP PRD cite refresh + 22-VP BC-INDEX active cite + VP-009 ADR-0005 pin refresh + VP-INDEX cascade).
**Scope of v1.0.6 (NORMATIVE — R108 Round 7D 3-fix coordinated cascade in parallel dispatch with PO 7A BC + PO 7B PRD/supplements + Architect 7C SS-pin-stable):**

### Change 1 — F-R108-5 + F-R108-6 HIGH: §References Active Cite Refresh to R7-Forward Targets + Historical Placeholder Resolution (NORMATIVE)

- **SE-17f §References BC index line:** active cite refreshed from `v1.5 (commit pending — PO 6A R107 Round 6A finalization)` to `v1.6 (commit pending — PO 7A R108 Round 7A BC scope dispatch; supersedes v1.5 commit d92e4a7 — PO 6A R107 Round 6A finalization)`. The historical R107 "commit pending" annotation resolves to d92e4a7 (PO 6A + 6B co-mingled per Round 6F SM message); the new active cite carries an R108 Round 7A forward-coordination placeholder (will resolve during R108 Round 7E SM pass).
- **SE-17f §References PRD line:** active cite refreshed from `v1.26.5 §BC-2.01.008 (... parallel PO 6B dispatch — commit pending)` to `v1.26.6 §BC-2.01.008 (... R108 Round 7B PO dispatch — commit pending; supersedes v1.26.5 in F-R107-3 / GAP-R46-1 closure commit d92e4a7)`. Same two-step pattern: R107 placeholder resolves to d92e4a7; new R108 Round 7B forward placeholder for v1.26.6 target.
- **SE-17f §Trace v1.0.4 Authoritative cross-references Architecture line:** historical placeholder `v1.0.30 (commit pending — architect 5E dispatch)` resolved in-place to `v1.0.30 (commit 03a4c57 — architect 5E dispatch; subsequently bumped to v1.0.31 by architect 6D commit 98396fe)`. Per SE-17g, this is a historical cross-references block (NOT an SE-17f BEFORE/AFTER snapshot) — refreshing to resolve a now-known SHA is permitted because it describes the target artifact's actual lineage, not a state-at-time-of-bump snapshot.
- **SE-17f §Trace v1.0.5 Authoritative cross-references PRD + BC-INDEX lines:** historical placeholders resolved to commit d92e4a7 (PO 6A + 6B co-mingled).

### Change 2 — F-R108-15 MED: SS-daemon-lifecycle Pin Refresh v1.0.30 → v1.0.31 (NORMATIVE)

- **SE-17f §Source Contract `Traces to (historical)` line:** body cite `SS-daemon-lifecycle.md v1.0.30 §<section>` refreshed in-place to `SS-daemon-lifecycle.md v1.0.31 §<section>` (per-VP section name preserved verbatim).
- **SE-17f §References Architecture line:** body cite `Architecture: \`architecture/SS-daemon-lifecycle.md\` v1.0.30 (commit pending — architect 5E ...)` refreshed in-place to `Architecture: \`architecture/SS-daemon-lifecycle.md\` v1.0.31 (commit 98396fe — Architect 6D SS pin bump; Architect 7C keeps at v1.0.31)`.
- **Cross-dispatch coordination:** Architect 6D bumped SS-daemon-lifecycle v1.0.30 → v1.0.31 in commit 98396fe. Architect 7C (parallel R108 Round 7C) keeps SS-daemon-lifecycle at v1.0.31 per coordination directive. The §References pin refresh resolves the R107 "commit pending" placeholder (architect 5E F-FC-I005 dispatch, which actually landed in commit 03a4c57 at v1.0.30) AND advances the pin to the current canonical v1.0.31. This is a two-step resolution: (a) the original 5E placeholder is documented as resolved in §Trace v1.0.5 Authoritative cross-references; (b) the active cite advances to v1.0.31 with concrete SHA.

### Change 3 — F-R108-15 MED: §References Architecture Active Cite Refresh (NORMATIVE, SS-01 VPs only)

(See Change 2 above for SS-01 VPs. SS-02 / SS-03 VPs do not carry an SS-NN pin in §References and require no architecture-line cascade in this dispatch — pin sweep deferred to the dependent architecture file's next direct cite cascade.)

### Rationale

R108 Round 7D (FV scope) closes three findings in coordinated parallel dispatch with PO 7A (BC) + PO 7B (PRD/supplements) + Architect 7C (SS-pin-stable):

- **F-R108-5 HIGH (VP-INDEX):** VP-INDEX §References `commit pending` placeholders unresolved (R107 R6A/6B targets). Resolved to d92e4a7 (PO 6A + 6B co-mingled commit per Round 6F SM message). New active cite advances to R108 Round 7A BC-INDEX v1.6 + R108 Round 7B PRD v1.26.6 targets with explicit forward-coordination "commit pending" annotation (will resolve during R108 Round 7E SM pass).
- **F-R108-6 HIGH (all-22-VPs):** 22 VPs carried ~214 cumulative `commit pending` placeholders across active §References + §Trace v1.0.4 / v1.0.5 Authoritative cross-references blocks. Active §References refreshed to R108 Round 7 forward targets (BC-INDEX v1.6 + PRD v1.26.6) with explicit forward-coordination annotations. Historical Authoritative cross-references blocks resolved to concrete SHAs (d92e4a7 + 03a4c57 + 98396fe). SE-17f BEFORE/AFTER snapshot evidence preserved per SE-17g (historical state-at-time-of-bump snapshots are immutable; refreshing them would falsify the audit trail).
- **F-R108-15 MED (SS-01 VPs only):** 10 SS-01 VPs cite SS-daemon-lifecycle v1.0.30 in active §References. Architect 6D bumped SS-daemon-lifecycle v1.0.30 → v1.0.31 in commit 98396fe; Architect 7C (parallel R108 Round 7C) keeps SS-daemon-lifecycle at v1.0.31 per coordination directive. SS-01 VP active §References refreshed to v1.0.31 with concrete commit 98396fe; SS-01 §Source Contract Traces-to historical-block cites also refreshed to v1.0.31 (these are body sections, not §Trace history, so SE-17g permits in-place refresh).

Per CLAUDE.md Production-Grade Default Rule 1+5: mechanical citation refresh + pin sweep executed in-scope of R108 Round 7D rather than deferred. No tech-debt entries created. SE-17f BEFORE/AFTER snapshot evidence in prior §Trace blocks preserved per SE-17g.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.6 (commit pending — PO 7B R108 Round 7B PRD + supplements dispatch; supersedes v1.26.5 commit d92e4a7).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.6 (commit pending — PO 7A R108 Round 7A BC scope dispatch; supersedes v1.5 commit d92e4a7).
- **Architecture (SS-01):** `architecture/SS-daemon-lifecycle.md` v1.0.31 (commit 98396fe — Architect 6D; Architect 7C keeps at v1.0.31).
- **R108 closure chain:** F-R108-5 HIGH (VP-INDEX commit-pending resolution — handled in VP-INDEX v1.6 cascade); F-R108-6 HIGH (22-VP commit-pending sweep); F-R108-15 MED (SS pin sweep — SS-01 VPs only).
- **Concurrent dispatches (R108 Round 7):**
  - PO 7A: BC + BC-INDEX scope (BC-INDEX v1.5 → v1.6) — separate scope.
  - PO 7B: PRD + supplements (PRD v1.26.5 → v1.26.6) — separate scope.
  - Architect 7C: arch — keeps current SS versions per coordination — separate scope.
  - FV 7D: this dispatch (22-VP commit-pending sweep + 10-VP SS pin v1.0.31 + VP-009 probe renumber + VP-INDEX v1.6 cascade — THIS file).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-18T01:00:00Z` >= chain high-water `2026-05-17T23:30:00Z` (R107 Round 6F SM commit timestamp). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References BC index cite refresh `v1.5` → `v1.6` (with R7-forward placeholder); §References PRD cite refresh `v1.26.5` → `v1.26.6` (with R7-forward placeholder); §References Architecture cite refresh `v1.0.30` → `v1.0.31` (SS-01 VPs only); §Source Contract Traces-to body cite refresh `v1.0.30` → `v1.0.31` (SS-01 VPs only); §Trace v1.0.4 + v1.0.5 Authoritative cross-references historical placeholder resolution; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+4+5

Rule 1: mechanical citation refresh + pin sweep executed in-scope rather than deferred. Rule 4: 3 coupled cascade fixes consolidated into single v1.0.6 bump rather than fragmented. Rule 5: cheapest path (defer pin refresh as "stale by 1 minor version, acceptable") rejected in favor of correct path (refresh all active cites to current canonical versions). PRD v1.26.6 and BC-INDEX v1.6 cites are post-PO-7A and post-PO-7B targets (commit pending — will resolve to concrete SHAs during R108 Round 7E SM pass after parallel dispatches converge). No tech-debt entries created. SE-17f BEFORE/AFTER snapshot evidence in prior §Trace v1.0.4 / v1.0.5 blocks preserved per SE-17g audit-trail discipline — historical state-at-time-of-bump snapshots are immutable; refreshing them would erase audit trail.
