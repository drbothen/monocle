---
document_type: architecture-section
level: L3
section: "daemon-wiring"
subsystem: SS-04
version: "1.3.0"
status: draft
producer: vsdd-factory:architect
phase: phase-1-expansion
timestamp: 2026-05-26T00:00:00Z
inputs:
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/product-brief.md, version: "1.4.30"}
  - {path: .factory/specs/research/domain-monocle-vision-synthesis.md, version: "1.1.3"}
  - {path: crates/monocle-runtime/src/server.rs}
  - {path: crates/monocle-runtime/src/state.rs}
input-hash: "[pending]"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: Daemon Wiring

## Scope

SS-04 is the composition root of the monocle binary. It does NOT define a new daemon or
duplicate the infrastructure established in SS-01 (Daemon Lifecycle). Instead, SS-04 specifies
how the existing `build_server()`, `DaemonState`, `RingBuffer`, `RecoveryCheckpoint`,
`ClaudeCodeModule`, and `VsddFactoryAdapter` are wired together into a runnable binary, how
the CLI surface is structured, and how the bounded event bus connects incoming hook events to
connected TUI clients.

The implementing crates are:
- `monocle` — binary crate. Owns `main.rs`, the `clap` CLI, the daemon entrypoint, and the
  TUI entrypoint. No library surface; all logic delegates to `monocle-runtime`.
- `monocle-runtime` — owns hook tmpfile generation, bounded event bus initialization, and
  the `MONOCLE_NO_AUTOSTART` env-var check. All server construction is already implemented
  in `monocle-runtime::server` (`build_server` / `run_server`); SS-04 wires these into the
  `main.rs` call sequence.

For daemon lifecycle infrastructure — port binding, lock file schema, auth token format, ring
buffer implementation, graceful shutdown protocol, crash recovery checkpoint — see
`SS-daemon-lifecycle.md`. This document does not repeat that content; it references it and
specifies only the wiring contract that coordinates those components at startup.

---

## CLI Interface

The `monocle` binary crate exposes a `clap`-based CLI with the following structure:

```
monocle [SUBCOMMAND]

SUBCOMMANDS:
    daemon start    Start the daemon in the background; exit 0 when lock file is written.
    daemon stop     Send SIGTERM to the daemon process; wait up to 15 s for exit.
    (default)       Launch the TUI. Auto-start the daemon if not running.
```

### Subcommand: `monocle daemon start`

- Runs the daemon as a background process, detached from the terminal (double-fork or `nohup`
  pattern on Unix; the process group is set so the daemon survives the parent shell exiting).
- The foreground caller blocks until the lock file appears at `<runtime_dir>/monocle.lock`,
  confirming that the daemon has bound its port and written the lock file.
- If the lock file already exists AND the PID in it is alive (`kill(pid, 0)` returns 0), the
  command exits with code 1 and writes to stderr:
  `error: daemon already running (pid=<N>)`.
- On success, the command exits with code 0 and no stdout output.

### Subcommand: `monocle daemon stop`

- Reads the lock file at `<runtime_dir>/monocle.lock`.
- Sends SIGTERM to the recorded PID.
- Polls for process exit up to 15 seconds (1-second poll interval).
- If the process exits within 15 seconds, exits with code 0.
- If the lock file does not exist, exits with code 1 and writes to stderr:
  `error: no lock file found; daemon may not be running`.
- If the PID does not exist (`kill(pid, 0)` returns ESRCH), exits with code 1 and writes to
  stderr: `error: daemon not running (stale lock file?)`.
- If the process does not exit within 15 seconds, exits with code 2 and writes to stderr:
  `error: daemon did not exit within 15 s; it may still be draining`.

### Default Mode: TUI with Auto-Start

When invoked without a subcommand, `monocle` launches the TUI. Before rendering any TUI
output, it checks the daemon state and conditionally starts the daemon. The auto-start
decision sequence is specified in §Daemon Auto-Start Logic below.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Usage or precondition error (see subcommand descriptions) |
| 2 | Timeout or partial-success condition (daemon stop timed out) |
| 70 | Internal error: runtime directory cannot be resolved |
| 71 | Internal error: lock file write failed |
| 72 | Internal error: hooks-settings.json write failed |

---

## Daemon Auto-Start Logic

This logic runs when `monocle` is invoked without a subcommand (TUI mode).

<a id="monocle_no_autostart-check-bc-2"></a>
### MONOCLE_NO_AUTOSTART Check (BC-2.04.003)

