---
document_type: architecture-section
level: L3
section: "session-manager"
subsystem: SS-08
version: "2.3.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T23:00:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - specs/product-brief.md
  - specs/architecture/adr/ADR-0009-native-session-host-process-model.md
  - specs/architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md
  - specs/architecture/SS-daemon-lifecycle.md
  - specs/architecture/SS-ipc.md
  - specs/architecture/SS-engine-module.md
input-hash: "13e1215"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: Session Manager (SS-08)

## Scope

SS-08 defines:

1. **SessionManager** — the daemon-side coordinator in `monocle-runtime` that manages
   session lifecycle, spawns/kills/attaches `monocle-session-host` processes, and proxies
   PTY bytes between session-hosts and TUI clients.
2. **monocle-session-host binary** — the detached per-session supervisor process that owns
   the PTY master, vt100 parser, harness child handle, and exposes a per-session UDS socket.
3. **Session state sidecar** (`session-state.json`) — the per-session metadata file used
   for daemon-restart re-discovery.

This design implements ADR-0009 (native detached session-host) and ADR-0010 (shared UDS
for PTY bytes). Q-2 resolution: lifecycle lives in SessionManager; `EngineModule` provides
`spawn_recipe()` only (see §Q-2 Resolution below).

---

## Q-2 Resolution: EngineModule vs SessionManager lifecycle boundary

**Decision:** `EngineModule::spawn_recipe()` provides the spawn recipe (binary, args, env,
cwd). `SessionManager` owns all lifecycle operations (spawn, attach, detach, kill, rename,
resize). No lifecycle methods on the `EngineModule` trait.

**Rationale:**
- `EngineModule` is the harness abstraction (codemachine-cli gene). It knows how to describe
  a session launch, not how to manage it. Lifecycle management is a daemon infrastructure
  concern, not a harness concern.
- SS-engine-module.md §Purpose states: "struct-level inherent operations" for spawning —
  this has always been the design intent. This delta formalizes it.
- If lifecycle were on the trait, all future harness modules (CodeMachine, WASM plugins)
  would have to implement PTY lifecycle — which they do not need to know about.
- The `PtySpawner` trait provides the testability seam regardless of where lifecycle lives.

---

## SessionManager

### Location and crate

`SessionManager` is a sub-module of `monocle-runtime` crate, at
`crates/monocle-runtime/src/session_manager/mod.rs`. It is NOT a separate crate.

**Rationale (confirming vision self-adjudication):** PTY coordination is intrinsically a
daemon responsibility. SessionManager shares daemon-internal types (DaemonState,
Arc<Broker<Event>>). No other crate depends on SessionManager directly — the proto/IPC wire
is the interface. PtySpawner trait provides the test seam regardless of crate structure.

### session_id type — canonical ruling

**`session_id` is a `String` at all IPC, registry, and AppMode boundaries.** The underlying
value is a UUID (v4), but it is generated once (via `uuid::Uuid::new_v4().to_string()`) and
stored and transported as a `String` everywhere: `SessionEntry.session_id`, `SpawnOptions.session_id`,
`AppMode::EmbeddedTerminal { session_id }`, all IPC message fields, and `session-state.json`.
Callers MUST NOT assume any particular UUID variant or version; treat the value as an opaque
string identifier. This avoids a `uuid` dependency in `monocle-core` (pure types crate) and
eliminates UUID/String conversion friction at IPC/AppMode boundaries.

### SessionManager struct

```rust
/// Daemon-side coordinator for session-host processes.
/// Owned by DaemonState; one instance per daemon process.
pub struct SessionManager {
    /// Active sessions keyed by session ID.
    /// Key type: String (UUID rendered as string — see §session_id type ruling above).
    sessions: HashMap<String, SessionEntry>,
    /// Root directory for session sidecar files and per-session UDS sockets.
    runtime_dir: PathBuf,
    /// Spawner abstraction (RealSessionHostSpawner or MockSessionHostSpawner in tests).
    spawner: Arc<dyn SessionHostSpawner>,
    /// Broker fan-out for PTY bytes and session state changes to TUI clients.
    broker: Arc<Broker<Event>>,
}

/// Per-session entry in the SessionManager registry.
struct SessionEntry {
    session_id: String,
    session_host_pid: u32,
    session_host_socket: PathBuf,       // <runtime_dir>/session-<uuid>.sock
    state: SessionState,
    /// Canonical working directory for the harness child process.
    /// When a git worktree is configured, this is the worktree root.
    /// When no worktree is configured (or the project is not a git repo),
    /// this equals project_root. Always absolute. See §SpawnRecipe integration
    /// for the worktree resolution rules.
    cwd: PathBuf,
    /// The project root passed at session creation (wizard Step 2 selection).
    /// This is always the user-selected project directory, regardless of worktree.
    /// Used for display grouping (sessions panel groups by project_root, not cwd).
    project_root: PathBuf,
    harness_id: String,                 // "claude-code", "codemachine", etc.
    profile_id: String,
    started_at: chrono::DateTime<chrono::Utc>,
    /// I3-002: Absolute kill deadline for sessions in Terminating state.
    /// Written to session-state.json as kill_deadline_unix_ms. None unless state == Terminating.
    kill_deadline: Option<std::time::Instant>,
    /// I3-009: Session-host reported missing critical env vars at startup (HOME, PATH, etc.).
    /// false for healthy sessions; true if session-host sent degraded_env in StateChanged.
    degraded: bool,
    /// I3-009: Human-readable degraded reason (e.g., "Missing env: HOME, PATH").
    degraded_reason: Option<String>,
    /// Active connection to the session-host for PTY byte proxying.
    /// None if daemon is not currently attached (e.g., session-host just discovered).
    host_conn: Option<SessionHostConnection>,
}

/// Per-session connection to the session-host process.
struct SessionHostConnection {
    /// Write half of the per-session UDS connection.
    writer: Arc<Mutex<UnixStream>>,
    /// Background task proxying session-host PTY output to daemon broker.
    proxy_task: JoinHandle<()>,
}
```

### Session lifecycle state machine

```rust
/// Session lifecycle state (adapted from claude-squad A.3 pattern).
/// Serialize/Deserialize for session-state.json sidecar.
///
/// REACHABLE STATES ONLY (I4 audit — pruned unreachable variants):
///   Created — REMOVED: spawn_session() transitions directly to Launching (the OS process
///     is spawned synchronously inside spawn_session()); Created was never persisted or observed.
///   Killed — REMOVED: superseded by Terminating (see below). kill_session() now transitions
///     to Terminating (not Killed), providing observable in-flight kill status without the
///     confusion of "Killed" implying completion.
///
/// I2-004: Terminating transient state added. Without it, a session whose harness child
/// ignores SIGTERM stays in Running state for up to 10 seconds after the user presses kill.
/// This violates production-grade observability: the user cannot tell whether the kill was
/// sent or is still pending. The Terminating state closes this gap.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// session-host process spawned; waiting for its UDS socket to become connectable.
    /// Initial state written to sidecar at spawn time.
    Launching,
    /// session-host alive, daemon attached, PTY streaming to broker.
    Running,
    /// TUI or daemon explicitly detached; session-host still alive; daemon not currently attached.
    /// Daemon can re-attach at any time via attach_session().
    Detached,
    /// kill_session() has been called; DaemonToHost::Kill has been sent to the session-host;
    /// awaiting HostToDaemon::StateChanged { new_state: Terminated } confirmation.
    ///
    /// The TUI renders this as `[Terminating]` (e.g., with a spinner or dimmed indicator).
    /// Lifecycle actions (Kill, Detach, Rename) are disabled for sessions in Terminating state.
    ///
    /// Transitions:
    ///   Running / Detached / Launching → Terminating: kill_session() called.
    ///   Terminating → Terminated: session-host confirms exit via StateChanged::Terminated.
    ///   Terminating → Terminated (timeout): if no StateChanged received within 12 seconds
    ///     (10s SIGTERM window + 2s buffer), the daemon forces the session to Terminated,
    ///     sends SIGKILL to the session-host PID directly, and GCs the sidecar. This prevents
    ///     Terminating state from persisting indefinitely if the session-host itself crashes.
    ///
    /// Re-discovery: if a sidecar is found in Terminating state at daemon restart, the daemon
    /// treats it as Running (sidecar may have been written mid-kill-sequence) and probes
    /// liveness. If alive, it re-sends Kill and waits. If dead, it transitions to Terminated
    /// and GCs the sidecar.
    Terminating,
    /// Harness child exited (naturally or via SIGTERM/SIGKILL from kill_session()); session-host
    /// sent StateChanged::Terminated to daemon. Terminal state — no further transitions.
    Terminated,
}
```

#### State transition table

| From | Event | To | Notes |
|------|-------|----|-------|
| *(none)* | `spawn_session()` called | `Launching` | OS process spawned; sidecar written |
| `Launching` | Daemon receives `StateChanged::Running` from session-host | `Running` | Session-host UDS connectable; proxy task started |
| `Launching` | `ScrollbackDumpComplete` received on re-discovery | `Running` | Re-discovery path: host was already up |
| `Launching` | PID dead on re-discovery probe | `Terminated` | GC sidecar |
| `Launching` | `StateChanged::Terminated` (spawn failed inside session-host) | `Terminated` | GC sidecar |
| `Launching` | `kill_session()` called | `Terminating` | Kill sent; awaiting session-host confirmation |
| `Running` | `detach_session()` called | `Detached` | Proxy task aborted; session-host continues |
| `Running` | `StateChanged::Terminated` from session-host | `Terminated` | Child exited naturally or via SIGTERM |
| `Running` | PID dead on re-discovery | `Terminated` | GC sidecar (should not occur normally) |
| `Running` | `kill_session()` called | `Terminating` | Kill sent; awaiting session-host confirmation |
| `Detached` | `attach_session()` called | `Running` | New proxy task created; ScrollbackDumpComplete received |
| `Detached` | PID dead on re-discovery | `Terminated` | GC sidecar |
| `Detached` | `kill_session()` called | `Terminating` | Fresh UDS connect + Kill sent; awaiting confirmation |
| `Terminating` | `StateChanged::Terminated` from session-host | `Terminated` | Normal kill completion |
| `Terminating` | 12s timeout (no StateChanged received) | `Terminated` | Daemon sends SIGKILL to session-host PID; GC sidecar |
| `Terminating` | PID dead on re-discovery | `Terminated` | GC sidecar |
| `Terminated` | GC timer (10s) | *(removed)* | Entry removed from registry |

#### Re-discovery state handling (I4 — all states covered)

`rediscover_sessions()` probes all sidecar files and handles every persisted state:

- **Sidecar state `Launching`:** The session-host was spawned but the daemon crashed before
  confirming Running. Probe liveness: if alive, attempt `Attach` with 5s timeout. If
  `ScrollbackDumpComplete` received → register as `Running`. If no response within 5s →
  `SIGTERM` session-host; mark `Terminated`; GC sidecar. If process dead → GC sidecar.
- **Sidecar state `Running`:** Normal re-discovery path. Probe liveness; if alive, `Attach`;
  on `ScrollbackDumpComplete` → `Running`. If dead → GC.
- **Sidecar state `Detached`:** Session-host was alive but daemon was not attached. The
  session was intentionally detached — the user's `DetachSession` request resulted in this
  state, and re-discovery MUST respect the persisted intent.
  **I3-005 fix:** Re-discovery restores `Detached` sidecars to `Detached` state (NOT `Running`).
  Probe liveness: if alive, connect to the session-host socket (SO_PEERCRED check), register
  a `SessionEntry` with `state: Detached` and `host_conn: None` (no proxy task, no force-
  attach, no streaming). If dead → GC. The daemon does NOT call `DaemonToHost::Attach` for
  Detached sidecars during re-discovery. The TUI can later initiate an explicit `AttachSession`
  if the user wants to resume streaming.
  Rationale: 8 background Detached sessions force-attached on restart would all become Running
  streamers simultaneously, violating BC-2.08.007 Inv-1 ("Detached sessions don't stream") and
  consuming the per-session proxy task budget unnecessarily. Respecting the persisted Detached
  state is the production-grade behavior.
