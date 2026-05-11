# Pass 3 — Behavioral Contracts: any-context/lazyclaude

Each contract is grounded in a test file (HIGH confidence) or source rationale (MEDIUM confidence) and cites file:line. Contracts marked with **M+** are monocle-relevant; **PMW** are PM/Worker-specific.

## BC Format Legend

- `BC-<area>-<NNN>` — unique ID per contract
- Confidence: HIGH = directly asserted by a test; MEDIUM = derived from source code logic; LOW = inferred from comments
- Subsystems: BROKER, LIFECYCLE, TMUX, SERVER (MCP), DAEMON, SESSION, GUI, MIRROR, REMOTE, HOOK, PROFILE, ASKPASS, TUNNEL

---

## BROKER (`internal/core/event`) [M+]

### BC-BROKER-001: Single subscriber receives every published event in order
**Preconditions:** `Broker[T]` created via `NewBroker[T]()`; subscriber via `b.Subscribe(bufSize)` with `bufSize >= 1`.
**Postconditions:** Two `b.Publish(x)` calls deliver both events to the subscriber's channel in publish order.
**Evidence:** broker_test.go:44-60 (`TestBroker_SubscribeAndPublish`).
**Confidence:** HIGH

### BC-BROKER-002: Multiple subscribers each receive every event (fan-out)
**Preconditions:** N subscribers active, each with sufficient buffer.
**Postconditions:** Every `Publish(x)` delivers `x` to every subscriber.
**Evidence:** broker_test.go:66-86 (`TestBroker_MultipleSubscribers`).
**Confidence:** HIGH

### BC-BROKER-003: Publish is NON-BLOCKING; events drop for slow subscribers
**Preconditions:** A subscriber with buffer size 1 that does not read.
**Postconditions:** Publishing 100 events completes within 2 seconds (i.e., does not block on the slow subscriber). Some events are silently dropped for that subscriber.
**Evidence:** broker_test.go:93-118 (`TestBroker_NonBlockingPublish`). Source: broker.go:69-74 select with `default`.
**Confidence:** HIGH
**Importance:** This is **load-bearing** for the hook handlers — POSTs from claude must return fast.

### BC-BROKER-004: Cancel closes the subscriber's channel
**Preconditions:** Active subscription.
**Postconditions:** After `sub.Cancel()`, `<-sub.Ch()` returns `(zero, false)`.
**Evidence:** broker_test.go:124-150 (`TestBroker_Cancel`).
**Confidence:** HIGH

### BC-BROKER-005: Cancel is idempotent
**Postconditions:** Calling `sub.Cancel()` twice does not panic.
**Evidence:** broker_test.go:156-167.
**Confidence:** HIGH

### BC-BROKER-006: Close closes all subscribers' channels
**Postconditions:** After `b.Close()`, every subscription's channel is closed.
**Evidence:** broker_test.go:173-192.
**Confidence:** HIGH

### BC-BROKER-007: Close is idempotent; Publish after Close is a no-op (no panic)
**Evidence:** broker_test.go:198-242 (`TestBroker_CloseIdempotent`, `TestBroker_PublishAfterClose`).
**Confidence:** HIGH

### BC-BROKER-008: Subscribe after Close returns an already-closed channel
**Postconditions:** `<-sub.Ch()` returns `(zero, false)` immediately.
**Evidence:** broker_test.go:213-227. Source: broker.go:48-51.
**Confidence:** HIGH

### BC-BROKER-009: Concurrent publishers do not panic or race
**Preconditions:** 8 publishers × 50 events each (400 total) into a buffer of 400.
**Postconditions:** All 400 events delivered.
**Evidence:** broker_test.go:249-280 (under `-race`).
**Confidence:** HIGH

### BC-BROKER-010: Cancelled subscription does not receive further publishes
**Preconditions:** Two subscribers; one is cancelled.
**Postconditions:** Subsequent publishes do not panic (send-on-closed-channel) and the active subscriber receives every event.
**Evidence:** broker_test.go:288-313.
**Confidence:** HIGH

### BC-BROKER-011: HasSubscribers reflects current state
**Postconditions:** Returns `true` iff at least one un-cancelled subscription exists; `false` after `Close`.
**Evidence:** broker_test.go:339-368.
**Confidence:** HIGH

### BC-BROKER-012: Goroutine-leak free
**Postconditions:** `goleak.VerifyTestMain(m)` runs at package level; tests complete without leaks.
**Evidence:** broker_test.go:14-16.
**Confidence:** HIGH

---

## LIFECYCLE (`internal/core/lifecycle`) [M+]

### BC-LIFECYCLE-001: Register + Close calls cleanup exactly once
**Evidence:** lifecycle_test.go:20-32.
**Confidence:** HIGH

### BC-LIFECYCLE-002: Cleanup runs in LIFO (reverse registration) order
**Postconditions:** Last-registered runs first.
**Evidence:** lifecycle_test.go:35-49.
**Confidence:** HIGH

### BC-LIFECYCLE-003: Close is idempotent (cleanup runs once total)
**Evidence:** lifecycle_test.go:53-67.
**Confidence:** HIGH

### BC-LIFECYCLE-004: Panic in cleanup is recovered; subsequent cleanups still run
**Evidence:** lifecycle_test.go:71-88. Source: lifecycle.go:75-82.
**Confidence:** HIGH

