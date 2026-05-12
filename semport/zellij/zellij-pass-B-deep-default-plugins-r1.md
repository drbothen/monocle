# Phase B Deep — Default Plugins Survey (Round 1)

13 plugins ship with zellij at this HEAD. This pass surveys their structure and lifts ONE representative pattern from each archetype — not a per-plugin deep dive.

## Plugin Catalog (LOC)

| Plugin | LOC | Archetype |
|---|---|---|
| `session-manager` | 5,955 | **Stateful UI** — interactive session picker with attach/resurrect/kill |
| `status-bar` | 5,945 | **Mode-aware decoration** — bottom bar reflecting current InputMode + keybinds |
| `layout-manager` | 4,344 | Stateful UI (out of scope per user: layout-coupled) |
| `configuration` | 3,801 | **Reconfigure UI** — mutates host config via plugin commands |
| `share` | 2,488 | Web-sharing UI |
| `about` | 2,477 | Splash screen + tips (least interesting) |
| `compact-bar` | 2,420 | Mode-aware decoration variant |
| `strider` | 1,338 | **Filesystem-permission consumer** — ranger-like file browser |
| `plugin-manager` | 1,103 | Plugin install/manage UI |
| `fixture-plugin-for-tests` | 1,005 | Test harness |
| `tab-bar` | 796 | **Mode-aware decoration** — top tab bar (smallest "real" plugin) |
| `multiple-select` | 695 | Pane selection helper |
| `link` | 474 | OSC-8 hyperlink follower |

## Archetype 1 — Mode-Aware Decoration (tab-bar)

`default-plugins/tab-bar/src/main.rs:1-80`:

```rust
mod line;
mod tab;

use zellij_tile::prelude::*;

#[derive(Default, Debug)]
struct State {
    tabs: Vec<TabInfo>,
    active_tab_idx: usize,
    mode_info: ModeInfo,
    tab_line: Vec<LinePart>,
    hide_swap_layout_indication: bool,
    cached_keybinds: KeybindsVec,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.hide_swap_layout_indication = configuration
            .get("hide_swap_layout_indication")
            .map(|s| s == "true")
            .unwrap_or(false);
        set_selectable(false);                 // decoration; never receives focus
        subscribe(&[
            EventType::TabUpdate,
            EventType::ModeUpdate,
            EventType::Mouse,
            EventType::InitialKeybinds,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::InitialKeybinds(keybinds) => {
                self.cached_keybinds = keybinds;
                if !self.cached_keybinds.is_empty() {
                    self.mode_info.keybinds = self.cached_keybinds.clone();
                }
                should_render = true;
            },
            Event::ModeUpdate(mut mode_info) => {
                // Use cached keybinds if ModeUpdate didn't ship them
                if mode_info.keybinds.is_empty() && !self.cached_keybinds.is_empty() {
                    mode_info.keybinds = self.cached_keybinds.clone();
                } else if !mode_info.keybinds.is_empty() {
                    self.cached_keybinds = mode_info.keybinds.clone();
                }
                if self.mode_info != mode_info {
                    should_render = true;
                }
                self.mode_info = mode_info;
            },
            Event::TabUpdate(tabs) => {
                if let Some(active_tab_index) = tabs.iter().position(|t| t.active) {
                    self.active_tab_idx = active_tab_index;
                    self.tabs = tabs;
                    should_render = true;
                }
            },
            // mouse handling elided
            _ => {},
        }
        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        // builds ANSI escape sequences using ansi_term
        // prints to stdout (the host's WASI stdin sees them)
    }
}
```

Pattern highlights:

1. **`set_selectable(false)`** — decoration plugins never receive focus. Critical for `tab-bar`/`status-bar` not to "steal" cursor focus.
2. **`InitialKeybinds` subscription pattern** — plugin asks for keybinds ONCE on load, caches them, and then subsequent `ModeUpdate` events arrive WITHOUT keybinds (lighter payload). The plugin must reattach cached keybinds to incoming `ModeUpdate.keybinds` if empty.
3. **`Default` impl on State struct** — required by `ZellijPlugin: Default`. Trivial when all fields have natural defaults.
4. **Configuration via `BTreeMap<String, String>`** — `hide_swap_layout_indication` is a config flag, parsed string→bool inline.
5. **Plugin output is ANSI escape sequences** — `render` writes to stdout (the WASI stdout pipe, which the host captures and forwards to the screen).

