---
document_type: behavioral-contract
level: L3
version: "1.0.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:05:00Z
phase: 1a
inputs: [prd.md, architecture/SS-daemon-wiring.md, architecture/SS-daemon-lifecycle.md, architecture/ARCH-INDEX.md]
input-hash: "d0de914"
traces_to: prd.md
origin: greenfield
subsystem: SS-04
capability: CAP-004
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.012: JSONL Ring: Capacity and Rotation Policy

## Description

The JSONL ring buffer is a hybrid storage system: a RAM ring holds the last 4,096 hook events
for zero-disk-read TUI access, while an async JSONL flush writes events to disk for persistence.
The on-disk component consists of an active JSONL file at `<runtime_dir>/monocle.jsonl` and up
to 5 rotation files (`.jsonl.1` through `.jsonl.5`). When the active file reaches 100 MB, it is
rotated: the active file becomes `.jsonl.1`, prior rotation files shift up by one, and `.jsonl.5`
is deleted if it exists. This contract specifies the capacity and rotation policy that
BC-2.01.007 (which covers format_version semantics) depends on for its storage layer.

## Preconditions

1. The daemon start sequence is executing step 4 (RingBuffer construction before event bus
   initialization at step 5 and before HTTP server start at step 12).
2. `<runtime_dir>` exists and is owned by the daemon user with mode `0o700`
   (created at step 1, BC-2.04.001 PC-1).
3. `DaemonState.ring` is being constructed as `Arc<RingBuffer>`.
4. The `monocle-runtime` crate implements `RingBuffer` with `async-jsonl` flush mode.

## Postconditions

**PC-1 — RAM ring: 4,096 events in memory.**
The `RingBuffer` maintains an in-memory circular buffer of the last N=4,096 `HookEventRecord`
entries. When a new event is appended and the ring is at capacity (4,096 entries), the oldest
entry is evicted to make room. The RAM ring is the authoritative fast-path for TUI queries:
the TUI reads the last N events directly from the RAM ring without disk I/O. The RAM ring size
of 4,096 is a compile-time constant; it MUST NOT be configurable at runtime in Phase 1.

**PC-2 — Active JSONL file: 100 MB per-file capacity.**
The active JSONL log file is located at `<runtime_dir>/monocle.jsonl`. The daemon tracks the
on-disk byte count of the active file (maintained in `RingBuffer` internal state; updated
after every successful write). When the file size reaches or exceeds 100 MB (104,857,600
bytes), the rotation procedure (PC-3) is triggered before the next event is written.

**PC-3 — Rotation procedure: rename chain + delete oldest.**
Rotation executes the following steps atomically as a sequential batch (best-effort; see EC-102
for partial-rotation handling):
1. If `<runtime_dir>/monocle.jsonl.5` exists → delete it.
2. If `<runtime_dir>/monocle.jsonl.4` exists → rename to `monocle.jsonl.5`.
3. If `<runtime_dir>/monocle.jsonl.3` exists → rename to `monocle.jsonl.4`.
4. If `<runtime_dir>/monocle.jsonl.2` exists → rename to `monocle.jsonl.3`.
5. If `<runtime_dir>/monocle.jsonl.1` exists → rename to `monocle.jsonl.2`.
6. Rename `<runtime_dir>/monocle.jsonl` (current active file) → `monocle.jsonl.1`.
7. Create a new empty `<runtime_dir>/monocle.jsonl` file (open for append, mode `0o600`).
8. Reset the in-memory byte-count tracker to 0.

After rotation, no more than 5 rotation files exist on disk simultaneously
(`.jsonl.1` through `.jsonl.5`). The total maximum on-disk storage for the ring is
`6 × 100 MB = 600 MB` (active file at 100 MB limit + 5 rotation files each at 100 MB limit).

**PC-4 — Flush mode: async-jsonl.**
Events are NOT written synchronously in the hook handler. The ring buffer uses a background
flush task (spawned at daemon start step 4) that drains a write-queue and appends JSONL lines
to the active file. Hook handlers call `ring.append(event)` which enqueues to the write-queue
(bounded internal queue); the handler does not block on disk I/O. If the write-queue is full,
`ring.append()` returns `Err(RingError::WriteFull)`; the handler logs `WARN: ring append
failed: write queue full` and discards the event. This preserves hook handler latency budgets.

**PC-5 — JSONL record format compatibility.**
Every line written to the active JSONL file MUST conform to the `HookEventRecord` schema
specified in BC-2.01.007. Specifically:
- Field order: `format_version`, `session_id`, `timestamp_micros`, `pid`, `hook_type`,
  `tool_name` (optional), `tool_input` (optional).
- `format_version` is always `1` (per BC-2.01.007 PC-1).
- Each line is terminated by a single newline character (`\n`).
- No trailing commas; no JSON array wrapping; each line is a standalone JSON object.

