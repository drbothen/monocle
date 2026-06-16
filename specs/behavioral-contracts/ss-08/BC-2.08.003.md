---
document_type: behavioral-contract
level: L3
version: "1.4.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md]
input-hash: "081ebe8"
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
The session transitions from `Running` (or `Detached`, `Launching`) to
`SessionState::Terminating` when Kill is sent — an observable in-flight kill state that
prevents the user from wondering whether the kill was sent. The session transitions from
`Terminating` to `Terminated` when the session-host confirms exit via
`HostToDaemon::StateChanged`. A 12-second watchdog fires if the session-host does not
confirm exit (10s SIGTERM window + 2s buffer), after which the daemon forces Terminated
and sends SIGKILL directly to the session-host PID. The sidecar is not immediately deleted
— GC handles deletion after the GC timer expires (BC-2.08.005).

## Preconditions

1. A `SessionEntry` exists in the registry for `session_id` with state `Running`, `Detached`,
   or `Launching`.
2. The session-host process is alive.
3. `SessionManager.sessions[session_id].host_conn` is `Some(_)` (control connection established — true for Running and Launching sessions after the post-spawn monitor has connected) OR the session is `Detached` (where `host_conn` is `None`; a fresh UDS connect + SO_PEERCRED check is required before sending Kill — see EC-164 and Invariant 5). For Launching sessions in the brief window before the post-spawn monitor has connected (`host_conn: None`), `kill_session()` falls back to PID-based SIGTERM/SIGKILL (see SS-session-manager.md §Kill-path host_conn rules).

## Postconditions

1. `SessionManager` determines the kill path based on session state:
   **Running / Launching state (host_conn established):** Uses the existing `host_conn.writer` (control connection already live); sends `DaemonToHost::Kill` over the control connection. No fresh UDS connect needed. `proxy_task` may be `None` (Launching) or `Some(_)` (Running) — kill uses the writer only.

   **Launching state (host_conn not yet established — rare race):** If `kill_session()` is called during the brief window before the post-spawn monitor has connected to the session-host UDS socket (`host_conn` is still `None`), the daemon falls back to PID-based SIGTERM/SIGKILL. This is the same mechanism as the pre-socket-bind orphan kill but with a 10s SIGTERM window (harness child may be starting). Transition: `Launching → Terminating` is immediate; `SessionStateChanged{Terminating}` + `SessionListUpdate` published. Failure code: `kill_failed`.

   - **Detached state:** `host_conn` is `None`; makes a fresh UDS connect to the session-host
     socket; applies SO_PEERCRED peer-credential check BEFORE sending any message (Invariant 5);
     if uid matches: sends `DaemonToHost::Kill`. This is the explicit kill-path-selection rule
     for Detached sessions — independent of the re-discovery Detached-preservation behavior
     (re-discovery preserves Detached; kill is still correct on a Detached session via fresh
     connect). A Detached session has `host_conn: None`; it is alive but not currently streamed.
   Kill is sent within 500ms of `kill_session()` being invoked. `SessionEntry.state` transitions
   to `SessionState::Terminating` atomically with the Kill send.
2. `ServerToClient::SessionStateChanged { session_id, new_state: Terminating }` is published
   to the broker BEFORE `ServerToClient::SessionListUpdate` — both under the `SessionManager`
   mutex per BC-2.08.008 Invariant 4 and SS-daemon-wiring-v2-delta.md v1.11.4 §3b. TUI renders
   `[Terminating]` indicator on receipt of `SessionStateChanged{Terminating}`.
3. When the session-host receives `DaemonToHost::Kill`:
   a. It sends SIGTERM to the harness child process.
   b. It monitors child exit. If child has not exited within 10 seconds, it sends SIGKILL.
   c. On child exit, the session-host sends `HostToDaemon::StateChanged { new_state: Terminated }`
      to the daemon.
   d. It sends `HostToDaemon::Goodbye` and closes the UDS connection.
   e. It removes its UDS socket file (`<runtime_dir>/session-<uuid>.sock`).
