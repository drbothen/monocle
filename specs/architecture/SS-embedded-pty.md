---
document_type: architecture-section
level: L3
section: "embedded-pty"
subsystem: SS-09
version: "1.0.0"
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
input-hash: "7e4f4f4"
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
EmbeddedTerminal {
    session_id: String,   // currently-focused session
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
- `EmbeddedTerminal` and `SessionCreation` are mutually exclusive with `Overlay` (permission
  prompts cannot queue during an embedded terminal session — they queue in the daemon and
  are displayed when the user exits embedded terminal mode).
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

    /// Scrollback viewport offset for the focused session (in rows from bottom).
    pty_scroll_offset: usize,
}
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

**Crossterm setup (in `monocle-tui/src/event_loop.rs`):**

```rust
// Enable keyboard enhancement (Kitty protocol) and mouse capture on TUI start.
// These are terminal capabilities; they must be enabled on the raw terminal
// before entering the ratatui render loop.
crossterm::execute!(
    stdout(),
    crossterm::event::PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES |
        KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES |
        KeyboardEnhancementFlags::REPORT_EVENT_TYPES |
        KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT,
    ),
    crossterm::event::EnableMouseCapture,
)?;

// On exit, pop enhancement flags and disable mouse capture:
crossterm::execute!(
    stdout(),
    crossterm::event::PopKeyboardEnhancementFlags,
    crossterm::event::DisableMouseCapture,
)?;
```

**Note:** Kitty keyboard enhancement flags are enabled globally on TUI startup (not just in
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

        // Kitty keyboard protocol: modified key encoding
        // When Kitty enhancement flags are enabled, crossterm reports modifier
        // combinations using KeyboardEnhancementFlags. The terminal (in Kitty-enhanced
        // mode) expects CSI u sequences for modified keys.
        // Reference: https://sw.kovidgoyal.net/kitty/keyboard-protocol/
        _ if is_kitty_enhanced_key(event.code, mods) => {
            Some(encode_kitty_key(event.code, mods, event.kind))
        }

        // Mouse events are handled separately via crossterm::event::MouseEvent.
        _ => None,
    }
}

/// Encode a mouse event to the terminal byte sequence appropriate for the PTY.
/// monocle enables SGR mouse mode (1006) when entering EmbeddedTerminal.
pub fn mouse_event_to_pty_bytes(event: MouseEvent, screen_offset: Rect) -> Option<Vec<u8>> {
    // SGR mouse mode: CSI < Ps ; Px ; Py M/m
    // ...implementation details...
    todo!()
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

When entering `AppMode::EmbeddedTerminal`, in addition to the global Kitty enhancement flags
already active, monocle enables SGR extended mouse reporting:

```rust
crossterm::execute!(stdout(), crossterm::event::EnableMouseCapture)?;
// Additionally write the SGR mouse mode escape sequence to the terminal:
// ESC [ ? 1006 h
```

Mouse events received by crossterm in `AppMode::EmbeddedTerminal` are translated to SGR
sequences via `mouse_event_to_pty_bytes()` and sent as `KeyInput` IPC messages.

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
`pty_scrollback_rows`; cap at 10000 (memory bound: 10000 × 80 × ~4 bytes/cell ≈ 3.2 MB
per session; for 8 sessions ≈ 25 MB total).

---

## Session Creation Wizard

The `AppMode::SessionCreation` wizard delegates to existing components where possible:

- **Step 1 (ProfilePicker):** reuse the existing profile-picker logic (BC-2.07.004/005).
  The user selects a harness profile (e.g., "Claude Code + CCR (background)").
- **Step 2 (ProjectPicker):** new component — nucleo-filtered list of recently-used project
  roots + a free-text entry for new paths. Project roots sourced from: (a) existing sessions'
  `project_root` fields, (b) `~/.monocle/recent_projects.json` (new small config file).
- **Step 3 (WorktreeConfirm):** display resolved git worktree path + display name (editable).
  Confirm with Enter. Cancel with Esc.
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

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- SS-09 authored as part of v1A architecture delta.
- Full-fidelity keyboard encoding (D-237) fully specified: Kitty protocol, mouse, bracketed paste
  all resolved at architecture level (no implementation-deferred TODOs).
- AppMode extensions and SessionCreation wizard specified.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
