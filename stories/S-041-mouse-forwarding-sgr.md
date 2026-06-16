---
document_type: story
level: L4
story_id: S-041
epic_id: EPIC-09
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 5
wave: 9
tdd_mode: strict
priority: P0
depends_on: [S-040]
blocks: [S-044]
target_module: monocle-core
subsystems: [SS-09]
behavioral_contracts: [BC-2.09.003]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.003.md, version: "1.5.2"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.7.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.09.003 (mouse events forwarded to PTY in SGR encoding when in EmbeddedTerminal; scoped EnableMouseCapture; SGR 1006 mode)"
# BC status at S-041 authoring time: BC-2.09.003 v1.5.1 — non-empty; status draft pending Phase-2 adversarial convergence gate
# Clustering rationale: BC-2.09.003 is standalone because it adds distinct effectful terminal I/O:
# EnableMouseCapture + SGR 1006 h/l writes at EmbeddedTerminal entry/exit — these are different lifecycle
# operations from keyboard setup (S-040). The scoped mouse capture contract (CC-GLOBAL-MOUSE-CAPTURE gate)
# makes this a discrete deliverable with its own risk boundary.
---

# S-041: Mouse Forwarding — mouse_event_to_pty_bytes, SGR 1006 Scoped Entry/Exit, Out-of-Pane Clip

## Narrative

As the monocle TUI in `AppMode::EmbeddedTerminal`, I want all mouse button presses, releases,
scroll events, and drags to be translated to SGR 1006-encoded byte sequences and forwarded to
the PTY as `ClientToServer::KeyInput` — with mouse capture enabled only while embedded terminal
mode is active — so that Claude Code and other mouse-aware programs inside the embedded terminal
receive correct mouse input without stealing text selection from monocle's own panels.

## Acceptance Criteria

### AC-001 (traces to BC-2.09.003 precondition 2 / invariant 1 — EnableMouseCapture scoped to EmbeddedTerminal entry)

On `App::enter_embedded_terminal()`, the scoped entry sequence executes in this exact order:
1. `crossterm::execute!(stdout(), crossterm::event::EnableMouseCapture)?`
2. `print!("\x1b[?1006h")` (SGR extended mouse mode on)

`EnableMouseCapture` is NOT called at TUI startup. It is NOT globally active. This is the ratified
I3 scoped-capture design (CC-GLOBAL-MOUSE-CAPTURE sign-off gate applies to any change to this invariant).

### AC-002 (traces to BC-2.09.003 invariant 1 — DisableMouseCapture scoped to EmbeddedTerminal exit; SGR-l before DisableMouseCapture)

On `App::exit_embedded_terminal()`, the scoped exit sequence executes in this exact order:
1. `print!("\x1b[?1006l")` (SGR extended mouse mode off)
2. `crossterm::execute!(stdout(), crossterm::event::DisableMouseCapture)?`

Ordering is critical: SGR `l` MUST precede `DisableMouseCapture`. Inverting the order leaves
the terminal in a broken state.

### AC-003 (traces to BC-2.09.003 postcondition 1 — mouse_event_to_pty_bytes called on each mouse event in EmbeddedTerminal)

When `crossterm::event::Event::Mouse(event)` is received while `AppMode::EmbeddedTerminal` is
active, `mouse_event_to_pty_bytes(event, pane_area)` is called. `pane_area: Rect` is the `Rect`
of the PTY widget in the current frame.

### AC-004 (traces to BC-2.09.003 postcondition 2 — complete SGR encoding per base-Ps table)

