---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-engine-module-v2-delta.md]
input-hash: "0a5432c"
traces_to: prd.md
origin: greenfield
subsystem: SS-03
capability: CAP-003
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.03.008: Default spawn_recipe() Returns UnsupportedOperation

## Description

The `EngineModule` trait provides a default implementation of `spawn_recipe()` that returns
`Err(EngineError::UnsupportedOperation("spawn_recipe"))`. This default applies to all
engine implementations that do NOT override the method — including `CodeMachineModule` in
v1A scope. This establishes a safe, explicit boundary: session spawning is a capability
that must be explicitly opted into, not accidentally inherited.

## Preconditions

1. An `EngineModule` implementation exists that does NOT override `spawn_recipe()`.
2. `spawn_recipe()` is called on that implementation with any `SpawnOptions`.

## Postconditions

1. Returns `Err(EngineError::UnsupportedOperation("spawn_recipe"))` immediately. No I/O,
   no filesystem access, no `PATH` lookup is performed.
2. The error message format is: `"unsupported operation: spawn_recipe"`.
3. When the daemon receives `UnsupportedOperation` from `spawn_recipe()`, it MUST surface
   an error to the TUI: `"Session spawn not supported for this harness"`. The session
   creation wizard MUST present this error in the UI and return to the ProfilePicker step.
4. The default impl is defined in the `EngineModule` trait body in `monocle-core/src/engine.rs`.
   It does NOT require any overriding `impl EngineModule for X` block — the default fires
   automatically for any implementor that does not override the method.
5. Adding `spawn_recipe()` with a default `Err` impl to the trait is NON-BREAKING for
   existing trait implementations. Existing `EngineModule` implementors that compiled before
   this addition continue to compile unchanged; they simply inherit the default `Err` behavior.

## Invariants

1. The default implementation fires for ALL engines that do not opt in — including any future
   WASM plugin engines loaded in Phase 3 that have not implemented `spawn_recipe()`.
2. `UnsupportedOperation` is NOT a retriable error. The caller MUST treat it as a permanent
   capability boundary for the selected engine, not a transient failure.
3. The `"spawn_recipe"` string in `UnsupportedOperation("spawn_recipe")` identifies the
   specific operation that is unsupported. This enables callers to distinguish "this engine
   cannot spawn" from other unsupported operations without string matching the full message.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-112 | User selects a CodeMachineModule profile and attempts to create a new session | Daemon calls `spawn_recipe()` on `CodeMachineModule`; default impl returns `Err(UnsupportedOperation("spawn_recipe"))`; TUI shows "Session spawn not supported for this harness" error; wizard returns to ProfilePicker |
| EC-113 | WASM engine loaded in Phase 3 that does not implement `spawn_recipe()` | Default `Err(UnsupportedOperation("spawn_recipe"))` fires; same error path as EC-112 |
| EC-114 | `CodeMachineModule::spawn_recipe()` called with any `SpawnOptions` including valid paths | Returns `Err(UnsupportedOperation("spawn_recipe"))` regardless of input validity — input is not inspected |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `CodeMachineModule.spawn_recipe(any_opts)` | `Err(EngineError::UnsupportedOperation("spawn_recipe"))` | happy-path (expected unsupported) |
| Any `EngineModule` impl without override called | `Err(EngineError::UnsupportedOperation("spawn_recipe"))` | happy-path (expected unsupported) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `CodeMachineModule.spawn_recipe()` (or any non-overriding impl) returns `Err(UnsupportedOperation("spawn_recipe"))` | unit |
| VP-TBD | Trait compiles without requiring existing implementations to add `spawn_recipe()` override | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC defines the capability boundary for the engine abstraction: spawn is opt-in, not universal; the default Err impl enforces that boundary for all engines that do not explicitly support monocle-controlled session spawning |
| L2 Domain Invariants | DI-006 (EngineModule implementations must be stateless — the default impl performs no I/O and returns a constant error value, satisfying stateless detection requirement; spawn_recipe() is not a detection method but the same stateless principle applies to non-overriding impls) |
| Architecture Module | monocle-core (`EngineModule` trait default impl) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module-v2-delta.md v1.1.0 §spawn_recipe() — new trait method (default impl signature) |
| Cross-Ref | BC-2.03.005 (ClaudeCodeModule overrides this default with the real spawn_recipe() implementation) |
| Test Name | test_BC_2_03_008_default_spawn_recipe_unsupported_operation |

## Related BCs

- [BC-2.03.005] — contrasts with: ClaudeCodeModule overrides the default with a real implementation
- [BC-2.03.001] — depends on: EngineModule trait definition; spawn_recipe() is added to this trait

## Architecture Anchors

- `architecture/SS-engine-module-v2-delta.md#spawn_recipe-new-trait-method` — trait method signature and default impl

## Story Anchor

S-TBD — Same story as BC-2.03.005 (EngineModule trait extension with spawn_recipe() default; filled by story-writer)

## VP Anchors

VP-TBD — Default UnsupportedOperation unit test (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.03.008 authored for SS-03 as part of the v1A control-center pivot BC burst.
- Covers: default trait method impl returning UnsupportedOperation; non-breaking trait addition;
  CodeMachineModule v1A capability boundary.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
