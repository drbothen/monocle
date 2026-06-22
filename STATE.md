---
document_type: pipeline-state
level: ops
project: monocle
version: "8.15"
status: active
producer: state-manager
timestamp: 2026-06-22T12:00:00Z
phase: phase-3-v1A-wave-9
current_step: "D-350 (2026-06-22): ZERO-CONTEXT DURABILITY CHECKPOINT — S-047 MID-IMPLEMENTATION (NOT merged). S-047 EXPANDED 8 pts→16 pts per human ruling 2026-06-22 (full producer scope: BC-2.05.010/011, SS-05/SS-08, story v1.8). Architect design: SS-session-manager v2.18.0 §Ruling M (commit 43c89d7). Story integrated by story-writer (commit 6d9d2e7). Worktree story/S-047-ipc-lifecycle-variants @ .worktrees/S-047, based develop @ 45343ca. Commits: stub 6631aa2, tests da18a3e (22 tests: 14 RED→now green, 8 pre-existing-GREEN), impl (14/14 green). BLOCKER F-S047-DAEMON-RELAY-INERT: daemon does NOT relay real ScrollbackChunk frames to TUI client — total_chunks:0 UNCONDITIONAL stub in forward_scrollback_dump_to_client (mod.rs:4400) + proxy-task TODO at mod.rs:3914 discards HostToDaemon::ScrollbackChunk. Adversarial NOT started. 44/51 done (267/322 pts after S-047 expansion). develop HEAD: 45343ca. STATE v8.14→v8.15."
prior_step: "D-349 (2026-06-22): S-046 MERGED to develop via PR #55 @ 45343ca (squash-merge). PtyOutput Fan-out Broker — bounded INPUT channel(1024), broadcast_to_subscribers (Option A), 1-strike disconnect, pty_drop_counter, ServerToClient::PtyReset. 5 pts, EPIC-05, Wave 8. 8-pass adversarial (3 CLEAN: passes 6/7/8). Security PASS (0 crit/high). SS-ipc v1.25.0; BC-2.05.009 v1.6.0; S-046 v2.0. Wave 8: 7/12 done (43/74 pts). 44/51 stories done (267/314 pts). S-047 UNBLOCKED. develop HEAD: 45343ca. STATE v8.13→v8.14."
mode: greenfield-with-reference-ingest
input-hash: "[live-state]"
inputs: []
traces_to: "D-001..D-241 at cycles/cycle-001/decisions-archive.md. D-242..D-338 appended at cycles/cycle-001/decisions-archive.md §Phase-1d+Phase-2+Phase-3. Full durable_task_register YAML (127+ entries) at cycles/cycle-001/task-register-full.yaml."
awaiting: "D-350 S-047 IN PROGRESS (mid-implementation, worktree .worktrees/S-047, NOT merged). BLOCKER F-S047-DAEMON-RELAY-INERT must be fixed BEFORE adversarial convergence. Resume: S-047-NEXT-1 (wire daemon relay), S-047-NEXT-2 (real-relay integration test), S-047-NEXT-3 (chunk-rows alignment), then adversarial→demo→PR→checkpoint (S-047-NEXT-4). Then S-044 (13 pts, EPIC-09) after S-047. Wave-9 contract-green gate after S-044. EPIC-09 end-to-end integration gate after S-047+S-044. SEC-006-CCR-URL-VALIDATION MUST be fixed before S-045. S-048 waits on S-022+S-033+S-047. F-W8INT-001/002/003 parked for Wave-8 gate."
dtu_required: true
dtu_assessment: 2026-05-12
dtu_clones_built: 2026-06-03
dtu_services: [hook-endpoints-x5]
current_cycle: cycle-001
next_session_resume_protocol: |
  ============================================================================
  ZERO-CONTEXT RESUME CHECKPOINT v8.15 — 2026-06-22 (D-350)
  PHASE-3 v1A ACTIVE — WAVES 8+9 — S-047 IN PROGRESS (mid-impl, NOT merged)
  D-348: Wave-9 RE-SEQUENCED — PRODUCER (S-046→S-047) before CONSUMER (S-044)
  ============================================================================
  POSITION: Phase-3 TDD implementation, v1A control-center scope, Waves 8+9.
  STATUS: S-046 MERGED PR #55 @ 45343ca (D-349). develop @ 45343ca.
  44/51 stories done (267/322 pts — S-047 expanded 8→16 pts, total 314→322).
  Wave 8: 7/12 done (43/74 pts). Wave 9: 5/6 done (32/45 pts). 10 workspace crates.

  D-350 CHECKPOINT (2026-06-22):
  S-047 MID-IMPLEMENTATION — NOT merged. Expanded 8 pts→16 pts (human ruling
  2026-06-22; full producer scope: session-host producer + daemon relay + TUI
  consumers; BC-2.05.010/011; subsystems SS-05/SS-08; story v1.8).
  Architect design: SS-session-manager v2.18.0 §Ruling M (commit 43c89d7 on
  factory-artifacts). Story-writer integrated scope (commit 6d9d2e7).
  Worktree: /Users/jmagady/Dev/monocle/.worktrees/S-047
  Branch: story/S-047-ipc-lifecycle-variants (based develop @ 45343ca)
  Commits in worktree (NOT pushed to origin):
    stub 6631aa2, tests da18a3e (22 tests: 14 RED→green, 8 pre-existing-GREEN),
    implementation (14/14 target tests green). build/clippy/fmt/POL-11/POL-12 clean.

  CONFIRMED PRE-ADVERSARIAL DEFECT — MUST FIX BEFORE CONVERGENCE:
  F-S047-DAEMON-RELAY-INERT (BLOCKER): daemon does NOT relay real ScrollbackChunk
  frames to TUI client. forward_scrollback_dump_to_client (mod.rs:4400) sends
  ServerToClient::ScrollbackDumpComplete{total_chunks:0} UNCONDITIONALLY.
  Proxy task receives HostToDaemon::ScrollbackChunk (mod.rs:3585) but TODO at
  mod.rs:3914 discards chunks. Session-host correctly produces real chunks but
  daemon→client relay is UNWIRED. AC-006 + AC-SH-005 not delivered end-to-end.
  The 14 green tests cover isolated host producer + EC-306 empty-dump path only —
  no test exercises real relay. Same class as S-046-inert-broker gap.

  D-349 DECISION (2026-06-22):
  S-046 (PtyOutput Fan-out Broker) MERGED. Key architectural detail (Option A):
  broker owns INPUT channel(1024) + Arc<SubscriberList> only; fans out via
  broadcast_to_subscribers (shared SubscriberList — NOT a broker-owned client
  registry). 1-strike disconnect. pty_drop_counter (proxy-task OOM-only). biased
  select! hook priority. ServerToClient::PtyReset variant added. 8-pass adversarial
  (3 CLEAN: passes 6/7/8). CRITICAL LESSON (LESSON-S046-INERT-BROKER-UNIT-TEST-TAUTOLOGY):
  P1 found the broker was INERT in production (own client registry never populated
  → PTY bytes silently discarded). Unit tests passed against a parallel UNWIRED code
  path. Architect RE-ARCHITECTED to Option A. Codified in lessons.md.

  D-348 RULING (2026-06-22, Joshua Magady — still governs):
  - DELIVER PRODUCER FIRST: S-046 DONE → S-047 → S-044. S-044 remains after S-047.
  - Wave-9 gate = CONTRACT-GREEN (BCs green + 3 clean adversarial per story).
  - EPIC-09 end-to-end gate = SEPARATE milestone after S-047+S-044 (live vertical
    slice: enter→live PtyOutput→keyboard/mouse forwarding→exit).
  - WG-S042-SESSIONHOST-KEYINPUT CARRY-FORWARD (severity UPGRADED): main.rs:731-737
    DROPS KeyInput in catch-all. Keyboard+mouse live forwarding blocked until S-047.
  - F-S039-P9-OBS-001: MARK FOR CLOSE-AT-GATE (S-042 resize landed).

  NEXT-ACTION QUEUE — S-047 RESUME (D-350 order):
  1. S-047-NEXT-1 (implementer, BLOCKER): wire daemon scrollback relay per architect
     Ruling M — forward session-host's real ScrollbackChunk + ScrollbackDumpComplete
     frames to the REQUESTING client. Replace total_chunks:0 stub in
     forward_scrollback_dump_to_client (mod.rs:4400) AND/OR implement proxy-task
     TODO at mod.rs:3914. If proxy-task↔forward coordination is ambiguous vs
     Ruling M, route to architect first.
  2. S-047-NEXT-2 (test-writer): add real-relay integration test (session-host
     produces N>0 chunks → daemon forwards → client receives N ScrollbackChunk +
     ScrollbackDumpComplete{total_chunks:N}) that FAILS against current empty-dump
     stub. Closes coverage gap that masked F-S047-DAEMON-RELAY-INERT.
  3. S-047-NEXT-3: resolve F-S047-CHUNK-ROWS-50-VS-200 — align to ≤200 rows/chunk
     per Ruling M / story AC-SH-004, or confirm 50 is intentional.
  4. S-047-NEXT-4: adversarial convergence (3 consecutive CLEAN, fresh-context,
     whole-system trace; specifically verify daemon relay reaches real clients, like
     S-046 inert-broker check) → demo (WEBM+.tape) → pr-manager (9 steps) →
     devops cleanup + reconcile → state-manager checkpoint.

  POST-S-047 QUEUE:
  5. S-044 (EmbeddedTerminal+SessionCreation entry trigger + wizard + SpawnAck +
     BC-2.09.009 permission badge+bell, 13 pts, EPIC-09) — after S-047.
  6. Wave-9 contract-green integration gate (after S-044 merges).
  7. EPIC-09 end-to-end integration gate (after S-047+S-044; live slice:
     enter→PtyOutput→keyboard/mouse→exit). Verify:
     WG-S046-SPAWN-PROXY-NOT-STARTED (architect Ruling 5: attach-driven canonical),
     WG-S046-OOM-POSITIVE-COVERAGE, WG-S042-SESSIONHOST-KEYINPUT (closed S-047),
     WG-S043-PERMISSION-BADGE-COEXIST, full enter→live-output→keyboard/mouse→exit.
  8. SEC-006-CCR-URL-VALIDATION (CWE-20/93) MUST be fixed before S-045.
  9. S-048 (after S-022+S-033+S-047).
  [F-W8INT-001/002/003 parked for Wave-8 gate; do NOT re-raise per-story.]

  Per-story flow: stub-architect → test-writer (Red Gate) → implementer (TDD) → adversarial
  (3 consecutive CLEAN) → demo-recorder (WEBM+.tape, NO GIF) → pr-manager (9 steps) →
  state-manager checkpoint.

  HUMAN DIRECTIVE: CONTINUE AUTONOMOUSLY into subsequent ready stories until told
  to stop. Per-story flow + 3-consecutive-CLEAN adversarial convergence + clean
  merge applies to each story. Demo default: WEBM + .tape ONLY. NO GIF.
  Branch protection: 11 required CI contexts enforced (BARE check names).

  READ FIRST (in order):
  1. /Users/jmagady/Dev/monocle/NEXT-SESSION-RESUME.md  <- concise entry point
  2. /Users/jmagady/Dev/monocle/CLAUDE.md               <- production-grade + routing
  3. This STATE.md (task register table below)          <- task register + history pointers

  CORPUS FACTS (post-D-350 checkpoint):
  Stories: 51 total / 322 pts (44 done; S-047 IN PROGRESS 16 pts; 5 draft v1A; 1 blocked)
  New v1A: S-033..S-048 (16 stories); EPIC-08 Session Manager; EPIC-09 Embedded PTY
  Waves: 8 (7/12 done, 43/74 pts); 9 (5/6 done, 32/45 pts)
  S-047 expanded: 8 pts → 16 pts (total 314 → 322 pts)

  KEY VERSION PINS (canonical source: .factory/specs/version-pin-registry.yaml):
  STORY-INDEX v5.66 | sprint-state v1.59 | wave-schedule v2.1
  dependency-graph-expansion v2.10 | BC-INDEX v1.44.5 | EVAL-INDEX v1.42
  ARCH-INDEX v1.0.30 | SS-ipc v1.25.0 | SS-session-manager v2.18.0
  SS-embedded-pty v1.17.0 | SS-config v1.4.0 | SS-engine-module-v2-delta v1.6.0
  SS-daemon-wiring-v2-delta v1.12.0 | SS-deps-pin-manifest-v2-delta v1.0.2
  prd v1.28.3 | product-brief v2.0.4 | domain-monocle-vision-synthesis v2.2.3
  BC-2.05.009 v1.6.0 | BC-2.09.001 v1.7.3 | BC-2.09.002 v1.2.2 | BC-2.09.003 v1.6.1
  BC-2.09.004 v1.0.11 | BC-2.09.005 v1.0.7 | BC-2.09.006 v1.3.0 | BC-2.09.007 v1.5.1
  BC-2.07.002 v1.1.0 | S-039 v1.8 | S-040 v1.8 | S-041 story v1.3 | S-042 story v1.5
  S-043 story v1.5 | S-046 story v2.0 | S-047 story v1.8 | BC-2.05.010 (S-047)
  BC-2.08.002 v1.2.6 | BC-2.08.004 v1.4.0 | BC-2.08.006 v1.5.0
  (Pull the rest from .factory/specs/version-pin-registry.yaml)

  PER-STORY FLOW (verbatim — follow exactly):
  stub-architect → test-writer (Red Gate, all fail) → implementer (TDD green) →
  Step 4.5 adversarial convergence (3 CONSECUTIVE CLEAN, fresh-context each
  pass; route findings to correct specialist; architect adjudicates cross-
  component design as SPEC-ONLY) → demo-recorder (WEBM+.tape, NO GIF,
  docs/demo-evidence/S-NNN/) → push → pr-manager full lifecycle (push→PR→
  security-review→pr-reviewer→11 CI→clean merge→delete branch) →
  orchestrator verifies merge via gh pr view → devops worktree cleanup +
  reconcile main-checkout develop to origin (ff; if a human direct-commit
  diverged develop, ASK human) → state-manager checkpoint (commit + PUSH
  factory-artifacts; verify origin in sync).

  CI-PARITY RULES (all 8 apply):
  1. clippy --workspace --all-targets -- -D warnings (IN worktree)
  2. python3 scripts/check_version_pins.py (POL-11) from REPO ROOT
     python3 scripts/check_structural_claims.py (POL-12) from REPO ROOT
  3. No version literals in doc-comments / test prose (POL-11 will flag);
     historical snapshots use <!-- version-pin-historical: ... --> HTML comment.
  4. Registry atomicity (L-S027-004): BC/SS spec version bump +
     version-pin-registry.yaml update = ONE atomic factory-artifacts commit;
     cascade STORY-INDEX/EVAL-INDEX/dependency-graph pins in same commit.
  5. Unique /tmp paths per story dispatch (prevents commit-message mixup).
  6. Architect = spec only; all code to implementer-in-worktree.
  7. pr-manager must complete ALL 9 steps; orchestrator verifies merge via
     gh pr view before declaring done.
  8. (PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED, D-337): after every
     spec-bumping agent dispatch, verify `git -C .factory log
     origin/factory-artifacts..HEAD` is EMPTY and push immediately if not.
  B002 BUILD ORDER: 2 B002 integration tests require `cargo build --workspace`
  first (monocle-session-host binary needed); PASS in CI; fail bare test locally.

  RATIFIED DECISIONS (do NOT re-litigate):
  - D-238: session-host-owns-PTY; daemon restart SURVIVES (CASE 2); NO tmux default
  - D-304: Autonomous Phase-2 dispatch; no per-burst plan-review gate
  - D-315: Pre-pivot disposition RATIFIED (32 done; 3 active kept; 0 archive)
  - D-325: Phase-2 gate APPROVED; Phase-3 v1A active
  - D-332..D-339: S-033..S-036 MERGED; Wave-8 Tier-1/2/3 complete; all decisions archived
  - D-340: S-039 MERGED PR #47 @ a7ad00e. PTY output pipeline. <!-- version-pin-historical: BC-2.09.001 v1.7.2 at S-039 merge (D-340) -->
           10-pass adversarial (3 CLEAN: 8/9/10). Admin bypass (DTU deadlock).
  - D-341: PROCESS-GAP-DTU-FIDELITY-PATH-FILTER-DEADLOCK RESOLVED. PR #48 @
           3eba172 merged CLEAN. dtu-fidelity.yml always-report pattern.
  - D-342: S-040 MERGED PR #50 @ d230a26 (squash-merge 2026-06-21). Full-Fidelity
           Keyboard Forwarding. 17-pass adversarial (3 CLEAN: 15/16/17). 65 tests.
  - D-344: Human ruling — S-042 expanded. BC-2.09.006 v1.3.0, S-042 v1.5 (8 pts).
  - D-345: S-042 MERGED PR #51 @ 2f01de0. Full end-to-end PTY Resize. 8 pts. 9-pass.
  - D-346: S-043 MERGED PR #53 @ 5e6a2e0. Scrollback Navigation. 3 pts. 8-pass.
  - D-347: S-041 MERGED PR #54 @ 58fbd617. Mouse Forwarding. 5 pts. 14-pass.
  - D-348: Wave-9 RE-SEQUENCED. S-046→S-047→S-044 (producer before consumer).
           WG-S042-SESSIONHOST-KEYINPUT UPGRADED. F-S039-P9-OBS-001 CLOSE-AT-GATE.
  - D-349: S-046 MERGED PR #55 @ 45343ca. PtyOutput Fan-out Broker. 5 pts. 8-pass.
           Wave 8: 7/12 done (43/74 pts). 44/51 done (267/314 pts). S-047 UNBLOCKED.
           develop HEAD: 45343ca. STATE v8.13→v8.14.
  - Spawn-path Model A: SpawnOptions on wire; SpawnRecipe daemon-internal
  - IPC: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
  - PTY (ADR-0011): portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
  - SessionState: 5 variants (Launching/Running/Detached/Terminating/Terminated)
  - Full history: cycles/cycle-001/decisions-archive.md (D-001..D-349)

  OPEN NON-BLOCKING DURABLE FOLLOW-UPS (pointer — full register in STATE.md
  durable_task_register + cycles/cycle-001/task-register-full.yaml):
  SEC-001-S040-LOG-BEFORE-VALIDATE (CWE-20 LOW, maintenance sweep);
  SEC-003-S040-OVERSIZE-PASTE-UX (CWE-400 LOW, UX follow-up);
  F-S040-KEYINPUT-DOC-EMPTYBYTES (LOW nit);
  F-S039-P9-OBS-001 (CLOSE-AT-GATE — S-042 resize landed, D-348);
  SEC-006-CCR-URL-VALIDATION (before S-045); F-S038-EXIT72-ENFORCEMENT;
  F-S038-INV6-PROD-CANON-TEST; F-S038-TRACING-TEST-DOC-DEVERSION;
  F-S035-AC005-DAEMON-BROADCAST (S-047 owner); F-S035-LAUNCHING-CONN-DETACH-MATRIX;
  DEMO-BINARY-ARTIFACTS-DEVELOP (7+ stories' WEBM — repo-hygiene pending);
  F-W8INT-001/002/003 (Wave-8 gate); PROCESS-GAP-STUB-PHASE-DOCCOMMENTS (lessons.md);
  F-S043-SEC-DUMPTIMEOUT-UUID-VALIDATE (LOW); F-S043-PRREV-SUGGESTIONS (LOW);
  F-S043-RENDER-OFFSET-WRITEBACK (LOW/benign); WG-S043-PERMISSION-BADGE-COEXIST (Wave-9 gate);
  F-S041-SEC-PANIC-HOOK-STDOUT-LOCK (MED, hardening sweep); F-S041-SEC-LOW (4×LOW bundle);
  F-S041-CAPTURE-LIFECYCLE-WRITESEAM (process-gap, write seam);
  WG-S041-EMBEDDED-ENTRY-TRIGGER (Wave-9 gate);
  WG-S042-SESSIONHOST-KEYINPUT (UPGRADED CARRY-FORWARD — S-047; keyboard+mouse blocked);
  WG-S046-SPAWN-PROXY-NOT-STARTED (EPIC-09 integration gate; anchor: S-047);
  WG-S046-OOM-POSITIVE-COVERAGE (EPIC-09 integration gate; anchor: test-writer);
  F-S046-PRREV-SUGGESTIONS (3 non-blocking suggestions from PR #55; LOW bundle);
  F-S046-SEC-PREEXISTING (SEC-001/003 + MEDIUMs pre-existing; maintenance sweep);
  WG-S046-SERVERTOCLIENT-NONEXHAUSTIVE (cross-story / PO; wave-gate reconcile).

  KNOWN-FLAKY TESTS (do NOT flag as new findings):
  cli_daemon_stop, factory_self_referential, test_BC_2_07_006,
  wit-bindgen unmatched-skip, PATH isolation flake.

  factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'`
  develop HEAD: 45343ca (S-046 PR #55; 58fbd617 = S-041 PR #54; 5e6a2e0 = S-043 PR #53)
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
| 3 TDD v1A Waves 8-9 | IN PROGRESS — Waves 8+9 | S-033..S-036+S-038 MERGED. S-046 MERGED PR #55 @ 45343ca (D-349, Wave 8: 7/12 done, 43/74 pts). S-039 MERGED PR #47 (D-340). S-040 MERGED PR #50 (D-342). S-041 MERGED PR #54 (D-347). S-042 MERGED PR #51 (D-345). S-043 MERGED PR #53 (D-346). Wave 9: 5/6 done (32/45 pts). 44/51 done (267/322 pts — S-047 expanded 8→16 pts). **S-047 IN PROGRESS** (D-350, mid-impl; BLOCKER F-S047-DAEMON-RELAY-INERT; adversarial NOT started). |
| 3 TDD Waves 1-7 (pre-pivot) | COMPLETE D-232 | 32/33 done (192/195 pts). 1514 tests. develop @ 6811103 |
| 4-7 | PENDING after Phase-3 v1A | Old observe-only scope superseded by v1A control-center |

## Blocking Issues

None. All durable task register items are non-blocking.

## Durable Task Register

112 active tasks. Full YAML detail (all 149+ entries including 37 resolved/closed): `cycles/cycle-001/task-register-full.yaml`.

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
| WG-S036-001 | pending | wave-gate/architect | n | [wave-gate] BC-2.08.002 PC-7 scrollback availability after re-discovery — re-discovery registers Running with reader:None/proxy_task:None; live scrollback/PTY streaming depends on S-035 AttachSession + S-039/S-047 PTY pipeline. Cross-story integration. Surface at Wave-8 integration gate. NON-BLOCKING. |
| WG-S036-002 | pending | wave-gate/architect | n | [wave-gate] re-discovered Running entry (reader:None/proxy_task:None) + subsequent kill_session falls to 12s-watchdog pre-Running-race branch rather than fast-path Terminated confirmation. Intended S-039/S-047 boundary. Surface at Wave-8 integration gate. NON-BLOCKING. |
| F-S036-PRREV-SUGGESTIONS | pending | implementer | n | [LOW post-convergence] 4 non-blocking pr-reviewer suggestions: (a) further code-duplication cleanup in GC branches, (b) lifecycle step-8b comment nit, (c) orphan watchdog task tracking/handle, (d) PR-description filename. Bundle for a future hardening pass. |
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
| PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP | human-remediation | human | n | [process-gap][D-348 RULING] HUMAN-OWNED MANUAL REMEDIATION. Joshua Magady will implement structural guard manually outside pipeline. Orchestrator continues to catch+remediate stray architect-on-develop commits in interim. DO NOT open a self-improvement story. Fourth recurrence: S-035/S-036/S-042/S-041 cycles. Full history: lessons.md. |
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
| F-S038-EXIT72-ENFORCEMENT | pending | daemon-mode/phase-5 story | n | BC-2.08.006 Inv5/EC-183 mandate daemon exit 72 on hooks-settings write failure; DaemonExit taxonomy has no code-72 variant; TUI auto-start logs+continues. PRE-EXISTING/system-level; deferred to phase-5 or dedicated story. |
| F-S038-INV6-PROD-CANON-TEST | pending | test-writer/follow-up | n | No integration test asserts BC-2.08.006 Invariant 6 production canonicalization (lifecycle writes to non-canonical runtime_dir.join while SessionManager stores canonical). Symlinked-runtime_dir integration test missing. Non-blocking. |
| F-S038-TRACING-TEST-DOC-DEVERSION | pending | implementer | n | [LOW] doc-comment at session_manager/mod.rs ~line 11621 embeds tracing-test version literal as prose. De-version in future pass. |
| BURST-GAP-001-S038-HOOK-SCHEMA | CLOSED D-338 | story-writer | n | S-038 hook schema gap — RESOLVED in F-S038-AC004-SCHEMA burst (AC-004 schema array-of-hook-objects fix, S-038 v1.5) |
| BURST-GAP-002-S033-SESSIONNOTREADY-VARIANT | pending | story-writer | n | S-033 SessionNotReady variant gap |
| BURST-GAP-003-S043-TUITERM-SCROLLBACK-API | CLOSED D-346 | story-writer | n | S-043 tui-term scrollback API gap — RESOLVED (S-043 v1.1+ pinned the verified set_scrollback API; implemented & merged) |
| BURST-GAP-004-S043-S042-CROSSSTORY-RESET | CLOSED D-346 | story-writer | n | S-043/S-042 cross-story reset gap — RESOLVED (S-043 spec made S-042 ownership explicit; AC-009 asserted & verified in merge) |
| BURST-GAP-005-S039-S043-CROSSSTORY-CONFIG | CLOSED D-346 | story-writer | n | S-039/S-043 cross-story config gap — RESOLVED (S-043 spec made S-039 ownership explicit; verified) |
| F-S043-SEC-DUMPTIMEOUT-UUID-VALIDATE | pending | implementer/maintenance | n | [LOW, non-blocking] on_dump_window_timeout uses session_id in format!/tracing::warn! without Uuid::parse_str re-validation (pre-existing, S-039-owned). Add guard matching on_pty_output/on_scrollback_dump_complete pattern. Anchor: maintenance-sweep wave. |
| F-S043-PRREV-SUGGESTIONS | pending | implementer | n | [LOW, non-blocking] 3 pr-reviewer suggestions from S-043: (a) trace log when max_available==0; (b) defensive dispatch-arm documentation; (c) render_embedded_terminal mutation note. Bundle for hardening pass. |
| F-S043-RENDER-OFFSET-WRITEBACK | pending | implementer | n | [LOW/benign, OBS-P7-001] render reads pty_scroll_offsets and clamps via set_scrollback but does not write clamped value back; benign (badge uses effective offset; divergence only via S-039 parser-reset paths). Non-blocking. |
| WG-S043-PERMISSION-BADGE-COEXIST | pending | wave-gate/architect | n | [wave-gate, cross-story] AC-007 lists [N pending permission(s)] as a must-coexist badge, but that badge is owned by BC-2.09.009 (unimplemented project-wide). Verify scrollback indicator coexists with permission badge once BC-2.09.009 ships. Anchor: Wave-9 integration gate / BC-2.09.009. |
| F-S041-SEC-PANIC-HOOK-STDOUT-LOCK | pending | implementer/security | n | [S-041, MED, non-blocking] panic hook acquires stdout lock during terminal teardown — review for re-entrancy/deadlock safety in a future hardening pass. Security review non-blocking per PR #54. Anchor: hardening sweep. |
| F-S041-SEC-LOW | pending | implementer | n | [S-041, 4×LOW, informational] four LOW security informational items from PR #54 security review; bundled for future hardening pass. |
| F-S041-CAPTURE-LIFECYCLE-WRITESEAM | pending | architect/implementer | n | [S-041, process-gap, test-coverage] scoped mouse-capture enter/exit + panic teardown byte ORDERING cannot be asserted in tests — helpers write directly to stdout with no mock-write seam. Recommend &mut dyn Write seam for positive coverage of ordering + teardown-on-departure invariant. Anchor: future hardening story / S-044+ when EmbeddedTerminal entry trigger lands. |
| WG-S041-EMBEDDED-ENTRY-TRIGGER | pending | wave-gate/implementer | n | [wave-gate, cross-story] EmbeddedTerminal entry trigger (user action entering the mode) is owned by S-044; mode is not reachable in production yet. Verify full enter→mouse-forward→exit user flow end-to-end once S-044 wires the trigger. Anchor: Wave-9 integration gate. |
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
| DEMO-BINARY-ARTIFACTS-DEVELOP | pending | devops/human-decision | n | ~6 MB WEBM added by S-038 (docs/demo-evidence/S-038/) + prior S-033/S-034/S-037/S-035. Now 5 stories' demo binaries on develop. Repo-hygiene policy decision still pending. |
| F-S035-001 | RESOLVED D-336 | architect/implementer | n | kill-after-attach reader ownership → RESOLVED via SS-session-manager Ruling L (proxy_task fast-path kill confirmation; kill reader=None+proxy=Some delegation). Fixed in-scope. |
| F-S035-AC005-DAEMON-BROADCAST | pending | story-writer (S-039/S-047) | n | daemon-side ServerToClient::ScrollbackChunk* forwarding (AC-005) deferred — session-host emits empty scrollback dump; screen-content source = S-039 (PTY output pipeline) / S-047 (live parser). TODO(S-039/S-047) tracker in session_manager/mod.rs attach Step 7. Non-blocking. |
| F-S035-LAUNCHING-CONN-DETACH-MATRIX | pending | architect | n | action×state matrix Detached/Launching cell ambiguity — detach on Launching-WITH-established-host_conn: matrix says Err(SessionNotReady) if host_conn not yet active but impl LaunchingWithConn path transitions to Detached. Official TUI never reaches this path. Architect to clarify matrix wording. Non-blocking. |
| PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED | codified 2026-06-19 | devops/orchestrator | n | [process-gap] S-038 cycle: spec-bumping agents (architect/story-writer/product-owner) committed 4 commits to factory-artifacts but never pushed — origin was 4 commits stale (f8f0b38→485cfbb). CI POL-11 checks out origin/factory-artifacts at PR-open time; stale origin caused PR #44 to pass spuriously-consistent and PR #45 to fail stale-forward. Fixed by pushing 4 commits 2026-06-19. CODIFY: orchestrator MUST verify `git -C .factory log origin/factory-artifacts..HEAD` is empty after every spec-bumping dispatch; add to burst checklist. Lesson: cycles/cycle-001/lessons.md §PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED. |
| PROCESS-GAP-DTU-FIDELITY-PATH-FILTER-DEADLOCK | RESOLVED D-341 | devops | n | [process-gap] RESOLVED: PR #48 @ 3eba172 merged CLEAN to develop (no admin bypass). dtu-fidelity.yml: removed top-level paths-filter; added internal pure-bash change-detection gate; skip path exits 0 (always-report); oracle runs when DTU-relevant paths changed; job name byte-identical. S-040/S-042/S-043 now merge without admin bypass. Original: develop branch protection required 'DTU fidelity oracle' but paths-filter caused TUI-only PRs to never trigger workflow (PR #47/S-039 required admin bypass). |
| F-S039-P9-OBS-001 | pending-close-at-gate | wave-gate | n | [wave-gate, MARK-FOR-CLOSE-AT-GATE — D-348] S-039 placeholder pane resize→parser reconciliation was S-042/BC-2.09.006 scope. S-042 resize reconciliation LANDED (PR #51 @ 2f01de0, D-345). Verify closed at Wave-9/EPIC-09 integration gate. NON-BLOCKING. |
| SEC-001-S040-LOG-BEFORE-VALIDATE | pending | devops/maintenance-sweep | n | [SEC S-040] (CWE-20, LOW, non-blocking) ipc_server.rs handle_key_input logs unvalidated session_id before UUID parse check; defense-in-depth, path-traversal already mitigated downstream; deferred to maintenance sweep. Anchor: maintenance-sweep wave. |
| SEC-003-S040-OVERSIZE-PASTE-UX | pending | implementer/UX-follow-up | n | [SEC S-040] (CWE-400, LOW, non-blocking) oversized-paste drop-with-WARN has no TUI status-bar notification; security adequate (drop is correct), UX improvement pending. Anchor: future UX hardening pass. |
| F-S040-KEYINPUT-DOC-EMPTYBYTES | pending | implementer/tech-writer | n | [LOW nit] monocle-ipc types.rs KeyInput doc-comment silent on empty-bytes semantics (what does an empty Vec<u8> mean?); optional polish. Anchor: future maintenance pass. |
| PROCESS-GAP-STUB-PHASE-DOCCOMMENTS | codification-pending | devops/session-reviewer | n | [process-gap] stub-architect emits module/test doc-comments with stub-phase language ("stubs", "All function bodies are todo!()", "Tests MUST fail (Red Gate)") that are NOT refreshed when the implementer fills them in; adversarial passes 10/12/13/14/17 of S-040 each caught a different stale doc-comment, costing ~5 extra convergence passes. Fix options: (a) stub-architect writes neutral/forward-looking module docs; (b) implementer self-audit step must refresh all stub-phase doc-comments. Lesson recorded at cycles/cycle-001/lessons.md §PROCESS-GAP-STUB-PHASE-DOCCOMMENTS. |
| WAVE-GATE-IPC-WRITER-ERROR-TAXONOMY | SUPERSEDED D-344/D-345 | wave-gate/architect | n | [wave-gate] RESOLVED: S-042 ResizePane handler implemented with WARN-drop carve-out (per D-344 ruling); full 12-code wire taxonomy alignment is satisfied for the resize path. No remaining action. |
| F-S042-OBS-WRITEFRAMED-DOCNAMING | pending | tech-writer/implementer | n | [LOW, doc-only] resize_session doc-comment + S-042 AC-015 cite `write_framed_to_stream` while impl uses byte-identical inline framing; rename/clarify in a future doc pass. Non-blocking. |
| F-S042-OBS-LASTPANEAREA-DOC | pending | implementer | n | [LOW, doc-only] `App::last_pty_pane_area` doc claims "None when not EmbeddedTerminal" but field is never reset on mode exit (benign — mode guard prevents stale read). Optional polish. Non-blocking. |
| WG-S042-SESSIONHOST-KEYINPUT | pending | wave-gate/S-047 | n | [wave-gate, CARRY-FORWARD, severity UPGRADED — D-348] session-host main.rs:731-737 DROPS DaemonToHost::KeyInput in catch-all arm. Keyboard AND mouse forwarding produce no live effect until S-047 implements the session-host KeyInput→PTY-stdin arm. Owner: S-047 (IPC lifecycle + session-host producer half). Surface at Wave-9 gate AND EPIC-09 end-to-end gate. BLOCKING end-to-end keyboard+mouse even though per-story BCs are contract-green. |
| PROCESS-GAP-STORYWRITER-VERSION-LITERALS-IN-TRACE | note | devops/story-writer | n | [process-gap, recurrence-watch, 2026-06-22] story-writer (bea02a3) wrote "vX.Y→vX.Y" version-literal deltas in S-046 Trace row and version-pin-registry.yaml last_bump_commit field — POL-11 flagged as live-citation violations (exit non-zero). story-writer should use version-free phrasing in Trace/audit changelog prose (e.g., "bumped to canonical; deltas archived") rather than explicit "vX.Y.Z→vX.Y.Z" pairs in those fields. If historical deltas must be preserved, use HTML comment <!-- version-pin-historical: ... -->. Lesson at cycles/cycle-001/lessons.md §PROCESS-GAP-STORYWRITER-VERSION-LITERALS-IN-TRACE. Non-blocking. |
| PROCESS-GAP-IMPLEMENTER-COMMITS-FACTORY-ARTIFACTS | note | devops/process | n | [process-gap, recurrence-watch, 2026-06-22] implementer committed to factory-artifacts (94035ff) to fix the above POL-11 failure — a routing overstep (spec governance belongs to state-manager/story-writer/PO/architect, not implementer). Fix content was correct and state-manager adopted it (verified POL-11 exit 0; pushed to origin). Correct path: implementer surfaces POL-11 failure to orchestrator → orchestrator routes to state-manager or story-writer → they fix and push. Non-blocking. Human owns structural process-gap fixes per D-348. Lesson at cycles/cycle-001/lessons.md §PROCESS-GAP-IMPLEMENTER-COMMITS-FACTORY-ARTIFACTS. |
| WG-S046-SPAWN-PROXY-NOT-STARTED | pending | wave-gate/architect | n | [S-046, integration, EPIC-09 gate] PTY proxy task (HostToDaemon::PtyBytes → broker INPUT) starts ONLY on AttachSession, NOT on fresh spawn (post_spawn_monitor stores reader but starts no proxy) — pre-existing S-035/S-039 design. Live streaming for a fresh session awaits S-047 (and/or spawn auto-attach). Verify end-to-end at the EPIC-09 integration gate. Anchor: S-047 / EPIC-09 integration gate. Non-blocking. |
| WG-S046-OOM-POSITIVE-COVERAGE | pending | test-writer/wave-gate | n | [S-046, integration, test coverage] OOM-positive path (proxy tx.send error → drop-counter++ + PtyReset broadcast) and the proxy positive HostToDaemon::PtyReset broadcast have NO positive unit coverage (integration-level). Add session_manager integration test at the EPIC-09 gate. Anchor: EPIC-09 integration gate. Non-blocking. |
| F-S046-PRREV-SUGGESTIONS | pending | implementer | n | [S-046, LOW, non-blocking] 3 non-blocking pr-reviewer suggestions from PR #55. Bundle for a future hardening pass. Anchor: hardening sweep. |
| F-S046-SEC-PREEXISTING | pending | security/maintenance | n | [S-046, pre-existing] SEC-001/SEC-003 + remaining MEDIUM security findings from PR #55 review are pre-existing in unchanged code (not introduced by S-046). Record for a maintenance/security sweep. Reference: PR #55 security review. Anchor: maintenance-sweep wave. |
| WG-S046-SERVERTOCLIENT-NONEXHAUSTIVE | pending | prod-owner/wave-gate | n | [S-046, cross-story, wave-gate] ServerToClient is not #[non_exhaustive] vs BC-2.05.011 Invariant 1 — pre-existing enum decision. Reconcile at Wave-9 gate (PO). Anchor: Wave-9 gate / BC-2.05.011. Non-blocking. |
| F-S047-DAEMON-RELAY-INERT | pending-BLOCKER | implementer (S-047-NEXT-1) | n | **[S-047, BLOCKER, pre-adversarial]** daemon does NOT relay real ScrollbackChunk frames to TUI client. forward_scrollback_dump_to_client (crates/monocle-runtime/src/session_manager/mod.rs:4400) sends ServerToClient::ScrollbackDumpComplete{total_chunks:0} UNCONDITIONALLY (empty dump). Proxy task receives HostToDaemon::ScrollbackChunk (mod.rs:3585) but TODO at mod.rs:3914 discards them — host chunks DISCARDED. Session-host correctly PRODUCES real chunks (stream_scrollback_dump_chunked) but daemon→client relay is UNWIRED → AC-006 (AttachSession scrollback sequence) and AC-SH-005 (daemon forwards host chunks to attaching client) NOT delivered end-to-end. 14 green tests cover isolated host producer + EC-306 empty-dump path only — NO test exercises real relay. Same class as S-046-inert-broker integration gap. MUST be fixed before adversarial convergence. Anchor: S-047-NEXT-1 (implementer) + S-047-NEXT-2 (test-writer). |
| F-S047-CHUNK-ROWS-50-VS-200 | pending-LOW | implementer (S-047-NEXT-3) | n | [S-047, LOW] implementer used SCROLLBACK_CHUNK_MAX_ROWS=50; architect Ruling M / story AC-SH-004 specify ≤200 rows/chunk. Value is within spec but deviates from stated maximum — verify intent or align. Anchor: S-047-NEXT-3. |
| S-047-NEXT-1 | pending | implementer | n | [S-047 resume step 1, BLOCKER] Wire daemon scrollback relay per architect Ruling M: forward session-host's real ScrollbackChunk + ScrollbackDumpComplete frames to the REQUESTING client. Replace total_chunks:0 stub in forward_scrollback_dump_to_client (mod.rs:4400) AND/OR implement proxy-task TODO at mod.rs:3914. Coordinate forward_scrollback_dump_to_client with proxy task that reads HostToDaemon::ScrollbackChunk (mod.rs:3585). If proxy-task↔forward coordination is ambiguous vs Ruling M, route to architect first. Precondition for adversarial. |
| S-047-NEXT-2 | pending | test-writer | n | [S-047 resume step 2] Add real-relay integration test: session-host produces N>0 chunks → daemon forwards → client receives N ScrollbackChunk + ScrollbackDumpComplete{total_chunks:N}. Test MUST FAIL against current empty-dump stub (Red Gate discipline). Closes coverage gap that masked F-S047-DAEMON-RELAY-INERT. Run after S-047-NEXT-1. |
| S-047-NEXT-3 | pending | implementer | n | [S-047 resume step 3] Resolve F-S047-CHUNK-ROWS-50-VS-200: align SCROLLBACK_CHUNK_MAX_ROWS to ≤200 per Ruling M / AC-SH-004, or confirm 50 is intentional and update spec/story. |
| S-047-NEXT-4 | pending | orchestrator | n | [S-047 resume step 4] After NEXT-1/2/3 complete: adversarial convergence (3 consecutive CLEAN, fresh-context, whole-system trace — specifically verify daemon relay reaches real clients) → demo-recorder (WEBM+.tape) → pr-manager (9 steps) → devops worktree cleanup + reconcile → state-manager checkpoint. |

## Resolved/Closed Tasks (archived)

54 entries. Full detail at `cycles/cycle-001/task-register-full.yaml`:
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
SEC-001-S038-HOOKS-PERMS (RESOLVED D-338 2026-06-19 — CWE-732 perms-before-persist; fixed in-scope S-038 commit daeb4f2).
SEC-002-S038-HOOKS-DEBUG (RESOLVED D-338 2026-06-19 — CWE-532 redacting Debug impl + leak test; fixed in-scope S-038 commit daeb4f2).
F-S038-CONV-001 (RESOLVED D-338 2026-06-19 — BC-2.04.010 PC-3 missing lock.app; fixed via BC-2.04.010→v1.4.0).
BURST-GAP-001-S038-HOOK-SCHEMA (RESOLVED D-338 2026-06-19 — AC-004 schema fix in S-038 v1.5 burst; CLOSED).
SEC-001-S036 (RESOLVED D-339 2026-06-20 — CWE-284 pid=0 guards; fixed in-scope PR #46 security review).
SEC-002-S036 (RESOLVED D-339 2026-06-20 — CWE-284 SO_PEERCRED pid=None bypass; fixed in-scope PR #46 security review).
SEC-003-S036 (RESOLVED D-339 2026-06-20 — CWE-22 socket_path traversal; fixed in-scope PR #46 security review).
PROCESS-GAP-DTU-FIDELITY-PATH-FILTER-DEADLOCK (RESOLVED D-341 2026-06-20 — PR #48 @ 3eba172 merged CLEAN; dtu-fidelity.yml always-report pattern with internal pure-bash change-detection; TUI-only PRs now merge without admin bypass).
SEC-001-S042-CWE-749 (RESOLVED D-345 2026-06-22 — #[cfg(test)] gate on handle_resize_pane_pub seam; fixed in-scope S-042 commits 4865ffe + 9cf2482).
SEC-002-S042-CWE-190 (RESOLVED D-345 2026-06-22 — checked u32 cast; fixed in-scope S-042 commits 4865ffe + 9cf2482).
SEC-003-S042-CWE-532 (RESOLVED D-345 2026-06-22 — UUID guard before log emit; fixed in-scope S-042 commits 4865ffe + 9cf2482).
WAVE-GATE-IPC-WRITER-ERROR-TAXONOMY (SUPERSEDED D-344/D-345 — ResizePane WARN-drop carve-out implemented; no remaining action).
BURST-GAP-003-S043-TUITERM-SCROLLBACK-API (RESOLVED D-346 2026-06-22 — S-043 v1.1+ pinned the verified set_scrollback API; implemented & merged).
BURST-GAP-004-S043-S042-CROSSSTORY-RESET (RESOLVED D-346 2026-06-22 — S-043 spec made S-042 ownership explicit; AC-009 asserted & verified in merge).
BURST-GAP-005-S039-S043-CROSSSTORY-CONFIG (RESOLVED D-346 2026-06-22 — S-043 spec made S-039 ownership explicit; verified).

## Decision History

Full decisions archive: `cycles/cycle-001/decisions-archive.md`
D-001..D-241: early phases; D-242..D-350: Phase-1d + Phase-2 + Phase-3 (appended 2026-06-16/17/18/19/20/21/22)

Key decisions last session:
- D-350 (2026-06-22): ZERO-CONTEXT DURABILITY CHECKPOINT — S-047 MID-IMPLEMENTATION. S-047 EXPANDED 8 pts→16 pts per human ruling 2026-06-22 (full producer scope: session-host producer + daemon relay + TUI consumers; BC-2.05.010/011; subsystems SS-05/SS-08; story v1.8). Architect design: SS-session-manager v2.18.0 §Ruling M (commit 43c89d7). Story-writer integrated scope (commit 6d9d2e7). Worktree .worktrees/S-047 on story/S-047-ipc-lifecycle-variants (based develop @ 45343ca). Commits: stub 6631aa2, tests da18a3e (22 tests: 14 RED→green, 8 pre-existing-GREEN), implementation (14/14 green). build/clippy/fmt/POL-11/POL-12 clean. CONFIRMED BLOCKER before adversarial: F-S047-DAEMON-RELAY-INERT — daemon does NOT relay real ScrollbackChunk frames to TUI client; forward_scrollback_dump_to_client (mod.rs:4400) sends total_chunks:0 unconditionally; proxy-task TODO at mod.rs:3914 discards HostToDaemon::ScrollbackChunk; AC-006/AC-SH-005 undelivered end-to-end; S-046-inert-broker class gap. LOW: F-S047-CHUNK-ROWS-50-VS-200 (SCROLLBACK_CHUNK_MAX_ROWS=50 vs spec ≤200). Resume next-steps registered: S-047-NEXT-1 (daemon relay wiring), S-047-NEXT-2 (real-relay integration test), S-047-NEXT-3 (chunk-rows alignment), S-047-NEXT-4 (adversarial→demo→PR→checkpoint). 44/51 done (267/322 pts — total 314→322 after expansion). develop HEAD: 45343ca. STATE v8.14→v8.15.
- D-349 (2026-06-22): S-046 MERGED to develop via PR #55 @ 45343ca (squash-merge 2026-06-22). PtyOutput Fan-out Broker — bounded INPUT channel(1024) + .send().await backpressure; fans out via Arc<SubscriberList>/broadcast_to_subscribers (Option A — broker owns NO client registry); 1-strike disconnect; pty_drop_counter (proxy-task OOM-only); ServerToClient::PtyReset variant added (S-046 owns it); biased select! hook priority. 5 pts, EPIC-05, Wave 8. 8-pass adversarial convergence (3 CLEAN: passes 6/7/8). P1 found broker was INERT in production (own client registry never populated → PTY bytes silently discarded). Architect RE-ARCHITECTED to Option A (fan out via shared SubscriberList) → fixed & verified. Security PASS (0 crit/high). 41-citation SS-ipc de-version cascade permanently broke the SS-ipc version-pin treadmill (POL-11 now exit 0). Spec: SS-ipc v1.25.0, BC-2.05.009 v1.6.0, BC-2.05.011 (PtyReset variant), S-046 v2.0. New follow-ups: WG-S046-SPAWN-PROXY-NOT-STARTED, WG-S046-OOM-POSITIVE-COVERAGE, F-S046-PRREV-SUGGESTIONS, F-S046-SEC-PREEXISTING, WG-S046-SERVERTOCLIENT-NONEXHAUSTIVE. LESSON-S046-INERT-BROKER-UNIT-TEST-TAUTOLOGY recorded in lessons.md. sprint-state v1.58→v1.59; STORY-INDEX v5.65→v5.66. Wave 8: 7/12 done (43/74 pts). 44/51 done (267/314 pts). develop HEAD: 45343ca. STATE v8.13→v8.14.
- D-348 (2026-06-22): GOVERNANCE CHECKPOINT — Wave-9 re-sequencing + gate semantics (Joshua Magady ruling). (1) NEW DELIVERY ORDER: S-046→S-047→S-044 (PRODUCER before CONSUMER). S-044 deferred behind S-046+S-047 so that when EmbeddedTerminal entry trigger wires, the live vertical slice (PtyOutput + keyboard/mouse forwarding) is already functional. (2) Wave-9 gate = CONTRACT-GREEN (per-story BCs + 3 clean adversarial). EPIC-09 end-to-end gate = SEPARATE milestone post-S-047+S-044 (enter→live PtyOutput→keyboard/mouse→exit). (3) WG-S042-SESSIONHOST-KEYINPUT re-framed from informational to CARRY-FORWARD, severity UPGRADED; main.rs:731-737 DROPS KeyInput in catch-all; keyboard+mouse live forwarding blocked until S-047. WG-S036-001/002 + F-S035-AC005-DAEMON-BROADCAST anchored S-047 confirmed. F-S039-P9-OBS-001 marked for close-at-gate (S-042 resize landed). (4) PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP → human-owned manual remediation; no self-improvement story. S-046 UNBLOCKED (deps S-021+S-032 done). STATE v8.12→v8.13.
- D-347 (2026-06-22): S-041 MERGED to develop via PR #54 @ 58fbd617 (squash-merge). Mouse Forwarding — pure mouse_event_to_pty_bytes SGR 1006 encoder (full Ps table, modifier bits, 1-indexed pane-relative coords, out-of-pane→None), scoped EnableMouseCapture lifecycle (capture active IFF EmbeddedTerminal), Event::Mouse dispatch→KeyInput IPC. 5 pts, EPIC-09, Wave 9, P0. 14-pass adversarial convergence (3 consecutive CLEAN: passes 12/13/14). Trajectory: P1 BLOCKER (crossterm 1003 Moved-reachable SPEC ERROR corrected)→P4 stale line-anchor→P7 HIGH (scoped capture leaked across Overlay transition; fixed + mouse_capture_active observable + regression test)→P11 BLOCKER (panic/exit no teardown; fixed unconditional restore_terminal + main.rs panic hook)→P12/13/14 CLEAN. Spec: BC-2.09.003 v1.5.2→v1.6.1, SS-embedded-pty v1.16.0→v1.17.0, S-041 v1.0→v1.3, EVAL-INDEX v1.41→v1.42, BC-2.09.007 v1.5.0→v1.5.1. Security PASS (0 crit/high; 1 MED non-blocking, 4 LOW). pr-reviewer APPROVE. 11/11 CI, NO admin bypass. Demo: docs/demo-evidence/S-041/ (WEBM+.tape). S-044 UNBLOCKED. New follow-ups: F-S041-SEC-PANIC-HOOK-STDOUT-LOCK, F-S041-SEC-LOW, F-S041-CAPTURE-LIFECYCLE-WRITESEAM, WG-S041-EMBEDDED-ENTRY-TRIGGER. PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP 4th recurrence (commit 5c93579 to develop; orchestrator caught). PROCESS-GAP-SPEC-ALGORITHM-NOT-GROUNDED-IN-CRATE-SOURCE 2nd instance. sprint-state v1.57→v1.58; STORY-INDEX v5.64→v5.65. Wave 9: 5/6 done (32/45 pts). 43/51 done (262/314 pts). develop HEAD: 58fbd617. STATE v8.11→v8.12.
- D-346 (2026-06-22): S-043 MERGED to develop via PR #53 @ 5e6a2e0 (squash-merge). Scrollback Navigation — PtyScrollUp/Down actions, per-session HashMap<String,usize> offsets, configurable capacity (default 1000, clamp 1..10000), [scrolled back N rows] status indicator, content-anchored preservation via vt100-native mechanism. 3 pts, EPIC-09, Wave 9, P1. 8-pass adversarial convergence (3 consecutive CLEAN: passes 6/7/8). Trajectory: P1 BLOCKER (scroll actions unreachable from keyboard — dead trigger; KeyEventKind dispatch fix)→P2 MED (status_message suppressed by .or())→P3 BLOCKER (Kitty Release double-scroll — KeyEventKind::Press/Repeat guard)→P4 2×BLOCKER (content-anchor drift at scrollback cap + missing cap test) + HIGH (tautological test)→P5 HIGH (stale delta-probe doc-comment)→P6/P7/P8 CLEAN. All fixed in-scope. Spec corrections: SS-embedded-pty v1.14.0→v1.16.0 (vt100-native content-anchoring; corrected history-depth-delta algorithm), BC-2.09.007 v1.3.2→v1.5.0 (PC-4 concurrent-badge mandate, PC-5 content-anchored via vt100-native), S-043 story v1.1→v1.5. EVAL-INDEX v1.41→v1.42 (S-043 pin). BC-2.09.002/004 arch-source pins historical-qualified (POL-11 cascade). Security PASS (2 LOW pre-existing in on_dump_window_timeout; not introduced by S-043). pr-reviewer APPROVE. 11/11 CI green, NO admin bypass. Demo evidence docs/demo-evidence/S-043/ (WEBM + .tape). Closed: BURST-GAP-003/004/005. New follow-ups: F-S043-SEC-DUMPTIMEOUT-UUID-VALIDATE, F-S043-PRREV-SUGGESTIONS, F-S043-RENDER-OFFSET-WRITEBACK, WG-S043-PERMISSION-BADGE-COEXIST. PROCESS-GAP-STUB-PHASE-DOCCOMMENTS 2nd recurrence (S-043 P1 HIGH-004 + P5 HIGH-001); lessons.md updated. NEW LESSON: PROCESS-GAP-SPEC-ALGORITHM-NOT-GROUNDED-IN-CRATE-SOURCE recorded in lessons.md. Wave 9: 4/6 done (27/45 pts). 42/51 done (257/314 pts). develop HEAD: 5e6a2e0. STATE v8.10→v8.11.
- D-345 (2026-06-22): S-042 MERGED to develop via PR #51 @ 2f01de0 (squash-merge). Full end-to-end PTY Resize Detection + 50ms Debounce + ResizePane IPC. 8 pts, EPIC-09, Wave 9, P0. 9-pass adversarial convergence (3 consecutive CLEAN: passes 7/8/9). Trajectory: P1 BLOCKER (dead detection code)→P2 BLOCKER (unwired debounce fire) + daemon AC-013/014/016 violations + false-green daemon tests→P3 HIGH (stale resize-state on EmbeddedTerminal→Overlay)→P4 MED (fire latency vs PC-8 100ms tick)→P5 CLEAN(+2 LOW OBS fixed)→P6 spec residue (SS-session-manager stale ResizePane ownership)→P7/P8/P9 CLEAN. Security PASS (3 in-scope fixes: SEC-001 CWE-749 #[cfg(test)] gate, SEC-002 CWE-190 checked u32 cast, SEC-003 CWE-532 UUID guard before log; commits 4865ffe+9cf2482). pr-reviewer APPROVE. 11/11 CI green, NO admin bypass. Spec: BC-2.09.006 v1.3.0, S-042 story v1.5, SS-session-manager v2.17.1 (spec cascade already committed D-344). New follow-ups: F-S042-OBS-WRITEFRAMED-DOCNAMING, F-S042-OBS-LASTPANEAREA-DOC, WG-S042-SESSIONHOST-KEYINPUT. Wave 9: 3/6 done (24/45 pts). 41/51 stories done (254/314 pts). develop HEAD: 2f01de0. STATE v8.09→v8.10.
- D-343 (2026-06-21): Zero-context durability checkpoint. S-042 stubs committed @ 40dd53a on story/S-042-resize-debounce (local worktree, not pushed). ResizePane wire variant + ipc_server stub + App fields + todo!() methods. Red Gate enforceable. Test-writer next. New follow-up: WAVE-GATE-IPC-WRITER-ERROR-TAXONOMY (Wave-9 gate). STATE v8.08→v8.09.
- D-342 (2026-06-21): S-040 MERGED to develop via PR #50 @ d230a26 (squash-merge). Full-Fidelity Keyboard Forwarding, 8 pts, EPIC-09, Wave 9, P0. 17-pass adversarial convergence (3 consecutive CLEAN: passes 15/16/17). Trajectory: P1 BLOCKER (unwired dispatch)→P2 BLOCKER (Kitty dead-code)→P3 BLOCKER (CSI?u keystroke-theft)→P4 2×HIGH→P5 BLOCKER (paste guard size)→P6 MED→P7-P14 polish→P15/16/17 CLEAN. 65 behavioral tests green. Security PASS_WITH_NOTES (0 crit/high, 2 LOW non-blocking). pr-reviewer APPROVE. 11/11 CI green, NO admin bypass. Spec: SS-embedded-pty v1.14.0, BC-2.09.002 v1.2.2, BC-2.09.004 v1.0.11, BC-2.09.005 v1.0.7, S-040 v1.8, BC-INDEX v1.44.5, EVAL-INDEX v1.38. New follow-ups: SEC-001-S040-LOG-BEFORE-VALIDATE, SEC-003-S040-OVERSIZE-PASTE-UX, F-S040-KEYINPUT-DOC-EMPTYBYTES, PROCESS-GAP-STUB-PHASE-DOCCOMMENTS. Wave 9: 2/6 done (16/42 pts). 40/51 stories done (246/311 pts). STATE v8.07→v8.08.
- housekeeping (2026-06-20): docs(resume) PR #49 @ a852934 merged CLEAN. D-341 production validation. STATE v8.06→v8.07.
- D-341 (2026-06-20): PROCESS-GAP-DTU-FIDELITY-PATH-FILTER-DEADLOCK RESOLVED. PR #48 @ 3eba172 merged CLEAN. develop HEAD: 3eba172. STATE v8.05→v8.06.
- D-340 (2026-06-20): S-039 MERGED PR #47 @ a7ad00e. PTY output pipeline. Wave 9: 1/6 (8/42 pts). 39/51 stories done (238/311 pts). STATE v8.04→v8.05.
- D-339 (2026-06-20): S-036 MERGED to develop via PR #46 @ d924183 (squash). Sixth Wave-8 story (TIER-3). rediscover_sessions, 8 pts, EPIC-08. BC-2.08.002 (session-host survives graceful restart via setsid) + BC-2.08.004 v1.4.0 (all alive sessions visible after restart within 5s; UDS bind blocked). 12-pass adversarial convergence (3 consecutive CLEAN: passes 10/11/12). BLOCKER-001 SO_PEERCRED on Terminating Kill connect + BLOCKER-002 watchdog select! early-close leak fixed in-scope. HIGH (PID cross-check/dual-pid SIGTERM/watchdog registry leak/frame-cap MAX_FRAME_LEN/§3b emission gaps) fixed in-scope. MED (null-deadline 12s-window/GC-grace anchored at Terminated/§3b emission helper) fixed in-scope. SEC-001 CWE-284 pid=0 guards + SEC-002 CWE-284 SO_PEERCRED pid=None bypass + SEC-003 CWE-22 socket_path traversal — all RESOLVED IN-SCOPE during PR #46 security review. AC-004 spec-text drift (proxy_task: Some→None) corrected. MED-002 RESOLVED for S-036 scope (no SessionEntry.generation guard; three-layer safety argument extends to re-discovery). SS-session-manager v2.15.0→v2.15.1; BC-2.08.004 v1.3.5→v1.4.0; EVAL-INDEX v1.26→v1.28; S-036 v1.3→v1.5. PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP recurred (architect commit 772cb68 to develop; caught by orchestrator; develop reset; folded into worktree PR). POSITIVE: PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED held. Wave 8: 6/12 done (38/74 pts). 38/51 stories done (230/311 pts). develop HEAD: d924183. STATE v8.03→v8.04.
- D-338 (2026-06-19): S-038 MERGED to develop via PR #44 @ 8d649ea (squash) + chore PR #45 @ 7f005af. Fifth Wave-8 story; WAVE-8 TIER-2 COMPLETE. 6-pass adversarial convergence (3 consecutive CLEAN: passes 4/5/6). Single-writer mandate: lifecycle step 9 is sole canonical write_hooks_settings_json; lifecycle::write_hooks_settings + HooksSettings/HooksMap/HookEntry/HookCommand removed. lock.app='monocle' mandatory. BC-2.08.006→v1.5.0; BC-2.04.010→v1.4.0 (PC-3 lock.app). SS-session-manager→v2.15.0; BC-2.08.007→v1.5.6 (arch-source cascade). SEC-001 (CWE-732 perms-before-persist) + SEC-002 (CWE-532 redacting Debug) fixed in-scope (daeb4f2). security-reviewer re-verify PASS; adversary re-verify CLEAN. 10 BC-2.08.006 tests. 11/11 CI. develop HEAD: 7f005af. sprint-state v1.50→v1.51; STORY-INDEX v5.49→v5.50. STATE v8.01→v8.02.
- D-336 (2026-06-19): S-035 MERGED to develop via PR #43 @ 270b7d4. Fourth Wave-8 story. 9-pass adversarial (3 CLEAN). attach/detach. CRIT-001 fixed. Ruling L. S-036 UNBLOCKED. Sprint-state v1.50; STORY-INDEX v5.49. STATE v8.00→v8.01.
- D-335 (2026-06-19): S-037 MERGED to develop via PR #42 @ a7e4081. Third Wave-8 story (second Tier-2). GC task + rename_session. 7-pass adversarial convergence (3 consecutive CLEAN: passes 5/6/7). SEC-001 (CWE-20) + SEC-002 (CWE-706) fixed in-scope (b2d65db). PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH RESOLVED. Wave 8: 3/12 done (19/74 pts). sprint-state v1.48→v1.49; STORY-INDEX v5.47→v5.48. STATE v7.98→v7.99.
- D-334 (2026-06-18): S-034 MERGED to develop via PR #41 @ 4dfe0db. Second Wave-8 story (first Tier-2). Kill path: 18-pass adversarial convergence (3 CLEAN). SS-session-manager v2.11.0, BC-2.08.003 v1.5.0, BC-2.08.008 v1.3.5. Unblocks S-037. Wave 8: 2/12 done (16/74 pts). sprint-state v1.47→v1.48; STORY-INDEX v5.46→v5.47. STATE v7.97→v7.98.
- D-333 (2026-06-17): Zero-context resume checkpoint. develop @ 314326e. S-034 worktree ready. Wave-8 Tier-2 autonomous delivery authorized. Demo: WEBM+.tape-NO-GIF. 11 CI branch-protection contexts enforced. STATE v7.95→v7.96.
- D-332 (2026-06-17): S-033 MERGED to develop via PR #40 @ c7e10f2. First Wave-8 story. 10th crate monocle-session-host. Unblocks S-034/035/037/038/045. sprint-state v1.46→v1.47; STORY-INDEX v5.45→v5.46.
- D-323..D-331 (2026-06-16/17): Phase-2 pre-gate cleanup (D-323), inputs-pin fix (D-324), Phase-2 gate APPROVED (D-325), S-033 Rulings A–H (D-326..D-329), pass 4 remediation (D-330), S-033 convergence COMPLETE 3/3 clean (D-331). Full detail in decisions-archive.md.
