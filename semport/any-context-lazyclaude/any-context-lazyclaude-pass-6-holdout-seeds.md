# Pass 6 — Holdout Seeds: any-context/lazyclaude

This pass identifies code areas that look "obviously simple" but harbor non-trivial behavior. Each seed is a hypothesis: **this looks like a one-liner but isn't.** Seeds drive Phase 4 holdout evaluation and feed deepening prioritization.

The lazyclaude codebase is Go and the "subtle" patterns tend to be: tmux quoting rules, async ordering between hook/SSE/GUI threads, gocui dispatch order, and SSH command shape. Seeds below mirror these themes.

## Seed 1: `SendKeysLiteral` escaping is "double-backslash + double-quote" only

`internal/core/tmux/control.go:182-185`:

```go
escaped := strings.ReplaceAll(text, `\`, `\\`)
escaped = strings.ReplaceAll(escaped, `"`, `\"`)
_, err := fmt.Fprintf(c.stdin, "send-keys -l -t %s -- \"%s\"\n", target, escaped)
```

Looks like standard string escaping. But:

- `\n \r \x00` already rejected upstream (control.go:163-169) so newlines aren't a vector.
- `;` not rejected in literal text — but the surrounding quotes protect it.
- **Unicode combining characters are not normalized.** A pasted user message containing combining accents will be forwarded verbatim. tmux control mode's parser may or may not handle them; the existing TODO at control.go:176-179 acknowledges this. Risk surface: messages containing `U+0300` combining-grave on top of a literal `"` could in principle slip past `ReplaceAll` if the bytes interleave — they don't in UTF-8, but the comment exists for a reason.
- **The buffer is unbounded.** A 100 MB paste from `OnPasteContent` flows into a single `Fprintf` here. No upper bound. tmux's own `send-keys -l` will accept it; the wire is the pipe to the tmux control process. Backpressure may stall the layout goroutine.
- The line is terminated by `\n` only — no \r — which is fine on POSIX but the comment about "unsafe character %q" for `\r` in payload implies the line protocol itself uses bare `\n`. Cross-check: this aligns with tmux's documented protocol.

**Deepening priority:** P1 — verify large-paste behavior and Unicode edge cases.

## Seed 2: `buildTmuxAttachCommand` uses `shell.Quote` inside an SSH command string

`internal/daemon/remote_provider.go:450-462`:

```go
func buildTmuxAttachCommand(tmuxTarget string) string {
    window := tmuxTarget
    if _, after, ok := strings.Cut(tmuxTarget, ":"); ok {
        window = after
    }
    return fmt.Sprintf(
        "tmux -L lazyclaude set-option -t lazyclaude window-size largest 2>/dev/null; "+
            "tmux -L lazyclaude new-session -t lazyclaude -s attach-$$ "+
            "\\; set-option destroy-unattached on "+
            "\\; select-window -t %s",
        shell.Quote(window),
    )
}
```

Looks like reasonable shell-escaping. But `.claude/CLAUDE.md` SSH command generation section explicitly says: "No nested quoting. Do not use `shell.Quote` inside SSH command strings." Yet here we are.

The returned string is later either base64-encoded by `runSSHInteractive` (remote_provider.go:434-435) OR passed in another shape. If base64-wrapped, the inner quoting is safe (one layer of `eval`). If passed directly as an SSH arg, the outer SSH-shell expansion creates a two-pass quoting environment which is exactly what the CLAUDE.md warning addresses.

**Verification needed:** trace whether `buildTmuxAttachCommand`'s output is always base64-wrapped before reaching `ssh`. If so, the CLAUDE.md note is over-conservative for this site (still worth a comment). If not, this is the active reproducer for the P0 risk.

**Deepening priority:** P0 — verify or refute the .claude/CLAUDE.md warning's applicability at this call site.

## Seed 3: GUI broker subscription buffer = 8

`internal/gui/notify_loop.go:44`: `nl.brokerSub = broker.Subscribe(8)`.

Looks reasonable. But:

- `event.Broker` is documented as **non-blocking publish, drop-on-full** (broker.go:69-74; BC-BROKER-003).
- 8 is a small number. Under burst load — a Claude command that triggers 10+ rapid `PreToolUse` hooks (e.g., MultiEdit, Glob with many matches) — events would queue at the GUI side, and the 9th onward are dropped.
- Dropped events from `Notification` (permission_prompt) would mean **a popup never appears** for that tool call. The user sees a hung claude and no popup.
- This is the realization of P0 risk "Tool notification dropped when broker subscriber buffer full" in pass-4-verification-gaps.md.

The 8 was likely chosen for "small enough to detect backpressure, large enough for normal interactive use." But there's no test for burst-load behavior.

**Deepening priority:** P0 — verify whether this buffer size is adequate for realistic Claude usage patterns, and whether dropping is observable to the user as a missing popup.

