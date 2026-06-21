---
document_type: architecture-section
level: L3
section: "session-manager"
subsystem: SS-08
version: "2.16.0"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-21T00:00:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - specs/product-brief.md
  - specs/architecture/adr/ADR-0009-native-session-host-process-model.md
  - specs/architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md
  - specs/architecture/SS-daemon-lifecycle.md
  - specs/architecture/SS-ipc.md
  - specs/architecture/SS-engine-module.md
input-hash: "042e8f5"
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
    /// Live CONTROL connection to the session-host's per-session UDS socket.
    ///
    /// **When populated:**
    /// - `Some(_)` from the moment the daemon's background post-spawn monitor connects to
    ///   the session-host socket (after the session-host binds its UDS at startup step 7),
    ///   through the end of the session-host's life. This means `host_conn` is `Some(_)`
    ///   during BOTH `Launching` and `Running` states for freshly-spawned sessions.
    /// - `None` ONLY in two cases:
    ///   (a) Detached sessions — the daemon has explicitly disconnected the control
    ///       connection via `detach_session()`; session-host remains alive.
    ///   (b) Sessions registered from sidecar during re-discovery with `state: Detached`
    ///       or `state: Terminating` — the control connection is not pre-established for
    ///       these (see §Re-discovery state handling for details).
    ///
    /// **CONTROL connection vs PTY-STREAMING proxy task (F-P50-001 distinction):**
    /// These are two separate mechanisms:
    /// - The CONTROL connection (this field) carries `DaemonToHost::Kill/Attach/Detach/Resize`
    ///   commands and receives `HostToDaemon::StateChanged/Goodbye`. It is established
    ///   DURING `Launching` state by the background post-spawn monitor, so that the daemon
    ///   can receive the session-host's `StateChanged { new_state: Launching, degraded_env }`
    ///   startup handshake and the subsequent `StateChanged { new_state: Running }` readiness
    ///   signal before transitioning to `Running`. The `SessionHostConnection.proxy_task`
    ///   is `None` during `Launching` — only the writer half is active.
    /// - The PTY-STREAMING proxy task (`SessionHostConnection.proxy_task`) forwards
    ///   `HostToDaemon::PtyBytes` from the session-host to the daemon's broker. It is started
    ///   ONLY after `StateChanged { new_state: Running }` is received from the session-host
    ///   (i.e., the transition from `Launching → Running`). Before `Running`, no PTY streaming
    ///   occurs. The state transition table note "proxy task started" at `Launching → Running`
    ///   refers exclusively to the PTY-streaming proxy task, NOT the control connection.
    host_conn: Option<SessionHostConnection>,
}

