# monocle — Resume From Here (Phase-2 adversarial Pass-10 next, 2026-06-16)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.65, for the full checkpoint +
section E ratified decisions + durable_task_register).

---

## Status at Pause — Phase-2 Adversarial Pass-9 Findings Resolved

**Phase-2 adversarial Pass-9 FINDINGS (1C/0I/1S) ALL RESOLVED.** factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'`.

51 stories (311 pts) total: 32 done (192 pts, Phases 1-3), 16 not_started (v1A Waves 8-9),
1 blocked (S-PHASE-3-PREP), 2 draft.

BC-INDEX v1.43.7, ARCH-INDEX v1.0.30, EVAL-INDEX v1.16, STORY-INDEX v5.41, sprint-state v1.43, wave-schedule v1.8.
SS-ipc v1.24.0, SS-session-manager v2.6.1, SS-daemon-wiring-v2-delta v1.11.4.
SS-embedded-pty v1.7.0, SS-deps-pin-manifest-v2-delta v1.0.2.
POL-11 PASS (371 active). POL-12 PASS. compute-input-hash: 99 clusters updated.

**D-305 outstanding**: pre-pivot disposition recommendation written
(`.factory/stories/pre-pivot-disposition-recommendation.md`) — PENDING human ratification.
Bulk-archive of pre-pivot stories BLOCKED until human ratifies.

**Adversarial counter: 0/3** (Pass-9 not clean; Pass-10 = clean-candidate 1/3).
5 original story gaps in durable_task_register (BURST-GAP-001..005) + F-P1-S-005 + F-P2-S03 (process-gaps).

develop @ 2141adc — no v1A production code written.
STATE.md = v7.65.

---

## Next Action: Phase-2 Adversarial Pass-10 (IMMEDIATE)

Dispatch `vsdd-factory:adversary` fresh-context on the full Phase-2 story corpus
(S-033..S-048 + epics EPIC-08/09 + STORY-INDEX v5.41 + sprint-state v1.43 + all updated BC files).
Pass-10 = clean-candidate 1/3. Counter 0/3.

### Phase-2 Burst A–G + Pass-1 through Pass-6 Fix Bursts Complete

All 7 bursts executed and committed to factory-artifacts. Pass-1 through Pass-6 findings resolved.

