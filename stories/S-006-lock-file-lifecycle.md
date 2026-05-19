---
document_type: story
story_id: S-006
epic_id: EPIC-01
version: "1.0"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 8
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001]
blocks: [S-007, S-008, S-009]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.005, BC-2.01.010]
verification_properties: [VP-005, VP-010]
estimated_days: 3
---

# S-006: Lock File Atomic Lifecycle (Create + Pid Check + Cleanup)

## Narrative

As a daemon administrator, I want the monocle daemon to enforce single-instance semantics
via an atomic lock file at `<runtime_dir>/monocle.lock`, so that concurrent daemon starts
are prevented, stale locks are cleaned up, and auth tokens are not readable by other OS users.

## Acceptance Criteria

### AC-001 (traces to BC-2.01.005 postcondition 3 — atomic write via tempfile::persist)
On a clean start (no lock file exists), the daemon writes the lock file atomically via
`tempfile::persist`. The lock file is created at `<runtime_dir>/monocle.lock` with mode
`0o600` (owner-read-write only). No partial lock file is observable during creation.

### AC-002 (traces to BC-2.01.005 postcondition 4 — JSON field order)
The lock file JSON content has `contract_version` as the FIRST key (value `1`), followed
by `pid`, `port`, `authToken`, `startTimeUtc`, `app`, `version`. Key order is enforced
by using an ordered serialization approach (not HashMap serialization).

### AC-003 (traces to BC-2.01.005 postcondition 1 — live pid conflict → exit 1)
If a lock file exists at startup with a live PID (`nix::sys::signal::kill(Pid::from_raw(pid), None)` returns Ok),
the daemon logs `ERROR: daemon already running at pid=<N>; exiting` and exits 1 (E-LOCK-001).

### AC-004 (traces to BC-2.01.005 postcondition 2 — stale lock cleanup)
If a lock file exists with a dead PID, the daemon logs `WARN: stale lock file removed`
(E-LOCK-002) and proceeds with normal startup.

### AC-005 (traces to BC-2.01.005 postconditions 6–7 — cleanup on shutdown)
On successful graceful shutdown, `<runtime_dir>/monocle.lock` is removed. Also,
`<runtime_dir>/monocle.sock` is removed.

### AC-006 (traces to BC-2.01.005 postcondition 8 — runtime directory 0o700)
If `<runtime_dir>` does not exist at start, it is created with mode `0o700` (owner-only)
using `DirBuilder::new().mode(0o700).recursive(true).create(&runtime_dir)` with
`std::os::unix::fs::DirBuilderExt` in scope. NFR-012 validation gate.

### AC-007 (traces to BC-2.01.005 precondition 2a — MONOCLE_RUNTIME_DIR override)
If `MONOCLE_RUNTIME_DIR` is set and non-empty, it is used as the runtime directory path.
Logs `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var`.

### AC-008 (traces to BC-2.01.005 precondition 2b/2c — platform fallback chain)
On macOS (where `directories::ProjectDirs::runtime_dir()` returns `None`),
`data_local_dir()` is used as fallback. Logs `INFO: runtime_dir fallback to data_local_dir (platform: macos)`.

### AC-009 (traces to BC-2.01.005 precondition 2d — RuntimeDirUnresolvable fail-fast)
If `ProjectDirs::new("monocle", "monocle", "monocle")` returns `None` (no home dir),
daemon exits 1 with `DaemonStartError::RuntimeDirUnresolvable` and message:
`ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`.

### AC-010 (traces to BC-2.01.010 postcondition 1 — contract_version first key, value 1)
The `contract_version` field in the lock file JSON is the first key and has value `1`.
If the daemon reads a lock file with an unrecognized `contract_version`, it logs
`WARN: lock file contract_version <N> not recognized; skipping` (E-LOCK-003) and
removes the stale file.

### AC-011 (traces to BC-2.01.010 edge case EC-010)
If `contract_version` key is missing from an existing lock file, the daemon treats
the lock file as stale, logs E-LOCK-002 (stale removal), and restarts.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,100 |
| BC-2.01.005.md | ~900 |
| BC-2.01.010.md | ~600 |
| VP-005 + VP-010 files | ~1,000 |
| SS-daemon-lifecycle.md (start sequence, lock file, hard shutdown) | ~5,000 |
| tempfile + directories + nix crate usage | ~500 |
| Test file | ~1,000 |
| **Total estimate** | **~10,100** |

