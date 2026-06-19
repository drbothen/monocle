---
document_type: behavioral-contract
level: L3
version: "1.3.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/SS-daemon-wiring-v2-delta.md]
input-hash: "36aa262"
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

1. `rediscover_sessions()` reads ALL `session-*.json` files in `runtime_dir`. Schema version
   handling:
   - `schema_version` 1: legacy — no `cwd` field (read as `cwd = project_root`),
     no `kill_deadline_unix_ms` field (read as `null`). Fully accepted.
   - `schema_version` 2: `cwd` present, no `kill_deadline_unix_ms`. Fully accepted.
   - `schema_version` 3: current — `cwd` and `kill_deadline_unix_ms` both present. Fully accepted.
   - `schema_version` > 3 (future versions): log WARN; skip sidecar (forward-compat); treated
     as orphaned (delete the sidecar file). Only versions 1, 2, and 3 are accepted.
2. For each sidecar with accepted `schema_version` (1, 2, or 3):
   a. `nix::sys::signal::kill(Pid::from_raw(pid), None)` probes process liveness.
   b. If alive: apply state-dependent handling:
      - **State `Launching` or `Running`:** Verify SO_PEERCRED peer uid matches daemon uid
        (per SS-session-manager.md v2.12.0 §Per-session UDS security I5); if mismatch → log
        WARN, SIGTERM both pids, delete sidecar, skip. If uid matches: send `DaemonToHost::Attach`;
        wait up to 5s for the full `HostToDaemon::ScrollbackChunk*` + `HostToDaemon::ScrollbackDumpComplete`
        sequence (chunked scrollback protocol — `ScrollbackDump` single-message form is
        RETIRED per SS-session-manager.md v2.12.0); on `ScrollbackDumpComplete` receipt, register
        `SessionEntry` with `state: Running` and populate `host_conn`.
      - **State `Detached` (I3-005 fix):** Verify SO_PEERCRED; if uid matches: register
        `SessionEntry` with `state: Detached` and `host_conn: None`. DO NOT send
        `DaemonToHost::Attach` — the session was intentionally detached; the user's detach
        intent MUST be preserved. The TUI may later send `ClientToServer::AttachSession` if
        the user chooses to resume. Do NOT emit `SessionStateChanged` — re-discovery registration
        of an unchanged persisted state is NOT a state-value transition. The first TUI client's
        initial `InitialState` / `SessionListUpdate` conveys the Detached session; nothing is lost.
      - **State `Terminating` (I3-002 fix):** Verify SO_PEERCRED; if uid matches:
        (i) Check `kill_deadline_unix_ms` from sidecar: if present and already elapsed →
            immediate SIGKILL to session-host PID; transition to `Terminated`; GC sidecar;
            do NOT register a `SessionEntry`. If present and not yet elapsed →
        (ii) Re-send `DaemonToHost::Kill` over the fresh SO_PEERCRED-verified UDS connect
            (fire-and-forget; do NOT wait for `StateChanged::Terminated`).
        (iii) Register `SessionEntry` with `state: Terminating`, `host_conn: None`,
             `kill_deadline` restored from sidecar's `kill_deadline_unix_ms`.
        (iv) Spawn a BACKGROUND watchdog tokio task: waits for `HostToDaemon::StateChanged::Terminated`
             up to the absolute deadline from `kill_deadline_unix_ms` (not a new 12s window from
             restart time). If Terminated received → GC sidecar. If deadline elapses → SIGKILL
             session-host PID; GC sidecar.
        (v) Return immediately from this probe. Terminating watchdog is a BACKGROUND task,
            excluded from the 5s `tokio::join_all` budget (see Invariant 2 and PC-7).
      - **State `Terminated`:** Should not appear (GC deletes sidecar on timer). If found:
        delete sidecar; skip. Treat as GC cleanup of a crash-leftover sidecar.
   c. If dead: delete sidecar file (and orphaned socket file if present); no `SessionEntry` added.
3. All `SessionEntry` records from step 2b are in `DaemonState.session_manager` before any
   TUI client can connect (UDS bind has not happened yet).
4. `rediscover_sessions()` returns `Ok(RediscoveryReport)` reporting: `found_alive: usize`,
   `found_dead: usize`, `errors: Vec<RediscoveryError>`.
