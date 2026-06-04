---
document_type: adr
level: L3
adr_id: "ADR-0010"
title: "PTY Bytes Shared on Existing UDS IPC Channel (Option A)"
status: accepted
producer: vsdd-factory:architect
phase: v1A-architecture-delta
version: "1.2.0"
timestamp: 2026-06-03T23:00:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - specs/product-brief.md
  - specs/research/embedded-pty-evaluation.md
  - specs/architecture/SS-ipc.md
input-hash: "13e1215"
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

**Throughput sizing:** The per-session PTY output channel capacity is set at **1024 messages**
(each message carries at most 4096 bytes of PTY output). This bounds the in-flight PTY backlog
at 4 MiB per session, which is well within normal operating conditions. Terminal applications
rarely exceed 10–50 KiB/s of text output at human-legible speeds.

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
/// The TUI MUST NOT begin rendering PTY bytes for this session until it has
/// received ScrollbackDumpComplete. After Complete, the TUI discards the
/// dump and switches to streaming PtyOutput messages (the session-host
/// resumes streaming from the live PTY after sending Complete).
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
/// After receiving this, the TUI applies the accumulated rows, resets
/// pty_parsers[session_id], reconstructs the screen, and switches to live
/// streaming (subsequent PtyOutput messages are processed normally).
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
    recipe: SpawnRecipe, // serialized spawn recipe from ClaudeCodeModule::spawn_recipe()
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

### Interleaving of live PtyOutput during a ScrollbackDump transfer

When a daemon attaches to a session-host (on first spawn or re-discovery), the session-host
begins streaming ScrollbackChunk messages. During this multi-message window, the session-host's
PTY reader may be producing new bytes concurrently. The protocol is:

**Buffer-then-apply-after-Complete (mandatory).**

The session-host MUST pause forwarding live `HostToDaemon::PtyBytes` messages to the daemon
for the duration of the scrollback dump sequence (from the moment `Attach` is received until
`ScrollbackDumpComplete` is sent). Concretely:

1. On `DaemonToHost::Attach`: the session-host snapshots the current vt100::Screen state
   (styled cells, cursor, PTY dimensions).
2. The session-host streams `ScrollbackChunk` messages for the snapshot.
3. While streaming chunks, new PTY bytes from the PTY reader continue to accumulate in the
   session-host's PTY reader channel (the channel capacity is 1024; a typical dump completes
   in < 100ms, well within the channel's budget at normal terminal speeds).
4. After `ScrollbackDumpComplete` is sent, the session-host resumes forwarding
   `HostToDaemon::PtyBytes` from where the reader left off.
5. The TUI, after receiving `ScrollbackDumpComplete`, has a correct screen state and any
   subsequent `PtyOutput` messages are applied to the reset parser — no double-counting,
   no interleaving corruption.

**Rationale:** Interleaving live PtyOutput messages DURING a multi-message ScrollbackDump
would require the TUI to implement complex sequencing logic (apply bytes to which parser
state? before or after the dump rows?). Buffer-then-apply-after-Complete is simpler, correct
by construction, and the latency cost (< 100ms during attach) is acceptable. No live streaming
is lost — bytes are buffered in the session-host PTY reader channel during the brief window.

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
channel (capacity = `CLIENT_SEND_BUFFER_SIZE`, default 256 messages). The broker's fan-out
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

**Per-client buffer sizing rationale:**
- 256 messages × 4096 bytes/message (typical max chunk) = 1 MiB per client per session in
  the worst case. For 8 sessions and 4 clients: 32 MiB — within the memory budget.
- The disconnect threshold (3 consecutive full sends) gives slow clients brief leeway for
  render pauses without disconnecting on transient lag spikes.

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
  send buffers (capacity 256) decouple each client's consumption rate."
- Add new Invariant: "Each TUI client has an owned bounded send buffer (capacity 256). A slow
  client is disconnected after 3 consecutive full-buffer `.try_send()` failures (per
  BC-2.05.004 EC-005). This isolation guarantees zero cross-client backpressure."
- EC-272: update to reference per-client buffer mechanism.

## ADR Cross-References

- Extends: SS-ipc.md §Message Types (new variants documented in SS-05 delta).
- Requires: SS-08 Session Manager (session-host proxy that posts to per-session PTY channel).
- Pre-gate benchmark deliverable: routes to `vsdd-factory:performance-engineer`.

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
