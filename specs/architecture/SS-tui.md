---
document_type: architecture-section
level: L3
section: "tui"
subsystem: SS-06
version: "1.0.0"
status: draft
producer: vsdd-factory:architect
phase: phase-1-expansion
timestamp: 2026-05-26T02:00:00Z
inputs:
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.13"}
  - {path: .factory/specs/product-brief.md, version: "1.4.30"}
  - {path: .factory/specs/research/domain-monocle-vision-synthesis.md, version: "1.1.3"}
  - {path: .factory/specs/architecture/SS-core-types-and-abi.md, version: "1.2.13"}
input-hash: "[pending]"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: TUI

## Scope

SS-06 specifies the `monocle-tui` crate — the ratatui-based terminal UI — and the
pure-core types that reside in `monocle-core` to support it. The TUI is the entire
product value proposition: one `Ctrl-\` tmux popup that lets a developer observe and
control all running Claude Code sessions without leaving their editor.

The implementing crates are:

- `monocle-core` — pure data types: `AppMode`, `Action`, `PanelId`, `PanelFocus`,
  `FocusSnapshot`, `PromptModal`, `BindingSource`. The state transition function
  `fn transition(mode: AppMode, action: Action) -> AppMode` lives here. No I/O,
  no ratatui, no crossterm.
- `monocle-tui` — ratatui renderer, panel layout, crossterm event loop, IPC
  client connection, keybinding dispatcher, tmux integration. All side-effectful
  code lives here.

For daemon infrastructure (lock file, port, auth) see `SS-daemon-lifecycle.md`.
For IPC message types and UDS framing see the forthcoming `SS-ipc.md`.
For config persistence and profile picker see `SS-config.md`.

This document does not respecify those; it references them and specifies only the
TUI-side contract.

---

## Architectural Principle: Observe-Only, Action-Only via Overlays

The TUI is a client. It NEVER writes to daemon state directly. The invariant:

> All state mutations flow: TUI keypress → IPC `DecisionMessage` → daemon →
> IPC `StateUpdate` pushed back to all TUI clients.

The TUI is read-only for session state and read-only for workflow state. The single
exception is permission decisions (`Accept-once`, `Accept-always`, `Reject`): these
are sent as IPC messages to the daemon, which holds the HTTP response to Claude Code
open and forwards the decision. The TUI never directly touches the Claude Code process.

---

## AppMode State Machine

The AppMode enum is the central architectural decision for the TUI. It provides
compile-time mutual exclusion: the compiler enforces that exactly one mode is active
at any time. This eliminates the bag-of-`Option<Panel>` anti-pattern seen in
NikiforovAll/lazyclaude and the single-popup drop-on-concurrent anti-pattern in
lazygit.

### Enum Definition (monocle-core)

```rust
// monocle-core/src/app_mode.rs

/// Exactly one AppMode is active at any time.
/// The compiler enforces mutual exclusion — no `Option<Panel>` fields anywhere.
///
/// State transitions are pure functions:
///   fn transition(mode: AppMode, action: Action) -> AppMode
/// No Arc<Mutex<...>>; no runtime panics on None unwrap.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AppMode {
    /// Normal dashboard view with panel focus tracking.
    Dashboard { focused: FocusSnapshot },

    /// Telescope-style filter input is active over the focused panel.
    Filtering { panel: PanelId, query: String, prior: FocusSnapshot },

    /// Modal overlay stack is open (permission prompts).
    /// VecDeque<PromptModal> fixes lazygit's single-popup drop-on-concurrent:
    /// new prompts push_back; OverlayCycleNext rotates front to back;
    /// decision pops front; Esc hides without popping.
    Overlay { stack: VecDeque<PromptModal>, prior: FocusSnapshot },

    /// Full-screen view of a single panel (Enter key from Dashboard).
    Fullscreen { panel: PanelId, prior: FocusSnapshot },
}

/// Which panel had focus before a mode transition.
/// Explicit enum eliminates NikiforovAll's gap where modal-close from Sessions
/// focus loses the Sessions fact (was stored as None in Option<Panel>).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FocusSnapshot {
    Sessions,
    EventRibbon,
    // Phase 2+: Customizations, Workflow, Preview
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum PanelId {
    Sessions,
    EventRibbon,
    // Phase 2+: Customizations, Workflow, Preview
}
```

### Transition Function Contract

The `transition` function is a pure function in `monocle-core`. It takes ownership
of the current `AppMode` and an `Action`, and returns the new `AppMode`. It never
panics, never touches I/O, and is fully deterministic.

```rust
// monocle-core/src/transitions.rs

