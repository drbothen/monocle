# VHS Demo — S-034 Idempotency + Session-Host Kill Handler
# WEBM only — NO GIF (D-333)
#
# Covers:
# AC-003: session-host DaemonToHost::Kill handler (SIGTERM→10s→SIGKILL; Goodbye; socket removal)
# AC-007: idempotency — kill on Terminating → Ok(); kill on Terminated → Ok(); no dup Kill
#
# Demo strategy: run the idempotent-on-terminated test (genuine Terminated seam via
# insert_terminated_session_for_test), idempotent-on-terminating (double kill on Running→
# Terminating session), and the MED-004 test that verifies the kill-confirm monitor uses
# the same connection as the post-spawn monitor (Ruling I — session-host handler integration).

# === Output Configuration (WEBM only — D-333) ===
Output AC-003-007-idempotent-and-session-host.webm

# === Dependency Check ===
Require cargo

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 720
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

# === Hidden Setup ===
Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-034 && clear"
Enter
Sleep 2s
Show

# === Demo: AC-007 — idempotent kill on Terminated (no dup Kill, no Terminating broadcast) ===
# test_BC_2_08_003_kill_session_idempotent_on_terminated:
#   - Session inserted directly in Terminated state (F-S034-ADV-LOW-001 seam)
#   - kill_session() → Ok(()) without sending DaemonToHost::Kill
#   - No SessionStateChanged{Terminating} emitted

Type "cargo test -p monocle-runtime --test s034_kill_session_red_gate -- test_BC_2_08_003_kill_session_idempotent_on_terminated --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-007 — idempotent kill on Terminating (double kill → both Ok(())) ===
# test_BC_2_08_003_kill_session_idempotent_on_terminating:
#   - First kill: Running → Terminating, Ok(())
#   - Second kill on Terminating: Ok(()) immediately, no dup Kill, watchdog not re-spawned

Type "cargo test -p monocle-runtime --test s034_kill_session_red_gate -- test_BC_2_08_003_kill_session_idempotent_on_terminating --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-003 — session-host kill-confirm monitor (Ruling I) ===
# test_MED_004_BC_2_08_003_kill_confirmation_uses_same_connection_as_post_spawn_monitor:
#   - Terminating → Terminated happens via the SAME connection as the post-spawn monitor
#   - Session-host sends HostToDaemon::StateChanged{Terminated} + Goodbye on existing conn
#   - Sidecar updated; SessionStateChanged{Terminated} broadcast

Type "cargo test -p monocle-runtime --test s034_med_findings -- test_MED_004_BC_2_08_003_kill_confirmation_uses_same_connection_as_post_spawn_monitor --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s
