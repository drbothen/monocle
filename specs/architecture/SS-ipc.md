---
document_type: architecture-section
level: L3
section: "ipc"
subsystem: SS-05
version: "1.9.0"
status: draft
producer: vsdd-factory:architect
phase: phase-1-expansion
timestamp: 2026-05-28T00:00:00Z
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
        /// Last N events from the RAM ring in `HookEventRecord` format (BC-2.04.012 PC-1).
        /// The TUI renders event ribbon display from `hook_type`, `session_id`,
        /// `timestamp_micros`, and `tool_name` fields. Using the ring's native storage type
        /// avoids lossless-vs-lossy reconstruction ambiguity; see ADR-0006.
        ring_tail: Vec<HookEventRecord>,
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
    /// All prompt fields are carried in `PermissionPromptPayload` — the same type
    /// used in `InitialState.overlay_stack` — so TUI rendering code is shared.
    PermissionPromptQueued {
        payload: PermissionPromptPayload,
    },

    /// The prompt identified by prompt_id has been resolved. All connected TUI clients
    /// MUST remove this prompt_id from their local overlay stacks.
    ///
    /// This message is sent in TWO situations:
    /// 1. A TUI client sent PermissionDecision for this prompt_id (another client resolved it).
    /// 2. The daemon's hook timeout expired for this prompt_id (PreToolUse 300ms budget).
    ///
    /// TUI clients treat PermissionPromptResolved identically regardless of source —
    /// pop the matching PromptModal from the VecDeque<PromptModal> if present; no-op if absent.
    PermissionPromptResolved {
        prompt_id: Uuid,
    },

    /// Drop counter changed. Sent whenever the drop counter increments.
    DropCounterUpdate {
        count: u64,
    },

    /// Keepalive response.
    /// NOTE: Pong is reserved for Phase 2 keepalive detection.
    /// Phase 1 implementations MUST accept and silently discard Pong if received —
    /// do NOT close the connection or return an error on receipt of an unexpected Pong.
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
    /// NOTE: Ping is reserved for Phase 2 keepalive detection.
    /// Phase 1 implementations MUST accept and silently discard Ping if received —
    /// do NOT close the connection or return an error on receipt of an unexpected Ping.
    Ping,
}

