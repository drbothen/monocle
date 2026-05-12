# Pass 5: Conventions and Patterns — claude-squad

## Naming Conventions

- **Go-idiomatic exported names:** PascalCase exports, camelCase internals.
- **Methods on receiver types:** `(t *TmuxSession) Method()`, `(i *Instance) Method()`, `(g *GitWorktree) Method()` — never standalone functions where a method makes sense.
- **Test files:** mirror source paths, `_test.go` suffix. TestFunctionName camel-cased.
- **Type prefix where useful:** `Key*` for KeyName enum values, `State*` for state enum, `helpType*` for helpText impls, `tea.KeyMsg` etc.
- **Package names:** singular (`session`, `git`, `tmux`, `daemon`, `config`, `app`, `ui`, `overlay`, `keys`, `log`, `cmd`).
- **Constant naming:** UPPER_SNAKE not used; PascalCase for exported (`GlobalInstanceLimit`, `TmuxPrefix`, `MaxBranchSearchResults`), lowerCamel for unexported (`branchSearchDebounce`).

## Module Organization

- **Feature-folder-ish but light:** each subdomain has its own package with a clear single concern.
- **No barrel exports** (Go doesn't use them; nothing simulates them).
- **`cmd/` is small and unusual:** Just an `Executor` interface plus `MakeExecutor` factory. Used to mock `exec.Cmd` in tests.
- **`cmd/cmd_test/` separate test-helper package:** Holds `MockCmdExec` shared across tmux/git tests. (Only 18 LOC.)
- **One file per top-level concept:** `instance.go`, `storage.go`, `worktree.go`, `worktree_ops.go`, `worktree_git.go`, etc. Files are big but legible.
- **Build-tag splits:** `_unix.go` / `_windows.go` for platform-specific bits — clean Go idiom.

## Error Handling Patterns

- **`fmt.Errorf` with `%w`** for wrapping nearly always (e.g., `/session/instance.go:240`). Consistent across the codebase.
- **`combineErrors` helper** on both `Instance` (`/session/instance.go:303`) and `GitWorktree` (`/session/git/worktree_branch.go:8`). Same pattern, separate impls.
- **Errors bubble up to BubbleTea via `handleError`** which sets the error box and schedules its clearing after 3s.
- **Best-effort cleanup pattern:** in destructive paths, errors are appended to a slice and combined, NOT short-circuited. Allows partial cleanup to proceed.
- **Setup error pattern:** Deferred `if setupErr != nil { Kill() }` cleanup in `Instance.Start` (`/session/instance.go:237-245`) — combines `defer` with an in-scope error variable.
- **`log.ErrorLog.Print(err)` before return** is common (e.g., `/session/instance.go:425, 432, 449`).

## Test Patterns

- **`TestMain` for log init:** Both `app/app_test.go` and `config/config_test.go` use `TestMain` to call `log.Initialize(false)` before tests. Required because most code paths assume the global loggers exist.
- **`testify/assert` + `testify/require`** used consistently.
- **Table-driven tests:** Used in `TestSanitizeBranchName`, `TestConfirmationModalKeyHandling`, `TestGetProgram`, etc.
- **Subtests with `t.Run`:** Used everywhere, e.g., `TestGetClaudeCommand`'s sub-scenarios.
- **TempDir + HOME override pattern:** Several tests override `HOME` to a `t.TempDir()` so config can be written without touching real user files.
- **Mocks via interfaces:** `MockPtyFactory` (`/session/tmux/tmux_test.go:18-42`), `MockCmdExec` (in `cmd_test`) — injected through constructors.
- **Test coverage is low:** Only 6 `_test.go` files across 44 source files. Tests focus on: confirmation overlay state machine, config loading + profiles, branch sanitization, tmux session start.
- **Untested code:** UI rendering, BubbleTea message flow (mostly), daemon, almost all git worktree operations, the entire `instance.go` flow.

## Design Patterns

- **The Elm Architecture (TEA):** BubbleTea framework — `home` is a Model with Init/Update/View. Messages flow via `tea.Cmd`. State transitions are explicit (`state` enum with stateDefault/stateNew/statePrompt/stateHelp/stateConfirm).
- **Command pattern via tea.Cmd:** Side effects (start instance, search branches, schedule ticks) are returned as Cmds, not executed synchronously.
- **Strategy via Executor/PtyFactory interfaces:** Real impls + mock impls for tests.
- **Storage interface decoupling:** `config.InstanceStorage` interface lets `session.Storage` not know about file I/O. Currently one impl (`*State`), but the seam is there.
- **Aggregate root:** `Instance` is the root that composes `TmuxSession` + `GitWorktree`. They're not exposed independently from the session package.
- **Finalizer pattern for list/instance coupling:** `List.AddInstance` returns a `finalize` closure that registers repo metadata. Caller invokes it after Start completes — clean separation of "add to list" vs "finalize".
- **Message-driven state machine for prompts:** `branchSearchDebounceMsg`/`branchSearchResultMsg` with monotonically-increasing version numbers to discard stale results. Solid.

## TUI-Specific Conventions

- **Lipgloss for ALL styling.** Colors, padding, borders, alignment.
- **Adaptive colors (`AdaptiveColor{Light, Dark}`)** used throughout for theme-aware rendering.
- **`runewidth.StringWidth`** for terminal-width math — handles East Asian wide chars.
- **`muesli/ansi.PrintableRuneWidth`** for ansi-aware widths in the overlay.
- **Hard-coded percent-based layout:** List = 30%, tabs = 70% (`/app/app.go:157-159`); preview width = 90% of pane (`/ui/tabbed_window.go:77-79`).
- **Tab-key cycles between Preview/Diff/Terminal tabs.**
- **`shift+up/down` is THE scroll convention** (not pgup/pgdn).

## Anti-Patterns / Code Smells

- **Two `combineErrors` impls** (one on Instance receiver, one on GitWorktree receiver) — should be a shared helper. Minor DRY violation.
- **`go-git` is imported but mostly bypassed:** the dep adds compile-time weight without much runtime use; most paths shell out (justified by "much faster" comments).
- **`saveConfig` and `SaveConfig`:** lowercase private + uppercase exported wrapper. Just makes one of them, or use the uppercase. Minor.
- **`KeyQuit` defined but not in `GlobalKeyStringsMap`** — `q` literal is handled directly in `handleKeyPress` (`/app/app.go:597`). Bypasses the keymap layer.
- **`Detach()` has TODO + panic:** "control flow is a bit messy here. If there's an error, I'm not sure if we get into a bad state. Needs testing." (`/session/tmux/tmux.go:379-380`).
- **`TmuxPrefix = "claudesquad_"`** — global state for namespace, but the reset code searches by this prefix, so a user with multiple installations (different binary names) would collide.
- **`autoYes` defer in main.go:** `defer func(){LaunchDaemon()}()` is fired at function exit. If `app.Run` returns an error, the daemon is still launched. Probably intentional but worth noting.
- **`DefaultProgram` overloaded:** sometimes a profile name, sometimes a literal program string (`/config/config.go:50-59`). The double meaning makes config harder to reason about.
- **No `context.Context` passed deep through session/git/tmux ops.** Most operations are synchronous and uncancellable. The `ctx` on `home` exists but isn't propagated to long-running git operations.
- **`time.Sleep(100 * time.Millisecond)` after `SendKeys`** before tapping Enter (`/session/instance.go:585-587`) — "to prevent carriage return from being interpreted as newline". Hacky but pragmatic.

## Consistent vs. Sporadic Patterns

| Pattern | Consistency |
|---------|-------------|
| `fmt.Errorf` with `%w` | Highly consistent |
| Method receivers on structs | Highly consistent |
| Lipgloss for styling | 100% consistent in UI layer |
| Error wrapping at every boundary | Highly consistent |
| Mock-via-interface | Only tmux and cmd packages; not pervasive |
| Tests | Sparse and selective |
| Context propagation | Inconsistent (mostly missing) |
| Logging | Always file-only via global vars (no zerolog, no slog) |
| Build-tag platform splits | Used for daemon + tmux; nowhere else |

## State Checkpoint

```yaml
pass: 5
status: complete
timestamp: 2026-05-11T19:55:00Z
next_pass: 6
```
