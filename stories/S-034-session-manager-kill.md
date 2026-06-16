---
document_type: story
level: L4
story_id: S-034
epic_id: EPIC-08
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 8
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-033]
blocks: []
target_module: monocle-runtime
subsystems: [SS-08]
behavioral_contracts: [BC-2.08.003, BC-2.08.008]
verification_properties: []
estimated_days: 4
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.003.md, version: "1.4.2"}
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.008.md, version: "1.3.4"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.6.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.08.003 (kill_session: SIGTERM via DaemonToHost::Kill within 500ms; Terminating/Terminated transitions; 12s watchdog) and BC-2.08.008 (SessionStateChanged{Terminating/Terminated} broadcast)"
# BC status at S-034 authoring time: BC-2.08.003 v1.4.2, BC-2.08.008 v1.3.3 — non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-034: SessionManager::kill_session — DaemonToHost::Kill Within 500ms; Terminating/Terminated Transitions; 12s Watchdog

## Narrative

As the monocle daemon, I want `SessionManager::kill_session()` to deliver `DaemonToHost::Kill`
to the target session-host within 500ms, transition the session immediately to
`SessionState::Terminating`, and then transition to `SessionState::Terminated` when the
session-host confirms exit — so that users have immediate observable feedback when killing a
session, the session-host has 10 seconds for graceful SIGTERM shutdown, and the 12-second
watchdog prevents `Terminating` from persisting indefinitely if the session-host stalls.

## Acceptance Criteria

### AC-001 (traces to BC-2.08.003 postcondition 1 — kill path selection by state; 500ms delivery)

`kill_session()` selects the kill path based on session state:
- **Running / Launching (host_conn established)**: uses existing `host_conn.writer` (control connection); sends `DaemonToHost::Kill` over it. No fresh UDS connect. Kill is delivered within 500ms. `SessionEntry.state` transitions to `SessionState::Terminating` atomically with the Kill send under the `SessionManager` mutex.
- **Launching (host_conn not yet established — rare race, `host_conn: None`)**: PID-based SIGTERM fallback (10s SIGTERM window, then SIGKILL). `Launching → Terminating` transition is immediate. Failure code: `"kill_failed"`.
- **Detached (`host_conn: None`)**: makes a fresh UDS connect to `<runtime_dir>/session-<session_id>.sock`; applies SO_PEERCRED peer-uid check BEFORE sending any message (Invariant 5); if uid matches: sends `DaemonToHost::Kill`. `Detached → Terminating` transition is immediate.

### AC-002 (traces to BC-2.08.003 postcondition 2 — SessionStateChanged{Terminating} before SessionListUpdate)

Immediately when Kill is sent, `SessionEntry.state` transitions to `Terminating` and:
- `ServerToClient::SessionStateChanged { session_id, new_state: Terminating }` is published to the broker BEFORE `ServerToClient::SessionListUpdate`.
- Both publications are under the `SessionManager` mutex (per BC-2.08.008 Invariant 4).
- TUI renders `[Terminating]` indicator on receipt of `SessionStateChanged{Terminating}`.

### AC-003 (traces to BC-2.08.003 postcondition 3 — session-host SIGTERM/SIGKILL delivery)

When the session-host receives `DaemonToHost::Kill`:
- Sends SIGTERM to the harness child process.
- Monitors child exit; if not exited within 10 seconds, sends SIGKILL.
- On child exit, sends `HostToDaemon::StateChanged { new_state: Terminated }` to daemon.
- Sends `HostToDaemon::Goodbye` and closes the UDS connection.
- Removes its UDS socket file.

(This AC is validated via MockSessionHostSpawner simulation; the session-host binary implementation is in scope for this story as the receive-side of `DaemonToHost::Kill`.)

### AC-004 (traces to BC-2.08.003 postcondition 4 — daemon receives StateChanged{Terminated})

When the daemon receives `HostToDaemon::StateChanged { new_state: Terminated }`:
- `SessionEntry.state` transitions from `Terminating` to `Terminated`.
- `session-state.json` is updated atomically (via `tempfile::persist`) to `state: "Terminated"`.
- `ServerToClient::SessionStateChanged { session_id, new_state: Terminated }` is published BEFORE `ServerToClient::SessionListUpdate` (both under mutex).
- The GC timer starts (10s; BC-2.08.005 — covered by S-037).

