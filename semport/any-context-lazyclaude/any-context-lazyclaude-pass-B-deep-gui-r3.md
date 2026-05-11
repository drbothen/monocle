# Pass B Deep: `internal/gui` — Round 3

**Scope this round:** preview cache concurrency, plugin/MCP/logs state types, scrollback capture pipeline, render.go server-log + popup + scroll content paths, all session/popup/scroll AppActions methods, ScrollMode mouse handling.

**Files read in full this round:** preview.go, plugin_state.go, mcp_state.go, logs_state.go, app_actions.go (lines 470-1482, completing the file), render.go (lines 200-442).

## PreviewCache concurrency contract

`PreviewCache` (preview.go full file) is a small mutex-protected struct used as the preview-panel rendering cache. Every method documents whether the caller must hold the lock.

| Method | Caller must hold lock? | Effect |
|---|---|---|
| `Lock()` / `Unlock()` | n/a | Manual lock control |
| `Content()`, `Cursor()`, `CursorX()`, `CursorY()`, `Busy()` | **YES** | Read-only accessors |
| `Stale(threshold)` | **YES** | `time.Since(fetchAt) > threshold` |
| `SetBusy(b)` | **YES** | Set busy flag |
| `Update(content, cursorIdx, cursorX, cursorY)` | **YES** | Write captured content, clear busy, set fetchAt=now |
| `Invalidate()` | **NO** (acquires itself) | Clear content + fetchAt |
| `InvalidateTimestamp()` | **YES** | Only clear fetchAt, keep content |
| `MarkFetched(cursorIdx)` | **YES** | Clear busy, set fetchAt=now, set cursor — used after empty fetch to prevent retry loop |

### BC-GUI-PREVIEW-001: Invalidate is the only method that acquires the lock internally; all others require external locking
**Postconditions:** Mixed-lock-discipline antipattern. The caller must remember which methods need the lock. Common pattern at callsite: `pc.Lock(); if !pc.Busy() { pc.InvalidateTimestamp() }; pc.Unlock()` (fullscreen.go:228-232).
**Evidence:** preview.go:20-86.
**Confidence:** HIGH

### BC-GUI-PREVIEW-002: MarkFetched updates cursor along with timestamp to prevent needFetch loop
**Postconditions:** Without MarkFetched, an empty pane (e.g. Claude Code startup) would have `Cursor() != cursor` stay true forever, triggering `capture-pane` on every frame.
**Evidence:** preview.go:75-86 + explicit comment 78-81.
**Confidence:** HIGH — load-bearing performance comment.

### BC-GUI-PREVIEW-003: Stale uses `fetchAt`; an Invalidated cache (fetchAt = zero) is unconditionally stale
**Postconditions:** `time.Since(zero).hours()` is astronomically large, always > any threshold.
**Evidence:** preview.go:42-44.
**Confidence:** HIGH

## State holders

### LogsState (logs_state.go full file)

Pure data structure. Manages cursor + selection + line count for the logs panel.

### BC-GUI-LOGS-001: CursorDown is bounded by `lineCount-1`; CursorUp bounded by 0
**Evidence:** logs_state.go:27-38.
**Confidence:** HIGH

### BC-GUI-LOGS-002: ToggleSelect toggles selecting; on Enter, anchors at current cursor
**Evidence:** logs_state.go:53-60.
**Confidence:** HIGH

### BC-GUI-LOGS-003: ClampCursor sets cursor to max valid index (or 0 if no lines)
**Postconditions:** Used after line count changes so cursor stays in range.
**Evidence:** logs_state.go:112-119.
**Confidence:** HIGH

### BC-GUI-LOGS-004: CopyText returns single-line text when not selecting (the cursor line)
**Evidence:** logs_state.go:84-109.
**Confidence:** HIGH

### PluginState (plugin_state.go full file)

Holds plugin panel UI state with **two cursors**: `installedCursor` for the Plugins tab and `marketCursor` for the Marketplace tab. `tabIdx` selects which is active.

