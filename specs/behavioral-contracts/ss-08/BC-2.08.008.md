---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:59:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/SS-daemon-wiring-v2-delta.md]
input-hash: ""
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
   watchdog timeout, etc.).
3. At least one TUI client is connected to the daemon's UDS.

## Postconditions

### Emission on every transition

1. For EVERY `SessionState` transition (complete list):
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
   
   The daemon publishes `ServerToClient::SessionStateChanged { session_id, new_state }` to
   the broker.

2. The broker dispatches `SessionStateChanged` to ALL connected TUI clients via their
   per-client isolated send buffers (capacity 256, per BC-2.05.009 Invariant 3b).

### Ordering relative to SessionListUpdate

3. `SessionStateChanged` is emitted BEFORE or CONCURRENTLY WITH `SessionListUpdate` for the
   same state transition. The invariant: a TUI client MUST NOT receive `SessionListUpdate`
   with the new state BEFORE receiving `SessionStateChanged` for that same transition.
   
   In practice: both messages are posted to the broker in the same event-loop tick. The
   broker dispatches them in order to each client's per-client buffer. Since both are posted
   atomically (under the same `SessionManager` mutex hold), their order on the wire to any
   given TUI client is deterministic: `SessionStateChanged` first, then `SessionListUpdate`.

4. The `InitialState` push (on TUI client connect) includes the current session list with
   current states. TUI clients that connect after a transition has already occurred will see
   the post-transition state in `InitialState.sessions` — they do not receive a retrospective
   `SessionStateChanged` for past transitions.

### SessionCreation wizard auto-advance

5. When the TUI is in `AppMode::SessionCreation { step: Launching, session_id }` and receives
   `SessionStateChanged { session_id: <matching>, new_state: Running }`:
   - The `SessionCreation` wizard auto-transitions to
     `AppMode::EmbeddedTerminal { session_id, prior: Dashboard }`.
   - The TUI does NOT require the user to press Enter or any key — the transition is automatic.
   - This auto-transition requires that `SessionStateChanged` is delivered before or at the
     same time as `SessionListUpdate` (so the wizard can act on the exact transition event,
     not just the list update).

6. When the TUI is in `AppMode::EmbeddedTerminal { session_id }` and receives
   `SessionStateChanged { session_id: <matching>, new_state: Terminated }`:
   - The TUI exits `EmbeddedTerminal` mode and transitions to `AppMode::Dashboard`.
   - Status bar shows `Session <display_name> terminated` for 5 seconds.
   - This handles the `Ctrl-D` / natural exit path (session-host detects child exit and
     sends `StateChanged::Terminated` without an explicit kill command from the user).

## Invariants

1. `SessionStateChanged` is published for EVERY state transition without exception. There is
   no "silent" state transition in the daemon. Every state change that updates
   `SessionEntry.state` MUST be accompanied by a `SessionStateChanged` broadcast.
2. The `session_id` in `SessionStateChanged` is the same UUID string used in all other IPC
   messages for the session (canonical per SS-session-manager.md §session_id type ruling).
3. `SessionStateChanged` is NOT an acknowledgement of a TUI request — it is a fact about
   daemon state. When the TUI sends `KillSession`, the daemon responds with `SessionStateChanged`
   when the state actually transitions (not immediately upon receipt of `KillSession`). The
   transition to `Terminating` fires `SessionStateChanged` immediately; `Terminated` fires
   when confirmed.
4. Ordering: `SessionStateChanged` precedes `SessionListUpdate` in the per-client send buffer
   for the same state transition (atomically posted under the same mutex hold in
   `SessionManager`). A TUI consumer that processes messages in receipt order will always see
   `SessionStateChanged` before the corresponding `SessionListUpdate`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-300 | Session transitions to `Terminating` (kill sent); then immediately to `Terminated` (fast exit) | Two `SessionStateChanged` messages emitted: first `{new_state: Terminating}`, then `{new_state: Terminated}`. Both are broadcast. TUI receives both in order. |
| EC-301 | No TUI clients connected when transition occurs | `SessionStateChanged` posted to broker; broker fan-out has no subscribers; message discarded. No error. Connecting TUI clients receive current state in `InitialState`. |
| EC-302 | TUI client connects during a transition (between `SessionStateChanged` and `SessionListUpdate` for the same session) | The client's `InitialState` push includes the post-transition state. The in-flight `SessionStateChanged` / `SessionListUpdate` pair for the transition is NOT replayed for the new client. The new client may miss the transition event but will see correct state in `InitialState`. |
| EC-303 | `SessionCreation` wizard's session_id changes (session spawn failed and new session spawned) | Wizard tracks the new session_id; `SessionStateChanged` for the OLD session_id is ignored by the wizard (session_id filter). |
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
| Architecture Source | SS-session-manager.md v1.3.0 §Session lifecycle state machine (state transitions); SS-embedded-pty.md v1.2.0 §TUI AppMode Extensions (SessionCreation::Launching auto-transition to EmbeddedTerminal); SS-daemon-wiring-v2-delta.md v1.2.0 §IPC handler (broker publish path) |
| Cross-Ref | BC-2.09.008 (SessionCreation wizard auto-transition to EmbeddedTerminal on Running); BC-2.08.003 (kill → Terminating transition; 12s watchdog → Terminated); BC-2.05.003 (SessionListUpdate — emitted concurrently with SessionStateChanged for same transition) |
| Test Name | test_BC_2_08_008_session_state_changed_emitted_on_every_transition |

## Related BCs

- [BC-2.09.008] — depends on: wizard auto-advance uses SessionStateChanged{Running} as trigger
- [BC-2.08.003] — composes with: kill path (Terminating transitions) trigger SessionStateChanged
- [BC-2.05.003] — composes with: SessionListUpdate is the companion to SessionStateChanged for each transition
- [BC-2.05.009] — composes with: per-client isolated send buffer carries SessionStateChanged to TUI clients

## Architecture Anchors

- `architecture/SS-session-manager.md#session-lifecycle-state-machine` — complete state transition table
- `architecture/SS-embedded-pty.md#tui-appmode-extensions` — SessionCreation::Launching auto-advance rule
- `architecture/SS-daemon-wiring-v2-delta.md#ipc-handler-new-clienttoserver-variants` — broker publish path

## Story Anchor

S-TBD — Implement SessionStateChanged broadcast on every SessionEntry state transition (filled by story-writer)

## VP Anchors

VP-TBD — SessionStateChanged emission and TUI response integration tests (filled after VP creation)

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
