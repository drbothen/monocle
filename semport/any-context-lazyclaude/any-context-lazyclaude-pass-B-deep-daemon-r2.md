# Pass B Deep: `internal/daemon` — Round 2

**Scope:** server.go body (handler implementations), server_sse.go (full), remote_provider.go body (SSE consumer + SessionProvider methods), lifecycle.go.

**Files read in full this round:** server_sse.go (188 LOC), lifecycle.go (84). server.go (full now, tail read 200-784). remote_provider.go (full now, tail read 200-699).

## Server handler implementations

### Session CRUD

`handleSessionCreate` (server.go:216-272) dispatches by `req.SessionType`:

| session_type | Manager call | Profile/Options forwarded? |
|---|---|---|
| "plain", "" | `mgr.Create(ctx, path)` | **NO** (Phase 2b deferral, server.go:229-231) |
| "worktree" | `mgr.CreateWorktreeOpts(ctx, WorktreeOpts{...})` | yes |
| "pm" | `mgr.CreatePMSessionOpts(ctx, PMOpts{...})` | yes |
| "worker" | `mgr.CreateWorkerSessionOpts(ctx, WorkerOpts{...})` | yes |
| other | 400 BadRequest | n/a |

### BC-DAEMON-SRV-007: readJSON enforces 1 MB max body size (MaxBytesReader)
**Postconditions:** Larger requests are truncated and decode fails. Hard limit applies to ALL request handlers using readJSON.
**Evidence:** server.go:209-212.
**Confidence:** HIGH

### BC-DAEMON-SRV-008: handleSessionCreate `"plain"` type explicitly drops Profile/Options (Phase 2b deferral)
**Postconditions:** Schema-compatible; the fields parse but are ignored. Worktree/PM/Worker forward them.
**Evidence:** server.go:227-232 + explicit comment.
**Confidence:** HIGH — confirms BC-DAEMON-004 from Pass 3.

### BC-DAEMON-SRV-009: handleSessionDelete returns 404 when error contains "not found", else 500
**Postconditions:** String-match-based status code mapping. Fragile if error wording changes.
**Evidence:** server.go:280-288.
**Confidence:** HIGH — minor: should use sentinel errors.

### BC-DAEMON-SRV-010: handleMsgSend returns JSON body with `Error:` field at 400/404/502; success returns `Delivered:true`
**Postconditions:** Different from server (MCP) /msg/send which uses `http.Error` text. The JSON shape preserves client parsing.
**Evidence:** server.go:492-549.
**Confidence:** HIGH — confirms BC-DAEMON-011 from Pass 3.

### BC-DAEMON-SRV-011: handleMsgSend message format: `"[MESSAGE from %s (%s)]\ntype: %s\n---\n%s\n"` where %s args are senderName, From-ID, Type, Body
**Postconditions:** Format matches BC-MCPSRV-016. Both message paths produce identical text.
**Evidence:** server.go:535-536; mirrors server/handler_msg.go pattern (Pass 3).
**Confidence:** HIGH

### BC-DAEMON-SRV-012: handleMsgSend dispatches via SendKeysLiteral then SendKeys("Enter"); the Enter error is logged but not returned (line 544-546)
**Postconditions:** If literal succeeds but Enter fails, the message text is in the pane but unsubmitted. Caller sees `Delivered:true`. **Minor race**: user sees partial text.
**Evidence:** server.go:539-548.
**Confidence:** HIGH — observability gap.

### BC-DAEMON-SRV-013: handleMsgCreate supports type ∈ {"worker", "pm"}; other returns 400
**Postconditions:** **DIFFERS from BC-MCPSRV-018** which supports "worker" or "local". This daemon path explicitly does NOT support "local" (server.go:573-591 has only worker/pm cases). This is intentional because remote-daemon-driven /msg/create is for inter-session communication on the same host.
**Evidence:** server.go:573-590.
**Confidence:** HIGH — **NEW finding**: behavioral divergence between daemon and server MCP /msg/create.

### BC-DAEMON-SRV-014: handleMsgCreate project is resolved from `req.From` (caller) via `Store().FindProjectForSession`; required
**Evidence:** server.go:563-567.
**Confidence:** HIGH

### BC-DAEMON-SRV-015: handleProfiles always returns 200; daemon-side errors are encoded in ProfileListResponse.Error
**Postconditions:** Confirms BC-DAEMON-013. Four documented error/success states (server.go:621-626).
**Evidence:** server.go:633-655 + multi-paragraph doc 616-632.
**Confidence:** HIGH

