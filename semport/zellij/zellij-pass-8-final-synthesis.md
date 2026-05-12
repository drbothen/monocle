# Pass 8: Final Synthesis — zellij Scoped Ingest

## Summary

zellij is a production-grade Rust terminal multiplexer with a sophisticated three-layer architecture (client / server / WASM-hosted plugins) and an exemplary set of design choices that monocle's runtime plane will inherit verbatim. The reference is at commit `de1e0f7560d03ce6b5514e22eef7e7852c9385e8` on branch `main`. This ingest focused exclusively on patterns transferable to monocle: workspace topology, client/server IPC, WASM plugin SDK, KDL config, modal keybinds, theming, session persistence, and default-plugin shape. PTY internals, layout geometry, terminal emulation, i18n, SSH plumbing, and host-emulator integration were declared out-of-scope and were not deepened.

## Scope Statement

**In-scope (deepened):** workspace architecture, IPC client/server model, plugin SDK + host, configuration & keybinds, session persistence + resume, theming, default-plugins shape, build/release topology.

**Out-of-scope (mentioned only):** PTY (`pty.rs`, `pty_writer.rs`, `terminal_bytes.rs`, `os_input_output*.rs`), layout geometry (`panes/floating_panes/`, `panes/tiled_panes/`), translations/i18n (not present at HEAD), SSH remote-attach (`zellij-client/src/remote_attach/`), asciinema export (not present), wezterm/alacritty embedding (not present), mouse handling (`mouse.rs`, `*_mouse*`), ANSI stdin parser (`stdin_ansi_parser.rs`, `keyboard_parser.rs`), vendored termwiz (`vendored/termwiz/`), wix MSI installer config.

## Snapshot

| Field | Value |
|---|---|
| HEAD | `de1e0f7560d03ce6b5514e22eef7e7852c9385e8` |
| Branch | `main` |
| Tip commit subject | `docs(changelog): session manager close pr` |
| Commit time | 2026-05-11 16:14:32 +0200 |
| File count (.rs, excluding target/.git) | 307 |
| Total .rs LOC | 275,464 |
| Workspace members | 20 (corrected from initial 23 in B.6) |
| Rust edition | 2021 |
| MSRV | 1.92 |
| License | MIT |
| Async runtime | tokio 1.40 (multi-thread) |
| Wire format | protobuf (`prost` 0.11.9), length-prefixed |
| WASM VM | wasmi 0.51.3 (interpreter) + wasmi_wasi |
| Config language | KDL (`kdl` 4.5.0) |
| Plugin compile target | wasm32-wasip1 |

LOC per top-level crate:

| Crate | .rs LOC |
|---|---|
| `zellij-server/` | 142,410 |
| `zellij-utils/` | 70,395 |
| `zellij-client/` | 18,340 |
| `src/` (root binary) | 5,822 |
| `zellij-tile/` | 3,889 |
| `xtask/` | 1,736 |
| `zellij-tile-utils/` | 31 |

LOC per default plugin: session-manager 5,955; status-bar 5,945; layout-manager 4,344; configuration 3,801; share 2,488; about 2,477; compact-bar 2,420; strider 1,338; plugin-manager 1,103; fixture-plugin-for-tests 1,005; tab-bar 796; multiple-select 695; link 474.

## Workspace Architecture

20-member workspace organized into three logical groups:

1. **Native code (5 libs + 1 bin + 1 tool):** `zellij-utils`, `zellij-client`, `zellij-server`, `zellij-tile-utils`, `zellij-tile`, `.` (the `zellij` bin), `xtask`.
2. **WASM plugins (13 binaries):** all of `default-plugins/*`.
3. **Build orchestrator:** `xtask` (separate from the runtime).

`zellij-utils` is the canonical "shared boundary types" crate. It holds:
- The wire schema (`ipc.rs`, `client_server_contract/*.proto`)
- Domain enums (`data.rs`, 3,656 LOC: `Event`, `EventType`, `Action`, `InputMode`, `KeyWithModifier`, `BareKey`, `PaneId`, `PluginCommand`, `PermissionType`, …)
- Config aggregates (`input/*.rs`: `Config`, `Options`, `Themes`, `Keybinds`, `PluginAliases`, `WebClientConfig`, `PermissionCache`, `EnvironmentVariables`)
- KDL parsing (`kdl/mod.rs` 7,255 LOC + `kdl/kdl_layout_parser.rs` 2,601 LOC)
- Session enumeration + serialization (`sessions.rs`, `session_serialization.rs`)
- Error model (`errors.rs`: `ErrorContext`, `LoggableError`, `FatalError`, `ZellijError`)
- Cross-thread channel wrapper (`channels.rs`: `SenderWithContext`)
- Path constants (`consts.rs`: `ZELLIJ_SOCK_DIR`, `ZELLIJ_CACHE_DIR`, …, `CLIENT_SERVER_CONTRACT_VERSION`)
- CLI parser (`cli.rs`: `CliArgs`, `Command`, `CliAction`)

Internal dependency graph (in-scope only):

```
zellij (bin)
├─ zellij-client    (path dep, optional features mirrored)
├─ zellij-server    (path dep)
└─ zellij-utils     (workspace dep)

zellij-client       ──┐
zellij-server       ──┼─→ zellij-utils
zellij-tile         ──┘

zellij-tile-utils   (no deps on other zellij crates)

default-plugins/*   ──→ zellij-tile  (path: ../../zellij-tile)
```

