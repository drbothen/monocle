---
document_type: story
level: L4
story_id: S-020
epic_id: EPIC-04
version: "1.1"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 5
wave: 5
tdd_mode: strict
priority: P1
depends_on: [S-008, S-017]
blocks: []
target_module: monocle-runtime
subsystems: [SS-04]
behavioral_contracts: [BC-2.04.012]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-04/BC-2.04.012.md, version: "1.0.2"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.04.012 (JSONL ring capacity: 4096 RAM entries, 100 MB × 5 rotation, async flush)"
---

# S-020: JSONL Ring Capacity and Rotation Policy

## Narrative

As the monocle daemon, I want the JSONL ring buffer to enforce a 100 MB per-file cap with a
5-rotation rename chain and a 4,096-entry RAM ring for zero-disk-read TUI access, so that
disk usage is bounded and hook events are never lost to unbounded file growth.

## Acceptance Criteria

### AC-001 (traces to BC-2.04.012 postcondition PC-1 — RAM ring: 4096 events)
The `RingBuffer` maintains an in-memory circular buffer of the last N=4,096 `HookEventRecord`
entries. When a new event is appended and the ring is at capacity, the oldest entry is evicted.
N=4,096 is a compile-time constant; NOT configurable at runtime in Phase 1.

### AC-002 (traces to BC-2.04.012 postcondition PC-2 — 100 MB active file cap)
The active JSONL file is at `<runtime_dir>/monocle.jsonl`. The daemon tracks the on-disk byte
count. When the file size reaches or exceeds 100 MB (104,857,600 bytes), the rotation procedure
is triggered before the next event is written.

### AC-003 (traces to BC-2.04.012 postcondition PC-3 — rotation procedure)
Rotation executes in this exact order:
1. If `.jsonl.5` exists → delete it.
2. If `.jsonl.4` exists → rename to `.jsonl.5`.
3. If `.jsonl.3` exists → rename to `.jsonl.4`.
4. If `.jsonl.2` exists → rename to `.jsonl.3`.
5. If `.jsonl.1` exists → rename to `.jsonl.2`.
6. Rename active `monocle.jsonl` → `monocle.jsonl.1`.
7. Create new empty `monocle.jsonl` (mode `0o600`).
8. Reset in-memory byte-count tracker to 0.
After rotation, no more than 5 rotation files exist on disk simultaneously.

### AC-004 (traces to BC-2.04.012 postcondition PC-4 — async-jsonl flush mode)
Events are NOT written synchronously in hook handlers. `ring.append(event)` enqueues to
a bounded internal write-queue; the background flush task drains and appends JSONL lines.
If the write-queue is full, `ring.append()` returns `Err(RingError::WriteFull)`. Hook handlers
log `WARN: ring append failed: write queue full` and discard the event. No handler blocks on disk I/O.

### AC-005 (traces to BC-2.04.012 postcondition PC-5 — JSONL format compatibility)
Every line written to `monocle.jsonl` MUST conform to the `HookEventRecord` schema from
BC-2.01.007: field order `format_version`, `session_id`, `timestamp_micros`, `pid`, `hook_type`,
`tool_name` (optional), `tool_input` (optional). `format_version` is always `1`. Each line
terminated by single `\n`.

### AC-006 (traces to BC-2.04.012 postcondition PC-6 — active file mode 0o600)
The active JSONL file and all rotation files have mode `0o600`. When the active file is
created (at daemon start or after rotation step 7), mode `0o600` is set immediately.
Rotation files inherit mode from the active file rename (POSIX rename preserves mode).

### AC-007 (traces to BC-2.04.012 postcondition PC-7 — graceful shutdown: flush and close)
On graceful shutdown, the background flush task drains the write-queue and closes the active
JSONL file. The file is NOT deleted on shutdown (unlike lock file and hooks-settings.json).
Rotation files are also retained.

### AC-008 (traces to BC-2.04.012 postcondition PC-8 — crash recovery: partial-line truncation)
If the daemon crashes and `monocle.jsonl` ends with a partial JSONL line (no trailing `\n`),
the next daemon start MUST detect and truncate to the last complete `\n`-terminated line.
This truncation is performed in `RingBuffer` construction at step 4 of the start sequence.

### AC-009 (traces to BC-2.04.012 invariant 1 — max 5 rotation files)
The maximum number of on-disk rotation files is 5 at all times. Rotation step 1 (delete
`.jsonl.5`) enforces this on every rotation. After 6 rotations, the oldest data is gone.

