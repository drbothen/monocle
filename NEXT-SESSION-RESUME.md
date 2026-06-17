# monocle — Resume From Here (Wave-8 Tier-2, D-333, 2026-06-17)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`.

---

## Current Position

- **develop HEAD:** `314326e` (resume-doc commit; squash-merge of S-033 is `c7e10f2`)
- **factory-artifacts HEAD:** `git -C .factory log -1 --format='%h %s'`
- **STATE.md:** v7.96
- **S-034 worktree:** READY at `/Users/jmagady/Dev/monocle/.worktrees/S-034`, branch `story/S-034-kill-session`, base `314326e`, build GREEN
- **10 workspace crates:** monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui, monocle-session-host
- **Stories:** 33/51 done (200/311 pts); Wave 8: 1/12 delivered (S-033)

---

## Human Directive

**Continue Wave-8 Tier-2 autonomously.**

Deliver each Tier-2 story through the full per-story flow (see procedure below) without stopping for human approval between stories. Drive each story to merge.

**Demo default going forward: WEBM + .tape ONLY. NO GIF.** (Curb repo bloat — ~13 MB GIFs already on develop from S-033.)

---

## Wave-8 Tier-2 Plan

### Ready now (depend only on S-033 — DONE)

| Story | Description | Notes |
|-------|-------------|-------|
| **S-034** | `kill_session` + `KillSession` IPC arm | WORKTREE READY — start here |
| **S-035** | `attach_session`/`detach_session` + Attach+Detach IPC arms | |
| **S-038** | Hook auto-injection | |
| **S-045** | `ClaudeCodeModule::spawn_recipe` | BLOCKED by SEC-006-CCR-URL-VALIDATION first |

### Dependencies

- **S-037** (SessionManager GC task): needs S-033 + S-034 — start after S-034 merges
- **S-046**: needs S-032 (S-032 draft; no hard blocker, schedule when ready)
- **S-036**: needs S-033 + S-034 + S-035
- **S-047**: needs S-046
- **S-048**: needs S-022 + S-033 + S-047

### IPC handler arm ownership

**Do NOT add IPC arms outside the owning story.** Each story owns exactly its arms:
- `KillSession` → S-034
- `Attach`/`Detach` → S-035
- `KeyInput`/`Resize`/`Rename` → S-047

### PREREQUISITE before S-045

**SEC-006-CCR-URL-VALIDATION must be fixed before S-045.** `ccr_base_url` flows unvalidated from TUI wire to child env (CWE-20/93, MEDIUM). Fix it in S-045 scope before activating the CCR injection path end-to-end.

### Next concrete action

**S-034 worktree is ready.** Dispatch `stub-architect` for S-034 now.

---

## Per-Story Delivery Procedure

1. **stub-architect** — generate compilable `todo!()` stubs for all files in story scope
2. **test-writer** — write failing tests anchored to BCs; confirm Red Gate (all tests fail)
3. **implementer** — TDD green loop (pick failing test → minimum code → micro-commit)
4. **adversarial convergence** — 3 consecutive CLEAN passes required (Step 4.5); route blockers to implementer; route cross-story integration findings to wave-gate register, not in-scope
5. **demo-recorder** — WEBM + .tape recording (NO GIF); save to `docs/demo-evidence/S-NNN/`
6. **push story branch** — push to remote
7. **pr-manager all 9 steps** — create PR → review dispatch → findings triage → fix delegation → merge to develop
8. **merge** — squash-merge to develop
9. **worktree cleanup** — `git worktree remove .worktrees/S-NNN`

---

## CI-Parity Rules (all 7 apply to every story)

1. `cargo clippy --workspace --all-targets -- -D warnings` **in worktree** (CI uses `--all-targets`; plain `--workspace` misses test targets)
2. From **REPO ROOT** (not worktree): `python3 scripts/check_version_pins.py` (POL-11) AND `python3 scripts/check_structural_claims.py` (POL-12)
3. **No version literals in source doc-comments** — POL-11 will flag them; de-version all citations
4. **Registry atomicity** (L-S027-004): any BC/SS spec version bump + `version-pin-registry.yaml` update = one atomic `factory-artifacts` commit; never commit the spec without updating the registry
5. **Unique `/tmp` paths** per story dispatch — prevents commit-message mixup across concurrent story agents
6. **Architect = spec only** — all code changes go to implementer-in-worktree, never to architect
7. **pr-manager completes all 9 steps** — orchestrator verifies merge via `gh pr view` before declaring done

