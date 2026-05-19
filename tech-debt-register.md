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
**Scope:** Phase 2 story corpus propagation discipline (consumer-summary AC-range drift; sibling-sweep §Trace completeness; consumer-ledger version-pin cascade; cross-BC AC anchor disambiguation)
**Date:** 2026-05-19 (initial); 2026-05-19 (updated with r13 empirical confirmation per D-159)

### Description

Phase 2 story corpus exhibits asymptotic propagation-discipline residuals. Each fix iteration triggers another sibling-sweep opportunity in a different artifact; iteration is structurally unbounded with the current prose-rule discipline architecture (SE-22, SE-25). The underlying cause is identical to Phase 1 TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE: prose-rule disciplines (SE-22, SE-23, SE-25, SE-26) cannot mechanically enforce sibling-sweep completeness at write time.

The specific residuals at asymptote involve cross-BC AC anchor relationships in the STORY-INDEX BC Coverage Table. The summary table cannot disambiguate which story's ACs cross-anchor to which BC clause without either (a) per-AC attribution in every cell (mechanical cost too high to enforce by prose) or (b) schema enforcement via spec-kit-mcp. This is the root structural cause of F-R13-01 and GAP-R13-1.

### Empirical Evidence

**Initial convergence (r01..r12):**
- Adversarial trajectory: 26→17→13→6→9→7→7→3→2→3→1→1 across 12 adversary rounds. 96% finding reduction.
- Substantive content (BCs, VPs, ACs, dep-graph, wave-schedule, coverage matrices, bidirectional DAG symmetry) converged since r05.
- Final 6 rounds (r07-r12) surfaced 1-3 LOW propagation-discipline findings each — same defect class in different siblings each time.
- Asymptote-at-1 confirmed at adversarial r11 and r12 (1 finding each, same class).

**Fix-all burst at r12 (new empirical evidence per D-159):**
- Adversarial r12: 1 LOW finding (F-PHASE2-R12-01)
- User explicit override: "fix everything" directive (2026-05-19)
- Story-writer burst `abe958e`: closed all 6 r12 residual findings (5 consistency + 1 adversary) cleanly across 21 files
- Adversarial r13 (post-fix, fresh context): 1 NEW LOW finding (F-PHASE2-R13-01 — STORY-INDEX BC-2.01.007 row over-includes AC-005 which cross-anchors BC-2.01.004 EC-049 per dep-graph line 250)
- Consistency r13 (post-fix, fresh context): 1 NEW LOW finding (GAP-PHASE2-R13-1 — STORY-INDEX BC-2.01.002 row missing S-009 attribution for cross-anchor: S-009 AC-010b cross-anchors BC-2.01.002 PC-1 sub-bullet hook_endpoints)
- TOTAL: 14 propagation-discipline findings across 13 adversary rounds. The "fix everything" attempt produced exactly the asymptote pattern predicted: every fix surfaces a new sibling.

**CONCLUSION: Asymptote empirically confirmed at n=13 rounds × ~1 finding/round near asymptote.**

Note on F-R13-01 + GAP-R13-1: Both involve cross-BC AC anchors (S-008 AC-005 → BC-2.01.004 EC-049; S-009 AC-010b → BC-2.01.002 PC-1 sub-bullet). The STORY-INDEX BC Coverage Table summary cannot disambiguate cross-anchors without either per-AC attribution (too expensive to enforce by prose) or schema enforcement via spec-kit-mcp.

### Residual Finding Catalog (8 findings — 6 CLOSED + 2 ACTIVE)

| ID | Status | Source | Description |
|----|--------|--------|-------------|
| F-PHASE2-R12-01 | CLOSED (abe958e) | adversary r12 | STORY-INDEX BC Coverage Table AC-range column drifts from dep-graph BC Clause Coverage Matrix in 9 of 22 rows |
| GAP-PHASE2-R12-1 | CLOSED (abe958e) | consistency r12 | All 17 story files missing `level: L4` frontmatter field (systematic template omission since r01) |
| GAP-PHASE2-R12-2 | CLOSED (abe958e) | consistency r12 | holdout-scenarios.md Wave 3 section: HS-W3-006 non-monotonic ordering |
| GAP-PHASE2-R12-3 | CLOSED (abe958e) | consistency r12 | BC-2.01.004 (Graceful Shutdown) has no dedicated holdout scenario; S-005 absent from Wave Coverage Summary |
| GAP-PHASE2-R12-4 | CLOSED (abe958e) | consistency r12 | BC-2.02.006/007/008 (HookEnvelope) have no holdout scenario; S-013 absent from Wave Coverage Summary |
| GAP-PHASE2-R12-5 | CLOSED (abe958e) | consistency r12 | S-014 Token Budget table omits BC-2.02.003.md entry (non-normative advisory-only) |
| F-PHASE2-R13-01 | ACTIVE residual | adversary r13 | STORY-INDEX BC-2.01.007 row over-includes AC-005 (cross-BC anchor: S-008 AC-005 → BC-2.01.004 EC-049 per dep-graph line 250; summary table cannot unambiguously attribute cross-BC ACs) |
| GAP-PHASE2-R13-1 | ACTIVE residual | consistency r13 | STORY-INDEX BC-2.01.002 row missing S-009 attribution (S-009 AC-010b cross-anchors BC-2.01.002 PC-1 sub-bullet hook_endpoints; same cross-BC attribution class as F-R13-01) |

