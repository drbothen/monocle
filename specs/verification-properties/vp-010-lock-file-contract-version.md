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
source_bc: BC-2.01.010
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

# VP-010: Lock File `contract_version: 1` First Key

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-LOCK-001 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

The JSON content written to `<runtime_dir>/monocle.lock` is structurally a
JSON object whose first key (per `serde_json` declaration-order
serialization) is `contract_version` with integer value `1`. Any
lock-file reader (e.g., TUI client) MUST inspect `contract_version` before
consuming other fields; on mismatch the reader returns
`Err(LockFileError::UnsupportedContractVersion(N))`; on absence the reader
returns `Err(LockFileError::MissingContractVersion)`. Neither error path
panics. The first-key invariant is the consumer-side reciprocal of
VP-005's producer-side joint mode-and-content assertion (`0o600` + JSON
prefix).

## Source Contract

- **BC (primary):** BC-2.01.010 — Lock File Contract Version Field.
- **BCs (partial coverage):** BC-2.01.005 (producer-side joint
  mode-and-content assertion).
- **Postcondition/Invariant:** declaration-order serialization with
  `contract_version` first; round-trip parse via `LockFile` struct
  preserves `contract_version == 1`; version-mismatch returns a typed
  error (not a panic); absent-`contract_version` returns
  `MissingContractVersion`; cross-property reciprocation with VP-005
  §Post-condition 1.
- **Traces to (historical):** BC-LOCK-001 (SS-daemon-lifecycle.md §Start
  Sequence).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test | Bounded — finite probe set | First-key prefix; round-trip; v2-mismatch error variant; missing-field error variant |
| Mutation test (auxiliary) | cargo-mutants | N/A — mutation surface | `contract_version: u32 = 1` value mutation; reader-side gate condition |

## Mechanism

Integration test (primary; harness at
`monocle-runtime/tests/lock_file_contract.rs` — files in `<crate>/tests/`
are cargo integration tests; PRD v1.25 §7 RTM Test Type column labels this
BC `Integration`); mutation-test (auxiliary). The harness asserts the
written file's literal prefix, performs round-trip via
`serde_json::from_str::<LockFile>`, constructs synthetic `v2` and
`absent-version` files, and asserts the typed-error returns (NOT panics).
`cargo-mutants` is configured to mutate the writer's `contract_version`
literal `1` to `0` and to mutate the reader's gate-condition; both
mutations must be caught.

## Pre-conditions

- Daemon start sequence completes; lock file is written via
  `tempfile::persist`.
- Lock-file reader code is
  `pub fn read_lock_file(path: &Path) -> Result<LockFile, LockFileError>`.
- `chrono 0.4` is the project pin (per SS-deps-pin-manifest.md v1.1.15)
  for the lock-file `startTimeUtc` ISO 8601 millisecond timestamp
  formatter. The daemon emits `startTimeUtc` via
  `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")` per
  SS-daemon-lifecycle.md §Start Sequence step 6 and §Trace v1.0.14
  F-R72-1 rationale (cross-field uniformity with `last_hook_ts` and
  `shutdown_utc`). Although VP-010 primarily asserts the
  `contract_version` first-key invariant, the `startTimeUtc` field is
  one of the lock file's normative fields and the round-trip read-back
  (post-condition 2 via `serde_json::from_str::<LockFile>`) requires the
  deserialization path to accept the chrono-emitted format.

## Post-conditions

1. `std::fs::read_to_string(&lock_path).unwrap().starts_with("{\"contract_version\":1,")`.
2. `serde_json::from_str::<LockFile>(&content).unwrap().contract_version == 1`.
3. With a synthetic lock file where `contract_version = 2`, the reader logs
   a warning and returns
   `Err(LockFileError::UnsupportedContractVersion(2))` — NOT a panic and
   NOT a silent acceptance of unknown fields.
4. With a synthetic lock file where `contract_version` is absent entirely,
   the reader returns `Err(LockFileError::MissingContractVersion)`.
5. **Cross-property with VP-005 §Post-condition 1** (lock-file 0o600 mode
   + `contract_version` JSON content): VP-005 asserts the lock-file file
   mode is `0o600` AND the JSON content begins with
   `{"contract_version":1,`; this VP asserts the same `contract_version`
   first-key invariant from the consumer side (reader-facing schema
   assertion) and is consumed by VP-005's mode-and-content joint
   post-condition.

## Counter-examples

1. Lock file written with `serde_json::to_value(&lock).to_string()` (which
   alphabetizes) — would place `app` before `contract_version`; the prefix
   assertion must fail.
2. Reader implements `serde_json::from_str` without an explicit
   `contract_version` check before field access — a future v2 lock file
   would be silently misparsed; the integration test must construct a
   synthetic v2 file and assert the version-gate error.
3. Lock-file writer omits `contract_version` (regression) — readers MUST
   reject; covered by post-condition 4.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 10.a | Daemon start writes lock file | `read_to_string` starts with `{"contract_version":1,` |
| 10.b | `serde_json::from_str::<LockFile>(&content)` | `.contract_version == 1` |
| 10.c | Synthetic `contract_version: 2` lock file | `Err(LockFileError::UnsupportedContractVersion(2))`; no panic |
| 10.d | Synthetic absent-`contract_version` lock file | `Err(LockFileError::MissingContractVersion)`; no panic |
| 10.e | Mutation: writer emits `contract_version = 0` | integration test 10.a fails |
| 10.f | Mutation: reader skips gate check | integration test 10.c or 10.d slips silently — caught by `cargo-mutants` |

**Mutation-test rationale:** the `contract_version` integer value `1` is a
prime mutation target. `cargo-mutants` will attempt to mutate the writer
to `contract_version = 0` and the reader's gate condition; both must be
caught by the integration test.

## Harness Location

- `monocle-runtime/tests/lock_file_contract.rs` (integration)
- Test name: `test_BC_LOCK_001_contract_version_first_key` (per PRD v1.25
  §BC-LOCK-001, Verification subsection — to be migrated to
  `test_BC_2_01_010_contract_version_first_key`).

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
fn verify_bc_2_01_010() {
    // Pre-conditions: see ## Pre-conditions section
    // Probe execution: see ## Probe Matrix section
    // Post-condition assertions: see ## Post-conditions section
    // Counter-example coverage: see ## Counter-examples section
}
```

**Harness implementation location:**

- `monocle-runtime/tests/lock_file_contract.rs` (integration)
- Test name: `test_BC_LOCK_001_contract_version_first_key` (per PRD v1.25
  §BC-LOCK-001, Verification subsection — to be migrated to
  `test_BC_2_01_010_contract_version_first_key`).

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
- Predecessor: monolithic VP-LOCK-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.010.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 §Start
  Sequence (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.010 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-005 §Post-condition 1 (producer-side joint
  mode-and-content assertion).

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
