---
document_type: pipeline-state
level: ops
project: monocle
version: "7.93"
status: active
producer: state-manager
timestamp: 2026-06-17T00:00:00Z
phase: phase-3-v1A-wave-8
current_step: "D-330 (2026-06-17): S-033 adversarial pass 4 — no CRITICAL; all named fixes (HIGH-001/MED-001/LOW-001/LOW-004/MED-003 + Rulings A–H) confirmed holding. IMP-001 (monitor-spawn-before-Launching-broadcast ordering) + OBS-001 (orphan sidecar on collision) routed to implementer. Three cross-story integration findings F-W8INT-001/002/003 deferred to Wave-8 integration gate. Convergence counter 0/3 (pass 5 next after IMP-001/OBS-001 fix)."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-001..D-241 at cycles/cycle-001/decisions-archive.md. D-242..D-330 appended at cycles/cycle-001/decisions-archive.md §Phase-1d+Phase-2+Phase-3. Full durable_task_register YAML (127 entries) at cycles/cycle-001/task-register-full.yaml."
awaiting: "S-033 Wave 8: pass 4 REMEDIATED — IMP-001 (monitor-spawn ordering) + OBS-001 (orphan sidecar) to implementer; pass 5 next (convergence counter 0/3). Three integration findings F-W8INT-001/002/003 deferred to Wave-8 gate. Branch protection (PROC-BRANCH-PROTECTION-CONTEXTS) pending human/repo-admin before first Wave 8 PR merge."
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: 2026-06-03
dtu_services: [hook-endpoints-x5]
current_cycle: cycle-001
next_session_resume_protocol: |
  ============================================================================
  ZERO-CONTEXT RESUME CHECKPOINT v7.93 — 2026-06-17
  PHASE-3 v1A ACTIVE — WAVE 8 / S-033 ADVERSARIAL PASS 5 NEXT (after IMP-001/OBS-001 fix)
  ============================================================================
  POSITION: Phase-3 TDD implementation, v1A control-center scope, Wave 8.
  STATUS: S-033 adversarial pass 4 REMEDIATED (D-330). No CRITICAL. All prior
  named fixes (HIGH-001/MED-001/LOW-001/LOW-004/MED-003 + Rulings A–H) confirmed
  holding. Two in-scope findings routed to implementer: IMP-001 (monitor-spawn-
  before-Launching-broadcast ordering inversion) + OBS-001 (orphan sidecar on
  collision). Three cross-story integration findings F-W8INT-001/002/003 deferred
  to Wave-8 integration gate. SS-session-manager v2.7.3, S-033 v1.9.
  Convergence counter: 0/3. Next: implementer fixes IMP-001/OBS-001, then pass 5.

  READ FIRST (in order):
  1. /Users/jmagady/Dev/monocle/NEXT-SESSION-RESUME.md  <- concise entry point
  2. /Users/jmagady/Dev/monocle/CLAUDE.md               <- production-grade + routing
  3. This STATE.md (task register table below)          <- task register + history pointers

  CORPUS FACTS at Phase-3 start:
  Stories: 51 total / 311 pts (32 done Phase 1-3; 16 not_started v1A Waves 8-9;
           1 blocked S-PHASE-3-PREP; 2 draft S-032/S-DAEMON-WIRE-FIX-001)
  New v1A: S-033..S-048 (16 stories); EPIC-08 Session Manager; EPIC-09 Embedded PTY
  Waves: 8-9; 25 v1A BCs; 5 holdouts HS-EXP-011..015 anchored

  KEY VERSION PINS (canonical source: .factory/specs/version-pin-registry.yaml):
  STORY-INDEX v5.45 | sprint-state v1.46 | wave-schedule v2.1
  dependency-graph-expansion v2.5 | BC-INDEX v1.43.8 | EVAL-INDEX v1.19
  ARCH-INDEX v1.0.30 | SS-ipc v1.24.0 | SS-session-manager v2.7.3
  SS-embedded-pty v1.7.0 | SS-engine-module-v2-delta v1.6.0
  SS-daemon-wiring-v2-delta v1.11.4 | SS-deps-pin-manifest-v2-delta v1.0.2
  prd v1.28.3 | product-brief v2.0.4 | domain-monocle-vision-synthesis v2.2.3

  NEXT-ACTION (S-033 convergence — pass 4 REMEDIATED, pass 5 next):
  1. implementer: fix IMP-001 (monitor-spawn-before-Launching-broadcast ordering
     inversion) + OBS-001 (orphan sidecar cleanup on UUID collision path).
     Worktree dfc6765 (or current HEAD) has current implementation.
  2. adversary: fresh-context pass 5 (convergence counter 0/3).
     F-W8INT-001/002/003 are cross-story/deferred — do NOT flag as new in-scope findings.
  3. If CLEAN → counter 1/3. If BLOCKED → architect rulings + remediation burst.
  CI pre-requisites status:
    DTU clone: PASS (D-234, fidelity 1.0)
    ci.yml: PASS
    develop build/clippy/tests: PASS (1192 tests @ 50c195e)
    Branch protection contexts: BLOCKED (PROC-BRANCH-PROTECTION-CONTEXTS) — HUMAN ACTION NEEDED

  RATIFIED DECISIONS (do NOT re-litigate):
  - D-238: session-host-owns-PTY; daemon restart SURVIVES (CASE 2); NO tmux default
  - D-304: Autonomous Phase-2 dispatch authorized; no per-burst plan-review gate
  - D-315: Pre-pivot disposition RATIFIED; 32 done-historical; 3 active kept
    (S-032/S-DAEMON-WIRE-FIX-001/S-PHASE-3-PREP); actual pre-pivot count 35; 0 archive/retire
  - D-325: Phase-2 gate APPROVED; Phase-3 v1A active
  - D-326: S-033 Rulings A+B (scope + SessionSidecarV3 in monocle-ipc)
  - D-327: S-033 Rulings C+D (setsid host-side; host_conn storage S-033 scope)
  - D-328: S-033 Rulings E+F+G (DaemonToHost/HostToDaemon in monocle-ipc; EC-152 retry IPC-handler-only + second SpawnAck; single-lock atomicity for Running/EC-163/Terminated pairs)
  - D-329: S-033 Ruling H — MED-002 monitor↔entry generation guard DEFERRED to S-036; safe in v1 (3-layer argument); SS-session-manager v2.7.3
  - D-330: pass 4 REMEDIATED; IMP-001+OBS-001 to implementer; F-W8INT-001/002/003 to Wave-8 gate
  - Spawn-path Model A: SpawnOptions on wire; SpawnRecipe daemon-internal
  - IPC: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
  - PTY (ADR-0011): portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
  - SessionState: 5 variants (Launching/Running/Detached/Terminating/Terminated) in monocle-ipc
  - Full history: cycles/cycle-001/decisions-archive.md (D-001..D-330)

  KNOWN-FLAKY TESTS (do NOT flag as new findings):
  cli_daemon_stop, factory_self_referential, test_BC_2_07_006,
  wit-bindgen unmatched-skip, PATH isolation flake.

  factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'`
  develop HEAD: 50c195e (1192 tests passing; no v1A production code yet)
  ============================================================================