`mouse_event_to_pty_bytes(event, pane_area)` returns `Some(bytes)` encoding the event in SGR
mouse mode `CSI < Ps_final ; Px ; Py M` (press/drag/scroll/moved) or `CSI < Ps_final ; Px ; Py m`
(release) where:
- Base `Ps` per the authoritative table:
  - `Down(Left)=0`, `Down(Middle)=1`, `Down(Right)=2` (terminator `M`)
  - `Up(Left)=0`, `Up(Middle)=1`, `Up(Right)=2` (terminator `m`)
  - `Drag(Left)=32`, `Drag(Middle)=33`, `Drag(Right)=34` (terminator `M`)
  - `Moved=35` (`3+32`; UNREACHABLE on Unix under 1002 tracking; retained for match-exhaustiveness and Windows)
  - `ScrollUp=64`, `ScrollDown=65`, `ScrollLeft=66`, `ScrollRight=67` (terminator `M`)
- Modifier bits additive: `Shift |= 4`, `Alt |= 8`, `Ctrl |= 16`. `Ps_final = base_Ps | modifier_bits`.
- Coordinate convention (1-indexed): `Px = col - pane_area.x + 1`, `Py = row - pane_area.y + 1`.
  Example: crossterm `(column: 10, row: 5)` at pane origin → `\x1b[<0;11;6M` (Px=11, Py=6).

### AC-005 (traces to BC-2.09.003 postcondition 3 — bytes sent as KeyInput IPC)

The encoded bytes are sent as `ClientToServer::KeyInput { session_id, bytes }` over the IPC channel.
The daemon forwards to the session-host → PTY stdin as `DaemonToHost::KeyInput`.

### AC-006 (traces to BC-2.09.003 postcondition 5 — out-of-pane clicks return None; not forwarded)

Mouse events with `event.column` or `event.row` outside the PTY pane `Rect` return `None`
from `mouse_event_to_pty_bytes()` and are NOT forwarded. No spurious PTY mouse event is sent
for clicks on the status bar or sessions panel.

### AC-007 (traces to BC-2.09.003 invariant 2 — mouse_event_to_pty_bytes is pure; uses core-owned types)

