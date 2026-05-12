# Pass 6: Broad-Sweep Synthesis — claude-squad

## Executive Summary

claude-squad is a 8.7k-LOC Go TUI (BubbleTea + Lipgloss + Cobra) that supervises N concurrent local AI coding agents (default Claude Code; supports codex/gemini/aider/anything via shell command). Each "instance" is the tuple `(title, tmux session, git worktree, branch, program command, autoyes flag)`. The orchestration model is **not** a PM-and-workers semantic: there's no coordinator agent and no inter-agent communication. It's a **supervisor multiplexer** — the user is the coordinator, the agents are isolated workers, and claude-squad provides terminal-multiplexed UI + workspace isolation.

The whole product is a fancy TUI for `tmux new-session + git worktree add` repeated N times, with a smart polling daemon for "yolo mode" (auto-press-Enter on prompts).

## Top 5 Findings

1. **Multi-harness IS supported, but in a thin way.** Profiles (`config.Profile{Name, Program}`) let users define multiple named shell commands (claude, codex, aider, gemini). Each instance can use a different profile. But there is NO harness abstraction: claude-squad doesn't speak the agents' APIs, doesn't know their commands, doesn't normalize their outputs. It just runs them and watches the tmux pane for prompt strings.
2. **Trust prompts are pattern-matched, per-program, with hardcoded strings.** This is the totality of "agent integration": for claude, watch for `"Do you trust the files in this folder?"` and tap Enter. For aider, watch for `"(Y)es/(N)o/(D)on't ask again"`. For gemini, `"Yes, allow once"`. Adding a new agent harness = adding a string to a switch.
3. **The web/ folder is a Next.js marketing site for `smtg-ai.github.io/claude-squad`.** It is NOT an admin UI, NOT an API, NOT a server. It's deployed-static React. Has no connection to the Go binary.
4. **The daemon model is the simplest possible:** a self-fork via `exec.Command("cs --daemon")`, PID file at `~/.claude-squad/daemon.pid`, polling every 1s for instances needing Enter-tapping. No IPC protocol. The "daemon" doesn't communicate with the running TUI — it operates independently on shared filesystem state (state.json + tmux server).
5. **Worktree-per-agent + tmux-per-agent is the entire isolation primitive.** It's tmux + git worktrees + bubbletea. Crucially, the worktrees are NOT under the user's repo — they're under `~/.claude-squad/worktrees/<branch>_<nanos>/` to avoid polluting the user's tree.

## Direct Answers to Special-Interest Questions

### 1. Orchestration model — does it differ from PM/Worker?

**Yes, fundamentally.** This is NOT a PM/Worker pattern. There is no PM. There is no inter-agent message passing. Each agent runs independently. The user is the orchestrator.

Comparison table:

| Dimension | any-context PM/Worker | claude-squad |
|-----------|----------------------|--------------|
| Coordinator | PM agent (an LLM persona) | Human user (TUI) |
| Worker abstraction | Worker session orchestrated by PM via `/msg/*` API | Instance = tmux session running an agent |
| Inter-agent messaging | Yes (`/msg/*` IPC, PM sends to workers) | None — agents are completely isolated |
| Workspace isolation | Worktree-per-worker | Worktree-per-instance |
| PR review feedback loop | PM reviews PRs, sends feedback to workers | None — user reviews + uses `gh repo sync` to push |
| LLM-as-router | PM uses LLM to assign work | Human chooses which instance to attach to |
| Multi-step plans | PM plans + delegates | User plans manually, types prompts |
| Programmability | Programmatic API + LLM agents | Pure interactive TUI |

**Verdict:** claude-squad's orchestration model is **strictly weaker and simpler** than PM/Worker. It deliberately leaves coordination to the human. The user excluded PM/Worker from monocle's scope; claude-squad is even further from PM/Worker than monocle aims to be.

### 2. Session abstraction

A "session" in claude-squad means an **`Instance`** (preferred term in code), which is:

```
Instance := {
  Title        string,   // primary key, max 32 chars, immutable post-Start
  Path         string,   // host repo path
  Branch       string,   // git branch
  Status       Status,   // Running | Ready | Loading | Paused
  Program      string,   // shell command (the harness)
  AutoYes      bool,
  TmuxSession  → claudesquad_<title>  (one tmux session running Program)
  GitWorktree  → ~/.claude-squad/worktrees/<branch>_<nanos>  (one worktree, one branch)
}
```

Mapping to Claude Code's session model: **no direct mapping**. Claude Code has its own session concept (claude.json, ~/.claude/), but claude-squad doesn't reach into it. From claude-squad's perspective, claude is an opaque shell command. Each Instance starts a fresh claude session inside its tmux session; if you Pause + Resume, the tmux session is detached/reattached (so the claude in-process state survives), but if the tmux server dies, the claude session is gone.

