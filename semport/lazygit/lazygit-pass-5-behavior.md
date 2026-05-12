# Pass 5 — Behavioral Contracts (TUI-only, scoped)

These are the binding-time and runtime invariants that the lazy* pattern depends on. Each is grounded in source.

## BC-DRAFT-001: Key dispatch is scoped by `ViewName`
**Where:** `pkg/gui/keybindings.go:448-454` — `SetKeybinding` calls `gui.g.SetKeybinding(binding.ViewName, binding.Key, handler)`. gocui's keybinding map is `(viewName, key) → handler`. If `ViewName == ""`, the binding fires globally regardless of focused view.
**Pre:** A binding registered for view `"commits"` only fires when `"commits"` is the current view.
**Post:** Two bindings with the same `Key` but different `ViewName` coexist with distinct behaviour — this is the **lazy\* signature pattern**.
**Confidence:** HIGH (mechanism is direct).

## BC-DRAFT-002: Per-context bindings flow from controllers via `KeybindingsFn`
**Where:** `pkg/gui/context/base_context.go:121-130` — `GetKeybindings` runs **all** `keybindingsFns` registered on the context, in reverse-registration order (the most recently attached controller wins). Controllers attach with `AddKeybindingsFn` via `AttachControllers` (`pkg/gui/controllers.go:202..429`).
**Pre:** When `resetKeybindings` runs (`pkg/gui/keybindings.go:415`), every context is queried and its bindings are registered with the context's view name (`keybindings.go:361-369`).
**Post:** Multiple controllers can target the same context (e.g., `Branches` gets `branchesController` + `gitFlowController`, see `controllers.go:346`) and their bindings combine. Latest-attached controller's binding wins on key collision.
**Confidence:** HIGH.

## BC-DRAFT-003: Global keybindings live in a synthetic GLOBAL_CONTEXT
**Where:** `pkg/gui/context/setup.go:9-18` — the `Global` context has `Kind: GLOBAL_CONTEXT`, `View: nil`, `Focusable: false`, `HasUncontrolledBounds: true`. `controllers.go:413-420` attaches the global controller list (undo, global, contextLines, renameSimilarityThreshold, jumpToSideWindow, sync) to it. The empty `ViewName` on its bindings causes gocui to register them globally.
**Pre:** Anything universal (quit, refresh, screen-mode cycle, options menu, filtering menu, diffing menu) lives here.
**Post:** Per-view bindings can shadow globals because gocui dispatches view-specific bindings before global ones.
**Confidence:** HIGH.

## BC-DRAFT-004: Custom commands inject themselves *before* built-in bindings
**Where:** `pkg/gui/keybindings.go:405-411` — `GetInitialKeybindingsWithCustomCommands` prepends custom bindings: `bindings = append(customBindings, bindings...)`. The comment on line 410 explains: "we want to give our custom keybindings precedence over default keybindings."
**Pre:** A user's `customCommands` entry binding `q` to a confirmation will override the built-in quit.
**Post:** This is intentional: lazygit treats custom commands as user authority. Monocle should adopt this.
**Confidence:** HIGH.

## BC-DRAFT-005: When the search prompt is open, no other bindings fire
**Where:** `pkg/gui/keybindings.go:396-403` — `GetInitialKeybindingsWithCustomCommands` checks if `Current().GetKey() == SEARCH_CONTEXT_KEY` and returns *only* that context's bindings. Custom commands and global bindings are skipped while typing in `/`.
**Pre:** During `/` prompt, only Enter / Esc / history-scroll / textarea editor are active.
**Post:** No way to accidentally trigger global actions mid-typing.
**Confidence:** HIGH.

## BC-DRAFT-006: Guards short-circuit handlers
**Where:** `pkg/gui/keybindings.go:24-45` — `outsideFilterMode` wraps a handler; if filter is active it shows a confirm dialog ("Must exit filter mode first") and returns nil. `noPopupPanel` (line 14) returns nil if a popup is focused. Both are exposed as `opts.Guards` and used inline at registration time: `Handler: opts.Guards.NoPopupPanel(self.refresh)` (e.g. `global_controller.go:48`).
**Pre:** Guards are not invoked at dispatch-time by the framework; they're applied by the binding *author* at registration.
**Post:** This is **inlining preconditions** rather than declarative guards, so adding a new guard means touching every registration. Monocle can improve on this with a declarative `requires:` slot on `Binding`.
**Confidence:** HIGH.

