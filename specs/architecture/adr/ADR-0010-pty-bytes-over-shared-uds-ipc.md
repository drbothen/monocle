---
document_type: adr
level: L3
adr_id: "ADR-0010"
title: "PTY Bytes Shared on Existing UDS IPC Channel (Option A)"
status: accepted
producer: vsdd-factory:architect
phase: v1A-architecture-delta
version: "1.6.0"
timestamp: 2026-06-03T00:00:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - specs/product-brief.md
  - specs/research/embedded-pty-evaluation.md
  - specs/architecture/SS-ipc.md
input-hash: "042e8f5"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# ADR-0010: PTY Bytes Shared on Existing UDS IPC Channel (Option A)

## Status

Accepted — 2026-06-03 (Q-1 resolution, architecture delta)

## Context

With the control-center pivot, the daemon must stream PTY bytes from session-host processes
to connected TUI clients. The key architectural question (Q-1 from vision §Open Questions):
should `PtyOutput { session_id, bytes }` messages share the existing UDS channel with hook
events and control messages, or should each session have a dedicated high-throughput path?

### Option A: Shared UDS channel, per-session bounded buffer

The existing UDS IPC between daemon and TUI (SS-05, `monocle.sock`) is extended with three
new message types: `PtyOutput`, `KeyInput`, `ResizePane`. These are additional variants of
the `ServerToClient` and `ClientToServer` enums (already `#[non_exhaustive]`).

Each session's PTY reader thread posts bytes to a bounded `mpsc::channel(N)` in the session-host
proxy on the daemon side. The daemon's broker fan-out task sends `PtyOutput` messages to all
subscribed TUI clients over the existing per-client UDS write path.

*Concern:* PTY byte bursts could starve hook event delivery. Mitigated by the per-session
channel being separate from the hook-event channel; the broker interleaves them in a single
`tokio::select!` arm. The `CLAUDE.md` drop-counter convention surfaces starvation visibly in
the TUI status bar.

### Option B: Dedicated per-session streaming path

Each session spawns a second UDS socket (or a named pipe) for raw PTY byte streaming.
The TUI connects to N sockets for N active sessions.

*Concern:* Complexity grows O(N) in active session count. The TUI must manage N async
connections. Session discovery (TUI must learn the per-session socket path) requires
additional IPC messages anyway. Option B solves one problem (throughput isolation) while
creating a larger problem (connection multiplicity).

## Decision

**Option A: share the existing UDS IPC channel with bounded per-session PTY buffers.**

The session-host proxy inside the daemon sends `ServerToClient::PtyOutput { session_id, bytes }`
over the existing single UDS connection to each TUI client. `KeyInput` and `ResizePane` arrive
as `ClientToServer` variants over the same connection.

**Throughput sizing (I5-002 — reconciled with MAX_MESSAGE_BYTES):**

The per-session PTY output channel capacity is set at **1024 messages** in the session-host
PTY reader (the bounded `mpsc::channel::<Bytes>(1024)` — see SS-session-manager §PTY reader
thread). This is the session-host-internal buffer; its backlog bound depends on the actual
per-message size, not a fixed 4096-byte assumption.

- **Typical case:** PTY output is buffered in line-buffered or page-buffered chunks. Observed
  terminal application output is typically 256 B – 4 KiB per PTY read call. At the typical
  4 KiB per message: 1024 × 4 KiB = **4 MiB per-session typical backlog**.
- **Maximum case per `MAX_MESSAGE_BYTES` (BC-2.05.001):** Each `PtyOutput` message payload is
  bounded at 256 KiB (`MAX_MESSAGE_BYTES = 262_144`). A single EC-273–class burst (e.g.,
  BC-2.05.009 EC-273 sends 64 KiB in one message) is well within the 256 KiB ceiling.
  At the maximum 256 KiB per message: 1024 × 256 KiB = **256 MiB per-session worst-case backlog**.
  In practice this maximum is never reached — the PTY read syscall returns at most one kernel
  buffer's worth of bytes per read (typically ≤ 64 KiB); no single PTY read fills a 256 KiB frame.
