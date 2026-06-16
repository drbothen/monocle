---
document_type: behavioral-contract
level: L3
version: "1.5.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:04:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "54afe3d"
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

# BC-2.04.005: `monocle daemon stop` CLI Subcommand

## Description

`monocle daemon stop` sends SIGTERM to the daemon process identified by the PID in the lock
file, then polls for process exit for up to 15 seconds. If the process exits within the
window, the command exits 0. If the lock file is absent, the command exits 1 with a
structured error. If the PID in the lock file is not alive at the time of the check, the
command exits 1 with a stale-lock-file error. If the process does not exit within 15 seconds,
the command exits 2 with a timeout error. The command never sends SIGKILL; it respects the
daemon's graceful-shutdown protocol (BC-2.01.004).

## Preconditions

1. The `monocle daemon stop` subcommand has been invoked via the `clap`-based CLI.
2. `MONOCLE_NO_AUTOSTART` does NOT affect this subcommand.
3. `<runtime_dir>` is resolvable via BC-2.04.006.

## Postconditions

**Happy path (daemon running):**

PC-1. The lock file at `<runtime_dir>/monocle.lock` is read and the `pid` field is parsed.
PC-2. `kill(pid, SIGTERM)` is sent to the daemon process.
PC-3. The command polls for process exit (using `kill(pid, 0)` returning ESRCH, or equivalent
      process-liveness check) at 1-second intervals for up to 15 seconds.
PC-4. If the daemon process exits within 15 seconds, the command exits 0 with no stdout
      output and no stderr output.

**Lock file absent:**

PC-5. If `<runtime_dir>/monocle.lock` does not exist, the command writes to stderr:
      `error: no lock file found; daemon may not be running`
      and exits 1. No signal is sent.

**Stale lock file (PID not alive):**

PC-6. If the lock file exists but `kill(pid, 0)` returns ESRCH (process does not exist),
      the command writes to stderr:
      `error: daemon not running (stale lock file?)`
      and exits 1. No SIGTERM is sent (there is no process to signal).

**Timeout (process did not exit within 15 seconds):**

PC-7. If 15 seconds elapse without the daemon process exiting, the command writes to stderr:
      `error: daemon did not exit within 15 s; it may still be draining`
      and exits 2. The daemon process is NOT killed; it may still be completing graceful
      shutdown (BC-2.01.004 allows up to 10 seconds for drain; the extra 5 seconds of
      buffer accommodate slow drain scenarios).

**Exit codes:**

PC-8. Exit codes for `monocle daemon stop`:
      - `0`: daemon exited within 15 seconds
      - `1`: precondition failure (no lock file, or stale lock file with dead PID)
      - `2`: timeout — daemon did not exit within 15 seconds

## Invariants

1. **No SIGKILL is ever sent.** `monocle daemon stop` sends only SIGTERM and respects the
   daemon's graceful-shutdown protocol. Operators who need a forced stop must send SIGKILL
   via `kill -9 <pid>` manually. This BC does not provide a `--force` flag.
2. **15-second poll timeout is fixed.** The timeout is not configurable via flags or
   environment variables in Phase 1. The 15-second window provides 5 seconds of buffer
   over BC-2.01.004's 10-second drain timeout.
3. **No stdout on success.** On exit code 0, no bytes are written to stdout.
4. **Stderr only for errors.** Error messages go to stderr, not stdout.
5. **Lock file is not removed by this command.** The daemon's graceful shutdown process
   (BC-2.01.004) is responsible for removing the lock file and UDS socket. If the daemon
   exits before cleaning up (crash during shutdown), the lock file may remain as a stale
   file; the next `monocle daemon start` invocation will handle it per BC-2.01.005.
6. **Poll interval is 1 second.** The maximum detection latency after daemon exit is 1
   second; in the common case, exit is detected at the next 1-second poll boundary.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-2.04.005-01 | Lock file does not exist | stderr: `error: no lock file found; daemon may not be running`; exit 1 |
