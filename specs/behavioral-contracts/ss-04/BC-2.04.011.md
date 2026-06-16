---
document_type: behavioral-contract
level: L3
version: "1.4.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:04:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "9615ea9"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D-012, F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.011: Bounded Event Bus with Drop Counter

## Description

The bounded event bus is a `tokio::sync::mpsc::channel(4096)` pair that connects incoming
hook events to all connected TUI clients via a dedicated fan-out task. Hook handlers publish
events using `try_send` (non-blocking); if the channel is full, the event is dropped and an
`AtomicU64` drop counter is incremented by 1. The drop counter is visible in every
daemon-to-TUI state push, enabling the TUI status bar to show real-time saturation. A
dedicated `event_bus_task` owns the receiver and fans each event out to all connected IPC
writers, removing disconnected clients from the list. The bus is initialized at daemon start
step 5 and runs until the graceful-shutdown signal.

## Preconditions

1. The daemon start sequence is executing step 5 (before step 6 — EngineModule registry
   registration — and before step 12 — HTTP server start).
2. `tokio::sync::mpsc::channel` is available in the tokio runtime initialized for the
   daemon process.
3. `DaemonState` is being constructed and is not yet published to the axum router.

## Postconditions

**PC-1 — Channel construction with capacity N=4096.**
`tokio::sync::mpsc::channel::<HookEvent>(4096)` is called exactly once per daemon start.
The sender half (`EventBusTx`, type alias for `tokio::sync::mpsc::Sender<HookEvent>`) is
stored in `DaemonState` wrapped in `Arc<EventBusTx>`. The receiver half (`EventBusRx`,
type alias for `tokio::sync::mpsc::Receiver<HookEvent>`) is consumed by the fan-out task.
The channel capacity of 4096 is a compile-time constant; it MUST NOT be configurable at
runtime in Phase 1 (future Phase 4 tuning is acceptable).

**PC-2 — Drop counter initialized to zero.**
`DaemonState.drop_counter` is initialized as `Arc<AtomicU64>::new(0)` (or equivalent
`AtomicU64::new(0)` stored directly in `DaemonState` and accessed by `Arc<DaemonState>`).
The counter starts at 0 and monotonically increases; it is never reset to 0 during a
daemon run (no counter reset on reconnect or other lifecycle events).

**PC-3 — Hook handlers use `try_send` (non-blocking publish).**
Every hook handler (BC-2.04.007, BC-2.04.008, BC-2.04.009) calls
`DaemonState.event_bus_tx.try_send(hook_event)` after obtaining a `HookDecision`. If
`try_send` returns `Ok(())`, the event is in the channel buffer. If `try_send` returns
`Err(TrySendError::Full)`:
  a. `DaemonState.drop_counter.fetch_add(1, Ordering::Relaxed)` increments the counter.
  b. `tracing::warn!("event bus full; dropping event (drop_count={})", ...)` is emitted.
  c. The event is silently discarded.
The hook handler MUST NOT use `send(event).await` (blocking send); the 300ms / 2000ms
timeout budgets require non-blocking publish.

**PC-4 — `event_bus_task` fan-out loop.**
A dedicated tokio task is spawned during daemon startup (step 5). The task:
1. Awaits `EventBusRx.recv()`. If the channel is closed (sender dropped), the task exits.
2. For each connected TUI IPC writer in `DaemonState.tui_clients`, attempts to send the
   `HookEventReceived` IPC message (SS-05 framing).
3. If a write to a TUI client fails (broken pipe, disconnect), the client is removed from
   the `tui_clients` list.
4. Loops back to step 1.

**PC-5 — Fan-out does not block recv on slow TUI clients.**
The fan-out task MUST use a non-blocking or timeout-bounded write to each TUI client. A slow
or unresponsive TUI client MUST NOT cause `EventBusRx.recv()` to stall (which would allow
the channel to fill, increasing hook-handler drop rate). Each client write has a per-client
timeout of 50ms; if the write does not complete within 50ms, the client is removed from the
`tui_clients` list and the event is logged as undeliverable to that client.

**PC-6 — Drop counter included in state pushes.**
Every daemon-to-TUI state push message (periodic or on-change) MUST include the current
value of `DaemonState.drop_counter` (read as `Ordering::Relaxed`). The TUI status bar
renders this as `drops: <N>` or equivalent (rendering is TUI scope, not SS-04 scope; SS-04
only requires the value be present in the state push payload).

**PC-7 — Graceful shutdown: channel close.**
On graceful shutdown (SIGTERM/SIGINT, BC-2.01.004), the `EventBusTx` sender stored in
`DaemonState` is dropped before the tokio runtime shuts down. This closes the channel,
causing `EventBusRx.recv()` in the fan-out task to return `None`, terminating the fan-out
loop cleanly. The fan-out task MUST NOT be forcibly aborted; it must exit via channel close.

