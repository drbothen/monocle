# S-040 AC-002 — Keyboard dispatch + bracketed paste wiring tests
# Demonstrates: EmbeddedTerminal event dispatch arm, Esc-intercept, bracketed paste,
#               oversized-paste guard, IPC KeyInput send path
# Covers: BC-2.09.002 (dispatch wiring), BC-2.09.004 (Kitty IPC path),
#         BC-2.09.005 (bracketed paste + size guard)
# 24 integration tests across bc_2_09_keyboard_tests + bc_2_09_wiring_tests
# S-047 dependency note: session-host PTY-stdin write is S-047 scope;
# these tests verify the TUI-side send path via mock IPC channels.

Output AC-002-keyboard-dispatch-wiring.webm

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
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-040 && clear"
Enter
Sleep 2s
Show

Type "# S-040: keyboard dispatch + bracketed paste wiring (BC-2.09.002/004/005)"
Enter
Sleep 500ms

Type "cargo test -p monocle-tui --test bc_2_09_keyboard_tests --test bc_2_09_wiring_tests 2>&1"
Enter
Sleep 30s
