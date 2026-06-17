---
document_type: behavioral-contract
level: L3
version: "1.5.3"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-14T12:00:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-session-manager.md, architecture/SS-engine-module-v2-delta.md, architecture/SS-ipc.md, architecture/adr/ADR-0009-native-session-host-process-model.md]
input-hash: "051000a"
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

`SessionManager::spawn_session()` receives a `SpawnOptions` value from the TUI (via
`ClientToServer::SpawnSession { opts }`). As its **first step**, it calls
`engine_module.spawn_recipe(&opts)?` (daemon-side) to obtain a `SpawnRecipe`. If
`spawn_recipe()` fails (e.g., `EngineError::BinaryNotFound` — harness not on PATH;
`EngineError::InvalidPath` — invalid hooks settings path), the error is converted to
`SessionError::EngineError` and returned before any OS process is spawned. On success,
`spawn_session()` uses the returned `SpawnRecipe` to call `SessionHostSpawner::spawn()`,
start a `monocle-session-host` process, add a `SessionEntry` to the registry, write the
`session-state.json` sidecar, and return the `session_id` to the caller — all within 2
seconds. The session's initial `SessionEntry` state is `Launching` — there is no `Created`
intermediate state; `SessionState::Created` was removed from the state machine and spawn
goes directly to `Launching`.

## Preconditions

1. `SessionManager` is initialized with a valid `spawner` (real or mock) and a bound
   `engine_module` reference.
2. `opts` is a valid `SpawnOptions` with the required fields populated: `project_root`,
   `worktree_root`, `harness_id`, `profile_id`, and `ccr_base_url` are set by the TUI
   (from the SessionCreation wizard); `session_id` and `hooks_settings_path` are filled by
   the daemon IPC handler on receipt of `ClientToServer::SpawnSession { opts }`, before
   `spawn_session(opts)` is called. The binary-existence check and arg-validity check occur
   DAEMON-SIDE as the first step of `spawn_session()` via
   `engine_module.spawn_recipe(&opts)?` — they are NOT preconditions of the caller.
3. The daemon's `runtime_dir` exists and is writable.
4. `SessionHostSpawner::spawn()` is expected to succeed (pre-condition for happy-path).

## Postconditions

