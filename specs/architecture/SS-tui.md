---
document_type: architecture-section
level: L3
section: "tui"
subsystem: SS-06
version: "1.8.0"
status: draft
producer: vsdd-factory:architect
phase: phase-1-expansion
timestamp: 2026-05-28T00:00:00Z
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
    /// The modal stack itself lives in `App.overlay_stack: VecDeque<PromptModal>` —
    /// the single source of truth for queued prompts (see §Rendering Architecture §App Struct).
    /// `AppMode::Overlay` carries only `prior` so that focus can be restored on dismiss.
    /// new prompts push_back to App.overlay_stack; OverlayCycleNext rotates front to back;
    /// decision sends IPC and awaits PermissionPromptResolved (IPC-driven remove);
    /// Esc hides without popping.
    Overlay { prior: FocusSnapshot },

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

### Profile Picker: Transient Overlay (Not an AppMode Variant)

The profile picker (triggered by `Action::ProfilePicker` / `Ctrl-P`) is a brief
dropdown-like interaction that must not be modeled as an `AppMode` variant. The
architectural decision:

**Decision:** The profile picker is managed by a separate `Option<ProfilePickerState>`
field in the `App` struct, NOT via the `AppMode` state machine.

Rationale:
1. The profile picker is a transient overlay lasting one interaction (select or dismiss);
   adding an `AppMode::ProfilePicker` variant would inflate the state machine and all
   `match` sites with a variant that carries no distinct transition behavior.
2. The picker can appear over any `AppMode` (`Dashboard`, `Fullscreen`, `Overlay`)
   without replacing it. Modeling this as an `AppMode` would require stacking or
   wrapping, introducing the same `prior` field complexity as `Overlay` for a much
   simpler case.
3. When the picker closes (selection made or `Esc`), the underlying `AppMode` is
   unchanged — no focus restoration logic is needed.
4. The `Option<ProfilePickerState>` field gives the draw loop a clear nil-check:
   `if let Some(picker) = &app.picker { draw_profile_picker(...) }`.

The `App` struct includes this field alongside `mode`:

```rust
pub struct App {
    pub mode: AppMode,
    pub picker: Option<ProfilePickerState>,  // transient; does not affect AppMode
    // ... other fields (see §Rendering Architecture §App Struct)
}
```

`ProfilePickerState` is defined in `monocle-tui` (not `monocle-core`) because it
holds config read state and is effectful. The picker's keystrokes (`Enter` to select,
`Esc` to dismiss, arrow keys to navigate) are consumed in `app.handle_action()` before
reaching `transition()` when `app.picker.is_some()` — the profile picker short-circuits
normal `AppMode` dispatch while it is open.

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
        // App.overlay_stack is the single source of truth for the modal VecDeque;
        // transition() receives the AppMode only (no stack field). The TUI's
        // handle_action() rotates App.overlay_stack directly before or after
        // calling transition() for the mode change (mode is unchanged for CycleNext).
        (mode @ AppMode::Overlay { .. }, Action::OverlayCycleNext) => mode,

        // Overlay: decision (accept-once, accept-always, reject) — does NOT pop front.
        // The TUI sends ClientToServer::PermissionDecision and then waits for
        // ServerToClient::PermissionPromptResolved { prompt_id } from the daemon.
        // The prompt is removed via handle_ipc_message() / retain() (BC-2.06.023).
        // transition() leaves the stack unchanged; the AppMode stays in Overlay.
        (mode @ AppMode::Overlay { .. }, Action::PermissionAcceptOnce)
        | (mode @ AppMode::Overlay { .. }, Action::PermissionAcceptAlways)
        | (mode @ AppMode::Overlay { .. }, Action::PermissionReject) => mode,

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
2. An empty `App.overlay_stack` in `Overlay` state cannot exist after IPC-initiated removal:
   `handle_ipc_message()` collapses `AppMode::Overlay { prior }` →
   `Dashboard { focused: prior }` after `retain()` empties `App.overlay_stack` (BC-2.06.023).
   Decision actions (`PermissionAcceptOnce`, `PermissionAcceptAlways`, `PermissionReject`)
   do NOT pop the stack in `transition()` — removal is always IPC-driven.
3. `Escape` from `Overlay` is a no-op on the stack (SOQ-3 support): prompts survive
   the `Ctrl-\` hide/show cycle because `Escape` does not pop any `PromptModal`.

**BC-2.06.023 — IPC-Initiated Prompt Removal (transition() scope boundary):**

The `transition()` function handles `Action`-based state transitions only. It is a pure
function that maps `(AppMode, Action) → AppMode`; it has no access to a `prompt_id`.

IPC-initiated operations such as removing a specific prompt on receipt of
`ServerToClient::PermissionPromptResolved` are handled by the TUI event handler
directly, outside `transition()`:

```rust
// monocle-tui/src/app.rs — IPC message handler (not transition())

