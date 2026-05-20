---
document_type: story-uncertainty-assessment
story_id: S-011
story_version: "1.1"
story_title: Non-Exhaustive Enum Policy (FC-02)
assessment_batch: BATCH-3
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: PASS_WITH_OBSERVATIONS
---

# Story Assessment: S-011

## Verdict

**PASS_WITH_OBSERVATIONS** — Story is implementable as written. Two MEDIUM observations on
the canonical 9-enum list completeness and the test audit scope.

## Summary

S-011 is the non-exhaustive enum policy enforcement story. The 9-enum canonical list is
derived from BC-2.02.003 PC-4 and SS-permissions-phase1.md lines 162–203. The story is
well-specified for the core task (add `#[non_exhaustive]` to enums; create AST audit test).
Two medium observations: (1) the 9-enum list in AC-001 does not include `EngineMetadataError`,
`HookType`, or `ShutdownReason` — all of which appear to be public enums in `monocle-core`
or `monocle-runtime`; (2) the AST audit scope is limited to `monocle-core/src/` but some
relevant enums are in `monocle-runtime`.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | `syn = { version = "2", features = ["full"] }` and `quote = "1"` are correctly specified as dev-dependencies. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S011-D2-01 | MEDIUM | The canonical 9-enum list in AC-001 includes `HookType`, `HookEvent`, `HookDecision`, `DeferUntil`, `BlockingSeverity`, `SessionStatus`, `DenyReason`, `AllowPattern`, `DenyPattern`. But `EngineMetadataError` (defined in S-014's `monocle-core::engine`) is also a public enum. Should it carry `#[non_exhaustive]`? It has one variant (`HomeUnresolvable`) in Phase 1 — exactly the pattern that needs `#[non_exhaustive]` for forward compatibility. The story should explicitly address whether `EngineMetadataError` is in scope for this policy. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S011-D3-01 | MEDIUM | The AST audit in AC-003 scans `monocle-core/src/`. But `ShutdownReason` (defined in S-007's `monocle-runtime/src/types.rs`) is also a public enum that S-007's Tasks specifies as `#[non_exhaustive]`. The audit does not cover `monocle-runtime`. Either: (a) the `#[non_exhaustive]` policy only applies to `monocle-core` public enums (and `monocle-runtime` follows the same policy by convention without an audit), or (b) the audit should cover both crates. The story should make this explicit. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | The syn 2 AST audit approach is sound. The compile-time enforcement via wildcard arm (AC-004) is correctly described (Rust enforces this at compile time; the test is the build). |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter complete. inputs versioned correctly including SS-permissions-phase1.md v1.5.2. |

## Research Queue

None.

## Recommended Fixes

1. S011-D2-01 (MEDIUM): Explicitly state whether `EngineMetadataError` is covered by the non-exhaustive policy. If yes, add it to the 9-enum canonical list (making it 10). If not (because it is in `monocle-core::engine` and only has one Phase 1 variant), document the rationale. Routing: architect (policy decision), then story-writer.
2. S011-D3-01 (MEDIUM): Clarify audit scope — `monocle-core/src/` only, or both crates. If `monocle-runtime` is excluded, add a note that it follows the same convention by policy but without an automated audit. Routing: story-writer.
