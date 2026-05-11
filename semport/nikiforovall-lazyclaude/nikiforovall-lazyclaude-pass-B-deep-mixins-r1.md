# Pass B Deepening — Mixins Layer (Round 1)

**Reference:** `/Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/`
**HEAD:** `ebc1f8f3b046a04707340f749b4a441e26df7f6d` (main)
**Round:** 1
**Subject:** `src/lazyclaude/mixins/` — five-mixin composition that supplies `action_*` and `on_*` handlers to `LazyClaude(App)`.

This round establishes the action-dispatch design surface, files KNOWN P1s with full citations, characterizes shared-state ownership, and proposes the Rust translation pattern. Round 2 should hunt for any handler family or invariant missed here.

## File scope and LOC

| File | Lines (wc) | Public surface |
|---|---|---|
| `mixins/__init__.py` | 15 | re-exports the 5 mixin classes |
| `mixins/navigation.py` | 132 | 13 `action_*`, 1 private helper, 1 private focus |
| `mixins/filtering.py` | 107 | 6 `action_*`, 1 private updater |
| `mixins/help.py` | 73 | 1 `action_*`, 2 private show/hide |
| `mixins/marketplace.py` | 430 | 2 `action_*`, 17 `on_*` modal handlers, 1 thread worker |
| `mixins/customization_actions.py` | 279 | 4 `action_*`, 7 `on_*` confirm handlers, 3 private helpers |
| `mixins/CLAUDE.md` | 67 | module documentation |
| **Total** | **1,103** | — |

Citations below use `path:line`.

## Composition (MRO) and class statement

`src/lazyclaude/app.py:53-60`:

```python
class LazyClaude(
    NavigationMixin,
    FilterMixin,
    MarketplaceMixin,
    CustomizationActionsMixin,
    HelpMixin,
    App,                  # Textual base last
):
```

Python C3 MRO: `LazyClaude → NavigationMixin → FilterMixin → MarketplaceMixin → CustomizationActionsMixin → HelpMixin → App → object`. None of the five mixins define overlapping `action_*` or `on_*` names — verified by inspecting every file. No diamond conflicts, no `super()` chains inside mixins (each handler stands alone), no `__init__` defined in any mixin. **State lives exclusively on the `LazyClaude` instance**, not on the mixin classes, even though the mixins declare type stubs for IDE/mypy support.

Class constants `_COPYABLE_TYPES` and `_PROJECT_LOCAL_TYPES` are declared on `LazyClaude` (`app.py:70-78`) and **referenced from the mixin** via the type stub at `customization_actions.py:25-26`. This is a structural coupling, not a runtime one — mixin can only be used with a host that supplies those constants.

## Action-dispatch surface — every binding to every handler

The complete keypress → handler routing, gathered by triangulating `src/lazyclaude/bindings.py:5-37` against each mixin and `app.py`:

| Key | Action name | Definer | Method |
|---|---|---|---|
| `q` | `quit` | `app.py` | `action_quit` (`app.py:543`) |
| `?` | `toggle_help` | HelpMixin | `action_toggle_help` (`help.py:13`) |
| `r` | `refresh` | `app.py` | `action_refresh` (`app.py:547`) |
| `e` | `open_in_editor` | `app.py` | `action_open_in_editor` (`app.py:552`) |
| `c` | `copy_customization` | CustomizationActionsMixin | `action_copy_customization` (`customization_actions.py:37`) |
| `m` | `move_customization` | CustomizationActionsMixin | `action_move_customization` (`customization_actions.py:63`) |
| `d` | `delete_customization` | CustomizationActionsMixin | `action_delete_customization` (`customization_actions.py:93`) |
| `C` | `copy_config_path` | `app.py` | `action_copy_config_path` (`app.py:602`) |
| `tab` | `focus_next_panel` | NavigationMixin | `action_focus_next_panel` (`navigation.py:23`) |
| `shift+tab` | `focus_previous_panel` | NavigationMixin | `action_focus_previous_panel` (`navigation.py:37`) |
| `a` | `filter_all` | FilterMixin | `action_filter_all` (`filtering.py:28`) |
| `u` | `filter_user` | FilterMixin | `action_filter_user` (`filtering.py:38`) |
| `p` | `filter_project` | FilterMixin | `action_filter_project` (`filtering.py:48`) |
| `P` | `filter_plugin` | FilterMixin | `action_filter_plugin` (`filtering.py:58`) |
| `D` | `toggle_plugin_enabled_filter` | FilterMixin | `action_toggle_plugin_enabled_filter` (`filtering.py:68`) |
| `t` | `toggle_plugin_enabled` | CustomizationActionsMixin | `action_toggle_plugin_enabled` (`customization_actions.py:117`) |
| `/` | `search` | FilterMixin | `action_search` (`filtering.py:86`) |
| `[` | `prev_view` | NavigationMixin | `action_prev_view` (`navigation.py:106`) |
| `]` | `next_view` | NavigationMixin | `action_next_view` (`navigation.py:113`) |
| `0` | `focus_main_pane` | NavigationMixin | `action_focus_main_pane` (`navigation.py:101`) |
| `1`-`7` | `focus_panel_N` | NavigationMixin | `action_focus_panel_1..7` (`navigation.py:65-99`) |
| `ctrl+u` | `open_user_config` | `app.py` | `action_open_user_config` (`app.py:589`) |
| `M` | `toggle_marketplace` | MarketplaceMixin | `action_toggle_marketplace` (`marketplace.py:54`) |
| `escape` (priority) | `exit_preview` | MarketplaceMixin | `action_exit_preview` (`marketplace.py:154`) |
| `escape` (fallback) | `back` | NavigationMixin | `action_back` (`navigation.py:120`) |

**Notes that matter for the Rust port:**