Before any daemon liveness check, `monocle` reads the environment variable
`MONOCLE_NO_AUTOSTART`. If it is set to any non-empty value (conventionally `1`), the
auto-start logic is skipped entirely. The TUI launches in "daemon offline" mode, rendering a
status-bar indicator `[daemon: offline]`. No daemon process is started. No lock file is read.

This escape hatch is provided for CI environments and power users who manage the daemon
lifecycle externally.

### Auto-Start Decision Sequence (BC-2.04.002)

When `MONOCLE_NO_AUTOSTART` is not set:

1. Resolve `runtime_dir` via the chain in SS-daemon-lifecycle.md §Start Sequence step 1
   (env override → `runtime_dir()` → `data_local_dir()` → fail-fast). If resolution fails,
   exit with code 70.
2. Check for `<runtime_dir>/monocle.lock`. If the file does not exist, proceed to step 4.
3. If the lock file exists, parse the `pid` field and call `kill(pid, 0)`. If the process
   is alive, treat the daemon as already running and skip to TUI connection (step 5).
   If the process is dead (stale lock), log `WARN: stale lock file removed` and proceed to
   step 4.
4. Start a daemon subprocess (`monocle daemon start` equivalent, run in-process via
   `Command::new(current_exe()).arg("daemon").arg("start")` rather than exec-ing again, or
   via direct call to the daemon start function). Wait up to 5 seconds for the lock file to
   appear. If it does not appear within 5 seconds, render a TUI error: "daemon start timed
   out; retrying…" and retry once. If retry also fails, render "daemon unavailable — running
   in offline mode".
5. Connect the TUI to the daemon via the UDS at `<runtime_dir>/monocle.sock`
   (SS-05 specifies the IPC protocol).

The daemon PID must pass a liveness check (`kill(pid, 0)` returns 0) before the TUI attempts
a UDS connection.

---

<a id="daemon-start-sequence-bc-2"></a>
<a id="daemon-start-sequence"></a>
## Daemon Start Sequence (BC-2.04.001)

When `monocle daemon start` runs (or when the auto-start path triggers the daemon entrypoint
directly), the daemon executes the following ordered steps. The SOQ-2 invariant —
bind socket + write lock file + write token THEN hooks-settings reads token — is the ordering
constraint that this sequence enforces.

**Step 1 — Resolve runtime_dir.**
Resolve the runtime directory per SS-daemon-lifecycle.md §Start Sequence step 1 (env override
→ `ProjectDirs::runtime_dir()` → `ProjectDirs::data_local_dir()` → fail-fast). Create the
directory at mode `0o700` if absent. If resolution fails, exit with `DaemonStartError::RuntimeDirUnresolvable` and code 70.

**Step 2 — Check for existing lock file and PID liveness (BC-2.01.005).**
This step is specified fully in SS-daemon-lifecycle.md §Start Sequence step 2. If a live
daemon is detected, exit with code 1.

**Step 3 — Bind axum HTTP listener.**
Call `TcpListener::bind("127.0.0.1:0")` to bind on an OS-assigned port. Record the assigned
port number. This step MUST complete before any lock file write (SOQ-2 invariant: bind first,
then write).

**Step 4 — Create RingBuffer.**
Construct a `RingBuffer` with capacity `100MB × 5 rotations` and flush mode `async-jsonl`
at `<runtime_dir>/monocle.jsonl`. The ring buffer is wrapped in `Arc<RingBuffer>` and
assigned to `DaemonState.ring`. Ring buffer implementation is specified in
SS-daemon-lifecycle.md §JSONL Ring Buffer. The rotation policy (BC-2.04.012) is:
- When the active JSONL file reaches 100 MB, rotate: rename `.jsonl` → `.jsonl.1`, shift
  `.jsonl.1` → `.jsonl.2`, …, delete `.jsonl.5` (oldest). The active file becomes a new
  empty `.jsonl`.
- Maximum 5 rotation files on disk at any time.
- The RAM ring holds the last 4,096 events in memory for zero-disk-read TUI access.

**Step 5 — Create bounded event bus (BC-2.04.011).**
Construct a `tokio::sync::mpsc::channel(4096)` pair. The sender half (`EventBusTx`) is
retained in `DaemonState`. The receiver half (`EventBusRx`) is consumed by the event-bus
fan-out task (see §Bounded Event Bus). The drop counter is initialized to `AtomicU64::new(0)`.

