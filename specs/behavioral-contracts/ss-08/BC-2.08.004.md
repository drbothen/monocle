---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/SS-daemon-wiring-v2-delta.md]
input-hash: "ded317c"
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

# Behavioral Contract BC-2.08.004: Re-Discovery — All Alive Sessions Visible After Daemon Restart Within 5s; UDS Bind Blocked Until Complete

## Description

On daemon startup, `SessionManager::rediscover_sessions()` runs as step 8b of
`daemon_start_sequence` — after the lock file is written but BEFORE the UDS socket is bound.
This ordering guarantee means no TUI client can connect and receive a stale (incomplete)
session list. All alive session-hosts are re-attached, their `SessionEntry` records are
restored, and dead sessions are GC'd. The total re-discovery time MUST complete within 5
seconds for the typical case of up to 8 sessions.

## Preconditions

1. `daemon_start_sequence` is executing.
2. Lock file has been written (step 8 complete).
3. UDS socket has NOT yet been bound (step 10 has not started).
4. `runtime_dir` is accessible.
5. Zero or more `session-*.json` sidecar files exist in `runtime_dir`.

## Postconditions

1. `rediscover_sessions()` reads ALL `session-*.json` files in `runtime_dir`. Files with
   unknown `schema_version` (not `1`) are skipped with a WARN log and treated as orphaned
   (deleted).
2. For each sidecar with `schema_version: 1`:
   a. `nix::sys::signal::kill(Pid::from_raw(pid), None)` probes process liveness.
   b. If alive: `connect(socket_path)` → `DaemonToHost::Attach` → wait up to 5s for
      `HostToDaemon::ScrollbackDump` → on receipt, add `SessionEntry` to registry with
      `state: Running` and populate the `host_conn` with the live connection.
   c. If dead: delete sidecar file; no `SessionEntry` added.
3. All `SessionEntry` records from step 2b are in `DaemonState.session_manager` before any
   TUI client can connect (UDS bind has not happened yet).
4. `rediscover_sessions()` returns `Ok(RediscoveryReport)` reporting: `found_alive: usize`,
   `found_dead: usize`, `errors: Vec<RediscoveryError>`.
5. On corrupt sidecar (JSON parse failure): log WARN, delete sidecar, continue; do NOT abort
   startup. A single corrupt sidecar MUST NOT prevent re-discovery of other sessions.
6. UDS socket bind (daemon_start_sequence step 10) proceeds only after `rediscover_sessions()`
   returns. If re-discovery fails entirely (e.g., `runtime_dir` unreadable), log ERROR and
   continue startup with an empty session registry — daemon starts clean rather than not at all.
7. The entire re-discovery completes within 5 seconds for up to 8 concurrent sessions
   (parallel attach probing with 5s per-session timeout; total wall-clock ≤ 5s by parallelizing
   the attach phase across all sessions simultaneously with `tokio::join_all`).

## Invariants

1. **Ordering is mandatory:** Re-discovery MUST complete before UDS bind. This invariant
   prevents the race condition where a TUI client connects and receives an `InitialState`
   push that is missing re-discovered sessions. This is enforced by placement in
   `daemon_start_sequence` (step 8b precedes step 10).
2. The 5-second timeout per session-host during attach probing is a hard deadline. After 5s
   without `ScrollbackDump`, the session-host is treated as non-responsive (see BC-2.08.002
   EC-156 for the termination path).
3. Parallel attach: all session-host attach probes run concurrently (not sequentially).
   With up to 8 sessions and 5s per timeout, sequential probing would take up to 40s — which
   is unacceptable for daemon startup. `tokio::join_all` (or equivalent) ensures all probes
   run simultaneously.
4. A dead session-host (process not found) is GC'd immediately during re-discovery by
   deleting its sidecar. It does NOT appear in the registry or in the initial session list.
5. Orphaned socket files (socket file exists but process is dead) are deleted during re-discovery
   alongside the sidecar.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-167 | `runtime_dir` is empty (no sidecars) | `rediscover_sessions()` returns `Ok(RediscoveryReport { found_alive: 0, found_dead: 0, errors: [] })`; startup continues normally |
| EC-168 | One sidecar is corrupt JSON | WARN log; corrupt sidecar deleted; other sidecars processed normally; startup continues |
| EC-169 | Session-host process alive but socket file missing | Process alive (`kill(pid, None)` OK) but `connect(socket_path)` fails; session treated as non-responsive; sidecar deleted; `SIGTERM` sent to orphaned process |
| EC-170 | `runtime_dir` is unreadable | Log ERROR; `rediscover_sessions()` returns `Ok(RediscoveryReport { found_alive: 0, found_dead: 0, errors: [RuntimeDirUnreadable] })`; startup proceeds with empty registry |
| EC-171 | 8 sessions: 4 alive, 4 dead | All 8 probed in parallel; 4 dead sidecars deleted; 4 alive `SessionEntry` records added; total time ≤ 5s |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| 2 sidecar files; both session-hosts alive (mock) | `RediscoveryReport { found_alive: 2, found_dead: 0 }`; both entries in registry with `state: Running` | happy-path |
| 2 sidecar files; 1 alive, 1 dead | `found_alive: 1, found_dead: 1`; dead sidecar deleted; alive entry in registry | happy-path |
| No sidecar files | `found_alive: 0, found_dead: 0` | happy-path |
| Corrupt sidecar JSON | 1 WARN logged; sidecar deleted; `errors` list has 1 entry | edge-case |
| `test_daemon_start_sequence_with_session_rediscovery` (integration) | Session sidecar pre-exists; daemon starts; session in InitialState after TUI connect | integration |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `rediscover_sessions()` completes before UDS bind (ordering check via mock startup sequence) | integration |
| VP-TBD | Alive session-host → `SessionEntry` with `state: Running` after re-discovery | integration |
| VP-TBD | Dead session-host → sidecar deleted; no registry entry | integration |
| VP-TBD | Wall-clock ≤ 5s for 8 sessions with `MockSessionHostSpawner` (all responding) | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — re-discovery on daemon restart is explicitly named in CAP-008; this BC defines the complete re-discovery algorithm including the ordering guarantee |
| Architecture Module | monocle-runtime (SessionManager `rediscover_sessions()`; `daemon_start_sequence` step 8b) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v1.2.0 §Daemon startup: session re-discovery; SS-daemon-wiring-v2-delta.md v1.1.0 §daemon_start_sequence() — session re-discovery step (step 8b placement and insertion invariant) |
| Test Name | test_BC_2_08_004_rediscovery_completes_before_uds_bind |

## Related BCs

- [BC-2.08.001] — depends on: session-state.json written at spawn is the input to re-discovery
- [BC-2.08.002] — composes with: persistence behavior (session survives restart) enabled by re-discovery
- [BC-2.08.005] — composes with: GC for dead sessions during re-discovery

## Architecture Anchors

- `architecture/SS-session-manager.md#daemon-startup-session-re-discovery` — re-discovery algorithm
- `architecture/SS-daemon-wiring-v2-delta.md#daemon_start_sequence-session-re-discovery-step` — step 8b placement

## Story Anchor

S-TBD — Implement daemon_start_sequence step 8b: rediscover_sessions() (filled by story-writer)

## VP Anchors

VP-TBD — Re-discovery integration tests including timing (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.004 authored for SS-08 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): 5s total budget with parallelized attach probing (`tokio::join_all`)
  specified explicitly. Sequential probing is NOT acceptable (up to 40s for 8 sessions). This
  design decision is production-grade and does not require human input — parallel I/O is the
  only correct choice for a startup sequence with a tight deadline.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
