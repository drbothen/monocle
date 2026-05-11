# Pass B Deep: `internal/gui` — Round 1

**Scope:** popup state machine, fullscreen + scroll modes, layout pipeline, key dispatch chain, notify/refresh loop, render caches.
**Source LOC:** ~18,276 (Pass 0 recount). Files read in full this round: app.go, layout.go (head), render.go (head), notify_loop.go, scroll_state.go, popup_controller.go, popup_types.go, fullscreen.go, input.go (head), keydispatch/dispatcher.go, keyhandler/popup.go.

## Run loop architecture

`App.Run` (app.go:252-389) starts **two goroutines** plus the gocui MainLoop:

1. **Key forwarder** goroutine: `fullscreen.RunKeyForwarder(done)` (app.go:260) — drains `fs.keyQueue` (chan keyCmd, cap 1024) and dispatches via the injected `InputForwarder`. Adjacent literal commands for the same target are batched into a single `ForwardLiteral` call (fullscreen.go:120-155, BC-GUI-FS-002).
2. **Refresh loop** goroutine: 100 ms ticker + broker-event select (app.go:263-378). Three paths per cycle:
   - **OutputCh** (control-mode `%output`): sets `outputPending=true` so the next tick coalesces multiple output bursts into one preview invalidation. This explicitly prevents the "CPU 100%+ over time" failure (app.go:269-272 comment).
   - **BrokerCh** (event.Subscription, buffer **8** — notify_loop.go:44): dispatches per-event variant to `setWindowActivity` + `showToolPopup`. All gocui state mutation is funneled through `gui.Update`.
   - **Ticker (100 ms fallback)**: invalidates preview if outputPending, calls `OnTick` (control-mode health check from root.go), refreshes session list async, drains `PendingNotifications` (remote SSE-buffered + local file-based when broker absent), and **calls showToolPopup for each** — this is the file-polling fallback path that the daemon-mode user depends on.

`done` channel closes when `gui.MainLoop` exits, draining both goroutines (app.go:380-388). The control-mode health-check tick callback is set by root.go via `app.SetOnTick`, not in this package.

### BC-GUI-RUN-001: Refresh-loop output coalescing
**Preconditions:** Control mode emits `%output` events, each calling `App.NotifyOutput()`.
**Postconditions:** Each ticker cycle invalidates the preview at most once regardless of how many output events arrived. Without this, "every tmux %output line triggers a CapturePreview goroutine, which spawns tmux subprocesses and drives CPU to 100%+ over time" (app.go:269-272 explicit comment).
**Evidence:** app.go:273, 281-283, 337-345.
**Confidence:** HIGH

### BC-GUI-RUN-002: refreshSessionsAsync is single-flight by `sessionRefreshing` flag without mutex
**Postconditions:** Skipped if already in flight; otherwise spawns a goroutine that calls `sessions.Sessions()` and updates `cachedSessionItems` via `gui.Update`. Flag is single-threaded on the event loop goroutine, no mutex.
**Evidence:** app.go:541-553. Comment "IMPORTANT: Must only be called from the gocui event loop goroutine ... has no mutex protection and relies on single-threaded access."
**Confidence:** HIGH

### BC-GUI-RUN-003: Broker subscription buffer is hard-coded to 8
**Postconditions:** Under bursts >8 unread events (e.g., parallel MultiEdit on many files), `broker.Publish` drops on subscriber-full (BC-BROKER-003). The drop is silent at the GUI level.
**Evidence:** notify_loop.go:44 (`broker.Subscribe(8)`).
**Confidence:** HIGH — **this is the realization of the P0 risk** "Tool notification dropped when broker subscriber buffer full" flagged in pass-4-verification-gaps.md and seed 3 in pass-6. The constant has no test exercising backpressure.

### BC-GUI-RUN-004: File-polling fallback runs unconditionally every tick
**Postconditions:** Even when broker is wired, the ticker block (app.go:347-374) calls `sessions.PendingNotifications()` which for remote-only providers buffers SSE events. The file-based fallback only fires when no broker is wired (since `notify.Enqueue` is skipped when `s.notifyBroker.HasSubscribers()` returns true — BC-MCPSRV-008). Net: no double-delivery for local sessions; remote SSE-buffered notifications come through this path because they are not in the local broker.
**Evidence:** app.go:347-373; explicit comment 352-357.
**Confidence:** HIGH

## Popup stack state machine

`PopupController` (popup_controller.go:28-211) manages a LIFO stack with focus tracking and suspend/unsuspend semantics:

```
stack: [popupEntry{popup, suspended}]
focusIdx: int (index into stack)
```

Operations:

| Op | Effect on stack | Effect on focusIdx |
|---|---|---|
| `PushPopup(p)` | append entry | `focusIdx = len(stack)-1` (focus new) |
| `DismissActive(c)` | delete at focusIdx | clamp to `len(stack)-1`; if landed on suspended, FocusNext |
| `DismissAll(c)` | nil | reset to 0 |
| `SuspendAll` | all `.suspended = true` | unchanged |
| `UnsuspendAll` | all `.suspended = false` | reset to `len(stack)-1` |
| `FocusNext`/`FocusPrev` | unchanged | rotate to next non-suspended (wrap) |

### BC-GUI-POPUP-004: PushPopup always grabs focus
**Postconditions:** A new popup arriving during user interaction with an existing popup will steal focus. No queueing.
**Evidence:** popup_controller.go:45-48 (`focusIdx = len(pc.stack) - 1`).
**Confidence:** HIGH

### BC-GUI-POPUP-005: After DismissActive, focusIdx clamps; if the clamped slot is suspended, advances to next visible
**Evidence:** popup_controller.go:109-115. Pattern: after popping element at focusIdx, clamp focusIdx, then if suspended → FocusNext.
**Confidence:** HIGH

### BC-GUI-POPUP-006: SuspendAll preserves focusIdx but UnsuspendAll resets focusIdx to top
**Postconditions:** Round-trip Suspend → Unsuspend does **not** restore the previously-focused popup unless it was the last in the stack.
**Evidence:** popup_controller.go:134-148. `SuspendAll` does not touch focusIdx; `UnsuspendAll` sets `focusIdx = len(stack)-1`.
**Confidence:** HIGH — confirms seed 7 from Pass 6.

### BC-GUI-POPUP-007: notificationFromPopup is type-switch over `*ToolPopup` and `*DiffPopup` only
**Postconditions:** Any other Popup implementation yields nil. Used by `ActiveNotification()` for badge rendering.
**Evidence:** popup_controller.go:203-212.
**Confidence:** HIGH

### BC-GUI-POPUP-008: `Popup.ID()` is window+toolname (ToolPopup) or window+oldFilePath (DiffPopup)
**Postconditions:** ID is a stable composite that allows deduplication-by-key. **Not enforced unique** at PushPopup — duplicates can stack.
**Evidence:** popup_types.go:52-54, 137-139.
**Confidence:** HIGH

### BC-GUI-POPUP-009: DiffPopup lazily computes the unified diff on first ContentLines/ContentKinds call; cached in struct fields
**Postconditions:** `ensureCache()` is single-threaded (must be called from gocui layout goroutine only — popup_types.go:206-207 comment). No mutex. Reading `lines/kinds` from another goroutine without first hitting ContentLines on the layout goroutine races.
**Evidence:** popup_types.go:207-248.
**Confidence:** HIGH

### BC-GUI-POPUP-010: DiffPopup formats file path as first line, prepends blank line, then formats diff hunks skipping `DiffHeader` lines
**Postconditions:** Hunks are separated by blank lines (one before each subsequent hunk). Line number column width is computed from the max line number across all parsed lines.
**Evidence:** popup_types.go:219-248.
**Confidence:** HIGH

## Fullscreen + scroll

`FullScreenState` (fullscreen.go:11-234) is fundamentally:

- `active bool` — entering/exiting is idempotent (Enter no-ops if active; Exit no-ops if not active).
- `target string` — session ID.
- `scrollY int` — mouse scroll offset, 0-clamped on ScrollUp.
- `keyQueue chan keyCmd` — capacity **1024**.
- `forwarder InputForwarder` — injected via SetForwarder.

Two key-input paths:

1. **EnqueueKey(target, name)** — pushes a non-literal cmd into keyQueue. Dispatched via `forwarder.ForwardKey` → `client.SendKeys` (input.go:33-35) which translates names like "Enter", "Space" to tmux escape sequences.
2. **EnqueueLiteral(target, text)** — pushes literal text. Dispatched via `forwarder.ForwardLiteral` → `client.SendKeysLiteral` which is the escaped-string path (control.go:182-185).

### BC-GUI-FS-004: EnqueueKey and EnqueueLiteral are non-blocking; dropped silently on queue full
**Evidence:** fullscreen.go:84-89, 93-98.
**Confidence:** HIGH

