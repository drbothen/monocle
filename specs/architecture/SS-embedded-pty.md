---
document_type: architecture-section
level: L3
section: "embedded-pty"
subsystem: SS-09
version: "1.7.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T00:00:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - specs/product-brief.md
  - specs/research/embedded-pty-evaluation.md
  - specs/architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md
  - specs/architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md
  - specs/architecture/SS-ipc.md
  - specs/architecture/SS-tui.md
input-hash: "042e8f5"
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
///
/// `launching_session_id` is `None` until the TUI receives
/// `ServerToClient::SpawnAck { session_id }` from the daemon (F-P41-IMP-001 resolution).
/// It is set in the `Launching` step and used to filter `SessionStateChanged` events
/// so the wizard only auto-advances on the session IT spawned (EC-303 deterministic filter).
/// It is cleared (set back to `None`) on wizard exit (success or cancellation).
SessionCreation {
    step: SessionCreationStep,
    prior: FocusSnapshot,
    /// The daemon-assigned session UUID, populated on receipt of `ServerToClient::SpawnAck`.
    /// `None` before SpawnAck is received (steps ProfilePicker/ProjectPicker/WorktreeConfirm)
    /// or after wizard exit. `Some(id)` during the `Launching` step.
    launching_session_id: Option<String>,
},

