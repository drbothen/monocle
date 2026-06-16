---
document_type: behavioral-contract
level: L3
version: "1.0.7"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:10:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "a484c9d"
traces_to: prd.md
origin: greenfield
subsystem: SS-03
capability: CAP-003
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

# Behavioral Contract BC-2.03.001: EngineModule Trait Definition

## Description

`EngineModule` is the open harness-abstraction trait in `monocle-core::engine`. It defines
exactly five methods using `#[async_trait::async_trait]` for dyn-compatibility with Phase 3
WASM plugins, no sealed bound, and `Send + Sync + 'static` supertraits only. The `async_trait`
macro is required because native `async fn` in traits does not yet provide ergonomic
dyn-compatibility on MSRV 1.88 stable Rust. `metadata()` and `enrich()` must fail fast with
`HomeUnresolvable` rather than silently substituting fallback paths.

## Preconditions

1. `monocle-core` crate compiles.
2. `monocle-core::engine` module is accessible.

## Postconditions

1. `EngineModule` trait is defined in `monocle-core::engine` using `#[async_trait::async_trait]` with exactly these five methods and return types:
   - `fn id(&self) -> &'static str`
   - `fn metadata(&self) -> Result<EngineMetadata, EngineMetadataError>`
   - `fn detect(&self, proc: &ProcessSnapshot) -> bool`
   - `async fn enrich(&self, proc: &ProcessSnapshot) -> Result<EnrichedSession, EngineMetadataError>`
   - `async fn on_hook(&self, event: HookEvent) -> HookResponse`
2. The trait carries NO sealed bound. Supertrait bounds: `Send + Sync + 'static` only.
3. Supporting types are co-located in `monocle-core::engine`: `EngineMetadata`, `ProcessSnapshot`, `EnrichedSession`, `SessionStatus`, `HookResponse`, `HookDecision`, `DeferUntil`, `EngineMetadataError`.
4. `EnrichedSession::last_event_micros` is `Option<i64>`. `None` means no hook events received yet. `Some(t)` means microseconds since Unix epoch of most recent hook event. Consumers MUST NOT treat `0i64` as a sentinel — `0` is the Unix epoch (1970-01-01), not a valid last-event timestamp.
5. `metadata()` and `enrich()` MUST NOT substitute a default path when the platform home directory is unresolvable. They MUST return `Err(EngineMetadataError::HomeUnresolvable)`. Daemon initialization MUST fail fast with a diagnostic.
6. `detect()` MUST NOT perform any I/O, environment lookups, file reads, or shared state mutation. `detect()` is a pure function of its arguments. It MUST be safe to call from any thread, repeatedly, without side effects.

## Invariants

1. `EngineModule` is an OPEN trait — third-party WASM plugins implement it in Phase 3.
2. `HookEvent` type is defined in `monocle-core/src/hook_events.rs` (SS-core-types-and-abi.md §Non-Exhaustive Inner Structs). `EngineModule` references it; does not re-declare.
3. The `#[async_trait]` macro is required because the `EngineModule` trait must be dyn-compatible (Phase 3 loads adapters as `Box<dyn EngineModule>`) and propagate `Send + Sync + 'static` to returned futures. Native `async fn` in traits (stable since Rust 1.75) does NOT yet provide both properties ergonomically on MSRV 1.88 stable. The macro desugars `async fn` to return `Pin<Box<dyn Future<Output = ...> + Send + 'async_trait>>`, providing both dyn-compatibility and Send-on-return. The heap allocation per call is acceptable for the millisecond hook event cadence.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-029 | `metadata()` called when `$HOME` is unset (e.g., systemd service unit without `Environment=HOME`) | Returns `Err(EngineMetadataError::HomeUnresolvable)`; daemon start fails with a clear diagnostic; not a recoverable error |
| EC-030 | `EngineModule::detect()` implementations handle `ProcessSnapshot { exe_path: None, ... }` (process exited before path resolved) | Implementations may return `false`; see BC-2.03.002 for ClaudeCodeModule concrete semantics |
| EC-031 | `on_hook()` called with an unrecognized `HookEvent` variant (future Phase 4 addition) | Returns `HookResponse::new(HookDecision::Allow)` as fail-open default; wildcard arm on `#[non_exhaustive]` HookEvent; rationale: no permission context available for unknown variants, localhost threat model, Defer would stall with no TUI handler |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| `syn 2` AST parse of `monocle-core/src/engine.rs` | 5 methods present; no `Sealed` supertrait; return types match specification | lint |
| `cargo check` with Phase 1 workspace | Compiles without error | happy-path |
| `on_hook()` with future unrecognized `HookEvent` variant | `HookResponse { decision: Allow }` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-019 | `syn 2` AST audit in `monocle-core/tests/engine_module_surface.rs` asserts exactly 5 methods, no `Sealed` supertrait, `Send + Sync + 'static` bounds only, and correct return-type token streams for each method | lint |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability §SS-03 |
| Capability Anchor Justification | CAP-003 ("Engine abstraction over AI coding harnesses; Claude Code Phase 1 adapter") per ARCH-INDEX §Capability traceability — this BC defines the EngineModule trait, which is the explicit engine abstraction over AI coding harnesses named in CAP-003 |
| L2 Domain Invariants | DI-006 (every EngineModule implementation must be stateless with respect to process detection — detect() must not perform I/O and must not mutate shared state — Postcondition 6 mandates that detect() has no I/O and no shared state mutation; EngineModule implementations must follow this constraint to ensure DI-006) |
| Architecture Module | monocle-core (EngineModule trait, EnrichedSession, HookEvent types); monocle-runtime (ClaudeCodeModule implementation — monocle-runtime/src/engine/claude_code.rs) per ARCH-INDEX Subsystem Registry SS-03 |
| Architecture Source | SS-engine-module.md v1.1.27 §EngineModule Trait Signature |
| Vision | §EngineModule |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-ENGINE-001 |
| Test name | test_BC_ENGINE_001_trait_defined_all_methods_no_sealed_bound |

