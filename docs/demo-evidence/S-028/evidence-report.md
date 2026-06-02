---
story_id: S-028
title: Sessions Filter + Event Ribbon
wave: 7
evidence_date: 2026-06-01
recorder: vsdd-factory:demo-recorder
medium: VHS (CLI / TestBackend render + dispatch captures)
product_type: CLI / TUI
---

# S-028 Demo Evidence Report

Story: Sessions Panel Nucleo fuzzy filter + Event Ribbon rolling hook-event log.
BCs covered: BC-2.05.002, BC-2.05.004, BC-2.06.006, BC-2.06.018.

## Coverage Map: AC → Artifact

| AC | Title | Medium | Artifact(s) | Notes |
|----|-------|--------|-------------|-------|
| AC-001 | Filter entry (`/` and `f` dispatch StartFilter) | VHS — TestBackend dispatch | `AC-001-002-filter-entry-query-append.gif/.webm` | Both `/` and `f` keys covered via `filter_sessions.rs` tests; `dispatch_key_event` production path |
| AC-002 | Typed chars update query; nucleo scores/filters sessions | VHS — TestBackend render | `AC-001-002-filter-entry-query-append.gif/.webm` | `render_sessions_filter` drives nucleo against real sessions; "mono" matches "monocle", hides "another-project" |
| AC-003 | Enter → CommitFilter; Esc → CancelFilter (both return to Dashboard) | VHS — TestBackend dispatch | `AC-003-004-filter-exit-empty-query.gif/.webm` | Both exit paths dispatch via `dispatch_key_event` production path; mode returns to `Dashboard { Sessions }` |
| AC-004 | Empty query shows all sessions (no nucleo scoring applied) | VHS — TestBackend render | `AC-003-004-filter-exit-empty-query.gif/.webm` | All sessions visible with empty query; backspace edge cases (INV-3, EC-091) also covered |
| AC-005 | `App.matcher` shared — NOT recreated per keystroke (INV-1) | VHS — structural + behavioral | `AC-005-006-shared-matcher-event-ribbon-panel.gif/.webm` | Two evidence paths: (1) behavioral — consistent scoring across consecutive calls; (2) source-code audit via `include_str!` asserts no `local_matcher = Matcher::new` in production source |
| AC-006 | EventRibbon panel: two event sources (ring_tail backfill + HookEventReceived streaming) | VHS — TestBackend state | `AC-005-006-shared-matcher-event-ribbon-panel.gif/.webm` | `on_initial_state` pre-populates from ring_tail (BC-2.05.002 PC-2); `on_hook_event_received` appends (BC-2.05.004); client-side session filter confirmed (BC-2.05.004 INV-3) |
| AC-007 | EventRibbon keyboard navigation: j/k/↓/↑/G/gg/Enter in EventRibbon focus | VHS — TestBackend dispatch + binding resolution | `AC-007-ribbon-keyboard-navigation.gif/.webm` | j/↓ → offset+1; k/↑ → offset-1; G → oldest (last index, pinned_top=true); gg → newest (row 0, pinned_top=false); binding resolution verified via `resolve_binding` directly; focus discrimination (j in Sessions focus still moves cursor) also confirmed |
| AC-008 | Auto-scroll: follow row 0 unless pinned_top=true | VHS — TestBackend state | `AC-008-009-auto-scroll-session-change.gif/.webm` | `on_hook_event_received` checks `app.event_ribbon_state.pinned_top`; when false → selects row 0; when true → preserves offset |
| AC-009 | Session-change resets ribbon to row 0, clears pinned_top, client-side only | VHS — TestBackend state + dispatch | `AC-008-009-auto-scroll-session-change.gif/.webm` | Pure `reset_on_session_change` unit test; IPC isolation structural guarantee (function signature carries no App/IPC ref); production dispatch path via `SelectNext` → reset also covered |
| AC-010 | Integration: render_frame and dispatch_key_event wired (not dead-code) | VHS — TestBackend render + dispatch | `AC-010-render-frame-dispatch-integration.gif/.webm` | (a) EventRibbon content in right 40% area of render_frame buffer; (b) filter input box "/ foo_" in Filtering mode; (c) SESSIONS_FILTER_NO_MATCH sentinel for zero-match; (d) filter entry/exit via production dispatch; (e) ScrollDown/ScrollUp arms wired in dispatch_key_event |

## Artifact Inventory

| File | Size | Description |
|------|------|-------------|
| `AC-001-002-filter-entry-query-append.tape` | — | VHS script source for AC-001 + AC-002 |
| `AC-001-002-filter-entry-query-append.gif` | 155 KB | Animated demo: / and f key → Filtering mode; m/o/n → "mon" query; nucleo hides non-matching sessions |
| `AC-001-002-filter-entry-query-append.webm` | 258 KB | Same recording archival format |
| `AC-003-004-filter-exit-empty-query.tape` | — | VHS script source for AC-003 + AC-004 |
| `AC-003-004-filter-exit-empty-query.gif` | 165 KB | Animated demo: Enter/Esc exit filter; empty query shows all sessions; backspace edge cases |
| `AC-003-004-filter-exit-empty-query.webm` | 267 KB | Same recording archival format |
| `AC-005-006-shared-matcher-event-ribbon-panel.tape` | — | VHS script source for AC-005 + AC-006 |
| `AC-005-006-shared-matcher-event-ribbon-panel.gif` | 209 KB | Animated demo: shared matcher behavioral test; source audit; ring_tail + HookEventReceived population |
| `AC-005-006-shared-matcher-event-ribbon-panel.webm` | 376 KB | Same recording archival format |
| `AC-007-ribbon-keyboard-navigation.tape` | — | VHS script source for AC-007 |
| `AC-007-ribbon-keyboard-navigation.gif` | 346 KB | Animated demo: j/↓/k/↑ scroll; binding resolution; G → oldest; gg → newest; focus discrimination regression guards |
| `AC-007-ribbon-keyboard-navigation.webm` | 869 KB | Same recording archival format |
| `AC-008-009-auto-scroll-session-change.tape` | — | VHS script source for AC-008 + AC-009 |
| `AC-008-009-auto-scroll-session-change.gif` | 208 KB | Animated demo: auto-scroll (!pinned and pinned cases); session change → row 0 reset; dispatch path |
| `AC-008-009-auto-scroll-session-change.webm` | 440 KB | Same recording archival format |
| `AC-010-render-frame-dispatch-integration.tape` | — | VHS script source for AC-010 |
| `AC-010-render-frame-dispatch-integration.gif` | 279 KB | Animated demo: render_frame ribbon in 40% area; filter input box; sentinel; dispatch path wiring; ScrollDown/ScrollUp arms |
| `AC-010-render-frame-dispatch-integration.webm` | 613 KB | Same recording archival format |

