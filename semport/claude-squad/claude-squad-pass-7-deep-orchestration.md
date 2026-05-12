# Pass 7 (Deepening): Orchestration Model — claude-squad

## Scope

Targeted convergence on **orchestration**, the user's primary special-interest topic. Question to answer: *how does claude-squad coordinate multiple Claude sessions, and how does that compare to PM/Worker?*

## What's New vs Broad Sweep

The broad sweep (Pass 6) gave the headline answer ("not a PM/Worker, no inter-agent IPC, user is the orchestrator"). This deepening verifies the claim by tracing every cross-instance message path in the code, and surfaces a few finer-grained points.

## All Cross-Instance Message Paths Traced

I grepped every file for any code that might enable instance-to-instance communication. Result: **no such path exists.**

Every operation on an instance is initiated by one of:

1. **User keypress** routed through `home.handleKeyPress` (`/app/app.go:383-795`). Targets one selected instance.
2. **Periodic tick** from `tickUpdateMetadataCmd` (`/app/app.go:930-954`). Updates metadata for all started+non-paused instances **in parallel**, but each goroutine touches only its own instance.
3. **Daemon poll** (`/daemon/daemon.go:47-72`). Iterates instances sequentially; for each, captures pane content + taps Enter independently.
4. **Storage load** at startup (`session.Storage.LoadInstances`). Constructs instances; no cross-instance state.
5. **Reset path** (`main.go:78-113`). Bulk wipe, no per-instance coordination.

Cross-instance state in code:
- `list.repos map[string]int` — tracks how many instances reference each repo. Used purely to decide whether to display the repo name (when multiple repos are in play). **No coordination effect.**
- `GlobalInstanceLimit = 10` — global cap. Read at create time. **No coordination effect.**

That is the totality of multi-instance state in claude-squad.

## Compared to PM/Worker (Verified Detail)

The user noted that any-context/lazyclaude uses a "PM persona orchestrates worker sessions via /msg/* API, worktree-per-worker, PR review feedback loop". claude-squad does **none** of the PM half:

| PM/Worker requirement | Present in claude-squad? | Evidence |
|-----------------------|--------------------------|----------|
| PM agent (LLM persona that decides who works on what) | No | No persona/role concept in the codebase. `Instance.Program` is opaque. |
| Inter-agent IPC | No | No `/msg/*` API, no shared bus, no channel between instances |
| PM-routed task assignment | No | User picks instance via TUI keys; no assignment logic |
| PR review feedback loop | No (manual) | `KeySubmit` calls `gh repo sync` + opens PR in browser; review is human-driven |
| Worktree-per-worker | YES | This is the one shared pattern |
| Multi-step plan | No | No plan structure exists in code |

So the worktree-per-worker primitive is the ONLY overlap with PM/Worker. Everything else differs.

## What claude-squad Actually Provides

A **TUI for human-driven supervision of N concurrent agents**:

- View a list of agents with status indicators (running, ready, paused) + diff stats.
- Switch between their tmux panes (attach/detach).
- Inspect their work (preview pane + diff pane).
- A separate shell tab inside each worktree (terminal tab, since v1.0.17 — see commit `e69ff9c`).
- Optional autoyes (daemon polls + auto-presses Enter on agent prompts).
- Push branch + open PR with one keystroke (`p` → `gh repo sync` + `gh browse`).

That is the entire orchestration model. The user is the entire control plane.

## A "Multi-Agent" Product That Doesn't Talk Multi-Agent

The README says "manages multiple Claude Code (and other local agents) in separate workspaces". The implementation matches that wording literally: it manages multiple agents in the sense that you can launch several. It does NOT coordinate them.

This is fine for what it is. But it's important to be precise: any architecture document that claims claude-squad is "doing multi-agent orchestration" is misreading the codebase. It's a multi-agent **multiplexer**.

## Relation to "Multi-Harness" Question

The orchestration of "which harness does this instance speak" is also user-driven: at instance-creation time, if profiles are configured, the user picks one in the overlay (`/ui/overlay/profilePicker.go`). The chosen profile's `Program` becomes `instance.Program`. After that, claude-squad's only "harness-awareness" is the prompt-string detection in `tmux.go` lines 243-249, which is a per-program switch on hardcoded strings.

There is no:
- Harness capability negotiation
- Harness version detection
- Harness API translation
- Harness output normalization

**This is the polar opposite of CodeMachine's EngineModule.** EngineModule has a structured interface; claude-squad has none.

## Edge Cases Verified

1. **What if two instances target the same branch?** The branch sanitization makes collision highly unlikely (different titles → different branches), but if a user does try this: `setupNewWorktree` calls `git branch -D <branch>` before adding (`/session/git/worktree_ops.go:74`), so it would silently delete the other instance's branch reference. The previous worktree still exists on disk but is orphaned. **This is a real footgun**, not exercised in tests.
2. **What if an instance crashes inside tmux?** The pane content stops updating but `HasUpdated` doesn't see a state change. The status doesn't get demoted from Running. claude-squad has no liveness check. The instance appears "running" until user inspects.
3. **What if the daemon crashes?** PID file remains; `StopDaemon` next launch fails with "failed to find daemon process" (`/daemon/daemon.go:152`) but actually it succeeds since `os.FindProcess` doesn't verify on unix. The next `proc.Kill()` returns ESRCH but the code treats this as fatal. **Bug:** daemon crash can prevent restart. Not severe but real.

## Delta Summary

- New items added: 0 new entities, 0 new subsystems
- Existing items refined: confirmed that no inter-instance IPC exists (verified by exhaustive code reading); surfaced 3 edge cases (collision, crash, daemon-crash-PID-stale)
- Remaining gaps: none — the orchestration story is small and fully understood

## Novelty Assessment

Novelty: **NITPICK**

Justification: The broad sweep correctly characterized the orchestration model. This pass confirmed by reading every file. The new edge cases are findings, not gaps in the architecture story. Removing this round's content would not change how monocle should spec orchestration relative to claude-squad.

## Convergence Declaration

Orchestration pass has converged — claude-squad has no orchestration model in the PM/Worker sense, and this is verified at the code level.

## State Checkpoint

```yaml
pass: 7
subsystem: orchestration
round: 1
status: complete
novelty: NITPICK
timestamp: 2026-05-11T19:55:00Z
```
