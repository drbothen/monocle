---
document_type: adversary-report
version: "1.0"
status: complete
producer: adversary
project: monocle
phase: phase-1-spec-crystallization
input-hash: "[live-state]"
traces_to: "Phase 1 PRD v1.1 f855835 + VP v1.1 8454ff2 + arch v1.0.8 2db408f; F-R62 fix-burst applied; D-047 strict pass 1 of 3"
level: ops
timestamp: 2026-05-14T23:55:00Z
pass_number: 1
policy: D-047-strict
---

# Adversarial Review Pass R63 — Phase 1 (D-047 Strict, Pass 1 of 3, Post-F-R62 Fix-Burst)

## Summary

**Verdict:** FINDINGS — pass 1 FAILS, counter resets to 0.

**Counts by severity:**
- CRITICAL: 0
- HIGH: 1
- MEDIUM: 1
- LOW: 0

**Total findings:** 2

## 22-BC ↔ 22-VP Mapping Audit

All 22 BCs have exactly one VP with matching ID. F-R62-4 test-file path reconciliation HELD across all 22 BCs (paths IDENTICAL between PRD and VP). Test-NAME coherence failed for 4 BCs (see F-R63-adv-1).

## Findings Table

| ID | Severity | Domain | File | Description | Recommended Route |
|----|----------|--------|------|-------------|-------------------|
| F-R63-adv-1 | HIGH | VP/PRD coherence | `.factory/specs/verification-properties.md` lines 994, 1538, 1598, 1656 | VP cites `(per PRD v1.1 §<BC>, Verification subsection)` for 4 test names that DIFFER from canonical PRD names. Falsified-source claim + name drift. | product-owner (adjudicate canonical names) → formal-verifier (propagate to VP) |
| F-R63-adv-2 | MEDIUM | Architecture partial-fix regression (S-7.01) | `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.8 line 318 | Arch BC-AUTH-002 §Verification cites stale `monocle-runtime/tests/auth.rs`. F-R62-4 split into `auth_token_lifecycle.rs` + `auth_header_rejection.rs` in PRD/VP but did not propagate back to architecture. Partial-fix regression. | architect (bump arch v1.0.9 with split paths) |

## Per-Finding Detail

### F-R63-adv-1 [HIGH] — VP test-name drift from canonical PRD source (4 VPs)

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/verification-properties.md`

**Evidence (4 instances):**

1. **VP-ABI-001** (line 994): VP claims `test_BC_ABI_001_status_endpoint_returns_abi_version_1 (per PRD v1.1 §BC-ABI-001, Verification subsection)`. PRD canonical: `test_BC_ABI_001_status_abi_version_field`.

2. **VP-ENGINE-002** (line 1538): VP claims `test_BC_ENGINE_002_claude_code_module_strict_basename_detect (per PRD v1.1 §BC-ENGINE-002, Verification subsection)`. PRD canonical: `test_BC_ENGINE_002_claude_code_module_detect`.

3. **VP-ENGINE-002-ERR** (line 1598): VP claims `test_BC_ENGINE_002_ERR_home_unresolvable_sync_and_async (per PRD v1.1 §BC-ENGINE-002-ERR, Verification subsection)`. PRD canonical: `test_BC_ENGINE_002_ERR_home_unresolvable_metadata_and_enrich`.

4. **VP-ENGINE-003** (line 1656): VP claims `test_BC_ENGINE_003_claude_module_inherent_hook_paths (per PRD v1.1 §BC-ENGINE-003, Verification subsection)`. PRD canonical: `test_BC_ENGINE_003_hook_paths_five_entries`.

**Why HIGH (not MEDIUM):** Falsified PG-4-style coherence claim. The VP's `(per PRD v1.1 §<BC>, Verification subsection)` annotation asserts the test name was sourced from PRD verbatim. F-R62-4 explicitly named PRD as test-path/test-name source-of-truth. Test PATHS reconciled correctly; test NAMES did not. For VP-ENGINE-002-ERR specifically, the divergence is semantic, not stylistic: PRD's `_metadata_and_enrich` suggests testing both `metadata()` and `enrich()` paths; VP's `_sync_and_async` suggests sync/async split. Different test design implications. The VP author should either adopt PRD's names verbatim OR surface the divergence to product-owner for adjudication. Neither happened.

**Recommended fix:** Route to product-owner to adjudicate the canonical name for each of the 4 BCs. Either PRD names win (VP updates 4 names) or VP names win (PRD updates 4 names). Both artifacts agree at end of burst before pass 2.

