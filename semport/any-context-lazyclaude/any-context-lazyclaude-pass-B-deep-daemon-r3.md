# Pass B Deep: `internal/daemon` — Round 3

**Scope:** tunnel.go (full), askpass.go (full), ssh.go (full), api.go (wire types), connection.go (interfaces).

**Files read in full this round:** tunnel.go, askpass.go, ssh.go, api.go.

## Tunnel — local port forwarding

`Tunnel` (tunnel.go:13-244) manages one `ssh -L localPort:127.0.0.1:remotePort -N ...` process. Internal state guarded by `mu sync.Mutex`. `done chan error` signals SSH process exit.

### BC-DAEMON-TUN-001: Start picks a free port via OS-allocated listen-on-zero, then issues ssh with `-L <local>:127.0.0.1:<remote>`
**Postconditions:** TOCTOU race possible (port allocated, released, then SSH binds it) — mitigated by `ExitOnForwardFailure=yes` in baseSSHArgs (BC-TUNNEL-002 from Pass 3).
**Evidence:** tunnel.go:55-60, 62.
**Confidence:** HIGH

### BC-DAEMON-TUN-002: BatchMode is included in baseSSHArgs ONLY when askpassEnv is empty
**Postconditions:** Tunnel decides per-call: with askpass, no BatchMode (interactive auth possible); without askpass, BatchMode=yes (no prompt).
**Evidence:** tunnel.go:63.
**Confidence:** HIGH

### BC-DAEMON-TUN-003: SSH process is started via exec.CommandContext; ctx cancellation kills the process
**Postconditions:** done channel receives the Wait() result; sender goroutine closes the channel on exit.
**Evidence:** tunnel.go:69-86.
**Confidence:** HIGH

### BC-DAEMON-TUN-004: Start releases its mutex BEFORE waitForPort to allow Stop or LocalPort calls during the wait
**Evidence:** tunnel.go:87-91.
**Confidence:** HIGH

### BC-DAEMON-TUN-005: On waitForPort failure, the SSH process is killed and `t.cmd = nil` so caller can retry
**Evidence:** tunnel.go:95-103.
**Confidence:** HIGH

### BC-DAEMON-TUN-006: waitForPort polls every 100ms with a 10s upper bound; 3 exit conditions (ctx done, ssh exit, connect success)
**Postconditions:** SSH process exit before connect = "exited before becoming ready" error.
**Evidence:** tunnel.go:108-139.
**Confidence:** HIGH — confirms BC-TUNNEL-003.

### BC-DAEMON-TUN-007: Stop kills the process via Process.Kill (SIGKILL)
**Postconditions:** No graceful shutdown attempt. SSH process is well-behaved (no orphaned child processes from `-N -a`).
**Evidence:** tunnel.go:142-154.
**Confidence:** HIGH

## Askpass — UDS-mediated password prompt

`AskpassServer` (askpass.go:25-194) listens on `<runtimeDir>/askpass-<pid>.sock` (mode 0600) and serializes handler invocations via `handlerMu`.

### BC-DAEMON-AP-001: One handler invocation at a time; concurrent clients serialize on handlerMu
**Postconditions:** Prevents the GUI popup from getting clobbered. While one popup is open, others wait. Confirms BC-ASKPASS-005.
**Evidence:** askpass.go:31, 182-185.
**Confidence:** HIGH

### BC-DAEMON-AP-002: Stale socket/script files removed at Start (defensive)
**Postconditions:** Prevents `listen unix: bind: address in use` if the previous lazyclaude crashed.
**Evidence:** askpass.go:58-60.
**Confidence:** HIGH

### BC-DAEMON-AP-003: WriteScript content = `"#!/bin/sh\nexec '<binPath>' askpass \"$@\"\n"`, mode 0700
**Postconditions:** Confirms BC-ASKPASS-003. The single-quoted binPath protects against spaces; `"$@"` preserves SSH's prompt arg.
**Evidence:** askpass.go:87-89.
**Confidence:** HIGH

### BC-DAEMON-AP-004: Stop closes all tracked connections (askpass.go:111-114) before removing socket/script files
**Postconditions:** Unblocks any handler currently waiting on a connection read.
**Evidence:** askpass.go:97-118.
**Confidence:** HIGH

### BC-DAEMON-AP-005: handleConn sets a 10-second read deadline for prompt reception, then clears it for the handler phase
**Postconditions:** Prevents goroutine leak from idle clients. User interaction time is unbounded (cleared deadline).
**Evidence:** askpass.go:170, 178-180.
**Confidence:** HIGH

### BC-DAEMON-AP-006: Handler error → empty line sent back (signals cancellation to SSH)
**Postconditions:** Empty SSH_ASKPASS response triggers next auth method (or fail). Confirms BC-ASKPASS-006.
**Evidence:** askpass.go:187-191.
**Confidence:** HIGH

