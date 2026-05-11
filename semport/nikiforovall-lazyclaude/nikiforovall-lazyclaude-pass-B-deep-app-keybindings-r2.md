# Phase B Deepening: App / Bindings / Composition Layer — Round 2

Goal: deepen the **mixin internals** that Round 1 flagged as the remaining gap. The action methods themselves (now read directly) reveal a coherent dispatch pattern, the focus-restore protocol, the subprocess-shell-out flow, and the help overlay content.

## 1. Mixin file map (sizes + responsibilities)

| Mixin file | LOC | `action_*` methods | `on_<Msg>` handlers | Helpers |
|---|---|---|---|---|
| `navigation.py` | 133 | 11 (focus_next/prev_panel, focus_main_pane, focus_panel_1..7, prev_view, next_view, back) | 0 | `_get_focused_panel_index`, `_focus_panel` |
| `filtering.py` | 108 | 6 (filter_all/user/project/plugin, toggle_plugin_enabled_filter, search) | 0 | `_update_status_filter` |
| `help.py` | 74 | 1 (toggle_help) | 0 | `_show_help`, `_hide_help` |
| `customization_actions.py` | 280 | 4 (copy/move/delete_customization, toggle_plugin_enabled) | 6 (level_selector ×2, plugin_confirm ×2, delete_confirm ×2) | `_get_available_target_levels`, `_handle_copy_or_move`, `_delete_customization` |
| `marketplace.py` | 431 | 2 (toggle_marketplace, exit_preview) | 14 (marketplace_modal ×11, marketplace_confirm ×2, marketplace_source_input ×2) | `_enter/_exit_plugin_preview`, `_resolve_plugin_scope`, `_run_plugin_command` (worker), `_on_plugin_command_success/error` |
| **Total** | **1026** | **24** | **20** | many |

The 24 `action_*` methods + the 8 `action_*` in `app.py` (quit, refresh, open_in_editor, open_user_config, copy_config_path, plus `_fatal_error` not an action) = **~30 actions**, well aligned with the 29 BINDINGS in `bindings.py`. (Quit is defined twice — once in `App` base, once overridden in `app.py:543`.)

## 2. NavigationMixin (the focus state machine)

### 2.1 Panel-index semantics

`_get_focused_panel_index` (`navigation.py:51-58`):

```python
for i, panel in enumerate(self._panels):
    if panel.has_focus:
        return i              # 0, 1, 2 → TypePanels
if self._combined_panel and self._combined_panel.has_focus:
    return len(self._panels)  # 3 → CombinedPanel
return None
```

**The CombinedPanel is index 3 in this scheme**, even though it has 4 internal tabs. Tab/Shift+Tab navigation treats it as a single panel; tab cycling within is delegated to the panel itself (covered in widgets-r1).

### 2.2 Tab cycling

`action_focus_next_panel` (`navigation.py:23-35`):
- If nothing focused: → panels[0] (slash commands)
- If on TypePanel N (N < 2): → panels[N+1]
- If on TypePanel 2 (last TypePanel): → switch CombinedPanel to MEMORY_FILE tab + focus it
- If on CombinedPanel: → wraparound to panels[0]

`action_focus_previous_panel` (`navigation.py:37-49`):
- If nothing or first panel focused: → switch CombinedPanel to HOOK tab + focus it (`HOOK` is COMBINED_TYPES[2] — not the last). **This asymmetry surprised me on first reading** — going backwards lands on Hook, not LSP_SERVER. Possibly a bug — should be COMBINED_TYPES[-1].
- If on CombinedPanel: → panels[-1]
- Else (TypePanel N, N>0): → panels[N-1]

**Asymmetric wraparound: forward lands on first combined tab (MEMORY_FILE); backward lands on third combined tab (HOOK), not last.** P2 bug candidate — author may have intended `LSP_SERVER` but coded `HOOK`. Worth verifying via behavior test but the asymmetry is real in source.

### 2.3 Numeric direct-focus

`action_focus_panel_1..3` (`navigation.py:65-75`):
```python
def action_focus_panel_1(self) -> None: self._focus_panel(0)
```
Maps 1-indexed binding to 0-indexed panel list. Simple.

`action_focus_panel_4..7` (`navigation.py:77-99`):
```python
def action_focus_panel_4(self) -> None:
    if self._combined_panel:
        self._combined_panel.switch_to_type(CustomizationType.MEMORY_FILE)
        self._combined_panel.focus()
```
Each direct-focus key calls `switch_to_type` on the combined panel with the specific `CustomizationType`. **Hardcoded 1-to-1 mapping** between numeric key and customization type:

| Key | CustomizationType |
|---|---|
| 4 | MEMORY_FILE |
| 5 | MCP |
| 6 | HOOK |
| 7 | LSP_SERVER |

**Port note:** monocle should encode this as a const table: `const PANEL_KEY_TO_TYPE: &[(u8, CustomizationType)]`. Avoid four separate methods.

### 2.4 View switching (`[` / `]`)

`action_prev_view` (`navigation.py:106-111`):
- If CombinedPanel focused: → `combined_panel.action_prev_tab()` (cycles tabs)
- Else (any other widget focused, including MainPane): → `main_pane.action_prev_view()` (toggles content/metadata)

**Same key, two different behaviors based on focus.** This is the **context-sensitive action delegation pattern** — App-level binding dispatches to the focused widget's contextual handler.

### 2.5 `action_back` — the Esc behavior

`navigation.py:120-132`:
```python
async def action_back(self) -> None:
    if self._plugin_preview_mode:
        self._exit_plugin_preview()
        return

    if self._main_pane and self._main_pane.has_focus:
        if self._last_focused_combined and self._combined_panel:
            self._combined_panel.focus()
        elif self._last_focused_panel:
            self._last_focused_panel.focus()
        elif self._panels:
            self._panels[0].focus()
```

**Async signature** (`async def`) but never awaits anything. Likely because `action_exit_preview` was once async and this kept the signature. **P3 vestige** — could be sync.

