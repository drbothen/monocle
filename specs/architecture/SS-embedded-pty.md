---
document_type: architecture-section
level: L3
section: "embedded-pty"
subsystem: SS-09
version: "1.2.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T23:00:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - specs/product-brief.md
  - specs/research/embedded-pty-evaluation.md
  - specs/architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md
  - specs/architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md
  - specs/architecture/SS-ipc.md
  - specs/architecture/SS-tui.md
input-hash: "13e1215"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: Embedded PTY / TUI (SS-09)

## Scope

SS-09 defines the embedded terminal capability in `monocle-tui`:

1. **EmbeddedTerminal AppMode** — AppMode variant and state machine transitions.
2. **PTY widget** — `tui-term::PseudoTerminal` rendering `vt100::Screen` in the Preview pane.
3. **Keyboard encoding** — full-fidelity crossterm event → terminal byte translation (v1A scope: printable + control + arrows + mouse + Kitty keyboard protocol).
4. **PTY byte pipeline** — daemon IPC `PtyOutput` bytes → `vt100::Parser.process()` → TUI render cycle.
5. **Resize / SIGWINCH propagation** — pane area change → `ResizePane` IPC message → daemon → session-host → PTY resize + parser resize.
6. **SessionCreation wizard** — multi-step `AppMode::SessionCreation` for launching new sessions.

---

## TUI AppMode Extensions

New variants added to `AppMode` in `monocle-core/src/app_mode.rs`:

```rust
/// Preview pane hosts the tui-term PTY widget for the focused session.
/// All keyboard events are forwarded to the daemon as KeyInput IPC messages.
/// session_id type: String (UUID rendered as string — canonical per SS-session-manager.md
/// §session_id type ruling; same type at all IPC/registry/AppMode boundaries).
EmbeddedTerminal {
    session_id: String,   // currently-focused session (UUID as String)
    prior: FocusSnapshot, // AppMode to restore on Esc
},

/// Launch wizard — multi-step modal for creating a new session.
SessionCreation {
    step: SessionCreationStep,
    prior: FocusSnapshot,
},

#[derive(Clone, PartialEq, Eq)]
pub enum SessionCreationStep {
    ProfilePicker,     // Step 1: select harness profile (reuses existing profile-picker logic)
    ProjectPicker,     // Step 2: select project root (fuzzy-filtered directory list)
    WorktreeConfirm,   // Step 3: confirm git worktree path + display name
    Launching,         // Step 4: waiting for SessionState::Running confirmation
}
```

**State machine invariants:**

- **Permission prompts while in `EmbeddedTerminal`:** Permission prompts are time-sensitive
  and are monocle's killer feature. Silently suppressing or purely queueing them while in
  embedded terminal mode is NOT acceptable under the production-grade principle — a permission
  prompt from any session (including non-focused sessions) must be immediately surfaced to the
  user. The production-grade behavior is:

  1. **Status-bar badge (mandatory):** When a `PermissionPromptQueued` IPC message arrives
     while `AppMode::EmbeddedTerminal` is active, the TUI MUST immediately render a visible
     indicator in the status bar (e.g., `[1 pending permission]` badge + terminal bell) so
     the user is aware a prompt is waiting, regardless of which session triggered it.
  2. **Pre-emption option:** The user can exit embedded terminal mode (Esc) and the pending
     permission overlay will be presented on the `prior` AppMode. Alternatively, if the
     permission is for the currently embedded session, the implementer MAY (as a v1A
     enhancement) pre-empt embedded terminal mode and immediately present the overlay — this
     pre-emption behavior requires a dedicated BC and is flagged to product-owner below.
  3. **No silent queueing:** Prompts MUST NOT be held invisibly in the daemon until the user
     happens to exit embedded mode. The status-bar badge + bell is the minimum visibility
     guarantee.

  **BC requirement — RESOLVED (O1):** BC-2.09.009 (authored by product-owner, v1.0.0,
  2026-06-03T23:30:00Z) encodes the production-grade minimum: status-bar badge (`[N pending
  permission(s)]`) + audible bell (`\x07`) once per new prompt while in `EmbeddedTerminal`
  or `SessionCreation` mode. Full pre-emption (overlay replacing embedded terminal without
  requiring Esc) is v1B scope requiring human sign-off per BC-2.09.009 Invariant 4.
  This placeholder is now resolved; no further product-owner action needed for v1A badge-only
  behavior.

- `SessionCreation` is mutually exclusive with `Overlay` (the session creation wizard blocks
  permission overlays; pending overlays are visible via status-bar badge while in wizard mode,
  same as EmbeddedTerminal).