4. When the daemon receives `HostToDaemon::StateChanged { new_state: Terminated }` for the
   killed session:
   a. `SessionEntry.state` transitions from `SessionState::Terminating` to
      `SessionState::Terminated`.
   b. `session-state.json` is updated (atomically via `tempfile::persist`) to reflect
      `state: "Terminated"`.
   c. `ServerToClient::SessionStateChanged { session_id, new_state: Terminated }` is published
      to the broker BEFORE `ServerToClient::SessionListUpdate` (both under the `SessionManager`
      mutex per BC-2.08.008 Invariant 4).
   d. The GC timer starts (BC-2.08.005).
5. **12-second watchdog:** If the daemon does not receive `HostToDaemon::StateChanged` within
   12 seconds of sending `DaemonToHost::Kill` (10s SIGTERM window + 2s buffer for the session-
   host itself to clean up), the daemon:
   a. Forces `SessionEntry.state` → `SessionState::Terminated`.
   b. Sends SIGKILL directly to the session-host PID (`SpawnedHostHandle.pid`) to release PTY
      resources, since the session-host may have stalled (harness child not responding to SIGKILL).
   c. Updates `session-state.json` atomically.
   d. Publishes `ServerToClient::SessionStateChanged { session_id, new_state: Terminated }`
      BEFORE `ServerToClient::SessionListUpdate` (per BC-2.08.008 Invariant 4).
   e. Starts GC timer (BC-2.08.005).
   The watchdog ensures `Terminating` state never persists indefinitely.

## Invariants

1. `kill_session()` MUST NOT block waiting for the harness child to exit. It is fire-and-confirm:
   send Kill, transition session to `Terminating`, return `Ok(())`. The kill path is:
   `Running | Detached | Launching → Terminating → Terminated`. `Terminating` is the observable
   in-flight state; `Terminated` is the confirmed terminal state.
2. `kill_session()` on a `Terminated` or `Terminating` session MUST return `Ok(())` (idempotent).
   For `Terminated`: kill is complete; no duplicate Kill message sent. For `Terminating`: kill
   is in-flight; no duplicate Kill message sent; watchdog already running.
3. `kill_session()` on a `Launching` session is allowed; the Kill is delivered to the
   session-host, which terminates any partially-started harness child. Transition:
   `Launching → Terminating`.
4. SIGTERM is used (not SIGKILL) for the harness child. The harness child has an opportunity
   for clean shutdown (e.g., flushing output, removing temp files). If the child does not exit
   within 10 seconds of SIGTERM, the session-host sends SIGKILL to escalate. Kill path:
   Running/Detached/Launching → Terminating → Terminated (on session-host confirmation OR
   12s watchdog).
