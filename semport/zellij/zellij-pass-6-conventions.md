# Pass 6: Conventions & Patterns — zellij

## Naming

| Pattern | Example | Source |
|---|---|---|
| Crate names use kebab-case with `zellij-` prefix | `zellij-utils`, `zellij-client`, `zellij-server`, `zellij-tile`, `zellij-tile-utils` | workspace `Cargo.toml:39-62` |
| Member modules use snake_case | `zellij-server/src/thread_bus.rs`, `plugins/wasm_bridge.rs`, `pty_writer.rs` | filesystem |
| Instruction-enum naming | `<Subsystem>Instruction` — `ServerInstruction`, `ScreenInstruction`, `PluginInstruction`, `PtyInstruction`, `PtyWriteInstruction`, `ClientInstruction`, `BackgroundJob` (the odd one out) | many |
| Wire-message naming | `<Source>To<Dest>Msg` — `ClientToServerMsg`, `ServerToClientMsg` | `ipc.rs:95-167, 174-204` |
| Plugin-side identity types are aliases | `pub type ClientId = u16;` (both `server/src/lib.rs:79` and `utils/src/data.rs:31`) — duplicated; comment notes "TODO: merge with crate type?" | `data.rs:31` |
| Domain types in lib.rs of a module-as-directory | `zellij-utils/src/input/` (mod.rs + actions.rs + keybinds.rs + …) | `input/mod.rs:1-12` |
| Test module naming | `mod theme_test;` with `#[path = "./unit/theme_test.rs"]` and `#[cfg(test)]` | `theme.rs:147-149` |
| `unit/` subdirectory for unit tests | `zellij-server/src/unit/`, `zellij-utils/src/input/unit/`, `zellij-client/src/unit/` | filesystem |
| Snapshot tests via insta | `*/unit/snapshots/`, plus `[dev-dependencies] insta = "1.6.0"` | many `Cargo.toml`s |
| Plugin-side helpers in shim.rs | `zellij-tile/src/shim.rs` — 2,864 LOC of host-call wrappers | filesystem |

## Module Organization

| Pattern | Example | Notes |
|---|---|---|
| Per-target conditional file via `#[path]` | `#[path = "os_input_output_unix.rs"] mod os_input_output_unix;` | `zellij-client/src/lib.rs:3-9`, `zellij-server/src/lib.rs:1-5` |
| `target_family` gating in Cargo.toml | `[target.'cfg(not(target_family = "wasm"))'.dependencies]` | `zellij-utils/Cargo.toml` |
| Feature-gated modules | `#[cfg(feature = "web_server_capability")] pub mod web_client;` | `zellij-client/src/lib.rs:18-23` |
| Module-per-file vs module-as-directory | `kdl/` is a directory (mod.rs + kdl_layout_parser.rs + snapshots/); `keybinds.rs` is a single file | filesystem |
| Tests colocated in `unit/` siblings | `zellij-server/src/plugins/` (the plugin code) + `zellij-server/src/plugins/unit/` (the tests) | filesystem |
| Vendored deps lived in tree | `zellij-utils/src/vendored/termwiz/` — vendored copy of termwiz parsing | `zellij-utils/src/vendored/` |
| Protobuf-pair files: `.proto` + `.rs` together | `plugin_api/event.proto` + `plugin_api/event.rs` (Rust ↔ proto conversion) | `plugin_api/` |

## Error Handling

| Pattern | Example |
|---|---|
| `anyhow::Result` is the default return type | Almost every `pub fn` in server / client returns `Result<()>` or `Result<T>`. |
| `prelude` re-exports `anyhow::*` + custom traits | `zellij-utils/src/errors.rs:21-32` — `pub use anyhow::{anyhow, bail, Context, Error as anyError, Result}` plus `FatalError`, `LoggableError`, `ToAnyhow`, `ZellijError`. |
| `.context("description")?` everywhere | `thread_bus.rs:27-29` — `self.to_screen.as_ref().context("failed to get screen sender")?.send(instruction).to_anyhow().context("failed to send message to screen")` |
| Custom traits to chain log behavior | `LoggableError::to_log()`, `to_stderr()`, `to_stdout()`, and `FatalError::non_fatal()` / `fatal()` | `errors.rs:43-160` |
| `bail!` for early error return | Used throughout `kdl/mod.rs`. |
| Domain enum for typed errors: `ZellijError` | `errors.rs` — variants like `ZellijError::PluginDoesNotExist`. (`plugin_map.rs:165-167`) |
| KDL errors carry miette span info | `KdlError { error_message, src: NamedSource, offset, len, help_message }` (`config.rs:43-78`). |
| `#[track_caller]` on log helpers | `errors.rs:55, 71, 92` — preserves caller location instead of error-helper location. |