- Two `escape` bindings share a key — the priority one (`exit_preview`) is gated by `check_action` at `app.py:227-228` so it only fires in preview mode. Textual's `check_action` returning `False` makes the priority binding fall through to `back`. Rust port must replicate the **conditional priority** semantics (a binding can be defined but inactive based on app state).
- `7` (`focus_panel_7` for LSP Servers) is bound (`bindings.py:32`) but the LSP-related help text in `help.py:30` lists only `1-6`. Latent inconsistency — Rust port should reconcile.
- `Tab`/`Shift+Tab` are `show=False` in bindings (`bindings.py:14-15`). Footer is the visible discoverability path for actions; the bindings list controls the keymap, the footer controls UX.

## Modal-confirm-callback pattern (THE central design pattern)

Every destructive or scope-selecting action follows the same three-phase shape:

1. **Phase A — Initiator** (`action_*` in mixin):
   - Validate preconditions (selection exists, type is copyable, level is not plugin).
   - Snapshot focus restoration state into instance attrs (`_panel_before_selector`, `_combined_before_selector`).
   - For copy/move only: snapshot the pending customization into `_pending_customization`.
   - Show the modal widget; the modal grabs focus and grabs keys.

2. **Phase B — Modal** (widget in `widgets/`):
   - Modal has its own `BINDINGS` (`1`/`2`/`3` for level, `y`/`n` for confirm, `escape` for cancel).
   - Modal's own `action_*` handlers `post_message` a typed `Confirmed` or `Cancelled` event back to the app.
   - Modal calls `self.hide()` before posting the message (e.g., `level_selector.py:115`).

3. **Phase C — Resolver** (`on_*` handler in mixin):
   - Handler signature must match Textual's snake_case message routing convention: `on_<WidgetSnake>_<MessageSnake>`.
   - Performs the actual write via a service (`CustomizationWriter`).
   - Reports success/error via `notify()` or `_show_status_success/error`.
   - Calls `action_refresh()` to reload disk state.
   - Calls `_restore_focus_after_selector()` (defined in `app.py:647-658`) to return focus to the panel snapshotted in Phase A.

### Complete pairing table for the modal/callback pattern

| Modal widget | Mixin initiator | Confirmed handler | Cancelled handler |
|---|---|---|---|
| `LevelSelector` | `action_copy_customization` (`customization_actions.py:37`), `action_move_customization` (`customization_actions.py:63`) | `on_level_selector_level_selected` (`customization_actions.py:214`) | `on_level_selector_selection_cancelled` (`customization_actions.py:225`) |
| `DeleteConfirm` | `action_delete_customization` (`customization_actions.py:93`) | `on_delete_confirm_delete_confirmed` (`customization_actions.py:259`) | `on_delete_confirm_delete_cancelled` (`customization_actions.py:274`) |
| `PluginConfirm` | `action_toggle_plugin_enabled` (`customization_actions.py:117`) | `on_plugin_confirm_plugin_confirmed` (`customization_actions.py:233`) | `on_plugin_confirm_confirmation_cancelled` (`customization_actions.py:252`) |
| `FilterInput` | `action_search` (`filtering.py:86`) | `on_filter_input_filter_applied` (in `app.py:532`) | `on_filter_input_filter_cancelled` (in `app.py:515`) |
| `MarketplaceModal` (composite) | `action_toggle_marketplace` (`marketplace.py:54`) | 9 distinct `on_marketplace_modal_*` handlers | `on_marketplace_modal_modal_closed` |
| `MarketplaceConfirm` | (via `MarketplaceModal.MarketplaceRemove`) | `on_marketplace_confirm_remove_confirmed` (`marketplace.py:390`) | `on_marketplace_confirm_remove_cancelled` (`marketplace.py:403`) |
| `MarketplaceSourceInput` | (via `MarketplaceModal.MarketplaceAddRequest`) | `on_marketplace_source_input_source_submitted` (`marketplace.py:413`) | `on_marketplace_source_input_source_cancelled` (`marketplace.py:424`) |

The `MarketplaceModal` is unusual: it emits **9 distinct message classes** for different actions (preview, toggle, install-with-scope, uninstall, update plugin, update marketplace, open folder, open source, marketplace-update, marketplace-remove, add-request), each with its own handler. The marketplace is a *self-contained sub-app* inside the modal, and it bubbles up typed messages — not a single "selection" signal.

## Shared state — every attribute the mixins touch on `self`

Gathered by scanning each mixin's type stubs and method bodies. All attributes are initialized in `LazyClaude.__init__` (`app.py:80-123`).

### Owned by `NavigationMixin` (read/write)

- `_panels: list[TypePanel]` — three list panels for SlashCommand/Subagent/Skill (`navigation.py:16`)
- `_combined_panel: CombinedPanel | None` — tabbed panel for Memory/MCP/Hook/LSP (`navigation.py:17`)
- `_main_pane: MainPane | None` — content/metadata viewer (`navigation.py:18`)
- `_last_focused_panel: TypePanel | None` — Esc-back target (`navigation.py:19`)
- `_last_focused_combined: bool` — Esc-back target qualifier (`navigation.py:20`)
- `_plugin_preview_mode: bool` — affects `action_back` branching (`navigation.py:21`)

Calls cross-mixin: `self._exit_plugin_preview()` (in MarketplaceMixin) at `navigation.py:123`.

### Owned by `FilterMixin` (read/write)