#[derive(Clone, PartialEq, Eq)]
pub enum SessionCreationStep {
    ProfilePicker,     // Step 1: select harness profile (reuses existing profile-picker logic)
    ProjectPicker,     // Step 2: select project root (fuzzy-filtered directory list)
    WorktreeConfirm,   // Step 3: confirm git worktree path + display name
    Launching,         // Step 4: waiting for SessionState::Running confirmation
}
```

<a id="state-machine-invariants"></a>
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

    /// Tracks which session IDs have received a `ScrollbackDumpComplete` in this TUI
    /// process lifetime. Used by `enter_embedded_terminal()` to decide whether to send
    /// `ClientToServer::AttachSession` (auto-attach mandate, I11-001).
    /// - Insert `session_id` on receipt of `ScrollbackDumpComplete`.
    /// - Remove on session GC (`SessionState::Terminated` + list removal) so a future
    ///   re-entry triggers a fresh dump for a restarted session.
    pty_dump_received: HashSet<String>,

    /// Per-session in-progress scrollback dump flag (ADR-0010 §TUI PtyOutput buffer).
    /// `true` while `ScrollbackChunk*` messages are being accumulated for `session_id`
    /// (i.e., from the first `ScrollbackChunk` until `ScrollbackDumpComplete` is received).
    /// Canonical type: `HashMap<String, bool>` keyed by `session_id`.
    /// MUST be set to `true` in `enter_embedded_terminal()` when `AttachSession` is sent
    /// (auto-attach path — live `PtyOutput` arrives before the dump completes; buffering
    /// MUST begin immediately, not on first chunk receipt).
    /// Set to `false` after replay on `ScrollbackDumpComplete`.
    /// See ADR-0010 §TUI PtyOutput buffer during dump; BC-2.05.011 Inv-6; BC-2.09.001 PC-6.
    dump_in_progress: HashMap<String, bool>,

    /// Per-session buffer for `ServerToClient::PtyOutput` bytes received while a scrollback
    /// dump is in progress for that session (ADR-0010 §TUI PtyOutput buffer).
    /// Canonical type: `HashMap<String, Vec<Vec<u8>>>` keyed by `session_id`.
    /// Each inner `Vec<u8>` is the raw bytes from one `PtyOutput` message, stored in receipt
    /// order. On `ScrollbackDumpComplete`: replay all buffered vecs through the freshly-reset
    /// `vt100::Parser` in order, then clear this buffer and set `dump_in_progress[id] = false`.
    /// See ADR-0010 §TUI PtyOutput buffer during dump; BC-2.05.011 Inv-6; BC-2.09.001 PC-6.
    pending_pty_bytes: HashMap<String, Vec<Vec<u8>>>,
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

**Blank-parser state for pre-existing sessions:** When the TUI starts fresh (new process) and
receives sessions via `InitialState.sessions` (or `SessionListUpdate`), the parsers for those
sessions start blank — they contain no screen history. This is correct for sessions that the
TUI has never yet displayed; the blank state becomes populated as PTY output arrives. However,
for an ALREADY-RUNNING session (one that has been producing output before this TUI process
started), the blank parser means the user would see an empty screen on first entry. The
auto-attach mandate (§EmbeddedTerminal ENTRY above) closes this gap: `enter_embedded_terminal()`
MUST trigger `AttachSession` for any session that has not yet received a scrollback dump in
this process lifetime.

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

<a id="full-fidelity-keyboard-encoding"></a>
## Full-Fidelity Keyboard Encoding (v1A scope — D-237 ratification)

Full keyboard fidelity is IN v1A scope (human-ratified at D-237, 2026-06-03). This section
is the authoritative implementation specification. No input class is deferred.

<a id="crossterm-setup"></a>
### Crossterm to PTY byte translation

When `AppMode::EmbeddedTerminal` is active, crossterm `KeyEvent` values are intercepted by
the Action dispatch layer before the standard keybinding lookup. They are translated to
terminal byte sequences and sent as `ClientToServer::KeyInput { session_id, bytes }`.

**Crossterm setup (in `monocle-tui/src/event_loop.rs`) — I3 fix:**

Keyboard enhancement (Kitty) flags are enabled GLOBALLY at TUI startup. `EnableMouseCapture`
is NOT global — it is scoped to `EmbeddedTerminal` entry/exit (per BC-2.09.002 Invariant-5)
precisely to avoid stealing mouse selection/copy from monocle's own panels.

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

**Auto-attach on first entry (I11-001 fix — normative):**

When `enter_embedded_terminal(session_id)` is called AND the TUI has not yet received a
`ScrollbackDumpComplete` for this `session_id` in the current process lifetime (i.e., the
`vt100::Parser` for this session was initialized blank by `SessionListUpdate` or `InitialState`
and has never been populated via a scrollback dump), the TUI MUST IMMEDIATELY send
`ClientToServer::AttachSession { session_id }` to the daemon.

```rust
// Auto-attach mandate: send AttachSession if this parser has never been
// populated from a scrollback dump in this process lifetime.
// S12-001 fix: set dump_in_progress = true HERE (not on first ScrollbackChunk receipt)
// because live PtyOutput may arrive before the first chunk — buffering must start
// immediately when AttachSession is sent.
if !app.pty_dump_received.contains(&session_id) {
    // Mark dump in progress BEFORE sending AttachSession so any PtyOutput that
    // arrives before the first ScrollbackChunk is captured in pending_pty_bytes.
    // The completed-set (pty_dump_received) is the "done" signal; dump_in_progress
    // is the "in-flight" signal — they serve different purposes and MUST NOT be conflated.
    app.dump_in_progress.insert(session_id.clone(), true);
    app.ipc_tx.send(ClientToServer::AttachSession {
        session_id: session_id.clone(),
    }).await.ok();
}
```

`App::pty_dump_received: HashSet<String>` tracks which session IDs have received a
`ScrollbackDumpComplete` in this TUI process lifetime. On receipt of `ScrollbackDumpComplete`
for a session, insert `session_id` into `pty_dump_received`. On session GC (`SessionState::Terminated`
plus list removal), remove from `pty_dump_received` so a future re-entry triggers a fresh dump.

`App::dump_in_progress: HashMap<String, bool>` is the in-flight signal: `true` from the moment
`AttachSession` is sent until `ScrollbackDumpComplete` is received and the buffer is replayed.
While `dump_in_progress[session_id] == true`, all incoming `ServerToClient::PtyOutput` for that
session MUST be appended to `App::pending_pty_bytes[session_id]` instead of fed to the parser.
On `ScrollbackDumpComplete`: (1) reset parser, (2) replay `pending_pty_bytes[session_id]` in
order, (3) clear the buffer, (4) set `dump_in_progress[session_id] = false`, (5) insert into
`pty_dump_received`. These are the **canonical ADR-0010 types** (`HashMap<String, bool>` and
`HashMap<String, Vec<Vec<u8>>>`); the App struct field list above is the authoritative
single-document definition. See ADR-0010 §TUI PtyOutput buffer during dump (canonical source).

**Rationale:** When the TUI starts fresh (new process) and connects to a daemon that has one
or more already-running sessions, it receives those sessions in `InitialState.sessions` (or
`SessionListUpdate`) and creates blank `vt100::Parser` instances for them. Without the
`AttachSession` trigger, selecting any of those pre-existing sessions in the TUI would show a
blank embedded terminal until the next PTY byte from the harness child — a blank screen for
an already-running session is silently wrong (the user sees no history or current state).
The `AttachSession` trigger causes the daemon to call `SessionManager::attach_session()` which
issues `DaemonToHost::Attach` to the session-host, triggering the `ScrollbackChunk*` +
`ScrollbackDumpComplete` sequence. The TUI then reconstructs the full terminal state
(per BC-2.09.001 Invariant 5, BC-2.05.011 §ScrollbackDumpComplete PC-3, and
SS-session-manager.md §Screen-state transfer on Attach). This is a v1A-critical guarantee:
sessions survive TUI close and daemon restart; a TUI opening an already-running session MUST
see the current terminal state immediately.

This mandate does NOT apply when transitioning from `SessionCreation::Launching` to
`EmbeddedTerminal` on a NEW session (the new session emits live `PtyOutput` from the start;
no historical screen state exists to restore).

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

---

<a id="dependency-boundary-f-p2-i06"></a>
### Dependency Boundary: monocle-core MUST NOT depend on crossterm or ratatui

**Ruling for F-P2-I06 (Phase-2 adversarial Pass-2, 2026-06-16):**

`monocle-core` carries an explicit architectural prohibition against crossterm and ratatui
dependencies. SS-tui.md §Scope states: "`monocle-core` — pure data types … No I/O, no ratatui,
no crossterm." This is the categorical rule; it is not nuanced to "no crossterm I/O only."
The prohibition also covers ratatui (whose `Rect` type would appear as a parameter of
`mouse_event_to_pty_bytes`). Existing precedents (nucleo, similar) confirm this: both are
"ONLY in monocle-tui — NEVER monocle-core (purity boundary)" per monocle-tui/Cargo.toml.

**Option A (feature-gated crossterm in monocle-core) is REJECTED** because:
1. SS-tui.md §Scope explicitly forbids it without exception.
2. A feature-flag crossterm dependency still means monocle-core unconditionally depends on
   crossterm when the feature is enabled — the purity boundary is violated in the binary.
3. Re-exports FROM monocle-tui into monocle-core would invert the dependency (monocle-core
   must never depend on monocle-tui).

**Canonical resolution: Option B — core-owned mirror types.** Define minimal mirror types in
`monocle-core` that carry exactly the fields the pure translation functions need. The
`monocle-tui` effectful shell converts crossterm/ratatui types to core-owned types at the
event dispatch site before calling into `monocle-core`. This is identical in structure to the
existing pattern: `AppMode`, `Action`, `PromptModal` are core-owned types constructed by
monocle-tui from TUI framework events.

#### Core-Owned Mirror Types (monocle-core/src/keyboard.rs)

```rust
// These types mirror crossterm/ratatui fields exactly, but live in monocle-core
// so key_event_to_pty_bytes and mouse_event_to_pty_bytes remain crossterm/ratatui-free.
// monocle-tui converts at the dispatch boundary (zero-cost field copy of primitives/enums).

/// Mirror of crossterm::event::KeyCode.
/// Variants cover only the v1A scope (BC-2.09.002 table). Add variants as BCs expand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyKeyCode {
    Char(char),
    Enter, Backspace, Tab, BackTab, Esc, Delete, Insert,
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,
    F(u8),
    // Extend as needed for future BCs.
    Null,
}

/// Mirror of crossterm::event::KeyModifiers (bitflags).
/// Matches crossterm bit values exactly so monocle-tui conversion is a single cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtyKeyModifiers(pub u8);

