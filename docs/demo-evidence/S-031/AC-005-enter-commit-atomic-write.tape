Output AC-005-enter-commit-atomic-write.gif
Output AC-005-enter-commit-atomic-write.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-005 (BC-2.07.005 PC-5a/b):
# On Enter: write selected profile id to config.project_profiles[current_dir],
# call write_config atomically via tempfile::persist, close picker (set None),
# log INFO: profile switched, call detect_ccr.
#
# SUCCESS PATH: commit closes picker, writes project_profiles entry
# SUCCESS PATH: Enter while picker open commits and closes (dispatch_key_event)
# ERROR PATH: selecting same profile is idempotent (EC-108)

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-005: Enter commit closes picker"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_switch BC_2_07_005_commit_selection_closes_picker -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-005: commit writes project_profiles[current_dir]"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_switch BC_2_07_005_commit_selection_writes_project_profiles_entry -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-005: dispatch Enter while picker open commits and closes picker"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_picker_open_enter_commits_and_closes_picker -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-005 edge: selecting same profile is idempotent (EC-108)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_switch BC_2_07_005_ec108_select_same_profile_is_idempotent -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