## Seed 4: `ToolPopup.MaxOption()` default fallback to 3

`internal/gui/popup_types.go:72-77`:

```go
func (p *ToolPopup) MaxOption() int {
    if p.notification.MaxOption > 0 {
        return p.notification.MaxOption
    }
    return 3
}
```

Looks like a safe default. But:

- `MaxOption` is the count of choices shown to the user (e.g., `1` accept, `2` allow always, `3` reject).
- When the underlying detection (`tmuxadapter.DetectMaxOption`, server.go:497) fails or returns 0, we fall back to 3.
- If claude actually offered only 2 options (e.g., simpler hooks): the user sees a "3" option label that, when pressed, sends `3` into the tmux pane, where claude doesn't recognize it. Net behavior: popup dismisses but the underlying claude dialog stays open. The user thinks they rejected; actually they did nothing.
- The graceful fallback to "3" assumes most popups are 3-option. Empirically true today, but if Claude Code changes its UX, this default becomes wrong silently.

**Deepening priority:** P1 — confirm DetectMaxOption test coverage (Gap-VER-005) and document this fallback chain explicitly.

## Seed 5: `ScrollState.SetLines` auto-detects top-of-scrollback

`internal/gui/scroll_state.go:126-137`:

```go
func (s *ScrollState) SetLines(lines []string) {
    s.lines = lines
    s.linesVersion++
    if len(lines) > 0 && len(lines) < s.viewHeight && s.maxOffset == 0 {
        s.maxOffset = s.scrollOffset
    }
    if len(lines) > 0 && s.cursorY >= len(lines) {
        s.cursorY = len(lines) - 1
    }
}
```

Looks like a defensive update. But:

- `maxOffset` is set ONCE — when `lines < viewHeight` and `maxOffset == 0`. After that, no further auto-update.
- If the scrollback grows between captures (e.g., claude streams output), the user can scroll past the new bottom because we never re-detect.
- The "len < viewHeight" heuristic conflates "top of scrollback" with "small terminal". For viewHeight=80 and a session that's printed 70 lines, every capture returns 70 lines and `maxOffset` gets clamped to current `scrollOffset` — possibly mid-stream.
- The async capture model with `generation` counter (line 17, BumpGeneration:194) means stale results are intended to be discardable, but `SetLines` itself doesn't consult generation.

**Deepening priority:** P1 — verify scroll-state convergence under streaming output.

## Seed 6: `PopupController.DismissAll` discards `choice` argument

`internal/gui/popup_controller.go:121-131`:

```go
func (pc *PopupController) DismissAll(choice Choice) []string {
    entries := make([]popupEntry, len(pc.stack))
    copy(entries, pc.stack)
    pc.stack = nil
    pc.focusIdx = 0
    windows := make([]string, len(entries))
    for i, e := range entries {
        windows[i] = e.popup.Window()
    }
    return windows
}
```

Looks fine. But notice: `choice` parameter is **unused inside the function**. The caller is expected to use the returned window IDs to send the choice elsewhere. So semantically `DismissAll` "promises" to dismiss-with-choice but only the caller's loop applies the choice. If a caller forgets the iteration, popups vanish without choice delivery.

This is correct-by-design (the layer separation isolates tmux I/O from popup state), but reviewers/porters may assume the function dispatches the choice. The signature lies.

**Deepening priority:** P2 — documentation gap; verify all call sites loop and SendChoice.

## Seed 7: `PopupController.FocusNext` / `FocusPrev` skip suspended entries

`internal/gui/popup_controller.go:151-178`:

```go
for i := 0; i < n; i++ {
    next := (pc.focusIdx + 1 + i) % n
    if !pc.stack[next].suspended {
        pc.focusIdx = next
        return
    }
}
```

Looks like a standard wrap. But:

- If ALL entries are suspended (post-`SuspendAll`), the loop completes without setting `focusIdx` — `focusIdx` retains its previous value pointing at a suspended slot. `ActiveEntry()` then returns nil because the suspended check (popup_controller.go:87) blocks it.
- `UnsuspendAll` (line 141-148) resets `focusIdx = len(stack) - 1` unconditionally — but it does this **after** removing `suspended` from all entries, so the focus lands on the last popup. Fine if the user wanted that, but Suspend/Unsuspend round-trip is not symmetric: original focus position is lost.

**Deepening priority:** P2 — verify Suspend/Unsuspend symmetry expectations in popup_controller_test.go.

## Seed 8: `notificationFromPopup` defaults to nil for non-Tool/Diff popups

`internal/gui/popup_controller.go:203-212`:

```go
func notificationFromPopup(p Popup) *model.ToolNotification {
    switch v := p.(type) {
    case *ToolPopup:
        return v.Notification()
    case *DiffPopup:
        return v.Notification()
    default:
        return nil
    }
}
```

