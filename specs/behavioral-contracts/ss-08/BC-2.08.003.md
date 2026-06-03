---
document_type: behavioral-contract
level: L3
version: "1.1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md]
input-hash: "692beb0"
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

# Behavioral Contract BC-2.08.003: Session Kill — SIGTERM Delivered via DaemonToHost::Kill Within 500ms

## Description

`SessionManager::kill_session()` sends `DaemonToHost::Kill` to the target session-host's
per-session UDS socket. The session-host then sends SIGTERM to the harness child process
and initiates clean shutdown. The kill command must be delivered within 500ms of the call.
The session transitions directly from `Running` (or `Detached`, `Launching`) to
`SessionState::Terminated` — there is no `SessionState::Killed` intermediate state.
`Terminated` is set when the session-host confirms exit via `HostToDaemon::StateChanged`.
The sidecar is not immediately deleted — GC handles deletion after the GC timer expires
(BC-2.08.005).

## Preconditions

1. A `SessionEntry` exists in the registry for `session_id` with state `Running` or `Detached`.
2. The session-host process is alive.
3. `SessionManager.sessions[session_id].host_conn` is `Some(_)` (daemon is attached) OR the
   daemon can re-attach (Detached sessions require a fresh UDS connect before sending Kill).

## Postconditions

1. `SessionManager` sends `DaemonToHost::Kill` over the per-session UDS within 500ms of
   `kill_session()` being invoked.
2. A `ServerToClient::SessionListUpdate` IPC message is published to the broker immediately
   after the Kill command is sent (to notify TUI clients that termination is in progress).
3. When the session-host receives `DaemonToHost::Kill`:
   a. It sends SIGTERM to the harness child process.
   b. It monitors child exit. If child has not exited within 10 seconds, it sends SIGKILL.
   c. On child exit, the session-host sends `HostToDaemon::StateChanged { new_state: Terminated }`
      to the daemon.
   d. It sends `HostToDaemon::Goodbye` and closes the UDS connection.
   e. It removes its UDS socket file (`<runtime_dir>/session-<uuid>.sock`).
4. When the daemon receives `HostToDaemon::StateChanged { new_state: Terminated }` for the
   killed session:
   a. `SessionEntry.state` transitions to `SessionState::Terminated`.
   b. `session-state.json` is updated (atomically via `tempfile::persist`) to reflect
      `state: "Terminated"`.
   c. A second `ServerToClient::SessionListUpdate` IPC message is published.
   d. The GC timer starts (BC-2.08.005).

## Invariants

1. `kill_session()` MUST NOT block waiting for the harness child to exit. It is fire-and-confirm:
   send Kill, return `Ok(())`. The `SessionEntry` remains in its prior state (e.g., `Running`)
   until the session-host confirms exit via `HostToDaemon::StateChanged { new_state: Terminated }`.
   `Terminated` is the single terminal state — there is no `Killed` intermediate state.
2. `kill_session()` on a `Terminated` session MUST return `Ok(())` (idempotent).
   The kill is already complete; no duplicate Kill message is sent.
3. `kill_session()` on a `Launching` session is allowed; the Kill is delivered to the
   session-host, which terminates any partially-started harness child.
4. SIGTERM is used (not SIGKILL). The harness child has an opportunity for clean shutdown
   (e.g., flushing output, removing temp files). If the child does not exit within 10 seconds
   of SIGTERM, the session-host sends SIGKILL to escalate. The kill path is:
   Running (or Detached/Launching) → [Kill sent, awaiting confirmation] → Terminated.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-162 | Session-host process is dead (crash) when kill_session() is called | `kill(pid, None)` probe fails; session immediately transitions to `Terminated`; sidecar updated; GC timer starts; `Ok(())` returned — kill is effectively a no-op on a dead session |
| EC-163 | Session-host UDS connect fails (socket deleted) when attempting kill | Session transitions to `Terminated` immediately; sidecar updated; `Ok(())` returned |
| EC-164 | Session state is `Detached` when kill_session() is called | Daemon re-connects to session-host UDS (fresh connect + Attach/Kill sequence); Kill delivered; session transitions → Terminated on confirmation |
| EC-165 | Concurrent kill_session() calls for the same session_id | First call sends Kill; second call is a no-op (state already in-flight-toward-Terminated or already Terminated); returns `Ok(())` |
| EC-166 | kill_session() called for unknown session_id | Returns `Err(SessionError::SessionNotFound { session_id })` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `kill_session("existing-running-session")` with live session-host (mock) | `Ok(())`; `DaemonToHost::Kill` sent within 500ms; on mock confirmation: session state → Terminated; sidecar updated | happy-path |
| `kill_session("nonexistent-id")` | `Err(SessionError::SessionNotFound {...})` | error |
| `kill_session("already-terminated-session")` | `Ok(())` — idempotent | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `DaemonToHost::Kill` sent within 500ms of `kill_session()` call | unit |
| VP-TBD | Session state = Terminated after session-host confirms exit (no Killed intermediate state) | unit |
| VP-TBD | kill_session() on unknown session → `SessionNotFound` error | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — this BC defines the kill operation, a core session lifecycle action named explicitly in CAP-008 |
| Architecture Module | monocle-runtime (SessionManager `kill_session()`); monocle-session-host (SIGTERM delivery) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v1.2.0 §SessionManager §Public API (kill_session signature); §Per-session UDS protocol (DaemonToHost::Kill, HostToDaemon::StateChanged, Goodbye) |
| Test Name | test_BC_2_08_003_kill_session_sigterm_within_500ms |

## Related BCs

- [BC-2.08.001] — depends on: session was spawned by spawn_session() and is in the registry
- [BC-2.08.005] — composes with: GC timer starts when Terminated state is reached after kill

## Architecture Anchors

- `architecture/SS-session-manager.md#public-api` — kill_session() signature
- `architecture/SS-session-manager.md#per-session-uds-protocol` — DaemonToHost::Kill message

## Story Anchor

S-TBD — Implement SessionManager::kill_session() (filled by story-writer)

## VP Anchors

VP-TBD — kill_session() timing and state transition tests (filled after VP creation)

## §Trace v1.1.0

**Adversarial pass-1 fix — Remove SessionState::Killed intermediate state** (2026-06-03):
- Description updated: removed `SessionState::Killed` as an intermediate/terminal state.
  Kill path is now `Running → [awaiting confirmation] → Terminated` (or `Detached/Launching → Terminated`).
- Postconditions restructured: removed optimistic Killed transition. PC-2 now is SessionListUpdate
  on Kill-sent; PC-3 is the session-host SIGTERM/SIGKILL delivery; PC-4 is daemon receiving
  StateChanged::Terminated which sets Terminated state, updates sidecar, and starts GC.
- Invariant 1: removed "update state to Killed" — the session stays in prior state until
  confirmed exit.
- Invariant 2: removed "Killed" from idempotency condition; only "Terminated" now.
- Edge cases and test vectors updated to remove all Killed state references.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.003 authored for SS-08 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): 500ms bound from SS-session-manager.md architect proposal preserved.
  SIGTERM → SIGKILL escalation (10s) is specified in Invariant 4 per production-grade default
  (omitting escalation would leave zombie harness children in the wild).
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
