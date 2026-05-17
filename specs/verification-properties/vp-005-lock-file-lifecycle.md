---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:00:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "3547eed"
traces_to: prd.md
source_bc: BC-2.01.005
module: monocle-runtime
proof_method: manual+mutation
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-005: Lock File Lifecycle — Atomic Create, Pid-Liveness Gate, Mode 0o600, Cleanup, 4-Path Resolution

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-DAEMON-005 (PG-5 historical) per template-compliance Dispatch 5a.

## Property Statement

The daemon resolves `<runtime_dir>` via an ordered 4-path chain
(`MONOCLE_RUNTIME_DIR` env → `ProjectDirs::runtime_dir()` →
`ProjectDirs::data_local_dir()` → fail-fast `RuntimeDirUnresolvable`). On
start it atomically creates `<runtime_dir>/monocle.lock` via
`tempfile::persist` with mode `0o600`; the containing `<runtime_dir>` is
created with mode `0o700` (defense-in-depth). On start with an existing lock
file the daemon checks pid-liveness via `kill(pid, 0)` and either exits 1
(live) or proceeds with stale-pid recovery (ESRCH). On clean shutdown the
lock file AND `monocle.sock` are removed. Naked `std::fs::write` for the
lock-file path is forbidden (source-grep negative assertion). The asymmetry
with BC-2.03.003 `HomeUnresolvable` is intentional.

## Source Contract

- **BC (primary):** BC-2.01.005 — Lock File Atomic Lifecycle (Create + Pid
  Check + Cleanup).
- **BCs (partial coverage):** BC-2.01.010 (joint mode-and-content assertion
  via `contract_version` first key), BC-2.03.003 (asymmetry rationale with
  `HomeUnresolvable`), BC-2.01.008 (defense-in-depth pairing with `0o700`
  runtime-dir mode).
- **Postcondition/Invariant:** 4-path resolution-chain ordering (EC-057,
  EC-058, EC-059); atomic-create via `tempfile::persist`; mode `0o600` for
  lock file; mode `0o700` for runtime dir on creation; pid-liveness gate;
  clean-shutdown cleanup; `RuntimeDirUnresolvable` fail-fast → exit 1.
- **Traces to (historical):** BC-DAEMON-005 (PRD v1.25 §BC-DAEMON-005;
  SS-daemon-lifecycle.md v1.0.25 §Start Sequence + §Hard Shutdown;
  F-R70-1 closure — hybrid runtime-dir resolution chain disposition (c);
  F-R88-2 wording correction landed in PRD v1.17 commit 27e663c and
  carried forward verbatim into PRD v1.25 commit 7735c84 per C-R90-1).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test + `tempfile::TempDir` + `temp-env` env-var isolation | Bounded — finite probe set per path | All 4 resolution-chain paths; mode-bit assertions; pid-liveness gate; cleanup |
| Mutation test (auxiliary) | cargo-mutants | N/A — mutation surface | `0o600`, `0o700`, `kill(pid, 0)`, chain ordering all mutation surfaces |
| Source-grep (structural) | ripgrep | N/A — static | `tempfile::persist` present; no `std::fs::write` for `monocle.lock`; chain-ordering preserved |

## Mechanism

