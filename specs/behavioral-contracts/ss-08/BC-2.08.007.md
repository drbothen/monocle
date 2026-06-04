---
document_type: behavioral-contract
level: L3
version: "1.4.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md]
input-hash: "81ce91a"
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

# Behavioral Contract BC-2.08.007: Attach/Detach — Chunked Scrollback (ScrollbackChunk*+ScrollbackDumpComplete) on Attach; session-host Stays Alive on Detach

## Description

`SessionManager::attach_session()` re-connects the daemon to a `Detached` session-host,
receives the full `HostToDaemon::ScrollbackChunk*` + `HostToDaemon::ScrollbackDumpComplete`
chunked scrollback sequence (current vt100 screen state as styled cells), and begins proxying
live PTY bytes. The retired single-message `ScrollbackDump` form MUST NOT be used.
`SessionManager::detach_session()` sends `DaemonToHost::Detach` to the session-host and
removes the proxy task without terminating the session. The session continues running in the
background. The TUI can re-attach at any time.

## Preconditions (attach)

1. `SessionEntry` exists with `state: Detached` and `host_conn: None`.
2. The session-host process is alive.

## Preconditions (detach)

1. `SessionEntry` exists with `state: Running` and `host_conn: Some(_)`.
2. The session-host process is alive.

## Postconditions (attach)

1. `SessionManager` connects to `<runtime_dir>/session-<session_id>.sock`.
2. Verifies SO_PEERCRED peer uid matches daemon uid before sending any messages (per
   SS-session-manager.md §Per-session UDS security; failure → abort attach).
3. Sends `DaemonToHost::Attach` over the connection.
4. Receives the full `HostToDaemon::ScrollbackChunk*` + `HostToDaemon::ScrollbackDumpComplete`
   chunked scrollback sequence within 5 seconds total. (C5/C2-002 fix: chunked protocol
   replaces the retired single-message `ScrollbackDump` form. Styled-cell serialization
   `Vec<Vec<SerializedCell>>` preserves full visual fidelity.)
5. Stores `host_conn: Some(SessionHostConnection { writer, proxy_task })` on the `SessionEntry`.
6. `SessionEntry.state` transitions to `Running`.
7. The proxy task begins forwarding `HostToDaemon::PtyBytes` to the daemon broker as
   `Event::PtyOutput { session_id, bytes }`.
8. `ServerToClient::SessionStateChanged { session_id, new_state: Running }` is published to
   the broker BEFORE `ServerToClient::SessionListUpdate` (per BC-2.08.008 Invariant 4,
   both under the `SessionManager` mutex).
9. The scrollback chunk stream is forwarded to connected TUI clients as
    `ServerToClient::ScrollbackChunk` / `ServerToClient::ScrollbackDumpComplete` messages
    (per BC-2.05.011 ScrollbackChunk/ScrollbackDumpComplete postconditions). The TUI
    accumulates chunks, validates count on `ScrollbackDumpComplete`, resets its `vt100::Parser`,
    and reconstructs the screen. No raw PTY bytes are synthesized for the scrollback — styled
    cells are applied directly.

## Postconditions (detach)

1. `SessionManager` sends `DaemonToHost::Detach` over the connection.
2. The proxy task for this session is terminated (`proxy_task.abort()`).
3. `SessionEntry.host_conn` is set to `None`.
4. `SessionEntry.state` transitions to `Detached`.
5. `session-state.json` is updated to `state: "Detached"` (atomically).
6. A `ServerToClient::SessionListUpdate` IPC message is published.
7. The session-host process continues running — the harness child keeps executing.

## Invariants

1. Detach does NOT kill the session. The session-host continues operation; it stops sending
   `PtyBytes` to the daemon until the next `Attach`. A Detached session's state is persisted
   to `session-state.json` as `"Detached"`. On daemon restart, `rediscover_sessions()` MUST
   restore the session in `Detached` state (NOT force-attach it; per BC-2.08.004 I3-005 and
   Invariant 6). The TUI may send `ClientToServer::AttachSession` to explicitly resume
   streaming. This invariant means: Detached sessions survive daemon restarts in Detached state.
