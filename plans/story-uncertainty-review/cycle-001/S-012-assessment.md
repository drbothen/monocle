---
document_type: story-uncertainty-assessment
story_id: S-012
story_version: "1.4"
story_title: FactoryAdapter Trait + VsddFactoryAdapter Implementation (FC-04)
assessment_batch: BATCH-4
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_REVISION
---

# Story Assessment: S-012

## Verdict

**NEEDS_REVISION** — Two CRITICAL findings. (1) `serde_yaml_ng 0.10` is listed as required
but is not in SS-deps-pin-manifest.md v1.1.17 — must be added to the manifest before use.
(2) The `parse_frontmatter_field` implementation (AC-013, 8 test vectors) has significant
complexity that warrants a separate test module, but the story assigns all of it to a single
test file (`factory_self_referential.rs`).

## Summary

S-012 is the highest-complexity `monocle-core` story (8 points, 13 ACs). The `FactoryAdapter`
trait is correctly specified (7 methods, no Sealed, `Send + Sync + 'static`). The
`VsddFactoryAdapter` implementation with frontmatter parsing is detailed and precise.
AC-013's parse guards (4 guards + unquoting) are exceptional specification work. The two
critical findings must be resolved before dispatch.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S012-D1-01 | CRITICAL | `serde_yaml_ng 0.10` is listed in Library & Framework Requirements but is NOT in SS-deps-pin-manifest.md v1.1.17. Per the project's Patch-Pinning Policy, all crates used in `monocle-core` must be declared in the manifest. `serde_yaml_ng` must be added to SS-deps-pin-manifest.md with a version pin and a caret or exact designation before this story is dispatched. The manifest also needs to declare whether this is a caret pin (`^0.10`) or EXACT pin. Routing: architect (manifest update) + story-writer (AC update). |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S012-D2-01 | MEDIUM | AC-004 specifies `custom_fields: std::collections::HashMap<String, serde_yaml_ng::Value>`. But `serde_yaml_ng::Value` is the YAML value type — it supports all YAML types including sequences and mappings. For the frontmatter parsing context, `custom_fields` accumulates all frontmatter keys that are not in the canonical 7-field list. This is correct, but the story should note that only scalar values will be usefully populated via `parse_frontmatter_field` — sequences and mappings will hit guard 3/4 and return `None`. This means `custom_fields` only captures scalar extras. |
| S012-D2-02 | MEDIUM | AC-012 specifies that `convergence: None` when "§Session Resume Checkpoint section is absent." But `ConvergenceMetrics` is specified in AC-003 as a struct in `monocle-core::factory`. The story does not define `ConvergenceMetrics` fields. What fields does `ConvergenceMetrics` have? The test-writer cannot write a failing test for `convergence: None` without knowing the struct shape. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S012-D3-01 | CRITICAL | AC-013's `parse_frontmatter_field` implementation has 8 test vectors with precise guard specifications. These are in `factory_self_referential.rs` (per Tasks). But `factory_self_referential.rs` is labeled as the VP-015 integration test (self-referential detection on the monocle repo). Mixing unit tests for `parse_frontmatter_field` with VP-015 integration tests in a single file is a test organization concern — the test file should be split into `factory_adapter_unit.rs` (for parse guards) and `factory_self_referential.rs` (for VP-015 integration). |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | The 8 test vectors in AC-013 are comprehensive and precise. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Frontmatter complete. inputs versioned correctly. `serde_yaml_ng 0.10` is the only missing reference. |

## Research Queue

None. `serde_yaml_ng` is a known crate; architect can pin it without external research.

## Recommended Fixes

1. S012-D1-01 (CRITICAL): Add `serde_yaml_ng` to SS-deps-pin-manifest.md with version pin before dispatching S-012. Routing: architect.
2. S012-D2-02 (MEDIUM): Define `ConvergenceMetrics` struct fields in AC-003 or Architecture Compliance Rules. Routing: architect (confirm fields from SS-daemon-lifecycle.md or SS-core-types-and-abi.md), then story-writer.
3. S012-D3-01 (CRITICAL severity): Split test files: add `factory_adapter_unit.rs` for AC-013 parse guard tests; keep `factory_self_referential.rs` for VP-015. Update File Structure Requirements. Routing: story-writer.
4. S012-D2-01 (MEDIUM): Add note that `custom_fields` only captures scalar extras from frontmatter (sequences/mappings are filtered by parse guards). Routing: story-writer.