- `_level_filter: ConfigLevel | None` (`filtering.py:19`)
- `_plugin_enabled_filter: bool | None` (`filtering.py:20`)
- `_last_focused_panel: TypePanel | None` — **shared with NavigationMixin**, mutated to clear the panel snapshot after filter change (`filtering.py:21`)
- `_main_pane: MainPane | None` (`filtering.py:22`)
- `_filter_input: FilterInput | None` (`filtering.py:23`)
- `_status_panel: StatusPanel | None` (`filtering.py:24`)
- `_app_footer: AppFooter | None` (`filtering.py:25`)
- `_discovery_service: ConfigDiscoveryService` (`filtering.py:26`)

Calls cross-mixin / cross-class: `self._update_panels()`, `self._update_subtitle()` (both on `LazyClaude`).

### Owned by `MarketplaceMixin` (read/write)

13 attributes (`marketplace.py:36-52`):
- modal references: `_marketplace_modal`, `_marketplace_confirm`, `_marketplace_source_input`, `_marketplace_loader`
- preview state: `_plugin_preview_mode`, `_previewing_plugin`, `_plugin_customizations`
- shared with Filter: `_search_query`, `_status_panel`, `_filter_input`, `_app_footer`, `_main_pane`, `_combined_panel`
- shared with CustomizationActions: `_panel_before_selector`, `_combined_before_selector`
- `_discovery_service`, `_settings`

Calls cross-mixin: `_restore_focus_after_selector` (app.py), `_get_focused_panel` (app.py), `_update_panels` (app.py), `_update_subtitle` (app.py), `_update_footer_actions` (app.py), `_update_status_panel` (app.py), `action_refresh` (app.py), `self.refresh_bindings` (App).

### Owned by `CustomizationActionsMixin` (read/write)

12 attributes (`customization_actions.py:25-35`):
- class constants: `_COPYABLE_TYPES`, `_PROJECT_LOCAL_TYPES` (declared on `LazyClaude`, type-stubbed in mixin)
- widget refs: `_main_pane`, `_level_selector`, `_plugin_confirm`, `_delete_confirm`, `_combined_panel`
- pending operation: `_pending_customization`
- focus restoration: `_panel_before_selector`, `_combined_before_selector` (shared with Marketplace)
- `_discovery_service`

### Owned by `HelpMixin` (read/write)

- `_help_visible: bool` (`help.py:11`) — sole owned state

Help is the **simplest mixin** — no widget refs, no service refs, no message routing. The overlay is created on demand via `self.mount(Static(...))` (`help.py:63`) and removed via `query_one(...).remove()` (`help.py:70`).

### Cross-mixin shared state summary

| Attribute | Owners | Read | Write |
|---|---|---|---|
| `_main_pane` | Navigation, Filter, Marketplace, CustomizationActions | all four | Filter (sets to None after filter), Marketplace (sets to README) |
| `_combined_panel` | Navigation, Marketplace, CustomizationActions | all three | Navigation (focus), Marketplace (switch tab) |
| `_last_focused_panel` | Navigation, Filter | both | Filter (resets to None on filter change) |
| `_panel_before_selector` | Marketplace, CustomizationActions | both | both |
| `_combined_before_selector` | Marketplace, CustomizationActions | both | both |
| `_search_query` | Filter (via app message handler), Marketplace | both | Marketplace (clears on preview exit) |
| `_plugin_preview_mode` | Navigation, Marketplace | both | Marketplace only |
| `_filter_input` | Filter, Marketplace | both | Marketplace (clears on preview exit) |
| `_status_panel` | Filter, Marketplace | both | both |
| `_app_footer` | Filter, Marketplace | both | both |
| `_discovery_service` | Filter, Marketplace, CustomizationActions | all three | none |

**Conclusion:** mixins share state freely via the `LazyClaude` instance — they are not isolated. They are syntactic groupings of related handlers, not encapsulated capabilities. A Rust port that tries to encapsulate (e.g., one struct per mixin holding its own state) will discover this entangled graph the hard way; it must replicate the **all-state-on-app** pattern.

## P1 confirmed: move-without-rollback

**Citation:** `customization_actions.py:165-212`, specifically the `_handle_copy_or_move` flow.

```
165 def _handle_copy_or_move(
166     self, customization, target_level, operation
167 ):
...
173     writer = CustomizationWriter()
174
175     if customization.type == CustomizationType.MCP:
176         success, msg = writer.write_mcp_customization(...)
181     elif customization.type == CustomizationType.HOOK:
182         success, msg = writer.write_hook_customization(...)
188     else:
189         success, msg = writer.write_customization(...)
196     if not success:
197         self._show_status_error(msg)
198         return
199
200     if operation == "move":
201         delete_success, delete_msg = self._delete_customization(
202             customization, writer
203         )
204         if not delete_success:
205             self._show_status_error(
206                 f"Copied but failed to delete source: {delete_msg}"
207             )
208             return
209         msg = f"Moved '{customization.name}' to {target_level.label} level"
210
211     self._show_status_success(msg)
212     self.action_refresh()
```

**Confirmed semantics:** A move is implemented as `write_<type>_customization(target)` followed by `_delete_customization(source)`. **If the write succeeds and the delete fails, the source remains and the user has two copies.** The error toast tells the user — but no rollback (delete of target) is attempted, no transaction marker is stored, and a subsequent `r` refresh will surface both. The next "move" attempt from the duplicated source would silently re-overwrite the target with itself.

This is non-atomic in three orthogonal ways:
1. **No rollback** — copy is not undone if delete fails (the documented finding).
2. **No same-volume rename optimization** — even when source and target are on the same filesystem, the writer always reads+writes; an atomic `rename(2)` could move single-file customizations transactionally.
3. **No interlock with `claude` CLI** — if the user has `claude` running concurrently, the per-file mutation of `~/.claude.json` (for MCP entries) is racy with `claude`'s own writes.

### Rust port recommended fix (two-tier)

