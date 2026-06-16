---
document_type: behavioral-contract
level: L3
version: "1.0.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-18T05:13:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "0d0918b"
traces_to: prd.md
origin: greenfield
subsystem: SS-02
capability: CAP-002
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.02.004: FactoryAdapter Trait Definition (FC-04 CRITICAL)

## Description

`FactoryAdapter` is an open extension trait in `monocle-core::factory` that abstracts
over `.factory/`-style project state files. It carries exactly seven methods (no more, no
less), no sealed bound, and `Send + Sync + 'static` supertraits only. Third-party WASM
plugins (Phase 3) may implement this trait. Any modification to the seven method signatures
is a BREAKING change requiring an ADR.

## Preconditions

1. The `monocle-core` crate compiles.
2. `monocle-core::factory` module is accessible.

## Postconditions

1. `FactoryAdapter` trait is defined in `monocle-core::factory` with the exact signature:
   - `fn detect(workspace_root: &Path) -> Option<FactoryDetection> where Self: Sized`
   - `fn matches(&self, workspace_root: &Path) -> bool`
   - `fn state_file_path(&self) -> &Path`
   - `fn read_state(&self) -> Result<FactoryState, FactoryReadError>`
   - `fn subscribe(&self) -> Result<StateChangeStream, FactorySubscribeError>`
   - `fn display_name(&self) -> &str`
   - `fn abi_version(&self) -> u32` (default impl returning `crate::MONOCLE_ABI_VERSION`)
2. The trait carries NO sealed bound — supertrait bounds are `Send + Sync + 'static` ONLY. No `private::Sealed` supertrait exists.
3. Supporting types are co-located in `monocle-core::factory`: `FactoryDetection` (3 fields), `FactoryState` (7-field canonical struct), `BlockingIssue`, `BlockingSeverity`, `ConvergenceMetrics`, `FactoryReadError`, `FactorySubscribeError`, `StateChangeStream` type alias.
4. `FactoryState` uses `serde_yaml_ng::Value` for `custom_fields` (not `serde_json::Value`). `convergence` is `Option<ConvergenceMetrics>`. `cycle` is `Option<String>`. These `Option` types represent legitimate absence — NOT unknown. Consumers display `"—"` or `"pending"` for `None`, not `"unknown"`.

## Invariants

1. `FactoryAdapter` is an OPEN extension trait — third-party crates may implement it. Sealing would defeat Phase 3 WASM plugin extensibility.
2. The 7-field `FactoryState` struct is the canonical shape from vision §FactoryAdapter. No `raw_content` field (user red-line per SS-core-types-and-abi.md §FactoryAdapter Trait §Trait Signature rustdoc).
3. Phase 1 `subscribe()` implementations MUST return `Ok(Box::pin(futures::stream::empty()))`. The live watcher is Phase 3 scope.
4. Any modification to the method signatures listed in postcondition 1 is a BREAKING change requiring an ADR.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-018 | `dyn FactoryAdapter` dispatch — `detect` method has `where Self: Sized` | Callers using `dyn FactoryAdapter` use `matches()` instead (no `Self: Sized` bound); asymmetry is intentional and documented |
| EC-019 | `custom_fields` with YAML flow-style lists or block scalars | `parse_frontmatter_extra_fields` skips these; callers needing full YAML semantics re-parse with `serde_yaml_ng::from_str` |
| EC-020 | Phase 3 WASM adapter implements `FactoryAdapter` in a separate crate | The trait is open; the adapter's `fn abi_version()` default returns `crate::MONOCLE_ABI_VERSION`; Phase 3 plugin loader checks this value before activating |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| `VsddFactoryAdapter::detect` called with monocle repo root | `Some(FactoryDetection { display_name: "VSDD Factory", ... })` | happy-path |
| `VsddFactoryAdapter::matches` with non-factory dir | `false` | edge-case |
| `cargo check` with Phase 1 workspace | Compiles without error; no `private::Sealed` supertrait in rustdoc | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-014 | `syn 2` AST parse of `monocle-core/src/factory.rs` asserts exactly 7 methods, no `Sealed` supertrait bound, and `Send + Sync + 'static` bounds only | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability §SS-02 |
| Capability Anchor Justification | CAP-002 ("Forward-compatible ABI; wire format stability; factory-state abstraction") per ARCH-INDEX §Capability traceability — this BC defines the FactoryAdapter trait, which is the explicit factory-state abstraction component of CAP-002 |
| L2 Domain Invariants | DI-007 (monocle must not write to any file owned by a harness or factory workflow system — FactoryAdapter is an open extension trait that abstracts read-only access to factory state files; Invariant 1 explicitly prohibits sealing the trait to support third-party implementations that also observe the read-only constraint; subscribe() Phase 1 stub returns empty stream without writing to any factory file) |
| Architecture Module | monocle-core (FactoryAdapter trait, wire format types, protocol versioning) per ARCH-INDEX Subsystem Registry SS-02 |
| Architecture Source | SS-core-types-and-abi.md v1.2.13 §FactoryAdapter Trait |
| FC | FC-04 (CRITICAL) |
| Brief Section | §Scope (forward-compatibility contracts sub-bullet — FactoryAdapter trait) |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-FACTORY-001 |
| Test name | test_BC_FACTORY_001_trait_defined_open_no_sealed_bound |

## Related BCs (Recommended)

- [BC-2.02.005] — composes with: VsddFactoryAdapter is the Phase 1 implementation of this trait
- [BC-2.02.003] — depends on: non-exhaustive enum policy applies to `BlockingSeverity` and other enums in `monocle-core::factory`

## Architecture Anchors (Recommended)

- `architecture/SS-core-types-and-abi.md#factoryadapter-trait` — trait signature, supporting types, FactoryState struct definition, open-trait rationale

## Story Anchor (Recommended)

S-TBD — Implement FactoryAdapter trait in monocle-core (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-014-factory-adapter-trait.md` — VP-014 factory adapter trait AST audit test

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-002 per ARCH-INDEX is authoritative source`
  - After: `DI-007 ...`
  - DI-007 mapping: FactoryAdapter is a READ-ONLY abstraction — it reads factory state (read_state, subscribe) and never writes to factory-owned files. The open trait allows third-party implementations that must also respect this read-only constraint. Phase 1 subscribe() returns an empty stream, no writes.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.2.8 → v1.2.13** (2026-05-18T05:13:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.2.8 → v1.2.13); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-core-types-and-abi.md v1.2.8 §FactoryAdapter Trait`
  - SE-17f AFTER: `SS-core-types-and-abi.md v1.2.13 §FactoryAdapter Trait`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:13:00Z > prior 2026-05-17T18:00:00Z (v1.0.2). ARITHMETICALLY TRUE: 2026-05-18T05:13:00Z > 2026-05-17T18:00:00Z PASS.
