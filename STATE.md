---
document_type: pipeline-state
level: ops
version: "2.0"
status: active
producer: state-manager
timestamp: 2026-05-12T00:00:00Z
phase: 0
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: monocle
mode: greenfield
current_step: phase-1-spec-crystallization-blocked-on-brief-redline
current_cycle: cycle-001
dtu_required: false
awaiting: human red-line of /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
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
| **Last Updated** | 2026-05-12T00:00:00Z |
| **Current Phase** | phase-1-ready-awaiting-brief-redline |
| **Current Step** | phase-1-spec-crystallization-blocked-on-brief-redline |
| **Canonical vision** | /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md |
| **Product brief** | /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md |

Rust TUI for managing AI coding harness sessions across multiple engines (Claude Code, CodeMachine, others) with workflow awareness for sessions operating against factory-pattern projects (vsdd-factory and other dispatchers compatible with the document_type: pipeline-state discriminator). Fuses five genetic planes drawing on 8 reference repos:
- Runtime plane (any-context, zellij) — multi-harness session manager + Rust IPC + WASM plugin SDK
- Static plane (NikiforovAll) — Claude Code customization explorer
- Workflow plane (vsdd-factory) — factory-awareness (observe-only)
- Harness plane (codemachine, claude-squad, claude-code-router) — EngineModule abstraction + worktree-isolation + integrate-external routing
- TUI-philosophy plane (lazygit) — the canonical lazy* signature pattern: context-aware bindings, telescope help, modal cascade

Future: federated multi-host roster, OTel cost/token panel, trigger-trace from popup to defining customization.

## Phase Progress

| Phase | Status | Started | Completed | Gate | Finding Progression |
|-------|--------|---------|-----------|------|---------------------|
| -1: Reference Ingest (any-context-lazyclaude) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer | 23 artifacts; 867-line v2 synthesis; ~644 BCs; 30 Phase B rounds |
| -1: Reference Ingest (nikiforovall-lazyclaude) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer | 14 artifacts; 1072-line v2 synthesis; 12 BCs; 16 Phase B rounds |
| -1: Phase B deepening + B.5 v2 audits + gap-fill (any-context + nikiforovall) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer (x8+) | broker/hooks/tmuxadapter gap-fill; B.5 v2 fresh audits; v2 syntheses |
| -1: Reference Ingest (codemachine-cli) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer | 14 artifacts; 678-line synthesis; EngineModule contract; 7-state FSM |
| -1: Reference Ingest (vsdd-factory, SCOPED) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer | 14 artifacts; 761-line synthesis; lobster YAML; dispatcher; factory pattern |
| -1: EXPANSION — 4 additional ingests (zellij, lazygit, claude-squad, claude-code-router) | DONE | 2026-05-11 | 2026-05-11 | codebase-analyzer (x4) | 57 new files; 5-plane gene corpus complete across 8 repos |
| 0.5: Product Brief | DONE | 2026-05-12 | 2026-05-12 | validate-brief (manual PASS) | 290 lines; 3 personas; 7 success rows; 10 OQs; 3 judgment calls for redline |
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
| Vision synthesis saved — orchestrator vision approved by human; canonical doc written to specs/research/domain-monocle-vision-synthesis.md | state-manager | DONE | D-012 logged; single-commit burst to factory-artifacts @ 2737bfd |
| Product brief drafted — product-owner direct-draft from approved vision + 8 Pass 8 syntheses | product-owner | DONE | specs/product-brief.md; 290 lines; 3 personas; 7 success rows; 10 OQs; 3 judgment calls |
| Brief committed to factory-artifacts + STATE.md updated | state-manager | DONE | single-commit burst; input-drift CLEAN (STALE=0) |
| Human red-line of specs/product-brief.md (3 judgment calls) | human | PENDING | — |
| Parallel dispatch: /vsdd-factory:create-architecture + /vsdd-factory:create-prd | architect + product-owner | PENDING | — |

## Reference Repos (Phase -1 inputs)

