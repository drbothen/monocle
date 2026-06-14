# monocle — Resume From Here (D-270, 2026-06-13)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.21, for the full checkpoint).

---

## Status at Pause

**Phase-1d adversarial spec convergence is in progress.** 28 passes complete.
Consecutive-clean counter = 0. Pass-29 is the next dispatch (clean candidate 1 of 3).

develop branch is UNCHANGED at 8bc22a5. No production code has been written for v1A.
All pivot work lives on the factory-artifacts branch under `.factory/`.

---

## The Most Recent Major Change: Model-A Spawn Path (Pass-27, D-269)

Pass-27 adjudicated the SPAWN-PATH MODEL A decision — the most significant architectural
decision of the convergence cycle:

- `spawn_session(opts: SpawnOptions)` is the canonical signature (SS-session-manager v2.1.0, SpawnOptions constructors added)
- `SpawnOptions` is the IPC wire type (`#[non_exhaustive]` + Serialize/Deserialize)
- `SpawnRecipe` is daemon-internal only (never on the wire, never at the call site)
- `ClientToServer::SpawnSession` carries `opts: SpawnOptions`
- Daemon calls `spawn_recipe()` internally inside `spawn_session()`
- `EngineError::BinaryNotFound`→`binary_not_found` / `InvalidPath`→`invalid_spawn_arg` are reachable via `SessionError::EngineError #[from]` bridge (BC-2.03.007 PC-3/PC-7)

Pass-28 (D-270) fixed a survivor of this change: HS-EXP-014 Step 1 still used the retired
Model-B `spawn_session(recipe_A, harness_A, profile_A)` signature. Fixed to `spawn_session(opts_A)`.
Sibling holdouts HS-EXP-011/012/013/015 confirmed clean.

---

## Next Action: Adversarial Pass 29

Dispatch `vsdd-factory:adversary` in a fresh context for Pass 29.
Feed the full spec package (section D of `next_session_resume_protocol` in STATE.md).

The human's directive: **strict 3 consecutive clean passes** (zero Critical + zero Important).
Do NOT accept fewer than 3. Do NOT resume Phase 4-7 of the old observe-only scope.

Finding trajectory summary (C/I counts per pass):
- Passes 1-6: Critical present (5/8, 5/6, 4/9, 1/4, 2/4, 2/2)
- Passes 7-28: ALL zero-Critical (22 consecutive)
- Pass 20: 0C/0I — FIRST CLEAN
- Pass 21: 0C/0I — SECOND CLEAN
- Pass 22: 0C/3I — RESET counter 2→0 (sibling-BC cluster caught)
- Passes 23-28: 0C/1I each — counter stays 0

---

## Current Spec Package Headline Versions

| Document | Version |
|----------|---------|
| domain-monocle-vision-synthesis.md | v2.2.3 (APPROVED) |
| product-brief.md | v2.0.4 |
| prd.md | v1.28.3 |
| ARCH-INDEX | v1.0.28 |
| SS-ipc | v1.19.0 |
| SS-session-manager | v2.0.0 (MAJOR — spawn_session(opts)) |
| SS-embedded-pty | v1.5.1 |
| SS-engine-module-v2-delta | v1.2.0 |
| SS-daemon-wiring-v2-delta | v1.8.0 |
| SS-deps-pin-manifest-v2-delta | v1.0.1 |
| ADR-0009 | v1.0.2 |
| ADR-0010 | v1.6.0 |
| ADR-0011 | v1.2.1 |
| BC-INDEX | v1.40.0 (138 BCs) |
| EVAL-INDEX | v1.15 |
| version-pin-registry.yaml | source of truth |

---

## Remaining Tasks (in order)

1. Finish Phase-1d convergence: Pass 29/30/31+ until 3 consecutive clean (counter = 0 now).
2. Human spec-package approval gate (run check-input-drift first; present review questions;
   gate items: CC-TUITERM-WIP-SIGNOFF + CC-GLOBAL-MOUSE-CAPTURE).
3. Phase 2 story decomposition (vsdd-factory:story-writer): v1A delta into stories + waves;
   resolve all S-TBD anchors in 25 BCs + holdout stories_tested fields.
4. VP authoring (vsdd-factory:architect) — deferred to formal-hardening (VP-TBD pattern).
5. Pre-Phase-3: DTU clone check (S-DTU-001 fidelity 1.0 — D-234; UNBLOCKED); CI/CD verify.
6. Phase 3 TDD implementation of v1A stories (wave gates). v1B stories authored later.

---

## Full Checkpoint

See `.factory/STATE.md` block `next_session_resume_protocol` (version 7.21, D-270) for:

- The complete convergence-loop procedure (Steps A/B/C + commit rules + cycle checklist).
- The full 28-pass finding trajectory with per-pass detail.
- The full spec package list with all current versions (derived from registry).
- Ratified decisions: Model-A spawn path, session-host-owns-PTY, keyboard fidelity, etc.
- All codified lessons (registry atomicity, propagation-closure, anchor-resolution, etc.).
- The durable_task_register for all non-blocking open items (including S28-001).
- Already-built substrate inventory (1514 tests, 9 workspace crates, daemon, TUI, IPC).
