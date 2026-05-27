---
document_type: architecture-section
level: L3
section: "ipc"
subsystem: SS-05
version: "1.0.0"
status: draft
producer: vsdd-factory:architect
phase: phase-1-expansion
timestamp: 2026-05-26T02:00:00Z
inputs:
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/architecture/SS-daemon-wiring.md, version: "1.0.0"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.13"}
  - {path: .factory/specs/product-brief.md, version: "1.4.30"}
  - {path: .factory/specs/research/domain-monocle-vision-synthesis.md, version: "1.1.3"}
input-hash: "[pending]"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: IPC

## Scope

SS-05 defines the `monocle-ipc` crate: the internal transport between TUI clients and the
daemon. The daemon is the UDS server; TUI instances are clients. Multiple TUI clients can
connect simultaneously — the vision §Process Topology diagram shows three clients (A, B, C)
fanning out from the daemon's broker. The IPC layer delivers session state, hook events, and
permission prompts from the daemon to each TUI client, and routes permission decisions from
TUI clients back to the daemon.

The daemon creates the UDS socket as step 10 of the daemon start sequence (SS-daemon-wiring.md
§Daemon Start Sequence). SS-05 specifies the protocol that operates over that socket: framing,
message types, connection lifecycle, reconnection behavior, and the SOQ-3 overlay-clear invariant.

Phase 1 constraint: UDS only. No `mmap`, `shm_open`, or shared-memory primitives (OQ-08,
BC-2.05.008). The `Transport` trait defined in this document provides the abstraction point for
the Phase 4 shared-memory transport variant without requiring a re-architecture of the IPC layer.

---

## Transport Layer

### Socket Path

The Unix domain socket is created at `<runtime_dir>/monocle.sock`. The `runtime_dir` is
resolved by the same fallback chain as the lock file (SS-daemon-wiring.md §Daemon Auto-Start
Logic). The socket file mode is `0o600` — readable and writable only by the owning user.

Rationale for `0o600`: A UDS socket at `0o600` ensures that only the user who started the
daemon can connect TUI clients. This is the same access model as the lock file and
hooks-settings.json. A malicious process running as a different user cannot inject IPC
messages or intercept permission decisions.

### Lifecycle

- **Daemon bind:** On startup (step 10 of daemon start sequence), the daemon calls
  `UnixListener::bind(&sock_path)` after checking for a stale socket file. If the socket
  path exists, it is removed before rebind (stale socket from a prior crashed daemon).
  This mirrors the lock-file stale-check at step 2.
- **Daemon shutdown:** On graceful shutdown, the daemon removes `monocle.sock` alongside
  `monocle.lock` and `hooks-settings.json`.
- **TUI connect:** The TUI calls `UnixStream::connect(&sock_path)`. Connection attempt begins
  after a successful lock-file read (daemon is confirmed alive). If the socket file does not
  exist, the TUI waits for the daemon start sequence to complete (up to 5 seconds per
  SS-daemon-wiring.md §Daemon Auto-Start Logic step 4).

### Transport Trait

```rust
/// Phase 1: implemented by UdsTransport.
/// Phase 4: an additional ShmTransport will implement this trait.
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send_message(&mut self, msg: &ServerToClient) -> Result<(), IpcError>;
    async fn recv_message(&mut self) -> Result<ClientToServer, IpcError>;
}
```

All IPC logic in `monocle-ipc` operates against `Transport`. The Phase 1 `UdsTransport`
implements `Transport` over `tokio::net::UnixStream`. This ensures that Phase 4 shared-memory
transport can be added as a second `Transport` implementation without touching message dispatch,
framing, or reconnection logic.

---

## Framing Protocol

Messages are framed over the raw UDS byte stream using a length-prefix scheme.

### Wire Format

```
┌──────────────────────┬─────────────────────────────────────────────┐
│  Length (4 bytes LE) │  JSON payload (N bytes, UTF-8)              │
└──────────────────────┴─────────────────────────────────────────────┘
```

- **Length prefix:** 4 bytes, little-endian `u32`. Encodes the byte length of the JSON
  payload that follows. The length prefix itself is not included in the count.
- **Payload:** `serde_json`-serialized message. UTF-8 encoded. No trailing newline or
  null terminator.
- **Maximum message size:** 262,144 bytes (256 KiB). This matches the 256 KiB HTTP body
  limit enforced by the hook receiver (BC-2.01.003). A message exceeding this limit is a
  protocol error; the receiver closes the connection with `IpcError::MessageTooLarge`.

