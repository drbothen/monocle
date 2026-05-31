Output AC-010-no-client-disconnect-variant.gif
Output AC-010-no-client-disconnect-variant.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-010: monocle-tui MUST NOT reference ClientToServer::ClientDisconnect.
# That variant does not exist (removed in BC-2.06.004 v1.1.0).
# Disconnection is detected exclusively via TransportEvent::Disconnected.
# Evidence: structural test + successful compilation with no ClientDisconnect reference.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-010: no ClientDisconnect variant in monocle-tui codebase"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test startup_connect ac010 -- --nocapture"
Enter
Sleep 4s

Type "cargo build -p monocle-tui 2>&1 | tail -3"
Enter
Sleep 5s
