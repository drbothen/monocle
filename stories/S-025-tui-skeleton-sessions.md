---
document_type: story
level: L4
story_id: S-025
epic_id: EPIC-06
version: "1.5"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-28T00:00:00Z
phase: 2
points: 8
wave: 6
tdd_mode: strict
priority: P0
depends_on: [S-024, S-022, S-030]
blocks: [S-027, S-028, S-031]
target_module: monocle-tui
subsystems: [SS-06]
behavioral_contracts: [BC-2.06.004, BC-2.06.005, BC-2.06.007, BC-2.05.002]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.004.md, version: "1.2.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.005.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.007.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md, version: "1.0.5"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "a47e758"
traces_to: "Implements BC-2.06.004 (Ctrl-\\ popup: appears and dismisses without state loss), BC-2.06.005 (Sessions panel rendering), BC-2.06.007 (Sessions panel: Enter transitions to fullscreen), BC-2.05.002 Invariant 4 (apply_permission_prompt_queued idempotency helper)"
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
On startup, `monocle-tui` calls `load_config(MonocleConfig::config_path()?)` (the free
function from `monocle-config`, NOT a `MonocleConfig::load` method — no such method
exists). If `config_path()` returns `Err(ConfigError::HomeDirUnresolvable)`, the TUI
logs the error and falls back to `MonocleConfig::default()`. Config load errors other
than missing file (e.g., `ParseError`, `SchemaMismatch`) are displayed to the user in a
modal before the TUI proceeds with defaults.

### AC-005 (traces to BC-2.06.005 postcondition PC-1 — Sessions panel renders session list)
The Sessions panel renders a scrollable list of active sessions. Each row shows six
columns: icon, project name, status, token count, cost, uptime (in that order). For
example: `● monocle Active 437k — 03:47:00`. The list is sourced from
`ServerToClient::SessionListUpdate` IPC messages. If no sessions are active, the panel
renders two lines:
```
No sessions detected
Start Claude Code in any terminal to see it here.
```

### AC-006 (traces to BC-2.06.005 postcondition PC-2 — Sessions panel keyboard navigation)
In `Dashboard { focused: FocusSnapshot::Sessions }` mode:
- `j` / `↓` moves selection down one row
- `k` / `↑` moves selection up one row
- `Enter` enters `Fullscreen { panel: PanelId::Sessions, prior: current_focus }`
- `Tab` cycles focus to the next panel
These key actions are dispatched via `resolve_binding()` from S-024 using the `Global`
binding layer.

### AC-007 (traces to BC-2.06.005 postcondition PC-3 — Sessions panel drop counter)
The Sessions panel status bar shows the drop counter from `ServerToClient::DropCounterUpdate { drop_counter: u64 }`.
When `drop_counter > 0`, the status bar renders: `"[dropped: N]"` in yellow. When `drop_counter == 0`,
no drop indicator is shown.

### AC-008 (traces to BC-2.06.004 postcondition PC-4 — daemon overlay_stack sync on connect; also traces to BC-2.05.002 Invariant 4 — idempotent insert)
On initial IPC connection, the daemon sends `ServerToClient::InitialState { overlay_stack, sessions, .. }`.
The TUI initializes its local `VecDeque<PromptModal>` from `overlay_stack`. Population MUST
use the `apply_permission_prompt_queued(overlay, payload)` helper (see Tasks) for each entry
in `overlay_stack` so that idempotent-on-`prompt_id` semantics are enforced: if a `prompt_id`
is already present in the VecDeque (e.g., from a streaming `PermissionPromptQueued` that
arrived before `InitialState`), the duplicate MUST be silently discarded. If the overlay
stack is non-empty after population, the TUI immediately transitions to
`Overlay { prior: default_focus }` before rendering the first frame.
(The modal stack is carried in `App.overlay_stack`, not in the `Overlay` variant, per BC-2.06.004 v1.2.0 PC-2.)
(BC-2.05.002 Invariant 4: the IPC layer provides at-least-once delivery for
`PermissionPromptQueued` across the snapshot window; consumer idempotency on `prompt_id` is
the correct resolution.)

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
| BC-2.05.002.md (Invariant 4) | ~300 |
| S-024 (AppMode, transition) | ~800 |
| S-022 (UDS IPC types) | ~600 |
| S-030 (MonocleConfig) | ~500 |
| ratatui layout patterns | ~400 |
| Test files | ~1,000 |
| **Total estimate** | **~8,400** |

## Tasks

