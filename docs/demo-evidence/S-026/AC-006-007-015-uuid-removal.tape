# VHS Demo — S-026 AC-006/AC-007/AC-015
# AC-006: PermissionPromptResolved → retain() removes by UUID regardless of position (BC-2.06.023 PC-1)
# AC-007: Unknown prompt_id is silent no-op — no WARN/ERROR (BC-2.06.023 PC-3)
# AC-015: Empty stack after retain() collapses to Dashboard (BC-2.06.023 PC-4)

Output docs/demo-evidence/S-026/AC-006-007-015-uuid-removal.gif
Output docs/demo-evidence/S-026/AC-006-007-015-uuid-removal.webm

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

Type "cargo test -p monocle-tui --test overlay_uuid_removal 2>&1 | tail -4"
Enter
Sleep 12s
Sleep 2s
