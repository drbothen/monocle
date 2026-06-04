---
document_type: behavioral-contract
level: L3
version: "1.2.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "2d6731a"
traces_to: prd.md
origin: greenfield
subsystem: SS-09
capability: CAP-009
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.09.003: Mouse Events Forwarded to PTY in SGR Encoding When in EmbeddedTerminal

## Description

When `AppMode::EmbeddedTerminal` is active, mouse events received by crossterm are
translated to SGR (1006) mouse encoding sequences and forwarded to the PTY stdin as
`KeyInput` IPC messages. SGR mouse mode is enabled by writing `ESC [ ? 1006 h` to the
terminal when entering `EmbeddedTerminal` mode. Mouse events include button press/release,
scroll (wheel), and motion events. This enables mouse-driven Claude Code TUI features
(e.g., clicking on file paths, scrolling output) to work inside monocle's embedded terminal.

## Preconditions

1. `AppMode::EmbeddedTerminal { session_id }` is active.
2. `crossterm::event::EnableMouseCapture` was called during `App::enter_embedded_terminal()`
   — mouse capture is scoped to `EmbeddedTerminal` entry, NOT globally active at TUI startup
   (I3 fix per SS-embedded-pty.md v1.3.0: global mouse capture is NOT used; it would steal
   text selection from monocle's own panels).
3. SGR mouse mode (`ESC [ ? 1006 h`) has been written to the terminal on EmbeddedTerminal entry
   (immediately after `EnableMouseCapture`).
4. The terminal emulator hosting monocle supports SGR extended mouse reporting.

## Postconditions

1. When a `crossterm::event::Event::Mouse(event)` is received in `AppMode::EmbeddedTerminal`,
   `mouse_event_to_pty_bytes(event, pane_area: Rect)` is called.
2. The function returns `Some(bytes)` encoding the event in SGR mouse mode:
   `CSI < Ps ; Px ; Py M` (press) or `CSI < Ps ; Px ; Py m` (release).
   - `Ps` encodes button number: 0 = left, 1 = middle, 2 = right; 64 = scroll up; 65 = scroll down.
   - `Px`, `Py` are the 1-indexed column and row coordinates of the event within the PTY pane area
     (offset applied so that clicks in the pane margins are not misreported).
   - `M` for press; `m` for release.
3. The encoded bytes are sent as `ClientToServer::KeyInput { session_id, bytes }`.
4. The daemon forwards to session-host → PTY stdin as `DaemonToHost::KeyInput { bytes }`.
5. Mouse events outside the PTY pane area (e.g., click on the status bar) are NOT forwarded
   to the PTY — `mouse_event_to_pty_bytes()` returns `None` if the event coordinates are
   outside the PTY widget's `Rect`.

## Invariants

1. Mouse capture and SGR 1006 are symmetric and SCOPED to `EmbeddedTerminal`:
   - **Entry** (`App::enter_embedded_terminal()`): `crossterm::execute!(stdout(), EnableMouseCapture)?`
     followed by `print!("\x1b[?1006h")` (SGR mouse mode on).
   - **Exit** (`App::exit_embedded_terminal()`): `print!("\x1b[?1006l")` (SGR mouse mode off)
     followed by `crossterm::execute!(stdout(), DisableMouseCapture)?`.
   - Ordering is critical: SGR `l` BEFORE `DisableMouseCapture` on exit.
   - `EnableMouseCapture` is NOT called at TUI startup; it is NOT globally active outside
     `EmbeddedTerminal` mode. Rationale: global mouse capture would steal terminal text
     selection from monocle's sessions panel, event ribbon, and other panels. See
     SS-embedded-pty.md v1.3.0 §I3 UX tradeoff.
2. `mouse_event_to_pty_bytes(event, pane_area: Rect)` is a PURE function — no I/O or state mutation.
3. Mouse motion events (`MouseEventKind::Moved`) are forwarded only if button 1/2/3 tracking
   is active (i.e., the harness's TUI has requested motion reporting). monocle always forwards
   motion events in EmbeddedTerminal mode to maximize compatibility.
4. Scroll events (`MouseEventKind::ScrollUp / ScrollDown`) are mapped to `Ps=64` (up) and
   `Ps=65` (down) per SGR standard.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-220 | Click at (row=0, col=0) within PTY pane area | SGR sequence with Px=1, Py=1 forwarded (1-indexed) |
| EC-221 | Click outside PTY pane area (e.g., status bar row) | `None` returned; not forwarded; no spurious PTY mouse event |
| EC-222 | Scroll wheel (MouseEventKind::ScrollUp) | SGR `\x1b[<64;Px;PyM` forwarded |
| EC-223 | Terminal does not support SGR mouse (reports normal X10 mouse) | SGR mode `ESC [ ? 1006 h` is still written; terminal may silently ignore it; normal X10 mouse events may arrive; `mouse_event_to_pty_bytes()` still encodes with SGR format; worst case = garbled mouse in harness (acceptable — harness can be run without mouse support) |

## Canonical Test Vectors

| Input | Expected PTY bytes | Category |
|-------|-------------------|----------|
| Left button press at (row=3, col=5) | `\x1b[<0;5;3M` | happy-path |
| Left button release at (row=3, col=5) | `\x1b[<0;5;3m` | happy-path |
| Scroll up at (row=10, col=20) | `\x1b[<64;20;10M` | happy-path |
| Click outside PTY pane area | `None` — not forwarded | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `mouse_event_to_pty_bytes()` produces correct SGR sequences for common events | unit |
| VP-TBD | Out-of-pane clicks return `None` | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — mouse forwarding is explicitly named in CAP-009 ("mouse") as part of the full-fidelity keyboard forwarding capability |
| Architecture Module | monocle-core (`mouse_event_to_pty_bytes()` pure function); monocle-tui (EmbeddedTerminal event handler, SGR mode write) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md v1.3.0 §Mouse support (SGR mode); §I3 UX tradeoff (scoped mouse capture) |
| Test Name | test_BC_2_09_003_mouse_events_sgr_encoded |

## Related BCs

- [BC-2.09.002] — composes with: mouse events are a sub-class of full-fidelity input forwarding

## Architecture Anchors

- `architecture/SS-embedded-pty.md#mouse-support-sgr-mode` — SGR encoding specification

## Story Anchor

S-TBD — Implement mouse_event_to_pty_bytes() and SGR mode entry (filled by story-writer)

## VP Anchors

VP-TBD — Mouse event SGR encoding unit tests (filled after VP creation)

## §Trace v1.2.0

**I2-001 adversarial pass-2 fix — scoped mouse capture model** (2026-06-03):
- I2-001 finding: PC-2 + Invariant 1 still encoded the PRE-fix GLOBAL mouse-capture model
  ("EnableMouseCapture globally active"; "DisableMouseCapture is NOT called on exit"). These
  preconditions contradicted the architecture (SS-embedded-pty v1.3.0, I3 fix) which scopes
  mouse capture to EmbeddedTerminal entry/exit.
- PC-2 rewritten: `EnableMouseCapture` called at `EmbeddedTerminal` entry (not globally at
  TUI startup). Rationale for scoping added: global capture would steal terminal text selection.
- Invariant 1 rewritten: symmetric Enable/Disable with correct entry/exit sequence and SGR
  `h/l` ordering. References SS-embedded-pty.md v1.3.0 §I3 UX tradeoff for authoritative
  design rationale.
- Architecture Source updated to reference SS-embedded-pty.md v1.3.0.

## §Trace v1.1.0

**Adversarial pass-1 fix — parameter rename screen_offset → pane_area: Rect** (2026-06-03):
- PC-1: renamed `mouse_event_to_pty_bytes(event, screen_offset)` to
  `mouse_event_to_pty_bytes(event, pane_area: Rect)`. This is a naming-alignment fix with
  SS-embedded-pty v1.1.0 which uses `pane_area: Rect` as the second parameter. No semantic
  change — the `Rect` type carries x/y origin and width/height, replacing the simpler
  `screen_offset` tuple. Boundary detection (EC-221, PC-5) uses the same `Rect` bounds logic.
- Invariant 2: parameter name updated for consistency.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.003 authored for SS-09 as part of the v1A control-center pivot BC burst.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