### BC-DAEMON-SRV-016: handleProfiles uses `os.UserHomeDir()` which resolves to the remote user's home (because the daemon runs on remote)
**Postconditions:** This is the canonical way to discover remote profiles. The doc explicitly notes this.
**Evidence:** server.go:617-619.
**Confidence:** HIGH

### BC-DAEMON-SRV-017: handleProfiles security note — ProfileDefAPI.Env carries raw env vars; any authenticated client can read them
**Evidence:** server.go:628-632 + Pass 5 (BC-DAEMON-013 disposition).
**Confidence:** HIGH

### BC-DAEMON-SRV-018: handleCWD falls back to daemon's `os.Getwd()` if `detectUserShellCWD()` fails
**Postconditions:** Always returns a CWD on success (200 + CWDResponse). Returns 500 with `{"error": ...}` if Getwd also fails.
**Evidence:** server.go:686-697.
**Confidence:** HIGH

### BC-DAEMON-SRV-019: handleShutdown returns 200 then closes shutdownCh; the daemon process exits via daemon_cmd.go select loop
**Postconditions:** Idempotent via `if !s.shutdown` guard.
**Evidence:** server.go:713-724.
**Confidence:** HIGH — confirms BC-DAEMON-012.

### BC-DAEMON-SRV-020: writeDaemonInfo creates the runtime dir with 0o700 and daemon.json with 0o600
**Postconditions:** Owner-only access. Token is in daemon.json.
**Evidence:** server.go:753-767.
**Confidence:** HIGH — confirms BC-DAEMON-015.

### BC-DAEMON-SRV-021: GenerateDaemonToken returns 16 random bytes hex-encoded (32 hex chars)
**Evidence:** server.go:777-784.
**Confidence:** HIGH

## SSE emission

`handleSSE` (server_sse.go:17-66) is the daemon's SSE handler:

1. Set SSE headers (text/event-stream, no-cache, keep-alive).
2. Send initial `EventFullSync` with all sessions.
3. Subscribe to broker with **buffer 64**.
4. Select loop on (ctx.Done, shutdownCh, sub.Ch).
5. For each broker event, convert via `brokerEventToNotification`, then `writeSSEEvent`.

### BC-DAEMON-SSE-001: Broker subscription buffer is 64 (larger than GUI's 8)
**Postconditions:** SSE clients tolerate larger burst before drop. This reduces drop probability for remote forwarding compared to the GUI's local broker subscription.
**Evidence:** server_sse.go:44.
**Confidence:** HIGH — confirms the "burst tolerance" claim in Pass 6 seed 3 from the daemon side, while GUI side is 8.

### BC-DAEMON-SSE-002: brokerEventToNotification handles 5 event variants
- ActivityNotification → EventActivity
- Notification (tool) → EventToolInfo (carries the full ToolNotification copy)
- StopNotification → EventActivity (stop_reason → Activity)
- SessionStartNotification → EventActivity (ActivityRunning)
- PromptSubmitNotification → EventActivity (ActivityRunning)
**Evidence:** server_sse.go:70-134.
**Confidence:** HIGH — confirms BC-DAEMON-009.

### BC-DAEMON-SSE-003: SessionID translation via sessionIDForWindow on every event before SSE emission
**Postconditions:** Raw tmux window IDs ("@22") or canonical names ("lc-abcd1234") are normalized to UUID. Empty/unmatched → empty string (BC-DAEMON-010 confirmed).
**Evidence:** server_sse.go:78, 88, 110, 119, 128.
**Confidence:** HIGH

### BC-DAEMON-SSE-004: sessionIDForWindow has two passes: exact match (TmuxWindow == w || WindowName() == w), then prefix-match against `lc-<8>` hint
**Postconditions:** The prefix-match is a fallback for legacy callers. UUID prefix uniqueness handles uniqueness (Pass 6 seed 11).
**Evidence:** server_sse.go:158-174.
**Confidence:** HIGH

### BC-DAEMON-SSE-005: sseEventID is `atomic.Uint64` incremented per event; format `%d`
**Postconditions:** Monotonic IDs across the daemon's lifetime. Clients can use Last-Event-ID for replay — but the daemon does NOT honor that header (Gap-VER-007 from Pass 4).
**Evidence:** server.go:55, server_sse.go:177-179.
**Confidence:** HIGH

### BC-DAEMON-SSE-006: SSE frame format: `id: %s\nevent: %s\ndata: %s\n\n`
**Evidence:** server_sse.go:187.
**Confidence:** HIGH

