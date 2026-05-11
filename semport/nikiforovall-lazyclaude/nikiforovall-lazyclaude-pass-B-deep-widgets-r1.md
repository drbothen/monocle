# Phase B Deepening: Widgets — Round 1

Goal: map widget state machines that aren't captured in service contracts.

## TypePanel state machine

Reactive state (`widgets/type_panel.py:87-93, 129-130`):
- `customization_type` — set once at construction, never mutated
- `customizations` — replaced wholesale by `set_customizations` (filters by type)
- `selected_index` — clamped on `customizations` mutation
- `is_active` — toggled by `on_focus` / `on_blur`
- `panel_number` — for header rendering
- `expanded_skills: set[str]` — skill names that are expanded (skills panel only)
- `expanded_memory_files: set[str]` — memory paths that are expanded (memory panel only)

### "Flat items" projection

For SKILL and MEMORY_FILE panels, the visible list isn't `customizations` directly — it's a flattened projection with expanded children inline. This is rebuilt on every:
- `set_customizations` call
- `watch_customizations` callback
- `action_expand` / `action_collapse`

`_rebuild_flat_items` (`type_panel.py:541-557`):
```python
self._flat_items = []
for skill in self.customizations:
    self._flat_items.append((skill, None))      # parent row
    if skill.name in self.expanded_skills:
        files = skill.metadata.get("files", [])
        self._add_files_to_flat_list(skill, files, indent=1)
```

`_flat_items` is `list[tuple[Customization, Path | None]]`. Parent rows have `file_path=None`; child rows are `(skill, file_path)`. The rendered indent is computed from `file_path.relative_to(skill_dir).parts` length (`type_panel.py:580-589`) — **the indent doesn't come from the flat list's depth tracking** because the recursion already encoded depth via path.

`_rebuild_memory_flat_items` differs (`type_panel.py:559-563`):
```python
self._memory_flat_items = build_memory_flat_items(
    self.customizations, self.expanded_memory_files
)
```
`build_memory_flat_items` (`widgets/helpers/rendering.py:25-46`) returns `list[(memory, ref, depth)]` where depth IS tracked explicitly — because memory refs don't have a direct path relationship to the source file's directory.

**The two flat-list shapes are different.** Skill: `(c, Path | None)`; Memory: `(c, ref, depth)`. The render functions account for this. Monocle's port can converge to a single shape.

### Selection emission

`_emit_selection_message` (`type_panel.py:565-578`):
- For skills panel: emits BOTH `SelectionChanged(skill)` AND `SkillFileSelected(skill, file_path)`. The latter even with `file_path=None` for parent rows.
- For memory panel: emits `SelectionChanged(memory)` AND `MemoryFileRefSelected(memory, ref)`.
- For other panels: emits only `SelectionChanged(c)`.

**Two messages on every navigation step in skills/memory panels.** Each triggers separate handlers in app. `on_type_panel_skill_file_selected` (`app.py:452-464`) updates `MainPane.selected_file` and re-resolves display_path. Multiple back-to-back updates fine because reactives coalesce, but it's noise.

### Click handling

`on_click` (`type_panel.py:423-447`):
1. Focus self.
2. Use `screen.get_widget_at(screen_x, screen_y)` to find clicked widget.
3. Walk up DOM looking for `id="item-N"`.
4. If found, set `selected_index = N`.

**Mouse support is enabled but limited** — only selection. No drill-down on click. P2 UX.

### Collapse/expand semantics

`action_expand` and `action_collapse` operate on the **currently selected item**. They only work when the cursor is on a parent row (file_path=None for skills, ref=None for memory). The implementation **copies the set, mutates, reassigns** to trigger Textual's reactive change detection:

```python
new_expanded = self.expanded_skills.copy()
new_expanded.add(skill.name)
self.expanded_skills = new_expanded
```

After collapse, `_adjust_selection_after_collapse` (`type_panel.py:647-651`) walks the new flat list to find the parent of the collapsed skill and re-selects it. Without this, the cursor would be stranded at an index beyond the new list bounds. Subtle but important.

## CombinedPanel state machine