/// Pure state transition function.
/// Every TUI keypress resolves to an Action via the keybinding dispatcher,
/// then passes through this function. The monocle-tui crate calls this;
/// it never mutates AppMode directly.
pub fn transition(mode: AppMode, action: Action) -> AppMode {
    match (mode, action) {
        // Dashboard navigation
        (AppMode::Dashboard { focused }, Action::CyclePanel) =>
            AppMode::Dashboard { focused: focused.cycle() },

        // Filtering entry
        (AppMode::Dashboard { focused }, Action::FilterStart) =>
            AppMode::Filtering {
                panel: focused.to_panel_id(),
                query: String::new(),
                prior: focused,
            },

        // Filtering exit
        (AppMode::Filtering { prior, .. }, Action::Escape) |
        (AppMode::Filtering { prior, .. }, Action::FilterClear) =>
            AppMode::Dashboard { focused: prior },

        // Fullscreen entry
        (AppMode::Dashboard { focused }, Action::Enter) =>
            AppMode::Fullscreen {
                panel: focused.to_panel_id(),
                prior: focused,
            },

        // Fullscreen exit
        (AppMode::Fullscreen { prior, .. }, Action::Escape) =>
            AppMode::Dashboard { focused: prior },

        // Overlay push is NOT handled here — the TUI calls push_prompt() directly
        // on the VecDeque after receiving PermissionPromptQueued from IPC.
        // This transition handles overlay navigation and exit only.

        // Overlay: cycle stack
        (AppMode::Overlay { mut stack, prior }, Action::OverlayCycleNext) => {
            if let Some(front) = stack.pop_front() { stack.push_back(front); }
            AppMode::Overlay { stack, prior }
        },

        // Overlay: decision (accept-once, accept-always, reject) — pops front
        (AppMode::Overlay { mut stack, prior }, Action::PermissionAcceptOnce)
        | (AppMode::Overlay { mut stack, prior }, Action::PermissionAcceptAlways)
        | (AppMode::Overlay { mut stack, prior }, Action::PermissionReject) => {
            stack.pop_front();
            if stack.is_empty() {
                AppMode::Dashboard { focused: prior }
            } else {
                AppMode::Overlay { stack, prior }
            }
        },

        // Overlay: Esc hides without popping (SOQ-3 complement — prompts stay queued)
        (mode @ AppMode::Overlay { .. }, Action::Escape) => mode,

        // Default: no transition for unrecognized (mode, action) combinations
        (mode, _) => mode,
    }
}
```

**Key invariants enforced by this design:**

1. `FocusSnapshot` is always captured when entering `Overlay` or `Fullscreen`.
   Focus is always restored on exit. There is no code path that loses focus context.
2. An empty `VecDeque` in `Overlay` state cannot exist after a decision: the transition
   function collapses `Overlay { stack: empty, prior }` → `Dashboard { focused: prior }`
   atomically.
3. `Escape` from `Overlay` is a no-op on the stack (SOQ-3 support): prompts survive
   the `Ctrl-\` hide/show cycle because `Escape` does not pop any `PromptModal`.

---

## Action Enum and 5-Level Binding Precedence

### Action Enum (monocle-core)

```rust
// monocle-core/src/action.rs

/// Every user-triggerable operation in monocle.
/// Enum variants (not closures) keep bindings Eq + Hash + inspectable
/// for the telescope help overlay (Phase 2 scope).
#[derive(Clone, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub enum Action {
    // System
    Quit,
    DaemonRestart,
    ConfigReload,

    // Navigation
    CyclePanel,
    FocusPanel(PanelId),
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Enter,
    Escape,

    // Filtering
    FilterStart,
    FilterType(char),
    FilterClear,

    // Overlay control
    OverlayCycleNext,

    // Permission decisions
    PermissionAcceptOnce,
    PermissionAcceptAlways,
    PermissionReject,
    PermissionTraceToSource, // Phase 1: stub renders placeholder message

    // Profile picker
    ProfilePicker,

    // Phase 2+ actions (defined now to reserve keybinding slots)
    HelpOverlay,
    SessionKill,
    SessionAttach,
}
```

`Action` derives `Eq + Hash` so the dispatcher can use a `HashMap<KeyEvent, Action>`
per binding level and the help overlay (Phase 2) can enumerate all bound actions.

### BindingSource Enum (monocle-core)

```rust
// monocle-core/src/binding.rs

/// The precedence level that resolved a key → Action mapping.
/// Highest precedence (SearchPrompt) wins; dispatcher stops at first match.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BindingSource {
    SearchPrompt,       // active filter input captures all printable keys
    UserCustomCommand,  // user-defined custom commands in monocle-config
    PerContext,         // bindings declared for the current AppMode/panel
    Global,             // bindings active in all modes
    Builtin,            // factory defaults compiled into monocle-core
}

