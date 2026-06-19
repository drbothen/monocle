---
story_id: S-037
title: "SessionManager GC Task — Terminated Sessions Removed After 10s Grace Period"
bc: BC-2.08.005
recorded: 2026-06-19
recorder: vsdd-factory:demo-recorder
---

# Demo Evidence Report — S-037

## Story

**S-037** implements the GC task (`tokio::spawn`) that removes `SessionEntry` records from
`SessionManager.sessions` after a 10-second grace period following `Terminated` transition,
deletes the `session-state.json` sidecar and per-session UDS socket file (ENOENT-tolerant),
and publishes `SessionListUpdate` to the broker. It also adds `rename_session()` — a
metadata-only method that updates `display_name`, re-persists the sidecar, and publishes
`SessionListUpdate` (never `SessionStateChanged`), with a state guard that rejects renames on
Terminated-in-grace sessions.

## Recordings

| File | Format | Size | Description |
|------|--------|------|-------------|
| `s037-gc-rename.webm` | WEBM | 3.2 MB | VHS terminal recording covering all evidenced ACs |
| `s037-gc-rename.tape` | VHS source | 3.5 KB | Tape script (reproducible) |
| `s037-test-output.txt` | Plain text | 4.3 KB | Full `cargo test` transcript (supplementary) |

Note: No .gif produced per repo-bloat policy DEMO-BINARY-ARTIFACTS-DEVELOP (directive D-333).

## Acceptance Criteria Coverage

| AC | Description | Evidence | Test(s) |
|----|-------------|----------|---------|
| AC-001 | GC fires at 10s ±1s after first `Terminated` | WEBM 0:00–0:20 | `test_BC_2_08_005_terminated_session_gc_after_10s` |
| AC-002 | `session-state.json` deleted on GC fire | WEBM 0:20–0:40 | `test_BC_2_08_005_gc_wired_via_real_transition_to_terminated` |
| AC-003 | UDS socket file deleted on GC fire (best-effort) | WEBM 0:20–0:40 | `test_BC_2_08_005_gc_wired_via_real_transition_to_terminated` |
| AC-004 | `SessionListUpdate` published within one broker tick of registry removal | WEBM 0:00–0:20 | `test_BC_2_08_005_terminated_session_gc_after_10s` |
| AC-006 | Timer starts at FIRST Terminated; duplicate does NOT reset | WEBM 0:00–0:20 | `test_BC_2_08_005_duplicate_terminated_does_not_reset_gc` |
| AC-008 | `std::fs::remove_file`; ENOENT not an error | WEBM 0:20–0:40 | `test_BC_2_08_005_gc_sidecar_enoent_no_error` |
| AC-009 | `rename_session()` on Terminated → `Err(InvalidSessionName{reason:"session terminated"})` | WEBM 0:40–0:55 | `test_BC_2_08_005_rename_on_terminated_fails` |
| AC-011 | GC fires when sidecar already deleted: ENOENT → no error; session removed; `SessionListUpdate` published | WEBM 0:20–0:40 | `test_BC_2_08_005_gc_sidecar_enoent_no_error` |
| AC-012 | Two sessions terminate independently; two independent GC tasks; no interference | WEBM 0:55–1:15 | `test_BC_2_08_005_two_sessions_terminate_independently` |
| rename-success | `rename_session()` on Running: `display_name` updated; `SessionListUpdate` broadcast carries new name; never `SessionStateChanged` | WEBM 1:15–1:40 | `test_BC_2_08_005_rename_on_running_succeeds`, `test_BC_2_08_005_rename_broadcast_carries_new_display_name`, `test_BC_2_08_005_session_list_carries_rename_after_rename_session` |

### Not Directly Evidenced (validated via prior story or future story)

| AC | Reason |
|----|--------|
| AC-005 | TUI-side vt100::Parser cleanup; daemon responsibility (emit `SessionListUpdate`) already covered by AC-004. TUI-side validated in SS-09 stories. |
| AC-007 | Orphaned sidecar immediate GC: boundary between S-037 and S-036; immediate-GC path is in S-036. |
| AC-010 | Daemon restart re-discovery path: validated at the integration level (S-036 scope); S-037 unit tests use virtual clock. |

## Full Suite Result

```
running 10 tests
test session_manager::tests::test_BC_2_08_005_rename_on_terminated_fails ... ok
test session_manager::tests::test_BC_2_08_005_gc_sidecar_enoent_no_error ... ok
test session_manager::tests::test_BC_2_08_005_terminated_session_gc_after_10s ... ok
test session_manager::tests::test_BC_2_08_005_duplicate_terminated_does_not_reset_gc ... ok
test session_manager::tests::test_BC_2_08_005_two_sessions_terminate_independently ... ok
test session_manager::tests::test_BC_2_08_005_session_list_carries_rename_after_rename_session ... ok
test session_manager::tests::test_BC_2_08_005_gc_wired_via_real_transition_to_terminated ... ok
test session_manager::tests::test_BC_2_08_005_rename_broadcast_carries_new_display_name ... ok
test session_manager::tests::test_BC_2_08_005_rename_on_running_succeeds ... ok
test session_manager::tests::test_BC_2_08_005_rename_while_launching_survives_running_sidecar_repersist ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 0.26s
```

**10/10 BC-2.08.005 tests passing. 0 failures.**
