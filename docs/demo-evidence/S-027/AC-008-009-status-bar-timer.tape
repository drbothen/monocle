# AC-008 + AC-009 — Two-row status bar + Waiting: Ns elapsed timer
# BC-2.06.019 PC-1/PC-2 (mode indicator "[DASHBOARD]"/"[OVERLAY N]"; drops: N in yellow)
# BC-2.06.020 PC-1/PC-2/PC-3 (breadcrumb: Dashboard>Sessions, Overlay [1 prompt]/[N prompts])
# BC-2.06.021 PC-1 (exact canonical hint strings: Dashboard and Overlay)
# BC-2.06.020 PC-1 (AC-009: "Waiting: Ns" elapsed timer in overlay widget)
# BC-2.06.019 PC-7 (page-level: mode indicator + drop counter; integration render tests)
#
# Evidence via overlay_render.rs and render_frame_integration.rs TestBackend tests.

Output docs/demo-evidence/S-027/AC-008-009-status-bar-timer.gif
Output docs/demo-evidence/S-027/AC-008-009-status-bar-timer.webm

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

# AC-008: mode_indicator_text for Dashboard/Overlay/Filtering/Fullscreen
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_019_mode_indicator_text_dashboard_returns_dashboard_label test_BC_2_06_019_mode_indicator_text_overlay_includes_depth test_BC_2_06_019_mode_indicator_text_filtering_returns_filtering_label test_BC_2_06_019_mode_indicator_text_fullscreen_returns_fullscreen_label 2>&1 | tail -12"
Enter
Sleep 10s

# AC-008: drop_counter_span: None when 0, Some("drops: N") in yellow when >0
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_019_drop_counter_span_returns_none_when_zero test_BC_2_06_019_drop_counter_span_returns_some_with_text_when_nonzero test_BC_2_06_019_render_status_bar_drop_zero_renders_mode_indicator test_BC_2_06_019_render_status_bar_drop_nonzero_renders_dropped_label_in_yellow 2>&1 | tail -12"
Enter
Sleep 10s

# AC-008: breadcrumb derivation and exact canonical hint strings
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_020_render_status_bar_breadcrumb_dashboard_sessions test_BC_2_06_020_render_status_bar_breadcrumb_overlay_singular_prompt test_BC_2_06_020_render_status_bar_breadcrumb_overlay_plural_prompts test_BC_2_06_021_render_status_bar_hint_line_dashboard_contains_key_hints test_BC_2_06_021_render_status_bar_hint_line_overlay_contains_trace_stub_binding 2>&1 | tail -12"
Enter
Sleep 10s

# AC-009: "Waiting: Ns" timer present in overlay widget header/footer
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_020_overlay_header_shows_elapsed_timer_waiting_prefix 2>&1 | tail -8"
Enter
Sleep 10s
Sleep 2s