/// Per-session connection to the session-host process.
///
/// The `writer` is the CONTROL connection write half — active from the end of `Launching`
/// onward (i.e., after the post-spawn monitor connects to the session-host socket).
/// The `reader` is the CONTROL connection read half — held for the session lifetime so that
/// `StateChanged` and `Goodbye` messages from the session-host can be received without
/// making a fresh UDS connect (see §Session-host accept model — ADV-S034-BLOCKER-001 ruling).
/// The `proxy_task` is the PTY-streaming task — started ONLY at the `Launching → Running`
/// transition when `StateChanged { new_state: Running }` is received.
///
/// This distinction resolves F-P50-001: `kill_session()` on a `Launching` session uses the
/// existing `host_conn.writer` (control connection already present) to send `DaemonToHost::Kill`.
/// No fresh UDS connect is needed for Launching kill — only for Detached kill.
///
/// ADV-S034-BLOCKER-001 ruling: `kill_confirm_monitor` MUST read `StateChanged{Terminated}`
/// from `reader` (this field), NOT by making a new `UnixStream::connect`. The session-host
/// sends `StateChanged{Terminated}` on the SAME connection where it received `DaemonToHost::Kill`.
struct SessionHostConnection {
    /// Write half of the per-session UDS control connection.
    /// Present from post-spawn monitor connect (during Launching) through session end.
    writer: Arc<Mutex<tokio::net::unix::OwnedWriteHalf>>,
    /// Read half of the per-session UDS control connection.
    /// Present from post-spawn monitor connect through session end.
    /// Moved into `kill_confirm_monitor` task when Kill is sent on Running/Launching path.
    /// For Detached kill: `host_conn` is None; a fresh connect supplies both halves.
    /// MUST NOT be dropped while the control connection is in active use.
    reader: Option<tokio::net::unix::OwnedReadHalf>,
    /// Background task proxying session-host PTY output to daemon broker.
    /// None during Launching state; started at the Launching → Running transition.
    proxy_task: Option<JoinHandle<()>>,
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
    /// session-host process spawned; waiting for `StateChanged { new_state: Running }` from
    /// the session-host. The daemon's post-spawn monitor polls for the session-host's UDS
    /// socket to become connectable; once connected + SO_PEERCRED verified, `host_conn`
    /// transitions from `None` to `Some(_)` (control connection established) while this
    /// state remains `Launching`. The PTY-streaming proxy task is NOT started until `Running`.
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
    ///
    /// Terminated-in-grace lifecycle action dispositions (F-P52-001):
    ///   kill_session()   → Ok(()) idempotent (BC-2.08.003 Invariant 2 — kill is already complete).
    ///   rename_session() → Err(InvalidSessionName { reason: "session terminated" }) → "rename_failed"
    ///                      (BC-2.08.005 Invariant 4: revive-via-rename is not allowed; no new code).
    ///   detach_session() → Ok(()) idempotent (session-host is dead; detach is a no-op, parallel to
    ///                      kill precedent; no new code).
    ///   attach_session() → Err(SessionHostDead { session_id }) → "attach_failed" (host is dead).
    ///   resize_session() → WARN-drop (existing ResizePane carve-out; no ServerToClient::Error).
    ///   send_key_input() → Err(SessionHostDead { session_id }) → "attach_failed" (host is dead;
    ///                      closest existing code; no new code).
    ///   (None of these require new SessionError variants or wire codes — F-P52-001 constraint.)
    ///
    /// TUI panel rule (F-P52-001): Terminated-in-grace sessions render [X] per BC-2.06.025
    /// Invariant 4. The TUI MUST block all lifecycle actions (k/d, D, r) on Terminated sessions
    /// — no-op + status bar hint "Session has terminated" — mirroring the Terminating guard.
    /// Kill (k/d) is explicitly blocked TUI-side despite the daemon's idempotent Ok(()) path;
    /// a dead session has no user-meaningful kill operation, and panel consistency with the
    /// Terminating guard is the production-grade default. The daemon defensive path remains
    /// available for untrusted clients and the existing BC-2.08.003 Invariant 2 coverage.
    Terminated,
}
```

#### State transition table

| From | Event | To | Notes |
|------|-------|----|-------|
| *(none)* | `spawn_session()` called | `Launching` | OS process spawned; sidecar written; background post-spawn monitor started (polls for socket connectable) |
| `Launching` | Post-spawn monitor: session-host UDS socket becomes connectable | *(still Launching)* | Daemon connects (SO_PEERCRED), stores `host_conn: Some(_)` (control connection established; PTY proxy NOT yet started) |
| `Launching` | Daemon receives `StateChanged::Running` from session-host over control connection | `Running` | PTY-streaming proxy task started; `host_conn.proxy_task` populated |
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
| `Terminated` | `kill_session()` called | *(no transition)* | Idempotent Ok(()) — kill already complete; no duplicate message (BC-2.08.003 Inv 2). TUI blocks this via panel guard (F-P52-001). |
| `Terminated` | `rename_session()` called | *(no transition)* | Err(InvalidSessionName { reason: "session terminated" }) → wire code "rename_failed" (BC-2.08.005 Inv 4: revive not allowed; no new code) (F-P52-001). |
| `Terminated` | `detach_session()` called | *(no transition)* | Idempotent Ok(()) — session-host dead; no active connection to detach (parallel to kill precedent; no new code) (F-P52-001). |
| `Terminated` | `attach_session()` called | *(no transition)* | Err(SessionHostDead) → wire code "attach_failed" — session-host process is dead (F-P52-001). |
| `Terminated` | `resize_session()` called | *(no transition)* | WARN-drop (existing ResizePane carve-out; no ServerToClient::Error) (F-P52-001). |
| `Terminated` | `send_key_input()` called | *(no transition)* | Err(SessionHostDead) → wire code "attach_failed" — host dead; no stdin to forward (F-P52-001). |

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

**Kill-path host_conn rules (F-P50-001 — no ambiguity):**
- **Running state:** `host_conn` is `Some(_)` (control connection + PTY proxy both active).
  `kill_session()` uses the existing `host_conn.writer` to send `DaemonToHost::Kill`.
  No fresh UDS connect.
- **Launching state:** `host_conn` is `Some(_)` (control connection active; PTY proxy not yet
  started). `kill_session()` uses the existing `host_conn.writer` to send `DaemonToHost::Kill`.
  No fresh UDS connect. The session-host aborts any partially-started harness child.
  **Special sub-case — kill-before-socket-bind:** If `kill_session()` is called before the
  post-spawn monitor has established `host_conn` (i.e., `host_conn` is still `None` during
  the brief window between `spawn_session()` returning and the monitor connecting), the daemon
  MUST use the PID-based SIGTERM/SIGKILL fallback (same as the §Pre-socket-bind orphan kill
  mechanism). This window is typically milliseconds but MUST be handled. The kill_session()
  implementation MUST check `host_conn.is_none() && state == Launching` and branch to PID-kill.
  Error code on PID-kill failure: `SessionError::SessionHostDead` → `"kill_failed"`.
- **Detached state:** `host_conn` is `None` (daemon explicitly disconnected). `kill_session()`
  makes a fresh UDS connect to the session-host socket, applies SO_PEERCRED, sends Kill.
  This is the only state where a fresh connect is needed on the kill path.

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

### Post-spawn monitor (F-P50-001 — control connection establishment during Launching)

`spawn_session()` returns `Ok(session_id)` immediately after OS process spawn and sidecar
write, leaving the `SessionEntry` in `state: Launching` with `host_conn: None`. The session
transitions to `Running` ASYNCHRONOUSLY. To bridge the Launching state:

`spawn_session()` MUST spawn a `tokio::spawn` background task (the **post-spawn monitor**)
that:

1. **Polls for UDS socket connectable** — retries `UnixStream::connect(socket_path)` with a
   short backoff (e.g., 20ms intervals) until the session-host binds its socket (startup step 7).
   Timeout: 30 seconds (allows for slow harness binary startup on constrained machines).

2. **Applies SO_PEERCRED** — immediately after connect, verifies peer uid matches daemon uid
   (same as every other per-session UDS connect per §Per-session UDS security). On mismatch:
   mark session `Terminated`, GC sidecar, publish `SessionStateChanged{Terminated}` +
   `SessionListUpdate`. This is `EC-163` (UDS connect fails / socket spoofed).

3. **Stores `host_conn: Some(SessionHostConnection { writer, proxy_task: None })`** — inside
   the `SessionManager` mutex. The control connection is now live. `proxy_task` is `None` at
   this point; only the writer is active. Kill/Resize/Detach commands are now deliverable over
   the control connection.

4. **Receives `StateChanged` messages** from the session-host over the control connection.
   The session-host may send `StateChanged { new_state: Launching, degraded_env: Some(...) }`
   as its first message (I3-009 degraded-env handshake). The daemon handles this by setting
   `SessionEntry.degraded: true` and `degraded_reason`. This message does NOT change the
   session's `SessionState` — it remains `Launching`.

5. **On `StateChanged { new_state: Running }`**: inside the mutex, starts the PTY-streaming
   proxy task (`host_conn.proxy_task = Some(tokio::spawn(...))`), transitions `SessionEntry.state`
   to `Running`, emits `SessionStateChanged{Running}` + `SessionListUpdate` to the broker.

6. **On `StateChanged { new_state: Terminated }` (spawn-path failure)**: transitions to
   `Terminated`, GC sidecar. This covers the case where the session-host fails during startup
   (e.g., harness binary not executable — distinct from `EngineError::BinaryNotFound` which
   catches it earlier; or PTY open failure).

7. **Kill-during-Launching (race with post-spawn monitor):**
   - If `kill_session()` is called while `host_conn` is `Some(_)` (monitor has connected):
     sends `DaemonToHost::Kill` over `host_conn.writer`. The monitor task, on seeing the
     connection close or receiving no further StateChanged, exits cleanly.
   - If `kill_session()` is called while `host_conn` is still `None` (monitor has not yet
     connected to the socket): uses PID-based SIGTERM/SIGKILL fallback (same as §Pre-socket-bind
     orphan kill, but with 10s SIGTERM window since the harness child may have already started).
     The daemon sets `SessionEntry.state = Terminating` immediately and publishes
     `SessionStateChanged{Terminating}` + `SessionListUpdate`. The post-spawn monitor, if it
     later connects to the socket, detects the session is already `Terminating` and exits.
     Error code on PID-kill failure: `SessionError::SessionHostDead` → `"kill_failed"` (mapped
     via `session_error_to_code(IpcOp::Kill, e)`).

8. **Detach-during-Launching (F-P51-001):** `detach_session()` requires `host_conn: Some(_)`.
   If called while `host_conn` is `None` (monitor not yet connected), it MUST return
   `Err(SessionError::SessionNotReady)` → mapped to `"session_not_ready"` error code on the
   IPC Detach arm. The official TUI never sends `DetachSession` before receiving
   `SessionStateChanged{Running}` (BC-2.06.025 + BC-2.08.007 Precondition 1 require
   `state: Running`). `SessionError::SessionNotReady` is a defensive invariant for untrusted
   clients sending `DetachSession` against a Launching session. It IS a live reachable wire code
   via the `DetachSession` arm.

   **ResizePane carve-out (no change):** `resize_session()` errors are WARN-logged and dropped
   by the IPC handler regardless of variant (SS-daemon-wiring-v2-delta.md §3 ResizePane arm;
   BC-2.05.010 Invariant 6 Exception). `resize_session()` MUST NOT be claimed to produce
   `"session_not_ready"` on the wire. If `resize_session()` returns `Err(SessionNotReady)` at
   the session-manager level (session in Launching, host_conn: None), the IPC handler WARN-drops
   it — no `ServerToClient::Error` is sent. The wire code `"session_not_ready"` is only reachable
   via the `DetachSession` arm.

**Ruling H — Monitor↔entry epoch/generation guard (MED-002 disposition):**

The post-spawn monitor is keyed only by `session_id: String` (UUID v4). An adversarial review
(MED-002) raised the question of whether the monitor should capture a generation/epoch token at
spawn time and compare it against the current `SessionEntry` before mutating state (e.g., on the
EC-163 SO_PEERCRED mismatch path setting state=Terminated + GC).

**Decision: no generation guard in v1. Deferred to S-036. MED-002 RESOLVED for S-036 scope — see resolution below.**

**Safety argument — three independent layers make the race structurally infeasible:**

1. **UUID v4 collision probability.** A second `spawn_session()` call inserting a new entry
   under a reused session_id requires the Terminated entry to have been GC'd first AND the
   new UUID to collide. UUID v4 collision probability ≈ 2^(-122). The `SessionIdCollision`
   error path (AC-006, EC-152) documents this as a handled invariant, not an operational risk.

2. **GC timer interlock.** The `Terminated` GC timer (10 seconds; S-037) is the earliest an
   entry can be removed from the registry. A `Launching` entry has NO removal timer — it stays
   in the registry until it transitions to Running or Terminated. The post-spawn monitor's 30s
   connect timeout cannot outlive the Terminated+GC+UUID-collision sequence without the 10s GC
   timer having fired first. The monitor is alive during `Launching` only; by the time GC could
   remove the entry (10s after Terminated), the monitor has long since exited (it transitions the
   entry itself before reaching Terminated, or detects Terminating state and exits cleanly per item 7).

3. **Outer `Mutex<SessionManager>` serialization.** Every entry removal (GC), every entry
   insertion (`spawn_session()`), and every monitor mutation (`host_conn`, `state`) acquires the
   same `&mut self` mutex. No interleaving is possible between removal and re-insertion. The
   sequence (removal) → (new UUID generation) → (collision with removed key) → (insertion)
   requires three separate mutex acquisitions and a 2^(-122) event in the middle.

**Why S-036 is the correct deferral anchor:** S-036 owns `rediscover_sessions()`, which is the
only path that registers `SessionEntry` records for sessions whose original post-spawn monitors
are gone (daemon crashed and restarted). Re-discovery does NOT spawn a new post-spawn monitor
(it connects directly, treating the session-host event loop as already running). If a future
cycle introduces entry eviction with shorter TTLs or a new lifecycle path that narrows the
safety margin above, S-036 is the correct story to add a `u64 generation` field to
`SessionEntry` (incremented at insert) and have the monitor capture it at spawn time for a
pre-mutation check. The mechanism would be: `if entry.generation != captured_generation { return; }`
inside the mutex before any state mutation on the EC-163 path.

**Implementation note for S-033:** no `generation` field is required. No generation-check is
required in the post-spawn monitor. This ruling documents the decision and the deferred
mechanism description for S-036.

**MED-002 RESOLVED — S-036 scope adjudication (2026-06-19, §Trace v2.15.1):**

The four questions raised by the MED-002 deferral anchor were adjudicated in S-036 scope. The
three-layer safety argument extends fully to `rediscover_sessions()`. No `SessionEntry.generation`
field is required. No new ACs or tasks are added to S-036.

*Q1 — Session ID reuse/collision:* Re-discovery does not generate new UUIDs. It re-inserts the
pre-existing session_id read from the sidecar file — the same UUID that was assigned at original
`spawn_session()` time. No collision scenario exists: there is no new random draw. Layer 1
(UUID v4 collision 2^(-122)) is inapplicable on the re-discovery path because no new UUID is
ever generated.

*Q2 — Monitor↔entry race existence:* `rediscover_sessions()` does not spawn a post-spawn
monitor. Per S-036 story intelligence: "re-discovery does NOT replay the post-spawn monitor —
it connects directly and sends Attach, treating the session-host as already running." The EC-163
SO_PEERCRED mismatch path that MED-002 targets (monitor mutates state=Terminated on a stale
entry) is a post-spawn monitor code path that is structurally absent during re-discovery. The
race cannot exist where the actor (post-spawn monitor) is never instantiated.

*Q3 — In-flight monitor coexistence during re-discovery:* Re-discovery runs at
`daemon_start_sequence` step 8b — BEFORE UDS socket bind (step 10, hard invariant per
BC-2.08.004 Invariant 1). No client request can arrive before UDS bind, therefore no
`spawn_session()` call can occur during `rediscover_sessions()`. No new post-spawn monitors
can be started during re-discovery. The sessions being re-discovered are from a PRIOR daemon
incarnation; their original monitors belonged to the previous daemon process (now dead). The
current process lifetime has zero in-flight post-spawn monitors during `rediscover_sessions()`.

*Q4 — Three-layer weakening:* Layer 1 is inapplicable (no UUID generation during re-discovery).
Layer 2 is inapplicable (no in-flight monitors; re-discovered entries start in Running/Launching/
Detached/Terminating — none have a pending GC timer at insertion time). Layer 3 is fully
maintained: `rediscover_sessions(&mut self)` acquires the same outer Mutex as all other
SessionManager operations; no interleaving possible.

**Conclusion:** No `SessionEntry.generation: u64` field is required. No monitor capture/compare
guard is required. The three-layer safety argument is unaffected by the S-036 lifecycle path.
MED-002 is closed with no implementation obligation. S-036 ACs and tasks are unchanged.

**`SessionError` taxonomy addition (F-P50-001):**
```rust
/// Operation called on a session in Launching state before the control connection was
/// established. Defensive invariant for untrusted clients — correct TUI never sends
/// DetachSession during Launching (TUI only enables detach for Running/Detached).
/// Wire producer: DetachSession IPC arm only. ResizePane IPC arm WARN-drops ALL
/// resize_session() errors (never reaches wire). Error code: "session_not_ready".
/// (F-P50-001; resize excluded F-P51-001)
#[error("session not ready: {session_id} (state: Launching, control connection pending)")]
SessionNotReady { session_id: String },
```

Add `SessionNotReady` to the SessionError taxonomy mapping table with code `"session_not_ready"`.

**Why `session_not_ready` is a dedicated code rather than reusing `invalid_request`:**

Three compounding reasons make a distinct code mandatory:

(a) **Untrusted-client wire posture.** The IPC socket is Unix-domain but not exclusively controlled by the official TUI — any process with socket access can connect. The daemon MUST NOT assume the client is the canonical TUI or that TUI-side state guards are active. A `DetachSession` message arriving for a session in `Launching` state (with `host_conn: None`) is a well-formed IPC message that the daemon MUST route defensively. Relying on TUI guards to make this path unreachable would be a safety invariant violation. (Note: `ResizePane` arriving during Launching IS handled defensively by `resize_session()` returning `SessionNotReady` at the session-manager level, but the IPC handler WARN-drops that error per the ResizePane carve-out — no wire code is emitted.)

(b) **Semantically distinct from `invalid_request`.** The `invalid_request` catch-all covers structurally malformed or unsupported messages. A `DetachSession` for a Launching session is fully valid and supported; it is merely MISTIMED. The session exists and is in a known transient state. Classifying this as `"invalid_request"` would prevent the TUI from distinguishing "bad message format" (unrecoverable) from "session not yet ready" (transient, retry-eligible after `SessionStateChanged{Running}` arrives).

(c) **No-silent-failure invariant without misclassification.** BC-2.05.010 Invariant 6 mandates that every `SessionError` path on user-initiated IPC operations (DetachSession is user-initiated) produces a `ServerToClient::Error` with a specific, typed code. A typed `"session_not_ready"` code satisfies this invariant without collapsing the error into the generic catch-all. If `invalid_request` were returned instead, the TUI banner would display `"[operation failed]"` — a misleading, unactionable message for a transient state condition.

**Why no explicit `SessionNotReady` for kill-during-Launching:** Kill on a `Launching` session
is explicitly ALLOWED (BC-2.06.025 EC-293; BC-2.08.003 Invariant 3). It MUST succeed — either
via the control connection (if established) or via PID fallback (if not yet established).
`SessionNotReady` is NOT returned by `kill_session()`.

**Integration test:** `test_kill_during_launching_before_socket_bind` — verifies that
`kill_session()` on a Launching session whose post-spawn monitor has not yet connected
falls back to PID-based SIGTERM and the session transitions to Terminating.
`test_kill_during_launching_after_socket_bind` — verifies that `kill_session()` on a
Launching session whose post-spawn monitor HAS connected sends `DaemonToHost::Kill` over the
control connection and the session transitions to Terminating.

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
    ///
    /// State-specific dispositions (F-P52-001):
    ///   Running: normal path — proxy task aborted; state transitions Running → Detached.
    ///   Launching (host_conn: None): Err(SessionNotReady) → "session_not_ready" (F-P50-001).
    ///     Defensive invariant — correct TUI never sends DetachSession during Launching
    ///     (BC-2.06.025 Invariant 5; F-P51-001). See §Post-spawn monitor item 8.
    ///   Detached: no-op or idempotent Ok(()) — already detached; no state transition.
    ///   Terminating: Err(SessionNotFound) semantics are acceptable here, or idempotent Ok(()).
    ///     Correct TUI blocks detach on Terminating (BC-2.06.025 Invariant 4). Defensive path.
    ///   Terminated: idempotent Ok(()) — session-host process is dead; there is no active
    ///     connection to detach from (parallel to kill-on-Terminated idempotency precedent).
    ///     No new SessionError variant or wire code (F-P52-001 constraint honored).
    pub async fn detach_session(&mut self, session_id: &str) -> Result<(), SessionError>;

    /// Re-attach the daemon to a running session-host (reconnect proxy).
    pub async fn attach_session(&mut self, session_id: &str) -> Result<(), SessionError>;

    /// Rename a session (updates display_name in sidecar; publishes SessionListUpdate to broker).
    ///
    /// Rename is NOT a SessionState transition. SessionStateChanged carries `new_state: SessionState`
    /// only and cannot convey the updated display_name. The correct broadcast is SessionListUpdate,
    /// which carries the full SessionSnapshot including the new display_name. SessionStateChanged
    /// is NOT emitted. See C3-003 / SS-daemon-wiring-v2-delta §3b emission table.
    ///
    /// State-specific dispositions (F-P52-001):
    ///   Launching, Running, Detached, Terminating: rename succeeds (display_name update; no state
    ///     transition). Note: the CORRECT TUI blocks rename on Terminating and Terminated via panel
    ///     guards (BC-2.06.025 Invariants 4 and 5/F-P52-001); this path is defensive for untrusted
    ///     clients. Rename on Launching IS allowed per BC-2.06.025 Invariant 5 (F-P51-001).
    ///   Terminated: returns Err(InvalidSessionName { reason: "session terminated" }) → wire code
    ///     "rename_failed". BC-2.08.005 Invariant 4 prohibits reviving a Terminated session via
    ///     rename. Uses the existing InvalidSessionName variant with a state-reason field — no new
    ///     SessionError variant or wire code (F-P52-001 constraint honored).
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
    /// Operation requires an established control connection but session is in Launching state
    /// with the post-spawn monitor not yet connected. Defensive invariant — correct TUI never
    /// sends DetachSession during Launching (only enables detach for Running/Detached).
    /// Wire producer: DetachSession IPC arm. ResizePane IPC arm WARN-drops this variant.
    /// See §Post-spawn monitor step 8 (F-P50-001; resize excluded F-P51-001). Error code: "session_not_ready".
    #[error("session not ready: {session_id} (state: Launching, control connection pending)")]
    SessionNotReady { session_id: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// An EngineModule operation failed before the OS process was spawned.
    /// Covers `EngineError::BinaryNotFound` (→ `"binary_not_found"`),
    /// `EngineError::InvalidPath` (→ `"invalid_spawn_arg"`), and
    /// `EngineError::UnsupportedOperation` (→ `"spawn_unsupported"`) (F-P44-IMP-001).
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
| `EngineError(UnsupportedOperation)` | `spawn_session` (via `spawn_recipe()`) | `"spawn_unsupported"` | Harness does not support monocle-controlled spawning; EC-112 defensive path (F-P44-IMP-001) |
| `SessionNotFound` | `kill_session`, `detach_session`, `attach_session`, `rename_session`, `send_key_input`, `resize_session` | `"session_not_found"` | Session ID not in registry. Note: a Terminated-in-grace session IS still in the registry; these methods reach state-specific dispatch BEFORE the SessionNotFound check would apply. See Terminated-in-grace dispositions in §State transition table (F-P52-001). |
| `SpawnFailed` | `spawn_session` | `"spawn_failed"` | OS process spawn failure (from spawner) |
| `SidecarWriteFailed` | `spawn_session` | `"sidecar_write_failed"` | Sidecar write failed after OS process spawned; orphan-kill protocol runs before this error surfaces |
| `SessionIdCollision` | `spawn_session` | `"session_id_collision"` | UUID v4 collision in registry; astronomically rare; do not auto-retry |
| `SessionHostDead` (attach-path) | `attach_session` | `"attach_failed"` | Session-host PID dead when daemon attempts attach |
| `SessionHostDead` (kill-path) | `kill_session` | `"kill_failed"` | Session-host PID dead when daemon attempts kill; see `session_error_to_code(Op, &SessionError)` |
| `InvalidSessionName` | `rename_session` | `"rename_failed"` | Empty name or name exceeding length limit. Also returned when `rename_session()` is called on a Terminated-in-grace session: `InvalidSessionName { reason: "session terminated" }` (BC-2.08.005 Invariant 4 — revive via rename not allowed; F-P52-001 — no new variant or code). |
| `SessionNotReady` | `detach_session` | `"session_not_ready"` | Session in Launching state; `host_conn: None` (monitor not yet connected); defensive invariant for untrusted clients — correct TUI never sends DetachSession during Launching (BC-2.06.025 + BC-2.08.007 Precondition 1). `resize_session()` may also return this variant internally but the ResizePane IPC arm WARN-drops ALL resize errors (never sends ServerToClient::Error). Wire producer: DetachSession arm only. (F-P50-001; resize excluded F-P51-001) |
| `Io` | Any | `"invalid_request"` | Unexpected I/O error; nearest generic failure code |
| `EngineError` (other/future variants) | `spawn_session` | `"invalid_request"` | Catch-all for any future `EngineError` variants not yet explicitly mapped (mandatory `_ =>` forward-compat arm); `"invalid_request"` is the nearest generic code. `UnsupportedOperation` now has its own arm and no longer falls through here (F-P44-IMP-001). |

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
  forward-compatible fallback that maps any truly novel future `EngineError` variants (added in
  Phase 3 for WASM engine modules) to the well-defined `"invalid_request"` wire code. No diagnostic
  information is silently lost: `BinaryNotFound`, `InvalidPath`, and `UnsupportedOperation` — the
  three Phase 1 variants that carry spawn-meaningful distinction — are all explicitly routed before
  the catch-all (F-P44-IMP-001 adds `UnsupportedOperation`). Any truly unknown future variant
  produces a deterministic, observable `ServerToClient::Error { code: "invalid_request" }` that is
  logged and sent to the requesting client (never dropped).

#### Terminated-in-grace defensive action×state matrix (F-P52-001)

The following matrix documents the canonical daemon-side disposition for every lifecycle action
dispatched against a session in EVERY non-Running state (including Terminated-in-grace). This
matrix closes the gap identified in F-P52-001: prior to v2.6.0, `rename_session()` and
`detach_session()` on Terminated-in-grace entries had no documented disposition in this spec,
creating a silent-success path that contradicts BC-2.08.005 Invariant 4 (revive-via-rename not
allowed) and the no-silent-failure invariant (BC-2.05.010).

**No new SessionError variants or wire codes are used.** All dispositions map to existing
variants and codes (F-P52-001 hard constraint honored).

| Action | Launching (no host_conn) | Launching (host_conn estab.) | Running | Detached | Terminating | Terminated-in-grace |
|--------|--------------------------|------------------------------|---------|----------|-------------|---------------------|
| `kill_session()` | Ok(()) — PID fallback SIGTERM; Launching→Terminating | Ok(()) — Kill over host_conn; Launching→Terminating | Ok(()) — Kill over host_conn; Running→Terminating | Ok(()) — fresh UDS+SO_PEERCRED + Kill; Detached→Terminating | Ok(()) — idempotent (BC-2.08.003 Inv 2) | Ok(()) — idempotent (BC-2.08.003 Inv 2; kill already complete) |
| `rename_session()` | Ok(()) — metadata op; no host_conn needed (BC-2.06.025 Inv 5) | Ok(()) — same | Ok(()) — metadata op | Ok(()) — metadata op | Ok(()) — metadata op; display_name update proceeds (TUI blocks this path per BC-2.06.025 Inv 4; daemon allows it defensively since rename is idempotent metadata) | Err(InvalidSessionName{"session terminated"}) → "rename_failed" (BC-2.08.005 Inv 4) |
| `detach_session()` | Err(SessionNotReady) → "session_not_ready" (F-P50-001) | Err(SessionNotReady) if host_conn not yet active for detach; see §Post-spawn monitor item 8 | Ok(()) — Running→Detached | Ok(()) — idempotent (already detached) | Ok(()) — idempotent (no active connection; consistent with detach-on-Terminated) | Ok(()) — idempotent (host dead; no connection to sever) |
| `attach_session()` | Err(SessionNotFound) or state-error — not valid target | N/A (not yet Running) | N/A (already attached) | Ok(()) — Detached→Running; fresh ScrollbackDump. Failure subpaths: (a) EC-187 ConnectFailed → Detached→Terminated (transition_to_terminated_standalone, full broadcasts+GC) + Err(SessionHostDead) — host confirmed dead; (b) SO_PEERCRED uid mismatch → Detached→Terminated (transition_to_terminated_standalone, full broadcasts+GC) + Err(SessionHostDead) — uid mismatch is structural/dead-host, not retryable; (c) protocol error after uid match → stay Detached (no transition) + Err(SessionHostDead) — host is alive+verified, retry is legitimate; (d) EC-188 5s timeout → SIGTERM to session-host PID → Detached→Terminated (transition_to_terminated_standalone, full broadcasts+GC) + Err(SessionHostDead) — SIGTERM declares host non-responsive/dead; re-attachable Detached entry for a SIGTERM'd host is incorrect. See BC-2.08.007 Invariant 5 for canonical enumeration. | Err(SessionHostDead) → "attach_failed" (session is being killed) | Err(SessionHostDead) → "attach_failed" (host process is dead) |
| `resize_session()` | WARN-drop (ResizePane carve-out) | WARN-drop | Ok(()) | WARN-drop (no active proxy; resize forwarded but no PTY streaming; IPC handler carve-out applies to all resize errors regardless of session state) | WARN-drop | WARN-drop |
| `send_key_input()` | Err(SessionNotFound or SessionHostDead) → "attach_failed" | SessionHostDead if host not live | Ok(()) — forwarded to stdin | Err — no active proxy/stdin path (session detached; use AttachSession first); maps to nearest existing code — SessionHostDead → "attach_failed" or SessionNotFound → "session_not_found" depending on impl; untrusted-client path | Err(SessionHostDead) → "attach_failed" | Err(SessionHostDead) → "attach_failed" |

**Clarification on rename-on-Terminating:** BC-2.06.025 Invariant 4 blocks rename at the TUI for
Terminating sessions. The daemon defensive path for an untrusted client sending RenameSession on
Terminating: the spec does not prohibit it at the daemon level (rename is metadata-only and
idempotent for the display_name field). However, since the correct TUI never reaches this path and
a terminated-in-progress session's display_name rename has no user value, returning idempotent
Ok(()) (metadata rename proceeds) is acceptable. No dedicated error needed.

**No panic / silent-success paths remain for any action on any state.** Every cell either returns
Ok(()) with documented idempotency rationale or returns Err with an existing SessionError variant
and existing wire code. The ResizePane carve-out WARN-drop is not a silent-success — it is
explicitly documented as the established benign-race exception.

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
///   on any cross-crate match over a `#[non_exhaustive]` enum. The three Phase 1
///   variants (`BinaryNotFound`, `InvalidPath`, `UnsupportedOperation`) each have
///   explicit arms (F-P44-IMP-001 adds `UnsupportedOperation`). The `_ => "invalid_request"`
///   arm is a deliberate, documented forward-compat fallback for future WASM engine
///   variants — not a silent swallow.
///
/// EngineError variants are unpacked to produce distinct spawn-path codes:
/// - `EngineError::BinaryNotFound` → `"binary_not_found"` (harness not on PATH)
/// - `EngineError::InvalidPath` → `"invalid_spawn_arg"` (bad argument to spawn_recipe())
/// - `EngineError::UnsupportedOperation` → `"spawn_unsupported"` (harness does not support
///   monocle-controlled spawning; EC-112 defensive path — F-P44-IMP-001)
/// - all other future `EngineError` variants → `"invalid_request"` (mandatory `_=>` forward-compat fallback)
fn session_error_to_code(op: IpcOp, e: &SessionError) -> &'static str {
    match e {
        SessionError::EngineError(engine_err) => match engine_err {
            monocle_core::engine::EngineError::BinaryNotFound(_)      => "binary_not_found",
            monocle_core::engine::EngineError::InvalidPath(_)         => "invalid_spawn_arg",
            monocle_core::engine::EngineError::UnsupportedOperation(_) => "spawn_unsupported",
            _                                                          => "invalid_request",
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
        SessionError::SessionNotReady { .. }      => "session_not_ready",
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

/// Production implementation: spawns monocle-session-host via std::process::Command.
/// The session-host binary calls `nix::unistd::setsid()` as its first startup step
/// (step 2 of §startup sequence), making itself a process group leader immune to
/// SIGHUP when the daemon exits.
///
/// **`pre_exec` is NOT used.** `std::process::Command::pre_exec()` is `unsafe fn`
/// (post-fork, pre-exec closure; async-signal-safety rules apply). `monocle-runtime`
/// carries `#![forbid(unsafe_code)]`, so `pre_exec` is categorically prohibited here.
/// The session-host binary is responsible for its own process-group detachment —
/// `nix::unistd::setsid()` is a safe Rust wrapper that it calls at startup before
/// any other initialization. See §Ruling C — setsid placement below.
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
        degraded_env: Some(vec!["HOME"]) }` is sent as the first message to the daemon over the
        control connection (established by the daemon's post-spawn monitor after the session-host
        binds its UDS socket at startup step 7). This message is deliverable DURING `Launching`
        state because `host_conn` is `Some(_)` by the time the session-host sends its first message
        — the control connection is established before the session-host enters its event loop.
        See §SessionEntry.host_conn doc-comment for the full control-vs-proxy distinction (F-P50-001).
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

1. **SO_PEERCRED peer-credential check (mandatory on EVERY per-session UDS connect or accept):**
   The SO_PEERCRED / LOCAL_PEERPID uid check applies universally — it is NOT restricted to
   attach or re-discovery. Every per-session UDS connection attempt (attach, re-discovery,
   AND kill/detach re-connect for Detached sessions per BC-2.08.003 EC-164) MUST verify the
   connecting peer's `uid` before reading any bytes.

   **Two-sided enforcement (ADV-S034-HIGH-001):**
   - **Daemon side (connector):** Every `UnixStream::connect` to a session-host socket MUST
     apply SO_PEERCRED on the connected stream before sending any message. This covers:
     post-spawn monitor initial connect, `kill_session()` on Detached (fresh connect),
     `attach_session()` (S-035 fresh connect), `rediscover_sessions()` (S-036 fresh connect).
   - **Session-host side (acceptor):** The session-host's accept-loop MUST apply SO_PEERCRED
     on EVERY accepted connection immediately after `listener.accept()` returns, BEFORE reading
     any DaemonToHost message. Uid mismatch: close the accepted connection immediately and loop
     back to `listener.accept()`. This prevents a rogue process from establishing a control
     connection to a running session-host (e.g., to inject `DaemonToHost::Kill` or
     `DaemonToHost::KeyInput`). The session-host accept-loop SO_PEERCRED check was previously
     unspecified — this is a spec gap closed by ADV-S034-HIGH-001.

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
    /// the moment the dump snapshot was taken. PtyBytes forwarding was NEVER paused
    /// (snapshot-then-resume protocol, I3-003 / ADR-0010 §Interleaving); live PtyBytes
    /// continued throughout the dump transfer and continue after this sentinel.
    ScrollbackDumpComplete {
        total_chunks: u32,
        cursor_row: u16,
        cursor_col: u16,
        /// Current PTY dimensions (rows × cols) at time of dump.
        pty_rows: u16,
        pty_cols: u16,
    },
    /// Live PTY output bytes. Sent continuously, INCLUDING during an active scrollback dump
    /// transfer (snapshot-then-resume protocol, I3-003 / ADR-0010 §Interleaving). The
    /// session-host does NOT pause PtyBytes for the dump; the TUI buffers live PtyBytes
    /// during the dump window and replays them after `ScrollbackDumpComplete`.
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

// C5-002 (SS-ipc.md v1.24.0): SerializedCell and SerializedColor are defined in
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

4. PtyBytes forwarding was never paused (step 2); it continues uninterrupted throughout
   and after the dump. After `ScrollbackDumpComplete` is sent no resume action is needed —
   the PTY reader loop continues normally.

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

### Session-host accept model (ADV-S034-BLOCKER-001 ruling)

**Decision: the session-host MUST run `listener.accept()` in a loop, not once.**

**Why the single-accept model is a spec gap (not just an implementer bug):**

The session state machine requires the daemon to make a FRESH `UnixStream::connect` in three
distinct scenarios after initial startup:

1. `kill_session()` on a `Detached` session — `host_conn` is `None`; daemon fresh-connects to
   send `DaemonToHost::Kill` (see §Kill-path host_conn rules and BC-2.08.003 EC-164).
2. `attach_session()` on a `Detached` session — S-035 scope; requires fresh connect.
3. `rediscover_sessions()` on any sidecar with a live session-host — S-036 scope; daemon
   connects to the session-host's socket for the first time after a daemon restart.

In all three cases the session-host must still be listening and able to accept. A single
`listener.accept()` followed by reading only from that one connection cannot serve a second
connect. The prior pseudo-code in §Main event loop implied a single connection (`daemon_conn`)
and the implementation matched that implication — both are wrong.

**Correct accept model: accept-loop over the listener, one active control connection at a time.**

The session-host maintains a `UnixListener` for the lifetime of the session. After handling
a `DaemonToHost::Detach` message (or after the initial connection is closed for any reason),
the session-host loops back to `listener.accept()` to wait for the next daemon connection.

**Invariants of the multi-accept model:**

1. **At most one active control connection at a time.** When a new connection is accepted,
   the previous connection (if any) is dropped. The session-host does NOT fan-out messages to
   multiple concurrent daemon connections — the daemon is the sole control-plane peer.

2. **SO_PEERCRED check on EVERY accepted connection** (BC-2.08.003 Invariant 5 / AC-009).
   Each call to `listener.accept()` is immediately followed by a SO_PEERCRED uid check before
   reading or sending any message. Uid mismatch: close the accepted socket and loop back to
   `accept()`. This applies to the FIRST accept (post-spawn monitor path) and ALL subsequent
   accepts (re-connect after Detach, re-discovery, kill-on-Detached).

3. **PTY continues running while waiting for the next accept.** The session-host is in
   `Detached` state between connections — it keeps the harness child alive and the PTY reader
   running. It does NOT freeze, sleep, or block solely on `accept()`. The `tokio::select!`
   outer loop MUST select over both `pty_reader.recv()` AND `listener.accept()` simultaneously
   when in Detached state, so PTY output continues to be processed (vt100 parser stays current)
   even when no daemon is connected.

4. **`DaemonToHost::Detach` transitions the session to Detached mode.** On receipt, the
   session-host stops forwarding `HostToDaemon::PtyBytes` on the current connection, drops
   the current connection's write half, and loops back to `listener.accept()`. The session
   continues alive (harness child, PTY reader, vt100 parser — all running).

5. **Connection EOF also triggers re-accept.** If the daemon's side of the control connection
   is dropped without sending `DaemonToHost::Detach` (daemon crash, daemon restart mid-session),
   the session-host MUST detect EOF on the connection and loop back to `listener.accept()`.
   This prevents the session-host from exiting when the daemon crashes and restarts.

6. **`DaemonToHost::Kill` terminates the accept loop.** On receipt of Kill, the session-host
   performs SIGTERM → 10s wait → SIGKILL, sends `StateChanged{Terminated}` + `Goodbye` on the
   CURRENT connection, removes the socket file, and exits. It does NOT loop back to accept.

**Consequence for the kill_confirm_monitor (daemon side — S-034 implementer action required):**

The current `kill_confirm_monitor` in mod.rs attempts a FRESH `UnixStream::connect` to read
back `StateChanged{Terminated}`. Under the corrected accept-loop model, this approach IS
structurally viable (the session-host can accept the second connection), but there is a
simpler and more correct design: the daemon should RETAIN the read half of the EXISTING
control connection rather than discarding it after the post-spawn monitor exits.

**Canonical implementation pattern for S-034:**

Extend `SessionHostConnection` to hold both halves of the control connection:

```rust
struct SessionHostConnection {
    /// Write half: used to send DaemonToHost commands.
    writer: Arc<Mutex<tokio::net::unix::OwnedWriteHalf>>,
    /// Read half: used to receive HostToDaemon responses (StateChanged, Goodbye).
    /// Present from post-spawn monitor connect through session end.
    /// The read half is moved into kill_confirm_monitor / watchdog task when kill is sent.
    reader: Option<tokio::net::unix::OwnedReadHalf>,
    /// Background task proxying PTY output to daemon broker.
    /// None during Launching; Some after Running.
    proxy_task: Option<JoinHandle<()>>,
}
```

The `kill_confirm_monitor` function MUST read `StateChanged{Terminated}` from `reader` (the
EXISTING connection's read half), NOT by making a new `UnixStream::connect`. Rationale:

- The session-host sends `StateChanged{Terminated}` on the SAME connection where it received
  `DaemonToHost::Kill`. If the daemon makes a fresh connect for the kill-confirm read, the
  session-host accepts that fresh connection as a NEW control connection and the
  `StateChanged{Terminated}` goes to the old (now-broken) connection that nobody is reading.
  The fresh-connect approach structurally cannot receive the Terminated confirmation.

- For DETACHED kill path (host_conn is None, fresh connect required): the daemon makes a
  fresh connect to send Kill AND to read the Terminated confirmation on the SAME fresh connection.
  The session-host accepts once for this operation. After accepting and completing the kill
  sequence, the session-host exits (no further accept needed).

**Consequence for attach (S-035) and re-discovery (S-036):**

These stories depend on the accept-loop model. The spec now explicitly states it. S-035
(`attach_session()`) and S-036 (`rediscover_sessions()`) MUST implement their daemon-side
fresh-connect using the same SO_PEERCRED discipline and MUST hold the resulting stream's
read half in `SessionHostConnection.reader` for any response processing.

### Main event loop

The session-host event loop has two structural phases driven by the accept-loop model:

**Phase A — Waiting for daemon connection (Detached mode):**

```rust
// In Detached mode, loop on accept() while keeping the PTY running.
tokio::select! {
    Some(bytes) = pty_reader.recv() => {
        // PTY bytes arrive; process into vt100 parser; do NOT forward (no active connection).
        parser.process(&bytes);
    }
    Ok((stream, _)) = listener.accept() => {
        // New daemon connection. Apply SO_PEERCRED before any I/O.
        if !verify_so_peercred(&stream) {
            // Uid mismatch — reject and loop.
            continue; // back to select!
        }
        // Transition to Phase B: active connection mode.
        current_conn = Some(stream);
    }
    // [child_exit_watch arm — implemented in S-039/S-040 (PTY output pipeline; the
    // session-host-side pty_reader and child_exit_watch are part of the full event loop
    // built when PTY streaming is wired in). S-034 does NOT implement this arm. S-034
    // session-host scope is DaemonToHost::Kill handler only (SIGTERM→SIGKILL,
    // StateChanged{Terminated}, Goodbye, remove socket). Absence of this arm in S-034
    // code is NOT a gap. See §Trace v2.7.0 Ruling A for the S-033 scope boundary table.]
    Some(exit) = child_exit_watch.recv() => {
        // Harness child exited while detached — notify whoever connects next (or nobody).
        // If current_conn is None: no connection to notify; just exit.
        send_state_changed_on_next_conn_or_exit().await;
        break;
    }
}
```

**Phase B — Active control connection (Running mode):**

```rust
// In Running mode, select over PTY, active connection messages, and child exit.
// daemon_conn is the accepted UnixStream (the current active control connection).
tokio::select! {
    Some(bytes) = pty_reader.recv() => {
        parser.process(&bytes);
        // Forward PtyBytes to daemon over current_conn.
        send_host_msg(&mut current_conn_writer, &HostToDaemon::PtyBytes { bytes }).await?;
    }
    msg = recv_daemon_msg(&mut current_conn_reader) => match msg {
        DaemonToHost::Attach => {
            // Scrollback dump: snapshot, stream ScrollbackChunk*, ScrollbackDumpComplete.
            // Resume live PtyBytes IMMEDIATELY (I3-003 / ADR-0010 §Interleaving).
            stream_scrollback_dump_chunked(&mut current_conn_writer).await;
        }
        DaemonToHost::KeyInput { bytes } => { pty_writer.write_all(&bytes).await?; }
        DaemonToHost::Resize { rows, cols } => {
            pty.resize(PtySize { rows, cols, .. })?;
            parser.set_size(rows, cols);
        }
        DaemonToHost::Kill => {
            // SIGTERM → 10s → SIGKILL; StateChanged{Terminated}; Goodbye; remove socket; exit.
            kill_sequence(&mut current_conn_writer, child_pid, &socket_path).await;
            return Ok(());
        }
        DaemonToHost::Detach => {
            // Drop current_conn; loop back to Phase A (Detached mode / listener.accept()).
            drop(current_conn_writer);
            drop(current_conn_reader);
            break; // back to Phase A select!
        }
        Err(ConnectionClosed) => {
            // EOF on control connection without DaemonToHost::Detach (daemon crash/restart).
            // Treat as implicit Detach: loop back to Phase A.
            drop(current_conn_writer);
            drop(current_conn_reader);
            break; // back to Phase A select!
        }
    }
    // [child_exit_watch arm — implemented in S-039/S-040 (PTY output pipeline; the
    // session-host-side pty_reader and child_exit_watch are part of the full event loop
    // built when PTY streaming is wired in). S-034 does NOT implement this arm. S-034
    // session-host scope is DaemonToHost::Kill handler only. Absence of this arm in S-034
    // code is NOT a gap. BC-2.08.008 PC-6 (natural-exit StateChanged{Terminated}) is
    // covered by the session-host child-exit path in S-039/S-040, flowing into the
    // daemon's BC-2.08.008 broadcast path (which S-034 wires for the kill-confirmation
    // side via AC-004). See §Trace v2.7.0 Ruling A scope table.]
    Some(exit) = child_exit_watch.recv() => {
        // Harness child exited while connected.
        send_state_changed(&mut current_conn_writer, SessionState::Terminated).await;
        send_goodbye(&mut current_conn_writer).await;
        return Ok(());
    }
}
```

**Implementation notes:**

- The `UnixListener` is bound at startup step 7 and persists for the session's lifetime. It is
  not rebound on each detach cycle.
- `tokio::net::UnixStream::into_split()` splits each accepted connection into
  `OwnedReadHalf` and `OwnedWriteHalf`. Both halves MUST be held for the duration of the
  active connection. The write half is used for DaemonToHost → HostToDaemon responses; the
  read half reads DaemonToHost commands.
- The first accepted connection (via the post-spawn monitor) uses the read half to deliver
  `StateChanged{Launching, degraded_env?}` followed by `StateChanged{Running}`. After the
  `Running` message is sent, the session is in Running mode (Phase B). The post-spawn monitor
  on the DAEMON side transitions `host_conn` from `None` to `Some(...)` and starts the proxy task.
- The daemon's `kill_confirm_monitor` reads `StateChanged{Terminated}` from `host_conn.reader`
  (the read half held in `SessionHostConnection`) — it does NOT make a fresh connection.
  See §kill_confirm_monitor implementation requirement.

### kill_confirm_monitor implementation requirement (ADV-S034-BLOCKER-001/BLOCKER-002)

The `kill_confirm_monitor` function in `monocle-runtime/src/session_manager/mod.rs` MUST be
refactored. The current implementation (fresh `UnixStream::connect` to read back
`StateChanged{Terminated}`) is structurally broken for the Running/Launching kill path:

**Why fresh-connect cannot work for Running/Launching kill:**

The session-host sends `StateChanged{Terminated}` on the SAME connection that received
`DaemonToHost::Kill`. Under the corrected accept-loop model, a second `UnixStream::connect`
from the daemon causes the session-host to accept a second connection — but the session-host
is already in Phase B (active connection with Kill in-flight). The Kill handler runs on the
FIRST connection; the `StateChanged{Terminated}` is sent back on the FIRST connection's write
half. The second connection receives nothing (the session-host exits before handling another
command on the second connection). The daemon's fresh-connect `kill_confirm_monitor` would
hang waiting for a message that will never arrive on the second connection, falling through
to its own 13s timeout and returning without processing the confirmation.

**Correct implementation:**

1. **Running/Launching kill path (host_conn is Some):**
   - `kill_session()` sends `DaemonToHost::Kill` using `host_conn.writer`.
   - The `kill_confirm_monitor` task takes ownership of `host_conn.reader` (moved out of
     `SessionHostConnection`; `host_conn.reader` set to `None` after the move).
   - The task reads from `reader` until `StateChanged{Terminated}` or `Goodbye` is received,
     or the connection closes (EOF = session-host exited without sending Terminated — watchdog
     handles the state transition).
   - No fresh UDS connect.

2. **Detached kill path (host_conn is None):**
   - `kill_session()` makes a fresh `UnixStream::connect` to the session-host socket.
   - Applies SO_PEERCRED before any I/O.
   - Splits the stream: writer sends `DaemonToHost::Kill`; reader is handed to
     `kill_confirm_monitor`.
   - The task reads from this same fresh connection's reader for `StateChanged{Terminated}`.
   - This is the ONLY case where a fresh connect is used; the session-host accepts it as a
     new control connection, receives Kill, responds with Terminated on the same connection.

**12-second watchdog timer:**

The watchdog timer fires if `StateChanged{Terminated}` is not received within 12 seconds of
sending `DaemonToHost::Kill`. The watchdog MUST implement the following SIGKILL safety check
to prevent PID-reuse attacks (ADV-S034-HIGH-002):

```rust
// Re-check session state under the lock before sending SIGKILL.
// The session-host may have confirmed Terminated via StateChanged between
// the timeout firing and the lock acquisition.
let should_sigkill = {
    let guard = sessions.lock().await;
    matches!(guard.get(&session_id).map(|e| &e.state), Some(SessionState::Terminating))
};
if !should_sigkill {
    // Session already transitioned (StateChanged{Terminated} was processed by another path).
    return;
}
// PID-reuse discrimination: check for ESRCH before treating kill() success as meaningful.
let pid = { sessions.lock().await.get(&session_id).map(|e| e.session_host_pid) };
if let Some(pid) = pid {
    match nix::sys::signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
        Ok(()) => { /* SIGKILL sent */ }
        Err(nix::errno::Errno::ESRCH) => {
            // ESRCH = process does not exist. PID was already reaped.
            // Treat this as a clean exit — transition to Terminated normally.
            tracing::info!(session_id = %session_id, pid = pid,
                "watchdog: SIGKILL returned ESRCH — session-host already exited; GC");
        }
        Err(e) => {
            tracing::warn!(session_id = %session_id, pid = pid, error = %e,
                "watchdog: SIGKILL failed unexpectedly");
        }
    }
    // Transition to Terminated regardless of SIGKILL outcome.
    // (State may already be Terminated if StateChanged{Terminated} raced with the watchdog.)
    force_terminate_session(&session_id, &sessions, &broker, &sidecar_path).await;
}
```

**ESRCH discrimination is mandatory.** A blind `kill(pid, SIGKILL)` that ignores ESRCH may
deliver SIGKILL to an unrelated process that reused the PID in the interval between the
session-host exiting and the watchdog firing. This is a real risk on high-churn systems.
The ESRCH branch MUST be handled explicitly, not collapsed into the general error arm.

### kill_deadline_unix_ms ownership boundary (ADV-S034-HIGH-003)

**Decision: S-034 owns the write; S-036 owns the read-back.**

`kill_session()` (S-034) MUST write `kill_deadline_unix_ms` to `session-state.json` when
transitioning a session to `Terminating` state. The value is `now_unix_ms + 12_000` (the
absolute kill deadline as Unix epoch milliseconds). This write is REQUIRED in S-034; it is
not deferred to S-036.

`rediscover_sessions()` (S-036) reads `kill_deadline_unix_ms` from the sidecar on re-discovery
of a `Terminating` sidecar and uses it to determine whether to SIGKILL immediately (deadline
elapsed) or start a deadline-preserving watchdog (deadline not yet elapsed). The in-process
watchdog (S-034 scope, for running daemon) derives its deadline from `SessionEntry.kill_deadline`
which is populated from the same value at the moment `kill_session()` is called — NOT from a
re-read of the sidecar. This means:

- The in-process watchdog always has the correct absolute deadline (`SessionEntry.kill_deadline`
  is set at `kill_session()` time, same instant as `kill_deadline_unix_ms` is written).
- On daemon restart (S-036), the sidecar `kill_deadline_unix_ms` restores the absolute deadline
  so the restarted daemon does not reset the SIGTERM window.

This boundary is explicit and non-overlapping. S-034 implementer MUST write
`kill_deadline_unix_ms` to the sidecar. S-034 adversarial review finding HIGH-003 is resolved
as a SPEC GAP (the write obligation was implied but not stated) and is now explicit in this section.

**Sidecar write timing:** `kill_deadline_unix_ms` is written in the SAME `tempfile::persist`
atomic write that transitions `session-state.json` to `state: "Terminating"`. These two
fields change atomically — there is no window where the sidecar shows `Terminating` with no
deadline or `Running` with a deadline.

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
        `DaemonToHost::Attach`; wait up to 5s for `ScrollbackDumpComplete`; register `Running`
        with `host_conn: Some(SessionHostConnection { writer, proxy_task: Some(...) })`.
        (Re-discovery does NOT replay the post-spawn monitor path — it connects directly and
        sends Attach, treating the session-host as already running. The result is always either
        Running or Terminated, never Launching. This is correct because the daemon crashed and
        the session-host is already in its event loop.)
      - `Detached`: attempt UDS connect; verify `SO_PEERCRED`; register `SessionEntry` with
        `state: Detached`, `host_conn: None`. DO NOT send `DaemonToHost::Attach`. (I3-005 fix —
        Detached intent preserved across restart; user must explicitly re-attach.)
      - `Terminating`: connect; verify SO_PEERCRED; send `DaemonToHost::Kill` (fire-and-forget);
        register `SessionEntry` with `state: Terminating`, `host_conn: None`; spawn BACKGROUND
        watchdog task (12s, absolute deadline from sidecar's `kill_deadline_unix_ms` if present
        and not elapsed; immediate SIGKILL if deadline already elapsed). Return immediately from
        this probe. (I3-002 fix — Terminating watchdog is a background task, not blocking the 5s
        re-discovery budget. BC-2.08.004 PC-7 5s budget excludes Terminating watchdog wait.)
        NOTE: `host_conn: None` for re-discovered Terminating sessions is intentional — the
        kill is fire-and-forget; no streaming or further control messages are needed; the
        watchdog uses a fresh connect for re-sends if needed.
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
filter in the hook JSON ensures only monocle-launched sessions trigger the monocle endpoint.

**Single-writer mandate (F-S038-PASS1-001 — BC-2.08.006):**
`session_manager::write_hooks_settings_json` is the SOLE function that serializes
`hooks-settings.json`. It is called from exactly two sites:

1. **Lifecycle step 9** — `daemon_start_sequence` constructs a `HookEndpointConfig` from the
   daemon's real `port` and `auth_token`, then calls
   `session_manager::write_hooks_settings_json(&config, &hooks_settings_path)`. This replaces the
   former `lifecycle::write_hooks_settings` function (which used `HooksSettings`/`HooksMap`/`HookEntry`
   types lacking a `lock` field — the missing `lock.app` was a security/data-integrity defect).
   `lifecycle::write_hooks_settings` and the `HooksSettings`, `HooksMap`, `HookEntry` types in
   `types.rs` are removed. The `serde_json::json!` schema in `write_hooks_settings_json` is the
   canonical serialization. Failure at step 9 → `DaemonStartError::HooksSettingsWriteFailure` →
   daemon exits code 72 (abort-on-failure, no partial file, no UDS bind).

2. **EC-182 re-write** inside `SessionManager::spawn_session()` — if hooks-settings.json is
   absent at spawn time (external deletion), `spawn_session()` calls
   `write_hooks_settings_json(&self.hook_endpoint_config, &self.hooks_settings_path)`. This
   produces a file with real curl URLs + `lock.app = 'monocle'` because `hook_endpoint_config`
   was populated at construction with the real config (see `SessionManager::new()` below).

**`SessionManager::new()` — real `HookEndpointConfig` required:**
`SessionManager::new()` signature MUST accept a `hook_endpoint_config: HookEndpointConfig`
parameter from the caller. In production, lifecycle constructs this from real port+token before
calling `SessionManager::new()`. The previously-existing `if !path.exists()` write guard in
`new()` is REMOVED: `new()` no longer writes hooks-settings.json. Lifecycle owns the step-9 write;
`new()` only stores the config in `self.hook_endpoint_config` for EC-182 re-writes.

In tests that do not set up a real daemon, the caller passes `HookEndpointConfig::default()`
(empty-string URLs). This is acceptable in test contexts where the written file's URL content
is not verified. Production construction ALWAYS provides non-empty URLs.

```rust
// Lifecycle step 9 (pseudocode — canonical single-writer call):
let hook_endpoint_config = HookEndpointConfig {
    pre_tool_use:        make_curl_cmd(port, auth_token, "pre-tool-use"),
    notification:        make_curl_cmd(port, auth_token, "notification"),
    stop:                make_curl_cmd(port, auth_token, "stop"),
    user_prompt_submit:  make_curl_cmd(port, auth_token, "prompt-submit"),
};
session_manager::write_hooks_settings_json(&hook_endpoint_config, &hs_path)?;
// ... then construct SessionManager::new(runtime_dir, spawner, broker, engine, hook_endpoint_config)