### BC-LIFECYCLE-005: Concurrent Register is safe; all functions tracked and called
**Postconditions:** 100 goroutines each call Register; all 100 are recorded; all 100 cleanups run on Close.
**Evidence:** lifecycle_test.go:92-117 (under `-race`).
**Confidence:** HIGH

### BC-LIFECYCLE-006: Close on empty Lifecycle is safe
**Evidence:** lifecycle_test.go:121-128.
**Confidence:** HIGH

### BC-LIFECYCLE-007: Register after Close is rejected silently
**Postconditions:** Function passed to Register-after-Close is never invoked.
**Evidence:** lifecycle_test.go:147-161. Source: lifecycle.go:36-39.
**Confidence:** HIGH

---

## TMUX (`internal/core/tmux`) [M+]

### BC-TMUX-CTL-001: ControlClient.SendKeys rejects targets containing spaces, semicolons, newlines
**Postconditions:** Returns error mentioning the unsafe character; nothing written to stdin.
**Evidence:** control.go:131-134 calls `validateControlTarget`; validateControlTarget rejects `\n \r ; ' '` (control.go:190-198).
**Confidence:** MEDIUM (validation function fully visible; corresponding test exists in control_test.go to verify).

### BC-TMUX-CTL-002: SendKeysLiteral escapes backslash and double-quote in payload
**Preconditions:** Text contains `\` or `"`.
**Postconditions:** Written as `\\` and `\"` respectively inside the `send-keys -l -t TARGET -- "..."` command.
**Evidence:** control.go:182-185.
**Confidence:** MEDIUM

### BC-TMUX-CTL-003: SendKeysLiteral rejects newline, carriage return, NUL in payload
**Postconditions:** Returns error `"literal text contains unsafe character %q"`. Caller is expected to use SendKeys with `"Enter"` for newlines.
**Evidence:** control.go:163-169.
**Confidence:** MEDIUM

### BC-TMUX-CTL-004: Query is serialized; FIFO order matches command order
**Preconditions:** Concurrent Query calls (control mode is single-stream).
**Postconditions:** Each %begin/%end response block is matched to the oldest pending query in FIFO order (control.go:298-311).
**Confidence:** MEDIUM. Tested in control_test.go (see file list).

### BC-TMUX-CTL-005: Query honors context cancellation; removes queued entry on cancel
**Postconditions:** When ctx is cancelled, the pending query is removed from the FIFO to prevent later %begin from being mis-routed to a leaked listener (control.go:361-377).
**Confidence:** MEDIUM

### BC-TMUX-CTL-006: Close kills the tmux process after a 3-second grace period
**Evidence:** control.go:233-238.
**Confidence:** MEDIUM

### BC-TMUX-CTL-007: PasteToPane is unsupported via control mode
**Postconditions:** Always returns an error (control.go:244-246) — caller must fall back to file-based `load-buffer`.
**Confidence:** HIGH (explicit)

### BC-TMUX-PIDWALK-001: FindWindowForPid walks the process tree to find the tmux window owning a PID
**Source:** core/tmux/pidwalk.go (referenced server.go:463, handler.go:124).
**Confidence:** MEDIUM (subject to deep verification in Pass B).

---

## CONFIG / HOOKS (`internal/core/config`) [M+]

### BC-HOOK-001: All 5 hook commands resolve the alive server via lock-file scan (PID-liveness check)
**Postconditions:** Each hook one-liner reads `~/.claude/ide/*.lock`, filters by `process.kill(pid, 0)` succeeding, picks the highest port (most recent). When no alive server is found:
  - PreToolUse hook echoes stdin unchanged and exits (allows the tool call to proceed).
  - Others exit silently.
**Evidence:** hooks.go:13-44 (5 const strings).
**Confidence:** HIGH (explicit code)

### BC-HOOK-002: Hooks are auth'd via `X-Claude-Code-Ide-Authorization` header
**Postconditions:** Token is read from the lock file; included in every POST.
**Evidence:** hooks.go:31, 35, 38, 41, 44 (all 5 hook one-liners).
**Confidence:** HIGH

### BC-HOOK-003: WriteHooksSettingsFile writes a JSON file with `SetEscapeHTML(false)`
**Postconditions:** The `=>`, `<`, `>`, `&` characters survive JSON encoding so node parses them literally.
**Evidence:** hooks.go:54-65.
**Confidence:** HIGH

### BC-HOOK-004: Hooks settings file is written to `<runtimeDir>/hooks-settings.json` with mode 0600
**Evidence:** hooks.go:70-74.
**Confidence:** HIGH

### BC-HOOK-005: Hook command bodies are tagged with hook type
- PreToolUse: `type: 'tool_info'`, path `/notify`, timeout 300ms
- Notification (permission_prompt only): no `type` field, path `/notify`, timeout 2000ms (long enough for user response)
- Stop: path `/stop`, timeout 300ms
- SessionStart: path `/session-start`, timeout 300ms
- UserPromptSubmit: path `/prompt-submit`, timeout 300ms
**Evidence:** hooks.go:31, 35, 38, 41, 44.
**Confidence:** HIGH