/// A resolved key → action mapping with source tracking.
pub struct Binding {
    pub action: Action,
    pub source: BindingSource,
}
```

### Dispatcher Logic (monocle-tui)

The dispatcher lives in `monocle-tui::keybinding::Dispatcher`. It holds five
`HashMap<KeyEvent, Action>` tables — one per `BindingSource` level — and resolves
in priority order:

```rust
// monocle-tui/src/keybinding.rs (sketch)

pub struct Dispatcher {
    search_prompt: HashMap<KeyEvent, Action>,
    user_custom:   HashMap<KeyEvent, Action>,
    per_context:   HashMap<KeyEvent, Action>, // rebuilt on AppMode change
    global:        HashMap<KeyEvent, Action>,
    builtin:       HashMap<KeyEvent, Action>, // compiled in; never mutated
}

impl Dispatcher {
    /// Resolve a raw crossterm KeyEvent to a Binding, or None if no match.
    pub fn resolve(&self, key: KeyEvent, mode: &AppMode) -> Option<Binding> {
        for (table, source) in [
            (&self.search_prompt, BindingSource::SearchPrompt),
            (&self.user_custom,   BindingSource::UserCustomCommand),
            (&self.per_context,   BindingSource::PerContext),
            (&self.global,        BindingSource::Global),
            (&self.builtin,       BindingSource::Builtin),
        ] {
            if let Some(action) = table.get(&key) {
                return Some(Binding { action: action.clone(), source });
            }
        }
        None
    }

    /// Rebuild the per_context table when AppMode changes.
    pub fn update_context(&mut self, mode: &AppMode) { /* ... */ }
}
```

The `per_context` table is rebuilt whenever `AppMode` changes. This allows the
`Overlay` mode to expose `[1]/[2]/[3]` for accept/reject without those bindings
being active in `Dashboard` mode.

---

## Panel Architecture

Phase 1 ships three panels. All panel data arrives exclusively via IPC messages from
the daemon; panels never read from disk, network, or process state directly.

### Sessions Panel

Renders `Vec<EnrichedSession>` received via `SessionListUpdate` IPC messages.

**Column layout:**

| Column | Source field | Notes |
|--------|-------------|-------|
| Icon | `EngineMetadata::icon` | Single `char`; `●` for Claude Code in Phase 1 |
| Project | `EnrichedSession::project_name` | Derived from working directory |
| Phase | `EnrichedSession::phase_tag` | From FactoryAdapter; blank if not a factory project |
| Tokens | `EnrichedSession::token_count` | Human-formatted: `142k` |
| Cost | `EnrichedSession::cost_usd` | `$0.83`; blank if not available |
| Uptime | `EnrichedSession::uptime` | `HH:MM:SS` wall clock since SessionStart |

**Empty state:** When `Vec<EnrichedSession>` is empty, the panel renders:

```
No sessions detected
Start Claude Code in any terminal to see it here.
```

**Filter mode:** Pressing `/` transitions `AppMode` to `Filtering { panel: PanelId::Sessions, ... }`.
Typed characters are sent to the nucleo-matcher on each keystroke. Only sessions
whose `project_name` or harness `display_name` fuzzy-match the query are shown.
The match highlights the matched characters using ratatui `Span` styling.

**Fullscreen:** Pressing `Enter` on a focused session row transitions to
`AppMode::Fullscreen { panel: PanelId::Sessions, prior: focused }`. The fullscreen
view renders the session detail: token history (last N hook events contributing to
token count), cost breakdown, hook event count, and current phase tag.

### Event Ribbon Panel

Renders the last N hook events received via `HookEventReceived` IPC messages. N is
determined by the visible panel height; no artificial cap is set beyond what fits on
screen. New events prepend to the top (newest first).

**Column layout:**

| Column | Source field | Width |
|--------|-------------|-------|
| Timestamp | HH:MM:SS.mmm | 12 chars |
| Hook type | `HookType` display name | 16 chars |
| Session ID | Short form (first 8 chars) | 10 chars |
| Latency | `latency_ms` from `HookEventReceived` | 8 chars |
| Status | `PENDING` for unresolved PreToolUse; blank otherwise | 8 chars |

`PENDING` status renders in yellow. Resolved events render in default color.

The panel is scrollable: `Action::ScrollUp / ScrollDown` navigates the ring.

### Status Bar

The status bar is not a panel — it is always rendered at the bottom of the terminal,
one row from the bottom (keybinding hint line) and one row above that (breadcrumb +
drop counter). It is never hidden, even in Fullscreen or Overlay modes.

**Breadcrumb:** Derived from `AppMode`:
- `Dashboard { focused: Sessions }` → `Dashboard > Sessions`
- `Overlay { stack, .. }` (2 items) → `Dashboard > Overlay [2 prompts]`
- `Fullscreen { panel: Sessions, .. }` → `Dashboard > Sessions > Fullscreen`
- `Filtering { panel: Sessions, .. }` → `Dashboard > Sessions > Filter`

**Drop counter:** Receives the `drop_counter: u64` value from each IPC `StateUpdate`
push from the daemon. Renders as `drops: N` in the status bar when N > 0. Renders
nothing when N == 0 (no visual clutter when healthy). The counter is cumulative
across daemon lifetime.

**Keybinding hint line:** Renders a context-sensitive one-line summary of available
actions for the current `AppMode`. Examples:
- `Dashboard`: `Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit`
- `Overlay`: `1: accept-once  2: accept-always  3: reject  ↑↓: cycle  Esc: hide`
- `Filtering`: `(type to filter)  Esc: cancel`
- `Fullscreen`: `Esc: back  /: filter  q: quit`

---

## Permission Overlay

The permission overlay is the most complex TUI feature and the product's primary
competitive differentiator over lazygit (single-popup) and NikiforovAll (Option<Panel>).

### PromptModal Type (monocle-core)

```rust
// monocle-core/src/prompt_modal.rs

