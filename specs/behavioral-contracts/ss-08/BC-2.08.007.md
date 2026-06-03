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

# Behavioral Contract BC-2.08.007: Attach/Detach — ScrollbackDump on Attach; session-host Stays Alive on Detach

## Description

`SessionManager::attach_session()` re-connects the daemon to a `Detached` session-host,
receives a `ScrollbackDump` (current vt100 screen state), and begins proxying live PTY bytes.
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
4. Receives `HostToDaemon::ScrollbackDump { rows: Vec<Vec<SerializedCell>>, cursor_row,
   cursor_col, pty_rows, pty_cols }` within 5 seconds. (C5 fix: rows are styled-cell
   serialization, NOT `Vec<String>`. Large scrollbacks may arrive as a stream of
   `ScrollbackDump` messages terminated by `ScrollbackDumpComplete`.)
5. Stores `host_conn: Some(SessionHostConnection { writer, proxy_task })` on the `SessionEntry`.
6. `SessionEntry.state` transitions to `Running`.
7. The proxy task begins forwarding `HostToDaemon::PtyBytes` to the daemon broker as
   `Event::PtyOutput { session_id, bytes }`.
8. A `ServerToClient::SessionListUpdate` IPC message is published.
9. The `ScrollbackDump` styled-cell data is forwarded to connected TUI clients as
   `ServerToClient::ScrollbackDump { session_id, rows, cursor_row, cursor_col, pty_rows, pty_cols }`.
   The TUI MUST reset its `vt100::Parser` for the session before applying the dump (see
   SS-session-manager.md §Screen-state transfer). No raw PTY bytes are synthesized for
   the scrollback — styled cells are applied directly to reconstruct the screen state.

## Postconditions (detach)

1. `SessionManager` sends `DaemonToHost::Detach` over the connection.
2. The proxy task for this session is terminated (`proxy_task.abort()`).
3. `SessionEntry.host_conn` is set to `None`.
4. `SessionEntry.state` transitions to `Detached`.
5. `session-state.json` is updated to `state: "Detached"` (atomically).
6. A `ServerToClient::SessionListUpdate` IPC message is published.
7. The session-host process continues running — the harness child keeps executing.

## Invariants

1. Detach does NOT kill the session. The session-host is notified (Detach message) and
   continues operation; it stops sending `PtyBytes` to the daemon until the next `Attach`.
2. Multiple concurrent `Attach` operations on the same session MUST be serialized via the
   `Arc<Mutex<SessionManager>>`. The second `Attach` must not create a duplicate `proxy_task`.
3. `ScrollbackDump` contains ALL vt100 screen state as `Vec<Vec<SerializedCell>>` — full
   styled cells including fg/bg color and attribute flags. `Vec<String>` (C5 bug: no color,
   no attrs, double-applies on a live parser) is NOT the format. The TUI MUST reset its
   parser for the session BEFORE applying the dump to prevent double-counting live parser
   state. See SS-session-manager.md §Screen-state transfer for the full protocol.
4. The 5-second `ScrollbackDump` timeout applies to both re-discovery (BC-2.08.004) and
   interactive attach. After 5s, the session is treated as non-responsive.
5. SO_PEERCRED peer-credential check is mandatory on attach before sending any messages.
   A peer uid mismatch aborts the attach (session treated as dead).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-185 | `attach_session()` on a `Running` session (already attached) | Returns `Err(SessionError::AlreadyAttached { session_id })`; no duplicate connection created |
| EC-186 | `detach_session()` on a `Detached` session | Returns `Ok(())` — idempotent; no duplicate Detach sent |
| EC-187 | Session-host process died between detach and re-attach | `connect(socket_path)` fails; `kill(pid, None)` confirms dead; `SessionEntry.state` → `Terminated`; `attach_session()` returns `Err(SessionError::SessionHostDead)` |
| EC-188 | `ScrollbackDump` not received within 5s | Session treated as non-responsive; `Err(SessionError::AttachTimeout)`; session-host sent SIGTERM |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| `attach_session("detached-id")` with mock session-host | `Ok(())`; `host_conn` set; state → Running; ScrollbackDump rows forwarded | happy-path |
| `detach_session("running-id")` with mock session-host | `Ok(())`; `host_conn` cleared; state → Detached; proxy task aborted | happy-path |
| attach → detach → attach cycle | Session-host alive throughout; second attach restores screen state | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `ScrollbackDump` received and rows forwarded to broker on attach | integration |
| VP-TBD | Session-host alive after detach (process not killed) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — detach/attach are explicitly named session lifecycle operations in CAP-008 |
| Architecture Module | monocle-runtime (SessionManager `attach_session()`, `detach_session()`) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v1.2.0 §Public API (attach_session, detach_session signatures); §Per-session UDS protocol (DaemonToHost::Attach/Detach, HostToDaemon::ScrollbackDump) |
| Test Name | test_BC_2_08_007_attach_receives_scrollback_detach_keeps_session_alive |

## Related BCs

- [BC-2.08.002] — composes with: re-discovery uses the same Attach → ScrollbackDump protocol
- [BC-2.09.001] — depends on: PTY output after attach flows through the broker to the TUI renderer

## Architecture Anchors

- `architecture/SS-session-manager.md#per-session-uds-protocol` — Attach/Detach/ScrollbackDump messages

## Story Anchor

S-TBD — Implement SessionManager attach/detach (filled by story-writer)

## VP Anchors

VP-TBD — Attach/detach integration tests (filled after VP creation)

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
