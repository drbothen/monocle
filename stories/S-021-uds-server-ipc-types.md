---
document_type: story
level: L4
story_id: S-021
epic_id: EPIC-05
version: "1.1"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 8
wave: 5
tdd_mode: strict
priority: P0
depends_on: [S-017, S-013, S-014]
blocks: [S-022]
target_module: monocle-ipc
subsystems: [SS-05]
behavioral_contracts: [BC-2.05.001, BC-2.05.003, BC-2.05.004, BC-2.05.008]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.001.md, version: "1.2.0"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.003.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.004.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.008.md, version: "1.0.3"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.05.001 (UDS socket bind at runtimeDir/monocle.sock), BC-2.05.003 (SessionListUpdate fan-out), BC-2.05.004 (HookEventReceived with 256-byte excerpt), BC-2.05.008 (UDS-only Phase 1 constraint + Transport trait)"
---

# S-021: UDS Server Bind, IPC Message Types, and Transport Trait

## Narrative

As the monocle daemon, I want to bind a Unix domain socket at `<runtime_dir>/monocle.sock`,
broadcast `SessionListUpdate` and `HookEventReceived` messages to all connected TUI clients,
and enforce the UDS-only Phase 1 constraint via `#![forbid(unsafe_code)]` and `cargo deny`,
so that TUI clients receive live session and event data over a secure, owner-only transport.

## Acceptance Criteria

### AC-001 (traces to BC-2.05.001 postcondition PC-1 — UDS bind)
The daemon calls `UnixListener::bind("<runtime_dir>/monocle.sock")` at step 10 of the start
sequence (after steps 1-9 per BC-2.04.001). The socket is constructed as
`Path::new(runtime_dir).join("monocle.sock")` — never via shell string concatenation.

### AC-002 (traces to BC-2.05.001 postcondition PC-2 — mode 0o600)
The socket file at `<runtime_dir>/monocle.sock` has mode `0o600` (owner-readable and
owner-writable only; no group or world access). The mode is set immediately after bind via
`std::fs::set_permissions`.

### AC-003 (traces to BC-2.05.001 postcondition PC-3 — stale socket removal)
If a file already exists at `<runtime_dir>/monocle.sock` at bind time (stale socket from a
prior crashed daemon), the daemon removes it before calling `UnixListener::bind`. On removal,
the daemon logs `WARN: removed stale UDS socket at <path>`. This mirrors the stale lock-file
removal at step 2 of the daemon start sequence.

### AC-004 (traces to BC-2.05.001 postcondition PC-4 — graceful shutdown removal)
On graceful shutdown, the daemon removes `<runtime_dir>/monocle.sock`. The removal happens
in the same cleanup sequence as `monocle.lock` removal (per BC-2.04.001 shutdown procedure).

### AC-005 (traces to BC-2.05.001 postcondition PC-5 — bind failure exit)
If `UnixListener::bind` fails after the stale-file removal (permissions error, path too long,
filesystem full), the daemon logs `ERROR: failed to bind UDS socket at <path>: <reason>` and
exits with code 1. No TUI connections are accepted.

### AC-006 (traces to BC-2.05.003 postcondition PC-1 — SessionListUpdate broadcast)
When the session roster changes (session added, removed, or enriched), the daemon serializes
and sends `ServerToClient::SessionListUpdate { sessions: Vec<EnrichedSession> }` to every
currently connected TUI client via the fan-out subscriber list. The fan-out task skips
disconnected clients (closed send half) without panic.

### AC-007 (traces to BC-2.05.003 postcondition PC-2 — full list semantics)
The `sessions` field in every `SessionListUpdate` contains the complete current session list
at the time of emission. It is not a diff. A TUI client that receives this message replaces
its entire local session roster with the contents of this field.

### AC-008 (traces to BC-2.05.003 postcondition PC-3 — SessionListUpdate 256 KiB limit)
If the serialized `SessionListUpdate` message exceeds 256 KiB, the daemon logs
`ERROR: SessionListUpdate exceeds 256 KiB; cannot broadcast` and does NOT send the message
to any client. The session list remains in daemon internal state.

