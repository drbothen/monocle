---
document_type: behavioral-contract
level: L3
version: "1.2.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-14T18:00:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/SS-embedded-pty.md, architecture/SS-ipc.md, architecture/SS-daemon-wiring-v2-delta.md]
input-hash: "1d2bd94"
traces_to: prd.md
origin: greenfield
subsystem: SS-08
capability: CAP-008
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

# Behavioral Contract BC-2.08.008: SessionStateChanged — Daemon Emits on Every SessionState Transition; Delivered to All TUI Clients; Ordering Relative to SessionListUpdate

## Description

The daemon emits `ServerToClient::SessionStateChanged { session_id, new_state }` to all
connected TUI clients on every `SessionState` transition (e.g., `Launching → Running`,
`Running → Terminating`, `Terminating → Terminated`, `Running → Detached`, etc.). This IPC
message is the trigger for TUI consumers such as the `SessionCreation` wizard auto-advance
(transitions from `Launching` step to `EmbeddedTerminal` when `Running` is received) and
the `EmbeddedTerminal` exit (when `Terminated` is received for the currently-displayed session).
`SessionStateChanged` is broadcast to ALL connected TUI clients, not just the requesting client.
It is emitted in addition to (not as a replacement for) `SessionListUpdate`.

## Preconditions

1. A `SessionEntry` exists in the daemon's session registry.
2. A `SessionState` transition event occurs (process state change, kill, detach, re-discovery,
   watchdog timeout, GC of a dead session during re-discovery, etc.).
3. Zero or more TUI clients are connected to the daemon's UDS.

## Postconditions

### Emission on every transition (no silent transitions)

1. `SessionStateChanged` is emitted for EVERY `SessionState` transition WITHOUT EXCEPTION —
   including re-discovery GC paths, watchdog-forced Terminated transitions, and Detached
   re-discovery registration. The complete transition list:
   - `Launching → Running` (session-host confirms readiness via `HostToDaemon::StateChanged`)
   - `Running → Terminating` (kill_session() called)
   - `Detached → Terminating` (kill_session() called on Detached session)
   - `Launching → Terminating` (kill_session() called on Launching session)
   - `Terminating → Terminated` (session-host confirms exit or 12s watchdog fires)
   - `Running → Detached` (detach_session() called)
   - `Running → Terminated` (session-host sends `StateChanged::Terminated` without kill)
   - `Launching → Terminated` (session-host startup failure)
   - `Detached → Running` (attach_session() called and `ScrollbackDumpComplete` received)
   - `* → Terminated` (any path — re-discovery GC, watchdog, crash detection)
   - Re-discovery GC (dead session): `any → Terminated`, emitted via broker before sidecar GC

   The daemon publishes `ServerToClient::SessionStateChanged { session_id, new_state }` to
   the broker. If no TUI clients are connected, the broker discards the message — no error.

2. The broker dispatches `SessionStateChanged` to ALL connected TUI clients via their
   per-client isolated send buffers (capacity 64 per SS-ipc.md §TUI IPC Read Loop Pattern,
   per BC-2.05.009 Invariant 3b).

### Ordering relative to SessionListUpdate

3. `SessionStateChanged` is enqueued BEFORE `SessionListUpdate` into each client's per-client
   FIFO channel. The ordering mechanism is: both `.try_send()` calls are made while holding
   the `SessionManager` mutex, into the same per-client `mpsc::Sender` in the correct sequence.
   The per-client channel FIFO draining order then guarantees that if both messages are
   delivered, `SessionStateChanged` is received first. The mutex provides the atomicity
   window for both enqueues — it does NOT directly control wire order (the channel FIFO does).
   See SS-daemon-wiring-v2-delta.md v1.10.0 §3b for the canonical emission code pattern.

   **Ordered-pair split on full buffer:** If the first `.try_send()` (SessionStateChanged)
   succeeds but the second `.try_send()` (SessionListUpdate) fails (client buffer full), the
   pair has split. The client is IMMEDIATELY disconnected, INDEPENDENT of the slow-client
   3-strike counter. `slow_send_count` is incremented for telemetry only; the disconnect is
   unconditional on the first split (it does NOT require reaching any threshold). The client
   may reconnect and receive a fresh `InitialState` containing the post-transition state.
   Rationale: delivering a half-pair leaves the TUI in an inconsistent state; partial
   delivery is unsafe.