| Burst | Agent | Deliverable | Status |
|-------|-------|-------------|--------|
| A | story-writer | S-033..S-038 (EPIC-08, Session Manager) | DONE |
| B | story-writer | S-039..S-044 (EPIC-09, Embedded PTY) | DONE |
| C | story-writer | S-045..S-048 (EPIC-03/05/06 delta) | DONE |
| D | story-writer | pre-pivot-disposition-recommendation.md | DONE — PENDING D-305 human ratification |
| E | product-owner | 25 v1A BC S-TBD anchors resolved; BC-2.06.025 v1.5.3 (at Burst E authoring time); HS-EXP-011..015 story IDs | DONE |
| F | story-writer | STORY-INDEX v5.33 (at Burst F authoring time), sprint-state v1.41, wave-schedule v1.7 (at Burst F authoring time) | DONE |
| G | state-manager | BC-INDEX v1.43.4 (at Burst G authoring time), EVAL-INDEX v1.16, POL-11/POL-12 PASS, 130 input-hash clusters updated, STATE v7.57 | DONE |
| Pass-1 fix | architect + product-owner + state-manager | SS-ipc v1.24.0, 30+ BC cascade, BC-INDEX v1.43.4 (at Pass-1 authoring time) | DONE (D-306) |
| Pass-2 fix | architect + state-manager | SS-embedded-pty v1.7.0, SS-deps-pin-manifest-v2-delta v1.0.2, 16 story inputs reconciled, 11 BC arch-source pins, BC-INDEX v1.43.5 (at Pass-2 authoring time), ARCH-INDEX v1.0.30, STORY-INDEX v5.34 (at Pass-2 authoring time) | DONE (D-307) |
| Pass-3 fix | story-writer + state-manager | S-038/S-037/S-045/S-046/S-047 path corrections; 32 BC input-pin refresh across 16 stories; STORY-INDEX v5.35 (at Pass-3 authoring time) | DONE (D-308) |
| Pass-4 fix | story-writer + state-manager | IPC-arm ownership S-033/034/035/047; S-037 contradiction; S-041 dep; BC body de-version 13 stories; STORY-INDEX v5.36 (at Pass-4 authoring time) | DONE (D-309) |
| Pass-5 fix | story-writer + state-manager | S-047 scrollback wire-shape; dep-graph-expansion v2.0 Wave 8-9 DAG; AC-range corrections; S-042→S-043 dep edge; STORY-INDEX v5.37 (at Pass-5 authoring time) | DONE (D-310) |
| Pass-6 fix | story-writer + product-owner + state-manager | wave-schedule v1.8 (S-042→S-043 4-location propagation); S-042 Dependency Justification + stale BC comment; 9 story authoring-note BC comments de-versioned; BC-2.05.011 v1.2.5 Story Anchor co-ownership; BC-INDEX v1.43.6 (at Pass-6 authoring time) | DONE (D-311) |
| Pass-7 fix | architect + product-owner + story-writer + state-manager | BC-2.03.008 v1.0.8 Story Anchor S-045→S-033 (Approach 1); AC-009c; S-033 +BC-2.03.008 behavioral_contract; S-045 -BC-2.03.008 from behavioral_contracts; dep-graph-expansion v2.1 S-047 self-loop fixed; STORY-INDEX BC Coverage row (at Pass-7 authoring time v5.38); S-038 path; BC-INDEX v1.43.7 | DONE (D-311) |
| Pass-8 fix | story-writer + state-manager | SE-25 bidirectional symmetry reconciliation across 4 artifacts; EPIC-08+EPIC-03 BC Coverage AC ranges corrected; S-045 AC renumber; dep-graph-expansion v2.2; STORY-INDEX v5.40 (at Pass-8 authoring time); sprint-state v1.43 S-033/034/035/039 blocks + S-038 canonical title; version-pin-registry STORY-INDEX→5.40 | DONE (D-311) |
| Pass-9 fix | story-writer + state-manager | S-033 v1.3 AC-009d BC-2.03.008 PC-3 spawn_unsupported; STORY-INDEX v5.41 26 BC Coverage Title cells normalized; S-033 body BC table title aligned; version-pin-registry STORY-INDEX→5.41 | DONE (this burst) |

### Next: Phase-2 Adversarial Pass-10

3 consecutive clean passes required. Adversarial counter: 0/3.

After convergence: fresh consistency audit (vsdd-factory:consistency-validator),
then Phase-2 human approval gate, then Phase-3 TDD implementation for Waves 8-9.

**Blocking prerequisite**: D-305 pre-pivot disposition recommendation
(`.factory/stories/pre-pivot-disposition-recommendation.md`) requires human ratification
before bulk-archive. Adversarial work and human gate are independent of D-305 ratification.

---

## Remaining Tooling Tasks (non-blocking for Phase-2; before Phase-4)

1. **POL-11-PINFORMAT-BLIND-SPOT** (D-301, devops-engineer): extend check_version_pins.py with
   a Pattern (like POL-14 Pattern C) to detect `path.md vX.Y.Z §section` live Architecture-Source
   pins — invisible to current POL-11 (3rd POL blind-spot recurrence). Add CI + lefthook.

2. **INPUT-HASH-CHILD-RECOMPUTE** (D-303, devops-engineer): pre-commit hook on factory-artifacts
   should run compute-input-hash --scan --update when shared parent specs are edited, preventing
   re-accumulation. Also document circular-input-dep STALE as non-blocking tool limitation.

3. **ADV-W5GATE-HIGH-002** (pending): duplicate S-009 handler dead code. Route to implementer.

4. **DEDUP-IPC-HANDLER-SKELETON** (pending): de-duplicate SS-session-manager §IPC handler
   (canonical) vs SS-daemon-wiring-v2-delta §3 (mirror). Schedule before/with Phase-2.

5. **OBS-HS-PROSE-PHASE4-PREP**: two LOW holdout-prose imprecisions (HS-EXP-014:46 child_pid;
   HS-EXP-013:54 step-9 display-order). Deferred to Phase-4 holdout-eval prep.

6. **Long-standing durable_task_register items** (ADV-W5GATE-MED-001/003, SS-IPC-181 historical
   marker stale, BC-INDEX-TRACE-SS08-COUNT, etc.) — see STATE.md §H for full list.

---

## Canonical Spec Package Versions (v1A, at Phase-1d convergence)