fn handle_ipc_message(&mut self, msg: ServerToClient) {
    match msg {
        ServerToClient::PermissionPromptResolved { prompt_id } => {
            if let AppMode::Overlay { ref prior } = self.mode {
                self.overlay_stack.retain(|m| m.prompt_id != prompt_id);
                if self.overlay_stack.is_empty() {
                    self.mode = AppMode::Dashboard { focused: prior.clone() };
                }
            }
            // No-op if prompt_id not present (already decided or daemon timeout race)
        }
        // ... other variants
    }
}
```

The empty-stack-to-Dashboard collapse after IPC-initiated removal reuses the same
invariant as `transition(Overlay { empty_stack }, PermissionAcceptOnce)` — an empty
`Overlay` stack always collapses to `Dashboard { focused: prior }`. But the collapse
in the IPC path is triggered by `stack.is_empty()` after `retain()`, not by dispatching
an `Action`. This is intentional: `Action::PermissionPromptResolved` does not exist as
a variant; IPC events are not `Action`s and must not be routed through the `transition()`
function.

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
`Overlay` mode to expose `y`/`Enter`/`A`/`n`/`r` for accept/reject without those
bindings being active in `Dashboard` mode.

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
| Project | `EnrichedSession::project_name` | Derived from transcript directory name; `"—"` when `None` |
| Status | `EnrichedSession::status` | `SessionStatus` display: Active, Idle, WaitingOnPermission, etc. |
| Tokens | `EnrichedSession::token_count` | Human-formatted: `142k`; accumulated from hook event metadata |
| Cost | `EnrichedSession::cost_usd` | `$0.83`; `"—"` when `None` (harness not emitting cost data) |
| Uptime | `EnrichedSession::started_at` | `HH:MM:SS` wall clock computed as `now - started_at`; `"—"` when `None` |

> **Note:** `phase_tag` is not present on `EnrichedSession` in Phase 1 — it requires
> `FactoryAdapter` integration not available in Phase 1. `uptime` is derived at render
> time from `started_at`; there is no dedicated `uptime` field on the struct.

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
- `AppMode::Overlay { .. }` (App.overlay_stack len = 2) → `Dashboard > Overlay [2 prompts]`
- `Fullscreen { panel: Sessions, .. }` → `Dashboard > Sessions > Fullscreen`
- `Filtering { panel: Sessions, .. }` → `Dashboard > Sessions > Filter`

**Drop counter:** Receives the `drop_counter: u64` value from each IPC `StateUpdate`
push from the daemon. Renders as `drops: N` in the status bar when N > 0. Renders
nothing when N == 0 (no visual clutter when healthy). The counter is cumulative
across daemon lifetime.

**Keybinding hint line:** Renders a context-sensitive one-line summary of available
actions for the current `AppMode`. Examples:
- `Dashboard`: `Tab: cycle  Enter: fullscreen  /: filter  Ctrl-P: profile  q: quit`
- `Overlay`: `y/Enter: accept-once  A: accept-always  n/r: reject  ↑↓: cycle  Esc: hide`
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
    /// Fallback for tools not explicitly handled (WebSearch, mcp__*, etc.).
    /// Carries `tool_name` so the overlay header can display the tool name
    /// without requiring a separate `tool_name` field lookup on `PromptModal`.
    Generic { tool_name: String, tool_input: serde_json::Value },
}
```

### §IPC Payload to PromptModal Conversion

When the TUI event handler receives a `ServerToClient::PermissionPromptQueued` message
(or rebuilds the overlay stack from `ServerToClient::InitialState`), it constructs a
`PromptModal` from the `PermissionPromptPayload`. The conversion is defined here to
ensure a single canonical mapping between IPC wire types and TUI display types:

1. `payload.prompt_id` → `PromptModal::prompt_id` (copied directly; both `Uuid`)
2. `payload.session_id` → `PromptModal::session_id` (copied directly; both `String`)
3. `payload.tool_name` → selects `ToolPayload` variant:
   - `"Edit"` if `old_content.is_some() || new_content.is_some()` → `ToolPayload::Edit { old_content, new_content, path: tool_input["file_path"].as_str() }`
   - `"Bash"` → `ToolPayload::Bash { command: tool_input["command"].as_str() }`
   - `"Read"` → `ToolPayload::Read { path: tool_input["file_path"].as_str() }`
   - `_` (including `"Edit"` with both content fields absent) → `ToolPayload::Generic { tool_name: payload.tool_name, tool_input: payload.tool_input }`
