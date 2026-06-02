# AC-001 + AC-002 — Filter entry (/ and f dispatch StartFilter) + query append per keystroke
# BC-2.06.006 PC-1 (pressing / or f transitions to Filtering { Sessions } mode)
# BC-2.06.006 PC-2 (each typed character appends to query; nucleo scores/filters sessions)
#
# Evidence via filter_sessions.rs TestBackend dispatch tests:
#   - test_BC_2_06_006_filter_entry_slash_transitions_to_filtering (AC-001, /  key)
#   - test_BC_2_06_006_filter_entry_f_key_transitions_to_filtering (AC-001, f key)
#   - test_BC_2_06_006_filter_query_appends_on_char (AC-002, m/o/n accumulate to "mon")
#   - test_BC_2_06_006_nucleo_score_filters_non_matching (AC-002, nucleo hides non-matching)
#
# Success path: / enters Filtering mode; typed chars accumulate in query; nucleo keeps only matching sessions.
# Error path: Pressing / in Dashboard mode and seeing Filtering mode confirmed; non-match NOT rendered.

Output docs/demo-evidence/S-028/AC-001-002-filter-entry-query-append.gif
Output docs/demo-evidence/S-028/AC-001-002-filter-entry-query-append.webm

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

# AC-001: / and f keys dispatch StartFilter → Filtering mode
Type "cargo test -p monocle-tui --test filter_sessions -- test_BC_2_06_006_filter_entry_slash_transitions_to_filtering test_BC_2_06_006_filter_entry_f_key_transitions_to_filtering 2>&1 | tail -10"
Enter
Sleep 15s

# AC-002: typed chars accumulate query; nucleo scoring hides non-matching sessions
Type "cargo test -p monocle-tui --test filter_sessions -- test_BC_2_06_006_filter_query_appends_on_char test_BC_2_06_006_nucleo_score_filters_non_matching 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
