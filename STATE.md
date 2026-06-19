---
document_type: pipeline-state
level: ops
project: monocle
version: "8.01"
status: active
producer: state-manager
timestamp: 2026-06-19T00:01:00Z
phase: phase-3-v1A-wave-8
current_step: "D-336 (2026-06-19): S-035 MERGED to develop via PR #43 @ 270b7d4. Fourth Wave-8 story (third Tier-2). attach_session/detach_session, BC-2.08.007+BC-2.08.008, 8 pts. 9-pass adversarial convergence (3 consecutive CLEAN: passes 7/8/9). CRIT-001 silent Detached→Terminated fixed; Ruling L (proxy_task kill-reader for attached sessions); 4-disposition action×state matrix. Sprint-state v1.50; STORY-INDEX v5.49. Wave 8: 4/12 done (27/74 pts). S-036 now UNBLOCKED."
prior_step: "D-335 (2026-06-19): S-037 MERGED to develop via PR #42 @ a7e4081. Third Wave-8 story (second Tier-2). GC task + rename_session. 7-pass adversarial convergence (3 consecutive CLEAN: passes 5/6/7). SEC-001 (CWE-20) + SEC-002 (CWE-706) fixed IN-SCOPE (b2d65db). Sprint-state v1.49; STORY-INDEX v5.48. Wave 8: 3/12 done (19/74 pts)."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-001..D-241 at cycles/cycle-001/decisions-archive.md. D-242..D-332 appended at cycles/cycle-001/decisions-archive.md §Phase-1d+Phase-2+Phase-3. Full durable_task_register YAML (127+ entries) at cycles/cycle-001/task-register-full.yaml."
awaiting: "Next Tier-2 dispatch: S-038 (hook injection, 3 pts, needs S-033 — ready). S-036 (needs S-033+S-034+S-035 — now UNBLOCKED). SEC-006-CCR-URL-VALIDATION MUST be fixed before S-045. S-046 waits on S-032; S-047 waits on S-033+S-034+S-035+S-046; S-048 waits on S-022+S-033+S-047. F-W8INT-001/002/003 parked for Wave-8 integration gate."
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: 2026-06-03
dtu_services: [hook-endpoints-x5]
current_cycle: cycle-001
next_session_resume_protocol: |
  ============================================================================
  ZERO-CONTEXT RESUME CHECKPOINT v8.00 — 2026-06-19
  PHASE-3 v1A ACTIVE — WAVE 8 TIER 2 — S-035 DONE, NEXT: S-038/S-036
  ============================================================================
  POSITION: Phase-3 TDD implementation, v1A control-center scope, Wave 8 Tier 2.
  STATUS: S-035 MERGED (PR #43 @ 270b7d4, D-336). develop @ 270b7d4. S-035
  worktree cleaned up. 10 workspace crates (incl. monocle-session-host). 36/51
  stories done (219/311 pts). Wave 8: 4/12 done (27/74 pts).
  Human directive: CONTINUE WAVE-8 TIER-2 AUTONOMOUSLY.
  Demo default GOING FORWARD: WEBM + .tape ONLY (NO GIF).
  Branch protection: 11 required CI contexts enforced (10 ci.yml + DTU fidelity).
  NOTE: Branch protection uses BARE check names (no 'CI /' prefix) — FIXED D-335.

  READ FIRST (in order):
  1. /Users/jmagady/Dev/monocle/NEXT-SESSION-RESUME.md  <- concise entry point
  2. /Users/jmagady/Dev/monocle/CLAUDE.md               <- production-grade + routing
  3. This STATE.md (task register table below)          <- task register + history pointers

  CORPUS FACTS (post-S-035-merge, checkpoint D-336):
  Stories: 51 total / 311 pts (36 done; 14 draft v1A Waves 8-9; 1 blocked)
  New v1A: S-033..S-048 (16 stories); EPIC-08 Session Manager; EPIC-09 Embedded PTY
  Waves: 8-9; 25 v1A BCs; 5 holdouts HS-EXP-011..015 anchored

  KEY VERSION PINS (canonical source: .factory/specs/version-pin-registry.yaml):
  STORY-INDEX v5.49 | sprint-state v1.50 | wave-schedule v2.1
  dependency-graph-expansion v2.7 | BC-INDEX v1.43.8 | EVAL-INDEX v1.23
  ARCH-INDEX v1.0.30 | SS-ipc v1.24.0 | SS-session-manager v2.15.0
  SS-embedded-pty v1.7.0 | SS-engine-module-v2-delta v1.6.0
  SS-daemon-wiring-v2-delta v1.11.4 | SS-deps-pin-manifest-v2-delta v1.0.2
  prd v1.28.3 | product-brief v2.0.4 | domain-monocle-vision-synthesis v2.2.3
  BC-2.08.007 v1.5.6 | BC-2.08.008 v1.3.7 | S-035 v1.2.3

  NEXT-ACTION (Wave-8 Tier 2 — S-035 done, continue Tier-2):
  S-038 (hook injection, 3 pts, needs S-033 — ready).
  S-036 (rediscovery, 8 pts, needs S-033+S-034+S-035 — NOW UNBLOCKED).
  PREREQUISITE: SEC-006-CCR-URL-VALIDATION MUST be fixed before S-045 (CWE-20/93).
  S-046 waits on S-032; S-047 needs S-033+S-034+S-035+S-046;
  S-048 needs S-022+S-033+S-047. F-W8INT-001/002/003 parked for Wave-8 gate;
  do NOT re-raise as in-scope findings during per-story delivery.

  PER-STORY FLOW: stub-architect → test-writer → implementer → adversarial
  convergence (3 consecutive CLEAN, Step 4.5) → demo evidence (WEBM+.tape) →
  push → pr-manager all 9 steps → merge → worktree cleanup.

  CI-PARITY RULES (all 7 apply):
  1. clippy --workspace --all-targets -- -D warnings (IN worktree)
  2. python3 scripts/check_version_pins.py (POL-11) from REPO ROOT
     python3 scripts/check_structural_claims.py (POL-12) from REPO ROOT
  3. No version literals in doc-comments (POL-11 will flag)
  4. Registry atomicity: BC/SS spec bump + version-pin-registry.yaml = one atomic
     factory-artifacts commit (L-S027-004)
  5. Unique /tmp paths per story dispatch
  6. Architect = spec only; all code to implementer-in-worktree
  7. pr-manager must complete all 9 steps

  RATIFIED DECISIONS (do NOT re-litigate):
  - D-238: session-host-owns-PTY; daemon restart SURVIVES (CASE 2); NO tmux default
  - D-304: Autonomous Phase-2 dispatch; no per-burst plan-review gate
  - D-315: Pre-pivot disposition RATIFIED (32 done; 3 active kept; 0 archive)
  - D-325: Phase-2 gate APPROVED; Phase-3 v1A active
  - D-326..D-331: S-033 Rulings A–H; adversarial convergence COMPLETE (7 passes, 3 clean)
  - D-332: S-033 MERGED PR #40 @ c7e10f2
  - D-333: Wave-8 Tier-2 autonomous delivery authorized; demo WEBM+.tape-no-GIF
  - D-334: S-034 MERGED PR #41 @ 4dfe0db. Kill path complete. Rulings H/I/J/K.
  - D-335: S-037 MERGED PR #42 @ a7e4081. GC task + rename_session. SEC-001/002 fixed in-scope.
  - D-336: S-035 MERGED PR #43 @ 270b7d4. attach/detach. Ruling L. 9-pass convergence. S-036 UNBLOCKED.
  - Spawn-path Model A: SpawnOptions on wire; SpawnRecipe daemon-internal
  - IPC: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
  - PTY (ADR-0011): portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
  - SessionState: 5 variants (Launching/Running/Detached/Terminating/Terminated)
  - Full history: cycles/cycle-001/decisions-archive.md (D-001..D-336)

  KNOWN-FLAKY TESTS (do NOT flag as new findings):
  cli_daemon_stop, factory_self_referential, test_BC_2_07_006,
  wit-bindgen unmatched-skip, PATH isolation flake.

  factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'`
  develop HEAD: 270b7d4 (S-035 merged PR #43; 11 CI checks passed; clean merge)
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
| 3 TDD v1A Waves 8-9 | IN PROGRESS — Wave 8 Tier 2 | S-033 MERGED PR #40 @ c7e10f2 (D-332). S-034 MERGED PR #41 @ 4dfe0db (D-334). S-037 MERGED PR #42 @ a7e4081 (D-335). S-035 MERGED PR #43 @ 270b7d4 (D-336). Wave 8: 4/12 done (27/74 pts). 36/51 stories done (219/311 pts). S-036 UNBLOCKED. Next: S-038/S-036. |
| 3 TDD Waves 1-7 (pre-pivot) | COMPLETE D-232 | 32/33 done (192/195 pts). 1514 tests. develop @ 6811103 |
| 4-7 | PENDING after Phase-3 v1A | Old observe-only scope superseded by v1A control-center |

## Blocking Issues

None. All durable task register items are non-blocking.

## Durable Task Register

108 active tasks. Full YAML detail (all 136+ entries including 28 resolved/closed): `cycles/cycle-001/task-register-full.yaml`.

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
| SEC-006-CCR-URL-VALIDATION | pending | implementer/security | n | ccr_base_url flows unvalidated from TUI wire to child env (CWE-20/93, MEDIUM); MUST be addressed before S-045 (ClaudeCodeModule::spawn_recipe) activates CCR URL injection path end-to-end. Anchor: S-045. |
| F-S033-CIPARITY-POL11-DOCCOMMENT | note | devops | n | PR #40 required de-versioning SS-session-manager doc-comment citations (POL-11) mid-PR (commit 579f077 on factory-artifacts); confirms POL-11 catches version literals in source doc-comments — reinforces CLAUDE.md CI-PARITY rule 3. |
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
| BC-2.08.008-PC6-NATURAL-EXIT | deferred-S-039/S-040 | implementer | n | Session-host natural-child-exit watch (PTY master EOF -> HostToDaemon::StateChanged{Terminated} + Goodbye without a Kill) — BC-2.08.008 PC-6 / Ctrl-D canonical test vector. NOT S-034 scope (S-034 session-host = Kill handler only, SS-session-manager v2.11.0 Ruling K). Owned by S-039/S-040 PTY output pipeline. Daemon-side Terminated broadcast already wired by S-034 AC-004. |
| PROCESS-GAP-ARCHITECT-NO-COMMIT | codification-pending | devops | n | [process-gap] architect agent edits SS-session-manager.md + registry but leaves them uncommitted — recurred 2x in S-034 cycle. Positive observation D-336: S-035 architect committed spec+registry atomically (622913e, 4b31bc0, 16f2f28, cf80bba, d9cca3c); pattern held. Consider downgrading/closing if no recurrence. Codify: architect must commit its own spec+registry atomically OR orchestrator always verifies git -C .factory status clean. |
| PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH | RESOLVED D-335 | devops/human | n | [process-gap] RESOLVED: human-authorized D-335; devops PATCHed 11 required-check contexts to bare names. PR #42 merged CLEAN without admin override (first Wave-8 PR). |
| SEC-001-RENAME-NEWNAME-VALIDATION | RESOLVED D-335 | implementer/security | n | CWE-20 new_name validation in rename_session — FIXED IN-SCOPE S-037 commit b2d65db. MAX_DISPLAY_NAME_BYTES=256 + InvalidSessionName mapping. Was candidate S-047 deferral; corrected per production-grade default. |
| SEC-002-RENAME-UUID-GUARD | RESOLVED D-335 | implementer/security | n | CWE-706 UUID guard in rename_session — FIXED IN-SCOPE S-037 commit b2d65db. security-reviewer re-verified PASS; adversary re-verified CLEAN. |
| F-S037-OBS-SPAWN-DISPLAYNAME-NOCAP | pending | architect/implementer | n | [LOW] spawn_session default display_name (harness_id — basename) not length-capped while rename_session caps at 256 bytes; latent asymmetry, unreachable in practice (NAME_MAX ~255), pre-existing, outside BC-2.08.005 scope. Non-blocking. |
| F-S037-PRREV-OBS | pending | implementer | n | [LOW] pr-reviewer non-blocking: (a) consider filtering Unicode Cf-class chars in new_name; (b) spawn_blocking for GC filesystem ops future improvement; (c) defensive guard suggestion on EC-163 path. Bundle for future hardening pass. |
| DEMO-BINARY-ARTIFACTS-DEVELOP | pending | devops/human-decision | n | ~3.2 MB WEBM added by S-037 (docs/demo-evidence/S-037/) + ~13 MB from S-033 + ~3.1 MB from S-035 (docs/demo-evidence/S-035/). Now 3 stories' WEBMs on develop (S-033/S-034 GIFs+WEBM, S-037 WEBM, S-035 WEBM). Repo-hygiene policy decision still pending. |
| F-S035-001 | RESOLVED D-336 | architect/implementer | n | kill-after-attach reader ownership → RESOLVED via SS-session-manager Ruling L (proxy_task fast-path kill confirmation; kill reader=None+proxy=Some delegation). Fixed in-scope. |
| F-S035-AC005-DAEMON-BROADCAST | pending | story-writer (S-039/S-047) | n | daemon-side ServerToClient::ScrollbackChunk* forwarding (AC-005) deferred — session-host emits empty scrollback dump; screen-content source = S-039 (PTY output pipeline) / S-047 (live parser). TODO(S-039/S-047) tracker in session_manager/mod.rs attach Step 7. Non-blocking. |
| F-S035-LAUNCHING-CONN-DETACH-MATRIX | pending | architect | n | action×state matrix Detached/Launching cell ambiguity — detach on Launching-WITH-established-host_conn: matrix says Err(SessionNotReady) if host_conn not yet active but impl LaunchingWithConn path transitions to Detached. Official TUI never reaches this path. Architect to clarify matrix wording. Non-blocking. |
| PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED | codified 2026-06-19 | devops/orchestrator | n | [process-gap] S-038 cycle: spec-bumping agents (architect/story-writer/product-owner) committed 4 commits to factory-artifacts but never pushed — origin was 4 commits stale (f8f0b38→485cfbb). CI POL-11 checks out origin/factory-artifacts at PR-open time; stale origin caused PR #44 to pass spuriously-consistent and PR #45 to fail stale-forward. Fixed by pushing 4 commits 2026-06-19. CODIFY: orchestrator MUST verify `git -C .factory log origin/factory-artifacts..HEAD` is empty after every spec-bumping dispatch; add to burst checklist. Lesson: cycles/cycle-001/lessons.md §PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED. |

## Resolved/Closed Tasks (archived)

43 entries. Full detail at `cycles/cycle-001/task-register-full.yaml`:
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
PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH (RESOLVED D-335 2026-06-19).
SEC-001-RENAME-NEWNAME-VALIDATION (RESOLVED D-335 2026-06-19 — fixed in-scope S-037 b2d65db).
SEC-002-RENAME-UUID-GUARD (RESOLVED D-335 2026-06-19 — fixed in-scope S-037 b2d65db).
F-S035-001 (RESOLVED D-336 2026-06-19 — kill-after-attach reader ownership fixed via Ruling L; proxy_task fast-path kill confirmation).

## Decision History

Full decisions archive: `cycles/cycle-001/decisions-archive.md`
D-001..D-241: early phases; D-242..D-336: Phase-1d + Phase-2 + Phase-3 (appended 2026-06-16/17/18/19)

Key decisions last session:
- D-336 (2026-06-19): S-035 MERGED to develop via PR #43 @ 270b7d4. Fourth Wave-8 story (third Tier-2). attach_session/detach_session, BC-2.08.007 v1.5.5 + BC-2.08.008 v1.3.7, 8 pts. 9-pass adversarial convergence (3 consecutive CLEAN: passes 7/8/9). CRIT-001 silent Detached→Terminated on EC-187 fixed; Ruling L (proxy_task kill-reader delegation); 4-disposition action×state matrix. SS-session-manager v2.11.0→v2.14.0. SEC-002 path-traversal class honored. S-036 UNBLOCKED. sprint-state v1.49→v1.50; STORY-INDEX v5.48→v5.49. STATE v7.99→v8.00.
- D-335 (2026-06-19): S-037 MERGED to develop via PR #42 @ a7e4081. Third Wave-8 story (second Tier-2). GC task + rename_session. 7-pass adversarial convergence (3 consecutive CLEAN: passes 5/6/7). SEC-001 (CWE-20) + SEC-002 (CWE-706) fixed in-scope (b2d65db). PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH RESOLVED. Wave 8: 3/12 done (19/74 pts). sprint-state v1.48→v1.49; STORY-INDEX v5.47→v5.48. STATE v7.98→v7.99.
- D-334 (2026-06-18): S-034 MERGED to develop via PR #41 @ 4dfe0db. Second Wave-8 story (first Tier-2). Kill path: 18-pass adversarial convergence (3 CLEAN). SS-session-manager v2.11.0, BC-2.08.003 v1.5.0, BC-2.08.008 v1.3.5. Unblocks S-037. Wave 8: 2/12 done (16/74 pts). sprint-state v1.47→v1.48; STORY-INDEX v5.46→v5.47. STATE v7.97→v7.98.
- D-333 (2026-06-17): Zero-context resume checkpoint. develop @ 314326e. S-034 worktree ready. Wave-8 Tier-2 autonomous delivery authorized. Demo: WEBM+.tape-NO-GIF. 11 CI branch-protection contexts enforced. STATE v7.95→v7.96.
- D-332 (2026-06-17): S-033 MERGED to develop via PR #40 @ c7e10f2. First Wave-8 story. 10th crate monocle-session-host. Unblocks S-034/035/037/038/045. sprint-state v1.46→v1.47; STORY-INDEX v5.45→v5.46.
- D-323..D-331 (2026-06-16/17): Phase-2 pre-gate cleanup (D-323), inputs-pin fix (D-324), Phase-2 gate APPROVED (D-325), S-033 Rulings A–H (D-326..D-329), pass 4 remediation (D-330), S-033 convergence COMPLETE 3/3 clean (D-331). Full detail in decisions-archive.md.