All versions derived from `.factory/specs/version-pin-registry.yaml` (source of truth).

| Document | Version |
|---|---|
| domain-monocle-vision-synthesis.md | v2.2.3 (APPROVED) |
| product-brief.md | v2.0.4 |
| prd.md | v1.28.3 |
| ARCH-INDEX | v1.0.30 |
| SS-ipc | v1.24.0 |
| SS-session-manager | v2.6.1 |
| SS-embedded-pty | v1.7.0 |
| SS-engine-module-v2-delta | v1.6.0 |
| SS-daemon-wiring-v2-delta | v1.11.4 |
| SS-deps-pin-manifest-v2-delta | v1.0.2 |
| ADR-0009 | v1.0.2 |
| ADR-0010 | v1.6.0 |
| ADR-0011 | v1.2.1 |
| BC-INDEX | v1.43.7 (138 BCs; 25 v1A BCs) |
| EVAL-INDEX | v1.16 |
| STORY-INDEX | v5.41 |
| sprint-state.yaml | v1.43 |

---

## Key Ratified Decisions (do NOT re-litigate — see section E in STATE.md for full list)

- **Persistence model**: session-host-owns-PTY (monocle-session-host binary, setsid-detached,
  per-session UDS). Graceful daemon restart SURVIVES (CASE 2 = survive). NO tmux default.
- **Spawn-path Model A**: SpawnOptions on wire; SpawnRecipe daemon-internal; spawn_recipe()
  called INSIDE spawn_session(); EngineError 3-variant canonical enum.
- **IPC schema**: 12-code wire taxonomy; 9-variant SessionError; SpawnAck handshake;
  launching_session_id; schema_version 3; snapshot-then-resume scrollback.
- **PTY stack** (ADR-0011): portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88.
- **Scoped mouse capture**: enabled on EmbeddedTerminal ENTRY, disabled on EXIT. NOT global.
- **Session lifecycle**: Launching / Running / Detached / Terminating / Terminated.
  Created and Killed are RETIRED.
- **Terminated-in-grace action matrix**: rename->rename_failed; detach->idempotent Ok;
  kill->idempotent Ok; resize->WARN-drop. BC-2.06.025 Invariant 6 closes all cells.
- **BC-2.06.025 Launching action rules**: kill ALLOWED; detach BLOCKED (session_not_ready);
  rename ALLOWED. EC-298/EC-299 added.
- **session_not_ready producer**: DetachSession arm only (Launching, host_conn None).
  Resize WARN-drops ALL errors (Invariant 6 Exception). Kill -> kill_failed (PID fallback).
- **hooks-settings.json**: 4 URL-bearing keys + 2 reserved-empty keys; SessionStart NOT a key.
- **ADR-0006 constructors**: all v1A #[non_exhaustive] wire structs have compliant constructors.
- **Version-less §Architecture Anchors**: navigational only; authoritative pins in §Architecture Source.
- **Concurrent multi-TUI-client**: ratified FUTURE scope (v1B+). Not a v1A defect.
- **Wave 9 ordering**: S-042→S-043 serial (S-042 owns pty_scroll_offsets reset in ResizePane handler).
- v1B (Interactive Tune) BCs/stories: NOT yet authored. Author when v1B scheduled.
- **D-304**: Autonomous Phase-2 dispatch authorized. No per-burst plan-review gate.
- **D-305**: 143-story pre-pivot disposition requires story-writer RECOMMENDATION first, then
  human RATIFICATION before execution. Bulk-archive BLOCKED until ratified.

---

## Read-First Order for Any Agent

1. This file (NEXT-SESSION-RESUME.md) — concise entry point
2. `/Users/jmagady/Dev/monocle/CLAUDE.md` — production-grade + agent-routing rules
3. `.factory/STATE.md` `next_session_resume_protocol` block (v7.65) — full checkpoint,
   durable_task_register, section E ratified decisions

---

## Already-Built Substrate (do NOT re-implement)

9 workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness,
monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui.
1514 tests, 0 failures (develop @ 6811103 wave-7-gate). Waves 1-7 DONE (32/33 stories, 192/195 pts).
Daemon wires (D-235), TUI (S-025..S-029/S-031), hook ingestion, VecDeque permission overlay,
EngineModule/FactoryAdapter traits, proto/ring. DTU clone S-DTU-001 validated fidelity 1.0 (D-234).