---

# Pipeline State: Monocle

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| -1 Reference Ingest | DONE 2026-05-11 | 8 repos; semport/ |
| 0.5-0.9 Brief + market intel | DONE 2026-05-14 | brief v2.0.4; validate-brief VALID |
| 1 Spec Crystallization | DONE D-170 APPROVED | 57-pass Phase-1d adversarial. 113 Phase-1 BCs (138 total incl Phase-2 v1A). <!-- version-pin-historical: BC-INDEX v1.43.7 at Phase-1d close --> |
| 2 Story Decomp v1A | PASSED/APPROVED D-325 | 26 passes (3/3 clean). 51 stories/311 pts. Gate APPROVED Joshua Magady 2026-06-16. |
| 3 TDD v1A Waves 8-9 | IN PROGRESS — Wave 8 | Wave 8 starting. Tier-1 root S-033 (monocle-session-host). Branch protection pending human. |
| 3 TDD Waves 1-7 (pre-pivot) | COMPLETE D-232 | 32/33 done (192/195 pts). 1514 tests. develop @ 6811103 |
| 4-7 | PENDING after Phase-3 v1A | Old observe-only scope superseded by v1A control-center |

## Blocking Issues

None. All durable task register items are non-blocking.

## Durable Task Register

101 active tasks. Full YAML detail (all 127 entries including 26 resolved/closed): `cycles/cycle-001/task-register-full.yaml`.

