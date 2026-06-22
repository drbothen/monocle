---
document_type: story
level: L4
story_id: S-043
epic_id: EPIC-09
version: "1.4"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
points: 3
wave: 9
tdd_mode: strict
priority: P1
depends_on: [S-039, S-042]
blocks: []
target_module: monocle-tui
subsystems: [SS-09]
behavioral_contracts: [BC-2.09.007]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.007.md, version: "1.4.1"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.15.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.09.007 (scrollback 1000 default; configurable; PtyScrollUp/Down per-session offset navigation; status bar indicator)"
# BC status: non-empty; status draft pending Phase-2 adversarial convergence gate (authoritative versions in inputs: frontmatter)
---

# S-043: Scrollback Navigation — PtyScrollUp/Down, Per-Session Offsets, Configurable Capacity

## Narrative

As the monocle TUI user in `AppMode::EmbeddedTerminal`, I want to scroll backward through
the session's PTY output using `PtyScrollUp` / `PtyScrollDown` actions — with per-session
independent scroll positions, a configurable scrollback buffer (default 1000 rows, max 10000),
and a status bar indicator showing how far back I am scrolled — so that I can review previous
Claude Code output without leaving the embedded terminal or resizing the PTY.

## Acceptance Criteria

### AC-001 (traces to BC-2.09.007 postcondition 1 — parser initialized with configured scrollback_rows)

`vt100::Parser::new(rows, cols, scrollback_rows)` is initialized with `scrollback_rows` equal to:
- `~/.monocle/config.json:pty_scrollback_rows` if present and valid.
- Default 1000 if the key is absent, null, or non-numeric.
- Clamped to a minimum of 1 and maximum of 10000. Values outside this range are silently
  clamped — no error is returned to the user.

### AC-002 (traces to BC-2.09.007 postcondition 2a — PtyScrollUp increments per-session offset)

`Action::PtyScrollUp` in `AppMode::EmbeddedTerminal` increments
`App::pty_scroll_offsets[focused_session_id]` by one scroll step. The offset is bounded by
the maximum available scrollback rows — it CANNOT exceed that count. The upper bound is
determined via the vt100 set_scrollback read-back probe (vt100 0.16.2 does not expose a
public scrollback length accessor). If already at the maximum, the offset stays clamped.

### AC-003 (traces to BC-2.09.007 postcondition 2b — PtyScrollDown decrements per-session offset; floor 0)

`Action::PtyScrollDown` in `AppMode::EmbeddedTerminal` decrements
`App::pty_scroll_offsets[focused_session_id]` toward 0. Floor is 0 (live tail). Scrolling
down when already at 0 is a no-op (no error, no state change).

### AC-004 (traces to BC-2.09.007 postcondition 2c — clamp on both ends)

Both `PtyScrollUp` and `PtyScrollDown` clamp `pty_scroll_offsets[focused_session_id]`:
- Upper bound: the maximum available scrollback rows (determined via the vt100 set_scrollback
  read-back probe, since vt100 0.16.2 does not expose a public scrollback length accessor).
- Lower bound: 0.

### AC-005 (traces to BC-2.09.007 postcondition 2d — per-session offset independence on focus switch)

Each session's scroll offset is stored independently in `pty_scroll_offsets: HashMap<String, usize>`.
Switching focus (arrow key in sessions panel) preserves each session's offset in its own entry.
Focus switch does NOT reset the incoming session's offset. The newly focused session renders
with its own `pty_scroll_offsets[session_id]`.

### AC-006 (traces to BC-2.09.007 postcondition 3 — no IPC sent for scroll actions)

Neither `PtyScrollUp` nor `PtyScrollDown` sends any IPC message (`ResizePane`, `KeyInput`, or
otherwise). Scrollback navigation is a TUI-local viewport operation only.

### AC-007 (traces to BC-2.09.007 postcondition 4 — scrolled-back indicator in status bar; concurrent with transient badges)

When `pty_scroll_offsets[focused_session_id] > 0`, the status bar displays a visible indicator
such as `[scrolled back N rows]` (where N is the current offset value). When the offset is 0
(live tail), the indicator is absent.

