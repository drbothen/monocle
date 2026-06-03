---
document_type: adr
level: L3
adr_id: "ADR-0010"
title: "PTY Bytes Shared on Existing UDS IPC Channel (Option A)"
status: accepted
producer: vsdd-factory:architect
phase: v1A-architecture-delta
version: "1.0.0"
timestamp: 2026-06-03T23:00:00Z
inputs:
  - research/domain-monocle-vision-synthesis.md
  - specs/product-brief.md
  - specs/research/embedded-pty-evaluation.md
  - specs/architecture/SS-ipc.md
input-hash: "7e4f4f4"
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

**Pre-v1A gate benchmark deliverable:** Before the v1A launch gate, the performance-engineer
MUST benchmark PTY byte delivery at terminal-refresh rates across N concurrent sessions on
the shared UDS channel, confirming that:
1. The drop counter does NOT fire under normal terminal refresh load (target: 0 drops at
   session count ≤ 8 at 60Hz display refresh).
2. Hook event delivery latency does not exceed 50ms during concurrent PTY streaming.
3. The `≥ 1000 events/sec` CLAUDE.md convention is satisfied end-to-end.

If the benchmark reveals that Option A is insufficient (drop counter fires consistently, or
hook latency exceeds 50ms under realistic load), the architect must be re-engaged to evaluate
Option B before v1A ships. This is a pre-gate verification deliverable, not an open design
question.

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

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- ADR-0010 authored to resolve Q-1 in the architecture delta.
- Option A selected; pre-gate benchmark deliverable defined for performance-engineer.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
