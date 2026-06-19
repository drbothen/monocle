# S-037: SessionManager GC Task + rename_session
# Story: S-037 — SessionManager GC Task — Terminated Sessions Removed After 10s Grace Period
# Behavioral Contract: BC-2.08.005
#
# Acceptance Criteria evidenced by this recording:
#
#   AC-001/004/006  GC fires at 10s (tokio::time::pause virtual clock)
#                   → SessionEntry removed; SessionListUpdate published;
#                   timer starts at FIRST Terminated; duplicate does not reset.
#   AC-002/003/008  sidecar + socket deleted (ENOENT-tolerant).
#   AC-009          rename on Terminated → Err(InvalidSessionName{reason:"session terminated"}).
#   AC-011          GC fires when sidecar already deleted (ENOENT → no error; session removed).
#   AC-012          Two sessions terminate independently; two independent GC tasks.
#   rename-success  rename on Running → display_name updated; SessionListUpdate broadcast
#                   carries new name; never SessionStateChanged.
#
# Format: WEBM only (no GIF — repo bloat policy DEMO-BINARY-ARTIFACTS-DEVELOP).

Output docs/demo-evidence/S-037/s037-gc-rename.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 17
Set Width 1200
Set Height 680
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

# === Hidden Setup — navigate to worktree ===
Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-037 && clear"
Enter
Sleep 2s
Show

# === AC-001/AC-004/AC-006: GC fires after 10s; timer not reset by duplicate Terminated ===
Type "# AC-001/004/006 — GC fires at 10s; duplicate Terminated does NOT reset timer"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_005_terminated_session_gc_after_10s test_BC_2_08_005_duplicate_terminated_does_not_reset_gc -- 2>&1 | grep -E 'test_BC_2_08_005|test result'"
Enter
Sleep 8s

# === AC-002/AC-003/AC-008/AC-011: sidecar + socket deleted; ENOENT-tolerant ===
Type "# AC-002/003/008/011 — sidecar+socket deleted; ENOENT tolerated; GC continues"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_005_gc_sidecar_enoent_no_error test_BC_2_08_005_gc_wired_via_real_transition_to_terminated -- 2>&1 | grep -E 'test_BC_2_08_005|test result'"
Enter
Sleep 8s

# === AC-009: rename on Terminated → Err(InvalidSessionName{reason:"session terminated"}) ===
Type "# AC-009 — rename on Terminated → Err(InvalidSessionName)"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_005_rename_on_terminated_fails -- 2>&1 | grep -E 'test_BC_2_08_005|test result'"
Enter
Sleep 6s

# === AC-012: two sessions terminate independently ===
Type "# AC-012 — two independent GC tasks; no interference between concurrent GC tasks"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_005_two_sessions_terminate_independently -- 2>&1 | grep -E 'test_BC_2_08_005|test result'"
Enter
Sleep 8s

# === rename success path ===
Type "# rename success — display_name updated; SessionListUpdate; no SessionStateChanged"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_005_rename_on_running_succeeds test_BC_2_08_005_rename_broadcast_carries_new_display_name test_BC_2_08_005_session_list_carries_rename_after_rename_session -- 2>&1 | grep -E 'test_BC_2_08_005|test result'"
Enter
Sleep 8s

# === Full BC-2.08.005 suite — all 10 tests green ===
Type "# Full BC-2.08.005 suite — 10/10 green"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_005 -- 2>&1 | grep -E 'running [0-9]+ tests|test_BC_2_08_005|test result'"
Enter
Sleep 8s

Sleep 2s
