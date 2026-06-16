---
document_type: story
level: L4
story_id: S-040
epic_id: EPIC-09
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 8
wave: 9
tdd_mode: strict
priority: P0
depends_on: [S-039]
blocks: [S-041, S-044]
target_module: monocle-core
subsystems: [SS-09]
behavioral_contracts: [BC-2.09.002, BC-2.09.004, BC-2.09.005]
verification_properties: []
estimated_days: 4
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.002.md, version: "1.1.4"}
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.004.md, version: "1.0.4"}
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.005.md, version: "1.0.3"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.7.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.09.002 (full-fidelity keyboard forwarding — all v1A input classes), BC-2.09.004 (Kitty keyboard protocol CSI u sequences), BC-2.09.005 (bracketed paste)"
# BC status at S-040 authoring time: BC-2.09.002 v1.1.3, BC-2.09.004 v1.0.3, BC-2.09.005 v1.0.2 — non-empty; status draft pending Phase-2 adversarial convergence gate
# Clustering rationale: BC-2.09.004 and BC-2.09.005 story anchors in their respective BCs both explicitly state "Same story as BC-2.09.002".
# key_event_to_pty_bytes(), is_kitty_enhanced_key(), encode_kitty_key(), and bracketed-paste dispatch live in the same pure-core module.
# All three BCs share the same monocle-core file, the same action dispatch arm, and the same KeyInput IPC send path.
---

# S-040: Full-Fidelity Keyboard Forwarding — key_event_to_pty_bytes, Kitty Protocol CSI u, and Bracketed Paste

## Narrative

As the monocle TUI in `AppMode::EmbeddedTerminal`, I want every keyboard event (printable
characters, control keys, arrows, function keys, Kitty-enhanced modifier combos, and bracketed
paste) to be translated to the canonical terminal byte sequence and forwarded to the PTY as a
`ClientToServer::KeyInput` IPC message — with no key class silently dropped or misencoded —
so that Claude Code in the embedded terminal responds exactly as if the user were in a
native terminal.

## Acceptance Criteria

### AC-001 (traces to BC-2.09.002 postcondition 1 — all v1A key classes translated and forwarded)

For each crossterm `KeyEvent` with `kind == KeyEventKind::Press` or `KeyEventKind::Repeat`,
while `AppMode::EmbeddedTerminal` is active:
- `key_event_to_pty_bytes(event)` returns `Some(bytes)`.
- `ClientToServer::KeyInput { session_id, bytes }` is sent over the IPC channel.
- The bytes match the authoritative BC-2.09.002 key translation table (printable chars, Ctrl+[A-Z],
  Enter=`\r`, Backspace=`\x7f`, Tab=`\t`, arrows as `\x1b[A`–`\x1b[D`, F1–F12, Home/End/PgUp/PgDn/Ins/Del,
  Alt+char as ESC-prefix, Shift+Tab=`\x1b[Z`, Ctrl+Arrow VT fallbacks per BC-2.09.002 PC-2 table).

### AC-002 (traces to BC-2.09.002 postcondition 3 — Release events discarded)

`KeyEventKind::Release` events return `None` from `key_event_to_pty_bytes()`. They are NOT
sent to the PTY. Zero bytes are forwarded for Release events.

### AC-003 (traces to BC-2.09.002 invariant 1 — key_event_to_pty_bytes is pure)

`key_event_to_pty_bytes(event: KeyEvent) -> Option<Vec<u8>>` is a pure function with no I/O
and no state mutation. It is deterministic: the same `KeyEvent` input always produces the same
output. It lives in `monocle-core` (pure core crate), not `monocle-tui`.

### AC-004 (traces to BC-2.09.002 invariant 2 — Esc intercepted before key_event_to_pty_bytes)

The action dispatch layer MUST intercept `KeyCode::Esc` (no modifiers) as
`Action::ExitEmbeddedTerminal` BEFORE calling `key_event_to_pty_bytes()`. Esc does NOT reach
`key_event_to_pty_bytes()` and is NOT forwarded to the PTY on the first press. A user
intending to send Esc to the PTY must press Esc twice.

### AC-005 (traces to BC-2.09.002 invariant 3 — Ctrl-D forwards as \x04; session end)

`KeyCode::Char('d')` with `KeyModifiers::CONTROL` is translated to `\x04` (ASCII EOT) and
forwarded to the PTY. Claude Code interprets this as session termination. The session-host
detects the child exit and sends `StateChanged::Terminated`.

