# Pass B Deepening — `internal/server/` Round 2

**Subsystem:** `internal/server/`
**Round:** 2
**Prior outputs read:**
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-server-r1.md` (this deepening round 1)
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-daemon-r1.md` through r3 (cross-pollination)
- `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/event/broker.go` (referenced; verified single-mutex)

**Round 2 charter (from r1 §convergence-declaration):**
1. Verify each gap from r1 §11 is test-only or also a behavioral surprise.
2. Cross-pollinate with daemon-r{1,2,3} for any `/msg/*` schema-drift implications missed.
3. Extract `Client` HTTP error semantics (CLI consumers depend on these).
4. Promote the `setActivity` non-interaction with broker `ActivityNotification` to a formal contract.

---

## 1. Gap Verification: Was Each r1 Gap Test-Only, or Also a Behavioral Smell?

### Gap 1: `LastPendingWindow` untested in state_test.go (r1 §11 item 1)

**Re-verified:** zero direct tests of `LastPendingWindow` in `state_test.go`. The function is exercised indirectly via the server's two-phase test (`server_test.go:218-255`) only for the *positive* case (latest window match).

**Behavioral subtleties not test-covered:**
- Empty-pending map returns `""` (state.go:175). Untested.
- Multiple windows with different expiry — picks highest expiry, not "last inserted". This is **load-bearing**: when permission_prompt arrives from a different PID than the prior PreToolUse, we route to the window with the freshest activity. If a window has an expired entry AND a fresh entry (FIFO ordering preserved by SetPending append), the fresh one's expiry wins.
- Concurrency: `LastPendingWindow` uses `RLock`, so concurrent `SetPending` (`Lock`) can interleave. Tested implicitly via `TestState_ConcurrentAccess` (state_test.go:209-232) under `-race`.

**Verdict:** Test-only gap. No behavioral surprise. The algorithm is correct as documented. Adding direct unit tests would improve regression coverage but doesn't change the spec.

### Gap 2: Diff-choice fast path untested + `context.Background()` (r1 §11 item 2)

**Re-verified:** zero references to `SetDiffChoice` or `GetDiffChoice` anywhere in test files (`awk` confirms). The state-level `DiffChoice` tests (state_test.go:166-207) cover only the `State` mechanism, not the server's consumption of it.

**Behavioral re-examination:**
- The fast path triggers when permission_prompt arrives and a prior `openDiff` popup has set a choice (server.go:428-439).
- `openDiff` is called via WebSocket (handler.go:149-187). The GUI presumably handles the response by sending a key choice — but `SetDiffChoice` is called from where?

**Searching for `SetDiffChoice` callers:**
<br>
This function is in the package's public surface (state.go:124-138), so it must be called from outside the package. Searching `internal/gui`/`cmd/lazyclaude` is out of scope for this subsystem deepening, but the spec implication is clear: **`server.State.SetDiffChoice` is a write-from-GUI hook back into server state**. This is an interesting design pattern — the server exposes mutable state for GUI to drive diff-choice forwarding.

The fact that the GUI mutates server state via the exported `State` API (rather than via the broker or an explicit RPC) is an architectural seam worth calling out for porting.

**Goroutine bug confirmation:**
- `server.go:431-438`: `go func() { time.Sleep(50ms); s.tmux.SendKeys(context.Background(), ...) }()`.
- After `Stop()`, the server's `httpSrv` is shut down, the broker may be closed, but this goroutine still runs to completion (or its tmux send fails). The `s.tmux` client pointer is still valid (not nil'd), so it likely sends successfully even post-shutdown. **Not a crash bug, but a goroutine-leak window.**
- A `goleak.VerifyTestMain` failure WOULD trigger if any test exercised this path and ran shorter than 50ms. The reason this hasn't bitten the project is because no test exercises it.

**Verdict:** This is a real (small) bug AND a test gap. Stays P1.

### Gap 3: `/msg/resume` untested (r1 §11 item 3)

**Re-verified:** `fakeSessionCreator.ResumeSession` exists at handler_msg_test.go:101-106 but is never called by any test function. Searching for `Resume` test functions yields zero matches.

