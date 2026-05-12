# Pass B (deep) — TUI Architecture (round 1)

## Findings beyond the broad sweep

### F-TUI-1: ContextTree is a typed inventory; flat ordering matters
`/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/context.go:85-170`

The `ContextTree` struct names every context by typed field (`Files *WorkingTreeContext`, `Branches *BranchesContext`, `Menu *MenuContext`, etc.). The `Flatten()` method returns them in a hand-curated order which defines z-stacking when contexts share a window. Side panels first (status/snake/files/branches/commits/stash), then popups (menu/confirmation/prompt/commitMessage), then main windows in reverse-stack order (mergeConflicts → stagingSecondary → staging → patchBuilders → normalSecondary → normal), then display-only views.

This is **strongly typed enum-of-panels with hand-ordered z-index**. For monocle, the Rust equivalent is a struct of `Context` values keyed by name, but the ordering rule still applies — the order in which contexts are flattened decides which view appears on top when windows overlap.

### F-TUI-2: TabView allows multiple contexts on one window
`/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gui/context/context.go:172-176` + `pkg/gui/gui.go:432-443` (`gui.viewTabMap` iteration in `resetKeybindings`).

`TabView{Tab, ViewName}` couples a tab-label to a view-name. Multiple `ViewName`s can share the same window, and the gocui `View.Tabs []string` is the rendering surface (see `pkg/gui/views.go:267`). Tab click → `onViewTabClick(window, tabIndex)`. This is how the *Branches* window shows local + remote + tags tabs.

For monocle: a single panel that can host different sub-views (e.g. "Sessions" vs "Threads") should use this tab pattern. ratatui-side, this is a `BlockTabs` widget atop a `match active_tab` switch.

### F-TUI-3: Window vs View vs Context — three distinct concepts
- **Window** = layout cell (boxlayout node), identified by string name (`"main"`, `"secondary"`, `"files"`, `"extras"`, …). Has dimensions.
- **View** = gocui terminal-buffer (`*gocui.View`). 1-to-1 with viewName.
- **Context** = panel-with-state (Go struct). N-to-1 with view (multiple contexts can render to the same view, e.g. main shows Normal or Staging or MergeConflicts).

A context's window is set via `SetWindowContext` (`pkg/gui/controllers/helpers/window_helper.go`) — when the merge-conflicts context activates, its window becomes "main", taking that cell.

### F-TUI-4: `HasUncontrolledBounds` is the opt-out for layout
`pkg/gui/context/base_context.go:51` — most contexts are sized by `setViewFromDimensions` (`pkg/gui/layout.go:54`) which reads from the boxlayout. Popups (Menu, Confirmation, Prompt, Suggestions, CommitMessage, CommitDescription) set `HasUncontrolledBounds: true` so they're sized by the popup helper instead (`ResizeCurrentPopupPanels`, called at the bottom of every layout tick — `layout.go:190`).

### F-TUI-5: `NeedsRerenderOnWidthChange` is a three-valued enum
`pkg/gui/types/context.go:42-52`:
```
NEEDS_RERENDER_ON_WIDTH_CHANGE_NONE
NEEDS_RERENDER_ON_WIDTH_CHANGE_WHEN_WIDTH_CHANGES
NEEDS_RERENDER_ON_WIDTH_CHANGE_WHEN_SCREEN_MODE_CHANGES
```
The first is for views that don't care (e.g. status). The second is for line-truncating views (the branches panel cuts long names). The third is for views that render differently when the user toggles screen-mode (half/full) — those rerender only on mode change to avoid wasting renders on each character of resize.

Monocle should adopt this trichotomy because it's the right way to keep dirty-rendering cheap: most views don't need rerendering on resize.

### F-TUI-6: Per-context render hooks are chained
`base_context.go:191-198,200-216` — `AddOnRenderToMainFn`, `AddOnFocusFn`, `AddOnFocusLostFn`, `AddOnQuitFn`. The `onRenderToMainFn` slot is single-owner (panics if two controllers set it), but focus and focus-lost are appended as slices (multiple controllers can each get notified). This is a small but important asymmetry: render-to-main is opinionated; focus events are observer-pattern.

### F-TUI-7: `StateAccessor` is a deliberate seam for testability
`pkg/gui/gui.go:152-221` — instead of exposing `*Gui` to helpers, lazygit threads `StateAccessor` (and `GuiRepoState` as `IRepoStateAccessor`) through the `IStateAccessor` interface (`pkg/gui/types/common.go:368-384`). This is enough surface for helpers to read/write the screen mode, popup options, search state, item-operations, etc., without coupling to the god struct.

### F-TUI-8: The driver pattern enables integration tests
`pkg/gui/gui_driver.go:165` (file size confirmed by Pass 1; full contents not deepened here). The integration test framework uses a `GuiDriver` to inject keystrokes and observe the rendered output — same pattern as a Rust ratatui app exposing a `tick(input: Event)` method.

### F-TUI-9: `Mutexes` struct centralises lock points
`pkg/gui/types/common.go:336-346` — 10 named `deadlock.Mutex` instances bundled into one struct. The `PopupMutex` is held when reading/writing `CurrentPopupOpts` (`confirmation_helper.go:80,82,191,193`). The `RefreshingFilesMutex` blocks concurrent file refreshes. Centralising locks here makes it visually obvious which subsystems are concurrent-sensitive.

## Delta Summary
- New items: 9 (z-ordering, tab pattern, window/view/context trichotomy, uncontrolled bounds, three-valued NeedsRerenderOnWidthChange, render-hook chaining, state-accessor seam, driver pattern note, central mutex struct).
- Existing refinements: ContextStack push/pop precision per kind.
- Remaining gaps: how the gocui event loop interacts with the goroutine pool (would need pkg/gocui + BackgroundRoutineMgr deepening — out of scope for this scoped ingest).

## Round assessment
This round is SUBSTANTIVE — added 9 architectural items not in the broad sweep. Another round could enumerate the controller/helper graph fully, but for monocle's purposes the abstractions are now mapped. Declaring this lane CONVERGED after one deep round given the scoped mandate.
