---
document_type: story
level: L4
story_id: S-016
epic_id: EPIC-04
version: "1.2"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 5
wave: 4
tdd_mode: strict
priority: P0
depends_on: [S-001, S-006]
blocks: [S-017, S-019, S-DAEMON-WIRE-FIX-001]
target_module: monocle
subsystems: [SS-04]
behavioral_contracts: [BC-2.04.004, BC-2.04.005, BC-2.04.006]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.004.md, version: "1.4.0"}
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.005.md, version: "1.4.0"}
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.006.md, version: "1.5.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.04.004 (monocle daemon start CLI), BC-2.04.005 (monocle daemon stop CLI), BC-2.04.006 (runtime_dir fallback chain)"
---

# S-016: Daemon Binary Crate Init + CLI Subcommands (`monocle daemon start/stop`)

## Narrative

As a daemon operator, I want the `monocle` binary to expose `daemon start` and `daemon stop`
subcommands with correct exit codes and foreground polling semantics, so that the daemon
lifecycle can be managed from the command line without manual pid-file juggling.

## Acceptance Criteria

### AC-001 (traces to BC-2.04.006 postcondition PC-1 — MONOCLE_RUNTIME_DIR level 1)
When `MONOCLE_RUNTIME_DIR` is set to a non-empty string, `resolve_runtime_dir()` returns that
path verbatim and logs `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var`. When set to the
empty string, resolution falls through to level 2.

### AC-002 (traces to BC-2.04.006 postcondition PC-5/PC-6 — level 2: runtime_dir())
On Linux with `XDG_RUNTIME_DIR=/run/user/1000` set and no `MONOCLE_RUNTIME_DIR`, `resolve_runtime_dir()`
returns `$XDG_RUNTIME_DIR/monocle` and logs `INFO: runtime_dir from ProjectDirs::runtime_dir()`.

### AC-003 (traces to BC-2.04.006 postcondition PC-8/PC-9 — level 3: data_local_dir())
On macOS (where `proj.runtime_dir()` returns `None`), `resolve_runtime_dir()` returns
`~/Library/Application Support/monocle/` and logs
`INFO: runtime_dir fallback to data_local_dir (platform: macos)`.

### AC-004 (traces to BC-2.04.006 postcondition PC-12/PC-13 — level 4: fail-fast)
When `ProjectDirs::from("", "", "monocle")` returns `None` (no home directory),
`resolve_runtime_dir()` returns `Err(DaemonStartError::RuntimeDirUnresolvable)`. The CLI
exits with code 70 and writes to stderr:
`ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`

### AC-005 (traces to BC-2.04.004 postcondition PC-1/PC-2/PC-3/PC-4 — happy path)
`monocle daemon start` (no live daemon): checks PID liveness, spawns detached daemon subprocess,
polls `<runtime_dir>/monocle.lock` at 100ms intervals, exits 0 with no stdout when the lock file
appears within 10 seconds.

### AC-006 (traces to BC-2.04.004 postcondition PC-6 — already running)
`monocle daemon start` when a live daemon is already running writes to stderr:
`error: daemon already running (pid=<N>)` and exits 1.

### AC-007 (traces to BC-2.04.004 postcondition PC-7 — timeout)
`monocle daemon start` when the lock file does not appear within 10 seconds writes to stderr:
`error: daemon failed to start within 10 s` and exits 1.

### AC-008 (traces to BC-2.04.004 postcondition PC-8 — exit codes)
Exit codes for `monocle daemon start`: 0 (success), 1 (already running or timeout), 70
(RuntimeDirUnresolvable), 71 (internal error from BC-2.04.001).

### AC-009 (traces to BC-2.04.005 postcondition PC-1/PC-2/PC-3/PC-4 — happy path)
`monocle daemon stop` (daemon running): reads PID from lock file, sends SIGTERM, polls at
1-second intervals, exits 0 with no stdout when the daemon process exits within 15 seconds.

### AC-010 (traces to BC-2.04.005 postcondition PC-5 — lock file absent)
`monocle daemon stop` (no lock file) writes to stderr:
`error: no lock file found; daemon may not be running` and exits 1.

### AC-011 (traces to BC-2.04.005 postcondition PC-6 — stale lock)
`monocle daemon stop` (lock file exists, PID dead) writes to stderr:
`error: daemon not running (stale lock file?)` and exits 1.

### AC-012 (traces to BC-2.04.005 postcondition PC-7 — timeout)
`monocle daemon stop` when the process does not exit within 15 seconds writes to stderr:
`error: daemon did not exit within 15 s; it may still be draining` and exits 2. No SIGKILL is sent.

### AC-013 (traces to BC-2.04.005 invariant 1 — no SIGKILL)
`monocle daemon stop` NEVER sends SIGKILL. The daemon is responsible for its own graceful shutdown.

### AC-014 (traces to BC-2.04.004 invariant 2 — detachment)
The daemon subprocess launched by `monocle daemon start` continues running after the foreground
caller exits. The daemon's process group is set such that it survives terminal session end
(double-fork or `setsid()` equivalent).

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,400 |
| BC-2.04.004.md | ~900 |
| BC-2.04.005.md | ~900 |
| BC-2.04.006.md | ~1,000 |
| SS-daemon-wiring.md §CLI Interface | ~3,000 |
| S-006 (lock file / resolve_runtime_dir) | ~600 |
| clap 4.6 usage | ~400 |
| Test files | ~800 |
| **Total estimate** | **~9,000** |

## Tasks