`xtask` is the build orchestrator: 10 sub-modules (build, ci, clippy, dist, flags, format, metadata, pipelines, test, main), 11 subcommands. The CI workflow (`.github/workflows/rust.yml`) calls `cargo xtask build/test/format` exclusively. Plugin builds are a single `cargo build --target wasm32-wasip1 -p <p1> -p <p2> ...` invocation so Cargo unifies features across all plugins and compiles `zellij-utils` once.

Feature flags propagate by identical-name mirroring: workspace `Cargo.toml` defines `web_server_capability = ["zellij-client/web_server_capability", "zellij-server/web_server_capability", "zellij-utils/web_server_capability"]`. Five workspace features: `plugins_from_target`, `disable_automatic_asset_installation`, `vendored_curl`, `unstable`, `web_server_capability`.

Release profile: `lto = true, strip = true, codegen-units = 1`. A `[profile.dev-opt]` inherits dev but optimizes external deps for fast iteration.

## Client/Server IPC Model

zellij runs as **one binary, two roles, N plugins** in one process. The client/server split is a logical-process boundary within a single OS process most of the time (`zellij` starts a backgrounded server and connects to it; subsequent `zellij attach` invocations connect to the same server).

**Transport:** `interprocess::local_socket::Stream` — Unix domain socket on \*nix (via `GenericFilePath`), named pipe on Windows (via `GenericNamespaced`). Helpers `ipc_bind`, `ipc_bind_async`, `ipc_connect` live in `zellij-utils/src/consts.rs:195-300`. Windows also writes a marker file containing the server PID for liveness probing.

**Framing:** Length-prefixed. 4-byte little-endian u32 → exactly that many bytes of `prost`-encoded protobuf payload. See `write_protobuf_message` / `read_protobuf_message` at `zellij-utils/src/ipc.rs:402-426`.

**Schema:** Three `.proto` files in `zellij-utils/src/client_server_contract/`: `client_to_server.proto`, `server_to_client.proto`, `common_types.proto`. Schema version constant: `CLIENT_SERVER_CONTRACT_VERSION: usize = 1` (`consts.rs:25`). Socket directory is scoped by version: `~/.cache/zellij/contract_version_1/session_info/<session_name>`.

**Message catalog:** 20 `ClientToServerMsg` variants and 13-16 `ServerToClientMsg` variants (variant count differs slightly between the Rust enum and the From-conversion impl because subscribe-only messages get folded). Full table in `zellij-pass-B-deep-ipc-r1.md`.

**`route` thread (one per attached client):** The ONLY socket reader. Translates each `ClientToServerMsg` variant into the right typed thread-bus message (`ScreenInstruction`, `PluginInstruction`, `PtyInstruction`, `ServerInstruction`). Lives at `zellij-server/src/route.rs` (3,179 LOC).

**Server thread bus:** `ThreadSenders` (`zellij-server/src/thread_bus.rs:11-23`) is a typed actor mesh:

```rust
pub struct ThreadSenders {
    pub to_screen: Option<SenderWithContext<ScreenInstruction>>,
    pub to_pty: Option<SenderWithContext<PtyInstruction>>,
    pub to_plugin: Option<SenderWithContext<PluginInstruction>>,
    pub to_server: Option<SenderWithContext<ServerInstruction>>,
    pub to_pty_writer: Option<SenderWithContext<PtyWriteInstruction>>,
    pub to_background_jobs: Option<SenderWithContext<BackgroundJob>>,
    pub should_silently_fail: bool,
}
```

Each `send_to_*` method does `.context("failed to get X sender")?.send(...).to_anyhow().context("failed to send to X")`. The `should_silently_fail` field is a test-mode escape hatch.

**`SessionConfiguration` (per-client config overlay):** `zellij-server/src/lib.rs:185-308`:

```rust
pub(crate) struct SessionConfiguration {
    runtime_config: HashMap<ClientId, Config>,  // overrides saved_config
    saved_config: Config,                       // baseline; reset target
}
```

`get_client_keybinds(&client_id)` falls back to `saved_config.keybinds` if no runtime override exists. `reconfigure_runtime_config(&client_id, stringified_kdl)` parses incoming KDL with the current client config as the diff base. `rebind_keys(...)` is a surgical alternative.

**`NotificationEnd` drop-signal pattern:** `zellij-server/src/route.rs:316-388`. A struct holding `Option<oneshot::Sender<ActionCompletionResult>>` whose `Drop` impl fires the oneshot. `Clone` strips the sender. Pattern: hand it to whichever subsystem will perform the action; the sender fires automatically when the action completes (the struct goes out of scope in the executing thread).

**`ExitReason`** (`ipc.rs:209-260`) carries user-friendly Display impls — `Disconnect` even embeds a "try `zellij attach <name>`" recovery hint.

## Plugin SDK (WASM-based, capability model, lifecycle)

zellij's plugin system is one of its most exemplary architectural achievements and the most monocle-relevant pattern.

**Plugin runtime:** `wasmi` 0.51.3 (pure-Rust WASM interpreter, NOT JIT) + `wasmi_wasi`. Plugins compile to `wasm32-wasip1`. The Engine is shared across all loaded plugins; each plugin gets its own `Module` (cached in `Arc<Mutex<HashMap<PathBuf, Module>>>`) and per-`(PluginId, ClientId)` `Store<PluginEnv>` + `Instance`.

**Host ABI surface:** EXACTLY ONE imported host function — `host_run_plugin_command()` (no args, no return) — registered via `zellij_exports(linker)` in `zellij-server/src/plugins/zellij_exports.rs:155-158`:

```rust
pub fn zellij_exports(linker: &mut Linker<PluginEnv>) {
    linker
        .func_wrap("zellij", "host_run_plugin_command", host_run_plugin_command)
        .unwrap();
}
```

**Command flow:** Plugin builds a `PluginCommand` enum variant → converts to `ProtobufPluginCommand` → `encode_to_vec()` → writes bytes to wasi stdout → calls `host_run_plugin_command()`. Host reads the bytes (`wasi_read_bytes(env)`), decodes the protobuf, dispatches via a 5,376-LOC match in `zellij_exports.rs:161-5170`.

**Plugin command catalog:** ~120 variants in `PluginCommand` (`zellij-utils/src/data.rs:3325+`). Logical clusters: subscriptions, open file/terminal/command, pane control, tabs, modes, scrolling, self-state, stdin/clipboard, config/permissions, process control, web, sessions, pipes, filesystem observe, scrollback, pane info, plugin↔plugin, layout, theme.

**Capability model:** 17 `PermissionType` variants (`data.rs:1063-1086`): `ReadApplicationState`, `ChangeApplicationState`, `OpenFiles`, `RunCommands`, `OpenTerminalsOrPlugins`, `WriteToStdin`, `WebAccess`, `ReadCliPipes`, `MessageAndLaunchOtherPlugins`, `Reconfigure`, `FullHdAccess`, `StartWebServer`, `InterceptInput`, `ReadPaneContents`, `RunActionsAsUser`, `WriteToClipboard`, `ReadSessionEnvironmentVariables`.

Every host-side dispatch first calls `check_command_permission(plugin_env, command)` (`zellij_exports.rs:5175-5380`). The function:
1. Returns `Granted` unconditionally if the plugin is built-in (`plugin_env.plugin.is_builtin()`).
2. Otherwise looks up the required permission in a giant match and checks against `PermissionCache`.

Grants are persisted to `~/.cache/zellij/permissions.kdl` (`ZELLIJ_PLUGIN_PERMISSIONS_CACHE`).

**Plugin lifecycle (`plugin_loader.rs:175-181`):**

```rust
pub fn start_plugin(&mut self) -> Result<()> {
    let module = if self.skip_cache {
        self.interpret_module()?
    } else {
        self.load_module_from_memory()
            .or_else(|_e| self.interpret_module())?
    };
    let (store, instance) = self.create_plugin_environment(module)?;
    self.load_plugin_instance(store, &instance)?;
    self.clone_instance_for_other_clients()?;
    Ok(())
}
```

Each plugin gets four WASI preopens (`zellij_exports.rs:185-201`):
- `/host` → user's CWD (project files)
- `/data` → per-plugin persistent data dir
- `/cache` → per-plugin cache dir
- `/tmp` → shared scratch dir

**Plugin trait (`zellij-tile/src/lib.rs:31-49`):**

```rust
pub trait ZellijPlugin: Default {
    fn load(&mut self, configuration: BTreeMap<String, String>) {}
    fn update(&mut self, event: Event) -> bool { false }   // true → render() will run
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool { false }
    fn render(&mut self, rows: usize, cols: usize) {}
}
```

`load` runs once at startup. `update` returns true to request a render. `pipe` receives cross-plugin messages by name+payload. `render` writes ANSI bytes to wasi stdout.

**Workers** are wasm exports named `<name>_worker`. The plugin loader scans for them, builds a fresh `Store` + `Instance` per worker, and spawns a dedicated tokio task per worker. Communication via `post_message_to(worker_name, message, payload)` (plugin → worker) and `post_message_to_plugin(message, payload)` which fires `Event::CustomMessage(message, payload)` (worker → plugin).

**Subscription model:** Each plugin holds a `Subscriptions: HashSet<EventType>` in `Arc<Mutex<>>`. Events the plugin hasn't subscribed to are filtered out before delivery. `InitialKeybinds` is a subscribe-once optimization — plugins that subscribe to it receive keybinds once at load and can cache them, so subsequent `ModeUpdate` events ship empty keybinds (lighter payload).

**UI component library** (`zellij-tile/src/ui_components/`, ~787 LOC): `Text`, `Table`, `Ribbon`, `NestedList`. These produce ANSI bytes via helper `print_*_with_coordinates` shim functions.

## Configuration and Keybinds (KDL format, layered loading, modal keymaps)

**Config aggregate** (`zellij-utils/src/input/config.rs:31-41`):

```rust
pub struct Config {
    pub keybinds: Keybinds,
    pub options: Options,
    pub themes: Themes,
    pub plugins: PluginAliases,
    pub ui: UiConfig,
    pub env: EnvironmentVariables,
    pub background_plugins: HashSet<RunPluginOrAlias>,
    pub web_client: WebClientConfig,
}
```

**Layered resolution (`Config::try_from(opts)`, `config.rs:166-200`):**
1. Explicit `--config <path>` → parse that file on top of defaults.
2. `zellij setup --clean` → defaults only.
3. `--config-dir <dir>` or `home::find_default_config_dir()` → look for `<dir>/config.kdl`, parse on top of defaults.
4. Fallback: defaults only.

`ZELLIJ_CONFIG_FILE`, `ZELLIJ_CONFIG_DIR`, `ZELLIJ_LAYOUT_DIR` env vars override the search paths.

**Config merge (`Config::merge`, `config.rs:264-271`):** Each sub-aggregate has its own merge policy. Themes use named-overwrite (`Themes::merge` at `theme.rs:60-66`). Options uses per-field `other.field.or(self.field)`. Keybinds merge is per-mode per-key.