Integration test (primary; harness at `monocle-runtime/tests/lock_file_lifecycle.rs`
— files in `<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM
Test Type column labels this BC `Integration`); mutation-test (auxiliary —
the `0o600` lock-file mode value, the `0o700` runtime-dir mode value
(defense-in-depth pairing per Post-condition 9 / BC-2.01.005 Postcondition
8), the `kill(pid, 0)` gate, and the 4-path resolution-chain ordering are
mutation surfaces). The harness uses `tempfile::TempDir` to isolate
`<runtime_dir>` per test AND mocks the `directories::ProjectDirs` API
(via dependency injection or `temp-env`-controlled env vars) to exercise
paths (a)-(d) deterministically.

## Pre-conditions

- Runtime directory `<runtime_dir>` is resolved per the 4-path chain. Tests
  use `tempfile::TempDir` to isolate `<runtime_dir>` per test AND mock the
  `directories::ProjectDirs` API (via dependency injection or
  `temp-env`-controlled env vars) to exercise paths (a)-(d)
  deterministically.
- `directories 6` (per SS-deps-pin-manifest.md v1.1.15) is the project pin
  for `ProjectDirs::runtime_dir()` and `ProjectDirs::data_local_dir()`.
- `tempfile 3` is the project pin (per SS-deps-pin-manifest.md v1.1.15).
- `nix 0.30` is the project pin (per SS-deps-pin-manifest.md v1.1.15) for
  the pid-liveness probe; the test asserts
  `nix::sys::signal::kill(Pid::from_raw(pid), None)` per BC-2.01.005
  postcondition 3.
- `temp-env ^0.3` is the project pin for `MONOCLE_RUNTIME_DIR` env
  isolation (shared with VP-ENGINE-002-ERR, see SS-03 VPs in Dispatch 5b).

## Post-conditions

1. Fresh start with no lock file (after successful runtime-dir resolution
   via any of paths a/b/c) → lock file created at
   `<resolved_runtime_dir>/monocle.lock`; `stat().mode() & 0o777 == 0o600`;
   JSON content begins with `{"contract_version":1,` (cross-property with
   VP-010).
2. Daemon already running (mock: PID file contains current test
   process PID, which is alive) → daemon start returns exit code 1;
   stderr contains the substring `daemon already running at pid=`.
3. Stale lock file (PID file contains `1` or another known-dead PID
   for the test environment, or contains a PID that `kill(0)` ESRCHes)
   → daemon start succeeds; the old file is replaced; the new file
   has the live daemon's PID.
4. Daemon graceful shutdown via synthetic SIGTERM → after drain
   completes, `<resolved_runtime_dir>/monocle.lock` does not exist
   (`Path::exists()` returns `false`). Cross-property with VP-004
   §Mechanical property item 5 (drain completion / lock-file lifecycle
   interaction).
5. **4-path resolution chain probe matrix (per PRD v1.25 §BC-DAEMON-005
   canonical test vectors EC-057/058/059; F-R70-1 closure):**

   | Probe | Setup | Expected resolution path | Expected log | Expected outcome |
   |-------|-------|---------------------------|--------------|------------------|
   | 5.a | `MONOCLE_RUNTIME_DIR=<temp_dir>` set; ProjectDirs mocked to either real OS values or `None` | (a) env override | `INFO: runtime_dir from MONOCLE_RUNTIME_DIR env var` | daemon uses `<temp_dir>` as runtime dir; lock file created there |
   | 5.b | `MONOCLE_RUNTIME_DIR` unset; ProjectDirs mocked so `runtime_dir()` returns `Some(<temp_dir>)` | (b) ProjectDirs::runtime_dir() | `INFO: runtime_dir from ProjectDirs::runtime_dir()` | daemon uses ProjectDirs `runtime_dir` path |
   | 5.c | `MONOCLE_RUNTIME_DIR` unset; ProjectDirs mocked so `runtime_dir()` returns `None` AND `data_local_dir()` returns `<temp_dir>` (EC-057 macOS pattern) | (c) ProjectDirs::data_local_dir() | `INFO: runtime_dir fallback to data_local_dir (platform: <os>)` | daemon uses `data_local_dir` path; happy-path on macOS; best-effort resolution on Windows per PRD §8.7 (Phase 1 CI does not formally validate Windows per NFR-008) |
   | 5.d | `MONOCLE_RUNTIME_DIR` unset; `ProjectDirs::new()` mocked to return `None` (EC-059 full-fail pattern) | (d) fail-fast `RuntimeDirUnresolvable` | (no `INFO` log; `ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path`) | daemon exits 1; NO lock file created at any path; cross-property with VP-004 post-condition 6 exit code `1` |

6. Daemon graceful shutdown → `<resolved_runtime_dir>/monocle.sock`
   does not exist (`Path::exists()` returns `false`).
7. Source-grep over `monocle-runtime/src/lock.rs`:
   - `tempfile::persist` appears at least once.
   - `std::fs::write` does NOT appear for the lock file path
     (an exception list may permit `std::fs::write` for non-lock paths,
     e.g., the recovery checkpoint file via separate path; the test
     restricts the negative match to lines mentioning `"monocle.lock"`).
8. Source-grep over `monocle-runtime/src/start.rs` (or wherever
   `resolve_runtime_dir` is implemented): the resolution chain order
   `MONOCLE_RUNTIME_DIR` → `runtime_dir()` → `data_local_dir()` →
   `RuntimeDirUnresolvable` is preserved (the chain MUST evaluate env
   override first; flipping the order would silently break the
   operator escape hatch).
9. **Runtime-dir mode 0o700 owner-only enforcement (per PRD v1.25
   §BC-DAEMON-005 Postcondition 8 + EC-052 + arch v1.0.25 §Start
   Sequence step 1; F-R75-1 VP-side closure + F-R79-3 PRD-side
   lift_invariants_to_bcs closure):** on runtime-dir creation
   (resolution chain paths b or c when the directory is absent prior to
   `monocle daemon start`), `stat(&runtime_dir).mode() & 0o777 == 0o700`
   (owner-only access). Verification: integration test creates a fresh,
   non-existent runtime_dir path, starts the daemon, then reads the
   directory's mode bits and asserts equality with `0o700`. Probe matrix
   row 5.e below covers this case directly. When the runtime_dir already
   exists from a prior start (idempotent restart path), the daemon MUST
   NOT modify the mode bits of the existing directory; the assertion
   applies only to the newly-created-this-start path. Cross-property
   with VP-008 (the auth token written into `<runtime_dir>/monocle.lock`
   is protected by both the lock-file 0o600 mode AND the containing
   directory's 0o700 mode — defense-in-depth).

## Counter-examples

1. Lock file written via naked `std::fs::write` — would expose a
   partial-write window between truncate and content-write; the
   source-grep negative assertion catches this. (This is also a
   semgrep rule per SS-conventions-anti-patterns.md §Semgrep Rules.)
2. Lock file written with mode `0o644` (group/other readable) — fails
   the `0o600` mode assertion; this is critical because the auth token
   is in the lock file and group/other readability would expose it to
   other OS users.
3. Stale-pid handling skipped (daemon refuses to start because lock
   file exists, without checking liveness) — fails post-condition 3.
4. Lock file not removed on clean shutdown — fails post-condition 4;
   subsequent starts would exercise the stale-pid path unnecessarily.
5. `tempfile::persist` argument `dest_path` set to a path that differs
   from the canonical `<resolved_runtime_dir>/monocle.lock` — fails the
   canonical-path assertion in post-condition 1.
6. **`ProjectDirs::runtime_dir() == None` on macOS triggers fail-fast
   without consulting `data_local_dir()`** — fails post-condition 5.c
   probe (the EC-057 macOS happy path); the daemon would refuse to
   start on the primary-target platform (NFR-008). This is the F-R70-1
   recurrence guard.
7. **Resolution-chain order flipped** (e.g., `ProjectDirs::runtime_dir()`
   evaluated before `MONOCLE_RUNTIME_DIR`) — silently breaks the
   operator escape hatch; an operator setting `MONOCLE_RUNTIME_DIR`
   would have their override ignored on Linux where `runtime_dir()`
   returns `Some`. Post-condition 8 source-grep assertion catches this.
8. **`DaemonStartError::RuntimeDirUnresolvable` raised when only path
   (b) returned `None`** (e.g., on macOS where `runtime_dir()` returns
   `None` but `data_local_dir()` returns `Some`) — fails the
   chain-coverage assertion; path (c) MUST be consulted before the
   fail-fast path (d) fires.
9. **`E-DAEMON-004 RuntimeDirUnresolvable` exit code other than `1`** —
   the fail-fast path MUST exit `1` (startup-failure code per
   VP-004 exit-code taxonomy item 6.5); a `0` or `143` would
   confuse monitoring tools. Cross-property assertion.
10. **Runtime dir created with umask-default mode (F-R75-1 attack
    surface):** implementer uses `std::fs::create_dir_all(&runtime_dir)?`,
    which creates the directory honoring the process umask (typical
    default ~0o022, yielding mode bits ~0o755 — world-readable). VP probe
    5.e fails because `stat(&runtime_dir).mode() & 0o777 != 0o700`.
    Information leak: other OS users can `stat` the runtime dir and
    enumerate monocle's paths (`/monocle.lock`, `/monocle.sock`,
    `/monocle.recovery.json`), aiding reconnaissance of an active
    daemon's token-bearing files (the lock file itself is 0o600, but
    the containing directory's readable mode reveals the path namespace).
    Correct approach: `std::os::unix::fs::DirBuilderExt` —
    `DirBuilder::new().mode(0o700).recursive(true).create(&runtime_dir)`.
    Cross-platform note: on Windows the `mode()` Unix API is not
    available; the Phase 1 0o700 contract is asserted on Linux/macOS
    primary targets per NFR-008 (Windows is a secondary build target
    per PRD §8.7; Phase 1 CI does not formally validate Windows mode
    bits). This is the F-R75-1 recurrence guard.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 5.a | `MONOCLE_RUNTIME_DIR=<temp>` set | path (a) env override; INFO log; lock file at `<temp>/monocle.lock` |
| 5.b | env unset; `ProjectDirs::runtime_dir()` → `Some(<temp>)` | path (b); INFO log; lock at runtime_dir path |
| 5.c | env unset; runtime_dir → `None`; data_local_dir → `Some(<temp>)` (EC-057 macOS) | path (c); INFO log; lock at data_local_dir path |
| 5.d | env unset; `ProjectDirs::new()` → `None` (EC-059) | path (d); ERROR log; exit 1; NO lock file created |
| 5.e | Runtime dir absent prior to start (path (b) or (c) creates it) | Daemon creates dir; mode bits = `0o700` |
| 5.f | Existing live PID lock | Daemon exit 1; stderr `daemon already running at pid=` |
| 5.g | Stale PID lock (ESRCH on `kill(pid, 0)`) | Daemon proceeds; WARN log `stale lock file removed`; new lock written |
| 5.h | Clean shutdown via synthetic SIGTERM | `monocle.lock` AND `monocle.sock` removed (`Path::exists()` returns false) |
| 5.i | Source-grep: `tempfile::persist` present + no `std::fs::write` on `monocle.lock` line | structural assertions pass |
| 5.j | Source-grep: resolution-chain ordering preserved | chain order intact (env → runtime_dir → data_local_dir → fail-fast) |

**Mutation-test rationale:** the `0o600` lock-file mode literal, the
`0o700` runtime-dir mode literal, the `kill(pid, 0)` syscall result
check, AND the resolution-chain ordering (env-first → runtime_dir →
data_local_dir → fail-fast) are prime mutation targets. `cargo-mutants`
will attempt to mutate the lock-file mode to `0o644` (passing
functional tests that don't check mode), to mutate the runtime-dir
mode to `0o755` (umask-default leak — F-R75-1 surface), to flip the
`kill` result interpretation, and to reorder the resolution-chain
conditionals; all must be caught.

## Harness Location

- `monocle-runtime/tests/lock_file_lifecycle.rs` (integration)
- Test name: `test_BC_DAEMON_005_lock_file_create_and_cleanup` (per PRD
  v1.25 §BC-DAEMON-005, Verification subsection — covers the lock-file
  mode/lifecycle assertions AND the EC-057/058/059 resolution-chain
  probes via the canonical test-vector matrix; to be migrated to
  `test_BC_2_01_005_lock_file_create_and_cleanup`).

## References

- Current as of `2026-05-17T13:00:00Z` (Dispatch 5a).
- Predecessor: monolithic VP-DAEMON-005 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; to be retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-01/BC-2.01.005.md`.
- Architecture: `architecture/SS-daemon-lifecycle.md` v1.0.25 (commit 18fe265).
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.01.005 (Dispatch 4 commit 1030c65).
- Dependency pins: `architecture/SS-deps-pin-manifest.md` v1.1.15.
- Cross-property: VP-004 (drain completion → lock removal; exit-code
  taxonomy with `1` for `RuntimeDirUnresolvable`), VP-008 (defense-in-depth
  with auth-token), VP-010 (`contract_version` JSON-content joint
  assertion).
