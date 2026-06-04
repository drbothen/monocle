---
document_type: behavioral-contract
level: L3
version: "1.4.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:45:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-ipc.md, architecture/SS-daemon-wiring-v2-delta.md]
input-hash: "f01604c"
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

# Behavioral Contract BC-2.05.010: New ClientToServer IPC Variants — SpawnSession, KillSession, KeyInput, ResizePane, DetachSession, RenameSession, AttachSession

## Description

v1A adds seven `ClientToServer` IPC message variants for session lifecycle operations and
re-attach. The TUI sends these messages to the daemon's UDS server, which routes them to
`SessionManager`. Each variant is `#[non_exhaustive]` per BC-2.02.003. The daemon handles
each variant by delegating to the corresponding `SessionManager` method and broadcasting
state updates to all TUI clients. `AttachSession` (7th variant, I3-004) was added in v1.2.0
to support TUI-initiated re-attach after `PtyReset` and explicit user re-attach of Detached
sessions. The TUI MUST NOT send `DaemonToHost::Attach` directly — that is a daemon-only
message; `ClientToServer::AttachSession` is the correct TUI→daemon message.

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
4. On failure (session not found or dead): `ServerToClient::Error { code: "session_not_found", message: ... }` sent to requesting client.

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
3. On success: `ServerToClient::SessionListUpdate` broadcast. NOTE: `SessionStateChanged` is
   NOT emitted for rename — rename is not a `SessionState` transition (per BC-2.08.008 PC-4a).

### AttachSession

1. `ClientToServer::AttachSession { session_id: String }` is received.
2. Daemon calls `SessionManager::attach_session(&session_id)` (BC-2.08.007).
3. On success: the session-host streams a fresh `HostToDaemon::ScrollbackChunk*` +
   `HostToDaemon::ScrollbackDumpComplete` sequence; the daemon fans these out as
   `ServerToClient::ScrollbackChunk` / `ServerToClient::ScrollbackDumpComplete` to all
   connected TUI clients (per BC-2.05.011). If state transitions from `Detached → Running`,
   `ServerToClient::SessionStateChanged { session_id, new_state: Running }` is broadcast
   BEFORE `ServerToClient::SessionListUpdate`.
4. On failure (session not found, session-host dead): `ServerToClient::Error { code: "attach_failed", message: ... }` sent to the requesting client.
5. Use cases: (a) TUI re-attach after `PtyReset` (BC-2.05.011 PC-3c), (b) user explicitly
   re-attaches a `Detached` session from the sessions panel. The TUI MUST NOT send
   `DaemonToHost::Attach` directly — that is a daemon→session-host message. The TUI sends
   `ClientToServer::AttachSession` to the daemon, which routes to `SessionManager::attach_session()`.
   Per SS-ipc.md v1.15.0 §`ClientToServer::AttachSession`.

## Invariants

1. All seven variants are `#[non_exhaustive]` fields per ADR-0006 (non-exhaustive structs with
   public constructors). The message enum itself is `#[non_exhaustive]` per BC-2.02.003.
2. `KeyInput` and `ResizePane` are high-frequency messages. The IPC handler MUST process
   them with minimal latency (no locking beyond `Arc<Mutex<SessionManager>>.lock()`).
3. `session_id` is a `String` (UUID rendered as string) at all IPC boundaries per
   SS-session-manager.md §session_id type ruling.
4. Unknown variants (future additions from newer TUI to older daemon) are silently ignored
   per `#[non_exhaustive]` forward-compat policy.
5. **Zero-dimension clamp (S2-004 consistency rule):** The daemon IPC handler MUST clamp
   `ResizePane.rows` and `ResizePane.cols` to a minimum of 1 before forwarding to
   `resize_session()`. This rule is applied at the daemon boundary (not in the TUI). The TUI
   (BC-2.09.006) should also prevent sending zero dimensions, but the daemon is the final
   enforcement point. Clamping prevents undefined PTY behavior without surfacing an error to
   the TUI. Cross-reference: BC-2.09.006 EC-237 (TUI-side resize no-op detection).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-280 | `SpawnSession` with a `SpawnRecipe` where `binary` is empty path | `SessionError::SpawnFailed`; `ServerToClient::Error { code: "spawn_failed", message: ... }` sent back |
