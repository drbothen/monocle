---
document_type: story
level: L4
story_id: S-046
epic_id: EPIC-05
version: "2.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-22T00:00:00Z
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
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.009.md, version: "1.6.0"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.011.md, version: "1.2.11"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.25.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.17.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.05.009 (PtyOutput fan-out broker — bounded INPUT channel Arc<Bytes>(1024), broadcast_to_subscribers via shared SubscriberList, 1-strike disconnect, PtyReset on proxy tx.send error or session-host PtyReset) and BC-2.05.011 (PtyReset variant definition — S-046 OWNS the ServerToClient::PtyReset variant; S-047 references it)"
# BC status: BC-2.05.009 and BC-2.05.011 non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-046: PtyOutput Fan-out Broker — Bounded Channel, Backpressure, and Client Lifecycle

## Narrative

As the monocle daemon, I want the PTY output broker to fan-out terminal bytes to all
connected IPC clients using a bounded `tokio::mpsc::channel(1024)` INPUT channel with
`.send().await` backpressure (never drop on full), and then broadcast each wrapped
`ServerToClient::PtyOutput` frame to all registered TUI clients via
`broadcast_to_subscribers(&shared_subscriber_list, msg)` — the same `SubscriberList`
used for all other daemon fan-out — so that TUI clients receive a faithful, ordered
view of terminal output, slow clients are isolated immediately (1-strike), and the
daemon does not silently lose bytes or fail to notify clients when a session terminates.
`ServerToClient::PtyReset` is emitted when the session-host explicitly sends
`HostToDaemon::PtyReset`, or when the proxy task's `tx.send` returns an error
(OOM-level failure). The broker does NOT own its own client registry.

## Acceptance Criteria

### AC-001 (traces to BC-2.05.009 postcondition 1 — bounded INPUT channel 1024 with await backpressure)

The `PtyBroker` owns a bounded INPUT channel (`tokio::mpsc::channel::<Arc<Bytes>>(1024)`,
NOT `unbounded_channel`) between the proxy task and the broker event loop. When the INPUT
channel is full, the proxy task's `tx.send(frame).await` call blocks — it does NOT drop
the message, does NOT yield and retry later, and does NOT log and continue. Backpressure
propagates through the proxy task to the PTY reader. The `PtyBroker` struct field is the
INPUT channel sender/receiver pair; it does NOT own per-client channel fields.

### AC-002 (traces to BC-2.05.009 postcondition 1b — fan-out via broadcast_to_subscribers on shared SubscriberList)

The broker event loop wraps each `Arc<Bytes>` frame as
`ServerToClient::PtyOutput { session_id: session_id.to_string(), bytes: frame.to_vec() }`
and fans it out to ALL connected TUI clients by calling
`broadcast_to_subscribers(&shared_subscriber_list, msg)`:
- `shared_subscriber_list` is the daemon's single `Arc<SubscriberList>` (defined in
  `monocle_ipc::server`), populated by `register_subscriber` on IPC connect and drained
  by `remove_subscriber` on disconnect. The `PtyBroker` MUST NOT own its own
  `clients: HashMap<...>` or `strike_counters: HashMap<...>` fields.
- `broadcast_to_subscribers` uses `.try_send()` into each per-client
  `mpsc::channel::<ServerToClient>(64)`. A slow client whose buffer is full is
  disconnected immediately (1-strike, per BC-2.05.004 EC-005). Other clients are unaffected.
- `Arc<Bytes>` MUST NOT be the item type of the per-client fan-out channel.
  The per-client channel item type is `ServerToClient` (capacity 64).
- The `PtyBroker` struct holds ONLY the INPUT channel sender/receiver and an
  `Arc<SubscriberList>` reference passed at construction time from `spawn_session`.

### AC-003 (traces to BC-2.05.009 postcondition 1b — 1-strike disconnect for slow clients via broadcast_to_subscribers)

Slow-client disconnection is governed by `broadcast_to_subscribers` semantics
(BC-2.05.004 EC-005, BC-2.05.009 Invariant 3b):
- A single `TrySendError::Full` on any per-client send buffer (capacity 64) removes
  that client immediately and fires its `disconnect: Notify`, causing the per-client
  write task to close the UDS socket.
- All other clients are unaffected by one client's disconnection.
- No 3-strike counter is maintained; there is no `strike_counters` field in `PtyBroker`.
- The 3-strike threshold was specified for the retired isolated per-broker registry
  design and is superseded by the unified `SubscriberList` model per SS-ipc.md
  §PtyBroker integration (Q1 ruling).

