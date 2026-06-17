---
document_type: behavioral-contract
level: L3
version: "1.5.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-ipc.md, architecture/SS-daemon-wiring-v2-delta.md, architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md]
input-hash: "c0e44ba"
traces_to: prd.md
origin: greenfield
subsystem: SS-05
capability: CAP-005
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1A
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.05.009: PtyOutput Fan-Out — Per-Session Bounded Channel (1024) with Drop Counter (stderr WARN) + PtyReset TUI Recovery

## Description

When a `monocle-session-host` sends `HostToDaemon::PtyBytes { bytes }`, the daemon's
session-host proxy task posts the bytes to the broker as `Event::PtyOutput { session_id, bytes }`.
The broker fan-out sends `ServerToClient::PtyOutput { session_id, bytes }` to all connected
TUI clients. The PTY reader channel inside the session-host is bounded at capacity 1024.
When the channel is full the PTY reader thread BLOCKS (backpressure to the PTY read
syscall) — it does NOT drop bytes. The `pty_drop_counter` counts only sender-error /
OOM / receiver-gone conditions (unreachable under normal operation); it is NOT incremented
on normal channel fullness. Under normal backpressure, PTY bytes are never dropped.

## Preconditions

1. A session-host is running and connected to the daemon.
2. At least one TUI client is connected to the daemon's UDS.
3. The session-host's PTY reader is producing bytes.

## Postconditions

1. For each `HostToDaemon::PtyBytes { bytes }` received from the session-host:
   a. The daemon proxy task posts `Event::PtyOutput { session_id: session_id.clone(), bytes }` to
      the broker.
   b. The broker fan-out sends `ServerToClient::PtyOutput { session_id, bytes }` to ALL
      connected TUI clients via each client's per-client isolated send buffer (capacity 64
      messages; see Invariant 3b). The broker uses `.try_send()` into each client's
      `mpsc::Sender<ServerToClient>`. A dedicated per-client writer task drains the channel
      to the UDS socket. Slow clients are isolated — a stalled client does NOT apply
      backpressure to other clients or to the PTY reader.
2. The per-session UDS connection has a bounded mpsc channel (capacity 1024) between the
   PTY reader blocking thread and the session-host async event loop. When the channel is
   full, the PTY reader thread BLOCKS on `.send().await` (backpressure propagates to the
   PTY read syscall) — it does NOT drop bytes and does NOT increment the `drop_counter`.
   The `drop_counter` is incremented only on sender errors (receiver gone / OOM), which
   are unreachable conditions under normal operation.
3. The `drop_counter` is NOT surfaced in the TUI status bar (session-host has no TUI). It
   is logged to the session-host's stderr at `WARN` level:
   `WARN: PTY channel drop #N for session <session_id>`.
4. The broker fan-out for `PtyOutput` follows the same fan-out semantics as
   `HookEventReceived` (BC-2.05.004): slow TUI clients are disconnected; other clients continue.
5. `ServerToClient::PtyOutput` is framed with the standard 4-byte LE length-prefix protocol
   per SS-ipc.md.

## Invariants

1. PTY bytes are forwarded verbatim — no truncation, no excerpt bounding. Unlike
   `HookEventReceived` which truncates POST bodies to 256 bytes, `PtyOutput` carries
   raw terminal bytes that MUST be preserved in full (vt100 sequences would be corrupted
   by truncation). **No silent drops permitted for PTY bytes** (C3 fix: production-grade
   principle violation to silently drop a mid-CSI byte).
2. Fan-out is to ALL TUI clients, not just the client currently displaying the session.
   This supports future multi-TUI scenarios and ensures background sessions can be
   monitored by connecting a second TUI instance.
3. The session-host's PTY reader channel (capacity 1024) uses `.send().await` (backpressure),
   NOT `.try_send()` (drop). Backpressure propagates from the durable session ring up through
   the daemon broker → session-host proxy → session-host async event loop → PTY reader
   `spawn_blocking` thread. The `pty_drop_counter` counts channel sender errors (receiver
   gone), not overflow drops. Under normal backpressure, PTY bytes are never dropped.
   **Backpressure source is the durable session ring (NOT TUI clients)**: the durable ring's
   write path is the upstream backpressure signal; a slow TUI client applies backpressure only
   within its own per-client send buffer and does NOT propagate backpressure to the PTY reader.

