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
source_bc: BC-2.02.003
module: monocle-core
proof_method: ast-audit+mutation-test
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

# VP-013: Non-Exhaustive Enum Policy (Modulo ADR-0004 Exemptions)

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-TYPES-001 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

For every `pub enum E` defined in any source file of the `monocle-core`
crate, exactly one of the following holds:

1. `E` carries `#[non_exhaustive]`, OR
2. `E` is listed in the ADR-0004 exemption set
   `{ "Phase1Permission", "ClaudeCodeTool" }`.

No other exemption is allowed without a new ADR superseding ADR-0004.

## Source Contract

- **BC:** BC-2.02.003 — Enum Extensibility (Non-Exhaustive Default Policy).
- **Postcondition/Invariant:** BC-2.02.003 invariant 1 (every `pub enum` in
  `monocle-core` is `#[non_exhaustive]` modulo the explicit ADR-0004
  exemption set); supporting clippy lint configuration `non_exhaustive_omitted_patterns`
  deny-listed at workspace level.
- **Traces to (historical):** BC-TYPES-001 (SS-core-types-and-abi.md
  §Enum Extensibility; PRD v1.25 §BC-TYPES-001 Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| AST audit (primary) | syn 2 | Bounded — finite enum set in `monocle-core/src/**/*.rs` | All `pub enum` declarations in `monocle-core` |
| Mutation testing (auxiliary) | cargo-mutants | Bounded — mutation budget | `EXEMPT` constant length + `#[non_exhaustive]` attribute-presence check are mutation surfaces |
| Clippy lint (supplementary) | cargo clippy | N/A — static | `non_exhaustive_omitted_patterns` deny-listed for `#[allow]` (per SS-conventions-anti-patterns.md) |

## Mechanism

AST audit (primary, via a `syn 2` AST audit harness at
`monocle-core/tests/enum_audit.rs` parsing every
`monocle-core/src/**/*.rs` file and walking all `Item::Enum` nodes; per
PRD v1.25 §BC-TYPES-001 invariant 1; PRD v1.25 §7 RTM Test Type column
labels this BC `AST audit (syn 2)`); mutation-test (auxiliary); clippy
`non_exhaustive_omitted_patterns` lint configuration (supplementary).

## Pre-conditions

- `monocle-core` source tree is the audit scope.
- The exempt-list constant in the test harness is
  `EXEMPT: &[&str] = &["Phase1Permission", "ClaudeCodeTool"]`.

## Post-conditions

1. A test harness in `monocle-core/tests/enum_audit.rs` parses every
   `monocle-core/src/**/*.rs` file via `syn 2`, walks all `Item::Enum`
   nodes, and asserts that for each enum either `#[non_exhaustive]` is
   present in the attribute list OR the enum's identifier is in
   `EXEMPT`.
2. The test fails with a descriptive error listing every offending enum
   if the property is violated.
3. The `cargo clippy --workspace -- -D warnings` invocation passes with
   the project-local lint `non_exhaustive_omitted_patterns` deny-listed
   for `#[allow]` (per SS-conventions-anti-patterns.md).

## Counter-examples

1. A new contributor adds `pub enum NewError { ... }` to
   `monocle-core/src/` without `#[non_exhaustive]` and not in the
   exempt list — must fail the audit.
2. A contributor sneaks `#[allow(non_exhaustive_omitted_patterns)]` into
   a match site — must fail the clippy step (semgrep rule co-enforces,
   per SS-conventions-anti-patterns.md §Semgrep Rules).
3. A contributor adds `pub enum Phase2Permission` to `monocle-core/`
   (NOT in `monocle-plugin-sdk`) without `#[non_exhaustive]` and not in
   the exempt list — must fail the audit. (Even though ADR-0004
   contemplates a parallel `Phase 3` enum in the plugin SDK, that enum
   is in a different crate and the audit is `monocle-core`-scoped.)
4. Exempt list expanded silently to add a third enum without an ADR
   superseding ADR-0004 — covered by an orthogonal consistency check
   that greps for the `EXEMPT` constant length and asserts it equals
   the count of exhaustive enums documented in ADR-0004 (currently 2).

## Mutation-Test Rationale

Mutating the `EXEMPT` constant length (e.g., adding a stray entry) or
the `#[non_exhaustive]` attribute presence check (e.g., flipping
`has_attr` to `!has_attr`) must be caught by the audit harness — this
is a high-leverage mutation surface. `cargo-mutants` is the auxiliary
mechanism: a mutant that flips the attribute-presence check or grows
the exempt list silently must be killed by an existing failing test.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 13.a | Walk all `pub enum` in `monocle-core/src/**/*.rs` | Each is `#[non_exhaustive]` OR in EXEMPT |
| 13.b | Inject `pub enum NewError { ... }` without `#[non_exhaustive]` | Audit fails with descriptive error |
| 13.c | Inject `#[allow(non_exhaustive_omitted_patterns)]` | Clippy step fails |
| 13.d | Expand EXEMPT list silently | Consistency check fails (length mismatch with ADR-0004 documented count) |
| 13.e | cargo-mutants flips `has_attr` to `!has_attr` | Mutation killed by audit failure |

## Harness Location

- `monocle-core/tests/enum_audit.rs` (AST audit)
- Test name: `test_BC_TYPES_001_non_exhaustive_enum_coverage` (per PRD
  v1.25 §BC-TYPES-001, Verification subsection — to be migrated to
  `test_BC_2_02_003_non_exhaustive_enum_coverage` post BC renumber
  propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-TYPES-001 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.003.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §Enum Extensibility.
- ADR: `architecture/adr/ADR-0004` (non-exhaustive default policy with
  enumerated exemption set).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.003 (Dispatch 4 commit 1030c65).
- Conventions: `architecture/SS-conventions-anti-patterns.md` §Semgrep Rules.
