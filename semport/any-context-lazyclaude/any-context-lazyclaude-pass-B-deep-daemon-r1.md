# Pass B Deep: `internal/daemon` — Round 1

**Scope:** composite_provider.go, connection_impl.go, http_client.go, capture_preview.go, server.go head, remote_provider.go head.

**Source LOC:** 9,153 total in `internal/daemon/`. Files read in full this round: composite_provider.go (561 LOC), connection_impl.go (386), capture_preview.go (42), http_client.go (351). Partial: remote_provider.go (200/699), server.go (200/784).

## CompositeProvider — host-based dispatch

`CompositeProvider` (composite_provider.go:89-561) merges a local SessionProvider with N named remote SessionProviders. The TUI sees a single `SessionProvider` interface; routing is internal.

### Concurrency model
- `local` is immutable after construction → safe to read without mutex.
- `remotes`, `staleCache`, `profilesCache`, `profilesError` are guarded by `mu sync.RWMutex`.
- Pattern: RLock to collect snapshots, drop lock, then optional Lock to apply cache updates with **provider-pointer identity re-check** (composite_provider.go:202-213). This is the dual-lock pattern guarding against concurrent RemoveRemote.

### BC-DAEMON-COMP-001: Composition of small interfaces (SessionLister, SessionMutator, PreviewProvider, SessionActioner, WorktreeProvider, RoleSessionProvider, ConnectionAware) into the full SessionProvider
**Postconditions:** Smaller interfaces are test-friendly. The full interface is the dispatch boundary.
**Evidence:** composite_provider.go:14-82.
**Confidence:** HIGH

### BC-DAEMON-COMP-002: Sessions() collects local + tagged remote sessions; disconnected remotes return staleCache
**Postconditions:** Disconnected remotes show last-known sessions (instead of disappearing). On reconnect, the cache is refreshed. Race-safe via dual-lock + pointer-identity check.
**Evidence:** composite_provider.go:156-216.
**Confidence:** HIGH

### BC-DAEMON-COMP-003: Pointer-identity check (`c.remotes[u.host] != u.provider`) before writing staleCache prevents ghost entries
**Postconditions:** If RemoveRemote ran between RLock release and Lock acquire, the update is discarded. Without this, a stale Sessions() result for a removed host would re-introduce ghosts.
**Evidence:** composite_provider.go:201-213 + comment 196-200.
**Confidence:** HIGH — load-bearing concurrency design.

### BC-DAEMON-COMP-004: Profiles() caches per-host; error responses cache for the lifetime of the connection
**Postconditions:** RemoveRemote + AddRemote required to re-fetch after fixing remote config. Documented at composite_provider.go:107-115.
**Evidence:** composite_provider.go:237-282.
**Confidence:** HIGH

### BC-DAEMON-COMP-005: providerForSession routes by HasSession check; local checked first
**Postconditions:** Returns nil if no provider claims the session. Routing is asymmetric: for mirror sessions, providerForSession returns local (because the local store has them); providerForCapture returns the actual remote.
**Evidence:** composite_provider.go:456-468.
**Confidence:** HIGH

### BC-DAEMON-COMP-006: providerForCapture asks local.LocalSessionHost(id) first to dispatch capture by true host (not by store)
**Postconditions:** Critical for scrollback: the local mirror window's tmux buffer doesn't contain the remote tmux's historical scrollback. CaptureScrollback and HistorySize use providerForCapture; CapturePreview does NOT (preview works via the mirror window already).
**Evidence:** composite_provider.go:316-336 (CapturePreview/Scrollback/History routing), 471-510 (providerForCapture impl + comment).
**Confidence:** HIGH — confirms BC-DAEMON-006 from Pass 3.

### BC-DAEMON-COMP-007: SendChoice tries local first, then loops through connected remotes
**Postconditions:** If local fails, every connected remote is tried. The first success returns nil; the last error is returned if none succeed. **Unordered** through `range c.remotes` (map iteration), so behavior on conflicting window IDs is nondeterministic.
**Evidence:** composite_provider.go:349-365.
**Confidence:** HIGH — minor anti-pattern (nondeterministic ordering); acceptable because window IDs should be unique across hosts.