### Rename does NOT emit SessionStateChanged

4a. **`rename_session()` does NOT emit `SessionStateChanged`.** Rename updates `display_name`
   only — it is NOT a `SessionState` transition. `SessionStateChanged` carries
   `new_state: SessionState` and cannot convey the updated name. Only `SessionListUpdate`
   (carrying the full `SessionSnapshot` with updated `display_name`) is emitted for rename.
   See SS-daemon-wiring-v2-delta.md v1.10.0 §3b emission table.

4b. The `InitialState` push (on TUI client connect) includes the current session list with
   current states. TUI clients that connect after a transition has already occurred will see
   the post-transition state in `InitialState.sessions` — they do not receive a retrospective
   `SessionStateChanged` for past transitions.

### SessionCreation wizard auto-advance

5. When the TUI is in `AppMode::SessionCreation { step: Launching, launching_session_id: Some(session_id), .. }` and receives
   `SessionStateChanged { session_id: <matching launching_session_id>, new_state: Running }`:
   - The `SessionCreation` wizard auto-transitions to
     `AppMode::EmbeddedTerminal { session_id, prior: Dashboard }`.
   - The TUI does NOT require the user to press Enter or any key — the transition is automatic.
   - Auto-advance matches against `launching_session_id` (populated from
     `ServerToClient::SpawnAck { session_id }` on successful spawn), NOT a broadcast-race
     heuristic. `SessionStateChanged` events whose `session_id` does not match
     `launching_session_id` are ignored by the wizard.
   - This auto-transition requires that `SessionStateChanged` is delivered before or at the
     same time as `SessionListUpdate` (so the wizard can act on the exact transition event,
     not just the list update). `SpawnAck` is guaranteed to arrive before any
     broker-published `SessionStateChanged { Launching }` by TWO complementary properties:
     (1) **Causal step ordering** — in the daemon IPC handler, `SpawnAck` is sent at
     step 2 (before `spawn_session()` is called at step 4, which is before the broker
     emits `SessionStateChanged { Launching }` at step 5); and
     (2) **Per-client FIFO** — the requesting client's per-client `mpsc` channel delivers
     messages in send order, guaranteeing that `SpawnAck` (step 2) arrives at the TUI
     before any broker-published `SessionStateChanged { Launching }` (step 5).
     Canonical source: SS-ipc.md §ServerToClient::SpawnAck §Delivery ordering steps 1-5.

6. When the TUI is in `AppMode::EmbeddedTerminal { session_id }` and receives
   `SessionStateChanged { session_id: <matching>, new_state: Terminated }`:
   - The TUI exits `EmbeddedTerminal` mode and transitions to `AppMode::Dashboard`.
   - Status bar shows `Session <display_name> terminated` for 5 seconds.
   - This handles the `Ctrl-D` / natural exit path (session-host detects child exit and
     sends `StateChanged::Terminated` without an explicit kill command from the user).

## Invariants

1. `SessionStateChanged` is published for EVERY state transition without exception —
   including re-discovery GC (dead session → Terminated), Terminating watchdog fires, and
   Detached re-discovery registration. There is no silent state transition in the daemon.
   Every code path that mutates `SessionEntry.state` MUST post `SessionStateChanged` to
   the broker. This includes paths in `rediscover_sessions()` (where transitions occur
   during daemon startup before any TUI client is connected — messages are discarded by
   the broker fan-out, which is correct).
2. The `session_id` in `SessionStateChanged` is the same UUID string used in all other IPC
   messages for the session (canonical per SS-session-manager.md §session_id type ruling).