**KDL is the canonical format** for config, layouts, themes, and the resurrection cache. `kdl` 4.5.0 with the `span` feature enabled lets parse errors carry source-offset info, which is attached to `KdlError` as a `NamedSource` for miette's caret-and-arrow rendering.

**Error handling sophistication:** `Config::from_path` (`config.rs:219-263`) catches specific `kdl::KdlErrorKind::Context("valid node terminator")` and rewrites it as a 4-line bullet list of likely causes ("Missing `;` after a node name", "Missing quotations", etc.). This is high-quality UX.

**Keybinds structure (`keybinds.rs:11`):**

```rust
pub struct Keybinds(pub HashMap<InputMode, HashMap<KeyWithModifier, Vec<Action>>>);
```

Two-level map. Each cell holds an ordered `Vec<Action>` — `bind "n" { NewPane; SwitchToMode "Normal"; }` produces a 2-element vec.

**Default action for unbound keys** (`keybinds.rs:71-99`):
- Locked or `default_input_mode` → `Action::Write { key_with_modifier, bytes: raw_bytes, ... }` (passthrough)
- RenameTab → `Action::TabNameInput { input: raw_bytes }`
- RenamePane → `Action::PaneNameInput`
- EnterSearch → `Action::SearchInput`
- Else → `Action::NoOp`

**Modal keymap** — 14 `InputMode` variants (`data.rs:1146-1196`): Normal, Locked, Resize, Pane, Tab, Scroll, EnterSearch, Search, RenameTab, RenamePane, Session, Move, Prompt, Tmux. Each mode has its own keybind table. Transitions happen via `Action::SwitchToMode(InputMode)` actions attached to keys.

**Hot-reload (`watch_config_file_changes`, `config.rs:442-510`):** Uses `notify::PollWatcher` with a 1-second poll interval (NOT native fs events, for cross-OS reliability). Watches the file directly (not parent dir, to avoid `EMFILE`). On modify/create, waits 100ms for editor write-rename to settle. On delete, polls for re-creation at 3-second intervals.

**Reconfigure flow (plugins mutate config):** `PluginCommand::Reconfigure(stringified_kdl, save_to_disk)` lands in `SessionConfiguration::reconfigure_runtime_config(&client_id, stringified_kdl)`, which parses incoming KDL on top of the current per-client config. `PluginCommand::RebindKeys` is the surgical variant for `keys_to_rebind: Vec<(InputMode, KeyWithModifier, Vec<Action>)>` + `keys_to_unbind: Vec<(InputMode, KeyWithModifier)>`.

## Theming

Theme model is **semantic-token-based, not color-name-based** (`zellij-utils/src/data.rs:1397-1414`):

```rust
pub struct Styling {
    pub text_unselected: StyleDeclaration,
    pub text_selected: StyleDeclaration,
    pub ribbon_unselected: StyleDeclaration,
    pub ribbon_selected: StyleDeclaration,
    pub table_title: StyleDeclaration,
    pub table_cell_unselected: StyleDeclaration,
    pub table_cell_selected: StyleDeclaration,
    pub list_unselected: StyleDeclaration,
    pub list_selected: StyleDeclaration,
    pub frame_unselected: Option<StyleDeclaration>,
    pub frame_selected: StyleDeclaration,
    pub frame_highlight: StyleDeclaration,
    pub exit_code_success: StyleDeclaration,
    pub exit_code_error: StyleDeclaration,
    pub multiplayer_user_colors: MultiplayerColors,
}

pub struct StyleDeclaration {
    pub base: PaletteColor,
    pub background: PaletteColor,
    pub emphasis_0: PaletteColor,
    pub emphasis_1: PaletteColor,
    pub emphasis_2: PaletteColor,
    pub emphasis_3: PaletteColor,
}
```

14 token groups (+ multiplayer-colors) × 6 slots = **84 colors per theme**.

Two acceptable KDL formats:
- **Old palette:** `fg 60 56 54` or `fg "#D5C4A1"` — 16-color ANSI vocabulary
- **New semantic:** `text_unselected { base 251 241 199; background 60 56 54; emphasis_0 214 93 14; ... }`

41 built-in themes embedded via `include_dir!` (`consts.rs:23`).

**Color representation** (`data.rs:1211-1223`):

```rust
pub enum PaletteColor {
    Rgb((u8, u8, u8)),
    EightBit(u8),
}
```

24-bit RGB or 8-bit indexed.

**Runtime theme switching:** Three actions — `SetDarkTheme`, `SetLightTheme`, `ToggleTheme` — plus the implicit reaction to `Event::HostTerminalThemeChanged(HostTerminalThemeMode)` (Dark or Light).

**Host terminal theme detection:** Client sends `CSI ? 2031 h` to subscribe to host theme changes; sends `CSI ? 996 n` to actively query. Replies arrive as `CSI ? 997 ; {1|2} n` (1=dark, 2=light). Client emits `ClientToServerMsg::HostTerminalThemeChanged { mode }` upward.

## Session Persistence and Resume

This is THE pattern monocle's session-bookmark feature should mirror.

**Filesystem layout** (`zellij-utils/src/consts.rs:89-110`):

```
~/.cache/zellij/
├── permissions.kdl                       # plugin grants
├── 0.45.0/...                            # per-version artifacts
└── contract_version_1/
    └── session_info/
        ├── <session_name>/
        │   ├── session-metadata.kdl      # current state
        │   ├── session-layout.kdl        # resurrection artifact
        │   └── <pane-content-files>      # optional scrollback
        └── ...
```

