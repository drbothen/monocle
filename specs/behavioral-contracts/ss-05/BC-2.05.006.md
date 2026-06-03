---
document_type: behavioral-contract
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T04:00:00Z
phase: phase-1-expansion
inputs: [prd-expansion-scope.md, architecture/SS-ipc.md, architecture/ARCH-INDEX.md]
input-hash: "0292752"
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

# Behavioral Contract BC-2.05.006: TUI Reconnects After Daemon Restart

## Description

When the TUI detects a connection loss to the daemon (EOF, `BrokenPipe`, or `ConnectionReset`
on the UDS stream), it enters a reconnection loop with exponential backoff. The TUI renders a
`[daemon: reconnecting...]` status bar indicator during the reconnect window. After each failed
attempt, the TUI re-reads the lock file to discover whether a new daemon has started at a new
port. If reconnection succeeds within 5 seconds, the daemon sends a fresh `InitialState` push
and the TUI rebuilds its complete state. If reconnection fails for 5 seconds, the TUI
transitions to "daemon offline" mode and polls the lock file every 5 seconds for a new daemon.

## Preconditions

1. A TUI client was connected to the daemon's UDS socket and receiving push messages.
2. The `read_framed` call in the TUI's receive loop returns an error: `UnexpectedEof`,
   `BrokenPipe`, or `ConnectionReset`.
3. The SOQ-3 overlay-clear signal (BC-2.05.007) has been emitted by `UdsTransport` before
   the reconnect loop begins.

## Postconditions

**Reconnect loop behavior:**
1. Immediately upon detecting connection loss, the TUI emits `TransportEvent::Disconnected`.
   The SOQ-3 handler (BC-2.05.007) clears the overlay stack in response to this event, before
   any reconnect attempt begins.
2. The TUI renders `[daemon: reconnecting...]` in the status bar (replacing any prior status).
3. The TUI re-reads `<runtime_dir>/monocle.lock` after each failed reconnect attempt. If the
   lock file has changed (new `port`, new `authToken`, new `pid`), the TUI uses the new socket
   path for subsequent connection attempts. This handles the daemon restart case where the new
   daemon binds a different port and creates a new UDS socket at the same path.
4. The TUI attempts reconnection with exponential backoff:
   - Attempt 1: wait 250ms before retry.
   - Attempt 2: wait 500ms before retry.
   - Attempt 3: wait 1000ms before retry.
   - Attempt 4+: wait 2000ms before retry (cap at 2 seconds).
5. The total reconnect window is 5 seconds. If no connection succeeds within 5 seconds from
   the first disconnect detection, the TUI transitions to "daemon offline" mode:
   - Status bar renders `[daemon: offline]`.
   - TUI enters passive mode (observe-only; no IPC push messages received).
   - TUI polls the lock file every 5 seconds. When a new lock file is detected (new daemon
     started), the TUI re-enters the reconnect loop from step 1.

**Successful reconnect behavior:**
6. On successful reconnect, the daemon sends a fresh `ServerToClient::InitialState` push
   (BC-2.05.002). The TUI rebuilds its complete state from this message.
7. If the TUI was in `AppMode::Overlay` when the disconnect occurred, it transitions to
   `AppMode::Dashboard` on reconnect. (The overlay stack was already cleared in step 1;
   `AppMode::Overlay` with an empty stack is an invalid state.)
8. Status bar reverts to normal (no `[daemon: reconnecting...]` or `[daemon: offline]`
   indicator) after successful reconnect and `InitialState` receipt.

## Invariants

1. The SOQ-3 overlay clear (BC-2.05.007) ALWAYS happens before the first reconnect attempt.
   The sequence is: `connection loss → overlay cleared → reconnect loop begins → (success) InitialState → TUI rebuilds`.
   This ordering is never reversed.
2. The TUI never sends a `ClientToServer::PermissionDecision` for a prompt that was in the
   overlay stack at the time of disconnect. The overlay is cleared before reconnect; the fresh
   `InitialState` on reconnect contains only prompts still pending in the daemon registry.
3. The lock file re-read after each failed attempt ensures the TUI discovers a new daemon
   automatically, without requiring user intervention.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Daemon crashes and restarts; new daemon starts within 3 seconds | TUI retries lock file after first failed reconnect; discovers new lock file; connects to new UDS socket; receives fresh `InitialState`. Reconnect succeeds within 5-second window. |