| Name | URL | Branch | HEAD | Local source path | Analysis output path | Ingest status |
|------|-----|--------|------|-------------------|----------------------|---------------|
| any-context-lazyclaude | https://github.com/any-context/lazyclaude | stg | 4516c004 | /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude | /Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude | DONE — canonical: semport/any-context-lazyclaude/any-context-lazyclaude-pass-8-final-synthesis-v2.md |
| nikiforovall-lazyclaude | https://github.com/NikiforovAll/lazyclaude | main | ebc1f8f3 | /Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude | /Users/jmagady/Dev/monocle/.factory/semport/nikiforovall-lazyclaude | DONE — canonical: semport/nikiforovall-lazyclaude/nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md |
| codemachine-cli | https://github.com/moazbuilds/CodeMachine-CLI | main | 572def6 | /Users/jmagady/Dev/monocle/.reference/codemachine-cli | /Users/jmagady/Dev/monocle/.factory/semport/codemachine-cli | DONE — canonical: semport/codemachine-cli/codemachine-cli-pass-8-final-synthesis.md |
| vsdd-factory | https://github.com/drbothen/vsdd-factory | develop | 99d2431 | /Users/jmagady/Dev/monocle/.reference/vsdd-factory | /Users/jmagady/Dev/monocle/.factory/semport/vsdd-factory | DONE (SCOPED) — canonical: semport/vsdd-factory/vsdd-factory-pass-8-final-synthesis.md |
| zellij | https://github.com/zellij-org/zellij | main | de1e0f75 | /Users/jmagady/Dev/monocle/.reference/zellij | /Users/jmagady/Dev/monocle/.factory/semport/zellij | DONE (SCOPED) — canonical: semport/zellij/zellij-pass-8-final-synthesis.md |
| lazygit | https://github.com/jesseduffield/lazygit | master | c4935036 | /Users/jmagady/Dev/monocle/.reference/lazygit | /Users/jmagady/Dev/monocle/.factory/semport/lazygit | DONE (SCOPED) — canonical: semport/lazygit/lazygit-pass-8-final-synthesis.md |
| claude-squad | https://github.com/smtg-ai/claude-squad | main | a4ab6988 | /Users/jmagady/Dev/monocle/.reference/claude-squad | /Users/jmagady/Dev/monocle/.factory/semport/claude-squad | DONE (FULL) — canonical: semport/claude-squad/claude-squad-pass-8-deep-synthesis.md |
| claude-code-router | https://github.com/musistudio/claude-code-router | main | e270dea5 | /Users/jmagady/Dev/monocle/.reference/claude-code-router | /Users/jmagady/Dev/monocle/.factory/semport/claude-code-router | DONE (FULL, consolidated) — canonical: semport/claude-code-router/claude-code-router-pass-C-final-synthesis.md |

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
| D-006 | Full brownfield-ingest protocol on any-context + nikiforovall; vsdd-factory workflow-awareness scope; codemachine as multi-harness gene source | User directed full protocol (not fast-path) for both original repos; vsdd-factory scoped to factory-pattern recognition only; codemachine-cli provides EngineModule + FSM + MCP-router gene set as second harness reference | pre-phase-0 | 2026-05-11 | human |
| D-007 | Expand reference scope to 8 repos across 5 genetic planes; scope zellij/lazygit (skip multiplexer/git internals), full claude-squad/CCR (small) | Broader gene corpus needed before brief; zellij/lazygit scoped to avoid unrelated internals; small repos justified consolidated convergence | pre-phase-0 | 2026-05-11 | human |
| D-008 | Adopt zellij's crate split pattern (monocle-core/-runtime/-tui/-plugin-sdk/-monocle) | Clean separation of pure core, async runtime, TUI renderer, plugin SDK, and binary crate; prevents god-crate accumulation | pre-phase-0 | 2026-05-11 | state-manager |
| D-009 | Adopt lazygit's context-aware binding pattern with 5-level precedence as monocle's Action enum dispatch model | Lazygit's (view,key,handler) signature + precedence stack (search-prompt > custom > per-context > global > builtin) is the most validated approach in the lazy* ecosystem | pre-phase-0 | 2026-05-11 | state-manager |
| D-010 | Treat claude-code-router as INTEGRATE-EXTERNAL not build-in; no CCR APIs change required | CCR is a mature standalone reverse proxy; monocle integrates by detecting CCR on PATH + writing per-session JSON + setting ANTHROPIC_BASE_URL; no new CCR APIs needed | pre-phase-0 | 2026-05-11 | state-manager |
| D-011 | claude-squad teaches UX (worktree isolation, snapshot-fork concurrency) not orchestration (it has none — human is coordinator) | claude-squad has no PM/Worker; human is the coordinator; treat as TUI prior art + worktree isolation pattern only | pre-phase-0 | 2026-05-11 | state-manager |
| D-012 | Vision synthesis approved by human 2026-05-11; canonical reference at /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md; supersedes any free-form vision statements in pre-phase-0 burst log | Human approved orchestrator vision verbatim ("I agree with this fully"); saved as pre-brief canonical doc for product-owner, architect, and disposition-pass agents | pre-phase-0 | 2026-05-11 | state-manager |
| D-013 | Product brief drafted via direct-draft per human choice; 10 open architectural questions surfaced for Phase 1; 3 judgment calls flagged for red-line: (1) static 7-type rendering Phase 1 criterion vs Phase 2 exit, (2) PostToolUse endpoint v1 in-scope vs omitted per any-context BC-HOOK-007, (3) port 2748 fixed value vs OS-assigned + lock-file | Direct-draft (product-owner from vision + gene corpus) preferred over guided-brief-creation session; architect open questions deferred to OQ-01..OQ-10 in brief | pre-phase-1 | 2026-05-12 | state-manager |

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
| **Date** | 2026-05-12 |
| **Position** | phase-1-ready-awaiting-brief-redline; product brief drafted (290 lines, validate-brief PASS) at /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md; D-013 logged; 3 judgment calls flagged for human red-line; 10 open architect questions (OQ-01..OQ-10) in brief |
| **Next** | Human red-line of product-brief.md; resolve 3 judgment calls; then parallel dispatch /vsdd-factory:create-architecture (architect) + /vsdd-factory:create-prd (product-owner) |
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
| Full-protocol Phase B rounds — broker/hooks/tmuxadapter gap-fill (any-context) + remaining nikiforovall rounds | 2026-05-11 | codebase-analyzer (x8) | broker-r1,r2 + hooks-r1,r2 + tmuxadapter-r1 added; any-context 30 Phase B rounds total |
| B.5 v2 fresh-context audits — any-context + nikiforovall | 2026-05-11 | codebase-analyzer (x2) | any-context: TOPIC-DRIFT-FOUND-AND-RESOLVED; nikiforovall: TOPIC-DRIFT-CLEAN |
| Brownfield-ingest codemachine-cli (full) + vsdd-factory (scoped) | 2026-05-11 | codebase-analyzer (x2) | codemachine-cli: 14 files, 678-line synthesis; vsdd-factory: 14 files, 761-line synthesis |
| Pass 8 v2 syntheses — any-context (867 lines, ~644 BCs) + nikiforovall (1072 lines, 21 BCs) | 2026-05-11 | codebase-analyzer (x2) | v2 files created; v1 preserved unchanged |
| Atomic commit: Phase -1 COMPLETE — all 4 repos ingested; STATE.md transitioned to human-checkpoint gate | 2026-05-11 | state-manager | factory-artifacts HEAD = 234b6bd |
| Clone 4 new refs (zellij/lazygit/claude-squad/CCR) | 2026-05-11 | devops | refs at de1e0f75 / c4935036 / a4ab6988 / e270dea5 |
| 4 parallel ingests — 57 new semport files across 4 subtrees | 2026-05-11 | codebase-analyzer (x4) | zellij: 24 files; lazygit: 17 files; claude-squad: 15 files; CCR: 1 file |
| Atomic commit: EXPANSION — 8-repo / 5-plane corpus committed; STATE.md updated | 2026-05-11 | state-manager | input-drift CLEAN; this commit |
| Vision synthesis saved — orchestrator canonical vision doc + STATE.md update (D-012, awaiting update, burst row, next-step) | 2026-05-11 | state-manager | specs/research/domain-monocle-vision-synthesis.md created; single-commit burst to factory-artifacts |
| Brief commit — product brief drafted by product-owner; STATE.md updated (current_phase, phase progress row, current steps, D-013, next-step, project snapshot); input-drift CLEAN (STALE=0) | 2026-05-12 | state-manager | specs/product-brief.md committed; single-commit burst per TD-VSDD-053 |

## Next Step

1. Human red-line of /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
2. Resolve 3 judgment calls (static 7-type Phase 1 vs 2; PostToolUse v1 in/out; port 2748 fixed vs OS-assigned)
3. Optionally resolve some of the 10 open architect questions (OQ-01..OQ-10) before architect dispatch
4. On red-line approval: parallel dispatch of /vsdd-factory:create-architecture (architect) and /vsdd-factory:create-prd (product-owner) per greenfield sequence
5. After PRD + architecture: /vsdd-factory:phase-1d-adversarial-spec-review for 3-clean-pass convergence
6. Then human approval gate → Phase 2 story decomposition

## Historical Content

<!-- This section is populated by /vsdd-factory:compact-state when extracting historical content. -->

| Content | Location |
|---------|----------|
| Burst history | `cycles/cycle-001/burst-log.md` |
| Convergence trajectory | `cycles/cycle-001/convergence-trajectory.md` |
| Session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Lessons learned | `cycles/cycle-001/lessons.md` |
| Resolved blockers | `cycles/cycle-001/blocking-issues-resolved.md` |