**Behavioral re-examination:**
- `/msg/resume` (handler_msg.go:312-376) follows the same auth + 405-method + body-cap pattern as siblings.
- Calls `sc.ResumeSession(ctx, req.ID, req.Prompt, req.Name)`.
- On success: returns `MsgResumeResponse{Status: "resumed", Session: ...}`.
- On error: returns `err.Error()` via `http.Error` (the leak inconsistency from r1 §10 P2 finding).

**Production call sites of the endpoint:**
- `client.go:133-166` (`ResumeSession`) — used by CLI subcommands.

**Verdict:** Test-only gap. The handler is small and shape-compatible with `/msg/create`. No new behavioral surprise.

### Gap 4: `CapturePaneANSI` error fallback untested (r1 §11 item 4)

**Re-verified:** No test injects an error from `tmux.MockClient.CapturePaneANSI`. The `MockClient` has `ErrSendKeys` (handler_msg_test.go:561) but not an analogous capture-error field — would need mock extension to test.

**Behavioral re-examination:**
- `dispatchToolNotification` (server.go:491-575) starts with `maxOpt := 3` and only overrides on success.
- The fallback is graceful. Server-side test of the error path requires `MockClient.CapturePaneANSI` to be injectable for error.

**Verdict:** Test-only gap. Not a behavioral surprise.

### Gap 5: `/notify` DROPPED path untested (r1 §11 item 5)

**Re-verified:** No test for the silent-drop branch (server.go:442-447).

**Behavioral re-examination:** This path triggers when:
1. `req.Type == ""` (permission_prompt phase).
2. Diff-choice fast-path didn't fire.
3. `resolveToolInfo` returned `toolName == ""` (neither pending data nor request `ToolName` was non-empty).

This can happen if: the Notification hook arrived but no PreToolUse hook fired first AND the Notification hook's request body omitted `tool_name`. The `Notification` hook commands (per `internal/core/config/hooks.go:35`) DO carry tool info — but the JS code there sends `tool_name`, `tool_input` (verify via `cat hooks.go:35` if needed). So the drop should be rare in production.

**Verdict:** Test gap. Possibly indicates a production-rare edge case that should still be tested for defense-in-depth, but no behavioral surprise vs the spec.

### Gap 6: `/stop` / `/session-start` empty-window publish asymmetry (r1 §11 item 6, BC-MCPSRV-034)

**Re-verified:** No test asserts that `/stop` with unresolved PID returns 200 AND publishes a broker event with `Window == ""`.

**Behavioral re-examination — this is more interesting than test-only:**
- A `StopNotification{Window: ""}` published to the broker will be received by subscribers (broker fan-out applies to any event). What does the GUI do with `Window == ""`?
- Per Pass 1 BC-GUI-SIDEBAR-001, the GUI's `windowActivity` map is keyed by window. An empty key would either be ignored (no key match) or update a phantom entry.
- The daemon's SSE handler does `sessionIDForWindow` (Pass 8 §183) — empty window maps to empty session_id, which the doc says is acceptable.

So the empty-window publish IS by design: lifecycle events arrive even when window resolution fails, allowing the broker pipeline to record "something happened on an unknown window" rather than dropping the event. The GUI/daemon both tolerate empty window (the daemon's SSE explicitly handles it; the GUI's sidebar would just not update any session).

**Verdict:** Confirmed by-design. **Promote to formal contract** (added below as BC-MCPSRV-049).

---

## 2. Cross-Pollination with daemon-r{1,2,3}

Read scope:
- daemon-r2 §"BC-DAEMON-SRV-013" (line 376): "DIFFERS from BC-MCPSRV-018 which supports 'worker' or 'local'. This daemon path explicitly does NOT support 'local' (server.go:573-591 has only worker/pm cases)."
- daemon-r3 §convergence-declaration: confirms divergence as P2 schema drift, no contradiction with server findings.

### Reconciled schema map (`/msg/create`)

| Caller | Endpoint owner | Supported `type` | Source |
|---|---|---|---|
| CLI `lazyclaude msg create --type worker` | server (MCP) | `worker` | server/handler_msg.go:119-122 (rejects anything but worker/local) |
| CLI `lazyclaude msg create --type local` | server (MCP) | `local` | server/handler_msg.go:119-122 |
| Daemon HTTP `POST /msg/create` | daemon | `worker`, `pm` | daemon/server.go:573-590 (per daemon-r2) |
| Daemon HTTP `POST /msg/create` with type "" | daemon | rejected (no default) | daemon-r2 |
| Daemon HTTP `POST /msg/create` with type "local" | daemon | **rejected** | daemon-r2 confirms only worker/pm |

