---
document_type: behavioral-contract
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-19T12:05:00Z
phase: 1a
inputs: [prd.md, architecture/ARCH-INDEX.md]
input-hash: "2bc763f"
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

# Behavioral Contract BC-2.01.006: Crash Recovery Checkpoint

## Description

When the monocle daemon starts and finds a `monocle.recovery.json` file in the runtime
directory, it offers the prior session's state to any TUI client that attaches within 60
seconds. The recovery file is written during the drain sequence (before lock file removal),
so its presence indicates the prior daemon exited without clean shutdown. The recovery file
schema uses a closed-set `shutdown_reason` enum and mandatory millisecond-precision ISO 8601
timestamps to enable reliable automated parsing.

## Preconditions

1. On startup, `<runtime_dir>/monocle.recovery.json` exists.
2. The pid in the stale or absent lock file is dead (prior daemon exited without clean shutdown).

## Postconditions

1. Daemon logs `WARN: recovery checkpoint found; prior daemon exited without clean shutdown`.
2. Daemon reads `last_app_mode` and `shutdown_reason` from the recovery file.
3. If a TUI client attaches within 60 seconds of daemon start, daemon sends the recovery state via the UDS control protocol: `{"type":"recovery_available","last_app_mode":"<...>"}`.
4. TUI displays a recovery banner: `"Prior session ended unexpectedly. Restore state? [Y/n]"`.
5. On TUI acknowledgment (Y or 60-second timeout): `monocle.recovery.json` is deleted.
6. On TUI decline (N): `monocle.recovery.json` is deleted without restoring state.
7. If no TUI attaches within 60 seconds: recovery file is deleted silently and daemon starts fresh.

## Invariants

1. The recovery checkpoint file schema is:
   ```json
   {"pid":<N>,"shutdown_reason":"graceful|signal|forced","last_app_mode":"<string>","shutdown_utc":"YYYY-MM-DDTHH:MM:SS.sssZ"}
   ```
   The `shutdown_utc` field MUST use ISO 8601 UTC format with mandatory millisecond precision: `YYYY-MM-DDTHH:MM:SS.sssZ` (matching the `last_hook_ts` format in EC-044). A seconds-only timestamp (e.g., `2026-05-15T07:30:00Z`) is non-compliant. VP-006 enforces this with regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`.
   Field constraints:
   - `pid`: positive integer (≥ 1) per POSIX (PID 0 is reserved for the scheduler)
   - `shutdown_reason`: closed-set enum — exactly one of `"graceful"`, `"signal"`, or `"forced"` (no other value permitted)
   - `last_app_mode`: non-empty string (e.g., `"Running"`, `"ShuttingDown"`, `"Crashed"`); empty string is invalid
   - `shutdown_utc`: ISO 8601 millisecond timestamp matching regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` (UTC explicitly required)
2. Recovery file creation occurs during the drain sequence (step 5 of §Drain) — BEFORE the lock file is removed. If the daemon crashes hard (SIGKILL), the recovery file may not be written; this is acceptable (no recovery file = clean-start behavior).
3. The 60-second TUI attach window is measured from daemon start time, not from the moment the control socket becomes ready.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-054 | Recovery file is malformed JSON (e.g., truncated due to a crash during write) | Daemon logs `WARN: recovery file malformed; starting fresh` and deletes the file; no banner shown to TUI |
| EC-055 | Multiple crash cycles (hypothetical stacking) | Only one `monocle.recovery.json` exists per runtime directory (each shutdown overwrites the previous recovery file) |
| EC-056 | TUI attaches exactly at 60-second boundary | If the recovery offer has already been sent (within the window), the TUI receives it; if the 60-second timeout has expired and the recovery file deleted, the TUI connects to a fresh daemon with no recovery state |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `monocle daemon start` with no recovery file | No WARN log; normal start | happy-path |
| `monocle daemon start` with existing recovery file | WARN logged; UDS message sent if TUI attaches within 60s | edge-case |
| TUI responds Y to banner | Recovery file deleted; state offered to TUI | happy-path |
| TUI responds N to banner | Recovery file deleted; clean start | edge-case |
| 60 seconds elapse without TUI attach | Recovery file deleted silently | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-006 | Recovery file presence triggers WARN log on startup | integration |
| VP-006 | TUI receives `recovery_available` UDS message if it attaches within 60s | integration |
| VP-006 | Recovery file deleted after TUI Y acknowledgment | integration |
| VP-006 | `shutdown_utc` in recovery file matches mandatory millisecond regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$` | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability §SS-01 |
| Capability Anchor Justification | CAP-001 ("Daemon ingestion of Claude Code hook events; lifecycle management") per ARCH-INDEX §Capability traceability — this BC governs crash recovery state continuity which is part of daemon lifecycle management for the hook ingestion subsystem |
| L2 Domain Invariants | DI-002 (the lock file must be present before hook endpoints accept connections — this BC governs the recovery path when the lock file is absent or stale: the new daemon creates a fresh lock file before accepting any connections; the recovery file does not substitute for the lock file) |
| Architecture Module | monocle-runtime (daemon binary) per ARCH-INDEX Subsystem Registry SS-01 |
| Architecture Source | SS-daemon-lifecycle.md v1.0.33 §Daemon Lifecycle Protocol §Crash Recovery |
| Test File | `monocle-runtime/tests/crash_recovery.rs` |
| Test Name | `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup` |
| Stories | S-TBD (filled by story-writer) |
| Old ID (historical) | BC-DAEMON-006 |

## Related BCs (Recommended)

- [BC-2.01.004] — depends on: recovery file is written during drain sequence (BC-2.01.004 Postcondition 6 / ring flush)
- [BC-2.01.005] — composes with: lock file lifecycle determines whether prior daemon exited cleanly or crashed

## Architecture Anchors (Recommended)

- `architecture/SS-daemon-lifecycle.md#daemon-lifecycle-protocol` — crash recovery checkpoint protocol, drain sequence step ordering
- `architecture/SS-daemon-lifecycle.md#crash-recovery` — recovery file schema, TUI attach window, deletion conditions

