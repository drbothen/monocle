---
title: S-025 Adversarial Pass 14
pass_number: 14
counter_before: 0/3
counter_after: 0/3 (HOLD — orchestrator rubric: NIT finding requires fix-round per Pass-12 dispatch protocol "If findings: route to fix")
verdict: NITPICK_ONLY (1 NIT finding + 3 sub-NIT observations; no LOW/MED/HIGH/CRITICAL)
head_sha_reviewed: c8204df
created: 2026-05-28
---

## Summary

Pass 14 applied maximum skepticism per L-W6-S025-001..007 and the Pass 12/13 trajectory pattern (each fix-round has surfaced a new class). Re-derived from artifacts cold. Verified Pass 13 fix-round landed: workspace_structure.rs:162 rename, 3 new color/precedence assertions in startup_connect.rs, 4 ui consts re-exported at lib.rs:33-36 (15 total re-exports). Pass 12 + 11 fixes preserved. App field re-scan: status_message correctly traced render→buffer; event_ring + config remain forward-compatible (S-027/S-030+ documented inline deferrals — not Pass-12-class vacuous-mirror). PR-diff scope clean.

Found 1 NIT-level finding via the same class-symmetry lens that Pass 13 NIT-002 covered: render_frame's THIRD branch (DarkGray baseline at app.rs:948-952) has NO render+color assertion test. Pass 13's NIT-002 fix added Color::Yellow assertions on the Yellow status_message branch; the drop_counter Yellow branch is covered by test_ac007. The DarkGray baseline (status_message=None AND drop_counter=0) is the missing branch.

This satisfies the trajectory pattern: each fix-round reveals a new class, but the novelty is decaying — Pass 14 finds 1 NIT (class-symmetry completion), not a new BC-violating bug. The pattern is naturally closing.

## Verifications Performed

- [x] Pass 13 fix-round commits verified:
  - workspace_structure.rs:162 renamed to `ac_005_workspace_declares_three_phase1_core_members`
  - startup_connect.rs:1099 + 1193 Pass-12 tests now assert Color::Yellow (lines 1152-1177, 1246-1272)
  - startup_connect.rs:1291 new precedence test asserts message wins + Color::Yellow when both branches active
  - lib.rs:33-36 re-exports 4 ui consts (15 total re-exports)
- [x] No additional unre-exported pub consts in monocle-tui (sweep complete)
- [x] Pass 12 + 11 fix landings still present: render_frame precedence at app.rs:941-953; 6 const extractions; SS-tui v1.8.2; BC-2.06.016 v1.0.8
- [x] App field re-scan via Heuristic 1: sessions, drop_counter, status_message all traced state→render→buffer; event_ring + config forward-compatible deferrals (not vacuous-mirror)
- [x] Heuristic 4 re-sweep: no stale Phase-N member-list assertions remain
- [x] PR-diff scope: 3 files touched (workspace_structure.rs, startup_connect.rs, lib.rs); no extraneous edits
- [x] Test-writer sub-NIT #1 (DarkGray baseline): adjudicated to NIT (rises above sub-NIT under production-grade lens)
- [x] Test-writer sub-NIT #2 (PC-6 highlight text symbol): adjudicated to sub-NIT (BC-2.06.005 PC-6 contract is "selected row has highlight bg"; current test is contract-aligned)
- [ ] cargo build/test/clippy/fmt — NOT EXECUTED (adversary read-only)
- [ ] PR #28 CI status — NOT EXECUTED (adversary read-only)

## Findings

### F-S025-ADV14-NIT-001 — DarkGray baseline status-line branch has no render+color assertion test

**Severity:** NITPICK. **Confidence:** HIGH. **Routing:** test-writer.

**Evidence:**
- app.rs:941-953 status_line builder has 3 branches per inline precedence rationale (lines 922-940):
  1. status_message.as_deref() → Color::Yellow (3 tests assert)
  2. drop_counter > 0 → Color::Yellow (test_ac007 at lines 897-979 asserts)
  3. Default → MONOCLE_STATUS_LABEL with Color::DarkGray — NO render+color assertion exists