3. `SessionStateChanged` is NOT an acknowledgement of a TUI request — it is a fact about
   daemon state. When the TUI sends `KillSession`, the daemon responds with `SessionStateChanged`
   when the state actually transitions (not immediately upon receipt of `KillSession`). The
   transition to `Terminating` fires `SessionStateChanged` immediately; `Terminated` fires
   when confirmed.
4. Ordering: `SessionStateChanged` is enqueued BEFORE `SessionListUpdate` into each client's
   per-client FIFO channel for the same state transition. The ordering is guaranteed by the
   per-client channel FIFO drain order (NOT by the mutex hold itself). The mutex provides the
   atomicity window that prevents any other actor from interleaving posts between the two
   `.try_send()` calls. The actual received order for any client is determined by the channel
   FIFO. A TUI consumer that processes messages in receipt order WILL always see
   `SessionStateChanged` before the corresponding `SessionListUpdate` for the same transition.
   If the ordered-pair splits (SessionStateChanged delivered but SessionListUpdate dropped due
   to full buffer), the client is disconnected immediately (PC-3 split rule).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-300 | Session transitions to `Terminating` (kill sent); then immediately to `Terminated` (fast exit) | Two `SessionStateChanged` messages emitted: first `{new_state: Terminating}`, then `{new_state: Terminated}`. Both are broadcast. TUI receives both in order. |
| EC-301 | No TUI clients connected when transition occurs | `SessionStateChanged` posted to broker; broker fan-out has no subscribers; message discarded. No error. Connecting TUI clients receive current state in `InitialState`. |
| EC-302 | TUI client connects during a transition (between `SessionStateChanged` and `SessionListUpdate` for the same session) | The client's `InitialState` push includes the post-transition state. The in-flight `SessionStateChanged` / `SessionListUpdate` pair for the transition is NOT replayed for the new client. The new client may miss the transition event but will see correct state in `InitialState`. |
| EC-303 | `SessionCreation` wizard's session_id changes (session spawn failed and new session spawned) | Wizard tracks the new session_id via `launching_session_id: Option<String>` (populated from `ServerToClient::SpawnAck { session_id }` on successful spawn). `SessionStateChanged` events whose `session_id` does not match `launching_session_id` are ignored by the wizard. On spawn failure, the daemon has already sent `SpawnAck`; the wizard MUST clear `launching_session_id` to `None` when the subsequent `ServerToClient::Error` is received, then re-populate it from the `SpawnAck` of the retry spawn. |
| EC-304 | `SessionStateChanged { new_state: Running }` received for a session the TUI does not have in its local state | TUI logs WARN and requests a fresh `InitialState` re-sync (or ignores if the session appears in the next `SessionListUpdate`). |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| `spawn_session()` → session-host sends `StateChanged::Running` | `SessionStateChanged{Running}` received by TUI; `SessionCreation` wizard auto-advances to `EmbeddedTerminal` | happy-path |
| `kill_session()` → session-host confirms exit | `SessionStateChanged{Terminating}` then `SessionStateChanged{Terminated}` in order; TUI sessions panel shows `[Terminating]` then `[X]` | happy-path |
| `Ctrl-D` in `EmbeddedTerminal` (natural session exit) | `SessionStateChanged{Terminated}` received; TUI exits `EmbeddedTerminal` → Dashboard; status bar shows "terminated" | happy-path |
| `detach_session()` | `SessionStateChanged{Detached}` received; TUI updates sessions panel; session not removed | happy-path |
| `SessionStateChanged{Running}` arrives before `SessionListUpdate` | TUI wizard auto-advances immediately on `SessionStateChanged`; no waiting for `SessionListUpdate` | ordering |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `SessionStateChanged` emitted for every state transition (Launching→Running, Running→Terminating, Terminating→Terminated) | integration |
| VP-TBD | `SessionCreation` wizard auto-advances to `EmbeddedTerminal` on `SessionStateChanged{Running}` | unit |
| VP-TBD | `EmbeddedTerminal` exits to Dashboard on `SessionStateChanged{Terminated}` | unit |
| VP-TBD | `SessionStateChanged` precedes `SessionListUpdate` in per-client buffer for same transition | integration |
| VP-TBD | No TUI clients → `SessionStateChanged` discarded; no error | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — this BC defines the `SessionStateChanged` IPC message which is the primary notification mechanism for session lifecycle state transitions; it is the trigger for the wizard auto-advance and EmbeddedTerminal exit, both of which are core session lifecycle behaviors in CAP-008 |
| Architecture Module | monocle-runtime (SessionManager state transitions → broker publish); monocle-ipc (`ServerToClient::SessionStateChanged` variant); monocle-tui (wizard auto-advance, EmbeddedTerminal exit handlers) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v2.3.0 §Session lifecycle state machine (state transitions, including re-discovery GC and Detached re-discovery; IPC handler generates UUID + sends SpawnAck before spawn_session()); SS-embedded-pty.md v1.6.0 §TUI AppMode Extensions (SessionCreation::Launching auto-transition to EmbeddedTerminal; `launching_session_id: Option<String>` field added — F-P41-IMP-001); SS-ipc.md v1.21.0 §ServerToClient::SpawnAck (new variant; per-client point-to-point delivery before SessionStateChanged{Launching}); SS-daemon-wiring-v2-delta.md v1.10.0 §3b (SessionStateChanged emission rule, ordered-pair-split-on-Full disconnect rule, rename-only-SessionListUpdate rule) |
| Cross-Ref | BC-2.09.008 (SessionCreation wizard auto-transition to EmbeddedTerminal on Running); BC-2.08.003 (kill → Terminating transition; 12s watchdog → Terminated); BC-2.05.003 (SessionListUpdate — emitted concurrently with SessionStateChanged for same transition) |
| Test Name | test_BC_2_08_008_session_state_changed_emitted_on_every_transition |

