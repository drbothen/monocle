# Phase B Deepening: App / Bindings / Composition Layer — Round 1

Goal: map the composition layer **above** the widget set — `app.py` (the `App` shell, its MRO, lifecycle, message routing), `bindings.py` (the master keymap), `themes.py`, `styles/app.tcss`, and `__main__.py`. This layer is the direct port template for the monocle ratatui `App` struct.

Scope clarification: widgets were covered in `nikiforovall-lazyclaude-pass-B-deep-widgets-r1.md`. This round is **strictly** the wiring layer above them — what mounts the widgets, what dispatches events, what owns the master state.

## 1. The `LazyClaude` class — composition by MRO

### 1.1 Class declaration

`app.py:53-60`:
```python
class LazyClaude(
    NavigationMixin,
    FilterMixin,
    MarketplaceMixin,
    CustomizationActionsMixin,
    HelpMixin,
    App,
):
```

Five mixins + Textual's `App`. Order matters: Python's C3 linearization resolves methods left-to-right. **The mixins are listed in order of concern depth — Navigation is the most generic (panel focus), Help is the most overlay-specific. `App` is last (most general base).**

The mixins are pure **action containers**. Their __init__.py exports (`mixins/__init__.py:3-7`) names them with consistent `*Mixin` suffix. Their CLAUDE.md (`mixins/CLAUDE.md`) is explicit: "Textual's `action_*` and `on_*` method discovery works via MRO."

### 1.2 Why MRO matters here (Textual mechanics)

Textual's `App.run_action(name)` machinery calls `getattr(self, f"action_{name}")`. The lookup walks the MRO. So:

- `action_quit` → defined in `App` itself (overridden in `LazyClaude.action_quit` at `app.py:543`)
- `action_focus_panel_1` → resolved from `NavigationMixin` (first in MRO that defines it)
- `action_filter_user` → resolved from `FilterMixin`
- `action_copy_customization` → resolved from `CustomizationActionsMixin`
- `action_toggle_help` → resolved from `HelpMixin`
- `action_toggle_marketplace` → resolved from `MarketplaceMixin`

**No mixin overrides another mixin's `action_*`.** The author has carefully partitioned the namespace so each action lives in exactly one mixin. (The author's `mixins/CLAUDE.md` says exactly that: "Identify if functionality belongs in existing mixin or needs new one.")

### 1.3 Method routing within mixins (Textual's `on_<MessageType>` auto-dispatch)

Textual sends messages to widgets by calling `on_<snake_case_message_class>`. The App's `on_*` handlers are auto-discovered through the MRO too. The discovered handlers in `LazyClaude`:

| Handler | Source | Triggered by |
|---|---|---|
| `on_type_panel_selection_changed` | `app.py:414` | `TypePanel.SelectionChanged` |
| `on_type_panel_drill_down` | `app.py:424` | `TypePanel.DrillDown` |
| `on_type_panel_skill_file_selected` | `app.py:452` | `TypePanel.SkillFileSelected` |
| `on_type_panel_memory_file_ref_selected` | `app.py:466` | `TypePanel.MemoryFileRefSelected` |
| `on_combined_panel_selection_changed` | `app.py:433` | `CombinedPanel.SelectionChanged` |
| `on_combined_panel_drill_down` | `app.py:443` | `CombinedPanel.DrillDown` |
| `on_combined_panel_memory_file_ref_selected` | `app.py:472` | `CombinedPanel.MemoryFileRefSelected` |
| `on_filter_input_filter_changed` | `app.py:499` | `FilterInput.FilterChanged` |
| `on_filter_input_filter_cancelled` | `app.py:515` | `FilterInput.FilterCancelled` |
| `on_filter_input_filter_applied` | `app.py:532` | `FilterInput.FilterApplied` |
| `on_level_selector_*` | in `CustomizationActionsMixin` | various confirm messages |
| `on_plugin_confirm_*` | in `CustomizationActionsMixin` | y/n responses |
| `on_delete_confirm_*` | in `CustomizationActionsMixin` | y/n responses |
| `on_marketplace_modal_*` | in `MarketplaceMixin` | plugin lifecycle messages |
| `on_marketplace_confirm_*`, `on_marketplace_source_input_*` | in `MarketplaceMixin` | marketplace add/install |

