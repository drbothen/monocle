# VHS Demo — S-026 AC-003/AC-004/AC-005/AC-008
# AC-003: y/Enter → Accept IPC send, modal stays (BC-2.06.011 PC-1)
# AC-004: A → AcceptAlways IPC send, modal stays (BC-2.06.012 PC-1)
# AC-005: n/r → Reject IPC send, modal stays (BC-2.06.013 PC-1)
# AC-008: Esc in Overlay is no-op identity (BC-2.06.014 PC-1)

Output docs/demo-evidence/S-026/AC-003-004-005-008-decision-keybindings.gif
Output docs/demo-evidence/S-026/AC-003-004-005-008-decision-keybindings.webm

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

Type "cargo test -p monocle-tui --test overlay_decision 2>&1 | tail -4"
Enter
Sleep 12s
Sleep 2s
