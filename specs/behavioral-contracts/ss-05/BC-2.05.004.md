---
document_type: behavioral-contract
level: L3
version: "1.1.0"
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

# Behavioral Contract BC-2.05.004: IPC Message Types: HookEventReceived

## Description

When the daemon ingests a hook event (via any of the 5 hook endpoints), it broadcasts a
`ServerToClient::HookEventReceived` message to all connected TUI clients. The message carries
the hook type, session ID, a bounded payload excerpt (first 256 bytes), and the observed
latency from HTTP receipt to daemon ACK in milliseconds. This is the primary mechanism by
which the TUI's event ribbon panel is populated with live hook event data.

## Preconditions

1. At least one TUI client is connected to the daemon's UDS socket.
2. The daemon has received and ACK'd a hook HTTP POST (any of the 5 endpoints: PreToolUse,
   Notification, Stop, SessionStart, UserPromptSubmit).
3. The hook event has been written to the JSONL ring per DI-001 (durable before broadcast).
4. The bounded event bus has delivered the hook event to the IPC fan-out task
   (BC-2.04.011 bounded event bus; if the bus is full, the event is dropped and
   `drop_counter` increments — in that case, `DropCounterUpdate` is sent instead of
   `HookEventReceived` for the dropped event).

## Postconditions

1. The daemon serializes and sends `ServerToClient::HookEventReceived` to every currently
   connected TUI client. The message fields are:
   - `hook_type: HookType` — the discriminant of the hook that was received.
   - `session_id: String` — the session identifier from the hook POST body.
   - `payload_excerpt: String` — the **first 256 bytes** (UTF-8 characters; truncated at a
     character boundary to avoid splitting multi-byte sequences) of the hook POST body JSON.
     This excerpt keeps the message size bounded. The full payload is in the JSONL ring.
   - `latency_ms: u64` — the time in milliseconds from when the daemon received the HTTP POST
     request to when it sent the HTTP ACK response to the hook caller.
   - `timestamp_micros: i64` — the daemon-owned event time as Unix epoch microseconds (signed).
     The daemon hook handler MUST populate this from the same clock capture used to write
     `HookEventRecord::timestamp_micros` to the JSONL ring — not a separate `SystemTime::now()`
     call. The TUI event ribbon MUST display this value as the wall-clock event time; it MUST
     NOT substitute `SystemTime::now()` at receipt time (which would show IPC transit latency
     rather than the real event time). Added in BC v1.1.0 / SS-ipc.md v1.10.0 (S-028 ADR). <!-- version-pin-historical: records the SS-ipc version at which timestamp_micros was introduced per S-028 ADR decision -->
2. The `timestamp_micros` value in `HookEventReceived` and the `timestamp_micros` value in the
   corresponding `HookEventRecord` written to the JSONL ring MUST be equal. They are derived from
   the same clock capture — the daemon MUST NOT take two separate clock readings.
3. The `HookEventReceived` message is framed with the standard 4-byte LE length-prefix protocol.
4. Disconnected TUI clients are not in the fan-out subscriber list; no message is sent to them.
5. The `drop_counter` is NOT incremented by sending a `HookEventReceived` IPC message to TUI
   clients. The drop counter only increments when the bounded event bus (daemon side) drops an
   event before it reaches the fan-out task. Once an event reaches the fan-out task, it is
   always delivered to connected clients (per-client send buffers are bounded by the OS socket
   buffer; a slow TUI client may cause a full send buffer, resulting in a per-client disconnect).
6. The `HookType` enum carries `#[non_exhaustive]` per BC-2.02.003. TUI clients that receive
   an unknown `HookType` variant (future hook type not present in their compiled version) must
   render a safe fallback label (e.g., "unknown") rather than panicking.

## Invariants

1. The `payload_excerpt` is always bounded to 256 bytes or fewer. No `HookEventReceived`
   message can grow the IPC wire size unboundedly due to a large hook POST body.
2. The `latency_ms` field measures wall-clock time from HTTP POST receipt to HTTP ACK. It
   does NOT include TUI rendering time. The event ribbon's latency column displays this value.
3. `HookEventReceived` is broadcast for every hook event that passes through the fan-out task,
   including events from all 5 hook types (PreToolUse, Notification, Stop, SessionStart,
   UserPromptSubmit). There is no filtering at the IPC layer; the TUI may filter for display.
