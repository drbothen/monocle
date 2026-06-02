Output AC-003-navigation-wrap.gif
Output AC-003-navigation-wrap.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-003 (BC-2.07.004 PC-3):
# j/down moves selection down (wraps to top); k/up moves up (wraps to bottom).
# Enter selects the highlighted profile; Esc closes without changes.
#
# SUCCESS PATH: increment/decrement index with wrapping
# ERROR PATH (edge): navigation no-ops on empty profiles list

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-003: j/down increments selection index"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_004_picker_select_next_increments_index -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-003: j/down wraps from last to index 0"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_004_picker_select_next_wraps_to_top -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-003: k/up decrements selection index"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_004_picker_select_prev_decrements_index -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-003: k/up wraps from index 0 to last"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_004_picker_select_prev_wraps_to_bottom -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-003 edge: navigation no-ops on empty profile list"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_004_navigation_noop_on_empty_profiles -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