### AC-005 (traces to BC-2.08.003 postcondition 5 — 12s watchdog)

If the daemon does NOT receive `HostToDaemon::StateChanged` within 12 seconds of sending `DaemonToHost::Kill` (10s SIGTERM window + 2s buffer):
- A background `tokio::spawn` watchdog task fires.
- Forces `SessionEntry.state → Terminated`.
- Sends SIGKILL directly to the session-host PID (`SpawnedHostHandle.pid`).
- Updates `session-state.json` atomically.
- Publishes `ServerToClient::SessionStateChanged { session_id, new_state: Terminated }` BEFORE `ServerToClient::SessionListUpdate`.
- Starts GC timer.
Test uses `tokio::time::pause()` to advance virtual time without real delays.

### AC-006 (traces to BC-2.08.003 invariant 1 — kill is fire-and-confirm; does not block)

`kill_session()` MUST NOT block waiting for the harness child to exit. It sends Kill, transitions to `Terminating`, and returns `Ok(())` immediately. The confirmation (`Terminating → Terminated`) happens asynchronously.

### AC-007 (traces to BC-2.08.003 invariant 2 — idempotency: Terminated and Terminating sessions)

- `kill_session()` on a `Terminated` session returns `Ok(())` (idempotent; kill already complete; no duplicate Kill sent).
- `kill_session()` on a `Terminating` session returns `Ok(())` (idempotent; kill in-flight; watchdog already running; no duplicate Kill sent).

### AC-008 (traces to BC-2.08.003 invariant 3 — kill on Launching is allowed)

`kill_session()` on a `Launching` session is allowed and MUST succeed. The Kill is delivered either via the control connection (if `host_conn` is established) or via PID fallback (if not). Transition: `Launching → Terminating`.

### AC-009 (traces to BC-2.08.003 invariant 5 — SO_PEERCRED universal: no coverage holes)

SO_PEERCRED peer-credential check is applied on EVERY per-session UDS fresh-connect — including the kill-path fresh-connect for Detached sessions (EC-164). If the peer uid does not match the daemon uid: session immediately transitions to `Terminated`; `Ok(())` returned (kill is effectively complete — the session-host is assumed dead or rogue). No exceptions.

### AC-010 (traces to BC-2.08.003 edge case EC-164 — Detached kill: fresh connect + SO_PEERCRED)

When `kill_session()` is called on a `Detached` session:
- Makes a fresh UDS connect to the session-host socket.
- Applies SO_PEERCRED BEFORE sending any message.
- If uid matches: sends `DaemonToHost::Kill`; session transitions `Detached → Terminating` immediately.
- No intermediate `DaemonToHost::Attach` is needed — Kill can be sent directly on a fresh connect.

### AC-011 (traces to BC-2.08.003 edge case EC-166 — unknown session_id)

`kill_session()` called with a `session_id` not in the registry returns `Err(SessionError::SessionNotFound { session_id })`, which maps to wire code `"session_not_found"` via `session_error_to_code(IpcOp::Kill, e)`.

### AC-012 (traces to BC-2.08.008 postcondition 1 — no silent state transitions on kill path)

`SessionStateChanged` is emitted for EVERY kill-path transition:
- `Running → Terminating` (immediate on Kill send).
- `Detached → Terminating` (immediate on Kill send).
- `Launching → Terminating` (immediate on Kill send).
- `Terminating → Terminated` (on session-host confirmation OR 12s watchdog).
No silent transitions permitted.

## Tasks

- [ ] Implement `SessionManager::kill_session(&mut self, session_id: &str) -> Result<(), SessionError>` per SS-session-manager.md §Public API:
  - Running/Launching (host_conn established): use `host_conn.writer` to send `DaemonToHost::Kill`; transition → Terminating; emit `SessionStateChanged{Terminating}` + `SessionListUpdate` under mutex; spawn 12s watchdog `tokio::spawn`.
  - Launching (host_conn: None, rare race): PID-based SIGTERM (10s SIGTERM window, then SIGKILL) via `nix::sys::signal::kill`; transition → Terminating; emit `SessionStateChanged{Terminating}` + `SessionListUpdate`.
  - Detached: fresh UDS connect → SO_PEERCRED → send `DaemonToHost::Kill`; transition → Terminating; emit `SessionStateChanged{Terminating}` + `SessionListUpdate`; spawn 12s watchdog.
  - Terminated: return `Ok(())` idempotent.
  - Terminating: return `Ok(())` idempotent.
  - SessionNotFound: return `Err(SessionError::SessionNotFound { session_id })`.
