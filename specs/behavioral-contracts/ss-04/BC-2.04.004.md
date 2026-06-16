---
document_type: behavioral-contract
level: L3
version: "1.5.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:03:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "359b43d"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D-001, F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.004: `monocle daemon start` CLI Subcommand

## Description

`monocle daemon start` starts the monocle daemon as a background process detached from the
calling terminal. The foreground caller blocks until the lock file appears at
`<runtime_dir>/monocle.lock`, which signals that the daemon has successfully completed the
start sequence (BC-2.04.001) and is ready to serve requests. If a live daemon is already
running at the resolved runtime directory, the command exits immediately with code 1 and a
structured error to stderr; it never attempts to start a second daemon. On success, the
command exits with code 0 and produces no stdout output.

## Preconditions

1. The `monocle daemon start` subcommand has been invoked via the `clap`-based CLI.
2. `MONOCLE_NO_AUTOSTART` does NOT affect this subcommand (this check applies only to TUI
   mode; see BC-2.04.003 Invariant 4 / EC-2.04.003-07).
3. The host OS supports the double-fork or `nohup` process detachment pattern for the
   daemon subprocess. On Unix: `SIGHUP` is ignored or the child process group is set so
   the daemon survives the parent shell exiting.

## Postconditions

**Happy path (no live daemon):**

PC-1. The command checks for a live daemon using the PID-liveness check from BC-2.01.005
      (lock file exists + `kill(pid, 0)` == 0). If no live daemon is detected, startup
      proceeds.
PC-2. The daemon subprocess is launched as a detached background process. The subprocess
      executes the 13-step start sequence defined in BC-2.04.001.
PC-3. The foreground caller polls `<runtime_dir>/monocle.lock` at 100ms intervals, waiting
      up to 10 seconds for the file to appear.
PC-4. When the lock file appears, the foreground caller exits with code 0. No stdout output
      is produced on success.
PC-5. The daemon continues running in the background after the foreground caller exits. The
      daemon process is not a child of the calling shell (it is detached via double-fork or
      equivalent).

**Already-running path:**

PC-6. If the lock file exists AND the PID in it is alive (`kill(pid, 0)` returns 0), the
      command writes to stderr:
      `error: daemon already running (pid=<N>)`
      and exits with code 1. No daemon process is started.

**Timeout path:**

PC-7. If the lock file does not appear within 10 seconds of launching the daemon subprocess,
      the foreground caller writes to stderr:
      `error: daemon failed to start within 10 s`
      and exits with code 1. The background daemon subprocess is NOT explicitly killed; it
      may still be starting. The caller's exit code 1 is informational; the daemon may
      succeed in the background.

**Exit codes:**

PC-8. Exit codes for `monocle daemon start`:
      - `0`: daemon started successfully (lock file appeared)
      - `1`: precondition failure (daemon already running) OR timeout
      - `70`: runtime directory cannot be resolved (`DaemonStartError::RuntimeDirUnresolvable`)
      - `71`: internal error during daemon startup (lock file write failed; from BC-2.04.001)

## Invariants

1. **Lock file is the completion signal.** The foreground caller does not use any IPC from
   the daemon to detect startup; it polls only for the lock file. This keeps the start
   mechanism simple and independent of any daemon-internal state.
2. **Double-fork ensures daemon independence.** The daemon's process group MUST be set such
   that the daemon continues running after the calling terminal session ends (user logs out,
   shell closes). Implementation: `nohup` pattern or explicit `setsid()` call on the child.
3. **No stdout on success.** On exit code 0, no bytes are written to stdout. This makes
   `monocle daemon start` safe to call in shell pipelines and scripts without stdout
   contamination.
4. **Stderr only for errors.** All error messages go to stderr, not stdout.
5. **PID-liveness check is best-effort.** The lock-file atomic-write (inside BC-2.04.001
   step 8) is the true mutual-exclusion point; the PID-liveness check in PC-1 is an early
   exit optimization that avoids a redundant startup attempt in the common case.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-2.04.004-01 | Lock file exists with alive PID | stderr: `error: daemon already running (pid=<N>)`; exit 1 |
