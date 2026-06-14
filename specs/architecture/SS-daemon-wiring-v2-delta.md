---
document_type: architecture-section-delta
level: L3
section: "daemon-wiring-v2-delta"
subsystem: SS-04
version: "1.9.0"
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

**Current step numbering in SS-daemon-wiring.md (authoritative):**
- Step 8: Write lock file (SOQ-2 write point)
- Step 9: Generate hooks-settings.json (BC-2.04.010)
- Step 10: Create UDS socket (bind `<runtime_dir>/monocle.sock`)
- Step 13: Signal startup complete (foreground caller polls for lock file)

**C2 — Dual readiness signals: lock file vs UDS socket.**

These are two DISTINCT readiness signals with different semantics:

- **Lock file (step 8):** SOQ-2 write point. The foreground `daemon start` caller polls for
  the lock file to confirm "daemon process is alive and has bound the HTTP port." This is the
  existing startup-complete signal (step 13 description in SS-daemon-wiring.md). Session
  re-discovery has NOT yet completed at the moment the lock file is written — it runs at step 8b.
- **UDS socket (step 10):** TUI-ready write point. A TUI client can connect to the UDS socket
  and receive a valid `InitialState` push only after the UDS socket file exists. Since step 8b
  (re-discovery) precedes step 10 (UDS bind), the `InitialState` push always includes all
  re-discovered sessions.

**The foreground caller contract (step 13) is correct as written:** it polls for the lock
file (step 8), which appears BEFORE re-discovery (step 8b) and BEFORE UDS bind (step 10).
This means the foreground `daemon start` exits successfully while re-discovery may still
be in progress. This is intentional and safe: the foreground caller does not connect to
the UDS socket; it only checks the lock file. TUI clients that subsequently connect to the
UDS socket will see all re-discovered sessions because the UDS socket is not bound until
step 10, which comes after step 8b.

**There is NO race for TUI clients.** A TUI client that attempts to connect to the UDS
socket before the socket is created will receive ENOENT (file not found) or ECONNREFUSED,
and will retry per the auto-start polling contract in SS-daemon-wiring.md §Auto-Start
Decision Sequence step 4. The UDS socket bind (step 10) is the re-discovery-complete
signal for TUI clients; the lock file (step 8) is the startup-alive signal for the
foreground caller. These two signals are intentionally separate.

**Add step 8b between existing step 8 (write lock file) and step 9 (generate hooks-settings.json):**

```
Step 8b (NEW): Session re-discovery
  - Instantiate SessionManager with runtime_dir
  - Call session_manager.rediscover_sessions()
    - Runs parallel attach probes (tokio::join_all) for all sidecars
    - Each probe applies SO_PEERCRED cross-check before accepting session
  - Log re-discovered sessions count (found_alive, found_dead)
  - If re-discovery fails (corrupt sidecars, runtime_dir unreadable): log WARN/ERROR
    and continue with empty registry (do NOT abort startup — a clean start is better
    than no start; TUI will show no pre-existing sessions)
  - Re-discovery MUST return before step 9 begins
```

**Insertion invariant:** Step 8b MUST complete before step 9 (hooks-settings.json) and
before step 10 (UDS socket bind). The UDS bind at step 10 is the point at which TUI
clients can connect; session re-discovery must finish before any client can connect to
ensure the initial state push contains all re-discovered sessions. The lock file (step 8)
is written BEFORE step 8b begins, so the foreground caller can observe "daemon alive"
while re-discovery is still running — this is correct (see §Dual readiness signals above).
Steps 9 (hooks-settings.json) and 10 (UDS bind) retain their existing order and positions;
step 8b is inserted before both.

**Integration test:** `test_daemon_start_sequence_with_session_rediscovery` — verifies that
a pre-existing session sidecar is discovered and the session appears in the initial state push
to a TUI client that connects after the daemon starts.

### 3. IPC handler — new ClientToServer variants

> **CANONICAL PATTERN LOCK (P8-STRUCTURAL):** The error-routing discipline in this handler
> MUST match **SS-session-manager.md §Error handling** (lines ~443–507), which is the single
> authoritative specification for `IpcOp`, `session_error_to_code`, the per-op error-code
> mapping table, the KeyInput special rule, and the ResizePane special rule. Any future
> change to the canonical pattern in SS-session-manager.md §Error handling MUST propagate
> here in the same edit burst. Do NOT maintain a locally-diverging copy of the logic.
>
> **Import note:** `IpcOp` and `session_error_to_code` are defined in
> `monocle-session-manager` (or `monocle-runtime::session_manager`) per SS-session-manager.md
> §IPC handler pattern. The implementing crate must `use` them. All nine `ClientToServer`
> arms MUST follow the canonical no-silent-failure discipline defined there — no bare `?`
> that lets a `SessionError` escape to the per-client task boundary.

The IPC client message handler (`monocle-runtime/src/ipc_handler.rs` or equivalent) gains
branches for the new `ClientToServer` variants from ADR-0010:

```rust
ClientToServer::SpawnSession { opts } => {
    // C6-001: no-silent-failure — map SessionError to ServerToClient::Error.
    //
    // I27-001 (Model A): The TUI sends SpawnOptions (user intent). The daemon fills
    // opts.session_id and opts.hooks_settings_path here, before calling spawn_session().
    // spawn_session() internals:
    //   1. Calls engine_module.spawn_recipe(&opts)? — DAEMON-SIDE recipe construction.
    //      If this returns EngineError::BinaryNotFound or EngineError::InvalidPath, the
    //      error is converted to SessionError::EngineError via From<EngineError> and
    //      bubbles to the Err(e) arm below. No OS process has been spawned at this point.
    //      These EngineError codes are REACHABLE because spawn_recipe() runs daemon-side.
    //   2. Calls SessionHostSpawner::spawn(&recipe) — OS process spawn.
    //   3. Writes the sidecar file.
    //
    // Fill daemon-owned fields on opts before passing to spawn_session().
    // C30-001: SpawnOptions is #[non_exhaustive]; functional-update (`..opts`) is E0639
    // outside the defining crate. Use the consuming builder `with_daemon_fields()` instead.
    let opts = opts.with_daemon_fields(
        uuid::Uuid::new_v4().to_string(),
        state.hooks_settings_path.clone(), // pre-written at step 9
    ); // project_root, worktree_root, harness_id, profile_id, ccr_base_url came from TUI
    //
    // session_error_to_code maps the EngineError-derived variants:
    //   SessionError::EngineError(BinaryNotFound) → "binary_not_found"
    //   SessionError::EngineError(InvalidPath)    → "invalid_spawn_arg"
    // See SS-session-manager.md §SessionError taxonomy and §session_error_to_code().
    match state.session_manager.lock().await
        .spawn_session(opts)
        .await
    {
        Ok(_session_id) => {
            // spawn_session() emits SessionStateChanged{Launching} then
            // SessionListUpdate to the broker (see §SessionStateChanged emission rule below).
        }
        Err(e) => {
            let _ = client_tx.send(ServerToClient::Error {
                code: session_error_to_code(IpcOp::Spawn, &e).to_string(),
                message: e.to_string(),
            }).await;
        }
    }
}
ClientToServer::KillSession { session_id } => {
    // C6-001: no-silent-failure — map SessionError to ServerToClient::Error
    // IpcOp::Kill is required so SessionHostDead maps to "kill_failed" (not "attach_failed").
    match state.session_manager.lock().await.kill_session(&session_id).await {
        Ok(()) => {
            // kill_session() emits SessionStateChanged{Terminating} then SessionListUpdate.
        }
        Err(e) => {
            let _ = client_tx.send(ServerToClient::Error {
                code: session_error_to_code(IpcOp::Kill, &e).to_string(),
                message: e.to_string(),
            }).await;
        }
    }
}
ClientToServer::AttachSession { session_id } => {
    // TUI-initiated re-attach (used after PtyReset or explicit re-attach request).
    // Daemon calls SessionManager::attach_session() which issues DaemonToHost::Attach to
    // the session-host and streams a fresh ScrollbackDump (ScrollbackChunk* + Complete).
    // On ScrollbackDumpComplete receipt, daemon emits SessionStateChanged{Running} then
    // SessionListUpdate (if state changed from Detached to Running).
    // C6-002: ClientToServer::ReAttach does NOT exist — I3-004 consolidated all re-attach
    // onto AttachSession. The ReAttach alternative was removed in v1.4.0.
    match state.session_manager.lock().await.attach_session(&session_id).await {
        Ok(()) => { /* SessionStateChanged{Running} + SessionListUpdate emitted by attach_session */ }
        Err(e) => {
            // C6-001: no-silent-failure — send Error to requesting client
            let _ = client_tx.send(ServerToClient::Error {
                code: session_error_to_code(IpcOp::Attach, &e).to_string(),
                message: e.to_string(),
            }).await;
        }
    }
}
ClientToServer::KeyInput { session_id, bytes } => {
    // KeyInput special rule (SS-session-manager §Error handling): fire-and-forget on success
    // (no reply), but errors MUST NOT be dropped — SessionError is routed to
    // ServerToClient::Error. Bare `?` is forbidden (silently drops the connection).
    match state.session_manager.lock().await.send_key_input(&session_id, bytes).await {
        Ok(()) => { /* no acknowledgement sent on success — fire-and-forget */ }
        Err(e) => {
            let _ = client_tx.send(ServerToClient::Error {
                code: session_error_to_code(IpcOp::KeyInput, &e).to_string(),
                message: e.to_string(),
            }).await;
        }
    }
}
ClientToServer::ResizePane { session_id, rows, cols } => {
    // ResizePane special rule (SS-session-manager §Error handling): resize errors are
    // benign races (session may have terminated between TUI send and daemon processing).
    // Do NOT send ServerToClient::Error; do NOT propagate with `?`. Log at WARN and
    // continue — the per-client task MUST remain alive.
    if let Err(e) = state.session_manager.lock().await
        .resize_session(&session_id, rows, cols).await
    {
        tracing::warn!(
            session_id = %session_id,
            error = %e,
            "resize_session failed (benign race — session may have terminated); ignoring"
        );
    }
}
ClientToServer::DetachSession { session_id } => {
    // detach_session() emits SessionStateChanged{Detached} then SessionListUpdate.
    match state.session_manager.lock().await.detach_session(&session_id).await {
        Ok(()) => { /* SessionStateChanged{Detached} + SessionListUpdate emitted by detach_session */ }
        Err(e) => {
            let _ = client_tx.send(ServerToClient::Error {
                code: session_error_to_code(IpcOp::Detach, &e).to_string(),
                message: e.to_string(),
            }).await;
        }
    }
}
ClientToServer::RenameSession { session_id, new_name } => {
    // rename_session() emits SessionListUpdate ONLY (rename is not a state transition;
    // no SessionStateChanged is emitted — see C3-003 fix in SS-session-manager).
    match state.session_manager.lock().await.rename_session(&session_id, new_name).await {
        Ok(()) => { /* SessionListUpdate emitted by rename_session */ }
        Err(e) => {
            let _ = client_tx.send(ServerToClient::Error {
                code: session_error_to_code(IpcOp::Rename, &e).to_string(),
                message: e.to_string(),
            }).await;
        }
    }
}
// ... existing variants unchanged ...
```

#### §3 per-arm error-handling summary (all nine ClientToServer arms)

The following table is authoritative for this handler. No arm may use bare `?` that allows
a `SessionError` to escape to the per-client task boundary.

| Arm | Error routing | Op context | Notes |
|-----|---------------|------------|-------|
| `SpawnSession` | `ServerToClient::Error` | `IpcOp::Spawn` | Lifecycle op — includes EngineError-derived codes `"binary_not_found"` / `"invalid_spawn_arg"` via `SessionError::EngineError` bridge (I12-001) |
| `KillSession` | `ServerToClient::Error` | `IpcOp::Kill` | `IpcOp::Kill` required: `SessionHostDead` → `"kill_failed"` not `"attach_failed"` |
| `AttachSession` | `ServerToClient::Error` | `IpcOp::Attach` | `IpcOp::Attach` required: `SessionHostDead` → `"attach_failed"` |
| `KeyInput` | `ServerToClient::Error` | `IpcOp::KeyInput` | Fire-and-forget on success; errors MUST still be surfaced (special rule) |
| `ResizePane` | WARN-log-and-continue | `IpcOp::Resize` | Benign race; NO `ServerToClient::Error`; per-client task MUST stay alive |
| `DetachSession` | `ServerToClient::Error` | `IpcOp::Detach` | Lifecycle op |
| `RenameSession` | `ServerToClient::Error` | `IpcOp::Rename` | Rename emits `SessionListUpdate` only (not `SessionStateChanged`) |
| `PermissionDecision` | (existing — no change) | — | Oneshot channel; error path unchanged from D-235 |
| `Ping` | (existing — no change) | — | Pong reply; no `SessionError` path |

### 3b. SessionStateChanged emission rule (C3-001 fix)

