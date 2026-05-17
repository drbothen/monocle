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
source_bc: BC-2.02.003
module: monocle-core
proof_method: ast-audit+mutation-test
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

# VP-013: Non-Exhaustive Enum Policy (Modulo ADR-0004 Exemptions)

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-TYPES-001 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

For every `pub enum E` defined in any source file of the `monocle-core`
crate, exactly one of the following holds:

1. `E` carries `#[non_exhaustive]`, OR
2. `E` is listed in the ADR-0004 exemption set
   `{ "Phase1Permission", "ClaudeCodeTool" }`.

No other exemption is allowed without a new ADR superseding ADR-0004.

## Source Contract

- **BC:** BC-2.02.003 — Enum Extensibility (Non-Exhaustive Default Policy).
- **Postcondition/Invariant:** BC-2.02.003 invariant 1 (every `pub enum` in
  `monocle-core` is `#[non_exhaustive]` modulo the explicit ADR-0004
  exemption set); supporting clippy lint configuration `non_exhaustive_omitted_patterns`
  deny-listed at workspace level.
- **Traces to (historical):** BC-TYPES-001 (SS-core-types-and-abi.md
  §Enum Extensibility; PRD v1.25 §BC-TYPES-001 Verification subsection).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| AST audit (primary) | syn 2 | Bounded — finite enum set in `monocle-core/src/**/*.rs` | All `pub enum` declarations in `monocle-core` |
| Mutation testing (auxiliary) | cargo-mutants | Bounded — mutation budget | `EXEMPT` constant length + `#[non_exhaustive]` attribute-presence check are mutation surfaces |
| Clippy lint (supplementary) | cargo clippy | N/A — static | `non_exhaustive_omitted_patterns` deny-listed for `#[allow]` (per SS-conventions-anti-patterns.md) |

## Mechanism

AST audit (primary, via a `syn 2` AST audit harness at
`monocle-core/tests/enum_audit.rs` parsing every
`monocle-core/src/**/*.rs` file and walking all `Item::Enum` nodes; per
PRD v1.25 §BC-TYPES-001 invariant 1; PRD v1.25 §7 RTM Test Type column
labels this BC `AST audit (syn 2)`); mutation-test (auxiliary); clippy
`non_exhaustive_omitted_patterns` lint configuration (supplementary).

## Pre-conditions

- `monocle-core` source tree is the audit scope.
- The exempt-list constant in the test harness is
  `EXEMPT: &[&str] = &["Phase1Permission", "ClaudeCodeTool"]`.

## Post-conditions

1. A test harness in `monocle-core/tests/enum_audit.rs` parses every
   `monocle-core/src/**/*.rs` file via `syn 2`, walks all `Item::Enum`
   nodes, and asserts that for each enum either `#[non_exhaustive]` is
   present in the attribute list OR the enum's identifier is in
   `EXEMPT`.
2. The test fails with a descriptive error listing every offending enum
   if the property is violated.
3. The `cargo clippy --workspace -- -D warnings` invocation passes with
   the project-local lint `non_exhaustive_omitted_patterns` deny-listed
   for `#[allow]` (per SS-conventions-anti-patterns.md).

## Counter-examples

1. A new contributor adds `pub enum NewError { ... }` to
   `monocle-core/src/` without `#[non_exhaustive]` and not in the
   exempt list — must fail the audit.
2. A contributor sneaks `#[allow(non_exhaustive_omitted_patterns)]` into
   a match site — must fail the clippy step (semgrep rule co-enforces,
   per SS-conventions-anti-patterns.md §Semgrep Rules).
3. A contributor adds `pub enum Phase2Permission` to `monocle-core/`
   (NOT in `monocle-plugin-sdk`) without `#[non_exhaustive]` and not in
   the exempt list — must fail the audit. (Even though ADR-0004
   contemplates a parallel `Phase 3` enum in the plugin SDK, that enum
   is in a different crate and the audit is `monocle-core`-scoped.)
4. Exempt list expanded silently to add a third enum without an ADR
   superseding ADR-0004 — covered by an orthogonal consistency check
   that greps for the `EXEMPT` constant length and asserts it equals
   the count of exhaustive enums documented in ADR-0004 (currently 2).

## Mutation-Test Rationale

Mutating the `EXEMPT` constant length (e.g., adding a stray entry) or
the `#[non_exhaustive]` attribute presence check (e.g., flipping
`has_attr` to `!has_attr`) must be caught by the audit harness — this
is a high-leverage mutation surface. `cargo-mutants` is the auxiliary
mechanism: a mutant that flips the attribute-presence check or grows
the exempt list silently must be killed by an existing failing test.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 13.a | Walk all `pub enum` in `monocle-core/src/**/*.rs` | Each is `#[non_exhaustive]` OR in EXEMPT |
| 13.b | Inject `pub enum NewError { ... }` without `#[non_exhaustive]` | Audit fails with descriptive error |
| 13.c | Inject `#[allow(non_exhaustive_omitted_patterns)]` | Clippy step fails |
| 13.d | Expand EXEMPT list silently | Consistency check fails (length mismatch with ADR-0004 documented count) |
| 13.e | cargo-mutants flips `has_attr` to `!has_attr` | Mutation killed by audit failure |

## Harness Location

- `monocle-core/tests/enum_audit.rs` (AST audit)
- Test name: `test_BC_TYPES_001_non_exhaustive_enum_coverage` (per PRD
  v1.25 §BC-TYPES-001, Verification subsection — to be migrated to
  `test_BC_2_02_003_non_exhaustive_enum_coverage` post BC renumber
  propagation into source).

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: ast-audit+mutation-test
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_02_003() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-core/tests/enum_audit.rs` (AST audit)
- Test name: `test_BC_TYPES_001_non_exhaustive_enum_coverage` (per PRD
  v1.25 §BC-TYPES-001, Verification subsection — to be migrated to
  `test_BC_2_02_003_non_exhaustive_enum_coverage` post BC renumber
  propagation into source).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: ast-audit+mutation-test` per frontmatter; mechanism documented in `## Mechanism` section. |
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
- Predecessor: monolithic VP-TYPES-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.003.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §Enum Extensibility.
- ADR: `architecture/adr/ADR-0004` (non-exhaustive default policy with
  enumerated exemption set).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.003 (Dispatch 4 commit 1030c65).
- Conventions: `architecture/SS-conventions-anti-patterns.md` §Semgrep Rules.

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
- **Project-specific additional sections preserved unchanged:** ## Mutation-Test Rationale.


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