### BC-GUI-FS-005: dispatchBatch coalesces consecutive same-target literal cmds into a single ForwardLiteral
**Postconditions:** A paste of N characters that arrives as N separate `EnqueueLiteral` calls is forwarded as ONE call (buffer.String() concatenation). Critical for paste performance and IME input order preservation.
**Evidence:** fullscreen.go:120-155. Comment: "Serial key forwarder: preserves keystroke order (critical for IME input)."
**Confidence:** HIGH

### BC-GUI-FS-006: A mix of literal and non-literal in the queue flushes literals first, then dispatches the non-literal key
**Postconditions:** Order of dispatch is preserved (no reordering); only same-type consecutive runs are batched.
**Evidence:** fullscreen.go:127-150 (the inner select default branch).
**Confidence:** HIGH

### BC-GUI-FS-007: TriggerRefresh after key input only invalidates preview if not Busy AND last update >50ms ago
**Postconditions:** Rapid keypresses don't thrash preview captures. The 50 ms throttle is hard-coded.
**Evidence:** fullscreen.go:225-234.
**Confidence:** HIGH

`ScrollState` (scroll_state.go full file) is an in-memory model with no gocui dependency, designed for independent testability. Used by fullscreen's "scroll mode" sub-state (`/` or similar key).

### BC-GUI-SCROLL-001: Enter sets scrollOffset=0, cursorY=viewHeight-1 (bottom), lines=nil, maxOffset=0
**Evidence:** scroll_state.go:48-57.
**Confidence:** HIGH

### BC-GUI-SCROLL-002: SetLines auto-detects top-of-scrollback when `len(lines) < viewHeight && maxOffset==0`
**Postconditions:** Once `maxOffset` is set (non-zero), it is **never updated** by subsequent SetLines calls. So a session that grows past the captured window after `maxOffset` is locked may allow over-scroll.
**Evidence:** scroll_state.go:126-137.
**Confidence:** HIGH — confirms seed 5 from Pass 6.

### BC-GUI-SCROLL-003: CaptureRange returns (start, end) = (-scrollOffset, viewHeight-1-scrollOffset)
**Postconditions:** Translates to `tmux capture-pane -p -S <start> -E <end>`. Offset 0 → visible area; offset N → N lines into scrollback.
**Evidence:** scroll_state.go:203-207, docstring 196-202.
**Confidence:** HIGH

### BC-GUI-SCROLL-004: CopyText returns selection range, or current line if not selecting; clamped to valid indices
**Evidence:** scroll_state.go:166-191.
**Confidence:** HIGH

### BC-GUI-SCROLL-005: Generation counter is incremented by BumpGeneration; intended for stale-capture discard
**Postconditions:** `SetLines` does NOT consult generation. Caller must check generation match before calling SetLines if discard-stale is desired.
**Evidence:** scroll_state.go:194 (BumpGeneration), 126-137 (SetLines). Documented in struct comment line 16: "incremented on scroll; used to discard stale async results."
**Confidence:** MEDIUM — semantic gap: the discard-stale contract is **caller-side**, not enforced by SetLines.

## Layout pipeline

`App.layout` (layout.go:113-150) runs every frame:

1. `refreshTreeNodes()` — rebuild project/session tree once per frame.
2. `syncPluginProjectOnce()` then `syncPluginProject()` — defensive re-sync to handle out-of-band tree rebuilds (layout.go:122-130 has multi-paragraph rationale).
3. Resize detection: invalidate preview + scrollRenderCache when width/height changes (layout.go:133-138).
4. Branch: `fullscreen.IsActive()` → `layoutFullScreen` else `layoutMain`.
5. Always: `layoutToolPopup(g, maxX, maxY)` (so popups overlay both modes).

`ComputeLayout(width, height)` (layout.go:59-96) computes the four panel rects:

- `splitX = maxX / 3`, clamped `>= 20` and `< maxX-10` (else `maxX/2`).
- Left side height divided into thirds: Sessions / Plugins / Server.
- Main occupies the right side.
- Options bar occupies the bottom 2 rows.
- `Compact: width < 60` (CompactThreshold).

### BC-GUI-LAYOUT-001: splitX is `maxX/3` clamped to `[20, maxX-10]`, falling back to `maxX/2` if clamp would exceed bounds
**Evidence:** layout.go:63-69.
**Confidence:** HIGH

### BC-GUI-LAYOUT-002: Compact mode kicks in below width 60
**Evidence:** layout.go:55, 71.
**Confidence:** HIGH