### BC-HOOK-006: The Notification hook fires ONLY when `i.notification_type === 'permission_prompt'`
**Postconditions:** Other notification types are dropped client-side.
**Evidence:** hooks.go:35 (`if(i.notification_type!=='permission_prompt')return;`).
**Confidence:** HIGH

---

## MCP SERVER (`internal/server`) [M+]

### BC-MCPSRV-001: /health requires no auth and returns API version + uptime + session count
**Evidence:** daemon (not server) has /health; server.go has only protocol endpoints.
For daemon: daemon/server_test.go:71-95 (`TestHealth_NoAuth`); daemon/server.go:701-709.
**Confidence:** HIGH (daemon)

### BC-MCPSRV-002: Token mismatch yields 401 Unauthorized
**Evidence:** daemon/server_test.go:97 (`TestAuth_Unauthorized`). Source: daemon/server.go:189-195, server.go:278-280.
**Confidence:** HIGH

### BC-MCPSRV-003: POST /notify with type "tool_info" stores PreToolUse pending data; emits ActivityRunning
**Postconditions:**
  1. `state.SetPending(window, PendingTool{ToolName, Input, CWD})` (server.go:412-416).
  2. `setActivity(window, ActivityRunning, toolName)` (server.go:418).
  3. `notifyBroker.Publish(Event{ActivityNotification})` (server.go:419-424).
**Evidence:** server.go:409-424.
**Confidence:** MEDIUM (source-derived; covered by server_test.go).

### BC-MCPSRV-004: POST /notify without type (= permission_prompt) dispatches ToolNotification
**Postconditions:** Either:
  - If a pending `DiffChoice` was set by an earlier openDiff popup, the cached key is sent via tmux SendKeys (server.go:428-438).
  - Else: build ToolNotification, detect MaxOption via `CapturePaneANSI` + `tmuxadapter.DetectMaxOption`, set Activity to NeedsInput, and dispatch via broker if any subscriber, else `notify.Enqueue` to disk (server.go:491-575).
**Evidence:** server.go:425-453, dispatchToolNotification at 491-575.
**Confidence:** MEDIUM (source-derived; partially tested in server_test.go).

### BC-MCPSRV-005: Notify falls back to LastPendingWindow when PID lookup fails (permission_prompt path)
**Preconditions:** No window matches `req.PID` via cache or pidwalk; not a tool_info request.
**Postconditions:** Uses `s.state.LastPendingWindow()` as the window (server.go:392-397). When still empty, returns 404.
**Evidence:** server.go:389-401.
**Confidence:** HIGH (explicit branch)

### BC-MCPSRV-006: Write tool notification extracts `file_path` + `content` from JSON tool_input → routes to DiffPopup
**Postconditions:** `OldFilePath` set to file_path, `NewContents` set to content (server.go:511-520).
**Evidence:** server.go:511-520.
**Confidence:** HIGH (explicit)

### BC-MCPSRV-007: Edit tool notification reads the existing file (up to 2 MB) and computes new contents
**Postconditions:**
  - Resolves relative `file_path` against `cwd`.
  - Only reads regular files ≤ 2 MB (skips FIFOs, devices, oversize).
  - Applies `strings.ReplaceAll` if `replace_all`, else `strings.Replace(..., 1)`.
  - On read failure or oversize: notification falls back to ToolPopup (n.OldFilePath remains empty).
**Evidence:** server.go:525-555.
**Confidence:** HIGH

### BC-MCPSRV-008: Broker subscriber present → broker delivery; else → disk enqueue
**Evidence:** server.go:562-574 (`s.notifyBroker.HasSubscribers()` branch).
**Confidence:** HIGH

### BC-MCPSRV-009: handleStop maps stop_reason "error" / "interrupt" → ActivityError; else ActivityIdle
**Evidence:** server.go:238-244 (`stopReasonToActivity`).
**Confidence:** HIGH

### BC-MCPSRV-010: Lock-file discovery via `~/.claude/ide/<port>.lock`; only the highest alive port survives
**Postconditions:** TUI startup calls `LockManager.CleanAllExcept(port)` to remove all other lazyclaude locks so hooks always find the in-process server (root.go:440-442).
**Evidence:** root.go:437-443 + server/lock.go.
**Confidence:** MEDIUM

### BC-MCPSRV-011: WebSocket initialize returns ProtocolVersion "2024-11-05"
**Evidence:** server/handler.go:72-83.
**Confidence:** HIGH

### BC-MCPSRV-012: ide_connected stores per-connection PID → tmux window mapping
**Evidence:** server/handler.go:89-115.
**Confidence:** HIGH

### BC-MCPSRV-013: openDiff path must be absolute; non-absolute returns -32602 invalid params
**Evidence:** server/handler.go:139-146, 156-159.
**Confidence:** HIGH

### BC-MCPSRV-014: WithBroker injects an external broker; ownership stays with caller
**Postconditions:** server.Stop does NOT close an externally-owned broker (so it survives server restart with GUI subscribers attached).
**Evidence:** server.go:73-78, 176-180.
**Confidence:** HIGH

### BC-MCPSRV-015: /msg/send rejects empty from, empty to, self-message, body > 10 KB
**Evidence:** server/handler_msg.go:209-228.
**Confidence:** HIGH