There's no notion of "session continuity across daemon restarts" — only continuity across `cs` TUI restarts (because the tmux server is independent of `cs`).

### 3. UI framework

**`bubbletea` (`github.com/charmbracelet/bubbletea v1.3.4`)** with `lipgloss` for styling and `bubbles` for widgets (textarea, viewport, spinner, key). Classic Elm Architecture (Model → Update → View). The `home` struct is the top-level Model in `/app/app.go:51-104`. Submodels are typically Go structs with `String()` methods (not full sub-Models, just renderers).

Architecture:
- Top-level: `home` (BubbleTea Model)
- Sub-renderers (not Models, just struct.String() for view delegation): `List`, `Menu`, `TabbedWindow`, `ErrBox`
- Inside TabbedWindow: `PreviewPane`, `DiffPane`, `TerminalPane`
- Modal overlays composed via `overlay.PlaceOverlay`: `TextInputOverlay`, `TextOverlay`, `ConfirmationOverlay`

Notable: not all child components are full BubbleTea Models — they share message handling via the parent's `Update`. The textarea + branch/profile pickers expose `HandleKeyPress` methods that return `(shouldClose, branchFilterChanged)` tuples that the parent acts on. Pragmatic but not strictly TEA-pure.

### 4. The web/ package

**Next.js 15.3.2 + React 19** marketing site. It's the `smtg-ai.github.io/claude-squad` landing page (per the link in README). Contains:
- `web/src/app/page.tsx` — homepage with install instructions, demo video, feature list
- `web/src/app/components/CopyButton.tsx`, `ThemeToggle.tsx` — small UI components
- `web/public/*.svg` — placeholder Vercel template SVGs

It is **not** an API gateway. It is **not** a web UI for the Go app. It has zero functional coupling to the Go binary. Build/deploy are separate.

### 5. Daemon model

- **Spawn:** `cs --daemon` self-fork via `exec.Command(execPath, "--daemon")` with `Setsid` (unix) or `DETACHED_PROCESS` (windows). Triggered when the main app exits in autoyes mode.
- **PID file:** `~/.claude-squad/daemon.pid`. Used by `StopDaemon` for clean shutdown.
- **IPC:** **None.** The daemon and TUI don't talk. They share state via the JSON files and the tmux server (which is also outside both processes).
- **Polling loop:** Every `DaemonPollInterval` ms (default 1000), iterate over all loaded instances; for each Running/non-Paused instance, capture pane content, check if it contains a known prompt string (claude/aider/gemini), and tap Enter.
- **Shutdown:** SIGINT/SIGTERM handler saves instances and exits cleanly.

### 6. Configuration

- `~/.claude-squad/config.json` — user-facing config (`DefaultProgram`, `AutoYes`, `DaemonPollInterval`, `BranchPrefix`, `Profiles`).
- `~/.claude-squad/state.json` — program-managed state (`HelpScreensSeen` bitmask, `InstancesData` raw JSON).
- `~/.claude-squad/daemon.pid` — daemon PID.
- `~/.claude-squad/worktrees/` — worktree directory tree.
- `os.TempDir()/claudesquad.log` — log file.
- Per-instance config is **not** a thing — only the global Program + AutoYes can be customized per instance, and that happens via the Profile picker at creation time.

### 7. Multi-harness verdict — yes/no/partial?

**Partial-yes, but very thin.** Multi-harness IS supported in three ways:
1. CLI flag `-p, --program` (the legacy way — set the launch command).
2. Config `default_program` (persistent way).
3. Profiles (`profiles: [{name, program}, ...]`) for switchable presets, with a profile picker UI.

But there is **no harness abstraction**. The Program is opaque shell text. The integration points are:
- Prompt-string detection: hardcoded per program (`tmux.go:243-249`) — claude, aider (prefix match), gemini (prefix match). Adding a new harness requires Go code changes.
- Trust-prompt handling: hardcoded per program (`tmux.go:157-180`).

So the user CAN launch arbitrary commands, but only the four named programs (claude, aider, gemini, codex) get "treated as agents" with autoyes support. Codex isn't even in the prompt-detection list — adding it means writing Go code.

Compared to monocle's likely vision of a `Harness` plane: claude-squad is at the very low end — barely a polymorphic Program string, no API normalization.

## Confidence Assessment

| Area | Confidence | Basis |
|------|------------|-------|
| Architecture | HIGH | All code read, dependency graph clear, no surprises |
| Domain Model | HIGH | Small set of entities, fully traced |
| Behavioral Contracts | HIGH for most, MEDIUM for failure recovery paths | Tests cover confirmation flow, branch sanitization, config; UI flows mostly inferred from code |
| NFRs | HIGH | Comments + commit history confirm performance work; security is documented in code |
| Conventions | HIGH | Code is consistent and idiomatic |
| Orchestration model claim | HIGH | I read every file; there's genuinely no inter-agent IPC |

