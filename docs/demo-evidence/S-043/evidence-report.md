---
story_id: S-043
title: Scrollback Navigation — PtyScrollUp/Down, Per-Session Offsets, Configurable Capacity
version: "1.0"
produced_by: vsdd-factory:demo-recorder
date: 2026-06-22
---

# S-043 Demo Evidence Report

## Story Summary

S-043 implements scrollback navigation for monocle's `EmbeddedTerminal` mode (BC-2.09.007):
per-session scroll offsets (`HashMap<String, usize>` keyed by session_id), configurable scrollback
capacity (default 1000, clamped 1–10000), `PtyScrollUp`/`PtyScrollDown` action handlers with clamp,
`[scrolled back N rows]` status bar indicator (concurrent with all diagnostic badges), content-anchored
offset preservation when new PTY output arrives (vt100-native three-step algorithm), Ctrl+Up/Down
dispatch path with Kitty `KeyEventKind::Release` guard, and scroll offset reset on resize.

All 23 acceptance criteria across BC-2.09.007 postconditions, invariants, and edge cases are
covered by 23 passing tests including ADV Pass-4 rewrites (independent oracle for EC-244/EC-246
cap-saturation regime) and ADV Pass-3 Kitty Release guard (F-S043-P3-BLOCKER-001).

## Coverage Map

| Recording | Acceptance Criteria | BCs Covered | Tests |
|-----------|--------------------|-----------:|------:|
| AC-001-scrollback-config-and-navigation.webm | AC-001..AC-013 + EC-240..EC-243 + Invariant 3a/3b/5 | BC-2.09.007 | 23 (full suite) |
| AC-002-content-anchoring-badges-dispatch.webm | AC-002, AC-006, AC-008, AC-014, EC-244..EC-246, PC-3/4/5, F-S043-P2/P3 | BC-2.09.007 | 9 (focused) |

**Total: 23 tests passing, 0 failures** (1 test file: `bc_2_09_007_scrollback_navigation.rs`).

## Recording Details

### AC-001 — Full Test Suite: Scrollback Config and Navigation

**File:** `AC-001-scrollback-config-and-navigation.webm`
**Tape:** `AC-001-scrollback-config-and-navigation.tape`

Runs `cargo test -p monocle-tui --test bc_2_09_007_scrollback_navigation` (full 23-test suite):

**Config and scrollback capacity (BC-2.09.007 Postcondition 1 / Invariant 1/2):**
- `test_BC_2_09_007_scrollback_rows_default_1000` — AC-001: absent `pty_scrollback_rows` → `App::scrollback_rows == 1000`; `default_scrollback_rows()` pure helper; parser accepts scrollback
- `test_BC_2_09_007_scrollback_rows_capped_10000` — AC-001 / EC-242: `Some(15000)` → clamped to 10000
- `test_BC_2_09_007_scrollback_rows_clamped_min_1` — AC-001 / EC-243: `Some(0)` → clamped to 1 (key is present; not defaulted to 1000)

**PtyScrollUp/Down navigation (BC-2.09.007 Postcondition 2a/2b/2c/2d):**
- `test_BC_2_09_007_scrollup_increments_offset` — AC-002: `PtyScrollUp × 10` → `pty_scroll_offsets["s1"] = 10`; s2 unaffected
- `test_BC_2_09_007_scrolldown_decrements_floor_0` — AC-003 / EC-241: `PtyScrollDown` at 0 is no-op; repeated no-op; no panic
- `test_BC_2_09_007_clamp_at_max` — AC-004 / AC-012 / EC-240: scroll past max clamped; one more PtyScrollUp from max unchanged
- `test_BC_2_09_007_focus_switch_preserves_offsets` — AC-005 / AC-011 / Invariant 5: s1 offset=10 preserved after switch to s2; pty_scroll_offsets is HashMap, not shared field

**IPC and status bar (BC-2.09.007 Postcondition 3/4):**
- `test_BC_2_09_007_no_ipc_for_scroll` — AC-006 / PC-3: `PtyScrollUp × 5 + PtyScrollDown × 3`; IPC channel empty; scrollback is TUI-local
- `test_BC_2_09_007_status_bar_indicator_when_scrolled` — AC-007: `render_embedded_terminal` returns effective offset > 0 when scrolled; 0 at live tail
- `test_BC_2_09_007_status_bar_string_rendered` — AC-007 / HIGH-003: render_frame + TestBackend buffer literally contains `[scrolled back 5 rows]`; absent at live tail
- `test_BC_2_09_007_render_embedded_terminal_with_scroll_offset` — render path: set_scrollback applied; effective offset returned; clamped at max

