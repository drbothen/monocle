---
document_type: story
level: L4
story_id: S-007
epic_id: EPIC-01
version: "1.2"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-05-19T04:00:00Z
phase: 2
points: 5
wave: 3
tdd_mode: strict
priority: P0
depends_on: [S-006]
blocks: []
target_module: monocle-runtime
subsystems: [SS-01]
behavioral_contracts: [BC-2.01.006]
verification_properties: [VP-006]
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/BC-INDEX.md, version: "1.13"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.006.md, version: "1.0.5"}
  - {path: .factory/specs/verification-properties/VP-INDEX.md, version: "1.16"}
  - {path: .factory/specs/verification-properties/vp-006-crash-recovery-checkpoint.md, version: "1.0.14"}
  - {path: .factory/specs/prd.md, version: "1.26.15"}
  - {path: .factory/specs/architecture/ARCH-INDEX.md, version: "1.0.11"}
  - {path: .factory/specs/architecture/SS-daemon-lifecycle.md, version: "1.0.33"}
  - {path: .factory/specs/behavioral-contracts/ss-01/BC-2.01.006.md, section: "§Edge Cases L70-74"}
input-hash: "[live-state]"
traces_to: "Implements BC-2.01.006 (Crash Recovery Checkpoint); verifies VP-006; covers EC-054, EC-055, EC-056."
---

# S-007: Crash Recovery Checkpoint

## Narrative

As a daemon operator, I want the monocle daemon to write a crash-recovery checkpoint
JSON file during the drain sequence before exiting non-cleanly, so that the TUI client
can offer the user a resume option on next startup instead of losing in-progress context.

## Acceptance Criteria

### AC-001 (traces to BC-2.01.006 postcondition 1 — WARN log on startup with recovery file)
When the daemon starts and finds `<runtime_dir>/monocle.recovery.json`, it logs exactly:
`WARN: recovery checkpoint found; prior daemon exited without clean shutdown`
(BC-2.01.006 postcondition 1 verbatim).

### AC-002 (traces to BC-2.01.006 postcondition 2 — read last_app_mode and shutdown_reason)
The daemon reads `last_app_mode` and `shutdown_reason` from the recovery file. Both fields
are used to populate the UDS recovery offer message (AC-003 below).

### AC-003 (traces to BC-2.01.006 postcondition 3 — UDS recovery_available message)
If a TUI client attaches within 60 seconds of daemon start, the daemon sends:
`{"type":"recovery_available","last_app_mode":"<value-from-file>"}` via the UDS control
protocol. The 60-second window is measured from daemon start time, NOT from the moment
the control socket becomes ready (BC-2.01.006 invariant 3).

