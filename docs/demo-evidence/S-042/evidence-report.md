---
story_id: S-042
title: PTY Resize Detection, 50ms Debounce, and ResizePane IPC
version: "1.0"
produced_by: vsdd-factory:demo-recorder
date: 2026-06-21
---

# S-042 Demo Evidence Report

## Story Summary

S-042 implements the full end-to-end PTY resize pipeline for monocle's `EmbeddedTerminal` mode:
pane-area change detection per render cycle, immediate local `vt100::Parser` resize (not debounced),
50ms debounce for `ClientToServer::ResizePane` IPC, daemon dispatch arm with zero-dim clamp,
`SessionManager::resize_session()` implementation (replacing the prior `todo!()` stub),
`DaemonToHost::Resize` forwarding to session-host, and session-host `pty.resize()` + `parser.set_size()`.

All 16 acceptance criteria across the full end-to-end pipeline are covered by 30 passing tests.

## Coverage Map

| Recording | Acceptance Criteria | BCs Covered | Tests |
|-----------|--------------------|-----------:|------:|
| AC-001-resize-debounce-unit-tests.webm | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-008, AC-009, AC-010, AC-011, AC-012 | BC-2.09.006 | 15 |
| AC-002-run-loop-wiring-tests.webm | AC-001, AC-002, AC-007, AC-010 (run-loop level) | BC-2.09.006 | 6 |
| AC-003-daemon-resize-pipeline-tests.webm | AC-013, AC-014, AC-015, AC-016 | BC-2.09.006 | 9 |

**Total: 30 tests passing, 0 failures** (4 test files across 2 crates).

## Recording Details

### AC-001 — TUI Resize Debounce Unit Tests

**File:** `AC-001-resize-debounce-unit-tests.webm`
**Tape:** `AC-001-resize-debounce-unit-tests.tape`

Runs `cargo test -p monocle-tui --test bc_2_09_006_resize_debounce --test bc_2_09_006_poll_timeout_seam`:

**bc_2_09_006_resize_debounce.rs (13 tests):**
- `test_BC_2_09_006_size_change_detected_per_render_cycle` — AC-001: pane area vs parser size check
- `test_BC_2_09_006_resize_sends_resizepane_after_50ms` — AC-002, AC-005: `ResizePane` sent after 50ms debounce
- `test_BC_2_09_006_local_parser_resized_immediately` — AC-003, AC-008: `set_size()` called synchronously without debounce
- `test_BC_2_09_006_no_ipc_before_debounce_expires` — AC-005: no `ResizePane` before 50ms
- `test_BC_2_09_006_local_parser_not_debounced` — AC-003: parser size matches new area before debounce fires
- `test_BC_2_09_006_rapid_resize_coalesced` — AC-009: three intermediate sizes → single `ResizePane` for final size
- `test_BC_2_09_006_dashboard_mode_no_resizepane` — AC-010: `AppMode::Dashboard` → no IPC
- `test_BC_2_09_006_resize_to_same_size_no_op` — AC-011: identical dimensions → no IPC, `last_sent_size` unchanged
- `test_BC_2_09_006_zero_dimensions_no_op` — AC-012: `area.rows==0` or `area.cols==0` → no IPC
- `test_BC_2_09_006_scroll_offset_reset_on_resize` — AC-003 + BC-2.09.001 Invariant 6: scroll offset reset on resize
- `test_BC_2_09_006_mid_window_resize_does_not_reset_deadline` — AC-005: debounce deadline not reset on mid-window change
- `test_BC_2_09_006_clear_debounce_state_on_exit` — AC-006: `last_sent_size`/`resize_debounce_deadline` cleared on AppMode exit
- `test_BC_2_09_006_canonical_vector_24x80_to_30x100` — AC-004: canonical resize vector, `ResizePane { rows:30, cols:100 }`

**bc_2_09_006_poll_timeout_seam.rs (2 tests):**
- `test_BC_2_09_006_MED001_poll_timeout_never_exceeds_tick_rate` — AC-004: `crossterm::event::poll` timeout never > tick rate
- `test_BC_2_09_006_MED001_poll_timeout_shrinks_to_deadline` — AC-004: poll timeout shrinks to debounce deadline when closer than tick rate