- **Sidecar state `Terminating`:** Daemon crashed mid-kill-sequence. Probe liveness:
  - If dead → mark `Terminated`; GC sidecar.
  - If alive: **I3-002 fix — watchdog as post-UDS-bind background task.** The 5s re-discovery
    budget (BC-2.08.004 PC-7, `tokio::join_all` of all sessions) MUST NOT wait for the 12s
    Terminating watchdog. The 12s watchdog deadline is 2.4× the re-discovery budget — allowing
    it to block re-discovery would violate BC-2.08.004 Invariant 1 (UDS bind blocked until
    re-discovery complete). Instead:
    1. Probe liveness, verify SO_PEERCRED, register `SessionEntry` with `state: Terminating`
       and `host_conn: None` (no proxy task — it's being killed).
    2. Re-send `DaemonToHost::Kill` over a fresh SO_PEERCRED-verified UDS connect (fire-and-forget,
       do NOT wait for `StateChanged::Terminated`).
    3. Spawn a BACKGROUND watchdog tokio task that waits up to 12s for `StateChanged::Terminated`
       from the session-host. If Terminated received → GC sidecar. If 12s elapses → SIGKILL
       session-host PID; GC sidecar.
    4. Return from the re-discovery probe IMMEDIATELY (the background watchdog is now detached).
    The re-discovery join_all counts this session as found_alive (state: Terminating) and moves on.
    **Flapping-daemon kill-deadline persistence (I3-002):** To prevent repeated daemon restarts
    from resetting the kill escalation indefinitely, `SessionEntry` for a `Terminating` session
    carries a `kill_deadline: Instant` field (absolute deadline, not relative). The deadline is
    read from the sidecar (see schema below) on re-discovery. If the sidecar's recorded
    `kill_deadline_unix_ms` has already elapsed, the daemon immediately sends SIGKILL (skips
    the 12s SIGTERM window — it already expired). This prevents a harness child that ignores
    SIGTERM from surviving multiple daemon restart cycles.
    **SessionEntry and session-state.json additions for kill_deadline:**
    - `SessionEntry` gains `kill_deadline: Option<Instant>` — `Some` only when `state == Terminating`.
    - `session-state.json` gains an optional `kill_deadline_unix_ms: Option<u64>` field (Unix
      epoch milliseconds). Written by `kill_session()` when transitioning to `Terminating`; read
      back on re-discovery. When present and elapsed at re-discovery time → immediate SIGKILL.
      When present and not yet elapsed → 12s watchdog uses `kill_deadline_unix_ms` as the
      absolute deadline (not a new 12s window from restart time).
- **Sidecar state `Terminated`:** Should not appear (session-host deletes sidecar on clean
  exit; GC removes it on timer). If found: delete sidecar; skip. Treat as GC cleanup of a
  crash-leftover sidecar.
- **Unknown state string (forward-compat):** Log WARN; skip; delete sidecar.

### Pre-socket-bind orphan kill (I6)

When `spawn_session()` spawns the session-host OS process but fails BEFORE the session-host
has bound its UDS socket (e.g., sidecar write fails at step 8 of the session-host startup,
which comes AFTER process spawn), the daemon cannot use `DaemonToHost::Kill` over the
per-session UDS because the socket does not exist yet.

**Mandatory fallback: PID-based SIGTERM/SIGKILL.**

The `SpawnedHostHandle.pid` returned by `SessionHostSpawner::spawn()` MUST be used:

```rust
// On sidecar write failure after OS process is already running:
nix::sys::signal::kill(Pid::from_raw(pid), Signal::SIGTERM)?;
// Wait up to 2s for exit; if still running:
nix::sys::signal::kill(Pid::from_raw(pid), Signal::SIGKILL)?;
```

**Escalation deadline: 2 seconds (pre-socket-bind orphan, not normal kill).**
The 2s SIGTERM→SIGKILL deadline applies exclusively to the pre-socket-bind orphan path
(spawn-failure cleanup). The rationale: the session-host process has just been spawned and
has not yet written its sidecar or bound its socket — it is not running user workload. A 2s
window is sufficient for any startup-path initialization code to observe the signal.

**This is distinct from the normal kill-path deadline (10 seconds):**
`kill_session()` on a Running/Detached/Launching session-host sends `DaemonToHost::Kill`
over the per-session UDS. The session-host sends SIGTERM to the harness child and waits up
to 10 seconds before sending SIGKILL (BC-2.08.003 Invariant 4). The 10s window is justified
by the harness child (Claude Code) needing time for clean shutdown — flushing tool output,
saving state, removing temp files. These two kill paths are fundamentally different:
- **Pre-socket-bind orphan kill (2s):** no user workload; clean shutdown irrelevant.
- **Normal kill via DaemonToHost::Kill (10s):** harness child is running; clean shutdown needed.

Both deadlines are explicit and justified. There is no inconsistency.

This is the only correct path when the socket is not yet bound. This fallback applies
to ALL spawn-path failures that occur after `SessionHostSpawner::spawn()` returns `Ok`
but before `spawn_session()` returns to the caller:
- Sidecar write failure (EC-151 in BC-2.08.001)
- `session_id` insertion collision on second retry
- Any other post-spawn validation failure

**Integration test:** `test_spawn_session_orphan_kill_on_sidecar_failure` — verifies that
if `MockSessionHostSpawner::spawn()` succeeds but the sidecar write is injected to fail,
the mock session-host PID is SIGTERMed and no `SessionEntry` leaks into the registry.

### Public API

```rust
impl SessionManager {
    /// Spawn a new session from the given parameters.
    ///
    /// Internally, `spawn_session()` calls `engine_module.spawn_recipe(&opts)?` as its
    /// FIRST step to obtain a `SpawnRecipe` from the EngineModule. If `spawn_recipe()` fails
    /// (e.g., `EngineError::BinaryNotFound` — harness not on PATH; or `EngineError::InvalidPath`
    /// — invalid hooks settings path), the error is converted to `SessionError::EngineError`
    /// via `From<EngineError>` and returned before any OS process is spawned. The IPC handler
    /// maps it to the appropriate `ServerToClient::Error` code via `session_error_to_code()`.
    ///
    /// The `SpawnOptions` payload is received directly from the TUI via
    /// `ClientToServer::SpawnSession { opts }` over the shared UDS IPC channel. All spawn
    /// parameters originate TUI-side (from the SessionCreation wizard) and are transmitted
    /// to the daemon as `SpawnOptions`; the daemon builds the `SpawnRecipe` from them.
    ///
    /// Returns the session_id of the new session.
    pub async fn spawn_session(
        &mut self,
        opts: SpawnOptions,
    ) -> Result<String, SessionError>;

    /// Kill a running session (SIGTERM to session-host; session-host kills harness child).
    pub async fn kill_session(&mut self, session_id: &str) -> Result<(), SessionError>;

    /// Detach the daemon from a running session-host (disconnect proxy; session continues).
    pub async fn detach_session(&mut self, session_id: &str) -> Result<(), SessionError>;

    /// Re-attach the daemon to a running session-host (reconnect proxy).
    pub async fn attach_session(&mut self, session_id: &str) -> Result<(), SessionError>;

    /// Rename a session (updates display_name in sidecar; publishes SessionListUpdate to broker).
    ///
    /// Rename is NOT a SessionState transition. SessionStateChanged carries `new_state: SessionState`
    /// only and cannot convey the updated display_name. The correct broadcast is SessionListUpdate,
    /// which carries the full SessionSnapshot including the new display_name. SessionStateChanged
    /// is NOT emitted. See C3-003 / SS-daemon-wiring-v2-delta §3b emission table.
    pub async fn rename_session(&mut self, session_id: &str, new_name: String) -> Result<(), SessionError>;

    /// Resize the PTY for a session (forwards to session-host).
    pub async fn resize_session(&mut self, session_id: &str, rows: u16, cols: u16) -> Result<(), SessionError>;

    /// Forward keyboard bytes to a session's PTY stdin.
    pub async fn send_key_input(&mut self, session_id: &str, bytes: Vec<u8>) -> Result<(), SessionError>;

    /// Re-discover session-hosts from sidecar files on daemon startup.
    /// Probes liveness, attaches to alive ones, marks stale ones Terminated, GC orphaned sidecars.
    pub async fn rediscover_sessions(&mut self) -> Result<RediscoveryReport, SessionError>;

    /// Return current session list (for InitialState IPC push).
    pub fn session_list(&self) -> Vec<SessionSnapshot>;
}
```

<a id="error-handling-sessionerror-servertoclient-error-mapping"></a>
### §Error handling — SessionError → ServerToClient::Error mapping

Every `SessionManager` method in the Public API returns `Result<_, SessionError>`. The daemon IPC
handler (`monocle-runtime/src/ipc_handler.rs`) MUST map every `Err(SessionError::...)` return to
a `ServerToClient::Error { code, message }` sent to the requesting TUI client over its per-client
channel. **No `Err` may be silently swallowed at the task boundary** (return `Ok(())` after an
error) — doing so leaves the TUI hung in `Launching` state (or another stale state) with no
user-visible feedback, which is a user-visible silent failure (BC-2.05.010 §No-silent-failure invariant).

#### SessionError taxonomy

```rust
/// Errors returned by SessionManager lifecycle methods.
///
/// These map to ServerToClient::Error.code values per the v1A error code taxonomy
/// in SS-ipc.md §ServerToClient::Error.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session not found: {session_id}")]
    SessionNotFound { session_id: String },
    #[error("spawn failed: {reason}")]
    SpawnFailed { reason: String },
    /// Sidecar file could not be written after the session-host OS process was already
    /// spawned. The orphan-kill protocol (§Pre-socket-bind orphan kill) MUST run before
    /// this error is returned so the spawned process is cleaned up. Error code:
    /// `"sidecar_write_failed"`.
    #[error("sidecar write failed at {path}: {reason}")]
    SidecarWriteFailed { path: String, reason: String },
    /// A session with the generated session_id already exists in the registry. This is
    /// a UUID v4 collision — astronomically rare but must be handled. The spawn is
    /// aborted; the caller should not retry automatically. Error code:
    /// `"session_id_collision"`.
    #[error("session_id collision: {session_id}")]
    SessionIdCollision { session_id: String },
    #[error("session host dead: {session_id}")]
    SessionHostDead { session_id: String },
    #[error("invalid session name: {reason}")]
    InvalidSessionName { reason: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// An EngineModule operation failed before the OS process was spawned.
    /// Covers `EngineError::BinaryNotFound` (→ `"binary_not_found"`) and
    /// `EngineError::InvalidPath` (→ `"invalid_spawn_arg"`).
    /// Added by I12-001 to bridge EngineError into the SessionError taxonomy
    /// so BC-2.03.007 PC-3/PC-7 distinct user-visible diagnostics are satisfiable.
    #[error("engine error: {0}")]
    EngineError(#[from] monocle_core::engine::EngineError),
}
```

`From<EngineError> for SessionError` is derived automatically via the `#[from]` attribute above.
The bridge is ONLY for spawn-path use: `spawn_session()` calls `engine_module.spawn_recipe(opts)`
and uses `?` to propagate any `EngineError` as `SessionError::EngineError`. No other lifecycle
method calls EngineModule methods, so `SessionError::EngineError` can only be produced by
`spawn_session()`.

#### Mapping table (SessionError → ServerToClient::Error.code)

| SessionError variant | Triggering lifecycle method(s) | code | Notes |
|----------------------|-------------------------------|------|-------|
| `EngineError(BinaryNotFound)` | `spawn_session` (via `spawn_recipe()`) | `"binary_not_found"` | Harness binary not found on PATH; `which::which()` failure |
| `EngineError(InvalidPath)` | `spawn_session` (via `spawn_recipe()`) | `"invalid_spawn_arg"` | Non-UTF-8 or null-byte argument to `spawn_recipe()` |
| `SessionNotFound` | `kill_session`, `detach_session`, `attach_session`, `rename_session`, `send_key_input`, `resize_session` | `"session_not_found"` | Session ID not in registry |
| `SpawnFailed` | `spawn_session` | `"spawn_failed"` | OS process spawn failure (from spawner) |
| `SidecarWriteFailed` | `spawn_session` | `"sidecar_write_failed"` | Sidecar write failed after OS process spawned; orphan-kill protocol runs before this error surfaces |
| `SessionIdCollision` | `spawn_session` | `"session_id_collision"` | UUID v4 collision in registry; astronomically rare; do not auto-retry |
| `SessionHostDead` (attach-path) | `attach_session` | `"attach_failed"` | Session-host PID dead when daemon attempts attach |
| `SessionHostDead` (kill-path) | `kill_session` | `"kill_failed"` | Session-host PID dead when daemon attempts kill; see `session_error_to_code(Op, &SessionError)` |
| `InvalidSessionName` | `rename_session` | `"rename_failed"` | Empty name or name exceeding length limit |
| `Io` | Any | `"invalid_request"` | Unexpected I/O error; nearest generic failure code |
| `EngineError` (other variants) | `spawn_session` | `"invalid_request"` | Catch-all for any future EngineError variants not explicitly mapped; `"invalid_request"` is the nearest generic code |

**Exhaustiveness and forward-compatibility:** `session_error_to_code()` has two match layers with
distinct exhaustiveness guarantees:

- **OUTER `SessionError` match — compiler-enforced exhaustive:** `SessionError` is defined in
  `monocle-runtime` (same crate as `session_error_to_code()`). It does NOT carry `#[non_exhaustive]`.
  The compiler enforces full variant coverage; no `_ =>` arm exists on the outer match. Any new
  `SessionError` variant added in the future will produce a compile error here, forcing conscious
  routing.

- **INNER `EngineError` match — `_ =>` arm is MANDATORY and CORRECT:** `EngineError` is defined
  in `monocle-core` and carries `#[non_exhaustive]` for Phase 3 WASM engine forward-compatibility
  (see SS-engine-module-v2-delta.md §EngineError). Because `session_error_to_code()` lives in
  `monocle-runtime` — a DIFFERENT crate — Rust requires a `_ =>` arm on any match over a
  `#[non_exhaustive]` enum from another crate; the compiler will reject the code without it. The
  `_ => "invalid_request"` arm is therefore not a silent swallow: it is a deliberate,
  forward-compatible fallback that maps any future `EngineError` variants (added in Phase 3 for
  WASM engine modules) to the well-defined `"invalid_request"` wire code. No diagnostic information
  is silently lost: `BinaryNotFound` and `InvalidPath` — the only variants that carry
  spawn-meaningful distinction — are explicitly routed before the catch-all. Any truly unknown
  future variant produces a deterministic, observable `ServerToClient::Error { code: "invalid_request" }`
  that is logged and sent to the requesting client (never dropped).

#### IPC handler pattern (mandatory)

```rust
/// The operation context passed to `session_error_to_code` so that
/// `SessionHostDead` can map to the correct user-visible code.
/// Each variant corresponds to one IPC lifecycle request kind.
#[derive(Debug, Clone, Copy)]
pub enum IpcOp {
    Spawn,
    Kill,
    Attach,
    Detach,
    Rename,
    KeyInput,
    Resize,
}

/// Maps a SessionError to its v1A IPC error code.
/// The `op` context is required to distinguish `SessionHostDead` on the
/// kill-path (`"kill_failed"`) from the attach-path (`"attach_failed"`).
///
/// **Exhaustiveness model (two layers):**
/// - OUTER `SessionError` match: compiler-enforced exhaustive (no `_ =>`).
///   `SessionError` is same-crate; Rust requires full coverage. Any new
///   `SessionError` variant added in the future will produce a compile error here.
/// - INNER `EngineError` match: `_ =>` arm is MANDATORY. `EngineError` is
///   `#[non_exhaustive]` (defined in `monocle-core`); Rust requires a `_ =>` arm
///   on any cross-crate match over a `#[non_exhaustive]` enum. The
///   `_ => "invalid_request"` arm is a deliberate, documented forward-compat
///   fallback for future WASM engine variants — not a silent swallow.
///
/// EngineError variants are unpacked to produce distinct spawn-path codes:
/// - `EngineError::BinaryNotFound` → `"binary_not_found"` (harness not on PATH)
/// - `EngineError::InvalidPath` → `"invalid_spawn_arg"` (bad argument to spawn_recipe())
/// - all other `EngineError` variants → `"invalid_request"` (mandatory `_=>` forward-compat fallback)
fn session_error_to_code(op: IpcOp, e: &SessionError) -> &'static str {
    match e {
        SessionError::EngineError(engine_err) => match engine_err {
            monocle_core::engine::EngineError::BinaryNotFound(_) => "binary_not_found",
            monocle_core::engine::EngineError::InvalidPath(_)    => "invalid_spawn_arg",
            _                                                     => "invalid_request",
        },
        SessionError::SessionNotFound { .. }     => "session_not_found",
        SessionError::SpawnFailed { .. }          => "spawn_failed",
        SessionError::SidecarWriteFailed { .. }   => "sidecar_write_failed",
        SessionError::SessionIdCollision { .. }   => "session_id_collision",
        SessionError::SessionHostDead { .. } => match op {
            IpcOp::Kill               => "kill_failed",
            _                         => "attach_failed",
        },
        SessionError::InvalidSessionName { .. }  => "rename_failed",
        SessionError::Io(_)                       => "invalid_request",
    }
}