## Story Anchor (Recommended)

S-TBD — Implement crash recovery checkpoint offer/cleanup protocol (filled by story-writer)

## VP Anchors (Recommended)

- `verification-properties/vp-006-crash-recovery-checkpoint.md` — VP-006 crash recovery checkpoint integration tests

## §Trace v1.0.2

**F-R105-3 + F-R105-9 + OBS-R44-1 closure** (2026-05-17T18:00:00Z):
- F-R105-3: L2 Domain Invariants cell updated.
  - Before: `N/A — no domain-spec/invariants.md exists; CAP-001 per ARCH-INDEX is authoritative source`
  - After: `DI-002 ...`
  - DI-002 mapping: The crash recovery path detects an absent or stale lock file and starts a new daemon which creates a fresh lock file before accepting any hook connections. The recovery file is ephemeral and does not serve as a substitute for the lock file. DI-002 compliance is maintained: lock file present before any endpoint is active.
- F-R105-9 (SE-17c-d body-scope grep): Stale VP ID found and corrected:
  - Invariant 1 body: `VP-DAEMON-006 enforces this with regex` → `VP-006 enforces this with regex`
  - 0 stale BC IDs in non-historical body prose.
- SE-16d monotonicity PASS: 2026-05-17T18:00:00Z > prior 2026-05-17T17:00:00Z (v1.0.1).

## §Trace v1.0.3

**F-R107-2 CRITICAL — Architecture Source pin refresh v1.0.25 → v1.0.30** (2026-05-17T23:30:00Z):
- F-R107-2: Sibling-layer cascade miss from Round 5D (VPs swept but BCs not). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.25 §Daemon Lifecycle Protocol §Crash Recovery`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.30 §Daemon Lifecycle Protocol §Crash Recovery`
  - Canonical version per architect 5E commit 03a4c57 post-R106 closure.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-17T23:30:00Z > prior 2026-05-17T18:00:00Z (v1.0.2).

## §Trace v1.0.4

**F-R109-4 CRITICAL — Architecture Source pin refresh v1.0.30 → v1.0.32; F-R109-14 MED — §Trace reordered ascending** (2026-05-18T05:05:00Z):
- F-R109-4: Architect 8A bumped SS-daemon-lifecycle.md v1.0.30 → v1.0.32 (Round 8A). Architecture Source row updated.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.30 §Daemon Lifecycle Protocol §Crash Recovery`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Crash Recovery`
- F-R109-14: §Trace blocks were descending (v1.0.3, v1.0.2). Reordered to ascending (v1.0.2, v1.0.3, v1.0.4). Content of each section preserved verbatim; only insertion order corrected.
- SE-17c-d body-scope grep: 0 stale BC IDs in non-historical body prose. 0 stale VP IDs.
- SE-16d monotonicity PASS: 2026-05-18T05:05:00Z > prior 2026-05-17T23:30:00Z (v1.0.3). ARITHMETICALLY TRUE: 2026-05-18T05:05:00Z > 2026-05-17T23:30:00Z PASS.

## §Trace v1.0.5

**GAP-PHASE2-R06-1 closure — Architecture Source pin SS-daemon-lifecycle v1.0.32 → v1.0.33** (2026-05-19T12:05:00Z):
- GAP-PHASE2-R06-1: architect commit `2d43127` bumped SS-daemon-lifecycle.md v1.0.32 → v1.0.33 (Ring Buffer Rotation Policy added). BC ledger Architecture Source cell was not cascaded in that commit.
  - SE-17f BEFORE: `SS-daemon-lifecycle.md v1.0.32 §Daemon Lifecycle Protocol §Crash Recovery`
  - SE-17f AFTER: `SS-daemon-lifecycle.md v1.0.33 §Daemon Lifecycle Protocol §Crash Recovery`
- Pointer-only update. No behavioral content change. No new PCs/INVs/ECs.
- SE-17c-d body-scope grep: 0 stale BC IDs. 0 stale VP IDs. No other stale version pins found.
- SE-16d monotonicity PASS: 2026-05-19T12:05:00Z > prior 2026-05-18T05:05:00Z (v1.0.4). ARITHMETICALLY TRUE: PASS.
