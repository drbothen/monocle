# Phase B Deep — Configuration & Keybinds (Round 1)

## Config File Resolution

`Config::try_from(opts: &CliArgs)` (`zellij-utils/src/input/config.rs:166-200`) walks this priority order:

1. **Explicit `--config <path>` flag**: parse that file. Start from `Config::from_default_assets()` as the base.
2. **`zellij setup --clean`**: return `Config::from_default_assets()` only (no user config).
3. **`--config-dir <dir>`** or `home::find_default_config_dir()`: if `<dir>/config.kdl` exists, parse it on top of defaults. Otherwise return `from_default_assets()`.

`find_default_config_dir` looks at:
1. `ZELLIJ_CONFIG_DIR` env var
2. `XDG_CONFIG_HOME/zellij` or `~/.config/zellij` (via `directories::ProjectDirs`)
3. fallback locations per platform

`ZELLIJ_CONFIG_FILE` env var overrides the file name entirely.

The default config file name is `DEFAULT_CONFIG_FILE_NAME = "config.kdl"`.

## Config Aggregate (full)

`zellij-utils/src/input/config.rs:31-41`:

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

8 sub-aggregates. Every one of them implements `merge(self, other) -> Self` semantics.

## Config Merge Semantics

`Config::merge` (`config.rs:264-271`):

```rust
pub fn merge(&mut self, other: Config) -> Result<(), ConfigError> {
    self.options = self.options.merge(other.options);
    self.keybinds.merge(other.keybinds.clone());
    self.themes = self.themes.merge(other.themes);
    self.plugins.merge(other.plugins);
    self.ui = self.ui.merge(other.ui);
    self.env = self.env.merge(other.env);
    Ok(())
}
```

Each sub-aggregate decides its own merge policy. For example, `Themes::merge` (`theme.rs:60-66`):

```rust
pub fn merge(&self, mut other: Themes) -> Self {
    let mut merged = self.clone();
    for (name, theme) in other.0.drain() {
        merged.0.insert(name, theme);  // last-writer-wins per theme name
    }
    merged
}
```

So themes merge by **named-overwrite**: a user-defined theme named "gruvbox-dark" replaces a default theme with the same name.

