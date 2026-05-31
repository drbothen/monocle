Output AC-007-drop-counter-yellow-status-bar.gif
Output AC-007-drop-counter-yellow-status-bar.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-007: Status bar shows "[dropped: N]" in yellow when drop_counter > 0;
# no drop indicator when drop_counter == 0. DropCounterUpdate IPC message updates the field.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-007: drop counter status bar rendering"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test startup_connect ac007 -- --nocapture"
Enter
Sleep 4s
