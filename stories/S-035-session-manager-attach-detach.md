---
document_type: story
level: L4
story_id: S-035
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
behavioral_contracts: [BC-2.08.007, BC-2.08.008]
verification_properties: []
estimated_days: 4
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.007.md, version: "1.5.3"}
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.008.md, version: "1.3.4"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.6.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.08.007 (attach_session: chunked scrollback protocol; SO_PEERCRED; detach_session: session-host survives) and BC-2.08.008 (SessionStateChanged{Running/Detached} broadcast on attach/detach)"
# BC status at S-035 authoring time: BC-2.08.007 v1.5.3, BC-2.08.008 v1.3.3 — non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-035: SessionManager::attach_session and detach_session — Chunked Scrollback, SO_PEERCRED, Session-Host Stays Alive

## Narrative

As the monocle daemon, I want `SessionManager::attach_session()` to re-connect to a Detached
session-host via SO_PEERCRED-verified UDS, receive the full `ScrollbackChunk*` +
`ScrollbackDumpComplete` chunked scrollback sequence within 5 seconds, start the PTY proxy task,
and transition the session to `Running` — and `detach_session()` to send `DaemonToHost::Detach`,
abort the proxy task, and persist `Detached` state without terminating the session-host — so
that monocle can offer seamless background session suspension and resumption with full visual
fidelity at re-attach.

## Acceptance Criteria

### AC-001 (traces to BC-2.08.007 attach postcondition 1-2 — UDS connect + SO_PEERCRED before any message)

`attach_session()` connects to `<runtime_dir>/session-<session_id>.sock` and verifies
SO_PEERCRED peer uid matches daemon uid BEFORE sending any message. If uid mismatches:
abort the attach; session treated as dead; `Err(SessionError::SessionHostDead { session_id })`.

### AC-002 (traces to BC-2.08.007 attach postcondition 3-4 — DaemonToHost::Attach; full ScrollbackChunk* + ScrollbackDumpComplete within 5s)

After SO_PEERCRED verification, sends `DaemonToHost::Attach`. Then receives the full
`HostToDaemon::ScrollbackChunk*` + `HostToDaemon::ScrollbackDumpComplete` chunked scrollback
sequence within 5 seconds total. The retired single-message `ScrollbackDump` form MUST NOT be
accepted. If `ScrollbackDumpComplete` is not received within 5 seconds, the session is treated as
non-responsive: `attach_session()` returns `Err(SessionError::SessionHostDead { session_id })`
(maps to wire code `"attach_failed"` via `session_error_to_code(IpcOp::Attach, SessionHostDead)`).
SIGTERM is sent to the session-host PID (matching the 5s non-responsive handling in BC-2.08.004).

### AC-003 (traces to BC-2.08.007 attach postcondition 5 — host_conn updated with proxy_task Some)

After `ScrollbackDumpComplete` is received:
- `SessionEntry.host_conn` is set to `Some(SessionHostConnection { writer, proxy_task: Some(handle) })`.
- `proxy_task` is typed `Option<JoinHandle<()>>`; it is `Some(_)` after attach completes.
- The proxy task begins forwarding `HostToDaemon::PtyBytes` to the daemon broker as `Event::PtyOutput { session_id, bytes }`.

### AC-004 (traces to BC-2.08.007 attach postcondition 6-8 — SessionEntry.state → Running; SessionStateChanged{Running} before SessionListUpdate)

After the proxy task is started:
- `SessionEntry.state` transitions to `Running`.
- `ServerToClient::SessionStateChanged { session_id, new_state: Running }` is published to the broker BEFORE `ServerToClient::SessionListUpdate` (both under `SessionManager` mutex, per BC-2.08.008 Invariant 4).

### AC-005 (traces to BC-2.08.007 attach postcondition 9 — scrollback chunk stream forwarded to TUI clients)

The scrollback chunks (`ServerToClient::ScrollbackChunk` / `ServerToClient::ScrollbackDumpComplete`
messages) are forwarded to connected TUI clients so they can reconstruct the screen. TUI
receiver protocol (vt100 reconstruction via `scrollback-as-bytes` path) is defined in
SS-session-manager.md §Screen-state transfer on Attach; TUI implementation is in scope for
SS-09 stories.

### AC-006 (traces to BC-2.08.007 detach postcondition 1-5 — DaemonToHost::Detach; proxy task aborted; host_conn cleared; state → Detached; sidecar updated)

