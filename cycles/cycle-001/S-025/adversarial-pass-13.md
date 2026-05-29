---
title: S-025 Adversarial Pass 13
pass_number: 13
counter_before: 0/3
counter_after: 0/3 (HOLD — orchestrator rubric: LOW findings are not NITPICK_ONLY)
verdict: LOW (2 LOW + 4 NITPICK observations; no HIGH/CRITICAL)
head_sha_reviewed: 4a59074
created: 2026-05-28
---

## Summary

Pass 13 applied maximum skepticism per L-W6-S025-002/004 (Pass 9 and Pass 12 both found new vacuous-mirror class instances after fix-rounds). Traced every App field, every render path, every BC contract using verbs "renders/displays/shows," and verified buffer-level vs state-mutation assertions. Pass 12 CRITICAL fix is correctly implemented: idiomatic `if let Some(msg) = app.status_message.as_deref()` precedence at app.rs:941-953, Color::Yellow with full inline rationale (lines 924-940), status_line built BEFORE the mode match so both Fullscreen (977) and non-Fullscreen (991) branches honor it. Both red-gate tests at startup_connect.rs:1099-1218 verify the rendered buffer. Heuristic 1 sweep across all App fields found no new vacuous-mirror class instances in S-025 scope. Found 2 LOW + 4 NITPICK polish items. Orchestrator rubric interpretation: LOW findings are real cosmetic defects (foot-gun + L-W6-S025-003 asymmetry); counter HOLDS at 0/3 pending fix round.

## Verifications Performed

- [x] Pass 12 fix-round commit landings verified (4a59074 implementer, c48ae11 test-writer, 7b74fd3 state-manager) by state inspection
- [x] render_frame status_line builder uses idiomatic `if let Some(msg) = app.status_message.as_deref()` (no is_some+unwrap), Color::Yellow, inline rationale (lines 922-940)
- [x] Both render branches honor the fix: status_line constructed BEFORE the mode match (lines 941-953); passed to Fullscreen (977) AND Dashboard (991)
- [x] Both red-gate tests at startup_connect.rs:1099-1218 assert rendered buffer (TestBackend, not field-mutation)
- [x] No production clears status_message = None — clear-on-reconnect is TODO(S-023-merge) gated
- [x] 6 pub const extractions present: DAEMON_DISCONNECT_STATUS (app.rs:82), DAEMON_OFFLINE_STATUS (app.rs:89), MONOCLE_STATUS_LABEL (app.rs:96), TOKEN_COUNT_OVERFLOW_CAP (sessions_panel.rs:68), UPTIME_OVERFLOW_CAP (sessions_panel.rs:75), format_drop_counter (app.rs:110)
- [x] Re-export verified for 4 of 6 via lib.rs:20-30; TOKEN_COUNT_OVERFLOW_CAP + UPTIME_OVERFLOW_CAP + SESSIONS_EMPTY_LINE_1/2 are pub-but-not-re-exported (see LOW-002)
- [x] F-S025-CI-001 fix verified: ac_005_workspace_does_not_declare_monocle_auth (workspace_structure.rs:171) correctly excludes monocle-tui
- [x] Architectural-boundary tests verified intact (healthz/status/shutdown_handler does_not_import_monocle_tui)
- [x] Heuristic 1 sweep across all App fields: sessions, drop_counter, status_message, overlay_stack (S-026 deferred), event_ring (S-027 deferred), mode, config — no vacuous-mirror class siblings
- [x] Heuristic 4 sweep: no stale Phase-N member-list assertions beyond the test-name drift in LOW-001
- [ ] cargo build/test/clippy/fmt — NOT EXECUTED (adversary read-only profile)
- [ ] PR #28 CI status — NOT EXECUTED (adversary read-only profile)

## Findings

### F-S025-ADV13-LOW-001 — Stale Phase-1 test name `ac_005_workspace_declares_exactly_three_phase1_members` no longer matches workspace state

**Severity:** LOW. **Confidence:** HIGH. **Routing:** test-writer (rename to match logic).

- workspace_structure.rs:158 declares `fn ac_005_workspace_declares_exactly_three_phase1_members()`.
- Body (lines 159-168) asserts ONLY that 3 members are PRESENT — does NOT assert "exactly three."
- Workspace at Cargo.toml:3-13 lists 9 members.
- Foot-gun: a future maintainer could read the name and add an "and no others" clause that breaks the workspace.

Class-sibling sweep: ac_006_workspace_omits_rmcp_in_phase_1 (line 224) correctly named (rmcp still omitted, Phase 4); monocle_core_declares_phase1_modules (line 282) correctly named.

### F-S025-ADV13-LOW-002 — pub consts not re-exported asymmetrically violate L-W6-S025-003

**Severity:** LOW. **Confidence:** HIGH. **Routing:** implementer (re-export 4 ui consts).