### AC-009 (traces to BC-2.05.004 postcondition PC-1 — HookEventReceived fields)
When the daemon ingests a hook event (any of the 5 hook endpoints), it broadcasts
`ServerToClient::HookEventReceived` to all connected TUI clients. The message fields are:
- `hook_type: HookType` — the discriminant of the hook that was received.
- `session_id: String` — the session identifier from the hook POST body.
- `payload_excerpt: String` — the first 256 bytes of the hook POST body JSON, truncated at
  a valid UTF-8 character boundary (not a byte boundary). No data after 256 bytes appears.
- `latency_ms: u64` — wall-clock ms from HTTP POST receipt to HTTP ACK sent to the hook caller.

### AC-010 (traces to BC-2.05.004 postcondition PC-4 — drop counter not incremented by IPC)
The `drop_counter` is NOT incremented by sending a `HookEventReceived` IPC message to TUI
clients. The drop counter only increments when the bounded event bus drops an event before
it reaches the fan-out task (per BC-2.04.011). Once an event reaches the fan-out task, it is
always delivered to connected clients; slow-client send-buffer-full results in per-client
disconnect, not drop counter increment.

### AC-011 (traces to BC-2.05.004 postcondition PC-5 — HookType non_exhaustive)
The `HookType` enum carries `#[non_exhaustive]`. TUI clients that receive an unknown
`HookType` variant (future hook type) must render a safe fallback label (e.g., "unknown")
rather than panicking. The `#[non_exhaustive]` attribute enforces a catch-all arm at compile time.

### AC-012 (traces to BC-2.05.008 postcondition PC-1 — #![forbid(unsafe_code)])
The `monocle-ipc` crate root carries `#![forbid(unsafe_code)]`. Any attempt to add
inline `unsafe { ... }` blocks is rejected by the Rust compiler with a hard error.

### AC-013 (traces to BC-2.05.008 postcondition PC-2/PC-3 — cargo deny shared-memory prohibition)
The `Cargo.toml` for `monocle-ipc` does NOT list any of the following crates as dependencies:
`shared_memory`, `raw-sync`, `ipc-channel`. The `cargo deny` configuration in the workspace
includes deny rules for these crates. CI fails if any of them appear in the dependency graph.

### AC-014 (traces to BC-2.05.008 postcondition PC-4 — semgrep CI check)
The semgrep CI check scans `monocle-ipc/src/**/*.rs` for the following patterns and fails
if any match: `libc::mmap`, `nix::sys::mman`, `shm_open`, `mmap_rs`, `memmap2`.

### AC-015 (traces to BC-2.05.008 postcondition PC-5/PC-6 — Transport trait)
The `Transport` trait is defined in `monocle-ipc` with exactly two async methods:
```rust
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send_message(&mut self, msg: &ServerToClient) -> Result<(), IpcError>;
    async fn recv_message(&mut self) -> Result<ClientToServer, IpcError>;
}
```
`UdsTransport` is the sole `Transport` implementor in `monocle-ipc` during Phase 1. The
`Transport` trait's method signatures are stable for Phase 4 `ShmTransport` addition.

### AC-016 (traces to BC-2.05.001 edge case EC-002 — UDS path length limit)
If the computed UDS socket path exceeds the OS path length limit (104-108 bytes on POSIX
platforms), the daemon logs `ERROR: UDS socket path exceeds OS limit (<N> bytes, limit <M>)`
and exits 1 without attempting bind.

### AC-017 (traces to BC-2.05.004 edge case EC-005 — slow client disconnect)
When a TUI client's send buffer is full (slow TUI client), the daemon detects the send error,
removes the slow client from the fan-out subscriber list, closes the per-client connection,
and logs `WARN: removed slow TUI client (send buffer full)`. Other clients are unaffected.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,600 |
| BC-2.05.001.md | ~900 |
| BC-2.05.003.md | ~800 |
| BC-2.05.004.md | ~900 |
| BC-2.05.008.md | ~900 |
| SS-ipc.md §Transport Layer + §Message Types | ~4,000 |
| S-017 (daemon start sequence context — step 10 UDS bind) | ~400 |
| S-013 (HookEnvelope proto — HookType enum) | ~400 |
| Test files | ~800 |
| **Total estimate** | **~10,700** |