### AC-010 (traces to BC-2.04.012 invariant 5 — no handler blocking on disk I/O)
`ring.append()` MUST NOT block the calling hook handler thread on disk I/O. All disk I/O
is the exclusive responsibility of the background flush task.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,100 |
| BC-2.04.012.md | ~1,200 |
| SS-daemon-lifecycle.md §JSONL Ring Buffer | ~3,000 |
| BC-2.01.007 (format schema from S-008) | ~400 |
| Test file | ~700 |
| **Total estimate** | **~6,400** |

## Tasks

- [ ] Extend `RingBuffer` in `monocle-runtime/src/ring.rs` (from S-008) with:
  - RAM ring: `VecDeque<HookEventRecord>` with capacity 4096 (compile-time constant)
  - On-disk byte-count tracker in `RingBuffer` internal state
  - Rotation trigger: check byte-count after each write; rotate when >= 100 MB
- [ ] Implement rotation procedure as sequential rename chain (steps 1-8 from BC-2.04.012 PC-3)
- [ ] Background flush task: bounded write-queue → `ring.append()` enqueues → flush task drains → JSONL append
- [ ] `RingError::WriteFull` returned when write-queue is full (not a block)
- [ ] On first event (no active file): create `monocle.jsonl` with mode `0o600`
- [ ] Crash recovery at `RingBuffer::new()`: detect partial line at EOF; truncate to last `\n`
- [ ] Graceful shutdown: flush task drains queue → close file (file NOT deleted)
- [ ] Validate JSONL schema compatibility with BC-2.01.007 (format_version=1 first field)
- [ ] Unit tests `monocle-runtime/tests/ring_capacity_rotation.rs`:
  - 4096 events appended: RAM ring at capacity; 4097th evicts oldest
  - Rotation triggered at 100 MB: active file renamed to `.jsonl.1`; new empty file created
  - 6 rotations: `.jsonl.5` deleted; max 5 rotation files on disk
  - Partial-line recovery at startup: truncated to last `\n`; next write produces valid record
  - Disk full on rotation: subsequent appends return `Err(RingError::DiskFull)`; HTTP responses continue
  - Active file mode is `0o600` after rotation

## Previous Story Intelligence

S-008: `RingBuffer` struct exists with `async-jsonl` flush mode and `HookEventRecord` schema
(BC-2.01.007 format_version=1). This story adds capacity limits and rotation behavior on top
of the base format. The `format_version` first-field invariant from BC-2.01.007 is preserved.

S-017: `DaemonState.ring` is `Arc<RingBuffer>`. The `RingBuffer` is constructed at step 4 of
the start sequence. This story's rotation policy configuration is passed during construction.

## Architecture Compliance Rules

From `architecture/SS-daemon-wiring.md v1.2.0 §Daemon Start Sequence (Step 4)` (at S-020 authoring time):
- RAM ring N=4,096; 100 MB per-file threshold; 5 rotation files
- Background flush task is spawned AT step 4 of start sequence
- Ring capacity is a compile-time constant — NO runtime override in Phase 1

From `architecture/SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer`:
- The partial-line truncation MUST occur at RingBuffer construction (step 4), not lazily
- Active file mode `0o600` is set immediately after creation — not deferred

**Forbidden Dependencies:**
- `ring.append()` MUST NOT block the hook handler thread (async-jsonl pattern)
- `std::fs::write` MUST NOT be used for JSONL file creation (use open-for-append with mode)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| tokio | =1.52.0 | Background flush task; async I/O; bounded channel for write-queue |
| serde_json | =1.0.149 | JSONL line serialization (one JSON object per line) |
| serde | 1 (features=["derive"]) | `#[derive(Serialize)]` on `HookEventRecord` |
| tracing | 0.1 | WARN on disk full, write queue full, rotation events |

## File Structure Requirements

Files to modify:
- `monocle-runtime/src/ring.rs` — extend `RingBuffer` with RAM ring VecDeque, byte-count tracker, rotation procedure, flush task, crash recovery

Files to create:
- `monocle-runtime/tests/ring_capacity_rotation.rs` — capacity and rotation integration tests

## §Trace

**v1.1** (2026-05-30) — POL-11 version-pin staleness remediation: added `<!-- version-pin-historical -->` markers per ADR-0007 §Historical Anchor Classification to all active-pointer citations that document spec versions at story authoring time. No normative content changed.
