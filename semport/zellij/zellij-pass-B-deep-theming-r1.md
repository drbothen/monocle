# Phase B Deep — Theming (Round 1)

## Theme Token Model

`zellij-utils/src/data.rs:1397-1414`:

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

Theme tokens are **semantic, not nominal**. There's no "red" or "blue" — there's `text_unselected.base` and `text_selected.emphasis_0`. This is critical: a theme can be a high-contrast color scheme or a monochrome accessibility theme without breaking the UI.

The 15 token groups (14 normal + 1 multiplayer color set) each have 6 color slots (`base`, `background`, `emphasis_0..3`), totaling **84 distinct colors per theme**.

There's also a legacy `Palette` (`data.rs:1352-1372`) with the classic 16-color ANSI vocabulary (`fg`, `bg`, `black`, `red`, ..., `pink`, `brown`) — this is what older themes use. Both formats are accepted.

## Style Structure (full)

`data.rs:1379-1383`:

```rust
pub struct Style {
    pub colors: Styling,
    pub rounded_corners: bool,
    pub hide_session_name: bool,
}
```

`Style` is the runtime-attached theme — what every rendered pane sees. `rounded_corners` and `hide_session_name` come from `UiConfig.pane_frames` (`zellij-utils/src/input/theme.rs:26-37`).

## Themes Container

`zellij-utils/src/input/theme.rs:38-79`:

```rust
pub struct Themes(HashMap<String, Theme>);

pub struct Theme {
    pub sourced_from_external_file: bool,
    #[serde(flatten)]
    pub palette: Styling,
}
```

The `sourced_from_external_file: bool` flag tracks provenance — was this theme loaded from `assets/themes/*.kdl` (true) or defined inline in the user's `config.kdl` (false)? Useful for "reset to defaults" semantics.

## Theme File Format

### Old-style (Palette, 16 colors)

From `example/themes/example.kdl`:

```kdl
themes {
    gruvbox-light {
        fg 60 56 54           // RGB triple
        bg 251 82 75
        black 40 40 40
        red 205 75 69
        // ...
    }

    gruvbox-dark {
        fg "#D5C4A1"          // hex string
        bg "#282828"
        // ...
    }
}
```

Two acceptable formats:
- Three integer triples (R G B): `fg 60 56 54`
- Hex strings: `fg "#D5C4A1"` (works with `#RGB` or `#RRGGBB`, with or without leading `#`)

Hex parsing implemented via `HexColorVisitor` (`theme.rs:99-128`).

### New-style (Styling, 84-token semantic)

From `assets/themes/gruvbox-dark.kdl`:

```kdl
themes {
    gruvbox-dark {
        text_unselected {
            base 251 241 199
            background 60 56 54
            emphasis_0 214 93 14
            emphasis_1 104 157 106
            emphasis_2 152 151 26
            emphasis_3 177 98 134
        }
        text_selected {
            base 251 241 199
            background 80 73 69
            // ... emphasis_0..3
        }
        ribbon_unselected { ... }
        ribbon_selected { ... }
        // ... 14 more groups
    }
}
```

This is the canonical format for new themes. The old `fg bg red green ...` palette is supported for back-compat but maps onto the new semantic tokens internally.

## Built-in Theme Catalog

`zellij-utils/assets/themes/` contains 41 built-in `.kdl` theme files at this HEAD. Sample:

```
ansi.kdl                  ao.kdl                    atelier.kdl
ayu-dark.kdl              ayu-light.kdl             ayu-mirage.kdl
blade-runner.kdl          catppuccin-frappe.kdl     catppuccin-latte.kdl
catppuccin-macchiato.kdl  catppuccin-mocha.kdl      cyber-noir.kdl
dayfox.kdl                dracula.kdl               everforest-dark.kdl
everforest-light.kdl      flexoki-dark.kdl          gruber-darker.kdl
gruvbox-dark.kdl          gruvbox-light.kdl         ...
```

These are compiled into the binary via:

```rust
pub static ZELLIJ_DEFAULT_THEMES: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/themes");
```
(`consts.rs:23`)

`get_default_themes()` (`setup.rs:30-40`) walks each file, calls `Themes::from_string(content, sourced_from_external_file=true)`, and merges into a single Themes aggregate.

## Theme Selection

In `~/.config/zellij/config.kdl`:

```kdl
theme "gruvbox-dark"
```

That's it — one entry in `Options.theme`. If the named theme is not in `Themes`, the default palette is used.

For dark/light auto-switching:

```kdl
theme_dark "gruvbox-dark"
theme_light "gruvbox-light"
```

If both are set, the host terminal's reported palette (dark or light) determines which is active.

## Runtime Theme Switching

Three actions:
- `Action::SetDarkTheme`
- `Action::SetLightTheme`
- `Action::ToggleTheme`

Plus the implicit `Event::HostTerminalThemeChanged(HostTerminalThemeMode)` reaction (when CSI 2031 reports dark/light).

When any of these fires, the server:
1. Looks up `options.theme_dark` or `options.theme_light` in `Themes`.
2. Calls `Config::theme_config(name)` to get the `Styling` for that theme.
3. Updates `SessionMetaData.session_configuration` (or the per-client runtime config).
4. Sends `PluginInstruction::Update(events_with_ModeUpdate)` so plugins re-render with the new colors.
5. Triggers a full re-render of every pane (`ScreenInstruction::Render`).

