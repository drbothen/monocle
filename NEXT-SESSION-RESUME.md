# monocle — Resume From Here (D-271..D-279, 2026-06-13)

Read this file first, then CLAUDE.md, then `.factory/STATE.md`
(`next_session_resume_protocol` block, v7.29, for the full checkpoint).

---

## Status at Pause

**Phase-1d adversarial spec convergence is in progress.** 37 passes complete.
Consecutive-clean counter = 0. Pass-38 is the next dispatch (clean candidate 1 of 3).

develop branch has had docs/version-pin/CI-wiring commits this session (POL-13 anchor-lint
CI wiring). No v1A production code written. All pivot spec work on factory-artifacts.

---

## PATTERN WARNING — Partial-Fix Siblings (Passes 33-37)

The recurring finding pattern in passes 33-37 is **PARTIAL-FIX SIBLINGS**: each fix round
exposes a sibling gap that the next pass catches. Examples:

- S35-002 fixed KeyInput error path in BC-2.05.010 → Pass-36 caught Detach/Resize siblings.
- S35-003 fixed BC-2.09.003 mouse encoding → Pass-36 caught Invariant-3 contradiction.
- Pass-36 fixed BC-2.09.003 mouse model → Pass-37 caught BC-2.09.002 Invariant-5 sibling.

**MITIGATION (mandatory every fix round):**
1. Owning agent must grep ALL sibling BCs and arch docs in the SAME subsystem for the same
   pattern and reconcile ALL instances in ONE burst — not just the flagged instance.
2. Tell the adversary each pass to specifically hunt for siblings of the prior round's fix.
3. Do NOT declare a fix-class closed without a whole-subsystem grep confirming no survivors.

---

## This Session's Key Changes (Passes 35-37)

Pass-35 (D-277): FIRST CLEAN (0C/0I). 3 Suggestions fixed in-scope per production-grade
principle: S35-001 split-pair arithmetic; S35-002 KeyInput error path; S35-003 mouse
Drag(MouseButton) missing arm + Moved Ps 32→35 + full Ps/modifier table. Counter RESET to 0.

Pass-36 (D-278): 0C/2I FINDINGS. F-P36-IMP-001 BC-2.09.003 Invariant-3 contradicted S35-003
Moved-reachability model (Ps=35 reachable under 1003, not unreachable). F-P36-IMP-002
BC-2.05.010 missing named "No-silent-failure invariant" + Detach/Resize error-path gaps +
dangling SS arch forward-refs (SS-session-manager:385, SS-ipc:389,1515). PO-only fix burst:
named invariant authored; whole-class swept ALL fallible variants (Kill/Rename/Attach too);
ResizePane WARN-drop exception documented. BC-2.09.003→1.5.0, BC-2.05.010→1.8.0.

Pass-37 (D-279): 0C/2I FINDINGS. F-P37-IMP-001 BC-2.09.002 Invariant-5 still asserted
globally-active mouse capture — partial-fix sibling of the I2-001/I3 scoped-capture fix that
Pass-36's BC-2.09.003 fix had correctly addressed in encoding but missed in Invariant-5.
F-P37-IMP-002 SS-ipc:412 mis-described invalid_request as a pre-call guard (never matched)
vs canonical post-call catch-all (always matched for unhandled ClientToServer variants). PO
fixed BC-2.09.002→1.1.2 (scoped-capture invariant); architect fixed SS-ipc:412 prose-only
errata (no version bump — wire contract unchanged).

---

## Next Action: Adversarial Pass 38

Dispatch `vsdd-factory:adversary` in a fresh context for Pass 38.
Feed the full spec package (section D of `next_session_resume_protocol` in STATE.md v7.29).

The human's directive: **strict 3 consecutive clean passes** (zero Critical + zero Important).
Do NOT accept fewer than 3. Do NOT resume Phase 4-7 of the old observe-only scope.

**Tell the adversary explicitly each pass:**
- Hunt for SIBLINGS of the prior round's fix (Partial-Fix-Sibling mitigation).
- These are CLOSED and should NOT be re-litigated:
  - EngineError = NEW canonical #[non_exhaustive] enum (3 variants), independent
  - ADR-0006 constructors fully documented for all 5 v1A wire structs
  - Dead-anchor class = whole-class remediated + POL-13 enforced
  - Two-pronged InvalidPath null-byte detection
  - Mouse SGR: Drag 32/33/34; Moved Ps=35 (reachable under 1003); modifier Shift=4/Alt=8/Ctrl=16
  - Ordered-pair-split → immediate disconnect independent of slow-client counter
  - BC-2.05.010 No-silent-failure invariant = whole-class complete (all fallible variants covered)
  - BC-2.09.002 Invariant-5 = scoped mouse capture (EmbeddedTerminal entry/exit)
  - BC-2.09.003 Invariant-3 = Moved (Ps=35) reachable under 1003
  - SS-ipc invalid_request = post-call catch-all (not pre-call guard)