// Phase 1 note on Ping/Pong (F-P1D-011):
// Ping and Pong variants are present in the enum for forward-compatibility per the
// #[non_exhaustive] + reserved-variant policy (BC-2.02.003). Keepalive detection
// is a Phase 2 feature. In Phase 1:
// - The daemon does NOT send spontaneous Pong messages.
// - The TUI does NOT send spontaneous Ping messages.
// - If either side receives a Ping or Pong unexpectedly, it MUST silently discard it.
// - No BC is required for Ping/Pong in Phase 1; the types are present solely to
//   reserve their wire encoding so Phase 2 can activate them without a breaking change.

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionDecision {
    /// Accept this invocation only. Wire value: `"accept"`.
    #[serde(rename = "accept")]
    Accept,
    /// Accept this tool + input pattern always (daemon records for auto-accept).
    /// Wire value: `"always"` — matches Claude Code hook protocol.
    #[serde(rename = "always")]
    AcceptAlways,
    /// Reject this invocation. Wire value: `"deny"` — matches Claude Code hook protocol.
    #[serde(rename = "deny")]
    Reject,
}
```

### Supporting Types

`EnrichedSession`, `HookEvent`, and `HookType` are defined in `monocle-core` (SS-03).
`HookEventRecord` is defined in `monocle-runtime::ring` (SS-01). The `monocle-ipc` crate
depends on `monocle-runtime` for `HookEventRecord` in the `InitialState` message; this is
the only `monocle-runtime` dependency in `monocle-ipc`.

```rust
/// Shared payload for permission prompt data, used in both `InitialState.overlay_stack`
/// and `PermissionPromptQueued`. Carries all fields the TUI needs to render a prompt.
///
/// Using a shared struct (rather than inlining fields in each message variant) ensures
/// that TUI rendering code is unified: the same `render_prompt_modal(payload)` function
/// handles both the initial-state-push path and the live-queued-prompt path.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPromptPayload {
    /// Stable ID generated by the daemon when the PreToolUse hook first arrives.
    /// Remains stable for the lifetime of the pending decision.
    pub prompt_id: Uuid,
    /// String identifier originating from Claude Code (not daemon-generated).
    /// Matches `session_id: String` in `EnrichedSession`, `HookEvent`,
    /// `HookEventReceived`, `HookEnvelope`, and `PromptModal`.
    /// Only `prompt_id` is `Uuid` (daemon-generated); `session_id` is always `String`.
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    /// Present when tool_name is "Edit" or similar file-mutation tools.
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}
```

`InitialState.overlay_stack` is `Vec<PermissionPromptPayload>` — the daemon pushes all
currently-pending prompts in the initial state push so the TUI can rebuild its overlay
stack on connect or reconnect without re-requesting individual prompts.

`Uuid` is `uuid::Uuid` with the `serde` feature enabled. `prompt_id` is generated by the
daemon when the `PermissionPromptQueued` message is first created; it is stable for the
lifetime of the pending decision.

```rust
/// Events emitted by the UDS transport layer to the TUI event handler.
/// These are NOT IPC wire messages — they are local notifications about
/// the transport connection state.
///
/// TransportEvent is defined in `monocle-ipc` and consumed by `monocle-tui`.
/// It is NOT serialized over the wire — it is a process-local signal from
/// the transport layer to the TUI event handler. It is separate from
/// `ServerToClient` (wire messages from daemon) and `ClientToServer`
/// (wire messages to daemon).
#[derive(Debug, Clone)]
pub enum TransportEvent {
    /// The UDS connection to the daemon was lost (EOF, BrokenPipe,
    /// ConnectionReset). Triggers SOQ-3 overlay clear on the TUI side.
    Disconnected,
    /// The UDS connection to the daemon was re-established after a
    /// disconnect. The TUI will receive a fresh InitialState push.
    Reconnected,
}
```

`TransportEvent` is intentionally NOT `#[non_exhaustive]` — it represents a closed set of
transport states that the TUI must handle exhaustively. Adding a new transport state in the
future would require explicit TUI-side handling by design. It does NOT derive
`Serialize`/`Deserialize` because it is never written to the wire; it is a process-local
signal only.

---

## Connection Lifecycle

### Phase 1: Connect

1. TUI resolves `runtime_dir` and reads `monocle.lock` to confirm daemon liveness.
2. TUI opens `UnixStream::connect("<runtime_dir>/monocle.sock")`.
3. Daemon accepts the connection; spawns a dedicated Tokio task for the client session.
4. Daemon immediately sends `ServerToClient::InitialState` containing:
   - The full current `Vec<EnrichedSession>` roster.
   - The last N events from the RAM ring as `Vec<HookEventRecord>` (ring's native storage
     type per BC-2.04.012 PC-1; avoids lossless-vs-lossy reconstruction; see ADR-0006).
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
- `PermissionPromptResolved` — sent to ALL connected TUI clients in two cases:
  (a) when one TUI client resolves a prompt (all other clients remove the stale entry); and
  (b) when the daemon's hook timeout expires for a pending `prompt_id` (PreToolUse 300ms budget
  reached). In case (b), the daemon has already sent the fail-open/fail-closed HTTP response
  to Claude Code; the TUI clients must be notified so they remove the now-stale overlay entry.
  TUI clients handle `PermissionPromptResolved` identically in both cases: pop the matching
  `PromptModal` from the `VecDeque` if present; no-op if already absent.
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

### §TUI IPC Read Loop Pattern

> **WARNING:** `read_framed` performs sequential `read_exact` calls. It is NOT
> cancellation-safe across `tokio::time::timeout` or `tokio::select!` arms that may drop
> the future mid-read. Doing so will lose bytes from the kernel socket buffer and corrupt
> frame alignment.

**Forbidden pattern — cancellation-unsafe; will corrupt the frame stream:**

```rust
// FORBIDDEN — cancellation-unsafe; will corrupt the frame stream:
match tokio::time::timeout(
    Duration::from_millis(1),
    read_framed::<_, ServerToClient>(&mut transport),
).await {
    Ok(Ok(msg)) => { /* handle msg */ }
    Err(_timeout) => { /* loop again — bytes may have been consumed from kernel buffer */ }
}
```

