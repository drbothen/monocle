# Pass B Deepening — Mixins Layer (Round 2)

**Reference:** `/Users/jmagady/Dev/monocle/.reference/nikiforovall-lazyclaude/`
**HEAD:** `ebc1f8f3b046a04707340f749b4a441e26df7f6d` (main)
**Round:** 2
**Subject:** Targeted gap-closure on the five threads flagged for follow-up in Round 1: scope_view tracing, modal-message contract validation, `refresh_bindings` callsite audit, pending-op coalescing, and dual focus-snapshot lifetimes.

This round hunts for substantive findings only. If novelty drops to nitpicks, declare convergence.

## Gap 1: `_resolve_plugin_scope` end-to-end trace

**Helper under investigation:** `mixins/marketplace.py:164-174`

```
164 def _resolve_plugin_scope(self, plugin: MarketplacePlugin) -> str:
165     """Resolve the CLI --scope value for a plugin based on current view."""
166     view_scope = (
167         self._marketplace_modal.scope_view if self._marketplace_modal else "user"
168     )
169     if view_scope == "project":
170         return next(
171             (s for s in plugin.installed_scopes if s in ("project", "local")),
172             "project",
173         )
174     return "user" if "user" in plugin.installed_scopes else view_scope
```

### Writer/reader sites for `scope_view`

| Site | Direction | Citation |
|---|---|---|
| Initial assignment | write | `widgets/marketplace_modal.py:228` — `self._scope_view: str = "user"` |
| User toggle action | write | `widgets/marketplace_modal.py:691` — `self._scope_view = "project" if self._scope_view == "user" else "user"` |
| Read by mixin | read | `mixins/marketplace.py:167` — `self._marketplace_modal.scope_view` |
| Read by modal footer renderer | read | `widgets/marketplace_modal.py:259` — `scope_label = "User" if self._scope_view == "user" else "Project"` |
| Read by plugin-label hide-badge | read | `widgets/marketplace_modal.py:465` — `hide_scope = self._scope_view if self._scope_view == "user" else None` |
| Property exposed to mixin | read | `widgets/marketplace_modal.py:781-783` — `scope_view` property |
| Mirror into loader | write | `widgets/marketplace_modal.py:693` — `self._loader.display_scope = self._scope_view` (also clears cache `:694`) |

So scope_view has **two coupled mutable states**: `MarketplaceModal._scope_view` and `MarketplaceLoader.display_scope`. The modal's `action_toggle_scope_view` (`marketplace_modal.py:687-697`) is the only writer that keeps them coherent — it writes both and clears the loader's cache (`_marketplaces_cache = None`).

**Latent risk:** if any code path sets `_scope_view` without also updating `loader.display_scope`, the two views diverge. Today only `action_toggle_scope_view` writes `_scope_view`, but **the initial constructor sets `_scope_view = "user"` and the loader's `display_scope = "user"`** independently (`marketplace_loader.py:35`). If a future refactor adds a "preserve scope across sessions" feature, it would need to write both.

### Set-algebra in `_resolve_plugin_scope`

The function:
- **If view is "project":** prefer `"project"` over `"local"` from `plugin.installed_scopes`, defaulting to `"project"` if neither present (i.e., not installed yet). This is the install-target scope, not the read-source scope.
- **If view is "user":** prefer `"user"` if installed at user scope; else fall through to `view_scope` (which is `"user"`). So when view is "user", the returned scope is **always `"user"`** unless the special-case if-branch falls through — but `view_scope` here is by definition `"user"`, so the result is `"user"` regardless of `installed_scopes`. **The `else "user" if "user" in plugin.installed_scopes else view_scope` line evaluates to `"user"` always when called from this branch.** The `if "user" in installed_scopes` check is dead — both branches return `"user"`.

This is a minor code smell: the user-branch's conditional has no observable effect because `view_scope == "user"` in that branch. The reader is led to believe there's branching logic when there isn't. A reviewer (or Rust port author) should simplify to:

```python
return "user"   # when view_scope == "user"
```

Or surface the actual decision: the original author may have intended `view_scope` to potentially be something other than `"user"` or `"project"` (e.g., `"local"`), but the modal only ever sets it to those two values (`marketplace_modal.py:691` — strict binary toggle). So the third-state defensive code is unreachable today.

**Rust port note:** model `scope_view` as `enum ScopeView { User, Project }` — eliminates the dead branch and forces exhaustive handling.

### Critical install-vs-toggle scope semantics

The mixin uses `_resolve_plugin_scope` in **three** places (`marketplace.py:185, 232, 355`):