### BC-GUI-LAYOUT-003: Three left panels split heights exactly into thirds; integer division means remainders go to the bottom panel
**Postconditions:** `thirdH = leftH/3`. `sessions.Y1 = thirdH`. `plugins.Y1 = sessY1 + thirdH = 2*thirdH`. `server.Y1 = maxY-2`. Server panel absorbs all rounding remainder.
**Evidence:** layout.go:74-85.
**Confidence:** HIGH

### BC-GUI-LAYOUT-004: Focus priority (per layout.go:309-332): popup > dialog > active panel
**Postconditions:** When popup is visible, `layoutToolPopup` sets focus. When dialog is active but no popup, the dialog view receives focus. Otherwise, the active panel view is current.
**Evidence:** layout.go:309-332.
**Confidence:** HIGH

### BC-GUI-LAYOUT-005: Scroll render cache deliberately omits scrollOffset because SetLines always bumps linesVersion when offset changes
**Postconditions:** Cache invalidation key is `(linesVersion, cursorY, selecting, selStart, selEnd, width)`. Comment at render.go:170-172 explains the omission.
**Evidence:** render.go:165-180.
**Confidence:** HIGH

### BC-GUI-LAYOUT-006: Cursor clamp to len(nodes)-1 triggers `syncPluginProject` re-sync
**Postconditions:** Without this, the cached `projectDir` would reference the pre-clamp row and the next write keypress would hit stale context (layout.go:189-196 comment).
**Evidence:** layout.go:179-197.
**Confidence:** HIGH

### BC-GUI-LAYOUT-007: layoutFullScreen exits fullscreen if target session not found in cachedSessionItems
**Evidence:** layout.go:359-369.
**Confidence:** HIGH

### BC-GUI-LAYOUT-008: When scroll mode is active inside fullscreen, the gocui view is set Editable=false so Enter doesn't get sent
**Evidence:** layout.go:374-375.
**Confidence:** HIGH

## Key dispatch chain

`keydispatch.Dispatcher.Dispatch` (dispatcher.go:31-58) routes one key event through this priority chain:

1. **Popup** (popup_handler.go full file): if `actions.HasPopup()`, **consumes ALL keys** regardless of whether bound. Otherwise returns Unhandled.
2. **FullScreen special**: only in fullscreen mode.
3. **Active panel**: only outside fullscreen and only in Mode 0 (ModeMain).
4. **Global**: q, Ctrl+C, Tab, Shift+Tab, p, etc.

The priority chain is augmented by gocui's per-view binding logic. Per `.claude/CLAUDE.md`:

```
1. View-specific bindings (popupViewName etc.)
2. Editor.Edit() — only for views with Editable=true
3. Global bindings — but rune keys (ch != 0) skip global bindings on Editable views
```

Note: the gocui-level layer runs **before** the `Dispatcher.Dispatch` we trace here. Inside the popup view's binding, popup_handler is invoked. The "rune keys skip global on Editable views" rule is gocui native, not lazyclaude logic.

### BC-GUI-DISPATCH-001: Popup handler consumes ALL keys when popup visible (even unbound keys)
**Postconditions:** Returns `Handled` even when no action matches in the registry. Prevents leakage to panel/global handlers.
**Evidence:** keyhandler/popup.go:21-51 (note return at line 50 is `Handled` unconditionally inside the `HasPopup` branch).
**Confidence:** HIGH

### BC-GUI-DISPATCH-002: Active panel only runs in `Mode 0 && !IsFullScreen`
**Evidence:** dispatcher.go:43.
**Confidence:** HIGH

### BC-GUI-DISPATCH-003: ActiveOptionsBar returns empty string when popup or fullscreen is active
**Postconditions:** The options bar is hidden in those modes.
**Evidence:** dispatcher.go:62.
**Confidence:** HIGH

### BC-GUI-DISPATCH-004: globalBar starts with a leading ASCII space (BuildOptionsBar contract); panelBar+globalBar concatenation strips it
**Postconditions:** Single visual space between sections.
**Evidence:** dispatcher.go:77-79 + comment.
**Confidence:** HIGH

## Popup actions wired to choice delivery

The action handlers (PopupHandler dispatching to PopupActions interface) map keymap actions to choice delivery:

| ActionPopup* | Delivery |
|---|---|
| `Accept` | DismissPopup(choice.Accept) |
| `Allow` | DismissPopup(choice.Allow) |
| `Reject` | DismissPopup(choice.Reject) |
| `AcceptAll` | DismissAllPopups(choice.Accept) |
| `Suspend` | SuspendPopups() |
| `FocusNext` / `FocusPrev` | PopupFocusNext / Prev |
| `ScrollDown` / `ScrollUp` | PopupScrollDown / Up |