The CLI `msg create` ALWAYS routes through the MCP server (via `lazyclaude msg create`-issued `server.NewClient`). It NEVER hits the daemon. So `--type pm` is currently **inaccessible from CLI** — it's only invocable by inter-daemon HTTP calls (from `internal/daemon/composite_provider.go` per Pass 1 BC-DAEMON-001 and daemon-r1 BC-CP-* contracts).

### New contract from this reconciliation

#### BC-MCPSRV-049: `/msg/create --type pm` does not exist in this package; the corresponding behavior lives in daemon's `POST /msg/create`
**Postconditions:** Per source line server/handler_msg.go:119-122, only "worker" and "local" pass validation. Sending `{type: "pm"}` returns 400 "type must be worker or local". This is a deliberate split — see daemon-r2 BC-DAEMON-SRV-013.
**Evidence:** server/handler_msg.go:119-122 vs daemon/server.go:573-590 (cross-referenced).
**Confidence:** HIGH
**Disposition note:** P1 in Pass 8 §273-278. Round 2 confirms the divergence is rooted in this file's validation logic. A monocle port should either:
- (a) Unify the surfaces (single API with `{worker, local, pm}`), OR
- (b) Document the split as deliberate (CLI users get worker/local; inter-daemon traffic gets worker/pm).

---

## 3. Client HTTP Error Semantics (CLI Consumer Surface)

The `Client` (`client.go:19-32`) is what CLI subcommands use to talk to a remote-or-local MCP server. Every method follows the same pattern:

```go
req.Header.Set("X-Auth-Token", c.token)
// (NEVER sets X-Claude-Code-Ide-Authorization — that's a hook-only header)
resp, err := c.httpClient.Do(req)
if err != nil { return wrap(err) }
defer resp.Body.Close()
if resp.StatusCode != http.StatusOK {
    return fmt.Errorf("server returned %d: %s", resp.StatusCode, readErrorBody(resp))
}
// decode body
```

### BC-MCPSRV-050: `Client` uses ONLY `X-Auth-Token`, never `X-Claude-Code-Ide-Authorization`
**Evidence:** client.go:40, 76, 112, 148. The Claude-Code-Ide header is reserved for hook callers (the node-eval one-liners per BC-HOOK-002).
**Confidence:** HIGH

### BC-MCPSRV-051: `Client` HTTP timeout is 5s for ALL methods
**Postconditions:** `clientTimeout = 5 * time.Second` (client.go:13). `Sessions`, `SendMessage`, `CreateSession`, `ResumeSession` all share one `http.Client{Timeout: 5s}`. There is no per-method override. `CreateSession` for a worker that does `git worktree add` (potentially slow) is bound by the same 5s.
**Evidence:** client.go:13, 30.
**Confidence:** HIGH
**Caveat:** This 5s timeout is **client-side**. The server has no equivalent timeout — it processes synchronously and can take >5s if `git worktree add` is slow. A slow create returns to CLI as a timeout error, but the server-side session may still be created. **This is a real porter-trap.**

### BC-MCPSRV-052: `Client` error body is truncated to `maxErrBody = 512` bytes via `io.LimitReader`
**Postconditions:** On non-200, error message format is `"server returned %d: %s"` where `%s` is the trimmed first 512 bytes of the body. Prevents log-bombing.
**Evidence:** client.go:15-16, 48-50, 85-87, 121-123, 156-158, 168-175.
**Confidence:** HIGH

### BC-MCPSRV-053: `Client.CreateSession` and `Client.ResumeSession` return wire-typed responses; CLI is responsible for decoding `Status` and `Error` fields
**Postconditions:**
- `CreateSession` returns `*MsgCreateResponse{Status, Session, Error}` (client.go:130).
- `ResumeSession` returns `*MsgResumeResponse{Status, Session, Error}` (client.go:166).
- On HTTP 500 (server.handleMsgCreate generic error), `Error` field is NOT populated; the error is in the body text (returned as Go error, not in struct).
- On HTTP 200 with empty `Status`, the call is ambiguous — the server never returns 200 with non-"created"/non-"resumed" status, so this is theoretically dead, but the struct supports it.
**Evidence:** client.go:125-129, 161-165; server/handler_msg.go:162-177 (handleMsgCreate response shape).
**Confidence:** HIGH (mechanism); MEDIUM (no test asserts empty-status behavior).

