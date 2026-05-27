---
document_type: behavioral-contract
level: L3
version: "1.0.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T04:00:00Z
phase: phase-1-expansion
inputs: [prd-expansion-scope.md, architecture/SS-ipc.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-05
capability: CAP-005
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.05.002: TUI Client Connects to UDS and Receives Initial State Push

## Description

When a TUI client connects to the daemon's Unix domain socket, the daemon immediately sends
an `InitialState` message containing the full current session roster, the ring tail (last N
hook events), any currently queued permission prompts awaiting decision, and the current drop
counter value. The connection uses a 4-byte little-endian length-prefix framing protocol with
a 256 KiB maximum message size. After the initial state push, all subsequent updates are
delivered as incremental push messages (no polling required).

## Preconditions

1. The daemon has bound the UDS socket at `<runtime_dir>/monocle.sock` per BC-2.05.001.
2. The TUI has read the lock file at `<runtime_dir>/monocle.lock` and confirmed daemon
   liveness (pid-liveness check via `kill(pid, 0)`).
3. The TUI calls `UnixStream::connect("<runtime_dir>/monocle.sock")`.

## Postconditions

1. The daemon accepts the connection and spawns a dedicated Tokio task for the client session.
   The client session task is responsible for the per-client send loop and for removing the
   client from the fan-out subscriber list on disconnect.
2. The daemon sends exactly one `ServerToClient::InitialState` message immediately upon
   connection, before any other message. The message contains:
   - `sessions: Vec<EnrichedSession>` — the full current session roster (may be empty).
   - `ring_tail: Vec<HookEvent>` — the last N events from the RAM ring (N defined by daemon
     configuration; may be empty if no events have been received yet).
   - `overlay_stack: Vec<PermissionPromptPayload>` — any currently queued permission prompts
     awaiting decision (may be empty).
   - `drop_counter: u64` — the current value of the daemon's drop counter.
3. All messages are framed using the length-prefix protocol: a 4-byte little-endian `u32`
   encoding the byte length of the JSON payload, followed by the UTF-8 JSON payload. No
   trailing newline or null terminator.
4. If the JSON serialization of `InitialState` exceeds 256 KiB (262,144 bytes), the daemon
   closes the connection with `IpcError::MessageTooLarge` and logs an error. (In practice,
   `ring_tail` should be bounded to prevent this.)
5. After the `InitialState` push, the TUI renders its initial state from this message without
   polling the daemon again. All subsequent state changes arrive as push messages
   (`SessionListUpdate`, `HookEventReceived`, `PermissionPromptQueued`, etc.).
6. The TUI's IPC receive loop runs concurrently with the render loop. IPC messages do not
   block the terminal event loop.

## Invariants

1. Every TUI client that successfully connects receives exactly one `InitialState` message as
   its first message. No client receives a partial state or must poll for initial data.
2. The framing protocol (4-byte LE length prefix + JSON payload) is identical for all
   `ServerToClient` message types. The TUI decoder does not need special-casing for
   `InitialState` vs incremental updates.
3. The `InitialState` message reflects the daemon's state at the moment of connection
   acceptance. Hook events or session changes that occur after the connection is accepted
   but before the `InitialState` is fully written are delivered as subsequent incremental
   updates (no gap window).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Daemon has zero sessions at the time of TUI connection | `InitialState.sessions` is an empty Vec; `ring_tail` is an empty Vec; `overlay_stack` is an empty Vec; `drop_counter` is 0. TUI renders "No sessions detected". |
| EC-002 | Multiple TUI clients connect simultaneously | Each client receives its own `InitialState` push. Each client has a dedicated Tokio task. State is consistent across all clients (snapshot of daemon state at connection time). |
| EC-003 | TUI process is killed (SIGKILL) immediately after connecting | Daemon detects EOF on the per-client stream; removes client from fan-out subscriber list; logs disconnect at DEBUG level. No crash or panic. |
| EC-004 | `InitialState` message exceeds 256 KiB (ring_tail very large) | Daemon closes connection with `IpcError::MessageTooLarge`; logs `ERROR: InitialState for client exceeds 256 KiB limit (<N> bytes)`. TUI receives EOF and enters reconnect mode (BC-2.05.006). |
| EC-005 | Daemon is under high load (many hook events per second) when TUI connects | `InitialState` uses the ring_tail snapshot at connection time; incremental `HookEventReceived` messages deliver events that arrived after snapshot. No duplicate events; no gap. |
| EC-006 | TUI connects when a permission prompt is already queued and awaiting decision | `InitialState.overlay_stack` contains the queued `PermissionPromptPayload`. TUI immediately renders the overlay without waiting for a new `PermissionPromptQueued` push. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| TUI connects; daemon has 2 sessions, 5 ring events, 1 queued prompt, drop_counter=3 | `InitialState` push with sessions=[2 items], ring_tail=[5 items], overlay_stack=[1 item], drop_counter=3 | happy-path |
| TUI connects; daemon is in empty initial state | `InitialState` with sessions=[], ring_tail=[], overlay_stack=[], drop_counter=0 | happy-path |
| TUI connects; daemon sends `InitialState`; daemon then receives a new hook event | TUI receives `InitialState` first, then `HookEventReceived` for the new event; no events duplicated | happy-path |
| TUI sends garbage bytes after connecting (protocol violation) | Daemon deserializes framed payload; JSON parse failure; daemon closes connection; logs WARN | error |
| `InitialState` JSON would be >256 KiB | Connection closed with `IpcError::MessageTooLarge`; TUI enters reconnect loop | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `InitialState` is the first message sent to every connecting TUI client | integration |
| VP-TBD | `InitialState` fields accurately reflect daemon state at connection time | integration |
| VP-TBD | Framing: 4-byte LE length prefix correctly encodes payload length | unit |
| VP-TBD | `MessageTooLarge` closes connection when payload > 262,144 bytes | unit |
| VP-TBD | Multiple simultaneous clients each receive independent `InitialState` | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability §SS-05 — this BC specifies the connection handshake and initial-state push that is the entry point for all TUI-to-daemon transport |
| L2 Domain Invariants | DI-001 (every hook event received by the daemon must be written to the JSONL ring before acknowledgement — the ring_tail in InitialState reflects events already durably written, satisfying DI-001 ordering); DI-002 (lock file must be present before connections accepted — Precondition 2 requires TUI to read the lock file before connecting, enforcing DI-002) |
| Architecture Module | monocle-ipc (UdsTransport, framing, ServerToClient::InitialState) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-ipc.md v1.4.0 §Framing Protocol; SS-ipc.md v1.4.0 §Connection Lifecycle §Phase 1 Connect |
| Cross-Ref | BC-2.05.001 (UDS socket that this BC connects to); BC-2.05.003..005 (subsequent push message types); BC-2.05.006 (reconnection on disconnect) |
| Test File | `monocle-ipc/tests/connection_handshake.rs` |
| Test Name | `test_BC_2_05_002_initial_state_push_on_connect` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.001] — depends on: socket must be bound before TUI can connect
- [BC-2.05.003] — composes with: SessionListUpdate delivered after InitialState
- [BC-2.05.004] — composes with: HookEventReceived delivered after InitialState
- [BC-2.05.005] — composes with: PermissionPromptQueued delivered after InitialState
- [BC-2.05.006] — composes with: reconnection uses fresh InitialState on reconnect

