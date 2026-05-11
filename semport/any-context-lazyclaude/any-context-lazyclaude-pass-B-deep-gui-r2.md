# Pass B Deep: `internal/gui` — Round 2

**Scope this round:** popup view layer (popup.go), dialog state (dialog.go), state.go (fullscreen entry, key forwarding, paste), search dialog (search.go), keybinding registration (keybindings.go), app_actions.go head (cursor + plugin/MCP sync), keymap registry shape.

**Files read in full this round:** popup.go, dialog.go, state.go, keydispatch/dispatcher.go (already), keyhandler/popup.go (already). Partial: app_actions.go (lines 1-470), keybindings.go (lines 1-120), search.go (lines 1-150), keymap/registry.go (lines 1-100).

## Popup view-layer rendering

`layoutToolPopup` (popup.go:61-207) renders all visible popups as **cascaded overlays**, plus an action bar at the bottom of the focused popup.

### Cascade geometry

```
popW = maxX * 7 / 10, clamped to >= 40 (else maxX - 4)
popH = maxY * 6 / 10, clamped to >= 10 (else maxY - 4)
baseX = (maxX - popW) / 2          // center horizontally
baseY = (maxY - popH) / 2          // center vertically
cascadeOffset(i) = (baseX + i*2, baseY + i)   // 2x1 stagger per visible popup
```

Each visible popup gets a view named `tool-popup-<stack-index>`. The cascade index `visibleIdx` increments per non-suspended entry. Action bar view is `tool-popup-actions` (only for the focused entry).

### BC-GUI-POPUPVIEW-001: Popup width/height defaults to 7/10 × 6/10 of terminal, clamped to minimum 40×10 (or screen-1 if smaller)
**Postconditions:** Cascade stagger is +2 cols, +1 row per visible popup.
**Evidence:** popup.go:69-92, popupCascadeOffset (popup.go:271-273).
**Confidence:** HIGH

### BC-GUI-POPUPVIEW-002: Bottom-right cascade clamped to `maxY-3` and `maxX-1`
**Postconditions:** Overflow prevented on narrow/short terminals.
**Evidence:** popup.go:94-99.
**Confidence:** HIGH

### BC-GUI-POPUPVIEW-003: Popup viewport height is set on the popup struct each frame so scroll bounds adjust on resize
**Postconditions:** After resize, `SetScrollY(min(scrollY, MaxScroll(visibleLines)))` is applied — scroll position clamped to new max.
**Evidence:** popup.go:108-115.
**Confidence:** HIGH

### BC-GUI-POPUPVIEW-004: cleanupPopupViews iterates 0..19 and deletes any tool-popup-N not in the active stack
**Postconditions:** Cap of 20 popup slots. If 21+ popups stack (unlikely), the view #20 onward leaks (popup.go:212).
**Evidence:** popup.go:210-219.
**Confidence:** HIGH — **bug-like edge case** (acceptable given UX would already be broken with 20 popups).

### BC-GUI-POPUPVIEW-005: Action-bar hints are conditionally filtered
- `ActionPopupAllow` hidden when `maxOpt < 3`.
- `ActionPopupAcceptAll` hidden when `visible <= 1`.
- `ActionPopupFocusNext` hidden when `visible <= 1`.
- `ActionPopupScrollDown` hidden when popup `MaxScroll(vh) == 0`.

**Postconditions:** Hint bar adapts dynamically to popup state.
**Evidence:** popup.go:167-185.
**Confidence:** HIGH

### BC-GUI-POPUPVIEW-006: Indicator `[N/M]` shows position in stack when visible > 1
**Postconditions:** Format = `[<visibleIdx+1>/<visibleCount>]` appended to the hint bar.
**Evidence:** popup.go:192-195.
**Confidence:** HIGH

### BC-GUI-POPUPVIEW-007: Popup gets focus unconditionally; gocui Cursor set to false
**Postconditions:** Even when a dialog is open, popup steals focus (and layoutMain re-restores dialog focus after dismiss). Cursor hidden because popup is non-editable.
**Evidence:** popup.go:198-203.
**Confidence:** HIGH

## Popup → tmux choice flow

`dismissPopup` (popup.go:25-38) is the production path:

1. `DismissActive(choice)` pops focused popup, returns `window` string.
2. Set window activity to Running **immediately** (no wait for SendChoice).
3. Spawn goroutine: `_ = a.sessions.SendChoice(window, choice)` — fire-and-forget.

