---
document_type: behavioral-contract
level: L3
version: "1.5.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "6054018"
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

## Coordinate Convention (S23-001 fix — authoritative for all SS-09 mouse examples)

All coordinate examples in this BC (and in BC-2.09.002 EC-213) assume:
- The EmbeddedTerminal pane is at terminal origin: `pane_area.x = 0`, `pane_area.y = 0`.
- `crossterm::event::MouseEvent` provides 0-indexed `(column, row)` coordinates relative to
  the terminal window.
- SGR output is 1-indexed via the canonical formula (SS-embedded-pty.md lines 511-512):
  `px = col - pane_area.x + 1`, `py = row - pane_area.y + 1`.
- At origin: `px = col + 1`, `py = row + 1`.
- Example: crossterm `(column: 10, row: 5)` → `\x1b[<0;11;6M` (11 = 10+1, 6 = 5+1).
  This matches HS-EXP-015 step 15-16 exactly (the authoritative reference).

## Preconditions

1. `AppMode::EmbeddedTerminal { session_id }` is active.
2. `crossterm::event::EnableMouseCapture` was called during `App::enter_embedded_terminal()`
   — mouse capture is scoped to `EmbeddedTerminal` entry, NOT globally active at TUI startup
   (I3 fix per SS-embedded-pty.md v1.7.0: global mouse capture is NOT used; it would steal
   text selection from monocle's own panels).
3. SGR mouse mode (`ESC [ ? 1006 h`) has been written to the terminal on EmbeddedTerminal entry
   (immediately after `EnableMouseCapture`).
4. The terminal emulator hosting monocle supports SGR extended mouse reporting.

## Postconditions

1. When a `crossterm::event::Event::Mouse(event)` is received in `AppMode::EmbeddedTerminal`,
   `mouse_event_to_pty_bytes(event, pane_area: Rect)` is called.
2. The function returns `Some(bytes)` encoding the event in SGR mouse mode.
   The complete base-`Ps` table (per SS-embedded-pty.md v1.7.0 §mouse_event_to_pty_bytes):

   | `crossterm::MouseEventKind` | Base `Ps` | Terminator | Notes |
   |-----------------------------|-----------|-----------|-------|
   | `Down(Left)` | 0 | `M` | button press |
   | `Down(Middle)` | 1 | `M` | button press |
   | `Down(Right)` | 2 | `M` | button press |
   | `Up(Left)` | 0 | `m` | button release |
   | `Up(Middle)` | 1 | `m` | button release |
   | `Up(Right)` | 2 | `m` | button release |
   | `Drag(Left)` | 32 | `M` | button + 32 motion bit |
   | `Drag(Middle)` | 33 | `M` | button + 32 motion bit |
   | `Drag(Right)` | 34 | `M` | button + 32 motion bit |
   | `Moved` | 35 | `M` | 3 + 32; no-button motion. **UNREACHABLE on Unix** — crossterm enables 1002 button-event tracking (not 1003 any-event); retained for match-exhaustiveness + Windows correctness |
   | `ScrollUp` | 64 | `M` | wheel scroll up |
   | `ScrollDown` | 65 | `M` | wheel scroll down |
   | `ScrollLeft` | 66 | `M` | wheel scroll left |
   | `ScrollRight` | 67 | `M` | wheel scroll right |

   **Modifier-bit additive rule:** `Ps_final = base_Ps | modifier_bits`, where
   Shift |= 4, Alt |= 8, Ctrl |= 16. Example: Ctrl+Left-press = 0 | 16 = 16.

   Format: `CSI < Ps_final ; Px ; Py M` (press/drag/scroll/moved) or
   `CSI < Ps_final ; Px ; Py m` (release, i.e., `Up` variants).

   - `Px`, `Py` are the 1-indexed column and row coordinates of the event within the PTY pane
     area (offset applied so that clicks in the pane margins are not misreported).
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
     SS-embedded-pty.md v1.7.0 §I3 UX tradeoff.
2. `mouse_event_to_pty_bytes(event, pane_area: Rect)` is a PURE function — no I/O or state mutation.
3. **Motion delivery model — 1002 (button-event tracking), NOT 1003 (any-event tracking):**
   crossterm enables tracking mode 1002 (`CSI ? 1002 h`, enabled implicitly by
   `EnableMouseCapture`), which delivers motion ONLY while a mouse button is held — these are
   `Drag(button)` events with base Ps 32/33/34. Mode 1003 (`any-event`, Ps 35 `Moved` —
   no-button motion) is NOT enabled by monocle; `MouseEventKind::Moved` is therefore
   UNREACHABLE on Unix. `Moved` is retained in `mouse_event_to_pty_bytes()` for Rust
   match-exhaustiveness and correct Windows behavior only; it MUST NOT appear in production
   Unix inputs. The prior claim that "monocle always forwards motion events to maximize
   compatibility" described a 1003 any-event mode that is not enabled and is INCORRECT — remove
   it. See Invariant 4 / PC-2 (Moved is UNREACHABLE on Unix) for the authoritative statement.
4. The complete base-Ps enumeration: Down(L/M/R)=0/1/2 (terminator `M`); Up(L/M/R)=0/1/2
   (terminator `m`); Drag(L/M/R)=32/33/34 (button+32 motion bit, terminator `M`);
   Moved=35 (3+32, no-button motion, UNREACHABLE on Unix — see PC-2); ScrollUp=64;
   ScrollDown=65; ScrollLeft=66; ScrollRight=67. Modifier bits are additive: Shift|=4,
   Alt|=8, Ctrl|=16. This matches SS-embedded-pty.md v1.7.0 §mouse_event_to_pty_bytes
   exhaustively. The prior partial enumeration {0,1,2,64,65} was incomplete.

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
| Left button press at crossterm (row=3, col=5) — pane at origin | `\x1b[<0;6;4M` (Px=5+1=6, Py=3+1=4) | happy-path |
| Left button release at crossterm (row=3, col=5) — pane at origin | `\x1b[<0;6;4m` (Px=5+1=6, Py=3+1=4) | happy-path |
| Scroll up at crossterm (row=10, col=20) — pane at origin | `\x1b[<64;21;11M` (Px=20+1=21, Py=10+1=11) | happy-path |
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
| Architecture Source | SS-embedded-pty.md v1.7.0 §Mouse support (SGR mode); §I3 UX tradeoff (scoped mouse capture) |
| Test Name | test_BC_2_09_003_mouse_events_sgr_encoded |

## Related BCs

- [BC-2.09.002] — composes with: mouse events are a sub-class of full-fidelity input forwarding

## Architecture Anchors

- `architecture/SS-embedded-pty.md#mouse-support-sgr-mode` — SGR encoding specification

## Story Anchor

S-041 — Implement mouse_event_to_pty_bytes() and SGR mode entry

## VP Anchors

VP-TBD — Mouse event SGR encoding unit tests (filled after VP creation)

## §Trace v1.5.1

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-041** (2026-06-15):
- Story Anchor filled from Phase-2 Burst B story decomposition. No behavioral content changed.

## §Trace v1.5.0

**F-P36-IMP-001 — Invariant 3 rewritten: 1002 button-event tracking model; stale any-event/1003 claim removed** (2026-06-13 / D-278):
- F-P36-IMP-001: Invariant 3 previously stated "Mouse motion events (MouseEventKind::Moved) are
  forwarded only if button 1/2/3 tracking is active... monocle ALWAYS forwards motion events in
  EmbeddedTerminal mode to maximize compatibility." This contradicted PC-2 (Moved table note:
  "UNREACHABLE on Unix") and Invariant 4 (explicitly marks Moved UNREACHABLE) by implying monocle
  uses 1003 any-event tracking mode.