### Framing Implementation

```rust
/// Write a framed message to the stream.
pub async fn write_framed(
    stream: &mut (impl AsyncWrite + Unpin),
    payload: &[u8],
) -> Result<(), IpcError> {
    if payload.len() > MAX_MESSAGE_BYTES {
        return Err(IpcError::MessageTooLarge { size: payload.len() });
    }
    let len = (payload.len() as u32).to_le_bytes();
    stream.write_all(&len).await?;
    stream.write_all(payload).await?;
    Ok(())
}

/// Read a framed message from the stream.
pub async fn read_framed(
    stream: &mut (impl AsyncRead + Unpin),
) -> Result<Vec<u8>, IpcError> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_MESSAGE_BYTES {
        return Err(IpcError::MessageTooLarge { size: len });
    }
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok(payload)
}

pub const MAX_MESSAGE_BYTES: usize = 262_144;
```

JSON is chosen over protobuf for IPC in Phase 1 because:
1. Message types reference `EnrichedSession` and `HookEvent` structs that are already
   serialized via `serde` for the JSONL ring. Re-using `serde_json` adds zero new
   dependencies.
2. Human-readable wire format simplifies debugging and integration testing.
3. The `monocle-proto` prost schemas are reserved for the Phase 4 cross-host federation
   wire format (OQ-07); local IPC does not need the protobuf serialization overhead.

---

## Message Types

All message types derive `#[derive(Debug, Clone, Serialize, Deserialize)]`. All public enums
and message structs carry `#[non_exhaustive]` per the SS-02 extensibility policy
(BC-2.02.003). The `#[forbid(unsafe_code)]` attribute is declared at the crate root.

### Server-to-Client Messages

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClient {
    /// Sent immediately on connect. Pushes the full current state snapshot to the
    /// newly connected TUI client so it can render without waiting for incremental updates.
    InitialState {
        sessions: Vec<EnrichedSession>,
        ring_tail: Vec<HookEvent>,
        overlay_stack: Vec<PermissionPromptPayload>,
        drop_counter: u64,
    },

    /// Session roster changed: a session was added, removed, or enriched.
    /// Contains the full current session list (not a diff).
    SessionListUpdate {
        sessions: Vec<EnrichedSession>,
    },

    /// A hook event was ingested by the daemon.
    HookEventReceived {
        hook_type: HookType,
        session_id: String,
        /// First 256 bytes of the payload (to keep message size bounded).
        payload_excerpt: String,
        /// Time from hook POST receipt to daemon ACK, in milliseconds.
        latency_ms: u64,
    },

    /// A PreToolUse hook arrived with `decision_required: true`.
    /// The daemon is holding the HTTP response open, awaiting a TUI decision.
    PermissionPromptQueued {
        prompt_id: Uuid,
        session_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
        /// Present when tool_name is "Edit" or similar file-mutation tools.
        old_content: Option<String>,
        new_content: Option<String>,
    },

    /// Another connected TUI client already resolved this prompt.
    /// The receiving client should pop this prompt_id from its local overlay stack.
    PermissionPromptResolved {
        prompt_id: Uuid,
    },

    /// Drop counter changed. Sent whenever the drop counter increments.
    DropCounterUpdate {
        count: u64,
    },

    /// Keepalive response.
    Pong,
}
```

### Client-to-Server Messages

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientToServer {
    /// User responded to a permission prompt.
    PermissionDecision {
        prompt_id: Uuid,
        decision: PermissionDecision,
    },

    /// Keepalive probe. Daemon responds with Pong.
    Ping,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    /// Accept this invocation only.
    Accept,
    /// Accept this tool + input pattern always (daemon records for auto-accept).
    AcceptAlways,
    /// Reject this invocation.
    Reject,
}
```

### Supporting Types

`EnrichedSession`, `HookEvent`, and `HookType` are defined in `monocle-core` (SS-03). The
`PermissionPromptPayload` type used in `InitialState.overlay_stack` is the same struct as the
fields inlined in `PermissionPromptQueued` — the TUI can re-render queued prompts from the
initial state push without re-requesting them.

`Uuid` is `uuid::Uuid` with the `serde` feature enabled. `prompt_id` is generated by the
daemon when the `PermissionPromptQueued` message is first created; it is stable for the
lifetime of the pending decision.

