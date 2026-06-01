Output AC-004-keyboard-isolation.gif
Output AC-004-keyboard-isolation.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-004 (BC-2.07.004 INV-4 / BC-2.07.005 PC-9):
# While picker is open, ALL key events are consumed by the picker.
# Session nav keys (Tab, j/k for session scroll) do NOT fire while picker is open.
# Picker keyboard handler takes precedence over resolve_binding().
#
# SUCCESS PATH: dispatch_key_event routes Down to picker_select_next (not session scroll)
# SUCCESS PATH: Esc closes picker without mode change
# SUCCESS PATH: Tab does not leak to focus navigation while picker open

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-004: Down arrow while picker open routes to picker, not session scroll"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_picker_open_down_arrow_routes_to_picker_not_session_scroll -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-004: Tab while picker open does NOT move focus"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_picker_open_tab_key_isolated_does_not_move_focus -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-004: Esc routes to close_profile_picker, not mode transition"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_picker_open_esc_routes_to_close_picker -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