/// A single permission prompt queued for user decision.
/// Pushed to VecDeque<PromptModal> when PermissionPromptQueued IPC arrives.
#[derive(Clone, Debug)]
pub struct PromptModal {
    /// Stable ID returned by the daemon; used to correlate the decision response.
    pub prompt_id: Uuid,
    /// Which Claude Code session generated this prompt.
    pub session_id: String,
    /// The tool being requested (Read, Edit, Bash, etc.).
    pub tool_name: String,
    /// Tool-specific payload (path for Edit/Read, command for Bash, etc.).
    pub tool_payload: ToolPayload,
    /// Timestamp when the daemon received the PreToolUse hook.
    pub received_at: std::time::Instant,
}

#[derive(Clone, Debug)]
pub enum ToolPayload {
    Edit { old_content: String, new_content: String, path: String },
    Bash { command: String },
    Read { path: String },
    Generic { raw: serde_json::Value },
}
```

### Overlay Stack Lifecycle

The overlay stack lifecycle in `monocle-tui`:

1. **Push:** When an `IpcMessage::PermissionPromptQueued` arrives on the IPC
   receiver channel, the TUI:
   - Constructs a `PromptModal` from the message payload.
   - Pushes it to the back of the `VecDeque<PromptModal>`.
   - If the current `AppMode` is `Dashboard` or `Filtering`, transitions to
     `AppMode::Overlay { stack, prior: current_focus }`.
   - If `AppMode` is already `Overlay`, extends the existing stack's `VecDeque`
     (the `prior` focus is preserved from when the overlay was first opened).
   - Increments the overlay badge counter in the status bar.

2. **Rotate (`[↑↓]`):** `Action::OverlayCycleNext` is passed to `transition()`,
   which rotates the `VecDeque`: front item moves to back, exposing the next prompt.

3. **Decide (`[1]`, `[2]`, `[3]`):** The TUI:
   - Identifies the decision type from the action (`PermissionAcceptOnce`,
     `PermissionAcceptAlways`, `PermissionReject`).
   - Reads the `prompt_id` from the current front `PromptModal`.
   - Sends `IpcMessage::DecisionResponse { prompt_id, decision }` to the daemon
     via the IPC send channel. This is non-blocking: the TUI enqueues the message
     and continues.
   - Passes the action to `transition()` which pops the front `PromptModal` and
     collapses to `Dashboard` if the stack becomes empty.

4. **Hide (`[Esc]`):** `Action::Escape` in `Overlay` mode is a no-op on the stack
   per the transition function. The `Ctrl-\` popup is hidden by the user's tmux
   keybinding (external to the TUI process). Prompts remain queued in the
   `VecDeque`. On the next `Ctrl-\`, the popup reappears with `AppMode` unchanged,
   showing the same stack.

5. **Daemon disconnect (SOQ-3):** When the IPC channel signals a disconnect, the TUI:
   - Clears the entire `VecDeque<PromptModal>`.
   - Transitions `AppMode` to `Dashboard { focused: FocusSnapshot::Sessions }`.
   - Renders "Daemon disconnected — reconnecting..." in the status bar.
   Rationale: Claude Code subprocesses will time out stalled hook responses when the
   daemon restarts. Queued prompts against the old daemon are orphaned and will never
   be resolved via the new daemon connection. Clearing prevents ghost approvals.

### Diff Preview

When the front `PromptModal` has `ToolPayload::Edit`, the overlay renders a unified
diff computed via `similar::TextDiff::from_lines`:

```rust
// monocle-tui/src/overlay.rs (sketch)

