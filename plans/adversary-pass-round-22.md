---
document_type: adversarial-review-report
level: ops
version: "1.0"
status: complete
producer: adversary (fresh context, round 22, production-grade lens) — transcribed by state-manager during round-22 close-out
phase: pre-phase-1-final-gate-round-22-complete
timestamp: 2026-05-13T18:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md  # v1.1.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md  # v1.2.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md  # v1.1.5
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md  # v1.1.2
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-21 fix burst commits 83d5fc5 + 3495812 + state close-out ac87649; resolves round-20 F-R20-1/2/3 cleanly with 3 new findings on doc/test-spec surfaces"
project: monocle
verdict: MULTIPLE_DEFER_PATTERNS
---

# Adversarial Pass — Round 22

## Verdict
MULTIPLE_DEFER_PATTERNS — 0 CRITICAL + 3 MEDIUM + 0 LOW. Round-21 fix burst correctly resolved F-R20-1/2/3 on the code surface (no silent fallback remaining; sibling-coherent parser guards; rustdoc url-crate ref removed). Three new MEDIUM defects on documentation/test-spec surfaces from the trait-surface change propagation gap.

## Disposition of Round-20 Findings (all RESOLVED, no regressions)

- **F-R20-1 MEDIUM** (silent fallback in `metadata`/`enrich`): RESOLVED at SS-engine-module.md lines 324-325 and 365-366 via `BaseDirs::new().ok_or(EngineMetadataError::HomeUnresolvable)?`. Trait surface for both `metadata` and `enrich` now returns `Result<…, EngineMetadataError>` (lines 92, 104). `EngineMetadataError` enum declared at lines 543-563.
- **F-R20-2 MEDIUM** (sibling-parity guards on `parse_frontmatter_field`): RESOLVED at SS-core-types-and-abi.md lines 740-760. All four guards (empty / flow-list / block-scalar / quote-strip) present and semantically equivalent to sibling at lines 810-831. Guard ordering matches sibling.
- **F-R20-3 LOW** (`url` crate reference in rustdoc): RESOLVED at SS-engine-module.md lines 422-436. Crate recommendation removed; rustdoc preserves the infallible-construction / fail-at-preflight contract semantics.

## New Defects (Round 22)

### F-R22-1 MEDIUM — "Vision-exact" claim is now factually false (propagation gap)

File: `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md` lines 50-52.

Defective text: "EngineModule trait method signatures (`id`, `metadata`, `detect`, `enrich`, `on_hook`) match `domain-monocle-vision-synthesis.md` §EngineModule lines 111–128 exactly."

Vision lines 111-128 declare `fn metadata(&self) -> EngineMetadata` (line 116) and `async fn enrich(&self, ...) -> EnrichedSession` (line 124) — both unwrapped return types. Round-21 changed both to `Result<…, EngineMetadataError>`. Therefore `metadata` and `enrich` are no longer vision-exact; only `id`, `detect`, and `on_hook` match vision verbatim. Architect explicitly addressed this exact class of imprecision in v1.1.1 (N16-3: "claim 'matches vision exactly' replaced with precise text") but round-21 regressed it.

Correct fix: Replace lines 50-58 with a precise statement: methods `id`, `detect`, `on_hook` are vision-verbatim; `metadata` and `enrich` are vision-spirit-aligned elaborations wrapping the original return shape in `Result<…, EngineMetadataError>` to honor CLAUDE.md SOUL #4 (no silent fallback). The vision returns are recoverable from the Ok variant.

Routing: vsdd-factory:architect.

### F-R22-2 MEDIUM — BC-ENGINE-001 Pre-Staging description carries the same drift

File: `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md` line 612.

Defective text: row reads "vision-exact signature (detect/enrich/on_hook) and no sealed bound; metadata() returns Result<…>; enrich() returns Result<…>" — internally contradictory (lists `enrich` as vision-exact, then admits Result return).

Correct fix: Replace "(detect/enrich/on_hook)" with "(id/detect/on_hook)" — the three methods that genuinely match vision verbatim.

Routing: vsdd-factory:architect.

### F-R22-3 MEDIUM — BC-ENGINE-002 verification block has no test for the no-silent-fallback contract on `metadata`/`enrich`

File: `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md` lines 588-596.

BC-ENGINE-002's verification block specifies unit tests for `id()`, `new()`, and three `detect()` cases — but specifies NO test that asserts `metadata()` or `enrich()` return `Err(EngineMetadataError::HomeUnresolvable)` when home unresolvable. The no-silent-fallback contract (BC-ENGINE-001 line 580-581) is then enforceable only by implementer-discipline — exactly what CLAUDE.md SOUL #4 forbids.

Correct fix: Append to BC-ENGINE-002 verification block a unit test in `monocle-runtime/tests/engine_module.rs`:
1. Construct `ClaudeCodeModule::new("http://127.0.0.1:7891".into())`.
2. Set `$HOME` unset and `XDG_*` empty via `temp-env` or equivalent test harness.
3. Assert `module.metadata().is_err()` AND `matches!(module.metadata().unwrap_err(), EngineMetadataError::HomeUnresolvable)`.
4. Equivalent assertion for `module.enrich(&mock_snapshot).await`.

Routing: vsdd-factory:architect (extend BC verification block) then vsdd-factory:product-owner (formal BC with postconditions in Phase 1).

## Severity Trajectory (post-FC bursts)

| Round | CRITICAL | HIGH/MEDIUM | LOW |
|---|---|---|---|
| R12 (FC) | 4 | 6 | 4 |
| R14 | 3 | 5 | 0 |
| R16 | 1 | 4 | 0 |
| R18 | 1 | 2 | 1 |
| R20 | 0 | 2 | 1 |
| R22 | **0** | **3** | **0** |

MEDIUM count ticked up by 1 due to propagation-gap pattern from the round-21 trait-surface change. F-R22-1 and F-R22-2 are the same propagation gap in two places; F-R22-3 is a separate test-gap defect.

## Novelty Assessment

Novelty: MEDIUM. F-R22-1 and F-R22-2 are recurrence of v1.1.1 N16-3 pattern — every round-N trait-signature change must include a vision-claim re-check pass. Tag F-R22-1/2 as `[process-gap]` for the orchestrator: a structural process gap (no automated check ensures vision-claims stay accurate across trait signature changes). F-R22-3 is genuinely new (surfaces only after the Result migration).

## Recommendation

FIX (3 surgical fixes; ~35 min architect work). Then round 24 validation. F-R22-3 is the most important — the no-silent-fallback contract added in round 21 has zero test enforcement until F-R22-3 is fixed.