### Required for Resolution

This deferral is contingent on the following CONCRETE FUTURE DEPENDENCY (same as Phase 1 residual):

**spec-kit-mcp upstream implementation in vsdd-factory** (proposal SK-MCP-001, draft 2026-05-17):
- Library: `vsdd-spec-kit-core` (typed graph + invariants)
- MCP server: `spec-kit-mcp` (agent-callable mutation tools)
- Dispatcher pre-commit hook: `vsdd-spec-kit-validator.wasm`
- INV-005 (transitive closure with fixed-point iteration) eliminates reverse-cascade class
- NEW INV needed for AC-range column sibling-sweep (sibling-sweep enforcement at story write time)

Per spec-kit-mcp.md §1.3 ("Why Prose Rules Cannot Converge") and §8.3 ("Quality Risk Reduction"), schema-enforced invariants eliminate this defect class mechanically. F-R13-01 + GAP-R13-1 (the two ACTIVE residuals) require a NEW INV specifically for cross-BC AC anchor attribution in STORY-INDEX BC Coverage Table rows — this is a different invariant than the AC-range column sibling-sweep (SE-26 / INV for AC-range) because it concerns which story's ACs are attributed to which BC, not just whether the AC range column is in sync.

### Future Attachment

**SAME story as Phase 1 residual — S-PHASE-3-PREP-spec-kit-mcp-integration** (exists at `/Users/jmagady/Dev/monocle/.factory/stories/S-PHASE-3-PREP-spec-kit-mcp-integration.md`; Wave 0; blocks on vsdd-factory spec-kit-mcp rc.19+).

Phase 2 scope for the existing story's task list (items 1-6 CLOSED via abe958e; items 7-8 ACTIVE residual per D-159):
1. (CLOSED abe958e) Sweep STORY-INDEX BC Coverage Table AC-range column against dep-graph BC Clause Coverage Matrix (corrected 9 drifted rows)
2. (CLOSED abe958e) Add `level: L4` frontmatter field to all 17 story files
3. (CLOSED abe958e) Reorder holdout-scenarios.md Wave 3 section monotonically (HS-W3-001..006)
4. (CLOSED abe958e) Add holdout scenario for BC-2.01.004 (Graceful Shutdown) + add S-005 to Wave Coverage Summary
5. (CLOSED abe958e) Add holdout scenarios for BC-2.02.006/007/008 (HookEnvelope) + add S-013 to Wave Coverage Summary
6. (CLOSED abe958e) Add BC-2.02.003.md entry to S-014 Token Budget table
7. (ACTIVE F-R13-01) Fix STORY-INDEX BC-2.01.007 row: remove over-attribution of AC-005 (S-008 AC-005 cross-anchors BC-2.01.004 EC-049, not BC-2.01.007; dep-graph line 250 is the source of truth for cross-BC anchor direction)
8. (ACTIVE GAP-R13-1) Fix STORY-INDEX BC-2.01.002 row: add S-009 attribution with note that AC-010b cross-anchors BC-2.01.002 PC-1 sub-bullet hook_endpoints

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

- Human direction (FIRST): Orchestrator Decision 12 (2026-05-19T18:00:00Z) per D-155 precedent model — D-157 Phase 2 GATE PASS WITH RESIDUAL declared.
- Human direction (SECOND): Explicit user acceptance of asymptote conclusion (2026-05-19) after "fix everything" directive produced new r13 findings of same class — D-159 Phase 2 GATE PASS WITH RESIDUAL FINALIZED. User explicitly accepted that structural cause requires upstream spec-kit-mcp rc.19+ and that no further fix iterations should be attempted.
- Concrete future dependency: spec-kit-mcp upstream rc.19+ (in scope, on roadmap).
- Story attachment: S-PHASE-3-PREP-spec-kit-mcp-integration (Phase 2 scope extended: tasks 1-6 CLOSED via abe958e; tasks 7-8 ACTIVE residual F-R13-01 + GAP-R13-1).
- Empirical proof of structural cause: r12 fix-all closed 6 findings; r13 re-derivation surfaced 2 new findings of same class — proves prose-discipline architecture cannot achieve strict convergence on cross-BC anchor relationships in STORY-INDEX summary table.

**ACCEPTANCE: Phase 2 gate PASS WITH DOCUMENTED RESIDUAL FINALIZED per D-159.**

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
