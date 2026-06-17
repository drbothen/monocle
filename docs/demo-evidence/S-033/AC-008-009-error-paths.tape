# VHS Demo — S-033 SessionManager Error Paths
# Covers AC-008, AC-009, AC-009b, AC-009c, AC-009d
#
# AC-008: BinaryNotFound -> SessionError::EngineError(BinaryNotFound); no process spawned; IPC code "binary_not_found"
# AC-009: sidecar write failure -> orphan-kill SIGTERM -> SIGKILL; Err(SidecarWriteFailed); no registry entry
# AC-009b: spawner.spawn() OS error -> SessionError::SpawnFailed; no sidecar; no registry entry; IPC code "spawn_failed"
# AC-009c: default spawn_recipe() returns UnsupportedOperation("spawn_recipe")
# AC-009d: UnsupportedOperation maps to wire code "spawn_unsupported" (NOT "invalid_request")
#
# Demo strategy: run the targeted unit tests for each error variant.

# === Output Configuration ===
Output AC-008-009-error-paths.gif
Output AC-008-009-error-paths.webm

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

# === Demo: AC-008 — BinaryNotFound propagation ===
# spawn_recipe() returns EngineError::BinaryNotFound; no OS process; IPC code "binary_not_found"

Type "cargo test -p monocle-runtime -- 'test_BC_2_08_001_binary_not_found_propagation|test_BC_2_08_001_binary_not_found_ipc_code_is_binary_not_found|test_BC_2_08_001_session_error_to_code_binary_not_found_maps_correctly' 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-009 — Sidecar write failure + orphan-kill ===
# spawner succeeds; sidecar fails; SIGTERM -> SIGKILL; no registry entry

Type "cargo test -p monocle-runtime -- test_BC_2_08_001_sidecar_write_fail_orphan_kill 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-009b — SpawnFailed (OS error from spawner.spawn()) ===
# spawn_recipe() succeeds; spawner.spawn() returns Err; IPC code "spawn_failed"

Type "cargo test -p monocle-runtime -- 'test_BC_2_08_001_spawn_failed_os_error_returns_spawn_failed_code|test_BC_2_08_001_spawn_failed_ipc_code_is_spawn_failed' 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s

# === Demo: AC-009c + AC-009d — UnsupportedOperation default impl -> "spawn_unsupported" ===
# default spawn_recipe() returns UnsupportedOperation; IPC handler maps to "spawn_unsupported" NOT "invalid_request"

Type "cargo test -p monocle-runtime -- 'test_BC_2_03_008_default_spawn_recipe_unsupported_operation|test_BC_2_03_008_EC_112_unsupported_operation_maps_to_spawn_unsupported|test_BC_2_03_008_spawn_session_unsupported_engine_returns_engine_error' 2>&1"
Enter
Wait+Screen@120s /test result: ok/
Sleep 2s