Every `SessionState` transition in `SessionManager` MUST emit `ServerToClient::SessionStateChanged`
BEFORE `ServerToClient::SessionListUpdate` to all clients. This ordering is the contract defined
in BC-2.08.008 Invariant 4.

**Emission site:** Inside every `SessionManager` method that mutates `SessionEntry.state`, the
implementation MUST, while still holding the `Arc<Mutex<SessionManager>>` lock, call:

```rust
// Pattern for ALL state-mutating SessionManager methods:
// 1. Mutate SessionEntry.state to new_state
session_entry.state = new_state;

// 2. Post SessionStateChanged FIRST (while lock is still held for ordering guarantee)
broker.try_send(Event::SessionStateChanged {
    session_id: session_id.clone(),
    new_state: new_state.clone(),
});

// 3. Post SessionListUpdate SECOND (while lock is still held)
broker.try_send(Event::SessionListUpdate {
    sessions: self.session_list(),  // snapshot taken inside lock
});
```

The lock is held for BOTH posts. This guarantees ordered enqueue into each client's per-client
channel — `SessionStateChanged` is enqueued before `SessionListUpdate` for every client.

**Methods that emit the SessionStateChanged + SessionListUpdate pair:**

| Method | SessionState transition | Emits |
|--------|------------------------|-------|
| `spawn_session()` | (none) → `Launching` | `SessionStateChanged{Launching}` then `SessionListUpdate` |
| On `StateChanged::Running` from session-host | `Launching` → `Running` | `SessionStateChanged{Running}` then `SessionListUpdate` |
| `kill_session()` | `Running/Detached/Launching` → `Terminating` | `SessionStateChanged{Terminating}` then `SessionListUpdate` |
| On `StateChanged::Terminated` or 12s watchdog | `Terminating` → `Terminated` | `SessionStateChanged{Terminated}` then `SessionListUpdate` |
| `detach_session()` | `Running` → `Detached` | `SessionStateChanged{Detached}` then `SessionListUpdate` |
| `attach_session()` on `ScrollbackDumpComplete` | `Detached` → `Running` | `SessionStateChanged{Running}` then `SessionListUpdate` |
| Re-discovery GC (dead session) | any → `Terminated` | `SessionStateChanged{Terminated}` then `SessionListUpdate` |

**Methods that emit SessionListUpdate ONLY (no state change):**

| Method | Reason |
|--------|--------|
| `rename_session()` | Rename updates `display_name` only; no `SessionState` transition occurs; `SessionStateChanged` carries `new_state` and cannot convey the new name — only `SessionListUpdate` (carrying the full `SessionSnapshot` with updated `display_name`) is correct. See C3-003 fix in SS-session-manager. |

**Broker fan-out ordered-pair split behavior (I3-001 fix):**

The two posts (SessionStateChanged then SessionListUpdate) are enqueued via `.try_send()` into
each client's per-client channel. If the first `.try_send()` succeeds but the second fails
(channel full), the pair has SPLIT for that client. The split is treated as a slow-client
condition: the per-client `slow_send_count` is incremented AND the client is immediately treated
as having exhausted its 3-strike threshold for this pair (a split pair is equivalent to 2
consecutive full-buffer failures). The client is disconnected. The client may reconnect and
receive a fresh `InitialState` containing the post-transition state.

Rationale: delivering a half-pair (SessionStateChanged without SessionListUpdate, or vice versa)
leaves the TUI in an inconsistent state. Disconnecting is safer than tolerating partial delivery.
This extends the existing 3-strike slow-client disconnect model (ADR-0010 §Cross-Client /
Cross-Session Backpressure Isolation) with an explicit ordered-pair invariant.

**Real ordering guarantee statement:** The ordering guarantee is: for a given TUI client, if both
messages are delivered, `SessionStateChanged` always precedes `SessionListUpdate` in the
client's receive order. The guarantee is provided by (a) enqueuing both into the same FIFO
per-client channel in the correct order, and (b) the per-client writer task draining the
channel in FIFO order. The `SessionManager` mutex does NOT directly touch the channel enqueue
order — it provides the atomicity window during which both posts are made before any other
actor can post to the broker. The channel FIFO order is the actual ordering mechanism.

### 4. InitialState IPC push — session list and session snapshots (C3-004 fix)

#### SessionSnapshot — canonical boundary type

`SessionSnapshot` is the canonical boundary type that crosses the UDS wire in `InitialState`
and `SessionListUpdate`. It is defined in `monocle-ipc/src/types.rs` so both daemon and TUI
can use it without importing daemon-internal types.

> **Canonical definition:** See **SS-ipc.md §Supporting Types — `SessionSnapshot`** for the
> authoritative field list. The canonical definition includes `degraded: bool` and
> `degraded_reason: Option<String>` (added in v1.12.0 / I3-009 fix) in addition to the
> ten base fields. **Do not maintain a duplicate struct here** — the inline copy in §4
> was retired in v1.4.0 (I6-002 fix) because it diverged from the SS-ipc.md canonical by
> omitting the `degraded`/`degraded_reason` fields.
>
> **Current canonical field summary** (SS-ipc.md v1.20.1 §Supporting Types — authoritative):
> `session_id`, `display_name`, `state`, `harness_id`, `project_root`, `cwd`,
> `spawned_by_monocle: Option<bool>`, `started_at_micros: i64`, `pty_rows: u16`,
> `pty_cols: u16`, `degraded: bool` (`#[serde(default)]`), `degraded_reason: Option<String>`
> (`#[serde(default)]`). Total: 12 fields.

#### Three session representations — reconciliation

The architecture has three representations of a session. They serve distinct purposes and
must NOT be conflated:

| Type | Location | Purpose |
|------|----------|---------|
| `SessionEntry` | `monocle-runtime` (daemon-internal) | Internal registry: holds live OS-level data (pid, host_conn, proxy_task, socket_path). Never crosses the wire. |
| `EnrichedSession` | `monocle-ipc` (SS-05 existing type) | Externally-detected session data from `EngineModule::detect()`. Contains detection metadata. Remains for the `EngineModule` detection path. |
| `SessionSnapshot` | `monocle-ipc` (new SS-05 v1A type) | Wire boundary type for ALL sessions (both monocle-spawned and externally-detected). This is what `InitialState.sessions` and `SessionListUpdate.sessions` carry. |

`EnrichedSession` is NOT retired — it is still used by `EngineModule::detect()` internally.
However, `EnrichedSession` instances from detection are CONVERTED to `SessionSnapshot` before
being placed on the wire. The conversion fills `spawned_by_monocle: Some(false)` for detected
sessions.

