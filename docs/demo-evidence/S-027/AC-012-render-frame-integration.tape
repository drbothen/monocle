# AC-012 — render_frame integration: overlay + status bar wired into production path
# BC-2.06.019 PC-1 (mode indicator present via render_frame)
# BC-2.06.020 PC-1/PC-3 (breadcrumb on second-to-last row via render_frame)
# BC-2.06.021 PC-1 (hint line on last row via render_frame)
# BC-2.06.010 PC-1 (Permission Request header when Overlay via render_frame)
#
# Evidence via render_frame_integration.rs — drives App::render_frame against
# TestBackend, asserting that the production render path (not isolated widget calls)
# produces the breadcrumb, hint line, and overlay modal header in the buffer.
# Also covers the small-terminal edge: modal does NOT collide with status bar rows.

Output docs/demo-evidence/S-027/AC-012-render-frame-integration.gif
Output docs/demo-evidence/S-027/AC-012-render-frame-integration.webm

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

# AC-012 Dashboard: breadcrumb + hint line via production render_frame
Type "cargo test -p monocle-tui --test render_frame_integration -- test_BC_2_06_020_render_frame_dashboard_mode_breadcrumb_appears_in_buffer test_BC_2_06_021_render_frame_dashboard_mode_hint_line_appears_on_last_row 2>&1 | tail -10"
Enter
Sleep 10s

# AC-012 Overlay: "Permission Request" header + hint + breadcrumb via render_frame
Type "cargo test -p monocle-tui --test render_frame_integration -- test_BC_2_06_010_render_frame_overlay_mode_modal_header_appears_in_buffer test_BC_2_06_021_render_frame_overlay_mode_hint_line_appears_on_last_row test_BC_2_06_020_render_frame_overlay_mode_breadcrumb_appears_on_breadcrumb_row 2>&1 | tail -10"
Enter
Sleep 10s

# AC-012 edge: small terminal (7 rows) — modal footer does not collide with status bar
Type "cargo test -p monocle-tui --test render_frame_integration -- test_BC_2_06_010_small_terminal_modal_does_not_collide_with_status_bar_rows 2>&1 | tail -8"
Enter
Sleep 10s
Sleep 2s