## Related BCs

- [BC-2.09.008] — depends on: wizard auto-advance uses SessionStateChanged{Running} as trigger
- [BC-2.08.003] — composes with: kill path (Terminating transitions) trigger SessionStateChanged
- [BC-2.05.003] — composes with: SessionListUpdate is the companion to SessionStateChanged for each transition
- [BC-2.05.009] — composes with: per-client isolated send buffer carries SessionStateChanged to TUI clients

## Architecture Anchors

- `architecture/SS-session-manager.md#session-lifecycle-state-machine` (v2.3.0) — complete state transition table; IPC handler UUID-generation + SpawnAck step
- `architecture/SS-embedded-pty.md#tui-appmode-extensions` (v1.6.0) — SessionCreation::Launching auto-advance rule; `launching_session_id: Option<String>` field (F-P41-IMP-001)
- `architecture/SS-ipc.md#servertoClientspawnack` (v1.21.0) — SpawnAck variant; wizard storage obligation; spawn-failure clearing rule
- `architecture/SS-daemon-wiring-v2-delta.md#ipc-handler-new-clienttoserver-variants` — broker publish path

## Story Anchor

S-TBD — Implement SessionStateChanged broadcast on every SessionEntry state transition (filled by story-writer)

## VP Anchors

VP-TBD — SessionStateChanged emission and TUI response integration tests (filled after VP creation)

## §Trace v1.2.1

**CV-SS-005 — PC-5 SpawnAck ordering guarantee completed with causal step ordering** (2026-06-14):

