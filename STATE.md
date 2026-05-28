---
document_type: pipeline-state
level: ops
project: monocle
version: "6.27"
status: active
producer: state-manager
timestamp: 2026-05-28T10:30:00Z
phase: phase-3-wave-6-IN-PROGRESS
current_step: "Wave 6 authorized (D-183). S-022 adversarial Pass 13 NITPICK_ONLY (1/3 clean). Pass 14 next."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "Phase 1 GATE-PASS-WITH-RESIDUAL (D-155). Phase 2 GATE-PASS-WITH-RESIDUAL (D-159). Phase 3 Wave 1 DONE (D-164), Wave 2 GATE-PASSED (D-166), Wave 3 GATE-PASSED (D-167), Wave 4 GATE-PASSED (D-175), Wave 5 GATE-PASSED (D-182). Phase 1d CONVERGED (D-169, D-170). Phase 2 expansion adversarial CONVERGED (D-172). See cycles/cycle-001/ for full convergence history. Wave 6 AUTHORIZED (D-183)."
awaiting: "S-022 delivery (TUI Client Connect + Initial State + Permission Msg Types)."
durable_task_register:
  outstanding:
    - id: "ADV-W5GATE-HIGH-001"
      subject: "daemon_start_sequence() doesn't wire DaemonState — integration story needed"
      status: pending
      detail: "Wave 5 gate adversarial: daemon_start_sequence() in monocle-runtime does not wire DaemonState fields (sock_file_path, last_hook_ts, ring). Daemon runs but TUI state queries return stale/default values. Requires integration story. Route to story-writer for new story in Wave 6/7."
      blocking: false
    - id: "ADV-W5GATE-HIGH-002"
      subject: "Duplicate S-009 handler dead code — cleanup needed"
      status: pending
      detail: "Wave 5 gate adversarial: S-009 HTTP handler has a duplicate code path introduced during Wave 5 IPC routing work. Dead code is non-functional but increases maintenance burden and confusion. Route to implementer for cleanup fix-PR."
      blocking: false
    - id: "ADV-W5GATE-MED-001"
      subject: "S-017 UDS socket creation spurious WARN on rebind"
      status: pending
      detail: "Wave 5 gate adversarial: S-017 daemon start emits spurious WARN log when rebinding an already-removed socket path. Confuses log readers. Add explicit pre-remove check before bind to eliminate the warning."
      blocking: false
    - id: "ADV-W5GATE-MED-003"
      subject: "HookEvent serde round-trip fragility — add constructors to monocle-core"
      status: pending
      detail: "Wave 5 gate adversarial: HookEvent deserialization relies on field-name stability without tagged union. Adding constructors to monocle-core would enforce correct shape at build time. Route to architect for BC update, then implementer."
      blocking: false
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
    - id: "S-008-ADV-tempfile-spec"
      subject: "AC-003 tempfile::persist spec wording divergence"
      status: partial-closed
      detail: "RingError #[non_exhaustive] fixed in-scope during wave-3-gate adversarial review (bef6f4b). Remaining: AC-003 tempfile::persist spec wording (story-writer); BC-2.01.007 S-TBD anchor (PO)."
      blocking: false
    - id: "S-015-ADV-tracing-test"
      subject: "tracing-test 0.2 not in SS-deps-pin-manifest"
      status: pending
      detail: "S-015 added tracing-test 0.2.6 with no-env-filter feature to monocle-runtime dev-deps. Not registered in SS-deps-pin-manifest.md. Architect must add."
      blocking: false
    - id: "S-012-self-ref-test-fix"
      subject: "3 self-referential tests fail — workspace root detection"
      status: closed-fixed-in-gate
      detail: "Fixed in ceebd2d (wave-3-gate): monocle_repo_root uses ancestors().find(.git) instead of cargo metadata. All 3 self-referential tests now pass. 447 tests total."
      blocking: false
    - id: "S-009-ADV-non-utf8"
      subject: "Non-UTF-8 canonical header treated as absent"
      status: accepted-observation
      detail: "Non-UTF-8 X-Monocle-Authorization falls through to alias path. HTTP headers are ASCII per RFC 7230; axum rejects non-ASCII before middleware. Zero security impact."
      blocking: false
    - id: "ADV-W3GATE-MED-001"
      subject: "last_hook_ts never written by hook handlers (future daemon wiring)"
      status: pending
      detail: "Wave 3 gate adversarial: DaemonState.last_hook_ts field exists but hook handler paths never write it. Observation-only — requires daemon wiring integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-002"
      subject: "Ring buffer DaemonState.ring never set to Some (main.rs stub)"
      status: pending
      detail: "Wave 3 gate adversarial: DaemonState.ring is always None; ring_buffer_fill_pct always returns 0.0. Requires main.rs wiring in integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-003"
      subject: "Only 1/5 hook endpoints tested in running-mode full-stack path"
      status: pending
      detail: "Wave 3 gate adversarial: Full-stack integration tests cover only pre-tool endpoint. 4/5 endpoints lack running-mode full-stack coverage. Phase 4 integration story."
      blocking: false
    - id: "ADV-W3GATE-MED-004"
      subject: "ring_buffer_fill_pct hardcoded to 0.0"
      status: pending
      detail: "Wave 3 gate adversarial: Metric always returns 0.0 because DaemonState.ring is never set. Tracked with ADV-W3GATE-MED-002 (same root cause)."
      blocking: false
    - id: "SS-01-ProjectDirs-from"
      subject: "SS-daemon-lifecycle.md ProjectDirs::new → ProjectDirs::from"
      status: pending
      detail: "SS-daemon-lifecycle.md line 269 uses ProjectDirs::new('monocle','monocle','monocle') — should be ::from('','','monocle') per SS-config.md and BC-2.04.006 v1.5.0 correction. Architect maintenance item."
      blocking: false
    - id: "IMPL-EnrichedSession-fields"
      subject: "EnrichedSession implementation: add 4 TUI fields + serde derives"
      status: deferred-wave-4
      detail: "engine.rs EnrichedSession needs project_name, started_at, token_count, cost_usd fields and Serialize/Deserialize derives. Spec updated in SS-engine-module.md v1.1.22. Implementation update is a prerequisite for S-021 (IPC types) and S-025 (Sessions Panel). Must be done as part of Wave 4/5 story implementation."
      blocking: false
    - id: "IMPL-HookDecision-serde"
      subject: "HookDecision + HookResponse: add Serialize/Deserialize derives"
      status: deferred-wave-5
      detail: "engine.rs HookDecision and HookResponse need Serialize/Deserialize. Required for IPC wire transport. Must be done as part of S-021 or S-018 implementation."
      blocking: false
    - id: "IMPL-on-hook-Defer"
      subject: "ClaudeCodeModule::on_hook() implement Defer routing logic"
      status: deferred-wave-5
      detail: "Current implementation returns Allow unconditionally (Phase 1 placeholder). S-018 (Hook Endpoint Routing) must implement the Defer path for permission overlay activation. BC-2.04.007 PC-3 specifies the full routing."
      blocking: false
    - id: "ADV-P1D-pin-staleness"
      subject: "Architecture Source pin staleness (cosmetic)"
      status: accepted-cosmetic
      detail: "SS-05 BCs pin SS-ipc.md v1.4.0 (current v1.6.0); SS-04 BCs pin v1.2.0 (current v1.3.0); SS-07 BCs pin v1.1.0 (current v1.3.0). Content is correct; only metadata pins lag. Will be swept at story implementation time."
      blocking: false
    - id: "ADV-W4GATE-MED-001"
      subject: "PATH env mutation in detect_ccr tests (test isolation)"
      status: pending
      detail: "Wave 4 gate adversarial (P01): detect_ccr tests mutate PATH env var globally without isolation guard. Risk: test ordering sensitivity. Fix: use temp_env::with_var or scope the mutation. Sourced from ADV-W4GATE-P01-MED-001."
      blocking: false
    - id: "ADV-W4GATE-MED-002"
      subject: "tracing::error() no-ops in monocle CLI binary (no subscriber)"
      status: pending
      detail: "Wave 4 gate adversarial (P01): monocle CLI binary does not initialize a tracing subscriber, so tracing::error!() calls in main.rs are silent no-ops. Fix: initialize subscriber at binary entry or replace with eprintln! for startup errors. Sourced from ADV-W4GATE-P01-MED-002."
      blocking: false
    - id: "HS-EXP-009-hint"
      subject: "Exit 70 missing stderr remediation hint for MONOCLE_RUNTIME_DIR"
      status: pending
      detail: "Wave 4 gate holdout (HS-EXP-009 score 0.8): daemon emits exit code 70 for invalid MONOCLE_RUNTIME_DIR but provides no stderr hint to the user. BC-2.04.003 requires human-readable diagnostics. Fix: print actionable remediation message to stderr before exit. Sourced from HS-EXP-009 holdout evaluation."
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
  COLD-START RESUME GUIDE — WAVE 5 GATE PASSED, WAVE 6 READY:

  SESSION CONTEXT:
    This is a greenfield-with-reference-ingest Rust TUI project (monocle).
    The orchestrator (vsdd-factory:orchestrator) coordinates all work.
    Read CLAUDE.md at the repo root for project principles and conventions.

  PIPELINE STATE:
    1. Phase 1 (Spec Crystallization): DONE — 113 BCs, 7 subsystems, 5 ADRs.
    2. Phase 2 (Story Decomposition): DONE (D-173) — 33 stories, 195 pts, 24 holdout scenarios.
    3. Phase 3 (TDD Implementation): IN PROGRESS.
       - Waves 1-4: DONE (19 stories, 101 pts, gates D-164/D-166/D-167/D-175).
       - Wave 5: GATE PASSED (D-182): 753 tests, develop @ 1ce7838.
         S-017 (PR#22, 06432cf, 29 tests), S-018 (PR#26, 654e281, 46 tests),
         S-019 (PR#25, 11540fc, 25 tests), S-020 (PR#24, f69d53a, 24 tests),
         S-021 (PR#23, acaacb9, 49 tests) — new monocle-ipc crate.

  DEVELOP BRANCH STATE:
    4. develop @ 1ce7838. Verify: git log --oneline -5 develop
    5. 753 tests total. clippy clean. fmt clean.
    6. Workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness,
       monocle (binary), monocle-config, monocle-ipc, xtask.

  ARTIFACT VERSIONS:
    7. STORY-INDEX v5.2. sprint-state v1.28. BC-INDEX v1.23 (113 BCs).
       SS-tui v1.7.0. ARCH-INDEX v1.0.16. PRD v1.27.2.

  NEXT ACTION — WAVE 6 IN PROGRESS (D-183 AUTHORIZED):
    8. Wave 6: 4 stories, 34 pts total. AUTHORIZED (D-183). S-022 SERIAL FIRST.
       S-022 (8 pts, EPIC-05) — TUI Connect + Initial State + Permission Prompt — IN PROGRESS.
         Depends on: S-021 (done), S-018 (done).
         Blocks: S-023, S-025, S-026.
         Target: monocle-ipc. BCs: BC-2.05.002, BC-2.05.005.
       S-023 (5 pts, EPIC-05) — Daemon Reconnect (SOQ-3) — after S-022.
         Depends on: S-022, S-019 (done).
         Target: monocle-ipc. BCs: BC-2.05.006, BC-2.05.007.
       S-025 (8 pts, EPIC-06) — TUI Skeleton + Sessions Panel — after S-022.
         Depends on: S-022, S-024 (done), S-030 (done).
         Target: monocle-tui. BCs: BC-2.06.004, BC-2.06.005, BC-2.06.007.
       S-026 (13 pts, EPIC-06) — Permission Overlay Core — after S-022 and S-023.
         Depends on: S-022, S-023, S-024 (done).
         Target: monocle-tui. BCs: BC-2.06.008-014, 016, 023, 024.

       EXECUTION ORDER: S-022 serial first → (S-023 ∥ S-025) → S-026.
       Dispatch S-022 via deliver-story skill immediately.

  NON-BLOCKING FOLLOW-UPS (durable task register — do NOT fix unless specifically tasked):
    9. ADV-W5GATE-HIGH-001: DaemonState wiring integration story (route to story-writer).
       ADV-W5GATE-HIGH-002: Duplicate S-009 handler cleanup (route to implementer).
       ADV-W5GATE-MED-001: UDS socket spurious WARN on rebind.
       ADV-W5GATE-MED-003: HookEvent serde constructors (route to architect + implementer).
       ADV-W4GATE-MED-001: PATH test isolation in detect_ccr.
       ADV-W4GATE-MED-002: Dead tracing subscriber in CLI binary.
       HS-EXP-009-hint: Exit 70 missing stderr remediation hint.
       See full list in durable_task_register above.

  FACTORY INFRASTRUCTURE:
    10. .factory/ mounted at factory-artifacts branch (orphan worktree).
    11. Run factory-worktree-health via devops-engineer FIRST on session start.
    12. Commit hooks: block-ai-attribution, validate-input-hash, validate-table-cell-count.
        NEVER use --no-verify. NEVER add Co-Authored-By: Claude.
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
| 1 Spec Crystallization | DONE (expansion complete, D-169 APPROVED) | 2026-05-27 | D-155 original gate. D-168: PRD 22→70 BCs. D-169: Phase 1d CONVERGED (15 passes, trajectory 15→0). D-170: human gate APPROVED. BC-INDEX v1.19 (112 BCs). |
| 2 Story Decomposition | DONE (D-173 APPROVED) | 2026-05-27 | D-159 original gate: 17 stories, 86 pts. D-170: re-entry for 48 new BCs. D-171: 16 stories (S-016..S-031, 109 pts) + 10 holdout scenarios (HS-EXP-001..010) produced. Total: 33 stories, 195 pts. D-172: adversarial story review 4 passes, trajectory 18→11→9→4 (0 CRIT/HIGH at Pass 4). D-173: human gate APPROVED. BC-INDEX v1.23 (113 BCs). STORY-INDEX v4.7. |
| 3 TDD Implementation | IN PROGRESS — Wave 5 GATE PASSED (D-182) | 2026-05-27 | Wave 1+2+3 DONE (83 pts, 447 tests, all 6 gates). Wave 4 GATE PASSED (D-175): 634 tests. Wave 5 GATE PASSED (D-182): 753 tests, 0 failures, clippy clean, fmt clean. 24/33 stories done (143/195 pts). Wave 6 ready. |
| 4-7 | not-started | — | |

## Wave 5 — GATE PASSED (D-182)

| Story | Points | Status | Notes |
|-------|--------|--------|-------|
| S-017 Daemon Start Sequence + Hook Tmpfile | 8 | done | PR #22, 06432cf, 29 tests, adv 13→5→0 (3 passes) |
| S-018 Hook Endpoint Routing + Event Bus | 8 | done | PR #26, 654e281, 46 tests, adv 10→4→4 (CONVERGED) |
| S-019 Daemon Auto-Start + MONOCLE_NO_AUTOSTART | 5 | done | PR #25, 11540fc, 25 tests, adv 7→2→1 (CONVERGED) |
| S-020 JSONL Ring Capacity and Rotation | 5 | done | PR #24, f69d53a, 24 tests, adv 12→8→0 (CONVERGED) |
| S-021 UDS Server + IPC Transport + Core Message Types | 8 | done | PR #23, acaacb9, 49 tests, adv 9→4→4 (CONVERGED) |

develop @ 1ce7838. 753 tests, 0 failures. 24/33 stories done, 143/195 pts (73%). Wave 5 gate PASSED (D-182). Wave 6 ready: S-022, S-023, S-025, S-026 (4 stories, 34 pts).

## Blocking Issues

None. All durable_task_register items non-blocking.

## Decisions Log (recent — D-175 through D-182)

D-047 through D-174 archived at: `cycles/cycle-001/decisions-archive.md`

| ID | Decision | Date | Made By |
|----|----------|------|---------|
| D-175 | Wave 4 gate PASSED (634 tests, 0 failures, clippy/fmt clean). DTU SKIP. ADV PASS (0 CRIT/HIGH). Demo PASS (3/3). Holdout PASS (mean 0.90). develop @ b8a4ab7. | 2026-05-27 | orchestrator |
| D-176 | S-017 DELIVERED — PR #22 @ 06432cf, 29 tests, adv 13→5→0. BC-2.04.001/010 satisfied. | 2026-05-27 | orchestrator |
| D-177 | S-018 DELIVERED — PR #26 @ 654e281, 46 tests, adv 10→4→4. BC-2.04.007/008/009/011 satisfied. | 2026-05-27 | orchestrator |
| D-178 | S-019 DELIVERED — PR #25 @ 11540fc, 25 tests, adv 7→2→1. BC-2.04.002/003 satisfied. | 2026-05-27 | orchestrator |
| D-179 | S-020 DELIVERED — PR #24 @ f69d53a, 24 tests, adv 12→8→0. BC-2.04.012 satisfied. Fix: 5a3eaf4. | 2026-05-27 | orchestrator |
| D-180 | S-021 DELIVERED — PR #23 @ acaacb9, 49 tests, adv 9→4→4. BC-2.05.001/003/004/008 satisfied. New monocle-ipc crate. | 2026-05-27 | orchestrator |
| D-181 | Wave 5 COMPLETE — 5/5 stories done, 753 tests, 24/33 stories (143/195 pts). monocle-ipc crate added. | 2026-05-27 | orchestrator |
| D-182 | Wave 5 gate PASSED (753 tests, 0 failures, clippy/fmt clean). DTU SKIP. ADV PASS (0 CRIT, 0 HIGH blocking; 2 HIGH obs tracked). Demo PASS (5/5). Holdout PASS (mean 0.94, min 0.80). develop @ 1ce7838. Wave 6 unblocked. | 2026-05-27 | orchestrator |
| D-183 | Wave 6 AUTHORIZED — 4 stories (S-022, S-023, S-025, S-026), 34 pts. Execution order: S-022 serial-first → (S-023 ∥ S-025) → S-026. All dependencies satisfied. Human approval: "Approve as documented" (2026-05-27). | 2026-05-27 | orchestrator |

## Key Tech Stack

ratatui 0.30, crossterm 0.29, tokio 1.52, axum 0.8, interprocess 2.4, prost 0.14,
serde_yaml_ng 0.10, wasmtime 44, directories 6, notify 8, russh 0.60, rmcp 1.6,
reqwest 0.13, nucleo 0.5, nix 0.30, serde 1 (derive), chrono 0.4, serde_json =1.0.149 (EXACT), rand =0.8.6 (EXACT).
28 pinned production deps. **manifest v1.1.17**. **PRD v1.27.2**. **BC-INDEX v1.23** (72 numbered BCs + 41 DTU BCs = 113 total). **ARCH-INDEX v1.0.16** (7 subsystems). **SS-tui v1.7.0**. **STORY-INDEX v5.2** (33 stories, 195 pts). **sprint-state v1.28** (24/33 done, 143/195 pts). MSRV: Rust 1.86 (Phase 1-2); Rust 1.92 (Phase 3, wasmtime 44). 39 codified disciplines (SE-1..SE-23 + SE-40 candidate). Workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask.

## Historical Content

| Content | Location |
|---------|----------|
| Burst history (v5.89..v6.10) | `cycles/cycle-001/burst-log.md` |
| Decisions D-047 through D-154 | `cycles/cycle-001/decisions-archive.md` |
| Phase 1 convergence history (R62-R122) | `cycles/cycle-001/phase-1-convergence.md` |
| Task queue (T-1 through T-131) | `cycles/cycle-001/completed-tasks.md` |
| Lessons learned (all rounds) | `cycles/cycle-001/lessons.md` |
| Prior session checkpoints (through v5.88) | `cycles/cycle-001/session-checkpoints.md` |
| Adversary reports | `.factory/plans/adversary-pass-*.md` |

## §Trace v6.27 (S-022 PASS 13 — NITPICK_ONLY, 1/3 CLEAN)

**S-022 Adversarial Pass 13 PERSISTED** (2026-05-28): NITPICK_ONLY — 0 findings.
- Pass 12 F-S022-ADV12-MED-001 RESOLVED (commit 7dacab7): 3 integration tests in hook_defer_race.rs invoke production handler stack.
- Documented architectural limitation (no async yield between timeout-Err and guard) adjudicated as legitimate acceptance per CLAUDE.md Principle 1.
- passes_clean_consecutive: 0 → 1. last_classification: MEDIUM_PRESENT → NITPICK_ONLY. Earliest convergence: Pass 15.
- Frontmatter: version 6.26 → 6.27, current_step updated.
STATE v6.26 → v6.27.

## §Trace v6.26 (WAVE 6 AUTHORIZED — S-022 IN PROGRESS)

**WAVE 6 AUTHORIZED** (2026-05-27): D-183. Human approval: "Approve as documented."
- 4 stories, 34 pts: S-022 (8 pts, serial-first), S-023 (5 pts), S-025 (8 pts), S-026 (13 pts).
- Execution order: S-022 serial-first → (S-023 ∥ S-025) → S-026.
- All dependencies satisfied. phase → phase-3-wave-6-IN-PROGRESS.
- Frontmatter: version 6.25→6.26, awaiting → S-022 delivery.
STATE v6.25 → v6.26.

## §Trace v6.25 (WAVE 6 COUNT CORRECTION — DURABILITY HARDENING)

**WAVE 6 COUNT CORRECTED** (2026-05-27): Documentation-only fix. Wave 6 has 4 stories (S-022, S-023, S-025, S-026; 34 pts), not 6 stories (52 pts). S-027 is Wave 7, not Wave 6. Updated: frontmatter `awaiting`, `next_session_resume_protocol` NEXT ACTION block (expanded to concrete per-story dependencies + BCs + execution order), Phase Progress row footer, §Trace v6.24 footer line. No story data changed — correction of a count propagation error introduced at v6.24.
STATE v6.24 → v6.25.

## §Trace v6.24 (WAVE 5 GATE PASSED — WAVE 6 READY)

**WAVE 5 GATE PASSED** (2026-05-27): D-182. All 6 gates passed.
- Gate 1 test-suite PASS: 753 tests, 0 failures, 2 pre-existing flaky, clippy clean, fmt clean.
- Gate 2 DTU SKIP: no DTU modules in Wave 5 (monocle-ipc uses interprocess, not hook DTU).
- Gate 3 adversarial PASS: 0 CRIT, 0 HIGH blocking. 2 HIGH architectural observations tracked as ADV-W5GATE-HIGH-001 (DaemonState wiring — integration story) + ADV-W5GATE-HIGH-002 (duplicate S-009 handler — cleanup).
- Gate 4 demo-evidence PASS: 5/5 stories, all ACs verified by integration tests.
- Gate 5 holdout PASS: mean 0.94, min 0.80. HS-EXP-001 1.0, HS-EXP-002 0.95, HS-EXP-007 1.0, HS-EXP-009 0.80.
- Gate 6 state-update PASS.
- develop @ 1ce7838. 24/33 stories done (143/195 pts, 73%). Wave 6 unblocked: S-022, S-023, S-025, S-026 (4 stories, 34 pts).
- Durable task register: +4 items (ADV-W5GATE-HIGH-001, ADV-W5GATE-HIGH-002, ADV-W5GATE-MED-001, ADV-W5GATE-MED-003).
- Frontmatter: version 6.23→6.24, phase → phase-3-wave-5-GATE-PASSED, awaiting → Wave 6 authorization.
STATE v6.23 → v6.24.

## §Trace v6.23 (WAVE 5 COMPLETE — ALL 5 STORIES DONE)

**WAVE 5 COMPLETE** (2026-05-27): D-181. All 5 Wave 5 stories delivered.
- S-018 (PR #26, 654e281, 46 tests, adv 10→4→4): Hook Endpoint Routing + Bounded Event Bus. BC-2.04.007/008/009/011 satisfied.
- S-019 (PR #25, 11540fc, 25 tests, adv 7→2→1): Daemon Auto-Start + MONOCLE_NO_AUTOSTART. BC-2.04.002/003 satisfied.
- S-020 (PR #24, f69d53a, 24 tests, adv 12→8→0): JSONL Ring Capacity and Rotation. BC-2.04.012 satisfied. Fix: 5a3eaf4 (ring.rs merge conflict).
- S-021 (PR #23, acaacb9, 49 tests, adv 9→4→4): UDS Server + IPC Transport + Core Message Types. BC-2.05.001/003/004/008 satisfied. New monocle-ipc crate.
- Total Wave 5: 34 pts, ~193 new tests. Cumulative: ~780 tests. 24/33 stories done (143/195 pts).
- Frontmatter: version 6.22→6.23, phase → phase-3-wave-5-COMPLETE, awaiting → Wave 5 gate.
- sprint-state v1.27→v1.28 (done 20→24, not_started 12→8, points_complete 109→143). STORY-INDEX v5.1→v5.2.
STATE v6.22 → v6.23.

§Trace v6.16 through v6.22 (Wave 4 delivery + Wave 5 individual story deliveries) archived to `cycles/cycle-001/burst-log.md`.