| EC-281 | `KeyInput` for unknown `session_id` | `SessionError::SessionNotFound`; `ServerToClient::Error { code: "session_not_found", message: ... }` sent to requesting client |
| EC-282 | `ResizePane` with `rows=0` or `cols=0` | The daemon's IPC handler MUST clamp each dimension to a minimum of 1 BEFORE forwarding to `resize_session()`. `rows = max(rows, 1); cols = max(cols, 1)`. The PTY and parser are resized to the clamped values. No `SessionError` is returned; the operation succeeds with clamped dimensions. Clamping is consistent with BC-2.09.006 EC-237 and the resize behavior in the TUI. A zero-dimension PTY is undefined by POSIX; clamping to 1 is the most robust behavior. |
| EC-283 | `RenameSession` with empty `new_name` | `SessionError::InvalidSessionName`; `ServerToClient::Error { code: "invalid_request", message: ... }` sent to requesting client |
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
| VP-TBD | All 7 variants routed to correct `SessionManager` methods (incl. AttachSession → attach_session()) | integration |
| VP-TBD | `KeyInput` and `ResizePane` generate no broadcast (fire-and-forget) | unit |
| VP-TBD | Unknown `ClientToServer` variant handled without panic | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability §SS-05 |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability — these ClientToServer variants extend the internal transport capability with session lifecycle control messages (spawn, kill, key input, resize, detach, rename, re-attach) — all transported over the existing UDS per the session/event/prompt push design |
| Architecture Module | monocle-ipc (`ClientToServer` enum new variants); monocle-runtime (IPC handler routing to SessionManager) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-daemon-wiring-v2-delta.md v1.5.0 §IPC handler — new ClientToServer variants (including AttachSession); SS-ipc.md v1.15.0 §`ClientToServer::AttachSession` (I3-004 — TUI re-attach; replaces incorrect "TUI sends DaemonToHost::Attach" description); SS-ipc.md v1.15.0+ §`ServerToClient::Error` — Error variant + code taxonomy (`spawn_failed`, `session_not_found`, `attach_failed`, `kill_failed`, `rename_failed`, `invalid_request`) added by architect in Pass-6 parallel track (C6-001) |
| Cross-Ref | BC-2.08.001 (SpawnSession → spawn_session()); BC-2.08.003 (KillSession → kill_session()); BC-2.08.007 (DetachSession → detach_session()) |
| Test Name | test_BC_2_05_010_new_client_to_server_variants_routed |

## Related BCs

- [BC-2.08.001] — depends on: SpawnSession IPC triggers spawn_session()
- [BC-2.08.003] — depends on: KillSession IPC triggers kill_session()
- [BC-2.08.007] — depends on: DetachSession IPC triggers detach_session(); AttachSession IPC triggers attach_session()
- [BC-2.05.002] — composes with: existing IPC connection framework carries these new variants
- [BC-2.05.011] — depends on: AttachSession triggers ScrollbackChunk*/ScrollbackDumpComplete sequence fanned out by BC-2.05.011

## Architecture Anchors

- `architecture/SS-daemon-wiring-v2-delta.md#ipc-handler-new-clienttoserver-variants` — handler branches

## Story Anchor

S-TBD — Implement new ClientToServer IPC variants and daemon routing (filled by story-writer)

## VP Anchors

VP-TBD — IPC variant routing integration tests (filled after VP creation)

## §Trace v1.4.0

**S-P7-003 — Add AttachSession to H1 title for H1↔body consistency** (2026-06-03):
- H1 title listed 6 variants but Description, Invariant 1, and body all specify seven variants
  (AttachSession was added in v1.2.0 per I3-004 but the H1 was not updated at that time).
