# VHS Demo — S-031 Profile Switch, Atomic Config Save, CCR Path, Integration
# Covers ACs:
#
# AC-005 (BC-2.07.005 PC-5): Enter writes selected id to config.project_profiles[current_dir]
#         via write_config (atomic tempfile::persist); closes picker; logs INFO profile switched;
#         detect_ccr called
# AC-006 (BC-2.07.005 PC-5c): write_config error → status bar "Config save failed: <error>";
#         in-memory profile applied regardless of write failure; picker closes
# AC-007 (BC-2.07.005 PC-5): detect_ccr called after every selection (success or failure);
#         App.ccr_path updated; status bar shows "CCR: <path>" or "CCR: none" from first render;
#         App.ccr_path initialized at TUI startup
# AC-009 (BC-2.07.005 INV-2): write_config uses tempfile::persist — semgrep rule fires on
#         direct std::fs::write; atomic write invariant enforced
# AC-010 (BC-2.07.005 PC-1,7,8): full integration — dispatch_key_event routes Ctrl-P to
#         open_profile_picker; render_frame renders picker when open; navigation/commit/close
#         routed correctly; Esc does NOT call write_config; ccr_path in status bar from startup

# === Output Configuration ===
Output AC-005-010-switch-config-ccr-integration.gif
Output AC-005-010-switch-config-ccr-integration.webm

# === Dependency Check ===
Require cargo

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 680
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

# === Hidden Setup ===
Hide
Type "cd /Users/jmagady/Dev/monocle && clear"
Enter
Sleep 2s
Show

# === Demo: S-031 AC-005..010 — Profile Switch, Config Save, CCR, Integration ===
# Drives profile_switch.rs and render_frame_integration_s031.rs
# Demonstrates: atomic config write, write-failure in-memory apply, detect_ccr on switch,
# CCR path in status bar, sticky pre-selection per directory, keyboard isolation, full render path

Type "cargo test -p monocle-tui --test profile_switch --test render_frame_integration_s031 -- 2>&1 && echo 'ALL_TESTS_PASSED'"
Enter
Wait+Screen@120s /ALL_TESTS_PASSED/
Sleep 2s
