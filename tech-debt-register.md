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
