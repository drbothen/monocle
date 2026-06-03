---
document_type: architecture-section-delta
level: L3
section: "daemon-wiring-v2-delta"
subsystem: SS-04
version: "1.0.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T23:00:00Z
inputs:
  - specs/architecture/SS-daemon-wiring-impl.md
  - specs/architecture/SS-daemon-lifecycle.md
  - specs/architecture/adr/ADR-0009-native-session-host-process-model.md
  - specs/architecture/SS-session-manager.md
  - research/domain-monocle-vision-synthesis.md
input-hash: "09e0657"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Daemon Wiring v2 Delta (D-235 Rework Scope)

## Purpose

The D-235 daemon-wiring implementation (PR #39, feat/daemon-wire-serve) wired the daemon to
actually serve HTTP + UDS. It placed `SessionManager` as an in-process struct inside
`monocle-runtime`, with PTY ownership inside the daemon process.

ADR-0009 (D-238 escalation) changes the PTY ownership model: PTY masters and harness children
now live in `monocle-session-host` processes that outlive the daemon. `SessionManager` inside
`monocle-runtime` becomes a session-host COORDINATOR rather than a PTY owner.

This document specifies the rework scope for the implementer. It is a design-only spec;
all code changes go to the implementer-in-worktree.

---

## What changes (rework scope)

### 1. DaemonState struct — SessionManager field

**Before (D-235 model):**
```rust
pub struct DaemonState {
    // ... existing fields ...
    // No SessionManager field; sessions tracked in EnrichedSession registry only
}
```

**After (v1A model):**
```rust
pub struct DaemonState {
    // ... existing fields (unchanged) ...
    /// Session-host coordinator. Added in v1A.
    pub session_manager: Arc<Mutex<SessionManager>>,
}
```

`SessionManager` is wrapped in `Arc<Mutex<>>` because it is accessed from multiple tokio tasks
(IPC handler, HTTP hook handler, session-host proxy tasks).

### 2. daemon_start_sequence() — session re-discovery step

**Add step (between existing step 8 "write lock file" and step 9 that currently doesn't exist —
the UDS bind step):**

```
Step 8b (NEW): Session re-discovery
  - Instantiate SessionManager
  - Call session_manager.rediscover_sessions()
  - Log re-discovered sessions count
  - If re-discovery fails (corrupt sidecars), log WARN and continue with empty registry
    (do NOT abort startup — a clean start is better than no start)
```

This step MUST complete before the UDS socket is bound (step 9 / step 10 in current
SS-daemon-wiring.md sequence). TUI clients must not connect before the session registry is
populated.

**Integration test:** `test_daemon_start_sequence_with_session_rediscovery` — verifies that
a pre-existing session sidecar is discovered and the session appears in the initial state push
to a TUI client that connects after the daemon starts.

### 3. IPC handler — new ClientToServer variants

The IPC client message handler (`monocle-runtime/src/ipc_handler.rs` or equivalent) gains
branches for the new `ClientToServer` variants from ADR-0010:

```rust
ClientToServer::SpawnSession { recipe } => {
    let session_id = state.session_manager.lock().await
        .spawn_session(recipe, /* harness_id, profile_id from recipe context */)
        .await?;
    // Broadcast SessionListUpdate to all TUI clients
}
ClientToServer::KillSession { session_id } => {
    state.session_manager.lock().await.kill_session(&session_id).await?;
}
ClientToServer::KeyInput { session_id, bytes } => {
    state.session_manager.lock().await.send_key_input(&session_id, bytes).await?;
}
ClientToServer::ResizePane { session_id, rows, cols } => {
    state.session_manager.lock().await.resize_session(&session_id, rows, cols).await?;
}
ClientToServer::DetachSession { session_id } => {
    state.session_manager.lock().await.detach_session(&session_id).await?;
}
ClientToServer::RenameSession { session_id, new_name } => {
    state.session_manager.lock().await.rename_session(&session_id, new_name).await?;
}
// ... existing variants unchanged ...
```

### 4. InitialState IPC push — session list and session snapshots

The `ServerToClient::InitialState` message gains:

```rust
InitialState {
    sessions: Vec<EnrichedSession>,  // EXTENDED: includes spawned_by_monocle field
    ring_tail: Vec<HookEventRecord>,
    overlay_stack: Vec<PermissionPromptPayload>,
    drop_counter: u64,
    // (no new fields for session state — sessions list covers it)
}
```

The `sessions` list is now populated from BOTH:
1. `SessionManager.session_list()` — monocle-spawned sessions in all states.
2. Existing `EngineModule.detect()` scan — externally-launched sessions (unchanged).

The two lists are merged; `spawned_by_monocle` field distinguishes them.

### 5. broker fan-out — PtyOutput messages

The broker fan-out task (`monocle-runtime/src/broker.rs` or equivalent) gains a new message
type `Event::PtyOutput { session_id, bytes }`. When the session-host proxy task receives bytes
from a session-host (via `HostToDaemon::PtyBytes`), it posts them to the broker as:

```rust
broker.send(Event::PtyOutput {
    session_id: session_id.clone(),
    bytes,
}).await;
```

The broker fan-out sends `ServerToClient::PtyOutput { session_id, bytes }` to all connected
TUI clients. This follows the same fan-out pattern as `HookEventReceived`.

### 6. HTTP hook handler — session correlation

The existing HTTP hook handlers POST bodies carry a `session_id` field. The hook handler now
has an additional lookup path: if the `session_id` matches a monocle-spawned session in
`SessionManager`, no change in behavior (hooks are still processed normally). The correlation
is for metrics/labeling only (the `spawned_by_monocle` field on `EnrichedSession`).

---

## What does NOT change

- `daemon_start_sequence()` core sequence (steps 1–8, 9+) — only step 8b is added.
- `run_server()` — HTTP axum server; unchanged.
- Hook ingestion endpoints — unchanged.
- Auth token handling — unchanged.
- Lock file path and format — unchanged.
- JSONL ring and ring flush — unchanged.
- UDS socket bind/cleanup — unchanged.
- `PermissionDecision` handling and `oneshot::channel` mechanism — unchanged.
- Event broker architecture — extended (new message type), not replaced.

---

## Pre-existing in-process SessionManager stub

The D-235 implementation may have a stub `SessionManager` struct that holds an in-process
PTY (if implemented). That stub is REPLACED by the coordinator design in SS-08 (SS-session-manager.md).
The implementer MUST:
1. Remove any `portable-pty` or `vt100` dependencies from `monocle-runtime/Cargo.toml` (if
   added by D-235). These crates belong in `monocle-session-host`, not in `monocle-runtime`.
2. Replace the in-process PTY spawn with `RealSessionHostSpawner::spawn()`.
3. Retain the `PtySpawner` or `SessionHostSpawner` trait seam for tests.

If no in-process SessionManager stub exists in D-235 (i.e., the stub was skeletal), the
implementer creates `SessionManager` from scratch per SS-08.

---

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- D-235 rework scope specified for v1A architecture delta.
- Covers: DaemonState struct, daemon_start_sequence step 8b, IPC handler new variants,
  InitialState extension, broker PtyOutput fan-out, HTTP hook handler correlation.
- Documents what does NOT change to prevent scope creep.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