### BC-GUI-CHOICE-002: Window activity transitions NeedsInput → Running immediately on the gocui goroutine, BEFORE the async SendChoice completes
**Postconditions:** The badge clears immediately so the user perceives instant feedback even if tmux SendKeys is slow.
**Evidence:** popup.go:30-37. Comment line 30-31.
**Confidence:** HIGH

### BC-GUI-CHOICE-003: SendChoice errors are silently dropped (`_ =`)
**Postconditions:** If tmux is gone or window-id invalid, the user does not see the error — but the badge already cleared, so no popup re-appears.
**Evidence:** popup.go:35.
**Confidence:** HIGH — **observability gap**. P2 — log-only would be safer.

### BC-GUI-CHOICE-004: DismissAll loop sets every window to Running before spawning the SendChoice loop in a separate goroutine
**Postconditions:** Sequential SendChoice in a single goroutine, ordering preserved.
**Evidence:** popup.go:41-58.
**Confidence:** HIGH

## Diff generation

`generateDiffFromContents` (popup.go:221-256) is used by `DiffPopup.ensureCache`:

1. Write `newContents` to a temp file in `os.TempDir()`.
2. If `oldFilePath` does not exist: synthesize a `/dev/null → file` diff with all `+` lines (popup.go:236-245).
3. Else: shell out to `git diff --no-index --unified=3 -- <oldFilePath> <tmpFile>`.
4. Always returns a string; never errors. Error paths inline `(error: ...)` strings into the output.

### BC-GUI-DIFF-001: `oldFilePath` not existing yields a synthetic add-only diff with `--- /dev/null` and `+++ <basename>` headers; every non-empty line prefixed with `+`
**Evidence:** popup.go:236-245.
**Confidence:** HIGH

### BC-GUI-DIFF-002: Otherwise, `git diff --no-index --unified=3` is the canonical engine; output captured even when exit code != 0 (because `git diff` exits 1 when there are differences)
**Postconditions:** `cmd.Output()` returns `out, *exec.ExitError` when there is a diff. The `if err != nil && len(out) > 0` branch preserves the output.
**Evidence:** popup.go:247-251.
**Confidence:** HIGH

### BC-GUI-DIFF-003: Temp file is removed via `defer os.Remove` regardless of failure
**Evidence:** popup.go:223-227.
**Confidence:** HIGH

### BC-GUI-DIFF-004: External `git` dependency at runtime for diff rendering
**Postconditions:** If `git` is not on PATH, every Write/Edit popup renders an error string `(no differences or error: exec: "git": ...)`. No graceful fallback.
**Evidence:** popup.go:247 + 252-254.
**Confidence:** HIGH — **runtime dependency gap**. Should be documented in any porter spec.

## Dialog state machine

`DialogState` (dialog.go:28-65) is a flat struct that holds all input-dialog state. `DialogKind` enum (dialog.go:11-24) has 12 values:

| Kind | Purpose | Focus view |
|---|---|---|
| `DialogNone` | none | — |
| `DialogRename` | rename session | `rename-input` |
| `DialogWorktree` | new worktree (branch + prompt) | `worktree-branch` or ActiveField |
| `DialogWorktreeChooser` | select existing worktree | `worktree-chooser` |
| `DialogWorktreeResume` | resume worktree (prompt only) | `worktree-resume-prompt` or ActiveField |
| `DialogKeybindHelp` | Telescope-style help | `keybind-help-input` |
| `DialogSearch` | `/` filter on active panel | `search-input` |
| `DialogConnect` | connect to remote (host input) | `connect-input` |
| `DialogConnectChooser` | SSH host chooser | `connect-chooser` |
| `DialogAskpass` | masked password prompt | `askpass-input` |
| `DialogProfile` | profile chooser + options | `profile-chooser` or ActiveField |
| `DialogRemoteProfileError` | remote config parse error | `remote-profile-error` |

### BC-GUI-DIALOG-001: HasActiveDialog == (Kind != DialogNone)
**Evidence:** dialog.go:68-70.
**Confidence:** HIGH

### BC-GUI-DIALOG-002: dialogFocusView maps Kind → gocui view name; multi-field dialogs read DialogState.ActiveField
**Postconditions:** Worktree, WorktreeResume, and Profile dialogs use the explicit `ActiveField` (empty → default field). Used by layoutMain to restore focus.
**Evidence:** dialog.go:80-115.
**Confidence:** HIGH

### BC-GUI-DIALOG-003: isChooserView identifies non-editable chooser views: worktree-chooser, connect-chooser, profile-chooser, worktree-profile-chooser, worktree-resume-profile-chooser, remote-profile-error
**Postconditions:** Chooser views suppress gocui cursor; text-input views show cursor.
**Evidence:** dialog.go:121-132.
**Confidence:** HIGH

