---
story: S-028
phase: red-gate
date: 2026-06-01
agent: test-writer (ADV Pass-2)
---

# S-028 Red Gate Log — ADV Pass-2 Test Repair

## Summary

ADV Pass-2 findings + architect IPC/spec changes addressed. All new/repaired tests are
RED before implementation. Pre-existing correct tests remain GREEN.

---

## Test Files Modified

| File | Changes |
|------|---------|
| `tests/render_frame_integration_s028.rs` | Scroll nav tests uncommented + rewritten (5 RED); stale GREEN comments updated |
| `tests/event_ribbon.rs` | EC-116 de-tautologized (compile-gate RED); INV-1 strengthened (GREEN, correct); auto-scroll comments updated |
| `tests/event_ribbon_real_defects.rs` | Tests 5-9 comments updated to GREEN; Tests 10-11 added (compile-gate RED) |
| `tests/filter_sessions.rs` | INV-1 matcher test strengthened (behavioral, GREEN); display_name test kept; stale comments updated |

---

## RED Tests (new / repaired — must fail before implementation)

### Scroll Navigation — assertion-RED (BC-2.06.018 PC-5, AC-007, AC-010)

File: `tests/render_frame_integration_s028.rs`

| Test | RED Reason | Expected after fix |
|------|-----------|-------------------|
| `test_BC_2_06_018_AC010_scroll_j_dispatches_scroll_down` | `dispatch_key_event` with 'j' in Dashboard { EventRibbon } leaves offset at Some(0) (SelectNext is no-op in EventRibbon focus); asserts Some(1) | Some(1) after ScrollDown wired |
| `test_BC_2_06_018_AC010_scroll_down_arrow_dispatches_scroll_down` | Same — ↓ key maps to SelectNext (no-op in EventRibbon); asserts Some(1) | Some(1) |
| `test_BC_2_06_018_AC010_scroll_j_twice_advances_two_rows` | Two 'j' presses leave offset at Some(0); asserts Some(2) | Some(2) |
| `test_BC_2_06_018_AC010_scroll_k_dispatches_scroll_up` | 'k' maps to SelectPrev (no-op in EventRibbon); offset stays Some(3); asserts Some(2) | Some(2) |
| `test_BC_2_06_018_AC010_scroll_k_at_row0_clears_pinned_top` | 'k' from row 1 does not move offset; asserts Some(0) + !pinned_top | Some(0), pinned_top=false |

Missing symbols required by implementer:
- `Action::ScrollDown` — add to `monocle-core/src/tui/state.rs`
- `Action::ScrollUp` — add to `monocle-core/src/tui/state.rs`
- Wire both in `dispatch_key_event` for `Dashboard { focused: EventRibbon }` context
- `'j'`/`↓` → `ScrollDown` (scroll toward older events); `'k'`/`↑` → `ScrollUp` (toward newer)

### EC-116 Clamp — compile-gate RED (BC-2.06.018 EC-116)

File: `tests/event_ribbon.rs`

| Test | RED Reason | Expected after fix |
|------|-----------|-------------------|
| `test_BC_2_06_018_ec116_scroll_past_oldest_clamped` | `scroll_ribbon_down` does not exist in `event_ribbon` module | Compiles; asserts selected stays at Some(2) (clamped) |

Missing symbol: `scroll_ribbon_down(state: &mut EventRibbonState, events: &VecDeque<HookEventRow>)` in `crates/monocle-tui/src/ui/event_ribbon.rs`

### Daemon Timestamp on Streaming Events — compile-gate RED (BC-2.05.004 PC-2, SS-ipc ADR)

File: `tests/event_ribbon_real_defects.rs` — Test 10

| Test | RED Reason | Expected after fix |
|------|-----------|-------------------|
| `test_BC_2_05_004_PC2_streaming_event_uses_daemon_timestamp_micros` | `ServerToClient::HookEventReceived` has no `timestamp_micros` field; `on_hook_event_received` takes 5 args, not 6 | Compiles; asserts `row.timestamp_micros == 1_705_311_000_000_000` and formatted timestamp `"09:30:00.000"` |

Missing symbols:
- `timestamp_micros: i64` field on `ServerToClient::HookEventReceived` (SS-ipc breaking change — `monocle-ipc/src/types.rs`)
- 6th parameter `timestamp_micros: i64` on `on_hook_event_received` (must propagate daemon's timestamp to `hook_event_row_from_received`)

### display_name Filter Match — compile-gate RED (BC-2.06.006 PC-3, EnrichedSession ADR)

File: `tests/event_ribbon_real_defects.rs` — Test 11

| Test | RED Reason | Expected after fix |
|------|-----------|-------------------|
| `test_BC_2_06_006_PC3_filter_matches_session_display_name_field_not_hardcoded_map` | `EnrichedSession::new_with_display_name` does not exist (no `display_name` field on `EnrichedSession`) | Compiles; session with `harness_type="unknown-engine-xyz"` and `display_name="Claude Code"` visible with query "Cla" |

Missing symbols:
- `display_name: String` field on `EnrichedSession` (`monocle-core/src/engine.rs`)
- `EnrichedSession::new_with_display_name(...)` constructor (or update existing `new` to accept `display_name`)
- `render_sessions_filter` must score against `session.display_name` in addition to (or instead of) `harness_display_name(&s.harness_type)`

---

## GREEN Tests (pre-existing + repaired to assert real behavior)

| Test | File | Status |
|------|------|--------|
| All 14 event_ribbon.rs tests (except EC-116) | `event_ribbon.rs` | GREEN |
| All 13 filter_sessions.rs tests | `filter_sessions.rs` | GREEN |
| All 32 startup_connect.rs tests | `startup_connect.rs` | GREEN |
| Tests 1, 2a, 2b, filter dispatch (render_frame) | `render_frame_integration_s028.rs` | GREEN |
| Tests 5, 6a, 6b, 7, 8, 9 | `event_ribbon_real_defects.rs` | GREEN |

---

## Compile-Gate Details

```
error[E0432]: unresolved import `monocle_tui::ui::event_ribbon::scroll_ribbon_down`
error[E0559]: variant `ServerToClient::HookEventReceived` has no field named `timestamp_micros`
error[E0061]: function `on_hook_event_received` takes 5 arguments but 6 supplied
error[E0599]: no function or associated item `new_with_display_name` found for struct `EnrichedSession`
```

---

## Stale Comment Remediation

All stale "RED: FAILS because..." comments on tests that are now GREEN have been updated
to "GREEN: ..." descriptions. The Green/Red comment state now accurately reflects the
current production code state, preventing confusion for the implementer.

Tests 5-9 in `event_ribbon_real_defects.rs` comments updated from RED to GREEN.
Tests 1, 2a, 2b in `render_frame_integration_s028.rs` comments updated from RED to GREEN.
Auto-scroll (PC-8) tests in `event_ribbon.rs` comments updated from RED to GREEN.
INV-1 session-change test in `event_ribbon.rs` strengthened with behavioral assertions.