### AC-004 (traces to BC-2.05.009 postcondition 2/3 — pty_drop_counter incremented ONLY in proxy task on tx.send error)

The `pty_drop_counter` (`Arc<AtomicU64>`) is incremented ONLY when the proxy task's
`tx.send(frame).await` returns `Err(_)` — meaning the broker INPUT channel receiver has
been closed while the session is still live (OOM-level / programming error condition,
unreachable under normal operation).

The counter is NOT incremented when:
- The INPUT channel is full (that is backpressure — `.send().await` blocks until space is available).
- The INPUT channel receiver closes because the session exited gracefully (`input_rx.recv()
  == None` in the broker event loop — this is the NORMAL session-exit path).
- A TUI client's per-client send buffer is full and the client is disconnected by
  `broadcast_to_subscribers`.

When the counter increments, the proxy task logs at `WARN` level:
`WARN: PTY channel drop #N for session <session_id>`.
The counter is NOT surfaced over IPC and does NOT appear in any `ServerToClient` variant.
Actual PTY drops are surfaced to the TUI exclusively via `ServerToClient::PtyReset`
(the 5-second status bar indicator is handled in S-047/S-048 — the broker's responsibility
ends at emitting `PtyReset`, per BC-2.05.009 Invariant 5).

### AC-005 (traces to BC-2.05.009 invariant 4 — PtyReset emitted on exactly two triggers; NOT on graceful session exit)

`ServerToClient::PtyReset { session_id }` is emitted via
`broadcast_to_subscribers(&shared_subscriber_list, PtyReset)` on exactly TWO conditions:

1. The session-host sends `HostToDaemon::PtyReset` (the session-host's own PTY reader
   detected a byte drop in its internal ring). The proxy task calls
   `broadcast_to_subscribers` with `PtyReset` directly. This is the primary production path.

2. The proxy task's `tx.send(frame).await` returns `Err(_)` (the broker INPUT channel
   receiver has been dropped while the session is still live — OOM-level failure). The
   proxy task calls `broadcast_to_subscribers` with `PtyReset` and increments
   `pty_drop_counter`.

**The broker event loop MUST NOT emit `PtyReset` when `input_rx.recv()` returns `None`.**
Input channel close is the NORMAL graceful session-exit path (proxy task exits, drops its
sender). Emitting `PtyReset` on graceful teardown would spuriously corrupt TUI state.
The broker MUST simply return when `input_rx.recv()` yields `None`.

Emission is via `broadcast_to_subscribers` (1-strike slow-client model). Emission failure
for one client does not prevent emission to others.

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
      - **INPUT channel** (proxy task → broker): `tokio::mpsc::channel::<Arc<Bytes>>(1024)`. Carries raw PTY frame bytes before wrapping.
      - **`Arc<SubscriberList>` reference**: passed in at construction from `spawn_session`; this is the daemon's shared subscriber registry — NOT a duplicate per-broker registry.
      - The struct MUST NOT contain `clients: HashMap<...>` or `strike_counters: HashMap<...>` fields.
      - Constructor signature: `PtyBroker::new(session_id: SessionId, subscriber_list: Arc<SubscriberList>, drop_counter: Arc<AtomicU64>) -> (PtyBroker, mpsc::Receiver<Arc<Bytes>>)` — returns the broker and the INPUT channel receiver (the broker holds the sender end).
- [ ] Implement the broker event loop as a `tokio::spawn`ed task in `PtyBroker::spawn_event_loop`:
      - Loop: `tokio::select! { biased; <hook/control arm>, <PTY input arm> }`.
      - PTY input arm: on `Some(frame)` received from `input_rx`, call
        `broadcast_to_subscribers(&self.subscriber_list, ServerToClient::PtyOutput { session_id: session_id.to_string(), bytes: frame.to_vec() })`.
      - PTY input arm: on `None` (INPUT channel closed — graceful session exit), break out of
        the loop and return WITHOUT emitting `PtyReset`.
      - Ensure `biased;` keyword is used in `select!` macro per AC-006.
- [ ] Add `pty_drop_counter: Arc<AtomicU64>` to the `DaemonState` struct. Increment ONLY in the
      proxy task when `tx.send(frame).await` returns `Err(_)`.
