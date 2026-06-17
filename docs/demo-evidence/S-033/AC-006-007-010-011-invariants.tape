# VHS Demo — S-033 SessionManager Invariants
# Covers AC-006, AC-007, AC-010, AC-011
#
# AC-006: session_id uniqueness; UUID collision -> Err(SessionIdCollision); IPC handler retry once
# AC-007: sidecar written via tempfile::persist (atomic rename); no partial writes observable
# AC-010: SessionStateChanged for EVERY state transition; no silent transitions
# AC-011: broadcast dispatched to ALL connected TUI clients; no clients -> discarded no error
#
# Also covers B-001 production wiring, B-003 SO_PEERCRED, B-005 daemon-field preservation,
# MED-002 collision retry, MED-003 concurrent monitor pair ordering (Ruling G),
# and MED-004 degraded_env flag.

# === Output Configuration ===
Output AC-006-007-010-011-invariants.gif
Output AC-006-007-010-011-invariants.webm

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
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-033 && clear"
Enter
Sleep 2s
Show

# === Demo: AC-006 — UUID collision handling ===
# spawn_session() returns Err(SessionIdCollision) on collision; IPC handler retries once;
# second consecutive collision surfaces error to client

Type "cargo test -p monocle-runtime -- 'test_BC_2_08_001_EC_152_ipc_handler_regenerates_on_first_collision_succeeds|test_BC_2_08_001_EC_152_ipc_handler_second_collision_sends_error|test_BC_2_08_001_spawn_returns_err_collision_when_id_already_exists|test_BC_2_08_001_ipc_handler_two_attempt_retry_on_collision' 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-007 — Atomic sidecar write (tempfile::persist) ===
# No partial writes; concurrent reader never sees truncated sidecar

Type "cargo test -p monocle-runtime -- 'test_BC_2_08_001_invariant_sidecar_write_is_atomic|test_BC_2_08_001_invariant_partial_write_detector_catches_truncation' 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-010 + AC-011 — StateChanged broadcast to all clients ===
# SessionStateChanged on every transition; broadcast to all TUI clients; no clients -> no error

Type "cargo test -p monocle-runtime -- 'test_BC_2_08_008_session_state_changed_broadcast_to_all_clients|test_BC_2_08_008_session_state_changed_new_state_is_launching_on_spawn|test_BC_2_08_008_no_clients_connected_broadcast_discarded_no_error|test_BC_2_08_008_rename_session_does_not_emit_session_state_changed' 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: Full S-033 blocker red gate suite (B-001 thru MED-004) ===
# All 14 production-wiring integration tests green

Type "cargo test -p monocle-runtime --test s033_blocker_red_gate -- --test-threads=1 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s
