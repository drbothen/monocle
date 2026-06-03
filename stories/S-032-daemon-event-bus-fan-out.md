---
document_type: story
level: L4
story_id: S-032
epic_id: EPIC-05
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-01T14:00:00Z
phase: 2
points: 5
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-021, S-022, S-028]
blocks: []
target_module: monocle-runtime
subsystems: [SS-04, SS-05]
behavioral_contracts: [BC-2.05.004]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.004.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md, version: "1.0.7"}
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.011.md, version: "1.4.0"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.10.0"}
  - {path: .factory/specs/architecture/SS-daemon-wiring.md, version: "1.3.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.0"}
input-hash: "[pending]"
traces_to: "Discharges deferred daemon-side obligation from S-028 adversarial review: implements BC-2.05.004 PC-1/PC-2/INV-4 (daemon broadcasts ServerToClient::HookEventReceived with timestamp_micros equal to ring HookEventRecord::timestamp_micros; single clock capture). Un-stubs event_bus_fan_out_task in monocle-runtime/src/event_bus.rs."
# Deferred-from: S-028 adversarial review found that BC-2.05.004 v1.1.0 PC-2 / INV-4
# (daemon-side timestamp_micros obligation) was unowned. S-021 implemented the IPC types
# only; S-028 is TUI-consumer-only. This story owns the daemon producer path.
# BC status: BC-2.05.004 behavioral_contracts array is non-empty. status must remain
# draft until dispatch; do NOT set to ready without PO confirmation.
---

# S-032: Daemon Event-Bus Fan-Out: Broadcast HookEventReceived with daemon timestamp_micros

## Narrative

As the monocle daemon, I want `event_bus_fan_out_task` to broadcast
`ServerToClient::HookEventReceived` to all connected TUI IPC clients for each hook event that
clears the bounded event bus, populating the `timestamp_micros` field from the SAME clock
capture used to write `HookEventRecord::timestamp_micros` to the JSONL ring, so that the TUI
event ribbon displays accurate, daemon-stamped event times rather than IPC-transit-inflated
receipt-time estimates.

## Context: Deferred Daemon Obligation from S-028 Adversarial Review

During adversarial review for S-028 (Sessions Panel Nucleo Filter + Event Ribbon), BC-2.05.004
was amended to v1.1.0 — adding PC-2 and Invariant 4 which require that the daemon's
`HookEventReceived` broadcast carry a `timestamp_micros` field sourced from the same clock
capture as the ring's `HookEventRecord::timestamp_micros` (single-capture equality invariant).

At the time of that amendment, the daemon-side producer path was unowned:
- S-021 (done) implemented the UDS server, IPC message types, and initial `HookEventReceived`
  struct definition — but it was written against BC-2.05.004 before v1.1.0, before `timestamp_micros`
  was added.
- S-028 (Wave 7, TUI-consumer) correctly consumes `HookEventReceived.timestamp_micros` but
  owns no daemon code.
