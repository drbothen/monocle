Output AC-001-ctrl-p-opens-picker.gif
Output AC-001-ctrl-p-opens-picker.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-001 (BC-2.07.005 PC-1):
# Ctrl-P dispatches Action::ProfilePicker and sets app.profile_picker = Some(..)
# WITHOUT changing AppMode. Not AppMode::Overlay. picker is Option<ProfilePickerState>.
#
# SUCCESS PATH: open_profile_picker sets picker Some, AppMode unchanged (Dashboard)
# ERROR PATH (idempotent): second open_profile_picker is no-op (EC-110)

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-001: Ctrl-P opens picker (AppMode unchanged)"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_005_ctrl_p_sets_profile_picker_some -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-001 idempotent: second Ctrl-P is no-op (EC-110)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_005_ctrl_p_idempotent_when_picker_already_open -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-001: AppMode stays Dashboard after open"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker BC_2_07_005_open_picker_does_not_change_app_mode -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