// In the per-client IPC message handler task:
match msg {
    ClientToServer::SpawnSession { opts } => {
        // F-P41-IMP-001 resolution: UUID generation and SpawnAck happen in the IPC handler,
        // BEFORE spawn_session() is called. This is the canonical UUID-generation locus.
        // BC-2.08.001 PC-1 language ("inside spawn_session()") was incorrect; the UUID is
        // generated here and delivered via SpawnAck before spawn_session() is invoked.
        let session_id = uuid::Uuid::new_v4().to_string();
        // Step 1: send SpawnAck to requesting client ONLY (not broadcast).
        // The wizard stores this id in AppMode::SessionCreation::launching_session_id for
        // deterministic EC-303 session_id filtering (see SS-embedded-pty.md §Session Creation Wizard).
        let _ = client_tx.send(ServerToClient::SpawnAck {
            session_id: session_id.clone(),
        }).await;
        // Step 2: fill daemon-owned fields.
        let opts = opts.with_daemon_fields(session_id, state.hooks_settings_path.clone());
        // Step 3: call spawn_session() with the completed SpawnOptions.
        match state.session_manager.lock().await.spawn_session(opts).await {
            Ok(_session_id) => { /* broker emits SessionStateChanged{Launching} + SessionListUpdate */ }
            Err(e) => {
                // MUST NOT swallow — send error to requesting client only.
                // NOTE: TUI has already received SpawnAck; it MUST clear launching_session_id
                // (set to None in AppMode::SessionCreation) on receipt of this Error
                // (spawn failed; wizard returns to ProfilePicker).
                let _ = client_tx.send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::Spawn, &e).to_string(),
                    message: e.to_string(),
                }).await;
            }
        }
    }
    ClientToServer::KillSession { session_id } => {
        match state.session_manager.lock().await.kill_session(&session_id).await {
            Ok(()) => { /* broker emits SessionStateChanged{Terminating} + SessionListUpdate */ }
            Err(e) => {
                let _ = client_tx.send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::Kill, &e).to_string(),
                    message: e.to_string(),
                }).await;
            }
        }
    }
    ClientToServer::AttachSession { session_id } => {
        match state.session_manager.lock().await.attach_session(&session_id).await {
            Ok(()) => { /* broker streams ScrollbackChunk* + ScrollbackDumpComplete */ }
            Err(e) => {
                let _ = client_tx.send(ServerToClient::Error {
                    code: session_error_to_code(IpcOp::Attach, &e).to_string(),
                    message: e.to_string(),
                }).await;
            }
        }
    }
    // … same pattern for RenameSession (IpcOp::Rename), KeyInput (IpcOp::KeyInput) …
}
```

**KeyInput special rule:** `KeyInput` failures (session not found or Terminated) MUST still send
`ServerToClient::Error`. The BC-2.05.010 specification states `KeyInput` is fire-and-forget with
no acknowledgement on success — this does NOT mean errors are dropped. Success = no reply;
failure = `ServerToClient::Error`.

**ResizePane special rule:** `ResizePane` failures are silently dropped per BC-2.05.010 (no
`ServerToClient::Error` for resize failures). The daemon clamps zero dimensions (Invariant 5
in BC-2.05.010) before calling `resize_session()`, so the only remaining failure path is a
session-not-found error — which is benign (the session may have terminated between the TUI
sending the resize and the daemon processing it). Resize errors are logged at WARN level only.

### SessionHostSpawner trait

```rust
/// Test seam for session-host process spawning.
/// Mirrors the PtySpawner concept from claude-squad A.5 pattern.
pub trait SessionHostSpawner: Send + Sync + 'static {
    /// Spawn a monocle-session-host process with the given session ID and recipe.
    /// Returns the child PID and expected socket path.
    async fn spawn(
        &self,
        session_id: &str,
        recipe: &SpawnRecipe,
        runtime_dir: &Path,
    ) -> Result<SpawnedHostHandle, SessionError>;
}

pub struct SpawnedHostHandle {
    pub pid: u32,
    pub socket_path: PathBuf,
}

/// Production implementation: spawns monocle-session-host via std::process::Command
/// with pre_exec setsid() so the child becomes a process group leader immune to
/// SIGHUP when the daemon exits.
pub struct RealSessionHostSpawner {
    /// Absolute path to the monocle-session-host binary.
    /// Resolved via std::env::current_exe().parent() at daemon startup.
    session_host_bin: PathBuf,
}

/// Test double: spawns an in-memory mock session host.
pub struct MockSessionHostSpawner { /* ... */ }
```

---

## monocle-session-host binary

### Crate

`crates/monocle-session-host/` — new binary crate in the workspace.

**Process model:** The session-host is a minimal single-tokio-runtime binary. It does NOT
share any code with the daemon's async runtime (to keep it independent). It has its own
`Cargo.toml` and is built as a separate binary packaged alongside `monocle` in the release bundle.

### startup sequence

1. Parse CLI args: `--session-id <uuid>  --runtime-dir <path>  --binary <path>  --args <JSON>  --env <JSON>  --cwd <path>`.
2. Call `nix::unistd::setsid()` to become a process group leader.
3. Open PTY pair via `portable-pty::openpty(PtySize { rows: 24, cols: 80, ... })`.
4. Build `CommandBuilder` from `--binary/--args/--env/--cwd`:
   - **Env inheritance (I2-006 fix — mandatory):** The session-host process inherits its OWN
     environment from its parent (the daemon), which itself inherits from the user's shell.
     `CommandBuilder` for the harness child MUST inherit the session-host process environment
     FIRST, then overlay the recipe's `--env` fields on top. The `portable-pty` `CommandBuilder`
     API populates the environment via `env()` calls — if `CommandBuilder` does NOT inherit the
     parent env by default (check the `portable-pty` 0.9.x API), the session-host MUST
     explicitly seed the builder with `std::env::vars()` before calling `cmd.env()` for each
     recipe env var.
   - **Why this matters:** Without env inheritance, the harness child launches with NO `PATH`,
     NO `HOME`, and NO `TERM`. Claude Code's hook JS calls `os.homedir()` (BC-HOOK-029
     requires this for `~/.monocle/` path resolution) — `os.homedir()` on most platforms reads
     `HOME` from the env. A missing `PATH` causes the child to fail to resolve sub-processes.
     A missing `HOME` causes node's `os.homedir()` to return `undefined` or `/root`, breaking
     hook behavior silently.
   - **I3-009 fix — degraded-env surfaced to daemon (not stderr-only):** If the inherited
     env does not contain `HOME` or `PATH` (edge case: the daemon was launched in a degenerate
     environment), the session-host MUST report the degraded environment condition to the daemon
     as part of the `HostToDaemon` startup handshake so it appears in the sessions panel — not
     only as a WARN to stderr (which has no TUI surface). The mechanism:
     1. `HostToDaemon::StateChanged { new_state: SessionState }` is EXTENDED with an optional
        `degraded_env: Option<Vec<String>>` field listing the missing critical env vars (e.g.,
        `["HOME", "PATH"]`). When missing vars are detected, `StateChanged { new_state: Launching,
        degraded_env: Some(vec!["HOME"]) }` is sent as the first message to the daemon.
     2. The daemon, on receiving `StateChanged` with `degraded_env: Some(_)`, sets
        `SessionEntry.degraded: true` and includes the degraded state in the `SessionSnapshot`
        published via `SessionListUpdate`. The sessions panel renders a warning indicator
        (e.g., `[!]` badge or amber state color) for degraded sessions.
     3. The session-host also logs `WARN` to its stderr (belt-and-suspenders), but the TUI
        surface is the PRIMARY user-visible signal.
     **`SessionEntry` additions:** `degraded: bool` (default false) and `degraded_reason: Option<String>`.
     **`SessionSnapshot` additions:** `degraded: bool` and `degraded_reason: Option<String>`.
     **`HostToDaemon::StateChanged` extension:**
     ```rust
     StateChanged {
         new_state: SessionState,
         /// Missing critical env vars detected at session-host startup. None when env is healthy.
         degraded_env: Option<Vec<String>>,
     }
     ```
     This is a BACKWARD-COMPATIBLE extension: the daemon MUST treat a `StateChanged` with
     no `degraded_env` field (from session-hosts that don't send it yet) as `degraded_env: None`.
     The `#[serde(default)]` derive handles this. No protocol version bump required.
5. Spawn harness child on PTY slave.
6. Initialize `vt100::Parser` with initial size.
7. Bind per-session UDS socket at `<runtime_dir>/session-<session_id>.sock` (mode `0o600`).
8. Write `session-state.json` sidecar at `<runtime_dir>/session-<session_id>.json`.
9. Enter main event loop.

### session-state.json schema

```json
{
  "schema_version": 3,
  "session_id": "<uuid>",
  "pid": 12345,
  "socket_path": "<runtime_dir>/session-<uuid>.sock",
  "child_pid": 12346,
  "state": "Running",
  "project_root": "/path/to/project",
  "cwd": "/path/to/project/worktree-feature-branch",
  "harness_id": "claude-code",
  "profile_id": "default",
  "started_at": "2026-06-03T23:00:00Z",
  "display_name": "monocle — phase0",
  "pty_rows": 24,
  "pty_cols": 80,
  "kill_deadline_unix_ms": null
}
```

Schema version history:
- `schema_version` 1: original fields (no `cwd` field). Read as `cwd = project_root`.
- `schema_version` 2: adds `cwd` field (resolved worktree root; may equal `project_root`).
- `schema_version` 3: adds `kill_deadline_unix_ms: Option<u64>` (I3-002 fix). Written when
  `state = "Terminating"` to record the absolute kill deadline as Unix epoch milliseconds.
  `null` when not in Terminating state. On re-discovery of a `Terminating` sidecar: if
  `kill_deadline_unix_ms` is present and has elapsed → immediate SIGKILL (SIGTERM window
  expired across daemon restart); if present and not elapsed → 12s watchdog uses this as the
  absolute deadline rather than resetting to a new 12s window.

`schema_version` MUST be checked on read:
- Version 1: `cwd = project_root`, `kill_deadline_unix_ms = null`.
- Version 2: `kill_deadline_unix_ms = null`.
- Version 3: full schema.
- Unknown versions beyond 3: log WARN; skip sidecar (forward-compat).

### Per-session UDS security (I5)

The per-session UDS at `<runtime_dir>/session-<uuid>.sock` uses `0o600` permissions
on the socket file and `0o700` on the `runtime_dir`. These filesystem permissions are
necessary but not sufficient: the daemon can be tricked via a rogue sidecar file that
names an attacker-controlled socket path, enabling keystroke injection into a user's
Claude Code session via `DaemonToHost::KeyInput`.

**Required security controls:**

1. **SO_PEERCRED peer-credential check (mandatory on EVERY per-session UDS connect):**
   The SO_PEERCRED / LOCAL_PEERPID uid check applies universally — it is NOT restricted to
   attach or re-discovery. Every per-session UDS connection attempt (attach, re-discovery,
   AND kill/detach re-connect for Detached sessions per BC-2.08.003 EC-164) MUST verify the
   connecting peer's `uid` before reading any bytes.

   - On Linux: `nix::sys::socket::getsockopt::<nix::sys::socket::sockopt::PeerCredentials>(fd)`.
   - On macOS: `nix::sys::socket::getsockopt::<nix::sys::socket::sockopt::LocalPeerPid>(fd)`
     followed by `getpwuid` to resolve the pid to a uid.
   - The connection MUST be rejected (socket closed, operation aborted) if the peer `uid`
     differs from the current process `uid`.
   - **Kill-path specific:** `kill_session()` for a `Detached` session must fresh-connect to
     the session-host UDS (EC-164 path). This fresh connect MUST apply SO_PEERCRED before
     sending `DaemonToHost::Kill`. Failure to apply SO_PEERCRED on the kill-path fresh
     connect would expose the same keystroke-injection vector as attach. No exceptions.

2. **Sidecar trust validation on re-discovery (mandatory):** During `rediscover_sessions()`,
   after reading a sidecar file and connecting to the session-host socket, the daemon MUST
   cross-check:
   - The pid in the sidecar (`sidecar.pid`) matches the actual socket peer pid (from
     SO_PEERCRED/LOCAL_PEERPID on the connected socket).
   - If they differ, the sidecar is stale or spoofed. Log WARN; close the connection;
     send SIGTERM to both the sidecar pid and the socket peer pid (belt-and-suspenders);
     delete the sidecar; do NOT register the session.

3. **Per-session token (not required for v1A):** Given that SO_PEERCRED + sidecar cross-check
   provide same-uid guarantees, a separate cryptographic token on the per-session UDS is
   not required for v1A. The threat model is same-host multi-user (different UIDs); SO_PEERCRED
   closes this. An in-process same-uid attacker who can craft a sidecar file could also
   read the token from the file. Document this decision in the §Risk Mitigations section.
   Security-reviewer must validate this design at implementation (per CLAUDE.md routing rules).

### Per-session UDS protocol

The per-session UDS uses the same length-prefix framing as the daemon UDS (4-byte LE u32 +
JSON payload, 256 KiB max). Messages:

```rust
/// Messages the daemon sends to the session-host.
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonToHost {
    /// Request current scrollback + live-stream subscription.
    Attach,
    /// Send keyboard bytes to PTY stdin.
    KeyInput { bytes: Vec<u8> },
    /// Resize the PTY.
    Resize { rows: u16, cols: u16 },
    /// Request graceful shutdown (SIGTERM child, clean exit).
    Kill,
    /// Detach (daemon is disconnecting; session continues).
    Detach,
}

/// Messages the session-host sends to the daemon.
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostToDaemon {
    /// One chunk of the scrollback dump stream.
    /// Sent in response to DaemonToHost::Attach. Multiple ScrollbackChunk messages are sent
    /// followed by a single ScrollbackDumpComplete sentinel. The session-host resumes live
    /// PtyBytes forwarding IMMEDIATELY after taking the vt100::Screen snapshot — it does NOT
    /// pause during the dump transfer. Live PtyBytes continue to arrive during the dump; the
    /// TUI buffers them and replays after ScrollbackDumpComplete (I3-003 / ADR-0010 §Interleaving).
    /// `ScrollbackDump` (single-message form) is RETIRED; use ScrollbackChunk* + Complete.
    ScrollbackChunk {
        /// Serialized vt100::Screen cells for this chunk: row-major, each cell is
        /// (char, fg, bg, attrs). Rows in this chunk, oldest-to-newest (continuing
        /// from the previous chunk). Each chunk MUST be ≤ 256 KiB serialized.
        rows: Vec<Vec<SerializedCell>>,
        /// 0-indexed chunk sequence number. Non-contiguous sequence → daemon logs WARN
        /// and re-requests Attach.
        chunk_seq: u32,
    },
    /// Sentinel terminating the scrollback dump stream.
    /// Sent after the last ScrollbackChunk. Contains cursor and PTY dimensions at
    /// the moment the dump snapshot was taken. After sending this, the session-host
    /// resumes forwarding live PtyBytes.
    ScrollbackDumpComplete {
        total_chunks: u32,
        cursor_row: u16,
        cursor_col: u16,
        /// Current PTY dimensions (rows × cols) at time of dump.
        pty_rows: u16,
        pty_cols: u16,
    },
    /// Live PTY output bytes. NOT sent during an active scrollback dump transfer.
    PtyBytes { bytes: Vec<u8> },
    /// Session state changed (child exited, etc.).
    /// I3-009: extended with optional degraded_env field. Serde default = None for
    /// backward-compat with session-hosts that don't populate this field.
    /// NOTE: #[serde(default)] belongs at the FIELD level only (not variant level);
    /// the field-level attribute on degraded_env below is the correct and sufficient
    /// mechanism for backward-compat deserialization (S-P7-001 fix).
    StateChanged {
        new_state: SessionState,
        /// Missing critical env vars detected at startup (e.g., ["HOME", "PATH"]).
        /// None when env is healthy. Some when degraded at spawn time.
        #[serde(default)]
        degraded_env: Option<Vec<String>>,
    },
    /// Session-host is shutting down.
    Goodbye,
    /// PTY byte drop detected (channel sender returned Err). See §PTY reader thread.
    /// Daemon propagates as ServerToClient::PtyReset to TUI clients.
    PtyReset,
}

// C5-002 (SS-ipc.md v1.21.0): SerializedCell and SerializedColor are defined in
// monocle-ipc (crate::ipc::SerializedCell / crate::ipc::SerializedColor) so both
// monocle-session-host (writer) and monocle-tui (reader) share the type without a
// cross-binary dependency. The canonical definition with full field documentation
// and the vt100 0.16 5-flag bitmask verification lives in SS-ipc.md §Supporting Types.
//
// This crate uses: use monocle_ipc::{SerializedCell, SerializedColor};
//
// Field summary (for inline reference; SS-ipc.md §Supporting Types is authoritative):
//   SerializedCell { ch: String, fg: SerializedColor, bg: SerializedColor, attrs: u8 }
//   SerializedColor { Default, Ansi(u8), Rgb(u8, u8, u8) }
//   attrs bitmask: bit0=bold, bit1=dim, bit2=italic, bit3=underline, bit4=inverse (vt100 0.16 verified)
```