- **Finding (CV-SS-005):** PC-5 stated that `SpawnAck` is delivered before any
  broker-published `SessionStateChanged { Launching }` citing only "the per-client FIFO
  ordering guarantee." This is incomplete. The guarantee is causal, not merely channel-FIFO:
  the per-client FIFO property only ensures in-order delivery to the requesting client; it
  is the IPC handler's step ordering that guarantees `SpawnAck` is produced before the
  broadcast even exists. An implementer reading only PC-5 could wrongly conclude that FIFO
  alone is the mechanism — a misreading that would permit implementations that send `SpawnAck`
  AFTER `spawn_session()` returns and still rely on FIFO to save them (FIFO cannot guarantee
  ordering across the broker's fan-out vs. the client channel).

- **Fix — PC-5 ordering statement expanded to two properties:**
  (1) Causal step ordering: `SpawnAck` is sent in IPC-handler step 2 (before
  `spawn_session()` in step 4, which is before the broker emits
  `SessionStateChanged { Launching }` in step 5). The step sequence is defined in
  SS-ipc.md §ServerToClient::SpawnAck §Delivery ordering and enforced in the IPC handler
  skeleton in SS-session-manager.md §IPC handler pattern.
  (2) Per-client FIFO: the requesting client's per-client `mpsc` channel delivers messages
  in send order, guaranteeing in-order delivery of `SpawnAck` (step 2) ahead of any
  broker-published `SessionStateChanged { Launching }` (step 5).

- **Scope:** PC-5 ordering sentence only. No change to: PC-5 destructure pattern,
  auto-advance match logic, EC-303, emission completeness (PC-1), broker dispatch (PC-2),
  ordering/split rule (PC-3), rename rule (PC-4a), PC-4b, PC-6, Invariants 1-4,
  EC-300/301/302/304, Canonical Test Vectors, or wire contract/field names.

- **SE-16d monotonicity:** v1.2.1 timestamp 2026-06-14T18:00:00Z > v1.2.0 timestamp
  2026-06-14T12:00:00Z. PASS.

## §Trace v1.2.0

**F-P41-IMP-001 — PC-5 destructure corrected to `launching_session_id`; EC-303 SpawnAck mechanism added; arch-source pins to SS-embedded-pty v1.6.0 + SS-ipc v1.21.0** (2026-06-14):

- **PC-5 destructure (normative rewrite):** The old pattern
  `AppMode::SessionCreation { step: Launching, session_id }` was incorrect — the struct
  has no bare `session_id` field. Under F-P41-IMP-001 (SS-embedded-pty.md v1.6.0),
  `AppMode::SessionCreation` gains `launching_session_id: Option<String>` (populated from
  `ServerToClient::SpawnAck { session_id }` receipt). The correct destructure is:
  `AppMode::SessionCreation { step: Launching, launching_session_id: Some(session_id), .. }`.
  Auto-advance now matches against `launching_session_id` (deterministic, not a
  broadcast-race heuristic). `SessionStateChanged` events whose `session_id` does not
  match `launching_session_id` are ignored by the wizard. `SpawnAck` is guaranteed to
  arrive before any broker-published `SessionStateChanged { Launching }` via the per-client
  FIFO channel.

- **EC-303 (normative enrichment):** Added explicit SpawnAck mechanism: wizard tracks
  `launching_session_id: Option<String>` populated from `SpawnAck`. On spawn failure,
  wizard MUST clear `launching_session_id` to `None` when `ServerToClient::Error` is
  received (even though `SpawnAck` was already received for the failed spawn). Retry
  spawn re-populates from the new `SpawnAck`.

- **Arch-source pin:** SS-embedded-pty.md v1.6.0 → v1.6.0 (new `launching_session_id`
  field in `AppMode::SessionCreation` — F-P41-IMP-001); SS-ipc.md v1.21.0 → v1.21.0
  (new `ServerToClient::SpawnAck` variant). Architecture Anchors updated to match.

- No change to: emission completeness (PC-1), broker dispatch (PC-2), ordering/split rule
  (PC-3), rename rule (PC-4a), PC-4b, PC-6, Invariants 1-4, EC-300/301/302/304, or
  Canonical Test Vectors.

- SE-16d monotonicity: v1.2.0 timestamp 2026-06-14 > v1.1.1 timestamp 2026-06-13. PASS.

## §Trace v1.1.1

**S35-001 + arch-source pin sweep — drop contradictory N-strike rationale in split rule; v1.9.0→v1.9.1 + v1.5.1→v1.5.2** (2026-06-13 / D-277):
- S35-001: PC-3 ordered-pair-split rule reworded to eliminate "(equivalent to exhausting the
  3-strike threshold)" framing. The normative rule is: split triggers IMMEDIATE disconnect,
  INDEPENDENT of the slow-client 3-strike counter. `slow_send_count` is incremented for
  telemetry only; the disconnect is unconditional on the first split (does NOT require reaching
  any threshold). Rationale phrase updated to match architect's corrected wording in
  SS-daemon-wiring-v2-delta.md v1.9.1 §3b. Normative outcome unchanged (immediate disconnect
  + reconnect→fresh InitialState); only the incorrect "3-strike equivalence" framing removed.