### BC-GUI-DIALOG-004: ActiveFilter persists separately from SearchQuery
**Postconditions:** SearchQuery is live during the dialog; on Enter it migrates to ActiveFilter (persists after dialog close). On Esc, both clear. Empty query on Enter clears ActiveFilter.
**Evidence:** dialog.go:48-54 + search.go:78-117.
**Confidence:** HIGH

### BC-GUI-DIALOG-005: closeSearch with cancel=true restores SearchPreCursor; cancel=false (Enter) persists filter
**Postconditions:** Cursor restoration **deferred until after filter clear** to avoid currentNode() seeing the filtered tree with a pre-search cursor that may lie outside the filtered range (search.go:87-92 comment).
**Evidence:** search.go:78-134.
**Confidence:** HIGH — load-bearing ordering rationale.

## Key forwarding flow

`state.go:25-42` defines `resolveSessionTarget`:

```go
t := sess.TmuxWindow
if t == "" {
    windowName := "lc-" + id     // fallback: rebuild from session ID
    if len(id) > 8 {
        windowName = "lc-" + id[:8]
    }
    return "lazyclaude:" + windowName
}
return "lazyclaude:" + t
```

### BC-GUI-FWD-001: When TmuxWindow is empty, reconstruct as `"lazyclaude:lc-<id[:8]>"`
**Postconditions:** Matches the `WindowName()` convention from session/session.go (BC-SESSION-001). Provides resilience if SessionItem.TmuxWindow isn't populated.
**Evidence:** state.go:30-41.
**Confidence:** HIGH

### BC-GUI-FWD-002: resolveForwardTarget returns "" when not in fullscreen, no forwarder, OR popup visible
**Postconditions:** Key forwarding is gated by all three conditions. Popup mode blocks key forwarding completely.
**Evidence:** state.go:47-52.
**Confidence:** HIGH

### BC-GUI-FWD-003: forwardKey enqueues a LITERAL one-rune string via EnqueueLiteral
**Postconditions:** Even single-rune ASCII keys go through SendKeysLiteral, not SendKeys. This matters because SendKeysLiteral escapes `\` and `"` and rejects `\n \r \x00`. Rune `\n` from the keyboard never reaches this path because gocui delivers Enter as a key code, not a rune.
**Evidence:** state.go:54-61 + RuneToLiteral input.go:113-115.
**Confidence:** HIGH

### BC-GUI-FWD-004: forwardSpecialKey enqueues a non-literal key name via EnqueueKey
**Evidence:** state.go:63-70.
**Confidence:** HIGH

### BC-GUI-FWD-005: forwardPaste runs synchronously (not via the queue)
**Postconditions:** Bypasses keyQueue ordering because paste text has already been accumulated atomically by the gocui paste aggregator. Direct call to `forwarder.ForwardPaste` (tmux load-buffer + paste-buffer).
**Evidence:** state.go:72-84.
**Confidence:** HIGH

### BC-GUI-FWD-006: handlePasteContent only forwards when fullscreen AND no popup
**Postconditions:** Empty text is a no-op. Paste into dialogs is TODO (state.go:100).
**Evidence:** state.go:89-101.
**Confidence:** HIGH

## Keybinding registration

`App.setupGlobalKeybindings` (keybindings.go:42-...) registers keys in 6 categories:

1. **Rune keys** — for every rune the registry exposes (`Runes()`), bind to global view `""` with `dispatchRune`. Dispatch chain decides whether to consume.
2. **Special keys** — same pattern for `SpecialKeys()`.
3. **Popup view rune bindings** — duplicate registration on `popupViewName` view. Required because gocui may not deliver runes to global handlers when popup view has focus.
4. **Popup view special key bindings** — same pattern.
5. **Mouse wheel** — fullscreen-only, dispatches to ScrollModeMouseUp/Down.
6. **Dialog Enter/Esc handlers** — view-specific, registered per-dialog input view. These intercept Enter and Esc BEFORE gocui's Editor.Edit() runs.

### BC-GUI-KEYREG-001: Rune and special keys auto-register from the keymap.Registry; no hand-coded key tables
**Postconditions:** Registry.Runes() and Registry.SpecialKeys() drive registration. Adding a new keymap action means one Register call; no separate keybinding step.
**Evidence:** keybindings.go:43-55.
**Confidence:** HIGH

### BC-GUI-KEYREG-002: Popup view re-registers ALL popup-scope runes and specials so they receive keys when popup has focus
**Evidence:** keybindings.go:59-69.
**Confidence:** HIGH

