# S-041 AC-001 — Mouse SGR Encoding Unit Tests
# Demonstrates: mouse_event_to_pty_bytes SGR encoding — full Ps table (button
#   down/up/drag/scroll/moved), modifier bits (Ctrl/Shift/Alt/combined),
#   1-indexed pane-relative coords, out-of-pane→None, terminator M vs m.
# Covers: BC-2.09.003 AC-001..AC-010 (SGR encoding, coordinate transform, modifier bits)
# Test file: crates/monocle-core/tests/bc_2_09_003_mouse_forwarding.rs (27 tests)
# S-044 dependency note: EmbeddedTerminal entry trigger (the mode that activates mouse
# forwarding in production) is S-044 scope and not yet wired. These tests cover the
# pure-core SGR encoding layer verified through the full test suite.

Output AC-001-mouse-sgr-encoding-tests.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1280
Set Height 720
Set Theme "Dracula"
Set Padding 16
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-041 && clear"
Enter
Sleep 2s
Show

Type "# S-041: monocle-core mouse SGR encoding tests (BC-2.09.003)"
Enter
Sleep 500ms

Type "cargo test -p monocle-core --test bc_2_09_003_mouse_forwarding 2>&1"
Enter
Sleep 30s