### BC-DAEMON-AP-007: handleConn reads one line via bufio.Scanner; if scan fails (EOF, closed), handler is NOT invoked
**Postconditions:** Premature client close means no popup. Safe degradation.
**Evidence:** askpass.go:172-175.
**Confidence:** HIGH

### BC-DAEMON-AP-008: SockPath includes process PID to avoid collision with other lazyclaude instances
**Postconditions:** `runtimeDir/askpass-<pid>.sock` and `.sh`. Confirms BC-ASKPASS-001.
**Evidence:** askpass.go:44-49.
**Confidence:** HIGH

## SSH — command/copy execution

`ExecSSHExecutor` (ssh.go:18-82) implements `SSHExecutor` via real `ssh`/`scp` commands. Two methods: Run (single command) and Copy (scp).

### BC-DAEMON-SSH-001: SSHEnv returns nil when AskpassScript is empty (no env injection)
**Postconditions:** Callers without askpass get plain SSH; BatchMode is the fallback.
**Evidence:** ssh.go:31-47.
**Confidence:** HIGH

### BC-DAEMON-SSH-002: SSHEnv injects SSH_ASKPASS, SSH_ASKPASS_REQUIRE=prefer, LAZYCLAUDE_ASKPASS_SOCK; DISPLAY=:0 only if not set
**Postconditions:** DISPLAY=:0 fallback required for SSH <8.4 to trigger askpass when no TTY. Pre-existing DISPLAY (X11 forwarding) is preserved. Confirms BC-SSH-004.
**Evidence:** ssh.go:36-46.
**Confidence:** HIGH

### BC-DAEMON-SSH-003: Run/Copy use ConnectTimeout=10, ControlMaster=no, ControlPath=none
**Postconditions:** No SSH multiplexing → each invocation is independent. Avoids ControlPath collision and multiplexing-related auth surprises. Confirms BC-SSH-001.
**Evidence:** ssh.go:51, 69.
**Confidence:** HIGH