### BC-DAEMON-COMP-008: PendingNotifications collects from all connected remotes; window is remapped via remapRemoteWindow ("lc-xxxx" → "rm-xxxx")
**Postconditions:** Fallback path: when SSEToolInfoCallback isn't set, this static remap converts remote names to mirror names. Callback path (via SSEToolInfoCallback) is preferred and more accurate (uses session ID hop).
**Evidence:** composite_provider.go:532-561.
**Confidence:** HIGH

### BC-DAEMON-COMP-009: Lock order is c.mu (RLock) → rp.mu (Lock via PendingNotifications); no inversion exists
**Postconditions:** RemoteProvider.handleSSEEvent holds only rp.mu, never calls CompositeProvider. Comment at 528-531 documents the invariant.
**Evidence:** composite_provider.go:528-531.
**Confidence:** HIGH

### BC-DAEMON-COMP-010: PurgeOrphans is local-only
**Postconditions:** Remote orphans are not purged by this composite method. The remote daemon presumably has its own GC.
**Evidence:** composite_provider.go:312-314.
**Confidence:** HIGH

## RemoteConnection lifecycle

`RemoteConnection` (connection_impl.go:71-387) implements `ConnectionManager` with exponential backoff reconnection.

### State transitions

```
                  Connect()
Disconnected ─────────────────────> Connecting ─> Connected
     ▲                                   │            │
     │                                   │            │ tunnel dies
     │ Disconnect()                      ▼            ▼
     │                              ConnectionError  Reconnecting
     │                                                │
     │                                                │ backoff exhausted
     │                                                ▼
     └─────────────────────────── ConnectionError ─┘
```

### BC-DAEMON-CONN-001: ExponentialBackoff defaults: initial=1s, max=30s, factor=2.0, maxRetries=5
**Postconditions:** Sequence: 1s, 2s, 4s, 8s, 16s, 30s (capped). Total ~61s before ConnectionError.
**Evidence:** connection_impl.go:68 (DefaultMaxRetries=5), 100 (NewExponentialBackoff(1s, 30s, 2) WithMaxRetries(5)).
**Confidence:** HIGH

### BC-DAEMON-CONN-002: Two mutexes — connMu serializes Connect/Disconnect; mu protects state reads/writes
**Postconditions:** Connect and Disconnect cannot interleave. State reads are RLock-safe. Avoids the common "double-init" race.
**Evidence:** connection_impl.go:77-91 + Connect/Disconnect implementations.
**Confidence:** HIGH

### BC-DAEMON-CONN-003: Connect flow: discover → start (if discover fails) → tunnel → health check → version check → set Connected → spawn monitor goroutine
**Postconditions:** Discovery is "look for live daemon.json on remote"; start is "ssh <host> lazyclaude daemon" subprocess.
**Evidence:** connection_impl.go:148-232.
**Confidence:** HIGH

### BC-DAEMON-CONN-004: On discovered-but-dead daemon, kills tunnel and restarts the daemon (one retry only)
**Postconditions:** "Stale daemon.json" recovery — discovered daemon may be a defunct lock file. Comment at 181-183: "Stale daemon.json: discovered daemon is dead."
**Evidence:** connection_impl.go:181-201.
**Confidence:** HIGH

### BC-DAEMON-CONN-005: API version mismatch (local APIVersion ≠ remote APIVersion) triggers ConnectionError with explicit user-facing message
**Postconditions:** Message format: "API version mismatch on %s: local=%d remote=%d (update lazyclaude on remote)". Tunnel is stopped.
**Evidence:** connection_impl.go:209-214.
**Confidence:** HIGH

### BC-DAEMON-CONN-006: monitorTunnel goroutine waits on tunnel.Wait() channel; tunnel death triggers reconnect()
**Evidence:** connection_impl.go:310-330.
**Confidence:** HIGH

### BC-DAEMON-CONN-007: reconnect() applies exponential backoff, calls connectLocked, invokes reconnect hooks on success
**Postconditions:** OnReconnect-registered callbacks (e.g., re-StartSSE) fire after every reconnection but NOT after initial Connect (comment at 117-119).
**Evidence:** connection_impl.go:332-383.
**Confidence:** HIGH

### BC-DAEMON-CONN-008: setState only invokes callbacks on actual state transition (no-op if state unchanged)
**Postconditions:** Idempotent. Callbacks are copied under lock then invoked unlocked.
**Evidence:** connection_impl.go:293-307.
**Confidence:** HIGH