### BC-MCPSRV-016: /msg/send delivers via `SendKeysLiteral` + `SendKeys "Enter"` to `lazyclaude:<window>`
**Postconditions:** Message format: `"[MESSAGE from %s (%s)]\ntype: %s\n---\n%s\n"`.
**Evidence:** server/handler_msg.go:283-298, daemon/server.go:534-547.
**Confidence:** HIGH

### BC-MCPSRV-017: /msg/sessions returns sessions enriched with hook-based activity state
**Evidence:** server/handler_msg.go:423-425, 257-272.
**Confidence:** HIGH

### BC-MCPSRV-018: /msg/create requires `from` and `name`; type must be "worker" or "local"
**Postconditions:** Project is resolved from caller's session ID via `FindProjectForSession`. Worker is git-worktreed; local is plain at `project.Path`.
**Evidence:** server/handler_msg.go:114-141.
**Confidence:** HIGH

### BC-MCPSRV-019: /msg/create for local type sends the prompt via SendKeysLiteral + Enter when prompt non-empty
**Evidence:** server/handler_msg.go:153-160.
**Confidence:** HIGH

### BC-MCPSRV-020: validMsgTypes allowlist: review_request, review_response, status, done, issue
**Evidence:** server/handler_msg.go:379-386. Mirrored in cmd/lazyclaude/msg.go:13-19.
**Confidence:** HIGH

---

## DAEMON (`internal/daemon`) [M+]

### BC-DAEMON-001: APIVersion = 4 returned on /health
**Evidence:** daemon/api.go:25, server.go:701-709, server_test.go:84-94.
**Confidence:** HIGH

### BC-DAEMON-002: All endpoints except /health require `X-Daemon-Authorization`
**Evidence:** daemon/server.go:187-196 (`withAuth` middleware), 127 (note: `/health` not wrapped).
**Confidence:** HIGH

### BC-DAEMON-003: POST /session/create supports session_type ∈ {plain, worktree, pm, worker, ""}
**Postconditions:** Dispatches to `mgr.Create`, `CreateWorktreeOpts`, `CreatePMSessionOpts`, `CreateWorkerSessionOpts`. Empty type ≡ "plain". Any other value returns 400.
**Evidence:** daemon/server.go:227-258.
**Confidence:** HIGH

### BC-DAEMON-004: POST /session/create with plain type accepts but DOES NOT forward profile/options
**Postconditions:** "Phase 2b deferral" — fields are accepted but only forwarded for worktree/pm/worker types (daemon/server.go:229-232 explicit comment).
**Evidence:** daemon/server.go:228-232.
**Confidence:** HIGH (explicit doc + code)

### BC-DAEMON-005: DELETE /session/{id} returns 404 if not found, 204 on success
**Evidence:** daemon/server.go:280-289.
**Confidence:** HIGH

### BC-DAEMON-006: POST /session/{id}/scrollback uses tmux `capture-pane -p` ANSI range on the daemon's own tmux server
**Postconditions:** Returns 502 BadGateway on tmux error; the remote daemon owns the scrollback because the local mirror tmux buffer doesn't contain remote tmux history.
**Evidence:** daemon/server.go:322-346 (explicit comment 317-321).
**Confidence:** HIGH

### BC-DAEMON-007: GET /session/{id}/history-size parses tmux `#{history_size}` to int; non-integer → 502
**Evidence:** daemon/server.go:351-379.
**Confidence:** HIGH

### BC-DAEMON-008: /notifications is Server-Sent Events; sends an EventFullSync on connect, then streams broker events
**Postconditions:** Each frame: `id: <num>\nevent: <type>\ndata: <json>\n\n`. Subscriber cancelled on `r.Context().Done()` or `shutdownCh`.
**Evidence:** daemon/server_sse.go:17-66.
**Confidence:** HIGH

### BC-DAEMON-009: SSE broker→notification mapping
- `ActivityNotification` → `EventActivity` (Activity + ToolName + SessionID)
- `Notification` (tool) → `EventToolInfo` (full ToolNotification + SessionID)
- `StopNotification` → `EventActivity` (state derived from StopReason)
- `SessionStartNotification` → `EventActivity` ActivityRunning
- `PromptSubmitNotification` → `EventActivity` ActivityRunning
**Evidence:** daemon/server_sse.go:70-134.
**Confidence:** HIGH

### BC-DAEMON-010: SSE `sessionIDForWindow` translates raw tmux window IDs (e.g. "@22") to canonical session UUIDs before sending
**Postconditions:** Local subscribers can look up local mirror sessions by UUID (the authoritative key). Empty window → empty session_id. Unmatched window → empty session_id. "lc-<8>" prefix matches against session UUID prefix.
**Evidence:** daemon/server_sse.go:136-175.
**Confidence:** HIGH