| ID | Status | Route | Block? | Subject (truncated) |
|----|--------|-------|--------|---------------------|
| F-GATE-IMP-001 | CLOSED D-323 | state-mgr | n | sprint-state inputs pin cascade stale — RESOLVED v1.46 |
| F-GATE-IMP-002 | CLOSED D-323 | story-writer | n | wave-schedule S-032 dep text — RESOLVED v2.0 |
| F-GATE-ADV-001 | CLOSED D-323 | state-mgr | n | sprint-state S-033..S-048 not_started→draft — RESOLVED v1.46 |
| F-GATE-ADV-002 | CLOSED D-323 | story-writer | n | missing epic detail files E-04..E-09 — RESOLVED v1.0 |
| F-GATE-ADV-003 | CLOSED D-323 | state-mgr | n | EVAL-INDEX inputs S-033..S-048 — RESOLVED v1.19 |
| F-GATE-ADV-004 | CLOSED D-323 | story-writer | n | dep-graph-expansion title Waves 4-7→4-9 — RESOLVED v2.4 |
| F-P13-SUG-001 | CLOSED D-323 | story-writer | n | BC-2.06.024 title in dep-graph — SWEPT |
| F-P20-BCGAP-001 | CLOSED D-323 | prod-owner | n | BC-2.08.006 atomic-write + path-canon — RESOLVED v1.4.0 |
| F-P21-SUG-001 | CLOSED D-323 | story-writer | n | S-040 BC-2.09.002 citation scope — SWEPT |
| F-P21-SUG-002 | CLOSED D-323 | story-writer | n | S-033 SpawnAck step-label — SWEPT |
| F-P24-SUG-001 | CLOSED D-323 | story-writer | n | S-038 body BC-table title — SWEPT |
| F-P24-SUG-002 | CLOSED D-323 | story-writer | n | S-046 body BC-table title — SWEPT |
| F-P25-SUG-001 | CLOSED D-323 | story-writer | n | STORY-INDEX AC-ranges S-044/S-040 — SWEPT |
| S-047-AC009-PTYRESET-QUALIFIER | CLOSED D-323 | story-writer | n | S-047 AC-009 PtyReset qualifier — SWEPT |
| MED-002-monitor-generation-guard | deferred-S-036 | architect/implementer | n | post-spawn monitor lacks generation/epoch guard; DEFERRED to S-036; safe in v1 (Ruling H — 3-layer: UUID v4 2^-122 + no GC on Launching + outer Mutex); SS-session-manager v2.7.3 |
| LOW-S033-SIDECAR-STRUCT-GUARD | pending | devops | n | [process-gap] no CI guard (POL-12 candidate) forbidding ad-hoc SessionSidecar other than monocle_ipc::types::SessionSidecarV3 |
| F-W8INT-001 | pending | wave-gate/architect | n | [integration] SessionListUpdate carries Vec<EnrichedSession> (no degraded/degraded_reason); I3-009 degraded indicator never reaches TUI via wire. Requires SS-08 broadcast wire-type decision (EnrichedSession vs SessionSnapshot). Owners: S-021/S-028. Surface at Wave-8 integration gate. |
| F-W8INT-002 | pending | wave-gate/architect | n | [integration] SessionManager::session_list() (spec'd "for InitialState IPC push") is unwired from snapshot_initial_state; spawned sessions absent from InitialState on reconnect. Bridge SessionManager sessions into InitialState (reconcile with hook-driven session_registry). Owner: S-022 (BC-2.05.002). Surface at Wave-8 integration gate. |
| F-W8INT-003 | pending | wave-gate/architect | n | [integration] Two disjoint session sources (SessionManager spawn path + hook-driven session_registry) both publish full-roster SessionListUpdate with whole-roster-replace semantics (BC-2.05.003 PC-2) → mutually clobber the TUI roster. Needs unified roster source/merge. Owners: S-018/S-028/S-033. Surface at Wave-8 integration gate. |
| INPUT-HASH-CHILD-RECOMPUTE | codification-pending | devops | n | input-hash drift re-accumulates when parent specs bump |
| DEP-PIN-SWEEP-RULE | pending | devops | n | extend POL-11 to grep crate-name+version prose literals |
| POL-11-PINFORMAT-BLIND-SPOT | codification-pending | devops | n | POL-11 misses path.md vX.Y.Z section format |
| ADR-0010-TRACE-256-MARKER | pending | architect | n | ADR-0010 Trace missing [superseded by 64 in v1.3.0] |
| PRD-COUNT-CROSSCHECK-RULE | pending | devops | n | POL-12 sibling: PRD BC-table count vs BC-INDEX count |
| PROC-DTU-VALIDATE-LOCATION | pending | devops | n | DTU validation misses cargo-binary clone location |
| ADV-W5GATE-HIGH-002 | pending | implementer | n | Duplicate S-009 handler dead code |
| HIGH-2-SECOND-SIGNAL-DEFER | deferred-wave-8 | implementer | n | Second-signal exit codes 143/130 deferred to S-DAEMON-WIRE-FIX-001 |
| ADV-W5GATE-MED-001 | pending | implementer | n | S-017 UDS socket spurious WARN on rebind |
| ADV-W5GATE-MED-003 | pending | architect | n | HookEvent serde round-trip fragility |
| #28 | partial | architect | n | prost/reqwest exact-patch pin verification |
| #34 | pending | prod-owner | n | BC-2.03.001 PC-3 DeferUntil cleanup |
| VP-DTU-001 | deferred-phase-4 | architect | n | VP-DTU-001 to be created in Phase 4 |
| F-WAVE1-004 | deferred-maintenance | devops | n | Cron schedule collision audit.yml + dtu-fidelity.yml |
| F-WAVE1-005 | deferred-maintenance | devops | n | xtask not in cargo-deny graph.targets |
| S-014-ADV-SS-engine-module-stale | pending | architect | n | SS-engine-module HookDecision code blocks stale |
| S-011-ADV-HookArgs-divergence | pending | architect | n | HookArgs struct diverges from SS-permissions-phase1.md |
| S-013-ADV-prost-types-not-pinned | pending | architect | n | prost-types not registered in SS-deps-pin-manifest |
| S-008-ADV-tempfile-spec | partial-closed | story-writer | n | AC-003 tempfile::persist spec; BC-2.01.007 anchor |
| S-015-ADV-tracing-test | pending | architect | n | tracing-test 0.2 not in SS-deps-pin-manifest |
| SS-01-ProjectDirs-from | pending | architect | n | SS-daemon-lifecycle ProjectDirs::new -> ::from |
| IMPL-HookDecision-serde | deferred-wave-5 | implementer | n | HookDecision + HookResponse Serialize/Deserialize derives |
| IMPL-on-hook-Defer | deferred-wave-5 | implementer | n | ClaudeCodeModule::on_hook() Defer routing |
| ADV-W3GATE-MED-001 | pending | implementer | n | last_hook_ts never written by hook handlers |
| ADV-W3GATE-MED-003 | pending | implementer | n | Only 1/5 hook endpoints in full-stack test path |
| HS-EXP-009-hint | pending | implementer | n | Exit 70 missing stderr remediation hint |
| PROC-SEMGREP-DECOUPLE | pending | devops | n | Semgrep silently skipped when Preflight fails |
| PROC-GATE-SKIPPED-LOUD | pending | devops | n | Build+Test silently skipped — need GATE_SKIPPED indicator |
| PROC-COMPUTE-INPUT-HASH-YAML | pending | devops | n | compute-input-hash mishandles YAML-object inputs |
| PROC-COMPUTE-INPUT-HASH-YAML-CLASS | pending | devops | n | 16 v1A stories show [pending] due to YAML-object parser gap |
| CIRCULAR-HASH-CASCADE-HOUSEKEEPING | pending | state-mgr | n | 2-pass --update to settle 21 circular-hash-cascade stale |
| S-025-TODO-S023-MERGE | pending | implementer | n | Replace 2 TODO(S-023-merge) markers post-rebase |
| S-025-MAKE-MODAL-DEAD-CODE | pending | implementer | n | Dead make_modal test helpers until S-026 |
| PROC-BRANCH-PROTECTION-CONTEXTS | pending | human | n | develop branch protection has empty required-check contexts |
| F-S025-ADV13-NIT-003 | pending | prod-owner | n | BC-2.06.016 Trace stale follow-up note |
| F-S025-ADV13-NIT-004 | pending | prod-owner | n | BC-2.06.004 EC-079 cites non-production string |
| F-S025-ADV20-PROC-001 | pending | architect | n | Test File Documentation Standards rule (SS-conventions) |
| F-S025-PATH-B-CLAUDE-MD | pending | human | n | CLAUDE.md MSRV 1.86->1.88 PENDING HUMAN ACTION |
| F-S025-ADV16-PROC-001 | pending | devops | n | agents must use cargo clippy --all-targets to match CI |
| F-S025-ADV16-PROC-002 | pending | devops | n | audit-table.md + SS-engine-module must sync atomically |
| F-S025-ADV22-PROC-001 | pending | devops | n | CI-enforced bare-filename architecture anchor resolution |
| F-S025-ADV24-MED-001 | partial | story-writer | n | Cross-story S-026/27/28/31 inputs[] + done-story body stale |
| F-S025-ADV24-MED-002 | pending | story-writer | n | VP-body SS-deps-pin-manifest stale (14 VPs; phase-5 fix) |
| Task-9-m8-S028-cross-story | pending | story-writer | n | S-028 lines 63+147 cross-story drift (BC-5.39.002 PC2) |
| F-S025-ADV28-OBS-001 | pending | architect | n | 3rd consecutive ADR same-burst internal-consistency defect |
| F-S025-ADV28-OBS-002 | pending | architect | n | ADR-0008 Canonical Source Registry + structural-claim pattern |
| ADR-HOOK-001 | pending | architect | n | ADR-HOOK-001 wire-protocol decisions to migrate into new ADR |
| ADR-HOOK-001-WIRING | pending | implementer | n | ADR-HOOK-001 wiring implementation |
| S-025-POST-MERGE-S1 | pending | story-writer | n | S-025 post-merge PO sweep Task #9 |
| S-025-POST-MERGE-TD1 | pending | architect | n | S-025 post-merge tech-debt items Task #9 |
| F-S026-ADV1-LOW-002 | pending | architect | n | BC-2.06.023 low-finding from S-026 review |
| PROCESS-GAP-CI-PARITY-1 | pending | human | n | CLAUDE.md Lint add --all-targets PENDING HUMAN ACTION |
| PROCESS-GAP-CI-PARITY-2 | pending | devops | n | CI parity gap codification (second item) |
| F-S027-DOC-002 | pending | tech-writer | n | S-027 doc finding #2 |
| F-S027-DOC-003 | pending | tech-writer | n | S-027 doc finding #3 |
| L-S027-004-PROCESS-GAP-REGISTRY-ATOMICITY | pending | devops | n | registry atomicity: spec bump + registry must be atomic |
| F-S028-NIT-002-DEFERRED | deferred-post-conv | story-writer | n | S-028 nit #2 deferred post-convergence |
| PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP | pending | devops | n | architect=spec-only; all code to implementer-in-worktree |
| PROCESS-GAP-TMP-COMMIT-MSG-MIXUP | pending | devops | n | use unique /tmp paths per story dispatch |
| PROCESS-GAP-PRMANAGER-EARLY-RETURN | pending | devops | n | pr-manager must complete all 9 steps |
| SIGTERM-TERMINAL-RESTORE | pending | implementer | n | TUI terminal restore on SIGTERM |
| SS-DAEMON-WIRING-SCROLLBACKDUMP-TERM | pending | architect | n | SS-daemon-wiring scrollback-dump terminal restore spec |
| ADR-0011-UPGRADE-TYPO | pending | architect | n | ADR-0011 upgrade-path typo |
| S28-001 | pending | story-writer | n | S-028 Trace housekeeping |
| ENGINE-RS-DEVERSION | pending | architect | n | engine.rs deversion (doc-comment version literals) |
| BC-INDEX-TRACE-SS08-COUNT | pending | prod-owner | n | BC-INDEX Trace SS-08 count discrepancy |
| SS-IPC-181-REDUNDANT-HISTORICAL-MARKER | pending | architect | n | SS-ipc line 181 redundant historical marker |
| OBS-P57-001 | deferred-housekeeping | architect | n | Phase-1d Pass-57 LOW observation |
| OBS-1-BC2.09.002-TRACE-LINE-NUMBERS | deferred-housekeeping | architect | n | BC-2.09.002 Trace cites stale SS-embedded-pty line numbers |
| DEDUP-IPC-HANDLER-SKELETON | pending-orch-action | architect | n | Dedup SS-session-manager IPC vs SS-daemon-wiring-v2-delta §3 |
| OBS-HS-PROSE-PHASE4-PREP | deferred-phase4-prep | story-writer | n | HS-EXP-014/013 prose imprecisions for Phase-4 holdout-eval |
| BURST-GAP-001-S038-HOOK-SCHEMA | pending | story-writer | n | S-038 hook schema gap |
| BURST-GAP-002-S033-SESSIONNOTREADY-VARIANT | pending | story-writer | n | S-033 SessionNotReady variant gap |
| BURST-GAP-003-S043-TUITERM-SCROLLBACK-API | pending | story-writer | n | S-043 tui-term scrollback API gap |
| BURST-GAP-004-S043-S042-CROSSSTORY-RESET | pending | story-writer | n | S-043/S-042 cross-story reset gap |
| BURST-GAP-005-S039-S043-CROSSSTORY-CONFIG | pending | story-writer | n | S-039/S-043 cross-story config gap |
| F-P1-S-005 | pending | story-writer | n | Phase-2 Pass-1 story S-005 finding |
| F-P2-S03 | pending | architect | n | Phase-2 Pass-2 S03 finding |
| F-P14-SUG-001 | deferred-post-phase-2 | story-writer | n | Phase-2 Pass-14 Suggestion #1 (deferred post-Phase-2) |
| F-P3-S02-PROCGAP | pending | devops | n | add refresh story input-pins to convergence burst checklist |
| F-P16-SUG-002 | pending | sess-reviewer | n | add wire-type crate-residency adversarial axis |
| F-P20-SUG-WIRE-CODE-LINT | pending | devops | n | POL: validate wire-error-code literals vs SS-ipc taxonomy |
| HS-EXP-006-TTY-CAVEAT | pending | implementer | n | HS-EXP-006 scored 0.85 — terminal raw-mode restore unobservable in non-TTY harness |
| SE-40 | held-D-114 | orchestrator | n | SE-40 (deliver-story from main session only; held) |

## Resolved/Closed Tasks (archived)

40 entries. Full detail at `cycles/cycle-001/task-register-full.yaml`:
POL-14-PARENTHETICAL-ANCHOR-PIN, POL-11-MIRROR-TABLE-BLIND-SPOT, TD-MULTI-CLIENT-ATTACH-STORM-001,
DTU-CLONE-STORY, ADV-W5GATE-HIGH-001, F-DW-HIGH-001, BC-HOOK-034-typo, S-005-main-wiring,
ADV-W3GATE-MED-002, ADV-W3GATE-MED-004, ADV-W4GATE-MED-002, S-029-PROCESS-GAP-PC-LABEL-DRIFT,
F-S025-ADV28-MED-001, F-S025-ADV28-MED-002, F-S025-ADV37-DEFER-001, F-S026-ADV6-DEFER-001,
POINTS-TALLY-RECONCILE, F-S026-ADV1-LOW-002, F-S027-DOC-001, F-S028-NIT-001, F-S028-NIT-002,
FLAKY-TIMING-5MS, F-S028-NIT-002-DEFERRED, PIVOT-CONTROL-CENTER, CC-TUITERM-WIP-SIGNOFF,
CC-GLOBAL-MOUSE-CAPTURE,
F-GATE-IMP-001, F-GATE-IMP-002, F-GATE-ADV-001, F-GATE-ADV-002, F-GATE-ADV-003, F-GATE-ADV-004,
F-P13-SUG-001, F-P20-BCGAP-001, F-P21-SUG-001, F-P21-SUG-002, F-P24-SUG-001, F-P24-SUG-002,
F-P25-SUG-001, S-047-AC009-PTYRESET-QUALIFIER (all closed D-323 2026-06-16).

## Decision History

Full decisions archive: `cycles/cycle-001/decisions-archive.md`
D-001..D-241: early phases; D-242..D-325: Phase-1d + Phase-2 + Phase-3 gate (appended 2026-06-16)

Key decisions last session:
- D-323 (2026-06-16): Phase-2 pre-gate cleanup burst COMPLETE. D-324: inputs-pin BLOCKER fix. D-325: Phase-2 gate APPROVED. D-326: S-033 adversarial Pass-1 rulings A+B (scope + SessionSidecarV3). SS-session-manager v2.7.0; S-033 v1.7.
- D-327 (2026-06-17): S-033 adversarial pass (post-rework) NOT CLEAN. Rulings C+D (setsid host-side; host_conn storage S-033 scope). SS-session-manager v2.7.0→v2.7.1; S-033 v1.7→v1.8. Counter 0/3.
- D-328 (2026-06-17): S-033 adversarial pass 2 NOT CLEAN. Rulings E+F+G (DaemonToHost/HostToDaemon in monocle-ipc; EC-152 retry IPC-handler-only + second SpawnAck; single-lock atomicity). SS-session-manager v2.7.1→v2.7.2; S-033 v1.8→v1.9. Counter 0/3.
- D-329 (2026-06-17): S-033 adversarial pass 3 (post-batch) — no CRITICAL; all prior fixes confirmed holding (worktree dfc6765). Ruling H: MED-002 DEFERRED to S-036 (3-layer safety). SS-session-manager v2.7.2→v2.7.3. Counter 0/3.
- D-330 (2026-06-17): S-033 adversarial pass 4 — no CRITICAL; all named fixes (HIGH-001/MED-001/LOW-001/LOW-004/MED-003 + Rulings A–H) confirmed holding. IMP-001 (monitor-spawn-before-Launching-broadcast ordering inversion) + LOW OBS-001 (orphan sidecar on collision) routed to implementer. Three cross-story integration findings F-W8INT-001/002/003 deferred to Wave-8 integration gate. Counter 0/3 (pass 5 next after IMP-001/OBS-001 fix). STATE v7.92→v7.93.
