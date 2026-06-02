# AC-003 + AC-004 — Filter exit (Enter/Esc) + empty query shows all sessions
# BC-2.06.006 PC-5 (Esc dispatches CancelFilter → Dashboard { prior })
# BC-2.06.006 PC-2 exit (Enter dispatches CommitFilter → Dashboard { prior })
# BC-2.06.006 PC-2 empty (empty query shows all sessions without nucleo scoring)
#
# Evidence via filter_sessions.rs TestBackend dispatch tests:
#   - test_BC_2_06_006_commit_filter_returns_to_dashboard (AC-003, Enter → CommitFilter)
#   - test_BC_2_06_006_cancel_filter_returns_to_dashboard (AC-003, Esc → CancelFilter)
#   - test_BC_2_06_006_empty_query_shows_all_sessions (AC-004, empty query shows all)
#   - test_BC_2_06_006_backspace_removes_last_char (edge: backspace trims query)
#   - test_BC_2_06_006_ec091_backspace_on_empty_query_no_panic (edge: backspace on "" is no-op)
#
# Success path: Enter returns to Dashboard retaining selection; empty query renders all sessions.
# Error path: Backspace on empty query does not panic; Esc clears filter and restores full list.

Output docs/demo-evidence/S-028/AC-003-004-filter-exit-empty-query.gif
Output docs/demo-evidence/S-028/AC-003-004-filter-exit-empty-query.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms

Hide
Type "clear"
Enter
Sleep 1s
Show

# AC-003: CommitFilter (Enter) and CancelFilter (Esc) both return to Dashboard
Type "cargo test -p monocle-tui --test filter_sessions -- test_BC_2_06_006_commit_filter_returns_to_dashboard test_BC_2_06_006_cancel_filter_returns_to_dashboard 2>&1 | tail -10"
Enter
Sleep 15s

# AC-004: empty query shows all sessions; backspace edge cases
Type "cargo test -p monocle-tui --test filter_sessions -- test_BC_2_06_006_empty_query_shows_all_sessions test_BC_2_06_006_backspace_removes_last_char test_BC_2_06_006_ec091_backspace_on_empty_query_no_panic 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
