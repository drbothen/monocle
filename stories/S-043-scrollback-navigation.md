---
document_type: story
level: L4
story_id: S-043
epic_id: EPIC-09
version: "1.1"
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
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.007.md, version: "1.1.3"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.7.0"}
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
`pty_parsers[focused_session_id].screen().scrollback_len()` — it CANNOT exceed the number of
available scrollback rows. If already at the maximum, the offset stays clamped.

### AC-003 (traces to BC-2.09.007 postcondition 2b — PtyScrollDown decrements per-session offset; floor 0)

`Action::PtyScrollDown` in `AppMode::EmbeddedTerminal` decrements
`App::pty_scroll_offsets[focused_session_id]` toward 0. Floor is 0 (live tail). Scrolling
down when already at 0 is a no-op (no error, no state change).

### AC-004 (traces to BC-2.09.007 postcondition 2c — clamp on both ends)

Both `PtyScrollUp` and `PtyScrollDown` clamp `pty_scroll_offsets[focused_session_id]`:
- Upper bound: `pty_parsers[focused_session_id].screen().scrollback_len()` (available rows).
- Lower bound: 0.

### AC-005 (traces to BC-2.09.007 postcondition 2d — per-session offset independence on focus switch)

Each session's scroll offset is stored independently in `pty_scroll_offsets: HashMap<String, usize>`.
Switching focus (arrow key in sessions panel) preserves each session's offset in its own entry.
Focus switch does NOT reset the incoming session's offset. The newly focused session renders
with its own `pty_scroll_offsets[session_id]`.

### AC-006 (traces to BC-2.09.007 postcondition 3 — no IPC sent for scroll actions)

Neither `PtyScrollUp` nor `PtyScrollDown` sends any IPC message (`ResizePane`, `KeyInput`, or
otherwise). Scrollback navigation is a TUI-local viewport operation only.

### AC-007 (traces to BC-2.09.007 postcondition 4 — scrolled-back indicator in status bar)

When `pty_scroll_offsets[focused_session_id] > 0`, the status bar displays a visible indicator
such as `[scrolled back N rows]` (where N is the current offset value). When the offset is 0
(live tail), the indicator is absent.

### AC-008 (traces to BC-2.09.007 postcondition 5 — new PTY output does not force viewport to bottom)

New `PtyOutput` received while the user is scrolled back does NOT force the viewport to jump
to the bottom. The `pty_scroll_offsets[focused_session_id]` is preserved when new bytes arrive.
The user must explicitly `PtyScrollDown` to return to live output.

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

`PtyScrollUp` when `pty_scroll_offsets[focused] == scrollback_len()` (already at oldest row)
leaves the offset at the maximum. No error. No panic.

### AC-013 (traces to BC-2.09.007 edge case EC-241 — scroll down at live tail; no-op)

`PtyScrollDown` when `pty_scroll_offsets[focused] == 0` leaves the offset at 0. No error.
No IPC sent.

### AC-014 (traces to BC-2.09.007 edge case EC-244 — new output while scrolled back; indicator remains)

When new `PtyOutput` arrives for the focused session while `pty_scroll_offsets[focused] > 0`,
the parser is updated AND `pty_scroll_offsets[focused]` is preserved (not reset). The status
bar shows `[scrolled back N rows]`.

## Tasks

- [ ] Verify `App::pty_scroll_offsets: HashMap<String, usize>` field exists (added in S-039); if not, add it.
- [ ] Implement `Action::PtyScrollUp` handler in `crates/monocle-tui/src/app.rs`: increment `pty_scroll_offsets[focused_session_id]`; clamp to `parsers[id].screen().scrollback_len()`; no IPC.
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
- [ ] Write unit test `test_BC_2_09_007_clamp_at_max`: scroll past `scrollback_len()`; assert clamped.
- [ ] Write unit test `test_BC_2_09_007_focus_switch_preserves_offsets`: "s1" offset=10; switch to "s2" offset=0; assert `pty_scroll_offsets["s1"]=10`; render uses `pty_scroll_offsets["s2"]=0`.
- [ ] Write unit test `test_BC_2_09_007_no_ipc_for_scroll`: `PtyScrollUp`; assert no `ResizePane` or `KeyInput` in IPC sink.
- [ ] Write unit test `test_BC_2_09_007_scrollback_rows_default_1000`: no `pty_scrollback_rows` in config; assert parser initialized with 1000.
- [ ] Write unit test `test_BC_2_09_007_scrollback_rows_capped_10000`: config `pty_scrollback_rows: 15000`; assert clamped to 10000.
- [ ] Write unit test `test_BC_2_09_007_new_output_does_not_reset_scroll_offset`: scrolled to offset=10; `PtyOutput` arrives; assert offset still 10.

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
- New `PtyOutput` MUST NOT reset scroll offset. The user's viewport choice is preserved until they explicitly scroll down.
- Memory bound: 10000 rows × 80 cols × ~16 bytes/cell ≈ 12.8 MB per session. The 10000-row cap is justified by this bound. Do NOT allow values above 10000.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `vt100` | `=0.16.2` (exact) | `vt100::Parser::new()` with scrollback_rows; `screen().scrollback_len()` | SS-deps-pin-manifest-v2-delta.md |
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
| EC-240 | Scroll past scrollback_len() | Clamped; no error |
| EC-241 | Scroll down at live tail (offset=0) | No-op; stays at 0 |
| EC-242 | Config `pty_scrollback_rows: 20000` | Clamped to 10000 |
| EC-243 | Config `pty_scrollback_rows: 0` | Clamped to 1 |
| EC-244 | New output arrives while scrolled back | Parser updated; offset preserved; indicator still shown |

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because scrollback navigation and the `pty_scroll_offsets` per-session viewport are defined in SS-embedded-pty.md §Scrollback navigation and §Parser ownership in TUI (I7 fix) — the authoritative SS-09 scrollback spec.

**Dependency Anchors:**
- S-043 depends on S-039 because S-039 creates `pty_parsers` and `pty_scroll_offsets` (the infrastructure) and defines `AppMode::EmbeddedTerminal` that gates the `PtyScrollUp`/`PtyScrollDown` action dispatch.
- S-043 does not block other SS-09 stories — scrollback is an independent enhancement to the EmbeddedTerminal experience.
