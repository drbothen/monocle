---
document_type: architecture-section
level: L3
section: "ipc"
subsystem: SS-05
version: "1.21.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T00:00:00Z
inputs:
  - {path: .factory/specs/prd-expansion-scope.md, version: "1.0"}
  - {path: .factory/specs/architecture/SS-daemon-wiring.md, version: "1.0.0"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.13"}
  - {path: .factory/specs/product-brief.md, version: "1.4.30"}
  - {path: .factory/specs/research/domain-monocle-vision-synthesis.md, version: "1.1.3"}
input-hash: "50aa63d"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: IPC

## Scope

SS-05 defines the `monocle-ipc` crate: the internal transport between TUI clients and the
daemon. The daemon is the UDS server; TUI instances are clients. The IPC layer delivers session
state, hook events, and permission prompts from the daemon to each TUI client, and routes
permission decisions from TUI clients back to the daemon.

**v1A client multiplicity:** v1A operates a single TUI client per daemon at any given time
(monocle is a tmux popup — one popup per user session). The transport infrastructure is
forward-compatible with concurrent multi-TUI-client viewing: the broker fan-out design (§5
of SS-daemon-wiring-v2-delta.md) supports multiple subscribers, as illustrated by the
vision §Process Topology diagram showing three clients (A, B, C) fanning out from the daemon's
broker. Concurrent multi-TUI-client viewing of the SAME session is a FUTURE capability,
deferred to `TD-MULTI-CLIENT-ATTACH-STORM-001` (see SS-daemon-wiring-v2-delta.md §5b scope
note and BC-2.05.009 Inv-2). The present-tense three-client illustration is forward-compatible
infrastructure, not a v1A concurrency guarantee.

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
1. Message types reference `SessionSnapshot`, `HookEventRecord`, and `HookEvent` structs that
   are already serialized via `serde` for the JSONL ring and IPC wire boundary. Re-using
   `serde_json` adds zero new dependencies.
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
    ///
    /// C3-004 (v1.12.0): `sessions` type changed from `Vec<EnrichedSession>` to
    /// `Vec<SessionSnapshot>`. `SessionSnapshot` is the canonical wire boundary type for
    /// all sessions (monocle-spawned and externally-detected). `EnrichedSession` is retained
    /// internally for `EngineModule::detect()` but is NOT exposed on the wire. See
    /// SS-daemon-wiring-v2-delta.md v1.9.1 §4 for the three-representation reconciliation. <!-- version-pin-historical: v1.9.1 at C3-004 merge time -->
    InitialState {
        /// All sessions (monocle-spawned and externally-detected) as `SessionSnapshot`.
        sessions: Vec<SessionSnapshot>,
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
    ///
    /// C3-004 (v1.12.0): `sessions` type changed from `Vec<EnrichedSession>` to
    /// `Vec<SessionSnapshot>` (same rationale as InitialState above).
    SessionListUpdate {
        sessions: Vec<SessionSnapshot>,
    },

    /// A hook event was ingested by the daemon.
    ///
    /// # timestamp_micros field (S-028 ADR — breaking change from v1.9.0)
    ///
    /// `timestamp_micros: i64` added in v1.10.0. This is a BREAKING change to the
    /// `ServerToClient` wire format. Consumers of the `HookEventReceived` variant
    /// that pattern-match with named fields MUST add `timestamp_micros` to their
    /// match arm. Consumers using `..` wildcard patterns are unaffected at compile
    /// time but will silently discard the field — they MUST be audited.
    ///
    /// **Breaking-change consumer list (all must be updated):**
    /// 1. `monocle-ipc/src/types.rs` — variant definition (DONE v1.10.0)
    /// 2. `monocle-runtime/tests/ipc_broadcast.rs` — struct literal construction (DONE v1.10.0)
    /// 3. `monocle-ipc/tests/message_types.rs` — serde roundtrip test (DONE v1.10.0)
    /// 4. `monocle-tui/src/app.rs` — `ServerToClient::HookEventReceived { .. }` match arm
    ///    (S-028 implementer: extract `timestamp_micros` here; pass to `HookEventRow`)
    /// 5. Any future S-028+ code that constructs or matches `HookEventReceived`
    HookEventReceived {
        hook_type: HookType,
        session_id: String,
        /// First 256 bytes of the payload (to keep message size bounded).
        payload_excerpt: String,
        /// Time from hook POST receipt to daemon ACK, in milliseconds.
        latency_ms: u64,
        /// Daemon-owned event timestamp: Unix epoch microseconds (i64, signed).
        ///
        /// Set by the hook handler from the same clock source used to populate
        /// `HookEventRecord::timestamp_micros` in the JSONL ring buffer.
        /// The TUI event ribbon MUST render this as the wall-clock event time
        /// (BC-2.05.004 PC-1). MUST NOT be replaced by `SystemTime::now()` at
        /// TUI receipt time — that would show IPC transit time, not event time.
        timestamp_micros: i64,
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
<a id="servertoclient-permissionpromptresolved"></a>
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

    // ── v1A control-center additions (C5-001, ADR-0010 §IPC Message Type Additions) ─────

    /// Raw PTY bytes from a session's harness child. The TUI feeds these into
    /// the per-session vt100::Parser.
    ///
    /// Sent by the daemon broker when the session-host proxy posts
    /// `HostToDaemon::PtyBytes`. The TUI MUST buffer `PtyOutput` for a session while
    /// a scrollback dump is in progress (`dump_in_progress = true`) and replay after
    /// `ScrollbackDumpComplete` (ADR-0010 §Interleaving of live PtyOutput).
    PtyOutput {
        /// Matches `session_id: String` throughout the codebase (UUID rendered as String).
        session_id: String,
        /// Raw PTY output bytes; NOT pre-decoded. Fed into `vt100::Parser::process()`.
        bytes: Vec<u8>,
    },

    /// A session's lifecycle state changed (e.g., Launching → Running,
    /// Running → Terminated). Always emitted BEFORE `SessionListUpdate` for the same
    /// transition (BC-2.08.008 Invariant 4 ordering guarantee;
    /// SS-daemon-wiring-v2-delta §3b emission rule).
    SessionStateChanged {
        session_id: String,
        new_state: SessionState,
    },

    /// Scrollback dump — one chunk of a multi-message scrollback transfer.
    ///
    /// When the daemon first attaches to (or re-discovers) a session-host, it sends
    /// `DaemonToHost::Attach`. The session-host atomically snapshots the current
    /// `vt100::Screen` as styled cells, then streams it as `ScrollbackChunk*` messages
    /// terminated by `ScrollbackDumpComplete`. The session-host resumes live
    /// `HostToDaemon::PtyBytes` immediately after the snapshot (I3-003 fix —
    /// snapshot-then-resume protocol; ADR-0010 §Interleaving).
    ///
    /// Framing invariant: each `ScrollbackChunk` MUST fit within the 256 KiB
    /// per-message limit (`MAX_MESSAGE_BYTES`). The session-host chunks rows to
    /// respect this limit.
    ///
    /// TUI MUST buffer incoming `PtyOutput` for this session while
    /// `dump_in_progress = true` and replay after receiving `ScrollbackDumpComplete`.
    ScrollbackChunk {
        session_id: String,
        /// Row-major styled-cell data. Rows in this chunk, ordered oldest-to-newest
        /// (continuing from the previous chunk). Cell type defined in §Supporting Types.
        rows: Vec<Vec<SerializedCell>>,
        /// Chunk sequence number (0-indexed). Non-contiguous sequence → log WARN
        /// and re-attach to restart the dump.
        chunk_seq: u32,
    },

    /// Sentinel terminating a scrollback dump sequence.
    ///
    /// On receipt, the TUI MUST:
    ///   1. Validate `total_chunks` matches chunks received; mismatch → WARN + re-attach.
    ///   2. Reset `pty_parsers[session_id]` (fresh `vt100::Parser::new(pty_rows, pty_cols, SCROLLBACK_ROWS)`).
    ///   3. Reconstruct the screen via the scrollback-as-bytes path (SS-session-manager §Screen-state transfer).
    ///   4. Replay any `PtyOutput` bytes buffered during the dump (I3-003 fix).
    ///   5. Set `dump_in_progress = false`; process subsequent `PtyOutput` normally.
    ScrollbackDumpComplete {
        session_id: String,
        /// Total number of chunks sent (for integrity validation).
        total_chunks: u32,
        /// Cursor position at the time the dump was taken.
        cursor_row: u16,
        cursor_col: u16,
        /// PTY dimensions at the time of the dump.
        pty_rows: u16,
        pty_cols: u16,
    },

    /// PTY byte-sequence integrity reset. Sent when the session-host posts
    /// `HostToDaemon::PtyReset` (PTY reader channel drop detected — mid-CSI-sequence
    /// corruption risk). The TUI MUST reset `pty_parsers[session_id]` and re-attach
    /// by sending `ClientToServer::AttachSession { session_id }`, triggering a new
    /// `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence from the session-host.
    PtyReset {
        session_id: String,
    },

    /// A lifecycle operation requested by the TUI failed. Sent by the daemon to the
    /// requesting client ONLY (not broadcast to all clients). The TUI MUST surface an
    /// error banner to the user (e.g., in the sessions panel or status bar) so the
    /// failure is never silent.
    ///
    /// # When emitted
    ///
    /// Emitted on failure of any `ClientToServer` lifecycle operation:
    /// - `SpawnSession` → `EngineError::BinaryNotFound` (harness binary not on PATH; from `spawn_recipe()`)
    /// - `SpawnSession` → `EngineError::InvalidPath` (non-UTF-8 or null-byte path arg; from `spawn_recipe()`)
    /// - `SpawnSession` → `SessionError::SpawnFailed`
    /// - `SpawnSession` → `SessionError::SidecarWriteFailed` (post-spawn sidecar write failure)
    /// - `SpawnSession` → `SessionError::SessionIdCollision` (UUID v4 collision in registry)
    /// - `KillSession` → `SessionError::SessionNotFound`, `SessionError::SessionHostDead`
    /// - `AttachSession` → `SessionError::SessionHostDead`, `SessionError::SessionNotFound`
    /// - `KeyInput` → `SessionError::SessionNotFound` (session does not exist or is Terminated)
    /// - `RenameSession` → `SessionError::InvalidSessionName`
    ///
    /// # spawn_recipe() call site (Model A — daemon-side; I27-001 resolution)
    ///
    /// `ClaudeCodeModule::spawn_recipe()` is called INSIDE `SessionManager::spawn_session()`,
    /// DAEMON-SIDE, BEFORE `SessionHostSpawner::spawn()`. Specifically, the first step of
    /// `spawn_session()` is to call `engine_module.spawn_recipe(&opts)` to obtain a
    /// `SpawnRecipe`; if that call returns `Err(EngineError::...)`, `spawn_session()` translates
    /// the `EngineError` into a `SessionError` (via `From<EngineError>` on `SessionError` —
    /// see SS-session-manager.md §SessionError taxonomy) and returns it to the IPC handler.
    /// The IPC `SpawnSession` arm then maps it to the appropriate `ServerToClient::Error` code
    /// via `session_error_to_code()`.
    ///
    /// The TUI sends `ClientToServer::SpawnSession { opts: SpawnOptions }` carrying user-intent
    /// spawn parameters (project root, worktree root, harness ID, profile ID, CCR URL). The
    /// daemon IPC handler generates the session UUID (`uuid::Uuid::new_v4().to_string()`),
    /// fills `SpawnOptions.session_id` and `SpawnOptions.hooks_settings_path` (shared
    /// hooks-settings.json) via `opts.with_daemon_fields(session_id, hooks_path)`, sends
    /// `ServerToClient::SpawnAck { session_id }` to the requesting client (F-P41-IMP-001
    /// resolution — deterministic correlation), then passes the completed `SpawnOptions` to
    /// `SessionManager::spawn_session(opts)`. UUID generation and SpawnAck dispatch happen
    /// in the IPC handler BEFORE `spawn_session()` is called — `spawn_session()` receives
    /// `opts.session_id` already populated. The `SpawnRecipe` is built daemon-side inside
    /// `spawn_session()` via `spawn_recipe()` and is never transmitted over IPC. This is the
    /// ONLY path on which `EngineError::BinaryNotFound` and `EngineError::InvalidPath` can
    /// reach `ServerToClient::Error` as distinct codes (BC-2.03.007 PC-3/PC-7 diagnostic
    /// guarantee). The TUI is a thin client — it sends intent (SpawnOptions), not a
    /// pre-built harness command line.
    ///
    /// # No-silent-failure invariant (BC-2.05.010 PC-4 / PC-3-4 obligation)
    ///
    /// The IPC handler MUST NOT silently swallow `Err(SessionError::...)` from any lifecycle
    /// operation by returning `Ok(())` to the task boundary. Every `Err` from
    /// `SessionManager` MUST produce a `ServerToClient::Error` sent to the requesting
    /// client over its per-client channel. This includes EngineError-derived errors that are
    /// bridged through `SessionError::EngineError` (see §spawn_recipe() call site above).
    ///
    /// # v1A error code taxonomy (closed set for Phase 1 — 10 codes)
    ///
    /// The `code` field carries one of the following string literals (snake_case):
    ///
    /// | code                       | Trigger                                                              | TUI fixed banner text |
    /// |----------------------------|----------------------------------------------------------------------|-----------------------|
    /// | `"binary_not_found"`       | `EngineError::BinaryNotFound` — harness binary not found on PATH (e.g., `claude` not installed) | "claude binary not found — is Claude Code installed and on PATH?" |
    /// | `"invalid_spawn_arg"`      | `EngineError::InvalidPath` — structurally invalid argument to `spawn_recipe()` (e.g., non-UTF-8 hooks settings path) | "Session spawn failed: invalid hooks settings path (non-UTF-8)" |
    /// | `"spawn_failed"`           | OS process spawn failure from `SessionHostSpawner::spawn()`          | "Session spawn failed" |
    /// | `"sidecar_write_failed"`   | Sidecar write failed after OS process spawned (`SessionError::SidecarWriteFailed`) — orphan-kill protocol ran before error surfaces | "Session spawn failed: sidecar write error" |
    /// | `"session_id_collision"`   | UUID v4 collision in registry (`SessionError::SessionIdCollision`) — do not auto-retry | "Session spawn failed: internal ID collision" |
    /// | `"session_not_found"`      | `session_id` not in registry (any lifecycle op)                      | "Session not found" |
    /// | `"attach_failed"`          | `SessionError::SessionHostDead` on the attach-path                   | "Session attach failed" |
    /// | `"kill_failed"`            | `SessionError::SessionHostDead` on the kill-path (op-aware mapping via `IpcOp::Kill`) | "Session kill failed" |
    /// | `"rename_failed"`          | `SessionManager::rename_session()` returned error                    | "Session rename failed" |
    /// | `"invalid_request"`        | Generic/catch-all post-call code: `SessionError::Io` (unexpected I/O error) and any unmapped `EngineError` variant (`_ =>` forward-compat arm in `session_error_to_code()`); also reserved for any future pre-call validation failure path | "[operation failed]" |
    ///
    /// The `message` field carries a human-readable diagnostic string (not user-facing;
    /// logged by the TUI for diagnostics). The TUI renders the FIXED banner text from the
    /// table above keyed to `code`; it does NOT display `message` verbatim (avoids leaking
    /// internal detail). The fixed banner texts for `"binary_not_found"` and `"invalid_spawn_arg"`
    /// satisfy BC-2.03.007 PC-3 and PC-7 respectively (see §spawn_recipe() call site above).
    ///
    /// Future phases may add new codes. The TUI MUST handle unknown codes by rendering a
    /// generic `[operation failed]` banner without panicking (forward-compat with `#[non_exhaustive]`).
    Error {
        /// Machine-readable error class. One of the v1A taxonomy codes above.
        code: String,
        /// Human-readable diagnostic detail for logging. Not displayed verbatim to the user.
        message: String,
    },

    // ── F-P41-IMP-001 resolution (2026-06-14) ─────────────────────────────────

    /// Deterministic spawn acknowledgement. Sent by the daemon IPC handler to the
    /// REQUESTING CLIENT ONLY (never broadcast) immediately after the session UUID is
    /// generated and the `SpawnSession` IPC message is accepted, BEFORE calling
    /// `SessionManager::spawn_session()`.
    ///
    /// # Why this exists — F-P41-IMP-001 resolution
    ///
    /// `ClientToServer::SpawnSession` carries no correlation token. The daemon assigns the
    /// `session_id` (UUID v4) in the IPC handler and passes it to `spawn_session()` inside
    /// the completed `SpawnOptions`. The requesting TUI has no other way to learn this id
    /// deterministically — `SessionStateChanged { Launching }` is broadcast to ALL clients
    /// and carries no request correlation, making a heuristic "first unseen Launching id" race
    /// on multiple simultaneous spawns. `SpawnAck` closes this gap without a broadcast-race.
    ///
    /// # Delivery ordering (normative)
    ///
    /// The IPC handler MUST emit `SpawnAck` in this sequence, on the REQUESTING client's
    /// per-client channel only:
    ///   1. Generate `session_id` via `uuid::Uuid::new_v4().to_string()`.
    ///   2. Send `ServerToClient::SpawnAck { session_id: session_id.clone() }` to the
    ///      requesting client (per-client `client_tx.send(...)`) — NOT to the broker.
    ///   3. Build completed `SpawnOptions` via `opts.with_daemon_fields(session_id, hooks_path)`.
    ///   4. Call `SessionManager::spawn_session(opts)`.
    ///   5. On `Ok`: broker emits `SessionStateChanged { Launching }` + `SessionListUpdate`.
    ///   6. On `Err`: send `ServerToClient::Error` to requesting client (existing path unchanged).
    ///
    /// Step 2 MUST precede step 4 so that `SpawnAck` arrives at the TUI before any broadcast
    /// `SessionStateChanged { Launching }` event. The per-client FIFO channel guarantees
    /// in-order delivery to the requesting client.
    ///
    /// # TUI consumption (normative)
    ///
    /// On receipt of `SpawnAck { session_id }`, the TUI MUST:
    ///   - If `AppMode` is currently `SessionCreation { step: Launching, .. }`:
    ///     store `session_id` in `App::wizard_session_id` (see SS-09 §SessionCreation wizard).
    ///   - Otherwise: log WARN and ignore (SpawnAck arrived for a spawn the wizard no longer
    ///     owns — e.g., wizard was cancelled between SpawnSession send and SpawnAck receipt).
    ///
    /// # schema_version impact
    ///
    /// `SpawnAck` is a new `ServerToClient` variant. Because `ServerToClient` is `#[non_exhaustive]`,
    /// adding a new variant is forward-compatible for receivers that already handle unknown
    /// variants gracefully (required by `#[non_exhaustive]` contract). The wire `schema_version`
    /// in `session-state.json` is unchanged (that schema governs the sidecar file, not the
    /// IPC message types). No `schema_version` bump is required for IPC message type additions
    /// (the IPC protocol does not carry an explicit version field; forward-compat is handled
    /// by `#[non_exhaustive]` + serde tag).
    SpawnAck {
        /// The daemon-assigned session UUID (String, same type used everywhere else).
        /// The TUI stores this in `App::wizard_session_id` while `SessionCreation::Launching`
        /// is the active step, to enable deterministic session_id filtering in EC-303.
        session_id: String,
    },
}
```

<a id="servertoClientspawnack"></a>
#### ServerToClient::SpawnAck — normative cross-reference anchor

The `SpawnAck { session_id }` variant is defined above in the `ServerToClient` enum (F-P41-IMP-001,
2026-06-14). Full delivery-ordering and TUI-consumption spec is in the doc-comment for the variant.
This heading provides a stable anchor target for cross-references (POL-13 compliance).

### Client-to-Server Messages

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientToServer {
    /// User responded to a permission prompt.
    PermissionDecision {
        prompt_id: Uuid,
        decision: PermissionDecisionKind,
    },

    /// TUI-initiated re-attach to an existing session-host.
    ///
    /// I3-004 (v1.12.0): Added to support TUI re-attach after PtyReset and explicit
    /// user-initiated re-attach from the sessions panel (e.g., attaching to a Detached session).
    /// The TUI cannot send `DaemonToHost::Attach` directly (that is a daemon→session-host
    /// message). Instead the TUI sends `AttachSession` to the daemon over the shared UDS;
    /// the daemon calls `SessionManager::attach_session()` which issues `DaemonToHost::Attach`
    /// to the session-host and streams a fresh `ScrollbackChunk*` + `ScrollbackDumpComplete`
    /// sequence to all connected TUI clients.
    ///
    /// This replaces the incorrect BC-2.05.011 PC-3c description of sending "a fresh
    /// DaemonToHost::Attach" from the TUI (the TUI never sends DaemonToHost messages).
    ///
    /// I11-001 (v1.16.0): This variant is also sent AUTOMATICALLY by `enter_embedded_terminal()`
    /// when the TUI enters a session that has not yet received a `ScrollbackDumpComplete` in the
    /// current process lifetime (i.e., the parser is blank because the session was already running
    /// when this TUI process started). See SS-embedded-pty.md §EmbeddedTerminal ENTRY
    /// "Auto-attach on first entry". Three trigger cases:
    ///   (a) Auto-attach on first EmbeddedTerminal entry (blank parser, no prior dump received).
    ///   (b) Re-attach after PtyReset (parser corruption recovery).
    ///   (c) Chunk-count mismatch on ScrollbackDumpComplete (integrity retry per BC-2.05.011).
    AttachSession {
        session_id: String,
    },

    /// Keepalive probe. Daemon responds with Pong.
    /// NOTE: Ping is reserved for Phase 2 keepalive detection.
    /// Phase 1 implementations MUST accept and silently discard Ping if received —
    /// do NOT close the connection or return an error on receipt of an unexpected Ping.
    Ping,

    // ── v1A control-center additions (C5-001, ADR-0010 §IPC Message Type Additions) ─────

    /// Forward keyboard input bytes to the named session's PTY.
    /// The TUI encodes key events as terminal byte sequences (see SS-09 §Keyboard Encoding)
    /// before sending. The daemon routes to the session-host via `DaemonToHost::KeyInput`.
    KeyInput {
        session_id: String,
        /// Terminal-encoded key bytes (VT/Kitty/SGR depending on negotiated protocol).
        bytes: Vec<u8>,
    },

    /// Inform the daemon that the TUI's PTY widget has been resized. The daemon routes
    /// to the session-host via `DaemonToHost::Resize`, which calls `pty.resize()` and
    /// `parser.set_size()`.
    ResizePane {
        session_id: String,
        rows: u16,
        cols: u16,
    },

    /// Spawn a new harness session.
    ///
    /// The TUI sends spawn intent parameters (`SpawnOptions`). The daemon IPC handler:
    ///   1. Generates `session_id` via `uuid::Uuid::new_v4().to_string()` (F-P41-IMP-001:
    ///      UUID generation is in the IPC handler, NOT inside `spawn_session()`).
    ///   2. Fills `SpawnOptions.session_id` and `SpawnOptions.hooks_settings_path` via
    ///      `opts.with_daemon_fields(session_id.clone(), hooks_path)`.
    ///   3. Sends `ServerToClient::SpawnAck { session_id }` to the REQUESTING CLIENT ONLY
    ///      (not broadcast), so the TUI wizard can store the id for deterministic EC-303
    ///      session_id filtering before any `SessionStateChanged { Launching }` broadcast
    ///      arrives.
    ///   4. Calls `SessionManager::spawn_session(opts)`. Inside `spawn_session()`, the daemon
    ///      calls `engine_module.spawn_recipe(&opts)` FIRST to build the `SpawnRecipe`
    ///      (I27-001 Model A: daemon-side recipe construction). The `SpawnRecipe` is never
    ///      sent over IPC; only `SpawnOptions` crosses the wire.
    ///
    /// On success the daemon emits `SessionStateChanged { Launching }` + `SessionListUpdate`.
    /// On failure the daemon sends `ServerToClient::Error` to the requesting client with one of
    /// the spawn-path codes (SS-08 §Error handling; SS-05 §ServerToClient::Error taxonomy):
    /// - `"binary_not_found"` — `EngineError::BinaryNotFound` (harness not on PATH; reachable
    ///   because `spawn_recipe()` runs daemon-side via Model A)
    /// - `"invalid_spawn_arg"` — `EngineError::InvalidPath` (invalid argument to spawn_recipe())
    /// - `"spawn_failed"` — OS process spawn failure
    /// - `"sidecar_write_failed"` — sidecar I/O failure post-spawn
    /// - `"session_id_collision"` — UUID v4 collision (do not auto-retry)
    ///
    /// NOTE: If `spawn_session()` returns `Err`, the TUI will have already received `SpawnAck`.
    /// The `SpawnAck` session_id is therefore NOT guaranteed to correspond to a successfully
    /// spawned session. The TUI MUST clear `App::wizard_session_id` on receipt of
    /// `ServerToClient::Error` from the spawn path (treat as spawn failure; wizard returns
    /// to ProfilePicker per BC-2.09.008 PC-5 / BC-2.09.008 EC-252).
    SpawnSession {
        /// Spawn intent parameters from the TUI (SS-08 §SpawnOptions).
        /// `session_id` and `hooks_settings_path` are filled by the daemon IPC handler
        /// upon receipt (via `with_daemon_fields()`), before passing to
        /// `SessionManager::spawn_session()`. See step 1-2 above.
        opts: SpawnOptions,
    },

    /// Kill (terminate) a running or detached session. The daemon calls
    /// `SessionManager::kill_session()`, which sends `DaemonToHost::Kill` to the
    /// session-host and emits `SessionStateChanged { Terminating }` + `SessionListUpdate`.
    KillSession {
        session_id: String,
    },

    /// Detach from a session (session-host stays alive, PTY continues).
    /// The daemon calls `SessionManager::detach_session()`, which sends
    /// `DaemonToHost::Detach` and emits `SessionStateChanged { Detached }` + `SessionListUpdate`.
    DetachSession {
        session_id: String,
    },

    /// Rename a session's display label. The daemon calls `SessionManager::rename_session()`,
    /// which updates `SessionEntry.display_name` and emits `SessionListUpdate` ONLY
    /// (rename is not a state transition; `SessionStateChanged` is not emitted —
    /// see SS-daemon-wiring-v2-delta §3b C3-003 rule).
    RenameSession {
        session_id: String,
        new_name: String,
    },
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

/// The kind of permission decision the user made.
///
/// Named `PermissionDecisionKind` (not `PermissionDecision`) to avoid a name collision
/// with the `ClientToServer::PermissionDecision` variant — see §PermissionDecisionKind
/// Naming section below for the full rationale and BC-name mapping table.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionDecisionKind {
    /// Allow this invocation once.
    Allow,
    /// Allow and record a persistent auto-accept pattern (daemon-side; S-026).
    AcceptAlways,
    /// Deny this invocation.
    Deny,
}
```

### §PermissionDecisionKind Naming — Authoritative Reference

**Source of truth:** The Rust enum `PermissionDecisionKind` in `crates/monocle-ipc/src/types.rs`
is the authoritative definition for the wire-level permission decision type. BC prose,
SS specs, and story files that use abstract names such as `PermissionDecision::Accept`
or `PermissionDecision::Reject` MUST be interpreted via the mapping table below.

**Why two names exist:** The enum is named `PermissionDecisionKind` — not `PermissionDecision` —
to avoid a collision with the `ClientToServer::PermissionDecision` *variant*. If both had
the same identifier, every use site would require fully-qualified syntax and the code
would be confusing to read. The variant name `PermissionDecision` is kept because it
is the semantic name of the IPC operation; the payload type therefore takes the `Kind`
suffix.

**Canonical mapping table** (BC prose name → wire Rust variant):

| BC / prose name              | Rust variant (`PermissionDecisionKind`) | Keybinding | Semantics                                      |
|------------------------------|-----------------------------------------|------------|------------------------------------------------|
| `PermissionDecision::Accept` | `Allow`                                 | `y` / Enter | Allow this invocation once                    |
| `PermissionDecision::AcceptAlways` | `AcceptAlways`                    | `A`        | Allow + persist pattern for future auto-accept |
| `PermissionDecision::Reject` | `Deny`                                  | `n` / `r`  | Deny this invocation                           |

**Wire serialization:** `PermissionDecisionKind` has no `#[serde(rename)]` attributes;
variants serialize as their Rust identifiers: `"Allow"`, `"AcceptAlways"`, `"Deny"`.
The daemon reads these from the `ClientToServer::PermissionDecision.decision` field and
translates to the Claude Code hook protocol response (`{"decision":"accept"}`,
`{"decision":"always"}`, `{"decision":"deny"}`) in the daemon's IPC handler —
the TUI sends the `PermissionDecisionKind` variant verbatim and does not need to know
the hook protocol wire values.

**Rule for future BCs and specs:** When citing the permission decision type, use the
Rust name `PermissionDecisionKind` and its variants `Allow` / `AcceptAlways` / `Deny`.
The abstract BC shorthand names (`Accept`, `Reject`) remain acceptable in prose-level
descriptions provided they are understood to map via the table above and are NOT used
in Rust code snippets or test assertions.

### Supporting Types

`EnrichedSession`, `HookEvent`, and `HookType` are defined in `monocle-core` (SS-03).
`HookEventRecord` is defined in `monocle-runtime::ring` (SS-01). The `monocle-ipc` crate
depends on `monocle-runtime` for `HookEventRecord` in the `InitialState` message; this is
the only `monocle-runtime` dependency in `monocle-ipc`.

`SessionSnapshot` is defined in `monocle-ipc` (new in v1A, C3-004). `SerializedCell` and
`SerializedColor` are defined in `monocle-ipc` (new in v1A, C5-002) — see their definitions
below. `SpawnRecipe` is defined in `monocle-ipc` (or re-exported from `monocle-core`) so the
TUI can send `ClientToServer::SpawnSession` without importing `monocle-runtime` internals.

`SessionSnapshot` is defined in `monocle-ipc` (new in v1A, C3-004):

```rust
/// Canonical wire boundary type for session data in InitialState.sessions and
/// SessionListUpdate.sessions. Replaces Vec<EnrichedSession> on the wire.
///
/// Both monocle-spawned sessions (from SessionManager.session_list()) and
/// externally-detected sessions (from EngineModule::detect()) are converted to
/// SessionSnapshot before being placed on the wire. spawned_by_monocle distinguishes them.
///
/// C3-004: defined here (monocle-ipc) so both daemon and TUI share the type without
/// the TUI needing to import monocle-runtime internal types.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    pub display_name: String,
    pub state: SessionState,
    pub harness_id: String,
    pub project_root: String,
    pub cwd: String,
    /// Some(true) = monocle-spawned; Some(false) = externally-detected; None = legacy/unknown.
    pub spawned_by_monocle: Option<bool>,
    /// Unix epoch microseconds of session start.
    pub started_at_micros: i64,
    pub pty_rows: u16,
    pub pty_cols: u16,
    /// I3-009: true if session-host reported missing critical env vars (HOME, PATH, etc.).
    #[serde(default)]
    pub degraded: bool,
    /// I3-009: Human-readable degraded reason, e.g. "Missing env: HOME, PATH". None when healthy.
    #[serde(default)]
    pub degraded_reason: Option<String>,
}

impl SessionSnapshot {
    /// ADR-0006 constructor: required because `SessionSnapshot` is `#[non_exhaustive]` and
    /// constructed cross-crate by `monocle-runtime` (daemon) when building `InitialState`
    /// and `SessionListUpdate` payloads. Per Rust E0639, struct-literal construction is
    /// forbidden outside the defining crate for `#[non_exhaustive]` types. All 10 base
    /// fields are positional; `degraded` and `degraded_reason` use their `#[serde(default)]`
    /// defaults (false / None) when healthy — callers set them explicitly when degraded.
    ///
    /// # Construction path
    /// - `monocle-runtime` (daemon): `SessionManager::session_list()` and
    ///   `EngineModule::detect()` conversion both call `SessionSnapshot::new(...)` when
    ///   assembling `InitialState.sessions` and `SessionListUpdate.sessions`.
    /// - `monocle-ipc/tests/`: integration test binaries call `new(...)` directly.
    ///   Each `[[test]]` binary links `monocle-ipc` as external; E0639 applies.
    pub fn new(
        session_id: String,
        display_name: String,
        state: SessionState,
        harness_id: String,
        project_root: String,
        cwd: String,
        spawned_by_monocle: Option<bool>,
        started_at_micros: i64,
        pty_rows: u16,
        pty_cols: u16,
    ) -> Self {
        Self {
            session_id,
            display_name,
            state,
            harness_id,
            project_root,
            cwd,
            spawned_by_monocle,
            started_at_micros,
            pty_rows,
            pty_cols,
            degraded: false,
            degraded_reason: None,
        }
    }

    /// Builder method to set degraded state after construction.
    /// Called only when the session-host has reported missing critical env vars (I3-009).
    pub fn with_degraded(mut self, reason: String) -> Self {
        self.degraded = true;
        self.degraded_reason = Some(reason);
        self
    }
}
```

`SessionState` is defined in `monocle-ipc` (or re-exported from `monocle-core`) so both the
daemon's `SessionManager` and the TUI share the identical type with `Serialize`/`Deserialize`
derives. The same enum powers both the internal `SessionEntry.state` and the wire `SessionSnapshot.state`.

`SerializedCell` and `SerializedColor` are defined in `monocle-ipc` (new in v1A, C5-002).
They are the wire boundary types for styled terminal cell data carried in `ScrollbackChunk.rows`.
The authoritative definition lives here so that both the session-host (`monocle-session-host` binary)
and the TUI (`monocle-tui`) can share the type without either depending on the other's internal types.
`SS-session-manager.md` references `crate::ipc::SerializedCell` rather than owning the definition.

```rust
/// A single terminal cell serialized for scrollback dump (ScrollbackChunk rows).
/// Sufficient to reconstruct the full styled vt100::Screen without re-parsing PTY bytes.
///
/// # C5-002: defined in monocle-ipc (not monocle-session-host) so both daemon-side
/// session-host and TUI-side renderer share the type through monocle-ipc without
/// a cross-binary dependency. SS-session-manager.md §HostToDaemon references
/// `crate::ipc::SerializedCell` for `ScrollbackChunk.rows`.
///
/// # vt100 0.16 attribute surface (verified I3-008):
/// The vt100 0.16 `Cell` struct exposes EXACTLY FIVE attribute methods:
///   `bold()`, `dim()`, `italic()`, `underline()`, `inverse()`.
/// There is NO `blink()`, NO `hidden()`, and NO `strikethrough()` in vt100 0.16.
/// (Verified against docs.rs/vt100/0.16.0/vt100/struct.Cell.html — 2026-06-03.)
///
/// # attrs bitmask layout (5 bits, low-to-high):
///   bit 0: bold      (cell.bold())
///   bit 1: dim       (cell.dim())
///   bit 2: italic    (cell.italic())
///   bit 3: underline (cell.underline())
///   bit 4: inverse   (cell.inverse()) — SGR 7 "reverse video"
///   bits 5–7: reserved (MUST be 0 on write; MUST be ignored on read for forward-compat)
///
/// The u8 type is retained for forward-compat: if a future vt100 version exposes
/// additional attributes, they can occupy bits 5–7 without a wire format change.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedCell {
    /// The UTF-8 character at this cell (empty string for empty/null cells).
    pub ch: String,
    /// Foreground color.
    pub fg: SerializedColor,
    /// Background color.
    pub bg: SerializedColor,
    /// Cell attributes bitmask (5 bits used; see doc comment above for layout).
    pub attrs: u8,
}