4. `received_at` → `std::time::Instant::now()` set at TUI reception time; not
   deserialized from IPC (Instant is not serializable; the daemon's reception time is
   not forwarded over the wire).

**Fallback rules:** Two conditions force fallback to `ToolPayload::Generic { tool_name, tool_input }`:
1. If `tool_input` does not contain the expected field for the matched variant (e.g.,
   `"file_path"` absent for `"Edit"` or `"Read"`, `"command"` absent for `"Bash"`).
2. If `tool_name == "Edit"` but BOTH `old_content` and `new_content` are `None` — an
   `Edit` with no content to diff renders as an empty diff pane; the `Generic` fallback
   renders the raw `tool_input` JSON, which is more informative.

These rules prevent a missing or empty field from causing a panic or a failed push — the
overlay always receives something renderable.

The `old_content` and `new_content` fields on `PermissionPromptPayload` are flattened
into the `ToolPayload::Edit` variant rather than passed separately, consolidating all
tool-specific data into a single matched arm.

```rust
// monocle-tui/src/ipc_conversion.rs (sketch)

fn payload_to_modal(payload: PermissionPromptPayload) -> PromptModal {
    let tool_payload = match payload.tool_name.as_str() {
        // Guard 1: path must be present for a meaningful Edit variant.
        // Guard 2: at least one of old_content / new_content must be Some — an Edit with
        //          neither field has no content to diff and MUST fall back to Generic so
        //          the overlay renders the raw tool_input rather than an empty diff pane.
        "Edit" if (payload.old_content.is_some() || payload.new_content.is_some()) => {
            let path = payload.tool_input["file_path"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_default();
            if path.is_empty() {
                ToolPayload::Generic { tool_name: payload.tool_name.clone(), tool_input: payload.tool_input.clone() }
            } else {
                ToolPayload::Edit {
                    old_content: payload.old_content.unwrap_or_default(),
                    new_content: payload.new_content.unwrap_or_default(),
                    path,
                }
            }
        }
        "Bash" => {
            let command = payload.tool_input["command"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_default();
            if command.is_empty() {
                ToolPayload::Generic { tool_name: payload.tool_name.clone(), tool_input: payload.tool_input.clone() }
            } else {
                ToolPayload::Bash { command }
            }
        }
        "Read" => {
            let path = payload.tool_input["file_path"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_default();
            if path.is_empty() {
                ToolPayload::Generic { tool_name: payload.tool_name.clone(), tool_input: payload.tool_input.clone() }
            } else {
                ToolPayload::Read { path }
            }
        }
        // Fallback: covers (a) "Edit" with both old_content and new_content absent,
        //           (b) any unrecognised tool name.
        _ => ToolPayload::Generic { tool_name: payload.tool_name.clone(), tool_input: payload.tool_input.clone() },
    };
    PromptModal {
        prompt_id: payload.prompt_id,
        session_id: payload.session_id,
        // tool_name is kept on PromptModal for overlay header rendering so callers do not
        // need to pattern-match ToolPayload just to display the tool name.
        tool_name: payload.tool_name,
        tool_payload,
        received_at: std::time::Instant::now(),
    }
}
```

### Overlay Stack Lifecycle

The overlay stack lifecycle in `monocle-tui`:

1. **Push:** When a `ServerToClient::PermissionPromptQueued` arrives on the IPC
   receiver channel, the TUI:
   - Constructs a `PromptModal` from the message payload.
   - Pushes it to the back of the `VecDeque<PromptModal>`.
   - If the current `AppMode` is `Dashboard` or `Filtering`, transitions to
     `AppMode::Overlay { prior: current_focus }` and pushes the new `PromptModal`
     onto `App.overlay_stack`.
   - If `AppMode` is already `Overlay`, pushes onto `App.overlay_stack` directly
     (the `prior` focus in the existing `AppMode::Overlay` is preserved from
     when the overlay was first opened).
   - Increments the overlay badge counter in the status bar.

2. **Rotate (`[↑↓]`):** `Action::OverlayCycleNext` is passed to `transition()`,
   which rotates the `VecDeque`: front item moves to back, exposing the next prompt.

3. **Decide (`y`/`Enter`, `A`, `n`/`r`):** The TUI:
   - Identifies the decision type from the action (`PermissionAcceptOnce` via `y` or
     `Enter`; `PermissionAcceptAlways` via `A`; `PermissionReject` via `n` or `r`).
   - Reads the `prompt_id` from the current front `PromptModal`.
   - Sends `ClientToServer::PermissionDecision { prompt_id, decision }` to the daemon
     via the IPC send channel. This is non-blocking: the TUI enqueues the message
     and continues.
   - Does NOT pop the `PromptModal` from the stack. The overlay remains open showing
     the same prompt. Removal happens when the daemon sends
     `ServerToClient::PermissionPromptResolved { prompt_id }`, which triggers
     `handle_ipc_message()` → `stack.retain(|m| m.prompt_id != prompt_id)` →
     collapse to `Dashboard` if the stack becomes empty (BC-2.06.023).

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
- The pending overlay prompts as `Vec<PermissionPromptPayload>` (converted by the TUI
  to `VecDeque<PromptModal>` via `payload_to_modal()`; daemon stores the IPC-serializable
  form, not `PromptModal` which contains `Instant` and is not serializable)