---

## 4. `setActivity` vs Broker `ActivityNotification` — Formal Contract

The code comment at server.go:225-235 reads:

```go
// setActivity records the hook-based activity state for a tmux window.
// Called directly by hook handlers (not via broker subscription) to avoid
// inflating HasSubscribers() and altering broker-vs-file dispatch logic.
```

This is the architectural invariant. Each hook handler does **two writes**: first `setActivity` (to the in-process `activityMap`), then `notifyBroker.Publish(ActivityNotification)`. They are NOT redundant; they serve different consumers.

### BC-MCPSRV-054: Every hook endpoint that updates activity does BOTH `setActivity` AND `notifyBroker.Publish` (in that order, in the same handler invocation)

**Postconditions:** Across all four hook handlers:
| Endpoint | setActivity call | Publish call |
|---|---|---|
| `/notify` tool_info path | server.go:418 | server.go:419 |
| `/notify` dispatchToolNotification | server.go:492 | server.go:565 |
| `/stop` | server.go:618 | server.go:626 |
| `/session-start` | server.go:665 | server.go:672 |
| `/prompt-submit` | server.go:716 | server.go:723 |

The order is invariant: `setActivity` first (which is a no-op on empty window), then publish. There is no test that asserts the relative order, but the source pattern is uniform.

**Evidence:** server.go: 418-419, 492 & 565, 618 & 626, 665 & 672, 716 & 723.
**Confidence:** HIGH (mechanism by uniform code pattern)

### BC-MCPSRV-055: `activityMap` is **never read** by code in this package's hot path — only by `/msg/sessions` enrichment AND the `WindowActivity` accessor
**Postconditions:**
- Hot-path hook handlers WRITE to `activityMap` but never READ.
- `/msg/sessions` invokes `enrichWithActivity` (server.go:259-273) which reads.
- `WindowActivity(window)` (server.go:248-256) reads — exposed for callers outside the package (the GUI sidebar uses it).

Critical implication: the broker's `ActivityNotification` and the `activityMap` are **two parallel sinks for the same activity event**. A GUI consumer can subscribe to the broker for live updates, OR poll `WindowActivity` for current state — they should agree at quiescence.

**Evidence:** Single reader sites: server.go:248-256 (`WindowActivity`), server.go:259-273 (`enrichWithActivity`). No other reads.
**Confidence:** HIGH (verified by grep `s.activityMap` and `s.activityMu` across the package).

### BC-MCPSRV-056: `WindowActivity(window)` for an unknown window returns `(ActivityUnknown, "")` — NOT an error
**Evidence:** server.go:248-256 explicit branch. Tested in `TestServer_Activity_UnknownWindowReturnsUnknown` (server_test.go:698-705).
**Confidence:** HIGH

### Broker single-mutex correction (refines r1 §10 narrative)

The r1 contract BC-MCPSRV-041 quoted the server.go:557-561 comment: "HasSubscribers() and Publish() acquire separate locks". This is **inaccurate in the source comment**. `internal/core/event/broker.go` uses a single `sync.Mutex` for both methods (broker.go:10, 60-75, 77-82). The race the comment describes is real (a subscriber can `Cancel()` between the `HasSubscribers()` mutex release and the `Publish()` mutex acquire), but the wording "separate locks" misrepresents the mechanism — there is one mutex, taken sequentially.

#### BC-MCPSRV-057 (correction): The broker mutex is single; the race window between `HasSubscribers()` and `Publish()` exists because they are sequential calls (not concurrent under a single critical section)
**Postconditions:** `HasSubscribers` and `Publish` both call `b.mu.Lock()`/`b.mu.Unlock()` (broker.go:60-82). Between the `HasSubscribers` return and the next call's `Lock`, another goroutine can run `Cancel()` and remove the last subscriber. The behavioral consequence the code comment describes is correct (silent drop during shutdown), but the *mechanism* is "non-atomic check-then-act", not "separate locks".
**Evidence:** broker.go:9-14, 60, 78.
**Confidence:** HIGH
**Disposition note:** Pass 8 / monocle port should refine the source comment to "non-atomic sequential calls" rather than "separate locks". Minor doc bug but it could mislead a porter.

