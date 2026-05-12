---
document_type: pipeline-state
level: ops
version: "2.0"
status: active
producer: state-manager
timestamp: 2026-05-12T22:30:00Z
phase: pre-phase-1-final-gate
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: monocle
mode: greenfield-with-reference-ingest
current_step: market-intel-and-validate-v2-complete-awaiting-brief-v1.3
current_cycle: cycle-001
dtu_required: false
awaiting: brief v1.3 (competitive positioning revision) then re-validate then Phase 1
---

<!--
  STATE.md SIZE BUDGET: Keep this file under 200 lines.
  Historical content belongs in cycle files, NOT here.
  Run /vsdd-factory:compact-state if this file grows past 200 lines.
-->

# Pipeline State: Monocle

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | monocle |
| **Mode** | greenfield-with-reference-ingest |
| **Language** | Rust |
| **Current Phase** | pre-phase-1-final-gate |
| **Current Step** | market-intel-and-validate-v2-complete-awaiting-brief-v1.3 |
| **Product brief** | `.factory/specs/product-brief.md` v1.2 |
| **Canonical vision** | `.factory/specs/research/domain-monocle-vision-synthesis.md` |
| **Last Updated** | 2026-05-12T22:30:00Z |

## Phase Progress

| Phase | Status | Completed | Gate | Notes |
|-------|--------|-----------|------|-------|
| -1: Reference Ingest (8 repos / 5 planes) | DONE | 2026-05-11 | codebase-analyzer | 57+ artifacts; semport/ |
| 0.5-0.8: Brief v1.0→v1.2 + arch stubs | DONE | 2026-05-12 | product-owner | brief v1.2; 4 arch stubs |
| 0.9: Market Intel + validate-brief v2 | DONE | 2026-05-12 | orchestrator | CAUTION + NEEDS_WORK |
| 1: Spec Crystallization | not-started | | | |
| 2: Story Decomposition | not-started | | | |
| 3: TDD Implementation | not-started | | | |
| 4: Holdout Evaluation | not-started | | | |
| 5: Adversarial Refinement | not-started | | | |
| 6: Formal Hardening | not-started | | | |
| 7: Convergence | not-started | | | |

## Current Phase Steps

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| Brief v1.2 + arch stubs | product-owner | DONE | brief v1.2 (350 lines); 4 arch stubs; D-018 |
| Market intel assessment | orchestrator | DONE | planning/market-intelligence.md (CAUTION) |
| validate-brief v2 | orchestrator | DONE | plans/brief-validation-v2.md (NEEDS_WORK) |
| Compact STATE.md + resume checkpoint | state-manager | DONE | burst-log.md + session-checkpoints.md created |

## Decisions Log

<!-- D-001..D-015 archived to cycles/cycle-001/burst-log.md. Recent decisions below. -->

| ID | Decision | Rationale | Phase | Date | Made By |
|----|----------|-----------|-------|------|---------|
| D-016 | validate-brief v1.1: NEEDS_WORK (bloat 3.6x; JC-1 unresolved; leakage intentional) | Report at planning/brief-validation.md | pre-phase-1 | 2026-05-12 | state-manager |
| D-017 | OQ-01..OQ-11 researched; 10/11 HIGH confidence; 4 SOQs surfaced | planning/oq-research.md (1666 lines) | pre-phase-1 | 2026-05-12 | state-manager |
| D-018 | Brief v1.2 + 4 arch stubs. Bloat Option A. All 11 OQs + 4 SOQs + 5 JCs resolved | dependencies.md; ADR-0001; conventions.md; tech-debt-register.md; commit 6ac4279 | pre-phase-1 | 2026-05-12 | state-manager |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | no | TUI product requires UX spec |
| DTU Assessment | pending | Deferred until architecture complete |

## Blocking Issues

| ID | Issue | Severity | Blocking Phase | Owner | Resolution |
|----|-------|----------|---------------|-------|------------|

## Session Resume Checkpoint

<!-- Latest checkpoint. Prior checkpoints archived to cycles/cycle-001/session-checkpoints.md. -->

**Cycle:** cycle-001 | **Phase:** pre-phase-1-final-gate | **Mode:** greenfield-with-reference-ingest

### What This Is

monocle is a Rust TUI for managing AI coding harness sessions (Claude Code, future CodeMachine, others) across multiple projects and hosts. Five-plane architecture: Runtime, Static (customization explorer), Workflow (factory-awareness), Harness (EngineModule), TUI (lazy* signature). Observe-only for state; action-only for permission overlays + keybinding dispatch. Single Ctrl-\ tmux popup over user's editor.

### Where We Are