## Tasks

- [ ] Implement `resolve_runtime_dir()` in `monocle-runtime/src/lifecycle.rs` with 4-path chain
- [ ] Create `DaemonStartError` enum with `RuntimeDirUnresolvable`, `LockFileConflict`, `LockFileWriteFailure` variants
- [ ] Create `monocle-runtime/src/lock.rs` with `DaemonLock` struct
- [ ] Implement `DaemonLock::acquire()`: read existing lock → pid-liveness check → clean stale → write new
- [ ] Use `tempfile::NamedTempFile` + `persist()` for atomic lock file write
- [ ] Use `serde_json::to_string()` with a serialization that preserves field order (use `indexmap` or manually ordered struct with `#[serde(rename_all = "camelCase")]`)
- [ ] Lock file `authToken` field: raw 64-hex string (no prefix) — generated in S-009
- [ ] Create `monocle-runtime/src/lock.rs` cleanup: `DaemonLock::release()` removes lock + sock
- [ ] Runtime directory creation with `0o700` mode using `DirBuilderExt`
- [ ] Integration tests `monocle-runtime/tests/lock_file_lifecycle.rs`:
  - Clean start → lock file created mode 0600, contract_version first key
  - Runtime dir created mode 0700 when absent
  - Live pid conflict → exit 1 + error log
  - Dead pid (stale) → WARN log + restart
  - Clean shutdown → lock file removed + sock removed
  - MONOCLE_RUNTIME_DIR env override
  - macOS platform fallback (mocked ProjectDirs via temp-env)
  - RuntimeDirUnresolvable → exit 1 + error message
- [ ] Integration test `monocle-runtime/tests/lock_file_contract.rs` for EC-010 (unknown contract_version)

## Previous Story Intelligence

S-001 (Wave 1): Workspace initialized. `directories 6` and `tempfile 3` pinned in workspace.
`nix 0.30` pinned. `temp-env 0.3` pinned as dev-dependency.
The lock file `authToken` field is filled with a placeholder `"<TBD>"` value until S-009
delivers auth token generation. Tests use a synthetic token.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.32 §Start Sequence and §Hard Shutdown:
- `tempfile::persist` is MANDATORY for atomic write — `std::fs::write` is FORBIDDEN
- `DirBuilder::new().mode(0o700)` is MANDATORY — `std::fs::create_dir_all` is FORBIDDEN (umask issue)
- `nix::sys::signal::kill(Pid::from_raw(pid), None)` for pid-liveness — NOT `libc::kill` directly
- Lock file written AFTER port is bound (authToken ordering invariant DI-003)

From `architecture/SS-conventions-anti-patterns.md` v1.29.5:
- Atomic writes via `tempfile::persist` — codified as a named forbidden anti-pattern
- No `std::fs::write` for config-class files

**Forbidden Dependencies:**
- `monocle-runtime/src/lock.rs` MUST NOT import from `monocle-tui`
- `std::fs::write` MUST NOT be used for lock file creation (use `tempfile::persist`)
- `std::fs::create_dir_all` MUST NOT be used for runtime dir creation (use `DirBuilderExt`)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| tempfile | 3 | Atomic lock file write via `NamedTempFile::persist()` |
| directories | 6 | `ProjectDirs::new("monocle", "monocle", "monocle")` |
| nix | 0.30 | `kill(Pid::from_raw(pid), None)` pid-liveness |
| serde_json | =1.0.149 | Lock file JSON serialization |
| tracing | 0.1 | INFO/WARN/ERROR log entries |
| chrono | 0.4 | `startTimeUtc` ISO 8601 field |
| temp-env | 0.3 | Test: `MONOCLE_RUNTIME_DIR` env isolation |

## File Structure Requirements

Files to create:
- `monocle-runtime/src/lock.rs` — `DaemonLock`, `acquire()`, `release()`, lock file JSON struct
- `monocle-runtime/tests/lock_file_lifecycle.rs` — integration tests
- `monocle-runtime/tests/lock_file_contract.rs` — contract_version tests

Files to modify:
- `monocle-runtime/src/lifecycle.rs` — `resolve_runtime_dir()`, `DaemonStartError`
- `monocle-runtime/src/lib.rs` — add `pub mod lock;`