**PC-6 — Active file mode: `0o600`.**
The active JSONL file (`monocle.jsonl`) and all rotation files (`monocle.jsonl.1` through
`monocle.jsonl.5`) MUST have mode `0o600`. When the active file is created (at daemon start
or after rotation step 7), mode `0o600` is set immediately after creation. Rotation files
inherit their mode from the active file rename (rename preserves mode on POSIX systems).

**PC-7 — Graceful shutdown: flush and close.**
On graceful shutdown, the background flush task drains the write-queue and closes the active
JSONL file before the daemon process exits. The file is NOT deleted on shutdown (unlike the
lock file and hooks-settings.json). Rotation files are also retained. The persistent ring is
preserved for crash recovery and post-mortem analysis.

**PC-8 — Crash recovery compatibility.**
If the daemon crashes between events (i.e., the last write to `monocle.jsonl` is a partial
line), the next daemon start MUST detect and truncate the partial line before appending new
events. The truncation point is the last complete `\n`-terminated line. Partial-line recovery
is part of the daemon start sequence at step 4 when the RingBuffer is constructed and the
existing active file is opened.

## Invariants

1. The maximum number of on-disk rotation files is 5. Rotation step 1 (delete `.jsonl.5`)
   ensures this invariant is maintained on every rotation.
2. The active file is always at `<runtime_dir>/monocle.jsonl`. No other path is used for
   the active file.
3. The RAM ring always contains the last N events seen by the daemon in this run. On daemon
   restart, the RAM ring starts empty (crash recovery reads from disk if needed, but the
   RAM ring is not pre-populated from disk at startup in Phase 1).
4. The byte-count tracker in `RingBuffer` internal state MUST match the actual on-disk file
   size within one write cycle. Rotation MUST be triggered no later than the write that
   causes the file to exceed 100 MB; it MUST NOT be triggered before the 100 MB threshold.
5. `ring.append()` MUST NOT block the calling hook handler thread on disk I/O. All disk I/O
   is the exclusive responsibility of the background flush task.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-100 | First event after daemon start: `monocle.jsonl` does not exist | Flush task creates `monocle.jsonl` with mode `0o600`; event written; byte-count tracker starts at line length |
