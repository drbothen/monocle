---
document_type: behavioral-contract
level: L3
version: "1.3.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/SS-engine-module-v2-delta.md, architecture/adr/ADR-0009-native-session-host-process-model.md]
input-hash: "c6c45b3"
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

# Behavioral Contract BC-2.08.001: Session Spawn — SessionHostSpawner Called Within 2s; SessionEntry Created

## Description

`SessionManager::spawn_session()` receives a `SpawnRecipe` (from `ClaudeCodeModule::spawn_recipe()`),
a `harness_id`, and a `profile_id`. Within 2 seconds it must call `SessionHostSpawner::spawn()`
to start a `monocle-session-host` process, add a `SessionEntry` to the registry, write the
`session-state.json` sidecar, and return the `session_id` to the caller. The session's initial
`SessionEntry` state is `Launching` — there is no `Created` intermediate state; `SessionState::Created`
was removed from the state machine and spawn goes directly to `Launching`.

## Preconditions

1. `SessionManager` is initialized with a valid `spawner` (real or mock).
2. `recipe` is a valid `SpawnRecipe` (binary exists, args non-empty, cwd valid).
3. `harness_id` and `profile_id` are non-empty strings.
4. The daemon's `runtime_dir` exists and is writable.
5. `SessionHostSpawner::spawn()` is expected to succeed (pre-condition for happy-path).

## Postconditions

1. `SessionHostSpawner::spawn(session_id, recipe, runtime_dir)` is called within 2 seconds
   of `spawn_session()` being invoked. The `session_id` is a UUID v4 rendered as a String
   (generated via `uuid::Uuid::new_v4().to_string()` inside `spawn_session()`).
2. A `SessionEntry` is added to `SessionManager.sessions` keyed by `session_id`. The entry's
   initial `state` is `SessionState::Launching`.
3. `session-state.json` is written to `<runtime_dir>/session-<session_id>.json` with the
   schema specified in SS-session-manager.md §session-state.json schema:
   - `schema_version: 3`
   - `session_id`: the generated UUID string
   - `pid`: the `SpawnedHostHandle.pid` returned by the spawner
   - `socket_path`: `<runtime_dir>/session-<session_id>.sock`
   - `child_pid`: the session-host's tracked child PID (available after session-host startup)
   - `state: "Launching"`
   - `project_root`: `opts.project_root.to_string_lossy()` — the user-selected project
     directory from the SessionCreation wizard Step 2. Used for sessions panel grouping.
     This is distinct from `cwd` when a git worktree is configured.
   - `cwd`: `opts.worktree_root.to_string_lossy()` — the resolved worktree root path (per
     three-rule algorithm in SS-session-manager.md §SpawnOptions.worktree_root). Equals
     `project_root` when no worktree is configured or the project is not a git repo.
   - `harness_id`: the supplied `harness_id`
   - `profile_id`: the supplied `profile_id`
   - `started_at`: ISO-8601 UTC timestamp of spawn
   - `display_name`: defaults to `"<harness_id> — <project_root_basename>"`
   - `pty_rows: 24`, `pty_cols: 80` (initial default dimensions)
   - `kill_deadline_unix_ms: null` (always `null` for a freshly spawned `Launching` session;
     present in schema v3 for forward-compat; non-null only when `state == "Terminating"`)
4. `spawn_session()` returns `Ok(session_id)` (the UUID string).
5. `ServerToClient::SessionStateChanged { session_id, new_state: Launching }` is published
   to the broker BEFORE `ServerToClient::SessionListUpdate` (both under the `SessionManager`
   mutex per BC-2.08.008 Invariant 4 and SS-daemon-wiring-v2-delta.md v1.3.1 §3b).

## Invariants

1. The `session_id` MUST be unique across all sessions in the registry. UUID v4 generation
   provides collision-free IDs in practice; the implementation MUST check for collisions and
   regenerate if a collision is detected (probability negligible but the invariant is required
   for correctness under adversarial conditions).