- [ ] Implement 12s watchdog task: `tokio::spawn` + `tokio::time::sleep(Duration::from_secs(12))` → on timeout: SIGKILL to session-host PID → update sidecar → emit `SessionStateChanged{Terminated}` + `SessionListUpdate`; on `StateChanged::Terminated` received first: GC timer (defer to S-037).
- [ ] Implement session-host `DaemonToHost::Kill` handler in `monocle-session-host/src/main.rs`: SIGTERM to harness child; 10s wait; SIGKILL escalation; send `HostToDaemon::StateChanged{Terminated}`; send `HostToDaemon::Goodbye`; remove socket file.
- [ ] Add `ClientToServer::KillSession { session_id }` arm to IPC handler: call `kill_session()` → on error send `ServerToClient::Error`.
- [ ] Write unit test `test_BC_2_08_003_kill_session_sigterm_within_500ms`: mock session-host; verify `DaemonToHost::Kill` sent within 500ms; state → Terminating; on mock confirmation → Terminated; sidecar updated.
- [ ] Write unit test `test_BC_2_08_003_kill_session_idempotent_on_terminated`: kill on already-Terminated session → `Ok(())`.
- [ ] Write unit test `test_BC_2_08_003_kill_session_idempotent_on_terminating`: kill on Terminating → `Ok(())`.
- [ ] Write unit test `test_BC_2_08_003_12s_watchdog`: `tokio::time::pause()`; advance 12s; verify SIGKILL sent; state → Terminated; sidecar updated.
- [ ] Write unit test `test_BC_2_08_003_kill_detached_so_peercred`: fresh connect + SO_PEERCRED applied before Kill.
- [ ] Write unit test `test_BC_2_08_003_kill_session_not_found`: unknown session_id → `Err(SessionNotFound)`.
- [ ] Write unit test `test_BC_2_08_008_state_changed_ordering_on_kill`: `SessionStateChanged{Terminating}` arrives before `SessionListUpdate` in per-client channel.
- [ ] Write unit test `test_kill_during_launching_before_socket_bind`: `kill_session()` on Launching with `host_conn: None` → PID fallback SIGTERM → state → Terminating.
- [ ] Write unit test `test_kill_during_launching_after_socket_bind`: `kill_session()` on Launching with `host_conn: Some(_)` → Kill over control connection → state → Terminating.

## Previous Story Intelligence

- **S-033** (session-manager-spawn): `SessionManager`, `SessionEntry`, `SessionHostConnection`, `SessionState`, `SessionHostSpawner`, `session_error_to_code()`, `IpcOp`, and the IPC handler `SpawnSession` arm all exist. `SessionStateChanged` + `SessionListUpdate` broker publication pattern is established. Use the exact same mutex+double-publish pattern for kill-path emissions.
- The `SessionHostConnection.writer: Arc<Mutex<UnixStream>>` field is the existing control connection write half. Use `writer.lock().await` to serialize Kill message writes.
- The `SessionEntry.host_conn: Option<SessionHostConnection>` field was set to `None` at spawn time; the post-spawn monitor sets it to `Some(_)` asynchronously. Kill-path must handle both `Some` and `None` cases.

## Architecture Compliance Rules