- sessions_panel.rs:51, 57, 68, 75 declare 4 pub consts (SESSIONS_EMPTY_LINE_1, SESSIONS_EMPTY_LINE_2, TOKEN_COUNT_OVERFLOW_CAP, UPTIME_OVERFLOW_CAP).
- lib.rs:20-30 re-exports 4 sibling consts from app.rs but none of the ui/* consts.
- L-W6-S025-003 explicitly: "Re-export from lib.rs for external test crates" — no exception for ui-internal consts.
- Production-grade fix: re-export ALL 4 ui consts at crate root for symmetry.

## Observations

### F-S025-ADV13-NIT-001 — Precedence test coverage gap (simultaneous status_message + drop_counter > 0)

**Severity:** NITPICK. **Routing:** test-writer.

Pass 12 fix establishes precedence `status_message > drop_counter > default`. New red-gate tests cover status_message+drop_counter=0 (Pass 12) and status_message=None+drop_counter>0 (Pass 11). NOT covered: status_message=Some + drop_counter>0 simultaneous. A regression that reversed precedence would not be caught.

### F-S025-ADV13-NIT-002 — Color::Yellow for status_message not test-asserted

**Severity:** NITPICK. **Routing:** test-writer.

Implementer chose Color::Yellow with production-grade inline rationale. Drop_counter sibling test at startup_connect.rs:951-978 asserts Color::Yellow. status_message tests at 1099-1218 don't. Asymmetric coverage. Spec is silent on color so implementer's choice is acceptable; the test gap is the actual NIT.

### F-S025-ADV13-NIT-003 — BC-2.06.016 v1.0.8 §Trace stale "Follow-up required" note (Pass 12 LOW-002 carryover; deferred to Task #9 post-merge PO sweep)

**Severity:** NITPICK. **Routing:** product-owner — DEFERRED to Task #9 post-merge sweep.

BC-2.06.016 line 230: "Follow-up required (architect scope): SS-tui.md line 668 still cites prose form" — but SS-tui line 668 already uses bracketed form (architect commit 740465d). §Trace note is stale.

**Deferral rationale:** Pass 12 LOW-002 carryover; cosmetic documentation aging; BC bump triggers full propagation chain (BC v1.0.9 → story-writer S-026 frontmatter bump → BC-INDEX/STORY-INDEX bump → consistency-validator re-run) for a §Trace text change. Defer to post-merge PO sweep where multiple §Trace polish items can batch. Tracked in Task #9.

### F-S025-ADV13-NIT-004 — BC-2.06.004 EC-079 cites non-production string "Daemon offline" (deferred to Task #9 post-merge PO sweep)

**Severity:** NITPICK. **Routing:** product-owner — DEFERRED to Task #9 post-merge sweep.

BC-2.06.004 line 104 (EC-079): 'TUI starts but cannot connect to daemon; renders "Daemon offline" status message; no crash.' Production has DAEMON_NOT_RUNNING_ERROR (full-screen panel, AC-002 path) and DAEMON_OFFLINE_STATUS ("[daemon: offline]", S-023 reconnect-exhaust). Neither is literally "Daemon offline." EC-079 is ambiguous; the path described uses DAEMON_NOT_RUNNING_ERROR (full-screen, NOT a status message).

**Deferral rationale:** Same as NIT-003 — cosmetic spec text drift requiring BC bump + full propagation chain. EC-079 is informational (not a hard test contract); production behavior is correct (full-screen panel + key-press exit per AC-002). Defer to Task #9.

## Class-Sibling Sweep

**LOW-001:** workspace_structure.rs sibling Phase-1 test names checked — no other drift.
**LOW-002:** lib.rs vs ui/* pub const sweep — 4 unprived ui consts, not just 2. Class-wide asymmetry.
**Heuristic 1:** All App fields traced (sessions, drop_counter, status_message, overlay_stack [S-026], event_ring [S-027], mode, config) — no vacuous-mirror siblings.
**Heuristic 4:** No stale Phase-N member-list assertions beyond LOW-001.

## Counter Decision

**HOLDS at 0/3.** Orchestrator rubric interpretation per CLAUDE.md production-grade-default: LOW findings (foot-gun test name + L-W6-S025-003 asymmetry violation) are real cosmetic defects, not NITPICK_ONLY-equivalent. S-022 lesson convergence threshold was literally "3 consecutive NITPICK_ONLY." Fix round dispatched; counter holds for next pass after fix-round lands.

## Defense of the Search

Re-derived from artifacts cold (STATE.md v6.33, Pass 12 report, BC-2.06.016 v1.0.8, BC-2.06.004 v1.2.0, SS-tui v1.8.2, S-025 v1.6). Applied L-W6-S025-002 to every App field; Heuristics 2/3 to the new fix; Heuristic 4 to workspace tests; L-W6-S025-007 sweep wider on both LOWs (LOW-002 sweep found 4 unprived consts class-wide).