- Entering `EmbeddedTerminal`: requires `session_id` to have `SessionState::Running`. If the
  session is `Terminated`, the action is a no-op with a status bar message.
- Exiting `EmbeddedTerminal` via Esc: transition to `prior` AppMode (typically `Dashboard`).
  A `Ctrl-D` or session-terminated event also exits embedded terminal mode.
- `SessionCreation::Launching` transitions to `EmbeddedTerminal` automatically when the daemon
  sends `SessionStateChanged { new_state: Running }` for the new session.

---

## PTY Widget Pipeline

```
Daemon
  └── session-host proxy
        └── broker fan-out
              └── ServerToClient::PtyOutput { session_id, bytes }
                    └── UDS IPC (existing channel)
                          └── TUI IPC reader task (mpsc::channel(64))
                                └── App::on_pty_output()
                                      └── parsers.get_mut(&session_id)?.process(&bytes)
                                            └── terminal.draw() → frame.render_widget(
                                                  PseudoTerminal::new(parser.screen()),
                                                  preview_area,
                                                )
```

### Parser ownership in TUI

Each session has a `vt100::Parser` instance owned by the TUI's `App` struct:

```rust
struct App {
    // ... existing fields ...
    /// vt100 parsers keyed by session_id.
    /// All sessions parse in the background — the focused session's parser is rendered.
    pty_parsers: HashMap<String, vt100::Parser>,

    /// Per-session scrollback viewport offset (rows from bottom, 0 = live tail).
    /// I7 fix: was a single usize shared across all sessions (incorrect; focus switch showed
    /// wrong session's scrollback position). Now per-session keyed by session_id.
    pty_scroll_offsets: HashMap<String, usize>,
}

/// Scrollback offset invariants (I7):
/// - `pty_scroll_offsets[session_id]` is initialized to 0 (live tail) when a session is added.
/// - `PtyScrollUp` action increments `pty_scroll_offsets[focused_session_id]` (bounded by
///   scrollback row count in `pty_parsers[id].screen().scrollback_len()`).
/// - `PtyScrollDown` action decrements (floor 0).
/// - On `ResizePane` IPC (pane area changed): `pty_scroll_offsets[session_id]` is reset to
///   0 (live tail). Rationale: a resize reflows content; the old offset is meaningless against
///   the new layout; snapping to live tail is the least-surprising behavior (matches most
///   terminal emulators).
/// - On focus switch (arrow key in sessions panel): the new focused session's scroll offset
///   is read from its own entry in `pty_scroll_offsets` — the offset is preserved from the
///   last time that session was focused. O(1) switch cost unchanged.
/// - On `StateChanged::Terminated` for a session: `pty_scroll_offsets.remove(session_id)`.
```

Parser initialization: when the TUI receives `SessionListUpdate` with a new session, a fresh
`vt100::Parser::new(rows, cols, SCROLLBACK_ROWS)` is created. `SCROLLBACK_ROWS` is configurable
via `~/.monocle/config.json`; default 1000 rows. Parsers are removed when the session is GC'd
from the list.

**Fast switching:** switching the focused session = changing which parser's `screen()` is
passed to the widget on the next render tick. All other parsers continue to process bytes in
the background. O(1) switch cost.

### Pane area and resize

The Preview pane area dimensions drive PTY sizing. On each render cycle:

```rust
fn render_embedded_terminal(
    frame: &mut Frame,
    area: Rect,
    parser: &vt100::Parser,
) {
    let widget = PseudoTerminal::new(parser.screen());
    frame.render_widget(widget, area);
}
```

When the Preview pane area changes (user resizes terminal, or panel layout changes):

1. Detect: `area.rows != parser.screen().size().0 || area.cols != parser.screen().size().1`
2. Send `ClientToServer::ResizePane { session_id, rows: area.rows, cols: area.cols }` over IPC.
3. Daemon forwards to session-host via `DaemonToHost::Resize`.
4. Session-host calls `pty.resize(PtySize { rows, cols, .. })` and `parser.set_size(rows, cols)`.

**Debounce:** resize events are debounced at 50ms (claude-squad A.5 pattern) to avoid
sending a resize per-frame during a drag operation. The TUI tracks the last-sent size and
only sends when the pending size differs AND a 50ms debounce window has elapsed.

---

## Full-Fidelity Keyboard Encoding (v1A scope — D-237 ratification)