The overlay stack survives `Ctrl-\` because the daemon owns it, not the TUI process.
The daemon's UDS server sends the queued overlay stack in the initial state push
(per BC-2.05.002). The new TUI process transitions to `AppMode::Overlay` if the
daemon reports any queued prompts.

This satisfies the requirement from BC-2.06.004 and BC-2.06.014: "overlay survives
`Ctrl-\` hide/show cycle without dropping queued prompts." The daemon is the durable
state store; the TUI is the stateless view.

**Critical implication for the daemon:** The daemon stores pending prompts as
`Vec<PermissionPromptPayload>` in its pending-decision registry. The IPC
`InitialState` push sends `overlay_stack: Vec<PermissionPromptPayload>`. The TUI
converts each to a `PromptModal` via `payload_to_modal()` on receipt. This is the
mechanism by which overlay state survives TUI process restart (BC-2.05.002).

---

## Killer Scenario Flow

The product's primary user promise: 4 keystrokes to resolve 2 concurrent permission
prompts without leaving the editor (Success Criterion: ≤6 keystrokes).

### Precondition

- User is in nvim (or any editor) inside a tmux session.
- Two Claude Code sessions have stalled, each waiting for a PreToolUse decision.
- Daemon has received 2 `PermissionPromptQueued` events; its pending-decision registry
  holds both as `PermissionPromptPayload` entries.
- TUI is not running (last popup was dismissed).

### Step-by-Step

| Step | User action | AppMode before | AppMode after | Daemon action |
|------|-------------|---------------|--------------|---------------|
| 1 | `Ctrl-\` | (TUI not running) | `Overlay { prior: Sessions }` / App.overlay_stack: [P1, P2] | Sends initial state push: 2 queued prompts |
| 2 | `A` (Accept-always) | `Overlay { prior: Sessions }` / App.overlay_stack: [P1, P2] | `Overlay { prior: Sessions }` / App.overlay_stack: [P2] (after `PermissionPromptResolved` for P1 received) | Sends `{"decision":"always"}` to P1's stalled HTTP response; P1 unblocks; sends `PermissionPromptResolved { prompt_id: P1 }` to TUI |
| 3 | `y` (Accept-once) | `Overlay { prior: Sessions }` / App.overlay_stack: [P2] | `Dashboard { focused: Sessions }` (after `PermissionPromptResolved` for P2 received) | Sends `{"decision":"accept"}` to P2's stalled HTTP response; P2 unblocks; sends `PermissionPromptResolved { prompt_id: P2 }` to TUI |
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

    /// Permission prompt queue — single source of truth for the modal stack.
    /// `AppMode::Overlay` carries only `prior: FocusSnapshot`; the prompts
    /// themselves live here. This decouples stack mutation (push/retain) from
    /// AppMode transitions so `transition()` remains a pure (AppMode, Action) → AppMode
    /// function with no access to prompt data.
    pub overlay_stack: VecDeque<PromptModal>,

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

    /// IPC send channel — for outbound ClientToServer messages (e.g., PermissionDecision).
    pub ipc_tx: tokio::sync::mpsc::Sender<ClientToServer>,

    /// IPC receive channel — for inbound ServerToClient daemon pushes.
    pub ipc_rx: tokio::sync::mpsc::Receiver<ServerToClient>,
}
```

### Draw Loop

> **Async event loop note (F-P13-003):** The `event::poll()` and `event::read()` calls
> in the sketch below are **synchronous blocking calls** from `crossterm`. In the actual
> Phase 1 implementation, the event loop MUST NOT block the Tokio async executor thread.
> Two correct approaches:
>
> 1. **`crossterm::event::EventStream`** (preferred): use the `event-stream` feature of
>    `crossterm` to obtain an async `Stream<Item = Result<Event>>`. `select!` on the
>    stream alongside the IPC receiver channel and the tick interval. This is the
>    fully-async, Tokio-native pattern and avoids any blocking.
> 2. **Dedicated OS thread**: run the crossterm event poll on a `std::thread::spawn`
>    thread that communicates with the async task via a bounded `mpsc::channel`. The
>    async task awaits the channel receiver; no blocking on the executor thread.
>
> The sketch below uses the synchronous form for clarity of the control flow; the
> implementation MUST use one of the two async-safe patterns above.

