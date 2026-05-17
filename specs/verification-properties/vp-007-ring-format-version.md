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
source_bc: BC-2.01.007
module: monocle-runtime
proof_method: manual+mutation
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

# VP-007: JSONL Ring Record — Format-Version First Key (FC-01)

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-RING-001 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

For every `HookEventRecord` constructed via `HookEventRecord::new(...)`,
`serde_json::to_string(&record)` produces a JSON string whose first key after
the opening `{` is `"format_version":1`. Formally:

```
forall record: HookEventRecord constructed via HookEventRecord::new(...),
  serde_json::to_string(&record).unwrap().starts_with("{\"format_version\":1,")
```

For hook types with no tool surface (`SessionStart`, `UserPromptSubmit`,
`Stop`), the serialized output MUST NOT contain `"tool_name":null` or
`"tool_input":null` substrings — these fields are omitted entirely via
`#[serde(skip_serializing_if = "Option::is_none")]`.

## Source Contract

- **BC:** BC-2.01.007 — JSONL Ring Format Version (FC-01).
- **Postcondition/Invariant:** declaration-order serialization with
  `format_version` first; round-trip preservation of
  `format_version == 1`; absence-of-field for `None`-valued `tool_name`
  / `tool_input` (EC-001 normative `skip_serializing_if` pin).
- **Traces to (historical):** BC-RING-001 (SS-daemon-lifecycle.md §Drain).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test | Bounded — finite hook-type set | First-key invariant; round-trip; absence-of-field assertion |
| Mutation test (auxiliary) | cargo-mutants | N/A — mutation surface | `format_version: u32 = 1` value mutation; `#[serde(skip_serializing_if = "Option::is_none")]` attribute strip |

## Mechanism

Integration test (primary; harness at `monocle-runtime/tests/jsonl_ring.rs`
— cargo integration test per file location; PRD v1.25 §7 RTM Test Type
column labels this BC `Integration` consistent with the harness file
layout); mutation-test (auxiliary). The integration harness constructs
`HookEventRecord` instances for each `HookType` variant (including the 3
no-tool-surface variants `SessionStart`, `UserPromptSubmit`, `Stop`) and
asserts the serialized-prefix literal match plus the absence-of-field
substring assertions for the `None` Options. `cargo-mutants` is configured
to attempt mutations on the `format_version: u32` value `1` and on the
`#[serde(skip_serializing_if = ...)]` attribute.

## Pre-conditions

- `RING_FORMAT_VERSION` const equals `1` (loaded from `monocle-runtime::ring`).
- `HookEventRecord` carries `#[non_exhaustive]` (otherwise the constructor
  contract is moot — see VP-013 (renumbered from VP-TYPES-001 per
  VP-INDEX.md §Renumbering Appendix) / SS-02 VPs in Dispatch 5b for the
  orthogonal exhaustive-enum property).
- `serde 1` (with `derive` feature, declared as
  `serde = { version = "1", features = ["derive"] }`) is the project pin
  (per SS-deps-pin-manifest.md v1.1.17). The `derive` feature is
  mandatory: `HookEventRecord` carries `#[derive(serde::Serialize,
  serde::Deserialize)]` per SS-daemon-lifecycle.md §Drain, and without
  the `derive` feature the proc-macros `Serialize`/`Deserialize` do not
  compile.
- `serde_json 1` is the project pin (per SS-deps-pin-manifest.md v1.1.17)
  for emission of the JSONL ring record via
  `serde_json::to_string(&record)`. `serde_json` depends on `serde` but
  the two pins are tracked independently because `serde_json` does not
  re-export the `derive` macros.
- **`serde_json`'s `preserve_order` feature is NOT enabled
  (default-features-only; F-R94 I-R94-2 MED closure).** Verified by
  `cargo tree -e features | grep preserve_order` returning empty.
  This precondition ensures the alphabetic-reorder counter-example
  (§Counter-example 3 below — `serde_json::to_value(&record)`
  alphabetic key reordering) remains valid. The `preserve_order`
  feature would back `serde_json::Value` with `IndexMap` (preserving
  insertion order rather than the default alphabetic-sort behavior of
  the BTreeMap-backed `serde_json::Map`), which would silently
  invalidate the counter-example because `to_value().to_string()`
  would then preserve declaration order rather than alphabetize the
  keys. Per SS-deps-pin-manifest.md v1.1.17 `serde_json 1` pin is
  default-features (no explicit `features = [...]` override).

## Post-conditions