### BC-DAEMON-CONN-009: cancel context (rc.cancel) is derived from caller's Connect ctx; Disconnect cancels it to stop the monitor
**Postconditions:** Caller's ctx death propagates to the monitor goroutine.
**Evidence:** connection_impl.go:217-229 (cancel setup), 247 (Disconnect cancels).
**Confidence:** HIGH

## HTTPClient — REST surface

`HTTPClient` (http_client.go full file) is the daemon API client. 10-second timeout default for all calls; SSE uses a separate timeout-less client.

### BC-DAEMON-HTTP-001: 10-second timeout default for all HTTP calls; SSE uses no timeout
**Postconditions:** Long-lived SSE bypasses the default timeout (line 226-227).
**Evidence:** http_client.go:31-34, 226-227.
**Confidence:** HIGH

### BC-DAEMON-HTTP-002: All requests carry `AuthHeader: token` when token is non-empty
**Postconditions:** AuthHeader = "X-Daemon-Authorization" (from api.go:354 per Pass 5).
**Evidence:** http_client.go:330-333.
**Confidence:** HIGH

### BC-DAEMON-HTTP-003: HTTP error responses truncate to 4096 bytes via `maxErrorBodySize`
**Postconditions:** Prevents memory blowup on huge error bodies.
**Evidence:** http_client.go:17, 340-342.
**Confidence:** HIGH

### BC-DAEMON-HTTP-004: SSE channel buffer is 32; parseSSEStream pushes events into it with context-cancellation guard
**Evidence:** http_client.go:237, 267-271.
**Confidence:** HIGH

### BC-DAEMON-HTTP-005: SSE parser supports `event:`, `data:` lines; ignores `id:`, `retry:`, `:` comment lines
**Postconditions:** Event boundary is the empty line. Multi-line `data:` is concatenated with `\n`.
**Evidence:** http_client.go:279-290.
**Confidence:** HIGH

### BC-DAEMON-HTTP-006: SSE `event:` field overrides ev.Type after JSON decode
**Postconditions:** If the JSON body specifies a type AND the SSE `event:` line is present, the SSE field wins.
**Evidence:** http_client.go:263-265.
**Confidence:** HIGH

### BC-DAEMON-HTTP-007: sessionPath uses url.PathEscape on the session ID
**Postconditions:** UUID session IDs are URL-safe but defensive escaping handles any custom ID.
**Evidence:** http_client.go:37-40.
**Confidence:** HIGH

### BC-DAEMON-HTTP-008: ListWorktrees uses GET with query string `project_root=<value>`
**Postconditions:** url.Values handles URL encoding. Other endpoints use POST with JSON body.
**Evidence:** http_client.go:105-113.
**Confidence:** HIGH

### BC-DAEMON-HTTP-009: SubscribeNotifications returns a channel that closes on context cancel OR EOF
**Evidence:** http_client.go:244-246 (parseSSEStream defers close).
**Confidence:** HIGH

### BC-DAEMON-HTTP-010: doJSON's error path returns wrapped HTTP status text; success path decodes JSON via Decoder (streaming)
**Postconditions:** No upper limit on success-response body size (Decoder is streaming). This could OOM on a malicious server. Acceptable for a local 127.0.0.1 daemon.
**Evidence:** http_client.go:330-351.
**Confidence:** HIGH — minor: bounded-decoder would be safer.

## CapturePreviewContent — shared between local and remote

`CapturePreviewContent` (capture_preview.go:21-41) is the shared implementation:
1. `tc.CapturePaneANSI(ctx, target)` — gets ANSI-colored content.
2. `tc.ShowMessage(ctx, target, "#{cursor_x},#{cursor_y}")` — gets cursor position.
3. Returns `PreviewResponse{Content, CursorX, CursorY}`.

### BC-DAEMON-CAP-001: Cursor position errors are swallowed (best-effort); returns 0,0 if ShowMessage fails
**Postconditions:** Content is mandatory; cursor is optional.
**Evidence:** capture_preview.go:27-34.
**Confidence:** HIGH

### BC-DAEMON-CAP-002: Resize is caller's responsibility (not done here)
**Postconditions:** Comment at line 20: "Resize must be performed by the caller before calling this function, because resize deduplication logic differs between callers."
**Evidence:** capture_preview.go:17-20.
**Confidence:** HIGH

## DaemonServer — HTTP surface

