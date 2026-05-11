---
document_type: pipeline-state
level: ops
version: "2.0"
status: active
producer: state-manager
timestamp: 2026-05-11T23:45:00Z
phase: 0
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: monocle
mode: greenfield
current_step: pre-phase-0-reference-ingest
current_cycle: cycle-001
dtu_required: false
awaiting: additional reference ingest: codemachine-cli + vsdd-factory + final B.5/B.6/synthesis update for any-context + nikiforovall
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
| **Last Updated** | 2026-05-11T23:45:00Z |
| **Current Phase** | pre-phase-0 (additional reference ingest in progress) |
| **Current Step** | Phase B deepening DONE — cloning codemachine-cli + vsdd-factory PENDING |

Rust TUI for managing AI coding harness sessions across multiple engines (Claude Code, CodeMachine-CLI, others) with workflow awareness for sessions operating against factory-pattern projects (vsdd-factory and other dispatchers). Fuses three planes:
- Runtime plane — multi-harness session manager (live preview, permission popups, hook-driven status, SSH remote, profiles) — inspired by any-context/lazyclaude.
- Static plane — Claude Code customization explorer (slash commands, agents, skills, hooks, MCP, plugins, settings, memory) — inspired by NikiforovAll/lazyclaude.
- Workflow plane — factory-awareness: detect projects using a recognized factory dispatcher (vsdd-factory et al.), parse workflow files (lobster today, possibly other formats), surface current phase / step / pending gate / recent hook activity. Monocle observes; it does NOT execute workflows.

Future: federated multi-host roster, OTel cost/token panel, trigger-trace from popup to defining customization.

## Phase Progress

| Phase | Status | Started | Completed | Gate | Finding Progression |
|-------|--------|---------|-----------|------|---------------------|
| -1: Reference Ingest (any-context-lazyclaude) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer | 23 artifacts; 803-line synthesis; ~470 BCs; 4P0+10P1+15P2+10P3 backlog |
| -1: Reference Ingest (nikiforovall-lazyclaude) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer | 13 artifacts; 12 BCs; 12 holdout seeds; 4P0+5P1 risks; parsers 2r |
| -1: Phase B Deepening (any-context + nikiforovall) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer (x8) | 24 new files; server/mcp/plugin/pmw-full (any-context); services/mixins/keybindings/models (nikiforovall) |
| -1: Reference Ingest (codemachine-cli + vsdd-factory) | PENDING | | | codebase-analyzer | scope: codemachine-cli full; vsdd-factory scoped (workflows+hooks+factory pattern) |
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
| Brownfield-ingest nikiforovall-lazyclaude (13 artifacts, Phase A+B+C) | codebase-analyzer | DONE | semport/nikiforovall-lazyclaude/ (223-line synthesis, 12 BCs) |
| Commit dual ingest to factory-artifacts | state-manager | DONE | 36 semport files staged + STATE.md updated @ 9f00beb |
| Phase B deepening — 8 parallel agents (any-context: server/mcp/plugin/pmw-full; nikiforovall: services/mixins/app-keybindings/models) | codebase-analyzer (x8) | DONE | 24 new semport files; server 3r, mcp 3r, plugin 3r, pmw 4r, services 3r, mixins 2r, keybindings 3r, models 4r |
| Commit Phase B deepening + scope expansion to factory-artifacts | state-manager | DONE | 24 semport files + STATE.md updated |
| Clone codemachine-cli + vsdd-factory + remaining B.5/B.6/synthesis | devops-engineer + codebase-analyzer | PENDING | — |

## Reference Repos (Phase -1 inputs)

| Name | URL | Branch | HEAD | Local source path | Analysis output path | Ingest status |
|------|-----|--------|------|-------------------|----------------------|---------------|
| any-context-lazyclaude | https://github.com/any-context/lazyclaude | stg | 4516c004 | /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude | /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude | DONE — synthesis: semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis.md |
| nikiforovall-lazyclaude | https://github.com/NikiforovAll/lazyclaude | main | ebc1f8f3 | /Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude | /Users/jmagady/Dev/monocle/.factory/semport/nikiforovall-lazyclaude | DONE — synthesis: semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis.md |
| codemachine-cli | https://github.com/moazbuilds/CodeMachine-CLI | main | (clone pending) | /Users/jmagady/Dev/monocle/.reference/codemachine-cli | /Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli | PENDING (clone+ingest) |
| vsdd-factory | https://github.com/drbothen/vsdd-factory | develop | (clone pending) | /Users/jmagady/Dev/monocle/.reference/vsdd-factory | /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory | PENDING (clone+SCOPED-ingest) |

**Scope filter:** The PM/Worker multi-agent subsystem of any-context/lazyclaude is OUT OF SCOPE for monocle. Ingest documents it; disposition pass sorts it to "Leave behind".

