---
document_type: story
level: L4
story_id: S-017
epic_id: EPIC-04
version: "1.1"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 8
wave: 5
tdd_mode: strict
priority: P0
depends_on: [S-016, S-006, S-008, S-009, S-012, S-015]
blocks: [S-018, S-019, S-020, S-021]
target_module: monocle-runtime
subsystems: [SS-04]
behavioral_contracts: [BC-2.04.001, BC-2.04.010]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.001.md, version: "1.5.0"}
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.010.md, version: "1.2.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.04.001 (13-step daemon start sequence, SOQ-2), BC-2.04.010 (hooks-settings.json generation)"
---

# S-017: Daemon Start Sequence (SOQ-2) + Hook Tmpfile Generation

## Narrative

As a daemon system, I want the 13-step startup sequence to execute in strict order, writing
the lock file before `hooks-settings.json` (SOQ-2 invariant), so that any Claude Code
subprocess reading `hooks-settings.json` always finds an auth token already committed to the
lock file, eliminating the auth-token race condition.

## Acceptance Criteria

### AC-001 (traces to BC-2.04.001 postcondition PC-1/PC-2 — step 1: runtime dir)
`daemon_start_sequence()` resolves `<runtime_dir>` via `resolve_runtime_dir()` (BC-2.04.006)
as step 1. If the directory does not exist, it is created with mode `0o700` via
`DirBuilder::new().mode(0o700).recursive(true).create(&runtime_dir)`. If resolution fails,
exit with `DaemonStartError::RuntimeDirUnresolvable` (exit code 70).

### AC-002 (traces to BC-2.04.001 postcondition PC-3 — step 2: stale lock check)
Step 2 delegates to `DaemonLock::acquire()` (S-006). If a live daemon is detected, the
start sequence exits 1 with `error: daemon already running (pid=<N>)`. If a stale lock
file is found (dead PID), it is removed and the sequence continues.

### AC-003 (traces to BC-2.04.001 postcondition PC-4/PC-5/PC-6 — step 3: port bind)
`TcpListener::bind("127.0.0.1:0")` is called at step 3. The OS-assigned ephemeral port is
recorded in a local variable BEFORE any lock file write (SOQ-2 first anchor). If bind fails,
daemon exits with code 71.

### AC-004 (traces to BC-2.04.001 postcondition PC-7/PC-8/PC-9 — step 4: RingBuffer)
A `RingBuffer` is constructed with capacity 100 MB × 5 rotations and flush mode `async-jsonl`,
targeting `<runtime_dir>/monocle.jsonl`. The buffer is wrapped in `Arc<RingBuffer>` and
stored in `DaemonState.ring`.

### AC-005 (traces to BC-2.04.001 postcondition PC-10 — step 5: event bus)
`tokio::sync::mpsc::channel::<HookEvent>(4096)` is called at step 5. The sender is stored
in `DaemonState.event_bus_tx` (wrapped in `Arc<EventBusTx>`). The drop counter
`AtomicU64::new(0)` is initialized in `DaemonState`.

### AC-006 (traces to BC-2.04.001 postcondition PC-11/PC-12 — step 6: EngineModule registry)
A `ClaudeCodeModule` instance (S-015) is constructed and registered in `DaemonState.engine_registry`.
The `VsddFactoryAdapter` (S-012) is initialized and associated with the `ClaudeCodeModule`.

### AC-007 (traces to BC-2.04.001 postcondition PC-13/PC-14 — step 7: auth token)
32 bytes from `rand::rngs::OsRng` are hex-encoded to 64 lowercase characters. Only the
64-hex suffix is stored in `DaemonState.auth_token`. The `monocle-v1:` prefix is added only
at write time (step 8 and step 9). If `OsRng` is unavailable, exit with code 71.

### AC-008 (traces to BC-2.04.001 postcondition PC-15/PC-16/PC-17 — step 8: lock file, SOQ-2)
The lock file is written via `tempfile::persist` to `<runtime_dir>/monocle.lock` with mode
`0o600`. JSON content uses the schema from BC-2.01.010 with `contract_version: "monocle-lock-v1"`.
This write MUST occur AFTER step 3 (port bound) and AFTER step 7 (token generated). This is
the SOQ-2 commit point. If this write fails, exit with code 71.

### AC-009 (traces to BC-2.04.010 postcondition PC-1 — hooks-settings.json atomic write)
Step 9: `write_hooks_settings()` uses `NamedTempFile::new_in(&runtime_dir)?` →
`serde_json::to_writer_pretty(&mut tmp, &hooks_settings)?` → `tmp.persist(&hooks_settings_path)?`.
Naked `std::fs::write` to `hooks-settings.json` is forbidden.