- **Root cause:** crossterm's `EnableMouseCapture` enables mode 1002 (button-event tracking,
  `CSI ? 1002 h`) — not mode 1003 (any-event tracking). Under 1002, motion is delivered only
  while a button is held, as `Drag(button)` events (base Ps 32/33/34). `MouseEventKind::Moved`
  (Ps 35, no-button motion) is only produced under 1003, which monocle does NOT enable.
  The "always forwards motion events" claim described a 1003 mode that does not exist in monocle.
- **Invariant 3 rewritten:** Now accurately describes the 1002 tracking model — `Drag(button)`
  events (Ps 32/33/34) are the only motion variants delivered on Unix; `Moved` (Ps 35) is
  UNREACHABLE on Unix and retained solely for Rust match-exhaustiveness and Windows correctness.
  The inaccurate "maximize compatibility" phrase is removed. Invariant is now internally consistent
  with PC-2 table Moved row and Invariant 4.
- **Whole-class scan of BC-2.09.001, BC-2.09.002, BC-2.09.003:** No other stale "any-event" /
  1003 / "always forwards motion" / Moved-reachable claims found. BC-2.09.001 (PTY output
  pipeline) contains no mouse tracking mode claims. BC-2.09.002 Invariant 4 states "SGR mouse mode
  (`ESC [ ? 1006 h`) is written to the terminal when entering `AppMode::EmbeddedTerminal`, in
  addition to the globally-active mouse capture" — this is accurate (SGR 1006 is the encoding
  mode, separate from the tracking mode; 1006 + 1002 is the correct combination). BC-2.09.002
  Preconditions PC-2 states "Kitty keyboard enhancement flags + mouse capture enabled" — the
  "globally-active mouse capture" claim in PC-2 is a minor imprecision (mouse capture is scoped to
  EmbeddedTerminal per BC-2.09.003 Invariant 1 / I3 fix), but this was corrected in BC-2.09.003
  v1.2.0; BC-2.09.002 PC-2's "globally-active" language refers to Kitty flags (which ARE global)
  and the juxtaposed mouse capture — it does not claim a different tracking mode. No behavioral
  change needed in BC-2.09.001 or BC-2.09.002 for F-P36-IMP-001.