**Step 6 — Register EngineModule registry.**
Construct a `ClaudeCodeModule` and register it as the sole `EngineModule` implementation in
the daemon's `EngineModuleRegistry`. The registry is stored in `DaemonState`. The
`VsddFactoryAdapter` is initialized and associated with the `ClaudeCodeModule` per
SS-engine-module.md §VsddFactoryAdapter wiring.

**Step 7 — Generate auth token.**
Generate 32 bytes from `rand::rngs::OsRng`, hex-encode to 64 lowercase chars. The wire
token is `monocle-v1:<64-hex>`. Store only the 64-hex portion in `DaemonState.auth_token`
(the prefix is a wire-format concern only). Full token format is specified in
SS-daemon-lifecycle.md §Start Sequence step 3.

**Step 8 — Write lock file (SOQ-2 write point).**
Write `<runtime_dir>/monocle.lock` via `tempfile::persist` at mode `0o600` with JSON:
```json
{
  "pid": <N>,
  "port": <N>,
  "token": "monocle-v1:<64-hex>",
  "contract_version": "monocle-lock-v1",
  "started_at": "<ISO8601Z>"
}
```
This is the SOQ-2 ordering commit point: the lock file is written AFTER the port is bound
(step 3) and AFTER the token is generated (step 7). Any failure at this step causes the
daemon to abort with exit code 71. Lock file schema is the authoritative source for field
names; SS-daemon-lifecycle.md §Lock File Schema specifies the full schema contract.

**Step 9 — Generate hooks-settings.json (BC-2.04.010).**
Write `<runtime_dir>/hooks-settings.json` via `tempfile::persist` at mode `0o600` with the
hook configuration that Claude Code reads via its `--settings` flag. This step occurs AFTER
step 8 (SOQ-2: token is written to lock file before hooks-settings.json is generated, so any
process reading hooks-settings.json sees the stable token). See §Hook Tmpfile Generation for
the full schema.

**Step 10 — Create UDS socket.**
Bind a Unix domain socket at `<runtime_dir>/monocle.sock` at mode `0o600`. If a stale socket
file exists at that path, remove it before binding. Store the socket path in
`DaemonState.sock_file_path`. The UDS protocol is specified in SS-05 (IPC); SS-04 is
responsible only for creating the socket at the correct path and mode.

**Step 11 — Start recovery checkpoint detection (BC-2.01.006).**
Initialize the crash recovery checkpoint mechanism per SS-daemon-lifecycle.md §Crash Recovery
Checkpoint. This step starts the background task that writes periodic checkpoints and detects
incomplete shutdown markers on the previous lock file.

**Step 12 — Start HTTP server.**
Call `run_server(Arc::new(state), listener)`. This hands the axum router (built by
`build_server`) to the tokio runtime. From this point the daemon is serving requests.

**Step 13 — Signal startup complete.**
The foreground `daemon start` caller detects startup completion by polling for the lock file.
No explicit IPC from daemon to caller is required; the lock file write at step 8 is the
completion signal.

---

<a id="hook-endpoint-routing"></a>
## Hook Endpoint Routing (BC-2.04.007, BC-2.04.008, BC-2.04.009)

Hook POST requests arrive at the axum router registered in `build_server()`. The routing
path from HTTP request to `EngineModule::on_hook()` response is:

```
POST /hooks/<type>
  │
  ├── body_size_limit_middleware  (outermost; rejects Content-Length > 262144 with HTTP 413)
  ├── auth_middleware             (dual-accept per ADR-0005; rejects missing/invalid token with HTTP 401)
  ├── DefaultBodyLimit::max(262144)  (signals extractors; defense-in-depth)
  │
  └── hook handler (e.g., post_hook_pre_tool_use)
        │
        ├── Deserialize JSON body into HookEnvelope (monocle-proto)
        ├── Extract session_id; look up or create EnrichedSession in registry
        ├── Construct HookEvent variant from HookEnvelope fields
        │     (e.g., HookEvent::PreToolUse(PreToolUseEvent { tool_name, tool_input, session_id, pid }))
        ├── Call engine.on_hook(hook_event).await on each registered module
        │     └── ClaudeCodeModule::on_hook(HookEvent) → HookResponse { decision, redirect_url, diagnostic }
        ├── Publish HookEvent to event bus (EventBusTx::try_send; drop if full, increment counter)
        ├── Append JSONL record to RingBuffer (best-effort; WARN + 200 on failure per AC-005)
        └── Return HTTP response based on HookDecision within timeout budget
              PreToolUse / Stop / SessionStart / PromptSubmit: ≤300ms
              Notification: ≤2000ms
```