`DaemonServer` (server.go:42-178) wires the HTTP handlers and lifecycle.

### Routing (server.go:93-132)

| Method+Path | Handler | Auth |
|---|---|---|
| POST /session/create | handleSessionCreate | yes |
| DELETE /session/{id} | handleSessionDelete | yes |
| POST /session/{id}/rename | handleSessionRename | yes |
| GET /sessions | handleSessionList | yes |
| POST /session/{id}/scrollback | handleScrollback | yes |
| GET /session/{id}/history-size | handleHistorySize | yes |
| POST /worktree/create | handleWorktreeCreate | yes |
| POST /worktree/resume | handleWorktreeResume | yes |
| GET /worktrees | handleWorktreeList | yes |
| POST /session/resume | handleSessionResume | yes |
| POST /msg/send | handleMsgSend | yes |
| POST /msg/create | handleMsgCreate | yes |
| GET /msg/sessions | handleMsgSessions | yes |
| GET /profiles | handleProfiles | yes |
| GET /cwd | handleCWD | yes |
| GET /health | handleHealth | **NO** |
| POST /shutdown | handleShutdown | yes |
| GET /notifications | handleSSE | yes |

### BC-DAEMON-SRV-001: Only GET /health is auth-exempt; all other endpoints require X-Daemon-Authorization
**Evidence:** server.go:127 (no `s.withAuth` wrap on /health).
**Confidence:** HIGH — confirms BC-DAEMON-002.

### BC-DAEMON-SRV-002: Server binds to 127.0.0.1 explicitly (not 0.0.0.0)
**Postconditions:** No remote-network exposure regardless of misconfiguration.
**Evidence:** server.go:139 (addr = "127.0.0.1:%d").
**Confidence:** HIGH

### BC-DAEMON-SRV-003: Port=0 means random port; actual port is read back from listener.Addr() and stored in config
**Evidence:** server.go:139-148.
**Confidence:** HIGH

### BC-DAEMON-SRV-004: writeDaemonInfo is the first action after listen; failure → close listener + return error
**Postconditions:** Ensures daemon.json never points at a non-listening port.
**Evidence:** server.go:149-152.
**Confidence:** HIGH

### BC-DAEMON-SRV-005: Stop is idempotent (guarded by `s.shutdown` flag); first call closes shutdownCh + removes daemon.json + shuts down HTTP server
**Postconditions:** Multiple /shutdown POSTs are safe.
**Evidence:** server.go:165-177.
**Confidence:** HIGH

### BC-DAEMON-SRV-006: shutdownCh is closed (not sent) so multiple readers can receive the signal
**Evidence:** server.go:172.
**Confidence:** HIGH

## P0 Risk Verification: shell.Quote inside SSH command strings

**Status: CONFIRMED present at `daemon/remote_provider.go:450-462`** as flagged in pass-5-security-deps.md and Pass 6 seed 2.

The `buildTmuxAttachCommand` returns a string containing `shell.Quote(window)`. This string is then used by `AttachSession`. Let me confirm via the next read whether it's base64-wrapped or used directly.

Looking at runSSHInteractive (remote_provider.go:428-441), the pattern is:
```go
encoded := base64.StdEncoding.EncodeToString([]byte(remoteCmd))
args = append(args, sshHost, fmt.Sprintf("eval \"$(echo %s | base64 -d)\"", encoded))
```

**So buildTmuxAttachCommand's output IS base64-wrapped before reaching the SSH arg.** The CLAUDE.md warning "Do not use `shell.Quote` inside SSH command strings" is about the **outer SSH layer** of quoting. Inside the base64-decoded eval, the shell is bash, and the `shell.Quote(window)` is correctly quoting for that inner bash invocation.

### P0-VERIFICATION-001: shell.Quote at remote_provider.go:461 is safe because the entire command is base64-encoded before reaching the SSH arg
**Postconditions:** The outer ssh arg contains `eval "$(echo BASE64 | base64 -d)"`. Inside the eval'd bash, `shell.Quote(window)` is the single layer of quoting. **The CLAUDE.md warning still holds for the general case** (where commands aren't base64-wrapped), but this specific call site is correctly handled.
**Evidence:** remote_provider.go:430-441 (runSSHInteractive base64 wrap), 450-463 (buildTmuxAttachCommand with shell.Quote inside).
**Confidence:** HIGH
**Disposition:** **REFUTED as an active bug** — the seed-2 hypothesis is incorrect for this call site. The shell.Quote use is correct because the surrounding base64 encoding makes the wrap single-pass at the bash layer.
**Recommendation:** Add a code comment at remote_provider.go:461 noting "base64-wrapped above, so shell.Quote is at the correct layer." This would prevent future readers from being confused by the CLAUDE.md note.

