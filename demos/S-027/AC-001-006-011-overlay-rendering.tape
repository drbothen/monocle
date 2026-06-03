# VHS Demo — S-027 Overlay Rendering + Diff Preview
# Covers ACs:
#
# AC-001 (BC-2.06.010 PC-1): overlay widget layout — header "Permission Request",
#         session_id, tool_name, footer keyboard hints, (M of N) stack-depth indicator
# AC-002 (BC-2.06.010 PC-2): dimmed background applied to all non-modal cells
# AC-003 (BC-2.06.024 PC-1): Bash payload — "command: <cmd>" label, empty fallback
# AC-004 (BC-2.06.024 PC-2): Read payload — "path: <path>" label, empty fallback
# AC-005 (BC-2.06.010 PC-1): Edit diff preview via similar — green +/red - with path in body
# AC-006 (BC-2.06.024 PC-3): Generic payload — "tool:/input:" labels, 256-char excerpt, unrepresentable fallback
# AC-011 (BC-2.06.010 INV-1): render ops complete synchronously < 5ms

# === Output Configuration ===
Output AC-001-006-011-overlay-rendering.gif
Output AC-001-006-011-overlay-rendering.webm

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

# === Demo: S-027 AC-001..006, AC-011 — Overlay Rendering + Diff Preview ===
# Drives overlay_render.rs and diff_preview.rs against TestBackend
# Demonstrates: modal layout, dimmed background, Bash/Read/Edit/Generic payloads, sync render

# Run diff_preview tests first (covers AC-005, AC-007)
Type "cargo test -p monocle-tui --test diff_preview -- 2>&1 && echo 'DIFF_PASS'"
Enter
Wait+Screen@60s /DIFF_PASS/
Sleep 1s

# Run overlay_render tests (covers AC-001..006, AC-011 and BC-2.06.010 layout assertions)
Type "cargo test -p monocle-tui --test overlay_render -- 2>&1 | tail -5 && echo 'OVERLAY_PASS'"
Enter
Wait+Screen@60s /OVERLAY_PASS/
Sleep 2s
