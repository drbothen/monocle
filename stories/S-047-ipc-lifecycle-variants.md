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
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.010.md, version: "1.9.1"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.011.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.23.2"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.6.0"}
  - {path: .factory/specs/architecture/SS-conventions-anti-patterns.md, version: "1.32.6"}
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

### AC-002 (traces to BC-2.05.010 postcondition 5 — KillSession delivers SIGTERM)

`ClientToServer::KillSession { session_id }` causes the daemon to send `SIGTERM` to the
session's PTY process. If the process is in `Launching` state, the kill is ALLOWED per
BC-2.06.025 lifecycle rules. If the process is in `Terminating` state, `KillSession` is
rejected with error code `"session_not_ready"`.

### AC-003 (traces to BC-2.05.010 postcondition 6/7 — KeyInput and ResizePane routing)

`ClientToServer::KeyInput { session_id, bytes: Vec<u8> }` — daemon writes `bytes` to the
session's PTY master fd.

`ClientToServer::ResizePane { session_id, rows: u16, cols: u16 }` — daemon issues
`ioctl(TIOCSWINSZ)` on the PTY. Zero-dim clamp: if `rows == 0` OR `cols == 0`, clamp to 1
before calling ioctl. After clamping, WARN-drop all ioctl errors (do NOT propagate resize
errors as `ServerToClient::Error` — resize failures are advisory only per BC-2.05.010 PC-6
carve-out).

### AC-004 (traces to BC-2.05.010 postcondition 8 — DetachSession blocks on Launching, defensive path only)

`ClientToServer::DetachSession { session_id }` — daemon stops forwarding PTY output to
the requesting client but does NOT stop the session. If the session is in `Launching`
state, return `ServerToClient::Error { code: "session_not_ready", ... }` (defensive path
— the TUI should not send DetachSession during Launching per BC-2.06.025; this error is
a guard, not a primary flow).

### AC-005 (traces to BC-2.05.010 postcondition 9 — RenameSession updates display name)

`ClientToServer::RenameSession { session_id, name: String }` — daemon updates
`session.display_name` in `DaemonState`. Propagates as `SessionListUpdate` fan-out to all
clients. Rename is ALLOWED when the session is in `Launching` or `Running` state.

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
- Every failure produces a `ServerToClient::Error { code: <one of the 12 wire codes>, message: String }`.
- The daemon NEVER silently ignores a message.
- Unknown `ClientToServer` variants (forward-compatibility): return
  `ServerToClient::Error { code: "unknown_command", message: "..." }`.

### AC-012 (traces to BC-2.05.010 invariant 5 — 12 wire error codes are the complete taxonomy)

The complete set of wire error codes for `ServerToClient::Error.code` is:
`"binary_not_found"`, `"invalid_spawn_arg"`, `"session_not_found"`, `"session_not_ready"`,
`"pty_write_failed"`, `"rename_rejected"`, `"kill_rejected"`, `"unknown_command"`,
`"internal_error"`, `"spawn_unsupported"`, `"permission_denied"`, `"protocol_error"`.
No code outside this list shall be returned. (`"binary_not_found"` and `"spawn_unsupported"`
are from BC-2.03.007/BC-2.03.008.)

## Tasks

### IPC Protocol (monocle-ipc)
- [ ] Add 7 new `ClientToServer` variants to `monocle-ipc/src/proto.rs`:
      `SpawnSession { opts: SpawnOptions }`,
      `KillSession { session_id: SessionId }`,
      `KeyInput { session_id: SessionId, bytes: Vec<u8> }`,
      `ResizePane { session_id: SessionId, rows: u16, cols: u16 }`,
      `DetachSession { session_id: SessionId }`,
      `RenameSession { session_id: SessionId, name: String }`,
      `AttachSession { session_id: SessionId }`.
- [ ] Add 3 new `ServerToClient` variants to `monocle-ipc/src/proto.rs`
      (note: `PtyReset` was added in S-046; add `ScrollbackChunk` and `ScrollbackDumpComplete` here):
      `ScrollbackChunk { session_id: SessionId, chunk_seq: u32, data: Bytes }`,
      `ScrollbackDumpComplete { session_id: SessionId, total_chunks: u32, cursor_row: u16, cursor_col: u16, term_rows: u16, term_cols: u16 }`.
- [ ] Add `SpawnOptions` struct to `monocle-ipc/src/proto.rs` (if not already present from S-033/S-045).
      Fields: `project_root: PathBuf`, `worktree_root: PathBuf`, `session_id: SessionId`,
      `ccr_base_url: Option<String>`, `display_name: Option<String>`.
- [ ] Add `"unknown_command"`, `"pty_write_failed"`, `"rename_rejected"`, `"kill_rejected"`,
      `"permission_denied"`, `"protocol_error"` to the error code taxonomy comment in `proto.rs`
      (joining existing codes from BC-2.03.007 and BC-2.05.010). These are documentation-only
      constants; the actual matching is in `session_manager.rs`.

