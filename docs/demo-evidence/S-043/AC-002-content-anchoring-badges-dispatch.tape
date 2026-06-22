# S-043 AC-002 — Content-anchored viewport preservation and concurrent status bar badges
#               Focused on the hardest correctness cases:
#               - EC-244 (below-cap content-anchor, independent oracle, ADV P4 HIGH-001 rewrite)
#               - EC-246 (at-cap content-anchor clamp, ADV P4 BLOCKER-002 new test)
#               - EC-245 (concurrent [scrolled back N rows] + [dump: N drops] badges)
#               - F-S043-P2-MED-001 (status_message coexists with scrollback indicator)
#               - PC-3 dispatch: Ctrl+Up/Down → PtyScrollUp/Down, no IPC
#               - F-S043-P3-BLOCKER-001: Kitty KeyEventKind::Release discarded; Repeat acts
# Covers: BC-2.09.007 AC-008, AC-014, EC-244, EC-245, EC-246, PC-3, PC-4, PC-5
# Test file: crates/monocle-tui/tests/bc_2_09_007_scrollback_navigation.rs
# 9 tests — all passing (content_anch: 2, dispatch/Ctrl: 5, badge coexist: 2)

Output AC-002-content-anchoring-badges-dispatch.webm

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

Type "# S-043: content-anchoring + concurrent badges + Ctrl+Up/Down dispatch (BC-2.09.007 PC-3/4/5)"
Enter
Sleep 500ms

Type "cargo test -p monocle-tui --test bc_2_09_007_scrollback_navigation content_anch 2>&1"
Enter
Sleep 10s

Type "cargo test -p monocle-tui --test bc_2_09_007_scrollback_navigation dispatch 2>&1"
Enter
Sleep 10s

Type "cargo test -p monocle-tui --test bc_2_09_007_scrollback_navigation ctrl_up 2>&1"
Enter
Sleep 10s

Type "cargo test -p monocle-tui --test bc_2_09_007_scrollback_navigation concurrent_status_bar 2>&1"
Enter
Sleep 10s

Type "cargo test -p monocle-tui --test bc_2_09_007_scrollback_navigation status_message_coexists 2>&1"
Enter
Sleep 10s