<a id="screen-state-transfer-on-attach"></a>
### Screen-state transfer on Attach (C5 — correct ScrollbackChunk protocol)

When the session-host handles `DaemonToHost::Attach`, it MUST transfer the vt100 screen
state correctly so the TUI can reconstruct the current terminal without double-applying
PTY bytes.

**Correct protocol:**

1. **Snapshot the parser state** (not raw bytes, not string rows): Serialize the current
   `vt100::Screen` as `Vec<Vec<SerializedCell>>` — styled cells with character, fg color,
   bg color, and attribute flags. This preserves full visual fidelity (colors, bold, etc.)
   that is lost by `String`-only row serialization.

2. **Resume live PtyBytes forwarding IMMEDIATELY** after taking the snapshot (I3-003 fix —
   see SS-daemon-wiring-v2-delta §5b). Do NOT pause for the dump transfer. New PTY bytes
   continue to flow as `HostToDaemon::PtyBytes` while the dump streams. The TUI buffers live
   `PtyOutput` received during the dump and replays after `ScrollbackDumpComplete`.

3. **Stream `HostToDaemon::ScrollbackChunk` messages**, each ≤ 256 KiB (UDS message limit).
   After all chunks, send `HostToDaemon::ScrollbackDumpComplete` with cursor position, PTY
   dimensions, and total chunk count.

4. **Resume PtyBytes forwarding** after `ScrollbackDumpComplete` is sent.

5. **TUI receiver protocol:** On receipt of `ScrollbackDumpComplete`, the TUI MUST:
   a. Validate that `total_chunks` matches the number of `ScrollbackChunk` messages received.
      If mismatch → log WARN; send `ClientToServer::AttachSession { session_id }` to the
      daemon to trigger a fresh scrollback dump (TUI MUST NOT send `DaemonToHost::Attach`
      directly — that is a daemon→session-host message; per BC-2.05.011 Invariant 6 and
      SS-ipc.md §ClientToServer::AttachSession).
   b. Reset the parser: `pty_parsers[session_id] = vt100::Parser::new(pty_rows, pty_cols, SCROLLBACK_ROWS)`.
   c. Reconstruct the screen using the **`scrollback-as-bytes` path** (the single canonical
      reconstruction path for vt100 0.16). vt100 0.16 does NOT expose `set_screen()`,
      `inject_cells()`, or any API to directly write screen state; `Parser::process(&[u8])` is
      the only public input method (verified against docs.rs/vt100/0.16.0, 2026-06-03). The
      reconstruction path encodes the `ScrollbackChunk` cells as ANSI byte sequences and feeds
      them through the freshly-reset parser:
      1. Emit `\x1b[2J\x1b[H` (ED2 clear screen + cursor home) into the parser.
      2. For each row in the received cells (scrollback rows first, then visible screen rows),
         for each cell: emit the appropriate ANSI SGR sequences for the cell's fg/bg/attrs,
         then emit the cell's UTF-8 character. Emit `\r\n` between rows.
      3. After all rows, emit a cursor-positioning sequence (`\x1b[{row};{col}H`) to place
         the cursor at `cursor_row`/`cursor_col` from `ScrollbackDumpComplete`.
      All three steps are fed through `vt100::Parser::process()` on the freshly-reset parser
      instance. This is the only supported reconstruction path for vt100 0.16.
   d. (No alternate path.) vt100 0.16 exposes no direct screen-state injection API. The
      `scrollback-as-bytes` path in step (c) is unconditional; no runtime API-availability
      check is needed.
   e. After reconstruction, apply any buffered `PtyOutput` bytes received during the dump
      transfer (I3-003 fix: session-host no longer pauses PtyBytes; TUI buffers live bytes
      during dump and replays after Complete). Process buffered bytes through the now-reset
      parser in receipt order. After replay, subsequent `PtyOutput` events are processed
      normally. No double-counting occurs (parser was reset before dump was applied).

**Scrollback memory bound (O4 — wire-JSON vs in-RAM, reconciled with SS-embedded-pty §O4, I3-008):**

There are two distinct memory contexts to reason about. They must not be conflated:

**Wire-JSON (this section — session-host → daemon UDS):**
`SCROLLBACK_ROWS` is configurable (default 1000, max 10000 per SS-embedded-pty.md). Each
`SerializedCell` in JSON is ≈ 30–50 bytes (1 UTF-8 char + fg/bg color fields + attrs, with
JSON key overhead). A 80-column × 10000-row scrollback buffer in JSON ≈ 80 × 10000 × 40
bytes ≈ 32 MB per session. Across 8 sessions ≈ 256 MB of wire-JSON. This is TRANSIENT —
it is not stored anywhere permanently. The session-host serializes the buffer, sends it over
the per-session UDS, and the TUI deserializes and reconstructs it. After reconstruction the
wire-JSON is discarded. The peak transient allocation occurs DURING the attach/re-discovery
ScrollbackDump transfer: both the session-host (JSON encoding the buffer) and the TUI
(holding the received JSON before reconstruction) have ~32 MB of the same data in memory
simultaneously. For a single attach this is a ~64 MB transient spike; for 8 concurrent
re-discoveries at daemon restart it could be ~256 MB transient.

**In-RAM (cross-reference to SS-embedded-pty §O4 — TUI vt100::Parser storage):**
After reconstruction, the TUI's `vt100::Parser` holds the screen state as styled cells in
Rust-native form. The `vt100::Cell` struct (per SS-embedded-pty §O4) is ≈ 16 bytes
(char + fg/bg color enums + attrs + padding). A 80-col × 10000-row parser buffer ≈
80 × 10000 × 16 = 12.8 MB per session (in-RAM, not wire-JSON). For 8 sessions ≈ 102 MB.
This is the STEADY-STATE memory cost per SS-embedded-pty §O4 ruling.

**Chunked-stream mitigation for transient spike:**
The UDS 256 KiB message limit means large scrollbacks MUST be chunked: the `ScrollbackChunk`
protocol (replacing the legacy single-message `ScrollbackDump`) streams rows in batches of
≤ 256 KiB each. Chunking reduces the PEAK transient allocation on BOTH sides (session-host
holds only one chunk in JSON at a time during streaming; TUI accumulates chunks but can begin
parser reconstruction incrementally). The `ScrollbackDumpComplete` sentinel terminates the
stream. The session-host streams scrollback rows in batches; the TUI accumulates until
Complete, then reconstructs and discards the wire-JSON.

**I3-008 — attrs field impact on wire-JSON byte math:**
`SerializedCell.attrs` remains `u8` (1 byte in Rust; serialized as a JSON integer 0–255).
The I3-008 correction changes only the SEMANTIC interpretation of the bitmask (5 flags, not 6),
not the wire size. The `attrs` JSON integer is still 1–3 bytes depending on value. The ~40
bytes/cell estimate is unchanged — the "full visual fidelity" claim is now scoped correctly
to the 5 attributes that vt100 0.16 exposes (bold, dim, italic, underline, inverse). Blink,
hidden, and strikethrough are not observable from vt100 0.16 Cell API and therefore cannot
be serialized — this is a property of the vt100 library, not a spec deficiency.

**Summary of reconciled numbers (unchanged from v1.3.0 — attrs u8 size unaffected by I3-008):**
- Wire-JSON per session (transient during attach): ~32 MB (10k rows × 80 cols × 40 bytes/JSON-cell)
- In-RAM vt100::Parser per session (steady-state): ~12.8 MB (10k rows × 80 cols × 16 bytes/cell)
- Transient attach spike (wire+TUI simultaneously): ~64 MB per session for max scrollback
- Steady-state 8 sessions: ~102 MB (in-RAM only; wire-JSON is discarded after reconstruction)
- Live-PtyOutput buffer during dump (I3-003): ~500 KB at 1 MB/s for 500ms dump (bounded, transient)

### Main event loop

```rust
tokio::select! {
    // Backpressure: .await blocks the blocking thread if channel is full (see §PTY reader thread).
    // recv() here drains the channel; the spawn_blocking thread's send().await provides
    // the upstream backpressure signal.
    Some(bytes) = pty_reader.recv() => {
        parser.process(&bytes);
        for client in attached_clients.iter_mut() {
            // Also uses .await for backpressure up to daemon broker
            client.send(HostToDaemon::PtyBytes { bytes: bytes.clone() }).await;
        }
    }
    Some(msg) = daemon_conn.recv() => match msg {
        DaemonToHost::Attach => {
            // Snapshot vt100::Screen as styled cells; resume live PtyBytes IMMEDIATELY
            // (do NOT pause); stream ScrollbackChunk* messages; send ScrollbackDumpComplete
            // (see §Screen-state transfer on Attach, I3-003, ADR-0010 §Interleaving).
            // The TUI buffers live PtyBytes received during the dump and replays them
            // after ScrollbackDumpComplete.
            stream_scrollback_dump_chunked().await;
        }
        DaemonToHost::KeyInput { bytes } => { pty_writer.write_all(&bytes).await?; }
        DaemonToHost::Resize { rows, cols } => {
            pty.resize(PtySize { rows, cols, .. })?;
            parser.set_size(rows, cols);
        }
        DaemonToHost::Kill => { child.kill().await?; }
        DaemonToHost::Detach => { /* disconnect client; stay alive; stop sending PtyBytes */ }
    }
    Some(exit) = child_exit_watch.recv() => {
        send_state_changed(SessionState::Terminated).await;
        break;
    }
}
```

### PTY reader thread (C3 — no-silent-drop design)

The PTY master `read()` is blocking; it runs in a `tokio::task::spawn_blocking` thread.
PTY bytes MUST NOT be silently dropped under any backpressure scenario — dropping a
mid-CSI-sequence byte corrupts the vt100 parser's state machine, causing silent terminal
corruption that is invisible to the user until the screen becomes garbled.

**Design: bounded channel with backpressure + mandatory drop protocol.**

The PTY reader posts bytes to a bounded `tokio::sync::mpsc::channel::<Bytes>(1024)`
using `.send().await` (blocking on the channel, NOT `.try_send()`). The `spawn_blocking`
thread drives a `Handle::block_on(channel.send(bytes))` so the blocking thread waits
if the channel is full rather than dropping. This creates natural back-pressure from the
TUI's render speed all the way up to the PTY read syscall rate — if the TUI cannot keep
up, the session-host's PTY reader slows down (harness TUI will stall), which is the
correct behavior (the harness application is waiting for the user to consume output).

**Drop policy: no silent drops permitted for PTY bytes.** The channel is the backpressure
valve. If backpressure causes the `spawn_blocking` thread to block (because the async
event loop is overwhelmed), this is observable via the daemon's existing broker drop
counter surfaced in the TUI status bar. PTY bytes themselves are never discarded.