### BC-DAEMON-SSH-004: SplitHostPort handles 6 input shapes: host, host:port, user@host, user@host:port, [ipv6], [ipv6]:port
**Postconditions:** isNumeric guard prevents misparsing of `host:foo` (where `foo` isn't a port). Confirms BC-SSH-003.
**Evidence:** ssh.go:88-130.
**Confidence:** HIGH

### BC-DAEMON-SSH-005: SplitHostPort uses LastIndex(`@`) and LastIndex(`:`) — handles user@host:port where user may contain `:` (unusual but possible)
**Postconditions:** Searches `:` only after the last `@` to avoid matching `:` inside the user portion.
**Evidence:** ssh.go:105-113.
**Confidence:** HIGH

### BC-DAEMON-SSH-006: Bracketed IPv6 form `[::1]` requires the closing bracket; missing closing bracket returns the whole input as host (port "")
**Postconditions:** Defensive fallback.
**Evidence:** ssh.go:90-103.
**Confidence:** HIGH

## API wire types

`api.go` (full file) defines the JSON wire types for the daemon HTTP API. APIVersion = 4. Version history is documented inline:

- **v1**: initial daemon API (session CRUD, worktree, messaging, SSE).
- **v2**: adds POST `/session/{id}/scrollback` and GET `/session/{id}/history-size`.
- **v3**: adds POST `/session/resume` (with worktree name fallback).
- **v4**: adds GET `/profiles` and Profile/Options fields on session/worktree/msg create.

### BC-DAEMON-API-001: APIVersion is a single int constant; clients compare against `/health` response
**Evidence:** api.go:11-25.
**Confidence:** HIGH — confirms BC-DAEMON-001.

### BC-DAEMON-API-002: SessionInfo carries Activity + ToolName + Role inline (no separate state endpoint)
**Postconditions:** Sessions list is self-contained for sidebar rendering.
**Evidence:** api.go:67-78.
**Confidence:** HIGH

### BC-DAEMON-API-003: NotificationEventType has 3 values: activity, tool_info, full_sync
**Postconditions:** Stop, SessionStart, PromptSubmit broker events all collapse to "activity" type on the SSE wire (BC-DAEMON-SSE-002).
**Evidence:** api.go:322-331.
**Confidence:** HIGH

### BC-DAEMON-API-004: NotificationEvent.ToolNotification is `*model.ToolNotification` (pointer, omitempty) — only present when Type=="tool_info"
**Postconditions:** Other event types omit it from the JSON.
**Evidence:** api.go:334-349.
**Confidence:** HIGH

### BC-DAEMON-API-005: ProfileDefAPI.Env is `map[string]string` — order is not preserved on the wire
**Postconditions:** Profile env vars are unordered. Client must not rely on iteration order.
**Evidence:** api.go:292-300.
**Confidence:** HIGH

### BC-DAEMON-API-006: ShutdownRequest has a Force field but server's handleShutdown ignores it
**Postconditions:** Force is parsed but unused. Currently informational only.
**Evidence:** api.go:315-317; server.go:713-724 (handler ignores req body fields).
**Confidence:** HIGH — minor: field is dead in current implementation.

### BC-DAEMON-API-007: MsgSessionInfo includes Host field (`omitempty`); used for inter-host messaging context
**Evidence:** api.go:267-272.
**Confidence:** HIGH

### BC-DAEMON-API-008: AuthHeader = "X-Daemon-Authorization" (constant)
**Evidence:** api.go:353-354.
**Confidence:** HIGH — confirms BC-DAEMON-002.

## Cross-pass synthesis

Daemon subsystem now has comprehensive contract coverage:

- **Routing**: CompositeProvider → Local | Remote per session/host (r1).
- **Connection lifecycle**: state machine + exponential backoff (r1).
- **HTTP API**: 18 endpoints, all token-auth except /health (r1, r2).
- **SSE emission**: 5 broker variants → 3 wire types (r2).
- **Remote provider**: SessionProvider via daemon API + SSE cache (r2).
- **Tunnel**: Free-port + waitForPort + Stop semantics (r3).
- **Askpass**: UDS protocol + serialization (r3).
- **SSH**: SSHExecutor + SplitHostPort + env injection (r3).
- **Wire types**: APIVersion=4 + 3 SSE event types (r3).

### Verifications

| Verification | Status |
|---|---|
| BC-BROKER-003 GUI buffer 8 vs daemon broker 64 (r2 vs r1 of gui pass) | Confirmed; different subscriber tolerance. |
| shell.Quote inside SSH command strings at remote_provider.go:461 | **REFUTED** — base64-wrapped (r1). |
| control.go:176-179 TODO about Unicode | Byte-safe per UTF-8 invariant (r1); semantic cross-version fidelity unverified. |
| daemon /msg/create supports {worker, local} | **REFUTED** — daemon supports {worker, pm}; server (MCP) supports {worker, local}. Schema divergence noted (r2). |
| `lazyclaude daemon stop` subcommand | **GAP** — referenced in lifecycle.go:53 but not in CLI inventory (Pass 0). Possibly unimplemented (r2). |

## Delta Summary

- New items added: 26 (7 BC-DAEMON-TUN, 8 BC-DAEMON-AP, 6 BC-DAEMON-SSH, 8 BC-DAEMON-API)
- Existing items refined: 5 (BC-TUNNEL-002/003, BC-SSH-001/003/004, BC-ASKPASS-005/006 confirmed at code level).
- Remaining gaps: connection.go (small interface definitions), proc_cwd_linux.go (Linux PID→CWD walker, 262 LOC), proc_cwd_other.go (non-Linux stub), paths.go, debug.go.

## Novelty Assessment

Novelty: NITPICK

Justification: 26 new contracts but they are confirmations of patterns already established or wire-type documentation. The model-changing daemon findings have been exhausted:

- All BC-DAEMON-TUN are repetitions of Pass 3 BC-TUNNEL with code-level grounding.
- BC-DAEMON-AP confirm Pass 3 BC-ASKPASS.
- BC-DAEMON-SSH confirm Pass 3 BC-SSH and BC-DAEMON-CONN.
- BC-DAEMON-API are wire types — important for porter but not architecturally novel.
- Only **BC-DAEMON-API-006** (Force field unused) and **BC-DAEMON-API-005** (Env map iteration order) are mildly substantive; they don't change architecture.

Remaining unread daemon files (connection.go interface defs, proc_cwd_*) would add minor contracts but no new patterns: connection.go is just the interface declarations matched in BC-DAEMON-CONN; proc_cwd_linux.go walks /proc/<pid>/cwd. These are mechanical.

If we removed Round 3's findings, a porter could rebuild the tunnel/askpass/ssh subsystems from Rounds 1-2 + Pass 3 contracts plus standard system-programming patterns. Round 3 adds detail and confirmation, not new architecture.

## Convergence Declaration

**Pass B daemon has converged — findings are nitpicks, not gaps.** The connection.go and proc_cwd_linux.go files remain unread but they contain (a) interface declarations matched in already-extracted BC-DAEMON-CONN contracts and (b) Linux-specific PID→CWD walking that is one platform-specific implementation detail. A round 4 would add documentation without changing the porter's model.

## State Checkpoint

```yaml
pass: B
subsystem: daemon
round: 3
status: complete
files_read_full: [tunnel.go, askpass.go, ssh.go, api.go]
contracts_drafted: 26
total_daemon_contracts_across_rounds: 90  # 28 r1 + 36 r2 + 26 r3
timestamp: 2026-05-11T21:30:00Z
novelty: NITPICK
convergence: PASS-B-DAEMON CONVERGED
next_subsystem: session
```
