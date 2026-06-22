---
story_id: S-041
title: Mouse Forwarding (BC-2.09.003)
version: "1.0"
produced_by: vsdd-factory:demo-recorder
date: 2026-06-22
---

# S-041 Demo Evidence Report

## Story Summary

S-041 implements the full mouse-forwarding pipeline for monocle's `EmbeddedTerminal` mode:
pure-core SGR encoding (`mouse_event_to_pty_bytes`) covering the full Ps table (button
down/up/drag/scroll/moved), modifier bits (Ctrl/Shift/Alt/combined), 1-indexed pane-relative
coordinate transform, out-of-pane→None, and `M` vs `m` terminator selection;
crossterm↔PtyMouseEvent conversion boundary (`crossterm_mouse_to_pty`, `ratatui_rect_to_pty`);
run-loop dispatch arm (`dispatch_embedded_terminal_mouse` → `KeyInput` IPC); and scoped
mouse-capture lifecycle (`scoped_mouse_capture_enter` / `scoped_mouse_capture_exit` with
`mouse_capture_active` idempotency guard, including teardown on permission-prompt→Overlay,
transport disconnect, and normal Esc exit — F-S041-P7/P11 regression guards).

All 20 acceptance criteria across both test suites are covered by 47 passing tests.

## S-044 Dependency

A full live mouse-click demo (click in EmbeddedTerminal → Claude Code responds) requires
the `EmbeddedTerminal` entry trigger, which is S-044 scope and not yet wired. The demos
below record the honest evidence boundary for S-041: the pure-core SGR encoding layer and
the TUI-side dispatch+lifecycle pipeline, both verified through the full test suites.

## Coverage Map

| Recording | Acceptance Criteria | BCs Covered | Tests |
|-----------|--------------------|-----------:|------:|
| AC-001-mouse-sgr-encoding-tests.webm | AC-003, AC-004, AC-005, AC-006, AC-007, AC-008, AC-009, AC-010 | BC-2.09.003 | 27 |
| AC-002-mouse-dispatch-lifecycle-tests.webm | AC-001, AC-002, AC-003, AC-007, AC-008, Invariant 1 | BC-2.09.003 | 20 |

**Total: 47 tests passing, 0 failures** (2 test files across 2 crates).

## Recording Details

### AC-001 — Mouse SGR Encoding Unit Tests

**File:** `AC-001-mouse-sgr-encoding-tests.webm`
**Tape:** `AC-001-mouse-sgr-encoding-tests.tape`

Runs `cargo test -p monocle-core --test bc_2_09_003_mouse_forwarding`:

**bc_2_09_003_mouse_forwarding.rs (27 tests):**
- `test_BC_2_09_003_mouse_events_sgr_encoded_left_press` — left button down → `\x1b[<0;C;Rm` (AC-003)
- `test_BC_2_09_003_mouse_events_sgr_encoded_left_release` — left button release → `\x1b[<0;C;Rm` terminator `m` (AC-003, AC-010)
- `test_BC_2_09_003_mouse_events_sgr_scroll_up` — scroll up → `\x1b[<64;C;RM` (AC-004)
- `test_BC_2_09_003_drag_encoding` — left drag → Ps=32 (AC-005)
- `test_BC_2_09_003_out_of_pane_returns_none` — position outside pane → `None` (AC-008)
- `test_BC_2_09_003_out_of_pane_column_boundary_returns_none` — column==pane.right → `None` (AC-008)
- `test_BC_2_09_003_out_of_pane_row_boundary_returns_none` — row==pane.bottom → `None` (AC-008)
- `test_BC_2_09_003_1_indexed_origin` — pane at (0,0): col 0 → `\x1b[<0;1;1M` (1-indexed) (AC-007)
- `test_BC_2_09_003_1_indexed_nonzero_pane_origin` — pane at (10,5): col 10 → `\x1b[<0;1;1M` (AC-007)
- `test_BC_2_09_003_pane_relative_offset_nonzero_pane` — column offset from pane origin (AC-007)
- `test_BC_2_09_003_modifier_bits_ctrl` — Ctrl → Ps|=16 (AC-006)
- `test_BC_2_09_003_modifier_bits_shift` — Shift → Ps|=4 (AC-006)
- `test_BC_2_09_003_modifier_bits_alt` — Alt → Ps|=8 (AC-006)
- `test_BC_2_09_003_modifier_bits_ctrl_shift_combined` — Ctrl+Shift → Ps|=20 (AC-006)
- `test_BC_2_09_003_scroll_down_encoding` — scroll down → Ps=65 (AC-004)
- `test_BC_2_09_003_middle_button_press` — middle button down → Ps=1 (AC-003)
- `test_BC_2_09_003_right_button_press` — right button down → Ps=2 (AC-003)
- `test_BC_2_09_003_middle_button_release` — middle release → terminator `m` (AC-010)
- `test_BC_2_09_003_right_button_release` — right release → terminator `m` (AC-010)
- `test_BC_2_09_003_drag_middle_encoding` — middle drag → Ps=33 (AC-005)
- `test_BC_2_09_003_drag_right_encoding` — right drag → Ps=34 (AC-005)
- `test_BC_2_09_003_terminator_m_for_release_only` — release events use `m`, all others `M` (AC-010)
- `test_BC_2_09_003_moved_encoding` — moved (no button) → Ps=35 (AC-009)
- `test_BC_2_09_003_scroll_left_encoding` — scroll left → Ps=66 (AC-004)
- `test_BC_2_09_003_scroll_right_encoding` — scroll right → Ps=67 (AC-004)
- `test_BC_2_09_003_out_of_pane_column_underflow_nonzero_pane` — underflow guard (AC-008)
- `test_BC_2_09_003_terminator_M_for_non_release_variants` — pressed/dragged/scroll/moved all use `M` (AC-010)