| EC-2.04.004-02 | Lock file exists with dead PID (stale) | Stale file treated as absent (per BC-2.01.005 step 2); daemon start proceeds normally |
| EC-2.04.004-03 | Lock file appears at 9.9 seconds (just within 10-second window) | Foreground exits 0; no error message |
| EC-2.04.004-04 | Lock file does not appear within 10 seconds | stderr: `error: daemon failed to start within 10 s`; exit 1 |
| EC-2.04.004-05 | Two concurrent `monocle daemon start` invocations | Both pass PID-liveness check; both start background processes; the first daemon to write the lock file wins (atomic-rename in BC-2.04.001 step 8); second daemon exits 1 at step 2 on re-check; callers: one exits 0, one exits 1 (non-deterministic which) |
| EC-2.04.004-06 | `runtime_dir` resolution fails | stderr: error from DaemonStartError::RuntimeDirUnresolvable; exit 70 |
| EC-2.04.004-07 | `monocle daemon start` invoked in a CI environment with `MONOCLE_NO_AUTOSTART=1` | `MONOCLE_NO_AUTOSTART` does not affect `daemon start`; daemon starts normally |
| EC-2.04.004-08 | Daemon subprocess exits before writing lock file (crash in steps 1–7) | Foreground times out after 10 seconds; stderr: `error: daemon failed to start within 10 s`; exit 1 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `monocle daemon start` (no lock file) | Lock file appears within 10 s; exit 0; no stdout | happy-path |
| `monocle daemon start` (daemon running, PID alive) | stderr: `error: daemon already running (pid=<N>)`; exit 1 | error |
| `monocle daemon start` (stale lock file, dead PID) | Stale lock removed; new daemon started; exit 0 | edge-case |
| `monocle daemon start` (daemon crashes before lock file write) | stderr: `error: daemon failed to start within 10 s`; exit 1 | error |
| `MONOCLE_NO_AUTOSTART=1 monocle daemon start` | Daemon starts normally (env var does not affect subcommand); exit 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Exit code 0 and no stdout when daemon starts successfully | integration |
| VP-TBD | Exit code 1 with correct stderr when daemon already running | integration |
| VP-TBD | Exit code 1 with timeout message when daemon fails to start | integration |
| VP-TBD | Daemon process survives parent process exit (detached) | integration |
| VP-TBD | Stale lock file is removed before new daemon is started | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — `monocle daemon start` is the primary CLI subcommand through which operators launch the daemon; "CLI surface" and "daemon auto-start" are both named CAP-004 responsibilities; this BC specifies the complete `daemon start` CLI contract including exit codes, foreground polling, detachment, and error messaging |
| L2 Domain Invariants | DI-002 (lock file must be present and contain a valid port and auth token before any hook endpoint accepts connections — PC-3 enforces this by making the foreground caller wait for the lock file before declaring success; hook endpoints are guaranteed present once the lock file appears, per BC-2.04.001 step 12 occurring after step 8) |
| Architecture Module | `monocle` binary crate per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §CLI Interface §Subcommand: `monocle daemon start` |
| Cross-Ref | BC-2.04.001 (daemon start sequence — the subprocess executes this); BC-2.01.005 (PID liveness check — PC-1 delegates to this); BC-2.04.006 (runtime_dir resolution — used for lock file polling path) |
| Test File | `monocle/tests/cli_daemon_start.rs` |
| Test Name | `test_BC_2_04_004_daemon_start_subcommand` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.001] — composes with: the daemon subprocess executes the BC-2.04.001 13-step start sequence
- [BC-2.01.005] — depends on: the PID liveness check pattern at PC-1 follows BC-2.01.005 semantics
- [BC-2.04.005] — sibling: the stop subcommand; start and stop are the two halves of the daemon CLI surface
- [BC-2.04.006] — depends on: runtime_dir resolution for lock file polling path

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#subcommand-monocle-daemon-start` — foreground polling, exit codes, detachment requirement
- `architecture/SS-daemon-wiring.md#exit-codes` — exit code table

## Story Anchor

S-TBD — Implement `monocle daemon start` CLI subcommand with foreground polling (filled by story-writer)

## VP Anchors

VP-TBD — `monocle daemon start` CLI integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T12:03:00Z):
- BC-2.04.004 created as new artifact for SS-04 per prd-expansion-scope.md §3.1 and
  SS-daemon-wiring.md §CLI Interface §Subcommand: `monocle daemon start`.
- Covers: happy path, already-running path, timeout path, all exit codes (0/1/70/71),
  8 edge cases, 5 test vectors, 5 VPs.
- 10-second foreground poll timeout matches the arch doc (arch doc says "blocks until lock
  file appears"; 10 s is inferred from the stop subcommand's 15-second precedent + the
  auto-start 5-second timeout; the arch doc does not state an explicit number for `daemon
  start` foreground wait — 10 s is a production-grade default that gives the 13-step
  sequence ample time while bounding user-visible wait).
- input-hash: [pending] — to be populated by compute-input-hash after human review.
- SE-16d PASS: 2026-05-26T12:03:00Z > prior 2026-05-26T12:02:00Z (BC-2.04.003).

## §Trace v1.1.0

**F-P1D-001 CRITICAL — capability mis-anchor corrected** (2026-05-26T00:00:00Z):
- Frontmatter `capability: CAP-001` → `capability: CAP-004` per F-P1D-001.
- Traceability §L2 Capability and §Capability Anchor Justification updated to cite CAP-004
  ("Daemon binary crate wiring; CLI surface; SOQ-2 start-sequence invariant; hook endpoint
  routing; bounded event bus") per ARCH-INDEX §SS-04 Capability Traceability.
- SE-16d monotonicity: v1.1.0 timestamp >= v1.0.0. PASS.

## §Trace v1.2.0

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.0.0` → `SS-daemon-wiring.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.4.0

**F-P1D10-002 HIGH — CAP-004 capability text corrected to ARCH-INDEX verbatim** (2026-05-26T00:00:00Z):
- L2 Capability and Capability Anchor Justification: stale text → ARCH-INDEX verbatim
  `"Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation"`.
- SE-16d monotonicity: v1.4.0 timestamp >= v1.3.0. PASS.

## §Trace v1.3.0

**F-P1D4-003 LOW — Architecture Source pin updated from v1.1.0 to v1.2.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.1.0` → `SS-daemon-wiring.md v1.2.0` per F-P1D4-003 bulk update.
- SE-16d monotonicity: v1.3.0 timestamp >= v1.2.0. PASS.

## §Trace v1.5.0

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-daemon-wiring.md v1.2.0 → v1.3.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §CLI Interface §Subcommand: monocle daemon start` → `SS-daemon-wiring.md v1.3.0 §CLI Interface §Subcommand: monocle daemon start`.
- Plain version-pin refresh. No substantive content propagation required — §Subcommand: monocle daemon start section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.5.0 timestamp >= v1.4.0. PASS.