### SessionStart Invocation Path Note (F-P1D-009)

`POST /hooks/session-start` is called by Claude Code's internal lifecycle mechanism —
NOT through user-configurable hook scripts in `hooks-settings.json`. Claude Code fires
`SessionStart` events autonomously when a session begins; monocle does not configure
this via the `UserPromptSubmit` or similar hooks-settings entries. The
`hooks-settings.json` written by monocle at step 9 of the start sequence does NOT
include a `SessionStart` handler entry — Claude Code invokes this endpoint on its own
initiative.

Implementers: do NOT add a `SessionStart` key to the `hooks-settings.json` schema in
§Hook Tmpfile Generation. The `hooks-settings.json` `hooks` object has **6 JSON keys**:
4 URL-bearing keys (`PreToolUse`, `Notification`, `Stop`, `UserPromptSubmit` — each
pointing to `http://127.0.0.1:<port>/hooks/<endpoint>` with the `X-Monocle-Authorization`
token) plus 2 reserved-empty arrays (`PostToolUse: []`, `PreCompact: []`). This is
distinct from the **5 served HTTP endpoints** on the axum router (`/hooks/pre-tool-use`,
`/hooks/notification`, `/hooks/stop`, `/hooks/prompt-submit`, `/hooks/session-start`):
`SessionStart` is a served endpoint but NOT a `hooks-settings.json` key — Claude Code
invokes it via its internal lifecycle, not through the hook-script configuration in this
file (BC-2.04.010 PC-3). `SessionStart` arrives regardless of hooks-settings content.

### Timeout Budget Enforcement

