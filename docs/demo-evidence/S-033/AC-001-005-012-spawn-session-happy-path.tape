# VHS Demo — S-033 SessionManager::spawn_session Happy Path
# Covers AC-001, AC-002, AC-003, AC-004, AC-005, AC-012
#
# AC-001: SpawnAck sent before spawn_session(); spawn_recipe() called first; spawner called within 2s
# AC-002: SessionEntry{Launching} in registry after successful spawn_session()
# AC-003: sidecar written atomically (schema_version 3) at <runtime_dir>/session-<id>.json
# AC-004: spawn_session() returns Ok(session_id) without waiting for Running
# AC-005: SessionStateChanged{Launching} published before SessionListUpdate
# AC-012: SpawnAck delivered before SessionStateChanged{Launching} (causal step ordering)
#
# Demo strategy: run the monocle-runtime unit tests that exercise the spawn path
# (B-002 production broker test + all spawn unit tests), then the MED-001 real
# session-host end-to-end test, showing all tests green.

# === Output Configuration ===
Output AC-001-005-012-spawn-session-happy-path.gif
Output AC-001-005-012-spawn-session-happy-path.webm

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

# === Demo: Spawn Session Happy Path (AC-001 through AC-005, AC-012) ===
# Unit tests: spawn_session entry created, SpawnAck ordering, sidecar schema-v3,
# SessionStateChanged{Launching} before SessionListUpdate

Type "cargo test -p monocle-runtime -- 'test_BC_2_08_001_spawn_session_entry_created_within_2s|test_BC_2_08_001_ipc_arm_spawn_ack_precedes_state_changed_launching|test_BC_2_08_001_sidecar_written_with_schema_v3|test_BC_2_08_001_spawn_session_returns_ok_without_waiting_for_running|test_BC_2_08_008_session_state_changed_before_session_list_update_on_spawn|test_BC_2_08_008_spawn_ack_before_state_changed_launching|test_BC_2_08_001_spawn_session_entry_fields_correct_on_launch' 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: MED-001 — Real session-host binary reaches Running state ===
# AC-004: spawn returns Ok without waiting for Running
# Running transition happens asynchronously via post-spawn monitor

Type "cargo test -p monocle-runtime --test s033_blocker_red_gate -- test_BC_2_08_001_MED001_real_session_host_reaches_running --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: Production broker integration (B-002) ===
# AC-005, AC-012: daemon_start_sequence() wired; broadcast reaches subscriber

Type "cargo test -p monocle-runtime --test s033_blocker_red_gate -- test_BC_2_08_001_B002_production_broker_receives_state_changed --nocapture 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s