1. `SessionHostSpawner::spawn(session_id, recipe, runtime_dir)` is called within 2 seconds
   of `spawn_session()` being invoked. The `session_id` is a UUID v4 rendered as a String
   (generated via `uuid::Uuid::new_v4().to_string()` in the daemon IPC handler (the
   `ClientToServer::SpawnSession` match arm) BEFORE `spawn_session()` is called.
   `spawn_session()` receives `opts.session_id` already populated via
   `opts.with_daemon_fields()`; it does NOT generate the UUID itself.).
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
   mutex per BC-2.08.008 Invariant 4 and SS-daemon-wiring-v2-delta.md v1.11.4 §3b).

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
| EC-150 | The harness binary is not on `PATH` (`which::which("claude")` fails inside `spawn_recipe()`) | `spawn_recipe()` returns `Err(EngineError::BinaryNotFound("claude"))`, propagated via `?` as `SessionError::EngineError(BinaryNotFound)`. `spawn_session()` returns this error before any OS process is spawned. The IPC handler maps it to `ServerToClient::Error { code: "binary_not_found", message: ... }` via `session_error_to_code(IpcOp::Spawn, &e)`. No `SessionEntry` is added to the registry; no sidecar is written. The TUI displays the fixed banner: `"claude binary not found — is Claude Code installed and on PATH?"` (per BC-2.03.007 PC-3). |
| EC-151 | `runtime_dir` is not writable (permissions error on sidecar write) | `spawn_session()` returns `Err(SessionError::SidecarWriteFailed { path: ..., reason: ... })`; the session-host OS process may have been spawned — because the session-host's UDS socket may not be bound yet (making `DaemonToHost::Kill` unreachable), `SessionManager` MUST send SIGTERM directly to `SpawnedHostHandle.pid`; if the process has not exited after 2 seconds, escalate to SIGKILL |
| EC-152 | UUID collision (session_id already in registry) | Regenerate UUID; retry once; if second collision occurs (astronomically unlikely), return `Err(SessionError::SessionIdCollision)` |
| EC-153 | `spawn_session()` called while re-discovery is still running | Re-discovery is guaranteed complete before UDS bind (BC-2.08.004 invariant); TUI cannot call `SpawnSession` before the UDS is up; this case is structurally impossible in correct deployment |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid `SpawnOptions` (`harness_id: "claude-code"`, `profile_id: "default"`, `claude` binary on PATH), `MockSessionHostSpawner` succeeds | `Ok(session_id)` where `session_id` is a non-empty UUID string; `SessionEntry` in registry with `state: Launching`; sidecar written to `runtime_dir/session-<id>.json` | happy-path |
| `claude` binary NOT on PATH; `MockSessionHostSpawner` not reached | `Err(SessionError::EngineError(BinaryNotFound("claude")))`; no registry entry; no sidecar; IPC code `"binary_not_found"` (EC-150) | error |
| `hooks_settings_path` is non-UTF-8; `MockSessionHostSpawner` not reached | `Err(SessionError::EngineError(InvalidPath(...)))`; no registry entry; no sidecar; IPC code `"invalid_spawn_arg"` (per BC-2.03.007 PC-7) | error |
| `MockSessionHostSpawner` configured to fail (OS spawn failure after binary located) | `Err(SessionError::SpawnFailed {...})`; no registry entry; no sidecar; IPC code `"spawn_failed"` | error |
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
| Architecture Source | SS-session-manager.md v2.6.1 §SessionManager §Public API (`spawn_session(opts: SpawnOptions)` signature — Model A; `spawn_recipe()` called daemon-side as first step; UUID generated in IPC handler BEFORE spawn_session()); SS-session-manager.md v2.6.1 §session-state.json schema (schema_version 3); SS-session-manager.md v2.6.1 §session_error_to_code (spawn-path arms: `EngineError::BinaryNotFound` → `"binary_not_found"`, `EngineError::InvalidPath` → `"invalid_spawn_arg"`); SS-engine-module-v2-delta.md v1.6.0 §SpawnOptions and §SpawnRecipe types (Model A wire/internal type assignment — I27-001); SS-ipc.md v1.24.0 §`ClientToServer::SpawnSession { opts: SpawnOptions }` (wire variant — Model A) + §`ServerToClient::SpawnAck { session_id }` (new variant — F-P41-IMP-001); ADR-0009 v1.0.2 §Decision; SS-daemon-wiring-v2-delta.md v1.11.4 §3b (SessionStateChanged emission rule) |
| Test Name | test_BC_2_08_001_spawn_session_entry_created_within_2s |

## Related BCs

- [BC-2.03.005] — depends on: spawn_recipe() is called DAEMON-SIDE as the first step of spawn_session() to build the SpawnRecipe from SpawnOptions (Model A — I27-001); BC-2.08.001 depends on BC-2.03.005 for the happy-path recipe construction
- [BC-2.08.002] — composes with: session-state.json written here is read during re-discovery
- [BC-2.08.003] — composes with: session spawned here is later killed via kill_session()
- [BC-2.08.006] — composes with: hook auto-injection in the SpawnRecipe is validated by this spawn

## Architecture Anchors

- `architecture/SS-session-manager.md#sessionmanager` — struct definition, public API (`spawn_session(opts: SpawnOptions)` — Model A), session lifecycle state machine; IPC handler generates UUID + sends SpawnAck BEFORE spawn_session() (F-P41-IMP-001)
- `architecture/SS-session-manager.md#session-statejson-schema` — sidecar schema specification (schema_version 3)
- `architecture/SS-session-manager.md#error-handling-sessionerror-servertoclient-error-mapping` — `session_error_to_code()` spawn-path arms for EngineError bridge
- `architecture/SS-engine-module-v2-delta.md#spawnrecipe-and-spawnoptions-types` — SpawnOptions as wire type; SpawnRecipe as daemon-internal (I27-001 Model A)
- `architecture/SS-ipc.md#servertoClientspawnack` — SpawnAck variant definition; UUID-generation locus in IPC handler (F-P41-IMP-001)
- `architecture/adr/ADR-0009-native-session-host-process-model.md` — process model decision

