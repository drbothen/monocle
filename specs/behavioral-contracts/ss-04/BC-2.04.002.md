---
document_type: behavioral-contract
level: L3
version: "1.5.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:01:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/ARCH-INDEX.md]
input-hash: "130600f"
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

# BC-2.04.002: Daemon Auto-Start on TUI Launch

## Description

When `monocle` is invoked without a subcommand (TUI mode) and `MONOCLE_NO_AUTOSTART` is not
set, the binary automatically starts a daemon subprocess before rendering any TUI output. The
auto-start path checks daemon liveness via the lock file and PID check, starts the daemon if
absent, waits up to 5 seconds for the lock file to appear, and then connects the TUI to the
daemon. The TUI MUST NOT render its main content before a liveness verdict is reached; if the
daemon cannot be started, the TUI renders in "daemon unavailable — offline mode" state.

## Preconditions

1. `monocle` is invoked without any subcommand (TUI mode).
2. The `MONOCLE_NO_AUTOSTART` environment variable is either unset or empty. (If it is set to
   any non-empty value, BC-2.04.003 applies instead and this BC does not execute.)
3. `<runtime_dir>` is resolvable via the chain in BC-2.04.006.
4. The current executable path is obtainable via `std::env::current_exe()`.

## Postconditions

**Decision sequence (steps 1–5 execute in order):**

PC-1. `<runtime_dir>` is resolved via BC-2.04.006. If resolution fails, the process exits
      with code 70 before rendering any TUI output.

PC-2. The process checks for `<runtime_dir>/monocle.lock`. If the lock file does not exist,
      proceed to PC-4.

PC-3. If the lock file exists, the `pid` field is parsed and `kill(pid, 0)` is called.
      - If the process is alive (`kill` returns 0): the daemon is running. Proceed directly
        to PC-5 (TUI connection); do NOT start another daemon.
      - If the process is dead (`kill` returns ESRCH): log `WARN: stale lock file removed`,
        remove the lock file, and proceed to PC-4.

PC-4. A daemon subprocess is started equivalent to `monocle daemon start`. The implementation
      MAY call the daemon start function directly in-process rather than exec-ing a new
      process. The auto-start waits up to 5 seconds for `<runtime_dir>/monocle.lock` to
      appear (polling at 100ms intervals).
      - If the lock file appears within 5 seconds: proceed to PC-5.
      - If the lock file does not appear within 5 seconds: the TUI renders the status-bar
        message `daemon start timed out; retrying…` and retries once (another 5-second wait).
      - If the retry also fails: the TUI renders `daemon unavailable — running in offline mode`
        and continues without a daemon connection. The TUI MUST remain functional in offline
        mode (observe-only restrictions apply; no permission overlay dispatch).

PC-5. The TUI connects to the daemon via the UDS at `<runtime_dir>/monocle.sock`
      (IPC protocol specified in SS-05 / BC-2.05.001..BC-2.05.002). The daemon PID MUST
      pass a liveness check (`kill(pid, 0)` returns 0) before the TUI attempts the UDS
      connection.

PC-6. The TUI renders its main content only after PC-5 succeeds or after offline-mode
      determination at PC-4.

## Invariants

1. No TUI main content is rendered before the auto-start decision sequence completes.
2. Stale lock files (dead PID) are removed before a new daemon is started; a stale file is
   never left to interfere with subsequent startup.
3. The retry path (one retry after 5-second timeout) means the maximum wait before
   offline-mode fallback is 10 seconds.
4. Auto-start does NOT double-fork in the in-process variant; if the daemon start function
   is called directly, the daemon runs as a task within the same tokio runtime until
   `monocle daemon start` semantics separate it (implementation detail; the observable
   behavior is that the lock file appears within 5 seconds).
5. The daemon PID liveness check MUST succeed before the TUI sends any IPC message to the
   UDS socket. A socket that exists with no live daemon must not be connected to.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-2.04.002-01 | Lock file exists with alive PID on first check | No new daemon started; TUI connects immediately to existing daemon (PC-5) |