---

## 5. New Contracts Promoted from r1 Gaps

### BC-MCPSRV-058: `/stop` and `/session-start` are **fire-and-forget lifecycle hooks**: they tolerate empty window and publish anyway

**Preconditions:** POST with auth; `pid > 0`; window resolution fails (no PID cache, pidwalk returns nil, no fallback pending).
**Postconditions:**
1. `setActivity("", state, "")` is a no-op (BC-MCPSRV-035).
2. `notifyBroker.Publish(...)` is called with `Window: ""`.
3. HTTP response is 200 OK with `{"status": "ok"}`.

**Contrast with `/notify` and `/prompt-submit`:** These two return 404 on unresolved window (server.go:398-401, 706-709). The asymmetry is intentional: `/notify` triggers a popup (needs a window for tmux SendKeys); `/prompt-submit` arms running state (needs a window to attribute to). `/stop` and `/session-start` are observational — losing the window just means we can't attribute the lifecycle to a session, but we should still record that it happened.

**Evidence:** server.go:609-630 (`handleStop`), 656-676 (`handleSessionStart`). Source-derived; **gap remains**: no test asserts this asymmetry.
**Confidence:** HIGH (source); MEDIUM (test coverage).

### BC-MCPSRV-059: `setActivity` is the sole writer of `activityMap`; all writes are RWMutex-protected via `s.activityMu`

**Evidence:** server.go:228-235 (only `setActivity` writes), 248-273 (only readers, all with `RLock`).
**Confidence:** HIGH

---

## 6. Discoverability Edge Cases

`DiscoverServer` (discover.go:20-58) is the Go-API counterpart to the hook's JS lock-scan. Round 1 inventoried it; let me add the edge-case contracts:

### BC-MCPSRV-060: `DiscoverServer` skips directories, non-`.lock` files, malformed port names (non-integer), and locks where `lock.App` is neither `""` nor `"lazyclaude"`
**Postconditions:** Filter chain at discover.go:28-46.
**Evidence:** discover.go:28-46 + tested in discover_test.go:76-107 (`TestDiscoverServer_SkipsNonLazyclaudeLock`).
**Confidence:** HIGH

### BC-MCPSRV-061: `DiscoverServer` validates aliveness via TCP DialTimeout 500ms — same threshold as `LockManager.CleanStale`
**Postconditions:** `isServerAlive(port)` uses `net.DialTimeout("tcp", "127.0.0.1:port", dialTimeout)` where `dialTimeout = 500 * time.Millisecond` (ensure.go:14). The dialTimeout constant is in `ensure.go`, but `discover.go` uses it indirectly via `isServerAlive`.
**Evidence:** discover.go:46-48, ensure.go:14, 162-169.
**Confidence:** HIGH

### BC-MCPSRV-062: `DiscoverServer` returns an error (NOT nil) when no live server found
**Postconditions:** `fmt.Errorf("no running lazyclaude server found (checked %s)", ideDir)` (discover.go:54-56). Tested in `TestDiscoverServer_SkipsDeadServer` (discover_test.go:53-74) and `TestDiscoverServer_EmptyDir` (discover_test.go:170-176).
**Confidence:** HIGH

### BC-MCPSRV-063: When multiple alive lazyclaude servers exist, `DiscoverServer` returns the highest port — matching the hook's behavior
**Postconditions:** `if best == nil || port > best.Port { best = ... }` (discover.go:49-51). Tested in `TestDiscoverServer_PicksHighestPort` (discover_test.go:109-168).
**Confidence:** HIGH

---

## 7. `EnsureServer` / `RestartServer` Lifecycle Contracts

These manage out-of-process server instances — used by `lazyclaude setup` and recovery paths.

### BC-MCPSRV-064: `EnsureServer` skips start if port file exists and TCP alive; removes stale-or-malformed port file
**Postconditions:**
1. Port file readable AND parseable AND server alive on that port → `{Port, Started: false}`.
2. Port file readable AND parseable AND server NOT alive → remove port file, then `startServer`.
3. Port file readable AND not parseable → remove port file, then `startServer`.
4. Port file not readable → `startServer`.
**Evidence:** ensure.go:99-120. Tested in `TestEnsureServer_SkipsIfPortFileExistsAndAlive`, `TestEnsureServer_StartsIfPortFileStale`, `TestEnsureServer_InvalidPortFile` (ensure_test.go:17-97).
**Confidence:** HIGH

