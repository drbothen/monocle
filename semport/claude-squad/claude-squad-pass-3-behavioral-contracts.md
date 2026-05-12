# Pass 3: Behavioral Contracts — claude-squad

Confidence legend: HIGH (test-asserted), MEDIUM (clearly in code with guards), LOW (inferred).

## BC-1: New instance requires git repository, non-empty title, and writable home

**Preconditions:**
- Current working directory is a git repository (verified by `git -C . rev-parse --show-toplevel`)
- Title is non-empty and ≤ 32 runewidth chars
- `~/.claude-squad/` is writable

**Postconditions on success:**
- A tmux session named `claudesquad_<sanitized-title>` exists running `Program` inside the worktree dir
- A git worktree at `~/.claude-squad/worktrees/<sanitized-branch>_<unixnano>` exists, with branch `<BranchPrefix><sanitized-title>` from HEAD
- Instance is persisted to `~/.claude-squad/state.json`
- `instance.Started() == true`, `Status == Running`

**Error cases:**
- Not in git repo → main.go returns `error: claude-squad must be run from within a git repository` (`/main.go:46-48`)
- Empty title at Start → `instance title cannot be empty` (`/session/instance.go:204`)
- Title too long → `title cannot be longer than 32 characters` (`/app/app.go:447`)
- New repo with no commits → `this appears to be a brand new repository: please create an initial commit` (`/session/git/worktree_ops.go:81`)
- tmux session creation timeout (2s) → `timed out waiting for tmux session` (`/session/tmux/tmux.go:121`)
- On failure, Kill is called as cleanup (deferred at `/session/instance.go:237-244`)

**Evidence:**
- `TestSanitizeBranchName` (`/session/git/util_test.go:7`) — proves branch sanitization
- `TestStartTmuxSession` (`/session/tmux/tmux_test.go:52`) — proves tmux start sequence
- `TestSanitizeName` (`/session/tmux/tmux_test.go:44`) — proves tmux name sanitization
- Code reading: `/main.go:46-48`, `/session/instance.go:202-274`

**Confidence: HIGH**

## BC-2: Killing an instance cleans up tmux + worktree + branch atomically (best-effort)

**Preconditions:**
- Instance has been started (`started == true`)
- Branch is not currently checked out in the host repo (`!IsBranchCheckedOut`)
- User has confirmed via confirmation overlay (y key)

**Postconditions:**
- Tmux session is killed
- Worktree is removed
- Branch is deleted (UNLESS `isExistingBranch == true`, in which case branch is preserved)
- Instance is removed from storage

**Error cases:**
- Cleanup errors are accumulated, not fatal (`/session/instance.go:282-301`)
- If branch is checked out: returns `instance %s is currently checked out` (`/app/app.go:692-694`)

**Evidence:**
- Code reading: `/session/instance.go:277-301`, `/app/app.go:680-707`
- `TestConfirmationModalKeyHandling` (`/app/app_test.go:109`) — proves confirmation gating

**Confidence: HIGH**

## BC-3: Pause preserves work by committing locally before removing worktree

**Preconditions:**
- Instance `started == true && Status != Paused`