---

## Branch Protection (11 required CI contexts)

develop branch now requires all 11 status checks before merge:
- 10 from `ci.yml` (build, test, clippy, fmt, deny, semgrep, audit, xtask, etc.)
- 1 DTU fidelity: `DTU Fidelity / DTU fidelity oracle (cargo xtask dtu-fidelity)`

All 11 must be green for a PR to merge. Do not attempt workarounds.

---

## Parked Follow-ups (do NOT fix unless specifically tasked)

| ID | Priority | Anchor | Description |
|----|----------|--------|-------------|
| SEC-006-CCR-URL-VALIDATION | MEDIUM | S-045 | `ccr_base_url` unvalidated TUI→child env (CWE-20/93) — fix BEFORE S-045 |
| F-W8INT-001 | wave-gate | S-021/S-028 | `SessionListUpdate` lacks degraded indicator; I3-009 never reaches TUI |
| F-W8INT-002 | wave-gate | S-022 | `session_list()` unwired from `InitialState`; spawned sessions absent on reconnect |
| F-W8INT-003 | wave-gate | S-018/S-028/S-033 | Two disjoint session sources clobber TUI roster |
| MED-002-monitor-generation-guard | S-036 | S-036 | Post-spawn monitor lacks generation/epoch guard; deferred per Ruling H |
| DEMO-BINARY-ARTIFACTS-DEVELOP | devops/human | develop | ~13 MB demo artifacts on develop; decide keep/remove/git-lfs |
| LOW-S033-SIDECAR-STRUCT-GUARD | devops | CI | Add CI guard forbidding non-`SessionSidecarV3` sidecar structs (POL-12 candidate) |
| ADV-W5GATE-HIGH-002 | implementer | Wave 8+ | Duplicate S-009 handler dead code |
| ADV-W5GATE-MED-001 | implementer | Wave 8+ | S-017 UDS socket spurious WARN on rebind |
| PROC-BRANCH-PROTECTION-CONTEXTS | human | — | Originally pending; now RESOLVED (11 contexts enforced) |

Full register (104+ active tasks): `.factory/cycles/cycle-001/task-register-full.yaml`

---

## Ratified Decisions (do NOT re-litigate)

- **D-238**: session-host-owns-PTY; daemon restart SURVIVES (CASE 2); NO tmux default
- **D-304**: Autonomous Phase-2 dispatch; no per-burst plan-review gate
- **D-315**: Pre-pivot disposition RATIFIED (32 done; 3 active kept; 0 archive)
- **D-325**: Phase-2 gate APPROVED; Phase-3 v1A active
- **D-326..D-331**: S-033 Rulings A–H; adversarial convergence COMPLETE (7 passes, 3 clean)
- **D-332**: S-033 MERGED PR #40 @ c7e10f2
- **D-333**: Wave-8 Tier-2 autonomous delivery authorized; demo WEBM+.tape-no-GIF
- **Spawn-path Model A**: SpawnOptions on wire; SpawnRecipe daemon-internal
- **IPC taxonomy**: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
- **PTY (ADR-0011)**: portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
- **SessionState**: 5 variants (Launching/Running/Detached/Terminating/Terminated) in monocle-ipc

Full history: `.factory/cycles/cycle-001/decisions-archive.md` (D-001..D-333)

---

## Known-Flaky Tests (do NOT flag as new findings)

`cli_daemon_stop`, `factory_self_referential`, `test_BC_2_07_006`, wit-bindgen unmatched-skip, PATH isolation flake.

---

## Canonical Version Pins

Canonical source: `.factory/specs/version-pin-registry.yaml`

| Document | Version |
|----------|---------|
| STORY-INDEX | v5.46 |
| sprint-state.yaml | v1.47 |
| wave-schedule.md | v2.1 |
| dependency-graph-expansion.md | v2.5 |
| BC-INDEX | v1.43.8 (138 BCs; 25 v1A) |
| EVAL-INDEX | v1.19 |
| ARCH-INDEX | v1.0.30 |
| SS-ipc | v1.24.0 |
| SS-session-manager | v2.7.3 |
| SS-embedded-pty | v1.7.0 |
| SS-engine-module-v2-delta | v1.6.0 |
| SS-daemon-wiring-v2-delta | v1.11.4 |
| SS-deps-pin-manifest-v2-delta | v1.0.2 |
| prd | v1.28.3 |
| product-brief | v2.0.4 |
| domain-monocle-vision-synthesis | v2.2.3 |