fn render_diff(old: &str, new: &str, frame: &mut Frame, area: Rect) {
    let diff = similar::TextDiff::from_lines(old, new);
    let mut lines: Vec<Line> = Vec::new();
    for change in diff.iter_all_changes() {
        let (prefix, style) = match change.tag() {
            similar::ChangeTag::Delete => ("-", Style::default().fg(Color::Red)),
            similar::ChangeTag::Insert => ("+", Style::default().fg(Color::Green)),
            similar::ChangeTag::Equal  => (" ", Style::default()),
        };
        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, change.value()),
            style,
        )));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}
```

Lines prefixed with `-` render in red (`Color::Red`). Lines prefixed with `+` render
in green (`Color::Green`). Context lines render in the default terminal color.

The diff area is height-capped to `(overlay_height - 8)` rows to preserve room for
the prompt header and action hint line.

### Trace-to-Source Stub (`[t]`)

In Phase 1, pressing `[t]` in `Overlay` mode renders a placeholder message in the
overlay footer:

```
[t] Trace to source — Phase 2 feature (Static plane)
```

The keybinding is reserved, registered in the `Builtin` binding table, and
discoverable from the hint line. No navigation occurs. This ensures the keybinding
exists in Phase 1 so Phase 2 stories can implement the behavior without a keybinding
conflict.

### Hook Timeout Budget (BC-2.06.017)

The daemon holds the Claude Code HTTP response open until either a decision arrives
via IPC or the hook type's timeout ceiling is reached:

| Hook type | Timeout ceiling | Source |
|-----------|----------------|--------|
| PreToolUse | 300ms | BC-HOOK-022 (gene-source) |
| Stop | 300ms | BC-HOOK-022 |
| SessionStart | 300ms | BC-HOOK-022 |
| UserPromptSubmit | 300ms | BC-HOOK-022 |
| Notification | 2000ms | BC-HOOK-022 |

The TUI's responsibility is to present the overlay as fast as possible after
`PermissionPromptQueued` arrives. The latency budget for the TUI hop is:

```
hook_post_receipt → IPC push → TUI render ≤ 100ms (Success Criterion)
```

The remaining budget (300ms total − 100ms TUI render = 200ms) covers the user's
keypress and the IPC decision response path. On timeout, the daemon applies
fail-open or fail-closed semantics per BC-HOOK-001 (gene-source). The TUI does not
implement timeout logic itself; it simply sends the decision as fast as the user
acts.

---

## Ctrl-\ Integration

### tmux Popup Command

The TUI is launched as a `tmux display-popup` over the user's existing tmux session.
The canonical invocation pattern:

```bash
# Bound to Ctrl-\ in the user's tmux.conf
bind-key -n C-\\ display-popup -E -w 80% -h 80% 'monocle'
```

When `monocle` is invoked without a subcommand:
1. It reads the lock file to determine if a daemon is running.
2. If no daemon is running and `MONOCLE_NO_AUTOSTART` is not set, it starts the
   daemon (SS-04 auto-start sequence).
3. It connects to the UDS at `<runtime_dir>/monocle.sock` (SS-05).
4. It enters the ratatui event loop.

### State Preservation Across Hide/Show

The TUI process runs for the lifetime of the tmux popup. When the user presses
`Ctrl-\` to hide, tmux closes the popup window. When the user presses `Ctrl-\`
again, `tmux display-popup` spawns a NEW `monocle` process.

This means AppMode is NOT preserved across hide/show cycles at the process level.
However, the daemon continuously pushes IPC state updates to new connecting clients.
The reconnecting TUI receives:

- Current `Vec<EnrichedSession>` (session list)
- Recent `HookEventReceived` messages (ring tail)
- The current `VecDeque<PromptModal>` overlay stack (daemon holds queued prompts
  in its own state, not in the TUI process)

The overlay stack survives `Ctrl-\` because the daemon owns it, not the TUI process.
The daemon's UDS server sends the queued overlay stack in the initial state push
(per BC-2.05.002). The new TUI process transitions to `AppMode::Overlay` if the
daemon reports any queued prompts.

This satisfies the requirement from BC-2.06.004 and BC-2.06.014: "overlay survives
`Ctrl-\` hide/show cycle without dropping queued prompts." The daemon is the durable
state store; the TUI is the stateless view.

**Critical implication for the daemon:** The daemon's `DaemonState` must include
a `queued_prompts: VecDeque<PromptModal>` field that is pushed to every new TUI
client connection as part of the initial state push (BC-2.05.002). This is the
mechanism by which overlay state survives TUI process restart.

---

## Killer Scenario Flow

The product's primary user promise: 4 keystrokes to resolve 2 concurrent permission
prompts without leaving the editor (Success Criterion: ≤6 keystrokes).

### Precondition

- User is in nvim (or any editor) inside a tmux session.
- Two Claude Code sessions have stalled, each waiting for a PreToolUse decision.
- Daemon has received 2 `PermissionPromptQueued` events; `DaemonState::queued_prompts`
  holds both `PromptModal` entries.
- TUI is not running (last popup was dismissed).

### Step-by-Step

| Step | User action | AppMode before | AppMode after | Daemon action |
|------|-------------|---------------|--------------|---------------|
| 1 | `Ctrl-\` | (TUI not running) | `Overlay { stack: [P1, P2], prior: Sessions }` | Sends initial state push: 2 queued prompts |
| 2 | `2` (Accept-always) | `Overlay { stack: [P1, P2], prior: Sessions }` | `Overlay { stack: [P2], prior: Sessions }` | Sends `{"decision":"always"}` to P1's stalled HTTP response; P1's Claude Code session unblocks |
| 3 | `1` (Accept-once) | `Overlay { stack: [P2], prior: Sessions }` | `Dashboard { focused: Sessions }` | Sends `{"decision":"accept"}` to P2's stalled HTTP response; P2's Claude Code session unblocks |
| 4 | `Ctrl-\` | `Dashboard { focused: Sessions }` | (TUI exits) | No action |

Total: 4 keystrokes. No tmux window switches. No editor focus lost. Both Claude Code
sessions continue without operator awareness of which session stalled.

---

## Rendering Architecture

### App Struct (monocle-tui)

```rust
// monocle-tui/src/app.rs

