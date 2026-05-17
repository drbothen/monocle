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
source_bc: BC-2.03.001
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

# VP-019: `EngineModule` Trait Signature Stable; `last_event_micros: Option<i64>`; No Silent Fallback

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-ENGINE-001 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

1. The trait `monocle_core::engine::EngineModule` exists with the exact
   method set: `id`, `metadata`, `detect`, `enrich`, `on_hook`.
2. The trait has NO sealed bound (no `private::Sealed` supertrait).
3. `metadata()` returns `Result<EngineMetadata, EngineMetadataError>`;
   `enrich()` returns `Result<EnrichedSession, EngineMetadataError>` (both
   typed-error returns, not `Option<...>`-with-silent-fallback).
4. `EnrichedSession::last_event_micros` has type `Option<i64>` (NOT bare
   `i64`); `None` is distinguishable from any numeric value including the
   Unix epoch `0`.
5. Supporting types `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`,
   `SessionStatus`, `HookResponse`, `HookDecision`, `DeferUntil`,
   `EngineMetadataError` are all `pub` in `monocle_core::engine`.

## Source Contract

- **BC:** BC-2.03.001 — `EngineModule` Trait Surface (Open, Typed Errors,
  Option<i64> Timestamp).
- **Postcondition/Invariant:** BC-2.03.001 invariants 1-5 asserting trait
  method count + names, no `Sealed` supertrait, typed `Result<_,
  EngineMetadataError>` returns (no silent `Option<...>` fallback), and
  `last_event_micros: Option<i64>` field type. Open-trait rationale
  cross-referenced from VP-014's `FactoryAdapter` parallel trait per
  monolithic line 3192 (open-trait rationale referenced by VP-FACTORY-001
  and VP-ENGINE-001).
- **Traces to (historical):** BC-ENGINE-001 (SS-engine-module.md
  §Behavioral Contracts; PRD v1.25 §BC-ENGINE-001 Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| AST audit (primary) | syn 2 | Bounded — finite trait surface | Trait method-set HashSet equality; super-bound token-stream; `Sealed` substring absence; method return-type token-stream; field-type assertion on `last_event_micros` |
| `cargo check` (supplementary) | cargo | N/A — static | Supporting type re-exports compile via probe file |

## Mechanism

AST audit (via `syn 2` parse of `monocle-core/src/engine.rs` from the
harness at `monocle-core/tests/engine_module_surface.rs`; PRD v1.25 §7
RTM Test Type column labels this BC `AST audit (syn 2)`). The harness
parses the trait declaration, walks the method set, asserts method
return-type token-streams match the canonical `Result<_, _>` forms, and
asserts `EnrichedSession::last_event_micros` field type is `Option<i64>`.

## Pre-conditions

- `monocle-core` builds cleanly.

## Post-conditions

1. A `monocle-core/tests/engine_module_surface.rs` test parses the trait
   declaration and asserts:
   - method count equals 5;
   - method names match the canonical HashSet
     `{id, metadata, detect, enrich, on_hook}`;
   - super-bounds equal `Send + Sync + 'static` (no `Sealed`);
   - `metadata` return type token-stream matches
     `Result < EngineMetadata , EngineMetadataError >`;
   - `enrich` return type token-stream matches
     `Result < EnrichedSession , EngineMetadataError >`.
2. The same test asserts `EnrichedSession::last_event_micros` field type
   is `Option < i64 >` (not bare `i64`).
3. All eight supporting types resolve via `cargo check` with a probe
   file `let _: monocle_core::engine::EngineMetadata; ...`.

## Counter-examples

1. A refactor changes `metadata() -> Result<...>` to
   `metadata() -> EngineMetadata` (panicking on home-unresolvable) —
   fails the return-type token-stream match.
2. `last_event_micros` is reverted to bare `i64` with `0` as sentinel —
   fails the field-type assertion. This regression is what the v1.1.8
   fix (F-R28-1) closed; the VP enforces it.
3. A `private::Sealed` supertrait is added — fails the no-sealed
   assertion. `SS-forward-compatibility.md §Item P3-1 — Verdict on
   Sealed` governs the open trait property; sealing the trait would
   defeat Phase 3 plugin SDK adapter authoring.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 19.a | Walk `trait EngineModule` items via `syn 2` | method count == 5; names HashSet == `{id, metadata, detect, enrich, on_hook}` |
| 19.b | Token-stream match super-bound | equals `Send + Sync + 'static` exactly; no `Sealed` |
| 19.c | Token-stream match `metadata` return type | `Result < EngineMetadata , EngineMetadataError >` |
| 19.d | Token-stream match `enrich` return type | `Result < EnrichedSession , EngineMetadataError >` |
| 19.e | Field-type assertion on `EnrichedSession::last_event_micros` | `Option < i64 >` exactly (not bare `i64`) |
| 19.f | Probe file with `let _: monocle_core::engine::EngineMetadata; ...` for all 8 supporting types | `cargo check` passes |
| 19.g | Inject `private::Sealed` supertrait | Probe 19.b fails |
| 19.h | Revert `last_event_micros` to `i64` with `0` sentinel | Probe 19.e fails |

## Harness Location

- `monocle-core/tests/engine_module_surface.rs` (AST audit)
- Test name: `test_BC_ENGINE_001_trait_defined_all_methods_no_sealed_bound`
  (per PRD v1.25 §BC-ENGINE-001, Verification subsection — to be
  migrated to `test_BC_2_03_001_trait_defined_all_methods_no_sealed_bound`
  post BC renumber propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-ENGINE-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-03/BC-2.03.001.md`.
- Architecture: `architecture/SS-engine-module.md` §Behavioral Contracts.
- Forward compatibility: `architecture/SS-forward-compatibility.md`
  §Item P3-1 — Verdict on Sealed (open-trait rationale shared with VP-014).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.03.001 (Dispatch 4 commit 1030c65).
- Cross-VP: VP-014 (`FactoryAdapter` parallel trait — same open-trait
  property class); VP-020 (`ClaudeCodeModule::detect` strict basename
  match); VP-021 (`HomeUnresolvable` typed error for the no-silent-fallback
  property); VP-022 (`hook_paths()` inherent method).