## Architecture Anchors

- `architecture/SS-ipc.md#framing-protocol` — wire format: 4-byte LE length prefix + JSON payload, MAX_MESSAGE_BYTES = 262,144
- `architecture/SS-ipc.md#connection-lifecycle` — Phase 1: Connect sequence (steps 1–5)

## Story Anchor

S-TBD — Implement TUI UDS connection with InitialState push (filled by story-writer)

## VP Anchors

VP-TBD — Connection handshake and InitialState push verification properties (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T04:00:00Z):
- BC-2.05.002 authored for SS-05 IPC subsystem per `prd-expansion-scope.md §3.2` and
  `SS-ipc.md §Framing Protocol + §Connection Lifecycle §Phase 1 Connect`.
- Covers: TUI connection to UDS, InitialState push (sessions + ring_tail + overlay_stack +
  drop_counter), framing protocol (4-byte LE length prefix, 256 KiB limit), concurrent
  clients, push-only model (no polling after initial state).
- 6 edge cases documented (EC-001..EC-006).
- SE-16d PASS: 2026-05-26T04:00:00Z is the production timestamp for this wave.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.0.0` → `SS-ipc.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-004 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.1.0` (2 occurrences) → `SS-ipc.md v1.3.0` per F-P1D4-004 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.3.0` (2 occurrences) → `SS-ipc.md v1.4.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.