A pristine type switch. But: if a future Popup implementation isn't a Tool or Diff (e.g., a hypothetical "ConfirmDeletePopup"), `ActiveNotification()` returns nil — which means downstream code expecting a notification (for badge updates, etc.) silently no-ops. There's no compile-time guarantee.

**Deepening priority:** P2 — minor; flag for porter awareness.

## Seed 9: `Hook` PID-liveness check via `process.kill(pid, 0)`

From `internal/core/config/hooks.go` (referenced in pass-3-behavioral-contracts.md BC-HOOK-001) and `findAliveLockJS`:

The Node one-liner uses `process.kill(pid, 0)` to test if a PID is still alive. This is the textbook "is the process alive" test.

But:

- PID reuse: if process P with PID 42 exits and the OS assigns 42 to a new unrelated process, `process.kill(42, 0)` succeeds. The hook then POSTs to the port number captured in the (now-stale) lock file — best case wrong server, worst case auth fails and the hook silently drops.
- Time-of-check / time-of-use: liveness checked, port read, POST issued. Between check and POST the original process may exit; the new POST connects to whatever is listening on that port. Auth token mismatch saves us — but only if the new listener happens not to be lazyclaude with the same port (essentially never, but defensive).
- Multiple alive servers: the JS picks "highest port wins." If two lazyclaude instances are running simultaneously (e.g., user opens a second TUI before the first exits), hook events go to one only. This is the documented design (BC-MCPSRV-010 says startup calls `LockManager.CleanAllExcept(port)`) but if the cleanup races a hook firing during the brief overlap, events route to the dying instance.

**Deepening priority:** P1 — race window is small but real; verify lock-cleanup ordering.

## Seed 10: `gc` deletes Dead but NOT Orphan

`internal/session/gc.go` per BC-SESSION-013 in pass 3: "GC runs every 2s, calls Sync, and removes Dead sessions; does NOT delete Orphan." Sync uses `syncFailThreshold = 3` to upgrade transient `HasSession==false` into Dead only after 3 consecutive fails (manager.go:28-29).

This is explicit "fix(gc): do not delete Orphan sessions" per commit log. But:

- An Orphan is "tmux says no but we think there is" (or vice versa). The session never gets cleaned up by GC. It persists in `state.json` forever unless user manually purges (PurgeOrphans = `D` key per README).
- If a user starts and crashes many lazyclaude instances, Orphan sessions accumulate in `state.json` indefinitely. State file grows; sidebar grows.
- The 3-strikes threshold is to absorb transient `HasSession` failures (e.g., tmux server momentarily slow). Under high load, the GC could mark Dead a session that isn't actually dead, OR fail to mark Dead a session that is — depends on which side of the threshold the noisy reads fall.

**Deepening priority:** P1 — state.json bloat / Orphan accumulation is plausible.

## Seed 11: `sessionIDForWindow` substring-matches `lc-<8>` prefix

`internal/daemon/server_sse.go:136-175` per BC-DAEMON-010: "Empty window → empty session_id. Unmatched window → empty session_id. `lc-<8>` prefix matches against session UUID prefix."

The 8-char prefix on `lc-` matches the first 8 chars of UUID v4 (which has 32 hex digits + 4 dashes). UUID prefix collision probability for 8 chars: 1 in 2^32 = ~4 billion. Practically zero for typical session counts.

But:

- The match is `strings.HasPrefix(uuid.String(), prefix)` — uniqueness assumed.
- If a future change introduces non-UUID session IDs (sequential, custom-suffix, etc.), the 8-char prefix may not be unique. The match logic doesn't enforce UUID shape.
- The reverse: a session UUID prefix-match against `lc-<8>` is performed by stripping the `lc-` prefix. What if a session ID legitimately starts with `lc-`? Today UUID v4 doesn't, but a custom-format session ID could.

**Deepening priority:** P2 — defensive coding gap; flag for porter awareness.

## Seed 12: `Sync` doesn't mark Orphan on transient failure but Sets `syncFails`

`internal/session/manager.go:147-160` per BC-SESSION-005: "Sync does not mark sessions Orphan on transient `HasSession` failures."

The mechanism: a counter `syncFails` per session, incremented on each transient miss. Only at `syncFails >= syncFailThreshold = 3` does the session transition. But:

- Where is `syncFails` reset? It must be reset on a successful `HasSession==true`. If not, every session eventually accumulates 3 transient fails and gets marked.
- The counter is per-session in-memory only — not persisted in state.json. Restart resets to 0.
- If GC runs every 2s, three consecutive misses = 6 seconds of tmux-flakiness before promotion to Dead. Reasonable.

**Deepening priority:** P1 — verify reset semantics and persistence.

## Seed 13: `WriteHooksSettingsFile` uses `SetEscapeHTML(false)`

