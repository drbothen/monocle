# monocle — Resume From Here (Phase-2 adversarial convergence next, 2026-06-15)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.56, for the full checkpoint +
section E ratified decisions + durable_task_register).

---

## Status at Pause — Phase-2 Delta Story Decomposition COMPLETE

**Phase-2 delta story decomposition (Bursts A–G) is DONE.** factory-artifacts HEAD: `05a4a35`.

51 stories (311 pts) total: 32 done (192 pts, Phases 1-3), 16 not_started (v1A Waves 8-9),
1 blocked (S-PHASE-3-PREP), 2 draft.

BC-INDEX v1.42.0, EVAL-INDEX v1.16, STORY-INDEX v5.33, sprint-state v1.41, wave-schedule v1.7.
25 v1A BC S-TBD anchors resolved to S-033..S-048. POL-11 PASS. POL-12 PASS.
compute-input-hash: 100 clusters updated.

**D-305 outstanding**: pre-pivot disposition recommendation written
(`.factory/stories/pre-pivot-disposition-recommendation.md`) — PENDING human ratification.
Bulk-archive of pre-pivot stories BLOCKED until human ratifies.

**Adversarial counter RESET to 0/3 for Phase-2 story convergence.**
5 surfaced story gaps logged to durable_task_register (BURST-GAP-001..005).

develop @ 2141adc — no v1A production code written.
factory-artifacts HEAD: `05a4a35` (run `git -C .factory log -1 --format='%h %s'` to verify live HEAD).
STATE.md = v7.56.

---

## Next Action: Phase-2 Adversarial Story Convergence (IMMEDIATE)

Dispatch `vsdd-factory:story-writer` for **Burst A**: create SS-08 session-manager story
FILES ONLY (new EPIC-07), story IDs starting S-033. No BC edits, no index edits.

### Phase-2 Burst A–G Complete (factory-artifacts 05a4a35)

All 7 bursts executed and committed to factory-artifacts.

| Burst | Agent | Deliverable | Status |
|-------|-------|-------------|--------|
| A | story-writer | S-033..S-038 (EPIC-08, Session Manager) | DONE |
| B | story-writer | S-039..S-044 (EPIC-09, Embedded PTY) | DONE |
| C | story-writer | S-045..S-048 (EPIC-03/05/06 delta) | DONE |
| D | story-writer | pre-pivot-disposition-recommendation.md | DONE — PENDING D-305 human ratification |
| E | product-owner | 25 v1A BC S-TBD anchors resolved; BC-2.06.025 v1.5.1; HS-EXP-011..015 story IDs | DONE |
| F | story-writer | STORY-INDEX v5.33, sprint-state v1.41, wave-schedule v1.7 | DONE |
| G | state-manager | BC-INDEX v1.42.0, EVAL-INDEX v1.16, POL-11/POL-12 PASS, 100 input-hash clusters updated, STATE v7.56 | DONE |

### Next: Phase-2 Adversarial Story Convergence

Dispatch `vsdd-factory:adversary` fresh-context on the full Phase-2 story corpus
(S-033..S-048 + epics EPIC-08/09 + STORY-INDEX v5.33 + sprint-state v1.41).
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
| ARCH-INDEX | v1.0.28 |
| SS-ipc | v1.23.2 |
| SS-session-manager | v2.6.0 |
| SS-embedded-pty | v1.6.0 |
| SS-engine-module-v2-delta | v1.6.0 |
| SS-daemon-wiring-v2-delta | v1.11.3 |
| SS-deps-pin-manifest-v2-delta | v1.0.1 |
| ADR-0009 | v1.0.2 |
| ADR-0010 | v1.6.0 |
| ADR-0011 | v1.2.1 |
| BC-INDEX | v1.41.1 (138 BCs; 25 v1A BCs) |
| EVAL-INDEX | v1.15 |
| STORY-INDEX | v5.32 |
| sprint-state.yaml | v1.40 |

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
  kill->idempotent Ok; resize->WARN-drop. BC-2.06.025 v1.5.1 Invariant 6 closes all cells.
- **BC-2.06.025 Launching action rules**: kill ALLOWED; detach BLOCKED (session_not_ready);
  rename ALLOWED. EC-298/EC-299 added.
- **session_not_ready producer**: DetachSession arm only (Launching, host_conn None).
  Resize WARN-drops ALL errors (Invariant 6 Exception). Kill -> kill_failed (PID fallback).
- **hooks-settings.json**: 4 URL-bearing keys + 2 reserved-empty keys; SessionStart NOT a key.
- **ADR-0006 constructors**: all v1A #[non_exhaustive] wire structs have compliant constructors.
- **Version-less §Architecture Anchors**: navigational only; authoritative pins in §Architecture Source.
- **Concurrent multi-TUI-client**: ratified FUTURE scope (v1B+). Not a v1A defect.
- v1B (Interactive Tune) BCs/stories: NOT yet authored. Author when v1B scheduled.
- **D-304**: Autonomous Phase-2 dispatch authorized. No per-burst plan-review gate.
- **D-305**: 143-story pre-pivot disposition requires story-writer RECOMMENDATION first, then
  human RATIFICATION before execution. Bulk-archive BLOCKED until ratified.

---

## Read-First Order for Any Agent

1. This file (NEXT-SESSION-RESUME.md) — concise entry point
2. `/Users/jmagady/Dev/monocle/CLAUDE.md` — production-grade + agent-routing rules
3. `.factory/STATE.md` `next_session_resume_protocol` block (v7.55) — full checkpoint,
   durable_task_register, section E ratified decisions

---

## Already-Built Substrate (do NOT re-implement)

9 workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness,
monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui.
1514 tests, 0 failures (develop @ 6811103 wave-7-gate). Waves 1-7 DONE (32/33 stories, 192/195 pts).
Daemon wires (D-235), TUI (S-025..S-029/S-031), hook ingestion, VecDeque permission overlay,
EngineModule/FactoryAdapter traits, proto/ring. DTU clone S-DTU-001 validated fidelity 1.0 (D-234).
