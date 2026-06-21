---
story_id: S-040
title: Full-Fidelity Keyboard Forwarding
version: "1.0"
produced_by: vsdd-factory:demo-recorder
date: 2026-06-21
---

# S-040 Demo Evidence Report

## Story Summary

S-040 implements the TUI-side keyboard pipeline for monocle:
pure-core key-to-bytes translation (`key_event_to_pty_bytes`, `encode_kitty_key`,
`is_kitty_enhanced_key`), the `EmbeddedTerminal` event dispatch arm, and the
`ClientToServer::KeyInput` IPC send path. The session-host PTY-stdin write (the
part that makes a typed key reach Claude Code's process) is S-047 scope and is NOT
yet implemented.

## S-047 Dependency

A full keystroke→PTY round-trip ("type a key, Claude responds") requires the
session-host `KeyInput`→`PTY::write_stdin` path, which lands in S-047. The demos
below record the legitimate evidence boundary for S-040: the TUI-side translation
and dispatch pipeline, verified through the full test suite.

## Coverage Map

| Recording | Acceptance Criteria | BCs Covered | Tests |
|-----------|--------------------|-----------:|------:|
| AC-001-keyboard-unit-tests.webm | AC-001, AC-002, AC-003, AC-005, AC-006, AC-007, AC-014 | BC-2.09.002, BC-2.09.004 | 41 |
| AC-002-keyboard-dispatch-wiring.webm | AC-004, AC-009, AC-010, AC-011, AC-012, AC-013, AC-015 | BC-2.09.002, BC-2.09.004, BC-2.09.005 | 24 |

**Total: 65 tests passing, 0 failures.**

## Recording Details

### AC-001 — Pure-Core Keyboard Translation Unit Tests

**File:** `AC-001-keyboard-unit-tests.webm`
**Tape:** `AC-001-keyboard-unit-tests.tape`

Runs `cargo test -p monocle-core --lib keyboard` — the 41 unit tests for:

- `key_event_to_pty_bytes()` full translation table (printable chars, Ctrl+[A-Z],
  Enter=`\r`, Backspace=`\x7f`, arrows, F1–F12, Home/End/PgUp/PgDn/Ins/Del,
  Alt+char ESC-prefix, Shift+Tab `\x1b[Z`, Ctrl+Arrow VT fallbacks)
- `PtyKeyEventKind::Release` → `None` (zero bytes forwarded)
- `encode_kitty_key()` CSI-u sequences: `Ctrl+Shift+Enter` → `\x1b[13;6u`,
  `Ctrl+Up` → `\x1b[57352;5u` (functional-key codepoints)
- `is_kitty_enhanced_key()` — returns `false` when `kitty_active=false`,
  `true` for modifier-carrying combos when `kitty_active=true`
- `_ if !mods.is_empty()` TRACE+None arm for non-Kitty modifier combos (EC-217)
- Exact-equality modifier guards on arrow VT fallbacks (ADV-HIGH-001)
- Ctrl+@ → `\x00` (NUL), Ctrl+[ → `\x1b` (ESC)

Tests directly exercising AC-001, AC-002, AC-003, AC-005, AC-006, AC-007, AC-014.

### AC-002 — Keyboard Dispatch and Bracketed Paste Wiring Tests

**File:** `AC-002-keyboard-dispatch-wiring.webm`
**Tape:** `AC-002-keyboard-dispatch-wiring.tape`

Runs `cargo test -p monocle-tui --test bc_2_09_keyboard_tests --test bc_2_09_wiring_tests`:

- `bc_2_09_keyboard_tests` (12 tests) — dispatch-layer tests via
  `dispatch_embedded_terminal_key`:
  - Esc Press-only intercept → `Action::ExitEmbeddedTerminal`, no bytes forwarded (AC-011)
  - Esc Release → NOT an exit, zero bytes (ADV-MED-001 / AC-013)
  - Esc Repeat → forwards `\x1b`, no exit
  - Non-Esc key → bytes forwarded, no exit
  - Release events → not forwarded (AC-013)
  - Bracketed paste wrap: `\x1b[200~` + text + `\x1b[201~` (AC-009)
  - Bracketed paste empty string (AC-009)
  - Large paste (500 bytes) single KeyInput, no fragmentation (AC-012)
  - Paste newlines preserved verbatim
  - Paste containing ESC characters forwarded verbatim (EC-230)
  - Paste containing `\x1b[200~` forwarded verbatim inside outer brackets (EC-231)
  - `session_id` correct in KeyInput message

- `bc_2_09_wiring_tests` (12 tests) — IPC send path via `handle_crossterm_event`:
  - EmbeddedTerminal key routes to IPC KeyInput send
  - Esc exits embedded terminal, zero KeyInput sent
  - Esc intercept precedes `key_event_to_pty_bytes` call
  - Bracketed paste → KeyInput with brackets
  - Large paste single KeyInput via `handle_crossterm_event`
  - Kitty `kitty_active=true` → Ctrl+Shift+Enter produces `\x1b[13;6u`
  - Non-Kitty `kitty_active=false` → Ctrl+Shift+Enter no send (AC-007/AC-014)
  - Oversized paste guard (EC-245 / ADV-HIGH-002): framed payload > `MAX_MESSAGE_BYTES` → WARN + DROP
  - JSON expansion guard for paste ceiling
  - Small paste passes guard normally
  - Paste outside EmbeddedTerminal ignored
  - Quit key in dashboard returns error (non-keyboard-forwarding path verification)

Tests directly exercising AC-004, AC-009, AC-010, AC-011, AC-012, AC-013, AC-015.

## Format Notes

- Output format: WEBM only (no GIF — project policy for S-040).
- Font: `FiraCode Nerd Font Mono` (detected on this machine via `fc-list`).
- VHS version: 0.11.0.

## What Is Not Demonstrated

The full keystroke→PTY round-trip (type key → Claude Code responds) requires
S-047 (`session-host KeyInput→PTY-stdin write`). That path does not exist yet.
These recordings are the correct and honest evidence boundary for S-040: the
TUI-side translation and dispatch pipeline is fully implemented and verified by
65 passing tests.
