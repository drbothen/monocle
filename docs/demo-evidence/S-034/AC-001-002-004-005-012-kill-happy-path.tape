# VHS Demo — S-034 SessionManager::kill_session Happy Path
# WEBM only — NO GIF (D-333)
#
# Covers:
# AC-001: kill_session() selects kill path by state; Kill delivered within 500ms
# AC-002: SessionEntry.state → Terminating atomically with Kill send
# AC-004: Terminating → Terminated on session-host StateChanged confirmation
# AC-005: 12s watchdog forces Terminated + SIGKILL (session-host PID) when no confirmation
# AC-012: SessionStateChanged{Terminating/Terminated} emitted on every kill-path transition
#
# Demo strategy: run the red-gate integration tests that exercise the kill happy
# path (Running→Terminating→Terminated confirmation) and the 12s watchdog path,
# showing all tests green with --nocapture output visible.

# === Output Configuration (WEBM only — D-333) ===
Output AC-001-002-004-005-012-kill-happy-path.webm

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

# === Demo: AC-001/AC-002 — Kill within 500ms; Terminating on Kill send ===
# test_BC_2_08_003_kill_session_sigterm_within_500ms:
#   - kill_session() returns Ok(()) in < 500ms (AC-001)
#   - state → Terminating immediately after kill returns (AC-002)
#   - state → Terminated after mock session-host sends StateChanged{Terminated} (AC-004)
#   - sidecar updated atomically to Terminated (AC-004)

Type "cargo test -p monocle-runtime --test s034_kill_session_red_gate -- test_BC_2_08_003_kill_session_sigterm_within_500ms --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-005 — 12s watchdog forces Terminated + publishes SessionStateChanged ===
# test_BC_2_08_003_12s_watchdog:
#   - tokio::time::pause() advances 12s virtual time (no real delay)
#   - watchdog fires → SIGKILL to session-host PID (Ruling J — harness child PID)
#   - state forced → Terminated; sidecar updated; SessionStateChanged{Terminated} broadcast

Type "cargo test -p monocle-runtime --test s034_kill_session_red_gate -- test_BC_2_08_003_12s_watchdog --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-012 — SessionStateChanged ordering (Ruling I: kill-confirm monitor) ===
# test_BC_2_08_003_ruling_I_prompt_kill_reaches_terminated_via_kill_confirm_monitor:
#   - SessionStateChanged{Terminating} BEFORE SessionListUpdate (AC-002/AC-012)
#   - Terminating → Terminated via kill-confirm monitor (Ruling I) when session-host
#     sends HostToDaemon::StateChanged{Terminated} on the existing connection

Type "cargo test -p monocle-runtime --test s034_ruling_i_validation -- test_BC_2_08_003_ruling_I_prompt_kill_reaches_terminated_via_kill_confirm_monitor --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s
