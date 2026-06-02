# AC-007 — Event Ribbon keyboard navigation (j/k/↓/↑/G/gg/Enter in EventRibbon focus)
# BC-2.06.018 PC-5 (j/↓ scrolls toward older events; k/↑ toward newer; G/gg jumps)
# BC-2.06.018 AC-010 §2 (dispatch_key_event handles scroll actions; focus-discriminated)
#
# Evidence via render_frame_integration_s028.rs and adv_pass5_scroll_real_key.rs tests:
#   - test_BC_2_06_018_AC010_scroll_j_dispatches_scroll_down (j → offset 0→1)
#   - test_BC_2_06_018_AC010_scroll_down_arrow_dispatches_scroll_down (↓ → offset 0→1)
#   - test_BC_2_06_018_AC010_scroll_j_twice_advances_two_rows (j×2 → offset 0→2)
#   - test_BC_2_06_018_AC010_scroll_k_dispatches_scroll_up (k from 3→2)
#   - test_BC_2_06_018_AC010_scroll_k_at_row0_clears_pinned_top (k to row 0 clears pinned_top)
#   - test_BC_2_06_018_AC010_F1_real_j_resolves_to_scroll_down_in_event_ribbon_focus (resolve_binding)
#   - test_BC_2_06_018_AC010_F1_real_k_resolves_to_scroll_up_in_event_ribbon_focus (resolve_binding)
#   - test_BC_2_06_018_AC010_F1_dispatch_real_j_scrolls_ribbon_not_sessions_cursor (focus discrimin.)
#   - test_BC_2_06_018_AC007_G_jumps_to_oldest_event (G → last index; pinned_top=true)
#   - test_BC_2_06_018_AC007_gg_jumps_to_newest_event (gg → row 0; pinned_top=false)
#
# Success path: j/↓ increments offset; k/↑ decrements; G jumps to oldest; gg jumps to newest (row 0).
# Edge path: j in Sessions focus still moves sessions cursor (focus discrimination regression guards).

Output docs/demo-evidence/S-028/AC-007-ribbon-keyboard-navigation.gif
Output docs/demo-evidence/S-028/AC-007-ribbon-keyboard-navigation.webm

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

# AC-007 core scroll: j/↓/k/↑ scroll dispatch via render_frame_integration_s028
Type "cargo test -p monocle-tui --test render_frame_integration_s028 -- test_BC_2_06_018_AC010_scroll_j_dispatches_scroll_down test_BC_2_06_018_AC010_scroll_down_arrow_dispatches_scroll_down test_BC_2_06_018_AC010_scroll_j_twice_advances_two_rows test_BC_2_06_018_AC010_scroll_k_dispatches_scroll_up test_BC_2_06_018_AC010_scroll_k_at_row0_clears_pinned_top 2>&1 | tail -12"
Enter
Sleep 18s

# AC-007 binding resolution: j/k in EventRibbon focus resolve to ScrollDown/ScrollUp (adv_pass5)
Type "cargo test -p monocle-tui --test adv_pass5_scroll_real_key -- test_BC_2_06_018_AC010_F1_real_j_resolves_to_scroll_down_in_event_ribbon_focus test_BC_2_06_018_AC010_F1_real_k_resolves_to_scroll_up_in_event_ribbon_focus test_BC_2_06_018_AC010_F1_dispatch_real_j_scrolls_ribbon_not_sessions_cursor 2>&1 | tail -10"
Enter
Sleep 15s

# AC-007 G/gg jumps: G to oldest (last index); gg to newest (row 0)
Type "cargo test -p monocle-tui --test render_frame_integration_s028 -- test_BC_2_06_018_AC007_G_jumps_to_oldest_event test_BC_2_06_018_AC007_gg_jumps_to_newest_event 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