pub struct App {
    /// Current AppMode — single source of truth for which panels are rendered.
    pub mode: AppMode,

    /// Session list from last IPC SessionListUpdate message.
    pub sessions: Vec<EnrichedSession>,

    /// Hook events from last N IPC HookEventReceived messages.
    pub events: VecDeque<HookEventRow>,

    /// Drop counter from last IPC StateUpdate.
    pub drop_counter: u64,

    /// Keybinding dispatcher with current per-context bindings.
    pub dispatcher: Dispatcher,

    /// Nucleo matcher for filter mode. Re-used across filter inputs.
    pub matcher: nucleo::Matcher,

    /// IPC send channel — for outbound DecisionResponse messages.
    pub ipc_tx: tokio::sync::mpsc::Sender<IpcClientMessage>,

    /// IPC receive channel — for inbound daemon pushes.
    pub ipc_rx: tokio::sync::mpsc::Receiver<IpcServerMessage>,
}
```

### Draw Loop

```rust
// monocle-tui/src/main.rs (sketch)

async fn run_app(mut terminal: Terminal<CrosstermBackend<Stdout>>,
                 mut app: App) -> Result<()> {
    let tick_rate = Duration::from_millis(16); // ~60fps
    let mut last_tick = Instant::now();

    loop {
        // 1. Drain IPC messages (non-blocking)
        while let Ok(msg) = app.ipc_rx.try_recv() {
            app.handle_ipc_message(msg);
        }

        // 2. Render
        terminal.draw(|frame| draw(frame, &app))?;

        // 3. Poll crossterm events with remaining tick budget
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let Some(binding) = app.dispatcher.resolve(key, &app.mode) {
                    let new_mode = transition(app.mode.clone(), binding.action.clone());
                    app.mode = new_mode;
                    app.handle_action(binding.action).await?;
                }
            }
            if let Event::Resize(_, _) = event::read()? {
                // ratatui handles resize automatically on next draw
            }
        }

        if app.should_quit() { break; }
        last_tick = Instant::now();
    }
    Ok(())
}
```

### Draw Function Dispatch

```rust
// monocle-tui/src/draw.rs