**PC-8 — DropCounterUpdate debounce: at most once per 100ms.**
The daemon sends `ServerToClient::DropCounterUpdate` messages to connected TUI clients at
most once per 100ms, regardless of how many drop events occur within that window. The value
sent in each `DropCounterUpdate` reflects the cumulative `DaemonState.drop_counter` value
at debounce-fire time (not a delta since the last update). Implementation: a debounce timer
(tokio interval or `tokio::time::sleep` loop) fires at 100ms cadence; the timer task reads
`DaemonState.drop_counter` with `Ordering::Relaxed` and sends `DropCounterUpdate` only if
the value has changed since the last send. Architecture source: SS-ipc.md lines 288-289
(DropCounterUpdate debounce specification).
- **Rationale:** Without debounce, a burst of 4,096 drop events in 10ms would generate
  4,096 IPC messages, saturating the TUI IPC channel. The 100ms debounce coalesces all
  drops in the window into a single status update, protecting TUI IPC bandwidth while
  still providing timely drop visibility to the operator.

## Invariants

1. The channel capacity is exactly N=4096. No runtime override of this value is permitted
   in Phase 1. Changing N requires a code change and recompilation.
2. Hook handlers MUST use `try_send` (non-blocking). Blocking `send().await` in a hook
   handler is forbidden because it would allow TUI slowness to violate the 300ms/2000ms
   timeout budgets.
3. The drop counter MUST NOT be reset during a daemon run. Its value is a monotonically
   increasing counter from daemon start to daemon stop. A value of 0 means no events have
   been lost since daemon start.
4. Fan-out to TUI clients MUST be decoupled from hook-handler latency. The `event_bus_task`
   owns the receiver; hook handlers only `try_send` to the sender. No hook handler ever
   reads from the channel or waits for a TUI client to consume an event.
5. The drop counter uses `Ordering::Relaxed` for both reads (state pushes) and writes
   (increment on drop). `Relaxed` is sufficient because the counter is for monitoring only;
   no ordering guarantee between the counter value and other memory operations is required.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-094 | Channel fills to capacity 4096 exactly (one more event arrives) | Event 4097 is dropped; drop counter becomes 1; WARN logged; channel remains at 4096 capacity |
| EC-095 | Fan-out task finds all TUI clients disconnected | `event_bus_task` removes all clients from `tui_clients`; channel receiver loop continues; events are received from channel but not delivered (no clients); events are discarded after failed delivery; no memory accumulation |
| EC-096 | TUI client IPC write blocks (slow consumer) | Per-client 50ms write timeout fires; client removed from `tui_clients`; fan-out continues to remaining clients; `EventBusRx.recv()` continues without stall |
| EC-097 | 1000 hook events per second synthetic load (Phase 1 integration test target) | All events published via `try_send` at 1000/s; channel (4096 capacity) buffers ~4 seconds of events at 1000/s before drops begin; at 1000/s steady-state with fan-out consuming at similar rate, drop counter remains 0; integration test asserts `drop_counter == 0` at 1000 events/s |
| EC-098 | Daemon receives SIGTERM during active event bus operation | `EventBusTx` sender dropped on SIGTERM path; `EventBusRx.recv()` returns `None`; `event_bus_task` exits cleanly; no panic; remaining in-flight events in the channel are discarded (shutdown takes priority over delivery) |
| EC-099 | Hook handler calls `try_send` after channel is closed (sender dropped during shutdown) | `try_send` returns `Err(TrySendError::Disconnected)`; handler logs WARN and discards the event; no panic; this is expected during the shutdown race window |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Daemon start completes step 5; 0 events sent | `drop_counter == 0`; channel has 4096 capacity; fan-out task running | happy-path |
| 1 hook event published via `try_send` with 1 TUI client connected | TUI client receives `HookEventReceived` message; `drop_counter` unchanged (0) | happy-path |
| 4097 events published in rapid burst with fan-out task paused (simulate slow consumer) | First 4096 events in channel; 1 event dropped; `drop_counter == 1`; WARN logged | edge-case |
| TUI client disconnects mid-fan-out | Broken pipe detected; client removed from `tui_clients`; subsequent events not sent to that client; no crash | edge-case |
| Graceful shutdown signal | `EventBusTx` dropped; fan-out task exits via `recv() == None`; no lingering task | happy-path |
| `try_send` after channel closed (shutdown race) | `Err(TrySendError::Disconnected)`; WARN logged; no panic | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Channel constructed with capacity 4096 | unit (tokio::sync::mpsc capacity assertion) |
| VP-TBD | `drop_counter` increments on full channel and remains 0 under normal load at 1000 events/s | integration (load test harness) |
| VP-TBD | Fan-out task removes disconnected TUI clients without panic | integration |
| VP-TBD | `try_send` used (not blocking send) in hook handlers | static analysis / code review |
| VP-TBD | Fan-out task exits cleanly on channel close at shutdown | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — "bounded event bus" is named explicitly as a CAP-004 responsibility; this BC is the direct operationalization of the bounded event bus architecture: channel capacity, try_send semantics, drop counter, and fan-out task wiring |
| L2 Domain Invariants | DI-001 (every hook event received MUST be written to the JSONL ring before acknowledgement — the event bus drop counter does NOT exempt events from ring writes; ring append (PC-6 in BC-2.04.007/008/009) happens independently of bus saturation; bus drops affect TUI delivery only, not ring persistence) |
| Architecture Module | monocle-runtime (event bus initialization, fan-out task) per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §Bounded Event Bus |
| Cross-Ref | BC-2.04.007 (PreToolUse handler — PC-5 uses this event bus); BC-2.04.008 (Notification handler — PC-5 uses this bus); BC-2.04.009 (Stop/SessionStart/PromptSubmit — PC-5 uses this bus); BC-2.04.001 (daemon start sequence step 5 initializes this bus) |
| Test File | `monocle-runtime/tests/event_bus.rs` |
| Test Name | `test_BC_2_04_011_bounded_event_bus` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.007] — composes with: PreToolUse handler publishes to this bus at PC-5
- [BC-2.04.008] — composes with: Notification handler publishes to this bus at PC-5
- [BC-2.04.009] — composes with: Stop/SessionStart/PromptSubmit handlers publish to this bus at PC-5
- [BC-2.04.001] — depends on: daemon start sequence step 5 constructs this bus

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#bounded-event-bus` — capacity rationale (4096 × 512B = 2MB worst-case), drop counter, fan-out architecture, back-pressure design
- `architecture/SS-daemon-wiring.md#daemon-start-sequence` — step 5 initialization context