`SessionManager::session_list()` returns `Vec<SessionSnapshot>` (already defined in
SS-session-manager.md §Public API). The daemon merges:
1. `session_manager.session_list()` — monocle-spawned sessions, all states.
2. `EngineModule::detect()` scan results — externally-launched sessions, converted to `SessionSnapshot`.

The merged `Vec<SessionSnapshot>` is what appears in `InitialState.sessions` and `SessionListUpdate.sessions`.

#### Updated InitialState variant

```rust
InitialState {
    sessions: Vec<SessionSnapshot>,   // C3-004: SessionSnapshot replaces EnrichedSession on wire
    ring_tail: Vec<HookEventRecord>,
    overlay_stack: Vec<PermissionPromptPayload>,
    drop_counter: u64,
}
```

#### Updated SessionListUpdate variant

```rust
SessionListUpdate {
    sessions: Vec<SessionSnapshot>,   // C3-004: SessionSnapshot replaces EnrichedSession on wire
}
```

**Note for SS-ipc.md:** The existing `SS-ipc.md` v1.11.0 declares `InitialState.sessions: Vec<EnrichedSession>`
and `SessionListUpdate.sessions: Vec<EnrichedSession>`. These must be updated to
`Vec<SessionSnapshot>` in SS-ipc.md (see §Trace and downstream PO BC change list).

### 5. broker fan-out — PtyOutput, ScrollbackChunk, ScrollbackDumpComplete, PtyReset

The broker fan-out task (`monocle-runtime/src/broker.rs`) gains the following new event types.
Each is dispatched to all connected TUI clients using the per-client isolated send buffer
design (see ADR-0010 §Cross-Client / Cross-Session Backpressure Isolation).

#### 5a. PtyOutput fan-out

When the session-host proxy receives `HostToDaemon::PtyBytes`, it posts:

```rust
broker.send(Event::PtyOutput {
    session_id: session_id.clone(),
    bytes,
});
```

The broker fan-out dispatches `ServerToClient::PtyOutput { session_id, bytes }` to each
client's per-client send buffer via `.try_send()`. A dedicated per-client writer task drains
the buffer to the UDS socket.

#### 5b. ScrollbackChunk + ScrollbackDumpComplete fan-out (I3-003 fix: resume after snapshot)

When a daemon attaches to a session-host (on spawn or re-discovery) and receives the
session-host's scrollback dump stream (`HostToDaemon::ScrollbackChunk` / `HostToDaemon::ScrollbackDumpComplete`),
the daemon fans out these to TUI clients:

```rust
// For each HostToDaemon::ScrollbackChunk { rows, chunk_seq } received from session-host:
broker.send(Event::ScrollbackChunk {
    session_id: session_id.clone(),
    rows,
    chunk_seq,
});
// After HostToDaemon::ScrollbackDumpComplete received:
broker.send(Event::ScrollbackDumpComplete {
    session_id: session_id.clone(),
    total_chunks,
    cursor_row,
    cursor_col,
    pty_rows,
    pty_cols,
});
```

The broker delivers `ServerToClient::ScrollbackChunk` and `ServerToClient::ScrollbackDumpComplete`
to each client's per-client buffer. The TUI accumulates chunks until Complete, then resets
and reconstructs the parser.

**I3-003 fix — session-host resumes live PtyBytes immediately after snapshot:**

The prior ADR-0010 §Interleaving specification required the session-host to PAUSE live
`HostToDaemon::PtyBytes` forwarding for the entire duration of the scrollback dump transfer.
This is unbounded for large scrollbacks on slow consumers: the 1024-entry PTY reader channel
fills while the session-host waits for the dump to complete, causing `.send().await`
backpressure on the harness child's PTY read path, stalling the harness mid-output.

**Revised protocol (I3-003):** The scrollback dump is a point-in-time SNAPSHOT of the
`vt100::Screen` taken at the moment `DaemonToHost::Attach` is received. The session-host:

1. On receiving `DaemonToHost::Attach`: atomically snapshot the current `vt100::Screen` into
   a `Vec<Vec<SerializedCell>>` capture. This snapshot is taken in the async event loop (not
   blocking) — the parser state is copied (or a reference counted snapshot taken) at this
   instant.
2. **Resume live `PtyBytes` forwarding IMMEDIATELY** after taking the snapshot (do NOT pause
   for the dump transfer). New PTY bytes continue to flow to the daemon as `HostToDaemon::PtyBytes`
   while the dump is being chunked and sent.
3. Stream `ScrollbackChunk*` + `ScrollbackDumpComplete` from the snapshot asynchronously
   (not pausing the PTY reader).

**TUI ordering protocol after I3-003:**

Since live `PtyOutput` messages may now arrive BEFORE or DURING the dump transfer, the TUI
must apply them correctly:

1. While accumulating `ScrollbackChunk` messages (dump in progress), the TUI MUST buffer
   any `ServerToClient::PtyOutput` messages for that session rather than feeding them to
   the current (pre-reset) parser state.
2. On receipt of `ScrollbackDumpComplete`: reset the parser from the dump snapshot, then
   replay all buffered `PtyOutput` bytes through the freshly-reset parser. Discard the
   `PtyOutput` buffer after replay.
3. After replay, subsequent live `PtyOutput` messages are processed normally by the parser.

This "snapshot then resume, buffer live bytes, replay after Complete" protocol ensures:
- The harness child is NEVER stalled due to a slow scrollback consumer.
- The TUI receives a coherent terminal state: snapshot + post-snapshot live bytes applied
  in order, with no double-counting or interleaving corruption.
- The `PtyOutput` buffer for the dump window is bounded: a typical dump completes in < 500ms
  for a 10k-row scrollback; at 1 MB/s PTY output that is at most 500 KB of buffered live bytes
  — well within memory budget.

**ADR-0010 §Interleaving update:** ADR-0010 must be updated to reflect this revised protocol.
The prior "buffer-then-apply-after-Complete" (session-host pauses PtyBytes) is RETIRED.
The new protocol is "snapshot-then-resume, TUI buffers live PtyOutput, replays after Complete".
This change is recorded in ADR-0010 §Trace v1.3.0 (I3-003 fix).

**Benchmark note:** ADR-0010 already requires a pre-v1A benchmark gate. That benchmark MUST
additionally verify that a 10k-row max scrollback dump completes while the harness child
produces 1 MB/s PTY output with zero PTY channel drops.

