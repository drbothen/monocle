# VHS Demo — S-026 AC-016
# AC-016: payload_to_modal() conversion — Bash/Read/Edit/Write/Generic dispatch (BC-2.06.024)
# Includes Phase-1 critical path: Edit/Write with None/None content → ToolPayload::Generic
# (not ToolPayload::Edit with empty strings — avoids empty diff pane)

Output docs/demo-evidence/S-026/AC-016-payload-conversion.gif
Output docs/demo-evidence/S-026/AC-016-payload-conversion.webm

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

Type "cargo test -p monocle-tui --test overlay_push_pop payload_to_modal 2>&1 | tail -4"
Enter
Sleep 12s
Sleep 2s