- Grep confirms: zero hits for `MONOCLE_STATUS_LABEL.*DarkGray|DarkGray.*MONOCLE|DarkGray.*monocle` in tests/

**Class-symmetry:** Same class as Pass 13 NIT-002 (status_message Yellow assertion missing). Pass 13 closed the two Yellow branches but not the third DarkGray branch (not flagged at the time).

**Bug-injection:** Changing app.rs:951 Color::DarkGray → Color::Red would pass all current tests. The drop_counter=0 negative at sessions_panel.rs uses `render_sessions_panel` (panel-only render), NOT page-level render_frame.

**Proposed fix:** Add `test_bc_2_06_007_pc7_render_frame_renders_monocle_label_with_dark_gray_when_baseline` mirroring NIT-002 pattern: status_message=None, drop_counter=0, render via TestBackend(80,6), assert status row contains MONOCLE_STATUS_LABEL with cell.style().fg == Some(Color::DarkGray).

## Observations (sub-NIT — not flagged)

- **OBS-001 (Fullscreen status-bar structural inheritance):** Fullscreen branch (app.rs:957-981) and Dashboard branch (982-996) both render the SAME status_line variable. Single construction at 941-953 makes Fullscreen-specific status_line bug unlikely. Sub-NIT forward-compat risk.
- **OBS-002 (PC-6 selected row text symbol):** test_bc_2_06_005_pc6_selected_row_renders_blue_highlight_background asserts bg without text. Contract-aligned with PC-6 ("selected row has highlight bg"); ratatui ListState mapping is upstream-tested. Correctly sub-NIT.
- **OBS-003 (forward-compatible state):** app.config + app.event_ring are populated but not read by render_frame. Explicit inline deferral comments cite S-027/S-030+. No current S-025 render contract — NOT Pass-12-class vacuous-mirror.

## Class-Sibling Sweep

**NIT-001 (DarkGray baseline coverage):** All 3 render_frame branches reviewed; 2 of 3 have color assertion; 1 missing (the finding). Class-wide closure requires the 3rd test.

**Heuristic 1 re-sweep:** All App fields traced (mode, sessions, drop_counter, status_message, overlay_stack [S-026], event_ring [S-027], config [forward-compat]). No vacuous-mirror class siblings in S-025 scope.

**Heuristic 4 re-sweep:** No stale Phase-N assertions remain after Pass 13 LOW-001 fix.

**PR-diff scope sweep:** Pass 13 fix touched exactly 3 files; no extraneous edits.

## Counter Decision

**HOLDS at 0/3** per orchestrator rubric.

Rationale: Per the Pass-12 dispatch protocol ("If clean: counter → 1/3, dispatch Pass 13 immediately. If findings: route to fix → retry Pass 12"), "clean" means zero findings. Pass 14 has 1 NIT finding; not clean. Conservative path: dispatch fix → Pass 15 (expected unambiguously clean → 1/3).

Per Pass 13 precedent: HOLD on findings, fix in scope per production-grade default, advance only on clean pass.

## Defense of the Search

Re-derived cold from artifacts (STATE.md v6.34, Pass 12+13 reports, BC-2.06.016 v1.0.8, SS-tui v1.8.2, S-025 v1.6, app.rs production). Re-traced render_frame top-to-bottom. Re-traced Heuristic 1 across all 7 App fields. Re-traced L-W6-S025-003 across all pub consts. Adjudicated 2 test-writer sub-NITs: 1 rises to NIT, 1 stays sub-NIT.

The premature-clean signal was at peak per Pass-13 brief. Found 1 new class instance: third-branch coverage asymmetry. Could not have surfaced in Pass 13 (Pass 13 only had two Yellow branches to compare; the NIT-002 fix completed Yellow coverage and made the third branch gap visible). Pattern naturally decays — if Pass 15 finds nothing, color-branch coverage is class-wide complete.
