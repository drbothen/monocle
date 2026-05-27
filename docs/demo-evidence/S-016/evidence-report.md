---
document_type: evidence-report
story_id: S-016
title: "Demo Evidence — S-016: Daemon Binary Crate Init + CLI Subcommands"
producer: vsdd-factory:demo-recorder
recorded: 2026-05-27
total_tests: 37
passed: 37
failed: 0
verdict: PASS
---

# Evidence Report — S-016: Daemon Binary Crate Init + CLI Subcommands

## Summary

All 37 behavioral tests pass across four test suites. Every acceptance criterion
(AC-001 through AC-014) has passing test evidence covering both success and error paths.

| Suite | Tests | Pass | Fail | Evidence File |
|-------|-------|------|------|---------------|
| monocle-runtime: lock_file_lifecycle (runtime_dir filter) | 4 | 4 | 0 | `runtime-dir-tests.log` |
| monocle: cli_daemon_start | 15 | 15 | 0 | `start-tests.log` |
| monocle: cli_daemon_stop | 14 | 14 | 0 | `stop-tests.log` |
| monocle: daemon_detach | 4 | 4 | 0 | `detach-tests.log` |
| **Total** | **37** | **37** | **0** | |

---

## AC-to-Evidence Mapping

### AC-001..AC-004: resolve_runtime_dir() fallback chain

Evidence: `runtime-dir-tests.log`

| AC | Test | Path | Result |
|----|------|------|--------|
| AC-001 | test_BC_2_01_005_env_override_monocle_runtime_dir | success: MONOCLE_RUNTIME_DIR env var honored | PASS |
| AC-002 | test_BC_2_01_005_lock_file_path_is_in_runtime_dir | success: lock file resolves within runtime dir | PASS |
| AC-003 | test_BC_2_01_005_runtime_dir_created_recursively | success: missing parent dirs created | PASS |
| AC-004 | test_BC_2_01_005_runtime_dir_created_with_0o700 | success: permissions set to 0o700 | PASS |

---

### AC-005..AC-008: daemon start subcommand

Evidence: `start-tests.log`

| AC | Test(s) | Path | Result |
|----|---------|------|--------|
| AC-005 | test_ac_005_start_no_lock_file_exits_zero | success: no prior lock file, exits 0 | PASS |
| AC-005 | test_ac_005_start_no_stdout_on_success | success: stdout is empty | PASS |
| AC-005 | test_ac_005_stale_lock_treated_as_absent | error path: stale lock treated as no-daemon | PASS |
| AC-005 | test_ac_005_no_autostart_env_does_not_affect_daemon_start | success: MONOCLE_NO_AUTOSTART does not interfere | PASS |
| AC-006 | test_ac_006_start_already_running_exits_1_with_stderr | error path: daemon already running, exits 1 with stderr | PASS |
| AC-006 | test_ac_006_start_already_running_stderr_format | error path: stderr message format verified | PASS |
| AC-006 | test_ac_006_already_running_produces_no_stdout | error path: stdout empty on conflict | PASS |
| AC-007 | test_ac_007_start_timeout_exits_1_with_stderr | error path: start timeout, exits 1 with stderr | PASS |
| AC-007 | test_ac_007_start_timeout_stderr_format | error path: timeout stderr message format verified | PASS |
| AC-007 | test_ac_007_timeout_produces_no_stdout | error path: stdout empty on timeout | PASS |
| AC-008 | test_ac_008_exit_code_zero_on_success | success: exit code 0 on clean start | PASS |
| AC-008 | test_ac_008_exit_code_1_already_running | error path: exit code 1 when already running | PASS |
| AC-008 | test_ac_008_exit_code_1_timeout | error path: exit code 1 on start timeout | PASS |
| AC-008 | test_ac_008_exit_code_70_runtime_dir_unresolvable | error path: exit code 70 when runtime dir cannot be resolved | PASS |
| AC-008 | test_ac_008_exit_code_71_internal_error | error path: exit code 71 on internal error | PASS |

---

### AC-009..AC-013: daemon stop subcommand

Evidence: `stop-tests.log`

| AC | Test(s) | Path | Result |
|----|---------|------|--------|
| AC-009 | test_ac_009_stop_running_daemon_exits_zero | success: running daemon stopped cleanly, exits 0 | PASS |
| AC-009 | test_ac_009_stop_no_stdout_on_success | success: stdout empty | PASS |
| AC-009 | test_ac_009_stop_no_stderr_on_success | success: stderr empty | PASS |
| AC-009 | test_ac_009_malformed_lock_file_exits_1 | error path: malformed lock file, exits 1 | PASS |
| AC-009 | test_ac_009_malformed_lock_file_stderr_format | error path: malformed lock file stderr format verified | PASS |
| AC-009 | test_ac_009_exit_code_70_runtime_dir_unresolvable | error path: exit code 70 when runtime dir unresolvable | PASS |
| AC-010 | test_ac_010_stop_no_lock_file_exits_1_with_stderr | error path: no lock file present, exits 1 with stderr | PASS |
| AC-010 | test_ac_010_stop_no_lock_file_stderr_format | error path: no lock file stderr format verified | PASS |
| AC-011 | test_ac_011_stop_stale_lock_exits_1_with_stderr | error path: stale lock (dead PID), exits 1 with stderr | PASS |
| AC-011 | test_ac_011_stop_stale_lock_stderr_format | error path: stale lock stderr format verified | PASS |
| AC-012 | test_ac_012_stop_timeout_exits_2_with_stderr | error path: SIGTERM ignored, graceful timeout, exits 2 with stderr | PASS |
| AC-012 | test_ac_012_stop_timeout_stderr_format | error path: timeout stderr format verified | PASS |
| AC-012 | test_ac_012_stop_timeout_daemon_still_alive_no_sigkill | error path: daemon still alive after timeout, no SIGKILL sent | PASS |
| AC-013 | test_ac_013_no_sigkill_ever_sent | structural: SIGKILL absent from source code | PASS |

---

### AC-014: daemon detachment (double-fork / setsid)

Evidence: `detach-tests.log`

| AC | Test(s) | Path | Result |
|----|---------|------|--------|
| AC-014 | test_ac_014_daemon_subprocess_survives_parent_exit | success: daemon process outlives calling process | PASS |
| AC-014 | test_ac_014_daemon_survives_terminal_session_end | success: daemon not killed on SIGHUP / session end | PASS |
| AC-014 | test_ac_014_daemon_not_child_of_calling_shell | success: daemon PID is not a child of the invoking shell | PASS |
| AC-014 | test_ac_014_lock_file_exists_after_start_exits | success: lock file written before `monocle daemon start` returns | PASS |

---

## Coverage Verdict

All 14 acceptance criteria (AC-001 through AC-014) are covered with passing test evidence.
Both success and error paths are demonstrated for each AC that has an error path defined.
The no-SIGKILL invariant (AC-013) is verified structurally (source scan) and behaviorally
(timeout test confirms daemon remains alive and unkilled after graceful-stop timeout).