3b. **Per-client send buffer isolation:** Each connected TUI client has a dedicated
   `mpsc::channel::<ServerToClient>(64)` (capacity 64, per SS-ipc.md v1.24.0 §TUI IPC Read
   Loop Pattern canonical pattern — rationale: 64 covers typical burst sizes without unbounded
   memory growth; 64×256KiB=16MiB maximum in-flight per client). The broker uses `.try_send()`
   into the per-client channel (NOT `.send().await`). A dedicated per-client writer task drains
   this channel to the UDS socket via `.write_all().await`. Disconnection threshold: after 3
   consecutive full-buffer `.try_send()` failures for the same client, the broker disconnects
   that client and logs `WARN: slow TUI client disconnected`. Other clients are unaffected by
   the disconnected client's send-buffer pressure. This is the per-client backpressure
   isolation model per SS-daemon-wiring-v2-delta.md v1.11.4 §5d.
4. **Forced parser-reset protocol on ANY PTY drop:** If a PTY byte is ever dropped (sender
   error, OOM, other extreme condition), the session-host sends `HostToDaemon::PtyReset`.
   The daemon propagates `ServerToClient::PtyReset { session_id }` to all TUI clients.
   Each TUI client resets `pty_parsers[session_id]` and sends `ClientToServer::AttachSession`
   to trigger a fresh `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence (re-attach).
   The retired single-message `ScrollbackDump` form MUST NOT be requested or expected.
5. **TUI-surfaced PtyReset indicator:** On `PtyReset` receipt, the TUI status bar MUST
   display `[PTY reset — <session_id truncated>]` for 5 seconds. Silent terminal corruption
   is never acceptable.
6. `Event::PtyOutput` is a new event type in the broker. It does NOT increment the hook
   event drop counter (BC-2.04.011). The session-host's own `pty_drop_counter` is a
   separate metric. The broker MUST give hook events priority over PtyOutput in its fan-out
   `tokio::select!` (biased or dual-channel) per ADR-0010 §Head-of-line blocking mitigation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-270 | Session-host PTY produces bytes faster than daemon can consume (channel at capacity 1024) | PTY reader thread BLOCKS on `.send().await` (backpressure to PTY read syscall); drop_counter NOT incremented; no bytes lost; throughput limited by consumer speed; no crash |
| EC-271 | No TUI clients connected | Bytes posted to broker; broker fan-out has no subscribers; bytes discarded by broker; no error |
| EC-272 | TUI client's per-client send buffer full (slow TUI) | After 3 consecutive full-buffer `.try_send()` failures for this client, the broker disconnects the client. The per-client send buffer (capacity 64; 64×256KiB=16MiB maximum) is isolated: its overflow does NOT propagate to the PTY reader or to other clients. Other clients continue to receive PtyOutput messages uninterrupted. |
| EC-273 | Large `PtyBytes` chunk (e.g., 64 KiB of output) | Sent as a single `ServerToClient::PtyOutput` IPC message; 256 KiB message size limit per BC-2.01.003; 64 KiB is within limit |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| Session-host sends 10 `PtyBytes` messages; 2 TUI clients connected | Each TUI client receives 10 `PtyOutput` messages with correct bytes | happy-path |
| PTY channel at capacity (1024 messages): new read arrives | PTY reader thread BLOCKS on `.send().await`; no bytes dropped; drop_counter remains 0; send completes once consumer drains at least one slot | edge-case |
| `PtyBytes` arrives when no TUI clients connected | Bytes discarded by broker; no error | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `PtyOutput` fan-out to all connected TUI clients on PtyBytes receipt | integration |
| VP-TBD | Bytes NOT truncated (full PtyBytes content in PtyOutput) | unit |
| VP-TBD | PTY reader BLOCKS (no drop) when channel at capacity; drop_counter remains 0 on normal fullness | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability §SS-05 |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability — PtyOutput fan-out extends the session/event/prompt push capability of CAP-005 with real-time PTY byte streaming, which is transported over the same shared UDS per ADR-0010 |
| Architecture Module | monocle-ipc (`ServerToClient::PtyOutput` variant); monocle-runtime (session-host proxy task, broker fan-out) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-daemon-wiring-v2-delta.md v1.11.4 §broker fan-out — PtyOutput messages; ADR-0010 v1.6.0 §pty-bytes-over-shared-uds-ipc; SS-session-manager.md v2.6.1 §PTY reader thread; SS-ipc.md v1.24.0 §TUI IPC Read Loop Pattern (per-client channel capacity 64, rationale) |
| Cross-Ref | BC-2.05.004 (fan-out semantics for slow-client disconnect); BC-2.04.011 (hook event drop counter — separate from PTY channel drop counter) |
| Test Name | test_BC_2_05_009_pty_output_fan_out_bounded_channel |

## Related BCs

- [BC-2.05.004] — composes with: same fan-out semantics (slow client disconnect, no connected clients)
- [BC-2.09.001] — depends on: PtyOutput received by TUI triggers the 100ms render pipeline

## Architecture Anchors

- `architecture/SS-daemon-wiring-v2-delta.md#broker-fan-out-ptyoutput-messages` — PtyOutput event type
- `architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md` — shared UDS decision + bounded channel 1024