### BC-GUI-CHOICE-001: AcceptAll dismisses ALL popups with choice.Accept; individual Accept dismisses only the focused one
**Evidence:** keyhandler/popup.go:32-38.
**Confidence:** HIGH

## InputForwarder strategy

`InputForwarder` (input.go:13-21) is a 3-method interface:
- `ForwardKey(target, key)` — tmux key name (`send-keys`).
- `ForwardLiteral(target, text)` — literal text (`send-keys -l`).
- `ForwardPaste(target, text)` — bracketed paste (`load-buffer` + `paste-buffer`).

Two implementations:
- **TmuxInputForwarder** (input.go:24-43) — production: wraps `tmux.Client`.
- **MockInputForwarder** (input.go:46-110) — test: records all forwarded keys with mutex protection.

### BC-GUI-INPUT-001: PasteToPane is NOT supported via control-mode `tmux.Client` (BC-TMUX-CTL-007); only exec adapter supports it
**Postconditions:** If the InputForwarder is backed by ControlClient, ForwardPaste returns an error. Production wires both: control client for hot-path SendKeys, exec client for paste fallback.
**Evidence:** BC-TMUX-CTL-007 + input.go:42 + control.go:244-246.
**Confidence:** HIGH

## Notify loop API

`NotifyLoop` (notify_loop.go full file) is a small state holder, NOT a goroutine:
- `outputNotify chan struct{}` capacity 1 (non-blocking signal).
- `broker *event.Broker[model.Event]` injected via SetBroker.
- `brokerSub *event.Subscription[model.Event]` created by SetBroker.
- `onTick func()` injected via SetOnTick.

### BC-GUI-NOTIFY-001: SetBroker with nil is a no-op; SetBroker with non-nil subscribes with buffer 8
**Evidence:** notify_loop.go:39-45.
**Confidence:** HIGH

### BC-GUI-NOTIFY-002: SetBroker can be called only once safely; calling twice leaks the prior subscription
**Postconditions:** Re-calling SetBroker overwrites `nl.brokerSub` without cancelling the old one. Documented behavior implies single-call usage (SetBroker says "Must be called before Run()").
**Evidence:** notify_loop.go:39-45 (no `Cancel()` on prior `brokerSub`).
**Confidence:** HIGH — **bug-like edge case** that the contract (must be called before Run) makes acceptable.

### BC-GUI-NOTIFY-003: HasBroker is the dispositive flag for "broker delivery active"; file-polling fallback caller uses it
**Postconditions:** When `HasBroker()` is true, the file-polling path is redundant — the server-side enqueue is suppressed (BC-MCPSRV-008) so ReadAll returns empty.
**Evidence:** notify_loop.go:55-58.
**Confidence:** HIGH

### BC-GUI-NOTIFY-004: NotifyOutput is non-blocking (channel cap 1 + select-default)
**Evidence:** notify_loop.go:30-35.
**Confidence:** HIGH

## Window-activity map

`App.windowActivity map[string]WindowActivityEntry` (app.go:154) is the per-window 5-stage activity state.

### BC-GUI-WACT-001: windowActivity map has NO mutex; all reads/writes are on the gocui event loop goroutine
**Postconditions:** Broker events are dispatched via `gui.Update` (app.go:291, 302, 311, 320, 329) which enqueues to the gocui loop. Layout reads are inherently on the loop. So no race.
**Evidence:** app.go:151-153 comment.
**Confidence:** HIGH

### BC-GUI-WACT-002: setWindowActivity rejects empty window (no-op)
**Evidence:** app.go:502-505.
**Confidence:** HIGH

### BC-GUI-WACT-003: clearUnreadActivity only deletes if state is Idle or Error
**Postconditions:** Running and NeedsInput are preserved across the "unread" clear because they represent active work. The "unread" semantics tie to the unread indicator in the sidebar (the user dismissed the notification, but the agent might still be working).
**Evidence:** app.go:511-522.
**Confidence:** HIGH

### BC-GUI-WACT-004: stopReasonToActivity maps "error"/"interrupt" → Error; everything else → Idle
**Evidence:** app.go:525-532.
**Confidence:** HIGH

## Async preview pipeline (high-level)