## Test Patterns

| Pattern | Example |
|---|---|
| Insta snapshot for golden-file tests | `[dev-dependencies] insta = "1.6.0"` in every Cargo.toml. 10 `snapshots/` dirs across the codebase. |
| `expect-test` for inline expectations | `zellij-utils/Cargo.toml: expect-test = "1.4.1"` |
| `serial_test` for tests that can't run concurrently | `zellij-client/Cargo.toml: serial_test = "3.0"` |
| `tokio::test-util` for virtual time | `zellij-client/Cargo.toml: features = ["test-util"]`; comment "used by the forward-timeout timer tests" |
| `rcgen` for ad-hoc TLS certs in tests | `zellij-client/Cargo.toml: rcgen = "0.13"` (web client TLS) |
| Test fixtures as a workspace crate | `default-plugins/fixture-plugin-for-tests/` (1,005 LOC) — a real wasm plugin used purely for testing the plugin host. |
| Test directory: `test-fixtures/` | `zellij-utils/src/test-fixtures/` (assets used by config parsing tests). |
| E2E tests in `src/tests/e2e/` | `zellij-utils/src/snapshots/` and `zellij/src/tests/e2e/snapshots/`. |
| `cargo xtask test` | Single entry point in CI — see workflow file. |

## Design Patterns

| Pattern | Where |
|---|---|
| **Actor mesh** | Server-side: each long-lived "subsystem" is a thread with a typed mailbox; `ThreadSenders` is the cross-thread address book. |
| **Length-prefixed protobuf framing** | `ipc.rs:402-426`. Generic over the protobuf message type via `prost::Message`. |
| **Thread-local error context stack** | `OPENCALLS: thread_local!(RefCell<ErrorContext>)` + `task_local!(ASYNCOPENCALLS)`. (`channels.rs:36-43`) |
| **Capability tokens** | 17-variant `PermissionType` enum gates plugin actions. |
| **WASM plugin SDK as host-call shim** | `zellij-tile::shim` writes protobuf to stdout and triggers a single imported function. |
| **Per-`(PluginId, ClientId)` Instance** | The plugin map enforces multiplayer isolation. |
| **Module caching via Arc<Mutex<HashMap<PathBuf, Module>>>** | Avoid re-parsing wasm. (`plugin_loader.rs:218-228`) |
| **KDL config + symmetric serialization** | The session-resurrection persistence proves the parser+serializer round-trips through KDL successfully. |
| **Optional `Box<dyn ServerOsApi>`** | Allows test environments to inject mock OS calls. (`thread_bus.rs:144`) |
| **`Default` + builder pattern** | `Bus::empty()`, `LoadingContext::new(...)`, `LayoutManifest::default()` everywhere. |
| **Newtype wrappers** | `Keybinds(HashMap<...>)`, `Themes(HashMap<...>)`, `PluginAliases(...)` — give domain affordances on top of `HashMap`. |
| **`From<T> for U` for instruction translation** | `From<ServerToClientMsg> for ClientInstruction` (`client/src/lib.rs:196+`). |
| **Visitor pattern (serde)** | `HexColorVisitor` for parsing `#RGB`/`#RRGGBB`. (`theme.rs:99-128`) |
| **Macro-driven plugin registration** | `register_plugin!(MyPlugin)` and `register_worker!(MyWorker)` macros in `zellij-tile`. (`tile/src/lib.rs:8`) |
| **Macro-driven theme styling** | `rgb!`, `palette_match!`, `style!` in `zellij-tile-utils`. (`tile-utils/src/lib.rs`) |

## Consistency Assessment