5. **SO_PEERCRED applies to EVERY per-session UDS fresh-connect — no coverage holes:**
   SO_PEERCRED / LOCAL_PEERPID peer-credential check is applied on ANY per-session UDS
   connection attempt — whether that connection is for attach, re-discovery, kill on a Detached
   session (EC-164), or any other operation. `kill_session()` on a `Detached` session requires
   a fresh UDS connect (since `host_conn` is `None`); this connect MUST apply SO_PEERCRED before
   sending `DaemonToHost::Kill`. A Detached session with `host_conn: None` is still a valid,
   alive session-host — the re-discovery Detached-preservation behavior (BC-2.08.004 I3-005)
   does NOT exempt kill from SO_PEERCRED. Failure (uid mismatch) → session treated as dead;
   transition to `Terminated` immediately; `Ok(())` returned (sidecar updated, GC timer
   started). Per SS-session-manager.md v2.6.1 §Per-session UDS security item 1: "SO_PEERCRED
   applies universally — attach, re-discovery, kill/detach re-connect. No exceptions."

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-162 | Session-host process is dead (crash) when kill_session() is called | `kill(pid, None)` probe fails; session immediately transitions to `Terminated`; sidecar updated; GC timer starts; `Ok(())` returned — kill is effectively a no-op on a dead session |
| EC-163 | Session-host UDS connect fails (socket deleted) when attempting kill | Session transitions to `Terminated` immediately; sidecar updated; `Ok(())` returned |
| EC-164 | Session state is `Detached` when kill_session() is called | Daemon makes a fresh UDS connect to the session-host socket; applies SO_PEERCRED peer-uid check BEFORE sending any message (Invariant 5); if uid matches: send `DaemonToHost::Kill`; session transitions `Detached → Terminating` immediately, `Terminating → Terminated` on session-host confirmation. No intermediate `Attach` needed — Kill can be sent directly on a fresh connect without prior `Attach`. |
| EC-165 | Concurrent kill_session() calls for the same session_id | First call sends Kill; second call is a no-op (state already in-flight-toward-Terminated or already Terminated); returns `Ok(())` |
| EC-166 | kill_session() called for unknown session_id | Returns `Err(SessionError::SessionNotFound { session_id })` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `kill_session("existing-running-session")` with live session-host (mock) | `Ok(())`; `DaemonToHost::Kill` sent within 500ms; session state → Terminating immediately; on mock confirmation: session state → Terminated; sidecar updated | happy-path |
| `kill_session("nonexistent-id")` | `Err(SessionError::SessionNotFound {...})` | error |
| `kill_session("already-terminated-session")` | `Ok(())` — idempotent | edge-case |
| `kill_session("terminating-session")` | `Ok(())` — idempotent; no duplicate Kill | edge-case |
| 12s pass without session-host confirmation | Session forced to Terminated; SIGKILL to session-host PID; sidecar updated; GC timer started | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `DaemonToHost::Kill` sent within 500ms of `kill_session()` call | unit |
| VP-TBD | Session state → Terminating immediately on Kill sent; → Terminated on session-host confirmation | unit |
| VP-TBD | 12s watchdog fires: session forced to Terminated; SIGKILL sent to session-host PID | unit (tokio::time::pause) |
| VP-TBD | kill_session() on Detached session: SO_PEERCRED check before Kill; uid mismatch → Terminated | unit |
| VP-TBD | kill_session() on unknown session → `SessionNotFound` error | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — this BC defines the kill operation, a core session lifecycle action named explicitly in CAP-008 |
| Architecture Module | monocle-runtime (SessionManager `kill_session()`); monocle-session-host (SIGTERM delivery) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v2.6.1 §SessionManager §Public API (kill_session signature); §Kill-path host_conn rules (post-spawn monitor; PID fallback for Launching race window); §Per-session UDS protocol (DaemonToHost::Kill, HostToDaemon::StateChanged, Goodbye); §Per-session UDS security item 1 (SO_PEERCRED universal — no coverage holes); SS-daemon-wiring-v2-delta.md v1.11.4 §3b (SessionStateChanged emission rule: Terminating then SessionListUpdate) |
| Test Name | test_BC_2_08_003_kill_session_sigterm_within_500ms |

## Related BCs

- [BC-2.08.001] — depends on: session was spawned by spawn_session() and is in the registry
- [BC-2.08.005] — composes with: GC timer starts when Terminated state is reached after kill

## Architecture Anchors

- `architecture/SS-session-manager.md#public-api` — kill_session() signature
- `architecture/SS-session-manager.md#per-session-uds-protocol` — DaemonToHost::Kill message

## Story Anchor

S-034 — Implement SessionManager::kill_session()

## VP Anchors

VP-TBD — kill_session() timing and state transition tests (filled after VP creation)

## §Trace v1.4.1

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-034** (2026-06-15):
- Story Anchor filled from Phase-2 Burst A story decomposition. No behavioral content changed.

## §Trace v1.4.0