### AC-006 (traces to BC-2.09.004 postcondition 1–3 — Kitty-enhanced keys → CSI u sequences)

For key events where `is_kitty_enhanced_key(event.code, mods)` returns `true` (on Kitty-capable
terminals), `encode_kitty_key(event.code, mods, event.kind)` produces a CSI u sequence:
`ESC [ <unicode_codepoint> ; <modifier_value> u`.
- `<modifier_value>` = 1 + sum of active modifier bits: Shift=1, Alt=2, Ctrl=4.
- Example: `Ctrl+Shift+Enter` → `\x1b[13;6u` (codepoint 13; modifier = 1 + 1(shift) + 4(ctrl) = 6).
- Example: `Shift+Tab` (Kitty path) → `\x1b[9;2u` (codepoint 9; modifier = 1 + 1(shift) = 2).

### AC-007 (traces to BC-2.09.004 invariant 2 — Kitty unsupported terminals use VT fallback)

When the terminal does not support Kitty keyboard protocol (`PushKeyboardEnhancementFlags`
silently no-ops), `is_kitty_enhanced_key()` returns `false` for all keys because Kitty-enhanced
`KeyEvent` variants are never generated. Standard VT sequences from the BC-2.09.002 table are
used instead. No panic. No silent key loss.

### AC-008 (traces to BC-2.09.004 invariant 1 — Kitty flags enabled globally at TUI startup)

`crossterm::event::PushKeyboardEnhancementFlags` with all four flags
(`DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES | REPORT_ASSOCIATED_TEXT`)
is called in TUI startup (global, not gated on `EmbeddedTerminal` entry). On TUI exit,
`PopKeyboardEnhancementFlags` is called. TUI startup also enables `EnableBracketedPaste`
(globally); TUI exit calls `DisableBracketedPaste`.

### AC-009 (traces to BC-2.09.005 postcondition 1–4 — bracketed paste wrapped and forwarded)

When `crossterm::event::Event::Paste(text)` is received while `AppMode::EmbeddedTerminal`
is active:
- The paste text is wrapped: `\x1b[200~` + `text.as_bytes()` + `\x1b[201~`.
- The complete bracketed payload is sent as `ClientToServer::KeyInput { session_id, bytes }`.
- Large pastes (>64 KiB) are handled without timeout — the full payload in a single `KeyInput`.

### AC-010 (traces to BC-2.09.005 invariant 2 — Paste handled via Event::Paste branch, not key_event_to_pty_bytes)

`Event::Paste` is a separate match branch in the TUI event dispatcher. It is NOT routed through
`key_event_to_pty_bytes()`. `Event::Key` and `Event::Paste` are distinct arms; paste events
arriving as `KeyEvent` (should not happen, but defensively handled) do NOT trigger the paste path.

### AC-011 (traces to BC-2.09.002 edge case EC-210 — Esc in EmbeddedTerminal exits; not forwarded)

`KeyCode::Esc` with no modifiers in `AppMode::EmbeddedTerminal` triggers `Action::ExitEmbeddedTerminal`
and exits embedded terminal mode. It is NOT forwarded to the PTY.

### AC-012 (traces to BC-2.09.002 edge case EC-214 — large bracketed paste)

A paste of 500 bytes: `\x1b[200~` + 500 bytes + `\x1b[201~` forwarded as a single `KeyInput` message.
No fragmentation. No timeout.

### AC-013 (traces to BC-2.09.002 edge case EC-215 — Release events not forwarded)

`KeyEventKind::Release` for any key returns `None`; 0 bytes sent to PTY.

### AC-014 (traces to BC-2.09.004 edge case EC-228 — Ctrl+Shift+Enter on non-Kitty terminal)

On a non-Kitty terminal, `is_kitty_enhanced_key()` returns `false`; the key falls through to
the standard VT table. `Enter` maps to `\r`; Ctrl+Shift modifier is not distinguishable in
standard VT. The VT fallback is used; no panic; no silent loss.

## Tasks

