---
document_type: verification-property
level: L4
version: "1.0.2"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T20:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "7c094e3"
traces_to: prd.md
source_bc: BC-2.02.005
module: monocle-core
proof_method: integration-test+fuzz
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

# VP-015: `VsddFactoryAdapter::new` + Self-Referential Detection; `None` for Absent Optionals

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-FACTORY-002 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

1. A public constructor `VsddFactoryAdapter::new(workspace_root: PathBuf)
   -> Self` exists.
2. The constructor performs NO validation (it does not panic, does not
   error, does not stat the filesystem); validation happens in `detect()`
   and `read_state()`.
3. The static method `VsddFactoryAdapter::detect(<monocle repo root>)`
   returns `Some(FactoryDetection)` where `display_name == "VSDD Factory"`
   (self-referential test).
4. For a `FactoryState` produced from a STATE.md file lacking
   `current_cycle:`, `state.cycle == None` (NOT `Some("unknown")` or any
   placeholder string).
5. For a `FactoryState` produced from a STATE.md file lacking a §Session
   Resume Checkpoint section, `state.convergence == None`.

## Source Contract

- **BC:** BC-2.02.005 — `VsddFactoryAdapter` (Phase 1 Reference Adapter
  + Frontmatter Parsing Contract).
- **Postcondition/Invariant:** BC-2.02.005 invariants 1-5 plus
  §Edge Case EC-061 (present-but-empty `current_cycle: ""` collapses to
  `None`); cross-property with VP-013 (orthogonal exhaustive-enum
  concern noted in monolithic line 1247).
- **Traces to (historical):** BC-FACTORY-002 (SS-core-types-and-abi.md
  §FactoryAdapter Trait; PRD v1.25 §BC-FACTORY-002 Verification subsection).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test | Bounded — finite fixture set | Self-referential `detect()`; `None`/`Some` discrimination across 4 fixture STATE.md files; constructor no-op invariant |
| Fuzz (auxiliary) | cargo-fuzz | Unbounded UTF-8 byte space | `parse_frontmatter_field` + `parse_frontmatter_extra_fields` no-panic / no-OOM / flow-style + block-scalar skip invariants |

## Mechanism

