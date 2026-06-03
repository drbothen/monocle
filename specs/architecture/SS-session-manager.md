---
document_type: architecture-section
level: L3
section: "session-manager"
subsystem: SS-08
version: "1.2.0"
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
///
/// REACHABLE STATES ONLY (I4 audit — pruned unreachable variants):
///   Created — REMOVED: spawn_session() transitions directly to Launching (the OS process
///     is spawned synchronously inside spawn_session()); Created was never persisted or observed.
///   Killed — REMOVED: kill_session() sends SIGTERM to the session-host, which causes the
///     harness child to exit, which triggers the child_exit_watch arm, which sends
///     StateChanged::Terminated. The TUI/daemon never observe a "Killed" state in the
///     registry; the state goes Running → (SIGTERM) → Terminated. Killed was never reachable.
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
    /// Harness child exited (naturally or via SIGTERM from kill_session()); session-host
    /// sent StateChanged::Terminated to daemon. Terminal state — no further transitions.
    Terminated,
}
```

#### State transition table

| From | Event | To | Notes |
|------|-------|----|-------|
| *(none)* | `spawn_session()` called | `Launching` | OS process spawned; sidecar written |
| `Launching` | Daemon receives `StateChanged::Running` from session-host | `Running` | Session-host UDS connectable; proxy task started |
| `Launching` | `ScrollbackDump` received on re-discovery | `Running` | Re-discovery path: host was already up |
| `Launching` | PID dead on re-discovery probe | `Terminated` | GC sidecar |
| `Launching` | `StateChanged::Terminated` (spawn failed inside session-host) | `Terminated` | GC sidecar |
| `Running` | `detach_session()` called | `Detached` | Proxy task aborted; session-host continues |
| `Running` | `StateChanged::Terminated` from session-host | `Terminated` | Child exited naturally or via SIGTERM |
| `Running` | PID dead on re-discovery | `Terminated` | GC sidecar (should not occur normally) |
| `Detached` | `attach_session()` called | `Running` | New proxy task created; ScrollbackDump received |
| `Detached` | PID dead on re-discovery | `Terminated` | GC sidecar |
| `Terminated` | GC timer (10s) | *(removed)* | Entry removed from registry |

#### Re-discovery state handling (I4 — all states covered)

`rediscover_sessions()` probes all sidecar files and handles every persisted state:

- **Sidecar state `Launching`:** The session-host was spawned but the daemon crashed before
  confirming Running. Probe liveness: if alive, attempt `Attach` with 5s timeout. If
  `ScrollbackDump` received → register as `Running`. If no response within 5s → `SIGTERM`
  session-host; mark `Terminated`; GC sidecar. If process dead → GC sidecar.
- **Sidecar state `Running`:** Normal re-discovery path. Probe liveness; if alive, `Attach`;
  on `ScrollbackDump` → `Running`. If dead → GC.
- **Sidecar state `Detached`:** Session-host was alive but daemon was not attached. Probe
  liveness; if alive, attempt `Attach` (the session-host accepts Attach from Detached state);
  on `ScrollbackDump` → `Running`. If dead → GC. Note: re-discovery always tries to attach
  (even Detached sidecars) because the daemon needs the proxy task to maintain the broker
  fan-out.
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
// nix::sys::signal::kill(Pid::from_raw(pid), Signal::SIGKILL)?;
```

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

### Per-session UDS security (I5)

The per-session UDS at `<runtime_dir>/session-<uuid>.sock` uses `0o600` permissions
on the socket file and `0o700` on the `runtime_dir`. These filesystem permissions are
necessary but not sufficient: the daemon can be tricked via a rogue sidecar file that
names an attacker-controlled socket path, enabling keystroke injection into a user's
Claude Code session via `DaemonToHost::KeyInput`.

**Required security controls:**

1. **SO_PEERCRED peer-credential check (mandatory):** On every per-session UDS connection
   (both when the session-host accepts a daemon connection AND when the daemon connects to
   a session-host socket), the connecting peer's `uid` MUST be checked via `SO_PEERCRED`
   (Linux) or `LOCAL_PEERPID` + `getpwuid` (macOS). The connection MUST be rejected if the
   peer `uid` differs from the current process `uid`. Implementation: call
   `nix::sys::socket::getsockopt::<nix::sys::socket::sockopt::PeerCredentials>(fd)` (Linux)
   or `nix::sys::socket::getsockopt::<nix::sys::socket::sockopt::LocalPeerPid>(fd)` (macOS)
   immediately after `accept()`/`connect()`, before reading any bytes.

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
    /// Initial response to Attach: current vt100 screen state as styled-cell serialization.
    /// See §Screen-state transfer (C5) for the correct serialization contract.
    ScrollbackDump {
        /// Serialized vt100::Screen cells: row-major, each cell is (char, fg, bg, attrs).
        /// Format: Vec<Vec<SerializedCell>> where outer vec is rows (top to bottom),
        /// inner vec is columns. MUST include ALL rows (scrollback + visible screen),
        /// ordered from oldest scrollback row to current screen bottom.
        rows: Vec<Vec<SerializedCell>>,
        cursor_row: u16,
        cursor_col: u16,
        /// Current PTY dimensions (rows × cols) at time of dump.
        pty_rows: u16,
        pty_cols: u16,
    },
    /// Live PTY output bytes.
    PtyBytes { bytes: Vec<u8> },
    /// Session state changed (child exited, etc.).
    StateChanged { new_state: SessionState },
    /// Session-host is shutting down.
    Goodbye,
    /// PTY byte drop detected (channel sender returned Err). See §PTY reader thread.
    /// Daemon propagates as ServerToClient::PtyReset to TUI clients.
    PtyReset,
}

