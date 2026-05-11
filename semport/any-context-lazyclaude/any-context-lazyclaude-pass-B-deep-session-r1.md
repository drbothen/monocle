# Pass B Deep: `internal/session` — Round 1

**Scope:** Store + Session + Project model, GC, worktree, role + prompt resolution, gitcmd, launchspec, Manager head.

**Files read in full this round:** store.go (646 LOC), gc.go (89), worktree.go (98), role.go (170), launchspec.go (47), gitcmd.go (105), project.go (35), service.go (32). Partial: manager.go (200/1127).

## Store — projects + sessions on disk

`Store` (store.go:121-647) is the persistence layer. Internally organizes sessions into `[]Project`. Concurrency-safe via `mu sync.RWMutex`. Atomic write via temp-file rename.

### BC-SESSION-STORE-001: stateFile.Version = 2; legacy format ([]Session) resets to empty without migration
**Postconditions:** Users with v1 state.json lose their session list (one-time). Forward-compat: any future version mismatch also resets.
**Evidence:** store.go:112-117, 132-160.
**Confidence:** HIGH

### BC-SESSION-STORE-002: Project.Expanded is `json:"-"` so it's not persisted; defaults to `true` on Load
**Postconditions:** Users always see expanded projects after restart, regardless of pre-shutdown state.
**Evidence:** store.go:151-154; project.go:20.
**Confidence:** HIGH

### BC-SESSION-STORE-003: Save uses MarshalIndent + temp-file + rename for atomicity
**Postconditions:** State dir created with 0o700. JSON pretty-printed. Crash during write leaves either old or new file intact (no partial).
**Evidence:** store.go:164-202.
**Confidence:** HIGH

### BC-SESSION-STORE-004: PM sessions stored as `Project.PM *Session` (pointer); worker sessions in `Project.Sessions []Session`
**Postconditions:** A project can have at most ONE PM. PM is separately tracked from workers.
**Evidence:** project.go:16-17; store.go:268-281.
**Confidence:** HIGH

### BC-SESSION-STORE-005: Add() auto-creates Project if not found; matches by (path, host)
**Postconditions:** Each (projectPath, host) tuple is a unique project. Mixed-host projects unsupported (store.go:530-531 comment).
**Evidence:** store.go:248-283 + projectHost helper 516-542.
**Confidence:** HIGH

### BC-SESSION-STORE-006: Remove() removes the project entirely when both PM is nil AND Sessions is empty
**Evidence:** store.go:286-310 + maybeRemoveProjectLocked 544-552.
**Confidence:** HIGH

### BC-SESSION-STORE-007: All() returns flat slice including PM session per project
**Postconditions:** Iteration is per-project: PM first (if any), then Sessions. Order across projects is store insertion order.
**Evidence:** store.go:228-240.
**Confidence:** HIGH

### BC-SESSION-STORE-008: Projects() returns DEEP COPY — no shared pointers with internals
**Postconditions:** Callers can safely mutate. PM is copied separately via dereference; Sessions slice is copied.
**Evidence:** store.go:206-224.
**Confidence:** HIGH

### BC-SESSION-STORE-009: FindByID/FindByName scan all projects (PM + Sessions); return copies, not pointers
**Postconditions:** Mutations to the returned session do NOT affect the store.
**Evidence:** store.go:313-350.
**Confidence:** HIGH

### BC-SESSION-STORE-010: GenerateName produces "base", "base-2", "base-3", ... until unique
**Postconditions:** No upper limit. nameExistsLocked iterates all projects.
**Evidence:** store.go:442-455.
**Confidence:** HIGH

### BC-SESSION-STORE-011: SyncWithTmux uses windowName → window mapping; remote sessions also check mirror name `rm-<id[:8]>`
**Postconditions:** Sessions with no matching window → StatusOrphan; with window but no pane → StatusDetached; pane dead → StatusDead; pane alive with PID → StatusRunning.
**Evidence:** store.go:580-646.
**Confidence:** HIGH

