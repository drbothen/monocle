# VHS Demo — S-034 Ordering Invariants + Error Paths + Full Suite
# WEBM only — NO GIF (D-333)
#
# Covers:
# AC-002: SessionStateChanged{Terminating} BEFORE SessionListUpdate (ordering invariant)
# AC-007: idempotency confirmed via full suite green
# AC-011: kill on unknown session_id → Err(SessionNotFound) / wire code "session_not_found"
# All 18 S-034 tests passing (red gate + ruling_i + med_findings)
#
# Demo strategy: show the ordering test (BC-2.08.008 Invariant 4), the session-not-found
# error, the Ruling J watchdog test (watchdog fires on monitor EOF), then run all 18
# S-034 tests together to confirm green suite.

# === Output Configuration (WEBM only — D-333) ===
Output AC-002-007-011-ordering-errors-full-suite.webm

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

# === Demo: AC-002 — SessionStateChanged{Terminating} BEFORE SessionListUpdate ===
# test_BC_2_08_008_state_changed_ordering_on_kill:
#   - SessionStateChanged{Terminating} at index N in FIFO
#   - SessionListUpdate at index N+1 (adjacent, no messages between)
#   - Both emitted under single mutex hold (BC-2.08.008 Invariant 4 / Ruling G)

Type "cargo test -p monocle-runtime --test s034_kill_session_red_gate -- test_BC_2_08_008_state_changed_ordering_on_kill --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-011 — kill on unknown session_id → Err(SessionNotFound) ===
# test_BC_2_08_003_kill_session_not_found:
#   - kill_session("nonexistent-id") → Err(SessionNotFound)
# test_BC_2_08_003_kill_session_not_found_wire_code:
#   - session_error_to_code(Kill, SessionNotFound) → "session_not_found"

Type "cargo test -p monocle-runtime --test s034_kill_session_red_gate -- 'test_BC_2_08_003_kill_session_not_found|test_BC_2_08_003_kill_session_not_found_wire_code' --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: Ruling J watchdog — SIGKILL to harness child PID on monitor EOF ===
# test_BC_2_08_003_ruling_I_watchdog_fires_when_kill_confirm_monitor_gets_eof:
#   - kill_confirm_monitor connection drops (EOF) without StateChanged{Terminated}
#   - watchdog fires → SIGKILL to harness child PID (Ruling J — not session-host PID alone)
#   - state forced → Terminated; SessionStateChanged{Terminated} broadcast

Type "cargo test -p monocle-runtime --test s034_ruling_i_validation -- test_BC_2_08_003_ruling_I_watchdog_fires_when_kill_confirm_monitor_gets_eof --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: Full S-034 test suite (all 18 tests green) ===
# Runs all three S-034 test modules:
# - s034_kill_session_red_gate (12 tests)
# - s034_ruling_i_validation (2 tests)
# - s034_med_findings (4 tests)

Type "cargo test -p monocle-runtime --test s034_kill_session_red_gate --test s034_ruling_i_validation --test s034_med_findings -- --test-threads=1 2>&1 | tail -30"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s
