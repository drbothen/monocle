Output AC-006-keyboard-navigation-jk-enter-tab.gif
Output AC-006-keyboard-navigation-jk-enter-tab.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-006: j/down/k/up move selection; Enter transitions to Fullscreen;
# Tab cycles focus to next panel.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-006: j/k navigation, Enter -> Fullscreen, Tab -> focus cycle"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test sessions_panel ac006 -- --nocapture"
Enter
Sleep 4s

Type "cargo test -p monocle-tui --test sessions_panel enter_transitions_to_fullscreen -- --nocapture"
Enter
Sleep 4s

Type "cargo test -p monocle-tui --test startup_connect select_next -- --nocapture"
Enter
Sleep 4s