- **Benchmark gate basis:** The benchmark gate (§Benchmark gate) targets `pty_drop_counter = 0`
  at session count ≤ 8, 60Hz display refresh. The benchmark MUST be run at typical terminal
  output rates (10–50 KiB/s) and at burst rates (up to 1 MB/s as required by the
  scrollback-dump benchmark; see §Benchmark gate and §Interleaving of live PtyOutput).
  The 4 MiB typical backlog provides ample headroom at human-legible rates.

Terminal applications rarely exceed 10–50 KiB/s of text output at human-legible speeds.
High-throughput scenarios (e.g., `cargo build` flooding PTY at 1 MB/s) are covered by the
pre-v1A benchmark gate and the drop-counter convention.

**Benchmark gate — BEFORE THE v1A STORY WAVE BEGINS (C3 clarification):**

The performance-engineer MUST benchmark PTY byte delivery on the shared UDS channel
BEFORE implementation of the v1A story wave begins (i.e., before implementing SS-08/SS-09
stories). This is not a "before v1A launch" gate — it is a pre-implementation gate that
must unblock the story wave or force Option B before any PTY implementation work starts.

Benchmark targets:
1. The `pty_drop_counter` (session-host → daemon channel) does NOT fire under normal
   terminal refresh load (target: 0 drops at session count ≤ 8 at 60Hz display refresh
   with `.send().await` backpressure design per SS-session-manager.md §PTY reader thread).
2. Hook event delivery latency does not exceed 50ms during concurrent PTY streaming of
   8 sessions at 60Hz refresh rate.
3. The `≥ 1000 events/sec` CLAUDE.md convention is satisfied end-to-end (hook events
   + PTY bytes share the channel; combined throughput must not starve hook delivery).

**Head-of-line blocking analysis (C3 / I8):** The shared UDS carries both hook events
(small JSON, latency-critical) and PTY bytes (potentially large, throughput-oriented).
With the `.send().await` backpressure design, a slow TUI consumer could apply back-pressure
up to the PTY reader, slowing PTY output — which is the correct behavior (harness TUI
stalls). However, PTY bytes and hook events share the same broker fan-out and the same
per-client UDS write path. A burst of large PTY byte messages could starve small hook
event messages on the same `tokio::select!` arm.

**Head-of-line blocking mitigation in the broker:**
The broker's `tokio::select!` MUST interleave hook events and PTY output fairly. If the
broker receives both `Event::HookEventReceived` and `Event::PtyOutput` simultaneously,
hook events MUST be given priority (they are latency-critical for the PreToolUse 300ms
budget). Implementation: the broker `tokio::select!` uses `biased;` with hook events
polled first, or uses two separate channels (one per event class) with `select!` giving
hook events a priority weight. The benchmark must confirm this priority is effective.

**If the benchmark reveals that Option A is insufficient** (pty_drop_counter fires
consistently under test load, or hook latency exceeds 50ms), the architect MUST be
re-engaged to evaluate Option B (dedicated per-session streaming path) BEFORE v1A
implementation begins. This finding must not be deferred to post-implementation.

Route to: `vsdd-factory:performance-engineer` for benchmark design and execution.

## Consequences

### Positive

- Zero new IPC connections to manage in the TUI.
- Uses existing frame protocol, serialization, and reader task (SS-05 §TUI IPC Read Loop Pattern).
- The drop-counter convention (already built and rendered in the status bar) surfaces any
  starvation automatically.
- Simpler TUI reconnection logic: one connection covers all session state + PTY streams.

### Negative / costs

- A high-throughput session (e.g., `cargo build` output flooding the PTY) shares bandwidth
  with hook events on the same connection. Mitigated by bounded per-session channels and the
  pre-gate benchmark gate.