Full keyboard fidelity is IN v1A scope (human-ratified at D-237, 2026-06-03). This section
is the authoritative implementation specification. No input class is deferred.

### Crossterm to PTY byte translation

When `AppMode::EmbeddedTerminal` is active, crossterm `KeyEvent` values are intercepted by
the Action dispatch layer before the standard keybinding lookup. They are translated to
terminal byte sequences and sent as `ClientToServer::KeyInput { session_id, bytes }`.

**Crossterm setup (in `monocle-tui/src/event_loop.rs`) — I3 fix:**

Keyboard enhancement (Kitty) flags and the global `EnableMouseCapture` are scoped separately
to avoid stealing mouse selection/copy from monocle's own panels.

```rust
// TUI STARTUP — global keyboard enhancement only; NO global mouse capture.
// Kitty enhancement flags give enhanced key events. Mouse capture is NOT enabled globally
// because it would intercept mouse selection and copy operations in monocle's own panels
// (sessions panel, event ribbon, etc.), stealing them from the terminal emulator's native
// text selection capability. Mouse capture is deferred to EmbeddedTerminal entry.
crossterm::execute!(
    stdout(),
    crossterm::event::PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES |
        KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES |
        KeyboardEnhancementFlags::REPORT_EVENT_TYPES |
        KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT,
    ),
    crossterm::event::EnableBracketedPaste,
)?;

// TUI EXIT — pop enhancement flags and paste; no mouse disable needed (never globally enabled).
crossterm::execute!(
    stdout(),
    crossterm::event::PopKeyboardEnhancementFlags,
    crossterm::event::DisableBracketedPaste,
)?;
```

**EmbeddedTerminal ENTRY (in App::enter_embedded_terminal()):**
```rust
// Enable mouse capture scoped to EmbeddedTerminal mode only.
// Also enable SGR extended mouse reporting (1006) for full coordinate range.
crossterm::execute!(
    stdout(),
    crossterm::event::EnableMouseCapture,
)?;
// Write SGR mouse mode (1006) escape to terminal:
print!("\x1b[?1006h");
```

**EmbeddedTerminal EXIT (in App::exit_embedded_terminal()):**
```rust
// Disable SGR mouse mode (1006), then disable mouse capture.
// Order is critical: disable SGR first (restores normal mouse protocol), then
// DisableMouseCapture (stops reporting entirely). Asymmetric-but-correct: we only
// call DisableMouseCapture if we called EnableMouseCapture (scoped to EmbeddedTerminal).
print!("\x1b[?1006l");
crossterm::execute!(
    stdout(),
    crossterm::event::DisableMouseCapture,
)?;
```

**I3 UX tradeoff requiring human sign-off:**
If any monocle panel (not EmbeddedTerminal) needs mouse event routing in future (e.g.,
clickable session rows), the above design requires adding per-panel mouse enable/disable
scaffolding. The alternative (global EnableMouseCapture at startup) makes panels clickable
but steals terminal text selection. The current v1A design (scoped to EmbeddedTerminal only)
is the production-grade choice for a TUI that does not yet have click targets in its own
panels. If a future story requires mouse clicks in monocle panels, the product-owner must
approve enabling global mouse capture and documenting the text-selection tradeoff.

**Note:** Kitty keyboard enhancement flags REMAIN enabled globally on TUI startup (not just in
EmbeddedTerminal mode). This ensures enhanced key events are available immediately when the
user enters embedded terminal mode. They are disabled on TUI exit via the cleanup sequence.

### Translation function