### BC-GUI-PSTATE-001: Cursor()/SetCursor() route to installed vs market cursor based on tabIdx == PluginTabMarketplace
**Postconditions:** Switching tabs preserves the per-tab cursor independently.
**Evidence:** plugin_state.go:57-71.
**Confidence:** HIGH

### BC-GUI-PSTATE-002: projectDir is the cached project context; remoteDisabled flag tracks SSH-remote selection
**Evidence:** plugin_state.go:47-49.
**Confidence:** HIGH

### MCPState (mcp_state.go full file)

Holds MCP tab UI state with cursor, loading flag, remoteDisabled flag, and `remoteKey` for SSH-dedupe.

### BC-GUI-MSTATE-001: MCPProvider.SetRemote(host, projectDir) is **atomic** — single combined setter eliminates the mixed-pair race
**Postconditions:** Separate SetHost + SetProjectDir would let a racing async Refresh/ToggleDenied observe `(host=new, projectDir=old)` and mutate the wrong remote file.
**Evidence:** mcp_state.go:17-23 comment + interface signature line 25.
**Confidence:** HIGH — explicitly documented design.

### BC-GUI-MSTATE-002: remoteKey = `host|projectDir` is the dedupe key; only spawn SSH refresh when key changes
**Evidence:** mcp_state.go:34 (struct field) + app_actions.go:281-292 (consumer).
**Confidence:** HIGH

## Session-action methods (app_actions.go)

The action methods follow a consistent pattern: pre-check sessions+dialog, debug-log, then either route to a profile dialog (for create) or spawn an async goroutine that calls the session provider, then gui.Update with success/error.

### BC-GUI-ACTION-001: All Create* actions route through `showProfileDialog(g, kind, path)` rather than calling the provider directly
**Postconditions:** Profile chooser is mandatory for session creation. The actual sessions.Create* call happens after profile dialog confirms.
**Evidence:** app_actions.go:514-519 (CreateSession), 531-535 (CreateSessionAtCWD), 691-695 (StartPMSession).
**Confidence:** HIGH

### BC-GUI-ACTION-002: DeleteSession runs in a goroutine; on success, clamps cursor to `len(nodes)-1` and re-syncs panels
**Postconditions:** Cursor jumps to neighboring node may cross project/host boundaries; syncPluginProject ensures panel state tracks the new cursor.
**Evidence:** app_actions.go:556-586.
**Confidence:** HIGH

### BC-GUI-ACTION-003: LaunchLazygit and AttachSession suspend the gocui Gui (`g.Suspend()`) before exec'ing
**Postconditions:** Releases terminal control so lazygit/ssh has direct tty access. Gui resumed after subprocess exits. Errors during Suspend/Resume are surfaced via showError; the subprocess error itself is also surfaced.
**Evidence:** app_actions.go:588-643.
**Confidence:** HIGH

### BC-GUI-ACTION-004: ConnectRemote prefers SSH config chooser; falls back to manual input dialog when ~/.ssh/config has no hosts
**Postconditions:** ParseSSHHosts errors are surfaced as a status message but don't block the manual-input fallback.
**Evidence:** app_actions.go:762-795.
**Confidence:** HIGH

### BC-GUI-ACTION-005: connectToHost is fire-and-forget after setStatus; the async result updates status or error
**Evidence:** app_actions.go:740-760.
**Confidence:** HIGH

### BC-GUI-ACTION-006: Quit sets `quitRequested = true`; the gocui dispatchRune/dispatchKey wrapper detects it and returns ErrQuit
**Postconditions:** Two-step pattern (flag + post-check) lets the AppActions interface be pure-state without exposing gocui.
**Evidence:** app_actions.go:1481 + keybindings.go:19-24, 33-37.
**Confidence:** HIGH

## Scroll mode AppActions

The scroll mode is a sub-mode of fullscreen. All scroll actions follow the pattern:
1. Apply state change (CursorDown/Up, ScrollUp/Down, etc.).
2. `BumpGeneration()` to invalidate in-flight async captures.
3. `captureScrollbackAsync()` to fetch new content.

