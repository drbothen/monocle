# Pass B Deepening — `internal/server/` Round 1

**Subsystem:** `internal/server/` (MCP server — WebSocket + HTTP hook + msg bus)
**Path:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/`
**Coverage state at entry:** Pass A pass-3 drafted BC-MCPSRV-001..020 (20 contracts). No prior Pass B round existed. Pass 0/1 listed surface details only.
**Round:** 1 (structural pass + behavioral gap fill)
**Production LOC:** 2,262 (sum of non-test files); **Test LOC:** 3,263; **test density:** 144%.

This deepening confirms, refines, and extends those 20 contracts and adds **BC-MCPSRV-021..045** (25 new contracts), plus calls out **2 P1 findings**, **1 P2 finding**, and a security note. It also clarifies the boundary between `internal/server/` and `internal/daemon/` (which is its sibling, not its parent).

---

## 1. Subsystem Identity & Relationship to `internal/daemon/`

`internal/server/` is the **Claude Code IDE MCP server** — a WebSocket + HTTP server on 127.0.0.1:`<random-port>` that serves three distinct surfaces from one `http.ServeMux` (`internal/server/server.go:107-117`):

1. **MCP JSON-RPC 2.0 over WebSocket** at path `/` (`server.go:116`, `serveConn` at `server.go:302-335`). Handles `initialize`, `notifications/initialized`, `ide_connected`, `openDiff` (`handler.go:40-58`).
2. **Claude Code hook HTTP endpoints**: `/notify`, `/stop`, `/session-start`, `/prompt-submit` (`server.go:108-111`). Posted by node-eval hook one-liners injected via `claude --settings`.
3. **PM/Worker + CLI msg bus** at `/msg/send`, `/msg/create`, `/msg/resume`, `/msg/sessions` (`server.go:112-115`). Consumed by `cmd/lazyclaude/{sessions,msg}.go` and (push-side) by the `claude` agent itself.

It is **NOT** the remote daemon HTTP+SSE server. The remote daemon (`internal/daemon/`) is a *separate* HTTP server on its own random port that runs **on top of** the MCP server: in daemon mode, `lazyclaude daemon` spawns its own in-process MCP server via the same `server.New` factory (`cmd/lazyclaude/daemon_cmd.go:67`), so hook discovery is identical on remote hosts. The two share `event.Broker[model.Event]` via `WithBroker` injection — that's the load-bearing seam.

**Architectural placement** (verified): the package is a **leaf** below `cmd/lazyclaude` and `internal/daemon`. Its only intra-`internal/` imports are `internal/adapter/tmuxadapter`, `internal/core/event`, `internal/core/model`, `internal/core/tmux`, `internal/notify` (`server.go:18-22`, `handler_msg.go:11-13`). It does not depend on `session`, `daemon`, `gui`, `mcp`, `plugin`, or `profile`. The `SessionLister`/`SessionCreator` interfaces (`handler_msg.go:13-31`) are the dependency-inversion seam.

---

## 2. File Manifest & LOC Recount

Counted via `find ... -exec wc -l {} +`:

| File | LOC | Role |
|---|---|---|
| `server.go` | 727 | `Server` type, lifecycle, hook handlers `/notify` `/stop` `/session-start` `/prompt-submit`, WebSocket accept, `dispatchToolNotification`. |
| `handler_msg.go` | 511 | `/msg/{create,send,resume,sessions}` HTTP handlers + `SessionLister`/`SessionCreator` interfaces + `state.json` fallback. |
| `handler.go` | 188 | JSON-RPC `Handler` for `initialize`, `ide_connected`, `openDiff`; PID → window resolution. |
| `state.go` | 182 | `State` (connections + PID→window + pending FIFO + diffChoices) with `sync.RWMutex`. |
| `lock.go` | 182 | `LockManager` (write/read/exists/remove/CleanStale/CleanAllExcept). |
| `client.go` | 175 | HTTP `Client` (`Sessions`, `SendMessage`, `CreateSession`, `ResumeSession`). Used by CLI subcommands. |
| `ensure.go` | 169 | Out-of-process server lifecycle: `EnsureServer`, `RestartServer`, `StopDaemon`, `IsAlive`. |
| `jsonrpc.go` | 70 | JSON-RPC 2.0 `Request`/`Response`/`RPCError` + `ParseRequest`/`MarshalResponse`. |
| `discover.go` | 58 | `DiscoverServer(ideDir)` — highest-port-alive-lazyclaude scan. |
| **subtotal** | **2,262** | (production) |
| `server_test.go` | 748 | Hook endpoint + WS integration tests. |
| `handler_msg_test.go` | 672 | `/msg/*` integration tests. |
| `server_broker_test.go` | 633 | Broker dispatch, Write/Edit diff fields, `WithBroker` semantics. |
| `lock_test.go` | 254 | `LockManager` unit tests. |
| `handler_test.go` | 255 | JSON-RPC handler unit tests. |
| `state_test.go` | 232 | `State` unit tests + concurrent access. |
| `ensure_test.go` | 168 | `EnsureServer`/`RestartServer` lifecycle tests. |
| `discover_test.go` | 176 | `DiscoverServer` tests (alive, dead, non-lazyclaude, highest-port, empty). |
| `jsonrpc_test.go` | 114 | JSON-RPC parse/marshal tests. |
| `main_test.go` | 11 | `goleak.VerifyTestMain` package-level. |
| **subtotal** | **3,263** | (tests) |
| **total** | **5,525** | — |

Numbers match Pass 0 (5,525 total) and Pass 8 (5,525 total, 2,262 production) exactly.

---

## 3. Public Surface

Exported identifiers (from package `server`):

| Identifier | Kind | File:line | Notes |
|---|---|---|---|
| `Config` | struct | server.go:27-33 | `Port`, `Token`, `IDEDir`, `PortFile`, `RuntimeDir`. |
| `Server` | struct | server.go:42-64 | The MCP server. |
| `ServerOption` | func type | server.go:67 | Functional option pattern. |
| `WithBroker` | func | server.go:73-78 | External broker injection (BC-MCPSRV-014, BC-MCPSRV-038 below). |
| `New` | func | server.go:81-120 | Factory. Creates default broker if `WithBroker` not used. |
| `(*Server).Start` | method | server.go:123-158 | Binds `127.0.0.1:port`, writes lock + port file, serves in goroutine. |
| `(*Server).Stop` | method | server.go:161-183 | Idempotent. Closes broker only if owned. |
| `(*Server).Port` | method | server.go:186-188 | Returns actual port (resolved from port=0). |
| `(*Server).State` | method | server.go:191-193 | Exposed for tests. |
| `(*Server).RuntimeDir` | method | server.go:196-198 | Returns config.RuntimeDir. |
| `(*Server).SetSessionLister` | method | server.go:203-207 | Adapter injection (post-construction). |
| `(*Server).SetSessionCreator` | method | server.go:211-215 | Adapter injection (post-construction). |
| `(*Server).NotifyBroker` | method | server.go:221-223 | Returns broker (owned or injected). |
| `(*Server).WindowActivity` | method | server.go:248-256 | `(state, toolName)` for a tmux window. |
| `State` | struct | state.go:29-35 | Shared mutable state. |
| `NewState` | func | state.go:38-45 | Factory. |
| `ConnState` | struct | state.go:9-12 | PID + Window. |
| `PendingTool` | struct | state.go:15-20 | ToolName + Input + CWD + Expiry. |
| `(*State).SetConn`, `GetConn`, `RemoveConn`, `WindowForPID`, `SetPending`, `GetPending`, `SetPendingWithExpiry`, `SetDiffChoice`, `SetDiffChoiceWithExpiry`, `GetDiffChoice`, `LastPendingWindow`, `ConnCount` | methods | state.go:48-183 | All RWMutex-protected. |
| `Handler` | struct | handler.go:17-22 | JSON-RPC handler. |
| `NewHandler` | func | handler.go:25-31 | |
| `(*Handler).SetRuntimeDir` | method | handler.go:34-36 | For `openDiff` notification writes. |
| `(*Handler).HandleMessage` | method | handler.go:40-58 | Dispatch by method name. |
| `Request`, `Response`, `RPCError` | structs | jsonrpc.go:11-30 | JSON-RPC 2.0 wire types. |
| `(*Request).IsNotification` | method | jsonrpc.go:33-35 | `id == nil \|\| string(id) == "null"`. |
| `NewResponse`, `NewErrorResponse`, `ParseRequest`, `MarshalResponse` | funcs | jsonrpc.go:38-71 | Wire helpers. |
| `LockFile` | struct | lock.go:18-23 | `PID`, `AuthToken`, `Transport`, `App`. |
| `LockManager` | struct | lock.go:28-30 | `ideDir` only. |
| `NewLockManager`, `(*LockManager).{Write,Read,Remove,Exists,CleanStale,CleanAllExcept}` | funcs/methods | lock.go:34-179 | See Section 6. |
| `DiscoverResult`, `DiscoverServer` | struct + func | discover.go:12-58 | Hooks-equivalent server discovery from Go callers. |
| `EnsureOpts`, `EnsureResult`, `EnsureServer`, `RestartServer`, `StopDaemon`, `IsAlive` | structs + funcs | ensure.go:24-159 | Out-of-process server lifecycle (used by `lazyclaude setup`). |
| `Client`, `NewClient`, `(*Client).{Sessions,SendMessage,CreateSession,ResumeSession}` | struct + funcs | client.go:19-166 | HTTP client. |
| `MsgCreateResponse`, `MsgCreateSession`, `MsgResumeResponse`, `SessionInfo`, `SessionProjectInfo`, `SessionCreateResult` | structs | handler_msg.go:34-73, 47-61, 305-310 | Wire types. |
| `SessionLister` | interface | handler_msg.go:13-16 | One method: `Sessions() []SessionInfo`. |
| `SessionCreator` | interface | handler_msg.go:19-31 | Four methods. |

**Notable hidden seams:**

- `Server.activityMap` (`server.go:56`) is *only* updated by hook handlers, *only* read by `/msg/sessions` enrichment and `WindowActivity` accessor. It is intentionally NOT updated via broker subscription — see `server.go:225-235` comment: "Called directly by hook handlers (not via broker subscription) to avoid inflating `HasSubscribers()` and altering broker-vs-file dispatch logic." This is a non-obvious load-bearing invariant.
- `Server.ownsBroker` (`server.go:50`) is the singular determinant of whether `Stop()` closes the broker (`server.go:178-180`).
- `Server.shutdown` flag (`server.go:63`) makes `Stop` idempotent (`server.go:162-168`).

---

## 4. New / Strengthened Behavioral Contracts

The 20 contracts in Pass 3 (`BC-MCPSRV-001..020`) are confirmed. The following add detail not previously extracted.

### Lifecycle & Listening

#### BC-MCPSRV-021: Port=0 yields random OS-assigned port; `(*Server).Port()` reflects it after `Start`
**Preconditions:** `Config.Port = 0`, `New(cfg, ...)` then `Start(ctx)`.
**Postconditions:** `Start` returns the resolved port; `(*Server).Port()` returns the same value; `Config.Port` is mutated in place (`server.go:131-132`).
**Evidence:** server.go:124-132, server_test.go:35-39, 52-53 (`TestServer_StartAndStop`).
**Confidence:** HIGH

#### BC-MCPSRV-022: `Start` cleans stale lock files before writing its own
**Postconditions:** Before `lock.Write(port, token)`, `lock.CleanStale()` is invoked; count logged via `s.log.Printf`.
**Evidence:** server.go:134-137; CleanStale defined at lock.go:86-134; tested in lock_test.go:71-126 (`TestLockManager_CleanStale_*`).
**Confidence:** HIGH

#### BC-MCPSRV-023: Lock-file write failure aborts Start and closes the listener
**Postconditions:** If `lock.Write` returns error, `Start` calls `ln.Close()` and returns `fmt.Errorf("write lock: %w", err)`.
**Evidence:** server.go:140-143.
**Confidence:** HIGH (explicit branch)

#### BC-MCPSRV-024: Port file write failure is logged but does not abort Start
**Postconditions:** `s.writePortFile(port)` errors are warned via `s.log.Printf("warning: write port file: %v", err)`; server continues to serve.
**Evidence:** server.go:146-148; `writePortFile` at server.go:577-582 (no-op when `PortFile==""`).
**Confidence:** HIGH

#### BC-MCPSRV-025: Stop is idempotent; second call is a no-op
**Postconditions:** A `shutdown` flag is set under `mu`; second invocation observes `s.shutdown == true` and returns nil immediately.
**Evidence:** server.go:162-168, server_test.go:55-60 (`TestServer_StartAndStop` double-stop).
**Confidence:** HIGH

#### BC-MCPSRV-026: Stop removes the lock file (best-effort; logs warning on failure)
**Evidence:** server.go:170-173.
**Confidence:** HIGH

#### BC-MCPSRV-027: Default broker is closed on Stop; injected broker is NOT
**Postconditions:** `s.ownsBroker == true` (set when `WithBroker` was not used; `server.go:104`) implies `s.notifyBroker.Close()` is called; `ownsBroker == false` (set by `WithBroker`; `server.go:76`) implies the broker is left open so it survives server restart (load-bearing for "GUI subscriptions survive server restart").
**Evidence:** server.go:178-180, server_broker_test.go:279-340 (`TestServer_WithBroker_StopDoesNotCloseBroker`, `TestServer_DefaultBroker_StopClosesBroker`).
**Confidence:** HIGH

---

### Authentication

#### BC-MCPSRV-028: All hook endpoints use the same auth helper `extractAuthToken` — token taken from `X-Claude-Code-Ide-Authorization` OR `X-Auth-Token` (in that priority); compared with `subtle.ConstantTimeCompare`
**Preconditions:** POST to `/notify`, `/stop`, `/session-start`, `/prompt-submit`, `/msg/send`, `/msg/create`, `/msg/resume`, `/msg/sessions`, or WebSocket `GET /`.
**Postconditions:** When neither header matches, returns `401 Unauthorized` with body `"unauthorized\n"` (via `http.Error`); never accepts via URL query (no query parsing exists).
**Evidence:**
- `extractAuthToken` at server.go:358-363.
- `subtle.ConstantTimeCompare` invocations at server.go:278, 372, 597, 644, 690, and handler_msg.go:93, 197, 327, 401.
- Tests: server_test.go:279-292 (`TestServer_Notify_Unauthorized`), 389-403 (`TestServer_Stop_Unauthorized`), 434-448, 525-539; handler_msg_test.go:137-146 (`TestMsgCreate_missing_auth`), 374-389 (`TestMsgSend_missing_auth`, `TestMsgSend_wrong_token`), 648-655 (`TestMsgSessions_missing_auth`).
**Confidence:** HIGH (subsumes Pass 3 BC-MCPSRV-002)

#### BC-MCPSRV-029: WebSocket origin restricted to `localhost:*` and `127.0.0.1:*`
**Postconditions:** `websocket.AcceptOptions{OriginPatterns: []string{"localhost:*", "127.0.0.1:*"}}` rejects cross-origin upgrades.
**Evidence:** server.go:283-285.
**Confidence:** HIGH

---

### Hook Endpoint Method Semantics

Each of `/notify`, `/stop`, `/session-start`, `/prompt-submit` enforces `POST`-only, rejects non-POST with `405 Method Not Allowed`. `/msg/send`, `/msg/create`, `/msg/resume` enforce POST-only. `/msg/sessions` is GET-only.

#### BC-MCPSRV-030: Body cap on all POST endpoints is 1 MB via `http.MaxBytesReader`
**Evidence:**
- server.go:377 (/notify), 602 (/stop), 649 (/session-start), 695 (/prompt-submit)
- handler_msg.go:107 (/msg/create), 202 (/msg/send), 342 (/msg/resume)
**Confidence:** HIGH (explicit constant `1<<20`)

#### BC-MCPSRV-031: All four hook endpoints share the same PID→window resolution chain, with a fallback to `LastPendingWindow`
**Preconditions:** Valid POST with `pid > 0`.
**Postconditions:** Resolution order:
  1. `s.state.WindowForPID(pid)` (cache from earlier `ide_connected`, prior hook, or `SetConn`).
  2. `tmux.FindWindowForPid(ctx, s.tmux, pid)` (pidwalk).
  3. For non-`tool_info` types only: `s.state.LastPendingWindow()` (the window with the most recent un-expired pending tool entry).
  4. Else: respond `404 window not found` (for `/notify` and `/prompt-submit`) — note `/stop` and `/session-start` accept empty window and still publish (see BC-MCPSRV-034).
**Evidence:** `resolveNotifyWindow` at server.go:458-468; fallback at server.go:389-401, 609-612, 657-659, 702-705.
**Confidence:** HIGH (source); MEDIUM (tested partial — `LastPendingWindow` fallback is exercised via two-phase test in server_test.go:218-255 but not directly asserted as the resolution source).

#### BC-MCPSRV-032: Each hook invocation upserts a `hook-<pid>` connection cache entry
**Postconditions:** After window resolution, `s.state.SetConn(fmt.Sprintf("hook-%d", req.PID), &ConnState{PID, Window})`. The fixed `hook-` prefix is deliberate: "so all hook types share one entry per PID, avoiding unbounded accumulation" (server.go:403-405).
**Evidence:** server.go:404-405 (/notify), 613-615 (/stop), 660-662 (/session-start), 711-713 (/prompt-submit).
**Confidence:** HIGH (subsumes BC-MCPSRV-005 mechanism)

#### BC-MCPSRV-033: `/notify` with `tool_name` empty AND no pending entry is silently dropped (logged as `DROPPED`)
**Preconditions:** Permission_prompt phase (Type == ""); window resolved; no pending PreToolUse data; `req.ToolName == ""`.
**Postconditions:** No broker publish, no `notify.Enqueue`; logs `notify: DROPPED — empty toolName for window %s pid=%d (no pending and no tool_name in request)`. Returns 200 OK regardless.
**Evidence:** server.go:442-447. Not directly tested.
**Confidence:** MEDIUM (source-derived; **gap**: no test verifies the drop path).

#### BC-MCPSRV-034: `/stop` and `/session-start` accept empty window — they publish broker events with `Window == ""`
**Preconditions:** PID not resolvable, no pending fallback.
**Postconditions:**
  - `/stop`: log `stop: pid=%d window=%s reason=%s`, set activity (`setActivity("", ...)` is a no-op per server.go:228-231), publish `StopNotification{Window: "", StopReason, SessionID}`.
  - `/session-start`: same pattern with `SessionStartNotification`.
  - Contrast `/notify` and `/prompt-submit` which return `404 window not found` (server.go:398-401, 706-709).
**Evidence:** server.go:609-630 (`handleStop`), 656-676 (`handleSessionStart`). Branch at 613 `if window != "" && req.PID > 0` only short-circuits the cache write, NOT the publish. Not directly tested.
**Confidence:** MEDIUM (source-derived; **gap**: no test asserts empty-window behavior).
**Disposition note:** This is a subtle asymmetry — `/stop` and `/session-start` are "fire-and-forget" lifecycle hooks where missing window is non-fatal, while `/notify` (which triggers a popup) and `/prompt-submit` (which arms running state) require a window. Worth retaining in a port.

#### BC-MCPSRV-035: setActivity is a no-op for empty window
**Postconditions:** `if window == "" { return }` is the first line of `setActivity` (server.go:228-231). So `/stop` and `/session-start` with empty window do not pollute `activityMap`.
**Evidence:** server.go:229-231.
**Confidence:** HIGH

---

### /notify Two-Phase Flow

#### BC-MCPSRV-036: Pending tool queue is **FIFO**, **per-window**, **TTL=15s**
**Postconditions:** Multiple concurrent tools for the same window are tracked in a FIFO slice keyed by window. `SetPending` appends; `GetPending` pops the oldest non-expired entry. `pendingTTL = 15 * time.Second`.
**Evidence:** state.go:81-121, 14-20 (struct). Tests: state_test.go:65-151 (FIFO + skip-expired + TTL).
**Confidence:** HIGH (subsumes BC-MCPSRV-003; the Pass 3 contract did not capture the FIFO+TTL semantics)

#### BC-MCPSRV-037: `LastPendingWindow` returns the window of the latest-expiring un-expired pending entry
**Postconditions:** Iterates all windows and queue entries, returns the window whose newest pending entry has the highest `Expiry` (server.go:158-176 of state.go). When all entries expired or empty, returns `""`.
**Evidence:** state.go:158-176.
**Confidence:** HIGH (source-only — no direct unit test, exercised indirectly).
**Gap-VER addition:** No test in `state_test.go` covers `LastPendingWindow` (it's tested only through full-stack server tests). Recommend adding.

#### BC-MCPSRV-038: Diff-choice fast path: when `GetDiffChoice(window)` returns a stored key, `/notify` permission_prompt skips the full dispatch and sends the cached key via tmux after a 50ms delay
**Preconditions:** A prior `openDiff` popup completed and called `SetDiffChoice(window, key)` (GUI does this — not in this package). The next permission_prompt arrives for the same window before TTL (15s).
**Postconditions:** `/notify` spawns a goroutine that sleeps 50ms then calls `s.tmux.SendKeys(context.Background(), "lazyclaude:"+window, key)`. Errors logged. Skips broker publish AND notify.Enqueue.
**Evidence:** server.go:428-439. `GetDiffChoice` consumes-on-read (state.go:141-153). Not directly tested in `server_*_test.go` (the diff-choice mechanism is set by GUI code, so server-side tests don't exercise it).
**Confidence:** MEDIUM (source-derived; **gap**: no test in this package).
**Why 50ms?** Not commented. Likely allows the dialog to fully render before sending the key.

---

### dispatchToolNotification Edge Cases

#### BC-MCPSRV-039: MaxOption defaults to 3; falls back gracefully on `CapturePaneANSI` error
**Postconditions:** When `s.tmux.CapturePaneANSI(ctx, window)` errors, `maxOpt` remains 3. Otherwise `maxOpt = tmuxadapter.DetectMaxOption(content)`. Stored in `ToolNotification.MaxOption`.
**Evidence:** server.go:493-498.
**Confidence:** HIGH (explicit branch); **gap**: no test injects a CapturePaneANSI error.

#### BC-MCPSRV-040: Edit-tool diff handling rejects irregular files (FIFO, device) AND files > 2 MiB
**Postconditions:**
  - `os.Stat(absPath)` → must succeed AND `fi.Mode().IsRegular() == true` AND `fi.Size() <= 2<<20`.
  - On read failure (`os.ReadFile`), diff fields silently remain empty; the notification falls back to ToolPopup routing.
**Evidence:** server.go:540-553 (constant `maxEditFileSize = 2 << 20`).
**Confidence:** HIGH

#### BC-MCPSRV-041: HasSubscribers→Publish window has a benign race documented in code
**Postconditions:** `s.notifyBroker.HasSubscribers()` and `s.notifyBroker.Publish()` acquire separate locks; a subscriber may `Cancel()` between them. The dropped event during shutdown is acceptable per the explicit comment.
**Evidence:** server.go:557-561 (verbatim explanation).
**Confidence:** HIGH (explicit comment + by-design)

---

### /msg/send & /msg/create & /msg/resume

#### BC-MCPSRV-042: `/msg/send` Window resolution fallback: when `recipient.Window == ""`, computes `wName = "lc-" + recipient.ID[:8]` and queries `tmux.ListWindows("lazyclaude")` for a name match
**Postconditions:** Avoids dependence on status detection — even if SessionLister gave us no window, tmux can resolve the live window from the session ID.
**Evidence:** handler_msg.go:262-277. Tested partially via `TestMsgSend_PushDelivery_NoWindowForRecipient` (handler_msg_test.go:606-623) which asserts 502 when **both** `Window == ""` AND tmux yields no match.
**Confidence:** HIGH

#### BC-MCPSRV-043: `/msg/send` message text format includes both sender Name and sender ID
**Postconditions:** Template:
```
[MESSAGE from <senderName> (<fromID>)]
type: <type>
---
<body>
```
**Evidence:** handler_msg.go:284-285. Tested in handler_msg_test.go:574-603 (`TestMsgSend_PushDelivery_MessageFormat`).
**Confidence:** HIGH

#### BC-MCPSRV-044: `/msg/sessions` falls back to reading `~/.local/share/lazyclaude/state.json` directly when SessionLister is nil OR returns zero sessions
**Postconditions:**
  - When `SessionLister` not set OR returns `len(sessions) == 0`, `readSessionsFromState()` is invoked.
  - It uses `os.UserHomeDir()` and parses state.json as `[]stateSession` (handler_msg.go:432-462). On `UserHomeDir()` error, falls back to `<RuntimeDir>/../lazyclaude/state.json`.
  - For each parsed session, derives `wName = "lc-" + r.ID[:8]`, queries tmux `ListWindows("lazyclaude")` for the matching window ID, and `ListPanes("")` to determine Running/Detached/Orphan status.
**Evidence:** handler_msg.go:416-418, 443-510. Tested in `TestMsgSessions_no_lister_returns_empty` (handler_msg_test.go:657-672) which uses `t.Setenv("HOME", ...)` to isolate.
**Confidence:** HIGH

#### BC-MCPSRV-045: `/msg/resume` requires only `id`; `prompt` and `name` (worktree name) are optional fallback hints
**Postconditions:** 400 on missing id. `SessionCreator.ResumeSession` is invoked with all three; the creator is responsible for handling the GC'd-but-worktree-exists case (BC-SESSION-010).
**Evidence:** handler_msg.go:312-376. No direct unit test in this package (the test file lacks `Resume` tests entirely — see Section 8 gap).
**Confidence:** HIGH (source); **gap**: zero tests for `/msg/resume` in `handler_msg_test.go`.

---

## 5. Concurrency Model

The package has **three independent mutex domains**:

| Domain | Mutex | Scope |
|---|---|---|
| `Server.mu` | `sync.RWMutex` | `shutdown` flag + `sessionLister` + `sessionCreator` slots. |
| `Server.activityMu` | `sync.RWMutex` | `activityMap` only (per-window activity for `/msg/sessions` enrichment). |
| `State.mu` | `sync.RWMutex` | `connections`, `pidToWindow`, `pending`, `diffChoices` (single mutex across all four — see state.go:30). |

Plus the **broker's own** internal mutex (`internal/core/event`) which is independent.

### Goroutines spawned by this package

| Site | Goroutine | Lifecycle |
|---|---|---|
| `server.go:150-154` | `httpSrv.Serve(ln)` | Started in `Start`; ends on `httpSrv.Shutdown` in `Stop`. |
| `server.go:431-438` | Diff-choice key sender (50ms sleep + SendKeys) | Fire-and-forget per `/notify` invocation when a cached choice exists. Uses `context.Background()` — no cancellation. **Goroutine leak risk under shutdown:** if Stop happens before the 50ms sleep ends, the goroutine outlives the server (uses `context.Background()`). Bound is 50ms + tmux send time, so leak window is short, but technically a leak. |

The `goleak.VerifyTestMain` (main_test.go:9-11) verifies no leaks at the end of `go test`, but tests don't appear to exercise the diff-choice fast path (BC-MCPSRV-038 gap). If they did and ran without `t.Cleanup(...sub.Cancel())`, goleak would flag it.

### Mutex ordering

There is no nested locking observed:
- `Server.mu` is taken only in `SetSessionLister`/`SetSessionCreator`/`Stop` outermost.
- `Server.activityMu` is independent.
- `State.mu` is internal to state methods.

The only cross-mutex concern is `enrichWithActivity` (server.go:259-273) which takes `activityMu.RLock()` while iterating a slice — no risk.

### Race-test coverage

- `state_test.go:209-232` (`TestState_ConcurrentAccess`) runs 100 writes vs 100 reads under `-race`.
- The full `go test -race ./internal/...` (Makefile:14-15) catches any other races.
- `goleak` catches goroutine leaks.

---

## 6. Lock Manager Deep Dive

`LockManager` (lock.go:28-30) is the **only state shared across server instances and external IDE binaries**. Its semantics are load-bearing.

### Lock file schema

```json
{
  "pid": 12345,
  "authToken": "<hex>",
  "transport": "ws",
  "app": "lazyclaude"
}
```

The `App` field is the **post-#33** addition; legacy locks (binaries before #33) have no `app` field. The compat path is at lock.go:162-167 and 23-24.

### CleanStale vs CleanAllExcept

| Method | When called | Behavior |
|---|---|---|
| `CleanStale()` | `Server.Start` (server.go:135) | Removes locks where either PID is dead OR TCP port is dead. Returns removed count. Used to clean up after crashes. |
| `CleanAllExcept(exceptPort)` | Per Pass 3 BC-MCPSRV-010, called from `cmd/lazyclaude/root.go:440-442` (TUI startup) | Removes lazyclaude-owned locks (`App=="lazyclaude"` or `App==""` legacy) other than `exceptPort`. **Sends SIGINT to the lock's PID** for graceful shutdown (lock.go:170-173). Skips own PID (lock.go:170: `lock.PID != os.Getpid()`). Returns removed count. |

### BC-MCPSRV-046: CleanAllExcept respects non-lazyclaude locks (VS Code, JetBrains) — they survive
**Postconditions:** When `lock.App != ""` AND `lock.App != "lazyclaude"`, the lock is left untouched (lock.go:162-167).
**Evidence:** lock_test.go:137-182 (`TestLockManager_CleanAllExcept_RemovesLazyclaudeLocks`).
**Confidence:** HIGH (test asserts vscode lock retained)

### BC-MCPSRV-047: CleanAllExcept treats `App==""` (legacy) as lazyclaude-owned and removes it
**Postconditions:** Legacy locks created before #33 are removed by `CleanAllExcept`.
**Evidence:** lock_test.go:184-229 (`TestLockManager_CleanAllExcept_RemovesLegacyLocks`).
**Confidence:** HIGH

### BC-MCPSRV-048: CleanStale uses **AND** semantics: lock removed if EITHER PID is dead OR port is dead
**Postconditions:** Both `pidAlive` (via `proc.Signal(syscall.Signal(0))`) and `tcpAlive` (via DialTimeout 500ms) must be true to keep the lock (lock.go:128-131: `if !pidAlive || !tcpAlive`).
**Evidence:** lock_test.go:71-126 (`TestLockManager_CleanStale_RemovesDeadPID`, `TestLockManager_CleanStale_RemovesPIDAliveButPortDead`).
**Confidence:** HIGH
**Note:** Pass 3 BC-MCPSRV-010 said "highest port wins" — that's the `DiscoverServer` policy (discover.go:49-51), not `CleanStale`. Separate concerns.

### Lock-file write permissions

- Lock file: `0o600` (lock.go:56) — verified by lock_test.go:242-254 (`TestLockManager_FilePermissions`).
- IDE dir: `0o700` (lock.go:40).

**Note:** Pass 8 §270-310 flagged that `/tmp/lazyclaude/server.log` is mode 0644 (world-readable) while daemon's runtime dir is 0700. The lock dir is correctly 0700 — that's a separate path.

---

## 7. Error Paths Inventory

Cross-cutting taxonomy of every error-emitting path in the package (for porting completeness):

| Origin | Trigger | Wire behavior |
|---|---|---|
| WS upgrade reject | Wrong/missing auth token | 401 `unauthorized\n` (server.go:278-280) |
| WS read loop | `websocket.CloseStatus(err) != StatusNormalClosure` | Logs, exits loop, calls `RemoveConn(connID)` (server.go:298) |
| WS parse | JSON unmarshal fails | Logs `ws parse %s: %v`, **continues read loop** (server.go:314-317) — does not respond with JSON-RPC error |
| WS marshal | Response encode fails | Logs, **continues read loop** (server.go:324-328) |
| WS write | `conn.Write` errors | Logs, exits loop (server.go:330-333) |
| JSON-RPC unknown method (request) | `req.IsNotification() == false` | `-32601` error response (handler.go:55) |
| JSON-RPC unknown method (notification) | Notification | Silently ignored (handler.go:52-54) |
| `initialize` | — | Always success (handler.go:72-83); no error path |
| `ide_connected` | Invalid params JSON | Logged; no state change (handler.go:91-93) |
| `ide_connected` | `pid <= 0` | Logged; no state change (handler.go:95-98) |
| `ide_connected` | `resolveWindow` error | Logged; **proceeds to check window == ""** (handler.go:100-103) — only if window also empty does it bail (104-108) |
| `openDiff` | Invalid params | `-32602 invalid params` (handler.go:152) |
| `openDiff` | Empty / non-absolute path | `-32602 path must be absolute: %s` (handler.go:143-146) — subsumes BC-MCPSRV-013 |
| `openDiff` | `connID` not registered | `-32603 connection not registered` (handler.go:163) |
| `openDiff` | `notify.Enqueue` fails | Logged; **success response still returned** (handler.go:178-180) |
| `/notify` | non-POST | 405 |
| `/notify` | Bad JSON | 400 `bad request` |
| `/notify` | `pid <= 0` | 400 `invalid pid` |
| `/notify` | window unresolved (non-`tool_info`) | 404 `window not found` |
| `/notify` | window unresolved (`tool_info`) | **404** (server.go:398-401 catches both; permission_prompt path has fallback at 389-397, but `tool_info` does NOT) |
| `/notify` | json encode of response fails | Logged (server.go:451-453) — already sent 200 OK header |
| `/stop`, `/session-start` | Window unresolved | **200 with empty window** (see BC-MCPSRV-034) |
| `/prompt-submit` | Window unresolved | 404 |
| `/msg/send` | Empty from / empty to / from==to | 400 |
| `/msg/send` | Invalid type | 400 `invalid message type` |
| `/msg/send` | Body > 10 KB | 400 `body too large (max 10KB)` |
| `/msg/send` | Recipient not found in lister | 404 `recipient session not found` |
| `/msg/send` | Window unresolvable | 502 `recipient session has no tmux window` |
| `/msg/send` | tmux SendKeysLiteral fails | 502 `failed to deliver message` |
| `/msg/send` | tmux Enter SendKeys fails | **Logged, response still 200 "delivered"** (handler_msg.go:295-297) — silently degraded path |
| `/msg/create` | SessionCreator nil | 503 `session creator not available` |
| `/msg/create` | Missing from/name | 400 `from and name are required` |
| `/msg/create` | Invalid type | 400 `type must be worker or local` |
| `/msg/create` | `FindProjectForSession` returns nil | 404 `caller session not found` |
| `/msg/create` | Creator returns error | 500 `create session failed` |
| `/msg/create` | Creator returns nil result + nil error | 500 `create session failed` |
| `/msg/resume` | SessionCreator nil | 503 |
| `/msg/resume` | Empty id | 400 |
| `/msg/resume` | Creator error | 500 + **leaks creator error.Error() into response** (handler_msg.go:357-359) — different from `/msg/create` which uses generic "create session failed" |

### P2 finding: Error-message leak inconsistency between `/msg/create` and `/msg/resume`

- `/msg/create` returns generic `"create session failed"` (handler_msg.go:144).
- `/msg/resume` returns `err.Error()` (handler_msg.go:358).

If `SessionCreator.ResumeSession` returns an error containing internal paths or stack-like info, that leaks. The two endpoints should be consistent. **Disposition:** P2 — minor, but the asymmetry is worth a porter caveat.

---

## 8. Cross-Cutting Concerns

### Logging

The server uses stdlib `log.Logger` (passed in via `New`'s third arg). Every endpoint logs at least one line — there is no structured logging. Examples:

- `notify: type=%s pid=%d window=%s tool=%s` (server.go:407)
- `notify: pid=%d not found, using last pending window %q` (server.go:395)
- `notify: DROPPED — empty toolName for window %s pid=%d` (server.go:446)
- `ws connected: %s` / `ws disconnected: %s` (server.go:293, 299)
- `ide_connected: pid=%d window=%s` (handler.go:114)
- `openDiff: window=%s file=%s` (handler.go:167)
- `msg/create: %v` / `msg/sessions: encode: %v` etc.

Output destination is the `*log.Logger` passed to `New` — in production, the in-process MCP server uses a logger writing to `/tmp/lazyclaude/server.log` with prefix `lazyclaude-srv:` (per `cmd/lazyclaude/root.go:416` referenced in Pass 1 BC-MCPSRV section and Pass 0 paths table).

### Telemetry

**None.** No metrics emission, no OpenTelemetry spans, no Prometheus counters. The package is observability-poor by design (zero deps beyond logging + stdlib + tmux + broker).

### Error reporting

Every error path either:
1. Returns an HTTP status code with a text body (via `http.Error`), or
2. Returns a JSON-RPC error response (codes `-32601`/`-32602`/`-32603`), or
3. Silently logs and continues (read-loop parse errors, response encode errors).

No panics propagate out of the package. No `panic()` calls in production code.

### Auth

Detailed in Section 4 (BC-MCPSRV-028, 029). Sole token store is the lock file (mode 0600, owner-read). Token generated per server start by caller (not in this package — confirmed: there's no `generateToken` here; it's passed in via `Config.Token`).

---

## 9. Dependencies (Verified)

```
internal/server → internal/core/event (broker)
internal/server → internal/core/model (ActivityState, ToolNotification, etc.)
internal/server → internal/core/tmux (Client, FindWindowForPid)
internal/server → internal/adapter/tmuxadapter (DetectMaxOption)
internal/server → internal/notify (Enqueue)
internal/server → nhooyr.io/websocket (WS layer)
internal/server → encoding/json, net/http, net, crypto/subtle, sync, time, log, os, os/exec, path/filepath, strconv, strings, fmt, context, bytes, syscall, io
```

Confirmed via grep through each .go file's imports block.

**Reverse imports (who uses `internal/server`):**
- `cmd/lazyclaude/root.go` — composes the TUI + in-process server.
- `cmd/lazyclaude/server.go` — standalone `lazyclaude server` subcommand.
- `cmd/lazyclaude/daemon_cmd.go` — daemon mode also spawns an in-process MCP server (`daemon_cmd.go:67`).
- `cmd/lazyclaude/sessions.go`, `msg.go`, `setup.go` — use `server.DiscoverServer`, `server.NewClient`, `server.EnsureServer`, `server.StopDaemon`.
- `internal/daemon/*` — only at composition time via the shared broker (no Go-level dependency).

**No package imports `internal/server` for its types beyond the cmd layer.** Even `internal/daemon` does NOT import this package — the daemon talks to its parallel in-process MCP server via the shared broker only.

---

## 10. P0/P1 Findings (New)

### P1 finding 1: `/msg/resume` is untested in this package

**Source:** handler_msg.go:312-376 (the handler exists with full logic).
**Gap:** Zero tests in `handler_msg_test.go` exercise `/msg/resume`. Search for `Resume` in `handler_msg_test.go` yields zero matches in test functions. The `fakeSessionCreator.ResumeSession` method (handler_msg_test.go:101-106) exists but is never called by any test in this file.
**Risk:** Behavior contract BC-MCPSRV-045 is source-derived but not test-verified. If a porter implements `/msg/resume` from these contracts and it diverges, no test will catch it.
**Disposition:** Recommend adding tests OR explicitly down-grading BC-MCPSRV-045 to MEDIUM confidence in synthesis. P1 because resume is on the documented CLI surface (`lazyclaude msg create` resumes via this endpoint in some flows — confirmed via `client.go:132-166` `ResumeSession` method).

### P1 finding 2: Diff-choice fast path (BC-MCPSRV-038) is untested AND leaks a goroutine using `context.Background()`

**Source:** server.go:428-439.
**Code:**
```go
if key, ok := s.state.GetDiffChoice(window); ok {
    s.log.Printf("notify: using pending diff choice %q for window %s", key, window)
    go func() {
        time.Sleep(50 * time.Millisecond)
        target := "lazyclaude:" + window
        if err := s.tmux.SendKeys(context.Background(), target, key); err != nil {
            s.log.Printf("notify: send diff choice key: %v", err)
        }
    }()
    break
}
```
**Gaps:**
1. `context.Background()` instead of `r.Context()` — the goroutine outlives the HTTP request *and* the server (`Stop` does not wait on this goroutine).
2. No test in `server_test.go` or `server_broker_test.go` exercises the fast path (no `SetDiffChoice` + permission_prompt sequence).
3. Under heavy diff-choice load + server restart, goroutines accumulate (bounded only by 50ms tmux send time, but still — they hold a reference to the closed-over `s.tmux` and `s.log`).

**Disposition:** P1. Test gap is the main issue; goroutine leak is short-lived but technically incorrect. A porter should plumb `r.Context()` AND add a regression test.

### Security note (informational, not a finding)

- Lock file mode 0600 is correct (BC-MCPSRV verified at lock_test.go:242-254).
- Pass 8 §310-311 flagged that `~/.claude/ide/<port>.lock` file mode is "unspecified by Go default (typically 0644)". This is **WRONG** — `os.WriteFile(path, data, 0o600)` is explicit at lock.go:56. Pass 8 §311 should be corrected: the lock file IS 0600, not "unspecified".

---

## 11. monocle Relevance Assessment

**Verdict: HIGH RELEVANCE — strong port candidate.**

This subsystem encodes three monocle-relevant gene families:

1. **The hook discovery + auth protocol** (lock file scan, PID-liveness, constant-time token compare, restart-resilient discovery). Pass 8 §384-385 explicitly calls out "adopt the discovery pattern verbatim" with `subtle::ConstantTimeEq` as the Rust mapping. Every contract under §6 (BC-MCPSRV-046, 047, 048) is reusable.

2. **The broker-vs-file dual-path dispatch with single-broker injection** (`WithBroker` semantics, `ownsBroker` flag, BC-MCPSRV-027, BC-MCPSRV-041). This is the canonical answer to "what happens when the GUI subscriber dies during shutdown" — drop, don't backpressure. It's load-bearing for the hook-handler-must-return-fast requirement (Pass 3 BC-BROKER-003 "load-bearing for hook handlers").

3. **The two-phase /notify protocol** (BC-MCPSRV-003, 004, 036, 038) with FIFO pending-tool TTL. This is the core mechanism that lets PreToolUse and the subsequent Notification hook arrive from *different node processes* and still correlate — via window-keyed pending state with `LastPendingWindow` fallback (BC-MCPSRV-031). A monocle port that integrates with Claude Code hooks MUST replicate this exactly, or it will break across hook process boundaries.

The msg-bus surface (`/msg/{send,create,resume,sessions}`) is **separable** from the PM/Worker persona per Pass 8 §354-355 — those endpoints are a generic inter-session bus and are explicitly retained for monocle as `BC-Bus.*`.

The standalone-mode `/msg/create --type local` (vs daemon's `/msg/create --type pm`) schema divergence (Pass 8 §273-278 P1) is rooted here at handler_msg.go:119-122. A monocle port should unify or precisely document.

The `internal/server` package has the **highest test density** of any subsystem (3,263 test LOC / 2,262 production LOC = 144%). High behavioral confidence makes this an excellent port target.

---

## Delta Summary

- **New contracts added:** BC-MCPSRV-021..048 (28 new contracts, of which 6 strengthen prior Pass 3 contracts via mechanism detail and 22 are net-new).
- **Pass 3 contracts confirmed:** BC-MCPSRV-001..020 (all 20).
- **New P1 findings:** 2 (BC-MCPSRV-045 untested resume path; BC-MCPSRV-038 untested + goroutine context bug).
- **New P2 findings:** 1 (`/msg/resume` vs `/msg/create` error-message leak inconsistency).
- **Pass 8 correction proposed:** Pass 8 §311 incorrectly claims lock file mode is unspecified; it's explicitly 0600 (lock.go:56, lock_test.go:252).
- **Remaining gaps for round 2 consideration:**
  - `LastPendingWindow` is untested in `state_test.go` (BC-MCPSRV-037).
  - Diff-choice fast path (BC-MCPSRV-038) is untested.
  - `/msg/resume` endpoint is untested (BC-MCPSRV-045).
  - `CapturePaneANSI` error fallback (BC-MCPSRV-039) is untested.
  - `/notify` DROPPED path (BC-MCPSRV-033) is untested.
  - `/stop` / `/session-start` empty-window publish (BC-MCPSRV-034) is untested.
  - Boundary between `internal/server.Client` and the various `cmd/lazyclaude/*.go` consumers — could be deepened, but it's a consumer concern (`cmd-glue` deepening covered this).

---

## Novelty Assessment

**Novelty: SUBSTANTIVE.**

This round added 28 new contracts (BC-MCPSRV-021..048), uncovered 2 P1 findings (untested resume endpoint + diff-choice goroutine bug), 1 P2 finding (error-leak asymmetry), and corrected one Pass 8 fact. The new contracts include load-bearing mechanism details (FIFO+TTL semantics for pending tools, LastPendingWindow algorithm, diff-choice fast-path semantics, MaxOption fallback, Edit-tool file-size cap) that the Pass 3 contracts had only at a surface level. Without this round, a monocle port would either re-discover these from source or get them wrong.

The structural-pass goal — file manifest, types, public surface, state model, dependencies — was achieved AND extended into behavioral territory because the gaps were concentrated in tested-but-uncontracted endpoints. Removing this round's findings would materially change how the system is spec'd.

## Convergence Declaration

**Another round needed.** Specific gaps to target in round 2:

1. **Untested-but-source-clear behavior verification gaps** — round 2 should walk through each gap in Section 11 to determine if any indicate actual bugs or just missing test coverage. Most are likely test-only gaps but a few (BC-MCPSRV-038 goroutine, BC-MCPSRV-033 silent drop, BC-MCPSRV-034 empty-window publish asymmetry) might be design smells worth promoting to disposition findings.
2. **Cross-pollination with Pass B-deep-daemon-r{1,2,3}**: does the daemon's `/msg/create` schema drift (Pass 8 §273) have semantic implications missed here? Need to read daemon-r* and compare.
3. **`Client` HTTP error semantics** — `client.go` was inventoried but its 401/404/500 mappings weren't fully extracted; CLI consumers depend on these.
4. **Interaction between `setActivity` and the broker's `ActivityNotification` events** — currently described in code comment (server.go:225-235) but the actual non-interaction is the load-bearing invariant. Round 2 should make this a formal contract.

If round 2 finds these all to be either confirmable-from-source or test-coverage gaps without behavioral surprises, that's NITPICK territory and we stop. Cap is 5 rounds.

## State Checkpoint

```yaml
pass: B
subsystem: internal/server
round: 1
status: complete
files_scanned_round: 10 (all production .go files) + 9 test files (skim) + 4 prior pass files (read)
contracts_added: 28
contracts_confirmed: 20
p1_findings: 2
p2_findings: 1
pass8_corrections: 1
timestamp: 2026-05-11T17:55:00Z
novelty: SUBSTANTIVE
next_round_needed: true
```