// SessionManager::new() signature (updated):
pub fn new(
    runtime_dir: PathBuf,
    spawner: Arc<dyn SessionHostSpawner>,
    broker: Arc<monocle_ipc::server::SubscriberList>,
    engine_module: Arc<dyn monocle_core::engine::EngineModule>,
    hook_endpoint_config: HookEndpointConfig,  // NEW: passed from lifecycle with real port+token
) -> Self
```

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

## §Trace v2.9.0

**Rulings H + I: outer-mutex locking discipline (MED-001) and kill_confirm_monitor mandatory on ExistingConn path (OBS-001)** (2026-06-17):

- **Ruling H** added: outer `Mutex<SessionManager>` held across bounded (2s) UDS connect on
  FreshConnect (Detached kill) and ExistingConn fallback paths is an ACCEPTED TRADEOFF. Three
  independent reasons: state consistency (releasing mid-kill opens a GC/collision window),
  bound is tight and scoped to rare paths (common Running/Launching path does no network I/O
  under the outer lock), and symmetry with S-033 spawn pattern (spawn also holds outer lock
  across OS spawn). PTY byte forwarding is unaffected (uses broker, no outer lock). Worst case:
  2s serialization of all session IPC ops per stale-socket Detached kill. No deadlock.
  Explicit documentation closes MED-001 for all future adversary passes.

- **Ruling I** added: `kill_confirm_monitor` is MANDATORY on the ExistingConn SUCCESS path
  (Running/Launching kill where Kill write succeeds). Relying on `post_spawn_monitor` to read
  `StateChanged{Terminated}` is forbidden. Root cause of the OBS-001 gap: `post_spawn_monitor`
  exits on EOF/read-error, `host_conn.writer` may still be live, Kill is sent, no reader is
  present — clean Terminated confirmation silently discarded — 12s watchdog SIGKILLs a session
  that had already confirmed. Fix: (1) post_spawn_monitor stores `reader: Some(reader)` at
  connect time; (2) post_spawn_monitor exits immediately after StateChanged{Running} — does NOT
  continue reading post-Running; (3) kill_session() ExistingConn SUCCESS path takes
  `host_conn.reader` and spawns `kill_confirm_monitor`. Erroneous "Note: the post_spawn_monitor
  task continues reading..." comment must be removed from kill_session(). Detached/FreshConnect
  path unchanged. Production-grade default: always-monitored.

- SE-16d monotonicity: v2.9.0 > v2.8.0. PASS.

---

## §Trace v2.8.0

**ADV-S034-BLOCKER-001/002, HIGH-001/002/003 — session-host accept-loop model, SO_PEERCRED on accept, watchdog PID-reuse, kill_confirm_monitor, kill_deadline ownership** (2026-06-17):

Adversarial review of S-034 (SessionManager::kill_session) identified four spec gaps:

- **BLOCKER-001 (session-host single-accept):** `step_event_loop()` in the implementation only
  calls `listener.accept()` once. For `Detached` sessions, `kill_session()` makes a fresh
  `UnixStream::connect` — but the session-host cannot accept it (only accepted once). Same
  applies to S-035 `attach_session()` and S-036 `rediscover_sessions()`. Root cause: the spec
  never stated the accept-loop model; the implementer inferred single-accept from the
  pseudo-code's single `daemon_conn` variable. This is a spec gap, not solely an implementer bug.
  **Fix:** §Session-host accept model (ADV-S034-BLOCKER-001 ruling) added. Session-host MUST run
  `listener.accept()` in a loop. Detailed two-phase event loop model specified (Phase A: Detached/
  waiting; Phase B: active connection). Consequence for S-035 and S-036 documented.

- **BLOCKER-002 (kill_confirm_monitor fresh-connect is wrong):** The daemon's
  `kill_confirm_monitor` attempted a fresh `UnixStream::connect` to read `StateChanged{Terminated}`.
  Under the accept-loop model, this connect causes the session-host to accept a second connection,
  but `StateChanged{Terminated}` is sent on the FIRST (existing) connection. The fresh connect
  approach structurally cannot receive the confirmation.
  **Fix:** `SessionHostConnection` gains a `reader: Option<OwnedReadHalf>` field. The
  `kill_confirm_monitor` reads from the existing connection's read half, not a fresh connect.
  For the Detached kill path (where `host_conn` is `None`), the fresh connect IS correct and
  provides both writer (Kill) and reader (Terminated confirmation) on the same stream.
  §kill_confirm_monitor implementation requirement section added.

- **HIGH-001 (SO_PEERCRED on session-host accept):** BC-2.08.003 Invariant 5 and §Per-session
  UDS security only specified SO_PEERCRED on the DAEMON's `UnixStream::connect` side. The
  session-host accept-loop must ALSO apply SO_PEERCRED on every accepted connection before
  reading any DaemonToHost message — otherwise a rogue same-uid process can connect to the
  session-host socket and inject Kill or KeyInput. The accept-side check was unspecified.
  **Fix:** §Per-session UDS security item 1 extended with explicit "two-sided enforcement"
  requirement: both connector (daemon) and acceptor (session-host) MUST apply SO_PEERCRED.

- **HIGH-002 (watchdog SIGKILL PID-reuse):** The watchdog fires 12s after Kill is sent and
  sends SIGKILL to `session_host_pid`. If the session-host already exited between the Kill
  send and the watchdog firing, the PID may have been reused by an unrelated process.
  Blind SIGKILL delivers to the wrong process.
  **Fix:** §kill_confirm_monitor implementation requirement specifies mandatory ESRCH
  discrimination: re-check `SessionState::Terminating` under lock before SIGKILL; treat
  ESRCH as clean exit (not an error); discriminate ESRCH from other kill() failures.

- **HIGH-003 (kill_deadline_unix_ms write ownership):** The sidecar write obligation for
  `kill_deadline_unix_ms` when transitioning to `Terminating` was implied but never explicitly
  stated in S-034 scope. The `rediscover_sessions()` re-read on S-036 requires the field to be
  present and correct.
  **Fix:** §kill_deadline_unix_ms ownership boundary section added. S-034 owns the write
  (atomically with the state=Terminating sidecar update); S-036 owns the read-back. The in-
  process watchdog uses `SessionEntry.kill_deadline` (an `Instant` populated at kill_session()
  time), not a sidecar re-read.

**SessionHostConnection struct updated:** `writer` type changed from `Arc<Mutex<UnixStream>>`
to `Arc<Mutex<OwnedWriteHalf>>`; `reader: Option<OwnedReadHalf>` field added.

**Semver: minor (v2.7.3 → v2.8.0)** — normative additions: accept-loop model, two-phase event
loop, kill_confirm_monitor reader-based design, SO_PEERCRED on accept, watchdog ESRCH guard,
kill_deadline write obligation. Multiple behavioral obligations added for S-034 implementer.
Session-host and daemon-side contract both updated. S-035 and S-036 accept-loop dependency noted.

SE-16d monotonicity: v2.8.0 timestamp 2026-06-17 >= v2.7.3 timestamp 2026-06-17. PASS.

---

## §Trace v2.7.3

**MED-002 disposition — monitor↔entry generation guard ruled out for v1; deferred to S-036** (2026-06-17):

- **Finding (adversarial review MED-002):** Post-spawn monitor is keyed only by `session_id: String`
  with no generation/epoch token. On the EC-163 SO_PEERCRED mismatch path the monitor sets
  `state=Terminated` and GCs the sidecar for whatever entry currently occupies that session_id —
  if the key were removed and a different logical session re-inserted under a reused id during the
  30s monitor connect window, the monitor could act on the wrong entry.
- **Decision:** No generation guard in v1. Three-layer safety argument documented in §Post-spawn
  monitor Ruling H: (1) UUID v4 collision ≈ 2^(-122); (2) 10s GC timer interlock prevents
  removal during Launching state; (3) outer Mutex serializes all insert/remove/monitor mutations.
  The sequence the finding describes requires all three independent layers to fail simultaneously.
- **Deferral anchor:** S-036 (rediscover_sessions / SessionEntry lifecycle). If entry eviction
  TTLs shorten or new lifecycle paths are added, S-036 is the correct story to add
  `SessionEntry.generation: u64` + monitor capture/compare guard.
- **Spec change:** Ruling H block added to §Post-spawn monitor section; version bumped 2.7.2→2.7.3.
  No SessionEntry struct fields added. No S-033 task changes required.
- SE-16d monotonicity: v2.7.3 timestamp 2026-06-17 > v2.7.2 timestamp 2026-06-17. PASS.

---

## §Trace v2.6.1

**Phase-2 Pass-1 fix burst — SS-ipc v1.24.0 Architecture Source pin update** (2026-06-16T00:00:00Z):

- §Supporting Types inline code comment updated: `SS-ipc.md v1.23.2` → `SS-ipc.md v1.24.0` (1 occurrence; C5-002 comment in §Supporting Types code block). Plain version-pin refresh — the §Supporting Types section (SerializedCell, SerializedColor) is structurally unchanged between v1.23.2 and v1.24.0.
- SE-16d monotonicity: v2.6.1 timestamp 2026-06-16 > v2.6.0 timestamp 2026-06-14. PASS.

---

## §Trace v2.6.0

**F-P52-001 — Terminated-in-grace action dispositions specified; daemon defensive matrix closed; no new variants/codes** (2026-06-14):

- **Finding (F-P52-001, IMPORTANT/no-silent-failure + cross-doc contradiction):** Prior to this
  version, `rename_session()` and `detach_session()` had no documented disposition for sessions
  in `Terminated`-in-grace state (the 10-second window after entry transitions to Terminated but
  before the GC timer removes the registry entry — BC-2.08.005 PC-1). A Terminated entry IS in
  the registry; neither call would return `SessionNotFound`. This left two unguarded paths:
  (a) **rename on Terminated**: would silently succeed (update display_name on a corpse),
      contradicting BC-2.08.005 Invariant 4 ("reviving a Terminated session via rename is not
      allowed").
  (b) **detach on Terminated**: would proceed against a dead session-host with undefined behavior
      (`host_conn` is None; no socket to detach from).
  Additionally, BC-2.06.025 Invariant 4 / Invariant 5 blocked lifecycle actions only for
  Terminating (blanket) and Launching (action-specific), with no Terminated panel guard — leaving
  the official TUI able to dispatch `r` (RenameSession) and `D` (DetachSession) on Terminated
  entries shown during the [X] grace window.

- **Canonical model decision (F-P52-001):**

  **TUI layer (BC-2.06.025 scope — reported to product-owner):** Terminated-in-grace sessions
  MUST mirror the Terminating guard: all lifecycle actions (k/d, D, r) are BLOCKED as no-ops
  with status bar hint "Session has terminated". Kill is blocked TUI-side even though the daemon
  handles it idempotently — a dead session has no user-meaningful kill operation, and panel
  consistency is the production-grade default. The existing [X] indicator (BC-2.06.025 Invariant
  4) is retained; the new guard is additive.

  **Daemon layer (this document):** All lifecycle actions on Terminated-in-grace entries map to
  existing SessionError variants and existing wire codes. NO new variants or codes added.
  - kill_session: idempotent Ok(()) — BC-2.08.003 Invariant 2 (already covered).
  - rename_session: Err(InvalidSessionName { reason: "session terminated" }) → "rename_failed".
    Reuses InvalidSessionName with a state-reason string. BC-2.08.005 Invariant 4 satisfied.
  - detach_session: idempotent Ok(()) — session-host is dead; no active connection to sever.
    Parallel to kill-on-Terminated idempotency precedent (parallel structure, consistent mental
    model: "dead session, dead session-host, lifecycle ops are no-ops or already-done").
  - attach_session: Err(SessionHostDead) → "attach_failed" — host process is dead.
  - resize_session: WARN-drop — existing ResizePane carve-out.
  - send_key_input: Err(SessionHostDead) → "attach_failed" — host dead; no stdin to forward.

- **Fixes applied:**
  (a) `SessionState::Terminated` doc-comment extended with full Terminated-in-grace lifecycle
      action disposition table (see inline variant doc-comment).
  (b) State transition table extended with 6 new rows covering all lifecycle actions on
      Terminated entries (rows: kill/rename/detach/attach/resize/send_key_input).
  (c) `rename_session()` public API doc-comment extended with state-specific disposition section
      covering Launching/Running/Detached/Terminating (normal) and Terminated (error path).
  (d) `detach_session()` public API doc-comment extended with state-specific disposition section
      covering all states including Terminated (idempotent Ok(())).
  (e) Mapping table `SessionNotFound` row: note added that Terminated-in-grace entries ARE in
      the registry and do NOT trigger SessionNotFound — they reach state-specific dispatch first.
  (f) Mapping table `InvalidSessionName` row: note added for Terminated-state rename disposition.
  (g) New §"Terminated-in-grace defensive action×state matrix (F-P52-001)" section added between
      the mapping table exhaustiveness prose and §IPC handler pattern. This section closes every
      cell in the action×state product for all non-Running states, with no-new-code constraint
      explicitly documented.

- **No new SessionError variants.** No new wire codes. Taxonomy remains 9 variants / 12 codes.
  The Terminated-rename path reuses `InvalidSessionName` with a new reason string — this is not
  a new variant (InvalidSessionName already had a `reason: String` field).

- **BC impact (product-owner to act):**
  - BC-2.06.025: needs Terminated panel guard (Invariant 4 extension or new Invariant 6) that
    blocks k/d, D, r on Terminated sessions — mirroring Terminating's Invariant 4 guard.
    New ECs and VPs needed. See architect report F-P52-001 §BC reconciliation.
  - BC-2.08.005: Invariant 4 now has an arch backing (rename_on_Terminated → rename_failed);
    should be updated to cross-reference this document's disposition. Version bump TBD by PO.
  - BC-2.08.003 Invariant 2: already covered (kill idempotency confirmed); no change needed.

- **Semver: minor (v2.5.1 → v2.6.0)** — normative additions: Terminated-in-grace disposition
  matrix, doc-comment extensions on two public API methods, state table rows (7 new rows), new
  defensive action×state matrix section. Multiple behavioral obligations added for implementers.

SE-16d monotonicity: v2.6.0 timestamp 2026-06-14 ≥ v2.5.1 timestamp 2026-06-14. PASS.

## §Trace v2.6.0 — Errata

**F-P54-001 — Retired pause-during-dump survivors removed from three positions (ERRATA-NO-BUMP)** (2026-06-14):

- **Finding (F-P54-001, IMPORTANT):** Three doc-comment/prose positions carried stale text from
  the retired pause-during-dump model, contradicting the canonical snapshot-then-resume protocol
  already correctly specified at lines ~1074-1075 (ScrollbackChunk doc-comment), lines ~1149-1152
  (Screen-state transfer step 2), the event-loop Attach arm comment (line ~1256), and ADR-0010
  §Interleaving. The survivors were:

  **(1) `HostToDaemon::ScrollbackDumpComplete` doc-comment (line ~1089-1090):**
  Previous: "After sending this, the session-host resumes forwarding live PtyBytes."
  Implied that PtyBytes were paused during the dump and needed to be resumed after the sentinel.
  Under the snapshot-then-resume model no pause ever occurs; "resumes" is incoherent.

  **(2) `HostToDaemon::PtyBytes` variant doc-comment (line ~1099):**
  Previous: "/// Live PTY output bytes. NOT sent during an active scrollback dump transfer."
  This was an explicit statement of the retired pause model.

  **(3) §Screen-state transfer on Attach step 4 (line ~1158):**
  Previous: "**Resume PtyBytes forwarding** after `ScrollbackDumpComplete` is sent."
  Step 2 (line ~1149) already mandates IMMEDIATE resume after snapshot (before any chunks
  are sent), making step 4 contradictory — it re-implied a pause that step 2 prohibits.

- **Fixes applied:**
  (a) `ScrollbackDumpComplete` doc-comment: rewritten to "PtyBytes forwarding was NEVER paused
      (snapshot-then-resume protocol, I3-003 / ADR-0010 §Interleaving); live PtyBytes continued
      throughout the dump transfer and continue after this sentinel."
  (b) `PtyBytes` doc-comment: rewritten to "Sent continuously, INCLUDING during an active
      scrollback dump transfer (snapshot-then-resume protocol, I3-003 / ADR-0010 §Interleaving).
      The session-host does NOT pause PtyBytes for the dump; the TUI buffers live PtyBytes during
      the dump window and replays them after `ScrollbackDumpComplete`."
  (c) §Screen-state transfer step 4: rewritten to "PtyBytes forwarding was never paused (step 2);
      it continues uninterrupted throughout and after the dump. After `ScrollbackDumpComplete` is
      sent no resume action is needed — the PTY reader loop continues normally."

- **Exhaustive sweep results (third sweep — F-P54-001):**
  Checked: SS-session-manager.md, SS-daemon-wiring-v2-delta.md, SS-daemon-wiring-impl.md,
  SS-daemon-wiring.md, SS-embedded-pty.md, SS-engine-module.md, ADR-0010, BC-2.05.011.
  PtyReset path, AttachSession path, re-attach-on-chunk-mismatch path: all clean.
  SS-daemon-wiring-impl.md line ~1389 "pause before writing body" is HTTP test harness
  (unrelated). SS-engine-module.md line ~448 "paused awaiting user decision" is permission
  overlay state (unrelated).
  **Retired-pause survivor count after this errata: 0.**

- **No normative behavior change.** The snapshot-then-resume protocol (PtyBytes never paused,
  TUI buffers live bytes during dump, replays after Complete) was already normatively specified
  at step 2, the event-loop Attach arm comment, and ADR-0010. Only the three stale doc-comment
  survivors are corrected. No new wire variants, no new error codes, no obligation changes.

- **Semver: ERRATA-NO-BUMP (stays v2.6.0).** Consistent with Pass-39/48/53 errata precedent
  for retired-concept doc-comment corrections. No registry change. No POL-11 cascade.

---

## §Trace v2.5.1

**F-P51-001 — session_not_ready producer-set corrected: detach-only; resize excluded** (2026-06-14):

- **Finding (F-P51-001, IMPORTANT/cross-doc normative contradiction):** §Post-spawn monitor step 8 and the Mapping table (v2.5.0) listed `resize_session` as a co-producer of `session_not_ready` alongside `detach_session`. This contradicts BC-2.05.010 Invariant 6 Exception (ResizePane carve-out): the ResizePane IPC arm WARN-drops ALL `resize_session()` errors and never sends `ServerToClient::Error`. `resize_session()` may return `Err(SessionNotReady)` at the session-manager level, but that result is WARN-dropped by the handler — `"session_not_ready"` can never appear on the wire from a resize. The v2.5.0 wording also implied both resize and detach were live wire producers, which is wrong.
- **Fix:** Step 8 rewritten to `Detach-during-Launching` only (resize carve-out rule restated inline). Mapping table row corrected to `detach_session` only, with explicit note that resize is excluded as a wire producer. Doc-comment in `SessionError::SessionNotReady` and the taxonomy-addition prose updated. Inline rationale (a)/(c) updated to be detach-only where wire-emission is the claim. Fix labels (f)/(h) in §Trace v2.5.0 updated to reflect detach-only wire producer.
- **Semver: patch (v2.5.0 → v2.5.1)** — errata correction of producer-set claim; no behavior change (the ResizePane WARN-drop carve-out was always normative; this pass simply aligns the prose to it).

---

## §Trace v2.5.0

**F-P50-001 — host_conn lifecycle during Launching is contradictory: post-spawn monitor + control-vs-proxy distinction + kill-during-Launching semantics** (2026-06-14):

- **Finding (F-P50-001, IMPORTANT/behavioral):** `SessionEntry.host_conn` doc-comment said "None if daemon is not currently attached (e.g., session-host just discovered)" — implying `host_conn` is `None` during `Launching`. But BC-2.08.003 PC-1 ("Running/Launching: uses existing host_conn") requires `host_conn: Some(_)` during Launching. The state transition table note "proxy task started" conflated the CONTROL connection (established during Launching) with the PTY-STREAMING proxy task (started at Launching→Running). BC-2.06.025 EC-293 makes kill-during-Launching reachable; if `host_conn` is genuinely `None`, the "use existing host_conn" path panics or silently fails. The degraded-env mechanism (`StateChanged { new_state: Launching, degraded_env }` as "first message to daemon") also requires a connection to exist during Launching.

- **Decision — canonical model (F-P50-001 adjudication):**
  `spawn_session()` starts a background **post-spawn monitor** task that polls for the session-host's UDS socket to become connectable (after session-host startup step 7 — socket bind). On connect + SO_PEERCRED verification, the daemon stores `host_conn: Some(SessionHostConnection { writer, proxy_task: None })`. This is the CONTROL connection — active during Launching and Running. The PTY-STREAMING proxy task (`host_conn.proxy_task: Some(...)`) is started ONLY at the Launching→Running transition (`StateChanged { new_state: Running }` received). BC-2.08.003 PC-1 ("Running/Launching: uses existing host_conn") is CORRECT. The state transition table note "proxy task started" referred to the PTY proxy task only — not the control connection. SessionEntry.host_conn "None" is correct for Detached and for re-discovered Terminating sessions only.

- **Fix (a) — `SessionEntry.host_conn` doc-comment rewritten:** Precise semantics for when `Some` vs `None`. Documents the CONTROL vs PTY-STREAMING distinction explicitly. References F-P50-001.

- **Fix (b) — `SessionHostConnection` doc-comment rewritten:** Notes `proxy_task: Option<JoinHandle<()>>` — `None` during Launching, `Some` after Running. Explains F-P50-001 kill-path use of writer during Launching.

- **Fix (b2) — `SessionHostConnection.proxy_task` type changed:** `JoinHandle<()>` → `Option<JoinHandle<()>>`. This is a normative change: `proxy_task` is `None` during Launching (control connection established but PTY streaming not yet started) and `Some(...)` during Running. Implementations MUST NOT unwrap `proxy_task` unconditionally.

- **Fix (c) — State transition table:** Two new rows: (1) "Post-spawn monitor: session-host UDS socket becomes connectable" → control connection established, `host_conn: Some(_)`, state still Launching; (2) "Daemon receives `StateChanged::Running`" → PTY proxy started, state → Running. The conflated single-row "proxy task started" entry is replaced by these two precise rows.

- **Fix (d) — §Pre-socket-bind orphan kill section extended:** Added "Kill-path host_conn rules" subsection disambiguating all three kill paths: Running (existing writer), Launching (existing writer OR PID-fallback if monitor not yet connected), Detached (fresh UDS connect). The Launching kill-before-socket-bind sub-case explicitly calls for PID fallback with `"kill_failed"` error code on failure.

- **Fix (e) — §Post-spawn monitor section added:** New subsection specifying all post-spawn monitor steps (poll/connect/SO_PEERCRED/host_conn population/StateChanged handling/kill-race/resize-detach-race). Includes two new integration test specs.

- **Fix (f) — `SessionNotReady` variant added to `SessionError`:** Defensive error for `detach_session()` called during Launching before control connection established (`host_conn: None`). Also returned by `resize_session()` at the session-manager level for the same condition, but the ResizePane IPC arm WARN-drops ALL resize errors — `SessionNotReady` is never emitted on the wire from resize. Wire producer: DetachSession arm only. NOT returned by `kill_session()`. Compiler-enforced: must add arm to `session_error_to_code()`. (F-P51-001: resize excluded from wire-producer claim.)

- **Fix (g) — `session_error_to_code()` extended:** New arm `SessionError::SessionNotReady { .. } => "session_not_ready"` added. Outer `SessionError` match remains compiler-enforced exhaustive.

- **Fix (h) — Mapping table row added:** `SessionNotReady` → `"session_not_ready"`, triggered by `detach_session` (wire producer). `resize_session` may return this variant internally but it is WARN-dropped by the IPC handler — not a wire producer. (F-P51-001.)

- **Fix (i) — Re-discovery Launching/Running prose clarified:** Added note that re-discovery does NOT replay the post-spawn monitor path; it connects and sends `DaemonToHost::Attach` directly (treating the session-host as already in Running). Also added `host_conn: Some(...)` to the re-discovery Running registration.

- **Fix (j) — degraded-env "first message" prose clarified:** Added explicit statement that the `StateChanged { Launching, degraded_env }` message is deliverable during Launching because the control connection is established before the session-host enters its event loop. References F-P50-001.

- **Semver: minor (v2.4.0 → v2.5.0)** — normative new `post-spawn monitor` mechanism; `SessionHostConnection.proxy_task` type change from `JoinHandle` to `Option<JoinHandle>`; new `SessionNotReady` variant; `"session_not_ready"` error code. Multiple normative behavior additions.

- **BC impact (report to product-owner for dispatch):**
  - **BC-2.08.003 PC-1 + Precondition 3:** Model is confirmed correct; exact reconciliation text provided in architect report F-P50-001 response.
  - **BC-2.05.010 (no-silent-failure):** `"session_not_ready"` must be added to the SS-ipc.md §ServerToClient::Error closed error-code set.

## §Trace v2.4.0

**F-P44-IMP-001 — `UnsupportedOperation` given dedicated `"spawn_unsupported"` arm; all 3 EngineError variants now explicitly mapped** (2026-06-14):

- **Finding (F-P44-IMP-001, IMPORTANT):** `EngineError::UnsupportedOperation` fell through the
  inner `_ => "invalid_request"` arm in `session_error_to_code()`. BC-2.03.008 PC-3 / EC-112
  mandated the distinct banner `"Session spawn not supported for this harness"`, which was
  undeliverable because the wire code collapsed to `"invalid_request"` → `"[operation failed]"`.
- **Decision — defense-in-depth / EC-112 reachable:** Capability filtering in ProfilePicker is
  best-effort. The daemon must surface a distinct error if `UnsupportedOperation` occurs at
  spawn-time regardless of pre-filtering. EC-112 is a reachable defensive path.
- **Fix (a) — new explicit arm:** `monocle_core::engine::EngineError::UnsupportedOperation(_) =>
  "spawn_unsupported"` added immediately after `InvalidPath` arm and before the `_ =>` catch-all.
  All 3 Phase 1 `EngineError` variants now map to distinct codes:
  `BinaryNotFound` → `"binary_not_found"`, `InvalidPath` → `"invalid_spawn_arg"`,
  `UnsupportedOperation` → `"spawn_unsupported"`.
- **Fix (b) — mapping table updated:** `EngineError(UnsupportedOperation)` row added. Catch-all
  row clarified: no longer swallows `UnsupportedOperation`; covers only truly novel future variants.
- **Fix (c) — prose updated:** Inner-match exhaustiveness description updated to name all three
  Phase 1 variants as explicitly routed. Forward-compat fallback note updated accordingly.
- Semver: minor (v2.3.0 → v2.4.0) — new explicit `EngineError::UnsupportedOperation` arm;
  normative routing change for the `UnsupportedOperation` path.

## §Trace v2.4.0 — Errata

**F-P48-001 — `SessionError::EngineError` variant doc-comment enumeration corrected from two items to three** (2026-06-14):

- **Finding (F-P48-001, IMPORTANT):** The `SessionError::EngineError(...)` enum-variant doc-comment
  enumerated only two bridged `EngineError` variants: `BinaryNotFound` (→ `"binary_not_found"`) and
  `InvalidPath` (→ `"invalid_spawn_arg"`). The Pass-44 fix (F-P44-IMP-001) added a third variant,
  `UnsupportedOperation` (→ `"spawn_unsupported"`), and propagated it to the mapping table
  (~line 440), exhaustiveness prose (~lines 468-470), `session_error_to_code()` doc-comment
  (~lines 502-503), and function body (~line 518) — but the variant doc-comment at lines 418-420
  retained the stale two-item enumeration, contradicting the same document 20-100 lines below.
- **Fix:** Variant doc-comment updated to enumerate all three bridged variants:
  `BinaryNotFound` (→ `"binary_not_found"`), `InvalidPath` (→ `"invalid_spawn_arg"`), and
  `UnsupportedOperation` (→ `"spawn_unsupported"`) (F-P44-IMP-001).
- **Semver:** ERRATA-NO-BUMP. The function body and all normative contract surfaces were already
  correct after F-P44-IMP-001. This is a doc-comment-only correction with no normative obligation
  change. Version remains v2.4.0.

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

---

## §S-033 Architectural Rulings — Session-Host Scope Boundary and Sidecar Schema Ownership

### Ruling A — monocle-session-host scope boundary: S-033 vs S-039/S-040/S-042/S-043/S-044

**Status:** AUTHORITATIVE. Supersedes any ambiguity in S-033 §File Structure lines 255–273 and §Tasks lines 194–217.

#### What S-033 MUST implement

The table below is the minimum viable session-host for S-033's ACs to pass. Every item here
is required to satisfy S-033 BC-2.08.001 and BC-2.08.008 (specifically AC-004 and AC-010:
the post-spawn monitor must observe SessionState::Running from the session-host over the
control connection).

| Step | What | Why required in S-033 |
|------|------|----------------------|
| Parse CLI args | `--session-id`, `--runtime-dir`, `--binary`, `--args`, `--env`, `--cwd` | `RealSessionHostSpawner` passes these; session-host must parse them to proceed |
| `nix::unistd::setsid()` | Process group leader | Session-host calls `nix::unistd::setsid()` at startup step 2 — this is an OS requirement, not optional. `pre_exec` is NOT used: `std::process::Command::pre_exec()` is `unsafe fn`; `monocle-runtime` forbids unsafe code. See §Ruling C. |
| Open PTY pair | `portable_pty::openpty(PtySize { rows:24, cols:80 })` | Required to obtain a valid PTY master/slave before spawning the harness child |
| Build `CommandBuilder` from recipe | Binary, args, env, cwd; env inheritance (I2-006) | Session-host startup step 4; needed to launch the harness child |
| Spawn harness child on PTY slave | `CommandBuilder` on the PTY slave fd | Needed to produce a `child_pid` for the sidecar and reach Running state |
| Initialize `vt100::Parser` | `vt100::Parser::new(24, 80, 1000)` stub | Present in session-host main event loop; needed to compile; parser.process() stubs may be no-ops at S-033 |
| Bind per-session UDS socket | `<runtime_dir>/session-<session_id>.sock` at `0o600` | Post-spawn monitor in S-033 polls for this socket to become connectable; S-033 AC-010 requires the Running transition, which requires the control connection, which requires the socket |
| Write sidecar | `session-state.json` at startup step 8 with `child_pid` populated | S-033 AC-003 requires sidecar to exist with `child_pid`; this is the session-host's write (with real child_pid) that completes the schema |
| Enter minimal event loop | Handle at least `DaemonToHost::Kill` and `StateChanged` | `StateChanged { new_state: Running, degraded_env: None }` MUST be sent after the harness child is spawned; post-spawn monitor requires this to transition SessionEntry to Running (AC-010) |
| Send `StateChanged { Running }` | Over the control connection (per-session UDS) | THIS IS THE AC-010 SIGNAL — required for S-033's post-spawn monitor to transition to Running |

#### What is deferred to later stories

| Feature | Deferred to | Story anchor |
|---------|------------|--------------|
| PTY output streaming to daemon (`HostToDaemon::PtyBytes`) | S-039 / S-040 | PTY output pipeline; IPC dispatch wiring for `PtyOutput` |
| `vt100::Parser.process()` live byte processing | S-039 | vt100::Parser integration |
| `DaemonToHost::Attach` scrollback dump (`ScrollbackChunk*` + `ScrollbackDumpComplete`) | S-035 | attach_session; this is the daemon-side `attach_session()` implementation |
| Keyboard forwarding (`DaemonToHost::KeyInput`) | S-047 | S-047 owns KeyInput / ResizePane / RenameSession IPC arms |
| PTY resize daemon leg (`ClientToServer::ResizePane` dispatch arm, `resize_session()`, `DaemonToHost::Resize`) | S-047 | S-047 AC-003 Tasks owns the IPC arm + `session_manager.resize_session()` + `DaemonToHost::Resize` forwarding. S-042 owns the TUI-side leg only (detection, debounce, send). The prior "S-042" entry in this row was erroneous — superseded by S-047 v1.1 IPC Handler Arm Ownership table. |
| PTY resize session-host leg (`DaemonToHost::Resize` handler in `monocle-session-host`) | S-042 | S-042 owns the session-host's `DaemonToHost::Resize` match arm: `pty.resize()` + `parser.set_size()`. This is the session-host binary handler, not the daemon IPC handler. |
| Session re-discovery (`DaemonToHost::Attach` on Detached session) | S-035 | attach_session() daemon side |
| `ScrollbackChunk*` / `ScrollbackDumpComplete` framing | S-035 | screen-state transfer on Attach |
| vt100 screen-state transfer (styled cells) | S-035 | attach scrollback serialization |
| TUI `PseudoTerminal` widget render | S-039 | monocle-tui side |
| AppMode::EmbeddedTerminal transitions + SessionCreation wizard | S-044 | SS-09 TUI side |

#### MockSessionHostSpawner + RealSessionHostSpawner: both required in S-033

`MockSessionHostSpawner` is required because all S-033 unit tests (AC-001..AC-012, AC-009,
AC-009b) use it. `RealSessionHostSpawner` is required because S-033 creates the
`monocle-session-host` crate and the spawner invokes it. Without `RealSessionHostSpawner`
the post-spawn monitor has no real binary to connect to, and AC-010 (Running transition)
cannot be exercised in integration.

**The `monocle-session-host` binary itself must be non-trivial for S-033 ACs to pass.**
A `main.rs` that is all `todo!()` stubs will NOT satisfy S-033: the post-spawn monitor
(AC-004 / AC-010) will never see the UDS socket become connectable, will timeout after 30s,
and will never receive `StateChanged { Running }`. S-033 AC-010 therefore requires the session-host
to implement AT MINIMUM the steps listed in the "What S-033 MUST implement" table above.

#### Can S-033 ACs be fully satisfied with MockSessionHostSpawner alone?

AC-001 through AC-009d can be satisfied with `MockSessionHostSpawner` only (no real binary
needed for those ACs). AC-010 and AC-004 require the post-spawn monitor to observe
`StateChanged { Running }` — with `MockSessionHostSpawner`, this means the mock must simulate
the post-spawn monitor handshake by emitting `StateChanged { Running }` through the in-process
test mechanism. If the mock returns a `SpawnedHostHandle` with a real UDS socket that the
test harness controls (injecting the `StateChanged` message into the socket), then AC-010 can
be tested without a real `monocle-session-host` binary.

**Ruling:** S-033 test-writer SHOULD use `MockSessionHostSpawner` for all unit tests of
AC-001..AC-009d (fast, isolated). AC-010 / the "Running transition" unit test
(`test_BC_2_08_001_spawn_session_entry_created_within_2s`) SHOULD use a mock that simulates
`StateChanged { Running }` over an in-process UDS. The `monocle-session-host` binary MUST be
implemented (not all `todo!()`) so that integration tests and the real binary path compile
and pass. The binary's PTY output streaming and scrollback dump are stubs (returning empty /
not-yet-implemented), but MUST NOT panic — they should return or no-op gracefully so the
binary can reach `StateChanged { Running }` in the test environment.

---

### Ruling B — SessionSidecar struct: crate residence, ownership protocol, schema agreement

**Status:** AUTHORITATIVE. Resolves HIGH-009 (dual-writer schema drift risk).

#### Ruling

The canonical `SessionSidecarV3` struct MUST live in `monocle-ipc`.

**Rationale:**
1. `monocle-runtime` (daemon) and `monocle-session-host` (binary) are two separate processes.
   They share NO in-process types. The only safe way to guarantee byte-level schema agreement
   between them is a shared crate that both processes depend on.
2. `monocle-ipc` is already the shared wire-type crate. `SessionState` (per S-033 ruling
   v1.4 — F-P16-IMP-001) already lives in `monocle-ipc`. `SessionSidecarV3` is a wire type
   in the same sense: it is a file-based handoff contract between daemon and session-host.
3. `monocle-runtime` MUST NOT depend on `monocle-session-host` (no binary→library dep
   in a workspace). `monocle-ipc` has no such constraint — both runtime and session-host
   already depend on it for `SessionState` and other IPC types.

#### Struct definition (canonical; placed in `monocle-ipc/src/lib.rs`)

```rust
/// session-state.json schema v3 — shared between monocle-runtime (writer at spawn) and
/// monocle-session-host (writer at startup step 8). Both crates import this from monocle-ipc.
/// Serde serialization is the byte-level schema agreement mechanism.
/// All reads (rediscover_sessions) also use this struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSidecarV3 {
    pub schema_version: u32,      // always 3 for this struct
    pub session_id: String,
    pub pid: u32,                  // session-host process PID
    pub socket_path: String,       // <runtime_dir>/session-<uuid>.sock
    pub child_pid: Option<u32>,    // harness child PID; None in daemon's initial write; populated by session-host
    pub state: SessionState,       // SessionState from monocle-ipc (already imported)
    pub project_root: String,
    pub cwd: String,
    pub harness_id: String,
    pub profile_id: String,
    pub started_at: String,        // ISO-8601 UTC; session-host uses chrono::Utc::now().to_rfc3339()
    pub display_name: String,
    pub pty_rows: u16,
    pub pty_cols: u16,
    #[serde(default)]
    pub kill_deadline_unix_ms: Option<u64>, // null unless state == Terminating
}
```

**Forward-compat note:** `#[serde(default)]` on `kill_deadline_unix_ms` ensures schema v1 and v2
sidecars can be deserialized into this struct (field absent → `None`). Schema version checking
(v1/v2/v3 branching logic) continues to live in `rediscover_sessions()` in `monocle-runtime` —
the struct is for v3 reads/writes; the daemon's re-discovery code handles schema migration to v3.