impl PtyKeyModifiers {
    pub const NONE:    Self = PtyKeyModifiers(0b0000_0000);
    pub const SHIFT:   Self = PtyKeyModifiers(0b0000_0001);
    pub const CONTROL: Self = PtyKeyModifiers(0b0000_0100);
    pub const ALT:     Self = PtyKeyModifiers(0b0000_1000);

    pub fn contains(self, other: Self) -> bool { self.0 & other.0 != 0 }
    pub fn is_empty(self) -> bool { self.0 == 0 }
}

/// Mirror of crossterm::event::KeyEventKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyKeyEventKind { Press, Repeat, Release }

/// Mirror of crossterm::event::KeyEvent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyKeyEvent {
    pub code:      PtyKeyCode,
    pub modifiers: PtyKeyModifiers,
    pub kind:      PtyKeyEventKind,
}

/// Mirror of crossterm::event::MouseButton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyMouseButton { Left, Middle, Right }

/// Mirror of crossterm::event::MouseEventKind (v1A Ps table scope).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyMouseEventKind {
    Down(PtyMouseButton),
    Up(PtyMouseButton),
    Drag(PtyMouseButton),
    Moved,
    ScrollUp, ScrollDown, ScrollLeft, ScrollRight,
}

/// Mirror of crossterm::event::MouseEvent fields used by mouse_event_to_pty_bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyMouseEvent {
    pub kind:      PtyMouseEventKind,
    pub column:    u16,
    pub row:       u16,
    pub modifiers: PtyKeyModifiers,  // reuse PtyKeyModifiers; same bit layout
}

/// Minimal pane area rectangle — mirrors ratatui::layout::Rect fields.
/// monocle-core does NOT depend on ratatui. monocle-tui converts Rect → PtyRect
/// at the event dispatch site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtyRect {
    pub x:      u16,
    pub y:      u16,
    pub width:  u16,
    pub height: u16,
}
```

#### Conversion in monocle-tui (effectful shell boundary)

These conversions live in `crates/monocle-tui/src/keyboard_conv.rs` (new file, S-040 scope).
They are infallible field-by-field copies. No logic; no I/O.

```rust
// monocle-tui/src/keyboard_conv.rs
//
// Converts crossterm/ratatui types → monocle-core PtyKey*/PtyMouse*/PtyRect.
// Called at the EmbeddedTerminal event dispatch site in event_loop.rs before
// calling into monocle-core::keyboard functions.
//
// This is the ONLY place in the workspace where crossterm types touch the
// monocle-core purity boundary. Adding any crossterm or ratatui type to
// monocle-core/Cargo.toml is FORBIDDEN (SS-tui.md §Scope, F-P2-I06 ruling).

use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind,
                        MouseEvent, MouseEventKind, MouseButton};
use ratatui::layout::Rect;
use monocle_core::keyboard::{
    PtyKeyEvent, PtyKeyCode, PtyKeyModifiers, PtyKeyEventKind,
    PtyMouseEvent, PtyMouseEventKind, PtyMouseButton, PtyRect,
};

pub fn crossterm_key_to_pty(e: KeyEvent) -> PtyKeyEvent {
    PtyKeyEvent {
        code:      crossterm_keycode_to_pty(e.code),
        modifiers: crossterm_mods_to_pty(e.modifiers),
        kind:      crossterm_kind_to_pty(e.kind),
    }
}

pub fn crossterm_mouse_to_pty(e: MouseEvent) -> PtyMouseEvent {
    PtyMouseEvent {
        kind:      crossterm_mouse_kind_to_pty(e.kind),
        column:    e.column,
        row:       e.row,
        modifiers: crossterm_mods_to_pty(e.modifiers),
    }
}

pub fn ratatui_rect_to_pty(r: Rect) -> PtyRect {
    PtyRect { x: r.x, y: r.y, width: r.width, height: r.height }
}

fn crossterm_keycode_to_pty(c: KeyCode) -> PtyKeyCode {
    match c {
        KeyCode::Char(ch)  => PtyKeyCode::Char(ch),
        KeyCode::Enter     => PtyKeyCode::Enter,
        KeyCode::Backspace => PtyKeyCode::Backspace,
        KeyCode::Tab       => PtyKeyCode::Tab,
        KeyCode::BackTab   => PtyKeyCode::BackTab,
        KeyCode::Esc       => PtyKeyCode::Esc,
        KeyCode::Delete    => PtyKeyCode::Delete,
        KeyCode::Insert    => PtyKeyCode::Insert,
        KeyCode::Up        => PtyKeyCode::Up,
        KeyCode::Down      => PtyKeyCode::Down,
        KeyCode::Left      => PtyKeyCode::Left,
        KeyCode::Right     => PtyKeyCode::Right,
        KeyCode::Home      => PtyKeyCode::Home,
        KeyCode::End       => PtyKeyCode::End,
        KeyCode::PageUp    => PtyKeyCode::PageUp,
        KeyCode::PageDown  => PtyKeyCode::PageDown,
        KeyCode::F(n)      => PtyKeyCode::F(n),
        _                  => PtyKeyCode::Null,  // unrecognized → None from key_event_to_pty_bytes
    }
}

fn crossterm_mods_to_pty(m: KeyModifiers) -> PtyKeyModifiers {
    let mut bits = 0u8;
    if m.contains(KeyModifiers::SHIFT)   { bits |= PtyKeyModifiers::SHIFT.0; }
    if m.contains(KeyModifiers::CONTROL) { bits |= PtyKeyModifiers::CONTROL.0; }
    if m.contains(KeyModifiers::ALT)     { bits |= PtyKeyModifiers::ALT.0; }
    PtyKeyModifiers(bits)
}

fn crossterm_kind_to_pty(k: KeyEventKind) -> PtyKeyEventKind {
    match k {
        KeyEventKind::Press   => PtyKeyEventKind::Press,
        KeyEventKind::Repeat  => PtyKeyEventKind::Repeat,
        KeyEventKind::Release => PtyKeyEventKind::Release,
    }
}

