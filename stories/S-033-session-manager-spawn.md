---
document_type: story
level: L4
story_id: S-033
epic_id: EPIC-08
version: "1.8"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
points: 8
wave: 8
tdd_mode: strict
priority: P1
depends_on: [S-014, S-015, S-017, S-021]
blocks: [S-034, S-035, S-036, S-037, S-038, S-044, S-045, S-047, S-048]
target_module: monocle-runtime
subsystems: [SS-08]
behavioral_contracts: [BC-2.03.008, BC-2.08.001, BC-2.08.008]
verification_properties: []
estimated_days: 4
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-03/BC-2.03.008.md, version: "1.0.8"}
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.001.md, version: "1.5.3"}
  - {path: .factory/specs/behavioral-contracts/ss-08/BC-2.08.008.md, version: "1.3.4"}
  - {path: .factory/specs/architecture/SS-engine-module-v2-delta.md, version: "1.6.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.7.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.08.001 (spawn_session with SessionHostSpawner; SessionEntry; sidecar; SpawnAck) and BC-2.08.008 (SessionStateChanged{Launching} broadcast on spawn)"
# BC status: non-empty; status draft pending Phase-2 adversarial convergence gate (authoritative versions in inputs: frontmatter)
---

# S-033: SessionManager::spawn_session — SessionHostSpawner, SessionEntry, Sidecar, SpawnAck, and SessionStateChanged{Launching}

## Narrative

As the monocle daemon, I want `SessionManager::spawn_session()` to receive a `SpawnOptions`
value, call `engine_module.spawn_recipe(&opts)?` as its first step to obtain a `SpawnRecipe`,
invoke `SessionHostSpawner::spawn()` to start the `monocle-session-host` process, write a
`session-state.json` sidecar atomically, insert a `SessionEntry` with `state: Launching` into
the registry, and return `Ok(session_id)` — all within 2 seconds — so that a spawned session
is immediately observable in the registry and TUI clients receive `SessionStateChanged{Launching}`
followed by `SessionListUpdate` without any silent gaps.

## Acceptance Criteria

### AC-001 (traces to BC-2.08.001 postcondition 1 — spawner called within 2s; UUID generated in IPC handler)

When the daemon's IPC handler processes `ClientToServer::SpawnSession { opts }`, it:
- Generates a UUID v4 via `uuid::Uuid::new_v4().to_string()` and sends `ServerToClient::SpawnAck { session_id: session_id.clone() }` to the requesting client BEFORE calling `spawn_session()`.
- Fills daemon-owned fields via `opts.with_daemon_fields(session_id, hooks_settings_path)`.
- Calls `spawn_session(opts)` with the completed `SpawnOptions` (session_id already set).
- `SessionHostSpawner::spawn()` is called within 2 seconds of `spawn_session()` invocation.
- `spawn_session()` calls `engine_module.spawn_recipe(&opts)?` as its FIRST step before touching the spawner.

### AC-002 (traces to BC-2.08.001 postcondition 2 — SessionEntry in registry with state Launching)

After a successful `spawn_session()` call:
- A `SessionEntry` exists in `SessionManager.sessions` keyed by the UUID string `session_id`.
- `SessionEntry.state` is `SessionState::Launching`.
- `SessionEntry.host_conn` is `None` (post-spawn monitor has not yet connected).
- `SessionEntry.degraded` is `false`.
- `SessionEntry.kill_deadline` is `None`.
- `SessionEntry.project_root`, `cwd`, `harness_id`, `profile_id`, `started_at` are populated from `SpawnOptions`.

### AC-003 (traces to BC-2.08.001 postcondition 3 — session-state.json written atomically with schema_version 3)

`session-state.json` is written atomically via `tempfile::persist` to
`<runtime_dir>/session-<session_id>.json` with the schema v3 fields:
- `schema_version: 3`
- `session_id`: the UUID string
- `pid`: `SpawnedHostHandle.pid`
- `socket_path`: `<runtime_dir>/session-<session_id>.sock`
- `child_pid`: (initial value — may be 0 or omitted; session-host populates it at startup step 8)
- `state: "Launching"`
- `project_root`: `opts.project_root.to_string_lossy()`
- `cwd`: `opts.worktree_root.to_string_lossy()`
- `harness_id`, `profile_id`, `started_at` (ISO-8601 UTC), `display_name` (defaults to `"<harness_id> — <project_root_basename>"`)
- `pty_rows: 24`, `pty_cols: 80`
- `kill_deadline_unix_ms: null`

No `std::fs::write` is used — only `tempfile::persist`.

### AC-004 (traces to BC-2.08.001 postcondition 4 — spawn_session returns Ok(session_id))

`spawn_session()` returns `Ok(session_id)` (the UUID string) and does NOT wait for the
session-host to reach `SessionState::Running`. The `Running` transition happens asynchronously
via the post-spawn monitor.

### AC-005 (traces to BC-2.08.001 postcondition 5 — SessionStateChanged{Launching} before SessionListUpdate)

After the `SessionEntry` is inserted and the sidecar is written:
- `ServerToClient::SessionStateChanged { session_id, new_state: Launching }` is published to the broker BEFORE `ServerToClient::SessionListUpdate`.
- Both publications happen under the `SessionManager` mutex (per BC-2.08.008 Invariant 4).
- If a connected TUI client's per-client buffer is full and the ordered pair splits (SessionStateChanged delivered but SessionListUpdate dropped), the client is IMMEDIATELY disconnected (per BC-2.08.008 PC-3 split rule).

### AC-006 (traces to BC-2.08.001 invariant 1 — session_id uniqueness; UUID collision handling)

The `SessionEntry` insertion MUST check for collision. If a collision is detected (UUID already in registry), the implementation MUST either retry (generate a new UUID) or return `Err(SessionError::SessionIdCollision { session_id })`. Auto-retry is one attempt; second collision returns the error.

### AC-007 (traces to BC-2.08.001 invariant 2 — atomic sidecar write)

`session-state.json` is written via `tempfile::persist` (atomic rename). Naked `std::fs::write` is forbidden. Test must verify that a concurrent reader of the sidecar path never sees a partial write.

### AC-008 (traces to BC-2.08.001 edge case EC-150 — EngineError::BinaryNotFound propagation)

When `spawn_recipe()` returns `Err(EngineError::BinaryNotFound)`, `spawn_session()` returns
`Err(SessionError::EngineError(BinaryNotFound(...)))`. No OS process is spawned; no registry entry is created; no sidecar is written. The IPC handler maps this to `ServerToClient::Error { code: "binary_not_found", ... }` via `session_error_to_code(IpcOp::Spawn, &e)`.

### AC-009 (traces to BC-2.08.001 edge case EC-151 — orphan-kill on sidecar write failure)

When the spawner succeeds (`spawn()` returns `Ok(handle)`) but the sidecar write fails (injected `tempfile::persist` failure), `spawn_session()`:
- Sends SIGTERM to `SpawnedHostHandle.pid` directly (per §Pre-socket-bind orphan kill).
- If the process has not exited after 2 seconds, sends SIGKILL.
- Returns `Err(SessionError::SidecarWriteFailed { ... })`.
- No `SessionEntry` is added to the registry.

### AC-009b (traces to BC-2.08.001 edge case — SpawnFailed: spawner.spawn() OS error → wire code "spawn_failed")

When `spawn_recipe()` succeeds (returns `Ok(SpawnRecipe)`) but the subsequent
`SessionHostSpawner::spawn(&recipe)` returns `Err(...)` (OS-level process spawn failure,
distinct from EngineError before the spawner is called):
- `spawn_session()` returns `Err(SessionError::SpawnFailed { reason: e.to_string() })`.
- No sidecar is written; no `SessionEntry` is inserted into the registry.
- No orphan-kill is needed (the OS did not successfully create the process).
- The IPC handler maps `SessionError::SpawnFailed` to `ServerToClient::Error { code: "spawn_failed", message: reason }` via `session_error_to_code(IpcOp::Spawn, &e)`.
- Test: `test_BC_2_08_001_spawn_failed_os_error_returns_spawn_failed_code` — `MockSessionHostSpawner`
  configured to return `Err(...)` on `spawn()`; assert `SessionError::SpawnFailed` from
  `spawn_session()`; assert IPC code `"spawn_failed"`; assert no `SessionEntry` in registry.

### AC-009c (traces to BC-2.03.008 postcondition 1 — default spawn_recipe returns UnsupportedOperation)

This story defines the `EngineModule::spawn_recipe()` trait method with a DEFAULT impl
in `monocle-core/src/engine.rs`. When any `EngineModule` implementation that does NOT
override `spawn_recipe()` is called (a no-override test double):
- Returns `Err(EngineError::UnsupportedOperation("spawn_recipe"))` immediately.
- No I/O, no filesystem access performed.
- The `EngineError` enum is defined in `monocle-core/src/engine.rs` with variants:
  `BinaryNotFound(String)`, `InvalidPath(String)`, `UnsupportedOperation(&'static str)`.
- `EngineError` carries `#[non_exhaustive]`; the inner match in `session_error_to_code()`
  MUST have `_ => "invalid_request"` fallback because `EngineError` is cross-crate and
  non-exhaustive (the compiler requires it).
- Unit test `test_BC_2_03_008_default_spawn_recipe_unsupported_operation`: a no-override
  `EngineModule` impl returns `Err(EngineError::UnsupportedOperation("spawn_recipe"))`.

### AC-009d (traces to BC-2.03.008 postcondition 3 — UnsupportedOperation maps to "spawn_unsupported" wire code)

When `spawn_session()` calls `engine_module.spawn_recipe(&opts)` and the engine returns
`Err(EngineError::UnsupportedOperation("spawn_recipe"))` (the default-impl path for a
non-overriding engine), the IPC handler MUST surface this as
`ServerToClient::Error { code: "spawn_unsupported", ... }`, NOT as `"invalid_request"`.

This requires `session_error_to_code(IpcOp::Spawn, &e)` to resolve as follows:
- Outer match on `SessionError::EngineError(inner)` → enters inner `EngineError` match.
- Inner match on `EngineError::UnsupportedOperation(_)` → returns `"spawn_unsupported"`.
- Inner `_ =>` fallback returns `"invalid_request"` ONLY for future non-exhaustive variants.

**Canonical EC-112 integration test vector:**
`session_error_to_code(IpcOp::Spawn, &SessionError::EngineError(EngineError::UnsupportedOperation("spawn_recipe"))) == "spawn_unsupported"`

- Unit test `test_BC_2_03_008_EC_112_unsupported_operation_maps_to_spawn_unsupported`: assert
  `session_error_to_code(IpcOp::Spawn, &SessionError::EngineError(UnsupportedOperation("spawn_recipe"))) == "spawn_unsupported"`.
- Integration test: `MockEngineModule` with no `spawn_recipe()` override → `spawn_session(opts)` →
  IPC handler sends `ServerToClient::Error { code: "spawn_unsupported" }` to the requesting client.

**Relationship to AC-009c:** AC-009c covers "the default impl returns `Err(UnsupportedOperation)`";
AC-009d covers "that `UnsupportedOperation` propagates through `session_error_to_code()` to the
wire code `"spawn_unsupported"` — not collapsed to `"invalid_request"`." These are complementary:
AC-009c validates the trait method; AC-009d validates the error mapping pathway.

### AC-010 (traces to BC-2.08.008 postcondition 1 — no silent state transitions)

`SessionStateChanged` is emitted for EVERY `SessionState` transition without exception.
For the spawn path: `Launching` state is broadcast immediately after the entry is created.
Re-discovery registration of an unchanged persisted state (e.g., Detached re-discovery)
is NOT a transition and MUST NOT emit `SessionStateChanged`.

### AC-011 (traces to BC-2.08.008 postcondition 2 — broadcast to all TUI clients)

`SessionStateChanged` is dispatched to ALL connected TUI clients via their per-client isolated
send buffers (capacity 64 per SS-ipc.md §TUI IPC Read Loop Pattern). If no TUI clients are
connected, the broker discards the message with no error.

### AC-012 (traces to BC-2.08.008 postcondition 5 — SpawnAck before SessionStateChanged{Launching})

`ServerToClient::SpawnAck { session_id }` is delivered to the requesting TUI client BEFORE
any broker-published `SessionStateChanged { Launching }` for the same session. This is
guaranteed by TWO properties:
1. **Causal step ordering**: `SpawnAck` is sent in IPC-handler step 2 (UUID generation is step 1; `spawn_session()` is called at step 3; the broker emits `SessionStateChanged { Launching }` at step 5).
2. **Per-client FIFO**: the requesting client's per-client `mpsc` channel delivers messages in send order.

## Tasks

- [ ] Define `SessionState` enum in `crates/monocle-ipc/src/lib.rs` with exactly the canonical 5 variants: `Launching`, `Running`, `Detached`, `Terminating`, `Terminated`. (`Created` and `Killed` are retired variants — do NOT use them.) `SessionState` MUST live in `monocle-ipc`, not in `monocle-runtime`, because `SessionStateChanged { new_state: SessionState }` and `SessionSnapshot { state: SessionState }` are wire types in `monocle-ipc`. `monocle-ipc` MUST NOT depend on `monocle-runtime`; placing `SessionState` in `monocle-runtime` would create a circular dependency. (Consistent with S-048 which imports `SessionState` from `monocle_ipc`.) Add if not present from S-021; do not duplicate.
- [ ] Create `crates/monocle-runtime/src/session_manager/mod.rs` with `SessionManager`, `SessionEntry`, `SessionHostConnection` structs and enums per SS-session-manager.md §SessionManager §SessionEntry §SessionHostConnection §Session lifecycle state machine. `SessionState` is imported from `monocle_ipc::SessionState` (defined in monocle-ipc, see task above) — do NOT redefine it in monocle-runtime.
- [ ] Implement `SessionHostSpawner` trait (`spawn()` async fn) and `SpawnedHostHandle` struct.
- [ ] Implement `RealSessionHostSpawner`: invokes `monocle-session-host` binary via `std::process::Command::spawn()` — **NO `pre_exec`** — and passes `--session-id`, `--runtime-dir`, `--binary`, `--args`, `--env`, `--cwd` CLI args. `pre_exec` is `unsafe fn`; `monocle-runtime` is `#![forbid(unsafe_code)]`. The session-host binary calls `nix::unistd::setsid()` itself at startup step 2 (see SS-session-manager.md §Ruling C).
- [ ] Implement `MockSessionHostSpawner`: in-memory fake that returns a configurable `Ok(SpawnedHostHandle)` or error; used in all unit tests.
- [ ] Implement `session_error_to_code(op: IpcOp, e: &SessionError) -> &'static str` with full exhaustive outer match on `SessionError` and, for the `SessionError::EngineError(inner)` arm, an inner match on `EngineError` that MUST include both arms in order: `EngineError::UnsupportedOperation(_) => "spawn_unsupported"` FIRST, then `_ => "invalid_request"` as the mandatory non-exhaustive fallback (per SS-session-manager.md §session_error_to_code and BC-2.03.008 PC-3). The `UnsupportedOperation` arm MUST NOT be absent or collapsed into the catch-all — that would be the F-P44-IMP-001 regression.
- [ ] Implement `monocle-session-host/src/main.rs` minimum-viable binary: parse CLI args; call `setsid()`; open PTY pair; build CommandBuilder with env inheritance (I2-006); spawn harness child; init `vt100::Parser` stub; bind UDS at `<runtime_dir>/session-<session_id>.sock`; write sidecar via `tempfile::persist` with `child_pid: Some(child.process_id())`; send `HostToDaemon::StateChanged { new_state: Running, degraded_env: None }` over the per-session UDS; enter minimal event loop handling `DaemonToHost::Kill`. PTY output streaming, scrollback dump, keyboard forwarding, and resize are deferred (see SS-session-manager.md §Architecture Ruling A → S-039/S-035/S-047/S-042/S-044).
- [ ] Define `SessionSidecarV3` struct in `crates/monocle-ipc/src/lib.rs` (per SS-session-manager.md §Ruling B) with the schema-v3 shape. Both `monocle-runtime` and `monocle-session-host` import it via `monocle_ipc::SessionSidecarV3`. Daemon writes it with `child_pid: None` at spawn; session-host overwrites with `child_pid: Some(pid)` at startup step 8.
- [ ] Implement `spawn_session(opts: SpawnOptions)`: call `spawn_recipe()` first, then `spawner.spawn()`, then write sidecar via `tempfile::persist` using `monocle_ipc::SessionSidecarV3` as the serialization type (NOT an ad-hoc struct; `child_pid: None` at daemon's initial write), then insert `SessionEntry{state: Launching, host_conn: None}`, then spawn post-spawn monitor `tokio::spawn`, then publish `SessionStateChanged{Launching}` + `SessionListUpdate` under mutex.
- [ ] Implement post-spawn monitor as a `tokio::spawn` background task: polls UDS socket connectable (20ms backoff, 30s total timeout), verifies SO_PEERCRED, stores `host_conn: Some(SessionHostConnection { writer, proxy_task: None })`, receives `StateChanged` messages, on `StateChanged{Running}` starts PTY proxy task and transitions to Running state (emits `SessionStateChanged{Running}` + `SessionListUpdate`).
- [ ] Implement IPC handler skeleton: generate UUID → send `SpawnAck` → call `opts.with_daemon_fields()` → call `spawn_session()` → on error send `ServerToClient::Error`.
- [ ] Add `spawn_recipe(&self, opts: &SpawnOptions) -> Result<SpawnRecipe, EngineError>` to the `EngineModule` trait in `monocle-core/src/engine.rs` with a default impl returning `Err(EngineError::UnsupportedOperation("spawn_recipe"))`. (BC-2.03.008)
- [ ] Define `EngineError` enum in `monocle-core/src/engine.rs`: `BinaryNotFound(String)`, `InvalidPath(String)`, `UnsupportedOperation(&'static str)`; derive `thiserror::Error`, `Debug`; apply `#[non_exhaustive]`. (BC-2.03.008)
- [ ] Add `SpawnOptions` and `SpawnRecipe` type declarations (or re-exports per SS-engine-module-v2-delta.md) to `monocle-core/src/engine.rs` as required by the trait method signature. (BC-2.03.008)
- [ ] Write unit test `test_BC_2_03_008_default_spawn_recipe_unsupported_operation`: a no-override `EngineModule` impl (test double) calls the default `spawn_recipe()` and asserts `Err(EngineError::UnsupportedOperation("spawn_recipe"))`. (BC-2.03.008 PC-1)
- [ ] Write unit test `test_BC_2_03_008_EC_112_unsupported_operation_maps_to_spawn_unsupported`: assert `session_error_to_code(IpcOp::Spawn, &SessionError::EngineError(EngineError::UnsupportedOperation("spawn_recipe"))) == "spawn_unsupported"`. Verify `"invalid_request"` is NOT returned. (BC-2.03.008 PC-3 / EC-112)
- [ ] Add `SessionError` enum with all 9 variants per SS-session-manager.md §SessionError taxonomy.
- [ ] Add `From<EngineError> for SessionError` via `#[from]` on `EngineError(#[from] monocle_core::engine::EngineError)`.
- [ ] Implement orphan-kill logic: on sidecar write failure, SIGTERM → 2s wait → SIGKILL to `SpawnedHostHandle.pid`.
- [ ] Write unit test `test_BC_2_08_001_spawn_session_entry_created_within_2s`: uses `MockSessionHostSpawner`, asserts `Ok(session_id)`, `SessionEntry{Launching}` in registry, sidecar written.
- [ ] Write unit test `test_BC_2_08_001_binary_not_found_propagation`: `MockSessionHostSpawner` not reached; EC-150 verified.
- [ ] Write unit test `test_BC_2_08_001_sidecar_write_fail_orphan_kill`: injected sidecar failure; PID SIGTERMed; no registry entry.
- [ ] Write unit test `test_BC_2_08_008_session_state_changed_before_session_list_update_on_spawn`: verifies FIFO ordering in per-client channel; `SessionStateChanged{Launching}` arrives before `SessionListUpdate`.
- [ ] Write unit test `test_BC_2_08_008_spawn_ack_before_state_changed_launching`: verifies causal ordering — `SpawnAck` arrives before `SessionStateChanged{Launching}` on the requesting client's channel.

## Previous Story Intelligence

- **S-017** (daemon-start-sequence): Implemented `daemon_start_sequence()` up to step 8; step 8b (`rediscover_sessions`) is the hook for S-036. `DaemonState` struct exists.
- **S-021** (UDS server IPC types): `ClientToServer`, `ServerToClient`, `SessionStateChanged`, `SessionListUpdate` wire types exist in `monocle-ipc`. `Broker<Event>` and per-client `mpsc::channel(64)` pattern established.
- **S-014** (EngineModule trait, done Wave 2): `EngineModule` trait exists in `monocle-core/src/engine.rs`. `spawn_recipe()` does NOT yet exist — it is a v1A-pivot addition introduced by THIS story (S-033). `SpawnRecipe` and `SpawnOptions` types must be verified against SS-engine-module-v2-delta.md v1.6.0 §SpawnOptions and §SpawnRecipe before authoring them.
- **S-015** (ClaudeCodeModule, done Wave 3): `ClaudeCodeModule` struct and its existing `EngineModule` impl exist. `ClaudeCodeModule::spawn_recipe()` does NOT yet exist — the concrete override is a v1A-pivot addition introduced by S-045 (Wave 8, after this story). THIS story (S-033) adds only the trait method + default impl + `EngineError` enum (BC-2.03.008); S-045 later adds the `ClaudeCodeModule` override (BC-2.03.005/006/007).
- `DaemonState.session_manager: Arc<Mutex<SessionManager>>` must be wired by this story; confirm `DaemonState` struct in `monocle-runtime/src/lib.rs` does not already have a session_manager field.

## Architecture Compliance Rules

- `SessionManager` lives at `crates/monocle-runtime/src/session_manager/mod.rs` — NOT a separate crate (SS-session-manager.md §Location and crate).
- `SessionState` is defined in `crates/monocle-ipc/src/lib.rs`, NOT in `monocle-runtime`. The wire types `SessionStateChanged { new_state: SessionState }` and `SessionSnapshot { state: SessionState }` live in `monocle-ipc`; placing `SessionState` in `monocle-runtime` would force `monocle-ipc` to depend on `monocle-runtime`, creating a circular dependency. `monocle-runtime` imports `SessionState` as `monocle_ipc::SessionState`. The canonical 5 variants are: `Launching`, `Running`, `Detached`, `Terminating`, `Terminated`. (`Created` and `Killed` are RETIRED — do not use them.)
- `session_id` is `String` everywhere — UUID v4 value, never a typed `uuid::Uuid` at IPC/registry boundaries (SS-session-manager.md §session_id type ruling).
- All sidecar writes use `tempfile::persist`. No `std::fs::write` (CLAUDE.md conventions).
- `SessionError` does NOT carry `#[non_exhaustive]`; the outer match in `session_error_to_code()` must be compiler-enforced exhaustive.
- `EngineError` carries `#[non_exhaustive]`; the inner match in `session_error_to_code()` MUST include `EngineError::UnsupportedOperation(_) => "spawn_unsupported"` BEFORE the mandatory `_ => "invalid_request"` catch-all (per BC-2.03.008 PC-3 and SS-session-manager.md §session_error_to_code). The `UnsupportedOperation` arm MUST appear explicitly — omitting it and relying on the `_` fallback is the F-P44-IMP-001 regression pattern (collapses `UnsupportedOperation` to `"invalid_request"` instead of `"spawn_unsupported"`). The `_ =>` fallback is still required because `EngineError` is cross-crate and `#[non_exhaustive]`.
- `SessionStateChanged` MUST be published BEFORE `SessionListUpdate` for the same transition. Both `.try_send()` calls under the same mutex hold.
- Per-client channel capacity is 64 (`mpsc::channel(64)`). If `SessionStateChanged` succeeds but `SessionListUpdate` fails (buffer full), client is IMMEDIATELY disconnected (BC-2.08.008 PC-3).
- Forbidden dependency: `monocle-runtime` MUST NOT depend on `monocle-tui`. The `monocle-tui` crate is a TUI consumer; `monocle-runtime` is the producer.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tokio` | `=1.52` (exact) | Async runtime, `tokio::spawn`, `mpsc::channel`, `time::sleep` | SS-deps-pin-manifest.md §Exact-pinned |
| `serde` | `"1"` + features `["derive"]` | `Serialize`/`Deserialize` for `SessionState`, `session-state.json` | SS-deps-pin-manifest.md |
| `serde_json` | `=1.0.149` (exact) | JSON sidecar serialization | SS-deps-pin-manifest.md §Exact-pinned |
| `tempfile` | `"3"` | Atomic sidecar writes via `tempfile::persist` | SS-deps-pin-manifest.md |
| `chrono` | `"0.4"` + features `["serde"]` | `started_at: chrono::DateTime<chrono::Utc>` in sidecar | SS-deps-pin-manifest.md |
| `thiserror` | `"2"` | `SessionError` enum derivation | SS-deps-pin-manifest.md |
| `nix` | `"0.30"` | `nix::unistd::setsid()` in `monocle-session-host` binary (startup step 2 — session-host calls it, not `RealSessionHostSpawner`; see §Ruling C); `nix::sys::signal::kill()` for liveness probe and orphan-kill in `monocle-runtime` | SS-deps-pin-manifest.md |
| `uuid` | `"1"` + features `["v4", "serde"]` | UUID v4 generation in IPC handler | workspace `Cargo.toml` line 66 |
| `portable-pty` | `"0.9"` (caret) | PTY pair creation in `monocle-session-host` (not in `monocle-runtime` directly) | SS-deps-pin-manifest-v2-delta.md |

**Note:** `monocle-session-host` binary crate is created in this story as `crates/monocle-session-host/` — it uses `portable-pty "0.9"` and `vt100 "0.16"` but these are dependencies of the session-host binary, not of `monocle-runtime`.

## File Structure Requirements

Files to CREATE:

| File | Purpose |
|------|---------|
| `crates/monocle-runtime/src/session_manager/mod.rs` | `SessionManager`, `SessionEntry`, `SessionHostConnection`, `SessionHostSpawner` trait, `RealSessionHostSpawner`, `MockSessionHostSpawner`, `session_error_to_code()`, `SpawnedHostHandle`, `IpcOp` — `SessionState` is imported from `monocle_ipc::SessionState` (NOT defined here) |
| `crates/monocle-session-host/Cargo.toml` | New binary crate; deps: `portable-pty = "0.9"`, `vt100 = "0.16"`, `tokio = "=1.52"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "=1.0.149"`, `nix = "0.30"`, `tempfile = "3"`, `monocle-ipc = { path = "../monocle-ipc" }` |
| `crates/monocle-session-host/src/main.rs` | Session-host binary: parse CLI args, `setsid()`, open PTY, build `CommandBuilder`, spawn harness child, bind UDS, write sidecar, enter main event loop |

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-runtime/src/lib.rs` | Add `pub mod session_manager;`; add `session_manager: Arc<Mutex<SessionManager>>` to `DaemonState` |
| `crates/monocle-runtime/src/ipc_handler.rs` (or equivalent IPC handler file) | Add `ClientToServer::SpawnSession` arm: UUID gen → `SpawnAck` → `with_daemon_fields` → `spawn_session()` → on error `ServerToClient::Error` |
| `Cargo.toml` (workspace root) | Add `crates/monocle-session-host` to `[workspace.members]` |
| `crates/monocle-ipc/src/lib.rs` | (1) Define `SessionState` enum with the canonical 5 variants: `Launching`, `Running`, `Detached`, `Terminating`, `Terminated` — add if absent from S-021, do not duplicate. This is the authoritative location for `SessionState` (wire type used by `SessionStateChanged` and `SessionSnapshot`). (2) Ensure `ServerToClient::SpawnAck { session_id: String }` variant exists; add if absent. (3) Define `SessionSidecarV3` struct (per SS-session-manager.md §Ruling B): shared schema-v3 type imported by both `monocle-runtime` and `monocle-session-host`; fields per the canonical struct definition in §Ruling B. |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~4,000 |
| BC-2.03.008 | ~1,500 |
| BC-2.08.001 | ~3,500 |
| BC-2.08.008 | ~3,500 |
| SS-session-manager.md (session_manager struct, spawn_session, IPC handler, error taxonomy sections) | ~12,000 |
| SS-engine-module-v2-delta.md (SpawnOptions, SpawnRecipe, EngineError, spawn_recipe default) | ~4,000 |
| SS-ipc.md (SpawnAck, SessionStateChanged, ClientToServer::SpawnSession) | ~3,000 |
| SS-daemon-wiring-v2-delta.md (IPC handler pattern, §3b emission table) | ~3,000 |
| SS-deps-pin-manifest.md + v2-delta (version pins) | ~2,000 |
| Existing codebase (monocle-runtime lib.rs, monocle-ipc types, engine module) | ~8,000 |
| Test files to write | ~4,000 |
| **Total estimate** | **~48,500** |

Estimate is within the 30% context window bound for a Sonnet-class model (~200k tokens = 60k max per story). No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.03.008 | Default spawn_recipe() Returns UnsupportedOperation | (see inputs: frontmatter) |
| BC-2.08.001 | Session Spawn — SessionHostSpawner Called Within 2s; SessionEntry Created | (see inputs: frontmatter) |
| BC-2.08.008 | SessionStateChanged — Daemon Emits on Every SessionState Transition; Delivered to All TUI Clients; Ordering Relative to SessionListUpdate | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `SessionManager` struct | `monocle-runtime/src/session_manager/mod.rs` | Effectful (owns state; spawns processes; writes files; publishes to broker) |
| `SessionHostSpawner` trait | `monocle-runtime/src/session_manager/mod.rs` | Pure (trait definition) |
| `RealSessionHostSpawner` | `monocle-runtime/src/session_manager/mod.rs` | Effectful (spawns OS process) |
| `MockSessionHostSpawner` | `monocle-runtime/src/session_manager/mod.rs` (test cfg) | Effectful (in-memory fake) |
| `session_error_to_code()` | `monocle-runtime/src/session_manager/mod.rs` | Pure (mapping function) |
| `monocle-session-host` binary | `crates/monocle-session-host/src/main.rs` | Effectful (OS process; PTY; UDS) |
| IPC handler `SpawnSession` arm | `monocle-runtime/src/ipc_handler.rs` | Effectful (IPC dispatch; broker publish) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-150 | `which::which("claude")` fails inside `spawn_recipe()` | `EngineError::BinaryNotFound` propagated; no process spawned; IPC code `"binary_not_found"` |
| EC-151 | `runtime_dir` not writable; sidecar write fails after process spawned | Orphan-kill SIGTERM → 2s → SIGKILL; `Err(SidecarWriteFailed)`; no registry entry |
| EC-152 | UUID v4 collision on first attempt | Retry once; second collision → `Err(SessionIdCollision)` |
| EC-153 | `spawn_session()` called while re-discovery running | Structurally impossible (re-discovery completes before UDS bind; TUI cannot connect earlier) |
| Kill-before-socket-bind | `kill_session()` called while `host_conn` is still `None` during Launching (rare race) | PID-based SIGTERM/SIGKILL fallback; `Launching → Terminating` (covered by S-034) |

## IPC Handler Arm Ownership Disambiguation

S-033 authors the **`ClientToServer::SpawnSession`** arm in `monocle-runtime/src/ipc_handler.rs`.
The canonical 7-arm split across the SS-08/SS-09 stories is:

| IPC Handler Arm | Owning Story |
|-----------------|-------------|
| `ClientToServer::SpawnSession` | **S-033** (this story) |
| `ClientToServer::KillSession` | S-034 |
| `ClientToServer::AttachSession` | S-035 |
| `ClientToServer::DetachSession` | S-035 |
| `ClientToServer::KeyInput` | S-047 |
| `ClientToServer::ResizePane` | S-047 |
| `ClientToServer::RenameSession` | S-047 |

S-033 MUST NOT add KillSession, AttachSession, DetachSession, KeyInput, ResizePane, or RenameSession arms —
those belong to S-034, S-035, and S-047 respectively.

## Subsystem Anchor Justifications

**SS-08 owns this story's scope** because `SessionManager` is the core component of SS-08 (session-manager subsystem), defined in SS-session-manager.md, and the spawn path is the entry point for all SS-08 session lifecycle operations.

**Dependency Anchors:**
- STORY-033 depends on S-014 because the `EngineModule` trait in `monocle-core/src/engine.rs` must exist as a base before this story can extend it with `spawn_recipe()` (trait method + default impl + `EngineError`); S-014 authored the trait foundation.
- STORY-033 depends on S-015 because the `ClaudeCodeModule` struct and its existing `EngineModule` impl must exist for integration-test references (the test double that exercises the default `UnsupportedOperation` impl verifies non-override behavior against the trait, not the concrete `ClaudeCodeModule`); S-015 authored the struct. Note: `ClaudeCodeModule::spawn_recipe()` does NOT yet exist at S-033 — that concrete override is delivered by S-045.
- STORY-033 depends on S-017 because `DaemonState`, `daemon_start_sequence`, and the IPC handler infrastructure must exist.
- STORY-033 depends on S-021 because `ClientToServer::SpawnSession`, `ServerToClient::SpawnAck`, `ServerToClient::SessionStateChanged`, `ServerToClient::SessionListUpdate`, and `Broker<Event>` wire types must exist in `monocle-ipc`.
- STORY-033 blocks S-034/S-035/S-036/S-037/S-038 because all other SS-08 stories operate on `SessionManager` and `SessionEntry` types that are first defined in this story.

## Trace

| Pass | Date | Change |
|------|------|--------|
| v1.8 | 2026-06-16 | SS-session-manager v2.7.1 Rulings C+D propagation: (1) inputs[] SS-session-manager pin bumped 2.7.0→2.7.1. (2) Tasks: RealSessionHostSpawner task corrected — `pre_exec(|| setsid())` removed; replaced with explicit NO pre_exec note and reference to §Ruling C. `pre_exec` is `unsafe fn`; `monocle-runtime` is `#![forbid(unsafe_code)]`; session-host binary calls setsid() at startup step 2. (3) host_conn writer storage confirmed as S-033 scope (post-spawn monitor steps 1–5 are all S-033; Ruling D). |
| v1.7 | 2026-06-16 | SS-session-manager v2.7.0 Rulings A+B propagation: (1) inputs[] SS-session-manager pin bumped 2.6.1→2.7.0. (2) Tasks: added minimum-viable session-host task (Ruling A step table: parse args, setsid, PTY open, CommandBuilder, spawn child, vt100::Parser stub, bind UDS, write sidecar with child_pid, send StateChanged{Running}, minimal Kill event loop; deferred: PTY streaming→S-039, scrollback→S-035, keyboard→S-047, resize→S-042, TUI→S-044). (3) Tasks: added SessionSidecarV3 definition task in monocle-ipc (Ruling B dual-writer ownership protocol). (4) Existing spawn_session sidecar-write task updated to reference `monocle_ipc::SessionSidecarV3` as the serialization type (not ad-hoc struct). (5) File Structure: monocle-session-host/Cargo.toml row adds `tempfile = "3"` dep (Ruling B atomic-write requirement). (6) monocle-ipc/src/lib.rs modify row adds SessionSidecarV3 definition (Ruling B). |
| v1.6 | 2026-06-16 | F-P21-SUG-002: AC-012 SpawnAck step-label aligned to BC-2.08.008 PC-5 canonical numbering — "IPC-handler step 1" → "IPC-handler step 2" (UUID generation is step 1; SpawnAck send is step 2). Relative ordering identical and correct in both versions; only the absolute step-index label changed. |
| v1.5 | 2026-06-16 | Corpus-wide AC-trace-citation audit (F-P20-CRIT-001 class): AC-012 "postcondition 4b"→"postcondition 5" (SpawnAck ordering guarantee is in BC-2.08.008 PC-5 §SessionCreation wizard auto-advance, not PC-4b §InitialState push). AC body unchanged. |
| v1.4 | 2026-06-16 | F-P16-IMP-001: Moved `SessionState` definition from `monocle-runtime/src/session_manager/mod.rs` to `crates/monocle-ipc/src/lib.rs` (canonical wire-type location). Added 5-variant canonical list (Launching/Running/Detached/Terminating/Terminated; Created/Killed RETIRED). Updated Tasks, File Structure, and Architecture Compliance Rules accordingly. monocle-ipc MUST NOT depend on monocle-runtime — placing SessionState in monocle-ipc resolves the crate-residency issue and aligns with S-048. |