### AC-002 — Run-Loop Wiring Anti-Dead-Code Tests

**File:** `AC-002-run-loop-wiring-tests.webm`
**Tape:** `AC-002-run-loop-wiring-tests.tape`

Runs `cargo test -p monocle-tui --test bc_2_09_006_run_loop_wiring`:

**bc_2_09_006_run_loop_wiring.rs (6 tests):**
- `test_BC_2_09_006_run_loop_tick_fires_resizepane_without_check_call` — AC-001: anti-dead-code guard; `ResizePane` fires from the loop without explicit `check_resize_debounce()` call from outside
- `test_BC_2_09_006_run_loop_tick_fires_resizepane_after_resize_event` — AC-002: `ResizePane` fires after pane area change + 50ms in the loop
- `test_BC_2_09_006_run_loop_resize_in_dashboard_is_noop` — AC-007, AC-010: Dashboard mode produces no `ResizePane` in the loop
- `test_BC_2_09_006_exit_embedded_terminal_clears_debounce_state` — AC-006: loop-level debounce state cleared on EmbeddedTerminal exit
- `test_BC_2_09_006_overlay_transition_clears_resize_state` — AC-006: overlay transition clears debounce state
- `test_BC_2_09_006_per_render_layout_change_triggers_detection` — AC-001: area change in layout triggers detection per render cycle

### AC-003 — Daemon Resize Pipeline Tests

**File:** `AC-003-daemon-resize-pipeline-tests.webm`
**Tape:** `AC-003-daemon-resize-pipeline-tests.tape`

Runs `cargo test -p monocle-runtime --test bc_2_09_006_daemon_resize`:

**bc_2_09_006_daemon_resize.rs (9 tests):**
- `test_BC_2_09_006_daemon_resizepane_routes_to_resize_session` — AC-013: `ClientToServer::ResizePane` arm calls `resize_session()`
- `test_BC_2_09_006_handler_zero_dim_rows_clamp_no_error` — AC-014: `rows: 0` clamped to 1; WARN emitted; no `ServerToClient::Error`
- `test_BC_2_09_006_handler_zero_dim_cols_clamp_no_error` — AC-014: `cols: 0` clamped to 1; WARN emitted; no `ServerToClient::Error`
- `test_BC_2_09_006_handler_session_not_found_warn_drop` — AC-013: session not found → WARN-drop; no error response
- `test_BC_2_09_006_handler_session_manager_none_warn_drop` — AC-013: session manager unavailable → WARN-drop
- `test_BC_2_09_006_handler_session_host_dead_warn_drop` — AC-016: `SessionHostDead` → WARN-drop; no `ServerToClient::Error`
- `test_BC_2_09_006_resize_session_forwards_daemon_to_host_resize` — AC-015: `resize_session()` writes `DaemonToHost::Resize { rows, cols }` to session-host
- `test_BC_2_09_006_daemon_session_not_found_returns_err` — AC-013/AC-015: `resize_session()` returns `Err(SessionNotFound)` for unknown session
- `test_BC_2_09_006_ec238_session_host_dead_warn_drop` — AC-016 (EC-238): write failure to session-host → WARN-drop; session transitions to Terminated via watchdog

## What Is Not Demonstrated

AC-016's final assertion — that the session transitions to `Terminated` via the standard watchdog path
after `resize_session()` returns `SessionHostDead` — is verified at the WARN-drop level in the tests
above. The full watchdog state transition (which is tested in prior stories' test suites via S-033/S-036)
is out of scope for S-042's test responsibility.

A full live end-to-end demo (terminal resize → harness child reflow visible in the TUI) is not feasible
in a VHS `.tape` because it requires a running daemon, session-host, and Claude Code session.
See `README.md §Harness Limitations` for details.

## Format Notes

- Output format: WEBM only (no GIF — project demo policy for Wave 9).
- Font: `FiraCode Nerd Font Mono` (detected via `fc-list`).
- VHS version: 0.11.0.
- `Wait+Line` not used: this VHS version's current-line scanner misses fast pre-compiled test binaries
  (< 100ms run time). `Sleep 10s` is used per the established project fallback for this VHS version.
