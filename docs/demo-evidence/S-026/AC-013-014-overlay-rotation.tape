# VHS Demo — S-026 AC-013/AC-014
# AC-013: Up/Down rotates stack when len > 1 (BC-2.06.009 PC-1)
# AC-014: Single-item rotation is a no-op (BC-2.06.009 EC-065)
# Also covers AC-009/AC-010: binding layer isolation

Output docs/demo-evidence/S-026/AC-013-014-overlay-rotation.gif
Output docs/demo-evidence/S-026/AC-013-014-overlay-rotation.webm

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

Type "cargo test -p monocle-tui --test overlay_rotation 2>&1 | tail -4"
Enter
Sleep 12s
Sleep 2s