### BC-SESSION-STORE-012: SyncWithTmux pane-conflict resolution: prefer alive pane over dead pane in same window (remain-on-exit handling)
**Postconditions:** Multiple panes in one window (rare; mostly from `remain-on-exit` keeping dead panes) — alive wins. Both alive: first-seen wins.
**Evidence:** store.go:589-601 + comment 591-597.
**Confidence:** HIGH

### BC-SESSION-STORE-013: WindowName uses "lc-" + ID[:8]; MirrorWindowName uses "rm-" + ID[:8]
**Postconditions:** Distinct prefixes prevent collision between local and remote-mirror windows.
**Evidence:** store.go:62-78.
**Confidence:** HIGH

### BC-SESSION-STORE-014: TmuxTarget resolution order: TmuxWindow (if set) → MirrorWindowName for remote OR WindowName for local → prefix with "lazyclaude:" if no `:`
**Postconditions:** Single function handles local/remote distinction. Returns fully-qualified target.
**Evidence:** store.go:96-109.
**Confidence:** HIGH

## GC — periodic dead-session cleanup

`GC` (gc.go full file) is a goroutine that ticks at the given interval, calls `Sync`, then deletes Dead sessions older than gcGracePeriod.

### BC-SESSION-GC-001: Default tick interval is configurable via NewGC(svc, interval)
**Postconditions:** root.go calls NewGC with 2s (Pass 1 BC-SESSION-013).
**Evidence:** gc.go:19-23.
**Confidence:** HIGH

### BC-SESSION-GC-002: gcGracePeriod = 10 seconds; sessions younger than this are spared
**Postconditions:** Prevents Create → Sync (before tmux is ready) → Orphan → Delete race.
**Evidence:** gc.go:56-58.
**Confidence:** HIGH — **new explicit grace period contract**.

### BC-SESSION-GC-003: ONLY Dead sessions are deleted; Orphan sessions are SPARED
**Postconditions:** Confirms BC-SESSION-013 from Pass 3. The explicit comment: "Orphan means the tmux session was temporarily unreachable... NOT that the window is actually gone. Deleting Orphan sessions was causing state.json wipeout during heavy go test runs."
**Evidence:** gc.go:67-81 + comment 68-72.
**Confidence:** HIGH

### BC-SESSION-GC-004: Sync errors abort the GC cycle (debugLog + return)
**Postconditions:** Transient Sync errors don't trigger deletes.
**Evidence:** gc.go:60-64.
**Confidence:** HIGH

## Manager.Sync — transient-failure tolerance

`Manager.Sync` (manager.go:138-184) has the syncFailCount counter:

- If `tmux.HasSession` returns false, increment `syncFailCount` and return success (no orphan marking).
- If `HasSession` returns true, reset `syncFailCount = 0` and proceed.

### BC-SESSION-MGR-001: Sync acquires manager mutex for the entire operation
**Postconditions:** No concurrent Create/Delete during a Sync cycle.
**Evidence:** manager.go:138-140.
**Confidence:** HIGH — confirms BC-SESSION-005.

### BC-SESSION-MGR-002: syncFailCount is incremented on HasSession==false but NEVER triggers Orphan-marking
**Postconditions:** The counter exists for observability/debugging only. Pass 3 BC-SESSION-005 said `syncFails >= syncFailThreshold` triggers transition — **this is incorrect**. The actual implementation only logs the count and returns without mutating any session state.
**Evidence:** manager.go:147-160 + explicit comment 153-158.
**Confidence:** HIGH — **REFINES Pass 3 BC-SESSION-005**: there is no transition to Orphan based on syncFailCount in current code; the comment says "Individual windows are detected as Orphan by SyncWithTmux when HasSession does return true." Pass 3's claim of "transient transition based on counter" was overstated.

### BC-SESSION-MGR-003: syncFailCount = 3 (threshold) is a constant but unused in current logic
**Postconditions:** Reserved for future logic; currently dead. Likely a vestige of a removed orphan-promotion path.
**Evidence:** manager.go:26-29.
**Confidence:** HIGH — minor: constant exists without consumer.