- Minor bump: 1.4.0 → 1.5.0 (minor: normative Invariant 3 rewritten; stale any-event claim removed).

## §Trace v1.4.0

**S35-003 + arch-source pin v1.5.1→v1.5.2 — complete mouse Ps enumeration matching SS-embedded-pty v1.6.0** <!-- version-pin-historical: SS-embedded-pty v1.6.0 was canonical at §Trace v1.4.0 authoring time (2026-06-13 / D-277); superseded by v1.7.0 at Phase-2 Pass-2 fix burst --> (2026-06-13 / D-277):
- S35-003: PC-2 and Invariant 4 previously carried the incomplete Ps enumeration
  {0,1,2,64,65} (Down left/middle/right and ScrollUp/ScrollDown only). This missed:
  Up variants (release, terminator `m`), Drag variants (32/33/34 — button+32 motion bit),
  Moved (35 — 3+32, UNREACHABLE on Unix because crossterm enables 1002 button-event tracking
  not 1003 any-event; retained for match-exhaustiveness + Windows correctness),
  ScrollLeft (66), ScrollRight (67), and the modifier-bit additive rule (Shift|=4, Alt|=8,
  Ctrl|=16).
  - **PC-2:** replaced prose with the authoritative 14-row base-Ps table (12 base values +
    Moved-unreachable note) plus the modifier-bit additive rule and the terminator mapping
    (Up variants → `m`; all others → `M`). Source: SS-embedded-pty.md v1.7.0
    §mouse_event_to_pty_bytes exhaustive match.
  - **Invariant 4:** replaced "ScrollUp=64/ScrollDown=65" partial statement with the complete
    base-Ps summary covering all 12 base values + modifier rule + SS-embedded-pty citation.
    Explicit note that the prior {0,1,2,64,65} was incomplete.
  - No coordinate convention, boundary detection (EC-221/PC-5), or test vectors changed.
    The coordinate formula and `None`-on-out-of-pane behavior are unchanged.
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (all active citations).
- Minor bump: 1.3.0 → 1.4.0 (minor: PC-2 table extended; Invariant 4 corrected; normative Ps set enlarged).

## §Trace v1.3.0

**I23-001 + I23-002 + S23-001 — Coordinate convention added; test vectors corrected for +1 indexing** (2026-06-13):
- Finding I23-002: Canonical Test Vectors at lines ~97-99 all omitted the +1 offset required
  by the canonical formula `px = col - pane_area.x + 1`, `py = row - pane_area.y + 1`
  (SS-embedded-pty.md lines 511-512). The test vectors contradicted EC-220 (which correctly
  states (row=0, col=0) → Px=1, Py=1) and PC-2 (which states Px/Py are 1-indexed).
  Three examples were wrong:
    - `(row=3, col=5)` press: `\x1b[<0;5;3M` → `\x1b[<0;6;4M` (Px=6, Py=4)
    - `(row=3, col=5)` release: `\x1b[<0;5;3m` → `\x1b[<0;6;4m` (Px=6, Py=4)
    - `(row=10, col=20)` scroll up: `\x1b[<64;20;10M` → `\x1b[<64;21;11M` (Px=21, Py=11)
- Finding S23-001 root-cause: no explicit convention statement caused readers to assume
  crossterm's 0-indexed coordinates mapped directly to SGR output without the +1.
- Fix: Added §Coordinate Convention section (authoritative for the entire SS-09 family)
  stating the pane-origin assumption, 0-indexed crossterm input, 1-indexed SGR output,
  and canonical formula. Cross-references HS-EXP-015 step 15-16 as the authoritative example.
- Test vectors: all three coordinate-bearing rows corrected to apply the +1 convention with
  inline derivation annotations (Px=col+1=N, Py=row+1=N) for implementer clarity.
- EC-220 verified CORRECT (already states Px=1, Py=1 for row=0, col=0 — no change needed).
- PC-2 verified CORRECT (already states Px/Py are 1-indexed — no change needed).
- HS-EXP-015 verified CORRECT (step 15-16: col=10+1=11, row=5+1=6 → `\x1b[<0;11;6M` — no change needed).
- Version bump: 1.2.0 → 1.3.0 (minor: new normative section + corrected test vectors).

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
