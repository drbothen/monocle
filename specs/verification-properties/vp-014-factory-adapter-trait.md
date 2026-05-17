---
document_type: verification-property
level: L4
version: "1.0.3"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T20:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.02.004
module: monocle-core
proof_method: ast-audit
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

# VP-014: `FactoryAdapter` Trait Signature Stable

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-FACTORY-001 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

1. The trait `monocle_core::factory::FactoryAdapter` exists with the exact
   method set: `detect`, `matches`, `state_file_path`, `read_state`,
   `subscribe`, `display_name`, `abi_version`.
2. The trait's super-bounds are exactly `Send + Sync + 'static` — no
   `private::Sealed` (or any other sealing) supertrait appears.
3. The supporting types `{FactoryDetection, FactoryState, BlockingIssue,
   BlockingSeverity, ConvergenceMetrics, FactoryReadError,
   FactorySubscribeError, StateChangeStream}` are all `pub` and accessible
   from `monocle_core::factory::*`.
4. `FactoryState` has the 7 canonical fields:
   `{ phase: String, status: String, awaiting: Option<String>,
   blocking_issues: Vec<BlockingIssue>,
   convergence: Option<ConvergenceMetrics>, cycle: Option<String>,
   custom_fields: HashMap<String, serde_yaml_ng::Value> }`.

## Source Contract

- **BC:** BC-2.02.004 — `FactoryAdapter` Trait Surface (Open, No Sealed Bound).
- **Postcondition/Invariant:** BC-2.02.004 invariants 1-4 asserting trait
  method count + names, open super-bounds (no `Sealed`), supporting type
  visibility, and `FactoryState` field-name set. Open-trait rationale
  referenced by VP-019 (`EngineModule` parallel trait).
- **Traces to (historical):** BC-FACTORY-001 (SS-core-types-and-abi.md
  §FactoryAdapter Trait; PRD v1.25 §BC-FACTORY-001 Verification subsection).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| AST audit (primary) | syn 2 | Bounded — finite trait surface | Trait method-set HashSet equality; super-bound token-stream; `Sealed` substring absence; `FactoryState` field-set HashSet equality |
| `cargo check` (supplementary) | cargo | N/A — static | Supporting type re-exports compile via probe file |

## Mechanism

AST audit (specifically a `cargo check` + `syn 2` parse over the public
trait surface in `monocle-core/tests/factory_trait_surface.rs`; PRD v1.25
§7 RTM Test Type column labels this BC `AST audit (syn 2)`). The harness
parses `monocle-core/src/factory.rs`, locates the `trait FactoryAdapter`
item, and walks the method set + super-bound token-stream + supporting
type re-exports.

## Pre-conditions

- `monocle-core` builds cleanly.
- `rustdoc` JSON output is available via `cargo +nightly rustdoc -- -Z
  unstable-options --output-format json` OR equivalent stable `cargo
  doc` parsing.

## Post-conditions

1. `cargo check --workspace` passes.
2. A `monocle-core/tests/factory_trait_surface.rs` test uses `syn 2` to
   parse `monocle-core/src/factory.rs`, locates the `trait FactoryAdapter`
   item, and asserts:
   - method count equals 7;
   - method names match the canonical set (HashSet equality);
   - super-trait bounds equal `Send + Sync + 'static` (token-stream match);
   - no `Sealed` identifier appears anywhere in the trait declaration.
3. A `FactoryState` field-name check asserts the HashSet of field
   identifiers equals the 7-field canonical set above.

## Counter-examples

1. A future refactor adds a `Sealed` supertrait — must fail the substring
   check.
2. A method is renamed (e.g., `display_name` → `name`) — must fail the
   canonical-method-set HashSet equality.
3. A new method `priority` is added without a default body — must fail
   the method count check; a method added WITH a default body is
   permitted per SS-core-types-and-abi.md §Forward Compatibility
   Guarantees, so the audit must distinguish defaulted vs non-defaulted
   methods (the `has_block` check on the `TraitItemFn` syn node
   distinguishes them).
4. A `FactoryState` field is renamed (e.g., `phase` → `pipeline_phase`)
   — must fail the field-name HashSet equality.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 14.a | Walk `trait FactoryAdapter` items via `syn 2` | method count == 7; names HashSet == canonical set |
