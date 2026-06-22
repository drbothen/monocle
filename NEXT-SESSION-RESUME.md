# monocle — Resume From Here (Wave-9 IN PROGRESS — S-039 merged, DTU deadlock resolved, D-341, 2026-06-20)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`.

---

## Current Position

- **develop HEAD:** `3eba172` (fix(ci): DTU fidelity path-filter deadlock, PR #48). Last story commit: `a7ad00e` (S-039, PR #47).
- **factory-artifacts HEAD:** `5b0f748` (state(D-341): PROCESS-GAP-DTU-FIDELITY-PATH-FILTER-DEADLOCK RESOLVED — STATE v8.05→v8.06)
- **STATE.md:** v8.06
- **Stories:** 39/51 done (238/311 pts); Wave 9: 1/6 done (8/42 pts)
- **10 workspace crates:** monocle-core, monocle-runtime, monocle-proto, monocle-test-harness, monocle (binary), monocle-config, monocle-ipc, xtask, monocle-tui, monocle-session-host

---

## What This Session Delivered

**S-039 (PTY output pipeline, 8 pts, EPIC-09, Wave 9, BC-2.09.001) MERGED PR #47 @ a7ad00e (D-340).** 10-pass adversarial convergence (3 consecutive CLEAN: passes 8/9/10). 35 tests. Security review PASS_WITH_NOTES (2 LOW fixed in-scope).

| Story | Pts | PR | SHA | Decision | Notes |
|-------|-----|----|-----|----------|-------|
| **S-039** PTY output pipeline | 8 | #47 | a7ad00e | D-340 | IPC PtyOutput → vt100::Parser → tui-term PseudoTerminal render. Auto-attach-on-first-entry buffering/replay. Per-session parser lifecycle (create/GC via shared gc_session_with_mode_exit). Bounded buffer (512KiB / 4096-msg cap, drop-oldest + drop counter) + 10s dump-window timeout force-resolve. Reconnect cleanup. Spec evolution: BC-2.09.001 →1.7.2 (Inv-3/4/5/7/8/9, EC-200..208), BC-2.09.007 →1.3.1, SS-embedded-pty →1.10.0, SS-config →1.4.0, BC-2.07.002 →1.1.0 (pty_scrollback_rows: Option), S-039 story →1.8. |
| **fix(ci)** DTU deadlock | — | #48 | 3eba172 | D-341 | PROCESS-GAP-DTU-FIDELITY-PATH-FILTER-DEADLOCK RESOLVED: dtu-fidelity.yml no longer path-filters the pull_request trigger; internal pure-bash change-detection gate runs real oracle when DTU-relevant paths change, reports success-skip (exit 0) otherwise. Job name "DTU fidelity oracle (cargo xtask dtu-fidelity)" unchanged (required-context byte-identical). S-039 required a one-time HUMAN-AUTHORIZED admin merge (predated fix); all future TUI-only PRs (S-040/S-042/S-043) now merge with NO admin bypass. |

---

## Human Directive

**Continue autonomously** into subsequent ready stories until told to stop. Per-story flow + 3 consecutive CLEAN adversarial convergence + clean merge applies to each story.

**Demo default: WEBM + .tape ONLY. NO GIF.**

---

## Next-Action Queue

1. **S-040** (keyboard-forwarding, Wave 9) — **UNBLOCKED** (deps S-021 + S-025 + S-039 all merged). No DTU deadlock risk (RESOLVED D-341). Read the S-040 story + its BCs and confirm scope before dispatching stub-architect. **START HERE.**

2. **S-042** (resize-debounce) and **S-043** (scrollback-navigation) — Wave 9, depend on S-039 (now merged). S-042 owns embedded-terminal pane sizing / pty_scroll_offsets reset (BC-2.09.006). F-S039-P9-OBS-001 (S-039 render uses sessions-pane placeholder area at 24x80) is parked for the Wave-9 integration gate — S-042 reconciles.

3. **SEC-006-CCR-URL-VALIDATION** (CWE-20/93): `ccr_base_url` flows unvalidated from TUI wire to child env. **MUST be addressed before S-045.**

4. S-046 waits on S-032; S-047 needs S-033 + S-034 + S-035 + S-046; S-048 needs S-022 + S-033 + S-047.

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
6. **Architect = spec only** — all code changes go to implementer-in-worktree (PROCESS-GAP-ARCHITECT-CODE-ON-DEVELOP)
7. **pr-manager completes all 9 steps** — orchestrator verifies merge via `gh pr view` before declaring done (PROCESS-GAP-PRMANAGER-EARLY-RETURN)
8. **factory-artifacts push verification** (PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED, D-337): after every spec-bumping agent dispatch, orchestrator MUST verify `git -C .factory log origin/factory-artifacts..HEAD` is EMPTY and push immediately if not

**DTU deadlock RESOLVED (D-341):** non-DTU-path PRs now report the DTU oracle via the skip-success path; no admin bypass needed. The 11 required develop contexts are unchanged (bare names).

**B002 build-order note:** 2 B002 integration tests (`test_BC_2_08_001_B002_*`) require the `monocle-session-host` binary at `target/debug/deps/`. Run `cargo build --workspace` first; they PASS in CI (which builds the binary). Failing bare `cargo test --workspace` locally without prior build is NOT a regression.

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
- **D-334**: S-034 MERGED PR #41 @ 4dfe0db. Kill path complete. Rulings H/I/J/K. <!-- version-pin-historical: SS-session-manager v2.11.0, BC-2.08.003 v1.5.0, BC-2.08.008 v1.3.5 at D-334 checkpoint -->
- **D-335**: S-037 MERGED PR #42 @ a7e4081. GC + rename_session. SEC-001/002 fixed in-scope. BRANCH-PROTECTION RESOLVED.
- **D-336**: S-035 MERGED PR #43 @ 270b7d4. attach/detach. Ruling L (proxy_task kill-reader for attached sessions). S-036 UNBLOCKED.
- **D-337**: PROCESS-GAP-FACTORY-ARTIFACTS-NOT-PUSHED codified (c8ceee1). Always push factory-artifacts after spec-bumping agents.
- **D-338**: S-038 MERGED PR #44 @ 8d649ea + chore PR #45 @ 7f005af. WAVE-8 TIER-2 COMPLETE. Single-writer mandate (BC-2.08.006 at S-038 delivery; lock.app mandatory). SEC-001 CWE-732 + SEC-002 CWE-532 fixed in-scope (daeb4f2).
- **D-339**: S-036 MERGED PR #46 @ a7ad00e (setsid persistence; all states handled within 5s; UDS bind blocked). WAVE-8 TIER-3 COMPLETE. <!-- version-pin-historical: BC-2.08.002/BC-2.08.004 at D-339 checkpoint -->
- **D-340**: S-039 MERGED PR #47 @ a7ad00e. PTY output pipeline. IPC PtyOutput→vt100→tui-term render. Auto-attach buffering/replay. Bounded buffer + dump-window timeout. Security review PASS_WITH_NOTES (2 LOW fixed in-scope). 10-pass adversarial convergence (3 CLEAN).
- **D-341**: DTU fidelity path-filter deadlock RESOLVED (PR #48 @ 3eba172). dtu-fidelity.yml pure-bash internal gate; required-context name unchanged. Future TUI-only PRs merge with no admin bypass.
- **D-342**: S-040 MERGED PR #50 @ d230a26. Full-Fidelity Keyboard Forwarding. 17-pass adversarial (3 CLEAN). NO admin bypass.
- **D-343**: Zero-context durability checkpoint. S-042 stubs @ 40dd53a (local worktree). Test-writer next.
- **D-344**: Human ruling — S-042 expanded to full end-to-end resize pipeline. `ClientToServer` is exhaustive (no `#[non_exhaustive]`, no wildcard arm). S-047 draft/undelivered. S-042 owns: TUI detection/debounce, daemon `ResizePane` routing → `resize_session()`, zero-dim clamp, `DaemonToHost::Resize` forwarding, session-host `pty.resize()` + `parser.set_size()`. BC-2.09.006 v1.3.0, S-042 v1.5 (8 pts), SS-session-manager v2.17.0 at D-344 authoring time (patched to v2.17.1 by F-S042-ADV-MED-001), S-047 v1.7 (ResizePane removed).
- **Spawn-path Model A**: SpawnOptions on wire; SpawnRecipe daemon-internal
- **IPC taxonomy**: 12-code wire taxonomy; 9-variant SessionError; schema_version 3
- **PTY (ADR-0011)**: portable-pty 0.9.0 + vt100 0.16.2 + tui-term =0.3.4; MSRV 1.88
- **SessionState**: 5 variants (Launching/Running/Detached/Terminating/Terminated) in monocle-ipc

Full history: `.factory/cycles/cycle-001/decisions-archive.md` (D-001..D-341)

---

## Open Durable Follow-ups (do NOT fix unless specifically tasked)

| ID | Route | Description |
|----|-------|-------------|
| F-S039-P9-OBS-001 | wave-gate/S-042 | EmbeddedTerminal render uses sessions-pane placeholder area (24x80); pane sizing/resize reconciliation is S-042/BC-2.09.006. Surface at Wave-9 integration gate. |
| SEC-006-CCR-URL-VALIDATION | implementer/security | `ccr_base_url` unvalidated TUI→child env (CWE-20/93, MEDIUM). MUST fix before S-045. |
| DEMO-BINARY-ARTIFACTS-DEVELOP | devops/human | 6 stories' WEBM demo binaries now on develop (incl. S-039). Repo-hygiene policy decision pending. |
| F-S038-EXIT72-ENFORCEMENT | daemon-mode/phase-5 | BC-2.08.006 Inv5/EC-183 mandate daemon exit 72 on hooks-settings write failure; DaemonExit taxonomy missing code-72 variant. Pre-existing; deferred phase-5 or dedicated story. |
| F-S038-INV6-PROD-CANON-TEST | test-writer/follow-up | No integration test for BC-2.08.006 Invariant 6 production canonicalization. Non-blocking. |
| F-S035-AC005-DAEMON-BROADCAST | S-039/S-047 | daemon-side ScrollbackChunk* forwarding deferred — TODO tracker in session_manager/mod.rs attach Step 7. |
| F-S035-LAUNCHING-CONN-DETACH-MATRIX | architect | detach-on-Launching-WITH-established-host_conn matrix wording ambiguity. Non-blocking. |
| PROCESS-GAP-ARCHITECT-NO-COMMIT | devops | Architect agent recurred 2x leaving spec+registry uncommitted (S-034); held stable in S-035 (D-336 positive obs). Codify atomic commit obligation. |

Full register (108+ active tasks): `.factory/cycles/cycle-001/task-register-full.yaml` and `.factory/STATE.md` durable_task_register.

---

## Canonical Version Pins

Canonical source: `.factory/specs/version-pin-registry.yaml` (this table is a snapshot — registry is authoritative)

| Document | Version |
|----------|---------|
| STORY-INDEX | v5.55 |
| sprint-state.yaml | v1.53 |
| wave-schedule.md | v2.1 |
| dependency-graph-expansion.md | v2.9 |
| BC-INDEX | v1.43.8 (138 BCs; 25 v1A) |
| EVAL-INDEX | v1.25 |
| ARCH-INDEX | v1.0.30 |
| SS-ipc | v1.24.0 |
| SS-session-manager | v2.15.0 |
| SS-embedded-pty | v1.10.0 |
| SS-engine-module-v2-delta | v1.6.0 |
| SS-daemon-wiring-v2-delta | v1.12.0 |
| SS-deps-pin-manifest-v2-delta | v1.0.2 |
| SS-config | v1.4.0 |
| prd | v1.28.3 |
| product-brief | v2.0.4 |
| domain-monocle-vision-synthesis | v2.2.3 |
| BC-2.07.002 | v1.1.0 |
| BC-2.08.006 | v1.5.0 |
| BC-2.08.007 | v1.5.6 |
| BC-2.08.008 | v1.3.7 |
| BC-2.04.010 | v1.4.0 |
| BC-2.09.001 | v1.7.2 |
| BC-2.09.007 | v1.3.1 |
| S-039 story | v1.8 |

---

## Known-Flaky Tests (do NOT flag as new findings)

`cli_daemon_stop`, `factory_self_referential`, `test_BC_2_07_006`, wit-bindgen unmatched-skip, PATH isolation flake.