5. On corrupt sidecar (JSON parse failure): log WARN, delete sidecar, continue; do NOT abort
   startup. A single corrupt sidecar MUST NOT prevent re-discovery of other sessions.
6. UDS socket bind (daemon_start_sequence step 10) proceeds only after `rediscover_sessions()`
   returns. If re-discovery fails entirely (e.g., `runtime_dir` unreadable), log ERROR and
   continue startup with an empty session registry — daemon starts clean rather than not at all.
7. The entire re-discovery synchronous phase (the `tokio::join_all` over all session probes)
   completes within 5 seconds for up to 8 concurrent sessions. **Terminating watchdog tasks
   are excluded from this budget** — they are background tokio tasks spawned in step 2b and
   are NOT awaited in the `join_all`. The 5s bound applies only to Launching/Running/Detached
   probe-and-attach operations.

## Invariants

1. **Ordering is mandatory:** Re-discovery MUST complete (the `join_all` synchronous phase)
   before UDS bind. This prevents the race condition where a TUI client connects and receives
   an `InitialState` push missing re-discovered sessions. Enforced by placement in
   `daemon_start_sequence` (step 8b precedes step 10). Background Terminating watchdog tasks
   may still be running after UDS bind — this is intentional and safe, because they only
   mutate their own session's state asynchronously via the broker.
2. The 5-second timeout per session-host during attach probing (Launching/Running state) is a
   hard deadline. After 5s without `ScrollbackDumpComplete`, the session-host is treated as
   non-responsive. No exponential backoff or retry — one attempt, 5s hard deadline. This
   timeout does NOT apply to Detached or Terminating state probes (Detached = no Attach sent;
   Terminating = fire-and-forget Kill + background watchdog).
3. Parallel attach: all session-host probes run concurrently (not sequentially) via
   `tokio::join_all`. With up to 8 sessions and 5s per timeout, sequential probing would take
   up to 40s — unacceptable for daemon startup.
4. A dead session-host (process not found) is GC'd immediately during re-discovery by
   deleting its sidecar. It does NOT appear in the registry or the initial session list.
5. Orphaned socket files (socket file exists but process is dead) are deleted during
   re-discovery alongside the sidecar.
