---
document_type: pipeline-state
level: ops
project: monocle
version: "6.04"
status: active
producer: state-manager
timestamp: 2026-05-26T20:00:00Z
phase: phase-3-WAVE-2-GATE-PASSED
current_step: "Wave-2-gate-passed-Wave-3-ready-to-dispatch"
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "Phase 1 GATE-PASS-WITH-RESIDUAL (D-155). Phase 2 GATE-PASS-WITH-RESIDUAL (D-159). Phase 3 Wave 1 DONE (D-164), Wave 2 GATE-PASSED (D-166). See cycles/cycle-001/ for full convergence history."
awaiting: "Wave 3 Batch A: S-007 DONE, S-008 DONE (S-009 unblocked). S-012 + S-015 not started. 3 stories / 24 pts remaining."
durable_task_register:
  outstanding:
    - id: "#28"
      subject: "prost/reqwest exact-patch pin verification"
      status: partial
      detail: "prost = '=0.14.1' VERIFIED on crates.io + resolved in Cargo.lock. reqwest = '=0.13.0' exists but latest 0.13.x is 0.13.3 — reqwest not yet activated by any Phase 1 member. Architect adjudication needed when S-009 activates reqwest."
      blocking: false
    - id: "#34"
      subject: "BC-2.03.001 PC-3 DeferUntil cleanup"
      status: pending
      detail: "BC-2.03.001 v1.0.5 PC-3 still enumerates DeferUntil in supporting types. Authority hierarchy resolves to story v1.4 + SS-engine-module v1.1.20 (no DeferUntil). PO mechanical fix."
      blocking: false
    - id: "BC-HOOK-034-typo"
      subject: "BC-HOOK-034 typo decorated_by -> deprecated_by"
      status: pending
      detail: "Cosmetic typo in BC-HOOK-034 frontmatter. Non-blocking. Maintenance sweep."
      blocking: false
    - id: "VP-DTU-001"
      subject: "VP-DTU-001 to be created by architect in Phase 4"
      status: deferred-phase-4
      detail: "All 41 BC-HOOK files cite VP-DTU-001 as verification property. VP-DTU-001 is Phase 4 deferral marker; architect creates when holdout-evaluator scope is operational."
      blocking: false
    - id: "F-WAVE1-004"
      subject: "Cron schedule collision audit.yml + dtu-fidelity.yml"
      status: deferred-maintenance
      detail: "Both 0 0 * * 0 UTC. Stagger to avoid cache thrash. Low impact."
      blocking: false
    - id: "F-WAVE1-005"
      subject: "xtask not in cargo-deny [graph].targets"
      status: deferred-maintenance
      detail: "xtask is dev-tooling-only. Targets limit cargo-deny to 3 CI runner triples. Low likelihood Phase 1 risk. Document in deny.toml."
      blocking: false
    - id: "F-WAVE1-006"
      subject: "monocle-proto prost-build no-op build cost"
      status: closed-S-013-delivered
      detail: "build.rs no-op stub resolved — S-013 (PR#12, 8653977) wired real .proto files. Closed."
      blocking: false
    - id: "Wave-2-gate-dep-cleanup"
      subject: "Wave-2 gate dep hygiene — 4 findings fixed in closeout e2898be"
      status: closed
      detail: "Removed monocle-proto from monocle-runtime deps; removed futures+semver from monocle-core deps; added sock_file_path to DaemonState. develop @ e2898be. Closed by wave-gate PASS_WITH_OBSERVATIONS (2026-05-26T18:00:00Z)."
      blocking: false
    - id: "S-014-ADV-SS-engine-module-stale"
      subject: "SS-engine-module.md HookDecision code blocks stale"
      status: pending
      detail: "S-014 adversary surfaced stale HookDecision code blocks in SS-engine-module.md. Architect update needed. Sourced from S-014 adversarial review R2."
      blocking: false
    - id: "S-011-ADV-HookArgs-divergence"
      subject: "HookArgs struct diverges from SS-permissions-phase1.md"
      status: pending
      detail: "S-011 adversary surfaced HookArgs struct definition diverging from SS-permissions-phase1.md canonical definition. Architect update needed. Sourced from S-011 adversarial review."
      blocking: false
    - id: "S-013-ADV-prost-types-not-pinned"
      subject: "prost-types not registered in SS-deps-pin-manifest"
      status: pending
      detail: "S-013 adversary noted prost-types is a transitive dependency but not explicitly registered in SS-deps-pin-manifest.md. Architect must add explicit pin. Sourced from S-013 adversarial review."
      blocking: false
    - id: "S-005-main-wiring"
      subject: "S-005 main.rs wiring: 10s drain timeout + second-signal detection + signal-path lock release"
      status: pending
      detail: "S-005 graceful shutdown implemented at library level (BC-2.01.004 satisfied). Requires main.rs wiring for: (a) 10-second drain timeout enforcement, (b) second-signal detection (force-kill escalation), (c) signal-path lock release handoff. Deferred from S-005 scope (requires main.rs which belongs to TUI integration story). Route to integration story in Wave 3 or post-Wave-3."
      blocking: false
  se_candidates:
    - id: SE-40
      occurrences: 2
      threshold: 3
      description: "Orchestrator drives deliver-story from main session only; never delegates to sub-orchestrator that cannot spawn fresh-context specialists."
      status: HELD per D-114
  process_discoveries:
    - "Tautological-test risk pattern: test-writer compared fixture.clone() to fixture (identity = false-green)"
    - "Single-giant-commit-hides-todo: implementer R1 claimed completion but left todo!() in binary entrypoint"
    - "Production-Grade Default Rule 1 'Future:' comment violations caught 3x"
    - "Sibling-sweep gaps in 3-place status tracking (sprint-state/STORY-INDEX/story-frontmatter)"
    - "clippy --all-targets vs --workspace scope gap (test code violations invisible in lib-only mode)"
