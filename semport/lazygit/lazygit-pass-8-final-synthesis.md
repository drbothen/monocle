# lazygit — Final Synthesis (Phase C, SCOPED ingest for monocle)

## Summary

lazygit is the original "lazy\*" TUI: a single-binary Go terminal application with a panel-based interface, context-aware keybindings, telescope-style help, fzf-style filter, popup-stacking modals, user-extensible custom commands, hot-reload theming, and layered config (defaults + global + per-repo). The architecture is a god-struct (`Gui`) orchestrating a stack of `Context`s, each with attached `Controller`s that contribute `Binding`s scoped by view name. The popup system uses five primitives (Alert, Confirm, Menu, Prompt, WithWaitingStatus) over two underlying contexts (Confirmation, Prompt) sized by a stateless layout helper recomputed each tick. Monocle should inherit the lazy\* binding signature, the telescope `?` menu, the `/` filter convention, the layered config loader with focus-trigger reload, and the docked-bottom log-viewer-with-expand pattern, while improving on three known gaps (popup queueing, declarative guards, theme as struct not globals).

## Scope Statement

**Ingested (deepened):**
- `pkg/gui/` TUI core (excluding controllers/* git-business-logic bodies)
- `pkg/gui/context/`
- `pkg/gui/types/`
- `pkg/gui/popup/`
- `pkg/gui/services/custom_commands/`
- `pkg/gui/controllers/helpers/` cross-cutting helpers (search, confirmation, window-arrangement, mode, app-status)
- `pkg/gui/style/`, `pkg/theme/`
- `pkg/config/` (app_config.go, user_config.go schemas, keynames.go)
- `pkg/gui/extras_panel.go`, `pkg/gui/command_log_panel.go`, `pkg/gui/options_map.go`

**Skipped (scoped out):**
- `pkg/commands/*` — git command construction and shell execution
- `pkg/commands/git_commands/*`, `pkg/commands/oscommands/*` — git semantics
- `pkg/integration/` — full TUI integration tests against git fixtures
- `vendor/`, `docs/`, `docs-master/`, `demo/`
- GitHub/GitLab specific integration code
- `pkg/snake/`, `pkg/i18n/` (translation strings only, not deepened)
- `pkg/cheatsheet/`, `pkg/jsonschema/`, `pkg/tasks/`

**Rationale:** monocle is not a git tool; it inherits the TUI conventions only. Git mechanics, file diffing, rebase logic, and PR fetching are entirely irrelevant to monocle's domain.

## Snapshot

| Field | Value |
|---|---|
| Repo | `/Users/jmagady/Dev/monocle/.reference/lazygit/` |
| HEAD | `c4935036` ("Remove the invitation to submit PRs from the issue template (#5603)") |
| Branch | `master` |
| Files (total) | 2,169 |
| Go LOC scope-relevant | ~26,500 (gui core + config + popup + custom-commands + style + theme + extras + helpers) |
| Go LOC repo-wide | substantially larger (controllers/* + commands/* push past 80k) |
| Module | `github.com/jesseduffield/lazygit`, Go 1.25.0 |
| Substrate | `pkg/gocui/` fork (8,011 LOC, 20 files), `lazycore/pkg/boxlayout`, `tcell/v3` |
| Entry | `/Users/jmagady/Dev/monocle/.reference/lazygit/main.go` |

## TUI Architecture

### Component layering (high to low)

| Layer | Responsibility | Key types |
|---|---|---|
| Gui | God-struct orchestrator | `Gui`, `GuiRepoState`, `RepoStateMap`, `BackgroundRoutineMgr` |
| ContextMgr | Panel/popup stack with kind-aware push/pop | `ContextMgr.ContextStack`, `ContextTree` |
| Contexts | Panel-with-state primitives | `BaseContext`, `MenuContext`, `PromptContext`, `WorkingTreeContext`, etc. |
| Controllers | Per-context binding registration | `GlobalController`, `MenuController`, `FilterController`, `SearchController` |
| Helpers | Cross-cutting services | `SearchHelper`, `ConfirmationHelper`, `WindowArrangementHelper`, `ModeHelper` |
| Popup service | Confirm/Alert/Prompt/Menu/Toast/WithWaitingStatus surface | `PopupHandler` |
| Custom commands service | User-extensible bindings | `Client`, `HandlerCreator`, `KeybindingCreator` |
| gocui substrate | View, key event loop, cell rendering | `gocui.Gui`, `gocui.View` |

### Context kinds (the foundational enum)

`/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/types/context.go:14-35`
```
SIDE_CONTEXT       // left-rail panels; ≤1 on the stack
MAIN_CONTEXT       // right-side main panel
PERSISTENT_POPUP   // returnable popups (search, commit-message)
TEMPORARY_POPUP    // single-shot popups (confirm, menu, prompt)
EXTRAS_CONTEXT     // command log scrollback
GLOBAL_CONTEXT     // synthetic, holds global bindings
DISPLAY_CONTEXT    // render-only views (status bar, spacers)
```

### Layout pattern

`pkg/gui/controllers/helpers/window_arrangement_helper.go:124-498` builds a `boxlayout.Box` tree purely from `WindowArrangementArgs{Width, Height, ScreenMode, CurrentWindow, SplitMainPanel, …}` and returns `map[windowName]Dimensions`. `pkg/gui/layout.go:13-207` runs this every render tick and reassigns each view's geometry. Window-vs-view-vs-context is a three-level abstraction: multiple contexts can render to the same view; multiple views can occupy the same window through tab-switching (`TabView`).

### Window/view/context trichotomy

| Concept | Type | Cardinality |
|---|---|---|
| Window | layout cell (boxlayout) | one per logical region (`main`, `files`, `extras`, `commits`, …) |
| View | gocui terminal buffer | one per viewName, but the assignment to a window can change |
| Context | panel-with-state | many contexts can target the same view (Normal/Staging/MergeConflicts → `main` view) |

## Key Dispatch Pattern

### The lazy\* signature (one sentence)
**Every keybinding is a `Binding{ViewName, Key, Handler, …}` registered at `(viewName, key)` in gocui's table; `ViewName == ""` is the global fallback.**

### Precedence ordering (highest first)

| Rank | Source | Source location |
|---|---|---|
| 1 | Search prompt bindings (when search is open) | `pkg/gui/keybindings.go:396-403` |
| 2 | User custom-command bindings | `pkg/gui/keybindings.go:405-411` |
| 3 | Per-context controllers (last-attached wins) | `pkg/gui/context/base_context.go:121-130` |
| 4 | Global controller bindings | `pkg/gui/controllers/global_controller.go:23-148` |
| 5 | Built-in scroll/quit bindings in `GetInitialKeybindings` | `pkg/gui/keybindings.go:78-358` |

### DisabledReason — five-shape gate

| Return shape | Effect |
|---|---|
| nil | Run handler. |
| `&{Text:""}` | Swallow silently; hide from options map. |
| `&{Text:msg}` | Toast "Disabled: msg"; swallow. |
| `&{ShowErrorInPanel:true, Text:msg}` | Return error → popup panel. |
| `&{AllowFurtherDispatching:true}` | Return `ErrKeybindingNotHandled`; gocui tries next. |

### Guards vs DisabledReason

- **Guards** (`OutsideFilterMode`, `NoPopupPanel`) are applied at *registration time* by wrapping the handler. They early-exit with a side effect (e.g., show "exit filter mode first" confirm). Cheap; no UI metadata.
- **DisabledReason** is consulted at *dispatch time* AND read by menu/options-bar render to strike-through or hide. Richer; supports tooltips and `AllowFurtherDispatching`.

### Tag faceting
`Binding.Tag` (`pkg/gui/types/keybindings.go:28`) is a string used by `OptionsMenuAction` to bucket bindings into local/global/navigation sections of the `?` menu (`pkg/gui/controllers/options_menu_action.go:66-77`). A binding can have a non-empty `ViewName` *and* `Tag: "global"` to surface in the global section of the help menu while still being view-scoped at dispatch — this is a faceting mechanism orthogonal to ViewName.

### Translation to monocle (Action enum)

```rust
struct Binding {
    pub view: Option<ViewId>,                   // None = global
    pub key: KeyChord,
    pub action: Action,                          // enum variant, not a closure
    pub description: Cow<'static, str>,
    pub short_description: Option<Cow<'static, str>>,
    pub tag: BindingTag,                         // Local | Global | Navigation
    pub display_on_screen: bool,
    pub opens_menu: bool,
    pub get_disabled_reason: Option<fn(&AppState) -> Option<DisabledReason>>,
}

enum Action {
    ScrollUpMain, ScrollDownMain, OpenFilter, OpenSearch,
    OpenKeybindingsMenu, OpenCustomCommandsMenu, Quit, Escape, …
}

struct DisabledReason {
    pub text: Cow<'static, str>,
    pub show_in_panel: bool,
    pub allow_further_dispatching: bool,
}
```

`App::dispatch(key)` does:
1. Snapshot current view ID.
2. If a modal is open, route to modal-specific handler set.
3. Look up `(view, key)` in custom_command bindings; if disabled-reason allows, run.
4. Look up `(view, key)` in per-context bindings.
5. Look up `(None, key)` in global bindings.
6. Check `get_disabled_reason`; render toast/popup as appropriate.

## Popup System

### The five primitives

| Primitive | Source | Use |
|---|---|---|
| `Alert(title, message)` | `pkg/gui/popup/popup_handler.go:105` | one-shot notification |
| `Confirm(opts ConfirmOpts)` | line 109 | Y/N popup |
| `Menu(opts CreateMenuOptions)` | line 58 | list of actions |
| `Prompt(opts PromptOpts)` | line 132 | text entry with optional suggestions |
| `WithWaitingStatus(msg, fn)` | line 74 | spinner during async work |

Plus `Toast` / `ErrorToast` for non-blocking bottom-bar messages.

### Resize is recomputed each tick

`pkg/gui/controllers/helpers/confirmation_helper.go:315-331` — `ResizeCurrentPopupPanels` runs at the end of every layout tick. It walks `Context().CurrentPopup()` and dispatches by context identity to the matching `resize*` function, threading `parentPopupContext` through so nested popups cascade.

### Nested popups cascade
`confirmation_helper.go:116-122` — when a parent popup is present, the new popup is positioned at parent's `(x0+2, y0+1)` corner with its own dimensions. The whole stack stays visible.

### Known gap: popup queueing
`confirmation_helper.go:199-205` — concurrent popup pushes are dropped with a log message. Comment line 201 admits this needs fixing with a queue. Monocle should accept the fix-up cost and implement a real `VecDeque<Modal>` queue.

### Menu superpowers
- Per-item shortcut keys (`MenuItem.Key`) — direct invocation without arrow navigation.
- `@` prefix in filter text → filter by keybinding label instead of menu-item label (`menu_context.go:83-91`).
- `MenuSection` (pointer-equality) groups items under green-coloured headers.
- `DisabledReason` → strike-through item + toast on press.
- Tooltip sidecar view sized just below the menu (`confirmation_helper.go:344-352`).
- `keybindingsTakePrecedence` flag: when true, menu-item shortcut keys override base `j`/`k`/etc.; when false (cheatsheet menu only), base nav wins.

### Suggestions context = peer of prompt
When `Prompt.FindSuggestionsFunc != nil`, an autocomplete list appears beneath the prompt. The user toggles focus with `Universal.TogglePanel` (default `<tab>`). The prompt's confirm uses typed text; the suggestions' confirm uses the highlighted value (`confirmation_helper.go:259-288`). Title dynamically interpolates the configured keybinding.

## Help, Filter, and Telescope-Style Conventions

### `?` menu is the telescope-style cheatsheet
`pkg/gui/controllers/options_menu_action.go:13-79`:
1. Snapshot every binding (incl. custom) via `GetInitialKeybindingsWithCustomCommands`.
2. Bucket by `Tag == "global"` (global section), `Tag == "navigation"` (navigation section), `ViewName == currentView` (local section).
3. Dedupe by `GetDescription()`.
4. Open a Menu with `MenuSection{Local, Global, Navigation}`, `AllowFilteringKeybindings: true`, `KeepConflictingKeybindings: true`, `HideCancel: true`.

### `/` filter is per-list, immediate, no debouncing
`pkg/gui/controllers/filter_controller.go:46` + `pkg/gui/controllers/helpers/search_helper.go:32`:
1. Each filterable context has a `FilterController` attached during `controllers.go:201-203`.
2. `/` opens the search prompt with that context as target.
3. Each keystroke calls `OnPromptContentChanged` → `ctx.SetFilter(...)` → immediate filter recompute via `sahilm/fuzzy`.
4. Enter pushes filter to history and pops the search context.
5. Esc clears the filter.

### Filter vs Search trait
- **Filter** changes which items are visible (list contexts).
- **Search** highlights matches without hiding non-matches (main view, patch explorer).
- Same `/` key, same helper, same prompt view; dispatch is by the focused context's trait.

### Frame colour during search
`search_helper.go:319-326` — `theme.SearchingActiveBorderColor` (cyan by default) replaces `theme.ActiveBorderColor` (green by default) while the filter/search is active. Powerful unconscious cue that something is filtered.

### Per-context search history
Each `IFilterableContext` / `ISearchableContext` owns a `*utils.HistoryBuffer[string]` (`pkg/gui/types/context.go:122-126`). `<up>` / `<down>` in the search prompt scrolls through history (`search_prompt_controller.go:35-41`).

### Convention table — what to inherit verbatim

| Convention | Source | Monocle adoption |
|---|---|---|
| `?` opens telescope cheatsheet menu | `Universal.OptionMenuAlt1` default | Inherit; bind `Action::OpenKeybindingsMenu` to `?`. |
| Three sections (Local / Global / Navigation) | `options_menu_action.go:44-46` | Inherit `MenuSection` shape. |
| `@` prefix → filter by keybinding | `menu_context.go:83-91` | Inherit verbatim. |
| `/` opens filter on the focused list | `filter_controller.go:46` | Inherit; bind `Action::OpenFilter` per-list. |
| Frame colour change during search | `search_helper.go:319-326` | Inherit; theme key `searching_border`. |
| Per-context history buffer | `pkg/gui/types/context.go:122-126` | Inherit. |
| `useFuzzySearch` config toggle | `Gui.FilterMode ∈ {"substring", "fuzzy"}` | Inherit; same enum. |

## Custom Commands Framework

### Data shape

```yaml
customCommands:
  - key: 'C'                                 # validated at registration
    context: 'localBranches'                 # comma-separated or 'global'
    command: 'git checkout {{.SelectedLocalBranch.Name}}'
    description: 'checkout branch'
    output: 'log'                            # '' | terminal | log | logWithPty | popup
    loadingText: 'Switching branches'
    after:
      checkForConflicts: false
    prompts:
      - type: 'input'                        # input | menu | menuFromCommand | confirm
        key: 'branchName'
        title: 'New branch name'
        suggestions:
          preset: 'branches'                 # or command: '<shell>'
```

### Recursive `commandMenu`
A `customCommand` with `commandMenu: [...]` becomes a Menu instead of a direct binding. Sub-commands may recursively contain their own `commandMenu`. Sub-commands with a `context:` field that doesn't match the current view are hidden when the menu opens (`pkg/gui/services/custom_commands/client.go:80-90`).

### Prompt chain
`pkg/gui/services/custom_commands/handler_creator.go:47-131` — prompts right-fold into a wrapped handler. Each prompt has optional `condition` template; if it resolves to empty/`"false"`, prompt is skipped. The final handler resolves `command` via `text/template` with funcMap `{quote, runCommand}` plus the accumulated `Form{key: value}` and `PromptResponses[]`.

### Output routing
| `output` | Effect |
|---|---|
| `""` (default) | Run silently, refresh after. |
| `terminal` | Suspend gocui, hand TTY to child. |
| `log` | Stream stdout into command-log view. |
| `logWithPty` | Same but with PTY (preserves colours). |
| `popup` | Capture stdout, show in Alert. |

### Custom commands have *highest precedence*
`pkg/gui/keybindings.go:405-411` prepends them to the binding list. A user can override built-in bindings.

### Translation to monocle
- `monocle.toml [[custom_commands]]` array, same shape.
- `text/template` → Rust's `tera` (most feature-equivalent) or `handlebars`.
- `Context` validates against monocle's `ContextId` enum.
- Output modes: `terminal` requires suspending the ratatui app and re-entering on child exit (use crossterm `disable_raw_mode` / `enable_raw_mode`).
- Suggestion presets: monocle-domain equivalents (sessions, threads, recent commands).

## Theming System

### Runtime mutable globals
`pkg/theme/theme.go:9-48` — 12 package-level vars: `DefaultTextColor`, `ActiveBorderColor`, `InactiveBorderColor`, `SearchingActiveBorderColor`, `GocuiSelectedLineBgColor`, `GocuiInactiveViewSelectedLineBgColor`, `OptionsColor`, `OptionsFgColor`, `SelectedLineBgColor`, `InactiveViewSelectedLineBgColor`, `CherryPickedCommitTextStyle`, `MarkedBaseCommitTextStyle`, `UnstagedChangesColor`, `DiffTerminalColor`.

`UpdateTheme(themeConfig config.ThemeConfig)` reassigns every one. Called from `gui.go:472` (`setColorScheme()`) during `onUserConfigLoaded`.

### Theme keys (user-facing)
| Config key | Semantic |
|---|---|
| `ActiveBorderColor` | colour for focused panel border |
| `InactiveBorderColor` | non-focused borders |
| `SearchingActiveBorderColor` | active border while filter/search is on |
| `SelectedLineBgColor` | focused row background |
| `InactiveViewSelectedLineBgColor` | row background when view is not focused |
| `OptionsTextColor` | bottom-bar key labels |
| `DefaultFgColor` | base text |
| `CherryPickedCommitBgColor/FgColor` | held-for-paste commits |
| `MarkedBaseCommitBgColor/FgColor` | rebase base marker |
| `UnstagedChangesColor` | files view tinting |

### Style primitives
`pkg/gui/style/basic_styles.go` defines:
- 9 named colours × {Fg, Bg} = 18 pre-built `TextStyle` instances.
- `AttrUnderline`, `AttrBold`, `Nothing` (no-op style).
- `ColorMap` for template colour functions.
- `TemplateFuncMapAddColors(m)` decorates a `template.FuncMap` so user-config templates can call `{{ red "danger" }}`.

### Runtime reload
Focus-triggered, not fsnotify. `SetFocusHandler` (`pkg/gui/gui.go:351-375`) fires on terminal focus regain → stat-compares config files → reloads + `setColorScheme()` + `resetKeybindings()` + `checkForChangedConfigsThatDontAutoReload(old, new)` (which warns about language changes that can't be hot-applied).

### Border style
`pkg/gui/views.go:159-169` — `Gui.Border ∈ {single, double, rounded, hidden, bold}`. The hidden style is just spaces — useful for users on minimal terminals or those who prefer no borders.

### Translation to monocle
- Use a `Theme` struct (not globals) threaded as `Arc<RwLock<Theme>>` for hot-reload safety.
- ratatui's `Style` is the natural equivalent of `TextStyle`. Use methods like `Style::default().fg(theme.active_border)`.
- Border style → `BorderType::{Plain, Double, Rounded, Hidden, Thick}`.
- Hot reload: hook crossterm's `Event::FocusGained` (supported since 0.27).

## Log Viewer and Scrollback

### Pattern: docked-bottom, expand-on-focus
`pkg/gui/controllers/helpers/window_arrangement_helper.go:391-404`:
- Default size = `userConfig.Gui.CommandLogSize` (typically 8 lines) + 2 for frame.
- When focused: size = 1000 (lazygit idiom for "fill remaining space").
- When terminal short (`height < 40`): size = 1 (just the title bar).

### Pattern: three-tier write surface
| Layer | Method | Style |
|---|---|---|
| Action (intent) | `LogAction(action)` | yellow |
| Command (mechanism) | `LogCommand(cmdStr, isCli)` | default if CLI, magenta otherwise |
| Output (effect) | `getCmdWriter().Write(bytes)` | prefixed with "Git output:" header |

### Pattern: autoscroll discipline
- Writes set `Autoscroll = true`.
- Any user-initiated scroll (`scrollUpExtra`, `pageUpExtrasPanel`, `goToExtrasPanelTop`) sets `Autoscroll = false`.
- `goToExtrasPanelBottom` re-enables `Autoscroll = true`. Only way to resume following.

### Pattern: parent-context push for predictable Esc
`pkg/gui/extras_panel.go:43` — before pushing the CommandLog context, set its parent to the current side context. Esc returns to that side context regardless of how the user arrived.

### Vim-style navigation bindings
`pkg/gui/keybindings.go:288-357` registers per-view nav:
- `PrevItem`/`NextItem` → scroll line
- `PrevPage`/`NextPage` → page (uses `ViewTrait.PageDelta()`)
- `GotoTop`/`GotoBottom` (and alt-keys) → home/end
- Mouse wheel up/down

All tagged `Tag: "navigation"` so they appear in the `?` menu's navigation section.

### Visual-line-select + clipboard copy (the lazyclaude lineage)
The `v` / `V` range-select + `<c-o>` copy pattern is in lazygit's *list contexts* via `IListCursor` (`pkg/gui/types/context.go:284-303`): `ToggleStickyRange`, `ExpandNonStickyRange`, `GetSelectionRange`, `AreMultipleItemsSelected`. The extras view inherits this through being list-like. The lazyclaudes copy this verbatim; monocle should too.

### Startup banner with random tip
`pkg/gui/command_log_panel.go:54-189` prints a coloured intro on first render referencing `ExtrasMenu` key, then (if `ShowRandomTip`) a random tip from a 28-entry corpus where each tip references the user's actual configured keybindings via `config.Universal.Push` etc. Monocle should adopt with its own tip corpus.

## Config Loading

### Layered model

```
Compile-time defaults
   ↓ unmarshal on top of
~/.config/lazygit/config.yml (global)         [ConfigFilePolicyCreateIfMissing]
   ↓ unmarshal on top of (only on per-repo load)
<workdir-walk-up>/.lazygit.yml chain          [ConfigFilePolicySkipIfMissing]
   ↓ unmarshal on top of
<repo>/.git/lazygit.yml                       [ConfigFilePolicySkipIfMissing]
```

Scalar fields: last-write-wins.
`CustomCommands`: accumulate across all files (`pkg/config/app_config.go:193-199`).

### ConfigFilePolicy three-valued enum
| Policy | Behaviour on missing file |
|---|---|
| `CreateIfMissing` | Create empty file at path. |
| `ErrorIfMissing` | Fail the load. |
| `SkipIfMissing` | Silently skip. |

### XDG discovery + env overrides
- `CONFIG_DIR` env → overrides config-dir lookup.
- `LG_CONFIG_FILE` env → comma-separated explicit file list (each `ErrorIfMissing`).
- Legacy path `<XDG_CONFIG_HOME>/jesseduffield/lazygit/config.yml` checked first for backward compat.

### Per-repo upward walk
`pkg/gui/gui.go:436-457` — start at the repo, walk up to filesystem root, prepend a `.lazygit.yml` config-file entry for each ancestor directory. Each is `SkipIfMissing`.

### Migration is YAML-rewrite, in-place
`pkg/config/app_config.go:219-330` — parses YAML to a node tree, applies a list of `pathsToReplace` renames and bespoke transformers, re-marshals, writes back to disk. Migrations are idempotent and visible to the user (their config file is updated).

### State file separation
`<XDG_STATE_HOME>/lazygit/state.yml` — machine-managed state (recent repos, last update check, GitHub PR cache, command history). Distinct from config (user intent).

### Hot reload trigger
`pkg/gui/gui.go:351-375` — terminal focus regain → stat-compare → reload + `setColorScheme()` + `resetKeybindings()`. No fsnotify dependency.

### Permission-graceful semantics
Writes to config dir, config file, and state file all silently no-op on `os.IsPermission(err)`. The app keeps running with in-memory state.

## Conventions to Adopt in Monocle

| Convention | Source citation | Adopt verbatim? |
|---|---|---|
| `(viewName, key, handler)` binding signature | `pkg/gui/keybindings.go:448` | YES |
| `ViewName == ""` means global fallback | `pkg/gui/keybindings.go:431` | YES |
| Custom commands prepended (user authority) | `pkg/gui/keybindings.go:405-411` | YES |
| Search prompt isolates dispatch | `pkg/gui/keybindings.go:396-403` | YES |
| `DisabledReason` 5-shape gate | `pkg/gui/keybindings.go:460-479` | YES |
| Guards composed inline at registration | `pkg/gui/keybindings.go:14-32` | IMPROVE — declarative `requires` field |
| `?` opens telescope cheatsheet menu | `pkg/gui/controllers/options_menu_action.go:13-79` | YES |
| 3-section split (Local/Global/Navigation) | `options_menu_action.go:44-46` | YES |
| `@` prefix in menu filter | `pkg/gui/context/menu_context.go:83-91` | YES |
| `/` per-list filter, immediate, no debounce | `pkg/gui/controllers/helpers/search_helper.go:223-236` | YES (consider debounce for 10k+ lists) |
| Frame colour swap during search | `search_helper.go:319-326` | YES |
| Per-context search history | `pkg/gui/types/context.go:122-126` | YES |
| `useFuzzySearch` config toggle | `Gui.FilterMode` | YES |
| 5 popup primitives over 2 contexts | `pkg/gui/popup/popup_handler.go` | YES |
| Nested popup cascade `(+2, +1)` offset | `pkg/gui/controllers/helpers/confirmation_helper.go:116-122` | YES |
| Popup queueing (FIX gap) | code TODO at `confirmation_helper.go:201` | IMPROVE — implement queue |
| Menu sections via pointer-equality | `pkg/gui/types/common.go:280-284` | YES (use enum variant identity in Rust) |
| Tooltip sidecar view for menus | `confirmation_helper.go:344-352` | YES |
| Suggestions context as peer of prompt | `confirmation_helper.go:170-180` | YES |
| Layered config (defaults + global + repo walk-up) | `pkg/config/app_config.go:139-207` + `pkg/gui/gui.go:436-457` | YES |
| `ConfigFilePolicy` three-valued enum | `pkg/config/app_config.go:56-69` | YES |
| Custom commands accumulate across files | `pkg/config/app_config.go:193-199` | YES |
| YAML/TOML migration with in-place rewrite | `pkg/config/app_config.go:219-330` | YES |
| Focus-triggered hot reload | `pkg/gui/gui.go:351-375` | YES |
| State file separation from config | `pkg/config/app_config.go:613-622,696-712` | YES |
| Permission-graceful writes | `pkg/config/app_config.go:166-170,636-641` | YES |
| Docked-bottom, expand-on-focus log viewer | `window_arrangement_helper.go:391-404` | YES |
| 3-tier write surface (Action/Command/Output) | `pkg/gui/command_log_panel.go:25-52` | YES |
| Autoscroll discipline (pause on user scroll) | `pkg/gui/extras_panel.go:48-93` | YES |
| Parent-context push for Esc semantics | `pkg/gui/extras_panel.go:43` | YES |
| Theme as runtime mutable singleton | `pkg/theme/theme.go:9-48` | IMPROVE — use `Theme` struct, not globals |
| Border style enum (single/double/rounded/hidden/bold) | `pkg/gui/views.go:159-169` | YES |
| `Tag` field for binding faceting | `pkg/gui/types/keybindings.go:28` | YES |
| `JumpToBlock` array for direct panel jump | `pkg/config/user_config.go:452` | YES |
| Startup banner with random keybind-aware tip | `pkg/gui/command_log_panel.go:54-189` | YES (with monocle tips) |
| Range-select + clipboard copy in lists | `pkg/gui/types/context.go:284-303` | YES |
| Three-valued `NeedsRerenderOnWidthChange` enum | `pkg/gui/types/context.go:42-52` | YES |

## Risk Register

### P0 — high impact

| ID | Risk | Source | Mitigation |
|---|---|---|---|
| P0-1 | Popup queueing missing — concurrent popups dropped | `pkg/gui/controllers/helpers/confirmation_helper.go:199-205` | Monocle implements a `VecDeque<Modal>` and only shows one at a time. Acknowledged TODO in lazygit. |
| P0-2 | Custom commands run via `OS().Cmd.NewShell` with template-expanded strings | `pkg/gui/services/custom_commands/handler_creator.go:295` | Monocle must NOT inherit string-substitution shell execution. Use `Command::new` with arg-array form; restrict templating to argument values only. |
| P0-3 | Theme is package-level mutable globals | `pkg/theme/theme.go:9-48` | Monocle uses a `Theme` struct threaded via `Arc<RwLock<Theme>>`. Avoids data races and improves testability. |

### P1 — medium impact

| ID | Risk | Source | Mitigation |
|---|---|---|---|
| P1-1 | Guards inlined at registration (must touch every binding to add a new guard) | `pkg/gui/keybindings.go:14-32` + call sites | Monocle adds a declarative `requires: BindingGuards` field on `Binding`. |
| P1-2 | Last-attached-controller-wins binding precedence is subtle | `pkg/gui/context/base_context.go:121-130` | Monocle should make attach order explicit and enforced by lint or documented per context. |
| P1-3 | Filter has no debounce; could thrash on very large lists | `pkg/gui/controllers/helpers/search_helper.go:223` | Monocle should add an optional debounce (configurable; default off). |
| P1-4 | `Cursor` visibility logic relies on `view.Editable && view.Mask == ""` | `pkg/gui/context.go:199` | Monocle should make cursor visibility per-modal explicit. |
| P1-5 | Hot-reload only fires on focus regain (not on save) | `pkg/gui/gui.go:351-375` | Acceptable, but optional fsnotify-based reload could be a follow-up. |
| P1-6 | Per-context binding storage is a slice with O(N) lookup | `pkg/gui/context/base_context.go:121-130` | Acceptable for ~50 bindings; monocle should ensure same complexity is OK. |
| P1-7 | `Tag` as a free-form string is fragile | `pkg/gui/types/keybindings.go:28` | Monocle uses `BindingTag` enum (`Local | Global | Navigation`). |

## Test Coverage Notes

Test files in the scope-relevant subsystems:

| File | LOC | Coverage |
|---|---|---|
| `pkg/gui/controllers/helpers/window_arrangement_helper_test.go` | 729 | The gold standard — exercises hundreds of layout scenarios. Demonstrates `GetWindowDimensions` is a pure function. |
| `pkg/gui/context/list_renderer_test.go` | 269 | List rendering with non-model items, section headers. |
| `pkg/gui/style/style_test.go` | 225 | `TextStyle` composition. |
| `pkg/gui/controllers/scroll_off_margin_test.go` | 189 | Margin-scroll behaviour. |
| `pkg/gui/controllers/helpers/refresh_helper_test.go` | 124 | Refresh debouncing. |
| `pkg/gui/controllers/helpers/fixup_helper_test.go` | 341 | Git-specific. |
| `pkg/gui/controllers/helpers/commits_helper_test.go` | 41 | Git-specific. |
| `pkg/gui/controllers/remotes_controller_test.go` | 165 | Git-specific. |
| `pkg/gui/controllers/helpers/upstream_helper_test.go` | 31 | Git-specific. |
| `pkg/gui/types/version_number_test.go` | 81 | Version comparison. |
| `pkg/gui/services/custom_commands/menu_generator_test.go` | 88 | Custom-command menu line-parsing. |
| `pkg/config/app_config_test.go` | 1,186 | Config loading + migration heavily exercised. |
| `pkg/config/keynames_test.go` | 686 | Keyname parsing. |
| `pkg/config/user_config_validation_test.go` | 321 | Validation rules. |
| `pkg/config/editor_presets_test.go` | 126 | Editor command presets. |
| `pkg/theme/style_test.go` | 57 | Theme application. |

The **pure-function layout** (`window_arrangement_helper.go`) and the **YAML rewriting migrations** (`app_config.go`) are the testing exemplars to copy. Both are pure transforms with explicit inputs; both have rich test files. Monocle should structure its layout and migration code identically.

## Architecture Recommendations for Monocle

### Top-level App shape (ratatui-portable)

```rust
pub struct App {
    pub state: AppState,
    pub contexts: ContextTree,
    pub context_mgr: ContextMgr,
    pub helpers: Helpers,
    pub controllers: Vec<Box<dyn Controller>>,
    pub bindings: BindingRegistry,
    pub modal_queue: VecDeque<Modal>,
    pub custom_commands: CustomCommandsClient,
    pub theme: Arc<RwLock<Theme>>,
    pub config: Arc<RwLock<MonocleConfig>>,
    pub command_log: CommandLog,
}
```

### Context trait (Rust idiom)

```rust
pub trait Context: Send + Sync {
    fn kind(&self) -> ContextKind;
    fn key(&self) -> ContextKey;
    fn view_id(&self) -> Option<ViewId>;
    fn window(&self) -> WindowName;
    fn bindings(&self) -> &[Binding];
    fn on_focus(&mut self, opts: OnFocusOpts);
    fn on_focus_lost(&mut self, opts: OnFocusLostOpts);
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme);
}
```

### ContextKind enum

```rust
pub enum ContextKind {
    Side,            // left rail panels; ≤1 on the stack
    Main,            // right-side main panel
    PersistentPopup, // search prompt, etc. (returnable)
    TemporaryPopup,  // confirm, menu, prompt (single-shot)
    Extras,          // command log
    Global,          // synthetic; holds globals
    Display,         // render-only views
}
```

### ContextMgr push/pop rules
- Push of `Side` → wipe stack down to `[c]`.
- Push of `Main` → drop existing `Main` from stack; keep side.
- Push of any popup → keep stack, push on top. If top is `TemporaryPopup` and pushing non-search, pop top first.
- Pop → activate new top via `on_focus`.

### Update loop pattern

```rust
match event {
    Event::Key(key) => {
        let view = self.context_mgr.current().view_id();
        if let Some(action) = self.bindings.lookup(view, key) {
            self.dispatch(action);
        } else if let Some(action) = self.bindings.lookup(None, key) {
            self.dispatch(action);
        }
    }
    Event::FocusGained => {
        if self.config.read().unwrap().reload_if_changed() {
            self.theme.write().unwrap().reload();
            self.bindings.rebuild();
        }
        self.refresh();
    }
    Event::Resize(w, h) => {
        self.layout(w, h);
    }
    // ...
}
```

### Layout function (pure)

```rust
pub fn arrange_windows(args: WindowArrangementArgs) -> HashMap<WindowName, Rect> {
    use ratatui::layout::{Constraint, Direction, Layout};
    // boxlayout-equivalent recursive tree; identical structure to lazygit
}
```

Keep it pure. Test it identically to `window_arrangement_helper_test.go`.

### Modal queue

```rust
pub fn push_modal(&mut self, modal: Modal) {
    self.modal_queue.push_back(modal);
    if self.current_modal.is_none() {
        self.advance_modal();
    }
}
fn advance_modal(&mut self) {
    self.current_modal = self.modal_queue.pop_front();
}
```

This fixes lazygit's known gap (P0-1).

### Theme

```rust
pub struct Theme {
    pub active_border: Color,
    pub inactive_border: Color,
    pub searching_border: Color,
    pub selected_line_bg: Color,
    pub options_fg: Color,
    pub default_fg: Color,
    // ...
}

impl Theme {
    pub fn from_config(cfg: &ThemeConfig) -> Self { /* ... */ }
}
```

Threaded explicitly; no globals.

### Custom commands

```toml
[[custom_commands]]
key = "C"
context = "sessions"
command = "claude --session {{ session.id }} --thread {{ thread.id }}"
description = "Open session in claude"
output = "log"

  [[custom_commands.prompts]]
  type = "input"
  key = "topic"
  title = "Topic for new thread"
  suggestions.preset = "recent_topics"