- The 256 KiB message size limit (BC-2.05.001 / `MAX_MESSAGE_BYTES`) constrains per-`PtyOutput`
  payload. In practice, terminal output is sent in line-buffered or page-buffered chunks well
  under 4 KiB; the 256 KiB limit is not a practical constraint.

## IPC Message Type Additions

The following new variants are added to the existing SS-05 message enums:

```rust
// ── ServerToClient (new variants in v1A) ──────────────────────────────────

/// Raw PTY bytes from a session's harness child. The TUI feeds these into
/// the per-session vt100::Parser.
PtyOutput {
    session_id: String,  // matches session_id throughout the codebase
    bytes: Vec<u8>,      // raw PTY output bytes; NOT pre-decoded
},

/// A session's lifecycle state changed (e.g., Launching → Running,
/// Running → Terminated).
SessionStateChanged {
    session_id: String,
    new_state: SessionState,
},

/// Scrollback dump — one chunk of a multi-message scrollback transfer.
///
/// When the daemon first attaches to (or re-discovers) a session-host, it
/// sends DaemonToHost::Attach. The session-host responds by streaming the
/// current vt100::Screen as a series of ScrollbackChunk messages, terminated
/// by a single ScrollbackDumpComplete sentinel.
///
/// The session-host takes an atomic vt100::Screen snapshot on Attach, then
/// IMMEDIATELY resumes forwarding live PtyBytes — it does NOT pause during
/// the dump transfer. The TUI MUST buffer incoming PtyOutput for this session
/// in pending_pty_bytes while dump_in_progress is true, replay the buffer
/// through the freshly-reset parser after ScrollbackDumpComplete (in receipt
/// order), then clear the buffer and process subsequent PtyOutput normally.
/// See §Interleaving for the full snapshot-then-resume protocol.
///
/// Framing invariant: each ScrollbackChunk message MUST fit within the
/// 256 KiB per-message limit (BC-2.01.003). The session-host chunks rows
/// to respect this limit. Typical scrollback (80-col × 1000 rows × ~40
/// bytes/cell JSON) is ~3.2 MB — chunked into ~13 messages.
ScrollbackChunk {
    session_id: String,
    /// Row-major styled-cell data (Vec<Vec<SerializedCell>>; see
    /// SS-session-manager.md §Screen-state transfer). Rows in this chunk,
    /// ordered oldest-to-newest (continuing from the previous chunk).
    rows: Vec<Vec<crate::ipc::SerializedCell>>,
    /// Chunk sequence number (0-indexed). Used by the TUI to detect
    /// out-of-order or dropped chunks (if sequence is non-contiguous, TUI
    /// logs WARN and requests re-attach to restart the dump).
    chunk_seq: u32,
},

/// Sentinel that terminates a scrollback dump sequence.
/// After receiving this, the TUI: (1) resets pty_parsers[session_id] and
/// reconstructs the screen from the accumulated ScrollbackChunk rows;
/// (2) replays pending_pty_bytes (PtyOutput buffered during the dump, per
/// §Interleaving) through the freshly-reset parser in receipt order;
/// (3) clears the buffer and sets dump_in_progress = false. Subsequent
/// PtyOutput messages are processed normally. The session-host is NOT
/// paused during the dump — live PtyBytes continue to flow; see §Interleaving.
ScrollbackDumpComplete {
    session_id: String,
    /// Total number of chunks sent (for integrity validation on the TUI side).
    total_chunks: u32,
    /// Cursor position at the time the dump was taken.
    cursor_row: u16,
    cursor_col: u16,
    /// PTY dimensions at the time of the dump.
    pty_rows: u16,
    pty_cols: u16,
},

/// PTY byte-sequence integrity reset. Sent by the daemon when the
/// session-host sends HostToDaemon::PtyReset (see SS-session-manager.md
/// §PTY reader thread). The TUI must reset the vt100::Parser for this
/// session and re-attach (triggering a new ScrollbackChunk* + ScrollbackDumpComplete
/// sequence from the session-host).
PtyReset {
    session_id: String,
},

// ── ClientToServer (new variants in v1A) ──────────────────────────────────

KeyInput {
    session_id: String,
    bytes: Vec<u8>,      // terminal-encoded key bytes (see SS-09 §Keyboard Encoding)
},
ResizePane {
    session_id: String,
    rows: u16,
    cols: u16,
},
SpawnSession {
    opts: SpawnOptions,  // spawn intent parameters from TUI; daemon builds SpawnRecipe daemon-side
                         // (I27-001 Model A: spawn_recipe() runs in SessionManager::spawn_session())
},
KillSession {
    session_id: String,
},
DetachSession {
    session_id: String,
},
RenameSession {
    session_id: String,
    new_name: String,
},
```