---

## Connection Lifecycle

### Phase 1: Connect

1. TUI resolves `runtime_dir` and reads `monocle.lock` to confirm daemon liveness.
2. TUI opens `UnixStream::connect("<runtime_dir>/monocle.sock")`.
3. Daemon accepts the connection; spawns a dedicated Tokio task for the client session.
4. Daemon immediately sends `ServerToClient::InitialState` containing:
   - The full current `Vec<EnrichedSession>` roster.
   - The last N events from the RAM ring (ring tail) as `Vec<HookEvent>`.
   - Any currently queued `Vec<PermissionPromptPayload>` entries awaiting decision.
   - The current drop counter value.
5. TUI renders its initial state from the `InitialState` message. No subsequent poll is
   needed; all updates arrive as push messages from this point.

### Phase 2: Streaming Updates

After the initial state push, the daemon sends incremental updates as events occur:

- `SessionListUpdate` — whenever the `EngineModule` registry reports a session change.
- `HookEventReceived` — for every hook event that passes through the event bus fan-out
  task (SS-daemon-wiring.md §Bounded Event Bus).
- `PermissionPromptQueued` — whenever a PreToolUse hook with `decision_required: true`
  is received and the daemon creates a pending-decision entry.
- `PermissionPromptResolved` — when one TUI client resolves a prompt, all other connected
  clients are notified so they can remove the stale entry from their overlay stacks.
- `DropCounterUpdate` — whenever `DaemonState.drop_counter` increments (sent with debounce:
  at most once per 100ms to avoid flooding the TUI on saturation events).

The TUI sends `ClientToServer::PermissionDecision` when the user accepts or rejects a
permission prompt in the overlay. The daemon receives this, resolves the pending
`oneshot::channel` for the waiting HTTP handler (see SS-daemon-wiring.md §PreToolUse
Permission Decision Hold), broadcasts `PermissionPromptResolved` to all other clients,
and sends the HTTP decision response to Claude Code.

### Phase 3: Disconnect

Normal disconnect occurs when the TUI exits or the daemon shuts down. Abnormal disconnect
(network interruption, daemon crash) is handled by the reconnection logic below.

When the daemon receives a client disconnect (the per-client Tokio task gets an EOF or
`BrokenPipe`), it removes the client from the fan-out subscriber list and drops the per-client
sender. No explicit goodbye message is required.

---

## Reconnection Behavior

The TUI monitors its `UnixStream` connection via the receive loop. If `read_framed` returns an
error (EOF, `BrokenPipe`, or `ConnectionReset`), the TUI enters reconnection mode:

1. **Overlay clear (SOQ-3).** Immediately clear all entries from the `VecDeque<PromptModal>`
   overlay stack. Rationale: Claude Code subprocesses time out unanswered hook responses; if
   the TUI reconnects later, those prompts are no longer actionable. Cleared prompts prevent
   ghost approvals. This is BC-2.05.007 (IPC layer) and BC-2.06.016 (TUI layer acting on the
   signal from IPC).
2. **Render "reconnecting…".** The TUI renders a status-bar indicator `[daemon: reconnecting…]`
   during the reconnect window.
3. **Re-read lock file.** After each failed reconnect attempt, the TUI re-reads
   `<runtime_dir>/monocle.lock` to discover whether a new daemon has started (new port and UDS
   socket path). This handles the case of a daemon restart.
4. **Retry loop.** The TUI attempts reconnection with exponential backoff starting at 250ms,
   capped at 2 seconds. Total reconnect window: 5 seconds before the TUI transitions to
   "daemon offline" mode.
5. **Successful reconnect.** On connection, the daemon sends a fresh `InitialState` push.
   The TUI rebuilds its entire state from this message. AppMode resets to `Dashboard` if it
   was in `Overlay` (since the overlay stack was already cleared in step 1).
6. **Reconnect timeout.** If no reconnection succeeds within 5 seconds, the TUI renders
   `[daemon: offline]` and enters passive mode. It polls the lock file every 5 seconds; when
   a new daemon is detected, it retries the reconnect loop.

---

## SOQ-3: Overlay Clear on Disconnect

SOQ-3 is a safety invariant defined in the product brief (line 145): overlay clears on daemon
disconnect. The rationale is that Claude Code's PreToolUse hook waits for a response with a
300ms timeout. If the daemon restarts, the pending decision channel is destroyed. The Claude
Code subprocess will have already timed out and applied its default (fail-open or fail-closed).
Any `PromptModal` still visible in the TUI is stale: approving it would send a decision to a
channel that no longer exists.

