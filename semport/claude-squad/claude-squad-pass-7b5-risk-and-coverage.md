# Pass B.5: Risk Register + Test Coverage Notes — claude-squad

## Risk Register

Ordered by severity (P0 = blocking issue if monocle adopts the pattern, P3 = noise).

### P0 — Architectural blockers if adopted as-is

| ID | Risk | Evidence | Why P0 |
|----|------|----------|--------|
| P0.1 | Tmux server crash makes loaded instances unrecoverable | `/session/instance.go:140-143` calls `Start(false)` → `Restore()` (= `tmux attach-session`); failure cascades to `newHome` `os.Exit(1)` (`/app/app.go:137-139`) | If monocle imports this approach, one external-tool crash bricks the entire app. Must design recovery in. |
| P0.2 | No harness abstraction layer | `Instance.Program` is opaque shell text; prompt detection per-program is hardcoded substring matches in `/session/tmux/tmux.go:243-249` | Monocle's multi-harness plane needs structured harness interface. Borrowing claude-squad's model would lock in the no-abstraction model. |

### P1 — Significant operational risks

| ID | Risk | Evidence | Why P1 |
|----|------|----------|--------|
| P1.1 | Daemon polls instance list FROZEN at startup | `/daemon/daemon.go:27-34` loads instances once; loop iterates that fixed slice | New instances post-daemon-start don't get autoyes. |
| P1.2 | `Detach()` panics on PTY close failure | `/session/tmux/tmux.go:391-405` with comment "TODO: control flow is a bit messy here. If there's an error, I'm not sure if we get into a bad state. Needs testing." | User-facing crash on PTY error |
| P1.3 | Stale daemon PID file blocks restart | `/daemon/daemon.go:151-159` doesn't recover from "process not found" — returns wrapped error and the PID file is NOT removed | User must manually `rm ~/.claude-squad/daemon.pid` |
| P1.4 | Broken `config.json` not auto-recovered | `/config/config.go:178-183` logs error and returns DefaultConfig, but doesn't rename/backup the broken file. Next launch reads the same broken file | Silent stuck state |
| P1.5 | `git commit --no-verify` bypasses pre-commit hooks | `/session/git/worktree_git.go:88, 142` | Linters/formatters expected to enforce style won't run; pushed branches may fail CI |
| P1.6 | No agent liveness check | An agent that crashes inside tmux leaves Status as Running indefinitely | UI lies about agent state |
| P1.7 | Daemon spawns even when `app.Run` errored | `main.go:62-67` uses `defer` — if app fails to start, daemon still launches | Hard to debug "where did this daemon come from" |
| P1.8 | Branch deletion is opt-out via global isExistingBranch | `/session/git/worktree_ops.go:114-122` only skips delete if `isExistingBranch=true` — if a user reused an existing-branch instance once, the branch survives Kill; otherwise their branch is gone | Easy to lose work if branch wasn't checked out elsewhere |

### P2 — Code-quality issues

| ID | Issue | Evidence |
|----|-------|----------|
| P2.1 | Two `combineErrors` impls | `/session/instance.go:303-317`, `/session/git/worktree_branch.go:8-21` |
| P2.2 | `saveConfig`/`SaveConfig` aliasing | `/config/config.go:188-210` |
| P2.3 | `q` hardcoded outside keymap | `/app/app.go:597` |
| P2.4 | `KeyQuit` defined but not dispatched via keymap | `/keys/keys.go:15` vs `/app/app.go:600-603` switch |
| P2.5 | `handleKeyPress` is 410 LOC, single function | `/app/app.go:383-795` |
| P2.6 | `go-git` imported but mostly unused | `go.mod:13` — but worktree_ops.go shells out to `git` CLI with "much faster" comments |
| P2.7 | `--no-verify` on commits | `/session/git/worktree_git.go:88, 142` |
| P2.8 | Profile `DefaultProgram` semantic overload | A profile name vs a literal command ambiguity (`/config/config.go:50-59`) |
| P2.9 | `context.Context` not propagated to git/tmux ops | Many long-running ops can't be cancelled |
| P2.10 | `Pause` doesn't save instances immediately | Status mutation in memory; persistence relies on subsequent save trigger |

### P3 — Noise / very minor

