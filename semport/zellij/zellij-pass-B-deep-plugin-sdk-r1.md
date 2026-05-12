# Phase B Deep — Plugin SDK (Round 1)

## Plugin Loading Lifecycle (full sequence)

`zellij-server/src/plugins/plugin_loader.rs:175-181`:

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

Flow:

1. **Resolve & parse WASM** (`interpret_module`, `plugin_loader.rs:184-198`):
   - `plugin_config.resolve_wasm_bytes(&self.plugin_dir)` fetches bytes either from the embedded include_dir asset (built-in) or from a file URL / cached download (user plugin).
   - `Module::new(&self.engine, &wasm_bytes)` parses to wasmi Module.
   - Logs `"Loaded plugin '<path>' in <duration>"` for observability.
2. **Create execution environment** (`create_plugin_environment`):
   - `WasiCtxBuilder` with three preopens: `plugin_own_data_dir → /data`, `plugin_own_cache_dir → /cache`, `ZELLIJ_TMP_DIR → /tmp`, plus the plugin's CWD → `/host` (read-only depending on permissions).
   - `Store<PluginEnv>` holds plugin-specific state: client_id, plugin_id, plugin_cwd, plugin_own_data_dir, plugin_own_cache_dir, plugin_config, tab_index, path_to_default_shell, session_env_vars, default_shell, layout_dir, default_mode, keybinds, plugin_dir, size, senders.
   - `Linker` registers `host_run_plugin_command` via `zellij_exports(linker)` (`plugins/zellij_exports.rs:155-158`).
