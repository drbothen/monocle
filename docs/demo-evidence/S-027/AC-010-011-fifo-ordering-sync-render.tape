# AC-010 + AC-011 — FIFO oldest-first indicator + synchronous render (<5ms)
# BC-2.06.021 PC-1 (AC-010: "(1 of N)" always refers to oldest pending prompt, FIFO order)
# BC-2.06.010 INV-1 (AC-011: render_overlay_widget completes synchronously < 5ms, no async spawn)

Output docs/demo-evidence/S-027/AC-010-011-fifo-ordering-sync-render.gif
Output docs/demo-evidence/S-027/AC-010-011-fifo-ordering-sync-render.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms

Hide
Type "clear"
Enter
Sleep 1s
Show

# AC-010: "(1 of 2)" indicator when stack_depth=2 (oldest-first FIFO semantics)
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_021_overlay_oldest_first_fifo_indicator_in_header_for_multi_stack 2>&1 | tail -8"
Enter
Sleep 10s

# AC-011: render_overlay_widget completes in < 5ms (synchronous, CPU-bound only)
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_010_invariant_1_render_overlay_widget_completes_synchronously_within_5ms 2>&1 | tail -8"
Enter
Sleep 10s
Sleep 2s
