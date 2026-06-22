# S-041 AC-002 — Mouse Dispatch + Scoped-Capture Lifecycle Tests
# Demonstrates: Event::Mouse → KeyInput IPC dispatch wiring, out-of-pane no-IPC,
#   crossterm↔PtyMouseEvent conversion boundary, ratatui Rect→PtyRect conversion,
#   scoped mouse-capture symmetry (capture torn down on EmbeddedTerminal departure
#   incl. permission-prompt→Overlay, disconnect, normal Esc exit — F-S041-P7/P11
#   regression guards).
# Covers: BC-2.09.003 AC-001..AC-008, Invariant 1 (scoped capture symmetry)
# Test file: crates/monocle-tui/tests/bc_2_09_003_mouse_dispatch.rs (20 tests)
# S-044 dependency note: EmbeddedTerminal entry trigger is S-044 scope. These tests
# drive the full dispatch pipeline (handle_crossterm_event → dispatch_embedded_terminal_mouse
# → mouse_event_to_pty_bytes → KeyInput IPC) and lifecycle API directly.

Output AC-002-mouse-dispatch-lifecycle-tests.webm

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

Type "# S-041: monocle-tui mouse dispatch + lifecycle tests (BC-2.09.003)"
Enter
Sleep 500ms

Type "cargo test -p monocle-tui --test bc_2_09_003_mouse_dispatch 2>&1"
Enter
Sleep 30s
