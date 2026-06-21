---
document_type: story
level: L4
story_id: S-040
epic_id: EPIC-09
version: "1.7"
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
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.002.md, version: "1.2.0"}
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.004.md, version: "1.0.9"}
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.005.md, version: "1.0.7"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.13.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.09.002 (full-fidelity keyboard forwarding — all v1A input classes), BC-2.09.004 (Kitty keyboard protocol CSI u sequences), BC-2.09.005 (bracketed paste)"
# BC status: non-empty; status draft pending Phase-2 adversarial convergence gate (authoritative versions in inputs: frontmatter)
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
- The bytes match the authoritative key translation table (printable chars, Ctrl+[A-Z],
  Enter=`\r`, Backspace=`\x7f`, Tab=`\t`, arrows as `\x1b[A`–`\x1b[D`, F1–F12, Home/End/PgUp/PgDn/Ins/Del,
  Alt+char as ESC-prefix, Shift+Tab=`\x1b[Z`, Ctrl+Arrow VT fallbacks — exact sequences defined in
  SS-embedded-pty.md §Translation function; BC-2.09.002 PC-2 specifies the behavioral fidelity postcondition).

### AC-002 (traces to BC-2.09.002 postcondition 3 — Release events discarded)

`KeyEventKind::Release` events return `None` from `key_event_to_pty_bytes()`. They are NOT
sent to the PTY. Zero bytes are forwarded for Release events.

### AC-003 (traces to BC-2.09.002 invariant 1 — key_event_to_pty_bytes is pure)

`key_event_to_pty_bytes(event: PtyKeyEvent, kitty_active: bool) -> Option<Vec<u8>>` is a pure
function with no I/O and no state mutation. It is deterministic: the same `(event, kitty_active)`
input always produces the same output. It lives in `monocle-core` (pure core crate), not
`monocle-tui`. `kitty_active` is a plain `bool` input parameter — no I/O is performed inside
the function to determine it.

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

`App::kitty_active: bool` is set at TUI startup after the `CSI ? u` terminal capability query.
When `kitty_active = true`, `is_kitty_enhanced_key(&event.code, mods, kitty_active)` returns
`true` for any modifier-carrying key combo not already matched by a specific arm.
`encode_kitty_key(&event.code, mods, event.kind)` produces a CSI u sequence:
`ESC [ <unicode_codepoint> ; <modifier_value> u`.
- `<modifier_value>` = 1 + sum of active modifier bits: Shift=1, Alt=2, Ctrl=4.
- Example: `Ctrl+Shift+Enter` → `\x1b[13;6u` (codepoint 13; modifier = 1 + 1(shift) + 4(ctrl) = 6).
- Example: `Shift+Tab` (Kitty path) → `\x1b[9;2u` (codepoint 9; modifier = 1 + 1(shift) = 2).

IMPORTANT: `key_event_to_pty_bytes` signature is `(event: PtyKeyEvent, kitty_active: bool) -> Option<Vec<u8>>`.
The monocle-tui dispatch arm passes `app.kitty_active` when calling this function.
Both `is_kitty_enhanced_key` and `key_event_to_pty_bytes` remain PURE functions (no I/O);
`kitty_active` is a plain bool input parameter.

### AC-007 (traces to BC-2.09.004 invariant 2 — Kitty unsupported terminals use VT fallback)

When the `CSI ? u` query returns no response (timeout) or a non-Kitty response, `kitty_active`
is set to `false`. `PushKeyboardEnhancementFlags` is NOT called. `is_kitty_enhanced_key()`
returns `false` for all keys when `kitty_active = false`. Standard VT sequences from the
BC-2.09.002 table are used. Modifier combos with no VT encoding arm emit a TRACE log and
return `None` (observable but not forwarded — the best-effort boundary per BC-2.09.002 PC-1).
No panic. No silent key loss (TRACE makes all drops observable).

### AC-008 (traces to BC-2.09.004 invariant 1 — Kitty flags enabled globally at TUI startup)

Three Kitty enhancement flags are enabled at TUI startup: `DISAMBIGUATE_ESCAPE_CODES |
REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES`. `REPORT_ASSOCIATED_TEXT` is not
used (unavailable in crossterm-0.29). `REPORT_ALTERNATE_KEYS` is not used (no v1A
requirement). On TUI exit, `PopKeyboardEnhancementFlags` is called. TUI startup also
enables `EnableBracketedPaste` (globally); TUI exit calls `DisableBracketedPaste`.
Compilation succeeds on crossterm-0.29.

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

