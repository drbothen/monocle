Output AC-004-config-default-fallback.gif
Output AC-004-config-default-fallback.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-004: On startup, monocle-tui calls load_config(); on HomeDirUnresolvable
# it falls back to MonocleConfig::default() without panic.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-004: config load with default fallback on error"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test startup_connect ac004 -- --nocapture"
Enter
Sleep 4s