### BC-SESSION-MGR-004: EnsureClaudeConfigured idempotently sets onboarding skip flags in ~/.claude.json
**Postconditions:** Sets `hasCompletedOnboarding: true`, `numStartups: 10`, project trust entries with `hasTrustDialogAccepted: true`. Confirms BC-SESSION-006.
**Evidence:** manager.go:190-222 (referenced from Pass 3).
**Confidence:** HIGH

## ResolveProfile — name → ProfileDef

### BC-SESSION-PROF-001: Empty name → effective default (Default=true profile OR named "default" OR builtin)
**Evidence:** manager.go:95-106; calls profile.ResolveDefault.
**Confidence:** HIGH — confirms BC-SESSION-007.

### BC-SESSION-PROF-002: Non-empty name → exact match; built-in name reserved (manager.go:112-113)
**Postconditions:** profile.BuiltinDefaultName is the only reserved name; users cannot redefine it.
**Evidence:** manager.go:107-115.
**Confidence:** HIGH — confirms BC-PROFILE-003.

### BC-SESSION-PROF-003: Unknown non-builtin name → error with profileConfigHint path
**Postconditions:** Error format: `profile %q not defined in <home>/.lazyclaude/config.json`. Falls back to literal `$HOME/.lazyclaude/config.json` when UserHomeDir fails.
**Evidence:** manager.go:115-116, 121-126.
**Confidence:** HIGH

### BC-SESSION-PROF-004: SetProfiles copies the input slice; nil/empty resets to nil
**Postconditions:** Caller can mutate after passing. Confirms BC-SESSION-012.
**Evidence:** manager.go:65-73.
**Confidence:** HIGH

## Worktree — git worktree management

`worktree.go` is mostly pure functions (no I/O); `gitcmd.go` runs the git commands via a GitRunner abstraction.

### BC-SESSION-WT-001: WorktreePathSegment = ".lazyclaude/worktrees" (plural)
**Postconditions:** Distinct from per-branch ".lazyclaude/worktree/<branch>" (singular) used for custom prompts (role.go:53-58 comment).
**Evidence:** worktree.go:11-12 + role.go:52 + role.go:56-58 comment.
**Confidence:** HIGH

### BC-SESSION-WT-002: IsWorktreePath checks `/.lazyclaude/worktrees/` substring match
**Postconditions:** Detection is path-substring-based; works for both local absolute and relative paths.
**Evidence:** worktree.go:20-23.
**Confidence:** HIGH

### BC-SESSION-WT-003: ValidateWorktreeName rejects 8 special chars: `/`, `\`, `..`, `~`, `^`, `:`, `?`, `*`, `[`
**Postconditions:** Plus: empty/whitespace, leading `-`, trailing `.lock`. Git refname rules + path traversal defense.
**Evidence:** worktree.go:27-43.
**Confidence:** HIGH

### BC-SESSION-WT-004: BuildWorktreePrompt injects an isolation instruction with worktreePath + projectRoot
**Postconditions:** Worker session's system prompt has explicit "NEVER modify files outside this worktree" directive.
**Evidence:** worktree.go:14-18 + 48-50.
**Confidence:** HIGH

### BC-SESSION-WT-005: parseWorktreePorcelain parses `git worktree list --porcelain` blocks split by `\n\n`; filters only entries under .lazyclaude/worktrees/
**Postconditions:** Branch is `refs/heads/<name>` → `<name>`. Path is the `worktree <path>` line. Detached HEAD worktrees have empty Branch.
**Evidence:** worktree.go:67-93.
**Confidence:** HIGH

### BC-SESSION-WT-006: ListWorktrees returns nil (NOT error) when projectRoot is not a git repo
**Postconditions:** Silent fallthrough. Caller (TUI) shows "No worktrees found" rather than error.
**Evidence:** gitcmd.go:86-105.
**Confidence:** HIGH

### BC-SESSION-WT-007: ListWorktreesWithRunner filters out worktrees whose dir no longer exists
**Postconditions:** Stale `git worktree list` entries (after manual rm) are hidden from the chooser.
**Evidence:** gitcmd.go:95-104.
**Confidence:** HIGH