**Forced parser-reset protocol (mandatory if ANY drop ever occurs):** In the event that
a future refactoring or resource-exhaustion path does cause a PTY byte drop (e.g., an
OOM that kills the channel sender), the session-host MUST:
1. Detect the drop (sender returns `Err(SendError)` if the receiver is gone).
2. Immediately send `HostToDaemon::PtyReset { session_id }` to the daemon.
3. The daemon propagates `ServerToClient::PtyReset { session_id }` to all TUI clients.
4. Each TUI client, on receiving `PtyReset`, calls `pty_parsers[session_id] = vt100::Parser::new(rows, cols, scrollback_rows)` (fresh parser) and sends `ClientToServer::AttachSession { session_id }` to the daemon (NOT `DaemonToHost::Attach` directly — the TUI cannot send daemon→session-host messages; see SS-ipc.md §ClientToServer::AttachSession and C6-002 in SS-daemon-wiring-v2-delta §3) to trigger a new `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence.

This reset protocol ensures terminal corruption is NEVER silent. It is the mandatory
fallback; the primary design (backpressure via `.send().await`) makes it unreachable
in normal operation.

**TUI-surfaced PTY drop indicator:** If `PtyReset` is received by the TUI, the status
bar MUST display `[PTY reset — session <id>]` for 5 seconds. This surfaces any
architectural regression to the operator immediately.

`IPC channel (daemon → TUI):` The TUI-side `mpsc::channel(64)` uses the same backpressure
model: the IPC reader task uses `.send().await` into the app event loop, never `.try_send()`.
Backpressure propagates up to the daemon's broker fan-out, which then applies backpressure
to the session-host proxy task. The broker's drop counter (BC-2.04.011) is surfaced in
the TUI status bar and measures hook-event drops, not PTY drops. A separate per-session
`pty_drop_counter` metric is maintained by the session-host for observability.

---

## Daemon startup: session re-discovery

On daemon startup, `SessionManager::rediscover_sessions()` runs BEFORE the UDS socket
is bound (step 8b precedes step 10 in daemon_start_sequence — see SS-daemon-wiring.md).
This ensures no TUI client can observe a stale session list.

On daemon startup, BEFORE accepting UDS client connections, `SessionManager::rediscover_sessions()`
runs:

1. Read all `session-*.json` files in `<runtime_dir>`.
2. For each sidecar:
   a. Parse `session-state.json`; check `schema_version`.
   b. Probe liveness: `nix::sys::signal::kill(Pid::from_raw(pid), None)`.
   c. If alive: state-dependent handling:
      - `Launching`, `Running`: attempt UDS connect; verify `SO_PEERCRED`; if match: send
        `DaemonToHost::Attach`; wait up to 5s for `ScrollbackDumpComplete`; register `Running`.
      - `Detached`: attempt UDS connect; verify `SO_PEERCRED`; register `SessionEntry` with
        `state: Detached`, `host_conn: None`. DO NOT send `DaemonToHost::Attach`. (I3-005 fix —
        Detached intent preserved across restart; user must explicitly re-attach.)
      - `Terminating`: connect; verify SO_PEERCRED; send `DaemonToHost::Kill` (fire-and-forget);
        register `SessionEntry` with `state: Terminating`, `host_conn: None`; spawn BACKGROUND
        watchdog task (12s, absolute deadline from sidecar's `kill_deadline_unix_ms` if present
        and not elapsed; immediate SIGKILL if deadline already elapsed). Return immediately from
        this probe. (I3-002 fix — Terminating watchdog is a background task, not blocking the 5s
        re-discovery budget. BC-2.08.004 PC-7 5s budget excludes Terminating watchdog wait.)
   d. If dead: delete sidecar (GC); also delete orphaned socket file if present.
3. All re-discovered sessions are in `DaemonState.session_manager` before UDS bind (step 10).
4. Publish re-discovered sessions in `DaemonState.sessions` before serving TUI clients.

Re-discovery runs at step 8b (between step 8 lock-file-write and step 9 hooks-settings.json
in SS-daemon-wiring.md). The UDS bind is step 10 — clients cannot connect until re-discovery
is complete.

**Integration test gate:** `test_session_manager_rediscover_on_daemon_restart` verifies that
a live session-host survives a daemon restart and its session appears in the re-discovered list.

---

## Session GC policy

- `Terminated` sessions remain in the registry for **10 seconds** after termination, then
  are removed. GC timer is a tokio task; the TUI receives a `SessionListUpdate` on removal.
- `session-state.json` is deleted when the entry is GC'd from the registry.
- Orphaned sidecars (session-host PID dead, sidecar exists) are deleted during
  `rediscover_sessions()` at startup.

---

## SpawnRecipe integration with EngineModule

`EngineModule::spawn_recipe(opts)` returns:

```rust
/// The spawn recipe produced by an EngineModule.
/// SessionManager uses this to build the monocle-session-host command line.
/// All fields MUST be set by the implementing module (no Optional fields except where noted).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRecipe {
    /// Absolute path to the harness binary (e.g., /usr/local/bin/claude).
    pub binary: PathBuf,
    /// CLI args (e.g., ["--settings", "/tmp/monocle-hooks-abc.json"]).
    pub args: Vec<String>,
    /// Environment variables to OVERLAY on the session-host process's inherited environment.
    /// They do not replace the full env; the session-host builds CommandBuilder from
    /// the parent process env (inheriting PATH, HOME, etc.) and then overlays these fields.
    /// See §session-host startup step 4 for the env inheritance specification.
    pub env: HashMap<String, String>,
    /// Working directory for the harness child process.
    /// Populated from SpawnOptions.worktree_root — the resolved git worktree path
    /// (or project_root when no worktree applies). See SpawnOptions.worktree_root
    /// for resolution rules. NEVER hardcoded to project_root.
    pub cwd: PathBuf,
}

impl SpawnRecipe {
    /// ADR-0006 constructor: required because `SpawnRecipe` is `#[non_exhaustive]` and
    /// constructed cross-crate inside `ClaudeCodeModule::spawn_recipe()`, which lives in
    /// `monocle-runtime`. `SpawnRecipe` is defined in `monocle-core`; `monocle-runtime`
    /// depends on `monocle-core` as an external crate, so E0639 applies. All 4 fields are
    /// required positional parameters (no optional fields on `SpawnRecipe`).
    ///
    /// # Construction path
    /// - `monocle-runtime` (`src/engine/claude_code.rs`, `ClaudeCodeModule::spawn_recipe()`):
    ///   the ONLY production construction site. Daemon-internal; never transmitted over IPC.
    /// - `monocle-runtime/tests/`: integration test binaries that exercise `spawn_recipe()`
    ///   outcomes call `new(...)` for assertion fixtures.
    ///
    /// # ADR-0006 criteria
    /// (1) Internal workspace scope: `monocle-core` and `monocle-runtime` are both workspace
    ///     crates, never published to crates.io.
    /// (2) External protocol anchor: field additions (e.g., new harness CLI flags) arise from
    ///     Claude Code version bumps requiring coordinated BC revisions; not organic refactoring.
    /// (3) All 4 fields are required positional parameters.
    pub fn new(binary: PathBuf, args: Vec<String>, env: HashMap<String, String>, cwd: PathBuf) -> Self {
        Self { binary, args, env, cwd }
    }
}

/// Options sent from the TUI to the daemon via `ClientToServer::SpawnSession { opts }`.
/// The daemon's `spawn_session()` passes these to `engine_module.spawn_recipe(&opts)`
/// as its first step to obtain a `SpawnRecipe`.
///
/// `SpawnOptions` is the WIRE TYPE for the `ClientToServer::SpawnSession` IPC message
/// (I27-001 Model A resolution). It replaces the former `SpawnRecipe` wire payload.
/// The `SpawnRecipe` is daemon-internal: built by `spawn_recipe()` inside the daemon and
/// never transmitted over IPC. `SpawnOptions` carries the user-intent parameters that the
/// daemon needs to build the recipe.
///
/// Wire type policy: `#[non_exhaustive]` + `Serialize` + `Deserialize` (BC-2.02.003).
/// The `hooks_settings_path` and `session_id` fields are POPULATED BY THE DAEMON before
/// calling `spawn_recipe()` — the TUI sends `profile_id`, `harness_id`, `project_root`,
/// `worktree_root`, and `ccr_base_url` only. The daemon fills `session_id` (pre-generated
/// UUID) and `hooks_settings_path` (shared hooks-settings.json path) immediately upon
/// receiving the `SpawnSession` IPC message, before passing the completed `SpawnOptions`
/// to `spawn_session()`. See §IPC handler — new ClientToServer variants §3.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnOptions {
    /// The project root directory selected by the user in the SessionCreation wizard.
    /// Used for display grouping in the sessions panel (project_root, not cwd).
    pub project_root: PathBuf,
    /// The working directory for the harness child process.
    ///
    /// **Worktree-per-session (adopted from claude-squad A.1 gene, v1A scope):**
    /// When the user confirms a git worktree in SessionCreation Step 3 (WorktreeConfirm),
    /// the wizard resolves the worktree root path and sets this field. The harness child
    /// is spawned with cwd = worktree_root_or_project_root, NOT necessarily project_root.
    ///
    /// Resolution rules (applied in order by the SessionCreation wizard step 3):
    /// 1. If the project_root is a git worktree or the main worktree of a git repo AND the
    ///    user confirmed a specific worktree path in Step 3: `worktree_root` is that path.
    ///    Validation: the path exists AND `git -C <path> rev-parse --is-inside-work-tree`
    ///    returns exit 0. If validation fails, fall back to rule 3.
    /// 2. If the project_root is a git repo root and no specific worktree was selected:
    ///    `worktree_root` = project_root (the main worktree is the project root itself).
    /// 3. If the project_root is NOT a git repo (no `.git` present in any ancestor):
    ///    `worktree_root` = project_root. The cwd for the harness child is the project root.
    ///    BC-HOOK-029 requires `os.homedir()` to work in the hook JS — this is unrelated to
    ///    the project being a git repo; HOME is always available from the inherited env
    ///    (see §session-host startup step 4 env inheritance fix below).
    ///
    /// The wizard MUST resolve and validate this path in Step 3 (WorktreeConfirm). If the
    /// path cannot be validated, the wizard shows an error and stays on Step 3 until the
    /// user provides a valid path or cancels.
    ///
    /// `SpawnRecipe.cwd` is populated from this field (see §SpawnRecipe integration).
    pub worktree_root: PathBuf,
    /// Harness identifier selected by the user (e.g., "claude-code", "codemachine").
    pub harness_id: String,
    /// Harness profile ID selected in the SessionCreation wizard.
    pub profile_id: String,
    /// Daemon-assigned session UUID. Generated by the daemon IPC handler upon receipt of
    /// `ClientToServer::SpawnSession` (via `uuid::Uuid::new_v4().to_string()`), before
    /// `spawn_session()` is called. The IPC handler fills this field via
    /// `opts.with_daemon_fields(session_id, hooks_path)` BEFORE invoking `spawn_session()`,
    /// so `spawn_session()` receives `opts.session_id` already populated — it does NOT
    /// generate the UUID itself. (F-P41-IMP-001: canonical locus is the IPC handler, not
    /// `spawn_session()`.) The TUI does NOT generate this field.
    pub session_id: String,
    /// Path where the daemon has already written the shared hooks-settings.json.
    /// Populated by the daemon IPC handler (shared path: `<runtime_dir>/hooks-settings.json`).
    /// The TUI does NOT generate this field. See §Hook auto-injection — SHARED FILE MODEL.
    pub hooks_settings_path: PathBuf,
    /// If CCR is detected and a base URL is configured, this carries the URL.
    /// Populated by the TUI (it knows the CCR configuration from its profile settings).
    pub ccr_base_url: Option<String>,
}

impl SpawnOptions {
    /// ADR-0006 TUI-side constructor: required because `SpawnOptions` is `#[non_exhaustive]`
    /// and constructed cross-crate by `monocle-tui` when the user confirms a session in the
    /// SessionCreation wizard. The TUI populates exactly 5 fields; the daemon fills the
    /// remaining 2 (`session_id`, `hooks_settings_path`) upon IPC receipt via
    /// `with_daemon_fields()`. Per Rust E0639, struct-literal construction is forbidden
    /// outside the defining crate for `#[non_exhaustive]` types.
    ///
    /// The two daemon-owned fields are initialized to documented placeholder values:
    /// - `session_id: String::new()` — empty; always overwritten by daemon before use.
    /// - `hooks_settings_path: PathBuf::new()` — empty; always overwritten by daemon before use.
    /// These placeholders are never observable by production code because the daemon always
    /// calls `with_daemon_fields()` before passing `SpawnOptions` to `spawn_session()`.
    ///
    /// # Construction path
    /// - `monocle-tui` (`src/ui/session_creation.rs`): SessionCreation wizard Step 4
    ///   (Confirm) calls `SpawnOptions::for_spawn_request(...)` and sends the result in
    ///   `ClientToServer::SpawnSession { opts }`.
    /// - `monocle-ipc/tests/` and `monocle-tui/tests/`: integration test binaries call
    ///   `for_spawn_request(...)` for test fixture construction.
    ///
    /// # ADR-0006 criteria
    /// (1) Internal workspace scope: `monocle-core` is a workspace crate, never published.
    /// (2) External protocol anchor: `SpawnOptions` is the `ClientToServer::SpawnSession`
    ///     wire payload; field additions require coordinated BC revisions (I27-001 Model A).
    /// (3) All required fields are positional parameters; daemon-owned fields use documented
    ///     placeholder values (empty strings/paths) because they are ALWAYS overwritten before
    ///     production use — not arbitrary defaults that could silently propagate.
    pub fn for_spawn_request(
        project_root: PathBuf,
        worktree_root: PathBuf,
        harness_id: String,
        profile_id: String,
        ccr_base_url: Option<String>,
    ) -> Self {
        Self {
            project_root,
            worktree_root,
            harness_id,
            profile_id,
            ccr_base_url,
            // Daemon-owned fields: placeholder values; always overwritten by daemon
            // via with_daemon_fields() before spawn_session() is called.
            session_id: String::new(),
            hooks_settings_path: PathBuf::new(),
        }
    }

    /// ADR-0006 daemon-side consuming builder: fills the two daemon-owned fields on receipt
    /// of `ClientToServer::SpawnSession { opts }`. The daemon calls this immediately upon
    /// receipt, BEFORE passing the completed `SpawnOptions` to `spawn_session()`. This
    /// replaces the E0639-violating `SpawnOptions { session_id: ..., hooks_settings_path: ...,
    /// ..opts }` functional-update pattern (C30-001).
    ///
    /// # Why a consuming builder (not &mut self)?
    /// The IPC handler receives `opts` by value (from serde deserialization of the wire
    /// payload). A consuming builder avoids cloning and makes the "fill then pass" pattern
    /// natural: `let opts = opts.with_daemon_fields(uuid, path); spawn_session(opts).await`.
    ///
    /// # Construction path
    /// - `monocle-runtime` (daemon IPC handler, `src/ipc_handler.rs`): called immediately
    ///   on the deserialized `opts` from `ClientToServer::SpawnSession { opts }`.
    pub fn with_daemon_fields(mut self, session_id: String, hooks_settings_path: PathBuf) -> Self {
        self.session_id = session_id;
        self.hooks_settings_path = hooks_settings_path;
        self
    }
}
```

`ClaudeCodeModule::spawn_recipe()` fills the recipe from:
- Binary: `which::which("claude")` result
- Args: `["--settings", opts.hooks_settings_path.to_str().unwrap()]`
- Env: `ANTHROPIC_BASE_URL` if `opts.ccr_base_url.is_some()`; `MONOCLE_SESSION_ID = opts.session_id`
- Cwd: `opts.worktree_root` (the resolved worktree path — NOT opts.project_root directly)

**Hook auto-injection — SHARED FILE MODEL (C1 decision):** The daemon writes a SINGLE shared
`hooks-settings.json` at `<runtime_dir>/hooks-settings.json` (not per-session) using
`tempfile::persist`. This file is written ONCE on daemon startup (step 9 of daemon_start_sequence),
not per-session spawn. The `SpawnOptions.hooks_settings_path` always points to this one shared
file for all sessions in the same runtime_dir.

**Rationale:** The hooks-settings.json content is session-independent — it contains only
`hooks.<endpoint>` curl commands (pointing to the daemon port) and `lock.app = 'monocle'`.
It carries no port-per-session, no token-per-session, no session identifier. Two sessions
spawning concurrently both write identical content; the last-writer-wins result is
byte-identical to either individual write. Writing the file once at startup and reusing
it for all sessions is clobber-safe precisely because the content is a pure function of
(daemon_port, auth_token) — both of which are fixed for the daemon's lifetime.

**BC-HOOK-010 is the authoritative model.** BC-2.08.006 Invariant 3 and EC-182 (which
previously mandated per-session paths to avoid clobber) were architecture-level errors: they
misdiagnosed the clobber risk. Clobber is a problem only when content differs between writers;
here it never differs. BC-2.08.006 v1.2.0 has been RECONCILED in place by product-owner to <!-- version-pin-historical: v1.2.0 is the version at the time of this reconciliation event; historical record only -->
reflect the shared-file model — Invariant 3 and EC-182 have been rewritten (not removed) to
describe the correct shared-file behavior. This reconciliation is the canonical outcome; no
further BC-2.08.006 edits are needed.

The `--settings` arg carries this shared path. No user action required. `lock.app = 'monocle'`
filter in the hook JS ensures only monocle-launched sessions trigger the monocle endpoint.

---

## Module Purity Classification

| Module | Classification | Rationale |
|--------|----------------|-----------|
| `SessionState` enum | Pure core | Data type; `#[derive(Serialize, Deserialize)]`. No I/O. |
| `SpawnRecipe` struct | Pure core | Data type only; no I/O. Daemon-internal — not a wire type. |
| `SpawnOptions` struct | Pure core | Wire boundary data type; `#[derive(Serialize, Deserialize)]`. Carried in `ClientToServer::SpawnSession`. No I/O. |
| `SessionManager::spawn_session()` | Effectful shell | Spawns OS processes, writes sidecar, opens UDS. |
| `SessionManager::rediscover_sessions()` | Effectful shell | Reads filesystem, probes PIDs, connects UDS. |
| `SessionManager::send_key_input()` | Effectful shell | Writes to UDS stream. |
| `monocle-session-host main loop` | Effectful shell | PTY I/O, child process, UDS I/O, file writes. |
| `vt100::Parser` (owned by session-host) | Effectful shell | In-memory screen model; `process()` is stateful. |
| `SessionHostSpawner` trait (RealImpl) | Effectful shell | OS process spawn. |
| `MockSessionHostSpawner` | Effectful shell | In-memory mock; no real PTY. |