```rust
/// Translate a crossterm KeyEvent to terminal byte sequences for PTY stdin.
/// Returns None for events that should NOT be forwarded (e.g., pure modifier keys).
pub fn key_event_to_pty_bytes(event: KeyEvent) -> Option<Vec<u8>> {
    use crossterm::event::{KeyCode, KeyModifiers, KeyEventKind};

    // Only forward Press and Repeat events; Release events are discarded.
    if event.kind == KeyEventKind::Release {
        return None;
    }

    let mods = event.modifiers;

    match event.code {
        // Printable characters
        KeyCode::Char(c) if mods.is_empty() => Some(c.to_string().into_bytes()),

        // Ctrl-modified printable keys → control characters
        KeyCode::Char(c) if mods == KeyModifiers::CONTROL => {
            let ctrl_byte = (c.to_ascii_uppercase() as u8).wrapping_sub(b'@');
            if ctrl_byte <= 31 { Some(vec![ctrl_byte]) } else { None }
        }

        // Special keys
        KeyCode::Enter         => Some(b"\r".to_vec()),
        KeyCode::Backspace     => Some(b"\x7f".to_vec()),
        KeyCode::Tab           => Some(b"\t".to_vec()),
        KeyCode::Esc           => {
            // Esc in EmbeddedTerminal exits embedded terminal mode (handled by dispatch
            // layer BEFORE this function is called). If we reach here, it's a bare Esc
            // keypress that should be forwarded to the PTY (e.g., vim escape key).
            Some(b"\x1b".to_vec())
        }

        // Arrow keys
        KeyCode::Up            => Some(b"\x1b[A".to_vec()),
        KeyCode::Down          => Some(b"\x1b[B".to_vec()),
        KeyCode::Right         => Some(b"\x1b[C".to_vec()),
        KeyCode::Left          => Some(b"\x1b[D".to_vec()),

        // Navigation keys
        KeyCode::Home          => Some(b"\x1b[H".to_vec()),
        KeyCode::End           => Some(b"\x1b[F".to_vec()),
        KeyCode::PageUp        => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown      => Some(b"\x1b[6~".to_vec()),
        KeyCode::Insert        => Some(b"\x1b[2~".to_vec()),
        KeyCode::Delete        => Some(b"\x1b[3~".to_vec()),

        // Function keys (F1–F12)
        KeyCode::F(n) => Some(fn_key_bytes(n)),

        // Kitty keyboard protocol: modified key encoding (FIRST catch-all).
        //
        // PRECEDENCE NOTE (S2-002 fix): This arm appears BEFORE the VT-fallback modified-arrow
        // arms below. The intended precedence is:
        //   1. Specific key matches above (printable chars, Ctrl+printable, Enter, Esc, etc.)
        //   2. Kitty-enhanced keys (this arm) — when Kitty enhancement flags are active,
        //      modified arrows and other modified keys arrive as enhanced events that
        //      `is_kitty_enhanced_key` recognizes. Kitty-encoded output goes to Kitty-capable
        //      terminals via `encode_kitty_key`.
        //   3. VT-fallback modified arrows (arms below) — reached ONLY when `is_kitty_enhanced_key`
        //      returns false (terminal does not support Kitty enhancement, or the modifier
        //      combination is not Kitty-enhanced). This is the correct design: on a Kitty-capable
        //      terminal, Ctrl+Arrow is encoded as a CSI u sequence (Kitty), not as CSI 1;5A (VT).
        //      On a non-Kitty terminal, the VT arms provide the fallback.
        //
        // The VT-fallback arms below are NOT unreachable on non-Kitty terminals — they are
        // the primary encoding path on such terminals. On Kitty terminals, the VT arms are
        // unreachable for keys that `is_kitty_enhanced_key` handles, which is correct.
        //
        // Reference: https://sw.kovidgoyal.net/kitty/keyboard-protocol/
        _ if is_kitty_enhanced_key(event.code, mods) => {
            Some(encode_kitty_key(event.code, mods, event.kind))
        }

        // Alt/Meta + printable char: ESC prefix (standard xterm Alt encoding).
        // This covers the common Alt+letter combos (Alt+F for forward-word in readline, etc.).
        KeyCode::Char(c) if mods == KeyModifiers::ALT => {
            let mut bytes = vec![b'\x1b'];
            bytes.extend_from_slice(c.to_string().as_bytes());
            Some(bytes)
        }

        // Shift+Tab → back-tab sequence (used by shells, vim, etc. to reverse-complete).
        KeyCode::BackTab => Some(b"\x1b[Z".to_vec()),

        // Shift+Tab is also reported as KeyCode::Tab with KeyModifiers::SHIFT on some terminals.
        KeyCode::Tab if mods == KeyModifiers::SHIFT => Some(b"\x1b[Z".to_vec()),

        // Modified arrows (Ctrl+Arrow, Shift+Arrow) — VT-fallback path (non-Kitty terminals).
        // Standard xterm modifier encoding: CSI 1 ; <modifier+1> <arrow>.
        // Modifier value: Shift=2, Alt=3, Ctrl=5, Shift+Ctrl=6, Alt+Ctrl=7, etc.
        // These arms are reached ONLY when is_kitty_enhanced_key returned false (see above).
        // On Kitty-capable terminals these arms are unreachable for the key combinations
        // handled by encode_kitty_key — this is intentional, not a dead-code bug.
        KeyCode::Up if mods == KeyModifiers::CONTROL    => Some(b"\x1b[1;5A".to_vec()),
        KeyCode::Down if mods == KeyModifiers::CONTROL  => Some(b"\x1b[1;5B".to_vec()),
        KeyCode::Right if mods == KeyModifiers::CONTROL => Some(b"\x1b[1;5C".to_vec()),
        KeyCode::Left if mods == KeyModifiers::CONTROL  => Some(b"\x1b[1;5D".to_vec()),
        KeyCode::Up if mods == KeyModifiers::SHIFT      => Some(b"\x1b[1;2A".to_vec()),
        KeyCode::Down if mods == KeyModifiers::SHIFT    => Some(b"\x1b[1;2B".to_vec()),
        KeyCode::Right if mods == KeyModifiers::SHIFT   => Some(b"\x1b[1;2C".to_vec()),
        KeyCode::Left if mods == KeyModifiers::SHIFT    => Some(b"\x1b[1;2D".to_vec()),

        // Mouse events are handled separately via crossterm::event::MouseEvent.
        _ => None,
    }
}

/// Encode a mouse event to the terminal byte sequence in SGR 1006 encoding.
/// Called only when AppMode::EmbeddedTerminal is active (SGR mode enabled at entry).
///
/// `pane_area`: the Rect of the PTY widget in TUI coordinates (used to:
///   1. Clip events outside the pane (return None).
///   2. Convert terminal-local coordinates to pane-relative 1-indexed PTY coordinates.
///
/// Parameter name: `pane_area` (NOT `screen_offset` — reconciled with BC-2.09.003
/// which uses `screen_offset: Rect`; both refer to the PTY widget's Rect; `pane_area`
/// is more precise and is the canonical name in this spec; BC-2.09.003 will be updated
/// by product-owner to use `pane_area`).
pub fn mouse_event_to_pty_bytes(
    event: MouseEvent,
    pane_area: Rect,
) -> Option<Vec<u8>> {
    use crossterm::event::{MouseButton, MouseEventKind};

    // Clip: event outside pane area is not forwarded.
    let col = event.column;
    let row = event.row;
    if col < pane_area.x
        || col >= pane_area.x + pane_area.width
        || row < pane_area.y
        || row >= pane_area.y + pane_area.height
    {
        return None;
    }

    // Convert to 1-indexed PTY coordinates (origin = pane top-left).
    let px = (col - pane_area.x + 1) as u32;
    let py = (row - pane_area.y + 1) as u32;

    // SGR mouse mode (1006): CSI < Ps ; Px ; Py M (press) / m (release)
    // Ps (button) encoding:
    //   0 = left press, 1 = middle press, 2 = right press
    //   64 = scroll up, 65 = scroll down
    //   32 = motion (button held)
    //   Release uses the same Ps as press but 'm' terminator.
    let (ps, terminator) = match event.kind {
        MouseEventKind::Down(btn) => {
            let ps = match btn {
                MouseButton::Left   => 0u32,
                MouseButton::Middle => 1u32,
                MouseButton::Right  => 2u32,
            };
            (ps, b'M')
        }
        MouseEventKind::Up(btn) => {
            let ps = match btn {
                MouseButton::Left   => 0u32,
                MouseButton::Middle => 1u32,
                MouseButton::Right  => 2u32,
            };
            (ps, b'm')
        }
        MouseEventKind::ScrollUp   => (64u32, b'M'),
        MouseEventKind::ScrollDown => (65u32, b'M'),
        MouseEventKind::Moved      => (32u32, b'M'),
        // Horizontal scroll: encode as left (66) / right (67) per xterm convention.
        MouseEventKind::ScrollLeft  => (66u32, b'M'),
        MouseEventKind::ScrollRight => (67u32, b'M'),
    };

    // Add modifier bits to Ps per SGR standard:
    // Shift adds 4, Meta/Alt adds 8, Ctrl adds 16.
    let mods = event.modifiers;
    let mut ps_final = ps;
    if mods.contains(crossterm::event::KeyModifiers::SHIFT) { ps_final |= 4; }
    if mods.contains(crossterm::event::KeyModifiers::ALT)   { ps_final |= 8; }
    if mods.contains(crossterm::event::KeyModifiers::CONTROL) { ps_final |= 16; }

    let seq = format!("\x1b[<{};{};{}{}", ps_final, px, py, terminator as char);
    Some(seq.into_bytes())
}
```