fn crossterm_mouse_kind_to_pty(k: MouseEventKind) -> PtyMouseEventKind {
    match k {
        MouseEventKind::Down(b)   => PtyMouseEventKind::Down(cvt_btn(b)),
        MouseEventKind::Up(b)     => PtyMouseEventKind::Up(cvt_btn(b)),
        MouseEventKind::Drag(b)   => PtyMouseEventKind::Drag(cvt_btn(b)),
        MouseEventKind::Moved     => PtyMouseEventKind::Moved,
        MouseEventKind::ScrollUp  => PtyMouseEventKind::ScrollUp,
        MouseEventKind::ScrollDown => PtyMouseEventKind::ScrollDown,
        MouseEventKind::ScrollLeft => PtyMouseEventKind::ScrollLeft,
        MouseEventKind::ScrollRight => PtyMouseEventKind::ScrollRight,
    }
}

fn cvt_btn(b: MouseButton) -> PtyMouseButton {
    match b {
        MouseButton::Left   => PtyMouseButton::Left,
        MouseButton::Middle => PtyMouseButton::Middle,
        MouseButton::Right  => PtyMouseButton::Right,
    }
}
```

#### Call site in event_loop.rs

```rust
// monocle-tui/src/event_loop.rs — EmbeddedTerminal dispatch arm

use crate::keyboard_conv::{crossterm_key_to_pty, crossterm_mouse_to_pty, ratatui_rect_to_pty};
use monocle_core::keyboard::{key_event_to_pty_bytes, mouse_event_to_pty_bytes};

// Key events:
Event::Key(crossterm_key_event) if app_mode == EmbeddedTerminal => {
    // Esc interception BEFORE conversion (per BC-2.09.002 Invariant 2).
    if crossterm_key_event.code == crossterm::event::KeyCode::Esc
       && crossterm_key_event.modifiers.is_empty()
    {
        dispatch(Action::ExitEmbeddedTerminal);
    } else {
        let pty_event = crossterm_key_to_pty(crossterm_key_event);
        if let Some(bytes) = key_event_to_pty_bytes(pty_event) {
            ipc_tx.send(ClientToServer::KeyInput { session_id, bytes }).await.ok();
        }
    }
}