---

## Risk Mitigations

### Race: TUI connects before session re-discovery completes

Mitigation: `daemon_start_sequence` completes session re-discovery BEFORE binding the UDS
socket (step 8b precedes step 10). The lock file (step 8) is written BEFORE re-discovery
(step 8b), so the foreground `daemon start` caller can observe the lock file and declare
"daemon up" — but TUI clients that connect to the UDS socket (step 10) cannot connect
until re-discovery is complete. See §C2 fix below for the distinction between lock-file
readiness and IPC-ready readiness.

### Per-session UDS keystroke injection (I5)

Risk: daemon connects to a sidecar-specified socket path controlled by an attacker (via a
rogue sidecar), enabling `DaemonToHost::KeyInput` injection into a user's Claude Code session.

Mitigation: SO_PEERCRED peer-credential check on every per-session UDS connect (described
above in §Per-session UDS security). Security-reviewer MUST validate the SO_PEERCRED
implementation at implementation time.

### session-host process leaks on daemon kill -9

Mitigation: Session-hosts are process group leaders (setsid). They survive daemon crash (CASE 3).
On next daemon startup, re-discovery probes liveness and re-attaches or GC's orphaned sidecars.
If user explicitly kills all session-hosts, re-discovery marks them Terminated.

### Per-session UDS socket cleanup

Mitigation: session-host removes its socket file on clean exit (Goodbye message then unlink).
Daemon removes stale socket files during GC in re-discovery (alongside sidecar deletion).

---

## Behavioral Contracts (to be authored by product-owner in PRD delta)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.08.001 | Session spawn: recipe assembled and session-host started within 2s | P0 |
| BC-2.08.002 | Session persistence: sessions survive daemon graceful restart | P0 |
| BC-2.08.003 | Session kill: SIGTERM delivered within 500ms | P0 |
| BC-2.08.004 | Session re-discovery: all alive sessions visible after daemon restart within 5s | P0 |
| BC-2.08.005 | Session GC: Terminated session cleaned from registry after 10s grace period | P1 |
| BC-2.08.006 | Hook auto-injection: --settings arg present in child process args within 2s of spawn | P0 |
| BC-2.08.007 | SpawnRecipe: binary path resolved; cwd is project root; env carries CCR URL if applicable | P0 |

BC IDs are proposals; product-owner assigns canonical IDs in the PRD delta.

---

## §Trace v2.3.0 — Errata

**F-P42-IMP-001 — `wizard_session_id` orphan-name corrected to `launching_session_id` in IPC handler skeleton comment** (2026-06-14):

- **Finding (F-P42-IMP-001, IMPORTANT):** The `ClientToServer::SpawnSession` arm in §IPC handler
  (line ~548) contained a skeleton comment referencing `wizard_session_id`, a field name that does
  not exist. The canonical field is `AppMode::SessionCreation.launching_session_id` (SS-embedded-pty
  v1.6.0). The §Trace text for v2.3.0 Fix (b) also cited this incorrect name in the fix summary —
  that §Trace text is historical-exempt and left unchanged.
- **Affected site:**
  - Line ~548 (SpawnSession Err arm skeleton): `it MUST clear wizard_session_id` →
    `it MUST clear launching_session_id (set to None in AppMode::SessionCreation)`
- **Semver:** Errata-no-bump. No wire contract change; no behavioral change; prose-only reference-name
  correction matching the precedent of SS-ipc:412 (Pass-37) and SS-embedded-pty:250 (S39-001 Pass-39).
  Version remains v2.3.0.

## §Trace v2.3.0

**F-P41-IMP-001 — UUID-generation locus canonicalized to IPC handler; SpawnAck wiring added to handler skeleton** (2026-06-14):

- **Finding (F-P41-IMP-001, secondary — UUID locus):** `SpawnOptions.session_id` doc-comment
  at §SpawnOptions said "Pre-generated session UUID. Populated by the daemon IPC handler upon
  receipt ... before passing to `spawn_session()`." This was partially correct (IPC handler
  fills the field) but ambiguous: BC-2.08.001 PC-1 separately stated the UUID is generated
  "inside `spawn_session()`", creating a direct contradiction. The canonical locus is the IPC
  handler (the handler generates the UUID, fills SpawnOptions, then passes it to spawn_session()
  which receives opts.session_id already set).
- **Fix (a) — `SpawnOptions.session_id` doc-comment updated:** Explicitly states UUID generation
  is in the IPC handler, that `spawn_session()` receives the field already populated, and that
  `spawn_session()` does NOT generate the UUID itself. Cites F-P41-IMP-001 for traceability.
- **Fix (b) — IPC handler skeleton updated (§IPC handler):** `ClientToServer::SpawnSession` arm
  now reflects the 3-step sequence: (1) generate UUID, (2) send `SpawnAck` to requesting
  client only, (3) fill SpawnOptions via `with_daemon_fields()`, (4) call `spawn_session()`.
  Added normative comments explaining the canonical UUID locus and the TUI's obligation to
  clear `wizard_session_id` on spawn error.
- Semver: minor (v2.2.1 → v2.3.0) — new normative IPC handler behavior (SpawnAck step) and
  corrected UUID-locus wording.

## §Trace v2.2.1

**IMP-001 — `session_error_to_code()` prose self-contradiction about the `_ =>` arm on inner `EngineError` match** (2026-06-13):

- **Finding (IMP-001 IMPORTANT):** Lines ~449-452 prose and lines ~474-475 doc-comment were internally
  contradictory and factually wrong regarding the `_ =>` arm in `session_error_to_code()`.
  (a) The prose stated "all other inner variants fall through to `"invalid_request"`" and then
      immediately said "No `_ =>` wildcard swallow is permitted" — the "fall through" IS the `_ =>`
      arm, making the two sentences directly contradictory.
  (b) The doc-comment said "This function is EXHAUSTIVE over the SessionError enum — the compiler
      enforces coverage. Every arm must match." — overbroad; this exhaustiveness guarantee applies
      only to the OUTER `SessionError` match, not to the INNER `EngineError` match.
  (c) The actual code at line ~486 has `_ => "invalid_request"` inside the inner `EngineError` match
      — this arm is MANDATORY and CORRECT.
- **Root cause:** `EngineError` is defined in `monocle-core` and carries `#[non_exhaustive]` (confirmed
  in SS-engine-module-v2-delta.md §EngineError, lines 240-249). `session_error_to_code()` lives in
  `monocle-runtime` — a DIFFERENT crate. Rust requires a `_ =>` arm on any cross-crate match over a
  `#[non_exhaustive]` enum; the compiler WILL NOT compile without it. This is not a weakness in the
  design: the `_ => "invalid_request"` arm is a deliberate, forward-compatible fallback for future WASM
  engine variants (Phase 3). The `BinaryNotFound` and `InvalidPath` variants — the only ones that
  carry spawn-meaningful distinction — are explicitly routed before the catch-all.
- **Fix (a) — prose rewritten (§Exhaustiveness and forward-compatibility):** The prose now correctly
  describes the TWO-LAYER exhaustiveness model: (1) OUTER `SessionError` match is compiler-enforced
  exhaustive (no `_ =>`, same-crate); (2) INNER `EngineError` match requires a mandatory `_ =>` arm
  because `EngineError` is `#[non_exhaustive]` from `monocle-core`. The `_ => "invalid_request"` arm
  is documented as a deliberate, observable, audit-clean fallback — not a silent swallow.
- **Fix (b) — doc-comment rewritten:** Scopes "compiler-enforced exhaustive" to the OUTER
  `SessionError` match only. Notes the mandatory `_ =>` on the inner `EngineError` match and its
  `#[non_exhaustive]` cross-crate reason. No behavioral change; the actual code was already correct.
- Cross-reference: SS-engine-module-v2-delta.md §EngineError lines 245-246 states this correctly:
  "callers need a `_ =>` arm — which is already present in `session_error_to_code()`". This fix
  brings SS-session-manager.md into alignment with that reference.
- Semver: patch (v2.2.0 → v2.2.1) — prose/doc-comment correction only; no normative behavioral change.

---

## §Trace v2.2.0

**P31-HIGH-001 — `SpawnRecipe` in SS-session-manager.md missing `#[non_exhaustive]`, derives, and `impl new()` (Pass-31 sibling regression)** (2026-06-13):

- **Finding (P31-HIGH-001 IMPORTANT):** `SpawnRecipe` at line ~1063 was declared as a bare `pub struct SpawnRecipe { ... }` with no `#[non_exhaustive]`, no `#[derive(...)]`, and no `impl SpawnRecipe { pub fn new(...) }`. The canonical definition in `SS-engine-module-v2-delta.md §SpawnRecipe` carries all three (`#[non_exhaustive]` + `#[derive(Debug, Clone, Serialize, Deserialize)]` + `SpawnRecipe::new(binary, args, env, cwd)`). The audit table row (SS-engine-module.md §Cross-Crate Constructor Audit Table) also confirms the constructor exists. This was a Pass-30 sibling regression: `SpawnOptions` was made byte-for-byte consistent in v2.1.0 but `SpawnRecipe` was omitted from that propagation.
- **Fix:** Applied the same discipline as Pass-30 for `SpawnOptions`: `SpawnRecipe` struct declaration in §SpawnRecipe integration now carries the same `#[non_exhaustive]` + `#[derive(Debug, Clone, Serialize, Deserialize)]` + struct doc-comment + `impl SpawnRecipe::new(binary, args, env, cwd) -> Self` block as the canonical v2-delta definition — byte-for-byte consistent. The full ADR-0006 criteria rationale and construction path doc-comment are included (identical to SS-engine-module-v2-delta.md).
- **Byte-for-byte consistency verified:** Both the `struct SpawnRecipe` body and the `impl SpawnRecipe::new(...)` constructor are identical between this file (SS-session-manager.md) and SS-engine-module-v2-delta.md §SpawnRecipe. C29-001 lesson applied: both copies must remain identical; SS-engine-module-v2-delta.md is the canonical owner.
- **Audit table consistency:** The existing `SpawnRecipe` row in SS-engine-module.md §Cross-Crate Constructor Audit Table (v1.1.27) already records `new(binary: PathBuf, args: Vec<String>, env: HashMap<String, String>, cwd: PathBuf) -> Self` — this §Trace entry confirms the propagation of that constructor into the SS-session-manager.md mirror copy is now complete.
- Semver: minor (v2.1.0 → v2.2.0) — `SpawnRecipe` struct definition completed with `#[non_exhaustive]`, derives, and constructor; no behavioral change.

## §Trace v2.1.0

**C30-001 — ADR-0006 constructor gap: `SpawnOptions` lacked public constructors despite cross-crate construction** (2026-06-13):

- **Finding (C30-001 CRITICAL):** `SpawnOptions` is `#[non_exhaustive]` and constructed cross-crate by `monocle-tui` (SessionCreation wizard sends it as the `ClientToServer::SpawnSession { opts }` wire payload) and by `monocle-runtime` (daemon IPC handler fills `session_id` and `hooks_settings_path` on receipt). No public constructor existed. Additionally, the daemon-side fill step requires modifying two fields of an already-deserialized value — the E0639-violating `SpawnOptions { session_id: ..., ..opts }` functional-update pattern was used in SS-daemon-wiring-v2-delta.md §3 (also E0639 outside the defining crate).
- **Fix — `SpawnOptions::for_spawn_request(project_root, worktree_root, harness_id, profile_id, ccr_base_url) -> Self`:** TUI-side positional constructor added in §SpawnOptions struct definition. Daemon-owned fields initialized to documented placeholder values (`String::new()`, `PathBuf::new()`). See impl block in §SpawnOptions for full rationale and ADR-0006 criteria.
- **Fix — `SpawnOptions::with_daemon_fields(self, session_id: String, hooks_settings_path: PathBuf) -> Self`:** Daemon-side consuming builder added in §SpawnOptions struct definition. Replaces the `..opts` functional-update pattern. The daemon IPC handler now uses: `let opts = opts.with_daemon_fields(uuid, state.hooks_settings_path.clone()); spawn_session(opts).await`.
- **Byte-for-byte consistency with SS-engine-module-v2-delta.md:** The `impl SpawnOptions` block (constructor bodies, doc-comments, field order) is identical between this file (canonical definition) and SS-engine-module-v2-delta.md (cross-reference). C29-001 field-consistency lesson applied: both definitions must remain identical; SS-engine-module-v2-delta.md is the canonical owner for the constructor spec.
- **Audit table:** `SpawnOptions` added to `SS-engine-module.md §Cross-Crate Constructor Audit Table` (v1.1.26 → v1.1.27).
- Semver: minor (v2.0.0 → v2.1.0) — additive constructor additions; no API behavioral change.

## §Trace v2.0.0

**I27-001 — spawn-path model adjudication (Model A): `spawn_session()` accepts `SpawnOptions`; `SpawnOptions` becomes the wire type** (2026-06-13):

