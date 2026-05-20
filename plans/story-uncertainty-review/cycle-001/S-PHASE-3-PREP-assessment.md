---
document_type: story-uncertainty-assessment
story_id: S-PHASE-3-PREP
story_version: "1.0"
story_title: "spec-kit-mcp Integration — Phase 3 Pre-Implementation Mechanical Sweep"
assessment_batch: BATCH-2
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_RESEARCH
---

# Story Assessment: S-PHASE-3-PREP

## Verdict

**NEEDS_RESEARCH** — Story is BLOCKED on external dependency (vsdd-factory spec-kit-mcp rc.19+).
Stage 2 research required to determine availability before any further assessment is meaningful.
This is the only story in the corpus with a NEEDS_RESEARCH verdict.

## Summary

S-PHASE-3-PREP is the Wave 0 pre-implementation mechanical sweep story anchored to the
TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE tech debt entry. The story is structurally
complete and the rationale is sound. However, the AC set is entirely contingent on the
`spec-kit-mcp rc.19+` external dependency shipping. Without knowing the actual API surface
of `spec_kit_verify_invariants()` and `spec_kit_bump_artifact()`, none of the ACs can be
validated for implementation correctness.

The story explicitly documents this via AC-004 (human approval gate) and the `# BC status`
comment ("Cannot be ready until spec-kit-mcp rc.19+ is available"). This is a correctly
documented external-dependency block per CLAUDE.md §Canonical Principle Rule 3.

The story is NOT a Phase 3 blocker for any Wave 1/2/3 implementation work. It only gates
the final resolution of the version-pin staleness asymptote.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S-PREP-D1-01 | HIGH | Library & Framework Requirements table lists `spec-kit-mcp rc.19+ (external, vsdd-factory upstream)`. This is not a semver pin — it is a release candidate designator with a minimum version bound. When the library ships, the story must be updated to pin a specific version before implementation is dispatched. The "rc.19+" designator is a placeholder, not a verifiable constraint. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S-PREP-D2-01 | HIGH | AC-001 calls `spec_kit_verify_invariants(scope="all")`. AC-002 calls `spec_kit_bump_artifact()`. These function signatures are assumed — the actual API surface is unknown until the library ships. The ACs may need to be revised when the real API is available. |
| S-PREP-D2-02 | MEDIUM | AC-003 specifies migrating POL-29 and SE-22 v1/v2 prose rules to schema-enforced invariants. The target schema format is unspecified — this depends entirely on what `spec-kit-mcp` supports. The AC is too vague to drive TDD. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Correctly flagged as `blocks: []`. Does not block any Phase 2 or Phase 3 wave stories. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| S-PREP-D4-01 | MEDIUM | AC-001 and AC-002 are the primary testable outcomes but specify zero violations / zero stale pins as the success criteria. The integration test structure (how to verify "zero violations" in a CI context) is not specified. A Task should specify the test harness entry point. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter is complete. `external_dependency` field is correctly populated. Wave 0 designation is correct. |

## Research Queue

**STAGE 2 REQUIRED:**

1. Research vsdd-factory upstream for spec-kit-mcp release status. Query: "vsdd-factory spec-kit-mcp rc.19+ release date and API surface." Source: vsdd-factory GitHub releases, issue #150 comments, README.
2. Determine if `spec_kit_verify_invariants` and `spec_kit_bump_artifact` are the actual function names, or if the API has evolved since the story was written.

Until Stage 2 research is complete, this story remains BLOCKED and should not be dispatched.

## Recommended Fixes

All fixes contingent on Stage 2 research results. After research:

1. S-PREP-D1-01: Replace `rc.19+` placeholder with actual semver pin. Routing: architect (after library ships).
2. S-PREP-D2-01 + S-PREP-D2-02: Revise ACs to reflect actual API surface. Routing: product-owner + architect.
3. S-PREP-D4-01: Add CI test harness entry point to Tasks. Routing: story-writer.