### BC-DAEMON-SSE-007: writeSSEEvent's JSON marshal error is logged but not surfaced; client sees no frame for the dropped event
**Postconditions:** Silent loss for malformed events. Acceptable because the daemon controls construction.
**Evidence:** server_sse.go:182-187.
**Confidence:** HIGH

## RemoteProvider — SSE consumption + SessionProvider methods

### BC-DAEMON-RP-001: handleSSEEvent for EventActivity scans cached sessions to find match by ID or ID-prefix
**Postconditions:** Allows the daemon to emit either full UUID or short prefix as SessionID. On match: updates session.Activity and ToolName in cache, then invokes onSSEActivity callback with the remapped mirror window.
**Evidence:** remote_provider.go:198-217.
**Confidence:** HIGH

### BC-DAEMON-RP-002: handleSSEEvent for EventToolInfo invokes onSSEToolInfo callback BEFORE buffering in `rp.notifications`
**Postconditions:** Callback mutates the notification IN PLACE (e.g., to rewrite Window). The mutated notification is then appended to the pending buffer.
**Evidence:** remote_provider.go:218-228.
**Confidence:** HIGH — confirms BC-REMOTE-003.

### BC-DAEMON-RP-003: handleSSEEvent for EventFullSync replaces `rp.sessions` entirely, with `Host` tagged
**Postconditions:** No merge with existing data — full overwrite. Anchor for "what's currently on the remote".
**Evidence:** remote_provider.go:229-237.
**Confidence:** HIGH — confirms BC-REMOTE-004.

### BC-DAEMON-RP-004: HasSession iterates over the cached `rp.sessions` slice; O(N)
**Postconditions:** For typical session counts (<100) this is fine. Cache is the source of truth — no remote round-trip on HasSession.
**Evidence:** remote_provider.go:242-251.
**Confidence:** HIGH

### BC-DAEMON-RP-005: Sessions() always issues a remote round-trip; mutates `rp.sessions` cache with tagged copy
**Postconditions:** Each call refreshes the cache. Concurrent Sessions() calls would race on the cache write but the final state is well-defined.
**Evidence:** remote_provider.go:257-276.
**Confidence:** HIGH

### BC-DAEMON-RP-006: Create maps to CreateSession (returns full response) which calls addToCache after success
**Postconditions:** addToCache (composite_provider.go:475 from r1) keeps the cache coherent without waiting for SSE full_sync. Explained at remote_provider.go:467-477 comment.
**Evidence:** remote_provider.go:280-307.
**Confidence:** HIGH

### BC-DAEMON-RP-007: SendChoice and CapturePreview both return errors on RemoteProvider — these go through the local mirror instead
**Postconditions:** Confirms BC-REMOTE-006. The errors are explicit "not supported on remote provider (use mirror window)".
**Evidence:** remote_provider.go:364-374.
**Confidence:** HIGH

### BC-DAEMON-RP-008: resolveTmuxTarget prefers cached `TmuxWindow`; falls back to reconstructed `"lazyclaude:lc-<id[:8]>"`
**Postconditions:** Identical fallback to GUI's resolveSessionTarget (BC-GUI-FWD-001). The two layers agree on the convention.
**Evidence:** remote_provider.go:349-362.
**Confidence:** HIGH

### BC-DAEMON-RP-009: LaunchLazygit uses shell.Quote inside a remote command — wrapped by base64 in runSSHInteractive
**Postconditions:** Same pattern as buildTmuxAttachCommand. The shell.Quote operates inside the base64-decoded bash, not at the outer SSH layer. P0-VERIFICATION-001 from r1 applies here too.
**Evidence:** remote_provider.go:421-424.
**Confidence:** HIGH

### BC-DAEMON-RP-010: PendingNotifications returns AND clears the buffer (destructive read)
**Postconditions:** Confirms BC-REMOTE-005. Each caller drains the buffer for its consumer.
**Evidence:** (referenced in r1 deepening; the actual implementation is at remote_provider.go:689-699 per Pass 3 citation).
**Confidence:** HIGH

## LifecycleManager — remote daemon orchestration

`LifecycleManager` (lifecycle.go full file) is a thin wrapper around `SSHExecutor` for daemon discovery/start/stop on the remote.

### BC-DAEMON-LIFE-001: StartRemoteDaemon shell command tries `command -v lazyclaude || echo $HOME/.local/bin/lazyclaude`
**Postconditions:** PATH discovery first, then fallback to the install.sh default location.
**Evidence:** lifecycle.go:32-33.
**Confidence:** HIGH