`App.preview` is a `PreviewCache` (struct in preview.go, not read in this round). It maintains the cached pane snapshot. Invalidation triggers:

- Terminal resize (layout.go:133-138).
- Output debounce tick (app.go:339-343).
- Fullscreen Enter/Exit (fullscreen.go:65-67, 77-79).
- Manual `TriggerRefresh` after fullscreen key (fullscreen.go:225-234).

The capture itself is launched asynchronously; the layout renders from the cache.

### Likely gap: PreviewCache concurrency
`preview.Lock()/Unlock()` is used to guard `Busy()` checks (app.go:339, fullscreen.go:228). PreviewCache must be a mutex-protected struct. Read in round 2 to confirm and extract full contract.

## What still needs deepening (round 2 targets)

1. **app_actions.go (1481 LOC)** — not yet read. This is the giant file with the keybinding handler functions. Likely contains: create-session flow, profile chooser flow, worktree-create flow, MCP-toggle flow, plugin install flow, fullscreen Enter, search dialog flow, askpass dialog flow.
2. **keybindings.go (764 LOC)** — keybinding registration, hint label maps. Needed for full BC-GUI-KEYS coverage.
3. **render.go (442 LOC, tail not read)** — log render, MCP render, plugins render, render*.go variants.
4. **preview.go** — PreviewCache concurrency contract.
5. **state.go, plugin_state.go, mcp_state.go, logs_state.go** — sub-state holders.
6. **search.go (380 LOC)** — search input dialog, query state, panel-scoped filtering.
7. **dialog.go (132 LOC)** — DialogState, askpass, rename, worktree, profile, connect dialogs.
8. **popup.go (273 LOC)** — popup view rendering, layoutToolPopup, popup view creation.
9. **chooser/chooser.go** — chooser widget impl.
10. **keyhandler/** rest — panel.go, sessions.go, plugins.go, logs.go, fullscreen.go, global.go.
11. **keymap/** — registry contract, action labels.
12. **presentation/** — diff rendering, ANSI styles, line formatting (less critical but referenced).

## Delta Summary

- New items added: 32 (8 BC-GUI-RUN/POPUP/LAYOUT, 5 BC-GUI-FS, 5 BC-GUI-SCROLL, 4 BC-GUI-DISPATCH, 4 BC-GUI-NOTIFY, 4 BC-GUI-WACT, others)
- Existing items refined: 1 (BC-GUI-POPUP-007 above strengthens Pass 3 BC-GUI-POPUP-002 from "defensive" to "case-switch falls through to nil for non-Tool/Diff types")
- Remaining gaps: app_actions.go, keybindings.go, render.go tail, preview.go, state.go, search.go, dialog.go, popup.go view layer, chooser/, keyhandler/ rest, keymap/, presentation/

## Novelty Assessment

Novelty: SUBSTANTIVE
Justification: this round produced 32 new behavioral contracts (BC-GUI-RUN, BC-GUI-POPUP-004..010, BC-GUI-FS-004..007, BC-GUI-SCROLL-001..005, BC-GUI-LAYOUT-001..008, BC-GUI-DISPATCH-001..004, BC-GUI-NOTIFY-001..004, BC-GUI-WACT-001..004) including:
- The seed-3 P0 risk confirmed at notify_loop.go:44 (broker buffer = 8).
- A new bug-like edge case (BC-GUI-NOTIFY-002, double SetBroker leaks subscription).
- The Suspend/Unsuspend asymmetry (BC-GUI-POPUP-006, confirms seed 7).
- The scroll-state maxOffset latching (BC-GUI-SCROLL-002, confirms seed 5).
- A new MEDIUM-confidence gap: generation counter is caller-side enforcement (BC-GUI-SCROLL-005).
Removing these would materially change how a porter would build a monocle-equivalent GUI subsystem.

## Convergence Declaration

Another round needed — app_actions.go (1481 LOC), keybindings.go (764 LOC), search.go (380 LOC), popup.go (273 LOC), and several state holders remain unread. These contain substantial behavior surface for the keybind/dialog/popup-render layers.

## State Checkpoint

```yaml
pass: B
subsystem: gui
round: 1
status: complete
files_read_full: [app.go, fullscreen.go, scroll_state.go, popup_controller.go, popup_types.go, notify_loop.go, keydispatch/dispatcher.go, keyhandler/popup.go, input.go]
files_read_partial: [layout.go, render.go]
contracts_drafted: 32
timestamp: 2026-05-11T19:30:00Z
novelty: SUBSTANTIVE
next_round: 2
```