### BC-GUI-KEYREG-003: Mouse wheel up/down ONLY active in fullscreen mode
**Postconditions:** Mouse wheel in main mode is unbound; tmux/terminal pass-through.
**Evidence:** keybindings.go:71-87.
**Confidence:** HIGH

### BC-GUI-KEYREG-004: Dialog Enter/Esc handlers register on the specific input view name, so they intercept BEFORE Editor.Edit() can consume
**Postconditions:** Pattern: register on view `"rename-input"`, `"connect-input"`, etc. gocui dispatches view-specific bindings before the editor.
**Evidence:** keybindings.go:89-95 explicit comment; keybindings.go:96-120 for rename-input example.
**Confidence:** HIGH

### BC-GUI-KEYREG-005: quitRequested flag is checked after every Dispatch; if set, returns gocui.ErrQuit
**Postconditions:** Quit action sets the flag; the gocui handler returns ErrQuit which terminates MainLoop.
**Evidence:** keybindings.go:20-24, 33-37.
**Confidence:** HIGH

## Keymap registry shape

`keymap.Registry` (registry.go:7-9) holds a flat slice of `ActionDef`. Lookup is **linear scan** (no map). Per-scope Matchers:

- `Match(rune, key, mod, scope) → (ActionDef, bool)` — first match wins.
- `MatchTab(rune, key, mod, scope, tab) → (ActionDef, bool)` — tab-filtered version; TabAll matches any.
- `HintsForScope(scope) → []ActionDef` — actions with non-empty HintLabel, in registration order.
- `HintsForScopeTab(scope, tab)` — tab-filtered hints.
- `BindingsForScopeTab(scope, tab)` — all actions in scope+tab.
- `BindingsForScope(scope)` — all actions in scope.

### BC-GUI-REG-001: Lookup is O(N) linear scan over all action defs
**Postconditions:** For ~50-100 action defs and one key per event, this is fine. No optimization warranted.
**Evidence:** registry.go:22-33.
**Confidence:** HIGH

### BC-GUI-REG-002: First match in registration order wins for ambiguous bindings
**Postconditions:** Order of `Register` calls determines precedence.
**Evidence:** registry.go:22-33 (early return on first match).
**Confidence:** HIGH

### BC-GUI-REG-003: TabAll is a sentinel value matching any tab
**Evidence:** registry.go:43.
**Confidence:** HIGH

### BC-GUI-REG-004: HintsForScope filters by non-empty HintLabel (hints with empty labels are hidden from options bar)
**Evidence:** registry.go:57-65.
**Confidence:** HIGH

## App_actions plugin/MCP sync

`syncPluginProject` (app_actions.go:171-341) is **enormous** for what it does. It handles four branches:

1. **Empty tree (no nodes)** → reset to CWD fallback; clear remoteDisabled flags; SetRemote("", cwd); refresh.
2. **Nil node (transient: filter hides everything)** → preserve previous state.
3. **Remote node** → set pluginState.remoteDisabled=true; MCP atomically switches to remote project path via SetRemote(host, path); dedupe key prevents re-refresh on cursor movement within same project.
4. **Local node** → clear remoteDisabled; unconditionally SetRemote("", projectPath); refresh only when projectPath changed.

### BC-GUI-SYNC-001: Empty-tree recovery resets pluginState AND mcpState cursors to 0 to avoid out-of-range cursors silently blocking write handlers
**Evidence:** app_actions.go:204-249.
**Confidence:** HIGH

### BC-GUI-SYNC-002: Transient nil-node (filter hides everything) preserves previous projectDir and remoteDisabled flag
**Postconditions:** Write guards' flag fallback keeps writes honest until the tree resolves.
**Evidence:** app_actions.go:250-255 + comment.
**Confidence:** HIGH

### BC-GUI-SYNC-003: Remote-node sync dedupes by `host|projectPath` key to prevent SSH hammer on cursor movement
**Postconditions:** Each MoveCursorUp/Down triggers syncPluginProject; without the key, every cursor move spawns SSH round-trips.
**Evidence:** app_actions.go:281-292.
**Confidence:** HIGH

### BC-GUI-SYNC-004: Local-node sync unconditionally calls SetRemote("", projectPath) BEFORE the refresh short-circuit
**Postconditions:** Ensures remote→local transition into a cached project doesn't leave the MCP manager holding stale (remoteHost, remoteDir). The refresh itself short-circuits when projectPath is unchanged.
**Evidence:** app_actions.go:310-326 + multi-paragraph comment 310-322.
**Confidence:** HIGH — load-bearing ordering rationale.

