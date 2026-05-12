# Pass 5: NFR Catalog — zellij

## Performance

| Decision | Evidence |
|---|---|
| Custom protobuf wire format (vs JSON / bincode) | `prost` workspace dep + 16-file `plugin_api/*.proto`. Length-prefixed; minimal serialization overhead. |
| wasmi (interpreter) vs wasmtime (JIT) | `zellij-server/Cargo.toml:42-46`. Slower than JIT but no JIT-page allocation cost on plugin load, smaller binary footprint, fewer platform constraints. |
| Plugin module caching in memory | `PluginCache: Arc<Mutex<HashMap<PathBuf, Module>>>` — same Module reused across clients (one parse per binary). (`plugin_loader.rs:218-228`) |
| Plugin instance caching to disk | Inferred from `ZELLIJ_CACHE_DIR`/`ZELLIJ_SESSION_CACHE_DIR` + the `skip_cache` flag on `PluginLoader::start_plugin`. |
| Crossbeam channels (not mpsc) | `zellij-utils/src/channels.rs:5-8` — `crossbeam::channel::*` chosen over std::sync::mpsc for better contention behavior. |
| tokio multi-thread runtime sized to physical CPUs | `zellij-client/src/lib.rs:67-90` — `num_cpus::get_physical()`. |
| Release profile aggressive | `[profile.release] lto = true, strip = true, codegen-units = 1` — workspace `Cargo.toml:124-127`. |
| `[profile.dev-opt]` for hot iteration | `inherits = "dev"`, `[profile.dev-opt.package."*"] opt-level = 3` — workspace `Cargo.toml:119-122`. |
| Scroll buffer is configurable, fast-path default | `DEFAULT_SCROLL_BUFFER_SIZE: usize = 10_000`, `SCROLL_BUFFER_SIZE: OnceLock<usize>` — `consts.rs:14-15`. |
| Subscribe-to-pane-renders model rather than full broadcast | `ClientToServerMsg::SubscribeToPaneRenders { pane_ids, scrollback, ansi }` (`ipc.rs:142-147`). Clients request only the panes they need. |
| Initial keybinds shipped once + lightweight ModeUpdate | `Event::InitialKeybinds(KeybindsVec)` (`data.rs:1027-1030`). Plugins that cache keybinds avoid the cost of receiving them on every mode change. |

## Security

| Decision | Evidence |
|---|---|
| Capability-based plugin permissions | 17-variant `PermissionType` enum (`data.rs:1063-1086`), persisted grants in `PermissionCache` (`input/permission.rs`). Every `PluginCommand` arm in `zellij_exports.rs` validates permission before dispatch. |
| WASI sandbox for plugins | `WasiCtx` with explicit preopens for `plugin_own_data_dir`, `plugin_own_cache_dir`, and `ZELLIJ_TMP_DIR` only. (`plugin_loader.rs:46-60`) |
| Filesystem mount granularity per `(PluginId, ClientId)` | Plugin data dir = `ZELLIJ_SESSION_CACHE_DIR / <safe_url> / <pluginid>-<clientid>`. (`wasm_bridge.rs:LoadingContext.new`) |
| Plugin URL path-safety for Windows | `make_plugin_url_path_safe` replaces `:` with `_` on Windows. (`wasm_bridge.rs:55-62`) |
| Host-query whitelisting | The `ForwardQueryToHost`/`ForwardedReplyFromHost` flow only delivers reply bytes for *known* tokens issued by host code with whitelisted queries (Primary-DA barrier). (`ipc.rs:153-167, 190-200`) |
| Web client TLS | `rustls 0.23 (std)`, `tokio-rustls 0.26`, `rustls-pemfile 2`, `rustls-native-certs 0.8`. (`zellij-client/Cargo.toml`) |
| Web auth tokens | Persisted to disk; see `zellij-utils/src/web_authentication_tokens.rs`. |
| Vendored OpenSSL on `vendored_curl` feature | `openssl-sys` static link via `[features].vendored_curl`. (`zellij-utils/Cargo.toml`) |
| Cookies for web client | `axum-extra` with `cookie` feature. |

## Observability

| Decision | Evidence |
|---|---|
| `log` facade + `log4rs` rolling-file appender | `zellij-utils/Cargo.toml: log4rs = "1.2.0", features = ["pattern_encoder", "rolling_file_appender", "compound_policy", "fixed_window_roller", "size_trigger"]` |
| Plugin stderr forwarded to host log | `LoggingPipe` in `zellij-server/src/logging_pipe.rs` (referenced in `plugin_loader.rs:30`). |
| `ErrorContext` stack as crash-context | `zellij-utils/src/errors.rs:1-200`. Captures the chain of recent instructions (`OPENCALLS`, `ASYNCOPENCALLS` task-local). |
| miette + thiserror for human-friendly errors | `miette = { version = "5.7.0", features = ["fancy"] }`, `KdlError` carries `NamedSource` for source spans. |
| Backtrace in `anyhow` results | `anyhow = { features = ["backtrace", "std"] }` |
| `set_panic_handler` on plugin host | Referenced at `errors.rs` (`set_panic_handler` is implemented for plugin panic propagation). |

## Reliability

