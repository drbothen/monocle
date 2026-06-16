---
document_type: story
level: L4
story_id: S-047
epic_id: EPIC-05
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 8
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-021, S-022, S-023, S-033, S-034, S-035, S-046]
blocks: [S-048]
target_module: monocle-ipc
subsystems: [SS-05]
behavioral_contracts: [BC-2.05.010, BC-2.05.011]
verification_properties: []
estimated_days: 5
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.010.md, version: "1.9.4"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.011.md, version: "1.2.4"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.24.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.6.1"}
  - {path: .factory/specs/architecture/SS-conventions-anti-patterns.md, version: "1.32.6"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.05.010 (7 new ClientToServer variants + routing with SpawnOptions, no-silent-failure invariant, ResizePane zero-dim clamp) and BC-2.05.011 (ScrollbackChunk*/Complete/PtyReset server-to-client variants + pending_pty_bytes buffer + post-dump replay)"
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

### AC-001 (traces to BC-2.05.010 postcondition 1 — SpawnSession carries SpawnOptions, daemon builds SpawnRecipe)

`ClientToServer::SpawnSession { opts: SpawnOptions }` is the wire message for spawn
requests. The daemon receives `opts` and calls `engine_module.spawn_recipe(&opts)?` to
build `SpawnRecipe` internally (Model A — I27-001). The TUI NEVER sends a `SpawnRecipe`
directly. SpawnOptions includes `{ project_root, worktree_root, session_id, ccr_base_url, ... }`.

### AC-002 (traces to BC-2.05.010 postcondition 5 — KillSession delivers SIGTERM; idempotent on Terminating and Terminated)

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

### AC-003 (traces to BC-2.05.010 postcondition 6/7 — KeyInput and ResizePane routing via SessionManager → DaemonToHost)

`ClientToServer::KeyInput { session_id: String, bytes: Vec<u8> }` — the daemon IPC handler
calls `session_manager.send_key_input(&session_id, bytes)`, which sends
`DaemonToHost::KeyInput { bytes }` to the session-host over the per-session UDS control
connection. The session-host owns the PTY master fd and writes `bytes` to it. The daemon
NEVER holds the PTY master fd directly (PTY fd ownership is in the session-host process per
SS-session-manager.md §monocle-session-host binary and ADR-0009/ADR-0010).
`session_id` is a `String` (UUID-as-String) on the wire. On failure (session not found or
host dead), returns `ServerToClient::Error { code: "session_not_found" }` or `"attach_failed"`
as appropriate per the 12-code taxonomy — no `"pty_write_failed"` code.

`ClientToServer::ResizePane { session_id: String, rows: u16, cols: u16 }` — the daemon IPC
handler calls `session_manager.resize_session(&session_id, rows.max(1), cols.max(1))`, which
sends `DaemonToHost::Resize { rows, cols }` to the session-host over the control connection.
The session-host calls `pty.resize(PtySize { rows, cols, .. })` and `parser.set_size(rows, cols)`
(it owns the PTY; the daemon issues NO ioctl directly).
Zero-dim clamp: if `rows == 0` OR `cols == 0`, clamp to 1 BEFORE sending `DaemonToHost::Resize`.
After clamping, WARN-drop all transport errors for resize (do NOT propagate as
`ServerToClient::Error` — resize failures are advisory only per BC-2.05.010 PC-6 carve-out).

### AC-004 (traces to BC-2.05.010 postcondition 8 — DetachSession blocks on Launching, defensive path only)

`ClientToServer::DetachSession { session_id }` — daemon stops forwarding PTY output to
the requesting client but does NOT stop the session. If the session is in `Launching`
state, return `ServerToClient::Error { code: "session_not_ready", ... }` (defensive path
— the TUI should not send DetachSession during Launching per BC-2.06.025; this error is
a guard, not a primary flow).

### AC-005 (traces to BC-2.05.010 postcondition 9 — RenameSession updates display name)

`ClientToServer::RenameSession { session_id, new_name: String }` — daemon calls
`session_manager.rename_session(&session_id, new_name)`, which updates
`session.display_name` in the registry and publishes `SessionListUpdate` fan-out to all
clients. The field is `new_name` (canonical per SS-ipc.md §ClientToServer RenameSession
and BC-2.05.010 PC-4a). `name` MUST NOT be used — it is an incorrect alias.
Rename is ALLOWED when the session is in `Launching` or `Running` state.

### AC-006 (traces to BC-2.05.010 postcondition 10 — AttachSession triggers scrollback dump sequence)

`ClientToServer::AttachSession { session_id }` — daemon begins subscribing the requesting
client to the session's PTY output AND immediately initiates a scrollback dump sequence.
The dump sequence is:
1. Set `pending_pty_bytes = true` for this client (buffer live PTY bytes during dump).
2. Stream `ServerToClient::ScrollbackChunk { session_id, chunk_seq: u32, data: Bytes }` packets.
3. Send `ServerToClient::ScrollbackDumpComplete { session_id, total_chunks: u32, cursor_row: u16, cursor_col: u16, term_rows: u16, term_cols: u16 }`.
4. Set `pending_pty_bytes = false`; flush buffered bytes as live PTY output.

### AC-007 (traces to BC-2.05.011 postcondition 1 — ScrollbackChunk contiguity)

Each `ScrollbackChunk` for a given session MUST have `chunk_seq` values that are
contiguous and start from 0. If the client receives a chunk where
`chunk_seq != expected_seq`, the client MUST:
- Discard all buffered chunks for that session.
- Send `ClientToServer::AttachSession` to restart the dump.
- NOT attempt to reconstruct from out-of-order chunks.

### AC-008 (traces to BC-2.05.011 postcondition 2 — ScrollbackDumpComplete validates total_chunks)

When `ScrollbackDumpComplete` arrives:
- Client validates `count_of_received_chunks == total_chunks`.
- If mismatch: re-trigger AttachSession (restart dump).
- If match: reconstruct the full screen from concatenated chunk data, apply
  `cursor_row/cursor_col` and `term_rows/term_cols`, then replay buffered live PTY bytes.

### AC-009 (traces to BC-2.05.011 postcondition 3 — PtyReset clears buffer and re-triggers attach)

When `ServerToClient::PtyReset { session_id }` is received by the TUI:
- Clear all in-flight scrollback chunks for `session_id`.
- Clear the local PTY display buffer for `session_id`.
- Display a status bar message: "[PTY reset — <session_id_short>]" for 5 seconds.
- Re-trigger `ClientToServer::AttachSession { session_id }` automatically.

### AC-010 (traces to BC-2.05.011 invariant 1 — pending_pty_bytes buffer during dump)

During a scrollback dump (between AttachSession receipt and ScrollbackDumpComplete send):
- Live PTY bytes for this client are buffered in a `pending_pty_bytes: VecDeque<Bytes>`
  per session-client pair.
- The buffer is NOT bounded by the 64-item per-client send buffer (it is a separate
  accumulation buffer during dump).
- Once dump completes, buffered bytes are replayed as live `PtyOutput` messages.
- If the session exits during dump-in-progress, `PtyReset` is emitted instead of
  `ScrollbackDumpComplete`.

### AC-011 (traces to BC-2.05.010 invariant 4 — No-silent-failure invariant)

For all 7 `ClientToServer` variants (except ResizePane per AC-003 carve-out):
- Every failure produces a `ServerToClient::Error { code: <one of the canonical 12 wire codes>, message: String }`.
- The daemon NEVER silently ignores a message.
- Unknown `ClientToServer` variants (forward-compatibility): the `#[non_exhaustive]` wildcard
  arm maps them to `ServerToClient::Error { code: "invalid_request", message: "unknown variant" }` (the
  generic catch-all code per the 12-code taxonomy). The phantom code `"unknown_command"` MUST NOT be used.
- `KillSession` on `Terminating`/`Terminated`: idempotent `Ok(())` — NO `ServerToClient::Error` emitted
  (idempotent success is NOT a failure). This is consistent with BC-2.08.003 Invariant 2.

### AC-012 (traces to BC-2.05.010 invariant 5 — canonical 12 wire error codes are the CLOSED set for Phase 1)

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
- [ ] Add match arms for `KeyInput`, `ResizePane`, and `RenameSession` in the daemon's IPC dispatch
      loop (`monocle-runtime/src/ipc_handler.rs`). These are the 3 arms owned by S-047.
      The `SpawnSession`, `KillSession`, `AttachSession`, and `DetachSession` arms are authored by
      S-033, S-034, and S-035 respectively — S-047 MUST NOT re-add or duplicate those arms.
      - `KeyInput` → `session_manager.send_key_input(id, bytes)` — on `Err(SessionNotFound)` return
        `ServerToClient::Error { code: "session_not_found" }`; on `Err(SessionHostDead)` return
        `ServerToClient::Error { code: "attach_failed" }`. Do NOT use `"pty_write_failed"` — it is
        NOT in the canonical 12-code taxonomy (AC-012).
      - `ResizePane` → clamp `rows.max(1), cols.max(1)` (zero-dim guard) THEN call
        `session_manager.resize_session(id, clamped_rows, clamped_cols)`, which sends
        `DaemonToHost::Resize { rows, cols }` to the session-host (which owns the PTY fd
        and calls ioctl). The daemon issues NO ioctl directly. WARN-drop all
        transport errors (no `ServerToClient::Error` response for resize).
      - `RenameSession` → call `session_manager.rename_session(id, new_name)` → update display_name,
        fan-out `SessionListUpdate`.
- [ ] Add `pending_pty_bytes: HashMap<(String, String), VecDeque<Bytes>>` to `DaemonState`
      (or per-session state struct) for in-flight scrollback buffering. Keys are
      `(session_id: String, client_id: String)` — daemon-internal representation, consistent
      with how the broker tracks per-client channels by string key.
- [ ] Implement scrollback dump task in `monocle-runtime/src/scrollback.rs`:
      - Read PTY scrollback buffer (existing ring buffer from session state).
      - Chunk into ≤4 KiB `ScrollbackChunk` frames.
      - Stream to client via IPC sender; set `chunk_seq` monotonically from 0.
      - Send `ScrollbackDumpComplete` with final screen cursor/dimensions.
      - Flush `pending_pty_bytes` on complete.
      - On session exit during dump → emit `PtyReset` instead of DumpComplete.

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
- [ ] Write integration tests in `monocle-runtime/tests/ipc_lifecycle.rs`:
      - `test_BC_2_05_010_spawn_session_carries_spawn_options_not_recipe` (AC-001)
      - `test_BC_2_05_010_kill_session_allowed_launching_rejected_terminating` (AC-002)
      - `test_BC_2_05_010_resize_pane_zero_clamp_warns_not_errors` (AC-003)
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

From `architecture/SS-ipc.md v1.24.0` (canonical version):
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
| `bytes` | `^1.11` | `ScrollbackChunk.data` field |
| `thiserror` | `^2` | Error types in monocle-runtime routing layer |
| `tracing` | `^0.1` | warn!/error! in IPC handler and scrollback task |

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/monocle-ipc/src/lib.rs` | MODIFY | Add 7 new `ClientToServer` variants; add `ScrollbackChunk`, `ScrollbackDumpComplete` to `ServerToClient`; verify `SpawnOptions` struct (all wire fields use `String` — no typed newtypes) |
| `crates/monocle-runtime/src/ipc_handler.rs` | MODIFY | Add match arms for `KeyInput`, `ResizePane`, and `RenameSession` (3 arms owned by S-047); `SpawnSession`/`KillSession`/`AttachSession`/`DetachSession` arms are authored by S-033/S-034/S-035 — do NOT duplicate |
| `crates/monocle-runtime/src/session_manager/mod.rs` | MODIFY | Add `write_pty_bytes()`, `resize_pane()`, and `rename_session()` methods; `kill_session()` is authored by S-034 (canonical path: module dir, not flat .rs file) |
| `crates/monocle-runtime/src/scrollback.rs` | CREATE | Scrollback dump task implementation |
| `crates/monocle-runtime/src/lib.rs` | MODIFY | Add `pub mod scrollback;` |
| `crates/monocle-tui/src/ipc_receiver.rs` | MODIFY | Add handlers for ScrollbackChunk, ScrollbackDumpComplete, PtyReset (TUI-side) |
| `crates/monocle-runtime/tests/ipc_lifecycle.rs` | CREATE | Integration tests for all ACs |

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
| Story spec (this file) | ~6 000 tokens |
| BC files (2 BCs: BC-2.05.010 + BC-2.05.011) | ~8 000 tokens |
| Architecture sections (SS-ipc, SS-session-manager, SS-conventions) | ~3 500 tokens |
| Existing code context (monocle-ipc/src/lib.rs, ipc_handler.rs, session_manager/mod.rs, ipc_receiver.rs) | ~5 000 tokens |
| Test file to write | ~4 000 tokens |
| **Total estimated** | **~26 500 tokens** |

Approaches the 30% constraint; story is dense but coherent — all ACs share the IPC handler
dispatch context. If the implementing agent's context window is <90k tokens, consider splitting
AC-001..AC-006 (daemon routing) and AC-007..AC-012 (scrollback + no-silent-failure) into two
passes with the same story file.

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

S-047 authors the **`ClientToServer::KeyInput`**, **`ClientToServer::ResizePane`**, and
**`ClientToServer::RenameSession`** arms in `monocle-runtime/src/ipc_handler.rs`.
The canonical 7-arm split across the SS-08/SS-09 stories is:

| IPC Handler Arm | Owning Story |
|-----------------|-------------|
| `ClientToServer::SpawnSession` | S-033 |
| `ClientToServer::KillSession` | S-034 |
| `ClientToServer::AttachSession` | S-035 |
| `ClientToServer::DetachSession` | S-035 |
| `ClientToServer::KeyInput` | **S-047** (this story) |
| `ClientToServer::ResizePane` | **S-047** (this story) |
| `ClientToServer::RenameSession` | **S-047** (this story) |

S-047 MUST NOT re-add or duplicate the SpawnSession, KillSession, AttachSession, or DetachSession
arms — those arms are authored by S-033, S-034, and S-035 and already present in `ipc_handler.rs`
when S-047 is dispatched. S-047 only adds the 3 arms above (KeyInput, ResizePane, RenameSession)
and integrates the full dispatch loop by routing through the SessionManager methods established
by S-033/S-034/S-035.

## Subsystem Anchor Justification

SS-05 owns this story's scope because both BC-2.05.010 and BC-2.05.011 define IPC wire protocol
extensions — the client/server lifecycle message set — which is the core capability of SS-05
(monocle-ipc, daemon IPC layer) per ARCH-INDEX Subsystem Registry SS-05.