### AC-004 (traces to BC-2.01.006 postcondition 4 — TUI banner text)
TUI displays: `"Prior session ended unexpectedly. Restore state? [Y/n]"`.
(Phase 1: daemon sends the UDS message; TUI renders the banner — this AC validates the
daemon's send-side only. TUI render is Phase 3.)

### AC-005 (traces to BC-2.01.006 postcondition 5 — Y or 60-second timeout → delete recovery file)
On TUI acknowledgment (Y) OR 60-second timeout with no TUI response:
`monocle.recovery.json` is deleted from `<runtime_dir>`. No partial file is left.

### AC-006 (traces to BC-2.01.006 postcondition 6 — N response → delete recovery file, no restore)
On TUI decline (N): `monocle.recovery.json` is deleted from `<runtime_dir>` without
restoring any state. Clean start proceeds.

### AC-007 (traces to BC-2.01.006 postcondition 7 — no TUI in 60s → delete silently)
If no TUI client attaches within 60 seconds of daemon start: recovery file is deleted
silently and daemon starts fresh. No recovery state is offered.

### AC-008 (traces to BC-2.01.006 invariant 1 — recovery file schema)
The recovery checkpoint file schema is:
```json
{"pid":<N>,"shutdown_reason":"graceful|signal|forced","last_app_mode":"<string>","shutdown_utc":"YYYY-MM-DDTHH:MM:SS.sssZ"}
```
Field constraints (BC-2.01.006 invariant 1 verbatim):
- `pid`: positive integer (≥ 1); PID 0 is reserved for the scheduler
- `shutdown_reason`: closed-set enum — exactly one of `"graceful"`, `"signal"`, or `"forced"`
- `last_app_mode`: non-empty string; empty string is invalid
- `shutdown_utc`: ISO 8601 millisecond timestamp matching regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`
  (VP-006 enforces this regex)

### AC-009 (traces to BC-2.01.006 invariant 2 — write occurs during drain, before lock file removal)
The recovery file `monocle.recovery.json` is written during the drain sequence (step 5 of
§Drain) BEFORE the lock file is removed. SIGKILL-class hard crashes leave no recovery file;
this is acceptable per BC-2.01.006 invariant 2.

### AC-010 (traces to BC-2.01.006 edge case EC-054 — malformed recovery file)
If `monocle.recovery.json` is malformed JSON (e.g., truncated mid-write), the daemon logs:
`WARN: recovery file malformed; starting fresh` and deletes the file. No recovery banner
is shown to the TUI.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,200 |
| BC-2.01.006.md (1.0.5) | ~700 |
| VP-006 file (1.0.14) | ~500 |
| SS-daemon-lifecycle.md v1.0.33 (crash recovery + drain sections) | ~1,500 |
| S-006 lock.rs interface reference | ~300 |
| Test file (crash_recovery.rs) | ~900 |
| **Total estimate** | **~5,100** |

Well within 20% of 200k context window. No split required.

## Tasks

- [ ] Create `RecoveryCheckpoint` struct in `monocle-runtime/src/types.rs` with 4 canonical fields:
  - `pid: u32`, `shutdown_reason: ShutdownReason`, `last_app_mode: String`, `shutdown_utc: String`
  - `ShutdownReason` enum: `#[non_exhaustive]` with variants `Graceful`, `Signal`, `Forced`
  - `#[derive(serde::Serialize, serde::Deserialize)]`
- [ ] Implement `write_recovery_checkpoint(path: &Path, checkpoint: &RecoveryCheckpoint) -> Result<()>` in `monocle-runtime/src/lifecycle.rs`
  - Writes to `<runtime_dir>/monocle.recovery.json` via `tempfile::persist`
  - Mode: `0o600` — same as lock file
  - Called during drain sequence step 5, BEFORE lock file removal (SS-daemon-lifecycle.md v1.0.33 §Drain L534-736; recovery write at L721-735)
- [ ] On daemon start, check for existing `monocle.recovery.json`:
  - If present and valid JSON: log `WARN: recovery checkpoint found; prior daemon exited without clean shutdown`
  - Read `last_app_mode` and `shutdown_reason` fields
  - Start 60-second TUI attach window
  - If TUI attaches within window: send `{"type":"recovery_available","last_app_mode":"<...>"}` over UDS
  - If TUI responds Y or window expires: delete recovery file
  - If TUI responds N: delete recovery file, proceed clean
  - If no TUI attaches in 60s: delete recovery file silently
  - If present but malformed: log `WARN: recovery file malformed; starting fresh`, delete, proceed clean
- [ ] **Crash-simulation oracle:** Test harness uses `tokio::time::pause()` + synthetic `AppMode→ShuttingDown` signal injection per VP-006 §Mechanism (vp-006 L75-83); pre-existing recovery file simulates "prior crash" (no subprocess SIGKILL required). The "crash" in BC-2.01.006 means "prior daemon exited without clean shutdown" — i.e., the test fixture pre-creates a recovery file to simulate the prior crash.
- [ ] `shutdown_utc` field MUST use `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")` format
  (mandatory millisecond precision per BC-2.01.006 invariant 1 and VP-006 regex)
- [ ] Integration tests `monocle-runtime/tests/crash_recovery.rs`:
  - Drain-path write → `monocle.recovery.json` created mode 0600 with correct 4-field schema
  - `shutdown_utc` passes VP-006 regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`
  - Recovery file present on restart → WARN log (exact text from BC-2.01.006 PC-1)
  - UDS `recovery_available` message sent to TUI within 60s window
  - TUI Y-ack → file deleted
  - TUI N-ack → file deleted, clean start
  - 60s timeout with no TUI → file deleted silently
  - Malformed recovery file → `WARN: recovery file malformed; starting fresh` + delete
  - SIGKILL simulation → NO recovery file created (acceptable, per invariant 2)
  - Clean graceful shutdown → NO `monocle.recovery.json` created
- [ ] **Deterministic clock control:** Integration tests MUST use `tokio::time::pause()` + `tokio::time::advance(Duration::from_secs(61))` for the 60-second-window ACs (AC-003, AC-005, AC-007); wall-clock sleeps are forbidden in this test file.
- [ ] **Malformed-file test input:** Malformed-file test fixture: write `{"pid":1,"shutdown_reason":"graceful","last_app_mode":"Running"` (truncated; missing `shutdown_utc` and closing brace) — verifies serde_json parse failure path. Other malformations covered by mutation testing per VP-006 mutation surface.

## Previous Story Intelligence

S-006 (Wave 2): `runtime_dir` resolution and `tempfile::persist` pattern established in `lock.rs`.
Reuse the same atomic JSON write pattern from `DaemonLock::acquire()`.
`chrono 0.4` UTC formatting already in workspace — use `.format("%Y-%m-%dT%H:%M:%S%.3fZ")` for
mandatory millisecond precision (VP-006 enforces regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`).
UDS control socket established in S-005 (graceful shutdown signal path) — reuse the socket plumbing
for the `recovery_available` message dispatch.

### Interface contract consumed from S-006

- `DaemonLock` struct (fields used: `contract_version`, `pid`, `port`, `authToken`, `startTimeUtc`, `app`, `version`)
- `DaemonLock::acquire() -> Result<DaemonLock, DaemonStartError>`
- `DaemonLock::release(self)` (called by S-005 `lifecycle::exit_with`)
- pid-liveness check helper: `nix::sys::signal::kill(Pid::from_raw(pid), None)` — returns `Ok(())` if process is alive, `Err(Errno::ESRCH)` if dead
- Lock file JSON schema (7 fields, mode `0o600`, ISO-8601 `startTimeUtc`)

S-007 recovery branch reads `pid` from the prior (stale) lock file to confirm the prior daemon
process is dead (per BC-2.01.006 precondition 2).

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.33 §Drain L534-736 (recovery write at L721-735); §Crash Recovery L787-802:
- Recovery checkpoint file: `<runtime_dir>/monocle.recovery.json` (NOT `monocle-crash.json`)
- Written DURING drain sequence step 5, BEFORE lock file removal — NOT on panic hook
- SIGKILL leaves no recovery file — no mitigation required; this is explicitly acceptable
- 60-second window measured from daemon start time, NOT from socket-ready time
- Written via `tempfile::persist` — MANDATORY; `std::fs::write` is FORBIDDEN

**Forbidden Dependencies:**
- `std::fs::write` is FORBIDDEN for recovery checkpoint (use `tempfile::persist`)
- Recovery checkpoint writer MUST NOT import from `monocle-tui`
- File MUST be named `monocle.recovery.json` — NOT `monocle-crash.json` (old buggy name)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| tempfile | 3 | Atomic recovery checkpoint write via `NamedTempFile::persist()` |
| serde_json | =1.0.149 | Recovery checkpoint JSON serialization/deserialization |
| serde | 1 | Serialize/Deserialize derive on `RecoveryCheckpoint` |
| chrono | 0.4 | `shutdown_utc` ISO 8601 millisecond-precision timestamp |
| tracing | 0.1 | WARN logs on checkpoint detection and malformed file |
| tokio | =1.52 | Async UDS message dispatch within 60-second window |

## File Structure Requirements

Files to modify:
- `monocle-runtime/src/lifecycle.rs` — `write_recovery_checkpoint()`, startup detection + UDS dispatch
- `monocle-runtime/src/types.rs` — `RecoveryCheckpoint` struct, `ShutdownReason` enum

Files to create:
- `monocle-runtime/tests/crash_recovery.rs` — integration tests (test name: `test_BC_DAEMON_006_crash_recovery_checkpoint_offer_and_cleanup`)

## Trace

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-05-19 | vsdd-factory:story-writer | Initial story decomposition (Phase 2) |
| 1.2 | 2026-05-20 | vsdd-factory:story-writer | Phase 3.B Batch 5: crash-simulation oracle explicit; S-006 interface contract enumerated; error-taxonomy.md unused input replaced with BC-2.01.006 §Edge Cases pointer; deterministic clock control mandated; malformed-file test fixture pinned; SS-daemon-lifecycle line anchors added |
