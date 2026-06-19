# S-035 Demo Tape — SessionManager::attach_session / detach_session
# Behavioral evidence: BC-2.08.007 / BC-2.08.008
#
# DEMO FORMAT DIRECTIVE (D-333): WEBM + .tape ONLY — NO GIF
# Covers all 18 acceptance criteria across 7 test groups:
#   AC-001/002  SO_PEERCRED + DaemonToHost::Attach + ScrollbackChunk*+ScrollbackDumpComplete within 5s
#   AC-003/004  host_conn.proxy_task=Some; state→Running; SessionStateChanged{Running} before SessionListUpdate
#   AC-005      Scrollback chunks forwarded to TUI clients
#   AC-006/007  DaemonToHost::Detach; proxy_task aborted; state→Detached; session-host stays alive
#   AC-008      Sidecar updated atomically via tempfile::persist
#   AC-009      Mutex serializes concurrent attaches; no duplicate proxy_task
#   AC-010      Retired ScrollbackDump rejected
#   AC-011/012  Idempotency: Running-attach Ok, Detached-detach Ok
#   AC-013      EC-187 ConnectFailed→Terminated
#   AC-014      EC-186 Detach on Launching→SessionNotReady
#   AC-015      BC-2.08.008: SessionStateChanged on every transition
#   Ruling L    Kill attached session via proxy_task fast-path
#   TOCTOU      Terminated state not resurrected to Detached

# === Output — WEBM only per D-333 directive (NO GIF) ===
Output docs/demo-evidence/S-035/s035-attach-detach.webm

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
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-035 && clear"
Enter
Sleep 1s
Show

# -----------------------------------------------------------------------
# Group 1: Core attach/detach cycle — AC-002..007/015
# -----------------------------------------------------------------------
Type "# S-035 — attach_session / detach_session behavioral evidence"
Enter
Sleep 600ms
Type "# Group 1/7: AC-002..007/015 — core attach/detach cycle (BC-2.08.007)"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --test s035_attach_detach_red_gate test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive -- --nocapture 2>&1"
Enter
Sleep 30s

Hide
Type "clear"
Enter
Sleep 500ms
Show

# -----------------------------------------------------------------------
# Group 2: 5s timeout (EC-188) + event ordering (AC-004/007/015)
# -----------------------------------------------------------------------
Type "# Group 2/7: EC-188 timeout->Terminated; BC-2.08.008 ordering (AC-004/007/015)"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --test s035_attach_detach_red_gate test_BC_2_08_007_attach_5s_timeout_session_host_dead test_BC_2_08_008_state_changed_ordering_on_attach_detach -- --nocapture 2>&1"
Enter
Sleep 30s

Hide
Type "clear"
Enter
Sleep 500ms
Show

# -----------------------------------------------------------------------
# Group 3: 4 attach-failure dispositions
# -----------------------------------------------------------------------
Type "# Group 3/7: 4 attach-failure dispositions (EC-187/PeerCred/Timeout/ProtocolError)"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --test s035_attach_detach_red_gate test_BC_2_08_007_attach_running_session_dead test_BC_2_08_007_attach_peer_cred_mismatch_transitions_to_terminated test_BC_2_08_007_attach_protocol_error_stays_detached test_attach_timeout_sigterm_uses_pid_sigterm_fn_seam -- --nocapture 2>&1"
Enter
Sleep 30s

Hide
Type "clear"
Enter
Sleep 500ms
Show

# -----------------------------------------------------------------------
# Group 4: Idempotency (AC-011/012/014)
# -----------------------------------------------------------------------
Type "# Group 4/7: Idempotency (AC-011/012/014)"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --test s035_attach_detach_red_gate test_BC_2_08_007_attach_running_idempotent test_BC_2_08_007_detach_detached_idempotent test_BC_2_08_007_detach_launching_session_not_ready -- --nocapture 2>&1"
Enter
Sleep 30s

Hide
Type "clear"
Enter
Sleep 500ms
Show

# -----------------------------------------------------------------------
# Group 5: Ruling L — kill attached session via proxy_task fast-path
# -----------------------------------------------------------------------
Type "# Group 5/7: Ruling L — kill attached session via proxy_task fast-path"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --test s035_attach_detach_red_gate test_kill_attached_session_fast_path -- --nocapture 2>&1"
Enter
Sleep 30s

Hide
Type "clear"
Enter
Sleep 500ms
Show

# -----------------------------------------------------------------------
# Group 6: TOCTOU + sidecar + concurrency + retired-protocol + integration
# -----------------------------------------------------------------------
Type "# Group 6/7: TOCTOU + sidecar + concurrency + retired-protocol + cycle (AC-008/009/010)"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --test s035_attach_detach_red_gate test_BC_2_08_007_sidecar_updated_on_detach test_BC_2_08_007_concurrent_attach_no_duplicate_proxy_task test_BC_2_08_007_concurrent_attach_single_proxy_task_invariant test_BC_2_08_007_retired_scrollback_dump_rejected test_BC_2_08_007_attach_detach_cycle -- --nocapture 2>&1"
Enter
Sleep 30s

Hide
Type "clear"
Enter
Sleep 500ms
Show

# -----------------------------------------------------------------------
# Group 7: Full suite — all 18 tests green
# -----------------------------------------------------------------------
Type "# Group 7/7: FULL SUITE — 18 tests covering all BCs for S-035"
Enter
Sleep 500ms
Type "cargo test -p monocle-runtime --test s035_attach_detach_red_gate -- --nocapture 2>&1"
Enter
Sleep 30s
Sleep 2s
