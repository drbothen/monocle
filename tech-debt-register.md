---
document_type: tech-debt-register
level: ops
status: active
producer: product-owner
version: "1.0"
last_updated: 2026-05-12T00:00:00Z
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
