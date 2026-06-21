# VHS Demo — S-039 PTY Output Pipeline
# AC-001: BC-2.09.001 full test suite (35 tests) passes green
#
# Demonstrates: PtyOutput render, auto-attach buffering/replay,
# bounded buffer cap/timeout, reconnect cleanup, and GC.
#
# NOTE: Interactive entry into EmbeddedTerminal mode is gated by S-040
# (keyboard forwarding / [e] keybinding). This tape demonstrates the
# harness/test-driven evidence that the pipeline is correct.

# === Output Configuration (WEBM only — project directive D-333, no GIF) ===
Output docs/demo-evidence/S-039/AC-001-pty-pipeline-test-suite.webm

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

# === Hidden Setup ===
Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-039"
Enter
Sleep 1s
Show

# === Demo: AC-001 — BC-2.09.001 full suite (35 tests) ===
Type "# S-039: BC-2.09.001 PTY output pipeline — 35 tests"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test bc_2_09_001_pty_output_pipeline 2>&1 | tail -12"
Enter
Sleep 5s

Sleep 2s
