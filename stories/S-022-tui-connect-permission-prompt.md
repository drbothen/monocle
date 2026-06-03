---
document_type: story
level: L4
story_id: S-022
epic_id: EPIC-05
version: "1.4"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-28T00:00:00Z
phase: 2
points: 8
wave: 6
tdd_mode: strict
priority: P0
depends_on: [S-021, S-018]
blocks: [S-023, S-025, S-026, S-029]
target_module: monocle-ipc
subsystems: [SS-05]
behavioral_contracts: [BC-2.05.002, BC-2.05.005]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md, version: "1.0.3"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.005.md, version: "1.6.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "931a19a"
traces_to: "Implements BC-2.05.002 (TUI connect + InitialState push), BC-2.05.005 (PermissionPromptQueued broadcast + PermissionDecision routing)"
---

# S-022: TUI UDS Connection, InitialState Push, and Permission Prompt IPC

## Narrative

As a TUI client, I want to connect to the daemon's Unix domain socket and immediately receive
a complete initial state snapshot (session roster, ring tail, queued prompts, drop counter),
and I want permission prompts to flow bidirectionally (daemon broadcasts `PermissionPromptQueued`;
TUI sends `PermissionDecision`; daemon routes to the oneshot and broadcasts
`PermissionPromptResolved` to all clients), so that the TUI renders accurate live state on
first connect and enforces at-most-one permission decision per prompt.

## Acceptance Criteria

### AC-001 (traces to BC-2.05.002 postcondition PC-1 — per-client Tokio task)
The daemon accepts each TUI connection and spawns a dedicated Tokio task for the client
session. The client session task owns the per-client send loop and is responsible for
removing the client from the fan-out subscriber list on disconnect (clean EOF or error).

### AC-002 (traces to BC-2.05.002 postcondition PC-2 — InitialState as first message)
The daemon sends exactly one `ServerToClient::InitialState` message immediately upon
connection, before any other message. The message contains:
- `sessions: Vec<EnrichedSession>` — the full current session roster (may be empty).
- `ring_tail: Vec<HookEventRecord>` — the last N events from the RAM ring (may be empty).
- `overlay_stack: Vec<PermissionPromptPayload>` — queued permission prompts awaiting decision
  (may be empty).
- `drop_counter: u64` — the current daemon drop counter value.

### AC-003 (traces to BC-2.05.002 postcondition PC-3 — 4-byte LE framing)
All messages are framed using the length-prefix protocol: a 4-byte little-endian `u32`
encoding the byte length of the JSON payload, followed by the UTF-8 JSON payload. No trailing
newline or null terminator. The TUI decoder uses the same framing for all `ServerToClient`
message types without special-casing.

### AC-004 (traces to BC-2.05.002 postcondition PC-4 — InitialState 256 KiB limit)
If the JSON serialization of `InitialState` exceeds 256 KiB (262,144 bytes), the daemon
closes the connection with `IpcError::MessageTooLarge` and logs
`ERROR: InitialState for client exceeds 256 KiB limit (<N> bytes)`. The TUI receives EOF
and enters reconnect mode (BC-2.05.006).

### AC-005 (traces to BC-2.05.002 postcondition PC-5/PC-6 — push-only model)
After the `InitialState` push, the TUI renders its initial state from this message without
polling the daemon. All subsequent state changes arrive as push messages. The TUI's IPC
receive loop runs concurrently with the render loop; IPC messages do not block the terminal
event loop.

### AC-006 (traces to BC-2.05.002 invariant 3 — no gap window)
The `InitialState` message reflects the daemon's state at the moment of connection acceptance.
Hook events or session changes that occur after the connection is accepted but before
`InitialState` is fully written are delivered as subsequent incremental updates. No events
are dropped between `InitialState` snapshot and streaming phase.

### AC-007 (traces to BC-2.05.005 postcondition PC-1 — PermissionPromptQueued broadcast)
When a `PreToolUse` hook POST arrives with `decision_required: true`, the daemon generates
a unique `prompt_id: Uuid`, stores a `oneshot::Sender<PermissionDecision>` keyed by
`prompt_id`, and broadcasts `ServerToClient::PermissionPromptQueued { payload: PermissionPromptPayload }`
to all connected TUI clients. The enum variant carries all fields in the `payload` wrapper:
`payload.prompt_id`, `payload.session_id`, `payload.tool_name`, `payload.tool_input`,
`payload.old_content: Option<String>`, `payload.new_content: Option<String>`.

