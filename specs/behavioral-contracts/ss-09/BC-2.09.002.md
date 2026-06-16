---
document_type: behavioral-contract
level: L3
version: "1.1.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "cbbd04a"
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

# Behavioral Contract BC-2.09.002: Full-Fidelity Keyboard Forwarding — All v1A Input Classes Reach PTY stdin

## Description

When `AppMode::EmbeddedTerminal` is active, ALL keyboard events (except the Esc-to-exit
and monocle-level shortcuts) are translated to terminal byte sequences and forwarded to the
session's PTY stdin via `ClientToServer::KeyInput` IPC messages. v1A scope includes:
printable characters, Ctrl-modified keys, Enter/Backspace/Tab/Esc, arrow keys, function keys,
navigation keys (Home/End/PgUp/PgDn/Ins/Del), Kitty keyboard protocol enhanced sequences,
mouse events in SGR encoding, and bracketed paste. No keyboard class is deferred to v1B.

## Preconditions

1. `AppMode::EmbeddedTerminal { session_id }` is active.
2. The TUI has raw mode + Kitty keyboard enhancement flags enabled globally. Mouse capture is
   enabled on `AppMode::EmbeddedTerminal` entry (scoped) — it is NOT globally active. (I3
   design: SS-embedded-pty.md §"Crossterm setup (I3 fix)"; CC-GLOBAL-MOUSE-CAPTURE gate.)
3. A session with `SessionState::Running` is focused.
4. The session-host is connected and `KeyInput` messages are being proxied to the PTY.

## Postconditions

1. For each crossterm `KeyEvent` with `kind == KeyEventKind::Press` or `KeyEventKind::Repeat`:
   a. `key_event_to_pty_bytes(event)` returns `Some(bytes)` for all key classes below.
   b. `ClientToServer::KeyInput { session_id, bytes }` is sent over the IPC channel.
   c. The daemon forwards via `SessionManager::send_key_input()` → `DaemonToHost::KeyInput` →
      session-host writes `bytes` to PTY stdin.
