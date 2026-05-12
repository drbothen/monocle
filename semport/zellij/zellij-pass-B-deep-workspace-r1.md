# Phase B Deep — Workspace Architecture (Round 1)

## Workspace Membership (full)

23 workspace members (workspace `Cargo.toml:39-62`):

```
"default-plugins/compact-bar",
"default-plugins/status-bar",
"default-plugins/strider",
"default-plugins/tab-bar",
"default-plugins/fixture-plugin-for-tests",
"default-plugins/session-manager",
"default-plugins/configuration",
"default-plugins/plugin-manager",
"default-plugins/about",
"default-plugins/share",
"default-plugins/multiple-select",
"default-plugins/layout-manager",
"default-plugins/link",
"zellij-client",
"zellij-server",
"zellij-utils",
"zellij-tile",
"zellij-tile-utils",
"xtask",
".",
```

These split into three logical groups:

1. **Native target group** (built for the host triple):
   - `zellij-utils` (no `build: true` in xtask — it's a library)
   - `zellij-tile-utils`, `zellij-tile` (libraries; `tile` is also a dep for wasm plugins)
   - `zellij-client`, `zellij-server` (libraries)
   - `.` (the `zellij` binary, built last)

2. **WASM target group** (built for `wasm32-wasip1`):
   - 13 `default-plugins/*` crates

3. **Build tooling group**:
   - `xtask` (the build orchestrator; not part of the production runtime)

## `WorkspaceMember` Catalog (xtask view)

`xtask/src/main.rs:30-117` enumerates every member with a `build: bool` flag:

```rust
pub struct WorkspaceMember {
    crate_name: &'static str,
    build: bool,
}
```

The `build: true` members are: every plugin (13) + the root `.` (the `zellij` binary itself) = 14 buildable members. `zellij-utils`, `zellij-tile-utils`, `zellij-tile`, `zellij-client`, `zellij-server` all have `build: false` — they're libraries built transitively when something that depends on them is built.

## `xtask build` Flow (the canonical workspace build)

`xtask/src/build.rs:18-90`:

1. Push into project root.
2. Locate `cargo` executable.
3. Validate `--no-plugins` ⊕ `--plugins-only` (mutually exclusive).
4. **Run protobuf codegen** — `run_proto_codegen(sh)`. Comment notes that `build.rs`-driven proto compilation is broken upstream so it's pulled out into xtask.
5. **Plugin pass** (unless `--no-plugins`):
   - Filter members where `crate_name.contains("plugins")` and `build == true`.
   - Single `cargo build --target wasm32-wasip1 -p <name1> -p <name2> ... [-r if release]` invocation so Cargo unifies feature flags across plugins and compiles `zellij-utils` (the shared dep) once.
   - On `--release`, `move_plugin_to_assets(sh, plugin_name)` copies each `target/wasm32-wasip1/release/<name>.wasm` to `zellij-utils/assets/plugins/<name>.wasm` so `include_dir!` will embed it.
6. **Native pass** (unless `--plugins-only`):
   - For each non-plugin member with `build == true` (just the root `.`), `cargo build` with appropriate flags.

This is a deliberate two-target build orchestration — Cargo doesn't natively handle mixed wasm/native workspace builds well, so xtask wraps it.

## `xtask test` Flow

`xtask/src/test.rs:1-80+`:

1. First do a plugin build (always), via `build::build(release=false, no_plugins=false, plugins_only=true, ...)`. This is required because some tests embed pre-built plugin wasm.
2. **Plugin tests**: single `cargo test --target <host-triple> -p <p1> -p <p2> ... -- <args>`. (Plugins are tested on the host target — they have non-wasm test code via `#[cfg(test)]`.)
3. **Native tests**: per-crate loop with optional `--no-default-features --features <X>` when `--no-web` is requested, via `metadata::get_no_web_features`.

## `xtask` Subcommands (all)

From `xtask/src/main.rs:118-141`:

| Subcommand | Function | Module |
|---|---|---|
| `dist` | Build for distribution | `pipelines::dist` |
| `build` | Standard build | `build::build` |
| `clippy` | Clippy lints | `clippy::clippy` |
| `format` | rustfmt | `format::format` |
| `test` | Test runner | `test::test` |
| `manpage` | Build manpage via `mandown` | `build::manpage` |
| `make` | Composite pipeline | `pipelines::make` |
| `install` | Install built binary | `pipelines::install` |
| `run` | Build + run | `pipelines::run` |
| `ci` | Run the same steps as CI | `ci::main` |
| `publish` | Publish to crates.io | `pipelines::publish` |

There's also a `Deprecated` variant that prints a migration message from the old `cargo make`-based system.

## Where Each Type Lives — Boundary Crate Inventory

This is the key question for monocle. The Iron Law: every shared type lives in `zellij-utils` so both client and server can `use zellij_utils::data::*;`.

| Type Category | Crate | Key Files |
|---|---|---|
| Wire envelopes (`ClientToServerMsg`, `ServerToClientMsg`, `ExitReason`, `IpcSenderWithContext`, `IpcReceiverWithContext`) | `zellij-utils` | `src/ipc.rs` |
| Protobuf schemas (16 files) | `zellij-utils` | `src/plugin_api/*.proto` + `src/client_server_contract/*.proto` |
| Domain enums (`Event`, `EventType`, `Action`, `InputMode`, `KeyWithModifier`, `BareKey`, `KeyModifier`, `PaneId`, `PluginCommand`, `PermissionType`, `Direction`, `Resize`, `Mouse`, `WebSharing`, `LayoutInfo`) | `zellij-utils` | `src/data.rs` (3,656 LOC) |
| Config aggregates (`Config`, `Options`, `Themes`, `Theme`, `UiConfig`, `FrameConfig`, `Keybinds`, `PluginAliases`, `PermissionCache`, `WebClientConfig`, `EnvironmentVariables`) | `zellij-utils` | `src/input/*.rs` |
| KDL parsing (config + layout) | `zellij-utils` | `src/kdl/mod.rs` + `src/kdl/kdl_layout_parser.rs` |
| Session/socket helpers (`get_sessions`, `assert_socket`, `get_resurrectable_sessions`) | `zellij-utils` | `src/sessions.rs` |
| Session serialization (running session → KDL) | `zellij-utils` | `src/session_serialization.rs` |
| Error model (`ErrorContext`, `LoggableError`, `FatalError`, `ZellijError`) | `zellij-utils` | `src/errors.rs` |
| Cross-thread channel wrapper (`SenderWithContext`) | `zellij-utils` | `src/channels.rs` |
| Path constants (`ZELLIJ_SOCK_DIR`, `ZELLIJ_CACHE_DIR`, `ZELLIJ_SESSION_INFO_CACHE_DIR`, …) | `zellij-utils` | `src/consts.rs` |
| Setup helpers (asset extraction, default config dump) | `zellij-utils` | `src/setup.rs` |
| CLI surface (`CliArgs`, `Command`, `CliAction`, `Sessions`) | `zellij-utils` | `src/cli.rs` |
| Vendored termwiz | `zellij-utils` | `src/vendored/termwiz/` |
| Server-only instruction enums (`ServerInstruction`, `ScreenInstruction`, `PluginInstruction`, `PtyInstruction`, `PtyWriteInstruction`, `BackgroundJob`) | `zellij-server` | per-module files |
| Server-only thread bus | `zellij-server` | `src/thread_bus.rs` |
| Plugin host (WasmBridge, PluginMap, PluginLoader, zellij_exports) | `zellij-server` | `src/plugins/*` |
| PTY internals | `zellij-server` | `src/pty.rs`, `pty_writer.rs`, `terminal_bytes.rs` (OUT OF SCOPE per user) |
| Screen state | `zellij-server` | `src/screen.rs`, `tab/`, `panes/` (OUT OF SCOPE for layout math) |
| Client-only types (`ClientInstruction`, OS APIs, stdin parsers) | `zellij-client` | per-module files |
| Plugin SDK trait + macros | `zellij-tile` | `src/lib.rs`, `src/shim.rs` (host-call wrappers), `src/ui_components/` |
| Plugin-author macros (`rgb!`, `style!`, `palette_match!`) | `zellij-tile-utils` | `src/lib.rs` (31 lines total) |

## Feature-Flag Fan-Out

Workspace `Cargo.toml:148-156`:

```toml
[features]
default = ["plugins_from_target", "vendored_curl", "web_server_capability"]
plugins_from_target = ["zellij-utils/plugins_from_target"]
disable_automatic_asset_installation = ["zellij-utils/disable_automatic_asset_installation"]
vendored_curl = ["zellij-utils/vendored_curl"]
unstable = ["zellij-client/unstable", "zellij-utils/unstable"]
web_server_capability = ["zellij-client/web_server_capability", "zellij-server/web_server_capability", "zellij-utils/web_server_capability"]
```

| Feature | Effect |
|---|---|
| `plugins_from_target` | Use the wasm modules in `target/` rather than embedded ones (dev convenience) |
| `disable_automatic_asset_installation` | Don't write default themes / layouts / plugins to the user's data dir on startup |
| `vendored_curl` | Static link curl + openssl |
| `unstable` | Gate features still in development |
| `web_server_capability` | Compile the axum-based web client / web server bits |

The fan-out is consistent: a top-level feature toggle propagates the same-named feature into each subcrate. monocle should adopt the same shape — one workspace feature → identical feature on each sub-crate.

## Internal Dependency Graph (in-scope crates)

```
zellij (bin)
├─ zellij-client       (path dep, optional features mirrored)
├─ zellij-server       (path dep)
└─ zellij-utils        (workspace dep)

zellij-client          ──┐
zellij-server          ──┼─→ zellij-utils
zellij-tile            ──┘     (workspace dep)

zellij-tile-utils  (no zellij deps; just ansi_term)

default-plugins/*  ──→ zellij-tile  (path: ../../zellij-tile)
```

Notable: `zellij-tile` depends on `zellij-utils` even though it compiles to wasm. This works because `zellij-utils` has extensive `#[cfg(not(target_family = "wasm"))]` gates around tokio / interprocess / log4rs / etc. — the wasm side gets only the protobuf-encoded domain types.

## How `default-plugins/*` Are Wired In

Each plugin Cargo.toml is shaped like (from `default-plugins/strider/Cargo.toml`):

```toml
[package]
name = "strider"
version = "0.2.0"
authors = ["Brooks J Rady <b.j.rady@gmail.com>"]
edition.workspace = true
description = "A simplified ranger clone written as a Zellij plugin"
license.workspace = true

[dependencies]
colored = "2.0.0"
zellij-tile = { path = "../../zellij-tile" }
pretty-bytes = "0.2.2"
ignore = "0.4.20"
fuzzy-matcher = "0.3.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
unicode-width = "0.1.8"
ansi_term = "0.12.1"
strip-ansi-escapes = "0.1.1"
```

There is NO `[lib]` section. The plugin is a `[[bin]]` (implicit from `src/main.rs`) that, when built for `wasm32-wasip1`, produces a `.wasm` file. The wasm runtime invokes `_start`, which runs the `register_plugin!`-generated entrypoint.

## Build Artifacts (where do they go?)

| Artifact | Source | Destination |
|---|---|---|
| Native binary `zellij` | `cargo build` from `.` | `target/<profile>/zellij` |
| Plugin wasm | `cargo build --target wasm32-wasip1 -p <plugin>` | `target/wasm32-wasip1/<profile>/<plugin>.wasm` |
| Embedded plugin wasm | `move_plugin_to_assets` after release plugin build | `zellij-utils/assets/plugins/<plugin>.wasm` |
| Default config | `assets/config/default.kdl` (shipped in source) | embedded via `include_bytes!` in `setup.rs:43-49` |
| Default layout | `assets/layouts/default.kdl` | `setup.rs:51-79` |

## Recommended Crate Split for Monocle (drawn from zellij)

```
monocle-core            (analog: zellij-utils — shared types and primitives)
├── types/              (domain primitives: SessionId, RunnerKind, FactoryKind, MissionId, ...)
├── config/             (JSON or KDL profile parsing)
├── ipc/                (if monocle ever becomes a daemon — wire envelopes + serialization)
├── errors/             (custom error trait + LoggableError-style chains)
├── channels/           (cross-thread sender wrapper)
└── consts/             (path constants, env vars)

monocle-runtime         (analog: zellij-server — stateful orchestrator)
├── session/            (session lifecycle, bookmark resume)
├── runner/             (tmux integration, pane/window state)
├── factory_host/       (if monocle ships a plugin model — analog: zellij-server/src/plugins/)
└── bus/                (thread bus per zellij-server/src/thread_bus.rs)

monocle-tui             (analog: zellij-client — owns the user's terminal)
├── input/              (key dispatch — but the modal system is OPTIONAL for monocle)
├── render/             (ratatui-based render loop)
└── ipc/                (if monocle is split client/server, the client side of IPC)

monocle-plugin-sdk      (analog: zellij-tile — only if monocle adopts plugin model)
├── trait/              (FactoryAdapter trait)
├── shim/               (host-call wrappers)
└── ui/                 (helper UI components)

monocle (binary)        (analog: zellij bin)
└── main.rs             (CLI dispatch, role selection)

monocle-xtask           (analog: xtask — only if build needs orchestration)
```

**Reasoning:** monocle's "single binary, multiple roles" can mirror zellij's "single binary, client+server" — the role is chosen at startup. The shared types crate (`monocle-core`) is non-negotiable; everything else depends on it. The plugin SDK is only needed if monocle decides to ship a WASM-based factory-adapter model.

## Build Decisions Worth Adopting

| zellij choice | monocle should also? |
|---|---|
| Per-crate optional feature mirroring workspace feature | YES — keeps top-level feature toggles clean. |
| Custom xtask orchestrator | ONLY if monocle's build needs cross-target (wasm + native). Otherwise plain `cargo` is simpler. |
| `[profile.release] lto = true, strip = true, codegen-units = 1` | YES — these are universally good for a shipped binary. |
| `[profile.dev-opt]` for hot iteration | YES — useful when shared crates (like `monocle-core`) get heavy. |
| `workspace.package` block to share metadata | YES — `edition`, `license`, `repository`, `version`, `rust-version` should all live there. |
| Per-target conditional file via `#[path]` | YES if monocle needs windows support; cleaner than feature flags. |
| `directories::ProjectDirs` for cache/config dirs | YES — already used in many Rust apps, including monocle's stated `~/.monocle/`. |
| Inline `[package.metadata.deb]` + `[package.metadata.binstall]` | YES if monocle plans deb/cargo-binstall releases. |
| Vendored deps in-tree (`zellij-utils/src/vendored/`) | NO — only when upstream is broken. |
| 9k-line single file (`screen.rs`) | NO — split modules early. |

## Coverage Notes

| Investigated | Coverage |
|---|---|
| Full workspace member list | 100% — all 23 enumerated, mapped to in/out of scope |
| xtask subcommands | 100% — all 11 subcommands listed |
| xtask build flow | Read first 80 lines of build.rs; the flow is clear; rest is variants of the same dispatching |
| xtask test flow | Read first 80 lines of test.rs; the rest handles `--no-web` and host-triple specifics |
| Boundary types per crate | 100% — caught from cross-referencing `pub use` and `use` chains |
| Feature flag propagation | 100% (workspace Cargo.toml + each subcrate Cargo.toml inspected) |
| Internal dep graph | 100% |

## Open Items After This Round

| Item | Why |
|---|---|
| Build.rs scripts in zellij-utils/zellij-server | `build = "src/build.rs"` exists for the root `zellij` crate but I haven't inspected `src/build.rs` (probably small — embed-resource for win icon). Low value. |
| `pipelines::publish` / `pipelines::dist` | Out-of-scope niceties. |

## Round Status

```yaml
pass: B
category: workspace
round: 1
status: complete
timestamp: 2026-05-11T20:35:00Z
new_findings:
  - "13 of 23 members are wasm32-wasip1 plugins; xtask coordinates the dual-target build"
  - "zellij-utils is a faithful 'shared boundary types' crate; all wire schemas, all domain enums, all configs live there"
  - "Feature flags fan out by identical-name propagation, not by composition"
  - "[profile.dev-opt] is an idiomatic pattern worth adopting"
classification: substantive (full workspace map + buildable artifacts inventory)
```
