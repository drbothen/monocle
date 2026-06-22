---
document_type: story
level: L4
story_id: S-046
epic_id: EPIC-05
version: "1.9"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
points: 5
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-021, S-032]
blocks: [S-047]
target_module: monocle-runtime
subsystems: [SS-05]
behavioral_contracts: [BC-2.05.009, BC-2.05.011]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.009.md, version: "1.5.11"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.011.md, version: "1.2.11"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.24.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.17.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.05.009 (PtyOutput fan-out broker — bounded INPUT channel Arc<Bytes>(1024), per-client mpsc::Sender<ServerToClient>(64), 3-strike disconnect, PtyReset on broker task drop) and BC-2.05.011 (PtyReset variant definition — S-046 OWNS the ServerToClient::PtyReset variant; S-047 references it)"
# BC status: BC-2.05.009 and BC-2.05.011 non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-046: PtyOutput Fan-out Broker — Bounded Channel, Backpressure, and Client Lifecycle

## Narrative

As the monocle daemon, I want the PTY output broker to fan-out terminal bytes to all
connected IPC clients using a bounded `tokio::mpsc::channel(1024)` with
`.send().await` backpressure (never drop on full), maintain a per-client isolated
write buffer of capacity 64, disconnect after 3 consecutive send failures, and emit
`ServerToClient::PtyReset` when a PTY writer task is dropped — so that TUI clients
receive a faithful, ordered view of terminal output and the daemon does not silently
lose bytes or fail to notify clients when a session terminates.

## Acceptance Criteria

### AC-001 (traces to BC-2.05.009 postcondition 1 — bounded channel 1024 with await backpressure)

The PtyOutput broker's primary fan-out channel is `tokio::mpsc::channel(1024)` (bounded,
NOT `unbounded_channel`). When the channel is full, the broker caller (PTY reader task)
blocks on `.send().await` — it does NOT drop the message, does NOT yield and retry later,
and does NOT log and continue. Backpressure propagates to the PTY reader.

### AC-002 (traces to BC-2.05.009 postcondition 2 — per-client isolated `mpsc::Sender<ServerToClient>` with capacity 64)

Each connected IPC client has an isolated per-client channel of type
`mpsc::Sender<ServerToClient>` with capacity 64 (per SS-ipc.md §Daemon-Side Per-Client
Fan-out Channel and architect adjudication Q1):
- The broker wraps each raw PTY frame (`Arc<Bytes>`) as `ServerToClient::PtyOutput { session_id: String, bytes: Vec<u8> }` before sending to each per-client channel. `Arc<Bytes>` MUST NOT be the per-client channel item type — the channel carries `ServerToClient` messages.
- `fan_out(session_id: &str, frame: Arc<Bytes>)` creates a `ServerToClient::PtyOutput { session_id: session_id.to_string(), bytes: frame.to_vec() }` and sends it to each subscribed client's channel.
- Channels are independent: a slow client's full channel does NOT block sends to other clients (3-strike disconnect applies, not blocking send).
- The per-client `mpsc::Sender<ServerToClient>` is initialized fresh on connect and discarded on disconnect.
- The broker's INPUT channel (PTY reader task → broker) remains `Arc<Bytes>` with capacity 1024. This input channel carries raw PTY frames before wrapping.

### AC-003 (traces to BC-2.05.009 postcondition 3 — 3-strike disconnect for slow clients)

When sending to a per-client buffer fails `N` times consecutively:
- Strike 1: log at `tracing::warn!` level with session ID and client ID.
- Strike 2: log at `tracing::warn!`.
- Strike 3: disconnect the client (close the write half; remove from active client set).
- The strike counter resets to 0 after any successful send to that client.
- Clients are never silently removed without exhausting all 3 strikes (unless hardware error).

### AC-004 (traces to BC-2.05.009 PC-3 — pty_drop_counter is a stderr-WARN-only diagnostic; drops are surfaced to TUI via PtyReset per Invariant 5)

The `pty_drop_counter` metric is incremented ONLY when a message is dropped due to an
OOM-level failure (channel closed, receiver gone). It is NOT incremented on backpressure
waits, per-client buffer-full strikes, or graceful disconnects. When the counter increments,
the PTY-broker logs a `WARN`-level structured trace entry to the session-host's stderr:
`WARN: PTY channel drop #N for session <session_id>`. The counter is NOT surfaced over IPC
and does NOT appear in any `ServerToClient` variant — there is no `ServerToClient::StatusUpdate`
or similar counter-carrying variant in the IPC wire protocol. Actual PTY drops are surfaced
to the TUI exclusively via `ServerToClient::PtyReset` (the 5-second status bar indicator
is handled in S-047/S-048 — the broker's responsibility ends at emitting `PtyReset`, per
BC-2.05.009 Invariant 5).