```rust
// monocle-tui/src/main.rs (synchronous sketch — see async note above for implementation guidance)

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
        // NOTE: event::poll() and event::read() are blocking — use EventStream or
        // a dedicated thread in the actual implementation (see async note above).
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            // Single event::read() call — calling read() twice would block waiting
            // for a second event that may never arrive in this tick window.
            match event::read()? {
                Event::Key(key) => {
                    if let Some(binding) = app.dispatcher.resolve(key, &app.mode) {
                        let new_mode = transition(app.mode.clone(), binding.action.clone());
                        app.mode = new_mode;
                        app.handle_action(binding.action).await?;
                    }
                },
                Event::Resize(_, _) => {
                    // ratatui handles resize automatically on next draw
                },
                _ => {}
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
        AppMode::Overlay { .. } => {
            // App.overlay_stack is the single source of truth for queued modals.
            draw_dashboard(frame, main_area, app, /* dimmed */);
            draw_permission_overlay(frame, main_area, app.overlay_stack.front().unwrap(), app.overlay_stack.len());
            if app.overlay_stack.len() > 1 {
                draw_permission_overlay_peek(frame, main_area, &app.overlay_stack[1]);
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

- `AppMode::Overlay` is only produced when entering from `Dashboard` or `Filtering`
  (i.e., when `App.overlay_stack` is known non-empty at the call site — `transition()`
  itself does not inspect the stack; the invariant is enforced by the push path).
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

All 23 Phase 1 BCs for SS-06. Priority P0 must be delivered in Waves 6–7; P1 may
follow in a later wave within Phase 1.

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.06.001 | AppMode State Machine: Compile-Time Mutual Exclusion | P0 |
| BC-2.06.002 | FocusSnapshot: Focus Restored After Overlay/Fullscreen Close | P0 |
| BC-2.06.003 | Action Dispatch: 5-Level Binding Precedence | P0 |
| BC-2.06.004 | `Ctrl-\` Popup: Appears and Dismisses Without State Loss | P0 |
| BC-2.06.005 | Sessions Panel: Session List Renders from IPC State | P0 |
| BC-2.06.006 | Sessions Panel: `/` Filter with Nucleo Fuzzy Match | P1 |
| BC-2.06.007 | Sessions Panel: `Enter` Transitions to Fullscreen | P1 |
| BC-2.06.008 | Permission Overlay: VecDeque Stack Push on PermissionPromptQueued | P0 |
| BC-2.06.009 | Permission Overlay: `[↑↓]` Rotates Stack | P0 |
| BC-2.06.010 | Permission Overlay: Diff Preview via `similar 3` | P1 |
| BC-2.06.011 | Permission Overlay: Accept-Once Keybinding | P0 |
| BC-2.06.012 | Permission Overlay: Accept-Always Keybinding | P0 |
| BC-2.06.013 | Permission Overlay: Reject Keybinding | P0 |
| BC-2.06.014 | Permission Overlay: `[Esc]` Hides Without Rejecting | P0 |
| BC-2.06.015 | Permission Overlay: `[t]` Trace-to-Source Stub | P2 |
| BC-2.06.016 | Permission Overlay: Cleared on Daemon Disconnect | P0 |
| BC-2.06.017 | Permission Response Within Hook Timeout Budget | P0 |
| BC-2.06.018 | Event Ribbon Panel: Rolling Hook Event Log | P1 |
| BC-2.06.019 | Status Bar: Drop Counter Renders Under Load | P0 |
| BC-2.06.020 | Status Bar: Breadcrumb | P1 |
| BC-2.06.021 | Status Bar: Keybinding Hint Line | P1 |
| BC-2.06.022 | Killer Scenario: ≤6 Keystrokes for Dual Permission Resolve | P0 |
| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | P0 |

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
6. **`#[non_exhaustive]` scope for SS-06 `monocle-core` types.** The following types
   are `#[non_exhaustive]`: `PanelId`, `FocusSnapshot`, `BindingSource`, `Action`.
   `AppMode` is explicitly NOT `#[non_exhaustive]` — all match sites must handle every
   variant exhaustively per BC-2.06.001. Rationale: `PanelId`, `FocusSnapshot`,
   `BindingSource`, and `Action` are extended in Phase 2+ without a breaking change;
   `AppMode` must be exhaustively matched so the compiler enforces mutual-exclusion
   invariants at all call sites.
7. **TUI is a client.** No direct mutation of daemon state from `monocle-tui`. All
   mutations flow through IPC `ClientToServer::PermissionDecision` messages. `monocle-tui` has no
   dependency on `monocle-runtime`.
