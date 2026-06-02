# AC-010 — Integration: Event Ribbon + Sessions filter wired into render_frame and dispatch_key_event
# BC-2.06.018 PC-1/PC-5 (EventRibbon rendered in 40% area via production render_frame path)
# BC-2.06.006 PC-1/PC-2/PC-8 (Sessions filter input box + no-match sentinel via render_frame)
# AC-010 §1 and §2 (render_frame wiring + dispatch_key_event wiring)
#
# Evidence via render_frame_integration_s028.rs TestBackend integration tests:
#   - test_BC_2_06_018_AC010_render_frame_dashboard_shows_event_ribbon_content (AC-010 §1a)
#   - test_BC_2_06_006_AC010_render_frame_filtering_mode_shows_filter_input_box (AC-010 §1b)
#   - test_BC_2_06_006_AC010_render_frame_filtering_zero_match_shows_sentinel (AC-010 §1b zero-match)
#   - test_BC_2_06_006_AC010_dispatch_filter_entry_and_exit_through_production_path (AC-010 §2 filter)
#   - test_BC_2_06_018_AC010_C1_scroll_down_variant_is_dead_in_dispatch (adv_pass4, ScrollDown wired)
#   - test_BC_2_06_018_AC010_C1_scroll_up_variant_is_dead_in_dispatch (adv_pass4, ScrollUp wired)
#
# Success path: render_frame renders hook type names in right 40% area; "/ foo_" filter box rendered.
# Error path: zero-match query renders SESSIONS_FILTER_NO_MATCH sentinel; filter exit returns to Dashboard.

Output docs/demo-evidence/S-028/AC-010-render-frame-dispatch-integration.gif
Output docs/demo-evidence/S-028/AC-010-render-frame-dispatch-integration.webm

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

# AC-010 §1: render_frame wires EventRibbon in 40% area and filter input in Filtering mode
Type "cargo test -p monocle-tui --test render_frame_integration_s028 -- test_BC_2_06_018_AC010_render_frame_dashboard_shows_event_ribbon_content test_BC_2_06_006_AC010_render_frame_filtering_mode_shows_filter_input_box test_BC_2_06_006_AC010_render_frame_filtering_zero_match_shows_sentinel 2>&1 | tail -12"
Enter
Sleep 18s

# AC-010 §2: dispatch_key_event handles filter entry/exit via production path
Type "cargo test -p monocle-tui --test render_frame_integration_s028 -- test_BC_2_06_006_AC010_dispatch_filter_entry_and_exit_through_production_path 2>&1 | tail -10"
Enter
Sleep 15s

# AC-010 §2 ScrollDown/ScrollUp arms wired (adv_pass4 C1 tests)
Type "cargo test -p monocle-tui --test adv_pass4_scroll_dispatch -- test_BC_2_06_018_AC010_C1_scroll_down_variant_is_dead_in_dispatch test_BC_2_06_018_AC010_C1_scroll_up_variant_is_dead_in_dispatch 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