- [ ] Create `monocle-tui/` binary crate: `Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/ui/mod.rs`
- [ ] Add `monocle-tui` to workspace `Cargo.toml` members
- [ ] Implement `App` struct in `app.rs` with fields: `mode: AppMode`, `config: MonocleConfig`, `sessions: Vec<SessionState>`, `drop_counter: u64`, `overlay_stack: VecDeque<PromptModal>` (local TUI copy)
- [ ] Implement terminal setup in `main.rs`: `enable_raw_mode()`, `EnterAlternateScreen`, panic hook for terminal restore
- [ ] Implement terminal teardown: `disable_raw_mode()`, `LeaveAlternateScreen` — called on all exit paths
- [ ] Implement UDS connection attempt in `app.rs`: connect to `<runtime_dir>/monocle.sock`, display error panel on failure
- [ ] Implement idempotent `apply_permission_prompt_queued(overlay: &mut VecDeque<PromptModal>, payload: PermissionPromptPayload)` helper: checks `overlay.iter().any(|m| m.prompt_id == payload.prompt_id)` before `push_back`; if already present, logs at TRACE level and returns without inserting (BC-2.05.002 Invariant 4). This helper is used for BOTH `InitialState` population and streaming `PermissionPromptQueued` handling (S-026 reuses it).
- [ ] Implement `ServerToClient::InitialState` handler: initialize `overlay_stack` from `initial_state.overlay_stack` via `apply_permission_prompt_queued` (idempotent), transition to `Overlay` if non-empty
- [ ] Implement `ServerToClient::DropCounterUpdate { drop_counter }` handler: update `app.drop_counter`
- [ ] Implement `TransportEvent::Disconnected` handler: clear overlay, transition to `Dashboard`, show reconnect notice in status bar
- [ ] Implement `MonocleConfig::load()` call on startup with error display modal
- [ ] Create `monocle-tui/src/ui/sessions_panel.rs` — render Sessions panel with scrollable list, keyboard nav (j/k/Enter/Tab), drop counter in status bar
- [ ] Implement `resolve_binding()` call in the main event loop using `BindingLayers` from S-024
- [ ] Implement `q` / `Esc` from `Dashboard` mode → clean exit
- [ ] Integration tests `monocle-tui/tests/startup_connect.rs` — mock daemon, verify initial state sync, overlay pre-load
- [ ] Unit tests `monocle-tui/tests/sessions_panel.rs` — render assertions for session rows, empty state, drop counter display

## Previous Story Intelligence

S-022 (UDS server + IPC types): `ServerToClient`, `ClientToServer` canonical type names
(NOT `IpcServerMessage`/`IpcClientMessage`). `ServerToClient::DropCounterUpdate { drop_counter: u64 }`
is the drop counter variant. `ServerToClient::InitialState` is the full state push on connect.
`ServerToClient::SessionListUpdate` is the session list update variant (NOT `SessionState`).
`TransportEvent::Disconnected` is the disconnect signal. `ClientToServer::ClientDisconnect` does NOT exist.

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
- `DropCounterUpdate { drop_counter: u64 }` — not `DropCounterUpdate { count }`, not `StateUpdate`, not any other variant name
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

## §Trace v1.5

**F-S025-ADV3-BLOCKER-002 — SS-06 BC version pins propagated from PO sweep (commit 6d4fbb3)** (2026-05-28):
- BC-2.06.004 inputs pin updated: v1.1.0 → v1.2.0.
- AC-008 body updated: `Overlay { stack: loaded_stack, prior: default_focus }` → `Overlay { prior: default_focus }` with
  explicit note that the modal stack lives in `App.overlay_stack`, not the `Overlay` variant (BC-2.06.004 v1.2.0 PC-2
  arch-pass-2 HIGH-003 propagation — `AppMode::Overlay` carries only `{ prior: FocusSnapshot }`).
- SE-16d monotonicity: v1.5 timestamp 2026-05-28 >= v1.4 timestamp 2026-05-28. PASS.

## §Trace v1.4

**F-S025-ADV3-HIGH-001 — MonocleConfig::load API drift corrected** (2026-05-28):
- Finding: AC-004 referenced `MonocleConfig::load(MonocleConfig::config_path()?)` —
  no such method exists. The actual API is the free function `load_config(&path)` from
  `monocle-config`, confirmed in `crates/monocle-tui/src/app.rs:16,423`.
- Fix: AC-004 updated to reference `load_config(MonocleConfig::config_path()?)` and
  explicitly note the method-vs-free-function distinction to prevent re-introduction.

**F-S025-ADV3-HIGH-002 — Sessions panel columns and empty-state drift corrected** (2026-05-28):
- Finding: AC-005 specified 4 columns (`session_id | harness_type | status | uptime`)
  and empty-state `"No active sessions"`. BC-2.06.005 PC-1 canonically specifies 6
  columns (icon, project, status, tokens, cost, uptime) and empty-state "No sessions
  detected" / "Start Claude Code...". Implementation in `sessions_panel.rs:284-305`
  matches the BC, not the story.
- Root cause: story text was stale from a pre-BC-2.06.005 draft; BC and implementation
  are already in agreement. Story was the outlier.
- Fix: AC-005 updated to 6-column layout with correct empty-state text, matching both
  BC-2.06.005 PC-1 and the implementation. No escalation required.
- SE-16d monotonicity: v1.4 timestamp 2026-05-28 >= v1.3 timestamp 2026-05-28. PASS.

## §Trace v1.3

**F-S022-ADV8-HIGH-001 — BC-2.05.002 Invariant 4 dedup directive propagated** (2026-05-28):
- Finding: Pass 6 architect's Option D decision (dedup-on-insert for `PermissionPromptQueued`)
  was named at the story level in architect-decisions-pass-6.md §Implementer Directive but was
  never propagated into S-025 story content. CLAUDE.md Principle 3 violation — the deferral
  was functionally orphaned.
- Fix: BC-2.05.002 added to `behavioral_contracts` frontmatter and `inputs` list (v1.0.5).
- Fix: AC-008 updated with idempotent-insert precondition and explicit BC-2.05.002 Invariant 4
  citation. Population of `VecDeque<PromptModal>` from `InitialState.overlay_stack` MUST use
  `apply_permission_prompt_queued` helper to enforce idempotency on `prompt_id`.
- Fix: Tasks section updated — `apply_permission_prompt_queued` helper task added as the
  canonical idempotent-insert implementation to be used by both S-025 (InitialState population)
  and S-026 (streaming PermissionPromptQueued handling).
- Token Budget: BC-2.05.002.md row added (~300 tokens); total updated ~8,100 → ~8,400.
- `traces_to` frontmatter updated to include BC-2.05.002 Invariant 4.
- SE-16d monotonicity: v1.3 timestamp 2026-05-28 >= v1.2 timestamp 2026-05-27. PASS.