`mouse_event_to_pty_bytes(event: PtyMouseEvent, pane_area: PtyRect) -> Option<Vec<u8>>` is a pure
function with no I/O or state mutation. It lives in `monocle-core/src/keyboard.rs` (pure core), not
`monocle-tui`. The parameter types are `PtyMouseEvent` and `PtyRect` (core-owned mirror types defined
in S-040's `monocle-core/src/keyboard.rs`), NOT `crossterm::event::MouseEvent` or `ratatui::layout::Rect`.
The `monocle-tui` dispatch site converts crossterm/ratatui types via `keyboard_conv::crossterm_mouse_to_pty()`
and `keyboard_conv::ratatui_rect_to_pty()` before calling this function (F-P2-I06 ruling).

### AC-008 (traces to BC-2.09.003 invariant 3 — 1002 button-event tracking; Moved unreachable on Unix)

`EnableMouseCapture` enables mode 1002 (button-event tracking) — NOT 1003 (any-event tracking).
`MouseEventKind::Moved` (Ps=35, no-button motion) is UNREACHABLE on Unix in 1002 mode. It is
encoded correctly in the match arm (Ps=35) for Rust exhaustiveness and Windows correctness but
MUST NOT appear in production Unix inputs.

### AC-009 (traces to BC-2.09.003 edge case EC-220 — click at (row=0, col=0) within pane → SGR Px=1, Py=1)

A click at pane origin (`column=pane_area.x`, `row=pane_area.y`) produces `Px=1, Py=1` (1-indexed).

### AC-010 (traces to BC-2.09.003 edge case EC-221 — click outside pane returns None)

A click at coordinates outside the PTY pane `Rect` returns `None` from `mouse_event_to_pty_bytes()`.
No bytes sent.

### AC-011 (traces to BC-2.09.003 edge case EC-222 — scroll wheel forwarded)

`MouseEventKind::ScrollUp` produces `\x1b[<64;Px;PyM`. `MouseEventKind::ScrollDown` produces
`\x1b[<65;Px;PyM`. Both are forwarded to the PTY.

## Tasks

- [ ] Implement `mouse_event_to_pty_bytes(event: PtyMouseEvent, pane_area: PtyRect) -> Option<Vec<u8>>` in `crates/monocle-core/src/keyboard.rs` (alongside key functions) per the complete base-Ps table and modifier-bit additive rule. Uses `PtyMouseEvent` and `PtyRect` (core-owned types from S-040). Includes `Drag(btn)` arm (Ps = btn_base + 32) and `Moved` arm (Ps=35; UNREACHABLE on Unix — document in comment).
- [ ] Extend `crates/monocle-tui/src/keyboard_conv.rs` (created in S-040) with TWO new conversion functions per SS-embedded-pty.md §Conversion in monocle-tui: `crossterm_mouse_to_pty(e: crossterm::event::MouseEvent) -> PtyMouseEvent` and `ratatui_rect_to_pty(r: ratatui::layout::Rect) -> PtyRect`. These are the only additional crossterm/ratatui types that may appear in `keyboard_conv.rs` — confined to this file.
- [ ] Add `Event::Mouse(event)` dispatch arm in the `EmbeddedTerminal` event loop section of `crates/monocle-tui/src/event_loop.rs`: convert via `keyboard_conv::crossterm_mouse_to_pty(event)` and `keyboard_conv::ratatui_rect_to_pty(pane_area)`, then call `mouse_event_to_pty_bytes(pty_event, pty_rect)`; if `Some(bytes)`, send `ClientToServer::KeyInput { session_id, bytes }`.
- [ ] Extend `App::enter_embedded_terminal()` in `crates/monocle-tui/src/app.rs` with the scoped entry sequence: `EnableMouseCapture` then `print!("\x1b[?1006h")`.
- [ ] Extend `App::exit_embedded_terminal()` in `crates/monocle-tui/src/app.rs` with the scoped exit sequence: `print!("\x1b[?1006l")` then `DisableMouseCapture`.
- [ ] Wire `pane_area` from the last rendered frame into the event dispatch context so `mouse_event_to_pty_bytes()` can access it. Store `App::last_pty_pane_area: ratatui::layout::Rect` updated each render tick (monocle-tui is allowed to hold a ratatui::Rect field; the conversion to PtyRect happens at the dispatch call site via `keyboard_conv::ratatui_rect_to_pty()`).
- [ ] Write unit test `test_BC_2_09_003_mouse_events_sgr_encoded_left_press`: construct `PtyMouseEvent { kind: PtyMouseEventKind::Down(PtyMouseButton::Left), column: 5, row: 3, modifiers: PtyKeyModifiers::NONE }` with `pane_area = PtyRect { x: 0, y: 0, width: 80, height: 24 }`; assert `mouse_event_to_pty_bytes(event, pane_area) == Some("\x1b[<0;6;4M".as_bytes().to_vec())`. Tests pure monocle-core function directly.
- [ ] Write unit test `test_BC_2_09_003_mouse_events_sgr_encoded_left_release`: `PtyMouseEventKind::Up(PtyMouseButton::Left)` → terminator `m` → `\x1b[<0;6;4m`.
- [ ] Write unit test `test_BC_2_09_003_mouse_events_sgr_scroll_up`: `PtyMouseEventKind::ScrollUp` at `(col=20, row=10)` → `\x1b[<64;21;11M`.
- [ ] Write unit test `test_BC_2_09_003_drag_encoding`: `PtyMouseEventKind::Drag(PtyMouseButton::Left)` at `(col=10, row=5)` → `\x1b[<32;11;6M`.
- [ ] Write unit test `test_BC_2_09_003_out_of_pane_returns_none`: construct `PtyMouseEvent { column: 200, row: 200, .. }` where pane is 80x24; assert `None`.
- [ ] Write unit test `test_BC_2_09_003_1_indexed_origin`: event at `(column=pane.x, row=pane.y)` → `Px=1, Py=1`.
- [ ] Write unit test `test_BC_2_09_003_modifier_bits_ctrl`: `PtyMouseEventKind::Down(PtyMouseButton::Left)` + `PtyKeyModifiers::CONTROL` at pane origin → `\x1b[<16;1;1M` (`Ps_final = 0 | 16 = 16`).
- [ ] Write integration test `test_BC_2_09_003_scoped_mouse_capture_lifecycle`: verify `EnableMouseCapture` called on entry; `DisableMouseCapture` called on exit; ordering verified via mock terminal backend.

## Previous Story Intelligence

- **S-040** (keyboard forwarding): `App::enter_embedded_terminal()` and `App::exit_embedded_terminal()` skeletons exist (created in S-039). S-040 added the Kitty keyboard setup but explicitly deferred `EnableMouseCapture` to this story. The EmbeddedTerminal event dispatch arm exists; add `Event::Mouse` arm to it.
- **S-039** (PTY output pipeline): `App::last_pty_pane_area: Rect` needs to be added if not already present — the render loop must record the pane `Rect` so the mouse event handler can use it. Confirm whether this was added; if not, add it in this story.
- The `Moved` arm unreachability on Unix is intentional and MUST be documented with a comment. Do not remove it (Rust match exhaustiveness on a non-exhaustive enum).

## Architecture Compliance Rules

- `mouse_event_to_pty_bytes(event: PtyMouseEvent, pane_area: PtyRect)` is a pure function in `monocle-core/src/keyboard.rs`. No I/O. Uses ONLY core-owned `PtyMouseEvent` and `PtyRect` types — no crossterm or ratatui in the signature (F-P2-I06 ruling: SS-embedded-pty.md §Dependency Boundary, "monocle-core MUST NOT depend on crossterm or ratatui").
- Conversion from `crossterm::event::MouseEvent` → `PtyMouseEvent` and `ratatui::layout::Rect` → `PtyRect` is confined to `crates/monocle-tui/src/keyboard_conv.rs` (extended in this story from S-040's base). This is the ONLY place crossterm/ratatui mouse types touch the monocle-core purity boundary.
- `EnableMouseCapture` / `DisableMouseCapture` are effectful shell operations. They live in `App::enter_embedded_terminal()` / `App::exit_embedded_terminal()` in `monocle-tui`, NOT in `monocle-core`.
- Scoped-capture invariant: `EnableMouseCapture` is NOT called at TUI startup. ANY change to global mouse capture requires human sign-off on CC-GLOBAL-MOUSE-CAPTURE per ADR and SS-embedded-pty.md §I3 UX tradeoff.
- Coordinate indexing: 1-indexed output (`px = col - pane_area.x + 1`, `py = row - pane_area.y + 1`). The crossterm coordinate is 0-indexed terminal-window-relative; the SGR output is 1-indexed pane-relative.
- The `Drag(btn)` match arm is NON-OPTIONAL. Its absence is a compile error (non-exhaustive match on a non-`#[non_exhaustive]` enum variant). Ps values: `Drag(Left)=32`, `Drag(Middle)=33`, `Drag(Right)=34`.
- Forbidden: do NOT enable mode 1003 (any-event tracking). Only 1002 (via `EnableMouseCapture`) + 1006 (SGR via explicit `\x1b[?1006h` write).

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `crossterm` | `"0.29"` (caret) | `MouseEvent`, `MouseEventKind`, `MouseButton`, `EnableMouseCapture`, `DisableMouseCapture` | SS-deps-pin-manifest.md |
| `ratatui` | `"0.30"` (caret) | `Rect` (pane area for coordinate transform) | SS-deps-pin-manifest.md |
| `tokio` | `=1.52` (exact) | IPC send | SS-deps-pin-manifest.md §Exact-pinned |

## File Structure Requirements

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-core/src/keyboard.rs` | Add `mouse_event_to_pty_bytes(event: PtyMouseEvent, pane_area: PtyRect) -> Option<Vec<u8>>` with complete Ps table (all variants including `Drag` and `Moved`). Uses core-owned `PtyMouseEvent`/`PtyRect` types from S-040. |
| `crates/monocle-tui/src/keyboard_conv.rs` | Extend (created in S-040) with `crossterm_mouse_to_pty(e: MouseEvent) -> PtyMouseEvent` and `ratatui_rect_to_pty(r: Rect) -> PtyRect`. This is the ONLY place these crossterm/ratatui types touch the purity boundary. |
| `crates/monocle-tui/src/app.rs` | Extend `enter_embedded_terminal()` with `EnableMouseCapture` + SGR 1006h; extend `exit_embedded_terminal()` with SGR 1006l + `DisableMouseCapture`; add `last_pty_pane_area: ratatui::layout::Rect` field |
| `crates/monocle-tui/src/event_loop.rs` | Add `Event::Mouse(event)` arm in `EmbeddedTerminal` event dispatch section; convert via `keyboard_conv::crossterm_mouse_to_pty(event)` and `keyboard_conv::ratatui_rect_to_pty(pane_area)` before calling `mouse_event_to_pty_bytes()` |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~4,500 |
| BC-2.09.003 | ~4,500 |
| SS-embedded-pty.md §Mouse support; §I3 UX tradeoff; §mouse_event_to_pty_bytes code | ~6,000 |
| monocle-core/src/keyboard.rs (existing key functions from S-040) | ~3,000 |
| monocle-tui/src/app.rs (enter/exit embedded terminal from S-039) | ~3,000 |
| Test files to write | ~4,500 |
| **Total estimate** | **~25,500** |

Within the 30% context window bound. No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.09.003 | Mouse Events Forwarded to PTY in SGR Encoding When in EmbeddedTerminal | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `mouse_event_to_pty_bytes(event: PtyMouseEvent, pane_area: PtyRect)` | `monocle-core/src/keyboard.rs` | Pure core (no I/O; deterministic; uses core-owned Pty* types only; no crossterm/ratatui) |
| `crossterm_mouse_to_pty()`, `ratatui_rect_to_pty()` | `monocle-tui/src/keyboard_conv.rs` (extended in this story) | Effectful shell boundary (infallible conversions; confined to keyboard_conv.rs) |
| EmbeddedTerminal Mouse dispatch arm | `monocle-tui/src/event_loop.rs` | Effectful shell (IPC send) |
| `EnableMouseCapture` + SGR 1006h on entry | `monocle-tui/src/app.rs` (enter_embedded_terminal) | Effectful shell (terminal device I/O) |
| `DisableMouseCapture` + SGR 1006l on exit | `monocle-tui/src/app.rs` (exit_embedded_terminal) | Effectful shell (terminal device I/O) |
| `App::last_pty_pane_area` | `monocle-tui/src/app.rs` | Pure core (cached Rect; no I/O) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-220 | Click at pane origin | Px=1, Py=1 forwarded (1-indexed) |
| EC-221 | Click outside PTY pane area | `None`; not forwarded |
| EC-222 | Scroll wheel (ScrollUp) | SGR `\x1b[<64;Px;PyM` forwarded |
| EC-223 | Terminal does not support SGR mouse | SGR 1006h still written; terminal ignores; garbled mouse acceptable |

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because `mouse_event_to_pty_bytes()`, the scoped `EnableMouseCapture`/`DisableMouseCapture` lifecycle, and SGR 1006 entry/exit are all defined in SS-embedded-pty.md §Mouse support (SGR mode) and §I3 UX tradeoff — the authoritative SS-09 mouse contract.

**Dependency Anchors:**
- S-041 depends on S-040 because S-040 adds `App::enter_embedded_terminal()` / `App::exit_embedded_terminal()` (which S-041 extends) and establishes the EmbeddedTerminal event dispatch arm (which S-041 extends with a `Mouse` arm).
- S-041 blocks S-044 because S-044 (AppMode transitions + permission badge) requires the full enter/exit lifecycle to be in place, including the scoped mouse capture added by this story.