1. **Single-file types** (slash commands, subagents, skill directories, memory files) on the **same volume**: use `std::fs::rename(source, target_parent / name)` — POSIX rename is atomic and the source disappears in the same syscall as the target appears. No rollback needed because the operation is indivisible. Fall through to copy+delete only for cross-volume moves (detected by `EXDEV` errno from `rename`).
2. **JSON-key types** (MCP entries, hooks): two-key mutation in a single JSON file is naturally atomic if we rewrite the whole file with `tempfile::NamedTempFile + persist`. For cross-file moves (`./.mcp.json` → `~/.claude.json[mcpServers]`), perform: (a) write target, (b) **read-back verify** target, (c) delete source. If (c) fails, attempt **rollback delete on target** and emit error with rollback-status field. The user should see one of three end states: `moved`, `failed-source-unchanged`, `failed-rollback-failed-both-exist` (the latter requires manual reconciliation guidance).

The reference codebase is at level (0) — no transactional safety. Monocle should target at least level (1) for single-file types and level (2) for JSON types.

## P1 confirmed: subprocess.run(list_cmd, shell=True)

**Citation:** `marketplace.py:248-261`, inside the `_run_plugin_command` worker:

```
248 @work(thread=True)
249 def _run_plugin_command(self, cmd: list[str], success_msg: str) -> None:
250     """Run a plugin command in a background worker."""
251     try:
252         cwd = str(self._discovery_service.project_root)
253         subprocess.run(
254             cmd,
255             capture_output=True,
256             check=True,
257             shell=True,
258             encoding="utf-8",
259             errors="replace",
260             cwd=cwd,
261         )
```

`cmd` is a **list**, but `shell=True` is set. On POSIX, `subprocess.run(["claude","plugin","install","x","--scope","user"], shell=True)` executes `/bin/sh -c "claude"` and **discards all subsequent list items** — so on Linux/macOS, this command effectively launches `claude` with no arguments. The function then likely succeeds (because `claude` with no args prints help and exits 0) **without performing the install**, then notifies the user "Installed plugin X" — a **silent no-op success**, the worst kind of failure mode.

On Windows (`os.name == "nt"`), `subprocess.run(list, shell=True)` is more permissive — Windows-style shell joining of the list. So this bug is **POSIX-only**, which explains why the test suite (likely on macOS/Linux CI) might not catch it if it doesn't assert observed side effects against `installed_plugins.json`.

**Also a latent command-injection vector:** plugin IDs come from `marketplace.json` (a third-party manifest). If a marketplace contains a plugin with the ID `evil; rm -rf ~`, the shell would interpret it. Today plugin IDs are sanitized by the `claude` CLI's marketplace validator, but the layered defense here is broken.

**Same misuse repeats at `marketplace.py:293`:**

```
292 editor = os.environ.get("EDITOR", "vi")
293 subprocess.Popen([editor, str(plugin.install_path)], shell=True)
```

Same POSIX bug pattern — the path argument is dropped on Linux/macOS. The user sees their editor open with no file. The Pass 8 anti-pattern table also flags `app.py:576` and `app.py:587` which use the slightly-better idiom `shell=(sys.platform == "win32")` — gated on Windows only. The marketplace mixin variants are not gated and are unambiguously buggy on POSIX.

### Rust port fix

`std::process::Command::new("claude").arg("plugin").arg("install").arg(...).status()` — no shell involvement. Plugin IDs are passed as separate `arg` calls; injection is structurally impossible. Background execution: `tokio::process::Command` or a `std::thread::spawn` + `Command::status()`.

## Filtering — debounce, fuzzy match, lifecycle

### No debounce

`filtering.py:86-89`:

```
86 def action_search(self) -> None:
87     """Activate search mode."""
88     if self._filter_input:
89         self._filter_input.show()
```

That's the entire `action_search`. The `FilterInput` widget (`widgets/filter_input.py:77-80`) emits `FilterChanged` **on every keystroke**:

```
77 def on_input_changed(self, event: Input.Changed) -> None:
78     """Handle input changes."""
79     self.filter_query = event.value
80     self.post_message(self.FilterChanged(event.value))
```

The app handler (`app.py:499-513`) recomputes the filtered list and refreshes panels on every change. **No debounce, no throttle, no async scheduling.** For the lazyclaude dataset sizes (typically <500 customizations) this is fine; for a Rust port handling larger inventories or expensive matchers, a **debounce window (e.g., 50ms via tokio sleep)** would be a sensible addition.

### Matcher is substring on lowercase, not fuzzy

`services/filter.py:109-118`:

```
109 def _matches_query(self, customization, query):
110     """Check if customization matches the search query."""
111     if query in customization.name.lower():
112         return True
113     if customization.plugin_info:
114         prefix = f"{customization.plugin_info.short_name}:".lower()
115         full_name = f"{prefix}{customization.name.lower()}"
116         if query in prefix or query in full_name:
117             return True
118     return False
```

**This is plain substring containment after `.lower()` on both sides.** No fuzzy matching, no Levenshtein, no scoring, no typo tolerance. The brief's mention of "fuzzy match logic" is **inaccurate for this codebase** — the filtering is substring-only. The `marketplace_modal` has its own search and filtering implementation that I did not deepen here; that may be where fuzzy comes from, if anywhere. Pass B-deep-widgets-r1 should be cross-checked.

For monocle's Rust port, the substring-only matcher is trivial. If the project wants fuzzy, the `fuzzy-matcher` crate (Skim algorithm) is the idiomatic choice. The decision is a feature-level one, not a fidelity-to-reference issue.

### Filter state lifecycle

State machine for filtering, reconstructed:

```
            +-------- a (filter_all) --------+
            |                                |
            v                                |
    [level=None] -- u --> [level=USER] ------+
            ^                                |
            |                                v
            +------ p ------ [level=PROJECT] +
            ^                                |
            |                                v
            +------ P ------ [level=PLUGIN]  +
            |
            +------ D (toggle_plugin_enabled_filter):
                       toggles  _plugin_enabled_filter
                       between {True, None}
                       (note: False is unreachable from UI)
            +------ / (action_search):
                       shows FilterInput
                       on FilterChanged -> _search_query mutates,
                       panels refresh, no debounce
                       on FilterApplied -> hide input
                       on FilterCancelled -> _search_query = "",
                       panels refresh
```

Notable invariants from the code:

- Every level-filter action **clears `_last_focused_panel = None`** (`filtering.py:31`, `:41`, `:51`, `:61`) and **clears `_main_pane.customization`** to None. After a filter change, the previous selection is gone. This is a UX choice (filter changes context, so selection from prior context is invalid).
- `action_toggle_plugin_enabled_filter` (`filtering.py:68-84`) is a **two-state toggle** between `True` (show only enabled) and `None` (show all). The `False` state (show only disabled) is initialized in `app.py:98` to `True` and never enters via UI — `False` is reachable only via `FilterService.filter(plugin_enabled=False)` programmatically. Pass 8's filter feature table is correct.
- `_update_status_filter` (`filtering.py:91-107`) is interesting: it **hardcodes path strings** for User (`~/.claude`), Plugin (`~/.claude/plugins`), only Project pulls from `_discovery_service.project_config_path`. The "All" case shows the project root name. A Rust port should centralize these path constants in a `paths::*` module (consistent with the Pass 8 anti-patterns recommendation).

## Help overlay — composition and lifecycle

`help.py` is the simplest mixin and deserves its own row. Three observations matter for the Rust port:

1. **The help text is a string literal embedded in the method body** (`help.py:22-59`). No data structure, no map from binding to description, no localization hook. If the binding set changes, the help string must be edited by hand. There is **no shared source of truth** between `bindings.py` and `help.py`. Genuine inconsistency: `help.py:30` says `1-3` and `4-6`, but `bindings.py:32` defines a 7th panel binding.
2. **Mount/remove via DOM mutation** (`help.py:61-64`, `help.py:68-70`): `self.mount(Static(...))` is called when no `#help-overlay` exists; `query_one("#help-overlay").remove()` is called to dismiss. **No animation, no fade, no stacked overlays** — single help instance at most. The visibility flag (`_help_visible`) is set after mount but the mount/remove is the source of truth. There is a **race window**: if `_show_help` runs twice quickly (rapid double-`?`), the `if not self.query(...)` guards against duplicate mount. Good defensive code.
3. **`_hide_help` swallows `Exception` blanket** (`help.py:72-73`). This is the only error swallow in the mixin layer. Acceptable here because the only failure mode is "widget not present", but Rust port should match with a `if let Ok(widget) = self.query_one(...) { widget.remove() }` or similar.

For Rust port: **derive the help text from the binding registry** programmatically. The `ratatui` ecosystem has no built-in binding-to-help converter, so build one. A `KeyBinding { key, action, description, scope }` registry consumed by both the dispatcher and a help-overlay renderer eliminates the drift seen here.

## Navigation — focus state machine

The mixin maintains a **two-track focus model** (`navigation.py:51-58`):

- Track 1: the three `_panels` list (`TypePanel` instances for SlashCommand/Subagent/Skill).
- Track 2: the single `_combined_panel` (`CombinedPanel` with tabs for MemoryFile/MCP/Hook/LSPServer).

`_get_focused_panel_index` returns:
- `0..len(_panels)-1` if a regular panel has focus (typically 0, 1, or 2),
- `len(_panels)` (typically 3) if the combined panel has focus,
- `None` if no panel has focus.

`action_focus_next_panel` cycles: panel_0 → panel_1 → panel_2 → combined → panel_0 (wrap). `action_focus_previous_panel` reverses it. **The combined panel's tab is switched as a side effect of cycling into it**:
- Forward into combined → switches to `CustomizationType.MEMORY_FILE` (`navigation.py:32`).
- Backward into combined → switches to `CustomizationType.HOOK` (`navigation.py:42`).

This is an **affordance for tab ordering**: forward through panels lands on the first tab of the combined panel; backward lands on the last tab. The Rust port should preserve this — it's a subtle UX nicety that makes the tab-cycle feel continuous rather than landing mid-tabbed-pane.

Numbered keys `1`-`7` (`navigation.py:65-99`) provide direct focus to a specific panel/tab — bypassing the cycle. Key `0` focuses the main pane (content viewer). Note the **inconsistency**: `bindings.py:32` defines `7` → `focus_panel_7`, and the mixin defines `action_focus_panel_7` to switch the combined panel to `LSP_SERVER` — but the `help.py` overlay does not advertise this (it says `4-6`).

### `action_back` (Esc)

`navigation.py:120-132`:

```
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

Two-level back semantics:
1. If in plugin preview mode, **exit preview** (cross-mixin call into MarketplaceMixin).
2. If main pane has focus, **return focus to the last-focused panel** (using the `_last_focused_panel`/`_last_focused_combined` snapshots from drill-down). If neither snapshot exists, fall back to `_panels[0]`.

This is the only `async` action in the navigation mixin — everything else is sync. The `async` is preserved for Textual's awaiting of `_exit_plugin_preview` if it ever becomes async; today the called function is sync. Rust port can model this as a regular function returning a `Result<()>`.

## Marketplace — sub-app dispatch and worker model

The MarketplaceMixin is the largest mixin (430 LOC, ~40% of the layer) and is **not really a mixin in the same sense as the others** — it's a delegation surface to a sub-application (the `MarketplaceModal` widget) that has its own internal state, tree, and bindings.

### Toggle and preview

`action_toggle_marketplace` (`marketplace.py:54-68`) is a **show/hide toggle** with focus state snapshot on show. `_enter_plugin_preview` (`marketplace.py:70-132`) replaces the panel contents with a single plugin's customizations and routes the README.md into the main pane. `_exit_plugin_preview` (`marketplace.py:134-152`) reverses it.

The preview mode is a **modal state** that gates other actions via `check_action` (`app.py:244-264`). In preview, only a whitelist of 13 actions is allowed:

```
preview_allowed_actions = {
    "quit", "toggle_help", "search", "copy_config_path",
    "focus_next_panel", "focus_previous_panel",
    "focus_panel_1..7", "focus_main_pane",
    "prev_view", "next_view", "exit_preview",
}
```

Notably **forbidden** in preview: copy, move, delete, refresh, edit, filter changes, marketplace toggle, plugin enable toggle. The preview is read-only.

### Background worker for shell commands

`_run_plugin_command` (`marketplace.py:248-267`) uses Textual's `@work(thread=True)` decorator to run `subprocess.run` off the UI thread. On success, it calls `self.call_from_thread(self._on_plugin_command_success, success_msg)` to marshal the result back to the UI thread (Textual's thread-safety convention).

The error branches handle two distinct cases:
1. `subprocess.CalledProcessError` → wraps stderr/exit-code into an error message.
2. `FileNotFoundError` → "Claude CLI not found".

**Missing error branches:** generic `OSError` (permission denied, etc.), `TimeoutExpired` (no timeout is set so this is impossible today), `KeyboardInterrupt`. For a Rust port, `tokio::process::Command::output()` returns `Result<Output, io::Error>` and exhaustive matching is enforced.

### Marketplace handler families

Nine distinct `on_marketplace_modal_*` handlers map to nine modal events:

| Modal message | Handler | What it does |
|---|---|---|
| `PluginPreview` | `on_marketplace_modal_plugin_preview` (line 158) | enters preview mode |
| `PluginToggled` | `on_marketplace_modal_plugin_toggled` (line 176) | `claude plugin enable/disable` |
| `PluginInstallWithScope` | `on_marketplace_modal_plugin_install_with_scope` (line 203) | `claude plugin install --scope X` |
| `PluginUninstall` | `on_marketplace_modal_plugin_uninstall` (line 222) | `claude plugin uninstall` |
| `OpenPluginFolder` | `on_marketplace_modal_open_plugin_folder` (line 282) | `subprocess.Popen([editor, path], shell=True)` — **POSIX bug** |
| `OpenPluginSource` | `on_marketplace_modal_open_plugin_source` (line 295) | opens in file explorer or webbrowser |
| `OpenMarketplaceSource` | `on_marketplace_modal_open_marketplace_source` (line 321) | same |
| `MarketplaceUpdate` | `on_marketplace_modal_marketplace_update` (line 341) | `claude plugin marketplace update` |
| `PluginUpdate` | `on_marketplace_modal_plugin_update` (line 350) | `claude plugin update` |
| `ModalClosed` | `on_marketplace_modal_modal_closed` (line 367) | restores focus |
| `MarketplaceRemove` | `on_marketplace_modal_marketplace_remove` (line 375) | shows MarketplaceConfirm |
| `MarketplaceAddRequest` | `on_marketplace_modal_marketplace_add_request` (line 382) | shows MarketplaceSourceInput |

Two sub-modal handlers nested under the marketplace flow:
- `on_marketplace_confirm_remove_confirmed` (line 390) → `claude plugin marketplace remove`
- `on_marketplace_source_input_source_submitted` (line 413) → `claude plugin marketplace add`

The **scope resolution helper** `_resolve_plugin_scope` (`marketplace.py:164-174`) is non-trivial: it inspects the modal's `scope_view` and the plugin's `installed_scopes` set to pick the right `--scope` value. If the user views in "project" mode and the plugin is installed at both `project` and `local`, it picks `"project"` first, then `"local"`, defaulting to `"project"` if neither. This **set-algebra behavior** is documented in Pass B-deep-plugin-marketplace-r1; the mixin is the consumer.

## Error-handling consistency

| Mixin | Error path |
|---|---|
| Navigation | None — no error surfaces. Defensive `if` checks for None widgets. |
| Filter | None — purely declarative state mutation. |
| Help | Blanket `except Exception: pass` (`help.py:72-73`) on widget removal. |
| Marketplace | Worker captures `CalledProcessError` and `FileNotFoundError`, calls `_on_plugin_command_error` to notify. Sync handlers use `self.notify(..., severity="warning")` or `"error"` for explicit failures (missing path, unknown source type). |
| CustomizationActions | All initiators use `self._show_status_error(...)` for preconditions. Writer outcomes use `self._show_status_success/error` for copy and `self.notify(...)` for delete and toggle. **Inconsistent: copy/move uses `_show_status_*`; delete/toggle uses `notify`.** Both call `notify` underneath but the call sites diverge. |

The **inconsistency between `_show_status_*` and `notify`** is a small smell. `app.py:660-666` defines `_show_status_success(msg)` → `self.notify(msg, severity="information", timeout=3.0)` and `_show_status_error(msg)` → `self.notify(msg, severity="error", timeout=3.0)`. The `notify` direct-call paths use the default timeout (no `timeout=` argument, so Textual's default applies — typically 5s).

For Rust port: define **one notification surface** with a single severity enum (Info/Warn/Error) and a single timeout convention. Avoid the dual-call-path divergence.

## How a Rust port reimplements the mixin pattern

Three reasonable shapes, in increasing order of fidelity:

### Shape A: Flat module + free functions

Drop multiple inheritance entirely. Five modules under `src/app/`:

```
src/app/
  navigation.rs   — fn focus_next_panel(app: &mut App), etc.
  filtering.rs    — fn filter_all(app: &mut App), etc.
  marketplace.rs  — fn toggle_marketplace(app: &mut App), etc.
  customization_actions.rs
  help.rs
  mod.rs          — defines App struct with all state