### AC-005 (traces to BC-2.05.009 invariant 4 — PtyReset emitted on broker task drop)

When the PTY writer task for a session is dropped (session exit, OOM kill, unexpected error):
- The broker emits `ServerToClient::PtyReset { session_id }` to ALL connected clients
  that were subscribed to that session's PTY output.
- The emit is fire-and-forget per-client (3-strike rules apply). Emission failure for one
  client does not prevent emission to others.

### AC-006 (traces to BC-2.05.009 invariant 6 — hook events priority over PtyOutput in select!)

The broker task uses `tokio::select!` with two arms:
1. Hook/control event channel (higher priority — biased).
2. PtyOutput channel (lower priority — default arm).

When both arms are ready simultaneously, the hook/control event is always processed first.
This prevents PTY saturation from delaying permission prompts or overlay commands.

### AC-007 (traces to BC-2.05.009 invariant 3 — no .try_send(drop) on PTY reader channel; .send().await backpressure only; no global unbounded_channel)

There is NO `tokio::mpsc::unbounded_channel` call in the PTY fan-out code path.
`cargo grep unbounded_channel crates/monocle-runtime/src/` must return 0 results
in files related to PTY output.

### AC-008 (traces to BC-2.05.009 invariant 4 — PtyReset protocol triggers AttachSession re-trigger)

