# monocle — Resume From Here (D-280..D-289, 2026-06-14)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.39, for the full checkpoint).

---

## Status at Pause

**Phase-1d adversarial spec convergence is in progress.** 46 passes complete.
Consecutive-clean counter = 0. Pass-47 is the next dispatch (clean candidate 1 of 3).

Human directive: **strict 3 consecutive clean passes** (zero Critical + zero Important).
Do NOT accept fewer than 3. Do NOT resume old observe-only Phase 4-7.

develop @ 8bc22a5 — no v1A production code written. develop HAS had docs/version-pin/CI-wiring
commits (POL-13 anchor-lint, version-pin maintenance, POL-14 parenthetical-anchor-pin lint
at 5d9d603). All pivot spec work on factory-artifacts.

---

## This Session's Arc (Passes 38-46, D-280..D-289)

- **Pass-38 CLEAN** (D-280): S38-001 BC-2.09.008 PC-4/PC-1 partial mouse-capture restatement
  fixed in-scope whole-class; BC-2.09.008→1.2.0; counter RESET to 0.
- **Pass-39 CLEAN** (D-281): S39-001 SS-embedded-pty:250 global→scoped mouse-capture prose errata
  fixed in-scope; errata-no-bump; counter RESET to 0.
- **Pass-40 CLEAN** (D-282): OBS-1 BC-2.09.002 §Trace stale line-numbers deferred as housekeeping
  (navigational only; POL-13 green); no spec change; counter ADVANCES 0→1.
- **Pass-41 FINDINGS** (D-283): F-P41-IMP-001 SessionCreation→EmbeddedTerminal session_id handshake
  gap — SpawnAck mechanism (b) ratified; launching_session_id field added; SS-ipc→1.21.0,
  SS-embedded-pty→1.6.0, SS-session-manager→2.3.0, BC-2.08.001→1.5.0, BC-2.08.008→1.2.0,
  BC-2.09.008→1.3.0; counter RESET 1→0.
- **Proactive consistency-validator sweep** (D-285): CV-SS-001..005 + sibling doc; flushed
  SpawnAck ordering guarantee, spawn-fail ProfilePicker return target, wizard_session_id orphan
  pre-fix across all 7 spawn-handshake docs in one burst before Pass-42.
- **Pass-42 FINDINGS** (D-284): F-P42-IMP-001 orphan wizard_session_id field-name in 4 SS-ipc
  sites + 1 SS-session-manager site corrected to launching_session_id; whole-class grep zero
  survivors; errata-no-bump; counter stays 0.
- **Pass-43 FINDINGS** (D-286): F-P43-IMP-001 SpawnAck step missing from SS-daemon-wiring-v2-delta
  §3 duplicate IPC-handler skeleton — last of 7 spawn-handshake siblings; SS-daemon-wiring-v2-delta
  bumped to v1.10.0 at D-286 authoring time; counter stays 0.
- **Pass-44 FINDINGS** (D-287): F-P44-IMP-001 EngineError::UnsupportedOperation collapsed to generic
  invalid_request wire code — fixed with new spawn_unsupported (11th code); taxonomy 10→11;
  all 3 EngineError variants now map to dedicated codes; EngineError→wire-code class fully closed;
  SS-ipc v1.22.0, SS-session-manager v2.4.0, SS-engine-module-v2-delta v1.5.0,
  SS-daemon-wiring-v2-delta v1.11.0, BC-2.03.008 v1.0.2, BC-2.05.010 v1.8.1; counter stays 0.
- **Pass-45 CLEAN** (D-288): breadth sweep (PTY/scrollback/persistence/hooks/permission-overlay/
  SessionState/holdouts) sound; 2 LOW suggestions S-P45-001/S-P45-002 deferred to Phase-4
  holdout-prep (OBS-HS-PROSE-PHASE4-PREP); no spec change; counter ADVANCES 0→1.
- **Pass-46 FINDINGS** (D-289): F-P46-IMP-001 stale §Architecture Anchors version pins in
  BC-2.08.001 (5), BC-2.08.008 (3), BC-2.09.008 (3) — invisible to old POL-11 (missed
  `path#anchor (vX.Y.Z)` parenthetical form); FIXED: version-less navigational anchors adopted
  (errata-no-bump, 3 BCs); root-cause POL-11 blind spot CLOSED by POL-14/Pattern C in
  check_version_pins.py + CI pol-lint + lefthook (develop @ 5d9d603); counter RESET 1→0.

---

## Next Action: Adversarial Pass-47

Dispatch `vsdd-factory:adversary` FRESH-CONTEXT for Pass-47 (clean candidate 1 of 3).

Feed the full spec package (section D of `next_session_resume_protocol` in STATE.md v7.39).

**Tell the adversary Pass-47 CLOSED items — do NOT re-litigate:**
- Spawn-handshake complete across all 7 docs (SS-ipc, SS-session-manager, SS-embedded-pty,
  SS-daemon-wiring-v2-delta, BC-2.08.001/008, BC-2.09.008); launching_session_id canonical.