- [ ] Update the proxy task (in `SessionManager::spawn_session()` from S-033) to:
      - On `HostToDaemon::PtyBytes`: call `tx.send(Arc::new(Bytes::from(bytes))).await`;
        on `Err(_)` increment `pty_drop_counter` and call
        `broadcast_to_subscribers(&subscriber_list, PtyReset { session_id })`.
      - On `HostToDaemon::PtyReset`: call
        `broadcast_to_subscribers(&subscriber_list, PtyReset { session_id })` directly.
      - Do NOT emit `PtyReset` when the INPUT channel closes (`input_rx.recv() == None`).
- [ ] Update `SessionManager::spawn_session()` to construct a `PtyBroker` per-session,
      passing the daemon's shared `Arc<SubscriberList>` (not a cloned snapshot).
- [ ] Add `ServerToClient::PtyReset { session_id: String }` variant to the IPC message enum
      (in `crates/monocle-ipc/src/lib.rs`, NOT `proto.rs` — canonical file per S-033/S-039/S-044)
      — this is a new variant defined in BC-2.05.011 but needed here for the broker to emit it.
      `session_id` MUST be `String` (UUID-as-String per SS-session-manager.md §session_id type ruling);
      `SessionId` newtype MUST NOT appear in IPC wire types.
- [ ] Write unit tests in `monocle-runtime/tests/pty_broker.rs`:
      - `test_BC_2_05_009_bounded_channel_backpressure_blocks_not_drops` (AC-001)
      - `test_BC_2_05_009_fan_out_via_subscriber_list_not_broker_registry` (AC-002 — uses `register_subscriber` + `SubscriberList`, not `register_client`)
      - `test_BC_2_05_009_one_strike_disconnect_slow_client` (AC-003 — verifies 1-strike via `broadcast_to_subscribers`)
      - `test_BC_2_05_009_pty_drop_counter_only_oom_not_backpressure` (AC-004)
      - `test_BC_2_05_009_pty_reset_emitted_on_proxy_send_error_not_graceful_close` (AC-005 — two sub-cases: proxy send error emits reset; input_rx.recv()==None does NOT)
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

From `architecture/SS-ipc.md v1.25.0` (§PtyBroker integration — Ruling Q1):
- **`PtyBroker` MUST NOT own a client registry.** Fields `clients: HashMap<...>` and
  `strike_counters: HashMap<...>` are FORBIDDEN in the `PtyBroker` struct.
- **Broker INPUT channel (proxy task → broker):** `tokio::mpsc::channel::<Arc<Bytes>>(1024)`.
  Raw PTY frames; `Arc<Bytes>` is the item type at this layer only.
- **Fan-out:** The broker event loop calls `broadcast_to_subscribers(&shared_subscriber_list, msg)`
  with a `ServerToClient::PtyOutput` — the SAME `SubscriberList` used by all daemon-to-TUI
  fan-out (hook events, session state changes, permission prompts).
- **Slow-client disconnect:** 1-strike via `broadcast_to_subscribers` semantics
  (BC-2.05.004 EC-005). No per-broker strike counters. No 3-strike logic.
- **PtyReset triggers:** exactly two — (a) `HostToDaemon::PtyReset` from session-host;
  (b) proxy task `tx.send(frame).await` returns `Err(_)`. MUST NOT emit on
  `input_rx.recv() == None` (graceful session exit).
- **Drop counter site:** proxy task only, on `tx.send().await` error. NOT on backpressure
  full, NOT on `input_rx.recv() == None`, NOT on per-client `broadcast_to_subscribers` failure.