| 14.b | Token-stream match super-bound | equals `Send + Sync + 'static` exactly |
| 14.c | Substring scan trait body | no `Sealed` identifier |
| 14.d | Walk `struct FactoryState` field-set | 7 fields HashSet match |
| 14.e | Inject `pub trait FactoryAdapter: Sealed { ... }` | Probe 14.c fails |
| 14.f | Inject defaulted method `fn priority(&self) -> u8 { 0 }` | Audit distinguishes defaulted vs non-defaulted; count still passes (defaulted methods are forward-compat-safe per arch §Forward Compatibility) |

## Harness Location

- `monocle-core/tests/factory_trait_surface.rs` (AST audit)
- Test name: `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound`
  (per PRD v1.25 §BC-FACTORY-001, Verification subsection — to be
  migrated to `test_BC_2_02_004_trait_defined_open_no_sealed_bound`
  post BC renumber propagation into source).

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: ast-audit
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_02_004() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-core/tests/factory_trait_surface.rs` (AST audit)
- Test name: `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound`
  (per PRD v1.25 §BC-FACTORY-001, Verification subsection — to be
  migrated to `test_BC_2_02_004_trait_defined_open_no_sealed_bound`
  post BC renumber propagation into source).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: ast-audit` per frontmatter; mechanism documented in `## Mechanism` section. |
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
- Predecessor: monolithic VP-FACTORY-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.004.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §FactoryAdapter Trait.
- Forward compatibility: `architecture/SS-forward-compatibility.md`
  §Item P3-1 — Verdict on Sealed (open-trait rationale).
- PRD: `.factory/specs/prd.md` v1.26.3 §BC-2.02.004 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).
- Cross-VP: VP-015 (`VsddFactoryAdapter` constructor + self-referential detection);
  VP-019 (`EngineModule` parallel trait — same open-trait property class).

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

## §Trace v1.0.2 — F-R105-10 MED: VP-014 Title Sync to VP-INDEX Canonical

**Bump:** v1.0.1 → v1.0.2.
**Predecessor pin:** v1.0.1 (commit 4090d0b — RES-03 VP heading reconciliation).
**Scope of v1.0.2 (NORMATIVE — title sync to VP-INDEX canonical; NO substantive content change):**

### Change set (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** body H1 heading at line 33: `# VP-014: \`FactoryAdapter\` Trait Signature Stable; No Sealed Bound` (pre-edit grep). VP-INDEX.md v1.1 row reads: `VP-014 | \`FactoryAdapter\` Trait Signature Stable`.
  - **After:** body H1 heading at line 33: `# VP-014: \`FactoryAdapter\` Trait Signature Stable` — exact match with VP-INDEX canonical title (post-edit grep).
- **SE-17c-d body-scope grep:** `grep -n "# VP-014:" vp-014-factory-adapter-trait.md` post-edit returns the single H1 with exact-match title.

### Rationale

VP-INDEX.md v1.1 §SS-02 table (Dispatch 5b, authored 2026-05-17T13:30:00Z) is the canonical source of truth for all VP titles per its frontmatter `> Source of truth for all verification property IDs, titles, source BCs, proof methods, and file paths.` and per the closure chain dispatch directive `default to VP-INDEX as canonical (it was authored in D-122 D5 dispatch with explicit VP renumbering)`.

The pre-edit body H1 had an extended title (`; No Sealed Bound` suffix) that diverged from the VP-INDEX canonical short form. The divergence was a pre-Dispatch-5b residual; the substantive content (Property Statement invariant 2 — `private::Sealed` absence assertion) is preserved in the body Property Statement, Counter-examples §1, and BC-2.02.004 cross-reference (`FactoryAdapter Trait Surface (Open, No Sealed Bound)`). The H1 title is the discoverability label; the substantive "no sealed bound" content remains fully traceable through the body sections and the BC linkage.

### Authoritative cross-references

