# monocle — Resume From Here (Wave-8 Tier-2, D-334, 2026-06-18)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`.

---

## Current Position

- **develop HEAD:** `4dfe0db` (S-034 merged via PR #41)
- **factory-artifacts HEAD:** `eabcdbf` (D-334 state-burst)
- **STATE.md:** v7.98
- **S-034 DONE:** `SessionManager::kill_session` — DaemonToHost::Kill within 500ms; Terminating/Terminated transitions; 12s watchdog. 18-pass adversarial convergence (3 CLEAN). PR #41 merged 2026-06-18.
- **10 workspace crates:** monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui, monocle-session-host
- **Stories:** 34/51 done (208/311 pts); Wave 8: 2/12 delivered (S-033 + S-034, 16/74 pts)

---

## Human Directive

**Continue Wave-8 Tier-2 autonomously.**

Deliver each Tier-2 story through the full per-story flow (see procedure below) without stopping for human approval between stories. Drive each story to merge.

**Demo default going forward: WEBM + .tape ONLY. NO GIF.** (Curb repo bloat — demo artifacts already on develop from S-033/S-034.)

---

## CRITICAL: Branch Protection Check-Name Mismatch

**PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH** — develop branch protection requires check names with the `CI /` workflow prefix (e.g., `CI / Preflight (toolchain + fmt + lint)`), but GitHub Actions emits BARE names (`Preflight (...)`). Every PR shows `mergeStateStatus=BLOCKED` despite all 11 checks passing. Merges currently require admin override (`enforce_admins=false`), as done for PR #40 and PR #41.

**Recommended fix before more Wave-8 PRs:** Update branch-protection required-check contexts to drop the `CI /` prefix so PRs merge without admin override. This is a devops/human action; route to devops-engineer or the human. Do NOT attempt workarounds that skip hooks or bypass protection rules.

---

## Wave-8 Tier-2 Plan

### Ready now (S-033 + S-034 done)

| Story | Pts | Epic | Ready? | Notes |
|-------|-----|------|--------|-------|
| **S-037** | 3 | EPIC-08 | UNBLOCKED (S-033+S-034 done) | SessionManager GC — reap Terminated sessions. **Start here.** |
| **S-035** | 8 | EPIC-08 | UNBLOCKED (S-033+S-034 done) | `attach_session`/`detach_session` + Attach+Detach IPC arms |
| **S-038** | 3 | EPIC-08 | UNBLOCKED (S-033 done) | Hook auto-injection |

### Blocked / deferred

| Story | Blocker |
|-------|---------|
| **S-045** | SEC-006-CCR-URL-VALIDATION MUST be fixed first (CWE-20/93, `ccr_base_url` unvalidated TUI→child env). Fix in S-045 scope. |
| **S-036** | Needs S-033 + S-034 + S-035 |
| **S-046** | Needs S-032 (draft; no hard blocker, schedule when ready) |
| **S-047** | Needs S-046 |
| **S-048** | Needs S-022 + S-033 + S-047 |

### IPC handler arm ownership

**Do NOT add IPC arms outside the owning story.** Each story owns exactly its arms:
- `KillSession` → S-034 (DONE)
- `Attach`/`Detach` → S-035
- `KeyInput`/`Resize`/`Rename` → S-047

### Next concrete action

**Dispatch S-037** (GC, 3 pts) via the full per-story flow. S-035 (8 pts) and S-038 (3 pts) are also unblocked if you prefer to sequence differently.

---

## Per-Story Delivery Procedure

1. **stub-architect** — generate compilable `todo!()` stubs for all files in story scope
2. **test-writer** — write failing tests anchored to BCs; confirm Red Gate (all tests fail)
3. **implementer** — TDD green loop (pick failing test → minimum code → micro-commit)
4. **Step 4.5 adversarial convergence** — 3 consecutive CLEAN passes required; route blockers to implementer; route cross-story integration findings to wave-gate register, not in-scope
5. **demo-recorder** — WEBM + .tape recording (NO GIF); save to `docs/demo-evidence/S-NNN/`
6. **push story branch** — push to remote
7. **pr-manager all 9 steps** — create PR → review dispatch → findings triage → fix delegation → convergence → merge to develop
8. **merge** — squash-merge to develop (currently requires admin override — see branch-protection caveat above)
9. **worktree cleanup** — `git worktree remove .worktrees/S-NNN`

---

## CI-Parity Rules (all 7 apply to every story)

1. `cargo clippy --workspace --all-targets -- -D warnings` **in worktree** (CI uses `--all-targets`; plain `--workspace` misses test targets — PROCESS-GAP-CI-PARITY-1)
2. From **REPO ROOT** (not worktree): `python3 scripts/check_version_pins.py` (POL-11) AND `python3 scripts/check_structural_claims.py` (POL-12)
3. **No version literals in source doc-comments** — POL-11 will flag them; de-version all citations
4. **Registry atomicity** (L-S027-004): any BC/SS spec version bump + `version-pin-registry.yaml` update = one atomic `factory-artifacts` commit; never commit the spec without updating the registry
5. **Unique `/tmp` paths** per story dispatch — prevents commit-message mixup across concurrent story agents
6. **Architect = spec only** — all code changes go to implementer-in-worktree, never to architect (PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP)
7. **pr-manager completes all 9 steps** — orchestrator verifies merge via `gh pr view` before declaring done (PROCESS-GAP-PRMANAGER-EARLY-RETURN)

---

## Branch Protection (11 required CI contexts)

develop branch requires all 11 status checks before merge:
- 10 from `ci.yml`: Preflight (toolchain + fmt + lint), Build+Test (stable x86_64), Build+Test (stable aarch64), Build+Test (MSRV 1.88), Daemon E2E, POL-11 version-pin freshness, POL-12 structural claims, POL-14 anchor-pin freshness, Semgrep SAST, cargo audit + cargo deny
- 1 from `dtu-fidelity.yml`: DTU Fidelity oracle

**Caveat:** due to PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH, merges currently require admin override even when all 11 pass.

---

## Ratified Decisions (do NOT re-litigate)

- **D-238**: session-host-owns-PTY; daemon restart SURVIVES (CASE 2); NO tmux default
- **D-304**: Autonomous Phase-2 dispatch; no per-burst plan-review gate
- **D-315**: Pre-pivot disposition RATIFIED (32 done; 3 active kept; 0 archive)
- **D-325**: Phase-2 gate APPROVED; Phase-3 v1A active
- **D-326..D-331**: S-033 Rulings A–H; adversarial convergence COMPLETE (7 passes, 3 clean)
- **D-332**: S-033 MERGED PR #40 @ c7e10f2
- **D-333**: Wave-8 Tier-2 autonomous delivery authorized; demo WEBM+.tape-no-GIF
- **D-334**: S-034 MERGED PR #41 @ 4dfe0db. Kill path complete. SS-session-manager v2.11.0, BC-2.08.003 v1.5.0, BC-2.08.008 v1.3.5.
- **SS-session-manager Rulings H/I/J/K** (kill-path): H = accept-loop; I = kill_confirm_monitor reader-ownership; J = watchdog dual-PID SIGKILL (BC-2.08.003 PC-5b); K = natural-exit Terminated ownership = S-039/S-040, NOT S-034 scope (BC-2.08.008 PC-6 deferred)
- **Spawn-path Model A**: SpawnOptions on wire; SpawnRecipe daemon-internal
- **IPC taxonomy**: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
- **PTY (ADR-0011)**: portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
- **SessionState**: 5 variants (Launching/Running/Detached/Terminating/Terminated) in monocle-ipc

Full history: `.factory/cycles/cycle-001/decisions-archive.md` (D-001..D-334)

---

## Open Durable Follow-ups (do NOT fix unless specifically tasked)

| ID | Route | Description |
|----|-------|-------------|
| PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH | devops/human | Branch protection check-name prefix mismatch; admin-override required per PR. Fix before Wave-8 continues. |
| PROCESS-GAP-ARCHITECT-NO-COMMIT | devops | Architect agent leaves spec+registry uncommitted (recurred 2x in S-034). Codify atomic commit obligation. |
| SEC-006-CCR-URL-VALIDATION | implementer/security | `ccr_base_url` unvalidated TUI→child env (CWE-20/93, MEDIUM). MUST fix before S-045. |
| BC-2.08.008-PC6-NATURAL-EXIT | S-039/S-040 | Natural-child-exit watch (PTY EOF → Terminated broadcast without Kill). BC-2.08.008 PC-6. Deferred Ruling K. |
| DEMO-BINARY-ARTIFACTS-DEVELOP | devops/human | Demo GIF/WEBM artifacts on develop; repo hygiene decision pending. |
| F-W8INT-001/002/003 | wave-gate | Three cross-story integration findings parked for Wave-8 integration gate. Do NOT re-raise as in-scope findings during per-story delivery. |
| MED-002-monitor-generation-guard | S-036 | Post-spawn monitor generation/epoch guard. Deferred per Ruling H. |

Full register (107+ active tasks): `.factory/cycles/cycle-001/task-register-full.yaml` and `.factory/STATE.md` durable_task_register.

---

## Known-Flaky Tests (do NOT flag as new findings)

`cli_daemon_stop`, `factory_self_referential`, `test_BC_2_07_006`, wit-bindgen unmatched-skip, PATH isolation flake.

**Additional note:** 2 B002 integration tests require the `monocle-session-host` binary at `target/debug/deps/`. These pass in CI (which builds the binary). Running bare `cargo test --workspace` locally without first running `cargo build --workspace` will fail these tests with a "binary not found" error — this is NOT a regression, just a build-order dependency.

---

## Canonical Version Pins

Canonical source: `.factory/specs/version-pin-registry.yaml`

| Document | Version |
|----------|---------|
| STORY-INDEX | v5.47 |
| sprint-state.yaml | v1.48 |
| wave-schedule.md | v2.1 |
| dependency-graph-expansion.md | v2.6 |
| BC-INDEX | v1.43.8 (138 BCs; 25 v1A) |
| EVAL-INDEX | v1.20 |
| ARCH-INDEX | v1.0.30 |
| SS-ipc | v1.24.0 |
| SS-session-manager | v2.11.0 |
| SS-embedded-pty | v1.7.0 |
| SS-engine-module-v2-delta | v1.6.0 |
| SS-daemon-wiring-v2-delta | v1.11.4 |
| SS-deps-pin-manifest-v2-delta | v1.0.2 |
| prd | v1.28.3 |
| product-brief | v2.0.4 |
| domain-monocle-vision-synthesis | v2.2.3 |
| S-034 story | v1.3 |
| BC-2.08.003 | v1.5.0 |
| BC-2.08.008 | v1.3.5 |