## BC-DRAFT-007: `GetDisabledReason` is the post-binding gate
**Where:** `pkg/gui/keybindings.go:460-479` — `callKeybindingHandler` checks `binding.GetDisabledReason`. If non-nil:
- with `ShowErrorInPanel: true` → return the error (gets rendered as panel popup).
- with `Text != ""` → emit toast and **swallow** the keystroke (return nil).
- with `Text == ""` → swallow silently (used for hiding the binding from the options map without showing an error).
- with `AllowFurtherDispatching: true` → return `ErrKeybindingNotHandled` so gocui tries the next handler.
**Pre:** Disabled-reason is a richer gate than `Guard` because the framework consults it for menu rendering and the bottom-line filter (`pkg/gui/options_map.go:52`).
**Post:** This is the right level of abstraction for monocle — keep it.
**Confidence:** HIGH.

## BC-DRAFT-008: Popups serialise — at most one popup-creation in flight
**Where:** `pkg/gui/controllers/helpers/confirmation_helper.go:198-205` — `CreatePopupPanel` checks `CurrentPopupOpts`. If a non-loader popup is already open, the new one is dropped with `gui.c.Log.Error("ignoring create popup panel because a popup panel is already open")`. The line-201 comment admits "The proper solution is to have a queue of popup options".
**Pre:** Concurrent confirms / prompts arriving from goroutines can lose popups.
**Post:** Known shortcoming. Monocle should fix with a real queue.
**Confidence:** HIGH (acknowledged in code).

## BC-DRAFT-009: Popups stack via offset; nested popups are visible
**Where:** `confirmation_helper.go:116-122` — `getPopupPanelDimensionsAux` offsets nested popups by `(2, 1)` from the parent's `(x0, y0)`, *not* centred. The whole popup stack is iterated in `ResizeCurrentPopupPanels` (line 315) walking `CurrentPopup()` and passing the previous popup as `parentPopupContext`.
**Pre:** A confirmation opened over a menu sits cascaded down-right from the menu's top-left corner.
**Post:** Monocle should reproduce this; nested popups are a common UX in lazyclaudes.
**Confidence:** HIGH.

## BC-DRAFT-010: Side context switches blow away the stack
**Where:** `pkg/gui/context.go:91-95` — pushing a SIDE_CONTEXT removes every other context from the stack. Comment line 92: "if we are switching to a side context, remove all other contexts in the stack".
**Pre:** Jumping from Files → Branches via `2` discards any open menus/popups underneath.
**Post:** Stack is invariant: ≤1 side context at the bottom.
**Confidence:** HIGH.

## BC-DRAFT-011: Search prompt is the only TEMPORARY_POPUP that can stack with another popup
**Where:** `pkg/gui/context.go:117-119` — when popping the top to make room for a non-search push, the search context is the exception. The comment notes: "Ideally you'd be able to escape back to previous temporary popups, but because we're currently reusing views for this, you might not be able to get back to where you previously were."
**Pre:** Filtering inside a menu (e.g. `?` keybindings menu with fuzzy filter) works because search → menu doesn't clobber.
**Post:** This is the only "popup over popup" the architecture supports; the rest force a pop-and-push.
**Confidence:** HIGH.

## BC-DRAFT-012: Theme is global mutable state, recomputed on config reload
**Where:** `pkg/theme/theme.go:51-76` — `UpdateTheme(themeConfig)` reassigns 11 package-level globals. Called from `gui.go:472` (`setColorScheme()`), which is called from `onUserConfigLoaded` (`gui.go:459`). User-config file changes are detected by stat-mtime in `pkg/config/app_config.go:559-581`.
**Pre:** Editing the user config file while lazygit is open repaints the UI on focus-regain (`gui.go:351-375`).
**Post:** Theming has no per-component locality; everything reads the package globals. Monocle can improve with a `Theme` struct threaded explicitly (cleaner; testable), but mirror the runtime-reload behaviour.
**Confidence:** HIGH.

