# VHS Demo — S-026 IPC Wire Traversal (AC-003/AC-004/AC-005 wire-level proof)
# Critical evidence: PermissionDecision frame physically traverses the wire to daemon side.
# test_f_s026_adv5_crit001_decision_message_traverses_wire_to_daemon
# This was the Pass-5 CRITICAL fix: ipc_tx must be Some after IPC setup so that
# decision keybindings can physically send to the daemon's receiving channel.

Output docs/demo-evidence/S-026/AC-003-005-wire-traversal.gif
Output docs/demo-evidence/S-026/AC-003-005-wire-traversal.webm

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

Type "cargo test -p monocle-tui --test ipc_outbound_writer 2>&1 | tail -5"
Enter
Sleep 12s
Sleep 2s
