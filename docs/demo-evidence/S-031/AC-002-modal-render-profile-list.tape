Output AC-002-modal-render-profile-list.gif
Output AC-002-modal-render-profile-list.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-002 (BC-2.07.005 PC-2/PC-3):
# render_frame renders centered modal listing all profiles from harness_profiles.
# Active profile for CWD pre-selected via resolve_profile_for_dir.
# Empty profiles: "No profiles configured — add profiles to config.json" (em-dash).
#
# SUCCESS PATH: modal renders "Profile Picker" title + profile IDs in buffer
# ERROR PATH: empty profiles shows exact canonical em-dash message

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-002: modal renders profile list"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_render_frame_renders_picker_modal_when_picker_is_some -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-002: modal shows all profile IDs"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_render_frame_picker_modal_shows_profile_list -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-002 edge: empty profiles — exact em-dash message"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_render_frame_picker_modal_shows_no_profiles_message -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-002: active profile * marker uses per-dir lookup (not first-match)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker_adv_pass2 BC_2_07_005_active_marker_uses_per_dir_not_first_match -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