6. **Detached intent MUST be preserved across restarts.** A sidecar with `state: "Detached"`
   is NOT force-attached on re-discovery. Doing so would violate the user's explicit detach
   action and consume proxy task budget unnecessarily (BC-2.08.007 Inv-1: Detached sessions
   don't stream). The TUI must send `ClientToServer::AttachSession` explicitly to resume.
7. **Terminating watchdog uses absolute deadline, not fresh 12s window.** If the sidecar's
   `kill_deadline_unix_ms` has already elapsed at re-discovery time, the daemon sends SIGKILL
   immediately. If not yet elapsed, the watchdog fires at the recorded deadline — preventing
   a daemon restart from resetting the SIGTERM/SIGKILL escalation indefinitely.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-167 | `runtime_dir` is empty (no sidecars) | `rediscover_sessions()` returns `Ok(RediscoveryReport { found_alive: 0, found_dead: 0, errors: [] })`; startup continues normally |
| EC-168 | One sidecar is corrupt JSON | WARN log; corrupt sidecar deleted; other sidecars processed normally; startup continues |
| EC-169 | Session-host process alive but socket file missing | Process alive (`kill(pid, None)` OK) but `connect(socket_path)` fails; session treated as non-responsive; sidecar deleted; `SIGTERM` sent to orphaned process |
| EC-170 | `runtime_dir` is unreadable | Log ERROR; `rediscover_sessions()` returns `Ok(RediscoveryReport { found_alive: 0, found_dead: 0, errors: [RuntimeDirUnreadable] })`; startup proceeds with empty registry |
| EC-171 | 8 sessions: 4 alive, 4 dead | All 8 probed in parallel; 4 dead sidecars deleted; 4 alive `SessionEntry` records added; total time ≤ 5s |
| EC-172 | Sidecar with `state: "Detached"` and alive session-host | SO_PEERCRED verified; `SessionEntry` registered with `state: Detached`, `host_conn: None`; NO `DaemonToHost::Attach` sent; NO `SessionStateChanged` emitted (re-discovery of unchanged state is not a transition; pre-UDS-bind, no client connected to receive it); TUI sees the Detached session in `InitialState` on first connect; user must explicitly AttachSession to resume streaming |
| EC-173 | Sidecar with `state: "Terminating"` and `kill_deadline_unix_ms` elapsed at re-discovery time | Immediate SIGKILL to session-host PID; no `SessionEntry` registered; sidecar deleted; `SessionStateChanged{Terminated}` then `SessionListUpdate` published (dead session GC path) |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| 2 sidecar files (schema_version 3); both session-hosts alive (mock) | `RediscoveryReport { found_alive: 2, found_dead: 0 }`; both entries in registry with `state: Running` | happy-path |
| 2 sidecar files; 1 alive, 1 dead | `found_alive: 1, found_dead: 1`; dead sidecar deleted; alive entry in registry | happy-path |
| No sidecar files | `found_alive: 0, found_dead: 0` | happy-path |
| Corrupt sidecar JSON | 1 WARN logged; sidecar deleted; `errors` list has 1 entry | edge-case |
| Sidecar `schema_version: 1` (legacy, no cwd field) | Accepted; `cwd = project_root`; `kill_deadline_unix_ms = null`; proceeds as Running re-discovery | edge-case |
| Sidecar `schema_version: 4` (unknown future version) | WARN logged; sidecar deleted (forward-compat); `errors` list has 1 entry | edge-case |
| Sidecar `state: "Detached"`; session-host alive | `SessionEntry` registered with `state: Detached`, `host_conn: None`; NO Attach sent | edge-case |
| Sidecar `state: "Terminating"`; `kill_deadline_unix_ms` elapsed | Immediate SIGKILL to PID; sidecar deleted; no `SessionEntry` | edge-case |
| Sidecar `state: "Terminating"`; `kill_deadline_unix_ms` NOT yet elapsed | Re-send Kill; register `SessionEntry{Terminating}`; background watchdog spawned; re-discovery returns immediately | edge-case |
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
| Architecture Source | SS-session-manager.md v2.12.0 §Daemon startup: session re-discovery (including §Re-discovery state handling — I4 all states covered; I3-002 Terminating watchdog; I3-005 Detached preservation); §session-state.json schema (schema_version history 1/2/3); SS-daemon-wiring-v2-delta.md v1.11.4 §daemon_start_sequence() — session re-discovery step (step 8b placement and insertion invariant) |
| Test Name | test_BC_2_08_004_rediscovery_completes_before_uds_bind |

## Related BCs

- [BC-2.08.001] — depends on: session-state.json written at spawn is the input to re-discovery
- [BC-2.08.002] — composes with: persistence behavior (session survives restart) enabled by re-discovery
- [BC-2.08.005] — composes with: GC for dead sessions during re-discovery

## Architecture Anchors

- `architecture/SS-session-manager.md#daemon-startup-session-re-discovery` — re-discovery algorithm
- `architecture/SS-daemon-wiring-v2-delta.md#daemon_start_sequence-session-re-discovery-step` — step 8b placement

## Story Anchor

S-036 — Implement daemon_start_sequence step 8b: rediscover_sessions()

## VP Anchors

VP-TBD — Re-discovery integration tests including timing (filled after VP creation)

## §Trace v1.3.1

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-036** (2026-06-15):
- Story Anchor filled from Phase-2 Burst A story decomposition. No behavioral content changed.

## §Trace v1.3.0

**F-P47-001 — Detached re-discovery MUST NOT emit SessionStateChanged (Option B decision)** (2026-06-14):

- **Finding (F-P47-001):** PC-2b `State Detached` case mandated `Emit \`SessionStateChanged{Detached}\`
  then \`SessionListUpdate\`` after re-discovery registration. This was asymmetric with the
  `Launching`/`Running` re-discovery cases which register `SessionEntry` without emitting anything.
  Re-discovery registration of an unchanged persisted state (Detached→Detached) is NOT a state-value
  transition and MUST NOT emit `SessionStateChanged`.
- **Decision (Option B, adjudicated):** `SessionStateChanged` is emitted ONLY on a genuine
  state-VALUE change. Re-discovery of a Detached session that is still Detached is not a transition.
  The Terminated-GC re-discovery emission stays (persisted Terminating → Terminated IS a real value
  change). Re-discovery occurs before any TUI client connects (broker discards messages); the first
  client's `InitialState` conveys the Detached session — nothing is functionally lost.
- **PC-2b (normative change):** Removed `Emit \`SessionStateChanged{Detached}\` then \`SessionListUpdate\`.`
  Replaced with: "Do NOT emit `SessionStateChanged` — re-discovery registration of an unchanged
  persisted state is NOT a state-value transition. The first TUI client's initial `InitialState` /
  `SessionListUpdate` conveys the Detached session; nothing is lost."
- **EC-172 (normative change):** Removed `SessionStateChanged{Detached}` and `SessionListUpdate
  published` from expected behavior. Now: "NO `SessionStateChanged` emitted (re-discovery of
  unchanged state is not a transition; pre-UDS-bind, no client connected to receive it); TUI sees
  the Detached session in `InitialState` on first connect."
