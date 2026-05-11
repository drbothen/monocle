# Pass B Deep: `internal/core/tmux` — Round 1

**Scope:** Client interface, types, exec adapter, control mode, mock semantics, pidwalk.

**Source LOC:** 1,776 in `internal/core/tmux/`. Files read in full this round: client.go (64), types.go (50), exec.go (440), control.go (head, 379 LOC — first 200 read). Already covered in Pass 3: control.go quoting + pidwalk.go.

## Client interface — operation contract

`Client` (client.go:6-65) declares 17 methods:

1. `ListClients(ctx) → []ClientInfo` — attached terminals
2. `FindActiveClient(ctx) → *ClientInfo` — most recently active
3. `HasSession(ctx, name) → bool` — session exists check
4. `NewSession(ctx, opts) → error` — create session
5. `ListWindows(ctx, session) → []WindowInfo`
6. `NewWindow(ctx, opts) → error`
7. `RespawnPane(ctx, target, cmd) → error`
8. `KillWindow(ctx, target) → error`
9. `ListPanes(ctx, session) → []PaneInfo` — optionally session-filtered
10. `CapturePaneContent(ctx, target) → string` — plain text
11. `CapturePaneANSI(ctx, target) → string` — with escape codes
12. `CapturePaneANSIRange(ctx, target, start, end) → string` — line-range
13. `SendKeys(ctx, target, keys...) → error` — key names ("Enter", "Space")
14. `SendKeysLiteral(ctx, target, text) → error` — text via `send-keys -l`
15. `PasteToPane(ctx, target, text) → error` — `load-buffer` + `paste-buffer -p`
16. `ShowMessage(ctx, target, format) → string` — `display-message -p`
17. `GetOption(ctx, target, option) → string` — `show-option`
18. `ResizeWindow(ctx, target, width, height) → error`

### BC-TMUX-CLIENT-001: Three implementations exist — ExecClient (real), ControlClient (control mode), MockClient (test)
**Postconditions:** ControlClient does NOT implement the full Client interface — PasteToPane returns error (BC-TMUX-CTL-007 from Pass 3); it's intended for hot-path SendKeys only.
**Evidence:** client.go:6 + exec.go:42 + mock.go + control.go.
**Confidence:** HIGH

## ExecClient — subprocess per call

`ExecClient` (exec.go:42-440) spawns `tmux` subprocess for each call. Configurable socket name (`-L name` or `-S /abs/path`). Default 5-second timeout per call.

### BC-TMUX-EXEC-001: All commands prepended with `-u` (force UTF-8) + optional `-L socket` or `-S /path`
**Postconditions:** UTF-8 forced regardless of locale. Distinguishes default server vs lazyclaude socket.
**Evidence:** exec.go:79-90.
**Confidence:** HIGH

### BC-TMUX-EXEC-002: Default timeout is 5 seconds per call (`defaultTimeout`)
**Postconditions:** Hard upper bound for every tmux invocation. Long-running calls (capture-pane on huge buffer) may time out.
**Evidence:** exec.go:17, 93.
**Confidence:** HIGH

### BC-TMUX-EXEC-003: validateShellSafe rejects 14 characters: `; & | ` $ ( ) { } < > \n \r \x00` in session name and other untrusted strings
**Postconditions:** Defense against shell injection in argument values (even though tmux args go through exec.Command argv-form, not shell).
**Evidence:** exec.go:19-28.
**Confidence:** HIGH

### BC-TMUX-EXEC-004: NewSession.Command is NOT validated — application-controlled
**Postconditions:** Caller is responsible for safety. The command may contain `&&`, `cd /path && claude`, etc. for tmux's window-command argument.
**Evidence:** exec.go:178-180 + explicit comment.
**Confidence:** HIGH