| Decision | Evidence |
|---|---|
| Liveness probe on session sockets | `assert_socket` walks every socket file in `ZELLIJ_SOCK_DIR` and probes with `ConnStatus`. Cleans up stale sockets. (`sessions.rs:139-200`) |
| Pruning empty session-info folders | `prune_empty_session_info_folders` runs in `create_config_and_cache_folders`. (`consts.rs:57-75`) |
| `daemonize` for backgrounded server | `daemonize = "0.5"` workspace dep, used unix-only. |
| Crossbeam unbounded channels (no message loss) | Default in `zellij-utils::channels`. |
| Periodic session-layout dump for resurrection | `PluginInstruction::LogLayoutToHd` + `UpdateSessionSaveTime(u64)`. (`plugins/mod.rs:138, 197`) |
| Each thread has explicit error context | `Bus<T>` carries `os_input: Option<Box<dyn ServerOsApi>>` and senders, allowing each thread to surface its own context. (`thread_bus.rs:142-198`) |
| `ExitReason::Disconnect` includes recovery message in `Display` | `ipc.rs:225-260` — includes a verbatim "try `zellij attach <name>`" message in the user-facing string. |
| `should_silently_fail` test escape hatch | `ThreadSenders { should_silently_fail: bool }` lets unit tests not panic when no receiver is hooked. (`thread_bus.rs:22, 131-140`) |
| `notify-debouncer-full` for filesystem watch | Used in plugin host to debounce file events fed to plugins. (`plugins/watch_filesystem.rs`) |

## Scalability

| Decision | Evidence |
|---|---|
| One server, N clients ("multiplayer") | `ClientId = u16` allows up to 65,535 clients per session — sufficient for any conceivable use. |
| Per-client plugin instance | `PluginMap::plugin_assets: HashMap<(PluginId, ClientId), ...>` (`plugin_map.rs:30-37`). |
| Per-client `route` thread (not one big multiplexer thread) | `route::route_thread_main` is spawned per attach. (`zellij-server/src/route.rs:1-90`) |
| Plugin workers run on tokio tasks | `plugin_worker(worker)` returns an unbounded `Sender<MessageToWorker>` driven by a tokio task. (`plugin_loader.rs:212-216`, `plugin_worker.rs`) |
| Web client served by axum + tokio | Optional feature; not enabled by default in some build configs. |

## Cross-Platform Support

| Decision | Evidence |
|---|---|
| Unix vs Windows IPC abstraction | `IpcStream` trait + `cfg(unix)` / `cfg(windows)` socket vs named-pipe split. (`ipc.rs:30-42`, `consts.rs`) |
| `daemonize` unix-only | `[target.'cfg(unix)'.dependencies]` in `zellij-server/Cargo.toml` and `zellij-client/Cargo.toml`. |
| `signal-hook` not-windows | Same crate, gated by `[target.'cfg(not(windows))'.dependencies]`. |
| `crossterm` with `windows` feature for Win console | workspace `Cargo.toml:69` — `crossterm = { features = ["windows"] }`. |
| `windows-sys` for raw Win32 calls | `OpenProcess`, `PROCESS_QUERY_LIMITED_INFORMATION`, `CloseHandle` for session-pid checking. (`sessions.rs:163-200`) |
| Windows stdin handler is separate | `stdin_handler_windows.rs` is a parallel module. (`zellij-client/src/lib.rs:15-19`) |
| `os_input_output_unix.rs` / `os_input_output_windows.rs` | Behind `#[path = "..."]` to conditionally compile. (`zellij-server/src/lib.rs:1-5`) |

## Build / Release

| Decision | Evidence |
|---|---|
| Custom `xtask` orchestrator | `xtask/` crate, called from CI via `cargo xtask build / test / format`. |
| Multi-OS matrix in CI | `.github/workflows/rust.yml` — `ubuntu-latest, macos-latest, windows-latest`. |
| `cargo xtask test --no-web` job | Distinct CI job to confirm building without the web_server_capability feature. |
| Protobuf compile in build step | `Install Protoc` action via `arduino/setup-protoc@v3`. |
| `wasm32-wasip1` toolchain for plugins | Set up in CI via `actions-rust-lang/setup-rust-toolchain@v1` with `target: wasm32-wasip1`. |
| `nasm` required on Windows build | For ring/rustls compilation. |
| Plugin wasm shipped in binary | `include_bytes!(...)` in `setup.rs` for `DEFAULT_CONFIG`, `DEFAULT_LAYOUT`, plus `assets/plugins/*.wasm`. |
| Asset extraction at runtime | `disable_automatic_asset_installation` feature for environments that don't want runtime extraction. |

## What's Missing (gaps for monocle to consider)

| Gap | Why monocle might want it |
|---|---|
| No structured tracing (`tracing` crate) | `log` is fine for batched files; for monocle, `tracing` + spans would integrate better with modern observability. |
| No per-thread mailbox capacity limit | Unbounded channels avoid backpressure but allow unbounded memory growth under pathological inputs. monocle should consider bounded channels with `try_send` for "I'd rather drop than OOM" semantics. |
| Plugin schema version is implicit | `CLIENT_SERVER_CONTRACT_VERSION = 1` exists for the IPC wire, but the *plugin* protobuf schema has no version field — relies on additive evolution. |
| No prom/openmetrics export | Server has no metrics endpoint. For monocle's session lifecycle dashboarding, this might be wanted. |
| No SQLite persistence for general state | Only used for `web_server_capability` (`rusqlite` dep). Session metadata is KDL files. |

## State Checkpoint

```yaml
pass: 5
status: complete
timestamp: 2026-05-11T20:20:00Z
next_pass: 6
```
