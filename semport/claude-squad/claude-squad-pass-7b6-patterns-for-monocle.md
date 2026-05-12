# Pass B.6: Pattern Catalog — What Monocle Should Adopt / Skip

Given the user explicitly excluded PM/Worker from monocle's scope, and asked specifically about patterns worth borrowing.

## Adopt — Patterns Worth Borrowing

### A.1 Worktree-per-task isolation (HIGH value)

**Pattern:** Each agent gets a dedicated `git worktree` under `~/.app/worktrees/<sanitized-name>_<unixnano>/`. The worktree is NOT inside the user's repo — it lives in the app's data dir.

**Why borrow:** Filesystem-level isolation between concurrent agent tasks. Conflict-free parallel work on the same repo. User's working tree is never touched. Cleanup is `git worktree remove`.

**Where:** `/session/git/worktree.go`, `worktree_ops.go`.

**Monocle adoption notes:**
- Use git CLI, not `go-git`, for the actual worktree commands (faster, per the comment at `/session/git/worktree_ops.go:30`)
- Capture base commit SHA at worktree creation for diff baseline (`/session/git/worktree_ops.go:86`)
- Sanitize names with the safe-subset regex (`/session/git/util.go:13-33`)
- Append `_<unixnano>` to ensure uniqueness (`/session/git/worktree.go:67`)

### A.2 Profile selector for multi-harness UX (MEDIUM value)

**Pattern:** Named presets in config (`{name string, program string}`). When user creates a new instance, if multiple profiles exist, show a left/right picker. The selected profile's `Program` becomes the instance's launch command.

**Why borrow:** Minimal-overhead way to let users have multiple agents available. Default profile is sticky but switchable per-instance.

**Where:** `/config/config.go:30-82`, `/ui/overlay/profilePicker.go`.

**Monocle caveat:** claude-squad's Profile is JUST a shell command. For monocle's multi-harness plane, Profile should map to a richer Harness object (capabilities, API). The UX pattern (left/right picker, default-first ordering) is the take-away, not the data model.

### A.3 Snapshot-then-fork pattern for off-loop work (HIGH value)

**Pattern:** In a single-threaded UI, before spawning background goroutines, snapshot the relevant slice on the main thread. Hand the snapshot to goroutines — they touch only their own elements. Join with WaitGroup, send results back as a single message.

**Why borrow:** Avoids data races, keeps the UI thread fast, parallelizes naturally per element.

**Where:** `/app/app.go:915-923` (`snapshotActiveInstances`) + `/app/app.go:930-954` (`tickUpdateMetadataCmd`).

**Sample:**
```go
func snapshotActive(items []*X) []*X {
    var out []*X
    for _, x := range items { if x.Ready() { out = append(out, x) } }
    return out
}

func tickHeavyWork(snapshot []*X) tea.Cmd {
    return func() tea.Msg {
        time.Sleep(interval)
        results := make([]Result, len(snapshot))
        var wg sync.WaitGroup
        for i, x := range snapshot {
            wg.Add(1)
            go func(i int, x *X) {
                defer wg.Done()
                results[i] = x.ExpensiveOp()
            }(i, x)
        }
        wg.Wait()
        return doneMsg{results: results}
    }
}
```

### A.4 Debounced + version-stamped async UI input (MEDIUM-HIGH value)

**Pattern:** When the user types into a filter field, don't search on every keystroke. Schedule a debounced search (150ms timer); when results come back, check that the result's version still matches the picker's current version — discard if stale.

**Why borrow:** Eliminates a class of "old results overwrite new ones" bugs in async UIs.

**Where:** `/app/app.go:854-881`, `/ui/overlay/branchPicker.go:58-100`.

**Sample:** 
```go
type Picker struct { filter string; filterVersion uint64 }

func (p *Picker) OnKeyPress(k tea.KeyMsg) bool {
    p.filter = newFilter(p.filter, k)
    p.filterVersion++
    return true // tell caller filter changed
}

func (p *Picker) SetResults(items []Item, version uint64) {
    if version != p.filterVersion { return /* stale */ }
    p.items = items
}
```

### A.5 Executor / PtyFactory interface seams (MEDIUM value)

**Pattern:** Wrap `*exec.Cmd` in a small interface so tests can inject mocks.

**Where:** `/cmd/cmd.go` (Executor), `/session/tmux/pty.go` (PtyFactory).