- `kill_session()` MUST NOT block waiting for the harness child. It is fire-and-confirm: transition → Terminating, return `Ok(())`.
- SIGTERM deadline: 10 seconds for normal kill path (harness child may need clean shutdown); 2 seconds ONLY for pre-socket-bind orphan kill (S-033).
- SO_PEERCRED applies universally on EVERY per-session UDS fresh-connect. No exceptions for kill-path Detached sessions.
- `sidecar_write` updates on Terminating→Terminated MUST use `tempfile::persist`. Naked `std::fs::write` is forbidden.
- `kill_deadline_unix_ms` MUST be written to the sidecar when transitioning to `Terminating` (set to `now + 12s` as Unix epoch milliseconds). This enables re-discovery to use the absolute deadline rather than resetting the clock (BC-2.08.004 Invariant 7).
- `SessionStateChanged{Terminating}` MUST be published BEFORE `SessionListUpdate` for the kill transition. Both under the same mutex hold.
- Forbidden dependency: `monocle-runtime` MUST NOT depend on `monocle-tui`.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tokio` | `=1.52` (exact) | `tokio::spawn` for watchdog; `tokio::time::sleep`; async mutex | SS-deps-pin-manifest.md |
| `nix` | `"0.30"` | `nix::sys::signal::kill(Pid, Signal::SIGTERM/SIGKILL)` for PID fallback | SS-deps-pin-manifest.md |
| `serde_json` | `=1.0.149` (exact) | Sidecar JSON update on Terminating→Terminated transition | SS-deps-pin-manifest.md |
| `tempfile` | `"3"` | Atomic sidecar writes | SS-deps-pin-manifest.md |
| `thiserror` | `"2"` | `SessionError` enum (from S-033) | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to MODIFY (all from S-033):

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/session_manager/mod.rs` | Add `kill_session()` implementation; add watchdog task; add `kill_deadline_unix_ms` write on Terminating transition |
| `crates/monocle-runtime/src/ipc_handler.rs` | Add `ClientToServer::KillSession` arm |
| `crates/monocle-session-host/src/main.rs` | Add `DaemonToHost::Kill` handler in main event loop |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~3,500 |
| BC-2.08.003 | ~4,000 |
| BC-2.08.008 (kill-path sections) | ~2,000 |
| SS-session-manager.md (kill path, watchdog, host_conn rules, per-session UDS security) | ~8,000 |
| Existing session_manager code from S-033 | ~6,000 |
| session-host main.rs event loop (Kill handler) | ~2,000 |
| Test files | ~4,000 |
| **Total estimate** | **~30,000** |

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.08.003 | Session Kill — SIGTERM Delivered via DaemonToHost::Kill Within 500ms | v1.4.0 |
| BC-2.08.008 | SessionStateChanged — Daemon Emits on Every SessionState Transition; Delivered to All TUI Clients; Ordering Relative to SessionListUpdate | v1.3.0 |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `kill_session()` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (IPC send; OS signal; state mutation) |
| 12s watchdog task | `monocle-runtime/src/session_manager/mod.rs` | Effectful (tokio::spawn; SIGKILL; sidecar write; broker publish) |
| `DaemonToHost::Kill` handler | `crates/monocle-session-host/src/main.rs` | Effectful (SIGTERM/SIGKILL to child; StateChanged message; socket cleanup) |
| IPC handler `KillSession` arm | `monocle-runtime/src/ipc_handler.rs` | Effectful (IPC dispatch) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-162 | Session-host process is dead (crash) when `kill_session()` is called | PID liveness probe fails; session → Terminated; sidecar updated; `Ok(())` returned |
| EC-163 | Session-host UDS connect fails (socket deleted) | Session → Terminated; sidecar updated; `Ok(())` returned |
| EC-164 | Session state is `Detached` when `kill_session()` called | Fresh UDS connect + SO_PEERCRED + Kill; `Detached → Terminating` |
| EC-165 | Concurrent `kill_session()` calls for same session_id | First call sends Kill; second call is idempotent `Ok(())` (state already Terminating or Terminated) |
| EC-166 | `kill_session()` for unknown session_id | `Err(SessionError::SessionNotFound { session_id })` → wire code `"session_not_found"` |

## Subsystem Anchor Justifications

**SS-08 owns this story's scope** because `kill_session()` is a core `SessionManager` lifecycle operation defined in SS-session-manager.md §Public API.

**Dependency Anchor:**
- STORY-034 depends on S-033 because `SessionManager`, `SessionEntry`, `SessionHostConnection`, `SessionState`, `session_error_to_code()`, and the IPC handler skeleton all must exist before `kill_session()` can be added.