## Tasks

- [ ] Create `monocle-ipc/` crate with `Cargo.toml` listing `interprocess 2.4`, `serde_json =1.0.149`, `async-trait 0.1`, `uuid` (v4+serde features), `tokio =1.52.0`, `tracing 0.1`
- [ ] Add `#![forbid(unsafe_code)]` to `monocle-ipc/src/lib.rs` as the first crate-level attribute
- [ ] Define `Transport` trait in `monocle-ipc/src/transport.rs` with `send_message` and `recv_message` methods (async_trait)
- [ ] Define `ServerToClient` enum: `InitialState`, `SessionListUpdate`, `HookEventReceived`, `PermissionPromptQueued`, `PermissionPromptResolved`, `DropCounterUpdate`
- [ ] Define `ClientToServer` enum: `PermissionDecision`
- [ ] Define `PermissionPromptPayload` wrapper struct with fields: `prompt_id: Uuid`, `session_id: String`, `tool_name: String`, `tool_input: serde_json::Value`, `old_content: Option<String>`, `new_content: Option<String>`
- [ ] Implement `HookType` enum with `#[non_exhaustive]` (variants: PreToolUse, Notification, Stop, SessionStart, UserPromptSubmit)
- [ ] Implement `IpcError` enum: `MessageTooLarge`, `SerializeError(serde_json::Error)`, `IoError(std::io::Error)`, `Disconnected`
- [ ] Implement `UdsTransport` in `monocle-ipc/src/uds.rs`:
  - UDS bind at `<runtime_dir>/monocle.sock`: stale-file removal (WARN), mode 0o600, bind-failure exit 1
  - Path length validation: if path > OS limit, log error and exit 1
  - Per-client Tokio task spawner
  - Fan-out subscriber list (bounded `Vec<Sender<ServerToClient>>`)
  - `send_message` / `recv_message` implementing the `Transport` trait
- [ ] Implement 4-byte LE length-prefix framing in `monocle-ipc/src/framing.rs`:
  - `write_framed`: serialize to JSON, write 4-byte LE u32 length, write payload
  - `read_framed`: read 4-byte LE length, validate <= 256 KiB, read payload bytes, deserialize
  - `IpcError::MessageTooLarge` when payload > 262,144 bytes
- [ ] Implement fan-out broadcast task:
  - `broadcast_session_list_update(sessions: Vec<EnrichedSession>)`: 256 KiB guard; skip disconnected clients
  - `broadcast_hook_event_received(hook_type, session_id, body_bytes, latency_ms)`:
    - `payload_excerpt`: first 256 bytes of body, truncated at valid UTF-8 char boundary
    - Slow client (send error): remove from subscriber list; log WARN; disconnect client
    - Drop counter NOT incremented on IPC send failure
- [ ] Integrate UDS bind into `daemon_start_sequence()` at step 10 (modify `monocle-runtime/src/lifecycle.rs`)
- [ ] Wire graceful shutdown: remove `monocle.sock` alongside `monocle.lock` cleanup
- [ ] Add `cargo deny` deny-list entries for `shared_memory`, `raw-sync`, `ipc-channel` to workspace `deny.toml`
- [ ] Add semgrep CI rule scanning `monocle-ipc/src/**/*.rs` for `libc::mmap`, `nix::sys::mman`, `shm_open`, `mmap_rs`, `memmap2`
- [ ] Unit tests `monocle-ipc/tests/uds_bind_lifecycle.rs`:
  - Happy path: socket created with mode 0o600
  - Stale socket: WARN logged, removed, rebound with mode 0o600
  - Graceful shutdown: socket file removed
  - Bind failure: error logged, exit 1
  - Path length > OS limit: error logged, exit 1
