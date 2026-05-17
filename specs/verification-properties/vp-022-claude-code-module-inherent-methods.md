---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.03.004
module: monocle-runtime
proof_method: integration-test
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-022: `hook_paths()` Returns Exactly 5 Entries — One per `HookType` Variant

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-ENGINE-003 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

`ClaudeCodeModule::hook_paths()` returns a structure containing exactly
5 entries, one per `HookType` variant. The path strings are exactly:

| HookType variant | Path |
|------------------|------|
| `SessionStart` | `/hooks/session-start` |
| `UserPromptSubmit` | `/hooks/prompt-submit` |
| `PreToolUse` | `/hooks/pre-tool-use` |
| `Notification` | `/hooks/notification` |
| `Stop` | `/hooks/stop` |

## Source Contract

- **BC:** BC-2.03.004 — `ClaudeCodeModule` Inherent Methods
  (`hook_paths()` 5-Entry Canonical Set).
- **Postcondition/Invariant:** BC-2.03.004 §Postconditions asserting
  `hook_paths()` returns exactly 5 entries with the exact path strings
  per the canonical `HookType` variant → path mapping; `spawn()` and
  `preflight()` remain inherent methods (NOT trait methods); ABI
  versioning via `monocle_core::MONOCLE_ABI_VERSION` const (cross-property
  with VP-012), NOT via a trait method.
- **Traces to (historical):** BC-ENGINE-003 (SS-engine-module.md
  §Struct-level inherent operations; PRD v1.25 §BC-ENGINE-003 Verification
  subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test | Bounded — 5-variant exhaustive match | Length assertion + per-variant exhaustive path-string match; orthogonal source-grep against `monocle-core/src/engine.rs` to confirm `spawn()` / `preflight()` are NOT on the `EngineModule` trait |

## Mechanism

Integration test (harness located at
`monocle-runtime/tests/engine_module_claude_methods.rs` — files in
`<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test Type
column labels this BC `Unit` referring to conceptual scope, but the
harness layout is cargo-integration per file location). The harness
asserts `module.hook_paths().len() == 5`, exhaustively matches each
`HookType` variant to verify path-string equality with the canonical
table, and runs an orthogonal source-grep against
`monocle-core/src/engine.rs` to confirm `spawn()` and `preflight()` are
NOT declared as trait methods.

## Pre-conditions

- `ClaudeCodeModule::new("http://127.0.0.1:7891".into())` constructs a
  module.
- `HookType` is the canonical 5-variant enum from
  `monocle_core::HookType`.

## Post-conditions

1. `module.hook_paths().len() == 5`.
2. For each `HookType` variant `v`, `module.hook_paths().get(&v)`
   returns `Some(&"/hooks/...".to_string())` matching the table above
   exactly.
3. No extra variants exist (the `match` over `HookType` is exhaustive
   in the harness — adding a 6th variant would fail to compile, which
   is the correct propagation given `#[non_exhaustive]` on `HookType` is
   for external consumers; the trait implementer (this crate) is
   internal and so `HookType` exhaustive matching is valid here).

## Counter-examples

1. `hook_paths()` returns 4 entries (missing one) — fails
   post-condition 1.
2. A path is typoed (`/hooks/pre_tool_use` with underscore instead of
   hyphen) — fails the exact-string match.
3. A new variant added to `HookType` (e.g., `PostToolUse`) without
   updating `hook_paths()` — the exhaustive match in the harness fails
   to compile, forcing the implementer to update.
4. `spawn()` or `preflight()` are accidentally moved into the
   `EngineModule` trait (they MUST remain inherent methods on
   `ClaudeCodeModule`) — fails an orthogonal source-grep check against
   `monocle-core/src/engine.rs`.
5. The ABI version is read via a trait method (e.g., `module.abi_version()`)
   instead of `monocle_core::MONOCLE_ABI_VERSION` const — fails an
   orthogonal source-grep check (cross-property with VP-012).

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 22.a | `module.hook_paths().len()` | `== 5` |
| 22.b | Exhaustive `match v { SessionStart => "/hooks/session-start", ... }` for all 5 variants | Each variant maps to the exact canonical path string |
| 22.c | Source-grep `monocle-core/src/engine.rs` for `fn spawn(` or `fn preflight(` inside `trait EngineModule { ... }` | No match — inherent methods only |
| 22.d | Source-grep `monocle-core/src/engine.rs` for `fn abi_version(` inside `trait EngineModule { ... }` | No match — ABI is the `MONOCLE_ABI_VERSION` const (cross-VP-012) |
| 22.e | Mutation: typo `/hooks/pre_tool_use` (underscore) | Probe 22.b fails string equality |
| 22.f | Mutation: add 6th `HookType` variant `PostToolUse` without updating `hook_paths()` | Exhaustive match in harness fails to compile |

## Harness Location

- `monocle-runtime/tests/engine_module_claude_methods.rs` (integration test)
- Test name: `test_BC_ENGINE_003_claude_module_hook_paths_five_entries`
  (per PRD v1.25 §BC-ENGINE-003, Verification subsection — hybrid name
  adjudicated by product-owner combining `claude_module` (concrete
  struct under test, not the trait), `hook_paths` (the inherent
  method), and `five_entries` (the count assertion); see PRD v1.2
  §Trace v1.2 for the adjudication reasoning. To be migrated to
  `test_BC_2_03_004_claude_module_hook_paths_five_entries` post BC
  renumber propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-ENGINE-003 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-03/BC-2.03.004.md`.
- Architecture: `architecture/SS-engine-module.md` §Struct-level inherent operations.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.03.004 (Dispatch 4 commit 1030c65).
- Cross-VP: VP-012 (`MONOCLE_ABI_VERSION` const — ABI versioning channel
  for `monocle-core`; orthogonal source-grep prevents accidental trait
  method `abi_version()`); VP-019 (trait surface that `ClaudeCodeModule`
  implements — `spawn()` and `preflight()` must remain inherent, not
  trait, methods); VP-013 (`#[non_exhaustive]` policy — `HookType` external
  consumer protection that does NOT prevent internal exhaustive matching).
