---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-ipc.md, architecture/SS-daemon-wiring-v2-delta.md]
input-hash: "ec84d41"
traces_to: prd.md
origin: greenfield
subsystem: SS-05
capability: CAP-005
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.05.010: New ClientToServer IPC Variants — SpawnSession, KillSession, KeyInput, ResizePane, DetachSession, RenameSession

## Description

v1A adds six new `ClientToServer` IPC message variants for session lifecycle operations.
The TUI sends these messages to the daemon's UDS server, which routes them to
`SessionManager`. Each variant is `#[non_exhaustive]` per BC-2.02.003. The daemon handles
each variant by delegating to the corresponding `SessionManager` method and broadcasting
state updates to all TUI clients.

## Preconditions

1. TUI client is connected to the daemon's UDS.
2. The daemon's IPC handler processes `ClientToServer` messages.

## Postconditions

### SpawnSession

1. `ClientToServer::SpawnSession { recipe: SpawnRecipe }` is received.
2. Daemon calls `SessionManager::spawn_session(recipe, ...)` (BC-2.08.001).
3. On success: `ServerToClient::SessionListUpdate` broadcast.
4. On failure: `ServerToClient::Error { code: "spawn_failed", message: ... }` sent to the requesting client.

### KillSession

1. `ClientToServer::KillSession { session_id: String }` is received.
2. Daemon calls `SessionManager::kill_session(&session_id)` (BC-2.08.003).
3. On success: `ServerToClient::SessionListUpdate` broadcast.
4. On failure (not found): `ServerToClient::Error { code: "session_not_found", message: ... }`.

### KeyInput

1. `ClientToServer::KeyInput { session_id: String, bytes: Vec<u8> }` is received.
2. Daemon calls `SessionManager::send_key_input(&session_id, bytes)`.
3. No broadcast — key input is fire-and-forget; no acknowledgement message sent back.
4. On failure (session not found or dead): `ServerToClient::Error` sent to requesting client.

### ResizePane

1. `ClientToServer::ResizePane { session_id: String, rows: u16, cols: u16 }` is received.
2. Daemon calls `SessionManager::resize_session(&session_id, rows, cols)`.
3. No broadcast — resize is fire-and-forward; no acknowledgement.

### DetachSession

1. `ClientToServer::DetachSession { session_id: String }` is received.
2. Daemon calls `SessionManager::detach_session(&session_id)` (BC-2.08.007).
3. On success: `ServerToClient::SessionListUpdate` broadcast.

### RenameSession

1. `ClientToServer::RenameSession { session_id: String, new_name: String }` is received.
2. Daemon calls `SessionManager::rename_session(&session_id, new_name)`.
3. On success: `ServerToClient::SessionListUpdate` broadcast.

## Invariants

1. All six variants are `#[non_exhaustive]` fields per ADR-0006 (non-exhaustive structs with
   public constructors). The message enum itself is `#[non_exhaustive]` per BC-2.02.003.
2. `KeyInput` and `ResizePane` are high-frequency messages. The IPC handler MUST process
   them with minimal latency (no locking beyond `Arc<Mutex<SessionManager>>.lock()`).
3. `session_id` is a `String` (UUID rendered as string) at all IPC boundaries per
   SS-session-manager.md §session_id type ruling.
4. Unknown variants (future additions from newer TUI to older daemon) are silently ignored
   per `#[non_exhaustive]` forward-compat policy.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-280 | `SpawnSession` with a `SpawnRecipe` where `binary` is empty path | `SessionError::SpawnFailed`; `ServerToClient::Error` sent back |
| EC-281 | `KeyInput` for unknown `session_id` | `SessionError::SessionNotFound`; `ServerToClient::Error` |
| EC-282 | `ResizePane` with `rows=0` or `cols=0` | Passed to `resize_session()`; if PTY rejects zero dimensions, `SessionError` returned; `ServerToClient::Error` |
| EC-283 | `RenameSession` with empty `new_name` | `SessionError::InvalidSessionName`; `ServerToClient::Error` |
| EC-284 | Concurrent `KeyInput` messages from the same TUI client | Processed in order of arrival; each forwarded to session-host in receipt order |

## Canonical Test Vectors

| Message | Expected Daemon Response | Category |
|---------|-------------------------|----------|
| `SpawnSession { recipe: valid }` | `SessionListUpdate` broadcast; `spawn_session()` called | happy-path |
| `KillSession { session_id: "existing" }` | `SessionListUpdate` broadcast | happy-path |
| `KillSession { session_id: "nonexistent" }` | `Error { code: "session_not_found" }` to requesting client | error |
| `KeyInput { session_id: "running", bytes: [0x61] }` | `send_key_input()` called; no broadcast | happy-path |
| `ResizePane { rows: 30, cols: 120 }` | `resize_session()` called; no broadcast | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 6 new variants routed to correct `SessionManager` methods | integration |
| VP-TBD | `KeyInput` and `ResizePane` generate no broadcast (fire-and-forget) | unit |
| VP-TBD | Unknown `ClientToServer` variant handled without panic | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability §SS-05 |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability — these new ClientToServer variants extend the internal transport capability with session lifecycle control messages (spawn, kill, key input, resize, detach, rename) — all transported over the existing UDS per the session/event/prompt push design |
| Architecture Module | monocle-ipc (`ClientToServer` enum new variants); monocle-runtime (IPC handler routing to SessionManager) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-daemon-wiring-v2-delta.md v1.0.1 §IPC handler — new ClientToServer variants |
| Cross-Ref | BC-2.08.001 (SpawnSession → spawn_session()); BC-2.08.003 (KillSession → kill_session()); BC-2.08.007 (DetachSession → detach_session()) |
| Test Name | test_BC_2_05_010_new_client_to_server_variants_routed |

## Related BCs

- [BC-2.08.001] — depends on: SpawnSession IPC triggers spawn_session()
- [BC-2.08.003] — depends on: KillSession IPC triggers kill_session()
- [BC-2.08.007] — depends on: DetachSession IPC triggers detach_session()
- [BC-2.05.002] — composes with: existing IPC connection framework carries these new variants

## Architecture Anchors

- `architecture/SS-daemon-wiring-v2-delta.md#ipc-handler-new-clienttoserver-variants` — handler branches

## Story Anchor

S-TBD — Implement new ClientToServer IPC variants and daemon routing (filled by story-writer)

## VP Anchors

VP-TBD — IPC variant routing integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.05.010 authored for SS-05 as part of the v1A control-center pivot BC burst.
- Covers all 6 new ClientToServer variants from SS-daemon-wiring-v2-delta.md.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