## Gaps and Risks

1. **Tmux server lifecycle is unmanaged.** If the tmux server dies, all instances are orphaned. Loaded instances try `Restore()` which fails silently if the session doesn't exist.
2. **`Detach` panics on PTY close failure.** Marked TODO. A user with a flaky terminal could crash the app.
3. **Branch prefix collisions across two users running cs on the same machine.** `BranchPrefix = <username>/` but if two `cs` instances run on the same repo, both find tmux sessions with `claudesquad_*` prefix in the reset path.
4. **Profile `DefaultProgram` semantic overload.** A profile name that exactly matches a literal program name leads to ambiguity. Documented but fragile.
5. **No way to roll back a bad commit during Pause.** Commits are made with `--no-verify`, so pre-commit hooks (like formatters) are bypassed.
6. **`tickUpdateMetadataCmd` polls forever.** No backoff if an instance keeps erroring on diff.
7. **No graceful daemon supervision.** Daemon dies → silently. No restart. No health check.
8. **Branch search list is unpaginated; just first 50.** No way to find branch 51+.
9. **The Web folder pollutes the repo** with Next.js boilerplate, unrelated to the Go binary. Could confuse contributors.

## Inconsistencies Found

- Two `combineErrors` impls (instance.go, worktree_branch.go) — should be in `log` or a shared util.
- `saveConfig` (private) + `SaveConfig` (public wrapper) — pointless aliasing.
- `q` is hardcoded in `handleKeyPress` (`/app/app.go:597`), bypassing the `GlobalKeyStringsMap`. `KeyQuit` exists in the keymap but isn't wired to anything.
- `programFlag` precedence vs profile picker: flag overrides config, but profile picker overrides instance.Program at creation time. The interaction isn't documented.

## Architecture Recommendations for Monocle

### Patterns worth adopting

1. **Worktree-per-task isolation.** This is genuinely good. Worktrees under `~/.claude-squad/worktrees/<branch>_<unixnano>` keep the user's repo clean. Monocle should consider this for its task isolation primitive.
2. **`Executor` interface for shell command mocking.** Simple, idiomatic, makes tests possible. (`/cmd/cmd.go`.)
3. **Snapshot pattern for off-loop work.** `snapshotActiveInstances()` on the main thread before spawning background goroutines — clean concurrency pattern.
4. **Per-step debounce + version-stamping for async UI work.** The branch search debounce (`branchSearchDebounceMsg`/`branchSearchResultMsg` with version) is a textbook pattern for stale-result rejection.
5. **`log.Every` rate-limiting wrapper.** Useful in loops where the same error keeps recurring.
6. **`PtyFactory` interface for testability of PTY-driven code.** If monocle does PTY anywhere, this seam is essential.
7. **Profile concept** (`{Name, Program}`) — minimal viable harness selector. Could grow into something richer.
8. **State persistence to `~/.app/state.json` (separate from config.json).** Distinguishing user-facing config from program-managed state is good hygiene.

### Patterns to leave behind

1. **Polling daemon for "autoyes".** This is a hack — pattern-matching prompt strings is brittle. Monocle should design a richer agent API (or rely on the agent's own headless mode).
2. **Hardcoded prompt-string detection per program.** Not extensible without code changes.
3. **Tmux as the multiplexer primitive.** It works but pushes complexity onto external dependencies + makes IPC awkward. A native Go PTY model or a structured agent-API model is better long-term.
4. **No inter-agent communication.** This is fine for claude-squad's scope but is a non-starter for a PM/Worker-style orchestrator. Since the user excluded PM/Worker, this is moot — but it confirms claude-squad is a UI multiplexer, not an orchestrator.
5. **The Next.js website inside the repo.** Should be a separate repo.

### Specifically for the multi-harness plane

claude-squad's harness model is **just a shell command**. Compared to CodeMachine's `EngineModule` (which exposes structured methods like `Generate`, `ApplyDiff`, etc.), claude-squad has nothing comparable. **CodeMachine's EngineModule is the better starting point.** Claude-squad's only contribution is the simple `Profile` struct (`{Name, Program}`) for letting users pick at launch time.

### Concrete recommendation

Treat claude-squad as **prior art on the "TUI for managing N concurrent agents" UX**, NOT on the orchestration model. Borrow: worktree-per-task isolation pattern, the Executor mockable seam, branch picker debounce pattern, profile selector UX. Leave behind: the polling daemon, the hardcoded prompt strings, tmux as the multiplexer, the absence of any structured harness API.

## State Checkpoint

```yaml
pass: 6
status: complete
timestamp: 2026-05-11T19:55:00Z
next_pass: 7 (deepening)
```
