---
document_type: story
level: L4
story_id: S-011
epic_id: EPIC-02
version: "1.3"
status: done
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
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-02/BC-2.02.003.md, version: "1.0.2"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-013-non-exhaustive-enum-policy.md, version: "1.0.12"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
  - {path: .factory/specs/architecture/SS-conventions-anti-patterns.md, version: "1.29.5"}
  - {path: .factory/specs/architecture/SS-permissions-phase1.md, version: "1.5.2"}
  - {path: .factory/specs/architecture/adr/ADR-0004-exhaustive-enums-phase1-permission-and-claude-code-tool.md, version: "1.0.4"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.18"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.02.003 (Non-Exhaustive Enum Policy FC-02); verifies VP-013; establishes #[non_exhaustive] discipline for all public enums."
---

# S-011: Non-Exhaustive Enum Policy (FC-02)

## Narrative

As a downstream consumer of `monocle-core` types, I want all public enums (except
`Phase1Permission` and `ClaudeCodeTool` which are exhaustive per ADR-0004) to carry
`#[non_exhaustive]`, so that monocle can add variants in future phases without breaking
compiled downstream crates.

Note: Three permissions enums (`DenyReason`, `AllowPattern`, `DenyPattern`) are first-declared
in S-011, not S-010. S-010 only declares `Phase1Permission` and `ClaudeCodeTool`.

## Acceptance Criteria

### AC-001 (traces to BC-2.02.003 postcondition 1 + postcondition 4 — non_exhaustive on all 9 canonical public enums)
All `pub enum` types in `monocle-core` that are not explicitly exempt per ADR-0004
carry the `#[non_exhaustive]` attribute. This includes the full canonical minimum-9 list
from BC-2.02.003 PC-4: `HookType`, `HookEvent`, `HookDecision`, `DeferUntil`,
`BlockingSeverity`, `SessionStatus` — PLUS the three permissions enums declared in
`SS-permissions-phase1.md` lines 162–203:
- `DenyReason` (in `monocle-core/src/permissions.rs`)
- `AllowPattern` (in `monocle-core/src/permissions.rs`)
- `DenyPattern` (in `monocle-core/src/permissions.rs`)
All 9 enums carry `#[non_exhaustive]`; the `syn 2` AST audit in VP-013 verifies all.

### AC-001b (traces to BC-2.02.003 postcondition 4 — permissions enums declared in monocle-core)
`monocle-core/src/permissions.rs` declares `DenyReason`, `AllowPattern`, and `DenyPattern`
with `#[non_exhaustive]` attributes per SS-permissions-phase1.md §Permission Types lines 162–203.
These enums implement the permissions type surface without duplication of the Phase 1
permission decision model. The module is re-exported from `monocle-core/src/lib.rs` as
`pub mod permissions;`.

### AC-002 (traces to BC-2.02.003 postcondition 2 — ADR-0004 exemptions are exhaustive)
`Phase1Permission` and `ClaudeCodeTool` are `pub enum` without `#[non_exhaustive]`
per ADR-0004. Downstream code that matches on these enums gets compile-time exhaustiveness
checking (the intended behavior — Phase 1 sets are fixed).

### AC-003 (traces to BC-2.02.003 postcondition 3 — AST audit via VP-013)
`monocle-core/tests/enum_audit.rs` uses a `syn 2` AST audit to enumerate all
`pub enum` declarations in `monocle-core/src/` and assert that each is either:
(a) marked `#[non_exhaustive]`, or (b) in the ADR-0004 exemption list
(`Phase1Permission`, `ClaudeCodeTool`).

### AC-004 (traces to BC-2.02.003 invariant 1 — match arms on non_exhaustive enums have wildcard)
Integration test verifies that match expressions on `HookType` in `monocle-runtime`
have a wildcard `_` arm (required by Rust compiler for `#[non_exhaustive]` enums
from external crates). Compiler error at build time if wildcard is missing.
Contingency: if `monocle-runtime` contains zero match sites on `monocle-core` non-exhaustive enums
at S-011 dispatch, AC-004 is satisfied vacuously; if matches exist, all MUST have wildcard arms.

### AC-005 (traces to BC-2.02.003 postcondition 3 — EXEMPT list length and contents)
The ADR-0004 exemption list (`EXEMPT: [Phase1Permission, ClaudeCodeTool]`) declared in
`monocle-core/tests/enum_audit.rs` has exactly 2 entries, matching the documented count in
ADR-0004. If the constant has != 2 entries, the audit test MUST fail with a message indicating
the EXEMPT list length has changed from the ADR-0004-documented count. This prevents silent
expansion of the exemption list without a corresponding ADR update.
(Per VP-013 Test Vector 13.d: "EXEMPT list expanded silently → consistency check fails".)

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~800 |
| BC-2.02.003.md | ~500 |
| VP-013 file | ~400 |
| ADR-0004 (exhaustive enums exemption) | ~700 |
| SS-permissions-phase1.md (lines 156-212, permissions enums) | ~600 |
| syn 2 AST audit pattern | ~400 |
| Test file | ~600 |
| **Total estimate** | **~4,000** |

## Tasks

- [ ] Add `#[non_exhaustive]` to: `HookType`, `HookEvent`, `HookDecision`, `DeferUntil`, `BlockingSeverity`, `SessionStatus`
- [ ] Create `monocle-core/src/permissions.rs` declaring `DenyReason`, `AllowPattern`, `DenyPattern` with `#[non_exhaustive]` attribute (SS-permissions-phase1.md lines 162–203; BC-2.02.003 PC-4)
- [ ] Add `pub mod permissions;` to `monocle-core/src/lib.rs`
- [ ] Confirm `Phase1Permission` and `ClaudeCodeTool` have NO `#[non_exhaustive]` (ADR-0004)
- [ ] Create `monocle-core/tests/enum_audit.rs` with syn 2 AST audit:
  - Parse all `.rs` files in `monocle-core/src/`
  - For each `pub enum`: assert `#[non_exhaustive]` present OR name in exemption list
- [ ] Verify that match expressions in `monocle-runtime` on `HookType` have wildcard arm
  - Compiler enforces this; test is the build itself
- [ ] Create `monocle-core/tests/fixtures/missing_non_exhaustive.rs` with `#![cfg(test)]` ignore
  guard — a synthetic source file containing a `pub enum` without `#[non_exhaustive]` that the
  AST audit test parses to exercise the failure path (VP-013 Test Vector 13.b: "enum missing
  #[non_exhaustive] → audit fails"). This fixture is parsed directly by the test, not compiled.

## Previous Story Intelligence

S-010 (Wave 2): `monocle-core/src/lib.rs` with module structure established.
`HookType` and related enums are stubbed in `monocle-core`. This story adds the
`#[non_exhaustive]` attributes and the AST audit test.

## Architecture Compliance Rules

From `architecture/SS-core-types-and-abi.md` v1.2.13 §Non-Exhaustive Enum Policy:
- `#[non_exhaustive]` is MANDATORY on all public monocle-core enums except ADR-0004 exemptions
- ADR-0004 exemptions: `Phase1Permission`, `ClaudeCodeTool`
- All match arms on non_exhaustive external enums must have `_` wildcard (compiler enforced)

**Semgrep co-enforcement:** `SS-conventions-anti-patterns.md §Semgrep Rules` defines a semgrep rule
that co-enforces the `#[allow(non_exhaustive_omitted_patterns)]` ban (per VP-013 lines 101-102).
No new semgrep rule is required for this story — the existing rule covers the ban.

**Forbidden Mechanisms:** `SS-core-types-and-abi.md v1.2.13 §Forbidden Mechanisms` (lines 289-299)
denies `#[allow(non_exhaustive_omitted_patterns)]`. Implementer MUST NOT add this attribute to silence
any compiler lint — doing so bypasses the exhaustiveness guarantee that `#[non_exhaustive]` provides.

**Forbidden Dependencies:**
- `#[non_exhaustive]` MUST NOT be applied to `Phase1Permission` or `ClaudeCodeTool`

## Library & Framework Requirements

| Crate | Version | Usage (test only) |
|-------|---------|-------|
| syn | 2.0 (caret) | AST audit in test (dev-dependency); per SS-deps-pin-manifest v1.1.18 §Phase 1 Pin Manifest <!-- version-pin-historical: at S-011 authoring time --> |
| quote | 1 | Token stream in AST audit (dev-dependency) |

## File Structure Requirements

Files to create:
- `monocle-core/src/permissions.rs` — `DenyReason`, `AllowPattern`, `DenyPattern` enums (all `#[non_exhaustive]`)
- `monocle-core/tests/enum_audit.rs` — syn 2 AST audit covering all 9 canonical enums
  (canonical name per BC-2.02.003 Invariant 1 line 51 and VP-013 §References — was non_exhaustive_policy.rs; F-C-01 fix)
- `monocle-core/tests/fixtures/missing_non_exhaustive.rs` — synthetic failure fixture with `#![cfg(test)]`
  ignore guard; parsed by enum_audit.rs to exercise VP-013 test vector 13.b failure path

Files to modify:
- `monocle-core/src/engine.rs` — add `#[non_exhaustive]` to `HookType`, `HookEvent`, `HookDecision`
  (NOTE: `DeferUntil` removed from engine module surface per S-014 F-D-03; do NOT add to engine.rs)
- `monocle-core/src/factory.rs` — add `#[non_exhaustive]` to `BlockingSeverity`
- `monocle-core/src/types.rs` — add `#[non_exhaustive]` to `SessionStatus`
- `monocle-core/src/lib.rs` — add `pub mod permissions;`
- `monocle-core/Cargo.toml` — add `syn = { version = "2.0", features = ["full"] }` and `quote = "1"` to `[dev-dependencies]`

## §Trace

**v1.3** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at story authoring time. No normative content changed.

**v1.2 — Phase 3.B Batch 3: arch-touching story remediation** (2026-05-20):
- F-C-01 (HIGH): test file renamed `non_exhaustive_policy.rs` → `enum_audit.rs` throughout (BC-2.02.003
  Invariant 1 line 51 and VP-013 §References both specify `enum_audit.rs`).
- F-B-01 (MEDIUM): ADR-0004 v1.0.4 added to frontmatter inputs.
- F-A-01 (MEDIUM): SS-deps-pin-manifest v1.1.18 added to frontmatter inputs; syn version updated to
  "2.0 (caret)" with SS-deps-pin-manifest cite in Library table.
- F-A-02 (LOW): semgrep co-enforcement note added to Architecture Compliance Rules.
- F-B-02 (LOW): SS-core-types-and-abi §Forbidden Mechanisms note added (deny #[allow(non_exhaustive_omitted_patterns)]).
- F-C-02 (MEDIUM): AC-005 added — EXEMPT list length == 2 consistency check per VP-013 Test Vector 13.d.
- F-D-03 (LOW): synthetic-failure fixture file added to Tasks and File Structure.
- F-C-03 (LOW): AC-004 contingency clause added (vacuous satisfaction if monocle-runtime has no match sites).
- F-E-02 (LOW): Narrative note added — three permissions enums first-declared in S-011, not S-010.
- DeferUntil note added to File Structure: do NOT add #[non_exhaustive] to DeferUntil in engine.rs (ghost type dropped in S-014 F-D-03).
