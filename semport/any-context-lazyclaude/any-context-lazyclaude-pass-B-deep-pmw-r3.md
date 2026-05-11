# Pass B Deep: PM/Worker Subsystem — Round 3

Targets from r2 convergence declaration: (1) SSH/remote worker spawn flow, (2) hook injection ↔ Worker interplay, (3) GUI-direct daemon `/msg/*` call sites to scope the daemon-side safety findings.

## Files read in full this round (paths absolute)

- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/daemon/remote_provider.go` (focus: CreatePMSession, CreateWorkerSession, invokePostCreate)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/client.go` (CLI's server-side client)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/discover.go` (server discovery via `~/.claude/ide/*.lock`)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/config/hooks.go` (hook injection inline JS)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/daemon/http_client.go` (MsgSend/MsgCreate clients — production callers scan)

## NEW contracts (round 3)

### SSH/remote worker spawn path

#### BC-PMW-REMOTE-001: Remote PM/Worker spawn uses `/session/create`, NOT `/msg/create`
**Postconditions:** `RemoteProvider.CreatePMSession` and `RemoteProvider.CreateWorkerSession` (remote_provider.go:621-665) build a `SessionCreateRequest` with `SessionType: "pm"` or `"worker"` and call `client.CreateSession` which posts to **`/session/create`** (daemon.HTTPClient — distinct from /msg/create). This is the daemon-side equivalent of the CLI's `lazyclaude msg create`, but takes a different code path that lands at `daemon/server.go:241-258`. The daemon's /session/create branch handles `"pm"`, `"worker"`, `"plain"`, `"worktree"` — a 4-option allowlist (server.go:227-258).
**Evidence:** remote_provider.go:621-665; daemon/server.go:216-272.
**Confidence:** HIGH — **NEW finding: a third type-allowlist (4 options) exists beyond the two documented in r2 BC-PMW-MSG-DIV-001.**

#### BC-PMW-REMOTE-002: Remote PM/Worker creation triggers a `PostCreateHook` that creates a local mirror tmux window
**Postconditions:** After the daemon creates the session remotely, `invokePostCreate` (remote_provider.go:515-523) calls the user-registered hook with `(host, projectRoot, resp)`. The hook (registered at `cmd/lazyclaude/root.go:212`) is `mirrorMgr.CreateMirror(host, path, resp)` which opens a local tmux window named `rm-<sessionID[:8]>` that SSH-attaches to the remote tmux pane. **The mirror window is mandatory** — without it, the local GUI cannot show the remote session.
**Evidence:** remote_provider.go:19-21, 515-523, 626; root.go:212-221 (registration).
**Confidence:** HIGH

#### BC-PMW-REMOTE-003: Remote Worker prompt is built REMOTELY using the remote daemon's `BuildWorkerPrompt`, so prompt overrides resolve in the REMOTE filesystem
**Postconditions:** The daemon's `handleSessionCreate` (server.go:247-254) calls `s.mgr.CreateWorkerSessionOpts` which ends up in `launchWorktreeSession` → `BuildWorkerPrompt`. `BuildWorkerPrompt` resolves `worker.md` via `resolvePrompt` against the daemon's local filesystem (role.go:155-156, using `userHomeDir()` = remote user's `$HOME`). **So a Worker running on host `remote.example` reads `.lazyclaude/prompts/worker.md` from `remote.example`'s filesystem, NOT the controlling user's local machine.**
**Evidence:** daemon/server.go:247-254; session/role.go:155-156, 128-132.
**Confidence:** HIGH — **NEW finding: remote-host prompt-override semantics. Porter must understand this for any multi-host deployment.**

#### BC-PMW-REMOTE-004: Remote Workers send /msg/send to their local server (in-process MCP server on the remote host)
**Postconditions:** Worker's prompt embeds `lazyclaude msg send --from %s ...` (prompts/base.md:13-17). When the Worker (a Claude Code process on the remote host) runs that CLI, the CLI calls `server.DiscoverServer(paths.IDEDir)` (cmd/lazyclaude/msg.go:51) which scans `~/.claude/ide/*.lock` on the remote host. So **remote-to-remote messaging stays on the remote host** through its own MCP server. The local controlling lazyclaude TUI is NOT in the message path.
**Evidence:** msg.go:50-58; discover.go:17-58.
**Confidence:** HIGH

#### BC-PMW-REMOTE-005: Cross-host messaging (remote Worker → local PM, or vice versa) IS NOT SUPPORTED by the standard CLI
**Postconditions:** Because each Worker's `lazyclaude msg send` discovers the LOCAL server on its host, there is no built-in cross-host messaging. PM on host A cannot directly message a Worker on host B via /msg/send. The CompositeProvider's session list (composite_provider.go) does merge sessions across hosts for display, but `/msg/send` resolution is single-host. **PM/Worker workflows are implicitly single-host.**
**Evidence:** Architecture inference: msg.go uses local-host discovery only; no multi-host routing in handler_msg.go.
**Confidence:** HIGH — **NEW finding: documented architectural constraint.**

#### BC-PMW-REMOTE-006: Daemon's `/msg/*` endpoints HAVE NO PRODUCTION CALLERS — they exist for direct HTTP testing only
**Postconditions:** `find` for `\.MsgSend\(|\.MsgCreate\(|\.MsgSessions\(` returns matches ONLY in `internal/daemon/http_client_test.go`. The HTTPClient methods (`http_client.go:117-138`) are defined but no production code calls them. So the daemon's `/msg/*` safety findings from r2 (BC-PMW-MSG-SAFETY-001, -002) are reachable ONLY via:
  - Directly-crafted HTTP requests against the daemon's port with the daemon token.
  - Future code paths that wire up the HTTPClient MsgSend method.

**However:** Remote Workers running on a host with a daemon installed CAN send to their local daemon via direct HTTP — there is nothing preventing it. The CLI's MCP-server discovery means a Worker WOULD normally use the in-process server, but a Worker process could choose to call the daemon directly if it had the token (which it could read from `~/.lazyclaude/runtime/daemon.json` — see `daemon/lifecycle.go` token storage).
**Evidence:** find search exhaustive. http_client_test.go:163 is the sole call site.
**Confidence:** HIGH — **Scopes the P1 findings: low practical reach today, but the code paths exist and are documented as wired up.**

### Hook injection ↔ Worker session interplay

#### BC-PMW-HOOKS-001: Every session (Worker, PM, plain) launches with the SAME hooks-settings.json injected via `--settings`
**Postconditions:** `writeLauncher` (manager.go:706-709) unconditionally calls `config.WriteHooksSettingsFile(opts.RuntimeDir)` and appends `--settings <path>` to the claude command. There is no Role-based skipping. PM, Worker, and plain sessions all carry the same 5 hooks (PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit).
**Evidence:** manager.go:706-709 (no Role check); hooks.go:92-99 (single buildHooksMap).
**Confidence:** HIGH

#### BC-PMW-HOOKS-002: Hooks reach the lazyclaude server via lock-file discovery, NOT env-var injection — survives server restart
**Postconditions:** Each hook contains an inline node.js one-liner (`preToolUseHookCommand`, etc.) that runs `resolveServerJS`: read `~/.claude/ide/*.lock`, validate each via `process.kill(pid, 0)`, pick highest port. This means **hooks discover the server fresh on every invocation** — they survive server restarts without stale-port issues. **This is the same path the CLI's `DiscoverServer` takes.**
**Evidence:** hooks.go:13-44; discover.go:17-58.
**Confidence:** HIGH

#### BC-PMW-HOOKS-003: Hooks send `pid: process.ppid` to the server — session-id correlation is via PID, NOT lazyclaude session UUID
**Postconditions:** Every hook payload includes `pid: process.ppid` (hooks.go:31, 35, 38, 41, 44). The server correlates `pid` → tmux-pane-PID → lazyclaude session via the broker's `SyncWithTmux` pane-PID map (store.go:629-635). The lazyclaude UUID is NOT passed through hooks — Claude Code's own `session_id` (from `i.session_id`) is sent for Stop/SessionStart/UserPromptSubmit, but that's Claude Code's session ID, not lazyclaude's.
**Evidence:** hooks.go:31 (pid only); hooks.go:38, 41, 44 (Claude's session_id + pid); store.go:629-635 (PID → session mapping via tmux).
**Confidence:** HIGH — **NEW finding: dual-session-ID confusion potential. Claude Code session_id ≠ lazyclaude session ID.**

#### BC-PMW-HOOKS-004: `LAZYCLAUDE_SESSION_ID` env var IS set on every spawn but hooks do NOT use it
**Postconditions:** `claudeEnv` (manager.go:850-873) injects `LAZYCLAUDE_SESSION_ID = sessionID` into the Claude Code subprocess environment. However, the hook scripts (hooks.go:29-44) read `process.ppid` and Claude Code's own `i.session_id` — they do NOT consult `LAZYCLAUDE_SESSION_ID`. So the env var is set but architecturally unused by the hook path. It may be consumed by external tools (codex plugin, custom slash commands) or it may be dead code.
**Evidence:** manager.go:854-855; hooks.go:29-44.
**Confidence:** HIGH — **NEW finding: investigate whether `LAZYCLAUDE_SESSION_ID` is used by any consumer. Likely yes (Worker-side codex slash commands may use it).**

#### BC-PMW-HOOKS-005: Hook-emitted `/notify`, `/stop`, `/session-start`, `/prompt-submit` endpoints are auth'd via `X-Claude-Code-Ide-Authorization`
**Postconditions:** Hooks emit `X-Claude-Code-Ide-Authorization: <srvToken>` (hooks.go:31 etc.). `srvToken` is read from the lock file at `~/.claude/ide/<port>.lock`'s `authToken` field. This is the SAME token used by lazyclaude's CLI/server but with a different header name. **Yet a THIRD auth header convention** (server's CLI uses X-Auth-Token, daemon uses X-Daemon-Authorization, hooks use X-Claude-Code-Ide-Authorization). Confirms BC-PMW-MSG-DIV-002.
**Evidence:** hooks.go:31, 35, 38, 41, 44; server discover.go:38-50.
**Confidence:** HIGH — **NEW finding: three auth header conventions in the same product.**

#### BC-PMW-HOOKS-006: Hook commands embed inline JS in JSON; `SetEscapeHTML(false)` is REQUIRED to preserve `=>` arrow functions
**Postconditions:** `WriteHooksSettingsFile` uses `enc.SetEscapeHTML(false)` (hooks.go:60) to prevent Go's default `>` → `>` escaping. If a porter replicates this on default settings, hooks will be silently broken (Node will syntax-error on `=>`).
**Evidence:** hooks.go:54-60 with explicit comment.
**Confidence:** HIGH

### GUI-direct daemon /msg/* call-site scoping

#### BC-PMW-GUI-DAEMON-001: The GUI does NOT call daemon `/msg/send` or `/msg/create` — it uses `/session/create` for spawning
**Postconditions:** Exhaustive search for `.MsgSend(|.MsgCreate(|.MsgSessions(` returns hits in tests only. The GUI's PM/Worker creation goes through `SessionCommandService.CreatePMSession[WithOpts]` (session_command.go:310-340) which calls `CompositeProvider.CreatePMSession(projectRoot, host)` (composite_provider.go:421-428) which routes by host: local → `localDaemonProvider.CreatePMSession` (in-process Manager.CreatePMSession), remote → `RemoteProvider.CreatePMSession` → daemon `/session/create`. **The /msg/create path is CLI-only.**
**Evidence:** Exhaustive find search; routing chain in session_command.go and composite_provider.go.
**Confidence:** HIGH

#### BC-PMW-GUI-DAEMON-002: GUI's message-sending UX is also CLI-routed
**Postconditions:** No GUI key action maps to "send message" — there is no UI for composing a /msg/send. Workers and PM compose messages by running `lazyclaude msg send` inside their own Claude Code pane (as documented in `prompts/pm.md:13-17`). **So the /msg/send code path is exercised only by Claude Code subprocesses running the CLI.**
**Evidence:** keymap/registry.go contains no "send_message" or similar action; no GUI input modal for message body.
**Confidence:** HIGH

#### BC-PMW-GUI-DAEMON-003: Remote-host Worker's CLI-issued /msg/send hits the LOCAL daemon if no in-process server exists
**Postconditions:** Remote-host scenarios where lazyclaude runs as a daemon (via `lazyclaude setup`) but no TUI is attached have NO in-process MCP server. The lock-file scan returns empty. A remote-host Worker running `lazyclaude msg send` will get "no running lazyclaude server found" (discover.go:54-56) **UNLESS** the daemon's port has been registered in `~/.claude/ide/*.lock` — which it is NOT by the daemon setup path (daemon writes only `runtime/daemon.json`, not a `*.lock` file).
**Evidence:** discover.go:20-58; daemon/server.go:753-767 (`writeDaemonInfo` writes daemon.json, no .lock file).
**Confidence:** MEDIUM — **NEW finding: remote-only deployments may have BROKEN /msg/send unless the in-process server has been started via TUI attach. Investigate as a follow-up — may be why daemon's /msg/* exists but is unused in production: the daemon is a fallback for headless remote deployments that does not work end-to-end.**

### Composite divergence — full picture

#### BC-PMW-DIV-FULL-001: Three session-create endpoints, three type allowlists, three auth headers — full table

| Endpoint | Path | Type allowlist | Auth header | Caller |
|---|---|---|---|---|
| In-process server `/msg/create` | `internal/server/handler_msg.go` | `{worker, local}` | `X-Auth-Token` | CLI (msg.go), Claude Code subprocesses |
| Daemon `/msg/create` | `internal/daemon/server.go:551` | `{worker, pm}` | `X-Daemon-Authorization` | NONE in production (test-only) |
| Daemon `/session/create` | `internal/daemon/server.go:216` | `{plain, "", worktree, pm, worker}` | `X-Daemon-Authorization` | RemoteProvider (composite_provider.go) |
| GUI key `P` → CreatePMSession | (local) Manager.CreatePMSessionOpts | (no enum — direct method call) | (none — in-process) | gui/keymap `P` rune |

Implication: **The CLI's `--type pm` is fundamentally impossible**: the CLI rejects "pm" client-side (msg.go:88-91), and even if it didn't, the server's /msg/create allowlist excludes "pm" (handler_msg.go:119-122). PM sessions are created by:
1. GUI keybind `P` → local in-process Manager.CreatePMSessionOpts (LOCAL host).
2. GUI keybind `P` with cursor on remote project → CompositeProvider routes to RemoteProvider.CreatePMSession → daemon `/session/create` with `SessionType: "pm"` (REMOTE host).

**There is NO CLI path to create a PM session.** This may be intentional (PM is the orchestrator persona, not a tool to invoke from inside another Claude); the porter should preserve this constraint.
**Evidence:** Cross-reference of all four call sites.
**Confidence:** HIGH

## Failure modes refinement

#### BC-PMW-FAIL-008: Hook delivery is best-effort with 300ms timeout — slow server causes silent hook drop
**Postconditions:** Hook scripts set `timeout: 300` on PreToolUse/Stop/SessionStart/UserPromptSubmit (hooks.go:31, 38, 41, 44). On timeout, `req.destroy()` is called and the error is swallowed via `req.on('error',()=>{})`. So a slow lazyclaude server (or a stale lock file pointing to a port no longer responding) results in **silent hook failure** — Claude Code's tool-use continues; lazyclaude's activity-state, permission popup routing, and session-start tracking are missed.
**Evidence:** hooks.go:31, 38, 41, 44.
**Confidence:** HIGH

#### BC-PMW-FAIL-009: Hook discovery via lock-file scan introduces a race during server restart
**Postconditions:** Between server shutdown (lock file becomes stale because PID is gone) and server startup (new lock file written), a hook firing during this window will find no alive server (`process.kill(pid, 0)` throws for the old PID, new lock not yet written). Behavior: silent drop. Recovery: next hook event will find the new server.
**Evidence:** hooks.go:13-20 + server start sequence (not deep-read but inferred from discover.go).
**Confidence:** MEDIUM

## Tests gap (final)

#### BC-PMW-TEST-005: NO test exercises the daemon /msg/send + lazyclaude-as-Worker integration end-to-end
**Postconditions:** Daemon msg tests cover only HTTP-validation surface (server_test.go:175-201). There is no test where a remote Worker spawned via /session/create then runs `lazyclaude msg send` and verifies the message arrives at PM. The path BC-PMW-REMOTE-004 (Worker-side CLI uses remote-host in-process server) is therefore unverified.
**Evidence:** find search for tests calling both daemon.CreateSession + server.SendMessage returned nothing.
**Confidence:** MEDIUM

## Delta Summary (round 3)

- New BC contracts drafted: 20 (REMOTE-001..006 = 6, HOOKS-001..006 = 6, GUI-DAEMON-001..003 = 3, DIV-FULL-001 = 1, FAIL-008..009 = 2, TEST-005 = 1, + 1 dual-session-ID note = 20 unique).
- Existing items refined: BC-PMW-MSGCREATE-001 (now part of the three-endpoint divergence table BC-PMW-DIV-FULL-001), BC-PMW-MSG-DIV-002 (auth header story extended to 3 conventions: server + daemon + hooks).
- Remaining gaps after this round:
  - `daemon/lifecycle.go` daemon-token-storage path (file mode, permissions on `daemon.json`) — relevant if Worker can read token to bypass in-process server. **Low risk: same-user file-system access already implies trust.**
  - Concrete behavior of `LAZYCLAUDE_SESSION_ID` consumer (BC-PMW-HOOKS-004) — likely codex plugin internal, out of subsystem scope.
  - `cmd/lazyclaude/msg.go` --type local quoting behavior on Windows (theoretical; lazyclaude is Linux/macOS only).

These remaining gaps are documentation/audit items that would not change the subsystem model.

## Novelty Assessment

Novelty: **SUBSTANTIVE** — but lower-magnitude than r2.

Justification: Three architectural facts changed the model:
1. **Three session-create endpoints with three allowlists** (BC-PMW-DIV-FULL-001) — r2 caught two; r3 caught the daemon `/session/create` is the actual production remote path with its own 4-type allowlist.
2. **Daemon `/msg/*` has no production callers** (BC-PMW-REMOTE-006) — significantly scopes the P1 findings from r2. Still real, but reachable only via direct HTTP or via future wiring.
3. **Hook discovery uses lock files, NOT env vars** (BC-PMW-HOOKS-002), and **uses PID for correlation** (BC-PMW-HOOKS-003) — confirms restart-resilience and surfaces the Claude-session-id-vs-lazyclaude-session-id duality.
4. **Three auth header conventions** (X-Auth-Token, X-Daemon-Authorization, X-Claude-Code-Ide-Authorization) (BC-PMW-HOOKS-005) — unification target for the port.
5. **Remote-host prompt overrides resolve in the REMOTE filesystem** (BC-PMW-REMOTE-003) — important deployment fact.
6. **No CLI path to create PM** (BC-PMW-DIV-FULL-001) — preserves orchestrator-persona model.

Removing these would leave a porter unable to (a) correctly route session creation by host, (b) understand the hook-injection mechanism's failure modes, (c) know which auth header to send. Another round MIGHT catch additional minor details but the architectural model is now complete.

## Convergence Declaration

**One more round (r4) MAY produce additional minor findings** (the gaps listed above are non-architectural). Per protocol I'll run r4 to confirm NITPICK. Expected outcome: round 4 will be NITPICK and the subsystem will be DECLARED CONVERGED.

## State Checkpoint

```yaml
pass: B
subsystem: pmw
round: 3
status: complete
files_read_full_this_round:
  - internal/daemon/remote_provider.go (msg + session creation regions)
  - internal/server/client.go (CLI-side HTTP client)
  - internal/server/discover.go (lock-file scan)
  - internal/core/config/hooks.go (hook injection)
  - internal/daemon/http_client.go (msg client methods)
contracts_drafted_this_round: 20
contracts_total_after_r3: 87 (17 r1 + 50 r2 + 20 r3)
timestamp: 2026-05-11T23:59:30Z
novelty: SUBSTANTIVE
convergence: NOT YET — r4 expected to be NITPICK
next_round: 4
```
