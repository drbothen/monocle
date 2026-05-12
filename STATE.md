---
document_type: pipeline-state
level: ops
version: "2.0"
status: active
producer: state-manager
timestamp: 2026-05-12T22:00:00Z
phase: 0
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: monocle
mode: greenfield
current_step: oq-research-delivered-awaiting-human-review
current_cycle: cycle-001
dtu_required: false
awaiting: optional re-validate-brief on v1.2 OR proceed to architect+PRD parallel dispatch
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
| **Last Updated** | 2026-05-12T22:00:00Z |
| **Current Phase** | phase-1-ready-awaiting-brief-redline |
| **Current Step** | validate-brief-v1.1-and-human-redline |
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
| 0.5: Product Brief | DONE | 2026-05-12 | 2026-05-12 | validate-brief (manual PASS) | 341 lines; 3 personas; 7 success rows; 11 OQs; 5 judgment calls (JC-1..3, EX-1..2); RUSTSEC notes; 13 version corrections + 11 new pins |
| 0.6: validate-brief on v1.1 | NEEDS_WORK | 2026-05-12 | — | validate-brief skill | NEEDS_WORK: bloat 3.6x recommended; JC-1 scope contradiction unresolved; leakage WARNING (intentional + vision-traceable). Report at .factory/planning/brief-validation.md |
| 0.7: OQ research | DONE | 2026-05-12 | 2026-05-12 | research-agent | OQ-01..OQ-11 researched; 10/11 HIGH confidence; 4 SOQs surfaced; output at .factory/planning/oq-research.md (1666 lines) |
| 0.8: Brief v1.2 + arch stubs | DONE | 2026-05-12 | 2026-05-12 | product-owner + state-manager | brief v1.2 (350 lines); dependencies.md (123 lines); ADR-0001 (86 lines); conventions.md (67 lines); tech-debt-register.md (56 lines) |
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
| Brief v1.1 revision + STATE.md — RUSTSEC notes section, OQ-11, Revision History, heading fix | product-owner + state-manager | DONE | 291→341 lines; D-014, D-015 logged; single-commit burst @ factory-artifacts |
| validate-brief on v1.1 | orchestrator (validate-brief skill) | DONE | NEEDS_WORK — JC-1 contradiction + bloat 3.6x; report @ .factory/planning/brief-validation.md; D-016 logged |
| Human red-line of JC-1..3 + EX-1..2 + bloat-decision (Action A/B/C) | human | DONE | Bloat Option A applied; all 11 OQs + 4 SOQs + 5 JCs resolved |
| OQ-01..OQ-11 research delivered | research-agent + state-manager | DONE | .factory/planning/oq-research.md; 1666 lines; 10/11 HIGH confidence; 4 SOQs; D-017 logged |
| Brief v1.2 + arch stubs landed | product-owner + state-manager | DONE | brief v1.2 (350 lines); 4 architecture stubs; tech-debt-register.md; D-018 logged |

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
| D-014 | Crate version validation via crates.io API + Tavily + Perplexity; 13 corrections + 11 new pins applied to brief v1.1; canonical RUSTSEC notes added; wasmtime/wasmi rationale refreshed (wasmi WASI gap claim dropped as dated) | wasmtime 25→44 (RUSTSEC advisories on pre-44), russh 0.45→0.60 (RUSTSEC-2023-0071), prost 0.13→0.14 (RUSTSEC-2026-0007), thiserror pinned to 2 (major bump mid-2024); new Supply Chain section documents advisory context for architect | pre-phase-1 | 2026-05-12 | state-manager |
| D-015 | rmcp 1.6 confirmed canonical Anthropic Rust MCP SDK via modelcontextprotocol/rust-sdk org + alexhancock@Anthropic owner record | rmcp is the only Rust SDK under the modelcontextprotocol GitHub org with an Anthropic employee as sole owner; no competing canonical exists; pinned to 1.6 in brief | pre-phase-1 | 2026-05-12 | state-manager |
| D-016 | Brief v1.1 validate-brief outcome: NEEDS_WORK (bloat 3.6x recommended; JC-1 scope contradiction unresolved; leakage intentional + vision-traceable). Report at .factory/planning/brief-validation.md | validate-brief skill run by orchestrator; 1 quality blocker (JC-1), 1 bloat decision (Action A/B/C), 1 sequential gate (market intel Task #8 before Phase 1 entry) | pre-phase-1 | 2026-05-12 | state-manager |
| D-017 | OQ-01..OQ-11 researched; 10/11 HIGH confidence recommended defaults; 4 second-order questions (SOQ-1..4) surfaced. Research at .factory/planning/oq-research.md. | Research-agent used WebSearch + WebFetch + Context7 + crates.io API (Perplexity MCP unavailable). Sequencing: OQ-10→04→02→01→11→03→07→08→06→05→09. Cross-OQ themes: single LockFile type, single atomic_write helper, Phase 3/4 trait seams, XDG-aware MonoclePaths. | pre-phase-1 | 2026-05-12 | state-manager |
| D-018 | Brief v1.2 + 4 architecture stubs landed. Bloat Option A applied (qualitative; line-count near-neutral due to OQ resolutions added). All 11 OQs + 4 SOQs + 5 JCs resolved. | Supply Chain + pin manifest moved to dependencies.md; wasmtime choice to ADR-0001; anti-patterns to conventions.md; nucleo debt to tech-debt-register.md. Brief gains Phase 1 Constraints table (15 rows), Phase 2 Exit Criteria, OQ resolution column. 350 lines — slightly above v1.1 (342) due to decision-table additions that are appropriate brief-level content. | pre-phase-1 | 2026-05-12 | state-manager |

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
| **Position** | Brief v1.2 landed (350 lines); 4 architecture stubs created (dependencies.md, ADR-0001, conventions.md, tech-debt-register.md); all 11 OQs + 4 SOQs + 5 JCs resolved; D-018 logged; single-commit burst to factory-artifacts |
| **Next** | Optionally re-run /vsdd-factory:validate-brief on v1.2 (confirm qualitative bloat reduction); OR skip directly to parallel dispatch of /vsdd-factory:create-architecture (architect) + /vsdd-factory:create-prd (product-owner). Market intel assessment (Task #8) still required before Phase 1 entry. |
| **Convergence counter** | n/a (pre-spec) |

## Notes

- Secrets: `.mcp.json` (Perplexity + Tavily keys) is gitignored. Never commit. Use env-var injection.
- `.claude/settings.json` committed (no secrets). `.claude/settings.local.json` gitignored.
- Prior factory-attempt logs archived to `/tmp/monocle-prior-factory-logs/`. Safe to discard.

## Burst Log (summary — details in cycles/cycle-001/burst-log.md)

| Burst | Date | Agent | Outcome |
|-------|------|-------|---------|
| Atomic commit: EXPANSION — 8-repo / 5-plane corpus committed; STATE.md updated | 2026-05-11 | state-manager | input-drift CLEAN |
| Vision synthesis saved — orchestrator canonical vision doc + STATE.md update (D-012, awaiting update, burst row, next-step) | 2026-05-11 | state-manager | specs/research/domain-monocle-vision-synthesis.md created; single-commit burst to factory-artifacts |
| Brief commit — product brief drafted by product-owner; STATE.md updated (current_phase, phase progress row, current steps, D-013, next-step, project snapshot); input-drift CLEAN (STALE=0) | 2026-05-12 | state-manager | specs/product-brief.md committed; single-commit burst per TD-VSDD-053 |
| Version validation + brief v1.1 revision — 13 corrections + 11 new pins; RUSTSEC notes section; OQ-11 (MSRV); Revision History; heading fix; D-014 + D-015 logged; STATE.md updated | 2026-05-12 | state-manager | specs/product-brief.md 291→341 lines; single-commit burst per TD-VSDD-053 |
| validate-brief on v1.1 — NEEDS_WORK; planning/brief-validation.md created; D-016 logged; STATE.md updated (awaiting, phase progress 0.6, current steps, next step) | 2026-05-12 | state-manager | .factory/planning/brief-validation.md; single-commit burst per TD-VSDD-053 |
| OQ research delivered — OQ-01..OQ-11 at .factory/planning/oq-research.md (1666 lines); D-017 logged; STATE.md updated (awaiting, phase progress 0.7, current steps, D-017, checkpoint, next step) | 2026-05-12 | state-manager | planning/oq-research.md; single-commit burst per TD-VSDD-053 |
| Brief v1.2 + 4 arch stubs — dependencies.md (123 lines), ADR-0001 (86 lines), conventions.md (67 lines), tech-debt-register.md (56 lines); D-018 logged; STATE.md updated (awaiting, phase progress 0.8, current steps trimmed, D-018, checkpoint, next step, burst row) | 2026-05-12 | state-manager | specs/product-brief.md + specs/architecture/ + tech-debt-register.md; single-commit burst per TD-VSDD-053 |

## Next Step

1. OPTIONAL: re-run /vsdd-factory:validate-brief on v1.2 to confirm qualitative bloat reduction + catch any remaining structure findings
2. Run market intelligence assessment (Task #8) before Phase 1 entry — still required gate
3. Parallel dispatch: /vsdd-factory:create-architecture (architect) + /vsdd-factory:create-prd (product-owner)
   - Architect inherits: vision + brief v1.2 + dependencies.md + ADR-0001 + conventions.md + oq-research.md + tech-debt-register.md
   - Product-owner inherits: brief v1.2 + OQ resolutions + Phase 1 Constraints table
4. After PRD + architecture: /vsdd-factory:phase-1d-adversarial-spec-review for 3-clean-pass convergence → human approval gate → Phase 2
5. SOQ-1..4 from oq-research.md are for architect attention (single LockFile type, atomic_write helper, Phase 3/4 trait seams, XDG-aware MonoclePaths)

## Historical Content

<!-- This section is populated by /vsdd-factory:compact-state when extracting historical content. -->

| Content | Location |
|---------|----------|
| Burst history | `cycles/cycle-001/burst-log.md` |
| Convergence trajectory | `cycles/cycle-001/convergence-trajectory.md` |
| Session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Lessons learned | `cycles/cycle-001/lessons.md` |
| Resolved blockers | `cycles/cycle-001/blocking-issues-resolved.md` |