**Canonical pattern — dedicated reader task + bounded `mpsc::channel(64)`:**

- Spawn a dedicated reader task that owns the transport exclusively:

```rust
/// Spawn a dedicated IPC reader task.
/// Returns a JoinHandle; the event loop holds the Receiver.
pub fn spawn_ipc_reader(
    mut transport: UdsClientTransport,
    tx: mpsc::Sender<ServerToClient>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match transport.recv_message().await {
                Ok(msg) => {
                    if tx.send(msg).await.is_err() {
                        // Receiver dropped — event loop has exited; stop reading.
                        break;
                    }
                }
                Err(_) => {
                    // EOF, BrokenPipe, or ConnectionReset — signal disconnect.
                    let _ = tx.send(ServerToClient::Disconnected).await;
                    break;
                }
            }
        }
    })
}
```

- Event loop holds `mpsc::Receiver<ServerToClient>` and `JoinHandle<()>`; drains via
  `ipc_rx.try_recv()` on each keyboard tick (16ms / ~60Hz cadence).

**Design invariants:**

- **Channel capacity:** 64. Bounded backpressure preserves at-least-once delivery per
  BC-2.05.002 Invariant 4; 64 covers typical burst sizes without unbounded memory growth.
- **Drop policy on full:** BLOCK — sender uses `.await` (NOT `try_send`). Silent drop on
  full would violate BC-2.05.002 Invariant 4.
- **Keyboard polling cadence:** 16ms (~60Hz). The keyboard tick is the natural rate-limiter
  of the consumer drain loop; the IPC reader task is decoupled and runs at the socket's
  natural rate.
- **Transport ownership:** reader task takes exclusive `move` ownership of
  `UdsClientTransport`. The event loop holds `mpsc::Receiver` and `JoinHandle`. No
  `Arc<Mutex<UdsClientTransport>>`.
- **Reconnect handoff:** on disconnect (`Ok(Err(_))` or channel `Disconnected`):
  1. `reader_handle.abort()` — terminate the stale reader task.
  2. Invoke SOQ-3 handler — clear the TUI's local `VecDeque<PromptModal>` overlay stack.
  3. Call `monocle_ipc::reconnect::reconnect(...)` to obtain a fresh
     `(UdsClientTransport, EventReceiver)`.
  4. Spawn a new `spawn_ipc_reader(new_transport, ipc_tx.clone())` task.

**Reference implementation:** `crates/monocle-tui/src/app.rs::spawn_ipc_reader` (S-025).

**Proof of cancellation safety:** integration test
`test_bc_2_05_002_pc_6_no_frame_corruption_across_inter_message_gap` validates that
frame alignment is preserved across inter-message gaps and that no bytes are lost.

**Cross-references:**
- BC-2.05.002 — At-Least-Once delivery (Invariant 4: `prompt_id` idempotency)
- BC-2.05.007 — SOQ-3 overlay ordering
- BC-2.05.006 — Reconnect backoff policy

---

## §Client Disconnect and Overlay Persistence

This section documents the design decision that `ClientToServer` has no `ClientDisconnect`
variant, and explains why the overlay system is correct under all disconnect scenarios.

### The Daemon's Pending-Prompt Registry Is Client-Independent

The daemon maintains a pending-prompt registry: a `HashMap<Uuid, oneshot::Sender<PermissionDecision>>`
keyed by `prompt_id`. Each entry is created when a `PreToolUse` hook arrives with
`decision_required: true` and destroyed when:
(a) a `PermissionDecision` is received from any TUI client, or
(b) the 300ms hook timeout expires.

**Client connections and disconnections do not affect this registry.** A prompt's
lifetime is tied to the hook's response timeout window — not to the presence or absence
of any TUI client. This is intentional: the daemon is the durable authority for pending
decisions; TUI clients are display-and-input surfaces that come and go.

### Why No `ClientDisconnect` Message Is Needed

