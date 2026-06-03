# VHS Demo — S-029 Killer Scenario Integration Test Suite
# Covers AC-002 through AC-007 and BC-2.06.022 KS-001/KS-002
#
# AC-002: killer_scenario_accept         — accept decision E2E
# AC-003: killer_scenario_multi_prompt   — two-prompt FIFO stacking
# AC-004: killer_scenario_disconnect     — disconnect clears overlay
# AC-005: killer_scenario_esc_no_reject  — Esc is identity (no reject)
# AC-006: killer_scenario_edit_diff      — diff rendered in TestBackend
# AC-007: isolation via tempfile::TempDir per test (parallel-safe)
# BC-KS:  killer_scenario_accept_always  — AcceptAlways variant

# === Output Configuration ===
Output AC-002-007-killer-scenario.gif
Output AC-002-007-killer-scenario.webm

# === Dependency Check ===
Require cargo

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 680
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

# === Hidden Setup — navigate to worktree, build already warmed ===
Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-029 && clear"
Enter
Sleep 2s
Show

# === Demo: S-029 — Killer Scenario E2E Integration Test Suite ===
# Show the test command for BC-2.06.022 permission-prompt round-trip

Type "cargo test -p monocle-tui --test killer_scenario -- --test-threads=4 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s