**Scope expansion (2026-05-11):** monocle is now multi-harness + workflow-aware. CodeMachine-CLI joins as the second harness reference; future harness adapters will follow the same profile pattern. vsdd-factory joins as the workflow-awareness reference, ingested with SCOPE LIMITS — only workflows/*.lobster + hooks/ + factory pattern + STATE.md template are in-scope; the 38 agent prompts and most skills are explicitly out of ingest scope (monocle doesn't need to port them, only recognize they exist). The "factory awareness" abstraction must be factory-AGNOSTIC: presence of `.factory/STATE.md` + a discriminator (factory plugin manifest) determines workflow surface, with vsdd-factory as the first concrete adapter.

## Decisions Log

| ID | Decision | Rationale | Phase | Date | Made By |
|----|----------|-----------|-------|------|---------|
| D-001 | Greenfield-with-reference-ingest mode | New Rust codebase; two external lazyclaude repos analyzed first to inform brief | pre-phase-0 | 2026-05-11 | human |
| D-002 | PM/Worker subsystem out of scope | Monocle targets session management + customization exploration, not multi-agent orchestration | pre-phase-0 | 2026-05-11 | human |
| D-003 | Scope expansion: multi-harness + workflow-aware | User confirmed monocle manages sessions for multiple AI coding harnesses (Claude Code today; CodeMachine-CLI second); each harness gets profile in ~/.monocle/config.json | pre-phase-0 | 2026-05-11 | human |
| D-004 | factory-awareness via discriminator pattern | Monocle OBSERVES factory-pattern projects (.factory/STATE.md + factory plugin manifest) but does NOT execute workflows; vsdd-factory is first concrete adapter | pre-phase-0 | 2026-05-11 | human |
| D-005 | vsdd-factory ingest scoped | Only workflows/*.lobster + hooks/ + factory pattern + STATE.md template in scope; 38 agent prompts + most skills explicitly excluded | pre-phase-0 | 2026-05-11 | human |

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
| **Position** | pre-phase-0; Phase B deepening DONE (24 new files); scope expanded to multi-harness + workflow-aware; codemachine-cli + vsdd-factory clone + ingest PENDING |
| **Next** | Clone codemachine-cli (main) + vsdd-factory (develop, scoped) into .reference/; ingest both; run fresh B.5/B.6/synthesis updates for any-context + nikiforovall; final commit; then multi-repo synthesis preparation |
| **Convergence counter** | n/a (pre-spec) |

## Notes

- Secrets: `.mcp.json` (Perplexity + Tavily keys) is gitignored. Never commit. Use env-var injection.
- `.claude/settings.json` committed (no secrets). `.claude/settings.local.json` gitignored.
- Prior factory-attempt logs archived to `/tmp/monocle-prior-factory-logs/`. Safe to discard.

## Burst Log (summary — details in cycles/cycle-001/burst-log.md)

| Burst | Date | Agent | Outcome |
|-------|------|-------|---------|
| Init: factory-artifacts branch + .factory/ worktree + STATE.md + cycle-001 | 2026-05-11 | state-manager | factory-artifacts @ 3b8c3b5 |
| Ingest both reference repos (36 semport artifacts) + STATE.md update | 2026-05-11 | codebase-analyzer + state-manager | any-context 23 files (62 kB synthesis), nikiforovall 13 files (17 kB synthesis); all Iron Law compliant |
| Phase B deepening — 8 parallel agents (any-context: server/mcp/plugin/pmw-full; nikiforovall: services/mixins/app-keybindings/models) | 2026-05-11 | codebase-analyzer (x8) | 24 new semport files; scope expansion recorded |

## Pending Phase Log

| Timestamp | Phase | Event | Agent | Status |
|-----------|-------|-------|-------|--------|
| 2026-05-11T23:45:00Z | pre-phase-0 | Phase B deepening — 8 parallel agents (any-context: server/mcp/plugin/pmw-full; nikiforovall: services/mixins/app-keybindings/models) | codebase-analyzer (x8) | DONE |
| — | pre-phase-0 | Clone codemachine-cli + vsdd-factory into .reference/ | devops-engineer | PENDING |
| — | pre-phase-0 | Brownfield-ingest codemachine-cli (full) | codebase-analyzer | PENDING |
| — | pre-phase-0 | Brownfield-ingest vsdd-factory (scoped: workflows + hooks + factory pattern) | codebase-analyzer | PENDING |
| — | pre-phase-0 | Fresh B.5 audit any-context + nikiforovall (independent of original B.5) | codebase-analyzer | PENDING |
| — | pre-phase-0 | Fresh B.6 extraction validation any-context + nikiforovall | codebase-analyzer | PENDING |
| — | pre-phase-0 | Phase C synthesis updates any-context + nikiforovall (absorb new deepening) | codebase-analyzer | PENDING |
| — | pre-phase-0 | Final commit + multi-repo synthesis preparation | state-manager | PENDING |

## Historical Content

<!-- This section is populated by /vsdd-factory:compact-state when extracting historical content. -->

| Content | Location |
|---------|----------|
| Burst history | `cycles/cycle-001/burst-log.md` |
| Convergence trajectory | `cycles/cycle-001/convergence-trajectory.md` |
| Session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Lessons learned | `cycles/cycle-001/lessons.md` |
| Resolved blockers | `cycles/cycle-001/blocking-issues-resolved.md` |