**Postconditions on success:**
- If worktree was dirty: changes are committed locally (`--no-verify`) with message `[claudesquad] update from '<title>' on <RFC822 ts> (paused)` (`/session/instance.go:428`)
- Tmux session is DETACHED (not closed — still running) via `DetachSafely`
- Worktree directory is removed (`git worktree remove -f`)
- Worktree pruned (`git worktree prune`)
- Branch survives (intentional, `Remove()` doesn't delete branch)
- Status → Paused
- Branch name is copied to clipboard (`clipboard.WriteAll`)

**Error cases:**
- If commit fails: returns early with error (instance NOT marked paused; worktree NOT removed)
- If `DetachSafely` fails: continues anyway (logged)
- If `worktree remove` fails: returns error

**Evidence:**
- Code reading: `/session/instance.go:411-469`

**Confidence: MEDIUM (no direct test, but defensive code)**

## BC-4: Resume requires branch is not checked out elsewhere

**Preconditions:**
- Instance is Paused
- Instance's branch is NOT currently the checked-out branch in the host repo

**Postconditions on success:**
- Worktree is recreated (`git worktree add`)
- Tmux session is restored (if still alive) OR new tmux session is created
- Status → Running

**Error cases:**
- Branch is checked out → `cannot resume: branch is checked out, please switch to a different branch`

**Evidence:**
- Code reading: `/session/instance.go:472-525`

**Confidence: MEDIUM**

## BC-5: Push (`p` key) requires `gh` CLI and gated by confirmation

**Preconditions:**
- `gh` is in PATH and authenticated (`gh auth status` succeeds)
- Instance is not Loading
- User has confirmed (y on confirmation overlay)

**Postconditions on success:**
- All worktree changes are staged + committed with message `[claudesquad] update from '<title>' on <RFC822 ts>` (`/app/app.go:721`)
- `gh repo sync --source -b <branch>` is run (fallback: `git push -u origin <branch>`)
- `gh repo sync -b <branch>` is run again
- Browser opens to the branch URL (`gh browse --branch <branch>`)

**Error cases:**
- `gh` not installed → `GitHub CLI (gh) is not installed`
- `gh` not authenticated → `GitHub CLI is not configured`
- Push failures: bubbled up with combined output

**Evidence:**
- Code reading: `/session/git/worktree_git.go:68-124`, `/session/git/util.go:36-49`

**Confidence: MEDIUM**

## BC-6: AutoYes mode launches daemon at exit; daemon polls + taps Enter on prompts

**Preconditions:**
- AutoYes is set (config or `-y` flag)

**Postconditions on app exit:**
- `cs --daemon` child is spawned, detached via `Setsid` (unix) or `DETACHED_PROCESS` (windows)
- PID written to `~/.claude-squad/daemon.pid`
- Any previously running daemon is killed first (`StopDaemon()` called before `LaunchDaemon()`)

**Daemon behavior:**
- On startup, loads all stored instances and forces `AutoYes = true` on each
- Polls every `DaemonPollInterval` (default 1000ms)
- For each Started + non-Paused instance: if `HasUpdated().hasPrompt == true`, call `TapEnter` then refresh diff stats
- On SIGINT/SIGTERM, saves instances before exit

**Detection of prompt:**
- Claude: pane content contains `"No, and tell Claude what to do differently"` (`/session/tmux/tmux.go:244`)
- Aider: pane content contains `"(Y)es/(N)o/(D)on't ask again"` (`/session/tmux/tmux.go:246`)
- Gemini: pane content contains `"Yes, allow once"` (`/session/tmux/tmux.go:248`)
- **No detection for other programs** — autoyes is a no-op for unknown agents

**Evidence:**
- Code reading: `/daemon/daemon.go`, `/main.go:62-72`, `/session/tmux/tmux.go:235-256`

**Confidence: HIGH (clearly written, with platform-specific tests in daemon_unix/windows)**

## BC-7: Profile selector switches the program at instance creation time

**Preconditions:**
- Multiple profiles defined in config (`HasMultiple() == true`)
- User is in `statePrompt` overlay (Shift+N flow)

**Postconditions:**
- The selected profile's `Program` field replaces `instance.Program`
- The instance is started with that program

**Evidence:**
- Code reading: `/app/app.go:502-511`, `/ui/overlay/profilePicker.go`, `/ui/overlay/textInput.go:249-253`
- `TestGetProgram`, `TestGetProfiles` (`/config/config_test.go:205-269`) — proves profile resolution

**Confidence: HIGH**

## BC-8: Branch picker allows starting on an existing branch (vs creating new from HEAD)

**Preconditions:**
- User is in `statePrompt` overlay
- A non-empty branch was selected in the branch picker (otherwise "New branch (from HEAD)" is the choice)

**Postconditions on existing-branch path:**
- `instance.selectedBranch` is set; `Start(true)` calls `git.NewGitWorktreeFromBranch` instead of `NewGitWorktree`
- The new worktree uses the existing branch (cleanup will NOT delete the branch — `isExistingBranch=true`)

**Branch list behavior:**
- Lists local + remote branches via `git branch -a --sort=-committerdate --format=%(refname:short)` (`/session/git/worktree_git.go:23-26`)
- Up to `MaxBranchSearchResults = 50` (`/session/git/worktree_git.go:11`)
- Filter is case-insensitive substring
- Debounced 150ms in the UI (`/app/app.go:860`)
- "New branch" pseudo-option is hidden if filter exactly matches an existing name (`/ui/overlay/branchPicker.go:106-114`)

**Evidence:**
- Code reading: `/session/git/worktree_git.go:22-53`, `/ui/overlay/branchPicker.go`, `/session/git/worktree.go:93-108`

**Confidence: HIGH**

## BC-9: Title and branch sanitization rules

**For tmux session name (`toClaudeSquadTmuxName`):**
- Strip all whitespace (regex `\s+`)
- Replace `.` with `_`
- Prefix with `claudesquad_`

**For branch name (`sanitizeBranchName`):**
- Lowercase
- Replace ` ` with `-`
- Remove anything not `[a-z0-9\-_/.]`
- Collapse `-+` to `-`
- Trim leading/trailing `-` or `/`

**Test table (`/session/git/util_test.go:7-73`):**

| Input | Expected output |
|-------|-----------------|
| `"feature"` | `"feature"` |
| `"new feature branch"` | `"new-feature-branch"` |
| `"FeAtUrE BrAnCh"` | `"feature-branch"` |
| `"feature!@#$%^&*()"` | `"feature"` |
| `"feature/sub_branch.v1"` | `"feature/sub_branch.v1"` |
| `"feature---branch"` | `"feature-branch"` |
| `"-feature-branch-"` | `"feature-branch"` |
| `"/feature/branch/"` | `"feature/branch"` |
| `""` | `""` |
| `"USER/Feature Branch!@#$%^&*()/v1.0"` | `"user/feature-branch/v1.0"` |

**Confidence: HIGH (test-verified)**

## BC-10: Global instance limit is 10

`const GlobalInstanceLimit = 10` (`/app/app.go:24`). Hard cap enforced before `KeyNew` and `KeyPrompt`. Error: `you can't create more than %d instances`.

**Confidence: HIGH (code-asserted)**

## BC-11: Ctrl-Q is the detach key (not Ctrl-B like raw tmux)

The Attach loop in tmux.go reads stdin and intercepts `0x11` (Ctrl+Q) to call `Detach()`. Any other input is forwarded to the PTY. This is a deliberate choice to avoid users accidentally killing the session via Ctrl-D.

**Evidence:** `/session/tmux/tmux.go:319-328`, README "Detach from session" mapping.

**Confidence: HIGH**

## BC-12: Confirmation modal y/n/esc handling

**Test-asserted (`/app/app_test.go:109-189`):**
- `y` → state returns to default, overlay nil, action executed via `OnConfirm`
- `n` → state returns to default, overlay nil, action NOT executed
- `esc` → same as `n`
- other keys → state stays `stateConfirm`, overlay persists

**Confidence: HIGH**

## BC-13: Help screens are shown once per type (bitmask), except general help (always)

`helpTypeGeneral` is always shown (`alwaysShow = true` at `/app/help.go:132-136`). `helpTypeInstanceStart`, `helpTypeInstanceAttach`, `helpTypeInstanceCheckout` are each gated by bit-1, bit-2, bit-3 of `HelpScreensSeen` uint32.

**Confidence: MEDIUM**

## BC-14: Storage only persists Started() instances

`SaveInstances` filters out non-started instances (`/session/storage.go:60-64`). This means an instance created and immediately killed (before Start succeeded) never hits disk.

**Confidence: HIGH**

## BC-15: Reload-from-disk reattaches PTY to existing tmux session

`FromInstanceData` for a non-paused instance calls `Start(false)` which calls `Restore()` (= `tmux attach-session -t <name>` via new PTY). It does NOT restart the agent. **If the tmux server has been killed since last app run, this silently does nothing useful** — the PTY attach will fail and the instance won't be properly reattached. Currently no graceful handling of this case at session-load time.

**Confidence: MEDIUM (inferred from `/session/instance.go:140-143`)**

## BC-16: Terminal tab spawns a SECOND tmux session per instance

The Terminal tab (`/ui/terminal.go`) opens a separate tmux session named `claudesquad_term_<title>` running the user's `$SHELL` in the worktree directory. This is independent of the agent's tmux session. Sessions are cached in `TerminalPane.sessions` map and cleaned up on `Kill` or `Pause`.

**Confidence: HIGH (code-clear)**

## BC-17: Diff is computed against the worktree's base commit SHA, not current HEAD

`baseCommitSHA` is captured at `setupNewWorktree` time (HEAD at the moment the instance was created) and stored in `GitWorktree`. The diff is `git --no-pager diff <baseCommitSHA>` from the worktree. This means: diff stats represent total changes since the instance was created, regardless of intermediate commits in the host repo.

**Confidence: HIGH (`/session/git/diff.go:25-39`, `/session/git/worktree_ops.go:86`)**

## BC-18: Reset wipes all (instances + tmux + worktrees + daemon)

`cs reset` (`/main.go:78-113`):
1. Loads state, calls `storage.DeleteAllInstances()` (sets `InstancesData = []`)
2. Calls `tmux.CleanupSessions` which kills all `claudesquad_*` tmux sessions
3. Calls `git.CleanupWorktrees` which iterates `~/.claude-squad/worktrees/`, deletes branches for each via `git worktree list --porcelain` parsing, removes directories, prunes
4. Stops daemon if running

This is the panic button.

**Confidence: HIGH**

## Gaps (no behavioral contract derivable)

- Window resize handling on Windows (polling): no test coverage
- Daemon reload after daemon crash: not handled
- Multi-repo scenarios: a single host repo per session is assumed; the code can detect multiple repos in the list (`/ui/list.go:319-335`) but most operations assume current directory is the source repo
- Cancel-during-Start: if the user hits ctrl+c while `instance.Start` is running in a goroutine, what happens? The goroutine continues; the `instanceStartDoneMsg` is processed when it arrives, and the instance is finalized

## State Checkpoint

```yaml
pass: 3
status: complete
contracts_extracted: 18
timestamp: 2026-05-11T19:55:00Z
next_pass: 4
```
