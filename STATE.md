---
document_type: pipeline-state
level: ops
version: "2.0"
status: active
producer: state-manager
timestamp: 2026-05-13T04:30:00Z
phase: pre-phase-1-final-gate-FULLY-CONVERGED
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: monocle
mode: greenfield-with-reference-ingest
current_step: pre-phase-1-final-gate-FULLY-CONVERGED-spec-package-self-contained-awaiting-human-phase-1-approval
current_cycle: cycle-001
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: pending
dtu_services: [hook-endpoints-x5]
awaiting: "human Phase 1 approval; Phase 1 begins in fresh context with self-contained spec package"
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
| **Current Phase** | pre-phase-1-final-gate-FULLY-CONVERGED |
| **Current Step** | pre-phase-1-final-gate-FULLY-CONVERGED-spec-package-self-contained-awaiting-human-phase-1-approval |
| **Product brief** | `.factory/specs/product-brief.md` v1.4.7 (commit 816b1bc) |
| **Vision** | `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 (commit 4dfcffd; approved) |
| **Last Updated** | 2026-05-13T04:30:00Z |

## Phase Progress

| Phase | Status | Completed | Gate | Notes |
|-------|--------|-----------|------|-------|
| -1: Reference Ingest (8 repos / 5 planes) | DONE | 2026-05-11 | codebase-analyzer | 57+ artifacts; semport/ |
| 0.5-0.8: Brief v1.0→v1.2 + arch stubs | DONE | 2026-05-12 | product-owner | brief v1.2; 4 arch stubs |
| 0.9: Market Intel + validate-brief v2/v3 | DONE | 2026-05-12 | orchestrator | v3 VALID after v1.3 competitive positioning revision |
| 0.95: Pre-Phase-1 Consistency Audit | DONE | 2026-05-12 | consistency-validator | GAPS_FOUND non-blocking; fixes F-03/F-04/F-11 applied (b891b78) |
| 0.96: Production-Grade Re-Audit | DONE | 2026-05-12 | adversary | MULTIPLE_DEFER_PATTERNS — 14 violations identified and remediated (0bd4ba9) |
| 0.97: Production-Grade Remediation Burst | DONE | 2026-05-12 | state-manager | commits 0e4b0f4 70286e1 00a2993 79e268a 76db583 21c026d 4df2ff8 + this |
| 0.98: Validation Chain Post-Remediation | DONE | 2026-05-12 | consistency-validator + validate-brief | Rounds 1-3 textual defects decayed to zero; trajectory: 4IMP+6ADV → 2BLK+3IMP+2ADV → 0BLK+2IMP+3ADV; validate-brief v5 VALID (b7439ce) |
| 0.99: Adversary Fresh Pass (round-5) + Substantive Fix Burst | DONE | 2026-05-12 | adversary + specialists | Fresh pass (e2c224b): 4 CRITICAL+6 IMPORTANT+4 ADVISORY substantive defects — ALL FIXED IN-SCOPE (9 commits); D-022 logged |
| 0.99b: Round 6 Validation Chain | DONE | 2026-05-12 | consistency-validator + validate-brief + adversary | Rounds 6-7 audits surfaced nit-class fixes; resolved in round-7 fix burst |
| 0.99c: Round 7 Fix Burst | DONE | 2026-05-12 | specialists | serde_json concrete pin, rand declared, /healthz auth split, axum 0.8 idiom, deny.toml cross-ref, brief supplements complete; D-023 logged |
| 0.99d: Round 8 Validation Chain | DONE | 2026-05-12 | consistency-validator + validate-brief + adversary | R8 found 1 BLOCKING + 1 IMPORTANT + 1 ADVISORY; fix burst round-9 resolved all (190a849 + 438bf95) |
| 0.99e: Round 9 Fix Burst | DONE | 2026-05-12 | specialists | R8-001 phantom /hooks/post-tool-use removed; R8-002 stale "8"→"9" crate count; R8-003 typo; D-024 logged |
| 0.99f: Round 10 Adversary Final | DONE | 2026-05-12 | adversary | PRODUCTION_READY — 0 findings; novelty LOW; spec converged across all 15 artifacts |
| 0.99g: Final FC Lock-In Burst | DONE | 2026-05-12 | architect + product-owner | 6 FC items (FC-01..FC-06) locked as Phase 1 contracts; NEW SS-core-types-and-abi.md (700 lines); SS-daemon-lifecycle v1.0.3; SS-deps v1.1.4; brief v1.4.7; commits 4f5d4ff + 816b1bc + d77271a |
| Pre-Phase-1 Final Gate | FULLY CONVERGED + READY | 2026-05-12 | state-manager | 16 artifacts self-contained; 10 BCs pre-staged; D-025 logged |
| 1: Spec Crystallization | READY — awaiting human approval | | | |
| 2: Story Decomposition | not-started | | | |
| 3: TDD Implementation | not-started | | | |
| 4: Holdout Evaluation | not-started | | | |
| 5: Adversarial Refinement | not-started | | | |
| 6: Formal Hardening | not-started | | | |
| 7: Convergence | not-started | | | |

## Current Phase Steps

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| Round 10 adversary fresh pass | adversary | DONE | PRODUCTION_READY; 0 findings; D-024 logged |
| FC lock-in burst: SS-core-types-and-abi.md (NEW 700 lines) + SS-daemon-lifecycle v1.0.3 + SS-forward-compatibility v1.1 | architect | DONE | commit 4f5d4ff |
| FC lock-in burst: brief v1.4.7 (6 FC items as Phase 1 contracts; supplements 10; 10 BCs pre-staged) | product-owner | DONE | commit 816b1bc |
| FC lock-in burst: SS-deps v1.1.4 (constant_time_eq ^0.3 + futures ^0.3) | architect | DONE | commit d77271a |
| PHASE-1-READY close-out: STATE.md + burst-log + session-checkpoints | state-manager | DONE | this commit |

## Decisions Log

<!-- D-001..D-015 archived to cycles/cycle-001/burst-log.md. Recent decisions below. -->

| ID | Decision | Rationale | Phase | Date | Made By |
|----|----------|-----------|-------|------|---------|
| D-016 | validate-brief v1.1: NEEDS_WORK (bloat 3.6x; JC-1 unresolved; leakage intentional) | Report at planning/brief-validation.md | pre-phase-1 | 2026-05-12 | state-manager |
| D-017 | OQ-01..OQ-11 researched; 10/11 HIGH confidence; 4 SOQs surfaced | planning/oq-research.md (1666 lines) | pre-phase-1 | 2026-05-12 | state-manager |
| D-018 | Brief v1.2 + 4 arch stubs. Bloat Option A. All 11 OQs + 4 SOQs + 5 JCs resolved | dependencies.md; ADR-0001; conventions.md; tech-debt-register.md; commit 6ac4279 | pre-phase-1 | 2026-05-12 | state-manager |
| D-019 | Brief v1.3 VALID after competitive positioning revision; pre-phase-1 consistency audit GAPS_FOUND but no blockers; ready for human Phase 1 approval gate | brief v1.3 d6a8291; validation v3 b3d9560; consistency audit b891b78; pre-gate fixes a46a7ce | pre-phase-1 | 2026-05-12 | state-manager |
| D-020 | Production-grade canonical principle articulated and remediated; 14 defer-patterns fixed in-scope across brief/vision/architecture; Q-A1 vision v1.1 re-approved; Q-B R-001 reassessed at <10% (dropped from active risk acceptance); CLAUDE.md establishes principle + agent routing as project-binding; upstream canonicalization filed at drbothen/vsdd-factory#129; dispatcher bug filed at #130 | commits 0bd4ba9 0e4b0f4 70286e1 00a2993 79e268a 76db583 21c026d 4df2ff8 8342239 | pre-phase-1-final-gate-post-remediation | 2026-05-12 | state-manager |
| D-021 | Validation chain rounds 2-3 converge clean: validate-brief v5 VALID (b7439ce); consistency audits 0f28619+f8bffd8 caught self-introduced defects from prior remediation/fix bursts; all defects fixed in-scope per production-grade principle; convergence trajectory: round-1 4-IMP-6-ADV → round-2 2-BLK-3-IMP-2-ADV → round-3 0-BLK-2-IMP-3-ADV (cleanly decaying) | see burst-log.md burst-11 | pre-phase-1-final-gate-post-fix-burst | 2026-05-12 | state-manager |
| D-022 | Round 5 substantive-fix burst: adversary fresh pass found 4 CRITICAL+6 IMPORTANT+4 ADVISORY substantive defects (different class from rounds 1-4 textual defer-patterns); ALL FIXED IN-SCOPE per production-grade routing principle. Human decisions: Q-license MIT/Apache-2.0 dual, Q-permission-enum Option A re-derive. New architecture artifacts: SS-permissions-phase1.md (281 lines), SS-daemon-lifecycle.md (287 lines), ADR-0003 MIT/Apache-2.0 (199 lines). Upstream issues filed: #129 canonicalization, #130 dispatcher bug, #131 URL coherence axis. | commits 4dfcffd+6d87d6c+2308b31+9f25dcd+6e3c658+44019c2+d544731+ee7b3fb+c28fc64 | pre-phase-1-final-gate-post-round-5 | 2026-05-12 | state-manager |
| D-023 | Round 7 micro-fix burst resolved 8 findings from round-6 audits: F-R6-001 CRITICAL serde_json concrete pin =1.0.149; F-R6-002/G-02 IMP SS-conventions tokio prose typo 1.44→1.52; F-R6-003 IMP /healthz two-router auth split (axum 0.8); F-R6-004 IMP rand =0.8.6 added to SS-deps Pin Manifest (EXACT; 8→9 EXACT-pinned); F-R6-005 ADV axum 0.8 with_graceful_shutdown idiom; F-R6-006 ADV /healthz removed from body-size criterion endpoints; G-01 IMP brief supplements frontmatter complete (9 entries); G-03 IMP deny.toml content moved canonical to SS-conventions, ADR-0003 cross-reference added. Brief at v1.4.5. | commits d78fc13+a22ca03+803ea63+5589849 | pre-phase-1-final-gate-post-round-7 | 2026-05-12 | state-manager |
| D-024 | Rounds 8-10 convergence. Round 8 (01e030f): 1 BLOCKING + 1 IMPORTANT + 1 ADVISORY. Round 9 fix burst (190a849 + 438bf95): R8-001 phantom /hooks/post-tool-use route removed from SS-daemon-lifecycle; R8-002 stale count "8 security-sensitive" corrected to "9" in SS-deps-pin-manifest; R8-003 typo "remediatedstarting" corrected in SS-conventions. Round 10 adversary fresh pass: PRODUCTION_READY — 0 findings across all severity classes. Novelty LOW. Spec package converged at 15 artifacts. Phase 1 gate READY. | commits 190a849+438bf95 (round 9) | pre-phase-1-final-gate-converged | 2026-05-12 | state-manager |
| D-025 | Pre-Phase-1 FC lock-in: 6 forward-compatibility items (FC-01..FC-06) locked into binding Phase 1 contracts per human authorization to clear context for fresh Phase 1 start. New artifact SS-core-types-and-abi.md (700 lines) defines monocle-core public stability surface: MONOCLE_ABI_VERSION constant, #[non_exhaustive] enum policy, FactoryAdapter trait with VsddFactoryAdapter impl, prost HookEnvelope schema with schema_version field. SS-daemon-lifecycle v1.0.3 adds JSONL format_version + versioned auth token prefix (monocle-v1:<64-hex>). SS-deps v1.1.4 adds constant_time_eq + futures pins. 10 behavioral contracts pre-staged for Phase 1 PRD (BC-ABI-001/002, BC-TYPES-001, BC-FACTORY-001/002, BC-PROTO-001/002, BC-RING-001, BC-AUTH-001/002). Spec package SELF-CONTAINED for fresh Phase 1 context. | commits 4f5d4ff + 816b1bc + d77271a | pre-phase-1-final-gate-FULLY-CONVERGED | 2026-05-12 | state-manager |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | no | TUI product requires UX spec |

## Blocking Issues

| ID | Issue | Severity | Blocking Phase | Owner | Resolution |
|----|-------|----------|---------------|-------|------------|

## Session Resume Checkpoint

<!-- Latest checkpoint. Prior checkpoints archived to cycles/cycle-001/session-checkpoints.md. -->

**Cycle:** cycle-001 | **Phase:** pre-phase-1-final-gate-FULLY-CONVERGED | **Mode:** greenfield-with-reference-ingest

### What This Is

monocle is a Rust TUI for managing AI coding harness sessions (Claude Code, future CodeMachine, others) across multiple projects and hosts. Five-plane architecture: Runtime, Static (customization explorer), Workflow (factory-awareness), Harness (EngineModule), TUI (lazy* signature). Observe-only for state; action-only for permission overlays + keybinding dispatch. Single Ctrl-\ tmux popup over user's editor.

### Where We Are

Pre-Phase-1 spec package FULLY CONVERGED and self-contained. 10 audit rounds clean. Round 10 adversary fresh pass: PRODUCTION_READY (0 findings). FC burst (commits 4f5d4ff + 816b1bc + d77271a) locked 6 forward-compatibility items into binding Phase 1 contracts per human authorization. All 14 substantive adversary findings + 6 FC items resolved in-scope. NEW SS-core-types-and-abi.md (700 lines) defines monocle-core public stability surface. 10 BCs pre-staged for Phase 1 PRD. Spec package ready for Phase 1 dispatch in fresh context — Phase 1 agents read these artifacts as canonical inputs without needing this session's history. D-025 logged.

### Immediate Next Action

Human approves Phase 1 gate. Orchestrator dispatches `/vsdd-factory:run-phase 1` which spawns: business-analyst (create-domain-spec) → product-owner (create-prd, will formalize the 10 pre-staged BCs + ~12 additional Phase 1 BCs) → architect (create-architecture, will build workspace Cargo.toml from SS-deps + implement the SS-core-types-and-abi traits and constants + wire SS-daemon-lifecycle protocol + scaffold .github/workflows/r001-monitor.yml from SS-conventions §R-001 Monitoring Workflow) → product-owner (phase-1-prd-revision iter 1-3) → adversary (phase-1d-adversarial-spec-review 3 clean passes) → human Phase 1 approval → Phase 2.

### Critical Artifacts (read in this order for Phase 1)

1. `CLAUDE.md` (project root) — production-grade principle + agent routing table
2. `.factory/specs/research/domain-monocle-vision-synthesis.md` v1.1.2 (commit 4dfcffd; approved)
3. `.factory/specs/product-brief.md` v1.4.7 (commit 816b1bc)
4. `.factory/specs/architecture/SS-core-types-and-abi.md` (NEW, 700 lines, 7 BCs pre-staged)
5. `.factory/specs/architecture/SS-daemon-lifecycle.md` v1.0.3
6. `.factory/specs/architecture/SS-deps-pin-manifest.md` v1.1.4
7. `.factory/specs/architecture/SS-conventions-anti-patterns.md` v1.2.2 (commit 438bf95)
8. `.factory/specs/architecture/SS-permissions-phase1.md` v1.0 (commit 9f25dcd)
9. `.factory/specs/architecture/SS-forward-compatibility.md` v1.1 (PHASE 1 READY verdict)
10. `.factory/specs/architecture/adr/ADR-0001-wasmtime-vs-wasmi.md` v1.0.1 (commit ad6a303)
11. `.factory/specs/architecture/adr/ADR-0002-nucleo-acceptance-with-reeval-trigger.md` v1.0
12. `.factory/specs/architecture/adr/ADR-0003-license-selection.md` v1.0.1 (commit d544731)
13. `.factory/specs/dtu-assessment.md` v1.1
14. `.factory/tech-debt-register.md` (empty — TD-001 retired)
15. `.factory/STATE.md` (this file)

### Key Tech Stack (architect inherits)

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14, serde_yaml_ng 0.10, wasmtime 44, similar 3, directories 6, notify 8, russh 0.60 (Phase 4), rmcp 1.6 (Phase 4), tempfile 3, clap 4.6, arboard 3, tracing 0.1, thiserror 2, anyhow 1, reqwest 0.13, nucleo 0.5 (ADR-0002 accepted), pulldown-cmark 0.13, serde_json =1.0.149 (EXACT-pinned), rand =0.8.6 (EXACT-pinned), bytes (direct pin), semver 1, constant_time_eq ^0.3 (caret pin; BC-AUTH-001 timing-safe auth comparison), futures ^0.3 (caret pin; FactoryAdapter::subscribe StateChangeStream). 28 named workspace pins total. 9 EXACT-pinned crates: tokio, axum, prost, serde_json, rand, wasmtime, russh, rmcp, reqwest. MSRV Phase 1: Rust 1.86; Phase 3: 1.92.

### Critical Hook Lessons

- input-hash: 7-char truncated MD5 or sentinel "[live-state]" (NOT SHA-256)
- block-ai-attribution: rejects "Co-Authored-By: Claude" or robot emoji in commits
- validate-table-cell-count: every data row pipe count must match header exactly
- Use `git commit -F /tmp/<file>` (heredoc-in-bash blocked at large payloads)
- `.factory/planning/` NOT registered; use `.factory/plans/` for new planning docs
- FC items resolved pre-Phase-1: Phase 1 agents read SS-core-types-and-abi.md for the canonical monocle-core public stability surface (ABI version constant, FactoryAdapter trait signature, #[non_exhaustive] enum policy, prost HookEnvelope wire schema). Do NOT re-derive these — they are LOCKED.

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (all bursts through round-15) | `cycles/cycle-001/burst-log.md` |
| Prior session checkpoints | `cycles/cycle-001/session-checkpoints.md` |
| Full decisions D-001..D-015 | `cycles/cycle-001/burst-log.md` |
| Convergence trajectory | `cycles/cycle-001/convergence-trajectory.md` |
| Lessons learned | `cycles/cycle-001/lessons.md` |
| Resolved blockers | `cycles/cycle-001/blocking-issues-resolved.md` |