// Mouse events:
Event::Mouse(crossterm_mouse_event) if app_mode == EmbeddedTerminal => {
    let pty_event = crossterm_mouse_to_pty(crossterm_mouse_event);
    let pane      = ratatui_rect_to_pty(app.last_pty_pane_area);
    if let Some(bytes) = mouse_event_to_pty_bytes(pty_event, pane) {
        ipc_tx.send(ClientToServer::KeyInput { session_id, bytes }).await.ok();
    }
}
```

**Consequence for S-040 and S-041 tasks:** The Task lists in both stories specify
`key_event_to_pty_bytes(event: KeyEvent)` and `mouse_event_to_pty_bytes(event: MouseEvent, pane_area: Rect)`
as the function signatures. **These signatures are incorrect as stated.** The correct signatures are:

```rust
// monocle-core/src/keyboard.rs — canonical signatures (F-P2-I06 ruling)
pub fn key_event_to_pty_bytes(event: PtyKeyEvent) -> Option<Vec<u8>>
pub fn is_kitty_enhanced_key(code: &PtyKeyCode, mods: PtyKeyModifiers) -> bool
pub fn encode_kitty_key(code: &PtyKeyCode, mods: PtyKeyModifiers, kind: PtyKeyEventKind) -> Vec<u8>
pub fn mouse_event_to_pty_bytes(event: PtyMouseEvent, pane_area: PtyRect) -> Option<Vec<u8>>
```

Story-writer must align S-040 and S-041 task lists and Architecture Compliance Rules sections
to use `PtyKeyEvent`, `PtyMouseEvent`, `PtyRect` and to reference the conversion module
`monocle-tui/src/keyboard_conv.rs`.

---

### Translation function

```rust
/// Translate a PtyKeyEvent to terminal byte sequences for PTY stdin.
/// Returns None for events that should NOT be forwarded (e.g., pure modifier keys).
/// Parameter is a PtyKeyEvent (core-owned type, not crossterm::KeyEvent).
/// monocle-tui converts crossterm::KeyEvent → PtyKeyEvent at the dispatch boundary
/// via keyboard_conv::crossterm_key_to_pty() before calling this function.
/// See §Dependency Boundary above (F-P2-I06).
pub fn key_event_to_pty_bytes(event: PtyKeyEvent) -> Option<Vec<u8>> {
    use crate::keyboard::{PtyKeyCode, PtyKeyModifiers, PtyKeyEventKind};

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
/// `event`: PtyMouseEvent (core-owned type, not crossterm::MouseEvent).
/// `pane_area`: PtyRect (core-owned type, not ratatui::layout::Rect).
/// monocle-tui converts crossterm::MouseEvent → PtyMouseEvent and Rect → PtyRect
/// at the dispatch boundary via keyboard_conv functions. See §Dependency Boundary (F-P2-I06).
///
/// `pane_area`: PTY widget area in TUI coordinates (used to:
///   1. Clip events outside the pane (return None).
///   2. Convert terminal-local coordinates to pane-relative 1-indexed PTY coordinates.
///
/// Parameter name: `pane_area` (canonical name per BC-2.09.003 §X; `screen_offset: Rect`
/// was the pre-Pass-1 stub name and is retired).
pub fn mouse_event_to_pty_bytes(
    event: PtyMouseEvent,
    pane_area: PtyRect,
) -> Option<Vec<u8>> {
    use crate::keyboard::{PtyMouseButton, PtyMouseEventKind};

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

    // SGR mouse mode (1006): CSI < Ps ; Px ; Py M (press/motion) / m (release)
    //
    // Mouse tracking mode: crossterm's EnableMouseCapture enables button-event tracking
    // (xterm mode 1002) + SGR encoding (1006). It does NOT enable any-event tracking
    // (mode 1003). The additional explicit `\x1b[?1006h` write at EmbeddedTerminal entry
    // ensures SGR mode is active regardless of crossterm internals.
    //
    // Consequence for Moved: in mode 1002 (no 1003), terminals do NOT report no-button
    // motion events on Unix. MouseEventKind::Moved is therefore unreachable on Unix in
    // EmbeddedTerminal. It may be reachable on Windows (WinAPI console mouse input).
    // The arm is retained for exhaustiveness and encoded correctly as 35 (3+32).
    //
    // Complete Ps encoding table (base values before modifier bits):
    //   Buttons:   0 = left,   1 = middle,  2 = right   (Down/Up)
    //   Drag:     32 = left,  33 = middle, 34 = right   (Drag; btn_base + 32)
    //   Motion:   35 = no-button motion                  (Moved; 3+32; unreachable on Unix)
    //   Scroll:   64 = up, 65 = down, 66 = left, 67 = right
    //   Modifier bits added to Ps: Shift|=4, Alt|=8, Ctrl|=16
    //   Terminator: 'M' for press/drag/scroll/motion, 'm' for release
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
        // Drag: button held + motion. Ps = button_base + 32.
        // Delivered by 1002 button-event tracking when the mouse moves while a button is pressed.
        MouseEventKind::Drag(btn) => {
            let ps = match btn {
                MouseButton::Left   => 32u32,  // 0 + 32
                MouseButton::Middle => 33u32,  // 1 + 32
                MouseButton::Right  => 34u32,  // 2 + 32
            };
            (ps, b'M')
        }
        MouseEventKind::ScrollUp   => (64u32, b'M'),
        MouseEventKind::ScrollDown => (65u32, b'M'),
        // Moved: no-button motion. Ps = 3 + 32 = 35.
        // Unreachable on Unix in 1002 mode (no 1003); retained for exhaustiveness + Windows.
        MouseEventKind::Moved      => (35u32, b'M'),
        // Horizontal scroll: encode as left (66) / right (67) per xterm convention.
        MouseEventKind::ScrollLeft  => (66u32, b'M'),
        MouseEventKind::ScrollRight => (67u32, b'M'),
    };

    // Add modifier bits to Ps per SGR standard:
    // Shift adds 4, Meta/Alt adds 8, Ctrl adds 16.
    let mods = event.modifiers;
    let mut ps_final = ps;
    if mods.contains(PtyKeyModifiers::SHIFT)   { ps_final |= 4; }
    if mods.contains(PtyKeyModifiers::ALT)     { ps_final |= 8; }
    if mods.contains(PtyKeyModifiers::CONTROL) { ps_final |= 16; }

    let seq = format!("\x1b[<{};{};{}{}", ps_final, px, py, terminator as char);
    Some(seq.into_bytes())
}
```

<a id="esc-key-handling-contract"></a>
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
`App::pty_scroll_offsets[focused_session_id]` without sending a `ResizePane` IPC message.
`pty_scroll_offsets` is a `HashMap<String, usize>` keyed by `session_id` (I7 fix — was
a single shared `pty_scroll_offset: usize` before v1.1.0). The vt100::Parser retains the
scrollback buffer; the TUI passes the per-session scrollback viewport offset to the widget.

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
- **Step 4 (Launching):** the TUI sends `ClientToServer::SpawnSession { opts }` to the daemon,
  where `opts` is a `SpawnOptions` populated from the wizard steps (project_root, worktree_root,
  harness_id, profile_id, ccr_base_url). `SessionCreation.step` transitions to `Launching`.
  The daemon IPC handler (F-P41-IMP-001 resolution):
    1. Generates the session UUID.
    2. Sends `ServerToClient::SpawnAck { session_id }` to the requesting TUI client only.
    3. Calls `SessionManager::spawn_session(opts)` (daemon fills session_id and
       hooks_settings_path daemon-side; I27-001 Model A — SpawnRecipe is built in spawn_session()).
  On receipt of `SpawnAck { session_id }`, the TUI stores the id in
  `AppMode::SessionCreation { launching_session_id: Some(session_id.clone()), .. }`.
  This gives the wizard a deterministic session_id BEFORE any `SessionStateChanged { Launching }`
  broadcast arrives. When the TUI subsequently receives
  `ServerToClient::SessionStateChanged { session_id: <matching>, new_state: Running }`,
  the wizard matches against `launching_session_id` (not a broadcast-race heuristic) and
  auto-transitions to `AppMode::EmbeddedTerminal { session_id, prior: Dashboard }`.

If spawn fails (daemon returns `ServerToClient::Error`), the wizard clears `launching_session_id` to `None` and returns to `ProfilePicker` with an error banner.

---

## Module Purity Classification

| Module | Classification | Rationale |
|--------|----------------|-----------|
| `AppMode::EmbeddedTerminal` | Pure core | State variant; no I/O |
| `AppMode::SessionCreation` | Pure core | State variant; no I/O (F-P41-IMP-001: `launching_session_id: Option<String>` field added — still pure; in-memory only) |
| `SessionCreationStep` | Pure core | Enum; no I/O |
| `PtyKeyEvent`, `PtyKeyCode`, `PtyKeyModifiers`, `PtyKeyEventKind` | Pure core | Core-owned mirror types; no crossterm dep in monocle-core (F-P2-I06) |
| `PtyMouseEvent`, `PtyMouseEventKind`, `PtyMouseButton`, `PtyRect` | Pure core | Core-owned mirror types; no ratatui dep in monocle-core (F-P2-I06) |
| `key_event_to_pty_bytes(PtyKeyEvent)` | Pure core | Input → bytes; no I/O; deterministic |
| `mouse_event_to_pty_bytes(PtyMouseEvent, PtyRect)` | Pure core | Input → bytes; no I/O |
| `keyboard_conv::crossterm_key_to_pty()` | Effectful shell (boundary) | crossterm→core conversion; lives in monocle-tui; calls only data copies |
| `keyboard_conv::crossterm_mouse_to_pty()` | Effectful shell (boundary) | crossterm→core conversion; lives in monocle-tui |
| `keyboard_conv::ratatui_rect_to_pty()` | Effectful shell (boundary) | Rect→PtyRect conversion; lives in monocle-tui |
| `App::pty_parsers` | Effectful shell | `vt100::Parser.process()` is stateful mutation |
| `App::dump_in_progress` | Pure core | `HashMap<String, bool>` flag; in-memory state only |
| `App::pending_pty_bytes` | Pure core | `HashMap<String, Vec<Vec<u8>>>` buffer; in-memory accumulation only |
| `App::pty_dump_received` | Pure core | `HashSet<String>` completed-set; in-memory only |
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

## §Trace v1.7.0

**F-P2-I06 ruling — monocle-core ↔ crossterm dependency boundary: canonical resolution** (2026-06-16):

- **Finding (F-P2-I06, Phase-2 adversarial Pass-2):** S-040 contained a contradictory
  statement: `key_event_to_pty_bytes(event: KeyEvent)` was specified to live in `monocle-core`
  AND monocle-core was stated to be forbidden from depending on crossterm directly. The note
  "only via feature flags or re-exports from monocle-tui" is architecturally incoherent:
  re-exports from monocle-tui would invert the dependency (core cannot depend on tui); feature
  flags on monocle-core for crossterm still violate the categorical "no crossterm" rule in
  SS-tui.md §Scope. The same problem extended to `mouse_event_to_pty_bytes(event: MouseEvent,
  pane_area: Rect)` which would require BOTH a crossterm AND a ratatui dep in monocle-core.
- **Decision: Option B — core-owned mirror types.** Define `PtyKeyEvent`, `PtyKeyCode`,
  `PtyKeyModifiers`, `PtyKeyEventKind`, `PtyMouseEvent`, `PtyMouseEventKind`, `PtyMouseButton`,
  `PtyRect` in `monocle-core/src/keyboard.rs`. These are pure data structs/enums with no
  external crate dependencies — identical in design to how `AppMode`, `Action`, and `PromptModal`
  are core-owned types that monocle-tui constructs from TUI framework events.
- **Conversion module:** `monocle-tui/src/keyboard_conv.rs` provides `crossterm_key_to_pty()`,
  `crossterm_mouse_to_pty()`, and `ratatui_rect_to_pty()`. These are infallible field-by-field
  copies. All crossterm and ratatui type references are confined to monocle-tui (the effectful
  shell). monocle-core/Cargo.toml gains NO new dependencies.
- **Canonical function signatures in monocle-core:**
  `pub fn key_event_to_pty_bytes(event: PtyKeyEvent) -> Option<Vec<u8>>`
  `pub fn mouse_event_to_pty_bytes(event: PtyMouseEvent, pane_area: PtyRect) -> Option<Vec<u8>>`
  `pub fn is_kitty_enhanced_key(code: &PtyKeyCode, mods: PtyKeyModifiers) -> bool`
  `pub fn encode_kitty_key(code: &PtyKeyCode, mods: PtyKeyModifiers, kind: PtyKeyEventKind) -> Vec<u8>`
- **Story impact:** Story-writer must align S-040 and S-041 (Architecture Compliance Rules,
  task list function signatures, File Structure table) to use core-owned types and reference
  the `keyboard_conv` conversion module. The module purity table in this spec is updated.
- **monocle-core Cargo.toml unchanged:** No new deps. `keyboard.rs` uses only Rust primitives.
- Semver: minor (v1.6.0 → v1.7.0) — new §Dependency Boundary section, new core-owned type
  definitions, corrected function signatures, new `keyboard_conv` module specification.

## §Trace v1.6.0

**F-P41-IMP-001 resolution — `AppMode::SessionCreation` struct field + SpawnAck wizard wiring** (2026-06-14):

- **Finding (F-P41-IMP-001, IMPORTANT):** Two defects in the SessionCreation wizard auto-advance design:
  1. `AppMode::SessionCreation` had no `launching_session_id` field, making `BC-2.08.008 PC-5`'s
     destructure `{ step: Launching, session_id }` uncompilable against the canonical struct.
  2. The wizard had no deterministic mechanism to learn which `session_id` the daemon assigned —
     `SessionStateChanged { Launching }` is broadcast to ALL clients with no correlation token,
     creating a race on multiple simultaneous spawns.
- **Decision:** Mechanism (b) — `ServerToClient::SpawnAck { session_id }` sent to the requesting
  client only (not broadcast), generated in the IPC handler before `spawn_session()` is called.
  Rationale: mechanism (a) (broadcast-race heuristic) violates the production-grade principle —
  "usually works" on the single-TUI v1A path is not acceptable when a deterministic path is
  available. `SpawnAck` is zero-cost (no extra round-trip; it is sent in the same IPC handler
  `match` arm before the `spawn_session()` call), carries the exact assigned UUID, and gives the
  wizard a deterministic session_id with guaranteed ordering (per-client FIFO channel delivers
  `SpawnAck` before any broker-published `SessionStateChanged { Launching }`).
- **Fix (a) — `AppMode::SessionCreation` struct extended:** Added
  `launching_session_id: Option<String>` field. `None` during ProfilePicker/ProjectPicker/
  WorktreeConfirm steps; `Some(id)` after `SpawnAck` is received in the `Launching` step;
  `None` again on wizard exit (success or cancellation).
- **Fix (b) — Step 4 (Launching) prose updated:** Describes the 3-step IPC handler sequence
  (UUID generation → SpawnAck to requesting client → spawn_session()) and the TUI's obligation
  to store the id in `launching_session_id`. Wizard auto-advance now references
  `launching_session_id` as the filter (replaces the implicit broadcast-race model that
  EC-303 described without a mechanism).
- **Fix (c) — Module Purity table:** `AppMode::SessionCreation` row note updated (still Pure
  core; `launching_session_id: Option<String>` is in-memory state).
- **PO work-list:** See §Trace note — product-owner must update BC-2.08.008 PC-5, EC-303;
  BC-2.09.008 PC Step 4/Step 5; BC-2.08.001 PC-1 UUID-locus wording.
- Semver: minor (v1.5.2 → v1.6.0) — new struct field + new normative mechanism in wizard prose.

**CV-SS-003 + CV-SS-004 errata — §Session Creation Wizard Step 4 spawn-fail path** (2026-06-14):

- **Finding (CV-SS-003, IMPORTANT):** The spawn-fail sentence at §Session Creation Wizard Step 4
  said "the wizard returns to `Step 1` with an error banner." The canonical term across this feature
  (BC-2.09.008 §Postconditions item 7, BC-2.08.008 EC-303, SS-ipc line ~591-594, SS-session-manager
  lines ~548-549) is the named step `ProfilePicker`, not the numeric label "Step 1".
- **Fix (CV-SS-003):** "Step 1" → "`ProfilePicker`".
- **Finding (CV-SS-004, IMPORTANT):** The same sentence omitted the normative obligation to clear
  `launching_session_id` to `None` before returning to ProfilePicker. This obligation is canonical
  in BC-2.09.008 §Postconditions (entering SessionCreation) item 7 ("wizard clears `launching_session_id`
  to `None` and returns to `ProfilePicker` with an error banner"), BC-2.08.008 EC-303, SS-ipc
  lines ~591-594 (`The TUI MUST clear AppMode::SessionCreation.launching_session_id (set to None) on
  receipt of ServerToClient::Error`), and SS-session-manager lines ~548-549. SS-embedded-pty.md was
  the primary TUI spec site omitting this obligation.
- **Fix (CV-SS-004):** Spawn-fail sentence rewritten to: "If spawn fails (daemon returns
  `ServerToClient::Error`), the wizard clears `launching_session_id` to `None` and returns to
  `ProfilePicker` with an error banner." Also precised "daemon returns an error" → "daemon returns
  `ServerToClient::Error`" for unambiguous cross-reference.
- **Semver:** Errata-no-bump for both CV-SS-003 and CV-SS-004. CV-SS-003 is a terminology
  reconciliation (named step replaces numeric label; no behavioral change). CV-SS-004 adds
  normative text to SS-embedded-pty.md, but the obligation is already canonical in the BC set and
  SS-ipc/SS-session-manager — this is a cross-doc completeness reconciliation, NOT invention of a
  new obligation. No POL-11 Architecture-Source propagation cascade triggered. Version remains v1.6.0.

## §Trace v1.5.2

**S39-001 errata — prose↔code contradiction at §Crossterm setup intro sentence** (2026-06-14):

Descriptive sentence at §Crossterm setup (formerly line 250) incorrectly called `EnableMouseCapture`
"global" — directly contradicting the code block it introduces (which states "NO global mouse
capture" and "Mouse capture is deferred to EmbeddedTerminal entry"). Rewritten to be accurate:
keyboard enhancement (Kitty) flags are enabled GLOBALLY at TUI startup; `EnableMouseCapture` is
NOT global — it is scoped to `EmbeddedTerminal` entry/exit (per BC-2.09.002 Invariant-5).
No normative content changed; prose-only errata. Semver: no bump (v1.5.2 retained).

**S35-003 — `mouse_event_to_pty_bytes`: add `Drag(btn)` arm, fix `Moved` Ps (32→35), document mouse tracking mode and full Ps table** (D-277, 2026-06-13):

- **Finding S35-003a — missing `Drag(MouseButton)` arm:** The match on `MouseEventKind` covered `Down/Up/ScrollUp/ScrollDown/Moved/ScrollLeft/ScrollRight` but had NO `Drag(MouseButton)` arm. `Drag` is a non-`#[non_exhaustive]` variant of `MouseEventKind` — omitting it is a compile error (non-exhaustive match). Additionally, `Drag` is the primary motion event delivered under xterm 1002 button-event tracking (motion while a button is held), so it is functionally critical. **Fix:** Added `MouseEventKind::Drag(btn)` arm encoding Ps = btn_base + 32 (left=32, middle=33, right=34) with terminator `M`.
- **Finding S35-003b — incorrect `Moved` Ps (32 instead of 35):** `MouseEventKind::Moved => (32u32, b'M')` encoded no-button motion as Ps=32, which is left-button drag (0+32). Correct SGR encoding for no-button motion is Ps = 3+32 = 35. **Fix:** Changed to `(35u32, b'M')`.
- **Finding S35-003c — undocumented mouse tracking mode:** The spec did not state which xterm mouse modes `EnableMouseCapture` activates, creating ambiguity about which `MouseEventKind` variants are reachable. **Fix:** Added normative comment: `EnableMouseCapture` enables 1002 (button-event) + 1006 (SGR), NOT 1003 (any-event). Consequence: `Moved` is unreachable on Unix in 1002 mode (no no-button motion reports); `Drag` is the only motion variant. `Moved` arm retained for exhaustiveness and Windows correctness.
- **Complete Ps + modifier table added to spec:** Base: Down/Up: 0/1/2; Drag: 32/33/34; Moved: 35; Scroll: 64/65/66/67. Modifier bits: Shift|=4, Alt|=8, Ctrl|=16. Terminator: M for press/drag/scroll/motion, m for release.
- **BC mirror required:** PO to update BC-2.09.003 PC-2/Invariant-4 to enumerate the complete Ps table including Drag Ps values and the corrected Moved Ps (35), and to note that `Moved` is unreachable on Unix under the enabled tracking modes.
- Semver: patch (v1.5.1 → v1.5.2) — correctness fix (missing Drag arm was a compile error; Moved Ps was wrong) with no change to the externally visible SGR byte sequences for Down/Up/Scroll variants.