```

Rust-side template using `tera` with funcMap `{quote, run_command}`.

## Convergence Statement

All seven in-scope lanes ran one broad pass + one deepening round (Phase A: 7 broad passes; Phase B: 6 deep rounds, one per lane). Coverage audit (Pass B.5) confirms every requested in-scope item has a deepening, with all out-of-scope items cited once and not deepened. Extraction validation (Pass B.6) spot-checked 22 of 22 random claims; all verified against source.

This is a SCOPED ingest with explicit boundaries; the protocol calls for converging fast once the in-scope lanes are well-characterised. All six deepening rounds returned substantive new findings beyond their broad passes. Additional rounds would yield refinements (nitpicks), not new patterns. Per Iron Law honest convergence: this is converged.

## Handoff

Downstream skills (`semport-analyze`, `create-brief`, `create-prd`) should use **this synthesis (`lazygit-pass-8-final-synthesis.md`)** as the primary reference. The other pass files provide drill-down detail but are subsidiary.

Recommended next steps for monocle:
1. Crystallize the Action enum from the Key Dispatch Pattern section.
2. Implement the modal queue per Architecture Recommendations (fixes P0-1).
3. Adopt the layered config + ConfigFilePolicy pattern from Config Loading.
4. Build the `?` telescope menu per Help/Filter conventions table.
5. Set up the docked-bottom log viewer per Log Viewer section.
6. Wire the theme as a `Theme` struct (fixes P0-3) with focus-triggered hot reload.

## Files in this Scoped Ingest

All absolute paths under `/Users/jmagady/Dev/monocle/.factory/semport/lazygit/`:

| File | Bytes |
|---|---|
| `lazygit-pass-1-project-discovery.md` | 6,112 |
| `lazygit-pass-2-tech-and-build.md` | 2,632 |
| `lazygit-pass-3-architecture.md` | 11,999 |
| `lazygit-pass-4-domain.md` | 8,252 |
| `lazygit-pass-5-behavior.md` | 16,567 |
| `lazygit-pass-6-nfr.md` | 4,864 |
| `lazygit-pass-7-holdout-seeds.md` | 5,236 |
| `lazygit-pass-B-deep-tui-architecture-r1.md` | 6,049 |
| `lazygit-pass-B-deep-key-dispatch-r1.md` | 7,151 |
| `lazygit-pass-B-deep-popup-patterns-r1.md` | 7,763 |
| `lazygit-pass-B-deep-help-overlay-and-filter-r1.md` | 7,057 |
| `lazygit-pass-B-deep-custom-commands-and-theming-r1.md` | (this round) |
| `lazygit-pass-B-deep-log-viewer-and-scrollback-r1.md` | (this round) |
| `lazygit-pass-B-deep-config-loading-r1.md` | (this round) |
| `lazygit-pass-B5-coverage-audit.md` | (this round) |
| `lazygit-pass-B6-extraction-validation.md` | (this round) |
| `lazygit-pass-8-final-synthesis.md` | (this file) |

Total: 17 markdown files. No git commits made.