### Daemon Routing (monocle-runtime)
- [ ] Add match arms for all 7 new `ClientToServer` variants in the daemon's IPC dispatch loop
      (`monocle-runtime/src/ipc_handler.rs` or equivalent):
      - `SpawnSession` → call `session_manager.spawn_session(opts)` → return Ok or Error.
      - `KillSession` → `session_manager.kill_session(id)` with state-check.
      - `KeyInput` → `session_manager.write_pty_bytes(id, bytes)` — on failure return
        `ServerToClient::Error { code: "pty_write_failed" }`.
      - `ResizePane` → `session_manager.resize_pane(id, rows.max(1), cols.max(1))` — zero-dim
        clamp THEN call; WARN-drop all ioctl errors (no Error response).
      - `DetachSession` → detach client subscription; guard for Launching state.
      - `RenameSession` → update display_name, fan-out SessionListUpdate.
      - `AttachSession` → subscribe client + initiate scrollback dump sequence.
- [ ] Add `pending_pty_bytes: HashMap<(SessionId, ClientId), VecDeque<Bytes>>` to `DaemonState`
      (or per-session state struct) for in-flight scrollback buffering.
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
it already exists in `proto.rs`, skip the create step and confirm field parity.

Key lesson from S-033 review: `session_manager.rs` gained both the spawn path and error mapping
in S-033. The IPC dispatch loop in S-047 calls into session_manager methods — it must NOT duplicate
session state management directly in the handler.

## Architecture Compliance Rules

From `architecture/SS-ipc.md v1.23.2`:
- Model A (I27-001): SpawnSession carries `SpawnOptions`; daemon builds `SpawnRecipe` internally.
  Model B (TUI sends SpawnRecipe) is REJECTED by architecture decision. Do not implement Model B.
- 256 KiB maximum IPC frame size still applies to ScrollbackChunk (chunk at ≤4 KiB to respect
  the 256 KiB total frame limit with framing overhead).
- `pending_pty_bytes` is a daemon-side struct, NOT a TUI-side struct. The TUI has a parallel
  accumulation buffer for chunks received during dump.
- ResizePane zero-dim clamp MUST happen before the ioctl call — never pass 0 rows or 0 cols to ioctl.

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
| `crates/monocle-ipc/src/proto.rs` | MODIFY | Add 7 new `ClientToServer` variants; add `ScrollbackChunk`, `ScrollbackDumpComplete` to `ServerToClient`; add `SpawnOptions` struct (if not present) |
| `crates/monocle-runtime/src/ipc_handler.rs` | MODIFY | Add match arms for all 7 new variants |
| `crates/monocle-runtime/src/session_manager.rs` | MODIFY | Add `kill_session()`, `write_pty_bytes()`, `resize_pane()`, `rename_session()` methods |
| `crates/monocle-runtime/src/scrollback.rs` | CREATE | Scrollback dump task implementation |
| `crates/monocle-runtime/src/lib.rs` | MODIFY | Add `pub mod scrollback;` |
| `crates/monocle-tui/src/ipc_receiver.rs` | MODIFY | Add handlers for ScrollbackChunk, ScrollbackDumpComplete, PtyReset (TUI-side) |
| `crates/monocle-runtime/tests/ipc_lifecycle.rs` | CREATE | Integration tests for all ACs |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-300 | `AttachSession` for a session in `Terminating` state | `ServerToClient::Error { code: "session_not_ready" }` — blanket block per BC-2.06.025 lifecycle rules (Terminating) |
| EC-301 | `KillSession` for unknown `session_id` | `ServerToClient::Error { code: "session_not_found" }` |
| EC-302 | `ResizePane { rows: 0, cols: 0 }` | Clamp to `{ rows: 1, cols: 1 }` then ioctl; WARN log; NO Error response to TUI |
| EC-303 | `ResizePane { rows: 0, cols: 80 }` (only rows is 0) | Clamp rows to 1; cols stays 80; ioctl with `{ rows: 1, cols: 80 }` |
| EC-304 | `KeyInput` arrives while session is in `Terminating` state | `ServerToClient::Error { code: "session_not_ready" }` |
| EC-305 | `ScrollbackChunk` arrives with `chunk_seq` gap (e.g., 0, 1, 3, missing 2) | Client discards all received chunks for that session; re-triggers `AttachSession` |
| EC-306 | `ScrollbackDumpComplete.total_chunks` is 0 (empty session, no output yet) | Valid — client reconstructs empty screen with cursor at (0,0); replay zero pending bytes |
| EC-307 | Session exits mid-scrollback dump | Daemon emits `PtyReset` instead of `ScrollbackDumpComplete`; TUI clears and re-attaches |
| EC-308 | Two clients simultaneously request `AttachSession` for same session | Each gets an independent scrollback dump sequence; `pending_pty_bytes` is per session-client pair (not shared) |
| EC-309 | Client sends unknown `ClientToServer` variant (forward compat) | `ServerToClient::Error { code: "unknown_command" }` — no panic, no silent drop |

## Token Budget Estimate

| Category | Estimate |
|----------|----------|
| Story spec (this file) | ~6 000 tokens |
| BC files (2 BCs: BC-2.05.010 v1.9.2 + BC-2.05.011 v1.2.2) | ~8 000 tokens |
| Architecture sections (SS-ipc, SS-session-manager, SS-conventions) | ~3 500 tokens |
| Existing code context (proto.rs, ipc_handler.rs, session_manager.rs, ipc_receiver.rs) | ~5 000 tokens |
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

## Subsystem Anchor Justification

SS-05 owns this story's scope because both BC-2.05.010 and BC-2.05.011 define IPC wire protocol
extensions — the client/server lifecycle message set — which is the core capability of SS-05
(monocle-ipc, daemon IPC layer) per ARCH-INDEX Subsystem Registry SS-05.
