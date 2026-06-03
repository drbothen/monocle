---
story_id: S-028
story_title: "Sessions Nucleo Fuzzy Filter + Event Ribbon + IPC timestamp_micros"
wave: 7
evidence_method: VHS (ratatui TestBackend test-suite capture)
produced_by: vsdd-factory:demo-recorder
date: 2026-06-03
base_ref: develop @ 6811103
---

# S-028 Demo Evidence Report

## Method

monocle is a TUI application (ratatui). A live daemon+prompt setup is impractical
for isolated evidence generation. Evidence is captured via VHS recordings that
drive the real compiled binary's test suites. The test suites exercise all
acceptance criteria through the production ratatui `TestBackend` render path and
the production `dispatch_key_event` / `render_frame` paths — the same paths
validated by AC-010's integration render requirement. This is the correct evidence
vehicle for ratatui TUI products; plain `cargo test` output is not used — the
recordings show the commands executed against the real codebase with pass/fail
results visible in the terminal.

All test suites pass: 13 passed (filter_sessions.rs), 14+8 passed
(event_ribbon.rs + event_ribbon_real_defects.rs), 12 passed
(render_frame_integration_s028.rs). No failures.

## Artifacts

| Artifact | Format | ACs Covered |
|----------|--------|-------------|
| `AC-001-005-sessions-filter.gif` | GIF (embed) | AC-001, AC-002, AC-003, AC-004, AC-005 |
| `AC-001-005-sessions-filter.webm` | WebM (archival) | AC-001, AC-002, AC-003, AC-004, AC-005 |
| `AC-001-005-sessions-filter.tape` | VHS source | — |
| `AC-006-010-event-ribbon-integration.gif` | GIF (embed) | AC-006, AC-007, AC-008, AC-009, AC-010 + F-W7G3-MED-001 fix |
| `AC-006-010-event-ribbon-integration.webm` | WebM (archival) | AC-006, AC-007, AC-008, AC-009, AC-010 + F-W7G3-MED-001 fix |
| `AC-006-010-event-ribbon-integration.tape` | VHS source | — |

## AC Coverage Mapping

### AC-001 (BC-2.06.006 PC-1 — filter entry: '/' and 'f' → Filtering mode)
**Evidence:** `AC-001-005-sessions-filter.gif`
**Tests (filter_sessions.rs):**
- `test_BC_2_06_006_filter_entry_slash_transitions_to_filtering` — '/' dispatches StartFilter → Filtering { Sessions, query:"", prior:Sessions }
- `test_BC_2_06_006_filter_entry_f_key_transitions_to_filtering` — 'f' binding enters Filtering identically
**Path:** success path (both '/' and 'f' bindings)
**Status:** PASS (13/13 filter_sessions tests)

### AC-002 (BC-2.06.006 PC-2 — typed chars update query; nucleo scores; non-matches hidden)
**Evidence:** `AC-001-005-sessions-filter.gif`
**Tests (filter_sessions.rs):**
- `test_BC_2_06_006_filter_query_appends_on_char` — 'm','o','n' → query="mon"
- `test_BC_2_06_006_nucleo_score_filters_non_matching` — query="mono": "monocle" visible, "another-project" hidden; SESSIONS_FILTER_NO_MATCH absent
- `test_BC_2_06_006_case_insensitive_fuzzy_match` — query="MONO" matches "monocle" (BC-2.06.006 PC-3 case-insensitive)
- `test_BC_2_06_006_filter_no_match_renders_no_sessions_match` — query="xyz": SESSIONS_FILTER_NO_MATCH rendered (zero-match sentinel)
- `test_BC_2_06_006_display_name_match` — query="cla", project_name="xyz-project" (no 'cla' chars): session visible only via display_name="Claude Code" OR branch (BC-2.06.006 PC-3 display_name match)
- `test_BC_2_06_006_backspace_removes_last_char` — Backspace removes last char from query
- `test_BC_2_06_006_ec091_backspace_on_empty_query_no_panic` — Backspace on empty query is no-op
**Path:** success path + error path (zero match sentinel) + edge cases
**Status:** PASS

