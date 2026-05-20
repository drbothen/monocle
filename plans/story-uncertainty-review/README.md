---
document_type: plans-readme
artifact: story-uncertainty-review
status: active
producer: vsdd-factory:orchestrator
project: monocle
---

# Story Uncertainty Review — Pre-Phase-3 Quality Gate

## Purpose

The story uncertainty review is a structured pre-implementation quality gate that scans
every story spec in the pipeline corpus for uncertainties that would cause mid-sprint
failures: unpinned version assumptions, outdated API assumptions, incorrect feature claims,
cross-story contract ambiguities, and structural gaps that prevent TDD from starting cleanly.

This artifact directory captures the review output so that findings persist across sessions
and can be dispatched to remediation in Stage 3 without re-running the assessment.

The skill was field-developed for the monocle project as a pre-Phase-3 gate. The upstream
formalization is tracked in vsdd-factory issue #150 (https://github.com/drbothen/vsdd-factory/issues/150).

## Pipeline Position

This review runs AFTER Phase 2 GATE PASS (story decomposition converged) and BEFORE Phase 3
dispatch. It is not a Phase 2 gate artifact — it is a Phase 3 dispatch prerequisite. The
stories themselves are not modified by the review; findings are dispatched to the appropriate
specialist agents in Stage 3 for remediation.

## Cycle Directory Layout

Each pipeline run through the uncertainty review produces a cycle directory:

```
cycle-NNN/
├── master-inventory.md          # Aggregate findings, patterns, dispatch plan
├── S-NNN-assessment.md          # Per-story assessment (one file per story)
├── S-DTU-NNN-assessment.md      # DTU story assessments
└── S-PHASE-N-PREP-assessment.md # Wave-0 / prep story assessments
```

Cycles are numbered sequentially. A new cycle is created if the corpus changes substantially
(e.g., new stories added, major spec revisions) and the review needs to be re-run.

## Stage Protocol

- **Stage 1 (current — cycle-001):** Spec-reviewer scans all stories; produces per-story assessments and master inventory.
- **Stage 2:** Research agent resolves external-dependency uncertainties flagged as NEEDS_RESEARCH.
- **Stage 3:** Orchestrator dispatches remediation to specialist agents (product-owner, architect, story-writer) based on master-inventory dispatch plan.
- **Stage 4:** Spec-reviewer re-scans modified stories; confirms PASS or PASS_WITH_OBSERVATIONS; updates master-inventory with final verdicts.

## Cycle-001 Status

Stage 1 complete as of 2026-05-20. 17 stories scanned. Stages 2-4 pending orchestrator dispatch.
See `cycle-001/master-inventory.md` for aggregate findings and dispatch plan.
