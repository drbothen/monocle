---
story_id: S-042
title: PTY Resize Detection, 50ms Debounce, and ResizePane IPC
version: "1.0"
produced_by: vsdd-factory:demo-recorder
date: 2026-06-21
---

# S-042 Demo Evidence

## What Is Demonstrated

S-042 implements the full end-to-end PTY resize pipeline in monocle:

1. **TUI side** (`monocle-tui`): per-render-cycle pane-area change detection,
   immediate local `vt100::Parser` resize (not debounced), 50ms debounce timer
   for `ClientToServer::ResizePane` IPC send, `last_sent_size` dedup guard,
   `AppMode::EmbeddedTerminal`-only enforcement, edge cases (zero dims, same size,
   rapid coalescing, Dashboard mode no-op, scroll-offset reset on resize).

2. **Daemon side** (`monocle-runtime`): `ClientToServer::ResizePane` IPC dispatch arm,
   zero-dimension defense-in-depth clamp (`rows.max(1)`, `cols.max(1)`),
   `SessionManager::resize_session()` implementation (replaces prior `todo!()` stub),
   `DaemonToHost::Resize` forwarding to session-host, WARN-drop error paths
   (SessionNotFound, SessionNotReady, SessionHostDead, IO) per BC-2.09.006 ResizePane carve-out.

3. **Session-host side** (`monocle-session-host`): `DaemonToHost::Resize` match arm
   calling `pty.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })` and
   `parser.set_size(rows, cols)`, triggering SIGWINCH to the harness child.

## Artifacts

| File | Type | What It Shows | ACs Covered |
|------|------|---------------|-------------|
| `AC-001-resize-debounce-unit-tests.tape` | VHS tape | Source script for AC-001 recording | — |
| `AC-001-resize-debounce-unit-tests.webm` | Recording | 15 TUI debounce unit tests passing | AC-001..AC-012 |
| `AC-002-run-loop-wiring-tests.tape` | VHS tape | Source script for AC-002 recording | — |
| `AC-002-run-loop-wiring-tests.webm` | Recording | 6 run-loop anti-dead-code wiring tests passing | AC-001, AC-007 |
| `AC-003-daemon-resize-pipeline-tests.tape` | VHS tape | Source script for AC-003 recording | — |
| `AC-003-daemon-resize-pipeline-tests.webm` | Recording | 9 daemon pipeline + error-path tests passing | AC-013..AC-016 |
| `README.md` | This file | Artifact index + regeneration instructions | — |
| `evidence-report.md` | Report | Full AC-to-recording coverage map | All 16 ACs |

## Recording Details

### AC-001 — TUI Resize Debounce Unit Tests

**File:** `AC-001-resize-debounce-unit-tests.webm`

Runs `cargo test -p monocle-tui --test bc_2_09_006_resize_debounce --test bc_2_09_006_poll_timeout_seam`.

15 tests covering:
- `test_BC_2_09_006_size_change_detected_per_render_cycle` (AC-001)
- `test_BC_2_09_006_resize_sends_resizepane_after_50ms` (AC-002, AC-005)
- `test_BC_2_09_006_local_parser_resized_immediately` (AC-003, AC-008)
- `test_BC_2_09_006_no_ipc_before_debounce_expires` (AC-002, AC-005)
- `test_BC_2_09_006_local_parser_not_debounced` (AC-003, AC-008)
- `test_BC_2_09_006_rapid_resize_coalesced` (AC-009, EC-235)
- `test_BC_2_09_006_dashboard_mode_no_resizepane` (AC-010, EC-236)
- `test_BC_2_09_006_resize_to_same_size_no_op` (AC-011, EC-237)
- `test_BC_2_09_006_zero_dimensions_no_op` (AC-012, EC-239)
- `test_BC_2_09_006_scroll_offset_reset_on_resize` (AC-003 + BC-2.09.001 Invariant 6)
- `test_BC_2_09_006_mid_window_resize_does_not_reset_deadline` (AC-005 invariant)
- `test_BC_2_09_006_clear_debounce_state_on_exit` (AC-006 + AppMode exit)
- `test_BC_2_09_006_canonical_vector_24x80_to_30x100` (AC-004 latency vector)
- `test_BC_2_09_006_MED001_poll_timeout_never_exceeds_tick_rate` (AC-004, poll seam)
- `test_BC_2_09_006_MED001_poll_timeout_shrinks_to_deadline` (AC-004, poll seam)