### AC-003 (BC-2.06.006 PC-3 — CommitFilter/CancelFilter exit)
**Evidence:** `AC-001-005-sessions-filter.gif`
**Tests (filter_sessions.rs):**
- `test_BC_2_06_006_commit_filter_returns_to_dashboard` — Enter → CommitFilter → Dashboard { Sessions }
- `test_BC_2_06_006_cancel_filter_returns_to_dashboard` — Esc → CancelFilter → Dashboard { Sessions }
**Path:** success path (both exit bindings)
**Status:** PASS

### AC-004 (BC-2.06.006 PC-4 / empty query — all sessions shown, no scoring)
**Evidence:** `AC-001-005-sessions-filter.gif`
**Tests (filter_sessions.rs):**
- `test_BC_2_06_006_empty_query_shows_all_sessions` — query="": "monocle" AND "another-project" both visible
**Path:** success path
**Status:** PASS

### AC-005 (BC-2.06.006 INV-1 — shared nucleo::Matcher, not recreated per keystroke)
**Evidence:** `AC-001-005-sessions-filter.gif`
**Tests (filter_sessions.rs + event_ribbon_real_defects.rs):**
- `test_BC_2_06_006_invariant_matcher_not_recreated_per_keystroke` — two consecutive render_sessions_filter calls produce correct results from the shared matcher (behavioral consequence of INV-1)
- `test_BC_2_06_006_INV1_render_uses_shared_matcher_not_fresh_per_render` (event_ribbon_real_defects.rs) — structural source audit: "local_matcher = Matcher::new" absent from render_sessions_filter; the shared app.matcher is used
**Path:** invariant (both behavioral and structural proofs)
**Status:** PASS

### AC-006 (BC-2.06.018 PC-1 — EventRibbon panel: timestamp | hook_type | summary columns)
**Evidence:** `AC-006-010-event-ribbon-integration.gif`
**Tests (event_ribbon.rs):**
- `test_BC_2_06_018_newest_event_at_row_zero` — newest event prepended to front (PC-2 newest-first)
- `test_BC_2_06_018_rolling_window_bounded_to_panel_height` — 15 events, panel_height=10 → len=10 (PC-3 rolling window)
- `test_BC_2_06_018_pending_status_for_unresolved_pretooluse` — pending=true for unresolved PreToolUse (PC-4)
- `test_BC_2_06_018_pending_status_reverts_after_decision` — pending reverts to false after decision (PC-4)
- `test_BC_2_05_002_ring_tail_prepopulates_event_ribbon` — InitialState ring_tail → 3 rows in event_ribbon_events (BC-2.05.002 PC-2)
- `test_BC_2_05_004_hook_event_received_appends_to_ribbon` — HookEventReceived appends 1 row; correct session_id and latency_ms
- `test_BC_2_06_018_client_side_session_filter` — both sessions' events in event_ribbon_events; no IPC-layer filtering (BC-2.05.004 INV-3)
**Tests (event_ribbon_real_defects.rs):**
- `test_BC_2_06_018_PC1_wall_clock_timestamp_from_timestamp_micros` — timestamp column shows UTC wall-clock HH:MM:SS.mmm from timestamp_micros (not elapsed-delta)
- `test_BC_2_05_004_PC2_streaming_event_uses_daemon_timestamp_micros` — streaming HookEventReceived propagates daemon's timestamp_micros (BC-2.05.004 PC-2)
- `test_BC_2_06_018_PC3_panel_height_cap_enforced_at_push_time` — cap enforced at push via app.event_ribbon_panel_height
- `test_BC_2_06_018_PC4_pending_status_set_via_on_permission_prompt_queued` — on_permission_prompt_queued sets pending=true on matching ribbon row
- `test_BC_2_06_018_INV3_trim_on_resize_called_from_render_frame` — render_frame calls trim_to_panel_height after render
**Tests (render_frame_integration_s028.rs):**
- `test_BC_2_06_018_AC010_render_frame_dashboard_shows_event_ribbon_content` — right 40% area (x=60..100) contains hook type names from injected events
**Path:** success path + error path (empty events, pending revert, rolling-window drop)
**Status:** PASS

