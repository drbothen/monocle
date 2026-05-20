---
document_type: story-uncertainty-assessment
story_id: S-015
story_version: "1.5"
story_title: ClaudeCodeModule Implementation
assessment_batch: BATCH-4
cycle: cycle-001
assessor: vsdd-factory:spec-reviewer
timestamp: 2026-05-20T07:00:00Z
verdict: NEEDS_REVISION
---

# Story Assessment: S-015

## Verdict

**NEEDS_REVISION** — One CRITICAL finding inheriting from S-014: `HookType` vs `HookEvent`
disambiguation. S-015 implements `on_hook(event: HookEvent)` and returns
`HashMap<HookType, String>` from `hook_paths()`. Until S-014-D2-01 is resolved (defining
the relationship between `HookEvent` and `HookType`), S-015 cannot be dispatched.

## Summary

S-015 is the ClaudeCodeModule implementation (8 points, 10 ACs). The strict basename
detection logic is precisely specified with the exact two allowed basenames (`"claude"` and
`"claude.js"`). The `HomeUnresolvable` error path is unambiguous. The `hook_paths()` return
type (`HashMap<HookType, String>`) is correctly specified. The `todo!()` stubs for `spawn()`
and `preflight()` are explicitly documented with their Phase 3 replacement story note. The
critical finding is the inherited `HookType` vs `HookEvent` disambiguation from S-014.

## Dimension Findings

### D1 — Version Pin Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | `directories 6`, `async-trait 0.1`, `temp-env 0.3`, `tracing 0.1` are all correctly specified. The `which` crate is explicitly noted as Phase 3 scope and NOT added to Phase 1 — correct. |

### D2 — API Accuracy

| ID | Severity | Finding |
|----|----------|---------|
| S015-D2-01 | CRITICAL | AC-007 specifies `hook_paths() -> HashMap<HookType, String>` with 5 entries keyed by `HookType::SessionStart`, `HookType::UserPromptSubmit`, etc. But `HookType` is NOT defined in S-015's scope (it is referenced from `monocle-core::engine` or `monocle-core::types`). Until S-014-D2-01 is resolved (clarifying whether `HookType` is declared in S-014 or elsewhere), the S-015 implementer cannot know which module to import `HookType` from. |
| S015-D2-02 | MEDIUM | AC-010 specifies `on_hook()` returns `HookResponse::new(HookDecision::Allow)` for unrecognized `HookEvent` variants (wildcard arm). This is correct. But S-015's `inputs` frontmatter does not include SS-core-types-and-abi.md (which defines `HookResponse`, `HookDecision`). These types are defined in `monocle-core` per S-014, but the story inputs should reference the relevant spec. |

### D3 — Cross-Story Contracts

| ID | Severity | Finding |
|----|----------|---------|
| S015-D3-01 | LOW | S-015 Implementation Note correctly notes that `which 0.x` is NOT a Phase 1 dependency and must be added to SS-deps-pin-manifest.md by the architect when the Phase 3 preflight story is created. This is correctly documented. No action needed. |
| S015-D3-02 | MEDIUM | S-015 creates `monocle-runtime/src/engine/claude_code.rs` and `monocle-runtime/src/engine/mod.rs`. But no prior story creates the `monocle-runtime/src/engine/` directory/module. The story should note that S-015 creates this new sub-module structure and `monocle-runtime/src/lib.rs` must be updated to add `pub mod engine;`. This is present in File Structure Requirements ("Files to modify: monocle-runtime/src/lib.rs — engine module is now `pub mod engine` (sub-module)") but the rationale (it was previously a different structure) is unclear for implementers starting fresh. |

### D4 — Test Coverage Completeness

| ID | Severity | Finding |
|----|----------|---------|
| (none) | — | Test coverage is comprehensive. Detect tests cover all 6 enumerated cases. VP-021 (HomeUnresolvable) uses `temp-env 0.3 async_with_vars` correctly. VP-022 (`hook_paths().len() == 5`) is explicit. |

### D5 — Structural Integrity

| ID | Severity | Finding |
|----|----------|---------|
| S015-D5-01 | MEDIUM | `inputs` frontmatter is missing `{path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}`. This file defines `HookResponse`, `HookDecision`, and `DeferUntil` which are used in `on_hook()` implementation. |

## Research Queue

None. The `HookType` disambiguation is a spec clarity gap, not an external research question.

## Recommended Fixes

1. S015-D2-01 (CRITICAL): Blocked on S014-D2-01. After S-014 resolves the `HookType` vs `HookEvent` relationship and declares `HookType`, update S-015's Architecture Compliance Rules to specify the import path for `HookType`. Routing: story-writer (after S-014 fix).
2. S015-D2-02 + S015-D5-01 (MEDIUM): Add `{path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}` to inputs frontmatter. Routing: story-writer.
3. S015-D3-02 (MEDIUM): Add note clarifying that S-015 creates the `engine/` sub-module structure (new to this story). Routing: story-writer.
