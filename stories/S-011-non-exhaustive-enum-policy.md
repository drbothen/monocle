---
document_type: story
story_id: S-011
epic_id: EPIC-02
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 3
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-010]
blocks: [S-012]
target_module: monocle-core
subsystems: [SS-02]
behavioral_contracts: [BC-2.02.003]
verification_properties: [VP-013]
estimated_days: 1
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.11"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.003.md, version: "1.0.2"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-013-non-exhaustive-enum-policy.md, version: "1.0.12"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.10"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/architecture/SS-conventions-anti-patterns.md, version: "1.29.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.02.003 (Non-Exhaustive Enum Policy FC-02); verifies VP-013; establishes #[non_exhaustive] discipline for all public enums."
---

# S-011: Non-Exhaustive Enum Policy (FC-02)

## Narrative

As a downstream consumer of `monocle-core` types, I want all public enums (except
`Phase1Permission` and `ClaudeCodeTool` which are exhaustive per ADR-0004) to carry
`#[non_exhaustive]`, so that monocle can add variants in future phases without breaking
compiled downstream crates.

## Acceptance Criteria

### AC-001 (traces to BC-2.02.003 postcondition 1 — non_exhaustive on public enums)
All `pub enum` types in `monocle-core` that are not explicitly exempt per ADR-0004
carry the `#[non_exhaustive]` attribute. This includes `HookType`, `HookDecision`,
`DeferUntil`, `BlockingSeverity`, `SessionStatus`.

### AC-002 (traces to BC-2.02.003 postcondition 2 — ADR-0004 exemptions are exhaustive)
`Phase1Permission` and `ClaudeCodeTool` are `pub enum` without `#[non_exhaustive]`
per ADR-0004. Downstream code that matches on these enums gets compile-time exhaustiveness
checking (the intended behavior — Phase 1 sets are fixed).

### AC-003 (traces to BC-2.02.003 postcondition 3 — AST audit via VP-013)
`monocle-core/tests/non_exhaustive_policy.rs` uses a `syn 2` AST audit to enumerate all
`pub enum` declarations in `monocle-core/src/` and assert that each is either:
(a) marked `#[non_exhaustive]`, or (b) in the ADR-0004 exemption list
(`Phase1Permission`, `ClaudeCodeTool`).

### AC-004 (traces to BC-2.02.003 invariant 1 — match arms on non_exhaustive enums have wildcard)
Integration test verifies that match expressions on `HookType` in `monocle-runtime`
have a wildcard `_` arm (required by Rust compiler for `#[non_exhaustive]` enums
from external crates). Compiler error at build time if wildcard is missing.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~600 |
| BC-2.02.003.md | ~500 |
| VP-013 file | ~400 |
| ADR-0004 (exhaustive enums exemption) | ~700 |
| syn 2 AST audit pattern | ~400 |
| Test file | ~500 |
| **Total estimate** | **~3,100** |

## Tasks

- [ ] Add `#[non_exhaustive]` to: `HookType`, `HookDecision`, `DeferUntil`, `BlockingSeverity`, `SessionStatus`
- [ ] Confirm `Phase1Permission` and `ClaudeCodeTool` have NO `#[non_exhaustive]` (ADR-0004)
- [ ] Create `monocle-core/tests/non_exhaustive_policy.rs` with syn 2 AST audit:
  - Parse all `.rs` files in `monocle-core/src/`
  - For each `pub enum`: assert `#[non_exhaustive]` present OR name in exemption list
- [ ] Verify that match expressions in `monocle-runtime` on `HookType` have wildcard arm
  - Compiler enforces this; test is the build itself

## Previous Story Intelligence

S-010 (Wave 2): `monocle-core/src/lib.rs` with module structure established.
`HookType` and related enums are stubbed in `monocle-core`. This story adds the
`#[non_exhaustive]` attributes and the AST audit test.

## Architecture Compliance Rules

From `architecture/SS-core-types-and-abi.md` v1.2.13 §Non-Exhaustive Enum Policy:
- `#[non_exhaustive]` is MANDATORY on all public monocle-core enums except ADR-0004 exemptions
- ADR-0004 exemptions: `Phase1Permission`, `ClaudeCodeTool`
- All match arms on non_exhaustive external enums must have `_` wildcard (compiler enforced)

**Forbidden Dependencies:**
- `#[non_exhaustive]` MUST NOT be applied to `Phase1Permission` or `ClaudeCodeTool`

## Library & Framework Requirements

| Crate | Version | Usage (test only) |
|-------|---------|-------|
| syn | 2 | AST audit in test (dev-dependency) |
| quote | 1 | Token stream in AST audit (dev-dependency) |

## File Structure Requirements

Files to create:
- `monocle-core/tests/non_exhaustive_policy.rs` — syn 2 AST audit

Files to modify:
- `monocle-core/src/engine.rs` — add `#[non_exhaustive]` to `HookType`, `HookDecision`, `DeferUntil`
- `monocle-core/src/factory.rs` — add `#[non_exhaustive]` to `BlockingSeverity`
- `monocle-core/src/types.rs` — add `#[non_exhaustive]` to `SessionStatus`
- `monocle-core/Cargo.toml` — add `syn = { version = "2", features = ["full"] }` and `quote = "1"` to `[dev-dependencies]`