Integration test (primary; harness located at
`monocle-core/tests/factory_self_referential.rs` — files in
`<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test Type
column labels this BC `Integration`); fuzz (auxiliary) via
`cargo fuzz add fuzz_state_md_parser` feeding arbitrary UTF-8 byte
sequences into `parse_frontmatter_field` + `parse_frontmatter_extra_fields`
and asserting no-panic / no-OOM / flow-style + block-scalar skip
invariants.

## Pre-conditions

- `monocle-core` builds cleanly.
- Test fixture STATE.md files are checked in under
  `monocle-core/tests/fixtures/`:
  - `state_minimal.md` — has `current_cycle:` and §Session Resume Checkpoint.
  - `state_no_cycle.md` — lacks `current_cycle:` frontmatter key.
  - `state_no_checkpoint.md` — has `current_cycle:` but lacks §Session
    Resume Checkpoint section.
  - `state_empty_cycle.md` — has `current_cycle: ""` (present-but-empty
    string) — drives the §Post-condition 7 empty-string probe per
    I-R90-4 closure.

## Post-conditions

1. `VsddFactoryAdapter::new(PathBuf::from("/nonexistent/path"))` returns
   a value without error or panic.
2. `VsddFactoryAdapter::detect(&monocle_repo_root)` returns `Some(d)`
   where `d.display_name == "VSDD Factory"` and `d.state_file ==
   monocle_repo_root.join(".factory/STATE.md")`.
3. For fixture `state_no_cycle.md`, `state.cycle.is_none()`.
4. For fixture `state_no_checkpoint.md`, `state.convergence.is_none()`.
5. For fixture `state_minimal.md`, `state.cycle == Some("cycle-001".into())`
   (or whatever the fixture declares) — proves the `None` cases are
   discriminating, not a vacuous default.
6. `parse_frontmatter_field` returns `None` for: empty values, flow-style
   lists (`[...]`), block scalars (`|` or `>` lead), and continuation
   lines.
7. **Empty-string `current_cycle` fixture (per BC-2.02.005 §Edge Case
   EC-061):** For STATE.md frontmatter with `current_cycle: ""`
   (present-but-empty), `state.cycle` MUST equal `None` (NOT
   `Some("".into())`). Verified by integration test loading the
   empty-fixture and asserting `state.cycle.is_none()`. Cross-property
   with `parse_frontmatter_field` empty-value handling (§Post-condition
   6) — the empty-value form is the canonical "absence of meaningful
   content" path and MUST collapse to `None` at the `state.cycle`
   surface, not just at the `parse_frontmatter_field` return.

## Counter-examples

1. The constructor stats `workspace_root` and panics on absent path —
   fails post-condition 1.
2. `read_state` substitutes `"unknown"` for absent `current_cycle:` —
   fails post-condition 3.
3. `parse_frontmatter_field` accepts `awaiting: [a, b]` and returns
   `Some("[a, b]")` — fails post-condition 6 (the v1.2.3 round-20 fix
   site; this VP's permanent fuzz harness prevents recurrence).
4. `parse_frontmatter_field` accepts `phase: |` block-scalar marker and
   returns `Some("|")` — fails post-condition 6.
5. Self-referential detect fails because the `document_type:
   pipeline-state` substring check is too strict (e.g., requires exact
   line equality including trailing whitespace) — fails post-condition 2.
6. **Probe-matrix exhaustiveness regression (empty-string
   `current_cycle`):** `read_state` returns `Some("".into())` for
   frontmatter `current_cycle: ""` (present-but-empty value) instead of
   collapsing to `None` — fails post-condition 7. This is a regression
   class where an implementer treats "absent key" and "present-but-empty
   value" as different paths but only the absent-key path correctly
   yields `None`; the present-but-empty path leaks through as `Some("")`.
   `cargo-mutants` mutation-test rationale: this probe is the leverage
   point for the asymmetric `None`-collapse mutation surface in the
   frontmatter parser.

## Fuzz Harness

`cargo fuzz add fuzz_state_md_parser`. The fuzz target feeds arbitrary
UTF-8 byte sequences into `parse_frontmatter_field(content, "phase")` and
`parse_frontmatter_extra_fields(content, &known_keys)` and asserts: no
panic; no allocation > 1 MiB; flow-style and block-scalar inputs produce
`None` (frontmatter_field) or are skipped (extra_fields). The fuzzer is
seeded with `state_minimal.md`, `state_no_cycle.md`, `state_empty_cycle.md`,
and adversarial malformed corpora (truncated frontmatter, mismatched
quotes, Unicode direction overrides, deep nesting markers). Permanent
fuzz harness — prevents recurrence of the v1.2.3 (F-R20-2) regression
site.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 15.a | `new(PathBuf::from("/nonexistent"))` | No panic; no error; returns adapter |
| 15.b | `detect(&monocle_repo_root)` from `state_minimal.md` | `Some(d)`; `d.display_name == "VSDD Factory"` |
| 15.c | `read_state` over `state_no_cycle.md` | `state.cycle.is_none()` |
| 15.d | `read_state` over `state_no_checkpoint.md` | `state.convergence.is_none()` |
| 15.e | `read_state` over `state_minimal.md` | `state.cycle == Some("cycle-001".into())` |
| 15.f | `parse_frontmatter_field` over `awaiting: [a, b]` | Returns `None` (flow-style reject) |
| 15.g | `parse_frontmatter_field` over `phase: \|` | Returns `None` (block-scalar reject) |
| 15.h | `read_state` over `state_empty_cycle.md` (`current_cycle: ""`) | `state.cycle.is_none()` — present-but-empty collapses to `None` (EC-061) |
| 15.i | Fuzz: arbitrary UTF-8 input | No panic; no >1 MiB allocation |

## Harness Location

- `monocle-core/tests/factory_self_referential.rs` (integration test)
- `fuzz/fuzz_targets/fuzz_state_md_parser.rs` (fuzz harness)
- Test name: `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection`
  (per PRD v1.25 §BC-FACTORY-002, Verification subsection — to be
  migrated to `test_BC_2_02_005_vsdd_adapter_self_referential_detection`
  post BC renumber propagation into source).

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: integration-test+fuzz
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_02_005() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-core/tests/factory_self_referential.rs` (integration test)
- `fuzz/fuzz_targets/fuzz_state_md_parser.rs` (fuzz harness)
- Test name: `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection`
  (per PRD v1.25 §BC-FACTORY-002, Verification subsection — to be
  migrated to `test_BC_2_02_005_vsdd_adapter_self_referential_detection`
  post BC renumber propagation into source).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: integration-test+fuzz` per frontmatter; mechanism documented in `## Mechanism` section. |
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
- Predecessor: monolithic VP-FACTORY-002 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.005.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §FactoryAdapter Trait.
- PRD: `.factory/specs/prd.md` v1.26.3 §BC-2.02.005 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
- Cross-VP: VP-013 (orthogonal exhaustive-enum concern for `BlockingSeverity` etc.);
  VP-014 (trait surface this adapter implements).
- Mutation-test pairing: per monolithic §Coverage Matrix, `fuzz` is the
  auxiliary mechanism here because `parse_frontmatter_field` was the
  v1.2.3 (F-R20-2) regression site; permanent fuzz harness prevents
  recurrence.

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
- **Project-specific additional sections preserved unchanged:** ## Fuzz Harness.


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

## §Trace v1.0.2 — F-R105-13 LOW: VP §References PRD Citation Refresh v1.26 → v1.26.3

**Bump:** v1.0.1 → v1.0.2.
**Predecessor pin:** v1.0.1 (commit 4090d0b — Audit R2 Residual RES-03: VP Heading Reconciliation to L4 Template).
**Scope of v1.0.2 (NORMATIVE — §References PRD citation refresh; NO content cascade; NO BC-path changes — BC §References already cite canonical sharded `behavioral-contracts/ss-NN/BC-2.SS.NNN.md` paths):**

### Change set 1 — §References PRD Citation Refresh `v1.26` → `v1.26.3` (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** §References cited `prd.md v1.26 §BC-2.02.005 (Dispatch 4 commit 1030c65).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.3 §BC-2.02.005 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (post-edit grep).
- **SE-17c-d body-scope grep:** post-edit `grep -n "prd.md v1.26 " vp-015-vsdd-factory-adapter.md` → 0 matches; `grep -n "prd.md v1.26.3" vp-015-vsdd-factory-adapter.md` → 1 match (§References line).
- **BC §References scope:** §References §Source contract entry already cites canonical sharded `behavioral-contracts/ss-02/BC-2.02.005.md` (per BC-INDEX.md v1.2). No BC-path changes required in this dispatch.
- **Historical PRD v1.25 citations in body prose (Source Contract `Traces to (historical)`, Harness Location `to be migrated to`, Proof Harness Skeleton `to be migrated to`, where present):** UNCHANGED — these are explicitly historical predecessor citations pinned to the pre-Dispatch-4 PRD monolith and must not be refreshed.

### Rationale

PO commit b2b378b (T-128k Round-3 PO dispatch) bumped PRD `v1.26.2 → v1.26.3` for F-R105-12 VP alias + GAP-R44-4 closure. Parallel FV dispatch refreshes VP §References to cite the post-bump PRD version, preserving the stale-citation-zero invariant established in F-R105-7 (manifest pin refresh) and F-R105-11 (sister-VP reference reconciliation). Per CLAUDE.md Production-Grade Rule 1: no MVP-driven deferral; mechanical citation refresh executed in-scope of T-128k FV portion rather than left to post-Round-3 cleanup.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.3 (commit b2b378b — F-R105-12 VP alias + GAP-R44-4 closure).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7 — confirms canonical sharded path `ss-02/BC-2.02.005.md` for BC-2.02.005).
- **R105 closure chain:** F-R105-13 LOW — 22-VP §References PRD citation refresh sweep.
- **Concurrent dispatches (T-128k Round 3):**
  - PO: PRD v1.26.2 → v1.26.3 (F-R105-12 + GAP-R44-4) — COMPLETE (commit b2b378b).
  - architect: auth-header interop adjudication — separate scope.
  - BA: L2-INDEX anchor fixes — separate scope.
  - FV: this §Trace (F-R105-13 — 22-VP §References PRD citation refresh).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T20:30:00Z` >= chain high-water `2026-05-17T19:30:00Z` (this VP's prior v1.0.1 §Trace and PRD v1.26.3 frontmatter). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: §References PRD citation `v1.26` → `v1.26.3`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### SE-17g META audit (NORMATIVE)

Sweep-wide post-edit re-grep across all 22 VP files: `grep -rE "prd\.md v1\.(26[^.]|26\.[012])(\s|\$)" .factory/specs/verification-properties/vp-*.md` → 0 matches. Sweep-wide re-grep for non-sharded BC paths: `grep -rE "behavioral-contracts/BC-[^I]" .factory/specs/verification-properties/vp-*.md` → 0 matches. F-R105-13 closure verified.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Mechanical citation refresh executed in-scope rather than deferred. PRD v1.26.3 cite is valid as of PO commit b2b378b. No tech-debt entries created. Body-prose historical PRD v1.25 citations preserved unchanged per Production-Grade discipline (historical predecessor citations are not stale; refreshing them would erase audit trail).