All new variants are added to the existing `#[non_exhaustive]` enums — no breaking change
to existing consumers. Consumers using `..` wildcard matches on `ServerToClient` or
`ClientToServer` will silently ignore new variants at runtime; this is correct behavior for
the forward-compatibility model (BC-2.02.003).

### Interleaving of live PtyOutput during a ScrollbackDump transfer (I3-003 fix)

When a daemon attaches to a session-host (on first spawn or re-discovery), the session-host
begins streaming ScrollbackChunk messages. During this multi-message window, the session-host's
PTY reader may be producing new bytes concurrently.

**RETIRED: Buffer-then-apply-after-Complete (session-host pauses PtyBytes).**

The prior protocol required the session-host to pause `HostToDaemon::PtyBytes` for the entire
dump transfer duration. This is unbounded for large scrollbacks on slow consumers: if the dump
takes > 1s (10k-row scrollback over a congested UDS), the session-host's 1024-entry PTY reader
channel fills and `.send().await` backpressure stalls the harness child's PTY output. This is
the I3-003 harness stall finding.

**Current protocol: Snapshot-then-resume; TUI buffers live PtyOutput during dump.**

1. On `DaemonToHost::Attach`: the session-host atomically snapshots the current `vt100::Screen`
   state (styled cells, cursor, PTY dimensions) into a `Vec<Vec<SerializedCell>>` capture.
2. **The session-host IMMEDIATELY resumes forwarding `HostToDaemon::PtyBytes`** after taking
   the snapshot — it does NOT pause PtyBytes for the dump transfer.
3. The session-host streams `ScrollbackChunk*` + `ScrollbackDumpComplete` from the snapshot
   asynchronously alongside live `PtyBytes`.
4. The TUI, on receiving `ScrollbackDumpComplete`:
   a. Resets the parser from the accumulated snapshot cells.
   b. Replays any buffered `ServerToClient::PtyOutput` messages received during the dump
      through the freshly-reset parser (in receipt order).
   c. Discards the PtyOutput buffer. Processes subsequent PtyOutput messages normally.
5. No double-counting: the parser was reset before the snapshot was applied. No stall: the
   session-host PTY reader is never blocked by the dump transfer.

**TUI PtyOutput buffer during dump:**
The TUI MUST maintain a per-session `dump_in_progress: bool` flag and a `pending_pty_bytes:
Vec<Vec<u8>>` buffer. While `dump_in_progress`, incoming `PtyOutput` for that session is
appended to `pending_pty_bytes` instead of fed to the parser. On `ScrollbackDumpComplete`,
`dump_in_progress = false`, all pending bytes are replayed, buffer cleared.

**Bound:** A typical dump completes in < 500ms for a 10k-row scrollback. At 1 MB/s PTY output
that is at most ~500 KB of buffered live bytes — well within the per-session memory budget.

**Benchmark gate addition:** The pre-v1A benchmark (§Throughput sizing, benchmark gate section)
MUST additionally verify that a 10k-row max scrollback dump completes while the harness child
produces 1 MB/s PTY output with zero PTY channel drops. This validates the non-stall property.

**Interaction with O3-004 buffer sizing:** The snapshot-then-resume protocol means that
ScrollbackChunk messages and live PtyOutput messages are now interleaved in the per-client
channel. The per-client channel capacity is 64 (see §Cross-Client buffer sizing O3-004 fix
below) — adequate for the mixed stream.

