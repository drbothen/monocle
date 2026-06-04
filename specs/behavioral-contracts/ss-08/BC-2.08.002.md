---
document_type: behavioral-contract
level: L3
version: "1.2.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/adr/ADR-0009-native-session-host-process-model.md]
input-hash: "349527f"
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

# Behavioral Contract BC-2.08.002: Session Persistence — session-host Survives Graceful Daemon Restart

## Description

Sessions spawned by `SessionManager` survive a graceful daemon restart. The `monocle-session-host`
process is spawned with `setsid()` so it becomes a process group leader, making it immune to
`SIGHUP` when the daemon exits. After a graceful daemon restart, `rediscover_sessions()`
re-attaches to the alive session-hosts and the session list is re-populated before any TUI
client can connect.

## Preconditions

1. At least one session is in `SessionState::Running` with a live `monocle-session-host` process.
2. The daemon undergoes a graceful shutdown (SIGTERM → drain → exit; per BC-2.01.004).
3. The `session-state.json` sidecar for the session exists in `runtime_dir`.

## Postconditions

1. When the daemon process exits (gracefully), the `monocle-session-host` process continues
   running. It is NOT sent SIGTERM or SIGHUP by the daemon's shutdown sequence. The harness
   child (Claude Code) continues running inside the session-host's PTY.
2. The `session-state.json` sidecar remains intact in `runtime_dir` after daemon exit.
3. When the daemon restarts, `rediscover_sessions()` is called during `daemon_start_sequence`
   step 8b (before lock file write and before UDS bind).
4. For the previously-running session: `nix::sys::signal::kill(pid, None)` returns `Ok(())`
   (process alive). The daemon connects to the session-host's UDS socket, sends
   `DaemonToHost::Attach`, and receives the full `HostToDaemon::ScrollbackChunk*` +
   `HostToDaemon::ScrollbackDumpComplete` chunked scrollback sequence within 5 seconds total
   (per BC-2.08.004 PC-2b; the retired single-message `ScrollbackDump` form is NOT accepted).
5. After re-discovery, the session appears in `DaemonState.session_manager` with
   `SessionState::Running` (re-verified via scrollback dump receipt).
6. The first TUI client to connect after daemon restart receives an `InitialState` push that
   includes the re-discovered session in its sessions list. The session is visible to the
   user as a pre-existing running session.
7. The TUI can immediately enter `AppMode::EmbeddedTerminal` for the re-discovered session —
   the scrollback buffer is available (reconstructed from the `ScrollbackChunk*` +
   `ScrollbackDumpComplete` sequence received during re-discovery) and live PTY output resumes.

## Invariants

1. `setsid()` is called by `monocle-session-host` during its startup sequence (step 2 in
   SS-session-manager.md §startup sequence). This is non-negotiable — without `setsid()`,
   the session-host is in the daemon's process group and receives SIGHUP when the daemon
   exits.
2. The daemon's graceful shutdown sequence (BC-2.01.004) MUST NOT send SIGTERM to
   session-host processes. Session-hosts are not daemon children in the OS sense (they are
   detached); the daemon's process group exit does not affect them.
3. Session survival covers graceful daemon restart ONLY. A hard crash (SIGKILL to daemon)
   also results in session survival (session-host is detached) but re-discovery may need to
   handle a missing lock file. See EC-160 and BC-2.08.004 for crash recovery.