The timeout budgets (BC-HOOK-022) are enforced at the handler level using
`tokio::time::timeout`. If `EngineModule::on_hook()` does not return within the budget,
the handler returns the fail-open or fail-closed response as specified in BC-HOOK-001 and
BC-HOOK-002 for PreToolUse. For Notification, Stop, SessionStart, and PromptSubmit (which
are fire-and-forget from Claude Code's perspective), a 2000ms / 300ms timeout still applies
to ensure the daemon does not hold open HTTP connections indefinitely.

<a id="pretooluse-permission-decision-hold"></a>
### PreToolUse — Permission Decision Hold

When `ClaudeCodeModule::on_hook()` returns a `HookResponse` with `decision: HookDecision::Defer` on a `PreToolUse` event,
the handler holds the HTTP response open and waits for a user decision from the TUI. The
daemon pushes a `PermissionPromptQueued` IPC message to all connected TUI clients
(BC-2.05.005). The response is sent when:
- The TUI user accepts or rejects (via the permission overlay), OR
- The 300ms timeout budget is reached (fail-open or fail-closed per BC-HOOK-001/BC-HOOK-002).

The response decision channel between the HTTP handler and the TUI-facing event fan-out is a
`oneshot::channel` created per hook invocation and stored in the daemon's pending-decision
registry keyed by session ID.

### HookResponse Wire Format

All hook endpoint responses return HTTP 200 with a JSON body. The format is specified in the
DTU clone contracts (BC-HOOK-xxx series). SS-04 is responsible for ensuring the response is
produced within the timeout budget; the wire format itself is governed by SS-01 and the DTU
contracts.

---

<a id="bounded-event-bus"></a>
## Bounded Event Bus (BC-2.04.011)

The bounded event bus is the central fan-out mechanism that connects incoming hook events
to all connected TUI clients. It is implemented as a `tokio::sync::mpsc::channel(4096)` pair.

### Capacity

`N = 4096` events. This value is chosen to accommodate burst patterns from multiple concurrent
Claude Code sessions without excessive memory consumption:
- Each `HookEvent` record (session ID, hook type, payload excerpt, latency) is bounded to
  approximately 512 bytes worst-case.
- At full capacity: `4096 × 512B = 2MB` of in-flight events — well within workstation memory
  budgets.
- At 1000 events/second (the Phase 1 synthetic load test target), a full channel buffers 4
  seconds of back-pressure before dropping begins.

### Drop Counter

When `EventBusTx::try_send()` returns `Err(TrySendError::Full)`, the handler:
1. Increments `DaemonState.drop_counter` (an `AtomicU64`) by 1.
2. Logs `WARN: event bus full; dropping event (drop_count=<N>)`.
3. Discards the event. The daemon continues processing the current hook request normally.

The drop counter value is included in every daemon-to-TUI state push, allowing the TUI status
bar to render it in real time (BC-2.06.019). A drop counter of zero means no events have been
lost since daemon start; this is the healthy steady state.

Drop counting does not affect hook response semantics: the hook HTTP response is sent based
on `HookDecision`, regardless of whether the event was published to the bus.

### Fan-Out Architecture

A dedicated `event_bus_task` Tokio task owns the `EventBusRx` receiver. It loops:
1. Await next event from the channel.
2. For each connected TUI client IPC writer, attempt to send the `HookEventReceived` IPC
   message (SS-05 framing).
3. If a TUI client send fails (disconnected), remove it from the connected-client list.

This task is spawned during daemon startup (step 5) and runs until the graceful-shutdown
signal. The `EventBusTx` sender is cloned into each hook handler as needed via
`Arc<EventBusTx>` stored in `DaemonState`.

### Back-Pressure and Hook Latency

Hook handlers use `try_send` (non-blocking). If the bus is full, the event is dropped
immediately — the hook handler is never blocked waiting for the event bus to drain. This
design ensures that hook response latency is not affected by TUI consumer slowness or
disconnect. The hook timeout budget (300ms / 2000ms) is preserved regardless of event bus
saturation.

---

<a id="hook-tmpfile-generation"></a>
## Hook Tmpfile Generation (BC-2.04.010)

The daemon generates `hooks-settings.json` at `<runtime_dir>/hooks-settings.json` to enable
Claude Code to discover the daemon's hook endpoints via the `--settings` flag.

### Schema

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "curl -s -X POST http://127.0.0.1:<port>/hooks/pre-tool-use -H 'Content-Type: application/json' -H 'X-Monocle-Authorization: monocle-v1:<token>' -d @-"
          }
        ]
      }
    ],
    "Notification": [ ... same pattern for /hooks/notification ... ],
    "Stop": [ ... same pattern for /hooks/stop ... ],
    "PostToolUse": [],
    "UserPromptSubmit": [ ... same pattern for /hooks/prompt-submit ... ],
    "PreCompact": []
  }
}
```

Key properties:
- The daemon serves 5 hook endpoints; hooks-settings.json configures 4 of them with
  URLs (`PreToolUse`, `Notification`, `Stop`, `UserPromptSubmit`). `SessionStart` is
  invoked by Claude Code's internal lifecycle, not via hooks-settings.json (see
  §SessionStart Invocation Path Note above). `PostToolUse` and `PreCompact` are
  included as reserved empty arrays (forward-compatibility). All configured URL entries
  embed the OS-assigned port and the full wire token `monocle-v1:<64-hex>`.
- `PostToolUse` and `PreCompact` are included with empty hook arrays for forward-compatibility
  (Claude Code ignores hook types with empty arrays).
- The file is written atomically via `tempfile::persist` at mode `0o600`.
- The file is regenerated on every daemon restart (new port, new token).
- The file is removed on graceful shutdown alongside the lock file.

### Atomic Write Requirement

All writes to `hooks-settings.json` MUST use `tempfile::persist`. Naked `std::fs::write`
to this path is forbidden per SS-conventions-anti-patterns.md §Forbidden Patterns. The
`tempfile::persist` pattern is:

```rust
let mut tmp = tempfile::NamedTempFile::new_in(&runtime_dir)?;
serde_json::to_writer_pretty(&mut tmp, &hooks_settings)?;
tmp.persist(&hooks_settings_path)?;
// Set mode 0o600 after persist (persist preserves tempfile's mode on most platforms).
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&hooks_settings_path,
        std::fs::Permissions::from_mode(0o600))?;
}
```

### hooks-settings.json vs. Lock File Ordering (SOQ-2)

The hooks-settings.json file is written AFTER the lock file (step 9 in the start sequence,
step 8 is lock file). This guarantees that any process reading `hooks-settings.json` to
discover the token sees a token that is already committed to the lock file. This eliminates
the race condition where a Claude Code subprocess reads the token from hooks-settings.json
before the daemon has written it to the lock file, which would cause a subsequent `/status`
read to return a different token value.

---

## Dependency Graph

SS-04 wires the following existing components. No circular dependencies.

```
monocle (binary crate)
  └── depends on monocle-runtime (server, state, lock, ring, auth, recovery)
       ├── SS-01 Daemon Lifecycle: build_server(), run_server(), DaemonState,
       │         RingBuffer, DaemonLock, RecoveryCheckpoint, auth_middleware
       ├── SS-02 Core Types and ABI: MONOCLE_ABI_VERSION, HookEnvelope (via monocle-proto),
       │         FactoryAdapter trait (VsddFactoryAdapter in monocle-runtime)
       └── SS-03 Engine Module: EngineModule trait (in monocle-core),
                 ClaudeCodeModule (in monocle-runtime), EnrichedSession, HookDecision

