# S-042 AC-003 — Daemon Resize Pipeline: ResizePane IPC → resize_session → DaemonToHost::Resize
# Demonstrates: IPC dispatch arm (AC-013), zero-dim clamp (AC-014), resize_session forwarding (AC-015),
#               session-not-found WARN-drop, session-manager-none WARN-drop, session-host-dead WARN-drop (AC-016)
# Covers: BC-2.09.006 AC-013..AC-016 (daemon leg: full end-to-end routing from IPC to session-host)
# Test file: bc_2_09_006_daemon_resize.rs (9 tests in monocle-runtime)

Output AC-003-daemon-resize-pipeline-tests.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1280
Set Height 720
Set Theme "Dracula"
Set Padding 16
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-042"
Enter
Sleep 2s
Show

Type "# S-042: daemon resize pipeline tests (BC-2.09.006 AC-013..AC-016)"
Enter
Sleep 500ms

Type "cargo test -p monocle-runtime --test bc_2_09_006_daemon_resize 2>&1"
Enter
Sleep 10s
