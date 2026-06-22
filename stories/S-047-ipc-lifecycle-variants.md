---
document_type: story
level: L4
story_id: S-047
epic_id: EPIC-05
version: "1.8"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-22T00:00:00Z
phase: 2
points: 16
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-021, S-022, S-023, S-033, S-034, S-035, S-046]
blocks: [S-048]
target_module: monocle-ipc
subsystems: [SS-05, SS-08]
behavioral_contracts: [BC-2.05.010, BC-2.05.011]
verification_properties: []
estimated_days: 5
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.010.md, version: "1.9.11"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.011.md, version: "1.2.12"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.25.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.18.0"}
  - {path: .factory/specs/architecture/SS-conventions-anti-patterns.md, version: "1.32.6"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.05.010 (KeyInput + RenameSession IPC routing with no-silent-failure invariant) and BC-2.05.011 (ScrollbackChunk*/Complete/PtyReset server-to-client variants + pending_pty_bytes buffer + post-dump replay). Expanded per human ruling 2026-06-22: session-host producer legs (PTY read loop, vt100 parser, KeyInput write, real scrollback dump) added to scope. NOTE: ResizePane IPC arm and resize_session() remain S-042 per human ruling 2026-06-21."
# BC status: BC-2.05.010 and BC-2.05.011 non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-047: IPC Lifecycle Variants — Spawn/Kill/Detach/Attach/Rename/Input/Resize + Scrollback Protocol

## Narrative

As a TUI client, I want to send lifecycle commands (`SpawnSession`, `KillSession`,
`KeyInput`, `ResizePane`, `DetachSession`, `RenameSession`, `AttachSession`) over the
existing UDS IPC channel and receive structured responses or streamed scrollback chunks
(`ScrollbackChunk`, `ScrollbackDumpComplete`, `PtyReset`) — so that a single IPC
connection supports the full session lifecycle from spawn through detach/reattach without
requiring reconnect or protocol renegotiation.

## Acceptance Criteria

### AC-SH-001 (traces to BC-2.05.011 §ScrollbackChunk producer — session-host PTY read loop with backpressure)

The session-host binary implements a `tokio::task::spawn_blocking` PTY reader thread:
- `pty_master.try_clone_reader()` returns `Box<dyn Read + Send>` for blocking PTY reads.
- Read buffer: 4096 bytes.
- Each read posts `Bytes` into a bounded `mpsc::channel::<Bytes>(1024)` via
  `Handle::block_on(tx.send(bytes))` — NOT `try_send`. The blocking thread blocks on the
  channel if full (natural backpressure from TUI render speed → broker → proxy task).
- If `tx.send()` fails (receiver dropped, OOM-level condition): the reader thread logs WARN
  and exits; no silent drop.
- `pty_writer = pty_master.take_writer()` is called ONCE at startup. `take_writer()` is
  not idempotent; calling it again panics. The writer is held for the session lifetime.

### AC-SH-002 (traces to BC-2.05.011 I3-003 / ADR-0010 — Phase A and Phase B select! expansion with PTY arm)

The event loop is restructured as a real `tokio::select!` in BOTH phases:

**Phase A** (Detached — waiting for daemon connection) MUST use `tokio::select!` over:
- `pty_reader_rx.recv()` → `parser.process(&bytes)`; do NOT forward (no active connection).
- `listener.accept()` → SO_PEERCRED check → transition to Phase B.
- `child_exit_watch.recv()` → harness child exited while detached → session-host exits.

**Phase B** (Active — one daemon connection) MUST use `tokio::select!` over:
- `pty_reader_rx.recv()` → `parser.process(&bytes)` + `send_host_msg(HostToDaemon::PtyBytes { bytes })`. On send failure: WARN + break `PhaseBExit::Detach`.
- `recv_daemon_msg()` → full message dispatch (Kill, Detach, Attach, KeyInput, Resize, EOF).
- `child_exit_watch.recv()` → `StateChanged{Terminated}` + `Goodbye` + return.

The sequential `stream.read_exact()` Phase B loop from S-034 is replaced by this select!. `recv_daemon_msg()` is an async function wrapping `stream.read_exact()` that returns `Result<DaemonToHost, _>`.

### AC-SH-003 (traces to BC-2.05.010 KeyInput postcondition 1 / Ruling M — session-host writes KeyInput bytes to PTY stdin)

On `DaemonToHost::KeyInput { bytes }` in Phase B:
- Calls `pty_writer.write_all(&bytes).await`.
- On write success: continue Phase B (no response to daemon — KeyInput is fire-and-forget at this layer).
- On write failure: log WARN with error; break `PhaseBExit::Detach`. The daemon proxy task sees EOF on the control connection reader, calls the existing `transition_to_detached_or_terminated` path, and the daemon IPC layer maps the resulting `SessionHostDead` error to `"attach_failed"` wire code in the KeyInput response. No new error codes introduced.

### AC-SH-004 (traces to BC-2.05.011 §Screen-state transfer / Ruling M — session-host real scrollback dump replaces empty stub)

On `DaemonToHost::Attach`, the session-host MUST derive a real scrollback dump from the live `vt100::Parser` state (replacing the 0-chunk empty stub from S-035):

1. **Snapshot:** Capture `let screen = parser.screen()` at the instant Attach is received.
2. **No pause (I3-003):** PtyBytes forwarding continues uninterrupted in the `select!` while the dump is prepared and sent. The dump is synchronous within the Attach arm (the select! does not re-enter until the arm's async fn completes — this is acceptable as long as the dump is bounded; ≤200 rows/chunk + JSON serialization completes in <1ms per chunk on any reasonable hardware).
3. **Serialize rows:** For each row of scrollback + visible screen (scrollback rows first):
   - `screen.cell(row, col)` → `Option<&vt100::Cell>`.
   - `ch`: `cell.contents()` → `String`.
   - `fg`/`bg`: `vt100::Color::Default` → `SerializedColor::Default`; `vt100::Color::Idx(n)` → `SerializedColor::Ansi(n)`; `vt100::Color::Rgb(r,g,b)` → `SerializedColor::Rgb(r,g,b)`.
   - `attrs`: u8 bitmask — bit0=bold, bit1=dim, bit2=italic, bit3=underline, bit4=inverse (5-bit; `vt100::Attrs` verified for vt100 0.16).
   - `None` cells: `SerializedCell { ch: " ".to_string(), fg: Default, bg: Default, attrs: 0 }`.
4. **Chunk:** ≤200 rows per `HostToDaemon::ScrollbackChunk { rows, chunk_seq }`. `chunk_seq` starts at 0, monotonically increments.
5. **Complete:** Send `HostToDaemon::ScrollbackDumpComplete { total_chunks, cursor_row, cursor_col, pty_rows, pty_cols }` where cursor position comes from `screen.cursor_position()` and dimensions from `parser` state.
6. **On send failure during dump:** WARN log; break `PhaseBExit::Detach`.

`SerializedCell` and `SerializedColor` are already defined in `monocle-ipc/src/types.rs`. No new IPC types added.

### AC-SH-005 (traces to BC-2.05.011 §ScrollbackDumpComplete / Ruling M — daemon broadcasts scrollback dump to attaching TUI client)

After `attach_session()` returns `Ok(())` and the daemon holds `scrollback_chunks: Vec<HostToDaemon>` and `ScrollbackDumpComplete` fields, the daemon MUST broadcast the scrollback dump to the requesting TUI client (the TODO at `session_manager/mod.rs:~3899`):

1. For each chunk (in `chunk_seq` order): send `ServerToClient::ScrollbackChunk { session_id, rows, chunk_seq }` to the attaching client's sender only (NOT `broadcast_to_subscribers` — only the client that sent `ClientToServer::AttachSession`).
2. Send `ServerToClient::ScrollbackDumpComplete { session_id, total_chunks, cursor_row, cursor_col, pty_rows, pty_cols }` to the same client.
3. `pending_pty_bytes: VecDeque<Bytes>` (per `(session_id, client_id)` pair) accumulates live `ServerToClient::PtyOutput` messages arriving via the broker DURING this forwarding sequence. After `ScrollbackDumpComplete` is sent, flush `pending_pty_bytes` as live `PtyOutput` messages to the same client.

The proxy task (already running via S-046) continues handling live `PtyBytes → broker` in parallel. The `pending_pty_bytes` buffer prevents live PTY bytes from reaching the TUI before the screen snapshot, while ensuring they are not dropped.

### AC-001 (traces to BC-2.05.010 postcondition 1 — SpawnSession carries SpawnOptions, daemon builds SpawnRecipe)

`ClientToServer::SpawnSession { opts: SpawnOptions }` is the wire message for spawn
requests. The daemon receives `opts` and calls `engine_module.spawn_recipe(&opts)?` to
build `SpawnRecipe` internally (Model A — I27-001). The TUI NEVER sends a `SpawnRecipe`
directly. SpawnOptions includes `{ project_root, worktree_root, session_id, ccr_base_url, ... }`.

### AC-002 (traces to BC-2.05.010 KillSession postcondition 2 — daemon calls kill_session(); idempotent on Terminating and Terminated)

`ClientToServer::KillSession { session_id: String }` causes the daemon to send `SIGTERM` to the
session's PTY process. `session_id` is a `String` (UUID-as-String) on the wire.
If the process is in `Launching` state, the kill is ALLOWED per
BC-2.06.025 lifecycle rules (BC-2.08.003 Invariant 3). If the process is in `Terminating`
or `Terminated` state, `kill_session()` returns `Ok(())` (idempotent — kill is already
in-flight or complete; no duplicate Kill sent) per BC-2.08.003 Invariant 2 and S-034 AC-007.
The IPC handler MUST NOT return `"session_not_ready"` for a `Terminating` kill — that code
is reserved for `DetachSession` on a `Launching` session (F-P50-001).
No `ServerToClient::Error` is emitted for `KillSession` on a `Terminating` session — the
IPC handler returns silently after the idempotent `Ok(())` from `kill_session()`.

### AC-003 (traces to BC-2.05.010 KeyInput postcondition 1 — KeyInput routing via SessionManager → DaemonToHost)

`ClientToServer::KeyInput { session_id: String, bytes: Vec<u8> }` — the daemon IPC handler
calls `session_manager.send_key_input(&session_id, bytes)`, which sends
`DaemonToHost::KeyInput { bytes }` to the session-host over the per-session UDS control
connection. The session-host owns the PTY master fd and writes `bytes` to it. The daemon
NEVER holds the PTY master fd directly (PTY fd ownership is in the session-host process per
SS-session-manager.md §monocle-session-host binary and ADR-0009/ADR-0010).
`session_id` is a `String` (UUID-as-String) on the wire. On failure (session not found or
host dead), returns `ServerToClient::Error { code: "session_not_found" }` or `"attach_failed"`
as appropriate per the 12-code taxonomy — no `"pty_write_failed"` code.

NOTE: `ClientToServer::ResizePane` routing, `session_manager.resize_session()` implementation,
zero-dim clamp, and `DaemonToHost::Resize` forwarding are owned by **S-042** (human ruling
2026-06-21). S-047 does NOT implement `resize_session()` or the ResizePane IPC arm.

### AC-004 (traces to BC-2.05.010 DetachSession postcondition 1 — DetachSession received; blocks on Launching, defensive path only)

`ClientToServer::DetachSession { session_id }` — daemon stops forwarding PTY output to
the requesting client but does NOT stop the session. If the session is in `Launching`
state, return `ServerToClient::Error { code: "session_not_ready", ... }` (defensive path
— the TUI should not send DetachSession during Launching per BC-2.06.025; this error is
a guard, not a primary flow).

### AC-005 (traces to BC-2.05.010 RenameSession postcondition 1 — RenameSession received; daemon calls rename_session(); updates display name)

`ClientToServer::RenameSession { session_id, new_name: String }` — daemon calls
`session_manager.rename_session(&session_id, new_name)`, which updates
`session.display_name` in the registry and publishes `SessionListUpdate` fan-out to all
clients. The field is `new_name` (canonical per SS-ipc.md §ClientToServer RenameSession
and BC-2.05.010 PC-4a). `name` MUST NOT be used — it is an incorrect alias.
Rename is ALLOWED when the session is in `Launching` or `Running` state.

### AC-006 (traces to BC-2.05.010 AttachSession postcondition 1 — AttachSession received; daemon calls attach_session(); triggers scrollback dump sequence)

`ClientToServer::AttachSession { session_id }` — daemon begins subscribing the requesting
client to the session's PTY output AND immediately initiates a scrollback dump sequence.
The dump sequence is:
1. Set `pending_pty_bytes = true` for this client (buffer live PTY bytes during dump).
2. Stream `ServerToClient::ScrollbackChunk { session_id, chunk_seq: u32, rows: Vec<Vec<SerializedCell>> }` packets (styled-cell rows model — NOT a byte stream).
3. Send `ServerToClient::ScrollbackDumpComplete { session_id, total_chunks: u32, cursor_row: u16, cursor_col: u16, pty_rows: u16, pty_cols: u16 }`.
4. Set `pending_pty_bytes = false`; flush buffered bytes as live PTY output.

### AC-007 (traces to BC-2.05.011 §ScrollbackChunk postcondition 3 — ScrollbackChunk contiguity)

Each `ScrollbackChunk` for a given session MUST have `chunk_seq` values that are
contiguous and start from 0. If the client receives a chunk where
`chunk_seq != expected_seq`, the client MUST:
- Discard all buffered chunks for that session.
- Send `ClientToServer::AttachSession` to restart the dump.
- NOT attempt to reconstruct from out-of-order chunks.

### AC-008 (traces to BC-2.05.011 §ScrollbackDumpComplete postcondition 3 — ScrollbackDumpComplete validates total_chunks)

When `ScrollbackDumpComplete` arrives:
- Client validates `count_of_received_chunks == total_chunks`.
- If mismatch: re-trigger AttachSession (restart dump).
- If match: reconstruct the full screen from concatenated chunk rows (`Vec<Vec<SerializedCell>>`), apply
  `cursor_row/cursor_col` and `pty_rows/pty_cols`, then replay buffered live PTY bytes.

### AC-009 (traces to BC-2.05.011 §PtyReset postcondition 3 — PtyReset clears buffer and re-triggers attach)

When `ServerToClient::PtyReset { session_id }` is received by the TUI:
- Clear all in-flight scrollback chunks for `session_id`.
- Clear the local PTY display buffer for `session_id`.
- Display a status bar message: "[PTY reset — <session_id_short>]" for 5 seconds.
- Re-trigger `ClientToServer::AttachSession { session_id }` automatically.

### AC-010 (traces to BC-2.05.011 invariant 6 — pending_pty_bytes buffer during dump)

During a scrollback dump (between AttachSession receipt and ScrollbackDumpComplete send):
- Live PTY bytes for this client are buffered in a `pending_pty_bytes: VecDeque<Bytes>`
  per session-client pair.
- The buffer is NOT bounded by the 64-item per-client send buffer (it is a separate
  accumulation buffer during dump).
- Once dump completes, buffered bytes are replayed as live `PtyOutput` messages.
- If the session exits during dump-in-progress, `PtyReset` is emitted instead of
  `ScrollbackDumpComplete`.

### AC-011 (traces to BC-2.05.010 invariant 6 — No-silent-failure invariant)

For all 7 `ClientToServer` variants (except ResizePane per AC-003 carve-out):
- Every failure produces a `ServerToClient::Error { code: <one of the canonical 12 wire codes>, message: String }`.
- The daemon NEVER silently ignores a message.
- Unknown `ClientToServer` variants (forward-compatibility): the `#[non_exhaustive]` wildcard
  arm maps them to `ServerToClient::Error { code: "invalid_request", message: "unknown variant" }` (the
  generic catch-all code per the 12-code taxonomy). The phantom code `"unknown_command"` MUST NOT be used.
- `KillSession` on `Terminating`/`Terminated`: idempotent `Ok(())` — NO `ServerToClient::Error` emitted
  (idempotent success is NOT a failure). This is consistent with BC-2.08.003 Invariant 2.

### AC-012 (traces to BC-2.05.010 invariant 6 / BC-2.05.010 Architecture Source §ServerToClient::Error — canonical 12 wire error codes are the CLOSED set for Phase 1)

The complete and closed set of wire error codes for `ServerToClient::Error.code` is the
canonical 12-code taxonomy defined in SS-ipc.md §ServerToClient::Error v1A taxonomy
(§402-§419). Every code used in this story MUST come from this closed list:

| code | Trigger |
|------|---------|
| `"binary_not_found"` | `EngineError::BinaryNotFound` (harness binary not on PATH) |
| `"invalid_spawn_arg"` | `EngineError::InvalidPath` (structurally invalid spawn arg) |
| `"spawn_unsupported"` | `EngineError::UnsupportedOperation` (harness does not support spawning) |
| `"spawn_failed"` | OS process spawn failure |
| `"sidecar_write_failed"` | Sidecar write failed post-spawn |
| `"session_id_collision"` | UUID v4 collision in registry |
| `"session_not_found"` | Session ID not in registry |
| `"attach_failed"` | `SessionError::SessionHostDead` on the attach-path |
| `"kill_failed"` | `SessionError::SessionHostDead` on the kill-path |
| `"rename_failed"` | `SessionManager::rename_session()` returned error |
| `"session_not_ready"` | `SessionError::SessionNotReady` — DetachSession on Launching session (host_conn: None) |
| `"invalid_request"` | Catch-all: `SessionError::Io`, unrecognized `ClientToServer` variants |

No code outside this list SHALL be returned in Phase 1. The following phantom codes from
prior drafts are REMOVED and MUST NOT appear in any implementation, test, or commentary:
`"pty_write_failed"`, `"rename_rejected"`, `"kill_rejected"`, `"unknown_command"`,
`"internal_error"`, `"permission_denied"`, `"protocol_error"`. Any future code additions
require an explicit SS-ipc.md update first (the taxonomy is a closed set for Phase 1).

**KeyInput failure note**: `KeyInput` failures map to `"session_not_found"` (session ID absent
from registry) or `"attach_failed"` (session host dead) — NOT a phantom `"pty_write_failed"` code.
`"pty_write_failed"` is NOT in the 12-code taxonomy and MUST NOT be introduced.

## Tasks

### Session-Host Producer Legs (monocle-session-host) — NEW (human ruling 2026-06-22)

- [ ] Obtain `pty_writer` via `pty_master.take_writer()` once at startup step 5 (after PTY open).
      Hold as `pty_writer: Box<dyn Write + Send>` for the session lifetime. MUST call exactly once.
- [ ] Spawn PTY reader `tokio::task::spawn_blocking` thread: `pty_reader = pty_master.try_clone_reader()`;
      read buffer 4096 bytes; loop `read()` → `Handle::block_on(tx.send(Bytes::copy_from_slice(&buf[..n])))`.
      Channel: `mpsc::channel::<Bytes>(1024)`. On `tx.send()` error: log WARN; break reader thread.
      On channel receiver drop (event loop exited abnormally): thread exits silently.
- [ ] Expand Phase A `listener.accept().await` → `tokio::select!` with three arms:
      `pty_reader_rx.recv()` (parse bytes, no forward), `listener.accept()` (SO_PEERCRED, Phase B),
      `child_exit_watch.recv()` (exit session-host). See AC-SH-002.
- [ ] Expand Phase B sequential `read_exact()` loop → `tokio::select!` with three arms:
      `pty_reader_rx.recv()` (parse + `HostToDaemon::PtyBytes` forward via `send_host_msg()`),
      `recv_daemon_msg()` (full dispatch including KeyInput arm), `child_exit_watch.recv()`
      (`StateChanged{Terminated}` + `Goodbye` + return). See AC-SH-002.
- [ ] Add `DaemonToHost::KeyInput { bytes }` arm to Phase B dispatch:
      `pty_writer.write_all(&bytes).await` → on error: WARN + `break PhaseBExit::Detach`. See AC-SH-003.
- [ ] Implement `stream_scrollback_dump_chunked(parser, conn_writer)` async fn:
      snapshot `parser.screen()` → serialize rows (scrollback + visible) to `Vec<Vec<SerializedCell>>`
      using `SerializedCell`/`SerializedColor` from `monocle-ipc` (no new types). Chunk at ≤200
      rows per `ScrollbackChunk`. Stream `chunk_seq` 0..N. Send `ScrollbackDumpComplete` with
      cursor + dimensions from screen. On any `send_host_msg()` error: WARN + return error. See AC-SH-004.
- [ ] Replace the empty 0-chunk stub in the `DaemonToHost::Attach` arm with a call to
      `stream_scrollback_dump_chunked()`. Verify the select! arm handles failure (Detach break). See AC-SH-004.

### IPC Protocol (monocle-ipc)
- [ ] Verify and/or add 7 `ClientToServer` variants to `crates/monocle-ipc/src/lib.rs`
      (canonical file per S-033/S-039/S-044; NOT `proto.rs`):
      `SpawnSession { opts: SpawnOptions }`,
      `KillSession { session_id: String }`,
      `KeyInput { session_id: String, bytes: Vec<u8> }`,
      `ResizePane { session_id: String, rows: u16, cols: u16 }`,
      `DetachSession { session_id: String }`,
      `RenameSession { session_id: String, new_name: String }`,
      `AttachSession { session_id: String }`.
      All `session_id` fields are `String` (UUID-as-String per SS-session-manager.md §session_id type ruling).
      `ClientId` and `SessionId` newtypes MUST NOT appear on the wire — daemon-internal only.
- [ ] Verify and/or add 2 `ServerToClient` variants to `crates/monocle-ipc/src/lib.rs`
      (note: `PtyReset` was added in S-046; add `ScrollbackChunk` and `ScrollbackDumpComplete` here):
      `ScrollbackChunk { session_id: String, chunk_seq: u32, rows: Vec<Vec<SerializedCell>> }`,
      `ScrollbackDumpComplete { session_id: String, total_chunks: u32, cursor_row: u16, cursor_col: u16, pty_rows: u16, pty_cols: u16 }`.
- [ ] Verify `SpawnOptions` struct in `crates/monocle-ipc/src/lib.rs` (if not already present from S-033/S-045).
      All fields must use `String` for path-like fields that cross the wire, NOT typed newtypes:
      `project_root: String`, `worktree_root: String`, `session_id: String`,
      `ccr_base_url: Option<String>`, `display_name: Option<String>`.
      (`session_id` is filled by the daemon IPC handler via `opts.with_daemon_fields()`.)
- [ ] Update error code taxonomy comment in the IPC module to document the canonical 12 codes
      (matching the closed set in SS-ipc.md §ServerToClient::Error). Remove any phantom codes
      from previous drafts (`"unknown_command"`, `"pty_write_failed"`, `"rename_rejected"`,
      `"kill_rejected"`, `"permission_denied"`, `"protocol_error"`) if they appear.

### Daemon Routing (monocle-runtime)
- [ ] Add match arms for `KeyInput` and `RenameSession` in the daemon's IPC dispatch
      loop (`monocle-runtime/src/ipc_server.rs`). These are the 2 arms owned by S-047.
      The `SpawnSession`, `KillSession`, `AttachSession`, `DetachSession`, and `ResizePane` arms
      are authored by S-033, S-034, S-035, and S-042 respectively — S-047 MUST NOT re-add or
      duplicate those arms.
      - `KeyInput` → `session_manager.send_key_input(id, bytes)` — on `Err(SessionNotFound)` return
        `ServerToClient::Error { code: "session_not_found" }`; on `Err(SessionHostDead)` return
        `ServerToClient::Error { code: "attach_failed" }`. Do NOT use `"pty_write_failed"` — it is
        NOT in the canonical 12-code taxonomy (AC-012).
      - NOTE: `ResizePane` arm and `resize_session()` are S-042 scope (human ruling 2026-06-21).
        When S-047 is dispatched, both will already be implemented by S-042.
      - `RenameSession` → call `session_manager.rename_session(id, new_name)` → update display_name,
        fan-out `SessionListUpdate`.
- [ ] Implement daemon scrollback forwarding in the `AttachSession` handler
      (resolves TODO at `session_manager/mod.rs:~3899`): after `attach_session()` returns
      `Ok(())` with `scrollback_chunks` and `ScrollbackDumpComplete` fields:
      (a) For each chunk: send `ServerToClient::ScrollbackChunk { session_id, rows, chunk_seq }`
          directly to the requesting client's channel (NOT `broadcast_to_subscribers`).
      (b) Send `ServerToClient::ScrollbackDumpComplete { session_id, total_chunks, cursor_row,
          cursor_col, pty_rows, pty_cols }` to the same client.
      (c) Flush `pending_pty_bytes` for this `(session_id, client_id)` pair as live `PtyOutput`
          messages to the same client. See AC-SH-005.
- [ ] Add `pending_pty_bytes: HashMap<(String, String), VecDeque<Bytes>>` to `DaemonState`
      (or per-session state struct) for in-flight scrollback buffering. Keys are
      `(session_id: String, client_id: String)` — daemon-internal representation, consistent
      with how the broker tracks per-client channels by string key.
      Live `PtyOutput` from the broker is buffered into this map while `AttachSession` scrollback
      forwarding is in progress, then flushed after `ScrollbackDumpComplete` is sent.
- [ ] Remove/repurpose `monocle-runtime/src/scrollback.rs` CREATE task: the scrollback dump
      is produced by the session-host (not a daemon-side ring buffer). The daemon-side
      scrollback module (if needed) handles the forwarding logic above, not content production.
      If `scrollback.rs` was already created by a prior story pass, replace its content with
      the forwarding helper (or inline the forwarding into `session_manager/mod.rs`).

### TUI Handling (monocle-tui)
- [ ] Add client-side handler for `ScrollbackChunk` in `monocle-tui/src/ipc_receiver.rs`:
      - Buffer chunks by session in `HashMap<SessionId, Vec<ScrollbackChunk>>`.
      - Validate contiguity (AC-007).
      - On gap detected: clear and re-trigger `AttachSession`.
- [ ] Add client-side handler for `ScrollbackDumpComplete`:
      - Validate `total_chunks` (AC-008).
      - Reconstruct screen from chunks; apply cursor position; replay pending_pty_bytes.
- [ ] Add client-side handler for `PtyReset` (extends S-046's emit):
      - Clear scrollback buffer; display status bar "[PTY reset — <id_short>]" for 5 seconds.
      - Re-trigger `AttachSession`.

### Tests
- [ ] Write session-host unit tests in `crates/monocle-session-host/src/main.rs` or a
      `tests/session_host_*.rs` integration test file:
      - `test_AC_SH_001_pty_reader_task_sends_bytes_to_channel` (AC-SH-001 — spawn reader, write to PTY slave, assert bytes arrive in channel)
      - `test_AC_SH_002_phase_a_select_feeds_parser_and_accepts` (AC-SH-002 — Phase A select! feeds vt100 parser AND accepts daemon connection)
      - `test_AC_SH_002_phase_b_select_forwards_pty_bytes_as_PtyBytes` (AC-SH-002 — Phase B select! produces HostToDaemon::PtyBytes)
      - `test_AC_SH_003_key_input_writes_to_pty_stdin` (AC-SH-003 — DaemonToHost::KeyInput → pty_writer.write_all)
      - `test_AC_SH_003_key_input_write_failure_detaches` (AC-SH-003 — write failure → PhaseBExit::Detach)
      - `test_AC_SH_004_attach_produces_real_scrollback_from_parser` (AC-SH-004 — DaemonToHost::Attach sends ScrollbackChunk* then ScrollbackDumpComplete with real cell content)
      - `test_AC_SH_004_attach_chunk_seq_is_contiguous` (AC-SH-004 — chunk_seq values are 0,1,2,...)
      - `test_AC_SH_005_daemon_forwards_scrollback_to_attaching_client` (AC-SH-005 — daemon broadcasts ScrollbackChunk/Complete to attaching client only)
      - `test_AC_SH_005_pending_pty_bytes_flushed_after_dump_complete` (AC-SH-005 — live PtyOutput buffered during dump, flushed after DumpComplete)
- [ ] Write integration tests in `monocle-runtime/tests/ipc_lifecycle.rs`:
      - `test_BC_2_05_010_spawn_session_carries_spawn_options_not_recipe` (AC-001)
      - `test_BC_2_05_010_kill_session_allowed_launching_rejected_terminating` (AC-002)
      - `test_BC_2_05_010_key_input_routes_to_session_host` (AC-003 — KeyInput routing; ResizePane tests are in S-042 scope)
      - `test_BC_2_05_010_detach_session_blocks_on_launching` (AC-004)
      - `test_BC_2_05_010_rename_session_propagates_list_update` (AC-005)
      - `test_BC_2_05_010_attach_session_triggers_scrollback_sequence` (AC-006)
      - `test_BC_2_05_011_scrollback_chunk_contiguity_gap_triggers_reattach` (AC-007)
      - `test_BC_2_05_011_scrollback_dump_complete_validates_total_chunks` (AC-008)
      - `test_BC_2_05_011_pty_reset_clears_buffer_retriggers_attach` (AC-009)
      - `test_BC_2_05_011_pending_pty_bytes_buffered_during_dump` (AC-010)
      - `test_BC_2_05_010_no_silent_failure_all_variants` (AC-011 — parametrized)
      - `test_BC_2_05_010_error_codes_exhaustive_taxonomy` (AC-012 — compile-time exhaustive match)

## Previous Story Intelligence

S-021 established `ClientToServer`/`ServerToClient` enums with phase-1 variants. S-022 added
`InitialState` push on connect. S-023 added reconnect backoff. S-033 established `spawn_session()`
in the daemon. S-034/S-035 implemented session enrichment and status machine. S-046 added
`PtyBroker` and the `PtyReset` server-to-client variant.

This story extends the phase-1 IPC protocol with 7 new client variants and 2 new server variants
(PtyReset is already added in S-046). The extension is ADDITIVE — existing phase-1 variants are
not modified. The `SpawnOptions` struct should be verified against S-033/S-045 before adding; if
it already exists in `crates/monocle-ipc/src/lib.rs`, skip the create step and confirm field parity.

Key lesson from S-033 review: `session_manager/mod.rs` gained both the spawn path and error mapping
in S-033. The IPC dispatch loop in S-047 calls into session_manager methods — it must NOT duplicate
session state management directly in the handler.

## Architecture Compliance Rules

From `architecture/SS-session-manager.md §Ruling M`:
- **Two-parser architecture is canonical.** The session-host owns ONE parser (source of scrollback dump and live render state). The TUI owns ONE parser per session (renderer, fed by `PtyOutput` from the daemon broker). No runtime coupling between them.
- **`pty_master.take_writer()` is called ONCE at startup.** portable-pty panics if called again. The writer is held for the session lifetime, not acquired per-message.
- **PTY reader thread uses `.send().await` (block_on variant) — NOT `.try_send()`.** Silent PTY byte drops corrupt the vt100 parser state machine (mid-CSI sequence). `try_send` failure must send `PtyReset`; but the backpressure design makes this unreachable in normal operation.
- **`stream_scrollback_dump_chunked()` is synchronous within the Phase B select! arm.** The select! arm completes before re-entering the loop. This is correct: the dump is bounded (≤200 rows/chunk) and latency is negligible. Do NOT `tokio::spawn` the dump or split it across multiple select! iterations.
- **Scrollback dump does NOT pause PtyBytes (I3-003).** The `select!` continues delivering `pty_reader_rx.recv()` bytes to `parser.process()` and `send_host_msg(PtyBytes)` AFTER `DaemonToHost::Attach` is received and while the dump is streaming. The PtyBytes and the dump output interleave on the wire — the daemon handles them in order.
- **SerializedCell/SerializedColor: import from monocle-ipc, do NOT redefine.** `use monocle_ipc::types::{SerializedCell, SerializedColor};`
- **Chunking limit: ≤200 rows per ScrollbackChunk.** This keeps each frame comfortably under the 256 KiB MAX_FRAME_LEN with JSON overhead.

From `architecture/SS-ipc.md`:
- Model A (I27-001): SpawnSession carries `SpawnOptions`; daemon builds `SpawnRecipe` internally.
  Model B (TUI sends SpawnRecipe) is REJECTED by architecture decision. Do not implement Model B.
- **All wire `session_id` fields are `String`** (UUID-as-String). `ClientId` and `SessionId`
  newtypes are daemon-INTERNAL ONLY and MUST NOT appear on the wire. `SpawnOptions.session_id`
  is `String`. This is the canonical §session_id type ruling in SS-session-manager.md §session_id type ruling.
- 256 KiB maximum IPC frame size still applies to ScrollbackChunk (chunk rows to respect
  the 256 KiB total frame limit with framing overhead).
- `pending_pty_bytes` is a daemon-side struct, NOT a TUI-side struct. The TUI has a parallel
  accumulation buffer for chunks received during dump.
- ResizePane zero-dim clamp MUST happen before sending `DaemonToHost::Resize` — never send 0 rows or 0 cols to the session-host. The session-host owns the PTY fd and calls ioctl; the daemon sends the clamped values only.
- **Canonical 12 wire codes ONLY** (closed set for Phase 1): `binary_not_found`, `invalid_spawn_arg`,
  `spawn_unsupported`, `spawn_failed`, `sidecar_write_failed`, `session_id_collision`,
  `session_not_found`, `attach_failed`, `kill_failed`, `rename_failed`, `session_not_ready`,
  `invalid_request`. No other code may be introduced without a SS-ipc.md update.

From `architecture/SS-conventions-anti-patterns.md v1.32.6`:
- All new error types: `thiserror ^2` (NOT thiserror 1.x).
- No `println!` in production IPC handler paths — use `tracing::warn!/error!`.

**Forbidden dependencies**: `monocle-ipc` MUST NOT import from `monocle-runtime` or `monocle-tui`.
`monocle-ipc` defines wire types only. Routing logic is in `monocle-runtime`. TUI display logic
is in `monocle-tui`. These boundaries are enforced by the workspace dependency graph.

## Library and Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| `tokio` | `=1.52.0` (EXACT) | Async dispatch, `mpsc::channel`, `spawn` |
| `serde` | `^1` (features = ["derive"]) | `SpawnOptions`, all new IPC variants |
| `serde_json` | `=1.0.149` (EXACT) | JSON framing of new variants |
| `bytes` | `^1.11` | `pending_pty_bytes: VecDeque<Bytes>` buffering during scrollback dump (AC-010) |
| `thiserror` | `^2` | Error types in monocle-runtime routing layer |
| `tracing` | `^0.1` | warn!/error! in IPC handler and scrollback task |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/monocle-session-host/src/main.rs` | MODIFY | PTY reader task (spawn_blocking + mpsc::channel::<Bytes>(1024) + 4096-byte read buffer); `pty_writer = pty_master.take_writer()` at startup; Phase A `select!` (pty_reader_rx + listener.accept + child_exit_watch); Phase B `select!` (pty_reader_rx→PtyBytes + recv_daemon_msg→full dispatch + child_exit_watch); `DaemonToHost::KeyInput` arm → `pty_writer.write_all()`; `stream_scrollback_dump_chunked()` replacing empty stub in `DaemonToHost::Attach` arm |
| `crates/monocle-ipc/src/lib.rs` | MODIFY | Add 7 new `ClientToServer` variants; add `ScrollbackChunk`, `ScrollbackDumpComplete` to `ServerToClient`; verify `SpawnOptions` struct (all wire fields use `String` — no typed newtypes) |
| `crates/monocle-runtime/src/ipc_server.rs` | MODIFY | Add match arms for `KeyInput` and `RenameSession` (2 arms owned by S-047); `SpawnSession`/`KillSession`/`AttachSession`/`DetachSession`/`ResizePane` arms are authored by S-033/S-034/S-035/S-042 respectively — do NOT duplicate; add scrollback forwarding to `AttachSession` arm (resolves mod.rs:~3899 TODO) |
| `crates/monocle-runtime/src/session_manager/mod.rs` | MODIFY | Add `write_pty_bytes()` (called by KeyInput arm) and `rename_session()` methods; resolve TODO at ~line 3899: forward `scrollback_chunks` as `ServerToClient::ScrollbackChunk*` + `ScrollbackDumpComplete` to attaching client; add `pending_pty_bytes` accumulation and flush. NOTE: `resize_session()` is S-042 scope; `kill_session()` is S-034 scope. |
| `crates/monocle-runtime/src/scrollback.rs` | CREATE or SKIP | If scrollback forwarding logic is inlined in `session_manager/mod.rs`, this module is not needed. If a separate forwarding helper is useful, create with forwarding logic only (not a content producer — content comes from session-host). |
| `crates/monocle-runtime/src/lib.rs` | MODIFY | Add `pub mod scrollback;` only if scrollback.rs is created |
| `crates/monocle-tui/src/ipc_receiver.rs` | MODIFY | Add handlers for ScrollbackChunk, ScrollbackDumpComplete, PtyReset (TUI-side) |
| `crates/monocle-runtime/tests/ipc_lifecycle.rs` | CREATE | Integration tests for daemon ACs |
| `crates/monocle-session-host/tests/session_host_pty.rs` | CREATE | Session-host unit/integration tests for AC-SH-001..AC-SH-005 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-300 | `AttachSession` for a session in `Terminating` state | `ServerToClient::Error { code: "attach_failed" }` — session is being killed; `SessionHostDead` is the nearest existing error on the attach path (see Terminated-in-grace disposition: `attach_session()` on Terminating → `SessionHostDead` → `"attach_failed"`) |
| EC-301 | `KillSession` for a session in `Terminating` state | `kill_session()` returns `Ok(())` — idempotent (kill is in-flight; watchdog already running; no duplicate Kill sent) per BC-2.08.003 Invariant 2 and S-034 AC-007. IPC handler does NOT emit any `ServerToClient::Error`. `"session_not_ready"` MUST NOT be returned for kill on `Terminating` — that code is reserved exclusively for `DetachSession` on `Launching` (F-P50-001). |
| EC-301b | `KillSession` for unknown `session_id` | `ServerToClient::Error { code: "session_not_found" }` |
| EC-302 | `ResizePane { rows: 0, cols: 0 }` | Clamp to `{ rows: 1, cols: 1 }`; send `DaemonToHost::Resize { rows: 1, cols: 1 }` to session-host; WARN log; NO Error response to TUI |
| EC-303 | `ResizePane { rows: 0, cols: 80 }` (only rows is 0) — BC-2.05.010 EC-303 | Clamp rows to 1; cols stays 80; send `DaemonToHost::Resize { rows: 1, cols: 80 }` to session-host |
<!-- EC-303 disambiguation: This BC-2.05.010 EC-303 (ResizePane partial-zero clamp) is distinct from
     the "EC-303" identifier used in SS-ipc.md §SpawnAck to refer to wizard session_id filtering
     in BC-2.09.008 (S-044). Both are EC-303 within their respective BCs (BC-namespaced identifiers).
     There is NO collision — they are different BCs. This note prevents cross-story confusion. -->
| EC-304 | `KeyInput` arrives while session is in `Terminating` state | `ServerToClient::Error { code: "attach_failed" }` — session host is being killed; `SessionHostDead` maps to `"attach_failed"` on the key-input path (see SS-session-manager.md Terminated-in-grace matrix — `send_key_input()` on Terminating → `SessionHostDead` → `"attach_failed"`). `"session_not_ready"` is reserved for `DetachSession` on `Launching`; MUST NOT be used here. |
| EC-305 | `ScrollbackChunk` arrives with `chunk_seq` gap (e.g., 0, 1, 3, missing 2) | Client discards all received chunks for that session; re-triggers `AttachSession` |
| EC-306 | `ScrollbackDumpComplete.total_chunks` is 0 (empty session, no output yet) | Valid — client reconstructs empty screen with cursor at (0,0); replay zero pending bytes |
| EC-307 | Session exits mid-scrollback dump | Daemon emits `PtyReset` instead of `ScrollbackDumpComplete`; TUI clears and re-attaches |
| EC-308 | Two clients simultaneously request `AttachSession` for same session | Each gets an independent scrollback dump sequence; `pending_pty_bytes` is per session-client pair (not shared) |
| EC-309 | Client sends unknown `ClientToServer` variant (forward compat) | `ServerToClient::Error { code: "invalid_request", message: "unknown variant" }` — no panic, no silent drop. `"unknown_command"` MUST NOT be used (it is not in the canonical 12-code taxonomy). |

## Token Budget Estimate

| Category | Estimate |
|----------|----------|
| Story spec (this file) | ~9 000 tokens |
| BC files (2 BCs: BC-2.05.010 + BC-2.05.011) | ~8 000 tokens |
| Architecture sections (SS-ipc, SS-session-manager, SS-conventions) | ~4 000 tokens |
| Existing code context (monocle-ipc/src/lib.rs, ipc_server.rs, session_manager/mod.rs, session-host/src/main.rs, ipc_receiver.rs) | ~8 000 tokens |
| Test files to write | ~7 000 tokens |
| **Total estimated** | **~36 000 tokens** |

Exceeds comfortable single-pass; recommend splitting delivery into two implementation passes:
- Pass 1 (session-host producer legs): AC-SH-001 through AC-SH-004 in `monocle-session-host/src/main.rs`.
- Pass 2 (daemon + TUI): AC-SH-005 + AC-001 through AC-012 in `monocle-runtime` + `monocle-tui`.
Both passes use the same story file; no story split required.

## Dependency Justification

- S-047 depends on S-021 because the `ClientToServer`/`ServerToClient` enums and `SessionId`/`ClientId`
  types are defined in S-021; this story extends those enums.
- S-047 depends on S-022 because `InitialState` push established the connect protocol that
  `AttachSession` extends.
- S-047 depends on S-023 because reconnect backoff (S-023) must compose with the new scrollback
  re-attach on PtyReset — both produce AttachSession from the TUI perspective.
- S-047 depends on S-033 because `spawn_session()` and `DaemonState` are S-033's output;
  `SpawnSession` routing calls into `spawn_session()`.
- S-047 depends on S-034/S-035 because session state machine transitions (Launching, Running,
  Terminating) are validated in lifecycle variant handlers (AC-002, AC-004).
- S-047 depends on S-046 because `PtyReset` server-to-client variant was added in S-046 and
  the scrollback dump references it when a session exits during dump.
- S-047 blocks S-048 because the sessions panel (S-048) uses `KillSession`, `DetachSession`,
  `RenameSession` dispatch — these are the wire commands implemented in S-047.

## IPC Handler Arm Ownership Disambiguation

S-047 authors the **`ClientToServer::KeyInput`** and **`ClientToServer::RenameSession`**
arms in `monocle-runtime/src/ipc_server.rs`.
The canonical arm ownership split is:

| IPC Handler Arm | Owning Story |
|-----------------|-------------|
| `ClientToServer::SpawnSession` | S-033 |
| `ClientToServer::KillSession` | S-034 |
| `ClientToServer::AttachSession` | S-035 |
| `ClientToServer::DetachSession` | S-035 |
| `ClientToServer::KeyInput` | **S-047** (this story) |
| `ClientToServer::ResizePane` | **S-042** (human ruling 2026-06-21 — moved from S-047) |
| `ClientToServer::RenameSession` | **S-047** (this story) |

S-047 MUST NOT re-add or duplicate the SpawnSession, KillSession, AttachSession, DetachSession,
or ResizePane arms — those arms are authored by S-033, S-034, S-035, and S-042 respectively.
S-047 only adds the 2 arms above (KeyInput, RenameSession) and integrates the dispatch loop by
routing through the SessionManager methods established by S-033/S-034/S-035.

NOTE: `resize_session()` in `session_manager/mod.rs` is implemented by S-042. When S-047 is
dispatched, that stub will already be replaced with a real implementation.

## Subsystem Anchor Justification

SS-05 owns this story's scope because both BC-2.05.010 and BC-2.05.011 define IPC wire protocol
extensions — the client/server lifecycle message set — which is the core capability of SS-05
(monocle-ipc, daemon IPC layer) per ARCH-INDEX Subsystem Registry SS-05.

## Trace

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.8 | 2026-06-22 | vsdd-factory:architect | Human ruling 2026-06-22: S-047 expanded to full session-host producer scope per SS-session-manager.md Ruling M. Added AC-SH-001..AC-SH-005 (session-host PTY read loop, Phase A/B select! expansion, KeyInput write, real scrollback dump, daemon scrollback forwarding). Added session-host producer legs Tasks section. Updated File Structure with monocle-session-host/src/main.rs MODIFY and session_host_pty.rs CREATE rows. Updated daemon routing Tasks with scrollback forwarding obligation (resolves mod.rs:~3899 TODO). Added Architecture Compliance Rules for two-parser design, pty_writer lifecycle, chunking limits. Points bumped 8 → 16. Target subsystems updated SS-05 → [SS-05, SS-08]. Input pins refreshed: BC-2.05.011 → v1.2.12, SS-session-manager → v2.18.0. Token budget updated. |
| 1.7 | 2026-06-21 | vsdd-factory:architect | Human ruling 2026-06-21: ResizePane IPC arm and resize_session() moved from S-047 → S-042. AC-003 header updated to KeyInput-only; ResizePane NOTE added. IPC Handler Arm Ownership table: ResizePane row changed from S-047 → S-042 with ruling citation. Tasks: ResizePane arm removed from daemon routing task; 3 arms → 2 arms; test `test_BC_2_05_010_resize_pane_zero_clamp_warns_not_errors` renamed to `test_BC_2_05_010_key_input_routes_to_session_host` (resize tests are S-042 scope). File Structure: ipc_server.rs row updated (3 arms → 2 arms); session_manager/mod.rs NOTE added that resize_session() is S-042's. traces_to updated. IPC file reference corrected ipc_handler.rs → ipc_server.rs (canonical file used since S-040). |
| 1.6 | 2026-06-21 | vsdd-factory:architect | Architect ruling errata: File Structure `resize_pane()` → `resize_session()` (method name must match SS-session-manager.md §Public API; typo never caught in prior story-writer passes). AC-003 body was already correct. Input pins bumped: BC-2.05.010 v1.9.4→v1.9.8 (arch-source cascade, no behavioral change), SS-session-manager v2.6.1→v2.16.0 (Ruling A errata confirms resize daemon leg is S-047 scope). No AC changes. <!-- version-pin-historical: BC-2.05.010 v1.9.4 and SS-session-manager v2.6.1 are historical pre-cascade versions cited in this trace row only --> |
| 1.5 | 2026-06-16 | vsdd-factory:story-writer | S-047-AC009-PTYRESET-QUALIFIER: AC-009 trace header updated from "postcondition 3" to "§PtyReset postcondition 3" — adding the §PtyReset subsection qualifier to match BC-2.05.011's section structure (symmetric with AC-007 §ScrollbackChunk and AC-008 §ScrollbackDumpComplete corrections in v1.4). AC body unchanged. |
| 1.4 | 2026-06-16 | vsdd-factory:story-writer | F-P23-IMP-001: AC-007 header corrected from "postcondition 1" to "§ScrollbackChunk postcondition 3" (contiguity/gap→re-attach is §ScrollbackChunk PC-3, not PC-1); AC-008 header corrected from "postcondition 2" to "§ScrollbackDumpComplete postcondition 3" (total_chunks validation is §ScrollbackDumpComplete PC-3, not PC-2). Closes F-P20-CRIT-001 class for S-047: all AC-001..AC-012 headers now cite subsection-scoped real clauses. AC bodies unchanged. |
| 1.3 | 2026-06-16 | vsdd-factory:story-writer | Corpus-wide AC-trace-citation audit (F-P20-CRIT-001 class): re-anchored AC-002..AC-006 from flat global PC numbers (PC-5..PC-10) to subsection-scoped clauses (KillSession/KeyInput/ResizePane/DetachSession/RenameSession/AttachSession PC-1); AC-010 BC-2.05.011 invariant 1→6 (pending_pty_bytes); AC-011 invariant 4→6 (No-silent-failure); AC-012 invariant 5→invariant 6 / Architecture Source §ServerToClient::Error. AC bodies unchanged. |
| 1.2 | 2026-06-16 | vsdd-factory:story-writer | F-P19-SUG-001: Bump BC-2.05.011 input pin "1.2.4" → "1.2.5" (metadata-only Story-Anchor delta; no behavioral change). |
| 1.1 | 2026-06-16 | vsdd-factory:story-writer | Initial decomposition — F-P16-IMP-002 era; established 7 ClientToServer variants, SpawnOptions, ResizePane zero-dim clamp, scrollback protocol, and IPC handler arm ownership table. |
| 1.0 | 2026-06-15 | vsdd-factory:story-writer | Initial decomposition |
