---
document_type: gene-source-disposition
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
gene_source: claude-squad
disposition_pass: v2 (D-236 control-center pivot)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md §5, embedded-pty-evaluation.md §7.4
---

# Gene-Source Disposition v2: claude-squad (Control-Center Lens)

## Vision Lens Applied

monocle v1 (re-baselined): full TUI control center — launch + manage + observe + tune + control;
many sessions, many projects; never leave the TUI. Daemon owns PTYs; TUI is a client.
Session persistence across restart is a hard v1 requirement.

## Original Disposition (to be selectively reversed)

The v1.1.2 vision-synthesis §Explicit Non-Goals stated:
- "Does NOT include PM/Worker multi-agent orchestration — any-context/lazyclaude's PM/Worker
  subsystem is explicitly out of scope."

The claude-squad Pass-8 synthesis recommended:
- Adopt (A.1-A.8): worktree-per-task isolation, profile selector UX, snapshot-fork concurrency,
  debounced versioned filter, executor seam, log.Every, config+state split, help-screen bitmask.
- Skip (S.1-S.9): polling daemon, hardcoded prompt strings, tmux-as-primitive, no-harness-abstraction,
  no-inter-agent-IPC, no-liveness-check, --no-verify, mixed-ecosystem repo, Detach panic.

The original disposition did NOT reject A.1-A.8 — these were already ADOPT. What the pivot
reverses is the previous framing that the LAUNCH/MANAGE substrate itself was out of scope.
claude-squad is now the primary prior-art reference for the session launch + manage lifecycle.

## Disposition by Capability Area

### 1. Session Launch Mechanism (tmux session spawn + program invocation)

**Original verdict:** "S.3 Tmux as multiplexer primitive — Skip" — the original vision didn't need
to launch sessions, so the entire launch-and-own pattern was irrelevant.

**REVERSED by pivot.** monocle must now spawn and own sessions.

**New verdict: MODEL** — study the pattern; build native (daemon-owned PTY, not tmux).

Reasoning:
- claude-squad's core launch primitive is `tmux new-session + tmux send-keys` to start an agent
  in a dedicated tmux session. This is the BEHAVIOR to replicate, not the tmux mechanism.
- embedded-pty-evaluation.md §§3,4 already decided: use portable-pty (not tmux) as the spawn
  mechanism. The daemon owns PTYs directly. tmux dependency is rejected as primary approach.
- What monocle inherits from claude-squad's launch pattern: (a) dedicated subprocess per session,
  (b) cwd = git worktree (see §5 below), (c) env injection at spawn time (hook config injection),
  (d) program = harness binary (claude, codemachine, etc.) from Profile/EngineModule.
- The mapping is: claude-squad's `tmux new-session -s <name> -c <worktree_path>` →
  monocle's `portable-pty openpty() + CommandBuilder.cwd(worktree).envs(hook_config).spawn(claude_binary)`.

### 2. Instance Lifecycle State Machine (Running/Loading/Ready/Paused/Killed)

**Original verdict:** Not directly addressed — observe-only meant monocle received lifecycle
events passively via hooks.

**New verdict: MODEL** — the Instance state machine is the reference for monocle's session
lifecycle design.

claude-squad's Instance states:
```
Ready → Loading → Running → Paused → Running → ... → Killed
```

monocle's equivalent (mapped to the new control-center model):
```
Created → Launching → Running → Detached → Running → ... → Killed
```

