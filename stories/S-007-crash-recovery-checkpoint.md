---
document_type: story
story_id: S-007
epic_id: EPIC-01
version: "1.0"
status: draft
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
---

# S-007: Crash Recovery Checkpoint

## Narrative

As a daemon operator, I want the monocle daemon to write a crash-recovery checkpoint
JSON file before exiting abnormally, so that the TUI client can offer the user a
"Resume from checkpoint?" option on next startup instead of losing in-progress context.

## Acceptance Criteria

### AC-001 (traces to BC-2.01.006 postcondition 1 — checkpoint write on abnormal exit)
When the daemon exits abnormally (exit code ≠ 0, excluding SIGKILL which is uncatchable),
it writes a crash-recovery checkpoint JSON to `<runtime_dir>/monocle-crash.json` via
`tempfile::persist` (atomic write, mode 0o600).

### AC-002 (traces to BC-2.01.006 postcondition 2 — checkpoint JSON schema)
The checkpoint file contains: `{pid, shutdown_utc, exit_code, ring_fill_pct, hook_counts: {pre_tool_use, notification, stop, session_start, prompt_submit}}`.
`shutdown_utc` uses `chrono` ISO 8601 UTC with mandatory millisecond precision.

### AC-003 (traces to BC-2.01.006 postcondition 3 — offer checkpoint to TUI on restart)
When the daemon starts and finds a `monocle-crash.json` at `<runtime_dir>`, it logs
`INFO: crash checkpoint found from <shutdown_utc>; offering resume to TUI clients`.
(Phase 1 scope: write and detect the checkpoint. TUI "resume" UI is Phase 3.)

### AC-004 (traces to BC-2.01.006 postcondition 4 — checkpoint removed after offer)
After successfully offering (logging) the checkpoint, the daemon removes
`monocle-crash.json` from `<runtime_dir>` during startup.

### AC-005 (traces to BC-2.01.006 invariant 1 — clean shutdown does NOT write checkpoint)
On a clean graceful shutdown (exit code 0), NO crash checkpoint is written.
Integration test: clean shutdown → `monocle-crash.json` does NOT exist.

### AC-006 (traces to BC-2.01.006 invariant 2 — atomic write)
`monocle-crash.json` is written via `tempfile::persist` — no partial checkpoint
file is observable if the write fails midway.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~700 |
| BC-2.01.006.md | ~600 |
| VP-006 file | ~500 |
| SS-daemon-lifecycle.md (crash recovery section, ~60 lines) | ~900 |
| S-006 lock.rs interface reference | ~200 |
| Test file | ~600 |
| **Total estimate** | **~3,500** |

## Tasks

- [ ] Create `CrashCheckpoint` struct with the 6 required fields; derive `serde::Serialize`
- [ ] Implement `write_crash_checkpoint()` in `monocle-runtime/src/lifecycle.rs`
  - Called from panic hook and from abnormal-exit path
  - Writes to `<runtime_dir>/monocle-crash.json` via `tempfile::persist`
  - Mode: `0o600`
- [ ] Install tokio panic hook that calls `write_crash_checkpoint()` then logs before exit 3
- [ ] On daemon start, check for existing `monocle-crash.json`:
  - If found: log INFO with `shutdown_utc`, then remove the file
  - If absent: no action
- [ ] Integration tests `monocle-runtime/tests/crash_recovery.rs`:
  - Abnormal exit → checkpoint file created mode 0600 with correct schema
  - Checkpoint detected on restart → INFO log + file removed
  - Clean shutdown → NO checkpoint file created
  - Panic path → checkpoint written before exit 3

## Previous Story Intelligence

S-006 (Wave 2): `runtime_dir` resolution and `tempfile::persist` pattern established in `lock.rs`.
Reuse the same `atomic_write_json(path, &data, 0o600)` helper established for the lock file.
`chrono` UTC formatting already in use — reuse `chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ")`.

## Architecture Compliance Rules

From `architecture/SS-daemon-lifecycle.md` v1.0.32 §Crash Recovery Checkpoint:
- Checkpoint file: `<runtime_dir>/monocle-crash.json`
- Written via `tempfile::persist` — MANDATORY
- Mode `0o600` — same as lock file
- Clean shutdown MUST NOT write checkpoint — gated on exit code

**Forbidden Dependencies:**
- `std::fs::write` is FORBIDDEN for checkpoint (use `tempfile::persist`)
- Checkpoint writer MUST NOT import from `monocle-tui`

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| tempfile | 3 | Atomic checkpoint write |
| serde_json | =1.0.149 | Checkpoint JSON serialization |
| chrono | 0.4 | `shutdown_utc` ISO 8601 timestamp |
| tracing | 0.1 | INFO log on checkpoint detection |

## File Structure Requirements

Files to modify:
- `monocle-runtime/src/lifecycle.rs` — `write_crash_checkpoint()`, checkpoint detection on start
- `monocle-runtime/src/types.rs` — `CrashCheckpoint` struct (create if not exists)

Files to create:
- `monocle-runtime/tests/crash_recovery.rs` — integration tests