monocle-runtime (event bus, hooks-settings generation — SS-04 scope)
  └── depends on monocle-core (EngineModule, HookEvent, EnrichedSession — SS-03)
  └── depends on monocle-config (harness profiles, CCR path — SS-07)

monocle (TUI entry point — SS-04 wiring to TUI)
  └── depends on monocle-tui (ratatui renderer — SS-06)
  └── depends on monocle-ipc (UDS client — SS-05)
```

The `monocle` binary crate is the sole composition root. It imports from `monocle-runtime`
for daemon logic and from `monocle-tui` + `monocle-ipc` for TUI logic. No other crate has a
dependency on the `monocle` binary.

---

## Module Purity Classification

| Module / Function | Classification | Rationale |
|-------------------|---------------|-----------|
| `resolve_runtime_dir()` | Pure core (impure-adjacent) | Reads env vars and `ProjectDirs`; deterministic given same env; no I/O side effects beyond reads. Can be unit tested with env mocking. |
| `generate_token()` | Effectful shell | Calls `OsRng`; inherently non-deterministic. Tested via integration harness with mock RNG for determinism. |
| `write_lock_file()` | Effectful shell | Filesystem write via `tempfile::persist`. Integration tested with tempdir fixture. |
| `write_hooks_settings()` | Effectful shell | Filesystem write via `tempfile::persist`. Integration tested. |
| `build_server()` | Effectful shell | Constructs axum router; no I/O at construction time, but returns a type that performs I/O when served. |
| `run_server()` | Effectful shell | Runs tokio runtime, accepts TCP connections. |
| `event_bus_fan_out_task()` | Effectful shell | Reads channel, writes to UDS. |
| `daemon_start_sequence()` | Effectful shell | Orchestrates steps 1–13; all I/O. |
| Hook timeout logic | Pure core | `fn compute_deadline(budget_ms: u64) -> Instant` is deterministic. |
| Hook routing dispatch | Pure core | `fn route_hook(hook_type: &HookType) -> &'static str` is deterministic. |

---

## Behavioral Contracts

The following 12 behavioral contracts govern SS-04 behavior. Each BC is authored in its own
file under `.factory/specs/behavioral-contracts/ss-04/`. This table is a navigation index;
the authoritative contract text is in the BC files.

| BC ID | Title | Priority | Features |
|-------|-------|----------|----------|
| BC-2.04.001 | Daemon Start Sequence: Port Bind + Lock File + Token Write (SOQ-2) | P0 | F-02, F-03, F-22 |
| BC-2.04.002 | Daemon Auto-Start on TUI Launch | P0 | F-05, F-60 |
| BC-2.04.003 | `MONOCLE_NO_AUTOSTART=1` Suppresses Auto-Start | P1 | F-23, F-61 |
| BC-2.04.004 | `monocle daemon start` CLI Subcommand | P0 | F-01, F-59 |
| BC-2.04.005 | `monocle daemon stop` CLI Subcommand | P0 | F-01, F-59 |
| BC-2.04.006 | `directories::ProjectDirs::runtime_dir()` Fallback Chain | P0 | F-04 |
| BC-2.04.007 | Hook Endpoint: PreToolUse Request Routing | P0 | F-06 |
| BC-2.04.008 | Hook Endpoint: Notification Request Routing | P0 | F-06 |
| BC-2.04.009 | Hook Endpoint: Stop/SessionStart/PromptSubmit Routing | P0 | F-06 |
| BC-2.04.010 | Hook Tmpfile Generation at `runtimeDir/hooks-settings.json` | P0 | F-12, F-62 |
| BC-2.04.011 | Bounded Event Bus with Drop Counter | P0 | F-63, F-50 |
| BC-2.04.012 | JSONL Ring: Capacity and Rotation Policy | P1 | F-13 |

### BC Dependency Map

The following upstream BCs must be satisfied before SS-04 BCs can be verified:

| SS-04 BC | Depends On | Nature |
|----------|-----------|--------|
| BC-2.04.001 | BC-2.01.005 (Lock File Atomic Lifecycle) | SOQ-2 is a constraint on the lock-file-write sequence |
| BC-2.04.001 | BC-2.01.008 (Auth Token Wire Format) | Token in lock file must conform to `monocle-v1:<64-hex>` |
| BC-2.04.007..009 | BC-2.01.003 (Body Size Limit) | Body limit middleware runs before routing |
| BC-2.04.007..009 | BC-2.01.009 (Auth Header Validation) | Auth middleware runs before routing; dual-accept per ADR-0005 |
| BC-2.04.007..009 | BC-2.03.001 (EngineModule Trait) | Hook routing dispatches to `EngineModule::on_hook()` |
| BC-2.04.010 | BC-HOOK-009 (hooks-settings.json mode 0o600) | Monocle's file must match expected path and mode from gene-source |
| BC-2.04.011 | BC-2.04.007..009 (Hook Routing) | Events produced by hook routing are the event bus source |

---

## Risk Mitigations

### SOQ-2 Race Condition (F-22)

Risk: A Claude Code subprocess reads `hooks-settings.json` and obtains the auth token before
the daemon has committed the token to the lock file. If the TUI then reads the lock file and
compares the token, it would see a different value, breaking auth.

Mitigation: The start sequence mandates step 8 (lock file write) before step 9
(hooks-settings.json write). The lock file write uses `tempfile::persist` (atomic rename);
there is no window between a partial lock file state and a complete hooks-settings.json.
BC-2.04.001 formally specifies this ordering.

### Event Bus Back-Pressure (F-63)

Risk: A slow or disconnected TUI client causes the event bus to fill, blocking hook handler
tasks and violating the 300ms timeout budget.

Mitigation: Hook handlers use `try_send` (non-blocking). Full bus causes immediate event
drop with counter increment. Hook response latency is entirely decoupled from bus consumer
latency. The drop counter surfaces saturation to the operator for tuning.

### Daemon Double-Start (BC-2.04.004)

Risk: Two concurrent `monocle daemon start` invocations both pass the PID-liveness check
and both attempt to bind a port and write a lock file.

Mitigation: The lock file write uses `tempfile::persist`, which is an atomic rename on POSIX
systems. The first writer to persist wins; the second will see a valid lock file on re-check
and abort. The PID-liveness check in step 2 of the start sequence is a best-effort early
exit, not the exclusive mutex — the lock file atomic write is the true exclusion point.

---

## §Trace v1.2.0

**Adversarial Pass 2 review corrections** (F-P1D2-007) (2026-05-26):
- **F-P1D2-007** Clarified hook endpoint URL count in §Hook Tmpfile Generation §Key
  properties. The previous text "All 5 hook endpoint URLs" was ambiguous — the daemon
  serves 5 hook endpoints but hooks-settings.json only configures 4 with URLs. Rewrote
  to state explicitly: hooks-settings.json configures 4 URLs (`PreToolUse`,
  `Notification`, `Stop`, `UserPromptSubmit`); `SessionStart` is invoked by Claude
  Code's internal lifecycle (not configurable via hooks-settings.json, per §SessionStart
  Invocation Path Note); `PostToolUse` and `PreCompact` are present as reserved empty
  arrays for forward-compatibility.

## §Trace v1.0.0

**Initial production** (2026-05-26T00:00:00Z):
- SS-daemon-wiring.md created as new artifact for SS-04 per `prd-expansion-scope.md §Section 2`
  and task instruction.
- Covers: CLI interface (start/stop/default), daemon auto-start logic, 13-step start
  sequence, hook endpoint routing with timeout enforcement, bounded event bus (N=4096),
  hooks-settings.json generation with SOQ-2 ordering rationale, dependency graph, module
  purity classification, 12 behavioral contract navigation table, BC dependency map, and
  risk mitigations.
- Does NOT duplicate SS-daemon-lifecycle.md content; all infrastructure specs are referenced
  by section, not repeated.
- input-hash: [pending] — to be populated by compute-input-hash after human review.
- SE-16d PASS: 2026-05-26T00:00:00Z is the chain origin for this artifact.

## §Trace v1.1.0

**Adversarial review corrections** (F-P1D-004, F-P1D-009) (2026-05-26):
- **F-P1D-004** Priority sync with BC-INDEX (source of truth):
  - BC-2.04.003 corrected P0 → P1 (BC-INDEX §SS-04 row 3: P1).
  - BC-2.04.012 corrected P0 → P1 (BC-INDEX §SS-04 row 12: P1).