- **Finding (I27-001):** The `spawn_session()` signature (`recipe: SpawnRecipe, harness_id: String, profile_id: String`) was mutually exclusive with the §Trace v1.8.0 I12-001 prose ("`spawn_session()` calls `engine_module.spawn_recipe(&opts)?` as its first step") because no `opts: SpawnOptions` parameter existed on the function. The `ClientToServer::SpawnSession { recipe: SpawnRecipe }` wire payload was similarly inconsistent: the TUI cannot build a `SpawnRecipe` without access to the daemon-side `EngineModule`; and if the recipe is TUI-built, `EngineError::BinaryNotFound` / `EngineError::InvalidPath` are produced TUI-side with no wire bridge to `ServerToClient::Error`.
- **Adjudication — Model A is correct:** The daemon owns the `EngineModule` and `ClaudeCodeModule` instance. `spawn_recipe()` runs DAEMON-SIDE inside `spawn_session()`. The TUI sends user-intent spawn parameters (`SpawnOptions`); the daemon builds the `SpawnRecipe` from them. This is the only path on which `EngineError::BinaryNotFound` and `EngineError::InvalidPath` can reach `ServerToClient::Error` as the distinct BC-2.03.007 PC-3/PC-7 diagnostic codes. The thin-client principle (TUI sends intent; daemon executes) is consistent with the overall coordinator design.
- **Fix (a) — `spawn_session()` signature changed:** `spawn_session(recipe: SpawnRecipe, harness_id: String, profile_id: String)` → `spawn_session(opts: SpawnOptions)`. The method now calls `engine_module.spawn_recipe(&opts)?` internally as its first step. The `harness_id` and `profile_id` fields move to `SpawnOptions` (the `harness_id` was always needed for `SessionEntry` and is now an explicit field on `SpawnOptions`).
- **Fix (b) — `SpawnOptions` promoted to wire type:** `SpawnOptions` gains `#[non_exhaustive]`, `Serialize`, `Deserialize`. It becomes the `ClientToServer::SpawnSession { opts }` payload (replacing `SpawnRecipe`). The `SpawnRecipe` is now DAEMON-INTERNAL — it is produced by `spawn_recipe()` inside the daemon and never transmitted over IPC. `SpawnOptions` doc-comment updated: states wire-type status, `#[non_exhaustive]` rationale, and the field-population split (TUI populates `project_root`, `worktree_root`, `harness_id`, `profile_id`, `ccr_base_url`; daemon fills `session_id` and `hooks_settings_path` upon IPC receipt, before passing to `spawn_session()`).
- **Fix (c) — `SpawnRecipe` no longer a wire type:** `SpawnRecipe` loses its `Serialize`/`Deserialize` drives? No — `SpawnRecipe` already carries `#[derive(Debug, Clone, Serialize, Deserialize)]` in SS-engine-module-v2-delta.md (Pass-26 S26-001 fix). Under Model A, `SpawnRecipe` is daemon-internal; `Serialize`/`Deserialize` on a daemon-internal type is harmless but misleading. See wire-type reconciliation note in §SpawnRecipe integration — the `Serialize`/`Deserialize` derives on `SpawnRecipe` are removed (redundant for a non-wire type) in the companion SS-engine-module-v2-delta.md v1.2.0 edit.
- **Fix (d) — IPC handler pattern updated:** Sample code in §SessionError taxonomy §IPC handler pattern: `ClientToServer::SpawnSession { recipe }` → `ClientToServer::SpawnSession { opts }`, and the `spawn_session(recipe, ...)` call → `spawn_session(opts)`.
- **Fix (e) — Module Purity Classification updated:** `SpawnOptions` row updated from "Pure core / Data type only; no I/O" to "Pure core / Wire boundary data type; carried in `ClientToServer::SpawnSession`".
- **BC changes required (product-owner):** See I27-001 BC specification in the architect's final report.
- Semver: major (v1.9.0 → v2.0.0) — breaking API change to `spawn_session()` signature; `SpawnOptions` type promotion to wire boundary.

## §Trace v1.9.0

**S26-001 — `SessionState` missing `#[non_exhaustive]` (exhaustive wire-type class sweep)** (2026-06-13):

- **Finding (S26-001 exhaustive sweep):** `SessionState` was declared
  `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` without `#[non_exhaustive]`.
  `SessionState` is a wire-crossing type in multiple directions:
  (a) It is the value type of `ServerToClient::SessionStateChanged.new_state` — sent from daemon
      to TUI over the shared UDS IPC channel.
  (b) It appears in `SessionSnapshot.state`, which is carried in `ServerToClient::InitialState`
      and `ServerToClient::SessionListUpdate` over the same UDS channel.
  (c) It is serialized to the `session-state.json` sidecar file (schema_version 3) via
      `Serialize`/`Deserialize`.
  The SS-ipc.md §Message Types blanket policy ("All public enums and message structs carry
  `#[non_exhaustive]` per BC-2.02.003") requires it. Future session lifecycle phases (e.g., a
  `Suspending` state for future harness suspend/resume support) would benefit from forward-compat
  without a breaking match-arm change at all TUI consumers.
- **Fix — `#[non_exhaustive]` added above `#[derive(...)]` on `SessionState`.**
- **sidecar-file forward-compat note:** `#[non_exhaustive]` on `SessionState` ensures that a
  future daemon reading a sidecar written by an older binary with an unknown state string receives
  a serde deserialization error rather than a silent default. Combined with the existing unknown-state
  forward-compat rule ("Log WARN; skip; delete sidecar"), this is the correct behavior.
- **Module Purity Classification table:** No change needed — `SessionState` is already classified
  Pure core (data type, no I/O).
- Semver: minor (v1.8.1 → v1.9.0) — normative attribute addition; affects exhaustiveness rules <!-- version-pin-historical: changelog semver bump description; v1.8.1 is the prior version superseded by this entry -->
  at all Rust match sites on `SessionState` across the codebase.

## §Trace v1.8.1

**I14-001 fix — stale `portable-pty 0.8.x` version literal corrected to `0.9.x`** (2026-06-04):

- **I14-001 (IMPORTANT — stale crate-pin literal):** §session-host startup step 4 env-inheritance
  mandate contained one stale `portable-pty 0.8.x` version reference in the parenthetical
  "(check the `portable-pty` 0.8.x API)". The ratified pin is `portable-pty 0.9.0` (D-239;
  SS-deps-pin-manifest-v2-delta.md v1.0.0; ADR-0011 v1.0.0). All other `portable-pty` references
  in this file correctly cite the 0.9 line; the 0.8.x literal was the lone stale survivor.
  Changed: `0.8.x` → `0.9.x`. The behavioral mandate (seed builder from `std::env::vars()` before
  overlay) is version-independent and unchanged.
- SE-17c BEFORE: `(check the \`portable-pty\` 0.8.x API)`
- SE-17c AFTER:  `(check the \`portable-pty\` 0.9.x API)`
- Semver: patch (v1.8.0 → v1.8.1) — version-literal correction only; no normative behavior change. <!-- version-pin-historical: changelog semver bump description; v1.8.1 is the version created by this §Trace entry -->
- Root-cause: POL-11 enforcement keys on artifact-ID version pins (SS-x vN.M.P, BC-x vN.M.P),
  not crates.io version literals in prose, so this stale literal escaped CI enforcement.
  Durable follow-up DEP-PIN-SWEEP-RULE recommended for state-manager.

## §Trace v1.8.0

**I12-001 — EngineError bridge: `SessionError::EngineError` variant + `session_error_to_code()` arms** (2026-06-04):

- **Finding (I12-001):** `SessionError` had no `EngineError` variant. `spawn_recipe()` returns
  `Err(EngineError::BinaryNotFound)` and `Err(EngineError::InvalidPath)`, but neither could
  reach `ServerToClient::Error` as distinct codes. Both would have collapsed silently into the
  generic `"spawn_failed"` code (or been lost entirely), making BC-2.03.007 PC-3/PC-7 distinct
  user-visible diagnostics unsatisfiable.
- **Fix (a) — `SessionError::EngineError(#[from] EngineError)` variant added:** The new variant
  carries the inner `EngineError` verbatim via `#[from]`, giving `spawn_session()` a `?` propagation
  path. The `#[from]` derive generates `impl From<EngineError> for SessionError` automatically.
  This variant is spawn-path only: no other `SessionManager` lifecycle method calls `EngineModule`.
- **Fix (b) — `session_error_to_code()` extended with EngineError arm:** The match now unpacks
  `SessionError::EngineError(engine_err)` first and maps:
  - `EngineError::BinaryNotFound` → `"binary_not_found"`
  - `EngineError::InvalidPath`    → `"invalid_spawn_arg"`
  - all other EngineError variants → `"invalid_request"` (forward-compatible catch-all)
  The function remains EXHAUSTIVE over all `SessionError` variants; compiler enforces coverage.
- **Fix (c) — Mapping table updated:** Two new rows added at the top:
  `EngineError(BinaryNotFound)` → `"binary_not_found"` and `EngineError(InvalidPath)` →
  `"invalid_spawn_arg"`. Catch-all row for other EngineError variants added at the bottom.
- **spawn_recipe() call-site confirmed:** `spawn_session()` calls
  `engine_module.spawn_recipe(&opts)?` as its FIRST step (before `SessionHostSpawner::spawn()`),
  so `EngineError` failures abort the spawn before any OS process is created. No orphan-kill
  protocol is required for `EngineError`-derived failures.
- Semver: minor (v1.7.2 → v1.8.0) — new normative variant + function arm; BC-observable behavior change.

## §Trace v1.7.2

**S11-001 — §Trace v1.4.0 phantom `kill_deadline_reason` field corrected** (2026-06-04):

- **Finding (S11-001):** §Trace v1.4.0 (I3-002 bullet) listed `kill_deadline_reason:
  Option<String>` as a field added to `SessionEntry`. The normative `SessionEntry` struct
  (lines ~102–131) has no such field. The struct gained `kill_deadline: Option<Instant>`,
  `degraded: bool`, and `degraded_reason: Option<String>` in v1.4.0 — `degraded_reason` is
  the human-readable reason for the `degraded` flag (missing env vars, per I3-009). There
  is no kill-deadline-specific reason string; the `kill_deadline` field is an `Option<Instant>`
  with no associated reason string in the normative struct.
- **Correction (changelog only):** §Trace v1.4.0 I3-002 bullet: `kill_deadline_reason:
  Option<String>` replaced with a correction note pointing to this §Trace entry. The normative
  struct definition is unchanged and was never wrong — the phantom field existed only in the
  §Trace prose.
- Semver: patch (v1.7.1 → v1.7.2) — changelog correction only; no normative change.

## §Trace v1.7.1

**I10-001 — re-attach wording correction** (2026-06-04):

- **Location A (§Screen-state transfer on Attach, step 5a):** Replaced incorrect
  `SpawnSession`'s Attach re-send wording with the canonical `ClientToServer::AttachSession
  { session_id }` mechanism and explicit prohibition against sending `DaemonToHost::Attach`
  from the TUI. Phrasing now matches BC-2.05.011 Invariant 6 / PC-3a and SS-ipc.md
  §ClientToServer::AttachSession.
- **Location B (§PTY reader thread, Forced parser-reset protocol, step 4):** Replaced
  ambiguous "sends a fresh `Attach`" phrasing with explicit `ClientToServer::AttachSession
  { session_id }` and note that `DaemonToHost::Attach` is a daemon→session-host message
  the TUI cannot send directly. Cross-reference to SS-daemon-wiring-v2-delta §3 C6-002 added.
- Semver: patch (v1.7.0 → v1.7.1) — normative wording correction; no behavioral change.

## §Trace v1.7.0

**Pass-7 architecture findings — I-P7-001/I-P7-002/I-P7-004/S-P7-001** (2026-06-03):

- **I-P7-001 (SessionError taxonomy incomplete):** Added two missing variants to `SessionError`:
  `SidecarWriteFailed { path, reason }` (maps to `"sidecar_write_failed"`) and
  `SessionIdCollision { session_id }` (maps to `"session_id_collision"`). Both are
  spawn-path failures referenced by BC-2.08.001 EC-151 and EC-152. Distinct codes are used
  rather than collapsing to `"spawn_failed"` because they carry distinct user diagnostics:
  a sidecar I/O failure vs. a UUID v4 registry collision. The SS-ipc.md taxonomy is updated
  in v1.15.0 to add both codes to the closed error-code set. Added variant rows to the
  mapping table. `session_error_to_code()` is now exhaustive over all 7 variants (compiler
  enforced via exhaustive match).
- **I-P7-002 (kill_failed dead code):** Made `session_error_to_code` op-aware. Signature
  changed to `session_error_to_code(op: IpcOp, e: &SessionError)`. `IpcOp` enum added
  (Spawn / Kill / Attach / Detach / Rename / KeyInput / Resize). `SessionHostDead` now maps
  to `"kill_failed"` when `op == IpcOp::Kill` and `"attach_failed"` for all other ops.
  The mapping table updated: `SessionHostDead` split into two rows (attach-path and kill-path).
  `kill_failed` is now reachable; its previous dead-code status was a silent semantic error
  (kill failures surfaced as the misleading `"attach_failed"` code). All call sites updated
  to pass the `IpcOp` context (SpawnSession, KillSession, AttachSession arms shown).
- **I-P7-004 (retired pause-during-dump survivors):** Fixed two positions carrying the
  retired pause-during-dump model:
  (1) `HostToDaemon::ScrollbackChunk` doc-comment: rewritten from "session-host pauses live
  PtyBytes during the dump" to "session-host resumes live PtyBytes IMMEDIATELY after snapshot;
  does NOT pause; TUI buffers live bytes during dump and replays after Complete (I3-003)."
  (2) Event-loop Attach arm comment: rewritten from "pause PtyBytes, stream chunks" to
  "resume live PtyBytes IMMEDIATELY; do NOT pause; stream ScrollbackChunk*; TUI buffers
  live PtyBytes during dump (I3-003)." Both positions now consistently reference I3-003 and
  ADR-0010 §Interleaving.
- **S-P7-001 (spurious variant-level #[serde(default)]):** Removed `#[serde(default)]`
  from the `HostToDaemon::StateChanged` ENUM VARIANT. The attribute is invalid at enum
  variant level; serde `default` is field/container-level only. The field-level
  `#[serde(default)]` on `degraded_env` (which was already present) is the correct and
  sufficient mechanism for backward-compatible deserialization. Clarifying comment added.

## §Trace v1.6.0

**C6-001(b) — Pass-6 §Error handling section added; SessionError taxonomy + IPC handler mapping** (2026-06-03):

- **C6-001(b):** §Error handling section added between §Public API and §SessionHostSpawner trait.
  Documents: (1) `SessionError` enum with all variants and `thiserror` integration; (2) the
  mapping table from `SessionError` variant → `ServerToClient::Error.code` for every lifecycle
  method; (3) the canonical IPC handler pattern that MUST NOT swallow errors; (4) the
  `session_error_to_code()` helper; (5) the KeyInput special rule (errors still sent despite
  fire-and-forget success semantics); (6) the ResizePane special rule (errors silently dropped
  with WARN logging — benign race condition).