4. The 5-second attach timeout during re-discovery (Precondition 4) is a hard deadline.
   If the full `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence is not received within
   5s, the session-host is treated as non-responsive and the session is marked `Terminated`
   with the sidecar deleted. The retired single-message `ScrollbackDump` form is NOT accepted —
   only the chunked protocol terminating with `ScrollbackDumpComplete` (matching BC-2.08.004 PC-2b).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-155 | Session-host process is alive but its UDS socket file was deleted (e.g., tmpfs cleared) | `kill(pid, None)` succeeds (process alive) but `connect(socket_path)` fails; session marked `Terminated`; sidecar deleted; TUI session list omits the session |
| EC-156 | Session-host process is alive but does not respond within 5s (stuck) | Session marked `Terminated`; sidecar deleted; `SIGTERM` sent to session-host PID to release PTY resources; TUI shows session as terminated |
| EC-157 | Daemon exits with SIGKILL (crash); session-host survives | Session-host process survives (setsid); sidecar intact; next daemon startup re-discovers session per BC-2.08.004 |
| EC-158 | Session was in `SessionState::Launching` when daemon exited (spawn in progress) | On re-discovery: `kill(pid, None)` probe — if alive, attempt `DaemonToHost::Attach` with 5s timeout waiting for `HostToDaemon::ScrollbackDumpComplete`; if `ScrollbackDumpComplete` received within 5s, register as Running; if no response within 5s, send SIGTERM to session-host PID, mark Terminated, GC sidecar. No exponential backoff — the 5s is a hard single-attempt timeout (canonical per SS-session-manager.md v1.8.1 §Daemon startup: session re-discovery). |
| EC-159 | Multiple sessions exist; one alive, one dead | Re-discovery marks alive session Running; dead session Terminated with GC; TUI receives SessionListUpdate with only the alive session |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Start daemon; spawn session; daemon graceful shutdown; daemon restart; TUI connects | Session appears in InitialState.sessions with state Running; scrollback buffer available | happy-path |
| Session-host UDS socket deleted before daemon restart | Session absent from re-discovered list (Terminated/GC) | edge-case |
| Session-host dead before daemon restart | Session absent from re-discovered list (Terminated/GC) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `test_session_manager_rediscover_on_daemon_restart` — live session-host survives daemon restart and appears in re-discovered list | integration |
| VP-TBD | setsid() called by session-host before PTY open | unit (session-host startup sequence) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — this BC defines the persistence property: sessions survive daemon restart, which is the primary differentiator of the detached session-host model (ADR-0009) |
| L2 Domain Invariants | DI-001 (hook event durability — session survival ensures hook events from in-progress sessions continue to flow after daemon restart, because the session-host continues running; this supports DI-001 continuity) |
| Architecture Module | monocle-runtime (SessionManager, `rediscover_sessions()`); monocle-session-host (setsid startup step) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v1.8.1 §Daemon startup: session re-discovery; SS-session-manager.md §monocle-session-host binary §startup sequence step 2; ADR-0009 §native-detached-session-host |
| Test Name | test_BC_2_08_002_session_survives_daemon_graceful_restart |

## Related BCs

- [BC-2.08.001] — depends on: session-state.json written at spawn is the input to re-discovery
- [BC-2.08.004] — composes with: re-discovery procedure used here is specified in detail in BC-2.08.004
- [BC-2.08.005] — composes with: GC policy applies to dead sessions discovered during re-discovery

## Architecture Anchors

- `architecture/SS-session-manager.md#daemon-startup-session-re-discovery` — re-discovery algorithm
- `architecture/SS-session-manager.md#monocle-session-host-binary` — startup sequence (setsid step 2)
- `architecture/adr/ADR-0009-native-session-host-process-model.md` — persistence rationale

## Story Anchor

S-TBD — Implement session re-discovery and setsid in monocle-session-host (filled by story-writer)

## VP Anchors

VP-TBD — Daemon restart integration test (filled after VP creation)

## §Trace v1.2.0

**HIGH-001 adversarial pass-4 fix — PC-4/PC-7/Invariant 4: retired single-message ScrollbackDump → chunked protocol** (2026-06-03):
- PC-4: "receives `HostToDaemon::ScrollbackDump` within 5 seconds" → "receives the full
  `HostToDaemon::ScrollbackChunk*` + `HostToDaemon::ScrollbackDumpComplete` chunked scrollback
  sequence within 5 seconds total (per BC-2.08.004 PC-2b; retired single-message form NOT accepted)".
- PC-7: "scrollback buffer is available (from `ScrollbackDump`)" → "scrollback buffer is available
  (reconstructed from the `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence)".
- Invariant 4: "If `ScrollbackDump` is not received within 5s" → "If the full `ScrollbackChunk*`
  + `ScrollbackDumpComplete` sequence is not received within 5s" with explicit retirement note
  matching BC-2.08.004 PC-2b. Closes internal inconsistency with EC-158 (which already correctly
  referenced `ScrollbackDumpComplete`).

## §Trace v1.1.0

**I2-005 adversarial pass-2 fix — EC-158 synced to canonical re-discovery procedure** (2026-06-03):
- I2-005 finding: EC-158 described "retry with exponential backoff" for Launching-state sessions
  on re-discovery. This contradicts the canonical procedure in SS-session-manager.md v1.5.0
  §Daemon startup: session re-discovery (and BC-2.08.004 PC-2), which specifies a single
  `DaemonToHost::Attach` attempt with a 5-second hard timeout — no exponential backoff.
- EC-158: rewritten to match SS-session-manager.md v1.5.0 canonically: one Attach attempt,
  5s timeout for `ScrollbackDumpComplete`, then Terminated+GC. No invented retry logic.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.002 authored for SS-08 as part of the v1A control-center pivot BC burst.
- Covers: session-host setsid(), daemon graceful restart, re-discovery, scrollback on re-attach.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
