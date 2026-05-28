---
document_type: architect-decision
level: ops
cycle: cycle-001
story: S-025
pass: 2
version: "1.0"
status: binding
producer: vsdd-factory:architect
timestamp: 2026-05-28T12:00:00Z
phase: 3
inputs:
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.8.0"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md, version: "1.0.5"}
  - {path: crates/monocle-ipc/src/framing.rs}
  - {path: crates/monocle-tui/src/app.rs}
input-hash: "[live-state]"
traces_to: "Resolves F-S025-ADV2-BLOCKER-001 from S-025 adversarial Pass 2."
---

# S-025 Architect Decisions — Pass 2

## Decision — F-S025-ADV2-BLOCKER-001 (IPC read cancellation-safety)

**Chosen option: B — Dedicated reader task + bounded mpsc.**

**Rationale:** `read_framed` calls `reader.read_exact(&mut len_buf)` followed by
`reader.read_exact(&mut payload)` in sequence. Neither call is cancellation-safe: bytes
consumed from the UDS kernel buffer on the first `read_exact` are lost when the outer
`tokio::time::timeout` future is dropped mid-frame. Frame desync is then inevitable within
seconds of normal operation because the daemon emits messages with inter-message gaps
routinely exceeding 1ms (SessionListUpdate, HookEventReceived, etc.). This directly
violates BC-2.05.002 Postcondition 6 ("IPC receive loop runs concurrently with render;
IPC messages do not block the terminal event loop") and the at-least-once delivery
guarantee documented in SS-ipc §Risk Mitigations "PermissionPromptQueued Delivered Twice".

Option A (persistent boxed future) avoids the cancellation hazard but forces the event
loop to hold a borrow over `transport` across loop iterations. Because `UdsClientTransport`
is not `Clone`, an `Arc<Mutex<UdsTransport>>` wrapper would be required — adding lock
contention on the hot render path. `tokio::select!` with a persistent boxed borrow of
`transport` and keyboard input in the same select arm also creates lifetime complexity
that erodes clarity without benefit over Option B.

Option B is the direct application of SS-ipc §Backpressure and the CLAUDE.md channel
convention ("bounded `mpsc::channel(N)` with surfaced drop counters"). The reader task
holds exclusive ownership of `transport` (no shared-memory, no lock), runs
`read_framed` to completion on every call, and forwards results into a bounded channel.
Cancellation-safe by construction. The event loop uses `try_recv()` against the channel
receiver — a non-blocking, infallible call that fits cleanly into the existing
`loop { draw; poll_keyboard; poll_ipc; }` structure. Reconnect integration is explicit:
old task is aborted via its `JoinHandle`, new transport is moved into the freshly spawned
task. No borrow management across iterations.

---

## Required Event Loop Architecture

```rust
// ---- Initialization (before the event loop) -----------------------------------

// Connect and receive InitialState (existing flow).
let (mut transport, _event_rx) = connect_with_events(sock_path).await?;
let initial = read_framed::<_, ServerToClient>(&mut transport).await?;
apply_initial_state(&mut app, initial)?;

// Transfer transport ownership to the reader task.
// The reader task loops forever, forwarding completed frames or disconnect errors.
let (ipc_tx, mut ipc_rx) = tokio::sync::mpsc::channel::<Result<ServerToClient, IpcError>>(64);
let mut reader_handle: tokio::task::JoinHandle<()> = {
    let tx = ipc_tx.clone();
    tokio::spawn(async move {
        loop {
            match read_framed::<_, ServerToClient>(&mut transport).await {
                Ok(msg) => {
                    if tx.send(Ok(msg)).await.is_err() {
                        // Receiver dropped (TUI exiting): exit cleanly.
                        break;
                    }
                }
                Err(IpcError::Disconnected) => {
                    let _ = tx.send(Err(IpcError::Disconnected)).await;
                    break; // Task exits; event loop handles reconnect.
                }
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                    break;
                }
            }
        }
    })
};

// ---- Main event loop ----------------------------------------------------------

let tick_rate = Duration::from_millis(16); // ~60fps, keyboard cadence

loop {
    // 1. Render.
    terminal.draw(|frame| { /* ... */ })?;

    // 2. Poll keyboard (non-blocking, bounded by tick_rate).
    if event::poll(tick_rate)? {
        if let Event::Key(ct_key) = event::read()? {
            // ... handle key input ...
        }
    }

    // 3. Drain IPC channel (non-blocking try_recv; process all available messages).
    loop {
        match ipc_rx.try_recv() {
            Ok(Ok(msg)) => {
                if let Err(e) = handle_server_message(&mut app, msg) {
                    tracing::error!(error = %e, "fatal protocol error; closing IPC connection");
                    on_transport_event(&mut app, TransportEvent::Disconnected);
                    reader_handle.abort();
                    // Enter reconnect path or break.
                }
            }
            Ok(Err(IpcError::Disconnected)) | Ok(Err(_)) => {
                on_transport_event(&mut app, TransportEvent::Disconnected);
                reader_handle.abort();
                // Reconnect path: spawn new reader task after connect_with_events().
                break;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                // Reader task exited unexpectedly.
                on_transport_event(&mut app, TransportEvent::Disconnected);
                break;
            }
        }
    }
}
```

---

## Channel Provisioning

- **Channel:** `tokio::sync::mpsc::channel::<Result<ServerToClient, IpcError>>(64)`
- **Capacity N = 64.** Justification: the daemon's event bus emits at most one message per
  hook arrival. The hook receiver enforces a 1000 events/sec target (SS-conventions channel
  convention). At 16ms render cadence the event loop drains up to 64 messages per tick;
  at 1000 events/sec that is 16 events/tick — well within budget. N=64 provides 4× headroom
  against burst without holding unbounded memory (64 × ~1KB average message ≈ 64KB max
  enqueued).
- **Drop policy on full: Block backpressure.** The reader task calls `tx.send(msg).await`
  (blocking send), not `try_send`. This means the reader task blocks when the channel is
  full rather than dropping messages. Rationale: at-least-once delivery is the SS-ipc
  contract for `PermissionPromptQueued` (BC-2.05.002 Invariant 4). Silent drop on full
  would violate that contract for permission prompt messages. Back-pressure is the correct
  mechanism — if the event loop is consistently slower than the daemon, that is a render
  performance problem to diagnose, not a message-loss policy to encode.
- **Drop counter integration:** Drop counter is NOT bumped on channel back-pressure (that
  is an IPC transport metric, not a TUI backpressure metric). `app.drop_counter` reflects
  the daemon-reported drop counter from `ServerToClient::DropCounterUpdate` only. If TUI-side
  render lag becomes a concern in a future wave, a separate `tui_backpressure_counter` can
  be added; that is out of scope for S-025.

---

## Reader Task Lifecycle

- **Spawn on:** receipt of `InitialState` success (transport ownership transferred immediately
  after initial state is applied to `App`).
- **Terminate on:** `read_framed` returns any error — forwards the error to channel, then
  `break`s out of its loop. The task `JoinHandle` is retained in the event loop scope so
  the event loop can call `reader_handle.abort()` on clean exit or reconnect.
- **Restart on:** successful reconnect — `reader_handle.abort()` to ensure the old task
  is cleaned up, then `tokio::spawn(...)` a new reader task with the new `transport` from
  `connect_with_events()`. The `ipc_tx` clone is moved into the new task; the event loop
  retains `ipc_rx` (receiver is not replaced — the same channel is reused across
  reconnections to avoid losing messages in flight during the reconnect window).

---

## Keyboard Polling Cadence

**Recommended interval: 16ms (~60Hz).**

The existing implementation at `crates/monocle-tui/src/app.rs:427` already uses
`Duration::from_millis(16)` as `tick_rate` for the render loop. This is correct. The
1ms outer timeout on `read_framed` was a code smell introduced to approximate a
non-blocking IPC poll; it is removed entirely by Option B. The `event::poll(tick_rate)`
call (crossterm keyboard poll) retains its 16ms ceiling, which is consistent with the
~60fps render cadence and standard TUI UX expectations (lazygit, zellij both use 16-50ms).
No change to keyboard cadence is needed.

---

## Transport Ownership Model

**Reader task takes exclusive ownership of `transport`.** After `apply_initial_state()`
returns, `transport` is moved into the reader task closure via `async move`. The event
loop retains only `ipc_rx` (the channel receiver) and `reader_handle` (the `JoinHandle`).
There is no `Arc<Mutex<UdsClientTransport>>` — single-owner move semantics throughout.

**Reconnect handoff:**
1. Event loop receives `Err(IpcError::Disconnected)` from `ipc_rx`.
2. Event loop calls `reader_handle.abort()` and `.await`s the handle (or ignores the
   `JoinError::Cancelled`).
3. Event loop calls `on_transport_event(&mut app, TransportEvent::Disconnected)` — SOQ-3
   overlay clear fires here.
4. Event loop enters the reconnect loop from SS-ipc §Reconnection Behavior (exponential
   backoff from 250ms, cap 2s, 5s total window).
5. On `connect_with_events()` success: move new `transport` into a fresh `tokio::spawn`
   closure; bind `reader_handle` to the new `JoinHandle`. The existing `ipc_rx` receiver
   is reused (channel was provisioned once; the new `ipc_tx` clone is derived from the
   channel's `Sender` end, which remains valid across task restarts).
6. Resume the main event loop.

---

## Implementer Directive (step-by-step)

1. **Remove the `tokio::time::timeout(Duration::from_millis(1), read_framed(...))` block**
   from the event loop in `crates/monocle-tui/src/app.rs` (lines 531–553 in the current
   S-025 implementation). This is the cancellation-unsafe pattern being replaced.

2. **Add channel declaration** immediately after `apply_initial_state()` succeeds:
   ```rust
   let (ipc_tx, mut ipc_rx) =
       tokio::sync::mpsc::channel::<Result<ServerToClient, monocle_ipc::error::IpcError>>(64);
   ```

3. **Spawn the reader task**, moving `transport` into it:
   ```rust
   let mut reader_handle = spawn_ipc_reader(transport, ipc_tx);
   ```
   Extract the spawn into a named helper function `spawn_ipc_reader(transport, tx) -> JoinHandle<()>`
   to make reconnect re-spawn readable.

4. **Replace the timeout block** with the `loop { ipc_rx.try_recv() ... }` drain shown in
   the pseudocode above, placed after the keyboard poll block.

5. **Reconnect integration:** when the drain loop observes `Ok(Err(_))` or
   `Err(TryRecvError::Disconnected)`, call `reader_handle.abort()`, fire
   `on_transport_event(Disconnected)`, then invoke the reconnect loop. On success,
   call `spawn_ipc_reader(new_transport, ipc_tx.clone())` and assign the result back
   to `reader_handle`.

6. **Remove `Duration` import aliasing** for the 1ms poll if it is no longer used elsewhere.

7. **Cargo.toml:** no new dependencies. `tokio::sync::mpsc` is already available through
   the existing `tokio` workspace dependency with the `sync` feature.

---

## Test Directive

Add an integration test to `crates/monocle-tui/tests/ipc_reader_task.rs` (new file):

**Test name:** `test_BC_2_05_002_pc_6_no_frame_corruption_across_inter_message_gap`

**What it asserts:** A sequence of N=10 `ServerToClient::SessionListUpdate` messages
emitted by a mock daemon over a `tokio::io::duplex` pair, with a >5ms sleep between
each message, are received by the TUI-side reader task in order and without frame
corruption. After all N messages are emitted, the test drains `ipc_rx` and asserts:
- Exactly N messages received.
- Each decoded as `ServerToClient::SessionListUpdate`.
- Session list contents match emission order (message 1 has session `["s1"]`, message 2
  has `["s2"]`, etc.).

**Why this test validates the fix:** The >5ms gap between messages guarantees that the
outer event loop would have fired its 1ms timeout (and dropped the in-progress future)
under the old pattern. Under Option B the reader task holds the `read_framed` future to
completion regardless of event loop cadence — so N=10 messages must arrive intact.

---

## SS-ipc Update

SS-ipc is currently silent on the TUI IPC read loop architecture. This decision requires
a new §TUI IPC Read Loop Pattern section. Add under §Connection Lifecycle:

---

**Version bump: SS-ipc v1.8.0 → v1.9.0**

**New section to add after §Phase 3: Disconnect:**

```markdown
### §TUI IPC Read Loop Pattern

`read_framed` is NOT cancellation-safe: the two sequential `read_exact` calls inside
it will silently corrupt the byte stream if the future is dropped between the first and
second call. TUI implementations MUST NOT wrap `read_framed` in `tokio::time::timeout`
or any other construct that may drop the future mid-frame.

**Canonical TUI IPC read pattern (Option B — dedicated reader task):**

1. After receiving `InitialState`, transfer the `UdsClientTransport` (or raw `UnixStream`
   reader half) into a dedicated `tokio::task` whose sole job is to call `read_framed` in
   a loop and forward results into a bounded `mpsc::channel(64)`.
2. The event loop drains the channel receiver with `try_recv()` (non-blocking) on each
   render tick. It does NOT call `read_framed` directly.
3. On disconnect (reader task forwards `Err(IpcError::Disconnected)`): abort the reader
   task, fire `TransportEvent::Disconnected` (SOQ-3 overlay clear), enter the reconnect
   loop. On reconnect success, spawn a fresh reader task with the new transport.
4. Channel capacity = 64. Drop policy = block (sender awaits). This preserves at-least-once
   delivery semantics for `PermissionPromptQueued` (BC-2.05.002 Invariant 4).
```

---

## Cross-Story Coordination

**S-023 reconnect signature:** S-023 changes `connect_with_events()` to return
`(UdsClientTransport, EventReceiver)`. Option B integrates cleanly: after a disconnect,
the event loop calls `connect_with_events()`, receives the new `transport`, moves it into
`spawn_ipc_reader(new_transport, ipc_tx.clone())`. The `EventReceiver` is used for
`TransportEvent` signaling (Disconnected/Reconnected) — the reader task forwards
`IpcError::Disconnected` through the mpsc channel, and the event loop calls
`on_transport_event(Reconnected)` after a successful reconnect. S-023 and S-025 do not
share transport ownership; their integration point is the `connect_with_events()` API
shape, which is unchanged by this decision.

**S-026/S-027 implications:** The reader task + channel pattern is the correct extension
point for permission overlay and event ribbon. S-026 consumes `PermissionPromptQueued` and
`PermissionPromptResolved` from the same `ipc_rx` channel — no architectural change
needed. The `handle_server_message` dispatch table in `app.rs` gains new match arms; the
reader task and channel are unchanged. S-027 (event ribbon) likewise consumes
`HookEventReceived` from `ipc_rx`. The single-channel, single-reader-task model scales to
all current and planned downstream consumers because all of them run in the single TUI
event loop task that owns `ipc_rx`.
