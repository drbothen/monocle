# monocle — Resume From Here (Phase-2 next, D-303, 2026-06-14)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.54, for the full checkpoint +
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

Human said: **"Begin Phase-2 now."**
Then asked for a durable checkpoint first — so resume target is Phase-2, NOT a Phase-1d continuation.

**Adversarial counter is MOOT for Phase-1d.** It RESETS for Phase-2 (new convergence cycle, 0 of 3 clean).

develop @ 122eed5 — no v1A production code written.
factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'` (live; do NOT trust a static SHA here).
STATE.md = v7.54.

---

## Next Action: Phase-2 Delta Story Decomposition

Dispatch `vsdd-factory:story-writer` to decompose the v1A control-center pivot spec
package into implementable stories. This is a DELTA decomposition: new stories on top of
the existing pre-pivot corpus (STORY-INDEX v5.32, EPIC-01..06, S-001..S-032).

### Phase-2 Steps

1. **story-writer creates stories** for all 25 v1A BCs (listed below), resolving every
   `S-TBD` anchor in the BC files and every `stories_tested=[S-TBD]` in holdouts HS-EXP-011..015.

2. **story-writer integrates** new stories into STORY-INDEX v5.32 (continuous numbering
   S-033+), sprint-state.yaml v1.40, and defines Wave 8+ in wave schedule.
   New epics expected: EPIC-07 (Session Manager / SS-08), EPIC-08 (Embedded PTY / SS-09),
   plus additions to EPIC-03 (SS-03 engine-module), EPIC-05 (SS-05 IPC), EPIC-06 (SS-06 TUI).

3. **Pre-pivot story disposition:** 143 orphaned observe-only stories deferred here.
   story-writer decides: archive / retire / mark-done-historical. Do NOT carry them into
   the v1A wave as active targets. These are also the source of the 143 UNRESOLVABLE
   input-hash entries (circular-dep tool limitation; non-blocking).

4. **Adversarial story convergence:** 3 consecutive clean passes (new cycle; starts at 0).
   Fresh consistency audit before Phase-2 human gate.

> ORCHESTRATOR SPLIT RULE: dispatching story-writer to create >8 stories → split into
> "create" and "integrate" sub-bursts (context-overflow rule).

### v1A BCs Needing Stories (by subsystem)

**SS-03 (engine-module):** BC-2.03.001 (EngineModule trait/monocle-core), .002 (ClaudeCodeModule
strict-basename detect), .003 (HomeUnresolvable error path), .004 (ClaudeCodeModule inherent
methods/hook_paths), .005/.006/.007/.008 (ClaudeCodeModule::spawn_recipe() + default + error
handling — .006/.007/.008 may resolve to the same story as .005).

**SS-05 (IPC):** BC-2.05.001 (UDS server bind 0o600), .002 (TUI UDS connect + InitialState),
.003 (SessionListUpdate fan-out), .005 (PermissionPromptQueued), .006 (TUI reconnect backoff),
.007 (TransportEvent::Disconnected), .008 (Transport trait/UdsTransport), .009 (PtyOutput broker
fan-out), .010 (new ClientToServer IPC variants + routing), .011 (ScrollbackChunk*/Complete/PtyReset).
NOTE: BC-2.05.004 already resolved to S-021/S-032.

**SS-06 (TUI sessions panel):** BC-2.06.025 (multi-session grouped sessions panel + lifecycle actions).

**SS-08 (session-manager):** BC-2.08.001 (spawn_session/SessionHostSpawner), .002 (re-discovery +
setsid in monocle-session-host), .003 (kill_session), .004 (daemon_start_sequence 8b
rediscover_sessions), .005 (GC task), .006 (hook auto-injection in spawn path), .007
(attach/detach), .008 (SessionStateChanged broadcast).

**SS-09 (embedded-pty):** BC-2.09.001 (TUI PTY widget: vt100 parser/PseudoTerminal
render/PtyOutput handler), .002 (key_event_to_pty_bytes + KeyInput IPC), .003
(mouse_event_to_pty_bytes + SGR), .004 (Kitty keyboard branch — same cluster as .002),
.005 (paste — same cluster as .002), .007 (scrollback navigation), .008 (EmbeddedTerminal/
SessionCreation AppMode transitions), .009 (permission badge + bell).

**Holdouts with S-TBD anchors:** HS-EXP-011/012/013/014/015 — resolve stories_tested to real story IDs.

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
| BC-INDEX | v1.41.0 (138 BCs; 25 v1A BCs) |
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
- **Terminated-in-grace action matrix**: rename→rename_failed; detach→idempotent Ok;
  kill→idempotent Ok; resize→WARN-drop. BC-2.06.025 v1.5.0 Invariant 6 closes all cells.
- **BC-2.06.025 Launching action rules**: kill ALLOWED; detach BLOCKED (session_not_ready);
  rename ALLOWED. EC-298/EC-299 added.
- **session_not_ready producer**: DetachSession arm only (Launching, host_conn None).
  Resize WARN-drops ALL errors (Invariant 6 Exception). Kill → kill_failed (PID fallback).
- **hooks-settings.json**: 4 URL-bearing keys + 2 reserved-empty keys; SessionStart NOT a key.
- **ADR-0006 constructors**: all v1A #[non_exhaustive] wire structs have compliant constructors.
- **Version-less §Architecture Anchors**: navigational only; authoritative pins in §Architecture Source.
- **Concurrent multi-TUI-client**: ratified FUTURE scope (v1B+). Not a v1A defect.
- v1B (Interactive Tune) BCs/stories: NOT yet authored. Author when v1B scheduled.

---

## Read-First Order for Any Agent

1. This file (NEXT-SESSION-RESUME.md) — concise entry point
2. `/Users/jmagady/Dev/monocle/CLAUDE.md` — production-grade + agent-routing rules
3. `.factory/STATE.md` `next_session_resume_protocol` block (v7.54) — full checkpoint,
   durable_task_register, section E ratified decisions

---

## Already-Built Substrate (do NOT re-implement)

9 workspace crates: monocle-core, monocle-runtime, monocle-proto, monocle-test-harness,
monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui.
1514 tests, 0 failures (develop @ 6811103 wave-7-gate). Waves 1-7 DONE (32/33 stories, 192/195 pts).
Daemon wires (D-235), TUI (S-025..S-029/S-031), hook ingestion, VecDeque permission overlay,
EngineModule/FactoryAdapter traits, proto/ring. DTU clone S-DTU-001 validated fidelity 1.0 (D-234).