### AC-002 — Run-Loop Wiring Anti-Dead-Code Tests

**File:** `AC-002-run-loop-wiring-tests.webm`

Runs `cargo test -p monocle-tui --test bc_2_09_006_run_loop_wiring`.

6 tests covering:
- `test_BC_2_09_006_run_loop_tick_fires_resizepane_without_check_call` (AC-001 wiring guard — resize fires from loop, not manual call)
- `test_BC_2_09_006_run_loop_tick_fires_resizepane_after_resize_event` (AC-002 wiring guard)
- `test_BC_2_09_006_run_loop_resize_in_dashboard_is_noop` (AC-007/AC-010 from loop level)
- `test_BC_2_09_006_exit_embedded_terminal_clears_debounce_state` (debounce cleared on mode exit)
- `test_BC_2_09_006_overlay_transition_clears_resize_state` (debounce cleared on overlay)
- `test_BC_2_09_006_per_render_layout_change_triggers_detection` (AC-001 per render cycle)

### AC-003 — Daemon Resize Pipeline Tests

**File:** `AC-003-daemon-resize-pipeline-tests.webm`

Runs `cargo test -p monocle-runtime --test bc_2_09_006_daemon_resize`.

9 tests covering:
- `test_BC_2_09_006_daemon_resizepane_routes_to_resize_session` (AC-013 dispatch arm)
- `test_BC_2_09_006_handler_zero_dim_rows_clamp_no_error` (AC-014 zero-dim clamp)
- `test_BC_2_09_006_handler_zero_dim_cols_clamp_no_error` (AC-014 zero-dim clamp)
- `test_BC_2_09_006_handler_session_not_found_warn_drop` (AC-013 WARN-drop)
- `test_BC_2_09_006_handler_session_manager_none_warn_drop` (AC-013 WARN-drop)
- `test_BC_2_09_006_handler_session_host_dead_warn_drop` (AC-016 EC-238)
- `test_BC_2_09_006_resize_session_forwards_daemon_to_host_resize` (AC-015 DaemonToHost::Resize)
- `test_BC_2_09_006_daemon_session_not_found_returns_err` (AC-013/AC-015 error path)
- `test_BC_2_09_006_ec238_session_host_dead_warn_drop` (AC-016 EC-238)

## Harness Limitations

A full live end-to-end demo (resize terminal window → observe harness child reflow in the TUI)
requires a running daemon, session-host, and Claude Code session. This pipeline is not
driveable in a VHS `.tape` script because:

1. The `monocle-session-host` binary requires a running `monocle` daemon (IPC socket).
2. The daemon requires a bound Unix domain socket in a runtime directory.
3. Claude Code requires a valid API key and network access.

The demos above are the correct and honest evidence boundary: **30 unit and integration
tests passing across 4 test files** verify every acceptance criterion (AC-001..AC-016) through
direct behavioral assertions on the implementation. Each test exercises the exact code
path described in the AC, not a stub or mock at the wrong layer.

The session-host-side handler (`DaemonToHost::Resize` → `pty.resize()` + `parser.set_size()`)
is implemented in `crates/monocle-session-host/src/main.rs` and exercised indirectly by
the daemon-layer tests via a mock session-host connection.

## Format Notes

- Output format: WEBM only (no GIF — project demo policy for Wave 9).
- Font: `FiraCode Nerd Font Mono` (detected on this machine via `fc-list`).
- VHS version: 0.11.0.
- `Wait+Line` not used: VHS 0.11.0 on this machine polls the current terminal screen line
  after fast commands return (pre-compiled binaries run in <100ms), missing the output.
  `Sleep 10s` is used instead to hold the terminal output visible, per established project
  fallback for this VHS version.

## How to Regenerate

From the worktree root:

```bash
cd docs/demo-evidence/S-042
vhs AC-001-resize-debounce-unit-tests.tape
vhs AC-002-run-loop-wiring-tests.tape
vhs AC-003-daemon-resize-pipeline-tests.tape
```

Requires: `cargo` on PATH, VHS 0.11.0+, `FiraCode Nerd Font Mono` installed.