### AC-002 — Mouse Dispatch and Scoped-Capture Lifecycle Tests

**File:** `AC-002-mouse-dispatch-lifecycle-tests.webm`
**Tape:** `AC-002-mouse-dispatch-lifecycle-tests.tape`

Runs `cargo test -p monocle-tui --test bc_2_09_003_mouse_dispatch`:

**bc_2_09_003_mouse_dispatch.rs (20 tests):**

Conversion boundary (6 tests):
- `test_BC_2_09_003_crossterm_mouse_to_pty_left_down` — crossterm LeftButton Down → PtyMouseButton::Left + PtyMouseEventKind::Down (AC-007)
- `test_BC_2_09_003_crossterm_mouse_to_pty_right_release` — crossterm RightButton Up → PtyMouseEventKind::Up (AC-007)
- `test_BC_2_09_003_crossterm_mouse_to_pty_scroll_up` — crossterm ScrollUp → PtyMouseEventKind::ScrollUp (AC-007)
- `test_BC_2_09_003_crossterm_mouse_to_pty_drag` — crossterm drag → PtyMouseEventKind::Drag (AC-007)
- `test_BC_2_09_003_crossterm_mouse_to_pty_ctrl_modifier` — crossterm Ctrl modifier preserved in PtyKeyModifiers (AC-007)
- `test_BC_2_09_003_ratatui_rect_to_pty_fields_copied` — ratatui Rect fields copied to PtyRect (AC-007)

Dispatch wiring (8 tests):
- `test_BC_2_09_003_mouse_dispatch_forwards_keyinput` — Event::Mouse in EmbeddedTerminal → KeyInput IPC sent (AC-003, AC-005)
- `test_BC_2_09_003_mouse_dispatch_out_of_pane_no_ipc` — out-of-pane Event::Mouse → no IPC (AC-008 / EC-221)
- `test_BC_2_09_003_mouse_dispatch_scroll_up_forwarded` — scroll_up in EmbeddedTerminal → KeyInput with SGR bytes (AC-004)
- `test_BC_2_09_003_mouse_event_does_not_exit_embedded_terminal` — mouse event preserves EmbeddedTerminal mode (AC-002)
- `test_BC_2_09_003_mouse_event_in_dashboard_mode_no_ipc` — Event::Mouse in Dashboard → no IPC (AC-001)
- `test_BC_2_09_003_key_forwarding_unaffected_by_mouse_arm` — keyboard forwarding unchanged by presence of mouse arm (AC-003)
- `test_BC_2_09_003_mouse_capture_active_in_embedded_terminal` — `mouse_capture_active` true after `enter_embedded_terminal` (Invariant 1)
- `test_BC_2_09_003_scoped_mouse_capture_lifecycle_enter_transitions_mode` — `enter_embedded_terminal` transitions mode (AC-001)

Scoped-capture lifecycle / F-S041-P7+P11 regression guards (6 tests):
- `test_BC_2_09_003_scoped_mouse_capture_lifecycle_exit_restores_mode` — `exit_embedded_terminal` restores Dashboard mode (AC-002, Invariant 1)
- `test_BC_2_09_003_scoped_mouse_capture_lifecycle_full_roundtrip` — enter→exit round-trip: `mouse_capture_active` false after exit (Invariant 1)
- `test_BC_2_09_003_mouse_capture_torn_down_on_permission_prompt_overlay` — EmbeddedTerminal→Overlay (permission prompt): capture torn down (F-S041-P7-HIGH-001)
- `test_BC_2_09_003_mouse_capture_off_after_permission_resolve_to_dashboard` — permission resolve to Dashboard: capture stays off (F-S041-P7-HIGH-001)
- `test_BC_2_09_003_mouse_capture_torn_down_on_normal_exit` — normal Esc exit: capture torn down (Invariant 1)
- `test_BC_2_09_003_mouse_capture_torn_down_on_transport_disconnect` — transport disconnect while in EmbeddedTerminal: capture torn down (Invariant 1 / F-S041-P11-BLOCKER-001)

## What Is Not Demonstrated

A full live mouse-click-to-PTY round-trip ("click in EmbeddedTerminal → Claude Code responds")
requires the `EmbeddedTerminal` entry trigger, which is S-044 scope and not yet wired. The
`scoped_mouse_capture_enter` and `scoped_mouse_capture_exit` functions write directly to
`crossterm`'s `stdout()` (terminal device I/O), so the exact escape-sequence bytes
(`\x1b[?1000h`, `\x1b[?1006h`, etc.) cannot be captured in a unit-test assertion without
a mock terminal backend — the lifecycle tests assert mode transitions and the `mouse_capture_active`
flag, not the byte sequence. These recordings are the correct and honest evidence boundary
for S-041: the SGR encoding layer (27 tests) and the TUI-side dispatch+lifecycle pipeline
(20 tests) are fully implemented and verified.

## Format Notes

- Output format: WEBM only (no GIF — project demo policy for Wave 9).
- Font: `FiraCode Nerd Font Mono` (detected via `fc-list`).
- VHS version: 0.11.0.
- `Wait+Line` not used: this VHS version's current-line scanner misses fast pre-compiled
  test binaries. `Sleep 30s` is used per the established project fallback for this VHS version.
