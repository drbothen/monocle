---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md]
input-hash: "cc4aaa1"
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

# Behavioral Contract BC-2.09.005: Bracketed Paste — Paste Events Wrapped in Bracket Sequences Before Forwarding

## Description

When `AppMode::EmbeddedTerminal` is active and the user pastes text (via terminal paste
shortcut or tmux buffer paste), the paste event arrives as
`crossterm::event::Event::Paste(String)`. The paste text is wrapped in bracketed paste
sequences (`ESC[200~` + text + `ESC[201~`) before forwarding to the PTY via `KeyInput`.
This enables paste-aware programs (vim, nano, shell readline) to distinguish pasted text
from typed keystrokes and disable auto-indentation/escaping during paste.

## Preconditions

1. `AppMode::EmbeddedTerminal` is active.
2. `crossterm::event::EnableBracketedPaste` is globally active (enabled at TUI startup
   alongside keyboard enhancement flags).
3. The user initiates a paste operation (OS paste shortcut, tmux paste, etc.).

## Postconditions

1. `crossterm::event::Event::Paste(text)` is received by the TUI event loop.
2. The paste text is prepended with `\x1b[200~` and appended with `\x1b[201~`.
3. The complete bracketed sequence is sent as `ClientToServer::KeyInput { session_id, bytes }`.
4. The bytes are forwarded to the PTY stdin via `DaemonToHost::KeyInput`.
5. Programs in the PTY that support bracketed paste mode handle the paste correctly
   (e.g., vim pastes without auto-indentation, shell readline disables completion).
6. Programs that do NOT support bracketed paste receive the raw text with the bracket
   sequences as literal characters. This is the standard behavior for programs that have not
   enabled bracketed paste mode (`ESC[?2004h`).

## Invariants

1. `EnableBracketedPaste` is enabled globally at TUI startup (not gated on EmbeddedTerminal
   entry). This ensures paste events are available immediately.
2. Paste text MUST NOT be processed through `key_event_to_pty_bytes()`. Paste events arrive
   as `Event::Paste(text)`, NOT as `Event::Key(KeyEvent)`. The dispatch logic MUST check
   for `Event::Paste` as a separate branch.
3. Large paste operations (>64 KiB) MUST be handled without timeout. The `KeyInput` IPC
   message carries the full bracketed payload as a single `Vec<u8>`. No chunking is required
   for v1A (the IPC framing supports up to 256 KiB).
4. `EnableBracketedPaste` is disabled at TUI exit via `DisableBracketedPaste`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-230 | Paste text contains ESC characters (e.g., pasting ANSI-colored text) | ESC characters are included verbatim in the bracketed payload; the PTY receives them as data, not as escape sequences; the harness TUI may or may not handle this correctly |
| EC-231 | Paste text contains embedded `\x1b[200~` sequence (attacker input) | Forwarded verbatim; monocle does not sanitize paste text; the outer bracketed sequence is still present; security concern is mitigated by the fact that the PTY is a user-controlled terminal |
| EC-232 | Empty paste (zero-length text) | `\x1b[200~\x1b[201~` forwarded — the bracket sequences with no content between them |
| EC-233 | Paste in Dashboard mode (not in EmbeddedTerminal) | `Event::Paste` is processed by Dashboard handlers (typically as a no-op or search input); NOT sent to PTY |

## Canonical Test Vectors

| Input | Expected PTY bytes | Category |
|-------|-------------------|----------|
| `Event::Paste("hello world")` in EmbeddedTerminal | `\x1b[200~hello world\x1b[201~` | happy-path |
| `Event::Paste("")` (empty) | `\x1b[200~\x1b[201~` | edge-case |
| `Event::Paste("line1\nline2")` | `\x1b[200~line1\nline2\x1b[201~` (newlines preserved verbatim) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `Event::Paste(text)` → `\x1b[200~<text>\x1b[201~` in KeyInput bytes | unit |
| VP-TBD | Empty paste produces valid (but empty) bracketed sequence | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — full-fidelity keyboard forwarding in CAP-009 includes paste operations; bracketed paste is a standard terminal feature required for accurate paste handling in embedded terminal mode |
| Architecture Module | monocle-tui (EmbeddedTerminal event dispatch, paste handling) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md v1.0.2 §Bracketed paste |
| Test Name | test_BC_2_09_005_bracketed_paste_wrapped_in_bracket_sequences |

## Related BCs

- [BC-2.09.002] — composes with: paste is part of full-fidelity input forwarding; paste uses a separate Event::Paste branch, not key_event_to_pty_bytes()

## Architecture Anchors

- `architecture/SS-embedded-pty.md#bracketed-paste` — paste sequence format

## Story Anchor

S-TBD — Same story as BC-2.09.002 (paste handling in EmbeddedTerminal event dispatch; filled by story-writer)

## VP Anchors

VP-TBD — Bracketed paste unit tests (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.005 authored for SS-09 as part of the v1A control-center pivot BC burst.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
