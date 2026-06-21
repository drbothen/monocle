---
document_type: story
level: L4
story_id: S-036
epic_id: EPIC-08
version: "1.6"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-15T00:00:00Z
phase: 2
points: 8
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-033, S-034, S-035]
blocks: []
target_module: monocle-runtime
subsystems: [SS-08]
behavioral_contracts: [BC-2.08.002, BC-2.08.004]
verification_properties: []
estimated_days: 4
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.002.md, version: "1.2.5"}
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.004.md, version: "1.4.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.15.1"}
  - {path: .factory/specs/architecture/SS-daemon-wiring-v2-delta.md, version: "1.12.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.08.002 (session-host setsid + survival across graceful daemon restart) and BC-2.08.004 (rediscover_sessions: all alive sessions visible after restart within 5s; UDS bind blocked until complete)"
# BC status: BC-2.08.002, BC-2.08.004 — non-empty; status draft pending Phase-2 adversarial convergence gate
---

# S-036: SessionManager::rediscover_sessions — setsid Persistence; All States Handled Within 5s; UDS Bind Blocked

## Narrative

As the monocle daemon, I want `SessionManager::rediscover_sessions()` to run as
`daemon_start_sequence` step 8b — BEFORE the UDS socket is bound — probing all
`session-*.json` sidecar files in parallel via `tokio::join_all`, handling every persisted
`SessionState` correctly (Running/Launching attach, Detached preserve, Terminating watchdog,
Terminated GC), and completing the synchronous phase within 5 seconds for up to 8 sessions —
so that no TUI client ever receives a stale (incomplete) initial session list after a daemon
restart, and sessions that were running when the daemon crashed resume seamlessly for the user.

## Acceptance Criteria

### AC-001 (traces to BC-2.08.002 postcondition 1-2 — session-host survives daemon graceful shutdown; setsid)

When the daemon undergoes a graceful shutdown:
- The `monocle-session-host` process continues running. It is NOT sent SIGTERM or SIGHUP by the daemon's shutdown sequence.
- The session-host called `nix::unistd::setsid()` at startup step 2, making it a process group leader immune to SIGHUP when the daemon exits.
- `session-state.json` remains intact in `runtime_dir` after daemon exit.
- The harness child (Claude Code) continues running inside the session-host's PTY.

### AC-002 (traces to BC-2.08.002 postcondition 3 — rediscover_sessions called at daemon_start_sequence step 8b)

`SessionManager::rediscover_sessions()` is called during `daemon_start_sequence` at step 8b —
AFTER the lock file is written (step 8) but BEFORE the UDS socket is bound (step 10). This
ordering is enforced by the placement in `daemon_start_sequence()` per
SS-daemon-wiring-v2-delta.md §daemon_start_sequence() — session re-discovery step.

### AC-003 (traces to BC-2.08.004 postcondition 1 — schema version handling: all three versions accepted; future skipped)

`rediscover_sessions()` reads ALL `session-*.json` files in `runtime_dir`. Schema version handling:
- `schema_version` 1: `cwd = project_root`, `kill_deadline_unix_ms = null`. Fully accepted.
- `schema_version` 2: `cwd` present, `kill_deadline_unix_ms = null`. Fully accepted.
- `schema_version` 3: current; all fields present. Fully accepted.
- `schema_version` > 3: log WARN; delete sidecar (forward-compat orphan); continue. NOT accepted.

### AC-004 (traces to BC-2.08.004 postcondition 2b — state Launching or Running: attach + 5s timeout; SO_PEERCRED; register Running)

For sidecar with `state: "Launching"` or `state: "Running"` and alive PID:
- Verify SO_PEERCRED peer uid matches daemon uid. If mismatch: log WARN; SIGTERM both PIDs; delete sidecar; skip.
- Send `DaemonToHost::Attach`; wait up to 5s for full `HostToDaemon::ScrollbackChunk*` + `HostToDaemon::ScrollbackDumpComplete` sequence. The retired single-message `ScrollbackDump` form is NOT accepted.
- On `ScrollbackDumpComplete` receipt: register `SessionEntry` with `state: Running` and `host_conn: Some(SessionHostConnection { writer, proxy_task: None })`. Re-discovery registers the control connection only; the PTY-streaming proxy task (`proxy_task: Some(...)`) is established on an explicit `AttachSession` request (S-035 `attach_session()` path) — PTY output pipeline is S-039/S-047 scope. See SS-session-manager.md §Re-discovery state handling and §SessionHostConnection (`proxy_task` is None during Launching; started at `Launching → Running` via `StateChanged::Running` only).
- If 5s timeout: treat session-host as non-responsive; send SIGTERM to session-host PID; delete sidecar; skip (no `SessionEntry` added).

### AC-005 (traces to BC-2.08.004 postcondition 2b — state Detached: preserve intent; NO Attach sent; NO SessionStateChanged emitted)

For sidecar with `state: "Detached"` and alive PID:
- Verify SO_PEERCRED peer uid matches daemon uid. If mismatch: same non-responsive treatment as above.
- Register `SessionEntry` with `state: Detached` and `host_conn: None`.
- DO NOT send `DaemonToHost::Attach` — the session was intentionally detached; the user's detach intent MUST be preserved.
- DO NOT emit `SessionStateChanged` — re-discovery registration of an unchanged persisted state is NOT a state-value transition (BC-2.08.004 §F-P47-001; BC-2.08.008 Invariant 1).
- The TUI sees the Detached session in `InitialState` on first connect; user must send `ClientToServer::AttachSession` to resume streaming.

### AC-006 (traces to BC-2.08.004 postcondition 2b — state Terminating: absolute kill_deadline; fire-and-forget Kill; background watchdog)

For sidecar with `state: "Terminating"` and alive PID:
- Check `kill_deadline_unix_ms` from sidecar:
  - If present and ALREADY ELAPSED at re-discovery time: immediate SIGKILL to session-host PID; NO `SessionEntry` registered; delete sidecar. (The SIGTERM window expired across daemon restart cycles — do not reset.)
  - If present and NOT YET ELAPSED: proceed to watchdog path below.
  - If null or absent: proceed to watchdog path with a new 12s window from now.
- Verify SO_PEERCRED; re-send `DaemonToHost::Kill` (fire-and-forget; do NOT wait for `StateChanged::Terminated`).
- Register `SessionEntry` with `state: Terminating`, `host_conn: None`, `kill_deadline` restored from sidecar.
- Spawn a BACKGROUND watchdog tokio task: waits up to the absolute deadline from `kill_deadline_unix_ms` (not a new 12s window). On `StateChanged::Terminated` received → GC sidecar. If deadline elapses → SIGKILL; GC sidecar.
- Return IMMEDIATELY from this probe (background watchdog is now detached; excluded from 5s `join_all` budget per BC-2.08.004 Invariant 2 and PC-7).

### AC-007 (traces to BC-2.08.004 postcondition 2b — state Terminated: GC immediately; state Unknown: delete and skip)

- Sidecar with `state: "Terminated"`: delete sidecar; skip (crash-leftover cleanup; no `SessionEntry`).
- Sidecar with unknown state string: log WARN; delete sidecar; skip (forward-compat).

### AC-008 (traces to BC-2.08.004 postcondition 2c — dead PID: GC immediately)

For a sidecar where `nix::sys::signal::kill(Pid::from_raw(pid), None)` fails (process dead):
- Delete the sidecar file.
- Delete orphaned socket file if it exists.
- No `SessionEntry` added.

### AC-009 (traces to BC-2.08.004 postcondition 4 — RediscoveryReport returned)

`rediscover_sessions()` returns `Ok(RediscoveryReport)` with:
- `found_alive: usize` — count of sessions successfully re-registered.
- `found_dead: usize` — count of sessions GC'd (dead PID or non-responsive).
- `errors: Vec<RediscoveryError>` — any non-fatal errors (corrupt sidecars, unknown schema, etc.).

### AC-010 (traces to BC-2.08.004 postcondition 5 — corrupt sidecar: WARN; delete; continue)

A corrupt JSON sidecar (parse failure) MUST NOT prevent re-discovery of other sessions:
log WARN; delete the corrupt sidecar; continue; add a `RediscoveryError` to the report.

### AC-011 (traces to BC-2.08.004 postcondition 6 — UDS socket bind blocked until re-discovery returns)

`daemon_start_sequence` step 10 (UDS bind) MUST NOT proceed until `rediscover_sessions()` returns. This is enforced by sequential `await` ordering in `daemon_start_sequence()`. If re-discovery fails entirely (e.g., `runtime_dir` unreadable), log ERROR; proceed with empty registry.

### AC-012 (traces to BC-2.08.004 postcondition 7 / invariant 3 — parallel probing via tokio::join_all within 5s for up to 8 sessions)

All session-host probes for Launching/Running/Detached states run concurrently via `tokio::join_all`. Wall-clock time for the synchronous phase (excludes Terminating background watchdogs) MUST complete within 5 seconds for up to 8 concurrent sessions. Sequential probing is NOT acceptable.

### AC-013 (traces to BC-2.08.004 invariant 6 — Detached intent preserved across restart)

Re-discovery of a Detached sidecar MUST NOT force-attach the session. Doing so would violate the user's explicit detach action (BC-2.08.007 Invariant 1) and would start 8 proxy tasks unnecessarily. Test: spawn session → detach → simulate daemon restart (recreate `SessionManager` with same `runtime_dir`) → verify session re-discovered as Detached; NO `DaemonToHost::Attach` was sent to mock session-host.

### AC-014 (traces to BC-2.08.004 invariant 7 — Terminating absolute deadline preserved across restart)

When re-discovering a `Terminating` sidecar with an ELAPSED `kill_deadline_unix_ms`: daemon sends SIGKILL immediately. Test uses time injection to create an elapsed deadline.

### AC-015 (traces to BC-2.08.002 postcondition 5-7 — re-discovered session appears in InitialState after restart)

After re-discovery completes, the first TUI client to connect receives an `InitialState` push that includes the re-discovered session(s) in its sessions list. Re-discovered `Running` sessions have their scrollback buffer available. Integration test: spawn session → graceful daemon shutdown → daemon restart → TUI connects → asserts session in `InitialState.sessions` with `state: Running`.

## Tasks

- [ ] Implement `SessionManager::rediscover_sessions(&mut self) -> Result<RediscoveryReport, SessionError>`:
  - Read all `session-*.json` from `runtime_dir` via `std::fs::read_dir`.
  - Parse each sidecar; check `schema_version` (1/2/3 accepted; >3 WARN+delete+skip).
  - On parse failure: WARN+delete+skip; add to `errors`.
  - Collect all valid sidecars; probe each via `nix::sys::signal::kill(pid, None)`.
  - For alive sessions: build a `Vec<Future>` of state-dependent probe tasks; await all with `tokio::join_all` (5s total via `tokio::time::timeout`).
  - For Terminating: spawn background watchdog; return probe immediately (excluded from join_all budget).
  - Delete dead sidecars and orphaned socket files.
  - Return `Ok(RediscoveryReport { found_alive, found_dead, errors })`.
- [ ] Define `RediscoveryReport { found_alive: usize, found_dead: usize, errors: Vec<RediscoveryError> }` and `RediscoveryError` enum.
- [ ] Wire `rediscover_sessions()` call into `daemon_start_sequence()` at step 8b (after lock file write, before UDS bind) in `monocle-runtime/src/daemon.rs` (or equivalent startup file).
- [ ] Verify `monocle-session-host/src/main.rs` calls `nix::unistd::setsid()` at startup step 2 (established in S-033; confirm setsid call is present and before PTY open).
- [ ] Write unit test `test_BC_2_08_004_rediscovery_running_session_reregistered`: pre-populate sidecar with `state: "Running"` + mock alive process; run `rediscover_sessions()`; verify `SessionEntry{Running}` in registry; `found_alive: 1`.
- [ ] Write unit test `test_BC_2_08_004_rediscovery_detached_no_attach_sent`: sidecar `state: "Detached"`; verify NO `DaemonToHost::Attach` sent to mock session-host; `SessionEntry{Detached, host_conn: None}` in registry; NO `SessionStateChanged` emitted.
- [ ] Write unit test `test_BC_2_08_004_rediscovery_dead_pid_gc`: sidecar with dead PID; sidecar deleted; no registry entry; `found_dead: 1`.
- [ ] Write unit test `test_BC_2_08_004_rediscovery_corrupt_sidecar`: corrupt JSON; WARN logged; sidecar deleted; `errors` list has 1 entry; other sessions unaffected.
- [ ] Write unit test `test_BC_2_08_004_rediscovery_schema_v1_legacy`: `schema_version: 1` (no cwd); accepted; `cwd = project_root`; proceeds as Running re-discovery.
- [ ] Write unit test `test_BC_2_08_004_rediscovery_schema_v4_future`: `schema_version: 4`; WARN; deleted; skipped; `found_dead` NOT incremented.
- [ ] Write unit test `test_BC_2_08_004_rediscovery_terminating_elapsed_deadline`: `state: "Terminating"` + `kill_deadline_unix_ms` elapsed; immediate SIGKILL to PID; sidecar deleted; no `SessionEntry`.
- [ ] Write unit test `test_BC_2_08_004_rediscovery_terminating_not_elapsed_deadline`: `state: "Terminating"` + `kill_deadline_unix_ms` not elapsed; Kill fire-and-forget; `SessionEntry{Terminating}` registered; background watchdog spawned; re-discovery returns immediately.
- [ ] Write unit test `test_BC_2_08_004_rediscovery_parallelism_8_sessions`: 8 mock alive session-hosts; all probed concurrently; wall-clock ≤ 5s via `tokio::time::pause()`.
- [ ] Write integration test `test_BC_2_08_002_session_survives_daemon_graceful_restart`: daemon starts → spawns session → daemon gracefully stops → daemon restarts → `rediscover_sessions()` → session appears in `InitialState.sessions` with state Running; scrollback buffer available.
- [ ] Write integration test `test_BC_2_08_004_rediscovery_completes_before_uds_bind`: verify `rediscover_sessions()` completes before the UDS socket file appears in `runtime_dir` (ordering check).

## Previous Story Intelligence

- **S-033** (spawn): `SessionManager`, `SessionEntry`, `SessionState`, `SessionHostConnection`, `SessionHostSpawner`, `SpawnedHostHandle`, and sidecar write are all established. The session-host binary calls `setsid()` at step 2 (confirm this; do not reimplement).
- **S-034** (kill): `kill_session()` and the 12s watchdog pattern are established; Terminating re-discovery leverages the same watchdog infrastructure.
- **S-035** (attach/detach): `attach_session()` is established; Launching/Running re-discovery path reuses the same Attach + ScrollbackChunk* protocol. Note: re-discovery does NOT replay the post-spawn monitor — it connects directly and sends `Attach`, treating the session-host as already running (its event loop is live).
- `daemon_start_sequence()` location: `monocle-runtime/src/daemon.rs` or the equivalent startup entry point from S-017. Step 8b insertion must come AFTER the lock file write step (step 8) and BEFORE the UDS bind step (step 10).

## Architecture Compliance Rules

- Re-discovery MUST complete (the `join_all` synchronous phase) BEFORE UDS bind. This ordering is a HARD INVARIANT (BC-2.08.004 Invariant 1). Background Terminating watchdog tasks may still be running after UDS bind — this is intentional and safe.
- Terminating watchdog tasks are BACKGROUND `tokio::spawn` tasks, excluded from the 5s `join_all` budget (BC-2.08.004 Invariant 2 and PC-7). The 5s bound applies only to Launching/Running/Detached probe-and-attach.
- Detached sidecar re-discovery: MUST NOT send `DaemonToHost::Attach`. MUST NOT emit `SessionStateChanged` (re-discovery of unchanged persisted state is not a transition; BC-2.08.004 §F-P47-001).
- Terminating `kill_deadline_unix_ms`: if elapsed at re-discovery time → IMMEDIATE SIGKILL (not a new 12s window). This prevents repeated daemon restarts from resetting the kill escalation indefinitely.
- Schema version > 3: WARN + delete sidecar (forward-compat orphan). Do NOT skip silently without logging.
- SO_PEERCRED: mandatory on EVERY per-session UDS connect during re-discovery. No exceptions.
- `tempfile::persist` for all sidecar writes; `std::fs::remove_file` for sidecar deletes (tolerates ENOENT).
- Forbidden dependency: `monocle-runtime` MUST NOT depend on `monocle-tui`.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tokio` | `=1.52` (exact) | `tokio::join_all`; `tokio::time::timeout`; `tokio::spawn` for background watchdog | SS-deps-pin-manifest.md |
| `nix` | `"0.30"` | `nix::sys::signal::kill(Pid, None)` for liveness probe; `nix::unistd::setsid()` in session-host (from S-033) | SS-deps-pin-manifest.md |
| `serde_json` | `=1.0.149` (exact) | Sidecar JSON parse (`serde_json::from_str`) | SS-deps-pin-manifest.md |
| `tempfile` | `"3"` | Atomic sidecar writes (Terminating watchdog updates) | SS-deps-pin-manifest.md |
| `thiserror` | `"2"` | `RediscoveryError` enum | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to CREATE:

| File | Purpose |
|------|---------|
| (none — all crates established by S-033) | |

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/session_manager/mod.rs` | Add `rediscover_sessions()` implementation; add `RediscoveryReport`, `RediscoveryError` types |
| `crates/monocle-runtime/src/daemon.rs` (or startup entrypoint from S-017) | Insert `session_manager.rediscover_sessions().await?` at step 8b (after lock file write, before UDS bind) |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~4,000 |
| BC-2.08.002 | ~2,000 |
| BC-2.08.004 | ~4,000 |
| SS-session-manager.md (re-discovery algorithm; all-states handling; setsid; Terminating watchdog; Detached preservation; SO_PEERCRED) | ~12,000 |
| SS-daemon-wiring-v2-delta.md (step 8b placement) | ~2,000 |
| Existing session_manager code from S-033/S-034/S-035 | ~8,000 |
| Test files | ~5,000 |
| **Total estimate** | **~37,000** |

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.08.002 | Session Persistence — session-host Survives Graceful Daemon Restart | (see inputs: frontmatter) |
| BC-2.08.004 | Re-Discovery — All Alive Sessions Visible After Daemon Restart Within 5s; UDS Bind Blocked Until Complete | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `rediscover_sessions()` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (filesystem I/O; UDS connect; SO_PEERCRED; broker publish; OS signal) |
| `RediscoveryReport` / `RediscoveryError` | `monocle-runtime/src/session_manager/mod.rs` | Pure (data types) |
| Terminating background watchdog | `monocle-runtime/src/session_manager/mod.rs` | Effectful (tokio::spawn; SIGKILL; sidecar GC; broker publish) |
| `daemon_start_sequence` step 8b insertion | `monocle-runtime/src/daemon.rs` | Effectful (startup sequencing) |
| `setsid()` call in session-host | `crates/monocle-session-host/src/main.rs` | Effectful (process group leader) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-155 | Session-host alive but UDS socket file deleted (tmpfs cleared) | `kill(pid, None)` OK; `connect(socket_path)` fails; session → Terminated; sidecar deleted; TUI omits session |
| EC-156 | Session-host alive but does not respond within 5s (stuck) | Session → Terminated; sidecar deleted; SIGTERM to PID |
| EC-157 | Daemon exits with SIGKILL (crash); session-host survives (setsid) | Session-host process survives; sidecar intact; next daemon startup re-discovers |
| EC-158 | Session was in `Launching` when daemon exited | Same Launching handling: Attach + 5s timeout; if `ScrollbackDumpComplete` received → Running; else SIGTERM+GC |
| EC-159 | Multiple sessions; one alive, one dead | Alive → Running; dead → GC; TUI receives `InitialState` with only alive session |
| EC-167 | `runtime_dir` empty (no sidecars) | `RediscoveryReport { found_alive: 0, found_dead: 0, errors: [] }`; startup continues normally |
| EC-168 | One sidecar corrupt JSON | WARN; sidecar deleted; other sessions processed; startup continues |
| EC-170 | `runtime_dir` unreadable | ERROR logged; `RediscoveryReport { errors: [RuntimeDirUnreadable] }`; daemon starts with empty registry |
| EC-172 | `state: "Detached"` sidecar; alive process | SO_PEERCRED check; `SessionEntry{Detached, host_conn: None}` registered; NO Attach; NO `SessionStateChanged` |
| EC-173 | `state: "Terminating"` sidecar; `kill_deadline_unix_ms` ELAPSED at re-discovery | Immediate SIGKILL; sidecar deleted; no `SessionEntry` |

## Subsystem Anchor Justifications

**SS-08 owns this story's scope** because `rediscover_sessions()` is a core `SessionManager` operation (SS-session-manager.md §Daemon startup: session re-discovery) and the `setsid()` requirement is defined in SS-session-manager.md §monocle-session-host binary §startup sequence step 2.

**Dependency Anchors:**
- STORY-036 depends on S-033 because `SessionManager`, `SessionEntry`, `SessionHostConnection`, `SessionState`, the sidecar schema (schema_version 3), and the session-host binary (`setsid()` at startup step 2) all must exist.
- STORY-036 depends on S-034 because the Terminating re-discovery path reuses the 12s watchdog pattern and `kill_deadline_unix_ms` sidecar field established by kill_session().
- STORY-036 depends on S-035 because the Launching/Running re-discovery path reuses the `attach_session()` protocol (Attach + ScrollbackChunk* + ScrollbackDumpComplete) established by S-035.