## Story Anchor

S-046 — Implement PtyOutput broker fan-out and session-host PTY reader bounded channel

## VP Anchors

VP-TBD — PtyOutput fan-out integration tests (filled after VP creation)

## §Trace v1.5.3

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-046** (2026-06-15):
- Story Anchor filled from Phase-2 Burst C story decomposition. No behavioral content changed.

## §Trace v1.5.2

**S35-001 (no body change — no matching framing) + arch-source pin v1.9.0→v1.9.1** (2026-06-13 / D-277):
- S35-001 sweep: searched BC-2.05.009 for "equivalent to exhausting the 3-strike threshold"
  or similar ordered-pair-split language. NOT FOUND. The "3 consecutive full-buffer
  `.try_send()` failures" text in Invariant 3b and EC-272 describes the correct normal
  slow-client disconnection policy — this is NOT the split-pair path and requires no change.
- Arch-source pin: SS-daemon-wiring-v2-delta.md v1.9.0 → v1.9.1 (all active citations).
- Patch bump: 1.5.1 → 1.5.2.

## §Trace v1.5.1

**S20-001 Phase-1d Pass-20 fix — H1 title precision (title↔body agreement)** (2026-06-04):
- Finding: H1 title contained "Surfaced Drop Counter", which a fresh adversary read as the
  `drop_counter` being surfaced in the TUI status bar. PC-3 explicitly contradicts this:
  "The `drop_counter` is NOT surfaced in the TUI status bar (session-host has no TUI). It is
  logged to the session-host's stderr at WARN level." Actual PTY drops surface to the TUI via
  the separate PtyReset 5s indicator (Invariant 5), not via the drop_counter.
- Old title: "PtyOutput Fan-Out — Per-Session Bounded Channel (1024) with Surfaced Drop Counter"
- New title: "PtyOutput Fan-Out — Per-Session Bounded Channel (1024) with Drop Counter (stderr WARN) + PtyReset TUI Recovery"
- Change: title-only precision fix; NO behavioral change. PC-3 wording unchanged. Invariant 5
  wording unchanged. The new title encodes exactly where the counter goes (stderr WARN) and how
  actual drops reach the TUI (PtyReset recovery path).
- Propagated to: BC-INDEX.md line 125; prd.md §2.5 line 173.

## §Trace v1.5.0

**I5-001 Pass-5 fix — resolve DROP vs BACKPRESSURE self-contradiction** (2026-06-03):
- Finding: Description, PC-2, EC-270, and test vector asserted the DROP model (drop_counter
  incremented on channel fullness), contradicting Invariants 1 and 3 which assert the
  BACKPRESSURE model (`.send().await`, never drop). The canonical resolution is BACKPRESSURE
  per SS-session-manager.md §PTY reader thread, which uses `.send().await`.
- Description: removed "if the channel fills, a drop counter is incremented and logged";
  replaced with explicit statement that the PTY reader BLOCKS on `.send().await` when full
  and that `pty_drop_counter` counts only sender-error/OOM conditions.
- PC-2: "PTY reader thread drops the bytes and increments a `drop_counter`" → "PTY reader
  thread BLOCKS on `.send().await` (backpressure)"; clarified that drop_counter is NOT
  incremented on normal channel fullness — only on sender errors (receiver gone / OOM).
- EC-270: "Session-host drops excess reads; drop counter incremented" → "PTY reader thread
  BLOCKS on `.send().await`; drop_counter NOT incremented; no bytes lost".
