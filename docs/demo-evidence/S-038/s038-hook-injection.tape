# S-038: SessionManager Hook Auto-Injection
# Story: S-038 — Single canonical hooks-settings.json writer + SpawnOptions.hooks_settings_path
# Behavioral Contract: BC-2.08.006
#
# Acceptance Criteria evidenced by this recording:
#
#   AC-001  hooks-settings.json content: 4 URL-bearing hooks (PreToolUse/Notification/Stop/
#           UserPromptSubmit, array-of-hook-objects per BC-2.04.010),
#           PostToolUse/PreCompact empty arrays, lock.app="monocle", no SessionStart.
#   AC-002  Atomic write (tempfile::persist) + 0o600.
#   AC-003  SpawnOptions.hooks_settings_path populated before spawn_recipe.
#   AC-004  EC-182 re-write with real config at spawn time (real port + lock.app, not empty).
#   AC-005  Startup write failure aborts the daemon (DaemonStartError::HooksSettingsWriteFailure).
#   AC-006  Non-UTF-8 path → EngineError::InvalidPath → "invalid_spawn_arg".
#
# Format: WEBM only (no GIF — repo bloat policy DEMO-BINARY-ARTIFACTS-DEVELOP).

Output docs/demo-evidence/S-038/s038-hook-injection.webm

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
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-038 && clear"
Enter
Sleep 2s
Show

# === AC-001: hooks-settings.json content — 4 URL hooks, empty PostToolUse/PreCompact, lock.app, no SessionStart ===
Type "# AC-001 — JSON content: 4 URL hooks, empty PostToolUse/PreCompact, lock.app=monocle, no SessionStart"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_006_hooks_settings_json_content test_BC_2_08_006_production_writer_always_emits_lock_app -- 2>&1 | grep -E 'test_BC_2_08_006_(hooks_settings_json_content|production_writer)|test result'"
Enter
Sleep 10s

# === AC-002: Atomic write (tempfile::persist) + mode 0o600 ===
Type "# AC-002 — atomic write via tempfile::persist; file mode 0o600"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_006_hooks_settings_json_atomic_write -- 2>&1 | grep -E 'test_BC_2_08_006_hooks_settings_json_atomic_write|test result'"
Enter
Sleep 10s

# === AC-003: SpawnOptions.hooks_settings_path populated before spawn_recipe ===
Type "# AC-003 — SpawnOptions.hooks_settings_path set before spawn_recipe (S-038 single-writer)"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_006_spawn_options_hooks_settings_path_populated -- 2>&1 | grep -E 'test_BC_2_08_006_spawn_options|test result'"
Enter
Sleep 10s

# === AC-004: EC-182 re-write at spawn with real config (real port + lock.app) ===
Type "# AC-004 — EC-182 re-write at spawn time: real port URL + lock.app in file, not empty"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_006_ec182_rewrites_with_real_config test_BC_2_08_006_missing_settings_file_rewrites_at_spawn -- 2>&1 | grep -E 'test_BC_2_08_006_(ec182|missing)|test result'"
Enter
Sleep 10s

# === AC-005: Startup write failure aborts daemon ===
Type "# AC-005 — startup write failure → DaemonStartError::HooksSettingsWriteFailure → abort"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_006_startup_write_fail_aborts_daemon -- 2>&1 | grep -E 'test_BC_2_08_006_startup|test result'"
Enter
Sleep 10s

# === AC-006: Non-UTF-8 path → EngineError::InvalidPath → "invalid_spawn_arg" ===
Type "# AC-006 — non-UTF-8 path → EngineError::InvalidPath → session_error_to_code: invalid_spawn_arg"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_006_non_utf8_hooks_path_returned_from_spawn_recipe -- 2>&1 | grep -E 'test_BC_2_08_006_non_utf8|test result'"
Enter
Sleep 10s

# === BC-2.08.006 Invariant 2: daemon-written file has lock.app=monocle (integration test) ===
Type "# BC-2.08.006 INV-2: daemon startup writes lock.app=monocle (integration test)"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime --test daemon_start_sequence test_BC_2_08_006 -- 2>&1 | grep -E 'test_BC_2_08_006|test result'"
Enter
Sleep 10s

# === Full BC-2.08.006 suite — all 8 unit tests + 1 integration test green ===
Type "# Full BC-2.08.006 suite — 8 unit + 1 integration = 9/9 green"
Enter
Sleep 800ms
Type "cargo test -p monocle-runtime test_BC_2_08_006 -- 2>&1 | grep -E 'running [0-9]+ tests|test_BC_2_08_006|test result'"
Enter
Sleep 10s

Sleep 2s