1. `record.format_version == 1` after construction.
2. Serialized prefix is exactly `{"format_version":1,` (literal string match).
3. Round-trip preservation: `serde_json::from_str::<HookEventRecord>(&s).unwrap().format_version == 1`.
4. **Absence-of-field for `None` Options (BC-2.01.007 EC-001 normative
   form; F-R89-3 closure; F-R94 C-R94-2 HIGH correction —
   `Notification` carries `tool_name` + `tool_input` per gene-source
   BC-HOOK-019 wire schema, so it is NOT a no-tool-surface hook type;
   the canonical no-tool-surface set is `(SessionStart,
   UserPromptSubmit, Stop)`):** For a `HookEventRecord` constructed with
   `tool_name: None` and `tool_input: None` (e.g., a `SessionStart`,
   `UserPromptSubmit`, or `Stop` event that has no associated tool
   surface), the serialized output `serde_json::to_string(&record).unwrap()`
   MUST NOT contain the substrings `"tool_name":null` or `"tool_input":null`.
   The fields are omitted entirely from the JSON object because the
   struct declaration carries `#[serde(skip_serializing_if = "Option::is_none")]`
   on `tool_name` and `tool_input` (per arch v1.0.25 commit 18fe265
   `HookEventRecord` struct definition + SessionStart None-case
   serialization example; PRD v1.25 §BC-RING-001 EC-001 normative
   `skip_serializing_if` pin). Verified by integration test: construct a
   `HookEventRecord::new(...)` for a hook type with no tool surface,
   serialize, and assert the absence of both `null` substrings AND the
   presence of the remaining required keys (`format_version`,
   `hook_type`, `session_id`, `received_utc`, etc.).

## Counter-examples

1. Reorder struct field declarations in `HookEventRecord` such that
   `session_id` precedes `format_version` — must cause the integration test
   to fail because serde respects declaration order.
2. Change `RING_FORMAT_VERSION` to `0` or `2` — must cause the literal-prefix
   assertion to fail.
3. Use `serde_json::Value`-wrapped serialization that re-orders keys
   alphabetically (e.g., `serde_json::to_value(&record).unwrap().to_string()`)
   — this would order `format_version` after `hook_type` etc.; the
   integration test MUST use direct `to_string(&record)`, not `to_value`
   round-trip.
4. Replace `#[derive(Serialize)]` with a hand-written impl that emits fields in
   alphabetical order — must cause the integration test to fail.
5. **Implementer omits the `#[serde(skip_serializing_if = "Option::is_none")]`
   annotation on the `tool_name` and/or `tool_input` fields (F-R89-3
   counter-example; F-R94 C-R94-2 HIGH correction — corrected
   no-tool-surface example set; `Notification` carries
   `tool_name`/`tool_input` per gene-source BC-HOOK-019 wire schema,
   so it is NOT a no-tool-surface hook type; the canonical
   no-tool-surface set is `(SessionStart, UserPromptSubmit, Stop)`):**
   the resulting `HookEventRecord` for a hook type with no tool surface
   (e.g., `SessionStart`, `UserPromptSubmit`, or `Stop`) would
   serialize to a JSON object containing the literal `"tool_name":null`
   and/or `"tool_input":null` substrings instead of omitting the fields
   entirely. The Post-condition 4 absence-of-field assertion (literal
   substring search for `"tool_name":null` / `"tool_input":null` in the
   serialized output) catches this regression; the `cargo-mutants`
   mutation-test rationale targets this exact attribute strip as a
   high-leverage mutation (removing `#[serde(skip_serializing_if = "Option::is_none")]`
   is the canonical mutation that flips the absence-of-field contract).

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 7.a | Construct `HookEventRecord` via `::new(...)` for any HookType | `record.format_version == 1` |
| 7.b | Serialize via `serde_json::to_string(&record)` | `starts_with("{\"format_version\":1,")` |
| 7.c | Round-trip `serde_json::from_str::<HookEventRecord>(&s)` | `.format_version == 1` preserved |
| 7.d | Construct `HookEventRecord` for `SessionStart` (no tool surface) | serialized output does NOT contain `"tool_name":null` or `"tool_input":null` |
| 7.e | Construct `HookEventRecord` for `UserPromptSubmit` | same absence-of-field assertion |
| 7.f | Construct `HookEventRecord` for `Stop` | same absence-of-field assertion |
| 7.g | Mutation: `RING_FORMAT_VERSION = 0` | integration test fails on prefix mismatch |
| 7.h | Mutation: strip `#[serde(skip_serializing_if = "Option::is_none")]` from `tool_name`/`tool_input` | absence-of-field assertion fails |

**Mutation-test rationale:** the `format_version: u32` field value `1` is a
prime mutation target (off-by-one, sign-flip). Mutation testing with
`cargo-mutants` ensures the assertion is value-discriminating, not just
key-discriminating.

## Harness Location

- `monocle-runtime/tests/jsonl_ring.rs` (integration)
- Test name: `test_BC_RING_001_format_version_first_key` (per PRD v1.25
  §BC-RING-001, Verification subsection — to be migrated to
  `test_BC_2_01_007_format_version_first_key`).

## Proof Harness Skeleton

Per L4 template §Proof Harness Skeleton: the canonical proof-harness intent for
this VP is documented across `## Mechanism` (execution narrative — what the
harness does), `## Pre-conditions` / `## Post-conditions` (assertion surface),
`## Counter-examples` (negative cases), `## Probe Matrix` (probe enumeration),
and `## Harness Location` (file path + test name). The skeleton below is the
template-strict form pointing to the rich harness specification above.