**Convention:** App-level handlers live in `app.py`; mixin-level handlers (everything that's about a single feature flow) live in the corresponding mixin file.

### 1.4 The composition rule (one-shot port pattern)

| Textual concept | Monocle ratatui equivalent |
|---|---|
| Multiple inheritance with `*Mixin` action containers | A single `App` struct + an `Action` enum + a top-level `match` in `handle_action`. Each "mixin" becomes a Rust `impl App` block in a separate file. |
| `action_<name>` auto-dispatch | Explicit `Action::Name` variants + `match action` arms. Module per concern. |
| `on_<MessageType>` auto-dispatch | An `Event` enum unifying widget messages + central event loop `match event`. |
| Method resolution by MRO | Compile-time dispatch — explicit `match` arms; no implicit fallthrough. |

## 2. Constants on the App class

`app.py:63-78`:

| Constant | Value | Role |
|---|---|---|
| `CSS_PATH` | `"styles/app.tcss"` | Path relative to `app.py` containing the stylesheet. Textual loads on app construction. |
| `LAYERS` | `["default", "overlay"]` | Z-order layers. Help overlay and marketplace modal use `overlay`. |
| `BINDINGS` | `APP_BINDINGS` from `bindings.py` | App-level keymap (see §4) |
| `TITLE` | `f"LazyClaude v{__version__}"` | Window title pre-mount (replaced on mount with `f"{project_name} - LazyClaude"` at `app.py:191`) |
| `SUB_TITLE` | `""` | Updated dynamically by `_update_subtitle` (`app.py:343-365`) |
| `_COPYABLE_TYPES` | tuple of 6 `CustomizationType` values | Which types can be copied/moved/deleted. Slash, Subagent, Skill, Hook, MCP, Memory. **Note: LSP and RULE are excluded.** |
| `_PROJECT_LOCAL_TYPES` | `(HOOK, MCP)` | Types that support PROJECT_LOCAL config level (settings.local.json) |

**Port note:** Monocle's `App` struct should hold these as `const` associated items or as fields initialized in `App::new`. The `_COPYABLE_TYPES` list is a behavior policy — encode as `fn is_copyable(&self, customization_type: CustomizationType) -> bool`.

## 3. `__init__` — state allocation

`app.py:80-123` initializes **38 attributes** as `None`/empty defaults. They cluster into:

| Cluster | Attributes | Purpose |
|---|---|---|
| **Services** | `_discovery_service`, `_filter_service`, `_marketplace_loader`, `_config_path_resolver`, `_settings_service` | Stateless or cache-bearing services |
| **Persistent settings** | `_settings: AppSettings` | Theme, suggested marketplaces |
| **Discovered data** | `_customizations`, `_plugin_customizations` | Domain entities |
| **Filter state** | `_level_filter`, `_search_query`, `_plugin_enabled_filter` | Active query parameters |
| **Widget refs** | 12 attributes for each child widget (`_status_panel`, `_main_pane`, etc.) | Initialized in `compose()` (§5) |
| **Panel list** | `_panels: list[TypePanel]` | Three type panels, populated in `compose` |
| **Focus history** | `_last_focused_panel`, `_last_focused_combined`, `_panel_before_selector`, `_combined_before_selector` | Restore focus after modal/dialog dismissal |
| **In-flight workflow** | `_pending_customization` | Source of copy/move while LevelSelector resolves |
| **Mode flags** | `_help_visible`, `_plugin_preview_mode`, `_previewing_plugin` | Boolean modal/mode states |
| **Config overrides** | `_user_config_path`, `_project_config_path` | CLI args |

### 3.1 The widget-ref-pattern (`Optional` until mount)

All twelve widget refs (`_status_panel`, `_main_pane`, `_filter_input`, `_level_selector`, `_plugin_confirm`, `_delete_confirm`, `_marketplace_modal`, `_marketplace_confirm`, `_marketplace_source_input`, `_app_footer`, `_combined_panel`) are initialized to `None` then assigned during `compose()`. **Every method that uses them first does a truthiness check.** Examples:
- `app.py:154` `self._filter_input = FilterInput(id="filter-input")` then `app.py:537` `if self._filter_input:`
- `app.py:202` `if self._marketplace_modal:`

**Port note:** in Rust, these become `Option<WidgetState>` fields, set by an explicit `build_layout` step in `App::new` or a startup phase. Or — better — model them as owned (non-optional) substates and construct them eagerly in `App::new`. Optional-until-mount is a Textual lifecycle artifact, not domain-required.

### 3.2 Constructor dependency injection

`__init__(discovery_service=None, user_config_path=None, project_config_path=None)` (`app.py:80-93`):

- `discovery_service` is the only injected service — defaults to `ConfigDiscoveryService(...)`.
- All other services (`_filter_service`, `_settings_service`) are constructed eagerly with no DI hooks.
- `_marketplace_loader` and `_config_path_resolver` are **constructed lazily in `on_mount`** because they need `_discovery_service._plugin_loader` to be ready.

**Port note:** monocle should inject all services for testability. Lazy construction at mount is a Textual quirk worth abandoning — initialize everything eagerly with explicit dependency wiring.

## 4. Keybindings — `bindings.py`

### 4.1 The full keymap (29 bindings)

`bindings.py:5-37`:

| Key | Action | Label | Visible? | Priority? | Action Source (mixin) |
|---|---|---|---|---|---|
| `q` | `quit` | Quit | yes | no | `App` (overridden in `app.py:543`) |
| `?` | `toggle_help` | Help | yes | no | `HelpMixin` |
| `r` | `refresh` | Refresh | yes | no | `app.py:547` |
| `e` | `open_in_editor` | Edit | yes | no | `app.py:552` |
| `c` | `copy_customization` | Copy | yes | no | `CustomizationActionsMixin` |
| `m` | `move_customization` | Move | yes | no | `CustomizationActionsMixin` |
| `d` | `delete_customization` | Delete | yes | no | `CustomizationActionsMixin` |
| `C` | `copy_config_path` | Copy Path | yes | no | `app.py:602` |
| `tab` | `focus_next_panel` | Next Panel | **no** | no | `NavigationMixin` |
| `shift+tab` | `focus_previous_panel` | Prev Panel | **no** | no | `NavigationMixin` |
| `a` | `filter_all` | All | yes | no | `FilterMixin` |
| `u` | `filter_user` | User | yes | no | `FilterMixin` |
| `p` | `filter_project` | Project | yes | no | `FilterMixin` |
| `P` | `filter_plugin` | Plugin | yes | no | `FilterMixin` |
| `D` | `toggle_plugin_enabled_filter` | Disabled | yes | no | `FilterMixin` |
| `t` | `toggle_plugin_enabled` | Toggle | yes | no | `CustomizationActionsMixin` |
| `/` | `search` | Search | yes | no | `FilterMixin` |
| `[` | `prev_view` | `[` | yes | no | `NavigationMixin` |
| `]` | `next_view` | `]` | yes | no | `NavigationMixin` |
| `0` | `focus_main_pane` | Panel 0 | **no** | no | `NavigationMixin` |
| `1` | `focus_panel_1` | Panel 1 | **no** | no | `NavigationMixin` |
| `2` | `focus_panel_2` | Panel 2 | **no** | no | `NavigationMixin` |
| `3` | `focus_panel_3` | Panel 3 | **no** | no | `NavigationMixin` |
| `4` | `focus_panel_4` | Panel 4 | **no** | no | `NavigationMixin` |
| `5` | `focus_panel_5` | Panel 5 | **no** | no | `NavigationMixin` |
| `6` | `focus_panel_6` | Panel 6 | **no** | no | `NavigationMixin` |
| `7` | `focus_panel_7` | Panel 7 | **no** | no | `NavigationMixin` |
| `ctrl+u` | `open_user_config` | User Config | **no** | no | `app.py:589` |
| `M` | `toggle_marketplace` | Marketplace | yes | **yes** | `MarketplaceMixin` |
| `escape` | `exit_preview` | Exit Preview | yes | **yes** | `MarketplaceMixin` |
| `escape` | `back` | Back | **no** | no | `NavigationMixin` |

### 4.2 Notable patterns

**Two `escape` bindings.** Lines 35 and 36. Textual evaluates priority-first, then in declaration order. `exit_preview` is `priority=True` so it fires first when `check_action` returns truthy. When **not** in preview mode, `check_action("exit_preview", ...)` returns `False` (see `app.py:227-228`), so Textual falls through to the next `escape` binding which is `back`. **This is the entire mechanism for "Esc returns to previous panel/exits preview" — a check_action-gated priority binding cascade.**

**`priority=True` only on `M` and the first `escape`.** Priority bindings are evaluated before focused widget bindings. This means:
- `M` opens marketplace **even from inside a focused FilterInput** (would otherwise be consumed)
- `escape` in preview mode is intercepted at the App level even if a widget would otherwise consume it.

Default for all other bindings: `priority=False`, meaning the focused widget can swallow the key first.

**Numeric panel focus (`0`-`7`).** Eight numeric bindings. `0` → main pane. `1-3` → TypePanels (SLASH, SUBAGENT, SKILL). `4-7` → CombinedPanel tabs (MEMORY, MCP, HOOK, LSP). The semantics are encoded in `NavigationMixin` (which `app.py` references but is in a sibling file). The bindings file is content-blind — only declares key→action.

**Show vs no-show.** `show=False` items are not displayed in Textual's auto-footer, but **AppFooter is custom** (`widgets/app_footer.py`) so `show` flag is effectively unused. The custom footer reads from its own reactive properties (`can_refresh`, `can_edit`, etc.), not from `BINDINGS`. **The `show` flag is dead config from a port perspective** — `[` and `]` show flags exist but the custom footer doesn't read them.

### 4.3 What the bindings file does NOT declare

| Key | Where declared instead |
|---|---|
| `j`/`k` (cursor down/up) | Widget-level (`TypePanel.BINDINGS`, `CombinedPanel.BINDINGS`) |
| `h`/`l` (collapse/expand) | Widget-level (TypePanel for skills/memory, MarketplaceModal for tree) |
| `g`/`G` (top/bottom) | Widget-level |
| `d`/`u` (page in detail pane) | Widget-level (`MainPane.BINDINGS`) |
| `y`/`n` (confirm) | Widget-level (`PluginConfirm`, `DeleteConfirm`, etc.) |
| `i` (install plugin in marketplace) | Widget-level (`MarketplaceModal`) |
| `Enter` (drill down) | Widget-level (every list widget) |

**Two-tier keymap.** App-level for global actions; widget-level for context-dependent actions. Same key can mean different things in different contexts — `d` is `delete_customization` at app level but `page down` inside `MainPane`. Textual resolves through focus chain: focused widget's bindings first, then App-level priority, then App-level non-priority.

**Port note:** Monocle should encode this two-tier keymap explicitly. The natural Rust pattern: per-widget `handle_key(KeyEvent) -> Option<Action>`; if the widget returns `None`, propagate to the App's global handler. **Plus a priority intercept** at the App level for `M` and the preview-mode `Escape`.

### 4.4 `check_action` — the per-state action gate

`app.py:221-292`. Returns:
- `True` if action available
- `False` if explicitly disabled
- `None` if action should remain available but disabled-style (Textual treats as default behavior)

The gate has five precedence layers:

1. **`exit_preview` is preview-mode only** (`:227-228`). Returns the boolean directly — only ever True in preview mode.
2. **Marketplace modal blocks filters** (`:230-242`). When marketplace modal is visible, the five filter actions (`filter_all`, `filter_user`, `filter_project`, `filter_plugin`, `toggle_plugin_enabled_filter`) return `False`.
3. **Preview mode allowlist** (`:244-264`). In preview mode, only 16 actions remain enabled — quit, help, search, copy-path, panel focus (9 keys), view switch (2), exit-preview. Everything else returns `False`.
4. **`toggle_plugin_enabled` requires a plugin-info-bearing selection** (`:266-269`).
5. **`copy/move/delete` require copyable type, not-skill-subfile, and (move/delete only) not-PLUGIN-level** (`:271-290`).

**This is a hand-rolled state machine** that Textual queries on every `refresh_bindings()` call. The pattern: action availability is centralized in **one method** (rather than scattered as "is this action enabled?" guards across handlers).

**Port note:** Monocle should reproduce this pattern. A `fn is_action_available(&self, action: Action) -> bool` on `App` is the direct port. The widget-side check `_is_skill_subfile_selected` (`app.py:634-645`) is a helper for this gate.

### 4.5 `refresh_bindings` — the gate update trigger

Called from:
- `app.py:422` after `on_type_panel_selection_changed`
- `app.py:441` after `on_combined_panel_selection_changed`
- `app.py:530` after `on_filter_input_filter_cancelled`
- `app.py:539` after `on_filter_input_filter_applied`
- Plus various places in mixins (referenced)

**Every selection or filter change re-runs `check_action` for every binding.** This is how the footer hint reflects the current available-action set. **The custom AppFooter doesn't actually use `check_action` directly** — it has its own `can_refresh`/`can_edit`/`can_copy`/`can_move`/`can_delete` reactives that are kept in sync by `_update_footer_actions` (`app.py:367-410`). **Two parallel mechanisms** for the same goal:
1. Textual's `check_action` controls whether pressing the key works.
2. AppFooter's reactives control whether the key is **shown** in the footer.

These can theoretically drift — and the `_update_footer_actions` logic at `:367-410` duplicates the `check_action` logic at `:266-290`. Both must be updated together. **P2 dup risk** — encoded twice, once for "is action allowed", once for "is action visible in footer".

**Port note:** monocle should derive footer visibility from `is_action_available` directly. Single source of truth.

## 5. `compose()` — declarative widget tree

`app.py:131-178`. Yields top-level widgets in order. Textual's `App` machinery mounts them as children.

### 5.1 The composition tree (corrected from CLAUDE.md)

```
LazyClaude (App, grid layout 1fr 2fr)
├── Container#sidebar          (vertical)
│   ├── StatusPanel#status-panel       height=3
│   ├── TypePanel#panel-slash_command  panel_number=1, height=1fr
│   ├── TypePanel#panel-subagent       panel_number=2, height=1fr
│   ├── TypePanel#panel-skill          panel_number=3, height=1fr
│   └── CombinedPanel#panel-combined   height=1fr
├── MainPane#main-pane         height=100%
├── FilterInput#filter-input              (dock=bottom, hidden by default)
├── LevelSelector#level-selector          (dock=bottom, hidden)
├── PluginConfirm#plugin-confirm          (dock=bottom, hidden)
├── DeleteConfirm#delete-confirm          (dock=bottom, hidden)
├── MarketplaceModal#marketplace-modal    (layer=overlay, hidden)
├── MarketplaceConfirm#marketplace-confirm (dock=bottom, hidden)
├── MarketplaceSourceInput#marketplace-source-input (dock=bottom, hidden)
└── AppFooter#app-footer       (dock=bottom)
```

**The grid is two cells: sidebar (1fr) | main-pane (2fr).** All bottom-docked widgets sit below the grid because of TCSS `dock: bottom`. The MarketplaceModal uses the explicit `overlay` layer (declared at `app.py:64`).

### 5.2 Three panel-numbering schemes (subtle)

`compose` does `for i, ctype in enumerate([SLASH, SUBAGENT, SKILL], start=1)` (`app.py:142`) and sets `panel.panel_number = i`. **So TypePanels are panel_number 1/2/3.**

CombinedPanel doesn't have a `panel_number` per-tab, but the App's `action_focus_panel_4..7` (in NavigationMixin) maps numeric keys to (focus combined + set active_type to nth COMBINED_TYPE):
- `4` → MEMORY_FILE tab
- `5` → MCP tab
- `6` → HOOK tab
- `7` → LSP_SERVER tab

`0` is main pane (`action_focus_main_pane`).

**The numeric scheme overflows the docs slightly.** README and CLAUDE.md (in main project root) say `0-6` panel focus, but `bindings.py:32` declares `Binding("7", "focus_panel_7", "Panel 7", show=False)` — so 7 is actually wired. **Doc says 6, code says 7.** P3.

### 5.3 ID naming

All compose'd widgets have `id="..."`. IDs are used by TCSS selectors (e.g., `#sidebar` at `app.tcss:11`) and by `self.query_one(id)` for direct widget access. **The naming is consistent:**
- TypePanels: `panel-<ctype.name.lower()>` (e.g., `panel-slash_command`)
- Combined: `panel-combined`
- Modals: `<purpose>-<modal-suffix>` (`level-selector`, `plugin-confirm`, `delete-confirm`, `marketplace-modal`, `marketplace-confirm`, `marketplace-source-input`)
- Others: `<purpose>` (`main-pane`, `filter-input`, `app-footer`)

## 6. `on_mount` — the lifecycle entry point

`app.py:180-208`. Order of operations:

1. **Register custom themes** (`:182-183`). Iterates `CUSTOM_THEMES` (only `LAZYGIT_THEME`) and registers each with Textual.
2. **Load persistent settings** (`:184-185`). `self._settings_service.load()` reads `~/.lazyclaude/settings.json`. Applies `self.theme = self._settings.theme`.
3. **Subscribe to theme-change signal** (`:186`). `theme_changed_signal.subscribe(self, self._on_theme_changed)`. Whenever the user opens Textual's command palette and switches themes, `_on_theme_changed` (`:210-214`) persists the new theme to settings if changed.
4. **Discover customizations** (`:187`). `self._load_customizations()` runs `ConfigDiscoveryService.discover_all()` synchronously on the UI thread.
5. **Update status panel** (`:188`). Sets initial "All" filter label.
6. **Update footer** (`:189`). Initial enable/disable state for `can_*` reactives.
7. **Set window title** (`:190-192`). `f"{project_name} - LazyClaude"`. Also calls `self.console.set_window_title(self.title)` for terminal title bar update.
8. **Windows title workaround** (`:193-194`). `if os.name == "nt": os.system(f"title {self.title}")`. **Subprocess invocation for title on Windows.** P3 — fine but worth noting for the cross-platform Rust port (use `crossterm::terminal::SetTitle`).
9. **Initialize lazy services** (`:195-201`). `ConfigPathResolver` and `MarketplaceLoader` constructed now (need `_plugin_loader` from already-built discovery service).
10. **Wire marketplace modal and source input** (`:202-207`). `set_loader` / `set_suggestions` — late binding because the loader needed `on_mount` to be ready.
11. **Background settings migration** (`:208`). `_initialize_suggested_marketplaces()` runs `@work(thread=True)` to ensure the curated marketplaces are persisted to settings.

### 6.1 Order dependencies

- Themes must register **before** `self.theme = ...` because Textual validates the theme exists.
- Settings load must happen **before** theme set.
- Discovery must happen **before** panel updates because panels need customizations.
- ConfigPathResolver must be after PluginLoader because it wraps it.

**The order is not arbitrary** — there's a real DAG of init dependencies. **Port note:** monocle's `App::new` should encode this same DAG explicitly.

### 6.2 `_fatal_error` — the panic path

`app.py:125-129`:
```python
def _fatal_error(self) -> None:
    self.bell()
    traceback.print_exc()
    self.exit()
```

Bell + traceback + exit. **Not wired to any signal or exception handler.** It's a private method named like a callback but unreferenced in the codebase via grep search. **Likely dead code** — early developer scaffolding never wired into Textual's exception system. P2.

### 6.3 No `on_unmount`

There's no `on_unmount` handler. Textual's default unmount runs `App.exit()` which doesn't need explicit cleanup here because:
- `SettingsService` writes synchronously
- No file handles open
- No background workers requiring graceful shutdown (the `@work(thread=True)` workers are daemon by default)

**Port note:** monocle's port should explicitly handle shutdown — flushing pending writes, joining workers, restoring terminal state.

## 7. Message routing handlers — selection cascade

The five panel-selection handlers (`app.py:414-495`) implement a uniform pattern with subtle differences.

### 7.1 The common shape

Every selection handler:
1. Updates `_main_pane.display_path` (path-resolution for plugins via `_config_path_resolver`)
2. Updates `_main_pane.customization` (the displayed item)
3. Updates `_update_footer_actions()` (recompute `can_*` flags)
4. Calls `refresh_bindings()` (recompute action availability for Textual)

### 7.2 The shape variants

| Handler | Adds |
|---|---|
| `on_type_panel_selection_changed` (`:414-422`) | Base shape |
| `on_type_panel_drill_down` (`:424-431`) | Also: remembers `_last_focused_panel`, `_last_focused_combined=False`, focuses `_main_pane` |
| `on_combined_panel_selection_changed` (`:433-441`) | Base shape — no last-focus tracking |
| `on_combined_panel_drill_down` (`:443-450`) | Also: `_last_focused_panel = None`, `_last_focused_combined=True`, focuses `_main_pane` |
| `on_type_panel_skill_file_selected` (`:452-464`) | Sets `_main_pane.selected_file`, resolves a richer path (file_path or customization.path) |
| `on_type_panel_memory_file_ref_selected` (`:466-470`) | Delegates to `_handle_memory_file_ref_selected` |
| `on_combined_panel_memory_file_ref_selected` (`:472-476`) | Delegates to `_handle_memory_file_ref_selected` |

`_handle_memory_file_ref_selected` (`:478-495`) is the shared implementation for memory file ref selection from either panel — DRY for the two memory selection handlers.

### 7.3 Why drill-down handlers track last-focused

`_last_focused_panel` and `_last_focused_combined` are read by `NavigationMixin.action_back` to return focus to the panel that drilled into the MainPane. **This implements the "Esc returns to previous panel" UX** described in the keybinding conventions.

**Port note:** monocle needs a `focus_history: Option<FocusTarget>` field, set on drill-down, consumed on `back`.

## 8. Subtitle generation — the filter status display

`_update_subtitle` (`app.py:343-365`):

```python
if preview_mode:
    sub_title = f"Preview: {plugin.name} | Esc to exit"
else:
    parts = []
    parts.append("User Level" | "Project Level" | "Plugin Level" | "All Levels")
    if plugin_enabled_filter is True:
        parts.append("Enabled Only")
    if search_query:
        parts.append(f'Search: "{query}"')
    sub_title = " | ".join(parts)
```

**The subtitle is the user-facing state summary.** Shown in Textual's header bar.

**Port note:** monocle needs an equivalent `fn render_subtitle(&self) -> String` that the header widget reads each draw.

## 9. Themes — `themes.py` + TCSS

### 9.1 `themes.py` (`themes.py:1-30`)

```python
LAZYGIT_THEME = Theme(
    name="lazygit",
    primary="#d4d4d4",
    secondary="#808080",
    accent="#4a90d9",
    foreground="#cccccc",
    background="#1a1a1a",
    surface="#222222",
    panel="#2d2d2d",
    success="#98c379",
    warning="#e5c07b",
    error="#e06c75",
    dark=True,
    variables={
        "border": "#3a3a3a",
        "footer-background": "#1a1a1a",
        "footer-foreground": "#808080",
        "footer-key-background": "#1a1a1a",
        "footer-key-foreground": "#4a90d9",
        "footer-description-foreground": "#707070",
    },
)

CUSTOM_THEMES = [LAZYGIT_THEME]
DEFAULT_THEME = "gruvbox"
```

**Single custom theme + a default-theme constant.** `DEFAULT_THEME = "gruvbox"` is **never read** in the codebase (grep confirms). It's a vestige — actual default comes from `AppSettings.theme` which has its own default. **P3 dead constant.**

`LAZYGIT_THEME` defines 11 standard Textual color slots + 6 custom variables for footer styling. The footer (`AppFooter`) reads these variables via TCSS `$footer-foreground` etc.

**Port note:** monocle should define a `Theme` struct with the same 11 slot pattern (primary, secondary, accent, foreground, background, surface, panel, success, warning, error, border) plus per-component sub-themes (e.g., `footer: FooterTheme { background, foreground, key_foreground, ... }`). The 6 footer variables are essentially a sub-theme.

### 9.2 `styles/app.tcss` (157 lines)

Textual CSS. The salient rules:

```css
Screen { layout: grid; grid-size: 2; grid-columns: 1fr 2fr; background: $surface; }
```

**Two-column grid, sidebar:main = 1:2.** Textual's grid-columns equivalent in ratatui is `Layout::horizontal([Constraint::Ratio(1,3), Constraint::Ratio(2,3)])`.

```css
TypePanel { height: 1fr; border: solid $primary; padding: 0 1; }
TypePanel:focus { border: double $accent; }
TypePanel:focus-within { border: double $accent; }
TypePanel.empty { height: 3; min-height: 3; max-height: 3; }
```

**Focus styling via pseudo-classes.** `:focus` (widget has focus) and `:focus-within` (a child has focus) both trigger double-border. The `.empty` class shrinks an empty panel from `1fr` to fixed height 3. Custom classes set programmatically (e.g., when `customizations` is empty).

**Port note:** ratatui's `Block::border_style` and `Borders::ALL` with thick/double variants. Focus state lives in App, applied at draw time. The `:focus-within` semantic is a recursive check — for monocle, a `widget_has_focus_or_contains_focus(widget_id)` helper.

```css
FilterInput { dock: bottom; height: 3; display: none; }
FilterInput.visible { display: block; }
```

**Hide/show via class toggle.** `display: none` + `.visible` override. **All five bottom modals follow this pattern** — though only FilterInput's TCSS rule is shown explicitly; LevelSelector etc. have their own (in widget-specific TCSS or `add_class("visible")` programmatically).

```css
#help-overlay {
    layer: overlay;
    dock: right;
    width: 60;
    height: 100%;
    border: double $accent;
    background: $surface;
    padding: 1 2;
    overflow-y: auto;
}
```

**Help overlay floats over right side, 60 cells wide.** Uses the `overlay` layer (declared at `app.py:64`). `HelpMixin` mounts a Static widget with this ID.

**Port note:** ratatui's overlay pattern is to draw the overlay over the existing buffer in a final draw pass. Z-ordering is implicit (draw order). For monocle:
- Maintain a stack of overlays in `App.overlays: Vec<Overlay>`.
- Each draw cycle, draw base layout first, then iterate overlays in order, drawing each in its rect.
- The `Overlay` enum can be: `Help`, `Marketplace`, `FilterInput`, `LevelSelector`, etc.

### 9.3 TCSS-to-ratatui Style translation

| TCSS construct | ratatui equivalent |
|---|---|
| `background: $surface` | `Block::default().style(Style::default().bg(theme.surface))` |
| `border: solid $primary` | `Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.primary))` |
| `border: double $accent` | `Block::default().borders(Borders::ALL).border_type(BorderType::Double).border_style(Style::default().fg(theme.accent))` |
| `padding: 0 1` | `Padding::horizontal(1)` |
| `dock: bottom` | Layout: `Layout::vertical([Constraint::Min(0), Constraint::Length(N)])` |
| `layer: overlay` | Final draw pass over rendered buffer |
| `height: 1fr` | `Constraint::Fill(1)` (ratatui 0.27+) or `Constraint::Min(0)` |
| `height: 100%` | `Constraint::Percentage(100)` |
| `width: 60` | `Constraint::Length(60)` |
| `text-style: bold` | `Style::default().add_modifier(Modifier::BOLD)` |
| `text-wrap: nowrap; text-overflow: ellipsis` | manual: truncate string to width-1 + append `…` |
| `scrollbar-gutter: stable` | reserve column for scrollbar in layout |
| `.empty { height: 3 }` | conditional constraint: `if items.is_empty() { Constraint::Length(3) } else { Constraint::Fill(1) }` |
| `:focus { ... }` | App-state-driven: `if focused == widget_id { focused_block } else { unfocused_block }` |
| `display: none / .visible` | `if widget.is_visible() { draw } else { skip }` |

This table is the most concrete deliverable for the port — every TCSS rule in `app.tcss` maps to a known ratatui pattern.

## 10. Entry point — `__main__.py`

`__main__.py:10-47`. Argparse-based CLI:

```
lazyclaude [-V] [-d DIR] [-u USER_CONFIG]
```

| Arg | Type | Default | Purpose |
|---|---|---|---|
| `-V` / `--version` | flag | n/a | Print version and exit |
| `-d` / `--directory` | Path | None | Project directory to scan (replaces cwd) |
| `-u` / `--user-config` | Path | None | Override `~/.claude` |

If `--directory` given, `project_config_path = args.directory / ".claude"`. **The directory becomes the .claude root, not the project root** — i.e., `-d /path/to/project` reads from `/path/to/project/.claude`.

No environment-variable reading in `__main__.py`. Environment is only consulted later:
- `$EDITOR` in `app.py:574, 585` for opening files in editor.
- `os.name == "nt"` for Windows title workaround at `app.py:193`.

**Port note:** monocle CLI should mirror these args plus add `--version`, `--help`. `clap` is the canonical Rust choice.

## 11. `__init__.py` — package surface

`__init__.py:3-22`. Exports:
- `__version__` (from `_version.py` if generated, else `"0.0.0+dev"`)
- `ConfigLevel`, `Customization`, `CustomizationType` (re-export from `models.customization`)

**Public package API is intentionally minimal.** Only the three domain enums/dataclasses are exported — the `App` class and services are not. The package is consumed only as a CLI; programmatic use is not advertised.

**Port note:** monocle should preserve the same surface — `lib.rs` only re-exports the core domain types; the binary `main.rs` does the heavy lifting.

## 12. Textual → ratatui translation matrix (the master deliverable)

This is the table monocle's ratatui port template must implement.

### 12.1 App-shell concepts

| Textual concept | Source | ratatui equivalent | Port complexity |
|---|---|---|---|
| `class App` base + subclass | `app.py:53-60` | `struct App` + `impl App { fn new(), fn run() }` | Trivial |
| Multiple-inheritance mixins for actions | `app.py:53-60`, `mixins/*.py` | One `App` struct + `Action` enum + `match` in `dispatch_action`. Each mixin → one `impl App` block in its own file. | Medium — explicit dispatch is more verbose |
| `BINDINGS` class attribute | `app.py:65` + `bindings.py` | `const BINDINGS: &[(Key, Action)]` + `match key { ... }` | Easy |
| `CSS_PATH` | `app.py:63` | `Theme` struct + `Style` builders | Medium |
| `LAYERS = ["default", "overlay"]` | `app.py:64` | Vec of overlays drawn after base; manual z-ordering | Easy |
| `TITLE` / `SUB_TITLE` | `app.py:67-68` | `App.title: String` + `App.subtitle: String`; header widget reads each draw | Easy |
| `compose() -> ComposeResult` (yield widgets) | `app.py:131-178` | `fn build_layout(&self)` returns layout chunks; widgets are App fields | Easy |
| `on_mount()` lifecycle | `app.py:180-208` | `App::new(args)` constructor + post-construct `initialize()` for things needing fully-built App | Easy |
| `check_action(name, params)` action gate | `app.py:221-292` | `fn is_action_available(&self, action: Action) -> bool` | Trivial |
| `refresh_bindings()` | called from handlers | Re-derive footer state each draw; no explicit "refresh" call needed | Easy |
| `notify(msg, severity)` toast | called throughout | `Toast` enum + `App.toasts: VecDeque<Toast>`; toast widget draws each frame, auto-expires after timeout | Easy |
| `self.exit()` | `app.py:545` | `App.should_quit = true` checked by event loop | Trivial |
| `bell()` | `app.py:127` | `print!("\x07")` or `crossterm::terminal::Bell` (no API; write 0x07 to stdout) | Trivial |
| Action method auto-discovery (`action_*`) | Textual internal | Explicit `match action { Action::Quit => quit(), ... }` | Easy |
| Message handler auto-discovery (`on_*`) | Textual internal | Explicit `match event { Event::PanelSelectionChanged(c) => ... }` | Easy |
| `self.theme = "name"` | `app.py:185, 213` | `App.theme: Theme` field, set directly | Trivial |
| Theme change signal | `app.py:186` | When `App.theme` changes, also `SettingsService.save` | Trivial |
| `@work(thread=True)` background task | `app.py:217`, `mixins/marketplace.py:248` | `tokio::spawn` or `std::thread::spawn` + result channel back to event loop | Medium |
| `self.call_from_thread(callback, ...)` | not directly in app.py but used in mixins | `mpsc::Sender<Event>` from worker → main loop processes | Medium |

### 12.2 Widget-tree concepts

| Textual concept | ratatui equivalent |
|---|---|
| `Container#sidebar` with vertical layout | `Layout::vertical([...])` |
| `dock: bottom` widget | Reserved row at bottom of main layout |
| `layer: overlay` widget | Draw after base, in own rect |
| `id="foo"` for query | Field on `App` struct (no string lookups needed) |
| `add_class("visible")` / `remove_class` | Boolean field; condition in draw |
| `display: none` / `.visible` | `if widget.visible { draw }` |
| Reactive properties (`reactive(...)`) | Plain struct fields; redraw on every event-loop tick (or use dirty flag) |
| `watch_<prop>` callback | `fn set_prop(&mut self, v)` method that does work then updates |
| Widget message posting (`self.post_message(Msg)`) | Channel from widget → app: emit `Event::WidgetMsg(...)` |
| `query_one(selector)` | Direct struct field access (no DSL needed) |
| `focus()` / `has_focus` | `App.focus: FocusTarget` enum; widgets check `focus == self.id` |
| `refresh()` (force redraw) | Set `App.dirty = true`; draw on next tick |

### 12.3 TCSS concepts (recap from §9.3)

See the table in §9.3 above. The full TCSS rule set is small (~30 rules across all widget TCSSes) and reduces to about 20 distinct ratatui Style/Layout patterns.

## 13. Key registry summary (for monocle's port)

The keymap, distilled to a port-friendly enum:

```rust
enum Action {
    // Global lifecycle
    Quit,
    Refresh,
    ToggleHelp,
    // Editing
    OpenInEditor,
    OpenUserConfig,
    CopyConfigPath,
    // Customization actions
    CopyCustomization,
    MoveCustomization,
    DeleteCustomization,
    TogglePluginEnabled,
    // Filters
    FilterAll,
    FilterUser,
    FilterProject,
    FilterPlugin,
    TogglePluginEnabledFilter,
    Search,
    // Navigation
    FocusNextPanel,
    FocusPreviousPanel,
    FocusMainPane,
    FocusPanel(u8),  // 1..=7
    PrevView,
    NextView,
    Back,
    // Marketplace
    ToggleMarketplace,
    ExitPreview,
}
```

Plus widget-level actions (not in `bindings.py`):

```rust
enum WidgetAction {
    CursorUp,        // j / down
    CursorDown,      // k / up
    PageUp,          // u
    PageDown,        // d
    GoToTop,         // g
    GoToBottom,      // G
    Expand,          // l / right
    Collapse,        // h / left
    DrillDown,       // Enter
    GoBack,          // Esc (widget-level)
    ConfirmYes,      // y
    ConfirmNo,       // n
    SelectChoice(u8), // 1/2/3 in level selector
    Install,         // i (marketplace plugin)
    Uninstall,       // d (marketplace plugin, context-dependent)
}
```

**The two-tier keymap is the most important port abstraction.** Same `j` key means cursor-down inside a TypePanel but is unmapped at the App level.

## 14. App lifecycle summary

```
__main__.py:main()
  ├─ parse args
  └─ create_app(user_config, project_config)        app.py:669
       ├─ ConfigDiscoveryService(...)                app.py:683
       └─ LazyClaude(discovery_service=...)
            └─ __init__()                            app.py:80-123
                 ├─ super().__init__()  # Textual App init
                 ├─ allocate 38 state fields
                 ├─ self._discovery_service = ...
                 ├─ self._filter_service = FilterService()
                 ├─ self._settings_service = SettingsService()
                 └─ self._settings = AppSettings()  # placeholder, replaced on mount
  └─ app.run()                                       Textual App entry
       ├─ Textual constructs UI
       ├─ compose()                                  app.py:131-178
       │    └─ yields 12 widgets in tree order
       ├─ on_mount()                                 app.py:180-208
       │    ├─ register custom themes
       │    ├─ load settings; set self.theme
       │    ├─ subscribe theme_changed_signal
       │    ├─ _load_customizations()                # synchronous, blocks UI startup briefly
       │    ├─ _update_status_panel()
       │    ├─ _update_footer_actions()
       │    ├─ set window title
       │    ├─ construct ConfigPathResolver + MarketplaceLoader
       │    ├─ wire loader to marketplace_modal + source_input
       │    └─ _initialize_suggested_marketplaces()  # @work(thread=True)
       │
       ├─ event loop:
       │    ├─ key event → BINDINGS lookup → check_action → action_*
       │    ├─ widget event → on_<MessageType>
       │    ├─ worker callback → call_from_thread → UI update
       │    └─ theme change → _on_theme_changed → settings persist
       │
       └─ on user quit:
            └─ self.exit()                            app.py:545
                  └─ Textual tears down terminal
```

**Two distinct startup phases:**
1. `__init__` — pure state allocation, no I/O.
2. `on_mount` — full I/O + service wiring.

This separation matters because Textual's testing harness can construct an `App` and inspect state before mount.

## 15. Delta Summary

- **New items added:**
  - The full 29-binding keymap table with action source attribution
  - MRO-based action dispatch mechanism explanation
  - `check_action` 5-layer precedence model
  - The dead constant `DEFAULT_THEME = "gruvbox"` (never read)
  - The dead method `_fatal_error` (never wired)
  - The two parallel mechanisms for action availability (`check_action` vs `_update_footer_actions`) — duplicate logic
  - The `show=False` flag is dead config (AppFooter doesn't read `BINDINGS`)
  - Two-priority-binding mechanism for Esc cascade (priority `exit_preview` + non-priority `back`)
  - 38-attribute init state allocation, clustered by purpose
  - 11-step `on_mount` order DAG with dependency reasoning
  - Five panel-selection handler variants with their distinct payloads
  - TCSS → ratatui Style translation matrix (20 rule patterns)
  - Full Action enum for the Rust port
  - Two-tier keymap (App-level vs widget-level)
- **Existing items refined:**
  - The composition tree (vs. CLAUDE.md's diagram, which omits MarketplaceSourceInput and panel-7)
  - Doc says 0-6 panel focus but bindings declare 0-7 — discrepancy
  - Theme system: 1 custom + Textual builtins, with `DEFAULT_THEME = "gruvbox"` unused
- **Remaining gaps:**
  - **Mixin internals** — what NavigationMixin.action_focus_panel_N actually does (focus + active_type setting). This is in mixin code, not app.py. Out of scope here but adjacent.
  - **Widget-level BINDINGS** declared in each widget file (TypePanel, CombinedPanel, MarketplaceModal). Mostly covered in widgets-r1.md but worth a cross-reference round.
  - **Help overlay actual content** — what HelpMixin renders inside `#help-overlay`. Currently just a text dump; round 2 candidate.
  - **The `theme_changed_signal` API** — what Textual emits, what the callback receives. Confirmed signature `(theme: Theme)` from `_on_theme_changed:210`, but the signal source API needs reading from Textual docs to be sure it's safe to mirror in Rust.

## 16. Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: The keymap table with action attribution, the two-tier keymap separation, the `check_action` 5-layer precedence, the two parallel action-availability mechanisms, and the TCSS → ratatui translation matrix are all new findings that change the port plan. The translation matrix is the single most valuable artifact for the ratatui rewrite — it makes the port mechanical, not architectural. **Removing this round's findings would force the port team to re-derive these mappings from scratch.**

## 17. Convergence Declaration

Another round needed — substantive gaps remain:
- Mixin internals (NavigationMixin, FilterMixin, MarketplaceMixin, CustomizationActionsMixin, HelpMixin) — they own all the actual `action_*` implementations. The mixin file content is the next layer to deepen.
- The complete `on_*` handler set in mixins (level_selector confirmations, delete confirm flow, plugin confirm flow, marketplace_modal lifecycle).
- The HelpMixin's help-text content (the canonical "what keys do what" inline).

Round 2 should focus on the mixin layer's `action_*` and `on_*` implementations.

## 18. State Checkpoint

```yaml
pass: B
subpass: app-keybindings
round: 1
status: complete
timestamp: 2026-05-11T18:00:00Z
novelty: SUBSTANTIVE
files_analyzed:
  - src/lazyclaude/app.py
  - src/lazyclaude/bindings.py
  - src/lazyclaude/keybindings/__init__.py
  - src/lazyclaude/__main__.py
  - src/lazyclaude/__init__.py
  - src/lazyclaude/themes.py
  - src/lazyclaude/styles/app.tcss
  - src/lazyclaude/mixins/__init__.py
  - src/lazyclaude/mixins/CLAUDE.md
```