- Arch-source pin: SS-daemon-wiring-v2-delta.md v1.9.0 → v1.9.1 (all active citations;
  §Trace historical citations exempt per version-pin-historical policy).
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- Patch bump: 1.1.0 → 1.1.1.

## §Trace v1.1.0

**Adversarial Pass 3 fixes — C3-001 (emission completeness + ordering rationale + split rule + rename rule)** (2026-06-03):
- C3-001 (SessionStateChanged emission obligation): PC-1 rewritten to make "no silent transitions"
  explicit, including re-discovery GC, Terminating watchdog, and Detached re-discovery. Precondition
  3 changed from "at least one TUI client connected" to "zero or more" (emission happens regardless;
  broker discards when no subscribers).
- Ordering rationale corrected (PC-3 / Invariant 4): the ordering guarantee is provided by the
  per-client channel FIFO drain order, NOT by "the mutex hold guarantees wire order" (the mutex
  provides atomicity for both `.try_send()` calls but does not directly control wire order).
  Clarified in PC-3 and Invariant 4.
- Ordered-pair-split-on-Full → disconnect rule added (PC-3): if SessionStateChanged delivered
  but SessionListUpdate dropped (buffer full), client disconnected immediately. Rationale:
  half-pair delivery leaves TUI in inconsistent state.
- Rename-does-NOT-emit-SessionStateChanged added as PC-4a: rename updates display_name only;
  SessionStateChanged carries new_state and cannot convey the new name; only SessionListUpdate
  emitted for rename. Per SS-daemon-wiring-v2-delta.md v1.3.1 §3b.
- Architecture Source updated to SS-session-manager.md v1.5.0, SS-daemon-wiring-v2-delta.md v1.3.1.

## §Trace v1.0.0

**Initial production — S2-005 adversarial pass-2 finding** (2026-06-03T23:59:00Z):
- S2-005 finding: no BC governed `ServerToClient::SessionStateChanged` emission, though the
  SessionCreation wizard auto-transition and `Ctrl-D` handling depend on it. The wizard's
  `Launching` step auto-advance to `EmbeddedTerminal` requires a `SessionStateChanged{Running}`
  event; without a governing BC this behavior was unspecified and untestable.
- BC-2.08.008 authored to fill this gap. Governs: when `SessionStateChanged` is emitted
  (every transition), its ordering relative to `SessionListUpdate`, delivery to all TUI clients,
  and the two TUI consumer behaviors (wizard auto-advance, EmbeddedTerminal exit).
- Ordering invariant (PC-3/Invariant 4): `SessionStateChanged` precedes `SessionListUpdate`
  in the per-client buffer for the same transition, by virtue of being posted atomically under
  the same SessionManager mutex hold.
- Design decision (in-scope): `SessionStateChanged{new_state: Terminating}` fires immediately
  when `kill_session()` sends `DaemonToHost::Kill` — not when the session-host confirms.
  This is required for the wizard/panel to render `[Terminating]` immediately (BC-2.06.025
  Invariant 4), not only after the 10s SIGTERM window expires.
- SE-16d PASS: 2026-06-03T23:59:00Z (new artifact).
