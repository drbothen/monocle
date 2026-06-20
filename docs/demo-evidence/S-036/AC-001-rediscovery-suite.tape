# S-036 Demo Tape — SessionManager::rediscover_sessions()
# Behavioral evidence: BC-2.08.002 / BC-2.08.004
#
# DEMO FORMAT: WEBM + .tape ONLY (NO GIF)
#
# Modality: test-harness demo (fallback per task directive).
# Rationale: rediscover_sessions() is a daemon-internal feature that runs at
# daemon_start_sequence step 8b. There is no interactive CLI surface for
# rediscovery in this sprint. The 44-test suite covering all 15 ACs is the
# correct evidence vehicle.
#
# AC coverage (all 15 ACs evidenced):
#   AC-001 (BC-2.08.002 setsid / session-host survival):
#     test_BC_2_08_002_session_survives_daemon_graceful_restart
#   AC-002 (step 8b placement before UDS bind):
#     test_BC_2_08_004_rediscovery_completes_before_uds_bind
#   AC-003 (schema v1/v2 accepted; v4+ skipped):
#     test_BC_2_08_004_rediscovery_schema_v1_legacy
#     test_BC_2_08_004_rediscovery_schema_v2_accepted
#     test_BC_2_08_004_rediscovery_schema_v4_future
#   AC-004 (Running: SO_PEERCRED + Attach + 5s timeout):
#     test_BC_2_08_004_rediscovery_running_session_reregistered
#     test_BC_2_08_004_rediscovery_non_responsive_within_5s
#     test_BC_2_08_004_rediscovery_peercred_pid_mismatch_rejected
#   AC-005 (Detached: preserve intent; NO Attach; NO StateChanged):
#     test_BC_2_08_004_rediscovery_detached_peercred_verified_no_attach
#     test_BC_2_08_004_rediscovery_detached_pid_match_registers
#   AC-006 (Terminating: absolute deadline; fire-and-forget Kill; watchdog):
#     test_BC_2_08_004_rediscovery_terminating_elapsed_deadline
#     test_BC_2_08_004_rediscovery_terminating_not_elapsed_deadline
#     test_BC_2_08_004_rediscovery_terminating_null_deadline_new_window
#   AC-007 (Terminated GC; unknown state WARN+delete):
#     test_BC_2_08_004_rediscovery_terminated_state_gc
#     test_BC_2_08_004_rediscovery_unknown_state_string_warn_delete
#   AC-008 (dead PID GC + orphan socket delete):
#     test_BC_2_08_004_rediscovery_dead_pid_gc
#     test_BC_2_08_004_rediscovery_dead_pid_deletes_orphan_socket
#   AC-009 (RediscoveryReport shape):
#     test_BC_2_08_004_rediscovery_report_shape_mixed
#   AC-010 (corrupt sidecar: WARN; delete; continue):
#     test_BC_2_08_004_rediscovery_corrupt_sidecar
#   AC-011 (UDS bind blocked until re-discovery returns):
#     test_BC_2_08_004_rediscovery_completes_before_uds_bind
#   AC-012 (parallel probing via tokio::join_all within 5s for 8 sessions):
#     test_BC_2_08_004_rediscovery_parallelism_8_sessions
#     test_BC_2_08_004_rediscovery_parallelism_8_sessions_sequential_would_exceed_5s
#   AC-013 (Detached intent preserved across restart):
#     test_BC_2_08_004_rediscovery_detached_peercred_verified_no_attach
#   AC-014 (Terminating elapsed deadline -> SIGKILL immediately):
#     test_BC_2_08_004_rediscovery_terminating_elapsed_deadline
#   AC-015 (re-discovered session in InitialState after restart):
#     test_BC_2_08_002_session_survives_daemon_graceful_restart

# === Output — WEBM only (NO GIF) ===
Output docs/demo-evidence/S-036/AC-001-rediscovery-suite.webm

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 24
Set TypingSpeed 30ms

# === Hidden Setup ===
Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-036 && clear"
Enter
Sleep 1s
Show