## §Trace v1.5.1

**I27-001 — Step 4 (Launching) wizard prose: `SpawnSession { recipe }` → `SpawnSession { opts }`** (2026-06-13):

- **Finding (I27-001 propagation):** §SessionCreation wizard Step 4 stated "the TUI sends `ClientToServer::SpawnSession { recipe }` to the daemon" — using the old `SpawnRecipe` wire payload. Under Model A (I27-001), the wire payload is `SpawnOptions` (user intent), not a pre-built `SpawnRecipe`.
- **Fix:** Step 4 prose updated: `{ recipe }` → `{ opts }` with `SpawnOptions` context (fields from wizard steps; daemon fills session_id and hooks_settings_path on receipt; SpawnRecipe built daemon-side in spawn_session()).
- Semver: patch (v1.5.0 → v1.5.1) — prose correction to match wire-type change; no new behavioral spec.

## §Trace v1.5.0

**S12-001 — App struct `dump_in_progress` + `pending_pty_bytes` + `pty_dump_received` added; auto-attach skeleton made normative** (2026-06-04):

- **Finding (S12-001):** The App struct field list (§Parser ownership in TUI) defined `pty_parsers`
  and `pty_scroll_offsets` but NOT `pty_dump_received`, `dump_in_progress`, or `pending_pty_bytes`.
  These fields were mentioned in the auto-attach skeleton (~line 270) as comments or prose but
  were absent from the authoritative struct definition, making the spec not single-document-
  implementable. ADR-0010 §TUI PtyOutput buffer during dump defines the canonical types
  (`dump_in_progress: bool` per session, `pending_pty_bytes: Vec<Vec<u8>>` per session) but
  the struct definition here didn't reflect them.
