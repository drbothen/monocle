---
document_type: architecture-section
level: L3
section: "session-manager"
subsystem: SS-08
version: "1.0.0"
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
input-hash: "7e4f4f4"
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

### SessionManager struct

```rust
/// Daemon-side coordinator for session-host processes.
/// Owned by DaemonState; one instance per daemon process.
pub struct SessionManager {
    /// Active sessions keyed by session ID.
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
    project_root: PathBuf,
    harness_id: String,                 // "claude-code", "codemachine", etc.
    profile_id: String,
    started_at: chrono::DateTime<chrono::Utc>,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// spawn_session() called; monocle-session-host not yet started.
    Created,
    /// session-host process spawned; waiting for its socket to appear.
    Launching,
    /// session-host alive and PTY streaming.
    Running,
    /// TUI explicitly detached; session-host still alive; daemon not currently attached.
    Detached,
    /// Child process exited cleanly; session-host entered terminated state.
    Terminated,
    /// User-initiated kill; SIGTERM sent to session-host.
    Killed,
}
```

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

    /// Rename a session (updates sidecar; publishes SessionStateChanged to broker).
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
4. Build `CommandBuilder` from `--binary/--args/--env/--cwd`.
5. Spawn harness child on PTY slave.
6. Initialize `vt100::Parser` with initial size.
7. Bind per-session UDS socket at `<runtime_dir>/session-<session_id>.sock` (mode `0o600`).
8. Write `session-state.json` sidecar at `<runtime_dir>/session-<session_id>.json`.
9. Enter main event loop.

### session-state.json schema

```json
{
  "schema_version": 1,
  "session_id": "<uuid>",
  "pid": 12345,
  "socket_path": "<runtime_dir>/session-<uuid>.sock",
  "child_pid": 12346,
  "state": "Running",
  "project_root": "/path/to/project",
  "harness_id": "claude-code",
  "profile_id": "default",
  "started_at": "2026-06-03T23:00:00Z",
  "display_name": "monocle — phase0",
  "pty_rows": 24,
  "pty_cols": 80
}
```

`schema_version` MUST be checked on read; unknown versions must be ignored (forward-compat).

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
    /// Initial response to Attach: scrollback state.
    /// vt100::Parser state serialized as all screen rows.
    ScrollbackDump { rows: Vec<String>, cursor_row: u16, cursor_col: u16 },
    /// Live PTY output bytes.
    PtyBytes { bytes: Vec<u8> },
    /// Session state changed (child exited, etc.).
    StateChanged { new_state: SessionState },
    /// Session-host is shutting down.
    Goodbye,
}
```

### Main event loop

```rust
tokio::select! {
    Some(bytes) = pty_reader.recv() => {
        parser.process(&bytes);
        for client in attached_clients {
            send_pty_bytes_to_client(client, &bytes).await;
        }
    }
    Some(msg) = daemon_conn.recv() => match msg {
        DaemonToHost::Attach => { send_scrollback_dump().await; }
        DaemonToHost::KeyInput { bytes } => { pty_writer.write_all(&bytes).await?; }
        DaemonToHost::Resize { rows, cols } => {
            pty.resize(PtySize { rows, cols, .. })?;
            parser.set_size(rows, cols);
        }
        DaemonToHost::Kill => { child.kill().await?; }
        DaemonToHost::Detach => { /* disconnect client; stay alive */ }
    }
    Some(exit) = child_exit_watch.recv() => {
        write_state_changed(SessionState::Terminated).await;
        break;
    }
}
```

### PTY reader thread

The PTY master `read()` is blocking; it runs in a `tokio::task::spawn_blocking` thread,
posting bytes to a bounded `mpsc::channel::<Bytes>(1024)` with a drop counter surfaced
in the session-host's stderr log (not in the TUI, since the session-host has no TUI).

---

## Daemon startup: session re-discovery

On daemon startup, BEFORE accepting UDS client connections, `SessionManager::rediscover_sessions()`
runs:

1. Read all `session-*.json` files in `<runtime_dir>`.
2. For each sidecar:
   a. Parse `session-state.json`; check `schema_version`.
   b. Probe liveness: `nix::sys::signal::kill(Pid::from_raw(pid), None)`.
   c. If alive: attempt UDS connect to socket_path; send `DaemonToHost::Attach`;
      wait up to 5s for `HostToDaemon::ScrollbackDump`; register in SessionManager.
   d. If dead: mark entry Terminated; delete sidecar (GC).
3. Publish re-discovered sessions in `DaemonState.sessions` before serving TUI clients.

Re-discovery MUST complete before the daemon writes its lock file or accepts IPC connections
(step within the daemon start sequence — inserted between existing step 8 and step 9).

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
    /// Environment variables to inject (hook config, CCR env vars, etc.).
    pub env: HashMap<String, String>,
    /// Working directory for the session (git worktree path per claude-squad A.1).
    pub cwd: PathBuf,
}

pub struct SpawnOptions {
    pub project_root: PathBuf,
    pub profile_id: String,
    pub session_id: String,           // pre-generated UUID; used in hooks_settings_path
    pub hooks_settings_path: PathBuf, // pre-written by daemon; passed as --settings arg
    pub ccr_base_url: Option<String>, // injected as ANTHROPIC_BASE_URL if CCR detected
}
```

`ClaudeCodeModule::spawn_recipe()` fills the recipe from:
- Binary: `which::which("claude")` result
- Args: `["--settings", opts.hooks_settings_path.to_str().unwrap()]`
- Env: `ANTHROPIC_BASE_URL` if `opts.ccr_base_url.is_some()`
- Cwd: `opts.project_root`

**Hook auto-injection:** The daemon writes `hooks-settings.json` to a per-session temp path
(existing `OQ-02` mechanism) before calling `spawn_recipe()`. The `--settings` arg carries
this path. No user action required. `lock.app = 'monocle'` filter in the hook JS ensures
only monocle-launched sessions trigger the monocle endpoint.

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

Mitigation: `daemon_start_sequence` completes session re-discovery BEFORE writing the lock
file and binding the UDS socket. TUI cannot connect until the lock file is present. No TUI
client can see a stale session list.

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

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- SS-08 authored as part of v1A architecture delta.
- Implements ADR-0009 (native session-host) and ADR-0010 (shared UDS).
- Q-2 resolved: lifecycle on SessionManager; EngineModule provides recipe only.
- SessionManager location confirmed as sub-module of monocle-runtime (self-adjudication
  confirmed per vision §Workspace Layout note; now formalized here).
- monocle-session-host new binary crate specified.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