- **F-P1D-009** Added §SessionStart Invocation Path Note clarifying that
  `POST /hooks/session-start` is invoked by Claude Code's internal lifecycle mechanism,
  NOT via user-configurable hooks-settings.json entries. Implementers must NOT add a
  `SessionStart` entry to the hooks-settings.json schema. This eliminates routing
  ambiguity that could cause implementers to mistakenly configure hooks-settings.json
  to call the session-start endpoint.

## §Trace v1.3.0

**F-P12-001 (partial) — on_hook signature corrected in §Hook Endpoint Routing diagram** (2026-05-26):
- **F-P12-001** The ASCII routing diagram incorrectly described the `EngineModule::on_hook`
  call as `on_hook(hook_type, session_id, &payload)` — three separate parameters. The actual
  trait signature (engine.rs line 34; SS-engine-module.md §EngineModule Trait Signature) is
  `async fn on_hook(&self, event: HookEvent) -> HookResponse` — a single `HookEvent` enum
  value. The hook handler must first construct a `HookEvent` variant (e.g.,
  `HookEvent::PreToolUse(PreToolUseEvent { tool_name, tool_input, session_id, pid })`) from
  the deserialized `HookEnvelope` fields, then call `engine.on_hook(hook_event).await`.
  The return type is `HookResponse { decision, redirect_url, diagnostic }`, not `HookDecision`
  directly. The ASCII diagram now reflects the correct two-step construction-then-dispatch
  flow.
- **Note for PO (BC-2.04.006 fix required):** BC-2.04.006 PC-4, PC-5, PC-11 specify
  `ProjectDirs::new("monocle", "monocle", "monocle")` as the constructor. SS-config.md uses
  `ProjectDirs::from("", "", "monocle")`. These produce DIFFERENT paths on macOS: `from("",
  "", "monocle")` yields project path `monocle` → `data_local_dir()` =
  `~/Library/Application Support/monocle/`; `new("monocle", "monocle", "monocle")` treats all
  three arguments as qualifier/org/app and yields project path `monocle.monocle.monocle` →
  `data_local_dir()` = `~/Library/Application Support/monocle.monocle.monocle/`. The correct
  call is `ProjectDirs::from("", "", "monocle")` per SS-config.md. BC-2.04.006 must be updated
  by the PO. SS-daemon-wiring.md delegates constructor selection to BC-2.04.006 and
  SS-daemon-lifecycle.md; no direct fix is needed in this file.

## §Errata F-P53-001 (no version bump — clarification only) (2026-06-14)

**F-P53-001 — §SessionStart Invocation Path Note: imprecise hook-schema framing corrected.**

The implementer note at §SessionStart Invocation Path Note (lines 281-285 in v1.3.0) stated:
"The `hooks-settings.json` configures only the 5 hook types that support user-configurable
scripts (`PreToolUse`, `Notification`, `Stop`, `PostToolUse`, `UserPromptSubmit`)."

This was imprecise in three ways: (1) it said "5 hook types" when the JSON file has 6 keys
total; (2) it listed `PostToolUse` as active (it is a reserved-empty array `[]`); (3) it
omitted `PreCompact` entirely (also a reserved-empty array `[]`); (4) it did not distinguish
served HTTP endpoints (5) from hooks-settings.json JSON keys (6: 4 URL-bearing + 2 empty).

**Correction:** The note now precisely states the file has **6 JSON keys**: 4 URL-bearing
(`PreToolUse`, `Notification`, `Stop`, `UserPromptSubmit`) plus 2 reserved-empty arrays
(`PostToolUse: []`, `PreCompact: []`). The distinction between the 5 served axum endpoints
(`/hooks/pre-tool-use`, `/hooks/notification`, `/hooks/stop`, `/hooks/prompt-submit`,
`/hooks/session-start`) and the 6 JSON keys is now explicit, with a BC-2.04.010 PC-3
reference. The "do NOT add SessionStart key" implementer directive is preserved.

**Normative impact:** None. The actual schema (BC-2.04.010 PC-3) is unchanged. This is a
precision improvement to the §SessionStart Invocation Path Note to match the already-correct
framing in §Hook Tmpfile Generation §Key properties (F-P1D2-007, v1.2.0). No new wire codes,
variants, or obligations introduced. ERRATA-NO-BUMP disposition confirmed.
