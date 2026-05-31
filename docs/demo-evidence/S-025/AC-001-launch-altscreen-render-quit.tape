Output AC-001-launch-altscreen-render-quit.gif
Output AC-001-launch-altscreen-render-quit.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-001: monocle-tui App initialises in Dashboard{Sessions} mode.
# q is the sole Dashboard quit key. Esc in Dashboard mode is identity (does NOT quit).
# Tests exercise production dispatch_key_event and App::new code paths.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-001: launch, alt-screen, 200ms render, q-quit, Esc-identity"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test startup_connect adv38 -- --nocapture"
Enter
Sleep 4s

Type "cargo test -p monocle-tui --test startup_connect pc1_ac001 -- --nocapture"
Enter
Sleep 4s