#### Ownership protocol: who writes which fields when

| Field | Writer | When |
|-------|--------|------|
| `schema_version: 3` | daemon (S-033 `spawn_session`) | Step 8 of daemon spawn path — initial atomic write |
| `session_id` | daemon | same |
| `pid` | daemon | same — `SpawnedHostHandle.pid` |
| `socket_path` | daemon | same — `<runtime_dir>/session-<session_id>.sock` |
| `child_pid: None` | daemon | same — `None` at initial write; session-host doesn't exist yet |
| `state: "Launching"` | daemon | same |
| `project_root`, `cwd`, `harness_id`, `profile_id`, `started_at`, `display_name`, `pty_rows`, `pty_cols` | daemon | same |
| `kill_deadline_unix_ms: null` | daemon | same |
| `child_pid: Some(pid)` | session-host | startup step 8 — session-host overwrites the sidecar AFTER opening the PTY and spawning the harness child, at which point `child_pid` is known |
| `state: "Running"` | session-host | NOT written by session-host to sidecar — the session-host signals Running via `HostToDaemon::StateChanged { new_state: Running }` over the per-session UDS; the daemon updates the in-memory `SessionEntry.state` and re-writes the sidecar with `state: "Running"` |

**Important clarification (no write race):** The daemon writes the sidecar FIRST (step 8 of
spawn, before the session-host exists). The session-host writes the sidecar SECOND (startup
step 8, after it has spawned the harness child and knows `child_pid`). The two writes are
SEQUENTIAL, not concurrent — the daemon spawns the session-host process, the session-host
starts up and overwrites the sidecar. Both use `tempfile::persist` (atomic rename).