2. Key class translation table (authoritative):

   | Key / Modifier | PTY bytes |
   |----------------|-----------|
   | Printable char (no modifier) | UTF-8 bytes of the character |
   | Ctrl+A through Ctrl+Z | `\x01` through `\x1a` (control characters) |
   | Ctrl+@ | `\x00` (NUL) |
   | Ctrl+[ | `\x1b` (Esc) |
   | Enter | `\r` |
   | Backspace | `\x7f` (DEL) |
   | Tab | `\t` |
   | Esc (forwarded to PTY; see EC-210) | `\x1b` |
   | Arrow Up | `\x1b[A` |
   | Arrow Down | `\x1b[B` |
   | Arrow Right | `\x1b[C` |
   | Arrow Left | `\x1b[D` |
   | Home | `\x1b[H` |
   | End | `\x1b[F` |
   | Page Up | `\x1b[5~` |
   | Page Down | `\x1b[6~` |
   | Insert | `\x1b[2~` |
   | Delete | `\x1b[3~` |
   | F1 | `\x1bOP` |
   | F2 | `\x1bOQ` |
   | F3 | `\x1bOR` |
   | F4 | `\x1bOS` |
   | F5 | `\x1b[15~` |
   | F6 | `\x1b[17~` |
   | F7 | `\x1b[18~` |
   | F8 | `\x1b[19~` |
   | F9 | `\x1b[20~` |
   | F10 | `\x1b[21~` |
   | F11 | `\x1b[23~` |
   | F12 | `\x1b[24~` |
   | Kitty-enhanced key (modifier combo) | CSI u sequence per Kitty keyboard protocol spec |
   | Mouse event (in EmbeddedTerminal) | SGR sequence: `\x1b[<Ps;Px;Py M` or `m` (Px = col+1, Py = row+1 at pane origin; see BC-2.09.003 §Coordinate Convention) |
   | Bracketed paste text | `\x1b[200~<text>\x1b[201~` |

3. `KeyEventKind::Release` events are DISCARDED — they are NOT forwarded to the PTY.
4. Pure modifier key events (Shift alone, Ctrl alone, Alt alone) return `None` from
   `key_event_to_pty_bytes()` and are NOT forwarded.

## Invariants

1. `key_event_to_pty_bytes()` is a PURE function — no I/O, no state mutation. It is
   deterministic given a `KeyEvent` value.
2. The Action dispatch layer MUST intercept `KeyCode::Esc` (no modifiers) as
   `Action::ExitEmbeddedTerminal` BEFORE `key_event_to_pty_bytes()` is called. Esc exits
   embedded terminal mode; a second Esc (after re-entering) would be forwarded as `\x1b`.
3. `Ctrl-D` (`KeyCode::Char('d')` + `KeyModifiers::CONTROL`) is translated to `\x04` (EOT)
   and forwarded to the PTY. Claude Code interprets this as "end session."
4. Kitty keyboard enhancement flags are enabled globally on TUI startup. If the terminal
   does not support Kitty protocol, the flags silently no-op and standard sequences are used.
5. On entry to `AppMode::EmbeddedTerminal`, `EnableMouseCapture` is issued to the terminal
   (scoped — NOT globally active) AND SGR extended mouse mode (`ESC [ ? 1006 h`) is written
   immediately after. On exit from `AppMode::EmbeddedTerminal`, SGR mode is disabled
   (`ESC [ ? 1006 l`) and then `DisableMouseCapture` is issued. This scoped entry/exit
   pattern is the ratified I3 design (SS-embedded-pty.md §"Crossterm setup (I3 fix)",
   lines 254-258 / §"EmbeddedTerminal EXIT", lines 349-360). Global mouse capture is an
   unadopted option gated behind CC-GLOBAL-MOUSE-CAPTURE human sign-off. See also
   BC-2.09.003 Invariant 1 (sibling authoritative statement of this same contract).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-210 | Esc pressed in EmbeddedTerminal mode | Intercepted as `Action::ExitEmbeddedTerminal` by dispatch layer; NOT forwarded to PTY via key_event_to_pty_bytes() |
| EC-211 | Esc pressed TWICE in EmbeddedTerminal mode | First Esc exits EmbeddedTerminal; second Esc is processed in the restored prior AppMode (not in EmbeddedTerminal) — so second Esc is NOT forwarded to PTY via this path; it is processed by the prior AppMode's keybinding table |
| EC-212 | `Ctrl-D` in EmbeddedTerminal | Forwarded as `\x04`; Claude Code session ends; session-host sends `StateChanged::Terminated`; TUI transitions out of `AppMode::EmbeddedTerminal` automatically |
| EC-213 | Mouse click at crossterm (row=5, col=10) in EmbeddedTerminal (pane at terminal origin, pane_area.x=0, pane_area.y=0) | SGR sequence `\x1b[<0;11;6M` (button 0, Px=col+1=11, Py=row+1=6, press) sent as `KeyInput` bytes. Matches HS-EXP-015 step 15-16. |
| EC-214 | Paste of 500-byte text via bracketed paste | `\x1b[200~` + 500 bytes + `\x1b[201~` forwarded as single `KeyInput` message |
| EC-215 | `KeyEventKind::Release` for any key | `None` returned; NOT forwarded; 0 bytes sent |
| EC-216 | Kitty protocol unsupported by terminal (flags silently ignored) | `is_kitty_enhanced_key()` returns false for modifier combos; standard VT sequences used for regular keys; no panic; no silent key loss |

## Canonical Test Vectors

| Input | Expected PTY bytes | Category |
|-------|-------------------|----------|
| `KeyEvent { code: Char('a'), modifiers: NONE, kind: Press }` | `[0x61]` (`a`) | happy-path |
| `KeyEvent { code: Char('c'), modifiers: CONTROL, kind: Press }` | `[0x03]` (ETX) | happy-path |
| `KeyEvent { code: Up, modifiers: NONE, kind: Press }` | `[0x1b, 0x5b, 0x41]` (`ESC[A`) | happy-path |
| `KeyEvent { code: Enter, modifiers: NONE, kind: Press }` | `[0x0d]` (`\r`) | happy-path |
| `KeyEvent { code: Char('a'), kind: Release }` | `None` — not forwarded | happy-path |
| `KeyEvent { code: Char('d'), modifiers: CONTROL, kind: Press }` | `[0x04]` (EOT) | happy-path |
| Bracketed paste of "hello" | `\x1b[200~hello\x1b[201~` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `key_event_to_pty_bytes()` unit tests covering all key classes in the table | unit |
| VP-TBD | Release events return `None` | unit |
| VP-TBD | Ctrl+[A-Z] produce control characters `\x01`-`\x1a` | unit |
| VP-TBD | Bracketed paste wrapped in `\x1b[200~...\x1b[201~` | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — full-fidelity keyboard forwarding is explicitly named in CAP-009, and this BC defines the complete key translation table and forwarding contract |
| Architecture Module | monocle-core (`key_event_to_pty_bytes()` pure function); monocle-tui (Action dispatch, IPC KeyInput send) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md v1.7.0 §Full-Fidelity Keyboard Encoding; §Translation function; §Esc key handling contract; §Bracketed paste; §Mouse support |
| Test Name | test_BC_2_09_002_keyboard_forwarding_all_classes |

## Related BCs

- [BC-2.09.003] — composes with: mouse events are a sub-class of keyboard input forwarding
- [BC-2.09.004] — composes with: Kitty keyboard protocol is a sub-class of keyboard forwarding
- [BC-2.09.005] — composes with: bracketed paste is a sub-class of keyboard forwarding

## Architecture Anchors

- `architecture/SS-embedded-pty.md#full-fidelity-keyboard-encoding` — complete key class specification
- `architecture/SS-embedded-pty.md#esc-key-handling-contract` — Esc intercept before PTY forwarding

## Story Anchor

S-040 — Implement key_event_to_pty_bytes() and KeyInput IPC send in monocle-tui

## VP Anchors

VP-TBD — Keyboard translation unit tests (filled after VP creation)

## §Trace v1.1.3

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-040** (2026-06-15):
- Story Anchor filled from Phase-2 Burst B story decomposition. No behavioral content changed.

## §Trace v1.1.2

**F-P37-IMP-001 — Normative invariant corrected: scoped mouse capture replaces erroneous "globally-active" claim** (2026-06-13 / D-279):
- Finding: Invariant 5 (~line 106) stated SGR 1006 mode is written "in addition to the
  globally-active mouse capture." This contradicts the ratified I3 design: mouse capture is
  SCOPED to EmbeddedTerminal entry/exit, NOT globally active. Global mouse capture is an
  unadopted option gated behind CC-GLOBAL-MOUSE-CAPTURE human sign-off.
  Authority: SS-embedded-pty.md §"Crossterm setup (I3 fix)" lines 254-258 / §"EmbeddedTerminal
  EXIT" lines 349-360; BC-2.09.003 Invariant 1 (sibling authoritative statement).
- Precondition 2: tightened from "mouse capture enabled" (imprecise, implied global) to
  explicitly state mouse capture is enabled on EmbeddedTerminal entry (scoped), consistent
  with I3 design.
- Invariant 5: rewritten to describe the full scoped entry/exit cycle (EnableMouseCapture +
  SGR 1006 on entry; SGR 1006 disable + DisableMouseCapture on exit). Removes "globally-active"
  language entirely. Adds CC-GLOBAL-MOUSE-CAPTURE gate reference and BC-2.09.003 cross-reference.
- Whole-class sweep: BC-2.09.002 had exactly two "globally*" mentions — line 103 (Kitty flags,
  which ARE globally enabled — correctly left unchanged) and line 106 (mouse capture — the
  erroneous survivor, now corrected). No other global-capture phrasing survived.
  BC-2.09.003 was NOT touched (its §Trace dismissal is historical record per task instruction).
- This is a sibling-gap fix: Pass-36 §Trace in BC-2.09.003 had dismissed the I2-001/I3
  scoped-capture gap; this pass corrects the surviving normative error in BC-2.09.002.
- Version bump: 1.1.1 → 1.1.2 (patch: normative invariant corrected to ratified design).

## §Trace v1.1.1

**Arch-source pin v1.5.1→v1.5.2** (2026-06-13 / D-277):
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.1.0

**I23-001 — EC-213 mouse coordinate off-by-one corrected; SGR convention cross-reference added** (2026-06-13):
- Finding: EC-213 claimed `\x1b[<0;10;5M` for crossterm `(row=5, col=10)`. The canonical
  formula is `px = col - pane_area.x + 1`, `py = row - pane_area.y + 1` (SS-embedded-pty.md
  lines 511-512). At pane origin (pane_area.x=0, pane_area.y=0): px = col+1 = 11, py = row+1 = 6.
  Correct output is `\x1b[<0;11;6M`. This matches HS-EXP-015 step 15-16 exactly (the holdout's
  reference was CORRECT; the EC was wrong).
- EC-213: corrected byte sequence from `\x1b[<0;10;5M` → `\x1b[<0;11;6M`; added pane-origin
  assumption annotation and cross-reference to HS-EXP-015.
- PC-2 mouse row: added parenthetical cross-reference to BC-2.09.003 §Coordinate Convention
  to make the +1 indexing rule discoverable from this BC.
- S23-001 (SUGGESTION root-cause): explicit convention cross-reference added to the key
  translation table so readers of this BC encounter the coordinate-encoding rule directly.
- Version bump: 1.0.1 → 1.1.0 (minor: normative EC corrected; behavioral example was wrong).

## §Trace v1.0.1

**S-P7-002 — Remove stray `removal_range: null` frontmatter field** (2026-06-03):
- Removed spurious `removal_range: null` field that appeared in no sibling BC.
  Correct lifecycle fields are `removed: null` + `removal_reason: null`; this
  stray line was a copy-paste artifact. No content change.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.002 authored for SS-09 as part of the v1A control-center pivot BC burst.
- Key translation table authored verbatim from SS-embedded-pty.md §Translation function.
  D-237 ratification: full keyboard fidelity is IN v1A scope (all key classes; no deferral).
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