impl SerializedCell {
    /// ADR-0006 constructor: required because `SerializedCell` is `#[non_exhaustive]` and
    /// constructed cross-crate by the `monocle-session-host` binary when serializing a
    /// `vt100::Screen` snapshot for `HostToDaemon::ScrollbackChunk`. Per Rust E0639, struct-
    /// literal construction is forbidden outside the defining crate for `#[non_exhaustive]`
    /// types. All 4 fields are required positional parameters.
    ///
    /// # Construction path
    /// - `monocle-session-host` binary: reads `vt100::Cell` attributes and constructs
    ///   `Vec<Vec<SerializedCell>>` for the scrollback dump (C5-002).
    ///   The binary links `monocle-ipc` as an external crate; E0639 applies.
    /// - `monocle-ipc/tests/`: integration test binaries that exercise scrollback
    ///   serialization call `new(...)` directly.
    pub fn new(ch: String, fg: SerializedColor, bg: SerializedColor, attrs: u8) -> Self {
        Self { ch, fg, bg, attrs }
    }
}

/// Terminal cell color as serialized for scrollback dump.
/// Covers ANSI 16-color, 256-color, and 24-bit RGB as exposed by vt100 0.16.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedColor {
    Default,
    Ansi(u8),
    Rgb(u8, u8, u8),
}
```

`SpawnOptions` is defined in `monocle-ipc` (or re-exported from `monocle-core`) so the TUI can
construct a `ClientToServer::SpawnSession { opts }` without importing `monocle-runtime` internal
types. Its fields are defined in SS-session-manager.md §SpawnOptions. `SpawnOptions` carries
`#[non_exhaustive]`, `Serialize`, `Deserialize` as a wire boundary type (I27-001 Model A).