- 8 reference repos fully ingested into `.factory/semport/`
- Vision approved by human verbatim 2026-05-11 (D-012)
- Brief v1.2 at `.factory/specs/product-brief.md` (350 lines, commit 6ac4279)
- 4 architecture stubs at `.factory/specs/architecture/` + `.factory/tech-debt-register.md`
- 11 OQs + 4 SOQs + 5 JCs resolved via oq-research.md
- Two quality gates: validate-brief v2 (NEEDS_WORK, ONE blocker) + market intel (CAUTION)

### Immediate Next Action

Produce brief v1.3 with ONE targeted revision: 5-7 sentences in Competitive Positioning acknowledging Anthropic's `claude agents` (shipped 2026-05-11, research preview) and repositioning monocle's differentiation on depth and mechanism (hook-protocol-native vs attach-and-reply; VecDeque<PromptModal> overlay vs none; diff preview vs none; trigger-trace vs none; workflow plane vs none).

After v1.3 lands: run validate-brief, expected VALID, then dispatch Phase 1 starting with `/vsdd-factory:create-domain-spec`.

### Critical Artifacts (read in this order)

1. `.factory/specs/research/domain-monocle-vision-synthesis.md` — canonical vision (approved)
2. `.factory/specs/product-brief.md` v1.2 — current brief
3. `.factory/planning/oq-research.md` — 11 OQs + 4 SOQs resolved
4. `.factory/planning/market-intelligence.md` — CAUTION verdict, agent view shock
5. `.factory/plans/brief-validation-v2.md` — NEEDS_WORK verdict, single blocker
6. `.factory/specs/architecture/dependencies.md` — 24-crate pin manifest
7. `.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md` — first ADR
8. `.factory/specs/architecture/conventions.md` — anti-patterns
9. `.factory/tech-debt-register.md` — TD-001 nucleo dormant

### Resolution Summary

| Category | Status |
|---|---|
| OQ-01..OQ-11 (architect open questions) | All RESOLVED via oq-research.md recommended defaults |
| SOQ-1..SOQ-4 (second-order questions) | All accepted as architect inputs |
| JC-1 (7-type Phase 1 vs 2) | RESOLVED option B1 — moved to Phase 2 Exit Criteria |
| JC-2 (PostToolUse endpoint) | RESOLVED — omit for v1; revisit Phase 4 OTel |
| JC-3 (port 2748 fixed vs OS-assigned) | RESOLVED — OS-assigned via OQ-04 |
| EX-1 (workspace 13 crates) | RATIFIED |
| EX-2 (SessionStart + UserPromptSubmit) | ADDED to Phase 1 |
| OQ-M1 (agent view IPC coexistence) | NEW from market intel — architect input |
| OQ-M2 (claude-manager arch review) | NEW from market intel — architect input |
| OQ-M3 (PermissionRequest as 6th endpoint) | NEW from market intel — architect input |

### Pipeline Path After v1.3 Lands

Phase 1: create-domain-spec -> create-prd -> create-architecture -> phase-1-prd-revision (3x max) -> phase-1d-adversarial-spec-review (3 clean passes) -> HUMAN APPROVAL -> Phase 2

### Key Tech Stack (architect inherits)

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14, serde_yaml_ng 0.10, wasmtime 44, similar 3, directories 6, notify 8, russh 0.60 (Phase 4), rmcp 1.6 (Phase 4), tempfile 3, clap 4.6, arboard 3, tracing 0.1, thiserror 2, anyhow 1, reqwest 0.13, nucleo 0.5 (TD-001 dormant). MSRV Phase 1: Rust 1.86; Phase 3: 1.92.

### Known Inconsistencies

1. `.factory/planning/` not in artifact-path-registry.yaml; existing files grandfathered; new docs go in `.factory/plans/`.
2. Arch stubs at `.factory/specs/architecture/` may not match registry path patterns (validate-brief-v2 flagged PARTIAL FAIL).

### Critical Hook Lessons

- input-hash: 7-char truncated MD5 or sentinel "[live-state]" (NOT SHA-256)
- block-ai-attribution: rejects "Co-Authored-By: Claude" or robot emoji in commits
- validate-table-cell-count: every data row pipe count must match header exactly
- Use `git commit -F /tmp/<file>` (heredoc-in-bash blocked at large payloads)
- `.factory/planning/` NOT registered; use `.factory/plans/` for new planning docs

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (all 8 bursts) | `cycles/cycle-001/burst-log.md` |
| Prior session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Full decisions D-001..D-015 | `cycles/cycle-001/burst-log.md` |
| Convergence trajectory | `cycles/cycle-001/convergence-trajectory.md` |
| Lessons learned | `cycles/cycle-001/lessons.md` |
| Resolved blockers | `cycles/cycle-001/blocking-issues-resolved.md` |
