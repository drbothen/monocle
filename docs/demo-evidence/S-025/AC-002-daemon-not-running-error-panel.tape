Output AC-002-daemon-not-running-error-panel.gif
Output AC-002-daemon-not-running-error-panel.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-002: On UDS connect failure, monocle-tui renders error panel:
# "Daemon not running. Start it with: monocle daemon start" and exits with code 1.
# Test pins DAEMON_NOT_RUNNING_ERROR const and exercises ratatui TestBackend render.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-002: daemon-not-running error panel text + render"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test startup_connect ac002 -- --nocapture"
Enter
Sleep 4s
