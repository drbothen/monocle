Output AC-010-integration-dispatch-render.gif
Output AC-010-integration-dispatch-render.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-010 (BC-2.07.005 PC-1/PC-7/PC-8):
# Full integration: dispatch_key_event routes Ctrl-P to open_profile_picker;
# render_frame renders picker when open; picker-local keys routed before resolve_binding;
# Esc calls close_profile_picker without write_config; App.ccr_path at startup.
#
# SUCCESS PATH: Ctrl-P dispatch from Dashboard opens picker
# SUCCESS PATH: render_frame renders picker modal (TestBackend evidence)
# SUCCESS PATH: per-directory pre-selection (not first-match)

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-010: Ctrl-P dispatch from Dashboard opens picker"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_dispatch_ctrl_p_from_dashboard_opens_picker -- --nocapture 2>&1 | tail -5"
Enter
Sleep 6s

Type "# AC-010: render_frame renders picker modal with 'Profile Picker' title"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_render_frame_renders_picker_modal_when_picker_is_some -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-010: per-directory pre-selection (not first-match)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_per_directory_preselection_uses_current_dir_not_first_match -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-010: normalization round-trip + per-dir pre-selection"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_004_invariant_write_read_normalization_verbatim_consistent -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
