# Pass 4: Behavioral Contracts — zellij

These are the load-bearing behavioral contracts that monocle would need to honor if it cloned the same shapes. All claims are file:line-cited.

## BC-DRAFT-001: Client→Server IPC is length-prefixed protobuf over a single bidirectional byte stream

**Preconditions:** A `LocalSocketStream` (Unix) or named-pipe equivalent (Windows) is connected and `try_clone_stream` succeeds. (`zellij-utils/src/ipc.rs:30-42`).
**Postconditions:** Each transmission writes exactly `4 + len` bytes: 4-byte little-endian u32 length prefix followed by `prost`-encoded `ProtoClientToServerMsg` or `ProtoServerToClientMsg`. The receiver reads exactly 4 bytes, then exactly `len` bytes. (`zellij-utils/src/ipc.rs:402-426`)
**Error cases:** Conversion failure (Rust enum ↔ proto enum) logs a warning and yields `None` from `recv_*` — the loop continues. EOF is also `None`. (`zellij-utils/src/ipc.rs:296-310`)
**Wire schema version:** `CLIENT_SERVER_CONTRACT_VERSION: usize = 1`. (`zellij-utils/src/consts.rs:25`)
**Evidence:** `write_protobuf_message`, `read_protobuf_message` at `zellij-utils/src/ipc.rs:402-426`; `tests/` subdir under `zellij-utils/src/ipc/tests`.
**Confidence:** HIGH (the wire format is exhaustively documented in the source comment "we measure the length of the message and transmit it first so that the reader will be able to first read exactly 4 bytes (representing this length) and then read that amount of bytes as the actual message").

## BC-DRAFT-002: Cross-thread messages carry an ErrorContext stack thread-local

**Preconditions:** A thread running zellij sets its `OPENCALLS` (or in tokio task, `ASYNCOPENCALLS`) `RefCell<ErrorContext>` to track the current "call stack" of instructions. (`zellij-utils/src/channels.rs:36-42`)
**Postconditions:** Every `SenderWithContext<T>::send(event)` sends the pair `(event, get_current_ctx())`. The receiver thread can re-establish context when a panic happens. (`channels.rs:24-32`)
**Effect on monocle port:** monocle should consider whether a similar thread-local crash-context is worth the complexity — it's a tooling investment that pays off when debugging multi-thread panics, but adds boilerplate to every send.
**Confidence:** HIGH (explicit in the implementation, with documentation comments referencing the rationale at `errors.rs:1-13`).

## BC-DRAFT-003: The server is an actor mesh, not a single event loop

**Preconditions:** server bootstrap creates `crossbeam::unbounded()` channels for `(ServerInstruction, ScreenInstruction, PtyInstruction, PtyWriteInstruction, PluginInstruction, BackgroundJob)`, plus a per-client `route` thread. (`zellij-server/src/lib.rs:38-90` + `thread_bus.rs:11-23`)
**Postconditions:** Each thread receives only its own message type. Cross-thread sends MUST go through `ThreadSenders::send_to_*` which `context()`-wraps the error. (`thread_bus.rs:25-130`)
**Error case:** A thread can opt into `should_silently_fail` (test-only). (`thread_bus.rs:131-140`)
**Confidence:** HIGH (direct from the typed `ThreadSenders` struct).

## BC-DRAFT-004: Plugin code is wasm32-wasip1, host-bridge is wasmi 0.51 (interpreter)

**Preconditions:** Plugin wasm bytes exist at known URL or filesystem path, retrievable by `plugin_config.resolve_wasm_bytes(plugin_dir)`. (`plugins/plugin_loader.rs:184-186`)
**Postconditions:** `PluginLoader::start_plugin` produces `(Store<PluginEnv>, Instance)`, calls the wasm `_start` and `load` exports, instantiates one extra wasm instance per `*_worker` export. (`plugin_loader.rs:175-181, 188-241`)
**Caching:** Built modules are cached in `PluginCache: Arc<Mutex<HashMap<PathBuf, Module>>>` keyed by source path. (`plugin_loader.rs:218-228`)
**Cross-client cloning:** After loading once, the same `Module` is re-instantiated for each `ClientId` via `clone_instance_for_other_clients`. (`plugin_loader.rs:179-180`)
**WASI preopens:** Plugin gets its own data dir, cache dir, and tmp dir mounted into the WASI sandbox. (`plugin_loader.rs:48-60`)
**Confidence:** HIGH.