## Cross-Client / Cross-Session Backpressure Isolation (I2-003)

The shared UDS channel design (Option A) introduces a cross-client backpressure livelock
risk that the original head-of-line analysis did not address: one slow-but-alive TUI client
can, via end-to-end `.send().await` backpressure, block the PTY reader syscall even for
sessions the slow client is not displaying.

**End-to-end backpressure chain (with single broker buffer):**

```
PTY read syscall
  → spawn_blocking thread: channel.send().await (blocks if full)
  → session-host async event loop: PtyBytes → HostToDaemon
  → daemon session-host proxy: broker.send(Event::PtyOutput)
  → broker fan-out: iter over all clients
      → client_A.send().await  ← if Client A's UDS write buffer is full,
                                   this .await blocks the broker task,
                                   which blocks ALL other clients and ALL
                                   sessions sharing the same broker task.
```

This means: Client A watching Session 1 being slow causes Session 2's PTY reader to stall,
stalling Session 2's harness child — even though Client B (watching Session 2) is healthy.

**Required fix: per-client isolated bounded send buffer.**

Each connected TUI client gets an owned `mpsc::Sender<ServerToClient>` with a bounded
channel (capacity = `CLIENT_SEND_BUFFER_SIZE`, default 64 messages). The broker's fan-out
task sends into each client's per-client channel via `.try_send()` (non-blocking). If
`.try_send()` returns `Err(Full)`, the broker increments a per-client `slow_client_counter`
and (after a configurable threshold, default = 3 consecutive full-send attempts) disconnects
the client (per BC-2.05.004 EC-005 semantics). A separate per-client writer task drains the
channel to the UDS write socket using `.send().await` (backpressure is now contained inside
the per-client writer task, not the broker fan-out task).

This design guarantees that a slow or stalled TUI client's backpressure does NOT reach the
shared broker, the session-host proxy, or the PTY reader. The PTY reader's only source of
backpressure is the durable per-session ring/buffer in the session-host, not any individual
client's render rate.

**Cross-session isolation:** Each session-host proxy task owns its PTY reader independently.
The broker fan-out for Session 1's PtyOutput and Session 2's PtyOutput are dispatched
asynchronously into each client's per-client buffer. A slow client watching Session 1 does
not affect Session 2's PTY reader because the broker fan-out task returns immediately (via
`.try_send()`) for both sessions.

**Per-client buffer sizing rationale (O3-004 fix — corrected from prior 256-message claim):**

The prior sizing stated "256 × 4096 bytes = 1 MiB per client per session." This was wrong:
the same channel carries `ScrollbackChunk` messages up to 256 KiB each. At 256 messages ×
256 KiB = 64 MiB per session — far outside the memory budget.

**Corrected capacity: `CLIENT_SEND_BUFFER_SIZE = 64` messages.**

Worst case (all ScrollbackChunks at 256 KiB): 64 × 256 KiB = 16 MiB per client.
For 4 clients: 64 MiB — within budget.
Practical case (mixed messages, average ~16 KiB): 64 × 16 KiB = 1 MiB per client.

With the I3-003 snapshot-then-resume protocol, the peak chunk pressure is reduced because
ScrollbackChunk and live PtyOutput are interleaved; the session-host is never producing
only ScrollbackChunk messages for the full channel capacity.

The disconnect threshold (3 consecutive full sends) gives slow clients brief leeway for
render pauses without disconnecting on transient lag spikes.

**ADR-0010 benchmark target update:** The cross-client backpressure benchmark MUST validate
that with `CLIENT_SEND_BUFFER_SIZE = 64`, Client B watching Session 2 receives no throughput
reduction when Client A (watching Session 1) is stalled, even during a concurrent 10k-row
scrollback dump to Client A.

**Implementation location:** `monocle-runtime/src/broker.rs` fan-out task.
**Benchmark gate constraint:** the pre-v1A benchmark MUST validate that the per-client
buffer design eliminates the cross-client backpressure path. The benchmark success criterion
for the cross-client case: Client B watching Session 2 receives no throughput reduction when
Client A (watching Session 1) is stalled.

