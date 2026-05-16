---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-17T11:30:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
traces_to: prd.md
origin: greenfield
subsystem: SS-01
capability: CAP-001
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.01.005: Lock File Atomic Lifecycle (Create + Pid Check + Cleanup)

## Description

The monocle daemon creates and manages a lock file at `<runtime_dir>/monocle.lock` to enforce
single-instance semantics. The runtime directory is resolved via a platform-aware chain
(MONOCLE_RUNTIME_DIR env override → XDG runtime dir → data_local_dir fallback) to support
Linux, macOS, and containerized deployments without per-platform user configuration. The lock
file is written atomically via `tempfile::persist` with mode `0o600`, and the containing
directory is created with mode `0o700` (owner-only) for defense-in-depth.

## Preconditions

1. The monocle daemon is starting up (executing the start sequence).
2. The runtime directory `<runtime_dir>` is resolved via the following platform-aware chain (evaluated in order; first `Some` result wins):
   - (a) `MONOCLE_RUNTIME_DIR` environment variable — if set and non-empty, use as the runtime directory path verbatim. This is the operator escape hatch for containers, NixOS, and non-standard deployments.
   - (b) `directories::ProjectDirs::runtime_dir()` — returns `Some` on Linux (XDG `$XDG_RUNTIME_DIR/monocle`); returns `None` on macOS and Windows by platform-ABI design (not misconfiguration).
   - (c) `directories::ProjectDirs::data_local_dir()` — platform fallback for macOS (`~/Library/Application Support/monocle/`) and Windows (`%APPDATA%/monocle/`), and any Linux environment where `XDG_RUNTIME_DIR` is not set.
   - (d) If `MONOCLE_RUNTIME_DIR` is unset or empty AND `ProjectDirs::new("monocle", "monocle", "monocle")` returns `None` (no usable home directory; rare but possible in misconfigured containers), the daemon exits 1 with `DaemonStartError::RuntimeDirUnresolvable`. Error message: `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`. Note: paths (b) and (c) require a valid `ProjectDirs` instance; if `ProjectDirs::new()` succeeded, path (c) `data_local_dir()` always returns a valid path (never `None`) — the fail-fast only triggers when `ProjectDirs::new()` itself returned `None`.

   Rationale: `ProjectDirs::runtime_dir()` returns `None` on macOS and Windows by design — not due to misconfiguration. macOS is among the primary target platforms (NFR-008: `macOS + Linux`, darwin/linux × amd64/arm64). A fail-fast-only approach would require every macOS user to set `MONOCLE_RUNTIME_DIR` before starting monocle, which violates the zero-config startup requirement. The `data_local_dir()` fallback provides a correct, standards-compliant runtime state location on macOS (`~/Library/Application Support/monocle/`). Windows is a secondary build target per PRD §8.7; the same `data_local_dir()` fallback resolves to `%APPDATA%/monocle/` on Windows but Phase 1 CI does not formally validate Windows behavior per NFR-008's `macOS + Linux` target scope.

## Postconditions

**Start sequence:**
1. If a lock file exists at `<runtime_dir>/monocle.lock` with a live pid (`kill(pid, 0)` succeeds): daemon logs `ERROR: daemon already running at pid=<N>; exiting` and exits 1.
2. If a lock file exists with a dead pid: daemon logs `WARN: stale lock file removed` and proceeds with startup.
3. The lock file is written atomically via `tempfile::persist` to `<runtime_dir>/monocle.lock` after the daemon has bound its listener and obtained a port. Lock file mode: `0o600`.
4. The lock file JSON has `contract_version` as the first key (value `1`), followed by `pid`, `port`, `authToken`, `startTimeUtc`, `app`, `version`.
5. If `DaemonStartError::RuntimeDirUnresolvable` is raised (resolution path (d) reached), the daemon exits 1 with the message above. No lock file is created.

**Clean shutdown:**
6. On successful graceful shutdown, `<runtime_dir>/monocle.lock` is removed.
7. On successful graceful shutdown, `<runtime_dir>/monocle.sock` is removed.

**Runtime directory creation:**
8. If `<runtime_dir>` does not exist at start, daemon creates it with mode `0o700` (owner-only access) using `DirBuilder::new().mode(0o700).recursive(true).create(&runtime_dir)` (with `use std::os::unix::fs::DirBuilderExt` to bring the `mode` method into scope) — NOT `std::fs::create_dir_all` (which honors umask defaults, typically 0o755). This is defense-in-depth alongside the lock-file `0o600` mode (BC-2.01.010): both the containing directory AND the lock file must be owner-only to prevent other OS users from enumerating or reading auth-token-bearing paths. F-R75-1 establishes this contract. Cross-reference: VP-DAEMON-005 Post-condition 9 and probe 5.e verify this with `stat(&runtime_dir).mode() & 0o777 == 0o700`. Cross-platform note: the `0o700` mode assertion applies to Linux/macOS (primary targets per NFR-008); Windows does not expose Unix mode bits and is a secondary build target per §8.7.

## Invariants