Differences from claude-squad:
- monocle adds `Detached` (TUI disconnected but daemon continues supervising the PTY — the
  persistence model that claude-squad achieves via tmux's inherent detach/reattach). This is the
  primary architectural difference: monocle's daemon owns the PTY so the TUI can detach without
  killing the session.
- monocle uses hook-based liveness (SessionStart/Stop hooks) rather than tmux pane polling for
  state transitions to Running/Ready. This is strictly superior and monocle already has it.
- claude-squad's `Paused` state (commit + worktree removal without killing tmux) maps to
  monocle's `Detached` (TUI disconnects; PTY stays alive in daemon). The semantics differ: in
  claude-squad pause is user-initiated disk-reclaim; in monocle detach is TUI-restart survival.

Adopt the state machine structure; diverge on persistence mechanism.

### 3. Git Worktree Per Session (A.1 — HIGH value)

**Original verdict: ADOPT** — this was already chosen in the original disposition and is unchanged.

**New verdict: ADOPT + EXTEND** — confirm and extend with the new spawn context.

In the control-center model, worktree-per-session becomes more important, not less:
- When monocle SPAWNS a session, it constructs the CommandBuilder with `cwd = worktree_path`.
  This is the natural injection point for the worktree isolation.
- Worktree creation sequence (from claude-squad Pass-7 session-tmux-git):
  1. `sanitize_branch_name(title)` → branch name (regex `[^a-zA-Z0-9\-_] → -`, max 32 chars)
  2. `git worktree add <path>/<sanitized>_<unix_nano> <branch> --no-checkout` or `--checkout`
  3. Capture base commit SHA at creation time (for diff baseline — BC-17)
  4. On kill: `git worktree remove --force <path>` + branch deletion (configurable)
- monocle adopts pattern A.1 as written. The worktree lives under a monocle-managed directory
  (e.g., `~/.local/share/monocle/worktrees/<project>/<session_id>/`).
- Decision: worktree creation is OPTIONAL if the user provides an existing project dir. When the
  user launches a new session from the multi-project picker with no worktree, monocle creates one.
  When the user launches into an existing project root, use that root as cwd directly.

### 4. Session Manager UX / Multi-Session List (TUI Pattern)

**Original verdict:** The session list panel was already ADOPT (and is built). The new question
is whether claude-squad's TUI management patterns extend the built panel.

**New verdict: ENHANCE** — the existing sessions panel extends rather than replaces.

From claude-squad's TUI (pass-7-deep-tui): the key UX patterns are:
- Scrollable list with status indicators (Running/Ready/Loading/Paused)
- Tab-switching within a selected instance (Preview/Diff/Terminal panes)
- One-keystroke kill, attach, push-to-PR
- Profile picker overlay at session creation (A.2 UX pattern)

monocle's control-center adds to the existing sessions panel:
- "Terminal" tab (the embedded PTY pane — the new capability)
- "Create session" action (launch from TUI with profile picker + worktree creation)
- "Kill" action (already partially designed in original vision as `SessionKill` action)
- Project grouping header rows in the session list

The sessions panel becomes monocle's primary "session manager" surface. Its design is already
in monocle-tui; the new capability is the embedded PTY tab and the create/kill actions.

### 5. Profile Selector UX (A.2 — MEDIUM value)

**Original verdict: ADOPT** — already in the original disposition (Ctrl-P profile picker is built
in S-031). Confirmed unchanged.

**New verdict: ADOPT (already built)** — S-031 delivered BC-2.07.004/BC-2.07.005 (profile
picker, sticky-per-project, CCR path). No reversal needed; this was never rejected.

In the control-center model, the profile picker also gates the spawn parameters:
when launching a new session, the profile determines the agent binary, model routing, and any
per-profile env vars. This is exactly the `CommandBuilder.envs()` injection point.

### 6. AutoYes / Polling Daemon (S.1 — Skip)

**Original verdict: Skip** — pane-content scraping is brittle.

**CONFIRMED Skip.** The pivot does not change this verdict.

Reasoning:
- monocle already has the hook-based permission overlay (the "killer scenario") which is
  structurally superior to tmux capture-pane scraping for permission prompts.
- The hook protocol gives monocle structured, schema-validated permission events. There is no
  reason to add a polling fallback.
- claude-squad's `DetectMaxOption` (from any-context) which parses permission dialog options out
  of captured pane content is explicitly marked "Skip — S.2 hardcoded prompt strings" in the
  original disposition. Confirmed.

### 7. Tmux as Multiplexer Primitive (S.3 — originally Skip)

**Original verdict: Skip** (though with caveats about tmux control mode as strategic fallback).

**NEW STATUS: Leave Behind (primary path) / Strategic Fallback (persistence only).**

The embedded-pty-evaluation.md has already decided this authoritatively:
- Primary: native PTY via portable-pty + vt100 + tui-term (daemon-owned).
- Runner-up (strategic fallback only): tmux control mode for session persistence if daemon-owned
  PTY proves insufficient for cross-crash persistence.
- Specifically NOT claude-squad's capture-pane scraping approach.

In the control-center model, monocle's daemon owns the PTYs directly. The TUI attaches as a
streaming client. This gives monocle persistence through TUI restart without tmux dependency.

The one tmux-derived behavior worth modeling (but not via tmux): the `Ctrl-Q` detach key
convention (claude-squad's BC-11). monocle should have an equivalent "detach from session
without killing it" keybinding — but implemented over the daemon IPC, not tmux send-keys.

### 8. No Harness Abstraction / Program String (S.4 — Skip)

**Original verdict: Skip** — CodeMachine's EngineModule is the better template.

**CONFIRMED Skip.** Unchanged. monocle's EngineModule trait already exists and is strictly
superior to claude-squad's opaque `Program string`.

### 9. Snapshot-Fork Concurrency (A.3 — HIGH value)

**Original verdict: ADOPT** — confirmed.

In the control-center model, the snapshot-fork pattern is even more relevant: when the daemon
performs bulk session operations (kill-all, list-all, refresh metadata), it needs the same
"snapshot active sessions → goroutines → WaitGroup.Wait" pattern. In Rust:
`Arc::clone` + `tokio::spawn` + `JoinSet::join_all`.

### 10. Debounced + Versioned Async Filter (A.4 — MEDIUM value)

**Original verdict: ADOPT** — confirmed and unchanged.

The sessions panel filter (which is already built with Nucleo) uses the debounced-versioned
approach for search. The `branchSearchDebounceMsg.version` → match-vs-current-filter pattern
maps to Nucleo's existing debounce infrastructure.

### 11. Executor / PtyFactory Interface Seam (A.5 — MEDIUM value)

**Original verdict: ADOPT** — confirm and extend.

In the control-center model this seam is critical for testing: the daemon's session-manager
component must use a trait-based PTY-spawn seam so that integration tests can inject mock PTYs
(spawn `cat` instead of `claude`). This directly mirrors monocle's existing TestBackend +
DTU-clone discipline. The seam is:
```rust
pub trait PtySpawner: Send + Sync + 'static {
    fn spawn(&self, cmd: &CommandSpec) -> Result<Box<dyn PtyHandle>>;
}
```
`RealPtySpawner` uses `portable-pty`; `MockPtySpawner` uses an in-process echo loop.

### 12. Config + State File Separation (A.7 — LOW value)

**Original verdict: ADOPT** — confirmed unchanged.

`config.json` (user-facing) vs `state.json` (program-managed) separation maps directly to
monocle's `~/.monocle/config.json` (harness profiles, bindings, CCR path) vs session-roster
state. Already implemented in the original Phase-1 design.

### 13. PM/Worker Orchestration (original non-goal)

**Original verdict: LEAVE BEHIND** — confirmed.

The pivot does NOT reverse this. monocle is a human-driven multiplexer (claude-squad's shape),
not an automated PM/Worker orchestrator (any-context's shape). The pivot adds LAUNCH + MANAGE
capability; it does not add LLM-as-coordinator.

### 14. No Inter-Agent IPC (S.5 — originally Skip)

**Original verdict: Skip** — confirmed, with a note.

The control-center model does not add inter-agent IPC. Sessions are independent. The user (via
the TUI) is the coordinator. This is consistent with the human-driven-multiplexer shape.

### 15. Daemon Crash Fragility (Risk P0-1 from claude-squad)

**Original ingest flagged:** "tmux server crash makes loaded instances unrecoverable — os.Exit(1)".

**NEW RELEVANCE (HIGH).** In the control-center model, the daemon crash risk transfers directly
to monocle's PTY-owning daemon. If the daemon crashes, all PTYs die.

Resolution (from embedded-pty-evaluation.md §3.5 + §Recommended):
- Daemon-owned PTYs give detach/reattach for TUI restart (TUI process exits; daemon continues).
- For daemon crash: re-spawn-on-restart policy is the documented fallback. Sessions are lost on
  daemon crash; the user re-launches them. This is acceptable for v1 IF the daemon is stable.
- v1 does NOT need the complexity of PTY state serialization across daemon crash. The daemon is a
  long-lived supervised process; it should not crash. If it does, re-launch is the recovery path.
- Contrast with claude-squad: its fragility is structural (external tmux dependency). monocle's
  fragility is operational (daemon stability). Monocle can improve daemon stability; it cannot
  control tmux.

## Summary Table

| Capability | Original Verdict | New Verdict | Change? |
|-----------|-----------------|-------------|---------|
| Session launch mechanism | N/A (out of scope) | MODEL (portable-pty, not tmux) | REVERSED from irrelevant |
| Instance lifecycle state machine | N/A | MODEL + ADAPT (add Detached state) | NEW |
| Git worktree per session (A.1) | ADOPT | ADOPT + EXTEND (spawn cwd injection) | Extended |
| Session manager UX | ADOPT (observe-only) | ENHANCE (add create/kill/PTY tab) | Extended |
| Profile selector UX (A.2) | ADOPT | ADOPT (already built, S-031) | Confirmed |
| AutoYes / polling daemon (S.1) | Skip | LEAVE BEHIND | Confirmed |
| Tmux multiplexer (S.3) | Skip | Leave Behind (primary) / Strategic Fallback | Confirmed |
| No harness abstraction (S.4) | Skip | LEAVE BEHIND | Confirmed |
| Snapshot-fork concurrency (A.3) | ADOPT | ADOPT + EXTEND (tokio JoinSet) | Extended |
| Debounced versioned filter (A.4) | ADOPT | ADOPT (already in Nucleo filter) | Confirmed |
| Executor/PtyFactory seam (A.5) | ADOPT | ADOPT + EXTEND (PtySpawner trait) | Extended |
| Config+state split (A.7) | ADOPT | ADOPT (already built) | Confirmed |
| PM/Worker orchestration | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| Inter-agent IPC (S.5) | Skip | LEAVE BEHIND | Confirmed |

## Net Assessment

claude-squad is the PRIMARY prior-art reference for the control-center pivot. The original
disposition already captured its TUI patterns and isolation primitives (A.1-A.8). The pivot
adds the launch/lifecycle layer as the new primary inheritance — modeled (not copied) because
monocle uses native PTY instead of tmux.

The single most important gene from claude-squad in the control-center model:
**worktree-per-session isolation as the spawn context** (A.1). This pairs with portable-pty's
`CommandBuilder.cwd()` to give each session an isolated git worktree as its working directory.

The single most important REJECTION confirmed:
**capture-pane scraping (S.1, S.2, S.3 primary path)**. monocle's hook-based permission overlay
is structurally superior and already built.
