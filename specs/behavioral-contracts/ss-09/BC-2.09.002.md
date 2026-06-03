---
document_type: behavioral-contract
level: L3
version: "1.0.0"
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
removal_range: null
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
2. The TUI has raw mode + Kitty keyboard enhancement flags + mouse capture enabled.
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
   | Mouse event (in EmbeddedTerminal) | SGR sequence: `\x1b[<Ps;Px;Py M` or `m` |
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
5. SGR mouse mode (`ESC [ ? 1006 h`) is written to the terminal when entering
   `AppMode::EmbeddedTerminal`, in addition to the globally-active mouse capture.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-210 | Esc pressed in EmbeddedTerminal mode | Intercepted as `Action::ExitEmbeddedTerminal` by dispatch layer; NOT forwarded to PTY via key_event_to_pty_bytes() |
| EC-211 | Esc pressed TWICE in EmbeddedTerminal mode | First Esc exits EmbeddedTerminal; second Esc is processed in the restored prior AppMode (not in EmbeddedTerminal) — so second Esc is NOT forwarded to PTY via this path; it is processed by the prior AppMode's keybinding table |
| EC-212 | `Ctrl-D` in EmbeddedTerminal | Forwarded as `\x04`; Claude Code session ends; session-host sends `StateChanged::Terminated`; TUI transitions out of `AppMode::EmbeddedTerminal` automatically |
| EC-213 | Mouse click at (row=5, col=10) in EmbeddedTerminal | SGR sequence `\x1b[<0;10;5M` (button 0, col 10, row 5, press) sent as `KeyInput` bytes |
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
| Architecture Source | SS-embedded-pty.md v1.1.0 §Full-Fidelity Keyboard Encoding; §Translation function; §Esc key handling contract; §Bracketed paste; §Mouse support |
| Test Name | test_BC_2_09_002_keyboard_forwarding_all_classes |

## Related BCs

- [BC-2.09.003] — composes with: mouse events are a sub-class of keyboard input forwarding
- [BC-2.09.004] — composes with: Kitty keyboard protocol is a sub-class of keyboard forwarding
- [BC-2.09.005] — composes with: bracketed paste is a sub-class of keyboard forwarding

## Architecture Anchors

- `architecture/SS-embedded-pty.md#full-fidelity-keyboard-encoding` — complete key class specification
- `architecture/SS-embedded-pty.md#esc-key-handling-contract` — Esc intercept before PTY forwarding

## Story Anchor

S-TBD — Implement key_event_to_pty_bytes() and KeyInput IPC send in monocle-tui (filled by story-writer)

## VP Anchors

VP-TBD — Keyboard translation unit tests (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.002 authored for SS-09 as part of the v1A control-center pivot BC burst.
- Key translation table authored verbatim from SS-embedded-pty.md §Translation function.
  D-237 ratification: full keyboard fidelity is IN v1A scope (all key classes; no deferral).
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