## BC-DRAFT-013: User config files merge with earlier-wins-by-base; custom commands accumulate
**Where:** `pkg/config/app_config.go:139-207` — `loadUserConfig` iterates `configFiles` in order. Each file is yaml-unmarshalled **on top of** the running `base`. Scalar fields therefore last-write-wins. **Exception (line 193-199):** `CustomCommands` are concatenated rather than replaced: `base.CustomCommands = append(base.CustomCommands, existingCustomCommands...)`.
**Pre:** Per-repo `.lazygit.yml` overrides the global config for scalars but adds to its custom commands list.
**Post:** This is the "layered config" semantics monocle should adopt.
**Confidence:** HIGH.

## BC-DRAFT-014: Per-repo config search walks upward from the worktree root
**Where:** `pkg/gui/gui.go:436-457` — `getPerRepoConfigFiles` starts with `.git/lazygit.yml`, then walks `dir → parent → root` looking for `.lazygit.yml`, prepending each (so root-most has lowest precedence after the global).
**Pre:** A monorepo can have a root `.lazygit.yml` + per-package overrides.
**Post:** All discovered repo configs use `ConfigFilePolicySkipIfMissing` (no error if absent).
**Confidence:** HIGH.

## BC-DRAFT-015: Layout recomputes every render tick
**Where:** `pkg/gui/layout.go:13-207` — `layout(g)` is the gocui layout callback. It runs every time gocui needs to redraw: input event, resize, manual `Render()`, periodic background fetch, etc. Inside, `getWindowDimensions` (line 33) calls into the boxlayout tree (computed fresh every tick); views get repositioned (line 112-122).
**Pre:** A `setView(name, x0, y0, x1, y1, 0)` call is cheap because gocui re-uses existing views.
**Post:** Window arrangement is functionally pure given `(width, height, currentWindow, currentSideWindow, screenMode, splitMainPanel, …)`. **This is the most testable component in lazygit** — `window_arrangement_helper_test.go` (729 LOC) exercises hundreds of layouts. Monocle should adopt the same purity boundary.
**Confidence:** HIGH (entire test file exists).

## BC-DRAFT-016: The bottom options bar is rebuilt every layout from the current context's bindings
**Where:** `pkg/gui/options_map.go:37-106` — `renderContextOptionsMap` collects `currentContextBindings`, then adds globals that don't clash on key. Filters for `DisplayOnScreen && !IsDisabled`. Adds mode-specific bindings (cherry-pick paste, bisect, patch-builder, rebase) prepended in special colours. Output is ` | `-separated, truncated with `…` if too wide.
**Pre:** The bottom bar is **context-aware and dynamic** — change focus and the bottom-bar refreshes.
**Post:** Mode indicators are colour-coded: cyan for cherry-pick paste, green for bisect, yellow for rebase/patch.
**Confidence:** HIGH.

## BC-DRAFT-017: Menus support `@` prefix to filter by keybinding instead of label
**Where:** `pkg/gui/context/menu_context.go:83-91` — when the menu's `allowFilteringKeybindings` is set and the filter string starts with `@`, the prefix is stripped and the filter searches the bound key labels rather than the item labels. The keybindings menu enables this (`options_menu_action.go:53`).
**Pre:** In the `?` menu, typing `@q` filters to bindings whose key is `q`.
**Post:** This is a beloved UX detail — monocle should adopt.
**Confidence:** HIGH.

## BC-DRAFT-018: Filtering uses fuzzy or substring matching depending on user config
**Where:** `pkg/gui/context/filtered_list.go:83-105` — `applyFilter` builds a `fuzzy.Source`, calls `utils.FindFrom(filter, source, useFuzzySearch)`. `useFuzzySearch` comes from `UserConfig.Gui.UseFuzzySearch()` which reads `Gui.FilterMode ∈ {"substring", "fuzzy"}`.
**Pre:** Filter is run synchronously on every key press; `OnPromptContentChanged` (`search_helper.go:223`) calls `SetFilter` directly.
**Post:** No debouncing. Filter recomputes from scratch each keystroke. Acceptable because the `fuzzy` lib is fast and lists are usually small. **Monocle should consider debouncing** for very-large lists (10k+ items), though for typical lazyclaude data sizes the lazygit approach is fine.
**Confidence:** HIGH.