Each session is a directory. The directory's existence + presence of `session-layout.kdl` defines "resurrectable".

**Save chain** (5 threads deep):

1. **Screen thread** holds layout state, knows when something material changed.
2. **Plugin thread** enriches metadata with plugin list + available layouts.
3. **Pty thread** populates per-pane CWD + resolved-command-path. Calls `metadata.is_dirty()` — skip save if not dirty.
4. **BackgroundJobs thread** receives `BackgroundJob::ReportLayoutInfo`; offloads disk I/O so the render loop isn't blocked.
5. **Filesystem write** via `write_session_state_to_disk` (`background_jobs.rs:670-700`). Uses `file_content_changed` byte-comparison to skip redundant writes.

**Dirty detection** (`SessionLayoutMetadata::is_dirty`, `session_layout_metadata.rs:107-150`): A layout is dirty (worth saving) if (a) pane count differs from base layout, OR (b) any terminal pane runs a non-default-shell command, OR (c) any pane is in EditFile state. Modal/utility plugin panes (`zellij:session-manager`, `zellij:about`, `zellij:plugin-manager`, `zellij:configuration-manager`, `zellij:share`) are excluded from pane count.

**Two save flow variants:**
- `PtyInstruction::LogLayoutToHd(metadata)` — periodic / on-detach. Goes through `is_dirty()`.
- `PtyInstruction::SaveSessionToDisk { name, info, metadata, completion_tx }` — explicit save (plugin command `SaveSession`). No dirty gate. Updates `UpdateSessionSaveTime(timestamp)` for plugins to query.

**Resurrection enumeration** (`sessions::get_resurrectable_sessions`, `sessions.rs:46-92`): Walks `ZELLIJ_SESSION_INFO_CACHE_DIR` directly. No index file. Returns `Vec<(session_name, elapsed_since_layout_save)>` where age uses ctime (fallback to mtime).

**Live-vs-dead discrimination:** `get_sessions()` reads `ZELLIJ_SOCK_DIR` and probes each socket with `ConnStatus`. `assert_socket` cleans up stale sockets (`sessions.rs:139-200`). A session shows up in resurrectable list iff its `session-layout.kdl` exists and no live socket points to it.

**Replay:** `zellij attach <name>` parses `session-layout.kdl` as a `Layout` and replays it: tabs and panes are recreated; `Run::Command` panes re-spawn their commands; `Run::Plugin` panes reload their wasm. Scrollback is NOT replayed by default; plugin runtime state is NOT preserved (plugins start fresh from `load()` but their `/data` dir survives).

## Default Plugins Survey (one representative pass)

13 plugins ship with zellij. Four archetypes:

1. **Mode-aware decoration** — tab-bar (796 LOC), status-bar (5,945 LOC), compact-bar (2,420 LOC). `set_selectable(false)` on load; subscribe to `ModeUpdate`, `TabUpdate`, `Mouse`, `InitialKeybinds`. Render produces ANSI bytes to wasi stdout.
2. **Filesystem-permission consumer** — strider (1,338 LOC). Subscribes to `FileSystemUpdate`, `HostFolderChanged`, `PermissionRequestResult`. Calls `scan_host_folder("/host")`, `change_host_folder(path)`. The `/host` mount is the user's CWD.
3. **Stateful interactive UI** — session-manager (5,955 LOC), plugin-manager (1,103 LOC), about (2,477 LOC), share (2,488 LOC), multiple-select (695 LOC), layout-manager (4,344 LOC), link (474 LOC). Module-per-screen pattern; large state struct with sub-state types; `set_timeout` + `Event::Timer` for polling.
4. **Reconfigure UI** — configuration (3,801 LOC). Edits running config via `PluginCommand::Reconfigure` (full file) or `PluginCommand::RebindKeys` (surgical).

**Key SDK pattern: `InitialKeybinds` cache:** Plugins that need keybinds subscribe to `EventType::InitialKeybinds` to receive them once, then process subsequent `ModeUpdate` events that ship empty keybinds. The plugin folds cached keybinds into the incoming `ModeUpdate` before processing.

**Cross-plugin pipes:** `pipe()` method receives named messages from other plugins. session-manager uses this to receive filepicker results from strider.

## Conventions and Patterns Worth Adopting