### BC-DAEMON-LIFE-002: StartRemoteDaemon polls 20 times at 0.5s intervals (= 10 seconds max wait) for daemon.json existence on remote
**Postconditions:** If daemon hasn't written daemon.json within 10s, returns error "lazyclaude is not installed on %s". Misleading: the daemon may exist but be slow to write daemon.json.
**Evidence:** lifecycle.go:34-35 (`for i in $(seq 1 20); do sleep 0.5 && [ -f ... ] && cat ... && exit 0; done`).
**Confidence:** HIGH — minor: error message confounds "not installed" with "slow start" cases.

### BC-DAEMON-LIFE-003: StartRemoteDaemon launches with `nohup ... > /tmp/lazyclaude-daemon.log 2>&1 &`
**Postconditions:** Daemon survives SSH disconnect. Log goes to a fixed path on remote.
**Evidence:** lifecycle.go:32-33.
**Confidence:** HIGH

### BC-DAEMON-LIFE-004: DiscoverRemoteDaemon reads `/tmp/lazyclaude-$(whoami)/daemon.json` on the remote via SSH+cat
**Postconditions:** Uses $(whoami) on remote so the path matches whichever user owns the daemon process. Validates Port != 0 and Token != "".
**Evidence:** lifecycle.go:60-83.
**Confidence:** HIGH

### BC-DAEMON-LIFE-005: StopRemoteDaemon runs `lazyclaude daemon stop` via SSH
**Postconditions:** Note: there is no `daemon stop` subcommand visible in Pass 0's CLI inventory. This may be unsupported or the command may exit non-zero. **Possibly unimplemented or evolving.**
**Evidence:** lifecycle.go:52-58.
**Confidence:** HIGH — **NEW finding**: `lazyclaude daemon stop` may not be a real subcommand. Pass 0's cmd inventory shows only `daemon` (no stop sub-sub-command). Verify in next round or flag for the porter.

## Cross-pass observation

The daemon implements **two** flavors of /msg/create (worker, pm) while the MCP server implements (worker, local). This is documented as intentional for different topologies but means schema-drift risk. P2 — porter should treat them as separate APIs.

## Delta Summary

- New items added: 36 (15 BC-DAEMON-SRV-007..021, 7 BC-DAEMON-SSE-001..007, 10 BC-DAEMON-RP-001..010, 4 BC-DAEMON-LIFE-001..005 + minor)
- Existing items refined: BC-DAEMON-013 confirmed in code (4 states), BC-DAEMON-011 confirmed, BC-DAEMON-009 confirmed (5 variants → EventActivity/EventToolInfo)
- Remaining gaps: tunnel.go (244 LOC), askpass.go (194 LOC — partially covered by BC-ASKPASS in Pass 3), ssh.go (~270 LOC — partial in Pass 3), api.go (354 LOC — wire types), connection.go (interface defs), proc_cwd_linux.go (262 LOC), proc_cwd_other.go, paths.go, debug.go.

## Novelty Assessment

Novelty: SUBSTANTIVE

Justification: 36 new contracts, including:
- **BC-DAEMON-SRV-013** behavioral divergence between daemon and server /msg/create types (worker/pm vs worker/local) — **new finding**, not previously documented.
- **BC-DAEMON-LIFE-005** `lazyclaude daemon stop` may not exist as a subcommand — verification needed.
- **BC-DAEMON-SRV-009** string-based 404 detection (fragile).
- **BC-DAEMON-SSE-001** broker subscription buffer 64 (vs GUI's 8) — different drop tolerance.
- **BC-DAEMON-SRV-012** SendKeys("Enter") error is logged not returned (partial-deliver race).
- **BC-DAEMON-LIFE-002** 10s daemon-start timeout with misleading error.

These materially change the porter's understanding of error handling, schema drift, and timing.

## Convergence Declaration

Another round needed — tunnel.go, askpass.go, ssh.go bodies, api.go wire types, connection.go interfaces, proc_cwd_linux.go all unread. These contain (a) wire-type definitions important for the porter, (b) SSH/tunnel error semantics not yet fully captured, (c) askpass UDS protocol details only partially covered in Pass 3.

## State Checkpoint

```yaml
pass: B
subsystem: daemon
round: 2
status: complete
files_read_full: [server.go, server_sse.go, remote_provider.go, lifecycle.go]
contracts_drafted: 36
timestamp: 2026-05-11T21:05:00Z
novelty: SUBSTANTIVE
next_round: 3
```