## BC-DRAFT-005: Plugin↔Host RPC = write protobuf bytes to stdout, then call `host_run_plugin_command()`

**Preconditions:** Plugin module imports the host function `host_run_plugin_command` (no args, no return). Plugin SDK provides Rust helpers in `zellij-tile/src/shim.rs` that:
1. Build a `PluginCommand` variant.
2. Convert to `ProtobufPluginCommand`.
3. `encode_to_vec()`.
4. Write the bytes to stdout via `object_to_stdout(&bytes)`.
5. Call `unsafe { host_run_plugin_command() }`.

See `zellij-tile/src/shim.rs:50-57` (`subscribe` is the canonical example).

**Postconditions:** Host (`zellij-server/src/plugins/zellij_exports.rs`, 5,376 LOC) reads from the plugin's WASI stdout pipe, decodes the protobuf, switches on the `PluginCommand` variant, and either:
- Acts immediately (e.g. `Subscribe` registers `EventType` set on the plugin's `Subscriptions`).
- Sends a server-thread-bus message (e.g. `OpenFile` → `PtyInstruction::OpenFileInPane`).
- Performs a synchronous query (e.g. `GetPaneScrollback` writes the response back via `wasi_write_object`).

**Confidence:** HIGH.

## BC-DRAFT-006: Plugins must request permissions; the host caches grants persistently

**Preconditions:** Plugin calls `request_permission(&[PermissionType::ReadApplicationState, ...])` early in `load()`. (`zellij-tile/src/shim.rs:85-90`)
**Postconditions:** Host raises a UI prompt to the user (only on first request for that plugin URL). Result is delivered as `Event::PermissionRequestResult(PermissionStatus::Granted | Denied)`. If granted, the `(plugin_name, Vec<PermissionType>)` is added to `PermissionCache.granted` and persisted to the file at `ZELLIJ_PLUGIN_PERMISSIONS_CACHE`. (`zellij-utils/src/input/permission.rs:1-67`)
**Enforcement:** Every command in `zellij_exports.rs` that requires a permission checks it before dispatch — failing the check drops the command silently.
**Side effects:** Plugin should not assume any permission is implicit; the only no-permission API surface is rendering, basic event subscription, and `set_selectable`/`show_cursor`.
**Confidence:** HIGH.

## BC-DRAFT-007: KDL is the canonical config format for the entire app

**Preconditions:** A KDL document exists at `~/.config/zellij/config.kdl` (or env-overridden via `ZELLIJ_CONFIG_FILE` / `ZELLIJ_CONFIG_DIR`). (`zellij-utils/src/consts.rs:11-12`)
**Postconditions:** `Config::from_path` returns the full populated aggregate `{keybinds, options, themes, plugins, ui, env, background_plugins, web_client}`. Parsing is one pass over `KdlDocument`, with errors surfacing as miette-formatted `KdlError` carrying `offset`/`len` for source-pointer rendering. (`config.rs:31-41`, `kdl/mod.rs`)
**Layout files (also KDL):** Parsed by `KdlLayoutParser` (`kdl/kdl_layout_parser.rs:2601 LOC`). The serialization round-trip is symmetric — a running session can be `serialize_session_layout`d back to a parseable KDL doc. (`zellij-utils/src/session_serialization.rs:43-83`)
**Themes (also KDL):** `Themes::from_string` parses a `themes { ... }` block where each named child contains either RGB triples (`fg 60 56 54`) or hex strings (`fg "#D5C4A1"`). (`example/themes/example.kdl`)
**Confidence:** HIGH.

## BC-DRAFT-008: Keybinds are mode-scoped; default action for unbound keys is mode-dependent

**Preconditions:** `Keybinds::get_actions_for_key_in_mode_or_default_action(mode, key, raw_bytes, default_input_mode, key_is_kitty_protocol)`. (`keybinds.rs:38-58`)
**Postconditions:**
- If `(mode, key)` is bound → return cloned `Vec<Action>`.
- Else if `mode == Locked || mode == default_input_mode` → return single-element `vec![Action::Write { key_with_modifier, bytes: raw_bytes, is_kitty_keyboard_protocol }]`.
- Else if `mode == RenameTab` → `Action::TabNameInput { input: raw_bytes }`.
- Else if `mode == RenamePane` → `Action::PaneNameInput`.
- Else if `mode == EnterSearch` → `Action::SearchInput`.
- Else → `Action::NoOp`.

See `keybinds.rs:60-100` (`default_action_for_mode`).

**Confidence:** HIGH.

## BC-DRAFT-009: Session resurrection persists the active layout to KDL on detach

**Preconditions:** A session has at least one running pane and the server has computed a `SessionLayoutMetadata`. (`zellij-server/src/session_layout_metadata.rs`)
**Postconditions:** The server periodically (and on detach) walks `Plugin` thread → `LogLayoutToHd(SessionLayoutMetadata)` → `session_serialization::serialize_session_layout` → writes to `session_layout_cache_file_name(session_name)` = `~/.cache/zellij/session_info/<name>/session-layout.kdl`. (`consts.rs:32-37`, `session_serialization.rs:43-83`, `plugins/mod.rs:138`)
**Resurrection:**
- `zellij ls` calls `get_resurrectable_sessions()` → reads `ZELLIJ_SESSION_INFO_CACHE_DIR`, finds directories that contain a `session-layout.kdl`, and reports `(session_name, elapsed_since_layout_mtime)`. (`sessions.rs:46-92`)
- `zellij attach <name>` (when `<name>` is in resurrectable list) → parses the cached KDL into a `Layout`, replays the pane definitions.
- Pane scrollback / contents are NOT replayed; only structure + Run definitions are. The KDL has `pane_initial_contents` optionally embedded for restoring scrollback snapshot — but this requires opt-in via `serialize_pane_resurrection_pane_metadata`.
**Confidence:** HIGH.

## BC-DRAFT-010: Multiple clients per session — "multiplayer"; per-client `Keybinds`, `InputMode`, plugin instances

**Preconditions:** Server's `WasmBridge` holds per-client maps: `base_modes: HashMap<ClientId, InputMode>`, `keybinds: HashMap<ClientId, Keybinds>`. (`plugins/wasm_bridge.rs` LoadingContext.new at lines 130-150).
**Postconditions:** Every plugin instance is keyed by `(PluginId, ClientId)` — so a plugin running in a shared session has separate state per attached user. The same WASM module is shared (single `Module` cached), but each client gets its own `Instance`. (`plugin_loader.rs:179-180`)
**Caveat:** Plugin authors must NOT assume one global state per plugin; they get one state per attached client, which has surprising consequences for "shared" plugins like layout-manager.
**Confidence:** HIGH.

## BC-DRAFT-011: The `route` thread is the only translator between IPC and the typed instruction bus

**Preconditions:** A `route_thread_main` is spawned per client when they connect. (`zellij-server/src/route.rs:1-90`, `zellij-server/src/lib.rs` references `route::route_thread_main`)
**Postconditions:** It reads `ClientToServerMsg` off the socket, decides which thread (`Screen`, `Plugin`, `Pty`, etc.) should handle each variant, transforms the payload into the right `*Instruction` type, and sends. **No other place in the server is allowed to read from the client socket.**
**Effect:** This is a single bottleneck where you can intercept, validate, or rate-limit any client message — and where the server's authority over the wire schema lives.
**Confidence:** HIGH.

## BC-DRAFT-012: Plugins can spawn background workers via `*_worker` exports

**Preconditions:** Plugin module exports a function named `<name>_worker` (e.g. `search_worker`). (`plugin_loader.rs:198-216`)
**Postconditions:** During load, the host iterates exports, and for each matching the suffix:
1. Creates a fresh `Store<PluginEnv>` + `Instance` (independent of the main plugin).
2. Calls `_start` on the worker.
3. Wraps it in `RunningWorker` and launches a dedicated tokio task with an mpsc channel.
4. Records the channel in the plugin's `workers: HashMap<String, UnboundedSender<MessageToWorker>>` map. (`plugin_map.rs:96-115`)

**Communication:** Plugin uses `post_message_to(worker_name, message, payload)` (`shim.rs`) to send work; worker replies via `post_message_to_plugin(message, payload)` which fires `Event::CustomMessage(message, payload)` back to the main plugin instance.
**Confidence:** HIGH.

## BC-DRAFT-013: Session sockets are validated by liveness probe before being treated as alive

**Preconditions:** `get_sessions()` is called. It reads `ZELLIJ_SOCK_DIR`, finds files of type "IPC socket". (`sessions.rs:14-40`)
**Postconditions (Unix):** For each socket file, opens it, sends `ClientToServerMsg::ConnStatus`, awaits a `ServerToClientMsg::Connected` reply. If the reply doesn't come or `ConnectionRefused` is returned, the socket file is `fs::remove_file`d as stale. (`sessions.rs:139-160`)
**Postconditions (Windows):** Reads the file as a PID, calls `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, ...)` to test liveness, deletes the marker file if the process is gone. (`sessions.rs:163-200`)
**Confidence:** HIGH.

## BC-DRAFT-014: The "host query forwarding" flow lets plugins ask the host terminal questions safely

**Preconditions:** Plugin (or other host component) needs an answer from the user's actual host terminal (e.g. "what is your color palette" via DSR 996, or "what's your title", etc.).
**Postconditions:** Server emits `ServerToClientMsg::ForwardQueryToHost { token, query_bytes }`. The client writes `query_bytes` followed by a "Primary-DA barrier" (a known ANSI query whose reply unambiguously delimits the response window) to stdout. As the client reads stdin within a forwarding window keyed by `token`, raw reply bytes are captured and shipped back as `ClientToServerMsg::ForwardedReplyFromHost { token, reply_bytes }`. Server routes those bytes to whoever issued the original whitelisted query. (`ipc.rs:153-167, 190-200`)
**Why monocle should care:** This is a generally useful pattern for "I need to ask the user's terminal a question without races". monocle does NOT need it directly (tmux insulates the renderer from the terminal), but the modeling is instructive.
**Confidence:** HIGH (documented in source comments).

## BC-DRAFT-015: Plugin SDK exposes ≈120 host-call wrappers; ABI stability matters

**Preconditions:** Plugin links zellij-tile and uses `shim::*` helpers — e.g. `subscribe`, `set_selectable`, `show_cursor`, `request_permission`, `get_plugin_ids`, `open_file`, `switch_to_input_mode`, `write_chars`, `rename_session`, etc.
**Postconditions:** Each helper is one match arm under `PluginCommand`. To maintain ABI stability the `PluginCommand` protobuf cannot ever rename or repurpose a tag.
**Implication for monocle:** Defining a host ABI as protobuf with stable tags (rather than as Rust enums with insta-snapshots) is the right call if monocle ever ships a plugin model. The same approach is used in plugin_api/event.proto, plugin_command.proto, plugin_permission.proto, etc. (16 .proto files in `zellij-utils/src/plugin_api/`).
**Confidence:** HIGH.

## BC-DRAFT-016: Theme can be hot-swapped at runtime via SetDarkTheme/SetLightTheme/ToggleTheme actions

**Preconditions:** `Options.theme_dark` and `Options.theme_light` are both set in config. (`options.rs:55-65`)
**Postconditions:** `Action::SetDarkTheme`/`SetLightTheme`/`ToggleTheme` are dispatched by keybind or by `Event::HostTerminalThemeChanged(mode)`. The server updates its `Style` and broadcasts a re-render. (`kdl/mod.rs:74-76`, `data.rs:1034`)
**Confidence:** HIGH.

## Confidence Summary

| Contract | Source | Confidence |
|---|---|---|
| BC-001 to BC-016 | Direct code reading | HIGH |
| Test-derived | `zellij-utils/src/ipc/tests/`, `zellij-server/src/unit/`, `zellij-client/src/unit/`, `zellij-utils/src/input/unit/`, insta snapshots | (sampled — broad sweep, not exhaustive) |

Gaps: behaviors with no test coverage were not enumerated in this broad pass; deepening pass `zellij-pass-B-deep-ipc-r1.md` etc. will dive into specifics.

## State Checkpoint

```yaml
pass: 4
status: complete
timestamp: 2026-05-11T20:15:00Z
next_pass: 5
```