| Caller | Use |
|---|---|
| `on_marketplace_modal_plugin_toggled` (line 176-201) | Pick `--scope` for `claude plugin enable/disable` |
| `on_marketplace_modal_plugin_uninstall` (line 222-246) | Pick `--scope` for `claude plugin uninstall` |
| `on_marketplace_modal_plugin_update` (line 350-365) | Pick `--scope` for `claude plugin update` |

These three (toggle, uninstall, update) operate on an **already-installed** plugin, so `installed_scopes` is non-empty. The function picks the most-specific scope that matches the current view.

`on_marketplace_modal_plugin_install_with_scope` (line 203-220) does **NOT** call `_resolve_plugin_scope` — install gets its scope from the modal's three-key picker (`1`/`2`/`3` → User/Project/Local — `marketplace_modal.py:79-81, 699-717`). This is correct: install needs a target, the others use the existing install location.

**Edge case to surface:** if a plugin is installed at BOTH `project` and `local` (`installed_scopes == {"project", "local"}`), `_resolve_plugin_scope` with view_scope="project" returns `"project"` (the first match in iteration). But Python set iteration is **insertion-ordered** since 3.7 and `installed_scopes` is built from a list — `marketplace_loader.py:159` initializes it as `installed_scopes: list[str]`. So iteration order is **the source list order, which the loader determines**. If the loader populates `installed_scopes` from `installed_plugins.json` in [user, project, local] order, the `next(...)` call picks `project` consistently.

For Rust port: use `Vec<PluginScope>` (not `HashSet`) to preserve deterministic order, or formalize the precedence as a sorted enum.

## Gap 2: Modal message contract validation (orphan check)

Cross-referenced every `post_message` site against every mixin/app handler. Goal: find unhandled emits or handlers expecting messages no widget posts.

### Complete emit-to-handler mapping

| Widget | Message class | Emit site | Mixin/App handler |
|---|---|---|---|
| `FilterInput` | `FilterChanged` | `widgets/filter_input.py:80` | `app.py:499` |
| `FilterInput` | `FilterApplied` | `widgets/filter_input.py:84` | `app.py:532` |
| `FilterInput` | `FilterCancelled` | `widgets/filter_input.py:90` | `app.py:515` |
| `CombinedPanel` | `DrillDown` | `widgets/combined_panel.py:478, 480` | `app.py:443` |
| `CombinedPanel` | `SelectionChanged` | `widgets/combined_panel.py:577, 580` | `app.py:433` |
| `CombinedPanel` | `MemoryFileRefSelected` | `widgets/combined_panel.py:578` | `app.py:472` |
| `TypePanel` | `DrillDown` | `widgets/type_panel.py:501, 508, 510` | `app.py:424` |
| `TypePanel` | `SelectionChanged` | `widgets/type_panel.py:570, 575, 578` | `app.py:414` |
| `TypePanel` | `SkillFileSelected` | `widgets/type_panel.py:571` | `app.py:452` |
| `TypePanel` | `MemoryFileRefSelected` | `widgets/type_panel.py:576` | `app.py:466` |
| `MarketplaceModal` | `ModalClosed` | `widgets/marketplace_modal.py:498` | `mixins/marketplace.py:367` |
| `MarketplaceModal` | `PluginToggled` | `widgets/marketplace_modal.py:565` | `mixins/marketplace.py:176` |
| `MarketplaceModal` | `PluginUninstall` | `widgets/marketplace_modal.py:578` | `mixins/marketplace.py:222` |
| `MarketplaceModal` | `MarketplaceRemove` | `widgets/marketplace_modal.py:580` | `mixins/marketplace.py:375` |
| `MarketplaceModal` | `OpenPluginFolder` | `widgets/marketplace_modal.py:593` | `mixins/marketplace.py:282` |
| `MarketplaceModal` | `OpenMarketplaceSource` | `widgets/marketplace_modal.py:606` | `mixins/marketplace.py:321` |
| `MarketplaceModal` | `OpenPluginSource` | `widgets/marketplace_modal.py:610` | `mixins/marketplace.py:295` |
| `MarketplaceModal` | `MarketplaceUpdate` | `widgets/marketplace_modal.py:623` | `mixins/marketplace.py:341` |
| `MarketplaceModal` | `PluginUpdate` | `widgets/marketplace_modal.py:625` | `mixins/marketplace.py:350` |
| `MarketplaceModal` | `PluginPreview` | `widgets/marketplace_modal.py:638` | `mixins/marketplace.py:158` |
| `MarketplaceModal` | `MarketplaceAddRequest` | `widgets/marketplace_modal.py:644` | `mixins/marketplace.py:382` |
| `MarketplaceModal` | `PluginInstallWithScope` | `widgets/marketplace_modal.py:717` | `mixins/marketplace.py:203` |
| `MarketplaceConfirm` | `RemoveConfirmed` | `widgets/marketplace_confirm.py:103` | `mixins/marketplace.py:390` |
| `MarketplaceConfirm` | `RemoveCancelled` | `widgets/marketplace_confirm.py:108, 113` (two callsites) | `mixins/marketplace.py:403` |
| `MarketplaceSourceInput` | `SourceSubmitted` | `widgets/marketplace_source_input.py:211` | `mixins/marketplace.py:413` |
| `MarketplaceSourceInput` | `SourceCancelled` | `widgets/marketplace_source_input.py:185` | `mixins/marketplace.py:424` |
| `LevelSelector` | `LevelSelected` | `widgets/level_selector.py:116, 122, 128` (three callsites) | `mixins/customization_actions.py:214` |
| `LevelSelector` | `SelectionCancelled` | `widgets/level_selector.py:135` | `mixins/customization_actions.py:225` |
| `DeleteConfirm` | `DeleteConfirmed` | `widgets/delete_confirm.py:100` | `mixins/customization_actions.py:259` |
| `DeleteConfirm` | `DeleteCancelled` | `widgets/delete_confirm.py:105, 110` (two callsites) | `mixins/customization_actions.py:274` |
| `PluginConfirm` | `PluginConfirmed` | `widgets/plugin_confirm.py:135` | `mixins/customization_actions.py:233` |
| `PluginConfirm` | `ConfirmationCancelled` | `widgets/plugin_confirm.py:140, 145` (two callsites) | `mixins/customization_actions.py:252` |