## BC-DRAFT-019: Command log (extras panel) auto-scrolls unless user manually scrolls
**Where:** `pkg/gui/command_log_panel.go:30,41` — `LogAction` and `LogCommand` both set `gui.Views.Extras.Autoscroll = true`. `pkg/gui/extras_panel.go:49,58,65,73,82` — every manual scroll handler (`scrollUpExtra`, `scrollDownExtra`, `pageUpExtrasPanel`, `pageDownExtrasPanel`, `goToExtrasPanelTop`) sets `Autoscroll = false`. The `goToExtrasPanelBottom` handler (line 89) re-enables it.
**Pre:** As long as the user hasn't scrolled, new log entries are visible.
**Post:** Scrolling pauses autoscroll until the user reaches bottom.
**Confidence:** HIGH.

## BC-DRAFT-020: Toast emission goes via `setToastFunc` injected at construction
**Where:** `pkg/gui/popup/popup_handler.go:23,69-71` — `toastFn` is a dependency; `SetToastFunc` lets the test framework or the demo recorder swap it. The default impl renders into the bottom bar's app-status slot (`pkg/gui/controllers/helpers/app_status_helper.go`).
**Pre:** Toasts and waiting-statuses share the same right-side slot; the layout allocates room for both via `appStatus` window.
**Post:** Toasts are non-blocking.
**Confidence:** HIGH.

## BC-DRAFT-021: Custom-command prompts compose recursively (right-folded)
**Where:** `pkg/gui/services/custom_commands/handler_creator.go:47-131` — `call(customCommand)` returns a function that, when invoked, builds a chain. Each prompt right-folds into the previous handler. Optional `Condition` template-expanded skips the prompt. The final handler resolves the `Command` template with the accumulated `Form{}` and `PromptResponses` and runs via `OS().Cmd.NewShell`.
**Pre:** Prompt types: `input`, `menu`, `menuFromCommand`, `confirm`. Suggestions for input prompts come from one of 7 presets (`authors`, `branches`, `files`, `refs`, `remotes`, `remoteBranches`, `tags`) or a custom shell command.
**Post:** The `Output:` field on the command picks delivery: `terminal` (subprocess), `log` (stream to command log), `logWithPty` (stream with pty), `popup` (alert with output), default (silent + refresh).
**Confidence:** HIGH.

## BC-DRAFT-022: Context activation always sets cursor visibility from view editability
**Where:** `pkg/gui/context.go:199` — `gui.c.GocuiGui().Cursor = v.Editable && v.Mask == ""`. A masked editable view (password prompt) hides the cursor.
**Pre:** Cursor visibility is purely a function of the active view's editability + masking.
**Post:** Monocle's ratatui equivalent is the explicit `Frame::set_cursor` call.
**Confidence:** HIGH.

## BC-DRAFT-023: Menus can opt into having bound keys take precedence over base bindings
**Where:** `pkg/gui/context/menu_context.go:218-231` — when a regular menu opens (`keybindingsTakePrecedence: true`), menu-item shortcut keys override the base list-navigation bindings (`j`, `k`, `H`, `L`). When the keybindings cheatsheet menu opens (`keybindingsTakePrecedence: false`), base list nav wins so the user can still navigate. The comment explains the security choice: essential bindings (confirm, return) have already been stripped from menu items in the precedence-takes case.
**Pre:** This is the "the menu's shortcut is what the user sees, and what they pressed" property.
**Post:** Monocle's menu implementation should expose this knob.
**Confidence:** HIGH.

## BC-DRAFT-024: `OpensMenu` and `OpensMenuTitle` are presentational only
**Where:** `pkg/gui/types/keybindings.go:29`, `pkg/gui/context/menu_context.go:159` — `OpensMenu bool` adds a "..." or arrow indicator in the cheatsheet rendering. Setting it doesn't actually wire menu-opening logic; that's still the handler's job.
**Pre:** Don't conflate `OpensMenu: true` with actual menu opening.
**Confidence:** HIGH.

## BC-DRAFT-025: Resize triggers re-read from view buffer managers
**Where:** `pkg/gui/layout.go:36-49` — when main view grows, the `ViewBufferManager` for main/secondary reads `heightDiff` additional lines. This is part of lazygit's "stream content lazily" optimisation for slow git commands.
**Pre:** Main views back onto a pty / streaming command output; reads are lazy.
**Post:** Monocle's log viewer or process-output panel should adopt the same lazy-read-on-resize pattern.
**Confidence:** HIGH.

## State checkpoint
pass: 5
status: complete
contracts: 25
confidence: all HIGH
