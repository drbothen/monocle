---
document_type: tech-debt-register
producer: product-owner
version: "1.0"
last_updated: 2026-05-12T16:00:00Z
project: monocle
phase: pre-phase-1
---

# Technical Debt Register

## Summary

| Priority | Count | Estimated Points |
|----------|-------|-----------------|
| P0 (next cycle) | 0 | 0 |
| P1 (within 3 cycles) | 1 | 2 |
| P2 (backlog) | 0 | 0 |

## Debt Items

| ID | Source | Description | Priority | Introduced | Cycle | Story | Due |
|----|--------|-------------|----------|-----------|-------|-------|-----|
| TD-001 | Dependency | `nucleo 0.5` upstream dormant since 2024-04-02; helix-editor team's focus shifted. If maintenance becomes a release constraint, evaluate `frizbee 0.9` / `neo_frizbee 0.10` (SIMD alternatives) or `nucleo-picker 0.11` (TUI-focused fork). Functionality intact for Phase 1. | P1 | v1.0 | — | — | Phase 2 re-eval |

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
| TD-NNN | vX.Y.Z | — | — |

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