`internal/core/config/hooks.go:54-65`, BC-HOOK-003: "The `=>`, `<`, `>`, `&` characters survive JSON encoding so node parses them literally."

This is a Go json package quirk: by default, `json.Marshal` HTML-escapes `<`, `>`, `&`, `=`. For node-eval one-liners that contain arrow functions (`=>`), this breaks parsing. Fix: `enc.SetEscapeHTML(false)`.

But:

- Without `SetEscapeHTML(false)`, `=>` becomes `=>` in the JSON, which **is still valid JSON**. `JSON.parse` in node returns the same `=>` string. So why is it needed?
- The hook command is read by Claude Code (which then exec's it via `node -e`). If Claude Code reads via `JSON.parse`, the escaped form unescapes back to `=>`. The fix may be redundant — OR Claude Code reads via some path that doesn't unescape (e.g., pattern-matching the literal command string).
- Verification: read Claude Code's hook-loading source (out of scope) OR test the behavior end-to-end.

**Deepening priority:** P2 — unclear necessity; flag for verification.

## Seed 14: `Edit` tool notification reads up to 2 MB

`internal/server/server.go:525-555` per BC-MCPSRV-007: "Only reads regular files ≤ 2 MB (skips FIFOs, devices, oversize)."

Looks defensive. But:

- 2 MB is the **input** file size. The Diff output (after `strings.ReplaceAll`) could be larger if `new_string` is large and `replace_all` produces many matches. Net memory usage = old_size + new_size + intermediate strings.
- The DiffPopup then computes a unified diff (presentation.ParseUnifiedDiff via popup_types.go:213) which can blow up further. A user editing a large near-duplicate file pair could exhaust RAM.
- No per-popup memory cap.

**Deepening priority:** P1 — verify diff-render memory footprint for large files.

## Seed 15: `validateControlKey` allows spaces, `validateControlTarget` doesn't

`internal/core/tmux/control.go:188-210`: two near-identical validators that differ in whether `' '` is allowed:

```go
func validateControlTarget(s string) error {
    for _, c := range s {
        switch c {
        case '\n', '\r', ';', ' ':  // space blocked
            return fmt.Errorf("target contains unsafe character %q", c)
        }
    }
    return nil
}

func validateControlKey(s string) error {
    for _, c := range s {
        switch c {
        case '\n', '\r', ';':       // space allowed (Space key)
            return fmt.Errorf("key contains unsafe character %q", c)
        }
    }
    return nil
}
```

Reasonable per the comment — `validateControlKey` is for tmux key names like `"Space"` or even a literal `" "`. But:

- If a caller mistakenly uses `validateControlTarget` on a key, legitimate `" "` (Space) is rejected.
- If a caller mistakenly uses `validateControlKey` on a target, target with space character (e.g. `"lazyclaude:my window"`) slips through and causes a tmux parse error downstream.
- The naming is type-safety-by-convention only.

**Deepening priority:** P2 — review call sites for correct validator usage.

## Hot-spot deepening recommendation

For Phase B convergence, allocate at least 2 rounds to:

1. **`internal/gui/`** — popups, fullscreen, scroll, layout, notify_loop. Largest LOC by far (18k); highest density of "subtle" patterns. Specifically: popup stack/focus state machine, render cache invalidation, key dispatch ordering, fullscreen forwarder batching.
2. **`internal/daemon/remote_provider.go`** — SSE event handling, mirror-window remap (Bug 4/5), session cache coherence (addToCache, removeFromCache), and the buildTmuxAttachCommand SSH-quoting question.
3. **`internal/daemon/server.go`** — /session/* dispatch logic, /msg/* parallels to server/handler_msg.go, /profiles error format vs documented contract, shutdown ordering.
4. **`internal/session/manager.go`** — Create dispatching by session type, Sync's syncFails counter logic, role/worktree integration points.
5. **`internal/core/tmux/control.go`** — exec adapter, quoting, FIFO query routing, control-mode line protocol.

Lower priority for deepening (these are simpler than they look but not the gene material):

- `cmd/lazyclaude/profile.go` — straightforward CLI.
- `internal/plugin/` — wraps `claude plugins` CLI; thin shim.
- `internal/gui/chooser/` — chooser widget; small and well-bounded.
- `internal/gui/presentation/` — pure formatting, no async state.

PMW (PM/Worker subsystem) seeds are deliberately omitted from deepening as scope-outside per orienting prompt.

## State Checkpoint

```yaml
pass: 6
status: complete
timestamp: 2026-05-11T19:00:00Z
seeds_identified: 15
high_priority_seeds: [1, 2, 3]
p0_seeds: [2, 3]
p1_seeds: [1, 4, 5, 9, 10, 12, 14]
p2_seeds: [6, 7, 8, 11, 13, 15]
next_pass: B-deepening
```
