# Demo Evidence Report — S-039: PTY Output Pipeline

**Story:** S-039 — TUI-side PTY output pipeline  
**Behavioral Contract:** BC-2.09.001  
**Evidence produced:** 2026-06-20  
**Demo mode:** Harness/test-driven (see S-040 gate note below)

---

## S-040 Gate Note

Interactive entry into `EmbeddedTerminal` mode (the `[e]` keybinding or `Action::EnterEmbedded`)
is implemented by **S-040** (keyboard forwarding / key dispatch). The PTY output pipeline itself
(IPC `PtyOutput` → `vt100::Parser` → `tui-term PseudoTerminal` render; auto-attach
buffering/replay; bounded buffer + dump-window timeout; reconnect cleanup; GC) is complete and
verified in S-039. This evidence demonstrates the full pipeline via the behavioral test suite.
No interactive TUI demo was possible at this story boundary because the entry action is not yet
wired.

---

## Coverage Map

| Recording | AC / BC | Description | Result |
|-----------|---------|-------------|--------|
| AC-001-pty-pipeline-test-suite.webm | BC-2.09.001 (all) | Full 35-test suite: render, auto-attach, bounded buffer cap, dump timeout, reconnect, GC | PASS |
| AC-002-render-path-and-error-paths.webm | BC-2.09.001 (selected) | Render path (success) + 4 error/edge paths | PASS |

---

## Recordings

### AC-001 — Full BC-2.09.001 Test Suite

**File:** `AC-001-pty-pipeline-test-suite.webm` (186 KB)  
**Tape:** `AC-001-pty-pipeline-test-suite.tape`  
**Command:**
```
cargo test -p monocle-tui --test bc_2_09_001_pty_output_pipeline 2>&1 | tail -12
```
**Demonstrates:** All 35 tests of the PTY output pipeline passing. Covers the complete
BC-2.09.001 behavioral contract surface: IPC `PtyOutput` ingestion, `vt100::Parser` state
management per session, `tui-term PseudoTerminal` render, auto-attach dump protocol,
bounded buffer + byte-cap enforcement, dump-window timeout force-resolve, reconnect/GC.

**Test result:** `ok. 35 passed; 0 failed; 0 ignored; 0 measured`

---

### AC-002 — Render Path (Success) + Error/Edge Paths

**File:** `AC-002-render-path-and-error-paths.webm` (1.1 MB)  
**Tape:** `AC-002-render-path-and-error-paths.tape`  
**Demonstrates five targeted behavioral contracts:**

| Path | Test | BC sub-clause |
|------|------|---------------|
| SUCCESS: render path | `test_BC_2_09_001_render_frame_embedded_terminal_uses_focused_parser` | Focused session's `vt100::Parser` drives `tui-term PseudoTerminal` render |
| ERROR: unknown session | `test_BC_2_09_001_unknown_session_id_drop` | `PtyOutput` for unknown session_id silently dropped (no panic) |
| ERROR: buffer overflow | `test_BC_2_09_001_pending_pty_bytes_cap_drops_oldest` | Pending buffer count-cap exceeded: oldest bytes evicted |
| ERROR: dump timeout | `test_BC_2_09_001_dump_window_timeout_force_resolves` | Dump-window deadline exceeded: auto-attach force-resolved |
| RECONNECT: cleanup | `test_BC_2_09_001_disconnect_clears_dump_state_retains_parsers` | IPC disconnect clears dump-in-progress, retains `vt100` parsers |

**All 5 paths:** `ok. 1 passed` per run

---

## Full Test Coverage (35 tests in BC-2.09.001)

```
test_BC_2_09_001_invariant_bounded_channel_send_await_not_try_send
test_BC_2_09_001_invariant_scrollback_rows_default_and_clamp
test_BC_2_09_001_unknown_session_id_drop
test_BC_2_09_001_session_gc_removes_parser_and_scroll_offset
test_BC_2_09_001_render_embedded_terminal_calls_pseudo_terminal
test_BC_2_09_001_config_scrollback_rows_wiring
test_BC_2_09_001_on_initial_state_creates_parsers_no_clobber
test_BC_2_09_001_session_list_update_creates_and_gcs_parsers
test_BC_2_09_001_session_terminated_gc
test_BC_2_09_001_render_frame_embedded_terminal_uses_focused_parser
test_BC_2_09_001_scrollback_dump_complete_idempotency_guard
test_BC_2_09_001_roster_diff_gc_exits_embedded_mode_when_focused
test_BC_2_09_001_on_initial_state_gcs_stale_sessions_on_reconnect
test_BC_2_09_001_dump_complete_removes_dump_in_progress_entry
test_BC_2_09_001_terminated_session_exits_embedded_mode_before_gc
test_BC_2_09_001_pending_pty_bytes_cap_drops_oldest
test_BC_2_09_001_dump_window_timeout_force_resolves
test_BC_2_09_001_disconnect_clears_dump_state_retains_parsers
test_BC_2_09_001_pending_pty_bytes_byte_cap_drops_oldest
test_BC_2_09_001_status_bar_shows_dump_drops_when_focused
test_BC_2_09_001_enter_embedded_rollback_when_ipc_offline
test_BC_2_09_001_dump_window_timeout_end_to_end
test_BC_2_09_001_high_frequency_frame_merge
test_BC_2_09_001_non_focused_parser_updated
test_BC_2_09_001_reattach_after_detach_reruns_dump_protocol
test_BC_2_09_001_reentry_aborts_prior_timeout_handle
test_BC_2_09_001_pty_output_renders_within_100ms
test_BC_2_09_001_scrollback_replay_order
test_BC_2_09_001_second_enter_skips_attach_when_dump_already_received
test_BC_2_09_001_setup_ipc_streams_capacity_matches_production_channel
test_BC_2_09_001_inbound_channel_backpressure_no_drop
```
(35 total; 4 not listed above are additional coverage variants)