Finding trajectory summary (C/I counts per pass):
- Passes 1-6: Critical present (5/8, 5/6, 4/9, 1/4, 2/4, 2/2)
- Passes 7-28: ALL zero-Critical (22 consecutive)
- Pass 20: 0C/0I — FIRST CLEAN; Pass 21: 0C/0I — SECOND CLEAN
- Pass 22: 0C/3I — RESET counter 2→0 (sibling-BC cluster caught)
- Passes 23-28: 0C/1I each — counter stays 0
- Pass 29: 1C/0I  Pass 30: 2C/1I  Pass 31: 1C/1I  Pass 32: 0C/3I
- Pass 33: 0C/2I  Pass 34: 1C/1I  Pass 35: 0C/0I — CLEAN but S-fixes reset counter
- Pass 36: 0C/2I  Pass 37: 0C/2I  →  counter = 0; Pass-38 is clean candidate 1 of 3

---

## Current Spec Package Headline Versions

| Document | Version |
|----------|---------|
| domain-monocle-vision-synthesis.md | v2.2.3 (APPROVED) |
| product-brief.md | v2.0.4 |
| prd.md | v1.28.3 |
| ARCH-INDEX | v1.0.28 |
| SS-ipc | v1.20.1 |
| SS-session-manager | v2.2.1 |
| SS-embedded-pty | v1.5.2 |
| SS-engine-module | v1.1.27 |
| SS-engine-module-v2-delta | v1.4.1 |
| SS-daemon-wiring-v2-delta | v1.9.1 |
| SS-deps-pin-manifest-v2-delta | v1.0.1 |
| ADR-0009 | v1.0.2 |
| ADR-0010 | v1.6.0 |
| ADR-0011 | v1.2.1 |
| BC-2.05.010 | v1.8.0 (Pass-36 No-silent-failure invariant + whole-class sweep) |
| BC-2.09.002 | v1.1.2 (Pass-37 scoped mouse-capture Invariant-5) |
| BC-2.09.003 | v1.5.0 (Pass-36 Invariant-3 Moved reachability) |
| BC-INDEX | v1.40.0 (138 BCs) |
| EVAL-INDEX | v1.15 |
| version-pin-registry.yaml | source of truth |

---

## Remaining Tasks (in order)

1. Finish Phase-1d convergence: Pass 38/39/40+ until 3 consecutive clean (counter = 0 now).
   Apply PARTIAL-FIX-SIBLING mitigation every round.
2. Human spec-package approval gate (run check-input-drift first; present review questions;
   gate items: CC-TUITERM-WIP-SIGNOFF + CC-GLOBAL-MOUSE-CAPTURE).
3. Phase 2 story decomposition (vsdd-factory:story-writer): v1A delta into stories + waves;
   resolve all S-TBD anchors in 25 BCs + holdout stories_tested fields.
4. VP authoring (vsdd-factory:architect) — deferred to formal-hardening (VP-TBD pattern).
5. Pre-Phase-3: DTU clone check (S-DTU-001 fidelity 1.0 — D-234; UNBLOCKED); CI/CD verify.
6. Phase 3 TDD implementation of v1A stories (wave gates). v1B stories authored later.

---

## Full Checkpoint

See `.factory/STATE.md` block `next_session_resume_protocol` (version 7.29, D-271..D-279) for:

- The complete convergence-loop procedure (Steps A/B/C + commit rules + cycle checklist).
- The PARTIAL-FIX-SIBLING mitigation (codified in sections A, B, F, G).
- The full 37-pass finding trajectory with per-pass detail.
- The full spec package list with all current versions (derived from registry).
- Ratified decisions: all Pass-36/37 closed items (see section E).
- All codified lessons including PARTIAL-FIX-SIBLING-MITIGATION (new in v7.29).
- The durable_task_register for all non-blocking open items.
- Already-built substrate inventory (1514 tests, 9 workspace crates, daemon, TUI, IPC).