- **Symmetry:** Detached re-discovery now matches Launching/Running re-discovery cases — all three
  register `SessionEntry` without emitting SessionStateChanged or SessionListUpdate. The emission
  asymmetry was the defect.
- Minor bump: v1.2.1 → v1.3.0.

## §Trace v1.2.1

**Arch-source pin v1.9.0→v1.9.1** (2026-06-13 / D-277):
- Arch-source pin: SS-daemon-wiring-v2-delta.md v1.9.0 → v1.9.1 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.2.0

**Adversarial Pass 3 fixes — C3-002 (schema multi-version) + I3-002 (Terminating watchdog) + I3-005 (Detached preservation)** (2026-06-03):
- C3-002: PC-1 corrected — re-discovery now accepts schema_version 1 (legacy: cwd=project_root,
  no kill_deadline), 2 (cwd present), AND 3 (current). The previous PC-2 said "skip schema_version
  not 1" which would have self-deleted every sidecar monocle writes (monocle writes schema_version 3
  per BC-2.08.001 PC-3). Fixed to skip/orphan-delete ONLY schema_version > 3 (forward-compat).
  Schema version history documented in PC-1 for all three accepted versions.
- I3-005: PC-2b Detached state handling added: re-discovery of a Detached sidecar registers
  `SessionEntry{state: Detached, host_conn: None}` WITHOUT sending `DaemonToHost::Attach`.
  Detached intent persisted across restart. EC-172 added. Invariant 6 added.
- I3-002: PC-2b Terminating state handling added: absolute-deadline kill_deadline_unix_ms
  check (immediate SIGKILL if elapsed; fire-and-forget Kill + background watchdog if not).
  Background watchdog is excluded from the 5s join_all budget. EC-173 added. Invariant 7 added.
  PC-7 updated to explicit "5s applies to Launching/Running/Detached only; Terminating watchdog
  excluded."
- Architecture Source updated to SS-session-manager.md v1.5.0 and SS-daemon-wiring-v2-delta.md v1.3.1.

## §Trace v1.1.0

**I2-005 adversarial pass-2 fix — sync re-discovery to canonical procedure (SS-session-manager v1.3.0)** (2026-06-03):
- I2-005 finding: BC-2.08.004 PC-2b referenced `HostToDaemon::ScrollbackDump` (single-message
  retired form) as the success signal. SS-session-manager v1.3.0 retires `ScrollbackDump` in
  favor of the chunked `ScrollbackChunk*` + `ScrollbackDumpComplete` protocol. Invariant 2
  similarly referenced `ScrollbackDump`.
- PC-2b: updated to reference `ScrollbackChunk*` + `ScrollbackDumpComplete` chunked protocol;
  added SO_PEERCRED peer-uid cross-check step before `DaemonToHost::Attach` (per I5 security
  fix in SS-session-manager v1.3.0 §Per-session UDS security); clarified no exponential backoff.
- Invariant 2: updated `ScrollbackDump` → `ScrollbackDumpComplete` with explicit "no exponential
  backoff" clarification.
- Architecture Source: version pins bumped to SS-session-manager.md v1.3.0 and
  SS-daemon-wiring-v2-delta.md v1.2.0.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.004 authored for SS-08 as part of the v1A control-center pivot BC burst.
- Design decision (in-scope): 5s total budget with parallelized attach probing (`tokio::join_all`)
  specified explicitly. Sequential probing is NOT acceptable (up to 40s for 8 sessions). This
  design decision is production-grade and does not require human input — parallel I/O is the
  only correct choice for a startup sequence with a tight deadline.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).

## §Trace v1.3.2

**Phase-2 Pass-1 fix burst — SS-session-manager v2.7.3 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.3.2 timestamp >= v1.3.1. PASS.