## Story Anchor

S-TBD — Implement bounded event bus with `mpsc::channel(4096)`, try_send drop counter, and fan-out task (filled by story-writer)

## VP Anchors

- VP-TBD — filled after VP creation

## §Trace v1.0.0

**Initial production** (2026-05-26T12:04:00Z):
- BC-2.04.011 created as new artifact for SS-04 §Bounded Event Bus per task instruction.
- Covers: channel construction (N=4096), drop counter AtomicU64, try_send non-blocking
  publish, drop-on-full with WARN, fan-out task loop, per-client 50ms write timeout,
  drop counter in state pushes, graceful shutdown via channel close.
- Capability anchor: CAP-004 per ARCH-INDEX §SS-04 Capability Traceability row ("bounded
  event bus" named explicitly in CAP-004 statement).
- DI-001 clarification: event bus drops affect TUI delivery only; ring appends in hook
  handlers (BC-2.04.007/008/009 PC-6) are independent and not affected by bus saturation.
- SE-16d PASS: 2026-05-26T12:04:00Z > chain prior 2026-05-26T12:03:00Z. PASS.

## §Trace v1.1.0

**F-P1D-012 MEDIUM — DropCounterUpdate debounce postcondition added** (2026-05-26T00:00:00Z):
- PC-8 added: `ServerToClient::DropCounterUpdate` is sent at most once per 100ms regardless
  of how many drop events occur within that window; value reflects cumulative counter at
  debounce-fire time. Architecture source: SS-ipc.md lines 288-289.
- Rationale: without debounce, a 4,096-drop burst would generate 4,096 IPC messages,
  saturating the TUI IPC channel. The 100ms window coalesces drops into a single status
  update while preserving timely operator visibility.
- SE-16d monotonicity: v1.1.0 timestamp >= v1.0.0. PASS.

## §Trace v1.2.0

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.0.0` → `SS-daemon-wiring.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.3.0

**F-P1D4-003 LOW — Architecture Source pin updated from v1.1.0 to v1.2.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.1.0` → `SS-daemon-wiring.md v1.2.0` per F-P1D4-003 bulk update.
- SE-16d monotonicity: v1.3.0 timestamp >= v1.2.0. PASS.

## §Trace v1.4.0

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-daemon-wiring.md v1.2.0 → v1.3.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §Bounded Event Bus` → `SS-daemon-wiring.md v1.3.0 §Bounded Event Bus`.
- Plain version-pin refresh. No substantive content propagation required — §Bounded Event Bus section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.4.0 timestamp >= v1.3.0. PASS.