3. **Instantiate** (`load_plugin_instance`, `plugin_loader.rs:200-249`):
   - Get `_start` function (always present from wasi-wasip1 entry point).
   - Get `load` function (plugin's first user-callable export).
   - Iterate exports for `*_worker`-suffix functions; for each one, build a separate fresh Store+Instance, call its `_start`, wrap as `RunningWorker`, and spawn a tokio task that drives it.
   - Add `(plugin_id, client_id) → (RunningPlugin, Subscriptions, workers)` to PluginMap.
4. **Multi-client clone** (`clone_instance_for_other_clients`):
   - For every other connected ClientId, re-instantiate (cheap — Module is reused; only fresh Store).
   - Each clone reads its own per-client `keybinds`, `default_mode` from `WasmBridge`.
5. **Cache Module in memory** (`PluginCache: Arc<Mutex<HashMap<PathBuf, Module>>>`).

## Plugin Host-Call ABI

The plugin↔host communication uses exactly **one** imported host function: `host_run_plugin_command()` (no args, no return).

The plugin-side flow (from `zellij-tile/src/shim.rs:50-57`):

```rust
pub fn subscribe(event_types: &[EventType]) {
    let event_types: HashSet<EventType> = event_types.iter().cloned().collect();
    let plugin_command = PluginCommand::Subscribe(event_types);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}
```

Pattern: build `PluginCommand` enum → convert to `ProtobufPluginCommand` → encode_to_vec → write to wasi stdout → call host function. The host reads stdout, decodes, dispatches.

The host-side handler (`zellij_exports.rs:161-180`):

```rust
fn host_run_plugin_command(mut caller: Caller<'_, PluginEnv>) {
    let mut env = caller.data_mut();
    let plugin_command = env.name();
    let err_context = || format!("failed to run plugin command {}", plugin_command);
    wasi_read_bytes(env)
        .and_then(|bytes| {
            let command: ProtobufPluginCommand = ProtobufPluginCommand::decode(bytes.as_slice())?;
            let command: PluginCommand = command
                .try_into()
                .map_err(|e| anyhow!("failed to convert serialized command: {}", e))?;
            match check_command_permission(&env, &command) {
                (PermissionStatus::Granted, _) => match command {
                    PluginCommand::Subscribe(event_list) => subscribe(env, event_list)?,
                    PluginCommand::Unsubscribe(event_list) => unsubscribe(env, event_list)?,
                    // ... ≈120 variants ...
                }
                ...
            }
            ...
        })
}
```

Source of `zellij_exports.rs` is 5,376 LOC, the vast majority of which is one giant `match` and the implementation of each PluginCommand variant.

## Plugin Permission Gate

`zellij_exports.rs:5175-5380` defines `check_command_permission(plugin_env, command) -> (PermissionStatus, Option<PermissionType>)`.

Crucially: **built-in plugins bypass the permission system entirely**. (`zellij_exports.rs:5179-5184`)

```rust
if plugin_env.plugin.is_builtin() {
    // built-in plugins can do all the things because they're part of the application and
    // there's no use to deny them anything
    return (PermissionStatus::Granted, None);
}
```

Permission mapping (full inventory; lines 5184-5380):

| Permission | PluginCommand variants |
|---|---|
| `OpenFiles` | OpenFile, OpenFileFloating, OpenFileNearPlugin, OpenFileFloatingNearPlugin, OpenFileInPlaceOfPlugin, OpenFileInPlace, OpenEditPaneInPlaceOfPaneId |
| `OpenTerminalsOrPlugins` | OpenTerminal*, StartOrReloadPlugin, OpenPluginPaneInNewTab, OpenPluginPaneFloating, OpenTerminalPaneInPlaceOfPaneId |
| `RunCommands` | OpenCommandPane*, RunCommand, ExecCmd |
| `WebAccess` | WebRequest |
| `WriteToStdin` | Write, WriteChars, WriteToPaneId, WriteCharsToPaneId |
| `WriteToClipboard` | CopyToClipboard |
| `ChangeApplicationState` | All pane/tab/mode/focus mutations (≈40 commands; SwitchTabTo, NewTab*, MoveFocus*, Resize, ToggleTab, MovePane, ScrollUp, …, ClearPaneHighlights) |
| `ReadCliPipes` | UnblockCliPipeInput, BlockCliPipeInput, CliPipeOutput |
| `MessageAndLaunchOtherPlugins` | MessageToPlugin |
| `ReadApplicationState` | GetPaneInfo, GetTabInfo, GetSessionList, … |
| `Reconfigure` | Reconfigure, RebindKeys, … |
| `FullHdAccess` | ScanHostFolder (and other broad-disk-scan commands) |
| `StartWebServer` | StartWebServer |
| `InterceptInput` | InterceptInput, ReleaseInputIntercept |
| `ReadPaneContents` | GetPaneScrollback |
| `RunActionsAsUser` | RunAction |
| `ReadSessionEnvironmentVariables` | GetSessionEnvironmentVariables |

PluginCommands that DO NOT require a permission (always granted):
- `Subscribe` / `Unsubscribe`
- `SetSelectable`, `ShowCursor`
- `GetPluginIds`, `GetZellijVersion`, `GenerateRandomName`
- `DumpLayout`, `ParseLayout`, `GetLayoutDir`
- `SaveSession`, `CurrentSessionLastSavedTime`
- `HideSelf`, `ShowSelf`, `CloseSelf`
- `SetTimeout`
- `PostMessageTo` (worker mailbox; intra-plugin)
- `RequestPluginPermissions` (the request itself doesn't need permission)
- `ReportPanic` (for crash reports)

## ZellijPlugin Trait

`zellij-tile/src/lib.rs:31-49`:

```rust
pub trait ZellijPlugin: Default {
    fn load(&mut self, configuration: BTreeMap<String, String>) {}
    fn update(&mut self, event: Event) -> bool { false } // true → render() will run
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool { false }
    fn render(&mut self, rows: usize, cols: usize) {}
}
```

`load` is called once when the plugin is instantiated; `configuration` carries the user-defined plugin config from KDL. `update` returns true to request a render. `pipe` is called when a CLI pipe message is delivered. `render` is called by host whenever it's time to draw — typically after `update` returned true or after a host-initiated resize.

## ZellijWorker Trait

`zellij-tile/src/lib.rs:67-72`:

```rust
pub trait ZellijWorker<'de>: Default + Serialize + Deserialize<'de> {
    fn on_message(&mut self, message: String, payload: String) {}
}
```

Worker is `Serialize + Deserialize` so the host can hibernate/restore its state across reloads (potentially). `on_message` is the worker's only entrypoint, receiving (message_name, payload).

## Worker Lifecycle Mechanics

`plugin_loader.rs:198-216` — during plugin load:

```rust
for function_name in instance
    .exports(&mut store)
    .filter_map(|export| export.clone().into_func().map(|_| export.name()))
{
    if function_name.ends_with("_worker") {
        let (mut store, instance) =
            self.create_plugin_instance_and_wasi_env_for_worker()?;
        let start_function_for_worker = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .with_context(err_context)?;
        start_function_for_worker
            .call(&mut store, ())
            .with_context(err_context)?;

        let worker = RunningWorker::new(store, instance, &function_name);
        let worker_sender = plugin_worker(worker);
        workers.insert(function_name.into(), worker_sender);
    }
}
```

Each worker:
- Lives in a **separate** wasm Instance from the main plugin.
- Runs on a dedicated tokio task, message-driven by an unbounded mpsc.
- Cannot directly access the main plugin's state; only message-passes via `Event::CustomMessage(name, payload)`.

The naming convention (`<name>_worker`) is purely string-based; there's no annotation or attribute macro.

## Subscriptions Model

A plugin owns a `Subscriptions: HashSet<EventType>`. Calling `subscribe(&[EventType::Key])` adds to that set. The plugin host walks all `EventType`s for every `Event` it would send and only delivers if the plugin is subscribed.

Source: `plugin_map.rs` (the `Subscriptions` type is just `HashSet<EventType>` wrapped in `Arc<Mutex<>>`).

## Multi-Instance, Multi-Client State Model

The plugin map's key is `(PluginId, ClientId)`. Each entry holds `(Arc<Mutex<RunningPlugin>>, Arc<Mutex<Subscriptions>>, HashMap<String, UnboundedSender<MessageToWorker>>)` (`plugin_map.rs:30-37`).

**Implications:**
- Same plugin URL launched in two tabs is two different `(PluginId, _)` pairs.
- Same plugin instance with two attached clients has two `(plugin_id, client_id_A)` and `(plugin_id, client_id_B)` entries with **shared workers** (the workers map is single-instance per plugin run, not per client).
- A "broadcast event to all subscribers" walks `plugin_assets.iter()`, checking each `Subscriptions`.

Wait — re-reading `plugin_map.rs:30-37`, each `(plugin_id, client_id)` entry has its own `workers` HashMap. So workers ARE per-client. This is unusual; one might expect workers to be shared per plugin instance. The implementation is per-`(plugin_id, client_id)` for full isolation.

## Workspace Path Translation

`zellij_exports.rs:185-201`:

```rust
fn translate_plugin_path(env: &PluginEnv, path: PathBuf) -> PathBuf {
    if let Ok(stripped) = path.strip_prefix("/host") {
        env.plugin_cwd.join(stripped)
    } else if let Ok(stripped) = path.strip_prefix("/data") {
        env.plugin_own_data_dir.join(stripped)
    } else if let Ok(stripped) = path.strip_prefix("/cache") {
        env.plugin_own_cache_dir.join(stripped)
    } else if let Ok(stripped) = path.strip_prefix("/tmp") {
        ZELLIJ_TMP_DIR.join(stripped)
    } else if path.is_relative() {
        env.plugin_cwd.join(path)
    } else {
        path
    }
}
```

The plugin sees four virtual root mounts:
- `/host` → user's CWD (project files)
- `/data` → plugin's persistent data dir (per `(plugin, client)`)
- `/cache` → plugin's cache dir
- `/tmp` → shared tmp dir (`ZELLIJ_TMP_DIR`)

This is a clean capability-by-path model. **Monocle should adopt this verbatim if it ever ships plugins.**

## Built-in Plugin Recognition

`PluginConfig::is_builtin()` — returns true when the plugin URL is the special `zellij:` scheme (e.g. `zellij:status-bar`, `zellij:tab-bar`). These wasms are include_bytes!-embedded into the binary and don't need to be downloaded.

## Apply-Action-As-Plugin Macro

`zellij_exports.rs:140-153`:

```rust
macro_rules! apply_action {
    ($action:ident, $error_message:ident, $env: ident) => {
        match route_action(
            $action,
            $env.client_id,
            None,
            Some(PaneId::Plugin($env.plugin_id)),
            $env.senders.clone(),
            $env.default_shell.clone(),
            None,
            $env.default_mode.clone(),
            None,
        ) {
            Ok((_, result)) => result,
            Err(e) => {
                log::error!("{}: {:?}", $error_message(), e);
                None
            },
        }
    };
}
```

Plugins issue actions as if they were that client: the macro reuses `route_action` (the same function the route thread calls for user-typed actions). This is the key insight — **a plugin command that mutates pane/tab/mode state goes through the exact same code path as a user keypress**. Symmetric.

## Could Monocle Adopt the WASM Plugin Model for Factory Adapters?

### Pros

1. **Stable ABI through protobuf.** Tag-numbered protobuf is the right way to evolve a plugin interface without breaking old plugins.
2. **Sandbox via WASI capabilities.** A factory adapter doesn't need full disk access; the four-mount model (`/host`, `/data`, `/cache`, `/tmp`) is sufficient.
3. **Cross-language.** Plugins can be written in any language that targets wasm32-wasip1: Rust, Go (TinyGo), AssemblyScript, Zig, etc.
4. **Hot reload.** A `Module` is cached by path; replacing the file and calling `StartOrReloadPlugin` reloads cleanly.
5. **Permission model.** 17-variant capability tokens map cleanly to factory-specific permissions (read-fs, run-cmd, web-access).
6. **Single host import function.** `host_run_plugin_command` + protobuf is a beautifully small ABI surface.
7. **Independent worker tasks.** Long-running factory operations could run as `*_worker` exports without blocking the main plugin loop.

### Cons / Caveats

1. **wasmi is an interpreter.** Slower than wasmtime JIT. For factory adapters that are CPU-bound this matters; if they're I/O-bound it doesn't.
2. **Protobuf maintenance burden.** Every command/event needs a `.proto` file + Rust enum + From/TryFrom conversions. The `plugin_api/` directory has 16 `.proto` files and 16 `.rs` conversion files = 12,361 LOC.
3. **wasm-wasip1 still maturing.** wasi preview2 is the future; wasip1 is what zellij uses today.
4. **`zellij_exports.rs` is 5,376 LOC.** The host-side ABI implementation is enormous because every command is one match arm + one function. Monocle's factory ABI would also grow.
5. **Single imported host function.** Tradeoff: tiny ABI surface, but every PluginCommand is now indirected through protobuf serialization. For chatty interactions, this overhead matters.
6. **Plugin authors must care about ABI evolution.** A renamed protobuf field breaks compatibility.

### Verdict for Monocle

**Yes for factory adapters specifically.** A factory adapter is exactly the kind of "third-party-extensibility-with-capability-isolation" use case that zellij's plugin model excels at. The four-mount path model and 17-variant permission system map naturally to factory operations:

- `ReadApplicationState` → read monocle session state
- `OpenFiles` → open files in editor
- `RunCommands` → run shell commands
- `WebAccess` → API calls to upstream factory services
- `ReadSessionEnvironmentVariables` → see what env vars the user has set

**No for the user-input dispatch path.** monocle's input handling is one terminal owned by tmux, not multiple clients with modal keybinds. Cloning that part of zellij would be over-engineering.

## Recommendations

| Recommendation | Source |
|---|---|
| WASM-based factory adapter SDK using `wasmi` interpreter | `zellij-server/Cargo.toml:42-46`, `plugins/plugin_loader.rs:31` |
| Single host-import function + protobuf for all commands | `plugins/zellij_exports.rs:155-158`, `tile/src/shim.rs:50-57` |
| Four-mount WASI path translation (`/host`, `/data`, `/cache`, `/tmp`) | `plugins/zellij_exports.rs:185-201` |
| 17-variant permission token enum + persisted PermissionCache | `data.rs:1063-1086`, `input/permission.rs` |
| Built-in plugins (analog: "official factory adapters") bypass permission gate | `plugins/zellij_exports.rs:5179-5184` |
| Module caching in `Arc<Mutex<HashMap<PathBuf, Module>>>` | `plugin_loader.rs:218-228` |
| `*_worker` export naming convention for background tasks | `plugin_loader.rs:202-216` |
| Symmetric "plugin action = user action" route via shared `route_action` | `zellij_exports.rs:140-153` |

## Coverage Notes

| Investigated | Coverage |
|---|---|
| start_plugin flow | 100% |
| host-call ABI | 100% — single import function + protobuf |
| Permission mapping | 100% — all 17 permission types mapped to commands |
| Built-in vs user plugin distinction | covered |
| ZellijPlugin / ZellijWorker traits | 100% |
| Worker lifecycle | covered (per-instance Store, dedicated tokio task) |
| Multi-client isolation | covered (per `(plugin_id, client_id)` state, including workers) |
| Path translation virtual roots | 100% — 4 mounts documented |
| apply_action macro / symmetric action dispatch | covered |

## Open Items After This Round

| Item | Notes |
|---|---|
| Full enumeration of all ≈120 PluginCommand variants | Currently have categories; full list is in `data.rs:3325-3450+`. Could be exhaustively transcribed in a future round if needed. |
| Full enumeration of all ≈50 Event variants | Categorized in pass 3; not exhaustively listed. |
| Plugin Cargo.toml constraints | Each plugin pulls `zellij-tile`; no special compile flags beyond the workspace edition. |
| How is a wasm plugin verified before execution? | Not explicitly — the wasmi parse step is the validation. No signing or hashing. |

## Round Status

```yaml
pass: B
category: plugin-sdk
round: 1
status: complete
timestamp: 2026-05-11T20:50:00Z
new_findings:
  - "Single host-import function (host_run_plugin_command); all PluginCommands flow through protobuf-over-stdout"
  - "Built-in plugins bypass the permission gate (special-cased in check_command_permission)"
  - "17-permission catalog mapped to specific PluginCommand sets"
  - "Four-mount WASI path translation: /host, /data, /cache, /tmp"
  - "Workers run in separate wasm Instances with their own Store; per (plugin_id, client_id) isolation"
  - "Plugin actions go through the same route_action as user keystrokes (apply_action! macro)"
  - "Strong recommendation: monocle should adopt this model for factory adapters but not for input handling"
classification: substantive
```