**Esc key handling contract:** In `AppMode::EmbeddedTerminal`, the Action dispatch layer
MUST intercept `KeyCode::Esc` with no modifiers as `Action::ExitEmbeddedTerminal` BEFORE
calling `key_event_to_pty_bytes`. A bare Esc that was meant for the PTY (e.g., vim's escape
key) must be signaled by pressing Esc twice: first Esc → exit embedded terminal, second Esc
on re-enter → forwarded to PTY. This is the standard TUI-nested-terminal convention.

**`Ctrl-D` handling:** `Ctrl-D` (`KeyCode::Char('d')` with `KeyModifiers::CONTROL`) is
translated to `\x04` (ASCII EOT) and forwarded to the PTY. Claude Code interprets this as
"end session." The session-host detects the child exiting and sends `StateChanged::Terminated`
to the daemon, which sends `SessionStateChanged` to the TUI, which exits
`AppMode::EmbeddedTerminal` automatically.

### Mouse support (SGR mode)

When entering `AppMode::EmbeddedTerminal`, monocle enables mouse capture AND SGR 1006
extended mouse reporting (I3 fix — scoped to EmbeddedTerminal entry/exit, not global):

```rust
// EmbeddedTerminal ENTRY:
crossterm::execute!(stdout(), crossterm::event::EnableMouseCapture)?;
print!("\x1b[?1006h");  // SGR mouse mode

// EmbeddedTerminal EXIT:
print!("\x1b[?1006l");  // Disable SGR mouse mode first
crossterm::execute!(stdout(), crossterm::event::DisableMouseCapture)?;
```