### BC-GUI-SCROLLACT-001: ScrollModeUp/Down implement vim-like cursor+scroll
**Postconditions:** Cursor moves first; only when at edge does the viewport scroll. ScrollDown is no-op when scrollOffset == 0 (already at live position).
**Evidence:** app_actions.go:1229-1257.
**Confidence:** HIGH

### BC-GUI-SCROLLACT-002: ScrollModeHalfUp/Down scroll by ViewHeight/2; cursor position preserved relative
**Evidence:** app_actions.go:1259-1272.
**Confidence:** HIGH

### BC-GUI-SCROLLACT-003: ScrollModeToTop calls `captureScrollbackToTop` which fetches HistorySize FIRST to compute correct offset
**Postconditions:** Without this, first `g` press after mouse-wheel entry (which skips HistorySize) would scroll to an unknown position.
**Evidence:** app_actions.go:1274-1278, captureScrollbackToTop at 1401-1436.
**Confidence:** HIGH

### BC-GUI-SCROLLACT-004: ScrollModeMouseUp inlines Enter logic to avoid double BumpGeneration
**Postconditions:** Comment at 1306-1313 explains: calling ScrollModeEnter from here would bump twice, invalidating its own in-flight capture, leaving the view stuck on "Loading...".
**Evidence:** app_actions.go:1314-1325.
**Confidence:** HIGH

### BC-GUI-SCROLLACT-005: ScrollModeMouseDown auto-exits scroll mode when scrollOffset reaches 0
**Postconditions:** Smooth transition back to live preview on mouse-scroll-down at the bottom.
**Evidence:** app_actions.go:1329-1340.
**Confidence:** HIGH

### BC-GUI-SCROLLACT-006: ScrollModeCopy strips ANSI before copyToClipboard, then exits scroll mode
**Postconditions:** Clipboard text is plain. ANSI escape sequences regex: `\x1b\[[0-9;]*[a-zA-Z]`.
**Evidence:** app_actions.go:1290-1300, 1342-1347.
**Confidence:** HIGH

### BC-GUI-SCROLLACT-007: BumpGeneration is called inside ScrollModeExit/Copy so any async capture racing in-flight cannot SetLines after exit
**Postconditions:** This is the contract that fills BC-GUI-SCROLL-005 in round 1 — the caller (here) enforces generation discipline.
**Evidence:** app_actions.go:1221-1227 (Exit), 1295-1300 (Copy).
**Confidence:** HIGH

### BC-GUI-SCROLLACT-008: captureScrollback* goroutines guard SetLines with `a.scroll.Generation() != gen` to drop stale results
**Postconditions:** Same `gen` captured at goroutine start; check inside `gui.Update` callback before SetLines. The check + SetLines are both on the gocui goroutine, so the generation cannot change between check and SetLines.
**Evidence:** app_actions.go:1380-1392, 1422-1435, 1451-1459.
**Confidence:** HIGH

## Popup AppActions

### BC-GUI-POPUPACT-001: UnsuspendPopups only acts when `popupCount() > 0 && !hasPopup()` (i.e. all suspended)
**Postconditions:** Prevents accidental unsuspend when popups already visible.
**Evidence:** app_actions.go:825-829.
**Confidence:** HIGH

### BC-GUI-POPUPACT-002: PopupScrollDown clamps to MaxScroll; PopupScrollUp clamps to 0
**Postconditions:** ViewportHeight fallback to 20 when not yet set by layout.
**Evidence:** app_actions.go:833-855.
**Confidence:** HIGH

### BC-GUI-POPUPACT-003: SendKeyToPane (non-fullscreen send) launches a goroutine and discards the error
**Postconditions:** Used for actions like "send Enter to the pane from the side panel." Fire-and-forget.
**Evidence:** app_actions.go:867-878.
**Confidence:** HIGH

## Logs AppActions

