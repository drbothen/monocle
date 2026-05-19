---
document_type: tech-debt-register
level: ops
status: active
producer: product-owner
version: "1.0"
last_updated: 2026-05-12T08:30:00Z
inputs: []
input-hash: "[live-state]"
traces_to: "CLAUDE.md commit 3366d58 §Canonical Principle Rule 3; ADR-0002"
project: monocle
phase: pre-phase-1
---

# Technical Debt Register

**Governance:** Per `CLAUDE.md` Canonical Principle Rule 3, entries in this register MUST be human-directed deferrals with concrete future story/wave attachment. Agent-discovered issues that can be fixed in-scope must NOT be added here — they must be fixed via the correct specialist agent in the current cycle. This register is NOT a default catchment.

## Summary

| Priority | Count | Estimated Points |
|----------|-------|-----------------|
| P0 (next cycle) | 0 | 0 |
| P1 (within 3 cycles) | 0 | 0 |
| P2 (backlog) | 0 | 0 |

## Debt Items

| ID | Source | Description | Priority | Introduced | Cycle | Story | Due |
|----|--------|-------------|----------|-----------|-------|-------|-----|

*(No active debt items. See Resolution History for retired entries.)*

### Source Types

| Source | Detection Agent | Description |
|--------|----------------|-------------|
| Phase 5 deferred | adversary | Finding deferred as "fix later" from adversarial review |
| Phase 6 deferred | formal-verifier | Finding deferred from formal hardening |
| Spec drift | spec-steward | BC postcondition not enforced in code |
| Dependency | security-reviewer | Major version bump available or vulnerability |
| DTU fidelity | dtu-validator | Real API changed, clone is stale |
| Pattern inconsistency | code-reviewer | Legacy pattern in older code |
| Holdout decay | holdout-evaluator | Scenario tests removed/changed feature |
| Maintenance sweep | consistency-validator | Anti-pattern or code smell detected |

## Resolution History

| ID | Resolved In | Story | Resolution |
|----|------------|-------|------------|
| TD-001 | pre-phase-1 remediation burst (2026-05-12) | ADR-0002 | Accepted via ADR-0002 with explicit re-eval trigger; not a deferred item — production-grade decision to commit to nucleo 0.5 for Phase 1. AI-introduced registration violated canonical principle §Rule 3 (no story anchor, no human direction). |

## TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE

**Status:** ACCEPTED (human-directed deferral per CLAUDE.md Principle 3)
**Severity:** MEDIUM (audit-trail integrity; not behavioral)
**Scope:** Phase 1 spec set; specifically PRD ↔ VP-INDEX bidirectional pin coherence
**Date:** 2026-05-19

### Description

The PRD frontmatter `traces_to:` field consumes VP-INDEX version as a NORMATIVE pin. When VP-INDEX bumps (as cascade-tail to PRD bumps via SE-22 v2 forward consumer-ledger), PRD's pin to VP-INDEX becomes stale. The mechanical fix is single-line; the fix itself triggers another VP cascade; iteration is structurally unbounded with the current prose-rule discipline architecture.

### Empirical Evidence

- R121 returned 1 HIGH finding (F-R121-1): PRD `traces_to:` VP-INDEX v1.14, canonical v1.15.
- R122 returned 1 HIGH finding (F-R122-1): same class; PRD `traces_to:` VP-INDEX v1.15, canonical v1.16.
- Same defect class, two consecutive rounds. Asymptote-at-1 validated at n=2.

### Required for Resolution

This deferral is contingent on the following CONCRETE FUTURE DEPENDENCY:

**spec-kit-mcp upstream implementation in vsdd-factory** (proposal SK-MCP-001, draft 2026-05-17):
- Library: `vsdd-spec-kit-core` (typed graph + invariants)
- MCP server: `spec-kit-mcp` (agent-callable mutation tools)
- Dispatcher pre-commit hook: `vsdd-spec-kit-validator.wasm`
- Invariant INV-005 (transitive closure with fixed-point iteration) eliminates this class of defect

Per spec-kit-mcp.md §1.3 ("Why Prose Rules Cannot Converge") and §8.3 ("Quality Risk Reduction"):
> "F-LP78/79/80/81 all reached Phase 3 implementation before being caught. Under spec-kit: blocked by INV-005 (version pin staleness via transitive closure)."

### Future Attachment