**BC sync required (flag for product-owner):** BC-2.05.009 Postcondition 1b and Invariant 2
reference the broker fan-out model. Product-owner must update BC-2.05.009 to reflect the
per-client isolated buffer design: the broker fan-out uses `.try_send()` into per-client
channels (not `.send().await` directly to the UDS socket), slow clients are disconnected at
the per-client buffer threshold, and the PTY reader's backpressure source is the durable
session ring, not any TUI client. Specific BC-2.05.009 edits required:
- PC-1b: "daemon proxy task posts `Event::PtyOutput` to the broker; broker fans out to each
  client's per-client send buffer via `.try_send()`; a dedicated per-client writer task drains
  the buffer to the UDS socket".
- Invariant 3: revise backpressure description: "The PTY reader channel's backpressure source
  is the durable session-host ring, NOT individual TUI client render rate. Per-client isolated
  send buffers (capacity 64) decouple each client's consumption rate."
- Add new Invariant: "Each TUI client has an owned bounded send buffer (capacity 64). A slow
  client is disconnected after 3 consecutive full-buffer `.try_send()` failures (per
  BC-2.05.004 EC-005). This isolation guarantees zero cross-client backpressure."
- EC-272: update to reference per-client buffer mechanism.

## ADR Cross-References

- Extends: SS-ipc.md §Message Types (new variants documented in SS-05 delta).
- Requires: SS-08 Session Manager (session-host proxy that posts to per-session PTY channel).
- Pre-gate benchmark deliverable: routes to `vsdd-factory:performance-engineer`.

## §Trace v1.6.0

**I27-001 — `ClientToServer::SpawnSession` payload corrected: `recipe: SpawnRecipe` → `opts: SpawnOptions`** (2026-06-13):

- **Finding (I27-001):** The `SpawnSession` variant in the §IPC Message Type Additions code block carried `recipe: SpawnRecipe` with the comment "serialized spawn recipe from `ClaudeCodeModule::spawn_recipe()`". Under the correct Model A architecture (`spawn_recipe()` runs daemon-side inside `SessionManager::spawn_session()`), the wire payload must be `SpawnOptions` (user intent parameters), not a pre-built `SpawnRecipe`. This is one of three occurrences of the I27-001 contradiction (the others are in SS-ipc.md and SS-daemon-wiring-v2-delta.md).
- **Fix:** `recipe: SpawnRecipe` → `opts: SpawnOptions` with updated comment: "spawn intent parameters from TUI; daemon builds `SpawnRecipe` daemon-side (I27-001 Model A: `spawn_recipe()` runs in `SessionManager::spawn_session()`)".
- Semver: minor (v1.5.0 → v1.6.0) — normative wire-type correction.

## §Trace v1.5.0

**I-P9-001 — Pass-9 I3-003 propagation residue: retired pause-during-dump model removed from variant doc-comments** (2026-06-03):

- **ScrollbackChunk doc-comment (lines ~179-182):** The doc-comment carried the RETIRED
  pause-during-dump / wait-to-render model: "The TUI MUST NOT begin rendering PTY bytes for
  this session until it has received ScrollbackDumpComplete. After Complete, the TUI discards
  the dump and switches to streaming PtyOutput messages (the session-host resumes streaming
  from the live PTY after sending Complete)." This contradicted ADR-0010 §Interleaving's own
  canonical snapshot-then-resume protocol (lines ~256-297) and SS-ipc.md §286-318.
  Rewritten to: session-host takes atomic snapshot then IMMEDIATELY resumes live PtyBytes (no
  pause); TUI MUST buffer incoming PtyOutput in pending_pty_bytes while dump_in_progress is
  true; replay buffer through freshly-reset parser after ScrollbackDumpComplete in receipt
  order; then clear buffer and process normally. Cross-reference to §Interleaving added.