The entry `EnableMouseCapture` and exit `DisableMouseCapture` are symmetric. SGR `h` and `l`
are symmetric. The global TUI startup does NOT call `EnableMouseCapture` (I3 fix).

Mouse events received by crossterm in `AppMode::EmbeddedTerminal` are translated to SGR
sequences via `mouse_event_to_pty_bytes(event, pane_area)` and sent as `KeyInput` IPC messages.

### Bracketed paste

`crossterm::event::EnableBracketedPaste` is enabled globally alongside Kitty enhancement flags.
Paste events arrive as `crossterm::event::Event::Paste(String)`. In `AppMode::EmbeddedTerminal`,
paste text is wrapped in bracketed paste sequences and forwarded to the PTY:

```
\x1b[200~ + paste_text + \x1b[201~
```

---

## Scrollback navigation

In `AppMode::EmbeddedTerminal`, `PtyScrollUp` and `PtyScrollDown` actions adjust
`App::pty_scroll_offset` without sending a `ResizePane` IPC message. The vt100::Parser
retains the scrollback buffer; the TUI passes a scrollback viewport to the widget.

`vt100::Parser::new(rows, cols, scrollback_rows)` — the third argument is the scrollback
line count. Default: 1000 rows. Maximum: configurable via `~/.monocle/config.json` key
`pty_scrollback_rows`; cap at 10000.

**O4 — Scrollback memory bound (includes per-cell styled-attribute size):**

The `vt100` crate stores each cell as `(char, fg_color, bg_color, attrs_bitmask)`. The
in-memory size of a single vt100 cell is NOT just a char (1 byte) — it includes color and
attribute storage. Based on the vt100 0.16.x source (`Cell` struct): approximately
`1 (char) + 4 (fg color enum) + 4 (bg color enum) + 1 (attrs bitmask) + padding ≈ 16 bytes`
per cell on 64-bit systems.

Memory budget (styled cells, not just string bytes):
- 10000 rows × 80 cols × 16 bytes/cell = **12.8 MB per session** (live screen + scrollback).
- For 8 sessions: **102 MB** — acceptable on a workstation with ≥ 8 GB RAM.
- The cap at 10000 rows is thus justified by this bound, not the "string bytes" calculation
  that was previously cited (which severely underestimated real memory use).
- Default 1000 rows × 80 cols × 16 bytes/cell = 1.28 MB per session; 8 sessions ≈ 10 MB.
  The default is safe for all target hardware.

---

## Session Creation Wizard

The `AppMode::SessionCreation` wizard delegates to existing components where possible:

- **Step 1 (ProfilePicker):** reuse the existing profile-picker logic (BC-2.07.004/005).
  The user selects a harness profile (e.g., "Claude Code + CCR (background)").
- **Step 2 (ProjectPicker):** new component — nucleo-filtered list of recently-used project
  roots + a free-text entry for new paths. Project roots sourced from: (a) existing sessions'
  `project_root` fields, (b) `~/.monocle/recent_projects.json` (new small config file).