### AC-008 (traces to BC-2.05.005 postcondition PC-2 — prompt_id stability)
The `prompt_id` generated at prompt creation is stable for the lifetime of the pending
decision. It is identical in `PermissionPromptQueued` and the corresponding
`PermissionPromptResolved`. TUI clients use `prompt_id` as the exact key into their local
overlay stack.

### AC-009 (traces to BC-2.05.005 postcondition PC-3 — PermissionDecision routing)
When a TUI client sends `ClientToServer::PermissionDecision { prompt_id, decision }`:
- If `prompt_id` is found in the pending-decision registry: the daemon resolves the
  `oneshot::Sender` with the decision; sends the HTTP response to Claude Code; removes the
  entry from the registry; broadcasts `ServerToClient::PermissionPromptResolved { prompt_id }`
  to ALL connected TUI clients (including the resolving client, which treats this as a no-op).
- If `prompt_id` is NOT found (second resolution attempt or already timed out): the
  `PermissionDecision` message is silently discarded. No error is returned to the TUI client.

### AC-010 (traces to BC-2.05.005 postcondition PC-4 — PermissionPromptResolved on timeout)
When the hook timeout expires (300ms per BC-2.04.007) before any TUI client resolves the
prompt, the daemon:
1. Resolves the pending hook response with fail-open semantics (Claude Code's default applies).
2. Sends `ServerToClient::PermissionPromptResolved { prompt_id }` to ALL connected TUI
   clients so they can remove the stale overlay entry.
3. Removes the `prompt_id` entry from the pending-decision registry.

TUI clients handle the timeout-triggered `PermissionPromptResolved` identically to a
user-resolved prompt.

### AC-011 (traces to BC-2.05.005 invariant 2 — at-most-one resolution via oneshot)
The `oneshot::channel` per prompt enforces at-most-one resolution. The first
`PermissionDecision` to arrive resolves the channel. All subsequent decisions for the same
`prompt_id` are silently discarded. This prevents double-approval races when multiple TUI
clients view the same prompt simultaneously.

### AC-012 (traces to BC-2.05.005 invariant 3 — PermissionPromptResolved requires prior Queued)
The daemon never sends `PermissionPromptResolved` without a corresponding prior
`PermissionPromptQueued` for the same `prompt_id` within the lifetime of a single daemon
process. TUI clients can safely use `prompt_id` as an exact key into their local overlay stack.

### AC-013 (traces to BC-2.05.002 edge case EC-001 — empty InitialState)
When the daemon has zero sessions, zero ring events, zero queued prompts, and drop_counter=0,
`InitialState` is sent with all fields as empty Vecs and drop_counter=0. The TUI renders
"No sessions detected".

### AC-014 (traces to BC-2.05.005 edge case EC-001 — dual-resolution race)
When two TUI clients both send `PermissionDecision` simultaneously for the same `prompt_id`,
the first to reach the daemon resolves the oneshot. The second finds no registry entry and is
silently discarded. `PermissionPromptResolved` is broadcast exactly once, after the first
resolution.

### AC-015 (traces to BC-2.05.005 edge case EC-003 — no clients connected for PermissionPromptQueued)
When `PermissionPromptQueued` would be broadcast but no TUI clients are connected, the message
is not sent (empty subscriber list). The daemon holds the HTTP response open. The pending
prompt is visible in `InitialState.overlay_stack` for the next connecting TUI client.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,700 |
| BC-2.05.002.md | ~900 |
| BC-2.05.005.md (v1.6.0) | ~1,200 |
| SS-ipc.md §Connection Lifecycle + §Risk Mitigations | ~4,000 |
| S-021 (UDS Transport, framing, type definitions) | ~600 |
| S-018 (PreToolUse hook routing — Defer oneshot) | ~500 |
| Test files | ~900 |
| **Total estimate** | **~9,800** |

## Tasks

- [ ] Implement `UdsTransport::connect(runtime_dir: &Path)` in `monocle-ipc/src/uds.rs`:
  - `UnixStream::connect(runtime_dir.join("monocle.sock"))`
  - `read_framed` / `write_framed` using the 4-byte LE framing from `monocle-ipc/src/framing.rs`
  - Implement `Transport` trait: `send_message` / `recv_message`
- [ ] Implement daemon connection accept loop in `monocle-ipc/src/server.rs`:
  - Accept incoming connections from `UnixListener` (from S-021 UDS bind)
  - Spawn dedicated Tokio task per client (per-client send loop + fan-out subscriber registration)
  - On client task spawn: take `InitialState` snapshot from `DaemonState` and send as first message
  - Client task: remove from fan-out subscriber list on EOF or send error
- [ ] Implement `InitialState` snapshot in `monocle-runtime/src/state.rs`:
  - `fn snapshot_initial_state(state: &DaemonState) -> InitialState`
  - Fields: `sessions` (clone of current session map), `ring_tail` (last N from RAM ring), `overlay_stack` (clone of pending-decision registry payloads), `drop_counter` (current value)
  - 256 KiB guard: serialize first; if > 262,144 bytes log error and close connection with `IpcError::MessageTooLarge`
- [ ] Implement pending-decision registry in `monocle-runtime/src/permissions.rs`:
  - `HashMap<Uuid, (PermissionPromptPayload, oneshot::Sender<PermissionDecision>)>` protected by `Arc<Mutex<...>>`
  - `register_prompt(payload, sender) -> Uuid` — generates stable `prompt_id`
  - `resolve_prompt(prompt_id, decision) -> Option<()>` — returns None if not found (second resolution silently discarded)
  - `remove_timed_out_prompt(prompt_id)` — called by PreToolUse timeout handler to remove entry and broadcast PermissionPromptResolved
- [ ] Integrate `PermissionPromptQueued` broadcast into PreToolUse hook handler (from S-018):
  - On `decision_required: true` path: call `register_prompt`; broadcast `PermissionPromptQueued { payload: PermissionPromptPayload { ... } }` to all connected TUI clients
- [ ] Implement `PermissionDecision` receive path in per-client task:
  - Read `ClientToServer::PermissionDecision { prompt_id, decision }` from TUI client
  - Call `resolve_prompt(prompt_id, decision)` — if Some: resolve oneshot, remove registry entry, broadcast `PermissionPromptResolved { prompt_id }` to all clients
  - If None: silently discard
- [ ] Implement timeout `PermissionPromptResolved` broadcast in PreToolUse timeout handler (modify S-018 Defer path):
  - On 300ms timeout: resolve fail-open; call `remove_timed_out_prompt(prompt_id)`; broadcast `PermissionPromptResolved { prompt_id }` to all clients
- [ ] Integration tests `monocle-ipc/tests/connection_handshake.rs`:
  - InitialState is first message sent to every connecting TUI client
  - InitialState fields match daemon state at connection time
  - Framing: 4-byte LE length prefix encodes payload length correctly
  - MessageTooLarge closes connection when InitialState > 262,144 bytes
  - Multiple simultaneous clients each receive independent InitialState
  - Empty daemon state: InitialState with all empty Vecs and drop_counter=0
- [ ] Integration tests `monocle-ipc/tests/permission_prompt.rs`:
  - PermissionPromptQueued broadcast on `decision_required: true` PreToolUse arrival
  - prompt_id is stable across queued → resolved lifecycle
  - User resolves: oneshot resolved; PermissionPromptResolved broadcast to all clients (including resolver)
  - Second PermissionDecision for same prompt_id: silently discarded; no duplicate PermissionPromptResolved
  - Timeout path: registry entry removed; PermissionPromptResolved broadcast to all connected TUI clients
  - No clients connected: PermissionPromptQueued not sent; pending prompt in InitialState.overlay_stack for next connect
  - PermissionDecision for unknown prompt_id: silently discarded; no error response

## Previous Story Intelligence

S-021: `UdsTransport`, `Transport` trait, `ServerToClient` / `ClientToServer` enums,
`PermissionPromptPayload` struct, `IpcError`, and the 4-byte LE framing are defined. This
story implements the client-side connect path (`UdsTransport::connect`) and the server-side
connection accept loop (which was not part of S-021 — that story covered bind and fan-out
types). The `PermissionPromptQueued { payload: PermissionPromptPayload }` variant is the
canonical form with a wrapped struct, NOT flat fields.

S-018: The PreToolUse hook routing Defer path (via `oneshot::channel`) is established. This
story integrates with that path to register the prompt in the pending-decision registry and
broadcast `PermissionPromptQueued`. The 300ms timeout handler in S-018 must also call the
`remove_timed_out_prompt` cleanup and broadcast `PermissionPromptResolved`.

## Architecture Compliance Rules

From `architecture/SS-ipc.md v1.4.0 §Connection Lifecycle §Phase 1 Connect` (at S-022 authoring time):
- `InitialState` is the FIRST message on every new connection — no exceptions
- Snapshot is taken at connection acceptance time; events after acceptance are incremental
- No gap between snapshot and streaming phase (fan-out subscriber added before first event can be missed)

From `architecture/SS-ipc.md v1.4.0 §Message Types §PermissionPromptPayload` (at S-022 authoring time):
- `PermissionPromptQueued { payload: PermissionPromptPayload }` — wrapper struct, NOT flat fields
- `payload.old_content` and `payload.new_content` are BOTH `Option<String>` — present only for file-mutation tools

From `architecture/SS-ipc.md v1.4.0 §Risk Mitigations §Multiple TUI Clients Resolving the Same Prompt` (at S-022 authoring time):
- `oneshot::channel` per prompt enforces at-most-one resolution — not advisory, required
- `PermissionPromptResolved` is broadcast to ALL connected clients after BOTH user resolution AND timeout

**Forbidden Dependencies:**
- `monocle-ipc` MUST NOT call `oneshot::Sender::send` more than once per `prompt_id` (at-most-one invariant)
- The pending-decision registry MUST NOT be iterated under a held lock while holding the fan-out subscriber list lock (avoid deadlock)
- `PermissionPromptResolved` MUST be sent on BOTH user decision and timeout — omitting either is a BC violation

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| interprocess | 2.4 | `UnixStream::connect` (TUI-side); per-client streams (server-side) |
| tokio | =1.52.0 | Per-client Tokio task; `oneshot::channel` for permission decisions; async I/O |
| serde_json | =1.0.149 | JSON serialization/deserialization for framed messages |
| serde | 1 (features=["derive"]) | Derive macros on IPC message types |
| uuid | (features=["v4","serde"]) | `prompt_id: Uuid` generation and serialization |
| tracing | 0.1 | DEBUG on client connect/disconnect; ERROR on InitialState too large |

## File Structure Requirements

Files to create:
- `monocle-ipc/src/server.rs` — connection accept loop; per-client Tokio task spawner; InitialState snapshot send; fan-out subscriber registration
- `monocle-ipc/tests/connection_handshake.rs` — InitialState push and framing tests
- `monocle-ipc/tests/permission_prompt.rs` — PermissionPromptQueued/Resolved lifecycle tests
- `monocle-runtime/src/permissions.rs` — pending-decision registry (`register_prompt`, `resolve_prompt`, `remove_timed_out_prompt`)

Files to modify:
- `monocle-ipc/src/uds.rs` — add `UdsTransport::connect(runtime_dir)` (client-side connect); add `read_framed` on `UnixStream`
- `monocle-ipc/src/lib.rs` — re-export `server` module
- `monocle-runtime/src/state.rs` — add `pending_decisions: Arc<Mutex<PendingDecisionRegistry>>` to `DaemonState`; add `snapshot_initial_state` function
- `monocle-runtime/src/lifecycle.rs` — integrate connection accept loop start at end of `daemon_start_sequence()` (after step 10 UDS bind from S-021)
- `monocle-runtime/src/hook_handlers.rs` (or equivalent from S-018) — on `decision_required: true` path: `register_prompt`; broadcast `PermissionPromptQueued`; on timeout: `remove_timed_out_prompt`; broadcast `PermissionPromptResolved`

## §Trace v1.3

**F-S022-ADV15-LOW-001 — ring_tail type corrected to Vec<HookEventRecord>** (2026-05-28):
- Finding: AC-002 listed `ring_tail: Vec<HookEvent>`. BC-2.05.002 v1.0.5 PC-2 and the
  implementation use `Vec<HookEventRecord>` — `HookEventRecord` is the persisted ring
  entry type (in `monocle-ipc::types`), distinct from the raw `HookEvent` enum.
  `HookEventRecord` carries timestamp and additional metadata; `HookEvent` is the
  parsed hook payload only.
- Fix: AC-002 updated from `Vec<HookEvent>` to `Vec<HookEventRecord>`.
- No BC update required: BC-2.05.002 v1.0.5 already uses the correct type.