### AC-007 (BC-2.06.018 PC-2 — j/k/↓/↑ scroll; G→oldest; gg→newest)
**Evidence:** `AC-006-010-event-ribbon-integration.gif`
**Tests (event_ribbon.rs):**
- `test_BC_2_06_018_ec116_scroll_past_oldest_clamped` — scroll_ribbon_down at last index clamps (no panic, EC-116)
**Tests (event_ribbon_real_defects.rs):**
- `test_BC_2_06_018_INV1_AC009_session_change_resets_ribbon_scroll_via_dispatch` — session-change via production dispatch resets scroll to row 0
**Tests (render_frame_integration_s028.rs):**
- `test_BC_2_06_018_AC010_scroll_j_dispatches_scroll_down` — 'j' in Dashboard { EventRibbon }: offset 0→1, pinned_top→true (BC-2.06.018 PC-5)
- `test_BC_2_06_018_AC010_scroll_down_arrow_dispatches_scroll_down` — ↓ identical to 'j'
- `test_BC_2_06_018_AC010_scroll_j_twice_advances_two_rows` — two 'j' presses: 0→2
- `test_BC_2_06_018_AC010_scroll_k_dispatches_scroll_up` — 'k' in Dashboard { EventRibbon }: offset 3→2 (ScrollUp)
- `test_BC_2_06_018_AC010_scroll_k_at_row0_clears_pinned_top` — 'k' at row 1→row 0 clears pinned_top (auto-scroll resumes)
- `test_BC_2_06_018_AC007_G_jumps_to_oldest_event` — 'G' jumps to last index (oldest), sets pinned_top=true
- `test_BC_2_06_018_AC007_gg_jumps_to_newest_event` — 'gg' (two 'g' presses) jumps to row 0 (newest), clears pinned_top
**Path:** success path + edge case (clamp at oldest, k at top)
**Status:** PASS

### AC-008 (BC-2.06.018 PC-8 — auto-scroll follows newest unless pinned)
**Evidence:** `AC-006-010-event-ribbon-integration.gif`
**Tests (event_ribbon.rs):**
- `test_BC_2_06_018_auto_scroll_follows_bottom_when_not_pinned` — pinned_top=false: new event via on_hook_event_received → list_state.selected()=Some(0)
- `test_BC_2_06_018_auto_scroll_suppressed_when_pinned_top` — pinned_top=true: new event arrives, scroll offset stays at Some(3) (unchanged)
**Path:** success path (auto-follow) + success path (pin suppresses auto-follow)
**Status:** PASS

### AC-009 (BC-2.06.018 INV-1 — session change: client-side re-filter, reset scroll, no IPC)
**Evidence:** `AC-006-010-event-ribbon-integration.gif`
**Tests (event_ribbon.rs):**
- `test_BC_2_06_018_session_change_resets_scroll` — reset_on_session_change: offset 5→0, pinned_top true→false
- `test_BC_2_06_018_session_change_no_ipc_request` — reset_on_session_change signature (&mut EventRibbonState, &str): no App or IPC channel reference; IPC isolation structurally guaranteed
**Tests (event_ribbon_real_defects.rs):**
- `test_BC_2_06_018_INV1_AC009_session_change_resets_ribbon_scroll_via_dispatch` — session change via production dispatch_key_event (SelectNext) resets ribbon scroll to row 0, clears pinned_top
**Path:** success path + invariant (structural IPC isolation proof via function signature)
**Status:** PASS