## Story Anchor

S-033 — Implement SessionManager::spawn_session() with SessionHostSpawner

## VP Anchors

VP-TBD — Session spawn integration tests (filled after VP creation)

## §Trace v1.5.1

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-033** (2026-06-15):
- Story Anchor filled from Phase-2 Burst A story decomposition. No behavioral content changed.

## §Trace v1.5.0

**F-P46-IMP-001 — §Architecture Anchors: version parentheticals stripped; version-less navigational convention adopted** (2026-06-14):

- **Change:** Removed ` (vX.Y.Z)` parentheticals from all 5 entries in §Architecture Anchors (SS-session-manager.md ×3, SS-engine-module-v2-delta.md, SS-ipc.md). No normative content changed.
- **Rationale:** Version pins in navigational anchors duplicate the authoritative §Architecture Source Traceability-table row (POL-11-enforced) and are invisible to POL-11's ID↔version adjacency regex when in the `` `path#anchor` (vX.Y.Z) `` form. Eliminating the duplication removes the drift class entirely. Authoritative version citations remain in the §Architecture Source table unchanged: SS-session-manager.md v2.4.0, SS-engine-module-v2-delta.md v1.5.0, SS-ipc.md v1.22.0.
- **Bump disposition:** Errata-no-bump (navigational-anchor-only change, precedent D-275). BC version stays at v1.5.0.

**F-P41-IMP-001 — UUID-generation locus corrected to IPC handler; SpawnAck wiring; arch-source pins to SS-session-manager v2.4.0 + SS-ipc v1.22.0** (2026-06-14):

- **PC-1 UUID-locus (normative rewrite):** The old wording "generated via
  `uuid::Uuid::new_v4().to_string()` inside `spawn_session()`" was incorrect. The canonical
  mechanism (F-P41-IMP-001, SS-session-manager.md v2.4.0 §IPC handler skeleton): the daemon
  IPC handler generates the UUID in the `ClientToServer::SpawnSession` match arm BEFORE calling
  `spawn_session()`. `spawn_session()` receives `opts.session_id` already populated via
  `opts.with_daemon_fields(session_id, hooks_path)`; it does NOT generate the UUID itself.
  PC-1 now states this correctly.

- **Arch-source pin:** SS-session-manager.md v2.4.0 → v2.3.0 (F-P41-IMP-001 IPC handler
  change — UUID generation + SpawnAck step added); SS-ipc.md v1.22.0 → v1.21.0 (new
  `ServerToClient::SpawnAck { session_id }` variant). Architecture Anchors updated to match.

- **No invariant change:** The uniqueness invariant (Invariant 1) and all other behavioral
  content are unchanged. The UUID is still UUID v4; collision detection still required.
  Only the generation locus is corrected: handler arm, not spawn_session() body.

- SE-16d monotonicity: v1.5.0 timestamp 2026-06-14 > v1.4.3 timestamp 2026-06-13. PASS.

## §Trace v1.4.3