| Pattern | Source |
|---|---|
| `anyhow::Result` everywhere; `prelude` re-exports `anyhow::*` + `LoggableError` + `FatalError` + `ZellijError` | `zellij-utils/src/errors.rs:21-32` |
| `.context("X")?` chained error context | `thread_bus.rs:27-30` |
| Thread-local `OPENCALLS: RefCell<ErrorContext>` + `tokio::task_local! ASYNCOPENCALLS` | `channels.rs:36-43` |
| `SenderWithContext<T>` carrying ErrorContext on every send | `channels.rs:23-32` |
| Typed actor mesh via `ThreadSenders` | `thread_bus.rs:11-23` |
| `should_silently_fail` bool field as test escape hatch | `thread_bus.rs:22` |
| Length-prefixed protobuf wire format | `ipc.rs:402-426` |
| `CLIENT_SERVER_CONTRACT_VERSION` constant scoping socket path | `consts.rs:25, 106-110` |
| `From<ServerToClientMsg> for ClientInstruction` for protocol-to-internal-event translation | `zellij-client/src/lib.rs:196-220` |
| Drop-signaling `NotificationEnd { channel: Option<oneshot::Sender>, ... }` | `route.rs:316-388` |
| Per-client config overlay (`runtime_config` over `saved_config`) | `zellij-server/src/lib.rs:185-308` |
| Newtype wrappers around HashMap aggregates (`Keybinds`, `Themes`, `PluginAliases`) | `keybinds.rs:11`, `theme.rs:38-44` |
| `#[path = "x_unix.rs"]` / `#[path = "x_windows.rs"]` for per-OS file split | `zellij-server/src/lib.rs:1-5` |
| `directories::ProjectDirs` for cache/config paths | `consts.rs:89-99` |
| `include_dir!` and `include_bytes!` for embedded assets | `consts.rs:23`, `setup.rs:43-79` |
| `disable_automatic_asset_installation` feature for read-only-fs deployments | `setup.rs` |
| `KdlError { offset, len, src: NamedSource }` with miette for source-pointer error rendering | `config.rs:43-78` |
| `PollWatcher` (1-second interval) for config hot-reload with 100ms debounce | `config.rs:442-510` |
| Single host-import function (`host_run_plugin_command`) + protobuf-over-stdout plugin ABI | `zellij_exports.rs:155-180` |
| Four WASI virtual mounts for plugins (`/host`, `/data`, `/cache`, `/tmp`) | `zellij_exports.rs:185-201` |
| Capability-token permission gate, persisted to disk, one-time prompt | `data.rs:1063-1086`, `input/permission.rs` |
| Built-in plugins bypass permission gate (special-cased) | `zellij_exports.rs:5179-5184` |
| `*_worker`-suffix wasm exports for background tasks; dedicated tokio task per worker | `plugin_loader.rs:198-216` |
| Plugin actions go through the same `route_action` as user keystrokes (symmetric dispatch) | `zellij_exports.rs:140-153` |
| Two-file session persistence: metadata.kdl + layout.kdl per session, in a per-session directory | `consts.rs:27-37`, `background_jobs.rs:670-700` |
| `is_dirty()` + `file_content_changed` gates before writing | `session_layout_metadata.rs:107-150`, `background_jobs.rs:660-685` |
| 5-thread save chain offloads disk I/O from render path | full trace in `pass-B-deep-session-persistence-r1.md` |
| Modal keymap (`Keybinds(HashMap<InputMode, HashMap<KeyWithModifier, Vec<Action>>>)`) + default-action-per-mode | `keybinds.rs:11, 71-99` |
| Two-level action sequence per key (e.g. `bind "n" { NewPane; SwitchToMode "Normal"; }`) | `example/default.kdl` |
| Semantic-token theme model (15 groups × 6 slots = 84 colors); two KDL formats accepted | `data.rs:1397-1423` |
| Host terminal theme detection via CSI 2031 + DSR 996 | `zellij-client/src/lib.rs:52-67` |

## Risk Register

| ID | Severity | Finding | Impact for Monocle |
|---|---|---|---|
| P1-1 | important | 47-variant `PluginInstruction` enum with positional unnamed fields (8+ Option/bool tuples) | Hard to evolve and review. Monocle should prefer named-field struct variants from day one. |
| P1-2 | important | 9,958-line `screen.rs` | Heroic single-file modules are an anti-pattern. Monocle should split early. |
| P1-3 | important | Unbounded crossbeam channels everywhere | Risk of unbounded memory growth under bursty inputs. Monocle should consider bounded channels with try_send / drop-policy. |
| P1-4 | important | wasmi (interpreter) is slower than wasmtime (JIT) | CPU-bound factory adapters will pay this cost. Consider wasmtime if monocle's factory-adapters need throughput. |
| P1-5 | suggestion | Duplicate `ClientId` alias (`zellij-server/src/lib.rs:79` AND `zellij-utils/src/data.rs:31`) with `// TODO: merge` comment | Don't duplicate type aliases across crates. |
| P1-6 | suggestion | `should_silently_fail` test field bleeds into production type | Use wrapper or trait instead of a struct field. |
| P1-7 | suggestion | Stringly-typed `PluginCommand::Reconfigure(String, bool)` for whole-config replace | Prefer typed config end-to-end if possible. |
| P1-8 | suggestion | `serde_yaml = "0.8"` (deprecated for years) still in `zellij-client/Cargo.toml` | Don't use deprecated deps. |
| P1-9 | suggestion | `wasmi_wasi::sync::WasiCtxBuilder` despite tokio async server thread (potential blocking) | Audit for blocking calls inside async contexts. |
| P0-1 | critical | None identified in the in-scope code paths. | — |

No P0 findings: zellij's in-scope architecture is solid. P1s are stylistic / evolutionary risks, not correctness/security defects.

## Test Coverage Notes

- Test framework: `insta` snapshots (10 snapshot dirs), `expect-test` for inline expectations, `serial_test` for non-concurrent tests, `tokio::test-util` for virtual time.
- 25 `*_test*.rs` files + 10 `unit/` subdirectories.
- `cargo xtask test` is the canonical test entry; CI runs it across ubuntu-latest, macos-latest, windows-latest, plus a `test-no-web` job.
- Test fixtures live as workspace crates (`default-plugins/fixture-plugin-for-tests`, 1,005 LOC) — a real wasm plugin used purely for testing the plugin host.
- e2e tests in `src/tests/e2e/` use `ssh2`-based remote-controlled sessions (out of scope per user, but mentioned for completeness).
- Snapshot tests cover KDL parsing extensively (the `zellij-utils/src/snapshots/` and `zellij-utils/src/kdl/snapshots/` directories).

## Architecture Recommendations for Monocle

### Recommended crate split

Drawing from zellij's pattern:

```
monocle-core               (analog: zellij-utils — shared boundary types)
├── types/                 (SessionId, MissionId, RunnerKind, FactoryKind, ...)
├── config/                (JSON-or-KDL profile parsing)
├── ipc/                   (wire envelopes + serialization — if monocle becomes a daemon)
├── errors/                (custom error trait + LoggableError-style chains)
├── channels/              (cross-thread SenderWithContext wrapper)
└── consts/                (path constants, env vars, CONTRACT_VERSION)

monocle-runtime            (analog: zellij-server — stateful orchestrator)
├── session/               (session lifecycle + bookmark resume)
├── runner/                (tmux integration; pane/window state)
├── factory_host/          (if monocle ships a plugin model — analog: server/src/plugins/)
└── bus/                   (typed thread bus per zellij-server/src/thread_bus.rs)

monocle-tui                (analog: zellij-client — owns the user's terminal)
├── input/                 (key dispatch — single-mode, NOT modal)
├── render/                (ratatui-based render loop)
└── ipc/                   (client side of IPC if split client/server)

monocle-plugin-sdk         (analog: zellij-tile — only if factory-adapter WASM model adopted)
├── trait/                 (FactoryAdapter trait)
├── shim/                  (host-call wrappers)
└── ui/                    (helper UI components)

monocle (binary)           (analog: zellij — CLI dispatch + role selection)
└── main.rs
```

### IPC model summary (Rust-port-ready)

- **Transport:** Unix domain socket on \*nix, named pipe on Windows. Use the `interprocess` crate as zellij does.
- **Framing:** 4-byte little-endian u32 length prefix + protobuf payload. (`prost`)
- **Schema:** Three `.proto` files (`client_to_server.proto`, `server_to_client.proto`, `common_types.proto`) under `monocle-core/src/ipc/`.
- **Versioning:** `MONOCLE_CONTRACT_VERSION: usize = 1` constant; socket path scoped by `contract_version_1/`.
- **Routing:** One `route` thread per attached client; the ONLY socket reader for that client.
- **Internal mailbox bus:** `ThreadSenders` actor mesh, typed per worker thread; `SenderWithContext<T>` carrying `ErrorContext` on every send.
- **Per-client config:** `SessionConfiguration { runtime_config: HashMap<ClientId, Config>, saved_config: Config }` overlay model.
- **Action completion:** Drop-signaling `NotificationEnd { channel: Option<oneshot::Sender<MissionResult>>, ... }` for "block until done" semantics.
- **Exit reasons:** Enum with user-friendly Display impls; embed recovery hints in the message.

### Plugin SDK summary — should monocle adopt the WASM model for factory adapters?

**Yes for factory adapters specifically.**

Reasoning:
1. Factory adapters are exactly the "third-party-extensibility-with-capability-isolation" case that zellij's WASM plugin model excels at.
2. The 4-mount WASI path translation (`/host`, `/data`, `/cache`, `/tmp`) maps naturally to factory operations:
   - `/host` → user's project directory
   - `/data` → adapter's persistent storage (per-(adapter, session))
   - `/cache` → adapter's cache
   - `/tmp` → scratch
3. The 17-permission capability token model maps to factory operations: `OpenFiles`, `RunCommands`, `WebAccess`, `ReadSessionEnvironmentVariables`, etc.
4. Single-import-function ABI (`host_run_plugin_command`) is the right level of host coupling.
5. Built-in adapters can bypass the permission gate; third-party adapters get prompts.
6. Workers (`*_worker` exports) handle long-running adapter operations without blocking the main loop.

**No for the user input dispatch.** Monocle has one terminal owned by tmux, not multiple clients with modal keybinds. Don't import the InputMode / Keybinds / modal-state-machine machinery.

**Wasmtime vs wasmi choice:** zellij uses wasmi (interpreter) for portability and small binary size. Monocle should consider wasmtime (JIT) if factory adapters need CPU throughput. The protobuf ABI surface is the same either way.

### Session bookmark = zellij session resurrection (almost verbatim)

```
~/.cache/monocle/contract_version_1/session_info/<bookmark_name>/
├── session-metadata.kdl     (or .json)
├── session-layout.kdl       (or .json)  -- resurrection artifact
└── mission-runs/<run_id>.kdl  -- optional per-run history
```

- Save triggers: detach event + periodic 30-second dump.
- Dirty gate: pane count changed? mission state changed? if not, skip.
- `file_content_changed` byte-compare before writing.
- Background thread for FS I/O (NEVER on the render path).
- Resume: directory traversal + name match → parse layout → replay state.
- Last-save timestamp tracked separately for UI display.

### Configuration model

KDL is genuinely the right choice for human-edited config — better than YAML/TOML for nested structure, span-aware for error reporting. If monocle has settled on JSON (per the brief), it should at least:
- Layered priority (CLI flag > env > config dir > defaults).
- Per-sub-aggregate merge semantics.
- Hot-reload via PollWatcher (1-second interval, watch file directly not parent dir).
- 100ms debounce after modify/create to accommodate editor write-rename patterns.
- Source-span-aware error reporting via miette.

## Convergence Statement

All 7 in-scope categories (workspace, ipc, plugin-sdk, configuration-and-keybinds, session-persistence, theming, default-plugins-survey) converged in 2 deepening rounds — the minimum required by the Iron Law. Round 1 captured every architectural layer; Round 2 found only implementation-detail refinements (transport plumbing, wasi marshaling helpers, error-message rewriting, default-shell mutation patterns) that don't change the model.

