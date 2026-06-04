---
document_type: behavioral-contract
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T04:00:00Z
phase: phase-1-expansion
inputs: [prd-expansion-scope.md, architecture/SS-ipc.md, architecture/ARCH-INDEX.md]
input-hash: "73990b1"
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

# Behavioral Contract BC-2.05.003: IPC Message Types: SessionListUpdate

## Description

When the session roster changes in the daemon (a session is added, removed, or enriched with
updated metadata), the daemon broadcasts a `ServerToClient::SessionListUpdate` message to all
currently connected TUI clients. The message contains the full current session list — not a
diff. Each TUI client replaces its local session list on receipt. The message is framed using
the standard 4-byte LE length-prefix protocol (BC-2.05.002).

## Preconditions

1. At least one TUI client is connected to the daemon's UDS socket.
2. The `EngineModule` registry has reported a session change: a new session was detected,
   an existing session ended, or session metadata was enriched (token count, cost, phase tag).
3. The daemon event bus has delivered the session-change event to the IPC fan-out task.

## Postconditions

1. The daemon serializes and sends `ServerToClient::SessionListUpdate { sessions: Vec<EnrichedSession> }`
   to every currently connected TUI client via the fan-out subscriber list.
2. The `sessions` field contains the **complete current session list** at the time of emission.
   It is not a diff; a TUI client that receives this message replaces its entire local session
   roster with the contents of this field.
3. The `SessionListUpdate` message is framed with the standard 4-byte LE length-prefix protocol.
   If the serialized message exceeds 256 KiB, the daemon logs `ERROR: SessionListUpdate exceeds
   256 KiB; cannot broadcast` and does NOT send the message to any client. (In practice, a
   256 KiB session list implies thousands of simultaneous sessions — not a Phase 1 scenario.)
4. TUI clients that connect after the `SessionListUpdate` is sent receive the current session
   list in the `InitialState` push (BC-2.05.002 Postcondition 2), not a replay of
   `SessionListUpdate` messages.
5. A disconnected TUI client is removed from the fan-out subscriber list before the broadcast.
   `SessionListUpdate` is never sent to a disconnected client.
6. The `EnrichedSession` struct carries `#[non_exhaustive]` per BC-2.02.003. TUI clients that
   encounter unknown fields in a future `EnrichedSession` version must ignore them (forward-compat).

## Invariants

1. The session list delivered in `SessionListUpdate` is always a consistent snapshot of the
   daemon's current `DaemonState.sessions` map at the time the fan-out task emits it. No
   partial updates are delivered.
2. `SessionListUpdate` is sent for every session change, including session enrichment (token
   count updates, phase tag changes). The TUI always has an up-to-date session list without
   polling.