## P0 Risk Verification: control.go:176-179 TODO about Unicode/combining chars

The TODO comment at control.go:176-179 explicitly acknowledges the escaping "may need review for edge cases with tmux control mode quoting (e.g., unusual Unicode, combining characters, or tmux version-specific behavior)." Pass 6 seed 1 elaborates: combining characters atop a `"` don't slip past the `ReplaceAll` because UTF-8 doesn't interleave bytes.

### P0-VERIFICATION-002: control.go:182-185 escaping is byte-safe for UTF-8; the TODO is about higher-order semantic edge cases, not byte-injection
**Postconditions:** A combining character like `U+0300` (combining grave) encodes as two bytes in UTF-8 (0xCC 0x80). Neither byte is `\` or `"`. So the ReplaceAll loop cannot mis-match. **However**, the comment about "tmux version-specific behavior" remains unverified — different tmux versions may handle combining-char-after-quote differently when re-rendering. This is a behavioral fidelity question, not a security/injection one.
**Evidence:** control.go:176-185.
**Confidence:** HIGH (for byte-safety); MEDIUM (for cross-tmux-version semantic fidelity).
**Disposition:** Byte-safety: confirmed safe. Cross-version-semantic-fidelity: still unverified, low impact.

## Delta Summary

- New items added: 28 (10 BC-DAEMON-COMP, 9 BC-DAEMON-CONN, 10 BC-DAEMON-HTTP, 2 BC-DAEMON-CAP, 6 BC-DAEMON-SRV) + 2 P0-VERIFICATION findings
- Existing items refined: BC-DAEMON-002 confirmed (auth-only-on-non-health), BC-DAEMON-006 confirmed (scrollback via remote daemon, preview via mirror)
- Remaining gaps: server.go body (handler implementations), server_sse.go body, remote_provider.go tail (consumeSSE switch, addToCache/removeFromCache, AttachSession), lifecycle.go (LifecycleManager), tunnel.go, askpass.go (referenced in BC-ASKPASS), proc_cwd_linux.go (Linux PID→CWD walker), debug.go, paths.go.

## Novelty Assessment

Novelty: SUBSTANTIVE

Justification: 28 new contracts plus two P0 risk verifications. Specifically novel:
- **BC-DAEMON-COMP-003** pointer-identity check pattern for cache writes after RUnlock→Lock window.
- **BC-DAEMON-COMP-006** providerForCapture's asymmetric routing (local store has remote mirrors but capture must dispatch by host).
- **BC-DAEMON-CONN-004** stale daemon.json recovery (one-shot restart).
- **BC-DAEMON-CONN-005** explicit version mismatch error message.
- **P0-VERIFICATION-001 REFUTED** — the shell.Quote-inside-SSH risk for buildTmuxAttachCommand is non-existent at this call site (base64-wrapped). This is a substantial finding because it tells the porter NOT to redesign this code path.
- **BC-DAEMON-HTTP-010** unbounded streaming JSON decoder.
- **BC-DAEMON-COMP-007** SendChoice nondeterministic remote iteration.

These materially change the porter's mental model of the daemon subsystem.

## Convergence Declaration

Another round needed — server.go body (~600 LOC of handler implementations: handleSessionCreate, handleScrollback, handleProfiles, handleSSE), server_sse.go (full file, contains the SSE-emission logic including sessionIDForWindow lookup), remote_provider.go tail (~500 LOC of SessionProvider methods + consumeSSE handler details), and lifecycle.go all remain unread.

## State Checkpoint

```yaml
pass: B
subsystem: daemon
round: 1
status: complete
files_read_full: [composite_provider.go, connection_impl.go, capture_preview.go, http_client.go]
files_read_partial: [server.go (200/784), remote_provider.go (200/699)]
contracts_drafted: 28
p0_risks_verified: 2
timestamp: 2026-05-11T20:50:00Z
novelty: SUBSTANTIVE
next_round: 2
```