**Phase-3 pre-implementation mechanical sweep** when spec-kit-mcp ships upstream (vsdd-factory rc.19+):
1. Run `spec_kit_verify_invariants(scope="all")` against monocle `.factory/`
2. Run `spec_kit_bump_artifact()` cascade-tail closure for any remaining residual pin staleness
3. Migrate POL-29 / SE-22 v1/v2 prose rules to schema-enforced invariants

**Story attachment:** Pre-Phase-3 dependency — story to be created during Phase 2 decomposition: "Story S-PHASE-3-PREP-spec-kit-mcp-integration" (working title).

### D-047 Strict Exemption Scope

This residual catalog entry applies SPECIFICALLY to:
- Reverse-cascade class only (forward cascades fully closed per SE-22 v2)
- PRD ↔ VP-INDEX direction only (no other artifact pair currently demonstrates this class at named occurrence count)
- Phase 1 ONLY; Phase 2 + Phase 3 + onwards run under standard D-047 strict

All other Phase 1 disciplines remain enforced:
- 39 codified disciplines including SE-22, SE-23 are NORMATIVE
- All substantive content (BCs, NFRs, ECs, ADRs) PASS at R122
- Forward consumer-ledger cascades closed per SE-22 v2
- SE-16d cross-chain monotonicity PASS across all R15-R20 commits
- SE-17a/c-d/e/f/g evidence disciplines applied throughout

### Acceptance Conditions

- Human direction: explicit user authorization 2026-05-19 (strategic decision Q response).
- Concrete future dependency: spec-kit-mcp upstream rc.19+ (in scope, on roadmap).
- Story attachment: Phase 2 to create explicit story for Phase-3-prep integration.

**ACCEPTANCE: Phase 1 gate PASS WITH DOCUMENTED RESIDUAL.**

## TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT

**Status:** ACCEPTED (human-directed deferral per CLAUDE.md Principle 3)
**Severity:** LOW (audit-trail integrity / annotation accuracy; not behavioral)
**Scope:** Phase 2 story corpus propagation discipline (consumer-summary AC-range drift; sibling-sweep §Trace completeness; consumer-ledger version-pin cascade)
**Date:** 2026-05-19

### Description

Phase 2 story corpus exhibits asymptotic propagation-discipline residuals. Each fix iteration triggers another sibling-sweep opportunity in a different artifact; iteration is structurally unbounded with the current prose-rule discipline architecture (SE-22, SE-25). 6 LOW findings remain across adversarial r12 + consistency r12, all of the same defect class family.

This pattern matches the Phase 1 TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE class. The underlying cause is identical: prose-rule disciplines (SE-22, SE-23, SE-25, SE-26) cannot mechanically enforce sibling-sweep completeness at write time. Each sweep finds a different sibling in a different artifact that was not covered by the previous sweep.

### Empirical Evidence

- Adversarial trajectory: 26→17→13→6→9→7→7→3→2→3→1→1 across 12 adversary rounds (r01..r12). 96% finding reduction.
- Substantive content (BCs, VPs, ACs, dep-graph, wave-schedule, coverage matrices, bidirectional DAG symmetry) converged since r05.
- Final 6 rounds (r07-r12) surfaced 1-3 LOW propagation-discipline findings each — same defect class in different siblings each time.
- Asymptote-at-1 confirmed at adversarial r11 and r12 (1 finding each, same class).
- Phase 1 TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE pattern validated at n=12 rounds for Phase 2.

### Residual Finding Catalog (6 findings — all LOW)

| ID | Source | Description |
|----|--------|-------------|
| F-PHASE2-R12-01 | adversary r12 | STORY-INDEX BC Coverage Table AC-range column drifts from dep-graph BC Clause Coverage Matrix in 9 of 22 rows (cosmetic annotation drift; Full Coverage? column reads YES correctly; recurrence of GAP-R05-1 defect class) |
| GAP-PHASE2-R12-1 | consistency r12 | All 17 story files missing `level: L4` frontmatter field (systematic template omission since r01) |
| GAP-PHASE2-R12-2 | consistency r12 | holdout-scenarios.md Wave 3 section: HS-W3-006 precedes HS-W3-001..005 (non-monotonic ordering) |
| GAP-PHASE2-R12-3 | consistency r12 | BC-2.01.004 (Graceful Shutdown) has no dedicated holdout scenario; S-005 absent from Wave Coverage Summary |
| GAP-PHASE2-R12-4 | consistency r12 | BC-2.02.006/007/008 (HookEnvelope) have no holdout scenario; S-013 absent from Wave Coverage Summary |
| GAP-PHASE2-R12-5 | consistency r12 | S-014 Token Budget table omits BC-2.02.003.md entry (non-normative advisory-only) |