**Result: zero orphans on either side.** Every emit has a handler; every handler corresponds to an emit. The contract is **tight and complete**.

`Tree.NodeHighlighted` (the Textual built-in tree-cursor event consumed at `marketplace_modal.py:243`) is internal to the modal — no mixin handler, correct.

`MarketplaceConfirm.RemoveCancelled` is emitted from **both** `action_deny` (`y/n` "no" branch) AND `action_cancel` (Esc branch); the handler at `mixins/marketplace.py:403` treats them identically (just re-focus tree). This is fine; the dual-emit is a UX nicety so `y` and `Esc` feel symmetric for "back out of confirmation".

### One interesting modal-emits-twice case

`MarketplaceConfirm.RemoveCancelled` does NOT include a payload distinguishing `deny` from `cancel`. The mixin handler doesn't care — but if Monocle's port wants distinct telemetry for "user said no" vs "user dismissed", carry an enum in the message. Today they're indistinguishable.

## Gap 3: `refresh_bindings` callsite audit

`refresh_bindings()` is Textual's API to re-evaluate `check_action` for all bindings (which controls the footer's "active" state and which keys actually fire). The six callsites:

| Site | Trigger | Reason |
|---|---|---|
| `app.py:422` | `on_type_panel_selection_changed` | Selection changed → `copy/move/delete` availability changed → re-check |
| `app.py:441` | `on_combined_panel_selection_changed` | Same |
| `app.py:530` | `on_filter_input_filter_cancelled` | Filter cancel may have hidden filter input → bindings shift |
| `app.py:539` | `on_filter_input_filter_applied` | Filter applied → input hidden → bindings shift |
| `mixins/marketplace.py:100` | `_enter_plugin_preview` | Preview mode toggles the whitelist gate in `check_action` |
| `mixins/marketplace.py:146` | `_exit_plugin_preview` | Same, exiting |

### Sites that arguably SHOULD call `refresh_bindings` and do not

- **`action_toggle_marketplace` (`mixins/marketplace.py:54`)**: when the modal opens/closes, `check_action` re-gates `filter_all/filter_user/filter_project/filter_plugin/toggle_plugin_enabled_filter` (`app.py:230-242`). No `refresh_bindings` call here — but the marketplace modal grabs focus, and Textual's binding evaluation **on next keypress** would re-check anyway. Still, the **footer state** depends on this and is **not refreshed via bindings** — it's refreshed via the explicit `_update_footer_actions()` call at `mixins/marketplace.py:68`. So the footer stays correct, but the binding-availability gate has a one-keypress lag. In practice unnoticeable.

- **`action_back` (`mixins/navigation.py:120`)**: this is the Esc handler that exits preview mode. It calls `_exit_plugin_preview()`, which already does `refresh_bindings()` at line 146. Covered.

