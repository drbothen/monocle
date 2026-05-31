# VHS Demo — S-026 AC-011/AC-012
# AC-011: TransportEvent::Disconnected clears overlay stack (BC-2.06.016 PC-1, SOQ-3)
# AC-012: InitialState on reconnect restores overlay stack (BC-2.06.016 PC-2)
# Also includes snapshot-window dedup test (BC-2.05.002 Invariant 4)

Output docs/demo-evidence/S-026/AC-011-012-disconnect-restore.gif
Output docs/demo-evidence/S-026/AC-011-012-disconnect-restore.webm

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

Type "cargo test -p monocle-tui --test overlay_disconnect 2>&1 | tail -4"
Enter
Sleep 12s
Sleep 2s
