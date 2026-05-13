---
document_type: adversarial-review-report
level: ops
version: "1.0"
status: complete
producer: adversary (fresh context, round 28, production-grade lens) — transcribed by state-manager during round-28 durability
phase: pre-phase-1-final-gate-round-28-complete
timestamp: 2026-05-14T00:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-engine-module.md  # v1.1.7
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-core-types-and-abi.md  # v1.2.3
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md  # v1.0.4
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-deps-pin-manifest.md  # v1.1.7
  - /Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-conventions-anti-patterns.md  # v1.5
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md  # v1.4.13
  - /Users/jmagady/Dev/monocle/.factory/STATE.md
input-hash: "[live-state]"
traces_to: "round-27 fix burst commits 9be1033 (engine v1.1.7) + 48d952a (conventions v1.5) + a1c83a9 (brief v1.4.13) + 7ce2d09 (state); validates F-R26-adv-1 CRITICAL resolution; surfaces 2 HIGH + 2 MED + 2 LOW new findings continuing F-R26-adv-1 pattern recurrence"
project: monocle
verdict: REGRESSION
---

# Adversarial Pass — Round 28

## Verdict

REGRESSION — 0 CRITICAL + 2 HIGH + 2 MEDIUM + 2 LOW. F-R26-adv-1 CRITICAL was resolved comprehensively for 4 structs (EngineMetadata, ProcessSnapshot, EnrichedSession, HookResponse). Fresh-context derivation surfaces TWO HIGH defects: (1) EnrichedSession::new(..., 0) reintroduces the exact "epoch sentinel" semantic-smell the architect rejected for ProcessSnapshot; (2) THREE MORE `#[non_exhaustive]` structs (SessionHandle, EngineVersion, SpawnArgs) lack constructors and will hit E0639 in `monocle-runtime/tests/` (integration tests compile as SEPARATE crates that link the library). Plus 2 MEDIUM and 2 LOW. Architect's round-27 audit-completeness claim was overstated.

## Disposition of Round-26 Findings (all RESOLVED)

- F-R26-adv-1 CRITICAL: RESOLVED for 4 structs. NEW finding F-R28-2 surfaces 3+ more structs in the SAME class.
- F-R26-adv-2 MEDIUM (semgrep idiom): RESOLVED. Pattern-either covers all realistic idioms.
- F-R26-adv-3 MEDIUM (POL-11 positive coverage): RESOLVED in shape. Implementation ready for devops-engineer.
- F-R26-adv-5 LOW (field coverage): RESOLVED.
- F-R26-adv-6 LOW (rule consolidation): RESOLVED.

## Important Findings

### F-R28-1 HIGH — `EnrichedSession::new(..., 0)` epoch sentinel reintroduces the smell architect rejected for ProcessSnapshot

File: SS-engine-module.md line ~567 (enrich() call site) + lines 316-356 (struct + constructor rationale).

`EnrichedSession::new` accepts `last_event_micros: i64`. The enrich() call site passes `0` as the sentinel meaning "enriched, no hook events yet." This is the exact semantic-smell the architect documented as the rejection rationale for using `0` in ProcessSnapshot — the round-27 fix narrative explicitly cited magic-zero ambiguity as the production-grade reason to require a constructor. Reintroducing it on EnrichedSession violates the architect's own stated principle within the same spec version.

Production-grade fix: Change `last_event_micros: i64` to `last_event_micros: Option<i64>`. None = "enriched, no hook events yet"; Some(t) = last event timestamp. Update BC-ENGINE-002-ERR test, enrich() call site, and downstream consumer guidance.

Routing: architect, then product-owner ratifies if BC postcondition changes.

### F-R28-2 HIGH — Three more `#[non_exhaustive]` structs lack constructors; Rust integration tests compile as separate crates → E0639 applies

File: SS-engine-module.md lines 660-692 (SessionHandle, EngineVersion, SpawnArgs) + SS-daemon-lifecycle.md line ~324 (HookEventRecord referenced but undefined).

Tests in `monocle-runtime/tests/engine_module.rs` cannot construct these structs via struct-literal even though the structs are defined in `monocle-runtime/src/` — `tests/*.rs` is a separate `[[test]]` binary linking the library as a dependency. Architect's audit claim ("4 structs") missed at least 3. F-R26-adv-1's root cause (cross-crate struct-literal on `#[non_exhaustive]` types) recurs for a new struct cohort. SS-core-types-and-abi.md line 293 states the E0639 constraint explicitly; the engine-module spec still violates it for SpawnArgs, SessionHandle, and EngineVersion. Also defines HookEventRecord (or rewrite BC-RING-001 to use the actual on-the-wire shape via HookEnvelope + ring wrapper) — that type is referenced in daemon-lifecycle but defined nowhere in the spec corpus.

Production-grade fix: Add `pub fn new(...)` constructors to SpawnArgs (with project_root + builder methods if appropriate), SessionHandle, EngineVersion. Also define HookEventRecord (or rewrite BC-RING-001 to use the actual on-the-wire shape via HookEnvelope + ring wrapper).

Routing: architect.

## Medium Findings