- `event_bus_fan_out_task` in `monocle-runtime/src/event_bus.rs` remains a Phase-1 stub that
  receives events and discards them (comment: "Phase 1: TUI IPC clients not yet wired
  (S-021/S-022 scope)").

This story makes the streaming event ribbon path live end-to-end and satisfies BC-2.05.004
v1.1.0 PC-2 / INV-4 at the daemon producer.

## Acceptance Criteria

### AC-001 (traces to BC-2.05.004 postcondition PC-1 — daemon broadcasts HookEventReceived to all connected TUI clients)
When a hook event exits the bounded event bus (i.e., `EventBusRx::recv()` yields
`Some(event)` in `event_bus_fan_out_task`), the daemon serializes and sends
`ServerToClient::HookEventReceived { hook_type, session_id, payload_excerpt, latency_ms,
timestamp_micros }` to **every** entry in `SubscriberList` that has not been removed. The
broadcast iterates the subscriber list and attempts a bounded send to each per-client
`mpsc::Sender<ServerToClient>`. Per-client write timeout remains 50ms per BC-2.04.011 PC-5;
slow or disconnected clients are removed from `SubscriberList` without stalling the fan-out
loop.

### AC-002 (traces to BC-2.05.004 postcondition PC-2 — timestamp_micros equals ring record timestamp)
The `timestamp_micros` value in `HookEventReceived` MUST equal the `timestamp_micros` value
in the corresponding `HookEventRecord` that was written to the JSONL ring for the same hook
event. Both fields are derived from the single `SystemTime::now()` → epoch-micros clock
capture made by the hook handler before ring write. The `EventBusHookEvent` struct carries
this timestamp from handler to fan-out task (either inline or via a new
`timestamp_micros: i64` field added to `EventBusHookEvent`). The fan-out task reads this
carried value directly — it MUST NOT call `SystemTime::now()` again.

### AC-003 (traces to BC-2.05.004 invariant INV-4 — single clock capture; SystemTime::now() at fan-out forbidden)
Exactly ONE call to `SystemTime::now()` (or equivalent `chrono::Utc::now()`) is made per
hook event lifecycle: in the HTTP handler, before the ring write. The clock value is
propagated through `EventBusHookEvent` to the fan-out task. The fan-out task MUST NOT
contain any call to `SystemTime::now()` for timestamp purposes. A test MUST verify the
equality of `HookEventReceived.timestamp_micros` and `HookEventRecord.timestamp_micros`
using a test-injected clock, ensuring they share the same capture.

### AC-004 (traces to BC-2.05.004 postcondition PC-1 — no broadcast when zero clients connected)
When `SubscriberList` is empty at fan-out time, no IPC send is attempted and no error is
logged. The event is consumed without side effects. This is the existing Phase 1 behavior
(stub comment: "no TUI clients in Phase 1") formalized as a tested invariant — the stub
comment and `#[allow(dead_code)]` annotation on `FAN_OUT_CLIENT_TIMEOUT_MS` are removed.

### AC-005 (traces to BC-2.04.011 postcondition PC-5 — slow client removal; 50ms per-client timeout)
Each per-client IPC send in the fan-out loop is wrapped in
`tokio::time::timeout(Duration::from_millis(50), sender.send(msg))`. On timeout or
`SendError`, the client sender is removed from `SubscriberList` and the fan-out continues
to the next client. The removed client is not present in subsequent fan-out iterations.

### AC-006 (traces to BC-2.05.004 postcondition PC-3 — standard framing preserved)
`HookEventReceived` is transmitted using the standard 4-byte LE length-prefix framing
protocol (`write_framed` from `monocle_ipc::framing`). No special-casing for this variant.
The per-client sender channel (`mpsc::Sender<ServerToClient>`) feeds the existing per-client
write task that calls `write_framed` — no new framing logic is required in the fan-out task
itself.

### AC-007 (traces to BC-2.05.004 postcondition PC-4 — DropCounterUpdate when event bus drops event)
When `try_publish_event` drops an event due to a full bus (`TrySendError::Full`), the
fan-out task is NOT invoked for that event (it never enters the channel). `drop_counter`
increments and `DropCounterUpdate` debounce behavior remains unchanged (BC-2.04.011 PC-8).
A test MUST verify that no `HookEventReceived` is emitted for a bus-dropped event.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `event_bus_fan_out_task` | `monocle-runtime/src/event_bus.rs` | Effectful (tokio task, IPC writes) |
| `EventBusHookEvent` | `monocle-runtime/src/types.rs` | Pure (data struct; add `timestamp_micros: i64` field) |
| `SubscriberList` | `monocle-ipc/src/server.rs` | Effectful (shared state, Arc<Mutex<...>>) |
| `ServerToClient::HookEventReceived` | `monocle-ipc/src/types.rs` | Pure (data enum variant; add `timestamp_micros: i64` field) |
| Hook HTTP handlers | `monocle-runtime/src/router.rs` (hook handler fns) | Effectful (HTTP handlers; single clock capture here) |

## UX Screens

N/A — daemon-only change; no TUI rendering in scope (rendering is owned by S-028).

## Dependencies

### Depends on
- **S-021** (done): UDS server bind, `SubscriberList` and `ServerToClient::HookEventReceived`
  struct definition — the IPC substrate this story activates.
- **S-022** (done): TUI client connect + initial state push — establishes the per-client
  sender channel that the fan-out task writes to.
- **S-028** (Wave 7): TUI consumer of `HookEventReceived.timestamp_micros` — confirms the
  consumer contract before the producer is wired. This story MUST NOT be dispatched before
  S-028 is in the `done` state, because S-028's adversarial review is the source of the
  BC-2.05.004 v1.1.0 amendment that this story discharges.

### Blocks
None — this story delivers a standalone daemon-side capability. The end-to-end event ribbon
path is complete when S-028 + S-032 are both done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No TUI clients connected at fan-out time | No IPC send attempted; no error logged; stub comment removed |
| EC-002 | One of N clients is slow (send takes >50ms) | Slow client removed from SubscriberList; remaining clients receive HookEventReceived |
| EC-003 | All clients disconnected before fan-out loop starts | Empty SubscriberList; no sends; no panic |
| EC-004 | Hook POST body is empty (zero bytes) | `payload_excerpt = ""`; `timestamp_micros` still populated from single clock capture |
| EC-005 | Two concurrent hook events arrive; each must carry independent timestamp_micros | Each `EventBusHookEvent` carries its own captured timestamp; no shared mutable state |
| EC-006 | Bus-dropped event (TrySendError::Full) | No HookEventReceived emitted; only DropCounterUpdate sent via debounce task |

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| Story spec (this file) | ~2,500 |
| BC-2.05.004 v1.1.0 | ~1,800 |
| BC-2.05.002 v1.0.7 (ring_tail context) | ~1,200 |
| BC-2.04.011 (event bus + drop counter) | ~1,500 |
| SS-ipc.md v1.10.0 (message types + framing) <!-- version-pin-historical: authored against SS-ipc v1.10.0 at S-032 authoring time --> | ~3,000 |
| SS-daemon-wiring.md v1.3.0 (fan-out task context) | ~2,000 |
| SS-deps-pin-manifest.md (version pins) | ~2,000 | <!-- version-pin-historical: authored against v1.2.0 at S-032 authoring time -->
| `event_bus.rs` (current stub, ~200 lines) | ~800 |
| `types.rs` (EventBusHookEvent, ~50 relevant lines) | ~300 |
| `ipc_server.rs` (SubscriberList usage, ~80 relevant lines) | ~400 |
| `router.rs` (hook handler clock capture site) | ~1,000 |
| Test files (new + existing event_bus tests) | ~2,000 |
| **Total estimate** | **~18,500 tokens** |

Context window headroom: ~18,500 tokens is well within the 20-30% budget cap for a
claude-sonnet-class agent (200k context = 40-60k token story budget). No story split
required.

## Tasks

- [ ] **Task 1:** Add `timestamp_micros: i64` field to `EventBusHookEvent` in
  `monocle-runtime/src/types.rs`. Update `EventBusHookEvent::new(...)` constructor. Verify
  all construction sites populate this from the per-handler clock capture (not a fresh
  `SystemTime::now()`).
- [ ] **Task 2:** Add `timestamp_micros: i64` field to
  `ServerToClient::HookEventReceived` in `monocle-ipc/src/types.rs`. This is an
  additive struct change; all existing serialization round-trip tests must still pass.
- [ ] **Task 3:** Locate the single `SystemTime::now()` clock capture in each of the 5
  hook handlers in `monocle-runtime/src/router.rs` (or equivalent handler modules).
  Confirm the same captured value is propagated through `EventBusHookEvent.timestamp_micros`
  to the ring write. If the capture site and ring write are in the same handler body, this
  is already correct; if not, thread the value through.
- [ ] **Task 4:** Rewrite `event_bus_fan_out_task` in `monocle-runtime/src/event_bus.rs`
  to remove the Phase-1 stub body and implement the production fan-out loop:
  - For each `Some(event)` from `EventBusRx::recv()`: build
    `ServerToClient::HookEventReceived { hook_type, session_id, payload_excerpt, latency_ms,
    timestamp_micros }` from `event` fields; iterate `SubscriberList`; for each subscriber,
    attempt `tokio::time::timeout(50ms, sender.send(msg.clone()))` — on error or timeout,
    remove subscriber from list.
  - Remove the `#[allow(dead_code)]` annotation on `FAN_OUT_CLIENT_TIMEOUT_MS`; it is now
    live.
  - Remove the stub comment "Phase 1: TUI IPC clients not yet wired (S-021/S-022 scope)".
- [ ] **Task 5:** Write integration tests in `monocle-runtime/tests/event_bus.rs`:
  - `test_BC_2_05_004_hook_event_received_broadcast`: spin up a daemon with 2 TUI clients
    connected; POST a hook; assert both clients receive `HookEventReceived` with correct
    `timestamp_micros` matching the ring record.
  - `test_BC_2_05_004_timestamp_micros_equals_ring_record`: inject a test clock; capture
    the timestamp; assert `HookEventReceived.timestamp_micros ==
    HookEventRecord.timestamp_micros` (single-capture equality per INV-4).
  - `test_BC_2_05_004_no_clients_no_panic`: fan-out with empty subscriber list; assert no
    panic and no IPC traffic.
  - `test_BC_2_05_004_slow_client_removed`: insert a client whose send channel is full;
    assert it is removed; assert other clients still receive the message.
  - `test_BC_2_05_004_bus_drop_no_hook_event_received`: fill the event bus; assert
    `DropCounterUpdate` is sent and no `HookEventReceived` is emitted.
- [ ] **Task 6:** Run `cargo test --workspace` and `cargo clippy --workspace --all-targets
  -- -D warnings` to confirm no regressions.

## Previous Story Intelligence

- **S-021** established the `SubscriberList` abstraction in `monocle-ipc/src/server.rs` and
  defined `ServerToClient::HookEventReceived` without `timestamp_micros`. This story adds
  the field additively; the S-021 test suite must still pass after the field addition.
- **S-022** wired the TUI connect path and initial state push. The per-client sender
  channel (`mpsc::Sender<ServerToClient>`) and `spawn_client_task` in
  `monocle-runtime/src/ipc_server.rs` are S-022 outputs that this story's fan-out task
  writes to.
- **S-018** authored `event_bus_fan_out_task` as a Phase-1 stub. The stub contains the
  correct loop skeleton (`loop { match rx.recv().await { None => return, Some(event) =>
  ... } }`) and the correct shutdown contract (exit on channel close, BC-2.04.011 PC-7).
  This story replaces only the `Some(event)` arm body; the loop structure and shutdown
  contract are preserved verbatim.
- **S-028** amended BC-2.05.004 to v1.1.0 during adversarial review. The amendment added
  `timestamp_micros: i64` to the `HookEventReceived` message type definition. S-028 is the
  TUI consumer; this story is the daemon producer. They are complementary.
- **Clock capture precedent:** Review how the 5 hook handlers populate
  `EventBusHookEvent.received_at` (currently a `String` formatted timestamp). The new
  `timestamp_micros` field should be captured at the same site as `received_at` — both are
  "time the daemon received the HTTP POST". If `received_at` uses `chrono::Utc::now()`, the
  same instant should provide `timestamp_micros` via `.timestamp_micros()` — do NOT make a
  second clock call.

## Architecture Compliance Rules

Source: `architecture/SS-ipc.md v1.10.0`, `architecture/SS-daemon-wiring.md v1.3.0`, <!-- version-pin-historical: authored against SS-ipc v1.10.0 at S-032 authoring time -->
`architecture/SS-deps-pin-manifest.md`. <!-- version-pin-historical: authored against v1.2.0 at S-032 authoring time -->

1. **Single clock capture per hook event (BC-2.05.004 INV-4 / SS-ipc.md §timestamp_micros):**
   One and only one `SystemTime::now()` or `chrono::Utc::now()` call per hook handler
   lifecycle. Both `HookEventRecord.timestamp_micros` (ring write) and
   `HookEventReceived.timestamp_micros` (IPC broadcast) are populated from this single
   capture. `SystemTime::now()` inside `event_bus_fan_out_task` for timestamp purposes is
   FORBIDDEN.

2. **Fan-out task MUST NOT be forcibly aborted (BC-2.04.011 PC-7 / SS-daemon-wiring.md
   §Event Bus):** The task exits only when `EventBusRx::recv()` returns `None` (channel
   closed). No `task.abort()` call. No `tokio::select!` with a shutdown branch inside the
   fan-out loop. Shutdown is driven by dropping `EventBusTx` before the runtime stops.

3. **Bounded send to per-client channel (SS-ipc.md §Connection Lifecycle Phase 2):**
   Use `sender.send(msg).await` through `tokio::time::timeout(50ms, ...)`. NEVER use
   `sender.blocking_send()` (blocking in async context). NEVER use `try_send` for the
   fan-out (it would silently discard messages to slow clients; `timeout` correctly removes
   slow clients instead).

4. **monocle-runtime MUST NOT depend on monocle-tui (SS-deps-pin-manifest.md §Workspace
   Dependency Graph):** The fan-out task writes to `mpsc::Sender<ServerToClient>` — it has
   no dependency on TUI rendering types. If `monocle-tui` appears in
   `monocle-runtime/Cargo.toml` after this story, the build MUST fail (forbidden dep).

5. **Payload excerpt truncation is NOT re-applied at fan-out (BC-2.05.004 PC-1):**
   The 256-byte truncation of `payload_excerpt` is performed at the HTTP handler layer
   (S-021 / S-022 scope). The fan-out task receives an already-truncated value from
   `EventBusHookEvent.payload_excerpt` and passes it through verbatim.

## Library & Framework Requirements

All versions from `SS-deps-pin-manifest.md` (authoritative source; never invent
versions from training data): <!-- version-pin-historical: authored against v1.2.0 at S-032 authoring time -->

| Library | Version | Usage in This Story | Pin Policy |
|---------|---------|---------------------|------------|
| tokio | =1.52 | `tokio::time::timeout`, `mpsc::Sender::send`, async task | EXACT |
| serde | 1 | `#[derive(Serialize, Deserialize)]` on updated `HookEventReceived` variant | caret |
| serde_json | =1.0.149 | JSON serialization of `ServerToClient` for framing | EXACT |
| tracing | 0.1 | Structured logging in fan-out task | caret |

The `timestamp_micros: i64` field type matches `HookEventRecord::timestamp_micros` (signed
Unix epoch microseconds) per `SS-core-types-and-abi.md §HookEventRecord`. No new crate
imports are required; all types are already in scope.

## File Structure Requirements

Files to modify (NO new files required):

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/types.rs` | Add `timestamp_micros: i64` to `EventBusHookEvent` struct; update `new()` constructor |
| `crates/monocle-ipc/src/types.rs` | Add `timestamp_micros: i64` to `ServerToClient::HookEventReceived` variant |
| `crates/monocle-runtime/src/event_bus.rs` | Replace Phase-1 stub body in `event_bus_fan_out_task`; remove `#[allow(dead_code)]` on `FAN_OUT_CLIENT_TIMEOUT_MS`; remove stub comments |
| `crates/monocle-runtime/src/router.rs` (or hook handler modules) | Verify/ensure `timestamp_micros` is propagated from single clock capture to `EventBusHookEvent`; thread value if not already present |
| `crates/monocle-runtime/tests/event_bus.rs` | Add 5 new integration tests (see Tasks) |

Files to NOT modify:
- `crates/monocle-ipc/src/server.rs` — `SubscriberList` and `register_subscriber` /
  `remove_subscriber` are complete from S-021/S-022; no changes needed.
- `crates/monocle-tui/` — TUI rendering of `timestamp_micros` is S-028 scope; no TUI
  changes in this story.
- Any BC or spec file — BC-2.05.004 v1.1.0 is already correct; no spec amendments needed.

## Forbidden Dependencies

The following MUST NOT appear as new imports or `Cargo.toml` dependencies after this story:

- `monocle-tui` in `monocle-runtime/Cargo.toml` — circular dependency risk; daemon must not
  depend on TUI rendering types.
- Any second call to `SystemTime::now()` inside `event_bus_fan_out_task` for timestamp
  purposes — INV-4 violation; single clock capture is an architectural invariant.

## Anchor Justifications

**Subsystem anchors:**
- SS-04 owns `event_bus_fan_out_task` (in `monocle-runtime`, the binary composition root
  per ARCH-INDEX Subsystem Registry SS-04). The fan-out task is part of the daemon's
  internal wiring (hook routing → bounded event bus → IPC fan-out).
- SS-05 owns `ServerToClient::HookEventReceived` (in `monocle-ipc`, the UDS transport
  per ARCH-INDEX Subsystem Registry SS-05). Adding `timestamp_micros` to this variant is
  an SS-05 message-type change.
- Both subsystems are correctly listed in `subsystems: [SS-04, SS-05]`.

**Dependency anchors:**
- S-032 depends on S-021 because S-021 created `SubscriberList`, `ServerToClient`, and
  the `HookEventReceived` variant definition. Without S-021's IPC substrate, there is
  nothing to broadcast to.
- S-032 depends on S-022 because S-022 wired the daemon-side accept loop and per-client
  sender channels. The fan-out task writes to these channels; without S-022 they do not
  exist in the daemon start sequence.
- S-032 depends on S-028 because S-028's adversarial review authored the BC-2.05.004
  v1.1.0 amendment that defines PC-2 / INV-4 (the timestamp_micros equality obligation
  this story discharges). Dispatching S-032 before S-028 is done would implement a BC
  that may still be evolving.
- S-032 blocks nothing: it is an additive daemon-side capability that does not gate any
  subsequent story in the current wave plan.

**Wave assignment:**
- Wave 8 is post-Wave-7. Wave 7 is the final Phase 3 wave (S-027/028/029/031). This
  story is post-Phase-3 eligible per the task scope — it does NOT force its way into
  Wave 7. The wave label `8` is a placeholder indicating "after Wave 7 gate; Phase 5
  eligible as a standalone follow-up story".

## §Trace v1.0

**Initial authoring** (2026-06-01T14:00:00Z):
- S-032 created to anchor the deferred daemon-side obligation surfaced during S-028
  adversarial review (BC-2.05.004 v1.1.0 PC-2 / INV-4 — `timestamp_micros` single
  clock capture equality invariant).
- Scope: un-stub `event_bus_fan_out_task`; add `timestamp_micros: i64` to
  `EventBusHookEvent` and `ServerToClient::HookEventReceived`; integrate with
  `SubscriberList` from S-021/S-022.
- EPIC-05 assigned: IPC subsystem (SS-05) owns `HookEventReceived` message type;
  SS-04 owns the fan-out task location (`monocle-runtime`). Story touches both.
- Wave 8 (post-Phase-3 / Phase-5 eligible) — does NOT block Wave 7 delivery.
- Status: draft — BC-2.05.004 v1.1.0 is authored and canonical; this story satisfies
  the Spec-First Gate (S-7.01) requirement that `behavioral_contracts` be non-empty.
- Story discharges deferred finding from S-028: "daemon HookEventReceived broadcast stub
  + BC-2.05.004 PC-2 orphaned obligation".
- SE-16d monotonicity: v1.0 timestamp 2026-06-01 is initial.