fn draw(frame: &mut Frame, app: &App) {
    // Layout: split into main area + status bar (2 rows)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(frame.size());

    let main_area = chunks[0];
    let status_area = chunks[1];

    // Main area: delegate to AppMode-specific renderer
    match &app.mode {
        AppMode::Dashboard { focused } => {
            draw_dashboard(frame, main_area, app, focused);
        },
        AppMode::Filtering { panel, query, .. } => {
            draw_dashboard(frame, main_area, app, /* show filter input */);
            draw_filter_overlay(frame, main_area, panel, query);
        },
        AppMode::Overlay { stack, .. } => {
            draw_dashboard(frame, main_area, app, /* dimmed */);
            draw_permission_overlay(frame, main_area, stack.front().unwrap(), stack.len());
            if stack.len() > 1 {
                draw_permission_overlay_peek(frame, main_area, &stack[1]);
            }
        },
        AppMode::Fullscreen { panel, .. } => {
            draw_fullscreen(frame, main_area, app, panel);
        },
    }

    // Status bar: always rendered
    draw_status_bar(frame, status_area, app);
}
```

The dashboard layout in Phase 1 splits the main area into two horizontal panels:
Sessions (left, 60% width) and Event Ribbon (right, 40% width). This matches the
Phase 1 panel set. Phase 2 adds Customizations (below) and Workflow (right); the
layout splits are deferred to Phase 2 architecture.

---

## Purity Boundary

| Type / Function | Location | Rationale |
|----------------|----------|-----------|
| `AppMode` enum | `monocle-core` | Pure data; no I/O |
| `Action` enum | `monocle-core` | Pure data; no I/O |
| `FocusSnapshot` enum | `monocle-core` | Pure data; no I/O |
| `PanelId` enum | `monocle-core` | Pure data; no I/O |
| `PromptModal` struct | `monocle-core` | Pure data; no I/O |
| `BindingSource` enum | `monocle-core` | Pure data; no I/O |
| `Binding` struct | `monocle-core` | Pure data; no I/O |
| `fn transition(AppMode, Action) -> AppMode` | `monocle-core` | Pure function; formally verifiable |
| `FocusSnapshot::cycle()` | `monocle-core` | Pure function; testable without I/O |
| `FocusSnapshot::to_panel_id()` | `monocle-core` | Pure function; trivial conversion |
| `App` struct | `monocle-tui` | Holds IPC channels; effectful |
| `Dispatcher` struct | `monocle-tui` | Reads config bindings; effectful |
| `draw()` function | `monocle-tui` | Writes to terminal; effectful |
| `run_app()` event loop | `monocle-tui` | Async I/O; effectful |
| Nucleo matcher usage | `monocle-tui` | Stateful; effectful |
| `similar::TextDiff` rendering | `monocle-tui` | Stateless computation, but invoked from renderer; effectful context |

**Formal verification target:** `fn transition(AppMode, Action) -> AppMode` is the
primary verification target for SS-06. It is a pure, total function over a finite
domain; all reachable `(mode, action)` pairs can be enumerated. Kani proof harnesses
can verify:

- No `Overlay` variant is ever returned with an empty `stack`.
- `FocusSnapshot` carried in `prior` is always preserved through nested transitions.
- `Filtering` → `Dashboard` always restores the correct `FocusSnapshot`.

---

## Dependency Graph

`monocle-tui` depends on:

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.30 | Terminal UI framework |
| `crossterm` | 0.29 | Cross-platform terminal events and raw mode |
| `monocle-core` | workspace | Pure types: AppMode, Action, PromptModal, transition() |
| `monocle-ipc` | workspace | UDS client, IPC message types, framing |
| `monocle-config` | workspace | Config reads for binding overrides, profile picker |
| `similar` | 3.x | Unified diff computation for Edit tool diff preview |
| `nucleo` | 0.5 | Fuzzy matcher for `/` filter in sessions panel |
| `tokio` | 1.x (features: rt-multi-thread, macros) | Async runtime |
| `tracing` | 0.1 | Structured logging (no `println!` in production code) |
| `uuid` | 1.x (features: v4) | `PromptModal::prompt_id` |

`monocle-core` type additions for SS-06 (`AppMode`, `Action`, `PromptModal`,
`FocusSnapshot`, `PanelId`, `BindingSource`, `Binding`, `transition()`) do NOT add
new crate dependencies to `monocle-core`. `monocle-core` remains zero-dependency for
its pure-type surface. The only additions to `monocle-core`'s `Cargo.toml` are:

- `uuid 1.x` (for `PromptModal::prompt_id`) — no I/O feature flags enabled
- `serde` (already present for existing types) — `Serialize + Deserialize` on `Action`
- `similar 3.x` is NOT a `monocle-core` dependency; diff computation lives in
  `monocle-tui` only.

---

## Behavioral Contracts

All 22 Phase 1 BCs for SS-06. Priority P0 must be delivered in Waves 6–7; P1 may
follow in a later wave within Phase 1.

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.06.001 | AppMode State Machine: Compile-Time Mutual Exclusion | P0 |
| BC-2.06.002 | FocusSnapshot: Focus Restored After Overlay/Fullscreen Close | P0 |
| BC-2.06.003 | Action Dispatch: 5-Level Binding Precedence | P0 |
| BC-2.06.004 | `Ctrl-\` Popup: Appears and Dismisses Without State Loss | P0 |
| BC-2.06.005 | Sessions Panel: Session List Renders from IPC State | P0 |
| BC-2.06.006 | Sessions Panel: `/` Filter with Nucleo Fuzzy Match | P0 |
| BC-2.06.007 | Sessions Panel: `Enter` Transitions to Fullscreen | P0 |
| BC-2.06.008 | Permission Overlay: VecDeque Stack Push on PermissionPromptQueued | P0 |
| BC-2.06.009 | Permission Overlay: `[↑↓]` Rotates Stack | P0 |
| BC-2.06.010 | Permission Overlay: Diff Preview via `similar 3` | P0 |
| BC-2.06.011 | Permission Overlay: Accept-Once Keybinding | P0 |
| BC-2.06.012 | Permission Overlay: Accept-Always Keybinding | P0 |
| BC-2.06.013 | Permission Overlay: Reject Keybinding | P0 |
| BC-2.06.014 | Permission Overlay: `[Esc]` Hides Without Rejecting | P0 |
| BC-2.06.015 | Permission Overlay: `[t]` Trace-to-Source Stub | P1 |
| BC-2.06.016 | Permission Overlay: Cleared on Daemon Disconnect | P0 |
| BC-2.06.017 | Permission Response Within Hook Timeout Budget | P0 |
| BC-2.06.018 | Event Ribbon Panel: Rolling Hook Event Log | P0 |
| BC-2.06.019 | Status Bar: Drop Counter Renders Under Load | P0 |
| BC-2.06.020 | Status Bar: Breadcrumb | P1 |
| BC-2.06.021 | Status Bar: Keybinding Hint Line | P1 |
| BC-2.06.022 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | P0 |

---

## Constraints

The following constraints are non-negotiable for `monocle-tui` and `monocle-core`
SS-06 types. Violations are blocking in any PR review:

1. **`#[forbid(unsafe_code)]`** in `monocle-tui/src/lib.rs` and all new `monocle-core`
   SS-06 modules. No unsafe blocks.
