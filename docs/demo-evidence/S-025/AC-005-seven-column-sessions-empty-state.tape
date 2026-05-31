Output AC-005-seven-column-sessions-empty-state.gif
Output AC-005-seven-column-sessions-empty-state.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-005: Sessions panel renders 7-column rows (session ID, harness icon, project
# name, status, token count, cost, uptime) and empty-state two-line message when
# no sessions are active.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-005: 7-column session rows + empty-state message"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test sessions_panel ac005 -- --nocapture"
Enter
Sleep 4s

Type "cargo test -p monocle-tui --test sessions_panel canonical_row -- --nocapture"
Enter
Sleep 4s