The single citation error (workspace member count: 23 → 20) was caught by Pass B.6 extraction validation and corrected here in Pass 8.

No reserved adversarial-template section headers were used in this synthesis.

## Handoff

### For `create-brief`

The brief should capture:
- monocle is **Rust-native**; zellij is the model peer for crate split, IPC, plugin model, config, theming, session persistence.
- The "single binary, two roles, N plugins" pattern is the canonical shape (if monocle goes daemon).
- Recommended crate split as above.
- WASM plugin model is recommended for factory-adapters but NOT for input handling.
- KDL is the recommended config format; if monocle keeps JSON, mirror the layered-merge + hot-reload + source-span-error semantics.
- The session-bookmark feature is a direct port of zellij's session resurrection — directory-per-session, two-file persistence, is_dirty + file_content_changed gates, dedicated background-jobs thread for FS I/O.

### For `disposition-pass`

P1 risks to disposition:
- 47-variant enum with positional fields → "avoid"
- 9k-line modules → "avoid"
- Unbounded channels by default → "bounded with drop-policy"
- Stringly-typed Reconfigure → "typed config end-to-end"
- Duplicate type aliases → "single source of truth"
- Mixing async tokio with sync wasi → "audit; either tokio::task::spawn_blocking or use async wasi"

### For `create-prd`

Specific monocle features to design with zellij as the model:
1. **Session bookmark / resume** — verbatim adopt session-persistence model.
2. **Factory-adapter SDK (if needed)** — verbatim adopt plugin SDK (WASM + single host import + protobuf + 17-permission gate + 4 WASI mounts).
3. **IPC daemon split (if needed)** — verbatim adopt the length-prefixed-protobuf + route-thread + ThreadSenders pattern.
4. **Config hot-reload** — PollWatcher + 100ms debounce.
5. **Theme system** — semantic 84-color token model (or simplified subset).
6. **Per-client config overlay** — `SessionConfiguration` model (if monocle supports concurrent clients).

## Files in this Scoped Ingest

All absolute paths under `/Users/jmagady/Dev/monocle/.factory/semport/zellij/`:

| File | Purpose |
|---|---|
| `zellij-pass-1-project-discovery.md` | Phase A pass 1 — inventory + scope statement |
| `zellij-pass-2-architecture.md` | Phase A pass 2 — components, layers, thread bus, build/release |
| `zellij-pass-3-domain-model.md` | Phase A pass 3 — entity catalog, glossary, state machines, bounded contexts |
| `zellij-pass-4-behavioral-contracts.md` | Phase A pass 4 — 16 draft behavioral contracts (BC-001 to BC-016) |
| `zellij-pass-5-nfr-catalog.md` | Phase A pass 5 — performance, security, observability, reliability, scalability, cross-platform |
| `zellij-pass-6-conventions.md` | Phase A pass 6 — naming, module organization, error handling, test patterns, design patterns, anti-patterns |
| `zellij-pass-7-holdout-seeds.md` | Phase A pass 7 — list of in-scope and out-of-scope subsystems, known unknowns |
| `zellij-pass-B-deep-workspace-r1.md` | Phase B workspace round 1 — full crate catalog + xtask deep dive |
| `zellij-pass-B-deep-workspace-r2.md` | Phase B workspace round 2 — nitpick refinements |
| `zellij-pass-B-deep-ipc-r1.md` | Phase B IPC round 1 — full message catalog + lifecycle diagrams |
| `zellij-pass-B-deep-ipc-r2.md` | Phase B IPC round 2 — transport plumbing nitpicks |
| `zellij-pass-B-deep-plugin-sdk-r1.md` | Phase B plugin SDK round 1 — host ABI, permission gate, lifecycle, worker model |
| `zellij-pass-B-deep-plugin-sdk-r2.md` | Phase B plugin SDK round 2 — wasi helper nitpicks |
| `zellij-pass-B-deep-config-keybinds-r1.md` | Phase B config & keybinds round 1 — layered loading, merge semantics, modal keymap, hot-reload |
| `zellij-pass-B-deep-config-keybinds-r2.md` | Phase B config & keybinds round 2 — error-handling nitpicks |
| `zellij-pass-B-deep-session-persistence-r1.md` | Phase B session persistence round 1 — 5-thread save chain, dirty detection, resurrection enumeration |
| `zellij-pass-B-deep-session-persistence-r2.md` | Phase B session persistence round 2 — save flow refinements |
| `zellij-pass-B-deep-theming-r1.md` | Phase B theming round 1 — semantic token model, KDL formats, host terminal detection |
| `zellij-pass-B-deep-theming-r2.md` | Phase B theming round 2 — auxiliary type nitpicks |
| `zellij-pass-B-deep-default-plugins-r1.md` | Phase B default plugins round 1 — 4-archetype classification |
| `zellij-pass-B-deep-default-plugins-r2.md` | Phase B default plugins round 2 — helper-struct nitpicks |
| `zellij-pass-B5-coverage-audit.md` | Phase B.5 — scope compliance audit; all 10 in-scope items 100% covered, no out-of-scope leakage |
| `zellij-pass-B6-extraction-validation.md` | Phase B.6 — 11 spot checks; 10 confirmed, 1 corrected (workspace member count) |
| `zellij-pass-8-final-synthesis.md` | This file |

```yaml
final_status:
  pass_8: complete
  timestamp: 2026-05-11T21:30:00Z
  iron_law_satisfied: true
  scope_compliance: pass
  citation_quality_estimate: 10/11 = 91% verified-clean (one corrected count error)
```