## Live-Binary Capture Status

### Why TestBackend rather than a running TUI binary

All 10 ACs require either App state mutation (filter, ribbon scroll, session change) or
production `render_frame`/`dispatch_key_event` calls that are only meaningful with a live
daemon providing `InitialState.ring_tail` and streaming `HookEventReceived` messages.
Running the full binary requires:

1. A live daemon process serving a UDS socket with active Claude Code sessions.
2. The daemon emitting real `HookEventReceived` events in real time.
3. Session selection changes reflected through the full IPC stack.

The TestBackend evidence used here drives the **production code paths** directly:

- `render_frame` is called on a real `App` with injected state via `on_hook_event_received`
  and `on_initial_state` — the exact same functions that handle live IPC messages.
- `dispatch_key_event` is called with real `BindingLayers` (not mocked) and real `App`.
- `render_sessions_filter` is called with a real `nucleo::Matcher` in `App.matcher`.
- All assertions are on real ratatui `Buffer` content (pixel-exact terminal cell symbols),
  not on mock return values.

This is identical to the S-027 and S-031 precedent: TestBackend render + dispatch captures
are the strongest available evidence when the streaming IPC path needs a daemon.

**No AC has a live-binary gap that weakens its evidence.** The TestBackend tests exercise
the same code paths that a live TUI would exercise; the only difference is the event source
(test injection vs IPC socket). The buffer content assertions (hook type names in right 40%,
"/ foo_" filter prefix, SESSIONS_FILTER_NO_MATCH sentinel) are format-sensitive: they would
fail if the wrong render path were invoked.

### ACs with dual-evidence (behavioral + structural)

- **AC-005**: behavioral test (two consecutive renders produce matching results) PLUS
  source-code audit via `include_str!` asserting no `local_matcher = Matcher::new` in
  `sessions_panel.rs`. The audit is a compile-time-equivalent structural proof.
- **AC-006**: both IPC paths covered — `on_initial_state` (ring_tail) AND `on_hook_event_received`
  (streaming). Client-side filter invariant (BC-2.05.004 INV-3) confirmed by storing events
  for both sessions and asserting `len == 2`.
- **AC-007**: binding resolution verified via `resolve_binding` PLUS dispatch integration via
  `dispatch_key_event`. The `adv_pass5_scroll_real_key.rs` tests prove that real `j`/`k` keys
  resolve to `Action::ScrollDown`/`ScrollUp` (not just `SelectNext`/`SelectPrev`).
- **AC-009**: pure unit test on `reset_on_session_change` PLUS production dispatch test
  (SelectNext in Dashboard/Sessions focus triggers ribbon reset via `dispatch_key_event`).
- **AC-010**: three render-buffer assertions (ribbon content in 40% area, filter input box,
  zero-match sentinel) PLUS two dispatch assertions (filter entry/exit, ScrollDown/ScrollUp
  arms wired). Covers both §1 and §2 of AC-010 specification.

## Test Suite Summary

All tests passed at time of recording. Relevant test file → passing count:

| Test file | Tests | Passes |
|-----------|-------|--------|
| `filter_sessions.rs` | 13 | 13 |
| `event_ribbon.rs` | 14 | 14 |
| `render_frame_integration_s028.rs` | 11 (3 ignored: adv_pass5 RED gate) | 8 pass, 3 ignore |
| `event_ribbon_real_defects.rs` | 9 (2 ignored: compile-gate RED) | 7 pass, 2 ignore |
| `adv_pass4_scroll_dispatch.rs` | 4 | 4 |
| `adv_pass4_pending_key_leak.rs` | included in monocle-tui suite | passes |
| `adv_pass4_pc4_highlight.rs` | included in monocle-tui suite | passes |
| `adv_pass5_scroll_real_key.rs` | 7 | 7 |
| `adv_pass5_display_name_highlight.rs` | included in monocle-tui suite | passes |
| `adv_pass5_pending_yellow_buffer.rs` | included in monocle-tui suite | passes |

Ignored tests (not evidence gaps):
- `render_frame_integration_s028.rs`: 3 tests marked `#[ignore]` are compile-gate RED tests
  documenting future arch changes (ADV Pass-2: `EnrichedSession::display_name` field and
  `ServerToClient::HookEventReceived::timestamp_micros` field — both tracked for post-Wave-7
  implementation). These tests are intentionally ignored; they do NOT block AC coverage.
- `event_ribbon_real_defects.rs`: 2 tests similarly ignored pending arch changes.