`detach_session()` on a Running session:
- Sends `DaemonToHost::Detach` over the control connection.
- Aborts the proxy task: `proxy_task.take().map(|t| t.abort())` (typed `Option<JoinHandle<()>>`; `.take()` clears the field; `.map(|t| t.abort())` aborts if present).
- Sets `SessionEntry.host_conn` to `None`.
- Transitions `SessionEntry.state` to `Detached`.
- Updates `session-state.json` to `state: "Detached"` atomically via `tempfile::persist`.

### AC-007 (traces to BC-2.08.007 detach postcondition 6-7 — SessionListUpdate published; session-host continues running)

After `detach_session()`:
- `ServerToClient::SessionListUpdate` is published to the broker.
- The session-host process continues running — the harness child keeps executing.
- NO `SessionStateChanged{Detached}` before the `SessionListUpdate`? Clarification: on detach, `Running → Detached` IS a genuine state-value transition. Per BC-2.08.008 Invariant 4, `SessionStateChanged{Detached}` is published BEFORE `SessionListUpdate` under the same mutex hold.

### AC-008 (traces to BC-2.08.007 invariant 1 — Detached session does NOT stream; persisted across restart)

After detach:
- The session-host is NOT streaming `PtyBytes` to the daemon (proxy task aborted).
- `state: "Detached"` is persisted in `session-state.json`.
- On daemon restart, `rediscover_sessions()` MUST restore this session in `Detached` state (NOT force-attach). TUI must send `ClientToServer::AttachSession` to explicitly resume streaming. (This invariant is tested by S-036; this story verifies the sidecar persistence.)

### AC-009 (traces to BC-2.08.007 invariant 2 — concurrent Attach serialized via mutex; no duplicate proxy_task)

Multiple concurrent `attach_session()` calls on the same session are serialized via
`Arc<Mutex<SessionManager>>`. The second `Attach` MUST NOT create a duplicate `proxy_task`.
Test: simulate concurrent attach calls; verify only one proxy task exists after both complete.

### AC-010 (traces to BC-2.08.007 invariant 3 — chunked scrollback only; retired ScrollbackDump not accepted)

The `ScrollbackChunk*` + `ScrollbackDumpComplete` protocol is mandatory. `ScrollbackDump`
(single-message, retired form) MUST NOT be accepted. If the session-host sends a retired
`ScrollbackDump`, the daemon logs WARN and the attach fails.

### AC-011 (traces to BC-2.08.007 edge case EC-185 — attach on Running session is idempotent Ok(()))

`attach_session()` on a `Running` session (already attached) returns `Ok(())` — idempotent;
no duplicate `proxy_task` created (Invariant 2 mutex serialization prevents duplicate
connection). `AlreadyAttached` does not exist in the canonical `SessionError` taxonomy.

### AC-012 (traces to BC-2.08.007 edge case EC-186 — detach on Detached is idempotent Ok(()))

`detach_session()` on a `Detached` session returns `Ok(())` — idempotent; no duplicate Detach
message sent.

### AC-013 (traces to BC-2.08.007 edge case EC-187 — session-host died between detach and re-attach)

`connect(socket_path)` fails at attach time. `kill(pid, None)` confirms dead.
`SessionEntry.state → Terminated`. `attach_session()` returns
`Err(SessionError::SessionHostDead { session_id })` → wire code `"attach_failed"`.

### AC-014 (traces to BC-2.08.007 detach precondition defensive note F-P51-001 — detach on Launching returns SessionNotReady)

`detach_session()` called on a `Launching` session with `host_conn: None` (possible from
untrusted clients) returns `Err(SessionError::SessionNotReady { session_id })` → wire code
`"session_not_ready"`. The official TUI never sends `DetachSession` during Launching
(BC-2.06.025 guards). This is a defensive invariant for untrusted clients only.

### AC-015 (traces to BC-2.08.008 postcondition 1 — no silent transitions on attach/detach paths)

`SessionStateChanged` is emitted for every attach/detach state transition:
- `Detached → Running` (attach_session() succeeds and ScrollbackDumpComplete received).
- `Running → Detached` (detach_session() called).
No silent transitions permitted.

## Tasks

- [ ] Implement `SessionManager::attach_session(&mut self, session_id: &str) -> Result<(), SessionError>`:
  - Look up session; check `SessionNotFound` if absent.
  - If Running: idempotent `Ok(())` (already attached; no new proxy_task per Invariant 2).
  - If Detached: UDS connect → SO_PEERCRED → send `DaemonToHost::Attach` → receive `ScrollbackChunk*` + `ScrollbackDumpComplete` within 5s timeout → start proxy task → set `host_conn: Some(SessionHostConnection{writer, proxy_task: Some(handle)})` → transition → Running → emit `SessionStateChanged{Running}` + `SessionListUpdate` under mutex.
  - If other state: return appropriate error or idempotent Ok (see action×state matrix in SS-session-manager.md §Terminated-in-grace).