The `[scrolled back N rows]` indicator is **persistent viewport state** and is NEVER suppressed
by any transient diagnostic badge. Specifically:
- When a dump-drop warning (`[dump: N drops]`) is active, BOTH indicators render concurrently.
- When a permission badge (`[N pending permission(s)]`) is active, BOTH render concurrently.
- When a reconnect badge (`[reconnecting...]`) is active, BOTH render concurrently.
- The status bar accommodates multiple concurrent badges simultaneously.

Suppression would cause a silent correctness failure: the user believes they are at live tail
when they are not. This is a mandatory production-grade correctness requirement per BC-2.09.007
postcondition 4.

### AC-008 (traces to BC-2.09.007 postcondition 5 — new PTY output uses content-anchored offset preservation)

New `PtyOutput` received while the user is scrolled back does NOT force the viewport to jump
to the bottom. The `pty_scroll_offsets[focused_session_id]` is **content-anchored**: when new
bytes arrive and `pty_scroll_offsets[session_id] > 0`, the handler MUST:

1. Read `scrollback_before = parser.screen().scrollback()` (effective scrollback offset before
   processing; use the read-back probe to capture current scrollback depth).
2. Call `parser.process(&bytes)`.
3. Read `scrollback_after = parser.screen().scrollback()` (effective scrollback offset after
   processing). Note: vt100 0.16.2 does not expose a public `scrollback_len()` accessor on
   `Screen`; the maximum available scrollback rows are determined via the read-back probe:
   `set_scrollback(usize::MAX)` then `screen().scrollback()` yields the clamped effective maximum.
4. Add `new_rows = scrollback_after - scrollback_before` to `pty_scroll_offsets[session_id]`.
5. Clamp: `pty_scroll_offsets[session_id] = min(effective_max, pty_scroll_offsets[session_id])`
   where `effective_max` is determined via the read-back probe above.

This keeps the viewport pinned to the same content rows the user is viewing — new output
is appended at the bottom while the visible window stays in place.

When `pty_scroll_offsets[session_id] == 0` (live tail), `process(&bytes)` is called normally
and the offset stays at 0 (no adjustment). Live tail is never disturbed.

The user must explicitly `PtyScrollDown` to return to live output from a scrolled-back position.

### AC-009 (traces to BC-2.09.007 invariant 3a — scroll offset reset to 0 on ResizePane for that session)

When `ResizePane` fires for `session_id`, `pty_scroll_offsets[session_id]` is reset to 0
(live tail). Rationale: resize reflows content; the old row offset is meaningless against the
new layout. This rule applies regardless of whether the session is currently focused.

### AC-010 (traces to BC-2.09.007 invariant 3b — scroll offset entry removed on Terminated)

When a session transitions to `SessionState::Terminated` and is GC'd from the list,
`pty_scroll_offsets.remove(session_id)` is called. The entry is removed, not reset to 0.

### AC-011 (traces to BC-2.09.007 invariant 5 — HashMap per-session; NOT a shared single field)

`pty_scroll_offsets` is canonically `HashMap<String, usize>` keyed by `session_id`. There is
NO shared singular `pty_scroll_offset: usize` field. This is the I7 fix: a shared offset caused
focus-switch to show the wrong session's scrollback position.

### AC-012 (traces to BC-2.09.007 edge case EC-240 — scroll past beginning; clamped)

`PtyScrollUp` when `pty_scroll_offsets[focused]` already equals the maximum available scrollback
rows (already at oldest row) leaves the offset at the maximum. No error. No panic.

### AC-013 (traces to BC-2.09.007 edge case EC-241 — scroll down at live tail; no-op)

`PtyScrollDown` when `pty_scroll_offsets[focused] == 0` leaves the offset at 0. No error.
No IPC sent.

### AC-014 (traces to BC-2.09.007 edge case EC-244 — new output while scrolled back; content-anchored; indicator updated)

When new `PtyOutput` arrives for the focused session while `pty_scroll_offsets[focused] > 0`,
the parser is updated AND `pty_scroll_offsets[focused]` is incremented by the number of new
scrollback rows produced by that process call (content-anchored per AC-008). The offset is
then clamped to the maximum available scrollback rows (determined via the vt100 set_scrollback
read-back probe, since vt100 0.16.2 does not expose a public scrollback length accessor).
The status bar continues to show
`[scrolled back N rows]` with the updated N value. The viewport stays pinned to the same
content rows the user was reading before the new output arrived.

## Tasks