### BC-SESSION-WT-008: CreateWorktreeWithRunner first tries `git worktree add -b <branch> <path>` (new branch); falls back to `git worktree add <path> <branch>` (existing branch)
**Postconditions:** Both branches succeed paths are supported. If both fail, returns combined error output.
**Evidence:** gitcmd.go:73-83.
**Confidence:** HIGH

### BC-SESSION-WT-009: CreateWorktreeWithRunner is idempotent: if wtPath already exists, returns nil success (assumes reuse)
**Postconditions:** Resume worktree without rerunning git worktree add. Matches BC-SESSION-010 "ResumeOpts: SkipGitAdd: true" reasoning.
**Evidence:** gitcmd.go:60-66.
**Confidence:** HIGH

## Role + prompt resolution

### BC-SESSION-ROLE-001: Role enum = {RoleNone="", RolePM="pm", RoleWorker="worker"}
**Postconditions:** RoleNone is the zero value for backward compat. IsValid() validates membership.
**Evidence:** role.go:13-39.
**Confidence:** HIGH

### BC-SESSION-PROMPT-001: resolvePrompt searches 4 locations in priority order
1. `{projectRoot}/.lazyclaude/worktree/{branch}/.lazyclaude/prompts/{filename}` — per-worktree
2. `{projectRoot}/.lazyclaude/prompts/{filename}` — project-level
3. `{homeDir}/.lazyclaude/prompts/{filename}` — global user
4. Embedded default (compiled in)
**Evidence:** role.go:49-106.
**Confidence:** HIGH — confirms BC-SESSION-006 from Pass 3 and Gap-VER-002 from Pass 4.

### BC-SESSION-PROMPT-002: Layer-1 and Layer-2 candidates verified to have `strings.HasPrefix(candidate, cleanRoot)` — defense against `..` traversal
**Postconditions:** Even if a malicious worktree path tries `../../etc/passwd`, the prefix check rejects the candidate.
**Evidence:** role.go:75-86.
**Confidence:** HIGH

### BC-SESSION-PROMPT-003: Layer-3 ($HOME/.lazyclaude/prompts/) uses absolute prefix check
**Postconditions:** Defends against absolute-path injection like `/etc/passwd` via filename arg.
**Evidence:** role.go:88-96 + explicit comment 88-89.
**Confidence:** HIGH

### BC-SESSION-PROMPT-004: resolvePrompt returns fallback if projectRoot is not absolute
**Postconditions:** All custom-prompt locations require absolute projectRoot to reason about paths.
**Evidence:** role.go:65-68.
**Confidence:** HIGH

### BC-SESSION-PROMPT-005: First candidate that reads successfully AND has non-empty body wins
**Postconditions:** Empty-file fallthrough. Read errors silently skip.
**Evidence:** role.go:98-103.
**Confidence:** HIGH

### BC-SESSION-PROMPT-006: branchFromWorktreePath extracts the directory immediately under {projectRoot}/{WorktreePathSegment}/; rejects `..` in result
**Postconditions:** Worktree path `<root>/.lazyclaude/worktrees/foo/bar/baz` → branch `foo`. Worktree path `<root>/x/y` → empty (not a managed worktree).
**Evidence:** role.go:111-126.
**Confidence:** HIGH

### BC-SESSION-PROMPT-007: BuildPMPrompt composes pm.md (resolved via resolvePrompt) + base.md (always embedded — NOT searchable)
**Postconditions:** base.md is always the embedded default; only role-specific prompts are user-customizable.
**Evidence:** role.go:134-150.
**Confidence:** HIGH — **NEW finding**: base.md is intentionally not customizable. Porter must replicate.

### BC-SESSION-PROMPT-008: BuildWorkerPrompt formats with 4 placeholders (projectRoot, worktreePath, sessionID×2); BuildPMPrompt with 3 (sessionID×2, workerList)
**Postconditions:** The prompt template strings have positional %s placeholders. Adding new placeholders requires updating both the embedded templates and the format args.
**Evidence:** role.go:137-150 (PM), 155-169 (Worker).
**Confidence:** HIGH

## LaunchSpec — profile → execution context