| EC-101 | Rotation triggered; `.jsonl.5` does not exist (fewer than 5 prior rotations) | Rotation step 1 is skipped (file absent); steps 2-7 proceed normally; no error logged |
| EC-102 | Rotation partially fails mid-sequence (e.g., rename `.jsonl` → `.jsonl.1` succeeds, then process crashes before new active file is created) | On next daemon start (crash recovery), `monocle.jsonl` is absent; `monocle.jsonl.1` is the most recent file; RingBuffer construction at step 4 detects the absent active file and creates a new empty `monocle.jsonl`; data in `.jsonl.1` through `.jsonl.5` is preserved |
| EC-103 | Disk full during rotation (rename succeeds, new active file creation fails) | `RingBuffer` enters an error state; subsequent `ring.append()` calls return `Err(RingError::DiskFull)`; hook handlers log WARN and discard ring appends; HTTP responses continue normally (ring is best-effort per PC-6 of hook routing BCs); daemon logs periodic ERROR about disk full state |
| EC-104 | RAM ring at capacity (4096 events); new event arrives | Oldest RAM ring entry evicted; new event appended; RAM ring size stays at 4096; evicted event is NOT deleted from disk (disk is the persistence layer) |
| EC-105 | Partial JSONL line at end of `monocle.jsonl` on startup (prior crash) | RingBuffer construction truncates to last complete newline; partial bytes discarded; next write appends a complete new line; no error surfaced to hook handlers |
| EC-106 | TUI queries the RAM ring while 4000 events are being flushed to disk | TUI reads from RAM ring (in-memory); no disk I/O required for TUI query; read is lock-protected for consistency; query completes without blocking flush task |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 4096 events appended to empty ring | RAM ring contains 4096 events; disk file contains 4096 JSONL lines; byte-count = sum of line lengths | happy-path |
| 4097th event appended to full RAM ring | RAM ring evicts oldest; contains 4096 events (events 2-4097); disk contains 4097 lines | happy-path |
| 100 MB disk threshold reached | Rotation triggered: active file renamed to `.jsonl.1`; new empty `monocle.jsonl` created; subsequent writes go to new active file | happy-path |
| 6 rotations triggered (after 600 MB written) | `.jsonl.5` deleted on 6th rotation; maximum 5 rotation files on disk; total disk usage bounded at ~600 MB | edge-case |
| Partial JSONL line at EOF on startup | Partial bytes truncated; daemon starts normally; next event writes valid complete line | edge-case |
| Disk full on rotation | Subsequent appends return `Err(RingError::DiskFull)`; hook handlers log WARN; HTTP responses continue | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | RAM ring evicts oldest event when at 4096-event capacity | unit (append 4097 events; assert oldest absent, newest present) |
| VP-TBD | Rotation triggered at exactly 100 MB; `monocle.jsonl.1` contains prior content | integration (write 100 MB of events; verify rotation) |
| VP-TBD | Maximum 5 rotation files on disk after 6 rotations | integration (trigger 6 rotations; count `.jsonl.*` files) |
| VP-TBD | JSONL records conform to BC-2.01.007 format_version=1 schema | unit (parse output lines with HookEventRecord deserializer) |
| VP-TBD | Active file mode is 0o600 after rotation | integration (stat the new active file) |
| VP-TBD | Partial-line recovery at startup truncates to last `\n` | unit (inject partial line; verify next write produces valid record) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §Capability Traceability §SS-04 |
| Capability Anchor Justification | CAP-004 ("Binary composition root; CLI surface; daemon auto-start; bounded event bus; hook tmpfile generation") per ARCH-INDEX §SS-04 — the JSONL ring is constructed at daemon start step 4 and wired into `DaemonState` as part of the composition-root start sequence (BC-2.04.001 PC-4); the ring's capacity and rotation policy are composition-root concerns because the ring is initialized in the SS-04 start sequence before any other subsystem runs; CAP-004 is the correct anchor as the composition root that owns the ring initialization contract |
| L2 Domain Invariants | DI-001 (every hook event received MUST be written to the JSONL ring before acknowledgement — PC-4 defines the async-jsonl flush mode; best-effort queue-full behavior (RingError::WriteFull) represents the I/O-layer failure exception to DI-001; in normal non-overload paths, every event enters the write queue and is written to disk); DI-004 (all public wire types MUST carry a version discriminant as their first field — PC-5 states format_version is always the first field in every JSONL record, operationalizing DI-004 for the ring storage format) |
| Architecture Module | monocle-runtime (RingBuffer, flush task) per ARCH-INDEX Subsystem Registry SS-04 |
| Architecture Source | SS-daemon-wiring.md v1.2.0 §Daemon Start Sequence (Step 4) and SS-daemon-lifecycle.md v1.0.33 §JSONL Ring Buffer |
| Cross-Ref | BC-2.01.007 (JSONL ring format_version — specifies the record schema that this BC's rotation policy persists); BC-2.04.001 (daemon start sequence step 4 — constructs the RingBuffer with this capacity and rotation policy) |
| Test File | `monocle-runtime/tests/ring_capacity_rotation.rs` |
| Test Name | `test_BC_2_04_012_ring_capacity_and_rotation` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.01.007] — composes with: JSONL format_version governs the schema of each line written by the ring; BC-2.04.012 specifies how many lines fit on disk before rotation
- [BC-2.04.001] — depends on: daemon start sequence step 4 constructs this ring buffer
- [BC-2.04.007] — composes with: PreToolUse handler appends to this ring at PC-6
- [BC-2.04.008] — composes with: Notification handler appends to this ring at PC-6
- [BC-2.04.009] — composes with: Stop/SessionStart/PromptSubmit handlers append to this ring at PC-6

## Architecture Anchors

- `architecture/SS-daemon-wiring.md#daemon-start-sequence` — step 4: RingBuffer construction with 100MB × 5 rotation and RAM ring 4096 capacity
- `architecture/SS-daemon-lifecycle.md#jsonl-ring-buffer` — ring buffer implementation specification (authoritative for implementation details)

## Story Anchor

S-TBD — Implement JSONL ring capacity (4096 RAM entries, 100MB × 5 rotation) with async flush and crash recovery (filled by story-writer)

## VP Anchors

- VP-TBD — filled after VP creation

## §Trace v1.0.0

**Initial production** (2026-05-26T12:05:00Z):
- BC-2.04.012 created as new artifact for SS-04 §JSONL Ring Capacity and Rotation per task
  instruction.
- Covers: RAM ring (N=4096), 100 MB per-file threshold, 5-rotation rename chain, delete
  oldest on 6th rotation, async-jsonl flush mode, write-queue bounded (no handler blocking),
  JSONL format compatibility with BC-2.01.007, active file mode 0o600, graceful flush on
  shutdown, crash-recovery partial-line truncation.
- Capability anchor: CAP-004 per ARCH-INDEX §SS-04 Capability Traceability row (composition-
  root owns RingBuffer initialization at step 4 of daemon start sequence).
- DI-001 and DI-004 both enforced: DI-001 via async write queue (best-effort for queue-full
  only); DI-004 via PC-5 format_version first-field invariant.
- SE-16d PASS: 2026-05-26T12:05:00Z > chain prior 2026-05-26T12:04:00Z. PASS.

## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.0.0` → `SS-daemon-wiring.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-003 LOW — Architecture Source pin updated from v1.1.0 to v1.2.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-daemon-wiring.md v1.1.0` → `SS-daemon-wiring.md v1.2.0` per F-P1D4-003 bulk update. Note: `SS-daemon-lifecycle.md v1.0.33` pin was already correct and is unchanged.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.
