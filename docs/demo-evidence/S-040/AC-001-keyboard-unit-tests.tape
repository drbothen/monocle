# S-040 AC-001 — Pure-core keyboard translation unit tests
# Demonstrates: key_event_to_pty_bytes, encode_kitty_key, is_kitty_enhanced_key
# Covers: BC-2.09.002 (full translation table), BC-2.09.004 (Kitty CSI-u)
# 41 unit tests — all pure functions in monocle-core/src/keyboard.rs
# S-047 dependency note: PTY-stdin write is out of scope; these tests cover
# the translation layer only.

Output AC-001-keyboard-unit-tests.webm

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

Type "# S-040: monocle-core keyboard unit tests (BC-2.09.002 / BC-2.09.004)"
Enter
Sleep 500ms

Type "cargo test -p monocle-core --lib keyboard 2>&1"
Enter
Wait+Line /test result/
Sleep 2s