- **Fix (a) — App struct extended with three new fields:**
  - `pty_dump_received: HashSet<String>` — completed-set tracking which sessions have received
    `ScrollbackDumpComplete` in this process lifetime (auto-attach trigger for I11-001).
  - `dump_in_progress: HashMap<String, bool>` — in-flight flag per session; canonical ADR-0010 type.
  - `pending_pty_bytes: HashMap<String, Vec<Vec<u8>>>` — live PtyOutput buffer per session;
    canonical ADR-0010 type. Doc-comments cite ADR-0010 as the canonical source for both.
- **Fix (b) — auto-attach skeleton made normative:** The code block in §EmbeddedTerminal ENTRY
  now EXECUTES `app.dump_in_progress.insert(session_id.clone(), true)` BEFORE sending
  `AttachSession` (was only a comment). Normative prose added: `dump_in_progress` is the
  in-flight signal; `pty_dump_received` is the completed signal — they serve distinct purposes
  and MUST NOT be conflated. Setting `dump_in_progress = true` before `AttachSession` guarantees
  that any `PtyOutput` arriving before the first `ScrollbackChunk` is buffered (not fed to the
  blank parser), satisfying BC-2.05.011 Inv-6 / BC-2.09.001 PC-6.
- **Fix (c) — Module Purity table updated:** Three new rows added for the new App fields
  (all classified Pure core — in-memory state, no I/O).