There is no write race. The session-host's write cannot occur before the daemon's write
because the session-host process does not exist until after the daemon's write completes
and `spawn_session()` returns. The only failure mode to defend against is the session-host
writing a stale or partial `child_pid` — this cannot happen because `portable-pty` returns
the child PID synchronously before the session-host binds its UDS socket.

#### Atomic-write requirement (both writers)

Both the daemon and the session-host MUST write `session-state.json` via `tempfile::persist`
(NamedTempFile written in `runtime_dir`, then persisted to the target path). This is the
CLAUDE.md atomic-write convention and BC-2.08.006 Invariant 5 requirement. `std::fs::write`
is FORBIDDEN in both crates for this file.

`monocle-session-host` MUST add `tempfile = "3"` to its `Cargo.toml` (it is already used by
`monocle-runtime`). Both write to the same path; the atomic rename prevents concurrent readers
from seeing a partial file.

#### Path canonicalization (both writers)

The `runtime_dir` used to construct the sidecar path MUST be the canonicalized `runtime_dir`
passed as the `--runtime-dir` CLI argument to the session-host. The daemon passes the
canonicalized path (BC-2.08.006 Invariant 6 — runtime_dir is canonicalized at daemon startup
before any path is constructed from it). The session-host uses this value verbatim; it MUST NOT
re-canonicalize it (doing so would be redundant and could fail if the path contains symlinks
that resolve differently inside the session-host's mount namespace). `socket_path` in the sidecar
is `format!("{}/session-{}.sock", runtime_dir_arg, session_id)`.

#### Schema agreement guarantee

Because `SessionSidecarV3` is defined in `monocle-ipc` and both `monocle-runtime` and
`monocle-session-host` depend on `monocle-ipc`, any change to the struct definition will
produce a compile-time error in both crates simultaneously. This is the byte-level schema
agreement mechanism required by HIGH-009. No runtime version negotiation is needed for
same-release binaries; the `schema_version: 3` field provides the forward-compat
differentiation for cross-version (old daemon / new session-host) scenarios handled by
the re-discovery version-check logic.

#### S-036 re-discovery parseability

`rediscover_sessions()` (S-036) reads `session-state.json` files via `serde_json::from_str::<SessionSidecarV3>(&content)`.
Because both the daemon's initial write and the session-host's overwrite use `SessionSidecarV3`,
the re-discovery parser will always successfully deserialize any sidecar written by this
version of the binary, regardless of which writer last touched the file.

---

### Ruling C — setsid placement: session-host binary, not pre_exec in daemon

**Status:** AUTHORITATIVE. Amends Ruling A (setsid row) and `RealSessionHostSpawner` docstring. Resolves MED-003.

#### Decision

**Option (a) — session-host calls setsid() at startup** is the canonical approach.

`nix::unistd::setsid()` MUST be called by the `monocle-session-host` binary at startup step 2
(already specified in §startup sequence). `RealSessionHostSpawner::spawn()` in `monocle-runtime`
MUST NOT use `std::process::Command::pre_exec()`.

#### Rationale

1. **`#![forbid(unsafe_code)]` is non-negotiable.** `monocle-runtime` carries this attribute
   (confirmed in `crates/monocle-runtime/src/lib.rs` line 10). `std::process::Command::pre_exec()`
   is `unsafe fn` — it runs a closure in the forked child before exec, under POSIX
   async-signal-safety rules. `forbid(unsafe_code)` prohibits any `unsafe` block in the crate,
   including the single-line closure `unsafe { || { setsid(); Ok(()) } }`. Adding a crate-level
   `#[allow(unsafe_code)]` override for this one callsite would silently disable the safety
   invariant for the entire crate. A module-scoped `#[allow]` on a single function is
   technically possible but violates the spirit of the `forbid` gate and introduces a precedent
   for unsafe carve-outs. Neither option is production-grade.

2. **The session-host binary is NOT subject to `forbid(unsafe_code)` from `monocle-runtime`.**
   `monocle-session-host` is a separate binary crate. It can (and does) call
   `nix::unistd::setsid()` directly. `nix::unistd::setsid()` is itself a safe Rust wrapper —
   no `unsafe` block is required at all. The session-host calls it as step 2 of its startup
   sequence, which is before it opens any resources or spawns any threads.

3. **The pre-setsid race window is architecturally acceptable.** Between the moment the daemon
   forks the session-host process (via `Command::spawn()`) and the moment the session-host
   binary calls `setsid()` at its startup step 2, the child process shares the daemon's process
   group. This window is bounded by the OS loader time (typically <5ms on Linux/macOS).
   The risk the adversary raised is: could a signal sent to the daemon's process group during
   this window reach the session-host child?
   - **The daemon does not signal its own process group.** The daemon has no code path that
     sends a signal to its own pgid. SIGHUP (the relevant threat for terminal detachment) is
     sent by the kernel to a process group's controlling terminal — but the daemon already
     calls `setsid()` itself at startup (see `monocle-runtime/src/main.rs` line 53), so the
     daemon has no controlling terminal by the time any session-host is spawned. There is no
     SIGHUP source for the window.
   - **The window is not zero but the threat model is absent.** Accepting the brief pre-setsid
     window is consistent with ADR-0009's native-process-model decision. The claude-squad
     `pre_exec` pattern was adopted in the earlier spec text before the
     `#![forbid(unsafe_code)]` constraint was formalized; that pattern is now superseded.

4. **nix::unistd::setsid() is safe Rust — confirmed in the existing codebase.** The daemon
   already calls it at `crates/monocle-runtime/src/main.rs` line 53 without any `unsafe` block,
   with the comment "nix::unistd::setsid() is a safe Rust wrapper (BC-2.04.004 INV-2)".

#### Implementer instruction (Ruling C)

`RealSessionHostSpawner::spawn()` MUST NOT call `pre_exec` of any kind. It uses
`std::process::Command::spawn()` without `CommandExt::pre_exec`. The `monocle-session-host`
binary handles `setsid()` itself at startup step 2 via `nix::unistd::setsid()` (safe, no
`unsafe` block required). The implementation is already fully specified in §startup sequence
step 2 and in the Tasks checklist of S-033.

### Ruling D — host_conn writer storage: S-033 scope (not S-034)

**Status:** AUTHORITATIVE. Resolves HIGH-001. Confirms existing spec text; no spec content changes.

#### Decision

Establishing and storing the live control-connection writer (`host_conn = Some(SessionHostConnection { writer, proxy_task: None })`) is **part of S-033**, not S-034.

#### Rationale

The post-spawn monitor is spawned by `spawn_session()` in S-033. Its job — specified in
§Post-spawn monitor steps 1–3 — is to poll until the session-host's UDS socket becomes
connectable, apply SO_PEERCRED, and store `host_conn: Some(SessionHostConnection { writer, proxy_task: None })` into the `SessionEntry` under the `SessionManager` mutex. This is
unambiguously step 3 of the post-spawn monitor, which is S-033's `tokio::spawn` background
task.

S-034 (`kill_session`) USES the established `host_conn`. The kill-path F-P50-001 rules state:
for `Running` state, `kill_session()` uses the existing `host_conn.writer` — it does NOT
open a new connection. For `Launching` state with `host_conn: Some(_)` (monitor already
connected), same: uses the existing writer. S-034 is a consumer of the connection S-033
establishes.

The implementer's decision to drop the writer with "stored in S-034" is a spec compliance
defect. It would leave `host_conn: None` throughout the session's Launching phase, which
breaks:
- BC-2.08.001 postcondition 2 / S-033 AC-002 assertion: `host_conn` is `None` at
  `spawn_session()` return, then transitions to `Some(_)` when the monitor connects — this
  transition MUST happen in S-033's monitor task.
- The kill-during-Launching path (§Post-spawn monitor step 7): if `kill_session()` arrives
  when `host_conn` is `Some(_)` (monitor already connected), it MUST use the writer — which
  only exists if S-033's monitor stored it.
- AC-010: the Running transition requires the monitor to receive `StateChanged{Running}` over
  the control connection, which requires the monitor to have already established `host_conn`.

#### Implementer instruction (Ruling D)

S-033's post-spawn monitor task MUST complete all of steps 1–5 from §Post-spawn monitor:
1. Poll for UDS socket connectable.
2. Apply SO_PEERCRED.
3. Store `host_conn: Some(SessionHostConnection { writer, proxy_task: None })` under mutex.
4. Receive `StateChanged` messages (including degraded_env handshake).
5. On `StateChanged{Running}`: start PTY proxy task, set `host_conn.proxy_task`, transition
   to Running, emit `SessionStateChanged{Running}` + `SessionListUpdate`.

The implementer MUST NOT defer step 3 (writer storage) to S-034. S-034's `kill_session()`
uses the writer established in step 3 — it has nothing to establish.

---

### Ruling E — DaemonToHost crate residence: monocle-ipc, S-033 scope

**Status:** AUTHORITATIVE. Resolves MED-001. Supplements Ruling D.

#### Decision

`DaemonToHost` and `HostToDaemon` enums MUST be defined in `monocle-ipc` in S-033 scope.
Deferral to S-034 or any later story is NOT permitted.

#### Rationale

Ruling D establishes that S-033's post-spawn monitor stores `host_conn: Some(SessionHostConnection { writer, proxy_task: None })`. The `writer` is typed as `Arc<Mutex<UnixStream>>`. To write to the control connection, the implementer must frame `DaemonToHost` messages as length-prefixed JSON. Without `DaemonToHost` defined in a shared crate, the writer has no typed contract — the implementer is forced to either define an ad-hoc local type (creating a schema-drift risk identical to the one Ruling B resolved for `SessionSidecarV3`) or hand-write raw bytes (error-prone and unreviewed).

The full `DaemonToHost` enum is already specified in §Per-session UDS protocol above:
`Attach`, `KeyInput`, `Resize`, `Kill`, `Detach`. The full `HostToDaemon` enum is also
specified: `ScrollbackChunk`, `ScrollbackDumpComplete`, `PtyBytes`, `StateChanged`,
`Goodbye`, `PtyReset`. Both share the same length-prefix framing contract and both cross
the daemon/session-host process boundary — they are wire types by definition.

`monocle-ipc` is the shared wire-type crate; `SessionState`, `SessionSidecarV3`, and all
daemon↔TUI wire types already live there. `DaemonToHost` and `HostToDaemon` belong there for
the same reason. The post-spawn monitor (S-033) is the first code path that reads
`HostToDaemon` messages from the session-host. The post-spawn monitor MUST exist in S-033.
Therefore `HostToDaemon` (and its companion `DaemonToHost`) MUST exist in `monocle-ipc`
before S-033 implementation is complete.

#### Boundary ruling on variant completeness

S-033 MUST define both enums with ALL variants listed in §Per-session UDS protocol. Variants
whose handler is deferred (e.g., `Attach`, `KeyInput`, `Resize` implementations live in
S-035/S-047/S-042) are still PRESENT in the enum definition with `#[non_exhaustive]` on
`DaemonToHost` — the session-host receives messages with `todo!()` or `no-op` handler stubs
in S-033. The enum definition is the contract; the handlers are the implementation.

#### Implementer instruction (Ruling E)

Add to `crates/monocle-ipc/src/lib.rs` (or a new `crates/monocle-ipc/src/session_protocol.rs`
module re-exported from `lib.rs`):

```rust
/// Messages the daemon sends to the session-host over the per-session UDS control connection.
/// Same 4-byte LE u32 length-prefix framing as the daemon UDS (SS-ipc.md §Framing).
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonToHost {
    Attach,
    KeyInput { bytes: Vec<u8> },
    Resize { rows: u16, cols: u16 },
    Kill,
    Detach,
}

/// Messages the session-host sends to the daemon over the per-session UDS control connection.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostToDaemon {
    ScrollbackChunk { rows: Vec<Vec<monocle_ipc::SerializedCell>>, chunk_seq: u32 },
    ScrollbackDumpComplete { total_chunks: u32, cursor_row: u16, cursor_col: u16, pty_rows: u16, pty_cols: u16 },
    PtyBytes { bytes: Vec<u8> },
    StateChanged {
        new_state: SessionState,
        #[serde(default)]
        degraded_env: Option<Vec<String>>,
    },
    Goodbye,
    PtyReset,
}
```

`HostToDaemon::ScrollbackChunk` references `SerializedCell` which is already defined in
`monocle-ipc`. Use the crate-internal type directly (no `monocle_ipc::` prefix needed inside
the crate). Both enums MUST be exported at the `monocle_ipc` crate root.

---

### Ruling F — EC-152 UUID retry locus: IPC handler is the SINGLE retry locus

**Status:** AUTHORITATIVE. Resolves MED-002. Amends the collision-handling text in §Pre-socket-bind orphan kill and makes EC-152 unambiguous.

#### Decision

The IPC handler (the `ClientToServer::SpawnSession` match arm) is the SINGLE canonical retry
locus for UUID v4 collisions. `spawn_session()` MUST NOT perform its own retry loop.

#### Rationale

F-P41-IMP-001 (BC-2.08.001) established that UUID generation lives in the IPC handler,
not inside `spawn_session()`. `spawn_session()` receives an already-generated `session_id` via
`opts.with_daemon_fields()`. If `spawn_session()` detects a collision (session_id already in
registry), the correct response is `Err(SessionError::SessionIdCollision { session_id })` — it
cannot regenerate a UUID because UUID generation is not its responsibility.

The IPC handler, on receiving `Err(SessionError::SessionIdCollision)`, generates a new UUID,
rebuilds `opts` via `opts.with_daemon_fields(new_uuid, hooks_path)`, and retries
`spawn_session()` ONCE. If the retry also returns `SessionIdCollision`, the IPC handler sends
`ServerToClient::Error { code: "session_id_collision", ... }` to the client. Total UUID
attempts: 2 (initial + 1 retry). This matches BC-2.08.001 EC-152 ("retry once; second
collision → error") exactly.

#### Double-retry prohibition

`spawn_session()` MUST NOT contain its own UUID regeneration loop or retry. Doing so would
produce up to 4 UUID attempts total (IPC handler retries once → `spawn_session()` retries once
= 4 attempts), which violates EC-152's "auto-retry is one attempt" constraint.

The `session_id` insertion collision at line 403 of the §Pre-socket-bind orphan kill section
references "second retry" — this refers to the IPC handler's second attempt (the single retry),
not a second retry inside `spawn_session()`. The description at line 403 is correct in spirit
but could be read as endorsing an internal retry. This ruling clarifies: **no internal retry
inside `spawn_session()`**.

#### Implementer instruction (Ruling F)

`spawn_session()` implementation:
```rust
// On detecting session_id collision in the registry:
if self.sessions.contains_key(&opts.session_id) {
    return Err(SessionError::SessionIdCollision { session_id: opts.session_id });
}
// NO retry loop here. UUID retry is the IPC handler's responsibility.
```

IPC handler implementation:
```rust
ClientToServer::SpawnSession { opts } => {
    let session_id = uuid::Uuid::new_v4().to_string();
    let _ = client_tx.send(ServerToClient::SpawnAck { session_id: session_id.clone() }).await;
    let opts = opts.with_daemon_fields(session_id, state.hooks_settings_path.clone());
    let result = state.session_manager.lock().await.spawn_session(opts.clone()).await;
    let result = match result {
        Err(SessionError::SessionIdCollision { .. }) => {
            // Single retry: generate a new UUID.
            let retry_id = uuid::Uuid::new_v4().to_string();
            let retry_opts = opts.with_daemon_fields(retry_id, state.hooks_settings_path.clone());
            state.session_manager.lock().await.spawn_session(retry_opts).await
        }
        other => other,
    };
    match result {
        Ok(_) => { /* broker emits SessionStateChanged{Launching} + SessionListUpdate */ }
        Err(e) => {
            let _ = client_tx.send(ServerToClient::Error {
                code: session_error_to_code(IpcOp::Spawn, &e).to_string(),
                message: e.to_string(),
            }).await;
        }
    }
}
```

Note: the `SpawnAck` is sent with the FIRST UUID before the retry path is known. If the retry
uses a different UUID, the TUI's `launching_session_id` (set from `SpawnAck`) will be stale.
The IPC handler MUST send a second `SpawnAck` with the retry UUID before calling the retry
`spawn_session()`, so the wizard tracks the correct session_id. Add to the retry arm:
```rust
let _ = client_tx.send(ServerToClient::SpawnAck { session_id: retry_id.clone() }).await;
```

---

### Ruling G — Running-transition broadcast-pair atomicity: single lock across both try_send calls

**Status:** AUTHORITATIVE. Resolves MED-003.

#### Decision

The post-spawn monitor MUST hold the `SessionManager` mutex continuously across BOTH
`try_send()` calls for the Running-transition broadcast pair (`SessionStateChanged{Running}` +
`SessionListUpdate`). Per-client FIFO alone is NOT sufficient to satisfy BC-2.08.008 Invariant 4
when the sender is a detached background task.

#### Rationale

BC-2.08.008 Invariant 4 states:

> The mutex provides the atomicity window that prevents any other actor from interleaving posts
> between the two `.try_send()` calls.

The `spawn_session()` path holds the lock during the Launching-transition pair (step 5 of the
IPC handler pattern). The post-spawn monitor is a detached `tokio::spawn` task; it re-acquires
the mutex for step 3 (store `host_conn`) and again for step 5 (Running transition). If step 5
acquires the mutex, broadcasts `SessionStateChanged{Running}`, releases the mutex, then
re-acquires to broadcast `SessionListUpdate`, a concurrent spawn on another client could
interleave its `SessionStateChanged{Launching}` between the two Running broadcasts.

The invariant requires the Running pair to be non-interleaved by any third-party state
mutation. This is only achievable if one mutex acquisition spans both `try_send()` calls.

#### Implementer instruction (Ruling G)

In the post-spawn monitor, step 5 must be structured as a SINGLE mutex lock scope:

```rust
// Step 5: Running transition + broadcast pair, under ONE lock acquisition.
let mut sessions = self.sessions.lock().await; // one acquire
let entry = sessions.get_mut(&session_id).ok_or(...)?;
entry.state = SessionState::Running;
entry.host_conn = Some(SessionHostConnection {
    writer: entry.host_conn.take().unwrap().writer,
    proxy_task: Some(tokio::spawn(pty_proxy_task(...))),
});
let snapshot = SessionSnapshot::from(entry);
// Both try_send calls happen while sessions lock is held:
self.broker.broadcast(Event::SessionStateChanged { session_id: session_id.clone(), new_state: SessionState::Running });
self.broker.broadcast(Event::SessionListUpdate { sessions: vec![snapshot] });
// lock released here (end of scope)
```

The same requirement applies to the EC-163 (UDS connect fails) and `StateChanged{Terminated}`
(spawn-path failure) termination paths in the post-spawn monitor: each termination pair
(`SessionStateChanged{Terminated}` + `SessionListUpdate`) MUST also be emitted under a single
lock acquisition.

This does NOT extend to the degraded-env metadata update (step 4, `StateChanged{Launching}` handshake):
that update sets `entry.degraded` and `entry.degraded_reason` only, with no state transition
and no `SessionStateChanged` emission. It may acquire and release the mutex independently.

---

### Ruling H — Outer SessionManager mutex held across bounded UDS connect: accepted tradeoff (MED-001)

**Status:** AUTHORITATIVE. Documents the accepted locking discipline for the kill path. Closes MED-001 so future adversary passes do not re-raise it as an unresolved defect.

#### Decision

Holding the outer `Mutex<SessionManager>` across a bounded (2s) `UnixStream::connect` on the
`FreshConnect` path (Detached kill) and the ExistingConn broken-connection fallback path is an
**accepted tradeoff**. No code change is required for this specific behavior. Future adversary
passes MUST NOT re-raise this as a defect without citing a new failure mode not addressed here.

#### What is held and what is not

- The **outer `Mutex<SessionManager>`** (the `sm.lock()` in `handle_kill_session` in
  `ipc_server.rs`) IS held for the duration of `kill_session()`.
- The **inner `self.sessions: Mutex<HashMap<...>>`** (per-session registry) is NOT held during
  the UDS connect. `kill_session()` releases the inner lock before calling
  `tokio::time::timeout(2s, UnixStream::connect(...))`. The connect itself holds only the outer
  struct lock, not the per-session registry lock.

#### Rationale

Three independent reasons make the accepted tradeoff correct:

1. **State consistency across the kill sequence.** Releasing the outer lock before the
   fresh connect and re-acquiring afterward would require a second lookup and a stale-state
   check. The session entry (state, pid, socket path) could be mutated by a concurrent
   `kill_session` for the same session, a GC timer removing the entry, or a spawn that recycles
   a GC'd session_id. The current design holds the outer lock continuously, making all three
   concurrent mutations structurally impossible during the kill operation. The correctness gain
   of holding the lock outweighs the 2s serialization cost.

2. **Bound is tight, explicit, and scoped to rare paths.** The 2s `tokio::time::timeout` is
   applied uniformly to all fresh-connect operations (`MED-004` in the code). The FreshConnect
   path is taken ONLY for `Detached` sessions. The ExistingConn fallback is taken only when the
   existing control connection's write fails — a rare broken-connection scenario. The common
   path (Running/Launching ExistingConn SUCCESS) performs no network I/O at all under the outer
   lock: it writes to an already-established UDS connection, which is a local memory operation.
   Under normal operation, the outer lock is held for microseconds. The 2s worst-case applies
   only to Detached kill against a stale socket.

3. **Symmetry with the spawn pattern (S-033).** `spawn_session()` also acquires the outer
   lock and performs OS-level process spawn inside it. Changing only the kill path to
   release-before-connect would create an asymmetric locking model. A principled
   release-before-I/O refactor would need to cover both spawn and kill simultaneously and
   introduce a state-capture-before-release idiom. That is a cross-story redesign beyond
   S-034 scope.

#### Operational impact

- **Worst case:** one Detached kill against a stale socket serializes all other session IPC
  operations for up to 2s. This is a control-plane responsiveness cost, not a correctness
  violation or deadlock risk.
- **PTY byte forwarding is NOT affected.** PTY forwarding uses the broker (`Arc<Broker<Event>>`)
  and per-client subscription channels — these paths never acquire the outer `Mutex<SessionManager>`.
  TUI rendering continues uninterrupted during a stale-socket kill.
- **Bounded, no deadlock.** The UDS connect timeout is hard (2s). Even in the worst case, the
  outer lock is released when `kill_session()` returns `Ok(())`.

---

### Ruling I — ExistingConn kill path: kill_confirm_monitor is MANDATORY (OBS-001)

**Status:** AUTHORITATIVE. Amends §kill_confirm_monitor implementation requirement.
Closes OBS-001 definitively.

#### Decision

On the ExistingConn SUCCESS path (Running/Launching kill where `host_conn.writer` is present
and the Kill write succeeds), `kill_confirm_monitor` MUST be spawned and MUST take ownership
of `host_conn.reader`. Relying on the still-running `post_spawn_monitor` to read the
`StateChanged{Terminated}` confirmation is **forbidden**.

#### Why post_spawn_monitor reuse is unacceptable

`post_spawn_monitor` is a background task that exits when its read loop breaks. The read loop
can break due to EOF or any I/O error on the control connection's read half. This is a normal
occurrence — the session-host loops through `listener.accept()` cycles (Phase A / Phase B per
§Main event loop), and a transient EOF on the control connection causes the session-host to
transition from Phase B back to Phase A. The daemon's post_spawn_monitor observes EOF on its
reader and exits cleanly (per `match read_result { Err(UnexpectedEof) => break }`).

After `post_spawn_monitor` exits, `host_conn.writer` MAY still be `Some` (the TCP write half
can appear live while the read half is dead, particularly if the write is buffered). When
`kill_session()` then fires:

1. It finds `host_conn.writer: Some(...)` — ExistingConn path taken.
2. Kill is sent successfully over the writer.
3. Session transitions to Terminating; watchdog spawned.
4. The session-host receives Kill, sends `StateChanged{Terminated}` on the same connection.
5. Nobody reads the reply — `post_spawn_monitor` has already exited; `host_conn.reader` is
   `None` (the implementation set it to `None` at store time).
6. Result: a session-host that confirmed cleanly within 1s is still SIGKILL'd by the 12s
   watchdog. The clean confirmation is silently discarded.

Under the production-grade default this corner is a defect, not an accepted tradeoff: the
always-monitored design is straightforwardly implementable (move the reader into
`kill_confirm_monitor`), closes the gap unconditionally, and is what the spec mandated.

#### Amended implementation requirement

The correct implementation for the ExistingConn SUCCESS path:

1. **`post_spawn_monitor` stores `host_conn.reader: Some(reader)`** (NOT `None`) at the
   `Running` transition — i.e., immediately upon receiving `StateChanged { new_state: Running }`
   and before breaking from the read loop. Storing at connect time is incorrect: the reader is
   still actively consumed by the monitor's own message loop at that point, and placing it in
   `host_conn.reader` before the loop exits would create a double-consumer on the read half.
   `host_conn.reader` is `None` before the `Running` transition; a kill arriving before
   `Running` takes the watchdog-only fallback (no reader to hand to `kill_confirm_monitor`).
   The `reader` local variable from `stream.into_split()` is placed in `host_conn.reader` as
   the monitor's last act before returning.

2. **`post_spawn_monitor` exits immediately after observing `StateChanged{Running}`** — it
   sets `session_is_running = true` and then breaks from the read loop (or returns). It does
   NOT continue reading post-Running. Post-Running, the reader belongs to `host_conn.reader`
   and must not be consumed by the monitor.

3. **`kill_session()` on ExistingConn SUCCESS path**: after writing Kill and calling
   `transition_to_terminating`, the implementation MUST:
   - `take()` `host_conn.reader` (via `entry.host_conn.as_mut().and_then(|c| c.reader.take())`).
   - Spawn `kill_confirm_monitor(session_id, reader, sessions_arc, broker_arc, sidecar_path)`.
   - This is identical to what the Detached/FreshConnect path already does, just with the
     existing reader instead of a fresh-connect reader.

4. **Remove the erroneous "Note: the post_spawn_monitor task continues reading after
   StateChanged{Running}..." comment** from `kill_session()`. That comment described the
   forbidden reuse design.

The Detached/FreshConnect path is unchanged: fresh connect supplies a new reader, which is
immediately handed to `kill_confirm_monitor`.

---

### Ruling J — Watchdog force-kill must kill BOTH session-host AND harness child (F-S034-ADV-MED-001)

**Status:** AUTHORITATIVE. Amends §12-second watchdog timer and BC-2.08.003 PC-5b.

#### Process topology — verified against portable_pty 0.9.0 source

The harness child is spawned via `portable_pty::UnixSlave::spawn_command()`. Inspection of
`portable-pty-0.9.0/src/unix.rs` (line 257) confirms that `spawn_command` calls `libc::setsid()`
inside its `pre_exec` closure on the harness child. This means:

- **Session-host process:** calls `nix::unistd::setsid()` at startup step 2.
  Session-host PGID = session-host PID = session-host SID. Process Group A, Session A.
- **Harness child process:** `portable_pty::spawn_command()` calls `libc::setsid()` in its
  own `pre_exec`. Harness child PGID = harness child PID = harness child SID.
  Process Group B, Session B (entirely separate from Group A / Session A).

**Consequence:** The harness child is in a different process group AND a different session from
the session-host. Neither of the following reaches the harness child:

- `kill(Pid::from_raw(session_host_pid), SIGKILL)` — sends to the session-host only.
- `kill(Pid::from_raw(-(session_host_pid as i32)), SIGKILL)` / `killpg(session_host_pgid, SIGKILL)`
  — sends to Process Group A (the session-host's group), which does NOT contain the harness
  child (it is in Group B).

#### The gap

When the 12-second watchdog fires (session-host stalled / unresponsive — the watchdog's
purpose is to handle this exact case), the watchdog sends `SIGKILL` only to
`entry.session_host_pid`. If the session-host dies without running its own `kill_sequence()`
(e.g., it is stuck), the harness child is NOT killed. The harness child is reparented to
`init` (PID 1) or the system subreaper, continues executing, and remains attached to a now-
orphaned PTY master whose owning process is dead. This is a process leak and a PTY resource
leak. BC-2.08.003 PC-5b states that SIGKILL is sent "to release PTY resources" — that goal is
not achieved by the current single-PID kill.

#### Correct watchdog kill mechanism

The watchdog MUST kill both the session-host and the harness child. The daemon has
`session_host_pid` in `SessionEntry`. The `child_pid` is available in the sidecar
(`session-state.json` field `child_pid`, written by the session-host at startup step 8).

**Mechanism (mandatory):** On 12-second watchdog timeout, AFTER confirming the session is
still `Terminating` (ESRCH guard per existing spec), the daemon MUST:

1. Send `SIGKILL` to `session_host_pid` (as today — kills the session-host).
2. Read `child_pid` from the sidecar (`session-state.json`). If present (`Some(pid)`):
   send `SIGKILL` to `child_pid`. Apply the same ESRCH guard: if `kill(child_pid, SIGKILL)`
   returns `ESRCH`, the child already exited — benign, log DEBUG.
3. If `child_pid` is `None` in the sidecar (the session-host crashed before writing it),
   log WARN: `"watchdog: child_pid not in sidecar — harness child may be orphaned"`. This
   cannot be recovered without the child PID, but the session-host is dead so no further PTY
   output is possible; the orphaned process will be visible in system tools.

The sidecar read for `child_pid` is a best-effort I/O operation performed OUTSIDE any mutex.
It uses the same sidecar path already held by the watchdog task (`sidecar_path` parameter).

**Why reading child_pid from the sidecar is correct:**

- The sidecar is a stable file at a well-known path — the session-host writes `child_pid`
  atomically via `tempfile::persist` at startup step 8, before entering the event loop.
- By the time the 12-second watchdog fires, the sidecar MUST contain `child_pid` if the
  session-host ever reached Running state (which it must have, since the daemon received
  `StateChanged{Running}` and transitioned the session to `Running` or at minimum `Launching`
  with a live connection). If the session-host crashed before step 8, the harness child was
  never spawned, so there is nothing to kill.
- The sidecar read is race-free for watchdog purposes: the session-host is unresponsive (that
  is why the watchdog is firing); it is not writing the sidecar concurrently.

**Why NOT adding `child_pid` to `SessionEntry`:**

Adding `child_pid: Option<u32>` to `SessionEntry` would require the daemon to either (a) read
the sidecar at the `Launching → Running` transition (already done — the post-spawn monitor
reads `existing_child_pid` from the sidecar at the Running transition, line ~1507 of mod.rs)
and populate the field, or (b) receive `child_pid` from the session-host via an IPC message.
Path (a) is possible and is the preferred design for a later consolidation. For S-034 scope:
the sidecar read in the watchdog is the minimal-change path that provably closes the gap
without expanding scope into SessionEntry schema changes, IPC protocol additions, or re-
discovery logic. The implementer MAY populate `SessionEntry.child_pid: Option<u32>` from the
Running-transition sidecar read if they prefer; this is equivalent and avoids the sidecar I/O
in the watchdog hot path. Either approach is compliant with this ruling.

#### S-034 in-scope confirmation

The 12-second watchdog is S-034 code (`kill_session()` spawns the watchdog task). The
watchdog force-kill path is entirely within S-034 scope. The production-grade default applies:
fix in scope.

The session-host `kill_sequence()` (which correctly kills the harness child via SIGTERM →
SIGKILL with its own `child_pid`) is the non-watchdog path and is unaffected by this ruling.
The watchdog-only gap is in the daemon-side watchdog, which must not assume the session-host
ran `kill_sequence()`.

#### Updated watchdog pseudocode

```rust
// 12-second watchdog expiry:
let should_sigkill = {
    let guard = sessions.lock().await;
    matches!(guard.get(&session_id).map(|e| &e.state), Some(SessionState::Terminating))
};
if !should_sigkill { return; }

let pid = { sessions.lock().await.get(&session_id).map(|e| e.session_host_pid) };
if let Some(pid) = pid {
    // Kill session-host (as before).
    match nix::sys::signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
        Ok(()) => {}
        Err(nix::errno::Errno::ESRCH) => { tracing::info!(..., "watchdog: session-host already exited"); }
        Err(e) => { tracing::warn!(..., "watchdog: session-host SIGKILL failed"); }
    }
    // NEW: also kill harness child (portable_pty gives child its own session — single-PID
    // kill above does NOT reach it).
    let child_pid: Option<u32> = std::fs::read_to_string(&sidecar_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v["child_pid"].as_u64())
        .map(|n| n as u32);
    match child_pid {
        Some(cpid) => {
            match nix::sys::signal::kill(Pid::from_raw(cpid as i32), Signal::SIGKILL) {
                Ok(()) => { tracing::warn!(..., "watchdog: SIGKILL sent to harness child"); }
                Err(nix::errno::Errno::ESRCH) => { tracing::debug!(..., "watchdog: harness child already exited"); }
                Err(e) => { tracing::warn!(..., "watchdog: harness child SIGKILL failed"); }
            }
        }
        None => {
            tracing::warn!(..., "watchdog: child_pid not in sidecar — harness child may be orphaned");
        }
    }
    force_terminate_session(&session_id, &sessions, &broker, &sidecar_path).await;
}
```

---

### Ruling L — proxy_task is the kill-confirm reader for attached sessions (F-S035-001)

**Status:** AUTHORITATIVE. Resolves F-S035-001 (MEDIUM, cross-story-integration). Amends
§spawn_pty_proxy_task and §kill_session ExistingConn reader=None path. S-035 in-scope fix.

#### Finding

After `attach_session()` completes, `host_conn` stores
`SessionHostConnection { writer, reader: None, proxy_task: Some(handle) }`. The reader is
owned exclusively by the proxy_task. When `kill_session()` subsequently fires on a Running
(post-attach) session, `host_conn.reader.take()` returns `None`. The existing code falls to
the watchdog-only branch, bypassing `kill_confirm_monitor`. The clean `StateChanged{Terminated}`
confirmation (target: <500ms per BC-2.08.003) is degraded to the 12-second SIGKILL watchdog.

Additionally: the proxy_task's `other =>` arm discards `HostToDaemon::StateChanged{Terminated}`
silently. The `Goodbye` arm breaks the loop but does not call `transition_to_terminated`. Both
are defects: an attached session whose host sends `StateChanged{Terminated}` or `Goodbye`
without a prior kill (natural exit path, once S-039/S-040 wires it) would leave the session
stuck in `Running` with no state transition published.

#### Dependency analysis — why this is S-035 in-scope (not deferred to S-039/S-047)

S-039/S-040 own: PTY output fan-out (`PtyBytes` → broker → TUI), PTY reader task, harness
child exit detection (`child_exit_watch` arm). S-047 owns: `KeyInput`, `Resize`, live
interaction commands. None of these are prerequisites for handling `StateChanged{Terminated}`
in the proxy_task — the proxy_task already holds the reader and the broker, and
`transition_to_terminated` (the GC path) is existing S-034 code. The fix is a purely additive
match arm in S-035's own `spawn_pty_proxy_task`. No new IPC variants required.

#### Normative changes (S-035 implementer must implement)

**Change L-1 — `spawn_pty_proxy_task` signature extended:**

```rust
fn spawn_pty_proxy_task(
    session_id: String,
    reader: tokio::net::unix::OwnedReadHalf,
    broker: Arc<monocle_ipc::server::SubscriberList>,
    sessions: Arc<tokio::sync::Mutex<HashMap<String, SessionEntry>>>,   // NEW
    sidecar_path: PathBuf,                                               // NEW
) -> JoinHandle<()>
```

Both call sites (post-spawn monitor Running transition and `attach_session`) MUST pass the new
parameters.

**Change L-2 — proxy_task match arms for state-change messages:**

The `other =>` discard arm MUST be replaced with explicit handling:

```rust
HostToDaemon::StateChanged { new_state: SessionState::Terminated, .. } => {
    // Session-host confirmed termination on the attach connection.
    // Publish Terminated transition (BC-2.08.008 I4: StateChanged BEFORE SessionListUpdate).
    // This is the fast-path kill confirmation for attached sessions (<500ms).
    tracing::info!(session_id = %session_id,
        "proxy_task: StateChanged{Terminated} received — transitioning session to Terminated");
    force_terminate_session(&session_id, &sessions, &broker, &sidecar_path).await;
    break;
}
HostToDaemon::Goodbye => {
    // Session-host closed the connection. If StateChanged{Terminated} was already received
    // above, this branch is unreachable (loop already broke). If Goodbye arrives without
    // a prior Terminated: session-host exited unexpectedly (natural exit, S-039/S-040 path,
    // or protocol violation). Transition to Terminated defensively.
    tracing::debug!(session_id = %session_id,
        "proxy_task: Goodbye received; transitioning session to Terminated");
    force_terminate_session(&session_id, &sessions, &broker, &sidecar_path).await;
    break;
}
other => {
    // Unexpected HostToDaemon variant received post-Running on proxy connection.
    // ScrollbackChunk / ScrollbackDumpComplete should not arrive here (scrollback dump
    // completed before proxy_task was started). StateChanged{non-Terminated} is logged
    // at WARN — protocol invariant violation (session-host should only send Running once).
    tracing::warn!(
        session_id = %session_id,
        msg_type = ?std::mem::discriminant(&other),
        "proxy_task: unexpected HostToDaemon variant (post-Running); ignoring"
    );
}
```

**Change L-3 — `kill_session` ExistingConn reader=None path — distinguish proxy-owned from
pre-Running race:**

The existing `else` branch (reader=None fallback, line ~1622) handles two distinct cases that
MUST be distinguished:

```rust
if let Some(existing_reader) = maybe_reader {
    // ExistingConn SUCCESS path — reader present (post-spawn-monitor path).
    // ... (unchanged: spawn kill_confirm_monitor) ...
} else {
    // reader is None. Two sub-cases:
    let has_proxy = {
        let guard = self.sessions.lock().await;
        guard.get(session_id)
            .and_then(|e| e.host_conn.as_ref())
            .map(|c| c.proxy_task.is_some())
            .unwrap_or(false)
    };
    if has_proxy {
        // Attached session: proxy_task owns the reader and will handle
        // StateChanged{Terminated} via Change L-2. Do NOT abort the proxy_task.
        // The watchdog below provides the 12s fallback if the session-host is unresponsive.
        // Expected fast path: proxy_task receives StateChanged{Terminated} within 500ms
        // and calls force_terminate_session. Watchdog finds state != Terminating and exits.
        tracing::debug!(
            session_id = %session_id,
            "kill_session ExistingConn: reader=None, proxy_task=Some — \
             Terminated handling delegated to proxy_task (F-S035-001 fix)"
        );
    } else {
        // Pre-Running race: host_conn established but monitor has not yet stored reader
        // (StateChanged{Running} not yet received). Watchdog-only path. Unchanged.
        tracing::debug!(
            session_id = %session_id,
            "kill_session ExistingConn: reader=None, proxy_task=None — watchdog-only path \
             (pre-Running race)"
        );
    }
}
```

**The proxy_task MUST NOT be aborted.** Aborting would race with an in-flight
`StateChanged{Terminated}` message. The correct design: leave the proxy_task running; it
self-terminates after processing `StateChanged{Terminated}` or `Goodbye` (Change L-2).

**Change L-4 — host_conn comment at attach_session result storage:**

```rust
reader: None, // Reader is owned by proxy_task (spawn_pty_proxy_task).
              // kill_session on an attached Running session detects reader=None + proxy_task=Some
              // and delegates StateChanged{Terminated} handling to the proxy_task (Ruling L /
              // F-S035-001 fix). Watchdog is the 12s fallback.
```

#### BC-2.08.003 integration note

BC-2.08.003 PC-1 (<500ms kill confirmation) is preserved for attached sessions via this ruling:
the proxy_task reads `StateChanged{Terminated}` from the session-host on the same connection
where `DaemonToHost::Kill` was sent, and calls `force_terminate_session` immediately. The
watchdog fires at 12s only if the session-host fails to respond.

#### Tests required (S-035 in-scope)

Add to the S-035 test suite:
- `test_kill_attached_session_fast_path`: attach a session, kill it, assert
  `SessionStateChanged{Terminated}` is received within 500ms (proxy_task fast path, no SIGKILL).
- `test_proxy_task_handles_goodbye_without_terminated`: attach a session, send only `Goodbye`
  from the mock session-host (no prior `StateChanged{Terminated}`), assert session transitions
  to Terminated (defensive path).

---

## §Trace v2.15.1

**MED-002 RESOLVED for S-036 scope — three-layer safety argument extends to `rediscover_sessions()`; no `SessionEntry.generation` guard required** (2026-06-19):

- **Finding origin (MED-002, §Trace v2.7.3):** Adversarial review raised the question of whether
  the post-spawn monitor↔entry race requires a generation/epoch guard on the EC-163 SO_PEERCRED
  mismatch path. Deferred to S-036 as the story that introduces a new `SessionEntry` lifecycle
  insertion path (`rediscover_sessions()`).
- **S-036 adjudication:** All four questions in the deferral anchor were evaluated against the
  S-036 specification. The race the guard was designed to prevent cannot arise on the re-discovery
  path for the following reasons: (1) re-discovery does not generate new UUIDs — it restores the
  pre-existing session_id from the sidecar, so no UUID collision scenario exists; (2) re-discovery
  does not spawn a post-spawn monitor — the actor the guard would protect against is never
  instantiated; (3) re-discovery runs at daemon_start_sequence step 8b, before UDS bind (hard
  invariant BC-2.08.004 Invariant 1), so no spawn_session() call can occur concurrently and no
  in-flight monitors can be active for the current process lifetime at that moment; (4) all three
  safety layers remain intact — layer 1 (UUID collision 2^(-122)) is inapplicable (no new UUID
  drawn), layer 2 (10s GC interlock) is inapplicable (no in-flight monitors, no pending GC timer
  on newly inserted entries), layer 3 (outer Mutex serialization) is fully maintained.
- **Decision:** MED-002 CLOSED. No `SessionEntry.generation: u64` field. No monitor
  capture/compare guard. No new S-036 ACs or tasks. No BC obligation changes to BC-2.08.002 or
  BC-2.08.004.
- **Spec changes:** Ruling H updated in §Post-spawn monitor Ruling H to record resolution; this
  §Trace v2.15.1 block added.
- **SE-16d monotonicity:** v2.15.1 timestamp 2026-06-19 >= v2.15.0 timestamp 2026-06-19. PASS.

---

## §Trace v2.15.0

**F-S038-PASS1-001/002/005 — Single-writer mandate: lifecycle step 9 calls `write_hooks_settings_json`; `SessionManager::new()` receives real `HookEndpointConfig`; EC-182 re-write uses real config** (2026-06-19):

- **Root problem (dual-writer divergence):** Two divergent writers of `hooks-settings.json`
  existed before this change: (1) `lifecycle::write_hooks_settings` using `HooksSettings`/`HooksMap`
  types with NO `lock` field → production file lacked `lock.app = 'monocle'` (Invariant 2 violated,
  EC-181 latent defect); (2) `session_manager::write_hooks_settings_json` emitting correct schema
  but using `HookEndpointConfig::default()` (empty URLs) in `SessionManager::new()`, making
  EC-182 re-writes silently non-functional.
- **Decision:** `session_manager::write_hooks_settings_json` is the sole canonical writer.
  `lifecycle::write_hooks_settings` and the `HooksSettings`/`HooksMap`/`HookEntry` types in
  `monocle-runtime/src/types.rs` are removed.
- **Lifecycle step 9 change:** `daemon_start_sequence` constructs a `HookEndpointConfig` from real
  `port` and `auth_token`, calls `session_manager::write_hooks_settings_json`, propagates failure
  as `DaemonStartError::HooksSettingsWriteFailure` (exit code 72). Abort-on-failure obligation
  moves cleanly to lifecycle's single write call.
- **`SessionManager::new()` signature change:** Adds `hook_endpoint_config: HookEndpointConfig`
  parameter. Removes the `if !path.exists()` write guard (lifecycle owns step-9 write; `new()`
  stores config for EC-182 re-writes only). Production callers pass real config; test callers
  may pass `HookEndpointConfig::default()` when file content is not verified.
- **EC-182 re-write path:** `spawn_session()` re-writes with `self.hook_endpoint_config` — now
  holds real config → re-written file has real curl URLs and `lock.app = 'monocle'`.
- **Downstream:** BC-2.08.006 v1.5.0; SS-daemon-wiring-v2-delta v1.12.0.
- **SE-16d monotonicity:** v2.15.0 timestamp 2026-06-19 >= v2.14.0 timestamp 2026-06-19. PASS.

---

## §Trace v2.14.0

**F-S035-PASS5-MED-001 — EC-188 timeout → Terminated (transition_to_terminated_standalone); action×state matrix Detached cell extended with subpath (d) and EC-187 subpath (a)** (2026-06-19):

- **Finding (F-S035-PASS5-MED-001):** EC-188 attach timeout (5s, SIGTERM sent to session-host
  PID) left the entry in `Detached` state with no broadcast and no GC — a re-attachable entry
  for a host being actively killed. The Detached cell in the action×state matrix only enumerated
  uid-mismatch (subpath a) and protocol-error (subpath b), silently omitting the EC-188 timeout
  path (subpath d) and the ConnectFailed path (subpath a/EC-187).

- **Decision: A — EC-188 timeout → `transition_to_terminated_standalone`.**
  SIGTERM declares host non-responsive/dead. Leaving `Detached` for a SIGTERM'd host is
  semantically incorrect and inconsistent with EC-187 and PeerCredFailed which both transition
  to Terminated on lesser certainty of host-death.

- **Change:** Detached cell for `attach_session()` in the Terminated-in-grace action×state
  matrix expanded to four labeled subpaths:
  (a) EC-187 ConnectFailed → Terminated; (b) SO_PEERCRED uid mismatch → Terminated;
  (c) protocol error after uid match → stays Detached; (d) EC-188 5s timeout → Terminated.
  Canonical enumeration moved to BC-2.08.007 Invariant 5 (v1.5.5); matrix cell now
  cross-references it.

- **Downstream:** BC-2.08.007 v1.5.5 (EC-188 text + Invariant 5).

- **SE-16d monotonicity:** v2.14.0 timestamp 2026-06-19 >= v2.13.0 timestamp 2026-06-19. PASS.

---

## §Trace v2.13.0

**F-S035-PASS2-LOW-001 — attach_session uid-mismatch → Terminated (broadcasting+GC); protocol-error → stays Detached; action×state matrix Detached cell updated** (2026-06-19):

- **Change:** Terminated-in-grace action×state matrix `attach_session()` Detached cell
  updated to document the two failure subpaths explicitly:
  (a) SO_PEERCRED uid mismatch → `Detached→Terminated` via `transition_to_terminated_standalone`
  (full broadcasts + GC) + `Err(SessionHostDead)`.
  (b) Protocol error after uid match → stay `Detached` (no transition, no broadcast) +
  `Err(SessionHostDead)` — host alive and verified-as-ours; retry is legitimate.
- **Rationale:** uid mismatch is structural (not our child process owns the socket), identical
  to the kill-session EC-163 uid-mismatch path which already calls `transition_to_terminated`.
  Protocol errors post-uid-check are transient exchange failures on a verified-live host.
- **Downstream:** BC-2.08.007 v1.5.4 updated (Invariant 5 + Postcondition 2).
- **SE-16d monotonicity:** v2.13.0 timestamp 2026-06-19 >= v2.12.0 timestamp 2026-06-19. PASS.

---

## §Trace v2.12.0

**Ruling L (F-S035-001) — proxy_task is the kill-confirm reader for attached sessions; force_terminate_session delegation** (2026-06-19):

- **Ruling L** added: after `attach_session()`, `host_conn.reader` is `None` (reader owned by
  proxy_task). `kill_session` ExistingConn reader=None path must distinguish proxy-owned
  (reader=None + proxy_task=Some) from pre-Running race (reader=None + proxy_task=None). For
  proxy-owned: proxy_task handles `StateChanged{Terminated}` via Change L-2, calling
  `force_terminate_session` (existing GC path). Do NOT abort the proxy_task — aborting races
  with in-flight Terminated message. Watchdog is the 12s fallback. `spawn_pty_proxy_task`
  gains `sessions` + `sidecar_path` parameters. `Goodbye` arm calls `force_terminate_session`
  defensively. `other =>` arm logs WARN (not trace) for unexpected variants. Two new tests:
  `test_kill_attached_session_fast_path` and `test_proxy_task_handles_goodbye_without_terminated`.
  Fix is S-035 in-scope: proxy_task, `spawn_pty_proxy_task`, and `kill_session` ExistingConn path
  are all S-035 code; no dependency on S-039/S-047 (PtyBytes fan-out, KeyInput, harness child exit).

- SE-16d monotonicity: v2.12.0 timestamp 2026-06-19 > v2.11.0 timestamp 2026-06-17. PASS.

---

## §Trace v2.11.0

**Ruling K (F-S034-ADV-IMPORTANT-001) — §Main event loop child_exit_watch arms annotated with owning story; S-034 doc-comment scope clarified** (2026-06-17):

- **Finding (F-S034-ADV-IMPORTANT-001, adversarial pass 11):** The §Main event loop
  pseudocode shows `child_exit_watch.recv()` arms in both Phase A and Phase B. The
  adversary correctly observed that the S-034 code in
  `crates/monocle-session-host/src/main.rs` (function `step_event_loop`) has NO
  `child_exit_watch`: Phase A is a single `listener.accept()` await; Phase B is a single
  `stream.read_exact()`. The doc comment on `step_event_loop` lines 411-421 was authored
  during S-033 and claims "Deferred handlers (Attach/Resize/KeyInput/Detach) return
  gracefully for S-033; they will be implemented in S-034/S-035/S-047" — implying S-034
  brings the full event loop including child exit detection. That implication is FALSE.

- **Ownership ruling:**
  - The session-host natural-child-exit WATCH (detect harness child exit → send
    `HostToDaemon::StateChanged{Terminated}` + `Goodbye` without a kill) is S-039/S-040
    scope (PTY output pipeline — the full two-phase session-host event loop with
    `pty_reader` is built when PTY streaming is wired in, per §Trace v2.7.0 Ruling A scope
    table). The `child_exit_watch` arm requires a running `pty_reader` task to detect PTY
    master EOF (the natural signal that the harness child exited). Neither resource exists
    in S-034 scope.
  - S-034 session-host scope is precisely: receive `DaemonToHost::Kill` on the control
    connection → SIGTERM to harness child → 10s wait → SIGKILL escalation → send
    `HostToDaemon::StateChanged{Terminated}` → send `HostToDaemon::Goodbye` → remove socket
    → exit. This is the KILL-PATH only; it does not detect or handle natural exit.
  - The daemon-side `Terminating → Terminated` transition on receiving
    `HostToDaemon::StateChanged{Terminated}` (AC-004 / BC-2.08.008 kill-path transitions)
    IS S-034 scope and is correctly implemented.
  - BC-2.08.008 PC-6 ("Ctrl-D in EmbeddedTerminal — natural session exit →
    `SessionStateChanged{Terminated}`") is covered by the session-host child_exit_watch arm
    in S-039/S-040 (session-host side) flowing into BC-2.08.008's daemon broadcast path
    (which S-034 wires for the kill-confirmation side via AC-004). It is NOT S-034 scope.

- **Spec change:** `child_exit_watch` arms in both Phase A and Phase B pseudocode annotated
  with `// [child_exit_watch arm — implemented in S-039/S-040 (PTY output pipeline);
  S-034 does NOT implement this arm. ... Absence of this arm in S-034 code is NOT a gap.]`.
  Phase B arm also annotates the BC-2.08.008 PC-6 coverage anchor. These comments prevent
  future adversarial passes from reading the pseudocode as an S-034 omission.

- **Doc-comment correction required (S-034 implementer action):** The `step_event_loop`
  doc comment in `crates/monocle-session-host/src/main.rs` must be corrected before S-034
  PR merge. See Ruling K doc-comment correction specification below.

- **No BC edit:** BC-2.08.008 §Story Anchor line 220 correctly distributes: "S-034 covers
  Terminating/Terminated transitions" — this refers to the DAEMON-side broadcast of those
  state changes (AC-004, AC-012), which IS S-034 scope. The natural-exit path (PC-6 Ctrl-D
  canonical test vector) is a separate behavior that generates `Terminated` via the
  session-host's child-exit watch, not via kill_session(). The BC as written is not wrong;
  however a product-owner clarification to the §Story Anchor is recommended to make the
  split explicit: "S-034 covers KILL-PATH Terminating/Terminated transitions (daemon-side);
  S-040 covers NATURAL-EXIT Terminated transition (session-host child-exit detection)."
  This is advisory — the product-owner should apply it to prevent future confusion.

- **Ruling K doc-comment correction specification (S-034 implementer):**
  The `step_event_loop` doc comment MUST be replaced to remove the false claim that S-034
  brings the full two-phase event loop with child_exit_watch. The corrected comment is:

  ```rust
  /// Step 9: Main event loop — S-034 scope: DaemonToHost::Kill handler.
  ///
  /// Accepts the daemon's control connection (single accept — post-spawn monitor path from
  /// S-033). After the S-033 handshake (`StateChanged{Running}` sent), this function:
  ///
  /// **S-034 additions (this story):**
  /// - Reads one DaemonToHost message from the accepted stream.
  /// - Dispatches `DaemonToHost::Kill`: SIGTERM to harness child → 10s wait → SIGKILL
  ///   escalation → sends `HostToDaemon::StateChanged{Terminated}` → sends
  ///   `HostToDaemon::Goodbye` → removes socket file → exits.
  /// - All other message variants (Attach, Resize, KeyInput, Detach): stub / graceful
  ///   return for this story; implemented in S-035 / S-047.
  ///
  /// **NOT in S-034 scope — future stories:**
  /// - `child_exit_watch` (natural child exit → StateChanged{Terminated} without Kill):
  ///   implemented in S-039/S-040 (PTY output pipeline; PTY master EOF signals child exit).
  /// - Full `tokio::select!` two-phase (A: Detached/accept-loop; B: Running/active-conn):
  ///   also S-039/S-040+ scope (requires a running PTY reader to select over).
  /// - PTY bytes forwarding (`HostToDaemon::PtyBytes` fan-out): S-039/S-046.
  ```

- SE-16d monotonicity: v2.11.0 timestamp 2026-06-17 >= v2.10.0 timestamp 2026-06-17. PASS.

---

## §Trace v2.10.0

**Ruling J (F-S034-ADV-MED-001) — watchdog force-kill must target BOTH session-host AND harness child; Ruling I item 1 reword (LOW-001)** (2026-06-17):

- **Ruling J** added: portable_pty 0.9.0 `spawn_command()` calls `libc::setsid()` in its
  `pre_exec` closure (unix.rs line 257), placing the harness child in its OWN session and
  process group, completely separate from the session-host's process group. Neither a positive-
  PID kill to the session-host nor a process-group kill (killpg) to the session-host's group
  reaches the harness child. The 12-second watchdog MUST kill both PIDs: `session_host_pid`
  (as before) and `child_pid` (read from sidecar). ESRCH guard on both. If `child_pid` is
  absent from the sidecar (pre-step-8 crash), log WARN only — the harness child was never
  spawned. Implementer MAY alternatively cache `child_pid` in `SessionEntry` from the Running-
  transition sidecar read; both approaches are compliant. This is S-034 in-scope: the watchdog
  is S-034 code. BC-2.08.003 PC-5b amendment specified; product-owner applies the BC update.

- **Ruling I item 1** rewording (LOW-001): The prior text said "stores `host_conn.reader:
  Some(reader)` when it first establishes the connection." That was inaccurate: at connect time
  the monitor's own read loop is still consuming the reader; storing it in `host_conn.reader`
  at that point would create a double-consumer. The correct behavior: `host_conn.reader` is
  set to `Some(reader)` immediately upon the `Running` transition (the monitor's last act
  before returning, after breaking from its read loop). Before `Running`, `host_conn.reader`
  is `None`; a kill arriving before `Running` takes the watchdog-only fallback. No
  implementation change — this is a spec text correction to match the correct design.

- SE-16d monotonicity: v2.10.0 timestamp 2026-06-17 >= v2.9.0 timestamp 2026-06-17. PASS.

---

## §Trace v2.7.2

**Rulings E + F + G: DaemonToHost/HostToDaemon crate residence, EC-152 retry locus, Running-transition broadcast atomicity** (2026-06-17):

- **Ruling E** added: `DaemonToHost` and `HostToDaemon` enums MUST be defined in `monocle-ipc`
  in S-033 scope. Rationale: Ruling D established that S-033's post-spawn monitor stores
  `host_conn` writer — the writer needs a typed contract. Both enums are wire types (process
  boundary crossing between daemon and session-host); `monocle-ipc` is the established
  shared wire-type crate. Full variant completeness required in S-033; handler stubs acceptable
  for deferred variants (Attach → S-035, KeyInput → S-047, Resize → S-042). Resolves MED-001.

- **Ruling F** added: UUID retry locus is EXCLUSIVELY the IPC handler (one retry: initial
  UUID + one regenerated UUID = 2 total attempts). `spawn_session()` MUST NOT retry internally.
  Double-retry (IPC handler + spawn_session each retry once = 4 total attempts) violates
  BC-2.08.001 EC-152 "auto-retry is one attempt" constraint. Implementer instruction: on
  `Err(SessionIdCollision)`, the IPC handler regenerates UUID, sends a second `SpawnAck` with
  the new UUID (so wizard tracks correct session_id), then retries `spawn_session()` once.
  Resolves MED-002.

- **Ruling G** added: post-spawn monitor MUST hold the `SessionManager` mutex across BOTH
  `try_send()` calls for the Running-transition broadcast pair. Per-client FIFO alone does not
  prevent a concurrent spawn's Launching pair from interleaving between the two Running
  broadcasts. BC-2.08.008 Invariant 4 "mutex provides the atomicity window" applies to ALL
  broadcast-pair emissions including those from detached background tasks. Same applies to
  the EC-163 and `StateChanged{Terminated}` termination pairs. The degraded-env metadata
  update (step 4, no state transition) may hold the mutex independently. Resolves MED-003.

- SE-16d monotonicity: v2.7.2 > v2.7.1. PASS.

---

## §Trace v2.7.1

**Rulings C + D: setsid placement and host_conn scope clarification** (2026-06-16):

- **Ruling C** added: setsid MUST be called by the session-host binary at startup step 2
  (already in §startup sequence). `pre_exec` is prohibited: `std::process::Command::pre_exec()`
  is `unsafe fn`; `monocle-runtime` carries `#![forbid(unsafe_code)]`. Pre-setsid race window
  accepted: daemon has no controlling terminal (daemon called setsid at its own startup);
  no SIGHUP source exists for the window. `RealSessionHostSpawner` docstring and Ruling A
  setsid row updated to remove the erroneous pre_exec reference. Resolves MED-003.

- **Ruling D** added: `host_conn = Some(SessionHostConnection { writer, proxy_task: None })`
  is stored in S-033's post-spawn monitor (§Post-spawn monitor step 3), not S-034.
  S-034 (`kill_session`) USES the writer; it does not establish it. Implementer defect
  ("stored in S-034") identified and overturned. No spec content change needed beyond this
  ruling — existing spec text (§Post-spawn monitor, F-P50-001, state transition table) already
  correctly specifies this. Resolves HIGH-001.

- SE-16d monotonicity: v2.7.1 > v2.7.0. PASS.

---

## §Trace v2.16.0

**Ruling A errata — Resize row split: S-047 owns daemon leg; S-042 owns session-host leg** (2026-06-21):

- **Ruling A row correction:** The "PTY resize | S-042 | resize_session and ResizePane handler" row
  was erroneous. S-047 AC-003 and its IPC Handler Arm Ownership table (S-047 v1.1+) explicitly own
  `ClientToServer::ResizePane` dispatch arm in `ipc_handler.rs` and `session_manager.resize_session()`.
  S-042's `target_module: monocle-tui` and its File Structure Requirements do NOT list
  `monocle-runtime/src/ipc_server.rs` or `session_manager/mod.rs`. The STORY-INDEX BC-2.05.010
  row confirms S-047 owns all 7 `ClientToServer` IPC variants including ResizePane.
  Row split into: (1) daemon leg → S-047; (2) session-host `DaemonToHost::Resize` handler → S-042.
- **Production-grade rationale:** The S-042 implementer must NOT implement `resize_session()` in
  `session_manager/mod.rs` — that method belongs to S-047. The S-042 implementer is responsible
  only for the session-host's `DaemonToHost::Resize { rows, cols }` match arm: calling
  `pty.resize()` and `parser.set_size()`.
- **SPEC-CROSS-REF:** BC-2.09.006 Story Anchor and postconditions 4-7 updated in BC v1.2.0 to
  explicitly split ownership S-042 (PC-1/2/3/8, TUI-side) vs S-047 (PC-4/5/6/7, daemon-side).
- **SE-16d monotonicity: v2.16.0 > v2.15.1. PASS.**

---

## §Trace v2.7.0

**Rulings A + B: S-033 session-host scope boundary and SessionSidecarV3 crate residence** (2026-06-16):

- **Ruling A** added: §S-033 session-host scope boundary. Precise step-by-step table of what
  must be implemented in S-033 (parse args, setsid, open PTY, build CommandBuilder, spawn
  harness child, init vt100::Parser stub, bind UDS, write sidecar with child_pid, send
  StateChanged{Running}, enter minimal event loop) versus what is deferred (PTY output streaming
  → S-039, scrollback dump → S-035, keyboard forwarding → S-047, resize → S-042, TUI side →
  S-044). MockSessionHostSpawner vs RealSessionHostSpawner disposition clarified: both required;
  AC-010 requires non-trivial session-host (no all-todo!() stubs).

- **Ruling B** added: `SessionSidecarV3` struct lives in `monocle-ipc`, imported by both
  `monocle-runtime` and `monocle-session-host`. Ownership protocol table (who writes which
  fields when). No write race (sequential: daemon writes first, session-host overwrites with
  child_pid). Both writers MUST use `tempfile::persist`. Path canonicalization requirement
  (session-host uses `--runtime-dir` arg verbatim; no re-canonicalize). Schema agreement
  guarantee: compile-time breakage if struct changes. S-036 parseability confirmed.

- Motivation: adversarial review of S-033 surfaced two cross-component questions (HIGH-009 +
  scope boundary ambiguity). These rulings close both gaps before implementation proceeds.

- SE-16d monotonicity: v2.7.0 > v2.6.1. PASS.

## §Trace v2.6.1

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