### AC-010 (BC-2.06.006 PC-2 / BC-2.06.018 PC-5 — Event Ribbon + Sessions filter wired into render_frame / dispatch_key_event)
**Evidence:** `AC-006-010-event-ribbon-integration.gif`
**Tests (render_frame_integration_s028.rs):**
- `test_BC_2_06_018_AC010_render_frame_dashboard_shows_event_ribbon_content` — production render_frame renders Event Ribbon in right 40% area (x=60..100) with hook type names
- `test_BC_2_06_006_AC010_render_frame_filtering_mode_shows_filter_input_box` — production render_frame in Filtering mode renders "/ foo_" (filter input box with query and cursor)
- `test_BC_2_06_006_AC010_render_frame_filtering_zero_match_shows_sentinel` — production render_frame in Filtering mode with zero-match query renders SESSIONS_FILTER_NO_MATCH
- `test_BC_2_06_006_AC010_dispatch_filter_entry_and_exit_through_production_path` — '/' → Filtering → type 'f','o','o' → query="foo" → Esc → Dashboard; '/' again → 'm' → Enter → Dashboard (full filter cycle via dispatch_key_event)
- All scroll dispatch tests (AC-007 above) also satisfy AC-010's dispatch requirement
**Path:** success path + error path (zero-match sentinel, small-terminal safety) + full filter cycle
**Status:** PASS (6 integration render tests + 5 scroll dispatch tests + 1 dispatch filter cycle test = 12/12)

## F-W7G3-MED-001 Corrected Behavior Demonstration

The F-W7G3-MED-001 fix corrects the filter→ribbon session lookup. The old code used
`app.sessions.get(list_state_index)` where `list_state_index` is an index into the
scored/filtered list, but `app.sessions` is in insertion order. When a filter query
reorders sessions (session B at filtered-index 0, session A at filtered-index 1), the
old code showed session A's events (insertion-order index 0) while session B was
highlighted. The fix: `render_sessions_filter` returns the `session_id` of the
highlighted row from the scored list; `render_frame` passes it directly to EventRibbon.

**Evidence:** `AC-006-010-event-ribbon-integration.gif`
**Test:**
- `test_F_W7G3_MED_001_event_ribbon_shows_highlighted_session_events_in_filter_mode` — in Filtering mode with query="xyz":
  - sess-ALPHA (project "alpha-project", insertion index 0) has only "Notification" events
  - sess-ZETA (project "zeta-xyz", insertion index 1) has only "SessionStart" events
  - "xyz" scores "zeta-xyz" higher → sess-ZETA is at filtered-list position 0 (highlighted)
  - Ribbon right-40% region contains "SessionStart" (ZETA's event) — CORRECT
  - Ribbon right-40% region does NOT contain "Notification" (ALPHA's event) — confirming old bug is absent
**Status:** PASS — corrected behavior demonstrated

## Summary

| AC | Status | Evidence Artifact | Path |
|----|--------|-------------------|------|
| AC-001 | PASS | `AC-001-005-sessions-filter.gif` | success (both '/' and 'f') |
| AC-002 | PASS | `AC-001-005-sessions-filter.gif` | success + error (zero-match) + edge cases |
| AC-003 | PASS | `AC-001-005-sessions-filter.gif` | success (Enter + Esc) |
| AC-004 | PASS | `AC-001-005-sessions-filter.gif` | success |
| AC-005 | PASS | `AC-001-005-sessions-filter.gif` | invariant (behavioral + structural) |
| AC-006 | PASS | `AC-006-010-event-ribbon-integration.gif` | success + error |
| AC-007 | PASS | `AC-006-010-event-ribbon-integration.gif` | success + edge case (clamp) |
| AC-008 | PASS | `AC-006-010-event-ribbon-integration.gif` | success (follow) + success (pin) |
| AC-009 | PASS | `AC-006-010-event-ribbon-integration.gif` | success + invariant |
| AC-010 | PASS | `AC-006-010-event-ribbon-integration.gif` | success + error + full dispatch cycle |
| F-W7G3-MED-001 | PASS | `AC-006-010-event-ribbon-integration.gif` | corrected filter→ribbon lookup |

All 10 ACs have recorded evidence. No ACs without evidence.
F-W7G3-MED-001 corrected behavior is explicitly demonstrated.