2. **No `Option<Panel>` fields** anywhere in `AppMode` or `App`. `AppMode` is the
   single source of truth for which panels are active.
3. **No `Arc<Mutex<...>>` in state transitions.** The `transition()` function takes
   ownership of `AppMode` and returns a new `AppMode`. No shared mutable state in
   the transition path.
4. **`VecDeque<PromptModal>` for overlay stack.** `Option<PromptModal>` is forbidden.
   Queued prompts must survive `Ctrl-\` hide/show via the daemon's durable state
   (see §Ctrl-\ Integration).
5. **Bounded channels only.** All `mpsc` channels in `monocle-tui` use bounded
   variants. Drop counters are required on any channel that can drop messages.
6. **All `monocle-core` SS-06 types are `#[non_exhaustive]`.** Exception: `PanelId`
   and `FocusSnapshot` are `non_exhaustive` to allow Phase 2 panel additions without
   breaking `match` sites in `monocle-tui`.
7. **TUI is a client.** No direct mutation of daemon state from `monocle-tui`. All
   mutations flow through IPC `DecisionResponse` messages. `monocle-tui` has no
   dependency on `monocle-runtime`.
8. **No `println!` in production code.** Use `tracing::debug!` / `tracing::info!`
   for all diagnostic output.

---

## §Trace v1.0.0

**Initial production** (2026-05-26T02:00:00Z):
- SS-06 TUI architecture document created.
- Reads: `prd-expansion-scope.md` v1.0 §2 and §3.3 (22 BCs), `SS-daemon-lifecycle.md`
  v1.0.33, `ARCH-INDEX.md` v1.0.13, `product-brief.md` v1.4.30 lines 138-155 and
  275-295, `domain-monocle-vision-synthesis.md` v1.1.3 §Five Planes / §Process Topology /
  §Workspace Layout / §Key Abstractions / §TUI Layout, `SS-core-types-and-abi.md`
  v1.2.13 (first 150 lines).
- Documents: AppMode enum (compile-time mutual exclusion), pure `transition()` function,
  Action enum (5-level binding precedence), Sessions/EventRibbon/StatusBar panel
  architecture, VecDeque<PromptModal> overlay stack, diff preview via `similar 3`,
  Ctrl-\ popup state-preservation mechanism (daemon owns queued_prompts), 4-keystroke
  killer scenario trace, rendering architecture, purity boundary map, dependency graph,
  and full 22-BC behavioral contract table.
- Key architectural decision: AppMode state survives `Ctrl-\` hide/show via daemon
  ownership of `queued_prompts: VecDeque<PromptModal>`, not TUI process state. Each
  `display-popup` invocation spawns a fresh TUI process; daemon initial-state-push
  (BC-2.05.002) delivers current overlay stack to the new client.
- All 22 BCs from prd-expansion-scope.md §3.3 enumerated with correct IDs and priorities.
- version: 1.0.0; timestamp: 2026-05-26T02:00:00Z.
