# Demo Evidence Report — S-038

**Story:** S-038 — SessionManager Hook Auto-Injection  
**BC:** BC-2.08.006 — Single canonical hooks-settings.json writer + SpawnOptions.hooks_settings_path population  
**Branch:** story/S-038-session-manager-hook-injection  
**Recorded:** 2026-06-19  

---

## Recordings

| File | Format | Size |
|------|--------|------|
| `s038-hook-injection.webm` | WEBM (VHS) | 6.0 MB |
| `s038-hook-injection.tape` | VHS source | 4.3 KB |
| `s038-test-output.txt` | Plain-text transcript | 4.7 KB |

---

## Acceptance Criterion Coverage

| AC | Description | Test(s) | Result |
|----|-------------|---------|--------|
| AC-001 | 4 URL-bearing hooks (PreToolUse/Notification/Stop/UserPromptSubmit), array-of-objects per BC-2.04.010; PostToolUse/PreCompact empty arrays; lock.app="monocle"; no SessionStart | `test_BC_2_08_006_hooks_settings_json_content` `test_BC_2_08_006_production_writer_always_emits_lock_app` | PASS |
| AC-002 | Atomic write (tempfile::persist) + file mode 0o600 | `test_BC_2_08_006_hooks_settings_json_atomic_write` | PASS |
| AC-003 | SpawnOptions.hooks_settings_path populated before spawn_recipe (S-038 single-writer mandate) | `test_BC_2_08_006_spawn_options_hooks_settings_path_populated` | PASS |
| AC-004 | EC-182 re-write at spawn time with real config (real port URL + lock.app, not empty) | `test_BC_2_08_006_ec182_rewrites_with_real_config` `test_BC_2_08_006_missing_settings_file_rewrites_at_spawn` | PASS |
| AC-005 | Startup write failure aborts the daemon (DaemonStartError::HooksSettingsWriteFailure) | `test_BC_2_08_006_startup_write_fail_aborts_daemon` | PASS |
| AC-006 | Non-UTF-8 path → EngineError::InvalidPath → session_error_to_code maps to "invalid_spawn_arg" | `test_BC_2_08_006_non_utf8_hooks_path_returned_from_spawn_recipe` | PASS |
| BC-2.08.006 INV-2 | Daemon-written hooks-settings.json has lock.app="monocle" (integration test) | `daemon_start_sequence::test_BC_2_08_006_daemon_startup_hooks_settings_has_lock_app_monocle` | PASS |

**Total: 9/9 tests PASS (8 unit + 1 integration)**

---

## Recording Segment Guide

The WEBM is structured in segments, each narrated with a comment line:

| Segment | AC(s) | Tests run |
|---------|-------|-----------|
| 0:00 | AC-001 | `test_BC_2_08_006_hooks_settings_json_content` + `production_writer_always_emits_lock_app` |
| ~0:30 | AC-002 | `test_BC_2_08_006_hooks_settings_json_atomic_write` |
| ~1:00 | AC-003 | `test_BC_2_08_006_spawn_options_hooks_settings_path_populated` |
| ~1:30 | AC-004 | `test_BC_2_08_006_ec182_rewrites_with_real_config` + `missing_settings_file_rewrites_at_spawn` |
| ~2:00 | AC-005 | `test_BC_2_08_006_startup_write_fail_aborts_daemon` |
| ~2:30 | AC-006 | `test_BC_2_08_006_non_utf8_hooks_path_returned_from_spawn_recipe` |
| ~3:00 | BC-2.08.006 INV-2 | Integration: `daemon_start_sequence::test_BC_2_08_006_daemon_startup_hooks_settings_has_lock_app_monocle` |
| ~3:30 | All | Full suite: 8/8 unit tests green |

---

## Notes

- S-038 is daemon/runtime-internal. Evidence is captured via behavioral test runs, not a running GUI.
- S-038 does NOT append `--settings` to the spawn command line; that is S-045's responsibility. AC-003 only verifies `hooks_settings_path` is populated in `SpawnOptions` before `spawn_recipe` is called.
- EC-182 guard: if `hooks-settings.json` is absent at spawn time, `SessionManager::spawn_session()` re-writes it with the real config (real port + lock.app) before delegating to the engine.
- No `.gif` generated (repo bloat policy `DEMO-BINARY-ARTIFACTS-DEVELOP`).
