Output AC-008-not-appmode-overlay.gif
Output AC-008-not-appmode-overlay.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-008 (BC-2.07.004 INV-1):
# Profile picker MUST NOT use AppMode::Overlay.
# It MUST be modeled as Option<ProfilePickerState> in App.
# Orthogonal to permission overlays (VecDeque<PromptModal>).
#
# SUCCESS PATH: after open_profile_picker, app.mode is NOT AppMode::Overlay
# SUCCESS PATH: Ctrl-P from Overlay mode opens picker without changing mode

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-008: picker is NOT AppMode::Overlay after open"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_004_invariant_picker_is_not_app_mode_overlay -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-008: Ctrl-P from Overlay mode opens picker, AppMode stays Overlay"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_dispatch_ctrl_p_from_overlay_opens_picker_appmode_unchanged -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-008: Ctrl-P from Filtering mode opens picker (no AppMode guard)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_dispatch_ctrl_p_from_filtering_opens_picker -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