8. **No `println!` in production code.** Use `tracing::debug!` / `tracing::info!`
   for all diagnostic output.

---

## §Trace v1.8.0

**AppMode::Overlay shape sweep — F-S025-ADV4-BLOCKER-001** (2026-05-28):

Closes the BC sweep propagation loop. The BC sweep at commit `6d4fbb3` updated all
SS-06 BCs to the new `AppMode::Overlay { prior: FocusSnapshot }` shape (with
`App.overlay_stack: VecDeque<PromptModal>` as the single source of truth). SS-tui.md
is the architecture source cited by all 23 SS-06 BCs; it was missed by the BC sweep.
An implementer reading this document as their primary authority would have produced
the old shape and conflicted with the already-updated BCs.

Changes in this version:

- **Line 97 (enum definition):** `Overlay { stack: VecDeque<PromptModal>, prior: FocusSnapshot }` →
  `Overlay { prior: FocusSnapshot }`. Added prose explaining that `App.overlay_stack`
  is the single source of truth for the modal stack.
- **Lines 207-209 (transition OverlayCycleNext arm):** Removed `mut stack` field
  destructuring; the arm now passes `mode` through unchanged. Added comment explaining
  that `handle_action()` rotates `App.overlay_stack` directly.
- **Line 235 (Key Invariant 2):** Updated prose to reference `App.overlay_stack` and
  `AppMode::Overlay { prior }` without a `stack` field.
- **Line 257 (handle_ipc_message):** `AppMode::Overlay { ref mut stack, ref prior }` →
  `AppMode::Overlay { ref prior }`; all `stack.*` references updated to `self.overlay_stack.*`.
- **Line 471 (breadcrumb example):** `Overlay { stack, .. }` → `AppMode::Overlay { .. }`
  with `App.overlay_stack len` annotation.
- **Line 630 (overlay push description):** Rewritten to reference `App.overlay_stack`
  as push target for both the first-enter and already-in-Overlay paths.
- **Lines 808-810 (killer scenario table):** AppMode-before/after columns updated to
  `Overlay { prior: Sessions }` with separate `App.overlay_stack` annotation.
- **Line 940 (draw function Overlay arm):** `AppMode::Overlay { stack, .. }` →
  `AppMode::Overlay { .. }`; `stack.*` references updated to `app.overlay_stack.*`.
- **App struct:** Added `overlay_stack: VecDeque<PromptModal>` field with explanatory
  doc comment establishing it as the single source of truth.
- **Formal verification section:** Updated to remove the stale "no Overlay with empty
  stack" property (no longer a transition() invariant since the stack lives outside
  AppMode); replaced with the correct invariant scoped to the push path.

SS-ipc.md checked and confirmed clean — the §TUI IPC Read Loop Pattern section
(added at commit `27c1ff0`) references `overlay_stack: Vec<PermissionPromptPayload>`
(IPC wire field on daemon side) only; no `Overlay { stack }` shape references found.

## §Trace v1.7.0

**Keybinding canonicalization: mnemonic set replaces numeric set; IPC-driven pop semantics** (2026-05-27):
- **Keybindings updated** to match BC-2.06.011 / BC-2.06.012 / BC-2.06.013 v1.1.0:
  - `Accept-Once`: `y` or `Enter` (was `[1]`)
  - `Accept-Always`: `A` (was `[2]`)
  - `Reject`: `n` or `r` (was `[3]`)
  - `Esc`: No-op (unchanged)
  - Affected locations: §Dispatcher Logic comment, §Status Bar keybinding hint line,
    §Overlay Stack Lifecycle Step 3, §Killer Scenario table.
- **Pop semantics corrected** to match BC-2.06.011/012/013 v1.1.0 and BC-2.06.023:
  - `transition()` decision arms (`PermissionAcceptOnce`, `PermissionAcceptAlways`,
    `PermissionReject`) no longer pop the front `PromptModal`. Decision actions leave
    the `AppMode::Overlay` stack unchanged; the TUI sends
    `ClientToServer::PermissionDecision` and waits for the daemon's
    `ServerToClient::PermissionPromptResolved { prompt_id }` response.
  - Prompt removal continues to be handled by `handle_ipc_message()` via
    `stack.retain(|m| m.prompt_id != prompt_id)` (BC-2.06.023 path, unchanged).
  - Key Invariant 2 rewritten: collapse from `Overlay` to `Dashboard` on empty stack
    is triggered only by the IPC path, never by decision `Action` dispatch.
  - §Overlay Stack Lifecycle Step 3 rewritten to reflect the IPC-round-trip removal
    model.
  - §Killer Scenario table updated: AppMode-after column reflects that mode transition
    happens on `PermissionPromptResolved` receipt, not on keypress.

## §Trace v1.6.0