**Sample (for monocle's use):**
```go
type Executor interface {
    Run(cmd *exec.Cmd) error
    Output(cmd *exec.Cmd) ([]byte, error)
}

// Tests inject MockCmdExec{RunFunc, OutputFunc}
```

**Caveat:** claude-squad uses Executor only in `tmux/`; the git package shells out directly. If monocle wants comprehensive testability of subprocess code, apply this seam EVERYWHERE.

### A.6 Rate-limited error logging (LOW value but useful)

**Pattern:** A `log.Every(timeout)` wrapper exposes `ShouldLog() bool`. Loops that may repeat the same error wrap their log calls.

**Where:** `/log/log.go:51-76`.

**Sample:**
```go
everyN := log.NewEvery(60 * time.Second)
for {
    if err := op(); err != nil && everyN.ShouldLog() {
        log.ErrorLog.Printf("op failed: %v", err)
    }
}
```

### A.7 Config + State file separation (LOW value, conceptual hygiene)

**Pattern:** Two separate JSON files: `config.json` for user-edited preferences, `state.json` for program-managed state. Different access patterns: config is read mostly, state is read+written often.

**Where:** `/config/config.go` + `/config/state.go`.

**Why borrow:** When users hand-edit config, they don't accidentally trash machine-managed fields. Reduces "did I just lose all my open instances" anxiety.

### A.8 Help-screen "seen once" bitmask (NICHE value)

**Pattern:** Each help screen has a bit; once shown, the bit is set in `state.HelpScreensSeen`. Subsequent runs skip those screens.

**Where:** `/app/help.go:108-162`.

**Why borrow:** Pleasant UX — first-time users see explanations, returning users aren't pestered.

## Skip — Patterns to Leave Behind

### S.1 Polling daemon for autoyes

**Why skip:** Tmux pane scraping + substring matching is fundamentally brittle. Every agent UX change breaks autoyes. Better: design a structured agent protocol where the agent reports its state to the orchestrator.

**Where:** entire `/daemon/` package.

### S.2 Hardcoded per-program prompt strings

**Why skip:** `/session/tmux/tmux.go:243-249` and `:157-180` are switch statements over claude/aider/gemini hardcoded strings. Adding a new agent requires Go code changes. This is the antithesis of a multi-harness abstraction.

### S.3 Tmux as the multiplexer primitive

**Why skip — partially:** Tmux is a great DEVELOPMENT tool but a heavy external dependency. Pros: free terminal multiplexing, scrollback, attach/detach. Cons: external process dependency, IPC must go through tmux ($PROBLEM whenever you want to do anything non-trivial), platform fragility on Windows.

**Alternatives to consider:** Native Go PTY with a TUI multiplexer model (bubbletea + own scrollback management), structured agent stdin/stdout protocol, OR keep tmux but contain it in a single subsystem with a small interface.

### S.4 No harness abstraction (Program string only)

**Why skip:** This is the central reason claude-squad is "just a multiplexer". CodeMachine's EngineModule is the better template.

### S.5 No inter-agent communication

**Why skip:** ...is to be confirmed. If monocle wants multi-agent collaboration (even without PM/Worker semantics — e.g., one agent reviewing another), this needs to be designed. claude-squad gives no patterns here.

### S.6 No agent liveness check

**Why skip:** An agent that crashes inside tmux is invisible to claude-squad. Monocle should design heartbeats or expected-output checks.

### S.7 `git commit --no-verify` always

**Why skip:** Always bypassing pre-commit hooks is a foot-gun. Monocle should make this configurable (probably default to RUNNING hooks, with an opt-out for the agent-driven case).

### S.8 The Next.js website inside the Go repo

**Why skip:** Mixes ecosystems. Cosmetic but confusing. If monocle wants a website, separate repo.

### S.9 `Detach` panics on failure

**Why skip:** Just use error returns. If the detach truly cannot recover, log + signal handler should handle terminal restoration.

## Compare to CodeMachine EngineModule (where applicable)

| Concern | claude-squad | CodeMachine EngineModule | Better for monocle |
|---------|--------------|--------------------------|---------------------|
| Harness abstraction | None — opaque shell command | Structured interface with capabilities | CodeMachine's model |
| Multi-harness selector | Profile picker UX | (Inherent — pick an EngineModule impl) | Borrow claude-squad's UX, use CodeMachine's data model |
| Output interpretation | Substring matching in tmux output | API-level type-safe responses | CodeMachine |
| Workspace isolation | Git worktree per instance | (Varies) | Borrow claude-squad's worktree pattern |
| State persistence | JSON file with raw blob | (Varies) | Borrow split (config.json + state.json) |
| Agent supervision | Poll daemon + tmux | (Varies — process or API) | Neither pure; design heartbeats |

## Recommended Adoption Summary

| Adopt | Skip | Combine with |
|-------|------|--------------|
| A.1 Worktree-per-task | S.1 Polling daemon | CodeMachine harness for the agent layer |
| A.3 Snapshot-fork | S.2 Hardcoded prompts | Bubbletea (yes — it's good) |
| A.4 Debounced versioned filter | S.4 No harness abstraction | Lipgloss styling (yes) |
| A.5 Executor seam (everywhere, not just tmux) | S.3 Tmux-as-primitive (with caveats) | Real agent API, not pane scraping |
| A.7 Config+State separation | S.5 No inter-agent IPC | Real heartbeats not pane hashing |
| A.8 Help bitmask | S.7 git --no-verify always | |
| A.2 Profile selector UX | S.9 panic-on-detach | |
| A.6 log.Every | S.8 mixed-ecosystem repo | |

## State Checkpoint

```yaml
pass: 7.6 (B.6)
status: complete
adopt_patterns: 8
skip_patterns: 9
timestamp: 2026-05-11T19:55:00Z
```