| ID | Issue | Evidence |
|----|-------|----------|
| P3.1 | Next.js marketing site in main repo | `/web/` directory |
| P3.2 | Vercel template SVGs left in `/web/public/` | `file.svg`, `globe.svg`, `vercel.svg`, `window.svg` |
| P3.3 | Inconsistent prompt-string handling: claude/aider/gemini detected in `HasUpdated` but trust-prompt logic only handles claude/aider | `/session/tmux/tmux.go:157-180` vs `:235-256` |
| P3.4 | `time.Sleep(100ms)` after SendKeys before TapEnter | `/session/instance.go:585-587` — works but hacky |
| P3.5 | Tmux `claudesquad_` prefix is global — two `cs` instances on same machine would see each other's sessions in `cs reset` | `/session/tmux/tmux.go:60`, `/session/tmux/tmux.go:502-503` |

## Test Coverage Notes

### What IS Tested (6 test files, ~1,257 test LOC)

| File | LOC | Tests | Coverage focus |
|------|-----|-------|----------------|
| `/app/app_test.go` | 466 | TestConfirmationModalStateTransitions, TestConfirmationModalKeyHandling, TestConfirmationMessageFormatting, TestConfirmationFlowSimulation, TestConfirmActionWithDifferentTypes, TestMultipleConfirmationsDontInterfere, TestConfirmationModalVisualAppearance | Confirmation overlay state machine + flow |
| `/ui/terminal_test.go` | 375 | TestTerminalUpdateContent, TestTerminalFallbackStates, TestTerminalSessionCaching, TestTerminalScrolling, TestTerminalCloseForInstance | TerminalPane behavior with mocked tmux |
| `/ui/preview_test.go` | 389 | TestPreviewScrolling, TestPreviewContentWithoutScrolling | PreviewPane with mocked tmux |
| `/config/config_test.go` | 305 | TestGetClaudeCommand, TestDefaultConfig, TestGetConfigDir, TestLoadConfig, TestGetProgram, TestGetProfiles, TestSaveConfig | Config + Profile resolution |
| `/session/tmux/tmux_test.go` | 88 | TestSanitizeName, TestStartTmuxSession | tmux name sanitization + start sequence |
| `/session/git/util_test.go` | 73 | TestSanitizeBranchName | Branch name sanitization (10 test cases) |

### What is NOT Tested

- `app/app.go:1047 LOC` — only the confirmation overlay flow (~400 of those LOC). The remaining ~600 LOC (instance start flow, key dispatch, metadata tick, branch search, prompt handling) is untested at the unit level.
- `/session/instance.go:613 LOC` — entire Instance lifecycle untested directly (only exercised indirectly via ui tests that call instance.Start)
- `/session/git/` — only `sanitizeBranchName` (1 of 7 files). Setup, Cleanup, Push, Diff, IsDirty, FetchBranches, SearchBranches — ALL untested.
- `/daemon/` — entirely untested
- `/main.go` — entirely untested (cobra root, reset/debug/version subcommands)
- `/ui/list.go`, `/ui/menu.go`, `/ui/tabbed_window.go`, `/ui/diff.go`, `/ui/err.go` — entirely untested
- `/ui/overlay/` — only `ConfirmationOverlay` exercised indirectly via `app_test.go`. `TextInputOverlay`, `BranchPicker`, `ProfilePicker`, `TextOverlay`, `PlaceOverlay` — untested.
- `/log/` — `Every` and the loggers untested
- `/keys/` — untested (it's just data, but the dispatch coupling could break)

### Coverage Quality Observations

- Tests that DO exist are well-written (testify assertions, table-driven, subtests, t.Helper, t.TempDir).
- Mock infrastructure (`MockCmdExec`, `MockPtyFactory`) is well-designed; underused.
- **No integration tests** that span TUI → instance → tmux → git → storage end-to-end.
- **No daemon tests** — risky given the daemon's role.
- **No git integration tests** — risky given how much of the code is git CLI shell-out.
- Test count grew with the recent Terminal tab feature (terminal_test.go added in v1.0.16+ era).

### Coverage Risk for Monocle

If monocle wants to borrow patterns from claude-squad, the **patterns themselves are not test-protected**. Adopting an untested pattern means re-implementing AND re-testing it. The git worktree management pattern, in particular, deserves comprehensive tests because it's the data-loss-risk area.

## State Checkpoint

```yaml
pass: 7.5 (B.5)
status: complete
risks_identified: 23 (2 P0 + 8 P1 + 10 P2 + 5 P3)
test_files: 6
test_loc: ~1257
untested_packages: 7 of 11
timestamp: 2026-05-11T19:55:00Z
```