`SpawnRecipe` is DAEMON-INTERNAL after I27-001 (Model A). It is built by
`engine_module.spawn_recipe(&opts)` inside `SessionManager::spawn_session()` and never
transmitted over IPC. The `Serialize`/`Deserialize` derives on `SpawnRecipe` in
SS-engine-module-v2-delta.md are redundant for a non-wire type (retained for potential
diagnostic serialization but carry no wire-protocol obligation). `SpawnRecipe` does NOT
need to be in `monocle-ipc`; it lives in `monocle-core` (alongside the `EngineModule` trait)
as a daemon-internal type. Its fields are defined in SS-engine-module-v2-delta.md §spawn_recipe().

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

impl PermissionPromptPayload {
    /// ADR-0006 constructor: required because `PermissionPromptPayload` is `#[non_exhaustive]`
    /// and constructed cross-crate by `monocle-runtime` (daemon) when a `PreToolUse` hook
    /// with `decision_required: true` arrives. The daemon creates the payload, embeds it in
    /// `ServerToClient::PermissionPromptQueued { payload }`, and pushes it to all connected
    /// TUI clients. The TUI only ever RECEIVES and serde-deserializes this type — it does
    /// NOT construct it. Per Rust E0639, struct-literal construction is forbidden outside
    /// the defining crate for `#[non_exhaustive]` types.
    ///
    /// # Construction path (daemon-side cross-crate, not serde-deserialize-only)
    /// - `monocle-runtime` (daemon hook handler, `src/hooks/pre_tool_use.rs`): constructs
    ///   `PermissionPromptPayload::new(...)` when building `PermissionPromptQueued`.
    ///   `monocle-runtime` depends on `monocle-ipc`; E0639 applies cross-crate.
    /// - `monocle-ipc/tests/`: integration test binaries that exercise the permission overlay
    ///   path call `new(...)` directly.
    /// - `monocle-tui` only ever serde-deserializes received payloads — no construction site.
    ///
    /// # ADR-0006 criteria
    /// (1) Internal workspace scope: `monocle-ipc` and `monocle-runtime` are both workspace
    ///     crates, never published to crates.io.
    /// (2) External protocol anchor: fields are driven by the Claude Code PreToolUse hook
    ///     payload; new optional fields (e.g., `preview_diff`) arise from Claude Code version
    ///     bumps requiring coordinated BC revisions.
    /// (3) All fields are positional parameters, including the optional fields (`old_content`,
    ///     `new_content`). The daemon knows their values at construction time from the
    ///     PreToolUse hook body — they are passed as `Option<String>` positional arguments,
    ///     NOT set after construction. There is no post-construction setter for these fields.
    pub fn new(
        prompt_id: Uuid,
        session_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
        old_content: Option<String>,
        new_content: Option<String>,
    ) -> Self {
        Self {
            prompt_id,
            session_id,
            tool_name,
            tool_input,
            old_content,
            new_content,
        }
    }
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
   - The full current `Vec<SessionSnapshot>` roster.
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

`ServerToClient` has no `Disconnected` variant — `Disconnected` belongs to the separate
process-local `TransportEvent` enum (defined in §Supporting Types) which is NOT serialized
over the wire. The reader task therefore sends a wrapped type so it can signal both wire
messages and transport disconnects on a single channel without inventing non-existent variants:

```rust
/// Events delivered by the IPC reader task to the event loop.
/// This is a process-local type — it is NOT a wire message and NOT derived
/// from `ServerToClient`. `Disconnected` maps to `TransportEvent::Disconnected`.
pub enum IpcRead {
    /// A wire message received from the daemon.
    Msg(ServerToClient),
    /// The UDS connection was lost (EOF, BrokenPipe, ConnectionReset).
    /// Triggers SOQ-3 overlay clear and reconnect (see §Reconnect handoff below).
    Disconnected,
}

/// Spawn a dedicated IPC reader task.
/// Returns a JoinHandle; the event loop holds the Receiver<IpcRead>.
pub fn spawn_ipc_reader(
    mut transport: UdsClientTransport,
    tx: mpsc::Sender<IpcRead>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match transport.recv_message().await {
                Ok(msg) => {
                    if tx.send(IpcRead::Msg(msg)).await.is_err() {
                        // Receiver dropped — event loop has exited; stop reading.
                        break;
                    }
                }
                Err(_) => {
                    // EOF, BrokenPipe, or ConnectionReset — signal disconnect.
                    // IpcRead::Disconnected corresponds to TransportEvent::Disconnected.
                    let _ = tx.send(IpcRead::Disconnected).await;
                    break;
                }
            }
        }
    })
}
```

- Event loop holds `mpsc::Receiver<IpcRead>` and `JoinHandle<()>`; drains via
  `ipc_rx.try_recv()` on each keyboard tick (16ms / ~60Hz cadence).
- The `IpcRead` wrapper is defined in `monocle-ipc` alongside `TransportEvent` so both
  the reader task and the event loop share the type without extra crate boundaries.

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
- **Reconnect handoff:** on receiving `IpcRead::Disconnected` from the channel:
  1. `reader_handle.abort()` — terminate the stale reader task.
  2. Invoke SOQ-3 handler — clear the TUI's local `VecDeque<PromptModal>` overlay stack.
     (`IpcRead::Disconnected` corresponds to `TransportEvent::Disconnected` — the SOQ-3
     invariant fires on this signal, per §SOQ-3 Is a TUI-Side Concern Only.)
  3. Call `monocle_ipc::reconnect::reconnect(...)` to obtain a fresh `UdsClientTransport`.
  4. Spawn a new `spawn_ipc_reader(new_transport, ipc_tx.clone())` task.
  The disconnect→SOQ-3 stale-overlay-prevention path is representable because
  `IpcRead::Disconnected` unambiguously signals the event loop that the transport was lost;
  the event loop then clears the overlay stack before reconnecting.

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

<a id="phase-1-transport-constraint"></a>
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
  └── depends on monocle-ipc (UdsTransport, ServerToClient, ClientToServer,
                               SessionSnapshot, SerializedCell, SerializedColor)

monocle-runtime (SS-04, daemon side)
  └── depends on monocle-ipc (UdsTransport server-side, message types)

monocle-session-host (SS-08 binary)
  └── depends on monocle-ipc (SerializedCell, SerializedColor for ScrollbackChunk serialization;
                               HostToDaemon/DaemonToHost types are defined in SS-session-manager)
```

`monocle-ipc` depends on `monocle-runtime` for `HookEventRecord` only (the `InitialState.ring_tail`
field type). This is a narrow, read-only dependency on the ring's storage record type.
The daemon and TUI both depend on `monocle-ipc` as consumers. The `Transport` trait and
message types remain decoupled from ring I/O logic — `HookEventRecord` is a pure data type
(no I/O, no async traits).

`SerializedCell` and `SerializedColor` are defined in `monocle-ipc` (C5-002) so that
`monocle-session-host` can import them without depending on `monocle-runtime`, and the TUI
can import them without depending on `monocle-session-host`. This is the same pattern as
`SessionSnapshot` (C3-004): the wire boundary type lives in `monocle-ipc` to avoid a
cross-process dependency chain.

Note: The `monocle-ipc → monocle-runtime` edge exists for `HookEventRecord` only (SS-01 ring
record type). If this becomes undesirable for Phase 4, `HookEventRecord` can be moved to
`monocle-core` — `monocle-tui` already depends on `monocle-runtime`, so no new transitive
edges are introduced in Phase 1.

---

## Module Purity Classification

| Module / Function | Classification | Rationale |
|-------------------|---------------|-----------|
| `ServerToClient` enum | Pure core | Data type only; `#[derive(Serialize, Deserialize)]`. No I/O. |
| `ClientToServer` enum | Pure core | Data type only. No I/O. |
| `PermissionDecisionKind` enum | Pure core | Data type only. No I/O. |
| `TransportEvent` enum | Pure core | Process-local signal type only; NOT serialized. No I/O. |
| `SessionSnapshot` struct | Pure core | Wire boundary data type. `#[derive(Serialize, Deserialize)]`. No I/O. |
| `SerializedCell` struct | Pure core | Wire boundary data type for scrollback cells. No I/O. |
| `SerializedColor` enum | Pure core | Wire boundary data type for cell color. No I/O. |
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

## §Trace v1.21.0

**F-P41-IMP-001 — `ServerToClient::SpawnAck` added; UUID-locus prose corrected throughout** (2026-06-14):

- **Finding (F-P41-IMP-001, IMPORTANT):** Three defects in the spawn-correlation design:
  1. `ClientToServer::SpawnSession` carried no correlation token. The daemon assigned the UUID
     but had no channel to return it to the requesting TUI, making deterministic session_id
     filtering in `AppMode::SessionCreation` (EC-303) impossible.
  2. The `ServerToClient::Error` variant's `# spawn_recipe() call site` section said "the daemon
     IPC handler fills `SpawnOptions.session_id` (pre-generated UUID)". The term "pre-generated"
     was ambiguous — it implies the UUID was generated before the message arrived rather than by
     the IPC handler on receipt.
  3. `ClientToServer::SpawnSession` variant doc-comment said "The daemon IPC handler fills
     `SpawnOptions.session_id` and `SpawnOptions.hooks_settings_path` on receipt, then calls
     `SessionManager::spawn_session(opts)`" — missing the SpawnAck step entirely.
- **Decision — Mechanism (b):** `ServerToClient::SpawnAck { session_id }` is added as a new
  `ServerToClient` variant. It is sent to the REQUESTING CLIENT ONLY (not broadcast) by the
  IPC handler in the same `ClientToServer::SpawnSession` match arm, BEFORE calling
  `spawn_session()`. This gives the TUI wizard a deterministic UUID via a point-to-point message
  with guaranteed ordering (per-client FIFO channel delivers SpawnAck before any broker-published
  `SessionStateChanged { Launching }`). Mechanism (a) (broadcast-race heuristic, "claim first
  unseen Launching id") was rejected as production-grade non-compliant: it fails on multiple
  simultaneous spawns and has no correlation guarantee even in the v1A single-TUI case.
- **Fix (a) — `ServerToClient::SpawnAck` variant added:** New variant with `session_id: String`
  field. Full normative doc-comment specifies delivery ordering (5 steps), TUI consumption
  obligation (store in `AppMode::SessionCreation::launching_session_id`), and schema_version
  impact (no `session-state.json` schema bump needed; IPC forward-compat via `#[non_exhaustive]`).
- **Fix (b) — `ServerToClient::Error` spawn section rewritten:** "pre-generated UUID" →
  explicit "generates the session UUID (`uuid::Uuid::new_v4().to_string()`)" with SpawnAck
  step described in the call chain.
- **Fix (c) — `ClientToServer::SpawnSession` variant doc-comment rewritten:** Full 4-step
  IPC handler sequence documented. SpawnAck added as step 3. Spawn-failure case clarified:
  TUI must clear `wizard_session_id` on spawn error even though SpawnAck has been received.
- Semver: minor (v1.20.1 → v1.21.0) — new `ServerToClient::SpawnAck` wire variant;
  normative IPC handler behavior added.

## §Trace v1.20.1

**P31-LOW-001 — `PermissionPromptPayload::new()` doc-comment criterion (3) contradicted the constructor signature** (2026-06-13):

- **Finding (P31-LOW-001):** ADR-0006 criterion (3) in `PermissionPromptPayload::new()` stated: "optional fields (`old_content`, `new_content`) default to `None` and may be set after construction." This was inaccurate: the constructor DOES take `old_content: Option<String>` and `new_content: Option<String>` as positional arguments (as confirmed by the constructor body immediately below the comment). There is no post-construction setter. The audit table row in SS-engine-module.md (v1.1.27) correctly documented these as positional parameters ("all current fields positional (including the two Option<String> which are known at construction time)"). The doc-comment was the only incorrect site.
- **Fix:** Criterion (3) rewritten to state: "All fields are positional parameters, including the optional fields (`old_content`, `new_content`). The daemon knows their values at construction time from the PreToolUse hook body — they are passed as `Option<String>` positional arguments, NOT set after construction. There is no post-construction setter for these fields." This matches the audit table and the actual constructor signature.
- Semver: patch (v1.20.0 → v1.20.1) — doc-comment accuracy fix; no behavioral change.

## §Trace v1.20.0

**C30-002/I30-001 — ADR-0006 constructor gap: `SessionSnapshot`, `SerializedCell`, `PermissionPromptPayload` lacked public constructors despite cross-crate construction** (2026-06-13):

- **Finding (C30-002 CRITICAL):** `SessionSnapshot` and `SerializedCell` are `#[non_exhaustive]` structs defined in `monocle-ipc` and constructed cross-crate (daemon/session-host code in `monocle-runtime`/`monocle-session-host` binary). No `pub fn new(...)` constructor existed, so any `struct SessionSnapshot { ... }` literal outside `monocle-ipc` would be E0639 at compile time.
- **Finding (I30-001 IMPORTANT — adjudicated as cross-crate construction):** `PermissionPromptPayload` is `#[non_exhaustive]` and constructed by `monocle-runtime` (daemon hook handler `src/hooks/pre_tool_use.rs`) when building `ServerToClient::PermissionPromptQueued { payload }`. The daemon actively creates this struct and sends it. The TUI only ever serde-deserializes received payloads. This is NOT a serde-deserialize-only exemption (unlike `SessionStartEvent` which is never constructed outside `monocle-core`). Construction is daemon-side, cross-crate (`monocle-runtime` → `monocle-ipc`). Adjudication: **constructor required**.
- **Fix — `SessionSnapshot::new(session_id, display_name, state, harness_id, project_root, cwd, spawned_by_monocle, started_at_micros, pty_rows, pty_cols) -> Self`:** Added in §Supporting Types, immediately after the struct definition. The two `#[serde(default)]` fields (`degraded: bool`, `degraded_reason: Option<String>`) are initialized to `false`/`None` in the constructor body; the `with_degraded(reason: String) -> Self` builder method sets them when the I3-009 degraded-env path fires. This matches ADR-0006 §Breaking-Change Discipline: optional fields default in the body, not as positional parameters.
- **Fix — `SerializedCell::new(ch, fg, bg, attrs) -> Self`:** Added immediately after the struct definition. All 4 fields are required; none are optional. Called by `monocle-session-host` when serializing `vt100::Cell` values for `HostToDaemon::ScrollbackChunk`.
- **Fix — `PermissionPromptPayload::new(prompt_id, session_id, tool_name, tool_input, old_content, new_content) -> Self`:** Added immediately after the struct definition. The `old_content: Option<String>` and `new_content: Option<String>` fields are present as positional parameters because they are populated at construction time (not post-construction) from the PreToolUse hook body — the daemon knows their values at the moment it creates the payload.
- **ADR-0006 criteria satisfied for all three:** (1) internal workspace scope; (2) field additions driven by external protocol evolution (Claude Code version bumps) requiring BC revisions; (3) all required fields as positional parameters.
- **Audit table:** All three structs added to `SS-engine-module.md §Cross-Crate Constructor Audit Table` (v1.1.26 → v1.1.27).
- Semver: minor (v1.19.0 → v1.20.0) — additive constructor additions; no behavioral change.

## §Trace v1.19.0

**I27-001 — spawn-path wire-type correction: `ClientToServer::SpawnSession` now carries `SpawnOptions`, not `SpawnRecipe`** (2026-06-13):

- **Finding (I27-001):** `ClientToServer::SpawnSession { recipe: SpawnRecipe }` was the wire payload, but under the correct Model A architecture the `SpawnRecipe` is built daemon-side (by `engine_module.spawn_recipe(&opts)` inside `spawn_session()`). Sending a pre-built `SpawnRecipe` over IPC implies the TUI calls `spawn_recipe()` locally — which requires TUI-side access to the `EngineModule`, contradicts the thin-client principle, and makes `EngineError::BinaryNotFound`/`InvalidPath` TUI-local errors with no wire bridge to `ServerToClient::Error`. The §spawn_recipe() call-site doc-comment also contained a false "either way" hedge that acknowledged both Model A and Model B as valid — this is incorrect; only Model A is architecturally valid.
- **Fix (a) — `ClientToServer::SpawnSession` payload changed:** `recipe: SpawnRecipe` → `opts: SpawnOptions`. `SpawnOptions` is the wire type. The daemon IPC handler fills `session_id` and `hooks_settings_path` upon receipt, then passes the completed `SpawnOptions` to `SessionManager::spawn_session(opts)`.
- **Fix (b) — §spawn_recipe() call-site doc-comment rewritten:** The false "either way" hedge is removed. The section now unambiguously states: `spawn_recipe()` runs DAEMON-SIDE inside `spawn_session()`; the TUI sends `SpawnOptions` (intent); `SpawnRecipe` is daemon-internal and never transmitted over IPC. The reachability of `"binary_not_found"` and `"invalid_spawn_arg"` codes is explicitly justified: they are reachable ONLY because `spawn_recipe()` runs daemon-side (I27-001 Model A).
- **Fix (c) — `SpawnSession` variant doc-comment updated:** States daemon-side recipe construction, lists all spawn-path error codes with updated `"binary_not_found"` note ("reachable because `spawn_recipe()` runs daemon-side via Model A"), and documents the `session_id`/`hooks_settings_path` daemon-fill step.
- **Fix (d) — §Supporting Types updated:** `SpawnOptions` documented as the wire boundary type for `SpawnSession`. `SpawnRecipe` documented as daemon-internal (no longer a wire type; lives in `monocle-core` not `monocle-ipc`).
- **Wire-type reconciliation:** `SpawnOptions` acquires `#[non_exhaustive]`, `Serialize`, `Deserialize` (wire boundary per BC-2.02.003). `SpawnRecipe` loses its wire-type status; its `Serialize`/`Deserialize` derives are now optional (retained in SS-engine-module-v2-delta.md for potential diagnostic use, but carry no wire-protocol obligation).
- Semver: minor (v1.18.0 → v1.19.0) — breaking wire-format change to `ClientToServer::SpawnSession` payload type.

## §Trace v1.18.0

**S26-001 — `PermissionDecisionKind` missing `#[non_exhaustive]` (exhaustive wire-type class sweep)** (2026-06-13):

- **Finding (S26-001 exhaustive sweep):** `PermissionDecisionKind` was declared
  `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` without `#[non_exhaustive]`.
  The §Message Types blanket policy in this file states: "All public enums and message structs
  carry `#[non_exhaustive]` per the SS-02 extensibility policy (BC-2.02.003)."
  `PermissionDecisionKind` is a public wire enum: it is carried in
  `ClientToServer::PermissionDecision { decision: PermissionDecisionKind }`, which crosses the UDS
  IPC boundary. No documented exclusion analogous to `TransportEvent` (closed transport-state set
  requiring exhaustive TUI handling) or `SerializedColor` (closed vt100 0.16 color model) exists
  for `PermissionDecisionKind`. Future Claude Code protocol versions could introduce additional
  decision kinds (e.g., a permanent-deny variant); `#[non_exhaustive]` is the correct forward-
  compat mechanism.
- **Fix — `#[non_exhaustive]` added above `#[derive(...)]` on `PermissionDecisionKind`.**
- **Closed-set claim assessed:** The current variants (`Allow`, `AcceptAlways`, `Deny`) match the
  Phase 1 Claude Code hook protocol values, but the set is NOT guaranteed closed by that protocol.
  Unlike `SerializedColor` (whose variants are structurally determined by the vt100 0.16 Cell color
  API and cannot grow without a library version change) and `TransportEvent` (whose exhaustiveness
  is intentional by design to force TUI-side handling of new transport states), `PermissionDecisionKind`
  is a semantic policy set that can expand without a library constraint. `#[non_exhaustive]` is
  correct here. Consumers using exhaustive match on `PermissionDecisionKind` must add a `_ =>` arm.
- Semver: minor (v1.17.0 → v1.18.0) — normative attribute addition; affects exhaustiveness rules
  at all Rust match sites on `PermissionDecisionKind`.

## §Trace v1.17.0

**I12-001 / S12-002 — EngineError taxonomy bridge + spawn_recipe() call-site + multi-client scope clarification** (2026-06-04):

- **I12-001 (a) — `ServerToClient::Error` taxonomy extended from 8 to 10 codes:**
  Two new spawn-path codes added to satisfy BC-2.03.007 PC-3/PC-7 distinct diagnostics:
  - `"binary_not_found"` — maps from `EngineError::BinaryNotFound`; fixed banner:
    "claude binary not found — is Claude Code installed and on PATH?"
  - `"invalid_spawn_arg"` — maps from `EngineError::InvalidPath`; fixed banner:
    "Session spawn failed: invalid hooks settings path (non-UTF-8)"
  These codes are added at the TOP of the taxonomy table (before `"spawn_failed"`) because
  they occur earlier in the spawn flow (recipe validation precedes OS process spawn).
  Total v1A codes: 10 (was 8). Taxonomy table updated with a "TUI fixed banner text" column.
- **I12-001 (b) — spawn_recipe() call-site specified in §ServerToClient::Error doc-comment:**
  New "# spawn_recipe() call site" subsection added documenting that `spawn_recipe()` is called
  inside `SessionManager::spawn_session()` BEFORE `SessionHostSpawner::spawn()`, and that
  `EngineError` is bridged via `From<EngineError> for SessionError` (see SS-session-manager.md
  §SessionError taxonomy for the `EngineError` variant and `session_error_to_code()` arms).
- **I12-001 (c) — SpawnSession ClientToServer doc-comment updated:** Now enumerates all five
  spawn-path error codes (was only `"spawn_failed"`).
- **S12-002 — §Scope multi-client boundary clarified:** The prior §Scope presented concurrent
  multi-TUI-client as a present-tense capability. The v1A constraint (single TUI client) and the
  multi-client deferral anchor (`TD-MULTI-CLIENT-ATTACH-STORM-001`) were only documented in
  SS-daemon-wiring-v2-delta §5b and BC-2.05.009 Inv-2. §Scope rewritten to: (a) state v1A = one
  TUI client; (b) describe the fan-out infrastructure as forward-compatible; (c) note the deferral
  anchor; consistent with SS-daemon-wiring-v2-delta v1.6.0 §5b scope note.
- Semver: minor (v1.16.0 → v1.17.0) — new normative codes + scope clarification.

## §Trace v1.16.0

**I11-001 PRONG A — AttachSession auto-attach-on-entry trigger documented** (2026-06-04):

- **Finding (I11-001 PRONG A):** The `ClientToServer::AttachSession` doc-comment listed only
  two trigger cases: (b) re-attach after PtyReset, and (c) explicit user re-attach from sessions
  panel. The auto-attach-on-first-EmbeddedTerminal-entry case (blank parser, session already
  running when TUI process started) was not enumerated. The trigger is normatively specified in
  SS-embedded-pty.md §EmbeddedTerminal ENTRY v1.4.0 but was not cross-referenced here.
- **Fix:** `ClientToServer::AttachSession` doc-comment extended with case (a): auto-attach on
  first EmbeddedTerminal entry (blank parser, no prior dump received). All three trigger cases
  are now enumerated in the doc-comment. Cross-reference to SS-embedded-pty.md §EmbeddedTerminal
  ENTRY added.
- Semver: patch (v1.15.0 → v1.16.0) — doc-comment extension; no normative change to this file's
  wire types. The normative change is in SS-embedded-pty.md v1.4.0.

## §Trace v1.15.0

**Pass-7 architecture findings — I-P7-001/I-P7-002/I-P7-003** (2026-06-03):

- **I-P7-001 (error code taxonomy — two new codes):** v1A error code taxonomy updated to add
  `"sidecar_write_failed"` (maps from `SessionError::SidecarWriteFailed`) and
  `"session_id_collision"` (maps from `SessionError::SessionIdCollision`). Both are spawn-path
  codes for failures that occur after the OS process has been spawned but before the session
  is registered. Distinct codes are warranted because the diagnostics are distinct: a sidecar
  I/O failure points to filesystem/permissions issues; a collision points to a UUID generation
  anomaly. Total v1A codes: 8 (was 6). The "When emitted" doc list for `ServerToClient::Error`
  gains: `SpawnSession → SessionError::SidecarWriteFailed` and
  `SpawnSession → SessionError::SessionIdCollision`. `session_error_to_code()` in SS-08 §Error
  handling (SS-session-manager.md v1.7.0) is now exhaustive over all 7 `SessionError` variants.
- **I-P7-002 (kill_failed reachable via op-aware mapping):** `"kill_failed"` was previously
  unreachable because `session_error_to_code(&SessionError)` mapped `SessionHostDead`
  unconditionally to `"attach_failed"`. The function is now op-aware:
  `session_error_to_code(IpcOp, &SessionError)`. Kill-path `SessionHostDead` now correctly
  maps to `"kill_failed"`; all other paths map to `"attach_failed"`. The taxonomy table
  updated to reflect this: `"attach_failed"` row narrowed to "attach-path"; `"kill_failed"`
  row updated from "kill_session() returned error" to "SessionError::SessionHostDead on
  the kill-path (op-aware mapping via IpcOp::Kill)". `kill_failed` is now fully reachable.
- **I-P7-003 (ServerToClient::Disconnected non-variant fixed):** The `spawn_ipc_reader`
  canonical code block (~lines 789–810) was sending `ServerToClient::Disconnected` on
  transport error — a variant that does not exist on `ServerToClient`. `Disconnected` belongs
  to `TransportEvent` (process-local, not serialized). Fixed by:
  (1) Defining a `pub enum IpcRead { Msg(ServerToClient), Disconnected }` wrapper type
  (defined in `monocle-ipc`) as the channel payload type. `spawn_ipc_reader` now sends
  `IpcRead::Msg(msg)` for wire messages and `IpcRead::Disconnected` for transport errors.
  (2) The reconnect handoff prose (~line 828) updated to match: "on receiving
  `IpcRead::Disconnected` from the channel" (was "on disconnect (`Ok(Err(_))` or channel
  `Disconnected`)"). The disconnect→SOQ-3 stale-overlay-prevention path is confirmed
  representable via `IpcRead::Disconnected`.
  Reference implementation: `crates/monocle-tui/src/app.rs::spawn_ipc_reader` (S-025)
  uses a compatible channel-based disconnect signal; the spec pattern now matches the
  shipped implementation's approach.

## §Trace v1.14.0

**C6-001 — Pass-6 ServerToClient::Error variant added (13th variant); no-silent-failure on lifecycle ops** (2026-06-03):

- **C6-001(a) — ServerToClient::Error variant added:** `ServerToClient::Error { code: String, message: String }` added as the 13th variant of `ServerToClient`. This closes the silent-failure gap where the daemon could swallow `Err(SessionError::...)` from any lifecycle operation and leave the TUI hanging in `Launching` state with no user-visible signal.
  - `ServerToClient` grows from 12 variants to **13 variants** (InitialState, SessionListUpdate, HookEventReceived, PermissionPromptQueued, PermissionPromptResolved, DropCounterUpdate, Pong, PtyOutput, SessionStateChanged, ScrollbackChunk, ScrollbackDumpComplete, PtyReset, **Error** — in definition order).
  - The variant is `#[non_exhaustive]` alongside the rest of the enum.
  - Full v1A error code taxonomy documented inline: `spawn_failed`, `session_not_found`, `attach_failed`, `kill_failed`, `rename_failed`, `invalid_request`.
  - No-silent-failure invariant stated: the IPC handler MUST NOT return `Ok(())` at the task boundary on `Err(SessionError::...)`. Every error from `SessionManager` MUST produce `ServerToClient::Error` to the requesting client.
  - `SpawnSession` `ClientToServer` doc-comment cross-reference updated: "SS-08 §Error handling" now also cites "SS-05 §ServerToClient::Error taxonomy" so the link resolves to the new variant definition. The dangling reference to a non-existent section is resolved by the §Error handling addition in SS-session-manager.md v1.6.0 (same burst).
- **C6-001(b) — 12→13 census sweep:** Searched all architecture files for "12 ServerToClient", "ServerToClient.*12", "12 variant" census claims. Survivors:
  - SS-ipc.md §Trace v1.13.0 line "C5-001 (11 v1A wire variants added)" — **no change required**: this is a historical count of v1A-round-5 _additions_ (the 5 ServerToClient + 6 ClientToServer additions in Pass 5), not a total variant count claim. The addend "11 additions" remains correct regardless of the total reaching 13.
  - No other file carries a "12 ServerToClient variants" claim. The total variant count was never stated as a census number in any spec; the prior version was 12 by construction (7 original + 5 Pass-5 additions), now 13.

## §Trace v1.13.0

**C5-001/C5-002 — Pass-5 wire-type authority consolidation** (2026-06-03):

- **C5-001 (11 v1A wire variants added):** `ServerToClient` and `ClientToServer` enum bodies
  extended with all v1A control-center variants previously defined ONLY in ADR-0010
  §IPC Message Type Additions and SS-daemon-wiring-v2-delta §5. SS-ipc.md is now the
  complete IPC wire authority; BC Architecture-Source citations for SS-05 BCs resolve here.
  - `ServerToClient` additions (5): `PtyOutput { session_id, bytes }`,
    `SessionStateChanged { session_id, new_state }`,
    `ScrollbackChunk { session_id, rows: Vec<Vec<SerializedCell>>, chunk_seq }`,
    `ScrollbackDumpComplete { session_id, total_chunks, cursor_row, cursor_col, pty_rows, pty_cols }`,
    `PtyReset { session_id }`.
  - `ClientToServer` additions (6): `KeyInput { session_id, bytes }`,
    `ResizePane { session_id, rows, cols }`, `SpawnSession { recipe: SpawnRecipe }`,
    `KillSession { session_id }`, `DetachSession { session_id }`,
    `RenameSession { session_id, new_name }`.
  - All new variants added to existing `#[non_exhaustive]` enums — no breaking change to
    existing consumers using `..` wildcard patterns.
  - Field schemas match ADR-0010 and SS-daemon-wiring-v2-delta exactly. Inline doc comments
    cross-reference ADR-0010, SS-daemon-wiring-v2-delta, and I3-003/C3-003 rules.
- **C5-002 (SerializedCell + SerializedColor defined in monocle-ipc):** These wire types are
  referenced by `ScrollbackChunk.rows` and must be available to both `monocle-session-host`
  (writer) and `monocle-tui` (reader) without a cross-binary dependency. Defined in
  `monocle-ipc` §Supporting Types (same pattern as `SessionSnapshot` in C3-004).
  Field definitions match SS-session-manager.md v1.4.1 exactly: `ch: String`, `fg/bg:
  SerializedColor`, `attrs: u8` with the verified vt100-0.16 5-flag bitmask
  (bold/dim/italic/underline/inverse; NO blink, hidden, strikethrough — I3-008 verified).
  `SerializedColor` variants: `Default`, `Ansi(u8)`, `Rgb(u8, u8, u8)`.
  `SerializedCell` carries `#[non_exhaustive]`; `SerializedColor` does not
  (closed variant set matching vt100 0.16 color model).
  - SS-session-manager.md §HostToDaemon `ScrollbackChunk.rows` type updated from
    local `SerializedCell` definition to `crate::ipc::SerializedCell` reference in v1.5.0.
  - Dependency graph updated: `monocle-session-host` depends on `monocle-ipc` for
    `SerializedCell`/`SerializedColor`; `monocle-tui` same.
  - Module Purity Classification table updated: `SerializedCell` and `SerializedColor` rows added.

## §Trace v1.12.1

**F-01 closure-audit: §Connection Lifecycle prose Vec<EnrichedSession>→Vec<SessionSnapshot>** (2026-06-03):

- **Line ~479 (§Connection Lifecycle §Phase 1 Connect step 4):** Stale prose "The full current
  `Vec<EnrichedSession>` roster." corrected to "The full current `Vec<SessionSnapshot>` roster."
  This is the only place in the document where `EnrichedSession` appeared as the *wire roster type*
  in running prose rather than in historical changelog context or internal-type references.
  The `ServerToClient::InitialState.sessions` field and `SessionListUpdate.sessions` field were
  already correctly typed `Vec<SessionSnapshot>` in the code blocks and §Supporting Types;
  only this one prose line lagged.
- **Line ~144 (§Framing Protocol — JSON rationale):** Secondary stale reference corrected.
  "Message types reference `EnrichedSession` and `HookEvent` structs" changed to reference
  `SessionSnapshot`, `HookEventRecord`, and `HookEvent` — the actual wire-boundary types after
  C3-004. `EnrichedSession` is an internal `EngineModule::detect()` type and is never placed on
  the IPC wire.
- **Scope note:** All other `EnrichedSession` occurrences in this file are legitimate:
  historical C3-004 changelog text, `SessionSnapshot` doc-comment (explaining what it replaces),
  `PermissionPromptPayload` cross-reference for `session_id: String` type consistency,
  §Dependency Graph internal-type listing, and §Trace footnote history. None of these represent
  stale wire-type claims.

## §Trace v1.12.0

**C3-004/I3-004 — Adversarial Pass 3 resolution: SessionSnapshot + AttachSession** (2026-06-03):

- **C3-004 (SessionSnapshot defined):** §Supporting Types: `SessionSnapshot` struct defined
  in `monocle-ipc` with all fields (session_id, display_name, state, harness_id, project_root,
  cwd, spawned_by_monocle, started_at_micros, pty_rows, pty_cols, degraded, degraded_reason).
  `InitialState.sessions` and `SessionListUpdate.sessions` changed from `Vec<EnrichedSession>`
  to `Vec<SessionSnapshot>`. `SessionState` is defined in/re-exported from `monocle-ipc`.
  `EnrichedSession` is retained for `EngineModule::detect()` internal use but is NOT the wire
  type. Three-representation reconciliation cross-referenced to SS-daemon-wiring-v2-delta §4.
- **I3-004 (ClientToServer::AttachSession added):** `ClientToServer::AttachSession { session_id }`
  added to the `ClientToServer` enum. Used for TUI-initiated re-attach after PtyReset and
  explicit user re-attach from sessions panel. Daemon routes to `SessionManager::attach_session()`.
  Replaces the incorrect BC-2.05.011 PC-3c reference to "fresh DaemonToHost::Attach from TUI"
  (TUI cannot send DaemonToHost messages — that is a daemon→session-host message direction).

## §Trace v1.11.0

**F-S026-ADV1-LOW-002 — PermissionDecisionKind naming reconciliation** (2026-06-03):

- **Finding:** BC prose and the SS-ipc.md v1.10.0 spec code block both referenced a
  `PermissionDecision { Accept, AcceptAlways, Reject }` enum that does not exist in the
  production codebase. The actual wire type is `PermissionDecisionKind { Allow, AcceptAlways, Deny }`
  (`crates/monocle-ipc/src/types.rs`). Multiple adversarial passes flagged the prose-vs-type
  drift as a process gap routed to architect (F-S026-ADV1-LOW-002).
- **Root cause:** The production enum was renamed to `PermissionDecisionKind` during S-026
  implementation (to avoid a collision with the `ClientToServer::PermissionDecision` variant),
  but the spec code block was not updated at the same time. A `PermissionDecision` enum with
  serde-rename attributes (`"accept"` / `"always"` / `"deny"`) was authored in the spec as
  a design intent artifact; the implementation chose different variant names and no serde
  renames (variant names serialize as their Rust identifiers).
- **Resolution (spec-only; no production enum renamed):**
  1. `ClientToServer` code block: `decision: PermissionDecision` → `decision: PermissionDecisionKind`.
  2. Stale `PermissionDecision` enum code block replaced with correct `PermissionDecisionKind`
     definition (matching production: variants `Allow`, `AcceptAlways`, `Deny`; no serde renames).
  3. Added `§PermissionDecisionKind Naming — Authoritative Reference` section with:
     - Rationale for the `Kind` suffix (collision avoidance).
     - Canonical mapping table: BC prose name → Rust variant → keybinding → semantics.
     - Wire serialization note (no serde renames; variants serialize as Rust identifiers).
     - Hook-protocol translation note (daemon translates to `{"decision":"..."}` values;
       the TUI is not responsible for hook protocol wire values).
     - Rule for future BCs and specs.
  4. Module Purity Classification table: `PermissionDecision` row → `PermissionDecisionKind`.
- **Follow-up:** BC files (BC-2.06.011, BC-2.06.012, BC-2.06.013, BC-2.06.022) use
  abstract prose names (`PermissionDecision::Accept`, `PermissionDecision::Reject`).
  These are acceptable shorthand provided they are understood via the mapping table above;
  no BC version bumps required. Future BCs SHOULD prefer the Rust names directly.

## §Trace v1.10.0

**S-028 ADR — `HookEventReceived.timestamp_micros` field added (breaking change)** (2026-06-01):

- **Problem:** Adversarial review found that `ServerToClient::HookEventReceived` carried no
  daemon-side timestamp. The TUI event ribbon (BC-2.05.004 PC-1, S-028) was therefore forced
  to use `SystemTime::now()` at receipt time as the displayed event time — showing IPC transit
  latency, not the real event time. BC-2.05.004 PC-1 requires the ribbon to render the wall-clock
  event time as determined by the daemon.
- **Decision:** Add `timestamp_micros: i64` (signed Unix epoch microseconds) to
  `ServerToClient::HookEventReceived`. Field type matches `HookEventRecord::timestamp_micros`
  (the canonical daemon ring record type per BC-2.04.012 PC-1 and SS-core-types-and-abi.md
  §HookEventRecord). Signed `i64` is the correct type; unsigned `u64` was considered and
  rejected because `HookEventRecord` is already `i64` (consistency beats theoretical
  unsigned advantage for timestamps in the range 1970–2262).
- **Daemon-side obligation:** The hook handler that emits `HookEventReceived` MUST populate
  `timestamp_micros` from the same clock call used to populate `HookEventRecord::timestamp_micros`
  for the ring write. Both must use the same timestamp: capture once via
  `std::time::SystemTime::now()` → Unix micros conversion, write to ring record, write to IPC
  message. Do NOT capture two separate `SystemTime::now()` values.
- **TUI-side obligation:** `App::on_hook_event_received` MUST extract `timestamp_micros` from
  the message struct and pass it to the `HookEventRow` constructor. The ribbon render path
  converts `timestamp_micros` to a wall-clock `HH:MM:SS.mmm` string using
  `chrono::DateTime::<Utc>::from_timestamp_micros(timestamp_micros)`. MUST NOT substitute
  `Utc::now()` or `SystemTime::now()`.
- **Breaking-change scope:** This is a Rust struct field addition to a non-`#[non_exhaustive]`
  enum variant. All struct literal constructions of `HookEventReceived` fail to compile until
  `timestamp_micros` is added. Match arms using named bindings also fail. Match arms using `..`
  compile but silently discard the field — those must be audited. Full consumer list is
  documented in the variant doc comment above.
- **Wire format:** JSON key `"timestamp_micros"` added to the `HookEventReceived` object.
  Old TUI builds (< v1.10.0) that receive a v1.10.0 daemon's `HookEventReceived` will fail
  serde deserialization (unknown field). This is accepted: Phase 1 TUI and daemon are always
  co-deployed; no cross-version compatibility is required until Phase 4 federation.

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
- input-hash: 50aa63d — computed manually (YAML-object inputs: not resolved by compute-input-hash scan; see F-S025-ADV5-HIGH-001).
- SE-16d PASS: 2026-05-26T02:00:00Z > chain high-water 2026-05-26T01:00:00Z (monotonic).
