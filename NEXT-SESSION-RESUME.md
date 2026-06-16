# monocle — Resume From Here (Phase-2 gate ready, 2026-06-16)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.83, for the full checkpoint +
section E ratified decisions + durable_task_register).

---

## Status at Pause — Phase-2 Pre-Gate Validations DONE; Paused for Laptop Relocation

**PAUSED for laptop relocation. Phase-2 ADVERSARIAL CONVERGENCE COMPLETE (3/3).
Pre-gate validations DONE.** factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'`.

51 stories (311 pts) total: 32 done (192 pts, Phases 1-3), 16 not_started (v1A Waves 8-9),
1 blocked (S-PHASE-3-PREP), 2 draft.

BC-INDEX v1.43.7, ARCH-INDEX v1.0.30, EVAL-INDEX v1.18, STORY-INDEX v5.44, sprint-state v1.45, wave-schedule v1.9.
SS-ipc v1.24.0, SS-session-manager v2.6.1, SS-daemon-wiring-v2-delta v1.11.4.
SS-embedded-pty v1.7.0, SS-deps-pin-manifest-v2-delta v1.0.2.
S-033 v1.5, S-038 v1.2, S-046 v1.3, S-047 v1.4, S-048 v1.3, S-044 v1.1.
POL-11 PASS. POL-12 PASS.

**D-315 RATIFIED**: pre-pivot disposition ratified by Joshua Magady (2026-06-16).
Keep all 3 active (S-032, S-DAEMON-WIRE-FIX-001, S-PHASE-3-PREP). 32 done-historical. 0 archive/retire.
pre-pivot-disposition-recommendation.md v1.1 (status: ratified). D-305 caveat CLEARED.

**Adversarial counter: 3/3 COMPLETE.** Phase-2 adversarial story convergence COMPLETE.
Strict-3-clean gate satisfied (Passes 24/25/26). 26 passes total.

**D-322 Pre-gate validations (2026-06-16):**
- Consistency-validator gate audit = **GATE-AUDIT PASS** (0 blockers / 2 important / 5 advisory).
  Full v1A coverage confirmed (25 BCs + 16 stories + 5 holdouts). Scope aligned with D-238. Phase-3-ready.
  Findings: F-GATE-IMP-001 (state-manager: sprint-state pin cascade), F-GATE-IMP-002 (story-writer: wave-schedule S-032 dep text),
  F-GATE-ADV-001 (state-manager: sprint-state not_started→draft), F-GATE-ADV-002 (story-writer: epic detail stubs E-04..E-09),
  F-GATE-ADV-003 (state-manager: EVAL-INDEX inputs S-033..S-048), F-GATE-ADV-004 (story-writer: dep-graph-expansion title Waves 4-9).
  All NON-BLOCKING. See durable_task_register in STATE.md.
- Input-hash drift check = **INPUT-DRIFT CLEAN** (TOTAL=410, MATCH=131, STALE=21, UNCOMPUTED=1, NOINPUT=257).
  All 21 stale = circular-hash-cascade residual class-b; 0 genuine drift. Non-blocking.

develop @ 2141adc — no v1A production code written.
STATE.md = v7.83.

---

## Next Action: Phase-2 Human Approval Gate

**Option A (preferred if time allows):** Run optional post-convergence cleanup burst first, then gate.
Cleanup scope:
- state-manager: F-GATE-IMP-001 (sprint-state inputs pin cascade to current versions) + F-GATE-ADV-001 (sprint-state not_started→draft for S-033..S-048) + F-GATE-ADV-003 (EVAL-INDEX inputs S-033..S-048)
- story-writer: F-GATE-IMP-002 (wave-schedule S-032 dep text S-021/S-018→S-021/S-022/S-028) + F-GATE-ADV-002 (epic detail stubs E-04..E-09) + F-GATE-ADV-004 (dep-graph-expansion title/intro Waves 4-9) + deferred cosmetic F-P13-SUG-001, F-P21-SUG-001/002, F-P24-SUG-001/002, F-P25-SUG-001 + F-P20-BCGAP-001 (product-owner)

**Option B (fastest path):** Proceed directly to Phase-2 human approval gate. All maintenance findings are NON-BLOCKING.

### Phase-2 Human Approval Gate Criteria

Present to Joshua Magady for ratification:
- Phase-2 adversarial story convergence COMPLETE (26 passes; Passes 24/25/26 clean)
- Story corpus: 51 stories / 311 pts; 16 new v1A stories S-033..S-048; 2 new epics EPIC-08/EPIC-09; Waves 8-9
- All 25 v1A BCs anchored; 5 holdouts (HS-EXP-011..015) with story anchors
- Consistency gate: PASS (0 blockers); Input-hash drift: CLEAN; POL-11/POL-12: PASS

After human gate approval: Phase-3 TDD implementation for v1A Waves 8-9 (S-033..S-048).

### Phase-2 Adversarial Convergence Summary (passes 1–26)