- **VP-INDEX:** `verification-properties/VP-INDEX.md` v1.1 §SS-02 row `VP-014 | \`FactoryAdapter\` Trait Signature Stable | BC-2.02.004 | ast-audit | vp-014-factory-adapter-trait.md | VP-FACTORY-001`.
- **BC:** `behavioral-contracts/ss-02/BC-2.02.004.md` — `FactoryAdapter Trait Surface (Open, No Sealed Bound)` (BC carries the full "no sealed bound" semantics).
- **R105 closure chain:** F-R105-10 MED — VP-014 title sync.
- **Concurrent dispatch:** T-128g FV portion — F-R105-7 MED manifest pin refresh across 14 pin-citing VP files (this VP is NOT in the pin-citing set; T-128g leaves VP-014 untouched aside from this T-128j title sync).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T19:30:00Z` >= chain high-water `2026-05-17T19:00:00Z` (nfr-catalog.md). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: H1 title sync to VP-INDEX canonical; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection (explaining substantive-content preservation through body sections + BC linkage); cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Synced the H1 to VP-INDEX canonical in-scope of T-128j rather than deferring or routing through a parallel dispatch. The VP-INDEX-as-source-of-truth discipline (established in Dispatch 5b) authorized this mechanical title sync without requiring per-VP content review. The substantive "no sealed bound" semantics remain fully preserved in the body Property Statement, Counter-examples, and BC linkage; no tech-debt entries created.

---

## §Trace v1.0.3 — F-R105-13 LOW: VP §References PRD Citation Refresh v1.26 → v1.26.3

**Bump:** v1.0.2 → v1.0.3.
**Predecessor pin:** v1.0.2 (commit 927fcce — T-128g+T-128j FV — F-R105-7/10/11 (pin refresh + title sync + sister-VP reconciliation)).
**Scope of v1.0.3 (NORMATIVE — §References PRD citation refresh; NO content cascade; NO BC-path changes — BC §References already cite canonical sharded `behavioral-contracts/ss-NN/BC-2.SS.NNN.md` paths):**

### Change set 1 — §References PRD Citation Refresh `v1.26` → `v1.26.3` (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** §References cited `prd.md v1.26 §BC-2.02.004 (Dispatch 4 commit 1030c65).` (pre-edit grep).
  - **After:** §References cites `prd.md v1.26.3 §BC-2.02.004 (Dispatch 4 commit 1030c65; refreshed to v1.26.3 in F-R105-12 closure, parallel PO commit b2b378b).` (post-edit grep).
- **SE-17c-d body-scope grep:** post-edit `grep -n "prd.md v1.26 " vp-014-factory-adapter-trait.md` → 0 matches; `grep -n "prd.md v1.26.3" vp-014-factory-adapter-trait.md` → 1 match (§References line).
- **BC §References scope:** §References §Source contract entry already cites canonical sharded `behavioral-contracts/ss-02/BC-2.02.004.md` (per BC-INDEX.md v1.2). No BC-path changes required in this dispatch.
- **Historical PRD v1.25 citations in body prose (Source Contract `Traces to (historical)`, Harness Location `to be migrated to`, Proof Harness Skeleton `to be migrated to`, where present):** UNCHANGED — these are explicitly historical predecessor citations pinned to the pre-Dispatch-4 PRD monolith and must not be refreshed.

### Rationale

PO commit b2b378b (T-128k Round-3 PO dispatch) bumped PRD `v1.26.2 → v1.26.3` for F-R105-12 VP alias + GAP-R44-4 closure. Parallel FV dispatch refreshes VP §References to cite the post-bump PRD version, preserving the stale-citation-zero invariant established in F-R105-7 (manifest pin refresh) and F-R105-11 (sister-VP reference reconciliation). Per CLAUDE.md Production-Grade Rule 1: no MVP-driven deferral; mechanical citation refresh executed in-scope of T-128k FV portion rather than left to post-Round-3 cleanup.

### Authoritative cross-references

- **PRD:** `.factory/specs/prd.md` v1.26.3 (commit b2b378b — F-R105-12 VP alias + GAP-R44-4 closure).
- **BC-INDEX:** `behavioral-contracts/BC-INDEX.md` v1.2 (commit 61133a7 — confirms canonical sharded path `ss-02/BC-2.02.004.md` for BC-2.02.004).
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
