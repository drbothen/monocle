# VHS Demo — S-031 Profile Picker Open, Navigation, Isolation
# Covers ACs:
#
# AC-001 (BC-2.07.005 PC-1): Ctrl-P dispatches Action::ProfilePicker in ANY AppMode;
#         sets app.profile_picker = Some(ProfilePickerState); app.mode unchanged
# AC-002 (BC-2.07.005 PC-2): modal rendered over current view; profile list from
#         harness_profiles; sticky pre-selection via resolve_profile_for_dir();
#         empty-profiles message when harness_profiles empty
# AC-003 (BC-2.07.004 PC-3): j/down moves selection down (wraps), k/up moves up (wraps),
#         Enter selects, Esc closes without changes
# AC-004 (BC-2.07.004 PC-4): picker keyboard isolation — session nav keys (Tab, Enter for
#         sessions, j/k for session scroll) do NOT fire while picker is open
# AC-008 (BC-2.07.004 INV-1): picker modeled as Option<ProfilePickerState> — NOT AppMode::Overlay

# === Output Configuration ===
Output AC-001-004-008-picker-open-nav.gif
Output AC-001-004-008-picker-open-nav.webm

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

# === Demo: S-031 AC-001..004, AC-008 — Profile Picker Open, Navigation, Isolation ===
# Drives profile_picker.rs — open/close/navigation/invariant tests
# Demonstrates: Ctrl-P in all AppModes, sorted profile list, sticky pre-select,
# j/k wrap navigation, Esc without change, keyboard isolation, non-Overlay modeling

Type "cargo test -p monocle-tui --test profile_picker -- 2>&1 && echo 'ALL_TESTS_PASSED'"
Enter
Wait+Screen@120s /ALL_TESTS_PASSED/
Sleep 2s