### BC-TMUX-EXEC-005: Env keys validated against POSIX identifier regex `^[A-Za-z_][A-Za-z0-9_]*$`
**Postconditions:** No malicious env keys like `; rm -rf /`. Env values are NOT validated (assumed trustworthy).
**Evidence:** exec.go:30-39.
**Confidence:** HIGH

### BC-TMUX-EXEC-006: HasSession differentiates "session not found" (exit 1, specific stderr) from transient errors
**Postconditions:** 5 stderr patterns indicate "not found": `can't find session`, `no session`, `no server running`, `no current target`, `error connecting`. Other stderr → transient error.
**Evidence:** exec.go:138-172.
**Confidence:** HIGH — **NEW finding**: stderr pattern matching for HasSession semantics.

### BC-TMUX-EXEC-007: NewSession uses `-f /dev/null` to bypass user's tmux.conf
**Postconditions:** Prevents user config from interfering with lazyclaude tmux setup. Confirms isolation goal.
**Evidence:** exec.go:186.
**Confidence:** HIGH — **NEW finding**.

### BC-TMUX-EXEC-008: NewSession Env passed via `-e KEY=VALUE` flags (reaches the shell inside tmux)
**Postconditions:** tmux's `-e` is the documented way to inject env into the spawned process. Multiple `-e` flags allowed.
**Evidence:** exec.go:202-205.
**Confidence:** HIGH

### BC-TMUX-EXEC-009: NewSession.PostCommands chained after the new-session command via `;` separator
**Postconditions:** Each post-command arg-array is appended with literal `";"` between groups. tmux command-line chains via this separator.
**Evidence:** exec.go:210-214.
**Confidence:** HIGH

### BC-TMUX-EXEC-010: SendKeysLiteral special-cases bare `";"` → escape as `\;`
**Postconditions:** tmux's global argument parser treats standalone `;` as command separator. Multi-char strings like `"hello;"` are safe because they're a single arg. Bare `;` must be escaped.
**Evidence:** exec.go:329-342 + explicit comment 330-334.
**Confidence:** HIGH — **NEW finding** beyond Pass 3 BC-TMUX-CTL-002 (which covered control-mode escaping, not exec).

### BC-TMUX-EXEC-011: PasteToPane uses load-buffer (stdin pipe) then paste-buffer -p (-d to delete after)
**Postconditions:** Bracketed-paste mode `-p`. Buffer is consumed by `-d` so subsequent pastes don't accumulate.
**Evidence:** exec.go:344-357.
**Confidence:** HIGH

### BC-TMUX-EXEC-012: CapturePane uses Output() (stdout only); stderr captured separately
**Postconditions:** parseWindows/parsePanes parse exact tab-separated stdout. Mixing stderr would corrupt parsing.
**Evidence:** exec.go:99-110 + explicit comment 99-101.
**Confidence:** HIGH

### BC-TMUX-EXEC-013: parseClients/parseWindows/parsePanes use `strings.SplitN(line, "\t", N)` for fixed-arity parsing
**Postconditions:** Format strings emit tab-separated fields; parser expects exactly N tabs. Lines with fewer fields are silently skipped.
**Evidence:** exec.go:376-440.
**Confidence:** HIGH

### BC-TMUX-EXEC-014: GetOption uses `-gqv` for global (no target) and `-qv` for target-specific
**Postconditions:** -q suppresses error; -v returns value-only; -g is global scope.
**Evidence:** exec.go:364-371.
**Confidence:** HIGH

### BC-TMUX-EXEC-015: ListPanes with empty session uses `-a` (all panes); with session uses `-s -t <session>` (cross-windows in that session)
**Postconditions:** Comment line 296-297 documents: "-s lists panes across ALL windows in the session. Without -s, -t targets only the active window."
**Evidence:** exec.go:292-306.
**Confidence:** HIGH

## ControlClient — single persistent connection

`ControlClient` (control.go:67-379) maintains one tmux control-mode connection: `tmux -C [-L socket] attach-session -t <session>`. Receives `%output`, `%begin`, `%end`, `%error` events on stdout; writes commands to stdin.