2. Multiple concurrent `Attach` operations on the same session MUST be serialized via the
   `Arc<Mutex<SessionManager>>`. The second `Attach` must not create a duplicate `proxy_task`.
3. The scrollback is transferred as `HostToDaemon::ScrollbackChunk*` + `HostToDaemon::
   ScrollbackDumpComplete` — styled cells `Vec<Vec<SerializedCell>>` (full fg/bg color +
   attrs). The retired single-message `ScrollbackDump` form MUST NOT be used. The TUI
   MUST reset its parser for the session BEFORE applying the dump to prevent double-counting
   live parser state. See SS-session-manager.md v1.7.1 §Screen-state transfer for the
   full reconstruction protocol.
4. The 5-second timeout applies to the full `ScrollbackChunk*` + `ScrollbackDumpComplete`
   sequence for both re-discovery (BC-2.08.004) and interactive attach. After 5s without
   `ScrollbackDumpComplete`, the session is treated as non-responsive.
5. SO_PEERCRED peer-credential check is mandatory on attach before sending any messages.
   A peer uid mismatch aborts the attach (session treated as dead).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-185 | `attach_session()` on a `Running` session (already attached) | Returns `Err(SessionError::AlreadyAttached { session_id })`; no duplicate connection created |
| EC-186 | `detach_session()` on a `Detached` session | Returns `Ok(())` — idempotent; no duplicate Detach sent |
| EC-187 | Session-host process died between detach and re-attach | `connect(socket_path)` fails; `kill(pid, None)` confirms dead; `SessionEntry.state` → `Terminated`; `attach_session()` returns `Err(SessionError::SessionHostDead)` |
| EC-188 | `ScrollbackDumpComplete` (full `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence) not received within 5s | Session treated as non-responsive; `Err(SessionError::AttachTimeout)`; session-host sent SIGTERM. The retired single-message `ScrollbackDump` form is NOT accepted — only the chunked protocol terminating with `ScrollbackDumpComplete`. |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| `attach_session("detached-id")` with mock session-host | `Ok(())`; `host_conn` set; state → Running; ScrollbackChunk* + ScrollbackDumpComplete received and forwarded to TUI clients | happy-path |
| `detach_session("running-id")` with mock session-host | `Ok(())`; `host_conn` cleared; state → Detached; proxy task aborted | happy-path |
| attach → detach → attach cycle | Session-host alive throughout; second attach restores screen state | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `ScrollbackChunk*` + `ScrollbackDumpComplete` received and forwarded to broker on attach | integration |
| VP-TBD | Session-host alive after detach (process not killed) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — detach/attach are explicitly named session lifecycle operations in CAP-008 |
| Architecture Module | monocle-runtime (SessionManager `attach_session()`, `detach_session()`) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v1.7.1 §Public API (attach_session, detach_session signatures); §Per-session UDS protocol (DaemonToHost::Attach/Detach, HostToDaemon::ScrollbackChunk/ScrollbackDumpComplete); §Screen-state transfer on Attach; §Re-discovery state handling (I3-005 Detached preservation across restart); SS-daemon-wiring-v2-delta.md v1.5.0 §3b (SessionStateChanged{Running} before SessionListUpdate on attach) |
| Test Name | test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive |

## Related BCs

- [BC-2.08.002] — composes with: re-discovery uses the same Attach → `ScrollbackChunk*` + `ScrollbackDumpComplete` chunked protocol
- [BC-2.09.001] — depends on: PTY output after attach flows through the broker to the TUI renderer

## Architecture Anchors

- `architecture/SS-session-manager.md#per-session-uds-protocol` — Attach/Detach/ScrollbackChunk/ScrollbackDumpComplete messages
- `architecture/SS-session-manager.md#screen-state-transfer-on-attach` — chunked reconstruction protocol

## Story Anchor

S-TBD — Implement SessionManager attach/detach (filled by story-writer)

## VP Anchors

VP-TBD — Attach/detach integration tests (filled after VP creation)

## §Trace v1.4.0

**HIGH-002 adversarial pass-4 fix — H1 title + Description + Related-BCs: retired ScrollbackDump → chunked protocol** (2026-06-03):
- H1 title: "ScrollbackDump on Attach" → "Chunked Scrollback (ScrollbackChunk*+ScrollbackDumpComplete) on Attach" — aligns with the body (Invariant 3 already forbade the retired form since v1.2.0). Title propagated to prd.md §2.8 BC table and BC-INDEX.md §SS-08 table in this same burst.
- Description: "receives a `ScrollbackDump` (current vt100 screen state)" → "receives the full `HostToDaemon::ScrollbackChunk*` + `HostToDaemon::ScrollbackDumpComplete` chunked scrollback sequence". Explicit retirement note added.
- Related-BCs [BC-2.08.002] entry: "Attach → ScrollbackDump protocol" → "Attach → `ScrollbackChunk*` + `ScrollbackDumpComplete` chunked protocol".

## §Trace v1.3.0

**Adversarial Pass 3 fixes — O3-001 (dup PC) + I3-005 (Detached survive restart) + EC-188 + C3-001 (SessionStateChanged on attach)** (2026-06-03):
- O3-001: PC-3 and PC-4 were identical ("Sends DaemonToHost::Attach"). Deduped — removed
  duplicate PC; renumbered: old PC-5 → PC-4, old PC-6 → PC-5, old PC-7 → PC-6, old PC-8 → PC-7,
  old PC-9 → PC-8, old PC-10 → PC-9.
- C3-001: New PC-8 added: `SessionStateChanged{Running}` emitted BEFORE `SessionListUpdate`
  when `Detached → Running` transition occurs on attach (per BC-2.08.008 Invariant 4).
- I3-005 (Detached survive restart): Invariant 1 updated — Detached sessions survive daemon
  restart in Detached state; re-discovery MUST NOT force-attach Detached sessions (BC-2.08.004
  I3-005). TUI must send `ClientToServer::AttachSession` to explicitly resume.
- EC-188: corrected from "`ScrollbackDump` not received" to "`ScrollbackDumpComplete` (full
  `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence) not received". The retired single-
  message `ScrollbackDump` form is not referenced.
- Architecture Source updated to SS-session-manager.md v1.5.0 and SS-daemon-wiring-v2-delta.md v1.3.1.

## §Trace v1.2.0

**Architect-delegated BC edit — chunked scrollback protocol (C2-002) + cwd/project_root (I2-002)** (2026-06-03):
- C2-002: updated PC-4/5/10 and Invariant 3/4 to reference `ScrollbackChunk*` +
  `ScrollbackDumpComplete` (chunked protocol) rather than the retired single-message
  `ScrollbackDump`. Added BC-2.05.011 as the authoritative spec for TUI receiver protocol.
  Architecture Source updated to SS-session-manager.md v1.5.0.
- Added `architecture/SS-session-manager.md#screen-state-transfer-on-attach` anchor.
- VP table: updated from `ScrollbackDump` to `ScrollbackChunk*/ScrollbackDumpComplete`.

## §Trace v1.1.0

**C5 ScrollbackDump typed-cell serialization fix + I5 SO_PEERCRED** (2026-06-03):
- PC-3: Updated `ScrollbackDump` type from `rows: Vec<String>` to
  `rows: Vec<Vec<SerializedCell>>` with styled-cell encoding per SS-session-manager.md C5.
  Large scrollbacks may arrive in streaming chunks (`ScrollbackDumpComplete` sentinel).
- PC-9 (new): Added `pty_rows` and `pty_cols` fields to `ScrollbackDump` postcondition.
- PC-4 (former PC-3): SO_PEERCRED peer-credential check specified as mandatory on attach
  before any message exchange (per I5 security fix in SS-session-manager.md).
- Invariant 3: Replaced `Vec<String>` description with `Vec<Vec<SerializedCell>>`; added
  parser-reset precondition for TUI receiver; references SS-session-manager.md §Screen-state-transfer.
- Invariant 5 (new): SO_PEERCRED check invariant formalized.
- Architecture Source updated to SS-session-manager.md v1.2.0.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.007 authored for SS-08 as part of the v1A control-center pivot BC burst.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
