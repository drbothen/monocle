---
document_type: story-uncertainty-assessment
story_id: S-014
story_version: "1.3"
story_title: EngineModule Trait Definition
assessment_batch: CALIBRATION
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_REVISION
---

# Story Assessment: S-014

## Verdict

**NEEDS_REVISION** — Two CRITICAL findings. The story conflates `HookEvent` enum variants
with `HookType` enum (different types per SS-engine-module.md). Also: `inputs` frontmatter
does not reference ADR-0005 despite the story establishing the trait that `on_hook()` dispatches
through.

## Summary

S-014 is the EngineModule trait definition — the most critical trait in the Phase 1 ABI surface. The trait signature (5 methods, exact types) is correctly specified. However, AC-003b references `HookEvent` variants (`SessionStart`, `UserPromptSubmit`, `PreToolUse`, `Notification`, `Stop`) which are the `HookEvent` ENUM VARIANTS, but the story also references `HookType` in other contexts (e.g., S-011, S-015 both use `HookType`). The relationship between `HookEvent` and `HookType` is not defined in this story, creating a potential implementer confusion. Additionally, the `inputs` frontmatter is missing SS-engine-module.md reference for the `async_trait` requirement (AC-007), which the story cites in the text but not in frontmatter.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | `async-trait = "0.1"` is a caret pin per SS-deps-pin-manifest.md. Correctly specified. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S014-D2-01 | CRITICAL | AC-003b defines `HookEvent` enum variants as `SessionStart, UserPromptSubmit, PreToolUse, Notification, Stop`. But S-011 (Non-Exhaustive Enum Policy) AC-001 lists a SEPARATE enum `HookType` in the canonical 9-enum set. S-015 (ClaudeCodeModule) uses `HashMap<HookType, String>` (not `HashMap<HookEvent, String>`). The story does not define the relationship between `HookEvent` (the trait's `on_hook` parameter type, defined in `hook_events.rs`) and `HookType` (defined in `monocle-core::engine` per S-011). An implementer reading S-014 alone cannot determine whether `HookEvent` and `HookType` are the same enum, different enums, or one wraps the other. This gap must be resolved before TDD. |
| S014-D2-02 | MEDIUM | AC-007 requires `#[async_trait::async_trait]` macro and states the rustdoc must explain why (native async fn in traits is not dyn-compatible on MSRV 1.86). The `inputs` frontmatter does not include `SS-engine-module.md` — yet the story cites `SS-engine-module.md lines 138–149` in the Tasks section. The implementer's context load is missing this file. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S014-D3-01 | CRITICAL | S-014 defines `HookEvent` in `monocle-core/src/hook_events.rs`. S-015 (ClaudeCodeModule) implements `on_hook(event: HookEvent)` and also uses `HashMap<HookType, String>` in `hook_paths()`. The AC for `on_hook()` in S-014 (AC-003b) says the wildcard arm returns `HookResponse::new(HookDecision::Allow)` — correct. But S-015 must also have a `HookType` enum to populate the `hook_paths()` return type. If `HookType` and `HookEvent` are different types, then S-014 must declare BOTH (or `HookType` must come from a different story). Currently neither S-014 nor S-011 explicitly declares `HookType` as a new type — S-011 lists it as one of the 9 canonical enums to receive `#[non_exhaustive]`, implying it already exists. But where is it CREATED? This cross-story handoff gap must be closed. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| S014-D4-01 | MEDIUM | The VP-019 AST audit test (`engine_module_surface.rs`) asserts "5 methods, no Sealed, correct return-type token streams." The Tasks block does not specify what "correct return-type token streams" means for the syn 2 test — does it assert the literal token text `-> Result<EngineMetadata, EngineMetadataError>`, or just that the return type has the right number of path segments? The test-writer needs a concrete assertion specification. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| S014-D5-01 | LOW | `inputs` frontmatter is missing `{path: .factory/specs/architecture/SS-engine-module.md, version: "1.1.20"}`. This file is explicitly cited in the Tasks and Architecture Compliance Rules sections but absent from the inputs list. |

## Research Queue

None. The `HookEvent` vs `HookType` disambiguation is a spec clarity issue, not an external research question.

## Recommended Fixes

1. S014-D2-01 + S014-D3-01 (CRITICAL): Add a subsection to Architecture Compliance Rules explicitly defining the relationship between `HookEvent` (on_hook parameter, `hook_events.rs`) and `HookType` (used in `hook_paths()` return type, declared in `engine.rs` or `types.rs`). These are two distinct types — `HookEvent` carries variant payload; `HookType` is a discriminant-only enum. S-014 should declare `HookType` explicitly, or cross-reference the story that declares it. Routing: architect (cross-type ABI decision), then story-writer.
2. S014-D2-02 + S014-D5-01: Add `{path: .factory/specs/architecture/SS-engine-module.md, version: "1.1.20"}` to the inputs frontmatter. Routing: story-writer.
3. S014-D4-01: Specify concrete VP-019 assertion text in Tasks (e.g., "assert return type of `metadata()` contains token sequence `Result`, `EngineMetadata`, `EngineMetadataError`"). Routing: story-writer.