3. The order of sessions in the `sessions` Vec is determined by session discovery time
   (insertion order of the daemon's session registry). The TUI must not assume alphabetical
   or any other ordering.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All sessions end simultaneously (e.g., CTRL-C on all harness processes) | `SessionListUpdate { sessions: [] }` sent to all clients. TUI renders "No sessions detected". |
| EC-002 | Session enrichment arrives in rapid succession (e.g., token count updated every 100ms) | Each enrichment triggers a `SessionListUpdate`. Under high frequency, the daemon may coalesce updates (debounce policy TBD by architect). Each update is still a complete session list snapshot. |
| EC-003 | No TUI clients are connected when a session change occurs | Daemon updates internal state but sends no messages (empty subscriber list). Next connecting TUI receives the updated list in `InitialState`. |
| EC-004 | One TUI client disconnects while a `SessionListUpdate` broadcast is in progress | The fan-out task skips the disconnected client (closed send half). The other clients receive the message normally. No panic or partial broadcast. |
| EC-005 | Session has `EnrichedSession` fields added in a future daemon version | TUI clients compiled against an older `EnrichedSession` struct ignore unknown fields per `#[non_exhaustive]` + serde `deny_unknown_fields: false` default. Existing fields render correctly. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Daemon detects a new Claude Code session | `SessionListUpdate { sessions: [<new session>] }` broadcast to all connected clients | happy-path |
| Existing session ends (process exits) | `SessionListUpdate { sessions: [] }` (if it was the only session) or `sessions: [<remaining>]` | happy-path |
| Session enriched with updated token count | `SessionListUpdate { sessions: [<session with updated token count>] }` sent; TUI re-renders row | happy-path |
| No clients connected when session changes | No message sent; next client connection receives updated list in `InitialState` | edge-case |
| Two clients connected; one disconnects mid-broadcast | Remaining client receives `SessionListUpdate`; no error for disconnected client | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `SessionListUpdate` is sent to all connected clients on session roster change | integration |
| VP-TBD | `sessions` field contains the complete current roster (not a diff) | unit |
| VP-TBD | Disconnected clients are not in the fan-out list; no error on skip | integration |
| VP-TBD | `SessionListUpdate` with empty sessions Vec correctly broadcast when all sessions end | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability §SS-05 — this BC specifies the session list push that is the primary mechanism for the TUI to display live session state |
| L2 Domain Invariants | DI-006 (EngineModule detect() must not perform I/O — session detection happens in EngineModule::detect() per BC-2.03.002; this BC governs how the detection result is broadcast to TUI clients, not the detection itself; DI-006 holds at the detection boundary) |
| Architecture Module | monocle-ipc (ServerToClient::SessionListUpdate, fan-out broadcaster) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-ipc.md v1.17.0 §Message Types §Server-to-Client Messages; SS-ipc.md v1.17.0 §Connection Lifecycle §Phase 2 Streaming Updates |
| Cross-Ref | BC-2.05.002 (InitialState contains the initial session list; this BC governs incremental updates); BC-2.02.003 (non-exhaustive enum policy applies to EnrichedSession) |
| Test File | `monocle-ipc/tests/session_list_update.rs` |
| Test Name | `test_BC_2_05_003_session_list_update_broadcast` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.002] — composes with: InitialState contains first session list; this BC delivers incremental changes
- [BC-2.02.003] — depends on: EnrichedSession must be `#[non_exhaustive]` for forward-compat
- [BC-2.03.001] — depends on: EngineModule trait defines how sessions are detected and registered

## Architecture Anchors

- `architecture/SS-ipc.md#message-types` — `ServerToClient::SessionListUpdate` enum variant definition
- `architecture/SS-ipc.md#connection-lifecycle` — Phase 2: Streaming Updates (SessionListUpdate trigger condition)

## Story Anchor

S-TBD — Implement SessionListUpdate IPC fan-out broadcast (filled by story-writer)

## VP Anchors

VP-TBD — SessionListUpdate broadcast verification properties (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T04:00:00Z):
- BC-2.05.003 authored for SS-05 IPC subsystem per `prd-expansion-scope.md §3.2` and
  `SS-ipc.md §Message Types + §Connection Lifecycle §Phase 2 Streaming Updates`.
- Covers: SessionListUpdate message type, full-list (not diff) semantics, fan-out broadcast,
  disconnected-client skip, empty-list case, forward-compat via non-exhaustive EnrichedSession.
- 5 edge cases documented (EC-001..EC-005).
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

## §Trace v1.0.4

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-ipc.md v1.4.0 → v1.9.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-ipc.md v1.4.0 §Message Types §Server-to-Client Messages` → `SS-ipc.md v1.9.0 §Message Types §Server-to-Client Messages`; `SS-ipc.md v1.4.0 §Connection Lifecycle §Phase 2 Streaming Updates` → `SS-ipc.md v1.9.0 §Connection Lifecycle §Phase 2 Streaming Updates`.
- Plain version-pin refresh. No substantive content propagation required — §Message Types and §Connection Lifecycle §Phase 2 Streaming Updates section headings and content anchors are unchanged between v1.4.0 and v1.9.0.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.