- [ ] Define core-owned mirror types in `crates/monocle-core/src/keyboard.rs` per SS-embedded-pty.md §Core-Owned Mirror Types: `PtyKeyCode`, `PtyKeyModifiers`, `PtyKeyEventKind`, `PtyKeyEvent`, `PtyMouseButton`, `PtyMouseEventKind`, `PtyMouseEvent`, `PtyRect`. These types carry NO crossterm/ratatui dependency.
- [ ] Implement `key_event_to_pty_bytes(event: PtyKeyEvent) -> Option<Vec<u8>>` in `crates/monocle-core/src/keyboard.rs` per the full BC-2.09.002 PC-2 key translation table. Uses `PtyKeyEvent` (core-owned), NOT `crossterm::event::KeyEvent`.
- [ ] Implement `is_kitty_enhanced_key(code: &PtyKeyCode, mods: PtyKeyModifiers) -> bool` in `crates/monocle-core/src/keyboard.rs`. Uses core-owned types only.
- [ ] Implement `encode_kitty_key(code: &PtyKeyCode, mods: PtyKeyModifiers, kind: PtyKeyEventKind) -> Vec<u8>` in `crates/monocle-core/src/keyboard.rs` using CSI u encoding: `ESC [ <codepoint> ; <1 + modifier_bits> u`. Uses core-owned types only.
- [ ] Create `crates/monocle-tui/src/keyboard_conv.rs` (NEW file — this story's scope): implement `crossterm_key_to_pty(e: crossterm::event::KeyEvent) -> PtyKeyEvent` conversion. This is the ONLY place crossterm types touch monocle-core's keyboard path. See SS-embedded-pty.md §Conversion in monocle-tui for the full conversion template.
- [ ] Implement `fn_key_bytes(n: u8) -> Vec<u8>` helper for F1–F12 per BC-2.09.002 table.
- [ ] Add `EmbeddedTerminal` keyboard dispatch arm in `crates/monocle-tui/src/event_loop.rs`:
  - Match `Event::Key(event)`: check for Esc (→ `Action::ExitEmbeddedTerminal`); else call `keyboard_conv::crossterm_key_to_pty(event)` to get `PtyKeyEvent`, then call `key_event_to_pty_bytes(pty_event)`; if `Some(bytes)`, send `ClientToServer::KeyInput { session_id, bytes }`. Crossterm type is converted at this dispatch boundary via `keyboard_conv`; `monocle-core` functions see only `PtyKeyEvent`.
  - Match `Event::Paste(text)`: wrap as `\x1b[200~` + text + `\x1b[201~`; send `ClientToServer::KeyInput`.
- [ ] Add global TUI startup keyboard setup in `crates/monocle-tui/src/event_loop.rs`: `PushKeyboardEnhancementFlags` (4 flags) + `EnableBracketedPaste`. Detect Kitty support via `CSI ? u` query; log TRACE if unsupported and skip flags.
- [ ] Add global TUI exit cleanup: `PopKeyboardEnhancementFlags` + `DisableBracketedPaste`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_printable`: `PtyKeyEvent { code: PtyKeyCode::Char('a'), modifiers: PtyKeyModifiers::NONE, kind: PtyKeyEventKind::Press }` → `key_event_to_pty_bytes(event) == Some(vec![0x61])`. Tests the pure `monocle-core` function directly — no crossterm types in the test.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_ctrl`: `PtyKeyEvent { code: PtyKeyCode::Char('c'), modifiers: PtyKeyModifiers::CONTROL, kind: Press }` → `Some(vec![0x03])`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_arrows`: all four `PtyKeyCode::Up/Down/Left/Right` → `\x1b[A`–`\x1b[D`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_fn_keys`: `PtyKeyCode::F(1..=4)` → `\x1bOP`–`\x1bOS`; `F(5)` → `\x1b[15~`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_release_discarded`: `PtyKeyEventKind::Release` → `None`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_ctrl_d_eot`: `PtyKeyCode::Char('d') + PtyKeyModifiers::CONTROL + Press` → `Some(vec![0x04])`.
- [ ] Write unit test `test_BC_2_09_002_esc_not_forwarded_directly`: separate dispatch test; Esc intercepted as `ExitEmbeddedTerminal` before `key_event_to_pty_bytes`.
- [ ] Write unit test `test_BC_2_09_004_kitty_ctrl_shift_enter`: `encode_kitty_key(&PtyKeyCode::Enter, PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT, PtyKeyEventKind::Press)` → `\x1b[13;6u`.
- [ ] Write unit test `test_BC_2_09_004_kitty_unsupported_fallback`: `is_kitty_enhanced_key(&PtyKeyCode::Enter, PtyKeyModifiers::NONE)` returns false; standard table used; no panic.
- [ ] Write unit test `test_BC_2_09_005_bracketed_paste_wrapped`: `Event::Paste("hello world")` → `\x1b[200~hello world\x1b[201~`.
- [ ] Write unit test `test_BC_2_09_005_bracketed_paste_empty`: `Event::Paste("")` → `\x1b[200~\x1b[201~`.

## Previous Story Intelligence

- **S-039** (PTY output pipeline): `AppMode::EmbeddedTerminal` variant is now defined in `monocle-core/src/app_mode.rs`. `App::enter_embedded_terminal()` and the IPC channel to the daemon are established. This story adds the keyboard event dispatch arm to the event loop.
- **S-025** (TUI skeleton): `crates/monocle-tui/src/event_loop.rs` exists with a `crossterm::event::read()` loop. This story extends that loop with a new match arm gated on `AppMode::EmbeddedTerminal`.
- The key table in BC-2.09.002 PC-2 is the authoritative source. Do NOT invent sequences from training data — read the BC table directly.

## Architecture Compliance Rules

- `key_event_to_pty_bytes()`, `is_kitty_enhanced_key()`, `encode_kitty_key()` are pure functions — live in `monocle-core/src/keyboard.rs` (pure core; no I/O). NOT in `monocle-tui`.
- The Action dispatch layer in `monocle-tui` intercepts Esc BEFORE calling `key_event_to_pty_bytes()`. This ordering is non-negotiable per BC-2.09.002 Invariant 2.
- `Event::Paste` MUST be a separate dispatch branch. It is NOT a `KeyEvent`. Do NOT route through `key_event_to_pty_bytes()`.
- Kitty flags enabled globally at startup (NOT gated on EmbeddedTerminal entry). Mouse capture is NOT enabled at startup (that is S-041's responsibility, scoped to EmbeddedTerminal entry).
- `encode_kitty_key` modifier encoding: modifier_value = 1 + sum(active bits: Shift=1, Alt=2, Ctrl=4). This is `1 + shift_bit + alt_bit + ctrl_bit`, NOT bitflags.
- Forbidden dependency: `monocle-core` MUST NOT depend on `monocle-tui`, `monocle-runtime`, `crossterm`, or `ratatui` — no exceptions, no feature-flag workarounds, no re-exports from `monocle-tui`. SS-tui.md §Scope states categorically: "`monocle-core` — pure data types. No I/O, no ratatui, no crossterm." This is the F-P2-I06 ruling (SS-embedded-pty.md §Dependency Boundary).
- The pure keyboard functions in `monocle-core/src/keyboard.rs` use ONLY core-owned mirror types (`PtyKeyEvent`, `PtyKeyCode`, `PtyKeyModifiers`, `PtyKeyEventKind`). These are defined in the same file and carry no external crate dependency.
- Conversion from crossterm types to Pty* types is confined to the NEW file `crates/monocle-tui/src/keyboard_conv.rs` (added in this story's scope). This file is the ONLY place in the workspace where crossterm types touch the monocle-core purity boundary. Adding any crossterm or ratatui type to `monocle-core/Cargo.toml` is FORBIDDEN.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `crossterm` | `"0.29"` (caret) | `KeyEvent`, `KeyCode`, `KeyModifiers`, `KeyEventKind`, `PushKeyboardEnhancementFlags`, `EnableBracketedPaste` | SS-deps-pin-manifest.md |
| `tokio` | `=1.52` (exact) | IPC send (`mpsc::channel`) | SS-deps-pin-manifest.md §Exact-pinned |
| `tracing` | `"0.1"` | TRACE logging for Kitty unsupported detection | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to CREATE:

| File | Purpose |
|------|---------|
| `crates/monocle-core/src/keyboard.rs` | Core-owned mirror types (`PtyKeyCode`, `PtyKeyModifiers`, `PtyKeyEventKind`, `PtyKeyEvent`, `PtyMouseButton`, `PtyMouseEventKind`, `PtyMouseEvent`, `PtyRect`); pure functions `key_event_to_pty_bytes()`, `is_kitty_enhanced_key()`, `encode_kitty_key()`, `fn_key_bytes()` — no crossterm/ratatui dep |
| `crates/monocle-tui/src/keyboard_conv.rs` | `crossterm_key_to_pty()` conversion (NEW file, this story's scope — the ONLY place crossterm types touch the monocle-core purity boundary). S-041 extends this file with `crossterm_mouse_to_pty()` and `ratatui_rect_to_pty()` in that story's scope. |

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-core/src/lib.rs` | `pub mod keyboard;` |
| `crates/monocle-tui/src/lib.rs` (or mod declaration file) | `pub mod keyboard_conv;` |
| `crates/monocle-tui/src/event_loop.rs` | Add global Kitty+paste setup on TUI startup; add `EmbeddedTerminal` event dispatch arm (Key → `keyboard_conv::crossterm_key_to_pty(event)` → `key_event_to_pty_bytes(pty_event)` → KeyInput send; Paste → bracket-wrap → KeyInput send; Esc intercept) |
| `crates/monocle-ipc/src/lib.rs` | Ensure `ClientToServer::KeyInput { session_id: String, bytes: Vec<u8> }` exists; add if absent |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~5,000 |
| BC-2.09.002 | ~4,000 |
| BC-2.09.004 | ~2,000 |
| BC-2.09.005 | ~2,000 |
| SS-embedded-pty.md §Full-Fidelity Keyboard Encoding; §Translation function; §Bracketed paste; §Crossterm setup | ~8,000 |
| Existing event_loop.rs + app_mode.rs (context for integration) | ~5,000 |
| monocle-ipc wire types (KeyInput) | ~1,500 |
| Test files to write | ~5,000 |
| **Total estimate** | **~32,500** |

Within the 30% context window bound. No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.09.002 | Full-Fidelity Keyboard Forwarding — All v1A Input Classes Reach PTY stdin | (see inputs: frontmatter) |
| BC-2.09.004 | Kitty Keyboard Protocol — Enhanced Key Events Forwarded as CSI u Sequences | (see inputs: frontmatter) |
| BC-2.09.005 | Bracketed Paste — Paste Events Wrapped in Bracket Sequences Before Forwarding | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `PtyKeyEvent`, `PtyKeyCode`, `PtyKeyModifiers`, `PtyKeyEventKind` (mirror types) | `monocle-core/src/keyboard.rs` | Pure core (data types; no crossterm/ratatui dep) |
| `key_event_to_pty_bytes(event: PtyKeyEvent) -> Option<Vec<u8>>` | `monocle-core/src/keyboard.rs` | Pure core (no I/O; deterministic; uses Pty* types only) |
| `is_kitty_enhanced_key(code: &PtyKeyCode, mods: PtyKeyModifiers) -> bool` | `monocle-core/src/keyboard.rs` | Pure core (no I/O) |
| `encode_kitty_key(code: &PtyKeyCode, mods: PtyKeyModifiers, kind: PtyKeyEventKind) -> Vec<u8>` | `monocle-core/src/keyboard.rs` | Pure core (no I/O) |
| `crossterm_key_to_pty(e: KeyEvent) -> PtyKeyEvent` | `monocle-tui/src/keyboard_conv.rs` (NEW) | Effectful shell boundary (infallible field-by-field crossterm → Pty* conversion; the ONLY place crossterm types touch the core purity boundary) |
| EmbeddedTerminal key dispatch arm | `monocle-tui/src/event_loop.rs` | Effectful shell (IPC send) |
| Bracketed paste dispatch arm | `monocle-tui/src/event_loop.rs` | Effectful shell (IPC send) |
| Kitty flag setup (TUI startup/exit) | `monocle-tui/src/event_loop.rs` | Effectful shell (terminal device I/O) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-210 | Esc pressed in EmbeddedTerminal | Intercepted as `Action::ExitEmbeddedTerminal`; NOT forwarded to PTY |
| EC-212 | `Ctrl-D` in EmbeddedTerminal | Forwarded as `\x04`; Claude Code session ends |
| EC-214 | 500-byte bracketed paste | Single `KeyInput` message with full bracketed payload |
| EC-215 | `KeyEventKind::Release` for any key | `None` returned; 0 bytes sent |
| EC-216 | Kitty protocol unsupported | Standard VT sequences used; no panic |
| EC-228 | `Ctrl+Shift+Enter` on non-Kitty terminal | Best-effort VT fallback; `Enter` → `\r` |

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because `key_event_to_pty_bytes()` and the keyboard forwarding dispatch are defined in SS-embedded-pty.md §Full-Fidelity Keyboard Encoding, which is the authoritative SS-09 spec for this functionality.

**Dependency Anchors:**
- S-040 depends on S-039 because S-039 defines `AppMode::EmbeddedTerminal` (required by the dispatch arm gating) and establishes the IPC send infrastructure in the event loop.
- S-040 blocks S-041 because S-041 (mouse forwarding + scoped mouse capture) extends the same EmbeddedTerminal event dispatch arm added in this story; the arm must exist before it can be extended.
- S-040 blocks S-044 because S-044 (AppMode transitions + permission badge) depends on `AppMode::EmbeddedTerminal` keyboard dispatch context being fully functional.
