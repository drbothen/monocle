---
document_type: behavioral-contract
level: L3
version: "1.3.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T04:00:00Z
phase: phase-1-expansion
inputs: [prd-expansion-scope.md, architecture/SS-ipc.md, architecture/ARCH-INDEX.md]
input-hash: "73990b1"
traces_to: prd.md
origin: greenfield
subsystem: SS-05
capability: CAP-005
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.05.001: UDS Server Bind at runtimeDir/monocle.sock

## Description

The monocle daemon creates a Unix domain socket at `<runtime_dir>/monocle.sock` as part of
its startup sequence (step 10 of the daemon start sequence defined in SS-daemon-wiring.md).
The socket is created with mode `0o600` (owner-only access). If a stale socket file exists
from a prior crashed daemon, it is removed before rebind. On graceful shutdown, the socket
file is removed alongside `monocle.lock` and `hooks-settings.json`.

## Preconditions

1. The daemon has completed steps 1–9 of the start sequence: the lock file has been written
   and the HTTP hook receiver is accepting connections.
2. The `runtime_dir` has been resolved via the platform-aware fallback chain (same resolution
   as the lock file in BC-2.01.005 Precondition 2).
3. The daemon process has the necessary filesystem permissions to create files under
   `runtime_dir` (which was created with mode `0o700` per BC-2.01.005 Postcondition 8).

## Postconditions

1. The daemon calls `UnixListener::bind("<runtime_dir>/monocle.sock")` and begins accepting
   client connections.
2. The socket file at `<runtime_dir>/monocle.sock` has mode `0o600` (owner-readable and
   owner-writable only; no group or world access).
3. If a file already exists at `<runtime_dir>/monocle.sock` at the time of bind (stale socket
   from a prior crashed daemon), the daemon removes it before calling `UnixListener::bind`.
   This mirrors the stale lock-file removal at step 2 of the start sequence.
4. On graceful shutdown, the daemon removes `<runtime_dir>/monocle.sock`. The removal happens
   in the same cleanup sequence as `monocle.lock` removal.
5. If `UnixListener::bind` fails after the stale-file removal (e.g., permissions error, path
   too long), the daemon logs `ERROR: failed to bind UDS socket at <path>: <reason>` and exits
   with code 1 before accepting any TUI connections.

## Invariants

1. The UDS socket is always created after the lock file is written (SOQ-2 ordering). A TUI
   client that reads a valid lock file will always find a socket ready to accept within the
   daemon start-sequence completion window (≤5 seconds per SS-daemon-wiring.md §Daemon
   Auto-Start Logic).
2. Only the owning user can connect to the socket (mode `0o600`). A process running as a
   different OS user cannot inject IPC messages or intercept permission decisions.