**I11-001 SCOPE NOTE — v1A single-client broadcast; multi-client attach-storm is explicitly
deferred (post-v1A):**

The `ScrollbackChunk*` + `ScrollbackDumpComplete` fan-out in §5b broadcasts to ALL connected
TUI clients (BC-2.05.009 Invariant 2 and BC-2.05.011 Invariant 5 use "all connected TUI
clients"). This is intentional forward-compatible infrastructure for the v1A single-client
case. In v1A, there is exactly one TUI process attached to the daemon at any given time (monocle
is a tmux popup — one popup per user session; multiple TUI clients simultaneously is not a v1A
use case). The broadcast is therefore harmless for v1A.

**The known multi-client "attach-reset storm"** (one client's `ClientToServer::AttachSession`
triggers a `DaemonToHost::Attach` → session-host sends `ScrollbackChunk*` +
`ScrollbackDumpComplete` → broadcast resets ALL clients' parsers, including clients that did
NOT request re-attach) is a REAL correctness issue for concurrent multi-TUI scenarios. This
issue is **explicitly deferred** to the future multi-client capability (ref: `BC-2.05.009
Invariant 2` which states the all-clients fan-out "supports FUTURE multi-TUI scenarios").

**Deferral anchor:** This is tracked as `TD-MULTI-CLIENT-ATTACH-STORM-001`. The candidate fix
(per-client unicast scrollback — the daemon sends `ScrollbackChunk*` + `ScrollbackDumpComplete`
only to the client that sent `AttachSession`, not broadcast) MUST be designed and specified when
the multi-client TUI capability is scheduled. The fix requires: (a) routing `AttachSession` to a
client-scoped delivery path in the broker, and (b) ensuring the session-host can service
concurrent per-client dump requests without interleaving chunks. Until then, the broadcast is
the canonical implementation. Implementers reading this MUST NOT "fix" the broadcast to unicast
speculatively — that change has correctness implications for the PtyReset recovery path and
requires BC updates.

**HostToDaemon enum additions (session-host → daemon, per-session UDS):**
The `HostToDaemon` enum in SS-session-manager.md §Per-session UDS protocol gains two new
variants for the chunked scrollback protocol:
```rust
ScrollbackChunk { rows: Vec<Vec<SerializedCell>>, chunk_seq: u32 },
ScrollbackDumpComplete { total_chunks: u32, cursor_row: u16, cursor_col: u16, pty_rows: u16, pty_cols: u16 },
```
The existing `HostToDaemon::ScrollbackDump { rows, cursor_row, cursor_col, pty_rows, pty_cols }`
is REPLACED by this two-message protocol. `ScrollbackDump` is RETIRED from `HostToDaemon`
(the name was misleading for large scrollbacks). Session-hosts MUST NOT send `ScrollbackDump`
in v1A; they MUST send `ScrollbackChunk*` + `ScrollbackDumpComplete`.

#### 5c. PtyReset fan-out

When the session-host proxy receives `HostToDaemon::PtyReset`, it posts:

```rust
broker.send(Event::PtyReset {
    session_id: session_id.clone(),
});
```

The broker dispatches `ServerToClient::PtyReset { session_id }` to all TUI clients. Each TUI
client resets `pty_parsers[session_id]` and re-attaches (triggering a new
`ScrollbackChunk*` + `ScrollbackDumpComplete` sequence).

#### 5d. Per-client isolated send buffer (I2-003 fix + O3-004 sizing correction)

The broker fan-out MUST use per-client isolation to prevent cross-client / cross-session
backpressure livelock. For each connected TUI client, the broker maintains:
- `client_send_tx: mpsc::Sender<ServerToClient>` (capacity = `CLIENT_SEND_BUFFER_SIZE`,
  see sizing below)
- A dedicated per-client writer task that drains the channel to the UDS socket via
  `.write_all().await`.

The broker fan-out uses `.try_send()` into each client's channel. On `Err(Full)`, the broker
increments a per-client `slow_send_count`. After `SLOW_CLIENT_DISCONNECT_THRESHOLD`
consecutive full-send attempts (default 3), the broker disconnects the client and logs
`WARN: slow TUI client disconnected (send buffer repeatedly full)`.

This guarantees that a stalled client's backpressure does NOT propagate to the PTY reader
or to other clients. See ADR-0010 §Cross-Client / Cross-Session Backpressure Isolation.

**O3-004 fix — per-client buffer sizing corrected:**

The prior ADR-0010 §Cross-Client buffer sizing stated "256 × 4096 bytes = 1 MiB per client
per session." This was wrong: the same buffer carries `ScrollbackChunk` messages which can be
up to 256 KiB each. At 256 messages × 256 KiB/message the buffer could hold up to 64 MiB of
in-flight data per session — not 1 MiB.

**I3-003 interaction:** The "snapshot-then-resume" protocol (§5b I3-003 fix) substantially
reduces per-client buffer pressure during scrollback dumps because the session-host no longer
pauses live `PtyBytes` for the full dump duration. The dump chunks and live `PtyOutput`
messages are interleaved; the TUI-side buffers the live bytes and replays after Complete.
This means the per-client channel no longer needs to absorb the full in-flight dump volume
from the session-host at once.

**Corrected sizing rationale:**

The per-client channel carries a mix of message types with very different sizes:
- `PtyOutput`: typical 1–4 KiB; max 256 KiB (per BC-2.01.003 message limit).
- `ScrollbackChunk`: typical 10–256 KiB (depends on row density).
- `SessionStateChanged`, `SessionListUpdate`, `HookEventReceived`: < 1 KiB.

With the I3-003 "resume immediately" protocol, scrollback chunks and live PtyOutput are
interleaved in the per-client channel, but the peak chunk size is capped at 256 KiB (per
the session-host serializer). A buffer of 64 messages provides:
- Best case (small PtyOutput at 1 KiB average): 64 × 1 KiB = 64 KiB buffered.
- Worst case (all ScrollbackChunks at 256 KiB): 64 × 256 KiB = 16 MiB buffered.
- Practical case (mixed messages, average ~16 KiB/message): 64 × 16 KiB = 1 MiB buffered.

**`CLIENT_SEND_BUFFER_SIZE = 64` messages** (down from the erroneous 256, to bound the
worst-case memory consumption). 64 entries at 256 KiB max/entry = 16 MiB worst-case per
client × 4 clients = 64 MiB — within the memory budget.

The disconnect-after-3-consecutive-full-sends threshold is unchanged.

**ADR-0010 §Cross-Client buffer sizing note** must be updated to reflect 64 messages and
the correct worst-case computation (see ADR-0010 §Trace v1.3.0).

### 6. HTTP hook handler — session correlation

The existing HTTP hook handlers POST bodies carry a `session_id` field. The hook handler now
has an additional lookup path: if the `session_id` matches a monocle-spawned session in
`SessionManager`, no change in behavior (hooks are still processed normally). The correlation
is for metrics/labeling only.

**Wire representation (C3-004):** The `spawned_by_monocle` badge is carried on the wire as
`SessionSnapshot.spawned_by_monocle` (the wire type used for `InitialState.sessions` and
`SessionListUpdate.sessions` per C3-004). The hook handler reads `SessionEntry.spawned_by_monocle`
from the daemon-internal registry to populate the `SessionSnapshot` field on outbound events.
`EnrichedSession.spawned_by_monocle` is used only on the internal session-detect path
(`EngineModule::detect()` → `EnrichedSession`) — not on the IPC wire.

---

## What does NOT change

- `daemon_start_sequence()` core sequence (steps 1–8 and 9–13) — only step 8b is inserted
  between step 8 (lock file) and step 9 (hooks-settings.json). All other steps are unchanged.
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

## §Trace v1.9.0

**C30-001 — ADR-0006 constructor fix: replace E0639-violating `..opts` functional-update with `with_daemon_fields()` consuming builder** (2026-06-13):

- **Finding (C30-001 CRITICAL):** §3 `SpawnSession` handler arm used `SpawnOptions { session_id: ..., hooks_settings_path: ..., ..opts }`. Functional-record-update syntax (`..base`) on a `#[non_exhaustive]` struct from an external crate is E0639 in Rust — the same restriction that applies to struct-literal construction. `SpawnOptions` is defined in `monocle-core`; this handler lives in `monocle-runtime`; E0639 applies.
- **Fix — `..opts` replaced with `opts.with_daemon_fields(uuid, path)`:** `SpawnOptions::with_daemon_fields(self, session_id: String, hooks_settings_path: PathBuf) -> Self` is the ADR-0006-conformant consuming builder defined in SS-engine-module-v2-delta.md (canonical constructor owner) and mirrored in SS-session-manager.md §SpawnOptions. The sample now reads: `let opts = opts.with_daemon_fields(uuid::Uuid::new_v4().to_string(), state.hooks_settings_path.clone());`.
- **No semantic change:** The two daemon-owned fields (`session_id`, `hooks_settings_path`) are set to the identical values as before. Only the construction mechanism changes (struct-update → consuming builder). The TUI-provided fields (`project_root`, `worktree_root`, `harness_id`, `profile_id`, `ccr_base_url`) are preserved unchanged via `with_daemon_fields()`.
- Semver: patch (v1.8.0 → v1.9.0) — sample code correction; no behavioral change.

## §Trace v1.8.0

**I27-001 — `SpawnSession` arm: `recipe: SpawnRecipe` → `opts: SpawnOptions`; daemon fills session_id + hooks_settings_path** (2026-06-13):

- **Finding (I27-001):** The `SpawnSession` handler arm destructured `{ recipe }` (a pre-built `SpawnRecipe`) and called `spawn_session(recipe, ...)`. Under Model A (daemon-side `spawn_recipe()` execution), the wire payload is `SpawnOptions` (intent), not `SpawnRecipe` (built artifact). Additionally, the signature `spawn_session(recipe: SpawnRecipe, harness_id: String, profile_id: String)` was incoherent: `SpawnRecipe` carries no `harness_id`/`profile_id` fields, and `spawn_session()` cannot call `spawn_recipe()` without `SpawnOptions`. The comment `/* harness_id, profile_id from recipe context */` was a placeholder for a genuinely unresolvable architectural gap.
- **Fix — `SpawnSession` arm updated:** Destructures `{ opts: SpawnOptions }`. The daemon IPC handler fills `opts.session_id` (new UUID) and `opts.hooks_settings_path` (pre-written shared path from `DaemonState`) on receipt using struct update syntax, then calls `SessionManager::spawn_session(opts)`. The three-step `spawn_session()` internals comment is updated to label step 1 "DAEMON-SIDE recipe construction" and explain that `EngineError` codes are REACHABLE because `spawn_recipe()` runs daemon-side.
- Semver: minor (v1.7.0 → v1.8.0) — normative IPC handler pattern change; wire payload type change.

## §Trace v1.7.0

**I12-001 — SpawnSession arm annotated with EngineError call-site + bridge codes** (2026-06-04):

- **Finding (I12-001):** The `SpawnSession` arm in §3 only cited `session_error_to_code(IpcOp::Spawn, &e)`
  with no explanation of how `EngineError::BinaryNotFound` / `EngineError::InvalidPath` reach this
  path. An implementer reading §3 in isolation would not know that `spawn_session()` calls
  `spawn_recipe()` first, or that the two new EngineError-derived codes must be handled.
- **Fix:** `SpawnSession` arm inline comment updated (I12-001 call-site spec): documents the three
  steps inside `spawn_session()` and names the `EngineError` → `SessionError::EngineError` bridge
  and the two resulting `session_error_to_code` mappings. References SS-session-manager.md
  §SessionError taxonomy for the authoritative variant/code table.
- **Summary table updated:** `SpawnSession` row Notes column updated to mention the two new
  EngineError-derived codes and the I12-001 bridge.
- Semver: minor (v1.6.0 → v1.7.0) — normative annotation to call-site and summary table.

## §Trace v1.6.0

**I11-001 PRONG B — explicit scope-boundary note for broadcast fan-out + multi-client deferral anchor** (2026-06-04):

- **Finding (I11-001 PRONG B):** The `ScrollbackChunk*` + `ScrollbackDumpComplete` broadcast
  fan-out in §5b had no explicit statement clarifying that concurrent multi-TUI-client attach to
  the SAME session is a FUTURE (post-v1A) capability. An adversarial reviewer reading the broadcast
  semantics could reasonably conclude that the "attach-reset storm" (one client's AttachSession
  resets all other clients' parsers) is an unaddressed v1A bug. It is not — but that boundary was
  only implicit (the phrase "supports FUTURE multi-TUI scenarios" in BC-2.05.009 Invariant 2 was
  the only anchor, and it is not in this file).
- **Fix — §5b scope-boundary note added:** An explicit scope note in §5b states:
  (a) v1A has exactly one TUI client; the broadcast is correct and harmless for v1A;
  (b) the multi-client attach-reset storm is a real issue deferred to the future multi-client
  capability, tracked as `TD-MULTI-CLIENT-ATTACH-STORM-001`;
  (c) the candidate fix (per-client unicast scrollback) is named but must NOT be speculatively
  implemented — it requires BC updates and session-host concurrency design.
- **Deferral is concrete (per CLAUDE.md):** `TD-MULTI-CLIENT-ATTACH-STORM-001` is the named
  deferral anchor. It will be picked up when the multi-client TUI capability story is scheduled.
  It is NOT "later" or "Phase N" — it is tied to the specific future capability that introduces
  concurrent multi-TUI clients.
- Semver: minor (v1.5.0 → v1.6.0) — adds scope-clarifying normative note.

## §Trace v1.5.0

**I-P8-001/I-P8-002/P8-STRUCTURAL — Pass-8 op-aware error signature + no-silent-failure for all arms** (2026-06-03):

- **I-P8-001 — op-aware `session_error_to_code` signature propagated to all arms:** The `KillSession`
  and `AttachSession` Err branches used the single-argument form `session_error_to_code(&e)`, which
  does not exist in the canonical signature. The canonical signature is `session_error_to_code(op:
  IpcOp, e: &SessionError)` (SS-session-manager.md v1.7.0 §Error handling, lines ~443-456). The
  single-arg form made `SessionHostDead` → `"kill_failed"` unreachable on the kill path (it would
  always fall through to `"attach_failed"` if it compiled at all). Fixed: `KillSession` Err branch
  → `session_error_to_code(IpcOp::Kill, &e)`; `AttachSession` Err branch →
  `session_error_to_code(IpcOp::Attach, &e)`. The `SpawnSession` Err branch was using a hardcoded
  `"spawn_failed"` string literal instead of the function — corrected to
  `session_error_to_code(IpcOp::Spawn, &e)` for consistency and exhaustiveness. An `IpcOp` import
  note added to the §3 preamble.
- **I-P8-002 — bare `?`-propagation eliminated from four arms:** `KeyInput`, `ResizePane`,
  `DetachSession`, and `RenameSession` arms used `.await?`, which propagates `SessionError` out of
  the per-client task (silently dropping the connection) instead of surfacing a structured error.
  This violated BC-2.05.010 §KeyInput PC-4 / EC-281 / EC-283 and SS-session-manager §Error
  handling. Fixed per canonical pattern:
  - `KeyInput` → `match … { Ok(()) => {} Err(e) => client_tx.send(ServerToClient::Error { code:
    session_error_to_code(IpcOp::KeyInput, &e), message }).await }`. Success = no reply
    (fire-and-forget); errors MUST NOT be dropped.
  - `DetachSession` → same match/Error pattern with `IpcOp::Detach`.
  - `RenameSession` → same match/Error pattern with `IpcOp::Rename`.
  - `ResizePane` → `if let Err(e) = … { tracing::warn!(…) }` — WARN-log-and-continue. No
    `ServerToClient::Error` (benign race per ResizePane special rule). Per-client task stays alive.
  All nine `ClientToServer` arms now follow the no-silent-failure discipline. No bare `?` escapes
  a `SessionError` to the per-client task boundary.
- **P8-STRUCTURAL — canonical-pattern lock note added:** A blockquote note at the top of §3
  declares that the error-routing discipline MUST match SS-session-manager.md §Error handling and
  that any future canonical-pattern change in SS-session-manager MUST propagate here in the same
  edit burst. An IpcOp/session_error_to_code import note is included. A per-arm error-handling
  summary table (all nine arms) is added immediately after the code block to make the exhaustive
  discipline visible and reviewable without re-reading the full inline code.

## §Trace v1.4.0

**C6-001/C6-002/I6-002 — Pass-6 error routing + ReAttach removal + stale SessionSnapshot retired** (2026-06-03):

- **C6-002 — ClientToServer::ReAttach removed from §3 IPC handler:** The match arm
  `ClientToServer::AttachSession { session_id } | ClientToServer::ReAttach { session_id }` was
  uncompilable: `ClientToServer::ReAttach` does not exist. I3-004 (v1.3.0) consolidated all
  TUI re-attach onto `AttachSession`; `ReAttach` was never added to the `ClientToServer` enum
  in SS-ipc.md. The `ReAttach` alternative was introduced in v1.3.0 §Trace incorrectly
  (the §Trace v1.3.0 entry `ClientToServer::AttachSession / ReAttach variant added` is a
  historical artifact of the authoring session; the variant was never in the canonical enum).
  Fix: `| ClientToServer::ReAttach { session_id }` alternative deleted; only the
  `ClientToServer::AttachSession { session_id }` arm remains.
- **C6-001 — Error routing added to §3 IPC handler:** All lifecycle operation match arms
  (`SpawnSession`, `KillSession`, `AttachSession`) updated from `?`-propagation to explicit
  `Err(e)` branches that send `ServerToClient::Error { code, message }` to the requesting
  client. This implements the no-silent-failure invariant (BC-2.05.010 / SS-08 §Error
  handling / SS-ipc.md v1.14.0 §ServerToClient::Error).
- **I6-002 — Stale inline SessionSnapshot in §4 retired:** The 10-field inline `SessionSnapshot`
  struct definition omitted `degraded: bool` and `degraded_reason: Option<String>` (added by
  I3-009 / SS-ipc.md v1.12.0). Rather than maintaining a second copy that would continue to
  drift, the inline struct is replaced with an explicit pointer to SS-ipc.md §Supporting Types
  as the single source of truth. The pointer lists all 12 canonical fields for quick reference.

## §Trace v1.3.1

**SUG-002 (architect half) — Adversarial Pass 4 residue: §6 wire type corrected to SessionSnapshot** (2026-06-03):

- **SUG-002 (§6 spawned_by_monocle wire source):** §6 "HTTP hook handler — session correlation"
  previously described the `spawned_by_monocle` badge as reading from `EnrichedSession`. This
  was inconsistent with C3-004 (SS-daemon-wiring-v2-delta §Trace v1.3.0), which changed
  `InitialState.sessions` and `SessionListUpdate.sessions` from `Vec<EnrichedSession>` to
  `Vec<SessionSnapshot>`. The wire type is now `SessionSnapshot`; `EnrichedSession` is the
  internal EngineModule detect() result type only. §6 updated to name
  `SessionSnapshot.spawned_by_monocle` as the wire field, `SessionEntry.spawned_by_monocle`
  as the registry source (daemon-internal), and `EnrichedSession.spawned_by_monocle` as the
  internal detect path only. Three-type provenance now consistent with §4 reconciliation table.

## §Trace v1.3.0

**C3-001/C3-003/C3-004/I3-001/I3-003/I3-004/O3-004 — Adversarial Pass 3 resolution** (2026-06-03):

- **C3-001 (SessionStateChanged emission site):** §3b added: SessionStateChanged emission rule
  specifying the exact code pattern, the complete per-method emission table (which methods emit
  the SessionStateChanged+SessionListUpdate pair vs SessionListUpdate-only), and the ordered-pair
  atomicity guarantee (both posts made inside the SessionManager mutex hold; channel FIFO order
  is the actual ordering mechanism, not the mutex itself). `ClientToServer::AttachSession` /
  `ReAttach` variant added to §3 IPC handler with its SessionStateChanged emission on
  ScrollbackDumpComplete.
- **C3-003 (rename emits wrong message):** `rename_session()` emission clarified in the §3b
  emission table: rename emits `SessionListUpdate` ONLY (not `SessionStateChanged`), because
  rename is not a state transition — `SessionStateChanged` carries `new_state` only and cannot
  convey the new `display_name`. Full rationale included. `ClientToServer::RenameSession` handler
  comment updated to say "publishes SessionListUpdate" (was misleading in prior version).
- **C3-004 (SessionSnapshot undefined):** §4 rewritten. `SessionSnapshot` struct defined with
  all fields (session_id, display_name, state, harness_id, project_root, cwd, spawned_by_monocle,
  started_at_micros, pty_rows, pty_cols). Three-representation reconciliation table added
  (SessionEntry=daemon-internal, EnrichedSession=EngineModule detection, SessionSnapshot=wire).
  `InitialState.sessions` and `SessionListUpdate.sessions` changed from `Vec<EnrichedSession>`
  to `Vec<SessionSnapshot>`. SS-ipc.md update flagged.
- **I3-001 (ordering mechanism corrected):** §3b ordering guarantee text states explicitly that
  the mutex provides the atomicity window for both posts, but the channel FIFO order is the
  actual ordering mechanism. Ordered-pair split behavior specified: split pair (first try_send
  succeeds, second fails) triggers immediate client disconnect (treated as exhausting the
  3-strike threshold). Rationale: half-pair delivery leaves TUI in inconsistent state.
- **I3-003 (dump-pause stall):** §5b "I3-003 fix" added. Session-host resumes live PtyBytes
  immediately after taking the vt100::Screen snapshot — no pause for dump transfer duration.
  TUI buffers live PtyOutput received during dump, replays after ScrollbackDumpComplete. Bounded
  live-byte buffer during dump window specified (~500 KB at 1 MB/s for 500ms dump). ADR-0010
  §Interleaving flagged for update to "snapshot-then-resume" protocol.
- **I3-004 (ClientToServer::AttachSession added):** `ClientToServer::AttachSession { session_id }`
  added to §3 IPC handler. Daemon handles it by calling `SessionManager::attach_session()` which
  issues `DaemonToHost::Attach` to the session-host and streams a fresh ScrollbackDump. Used for
  TUI-initiated re-attach after PtyReset and explicit re-attach. SS-ipc.md ClientToServer enum
  update flagged.
- **O3-004 (buffer sizing corrected):** §5d "O3-004 fix" added. Per-client buffer capacity
  corrected from 256 to 64 messages. Prior "256 × 4096 bytes = 1 MiB" calculation was wrong —
  ScrollbackChunk can be 256 KiB; 256 × 256 KiB = 64 MiB. 64 messages × 256 KiB worst-case =
  16 MiB per client × 4 clients = 64 MiB (within budget). I3-003 interaction noted: resume-after-
  snapshot reduces peak in-flight chunk volume. ADR-0010 §Cross-Client buffer sizing flagged.

## §Trace v1.2.0

**C2-002 + I2-003 — Missing ServerToClient variants + per-client isolation** (2026-06-03):
- **C2-002:** §5 expanded from a single PtyOutput fan-out section to four subsections (5a–5d).
  `ServerToClient::ScrollbackChunk`, `ServerToClient::ScrollbackDumpComplete`, and
  `ServerToClient::PtyReset` added as named broker event types and fan-out paths.
  `HostToDaemon::ScrollbackDump` RETIRED; replaced by chunked `ScrollbackChunk*` +
  `ScrollbackDumpComplete` protocol in both `HostToDaemon` and `ServerToClient` directions.
  `HostToDaemon` enum additions documented in §5b.
- **I2-003:** §5d added: per-client isolated send buffer (capacity 256, `.try_send()` into
  per-client channel, dedicated writer task). Prevents cross-client / cross-session livelock.
  Slow-client disconnect threshold specified. See ADR-0010 §Cross-Client / Cross-Session
  Backpressure Isolation for full analysis.

## §Trace v1.1.0

**C2 — lock-file vs UDS readiness signal separation** (2026-06-03):
- Clarified that the foreground `daemon start` caller's lock-file poll (step 13) fires at
  step 8, BEFORE re-discovery (step 8b). This is correct and intentional: the foreground
  caller does not connect to the UDS socket; only TUI clients do. TUI clients cannot connect
  until step 10 (UDS bind), which follows step 8b (re-discovery). No race exists.
- Added §Dual readiness signals explaining the two-signal architecture explicitly.
- Step 8b description updated to include SO_PEERCRED cross-check and hooks-settings.json
  shared-file model from SS-session-manager.md v1.2.0 C1 decision.
- SS-daemon-wiring.md step 13 is confirmed correct as-is; no change required there.

## §Trace v1.0.1

**IMP-3 step placement correction** (2026-06-03):
- Corrected step 8b placement language. Original text erroneously said "step 9 that currently
  doesn't exist — the UDS bind step." Step 9 in SS-daemon-wiring.md IS the hooks-settings.json
  step (it exists); step 10 IS the UDS bind. The error arose from conflating the delta's
  "new step numbering" intent with the base document's established step numbering.
- Added explicit "Current step numbering" preamble citing SS-daemon-wiring.md as authoritative.
- Clarified insertion invariant: step 8b inserted BEFORE step 9 (hooks-settings.json); the
  "must complete before UDS bind (step 10)" constraint is stated in terms of the real step
  numbers. "What does NOT change" updated to be consistent (steps 1–8 and 9–13; only 8b added).

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- D-235 rework scope specified for v1A architecture delta.
- Covers: DaemonState struct, daemon_start_sequence step 8b, IPC handler new variants,
  InitialState extension, broker PtyOutput fan-out, HTTP hook handler correlation.
- Documents what does NOT change to prevent scope creep.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
