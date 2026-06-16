---
scenario_id: HS-EXP-015
title: "Full-Fidelity Keyboard Forwarding — Kitty + SGR Mouse + Bracketed Paste Reach PTY stdin in EmbeddedTerminal"
wave: 8
stories_tested: [S-040, S-041]
source_bcs: [BC-2.09.002, BC-2.09.003, BC-2.09.004, BC-2.09.005]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:45:00Z
---

# HS-EXP-015: Full-Fidelity Keyboard Forwarding — Kitty + SGR Mouse + Bracketed Paste Reach PTY stdin in EmbeddedTerminal

**Wave:** 8
**Source BC:** BC-2.09.002 (PC-1..PC-5: input class forwarding), BC-2.09.003 (PC-1: SGR mouse), BC-2.09.004 (PC-1: Kitty CSI u), BC-2.09.005 (PC-1: bracketed paste)
**Stories Tested:** S-040, S-041

## Setup

A `ratatui::backend::TestBackend` with the TUI in `AppMode::EmbeddedTerminal { session_id: S1 }`.
A mock `monocle-session-host` that exposes a read-end of the PTY stdin pipe. All bytes written
to the PTY stdin via the IPC `KeyInput` message are captured by the mock and returned as a byte
buffer for inspection. Note: mouse events are NOT transported as a separate `MouseInput` IPC
variant — there is no such variant. Mouse events are SGR-encoded by `mouse_event_to_pty_bytes()`
(per SS-embedded-pty SS-09 §Mouse support) and forwarded as `KeyInput { bytes: <SGR sequence> }`
(see Part D below).

## Steps

### Part A: Printable characters and control keys

1. Inject `KeyEvent { code: KeyCode::Char('a'), modifiers: NONE }` into the TUI event loop.
2. Verify: `KeyInput { session_id: S1, bytes: b"a" }` sent via IPC to session-host within 50ms.
3. Inject `KeyEvent { code: KeyCode::Enter, modifiers: NONE }`.
4. Verify: IPC delivers `b"\r"` (carriage return, the PTY canonical form for Enter).
5. Inject `KeyEvent { code: KeyCode::Char('c'), modifiers: CTRL }`.
6. Verify: IPC delivers `b"\x03"` (ETX / Ctrl-C).

### Part B: Arrow keys (ANSI escape sequences)

7. Inject `KeyEvent { code: KeyCode::Up, modifiers: NONE }`.
8. Verify: IPC delivers `b"\x1b[A"` (CSI A — cursor up).
9. Inject `KeyEvent { code: KeyCode::Left, modifiers: NONE }`.
10. Verify: IPC delivers `b"\x1b[D"` (CSI D — cursor left).

### Part C: Kitty keyboard protocol (CSI u enhanced key events)

11. Inject `KeyEvent { code: KeyCode::Char('a'), modifiers: SHIFT }` with Kitty encoding enabled.
12. Verify: IPC delivers `b"\x1b[97;2u"` (CSI u encoding: code=97 'a', modifier=2 shift).
13. Inject `KeyEvent { code: KeyCode::F(1), modifiers: NONE }` with Kitty encoding.
14. Verify: IPC delivers the Kitty-encoded F1 sequence per the Kitty keyboard specification.

### Part D: SGR mouse events

15. Inject a `MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: 10, row: 5, modifiers: NONE }`.
16. Verify: IPC delivers `b"\x1b[<0;11;6M"` (SGR 1-based coordinates: col=10+1=11, row=5+1=6, button 0=left, press M).

### Part E: Bracketed paste

17. Inject a `Paste("hello world")` event (or equivalent paste event from ratatui).
18. Verify: IPC delivers `b"\x1b[200~hello world\x1b[201~"` (bracketed paste: start marker + text + end marker).
19. Verify: if the input "hello world" contains a newline, the newline is NOT translated — it is
    forwarded as-is, embedded between the bracket markers.

## Expected Outcome