| Convention | Consistency |
|---|---|
| `anyhow::Result` returns | ~Universal in non-wasm crates |
| Snake_case modules + kebab-case crates | Universal |
| Per-target file via `#[path]` for unix/windows split | Used in 4 places (client lib, server lib, possibly more). Consistent. |
| Newtype wrappers around HashMap-style aggregates | Used for `Keybinds`, `Themes`, `Themes(HashMap<String, Theme>)`, `PluginAliases`. Consistent. |
| Per-`(PluginId, ClientId)` plugin state | Universal in plugin host. |
| Use of `LoggableError` chain | Sporadic — not every error path uses it; `to_log()` shows up where the original was discarded otherwise. |
| `FatalError::non_fatal()` to discard `Result<()>` | Frequent in server bootstrap (`zellij-server/src/lib.rs` etc.). |
| Test-mode-only `should_silently_fail` | Universal across all `ThreadSenders` consumers; one of the cleaner mock patterns in the tree. |
| KDL as config language | Universal — config.kdl, layouts, themes, even session-layout cache. No JSON / TOML / YAML for user-facing config. |
| Plugin tile shim style | Universal — every `shim::*` function is `build PluginCommand → encode_to_vec → object_to_stdout → unsafe { host_run_plugin_command() }`. (Inspected `shim.rs:50-95` — 100% pattern fidelity.) |

## Anti-Patterns / Code Smells (with charity)

| Smell | Where | Note |
|---|---|---|
| Duplicate type alias `ClientId` | `zellij-server/src/lib.rs:79` AND `zellij-utils/src/data.rs:31`. The latter has `// TODO: merge with crate type?` | Acknowledged by maintainers. |
| 47-variant `PluginInstruction` enum with positional unnamed fields | `zellij-server/src/plugins/mod.rs:57-220`. Many variants have 8+ positional `Option<bool>, bool, Option<String>, RunPluginOrAlias, …` parameters with inline `// pane title`-style comments. | Hard to evolve; type-safety good but human-readability suffers. monocle should prefer named-field struct variants. |
| `should_silently_fail` test escape hatch in production type | `ThreadSenders.should_silently_fail` is a struct field, not a wrapper. | Pragmatic but bleeds test concerns into production code. |
| 9,958-line `screen.rs` | `zellij-server/src/screen.rs` | Heroic single-file module — works but hard to onboard. (OUT OF SCOPE for monocle per user.) |
| 7,255-line `kdl/mod.rs` | Centralized KDL parsing. | Same. |
| `colored = "2.0.0"` and `ansi_term = "0.12.1"` both used | utils Cargo.toml | Two color libs for similar use cases. |
| `wasmi_wasi::sync::WasiCtxBuilder` despite tokio async runtime | `plugin_loader.rs:13-15` | wasi-sync calls inside async server thread; potentially blocking. |
| Unbounded crossbeam channels | Default everywhere | Risk of unbounded memory growth under bursty workloads. |
| Mixed `serde_yaml = "0.8"` (deprecated) in client | `zellij-client/Cargo.toml:39` | YAML support is legacy; new code is KDL only. |

## Patterns Worth Adopting for Monocle

| Pattern | Why for monocle |
|---|---|
| ThreadSenders / Bus actor-mesh | Maps directly to monocle's "session-orchestrator vs runner vs factory-adapter" planes. Each plane is a thread with a typed mailbox. |
| `SenderWithContext<T>` carrying error context | Helps cross-thread crash diagnosis. monocle could adopt this verbatim if it grows multi-thread state. |
| Length-prefixed protobuf for cross-process | If monocle ever needs CLI-to-daemon IPC, this is the right format. |
| Permission token capability model for plugins / factory adapters | If a factory-adapter is a third-party plugin, monocle should adopt the explicit-grant model. |
| WASM plugin SDK with stdout-protobuf shim | If monocle ever adds extensible factory adapters, the WASM-host approach scales better than dlopen. |
| KDL config + symmetric serialization | KDL is more human-readable than YAML/TOML for nested config. Worth considering vs the current JSON profile. |
| Cross-OS `#[path]` per-file split | Cleaner than feature flags for unix/windows divergence. |
| Insta snapshots for golden-config tests | Already common in Rust; zellij uses for config parsing. |
| `LoggableError` / `FatalError` traits to keep error chains expressive without ad-hoc `eprintln` | Subtle but powerful. |

## Patterns to AVOID

| Pattern | Why |
|---|---|
| 9k-line modules | Mostly the result of historical accretion; avoid by splitting submodules early. |
| 47-variant enum with positional unnamed fields | Use named-field enum variants. |
| Stringly-typed serialized config in `Reconfigure(String, bool)` | `PluginCommand::Reconfigure` ships the whole config as a KDL string. Monocle should keep typed config end-to-end if possible. |
| Duplicate `ClientId` alias | One canonical location only. |
| `serde_yaml = "0.8"` (deprecated for years) | Avoid. |

## State Checkpoint

```yaml
pass: 6
status: complete
timestamp: 2026-05-11T20:25:00Z
next_pass: 7
```
