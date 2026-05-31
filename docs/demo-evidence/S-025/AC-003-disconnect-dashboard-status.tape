Output AC-003-disconnect-dashboard-status.gif
Output AC-003-disconnect-dashboard-status.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-003: On TransportEvent::Disconnected the TUI transitions to Dashboard mode,
# clears overlay_stack, and sets status_message = "[disconnected] reconnecting...".

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-003: TransportEvent::Disconnected -> Dashboard + reconnect status"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test startup_connect ac003 -- --nocapture"
Enter
Sleep 4s