- H1 title corrected to enumerate all 7 variants: appended "AttachSession" to the comma-separated
  variant list. BC-INDEX row updated to match.
- No content change; cosmetic H1 consistency fix only.

## §Trace v1.3.0

**Pass-6 C6-001 — Align error codes with SS-ipc.md v1.14.0 architect taxonomy** (2026-06-03):
- C6-001: Architect is adding `ServerToClient::Error { code: String, message: String }` to SS-ipc.md
  (v1.14.0) with canonical code strings: `spawn_failed`, `session_not_found`, `attach_failed`,
  `kill_failed`, `rename_failed`, `invalid_request`.
- **KeyInput-PC-4:** Added explicit `code: "session_not_found"` — previously said only
  `ServerToClient::Error` with no code string. `session_not_found` is the correct code for
  unknown or dead session (matches architect taxonomy + SessionError::SessionNotFound).
- **EC-280:** Added `code: "spawn_failed"` to `ServerToClient::Error` — previously omitted code string.
- **EC-281:** Added `code: "session_not_found"` to `ServerToClient::Error` — previously omitted code string.
- **EC-283:** Added `code: "invalid_request"` to `ServerToClient::Error` — previously omitted code string.
  `rename_failed` in the architect's taxonomy covers operational rename failures (e.g., session-host
  unreachable); `invalid_request` covers input-validation failures such as an empty `new_name`.
  EC-283 is a validation failure, so `invalid_request` is the correct code.
- **Architecture Source:** Added SS-ipc.md v1.14.0+ citation for the Error variant and code taxonomy.
  (v1.13.0 does NOT define `ServerToClient::Error`; the architect adds it in Pass-6 in parallel.)

## §Trace v1.2.0

**Adversarial Pass 3 fix — I3-004 (AttachSession 7th variant per SS-ipc.md v1.13.0)** (2026-06-03):
- I3-004: `ClientToServer::AttachSession { session_id }` added as the 7th variant. Daemon
  routes to `SessionManager::attach_session()`. Used for: (a) TUI re-attach after PtyReset
  (replaces the incorrect BC-2.05.011 PC-3c reference to "TUI sends DaemonToHost::Attach"
  — the TUI cannot send DaemonToHost messages), (b) explicit user-initiated re-attach of a
  Detached session from the sessions panel. Per SS-ipc.md v1.13.0 §ClientToServer::AttachSession.
- Invariant 1: updated "six" → "seven" variants.
- Description: updated to seven variants; clarified TUI must NOT send DaemonToHost::Attach.
- Architecture Source updated to SS-daemon-wiring-v2-delta.md v1.3.1 and SS-ipc.md v1.13.0.

## §Trace v1.1.0

**S2-004 adversarial pass-2 fix — zero-dimension clamp at daemon boundary** (2026-06-03):
- S2-004 finding: EC-282 said "if PTY rejects zero dimensions, `SessionError` returned" — this
  was inconsistent with BC-2.09.006 which has no zero-dimension handling and simply forwards
  the resize. Two BCs with different behaviors for the same condition is production-grade non-
  conformant. Resolution: clamp-to-1 at the daemon boundary, no error returned.
- EC-282: rewritten with clamp-to-1 rule at daemon IPC handler. No SessionError. Clamped values
  forwarded to resize_session(). Rationale: zero-dimension PTY is POSIX-undefined; clamping
  is more robust than rejecting (avoids unnecessary error handling in the TUI while preventing
  undefined behavior in the PTY).
- Invariant 5 added: zero-dimension clamp rule as a production-grade enforcement point at the
  daemon boundary. Cross-referenced with BC-2.09.006 EC-237 (TUI-side no-op detection).

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.05.010 authored for SS-05 as part of the v1A control-center pivot BC burst.
- Covers all 6 new ClientToServer variants from SS-daemon-wiring-v2-delta.md.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