### AC-010 (traces to BC-2.04.010 postcondition PC-2 — mode 0o600)
After `tempfile::persist`, `std::fs::set_permissions(&path, Permissions::from_mode(0o600))` is
called. Mode `0o600` must be set even on platforms where tempfile's default is already restrictive.

### AC-011 (traces to BC-2.04.010 postcondition PC-3 — schema: 4 hook URLs)
`hooks-settings.json` contains `PreToolUse`, `Notification`, `Stop`, and `UserPromptSubmit`
entries, each with the correct URL embedding the OS-assigned port and `monocle-v1:<64-hex>` token.
`PostToolUse` and `PreCompact` are present with empty arrays. `SessionStart` is NOT listed.
If this write fails, exit with code 72.

### AC-012 (traces to BC-2.04.010 postcondition PC-4 — SOQ-2 ordering)
`write_hooks_settings()` is called at step 9, strictly after `write_lock_file()` at step 8.
The function call order in `daemon_start_sequence()` enforces this. No concurrent code path
can call `write_hooks_settings()` before step 8 completes.

### AC-013 (traces to BC-2.04.001 postcondition PC-20/PC-21/PC-22 — step 10: UDS socket)
A Unix domain socket is bound at `<runtime_dir>/monocle.sock` with mode `0o600`. If a stale
socket file exists, it is removed before binding. The socket path is stored in `DaemonState.sock_file_path`.

### AC-014 (traces to BC-2.04.001 postcondition PC-23 — step 11: crash recovery)
The crash recovery checkpoint background task is initialized per BC-2.01.006 (S-007).

### AC-015 (traces to BC-2.04.001 postcondition PC-24/PC-25/PC-26 — steps 12-13)
`run_server(Arc::new(state), listener)` starts the HTTP server at step 12. The foreground
`daemon start` caller detects completion by polling for the lock file. No explicit IPC from
daemon to caller.

### AC-016 (traces to BC-2.04.001 invariant 6 — lock file cleanup on post-step-8 failure)
If any step N > 8 fails (steps 9-12), the lock file written at step 8 is removed before
process exit to avoid leaving an orphaned lock file.

### AC-017 (traces to BC-2.04.010 postcondition PC-5 — hooks-settings.json removed on shutdown)
On graceful shutdown (BC-2.01.004 drain), `hooks-settings.json` is removed alongside
`monocle.lock` and `monocle.sock`. If removal fails, the error is logged at WARN and
shutdown continues.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,800 |
| BC-2.04.001.md | ~1,500 |
| BC-2.04.010.md | ~1,200 |
| SS-daemon-wiring.md §Daemon Start Sequence | ~5,000 |
| S-006, S-007, S-008, S-009, S-012, S-015 (summaries) | ~1,500 |
| Test file | ~1,000 |
| **Total estimate** | **~12,000** |

## Tasks

- [ ] Implement `daemon_start_sequence()` in `monocle-runtime/src/lifecycle.rs` with all 13 steps in strict order
- [ ] Step 1: call `resolve_runtime_dir()` + `DirBuilder::mode(0o700)` creation
- [ ] Step 2: call `DaemonLock::acquire()` from S-006
- [ ] Step 3: `TcpListener::bind("127.0.0.1:0")` — record port before any file write
- [ ] Step 4: construct `RingBuffer` with `Arc<>` wrapping; store in `DaemonState.ring`
- [ ] Step 5: `mpsc::channel::<HookEvent>(4096)`; initialize `AtomicU64` drop counter
- [ ] Step 6: instantiate `ClaudeCodeModule` (S-015) + `VsddFactoryAdapter` (S-012); register
- [ ] Step 7: `rand::rngs::OsRng` → 32 bytes → 64-hex; store 64-hex in `DaemonState.auth_token`
- [ ] Step 8: `write_lock_file()` via `tempfile::persist`; mode `0o600`; SOQ-2 commit point
- [ ] Implement `DaemonState` struct aggregating all fields (ring, event_bus_tx, drop_counter, auth_token, engine_registry, sock_file_path, session_registry)
- [ ] Step 9: `write_hooks_settings()` — `NamedTempFile::new_in` → `serde_json::to_writer_pretty` → `persist` → `set_permissions(0o600)` — AFTER step 8
- [ ] Validate hooks-settings.json schema: 4 active hook types (PreToolUse, Notification, Stop, UserPromptSubmit) + PostToolUse=[] + PreCompact=[]
- [ ] Step 10: `UnixListener::bind("<runtime_dir>/monocle.sock")`; remove stale socket before bind
- [ ] Step 11: initialize crash recovery checkpoint task (S-007)
- [ ] Step 12: call `run_server(Arc::new(state), listener)` — HTTP server starts
- [ ] Implement lock-file cleanup on post-step-8 failure (invariant 6)
- [ ] Implement hooks-settings.json removal in shutdown sequence (alongside lock + sock removal)
- [ ] Integration test `monocle-runtime/tests/daemon_start_sequence.rs`:
  - Clean start: all 13 steps in order; lock file correct fields; hooks-settings.json correct schema
  - SOQ-2: lock file mtime <= hooks-settings.json mtime (measured after both writes)
  - Lock file mode 0o600; runtime_dir mode 0o700
  - Step 8 failure → no hooks-settings.json; no HTTP server started
  - Step 9 failure → lock file removed before exit

