---
document_type: adr
level: L3
adr_id: "ADR-0010"
title: "PTY Bytes Shared on Existing UDS IPC Channel (Option A)"
status: accepted
producer: vsdd-factory:architect
phase: v1A-architecture-delta
version: "1.1.0"
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
// ServerToClient (new variants in v1A)
PtyOutput {
    session_id: String,  // matches session_id throughout the codebase
    bytes: Vec<u8>,      // raw PTY output bytes; NOT pre-decoded
},
SessionStateChanged {
    session_id: String,
    new_state: SessionState,
},

// ClientToServer (new variants in v1A)
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

## ADR Cross-References

- Extends: SS-ipc.md §Message Types (new variants documented in SS-05 delta).
- Requires: SS-08 Session Manager (session-host proxy that posts to per-session PTY channel).
- Pre-gate benchmark deliverable: routes to `vsdd-factory:performance-engineer`.

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