3. No stale socket file from a prior daemon crash is ever left in place after a new daemon
   successfully starts. The stale-file removal + rebind is atomic with respect to new daemon
   start (the prior daemon's acceptor is already gone before rebind).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Stale socket file exists from a prior crashed daemon | Daemon removes the stale socket file, then calls `UnixListener::bind`. Logs `WARN: removed stale UDS socket at <path>`. |
| EC-002 | `UnixListener::bind` fails because the path is too long (POSIX UDS path limit is 104-108 bytes depending on OS) | Daemon logs `ERROR: UDS socket path exceeds OS limit (<N> bytes, limit <M>)` and exits 1. No TUI connections accepted. |
| EC-003 | `runtime_dir` contains spaces or special characters | Daemon passes the path verbatim as a `Path` (not a shell string); no quoting or escaping issues. UDS path is constructed via `Path::new(runtime_dir).join("monocle.sock")`. |
| EC-004 | Daemon crashes without running the shutdown handler (SIGKILL, OOM) | Socket file remains on disk as a stale artifact. Next daemon start will detect it (EC-001) and remove it before rebind. |
| EC-005 | TUI connects before the daemon has finished binding (race during daemon start) | TUI waits up to 5 seconds for the lock file to appear (SS-daemon-wiring.md §Daemon Auto-Start Logic). After the lock file appears, the socket is guaranteed to exist (Postcondition 1 ordering). |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Daemon starts cleanly (no prior socket file) | Socket created at `<runtime_dir>/monocle.sock` with mode 0o600; accepts connections | happy-path |
| Stale socket file exists at `<runtime_dir>/monocle.sock` before bind | WARN logged; stale file removed; socket rebound; mode 0o600 | edge-case |
| Daemon shuts down gracefully (SIGTERM + drain completes) | Socket file removed from `runtime_dir` | happy-path |
| Daemon killed with SIGKILL (no shutdown handler runs) | Socket file remains; next daemon start removes it and rebinds | edge-case |
| `UnixListener::bind` fails (filesystem full) | `ERROR: failed to bind UDS socket` logged; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | UDS socket created at `<runtime_dir>/monocle.sock` with mode 0o600 on daemon start | integration |
| VP-TBD | Stale socket removed before rebind; WARN logged | integration |
| VP-TBD | Socket file removed on graceful shutdown | integration |
| VP-TBD | Daemon exits 1 and logs error when `UnixListener::bind` fails | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability §SS-05 — this BC governs the UDS socket bind lifecycle that establishes the transport foundation on which all TUI-to-daemon communication depends |
| L2 Domain Invariants | DI-002 (the lock file must be present and contain a valid port and auth token before any hook endpoint accepts connections — this BC extends that invariant to the UDS socket: the socket is bound after the lock file is written, ensuring ordered initialization); DI-003 (auth token written to lock file after port is bound — the socket bind happens after lock file write, preserving SOQ-2 ordering) |
| Architecture Module | monocle-ipc (UDS server bind), monocle-runtime (daemon start sequence step 10) per ARCH-INDEX Subsystem Registry SS-05 and SS-04 |
| Architecture Source | SS-ipc.md v1.23.0 §Transport Layer §Lifecycle; SS-daemon-wiring.md v1.3.0 §Daemon Start Sequence |
| Cross-Ref | BC-2.01.005 (lock file atomic lifecycle — socket bind happens after lock file write per SOQ-2); BC-2.05.002 (TUI connects to this socket) |
| Test File | `monocle-ipc/tests/uds_bind_lifecycle.rs` |
| Test Name | `test_BC_2_05_001_uds_bind_and_cleanup` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.01.005] — depends on: lock file created before socket bind (SOQ-2 ordering invariant)
- [BC-2.05.002] — composes with: TUI client connects to the socket this BC creates
- [BC-2.04.001] — depends on: daemon start sequence step 10 (this BC) follows steps 1–9

## Architecture Anchors

- `architecture/SS-ipc.md#transport-layer` — socket path, mode, stale-file removal, lifecycle
- `architecture/SS-daemon-wiring.md#daemon-start-sequence` — step 10: UDS bind follows lock file write

## Story Anchor

S-TBD — Implement UDS server bind at runtimeDir/monocle.sock with mode 0o600 and stale-file removal (filled by story-writer)

## VP Anchors

VP-TBD — UDS socket bind lifecycle verification properties (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T04:00:00Z):
- BC-2.05.001 authored for SS-05 IPC subsystem per `prd-expansion-scope.md §3.2` and
  `SS-ipc.md §Transport Layer §Lifecycle`.
- Covers: UDS socket bind at `<runtime_dir>/monocle.sock`, mode `0o600`, stale-file removal,
  graceful shutdown cleanup, bind-failure exit path.
- SOQ-2 ordering invariant cited: socket bind follows lock file write (Invariant 1).
- 5 edge cases documented (EC-001..EC-005).
- SE-16d PASS: 2026-05-26T04:00:00Z > chain high-water 2026-05-26T03:00:00Z (SS-06 registration).


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.0.0` → `SS-ipc.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.1.0

**F-P1D4-007 HIGH — Architecture Source pins updated; SS-daemon-wiring.md version pin added** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.1.0` → `SS-ipc.md v1.3.0` per F-P1D4-004 bulk update.
- Architecture Source: `SS-daemon-wiring.md §Daemon Start Sequence` → `SS-daemon-wiring.md v1.2.0 §Daemon Start Sequence` — version pin was missing (F-P1D4-007 HIGH finding); pin added.
- SE-16d monotonicity: v1.1.0 timestamp >= v1.0.1. PASS.

## §Trace v1.2.0

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.3.0` → `SS-ipc.md v1.4.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.2.0 timestamp >= v1.1.0. PASS.

## §Trace v1.3.0

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: dual Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row 1: `SS-ipc.md v1.4.0 §Transport Layer §Lifecycle` → `SS-ipc.md v1.9.0 §Transport Layer §Lifecycle`. Plain version-pin refresh — §Transport Layer §Lifecycle section heading and content anchors unchanged between v1.4.0 and v1.9.0.
- Architecture Source row 2: `SS-daemon-wiring.md v1.2.0 §Daemon Start Sequence` → `SS-daemon-wiring.md v1.3.0 §Daemon Start Sequence`. Plain version-pin refresh — §Daemon Start Sequence section heading and content anchors unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.3.0 timestamp >= v1.2.0. PASS.