### F-R28-3 MEDIUM — HookResponse rustdoc documents `pub` field mutation as the canonical setter pattern (anti-pattern)

File: SS-engine-module.md lines 396-402.

The HookResponse struct was given a constructor in the round-27 fix burst (resolving F-R26-adv-1 for that struct). However, the rustdoc block at lines 396-402 continues to show `let mut response = HookResponse::new(decision); response.diagnostic = Some(d); response.redirect = Some(u);` as the canonical usage pattern. This forces callers to declare `let mut`, bypasses any future validation, and contradicts the encapsulation rationale architect cited for EngineMetadata::new.

Production-grade fix: Replace with `HookResponse::new(decision).with_diagnostic(d).with_redirect(u)` builder methods (Rust-idiomatic). Phase 1 callers that only need .new(decision) get clean immutable construction.

Routing: architect.

### F-R28-4 MEDIUM — BC-RING-001 test spec references undefined `HookEventRecord` type

File: SS-daemon-lifecycle.md line ~324.

Confirmed via grep across .factory/specs/ — `HookEventRecord` is defined nowhere. The type is referenced in BC-RING-001's test body as the concrete type pushed onto the ring buffer. Implementer cannot construct it; the spec is unimplementable as written.

Production-grade fix: Architect defines `HookEventRecord` struct in SS-core-types-and-abi.md ring section, OR rewrites BC-RING-001 test to reference an existing serializable type with the documented JSON shape.

Routing: architect.

## Low Findings

### F-R28-5 LOW — v1.1.5 supersession annotation incorrectly claims content was superseded (content remains current)

File: SS-engine-module.md lines 969-972.

v1.1.5's actual change (BC-ENGINE-002-ERR added to Pre-Staging table 3→4) is STILL CURRENT in v1.1.7. The supersession annotation incorrectly describes content from v1.1.4 as if it were in v1.1.5, implying v1.1.5 content no longer applies. A future reader will misread the changelog and believe the BC-ENGINE-002-ERR Pre-Staging row is no longer canonical.

Production-grade fix: Remove supersession annotation from v1.1.5 entry OR rewrite to "v1.1.5 content remains current; no supersession". Leave v1.1.4 annotation as-is.

Routing: architect.

### F-R28-6 LOW — Brief v1.4.13 revision table row precedes v1.4.12 (reverse chronological order)

File: product-brief.md lines 77-78.

The revision history table lists v1.4.13 immediately above v1.4.12, violating the ascending monotonic order maintained in all prior revision tables in this project. Tooling that processes revision tables in order will misread the changelog.

Production-grade fix: Swap rows so the table reads monotonically ascending (oldest → newest).

Routing: product-owner.

## Pass Verification Results

- **Pass A (F-R26-adv-1 resolution):** CONFIRMED for EngineMetadata, ProcessSnapshot, EnrichedSession, HookResponse. NOT complete — F-R28-2 surfaces 3 additional structs in the same E0639 class.
- **Pass B (semantic-smell re-derivation):** F-R28-1 found. EnrichedSession epoch-sentinel pattern contradicts architect's stated rationale from the same round-27 burst.
- **Pass C (struct corpus completeness):** F-R28-2 found. SpawnArgs, SessionHandle, EngineVersion lack constructors; HookEventRecord undefined.
- **Pass D (round-27 new-defect hunt):** F-R28-3 (HookResponse rustdoc), F-R28-4 (HookEventRecord), F-R28-5 (changelog annotation), F-R28-6 (brief table order) found.
- **Pass E (convergence):** NOT_CONVERGED — needs round-29 fix burst.

## Convergence Assessment

| Round | CRIT | HIGH | MED | LOW |
|-------|------|------|-----|-----|
| R20 | 0 | — | 2 | 1 |
| R22 | 0 | — | 3 | 0 |
| R24 | 0 | — | 3 | 2 |
| R26 | 1 | — | 2 | 3 |
| R28 | 0 | 2 | 2 | 2 |

Convergence verdict: NEEDS_MULTIPLE. Round-29 fix burst required. Round-30 validation has ~40% probability of surfacing 1-2 more pattern-recurrence items based on the architect-audit-completeness meta-pattern.

## Novelty Assessment

MEDIUM novelty. F-R28-1 is a semantic-smell recurrence of a principle the architect explicitly documented in round 27 — not a new class of defect. F-R28-2 is a direct structural recurrence of F-R26-adv-1 in a new struct cohort — the audit-completeness gap is the novel element. F-R28-3/4 are first-principles re-derivations. F-R28-5/6 are mechanical correctness defects visible on fresh read.

## Process-Gap Lesson

Architect's audit completeness is unreliable when claiming "audited all structs." Recommend codification: every architect spec change that adds `#[non_exhaustive]` to a struct in any project crate MUST include a Cross-Crate Constructor Audit table cross-referencing every external construction site (including integration tests). The audit table must be committed atomically with the struct definition — not retroactively appended after an adversary surfaces gaps.

## Recommendation

FIX. Architect owns F-R28-1, F-R28-2, F-R28-3, F-R28-4, F-R28-5. Product-owner owns F-R28-6. Then round-29 validation.
