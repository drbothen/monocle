# S-042 AC-002 — Run-Loop Wiring: ResizePane fires on tick without explicit check call
# Demonstrates: run loop integrates resize check on every tick; no dead code; dashboard mode no-op;
#               overlay transition clears debounce state; layout change triggers detection
# Covers: BC-2.09.006 AC-001 (detection per render cycle), AC-007 (focused session only / Dashboard no-op)
# Test file: bc_2_09_006_run_loop_wiring.rs (6 anti-dead-code tests)
# These tests guard against the class of bug where check_resize_debounce exists but is never called.

Output AC-002-run-loop-wiring-tests.webm

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

Type "# S-042: run-loop resize wiring tests (BC-2.09.006 AC-001, AC-007)"
Enter
Sleep 500ms

Type "cargo test -p monocle-tui --test bc_2_09_006_run_loop_wiring 2>&1"
Enter
Sleep 10s
