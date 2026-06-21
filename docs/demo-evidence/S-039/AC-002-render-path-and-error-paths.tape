# VHS Demo — S-039 PTY Output Pipeline
# AC-002: Render path + error/edge paths
#
# Demonstrates specific behavioral contracts:
#   SUCCESS: render_frame_embedded_terminal_uses_focused_parser
#   ERROR:   unknown_session_id_drop (unknown session silently dropped)
#   ERROR:   pending_pty_bytes_cap_drops_oldest (bounded buffer overflow)
#   ERROR:   dump_window_timeout_force_resolves (timeout force-resolves)
#   RECONNECT: disconnect_clears_dump_state_retains_parsers
#
# NOTE: Interactive EmbeddedTerminal entry is S-040-gated (keyboard forwarding).

# === Output Configuration (WEBM only — project directive D-333, no GIF) ===
Output docs/demo-evidence/S-039/AC-002-render-path-and-error-paths.webm

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

# === Hidden Setup ===
Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-039"
Enter
Sleep 1s
Show

# === Success path: render frame uses focused parser ===
Type "# SUCCESS: render path — focused vt100 parser drives tui-term render"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test bc_2_09_001_pty_output_pipeline render_frame_embedded_terminal_uses_focused_parser 2>&1 | tail -4"
Enter
Sleep 4s

# === Error path: unknown session id silently dropped ===
Type "# ERROR: unknown session id dropped (not panicked)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test bc_2_09_001_pty_output_pipeline unknown_session_id_drop 2>&1 | tail -4"
Enter
Sleep 3s

# === Error path: bounded buffer overflow — cap drops oldest ===
Type "# ERROR: buffer cap exceeded — oldest bytes dropped"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test bc_2_09_001_pty_output_pipeline pending_pty_bytes_cap_drops_oldest 2>&1 | tail -4"
Enter
Sleep 3s

# === Error path: dump window timeout force-resolves ===
Type "# ERROR: dump-window timeout force-resolves auto-attach"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test bc_2_09_001_pty_output_pipeline dump_window_timeout_force_resolves 2>&1 | tail -4"
Enter
Sleep 3s

# === Reconnect cleanup: disconnect clears dump state ===
Type "# RECONNECT: disconnect clears dump state, parsers retained"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test bc_2_09_001_pty_output_pipeline disconnect_clears_dump_state_retains_parsers 2>&1 | tail -4"
Enter
Sleep 3s

Sleep 2s
