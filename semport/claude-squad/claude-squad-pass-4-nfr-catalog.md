# Pass 4: NFR Catalog — claude-squad

## Performance

| Concern | What's done | Where | Notes |
|---------|-------------|-------|-------|
| Hot path off UI loop | Expensive ops (`tmux capture-pane`, `git diff`) run in `tickUpdateMetadataCmd` in parallel goroutines, joined with WaitGroup | `/app/app.go:930-954` | This is the topic of HEAD commit `a4ab698` ("move expensive operations off UI event loop") |
| Instance start off UI loop | `instance.Start(true)` runs in a tea.Cmd goroutine, returning `instanceStartDoneMsg` | `/app/app.go:435-442`, `:905-910` | Prevents UI freeze on slow worktree setup |
| Branch search debouncing | 150ms debounce + version-stamped results to discard stale (`branchSearchDebounceMsg.version` matched against `BranchFilterVersion`) | `/app/app.go:854-881`, `/ui/overlay/branchPicker.go:97-101` | Plus 5-item windowed render |
| Tmux startup polling | Exponential backoff (5ms → 50ms cap) up to 2s timeout while polling `has-session` | `/session/tmux/tmux.go:112-129` | |
| Pane-content change detection | SHA-256 hash of captured content; emits "updated" only on hash change | `/session/tmux/tmux.go:193-208`, `:251-255` | Avoids redundant downstream work |
| Tmux history scrollback | `history-limit` is bumped from default 2000 → 10000 lines per session | `/session/tmux/tmux.go:133-136` | |
| Mouse scrolling | tmux `mouse on` set per session for scrollback navigation | `/session/tmux/tmux.go:139-142` | |
| `go-git` avoided in hot paths | Most git ops shell out to `git` CLI because go-git's `PlainOpen` is "much faster than" the go-git library code path (per code comment at `/session/git/worktree_ops.go:30, 73`) | `/session/git/worktree_ops.go:31, 74` | Surprising — they're using go-git as a dep but bypassing it |
| Rate-limited logging | `log.Every(60s)` wrapper for periodic errors (e.g., resize loop, diff stats) | `/log/log.go:51-76`, `/daemon/daemon.go:39, 54` | |
| Window resize debouncing | 50ms debounce on SIGWINCH | `/session/tmux/tmux_unix.go:55-63` | |

## Concurrency / Safety

| Concern | What's done | Where |
|---------|-------------|-------|
| Snapshot pattern for active instances | `snapshotActiveInstances()` runs on the main thread to capture the slice before passing to background goroutines, avoiding races with mutations | `/app/app.go:915-923` |
| WaitGroup-joined metadata refresh | All per-instance diff/capture goroutines finish before sending `metadataUpdateDoneMsg` | `/app/app.go:938-952` |
| TerminalPane mutex | `sync.Mutex` wraps all access to `sessions` map and `currentTitle` | `/ui/terminal.go:31-41` |
| Tmux Attach lifecycle | Context-cancellation + WaitGroup for stdin/stdout goroutines | `/session/tmux/tmux.go:258-333` |
| Detach panics on PTY close failure | Treated as unrecoverable; better to panic than corrupt the user's terminal pane | `/session/tmux/tmux.go:391-405` |

## Security

