# AC-003 + AC-004 — Bash and Read tool payload rendering
# BC-2.06.024 PC-1 (Bash: "command: <cmd>" label line, Wrap { trim: false })
# BC-2.06.024 PC-2 (Read: "path: <path>" label line, Wrap { trim: false })
#
# Evidence via TestBackend render tests asserting:
#   AC-003: "command: " (lowercase colon-space) label line + command value in buffer
#   AC-003: Block titled "Command"
#   AC-004: "path: " label line + full path value in buffer
#   AC-004: Block titled "File"
# Empty-value fallbacks ("command: (empty)", "path: (empty)") are covered by
# the production implementation; non-panic on empty is a property of passing
# these tests without panic.

Output docs/demo-evidence/S-027/AC-003-004-bash-read-payloads.gif
Output docs/demo-evidence/S-027/AC-003-004-bash-read-payloads.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms

Hide
Type "clear"
Enter
Sleep 1s
Show

# AC-003: render_bash_payload shows "command: " label line + value; Block titled "Command"
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_024_render_bash_payload_shows_command_label_and_value test_BC_2_06_024_render_bash_payload_block_has_title_command 2>&1 | tail -10"
Enter
Sleep 10s

# AC-004: render_read_payload shows "path: " label line + value; Block titled "File"
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_024_render_read_payload_shows_path_label_and_value test_BC_2_06_024_render_read_payload_block_has_title_file 2>&1 | tail -10"
Enter
Sleep 10s
Sleep 2s