The IPC layer enforces SOQ-3 at the transport level: when `UdsTransport` detects a connection
loss, it emits a `TransportEvent::Disconnected` signal before returning the error. The TUI
event loop receives this signal and clears the `VecDeque<PromptModal>` before initiating
reconnection. This is specified in BC-2.05.007 (IPC side) and BC-2.06.016 (TUI side).

The clearing must happen before any reconnect attempt — not after. The sequence is:
`connection loss detected → overlay cleared → reconnect loop begins → (if success) InitialState push → TUI rebuilds state`.

---

## Phase 1 Transport Constraint (BC-2.05.008)

The `monocle-ipc` crate MUST NOT import `libc::mmap`, `nix::sys::mman`, `shared_memory`,
or any crate that provides shared-memory or POSIX shared-memory primitives. This prohibition
is enforced by a `cargo deny` rule and a semgrep check in CI.

The Phase 4 shared-memory transport variant (OQ-08) is deferred. When Phase 4 arrives, it
will implement the `Transport` trait above. No `monocle-ipc` structural changes are needed;
only the addition of a `ShmTransport` struct.

The `monocle-ipc` crate root carries:
```rust
#![forbid(unsafe_code)]
```

This ensures that even if a contributor attempts to add inline `unsafe` for shared-memory
access, the compiler rejects it. Phase 4 `ShmTransport` will live in a separate crate
(e.g., `monocle-ipc-shm`) with an explicit `#![allow(unsafe_code)]` and a security review
gate.

---

## Behavioral Contracts

The following 8 behavioral contracts govern SS-05 behavior. Each BC will be authored in its
own file under `.factory/specs/behavioral-contracts/ss-05/`. This table is a navigation
index; the authoritative contract text is in the BC files.

| BC ID | Title | Priority | Features |
|-------|-------|----------|----------|
| BC-2.05.001 | UDS Server Bind at `runtimeDir/monocle.sock` | P0 | F-24 |
| BC-2.05.002 | TUI Client Connects to UDS and Receives Initial State Push | P0 | F-24, F-26 |
| BC-2.05.003 | IPC Message Types: SessionListUpdate | P0 | F-26 |
| BC-2.05.004 | IPC Message Types: HookEventReceived | P0 | F-26 |
| BC-2.05.005 | IPC Message Types: PermissionPromptQueued | P0 | F-26 |
| BC-2.05.006 | TUI Reconnects After Daemon Restart | P0 | F-27 |
| BC-2.05.007 | Overlay Stack Cleared on Daemon Disconnect (SOQ-3) | P0 | F-28, F-39 |
| BC-2.05.008 | UDS-Only in Phase 1 (No Shared-Memory Transport) | P1 | F-25 |

### BC Dependency Map

| SS-05 BC | Depends On | Nature |
|----------|-----------|--------|
| BC-2.05.001 | BC-2.04.001 (Daemon Start Sequence) | UDS socket created as step 10 of daemon start; lock file must be written first (SOQ-2) |
| BC-2.05.002 | BC-2.05.001 (UDS Bind) | Client can only connect after server has bound |
| BC-2.05.005 | BC-2.04.007 (PreToolUse Routing) | PreToolUse hook routing produces the event that triggers the PermissionPromptQueued push |
| BC-2.05.007 | BC-2.06.016 (Overlay Cleared on Disconnect) | IPC layer signals disconnect; TUI BC-2.06.016 responds by clearing the VecDeque |

---

## Dependency Graph

```
monocle-ipc
  ├── depends on monocle-core (EnrichedSession, HookEvent, HookType — SS-03)
  ├── depends on tokio (tokio::net::UnixStream, UnixListener, AsyncRead, AsyncWrite)
  ├── depends on serde + serde_json (message serialization)
  └── depends on uuid (prompt_id generation, serde feature)

monocle-tui (SS-06, consumer)
  └── depends on monocle-ipc (UdsTransport, ServerToClient, ClientToServer)

monocle-runtime (SS-04, daemon side)
  └── depends on monocle-ipc (UdsTransport server-side, message types)
```

`monocle-ipc` has no dependency on `monocle-runtime` or `monocle-tui`. It is a pure
transport library. The daemon and TUI both depend on it as consumers. This ensures that the
`Transport` trait and message types remain decoupled from either side's implementation logic.