### F-R63-adv-2 [MEDIUM] — Architecture stale test path (S-7.01 partial-fix regression)

**File:** `/Users/jmagady/Dev/monocle/.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.8

**Heading:** §Start Sequence → step 3 inline `**Behavioral contracts:**` → BC-AUTH-002 block

**Evidence (line 318):**
`Verification: integration test in \`monocle-runtime/tests/auth.rs\`:` (followed by 6 test vectors)

The architecture v1.0.8 BC-AUTH-002 §Verification block cites a single `auth.rs` file. PRD v1.1 F-R62-4 split into:
- `monocle-runtime/tests/auth_token_lifecycle.rs` for BC-AUTH-001 (positive control)
- `monocle-runtime/tests/auth_header_rejection.rs` for BC-AUTH-002 (rejection probes)

PRD §7 RTM and VP §Coverage Matrix use the split paths. Architecture is the only artifact citing the single-file path.

**Why MEDIUM:** Per CLAUDE.md §Architectural Authority, PRD v1.1 (later, more-specific) overrides — implementer should follow PRD. But the architecture's stale path actively misleads any implementer who reads architecture first. F-R62-4 should have propagated back to architecture per S-7.01 partial-fix regression discipline.

**Recommended fix:** Architect bumps SS-daemon-lifecycle.md v1.0.8 → v1.0.9. Update line 318 to cite `auth_header_rejection.rs` for BC-AUTH-002. Add adjacent note (or separate verification entry) for BC-AUTH-001 → `auth_token_lifecycle.rs`. Add §Trace v1.0.9 entry documenting F-R62-4 back-propagation closure.

## Frozen META Residual Catalog Status (D-054)

| ID | Pattern | Re-litigated? |
|----|---------|---------------|
| F-R55-adv-1 | PG-4 em-dash separator codification gap | NO |
| F-R55-adv-3 | PG-4 intra-document scope hole | NO |
| F-R61-adv-1 | PG-3-CLASSIFICATION-EVIDENCE bare L-numbers in §Trace shorthand | NO |
| F-R61-2 | §Trace-Heading-Convention ADR/vision/brief equivalents | NO |

F-R63 findings target DIFFERENT META-classes (test-name coherence, architecture-back-propagation). Frozen-residual discipline preserved.

## Spec-Review Axis Sweep (selected)

| Axis | Result |
|------|--------|
| 22-BC ↔ 22-VP 1:1 mapping | PASS |
| Test-file path coherence (F-R62-4) | PASS — all 22 paths identical |
| Test-name coherence | **FAIL — 4 drifts** |
| Architecture path propagation (F-R62-4 back-propagation) | **FAIL — line 318** |
| BC-AUTH-002 taxonomy (arch v1.0.8 ↔ PRD ↔ VP) | PASS |
| VP frontmatter `phase` / `status` (F-R62-5) | PASS |
| VP-PROTO-002 Phase-4-only (F-R62-7) | PASS |
| §G-4 RESOLVED (F-R62-9) | PASS |
| §Trace v1.1 PG-4 sweep evidence | PASS |
| Production-grade language | PASS |
| PG-2 count coherence | PASS (no further count drift in PRD body) |

## Novelty Assessment

**MEDIUM-HIGH.** Both findings are novel — prior R62 round did not check whether test-name reconciliation matched test-path reconciliation, nor whether F-R62-4 propagated back to architecture. Fresh-context cross-grep of PRD and VP test names + architecture path citations exposed both.

**Convergence trajectory:** Path coherence held. Test-name coherence and architecture-back-propagation are the remaining drift surfaces. Once addressed, pass 2 should find substantially fewer surfaces.

## Pass 1 Verdict and Pass 2 Readiness

**Pass 1 verdict: FINDINGS — D-047 strict counter resets to 0.**

**Pass 2 readiness:** NOT READY. Two fixes required:
1. F-R63-adv-1: product-owner adjudication of canonical test names for VP-ABI-001, VP-ENGINE-002, VP-ENGINE-002-ERR, VP-ENGINE-003 → VP and/or PRD update.
2. F-R63-adv-2: architect bumps SS-daemon-lifecycle.md v1.0.8 → v1.0.9 propagating F-R62-4 path split to architecture.

After both routes complete and state-manager records the fix-burst, dispatch pass 2.