# -----------------------------------------------------------------------
# Title card
# -----------------------------------------------------------------------
Type "# S-036: rediscover_sessions() — BC-2.08.002 + BC-2.08.004"
Enter
Sleep 400ms
Type "# 44 tests | 15 ACs | daemon restart session survival"
Enter
Sleep 800ms

Hide
Type "clear"
Enter
Sleep 300ms
Show

# -----------------------------------------------------------------------
# Group 1/5: AC-001 + AC-015 — headline behavior (BC-2.08.002)
# Session survives graceful daemon restart; re-discovered in InitialState
# -----------------------------------------------------------------------
Type "# Group 1/5: AC-001+AC-015 — session survives graceful daemon restart (BC-2.08.002)"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --lib 'rediscovery_tests::test_BC_2_08_002_session_survives_daemon_graceful_restart' -- --nocapture 2>&1 | tail -5"
Enter
Sleep 8s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# -----------------------------------------------------------------------
# Group 2/5: AC-002/011/012 — step 8b ordering + parallel probing
# UDS bind blocked; tokio::join_all within 5s for 8 sessions
# -----------------------------------------------------------------------
Type "# Group 2/5: AC-002+AC-011+AC-012 — step 8b ordering; parallel 8-session probe"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --lib 'rediscovery_tests::test_BC_2_08_004_rediscovery_completes_before_uds_bind' 'rediscovery_tests::test_BC_2_08_004_rediscovery_parallelism_8_sessions' 'rediscovery_tests::test_BC_2_08_004_rediscovery_parallelism_8_sessions_sequential_would_exceed_5s' -- --nocapture 2>&1 | tail -6"
Enter
Sleep 8s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# -----------------------------------------------------------------------
# Group 3/5: AC-003..010 — schema versions + all SessionState variants
# v1/v2 accepted; v4+ skipped; Running/Detached/Terminating/Terminated/dead
# -----------------------------------------------------------------------
Type "# Group 3/5: AC-003..010 — schema v1/v2/v4; Running/Detached/Terminated/dead PID"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --lib 'rediscovery_tests::test_BC_2_08_004_rediscovery_schema_v1_legacy' 'rediscovery_tests::test_BC_2_08_004_rediscovery_schema_v4_future' 'rediscovery_tests::test_BC_2_08_004_rediscovery_running_session_reregistered' 'rediscovery_tests::test_BC_2_08_004_rediscovery_detached_peercred_verified_no_attach' 'rediscovery_tests::test_BC_2_08_004_rediscovery_terminated_state_gc' 'rediscovery_tests::test_BC_2_08_004_rediscovery_dead_pid_gc' 'rediscovery_tests::test_BC_2_08_004_rediscovery_corrupt_sidecar' 'rediscovery_tests::test_BC_2_08_004_rediscovery_report_shape_mixed' -- --nocapture 2>&1 | tail -12"
Enter
Sleep 10s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# -----------------------------------------------------------------------
# Group 4/5: AC-006/013/014 — Terminating: elapsed SIGKILL; watchdog; Detached preserved
# -----------------------------------------------------------------------
Type "# Group 4/5: AC-006+AC-013+AC-014 — Terminating absolute deadline; Detached preserved"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --lib 'rediscovery_tests::test_BC_2_08_004_rediscovery_terminating_elapsed_deadline' 'rediscovery_tests::test_BC_2_08_004_rediscovery_terminating_not_elapsed_deadline' 'rediscovery_tests::test_BC_2_08_004_rediscovery_terminating_null_deadline_new_window' -- --nocapture 2>&1 | tail -7"
Enter
Sleep 8s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# -----------------------------------------------------------------------
# Group 5/5: Full suite — all 44 tests green
# -----------------------------------------------------------------------
Type "# Group 5/5: FULL SUITE — 44 tests all green (BC-2.08.002 + BC-2.08.004)"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --lib rediscovery 2>&1 | tail -4"
Enter
Sleep 8s
Sleep 2s