`status-bar/src/main.rs` follows essentially the same pattern with more state (TabInfo, ModeInfo, tip system, copy notifications, classic_ui flag).

## Archetype 2 — Filesystem-Permission Consumer (strider)

`default-plugins/strider/src/main.rs:1-80`:

```rust
impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        let plugin_ids = get_plugin_ids();
        let initial_cwd_str = plugin_ids.initial_cwd.to_string_lossy().to_string();
        let platform = Platform::detect(&initial_cwd_str);
        self.platform = platform;
        self.initial_cwd = Platform::normalize(&plugin_ids.initial_cwd);
        self.file_list_view.platform = platform;
        self.search_view.platform = platform;

        let show_hidden_files = configuration
            .get("show_hidden_files")
            .map(|v| v == "true")
            .unwrap_or(false);
        self.hide_hidden_files = !show_hidden_files;
        self.close_on_selection = configuration
            .get("close_on_selection")
            .map(|v| v == "true")
            .unwrap_or(false);

        subscribe(&[
            EventType::Key,
            EventType::Mouse,
            EventType::CustomMessage,        // worker replies
            EventType::Timer,
            EventType::FileSystemUpdate,      // requires permission!
            EventType::HostFolderChanged,
            EventType::PermissionRequestResult,
        ]);

        // No explicit request_permission() call here — the FileSystemUpdate subscribe
        // alone triggers a permission prompt for ReadApplicationState on first use.

        self.file_list_view.clear_selected();

        match configuration.get("caller_cwd").map(|c| Platform::normalize(&PathBuf::from(c))) {
            Some(caller_cwd) => { self.file_list_view.path = caller_cwd; },
            None => { self.file_list_view.path = self.initial_cwd.clone(); },
        }

        if self.initial_cwd != self.file_list_view.path {
            change_host_folder(self.file_list_view.path.clone());
        } else {
            scan_host_folder(&"/host");
        }
    }
    // ...
}
```

Pattern highlights:

