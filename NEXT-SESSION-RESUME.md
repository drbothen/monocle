# monocle — Resume From Here (D-350, S-047 MID-IMPL, 2026-06-22)

Read this file first, then `CLAUDE.md`, then `.factory/STATE.md` (authoritative).

STATE.md was compacted at D-350 and is the current source of truth for the full task
register, version pins, and durable follow-up register.

---

## Current Position

- **develop HEAD:** `45343ca` (S-046 PtyOutput Fan-out Broker, PR #55, D-349)
- **factory-artifacts HEAD:** run `git -C .factory log -1 --format='%h %s'`
- **STATE.md:** v8.15 (D-350 checkpoint, 2026-06-22)
- **Phase:** Phase-3 v1A — Waves 8+9
- **Stories:** 44/51 done (267/322 pts); Wave 8: 7/12 done (43/74 pts); Wave 9: 5/6 done (32/45 pts)
- **10 workspace crates:** monocle-core, monocle-runtime, monocle-proto,
  monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask,
  monocle-tui, monocle-session-host

---

## D-348: Producer-First Re-Sequencing (still governs)

Wave-9 delivery order: **S-046 (DONE) → S-047 (IN PROGRESS) → S-044**.
S-044 remains after S-047 merges.
Wave-9 gate = CONTRACT-GREEN (BCs green + 3 consecutive CLEAN adversarial per story).
EPIC-09 end-to-end integration gate = separate milestone after S-047 + S-044.

---

## What This Cycle Delivered (D-346 → D-349)

| Story | Pts | PR | SHA | Decision | Notes |
|-------|-----|----|-----|----------|-------|
| S-043 Scrollback Navigation | 3 | #53 | 5e6a2e0 | D-346 | PtyScrollUp/Down, per-session offsets, configurable capacity. BC-2.09.007. 8-pass adversarial (3 CLEAN). |
| S-041 Mouse Forwarding | 5 | #54 | 58fbd61 | D-347 | mouse_event_to_pty_bytes SGR 1006, scoped capture lifecycle, out-of-pane clip. BC-2.09.003. 14-pass adversarial (3 CLEAN). |
| S-046 PtyOutput Fan-out Broker | 5 | #55 | 45343ca | D-349 | Bounded INPUT channel, broadcast_to_subscribers (Option A), 1-strike disconnect, pty_drop_counter. BC-2.05.009/011. 8-pass adversarial (3 CLEAN). CRITICAL LESSON: inert-broker unit-test tautology (LESSON-S046-INERT-BROKER-UNIT-TEST-TAUTOLOGY). |

---

## S-047 IN PROGRESS (16 pts, EPIC-09/08, Waves 8+9)

**IPC Lifecycle Variants + Session-Host Producer (expanded from 8 pts by human ruling 2026-06-22).**
Full producer scope: session-host scrollback producer + daemon relay + TUI consumers.
BCs: BC-2.05.010/011. Subsystems: SS-05/SS-08. Story v1.8.
Architect design: SS-session-manager §Ruling M (commit 43c89d7 on factory-artifacts).
Story-writer integration: commit 6d9d2e7 on factory-artifacts.

**Worktree:** `/Users/jmagady/Dev/monocle/.worktrees/S-047`
**Branch:** `story/S-047-ipc-lifecycle-variants` (based develop @ 45343ca, pushed to origin @ 35c744d)
**Commits in worktree:**
- stub 6631aa2
- tests da18a3e (22 tests: 14 RED → now green, 8 pre-existing-GREEN)
- implementation commits (14/14 target tests green)

Build/clippy/fmt/POL-11/POL-12: clean at last checkpoint.
**Adversarial NOT started.**

---

## BLOCKER — Fix Before Adversarial

### F-S047-DAEMON-RELAY-INERT (BLOCKER, HIGH)

The daemon does NOT relay real `ScrollbackChunk` frames to the TUI client.

- `forward_scrollback_dump_to_client` in `session_manager/mod.rs:4400` sends
  `ServerToClient::ScrollbackDumpComplete{total_chunks:0}` **unconditionally** — stub never replaced.
- Proxy task receives `HostToDaemon::ScrollbackChunk` at `mod.rs:3585` but the
  TODO at `mod.rs:3914` discards those chunks instead of forwarding them.
- Session-host correctly produces real chunks; the daemon→client relay is UNWIRED.
- AC-006 + AC-SH-005 not delivered end-to-end.
- Same class of defect as the S-046 inert-broker gap (see LESSON-S046-INERT-BROKER-UNIT-TEST-TAUTOLOGY).

**Fix route:** implementer in `.worktrees/S-047`. If proxy-task vs. forward-function
coordination is ambiguous relative to Ruling M, route to architect first (spec-only adjudication).

### F-S047-CHUNK-ROWS-50-VS-200 (LOW)

Current implementation uses 50 rows/chunk; Ruling M / story AC-SH-004 specifies <= 200 rows/chunk.
Confirm 50 is intentional or align to <= 200. Non-blocking relative to the relay BLOCKER.

---

## Next-Action Queue (S-047 Resume, D-350 Order)

1. **S-047-NEXT-1** (implementer, BLOCKER): Wire the daemon scrollback relay per Ruling M.
   Replace `total_chunks:0` stub in `forward_scrollback_dump_to_client` (`mod.rs:4400`)
   AND/OR implement proxy-task TODO at `mod.rs:3914`. If proxy-task vs. forward coordination
   is ambiguous, route to architect first.

2. **S-047-NEXT-2** (test-writer): Add a real-relay integration test — session-host produces
   N > 0 chunks, daemon forwards, client receives N `ScrollbackChunk` +
   `ScrollbackDumpComplete{total_chunks:N}`. This test must FAIL against the current
   empty-dump stub (closes the coverage gap that masked F-S047-DAEMON-RELAY-INERT).

3. **S-047-NEXT-3**: Resolve F-S047-CHUNK-ROWS-50-VS-200 — align to <= 200 rows/chunk per
   Ruling M / story AC-SH-004, or confirm 50 is intentional with architect sign-off.

4. **S-047-NEXT-4**: Adversarial convergence (3 consecutive CLEAN, fresh-context each pass;
   whole-system trace; specifically verify daemon relay reaches real clients, like S-046
   inert-broker check) then demo-recorder (WEBM + .tape, NO GIF, `docs/demo-evidence/S-047/`)
   then pr-manager full lifecycle (9 steps) then devops worktree cleanup + reconcile
   then state-manager checkpoint.

**Post-S-047 queue:**
5. **S-044** (EmbeddedTerminal + SessionCreation entry trigger + wizard + SpawnAck +
   BC-2.09.009 permission badge + bell, 13 pts, EPIC-09) — after S-047 merges.
6. Wave-9 contract-green integration gate — after S-044 merges.
7. EPIC-09 end-to-end integration gate — after S-047 + S-044 (live vertical slice:
   enter → live PtyOutput → keyboard/mouse forwarding → exit). Verify:
   WG-S046-SPAWN-PROXY-NOT-STARTED, WG-S046-OOM-POSITIVE-COVERAGE,
   WG-S042-SESSIONHOST-KEYINPUT (closed by S-047), WG-S043-PERMISSION-BADGE-COEXIST.
8. **SEC-006-CCR-URL-VALIDATION** (CWE-20/93, MEDIUM) — MUST fix before S-045.
9. **S-048** — after S-022 + S-033 + S-047 all merged.
   F-W8INT-001/002/003 parked for Wave-8 gate; do NOT re-raise per-story.

---

## Per-Story Delivery Procedure (follow verbatim)

1. **stub-architect** — compilable `todo!()` stubs for all files in story scope (spec only — no code on develop)
2. **test-writer** — failing tests anchored to BCs; confirm Red Gate (all tests fail)
3. **implementer** — TDD green loop in worktree (pick failing test → minimum code → micro-commit)
4. **Step 4.5 adversarial convergence** — 3 consecutive CLEAN passes required; fresh-context each pass; route blockers to correct specialist; architect adjudicates cross-component design as SPEC-ONLY; cross-story integration findings go to wave-gate register, not in-scope
5. **demo-recorder** — WEBM + .tape recording (NO GIF); save to `docs/demo-evidence/S-NNN/`
6. **push story branch** — push to remote
7. **pr-manager all 9 steps** — create PR, security-review, pr-reviewer, 11 CI checks, clean merge to develop, delete branch
8. **orchestrator verifies merge** — via `gh pr view` before declaring done
9. **devops worktree cleanup** — `git worktree remove .worktrees/S-NNN` + reconcile main-checkout develop to origin (ff; if human direct-commit diverged develop, ASK human)
10. **state-manager checkpoint** — commit STATE.md + PUSH factory-artifacts; verify `git -C .factory log origin/factory-artifacts..HEAD` is EMPTY

---

## CI-Parity Rules (all 8 apply)

1. `cargo clippy --workspace --all-targets -- -D warnings` **in worktree** (CI uses `--all-targets`; plain `--workspace` misses test targets — PROCESS-GAP-CI-PARITY-1)
2. From **REPO ROOT** (not worktree): `python3 scripts/check_version_pins.py` (POL-11) AND `python3 scripts/check_structural_claims.py` (POL-12)
3. **No version literals in source doc-comments or test prose** — POL-11 will flag them; de-version all citations; historical snapshots use `<!-- version-pin-historical: ... -->` HTML comment
4. **Registry atomicity** (L-S027-004): any BC/SS spec version bump + `version-pin-registry.yaml` update = **one atomic** `factory-artifacts` commit; cascade STORY-INDEX/EVAL-INDEX/dependency-graph pins in same commit; NEVER commit the spec without updating the registry
5. **Unique `/tmp` paths** per story dispatch — prevents commit-message mixup across concurrent story agents
6. **Architect = spec only** — all code changes (including doc-comments in source files) go to implementer-in-worktree (PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP — THREE recurrences)
7. **pr-manager completes all 9 steps** — orchestrator verifies merge via `gh pr view` before declaring done (PROCESS-GAP-PRMANAGER-EARLY-RETURN)
8. **factory-artifacts push verification** (PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED, D-337): after every spec-bumping agent dispatch, verify `git -C .factory log origin/factory-artifacts..HEAD` is EMPTY and push immediately if not

**Mutating agents MUST be serialized in worktrees** (L-S042-ORCH-PARALLEL-WORKTREE): never dispatch two mutating agents to the same `.worktrees/S-NNN` concurrently.

---

## Branch Protection (11 required CI contexts — bare names)

develop branch requires all 11 status checks before merge (no admin override needed):
- 10 from `ci.yml`: `Preflight (toolchain + fmt + lint)`, `Build+Test (stable x86_64)`, `Build+Test (stable aarch64)`, `Build+Test (MSRV 1.88)`, `Daemon E2E`, `POL-11 version-pin freshness`, `POL-12 structural claims`, `POL-14 anchor-pin freshness`, `Semgrep SAST`, `cargo audit + cargo deny`
- 1 from `dtu-fidelity.yml`: `DTU fidelity oracle (cargo xtask dtu-fidelity)`

PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH RESOLVED (D-335).
PROCESS-GAP-DTU-FIDELITY-PATH-FILTER-DEADLOCK RESOLVED (D-341).

---

## Canonical Version Pins

Canonical source: `.factory/specs/version-pin-registry.yaml` (authoritative).

Do NOT read version numbers from this file — they go stale between cycles.
Always pull current pins from `.factory/specs/version-pin-registry.yaml` directly.

---

## Ratified Decisions (do NOT re-litigate)

- **D-238**: session-host-owns-PTY; daemon restart SURVIVES (CASE 2); NO tmux default
- **D-304**: Autonomous Phase-2 dispatch; no per-burst plan-review gate
- **D-315**: Pre-pivot disposition RATIFIED (32 done; 3 active kept; 0 archive)
- **D-325**: Phase-2 gate APPROVED; Phase-3 v1A active
- **D-332..D-349**: S-033..S-046 merged; Wave-8 Tier-1/2/3 complete; all decisions archived
- **D-348**: Wave-9 re-sequenced (S-046 then S-047 then S-044); WG-S042-SESSIONHOST-KEYINPUT UPGRADED
- **D-349**: S-046 MERGED PR #55 @ 45343ca; Wave 8: 7/12 done; 44/51 done
- **D-350**: S-047 expanded 8 pts to 16 pts; mid-implementation; BLOCKER F-S047-DAEMON-RELAY-INERT
- **Spawn-path Model A**: SpawnOptions on wire; SpawnRecipe daemon-internal
- **IPC taxonomy**: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
- **PTY (ADR-0011)**: portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
- **SessionState**: 5 variants (Launching/Running/Detached/Terminating/Terminated) in monocle-ipc

Full history: `.factory/cycles/cycle-001/decisions-archive.md` (D-001..D-350)

---

## Open Durable Follow-ups (do NOT fix unless specifically tasked)

See STATE.md `durable_task_register` and `.factory/cycles/cycle-001/task-register-full.yaml`
for the full register (127+ active entries). Key items:

| ID | Route | Description |
|----|-------|-------------|
| F-S047-DAEMON-RELAY-INERT | implementer (+ maybe architect) | BLOCKER — daemon relay unwired; see above |
| F-S047-CHUNK-ROWS-50-VS-200 | implementer | LOW — confirm 50 vs <= 200 rows/chunk per Ruling M |
| WG-S042-SESSIONHOST-KEYINPUT | wave-gate / S-047 | UPGRADED: KeyInput catch-all drops input; closed by S-047 |
| WG-S046-SPAWN-PROXY-NOT-STARTED | EPIC-09 gate | attach-driven canonical per Ruling 5 |
| WG-S046-OOM-POSITIVE-COVERAGE | EPIC-09 gate / test-writer | |
| SEC-006-CCR-URL-VALIDATION | implementer/security | CWE-20/93; MUST fix before S-045 |
| F-W8INT-001/002/003 | Wave-8 gate | Parked; do NOT re-raise per-story |
| DEMO-BINARY-ARTIFACTS-DEVELOP | devops/human | 7+ stories' WEBM on develop; repo-hygiene pending |
| PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP | devops | THREE recurrences; prompt hardening needed |

---

## Known-Flaky Tests (do NOT flag as new findings)

`cli_daemon_stop`, `factory_self_referential`, `test_BC_2_07_006`, wit-bindgen unmatched-skip, PATH isolation flake.

---

## Human Directive

**Continue autonomously** into subsequent ready stories until told to stop.
Per-story flow + 3 consecutive CLEAN adversarial convergence + clean merge applies to each story.
**Demo default: WEBM + .tape ONLY. NO GIF.**