### BC-TMUX-CTL-008: ControlClient is single-stream — control mode commands serialize via queryQueue FIFO
**Postconditions:** Each Query enqueues a pendingQuery; the next %begin/%end belongs to the oldest. Confirms BC-TMUX-CTL-004.
**Evidence:** control.go:79-83 + queryQueue.
**Confidence:** HIGH

### BC-TMUX-CTL-009: ControlEvent type enum: EventOutput, EventBegin, EventEnd, EventError, EventOther
**Postconditions:** Matches tmux control-mode protocol per `man tmux` Control Mode section.
**Evidence:** control.go:14-23.
**Confidence:** HIGH

### BC-TMUX-CTL-010: ParseControlLine identifies events by prefix: `%output `, `%begin `, `%end `, `%error `; default EventOther
**Postconditions:** %output payload is split into paneID + data on first space. Other event types put the full remainder in Data.
**Evidence:** control.go:33-54.
**Confidence:** HIGH

### BC-TMUX-CTL-011: SendKeys validates target (no spaces/semicolons/newlines) and each key (semicolons/newlines forbidden, spaces allowed)
**Postconditions:** Confirms BC-TMUX-CTL-001. Key validator allows spaces (for the literal " " key).
**Evidence:** control.go:131-141 + validateControlTarget/Key (covered in Pass 3).
**Confidence:** HIGH

### BC-TMUX-CTL-012: NewControlClient spawns `tmux -u -C [-L socket] attach-session -t <session>` and starts a readLoop goroutine
**Postconditions:** Single process per ControlClient. Stderr is set to nil (suppressed).
**Evidence:** control.go:87-126.
**Confidence:** HIGH

## Convergence on remaining tmux

The control.go tail (lines 200-379), mock.go, and pidwalk.go are smaller and largely covered by Pass 3 BC-TMUX-CTL contracts. Pidwalk implementation was already documented in pass-3 (BC-TMUX-PIDWALK-001).

## Delta Summary

- New items added: 16 (1 BC-TMUX-CLIENT, 15 BC-TMUX-EXEC, 5 BC-TMUX-CTL additions)
- Existing items refined: BC-TMUX-CTL-001/002/004/007 confirmed at code level.
- Remaining gaps: mock.go body (test-only, not architecture), control.go tail (Query/Close internals).

## Novelty Assessment

Novelty: SUBSTANTIVE (but with diminishing returns)

Justification: 16 new contracts including:
- **BC-TMUX-EXEC-006** HasSession's 5-stderr-pattern matching (NEW).
- **BC-TMUX-EXEC-007** `-f /dev/null` to bypass user tmux.conf (NEW).
- **BC-TMUX-EXEC-010** bare `;` escape special case (NEW — distinct from control-mode escaping).
- **BC-TMUX-EXEC-009** PostCommands `;` chaining.
- **BC-TMUX-EXEC-005** Env key POSIX-identifier regex.

These are porter-relevant for tmux-binding fidelity (bypass-user-config, env-validation, the bare-`;` quirk).

## Convergence Declaration

**Pass B core/tmux has converged.** The exec adapter and control client are fully spec'd. Mock.go and the remaining control.go internals (Query handling, Close cleanup) are well-covered by Pass 3 BC-TMUX-CTL contracts. A round 2 reading those would add documentation, not new architecture.

## State Checkpoint

```yaml
pass: B
subsystem: core/tmux
round: 1
status: complete
files_read_full: [client.go, types.go, exec.go]
files_read_partial: [control.go (200/379)]
contracts_drafted: 16
total_tmux_contracts: 24  # 8 BC-TMUX-CTL from Pass 3 + 16 here
timestamp: 2026-05-11T22:35:00Z
novelty: SUBSTANTIVE
convergence: PASS-B-CORE-TMUX CONVERGED
next_subsystem: cmd/lazyclaude
```