next_session_resume_protocol: |
  COLD-START RESUME GUIDE — WAVE 3 IN PROGRESS:

  1. Run factory-worktree-health via devops-engineer (BLOCKING).
  2. Verify: git log --oneline -1 develop → fe4db96 ([S-008]).
  3. Read STATE.md + CLAUDE.md.
  4. Wave 3 progress: S-007 DONE (PR #14), S-008 DONE (PR #15). S-009 now UNBLOCKED.
  5. Remaining: S-012 (8 pts, FactoryAdapter), S-015 (8 pts, ClaudeCodeModule), S-009 (8 pts, Auth Token — depends on S-008 ✓).
  6. All 3 remaining stories can run in parallel (S-009 dependency on S-008 is now satisfied).
  7. Per-story delivery: worktree → stubs → tests → impl → adversary convergence → PR → merge → cleanup.
  8. After all 5 Wave 3 stories merged + wave-gate: Phase 3 COMPLETE.
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: pending
dtu_services: [hook-endpoints-x5]
current_cycle: cycle-001
---

# Pipeline State: Monocle — ZERO-CONTEXT RESUME GUIDE

## Phase Progress

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| -1 Reference Ingest | DONE | 2026-05-11 | 8 repos; semport/ |
| 0.5-0.9 Brief + arch stubs | DONE | 2026-05-14 | |
| Pre-Phase-1 Final Gate | DONE | 2026-05-14 | D-054. 26 adv rounds. 22 BCs. |
| 1 Spec Crystallization | GATE-PASS-WITH-RESIDUAL | 2026-05-19 | D-155. 39 disciplines. |
| 2 Story Decomposition | GATE-PASS-WITH-RESIDUAL | 2026-05-19 | D-159. 17 stories; 86 pts. |
| 3 TDD Implementation | IN-PROGRESS | — | Wave 1+2 DONE (49 pts); **Wave 3 READY** (34 pts) |
| 4-7 | not-started | — | |

## Wave 3 Current Status

| Story | Points | Status | Batch | Deps |
|-------|--------|--------|-------|------|
| S-007 Crash Recovery | 5 | done | A | none |
| S-008 JSONL Ring Format | 5 | done | A | none |
| S-012 FactoryAdapter Trait | 8 | not_started | A | none |
| S-015 ClaudeCodeModule | 8 | not_started | A | none |
| S-009 Auth Token Wire Format | 8 | not_started | B | S-008 |

develop @ fe4db96. 332+ tests. clippy clean. 13/17 stories done, 59/86 pts.

## Blocking Issues

None. All durable_task_register items non-blocking.

## Decisions Log (D-155 through D-166)

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-155 | Phase 1 GATE PASS WITH RESIDUAL — asymptote-at-1 accepted (R121→1, R122→1). TD-VSDD-PHASE-1-ASYMPTOTIC-REVERSE-CASCADE documented. | 2026-05-19 | human (Josh Magady) |
| D-156 | Phase 2 PRE-APPROVAL — user explicitly pre-approved Phase 2 execution before context clear. | 2026-05-19 | human (Josh Magady) |
| D-157 | Phase 2 GATE PASS WITH DOCUMENTED RESIDUAL — 12 adv rounds (r01..r12); trajectory 26→1 (96% reduction). TD-VSDD-PHASE-2-ASYMPTOTIC-PROPAGATION-DRIFT. | 2026-05-19 | orchestrator |
| D-158 | Phase 3 PENDING HUMAN GATE — Phase 2→3 transition requires explicit human authorization. | 2026-05-19 | orchestrator |
| D-159 | Phase 2 GATE PASS WITH RESIDUAL FINALIZED — r13 empirical asymptote confirmation. Two human authorizations. | 2026-05-19 | human (Josh Magady) |
| D-160 | Phase 3 APPROVED-TO-EXECUTE — user re-authorized post-context-clear: "phase 3 is approved to execute." | 2026-05-20 | human (Josh Magady) |
| D-161 | Wave 1 partial — S-001 merged; S-DTU-001 BC-HOOK prereq pending. SE-40 candidate 1st occurrence. | 2026-05-20 | orchestrator |
| D-162 | Wave 1 S-001 COMPLETE — PR #1 (a6f119c) + PR #2 (184f7d4). 3 adversary rounds. 16 findings closed. | 2026-05-21 | orchestrator |
| D-163 | Wave 1 COMPLETE — PR #3 (cfeb1346) S-DTU-001 merged. Both Wave 1 stories DONE (8 pts). | 2026-05-21 | orchestrator |
| D-164 | Wave 1 FULLY CLOSED — wave-gate PASS_WITH_OBSERVATIONS. PR #4 (681c179) closeout. 134 tests. | 2026-05-21 | orchestrator |
| D-165 | Wave 2 APPROVED-TO-EXECUTE — user authorization 2026-05-24. 9 stories/41 pts dispatched. | 2026-05-24 | human (Josh Magady) |
| D-166 | Wave 2 GATE PASSED — PASS_WITH_OBSERVATIONS. 332 tests, clippy clean. 4 dep hygiene findings fixed (e2898be). | 2026-05-26 | orchestrator |

Decisions D-047 through D-154 archived at: `cycles/cycle-001/decisions-archive.md`

## Key Tech Stack

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14,
serde_yaml_ng 0.10, wasmtime 44, directories 6, notify 8, russh 0.60, rmcp 1.6,
reqwest 0.13, nucleo 0.5, nix 0.30, serde 1 (derive), chrono 0.4, serde_json =1.0.149 (EXACT), rand =0.8.6 (EXACT).
28 pinned production deps. **manifest v1.1.17**. **PRD v1.26.12**. **BC-INDEX v1.11**. MSRV: Rust 1.86 (Phase 1-2); Rust 1.92 (Phase 3, wasmtime 44). 39 codified disciplines (SE-1..SE-23 + SE-40 candidate).

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (pre-Phase-3) + D-048..D-052 | `cycles/cycle-001/burst-log.md` |
| Decisions D-047 through D-154 | `cycles/cycle-001/decisions-archive.md` |
| Phase 1 convergence history (R62-R122) | `cycles/cycle-001/phase-1-convergence.md` |
| Task queue (T-1 through T-131) | `cycles/cycle-001/completed-tasks.md` |
| Lessons learned (all rounds) | `cycles/cycle-001/lessons.md` |
| Prior session checkpoints (through v5.88) | `cycles/cycle-001/session-checkpoints.md` |
| Adversary reports | `.factory/plans/adversary-pass-*.md` |

## §Trace v6.01 (D-166)

**D-166 WAVE 2 GATE PASSED** (2026-05-26T18:00:00Z): Wave-gate PASS_WITH_OBSERVATIONS. 332 tests, clippy clean. 4 dep hygiene findings fixed in e2898be: removed monocle-proto from monocle-runtime; removed futures+semver from monocle-core; added sock_file_path to DaemonState. develop @ e2898be. Phase → `phase-3-WAVE-2-GATE-PASSED`. Wave 3 ready: Batch A parallel (S-007/S-008/S-012/S-015); Batch B after S-008 (S-009). STATE v6.00 → v6.01. SE-23 PASS. S-PHASE-3-PREP BLOCKED on rc.19+ (does NOT block Wave 3).

## §Trace v6.04 (S-008 delivery)

**S-008 JSONL RING FORMAT VERSION DELIVERED** (2026-05-26): PR #15 merged at develop @ fe4db96. BC-2.01.007 fully satisfied. 13 integration tests. 5 adversary rounds (convergence with spec-text residual). Deferred: AC-003 tempfile::persist wording (story-writer); consumer surface docs (story-writer); RingError #[non_exhaustive] (wave-gate); BC-2.01.007 S-TBD anchor (PO). sprint-state v1.14→v1.15: done 12→13, not_started 4→3, points_complete 54→59. STATE v6.03→v6.04. Wave 3 Batch A: 2/4 done; Wave 3 total: 2/5 done (10/34 pts). S-009 now UNBLOCKED.

## §Trace v6.03 (S-007 delivery)

**S-007 CRASH RECOVERY CHECKPOINT DELIVERED** (2026-05-26): PR #14 merged at develop @ 9982ff0. BC-2.01.006 fully satisfied at library level. 15 integration tests. 5 adversary rounds (3/3 convergence). Deferred: regex-lite manifest registration (architect); daemon-level AC wiring (S-005-main-wiring). sprint-state v1.13→v1.14: done 11→12, not_started 5→4, points_complete 49→54. STATE v6.02→v6.03. Wave 3 Batch A: 1/4 done; Wave 3 total: 1/5 done (5/34 pts).

## §Trace v6.02 (compaction)

**STATE.md v6.01 → v6.02 COMPACTION** (2026-05-26T20:00:00Z):
- NORMATIVE: STATE.md compacted from 1,169 lines to under 200 lines. Move-not-delete: every extracted line placed in a cycle file.
- NORMATIVE: Extracted to `cycles/cycle-001/phase-1-convergence.md`: numbered history entries 1-113 + Pre-Phase-1 Final Gate + Phase 1+ Convergence Policy + Phase 1 Entry Artifact Inventory + Phase 1 Gate PASS Summary + Phase 2 Entry Conditions.
- NORMATIVE: Appended to `cycles/cycle-001/session-checkpoints.md`: Session Resume Checkpoint v5.88.
- NORMATIVE: Appended to `cycles/cycle-001/lessons.md`: Critical Hook Lessons section.
- NORMATIVE: Appended to `cycles/cycle-001/burst-log.md`: §Trace v5.89, v5.91, v5.93, v5.94.
- NORMATIVE: Decisions D-047 through D-154 archived in `cycles/cycle-001/decisions-archive.md` (prior session). STATE.md now retains only D-155 through D-166 (last 12).
- NORMATIVE: STATE v6.01 → v6.02. SE-23 compliance: SM touched only STATE.md and cycle files; zero spec/story/sprint-state changes.
- SE-16d PASS: 2026-05-26T20:00:00Z > 2026-05-26T18:00:00Z (D-166 entry).