## Host Terminal Theme Detection

The client uses two ANSI escape sequences (defined in `zellij-client/src/lib.rs:52-67`):

```rust
const ENABLE_HOST_THEME_NOTIFY: &str = "\u{1b}[?2031h";
const DISABLE_HOST_THEME_NOTIFY: &str = "\u{1b}[?2031l";
const QUERY_HOST_THEME: &str = "\u{1b}[?996n";
```

- **CSI 2031**: Subscribe to host theme change notifications (the host emits unsolicited DSR 997 on change).
- **DSR 996**: Actively query current theme. Reply: `CSI ? 997 ; {1|2} n` where 1=dark, 2=light.

The same `CSI ? 997 ; {mode} n` format is used for both unsolicited notifications and replies to queries.

The stdin parser (`zellij-client/src/stdin_ansi_parser.rs`, out-of-scope per user but mentioned for completeness) recognizes these and emits internal events.

Client emits `ClientToServerMsg::HostTerminalThemeChanged { mode }` upward (`ipc.rs:159-167`).

Server propagates via `Event::HostTerminalThemeChanged(mode)` to subscribed plugins.

## Theme Hot-Reload

Three ways themes change at runtime:

1. **Config file modified** — `watch_config_file_changes` (`config.rs:442-510`) re-parses, fires `ServerInstruction::Reconfigure`, propagates updated themes to all clients.
2. **Plugin command `Reconfigure(string_kdl, save)`** — surgical theme change from a plugin.
3. **Theme directory file added** — `watch_layout_dir_changes` analog exists but theme dir watching is less prominent; users typically just reload config.

## Color Vocabulary

`PaletteColor` (`data.rs:1211-1223`):

```rust
pub enum PaletteColor {
    Rgb((u8, u8, u8)),
    EightBit(u8),
}
```

Two color representations:
- **24-bit RGB** (`Rgb((r, g, b))`) — used for modern themes.
- **8-bit indexed** (`EightBit(idx)`) — for terminals that don't support truecolor.

Most rendering paths handle both via match arms.

## Recommendations for Monocle

| Recommendation | Source |
|---|---|
| Semantic token model (`text_unselected.base` not `red`) | `data.rs:1397-1414` (Styling), `StyleDeclaration` |
| Two color representations: 24-bit RGB AND 8-bit indexed | `PaletteColor` (`data.rs:1211-1223`) |
| 6 slots per token group: base + background + emphasis_0..3 | `StyleDeclaration` (`data.rs:1416-1423`) |
| Theme files in KDL with the same parser as user config | `theme.rs:38-79` |
| Embed default themes via `include_dir!` | `consts.rs:23` |
| `sourced_from_external_file: bool` on Theme | `theme.rs:80-84` (useful for diff-from-defaults logic) |
| `Themes::merge` is named-overwrite | `theme.rs:60-66` |
| `theme_dark` + `theme_light` options for auto-switch | `options.rs:55-65` |
| Three theme actions: `SetDarkTheme`, `SetLightTheme`, `ToggleTheme` | `kdl/mod.rs:74-76` |
| Listen to host theme reports via CSI 2031 / DSR 996 | `zellij-client/src/lib.rs:52-67` |

## Coverage Notes

| Investigated | Coverage |
|---|---|
| Styling token catalog | 100% — 15 groups (14 normal + multiplayer) × 6 slots = 84 colors |
| KDL theme syntax (old + new) | 100% — both palette and styling formats |
| Built-in theme catalog | 41 themes counted in `assets/themes/` |
| Runtime theme actions | 3 actions identified |
| Host terminal theme detection | CSI 2031 + DSR 996 mechanics |
| Theme hot-reload paths | 3 paths catalogued |
| Color representations | RGB + 8-bit |

## Open Items After This Round

| Item | Notes |
|---|---|
| `DEFAULT_STYLES` const | Defined at `data.rs:1440+`; not exhaustively transcribed but visible from the struct definitions. |
| Multiplayer-colors specific use | Used to color the cursor / pane border of each connected client. Not deepened. |
| Frame-corner ASCII for `rounded_corners: true` vs false | Out-of-scope (rendering detail). |

## Round Status

```yaml
pass: B
category: theming
round: 1
status: complete
timestamp: 2026-05-11T21:05:00Z
new_findings:
  - "Semantic-token model: text_unselected/text_selected/ribbon_*/table_*/list_*/frame_*/exit_code_* (14 groups + multiplayer)"
  - "6 slots per token group (base + background + emphasis_0..3) = 84 colors per theme"
  - "Two KDL theme formats accepted: old (Palette: fg bg red green) and new (semantic Styling)"
  - "41 built-in themes embedded via include_dir!"
  - "Theme files use the same KDL parser as user config (and themes can also be inline in config.kdl)"
  - "Host terminal can report dark/light via CSI 2031 (subscribe) + DSR 996 (query); reply format CSI ? 997 ; {1|2} n"
  - "Plugin can hot-swap theme via Reconfigure command"
classification: substantive
```