## Previous Story Intelligence

S-001: Cargo workspace initialized. All workspace dependencies are pinned.
S-006: `DaemonLock::acquire()` and `resolve_runtime_dir()` are available from `monocle-runtime`.
S-007: `RecoveryCheckpoint` task is available — call its init function at step 11.
S-008: `RingBuffer` struct with `async-jsonl` flush mode is available.
S-009: `generate_session_token()` produces the 64-hex token. This story wires it into the start
sequence at step 7 (generating from OsRng directly; S-009 auth middleware reads it from the lock file).
S-012: `VsddFactoryAdapter::new()` is available.
S-015: `ClaudeCodeModule::new()` is available. Register it in the engine registry at step 6.

## Architecture Compliance Rules

From `architecture/SS-daemon-wiring.md v1.2.0 §Daemon Start Sequence (BC-2.04.001)` (at S-017 authoring time):
- SOQ-2 ordering is ABSOLUTE: step 3 < step 8 < step 9. No reordering permitted.
- `tempfile::persist` MANDATORY for both lock file (step 8) and hooks-settings.json (step 9)
- `std::fs::write` is FORBIDDEN for both files
- Mode `0o600` for both files; mode `0o700` for runtime_dir
- `contract_version` field in lock file JSON: `"monocle-lock-v1"` (string, not integer)
- `DirBuilder::new().mode(0o700)` — NOT `std::fs::create_dir_all`
- If any step > 8 fails, lock file MUST be removed before exit

From `architecture/SS-daemon-wiring.md v1.2.0 §Hook Tmpfile Generation` (at S-017 authoring time):
- `hooks-settings.json` schema: `PostToolUse` and `PreCompact` MUST be present with empty arrays
- `SessionStart` must NOT appear in `hooks-settings.json`
- Only 4 hook types have non-empty arrays (PreToolUse, Notification, Stop, UserPromptSubmit)

**Forbidden Dependencies:**
- `monocle-runtime/src/lifecycle.rs` MUST NOT import from `monocle-tui`
- `std::fs::write` MUST NOT be used for either lock file or hooks-settings.json
- `std::fs::create_dir_all` MUST NOT be used for runtime dir creation

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| tempfile | 3 | Atomic write for lock file (step 8) and hooks-settings.json (step 9) |
| rand | =0.8.6 | `OsRng` for auth token generation (step 7) — EXACT pin |
| serde_json | =1.0.149 | JSON serialization for both files |
| tokio | =1.52.0 | `TcpListener`, `mpsc::channel`, `UnixListener` |
| axum | =0.8.9 | `run_server()` at step 12 |
| tracing | 0.1 | INFO/WARN/ERROR structured logging throughout |
| nix | 0.30 | Stale lock/socket PID liveness check |
| directories | 6 | `ProjectDirs::from("", "", "monocle")` |

## File Structure Requirements

Files to create:
- `monocle-runtime/tests/daemon_start_sequence.rs` — integration tests for 13-step sequence

Files to modify:
- `monocle-runtime/src/lifecycle.rs` — add `daemon_start_sequence()`, `write_hooks_settings()`, `DaemonState`
- `monocle-runtime/src/lib.rs` — re-export `daemon_start_sequence`, `DaemonState`

## Downstream Consumer Contract

Public API produced by this story:

```
pub async fn daemon_start_sequence() -> Result<(), DaemonStartError>

pub struct DaemonState {
    pub ring: Arc<RingBuffer>,
    pub event_bus_tx: Arc<EventBusTx>,
    pub drop_counter: Arc<AtomicU64>,
    pub auth_token: Arc<String>,
    pub engine_registry: EngineModuleRegistry,
    pub sock_file_path: PathBuf,
    pub session_registry: SessionRegistry,
    pub tui_clients: TuiClientList,
}
```

S-018 (hook routing + event bus) consumes `DaemonState` directly. S-021 (UDS server) adds
to `DaemonState.tui_clients`. S-019 (auto-start) calls `daemon_start_sequence()` from TUI launch path.

## §Trace

**v1.1** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at story authoring time. No normative content changed.
