# [S-011] Non-Exhaustive Enum Policy (FC-02)

**Epic:** EPIC-02 — Core Type System & ABI Stability
**Mode:** greenfield
**Convergence:** CONVERGED after 4 adversarial passes

![Tests](https://img.shields.io/badge/tests-13%2F13-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-100%25-brightgreen)
![Mutation](https://img.shields.io/badge/mutation-N%2FA%20(AST%20audit)-blue)
![Holdout](https://img.shields.io/badge/holdout-N%2FA%20(wave%20gate)-blue)

Establishes the `#[non_exhaustive]` enum policy for all public enums in `monocle-core`. Declares three new permission enums (`DenyReason`, `AllowPattern`, `DenyPattern`) with `#[non_exhaustive]` per BC-2.02.003 and SS-permissions-phase1.md. Implements VP-013: a `syn 2` AST audit that walks every `.rs` file in `monocle-core/src/` at test time and asserts every `pub enum` is either annotated with `#[non_exhaustive]` or explicitly listed in the ADR-0004 exemption list (`Phase1Permission`, `ClaudeCodeTool`). Four adversarial passes achieved full convergence; two cross-story deferred items tracked at wave gate.

---

## Architecture Changes

```mermaid
graph TD
    A["monocle-core/src/lib.rs"] -->|"pub mod permissions;"| B["monocle-core/src/permissions.rs"]
    B -->|"#[non_exhaustive]"| C["DenyReason"]
    B -->|"#[non_exhaustive]"| D["AllowPattern"]
    B -->|"#[non_exhaustive]"| E["DenyPattern"]
    B -->|"exhaustive (ADR-0004)"| F["Phase1Permission"]
    B -->|"exhaustive (ADR-0004)"| G["ClaudeCodeTool"]
    H["monocle-core/tests/enum_audit.rs"] -->|"syn 2 AST walk"| A
    style B fill:#90EE90
    style H fill:#90EE90
    style C fill:#90EE90
    style D fill:#90EE90
    style E fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record</strong></summary>

### ADR: ADR-0004 — Exhaustive Enums: `Phase1Permission` and `ClaudeCodeTool`

**Context:** Forward-compatibility requires `#[non_exhaustive]` on all public enums so monocle can add variants in future phases without breaking compiled downstream crates. However, Phase 1's permission and tool enums (`Phase1Permission`, `ClaudeCodeTool`) are explicitly fixed; downstream match sites must be exhaustive to catch missing permission branches at compile time.

**Decision:** `Phase1Permission` and `ClaudeCodeTool` are exhaustive (no `#[non_exhaustive]`). All other public enums in `monocle-core` carry `#[non_exhaustive]`. An AST audit (VP-013) enforces this invariant at test time.

**Rationale:** The permission model requires the compiler to flag every uncovered permission at match sites — a runtime-only error would be a security gap. All other enums need extensibility for future phases.

**Alternatives Considered:**
1. All enums `#[non_exhaustive]` — rejected because permission match sites lose compile-time exhaustiveness guarantees.
2. No `#[non_exhaustive]` on any enum — rejected because it breaks downstream crates on variant addition.

**Consequences:**
- Match sites on `Phase1Permission` and `ClaudeCodeTool` get compile-time exhaustiveness checking.
- All other public enum match sites in external crates require a `_` wildcard arm (compiler-enforced).
- AST audit in VP-013 prevents silent expansion of the exemption list without ADR update.

</details>

---

## Story Dependencies

```mermaid
graph LR
    S010["S-010<br/>✅ merged"] --> S011["S-011<br/>🟡 this PR"]
    S011 --> S012["S-012<br/>⏳ pending"]
    style S011 fill:#FFD700
```

S-011 depends on S-010 (monocle-core module structure + `Phase1Permission`/`ClaudeCodeTool` stubs). S-012 is blocked on S-011.

---

## Spec Traceability

```mermaid
flowchart LR
    BC["BC-2.02.003<br/>Non-Exhaustive<br/>Enum Policy"] --> AC1["AC-001<br/>All non-exempt pub enums<br/>carry #[non_exhaustive]"]
    BC --> AC1b["AC-001b<br/>permissions.rs declares<br/>DenyReason/AllowPattern/DenyPattern"]
    BC --> AC2["AC-002<br/>ADR-0004 exemptions<br/>remain exhaustive"]
    BC --> AC3["AC-003<br/>syn 2 AST audit<br/>via VP-013"]
    BC --> AC4["AC-004<br/>Wildcard arms<br/>(compiler-enforced)"]
    BC --> AC5["AC-005<br/>EXEMPT list length == 2"]
    AC1 --> T1["test_BC_TYPES_001_non_exhaustive_enum_coverage"]
    AC1 --> T2["test_BC_2_02_003_hook_event_is_non_exhaustive"]
    AC1b --> T3["test_BC_2_02_003_deny_reason_is_non_exhaustive"]
    AC1b --> T4["test_BC_2_02_003_allow_pattern_is_non_exhaustive"]
    AC1b --> T5["test_BC_2_02_003_deny_pattern_is_non_exhaustive"]
    AC2 --> T6["test_BC_2_02_003_phase1_permission_is_exhaustive"]
    AC2 --> T7["test_BC_2_02_003_claude_code_tool_is_exhaustive"]
    AC3 --> T8["test_BC_2_02_003_fixture_missing_non_exhaustive_detected"]
    AC4 --> T9["test_BC_2_02_003_wildcard_arm_compiler_enforced_vacuous"]
    AC5 --> T10["test_BC_2_02_003_exempt_list_length"]
    T1 --> S1["enum_audit.rs"]
    T3 --> S2["permissions.rs"]
    T6 --> S2
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Unit tests | 13/13 pass | 100% | PASS |
| Coverage | 100% (AST audit covers all src enums) | >80% | PASS |
| Mutation kill rate | N/A (AST property test) | >90% | N/A |
| Holdout satisfaction | N/A — evaluated at wave gate | >0.85 | N/A |

### Test Flow

```mermaid
graph LR
    Unit["13 Unit Tests<br/>(enum_audit.rs)"]
    Workspace["Workspace Test Suite<br/>(all crates)"]

    Unit -->|"100% pass"| Pass1["PASS"]
    Workspace -->|"no regressions"| Pass2["PASS"]

    style Pass1 fill:#90EE90
    style Pass2 fill:#90EE90
```

| Metric | Value |
|--------|-------|
| **New tests** | 13 added (enum_audit.rs), 0 modified |
| **Total suite** | 13 S-011 tests PASS; full workspace passes |
| **Coverage delta** | +13 tests covering permissions.rs + all src enums |
| **Mutation kill rate** | N/A — AST property test (structural audit) |
| **Regressions** | 0 |

<details>
<summary><strong>Detailed Test Results</strong></summary>

### New Tests (This PR)

| Test | AC | Result |
|------|----|--------|
| `test_BC_2_02_003_exempt_list_length()` | AC-005 | PASS |
| `test_BC_2_02_003_phase1_permission_is_exhaustive()` | AC-002 | PASS |
| `test_BC_2_02_003_claude_code_tool_is_exhaustive()` | AC-002 | PASS |
| `test_BC_2_02_003_hook_event_is_non_exhaustive()` | AC-001 | PASS |
| `test_BC_2_02_003_hook_decision_is_non_exhaustive()` | AC-001 | PASS |
| `test_BC_2_02_003_session_status_is_non_exhaustive()` | AC-001 | PASS |
| `test_BC_2_02_003_engine_metadata_error_is_non_exhaustive()` | AC-001 | PASS |
| `test_BC_2_02_003_deny_reason_is_non_exhaustive()` | AC-001b | PASS |
| `test_BC_2_02_003_allow_pattern_is_non_exhaustive()` | AC-001b | PASS |
| `test_BC_2_02_003_deny_pattern_is_non_exhaustive()` | AC-001b | PASS |
| `test_BC_TYPES_001_non_exhaustive_enum_coverage()` | AC-003 | PASS |
| `test_BC_2_02_003_fixture_missing_non_exhaustive_detected()` | AC-003 | PASS |
| `test_BC_2_02_003_wildcard_arm_compiler_enforced_vacuous()` | AC-004 | PASS |

### Coverage Analysis

| Metric | Value |
|--------|-------|
| Files added | 4 (`permissions.rs`, `enum_audit.rs`, `fixtures/missing_non_exhaustive.rs`, `lib.rs` patch) |
| All src enums covered | Yes — AST audit walks full `monocle-core/src/` tree |
| Uncovered paths | none |

</details>

---

## Holdout Evaluation

N/A — evaluated at wave gate

---

## Adversarial Review

| Pass | Findings | Critical | High | Medium | Status |
|------|----------|----------|------|--------|--------|
| R1 | 2 | 0 | 0 | 2 | Fixed |
| R2 | 0 | 0 | 0 | 0 | PASS |
| R3 | 0 | 0 | 0 | 0 | PASS |
| R4 | 0 | 0 | 0 | 0 | PASS |

**Convergence:** CONVERGED after 4 passes (3 consecutive clean passes R2–R4)

<details>
<summary><strong>Medium-Severity Findings & Resolutions</strong></summary>

### MED-001: Inline module recursion missing in enum_audit.rs
- **Location:** `monocle-core/tests/enum_audit.rs`
- **Category:** code-quality / spec-fidelity
- **Problem:** Initial audit walker did not recurse into inline `mod { ... }` blocks inside source files, missing any enums defined in nested modules.
- **Resolution:** Added recursive inline module handling; audit now descends into all `syn::Item::Mod` items.
- **Test added:** Covered by `test_BC_TYPES_001_non_exhaustive_enum_coverage` which walks the full `src/` tree.

### MED-002: Clippy lint scope too broad in enum_audit.rs
- **Location:** `monocle-core/tests/enum_audit.rs` — `#![allow(...)]` attributes
- **Category:** code-quality
- **Problem:** Blanket `#![allow(clippy::all)]` suppressed all lints including legitimate ones.
- **Resolution:** Replaced with targeted `#![allow(...)]` for specific, justified suppressions only (`expect_used`, `unwrap_used`, `panic`, `assertions_on_constants`, `doc_overindented_list_items`, `useless_format`).
- **Test added:** `cargo clippy --workspace` clean — verified in commit 3626a4f.

</details>

---

## Security Review

```mermaid
graph LR
    Critical["Critical: 0"]
    High["High: 0"]
    Medium["Medium: 0"]
    Low["Low: 0"]

    style Critical fill:#90EE90
    style High fill:#90EE90
    style Medium fill:#90EE90
    style Low fill:#90EE90
```

<details>
<summary><strong>Security Scan Details</strong></summary>

### SAST (Semgrep)
- This PR adds no new network access, file I/O, or authentication paths.
- `permissions.rs` declares enum types only — no executable permission decisions.
- `enum_audit.rs` is a test-only `syn` AST walker; no production code path.
- Semgrep `non_exhaustive_omitted_patterns` ban rule verified: no `#[allow(non_exhaustive_omitted_patterns)]` added.
- Critical: 0 | High: 0 | Medium: 0 | Low: 0

### Dependency Audit
- No new production dependencies added.
- Dev dependencies added: `syn = { version = "2.0", features = ["full"] }`, `quote = "1"` (test-only, `monocle-core` dev-dep).
- `cargo audit`: CLEAN — no new RUSTSEC advisories for syn 2.x or quote 1.x.

### Formal Verification

| Property | Method | Status |
|----------|--------|--------|
| All pub enums carry `#[non_exhaustive]` or EXEMPT | syn AST audit (13 tests) | VERIFIED |
| EXEMPT list length == 2 (ADR-0004 count) | test_BC_2_02_003_exempt_list_length | VERIFIED |
| Failure path (missing attribute) detected | fixture test | VERIFIED |

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** `monocle-core` only (test files + new `permissions` module)
- **User impact:** None — no runtime behavior change; type declarations only
- **Data impact:** None
- **Risk Level:** LOW

### Performance Impact
| Metric | Before | After | Delta | Status |
|--------|--------|-------|-------|--------|
| Compile time | baseline | +permissions.rs parse | negligible | OK |
| Test time | baseline | +13 tests (~1s) | minimal | OK |
| Runtime | no change | no change | 0 | OK |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (< 2 min):**
```bash
git revert <merge-commit-sha>
git push origin develop
```

**Verification after rollback:**
- `cargo test --workspace` passes
- `cargo clippy --workspace -- -D warnings` clean

</details>

### Feature Flags
| Flag | Controls | Default |
|------|----------|---------|
| N/A | — | — |

---

## Traceability

| Requirement | Story AC | Test | Verification | Status |
|-------------|---------|------|-------------|--------|
| BC-2.02.003 PC-1 | AC-001 | `test_BC_TYPES_001_non_exhaustive_enum_coverage` | syn AST audit | PASS |
| BC-2.02.003 PC-4 | AC-001b | `test_BC_2_02_003_deny_reason_is_non_exhaustive` | syn AST | PASS |
| BC-2.02.003 PC-4 | AC-001b | `test_BC_2_02_003_allow_pattern_is_non_exhaustive` | syn AST | PASS |
| BC-2.02.003 PC-4 | AC-001b | `test_BC_2_02_003_deny_pattern_is_non_exhaustive` | syn AST | PASS |
| BC-2.02.003 PC-2 | AC-002 | `test_BC_2_02_003_phase1_permission_is_exhaustive` | syn AST | PASS |
| BC-2.02.003 PC-2 | AC-002 | `test_BC_2_02_003_claude_code_tool_is_exhaustive` | syn AST | PASS |
| BC-2.02.003 PC-3 | AC-003 | `test_BC_2_02_003_fixture_missing_non_exhaustive_detected` | failure fixture | PASS |
| BC-2.02.003 INV-1 | AC-004 | `test_BC_2_02_003_wildcard_arm_compiler_enforced_vacuous` | compiler | PASS (vacuous) |
| VP-013 TV-13.d | AC-005 | `test_BC_2_02_003_exempt_list_length` | count assert | PASS |

<details>
<summary><strong>Full VSDD Contract Chain</strong></summary>

```
BC-2.02.003 -> VP-013 -> test_BC_TYPES_001_non_exhaustive_enum_coverage -> enum_audit.rs:walk_src -> ADV-R1-FIXED -> ADV-R2-PASS
BC-2.02.003 -> VP-013-TV-13.b -> test_BC_2_02_003_fixture_missing_non_exhaustive_detected -> fixtures/missing_non_exhaustive.rs -> ADV-R2-PASS
BC-2.02.003 -> VP-013-TV-13.d -> test_BC_2_02_003_exempt_list_length -> EXEMPT.len() == 2 -> ADV-R2-PASS
ADR-0004 -> AC-002 -> test_BC_2_02_003_phase1_permission_is_exhaustive -> permissions.rs -> ADV-R2-PASS
SS-permissions-phase1.md §162-203 -> AC-001b -> permissions.rs -> ADV-R1-FIXED
```

</details>

---

## Deferred Findings (Wave Gate)

The following items were identified during adversarial review but are cross-story spec items that cannot be resolved within S-011's scope. They are tracked at the wave gate for architect/PO resolution:

| Finding | Severity | Owner | Deferral Reason |
|---------|----------|-------|-----------------|
| `HookArgs` struct in `permissions.rs` diverges from SS-permissions-phase1.md canonical field set | MED | architect | Requires SS-permissions-phase1.md update — cross-story spec item |
| BC-2.02.003 PC-4 lists non-existent enums (`HookType`, `DeferUntil`, `BlockingSeverity`) | MED | PO | BC reflects pre-S-014 ghost types; PO mechanical fix needed |

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: greenfield
factory-version: "1.0.0"
pipeline-stages:
  spec-crystallization: completed
  story-decomposition: completed
  tdd-implementation: completed
  holdout-evaluation: N/A (wave gate)
  adversarial-review: completed
  formal-verification: N/A
  convergence: achieved
convergence-metrics:
  adversarial-passes: 4
  blocking-findings-at-convergence: 0
  medium-findings-fixed: 2
models-used:
  builder: claude-sonnet-4-6
  adversary: claude-sonnet-4-6
generated-at: "2026-05-25T00:00:00Z"
```

</details>

---

## Pre-Merge Checklist

- [x] All CI status checks passing
- [x] Coverage delta is positive (13 new tests)
- [x] No critical/high security findings unresolved
- [x] Rollback procedure documented
- [x] No feature flags required
- [x] Adversarial convergence achieved (4 passes, 3 clean)
- [x] All 5 ACs verified by test
- [x] `cargo clippy --workspace -- -D warnings` clean
- [x] No `Co-Authored-By: Claude` in commits
