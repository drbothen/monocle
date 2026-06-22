# S-043 AC-001 — Scrollback config (default 1000, clamp 1–10000), PtyScrollUp/Down
#               navigation, per-session offsets, no-IPC guard, status bar indicator,
#               content-anchoring, concurrent badges, Ctrl+Up/Down dispatch path,
#               KeyEventKind Release guard (Kitty double-scroll prevention)
# Covers: BC-2.09.007 all 23 acceptance criteria (AC-001..AC-014 + EC-240..EC-246 +
#         F-S043-P2-MED-001 + F-S043-P3-BLOCKER-001 + PC-3/4/5)
# Test file: crates/monocle-tui/tests/bc_2_09_007_scrollback_navigation.rs
# 23 tests — all passing

Output AC-001-scrollback-config-and-navigation.webm

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
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-043"
Enter
Sleep 2s
Show

Type "# S-043: scrollback navigation unit tests (BC-2.09.007 — all 23 ACs)"
Enter
Sleep 500ms

Type "cargo test -p monocle-tui --test bc_2_09_007_scrollback_navigation 2>&1"
Enter
Sleep 15s
