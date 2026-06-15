# monocle — Resume From Here (Phase-2 next, D-305, 2026-06-15)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.55, for the full checkpoint +
section E ratified decisions + durable_task_register).

---

## Status at Pause — Phase-1d FULLY COMPLETE

**Phase-1d adversarial spec convergence is DONE.** 57 passes total.
3 consecutive clean passes achieved (D-298/D-299/D-300 = Passes 55/56/57).
Consistency audit DONE (D-301, 4 cross-doc gaps found and fixed).
Human gate PASSED (D-302): v1A spec package APPROVED by Joshua Magady.
Both risk sign-offs SIGNED: CC-TUITERM-WIP-SIGNOFF + CC-GLOBAL-MOUSE-CAPTURE.
Input-hash content-review DONE (D-303): 0 semantic drift across 148 stale clusters;
149 spec input-hashes re-baselined; circular-dep STALE residual documented (non-blocking).

Two new ratified human decisions (D-304/D-305, 2026-06-15):
- **D-304**: Orchestrator may run Phase-2 Bursts A–G WITHOUT a per-burst plan-review gate.
- **D-305**: 143 pre-pivot observe-only stories — story-writer writes a RECOMMENDATION doc
  (Burst D); human must RATIFY before any disposition is executed. Bulk-archive BLOCKED.

**Adversarial counter is MOOT for Phase-1d.** It RESETS for Phase-2 (new convergence cycle, 0 of 3 clean).

develop @ 2141adc — no v1A production code written.
factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'` (live; do NOT trust a static SHA here).
STATE.md = v7.55.

---

## Next Action: Phase-2 Burst A (IMMEDIATE)

Dispatch `vsdd-factory:story-writer` for **Burst A**: create SS-08 session-manager story
FILES ONLY (new EPIC-07), story IDs starting S-033. No BC edits, no index edits.

### Full Phase-2 Burst A–G Dispatch Plan

Bursts run **SEQUENTIALLY** so story IDs do not collide. Each create burst reports the
next free story ID before returning. state-manager is dispatched LAST in any burst
that also runs other agents. Numbering is continuous from S-033.

**Burst A — vsdd-factory:story-writer**
Create SS-08 session-manager STORY FILES ONLY (new EPIC-07), IDs starting S-033.
No BC edits, no index edits. Report created IDs + BC-to-story mapping + next free ID.
BCs: BC-2.08.001 (spawn_session/SessionHostSpawner), .002 (re-discovery + setsid in
monocle-session-host), .003 (kill_session), .004 (daemon_start_sequence step-8b
rediscover_sessions), .005 (GC task), .006 (hook auto-injection in spawn path),
.007 (attach/detach), .008 (SessionStateChanged broadcast). Some may cluster.

**Burst B — vsdd-factory:story-writer** (after Burst A reports IDs)
Create SS-09 embedded-pty STORY FILES ONLY (new EPIC-08), continuing IDs after A.
No BC edits, no index edits. Report created IDs + BC-to-story mapping + next free ID.
BCs: BC-2.09.001 (TUI PTY widget: vt100 parser/PseudoTerminal render/PtyOutput handler),
.002 (key_event_to_pty_bytes + KeyInput IPC), .003 (mouse_event_to_pty_bytes + SGR),
.004 (Kitty keyboard — clusters with .002), .005 (paste — clusters with .002),
.007 (scrollback navigation), .008 (EmbeddedTerminal/SessionCreation AppMode transitions),
.009 (permission badge + bell).

**Burst C — vsdd-factory:story-writer** (after Burst B reports IDs)
Create SS-03/SS-05/SS-06 DELTA story files only (additions to EPIC-03/05/06).
No BC edits, no index edits. Report created IDs + BC-to-story mapping.
SS-03 BCs: BC-2.03.001..008 (EngineModule trait + ClaudeCodeModule spawn path;
.006/.007/.008 may fold into the .005 spawn story).
SS-05 BCs: BC-2.05.001 (UDS bind 0o600), .002 (TUI UDS connect+InitialState),
.003 (SessionListUpdate fan-out), .005 (PermissionPromptQueued), .006 (TUI reconnect
backoff), .007 (TransportEvent::Disconnected), .008 (Transport trait/UdsTransport),
.009 (PtyOutput broker fan-out), .010 (new ClientToServer variants+routing), .011
(ScrollbackChunk*/Complete/PtyReset). SKIP BC-2.05.004 (already -> S-021/S-032).
SS-06 BC: BC-2.06.025 (multi-session grouped sessions panel + lifecycle actions).

**Burst D — vsdd-factory:story-writer** (after Burst C; independent of next story ID)
Analyze 143 orphaned pre-pivot observe-only stories. Write RECOMMENDATION DOC at
`.factory/stories/pre-pivot-disposition-recommendation.md`.
Per-story or per-group: archive / retire / mark-done-historical.
**CRITICAL**: DO NOT execute disposition. Orchestrator routes doc to human to RATIFY (D-305).
Bulk-archive BLOCKED until human ratifies.

**Burst E — vsdd-factory:product-owner** (after Burst C IDs are known; can run after Burst D)
Resolve every S-TBD anchor in the 25 v1A BC files AND every `stories_tested=[S-TBD]` in
holdouts HS-EXP-011..015, using now-known story IDs from Bursts A–C.
PO MUST NOT touch story body content — story-writer propagates body/AC changes in Burst F.

**Burst F — vsdd-factory:story-writer** (AFTER product-owner Burst E)
Integrate new stories into STORY-INDEX.md (S-033+, continuous), sprint-state.yaml,
wave schedule (define Wave 8+), dependency-graph. Propagate BC anchor changes from Burst E.
New epics: EPIC-07 (SS-08), EPIC-08 (SS-09) plus additions to EPIC-03, EPIC-05, EPIC-06.
Wave 7 stories remain DONE. New stories at Wave 8+. No dependency cycles.

**Burst G — vsdd-factory:state-manager LAST** (after Bursts A–F)
Refresh indexes/citations, version bumps (BC-INDEX, STORY-INDEX, EVAL-INDEX,
sprint-state.yaml), update STATE.md, run compute-input-hash --update, POL-11, POL-12.
Bookkeeping only — no spec content changes.

**Post-Burst**: Phase-2 adversarial story convergence (3 consecutive clean, counter from 0).
Fresh consistency audit before Phase-2 human approval gate.

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
  kill->idempotent Ok; resize->WARN-drop. BC-2.06.025 v1.5.0 Invariant 6 closes all cells.
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