**Arch-source pin v1.9.0→v1.9.1** (2026-06-13 / D-277):
- Arch-source pin: SS-daemon-wiring-v2-delta.md v1.9.0 → v1.9.1 (all active citations in
  PC-5 and Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.4.2

**I34-001 — Stale anchor version labels updated; arch-source pin v1.4.0→v1.4.1** (2026-06-13 / D-276):

- **I34-001 (Architecture Anchors stale version labels):**
  The Architecture Anchors section carried stale parenthetical version labels that diverged
  from the Architecture-Source row in this same file:
  - `architecture/SS-session-manager.md#sessionmanager` `(v2.0.0)` → `(v2.2.1)` (×3 anchors)
  - `architecture/SS-engine-module-v2-delta.md#spawnrecipe-and-spawnoptions-types` `(v1.2.0)` → `(v1.4.1)`
  The Architecture-Source row already cited SS-session-manager.md v2.4.0 and now cites
  SS-engine-module-v2-delta.md v1.5.0 (see arch-source pin below). The anchor labels now
  match the Architecture-Source row — single source of truth for version labels.

- **Arch-source pin:** SS-engine-module-v2-delta.md v1.4.0 → v1.4.1 (architect C34-001 bump).
  The v1.4.1 bump corrected the null-byte detection mechanism in spawn_recipe(); this BC's
  spawn-sequence description is unaffected behaviorally (the two-pronged check is internal
  to spawn_recipe() and produces the same EngineError::InvalidPath outcome).

- No behavioral content changed. Patch bump only.

## §Trace v1.4.0

**I27-001 (Model A) — spawn_session() receives SpawnOptions; spawn_recipe() called daemon-side** (2026-06-13):

Model A adjudication: `ClientToServer::SpawnSession` now carries `opts: SpawnOptions` (wire
type) instead of a pre-built `SpawnRecipe`. `SessionManager::spawn_session()` signature
changes from `spawn_session(recipe: SpawnRecipe, harness_id, profile_id, ...)` to
`spawn_session(opts: SpawnOptions)`. The `SpawnRecipe` is daemon-internal — built inside
`spawn_session()` as its first step via `engine_module.spawn_recipe(&opts)?`.

**Changes in this version:**

- **Description:** Rewritten. The old description incorrectly stated `spawn_session()` "receives
  a `SpawnRecipe` (from `ClaudeCodeModule::spawn_recipe()`), a `harness_id`, and a `profile_id`."
  Under Model A, `spawn_session()` receives `opts: SpawnOptions`; `spawn_recipe()` is called
  daemon-side internally.

- **Precondition 2 (normative rewrite):** Removed "recipe is a valid SpawnRecipe (binary exists,
  args valid, cwd valid)" — this was a pre-built-recipe precondition from the old Model B
  (TUI builds recipe). Under Model A, the TUI sends `SpawnOptions` (not a pre-built recipe)
  and the binary-existence / arg-validity check occurs daemon-side as the first step of
  `spawn_session()` via `engine_module.spawn_recipe(&opts)?`. New Precondition 2 states the
  `SpawnOptions` field population: TUI populates `project_root`, `worktree_root`, `harness_id`,
  `profile_id`, `ccr_base_url`; daemon IPC handler fills `session_id` and `hooks_settings_path`
  on receipt.

- **EC-150 (normative rewrite):** Old text: "SessionHostSpawner::spawn() fails (e.g., binary
  not found at recipe.binary)" → `SessionError::SpawnFailed` / `"spawn_failed"`. This was wrong
  under Model A: binary-not-found is now produced by `spawn_recipe()`'s `which::which` call
  INSIDE `spawn_session()` → `EngineError::BinaryNotFound` → (via `#[from]`) `SessionError::EngineError`
  → IPC code `"binary_not_found"` (per `session_error_to_code()` spawn-path arm). `SessionError::SpawnFailed`
  is for OS-level spawn failures AFTER the binary is located. EC-150 now correctly aligns with
  BC-2.03.007 PC-3.

- **Canonical Test Vectors:** "Valid `SpawnRecipe`" input renamed to "Valid `SpawnOptions`";
  error-path vectors extended to cover `BinaryNotFound` (EC-150), `InvalidPath` (per BC-2.03.007
  PC-7), and `SpawnFailed` (OS spawn after binary located) as distinct rows.

- **Related BCs BC-2.03.005:** Description updated to clarify `spawn_recipe()` is called
  daemon-side INSIDE `spawn_session()` (not pre-built by the TUI caller).