**PR review findings** (F-P13-001, F-P13-003) (2026-05-26):
- **F-P13-001** [CRITICAL] `EnrichedSession` field mismatch in Sessions Panel column table —
  corrected the column layout table to reference fields that actually exist on `EnrichedSession`.
  Removed `phase_tag` (requires `FactoryAdapter` integration not available in Phase 1) and
  `uptime` (no dedicated field; computed from `started_at` at render time). Replaced with
  `status` (from `EnrichedSession::status`) and updated the Uptime row to source from
  `EnrichedSession::started_at`. Added a note explaining the exclusions. The corresponding
  `EnrichedSession` struct expansion (adding `project_name`, `started_at`, `token_count`,
  `cost_usd`) is recorded in SS-engine-module.md §Trace v1.1.21.
- **F-P13-003** [HIGH] sync/async draw loop — added "Async event loop note" block above the
  draw loop sketch. The note clarifies that `crossterm::event::poll()` and `event::read()` are
  synchronous blocking calls and MUST NOT be used directly on the Tokio executor thread in the
  actual implementation. Specifies two correct async-safe patterns: (1) `crossterm::event::EventStream`
  with `select!` (preferred), (2) dedicated `std::thread` communicating via bounded `mpsc`. The
  sketch is labeled "(synchronous sketch — see async note above for implementation guidance)" and
  includes an inline comment at the poll/read site reinforcing the constraint.

## §Trace v1.5.0

**Adversarial Pass 6 review corrections** (F-P1D6-002, F-P1D7-002) (2026-05-26):
- **F-P1D6-002** [CRITICAL] `DaemonState` wrong type for queued prompts — corrected
  all references that implied the daemon stores `VecDeque<PromptModal>`. The daemon
  stores pending prompts as `Vec<PermissionPromptPayload>` in its pending-decision
  registry. The IPC `InitialState` push sends `overlay_stack: Vec<PermissionPromptPayload>`.
  The TUI converts each to a `PromptModal` via `payload_to_modal()` on receipt.
  `PromptModal` contains `received_at: std::time::Instant` which is not serializable
  and is a TUI-side display type; it must not appear in the daemon's persistent state
  or on the IPC wire. Updated: §Ctrl-\ Integration "Critical implication" paragraph,
  §Ctrl-\ Integration reconnect bullet, and §Killer Scenario precondition.
- **F-P1D7-002** [CRITICAL] Fabricated IPC type aliases — `IpcServerMessage` and
  `IpcClientMessage` are not canonical types. Replaced ALL occurrences with the
  canonical names from SS-ipc.md: `IpcServerMessage` → `ServerToClient`,
  `IpcClientMessage` → `ClientToServer`. Also replaced `IpcMessage::DecisionResponse`
  → `ClientToServer::PermissionDecision` (canonical variant name) in §Overlay Stack
  Lifecycle step 3 and Constraint 7. Replaced `IpcMessage::PermissionPromptQueued`
  → `ServerToClient::PermissionPromptQueued` in §Overlay Stack Lifecycle step 1.
  Affected locations: `App` struct `ipc_tx`/`ipc_rx` field types, overlay lifecycle
  push step, overlay lifecycle decision step, Constraint 7.

## §Trace v1.4.0

**Adversarial Pass 4 review corrections** (F-P1D4-001, F-P1D4-008) (2026-05-26):
- **F-P1D4-001** [CRITICAL] `ToolPayload::Generic` struct mismatch — corrected the
  `Generic` variant definition from `{ raw: serde_json::Value }` to
  `{ tool_name: String, tool_input: serde_json::Value }`. The previous definition was
  inconsistent with every call site in `payload_to_modal()`, which constructed the variant
  with `tool_name` and `tool_input` fields — the code would not compile against the old
  type. `tool_name` is carried inside `Generic` so the overlay header can display the
  unrecognised tool name without an additional lookup. The `PromptModal::tool_name` field
  is retained and now populated from `payload.tool_name.clone()` for all variants (not
  left as `String::new()`) so callers rendering the modal header do not need to pattern-match
  `ToolPayload` to display the tool name.
- **F-P1D4-008** [HIGH] Empty `old_content`/`new_content` fallback for `"Edit"` arm —
  added a match guard to the `"Edit"` arm in `payload_to_modal()`: the arm now only
  activates when `payload.old_content.is_some() || payload.new_content.is_some()`. When
  both are `None`, the fallthrough `_` arm produces `ToolPayload::Generic` instead of
  `ToolPayload::Edit { old_content: "", new_content: "" }`. An `Edit` with no content to
  diff renders as an empty diff pane, which is confusing and wastes overlay layout space;
  the Generic fallback renders the raw `tool_input` JSON instead. Updated the conversion
  sketch to use `"Edit" if (payload.old_content.is_some() || payload.new_content.is_some())`
  as the arm head, with an explanatory comment. The `_` fallback arm comment now explicitly
  names both cases it covers: (a) `"Edit"` with both content fields absent, (b) any
  unrecognised tool name.