- [ ] Implement `SessionManager::detach_session(&mut self, session_id: &str) -> Result<(), SessionError>`:
  - Look up session; check `SessionNotFound` if absent.
  - If Running: send `DaemonToHost::Detach` → abort proxy: `proxy_task.take().map(|t| t.abort())` → set `host_conn: None` → transition → Detached → update sidecar → emit `SessionStateChanged{Detached}` + `SessionListUpdate` under mutex.
  - If Launching with `host_conn: None`: return `Err(SessionError::SessionNotReady)` → wire `"session_not_ready"`.
  - If Detached: idempotent `Ok(())`.
  - If Terminating/Terminated: as per action×state matrix (SS-session-manager.md §Terminated-in-grace).
- [ ] Implement session-host `DaemonToHost::Attach` handler in `monocle-session-host/src/main.rs`:
  - Snapshot `vt100::Screen` as `Vec<Vec<SerializedCell>>`.
  - Resume live `PtyBytes` forwarding IMMEDIATELY after snapshot (I3-003 — no pause).
  - Stream `HostToDaemon::ScrollbackChunk` messages (each ≤ 256 KiB serialized).
  - Send `HostToDaemon::ScrollbackDumpComplete { total_chunks, cursor_row, cursor_col, pty_rows, pty_cols }`.
- [ ] Implement session-host `DaemonToHost::Detach` handler: disconnect daemon client; stay alive; stop sending `PtyBytes` to daemon.
- [ ] Add `ClientToServer::AttachSession` and `ClientToServer::DetachSession` arms to IPC handler.
- [ ] Write unit test `test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive`: mock session-host; attach → state Running; `ScrollbackChunk*` + `ScrollbackDumpComplete` forwarded; detach → state Detached; session-host alive.
- [ ] Write unit test `test_BC_2_08_007_attach_5s_timeout_session_host_dead`: session-host does not respond; 5s timeout; `Err(SessionHostDead)`; `"attach_failed"` wire code.
- [ ] Write unit test `test_BC_2_08_007_attach_running_idempotent`: attach on Running → `Ok(())`; no duplicate proxy_task.
- [ ] Write unit test `test_BC_2_08_007_detach_detached_idempotent`: detach on Detached → `Ok(())`.
- [ ] Write unit test `test_BC_2_08_007_detach_launching_session_not_ready`: detach on Launching with `host_conn: None` → `Err(SessionNotReady)`.
- [ ] Write unit test `test_BC_2_08_007_sidecar_updated_on_detach`: verify `session-state.json` updated to `state: "Detached"` atomically.
- [ ] Write unit test `test_BC_2_08_008_state_changed_ordering_on_attach_detach`: `SessionStateChanged{Running}` before `SessionListUpdate` on attach; `SessionStateChanged{Detached}` before `SessionListUpdate` on detach.
- [ ] Write integration test `test_BC_2_08_007_attach_detach_cycle`: spawn → wait Running → detach → wait Detached → re-attach → wait Running again; session-host alive throughout.

## Previous Story Intelligence

- **S-033** (session-manager-spawn): `SessionManager`, `SessionEntry`, `SessionHostConnection { writer, proxy_task: Option<JoinHandle<()>> }`, `SessionState`, post-spawn monitor, broker publication pattern, and the `Launching → Running` transition are all established. The `proxy_task: Option<JoinHandle<()>>` type is set in post-spawn monitor; attach/detach must use the same `Option<JoinHandle<()>>` manipulation.
- The `host_conn` field is set to `Some(_)` during post-spawn monitor in S-033; `attach_session()` also sets it.
- The scrollback chunk forwarding to TUI clients (broker fan-out of `ScrollbackChunk` messages) must align with SS-ipc.md `ServerToClient::ScrollbackChunk` / `ServerToClient::ScrollbackDumpComplete` wire types (from S-021).

## Architecture Compliance Rules