### Required for Resolution

This deferral is contingent on the following CONCRETE FUTURE DEPENDENCY (same as Phase 1 residual):

**spec-kit-mcp upstream implementation in vsdd-factory** (proposal SK-MCP-001, draft 2026-05-17):
- Library: `vsdd-spec-kit-core` (typed graph + invariants)
- MCP server: `spec-kit-mcp` (agent-callable mutation tools)
- Dispatcher pre-commit hook: `vsdd-spec-kit-validator.wasm`
- INV-005 (transitive closure with fixed-point iteration) eliminates reverse-cascade class
- NEW INV needed for AC-range column sibling-sweep (sibling-sweep enforcement at story write time)

Per spec-kit-mcp.md §1.3 ("Why Prose Rules Cannot Converge") and §8.3 ("Quality Risk Reduction"), schema-enforced invariants eliminate this defect class mechanically.

### Future Attachment

**SAME story as Phase 1 residual — S-PHASE-3-PREP-spec-kit-mcp-integration** (exists at `/Users/jmagady/Dev/monocle/.factory/stories/S-PHASE-3-PREP-spec-kit-mcp-integration.md`; Wave 0; blocks on vsdd-factory spec-kit-mcp rc.19+).

Add Phase 2 scope to the existing story's task list:
1. Sweep STORY-INDEX BC Coverage Table AC-range column against dep-graph BC Clause Coverage Matrix (correct 9 drifted rows)
2. Add `level: L4` frontmatter field to all 17 story files
3. Reorder holdout-scenarios.md Wave 3 section monotonically (HS-W3-001..006)
4. Add holdout scenario for BC-2.01.004 (Graceful Shutdown) + add S-005 to Wave Coverage Summary
5. Add holdout scenarios for BC-2.02.006/007/008 (HookEnvelope) + add S-013 to Wave Coverage Summary
6. Add BC-2.02.003.md entry to S-014 Token Budget table

### D-047 Strict Exemption Scope

This residual catalog entry applies SPECIFICALLY to:
- Propagation-discipline class only (consumer-summary drift / §Trace sibling-sweep / version-pin cascade)
- Phase 2 story corpus only; Phase 3+ runs under standard D-047 strict
- Behavioral content, BC-clause-anchor accuracy, dependency graph acyclicity, bidirectional symmetry, coverage completeness — ALL fully enforced

All 39 disciplines remain NORMATIVE for Phase 3. The exemption is scoped specifically to consumer-summary drift / §Trace sibling-sweep / version-pin cascade where the prose-rule discipline architecture cannot mechanically enforce sibling-sweep completeness.

### SE Candidates Identified (not yet codified — pending spec-kit-mcp upstream)

- **SE-24 CANDIDATE**: §Trace ascending-monotonic discipline + structural enforcement hook recommendation. 1st occurrence identified during Phase 2 r08 (§Trace reorder burst 81b09be). Held per D-114 (Goodhart's law; structural prevention via spec-kit-mcp preferred).
- **SE-25 CANDIDATE**: Bidirectional DAG symmetry sibling-sweep. Identified during Phase 2 r09/r10 (Decision 10/11). Held per D-114. Resolution: spec-kit-mcp schema-level graph invariant.
- **SE-26 CANDIDATE**: STORY-INDEX BC Coverage Table AC-range column sibling-sweep. Identified during Phase 2 r12 (F-PHASE2-R12-01 recurrence of GAP-R05-1 class). Held per D-114. Resolution: spec-kit-mcp AC-range column invariant.

### Acceptance Conditions

- Human direction: Orchestrator Decision 12 (2026-05-19T18:00:00Z) per D-155 precedent model. Explicit human ratification not required for same-class residual as Phase 1 (pattern established).
- Concrete future dependency: spec-kit-mcp upstream rc.19+ (in scope, on roadmap).
- Story attachment: S-PHASE-3-PREP-spec-kit-mcp-integration (Phase 2 scope extension added above).

**ACCEPTANCE: Phase 2 gate PASS WITH DOCUMENTED RESIDUAL per D-157.**

## Tech Debt as Feature Mode Cycles

When P0 items accumulate, they become a Feature Mode cycle (Path 3) with
cycle type "refactor":

```
orchestrator: "Tech debt P0 items need attention"
  -> Path 3 (Feature Mode) with cycle type "refactor"
  -> cycles/vX.Y.Z-refactor-[name]/
  -> Same VSDD rigor: specs updated, tests updated, adversarial review
  -> Release: PATCH (no new features) or MINOR (if public behavior changes)
```