- `Arc<Bytes>` MUST NOT be the per-client fan-out channel item type.
- `biased;` select! ensures hook events are never starved by PTY volume.
- **`spawn_session` passes `Arc<SubscriberList>`:** the shared list is passed at broker
  construction; it MUST NOT be cloned into a snapshot at spawn time (that would miss
  post-spawn connects).

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
| BC-2.05.009 | PtyOutput Fan-Out — Per-Session Bounded Channel (1024) with Drop Counter (stderr WARN) + PtyReset TUI Recovery | (see inputs: frontmatter) | OWNED by S-046 |
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
| EC-200 | No TUI clients in `SubscriberList`; PTY frame arrives | `broadcast_to_subscribers` iterates over empty list; frame bytes discarded; `pty_drop_counter` NOT incremented |
| EC-201 | One TUI client's per-client send buffer is full when `broadcast_to_subscribers` tries to send | 1-strike: client removed immediately by `broadcast_to_subscribers`; `disconnect: Notify` fired; other clients continue receiving uninterrupted |
| EC-202 | All TUI clients disconnect; PTY keeps producing | `broadcast_to_subscribers` iterates over empty list; bytes discarded; broker stays alive for new connects (new connects populate `SubscriberList` dynamically); `pty_drop_counter` NOT incremented |
| EC-203 | PTY produces frames faster than INPUT channel capacity (1024 items) | Proxy task's `tx.send(frame).await` blocks — backpressure to PTY reader until channel drains; no bytes dropped; `pty_drop_counter` NOT incremented |
| EC-204 | `broadcast_to_subscribers` called with `PtyReset` when `SubscriberList` is empty | No-op; no error; `pty_drop_counter` NOT incremented |
| EC-205 | Hook event arrives exactly when PTY frame arrives in broker `select!` | `biased; select!` guarantees hook/control arm processed first; PTY frame processed in next iteration |
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
| 2.0 | 2026-06-22 | vsdd-factory:story-writer | Architect re-architecture cascade (SS-ipc bump, BC-2.05.009 bump): inputs[] pins BC-2.05.009 and SS-ipc updated to current canonical versions. Narrative rewritten: broker owns INPUT channel + Arc<SubscriberList> only (no per-client HashMap, no strike_counters). AC-001 clarified as INPUT channel. AC-002 rewritten: fan-out via broadcast_to_subscribers on shared SubscriberList (not broker-owned registry). AC-003 rewritten: 1-strike via broadcast_to_subscribers (3-strike retired). AC-004 updated: drop counter ONLY on proxy task tx.send Err (not on input_rx.recv()==None). AC-005 updated: two canonical PtyReset triggers; explicitly MUST NOT emit on graceful input close. Tasks rewritten: register_client/unregister_client/fan_out/emit_pty_reset removed; broadcast_to_subscribers + Arc<SubscriberList> constructor; test names updated to SubscriberList model. Architecture Compliance Rules rewritten for Q1 ruling. EC-200/201/202/204 updated to SubscriberList semantics. |
| 1.9 | 2026-06-22 | vsdd-factory:story-writer | inputs[] pin refresh — BC-2.05.009, BC-2.05.011, SS-session-manager bumped to canonical (SS-ipc/SS-deps-pin-manifest/SS-deps-pin-manifest-v2-delta unchanged); body accuracy verified against canonical specs — §Daemon-Side Per-Client Fan-out Channel contract stable (per-client mpsc::Sender<ServerToClient> cap 64, broker INPUT Arc<Bytes> cap 1024, biased select!, PtyReset ownership all confirmed); SS-session-manager spawn_session/DaemonState integration claims confirmed stable at current canonical. |
| 1.4 | 2026-06-16 | vsdd-factory:story-writer | F-P24-SUG-002: BC-2.05.011 body BC-table title corrected — "ScrollbackChunk/ScrollbackDumpComplete/PtyReset Protocol" → "New ServerToClient IPC Variants — ScrollbackChunk, ScrollbackDumpComplete, PtyReset" to match BC canonical title. |
| 1.3 | 2026-06-16 | vsdd-factory:story-writer | Corpus-wide AC-trace-citation audit (F-P20-CRIT-001 class): AC-005 "postcondition 5"→"invariant 4" (PtyReset on drop); AC-006 "invariant 1"→"invariant 6" (hook priority); AC-007 "invariant 2"→"invariant 3" (backpressure/.send().await); AC-008 "invariant 3"→"invariant 4" (PtyReset protocol). AC bodies unchanged. |
| 1.0 | 2026-06-15 | vsdd-factory:story-writer | Initial decomposition |
| 1.2 | 2026-06-16 | vsdd-factory:story-writer | F-P19-SUG-001: Bump BC-2.05.011 input pin "1.2.4" → "1.2.5" (metadata-only Story-Anchor delta; no behavioral change). |
| 1.1 | 2026-06-16 | vsdd-factory:story-writer | F-P14-IMP-001: Rewrite AC-004 to conform to BC-2.05.009 PC-3 + Invariant 5 — `pty_drop_counter` is stderr-WARN-only; removed false `ServerToClient::StatusUpdate` reference (no such variant exists); corrected trace header from "postcondition 4" to "PC-3 + Invariant 5"; fixed EC-206 to remove `StatusUpdate` emission reference |
