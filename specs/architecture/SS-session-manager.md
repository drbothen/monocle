---
document_type: architecture-section
level: L3
section: "session-manager"
subsystem: SS-08
version: "1.4.1"
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
    /// Spawn a new session by running monocle-session-host with the given recipe.
    /// Returns the session_id of the new session.
    pub async fn spawn_session(
        &mut self,
        recipe: SpawnRecipe,
        harness_id: String,
        profile_id: String,
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
     parent env by default (check the `portable-pty` 0.8.x API), the session-host MUST
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
    /// followed by a single ScrollbackDumpComplete sentinel. The session-host pauses
    /// live PtyBytes forwarding during the dump (see ADR-0010 §Interleaving).
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
    #[serde(default)]
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

/// A single terminal cell as serialized for scrollback dump (ScrollbackChunk rows).
/// Sufficient to reconstruct the full styled vt100::Screen without re-parsing PTY bytes.
///
/// # I3-008: vt100 0.16 attribute surface — verified
///
/// The vt100 0.16 `Cell` struct exposes EXACTLY FIVE attribute methods:
///   `bold()`, `dim()`, `italic()`, `underline()`, `inverse()`.
/// The `inverse()` method corresponds to what SGR calls "reverse" (SGR 7).
/// There is NO `blink()`, NO `hidden()`, and NO `strikethrough()` in vt100 0.16.
/// (Verified against docs.rs/vt100/0.16.0/vt100/struct.Cell.html — 2026-06-03.)
///
/// The prior 6-flag u8 bitmask (bold/italic/underline/blink/reverse/dim) incorrectly named
/// "reverse" and included "blink" which vt100 0.16 does not expose. This is corrected:
/// the 5-bit bitmask covers the 5 actual vt100 0.16 attributes. The "full visual fidelity"
/// claim in §O4 is accurate with respect to what vt100 0.16 exposes — no fidelity is lost
/// by aligning to the actual API. Blink, hidden, and strikethrough are not part of vt100
/// 0.16's observable cell attribute set.
///
/// # attrs bitmask layout (5 bits, low-to-high):
///   bit 0: bold     (cell.bold())
///   bit 1: dim      (cell.dim())
///   bit 2: italic   (cell.italic())
///   bit 3: underline (cell.underline())
///   bit 4: inverse  (cell.inverse()) — SGR 7 "reverse video"
///   bits 5–7: reserved (MUST be 0 on write; MUST be ignored on read for forward-compat)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedCell {
    /// The UTF-8 character at this cell (empty string for empty/null cells).
    pub ch: String,
    /// Foreground color (ANSI 16, 256-color index, or RGB triple).
    pub fg: SerializedColor,
    /// Background color.
    pub bg: SerializedColor,
    /// Cell attributes bitmask (5 bits used; see doc comment above for layout).
    /// The u8 type is retained for forward-compat: if a future vt100 version exposes
    /// additional attributes, they can be added to bits 5–7 without a wire format change.
    pub attrs: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedColor {
    Default,
    Ansi(u8),
    Rgb(u8, u8, u8),
}
```

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
      If mismatch → log WARN; send a re-attach request (re-sends `SpawnSession`'s Attach)
      to restart the dump.
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
            // Snapshot vt100::Screen as styled cells, pause PtyBytes, stream chunks,
            // send ScrollbackDumpComplete (see §Screen-state transfer on Attach).
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
4. Each TUI client, on receiving `PtyReset`, calls `pty_parsers[session_id] = vt100::Parser::new(rows, cols, scrollback_rows)` (fresh parser) and re-attaches to the session-host (sends a fresh `Attach` to trigger a new `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence).

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
    pub profile_id: String,
    pub session_id: String,           // pre-generated UUID; used in hooks_settings_path
    pub hooks_settings_path: PathBuf, // pre-written by daemon; passed as --settings arg
    pub ccr_base_url: Option<String>, // injected as ANTHROPIC_BASE_URL if CCR detected
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
here it never differs. BC-2.08.006 v1.2.0 has been RECONCILED in place by product-owner to
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
| `SpawnRecipe` struct | Pure core | Data type only; no I/O. |
| `SpawnOptions` struct | Pure core | Data type only; no I/O. |
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
  `SessionEntry.kill_deadline: Option<Instant>` and `kill_deadline_reason: Option<String>` added.
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