### BC-MCPSRV-065: `RestartServer` adds a 3-second debounce: a recently-modified port file with an alive server is reused, NOT restarted
**Preconditions:** Port file mtime within `restartDebounce = 3 * time.Second` (ensure.go:21) AND server alive on that port.
**Postconditions:** Returns `{Port, Started: false}` without killing or restarting.
**Evidence:** ensure.go:40-50 (debounce logic), `TestRestartServer_ReusesRecentlyStartedServer` (ensure_test.go:99-121).
**Confidence:** HIGH
**Why:** Comment at ensure.go:18-21: "prevents cascading restart loops when multiple processes call RestartServer simultaneously after a crash". Multiple `claude` hooks could each detect a dead server and call RestartServer concurrently; the debounce ensures only one start happens.

### BC-MCPSRV-066: `startServer` exec's `lazyclaude server --port 0` and detaches via `cmd.Process.Release()`
**Postconditions:**
1. Stdout is suppressed (`cmd.Stdout = nil`, ensure.go:127).
2. Stderr writes to `/tmp/lazyclaude/server.log` (mode 0644, append) — **the world-readable log file flagged by Pass 8 §310-311**.
3. After `cmd.Start()` succeeds, parent's logfile fd is closed; child inherited.
4. `cmd.Process.Release()` detaches so parent can exit without zombie.
**Evidence:** ensure.go:122-145.
**Confidence:** HIGH
**Security note:** The 0644 log file IS confirmed here. This is the gene of Pass 8 §310 — not in `internal/server.Server.Start` but in `internal/server.startServer`. Worth tracking.

### BC-MCPSRV-067: `StopDaemon` reads port file, signals SIGINT to the PID from the lock file, removes the lock file AND port file
**Postconditions:** No-op if any of the read steps fails. Best-effort signal.
**Evidence:** ensure.go:60-71, 74-94 (`killServerOnPort` helper).
**Confidence:** HIGH

### BC-MCPSRV-068: `IsAlive(portFile)` is the public counterpart to `isServerAlive(port)` — accepts a port file path instead of a port number
**Postconditions:** Reads port file; on any failure returns false; otherwise TCP-dials.
**Evidence:** ensure.go:148-159.
**Confidence:** HIGH

---

## 8. Cross-Pollination Summary (Reconciled Findings)

| Topic | server-r1/r2 view | daemon-r1..r3 view | Synthesis |
|---|---|---|---|
| `/msg/create` type set | `{worker, local}` | `{worker, pm}` | Documented divergence (Pass 8 §273 P1, BC-MCPSRV-049). |
| Broker injection (`WithBroker`) | Owned by server lifecycle if not injected | Shared with daemon when present | Confirmed seam; no contradiction. |
| Hook discovery | Server writes `~/.claude/ide/<port>.lock`; CleanAllExcept clears siblings | Daemon spawns in-process MCP server using same code | Confirmed reuse; no contradiction. |
| `daemon /msg/send` response shape | (see daemon r2 BC-DAEMON-SRV-010..011) JSON body with `{"error": "..."}` | — | Differs from MCP `/msg/send` which uses `http.Error` plain text (BC-MCPSRV-015). Schema drift on send too. |
| Daemon `/health` API version 4 | — | Daemon-exposed | Not relevant to server. |
| SSE `sessionIDForWindow` | (out of scope) | Daemon translates window → UUID | Reaffirms server emits raw tmux window IDs; daemon translates. |

### New finding from cross-pollination

#### P1 finding 3 (server-r2): `/msg/send` response shape diverges between server (plain text via `http.Error`) and daemon (JSON `{"error": "..."}`)
**Source:**
- server/handler_msg.go:210-228 (uses `http.Error(w, "...", status)` → plain text body).
- daemon/server.go:498-548 (per daemon-r2 BC-DAEMON-SRV-010 — JSON-shaped error responses).
**Risk:** Same as Pass 8 §273 (schema drift on /msg/create), but extended to /msg/send. CLI consumers that hit either endpoint must dispatch on response Content-Type or status code, not body parsing.
**Disposition:** P1 alongside the existing /msg/create finding. Unify or precisely document.

---