All findings from Pass-1 through Pass-23 resolved. Passes 24, 25, 26 CLEAN. Strict-3-clean gate satisfied.

Key corpus at convergence:
- STORY-INDEX v5.44, sprint-state v1.45, wave-schedule v1.9, dep-graph-expansion v2.3
- BC-INDEX v1.43.7, SS-ipc v1.24.0, SS-session-manager v2.6.1, SS-embedded-pty v1.7.0
- SE-25 CLOSED GLOBALLY (depends_on↔blocks exact-inverse across 51 stories + 4 artifacts)
- SessionState canonical: 5 variants (Launching/Running/Detached/Terminating/Terminated); lives in monocle-ipc
- S-048 key bindings: k/d=kill (BC-2.06.025 PC-3 kill alias); D=detach; no "destroy" action

---

## Remaining Tooling Tasks (non-blocking for Phase-2 gate; before Phase-4)

1. **POL-11-PINFORMAT-BLIND-SPOT** (D-301, devops-engineer): extend check_version_pins.py with
   a Pattern (like POL-14 Pattern C) to detect `path.md vX.Y.Z §section` live Architecture-Source
   pins — invisible to current POL-11 (3rd POL blind-spot recurrence). Add CI + lefthook.

2. **INPUT-HASH-CHILD-RECOMPUTE** (D-303, devops-engineer): pre-commit hook on factory-artifacts
   should run compute-input-hash --scan --update when shared parent specs are edited, preventing
   re-accumulation. Also document circular-input-dep STALE as non-blocking tool limitation.

3. **ADV-W5GATE-HIGH-002** (pending): duplicate S-009 handler dead code. Route to implementer.

4. **DEDUP-IPC-HANDLER-SKELETON** (pending): de-duplicate SS-session-manager §IPC handler
   (canonical) vs SS-daemon-wiring-v2-delta §3 (mirror). Schedule before/with Phase-3.

5. **OBS-HS-PROSE-PHASE4-PREP**: two LOW holdout-prose imprecisions (HS-EXP-014:46 child_pid;
   HS-EXP-013:54 step-9 display-order). Deferred to Phase-4 holdout-eval prep.

6. **Long-standing durable_task_register items** (ADV-W5GATE-MED-001/003, SS-IPC-181 historical
   marker stale, BC-INDEX-TRACE-SS08-COUNT, etc.) — see STATE.md durable_task_register.

---

## Canonical Spec Package Versions (v1A, at Phase-2 convergence)

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
| EVAL-INDEX | v1.18 |
| STORY-INDEX | v5.44 |
| sprint-state.yaml | v1.45 |

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
- **BC-2.06.025 Launching action rules**: kill ALLOWED (k/d); detach BLOCKED (D, EC-298);
  rename ALLOWED (r). d is the kill alias per PC-3 (dispatches KillSession). D=detach only.
- **session_not_ready producer**: DetachSession arm only (Launching, host_conn None).
  Resize WARN-drops ALL errors (Invariant 6 Exception). Kill -> kill_failed (PID fallback).
- **hooks-settings.json**: 4 URL-bearing keys + 2 reserved-empty keys; SessionStart NOT a key.
- **ADR-0006 constructors**: all v1A #[non_exhaustive] wire structs have compliant constructors.
- **Version-less §Architecture Anchors**: navigational only; authoritative pins in §Architecture Source.
- **Concurrent multi-TUI-client**: ratified FUTURE scope (v1B+). Not a v1A defect.
- **Wave 9 ordering**: S-042→S-043 serial (S-042 owns pty_scroll_offsets reset in ResizePane handler).
- v1B (Interactive Tune) BCs/stories: NOT yet authored. Author when v1B scheduled.
- **D-304**: Autonomous Phase-2 dispatch authorized. No per-burst plan-review gate.
- **D-315**: Pre-pivot disposition RATIFIED (2026-06-16, Joshua Magady). Keep 3 active
  (S-032, S-DAEMON-WIRE-FIX-001, S-PHASE-3-PREP). 32 done-historical. 0 archive/retire.
  Actual pre-pivot story count: 35 (not 143 estimate). See pre-pivot-disposition-recommendation.md v1.1.

---

## Read-First Order for Any Agent

1. This file (NEXT-SESSION-RESUME.md) — concise entry point
2. `/Users/jmagady/Dev/monocle/CLAUDE.md` — production-grade + agent-routing rules
3. `.factory/STATE.md` `next_session_resume_protocol` block (v7.83) — full checkpoint,
   durable_task_register, section E ratified decisions

---

## Already-Built Substrate (do NOT re-implement)

9 workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness,
monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui.
1514 tests, 0 failures (develop @ 6811103 wave-7-gate). Waves 1-7 DONE (32/33 stories, 192/195 pts).
Daemon wires (D-235), TUI (S-025..S-029/S-031), hook ingestion, VecDeque permission overlay,
EngineModule/FactoryAdapter traits, proto/ring. DTU clone S-DTU-001 validated fidelity 1.0 (D-234).