## §Trace v1.3.0

**Adversarial Pass 3 review corrections** (F-P1D3-004, F-P1D3-005, F-P1D3-007) (2026-05-26):
- **F-P1D3-004** [HIGH] Profile picker AppMode design gap — added §"Profile Picker:
  Transient Overlay (Not an AppMode Variant)" in the AppMode section. The profile picker
  is modeled as `Option<ProfilePickerState>` in the `App` struct, NOT as an `AppMode`
  variant. Rationale: the picker is a brief transient interaction that can appear over
  any `AppMode` without replacing it; no focus restoration is needed on close; adding an
  `AppMode` variant would inflate all `match` sites for a variant with no distinct transition
  semantics. The section specifies the `App` struct field signature, the draw-loop nil-check
  pattern, and the action dispatch short-circuit rule while `picker.is_some()`.
- **F-P1D3-005** [HIGH] `PermissionPromptPayload` → `PromptModal` conversion unspecified —
  added §"IPC Payload to PromptModal Conversion" immediately before §"Overlay Stack
  Lifecycle". Specifies the canonical field-by-field mapping: `prompt_id` (Uuid→Uuid),
  `session_id` (String→String), `tool_name` → `ToolPayload` variant selection with fallback
  rules for missing `tool_input` fields, and `received_at` set to `Instant::now()` at TUI
  reception time (not deserialized from IPC). Includes a Rust sketch of `payload_to_modal()`.
- **F-P1D3-007** [HIGH] BC-2.06.023 Invariant 2 — `transition()` scope boundary for
  IPC-initiated prompt removal — added clarifying block immediately after the Key Invariants
  list in §Transition Function Contract. Specifies that `PermissionPromptResolved` IPC
  messages are handled by `handle_ipc_message()` (VecDeque `retain()` by `prompt_id`),
  NOT by dispatching an `Action` through `transition()`. The empty-stack-to-Dashboard
  collapse reuses the same invariant as the action path but is triggered by `stack.is_empty()`
  after `retain()`. `Action::PermissionPromptResolved` is not a variant and must not exist.

## §Trace v1.2.0

**Adversarial Pass 2 review corrections** (F-P1D2-004, F-P1D2-005, F-P1D2-013) (2026-05-26):
- **F-P1D2-004** Priority drift — corrected 3 BCs to match BC-INDEX (source of truth):
  - BC-2.06.007 corrected P0 → P1 (BC-INDEX §SS-06 row 7: P1).
  - BC-2.06.010 corrected P0 → P1 (BC-INDEX §SS-06 row 10: P1).
  - BC-2.06.018 corrected P0 → P1 (BC-INDEX §SS-06 row 18: P1).
- **F-P1D2-005** Added missing BC-2.06.023 row to §Behavioral Contracts table:
  `BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | P0`.
  Updated section count text from "All 22 Phase 1 BCs" to "All 23 Phase 1 BCs".
- **F-P1D2-013** Rewrote Constraint 6 to eliminate ambiguity about `#[non_exhaustive]`
  scope. The previous wording stated "All monocle-core SS-06 types are #[non_exhaustive]"
  then named `PanelId` and `FocusSnapshot` as examples in the exception clause, creating
  confusion about which types were the general rule and which were the exception. The
  rewrite explicitly enumerates the non_exhaustive types (`PanelId`, `FocusSnapshot`,
  `BindingSource`, `Action`) and states unambiguously that `AppMode` is NOT
  `#[non_exhaustive]` — exhaustive matching is required per BC-2.06.001.

## §Trace v1.1.0

**Adversarial review corrections** (F-P1D-004, F-P1D-013) (2026-05-26):
- **F-P1D-004** Priority sync with BC-INDEX (source of truth):
  - BC-2.06.006 corrected P0 → P1 (BC-INDEX §SS-06 row 6: P1).
  - BC-2.06.015 corrected P1 → P2 (BC-INDEX §SS-06 row 15: P2).
- **F-P1D-013** Fixed double `event::read()` bug in draw loop sketch. The original sketch
  called `event::read()` once inside `if let Event::Key(...)` and again inside a second
  `if let Event::Resize(...)` block. The second call would block the event loop waiting for
  a second event that may never arrive within the tick window, causing up to 16ms of
  spurious stall per tick when only a key event arrived. Replaced with a single
  `event::read()` + `match` expression covering `Event::Key`, `Event::Resize`, and `_ => {}`.

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