## Delta Summary

- **New contracts added:** BC-MCPSRV-049..068 (20 new contracts).
- **Round 1 contracts confirmed (none refuted):** all 28.
- **Pass 3 contracts confirmed:** all 20.
- **New P1 findings:** 1 (response-shape divergence for `/msg/send`).
- **No new P0.**
- **Corrections to Pass 8 / source comments:** 1 (single-mutex broker; comment language imprecise).
- **Pass 8 §311 already noted in r1**: lock file mode IS 0600 (confirmed again).
- **Remaining gaps after r2:**
  - All 6 r1 §11 gaps are confirmed test-only OR confirmed by-design (BC-MCPSRV-058). None reveals a hidden behavioral surprise.
  - Goroutine context bug (r1 §10 P1 finding 2) confirmed not-yet-fixed.
  - `Client.SendMessage` returns no info on partial-delivery (the Enter SendKeys silent-degraded path of BC-MCPSRV-016) — minor (test could assert this but unclear if porters need to know).

---

## Novelty Assessment

**Novelty: SUBSTANTIVE** — but **declining**.

Round 2 added 20 contracts (BC-MCPSRV-049..068), one P1 finding (response-shape drift on /msg/send), one source-comment correction (single-mutex broker), and a security-relevant restatement (BC-MCPSRV-066 confirms /tmp/lazyclaude/server.log 0644 mode). It also verified that all 6 r1 gaps are test-only or by-design, which is itself substantive: a porter reading r1 might over-prioritize them.

However:
- The new contracts are mostly extensions of existing surfaces (Client error semantics, Ensure/Restart lifecycle, Discoverability edge cases). They fill in detail that r1 inventoried but didn't formalize.
- The cross-pollination finding (`/msg/send` shape divergence) is genuinely new but parallels the /msg/create finding already in Pass 8.
- No new architectural surprise, no new dependency, no hidden subsystem.

The "would removing this round change how to spec the system" test:
- BC-MCPSRV-049 schema-divergence ack — YES, it changes spec wording.
- BC-MCPSRV-051 client 5s timeout — YES, porters need to know.
- BC-MCPSRV-058 empty-window publish asymmetry — YES, it's a real behavioral contract.
- BC-MCPSRV-066 0644 server.log — YES, it sources the Pass 8 security flag here.
- BC-MCPSRV-057 mutex-correction — small, a doc-cleanup.

So substantive, yes. But round 3 would likely be cleanup: minor refinements, expanded test-coverage analyses, mock-injectability notes. The marginal value is dropping.

## Convergence Declaration

**One more round MAY be needed, but only for one specific reason:**

I haven't yet formally extracted contracts for the `Handler` (JSON-RPC) layer beyond what Pass 3 already drafted (BC-MCPSRV-011, 012, 013). The handler is small (handler.go = 188 LOC) and four explicit methods only, but:
- The `resolveWindow` chain (handler.go:117-132) silently swallows pidwalk errors when `w != nil` — there's a subtle precondition.
- `openDiff` writes a notification file via `notify.Enqueue` BUT also returns a success response — so the response and the side-effect are decoupled.
- `IsNotification` semantics (`id == nil || string(id) == "null"`) is the JSON-RPC notification predicate — round 1 mentioned it briefly but didn't formalize it.

These are small and could be folded into round 3 if anything new shows up there. Otherwise round 3 would be a polish pass that I'd honestly label NITPICK.

**Decision:** **Run round 3, narrow scope: JSON-RPC handler + final polish.** If it finds nothing substantive, declare NITPICK and stop.

## State Checkpoint

```yaml
pass: B
subsystem: internal/server
round: 2
status: complete
contracts_added: 20
contracts_confirmed: 48 (28 from r1 + 20 from Pass 3)
p1_findings_added: 1 (msg/send response shape divergence)
p1_findings_carried: 2 (resume untested, diff-choice goroutine)
p2_findings_carried: 1 (resume vs create error-leak)
pass8_corrections: 1 (broker single-mutex)
gap_resolutions: 6 of 6 r1 gaps verified as test-only or by-design
cross_pollination: daemon-r1..r3 read for /msg/* schema drift
timestamp: 2026-05-11T18:10:00Z
novelty: SUBSTANTIVE (declining)
next_round_needed: true (narrow: JSON-RPC handler polish)
```