1. **`get_plugin_ids()` host call** — returns `PluginIds { plugin_id, zellij_pid, initial_cwd }`. Plugin knows its own identity at startup.
2. **`subscribe` to filesystem events** — `FileSystemUpdate`, `FileSystemCreate/Read/Delete` (declared but not consumed here).
3. **`scan_host_folder("/host")`** — explicit request to scan the special `/host` mount (= user's CWD). Triggers a permission prompt for the relevant permission type if not yet granted.
4. **`change_host_folder(path)`** — request to change which directory `/host` maps to.

The plugin author doesn't write file-system access code directly; they call `scan_host_folder` and receive `FileSystemUpdate` events. The host enforces permission at the boundary.

## Archetype 3 — Stateful Interactive UI (session-manager)

`default-plugins/session-manager/src/main.rs:1-95`:

```rust
#[derive(Default)]
struct State {
    session_name: Option<String>,
    sessions: SessionList,
    resurrectable_sessions: ResurrectableSessions,
    search_term: String,
    new_session_info: NewSessionInfo,
    renaming_session_name: Option<String>,
    error: Option<String>,
    active_screen: ActiveScreen,
    colors: Colors,
    is_welcome_screen: bool,
    is_multi_screen: bool,
    single_screen_state: SingleScreenState,
    show_kill_all_sessions_warning: bool,
    request_ids: Vec<String>,
    is_web_client: bool,
    current_session_last_saved_time: Option<u64>,
    is_visible: bool,
    refresh_timer_armed: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.is_welcome_screen = configuration.get("welcome_screen").map(|v| v == "true").unwrap_or(false);
        if self.is_welcome_screen {
            self.active_screen = ActiveScreen::NewSession;
        }
        // ...
        self.is_visible = true;
        subscribe(&[
            EventType::ModeUpdate,
            EventType::SessionUpdate,       // <-- this fires every time sessions change
            EventType::Key,
            EventType::RunCommandResult,
            EventType::Timer,
            EventType::Visible,
        ]);
        rename_plugin_pane(get_plugin_ids().plugin_id, "Session Manager");
        self.refresh_session_list();
        if !self.is_welcome_screen {
            self.arm_refresh_timer();
        }
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if pipe_message.name == "filepicker_result" {
            match (pipe_message.payload, pipe_message.args.get("request_id")) {
                (Some(payload), Some(request_id)) => {
                    // ...
                }
                _ => {}
            }
        }
        // ...
    }
    // ...
}
```

Pattern highlights:

1. **Massive state struct** — interactive UIs accumulate state. Sub-state types (`SessionList`, `ResurrectableSessions`, `NewSessionInfo`, `SingleScreenState`) are defined in sibling files.
2. **Subscribed to `SessionUpdate`** — fires every time the live session list OR resurrectable list changes (`Event::SessionUpdate(Vec<SessionInfo>, Vec<(String, Duration)>)`).
3. **`Event::Timer(f64)` for refresh** — the plugin sets a timer via `set_timeout(seconds)`; the host fires `Event::Timer(seconds)` when it elapses. Pattern: `arm_refresh_timer` calls `set_timeout(1.0)`; on `Timer` event, refresh and re-arm.
4. **`pipe()` for cross-plugin messaging** — receives messages by name + payload. Used here for "filepicker_result" — when the user uses strider to pick a file from session-manager's UI, the picked path comes back as a pipe message.
5. **`rename_plugin_pane(plugin_id, "Session Manager")`** — plugin renames its own pane.
6. **Configuration flags**: `welcome_screen`, `multi_screen` — drive different UI flows from the same plugin.

The session-manager **directly maps onto monocle's session-bookmark feature**:
- Live sessions table → `Event::SessionUpdate.0` (Vec<SessionInfo>)
- Resurrectable sessions table → `Event::SessionUpdate.1` (Vec<(String, Duration)>)
- Attach action → host command `SwitchSession`
- Delete dead → host command `DeleteDeadSession`
- New session → host command `NewTab` + `SwitchSession`

## Archetype 4 — Reconfigure UI (configuration)

`default-plugins/configuration/src/main.rs:1-80`:

```rust
struct State {
    notification: Option<String>,
    is_setup_wizard: bool,
    ui_size: usize,
    current_screen: Screen,        // enum: RebindLeaders or Presets
    latest_mode_info: Option<ModeInfo>,
    colors: Styling,
}

#[derive(Debug)]
enum Screen {
    RebindLeaders(RebindLeadersScreen),
    Presets(PresetsScreen),
}
```

This plugin's interesting trait: it *modifies the running config*. Two paths:

1. **Per-key edits**: build `keys_to_rebind` and `keys_to_unbind` vectors, send via host command (eventually surfacing as `ServerInstruction::RebindKeys`).
2. **Wholesale preset apply**: serialize a full `Config` to KDL string, send `PluginCommand::Reconfigure(stringified_kdl, write_to_disk: bool)`.

Both flows update the **per-client runtime config** (`SessionConfiguration::runtime_config[client_id]`), and optionally the on-disk file.

This is a UNIQUE design: the user can edit their config via a plugin without ever opening their editor, and the plugin's UI rendering uses the *current effective* `ModeInfo` (which includes keybinds) so the displayed bindings are always up-to-date.

## Plugin SDK Surface Used (sample)

Across these representative plugins:

| Host call | Source | Used by |
|---|---|---|
| `subscribe(&[EventType::...])` | shim.rs:50 | all |
| `unsubscribe(&[EventType::...])` | shim.rs:59 | rare |
| `set_selectable(bool)` | shim.rs:68 | tab-bar, status-bar, compact-bar (decorations) |
| `set_timeout(f64)` | shim.rs | session-manager (for polling) |
| `get_plugin_ids() -> PluginIds` | shim.rs:99 | strider, session-manager |
| `request_permission(&[PermissionType])` | shim.rs:85 | strider (file system access) |
| `change_host_folder(PathBuf)` | shim.rs | strider |
| `scan_host_folder(&str)` | shim.rs | strider |
| `rename_plugin_pane(id, name)` | shim.rs | session-manager |
| `switch_to_input_mode(InputMode)` | shim.rs | configuration |
| `print_text_with_coordinates(...)`, `print_table_with_coordinates(...)` | shim.rs (UI components) | all plugins via Text/Table widgets |
| `report_panic(string)` | shim.rs | all (on panic-handler hook) |

## UI Component Library

`zellij-tile/src/ui_components/` (4 files, ~787 LOC):

```
nested_list.rs        167  -- nested-bullet-list widget
ribbon.rs              81  -- horizontal mode/tab ribbon
table.rs               84  -- multi-column table with selection
text.rs               455  -- styled-text builder with color_range / color_indices
```

These are tile-side widgets that produce ANSI bytes via the `print_*` shim functions. They're the SDK's "make rendering ergonomic" layer — without them, every plugin would hand-craft cursor positioning + color escapes.

`Text::new(string).color_range(palette_idx, range)` — common idiom (see `resurrectable_sessions.rs:31` from session-manager).

## Recommendations for Monocle Plugin SDK Surface

If monocle adopts the WASM factory-adapter model:

| Element | Why |
|---|---|
| `register_plugin!(State)` macro | Removes boilerplate; defines `_start`, `load`, `update`, `pipe`, `render` exports |
| `FactoryAdapter: Default` trait (analog of `ZellijPlugin`) | Standardized lifecycle hooks |
| `register_worker!(Worker)` for background tasks | `*_worker`-suffix convention |
| `subscribe(&[EventType])` semantic event filter | Each adapter declares what it cares about |
| Built-in widget library (`text`, `table`, `ribbon`, `nested_list`) compiled into the SDK | Plugin authors render structured UI without ANSI knowledge |
| `set_timeout(f64)` for polling | Avoids spawning OS timers from inside wasm |
| `get_plugin_ids()` returns `PluginIds { plugin_id, zellij_pid, initial_cwd }` | Self-identity at startup |
| `request_permission(&[PermissionType])` | First-use permission prompt |
| `report_panic(string)` | Crash diagnostics propagation |
| Configuration as `BTreeMap<String, String>` passed to `load()` | Simple, no schema-coupling between SDK and host |

## Coverage Notes

| Investigated | Coverage |
|---|---|
| 13 plugins inventoried | full |
| 4 archetypes documented in depth (decoration, fs-permission, stateful UI, reconfigure) | sufficient |
| Plugin SDK surface usage table | covers ~12 host-call types |
| UI components library | enumerated (4 widget types) |

## Open Items After This Round

| Item | Notes |
|---|---|
| `share`, `multiple-select`, `layout-manager`, `link`, `plugin-manager` not deepened | Per user scope: representative pass only. Each is a variant of one of the four archetypes above. |
| Worker example | None of the 13 plugins inspected here use a worker. `strider` may; not confirmed. Consider deepening if monocle wants worker examples. |
| `fixture-plugin-for-tests` | Test fixture only; not user-facing. Out of scope. |

## Round Status

```yaml
pass: B
category: default-plugins-survey
round: 1
status: complete
timestamp: 2026-05-11T21:10:00Z
new_findings:
  - "Four archetypes: decoration (set_selectable(false)), filesystem-permission consumer (strider), stateful interactive UI (session-manager), reconfigure UI (configuration)"
  - "InitialKeybinds subscription pattern: plugin caches keybinds at load, subsequent ModeUpdate events ship empty keybinds (payload optimization)"
  - "Plugin configuration is BTreeMap<String, String> -- string-typed, parsed inline in load()"
  - "Timer pattern: set_timeout(seconds) -> Event::Timer(seconds); re-arm in handler for polling"
  - "pipe() method receives cross-plugin messages by name + payload (e.g. session-manager <- strider for filepicker_result)"
  - "Plugins can hot-swap host config via Reconfigure(stringified_kdl, save_to_disk) -- huge for in-app config editors"
  - "Built-in UI widgets: Text, Table, Ribbon, NestedList (in zellij-tile/src/ui_components/)"
classification: substantive
```