- [ ] Verify `App::pty_scroll_offsets: HashMap<String, usize>` field exists (added in S-039); if not, add it.
- [ ] Implement `Action::PtyScrollUp` handler in `crates/monocle-tui/src/app.rs`: increment `pty_scroll_offsets[focused_session_id]`; clamp to the maximum available scrollback rows via the vt100 read-back probe (`set_scrollback(usize::MAX)` then read `screen().scrollback()`); no IPC.
- [ ] Implement `Action::PtyScrollDown` handler in `crates/monocle-tui/src/app.rs`: decrement toward 0; no IPC.
- [ ] Add `PtyScrollUp` and `PtyScrollDown` to the `Action` enum in `monocle-core` if absent; bind to configurable keys (default: `Ctrl+Up` / `Ctrl+Down` while in `EmbeddedTerminal`).
- [ ] Thread `pty_scroll_offsets[focused_session_id]` into `render_embedded_terminal()` in
  `crates/monocle-tui/src/ui/embedded_terminal.rs` using the canonical tui-term 0.3.4 scrollback
  call sequence (authoritative: `.factory/specs/research/tui-term-0.3.4-scrollback-api.md`):
  ```rust
  // 1. Drive the scrollback offset on the Screen (mutates which rows the Screen reports):
  parser.screen_mut().set_scrollback(offset);  // offset from pending pty_scroll_offsets
  // 2. Hand the now-scrolled Screen (immutable ref) to the widget:
  let widget = tui_term::widget::PseudoTerminal::new(parser.screen())
      .block(/* optional */).style(/* optional */);
  frame.render_widget(widget, area);
  // 3. Read back clamped offset for status bar display:
  let effective_offset = parser.screen().scrollback();  // 0 == live bottom
  ```
  `PseudoTerminal` has NO `.scrollback()`, `.viewport()`, or `.offset()` builder method in
  0.3.4. The offset is applied entirely on the `vt100::Screen` via `set_scrollback(n)`.
  `set_scrollback(0)` returns to live view. The value is auto-clamped by vt100 to actual
  scrollback size — read back `screen().scrollback()` for the effective offset for the
  status bar `[scrolled back N rows]` affordance.
  `scrollback_len` MUST be `> 0` at `vt100::Parser::new()` construction or there is no
  history to view (S-039 owns this; S-043 consumes the configured value).
- [ ] Render status bar scrolled-back indicator: when `pty_scroll_offsets[focused] > 0`, append `[scrolled back N rows]` to the status bar in the embedded terminal layout.
- [ ] Ensure `pty_scroll_offsets[session_id]` is reset to 0 in the `ResizePane` handler. This reset is OWNED BY S-042 (resize handler in `crates/monocle-tui/src/app.rs`); S-043 depends on S-042 (`depends_on: [S-039, S-042]`) so S-042 is always complete when S-043 is dispatched. S-043 does NOT re-implement the resize handler — it verifies the reset is present; if for any reason it is absent (implementation gap), S-043 adds it and flags the discrepancy.
- [ ] Ensure `pty_scroll_offsets.remove(session_id)` is called in the session GC handler (`SessionState::Terminated`).
- [ ] The `pty_scrollback_rows` config load is OWNED BY S-039 (see S-039 AC-008 and Architecture Compliance Rules). S-043 asserts its existence via `App::scrollback_rows` field (set by S-039); it does NOT re-load the config. If S-039 has not yet created the `App::scrollback_rows` field, verify this before implementing S-043 (dependency on S-039 must be complete).
- [ ] Write unit test `test_BC_2_09_007_scrollup_increments_offset`: `PtyScrollUp` × 10; assert `pty_scroll_offsets["s1"] = 10`; other sessions unaffected.
- [ ] Write unit test `test_BC_2_09_007_scrolldown_decrements_floor_0`: at offset=0; `PtyScrollDown`; assert still 0; no error.
- [ ] Write unit test `test_BC_2_09_007_clamp_at_max`: scroll past maximum available scrollback rows; assert clamped.
- [ ] Write unit test `test_BC_2_09_007_focus_switch_preserves_offsets`: "s1" offset=10; switch to "s2" offset=0; assert `pty_scroll_offsets["s1"]=10`; render uses `pty_scroll_offsets["s2"]=0`.
- [ ] Write unit test `test_BC_2_09_007_no_ipc_for_scroll`: `PtyScrollUp`; assert no `ResizePane` or `KeyInput` in IPC sink.
- [ ] Write unit test `test_BC_2_09_007_scrollback_rows_default_1000`: no `pty_scrollback_rows` in config; assert parser initialized with 1000.
- [ ] Write unit test `test_BC_2_09_007_scrollback_rows_capped_10000`: config `pty_scrollback_rows: 15000`; assert clamped to 10000.
- [ ] Implement `on_pty_output` content-anchored logic in `crates/monocle-tui/src/app.rs`: when `pty_scroll_offsets[session_id] > 0`, capture `scrollback_before` via `screen().scrollback()`, call `parser.process(&bytes)`, capture `scrollback_after` via `screen().scrollback()`, compute `new_rows = scrollback_after - scrollback_before`, add to offset, clamp to effective max via the read-back probe (`set_scrollback(usize::MAX)` then `screen().scrollback()`). Note: vt100 0.16.2 does not expose a public `scrollback_len()` accessor — use the read-back probe. When offset == 0, call `process(&bytes)` only (no adjustment).
- [ ] Write unit test `test_BC_2_09_007_content_anchored_new_output`: scrolled to offset=10; 5 new rows arrive via `PtyOutput`; assert `pty_scroll_offsets["s1"] == 15` (incremented by new-row count); assert viewport rows unchanged; assert `[scrolled back 15 rows]` shown.
- [ ] Write unit test `test_BC_2_09_007_content_anchor_clamp_at_max`: scrolled to offset=990 (near max 1000); 20 new rows arrive; assert offset clamped to `min(1000, 990+20) = 1000`; no overflow, no error.
- [ ] Write unit test `test_BC_2_09_007_concurrent_status_bar_badges`: session scrolled back (offset > 0) AND dump-drop counter > 0 simultaneously; assert status bar renders BOTH `[scrolled back N rows]` AND `[dump: N drops]`; neither suppresses the other.