4. The `timestamp_micros` field in `HookEventReceived` MUST equal the `timestamp_micros` field
   in the corresponding `HookEventRecord` written to the JSONL ring for the same hook event.
   A single clock capture is made by the hook handler; both fields are populated from that
   capture. The TUI event ribbon uses `timestamp_micros` as the displayed event time. Using
   `SystemTime::now()` at TUI receipt time is forbidden for the ribbon timestamp display.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Hook POST body is empty (zero bytes) | `payload_excerpt` is an empty string `""`. `latency_ms` still reflects the round-trip time. |
| EC-002 | Hook POST body is 512 bytes (larger than 256-byte excerpt limit) | `payload_excerpt` contains the first 256 bytes of the body, truncated at a valid UTF-8 character boundary. No data after byte 256 appears in the excerpt. |
| EC-003 | Hook POST body is valid UTF-8 but the 256th byte is mid-way through a 3-byte sequence | Truncation happens at the last complete character before the 256-byte boundary. The excerpt is valid UTF-8 and ≤256 bytes. |
| EC-004 | Bounded event bus is full when a hook event arrives | The event is dropped; `DaemonState.drop_counter` increments by 1; a `DropCounterUpdate` IPC message is sent to TUI clients. No `HookEventReceived` is sent for the dropped event. |
| EC-005 | TUI client's send buffer is full (slow TUI client) | Daemon detects send error; removes slow client from fan-out subscriber list; closes the per-client connection; logs `WARN: removed slow TUI client (send buffer full)`. Other clients are unaffected. |
| EC-006 | `HookType` value in hook body is a future variant unknown to the TUI | TUI's `match hook_type { _ => render "unknown" }` arm handles the unknown variant. No panic. `#[non_exhaustive]` enforces catch-all arm at compile time. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| PreToolUse hook POST (50-byte body) arrives; 1 TUI client connected | `HookEventReceived { hook_type: PreToolUse, session_id: "abc", payload_excerpt: <50 bytes>, latency_ms: <measured> }` sent to client | happy-path |
| Notification hook POST (300-byte body) arrives | `payload_excerpt` is first 256 bytes of body (truncated at char boundary); full body in JSONL ring | happy-path |
| Hook POST arrives when no TUI clients connected | No IPC message sent; event written to JSONL ring; internal state updated | edge-case |
| Event bus full when hook arrives | `DropCounterUpdate` sent; no `HookEventReceived` for dropped event | edge-case |
| Slow TUI client causes send buffer fill | Slow client disconnected; other client still receives `HookEventReceived` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `HookEventReceived` broadcast to all connected clients on each hook ingestion | integration |
| VP-TBD | `payload_excerpt` always ≤256 bytes and valid UTF-8 | unit |
| VP-TBD | `latency_ms` reflects wall-clock HTTP receipt → ACK time | integration |
| VP-TBD | Dropped events (full bus) produce `DropCounterUpdate`, not `HookEventReceived` | integration |
| VP-TBD | Slow client disconnect does not affect other clients | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability §SS-05 — this BC specifies the hook event push that populates the TUI's event ribbon panel, which is the live-event-stream component of the internal transport |
| L2 Domain Invariants | DI-001 (every hook event must be written to the JSONL ring before ACK — Precondition 3 requires ring write before IPC broadcast; this BC's fan-out happens after DI-001 is satisfied) |
| Architecture Module | monocle-ipc (ServerToClient::HookEventReceived, fan-out broadcaster) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-ipc.md v1.17.0 §Message Types §Server-to-Client Messages; SS-ipc.md v1.17.0 §Connection Lifecycle §Phase 2 Streaming Updates |
| Cross-Ref | BC-2.01.003 (256 KiB body limit at HTTP layer; this BC's 256-byte excerpt is the IPC-layer bounding, not the HTTP-layer limit); BC-2.04.011 (bounded event bus — drop counter increments on bus-full; this BC's fan-out only sees events that cleared the bus); BC-2.02.003 (non-exhaustive HookType enum) |
| Test File | `monocle-ipc/tests/hook_event_received.rs` |
| Test Name | `test_BC_2_05_004_hook_event_received_broadcast` |
| Stories | S-021 (IPC message types + struct definition), S-032 (daemon producer path: fan-out broadcast + timestamp_micros PC-2/INV-4) |

## Related BCs

- [BC-2.05.002] — composes with: HookEventReceived delivered after InitialState.ring_tail
- [BC-2.04.011] — depends on: bounded event bus delivers events to IPC fan-out; dropped events bypass this BC
- [BC-2.02.003] — depends on: HookType must be `#[non_exhaustive]` per ABI policy

## Architecture Anchors

- `architecture/SS-ipc.md#message-types` — `ServerToClient::HookEventReceived` enum variant with field definitions
- `architecture/SS-ipc.md#connection-lifecycle` — Phase 2: Streaming Updates (HookEventReceived trigger condition)

## Story Anchor

S-021 — IPC message types + ServerToClient::HookEventReceived struct definition (done)
S-032 — Daemon event-bus fan-out: broadcast HookEventReceived with timestamp_micros (BC-2.05.004 PC-2/INV-4; Wave 8 draft)

## VP Anchors

VP-TBD — HookEventReceived broadcast and excerpt-bounding verification properties (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T04:00:00Z):
- BC-2.05.004 authored for SS-05 IPC subsystem per `prd-expansion-scope.md §3.2` and
  `SS-ipc.md §Message Types + §Connection Lifecycle §Phase 2 Streaming Updates`.
- Covers: HookEventReceived message type, 256-byte payload excerpt bounding, latency_ms
  semantics, fan-out broadcast, drop-counter interaction with bounded event bus, slow-client
  disconnect, non-exhaustive HookType forward-compat.
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

## §Trace v1.1.0

**S-028 ADR — `timestamp_micros` field added to `HookEventReceived`** (2026-06-01):
- Adversarial review for S-028 identified that `ServerToClient::HookEventReceived` carried
  no daemon-side timestamp. BC-2.05.004 PC-1 lists the event ribbon display fields — none
  included a real event time — meaning the TUI would be forced to substitute
  `SystemTime::now()` at receipt time, showing IPC transit delay rather than the actual event
  time. This is a correctness defect, not a future enhancement.
- **PC-1 update:** Added `timestamp_micros: i64` to the message field list. The field type
  matches `HookEventRecord::timestamp_micros` (signed Unix epoch micros per
  SS-core-types-and-abi.md §HookEventRecord). Daemon-side and TUI-side obligations documented.
- **PC-2 added (new):** `timestamp_micros` in `HookEventReceived` MUST equal `timestamp_micros`
  in the corresponding `HookEventRecord`. Both populated from single clock capture.
- **Postconditions renumbered:** Original PC-2 → PC-3; PC-3 → PC-4; PC-4 → PC-5; PC-5 → PC-6.
- **Invariant 4 added:** Invariant that `timestamp_micros` MUST equal ring record value;
  `SystemTime::now()` at receipt forbidden for ribbon timestamp.
- **Architecture Source:** `SS-ipc.md v1.9.0` → `SS-ipc.md v1.10.0`.
- SE-16d monotonicity: v1.1.0 timestamp 2026-06-01 > v1.0.4 timestamp 2026-05-29. PASS.

## §Trace v1.1.1

**Story backlink update — S-032 anchored as daemon producer story** (2026-06-01):
- Traceability §Stories row: `S-TBD` → `S-021 (types), S-032 (daemon producer)`.
- Story Anchor section: filled with S-021 (done) + S-032 (Wave 8 draft).
- S-032 discharges the orphaned BC-2.05.004 v1.1.0 PC-2/INV-4 obligation surfaced
  during S-028 adversarial review. Backlink established per story-writer BC Backlink
  Update Obligation.
- SE-16d monotonicity: v1.1.1 timestamp 2026-06-01 >= v1.1.0 timestamp 2026-06-01. PASS (same-day).

## §Trace v1.0.4

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-ipc.md v1.4.0 → v1.9.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-ipc.md v1.4.0 §Message Types §Server-to-Client Messages` → `SS-ipc.md v1.9.0 §Message Types §Server-to-Client Messages`; `SS-ipc.md v1.4.0 §Connection Lifecycle §Phase 2 Streaming Updates` → `SS-ipc.md v1.9.0 §Connection Lifecycle §Phase 2 Streaming Updates`.
- Plain version-pin refresh. No substantive content propagation required — §Message Types and §Connection Lifecycle §Phase 2 Streaming Updates section headings and content anchors are unchanged between v1.4.0 and v1.9.0.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.