- Semver: minor (v1.4.0 → v1.5.0) — new normative struct fields + skeleton correction.

## §Trace v1.4.0

**I11-001 PRONG A — auto-attach-on-entry normative mandate added** (2026-06-04):

- **Finding (I11-001 PRONG A):** No normative statement mandated that `enter_embedded_terminal()`
  send `ClientToServer::AttachSession` when the TUI enters an already-running session for the
  first time in the current process lifetime. The blank-parser scenario (TUI reopened, pre-existing
  sessions shown in InitialState/SessionListUpdate, user selects one) had no specified recovery path.
  The reconnect and PtyReset paths were both documented (BC-2.09.001 Invariant 5; BC-2.05.011
  §ScrollbackDumpComplete), but the INITIAL entry case (first enter for a session the TUI has
  never dumped) was implicit.
- **Fix — §EmbeddedTerminal ENTRY extended (normative):** Added "Auto-attach on first entry"
  requirement: `enter_embedded_terminal()` MUST send `ClientToServer::AttachSession { session_id }`
  if `session_id` is not in `App::pty_dump_received` (a new `HashSet<String>` field tracking
  which sessions have received `ScrollbackDumpComplete` in this process lifetime). The code
  skeleton, rationale, and new-session exclusion rule are specified.
- **Fix — §Parser ownership in TUI extended:** Added "Blank-parser state for pre-existing
  sessions" prose explaining when the blank state is acceptable vs. when the auto-attach mandate
  applies. Closes the gap between "parsers start blank" (correct behavior on `SessionListUpdate`)
  and "already-running sessions must show current state on entry" (production-grade v1A guarantee).
- **Scope:** v1A-critical. Sessions survive TUI close and daemon restart (the persistence model
  is ratified). A reopened TUI selecting a running session MUST see its current terminal state.
- Semver: minor (v1.3.0 → v1.4.0) — adds a new normative behavior obligation.

## §Trace v1.3.0

**I5-003/I5-004 — Pass-5 stale forward-instruction + stale scroll-offset reference** (2026-06-03):

- **I5-003 (stale pane_area forward-instruction converted to settled cross-reference):**
  `mouse_event_to_pty_bytes()` doc-comment and §Trace v1.1.0 I1 bullet both carried a
  forward-looking instruction "BC-2.09.003 will be updated by product-owner to use `pane_area`."
  That rename was applied in Pass 2 (BC-2.09.003 at v1.2.0 uses `pane_area`). The pending-action
  language is converted to a settled cross-reference: "BC-2.09.003 was updated to `pane_area`
  in v1.2.0 (Pass-2 closure)." No substantive behavior change; housekeeping only.
- **I5-004 (§Scrollback navigation: `pty_scroll_offset` → `pty_scroll_offsets[focused_session_id]`):**
  §Scrollback navigation prose referenced the retired singular `App::pty_scroll_offset` (the
  pre-I7 single usize field). The I7 fix in §Keyboard and Input Handling replaced it with
  `pty_scroll_offsets: HashMap<String, usize>` keyed by `session_id`. The §Scrollback navigation
  section was not updated in sync. Updated to `App::pty_scroll_offsets[focused_session_id]`
  with a note clarifying the I7 fix, consistent with the App struct definition (§Parser ownership
  in TUI) and the invariants block already in the spec.

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
  name canonicalized (was `screen_offset` in the todo stub). BC-2.09.003 was subsequently
  updated to `pane_area` in v1.2.0 (Pass-2 closure — see §Trace v1.3.0 I5-003 note below).
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