- [ ] Create `monocle/` binary crate with `Cargo.toml` listing `clap 4.6` (caret pin) and workspace members
- [ ] Implement `resolve_runtime_dir()` in `monocle-runtime/src/lifecycle.rs` with 4-level chain using `ProjectDirs::from("", "", "monocle")`
- [ ] Implement `DaemonStartError` enum: `RuntimeDirUnresolvable`, `LockFileConflict`, `LockFileWriteFailure`, `TokenGenerationFailed`
- [ ] Add `clap` CLI parser in `monocle/src/main.rs`: `monocle daemon start` and `monocle daemon stop` subcommands
- [ ] Implement `cmd_daemon_start()`: PID liveness check → detached subprocess spawn via `nohup`/`setsid` → 100ms poll loop (10s timeout) → exit 0 or 1
- [ ] Implement `cmd_daemon_stop()`: read lock file PID → SIGTERM via `nix::sys::signal::kill(Pid, SIGTERM)` → 1s poll loop (15s timeout) → exit 0, 1, or 2
- [ ] Implement `Invariant 1: no SIGKILL` — only SIGTERM is ever sent in stop
- [ ] Double-fork daemon detachment: the daemon subprocess must survive parent terminal exit
- [ ] Exit code routing: 70 (RuntimeDirUnresolvable), 71 (internal error), as per BC-2.04.004 PC-8 / BC-2.04.005 PC-8
- [ ] Integration tests `monocle/tests/cli_daemon_start.rs` covering AC-005 through AC-008
- [ ] Integration tests `monocle/tests/cli_daemon_stop.rs` covering AC-009 through AC-013
- [ ] Unit tests `monocle-runtime/tests/runtime_dir_resolution.rs` covering AC-001 through AC-004 (env mock via `temp-env`)

## Previous Story Intelligence

S-001: Cargo workspace initialized. `directories 6`, `tempfile 3`, `nix 0.30`, `temp-env 0.3`
(features=["async_closure"]) all pinned in workspace. `clap 4.6` is in SS-deps-pin-manifest.md
caret pin.

S-006: `resolve_runtime_dir()` was partially specified in BC-2.01.005 Precondition 2. This story
implements it as the full BC-2.04.006 normative specification. BC-2.04.006 supersedes any earlier
stub in S-006. The function lives in `monocle-runtime/src/lifecycle.rs`.

## Architecture Compliance Rules

From `architecture/SS-daemon-wiring.md v1.2.0 §CLI Interface` (at S-016 authoring time):
- `ProjectDirs::from("", "", "monocle")` — NOT `ProjectDirs::new(...)` (the `new` constructor does not exist)
- 4-level fallback chain is strict: L1 (MONOCLE_RUNTIME_DIR) → L2 (runtime_dir()) → L3 (data_local_dir()) → L4 (fail-fast)
- Level 4 exit code is 70, not 1
- No SIGKILL in `daemon stop` — invariant 1 is absolute
- Double-fork: daemon must survive parent shell exit (setsid or nohup pattern)
- No stdout on success for both subcommands

From `architecture/SS-conventions-anti-patterns.md v1.29.5` (at S-016 authoring time):
- `nix::sys::signal::kill` for signaling — NOT raw `libc::kill`
- Structured error types via `thiserror` for `DaemonStartError`

**Forbidden Dependencies:**
- `monocle` binary MUST NOT depend on `monocle-tui` crate
- `resolve_runtime_dir()` MUST NOT use `ProjectDirs::new(...)` (wrong constructor)
- `daemon stop` MUST NOT call `libc::kill` with SIGKILL under any circumstances

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| clap | 4.6 | CLI argument parsing; `monocle daemon start/stop` subcommand tree |
| directories | 6 | `ProjectDirs::from("", "", "monocle")` for runtime dir |
| nix | 0.30 | `kill(Pid::from_raw(pid), SIGTERM)` and `kill(pid, None)` liveness |
| serde_json | =1.0.149 | Lock file JSON parsing in stop (PID extraction) |
| tracing | 0.1 | INFO log for level selection; ERROR for fail-fast |
| temp-env | 0.3 (features=["async_closure"]) | Test: MONOCLE_RUNTIME_DIR env isolation |
| thiserror | 2.x | `DaemonStartError` enum |

## File Structure Requirements

Files to create:
- `monocle/src/main.rs` — clap CLI parser, `cmd_daemon_start()`, `cmd_daemon_stop()`
- `monocle/Cargo.toml` — binary crate manifest listing `clap`, `serde_json`, `nix`
- `monocle/tests/cli_daemon_start.rs` — integration tests for start subcommand
- `monocle/tests/cli_daemon_stop.rs` — integration tests for stop subcommand

Files to modify:
- `monocle-runtime/src/lifecycle.rs` — add `resolve_runtime_dir()` (full BC-2.04.006 4-level chain)
- `monocle-runtime/src/lib.rs` — re-export `resolve_runtime_dir`, `DaemonStartError`
- `Cargo.toml` (workspace root) — add `monocle` as workspace member

## Downstream Consumer Contract

Public API produced by this story for downstream consumption:

```
// monocle-runtime
pub fn resolve_runtime_dir() -> Result<PathBuf, DaemonStartError>

pub enum DaemonStartError {
    RuntimeDirUnresolvable,
    LockFileConflict { pid: i32 },
    LockFileWriteFailure(std::io::Error),
    TokenGenerationFailed,
}
```

S-017 (daemon start sequence) calls `resolve_runtime_dir()` as step 1 of the 13-step start
sequence. S-019 (auto-start) also calls `resolve_runtime_dir()` before the liveness check.

## §Trace

**v1.1** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at story authoring time. No normative content changed.
