# VHS Demo — S-028 Event Ribbon + Integration Wiring
# Covers ACs:
#
# AC-006 (BC-2.06.018 PC-1): EventRibbon panel renders scrollable chronological list;
#         timestamp | hook_type | summary columns; sourced from ring_tail + HookEventReceived
# AC-007 (BC-2.06.018 PC-2): j/k/↓/↑ scroll; G→oldest; gg→newest; Enter→Fullscreen
# AC-008 (BC-2.06.018 PC-8): auto-scroll follows newest unless user has pinned (pinned_top)
# AC-009 (BC-2.06.018 INV-1): session change resets scroll, re-filters events client-side;
#         no new IPC request (purely client-side per BC-2.05.004 INV-3)
# AC-010 (BC-2.06.006 PC-2 / BC-2.06.018 PC-5): Event Ribbon and Sessions filter wired into
#         production render_frame / dispatch_key_event — NOT dead code
#
# ALSO DEMONSTRATES:
#   F-W7G3-MED-001 FIX (corrected behavior): in Filtering mode with a query that reorders
#   sessions, the Event Ribbon shows the HIGHLIGHTED (filtered-top) session's events —
#   NOT app.sessions[0] (insertion order). Demonstrated via
#   test_F_W7G3_MED_001_event_ribbon_shows_highlighted_session_events_in_filter_mode.

# === Output Configuration ===
Output AC-006-010-event-ribbon-integration.gif
Output AC-006-010-event-ribbon-integration.webm

# === Dependency Check ===
Require cargo

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 680
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

# === Hidden Setup ===
Hide
Type "cd /Users/jmagady/Dev/monocle && clear"
Enter
Sleep 2s
Show

# === Demo Part 1: event_ribbon.rs — AC-006 through AC-009 (unit tests) ===
# Demonstrates:
#   AC-006: newest-first ordering (PC-2), rolling-window bounded (PC-3), PENDING status (PC-4),
#           auto-scroll follows/pins (PC-8), ring_tail pre-population (BC-2.05.002 PC-2),
#           HookEventReceived streaming append (BC-2.05.004), client-side filter (INV-3/INV-1)
#   AC-007: scroll_ribbon_down clamp at oldest (EC-116), session-change reset (INV-1/AC-009)
#   AC-008: auto-scroll suppressed when pinned; auto-scroll follows when not pinned
#   AC-009: session change resets scroll to row 0, clears pinned_top, no IPC request
#   AC-006 deep: wall-clock timestamp from timestamp_micros, streaming daemon timestamp,
#           panel_height cap, PENDING wiring, shared matcher structural audit, trim-on-resize

Type "cargo test -p monocle-tui --test event_ribbon --test event_ribbon_real_defects 2>&1 | grep -E 'test result|PASS|FAIL|error'; echo RIBBON_SUITE_DONE"
Enter
Wait+Screen@120s /RIBBON_SUITE_DONE/
Sleep 1s

# === Demo Part 2: render_frame_integration_s028.rs — AC-010 + F-W7G3-MED-001 FIX ===
# Demonstrates:
#   AC-010: Event Ribbon content in right-40% area (TestBackend render_frame);
#           Sessions filter input box "/ foo_" in Filtering mode;
#           SESSIONS_FILTER_NO_MATCH sentinel for zero-match query;
#           j/↓ dispatches ScrollDown (ribbon offset 0→1, pinned_top→true);
#           j×2 advances 2 rows; k dispatches ScrollUp (3→2);
#           k at row 0 clears pinned_top (auto-scroll resumes);
#           '/' → Filtering → type → Esc → Dashboard → '/' → type → Enter (full cycle)
#   F-W7G3-MED-001 FIX: in Filtering mode (query "xyz"), sess-ZETA (zeta-xyz) is
#           highlighted at filter-list position 0; ribbon shows "SessionStart" (ZETA's event)
#           NOT sess-ALPHA's "Notification" event — corrected filter→ribbon session lookup

Type "cargo test -p monocle-tui --test render_frame_integration_s028 2>&1 | grep -E 'test result|PASS|FAIL|error|F_W7G3'; echo INTEGRATION_SUITE_DONE"
Enter
Wait+Screen@120s /INTEGRATION_SUITE_DONE/
Sleep 2s