2. `session-state.json` MUST be written atomically via `tempfile::persist` (per CLAUDE.md
   conventions — no naked `std::fs::write`).
3. `SessionState::Launching` is the transition state — the entry is visible in the registry
   but the session-host has not yet confirmed readiness. The TUI may display a "Launching..."
   indicator for sessions in this state.
4. `spawn_session()` does NOT wait for the session-host to reach `SessionState::Running`. It
   returns after the OS process is spawned and the sidecar is written. The
   `SessionState::Running` transition happens asynchronously when the session-host's UDS
   socket becomes connectable.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-150 | `SessionHostSpawner::spawn()` fails (e.g., binary not found at `recipe.binary`) | `spawn_session()` returns `Err(SessionError::SpawnFailed { reason: ... })`; no `SessionEntry` is added to the registry; no sidecar is written |
| EC-151 | `runtime_dir` is not writable (permissions error on sidecar write) | `spawn_session()` returns `Err(SessionError::SidecarWriteFailed { path: ..., reason: ... })`; the session-host OS process may have been spawned — because the session-host's UDS socket may not be bound yet (making `DaemonToHost::Kill` unreachable), `SessionManager` MUST send SIGTERM directly to `SpawnedHostHandle.pid`; if the process has not exited after 2 seconds, escalate to SIGKILL |
| EC-152 | UUID collision (session_id already in registry) | Regenerate UUID; retry once; if second collision occurs (astronomically unlikely), return `Err(SessionError::SessionIdCollision)` |
| EC-153 | `spawn_session()` called while re-discovery is still running | Re-discovery is guaranteed complete before UDS bind (BC-2.08.004 invariant); TUI cannot call `SpawnSession` before the UDS is up; this case is structurally impossible in correct deployment |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid `SpawnRecipe`, `harness_id: "claude-code"`, `profile_id: "default"`, `MockSessionHostSpawner` | `Ok(session_id)` where `session_id` is a non-empty UUID string; `SessionEntry` in registry with `state: Launching`; sidecar written to `runtime_dir/session-<id>.json` | happy-path |
| `MockSessionHostSpawner` configured to fail | `Err(SessionError::SpawnFailed {...})`; no registry entry; no sidecar | error |
| Two rapid `spawn_session()` calls | Two distinct session_ids; two distinct sidecars; both entries in registry | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `spawn_session()` returns `Ok(session_id)` within 2s using `MockSessionHostSpawner` | unit |
| VP-TBD | `SessionEntry` in registry with `state: Launching` after spawn | unit |
| VP-TBD | `session-state.json` written atomically; contents match schema | unit |
| VP-TBD | `SessionListUpdate` published to broker on successful spawn | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability §SS-08 |
| Capability Anchor Justification | CAP-008 ("Session lifecycle (spawn, kill, detach, rename); session-host process model; re-discovery on daemon restart; GC; hook auto-injection on spawn") per ARCH-INDEX §Capability traceability — this BC is the primary definition of the spawn operation that launches the session-host process and creates the session registry entry |
| L2 Domain Invariants | DI-007 (monocle must not write to any file owned by a harness — the sidecar is a monocle-owned file, not a harness file; the atomic write via tempfile::persist ensures no partial writes to monocle's own state) |
| Architecture Module | monocle-runtime (SessionManager sub-module — `monocle-runtime/src/session_manager/mod.rs`) per ARCH-INDEX Subsystem Registry SS-08 |
| Architecture Source | SS-session-manager.md v1.4.1 §SessionManager §Public API (spawn_session signature); SS-session-manager.md §session-state.json schema (schema_version 3); ADR-0009 §native-detached-session-host-process-model; SS-daemon-wiring-v2-delta.md v1.3.1 §3b (SessionStateChanged emission rule) |
| Test Name | test_BC_2_08_001_spawn_session_entry_created_within_2s |

## Related BCs

- [BC-2.03.005] — depends on: spawn_recipe() provides the SpawnRecipe consumed by spawn_session()
- [BC-2.08.002] — composes with: session-state.json written here is read during re-discovery
- [BC-2.08.003] — composes with: session spawned here is later killed via kill_session()
- [BC-2.08.006] — composes with: hook auto-injection in the SpawnRecipe is validated by this spawn

## Architecture Anchors

- `architecture/SS-session-manager.md#sessionmanager` — struct definition, public API, session lifecycle state machine
- `architecture/SS-session-manager.md#session-statejson-schema` — sidecar schema specification
- `architecture/adr/ADR-0009-native-session-host-process-model.md` — process model decision

## Story Anchor

S-TBD — Implement SessionManager::spawn_session() with SessionHostSpawner (filled by story-writer)

## VP Anchors

VP-TBD — Session spawn integration tests (filled after VP creation)

## §Trace v1.3.0

**Adversarial Pass 3 fixes — C3-002 (schema_version 3) + C3-001 (SessionStateChanged PC-5)** (2026-06-03):
- C3-002: PC-3 `schema_version` corrected from `1` to `3` (current canonical schema per
  SS-session-manager.md v1.4.1 §session-state.json schema). Schema v1 = no cwd field,
  v2 = adds cwd, v3 = adds kill_deadline_unix_ms. A newly spawned sidecar always writes v3
  (the current schema). The previous PC-3 said `schema_version: 1` while simultaneously
  listing a `cwd` field — these are internally inconsistent (cwd was added in v2). This is
  now corrected: v3 is the written version, and `kill_deadline_unix_ms: null` is explicitly
  enumerated in PC-3.
- C3-001: PC-5 updated from `SessionListUpdate` only to `SessionStateChanged{Launching}` THEN
  `SessionListUpdate` (ordered pair per BC-2.08.008 Invariant 4). Both are published under the
  same mutex lock in the same broker tick. Architecture Source updated to
  SS-session-manager.md v1.4.1 and SS-daemon-wiring-v2-delta.md v1.3.1.

## §Trace v1.2.0

**Architect-delegated BC edit — cwd vs project_root distinction in sidecar (I2-002)** (2026-06-03):
- I2-002 finding: BC-2.08.001 PC-3 sidecar schema listed `project_root` as `recipe.cwd`
  (incorrect). The architecture (SS-session-manager.md v1.3.0 §session-state.json schema)
  defines two distinct fields: `project_root` (user-selected project dir; used for sessions
  panel grouping) and `cwd` (resolved worktree root; passed to session-host as working dir).
  The two fields are equal only when no worktree is configured.
- PC-3 sidecar schema: split `project_root` and `cwd` fields with correct sources
  (`opts.project_root` vs `opts.worktree_root`). Added explanation of when they differ.
- Architecture Source updated to SS-session-manager.md v1.3.0.

## §Trace v1.1.0

**Adversarial pass-1 fixes — Description + EC-151** (2026-06-03):
- Description corrected: removed "Created → Launching states during this call" —
  `SessionState::Created` was removed by the architect; spawn transitions directly to `Launching`.
- EC-151 corrected: changed orphan-cleanup mechanism from "send `DaemonToHost::Kill`"
  to "send SIGTERM to `SpawnedHostHandle.pid` directly; escalate to SIGKILL after 2s
  if not exited." Rationale: the session-host's UDS socket may not be bound at the point of
  sidecar write failure, making the IPC Kill message unreachable. Direct PID signal is the
  correct cleanup path at this stage of the spawn sequence.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.08.001 authored for SS-08 (new subsystem) as part of the v1A control-center pivot BC burst.
- Covers: spawn_session() happy path, sidecar write, SessionListUpdate publication.
- Design decision (in-scope): 2s timing bound from SS-session-manager.md architect proposal
  preserved verbatim. The sidecar initial state is `Launching` not `Running` (session-host
  has not yet confirmed PTY ready); this distinction matters for re-discovery.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