- This section is cross-referenced by SS-ipc.md v1.14.0 `SpawnSession` doc-comment ("SS-08 §Error
  handling") — the dangling reference from the SpawnSession variant is now resolved.
- BC-2.05.010's no-silent-failure invariant (PC-4 for SpawnSession, PC-4/last item for
  AttachSession/KillSession) is directly implemented by the mandatory IPC handler pattern here.

## §Trace v1.5.0

**C5-002 — SerializedCell/SerializedColor moved to monocle-ipc** (2026-06-03):

- **C5-002 (type ownership moved):** `SerializedCell` and `SerializedColor` were previously
  defined in this file (SS-session-manager.md §HostToDaemon) as structs owned by the
  `monocle-session-host` binary crate context. This created an incorrect dependency direction:
  `monocle-tui` would need to import from `monocle-session-host` (a binary) to use the scrollback
  reconstruction types — the same class of error as C3-004 (`SessionSnapshot` formerly embedded
  in session-manager before being moved to `monocle-ipc`).
  - **Resolution:** `SerializedCell` and `SerializedColor` are now defined canonically in
    `SS-ipc.md` §Supporting Types (monocle-ipc v1.13.0). Field definitions are identical to
    what was defined here: `ch: String`, `fg/bg: SerializedColor`, `attrs: u8` (5-flag vt100
    0.16 bitmask). `SerializedColor` variants: `Default`, `Ansi(u8)`, `Rgb(u8, u8, u8)`.
  - **This file:** The local `SerializedCell` / `SerializedColor` struct/enum definitions are
    replaced with a reference comment: `use monocle_ipc::{SerializedCell, SerializedColor}`.
    The `HostToDaemon::ScrollbackChunk.rows` field type remains `Vec<Vec<SerializedCell>>` —
    now resolved to `crate::ipc::SerializedCell` via the monocle-ipc import.
  - **SUG-001 consistency check:** The scrollback reconstruction path in §Screen-state transfer
    on Attach uses `SerializedCell` by reference (ANSI encoding from cell fields). Moving the
    type to `monocle-ipc` does not change the reconstruction algorithm — the field names
    (`ch`, `fg`, `bg`, `attrs`) and bitmask layout are identical. The §Screen-state transfer
    prose and the §Scrollback memory bound math are unchanged.

## §Trace v1.4.1

**SUG-001 — Adversarial Pass 4 residue: vt100 0.16 reconstruction path made canonical** (2026-06-03):

- **SUG-001 (screen reconstruction path, steps 5c-5d):** Steps 5c and 5d in §Screen-state
  transfer on Attach (TUI receiver protocol) previously presented three alternatives for screen
  reconstruction: a "set_screen()" API path, a direct cell-injection path, and the
  `scrollback-as-bytes` ANSI-encode-then-process path — with "if the crate does not expose
  direct cell injection" deferring the implementation choice to the implementer. This is an
  in-scope-answerable question per CLAUDE.md §Six Rules. vt100 0.16 public API verified against
  docs.rs/vt100/0.16.0 (2026-06-03): `Parser::process(&[u8])` is the only public input method.
  No `set_screen()`, no `inject_cells()`, no `screen_mut()` direct-write path exists in 0.16.
  Steps 5c-5d rewritten to name the single canonical path (`scrollback-as-bytes` via
  `Parser::process()`) with the explicit ANSI encoding algorithm (ED2 clear + SGR per-cell +
  cursor position). Step 5d now documents that no runtime API-availability check is needed
  because the bytes path is unconditional for vt100 0.16.

## §Trace v1.4.0

**Adversarial Pass 3 resolution — C3-003/I3-002/I3-003/I3-005/I3-008/I3-009** (2026-06-03):

- **C3-003 (rename emits wrong message):** `rename_session()` doc-comment corrected from
  "publishes SessionStateChanged to broker" to "publishes SessionListUpdate to broker". Rationale
  added inline: rename is not a SessionState transition; `SessionStateChanged` carries `new_state`
  only and cannot convey `display_name`. Full justification cross-references SS-daemon-wiring-v2-delta
  §3b emission table.
- **I3-002 (Terminating watchdog vs 5s budget):** §Re-discovery state handling updated for
  `Terminating` state: watchdog is spawned as a BACKGROUND background task (not blocking the
  `tokio::join_all` 5s budget). Fire-and-forget `DaemonToHost::Kill` re-send. Absolute kill
  deadline persisted in `session-state.json` as `kill_deadline_unix_ms` (schema_version 3).
  Elapsed deadline → immediate SIGKILL at re-discovery. Non-elapsed → watchdog uses sidecar
  deadline (not a new 12s window from restart). BC-2.08.004 PC-7 5s bound reconciliation note
  added: Terminating watchdog wait is excluded from the 5s re-discovery budget.
  `SessionEntry.kill_deadline: Option<Instant>` added. (`kill_deadline_reason` was listed here in
  error — no such field exists in the normative `SessionEntry` struct; see §Trace v1.7.2 correction.)
- **I3-003 (dump-pause stall):** §Screen-state transfer on Attach step 2 updated from
  "Pause live PtyBytes forwarding" to "Resume live PtyBytes forwarding IMMEDIATELY after snapshot."
  TUI receiver protocol step (e) updated: buffer live PtyOutput received during dump, replay after
  ScrollbackDumpComplete. §O4 summary row added: "Live-PtyOutput buffer during dump: ~500 KB."
  ADR-0010 §Interleaving flagged for update.
- **I3-005 (Detached forced to Running on re-discovery):** §Re-discovery state handling for
  `Detached` state corrected. Re-discovery now restores `Detached` sidecars to `Detached`
  (NOT `Running`). No `DaemonToHost::Attach` sent for Detached sidecars. User must explicitly
  request re-attach via TUI `AttachSession`. Rationale: respects persisted Detached intent;
  prevents 8 background sessions all becoming streamers on restart; BC-2.08.007 Inv-1
  compliance. Daemon startup re-discovery §c also updated.
- **I3-008 (SerializedCell.attrs — vt100 0.16 attribute surface verified):** `SerializedCell`
  doc-comment fully updated with verified vt100 0.16 Cell attribute API (5 methods: bold, dim,
  italic, underline, inverse). Prior 6-flag description (included blink; named "reverse" not
  "inverse") corrected. `attrs` bitmask layout table added (5 bits, bits 5–7 reserved). u8 type
  retained for forward-compat. "Full visual fidelity" claim scoped correctly to what vt100 0.16
  exposes. §O4 byte math unchanged (attrs remains u8). Research source: docs.rs/vt100/0.16.0
  verified 2026-06-03.
- **I3-009 (degraded-env surfaced to daemon):** Session-host startup step 4 EC updated: missing
  HOME/PATH is now surfaced via `HostToDaemon::StateChanged.degraded_env: Option<Vec<String>>`
  field (backward-compat, `#[serde(default)]`). Daemon sets `SessionEntry.degraded = true` and
  populates `degraded_reason`. `SessionSnapshot` carries `degraded`/`degraded_reason` fields
  so sessions panel can render warning badge. Stderr WARN retained as belt-and-suspenders.
  `SessionEntry` gains `kill_deadline`, `degraded`, `degraded_reason` fields.

## §Trace v1.3.0

**Adversarial Pass 2 resolution — C2-002/C2-003/C2-005/I2-002/I2-003/I2-004/I2-006/S2-003** (2026-06-03):

- **C2-002 (wire variants):** `HostToDaemon::ScrollbackDump` RETIRED; replaced by chunked
  `ScrollbackChunk`/`ScrollbackDumpComplete` two-message protocol. State transition table
  updated to reference `ScrollbackDumpComplete` (was `ScrollbackDump`). Re-discovery state
  handling table updated. Screen-state transfer §C5 updated to the chunked protocol with
  TUI integrity validation step (chunk count check). `ServerToClient::ScrollbackChunk`,
  `ServerToClient::ScrollbackDumpComplete`, `ServerToClient::PtyReset` are defined in
  ADR-0010; SS-daemon-wiring-v2-delta §5b/§5c carry the daemon fan-out paths.
- **C2-003 (directive-vs-outcome mismatch):** Removed stale "must be removed" directive.
  BC-2.08.006 Invariant 3 + EC-182 were RECONCILED IN PLACE by product-owner (not removed).
  The text now correctly states "BC-2.08.006 v1.2.0 has been reconciled in place." No further
  BC-2.08.006 edits are needed.
- **C2-005(a) (kill-path SO_PEERCRED):** §Per-session UDS security item 1 updated to state
  explicitly that SO_PEERCRED applies to EVERY per-session UDS connect including kill-path
  fresh-connect (EC-164). Added kill-path specific note.
- **C2-005(b) (SIGTERM escalation deadlines):** §Pre-socket-bind orphan kill updated to
  add SIGKILL call (was a comment), and added explicit explanation that the 2s deadline
  applies ONLY to pre-socket-bind orphan kills (not normal kills). Normal kill deadline (10s
  SIGTERM → SIGKILL, per BC-2.08.003 Invariant 4) is stated side-by-side for clarity.
  BC-2.08.003 sync flagged to product-owner (see product-owner action list below).
- **I2-002 (worktree-per-session operationalized):** `SpawnOptions.worktree_root` field added
  (was absent; only `project_root` existed). Three-rule resolution logic specified. `SpawnRecipe.cwd`
  doc-comment corrected to reference `worktree_root`. `ClaudeCodeModule::spawn_recipe()` fill
  corrected from `opts.project_root` to `opts.worktree_root`. `SessionEntry.cwd` field added
  (distinct from `project_root`). SessionCreation wizard Step 3 (WorktreeConfirm) owns
  resolution and validation. BC flag for product-owner: BC-2.03.005, BC-2.03.006, BC-2.08.001,
  and BC-2.08.007 need `worktree_root` vs `project_root` semantics reflected.
- **I2-003 (cross-client backpressure):** Specification deferred to ADR-0010 §Cross-Client/
  Cross-Session Backpressure Isolation and SS-daemon-wiring-v2-delta §5d. This document's
  references to "proxy task sends to broker" implicitly follow the new per-client buffer model.
- **I2-004 (Terminating state):** `Terminating` variant added to `SessionState` enum with
  full semantics (transitions, TUI render, 12s watchdog, re-discovery handling). State
  transition table updated with all Terminating transitions. BC flag for product-owner:
  BC-2.08.003 (kill description uses `Running → Terminating → Terminated`), BC-2.06.025
  (sessions panel must render `[Terminating]` state).
- **I2-006 (env inheritance):** Session-host startup step 4 updated with mandatory env
  inheritance rule: `CommandBuilder` inherits session-host process env THEN overlays
  recipe.env. EC for missing-critical-env (HOME/PATH missing) specified as WARN-logged
  (not abort). `SpawnRecipe.env` doc-comment updated to say "overlay" not "replace."
- **S2-003 (scrollback memory reconciliation):** §O4 expanded to explicitly separate
  wire-JSON (transient, ~32 MB/session at max scrollback) vs in-RAM vt100::Parser
  (steady-state, ~12.8 MB/session). Transient attach spike quantified (~64 MB per single
  max-scrollback attach; ~256 MB for 8 concurrent daemon-restart re-discoveries). Chunked
  protocol mitigation noted.

## §Trace v1.2.0

**Adversarial Pass 1 resolution — C1/C3/C5/I4/I5/I6/I7** (2026-06-03):

- **C1 (Hook-file model decision):** Resolved in favor of BC-HOOK-010 shared-file model.
  Per-session hooks file REMOVED from this spec. Hooks-settings.json is written once at
  daemon startup; all sessions share it. Rationale: content is session-independent; last-write-
  wins is safe because both writers produce byte-identical content. Product-owner delegation:
  BC-2.08.006 Invariant 3 + EC-182 must be removed; replace with shared-file model citing
  BC-HOOK-010 and this decision. HS-EXP-014 product-owner must reconcile to shared model.
- **C3 (PTY byte drop / no-silent-failure):** PTY reader redesigned to use `.send().await`
  (backpressure) instead of `.try_send()` (drop). Forced parser-reset + TUI-surfaced `PtyReset`
  indicator specified as mandatory fallback if any drop ever occurs. ADR-0010 head-of-line
  analysis deferred benchmark converted to explicit pre-v1A-gate requirement (not "before launch"
  — before the v1A story wave begins; see updated ADR-0010).
- **C5 (ScrollbackDump corruption):** `HostToDaemon::ScrollbackDump` redesigned: rows changed
  from `Vec<String>` to `Vec<Vec<SerializedCell>>` (styled cells with fg/bg/attrs). Screen-state
  transfer protocol specified: TUI resets parser on attach, reconstructs from cells, no double-
  counting. `ScrollbackDumpComplete` sentinel added for chunked large scrollbacks. Product-owner
  delegation: BC-2.08.007 PC-3 and PC-8 must be updated to reflect styled-cell serialization.
  BC-2.09.001 Invariant 4 scrollback memory bound must be recomputed with styled-cell overhead.
- **I4 (Unreachable SessionStates):** `Created` and `Killed` variants removed from `SessionState`
  enum (both are unreachable in the state machine). Re-discovery handling for all remaining
  states specified explicitly in §Re-discovery state handling table.
- **I5 (Per-session UDS security):** SO_PEERCRED peer-credential check specified as mandatory
  on every per-session UDS connect. Sidecar cross-check (sidecar pid vs socket peer pid) on
  re-discovery specified as mandatory. Per-session crypto token assessed as not required given
  SO_PEERCRED closes the uid-boundary threat. Security-reviewer validation flagged.
- **I6 (Orphan cleanup pre-socket-bind):** New §Pre-socket-bind orphan kill section. PID-based
  SIGTERM/SIGKILL fallback mandatory for all spawn-path failures after OS process is running
  but before UDS socket is bound. Product-owner delegation: BC-2.08.001 EC-151 must update
  the remediation from "send DaemonToHost::Kill" to "SIGTERM SpawnedHostHandle.pid" with
  2s escalation to SIGKILL.
- **I7 (Scroll offset per-session):** pty_scroll_offset field moved to per-session storage.
  See SS-embedded-pty.md update for the implementation.

## §Trace v1.0.1

**IMP-2 session_id type ruling** (2026-06-03):
- Added §session_id type — canonical ruling: `session_id` is `String` (UUID as String)
  at all IPC, registry, and AppMode boundaries. Rationale: avoids `uuid` dep in pure-core
  crate; eliminates UUID/String conversion friction. HashMap key comment updated.
- Propagated from consistency validation findings (IMP-2).

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- SS-08 authored as part of v1A architecture delta.
- Implements ADR-0009 (native session-host) and ADR-0010 (shared UDS).
- Q-2 resolved: lifecycle on SessionManager; EngineModule provides recipe only.
- SessionManager location confirmed as sub-module of monocle-runtime (self-adjudication
  confirmed per vision §Workspace Layout note; now formalized here).
- monocle-session-host new binary crate specified.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