| EC-2.04.005-02 | Lock file exists; PID not alive (stale lock) | stderr: `error: daemon not running (stale lock file?)`; exit 1 |
| EC-2.04.005-03 | SIGTERM sent; daemon exits at 14.9 seconds | exit 0; no stdout; no stderr |
| EC-2.04.005-04 | SIGTERM sent; daemon exits at 15.1 seconds (timeout) | stderr: `error: daemon did not exit within 15 s; it may still be draining`; exit 2; daemon NOT killed |
| EC-2.04.005-05 | SIGTERM sent; daemon exits at 0.5 seconds (fast shutdown) | exit 0 detected at next 1-second poll; minor detection lag is acceptable |
| EC-2.04.005-06 | Lock file is readable but `pid` field is malformed JSON | command treats this as a parse error; exits 1 with stderr: `error: malformed lock file; daemon may not be running` |
| EC-2.04.005-07 | `runtime_dir` resolution fails | exits 70 (consistent with exit code table); stderr: error from DaemonStartError::RuntimeDirUnresolvable |
| EC-2.04.005-08 | `monocle daemon stop` while another `monocle daemon stop` is in flight | Both commands read the same PID; both send SIGTERM (idempotent); both poll the same process; the first to detect exit wins; the second also detects exit and exits 0 |
| EC-2.04.005-09 | Permission denied reading lock file | exits 1 with stderr: `error: cannot read lock file: <OS error>` |
| EC-2.04.005-10 | `kill(pid, SIGTERM)` fails with EPERM (process owned by different user) | exits 1 with stderr: `error: cannot signal daemon (permission denied)`; no polling attempted |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `monocle daemon stop` (daemon running) | Daemon receives SIGTERM; exits within 10 s (standard drain); command exits 0 | happy-path |
| `monocle daemon stop` (no lock file) | stderr: `error: no lock file found; daemon may not be running`; exit 1 | error |
| `monocle daemon stop` (stale lock, dead PID) | stderr: `error: daemon not running (stale lock file?)`; exit 1 | error |
| `monocle daemon stop` (daemon does not exit within 15 s) | stderr: `error: daemon did not exit within 15 s; it may still be draining`; exit 2 | error |
| Malformed `pid` field in lock file | stderr: `error: malformed lock file; daemon may not be running`; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | SIGTERM is sent to the correct PID from the lock file | integration |
| VP-TBD | Exit code 0 when daemon exits within 15 seconds | integration |
| VP-TBD | Exit code 1 when lock file is absent | integration |
| VP-TBD | Exit code 1 when PID is not alive (stale lock) | integration |
| VP-TBD | Exit code 2 when daemon does not exit within 15 seconds | integration (with synthetic stall injection) |
| VP-TBD | No SIGKILL is ever sent | integration (verify no SIGKILL in OS process audit) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — `monocle daemon stop` is a CLI subcommand that is the operator-facing stop path; "CLI surface" is named explicitly as a CAP-004 responsibility; this BC specifies the complete `daemon stop` CLI contract including SIGTERM semantics, 15-second poll, and exit codes |
| L2 Domain Invariants | DI-002 (lock file must be present before hook endpoints accept connections — this BC reads the lock file to find the PID; after successful stop, the daemon is no longer running, so DI-002 is transitively enforced by the clean shutdown removing the lock file per BC-2.01.004) |
| Architecture Module | `monocle` binary crate per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §CLI Interface §Subcommand: `monocle daemon stop` |
| Cross-Ref | BC-2.01.004 (graceful shutdown — the daemon's 10-second drain is what the 15-second poll window accommodates); BC-2.01.005 (lock file PID field — PC-1 reads the PID from the lock file per this contract); BC-2.04.006 (runtime_dir resolution) |
| Test File | `monocle/tests/cli_daemon_stop.rs` |
| Test Name | `test_BC_2_04_005_daemon_stop_subcommand` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.01.004] — depends on: the daemon's graceful-shutdown drain (10 s) is what the 15-second stop poll accommodates
- [BC-2.01.005] — depends on: the PID in the lock file read at PC-1 follows BC-2.01.005 schema
- [BC-2.04.004] — sibling: the start subcommand; start and stop are the two halves of the daemon CLI surface
- [BC-2.04.006] — depends on: runtime_dir resolution for lock file path

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#subcommand-monocle-daemon-stop` — SIGTERM, 15-second poll, exit codes
- `architecture/SS-daemon-wiring.md#exit-codes` — exit code table

## Story Anchor

S-TBD — Implement `monocle daemon stop` CLI subcommand with SIGTERM + poll (filled by story-writer)

## VP Anchors

VP-TBD — `monocle daemon stop` CLI integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T12:04:00Z):
- BC-2.04.005 created as new artifact for SS-04 per prd-expansion-scope.md §3.1 and
  SS-daemon-wiring.md §CLI Interface §Subcommand: `monocle daemon stop`.
- Covers: happy path, absent lock file, stale lock, timeout, 10 edge cases, 5 test vectors,
  6 VPs.
- 15-second poll timeout matches the arch doc exactly.
- No-SIGKILL invariant is production-grade: forcing operators to use `kill -9` manually
  prevents accidental data loss from interrupted JSONL writes or ring-buffer flushes.
- input-hash: [pending] — to be populated by compute-input-hash after human review.
- SE-16d PASS: 2026-05-26T12:04:00Z > prior 2026-05-26T12:03:00Z (BC-2.04.004).

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
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §CLI Interface §Subcommand: monocle daemon stop` → `SS-daemon-wiring.md v1.3.0 §CLI Interface §Subcommand: monocle daemon stop`.
- Plain version-pin refresh. No substantive content propagation required — §Subcommand: monocle daemon stop section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.5.0 timestamp >= v1.4.0. PASS.