## Previous Story Intelligence

- **S-039** (PTY output pipeline): `App::pty_scroll_offsets: HashMap<String, usize>` is added in S-039. `vt100::Parser` initialization with `scrollback_rows` is also in S-039. Verify these are in place before adding duplicates.
- **S-042** (resize debounce): `pty_scroll_offsets[session_id]` reset to 0 on resize may have been added in S-042. Verify and add if missing.
- The tui-term 0.3.4 scrollback API is settled (`.factory/specs/research/tui-term-0.3.4-scrollback-api.md`
  — HIGH confidence, docs.rs verified). `PseudoTerminal` has NO viewport offset builder in 0.3.4.
  Scrollback is driven via `parser.screen_mut().set_scrollback(n)` BEFORE passing `parser.screen()`
  to `PseudoTerminal::new(...)`. No sub-screen slice manipulation needed. The implementer MUST use
  `screen_mut()` (mutable borrow) for `set_scrollback`, then `screen()` (immutable) for `PseudoTerminal`.
  Do NOT add `renderer.scroll()` or `widget.scrollback()` calls — these methods do not exist in 0.3.4.

## Architecture Compliance Rules

- `PtyScrollUp`/`PtyScrollDown` are pure TUI-local actions. NO IPC sent. Per BC-2.09.007 Postcondition 3, scroll navigation must not send `ResizePane`, `KeyInput`, or any other IPC.
- `pty_scroll_offsets` is `HashMap<String, usize>`. There MUST NOT be a shared singular `pty_scroll_offset: usize` field. The I7 fix is canonical.
- Scroll offset reset on resize is mandatory per SS-embedded-pty.md §Scrollback offset invariants. Do not omit.
- New `PtyOutput` MUST NOT reset scroll offset to zero. The `on_pty_output` handler uses **content-anchored** semantics: when `pty_scroll_offsets[session_id] > 0`, the offset MUST be incremented by `scrollback_after - scrollback_before` (new rows added by `parser.process(&bytes)`), then clamped to the maximum available scrollback rows (determined via the vt100 set_scrollback read-back probe — vt100 0.16.2 does not expose a public `scrollback_len()` accessor on `Screen`). A static numeric-preserve (offset unchanged) is INCORRECT — it causes the viewport to drift toward newer content as lines arrive. Live tail (offset == 0) is never adjusted. This matches the behavior of iTerm2, tmux, kitty, wezterm, and Alacritty.
- Status bar MUST render `[scrolled back N rows]` concurrently with all other status bar badges. The scrollback indicator MUST NOT be suppressed by any transient diagnostic badge (`[dump: N drops]`, `[N pending permission(s)]`, `[reconnecting...]`, or similar). When both conditions hold, both badges MUST render simultaneously.
- Memory bound: 10000 rows × 80 cols × ~16 bytes/cell ≈ 12.8 MB per session. The 10000-row cap is justified by this bound. Do NOT allow values above 10000.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `vt100` | `=0.16.2` (exact) | `vt100::Parser::new()` with scrollback_rows; `screen_mut().set_scrollback(n)`; `screen().scrollback()` for read-back probe (no public `scrollback_len()` in 0.16.2) | SS-deps-pin-manifest-v2-delta.md |
| `tui-term` | `=0.3.4` (exact) | `PseudoTerminal` scrollback rendering API | SS-deps-pin-manifest-v2-delta.md |
| `serde_json` | `=1.0.149` (exact) | `config.json:pty_scrollback_rows` deserialization | SS-deps-pin-manifest.md |
| `tokio` | `=1.52` (exact) | Runtime (no new direct usage in this story) | SS-deps-pin-manifest.md |
| `ratatui` | `"0.30"` (caret) | Status bar rendering (scrolled-back indicator) | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-tui/src/app.rs` | Add `PtyScrollUp`/`PtyScrollDown` action handlers; ensure `pty_scroll_offsets` reset on resize and remove on GC |
| `crates/monocle-core/src/actions.rs` (or equivalent) | Add `Action::PtyScrollUp` and `Action::PtyScrollDown` variants if absent |
| `crates/monocle-tui/src/ui/embedded_terminal.rs` | Thread scroll offset into `PseudoTerminal` rendering; add scrolled-back indicator in status bar |
| `crates/monocle-tui/src/config.rs` (or equivalent) | Ensure `pty_scrollback_rows` is loaded from config and clamped 1–10000 |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~4,000 |
| BC-2.09.007 | ~3,500 |
| SS-embedded-pty.md §Scrollback navigation; §Parser ownership §Scrollback offset invariants | ~4,000 |
| Existing App struct + render loop (S-039 context) | ~4,000 |
| tui-term 0.3.4 API reference | ~2,000 |
| Test files to write | ~4,000 |
| **Total estimate** | **~21,500** |

Within the 30% context window bound. No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.09.007 | Scrollback — 1000 Rows Default; Configurable; PtyScrollUp/Down Navigate | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `PtyScrollUp`/`PtyScrollDown` action handlers | `monocle-tui/src/app.rs` | Pure core logic (no I/O — increments/decrements in-memory map) |
| Scroll offset rendering | `monocle-tui/src/ui/embedded_terminal.rs` | Effectful shell (ratatui render) |
| `pty_scroll_offsets` | `monocle-tui/src/app.rs` | Pure core (HashMap; in-memory state) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-240 | Scroll past maximum available scrollback rows | Clamped; no error |
| EC-241 | Scroll down at live tail (offset=0) | No-op; stays at 0 |
| EC-242 | Config `pty_scrollback_rows: 20000` | Clamped to 10000 |
| EC-243 | Config `pty_scrollback_rows: 0` | Clamped to 1 |
| EC-244 | New output arrives while scrolled back | Parser processes bytes; offset is incremented by new-row count (content-anchored); viewport stays pinned to same content rows; `[scrolled back N rows]` indicator shown with updated N |
| EC-245 | Both scrolled-back AND dump-drop warning active simultaneously | Status bar renders BOTH `[scrolled back N rows]` AND `[dump: N drops]` concurrently; neither suppresses the other |

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because scrollback navigation and the `pty_scroll_offsets` per-session viewport are defined in SS-embedded-pty.md §Scrollback navigation and §Parser ownership in TUI (I7 fix) — the authoritative SS-09 scrollback spec.

**Dependency Anchors:**
- S-043 depends on S-039 because S-039 creates `pty_parsers` and `pty_scroll_offsets` (the infrastructure) and defines `AppMode::EmbeddedTerminal` that gates the `PtyScrollUp`/`PtyScrollDown` action dispatch.
- S-043 does not block other SS-09 stories — scrollback is an independent enhancement to the EmbeddedTerminal experience.