### BC-GUI-LOGSACT-001: LogsClear truncates the server log file in place; concurrent server writes may produce a sparse file
**Postconditions:** Best-effort: `os.Truncate(serverLogPath, 0)`. Comment at 897-900: "the server logger may be writing concurrently via its own *os.File handle, so the file position may be stale after truncation."
**Evidence:** app_actions.go:896-909.
**Confidence:** HIGH

### BC-GUI-LOGSACT-002: After truncate, the log caches are reset (modTime=-1 forces re-read)
**Evidence:** app_actions.go:904-908.
**Confidence:** HIGH

## Tab management

### BC-GUI-TAB-001: PanelNextTab/PrevTab clamp to [0, TabCount-1] (no wrap); call panel.OnTabChanged on transition
**Evidence:** app_actions.go:913-939.
**Confidence:** HIGH

### BC-GUI-TAB-002: panelTabs map[string]int stores per-panel active tab index, keyed by panel name
**Postconditions:** Different panels have independent tab state.
**Evidence:** app.go:142 + app_actions.go:919, 933, 946.
**Confidence:** HIGH

## Help dialog

### BC-GUI-HELP-001: ShowKeybindHelp populates HelpAllItems from BindingsForScopeTab(panel scope, tab) + BindingsForScope(ScopeGlobal)
**Postconditions:** Active panel's scope-tab combo + global bindings are shown. Initial HelpItems == HelpAllItems (no filter).
**Evidence:** app_actions.go:1151-1168.
**Confidence:** HIGH

## Plugin/MCP refresh pattern

`runPluginAsync` and `runMCPAsync` (app_actions.go:1060-1071, 1128-1140) follow identical pattern:
1. Set `loading = true` synchronously.
2. Spawn goroutine, call `fn(context.Background())`.
3. On result, `gui.Update` to clear loading + show error.

### BC-GUI-PASYNC-001: runPluginAsync/runMCPAsync use context.Background() (no caller-supplied context)
**Postconditions:** No cancellation when the user switches tabs or moves cursor. Pending operation completes against the original project even if the user has navigated away. Race with the dedupe logic is mitigated by `pluginState.projectDir` check at the next refresh trigger.
**Evidence:** app_actions.go:1063, 1131.
**Confidence:** HIGH — **observability + correctness gap**. P2 — would benefit from context with timeout.

### BC-GUI-PASYNC-002: Loading state is bool flag, not a counter; concurrent runs would overwrite each other's "loading=false"
**Postconditions:** First completed run clears loading even if second is still pending.
**Evidence:** app_actions.go:1060-1071. No counter.
**Confidence:** HIGH — minor bug-like edge case.

## Render pipeline (tail)

### BC-GUI-RENDER-001: renderServerLog skips re-render when all cache fields match (modTime, focused, cursor, selStart, selEnd, width, searchQuery)
**Postconditions:** Significant CPU savings — re-render only on actual state change.
**Evidence:** render.go:236-252.
**Confidence:** HIGH

### BC-GUI-RENDER-002: scrollToCursor adjusts view origin then sets cursor RELATIVE to origin
**Postconditions:** gocui's SetCursor is origin-relative; origin must be finalized first or cursor lands in wrong row.
**Evidence:** render.go:286-297 + explicit comment 283-285.
**Confidence:** HIGH

### BC-GUI-RENDER-003: renderToolPopup writes ALL content lines and uses SetOrigin to scroll; gocui handles scrollbar from ViewLinesHeight
**Postconditions:** Differs from "scroll window" pattern. Full content always in view, scroll just changes the visible portion.
**Evidence:** render.go:302-308.
**Confidence:** HIGH

### BC-GUI-RENDER-004: renderDiffPopup applies hard-coded ANSI color codes per DiffLineKind: 32 (add), 31 (del), 36 (hunk), 2 (dim file path)
**Evidence:** render.go:312-333.
**Confidence:** HIGH