- [ ] Unit tests `monocle-ipc/tests/session_list_update.rs`:
  - SessionListUpdate broadcast to all connected clients; full list (not diff)
  - Disconnected client skipped without error
  - Empty sessions Vec broadcast correctly
  - > 256 KiB serialization: error logged, no broadcast
- [ ] Unit tests `monocle-ipc/tests/hook_event_received.rs`:
  - payload_excerpt: 50-byte body → full excerpt; 512-byte body → 256-byte excerpt (valid UTF-8 boundary)
  - latency_ms value propagated correctly
  - drop_counter NOT incremented on IPC slow-client disconnect
  - Slow client: removed from subscriber, WARN logged, other clients unaffected
- [ ] Unit test `monocle-ipc/tests/framing.rs`:
  - 4-byte LE length prefix encodes/decodes payload length correctly
  - MessageTooLarge on payload > 262,144 bytes
- [ ] Static analysis test `monocle-ipc/tests/transport_trait_stability.rs`:
  - Verify UdsTransport implements Transport trait (compile-time check)

## Previous Story Intelligence

S-017: `daemon_start_sequence()` is implemented. Step 10 of the 13-step sequence is the UDS
bind. This story integrates with step 10 by calling `UdsTransport::bind(runtime_dir)` in the
start sequence after step 9 (hooks-settings.json write). The daemon's `DaemonState` struct
will gain a `uds: Arc<UdsTransport>` field.

S-013: `HookEnvelope` proto wire format and `HookType` mapping are established. The `HookType`
enum in this story must be consistent with the hook type discriminants from S-013. The
`#[non_exhaustive]` attribute required by BC-2.02.003 (from S-011) must be applied to `HookType`.

S-014: `EngineModule` trait established. Session detection produces `EnrichedSession` instances
that feed into `SessionListUpdate` broadcasts. The `EnrichedSession` struct must carry
`#[non_exhaustive]` (BC-2.02.003).

## Architecture Compliance Rules

From `architecture/SS-ipc.md v1.4.0 §Transport Layer §Lifecycle` (at S-021 authoring time):
- Socket path: `Path::new(runtime_dir).join("monocle.sock")` — NOT shell string interpolation
- Stale socket removal before bind; WARN log required
- Mode 0o600 set immediately after bind via `std::fs::set_permissions`
- Socket removed in shutdown sequence alongside lock file

From `architecture/SS-ipc.md v1.4.0 §Framing Protocol` (at S-021 authoring time):
- 4-byte little-endian `u32` length prefix + UTF-8 JSON payload
- MAX_MESSAGE_BYTES = 262,144 (256 KiB)
- No trailing newline or null terminator

From `architecture/SS-ipc.md v1.4.0 §Phase 1 Transport Constraint` (at S-021 authoring time):
- `#![forbid(unsafe_code)]` is the first crate attribute in `monocle-ipc/src/lib.rs`
- `cargo deny` deny-list: `shared_memory`, `raw-sync`, `ipc-channel`
- semgrep patterns: `libc::mmap`, `nix::sys::mman`, `shm_open`, `mmap_rs`, `memmap2`
- `Transport` trait: exactly `send_message` and `recv_message` — no additional methods

