# AC-001 + AC-002 — Overlay modal layout (header/body/footer) + DIM background
# BC-2.06.010 PC-1 (modal layout: header "Permission Request", footer hints, (1 of N), width cap)
# BC-2.06.010 PC-2 (DIM background: all non-modal cells dimmed; status bar NOT dimmed)
#
# Evidence via TestBackend render tests that drive render_overlay_widget and
# render_dimmed_background against a real ratatui Buffer and assert:
#   - "Permission Request" in header
#   - "[y] Accept" and "[n/r] Reject" in footer
#   - "(1 of 3)" stack depth indicator when stack_depth=3
#   - Modal does NOT extend to column 0 (width cap: min(width-4, 100))
#   - DIM modifier present on all dimmed-area cells after render_dimmed_background
#   - Last row (status bar) has NO DIM modifier after overlay render

Output docs/demo-evidence/S-027/AC-001-002-overlay-modal-layout-dim.gif
Output docs/demo-evidence/S-027/AC-001-002-overlay-modal-layout-dim.webm

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

# AC-001: header "Permission Request", footer hints, "(1 of N)", modal width cap
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_010_overlay_header_contains_permission_request_label test_BC_2_06_010_overlay_header_shows_stack_depth_indicator test_BC_2_06_010_overlay_footer_contains_accept_hint test_BC_2_06_010_overlay_footer_contains_reject_hint test_BC_2_06_010_overlay_modal_width_respects_cap_min_w_minus_4_100 2>&1 | tail -12"
Enter
Sleep 12s

# AC-002: DIM modifier on background cells; status bar row NOT dimmed
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_010_render_dimmed_background_applies_dim_modifier_to_all_cells test_BC_2_06_010_overlay_does_not_dim_status_bar_row 2>&1 | tail -10"
Enter
Sleep 10s
Sleep 2s