### AC-015 (traces to BC-2.09.002 edge case EC-218 — Esc+modifier not intercepted; routed by kitty_active)

`KeyCode::Esc` with ANY non-empty modifier set (e.g., Alt+Esc, Ctrl+Esc, Shift+Esc) is NOT
intercepted as `Action::ExitEmbeddedTerminal`. The `Action::ExitEmbeddedTerminal` intercept
applies ONLY to bare Esc (`mods.is_empty() == true`). For Esc+modifier combos:
- When `kitty_active = true`: `encode_kitty_key` produces a CSI u sequence (e.g., Alt+Esc →
  `ESC [ 27 ; 3 u`); the sequence is forwarded to the PTY as `ClientToServer::KeyInput`.
- When `kitty_active = false`: the `_ if !mods.is_empty()` TRACE+None arm fires; a TRACE log
  is emitted and `None` is returned; 0 bytes forwarded (best-effort boundary per EC-217).
In both cases no panic occurs and no silent drop (all drops are TRACE-observable).

## Tasks

- [ ] Define core-owned mirror types in `crates/monocle-core/src/keyboard.rs` per SS-embedded-pty.md §Core-Owned Mirror Types: `PtyKeyCode`, `PtyKeyModifiers`, `PtyKeyEventKind`, `PtyKeyEvent`, `PtyMouseButton`, `PtyMouseEventKind`, `PtyMouseEvent`, `PtyRect`. These types carry NO crossterm/ratatui dependency.
- [ ] Implement `key_event_to_pty_bytes(event: PtyKeyEvent, kitty_active: bool) -> Option<Vec<u8>>` in `crates/monocle-core/src/keyboard.rs` per the full BC-2.09.002 PC-2 key translation table and the corrected match precedence from SS-embedded-pty.md §Translation function. Uses `PtyKeyEvent` (core-owned), NOT `crossterm::event::KeyEvent`.
- [ ] Implement `is_kitty_enhanced_key(code: &PtyKeyCode, mods: PtyKeyModifiers, kitty_active: bool) -> bool` in `crates/monocle-core/src/keyboard.rs`. Returns `false` immediately when `!kitty_active || mods.is_empty()`. Uses core-owned types only.
- [ ] Implement `encode_kitty_key(code: &PtyKeyCode, mods: PtyKeyModifiers, kind: PtyKeyEventKind) -> Vec<u8>` in `crates/monocle-core/src/keyboard.rs` using CSI u encoding: `ESC [ <codepoint> ; <1 + modifier_bits> u`. Uses core-owned types only.
- [ ] Create `crates/monocle-tui/src/keyboard_conv.rs` (NEW file — this story's scope): implement `crossterm_key_to_pty(e: crossterm::event::KeyEvent) -> PtyKeyEvent` conversion. This is the ONLY place crossterm types touch monocle-core's keyboard path. See SS-embedded-pty.md §Conversion in monocle-tui for the full conversion template.
- [ ] Implement `fn_key_bytes(n: u8) -> Vec<u8>` helper for F1–F12 per BC-2.09.002 table.
- [ ] Add `EmbeddedTerminal` keyboard dispatch arm in `crates/monocle-tui/src/event_loop.rs`:
  - Match `Event::Key(event)`: check for Esc (→ `Action::ExitEmbeddedTerminal`); else call `keyboard_conv::crossterm_key_to_pty(event)` to get `PtyKeyEvent`, then call `key_event_to_pty_bytes(pty_event, app.kitty_active)`; if `Some(bytes)`, send `ClientToServer::KeyInput { session_id, bytes }`. Crossterm type is converted at this dispatch boundary via `keyboard_conv`; `monocle-core` functions see only `PtyKeyEvent`.
  - Match `Event::Paste(text)`: wrap as `\x1b[200~` + text + `\x1b[201~`; send `ClientToServer::KeyInput`.
  - SSOT DISPATCH RULE (ADV-HIGH-002 ruling): if a `dispatch_embedded_terminal_key` helper exists anywhere in the codebase (e.g., in `event_loop.rs`), `handle_crossterm_event` (app.rs) MUST call that helper — it MUST NOT inline duplicate key dispatch logic. There must be exactly ONE code path from crossterm event to KeyInput IPC send. If the helper exists and has a hardcoded `kitty_active=false`, add the `kitty_active: bool` parameter and route `app.kitty_active` through it. Delete any inline duplicate. Both production code and tests must use the same code path.
- [ ] Add `kitty_active: bool` field to the `App` struct (in the appropriate `App` struct definition in the codebase — monocle-core or monocle-tui per project conventions; the implementer places it). Initialize to `false`.
- [ ] At TUI startup: detect Kitty support using `crossterm::terminal::supports_keyboard_enhancement() -> io::Result<bool>` (available in crossterm-0.29 at `crossterm::terminal`; uses crossterm's internal event pipeline — NOT a raw stdin reader). Set `app.kitty_active = supports_keyboard_enhancement().unwrap_or(false)`. TRACE-log when `false`. Call `PushKeyboardEnhancementFlags` (3 flags: `DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES`) ONLY when `kitty_active == true`. Call `EnableBracketedPaste` unconditionally. On TUI exit, call `PopKeyboardEnhancementFlags` only if `kitty_active == true`; call `DisableBracketedPaste` unconditionally. FORBIDDEN: spawning any thread (detached or otherwise) calling `std::io::stdin().lock().read()` or any blocking raw stdin read — this steals keystrokes from crossterm's event pipeline and is a data-loss bug (SS-embedded-pty.md §Risk Mitigations ADV-BLOCKER-001+O-2).
- [ ] Update `key_event_to_pty_bytes` to signature `(event: PtyKeyEvent, kitty_active: bool) -> Option<Vec<u8>>` and `is_kitty_enhanced_key` to `(code: &PtyKeyCode, mods: PtyKeyModifiers, kitty_active: bool) -> bool`; update all callers in `event_loop.rs`/`app.rs` to pass `app.kitty_active`.
- [ ] Implement the corrected match precedence per SS-embedded-pty §Translation function inside `key_event_to_pty_bytes`: (1) unmodified specific keys, (2) Ctrl+printable, (3) kitty_active-gated catch-all (`is_kitty_enhanced_key`), (4) Alt+printable ESC-prefix, (5) BackTab, (6) VT-fallback modified arrows, (7) `_ if !mods.is_empty()` TRACE+None arm, (8) `_ => None`.
- [ ] Add `_ if !mods.is_empty()` arm in `key_event_to_pty_bytes`: TRACE-log the `(code, mods)` pair with message `"key_event_to_pty_bytes: no VT encoding for modifier combo on non-Kitty terminal; dropping"` and return `None` (BC-2.09.002 EC-217 — no silent drop).
- [ ] Add global TUI exit cleanup (if not already added in the startup task above): `PopKeyboardEnhancementFlags` (gated on `kitty_active`) + `DisableBracketedPaste`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_printable`: `PtyKeyEvent { code: PtyKeyCode::Char('a'), modifiers: PtyKeyModifiers::NONE, kind: PtyKeyEventKind::Press }` → `key_event_to_pty_bytes(event) == Some(vec![0x61])`. Tests the pure `monocle-core` function directly — no crossterm types in the test.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_ctrl`: `PtyKeyEvent { code: PtyKeyCode::Char('c'), modifiers: PtyKeyModifiers::CONTROL, kind: Press }` → `Some(vec![0x03])`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_arrows`: all four `PtyKeyCode::Up/Down/Left/Right` → `\x1b[A`–`\x1b[D`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_fn_keys`: `PtyKeyCode::F(1..=4)` → `\x1bOP`–`\x1bOS`; `F(5)` → `\x1b[15~`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_release_discarded`: `PtyKeyEventKind::Release` → `None`.
- [ ] Write unit test `test_BC_2_09_002_keyboard_forwarding_ctrl_d_eot`: `PtyKeyCode::Char('d') + PtyKeyModifiers::CONTROL + Press` → `Some(vec![0x04])`.
- [ ] Write unit test `test_BC_2_09_002_esc_not_forwarded_directly`: separate dispatch test; Esc intercepted as `ExitEmbeddedTerminal` before `key_event_to_pty_bytes`.
- [ ] Write unit test `test_BC_2_09_004_kitty_ctrl_shift_enter`: `encode_kitty_key(&PtyKeyCode::Enter, PtyKeyModifiers::CONTROL | PtyKeyModifiers::SHIFT, PtyKeyEventKind::Press)` → `\x1b[13;6u`.
- [ ] Write unit test `test_BC_2_09_004_kitty_ctrl_up`: `encode_kitty_key(&PtyKeyCode::Up, PtyKeyModifiers::CONTROL, PtyKeyEventKind::Press)` → `\x1b[57352;5u` (Kitty functional-key codepoint for Up = 57352; modifier = 1 + ctrl(4) = 5). This test vector is REQUIRED to verify correct Kitty codepoints — placeholder codepoints like 65='A' produce wrong output (SS-embedded-pty.md §Trace v1.13.0 O-1 ruling).
- [ ] Write unit test `test_BC_2_09_004_kitty_unsupported_fallback`: `is_kitty_enhanced_key(&PtyKeyCode::Enter, PtyKeyModifiers::NONE, false)` returns `false`; standard table used; no panic.
- [ ] Write unit test `test_BC_2_09_005_bracketed_paste_wrapped`: `Event::Paste("hello world")` → `\x1b[200~hello world\x1b[201~`.
- [ ] Write unit test `test_BC_2_09_005_bracketed_paste_empty`: `Event::Paste("")` → `\x1b[200~\x1b[201~`.
- [ ] Write unit test `test_BC_2_09_002_modifier_combo_no_vt_trace_none`: `key_event_to_pty_bytes(PtyKeyEvent { code: PtyKeyCode::Up, modifiers: PtyKeyModifiers::ALT, kind: PtyKeyEventKind::Press }, false)` → `None` (non-Kitty terminal; TRACE log emitted; no silent drop per EC-217).
- [ ] Write unit test `test_BC_2_09_004_kitty_active_true_modifier_combo`: `is_kitty_enhanced_key(&PtyKeyCode::Up, PtyKeyModifiers::CONTROL, true)` → `true` (kitty_active=true; modifier-carrying combo; returns true for Kitty CSI-u path).
- [ ] Write unit test `test_BC_2_09_004_kitty_active_false_returns_false`: `is_kitty_enhanced_key(&PtyKeyCode::Up, PtyKeyModifiers::CONTROL, false)` → `false` (kitty_active=false; function returns false unconditionally per early-return guard).
- [ ] Write unit test `test_BC_2_09_002_ctrl_at_nul`: `key_event_to_pty_bytes(PtyKeyEvent { code: PtyKeyCode::Char('@'), modifiers: PtyKeyModifiers::CONTROL, kind: Press }, false)` → `Some(vec![0x00])` (Ctrl+@ → `\x00` NUL).
- [ ] Write unit test `test_BC_2_09_002_ctrl_bracket_esc`: `key_event_to_pty_bytes(PtyKeyEvent { code: PtyKeyCode::Char('['), modifiers: PtyKeyModifiers::CONTROL, kind: Press }, false)` → `Some(vec![0x1b])` (Ctrl+[ → `\x1b` ESC).
- [ ] Write unit test `test_BC_2_09_005_paste_with_esc_verbatim` (EC-230): paste text containing ESC characters (e.g., `"\x1b[31mred\x1b[0m"`) is forwarded verbatim inside brackets — `\x1b[200~\x1b[31mred\x1b[0m\x1b[201~`.
- [ ] Write unit test `test_BC_2_09_005_paste_embedded_bracket_verbatim` (EC-231): paste text containing `"\x1b[200~"` is forwarded verbatim inside the outer brackets without sanitization.
- [ ] **ADV-HIGH-002 (in-scope mitigation — BC-2.09.005 EC-245):** Add a size guard in `dispatch_embedded_terminal_paste`: if the framed bracketed payload (`\x1b[200~` + text + `\x1b[201~` in the `KeyInput` JSON envelope) would exceed `monocle_ipc` framing `MAX_MESSAGE_BYTES` (262144), emit a WARN and DROP the paste without enqueuing (prevents killing the IPC writer task). Pastes at/below the ceiling forward normally (BC-2.09.005 Invariant-3).
- [ ] Write unit test `test_BC_2_09_005_oversized_paste_guard`: a paste whose framed bracketed payload exceeds `MAX_MESSAGE_BYTES` is NOT enqueued (0 `KeyInput` messages sent / guard returns), WARN logged (traces to BC-2.09.005 EC-245).
- [ ] **ADV-HIGH-001 (modified-arrow VT-fallback arm guards — SS-embedded-pty §Translation function):** Modified-arrow VT-fallback arms MUST use EXACT modifier equality per SS-embedded-pty §Translation function: `PtyKeyCode::Up if mods == CONTROL => \x1b[1;5A`; `if mods == SHIFT => \x1b[1;2A`; etc. — NOT `contains()`. Ctrl+Alt+Up and other multi-modifier arrow combos with no exact VT arm fall to the `_ if !mods.is_empty()` TRACE+None arm (EC-217), not a fabricated single-modifier sequence. **DISTINCTION:** The Ctrl+printable arm guard remains `contains(CONTROL) && !contains(ALT)` per the prior architect ruling (pass-3); only the ARROW VT-fallback arms use exact equality. This distinction must be explicit in the implementation to prevent regression of the Char arm.
- [ ] Write unit test `test_BC_2_09_002_ctrl_alt_up_trace_none`: `key_event_to_pty_bytes(PtyKeyEvent { code: PtyKeyCode::Up, modifiers: PtyKeyModifiers::CONTROL | PtyKeyModifiers::ALT, kind: PtyKeyEventKind::Press }, false)` → `None` (EC-217: multi-modifier arrow combo with no exact VT arm; TRACE logged; no fabricated sequence).

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
| `crates/monocle-tui/src/event_loop.rs` | Add global Kitty+paste setup on TUI startup (`supports_keyboard_enhancement()` → conditional flag push; `EnableBracketedPaste`); add `EmbeddedTerminal` event dispatch arm (Key → `keyboard_conv::crossterm_key_to_pty(event)` → `key_event_to_pty_bytes(pty_event, app.kitty_active)` → KeyInput send; Paste → bracket-wrap → KeyInput send; Esc intercept). SSOT: if `dispatch_embedded_terminal_key` helper exists, route through it (ADV-HIGH-002). |
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
| `key_event_to_pty_bytes(event: PtyKeyEvent, kitty_active: bool) -> Option<Vec<u8>>` | `monocle-core/src/keyboard.rs` | Pure core (no I/O; deterministic; uses Pty* types only) |
| `is_kitty_enhanced_key(code: &PtyKeyCode, mods: PtyKeyModifiers, kitty_active: bool) -> bool` | `monocle-core/src/keyboard.rs` | Pure core (no I/O) |
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
| EC-218 | Esc+modifier (e.g., Alt+Esc, Ctrl+Esc) | NOT intercepted as ExitEmbeddedTerminal (bare-Esc-only intercept); `kitty_active=true` → CSI-u sequence forwarded; `kitty_active=false` → TRACE+None (EC-217 boundary) |
| EC-228 | `Ctrl+Shift+Enter` on non-Kitty terminal | Best-effort VT fallback; `Enter` → `\r` |

## Wave-Gate Follow-Ups (non-blocking — do NOT implement in this story)

> **[wave-gate]** IPC writer-task error taxonomy (`spawn_ipc_writer`) should distinguish `MessageTooLarge` (drop-and-continue) from `IoError`/`Disconnected` (exit+reconnect) — this is a cross-cutting concern tracked at the Wave-9 integration gate. Origin: F-S026. Do NOT add this to S-040's scope.

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because `key_event_to_pty_bytes()` and the keyboard forwarding dispatch are defined in SS-embedded-pty.md §Full-Fidelity Keyboard Encoding, which is the authoritative SS-09 spec for this functionality.

**Dependency Anchors:**
- S-040 depends on S-039 because S-039 defines `AppMode::EmbeddedTerminal` (required by the dispatch arm gating) and establishes the IPC send infrastructure in the event loop.
- S-040 blocks S-041 because S-041 (mouse forwarding + scoped mouse capture) extends the same EmbeddedTerminal event dispatch arm added in this story; the arm must exist before it can be extended.
- S-040 blocks S-044 because S-044 (AppMode transitions + permission badge) depends on `AppMode::EmbeddedTerminal` keyboard dispatch context being fully functional.

## Trace

| Version | Change | Pass |
|---------|--------|------|
| v1.7 | BC-2.09.005 input pin bumped v1.0.6→v1.0.7 (EC-245 over-ceiling paste guard added by PO). Two ADV-HIGH tasks added: ADV-HIGH-002 paste-ceiling guard in `dispatch_embedded_terminal_paste` (DROP + WARN if framed payload exceeds `MAX_MESSAGE_BYTES` 262144) + `test_BC_2_09_005_oversized_paste_guard`; ADV-HIGH-001 modified-arrow VT-fallback arm exact-equality guard (`mods == CONTROL` not `contains()` — distinct from Ctrl+printable arm which retains `contains(CONTROL) && !contains(ALT)`) + `test_BC_2_09_002_ctrl_alt_up_trace_none`. Wave-gate follow-up section added (IPC writer-task error taxonomy, cross-cutting, Wave-9 gate; F-S026 origin). | story-writer |
| v1.6 | Input pins finalized: BC-2.09.002 bumped to v1.2.0 (EC-218 Esc+modifier edge case — PO patch), BC-2.09.005 bumped to v1.0.6 (Invariant-3 paste ceiling corrected — PO patch). AC-015 added (EC-218 Esc+modifier: bare-Esc-only intercept; kitty_active=true → CSI-u; kitty_active=false → TRACE+None). EC-218 row added to Edge Cases table. BC table version literals de-versioned (pins live in inputs[] only). | story-writer |
| v1.5 | supports_keyboard_enhancement (crossterm::terminal API) replaces CSI-probe design; SSOT dispatch rule (ADV-HIGH-002) added to Tasks; test_BC_2_09_004_kitty_ctrl_up Kitty codepoint test vector (Up=57352; modifier=5) added per SS-embedded-pty O-1 ruling. Input pins updated (implemented against BC-2.09.002 at v1.1.9 and BC-2.09.005 at v1.0.5 at S-040 v1.5 authoring time). | architect pass-3 |
| v1.4 | Implementation Tasks added per architect-specified Kitty CSI-u redesign. Input pins updated: BC-2.09.002 (implemented against v1.1.7 at S-040 v1.3 authoring time, now at v1.1.8), BC-2.09.004 (implemented against v1.0.7 at S-040 v1.3 authoring time, now at v1.0.8). Tasks updated to reflect corrected signatures (`kitty_active: bool` param on `key_event_to_pty_bytes` and `is_kitty_enhanced_key`); startup task expanded with `CSI ?u` query sequence, 100ms timeout, conditional flag push/pop; Architecture Mapping table updated; nine new unit tests added (EC-217 TRACE+None, kitty_active true/false, Ctrl+@ NUL, Ctrl+[ ESC, EC-230/EC-231 paste verbatim). | story-writer |
| v1.3 | S-040 adversarial pass-2 architect ruling (implemented against SS-embedded-pty at v1.12.0 at S-040 v1.3 authoring time). Three compounding design gaps corrected: (1) F-S040-BLOCKER-001: `is_kitty_enhanced_key` was hardcoded `false`; Kitty arm was dead code. (2) F-S040-HIGH-003: crossterm-0.29 has no "enhanced" KeyCode variants; a pure `(code, mods)` function cannot detect Kitty-active. (3) F-S040-HIGH-001: unmatched modifier combos silently dropped on non-Kitty terminals. Correct design: `is_kitty_enhanced_key(code, mods, kitty_active: bool)` + `key_event_to_pty_bytes(event, kitty_active: bool)`; `App::kitty_active` set from `CSI ? u` query at startup; `_ if !mods.is_empty()` TRACE+None arm for HIGH-001. AC-006 and AC-007 updated to reflect correct design. | architect ruling |
| v1.2 | AC-008 dependency-reality cascade: architect (SS-embedded-pty §Crossterm setup) and product-owner (BC-2.09.004) ruled that `REPORT_ASSOCIATED_TEXT` is unavailable in crossterm-0.29. AC-008 flag enumeration corrected from four flags to three (`DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES | REPORT_EVENT_TYPES`); `REPORT_ALTERNATE_KEYS` explicitly excluded (no v1A requirement). Tasks startup line updated to match. Input pins updated to authoritative versions at that time. <!-- version-pin-historical: authored against SS-embedded-pty v1.11.0 at S-040 v1.2 authoring time --> No behavior change to other ACs. | dependency-reality cascade |
| v1.1 | F-P21-SUG-001: AC-001 citation-scope correction — "BC-2.09.002 PC-2 table" was imprecise as the literal VT sequence table lives in SS-embedded-pty.md §Translation function; AC-001 now co-cites both (BC-2.09.002 PC-2 for behavioral fidelity; SS-embedded-pty.md §Translation function for the exact sequence mapping). No AC behavior change. | post-convergence |
| v1.0 | Initial decomposition. | Phase-2 |