- Test vector "1025th read → Drop counter = 1" → "channel at capacity → PTY reader BLOCKS;
  no bytes dropped; drop_counter remains 0; send completes once consumer drains a slot".
- Verification Property: "Drop counter incremented on channel overflow" → "PTY reader BLOCKS
  (no drop) when channel at capacity; drop_counter remains 0 on normal fullness".
- All five locations now consistently assert the BACKPRESSURE model. Invariants 1 and 3
  are unchanged (they were already correct).

## §Trace v1.4.0

**HIGH-003 adversarial pass-4 fix — PC-1b per-client send buffer capacity 256 → 64** (2026-06-03):
- PC-1b: "capacity 256 messages" → "capacity 64 messages". This was a propagation residue from
  the v1.2.0 introduction (which correctly stated 64 in Invariant 3b and EC-272 but missed PC-1b).
  The canonical value is 64 per SS-ipc.md v1.13.0 §TUI IPC Read Loop Pattern; Invariant 3b,
  EC-272, and the Architecture Source have all stated 64 since v1.3.0.

## §Trace v1.3.0

**Adversarial Pass 3 fixes — O3-004 (per-client buffer capacity 64) + Invariant 4 (ScrollbackChunk*/Complete message names)** (2026-06-03):
- O3-004 (I3-004 related): Per-client send buffer capacity corrected from 256 to **64** in
  Invariant 3b and EC-272. The canonical capacity is 64 per SS-ipc.md v1.13.0 §TUI IPC Read
  Loop Pattern (rationale: 64 covers typical burst sizes; 64×256KiB=16MiB maximum per client).
  The prior value of 256 was inconsistent with the architect's canonical pattern.
- Invariant 4: "re-fetches ScrollbackDump" replaced with "sends `ClientToServer::AttachSession`
  to trigger a fresh `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence". The retired
  single-message `ScrollbackDump` form MUST NOT be referenced; the chunked protocol is
  canonical per SS-session-manager.md v1.5.0. Also clarified that the TUI sends the
  `ClientToServer::AttachSession` IPC variant (per I3-004 fix in BC-2.05.010 and BC-2.05.011).
- Architecture Source updated to SS-daemon-wiring-v2-delta.md v1.3.1, SS-session-manager.md
  v1.4.0, SS-ipc.md v1.13.0.

## §Trace v1.2.0

**Architect-delegated BC edits — per-client backpressure isolation (SS-daemon-wiring-v2-delta v1.2.0 §5d)** (2026-06-03):
- PC-1b: broker fan-out description updated to reference per-client isolated send buffer
  (capacity 256, `.try_send()`, dedicated writer task). Clarified that slow clients are
  isolated and do NOT propagate backpressure to the PTY reader or other clients.
- Invariant 3: "Backpressure source is the durable session ring (NOT TUI clients)" added as
  explicit statement to prevent confusion between the `.send().await` upstream path and the
  `.try_send()` per-client buffer model.
- Invariant 3b added: per-client send buffer specification (capacity 256, `.try_send()`,
  3-failure disconnect threshold, dedicated writer task). Per SS-daemon-wiring-v2-delta.md
  v1.2.0 §5d (per-client isolated send buffer design).
- EC-272: updated from generic "slow client disconnected" to precise "3 consecutive full-buffer
  failures → disconnect; isolation from other clients".

## §Trace v1.1.0

**C3 architectural resolution — no-silent-drop + backpressure + PtyReset protocol** (2026-06-03):
- Invariant 1 strengthened: no silent drops permitted for PTY bytes.
- Invariant 3 revised: `.send().await` backpressure replaces `.try_send()` drop model.
  `pty_drop_counter` now counts sender errors (extreme conditions only), not normal overflow.
- Invariant 4 added: forced parser-reset protocol on any drop (PtyReset message chain).
- Invariant 5 added: TUI-surfaced PtyReset indicator (5-second status bar message).
- Invariant 6 updated: broker hook-event priority over PtyOutput added per ADR-0010.
- Architecture Source updated to reference SS-session-manager.md v1.2.0.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.05.009 authored for SS-05 as part of the v1A control-center pivot BC burst.
- ADR-0010 channel capacity 1024 from architect spec preserved verbatim.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).

## §Trace v1.5.5

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.5.5 timestamp >= v1.5.4. PASS.