**Forbidden Dependencies:**
- `monocle-ipc` MUST NOT depend on `shared_memory`, `raw-sync`, `ipc-channel` (any version)
- `monocle-ipc` MUST NOT contain `memmap2` or `mmap_rs` (allowed only in future `monocle-ipc-shm`)
- `monocle-ipc` MUST NOT call `libc::kill` or any POSIX signal API (that is `monocle`'s domain)
- If `monocle-ipc` gains a dependency on any shared-memory crate, the `cargo deny` CI job MUST fail

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| interprocess | 2.4 | `UnixListener` / `UnixStream` for UDS bind and per-client connections |
| tokio | =1.52.0 | Per-client Tokio task spawner; async I/O; bounded channel for fan-out |
| serde_json | =1.0.149 | JSON serialization/deserialization for framed IPC messages |
| serde | 1 (features=["derive"]) | `#[derive(Serialize, Deserialize)]` on ServerToClient/ClientToServer |
| async-trait | 0.1 | `#[async_trait]` for the `Transport` trait |
| uuid | (features=["v4","serde"]) | `prompt_id: Uuid` in `PermissionPromptPayload` |
| tracing | 0.1 | WARN on stale socket removal, slow client disconnect; ERROR on bind failure |

## File Structure Requirements

Files to create:
- `monocle-ipc/src/lib.rs` — crate root with `#![forbid(unsafe_code)]`; re-exports
- `monocle-ipc/src/transport.rs` — `Transport` trait definition (async_trait)
- `monocle-ipc/src/framing.rs` — `write_framed` / `read_framed` (4-byte LE length prefix, 256 KiB limit)
- `monocle-ipc/src/types.rs` — `ServerToClient`, `ClientToServer`, `PermissionPromptPayload`, `HookType`, `IpcError`, `EnrichedSession` re-export
- `monocle-ipc/src/uds.rs` — `UdsTransport` implementing `Transport`; fan-out subscriber list; broadcast methods
- `monocle-ipc/Cargo.toml` — crate manifest listing interprocess, tokio, serde_json, async-trait, uuid, tracing
- `monocle-ipc/tests/uds_bind_lifecycle.rs` — UDS bind/stale/shutdown/failure tests
- `monocle-ipc/tests/session_list_update.rs` — SessionListUpdate broadcast tests
- `monocle-ipc/tests/hook_event_received.rs` — HookEventReceived excerpt and slow-client tests
- `monocle-ipc/tests/framing.rs` — 4-byte LE framing correctness tests
- `monocle-ipc/tests/transport_trait_stability.rs` — Transport trait compile-time assertion

Files to modify:
- `monocle-runtime/src/lifecycle.rs` — integrate `UdsTransport::bind(runtime_dir)` at step 10 of `daemon_start_sequence()`; add UDS socket removal to shutdown cleanup
- `monocle-runtime/src/state.rs` — add `uds: Arc<UdsTransport>` to `DaemonState`
- `Cargo.toml` (workspace root) — add `monocle-ipc` as workspace member
- `deny.toml` (workspace root) — add deny rules for `shared_memory`, `raw-sync`, `ipc-channel`

## Downstream Consumer Contract

Public API produced by this story for downstream consumption:

```rust
// monocle-ipc

#[async_trait]
pub trait Transport: Send + Sync + 'static {
    async fn send_message(&mut self, msg: &ServerToClient) -> Result<(), IpcError>;
    async fn recv_message(&mut self) -> Result<ClientToServer, IpcError>;
}

pub struct UdsTransport { /* opaque */ }
impl UdsTransport {
    pub async fn bind(runtime_dir: &Path) -> Result<Self, IpcError>;
}
impl Transport for UdsTransport { ... }

pub enum ServerToClient {
    InitialState { sessions: Vec<EnrichedSession>, ring_tail: Vec<HookEvent>, overlay_stack: Vec<PermissionPromptPayload>, drop_counter: u64 },
    SessionListUpdate { sessions: Vec<EnrichedSession> },
    HookEventReceived { hook_type: HookType, session_id: String, payload_excerpt: String, latency_ms: u64 },
    PermissionPromptQueued { payload: PermissionPromptPayload },
    PermissionPromptResolved { prompt_id: Uuid },
    DropCounterUpdate { drop_counter: u64 },
}

pub enum ClientToServer {
    PermissionDecision { prompt_id: Uuid, decision: PermissionDecision },
}

pub struct PermissionPromptPayload {
    pub prompt_id: Uuid,
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}

#[non_exhaustive]
pub enum HookType {
    PreToolUse,
    Notification,
    Stop,
    SessionStart,
    UserPromptSubmit,
}

pub enum IpcError {
    MessageTooLarge,
    SerializeError(serde_json::Error),
    IoError(std::io::Error),
    Disconnected,
}
```

S-022 (TUI connect + permission prompt) uses `UdsTransport::connect(runtime_dir)` and the
full `ServerToClient` / `ClientToServer` type set from this story.

## §Trace

**v1.1** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at story authoring time. No normative content changed.