| EC-2.04.002-02 | Lock file exists with dead PID (stale lock) | WARN logged, stale file removed, new daemon started (PC-3 → PC-4) |
| EC-2.04.002-03 | Daemon start succeeds but lock file appears only at second 4.9 | Auto-start succeeds; TUI connects normally; no "timed out" message |
| EC-2.04.002-04 | First 5-second wait times out; retry succeeds within the second 5-second window | TUI shows `daemon start timed out; retrying…` then connects normally after retry |
| EC-2.04.002-05 | Both 5-second waits time out | TUI renders `daemon unavailable — running in offline mode`; TUI remains functional without daemon connection |
| EC-2.04.002-06 | `runtime_dir` resolution fails | Process exits with code 70 before rendering any TUI output |
| EC-2.04.002-07 | `current_exe()` returns an error | Daemon start falls back to invoking `monocle daemon start` by searching PATH; if PATH resolution also fails, offline mode is entered with an ERROR log |
| EC-2.04.002-08 | UDS socket exists but daemon is not alive (crashed between PC-4 and PC-5) | PID liveness check at PC-5 fails; offline mode entered; WARN logged |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `monocle` invoked, no lock file, `MONOCLE_NO_AUTOSTART` unset | Daemon started, lock file appears within 5 s, TUI renders main content | happy-path |
| `monocle` invoked, lock file with alive PID | No new daemon; TUI connects to existing daemon | happy-path |
| `monocle` invoked, lock file with dead PID | Stale lock removed; new daemon started; WARN logged | edge-case |
| Daemon start times out twice | `daemon unavailable — running in offline mode` rendered | error |
| `MONOCLE_NO_AUTOSTART=` (empty string) | Treated as unset; auto-start executes normally (per BC-2.04.003 Invariant 1) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | TUI does not render main content before liveness verdict | integration |
| VP-TBD | Stale lock file (dead PID) is removed before new daemon started | integration |
| VP-TBD | Offline-mode renders after two consecutive 5-second timeouts | integration (with synthetic timeout injection) |
| VP-TBD | Daemon PID liveness check precedes UDS connection attempt | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — "daemon auto-start" is named explicitly as a CAP-004 responsibility; this BC specifies the auto-start decision sequence that is the primary user-facing entry point for the daemon auto-start path |
| L2 Domain Invariants | DI-002 (lock file must be present and contain a valid port and auth token before any hook endpoint accepts connections — PC-5 enforces this by requiring a liveness check and lock-file presence before UDS connection) |
| Architecture Module | `monocle` binary crate per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.3.0 §Daemon Auto-Start Logic §Auto-Start Decision Sequence (BC-2.04.002) |
| Cross-Ref | BC-2.04.001 (daemon start sequence — PC-4 triggers this); BC-2.04.003 (MONOCLE_NO_AUTOSTART — precondition gate for this BC); BC-2.04.006 (runtime_dir resolution — PC-1 delegates to this); BC-2.01.005 (lock file PID liveness check pattern used in PC-3) |
| Test File | `monocle/tests/daemon_auto_start.rs` |
| Test Name | `test_BC_2_04_002_daemon_auto_start_on_tui_launch` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.04.001] — composes with: when this BC determines a new daemon is needed, it triggers the BC-2.04.001 start sequence
- [BC-2.04.003] — precondition gate: if `MONOCLE_NO_AUTOSTART` is set, BC-2.04.003 applies and this BC does not execute
- [BC-2.04.006] — depends on: runtime_dir resolution (PC-1) is specified by BC-2.04.006
- [BC-2.01.005] — depends on: the PID liveness check pattern in PC-3 follows BC-2.01.005 semantics
- [BC-2.05.001] — depends on: the UDS socket the TUI connects to at PC-5 is governed by BC-2.05.001

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#daemon-auto-start-logic` — auto-start decision sequence, MONOCLE_NO_AUTOSTART check
- `architecture/SS-daemon-wiring.md#monocle_no_autostart-check-bc-2.04.003` — precondition gate reference

## Story Anchor

S-TBD — Implement TUI auto-start daemon detection and startup (filled by story-writer)

## VP Anchors

VP-TBD — Daemon auto-start integration tests (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T12:01:00Z):
- BC-2.04.002 created as new artifact for SS-04 per prd-expansion-scope.md §3.1 and
  SS-daemon-wiring.md §Daemon Auto-Start Logic §Auto-Start Decision Sequence.
- Covers: 5-step auto-start decision sequence, 10-second total timeout (5s + 5s retry),
  offline-mode fallback, 8 edge cases, 5 test vectors, 4 verification properties.
- input-hash: [pending] — to be populated by compute-input-hash after human review.
- SE-16d PASS: 2026-05-26T12:01:00Z > chain origin 2026-05-26T12:00:00Z (BC-2.04.001).

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
- Architecture Source row: `SS-daemon-wiring.md v1.2.0 §Daemon Auto-Start Logic §Auto-Start Decision Sequence (BC-2.04.002)` → `SS-daemon-wiring.md v1.3.0 §Daemon Auto-Start Logic §Auto-Start Decision Sequence (BC-2.04.002)`.
- Plain version-pin refresh. No substantive content propagation required — §Daemon Auto-Start Logic section heading and content anchors are unchanged between v1.2.0 and v1.3.0.
- SE-16d monotonicity: v1.5.0 timestamp >= v1.4.0. PASS.