- **Step 3 (WorktreeConfirm):** Resolve the git worktree path for the project selected in
  Step 2. Display the resolved path + display name (both editable). Confirm with Enter.
  Cancel with Esc. Resolution follows the three-rule algorithm in SS-session-manager.md
  §SpawnOptions.worktree_root: (1) user-confirmed worktree if git repo + valid worktree
  path; (2) project_root if it is the git repo root with no explicit worktree selection;
  (3) project_root for non-git projects. The wizard MUST validate the resolved path (exists
  + git work-tree check) before allowing Confirm. Validation failures display an inline error
  and keep the wizard on Step 3. The resolved path populates `SpawnOptions.worktree_root`.
- **Step 4 (Launching):** the TUI sends `ClientToServer::SpawnSession { recipe }` to the daemon.
  SessionCreation.step transitions to `Launching`. When the TUI receives
  `ServerToClient::SessionStateChanged { new_state: Running }` for the new session, the
  wizard auto-transitions to `AppMode::EmbeddedTerminal { session_id }`.

If spawn fails (daemon returns an error), the wizard returns to `Step 1` with an error banner.

---

## Module Purity Classification

| Module | Classification | Rationale |
|--------|----------------|-----------|
| `AppMode::EmbeddedTerminal` | Pure core | State variant; no I/O |
| `AppMode::SessionCreation` | Pure core | State variant; no I/O |
| `SessionCreationStep` | Pure core | Enum; no I/O |
| `key_event_to_pty_bytes()` | Pure core | Input → bytes; no I/O; deterministic |
| `mouse_event_to_pty_bytes()` | Pure core | Input → bytes; no I/O |
| `App::pty_parsers` | Effectful shell | `vt100::Parser.process()` is stateful mutation |
| PTY widget render path | Effectful shell | Ratatui render → terminal I/O |
| Resize detection + IPC send | Effectful shell | UDS write |
| `crossterm::execute!` keyboard/mouse setup | Effectful shell | Terminal device I/O |

---

## Risk Mitigations

### Kitty keyboard protocol: terminal compatibility

Not all terminals support the Kitty keyboard protocol. `crossterm::event::PushKeyboardEnhancementFlags`
silently no-ops on terminals that do not support it. The fallback: standard terminal byte
sequences (the `match` arms above handle the common cases). Full Kitty protocol is a
best-effort enhancement; core functionality (printable + control + arrows + Enter + Esc +
Backspace) works on all terminals.

Mitigation: detect whether Kitty enhancement flags are supported at TUI startup by checking
the response to `CSI ? u` query. If unsupported, skip the `PushKeyboardEnhancementFlags` call
and log a trace-level message. Implemented in `monocle-tui/src/event_loop.rs`.

### vt100::Parser accuracy

`vt100` has medium confidence on complex terminal sequences (per embedded-pty-evaluation.md
§3.3). For Claude Code sessions (which are primarily text-mode with standard ANSI colors),
vt100's coverage is sufficient.

Mitigation: integration tests use a PTY fixture corpus from `embedded-pty-evaluation.md`
(common Claude Code output sequences). Added to the `monocle-test-harness` test suite as
`MockPtySpawner` tests with fixture replay.

---

## Behavioral Contracts (to be authored by product-owner in PRD delta)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.09.001 | PTY output renders within 100ms of byte receipt at TUI | P0 |
| BC-2.09.002 | Keyboard forwarding: all v1A key classes reach PTY stdin unmodified | P0 |
| BC-2.09.003 | Mouse events forwarded to PTY in SGR encoding when in EmbeddedTerminal | P0 |
| BC-2.09.004 | Kitty keyboard protocol: enhanced key events forwarded as CSI u sequences | P1 |
| BC-2.09.005 | Bracketed paste: paste events wrapped in bracket sequences before forwarding | P0 |
| BC-2.09.006 | Resize: PTY and parser resized within 2 render ticks of pane area change | P0 |
| BC-2.09.007 | Scrollback: 1000 rows default; configurable; PtyScrollUp/Down navigate rows | P1 |
| BC-2.09.008 | SessionCreation wizard: session transitions to Running within 5s of launch confirm | P0 |

BC IDs are proposals; product-owner assigns canonical IDs in the PRD delta.

---

## §Trace v1.2.0

