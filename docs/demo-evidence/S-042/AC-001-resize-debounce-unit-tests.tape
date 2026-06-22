# S-042 AC-001 — PTY Resize Detection + 50ms Debounce: Unit Tests
# Demonstrates: resize detection per render cycle, debounce logic, local parser immediate resize,
#               rapid-resize coalescing, dashboard-mode no-op, zero-dimensions no-op, scroll-offset reset
# Covers: BC-2.09.006 AC-001..AC-012 (TUI-side debounce, local parser, coalescing, edge cases)
# Test files: bc_2_09_006_resize_debounce.rs (13 tests), bc_2_09_006_poll_timeout_seam.rs (2 tests)
# 15 tests total — all passing

Output AC-001-resize-debounce-unit-tests.webm

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
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-042"
Enter
Sleep 2s
Show

Type "# S-042: TUI resize debounce unit tests (BC-2.09.006 AC-001..AC-012)"
Enter
Sleep 500ms

Type "cargo test -p monocle-tui --test bc_2_09_006_resize_debounce --test bc_2_09_006_poll_timeout_seam 2>&1"
Enter
Sleep 10s
