# S-036 Demo Evidence Report

**Story:** S-036 — SessionManager::rediscover_sessions() — setsid Persistence; All States Handled Within 5s; UDS Bind Blocked

**Behavioral Contracts:** BC-2.08.002 (session-host survives graceful daemon restart) / BC-2.08.004 (all alive sessions visible after restart within 5s; UDS bind blocked until complete)

**Demo Modality:** Test-harness demo (VHS terminal recording of cargo test output).

**Rationale for modality choice:** `rediscover_sessions()` is a daemon-internal function that runs at `daemon_start_sequence` step 8b — before the UDS socket is bound and before any TUI client can connect. There is no interactive CLI surface for triggering or observing rediscovery in this sprint. The 44-test suite is the correct evidence vehicle: each test exercises one or more acceptance criteria via mock session-hosts, fake PeerCred verifiers, and time injection.

---

## Recording

| File | Type | Description |
|------|------|-------------|
| `AC-001-rediscovery-suite.tape` | VHS tape script | Source script — 5 groups, 44 tests |
| `AC-001-rediscovery-suite.webm` | Recording | Rendered terminal demo (1.1 MB) |

---

## AC Coverage Map

| AC | Description | Test(s) Evidenced |
|----|-------------|------------------|
| AC-001 | Session-host survives graceful daemon shutdown (setsid; SIGHUP immune; sidecar intact) | `test_BC_2_08_002_session_survives_daemon_graceful_restart` |
| AC-002 | rediscover_sessions() called at step 8b (after lock file, before UDS bind) | `test_BC_2_08_004_rediscovery_completes_before_uds_bind` |
| AC-003 | Schema v1/v2/v3 accepted; v4+ skipped with WARN | `test_BC_2_08_004_rediscovery_schema_v1_legacy`, `test_BC_2_08_004_rediscovery_schema_v2_accepted`, `test_BC_2_08_004_rediscovery_schema_v4_future` |
| AC-004 | Running/Launching: SO_PEERCRED + Attach + 5s timeout; register Running; PID mismatch SIGTERM | `test_BC_2_08_004_rediscovery_running_session_reregistered`, `test_BC_2_08_004_rediscovery_non_responsive_within_5s`, `test_BC_2_08_004_rediscovery_peercred_pid_mismatch_rejected`, `test_BC_2_08_004_rediscovery_pid_mismatch_sigterms_both_pids` |
| AC-005 | Detached: SO_PEERCRED verified; NO Attach sent; NO SessionStateChanged; register Detached | `test_BC_2_08_004_rediscovery_detached_peercred_verified_no_attach`, `test_BC_2_08_004_rediscovery_detached_pid_match_registers`, `test_BC_2_08_004_rediscovery_detached_peercred_mismatch_no_entry`, `test_BC_2_08_004_rediscovery_detached_pid_mismatch_rejected` |
| AC-006 | Terminating: absolute deadline; elapsed→SIGKILL; not-elapsed→watchdog; null→new 12s window; fire-and-forget Kill; background watchdog | `test_BC_2_08_004_rediscovery_terminating_elapsed_deadline`, `test_BC_2_08_004_rediscovery_terminating_not_elapsed_deadline`, `test_BC_2_08_004_rediscovery_terminating_null_deadline_new_window`, `test_BC_2_08_004_rediscovery_terminating_pid_match_kill_sent`, `test_BC_2_08_004_rediscovery_terminating_watchdog_deadline_emits_broker`, `test_BC_2_08_004_rediscovery_terminating_watchdog_terminated_msg_emits_broker`, `test_BC_2_08_004_rediscovery_terminating_watchdog_socket_close_before_terminated`, `test_BC_2_08_004_rediscovery_watchdog_gc_grace_from_transition_deadline_arm`, `test_BC_2_08_004_rediscovery_watchdog_gc_grace_from_transition_msg_arm`, `test_BC_2_08_004_rediscovery_watchdog_terminated_entry_gcd` |
| AC-007 | Terminated: GC immediately; unknown state string: WARN+delete+skip | `test_BC_2_08_004_rediscovery_terminated_state_gc`, `test_BC_2_08_004_rediscovery_unknown_state_string_warn_delete` |
| AC-008 | Dead PID: delete sidecar + orphan socket; no SessionEntry | `test_BC_2_08_004_rediscovery_dead_pid_gc`, `test_BC_2_08_004_rediscovery_dead_pid_deletes_orphan_socket`, `test_BC_2_08_004_rediscovery_dead_pid_emits_terminated` |
| AC-009 | RediscoveryReport: found_alive + found_dead + errors fields populated correctly | `test_BC_2_08_004_rediscovery_report_shape_mixed` |
| AC-010 | Corrupt sidecar: WARN; delete; continue; RediscoveryError added | `test_BC_2_08_004_rediscovery_corrupt_sidecar`, `test_BC_2_08_004_rediscovery_missing_project_root_corrupt`, `test_BC_2_08_004_rediscovery_missing_required_field_corrupt`, `test_BC_2_08_004_rediscovery_empty_project_root_corrupt` |
| AC-011 | UDS bind blocked until rediscover_sessions() returns (sequential await ordering) | `test_BC_2_08_004_rediscovery_completes_before_uds_bind` |
| AC-012 | Parallel probing via tokio::join_all; 8 sessions within 5s wall-clock | `test_BC_2_08_004_rediscovery_parallelism_8_sessions`, `test_BC_2_08_004_rediscovery_parallelism_8_sessions_sequential_would_exceed_5s` |
| AC-013 | Detached intent preserved — NO force-attach across restart | `test_BC_2_08_004_rediscovery_detached_peercred_verified_no_attach`, `test_BC_2_08_004_rediscovery_detached_pid_match_registers` |
| AC-014 | Terminating with ELAPSED kill_deadline_unix_ms → immediate SIGKILL | `test_BC_2_08_004_rediscovery_terminating_elapsed_deadline` |
| AC-015 | Re-discovered session appears in InitialState after restart | `test_BC_2_08_002_session_survives_daemon_graceful_restart`, `test_BC_2_08_004_rediscovery_mixed_alive_dead_ec159`, `test_BC_2_08_004_rediscovery_alive_pid_socket_missing`, `test_BC_2_08_004_rediscovery_empty_runtime_dir`, `test_BC_2_08_004_rediscovery_unreadable_runtime_dir` |

**Total: 15/15 ACs evidenced. 44 tests. 0 failures.**

---

## Demo Groups in Recording

| Group | ACs | Key assertions shown |
|-------|-----|----------------------|
| 1/5 — Headline behavior | AC-001, AC-015 | `test_BC_2_08_002_session_survives_daemon_graceful_restart` — session-host survives; sidecar intact; InitialState includes session post-restart |
| 2/5 — Ordering + parallelism | AC-002, AC-011, AC-012 | UDS bind blocked until completes_before_uds_bind; 8 sessions parallelism within 5s |
| 3/5 — Schema + state variants | AC-003..010 | Schema v1/v4; Running reregistered; Detached no-attach; Terminated GC; dead PID GC; corrupt sidecar; report shape |
| 4/5 — Terminating deadline | AC-006, AC-013, AC-014 | Elapsed→SIGKILL; not-elapsed→watchdog; null→12s window |
| 5/5 — Full suite | All 15 ACs | 44 tests green; 1.4s wall-clock |