---

## Module Purity Classification

| Module / Function | Classification | Rationale |
|-------------------|---------------|-----------|
| `ServerToClient` enum | Pure core | Data type only; `#[derive(Serialize, Deserialize)]`. No I/O. |
| `ClientToServer` enum | Pure core | Data type only. No I/O. |
| `PermissionDecision` enum | Pure core | Data type only. No I/O. |
| `write_framed()` | Effectful shell | Async write to `UnixStream`. Integration tested with `tokio::io::duplex`. |
| `read_framed()` | Effectful shell | Async read from `UnixStream`. Integration tested. |
| `UdsTransport::send_message()` | Effectful shell | Serializes + calls `write_framed`. |
| `UdsTransport::recv_message()` | Effectful shell | Calls `read_framed` + deserializes. |
| Reconnect loop | Effectful shell | Reads lock file, opens new `UnixStream`. |
| SOQ-3 overlay-clear signal | Pure core | Emitting `TransportEvent::Disconnected` is a pure state transition in the event type; the clearing side-effect is in the TUI event handler. |

---

## Risk Mitigations

### Stale Socket File After Daemon Crash

Risk: The daemon crashes without removing `monocle.sock`. The next daemon start fails to bind
because the socket file already exists.

Mitigation: The daemon start sequence (step 10 in SS-daemon-wiring.md) explicitly removes any
existing socket file before `UnixListener::bind`. This is the same pattern used for stale lock
files at step 2. BC-2.05.001 specifies this behavior as a postcondition.

### Multiple TUI Clients Resolving the Same Prompt

Risk: Two TUI clients (two terminal tabs) both see `PermissionPromptQueued`. Both users press
Accept simultaneously. The daemon receives two `PermissionDecision` messages for the same
`prompt_id`.

Mitigation: The daemon's pending-decision registry uses `oneshot::channel` per prompt. The
first `PermissionDecision` to arrive resolves the oneshot and removes the entry from the
registry. The second `PermissionDecision` finds no entry and is silently discarded. The daemon
broadcasts `PermissionPromptResolved` after the first resolution; the second TUI client's
overlay stack entry will have already been removed by that broadcast before its own decision
message arrives at the daemon.

### TUI Reconnects During a Pending Decision

Risk: TUI disconnects while the user was viewing a `PermissionPromptQueued` overlay. The user
reconnects and sees a stale prompt. They approve it after the Claude Code timeout has already
expired.

Mitigation: SOQ-3 (overlay clear on disconnect) removes all `VecDeque<PromptModal>` entries
on TUI-side disconnect. The `InitialState` push on reconnect contains only prompts that are
still pending in the daemon's registry (i.e., still within the 300ms timeout window). Stale
prompts are never re-pushed.

---

## §Trace v1.0.0

**Initial production** (2026-05-26T02:00:00Z):
- SS-ipc.md created as new artifact for SS-05 per `prd-expansion-scope.md §Section 2 SS-05`
  and task instruction.
- Covers: transport layer (UDS, `Transport` trait, Phase 1 constraint), framing protocol
  (4-byte LE length prefix, 256 KiB limit, JSON payload), message types (7 ServerToClient
  + 2 ClientToServer variants, all `#[non_exhaustive]`), connection lifecycle (connect,
  streaming, disconnect), reconnection behavior (SOQ-3 overlay clear, 5-second window,
  exponential backoff, lock-file re-read), Phase 1 transport constraint (`#![forbid(unsafe_code)]`,
  `Transport` trait abstraction for Phase 4), 8 BC navigation table with dependency map,
  dependency graph, module purity classification, and risk mitigations.
- Consistent with SS-daemon-wiring.md: UDS socket at `runtimeDir/monocle.sock` created at
  step 10 of daemon start sequence; `oneshot::channel` per-prompt pending-decision registry
  referenced from SS-daemon-wiring.md §PreToolUse Permission Decision Hold.
- `PermissionDecision::AcceptAlways` maps to `{"decision": "always"}` per BC-2.06.012.
- `PermissionDecision::Reject` maps to `{"decision": "deny"}` per BC-2.06.013.
- input-hash: [pending] — to be populated by compute-input-hash.
- SE-16d PASS: 2026-05-26T02:00:00Z > chain high-water 2026-05-26T01:00:00Z (monotonic).