`UiConfig::merge` (`theme.rs:14-22`): `other.pane_frames` replaces self.pane_frames entirely (`FrameConfig::merge` overwrites each field unconditionally — there's no "preserve default if not specified" logic; if `other` has the field, it wins).

`Options::merge`: each `Option<T>` field uses `other.field.or(self.field)` semantics — explicit-in-other overrides, otherwise keep self.

Keybinds merge is more nuanced — see next section.

## Keybinds Structure

`zellij-utils/src/input/keybinds.rs:11`:

```rust
pub struct Keybinds(pub HashMap<InputMode, HashMap<KeyWithModifier, Vec<Action>>>);
```

Two-level map. Each `(InputMode, KeyWithModifier)` cell holds an ordered `Vec<Action>` — multiple actions fire on one key press, executed in order. The `default.kdl` shows this clearly:

```kdl
bind "n" { NewPane; SwitchToMode "Normal"; }
```

Two actions for the `n` key in `pane` mode: NewPane then SwitchToMode("Normal").

## Default-Action-for-Mode (the missing-binding fallback)

`keybinds.rs:71-99`:

```rust
pub fn default_action_for_mode(
    &self,
    mode: &InputMode,
    key_with_modifier: Option<&KeyWithModifier>,
    raw_bytes: Vec<u8>,
    default_input_mode: InputMode,
    key_is_kitty_protocol: bool,
) -> Action {
    match *mode {
        InputMode::Locked => Action::Write { ... },
        mode if mode == default_input_mode => Action::Write { ... },
        InputMode::RenameTab => Action::TabNameInput { input: raw_bytes },
        InputMode::RenamePane => Action::PaneNameInput { input: raw_bytes },
        InputMode::EnterSearch => Action::SearchInput { input: raw_bytes },
        _ => Action::NoOp,
    }
}
```

So in Locked mode or the default mode (typically Normal), unbound keys pass raw bytes to the focused pane's stdin (`Write` action). In modal-input modes (RenameTab, RenamePane, EnterSearch), unbound keys append to the in-progress input. In other modes, unbound keys are silently swallowed (NoOp).

## InputMode Catalog

14 modes (`data.rs:1146-1196`):

| Mode | Default leader (from `example/default.kdl`) | Purpose |
|---|---|---|
| Normal | (initial) | Default; passes input to pane unless leader pressed |
| Locked | `Ctrl+g` | All shortcuts disabled; passes everything |
| Resize | `Ctrl+n` | Resize active pane |
| Pane | `Ctrl+p` | Pane navigation/management |
| Tab | `Ctrl+t` | Tab navigation/management |
| Scroll | `Ctrl+s` | Scrollback navigation |
| EnterSearch | `s` in Scroll | Type search query |
| Search | (after Enter) | Step through search results |
| RenameTab | `r` in Tab | Type new tab name |
| RenamePane | `c` in Pane | Type new pane name |
| Session | `Ctrl+o` | Session management |
| Move | `Ctrl+h` | Pane movement |
| Prompt | (plugin-triggered) | Interactive prompt |
| Tmux | `Ctrl+b` (optional preset) | tmux-style binding scheme |

## KDL Keybind Syntax

From `example/default.kdl`:

```kdl
keybinds {
    normal {
        // uncomment this and adjust key if using copy_on_select=false
        // bind "Alt c" { Copy; }
    }
    pane {
        bind "Ctrl p" { SwitchToMode "Normal"; }
        bind "n" { NewPane; SwitchToMode "Normal"; }
        bind "d" { NewPane "Down"; SwitchToMode "Normal"; }
        bind "x" { CloseFocus; SwitchToMode "Normal"; }
        bind "f" { ToggleFocusFullscreen; SwitchToMode "Normal"; }
        bind "i" { TogglePanePinned; SwitchToMode "Normal"; }
    }
    // ... etc per mode
    session {
        bind "Ctrl o" { SwitchToMode "Normal"; }
        bind "w" {
            LaunchOrFocusPlugin "session-manager" {
                floating true
                // ...
            }
        }
    }
}
```

Special directive: `keybinds clear-defaults=true { ... }` discards all built-in defaults so the user-provided table is authoritative.

## Reconfigure Flow (runtime config mutation)

When a plugin invokes `PluginCommand::Reconfigure(String, bool)` — where `String` is a stringified KDL config and `bool` is "also write to disk":

1. `zellij_exports.rs` `Reconfigure` arm calls the server.
2. Server's `Reconfigure` `ServerInstruction` arm calls `session_configuration.reconfigure_runtime_config(&client_id, stringified_config)`.
3. `SessionConfiguration::reconfigure_runtime_config` (`zellij-server/src/lib.rs:228-251`):
   - Parses incoming KDL on top of the current client's `Config` as baseline (so partial reconfigures work).
   - If parsed successfully, replaces the per-client runtime_config entry.
   - Returns `(Option<Config>, bool)` — the full new config and whether anything changed.
4. If `config_changed`, `SessionMetaData::propagate_configuration_changes` (`zellij-server/src/lib.rs:373-450+`) fans the new config out to:
   - `default_shell` (from `options.default_shell`)
   - Theme dark/light selections (`options.theme_dark`, `options.theme_light` with `theme_config` lookups; logs a warning if the named theme doesn't exist)
   - The Plugin thread (so plugins get `PluginInstruction::Reconfigure { client_id, keybinds, default_mode, default_shell, layout_dir, was_written_to_disk }`)
5. If `write_config_to_disk == true`, the new stringified KDL is also written to the config file via `Config::write_config_to_disk`.

`PluginCommand::RebindKeys` is similar but surgical (per-key add/remove rather than full file replace).

## Config File Watcher (hot-reload)

`zellij-utils/src/input/config.rs:442-510`:

```rust
pub async fn watch_config_file_changes<F, Fut>(config_file_path: PathBuf, on_config_change: F)
where
    F: Fn(Config) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    // Watch file directly (not parent dir to avoid "too many open files" with many files)
    // Use PollWatcher with 1-second poll interval (not native fs events, for reliability across OSes)
    // On remove, periodically poll for re-creation
    // On modify/create, wait 100ms for editor write completion, then reparse + invoke callback
}
```

Key implementation choices:
- **Direct file watch, not parent dir.** Avoids `EMFILE` (too many open files) on large directories.
- **`PollWatcher` with 1-second interval.** Trades latency for cross-OS reliability — inotify/kqueue have edge cases on every platform.
- **100ms debounce after modify/create.** Editors write+rename or write+truncate; the brief delay lets the file settle.
- **On file deletion → poll for recreation at 3-second intervals.** Editors that delete-and-recreate are tolerated.

Server bootstrap spawns `tokio::spawn(watch_config_file_changes(path, |new_config| async { send_to_server(Reconfigure { ... }) }))`.

There's a sister function `watch_layout_dir_changes` that watches the layout directory similarly.

## Error Surface — KDL Parse Errors

`config.rs:111-180`:

```rust
pub struct KdlError {
    pub error_message: String,
    #[serde(skip)]
    pub src: Option<NamedSource>,  // miette source for column-arrow rendering
    pub offset: Option<usize>,
    pub len: Option<usize>,
    pub help_message: Option<String>,
}
```

When `Config::from_path` fails:
1. The underlying `kdl::KdlError` is caught.
2. Specific error patterns get human-friendly messages (e.g. `Context("valid node terminator") → "Missing ; after node name, eg. { node; another_node; }"` etc.)
3. A `NamedSource` is attached so miette can render the file with a caret pointing at the offending span.

This is a well-thought-out UX consideration that monocle should mirror.

## Setup CLI

`zellij setup` subcommands (from `zellij-utils/src/setup.rs:1-100+` and `zellij-utils/src/cli.rs`):

- `zellij setup --dump-config` — print the default config to stdout
- `zellij setup --dump-layout <name>` — print a named default layout
- `zellij setup --dump-swap-layout <name>` — print a named swap layout
- `zellij setup --dump-plugins <dir>` — extract built-in plugin wasm files
- `zellij setup --check` — diagnose installation
- `zellij setup --clean` — reset to factory defaults

These are user-facing tools for bootstrapping a config. Monocle should ship `monocle setup --dump-config` similarly.

## Default Assets

`zellij-utils/src/setup.rs:43-79`:

```rust
pub const DEFAULT_CONFIG: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "assets/config/default.kdl"));
pub const DEFAULT_LAYOUT: &[u8] = include_bytes!(...);
pub const DEFAULT_SWAP_LAYOUT: &[u8] = include_bytes!(...);
pub const STRIDER_LAYOUT: &[u8] = include_bytes!(...);
pub const STRIDER_SWAP_LAYOUT: &[u8] = include_bytes!(...);
// ... more layouts
```

Plus `ZELLIJ_DEFAULT_THEMES: Dir` via `include_dir!` (`consts.rs:23`) — a directory of theme files compiled into the binary.

These embedded assets are extracted to the user's config dir on first run via `setup::install_default_assets`. The `disable_automatic_asset_installation` feature gates this behavior off for read-only-filesystem deployments.

## Layout Format (KDL)

`example/layouts/multiple_tabs_layout.kdl` shows the canonical layout syntax. A layout file describes:
- `layout { ... }` block (top-level)
- `tab name="..." { pane ... }` children
- `pane` nodes with `command`, `args`, `cwd`, `name`, `focus`, `borderless`, `floating`, `pinned`, `stacked`, plus a `split_direction "Horizontal"/"Vertical"` for nested layouts
- `swap_tiled_layout { tab { ... } }` / `swap_floating_layout` for layout-cycling-via-key

`PluginCommand::ParseLayout(string)` lets plugins receive parsed layouts back.

## Plugin Configuration in KDL

Plugins can be declared in `config.kdl`:

```kdl
plugins {
    tab-bar location="zellij:tab-bar"
    status-bar location="zellij:status-bar"
    strider location="zellij:strider"
}
```

The `location=` specifies either:
- `zellij:<name>` — built-in (`is_builtin() == true`)
- `file:/path/to/plugin.wasm` — local file
- `https://...` — remote (downloaded + cached)

Plugin-instance configuration is a free-form `BTreeMap<String, String>` passed to the plugin's `load()`:

```kdl
plugin location="zellij:status-bar" {
    mode_normal_color "#ffffff"
}
```

## Recommendations for Monocle

| Recommendation | Source |
|---|---|
| KDL (or similarly span-aware format) for human-edited config | `kdl` 4.5.0 + `KdlError` w/ miette `NamedSource` for source-pointer error rendering |
| Layered config with explicit merge semantics per sub-aggregate | `Config::merge` (`config.rs:264-271`) |
| Hot-reload via PollWatcher on the file directly (not parent dir) | `watch_config_file_changes` (`config.rs:442-510`) |
| Per-client runtime_config overlay over saved_config | `SessionConfiguration` (`zellij-server/src/lib.rs:185-308`) |
| `setup --dump-*` CLI surface for bootstrapping | `setup.rs:1-100+` |
| Embed defaults via `include_bytes!` / `include_dir!`; extract to user dir on first run | `setup.rs:43-79`, `consts.rs:23` |
| Human-friendly error messages for known parse failures | `config.rs:111-180`, the match on `KdlErrorKind::Context` |
| `clear-defaults=true` directive to discard built-in keybinds | `example/default.kdl` line 1 comment |

## Open Items After This Round

| Item | Notes |
|---|---|
| Default `assets/config/default.kdl` full inventory | Used `example/default.kdl` as proxy — they're related but the asset file IS the canonical embedded default. Probably nearly identical. |
| Full Action enum (~170 variants) | `actions.rs` is 3,919 LOC; categorized in pass 3 but not exhaustively transcribed. Available for round 2 if needed. |
| Layout swap semantics | The swap-layout system warrants deepening if monocle cared about it — but the user excluded "layout geometry" from scope. |

## Round Status

```yaml
pass: B
category: configuration-and-keybinds
round: 1
status: complete
timestamp: 2026-05-11T20:55:00Z
new_findings:
  - "Config::try_from priority: --config flag > config dir > defaults"
  - "Config::merge is per-sub-aggregate; each implements its own merge policy"
  - "Themes merge by named-overwrite; Options merge by other.or(self) per-field"
  - "PollWatcher with 1-second poll interval (not native fs events) for cross-OS reliability"
  - "100ms debounce after modify+create accommodates editor write-and-rename patterns"
  - "Hot-reload via watch_config_file_changes feeds a Reconfigure ServerInstruction"
  - "Plugins can reconfigure via PluginCommand::Reconfigure(stringified_kdl, save_to_disk)"
  - "RebindKeys is a surgical alternative to full Reconfigure"
classification: substantive
```
