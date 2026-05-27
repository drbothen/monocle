---
document_type: story
level: L4
story_id: S-025
epic_id: EPIC-06
version: "1.0"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 8
wave: 6
tdd_mode: strict
priority: P0
depends_on: [S-024, S-022, S-030]
blocks: [S-027, S-028, S-031]
target_module: monocle-tui
subsystems: [SS-06]
behavioral_contracts: [BC-2.06.004, BC-2.06.005, BC-2.06.007]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.004.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.005.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.007.md, version: "1.0.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.06.004 (TUI connect/disconnect lifecycle), BC-2.06.005 (Sessions panel rendering), BC-2.06.007 (Ctrl-\\ popup launch)"
---

# S-025: TUI Binary Skeleton, Ctrl-\ Popup, Sessions Panel

## Narrative

As a daemon operator, I want the `monocle-tui` binary to launch as a tmux popup via
`Ctrl-\`, connect to the monocle daemon over UDS, load configuration, and render the
Sessions panel with live session data, so that I can observe all active harness sessions
from a single overlay without interrupting my editor workflow.

## Acceptance Criteria

### AC-001 (traces to BC-2.06.007 postcondition PC-1 — Ctrl-\ popup launch)
The `monocle-tui` binary, when launched as a tmux popup (via
`tmux display-popup -E monocle tui`), initializes the terminal in alternate screen
mode via `crossterm`, renders the full layout within 200ms of process start, and exits
cleanly when the user presses `q` or `Esc` from `Dashboard` mode (restoring the
terminal to normal mode before exit).

### AC-002 (traces to BC-2.06.004 postcondition PC-1 — IPC connection on startup)
On startup, `monocle-tui` attempts to connect to the monocle daemon via UDS at the
path returned by `resolve_runtime_dir() + "/monocle.sock"`. Connection is attempted
once; if the connection fails, the TUI renders a full-screen error panel:
`"Daemon not running. Start it with: monocle daemon start"` and exits with code 1
after the user presses any key.

### AC-003 (traces to BC-2.06.004 postcondition PC-2 — IPC disconnect handling)
When `TransportEvent::Disconnected` is received on the IPC channel, the TUI transitions
to `Dashboard` mode (discarding any Overlay state) and renders a status bar notification:
`"[disconnected] reconnecting..."`. The TUI does NOT exit; it enters reconnect polling
(see S-023 for reconnect logic). There is NO `ClientDisconnect` IPC message — this
BC-2.06.004 v1.1.0 behavior was removed.

### AC-004 (traces to BC-2.06.004 postcondition PC-3 — config load on startup)
On startup, `monocle-tui` calls `MonocleConfig::load(MonocleConfig::config_path()?)`.
If `config_path()` returns `Err(ConfigError::HomeDirUnresolvable)`, the TUI logs the
error and falls back to `MonocleConfig::default()`. Config load errors other than
missing file (e.g., `ParseError`, `SchemaMismatch`) are displayed to the user in a
modal before the TUI proceeds with defaults.

### AC-005 (traces to BC-2.06.005 postcondition PC-1 — Sessions panel renders session list)
The Sessions panel renders a scrollable list of active sessions. Each row shows:
`<session_id> | <harness_type> | <status> | <uptime>`. The list is sourced from
`ServerToClient::SessionState` IPC messages. If no sessions are active, the panel
renders: `"No active sessions"`.

### AC-006 (traces to BC-2.06.005 postcondition PC-2 — Sessions panel keyboard navigation)
In `Dashboard { focused: FocusSnapshot { panel: PanelId::Sessions, .. } }` mode:
- `j` / `↓` moves selection down one row
- `k` / `↑` moves selection up one row
- `Enter` enters `Fullscreen { panel: PanelId::Sessions, prior: current_focus }`
- `Tab` cycles focus to the next panel
These key actions are dispatched via `resolve_binding()` from S-024 using the `Global`
binding layer.

### AC-007 (traces to BC-2.06.005 postcondition PC-3 — Sessions panel drop counter)
The Sessions panel status bar shows the drop counter from `ServerToClient::DropCounterUpdate { count: u64 }`.
When `count > 0`, the status bar renders: `"[dropped: N]"` in yellow. When `count == 0`,
no drop indicator is shown.

### AC-008 (traces to BC-2.06.004 postcondition PC-4 — daemon overlay_stack sync on connect)
On initial IPC connection, the daemon sends `ServerToClient::FullState { overlay_stack, sessions, .. }`.
The TUI initializes its local `VecDeque<PromptModal>` from `overlay_stack`. If the
overlay stack is non-empty, the TUI immediately transitions to
`Overlay { stack: loaded_stack, prior: default_focus }` before rendering the first frame.

### AC-009 (traces to BC-2.06.007 postcondition PC-2 — alternate screen cleanup on exit)
When the TUI exits (any exit path: `q`, `Esc`, IPC error, SIGTERM), it MUST restore the
terminal to normal mode: call `crossterm::terminal::disable_raw_mode()` and
`crossterm::execute!(stdout, LeaveAlternateScreen)`. A panic handler (via `std::panic::set_hook`)
also restores the terminal before unwinding.

### AC-010 (traces to BC-2.06.004 invariant INV-1 — no ClientDisconnect message)
The `monocle-tui` codebase MUST NOT send or reference a `ClientToServer::ClientDisconnect`
message. This variant does not exist in the IPC type system (removed in BC-2.06.004
v1.1.0). Disconnection is detected exclusively via `TransportEvent::Disconnected`.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~2,000 |
| BC-2.06.004.md | ~1,100 |
| BC-2.06.005.md | ~900 |
| BC-2.06.007.md | ~800 |
| S-024 (AppMode, transition) | ~800 |
| S-022 (UDS IPC types) | ~600 |
| S-030 (MonocleConfig) | ~500 |
| ratatui layout patterns | ~400 |
| Test files | ~1,000 |
| **Total estimate** | **~8,100** |

## Tasks

- [ ] Create `monocle-tui/` binary crate: `Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/ui/mod.rs`
- [ ] Add `monocle-tui` to workspace `Cargo.toml` members
- [ ] Implement `App` struct in `app.rs` with fields: `mode: AppMode`, `config: MonocleConfig`, `sessions: Vec<SessionState>`, `drop_counter: u64`, `overlay_stack: VecDeque<PromptModal>` (local TUI copy)
- [ ] Implement terminal setup in `main.rs`: `enable_raw_mode()`, `EnterAlternateScreen`, panic hook for terminal restore
- [ ] Implement terminal teardown: `disable_raw_mode()`, `LeaveAlternateScreen` — called on all exit paths
- [ ] Implement UDS connection attempt in `app.rs`: connect to `<runtime_dir>/monocle.sock`, display error panel on failure
- [ ] Implement `ServerToClient::FullState` handler: initialize `overlay_stack` from `full_state.overlay_stack`, transition to `Overlay` if non-empty
- [ ] Implement `ServerToClient::DropCounterUpdate { count }` handler: update `app.drop_counter`
- [ ] Implement `TransportEvent::Disconnected` handler: clear overlay, transition to `Dashboard`, show reconnect notice in status bar
- [ ] Implement `MonocleConfig::load()` call on startup with error display modal
- [ ] Create `monocle-tui/src/ui/sessions_panel.rs` — render Sessions panel with scrollable list, keyboard nav (j/k/Enter/Tab), drop counter in status bar
- [ ] Implement `resolve_binding()` call in the main event loop using `BindingLayers` from S-024
- [ ] Implement `q` / `Esc` from `Dashboard` mode → clean exit
- [ ] Integration tests `monocle-tui/tests/startup_connect.rs` — mock daemon, verify initial state sync, overlay pre-load
- [ ] Unit tests `monocle-tui/tests/sessions_panel.rs` — render assertions for session rows, empty state, drop counter display

## Previous Story Intelligence

S-022 (UDS server + IPC types): `ServerToClient`, `ClientToServer` canonical type names
(NOT `IpcServerMessage`/`IpcClientMessage`). `ServerToClient::DropCounterUpdate { count: u64 }`
is the drop counter variant. `TransportEvent::Disconnected` is the disconnect signal.
`ClientToServer::ClientDisconnect` does NOT exist.

S-024 (TUI core types): `AppMode`, `transition()`, `resolve_binding()`, `FocusSnapshot`,
`PanelId`, `PromptModal`, `Action`, `BindingSource`, `BindingLayers` are all available
from `monocle-core::tui`. Import from there — do not redefine.

S-030 (config foundation): `MonocleConfig::load()`, `config_path()`, `detect_ccr()`,
`ConfigError` available from `monocle-config`. Use workspace dep declaration.

## Architecture Compliance Rules

From `architecture/SS-tui-core.md` and `architecture/SS-conventions-anti-patterns.md`:
- `monocle-tui` is the effectful boundary — ratatui, crossterm, nucleo, similar live HERE
- `monocle-core` (pure) is a dependency of `monocle-tui` (effectful) — not the reverse
- `TransportEvent::Disconnected` is the ONLY disconnect signal — no `ClientDisconnect` IPC message
- `overlay_stack` is the IPC field name; local TUI copy is `VecDeque<PromptModal>`
- `ServerToClient` / `ClientToServer` canonical type names — no IpcServerMessage/IpcClientMessage aliases
- Panic hook MUST restore terminal before unwinding — no raw-mode leakage
- `DropCounterUpdate { count: u64 }` — not `StateUpdate`, not any other variant name
- ratatui `StatefulWidget` pattern for scrollable lists; `ListState` for selection tracking

**Forbidden Dependencies:**
- `monocle-tui` MUST NOT be a dependency of `monocle-core` (circular)
- `monocle-tui` MUST NOT use `monocle-daemon` internal types — IPC types from `monocle-ipc` only
- Do NOT define any IPC message types in `monocle-tui` — use `monocle-ipc` crate

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| ratatui | workspace pin (0.30+) | Terminal UI widgets, Layout, StatefulWidget |
| crossterm | workspace pin | Raw mode, alternate screen, event reading |
| monocle-core | workspace path | `AppMode`, `transition()`, `resolve_binding()`, `PromptModal` |
| monocle-config | workspace path | `MonocleConfig`, `detect_ccr`, `ConfigError` |
| monocle-ipc | workspace path | `ServerToClient`, `ClientToServer`, `TransportEvent` |
| tokio | workspace pin (features=["full"]) | Async runtime for IPC and event loop |
| tracing | 0.1 | Structured logging |
| uuid | workspace pin | `Uuid` for `PromptModal.prompt_id` |

## File Structure Requirements

Files to create:
- `monocle-tui/Cargo.toml` — binary crate manifest
- `monocle-tui/src/main.rs` — terminal setup/teardown, panic hook, tokio runtime entry point
- `monocle-tui/src/app.rs` — `App` struct, IPC message handlers, event loop
- `monocle-tui/src/ui/mod.rs` — module declarations
- `monocle-tui/src/ui/sessions_panel.rs` — Sessions panel widget
- `monocle-tui/src/ui/layout.rs` — overall layout (Sessions + EventRibbon panels)
- `monocle-tui/tests/startup_connect.rs` — startup + IPC integration tests
- `monocle-tui/tests/sessions_panel.rs` — Sessions panel render tests

Files to modify:
- `Cargo.toml` (workspace root) — add `monocle-tui` to `members`

## Downstream Consumer Contract

Public API produced by this story for downstream consumption:

```rust
// monocle-tui::app
pub struct App {
    pub mode: AppMode,
    pub config: MonocleConfig,
    pub sessions: Vec<SessionState>,
    pub drop_counter: u64,
    pub overlay_stack: VecDeque<PromptModal>,
}
```

S-026 (permission overlay) and S-027 (overlay rendering + status bar) build on top of
`App` and the `monocle-tui` crate structure established here. S-028 adds Sessions filter
panel to the layout. S-031 (profile picker) adds `Option<ProfilePickerState>` to `App`.