- **ScrollbackDumpComplete doc-comment (lines ~200-203):** The doc-comment described only
  "applies the accumulated rows, resets, reconstructs, switches to live streaming" — omitting
  the mandatory pending_pty_bytes replay step and implicitly suggesting the session-host was
  paused (no mention of buffered live bytes). Rewritten to the three-step TUI procedure:
  (1) reset parser and reconstruct from snapshot rows; (2) replay pending_pty_bytes through
  freshly-reset parser in receipt order; (3) clear buffer and set dump_in_progress = false.
  Explicit note added: session-host is NOT paused during the dump. Cross-reference to
  §Interleaving added.
- **Comprehensive sweep result:** All other architecture files inspected for normative
  pause/wait-to-render survivors. SS-session-manager.md lines ~717-720 and ~793 already use
  correct "IMMEDIATELY resumes" language. SS-daemon-wiring-v2-delta.md lines ~455 and ~561
  correctly describe the new protocol in context of the I3-003 fix narrative (legitimate).
  SS-session-manager.md line ~980 "wait up to 5s for ScrollbackDumpComplete" is the daemon
  attach-handshake timeout (operational step, not a rendering pause — legitimate).
  SS-engine-module.md line ~448 "paused awaiting user decision" is unrelated (permission
  overlay state). SS-daemon-wiring-impl.md line ~1389 "pause before writing body" is HTTP
  testing prose, unrelated. I-P9-001 confirmed as the only live normative survivor.

## §Trace v1.4.0

**I5-002 — Pass-5 stale channel-sizing reconciled with MAX_MESSAGE_BYTES** (2026-06-03):

- **I5-002 (§Throughput sizing arithmetic corrected):** The original text stated "each message
  carries at most 4096 bytes of PTY output" and derived "4 MiB per session" from 1024 × 4096.
  This is inconsistent: the protocol allows `PtyOutput` messages up to `MAX_MESSAGE_BYTES`
  (256 KiB), and BC-2.05.009 EC-273 sends 64 KiB single-chunk messages. The 4096-byte
  per-message assumption understated the maximum backlog by 64×.
  - Resolution: §Throughput sizing rewritten to distinguish *typical* (4 KiB/message →
    4 MiB typical backlog) from *maximum* (256 KiB/message → 256 MiB worst-case backlog).
    The typical figure matches observed PTY read sizes; the maximum figure follows from
    `MAX_MESSAGE_BYTES`. In practice the worst case is never reached because no PTY read
    syscall fills a 256 KiB kernel buffer.
  - The benchmark gate framing is updated to reference both typical and burst rates
    (up to 1 MB/s per the scrollback-dump benchmark already defined in §Benchmark gate).
  - The 1024-message channel capacity and the 60Hz / ≤8-session benchmark targets are unchanged.

## §Trace v1.3.1

**HIGH-004 — Adversarial Pass 4 residue: normative 256-message default corrected to 64** (2026-06-03):

- **HIGH-004a (normative paragraph, line ~310):** §Cross-Client / Cross-Session Backpressure
  Isolation — the introductory paragraph describing the per-client bounded channel stated
  "default 256 messages." This contradicted the O3-004 correction already recorded in §Trace
  v1.3.0. Corrected to "default 64 messages."
- **HIGH-004b (BC sync block, lines ~370-371):** The "BC sync required (flag for product-owner)"
  edit block instructed product-owner to write "capacity 256" in both Invariant 3 and the new
  Invariant. These two occurrences are the downstream injection source that re-introduced 256
  in PO-edited BCs. Both corrected to "capacity 64." The O3-004 correction is now consistent
  across all three locations in this document: §Trace narrative (v1.3.0), normative paragraph,
  and PO edit instructions.

## §Trace v1.3.0

**I3-001/I3-003/O3-004 — Adversarial Pass 3 resolution** (2026-06-03):