### BC-DAEMON-011: POST /msg/send requires from + to non-empty, from ≠ to, recipient resolvable
**Postconditions:** Returns JSON body with `{"error": "..."}` and 400/404 status, NOT HTTP body text. (Different from server's /msg/send which uses http.Error text.)
**Evidence:** daemon/server.go:498-548.
**Confidence:** HIGH

### BC-DAEMON-012: /shutdown POSTs are auth-required; closes shutdownCh; the daemon process exits via its outer select loop
**Evidence:** daemon/server.go:713-724, daemon_cmd.go:108-119.
**Confidence:** HIGH

### BC-DAEMON-013: GET /profiles always returns HTTP 200; errors are encoded in ProfileListResponse.Error
**Postconditions:**
  - Config present + valid → `Profiles: [...]`, `Error: ""`
  - Config absent → `Profiles: [{builtin}]`, `Error: ""`
  - Config malformed → `Profiles: nil`, `Error: <human-readable>`
  - Home dir unavailable → `Profiles: nil`, `Error: "resolve home dir: ..."`
**Evidence:** daemon/server.go:617-655 (explicit doc).
**Confidence:** HIGH

### BC-DAEMON-014: Daemon emits `daemon.json` on stdout (JSON `{Port, Token}`) AND writes it to `<runtimeDir>/daemon.json` (mode 0600)
**Postconditions:** Local RemoteConnection parses the stdout JSON to bootstrap (no file-system dependency for first connect).
**Evidence:** daemon_cmd.go:102-104, daemon/server.go:761-766.
**Confidence:** HIGH

### BC-DAEMON-015: Daemon writes runtime dir with 0700, daemon.json with 0600
**Evidence:** daemon/server.go:758-766.
**Confidence:** HIGH

---

## ASKPASS (`internal/daemon/askpass.go`) [M+]

### BC-ASKPASS-001: AskpassServer socket path = `<runtimeDir>/askpass-<pid>.sock`, script path = `<runtimeDir>/askpass-<pid>.sh`
**Postconditions:** PID in path avoids collision across concurrent lazyclaude instances.
**Evidence:** askpass.go:42-49.
**Confidence:** HIGH

### BC-ASKPASS-002: Unix socket permissions are 0600 (owner-only)
**Evidence:** askpass.go:67-72.
**Confidence:** HIGH

### BC-ASKPASS-003: Wrapper script content = `"#!/bin/sh\nexec '<binPath>' askpass \"$@\"\n"`, mode 0700
**Evidence:** askpass.go:87-89.
**Confidence:** HIGH

### BC-ASKPASS-004: Read deadline is 10 seconds; prevents goroutine leak from idle clients
**Evidence:** askpass.go:15, 170.
**Confidence:** HIGH

### BC-ASKPASS-005: Handler invocations are serialized by handlerMu (only one GUI popup at a time)
**Evidence:** askpass.go:182-185.
**Confidence:** HIGH (explicit + tested in askpass_test.go)

### BC-ASKPASS-006: Cancellation (handler error) sends empty line back to SSH
**Evidence:** askpass.go:187-191.
**Confidence:** HIGH

### BC-ASKPASS-007: Stop closes all active connections and removes socket + script files
**Evidence:** askpass.go:97-118.
**Confidence:** HIGH

### BC-ASKPASS-008 (caller): root.go arms ExecSSHExecutor with askpass ONLY on full happy path
**Postconditions:** All four steps must succeed (Start, Executable, WriteScript, Register) — otherwise askpassScript stays empty so SSH falls back to BatchMode=yes.
**Evidence:** root.go:148-164.
**Confidence:** HIGH

### BC-ASKPASS-009: Top-level askpass request has 120-second handler timeout
**Postconditions:** Returns `"askpass timeout"` error if GUI does not respond.
**Evidence:** root.go:135-144.
**Confidence:** HIGH

---

## SSH / TUNNEL (`internal/daemon/{ssh,tunnel}.go`) [M+]

### BC-SSH-001: ExecSSHExecutor.Run uses ConnectTimeout=10, no ControlMaster, no ControlPath
**Postconditions:** No multiplexing surprises across concurrent invocations.
**Evidence:** daemon/ssh.go:51, 69.
**Confidence:** HIGH

### BC-SSH-002: BatchMode=yes is set ONLY when askpass is unavailable
**Evidence:** daemon/ssh.go:52-55, 70-72.
**Confidence:** HIGH

### BC-SSH-003: SplitHostPort handles "host", "host:port", "user@host", "user@host:port", "[ipv6]", "[ipv6]:port"
**Postconditions:** Returns ("host-part", "port" or "").
**Evidence:** daemon/ssh.go:88-120 + ssh_test.go.
**Confidence:** HIGH

### BC-SSH-004: SSHEnv injects SSH_ASKPASS, SSH_ASKPASS_REQUIRE=prefer, LAZYCLAUDE_ASKPASS_SOCK, and DISPLAY=:0 (only if DISPLAY unset)
**Evidence:** daemon/ssh.go:30-47.
**Confidence:** HIGH

### BC-TUNNEL-001: Tunnel uses base SSH args: `-N -a -o ServerAliveInterval=15 -o ServerAliveCountMax=3 -o ExitOnForwardFailure=yes -o ControlMaster=no -o ControlPath=none`
**Evidence:** daemon/tunnel.go:228-244.
**Confidence:** HIGH

### BC-TUNNEL-002: Tunnel picks a free local port via OS port-zero listen; warns about TOCTOU race
**Postconditions:** ExitOnForwardFailure=yes ensures the ssh process exits fast if the port is taken.
**Evidence:** daemon/tunnel.go:54-60, 191-202.
**Confidence:** HIGH

### BC-TUNNEL-003: Tunnel.Start waits up to 10s polling at 100ms for TCP connect to local port
**Postconditions:** If SSH exits before connect: returns wrapped error "SSH tunnel to %s exited before becoming ready". On context timeout: kills the SSH process. Marks the tunnel as failed so it can be retried.
**Evidence:** daemon/tunnel.go:108-139.
**Confidence:** HIGH

### BC-TUNNEL-004: Stop kills the SSH process and is safe on a not-started tunnel
**Evidence:** daemon/tunnel.go:142-154.
**Confidence:** HIGH

---

## SESSION (`internal/session`) [M+]

### BC-SESSION-001: Create generates a UUID and uses `WindowName() = "lc-" + ID[:8]` as the tmux window name
**Postconditions:** Session ID is a UUID v4; tmux window name has a 12-char prefix-form derived from the ID.
**Evidence:** session/manager.go:253 (uuid.New()), session/session.go (WindowName() — referenced root.go:625, daemon/server.go:269, 736).
**Confidence:** HIGH

### BC-SESSION-002: First Create creates the tmux session "lazyclaude"; subsequent create new windows in it
**Evidence:** manager_test.go:31-71 (`TestManager_Create_FirstSession`, `TestManager_Create_SecondSession`), manager.go:467-497.
**Confidence:** HIGH

### BC-SESSION-003: Session.Status enum: Running, Detached, Orphan, Dead (with String())
**Evidence:** Test asserts `session.StatusRunning` (manager_test.go:43); session/session.go enum.
**Confidence:** HIGH

### BC-SESSION-004: Delete returns error for unknown session ID
**Evidence:** manager_test.go:110-115 (`TestManager_Delete_NotFound`).
**Confidence:** HIGH

### BC-SESSION-005: Sync does not mark sessions Orphan on transient `HasSession` failures
**Postconditions:** A single `HasSession(ctx) == false` increments a fail counter; only the GC may eventually clean if appropriate. The explicit fix for "GC orphan delete bug (session-wipe under high load)" — see commit log.
**Evidence:** manager.go:147-160, syncFailThreshold = 3 (manager.go:28-29).
**Confidence:** HIGH

### BC-SESSION-006: EnsureClaudeConfigured idempotently writes `~/.claude.json` with onboarding skip flags
**Postconditions:** Sets `hasCompletedOnboarding: true`, `numStartups: 10`, adds project trust entries for the dirPath and "/" with `hasTrustDialogAccepted: true`. Skips write if already set. JSON file I/O only.
**Evidence:** manager.go:186-222.
**Confidence:** HIGH (called from root.go:95 unconditionally on startup)

### BC-SESSION-007: ResolveProfile: empty name → effective default (Default=true | named "default" | builtin); known name → exact match; unknown non-builtin name → error
**Evidence:** manager.go:95-116.
**Confidence:** HIGH (explicit doc comment + code)

### BC-SESSION-008: Built-in default profile name persists as `""` in state.json (not "default")
**Postconditions:** "so that resume uses whatever the user's current default is, rather than pinning to 'default'."
**Evidence:** manager.go:290-298 (`profileNameForPersist`).
**Confidence:** HIGH

### BC-SESSION-009: CreateWorktreeOpts validates name unless SkipGitAdd (= resume); creates `.lazyclaude/worktrees/<name>/` under projectRoot via `git worktree add`
**Postconditions:** Worker role assigned. Worktree path matches `WorktreePath(projectRoot, name)`.
**Evidence:** manager.go:314-360 (`createWorktreeSession`), session/worktree.go (`CreateWorktreeWithRunner`, `WorktreePath`, `ValidateWorktreeName`).
**Confidence:** HIGH

### BC-SESSION-010: Resume session fallback: if not in store but worktree dir exists, ResumeWorktreeOpts succeeds via `SkipGitAdd: true`
**Postconditions:** Reuses the existing worktree directory without rerunning `git worktree add`.
**Evidence:** manager.go:438-449 (ResumeWorktreeOpts → SkipGitAdd=true; manager.go:322-331 verifies path existence).
**Confidence:** HIGH

### BC-SESSION-011 (PMW): PM/Worker roles encoded as session.Role ∈ {RoleNone, RolePM, RoleWorker}
**Evidence:** session/role.go (referenced manager.go:306, 316, 419, daemon/server.go:270).
**Confidence:** MEDIUM (deep dive in PMW pass)

### BC-SESSION-012: SetProfiles is concurrency-safe via RWMutex; copies the input slice
**Evidence:** manager.go:65-73.
**Confidence:** HIGH

### BC-SESSION-013: GC runs every 2s, calls Sync, and removes Dead sessions; does NOT delete Orphan
**Postconditions:** Explicit fix per commit "fix(gc): do not delete Orphan sessions".
**Evidence:** root.go:122-124, gc.go (referenced).
**Confidence:** HIGH (commit log + code)

---

## REMOTE (`internal/daemon`, `cmd/lazyclaude/mirror.go`) [M+]

### BC-REMOTE-001: RemoteProvider.StartSSE replaces any previous SSE goroutine before starting a new one
**Postconditions:** No goroutine accumulation across reconnects.
**Evidence:** daemon/remote_provider.go:132-161.
**Confidence:** HIGH

### BC-REMOTE-002: handleSSEEvent for EventActivity updates the cached session AND forwards to the local broker via callback, with `mirrorWindow` remap
**Postconditions:** `onSSEActivity(model.Event{ActivityNotification: ...}, sessionID)` is invoked; the callback in root.go rewrites Window using the local store before publishing.
**Evidence:** daemon/remote_provider.go:197-238 + root.go:822-835.
**Confidence:** HIGH

### BC-REMOTE-003: handleSSEEvent for EventToolInfo invokes `onSSEToolInfo` BEFORE buffering for PendingNotifications
**Postconditions:** Window is rewritten in place using sessionID hop into local store BEFORE the notification is queued.
**Evidence:** daemon/remote_provider.go:218-228 + root.go:861-870.
**Confidence:** HIGH (load-bearing for Bug 5)

### BC-REMOTE-004: handleSSEEvent for EventFullSync tags every session's `Host` with the provider's host
**Evidence:** daemon/remote_provider.go:229-237.
**Confidence:** HIGH

### BC-REMOTE-005: PendingNotifications is destructive (returns + clears the buffer)
**Evidence:** daemon/remote_provider.go:689-699.
**Confidence:** HIGH

### BC-REMOTE-006: CapturePreview, SendChoice on RemoteProvider return errors — these go through local mirror tmux instead
**Evidence:** daemon/remote_provider.go:365-374.
**Confidence:** HIGH (explicit)

### BC-REMOTE-007: AttachSession on remote = SSH-interactive + grouped tmux session (`-t lazyclaude -s attach-$$`) + `destroy-unattached`
**Postconditions:** Multiple SSH attaches do not override each other's active window selection. Group is destroyed on SSH drop.
**Evidence:** daemon/remote_provider.go:442-463.
**Confidence:** HIGH (explicit doc + code)

### BC-REMOTE-008: SSH commands are wrapped in base64 to avoid shell-quoting issues
**Postconditions:** Form: `eval "$(echo <base64> | base64 -d)"`.
**Evidence:** daemon/remote_provider.go:431-435; .claude/CLAUDE.md ("Remote commands are written as plain bash scripts and base64-encoded").
**Confidence:** HIGH

### BC-REMOTE-009: CompositeProvider.AddRemote registers a named provider; dispatch is host-based
**Source:** daemon/composite_provider.go.
**Confidence:** MEDIUM (depth in Pass B)

### BC-MIRROR-001: MirrorManager creates a local tmux window that, on attach, exec's an SSH attach back to the remote daemon's tmux server
**Source:** cmd/lazyclaude/mirror.go.
**Confidence:** MEDIUM (depth in Pass B)

### BC-MIRROR-002: MirrorManager.RestoreExisting builds mirrors only for Running remote sessions; dead/orphan skipped
**Evidence:** root.go:240-249.
**Confidence:** HIGH

---

## GUI (`internal/gui`) [M+]

### BC-GUI-POPUP-001: PopupController stack is LIFO with focus tracking
**Postconditions:** PushPopup adds and focuses; DismissActive removes the focused entry and refocuses (LIFO order); SuspendAll/UnsuspendAll for toggle behavior.
**Evidence:** gui/popup_controller.go (full file) + popup_controller_test.go.
**Confidence:** HIGH

### BC-GUI-POPUP-002: ActiveNotification returns nil for non-tool popups (defensive)
**Evidence:** gui/popup_controller.go:201-211.
**Confidence:** HIGH

### BC-GUI-POPUP-003: DismissAll returns the list of window IDs so caller can SendChoice to each
**Evidence:** popup_controller.go:121-131.
**Confidence:** HIGH

### BC-GUI-FS-001: FullScreenState.EnqueueKey is non-blocking; drops keys when queue full (cap 1024)
**Evidence:** fullscreen.go:84-89, keyQueueSize const (fullscreen.go:9).
**Confidence:** HIGH

### BC-GUI-FS-002: Adjacent literal key commands are batched into a single SendKeysLiteral call
**Postconditions:** Performance optimization for paste bursts.
**Evidence:** fullscreen.go:120-155 (`dispatchBatch`).
**Confidence:** HIGH

### BC-GUI-FS-003: Enter/Exit clears scrollY and invalidates PreviewCache
**Evidence:** fullscreen.go:58-80.
**Confidence:** HIGH

### BC-GUI-KEYS-001: Key dispatch order (per .claude/CLAUDE.md):
  1. View-specific bindings (e.g. popupViewName)
  2. Editor.Edit() — only for views with Editable=true
  3. Global bindings — but rune keys (ch != 0) skip global bindings on Editable views
**Evidence:** .claude/CLAUDE.md "Editor and keybinding dispatch order"; gui/keydispatch/dispatcher.go.
**Confidence:** HIGH (documented)

### BC-GUI-KEYS-002: `Ctrl+\` is the normal-mode toggle (not Esc, which is indistinguishable from `Ctrl+[`)
**Evidence:** .claude/CLAUDE.md "Ctrl+[ and Esc"; README.md:170-172 (Fullscreen mode binding).
**Confidence:** HIGH (documented + observable in README)

### BC-GUI-PASTE-001: Bracketed paste is aggregated at pollEvent level into a single `eventPasteContent`
**Postconditions:** Structurally prevents gEvents channel overflow (cap 20). ESC[200~ fallback works around tmux display-popup not delivering EventPaste.
**Evidence:** .claude/CLAUDE.md "Paste handling"; gui/app.go:193-197 (OnPasteContent callback wiring).
**Confidence:** HIGH (documented + code)

### BC-GUI-SIDEBAR-001: Activity priority for sidebar icon
  - Priority 1: window activity from broker events (NeedsInput, Running, Idle, Error)
  - Priority 2: pending permission popup overrides to NeedsInput (file-polling fallback)
**Evidence:** root.go:606-622 (`sessionToItem`).
**Confidence:** HIGH

### BC-GUI-SIDEBAR-002: On popup dismiss, NeedsInput → Running immediately
**Evidence:** .claude/CLAUDE.md "Activity state (5-stage)".
**Confidence:** MEDIUM (documented behavior; code path in gui/keyhandler/popup.go)

### BC-GUI-ACTIVITY-001: 5-stage activity icons: `?` Running, `!` NeedsInput, `✓` Idle, `✗` Error, `×` Dead (per README), `?` (gray) for Unknown
**Evidence:** README.md:35; gui rendering code (presentation/style.go).
**Confidence:** HIGH

---

## CLI

### BC-CLI-001: `lazyclaude` default subcommand opens TUI; `--debug` redirects logs; `--log-file` overrides destination
**Evidence:** root.go:31, 380-382.
**Confidence:** HIGH

### BC-CLI-002: `--debug` and `--log-file` are PersistentFlags (visible on all subcommands)
**Evidence:** Per commit log: `fix(cli): make --debug and --log-file persistent flags (P4-A prep)`. root.go:380-382 confirms `PersistentFlags()`.
**Confidence:** HIGH

### BC-CLI-003: `lazyclaude sessions` lists via the MCP server's `/msg/sessions`, NOT the daemon's `/sessions`
**Evidence:** sessions.go:14-52 uses `server.DiscoverServer + server.NewClient`; not daemon discovery.
**Confidence:** HIGH

### BC-CLI-004: `lazyclaude msg send` accepts --type ∈ {review_request, review_response, status, done, issue}; --from defaults to "cli"
**Evidence:** msg.go:13-19, 46-48, 67-68.
**Confidence:** HIGH

### BC-CLI-005: `lazyclaude msg create --name <required>`; --type ∈ {worker, local}; --profile and --options forwarded
**Evidence:** msg.go:88-91, 117-123.
**Confidence:** HIGH

### BC-CLI-006: `lazyclaude daemon` writes JSON `{Port, Token}` to stdout for the local TUI to consume
**Evidence:** daemon_cmd.go:102-104.
**Confidence:** HIGH

### BC-CLI-007 (PMW): `lazyclaude msg create --type worker` creates a git-worktree session whose project is resolved from --from session's project
**Evidence:** server/handler_msg.go:124-141. Also documented as PM/Worker feature in README.md:61-67.
**Confidence:** HIGH (PMW)

---

## PROFILE (`internal/profile`) [M+]

### BC-PROFILE-001: Load reads `<configPath>` (typically `~/.lazyclaude/config.json`); absent config → empty list (no error); malformed → error
**Source:** profile/profile.go (referenced root.go:88-89, daemon/server.go:641-654).
**Confidence:** MEDIUM (depth in Pass B)

### BC-PROFILE-002: ResolveDefault picks first Default=true, else profile named "default", else BuiltinDefault
**Source:** Referenced session/manager.go:104.
**Confidence:** MEDIUM

### BC-PROFILE-003: BuiltinDefaultName is a reserved name; users cannot redefine it
**Source:** session/manager.go:111-112 checks `trimmed == profile.BuiltinDefaultName`.
**Confidence:** MEDIUM

### BC-PROFILE-004: profile.Args is the canonical way to pass args containing spaces; `--options` does NOT support quoted args
**Evidence:** session/manager.go:370-372 explicit doc: `"Quoted arguments with internal spaces are NOT supported; use profile.Args for that."`.
**Confidence:** HIGH

---

## Confidence Summary

| Subsystem | High | Medium | Low |
|---|---|---|---|
| BROKER | 12 | 0 | 0 |
| LIFECYCLE | 7 | 0 | 0 |
| TMUX | 1 | 6 | 0 |
| HOOK | 6 | 0 | 0 |
| MCP SERVER | 11 | 3 | 0 |
| DAEMON | 15 | 0 | 0 |
| ASKPASS | 9 | 0 | 0 |
| SSH/TUNNEL | 7 | 0 | 0 |
| SESSION | 11 | 2 | 0 |
| REMOTE | 7 | 3 | 0 |
| GUI | 6 | 1 | 0 |
| CLI | 7 | 0 | 0 |
| PROFILE | 1 | 3 | 0 |

**Gaps to address in deepening:**
- TMUX exec adapter (BC-TMUX-CTL pure-source; tests exist but not yet read)
- GUI popup_types, render flow, presentation modules
- session/store.go, session/gc.go, session/worktree.go internals
- cmd/lazyclaude/mirror.go, gui_adapter.go, local_provider.go (composition glue)
- profile package internal logic
- mcp manager / plugin manager (less critical for monocle but in scope)
- PMW prompts content & flow (single pass)

## State Checkpoint

```yaml
pass: 3
status: complete
contracts_drafted: 100+
files_scanned: ~10 additional test/source files
timestamp: 2026-05-11T18:15:00Z
next_pass: 4
```