- All 6 input classes (printable, control, arrows, Kitty, SGR mouse, bracketed paste) result in
  byte sequences delivered to session-host PTY stdin within 50ms of the TUI event injection.
- Byte sequences match the canonical encoding per BC-2.09.002 Table of Input Classes.
- Forwarding does not corrupt other PTY output arriving concurrently (no interleaving corruption).

## Adversarial Probe

Inject 100 rapid keystrokes in 100ms (10ms intervals) and verify: all 100 `KeyInput` IPC messages
are delivered in order, with no bytes dropped or reordered. Use a sequence number embedded in the
input (e.g., chars '0'..'9' repeating) and verify the output matches the input sequence exactly.

## Satisfaction Criteria

PASS: All 6 input classes produce correct byte sequences within 50ms; bracketed paste includes
correct markers; SGR mouse uses 1-based coordinates; Kitty protocol uses CSI u encoding; adversarial
100-keystroke flood delivers all bytes in order without drops.

FAIL: Any input class produces wrong byte sequences or no bytes; bracketed paste omits start/end
markers or translates newlines inside the paste; SGR mouse uses 0-based coordinates; Kitty encoding
absent for enhanced keys; adversarial flood drops any byte or delivers bytes out of order.

**NOT in any story AC:** The story implementing BC-2.09.002 will have ACs for individual input
class forwarding. The stories implementing BC-2.09.003/004/005 will have ACs for SGR mouse, Kitty,
and bracketed paste individually. This holdout tests **all six input classes in sequence within a
single running EmbeddedTerminal session**, with a concurrent adversarial flood. The interaction
between the six forwarding paths (particularly when Kitty encoding is enabled but SGR mouse events
also arrive) is not exercised by any single BC-specific AC, and the adversarial 100-keystroke flood
tests the IPC channel's bounded ordering guarantee under load.

---

## §Trace

### v1.2 — Pass-8 S-P8-001: Normalize input-class count to 6 (2026-06-03)

- **S-P8-001:** Expected Outcome (line 68) said "All 5 input classes" while parenthetically
  enumerating six distinct classes (printable, control, arrows, Kitty, SGR mouse, bracketed paste).
  Satisfaction Criteria PASS (line 81) already correctly stated "All 6 input classes". The count
  discrepancy was cosmetic — the enumerated set of six has always been six.
- **Fix:** "All 5" → "All 6" in Expected Outcome. "all five input classes" → "all six input
  classes" in the NOT-in-any-story-AC paragraph (two occurrences). Satisfaction Criteria line 81
  confirmed correct at "All 6" — no change needed.
- EVAL-INDEX bumped 1.13 → 1.14 for auditability.

### v1.1 — Pass-6 I6-001: Remove phantom MouseInput IPC variant (2026-06-03T23:45:00Z)

- **I6-001:** Setup (original line 23) described bytes delivered "via the IPC `KeyInput` or
  `MouseInput` messages." There is NO `MouseInput` variant in `ClientToServer` — it was never
  defined. Mouse events are SGR-encoded by `mouse_event_to_pty_bytes()` per
  SS-embedded-pty SS-09 §Mouse support and forwarded as `ClientToServer::KeyInput { bytes: <SGR sequence> }`.
  Part D (Step 15-16) already correctly expected the bytes to arrive via `KeyInput`; only the
  Setup was inconsistent.
- **Fix:** Setup rewritten to reference `KeyInput` only, with an inline note explaining that
  mouse events are SGR-encoded and carried as `KeyInput` (no separate `MouseInput` variant exists).
  Part D unchanged — it was already correct.
- Frontmatter `timestamp` updated to 2026-06-03T23:45:00Z.

### v1.0 — Initial (2026-06-03T12:00:00Z)

Created as part of D-241 control-center v1A holdout burst — Wave 8 SS-09 embedded PTY
keyboard-forwarding coverage. See EVAL-INDEX v1.10 §Trace.