### BC-GUI-SYNC-005: guardRemoteOp fallback to cached pluginState.remoteDisabled/mcpState.remoteDisabled when no node resolves
**Postconditions:** Decision order: (1) live local → allow, (2) live remote → block, (3) nil node → cached flag.
**Evidence:** app_actions.go:382-405 + comment 355-381.
**Confidence:** HIGH

### BC-GUI-SYNC-006: configDirForSession returns worktree path for worktree sessions; otherwise InferProjectRoot
**Postconditions:** Worktree sessions have project-local config (MCP, plugins, settings) scoped to the worktree.
**Evidence:** app_actions.go:462-470.
**Confidence:** HIGH

## Search dialog

`closeSearch` (search.go:78-134) handles Esc (cancel) vs Enter (commit):

- **Esc**: restore SearchPreCursor for the panel; clear ActiveFilter; defer resyncSessions to after filter-clear.
- **Enter**: persist SearchQuery → ActiveFilter (empty query clears filter).

The `searchInputEditor` (search.go:144+) handles each keystroke and re-filters the panel in-place.

### BC-GUI-SEARCH-001: Search Esc must clear SearchQuery and ActiveFilter BEFORE calling syncPluginProject; otherwise currentNode() sees the filtered tree
**Postconditions:** This is the explicit ordering rationale at search.go:87-92.
**Evidence:** search.go:78-134.
**Confidence:** HIGH

### BC-GUI-SEARCH-002: ActiveFilter persists per-panel (sessions, plugins, logs); clearActiveFilter only clears when panel matches
**Evidence:** search.go:137-142.
**Confidence:** HIGH

### BC-GUI-SEARCH-003: Search dialog view sits at the bottom of the panel rect, 1 row high; no-op when panel is too short
**Evidence:** search.go:18-26.
**Confidence:** HIGH

## Delta Summary

- New items added: 35 (8 BC-GUI-POPUPVIEW, 3 BC-GUI-CHOICE updates, 4 BC-GUI-DIFF, 5 BC-GUI-DIALOG, 6 BC-GUI-FWD, 5 BC-GUI-KEYREG, 4 BC-GUI-REG, 6 BC-GUI-SYNC, 3 BC-GUI-SEARCH)
- Existing items refined: 0 (this round covered new file territory)
- Remaining gaps: app_actions.go tail (lines 470-1481), keybindings.go tail (lines 120-764), render.go tail (lines 200-442), preview.go, render_plugins.go, render_mcp.go, plugin_state.go, mcp_state.go, logs_state.go, tree.go, popup_types_test.go scenarios, chooser/chooser.go (84 LOC, small enough to read full in next round), keyhandler/{actions, fullscreen, global, logs, panel, plugins, sessions}.go (~700 LOC combined), presentation/{diff, logline, mcp, plugins, style, tool}.go.

## Novelty Assessment

Novelty: SUBSTANTIVE
Justification: 35 new contracts covering popup rendering geometry, dialog state machine for 12 dialog kinds, key forwarding gating (fullscreen + popup + forwarder), plugin/MCP sync with explicit transition rationale (remote→local cache reset), and search-dialog ordering. Specifically novel:
- **BC-GUI-DIFF-004** runtime `git` dependency was not documented anywhere in Pass A.
- **BC-GUI-POPUPVIEW-004** 20-slot cleanup cap.
- **BC-GUI-CHOICE-003** silent SendChoice error (observability gap).
- **BC-GUI-SYNC-004** unconditional SetRemote-before-short-circuit.
- **BC-GUI-FWD-003** all rune forwarding uses Literal path.

These are model-changing: a porter must replicate the git-dependence, the 20-slot cap, the SetRemote-first ordering, etc.

## Convergence Declaration

Another round needed — keybindings.go tail (~640 LOC remaining) contains the per-dialog Enter/Esc handlers, fullscreen bindings, popup keymap, scope-specific handlers. app_actions.go tail (~1000 LOC) contains the actual action implementations (CreateSession, ShowProfileDialog, ConfirmWorktree, etc.). render.go tail contains server log render, MCP/plugins panel rendering. Without these, the contract set on GUI is still incomplete.

## State Checkpoint

```yaml
pass: B
subsystem: gui
round: 2
status: complete
files_read_full: [popup.go, dialog.go, state.go, keyhandler/popup.go]
files_read_partial: [app_actions.go (470/1481), keybindings.go (120/764), search.go (150/380), keymap/registry.go (100/848)]
contracts_drafted: 35
timestamp: 2026-05-11T19:50:00Z
novelty: SUBSTANTIVE
next_round: 3
```