- `proxy_task.take().map(|t| t.abort())` is the canonical abort pattern — `proxy_task` is `Option<JoinHandle<()>>`; `.take()` sets it to `None` atomically; `.map(|t| t.abort())` aborts if present. Do NOT use `proxy_task.as_ref().unwrap().abort()` (panics on None) or any form that leaves the field set after abort.
- `ScrollbackDump` (single-message retired form) MUST NOT be accepted in the `HostToDaemon` deserialization path for `Attach` responses. If observed, log WARN and treat as attach failure.
- SO_PEERCRED is mandatory on EVERY fresh UDS connect for `attach_session()`. No exceptions.
- Sidecar update on detach (→ `state: "Detached"`) MUST use `tempfile::persist`. Naked `std::fs::write` is forbidden.
- `SessionStateChanged{Detached}` MUST be published BEFORE `SessionListUpdate` for the detach transition. Both under the same mutex hold.
- Forbidden dependency: `monocle-runtime` MUST NOT depend on `monocle-tui`.
- `vt100 "0.16"` (caret) is used in `monocle-session-host` for screen snapshot. Do NOT use `vt100 0.15` or any earlier version.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tokio` | `=1.52` (exact) | Async runtime; proxy task `tokio::spawn`; 5s timeout via `tokio::time::timeout` | SS-deps-pin-manifest.md |
| `vt100` | `"0.16"` (caret) | `vt100::Screen` snapshot in `monocle-session-host`; `SerializedCell` encoding | SS-deps-pin-manifest-v2-delta.md |
| `nix` | `"0.30"` | SO_PEERCRED on attach UDS connect; SIGTERM to non-responsive session-host | SS-deps-pin-manifest.md |
| `serde_json` | `=1.0.149` (exact) | Sidecar update on detach | SS-deps-pin-manifest.md |
| `tempfile` | `"3"` | Atomic sidecar writes | SS-deps-pin-manifest.md |
| `thiserror` | `"2"` | `SessionError` variants (from S-033) | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to MODIFY (all established by S-033):

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/session_manager/mod.rs` | Add `attach_session()` and `detach_session()` implementations |
| `crates/monocle-runtime/src/ipc_handler.rs` | Add `ClientToServer::AttachSession` and `ClientToServer::DetachSession` arms |
| `crates/monocle-session-host/src/main.rs` | Add `DaemonToHost::Attach` handler (snapshot vt100 screen; stream ScrollbackChunk*; resume PtyBytes immediately); add `DaemonToHost::Detach` handler (stop streaming; stay alive) |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~3,500 |
| BC-2.08.007 | ~4,000 |
| BC-2.08.008 (attach/detach transition sections) | ~1,500 |
| SS-session-manager.md (attach, detach, screen-state transfer, scrollback memory bound) | ~10,000 |
| Existing session_manager code from S-033 | ~6,000 |
| session-host main.rs (Attach handler + scrollback chunk streaming) | ~3,000 |
| Test files | ~4,000 |
| **Total estimate** | **~32,000** |

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.08.007 | Attach/Detach — Chunked Scrollback (ScrollbackChunk*+ScrollbackDumpComplete) on Attach; session-host Stays Alive on Detach | v1.5.1 |
| BC-2.08.008 | SessionStateChanged — Daemon Emits on Every SessionState Transition; Delivered to All TUI Clients; Ordering Relative to SessionListUpdate | v1.3.0 |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `attach_session()` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (UDS connect; SO_PEERCRED; message I/O; state mutation; broker publish) |
| `detach_session()` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (IPC send; task abort; sidecar write; broker publish) |
| PTY proxy task | `monocle-runtime/src/session_manager/mod.rs` | Effectful (broker event publish loop) |
| `DaemonToHost::Attach` handler | `crates/monocle-session-host/src/main.rs` | Effectful (vt100 screen snapshot; ScrollbackChunk streaming; PtyBytes resume) |
| `DaemonToHost::Detach` handler | `crates/monocle-session-host/src/main.rs` | Effectful (stop streaming; continue alive) |
| IPC handler `AttachSession`/`DetachSession` arms | `monocle-runtime/src/ipc_handler.rs` | Effectful (IPC dispatch; error propagation) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-185 | `attach_session()` on a `Running` session (already attached) | `Ok(())` — idempotent; no duplicate `proxy_task` |
| EC-186 | `detach_session()` on a `Detached` session | `Ok(())` — idempotent; no duplicate Detach sent |
| EC-187 | Session-host process died between detach and re-attach | UDS connect fails; liveness probe confirms dead; `SessionEntry.state → Terminated`; `Err(SessionHostDead)` → `"attach_failed"` |
| EC-188 | `ScrollbackDumpComplete` not received within 5s | Session non-responsive; `Err(SessionHostDead)` → `"attach_failed"`; SIGTERM to session-host PID |

## Subsystem Anchor Justifications

**SS-08 owns this story's scope** because `attach_session()` and `detach_session()` are core `SessionManager` lifecycle operations defined in SS-session-manager.md §Public API.

**Dependency Anchor:**
- STORY-035 depends on S-033 because `SessionManager`, `SessionEntry`, `SessionHostConnection`, `SessionState`, `SessionHostSpawner`, and the IPC handler skeleton must all exist before attach/detach can be added.
