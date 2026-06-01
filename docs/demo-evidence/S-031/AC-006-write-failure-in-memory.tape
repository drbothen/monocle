Output AC-006-write-failure-in-memory.gif
Output AC-006-write-failure-in-memory.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-006 (BC-2.07.005 PC-5c / INV-3):
# If write_config() fails, TUI renders transient error in status bar:
# "Config save failed: <error>". In-memory selection IS applied regardless.
# Picker closes. Persisted file unchanged.
#
# SUCCESS PATH (forced write failure): status_message set + in-memory applied + picker closed
# ERROR PATH: empty-CWD commit must not write project_profiles[""] (MAJOR-2)

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-006: write failure sets status_message, applies in-memory, closes picker"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_pc5c_write_failure_applies_in_memory_and_sets_status_message -- --nocapture 2>&1 | tail -5"
Enter
Sleep 6s

Type "# AC-006 edge: empty-CWD commit must not write project_profiles['']"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker_adv_pass2 BC_2_07_005_commit_production_empty_cwd_does_not_write_empty_dir_key -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-006 edge: wrapper err-branch empty-CWD guard (ADV pass-4 regression)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_picker_adv_pass4 -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
