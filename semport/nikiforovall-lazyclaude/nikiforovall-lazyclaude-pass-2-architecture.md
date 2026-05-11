# Pass 2: Architecture — nikiforovall/lazyclaude

## Architectural Posture (in one sentence)

A **read-mostly Textual TUI** organized as a thin presentation layer over a service layer that crawls multiple Claude-Code config roots, parses heterogeneous customization files into a uniform `Customization` domain object, then renders/edits them through keyboard-driven panel widgets.

## Top-level shape

```
┌──────────────────────────────────────────────────────────────────────────┐
│ LazyClaude(App, ...Mixins)                src/lazyclaude/app.py:53      │
│                                                                          │
│   compose():                                                             │
│      Container#sidebar                                                   │
│         StatusPanel                       (path | filter | flags)        │
│         TypePanel[1] SLASH_COMMAND                                       │
│         TypePanel[2] SUBAGENT                                            │
│         TypePanel[3] SKILL                                               │
│         CombinedPanel  ([4]Memory | [5]MCP | [6]Hook | [7]LSP)           │
│      MainPane                             (content / metadata)           │
│      Modal overlays (docked bottom, dock-layer "overlay"):               │
│         FilterInput                                                      │
│         LevelSelector                                                    │
│         PluginConfirm                                                    │
│         DeleteConfirm                                                    │
│         MarketplaceModal                  (full-screen overlay)          │
│         MarketplaceConfirm                                               │
│         MarketplaceSourceInput                                           │
│      AppFooter                            (dock=bottom, dynamic)         │
└──────────────────────────────────────────────────────────────────────────┘
```

Layout citations: `app.py:131-178`, CSS grid `styles/app.tcss:4-9` (`grid-size: 2; grid-columns: 1fr 2fr`), layer registration `app.py:64` (`LAYERS = ["default", "overlay"]`).

## Component catalogue

### Application shell

- **`LazyClaude(NavigationMixin, FilterMixin, MarketplaceMixin, CustomizationActionsMixin, HelpMixin, App)`** — `app.py:53`. The `App` subclass uses **Python multiple inheritance as the action-composition mechanism** — every `action_*` handler comes from a mixin; the App itself only owns lifecycle (`__init__`, `compose`, `on_mount`), message routing (`on_type_panel_*`, `on_combined_panel_*`, `on_filter_input_*`), and shared state. (See `.reference/.../mixins/CLAUDE.md` for the in-repo rationale.)

### Mixins (each = a slice of action handlers)

| Mixin | File | Owns |
|---|---|---|
| `NavigationMixin` | `mixins/navigation.py:13` | panel focus (`action_focus_panel_N`), view switching (`action_prev_view`/`action_next_view`), `action_back` |
| `FilterMixin` | `mixins/filtering.py:16` | `action_filter_{all,user,project,plugin}`, `action_search`, `action_toggle_plugin_enabled_filter`, status updates |
| `MarketplaceMixin` | `mixins/marketplace.py:33` | `action_toggle_marketplace`, plugin preview enter/exit, all `on_marketplace_modal_*` plugin/marketplace CRUD via `claude plugin ...` subprocess |
| `CustomizationActionsMixin` | `mixins/customization_actions.py:22` | `action_copy_customization`, `action_move_customization`, `action_delete_customization`, `action_toggle_plugin_enabled`, level-selector + confirm-modal lifecycle |
| `HelpMixin` | `mixins/help.py:8` | `action_toggle_help` (mounts Static overlay at `#help-overlay`) |

### Services (Textual-free)