- **Architecture Anchors:** Extended to cite SS-engine-module-v2-delta.md v1.3.0 §SpawnOptions/SpawnRecipe
  types and SS-session-manager.md v2.1.0 §session_error_to_code.

- **Architecture Source:** Updated all version citations from v1.9.0 to v2.0.0 (SS-session-manager)
  and v1.7.0 to v1.8.0 (SS-daemon-wiring-v2-delta); added SS-engine-module-v2-delta.md v1.3.0
  and SS-ipc.md v1.20.0.

- No changes to Invariants, PC-1 through PC-5, EC-151, EC-152, EC-153, or Verification Properties
  (spawner call, sidecar schema, UUID invariants, publication ordering remain unchanged).

## §Trace v1.4.1

**ANCHOR-LINT-TOOL D-275 — Dead anchor citation corrected: `servertocommand` typo** (2026-06-13T00:00:00Z):
- Architecture Anchors line: `#error-handling-sessionerror-servertocommand-error-mapping` →
  `#error-handling-sessionerror-servertoclient-error-mapping`.
- Cause: typo introduced in v1.4.0 burst — "servertocommand" vs. the actual heading
  "SessionError → ServerToClient::Error mapping". Architect added the explicit `<a id>` anchor
  to SS-session-manager.md in the same D-275 pass; this corrects the BC side of the citation.
- No behavioral content changed; version bumped as patch 1.4.0→1.4.1.
- SE-16d monotonicity: v1.4.1 timestamp 2026-06-13T00:00:00Z > v1.4.0 timestamp 2026-06-03T23:30:00Z. PASS.

## §Trace v1.3.1

**I17-001 + S17-002 — Pin-symmetry fix: ADR-0009 in Architecture Source; unpinned SS-session-manager.md ref pinned; §-anchor corrected to exact heading** (2026-06-04):
- Architecture Source: two violations fixed:
  1. Second `SS-session-manager.md §session-state.json schema` (no version pin) → `SS-session-manager.md v1.8.1 §session-state.json schema`. Pin-symmetry rule: all refs in a multi-doc cell must carry explicit version pins. <!-- version-pin-historical: §Trace I17-001 fix record; v1.8.1 is SS-session-manager at Pass-17 fix time -->
  2. `ADR-0009 §native-detached-session-host-process-model` → `ADR-0009 v1.0.2 §Decision`. Fixes I17-001 (unpinned ADR) and S17-002 (loose paraphrase §-anchor; ADR-0009's decision section heading is `## Decision` per ADR-0009 file line 92).
- No behavioral content changed; version bumped as patch 1.3.0→1.3.1.

## §Trace v1.3.0

**Adversarial Pass 3 fixes — C3-002 (schema_version 3) + C3-001 (SessionStateChanged PC-5)** (2026-06-03):
- C3-002: PC-3 `schema_version` corrected from `1` to `3` (current canonical schema per
  SS-session-manager.md v1.5.0 §session-state.json schema). Schema v1 = no cwd field,
  v2 = adds cwd, v3 = adds kill_deadline_unix_ms. A newly spawned sidecar always writes v3
  (the current schema). The previous PC-3 said `schema_version: 1` while simultaneously
  listing a `cwd` field — these are internally inconsistent (cwd was added in v2). This is
  now corrected: v3 is the written version, and `kill_deadline_unix_ms: null` is explicitly
  enumerated in PC-3.
- C3-001: PC-5 updated from `SessionListUpdate` only to `SessionStateChanged{Launching}` THEN
  `SessionListUpdate` (ordered pair per BC-2.08.008 Invariant 4). Both are published under the
  same mutex lock in the same broker tick. Architecture Source updated to
  SS-session-manager.md v1.5.0 and SS-daemon-wiring-v2-delta.md v1.3.1.

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

## §Trace v1.5.3

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.5.3 timestamp >= v1.5.2. PASS.
