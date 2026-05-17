---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
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

# VP-014: `FactoryAdapter` Trait Signature Stable; No Sealed Bound

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

## Verification Method

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

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-FACTORY-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.004.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §FactoryAdapter Trait.
- Forward compatibility: `architecture/SS-forward-compatibility.md`
  §Item P3-1 — Verdict on Sealed (open-trait rationale).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.004 (Dispatch 4 commit 1030c65).
- Cross-VP: VP-015 (`VsddFactoryAdapter` constructor + self-referential detection);
  VP-019 (`EngineModule` parallel trait — same open-trait property class).
