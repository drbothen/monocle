# Pass B (deep) — Help Overlay, Filter, Telescope Conventions (round 1)

## The `?` help — what makes it "telescope-style"
`/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/controllers/options_menu_action.go:13-79`

When the user presses `?` (`Universal.OptionMenu` / `OptionMenuAlt1`), lazygit:
1. Snapshots the current context.
2. Calls `GetInitialKeybindingsWithCustomCommands()` to get *every* registered binding (including custom).
3. Buckets them into three sections — local (binding's `ViewName` matches current context's view), global (`ViewName == ""` or `Tag == "global"`), navigation (`Tag == "navigation"`) — see line 64-77.
4. De-duplicates by `GetDescription()` (line 78,83).
5. Creates a Menu with three `MenuSection`s ("Local", "Global", "Navigation") wired through `appendBindings(local, &MenuSection{...})` (line 44-46).
6. Opens the menu with `AllowFilteringKeybindings: true` and `KeepConflictingKeybindings: true` (line 53-54). This is what makes typing `@` invoke key-search-mode rather than label-search.

That's the recipe. **Telescope-style** in this context means:
- A single menu listing every binding available *now*.
- Filterable as you type (immediate fuzzy match).
- Section-grouped for orientation.
- `@`-prefix to filter by keybinding rather than label.
- Each entry knows its `Key`, `Description`, `Tooltip`, `OpensMenu` indicator, and `DisabledReason` (struck through if disabled).

This is the convention monocle should adopt verbatim. The `?` menu is the discoverability cornerstone of the lazy* aesthetic — without it, users cannot find out what keys do.

## `KeepConflictingKeybindings: true` — why
`options_menu_action.go:54` — normally a menu strips menu-item keybindings that conflict with essentials (Enter, Escape, j, k). When showing the keybindings *cheatsheet*, we want to display them all even if they clash, because the user is *looking at* what the keys mean. The menu_context's `keybindingsTakePrecedence` flag flips to false in this case (see BC-DRAFT-023).

## Filter ('/') — the lazy\* signature filter
The `/` keybinding is per-context (it's the per-context controller's binding, not global): each filterable context registers its own filter binding via `FilterControllerFactory.Create(ctx)` (`pkg/gui/controllers/filter_controller.go:46`). The factory loop in `controllers.go:201-203` iterates `gui.c.Context().AllFilterable()` and attaches a `FilterController` to each.

The filter behaviour:
1. Press `/` → `OpenFilterPrompt(self.context)` → `SearchHelper.OpenFilterPrompt(ctx)` (`search_helper.go:32`).
2. `SearchHelper` sets `state.Context = ctx`, sets search-prefix view content to `ctx.FilterPrefix(tr)` (line 39), pushes the `Search` context.
3. While typing, each `OnPromptContentChanged` (line 223) calls `ctx.SetFilter(searchString, useFuzzy)` — the filter applies *immediately* per keystroke. Cursor goes to top (`SetSelection(0)`).
4. Enter → `ConfirmFilter` (line 124): pushes the filter onto history, pops Search context (returns focus to the original).
5. Esc → `CancelPrompt` (line 178): clears filter, pops Search context.

**No debouncing**. The fuzzy lib + small list sizes keep this responsive.

## Filter mode is per-list, not global
This is a critical detail. The user can have an active filter on Branches (filtering to feature/*) and a separate active filter on Commits (filtering to "WIP:"). They're independent because each filterable context owns its own `FilteredList` state (`pkg/gui/context/filtered_list.go:12`).

By contrast, the global `FilteringMenu` (`Universal.FilteringMenu`, default `<space>` after `M`) is a *separate concept*: it filters the entire repo view to only show commits/files touching a path. This is git-specific and out of scope; monocle doesn't need it.

## Fuzzy vs substring
`pkg/gui/types/common.go` references `UserConfig.Gui.FilterMode ∈ {"substring", "fuzzy"}` via `UseFuzzySearch()` (`user_config.go:177-182` — `filterMode` field). Substring is faster; fuzzy is more forgiving.

## Search trait, distinct from filter
`pkg/gui/context/search_trait.go:71` adds search to non-list views: searching highlights matches *without* hiding non-matches. Used by `main` (diff viewer) and the patch explorer. The same `/` keybinding triggers either filter or search depending on whether the focused context implements `IFilterableContext` or `ISearchableContext`.

The two flows share the same `SearchHelper` and the same `Search` prompt view; they differ only in `state.SearchType()` and which callback fires on Confirm.

## Search history and `<c-p>`/`<c-n>` style navigation
`search_helper.go:186-204` — `ScrollHistory(increment)` lets the user cycle through previous search/filter strings. The `SearchPromptController` (`search_prompt_controller.go:24-43`) wires `Universal.PrevItem` and `Universal.NextItem` (default `<up>` and `<down>`) to history scroll while the prompt is open.

History is per-context: each `IFilterableContext` and `ISearchableContext` keeps its own `*utils.HistoryBuffer[string]` (`pkg/gui/types/context.go:122-126`).

## "Frame colour while filtering" — the visual cue
`search_helper.go:319-326` — when filter or search is active on a context, the gui's `SelFgColor` and `SelFrameColor` get set to `theme.SearchingActiveBorderColor` (typically `cyan`). When the filter clears, colours revert to `theme.ActiveBorderColor` (typically `green`). This is the famous "yellow/red border while searching" UX.

## DisplayFilterStatus and DisplaySearchStatus
`search_helper.go:67-88` — when a filter is *committed* (Enter pressed), the bottom-line shows `matches for 'foo' [press Esc to exit filter mode]` (line 77). The search variant shows `1 of 5` (line 87).

## Translation to monocle
Monocle inherits the lazyclaude `?` menu *and* `/` filter conventions. The verbatim mapping:

| lazygit feature | monocle equivalent |
|---|---|
| `?` menu with section headers | `KeybindingsMenu` with `MenuSection { local, global, navigation }` |
| `@` prefix in menu filter | strip prefix and switch `MenuFilterMode::ByKey` |
| `/` filter per-list | each `ListContext` exposes `set_filter(&str, fuzzy: bool)` |
| Frame colour during search | swap `theme.active_border` for `theme.searching_border` in the focused view's `Block::style` |
| History buffer per context | `History<String>` stored on the context |
| Filter mode config | `monocle.toml: ui.filter_mode = "fuzzy" | "substring"` |

The `useFuzzySearch` config split should be preserved — fuzzy is great for most lists but substring is occasionally what power-users want.

## Delta summary
- New items: 5 (`KeepConflictingKeybindings` rationale, frame-colour cue, per-context history buffers, the filter/search trait disambiguation, telescope-recipe enumeration).
- Refined: `?` menu wiring, `/` per-context attach via factory loop.
- Remaining gaps: how `Universal.FilteringMenu` differs from `/` (global path filter — out of scope).

## Round assessment
SUBSTANTIVE — the telescope recipe is now precisely documented. Lane CONVERGED.