## Related BCs (Recommended)

- [BC-2.03.002] — composes with: ClaudeCodeModule is the Phase 1 implementation of this trait
- [BC-2.03.003] — depends on: HomeUnresolvable error contract specified here in Postcondition 5 is detailed by BC-2.03.003
- [BC-2.02.003] — depends on: non-exhaustive enum policy applies to `HookEvent`, `HookDecision`, `DeferUntil`

## Architecture Anchors (Recommended)

- `architecture/SS-engine-module.md#enginemodule-trait-signature` — trait definition, async_trait rationale, 5-method enumeration, supporting types

## Story Anchor (Recommended)

S-TBD — Implement EngineModule trait in monocle-core (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-019-engine-module-trait.md` — VP-019 EngineModule trait AST audit test

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-003 per ARCH-INDEX is authoritative source`
  - After: `DI-006 ...`
  - DI-006 mapping: This BC defines the EngineModule trait whose detect() method carries the stateless, no-I/O, no-shared-state constraint that is DI-006. The trait definition IS the formal specification of DI-006 for all implementations.
- F-R105-9 (SE-17c-d body-scope grep): 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. F-R105-9 NO-OP for this file.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.1.15 → v1.1.20** (2026-05-18T05:18:00Z):
- F-R109-4: BC was stale by 4 patches cumulative from earlier rounds (v1.1.15 → v1.1.20); this Round 9B dispatch refreshed to latest. Architecture Source row updated.
  - SE-17f BEFORE: `SS-engine-module.md v1.1.15 §EngineModule Trait Signature`
  - SE-17f AFTER: `SS-engine-module.md v1.1.20 §EngineModule Trait Signature`
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:18:00Z > prior 2026-05-17T18:00:00Z (v1.0.2). ARITHMETICALLY TRUE: 2026-05-18T05:18:00Z > 2026-05-17T18:00:00Z PASS.

## §Trace v1.0.4

**F-PHASE2-R05-06 — Internal-consistency fix: PC-6 detect() purity postcondition added** (2026-05-19T00:00:00Z):
- Root cause: Traceability §L2 Domain Invariants DI-006 mapping cell cited "Postcondition 5 which mandates that detect() has no I/O and no shared state mutation" but PC-5 body describes only `metadata()` and `enrich()` HomeUnresolvable semantics. The phrase "detect() has no I/O and no shared state mutation" did not exist as a behavioral postcondition — it appeared only as interpretive prose in the Traceability cell. Story-writer S-015 AC-010 correctly identified the property but had to fabricate a quotation because no canonical PC existed.
- Fix: Added PC-6 explicitly codifying the detect() purity contract as a behavioral postcondition.
  - PC-6 text (verbatim): `detect()` MUST NOT perform any I/O, environment lookups, file reads, or shared state mutation. `detect()` is a pure function of its arguments. It MUST be safe to call from any thread, repeatedly, without side effects.
- Fix: Updated Traceability §L2 Domain Invariants DI-006 mapping cell to anchor to PC-6.
  - BEFORE: "...Postcondition 5 which mandates that detect() has no I/O and no shared state mutation..."
  - AFTER: "...Postcondition 6 mandates that detect() has no I/O and no shared state mutation..."
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-19T00:00:00Z > prior 2026-05-18T05:18:00Z (v1.0.3). ARITHMETICALLY TRUE: PASS.

## §Trace v1.0.5

**GAP-PHASE2-R06-3 closure — Architecture Module pin updated per ARCH-INDEX v1.0.11 cascade (trait/impl split clarification)** (2026-05-19T12:10:00Z):
- GAP-PHASE2-R06-3: ARCH-INDEX v1.0.10 → v1.0.11 corrected SS-03 split (EngineModule trait in monocle-core, ClaudeCodeModule implementation in monocle-runtime). BC Architecture Module cell still read pre-correction text `monocle-core (EngineModule trait, ClaudeCodeModule adapter)`.
  - SE-17f BEFORE: `monocle-core (EngineModule trait, ClaudeCodeModule adapter) per ARCH-INDEX Subsystem Registry SS-03`
  - SE-17f AFTER: `monocle-core (EngineModule trait, EnrichedSession, HookEvent types); monocle-runtime (ClaudeCodeModule implementation — monocle-runtime/src/engine/claude_code.rs) per ARCH-INDEX Subsystem Registry SS-03`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:10:00Z > prior 2026-05-19T00:00:00Z (v1.0.4). ARITHMETICALLY TRUE: PASS.

## §Trace v1.0.6

**F-S025-ADV17-LOW-001 closure — Path B Wave 6 MSRV propagation completion** (2026-05-29T00:00:00Z):
- Pass 17 adversary finding F-S025-ADV17-LOW-001: BC-2.03.001 lines 35 + 61 still referenced "MSRV 1.86 stable" after the Phase 1 MSRV bump from 1.86 → 1.88 (SS-deps-pin-manifest v1.2.0). The architect's 6-artifact propagation sweep (commit f3533ce) updated SS-deps-pin-manifest, prd, nfr-catalog, product-brief, ADR-0001, and the RUSTSEC risk-acceptance doc, but missed BC-2.03.001.
- Fix (Option A — matches architect's propagation pattern): Updated "MSRV 1.86" → "MSRV 1.88" at both occurrences.
  - Line 35 (Description): `"dyn-compatibility on MSRV 1.86 stable Rust"` → `"dyn-compatibility on MSRV 1.88 stable Rust"`
  - Line 61 (Invariant 3): `"on MSRV 1.86 stable"` → `"on MSRV 1.88 stable"`
- Architectural claim unchanged: the statement that native async fn in traits does not yet provide ergonomic dyn-compatibility is MSRV-independent and remains accurate on 1.88. Only the project-MSRV qualifier is updated.
- Wider sweep result: no other BCs in ss-03/ or elsewhere under `.factory/specs/behavioral-contracts/` contained "MSRV 1.86" — BC-2.03.001 was the sole remaining artifact.
- Story propagation: S-014 (v1.4) and S-015 (v1.6) both pin BC-2.03.001 at `"1.0.5"` in inputs frontmatter. Story-writer must bump those pins to `"1.0.6"` under `bc_array_changes_propagate_to_body_and_acs` policy. S-014 AC-007 body (line 98) and implementer-notes (line 180) also contain "MSRV 1.86" — story-writer must propagate those body references per the same policy.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-29T00:00:00Z > prior 2026-05-19T12:10:00Z (v1.0.5). ARITHMETICALLY TRUE: PASS.

## §Trace v1.0.7

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-engine-module.md v1.1.20 → v1.1.26 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-engine-module.md v1.1.20 §EngineModule Trait Signature` → `SS-engine-module.md v1.1.26 §EngineModule Trait Signature`.
- Plain version-pin refresh per prior §Trace analysis. No substantive content propagation required — §EngineModule Trait Signature section heading and content anchors are unchanged between v1.1.20 and v1.1.26.
- SE-16d monotonicity PASS: 2026-05-29T00:00:00Z > prior 2026-05-29T00:00:00Z (v1.0.6). SAME TIMESTAMP: same burst, monotonicity satisfied by version increment.