```

A single `App` struct holds every field the Python mixins reference. The mixin's `action_*` methods become free functions in their respective modules, taking `&mut App`. The dispatcher (the `Action` enum's `apply` method) maps `Action::FocusNextPanel` to `navigation::focus_next_panel(app)`, etc.

**Pros:** simplest, matches Pass 8 D3 recommendation. Mirrors the conceptual grouping. No trait hierarchy needed.

**Cons:** no compile-time enforcement of which actions belong to which "mixin". State is all on `App` and any function can mutate any field. (This matches the Python reference exactly.)

### Shape B: Trait per concern

```rust
trait NavigationActions {
    fn focus_next_panel(&mut self);
    fn focus_panel(&mut self, n: u8);
    fn back(&mut self);
}
impl NavigationActions for App { ... }
```

Each module defines a trait, `App` implements all five.

**Pros:** trait boundaries document the action surface. Type system enforces which actions are part of which concern.

**Cons:** the traits all take `&mut self` on the full `App`, so the encapsulation is illusory. Splitting state per trait via associated types is theoretically possible but practically painful (shared state across traits would require generic projections).

### Shape C: Command/Action enum + handler registry

```rust
enum Action {
    FocusNextPanel, FocusPreviousPanel, FocusPanel(u8), FocusMainPane, Back,
    FilterAll, FilterUser, FilterProject, FilterPlugin, TogglePluginEnabledFilter, Search,
    ToggleMarketplace, ExitPreview,
    CopyCustomization, MoveCustomization, DeleteCustomization, TogglePluginEnabled,
    ToggleHelp,
    Quit, Refresh, OpenInEditor, CopyConfigPath, OpenUserConfig,
    PrevView, NextView,
}

impl Action {
    fn apply(self, app: &mut App) -> Result<()> { match self { ... } }
}