| Service | Responsibility | LOC |
|---|---|---|
| `ConfigDiscoveryService` | The orchestrator. Walks user/project/plugin scopes, dispatches to type-specific parsers, merges results, caches | 722 |
| `FilesystemScanner` | Driven by a `ScanConfig` (`subdir`, `pattern`, `GlobStrategy`, `parser_factory`); applied uniformly per scope | 116 |
| `GitignoreFilter` | Loads `.gitignore` + a hardcoded skip-dirs list; provides `walk_filtered(root, pattern, max_depth)` | 149 |
| `PluginLoader` | Loads `installed_plugins.json` (V2 schema), three-phase scope enumeration, resolves directory-source plugin source paths via `marketplace.json` | 353 |
| `MarketplaceLoader` | Loads `known_marketplaces.json` + each marketplace's `marketplace.json`, joins against `PluginLoader` for install/enable state across scopes | 306 |
| `MCPParser`, `HookParser`, `LSPServerParser` | JSON-config parsers (multi-output: one Customization per server/event group) | 127 / 88 / 139 |
| `SlashCommandParser`, `SubagentParser`, `SkillParser`, `MemoryFileParser` | Markdown + YAML-frontmatter parsers (single-output, plus structural enrichment) | 89 / 80 / 147 / 148 |
| `CustomizationWriter` | Inverse of parsers. Type-dispatched copy/move/delete. Skill→`copytree`; MD→atomic text copy; MCP/Hook→JSON merge into shared settings | 518 |
| `ConfigPathResolver` | Translates a plugin's `install_path` to the canonical source path (for directory-source marketplaces) | 72 |
| `SettingsService` | Persists `AppSettings` (theme, marketplace_auto_collapse, suggested_marketplaces) at `~/.lazyclaude/settings.json` | 110 |
| `opener` | Cross-platform `open_in_file_explorer` (Windows/macOS/Linux) + GitHub-URL browser open | 42 |

### Widgets (presentation, Textual)

| Widget | Role | Distinctive feature |
|---|---|---|
| `TypePanel` | List of one customization type. Used 3× (slash/subagent/skill) | Tree-expansion for SKILL and MEMORY_FILE; emits `SelectionChanged`, `DrillDown`, `SkillFileSelected`, `MemoryFileRefSelected` |
| `CombinedPanel` | Tabbed list for MEMORY/MCP/HOOK/LSP (panels 4-7) | Active-tab state separate from panel focus; per-tab restored selected_index (`_selected_indices` dict) |
| `MainPane` | Right-hand pane with two modes: `content` (syntax-highlighted) / `metadata` (computed Rich text) | Themable Pygments mapping `detail_pane.py:16-30`; supports separate frontmatter-as-YAML highlighting for `.md` files |
| `StatusPanel` | Top-left status row: path \| level \| Search? \| Disabled? | Pure reactive Static composition |
| `AppFooter` | Bottom keybinding hints; dynamic per mode/selection | `format_keybinding` helper highlights active filter |
| `FilterInput` | Bottom `/`-activated search modal | Real-time `FilterChanged` messages |
| `LevelSelector` | Bottom modal for copy/move destination (1/2/3 = User/Project/Local) | Operation-aware prompt ("Copy to" vs "Move to") |
| `DeleteConfirm`, `PluginConfirm`, `MarketplaceConfirm` | y/n/Esc confirmations | Type-distinct border color (error/accent/warning) |
| `MarketplaceModal` | Full-screen `layer: overlay` browser using Textual `Tree` | `_WrappingTree` subclass with cursor wraparound; per-plugin/-marketplace footer; scope view toggle (user/project); inline scope-selection mode for install (1/2/3) |
| `MarketplaceSourceInput` | Adds a new marketplace by URL/repo + offers curated suggestions | `NavigableInput` swallows j/k/up/down so they propagate to parent for option navigation |

### Helpers

- `widgets/helpers/rendering.py:6-19` — `format_keybinding(key, label, active)` → `"[bold]X[/] Label"` or with `[$primary]` wrap when active.
- `widgets/helpers/rendering.py:25-58` — `build_memory_flat_items` walks memory file tree honoring `expanded_keys`; `render_memory_item` formats one item (root with `▼/▶` marker, or indented `@ref` line).

## Data flow (read path)

