# monocle — Resume From Here (Wave-9 S-042 MERGED, D-345, 2026-06-22)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`.

---

## Current Position

- **develop HEAD:** `2f01de0` (S-042 PTY Resize Detection + Debounce + ResizePane IPC, PR #51).
- **factory-artifacts HEAD:** run `git -C .factory log -1 --format='%h %s'`
- **STATE.md:** v8.10
- **Stories:** 41/51 done (254/314 pts); Wave 9: 3/6 done (24/45 pts)
- **10 workspace crates:** monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui, monocle-session-host

---

## What This Session Delivered

**S-042 (PTY Resize Detection + 50ms Debounce + ResizePane IPC, 8 pts, EPIC-09, Wave 9, BC-2.09.006) MERGED PR #51 @ 2f01de0 (D-345).** Full end-to-end resize pipeline. 9-pass adversarial convergence (3 consecutive CLEAN: passes 7/8/9). 32 behavioral tests. Security PASS (3 in-scope fixes).

| Story | Pts | PR | SHA | Decision | Notes |
|-------|-----|----|-----|----------|-------|
| **S-042** PTY Resize Detection + 50ms Debounce + ResizePane IPC | 8 | #51 | 2f01de0 | D-345 | Full end-to-end: TUI detection/debounce (last_pty_pane_area + 50ms deadline + check_resize_debounce), daemon ResizePane routing → resize_session() (zero-dim clamp + WARN-drop carve-out), DaemonToHost::Resize forwarding, session-host pty.resize() + parser.set_size(). 32 tests (13 resize_debounce + 6 run_loop_wiring + 2 poll_timeout_seam + 9 daemon_resize + 2 session-host). 9-pass adversarial (P1 BLOCKER dead-detection→P2 BLOCKER unwired-debounce+daemon-AC violations→P3 HIGH stale-state→P4 MED fire-latency→P5 CLEAN+2-LOW-fixed→P6 spec-residue→P7/P8/P9 CLEAN). Security PASS: SEC-001 CWE-749 #[cfg(test)] gate, SEC-002 CWE-190 checked u32 cast, SEC-003 CWE-532 UUID guard before log (commits 4865ffe+9cf2482). BC-2.09.006 v1.3.0; S-042 story v1.5; SS-session-manager v2.17.1. |

D-344 spec cascade (architect, already committed at factory-artifacts 22b5836+15d567a): BC-2.09.006 v1.3.0, SS-session-manager v2.17.1 (F-S042-ADV-MED-001 ResizePane ownership-drift cleanup), S-042 v1.5, S-047 v1.7, STORY-INDEX <!-- version-pin-historical: v5.62 at D-344 authoring time -->, EVAL-INDEX v1.40, sprint-state <!-- version-pin-historical: v1.55 at D-344 authoring time -->, 16 BC arch-source pin cascades, version-pin-registry.yaml.

---

## Human Directive

**Continue autonomously** into subsequent ready stories until told to stop. Per-story flow + 3 consecutive CLEAN adversarial convergence + clean merge applies to each story.

**Demo default: WEBM + .tape ONLY. NO GIF.**

---

## Next-Action Queue

1. **S-043** (scrollback-navigation, Wave 9) — **UNBLOCKED** (deps S-039 + S-042 both merged). BC: BC-2.09.007. 3 pts. **START HERE** or run S-041 in parallel.

2. **S-041** (mouse-forwarding-sgr, Wave 9) — **UNBLOCKED** (dep S-040 merged). BC: BC-2.09.003. 5 pts. S-043 and S-041 have no mutual dependency; may be delivered in either order.

3. **SEC-006-CCR-URL-VALIDATION** (CWE-20/93): `ccr_base_url` flows unvalidated from TUI wire to child env. **MUST be addressed before S-045.**

4. S-046 waits on S-032; S-047 needs S-033 + S-034 + S-035 + S-046; S-048 needs S-022 + S-033 + S-047.

---

## New Durable Follow-ups (from S-042 cycle)

| ID | Route | Description |
|----|-------|-------------|
| F-S042-OBS-WRITEFRAMED-DOCNAMING | tech-writer/implementer | [LOW, doc-only] resize_session doc-comment + AC-015 cite `write_framed_to_stream`; impl uses byte-identical inline framing. Clarify in future doc pass. Non-blocking. |
| F-S042-OBS-LASTPANEAREA-DOC | implementer | [LOW, doc-only] `App::last_pty_pane_area` doc claims "None when not EmbeddedTerminal" but field never reset on mode exit (benign — mode guard prevents stale read). Optional. |
| WG-S042-SESSIONHOST-KEYINPUT | wave-gate/implementer | [wave-gate] session-host has no DaemonToHost::KeyInput arm (S-040 keyboard leg) — confirm KeyInput end-to-end at Wave-9 integration gate. Non-blocking. |
| F-S039-P9-OBS-001 | wave-gate | EmbeddedTerminal render uses 24x80 placeholder; reconciled by S-042. Verify/close at Wave-9 integration gate. |
| WAVE-GATE-IPC-WRITER-ERROR-TAXONOMY | SUPERSEDED | ResizePane WARN-drop carve-out implemented in S-042 (D-344 ruling). No remaining action. |

---

## Per-Story Delivery Procedure (follow verbatim)

1. **stub-architect** — compilable `todo!()` stubs for all files in story scope (SPEC ONLY — no code on develop)
2. **test-writer** — failing tests anchored to BCs; confirm Red Gate (all tests fail)
3. **implementer** — TDD green loop in worktree (pick failing test → minimum code → micro-commit)
4. **Step 4.5 adversarial convergence** — 3 consecutive CLEAN passes required; fresh-context each pass; route blockers to correct specialist; architect adjudicates cross-component design as SPEC-ONLY; cross-story integration findings → wave-gate register, not in-scope
5. **demo-recorder** — WEBM + .tape recording (NO GIF); save to `docs/demo-evidence/S-NNN/`
6. **push story branch** — push to remote
7. **pr-manager all 9 steps** — create PR → security-review → pr-reviewer → 11 CI checks → clean merge to develop → delete branch
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
6. **Architect = spec only** — all code changes (including doc-comments in source files) go to implementer-in-worktree (PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP — THREE recurrences; see lessons.md)
7. **pr-manager completes all 9 steps** — orchestrator verifies merge via `gh pr view` before declaring done (PROCESS-GAP-PRMANAGER-EARLY-RETURN)
8. **factory-artifacts push verification** (PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED, D-337): after every spec-bumping agent dispatch, orchestrator MUST verify `git -C .factory log origin/factory-artifacts..HEAD` is EMPTY and push immediately if not

**Mutating agents MUST be serialized in worktrees** (L-S042-ORCH-PARALLEL-WORKTREE): never dispatch two mutating agents to the same `.worktrees/S-NNN` concurrently — commits can race. Serialize or use isolated worktrees.

---

## Branch Protection (11 required CI contexts — bare names)

develop branch requires all 11 status checks before merge (no admin override needed):
- 10 from `ci.yml`: `Preflight (toolchain + fmt + lint)`, `Build+Test (stable x86_64)`, `Build+Test (stable aarch64)`, `Build+Test (MSRV 1.88)`, `Daemon E2E`, `POL-11 version-pin freshness`, `POL-12 structural claims`, `POL-14 anchor-pin freshness`, `Semgrep SAST`, `cargo audit + cargo deny`
- 1 from `dtu-fidelity.yml`: `DTU fidelity oracle (cargo xtask dtu-fidelity)`

PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH is RESOLVED (D-335). PROCESS-GAP-DTU-FIDELITY-PATH-FILTER-DEADLOCK is RESOLVED (D-341). PRs merge CLEAN with no admin bypass.

---

## Ratified Decisions (do NOT re-litigate)

- **D-238**: session-host-owns-PTY; daemon restart SURVIVES (CASE 2); NO tmux default
- **D-304**: Autonomous Phase-2 dispatch; no per-burst plan-review gate
- **D-315**: Pre-pivot disposition RATIFIED (32 done; 3 active kept; 0 archive)
- **D-325**: Phase-2 gate APPROVED; Phase-3 v1A active
- **D-326..D-331**: S-033 Rulings A–H; adversarial convergence COMPLETE (7 passes, 3 clean)
- **D-332**: S-033 MERGED PR #40 @ c7e10f2
- **D-333**: Wave-8 Tier-2 autonomous delivery authorized; demo WEBM+.tape-no-GIF
- **D-334**: S-034 MERGED PR #41 @ 4dfe0db. Kill path complete. <!-- version-pin-historical: SS-session-manager v2.11.0, BC-2.08.003 v1.5.0, BC-2.08.008 v1.3.5 at D-334 checkpoint -->
- **D-335**: S-037 MERGED PR #42 @ a7e4081. GC + rename_session. SEC-001/002 fixed in-scope. BRANCH-PROTECTION RESOLVED.
- **D-336**: S-035 MERGED PR #43 @ 270b7d4. attach/detach. Ruling L (proxy_task kill-reader for attached sessions). S-036 UNBLOCKED.
- **D-337**: PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED codified (c8ceee1). Always push factory-artifacts after spec-bumping agents.
- **D-338**: S-038 MERGED PR #44 @ 8d649ea + chore PR #45 @ 7f005af. WAVE-8 TIER-2 COMPLETE. Single-writer mandate. <!-- version-pin-historical: BC-2.08.006 v1.5.0, SS-session-manager v2.15.0, BC-2.08.007 v1.5.6 at D-338 checkpoint -->
- **D-339**: S-036 MERGED PR #46 @ d924183. WAVE-8 TIER-3 COMPLETE. <!-- version-pin-historical: BC-2.08.002 v1.2.6, BC-2.08.004 v1.4.0 at D-339 checkpoint -->
- **D-340**: S-039 MERGED PR #47 @ a7ad00e. PTY output pipeline. 10-pass adversarial (3 CLEAN).
- **D-341**: DTU fidelity path-filter deadlock RESOLVED (PR #48 @ 3eba172). Future TUI-only PRs merge with no admin bypass.
- **D-342**: S-040 MERGED PR #50 @ d230a26. Full-Fidelity Keyboard Forwarding. 17-pass adversarial (3 CLEAN). NO admin bypass.
- **D-344**: Human ruling — S-042 expanded to full end-to-end resize pipeline (8 pts). S-047 ResizePane scope removed. Spec cascade committed factory-artifacts 22b5836+15d567a.
- **D-345**: S-042 MERGED PR #51 @ 2f01de0. Full end-to-end PTY Resize. 8 pts. 9-pass (3 CLEAN). Security PASS. NO admin bypass. Wave 9: 3/6 done. S-043 UNBLOCKED.
- **Spawn-path Model A**: SpawnOptions on wire; SpawnRecipe daemon-internal
- **IPC taxonomy**: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
- **PTY (ADR-0011)**: portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
- **SessionState**: 5 variants (Launching/Running/Detached/Terminating/Terminated) in monocle-ipc

Full history: `.factory/cycles/cycle-001/decisions-archive.md` (D-001..D-345)

---

## Open Durable Follow-ups (do NOT fix unless specifically tasked)

| ID | Route | Description |
|----|-------|-------------|
| F-S042-OBS-WRITEFRAMED-DOCNAMING | tech-writer/implementer | [LOW] resize_session doc-comment names `write_framed_to_stream`; impl uses byte-identical inline framing. Future doc pass. Non-blocking. |
| F-S042-OBS-LASTPANEAREA-DOC | implementer | [LOW] `App::last_pty_pane_area` doc: "None when not EmbeddedTerminal" but field never reset on mode exit (benign; mode guard prevents stale read). Optional. |
| WG-S042-SESSIONHOST-KEYINPUT | wave-gate/implementer | [wave-gate] session-host has no DaemonToHost::KeyInput arm (S-040 keyboard leg). Verify at Wave-9 gate. |
| F-S039-P9-OBS-001 | wave-gate | EmbeddedTerminal resize reconciliation — verify/close at Wave-9 gate (S-042 implemented the resize path). |
| SEC-006-CCR-URL-VALIDATION | implementer/security | `ccr_base_url` unvalidated TUI→child env (CWE-20/93, MEDIUM). MUST fix before S-045. |
| DEMO-BINARY-ARTIFACTS-DEVELOP | devops/human-decision | 6+ stories' WEBM demo binaries on develop (incl. S-042). Repo-hygiene policy decision pending. |
| F-S038-EXIT72-ENFORCEMENT | daemon-mode/phase-5 | BC-2.08.006 Inv5/EC-183 daemon exit 72 on hooks-settings write failure; DaemonExit taxonomy missing code-72. Deferred phase-5. |
| F-S038-INV6-PROD-CANON-TEST | test-writer/follow-up | No integration test for BC-2.08.006 Invariant 6 production canonicalization. Non-blocking. |
| F-S035-AC005-DAEMON-BROADCAST | S-039/S-047 | daemon-side ScrollbackChunk* forwarding deferred — TODO tracker in session_manager/mod.rs attach Step 7. |
| PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP | devops | THREE recurrences. Architect must NEVER commit directly to develop. Prompt hardening needed before next story. |

Full register (111+ active tasks): `.factory/cycles/cycle-001/task-register-full.yaml` and `.factory/STATE.md` durable_task_register.

---

## Canonical Version Pins

Canonical source: `.factory/specs/version-pin-registry.yaml` (this table is a snapshot — registry is authoritative)

| Document | Version |
|----------|---------|
| STORY-INDEX | v5.63 |
| sprint-state.yaml | v1.56 |
| wave-schedule.md | v2.1 |
| dependency-graph-expansion.md | v2.10 |
| BC-INDEX | v1.44.5 (138 BCs; 25 v1A) |
| EVAL-INDEX | v1.40 |
| ARCH-INDEX | v1.0.30 |
| SS-ipc | v1.24.0 |
| SS-session-manager | v2.17.1 |
| SS-embedded-pty | v1.14.0 |
| SS-engine-module-v2-delta | v1.6.0 |
| SS-daemon-wiring-v2-delta | v1.12.0 |
| SS-deps-pin-manifest-v2-delta | v1.0.2 |
| SS-config | v1.4.0 |
| prd | v1.28.3 |
| product-brief | v2.0.4 |
| domain-monocle-vision-synthesis | v2.2.3 |
| BC-2.09.006 | v1.3.0 |
| BC-2.09.007 | v1.3.2 |
| BC-2.09.001 | v1.7.3 |
| BC-2.09.002 | v1.2.2 |
| BC-2.09.004 | v1.0.11 |
| BC-2.09.005 | v1.0.7 |
| BC-2.07.002 | v1.1.0 |
| BC-2.08.006 | v1.5.0 |
| BC-2.08.007 | v1.5.6 |
| BC-2.08.008 | v1.3.7 |
| BC-2.04.010 | v1.4.0 |
| S-039 story | v1.8 |
| S-040 story | v1.8 |
| S-042 story | v1.5 |

---

## Known-Flaky Tests (do NOT flag as new findings)

`cli_daemon_stop`, `factory_self_referential`, `test_BC_2_07_006`, wit-bindgen unmatched-skip, PATH isolation flake.