**F-P50-001 — host_conn lifecycle resolution: post-spawn monitor; Launching race-window PID fallback** (2026-06-14):
- Precondition 3 rewritten: `host_conn` is `Some(_)` for Running and Launching sessions after the post-spawn monitor has connected. For Detached sessions `host_conn` is `None`. For Launching sessions in the brief window before the post-spawn monitor has connected, `kill_session()` falls back to PID-based SIGTERM/SIGKILL per SS-session-manager.md §Kill-path host_conn rules.
- PC-1 rewritten into three named cases: (1) Running/Launching (host_conn established) — uses `host_conn.writer`; notes `proxy_task` may be `None` (Launching) or `Some(_)` (Running); (2) Launching (host_conn not yet established — rare race) — PID fallback with 10s SIGTERM window; Launching → Terminating immediate; failure code `kill_failed`; (3) Detached — fresh UDS connect + SO_PEERCRED (unchanged).
- Architecture Source pins bumped: SS-session-manager.md v2.4.0 → v2.5.0 (adds §Kill-path host_conn rules); SS-daemon-wiring-v2-delta.md v1.11.0 → v1.11.1.
- Cited symbols verified: EC-164 (line 133 of this BC), Invariant 5 (line 115), SS-session-manager.md §Kill-path host_conn rules (new in v2.5.0 per architect).
- Minor bump: v1.3.1 → v1.4.0 (normative addition — new kill sub-path for Launching race window).

SE-16d monotonicity: v1.4.0 timestamp 2026-06-14 > v1.3.1 timestamp 2026-06-13. PASS.

## §Trace v1.3.1

**Arch-source pin v1.9.0→v1.9.1** (2026-06-13 / D-277):
- Arch-source pin: SS-daemon-wiring-v2-delta.md v1.9.0 → v1.9.1 (all active citations).
- No behavioral content changed. Patch bump only.

## §Trace v1.3.0

**Adversarial Pass 3 fixes — C3-001 (SessionStateChanged{Terminating} before SessionListUpdate) + I3-006 (kill path-selection + SO_PEERCRED no-hole)** (2026-06-03):
- C3-001: PC-1 updated with explicit kill-path-selection rule for Running/Launching (existing
  host_conn) vs Detached (fresh UDS connect). PC-2 updated: `SessionStateChanged{Terminating}`
  published BEFORE `SessionListUpdate` (ordered pair per BC-2.08.008 Invariant 4). PC-4c
  updated: `SessionStateChanged{Terminated}` published BEFORE `SessionListUpdate` on
  StateChanged receipt. PC-5d updated: same ordering for watchdog-forced Terminated.
- I3-006: Invariant 5 rewritten to make SO_PEERCRED coverage explicit and gapless: applies
  on ANY per-session UDS fresh-connect (attach, re-discovery, kill on Detached). The
  Detached re-discovery preservation (I3-005) does NOT exempt kill from SO_PEERCRED. A
  Detached session has host_conn:None; kill requires a fresh connect; that connect MUST apply
  SO_PEERCRED before any message. No coverage hole.
- Architecture Source updated to SS-session-manager.md v1.5.0 and SS-daemon-wiring-v2-delta.md v1.3.1.

## §Trace v1.2.0

**Architect-delegated BC edits — Terminating state, 12s watchdog, SO_PEERCRED on kill-path** (2026-06-03):
- Architect delegated these edits from SS-session-manager.md v1.3.0 §Terminating state (I2-004)
  and §Per-session UDS security (I5) + C2-005(b) clarification.
- Description: kill path updated from `Running → Terminated` to `Running/Detached/Launching →
  Terminating → Terminated`. Added 12s watchdog description.
- Precondition 1: `Launching` added to valid initial states for kill.
- Precondition 3: SO_PEERCRED note for Detached kill path added.
- PC-1: session transitions to `Terminating` on Kill sent (not after confirmation).
- PC-5 (new): 12-second watchdog specification. Forces Terminated + SIGKILL to session-host
  PID if no StateChanged::Terminated within 12s.
- Invariant 1: kill path now `Running|Detached|Launching → Terminating → Terminated`.
- Invariant 2: idempotency expanded to cover `Terminating` state (no duplicate Kill).
- Invariant 5 (new): SO_PEERCRED mandatory on kill-path fresh-connect for Detached sessions.
  Per SS-session-manager.md v1.3.0 §Per-session UDS security item 1 (kill-path specific note).
- EC-164: rewritten — fresh connect, SO_PEERCRED check BEFORE Kill, no intermediate Attach.
  `Detached → Terminating` transition specified explicitly.
- Test vectors: added Terminating idempotency and watchdog test cases.
- VP table: added Terminating state transition, watchdog, and SO_PEERCRED verification properties.

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

## §Trace v1.4.2

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.4.2 timestamp >= v1.4.1. PASS.