The four-branch back logic:
1. In preview mode → exit preview.
2. Main pane focused with combined-was-last → focus combined panel.
3. Main pane focused with type-panel-was-last → focus that type panel.
4. Main pane focused with no last-focus → focus panels[0].
5. Main pane NOT focused → no-op (Esc consumed by widget's own binding).

**Port note:** the Action::Back in monocle has this same five-arm logic. The `priority=True` `exit_preview` binding pre-empts when in preview; otherwise the `back` binding handles non-preview Esc.

## 3. FilterMixin (six actions, two reactive state vars)

### 3.1 The four level-filter actions

`action_filter_all/user/project/plugin` (`filtering.py:28-66`) are four near-identical methods. Each:
1. Sets `self._level_filter` to the corresponding `ConfigLevel` (or `None` for all).
2. Resets `_last_focused_panel = None`.
3. Clears `_main_pane.customization = None`.
4. Calls `_update_panels()`, `_update_subtitle()`, `_update_status_filter(label)`.

**Could be parameterized into one method.** A `def _set_level_filter(self, level: ConfigLevel | None, label: str)` would deduplicate 38 lines into one method + four 2-line delegates. **P3 DRY violation.** Author chose explicitness over compactness — probably for grep-ability.

**Port note:** monocle should parameterize. One `Action::Filter(ConfigLevel option)` variant, one match arm.

### 3.2 `action_toggle_plugin_enabled_filter` — the tri-state filter

`filtering.py:68-84`:
```python
if self._plugin_enabled_filter is True:
    self._plugin_enabled_filter = None
else:
    self._plugin_enabled_filter = True
```

Toggles between `True` (enabled-only) and `None` (show both). **Never `False`** — there's no "disabled-only" state. The `_plugin_enabled_filter: bool | None` type from `__init__` allows three values, but only two are used. **P3 type imprecision** — should be `Literal[True] | None` or a custom enum.

The mixin sets `status_panel.disabled_filter_active` and `app_footer.disabled_filter_active` based on `_plugin_enabled_filter is None`. **"Disabled filter active" really means "showing disabled too"**, not "hiding disabled". The name is confusing but the semantics are clear from context.

### 3.3 `action_search`

`filtering.py:86-89`. One line: `self._filter_input.show()`. The actual search-state changes come via `on_filter_input_filter_changed` in `app.py` (Round 1), not in this mixin.

### 3.4 `_update_status_filter` — the path display

`filtering.py:91-107`. Computes `config_path` for status panel based on level:

| Level | Path |
|---|---|
| User | `~/.claude` (literal string) |
| Project | `str(discovery_service.project_config_path)` (absolute resolved path) |
| Plugin | `~/.claude/plugins` (literal string) |
| All | `discovery_service.project_root.name` (just project name) |

**Inconsistent representations.** User/Plugin use `~/` shorthand (not the resolved home). Project uses absolute. All uses just the project name. **P2 polish issue.** Monocle should normalize — always show the path with `~` shorthand or always absolute.

## 4. HelpMixin (overlay mount/unmount + help text)

### 4.1 The toggle pattern

`action_toggle_help` (`help.py:13-18`) flips between `_show_help` and `_hide_help`. State tracked via `_help_visible: bool`.

`_show_help` (`help.py:20-64`):
1. Builds `help_content` string with Rich-formatted Textual markup.
2. Checks `if not self.query("#help-overlay")` — guard against double-mount.
3. Creates a Textual `Static` widget with the content, id="help-overlay".
4. `self.mount(help_widget)` — adds to app's widget tree dynamically.
5. Sets `_help_visible = True`.

`_hide_help` (`help.py:66-73`):
1. `query_one("#help-overlay")` — find widget.
2. `widget.remove()` — remove from tree.
3. `_help_visible = False`.
4. Wraps in `try/except Exception: pass` — silently swallows lookup failures.

**Two distinct patterns for modals:**
- Pre-composed-and-hidden (FilterInput, LevelSelector, etc.) — mounted at startup, shown/hidden via `add_class("visible")`.
- Mounted-on-demand (HelpMixin) — only constructed when first shown, removed when hidden.

**Help is the only "lazy mount" widget.** P3 inconsistency. Probably because `Static(content_string)` is cheap and `_help_visible` flag was simpler than threading state into a class.

### 4.2 The help text content (the canonical keymap doc)

`help.py:22-59`. Rich-formatted text. Notable observations:

- Listed sections: **Navigation, Filtering, Views, Actions**.
- Help text claims `1-3` focus panel and `4-6` focus combined panel tab. **Doc says 4-6, code says 4-7.** Same docs-vs-code gap as the README. The 7-key (LSP_SERVER) is wired but not documented in help. P3.
- Help text claims `j/k or Up/Down` — confirms Up/Down arrow keys are also bound. **Where?** Not in `bindings.py` (only TypePanel widget bindings have them — covered in widgets-r1). So this is widget-level binding being advertised at app-level help.
- Help text lists `Ctrl+u Open user config` — matches `bindings.py:33`.
- **Footer hint `[bold]?[/]` and `Esc` to close** — these are pseudo-bindings: `?` retoggle is real; `Esc` close is not in `bindings.py` for help specifically. Help close on Esc would need `_help_visible`-aware handling — **but I don't see it wired**. Likely a docs lie; Esc actually goes to `action_back` which doesn't dismiss help. **P2 doc-vs-code mismatch worth verifying.**

### 4.3 Action-binding mismatch

Help text says "`Esc Go back`" but doesn't reference the priority-based exit_preview binding. **The help doc is incomplete for preview mode** — a user in preview mode pressing `?` sees normal help, not preview-specific help. P3 polish.

## 5. CustomizationActionsMixin (the CRUD orchestration)

### 5.1 The four-stage CRUD lifecycle

Each of copy/move/delete/toggle follows the same five-stage flow:

```
1. action_<verb>_customization (entry guard checks)
     ├─ has selection?
     ├─ is copyable type?
     ├─ is not plugin-level (for move/delete)?
     └─ has available targets?
2. show modal (LevelSelector / DeleteConfirm / PluginConfirm)
3. user input → modal posts <Confirmed|Cancelled> message
4. on_<modal>_<msg> handler in this mixin
     ├─ on confirm: call writer + handle result
     └─ on cancel: just _restore_focus_after_selector
5. _restore_focus_after_selector (always)
```

### 5.2 Entry guard pattern

Each entry method (`customization_actions.py:37-136`) duplicates these guards:
```python
if not self._main_pane or not self._main_pane.customization:
    return
customization = self._main_pane.customization
if customization.type not in self._COPYABLE_TYPES:
    self._show_status_error(f"Cannot {verb} {customization.type_label} customizations")
    return
```

**Three entry methods (copy/move/delete) repeat this pattern.** P3 DRY violation. Plus the `_panel_before_selector` / `_combined_before_selector` save protocol is also duplicated four times.

**Port note:** monocle should factor this into a single `prepare_crud(verb) -> Result<Customization, AbortReason>` helper.

### 5.3 The `_handle_copy_or_move` dispatch

`customization_actions.py:165-212`. Type-dispatched writer call:
- `MCP` → `writer.write_mcp_customization(c, level, project_config_path)`
- `HOOK` → `writer.write_hook_customization(c, level, user_config, project_config)`
- else → `writer.write_customization(c, level, user_config, project_config)`

After copy succeeds, if move-operation: calls `_delete_customization` to remove source.

**Critical bug (already flagged in Pass 2 architecture): no rollback on copy-success-then-delete-failure.** `customization_actions.py:200-208`:
```python
if operation == "move":
    delete_success, delete_msg = self._delete_customization(customization, writer)
    if not delete_success:
        self._show_status_error(f"Copied but failed to delete source: {delete_msg}")
        return
```

The user is informed but the source is left behind. The copy at the target is NOT rolled back. **Move is not atomic.** P1 confirmed.

**Port note:** monocle's port should either implement transactional copy-then-delete or expose the "non-atomic move" guarantee explicitly. The simplest safe approach: never move; always copy and ask user to confirm delete separately.

### 5.4 `_get_available_target_levels` — the destination filter

`customization_actions.py:138-150`:
```python
if customization.type in self._PROJECT_LOCAL_TYPES:  # HOOK, MCP
    all_levels = [USER, PROJECT, PROJECT_LOCAL]
else:
    all_levels = [USER, PROJECT]
return [level for level in all_levels if level != customization.level]
```

**Exclude current level + maybe-include PROJECT_LOCAL.** PROJECT_LOCAL is only available for HOOK and MCP because those are the only types that write to `settings.local.json`. Other types' PROJECT_LOCAL files would just be more files in `./.claude/local/` which is checked into `.gitignore` by default — supported semantically but not implemented for copying.

**Port note:** monocle should preserve this asymmetry. Encode as a per-type capability matrix.

### 5.5 Six `on_*` handlers

| Handler | Confirms or Cancels | Effect |
|---|---|---|
| `on_level_selector_level_selected` (`:214-223`) | confirms | calls `_handle_copy_or_move`; clears `_pending_customization`; restores focus |
| `on_level_selector_selection_cancelled` (`:225-231`) | cancels | clears `_pending_customization`; restores focus |
| `on_plugin_confirm_plugin_confirmed` (`:233-250`) | confirms | calls `writer.toggle_plugin_enabled`; notifies; refreshes; restores focus |
| `on_plugin_confirm_confirmation_cancelled` (`:252-257`) | cancels | restores focus |
| `on_delete_confirm_delete_confirmed` (`:259-272`) | confirms | calls `_delete_customization`; notifies; refreshes; restores focus |
| `on_delete_confirm_delete_cancelled` (`:274-279`) | cancels | restores focus |

**Every handler ends with `_restore_focus_after_selector()`.** This is the central UX guarantee: after any modal interaction, focus returns to where the user was. The mechanism (`app.py:647-658`):

```python
def _restore_focus_after_selector(self) -> None:
    if self._combined_before_selector and self._combined_panel:
        self._combined_panel.focus()
        self._combined_before_selector = False
        self._panel_before_selector = None
    elif self._panel_before_selector:
        self._panel_before_selector.focus()
        self._panel_before_selector = None
        self._combined_before_selector = False
    elif self._panels:
        self._panels[0].focus()
```

Three branches: combined-was-focused → focus combined; type-panel-was-focused → focus it; nothing tracked → fall back to panels[0]. State is **mutually exclusive** between `_panel_before_selector` and `_combined_before_selector`. Both are reset after restore.

**Port note:** monocle's `App.previous_focus: Option<FocusTarget>` field replaces these two state vars cleanly. Set on modal open; consumed on modal close.

### 5.6 The toggle_plugin_enabled flow

`action_toggle_plugin_enabled` (`:117-136`):
1. Guard: has customization with `plugin_info`.
2. Save focus state.
3. `plugin_confirm.show(plugin_info, customizations=self._customizations)`.

The plugin_info contains the plugin ID; the full customizations list is passed so the confirm modal can show which customizations belong to that plugin (so user sees what they're affecting).

**On confirm (`:233-250`):** writer.toggle_plugin_enabled mutates `settings.json` / `settings.local.json` directly (not via `claude` CLI — unlike install/uninstall which shell out). This is the **only direct settings-file mutation in the marketplace flow**. Pass 2 architecture noted this.

## 6. MarketplaceMixin (the largest, most complex mixin)

### 6.1 The 11 marketplace_modal handlers

| Handler | Triggered by | Effect |
|---|---|---|
| `on_marketplace_modal_plugin_preview` (`:158-162`) | press Enter on plugin in tree | `_enter_plugin_preview(plugin)` |
| `on_marketplace_modal_plugin_toggled` (`:176-201`) | press `i` on installed plugin (enable/disable) | shell out `claude plugin enable/disable` |
| `on_marketplace_modal_plugin_install_with_scope` (`:203-220`) | press 1/2/3 after `i` on uninstalled plugin | shell out `claude plugin install --scope` |
| `on_marketplace_modal_plugin_uninstall` (`:222-246`) | press `d` on installed plugin | shell out `claude plugin uninstall` |
| `on_marketplace_modal_open_plugin_folder` (`:282-293`) | press `e` on installed plugin | subprocess `$EDITOR <install_path>` |
| `on_marketplace_modal_open_plugin_source` (`:295-319`) | press `o` on plugin | open in file explorer (directory) or browser (GitHub) |
| `on_marketplace_modal_open_marketplace_source` (`:321-339`) | press `o` on marketplace | open in file explorer or browser |
| `on_marketplace_modal_marketplace_update` (`:341-348`) | press `U` on marketplace | shell out `claude plugin marketplace update <name>` |
| `on_marketplace_modal_plugin_update` (`:350-365`) | press `u` on plugin | shell out `claude plugin update --scope` |
| `on_marketplace_modal_modal_closed` (`:367-373`) | Esc or `M` to close | `_restore_focus_after_selector` |
| `on_marketplace_modal_marketplace_remove` (`:375-380`) | press `D` on marketplace | show MarketplaceConfirm modal |
| `on_marketplace_modal_marketplace_add_request` (`:382-388`) | press `a` in marketplace modal | show MarketplaceSourceInput |

12 handlers (I miscounted to 11 — there's also `marketplace_add_request`). The full set covers every plugin/marketplace lifecycle event.

### 6.2 The subprocess shell-out pattern

`_run_plugin_command` (`:248-267`) is the **central subprocess invocation point**. Six handlers funnel through it:
- plugin_toggled → `claude plugin enable/disable`
- plugin_install_with_scope → `claude plugin install --scope`
- plugin_uninstall → `claude plugin uninstall --scope`
- marketplace_update → `claude plugin marketplace update`
- plugin_update → `claude plugin update --scope`
- marketplace_confirm_remove_confirmed → `claude plugin marketplace remove`
- marketplace_source_input_source_submitted → `claude plugin marketplace add`

The function `@work(thread=True)` decorator runs it on a Textual worker thread. Salient details:

```python
subprocess.run(
    cmd,
    capture_output=True,
    check=True,
    shell=True,                       # <-- shell=True!
    encoding="utf-8",
    errors="replace",
    cwd=cwd,
)
```

**Two notable choices:**
1. **`shell=True` with a list arg.** When `shell=True` and `cmd` is a list, behavior varies by platform. On POSIX, only the first arg is the shell command and the rest are positional `$0`, `$1`, etc. On Windows, the list is joined. **This is a known footgun** — `subprocess.run(["claude", "plugin", "install", "id"], shell=True)` on Linux actually runs `claude` with `plugin install id` as positional shell args, not as args to claude.

   Wait — does this actually work? Let me re-read... `["claude", "plugin", "install", plugin.full_plugin_id, "--scope", scope]` with `shell=True`. On Linux, `subprocess` calls `/bin/sh -c claude plugin install id --scope user`. That's actually fine because `sh -c` parses the joined string as a command line. Python's subprocess docs say "If args is a sequence, the first item specifies the command string, and any additional items will be treated as additional arguments to the shell itself" — so the second arg becomes `$0`, the third becomes `$1`. **The args are NOT passed to claude.** Only the first arg `claude` runs (with empty args). **This is a latent bug** — except that it would manifest immediately as `claude` printing usage. Either tests don't actually run plugin install, or there's something compensating.

   Re-reading more carefully... actually `subprocess.run(cmd, shell=True)` with `cmd` as a list — on POSIX this would be `sh -c claude plugin install id --scope user`. The shell sees `claude` as the command and `plugin install id --scope user` as positional. So `claude` runs with NO arguments. **This is broken on POSIX.** Likely the install fails silently or just prints `claude --help`. Either the test harness mocks subprocess or this code path isn't reached in tests. **P0 bug candidate** — verify with running the actual code.

   Actually wait — Python's subprocess docs (`subprocess.Popen.__init__`): "If args is a sequence, the first item specifies the command string, and any additional items will be treated as additional arguments to the shell itself." So `cmd[0]` becomes the script, `cmd[1:]` become `$0 $1 $2`. This means the command IS just `claude`, and on POSIX this is broken.

   **HOWEVER** — on Windows, `shell=True` means `cmd.exe /c` and the list is joined into a single string. So on Windows it works correctly.

   **The shell=True is for Windows compatibility but breaks POSIX.** P0 if confirmed — `claude plugin install` would never receive its arguments on Linux/Mac. **TODO for downstream**: run a test on POSIX to confirm. Or read the test files to see if `_run_plugin_command` is exercised.

2. **`check=True`** — non-zero exit raises `CalledProcessError`. The except clause grabs `.stderr` for the error notify.

3. **Two exception types caught:** `CalledProcessError` (claude returned error) and `FileNotFoundError` (claude binary not installed). **No timeout, no cancellation, no interrupt.** A hung `claude plugin install` would block the worker indefinitely. P1.

### 6.3 The plugin preview flow

`_enter_plugin_preview(plugin)` (`marketplace.py:70-132`):

1. Get plugin source dir via marketplace_loader.
2. Construct a fake `PluginInfo` with `version="preview"`.
3. **Discover customizations from that directory** using `discovery_service.discover_from_directory(...)`. This is a separate code path from `discover_all` — it scans a single plugin's source tree on demand.
4. Set `_plugin_preview_mode = True`, `_previewing_plugin = plugin`, `_plugin_customizations = ...`.
5. Hide marketplace modal with `preserve_state=True` (keeps tree expansion state for restore).
6. Refresh panels, subtitle, footer, bindings.
7. Update status_panel to show "Preview: <name> (<version>)".
8. **Load and display README.md** if it exists, as a fake Customization (type=MEMORY_FILE, level=PLUGIN). Catches OSError silently.
9. Switch CombinedPanel to MCP tab (default tab when entering preview).

`_exit_plugin_preview` (`:134-152`):
- Reverses everything: mode=False, plugin=None, customizations=[].
- Clears search query.
- Clears filter input.
- Refreshes panels/subtitle/status/footer/bindings.
- Clears main pane customization.
- Re-shows marketplace modal with `preserve_state=True` (restore tree state).

**Port note:** monocle's preview mode is a substantial state machine. The `_plugin_customizations` list is a **second customization corpus** that the panels render instead of the main list when in preview. The `update_panels` method (`app.py:306-320`) branches on `_plugin_preview_mode` to choose corpus.

### 6.4 `_resolve_plugin_scope` — the user/project scope inference

`marketplace.py:164-174`:
```python
view_scope = self._marketplace_modal.scope_view if self._marketplace_modal else "user"
if view_scope == "project":
    return next(
        (s for s in plugin.installed_scopes if s in ("project", "local")),
        "project",
    )
return "user" if "user" in plugin.installed_scopes else view_scope
```

**Three-step inference:**
1. Get user-facing scope view (user | project).
2. If "project" view: find first project-flavored scope in installed_scopes (project or local), default to "project".
3. Else: if installed at "user" scope, use "user"; else fall back to view_scope.

This is the mapping from "what scope view is the user looking at" to "what --scope flag does claude need" for the operation. Subtle but consistent.

**Port note:** monocle should reproduce — the user's mental scope and the CLI scope can differ when a plugin is installed at multiple scopes.

### 6.5 `open_plugin_folder` uses `$EDITOR` (not file explorer)

`marketplace.py:282-293`:
```python
editor = os.environ.get("EDITOR", "vi")
subprocess.Popen([editor, str(plugin.install_path)], shell=True)
```

**Opens in `$EDITOR`, not the system file explorer.** Name suggests file explorer but implementation is editor. **P2 mis-named** — should be `action_open_plugin_in_editor`.

Compare with `open_plugin_source` (`:295-319`) which uses `open_in_file_explorer` (POSIX `xdg-open` / macOS `open` / Windows `explorer`). Two different "open" semantics in same mixin.

## 7. The complete action → mixin map

For monocle's `match action`, here's every action with its source:

| Action | Mixin / file:line | Notes |
|---|---|---|
| `quit` | `app.py:543` (override) + `App` base | Trivial exit |
| `refresh` | `app.py:547-550` | Calls `discovery_service.refresh()` + `_update_panels` |
| `open_in_editor` | `app.py:552-576` | Type-dispatched: skill → parent dir; else → path |
| `open_user_config` | `app.py:589-600` | Opens `~/.claude` + `~/.claude.json` if exist |
| `copy_config_path` | `app.py:602-623` | `pyperclip.copy(resolved_path)` |
| `toggle_help` | `help.py:13-18` | Show/hide help overlay |
| `focus_next_panel` | `navigation.py:23-35` | Tab |
| `focus_previous_panel` | `navigation.py:37-49` | Shift+Tab |
| `focus_main_pane` | `navigation.py:101-104` | `0` |
| `focus_panel_1..3` | `navigation.py:65-75` | `1`, `2`, `3` |
| `focus_panel_4..7` | `navigation.py:77-99` | `4`, `5`, `6`, `7` (combined tabs) |
| `prev_view` | `navigation.py:106-111` | `[` — context-sensitive |
| `next_view` | `navigation.py:113-118` | `]` — context-sensitive |
| `back` | `navigation.py:120-132` | Esc — non-priority |
| `filter_all/user/project/plugin` | `filtering.py:28-66` | `a`, `u`, `p`, `P` |
| `toggle_plugin_enabled_filter` | `filtering.py:68-84` | `D` |
| `search` | `filtering.py:86-89` | `/` |
| `copy_customization` | `customization_actions.py:37-61` | `c` |
| `move_customization` | `customization_actions.py:63-91` | `m` |
| `delete_customization` | `customization_actions.py:93-115` | `d` |
| `toggle_plugin_enabled` | `customization_actions.py:117-136` | `t` |
| `toggle_marketplace` | `marketplace.py:54-68` | `M` (priority) |
| `exit_preview` | `marketplace.py:154-156` | Esc (priority, preview-mode only) |

**24 action methods total in mixins + 5 in `app.py` = 29 actions.** Exactly matches the 29 BINDINGS (one binding is Esc duplicate for the priority cascade, accounting for both `exit_preview` and `back` actions both bound to Esc).

## 8. Updated translation matrix additions

| Textual / Python concept | ratatui equivalent | Concern |
|---|---|---|
| `@work(thread=True)` decorator | `tokio::spawn(async move { ... })` with `mpsc::UnboundedSender<Event>` back to main loop | Concurrency |
| `self.call_from_thread(cb, arg)` | `tx.send(Event::WorkerResult(arg))` then handle in event loop | Cross-thread |
| `subprocess.run(cmd, capture_output=True, check=True)` | `std::process::Command::new(cmd[0]).args(&cmd[1..]).output()` — check `status.success()` | Process |
| `subprocess.Popen(cmd, shell=True)` | `std::process::Command::new(cmd[0]).args(&cmd[1..]).spawn()` — **no shell, pass args directly** | Process |
| `os.environ.get("EDITOR", "vi")` | `std::env::var("EDITOR").unwrap_or_else(\|_\| "vi".to_string())` | Env |
| `pyperclip.copy(s)` | `arboard::Clipboard::new()?.set_text(s)` (Rust clipboard crate) | I/O |
| Async `async def action_back` that doesn't await | sync `fn back(&mut self)` in Rust (async signature in Python is vestigial here) | Style |
| `widget.mount(child_widget)` (dynamic mount) | Push to overlay stack or set field to Some(widget_state) | Layout |
| `widget.remove()` | Pop from overlay stack or set field to None | Layout |
| `query("#id")` (CSS selector lookup) | Direct field access on App struct | Lookup |
| `query_one("#id")` raising on missing | `app.help_overlay.as_ref().expect(...)` or pattern match | Lookup |
| `try: ...; except Exception: pass` | Explicit `Result` handling; never silent | Error |

## 9. P0/P1 findings (new from this round)

### 9.1 P0 candidate: `shell=True` + list args on POSIX

`marketplace.py:253-261` — passing a list to `subprocess.run(cmd, shell=True)` on POSIX is broken. Per Python docs, only `cmd[0]` is the shell command; the rest become positional shell args, not args to claude. So `claude plugin install <id> --scope user` becomes `sh -c claude` with positional `plugin install <id> --scope user` that the shell silently ignores. **claude is invoked with zero args.**

**Verification needed.** If this is genuinely broken on POSIX, the marketplace operations only work on Windows. Worth running the actual code or checking integration tests to confirm.

### 9.2 P1: NavigationMixin `action_focus_previous_panel` wraps to HOOK, not LSP_SERVER

`navigation.py:42` — Shift+Tab from panel 1 lands on Hook (COMBINED_TYPES[2]), not LSP_SERVER (COMBINED_TYPES[3], the last). Asymmetric with `action_focus_next_panel` which wraps to MEMORY_FILE (COMBINED_TYPES[0], the first). **Likely intent: last tab, not third.** Bug.

### 9.3 P1: Move operation is not atomic

`customization_actions.py:200-208` — copy succeeds, delete fails → source remains, target created. No rollback. Already flagged in Pass 2; reconfirmed here.

### 9.4 P1: No timeout/cancellation on claude CLI subprocess calls

`marketplace.py:248-267` — `subprocess.run(...)` with no timeout. A hung claude CLI blocks the worker indefinitely. The user can quit the TUI but the worker thread persists.

### 9.5 P2: `_fatal_error` is dead code

`app.py:125-129` — defined but never invoked. Likely scaffolding.

### 9.6 P2: Two parallel action-availability mechanisms

`check_action` (`app.py:221-292`) and `_update_footer_actions` (`app.py:367-410`) each encode the same business logic. Must be updated together. **Should be unified** — derive footer from `check_action`.

### 9.7 P2: `open_plugin_folder` opens in $EDITOR not file explorer

`marketplace.py:292` — name mismatch with behavior.

### 9.8 P2: Help text overlay can't be dismissed by Esc

`help.py` overlay text claims "? or Esc to close" but Esc → `action_back` doesn't check for visible help overlay. Likely cosmetic — `?` works.

### 9.9 P3: Async signatures without await

`action_back` and `action_quit` are `async def` but never use await. Vestigial.

### 9.10 P3: DRY violations in FilterMixin and CustomizationActionsMixin

Four near-identical filter methods; three near-identical CRUD entry methods. Compactness possible.

### 9.11 P3: Docs/code panel-key count mismatch

README/CLAUDE.md say 0-6; bindings.py and navigation.py say 0-7 (LSP_SERVER added).

## 10. Delta Summary

- **New items added:**
  - Full mixin file map with LOC, action and handler counts
  - NavigationMixin focus state machine with the HOOK-vs-LSP_SERVER asymmetric wraparound bug (P1)
  - FilterMixin's DRY violation; tri-state filter that's actually bi-state
  - HelpMixin's lazy-mount pattern (unique among modals)
  - The complete help-text content (canonical keymap doc)
  - CustomizationActionsMixin's five-stage CRUD lifecycle
  - The four-method DRY violation in CRUD entries
  - MarketplaceMixin's 12 handlers fully mapped
  - The subprocess `shell=True` + list args POSIX bug (P0 candidate)
  - No-timeout on subprocess (P1)
  - `_resolve_plugin_scope` three-step inference
  - `open_plugin_folder` misnamed (uses $EDITOR not file explorer)
  - Plugin preview state machine: dual customization corpus
  - Full action → mixin location table (29 actions / 29 bindings)
  - Translation matrix additions for @work, subprocess, async-without-await
- **Existing items refined:**
  - Round 1's "two parallel action-availability mechanisms" — confirmed by reading
  - Round 1's `_fatal_error` — confirmed never invoked
  - Help help/Esc closure mechanism — actually broken on Esc
- **Remaining gaps:**
  - The actual POSIX behavior of `subprocess.run(shell=True, ...)` with list args — should be verified empirically or by reading tests. **High-value verification target.**
  - `_panels` numeric scheme of 0-7 vs 0-6 — discrepancy between docs (README, help text) and code (bindings, navigation). Already noted; just needs decision.

## 11. Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: Round 2 surfaced:
1. A **P0 candidate bug** (`shell=True` with list args on POSIX) that would change the port's subprocess handling design.
2. A **P1 bug** (`action_focus_previous_panel` wraps to HOOK not LSP_SERVER).
3. The complete action-to-source map for the 29 bindings — the canonical port reference.
4. The dual-corpus state machine for plugin preview mode.
5. The lazy-mount pattern unique to HelpMixin.

Removing this round's findings would force the port team to either re-derive the action map or introduce the same bugs. **Strong substantive content.**

## 12. Convergence Declaration

Another round needed — substantive verification gap remains:
- The `subprocess.run(shell=True, ...)` with list args is the central P0 candidate. **Must read tests** to determine whether the marketplace flow actually works as written, or if there's a hidden CI mock. If it works, my reading of Python's subprocess docs is wrong (less likely) or there's a platform-specific reason. If it's broken, this is the most important port-relevant finding.
- The `escape` priority cascade (Round 1) deserves a focused trace through actual key handling — confirm Textual's behavior matches my reading.

Round 3 should focus on test files (`tests/`) to validate behavioral claims, especially the subprocess shell-out flow.

## 13. State Checkpoint

```yaml
pass: B
subpass: app-keybindings
round: 2
status: complete
timestamp: 2026-05-11T18:20:00Z
novelty: SUBSTANTIVE
files_analyzed:
  - src/lazyclaude/mixins/navigation.py
  - src/lazyclaude/mixins/filtering.py
  - src/lazyclaude/mixins/help.py
  - src/lazyclaude/mixins/customization_actions.py
  - src/lazyclaude/mixins/marketplace.py
new_findings:
  p0_candidates: 1
  p1: 3
  p2: 4
  p3: 4
```
