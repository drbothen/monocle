Output AC-009-terminal-restore-panic-hook.gif
Output AC-009-terminal-restore-panic-hook.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-009: Terminal restored on all exit paths (q, IPC error, SIGTERM, panic).
# Panic hook calls disable_raw_mode + LeaveAlternateScreen before unwinding.
# Esc is identity in Dashboard/Overlay/Fullscreen modes (not an exit path in this story).
# Full TTY-dependent assertion deferred to Phase 4 holdout HS-EXP-009.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-009: terminal restore + panic hook (Esc identity, q sole quit key)"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test startup_connect adv38 -- --nocapture"
Enter
Sleep 4s

Type "cargo test -p monocle-tui --test startup_connect pc1_ac001 -- --nocapture"
Enter
Sleep 4s
