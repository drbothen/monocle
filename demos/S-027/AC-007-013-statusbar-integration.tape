# VHS Demo — S-027 Two-Row Status Bar + [t] Stub + Integration Wiring
# Covers ACs:
#
# AC-007 (BC-2.06.015 INV-1): similar crate in monocle-tui ONLY — purity boundary enforced
# AC-008 (BC-2.06.019 PC-2,7 / BC-2.06.020 PC-1,3,5 / BC-2.06.021 PC-3):
#         two-row status bar — upper breadcrumb (AppMode-derived) + drops:N yellow right-aligned;
#         lower hint line (context-sensitive Builtin bindings); coexistence: drops never suppressed
# AC-009 (BC-2.06.020 PC-1): overlay timer "Waiting: <N>s" in footer, updates each tick
# AC-010 (BC-2.06.021 PC-1): FIFO order — "(1 of N)" = oldest prompt active
# AC-012 (BC-2.06.019 PC-1 / BC-2.06.020 PC-1 / BC-2.06.021 PC-3):
#         overlay + status bar wired into production render_frame (TestBackend integration drive)
# AC-013 (BC-2.06.015 PC-1/2/3/7): [t] trace-to-source stub sets status_message on lower row;
#         coexistence: drops:N on upper row unchanged when [t] pressed

# === Output Configuration ===
Output AC-007-013-statusbar-integration.gif
Output AC-007-013-statusbar-integration.webm

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

# === Demo: S-027 AC-007..013 — Two-Row Status Bar, [t] Stub, Integration ===
# Drives overlay_render.rs (status bar tests) and render_frame_integration.rs
# Demonstrates: breadcrumb row, drop counter coexistence, hint line, [t] stub, purity boundary,
# and full production render_frame path wired into AppMode::Overlay and Dashboard

Type "cargo test -p monocle-tui --test overlay_render --test render_frame_integration -- 2>&1 && echo 'ALL_TESTS_PASSED'"
Enter
Wait+Screen@120s /ALL_TESTS_PASSED/
Sleep 2s