- EngineError taxonomy 11 codes complete; spawn_unsupported wire code; _=> arm = forward-compat only.
- Version-less §Architecture Anchors (errata-no-bump); authoritative pins only in §Architecture Source.
- spawn_unsupported (11th code) + banner "Session spawn not supported for this harness".
- All CV-SS-001..005 items flushed (SpawnAck ordering, spawn-fail ProfilePicker return, wizard_session_id).
- OBS-1 (BC-2.09.002 §Trace stale line-numbers) — ratified LOW housekeeping deferral.
- S-P45-001/S-P45-002 (holdout-prose imprecisions) — ratified LOW Phase-4-prep deferral.
- POL-14 parenthetical-anchor-pin lint live (develop @ 5d9d603).

**Emphasize BREADTH** — spawn/mouse/taxonomy areas are exhaustively closed. Hunt less-scrutinized
subsystems (hooks, permissions, scrollback/persistence, SessionState lifecycle, security paths,
PTY backpressure, daemon UDS, wire-type #[non_exhaustive] constructors).

---

## Current Spec-Package Headline Versions

| Document | Version |
|----------|---------|
| domain-monocle-vision-synthesis.md | v2.2.3 (APPROVED) |
| product-brief.md | v2.0.4 |
| prd.md | v1.28.3 |
| ARCH-INDEX | v1.0.28 |
| SS-ipc | v1.22.0 |
| SS-session-manager | v2.4.0 |
| SS-embedded-pty | v1.6.0 |
| SS-engine-module-v2-delta | v1.5.0 |
| SS-daemon-wiring-v2-delta | v1.11.0 |
| SS-deps-pin-manifest-v2-delta | v1.0.1 |
| ADR-0009 | v1.0.2 |
| ADR-0010 | v1.6.0 |
| ADR-0011 | v1.2.1 |
| BC-INDEX | v1.40.5 (138 BCs; 25 v1A BCs) |
| BC-2.03.008 | v1.0.2 |
| BC-2.05.010 | v1.8.1 |
| BC-2.08.001 | v1.5.0 |
| BC-2.08.008 | v1.2.1 |
| BC-2.09.008 | v1.3.1 |
| EVAL-INDEX | v1.15 |
| version-pin-registry.yaml | source of truth |

---

## Open Non-Blocking Follow-Ups

- **DEDUP-IPC-HANDLER-SKELETON**: de-duplicate SS-session-manager §IPC handler (canonical) vs
  SS-daemon-wiring-v2-delta §3 (mirror) — duplication caused F-P43. Orchestrator action; schedule
  before/with Phase-2. PENDING.
- **OBS-HS-PROSE-PHASE4-PREP**: two LOW holdout-prose imprecisions (HS-EXP-014:46 child_pid;
  HS-EXP-013:54 display-order). Deferred to Phase-4 holdout-eval prep. DEFERRED-PHASE4-PREP.
- **SS-IPC-181-REDUNDANT-HISTORICAL-MARKER**: line ~181 pin (v1.9.1) stale after D-286/D-287 bumps
  (now v1.11.0); POL-11-exempt (historical marker); manual update on next normative edit. PENDING.
- **Tooling note**: POL-14 (Pattern C anchor-pin freshness) now live in scripts/check_version_pins.py
  + CI pol-lint + lefthook (develop @ 5d9d603).
- Long-standing durable_task_register items (ADV-W5GATE-HIGH-002 dead code, ADV-W5GATE-MED-001/003,
  DTU-CLONE-STORY RESOLVED D-234, etc.) — see STATE.md §H for full list.

---

## After Phase-1d Converges (3 consecutive clean)

1. Run `/vsdd-factory:check-input-drift` first.
2. Human spec-package approval gate: CC-TUITERM-WIP-SIGNOFF (tui-term 0.3.4 WIP risk-acceptance)
   + CC-GLOBAL-MOUSE-CAPTURE (mouse capture scope ratification).
3. Phase-2 story decomposition (vsdd-factory:story-writer): v1A delta → stories + waves;
   resolve all S-TBD anchors in 25 BCs + holdout stories_tested fields.

---

## Full Checkpoint

See `.factory/STATE.md` block `next_session_resume_protocol` (version 7.39, D-280..D-289) for:

- The complete convergence-loop procedure (Steps A/B/C + commit rules + cycle checklist).
- The PARTIAL-FIX-SIBLING mitigation (sections A, B, F).
- The full 46-pass finding trajectory with per-pass detail.
- The full spec package list with all current versions (derived from registry).
- Ratified decisions: all Pass-38..46 closed items (section E).
- All codified lessons including POL-COVERAGE-BLIND-SPOT and PROACTIVE-CONSISTENCY-SWEEP.
- The durable_task_register for all non-blocking open items.
- Already-built substrate inventory (1514 tests, 9 workspace crates, daemon, TUI, IPC).