1. Only one monocle daemon instance runs per runtime directory. The pid-liveness check (step 1) enforces this.
2. `tempfile::persist` guarantees atomicity — no partial lock file is observable by concurrent readers.
3. Lock file mode `0o600` prevents other OS users from reading the auth token.
4. The asymmetry with BC-ENGINE-002-ERR (`HomeUnresolvable` fail-fast) is intentional: `BaseDirs::new() == None` signals a genuine system-configuration failure (no home directory at all); `ProjectDirs::runtime_dir() == None` on macOS is expected platform behavior, warranting a documented fallback rather than fail-fast.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-051 | Lock file write fails (filesystem full, permission denied) | Daemon exits before accepting any requests; no partial lock file with wrong or empty content is left on disk (tempfile guarantees) |
| EC-052 | Runtime directory does not exist on startup | Daemon creates it with mode `0o700` (owner-only); if directory creation fails, daemon logs error and exits 1 |
| EC-053 | Lock file removed between pid-liveness check and atomic write (TOCTOU race) | `tempfile::persist` atomic-replace pattern mitigates this — the rename step is atomic on POSIX filesystems |
| EC-057 | macOS startup — `MONOCLE_RUNTIME_DIR` not set, `ProjectDirs::runtime_dir()` returns `None` (expected on macOS) | `data_local_dir()` returns `Some("~/Library/Application Support/monocle/")`; daemon uses the `data_local_dir` path; logs `INFO: runtime_dir fallback to data_local_dir (platform: macos)` |
| EC-058 | `MONOCLE_RUNTIME_DIR` env override — operator sets `MONOCLE_RUNTIME_DIR=/tmp/monocle-test` | Daemon uses `/tmp/monocle-test` as runtime directory regardless of platform-default resolution; logs `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var` |
| EC-059 | Full-fail path — `MONOCLE_RUNTIME_DIR` not set, `ProjectDirs::new("monocle", "monocle", "monocle")` returns `None` (requires no home directory at all) | Daemon exits 1 with `DaemonStartError::RuntimeDirUnresolvable` and message `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`; no lock file created |
| EC-060 | `MONOCLE_RUNTIME_DIR=""` (empty string set via buggy shell script: `export MONOCLE_RUNTIME_DIR=$UNDEFINED_VAR`) | Daemon treats empty string as unset (per Precondition 2(a) "if set and non-empty") and silently falls through to platform-default resolution (path (b) `ProjectDirs::runtime_dir()`); NO error log line; NO daemon-startup failure; resolved `runtime_dir` is the platform default (not an empty path) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `monocle daemon start` (no lock file) | Lock file created at `<runtime_dir>/monocle.lock` with mode 0600 and `contract_version == 1` as first key | happy-path |
| Lock file exists; pid is not alive | WARN logged; old lock file removed; new daemon starts | edge-case |
| Lock file exists; pid is alive | Error logged; exit 1 | error |
| Daemon exits gracefully | Lock file removed; UDS socket removed | happy-path |
| `MONOCLE_RUNTIME_DIR` unset; `ProjectDirs::runtime_dir()` returns `None` (macOS) | `data_local_dir()` used; INFO logged; daemon starts normally | edge-case |
| `MONOCLE_RUNTIME_DIR=/tmp/monocle-test` | `/tmp/monocle-test` used as runtime dir; INFO logged | edge-case |
| `MONOCLE_RUNTIME_DIR` unset; `ProjectDirs::new(...)` returns `None` | `DaemonStartError::RuntimeDirUnresolvable` raised; exit 1; no lock file created | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-005 | Lock file created with mode 0600 and `contract_version == 1` as first key on clean start | integration |
| VP-005 | Runtime directory created with mode 0700 (owner-only) when absent on start | integration |
| VP-005 | Lock file removed on clean shutdown | integration |
| VP-005 | Stale lock file (dead pid) is removed and daemon starts normally | integration |
| VP-005 | `DaemonStartError::RuntimeDirUnresolvable` triggers exit 1 with correct error message | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs the lock file lifecycle that is the single-instance enforcement mechanism for the daemon lifecycle subsystem |
| L2 Domain Invariants | N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source |
| Architecture Module | monocle-runtime (daemon binary, lock file, auth) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.25 §Daemon Lifecycle Protocol §Start Sequence and §Hard Shutdown |
| Cross-Ref | BC-2.01.010 (lock file JSON schema contract) |
| Test File | `monocle-runtime/tests/lock_file_lifecycle.rs` |
| Test Name | `test_BC_DAEMON_005_lock_file_create_and_cleanup` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-DAEMON-005 |

## Related BCs (Recommended)

- [BC-2.01.004] — depends on: clean shutdown removes the lock file (per BC-2.01.004 drain sequence)
- [BC-2.01.010] — composes with: lock file JSON schema (contract_version first key) is formally specified in BC-2.01.010
- [BC-2.01.008] — composes with: auth token in lock file is governed by BC-2.01.008 (authToken wire format)

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#daemon-lifecycle-protocol` — start sequence, lock file creation, pid-liveness check
- `architecture/SS-daemon-lifecycle.md#hard-shutdown` — lock file removal on exit

## Story Anchor (Recommended)

S-TBD — Implement daemon lock file lifecycle with platform-aware runtime directory resolution (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-005-lock-file-lifecycle.md` — VP-005 lock file lifecycle integration tests