- **I3-001 (ordering guarantee mechanism):** The prior BC-2.08.008 Invariant 4 rationale
  stated ordering was guaranteed by "holding the SessionManager mutex." Corrected: the mutex
  provides the atomicity window for both posts, but the channel FIFO order is the actual
  ordering guarantee. Added ordered-pair split behavior: if `try_send(SessionStateChanged)`
  succeeds but `try_send(SessionListUpdate)` fails (channel full), the client is immediately
  disconnected (split pair = 2 consecutive full-buffer failures; treated as threshold exhausted).
  Rationale: half-pair delivery leaves TUI in inconsistent state. This is the correctness-
  preferred outcome over tolerating a half-pair and hoping the TUI handles it.
- **I3-003 (dump-pause → harness stall):** §Interleaving protocol RETIRED and REPLACED.
  "Buffer-then-apply-after-Complete (session-host pauses PtyBytes)" is retired. New protocol:
  "Snapshot-then-resume; TUI buffers live PtyOutput during dump." Session-host resumes PtyBytes
  immediately after snapshot. TUI maintains per-session `dump_in_progress` flag and
  `pending_pty_bytes` buffer; replays after ScrollbackDumpComplete. Harness child is never
  stalled. Benchmark gate extended: must verify 10k-row dump completes with 1 MB/s PTY output
  and zero PTY channel drops. Max-scrollback attach pause ADR benchmark target: complete the
  10k-row dump in < 500ms wall-clock (client-side replay included).
- **O3-004 (buffer sizing corrected):** `CLIENT_SEND_BUFFER_SIZE` corrected from 256 to 64.
  Prior "256 × 4096 = 1 MiB" calculation ignored that ScrollbackChunk can be 256 KiB. Correct
  worst-case: 64 × 256 KiB = 16 MiB per client; 4 clients = 64 MiB. Within budget. Benchmark
  must validate 64-message capacity is sufficient under the 10k-row dump + live PtyOutput load.

## §Trace v1.2.0

**C2-002 + I2-003 — Missing ServerToClient variants + cross-client backpressure isolation** (2026-06-03):
- **C2-002 (BLOCKING):** Added missing `ServerToClient` variants: `ScrollbackChunk`,
  `ScrollbackDumpComplete`, and `PtyReset`. These were referenced by the C5 scrollback and
  C3 PtyReset protocols in SS-session-manager.md but absent from the IPC message type table.
  `ScrollbackDump` in SS-session-manager.md §Per-session UDS protocol is a `HostToDaemon`
  variant (session-host → daemon); the `ServerToClient` direction to the TUI uses
  `ScrollbackChunk` + `ScrollbackDumpComplete` for chunked streaming. Field schemas, framing
  constraints, and chunk integrity semantics defined. Interleaving protocol specified:
  buffer-then-apply-after-Complete (session-host pauses live PtyBytes during dump).
- **I2-003 (BLOCKING):** Cross-client / cross-session backpressure livelock analyzed and
  resolved. Per-client isolated bounded send buffer (capacity 256) with `.try_send()` in
  broker fan-out and dedicated per-client writer tasks. PTY reader backpressure source is
  the durable session-host ring only. Slow-client disconnect threshold retained from
  BC-2.05.004. BC-2.05.009 sync flagged to product-owner (exact edits specified).

## §Trace v1.1.0

**C3/I8 — benchmark gate timing + head-of-line analysis** (2026-06-03):
- Benchmark gate moved from "before v1A launch" to "BEFORE v1A story wave begins" —
  it must unblock implementation (or force Option B) before any SS-08/SS-09 work starts.
- Head-of-line blocking analysis added: broker MUST prioritize hook events over PTY bytes
  via `biased;` or dual-channel design. Benchmark must confirm priority effectiveness.
- PTY reader redesigned in SS-session-manager.md to use `.send().await` (backpressure);
  `pty_drop_counter` is now a separate metric from the hook-event drop counter (BC-2.04.011).

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- ADR-0010 authored to resolve Q-1 in the architecture delta.
- Option A selected; pre-gate benchmark deliverable defined for performance-engineer.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