**Lifecycle invariants:**
- `test_BC_2_09_007_resize_resets_scroll_offset_to_zero` — AC-009 / Invariant 3a: `on_resize_detected` resets offset to 0 (S-042 delivery)
- `test_BC_2_09_007_terminated_session_removes_scroll_entry` — AC-010 / Invariant 3b: `gc_pty_session` removes entry (not resets); parser also removed
- `test_BC_2_09_007_no_singular_shared_offset_field` — AC-011 / Invariant 5: three sessions with distinct offsets; modification of one does not affect others; HashMap confirmed

**Content-anchoring and concurrent badges (also in AC-002 focused demo):**
- `test_BC_2_09_007_content_anchored_new_output` — AC-008 / AC-014 / EC-244 / PC-5 (ADV P4 HIGH-001 independent oracle): offset=3 + 5 new rows → offset=8; numeric-preserve (3) WRONG; zero-reset (0) WRONG
- `test_BC_2_09_007_content_anchor_clamp_at_max` — AC-008 / EC-246 / PC-5 (ADV P4 BLOCKER-002): history saturated at 1000; offset=990 + 20 rows → offset=1000 (clamped at cap); delta-probe would stay at 990
- `test_BC_2_09_007_concurrent_status_bar_badges` — AC-007 / PC-4 / EC-245: scrolled back (offset=5) + dump-drop active; BOTH `[scrolled back 5 rows]` AND `[dump: 3 drops]` appear
- `test_BC_2_09_007_status_message_coexists_with_scrollback` — AC-007 / PC-4 / F-S043-P2-MED-001: scrollback indicator + `app.status_message` warn both appear; `.or()` suppression fixed

**Ctrl+Up/Down dispatch (also in AC-002 focused demo):**
- `test_BC_2_09_007_ctrl_up_dispatch_scrolls_no_ipc` — AC-002 / AC-006 / PC-3 / BLOCKER-001: Ctrl+Up Press → offset incremented by 1; no IPC
- `test_BC_2_09_007_ctrl_down_dispatch_scrolls_no_ipc` — AC-003 / AC-006 / PC-3 / BLOCKER-001: Ctrl+Down from offset=5 → offset=4; no IPC
- `test_BC_2_09_007_ctrl_up_release_does_not_scroll` — F-S043-P3-BLOCKER-001: Ctrl+Up Release → offset UNCHANGED; Kitty Release guard
- `test_BC_2_09_007_ctrl_up_press_then_release_scrolls_once` — F-S043-P3-BLOCKER-001: Press (→+1) + Release (→no-op) = exactly 1 net scroll
- `test_BC_2_09_007_ctrl_up_repeat_scrolls` — F-S043-P3-BLOCKER-001: Ctrl+Up Repeat → +1 (hold-to-scroll); Repeat treated as Press

### AC-002 — Focused: Content-Anchoring, Concurrent Badges, Dispatch

**File:** `AC-002-content-anchoring-badges-dispatch.webm`
**Tape:** `AC-002-content-anchoring-badges-dispatch.tape`

Runs five targeted cargo test invocations highlighting the most critical correctness tests:

1. `cargo test … content_anch` — 2 tests: EC-244 below-cap content-anchor (independent oracle: 3+5=8), EC-246 at-cap clamp (990+20→1000)
2. `cargo test … dispatch` — 2 tests: Ctrl+Up and Ctrl+Down dispatch path, no IPC
3. `cargo test … ctrl_up` — 4 tests: Press, Release (discarded), Press+Release (=1 net), Repeat (+1)
4. `cargo test … concurrent_status_bar` — 1 test: scrollback + dump-drop badge coexist
5. `cargo test … status_message_coexists` — 1 test: scrollback indicator + `app.status_message` both visible

## What Is Not Demonstrated

The full live embedded terminal scrolling experience (user presses Ctrl+Up in a live TUI → content
scrolls, `[scrolled back N rows]` appears in the status bar) is not feasible in a VHS `.tape`
because it requires a running daemon, session-host, and Claude Code session. All behavioral
contracts are verified through TDD unit tests using the `App` struct, `render_frame`, and
`TestBackend` headless rendering — the same approach used by S-039, S-040, and S-042.

## Format Notes

- Output format: WEBM only (no GIF — project demo policy for Wave 9).
- Font: `FiraCode Nerd Font Mono` (detected via `fc-list`).
- VHS version: 0.11.0.
- `Sleep 15s` used for AC-001 full suite (23 tests, ~0.02s runtime with precompiled binary).
- `Sleep 10s` used for AC-002 focused invocations (each runs ≤5 tests).
- `Wait+Line` not used: fast pre-compiled test binaries complete before VHS line-scanner triggers.