### BC-GUI-RENDER-005: truncateToWidth uses runewidth.RuneWidth (not ansi-aware); padRightANSI uses ansi.StringWidth
**Postconditions:** Two padding paths: ANSI-aware (for scroll content) and rune-aware (for plain text). Mixing would mis-pad ANSI-containing lines.
**Evidence:** render.go:338-348, 426-432, 436-442.
**Confidence:** HIGH

### BC-GUI-RENDER-006: renderScrollContent uses ansi.Truncate + padRightANSI to preserve ANSI escapes; non-scroll renderServerLog uses runewidth.StringWidth (no ANSI in logs)
**Evidence:** render.go:351-393 (scroll), 254-280 (logs).
**Confidence:** HIGH

### BC-GUI-RENDER-007: scrollback "Loading..." placeholder resets origin (0,0) and cursor (0,0) to prevent stale values from non-scroll mode
**Evidence:** render.go:353-358.
**Confidence:** HIGH

## Delta Summary

- New items added: 36 (3 BC-GUI-PREVIEW, 4 BC-GUI-LOGS, 2 BC-GUI-PSTATE, 2 BC-GUI-MSTATE, 6 BC-GUI-ACTION, 8 BC-GUI-SCROLLACT, 3 BC-GUI-POPUPACT, 2 BC-GUI-LOGSACT, 2 BC-GUI-TAB, 1 BC-GUI-HELP, 2 BC-GUI-PASYNC, 7 BC-GUI-RENDER)
- Existing items refined: 1 (BC-GUI-SCROLL-005 from r1: confirmed caller-side enforcement at BumpGeneration → captureScrollback callsites)
- Remaining gaps: render_plugins.go, render_mcp.go (sub-renders), tree.go (TreeNode + filtered* helpers), keyhandler/{actions, fullscreen, global, logs, panel, plugins, sessions}.go (~700 LOC combined — these are the panel-specific HandleKey implementations), keymap/{registry, types, doc}.go body, chooser/chooser.go (84 LOC), presentation/{diff, logline, mcp, plugins, style, tool}.go, popup_types_test.go scenarios, keybindings.go tail (lines 120-764 — dialog Enter/Esc handlers).

## Novelty Assessment

Novelty: SUBSTANTIVE
Justification: 36 new contracts covering:
- **PreviewCache contract**: mutex discipline pattern + MarkFetched anti-loop pattern.
- **Scroll mode pipeline**: BumpGeneration + captureScrollbackAsync with stale-drop pattern (BC-GUI-SCROLLACT-007/008).
- **Plugin/MCP async pattern**: `context.Background()` no-cancel + bool flag race (BC-GUI-PASYNC-001/002 — both **new bug-like findings** not previously documented).
- **Mouse scroll inline-Enter** (BC-GUI-SCROLLACT-004) — non-obvious anti-pattern avoidance.
- **MCPProvider.SetRemote atomicity** (BC-GUI-MSTATE-001) — load-bearing race avoidance.
- **Render cache predicates** (BC-GUI-RENDER-001) — explicit field-by-field cache key.
- **scrollToCursor origin-first ordering** (BC-GUI-RENDER-002) — gocui-specific gotcha.

These materially change how a porter would model the preview pipeline, scroll mode, and async ops.

## Convergence Declaration

Another round needed — keybindings.go tail (~640 LOC) contains 15+ dialog Enter/Esc handlers with confirm/cancel semantics for every dialog kind (rename, worktree, worktree-resume, profile, askpass, connect, etc.). These define the dialog→action flow. keyhandler subpackage files (~700 LOC) contain the panel-specific HandleKey implementations. Without these, the dialog state machine and panel-key dispatch are incomplete.

## State Checkpoint

```yaml
pass: B
subsystem: gui
round: 3
status: complete
files_read_full: [preview.go, plugin_state.go, mcp_state.go, logs_state.go, app_actions.go]
files_read_partial: [render.go (full now), keybindings.go (120/764)]
contracts_drafted: 36
timestamp: 2026-05-11T20:10:00Z
novelty: SUBSTANTIVE
next_round: 4
```
