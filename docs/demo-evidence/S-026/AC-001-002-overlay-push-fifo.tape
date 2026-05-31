# VHS Demo — S-026 AC-001/AC-002
# AC-001: PermissionPromptQueued push + idempotent insert (BC-2.06.008 PC-1, BC-2.05.002 Inv-4)
# AC-002: FIFO ordering preserved (BC-2.06.008 PC-2)
#
# Evidence: overlay_push_pop tests exercising push-from-Dashboard → Overlay mode,
# FIFO ordering across 3 prompts, and idempotent duplicate discard.

Output docs/demo-evidence/S-026/AC-001-002-overlay-push-fifo.gif
Output docs/demo-evidence/S-026/AC-001-002-overlay-push-fifo.webm

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

Type "cargo test -p monocle-tui --test overlay_push_pop 2>&1 | tail -4"
Enter
Sleep 12s
Sleep 2s