```
User invocation                                                   __main__.py:10
    └─ create_app(user_config_path, project_config_path)          app.py:669
        └─ LazyClaude.__init__ initializes services               app.py:80
            └─ on_mount() loads customizations                    app.py:180
                └─ _load_customizations()                         app.py:301
                    └─ ConfigDiscoveryService.discover_all()      discovery.py:158
                        ├─ scan SCAN_CONFIGS for {commands, agents, skills}
                        │     × USER + PROJECT                    discovery.py:165-175
                        ├─ _discover_memory_files()               discovery.py:415
                        ├─ _discover_auto_memory()                discovery.py:486 (~/.claude/projects/<slug>/memory/)
                        ├─ _discover_rules()                      discovery.py:531 (~/.claude/rules/, ./.claude/rules/)
                        ├─ _discover_mcps()                       discovery.py:571 (~/.claude.json, ./.mcp.json)
                        ├─ _discover_hooks()                      discovery.py:622 (settings.json hooks key)
                        ├─ _discover_plugins()                    discovery.py:643 (per-plugin scan + .mcp.json + hooks.json + .lsp.json)
                        └─ _sort_customizations()                 discovery.py:243
                            └─ stable: (type_order, name.lower())
            └─ _update_panels() distributes filtered list         app.py:306
                ├─ TypePanel.set_customizations(c)                type_panel.py:524 (filters by self.customization_type)
                └─ CombinedPanel.set_customizations(c)            combined_panel.py:557 (filters by COMBINED_TYPES)
```

Caching: `ConfigDiscoveryService._cache` is `list[Customization] | None`. `refresh()` clears cache + delegates to `PluginLoader.refresh()` to drop registry cache.

## Data flow (write/edit path)

```
User presses 'c' (copy)
    └─ CustomizationActionsMixin.action_copy_customization()      mixins/customization_actions.py:37
        ├─ _get_available_target_levels(customization)             :138 (returns USER+PROJECT[+LOCAL for MCP/HOOK])
        └─ LevelSelector.show(available, "copy")                  level_selector.py:82
        ↓
User presses '1'/'2'/'3'
    └─ LevelSelector.action_select_*                              :112-130
        └─ posts LevelSelected(level, operation)
        ↓
LazyClaude.on_level_selector_level_selected                       mixins/customization_actions.py:214
    └─ _handle_copy_or_move(c, level, op)                          :165
        ├─ type-dispatch:
        │    MCP   → writer.write_mcp_customization()              writer.py:178
        │    HOOK  → writer.write_hook_customization()             writer.py:94
        │    else  → writer.write_customization()                  writer.py:20
        ├─ if op=="move": writer.delete_*_customization()          (rollback NOT implemented if delete fails)
        └─ self.action_refresh()                                   re-runs ConfigDiscoveryService
```

Failure handling: writer returns `tuple[bool, str]`. On move-after-copy failure, source is left intact and an error toast surfaces — **no atomic move**. P1 risk.

## Plugin install/uninstall flow

```
M ─► MarketplaceModal.show
    user navigates tree
    presses 'i' on uninstalled plugin
        ─► _scope_selection_mode = True, footer shows "1 User  2 Project  3 Local"
    user presses '1'/'2'/'3'
        ─► posts PluginInstallWithScope(plugin, scope)
        ─► MarketplaceMixin.on_marketplace_modal_plugin_install_with_scope
            ─► @work(thread=True) _run_plugin_command(["claude", "plugin", "install", "<id>", "--scope", "<scope>"], ...)
            ─► subprocess.run(cmd, shell=True, cwd=project_root)
                on success: marketplace_modal.refresh_tree() + action_refresh()
                on error : notify error
```

Side-channel: **the actual install is delegated to the `claude` CLI binary** — lazyclaude never directly mutates `installed_plugins.json` for marketplace lifecycle. It only reads and triggers. The exception is `toggle_plugin_enabled` which **does** write directly to `settings.json` / `settings.local.json` via `CustomizationWriter.toggle_plugin_enabled` (`writer.py:442-484`).

## State model (app-level)

`LazyClaude.__init__` allocates (`app.py:80-123`):

| State | Type | Default | Meaning |
|---|---|---|---|
| `_customizations` | `list[Customization]` | `[]` | All discovered items |
| `_level_filter` | `ConfigLevel \| None` | `None` (=all) | Active level filter |
| `_search_query` | `str` | `""` | Substring filter |
| `_plugin_enabled_filter` | `bool \| None` | `True` | `True` = hide disabled plugins; `None` = show both |
| `_panels` | `list[TypePanel]` | populated in `compose` | 3 type panels |
| `_plugin_preview_mode` | `bool` | `False` | "previewing not-yet-installed plugin" toggle |
| `_previewing_plugin` | `MarketplacePlugin \| None` | `None` | Active preview target |
| `_plugin_customizations` | `list[Customization]` | `[]` | Preview's discovered items |
| `_pending_customization` | `Customization \| None` | `None` | Holds source-of-copy until LevelSelector resolves |
| `_panel_before_selector`, `_combined_before_selector` | focus restore | `None`/`False` | Used by `_restore_focus_after_selector` |
| `_help_visible` | `bool` | `False` | Manual help-overlay mount/unmount |
| `_settings` | `AppSettings` | defaults | Persistent settings (theme, suggested marketplaces) |