Two reactive states co-exist: `active_type` (which tab is showing) and `selected_index` (cursor within the current tab's list). The class maintains `_selected_indices: dict[CustomizationType, int]` to remember per-tab cursor positions across tab switches.

`watch_active_type` (`combined_panel.py:242-267`):
1. Save current `selected_index` to `_selected_indices[old_type]`.
2. Compute new tab's item count.
3. Restore `_selected_indices.get(new_type, 0)`, clamped to `[0, count-1]`.
4. Re-render and emit selection.

**Cursor persistence per tab** is a small piece of polish that improves UX dramatically — switch to MCPs to inspect a server, switch back to Memory and you're still on the file you were viewing. Worth replicating.

### Tab navigation flow

`action_focus_next_panel` on CombinedPanel (`combined_panel.py:537-543`):
```python
current_idx = self.COMBINED_TYPES.index(self.active_type)
if current_idx < len(COMBINED_TYPES) - 1:
    self.active_type = COMBINED_TYPES[current_idx + 1]
else:
    cast("LazyClaude", self.app).action_focus_next_panel()
```

Tab cycling within the panel, then delegate up. This means **Tab from MCP→Hook stays within CombinedPanel; Tab from Hook→LSP stays within; Tab from LSP delegates to app**. Subtle: pressing Tab from a single-tab panel would cycle to itself (but COMBINED_TYPES has 4 entries, never just one).

`[` / `]` keys map to `action_prev_tab` / `action_next_tab` and wrap around (`combined_panel.py:520-530`):
```python
new_idx = (current_idx - 1) % len(COMBINED_TYPES)
```

Wraparound via modulo. Different semantics from `Tab` (which delegates at boundary). Two distinct mental models for the same goal. P2.

## MarketplaceModal sub-state machine

Five distinct internal modes (`marketplace_modal.py:213-231`):

1. **Normal browse** — `_filter_input.is_visible == False && _scope_selection_mode == False`. Tree navigation, all keybindings active.
2. **Filter mode** — `_filter_input.is_visible == True`. j/k are consumed by Input widget. Esc returns to browse.
3. **Scope-selection mode** — `_scope_selection_mode == True` after pressing `i` on an uninstalled plugin. Most keys (cursor_down, cursor_up, etc.) are no-ops. Only `1`, `2`, `3`, `Esc` work. **Critical: most other actions return early if `_scope_selection_mode == True`** (see `action_uninstall_plugin:567-575`, `action_open_plugin_folder:582-593`, `action_toggle_scope_view:687-697`).
4. **Installed-only filter** — `_installed_only_filter == True`. Just narrows tree contents; doesn't change input handling.
5. **Scope view (user/project)** — `_scope_view in {"user", "project"}`. Changes which installations are reflected as "installed" in the tree.

### Mode interactions

- Filter mode + scope-selection mode are mutually exclusive (search blocked when scope-selecting).
- Installed-only filter persists across scope-view changes (?). Let me check: `action_toggle_scope_view` calls `_load_data + _build_tree + _select_first_node`. It does NOT reset `_installed_only_filter`. So the filter persists. Good.
- Filter query (`_filter_query`) is NOT reset on scope-view toggle. Persists. Good.

### Close-or-cancel cascade

`action_close_or_cancel` (`marketplace_modal.py:490-498`):
1. If scope-selection mode → exit scope selection (back to normal browse).
2. Else if filter input visible → cancel filter (back to normal browse).
3. Else → hide modal + post `ModalClosed`.

**Three-level Esc** cascade. Subtle. Replicate in Rust port.

## MarketplaceSourceInput state machine

Two-state navigation between `Input` (text field) and `Static` options below. `_selected_index` semantics:
- `-1` → input is focused, text-entry mode
- `0+` → option N is selected, navigation mode

Transitions (`marketplace_source_input.py:165-179`):
- `j` / `down` from input → `_selected_index = 0`
- `j` / `down` from option N → `_selected_index = N+1` (or back to `-1` after last)
- `k` / `up` from option 0 → `_selected_index = -1` (back to input)
- `k` / `up` from input → `_selected_index = len(items) - 1` (last option)

Wrap-around in both directions. Symmetric. **`NavigableInput` swallows j/k/up/down/escape/enter** so the parent widget can implement this logic without the Input consuming the keystrokes (`marketplace_source_input.py:17-26`).

Selection visual (`_update_selection` `:222-239`):
- For each option, recompute label with `selected=True/False`, add/remove `option-selected` class.
- If `selected_index >= 0`, blur input and focus self (the parent widget).
- If `selected_index == -1`, focus input.

**This is the most non-trivial input widget in the codebase** — most others are single-field. The hand-rolled navigation around a single Input is what makes it special.

### Suggestion ordering

`_sort_suggestions` (`marketplace_source_input.py:290-302`):
1. Pinned first: `anthropics/claude-plugins-official`
2. Pinned second: `NikiforovAll/claude-code-rules`
3. Everyone else sorted by stars descending.

**Hardcoded promotional ordering** for the project's curated marketplaces and the author's own marketplace. P2 — visible bias, intentional.

## FilterInput state machine

Trivial show/hide. Three messages:
- `FilterChanged(query)` on every keystroke (real-time filtering)
- `FilterApplied(query)` on Enter
- `FilterCancelled()` on Esc + clear

The app handles `FilterChanged` to update the filtered customizations list and re-render panels. `FilterApplied` just hides the input and refocuses the previously-focused panel. **No "submit-then-close-on-Enter" semantic equivalent in the broader app — filter is persistent until canceled.**

## LevelSelector / DeleteConfirm / PluginConfirm / MarketplaceConfirm

These four widgets share a common skeleton:
- `show(args)` sets state, calls `add_class("visible")`, focuses self.
- `hide()` sets `remove_class("visible")`, clears state.
- Static prompt widget rendered with composed Rich-formatted text.
- y/n/Esc bindings post `Confirmed` or `Cancelled` messages.

**Note that `LevelSelector` uses 1/2/3 not y/n** — the only one in the family with numeric input. Reason: it presents up to three choices, not a binary.

The four are **not consolidated** into a single `ConfirmModal` base class. Each is independently 100-150 LOC. Minor refactoring opportunity. P2.

## MainPane render-decision tree

`_get_renderable` → `_render_metadata` or `_render_file_content`. Within `_render_file_content`:
1. If `selected_file` set → `_render_selected_file` (the user is browsing into a skill subfile).
2. Else if `selected_ref` set → `_render_selected_ref` (the user is browsing into a memory-file ref).
3. Else if no customization → empty state.
4. Else if customization has error → red error message.
5. Else if no content → empty state.
6. Else if `.md` extension → `_render_markdown_with_frontmatter` (special YAML-then-markdown rendering).
7. Else → generic `Syntax(content, lexer)` with lexer map (`.json` → `json`, else `text`).

For `_render_selected_file` (`detail_pane.py:195-235`), a richer lexer map: `.md`/`.json`/`.py`/`.sh`/`.yaml`/`.yml`/`.js`/`.ts`. Markdown gets frontmatter-aware rendering.

For `_render_selected_ref` (`detail_pane.py:237-272`), same lexer map as `_render_selected_file`. Different code path; same rules. **DRY violation** — should share with `_render_selected_file`.

### Pygments theme map

`detail_pane.py:16-30` maps Textual theme names to Pygments theme names. If the user picks a Textual theme not in this map, falls back to `monokai` (`detail_pane.py:32, 140`). **No discovery mechanism** — adding a theme requires editing this dict.

## AppFooter reactive cascade

12 reactive properties (filter_level, search_active, disabled_filter_active, preview_mode, marketplace_modal_visible, can_refresh, can_edit, can_copy, can_move, can_delete). Every change → `_update_content` → recompute and re-render the footer text. Cheap, but **every keypress that changes any reactive triggers a footer re-render**. For high-frequency events (cursor movement), the footer re-renders unnecessarily — none of those reactives change on cursor movement. So in practice it's fine.

Footer constructs differ by mode:
- Preview mode: shows `Esc Exit` instead of `M Marketplace`. Hides r/e/c/m/d.
- Marketplace modal visible: hides filter keys (a/u/p/P/D).
- Always shown: q, ?, /, palette hint.

## Delta Summary

- New items added: 5 widget state machines, 3 sub-state interactions, footer-mode-vs-action-availability rules, MainPane render decision tree, two distinct flat-list shapes (skill vs memory)
- Existing items refined: confirm-widget family commonality; theme map discoverability gap
- Remaining gaps: Marketplace modal's `_marketplace_order` persistence (sort by install count vs preserve user-pinned ordering); accessibility (no screen-reader story)

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: Discovered the **three-level Esc cascade** in MarketplaceModal, the **per-tab cursor persistence** in CombinedPanel, the **two distinct flat-list shapes** for skill vs memory expansion, and the **promotional sort order** in MarketplaceSourceInput. Each is a deliberate UX choice that influences the Rust port's design.

## Convergence Declaration

After Round 1 — there are still genuine gaps in: tests for marketplace modal flows, the writer's hook-merge dedup behavior (does it dedup matchers? — re-read says no, just appends), and a careful inspection of test_behavior.py and test_auto_memory.py. One more focused round on those would help.

## State Checkpoint

```yaml
pass: B
subpass: widgets
round: 1
status: complete
timestamp: 2026-05-11T17:45:00Z
novelty: SUBSTANTIVE
```