### BC-SESSION-LS-001: NewLaunchSpec expands `~` in command path AND `$VAR` in env values
**Postconditions:** Command uses profile.ExpandPath (~/$VAR aware). Env values use os.ExpandEnv ($VAR only).
**Evidence:** launchspec.go:29-46.
**Confidence:** HIGH

### BC-SESSION-LS-002: LaunchSpec.Args are NOT pre-quoted — the launcher-script builder applies shell.Quote at emit time
**Postconditions:** Callers must NOT shell-escape Args. Confirms BC-PROFILE-004.
**Evidence:** launchspec.go:18-20 explicit comment.
**Confidence:** HIGH

### BC-SESSION-LS-003: NewLaunchSpec deep-copies Args and Env so caller mutations don't leak into the spec
**Evidence:** launchspec.go:34-45.
**Confidence:** HIGH

## InferProjectRoot

### BC-SESSION-PROJ-001: InferProjectRoot strips `.lazyclaude/worktrees/<name>` suffix; non-worktree paths pass through unchanged
**Postconditions:** A worktree session's "project root" is the parent repo, not the worktree dir.
**Evidence:** project.go:23-35.
**Confidence:** HIGH

## Service interface

### BC-SESSION-SVC-001: Manager implements Service via compile-time assertion (`var _ Service = (*Manager)(nil)`)
**Postconditions:** GC depends on Service (smaller interface), allowing test mocks.
**Evidence:** service.go:30-31.
**Confidence:** HIGH

## Delta Summary

- New items added: 36 (14 BC-SESSION-STORE, 4 BC-SESSION-GC, 4 BC-SESSION-MGR, 4 BC-SESSION-PROF, 9 BC-SESSION-WT, 1 BC-SESSION-ROLE, 8 BC-SESSION-PROMPT, 3 BC-SESSION-LS, 1 BC-SESSION-PROJ, 1 BC-SESSION-SVC)
- Existing items refined: 1 important — **BC-SESSION-MGR-002 REFINES Pass 3 BC-SESSION-005**: there is no syncFailThreshold-based Orphan promotion in current code; the counter is observability-only.
- Remaining gaps: manager.go body (900 LOC unread — Create flows, Delete flow, ResumeSession, profile flow), client-call boundaries from manager methods.

## Novelty Assessment

Novelty: SUBSTANTIVE

Justification: 36 new contracts including:
- **BC-SESSION-MGR-002** REFINES Pass 3 — Pass 3's claim about syncFailThreshold-triggered orphan promotion was incorrect. The current behavior is "log only, never promote on transient miss." This corrects the porter's model.
- **BC-SESSION-PROMPT-007** base.md is non-customizable (NEW).
- **BC-SESSION-WT-009** CreateWorktree is idempotent when path exists (NEW behavioral guarantee).
- **BC-SESSION-STORE-012** pane-conflict resolution prefers alive over dead (remain-on-exit handling — NEW).
- **BC-SESSION-STORE-001** v1 → v2 forced reset without migration (NEW; porter must reproduce or migrate).
- **BC-SESSION-PROMPT-001..006** the 4-layer prompt search precedence is now fully spec'd with traversal defenses.

These materially change the porter's understanding of session lifecycle, prompt customization, and worktree semantics.

## Convergence Declaration

Another round needed — manager.go body has ~900 LOC unread containing the actual Create/Delete/Rename/Resume implementations. These dispatch to CreateWorktreeOpts, CreatePMSessionOpts, CreateWorkerSessionOpts, and include the launch-script builder that emits the actual `claude` invocation. These are the load-bearing flows for understanding how a session goes from "user pressed N" to "tmux window has claude running."

## State Checkpoint

```yaml
pass: B
subsystem: session
round: 1
status: complete
files_read_full: [store.go, gc.go, worktree.go, role.go, launchspec.go, gitcmd.go, project.go, service.go]
files_read_partial: [manager.go (200/1127)]
contracts_drafted: 36
pass-3-refinements: 1
timestamp: 2026-05-11T21:50:00Z
novelty: SUBSTANTIVE
next_round: 2
```