```rust
// Proof method: manual+mutation
// See ## Mechanism for execution narrative.
// See ## Probe Matrix for the canonical probe enumeration.
// See ## Harness Location for the implementing file and test name.
//
// Skeleton (illustrative; canonical assertions live in the probe matrix above):
#[test]  // or #[kani::proof] / proptest! / etc. per proof_method
fn verify_bc_2_01_007() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/jsonl_ring.rs` (integration)
- Test name: `test_BC_RING_001_format_version_first_key` (per PRD v1.25
  §BC-RING-001, Verification subsection — to be migrated to
  `test_BC_2_01_007_format_version_first_key`).

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Per `## Proof Method` Bounded? column above; finite probe set or bounded property quantification. |
| Proof complexity | Tractable | `proof_method: manual+mutation` per frontmatter; mechanism documented in `## Mechanism` section. |
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
- Predecessor: monolithic VP-RING-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.007.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 §Drain (commit
  18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.007 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.17.

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

## §Trace v1.0.2 — F-R105-7 MED + F-R105-11 MED: Manifest Pin Refresh v1.1.15 → v1.1.17 + Sister-VP Reference Reconciliation `VP-TYPES-001` → `VP-013`

**Bump:** v1.0.1 → v1.0.2.
**Predecessor pin:** v1.0.1 (commit 4090d0b — RES-03 VP heading reconciliation).
**Scope of v1.0.2 (NORMATIVE — two co-located fixes; NO content cascade):**

### Change set 1 — Manifest Pin Refresh (F-R105-7 MED) (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** body cited `SS-deps-pin-manifest.md v1.1.15` at 4 locations (Pre-conditions §serde pin, §serde_json pin, Mechanism note on serde_json pin, References §Dependency pins; pre-edit grep).
  - **After:** body cites `SS-deps-pin-manifest.md v1.1.17` at the same 4 locations (post-edit grep).
- **SE-17c-d body-scope grep:** pre-§Trace-block body cites of `1.1.15` → 0 remaining; cites of `1.1.17` → 4.

### Change set 2 — Sister-VP Reference Reconciliation (F-R105-11 MED) (NORMATIVE)

- **SE-17f before/after evidence:**
  - **Before:** Pre-conditions §HookEventRecord non-exhaustive note cited sister VP as `VP-TYPES-001` (pre-renumbering form; pre-edit grep at line 87).
  - **After:** Pre-conditions §HookEventRecord non-exhaustive note cites sister VP as `VP-013` (canonical per VP-INDEX.md §Renumbering Appendix), with parenthetical noting the old-ID origin for traceability into pre-Dispatch-5b artifacts.
- **SE-17c-d body-scope grep:** pre-§Trace-block body cites of `VP-TYPES-001` → 0 remaining (the only remaining `VP-TYPES-001` token is inside the §Trace block itself as historical before-evidence).

### Rationale

**Change set 1:** Architect confirmed (T-128d, commit 0d0c64b) the manifest delta v1.1.15 → v1.1.17 is **STRUCTURAL ONLY** — version-number swap with no content cascade. Mechanical citation refresh authorized.

**Change set 2:** VP-INDEX.md v1.1 §Renumbering Appendix (Dispatch 5b) canonicalized the VP-TYPES-001 → VP-013 renumbering. The pre-existing VP-007 body sister-VP reference predated the renumbering and was stale. Reconciliation aligns VP-007 with the VP-INDEX canonical ID per VP-INDEX-as-source-of-truth discipline.

### Authoritative cross-references

- **Manifest:** `architecture/SS-deps-pin-manifest.md` v1.1.17 (commit 0d0c64b — T-128d §Trace reconciliation).
- **VP-INDEX:** `verification-properties/VP-INDEX.md` v1.1 §Renumbering Appendix row `VP-TYPES-001 | VP-013 | vp-013-non-exhaustive-enum-policy.md | 5b | sharded`.
- **R105 closure chain:** F-R105-7 MED (manifest pin refresh sweep across 14 pin-citing VP files) + F-R105-11 MED (VP-007 sister-VP reference reconciliation).
- **Concurrent dispatch:** T-128j FV portion — VP-014 title sync to VP-INDEX canonical (separate VP file).

### SE-16d chain monotonicity (NORMATIVE)

UTC ISO-8601 `Z` form: `2026-05-17T19:30:00Z` >= chain high-water `2026-05-17T19:00:00Z` (nfr-catalog.md). SE-16d PASS.

### SE-17g NORMATIVE / INFORMATIONAL classification (NORMATIVE)

- NORMATIVE: pin refresh `v1.1.15` → `v1.1.17`; sister-VP reference reconciliation `VP-TYPES-001` → `VP-013`; frontmatter `version` / `timestamp` updates.
- INFORMATIONAL: rationale subsection; cross-reference subsection; concurrent dispatch context.

### Per CLAUDE.md Production-Grade Default Rule 1+5

Co-located both fixes in this dispatch rather than splitting across two commits. Both fixes are in-scope-of-T-128g+T-128j (FV-scope VP file edits), both consume the same cross-reference verification (VP-INDEX canonical), and both follow the same SE-17f before/after evidence + SE-17c-d body-scope grep discipline. No tech-debt entries created.