struct KeyMap { bindings: HashMap<KeyEvent, Action> }
```

The match arm calls into the appropriate module (`navigation::focus_next_panel(app)` etc.). The **help text is derived from `Action`'s `Display` impl** — solving the help-text-drift problem identified in `help.py`.

**Pros:**
- Single source of truth for the action set (the `Action` enum).
- Trivially serializable, testable (apply an Action without keyboard events), and remappable (KeyMap is data, not code).
- Conditional binding gates (the `check_action` priority pattern) become `fn is_available(self, app: &App) -> bool` on `Action`.
- The modal-confirm-callback pattern becomes a state machine over `AppMode { Normal, LevelSelectorOpen { pending: PendingOp }, DeleteConfirmOpen, ... }` with explicit transitions; modal events become `ModalMessage` variants routed through `apply`.

**Cons:** larger upfront design effort; the enum will have ~30 variants.

**Recommendation:** **Shape C** for Monocle. The reference codebase implicitly has an action set (the union of binding action names); making it explicit upfront avoids the drift bugs (help text, binding gates, scattered handler files) we found in the reference. The modal state-machine modeling is also more rigorous than Python's "snapshot focus into instance attrs and hope for the best" pattern.

### Translation table for the five mixins

| Python mixin | Rust crate-internal module | Rust idiom |
|---|---|---|
| `NavigationMixin` | `src/app/navigation.rs` | fns on `&mut App`; `FocusTarget` enum to centralize panel-index logic |
| `FilterMixin` | `src/app/filtering.rs` | fns on `&mut App` + `FilterState` struct on App |
| `HelpMixin` | `src/app/help.rs` | render derived from `Action::display_help()` registry; **no embedded string literal** |
| `MarketplaceMixin` | `src/app/marketplace.rs` + `src/app/marketplace_modal.rs` | sub-app pattern with its own state machine and message-passing channel; `tokio::process::Command` for CLI calls |
| `CustomizationActionsMixin` | `src/app/customization_actions.rs` | fns on `&mut App` + `PendingOp` enum (`Copy { src, target }`, `Move { src, target }`, `Delete { src }`, `Toggle { plugin_id }`) carrying the state currently scattered across `_pending_customization` / `_panel_before_selector` / `_combined_before_selector` |

### Modal pattern (Phase A → B → C) in Rust

```rust
enum AppMode {
    Normal,
    AwaitingLevelSelect { op: PendingMove, focus_snapshot: FocusSnapshot },
    AwaitingDeleteConfirm { target: CustomizationId, focus_snapshot: FocusSnapshot },
    AwaitingPluginToggleConfirm { plugin_id: PluginId, focus_snapshot: FocusSnapshot },
    Searching { focus_snapshot: FocusSnapshot },
    MarketplaceOpen { focus_snapshot: FocusSnapshot, modal: MarketplaceState },
    PluginPreview { plugin: MarketplacePlugin, customizations: Vec<Customization> },
}
```

The `apply(action)` dispatcher's first match arm is `match self.mode { ... }`. A key event in `AwaitingLevelSelect` mode routes to the level-select handler; same key in `Normal` mode routes to its normal handler. **No `check_action` gating string-matching** — the mode enum is the gate, exhaustive by construction.

This replaces:
- `_pending_customization: Customization | None` → carried inside `PendingMove`.
- `_panel_before_selector: TypePanel | None` + `_combined_before_selector: bool` → carried inside `FocusSnapshot { Panel(usize) | CombinedPanel | MainPane }`.
- `_help_visible: bool` → carried as `is_some()` on an `Option<HelpOverlay>`.
- `_plugin_preview_mode: bool` + `_previewing_plugin: ... | None` + `_plugin_customizations: list` → carried inside `PluginPreview` variant.

Compile-time invariant: **you cannot be in two modes at once**, and the mode is the gate for action availability. The reference codebase emulates this with a bag of optional fields; the Rust port should encode it as an enum.

## Delta Summary

- New items added:
  - 1 action-dispatch design summary (Modal-Confirm-Callback pattern with phase taxonomy, 7-modal pairing table)
  - 1 complete keybinding → handler mapping table (25 entries) cross-referenced to `bindings.py` and per-mixin definers
  - 1 shared-state table (12 attributes × {owners, read, write} columns)
  - 1 per-mixin error-handling consistency assessment
  - 2 P1 confirmations with byte-precise citations (move-rollback at `customization_actions.py:165-212`; subprocess shell=True at `marketplace.py:253-261` + `marketplace.py:293`)
  - 1 latent inconsistency (help text vs bindings for panel 7)
  - 1 Rust translation guide (3 shapes, recommended Shape C, module-by-module mapping, AppMode state-machine)
  - 1 silent-no-op semantic analysis for the POSIX `shell=True` bug (worst-kind failure: success notification with no install)
  - 1 P2 finding (no debounce on filter input — known small dataset, but flagged for Rust port)
  - 1 P2 finding (inconsistent notification surface — `_show_status_*` vs direct `notify`)
- Existing items refined: Pass 8 anti-pattern row "shell=True misuse" elevated to dual-citation (marketplace.py:253 and marketplace.py:293, not only the documented :253-261); the brief's "fuzzy match logic" claim corrected to substring-on-lowercase
- Remaining gaps:
  - `_resolve_plugin_scope` set-algebra is documented in plugin-marketplace-r1 but the mixin's consumption surface (line 164-174) was not deeply traced into the modal's `scope_view` mutation path
  - `MarketplaceModal` internal state machine — covered in widgets-r1 but the mixin-to-modal API surface (which messages exist, when they fire) was enumerated but not validated against the modal's emit sites
  - The `@work(thread=True)` worker's thread-safety contract w.r.t. `_marketplace_modal.refresh_tree()` called from `_on_plugin_command_success` (sync call on the UI thread, but worth verifying the modal handles `None` modal references)
  - Test coverage of mixins: only `test_app_customization_actions.py` exists (167 LOC, exercises only constants and `_get_available_target_levels`) — **no test exercises `_handle_copy_or_move`'s move-failure rollback path**; this is the P1 staying P1

## Novelty Assessment

Novelty: **SUBSTANTIVE**

This round established the entire action-dispatch design surface, which was previously documented only at a high level in Pass 2 architecture and Pass 8 synthesis D3. The pairing table, the AppMode enum sketch, the explicit modal-callback phase taxonomy, the keybinding→handler full inventory, and the shared-state table are all **new structural understanding** that would substantively change how monocle's ratatui port is designed. Without this round, the port would likely reproduce the Python pattern of a bag of optional fields plus string-keyed `check_action` gating — buggy and untestable in the same ways.

The P1 confirmations carry byte-precise reproduction analysis (POSIX vs Windows divergence, silent-no-op semantics) that goes beyond Pass 8's one-line flag.

## Convergence Declaration

Another round needed. Remaining substantive gaps:

1. **`_resolve_plugin_scope` end-to-end tracing** — the helper at `marketplace.py:164-174` reads `_marketplace_modal.scope_view` (a widget-state attribute not enumerated in this round). Need to map the scope_view's writer/reader sites and validate the assumed semantics.
2. **Modal message contract validation** — enumerate every `post_message` site inside the seven modal widgets and cross-check against the mixin handlers; surface any orphan emits or orphan handlers.
3. **`refresh_bindings` callsite audit** — `self.refresh_bindings()` is called at strategic points (`marketplace.py:100, 146`, `app.py:422, 441, 530, 539`) to recompute Textual's `check_action` results. The trigger points may not be exhaustive — actions could be stale until a binding refresh.
4. **Pending-op coalescing semantics** — what happens if `action_copy_customization` fires while `_pending_customization` is already set from a previous (un-resolved) copy? The mixin doesn't guard against this; the modal hides immediately on each `show()` call but the state isn't cleared.
5. **`_last_focused_panel` vs `_panel_before_selector`** — two separate focus snapshots. The relationship is non-obvious and could be a source of focus bugs. Round 2 should diagram both lifetimes and confirm whether they can race.

If Round 2 reveals nothing substantive on these threads, declare NITPICK.

## State Checkpoint

```yaml
pass: B-deep-mixins
round: 1
status: complete
timestamp: 2026-05-11T19:55:00Z
novelty: SUBSTANTIVE
files_read:
  - mixins/customization_actions.py
  - mixins/filtering.py
  - mixins/help.py
  - mixins/marketplace.py
  - mixins/navigation.py
  - mixins/CLAUDE.md
  - mixins/__init__.py
  - app.py
  - bindings.py
  - services/filter.py
  - widgets/filter_input.py
  - widgets/level_selector.py
  - widgets/delete_confirm.py
  - widgets/plugin_confirm.py
  - tests/unit/test_app_customization_actions.py
new_p1_findings: 0 (two known P1 confirmed with byte-precise citations; not new)
new_p2_findings: 2 (no-debounce filter input; inconsistent notification surface)
new_inconsistencies: 1 (help.py vs bindings.py panel 7 advertisement)
output: nikiforovall-lazyclaude-pass-B-deep-mixins-r1.md
next_round_targets:
  - _resolve_plugin_scope scope_view tracing
  - modal post_message inventory and orphan check
  - refresh_bindings callsite audit
  - pending-op coalescing race
  - _last_focused_panel vs _panel_before_selector lifetimes
```