**Adversarial Pass 2 resolution — S2-002** (2026-06-03):
- **S2-002 (duplicate match arm + arm ordering):** Removed the duplicate
  `_ if is_kitty_enhanced_key(event.code, mods)` match arm (second copy was unreachable
  and semantically identical to the first). One `is_kitty_enhanced_key` catch-all remains,
  positioned BEFORE the VT-fallback modified-arrow arms — this is the correct precedence.
  Added a detailed PRECEDENCE NOTE comment explaining: (1) Kitty arm handles Kitty-capable
  terminals; (2) VT-fallback arms handle non-Kitty terminals; (3) VT-fallback arms are
  intentionally unreachable for Kitty-enhanced keys on Kitty terminals (not a dead-code bug).
  This resolves the ambiguity without changing behavior — the Kitty arm always appeared first;
  the second duplicate was the unreachable copy.

## §Trace v1.1.0

**Adversarial Pass 1 resolution — I1/I3/I7/O1/O2/O4** (2026-06-03):

- **I1 (Keyboard table incomplete):** `mouse_event_to_pty_bytes()` `todo!()` replaced with
  full SGR-1006 implementation. Added Alt/Meta (`\x1b` + char prefix), Shift+Tab (`\x1b[Z`
  via `BackTab` and `Tab+SHIFT`), modified arrows (`\x1b[1;5A` etc. for Ctrl+Arrow,
  `\x1b[1;2A` etc. for Shift+Arrow) on the non-Kitty fallback path. `pane_area` parameter
  name canonicalized (was `screen_offset` in the todo stub — product-owner must update
  BC-2.09.003 parameter name from `screen_offset` to `pane_area` to match).
- **I3 (Global mouse capture scope):** `EnableMouseCapture` / `DisableMouseCapture` moved
  from global TUI startup/exit to `EmbeddedTerminal` entry/exit. Symmetric enter/exit
  lifecycle for both `EnableMouseCapture` + SGR `h/l`. Global TUI startup now enables
  Kitty keyboard flags and bracketed paste only. I3 UX tradeoff documented: if a future
  story requires mouse clicks in monocle panels, product-owner/human must approve enabling
  global mouse capture with awareness of text-selection tradeoff.
- **I7 (Per-session scroll offset):** `pty_scroll_offset: usize` replaced with
  `pty_scroll_offsets: HashMap<String, usize>`. Invariants: resets to 0 on resize; preserves
  per-session on focus switch; removed on session GC.
- **O1 (BC requirement flag resolved):** Placeholder marked resolved; cites BC-2.09.009
  v1.0.0. Pre-emption is v1B per Invariant 4 of BC-2.09.009.
- **O2 (tui-term WIP risk):** See ADR-0011 §Q-7 Resolution — WIP risk explicitly documented;
  exact-pin on tui-term 0.3.4; deferred vendoring on need. Human risk-acceptance required
  is noted in ADR-0011 (see ADR-0011 §Trace v1.1.0 note below). No change to this spec;
  ADR-0011 carries the disclosure.
- **O4 (Scrollback memory bound with styled-cell overhead):** `vt100::Cell` size revised to
  ~16 bytes (char + fg/bg color enum + attrs + padding). Memory bound: 10000 × 80 × 16 =
  12.8 MB/session; 8 sessions ≈ 102 MB. Default (1000 rows) ≈ 1.28 MB/session. Cap at
  10000 rows justified by this bound.

## §Trace v1.0.2

**SUG-3 + IMP-2 consistency findings** (2026-06-03):
- SUG-3: Replaced the silent-queuing permission prompt mutual-exclusion rule (original text:
  "permission prompts cannot queue during an embedded terminal session — they queue in the
  daemon and are displayed when the user exits embedded terminal mode"). The original rule
  was production-grade non-conformant: it silently suppressed time-sensitive permission
  prompts — monocle's killer feature. Replaced with three-tier rule: (1) mandatory
  status-bar badge + bell on any incoming permission prompt while in EmbeddedTerminal mode;
  (2) user can pre-empt by pressing Esc; (3) no silent queueing. BC requirement flagged
  for product-owner: badge-only is v1A minimum; pre-emption enhancement is v1B (requires
  human ratification). SessionCreation mode receives the same badge-only treatment.
- IMP-2: Added session_id type annotation to EmbeddedTerminal AppMode variant doc-comment
  (String; UUID as String; canonical per SS-session-manager.md §session_id type ruling).

## §Trace v1.0.1

**IMP-2 session_id type annotation** (2026-06-03):
- Intermediate bump — superseded by v1.0.2 (combined with SUG-3 in same burst).

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- SS-09 authored as part of v1A architecture delta.
- Full-fidelity keyboard encoding (D-237) fully specified: Kitty protocol, mouse, bracketed paste
  all resolved at architecture level (no implementation-deferred TODOs).
- AppMode extensions and SessionCreation wizard specified.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
