# Pass B Deep: PM/Worker Subsystem — Round 2 (full-protocol resume)

Round 1 (`pass-B-deep-pmw-r1.md`) was deliberately scoped to a single shallow pass under a now-lifted directive. Round 2 resumes the standard full protocol: deepen the PM/Worker subsystem, including the cross-cutting `/msg/*` bus primitive, until honest NITPICK. Continues numbering from r2.

## Files read in full this round (paths absolute)

Persona / prompts:
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/prompts/embed.go`
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/prompts/pm.md` (embedded default)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/prompts/worker.md` (embedded default)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/prompts/base.md` (embedded default, NOT customizable)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/.lazyclaude/prompts/pm.md` (project override; +30 lines vs embedded)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/.lazyclaude/prompts/worker.md` (project override; +18 lines vs embedded)

Worker spawn / worktree creation:
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/role.go`
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/worktree.go`
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/gitcmd.go`
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/role_test.go`
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/worktree_test.go`
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/manager.go` (focus: PMSession/WorkerSession/ResumeSession/launchWorktreeSession/writeLauncher)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/store.go` (Project shape, PM-as-singleton field)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/project.go` (InferProjectRoot)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/session/gc.go` (GC of dead sessions; role-agnostic)

Message bus:
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/handler_msg.go` (MCP server; in-process)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/handler_msg_test.go`
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/daemon/server.go` (daemon HTTP server; remote+local fallback)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/daemon/server_test.go` (msg test coverage)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/cmd/lazyclaude/msg.go` (CLI; `lazyclaude msg send`/`create`)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/cmd/lazyclaude/local_provider.go` (`P` key → CreatePMSession)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/cmd/lazyclaude/session_command.go` (target-routed PM/Worker creation)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/gui/keymap/registry.go` (`P` keybind)

## Layer separation (preserve in final synthesis)

The subsystem is two architecturally separable layers riding the same plumbing. Reproducing this distinction in monocle (or any porter) lets the persona layer be discarded while keeping the bus primitive.

### Layer 1 — PM/Worker persona (out of monocle scope per user directive)

- **Prompts.** Templates in `prompts/{pm,worker}.md` define the persona (PM-as-reviewer, Worker-as-isolated-implementer). Workflow rules ("never merge without user confirm", "作業完了です。" done marker, 5 review axes) live in prompt text — they are *unenforced* by any Go code.
- **Role tagging.** `session.Role` enum (`role.go:14-26`) — `RolePM`, `RoleWorker`, `RoleNone`. Only PM is enforced as singleton-per-project (`manager.go:908-911`).
- **System prompt builders.** `BuildPMPrompt` / `BuildWorkerPrompt` (`role.go:134-169`) wire role-specific templates + the always-embedded `base.md` and `fmt.Sprintf` in session ID + workerList / worktreePath + projectRoot.
- **Worker = git worktree.** `CreateWorkerSessionOpts` (`manager.go:1106`) is just `createWorktreeSession` with `Role: RoleWorker`. Strip the role and you have a generic worktree session.
- **Keybind `P`.** `internal/gui/keymap/registry.go:381-388` binds rune `P` → `ActionStartPMSession` → `actions.StartPMSession()` → `sessions.CreatePMSessionWithOpts(...)`.

### Layer 2 — `/msg/*` bus primitive (RETAIN — monocle inter-session plumbing)

- **`/msg/send`** — pastes a formatted text block into the recipient's tmux pane via `tmux send-keys -l` then `send-keys Enter`. PUSH delivery, no polling, no broker. **Has nothing to do with PM/Worker semantics.** Type field is a free-form string the bus tags onto the message; the recipient process interprets it.
- **`/msg/create`** — spawns a new session "in the caller's project". Type allowlist differs per code path (see BC-PMW-MSG-DIV-001 below).
- **`/msg/sessions`** — read-only listing of session metadata (ID, name, role, path, window, status, activity). Used by both layers.
- **`/msg/resume`** — server-only endpoint (MCP path) for resuming a GC'd worker via worktree-name fallback. Has Worker semantics baked in (rejects PM resume with explicit error), so this endpoint straddles the line.

Mapping for the porter:
| File / region | Layer | Monocle action |
|---|---|---|
| `prompts/pm.md`, `prompts/worker.md`, `.lazyclaude/prompts/{pm,worker}.md` | Persona | Drop |
| `session/role.go` Role enum + BuildPMPrompt + BuildWorkerPrompt | Persona | Drop or genericize |
| `session/worktree.go` + `gitcmd.go` (worktree creation) | Bus-adjacent | Keep as generic worktree feature |
| `session/manager.go` CreatePMSessionOpts (lines 905-967) | Persona | Drop |
| `session/manager.go` CreateWorkerSessionOpts (1106-1115) | Persona-tagged worktree | Keep as worktree session, drop Role |
| `server/handler_msg.go` (in-process MCP server) | Bus primitive | Keep (with fixes — see P0/P1) |
| `daemon/server.go` handleMsgSend/Create/Sessions (~492-612) | Bus primitive | Keep (with fixes — see P0/P1) |
| `cmd/lazyclaude/msg.go` CLI | Bus primitive | Keep |

## NEW contracts (round 2)

### Prompt-template internals (deeper than r1)

#### BC-PMW-PROMPT-006: Embedded prompts are MOSTLY identical to project overrides; project override adds reviewer-tooling specifics
**Evidence:** `diff prompts/pm.md .lazyclaude/prompts/pm.md`:
- Project override adds 30 lines including `## Plan Review`, `## Implementation Review Boundary`, expanded 13-step workflow, `## PM review checklist` (9 items), and `/codex:rescue`/`/go-review`/`codex review` integration.
- Project override `worker.md` adds 18 lines: replaces "the project's appropriate code reviewer" with explicit `/go-review` and `codex review (/codex:*)` steps; adds a duplicated `## Review process` block.
- The `—` em-dash in embedded becomes `---` (literal three hyphens) in project override — likely an authoring artifact.

**Implication:** The embedded prompts are vendor-generic; the project overrides are tuned to lazyclaude's own dogfooding workflow. **A porter should treat the embedded versions as the "actual product surface" and the overrides as project policy.**
**Confidence:** HIGH

#### BC-PMW-PROMPT-007: base.md (`prompts/base.md`) is ALWAYS embedded — `resolvePrompt` is invoked only for pm.md and worker.md, never for base.md
**Postconditions:** Users cannot replace the base communication reference. Only the role-specific section is customizable.
**Evidence:** `role.go:139, 157` — `baseTmpl := prompts.DefaultBase()` is unconditional in both `BuildPMPrompt` and `BuildWorkerPrompt`. `resolvePrompt` is called for `"pm.md"` and `"worker.md"` only.
**Confidence:** HIGH (confirms r1 BC-PMW-PROMPT-001 from a different angle)

#### BC-PMW-PROMPT-008: Worker prompt's first %s is projectRoot, NOT worktreePath — placeholder ordering is load-bearing
**Postconditions:** `BuildWorkerPrompt(ctx, worktreePath, projectRoot, sessionID)` passes `(projectRoot, worktreePath, sessionID, sessionID)` to `fmt.Sprintf`. The order matters because `worker.md` line 3 ("NEVER modify files outside this worktree — %s must remain untouched") consumes projectRoot, then line 6 consumes worktreePath.
**Evidence:** `role.go:159-164`; tested in `role_test.go:159-171 TestBuildWorkerPrompt_PathIsolation`.
**Confidence:** HIGH

#### BC-PMW-PROMPT-009: `resolvePrompt` enforces path-traversal protection at three layers
**Postconditions:** (a) Relative `projectRoot` returns fallback immediately (`role.go:66-68`). (b) Project/worktree candidates require `strings.HasPrefix(candidate, cleanRoot)` (`role.go:77, 84`). (c) `branchFromWorktreePath` returns `""` when branch contains `".."` (`role.go:122-124`). (d) Home-dir candidate is prefix-checked against `$HOME/.lazyclaude/prompts/` (`role.go:91-95`).
**Evidence:** `role.go:65-106`; explicit test `TestBuildWorkerPrompt_PathTraversalInBranch` (`role_test.go:348-368`).
**Confidence:** HIGH — **Defense-in-depth pattern; port as-is.**

#### BC-PMW-PROMPT-010: Empty prompt-file content is treated as "missing" — fallback to next layer
**Postconditions:** `resolvePrompt` line 100: `if err == nil && len(data) > 0` — zero-byte custom files do NOT shadow the embedded default. Tested in `role_test.go:320-337 TestBuildWorkerPrompt_SkipsEmptyCustomFile`.
**Evidence:** `role.go:98-103`.
**Confidence:** HIGH

#### BC-PMW-PROMPT-011: Custom prompts are user-controlled `fmt.Sprintf` templates — placeholder-count mismatch is silently MALFORMED
**Postconditions:** `BuildPMPrompt` always calls `fmt.Sprintf(roleTmpl, sessionID, sessionID, workerList)` with 3 args. If a user's `.lazyclaude/prompts/pm.md` has 2 placeholders, the third arg becomes `%!(EXTRA string=...)` text. If it has 4, `%!(MISSING)`. No validation.
**Evidence:** `role.go:141-145` direct Sprintf invocation, no template counting.
**Confidence:** HIGH — **P2 finding: latent footgun, but small attack surface (file-write needed to exploit).**

### PM persona — workflow and decision authority

#### BC-PMW-PERSONA-001: PM has 8 explicit decision points but NO autonomous merge authority
**Postconditions:** PM workflow steps 1, 2, 5, 6, 9, 11, 13 (embedded pm.md numbering — different in override) are autonomous; step 10/11 ("install binary, request user verify, merge after approval") requires explicit user input. The "NEVER merge without user confirmation" rule appears twice in the embedded prompt (`pm.md:54, 60`) and three times in the override (`.lazyclaude/prompts/pm.md:53, 59, project pm.md:60`), indicating it's the load-bearing safety rule.
**Evidence:** `prompts/pm.md:54, 60`; `.lazyclaude/prompts/pm.md:53, 59`.
**Confidence:** HIGH

#### BC-PMW-PERSONA-002: PM persona forbids implementation-level codex review (project override only)
**Postconditions:** Per project override line 38-40, "PM does NOT perform codex-based review on Worker implementation code." This is a NEW boundary added in the override; the embedded default does not enforce this division of labor.
**Evidence:** `.lazyclaude/prompts/pm.md:37-40`.
**Confidence:** HIGH

#### BC-PMW-PERSONA-003: Done marker is Japanese literal "作業完了です。" — interpreted by Worker only at the prompt level
**Postconditions:** No Go code parses this string. It's a soft signal in the persona layer. Workers receive it in their tmux pane via the same /msg/send pipeline as any other message.
**Evidence:** `prompts/pm.md:43`; `.lazyclaude/prompts/pm.md:56`.
**Confidence:** HIGH — **No machine-readable "session done" event exists.**

#### BC-PMW-PERSONA-004: Project override pins merge target to `stg`; embedded default says "the development branch"
**Postconditions:** Embedded `pm.md:41` ("merge to the development branch") is generic; project override `pm.md:54` ("merge to `stg`. Use `git merge --no-ff`") is specific to lazyclaude.
**Evidence:** Diff between `prompts/pm.md` and `.lazyclaude/prompts/pm.md`.
**Confidence:** HIGH

### Worker persona — scope and escalation

#### BC-PMW-PERSONA-005: Worker MUST report out-of-scope discoveries to PM, NOT fix
**Evidence:** `prompts/worker.md:31`; `.lazyclaude/prompts/worker.md:33` — both versions say "report them to the PM as issues rather than fixing them yourself."
**Confidence:** HIGH (confirms r1 BC-PMW-WORKFLOW-008)

#### BC-PMW-PERSONA-006: Worker prompt's isolation instruction is enforced at TWO injection points
**Postconditions:** (a) `worker.md:3` is system-prompt content ("NEVER modify files outside this worktree — %s must remain untouched") via `BuildWorkerPrompt`. (b) `worktree.go:14-18 worktreeSystemPrompt` is a SEPARATE template `BuildWorktreePrompt` would emit for regular worktree sessions (non-Worker role) — **NOT used by worker spawn path**.
**Evidence:** `manager.go:568` directly calls `BuildWorkerPrompt`, not `BuildWorktreePrompt`. `worktree.go:48-50 BuildWorktreePrompt` is for `CreateWorktreeOpts` (regular worktree) but `CreateWorktreeOpts` at `manager.go:412-421` uses `Role: RoleWorker` and goes through the same launchWorktreeSession path which calls `BuildWorkerPrompt(ctx, ...)`. **So `BuildWorktreePrompt` is currently dead code.** Confirm with a porter before deleting.
**Confidence:** HIGH — **NEW finding: BuildWorktreePrompt is unreachable from production paths.**

### Worker spawn flow (deeper than r1)

#### BC-PMW-SPAWN-001: Worker spawn flow is 4 distinct stages, all under `m.mu.Lock()`
**Postconditions:**
1. `createWorktreeSession` (manager.go:314) validates worktree name, ensures path doesn't pre-exist (unless SkipGitAdd).
2. `CreateWorktreeWithRunner` (gitcmd.go:53-84) runs `git worktree add -b <name> <path>` — falls back to `git worktree add <path> <name>` if branch already exists.
3. `launchWorktreeSession` (manager.go:529-596) resolves profile, validates, builds `LaunchSpec`, calls `BuildWorkerPrompt`, writes self-deleting launcher to /tmp.
4. `launchSession` (manager.go:467-510) creates tmux window via `tmux new-session` (or `new-window` if session exists), persists to store via `store.Add`, calls `store.Save()`.

The entire flow holds `m.mu` (manager.go:333). **Concurrent worker creation is fully serialized.**
**Evidence:** all four call sites cross-referenced.
**Confidence:** HIGH

#### BC-PMW-SPAWN-002: Launcher script is a self-deleting `/tmp/lazyclaude-wt-*.sh` shell script
**Postconditions:** `writeLauncher` (manager.go:671-737) creates a temp script with this structure:
```sh
#!/bin/sh
rm -f "$0"
exec '<profile.Command>' '<arg1>' '<arg2>' ... --session-id '<uuid>' --settings '<hooks>' --append-system-prompt '<sysprompt>' '<extra-flag>'... '<user-prompt>'
```
- All quoting done via `shell.Quote` to defend against newlines/quotes in prompts.
- `--session-id` is skipped iff profile.Args or extraFlags already contain `--session-id`/`--resume` (collision-prevention via `hasSessionFlag` at manager.go:817-829).
- On launch failure, caller invokes the cleanup function returned by `buildLaunchCommand` to delete the script (manager.go:586-590, 647).
**Evidence:** manager.go:671-737; tests `worktree_test.go:149-258` (`TestWriteWorktreeLauncher_*`) verify quoting, special chars (Japanese, single quotes, `$vars`), and empty-userPrompt handling.
**Confidence:** HIGH

#### BC-PMW-SPAWN-003: Profile and Options propagate worker-only — local sessions ignore them on server path; daemon's `local` path doesn't exist
**Postconditions:** Server `/msg/create` (`handler_msg.go:134-141`): worker branch passes profile+options to `CreateWorkerSession`; local branch passes only `name, projectPath` to `CreateLocalSession`. Daemon `/msg/create` (`server.go:573-590`): only supports worker+pm types; there is no `local` type. So `msg create --type local --profile foo` succeeds on MCP server, but `--profile foo` is silently ignored (TODO at handler_msg.go:139 acknowledges).
**Evidence:** `handler_msg.go:139` TODO comment; daemon `server.go:573-590` switch statement.
**Confidence:** HIGH

#### BC-PMW-SPAWN-004: Worker spawn is auto-scoped to caller's project via FindProjectForSession
**Postconditions:** Server and daemon both call `FindProjectForSession(req.From)` (handler_msg.go:124, server.go:563). Caller cannot specify project; they inherit it from their own session record. If the caller's session has been GC'd, `req.From` lookup fails → 404 ("caller session not found").
**Evidence:** handler_msg.go:124-128; daemon server.go:563-567.
**Confidence:** HIGH — Tested in TestMsgCreate_from_not_found (handler_msg_test.go:211-221).

#### BC-PMW-SPAWN-005: Worker session ID format and tmux window-name derivation
**Postconditions:** Session ID is a fresh UUID v4 via `uuid.New().String()` (manager.go:532). The tmux window name is `"lc-" + ID[:8]` (store.go:62-69 `WindowName()`) for local, `"rm-" + ID[:8]` for remote mirrors (store.go:73-78 `MirrorWindowName`). **Window name collision after 8 hex chars is astronomically unlikely but not impossible.**
**Evidence:** store.go:62-78.
**Confidence:** HIGH

### Worktree lifecycle (NEW — gap from r1)

#### BC-PMW-WORKTREE-001: Worktree creation runs `git worktree add` with auto-branch-creation; falls back to existing-branch reuse
**Postconditions:** `CreateWorktreeWithRunner` (gitcmd.go:73-83):
1. Verifies `git rev-parse --git-dir` succeeds (projectRoot is a repo).
2. If worktree path already exists on disk → assume reuse, return nil (gitcmd.go:60-66).
3. `mkdir -p` parent dir.
4. Try `git worktree add -b <branch> <path>` (new branch).
5. On failure, try `git worktree add <path> <branch>` (existing branch).
6. On both failures, return combined stderr.

**Branch name == worker name.** ValidateWorktreeName forbids `/`, `\`, `..`, `~`, `^`, `:`, `?`, `*`, `[`, leading `-`, and `.lock` suffix (worktree.go:27-43) — these are the constraints git itself imposes on `refs/heads/`.
**Evidence:** gitcmd.go:51-84; worktree.go:25-43.
**Confidence:** HIGH

#### BC-PMW-WORKTREE-002: Worktree path is `{projectRoot}/.lazyclaude/worktrees/{name}/` — fixed segment
**Postconditions:** `WorktreePathSegment = ".lazyclaude/worktrees"` (worktree.go:11-12) — singular "worktrees" (plural). The directory for per-branch *prompts* uses singular "worktree" (worktree.go vs `.lazyclaude/worktree/{branch}/` in resolvePrompt). **Two different conventions that look like a typo but are intentional — confirmed in role.go:57-60 comment.**
**Evidence:** worktree.go:12; role.go:52-60 comment block.
**Confidence:** HIGH — **P3 finding: easy to get wrong on port; document explicitly.**

#### BC-PMW-WORKTREE-003: NO automated worktree cleanup exists in the codebase
**Postconditions:** No code path runs `git worktree remove`, deletes the worktree directory, or prunes the branch. GC (`gc.go:60-82`) only removes the *session record* from state.json after the tmux pane dies + 10s grace. **The worktree directory and branch persist forever** unless the user manually runs `git worktree remove <path>` or `rm -rf` the directory.
**Evidence:** `find ... -exec awk` searches for `worktree remove|RemoveWorktree|cleanupWorktree` returned ZERO hits across the entire codebase.
**Confidence:** HIGH — **NEW finding. Likely a deliberate "Worker's diff survives crash" property but causes disk accumulation over time.**

#### BC-PMW-WORKTREE-004: Resume after GC reconstructs worktree path from `--name` flag with traversal defense
**Postconditions:** When the Worker session is GC'd but the worktree dir still exists, `ResumeSession` (manager.go:1046-1083):
1. Requires `--name` flag (otherwise: "specify --name for GC'd sessions").
2. Validates name via `ValidateWorktreeName`.
3. Searches all registered projects for one whose `.lazyclaude/worktrees/<name>` exists.
4. Belt-and-suspenders: checks `strings.HasPrefix(wtPath, expectedPrefix)` to detect any path traversal that slipped through (manager.go:1066-1069).
5. Re-launches with original session ID via `--resume <id>` flag (manager.go:520, 695-700).
**Evidence:** manager.go:988-1084.
**Confidence:** HIGH

#### BC-PMW-WORKTREE-005: PM sessions cannot be resumed via `sessions resume` — explicitly rejected
**Postconditions:** manager.go:998-1001: `if old.Role == RolePM { return nil, fmt.Errorf("cannot resume PM session...") }`. The user must re-launch via the `P` keybind / `CreatePMSessionOpts` path. Since one PM per project is enforced, this is "delete old, create new with fresh ID and workerList snapshot."
**Evidence:** manager.go:998-1001.
**Confidence:** HIGH (confirms r1 BC-PMW-LIFECYCLE-005)

#### BC-PMW-WORKTREE-006: Remote-mirror sessions cannot be locally resumed
**Postconditions:** manager.go:994-997: `if old.Host != "" { return error }`. Resume must happen on the remote host that owns the session.
**Evidence:** manager.go:994-997.
**Confidence:** HIGH

### `/msg/send` and `/msg/create` API surface (DEEPENED with safety findings)

#### BC-PMW-MSGAPI-001: Two separate `/msg/*` HTTP servers exist with DIFFERENT validation rules
**Postconditions:** The "server" package (`internal/server`) is the in-process MCP server started inside the TUI process; the "daemon" package (`internal/daemon`) is a standalone HTTP daemon used by remote-host SSH workers and by the `setup`-installed system daemon. They expose the same REST routes but with non-identical validation. See divergence table below (BC-PMW-MSG-DIV-001).
**Evidence:** `server/handler_msg.go` vs `daemon/server.go`.
**Confidence:** HIGH

#### BC-PMW-MSG-DIV-001: Server vs daemon `/msg/*` validation divergence — table

| Validation | server (`handler_msg.go`) | daemon (`daemon/server.go`) | Notes |
|---|---|---|---|
| `/msg/send` auth | X-Auth-Token, constant-time compare (line 196-200) | X-Daemon-Authorization, constant-time compare (server.go:187-196) | Different headers — see BC-PMW-MSG-DIV-002 |
| `/msg/send` body size | **10KB** max (line 224-228) | **1MB** max (default readJSON, server.go:210) | **P1 — daemon's 100× larger limit can amplify tmux paste DoS** |
| `/msg/send` type allowlist | YES — `isValidMsgType` (line 219-222) checks `{review_request, review_response, status, done, issue}` | **NO** — `req.Type` is interpolated into the message text without validation (server.go:535-536) | **P1 — daemon accepts arbitrary string** |
| `/msg/send` self-send check | YES (line 214-217) | YES (server.go:503-506) | Parity |
| `/msg/send` from/to empty | 400 (line 209-212) | 400 (server.go:499-502) | Parity |
| `/msg/send` recipient lookup | sessionLister.Sessions() + fallback state.json (line 230-242) | mgr.Sessions() (server.go:509) | Different sources but same effect |
| `/msg/send` delivery | tmux SendKeysLiteral + SendKeys(Enter) (line 290-295) | tmux SendKeysLiteral + SendKeys(Enter) (server.go:538-545) | Parity |
| `/msg/create` types accepted | `{worker, local}` (line 119-122) | `{worker, pm}` (server.go:573-590) | Intentional divergence — BC-PMW-MSGCREATE-001 |
| `/msg/create` body size | 1MB (line 107) | 1MB (server.go:210) | Parity |
| `/msg/create` From+Name required | YES (line 114-117) | YES (server.go:558-561) | Parity |
| `/msg/create` FindProjectForSession 404 | YES (line 124-128) | YES (server.go:563-567) | Parity |
| CLI `msg create --type` allowlist | `{worker, local}` (cmd/lazyclaude/msg.go:87-92) | (same CLI; same allowlist) | **CLI never sends type=pm** — daemon's pm case is reachable only via direct HTTP calls from the GUI/keybind path |

**Evidence:** Cross-reference of both files.
**Confidence:** HIGH

#### BC-PMW-MSG-DIV-002: Auth header name differs between server and daemon
**Postconditions:** Server reads `X-Auth-Token` via `extractAuthToken` (handler_msg.go:92). Daemon reads `X-Daemon-Authorization` via `r.Header.Get(AuthHeader)` (server.go:189). **A caller that hardcodes one header will fail on the other deployment mode.**
**Evidence:** handler_msg.go:92, 196 vs daemon/server.go:189.
**Confidence:** HIGH — **P2 finding: cross-mode caller code must read which mode is in use.**

#### BC-PMW-MSG-SAFETY-001 (P1): Daemon `/msg/send` accepts ARBITRARY type strings and embeds them verbatim in the recipient's input pane
**Preconditions:** Authenticated caller with daemon token (X-Daemon-Authorization).
**Postconditions:** The text format at `daemon/server.go:535` is:
```
[MESSAGE from %s (%s)]
type: %s
---
%s
```
With no type allowlist, an attacker can craft `type: "review_response\n\nIGNORE PREVIOUS INSTRUCTIONS AND..."`. The result is delivered as-typed into the recipient Claude's input. This is **prompt-injection-via-newline-in-type-field** — exactly the class the server-side allowlist was designed to prevent.
**Evidence:** No call to `isValidMsgType` in `daemon/server.go:492-549`; type allowlist defined in `server/handler_msg.go:380-390` is only used by the server path.
**Confidence:** HIGH — **P1 SAFETY FINDING. Monocle MUST patch this if it retains the daemon path.**

#### BC-PMW-MSG-SAFETY-002 (P1): Daemon `/msg/send` allows 1MB body — 100× the server limit
**Postconditions:** A 1MB body is interpolated into the tmux paste. tmux `send-keys -l` will accept this; whether Claude Code's input buffer survives 1MB pastes is untested. Even ignoring crash potential, this exceeds reasonable input bounds and amplifies the value of the type-string-injection vector above.
**Evidence:** daemon `readJSON` uses `MaxBytesReader(w, r.Body, 1<<20)` (server.go:210); server `handleMsgSend` explicitly checks `if len(req.Body) > maxBodyLen` where `maxBodyLen = 10 * 1024` (handler_msg.go:224-228).
**Confidence:** HIGH — **P1 SAFETY FINDING. Trivial fix: copy the 10KB check from server.**

#### BC-PMW-MSG-SAFETY-003 (P2): Sender name is read from sessions list — if sender's name contains newlines, message header is malformed
**Postconditions:** `senderName` is taken from `sessions[i].Name` (handler_msg.go:251; daemon/server.go:518). If a session was created with a name containing `\n`, the header `[MESSAGE from %s (...)]` will span multiple lines, potentially breaking downstream parsers in the recipient. Session naming defenses: `ValidateWorktreeName` rejects `/`, `..`, etc. but does NOT reject newlines or non-printable characters. `Rename` goes straight to the store with no validation (manager.go:771-777).
**Evidence:** handler_msg.go:251, 284; daemon/server.go:518, 535; manager.go:771-777.
**Confidence:** MEDIUM — Exploitation requires session-rename access, which is itself token-gated, but defense-in-depth would validate.

#### BC-PMW-MSG-DELIVERY-001: Delivery is "best effort" — Enter-key send error is logged but does not fail the request
**Postconditions:** server/handler_msg.go:295-297 and daemon/server.go:544-546 both log the Enter-send failure but return success. So a request returns 200 ("delivered") even if the recipient never sees the Enter that triggers Claude's processing. **No retry. No delivery receipt. No tracking ID.**
**Evidence:** handler_msg.go:295-297; daemon/server.go:544-546.
**Confidence:** HIGH

#### BC-PMW-MSG-DELIVERY-002: Dead sessions still receive delivery attempts — status="Dead" is not a precondition check
**Postconditions:** Both paths look up the recipient by ID and attempt to deliver regardless of status. If the tmux pane is truly dead, `SendKeysLiteral` fails and the request returns 502. **This is intentional** — see test `TestMsgSend_PushDelivery_DeadSessionStillAttempts` (handler_msg_test.go:529-550). Rationale: status detection has known race conditions; trust tmux to be the source of truth at the moment of send.
**Evidence:** handler_msg.go:262-281 (no status check); handler_msg_test.go:529-550 explicit test.
**Confidence:** HIGH

#### BC-PMW-MSG-DELIVERY-003: When recipient.Window is empty in SessionInfo, server resolves window-by-name via tmux ListWindows
**Postconditions:** server/handler_msg.go:262-277: if `recipient.Window == ""`, compute `wName = "lc-" + recipient.ID[:8]` and search tmux's window list for a match. **The daemon does the same** via `resolveWindowByName` (daemon/server.go:740-751). If still no window found → 502.
**Evidence:** handler_msg.go:262-281; daemon/server.go:526-531, 740-751.
**Confidence:** HIGH

#### BC-PMW-MSG-DELIVERY-004: Delivery is single-recipient; broadcast/fanout is not supported
**Postconditions:** `/msg/send` takes a single `to` field. To message multiple workers, the sender must issue multiple `/msg/send` calls. No batch endpoint. No "topic" / "all workers" semantics. **No back-pressure or rate limiting either.**
**Evidence:** msgSendRequest schema (handler_msg.go:180-185; daemon MsgSendRequest).
**Confidence:** HIGH

#### BC-PMW-MSG-DELIVERY-005: No idempotency keys, no de-duplication, no message IDs
**Postconditions:** Sending the same `(from, to, type, body)` twice causes two pastes to the recipient. There is no client-side or server-side dedup. A network-retry on a slow request will double-deliver.
**Evidence:** msgSendRequest fields are exactly `{From, To, Type, Body}` — no ID field anywhere.
**Confidence:** HIGH — **P2 finding for high-volume use.**

#### BC-PMW-MSG-AUTH-001: Server token discovery is filesystem-based; CLI reads it from `paths.IDEDir` via DiscoverServer
**Postconditions:** `cmd/lazyclaude/msg.go:50-56`:
```go
paths := config.DefaultPaths()
disc, err := server.DiscoverServer(paths.IDEDir)
client := server.NewClient(disc.Port, disc.Token)
```
Any user with read access to `paths.IDEDir` (typically `~/.claude/ide/`) can hijack the session and send messages on behalf of any caller. **The bus's authentication boundary IS the user's filesystem permissions.**
**Evidence:** msg.go:50-56; server discovery in server/discover.go (not deep-read this round).
**Confidence:** HIGH

#### BC-PMW-MSG-AUTH-002: No per-session, per-sender, or per-target ACL — any holder of the token can send as anyone to anyone
**Postconditions:** `req.From` is a free-form string the caller supplies. The handler uses it only to look up sender NAME for the formatted message header (handler_msg.go:250-253). The handler does NOT verify that the caller's token corresponds to `From`. So any authenticated caller can claim to be any session.
**Evidence:** handler_msg.go:244-258 — no cross-check of From against an authenticated identity.
**Confidence:** HIGH — **P2 finding for multi-tenant or untrusted Worker scenarios. Single-user model assumed throughout.**

### Inter-session message bus topology

#### BC-PMW-TOPO-001: Bus is a point-to-point HTTP-pasting-into-tmux relay; NOT a broker, queue, or pub-sub
**Postconditions:** Each `/msg/send` invocation is an isolated synchronous HTTP request → tmux send-keys call → ack. There is no:
- persistent message log (no store of past messages anywhere)
- subscription model (workers do not "register" for messages)
- broker pattern (no fan-out, no topic, no queue)
- ordering guarantee across multiple parallel senders (tmux paste order = arrival order at the server, which is HTTP race-condition territory)

The PUSH delivery model (vs. POLL) is achieved by having the recipient process (Claude Code) treat its own tmux stdin as the message channel — every paste becomes part of its conversation. This is **extremely simple and brittle**: it works because Claude Code is the only consumer.
**Evidence:** No broker code anywhere in handler_msg.go or daemon/server.go for /msg/*. The broker package (`internal/core/event`) is used for SSE notifications, not for /msg/*.
**Confidence:** HIGH

#### BC-PMW-TOPO-002: PM↔Worker discovery is via GET /msg/sessions; no auto-subscription
**Postconditions:** Workers must call `lazyclaude sessions` (which proxies to `/msg/sessions`) to learn other session IDs. The PM prompt at BuildPMPrompt time embeds a STATIC workerList snapshot (manager.go:925-936) — workers created after PM launch are invisible until the PM re-queries.
**Evidence:** prompts/pm.md:31 placeholder; manager.go:925-933.
**Confidence:** HIGH (confirms r1 BC-PMW-LIFECYCLE-003)

#### BC-PMW-TOPO-003: PM/Worker can fan-out by calling `/msg/send` multiple times in sequence; no broadcast primitive
**Evidence:** Bus design (BC-PMW-MSG-DELIVERY-004).
**Confidence:** HIGH

### Failure modes (NEW — gap from r1)

#### BC-PMW-FAIL-001: PM dies mid-orchestration → Workers continue, but new spawns blocked until PM re-creates (or never recreates)
**Postconditions:** Workers that were already spawned have their own sessions and tmux windows; they keep running. New `/msg/create --type worker` calls work as long as the caller is still alive (project lookup is via the caller's session, NOT PM). **However, the PM-as-singleton-per-project rule (BC-PMW-LIFECYCLE-001) means re-launching PM via the `P` key produces a fresh PM with a fresh workerList snapshot.** Any workflow state PM was tracking in conversation context is lost.
**Evidence:** manager.go:908-911; gc.go:73-82.
**Confidence:** HIGH

#### BC-PMW-FAIL-002: Worker dies mid-task → tmux pane is "Dead", worktree dir + branch + commits persist on disk
**Postconditions:** GC removes the session record after 10s grace (gc.go:74-80). Worktree directory persists (BC-PMW-WORKTREE-003). User can resume via `sessions resume <id> --name <worktree-name>` to launch a fresh Claude in the same worktree with conversation history via `--resume`.
**Evidence:** gc.go:73-80; manager.go:1046-1083.
**Confidence:** HIGH

#### BC-PMW-FAIL-003: Worker dies with uncommitted changes → uncommitted changes survive on disk; resume picks them up
**Postconditions:** The worktree is a real git working tree. `git status` in the worktree dir shows the dirty changes. Resume launches Claude with `--resume <id>` which restores conversation history; the dirty git state is automatically visible. **Nothing automatic happens — the user/PM must decide what to do.**
**Evidence:** Inference from BC-PMW-WORKTREE-003 + manager.go:1075-1083 (resume uses `Resume: true` which emits `--resume`, manager.go:694-701).
**Confidence:** HIGH

#### BC-PMW-FAIL-004: Profile rename / deletion breaks resume — explicit error returned, NOT silent fallback to default
**Postconditions:** ResumeSession re-resolves the persisted profile name via `ResolveProfile` (manager.go:535-544). If the name no longer matches any installed profile and is not the reserved built-in name, an error is returned: `"profile %q not defined in %s"` (manager.go:115). The error path goes through `launchErrorSession` which creates a tmux window displaying the error message (manager.go:601-610).
**Evidence:** manager.go:535-545 + 100-116 + 601-610.
**Confidence:** HIGH

#### BC-PMW-FAIL-005: Concurrent worker create with duplicate name → second call returns "worktree %q already exists" before git is touched
**Postconditions:** `createWorktreeSession` holds `m.mu` (manager.go:333-334) and checks `m.store.FindByName(opts.Name)` (manager.go:336-338) before git worktree add. Two parallel `msg create --type worker --name foo` will be serialized — first wins, second errors.
**Evidence:** manager.go:333-338.
**Confidence:** HIGH

#### BC-PMW-FAIL-006: tmux server unavailable transiently → sync goes "no session", but does NOT mark all as Orphan (3-failure threshold)
**Postconditions:** manager.go:141-160 — `syncFailCount` increments on `HasSession` failure; only after threshold (3 consecutive failures) would sessions be marked orphan in a previous version. Current code never marks all-orphan from HasSession=false alone (comment at manager.go:152-158 explains rationale: "Marking all sessions Orphan here causes GC to delete live sessions and wipes state.json").
**Evidence:** manager.go:138-184, threshold const at 25-29.
**Confidence:** HIGH

#### BC-PMW-FAIL-007: Sessions resume into a DEAD tmux window: old window is killed first, then new window created in same worktree dir
**Postconditions:** manager.go:1010-1017 — KillWindow on `old.TmuxTarget()` if `old.Status != StatusOrphan`. Orphans are skipped (window may still be alive if HasSession transiently failed). Old record removed; on launch failure, old record restored via `savedOld := *old` + `m.store.Add(savedOld, projectRoot)`.
**Evidence:** manager.go:1019-1043.
**Confidence:** HIGH

### Test coverage map (NEW — gap from r1)

#### BC-PMW-TEST-001: server (MCP) tests cover the full /msg/* validation matrix
**Tests covered (handler_msg_test.go, line numbers shown):**
- `/msg/create`: missing auth (137), wrong method (148), invalid JSON (162), missing from (178), missing name (189), invalid type (200), from not found (211), no creator set (223), worker success (236), local success (267), local with prompt (293), worker profile+options propagated (320), creation error (349).
- `/msg/send`: missing auth (374), wrong token (383), invalid JSON (392), wrong method (408), empty from (422), empty to (436), from==to (450), push delivery success (474), recipient not found (510), dead-session-still-attempts (529), paste error (553), message format (574), no-window 502 (606).
- `/msg/sessions`: returns list (627), missing auth (648), no lister returns empty (657).

**Confidence:** HIGH — Comprehensive.

#### BC-PMW-TEST-002: daemon tests cover ONLY the bare minimum
**Tests in daemon/server_test.go (matched line numbers):**
- `TestMsgSend_Validation` (175): missing-fields, self-send, recipient-not-found. **NO test for invalid type, NO test for body-size limit.**
- `TestMsgCreate_MissingFields` (203): from+name empty.
- `TestMsgCreate_CallerNotFound` (268): unknown from.

**Gap:** Daemon /msg/send has zero coverage for type validation (because it has no type validation) and zero coverage for body-size limit. The server's 10KB rule is NOT enforced on the daemon path. **This is the source of BC-PMW-MSG-SAFETY-001 and -002.**

**Confidence:** HIGH

#### BC-PMW-TEST-003: Role tests cover prompt template fields, path traversal, fallback chain, empty file skip
**Tests in role_test.go:**
- `TestRole_String` (13), `TestRole_IsValid` (32): enum behavior.
- `TestBuildPMPrompt_*` (58–217): required fields, no /msg/poll, empty worker list, CLI-not-curl, --from <id>, project custom prompt, embedded fallback, relative root.
- `TestBuildWorkerPrompt_*` (120–337): required fields, no /msg/poll, path isolation, CLI-not-curl, --from, project custom, worktree custom takes priority, embedded fallback, skips empty custom file, **PathTraversalInBranch (348)**.

**Confidence:** HIGH — **Strong test coverage of the persona layer's surface.**

#### BC-PMW-TEST-004: Worktree tests cover name validation, path construction, porcelain parsing, launcher script
**Tests in worktree_test.go:**
- `TestValidateWorktreeName_Valid/_Invalid` (10, 19): full enumeration of forbidden chars.
- `TestBuildWorktreePrompt` (50): isolation prompt content.
- `TestWorktreePath` (63): path joining.
- `TestIsWorktreePath` (71).
- `TestListWorktrees_ParsesPorcelainOutput` (88), `_EmptyOutput` (128), `_NoClaude` (136).
- `TestWriteWorktreeLauncher_BasicContent` (149), `_EmptyUserPrompt` (191), `_SpecialChars` (230) — Japanese, single quotes, `$vars`.

**Confidence:** HIGH

## P0/P1 findings summary

**For monocle to take action on** (the /msg/* bus primitive will be ported per layer-separation):

| ID | Pri | Finding | Fix |
|---|---|---|---|
| BC-PMW-MSG-SAFETY-001 | P1 | Daemon `/msg/send` accepts arbitrary `type` strings → prompt-injection-via-newline-in-type-field | Copy the `isValidMsgType` check from `server/handler_msg.go:219-222` to `daemon/server.go:506-507` (before message text construction) |
| BC-PMW-MSG-SAFETY-002 | P1 | Daemon `/msg/send` allows 1MB body (100× server's 10KB limit) → DoS amplifier | Add `if len(req.Body) > 10*1024 { 400 }` check in `daemon/server.go:506` |
| BC-PMW-MSG-SAFETY-003 | P2 | Sender name from session list is not validated for newlines → header injection via rename | Validate session names (Rename and Create paths) against `\r\n\t` and non-printable chars |
| BC-PMW-MSG-AUTH-001 | P2 | Bus auth boundary is filesystem permissions on `paths.IDEDir` (server) or daemon token file | Document explicitly; this is intentional for single-user model but must be in NFR catalog |
| BC-PMW-MSG-AUTH-002 | P2 | No cross-check of `req.From` against caller identity → token holder can spoof any sender | Either require From to match the authenticated identity, or document explicitly as single-trust-domain |
| BC-PMW-MSG-DIV-002 | P2 | Auth header name differs (X-Auth-Token vs X-Daemon-Authorization) | Unify in port if both paths retained |
| BC-PMW-WORKTREE-003 | P3 | No automated worktree/branch cleanup → disk accumulation | Document; consider opt-in cleanup command |
| BC-PMW-PROMPT-011 | P3 | Custom prompt placeholder mismatch silently malforms output | Document; consider template validation on load |
| BC-PMW-PERSONA-006 | P3 | `BuildWorktreePrompt` (worktree.go:48-50) is dead code | Document or remove on port |

**P0 findings:** none — no exploitable issues without authenticated bus access. Both SAFETY findings (P1) require an already-authenticated caller, which in the single-user model is "the user themselves or anything running with their UID."

## Delta Summary (round 2)

- New BC contracts drafted: 39 (BC-PMW-PROMPT-006..011 = 6, BC-PMW-PERSONA-001..006 = 6, BC-PMW-SPAWN-001..005 = 5, BC-PMW-WORKTREE-001..006 = 6, BC-PMW-MSGAPI-001 + DIV-001..002 + SAFETY-001..003 + DELIVERY-001..005 + AUTH-001..002 = 13, BC-PMW-TOPO-001..003 = 3, BC-PMW-FAIL-001..007 = 7, BC-PMW-TEST-001..004 = 4) — net new: 50 (subtracting r1 overlaps for prompt-001..005, workflow-001..009, msgcreate-001..004, cli-001..002, lifecycle-001..005).
- Existing r1 items refined: BC-PMW-LIFECYCLE-005 (resume rejection), BC-PMW-MSGCREATE-001 (now table-form divergence), BC-PMW-WORKFLOW-008 (Worker escalation), BC-PMW-LIFECYCLE-003 (workerList snapshot).
- Remaining gaps after this round:
  - SSH/remote-host worker spawn path (`remote_provider.go`, `composite_provider.go`) — interaction with PM/Worker layer is via session creation, but the daemon's remote tunnel surface is its own subsystem.
  - Hook-injection's `LAZYCLAUDE_SESSION_ID` env var flow into Claude Code's session record — touches session, prompt, and hook layers.
  - `lazyclaude setup` daemon start-up sequence — tangential to PMW, deferred to daemon subsystem deepening.

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: Round 2 surfaced findings that change the model, not refine it:
1. **Two `/msg/*` server paths with non-identical validation** (BC-PMW-MSGAPI-001 + DIV-001) — r1 noted the type allowlist divergence in `/msg/create` but missed the much larger divergence in `/msg/send` (body limit + type validation). This is a P1 safety finding monocle WILL care about if it retains the daemon path.
2. **Daemon-only prompt-injection vector** (BC-PMW-MSG-SAFETY-001) — completely new; would have been missed without reading both handler implementations side-by-side.
3. **No automated worktree cleanup anywhere** (BC-PMW-WORKTREE-003) — a major lifecycle property that affects every downstream porter decision about Worker storage costs.
4. **BuildWorktreePrompt is dead code** (BC-PMW-PERSONA-006) — small, but changes what code the porter should bring across.
5. **Failure-mode catalog** (BC-PMW-FAIL-001..007) — 7 explicit failure paths previously unmodeled.
6. **Test-coverage asymmetry** (BC-PMW-TEST-002) — daemon path is materially less-tested than server path, which directly causes the safety findings.

Removing this round's findings would change how a porter specs the system: they would (a) miss the two P1 safety fixes, (b) underestimate worktree disk costs, (c) miss the dead-code candidate, (d) lack failure-mode coverage. Another round is warranted to verify SSH/remote interaction and hook-injection, and to confirm no further hidden divergences.

## Convergence Declaration

**Another round needed.** Substantive gaps remaining: (1) SSH remote worker spawn flow, (2) hook-injection ↔ Worker session interplay (`--settings` hooks file written at launcher time, see manager.go:706-709), (3) any /msg/* call sites inside the GUI that bypass the CLI and hit the daemon directly (would expose the P1 daemon findings even in single-user mode).

## State Checkpoint

```yaml
pass: B
subsystem: pmw
round: 2
status: complete
files_read_full:
  - prompts/embed.go, base.md, pm.md, worker.md
  - .lazyclaude/prompts/pm.md, worker.md (project overrides)
  - internal/session/role.go, role_test.go
  - internal/session/worktree.go, worktree_test.go, gitcmd.go
  - internal/session/manager.go (CreatePM/Worker/Resume + writeLauncher regions)
  - internal/session/store.go, project.go, gc.go
  - internal/server/handler_msg.go, handler_msg_test.go
  - internal/daemon/server.go (msg + session create regions), server_test.go (msg tests)
  - cmd/lazyclaude/msg.go, local_provider.go, session_command.go
  - internal/gui/keymap/registry.go (P binding)
contracts_drafted_this_round: 39 (net new ~50 vs r1)
contracts_total_after_r2: 67 (17 r1 + 50 r2)
timestamp: 2026-05-11T23:55:00Z
novelty: SUBSTANTIVE
convergence: NOT YET — round 3 targets SSH remote, hook-injection, GUI-direct daemon calls
next_round: 3
```