| EC-002 | Daemon crashes and never restarts (process killed permanently) | TUI exhausts 5-second reconnect window; transitions to "daemon offline" mode; polls lock file every 5 seconds indefinitely. No crash or runaway CPU usage. |
| EC-003 | Daemon restarts but new UDS socket is at the same path `<runtime_dir>/monocle.sock` | TUI re-reads lock file; confirms new `pid` and `port`; connects to same path. Connection succeeds because new daemon replaced the stale socket at bind time (BC-2.05.001 Postcondition 3). |
| EC-004 | TUI is in `AppMode::Overlay` with 2 queued prompts when disconnect occurs | SOQ-3 clears the VecDeque; AppMode transitions to Dashboard. On reconnect, `InitialState.overlay_stack` re-delivers any still-pending prompts (those not yet timed out in the daemon). AppMode transitions back to Overlay if prompts are delivered. |
| EC-005 | Reconnect attempt fails because lock file does not exist yet (daemon still starting) | TUI waits for the next backoff interval; retries connection. If the daemon finishes starting within the 5-second window, TUI connects successfully. |
| EC-006 | User sends a `PermissionDecision` IPC message during the reconnect window (before connection re-established) | The TUI's send loop returns an error (no active connection). The decision is discarded. The overlay was cleared (SOQ-3), so no `PermptModal` remains for the user to interact with. No race condition. |
| EC-007 | Daemon restarts; new daemon has different runtime_dir (MONOCLE_RUNTIME_DIR changed between restarts) | TUI reads the new lock file at the old path. If the old runtime_dir path no longer has a lock file, TUI enters "daemon offline" mode. Administrator-level runtime_dir changes are not handled automatically; user must restart TUI. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| UDS connection drops (EOF); daemon restarts within 2 seconds | TUI shows `[daemon: reconnecting...]`; re-reads lock file; connects to new socket; receives `InitialState`; reverts to normal status | happy-path |
| UDS connection drops; no daemon restarts for 10 seconds | TUI shows `[daemon: reconnecting...]` for 5 seconds; transitions to `[daemon: offline]`; polls lock file every 5 seconds | happy-path |
| TUI in Overlay mode when disconnect occurs | Overlay cleared (SOQ-3); AppMode resets to Dashboard; reconnecting indicator shown | edge-case |
| Daemon restarts 4 times within 30 seconds (crash loop) | TUI reconnects on each restart; shows reconnecting indicator each time; no cumulative state leakage between reconnects | edge-case |
| Lock file absent during entire reconnect window | 5-second window exhausted; TUI transitions to offline mode | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | SOQ-3 overlay clear happens before first reconnect attempt (invariant 1) | integration |
| VP-TBD | Exponential backoff: 250ms → 500ms → 1000ms → 2000ms cap | unit (mock clock) |
| VP-TBD | Lock file re-read after each failed attempt; new daemon discovered | integration |
| VP-TBD | 5-second window exhaustion transitions to "daemon offline" mode | integration |
| VP-TBD | Fresh `InitialState` on reconnect causes full TUI state rebuild | integration |
| VP-TBD | `AppMode::Overlay` resets to `Dashboard` after overlay-clearing reconnect | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability §SS-05 — this BC specifies the reconnection behavior that makes the IPC transport resilient to daemon restarts, which is a key availability property of the internal transport |
| L2 Domain Invariants | DI-002 (lock file must be present before connections accepted — this BC's lock-file re-read after each retry enforces DI-002 from the client side: TUI only connects when a valid lock file exists) |
| Architecture Module | monocle-ipc (UdsTransport reconnect loop, TransportEvent::Disconnected) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-ipc.md v1.11.0 §Reconnection Behavior; SS-ipc.md v1.11.0 §SOQ-3 Overlay Clear on Disconnect |
| Cross-Ref | BC-2.05.001 (new daemon rebinds same socket path after stale removal); BC-2.05.002 (InitialState on reconnect); BC-2.05.007 (SOQ-3 overlay clear — happens before reconnect loop); BC-2.01.005 (lock file re-read to discover new daemon) |
| Test File | `monocle-ipc/tests/reconnect.rs` |
| Test Name | `test_BC_2_05_006_tui_reconnects_after_daemon_restart` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.007] — depends on: SOQ-3 overlay clear must happen before reconnect loop begins (Invariant 1)
- [BC-2.05.001] — depends on: new daemon removes stale socket before rebind, enabling TUI reconnect
- [BC-2.05.002] — composes with: successful reconnect triggers fresh InitialState push
- [BC-2.01.005] — depends on: lock file re-read semantics (pid liveness check pattern)

## Architecture Anchors

- `architecture/SS-ipc.md#reconnection-behavior` — exponential backoff parameters (250ms → 2s cap, 5-second window), lock-file re-read, "daemon offline" polling, AppMode reset to Dashboard
- `architecture/SS-ipc.md#soq-3-overlay-clear-on-disconnect` — sequence ordering: connection loss → overlay cleared → reconnect loop

## Story Anchor

S-TBD — Implement TUI IPC reconnect loop with exponential backoff and lock-file re-read (filled by story-writer)

## VP Anchors

VP-TBD — Reconnect loop and offline-mode transition verification properties (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T04:00:00Z):
- BC-2.05.006 authored for SS-05 IPC subsystem per `prd-expansion-scope.md §3.2` and
  `SS-ipc.md §Reconnection Behavior + §SOQ-3 Overlay Clear on Disconnect`.
- Covers: connection-loss detection, SOQ-3 ordering invariant (clear before reconnect),
  `[daemon: reconnecting...]` status bar indicator, exponential backoff (250ms → 2s cap),
  5-second total window, lock-file re-read on each retry, "daemon offline" passive mode
  with 5-second poll, AppMode Dashboard reset after overlay clear, InitialState rebuild on
  successful reconnect.
- 7 edge cases documented (EC-001..EC-007).
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
- Architecture Source row: `SS-ipc.md v1.4.0 §Reconnection Behavior` → `SS-ipc.md v1.9.0 §Reconnection Behavior`; `SS-ipc.md v1.4.0 §SOQ-3 Overlay Clear on Disconnect` → `SS-ipc.md v1.9.0 §SOQ-3 Overlay Clear on Disconnect`.
- Plain version-pin refresh. No substantive content propagation required — §Reconnection Behavior and §SOQ-3 Overlay Clear on Disconnect section headings and content anchors are unchanged between v1.4.0 and v1.9.0.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.