tmux kills the TUI process when the user hides the popup (`Ctrl-\`). A killed process
has no opportunity to send a graceful farewell — the OS delivers `SIGKILL` and the UDS
connection closes abruptly. Relying on a `ClientDisconnect` message for correctness would
create a hole: a message that is sent on normal exit but never sent on crash or kill.

The architecture handles this correctly without any goodbye message:
- The daemon detects EOF on the per-client `UnixStream`.
- It removes the client from the fan-out subscriber list and drops the per-client sender.
- The pending-prompt registry is unaffected. Any prompts awaiting decision remain in the
  registry until resolved or timed out.

There is no behavior difference between "TUI closed gracefully" and "TUI was killed". Both
produce the same daemon-side outcome: the connection is dropped and the client is removed
from the fan-out list. This is correct and by design.

### TUI Show/Hide Cycle (Normal Operation)

The expected user workflow:
1. **TUI hides (Ctrl-\):** tmux kills the TUI process. The UDS connection closes. The
   daemon detects EOF, removes the client from the fan-out list. The daemon's
   pending-prompt registry is unaffected — it persists with all pending prompts intact.
2. **TUI shows (Ctrl-\ again):** tmux spawns a NEW TUI process. It connects to
   `<runtime_dir>/monocle.sock`. The daemon sends `ServerToClient::InitialState` with
   the full current state: `sessions`, `ring_tail`, `drop_counter`, and the current
   `overlay_stack` (all prompts still pending in the registry). The TUI populates its
   `VecDeque<PromptModal>` from `overlay_stack` and renders any pending prompts.

This is safe because prompts that survive the TUI hide/show cycle are still within
their 300ms window (or have already timed out and been resolved via the timeout path).
The `InitialState.overlay_stack` only contains prompts that are genuinely still pending
in the daemon registry at the moment of connection — never stale entries.

### SOQ-3 Is a TUI-Side Concern Only

SOQ-3 applies when the TUI detects a `TransportEvent::Disconnected` from the daemon —
meaning the daemon itself has restarted or crashed. In this case:
- The TUI clears its LOCAL `VecDeque<PromptModal>` overlay stack.
- Rationale: the daemon that held the pending-decision channels is gone. Any prompts
  that were in-flight have already timed out. The TUI must not show overlay entries for
  decisions that can no longer be acted upon.
- The daemon's registry is irrelevant here: the daemon crashed and its state is gone.
  When a new daemon starts, it has an empty registry. The `InitialState` sent on
  reconnect will have an empty `overlay_stack`.

SOQ-3 is therefore a one-sided TUI invariant: **the TUI clears its local overlay stack
when it loses the daemon connection.** The daemon never clears anything in response to a
client disconnect — there is nothing to clear. The daemon's registry is driven by hook
arrivals and timeouts, not by client lifecycle.

### Summary

| Scenario | Daemon registry | TUI overlay stack |
|----------|----------------|-------------------|
| TUI hides (`Ctrl-\`) | Unaffected | Destroyed with the process |
| TUI shows (`Ctrl-\`) | Unaffected | Rebuilt from `InitialState.overlay_stack` |
| Daemon crash (SOQ-3) | Destroyed with the daemon | TUI clears on `TransportEvent::Disconnected` |
| Normal daemon shutdown | Graceful: registry flushed before exit | TUI clears on `TransportEvent::Disconnected` |

---

## Reconnection Behavior

The TUI monitors its `UnixStream` connection via the dedicated IPC reader task (see
§TUI IPC Read Loop Pattern). The reader task calls `recv_message()` (which calls
`read_framed` internally) in a blocking loop. On `recv_message` returning an error
(EOF, `BrokenPipe`, or `ConnectionReset`), the task sends a disconnect signal over
the `mpsc::channel(64)` and exits. The event loop receives the signal and enters
reconnection mode. **Do NOT poll `read_framed` directly inside a `tokio::select!` or
`tokio::time::timeout` arm — see §TUI IPC Read Loop Pattern for the forbidden pattern
and the canonical replacement.**

1. **Overlay clear (SOQ-3) — TUI LOCAL stack only.** Immediately clear all entries from the
   TUI's local `VecDeque<PromptModal>` overlay stack. This clears the TUI's in-memory view
   of pending prompts — it does NOT affect the daemon's pending-prompt registry (which is
   client-independent; see §Client Disconnect and Overlay Persistence). Rationale: the daemon
   that held the pending-decision channels has restarted; any prompts that were in-flight have
   already timed out and been acted upon by the hook timeout path. Cleared prompts prevent ghost
   approvals on a stale overlay. On successful reconnect, the daemon sends a fresh `InitialState`
   push; the TUI rebuilds its overlay stack from `InitialState.overlay_stack`, which contains
   only prompts that are genuinely still pending in the new daemon's registry. This is
   BC-2.05.007 (IPC layer) and BC-2.06.016 (TUI layer acting on the signal from IPC).
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
| BC-2.05.006 | TUI Reconnects After Daemon Restart | P1 | F-27 |
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
  ├── depends on monocle-runtime (HookEventRecord — SS-01; required for InitialState.ring_tail)
  ├── depends on tokio (tokio::net::UnixStream, UnixListener, AsyncRead, AsyncWrite)
  ├── depends on serde + serde_json (message serialization)
  └── depends on uuid (prompt_id generation, serde feature)

monocle-tui (SS-06, consumer)
  └── depends on monocle-ipc (UdsTransport, ServerToClient, ClientToServer)

monocle-runtime (SS-04, daemon side)
  └── depends on monocle-ipc (UdsTransport server-side, message types)
```

`monocle-ipc` depends on `monocle-runtime` for `HookEventRecord` only (the `InitialState.ring_tail`
field type). This is a narrow, read-only dependency on the ring's storage record type.
The daemon and TUI both depend on `monocle-ipc` as consumers. The `Transport` trait and
message types remain decoupled from ring I/O logic — `HookEventRecord` is a pure data type
(no I/O, no async traits).

Note: This introduces a `monocle-ipc → monocle-runtime` edge. The ring is in `monocle-runtime`
because it was introduced with the daemon lifecycle (SS-01). If this dependency direction is
undesirable for Phase 4 (e.g., TUI-only binaries importing `monocle-ipc` would transitively
pull in `monocle-runtime`), `HookEventRecord` can be moved to `monocle-core` in a future
refactor. That move is not needed in Phase 1 since the TUI binary (`monocle-tui`) already
depends on `monocle-runtime` for other types.

---

## Module Purity Classification

| Module / Function | Classification | Rationale |
|-------------------|---------------|-----------|
| `ServerToClient` enum | Pure core | Data type only; `#[derive(Serialize, Deserialize)]`. No I/O. |
| `ClientToServer` enum | Pure core | Data type only. No I/O. |
| `PermissionDecision` enum | Pure core | Data type only. No I/O. |
| `TransportEvent` enum | Pure core | Process-local signal type only; NOT serialized. No I/O. |
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

### PermissionPromptQueued Delivered Twice Across Snapshot Window

Risk: A `PermissionPromptQueued` broadcast can arrive in a new client's mpsc channel between
`register_subscriber` and `snapshot_initial_state`. The snapshot includes the prompt in
`InitialState.overlay_stack`; the streaming send loop also delivers the queued
`PermissionPromptQueued` message. The TUI receives the same prompt twice.

Mitigation: The IPC layer intentionally provides **at-least-once delivery** for
`PermissionPromptQueued` across the connection snapshot window. The register-before-snapshot
ordering is preserved to guarantee no-gap delivery (BC-2.05.002 Invariant 3). The consumer
(TUI `VecDeque<PromptModal>`) is required to apply `PermissionPromptQueued` with
idempotent-on-`prompt_id` semantics: if the `prompt_id` is already present in the overlay
stack, the second delivery is silently discarded (BC-2.05.002 Invariant 4).

This design mirrors the existing no-op requirement for `PermissionPromptResolved`: if a
`prompt_id` is absent from the VecDeque, removal is a no-op. Insert and remove are both
`prompt_id`-idempotent by design. No protocol epoch fields, no daemon-side per-client dedup
logic, and no global snapshot lock are required. Decision rationale: F-S022-ADV6-MED-001,
`cycles/cycle-001/S-022/adversarial/architect-decisions-pass-6.md`.

### TUI Reconnects During a Pending Decision

Risk: TUI disconnects while the user was viewing a `PermissionPromptQueued` overlay. The user
reconnects and sees a stale prompt. They approve it after the Claude Code timeout has already
expired.

Mitigation: SOQ-3 (overlay clear on disconnect) removes all `VecDeque<PromptModal>` entries
on TUI-side disconnect. The `InitialState` push on reconnect contains only prompts that are
still pending in the daemon's registry (i.e., still within the 300ms timeout window). Stale
prompts are never re-pushed.

---

## §Trace v1.9.0

**architect-decisions-pass-2.md binding directive — §TUI IPC Read Loop Pattern added** (2026-05-28):
- Added §TUI IPC Read Loop Pattern section after §Phase 3: Disconnect in §Connection Lifecycle.
- Forbids the cancellation-unsafe `tokio::time::timeout(1ms, read_framed(...))` pattern
  (F-S025-ADV2-BLOCKER-001): wrapping a `read_framed` future in a 1ms timeout will silently
  consume bytes from the kernel socket buffer on cancellation, corrupting frame alignment.
- Mandates the canonical pattern: dedicated `spawn_ipc_reader` task with `move` ownership
  of `UdsClientTransport` + bounded `mpsc::channel(64)`. Consumer drains via `ipc_rx.try_recv()`
  at 16ms keyboard-tick cadence.
- Encodes all design invariants: capacity=64, drop-policy=BLOCK (`.await`, not `try_send`),
  exclusive transport ownership (no `Arc<Mutex>`), reconnect handoff sequence (abort →
  SOQ-3 clear → reconnect → respawn reader).
- Updated §Reconnection Behavior to forbid direct `read_framed` polling in `tokio::select!`
  / `tokio::time::timeout` arms and redirect to §TUI IPC Read Loop Pattern.
- Cross-references: BC-2.05.002 (Invariant 4 at-least-once), BC-2.05.007 (SOQ-3),
  BC-2.05.006 (reconnect backoff). Reference implementation:
  `crates/monocle-tui/src/app.rs::spawn_ipc_reader` (S-025). Proof of cancellation safety:
  integration test `test_bc_2_05_002_pc_6_no_frame_corruption_across_inter_message_gap`.

## §Trace v1.8.0

**F-S022-ADV6-MED-001 MED — at-least-once semantics documented; prompt_id idempotency invariant** (2026-05-28):
- Added §Risk Mitigations entry: "PermissionPromptQueued Delivered Twice Across Snapshot Window".
- Documents IPC layer at-least-once delivery for `PermissionPromptQueued` across the connection
  snapshot window. Mandates TUI-side `prompt_id` idempotency (BC-2.05.002 Invariant 4).
- Rationale: register-subscriber-before-snapshot ordering (BC-2.05.002 Invariant 3) is correct
  and preserved; duplicate delivery is resolved by consumer idempotency, not daemon-side dedup
  or global locking. See architect-decisions-pass-6.md for Option D rationale vs A/B/C.

## §Trace v1.7.0

**F-S022-ADV2-HIGH-002 HIGH — ring_tail type corrected to Vec<HookEventRecord>** (2026-05-27):
- **F-S022-ADV2-HIGH-002** [HIGH] ring_tail fidelity violation — `InitialState.ring_tail`
  was typed `Vec<HookEvent>` but the RAM ring stores `Vec<HookEventRecord>`. Converting
  from `HookEventRecord` to `HookEvent` requires reconstructing variant-specific fields
  (cwd, transcript_path, prompt, stop_reason, notification_type, message) that are absent
  from the record struct. S-022 Round 2 silently fabricated empty strings for missing fields
  with no WARN — a silent data fidelity loss.
  - Resolution: `ring_tail` type changed to `Vec<HookEventRecord>` in `ServerToClient::InitialState`.
  - Added inline doc comment explaining the type choice.
  - Updated §Connection Lifecycle §Phase 1 Connect step 4.
  - Updated §Dependency Graph: added `monocle-ipc → monocle-runtime` edge for `HookEventRecord`.
  - Added note in §Dependency Graph explaining the narrow dependency and potential Phase 4 refactor.
  - Rationale: ring stores `HookEventRecord` per BC-2.04.012 PC-1; TUI event ribbon (S-025)
    renders from `hook_type`, `session_id`, `timestamp_micros`, `tool_name` — sufficient from
    `HookEventRecord`. Lossless pass-through preferred over lossy reconstruction.
    Option A (extend HookEventRecord with large fields) rejected: 4096-entry RAM ring with
    256 KiB prompt/message = unbounded RAM ring, violating BC-2.04.012 PC-1 bounded contract.
  - See ADR-0006 and `cycles/cycle-001/S-022/adversarial/architect-decisions-pass-2.md`.

## §Trace v1.6.0

**F-P15-004 type definition** (2026-05-27):
- **F-P15-004** [CRITICAL] `TransportEvent` referenced but never defined — added formal
  `TransportEvent` enum definition to §Supporting Types. The type was referenced in
  §SOQ-3 (line "emits a `TransportEvent::Disconnected` signal"), §Reconnection Behavior
  step 1, the SOQ-3 summary table, and the Module Purity Classification table, and is
  cited by BC-2.05.007 and BC-2.06.016 — but had no code-level definition anywhere in
  the spec, leaving implementers without a canonical type shape.
  - Added `TransportEvent` enum with two variants: `Disconnected` (UDS connection lost:
    EOF, BrokenPipe, ConnectionReset — triggers SOQ-3 overlay clear on TUI side) and
    `Reconnected` (UDS connection re-established — TUI will receive fresh InitialState push).
  - Added clarifying note: `TransportEvent` is defined in `monocle-ipc`, consumed by
    `monocle-tui`, NOT serialized over the wire (process-local signal only), and is
    separate from `ServerToClient` (wire messages from daemon) and `ClientToServer`
    (wire messages to daemon).
  - Added rationale for intentional omission of `#[non_exhaustive]`: `TransportEvent`
    is a closed transport-state set; the TUI must handle it exhaustively. New transport
    states require explicit TUI-side handling by design.
  - Added `TransportEvent` row to Module Purity Classification table (pure core — process-local
    signal, not serialized, no I/O).

## §Trace v1.5.0

**Architectural clarification** (F-P1D9-001, F-P1D10-001) (2026-05-26):
- **F-P1D9-001 / F-P1D10-001** [CLARIFICATION] `ClientDisconnect` does not exist and is correct — the `ClientToServer` enum has no `ClientDisconnect` variant by design. tmux kills the TUI process on `Ctrl-\`; a killed process cannot send a graceful farewell. The architecture requires no goodbye message because the daemon's pending-prompt registry is client-independent: client connect/disconnect events do not affect it. Added §Client Disconnect and Overlay Persistence section documenting:
  - The daemon's pending-prompt registry is CLIENT-INDEPENDENT: entries are driven by hook arrivals and timeouts, not by TUI client lifecycle.
  - No `ClientDisconnect` message is needed or defined; tmux kills the process, eliminating any opportunity for graceful goodbye.
  - The show/hide cycle (normal operation): daemon registry unaffected on hide; TUI rebuilds from `InitialState.overlay_stack` on show.
  - SOQ-3 is a TUI-SIDE invariant only: the TUI clears its LOCAL overlay stack when it detects `TransportEvent::Disconnected` from the daemon. The daemon never clears anything in response to a client disconnect.
  - Added summary table mapping each scenario (TUI hide, TUI show, daemon crash, normal shutdown) to daemon registry and TUI stack outcomes.
- Clarified §Reconnection Behavior step 1 ("Overlay clear (SOQ-3)"): added "TUI LOCAL stack only" to the heading and expanded the rationale to make clear that this clears the TUI's in-memory `VecDeque<PromptModal>` — not the daemon's registry — and that the daemon rebuilds the TUI's overlay from the fresh `InitialState.overlay_stack` on successful reconnect.

## §Trace v1.4.0

**Adversarial Pass 4 review corrections** (F-P1D4-002) (2026-05-26):
- **F-P1D4-002** [CRITICAL] `PermissionDecision` serde wire format mismatch — removed the
  crate-level `#[serde(rename_all = "snake_case")]` attribute on `PermissionDecision` and
  replaced it with explicit per-variant `#[serde(rename = "...")]` attributes. The previous
  attribute would produce wire values `"accept"`, `"accept_always"`, and `"reject"` — but
  the Claude Code hook protocol wire format requires `"accept"`, `"always"`, and `"deny"`.
  `"accept_always"` and `"reject"` are not valid decision values in the hook protocol; a
  daemon sending those strings would cause Claude Code to reject or ignore the decision,
  leaving the hook hanging until timeout. The Trace v1.0.0 footer already documented the
  correct wire values (`{"decision":"always"}` and `{"decision":"deny"}`); this correction
  brings the enum annotation into agreement with that documented intent. No new fields or
  variants were added; this is a serialization annotation change only.

## §Trace v1.3.0

**Adversarial Pass 3 review corrections** (F-P1D3-001) (2026-05-26):
- **F-P1D3-001** [CRITICAL] `session_id` type mismatch — corrected `PermissionPromptPayload.session_id`
  from `Uuid` to `String`. The `session_id` field originates from Claude Code as a string
  identifier and is typed `String` consistently across all other architecture types:
  `EnrichedSession`, `HookEvent`, `HookEventReceived`, `HookEnvelope`, and `PromptModal`.
  Only `prompt_id` is `Uuid` (daemon-generated). A `Uuid` type here would require
  TUI-side parsing on every received payload and would diverge from the string representation
  stored in the JSONL ring and passed through all hook envelope types. Added an inline comment
  on the field documenting the type rationale and naming the related fields for cross-reference.

## §Trace v1.2.0

**Adversarial Pass 2 review corrections** (F-P1D2-004, F-P1D2-008) (2026-05-26):
- **F-P1D2-004** Priority drift — BC-2.05.006 corrected P0 → P1 to match BC-INDEX
  (source of truth). BC-INDEX §SS-05 row 6: P1.
- **F-P1D2-008** Added explicit `PermissionPromptPayload` struct definition in
  §Supporting Types. Previously the type was described in prose as "the same struct as
  the fields inlined in `PermissionPromptQueued`" without a code-level definition,
  leaving the struct shape ambiguous for implementers.
  - Added `pub struct PermissionPromptPayload` with fields: `prompt_id: Uuid`,
    `session_id: Uuid`, `tool_name: String`, `tool_input: serde_json::Value`,
    `old_content: Option<String>`, `new_content: Option<String>`. Carries
    `#[non_exhaustive]` per BC-2.02.003 extensibility policy.
  - Updated `PermissionPromptQueued` variant to embed `payload: PermissionPromptPayload`
    instead of inlining the same fields. This makes the type relationship explicit and
    ensures TUI rendering code is unified across both the initial-state-push path and
    the live-queued-prompt path.
  - Updated `InitialState.overlay_stack` description to confirm `Vec<PermissionPromptPayload>`.

## §Trace v1.1.0

**Adversarial review corrections** (F-P1D-007, F-P1D-011) (2026-05-26):
- **F-P1D-007** Timeout stale prompt resolution — architectural decision recorded:
  - `PermissionPromptResolved` IS sent to all connected TUI clients on hook timeout
    (in addition to user-decision resolution). This reverses a prior implicit "not sent"
    reading of the spec; the message type's doc comment now explicitly lists both
    triggering cases.
  - §Phase 2: Streaming Updates updated: the `PermissionPromptResolved` bullet now
    documents both cases — (a) TUI client decision and (b) daemon hook timeout expiry.
  - TUI handling is identical in both cases: pop matching PromptModal, no-op if absent.
  - Rationale: reusing the existing `PermissionPromptResolved` message type is the
    cleanest resolution — it requires no new message variant, no TUI-side divergence,
    and ensures all TUIs consistently remove stale overlays on timeout.
- **F-P1D-011** Ping/Pong keepalive decision recorded:
  - Ping and Pong variants are reserved for Phase 2 keepalive detection.
  - No BC is required for Ping/Pong in Phase 1.
  - Both variants carry doc comments and a block comment after `ClientToServer` enum
    specifying the Phase 1 discard behavior: receive silently without error or close.
  - The `#[non_exhaustive]` attribute (already present per BC-2.02.003) ensures Phase 1
    code that matches on `ServerToClient` or `ClientToServer` does not need modification
    when Phase 2 activates these variants.

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
