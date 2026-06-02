Output AC-007-ccr-path-status-bar.gif
Output AC-007-ccr-path-status-bar.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-007 (BC-2.07.005 PC-5 / PC-3):
# After profile selection, detect_ccr() is called; result stored in App.ccr_path.
# Status bar shows "CCR: <path>" or "CCR: none".
# App.ccr_path initialized at TUI startup via detect_ccr(&config).
#
# SUCCESS PATH: detect_ccr called after commit, App.ccr_path updated
# SUCCESS PATH: render_frame shows "CCR: <path>" when Some, "CCR: none" when None

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-007: detect_ccr called after profile switch, ccr_path updated"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_switch BC_2_07_005_detect_ccr_called_and_ccr_path_updated_after_switch -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-007: startup initializes ccr_path via detect_ccr"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_startup_ccr_path_initialized_from_detect_ccr -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-007: status bar shows 'CCR: <path>' when ccr_path is Some"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_render_frame_status_bar_shows_ccr_path_when_some -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-007: status bar shows 'CCR: none' when ccr_path is None"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test render_frame_integration_s031 BC_2_07_005_render_frame_status_bar_shows_ccr_none_when_absent -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