## Cross-cutting concerns

### Action availability (`check_action`)

`app.py:221-292` is the action-gating sink. Pattern:

1. `exit_preview` available only in preview mode.
2. Filter actions disabled while marketplace modal visible.
3. In preview mode, action allowlist (quit/help/search/path-copy/focus/view) — copy/move/delete disabled.
4. `toggle_plugin_enabled` requires selection with `plugin_info`.
5. `copy/move/delete` require non-plugin, copyable type, non-subfile-of-skill.

This single method centralizes the "what is and isn't possible right now" rule — Monocle's TUI plane should mirror this gate explicitly.

### Theme system

- `themes.py` defines one custom theme `LAZYGIT_THEME` with explicit color variables.
- `DEFAULT_THEME = "gruvbox"` (one of Textual's built-ins).
- `app.py:182-186` registers custom themes on mount and subscribes to theme-change signal for persistence.
- `MainPane._get_syntax_theme` (`detail_pane.py:137`) maps Textual theme names → Pygments theme names with fallback to `monokai`.

### Background work

`@work(thread=True)` (Textual decorator) is used for:

- `app.py:217` — `_initialize_suggested_marketplaces` (settings migration)
- `mixins/marketplace.py:248` — `_run_plugin_command` (subprocess.run of `claude plugin ...`)

All subprocess returns marshall back to the UI thread via `self.call_from_thread(callback, ...)`. No timeouts. No cancellation.

### Logging

**There is no logging.** No `logging` import anywhere in the source tree. Errors are conveyed through:
- `self.notify(msg, severity=...)` for user-facing toasts
- `Customization.error` field for parse failures (rendered in `MainPane._render_metadata`)
- silent swallowing of `OSError`, `json.JSONDecodeError`, generic `Exception` in many service paths (gitignore_filter, settings save, several widget refreshes)

This is the most striking observability gap. P1 for the Rust port.

### Error display

Per-item parse failures are non-fatal: parsers set `Customization.error`, and `display_name` plus `_render_item` show a red `!` marker (`type_panel.py:236`, `combined_panel.py:230`).

The single unhandled-exception path: `app.py:125 _fatal_error` rings the bell, prints traceback, exits. Wired up via `app.run()` default handling.

## Deployment topology

Single-process, single-user, local-only application. No network calls except:
- `webbrowser.open` for GitHub plugin source URLs (`opener.py:31-41`, `widgets/marketplace_source_input.py:187-194`)
- Out-of-process delegation to `claude` CLI which itself fetches from GitHub/marketplaces

No daemon, no server, no IPC. The lazyclaude process owns the terminal until quit.

## Concurrency model

| Surface | Threading model |
|---|---|
| Discovery (`discover_all`) | Synchronous, on UI thread, called on `on_mount` and on each `r` refresh. Result cached. |
| Filter / panel update | Synchronous, on UI thread, called on every keypress event |
| Marketplace data load (`MarketplaceLoader.load_marketplaces`) | Synchronous, on UI thread, on first `M` |
| Plugin install/uninstall/enable/disable/update via CLI | `@work(thread=True)` — a Textual worker thread |
| Settings migration | `@work(thread=True)` once at startup |
| Clipboard ops | Synchronous (`pyperclip.copy` on UI thread) |

No async I/O in services. Refreshes after writes are also synchronous on the UI thread. For a Rust reimplementation, scheduling discovery on a tokio task or rayon pool will be straightforward and unbottled by any concurrency assumptions.

## State Checkpoint

```yaml
pass: 2
status: complete
timestamp: 2026-05-11T17:05:00Z
next_pass: 3
```
