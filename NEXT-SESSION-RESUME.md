# monocle — Resume From Here (Wave-8 Tier-2 COMPLETE, D-338, 2026-06-19)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`.

---

## Current Position

- **develop HEAD:** `7f005af` (chore PR #45 — BC-2.08.007 comment pin v1.5.5→v1.5.6)
- **factory-artifacts HEAD:** `078b21c` (STATE.md v8.03 — zero-context checkpoint)
- **STATE.md:** v8.03
- **Stories:** 37/51 done (222/311 pts); Wave 8: 5/12 delivered (30/74 pts)
- **10 workspace crates:** monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui, monocle-session-host

---

## What This Session Delivered

**Branch protection RESOLVED** (D-335, human-authorized): develop's 11 required-status-check contexts changed from `CI /`-prefixed names to the BARE names GitHub Actions emits. Wave-8 PRs now merge CLEAN with no admin override. `PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH = RESOLVED`.

| Story | Pts | PR | SHA | Decision | Notes |
|-------|-----|----|-----|----------|-------|
| **S-037** GC + rename_session | 3 | #42 | a7e4081 | D-335 | SEC-001 CWE-20 + SEC-002 CWE-706 fixed in-scope (b2d65db). 7-pass adversarial (3 CLEAN). |
| **S-035** attach/detach + Ruling L | 8 | #43 | 270b7d4 | D-336 | SS-session-manager → v2.14.0 (Ruling L + EC-188 4-disposition matrix). BC-2.08.007 v1.5.5 / BC-2.08.008 v1.3.7. 9-pass adversarial (3 CLEAN). |
| **S-038** hook auto-injection, single-writer | 3 | #44 | 8d649ea | D-338 | BC-2.08.006 → v1.5.0; BC-2.04.010 → v1.4.0; SS-session-manager → v2.15.0; BC-2.08.007 → v1.5.6. SEC-001 CWE-732 + SEC-002 CWE-532 fixed in-scope (daeb4f2). 6-pass adversarial (3 CLEAN). |
| **chore** BC-2.08.007 comment pin | — | #45 | 7f005af | — | Preserved human commit 4638006 via proper PR, not admin push. |

**New process-gap codified** — `PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED` (D-337 @ c8ceee1): S-038 cycle spec commits were committed to `.factory` worktree but never pushed; origin/factory-artifacts was 4 commits stale; POL-11 CI check validated against stale registry. **RULE:** orchestrator MUST verify `git -C .factory log origin/factory-artifacts..HEAD` is EMPTY after every spec-bumping agent dispatch.

---

## Human Directive

**Continue autonomously** into Wave-8 Tier-3 (S-036) and subsequent ready stories until told to stop. Per-story flow + 3 consecutive CLEAN adversarial convergence + clean merge applies to each story.

**Demo default: WEBM + .tape ONLY. NO GIF.**

---

## Next-Action Queue

1. **S-036** (rediscover_sessions, 8 pts, EPIC-08, SS-08) — **UNBLOCKED** (needs S-033 + S-034 + S-035, all done). Owns the deferred items:
   - S-037 AC-007/AC-010: orphaned-sidecar IMMEDIATE GC (no grace period)
   - S-035 AC-008: restart restores Detached state, NOT force-attach
   - `MED-002-monitor-generation-guard`: post-spawn monitor generation/epoch guard (Ruling H — deferred to S-036)
   - Read the BC for S-036 and confirm exact scope before dispatching stub-architect.
   **START HERE.**

2. **S-039 + S-040** (PTY output pipeline, Wave 9) — own `ScrollbackChunk` content source + natural-child-exit (`BC-2.08.008-PC6`). S-038 AC-005 daemon `ScrollbackChunk` broadcast forwarding deferred to S-039/S-047 via `TODO(S-039/S-047)` tracker in `session_manager/mod.rs`.

3. **SEC-006-CCR-URL-VALIDATION** (CWE-20/93): `ccr_base_url` flows unvalidated from TUI wire to child env. **MUST be addressed before S-045** (ClaudeCodeModule::spawn_recipe activates CCR URL injection path end-to-end).

4. S-046 waits on S-032; S-047 needs S-033 + S-034 + S-035 + S-046; S-048 needs S-022 + S-033 + S-047.

5. F-W8INT-001/002/003 — parked for Wave-8 integration gate. Do NOT re-raise as in-scope findings during per-story delivery.

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

## CI-Parity Rules (all 8 apply — rule 8 is NEW this session)

1. `cargo clippy --workspace --all-targets -- -D warnings` **in worktree** (CI uses `--all-targets`; plain `--workspace` misses test targets — PROCESS-GAP-CI-PARITY-1)
2. From **REPO ROOT** (not worktree): `python3 scripts/check_version_pins.py` (POL-11) AND `python3 scripts/check_structural_claims.py` (POL-12)
3. **No version literals in source doc-comments or test prose** — POL-11 will flag them; de-version all citations; historical snapshots use `<!-- version-pin-historical: ... -->` HTML comment
4. **Registry atomicity** (L-S027-004): any BC/SS spec version bump + `version-pin-registry.yaml` update = **one atomic** `factory-artifacts` commit; cascade STORY-INDEX/EVAL-INDEX/dependency-graph pins in same commit; NEVER commit the spec without updating the registry
5. **Unique `/tmp` paths** per story dispatch — prevents commit-message mixup across concurrent story agents
6. **Architect = spec only** — all code changes go to implementer-in-worktree (PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP)
7. **pr-manager completes all 9 steps** — orchestrator verifies merge via `gh pr view` before declaring done (PROCESS-GAP-PRMANAGER-EARLY-RETURN)
8. **NEW — factory-artifacts push verification** (PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED, D-337): after every spec-bumping agent dispatch, orchestrator MUST verify `git -C .factory log origin/factory-artifacts..HEAD` is EMPTY and push immediately if not

**B002 build-order note:** 2 B002 integration tests (`test_BC_2_08_001_B002_*`) require the `monocle-session-host` binary at `target/debug/deps/`. Run `cargo build --workspace` first; they PASS in CI (which builds the binary). Failing bare `cargo test --workspace` locally without prior build is NOT a regression.

---

## Branch Protection (11 required CI contexts — FIXED, bare names)

develop branch requires all 11 status checks before merge (no admin override needed):
- 10 from `ci.yml`: `Preflight (toolchain + fmt + lint)`, `Build+Test (stable x86_64)`, `Build+Test (stable aarch64)`, `Build+Test (MSRV 1.88)`, `Daemon E2E`, `POL-11 version-pin freshness`, `POL-12 structural claims`, `POL-14 anchor-pin freshness`, `Semgrep SAST`, `cargo audit + cargo deny`
- 1 from `dtu-fidelity.yml`: `DTU Fidelity oracle`

PROCESS-GAP-BRANCH-PROTECTION-CHECK-NAME-MISMATCH is RESOLVED (D-335). PRs merge CLEAN.

---

## Ratified Decisions (do NOT re-litigate)

- **D-238**: session-host-owns-PTY; daemon restart SURVIVES (CASE 2); NO tmux default
- **D-304**: Autonomous Phase-2 dispatch; no per-burst plan-review gate
- **D-315**: Pre-pivot disposition RATIFIED (32 done; 3 active kept; 0 archive)
- **D-325**: Phase-2 gate APPROVED; Phase-3 v1A active
- **D-326..D-331**: S-033 Rulings A–H; adversarial convergence COMPLETE (7 passes, 3 clean)
- **D-332**: S-033 MERGED PR #40 @ c7e10f2
- **D-333**: Wave-8 Tier-2 autonomous delivery authorized; demo WEBM+.tape-no-GIF
- **D-334**: S-034 MERGED PR #41 @ 4dfe0db. Kill path complete. Rulings H/I/J/K. <!-- version-pin-historical: SS-session-manager v2.11.0, BC-2.08.003 v1.5.0, BC-2.08.008 v1.3.5 at D-334 checkpoint -->
- **D-335**: S-037 MERGED PR #42 @ a7e4081. GC + rename_session. SEC-001/002 fixed in-scope. BRANCH-PROTECTION RESOLVED.
- **D-336**: S-035 MERGED PR #43 @ 270b7d4. attach/detach. Ruling L (proxy_task kill-reader for attached sessions). S-036 UNBLOCKED.
- **D-337**: PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED codified (c8ceee1). Always push factory-artifacts after spec-bumping agents.
- **D-338**: S-038 MERGED PR #44 @ 8d649ea + chore PR #45 @ 7f005af. WAVE-8 TIER-2 COMPLETE. Single-writer mandate (BC-2.08.006 v1.5.0; lock.app mandatory). SEC-001 CWE-732 + SEC-002 CWE-532 fixed in-scope (daeb4f2).
- **Spawn-path Model A**: SpawnOptions on wire; SpawnRecipe daemon-internal
- **IPC taxonomy**: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
- **PTY (ADR-0011)**: portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
- **SessionState**: 5 variants (Launching/Running/Detached/Terminating/Terminated) in monocle-ipc

Full history: `.factory/cycles/cycle-001/decisions-archive.md` (D-001..D-338)

---

## Open Durable Follow-ups (do NOT fix unless specifically tasked)

| ID | Route | Description |
|----|-------|-------------|
| SEC-006-CCR-URL-VALIDATION | implementer/security | `ccr_base_url` unvalidated TUI→child env (CWE-20/93, MEDIUM). MUST fix before S-045. |
| F-S038-EXIT72-ENFORCEMENT | daemon-mode/phase-5 | BC-2.08.006 Inv5/EC-183 mandate daemon exit 72 on hooks-settings write failure; DaemonExit taxonomy missing code-72 variant. Pre-existing; deferred phase-5 or dedicated story. |
| F-S038-INV6-PROD-CANON-TEST | test-writer/follow-up | No integration test for BC-2.08.006 Invariant 6 production canonicalization. Non-blocking. |
| F-S038-TRACING-TEST-DOC-DEVERSION | implementer | [LOW] doc-comment at session_manager/mod.rs ~line 11621 embeds tracing-test version literal. De-version in future pass. |
| F-S035-AC005-DAEMON-BROADCAST | S-039/S-047 | daemon-side ScrollbackChunk* forwarding deferred — TODO tracker in session_manager/mod.rs attach Step 7. |
| F-S035-LAUNCHING-CONN-DETACH-MATRIX | architect | detach-on-Launching-WITH-established-host_conn matrix wording ambiguity. Non-blocking. |
| DEMO-BINARY-ARTIFACTS-DEVELOP | devops/human | 5 stories' WEBM demo binaries on develop (~6 MB+ per story). Repo-hygiene policy decision pending. |
| F-W8INT-001/002/003 | wave-gate/architect | Three cross-story integration findings: (1) EnrichedSession degraded indicator never reaches TUI; (2) SessionManager sessions unwired from InitialState on reconnect; (3) Two disjoint session sources mutually clobber TUI roster. Surface at Wave-8 integration gate only. |
| MED-002-monitor-generation-guard | S-036 | Post-spawn monitor generation/epoch guard. Deferred per Ruling H. |
| BC-2.08.008-PC6-NATURAL-EXIT | S-039/S-040 | Natural-child-exit watch (PTY EOF → Terminated broadcast without Kill). BC-2.08.008 PC-6 / Ctrl-D canonical test vector. |
| PROCESS-GAP-ARCHITECT-NO-COMMIT | devops | Architect agent recurred 2x leaving spec+registry uncommitted (S-034); held stable in S-035 (D-336 positive obs). Codify atomic commit obligation. |

Full register (108+ active tasks): `.factory/cycles/cycle-001/task-register-full.yaml` and `.factory/STATE.md` durable_task_register.

---

## Canonical Version Pins

Canonical source: `.factory/specs/version-pin-registry.yaml`

| Document | Version |
|----------|---------|
| STORY-INDEX | v5.50 |
| sprint-state.yaml | v1.51 |
| wave-schedule.md | v2.1 |
| dependency-graph-expansion.md | v2.9 |
| BC-INDEX | v1.43.8 (138 BCs; 25 v1A) |
| EVAL-INDEX | v1.25 |
| ARCH-INDEX | v1.0.30 |
| SS-ipc | v1.24.0 |
| SS-session-manager | v2.15.0 |
| SS-embedded-pty | v1.7.0 |
| SS-engine-module-v2-delta | v1.6.0 |
| SS-daemon-wiring-v2-delta | v1.12.0 |
| SS-deps-pin-manifest-v2-delta | v1.0.2 |
| prd | v1.28.3 |
| product-brief | v2.0.4 |
| domain-monocle-vision-synthesis | v2.2.3 |
| BC-2.08.006 | v1.5.0 |
| BC-2.08.007 | v1.5.6 |
| BC-2.08.008 | v1.3.7 |
| BC-2.04.010 | v1.4.0 |
| S-038 story | v1.5 |

---

## Known-Flaky Tests (do NOT flag as new findings)

`cli_daemon_stop`, `factory_self_referential`, `test_BC_2_07_006`, wit-bindgen unmatched-skip, PATH isolation flake.
