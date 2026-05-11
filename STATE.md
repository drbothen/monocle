---
document_type: pipeline-state
level: ops
version: "2.0"
status: active
producer: state-manager
timestamp: 2026-05-11T22:06:53Z
phase: 0
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: monocle
mode: greenfield
current_step: pre-phase-0-reference-ingest
current_cycle: cycle-001
dtu_required: false
---

<!--
  STATE.md SIZE BUDGET: Keep this file under 200 lines.
  A hook warns at 200 and blocks at 500 (unless compacting).

  Historical content belongs in cycle files, NOT here:
  - Burst narratives → cycles/<cycle>/burst-log.md
  - Adversary pass details → cycles/<cycle>/convergence-trajectory.md
  - Old session checkpoints → cycles/<cycle>/session-checkpoints.md
  - Lessons learned → cycles/<cycle>/lessons.md
  - Resolved blockers → cycles/<cycle>/blocking-issues-resolved.md

  Run /vsdd-factory:compact-state if this file grows past 200 lines.
  See state-manager agent "Content Routing Rules" for the full policy.
-->

# Pipeline State: Monocle

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | monocle |
| **Repository** | https://github.com/drbothen/monocle |
| **Mode** | greenfield-with-reference-ingest |
| **Language** | Rust |
| **Target Workspace** | /Users/jmagady/Dev/monocle |
| **Started** | 2026-05-11 |
| **Last Updated** | 2026-05-11 |
| **Current Phase** | pre-phase-0 (reference ingest) |
| **Current Step** | Awaiting brownfield-ingest of reference repos |

## Phase Progress

| Phase | Status | Started | Completed | Gate | Finding Progression |
|-------|--------|---------|-----------|------|---------------------|
| -1: Reference Ingest (any-context-lazyclaude) | not-started | | | | |
| -1: Reference Ingest (nikiforovall-lazyclaude) | not-started | | | | |
| 0: Codebase Ingestion | not-started | | | | |
| 1: Spec Crystallization | not-started | | | | |
| 2: Story Decomposition | not-started | | | | |
| 3: TDD Implementation | not-started | | | | |
| 4: Holdout Evaluation | not-started | | | | |
| 5: Adversarial Refinement | not-started | | | | |
| 6: Formal Hardening | not-started | | | | |
| 7: Convergence | not-started | | | | |

## Current Phase Steps

<!-- Keep last 5 rows only. Archive older rows to cycles/<cycle>/burst-log.md. -->

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| Repo bootstrap (factory-artifacts branch, .factory/ worktree, rerere, gitignore, reference clones) | devops-engineer | DONE | factory-artifacts branch @ 3b8c3b5 |
| STATE.md + cycle-001 directory initialization | state-manager | IN-PROGRESS | /Users/jmagady/Dev/monocle/.factory/STATE.md |

## Reference Repos (Phase -1 inputs)

| Name | Branch | HEAD | Local path | Output path | Status |
|------|--------|------|------------|-------------|--------|
| any-context-lazyclaude | stg | 4516c004 | /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude | /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude | PENDING |
| nikiforovall-lazyclaude | main | ebc1f8f3 | /Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude | /Users/jmagady/Dev/monocle/.factory/semport/nikiforovall-lazyclaude | PENDING |

**Scope filter:** The PM/Worker multi-agent subsystem of any-context/lazyclaude is OUT OF SCOPE for monocle. Ingest documents it; disposition pass sorts it to "Leave behind".

## Decisions Log

| ID | Decision | Rationale | Phase | Date | Made By |
|----|----------|-----------|-------|------|---------|
| D-001 | Greenfield-with-reference-ingest mode | New Rust codebase; two external lazyclaude repos analyzed first to inform brief | pre-phase-0 | 2026-05-11 | human |
| D-002 | PM/Worker subsystem out of scope | Monocle targets session management + customization exploration, not multi-agent orchestration | pre-phase-0 | 2026-05-11 | human |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | no | TUI product requires UX spec |
| DTU Assessment | pending | Deferred until architecture complete |

## Blocking Issues

<!-- Open issues only. Move resolved issues to cycles/<cycle>/blocking-issues-resolved.md. -->

| ID | Issue | Severity | Blocking Phase | Owner | Resolution |
|----|-------|----------|---------------|-------|------------|

## Session Resume Checkpoint

<!-- Keep ONLY the latest checkpoint. Archive prior checkpoints to cycles/<cycle>/session-checkpoints.md. -->

| Field | Value |
|-------|-------|
| **Date** | 2026-05-11 |
| **Position** | pre-phase-0; reference ingest pending; next: dispatch brownfield-ingest for both reference repos |
| **Convergence counter** | n/a (pre-spec) |

## Notes

- Secrets: `.mcp.json` (Perplexity + Tavily keys) is gitignored. Never commit. Use env-var injection.
- `.claude/settings.json` committed (no secrets). `.claude/settings.local.json` gitignored.
- Prior factory-attempt logs archived to `/tmp/monocle-prior-factory-logs/`. Safe to discard.

## Historical Content

<!-- This section is populated by /vsdd-factory:compact-state when extracting historical content. -->

| Content | Location |
|---------|----------|
| Burst history | `cycles/cycle-001/burst-log.md` |
| Convergence trajectory | `cycles/cycle-001/convergence-trajectory.md` |
| Session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Lessons learned | `cycles/cycle-001/lessons.md` |
| Resolved blockers | `cycles/cycle-001/blocking-issues-resolved.md` |