| Concern | What's done | Where |
|---------|-------------|-------|
| Trust prompt handling | `claude`'s "Do you trust the files in this folder?" is auto-dismissed only after the prompt is detected (not pre-emptively) | `/session/tmux/tmux.go:157-180`, `/session/instance.go:333-345` |
| MCP server trust prompt | Similar handling for `"new MCP server"` substring in claude's pane | `/session/tmux/tmux.go:163-170` |
| Aider/Gemini trust handling | Distinct substring detection per program | `/session/tmux/tmux.go:171-178` |
| GitHub PR push requires `gh auth` | `checkGHCLI()` blocks push if `gh auth status` fails | `/session/git/util.go:36-49` |
| `--no-verify` on commits | Both `PushChanges` and `CommitChanges` use `git commit --no-verify` — bypasses pre-commit hooks intentionally | `/session/git/worktree_git.go:88, 142` |
| Branch name sanitization | Branch names are sanitized to a safe character subset (lowercase, alphanumeric + `-_/.`) — protects against shell injection via title | `/session/git/util.go:13-33` |
| Tmux name sanitization | Whitespace stripped, `.` → `_`, prefix added | `/session/tmux/tmux.go:64-68` |
| Daemon detachment | `Setsid` on unix, `DETACHED_PROCESS \| CREATE_NEW_PROCESS_GROUP` on windows | `/daemon/daemon_unix.go`, `/daemon/daemon_windows.go` |
| Log file location | `os.TempDir()/claudesquad.log` — predictable but not user-data | `/log/log.go:17` |
| **No code execution sandboxing** | The configured `Program` runs unrestricted in tmux. Worktree isolation provides FILESYSTEM-level safety (changes can't directly leak to other branches) but not process-level safety | n/a |

## Observability

| Concern | What's done | Where |
|---------|-------------|-------|
| Logging | Three levels (Info, Warning, Error) to file only | `/log/log.go` |
| Daemon log prefix | `[DAEMON]` prefix on all log lines from daemon | `/log/log.go:34-39` |
| Source location in logs | `log.Lshortfile` flag enabled | `/log/log.go:33` |
| End-of-run notice | `Close()` prints "wrote logs to <path>" so the user can find logs | `/log/log.go:48` |
| `cs debug` command | Prints config file path + JSON contents | `/main.go:115-134` |
| Metrics | None — no Prometheus, no OpenTelemetry, no spans | n/a |
| Tracing | None | n/a |
| Health checks | None | n/a |

## Reliability

| Concern | What's done | Where |
|---------|-------------|-------|
| Error cleanup on Start failure | Deferred Kill cleans up tmux + worktree if start fails | `/session/instance.go:237-244` |
| Cleanup error combining | `combineErrors` builds multi-error messages without losing individual errors | `/session/instance.go:303-317`, `/session/git/worktree_branch.go:8-21` |
| Best-effort cleanups | Reset and kill paths log errors but continue cleaning other resources | `/session/instance.go:284-298`, `/session/git/worktree_ops.go:100-134` |
| Graceful daemon shutdown | SIGINT/SIGTERM trapped, instances saved before exit | `/daemon/daemon.go:75-87` |
| Restore-fallback in Resume | If `Restore()` fails on a Paused instance, fall back to `Start()` | `/session/instance.go:495-509` |
| Save-after-state-change | Storage write happens after every successful start/kill/finalize | `/app/app.go:224, 310, 343` |
| Reset path | `cs reset` wipes all instances, all tmux sessions, all worktrees, kills daemon | `/main.go:78-113` |

## Scalability

| Concern | What's done | Where |
|---------|-------------|-------|
| Hard instance cap | 10 concurrent instances | `/app/app.go:24` |
| Branch search cap | 50 results max | `/session/git/worktree_git.go:11` |
| Single-process | Not horizontally scalable — that's not the goal | n/a |
| Tmux as the parallelism primitive | Each agent runs in its own tmux session under one tmux server | n/a |

## Configuration

- Two files, both in `~/.claude-squad/`: `config.json` (user-facing), `state.json` (program-managed).
- Daemon PID at `~/.claude-squad/daemon.pid`.
- Worktrees under `~/.claude-squad/worktrees/`.
- Logs at `os.TempDir()/claudesquad.log`.
- Defaults are reasonable; missing config silently creates default file.
- `cs debug` shows the active config and path.

## Cross-Platform Support

- Unix (Linux + macOS): Full support with SIGWINCH-based resize, Setsid daemon detach.
- Windows: Supported via build tags. Resize is polled at 250ms. Daemon uses `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`.
- However, tmux is a hard prerequisite — Windows support requires tmux available (e.g., WSL).

## Missing/Notable Absent NFRs

- **No agent harness abstraction.** Each agent is a literal shell command. There's no notion of "the harness API" — claude-squad doesn't know which subcommands claude has, only what its prompt strings are.
- **No rate limiting** at the API/orchestration layer (there's no API layer at all).
- **No retry logic** for any agent operation.
- **No timeout on prompt response.** The daemon polls forever — there's no notion of "agent stuck".
- **No multi-machine support.** All state is host-local. No way to inspect a daemon from a different machine.
- **No HTTP/JSON API.** Pure TUI; no programmatic access aside from CLI flags.

## State Checkpoint

```yaml
pass: 4
status: complete
timestamp: 2026-05-11T19:55:00Z
next_pass: 5
```