/// A single terminal cell as serialized for ScrollbackDump.
/// Sufficient to reconstruct the full styled vt100::Screen without re-parsing PTY bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedCell {
    /// The UTF-8 character at this cell (empty string for empty/null cells).
    pub ch: String,
    /// Foreground color (ANSI 16, 256-color index, or RGB triple).
    pub fg: SerializedColor,
    /// Background color.
    pub bg: SerializedColor,
    /// Cell attributes: bold/italic/underline/blink/reverse/dim flags as a bitmask.
    pub attrs: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedColor {
    Default,
    Ansi(u8),
    Rgb(u8, u8, u8),
}
```

### Screen-state transfer on Attach (C5 — correct ScrollbackDump semantics)

When the session-host handles `DaemonToHost::Attach`, it MUST transfer the vt100 screen
state correctly so the TUI can reconstruct the current terminal without double-applying
PTY bytes.

**Correct protocol:**

1. **Snapshot the parser state** (not raw bytes, not string rows): Serialize the current
   `vt100::Screen` as `Vec<Vec<SerializedCell>>` — styled cells with character, fg color,
   bg color, and attribute flags. This preserves full visual fidelity (colors, bold, etc.)
   that is lost by `String`-only row serialization.

2. **Send `HostToDaemon::ScrollbackDump`** with the serialized cells, cursor position,
   and current PTY dimensions.

3. **TUI receiver protocol:** On receipt of `ScrollbackDump`, the TUI MUST:
   a. Reset the parser: `pty_parsers[session_id] = vt100::Parser::new(pty_rows, pty_cols, SCROLLBACK_ROWS)`.
   b. Reconstruct the screen by replaying the cells as a synthetic `DCS` or `ED` clear +
      cursor-position + write sequence, OR by using `vt100::Parser::set_screen()` if the
      vt100 crate exposes such an API, OR by implementing a direct screen-state injection
      that sets each cell's content/attributes without going through byte parsing.
   c. If the vt100 crate does not expose direct cell injection: send the scrollback dump
      as a sequence of ANSI `\x1b[2J\x1b[H` (clear screen, home cursor) followed by
      cell-by-cell reconstruction using ANSI SGR sequences. This is the `scrollback-as-bytes`
      reconstruction path.
   d. After reconstruction, subsequent `PtyBytes` events are processed by the now-correctly-
      initialized parser. No double-counting occurs because the parser was reset before
      applying the dump.

**Scrollback memory bound (O4):** `SCROLLBACK_ROWS` is configurable (default 1000, max
10000 per SS-embedded-pty.md). Each row in a `SerializedCell` representation costs:
per cell ≈ `sizeof(SerializedCell)` ≈ 6 bytes (1 char + 2 color + 1 attrs, with enum
overhead ≈ 2 bytes per color) + serde JSON overhead ≈ 30–50 bytes per cell in JSON.
A 80-column × 10000-row scrollback buffer in JSON ≈ 80 × 10000 × 40 bytes ≈ 32 MB per
session. For 8 sessions ≈ 256 MB. The ScrollbackDump is transient (sent over UDS, not
stored); the TUI reconstructs and then discards the dump. The UDS 256 KiB message limit
means large scrollbacks MUST be chunked: `ScrollbackDump` may be split into multiple
messages, each ≤ 256 KiB. The session-host streams scrollback rows in batches; a
`ScrollbackDumpComplete` sentinel message terminates the stream.

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
            // Serialize current vt100::Screen as styled cells (see §Screen-state transfer)
            send_scrollback_dump_styled_cells().await;
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
4. Each TUI client, on receiving `PtyReset`, calls `pty_parsers[session_id] = vt100::Parser::new(rows, cols, scrollback_rows)` (fresh parser) and issues a forced full redraw request to the session-host (re-attaches to get a fresh `ScrollbackDump`).

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
   c. If alive: attempt UDS connect to socket_path; immediately verify `SO_PEERCRED` / 
      `LOCAL_PEERPID` matches sidecar pid (I5 cross-check); if mismatch: log WARN, SIGTERM
      both pids, delete sidecar, skip. If match: send `DaemonToHost::Attach`; wait up to 5s
      for `HostToDaemon::ScrollbackDump`; register in SessionManager as `Running`.
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
mandated per-session paths to avoid clobber) are architecture-level errors: they misdiagnosed
the clobber risk. Clobber is a problem only when content differs between writers; here it
never differs. BC-2.08.006 must be updated by product-owner to remove Invariant 3 and EC-182
and replace with the shared-file model (see §Trace C1 flag for product-owner delegation).

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
