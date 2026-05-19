---
document_type: story
story_id: S-006
epic_id: EPIC-01
version: "1.3"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 8
wave: 2
tdd_mode: strict
priority: P0
depends_on: [S-001]
blocks: [S-007, S-008]
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.005, BC-2.01.008, BC-2.01.010]
verification_properties: [VP-005, VP-010]
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.12"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.005.md, version: "1.0.4"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.008.md, version: "1.0.6"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md, version: "1.0.4"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-005-lock-file-lifecycle.md, version: "1.0.16"}
  - {path: .factory/specs/verification-properties/vp-010-lock-file-contract-version.md, version: "1.0.14"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/prd-supplements/error-taxonomy.md, version: "1.5"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.01.005 (Lock File Atomic Lifecycle), BC-2.01.008 (Auth Token Generation — PC-1 cryptographic token written at lock file creation), BC-2.01.010 (Lock File Contract Version Field); verifies VP-005, VP-010; covers EC-010, EC-011, EC-012, EC-051, EC-052, EC-053, EC-057, EC-058, EC-059, EC-060; addresses NFR-009, NFR-012, E-LOCK-001, E-LOCK-002, E-LOCK-003, E-DAEMON-004."
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

### AC-012 (traces to BC-2.01.010 edge case EC-011 — contract_version as string instead of integer)
If `contract_version` key is present but its value is a string (e.g., `"1"` instead of `1`),
the Phase 1 reader must handle gracefully: coerce to integer if parseable OR log E-LOCK-002 and
skip the lock file. No crash, no panic (BC-2.01.010 EC-011).

### AC-013 (traces to BC-2.01.010 edge case EC-012 — contract_version key missing entirely)
If `contract_version` key is missing from the lock file entirely (pre-Phase-1 format), same
treatment as EC-010: log `WARN: lock file contract_version missing; skipping` and proceed as
if no lock file exists (BC-2.01.010 EC-012).

### AC-014 (Orchestrator Decision 3 + traces to BC-2.01.008 postcondition 1 — real cryptographic auth token generation in S-006)
The lock file `authToken` field is populated with a REAL cryptographically random token at
lock file creation time (BC-2.01.008 PC-1). Implementation:
- `monocle_runtime::auth::generate_session_token() -> String` generates 32 cryptographically
  random bytes using `rand::rngs::OsRng` (rand `=0.8.6` EXACT pin per SS-deps-pin-manifest.md),
  hex-encoded as a 64-character lowercase hex string matching regex `/^[0-9a-f]{64}$/`.
  This function lives in `monocle-runtime/src/auth.rs` — NOT in a separate `monocle-auth` crate
  (Orchestrator Decision 3: no new crate justified for a one-function helper; pin manifest
  already declares `runtime --> rand` as the canonical OsRng consumer edge).
- `DaemonLock::acquire()` calls `monocle_runtime::auth::generate_session_token()` and stores
  the result in the `authToken` field BEFORE calling `tempfile::persist`.
- No placeholder value is ever written to the lock file.
- S-009 reads the 64-hex token from the lock file; no placeholder retrofit is needed.

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
- [ ] Implement `monocle_runtime::auth::generate_session_token() -> String` in `monocle-runtime/src/auth.rs`:
  - 32 random bytes from `rand::rngs::OsRng` (EXACT pin `=0.8.6` per SS-deps-pin-manifest.md)
  - Hex-encoded to 64-character lowercase hex string (BC-2.01.008 PC-1; regex `/^[0-9a-f]{64}$/`)
  - Token stored in `Arc<String>` for sharing between lock file writer and auth middleware
  - NOTE: function lives in `monocle-runtime/src/auth.rs`, NOT in a separate `monocle-auth` crate (Orchestrator Decision 3)
- [ ] Lock file `authToken` field: populated by calling `monocle_runtime::auth::generate_session_token()` at `DaemonLock::acquire()` time — no placeholder value ever written
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
`rand = "=0.8.6"` (EXACT pin per SS-deps-pin-manifest.md v1.1.17, §rand row line 43) is the
canonical version for `OsRng`. `monocle-auth` is NOT a workspace crate (Orchestrator Decision 3);
`generate_session_token()` is implemented in `monocle-runtime/src/auth.rs` (new module). `rand 0.9` is explicitly REJECTED by the pin
manifest: `OsRng` moved to a feature flag in 0.9, which is an ergonomic regression that
SS-deps-pin-manifest.md resolves by pegging to 0.8.6.
Auth token is generated and written in this story — NO placeholder value is used.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.33 §Start Sequence and §Hard Shutdown:
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