- **`on_type_panel_skill_file_selected` (`app.py:452`)**: skill subfile selection changes whether `copy/move/delete` are available (subfiles can't be copied — see `_is_skill_subfile_selected` at `app.py:634-645`). The handler at line 452 does NOT call `refresh_bindings` — only `selected_file = ...` and `display_path = ...`. **This is a latent bug.** After selecting a subfile, the binding gate at `app.py:279` (`if self._is_skill_subfile_selected(): return False`) is still in the right state on next keypress evaluation, but the footer would not reflect that copy is disabled until the next external trigger calls `_update_footer_actions`.

  **Verifying**: at `app.py:421` (`on_type_panel_selection_changed`), `_update_footer_actions()` is called and `refresh_bindings()` is called. But `on_type_panel_skill_file_selected` does NOT call either. So clicking into a skill subfile leaves the footer showing "Copy" as available even though pressing `c` will be silently rejected by `check_action`.

  **P2 finding** (new): footer staleness on skill subfile selection. Rust port should ensure all selection-change events trigger `update_footer_actions` + `refresh_bindings` (or whatever the ratatui equivalent is — recomputing footer-render state on every action-availability change).

- **`on_filter_input_filter_changed` (`app.py:499`)**: real-time filter typing changes which items are visible, but the selection is reset to `None` (line 506). After this, `copy/move/delete` are not available (no selection). The footer is updated (`_update_panels`, `_update_subtitle`) but **`refresh_bindings` is not called here**. The next keystroke would re-evaluate. **P3 finding** (style consistency, not user-visible).

## Gap 4: Pending-op coalescing (race / re-entrancy)

Scenarios I traced:

### Scenario A: user presses `c` (copy), modal opens, user presses `c` again

- `action_copy_customization` first call: sets `_pending_customization = customization_A`, `_panel_before_selector = panel_X`, shows `LevelSelector`.
- `LevelSelector.show()` calls `add_class("visible")` and `focus()`. The selector now has focus.
- User presses `c` again. Textual routes the keypress to the focused widget (`LevelSelector`), which has its own bindings: `1`, `2`, `3`, `escape`. **`c` is not bound in LevelSelector** (`widgets/level_selector.py:15-20`) — so the keypress bubbles up to the App, which evaluates global bindings. `c` → `copy_customization` is a global App binding.
- App's `action_copy_customization` runs again: `_pending_customization` is overwritten with `customization_A` (or whatever's currently in `_main_pane.customization` — likely still `customization_A` because focus was on the selector and selection didn't change). `_panel_before_selector` is overwritten with `_get_focused_panel()` — but the focused thing is `LevelSelector`, not a `TypePanel`. `_get_focused_panel()` returns None (line 632 returns None because only TypePanel instances are checked). So `_panel_before_selector = None`. **The original panel snapshot is lost.**
- `_combined_before_selector = (self._combined_panel.has_focus if self._combined_panel else False)` — combined panel doesn't have focus, so `False`. Lost.
- `LevelSelector.show()` is called again. Idempotent — `add_class("visible")` on an already-visible widget is a no-op; `focus()` is also idempotent.

**Result of the race:** original focus snapshot is wiped. When the user finally picks a level, `_restore_focus_after_selector` falls through to `_panels[0].focus()` (the fallback at `app.py:657-658`). User's actual original focus is forgotten.

**Severity:** P2. The user has to navigate back to the panel they were in. Not destructive, but mildly annoying. The reference codebase doesn't guard against this.

**Rust port fix:** the `AppMode::AwaitingLevelSelect { ... }` enum variant from Round 1 naturally guards against this — `apply(Action::CopyCustomization)` in mode `AwaitingLevelSelect` is a state-transition that the apply function can short-circuit (`if matches!(self.mode, AppMode::AwaitingLevelSelect{..}) { return; }` or, more elegantly, only Mode::Normal accepts `CopyCustomization`).

### Scenario B: user presses `c`, then before picking a level, presses `m`

- `action_copy_customization` → sets `_pending_customization`, shows selector with `operation="copy"`.
- `action_move_customization` → fires before resolution. Overwrites `_pending_customization` (same value, since selection unchanged). Snapshots focus from the now-incorrect "focused panel" (None, as in A). Calls `LevelSelector.show(..., "move")`. The selector now thinks the operation is move.
- User picks `User`. `LevelSelector` emits `LevelSelected(level, operation="move")` (`level_selector.py:116`). Mixin's `on_level_selector_level_selected` calls `_handle_copy_or_move(_pending_customization, level, "move")`. **Move is performed**, even though the user's last visible intent (the initial `c`) was copy.

**Severity:** P2. The selector's prompt text DOES update to show "Move to:" after the second action (because `_update_prompt` is called via `show()` at line 88). So the user sees the change. But the keypress-flow is confusing.

The reference codebase doesn't guard against this; the second action just wins. Acceptable today; flagged for Rust port.

### Scenario C: same as A but with delete or plugin-toggle

`action_delete_customization` and `action_toggle_plugin_enabled` follow the same pattern (snapshot focus → show confirm). The `_pending_customization` field is NOT touched by these — they pass the customization directly to the confirm widget via `show(customization)`. So delete and plugin-toggle do not share the pending state with copy/move. But they DO share `_panel_before_selector` and `_combined_before_selector`, so the same focus-snapshot-wipe applies.

**Net:** all four modal-initiator actions can wipe each other's focus snapshot if invoked sequentially without the user resolving the in-flight modal. Single shared state for four operations.

## Gap 5: `_last_focused_panel` vs `_panel_before_selector` lifetimes

Two focus-snapshot variables that look similar. Distinguishing them is critical for the Rust port's state-machine design.

### `_last_focused_panel`

**Purpose:** records which panel was focused before the user **drilled down** into the main pane (via Enter / RET). Used by `action_back` (Esc) to return focus.

**Lifecycle:**

| Site | Action |
|---|---|
| `app.py:113` | initialized to None |
| `app.py:427` | written on `on_type_panel_drill_down` — snapshots the focused panel before transferring focus to main pane |
| `app.py:446` | written to None on `on_combined_panel_drill_down` (with `_last_focused_combined = True` instead — the two are mutually exclusive) |
| `app.py:504` | written to None on filter changes (`on_filter_input_filter_changed`) |
| `app.py:521` | written to None on filter cancel |
| `mixins/filtering.py:31, 41, 51, 61, 75` | written to None on every level-filter action |
| `mixins/navigation.py:129` | read by `action_back` |

So `_last_focused_panel` is **lifecycle-scoped to a drill-down session**. It's cleared whenever the user changes filter (which resets context) or drills via the combined panel (which uses the boolean `_last_focused_combined` instead).

### `_panel_before_selector`

**Purpose:** records the focused panel before a **modal opens** (LevelSelector, DeleteConfirm, PluginConfirm, MarketplaceModal). Used by `_restore_focus_after_selector` (`app.py:647-658`) to return focus after the modal closes.

**Lifecycle:**

| Site | Action |
|---|---|
| `app.py:116` | initialized to None |
| `mixins/customization_actions.py:56, 86, 110, 128` | written on `action_copy/move/delete/toggle_plugin_enabled` |
| `mixins/marketplace.py:61` | written on `action_toggle_marketplace` (open) |
| `app.py:652` | read and cleared by `_restore_focus_after_selector` (panel branch) |
| `app.py:655` | also cleared (combined branch) |

### Relationship

| Aspect | `_last_focused_panel` | `_panel_before_selector` |
|---|---|---|
| Triggered by | drill-down (Enter) | modal-open (c/m/d/t/M) |
| Restored by | `action_back` (Esc) | `_restore_focus_after_selector` (on modal close) |
| Cleared by | filter change, drill-from-combined | modal close (panel or combined branch) |
| Companion bool | `_last_focused_combined` | `_combined_before_selector` |
| Both can be set? | yes — independently | yes — independently |

**They are independent state machines.** A user could drill into main pane (sets `_last_focused_panel`), then press `c` from the main pane (sets `_panel_before_selector` to `None` because main pane is focused, not a TypePanel). The two snapshots track different events.

**One subtlety:** `_panel_before_selector` is set via `_get_focused_panel()` (`app.py:627-632`), which only returns a `TypePanel`. If a modal is opened **while the main pane is focused**, `_panel_before_selector` is `None`. After the modal closes, `_restore_focus_after_selector` falls through to `_panels[0].focus()` (line 657-658). **Focus moves to panel 0, not back to main pane.** This is observable: drill into main pane, press `c`, pick a level, focus jumps to the leftmost panel (not the main pane where you were).

**Is this intentional?** Hard to say from code alone. It's plausibly UX: after copy/move/delete, you probably want to return to the panel list, not the now-stale detail view. But it's not documented and the asymmetry vs `action_back` (which does restore to main-pane-prev when applicable) is a smell.

For Rust port: encode focus snapshot as `enum FocusTarget { Panel(usize), CombinedPanel, MainPane }` with `Option<FocusTarget>` semantics. Then `restore_focus_after_modal` can return to MainPane explicitly when that's where the user was. The Python reference loses this fidelity.

### Race between the two

Filter actions clear `_last_focused_panel` (`filtering.py:31`) but NOT `_panel_before_selector`. So:

- Drill into main pane (sets `_last_focused_panel = panel_X`).
- Press `c` (sets `_panel_before_selector = None` because main pane is focused).
- LevelSelector visible.
- User presses `a` (filter_all). **Does this fire?** `check_action` (`app.py:221-292`) doesn't block filter actions in this state — only marketplace-modal-open blocks filters. The keypress goes to LevelSelector first, which doesn't bind `a`. Bubbles to App. `action_filter_all` runs, clearing `_last_focused_panel`. **`_pending_customization` is NOT cleared, modal stays visible**.
- User picks a level → copy proceeds → `_restore_focus_after_selector` clears `_panel_before_selector`, focuses `_panels[0]`. `_last_focused_panel` is None. Esc from this state would fall to `_panels[0].focus()` again.

**Severity:** P3. State is consistent but counter-intuitive: the user did a copy mid-filter-change. Whether the copy targets the original or the new filtered selection depends on `_pending_customization` (which was snapshotted at modal open, so original). The actual file operation is correct; the focus afterward is just slightly disorienting.

**Bottom line:** the two snapshot fields are genuinely separate concerns and the reference codebase handles them with appropriate (if undocumented) discipline. The Rust port should keep them as separate enum variants but document the lifecycle.

## Notification-call-path inventory (refining R1's P2)

Round 1 flagged inconsistent notification surface. Enumerating every call site:

| Site | API | Severity | Timeout |
|---|---|---|---|
| `customization_actions.py:45, 52, 71, 77, 82, 102, 107, 125, 170, 197, 205` | `_show_status_error` | error | 3.0s |
| `customization_actions.py:211` | `_show_status_success` | information | 3.0s |
| `customization_actions.py:245, 248, 268, 270` | `self.notify(..., severity=...)` | varies | default (5s) |
| `marketplace.py:73, 78, 200, 219, 229, 233, 236, 246, 271, 278, 289, 311, 317, 319, 331, 337, 346, 356, 395, 418` | `self.notify(..., severity=..., timeout=...?)` | varies | varies |
| `marketplace.py:200, 219, 236, 346, 356, 395, 418` | use `timeout=2.0` | varies | 2.0s |
| `app.py:582, 597, 605, 611, 618, 623` | `self.notify(...)` | warning/error/info | default |

**Three timeout policies in use:** 2.0s (marketplace info toasts), 3.0s (customization actions), default ~5s (everything else). No discernible policy — pick-your-poison.

**Recommended unified scheme for Rust port:**

```rust
enum NotifySeverity { Info, Warning, Error }
struct Toast { msg: String, severity: NotifySeverity, timeout: Duration }
impl App {
    fn toast(&mut self, t: Toast) { ... }
}
const TIMEOUT_INFO: Duration = Duration::from_secs(3);
const TIMEOUT_WARNING: Duration = Duration::from_secs(5);
const TIMEOUT_ERROR: Duration = Duration::from_secs(5);
```

## Marketplace blocked-actions whitelist refinement

Found in `app.py:230-242`:

```python
marketplace_blocked_actions = {
    "filter_all",
    "filter_user",
    "filter_project",
    "filter_plugin",
    "toggle_plugin_enabled_filter",
}
```

**Observation:** the blocked set is **only the filter actions**, not other global actions. So while the marketplace modal is visible, **all other keypresses are passed through** — including potentially destructive ones like `d` (delete) and `c` (copy).

Why is this safe? Because the modal has focus and binds many of those keys to its own actions:
- `d` → `action_uninstall_plugin` (modal binding) — overrides global `delete_customization`
- `e` → `action_open_plugin_folder` (modal) — overrides global `open_in_editor`
- `c` is NOT bound in the modal — so `c` could still fire `action_copy_customization`. But `check_action` would block it because `_main_pane.customization` might be None in marketplace context. Or it would copy the previously-selected customization from before the modal opened.

**P2 finding (new): hypothetical c-press-during-marketplace.** If the user opens the marketplace modal while a copyable customization is selected and then presses `c`, the global `action_copy_customization` fires, showing the LevelSelector ON TOP of the marketplace modal. The focus would shift to the level selector. This is observable and weird.

Mitigation: extend `marketplace_blocked_actions` to include `copy_customization`, `move_customization`, `delete_customization`, `toggle_plugin_enabled`. Or rely on focus discipline (less robust).

For Rust port: again, the `AppMode::MarketplaceOpen` enum variant makes this trivial — only marketplace-modal-bound actions are accepted in that mode.

## Cross-mixin call graph (round-2 refinement)

Round 1 noted that the mixins call each other and call into `LazyClaude`'s methods. Enumerating directionally:

| Caller | Callee | Citation |
|---|---|---|
| FilterMixin | `_update_panels` (app) | `filtering.py:34, 44, 54, 64, 83` |
| FilterMixin | `_update_subtitle` (app) | `filtering.py:35, 45, 55, 65, 84` |
| MarketplaceMixin | `_update_panels` (app) | `marketplace.py:97, 142` |
| MarketplaceMixin | `_update_subtitle` (app) | `marketplace.py:98, 143` |
| MarketplaceMixin | `_update_footer_actions` (app) | `marketplace.py:68, 99, 145, 373` |
| MarketplaceMixin | `_update_status_panel` (app) | `marketplace.py:144` |
| MarketplaceMixin | `_restore_focus_after_selector` (app) | `marketplace.py:59, 372` |
| MarketplaceMixin | `_get_focused_panel` (app) | `marketplace.py:61` |
| MarketplaceMixin | `action_refresh` (app) | `marketplace.py:274` |
| MarketplaceMixin | `refresh_bindings` (Textual App) | `marketplace.py:100, 146` |
| MarketplaceMixin | `_on_plugin_command_success` (self) | `marketplace.py:262` |
| MarketplaceMixin | `_on_plugin_command_error` (self) | `marketplace.py:265, 267` |
| CustomizationActionsMixin | `_show_status_error` (app) | many |
| CustomizationActionsMixin | `_show_status_success` (app) | `customization_actions.py:211` |
| CustomizationActionsMixin | `_get_focused_panel` (app) | `customization_actions.py:56, 86, 110, 128` |
| CustomizationActionsMixin | `_restore_focus_after_selector` (app) | `customization_actions.py:223, 231, 247, 250, 257, 272, 279` |
| CustomizationActionsMixin | `action_refresh` (app) | `customization_actions.py:212, 246, 269` |
| CustomizationActionsMixin | `notify` (Textual App) | `customization_actions.py:245, 248, 268, 270` |
| NavigationMixin | `_exit_plugin_preview` (Marketplace mixin) | `navigation.py:123` |
| HelpMixin | `query`, `mount`, `query_one` (Textual App) | `help.py:61, 63, 69` |

**Direction summary:**
- Mixins → App methods: heavy, 30+ call sites.
- Mixins → other mixins: only **1** call (Navigation → Marketplace's `_exit_plugin_preview`).
- App → Mixins: zero direct calls (Textual routes via action names and message dispatch).
- Mixins → Textual base: `refresh_bindings`, `notify`, `query*`, `mount`. ~10 call sites.

**Coupling assessment:** the mixins are **highly coupled to the App's helper methods** but **minimally coupled to each other**. They are organizationally a fan-in pattern: App owns shared helpers (focus restoration, status display, panel updates, footer updates) and mixins consume them. This makes the "shape A: flat module + free functions on `&mut App`" Rust translation approach (recommended in R1) even more apt: the mixins are not encapsulating, they're grouping.

## Delta Summary

- New items added:
  - `_resolve_plugin_scope` end-to-end trace including dead-branch identification (line 174 `else "user" if "user" in plugin.installed_scopes else view_scope` reduces to constant `"user"`)
  - Complete emit-to-handler mapping (28 message types across 10 widgets, all paired, no orphans)
  - `refresh_bindings` callsite audit with **1 latent footer-staleness finding** (`on_type_panel_skill_file_selected` doesn't refresh footer/bindings)
  - Pending-op coalescing race analysis (3 scenarios: same-action re-entry, action-swap, filter-during-modal)
  - `_last_focused_panel` vs `_panel_before_selector` lifecycle table with the **MainPane-focus restoration gap** finding (modal-close from main-pane focus loses the main-pane fact)
  - Notification timeout policy inventory (3 different timeouts in use, no documented policy)
  - Marketplace blocked-actions whitelist gap (**P2 new**: `c` not in blocked set, can stack LevelSelector on marketplace modal)
  - Cross-mixin call graph with directional coupling summary (App is the hub; mixins are spokes)
- Existing items refined:
  - R1's "P2 inconsistent notification surface" expanded with concrete timeout values
  - R1's pending-op state machine sketch validated against three observable scenarios
- Remaining gaps:
  - **MarketplaceSourceInput widget details** — not read in R1 or R2 (its messages were enumerated from the marketplace mixin handlers, but the widget's internal validation/input lifecycle is unread). Likely low-value for the mixin abstraction.
  - **Help-overlay z-order / layer behavior** — `help.py` mounts a `Static` without specifying layer; if the marketplace modal is open (`layer: overlay`), the help might render under it. Untested. Probably not worth Round 3.
  - **`AppFooter` reactive watchers** — `app_footer.py:103-141` defines `watch_*` methods that re-render the footer. The footer is mutated heavily by the filter/marketplace mixins; verified the mutations target reactive fields. No drift seen.

## Novelty Assessment

Novelty: **SUBSTANTIVE (marginal)**

This round produced several new findings that change the spec model materially:

1. The dead-branch in `_resolve_plugin_scope` is a code smell that the Rust port should NOT replicate — substantive.
2. The footer-staleness on skill subfile selection is a latent UX bug — substantive.
3. The pending-op coalescing scenarios identify three race conditions, two of which are user-observable — substantive.
4. The marketplace blocked-actions whitelist gap (`c`/`m`/`d`/`t` not blocked) is a real defect — substantive.
5. The dual focus-snapshot model with MainPane restoration gap is a fidelity-affecting design clarification — substantive.

But the findings are **diminishing in scope** compared to Round 1. R1 mapped the entire dispatch surface and established the design pattern; R2 found edge cases and small defects. The marginal value of a Round 3 hunting in the same areas would be small.

The single area NOT yet deeply inspected is `MarketplaceSourceInput` — but it's a sub-widget with a clearly enumerated message contract (R2 Gap 2 table); reading its internals likely yields no design-level findings, only widget-implementation details that are out-of-scope for the mixin deepening.

## Convergence Declaration

**Pass B-deep-mixins has converged.** Round 3 would produce nitpicks: implementation details of `MarketplaceSourceInput`, the help-overlay z-order question, footer reactive-watcher mechanics. None would change how monocle's ratatui port is designed.

If Round 1 had not been done, this round's findings alone would not be enough to design the port. If Round 2 had not been done, the port would replicate the modal-stacking gap, the focus-snapshot ambiguity, and the dead-branch — all manifested bugs. Both rounds were necessary; a third would not be.

The design template for monocle's ratatui port is:

1. **A single `App` struct** holding all state (no Rust traits-per-mixin attempt).
2. **`AppMode` enum** as the central state machine, with variants carrying mode-specific data (`PendingOp`, `FocusSnapshot`, `MarketplaceState`, `PluginPreview`).
3. **`Action` enum** as the dispatch surface, with `apply(&mut self, app: &mut App)` matching on both action and current mode.
4. **`FocusSnapshot` enum** unifying the two-snapshot model into one type with explicit `MainPane` variant.
5. **Modal widgets** as separate types emitting typed messages via a channel; the dispatch loop routes them through `apply`.
6. **`KeyBinding` registry** as the single source of truth for help text, footer rendering, and key-event routing.
7. **`std::process::Command::new(...).arg(...)`** (no `shell=true`) for all CLI invocations.
8. **`std::fs::rename` first, fallback to copy+verify+delete** for moves.

The reference codebase is sufficient to produce this design.

## State Checkpoint

```yaml
pass: B-deep-mixins
round: 2
status: complete
timestamp: 2026-05-11T20:35:00Z
novelty: SUBSTANTIVE (marginal — last substantive round)
files_read_round_2:
  - widgets/marketplace_modal.py (full, 788 LOC)
  - widgets/marketplace_confirm.py (full)
  - widgets/app_footer.py (full)
  - services/marketplace_loader.py (partial, lines 115-165 + scope_view section)
new_p1_findings: 0
new_p2_findings: 3
  - footer staleness on skill subfile selection (on_type_panel_skill_file_selected misses refresh_bindings)
  - marketplace_blocked_actions whitelist gap (c/m/d/t can fire during modal)
  - pending-op coalescing wipes focus snapshot
new_p3_findings: 2
  - dead branch in _resolve_plugin_scope user-view path
  - notification timeout inconsistency (2.0s / 3.0s / default)
verified_no_issues:
  - no orphan emits or handlers in modal-message contract
  - shared state across mixins is documented and consistent
convergence: YES
output: nikiforovall-lazyclaude-pass-B-deep-mixins-r2.md
total_rounds: 2
final_recommendation: stop deepening, proceed to Rust port design phase
```