When a TUI receives `ServerToClient::PtyReset`, the TUI-side handler:
- Clears the local scrollback buffer for that session.
- Re-triggers an `AttachSession` request to request a fresh scrollback dump.
- Displays a 5-second status bar notice (handled in S-048 and S-047, referenced here
  for traceability — the broker's responsibility ends at emission).

## Tasks

- [ ] Add `PtyBroker` struct to `monocle-runtime/src/pty_broker.rs` (new file):
      - **INPUT channel** (PTY reader → broker): `tokio::mpsc::channel::<Arc<Bytes>>(1024)`. Carries raw PTY frame bytes before wrapping.
      - **Per-client channels**: `HashMap<String, mpsc::Sender<ServerToClient>>` keyed by client_id (`String`). Each client channel has capacity 64. Strike counters: `HashMap<String, u8>`.
      - `register_client(id: String, capacity: usize = 64)` → creates `mpsc::channel::<ServerToClient>(64)`, stores the Sender in registry, returns the Receiver end to the IPC writer task.
      - `unregister_client(id: &str)` → removes and drops the per-client sender.
      - `fan_out(session_id: &str, frame: Arc<Bytes>)` → wraps as `ServerToClient::PtyOutput { session_id: session_id.to_string(), bytes: frame.to_vec() }`, then sends to all registered client channels; applies 3-strike logic. `Arc<Bytes>` MUST NOT be the item type in per-client channels.
      - `emit_pty_reset(session_id: &str)` → sends `ServerToClient::PtyReset { session_id: session_id.to_string() }` directly to all per-client channels (no Arc wrapping needed — already a `ServerToClient` message).
- [ ] Implement the broker event loop as a `tokio::spawn`ed task using `tokio::select!` with
      hook/control arm biased per AC-006. Ensure `biased;` keyword is used in `select!` macro.
- [ ] Add `pty_drop_counter: Arc<AtomicU64>` to the `DaemonState` struct and increment only on OOM-level channel failure.
- [ ] Update `SessionManager::spawn_session()` (from S-033) to create a `PtyBroker` for each session
      and wire the PTY reader task → broker channel → per-client channels.
- [ ] Add `ServerToClient::PtyReset { session_id: String }` variant to the IPC message enum
      (in `crates/monocle-ipc/src/lib.rs`, NOT `proto.rs` — canonical file per S-033/S-039/S-044)
      — this is a new variant defined in BC-2.05.011 but needed here for the broker to emit it.
      `session_id` MUST be `String` (UUID-as-String per SS-session-manager.md §session_id type ruling);
      `SessionId` newtype MUST NOT appear in IPC wire types.
- [ ] Write unit tests in `monocle-runtime/tests/pty_broker.rs`:
      - `test_BC_2_05_009_bounded_channel_backpressure_blocks_not_drops` (AC-001)
      - `test_BC_2_05_009_per_client_isolation_slow_client_does_not_block_fast` (AC-002)
      - `test_BC_2_05_009_three_strike_disconnect` (AC-003)
      - `test_BC_2_05_009_pty_drop_counter_only_oom_not_backpressure` (AC-004)
      - `test_BC_2_05_009_pty_reset_emitted_on_broker_drop` (AC-005)
      - `test_BC_2_05_009_hook_events_priority_over_pty_output` (AC-006)
      - `test_BC_2_05_009_no_unbounded_channel_in_pty_path` (AC-007 — compile-time or grep assertion)

## Previous Story Intelligence

S-021 established the UDS server, `DaemonState` struct, and the `ClientId` / `SessionId` types.
S-032 established the daemon event bus. This story adds the PTY-specific fan-out layer ON TOP of
the event bus: PTY bytes are NOT dispatched through the general event bus (they bypass it for
latency and volume reasons). The PTY broker is a dedicated channel per-session, separate from
the `SessionListUpdate` and hook-event buses from S-021/S-032.

**Ownership note:** `ServerToClient::PtyReset { session_id: String }` is OWNED BY S-046
(this story adds it to `crates/monocle-ipc/src/lib.rs`). S-047 depends on S-046 for the
`PtyReset` variant to exist and references it without re-creating it. Both stories co-own
BC-2.05.011: S-046 owns the daemon-side broker emission; S-047 owns the TUI-side protocol
handler. This split is by-design to avoid a dependency cycle (S-047 depends on S-046).

## Architecture Compliance Rules

From `architecture/SS-ipc.md v1.24.0` (§Daemon-Side Per-Client Fan-out Channel):
- **Broker INPUT channel (PTY reader → broker):** `tokio::mpsc::channel::<Arc<Bytes>>(1024)`.
  Raw PTY frames; `Arc<Bytes>` is the item type at this layer only.
- **Per-client channel:** `mpsc::Sender<ServerToClient>` with capacity 64. The broker wraps
  each frame as `ServerToClient::PtyOutput { session_id, bytes }` before per-client send.
  `Arc<Bytes>` MUST NOT be the per-client channel item type.
- `emit_pty_reset()` sends `ServerToClient::PtyReset { session_id }` directly to per-client
  channels — no wrapping needed, it is already a `ServerToClient` variant.
- `PtyReset` is a first-class `ServerToClient` variant, NOT a synthetic event.
- `biased;` select! ensures hook events are never starved by PTY volume.

From `architecture/SS-session-manager.md`:
- `DaemonState` owns all session-scoped resources; `PtyBroker` is created per-session inside
  `SessionManager::spawn_session()`.
- `pty_drop_counter` is a daemon-global atomic, not a per-session counter.

**Forbidden dependencies**: `monocle-runtime` PTY broker MUST NOT import from `monocle-tui`.
All TUI-side behavior (5-second status bar, scrollback clear, AttachSession re-trigger) is
in `monocle-tui` and is referenced in S-047/S-048 for implementation.

## Library and Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| `tokio` | `=1.52.0` (EXACT) | `mpsc::channel`, `select!`, `spawn` |
| `bytes` | `^1.11` | `Arc<Bytes>` for zero-copy fan-out |
| `tracing` | `^0.1` | Strike counter warn! logs |

No new dependencies. All three are in the workspace `Cargo.toml` already.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/monocle-runtime/src/pty_broker.rs` | CREATE | `PtyBroker` struct + event loop |
| `crates/monocle-runtime/src/lib.rs` | MODIFY | Add `pub mod pty_broker;` |
| `crates/monocle-runtime/src/session_manager/mod.rs` | MODIFY | Wire PtyBroker into spawn_session() (canonical path: module dir, not flat .rs file) |
| `crates/monocle-ipc/src/lib.rs` | MODIFY | Add `ServerToClient::PtyReset { session_id: String }` variant (canonical file per S-033/S-039/S-044; not proto.rs) |
| `crates/monocle-runtime/tests/pty_broker.rs` | CREATE | Unit tests for all ACs |

## Behavioral Contracts

| BC | Title | Version | Ownership |
|----|-------|---------|-----------|
| BC-2.05.009 | PtyOutput Fan-out Broker — Bounded Channel, Backpressure, Per-Client Isolation, 3-Strike Disconnect | (see inputs: frontmatter) | OWNED by S-046 |
| BC-2.05.011 | New ServerToClient IPC Variants — ScrollbackChunk, ScrollbackDumpComplete, PtyReset | (see inputs: frontmatter) | PtyReset variant OWNED by S-046; ScrollbackChunk/ScrollbackDumpComplete variants owned by S-047 |

**Ownership clarification (S-046 vs S-047):** `ServerToClient::PtyReset { session_id: String }` is
added to `monocle-ipc/src/lib.rs` in this story (S-046) because the broker needs to emit it when a
PTY writer task is dropped. S-047 depends on S-046 for `PtyReset` to exist; S-047 adds the TUI-side
handler for `PtyReset` (scrollback buffer clear + re-trigger `AttachSession`). `ServerToClient::PtyReset`
is a BC-2.05.011 variant, but BC-2.05.011 is split across two stories — the broker-side emission (this
story) and the client-side protocol (S-047). Both stories co-own BC-2.05.011 but at different layers.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-200 | No clients registered; PTY frame arrives | Frame is silently discarded (no clients to fan-out to); `pty_drop_counter` NOT incremented |
| EC-201 | One client disconnects mid-stream (Receiver dropped) | Next send attempt to that client fails → starts strike counter → disconnected after 3 strikes |
| EC-202 | All clients disconnect; PTY keeps producing | All sends fail → all clients strike out; broker stays alive for new connects; `pty_drop_counter` NOT incremented (no OOM, just empty registry) |
| EC-203 | PTY produces frames faster than channel capacity (1024 items) | Caller blocks on `.send().await` — backpressure to PTY reader until channel drains |
| EC-204 | `emit_pty_reset()` called with no clients registered | No-op; no error |
| EC-205 | Hook event arrives exactly when PTY frame arrives | `biased; select!` guarantees hook event processed first |
| EC-206 | `pty_drop_counter` is read concurrently by the logger task | Read via `Arc<AtomicU64>::load(Ordering::Relaxed)` — safe under concurrent writers; no IPC emission path reads this counter |

## Token Budget Estimate

| Category | Estimate |
|----------|----------|
| Story spec (this file) | ~4 500 tokens |
| BC files (1 BC) | ~3 000 tokens |
| Architecture sections (SS-ipc, SS-session-manager excerpts) | ~2 000 tokens |
| Existing code context (session_manager/mod.rs, monocle-ipc/src/lib.rs, event bus from S-032) | ~3 000 tokens |
| Test file to write | ~2 500 tokens |
| **Total estimated** | **~15 000 tokens** |

Well within the 20–30% context window constraint. No splitting needed.

## Dependency Justification

- S-046 depends on S-021 because `DaemonState`, `ClientId`, `SessionId`, and the UDS server
  infrastructure that hosts the client registry are all established in S-021; the broker
  plugs into that infrastructure.
- S-046 depends on S-032 because the daemon event bus (S-032) defines the priority ordering
  contract that the broker's `biased; select!` must respect.
- S-046 blocks S-047 because S-047 (IPC lifecycle variants) builds the `AttachSession` handler
  that triggers scrollback — and the `PtyReset` variant added in this story is the mechanism
  that `AttachSession` responds to on reconnect.

## Subsystem Anchor Justification

SS-05 owns this story's scope because the PTY output fan-out is a core IPC capability — it
controls how session terminal output reaches connected TUI clients over the UDS channel
managed by SS-05 per ARCH-INDEX Subsystem Registry SS-05 (monocle-ipc, daemon IPC layer).

## Trace

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.9 | 2026-06-22 | vsdd-factory:story-writer | inputs[] pin refresh — BC-2.05.009 v1.5.5→v1.5.11, BC-2.05.011 v1.2.5→v1.2.11, SS-session-manager v2.6.1→v2.17.1 (SS-ipc v1.24.0/SS-deps-pin-manifest v1.2.1/SS-deps-pin-manifest-v2-delta v1.0.2 unchanged); body accuracy verified against canonical specs — §Daemon-Side Per-Client Fan-out Channel contract stable (per-client mpsc::Sender<ServerToClient> cap 64, broker INPUT Arc<Bytes> cap 1024, biased select!, PtyReset ownership all confirmed); SS-session-manager spawn_session/DaemonState integration claims confirmed stable at v2.17.1. |
| 1.4 | 2026-06-16 | vsdd-factory:story-writer | F-P24-SUG-002: BC-2.05.011 body BC-table title corrected — "ScrollbackChunk/ScrollbackDumpComplete/PtyReset Protocol" → "New ServerToClient IPC Variants — ScrollbackChunk, ScrollbackDumpComplete, PtyReset" to match BC canonical title. |
| 1.3 | 2026-06-16 | vsdd-factory:story-writer | Corpus-wide AC-trace-citation audit (F-P20-CRIT-001 class): AC-005 "postcondition 5"→"invariant 4" (PtyReset on drop); AC-006 "invariant 1"→"invariant 6" (hook priority); AC-007 "invariant 2"→"invariant 3" (backpressure/.send().await); AC-008 "invariant 3"→"invariant 4" (PtyReset protocol). AC bodies unchanged. |
| 1.0 | 2026-06-15 | vsdd-factory:story-writer | Initial decomposition |
| 1.2 | 2026-06-16 | vsdd-factory:story-writer | F-P19-SUG-001: Bump BC-2.05.011 input pin "1.2.4" → "1.2.5" (metadata-only Story-Anchor delta; no behavioral change). |
| 1.1 | 2026-06-16 | vsdd-factory:story-writer | F-P14-IMP-001: Rewrite AC-004 to conform to BC-2.05.009 PC-3 + Invariant 5 — `pty_drop_counter` is stderr-WARN-only; removed false `ServerToClient::StatusUpdate` reference (no such variant exists); corrected trace header from "postcondition 4" to "PC-3 + Invariant 5"; fixed EC-206 to remove `StatusUpdate` emission reference |
