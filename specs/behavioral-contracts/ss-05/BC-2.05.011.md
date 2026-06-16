---
document_type: behavioral-contract
level: L3
version: "1.2.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:59:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-ipc.md, architecture/SS-session-manager.md, architecture/SS-daemon-wiring-v2-delta.md, architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md]
input-hash: "3b8b97d"
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

# Behavioral Contract BC-2.05.011: New ServerToClient IPC Variants — ScrollbackChunk, ScrollbackDumpComplete, PtyReset

## Description

v1A adds three new `ServerToClient` IPC message variants for PTY scrollback streaming and
PTY parser reset. The daemon broadcasts these to all connected TUI clients via the existing
per-client isolated send buffer (capacity 64, per SS-daemon-wiring-v2-delta.md §5d). The
TUI accumulates `ScrollbackChunk` messages until it receives `ScrollbackDumpComplete`, then
resets its local `vt100::Parser` and reconstructs the screen. `PtyReset` triggers a fresh
re-attach. These three variants are core to the chunked scrollback protocol that replaces
the retired single-message `HostToDaemon::ScrollbackDump`.

## Preconditions

1. TUI client is connected to the daemon's UDS.
2. The daemon is proxying a session-host via its session-host proxy task.

## Postconditions

### ScrollbackChunk

1. When the daemon receives `HostToDaemon::ScrollbackChunk { rows, chunk_seq }` from the
   session-host, it posts `Event::ScrollbackChunk { session_id, rows, chunk_seq }` to the
   broker.
2. The broker dispatches `ServerToClient::ScrollbackChunk { session_id, rows, chunk_seq }`
   to each connected TUI client via the per-client send buffer (`.try_send()`).
3. The TUI accumulates received chunks in-order. A chunk with a non-contiguous `chunk_seq`
   (i.e., `received_seq != expected_seq`) causes the TUI to log WARN and re-request Attach
   (triggering a fresh scrollback dump).
4. Each `ServerToClient::ScrollbackChunk` message is ≤ 256 KiB serialized (per BC-2.01.003
   body size limit; the session-host enforces this at serialization time).

### ScrollbackDumpComplete

1. When the daemon receives `HostToDaemon::ScrollbackDumpComplete { total_chunks, cursor_row,
   cursor_col, pty_rows, pty_cols }`, it posts `Event::ScrollbackDumpComplete { session_id,
   total_chunks, cursor_row, cursor_col, pty_rows, pty_cols }` to the broker.
2. The broker dispatches `ServerToClient::ScrollbackDumpComplete { session_id, total_chunks,
   cursor_row, cursor_col, pty_rows, pty_cols }` to each connected TUI client.
3. On receipt of `ScrollbackDumpComplete`, the TUI:
   a. Validates that the number of accumulated `ScrollbackChunk` messages equals `total_chunks`.
      If mismatch → log WARN; send `ClientToServer::AttachSession { session_id }` to trigger
      a fresh scrollback dump (TUI MUST NOT send `DaemonToHost::Attach` directly — see Invariant 6).
   b. If count matches → resets the parser:
      `pty_parsers[session_id] = vt100::Parser::new(pty_rows, pty_cols, SCROLLBACK_ROWS)`.
   c. Reconstructs the vt100 screen from the accumulated `SerializedCell` rows (per
      SS-session-manager.md v2.6.1 §Screen-state transfer on Attach reconstruction paths).
   d. Discards the accumulated wire-JSON after reconstruction (transient allocation released).
   e. Resumes processing subsequent `ServerToClient::PtyOutput` messages on the now-correctly
      initialized parser. No double-counting occurs because the parser was reset before
      the dump was applied.

### PtyReset

1. When the daemon receives `HostToDaemon::PtyReset` from the session-host, it posts
   `Event::PtyReset { session_id }` to the broker.
2. The broker dispatches `ServerToClient::PtyReset { session_id }` to all connected TUI clients.
3. On receipt of `PtyReset`, the TUI:
   a. Resets `pty_parsers[session_id] = vt100::Parser::new(rows, cols, SCROLLBACK_ROWS)`
      (fresh parser state — all prior screen state discarded).
   b. Displays `[PTY reset — <session_id truncated to 8 chars>]` in the status bar for 5
      seconds. This surfaces any architectural regression (PTY byte drop) to the operator.
   c. Sends `ClientToServer::AttachSession { session_id }` to the daemon to trigger a fresh
      `ScrollbackChunk*` + `ScrollbackDumpComplete` sequence (I3-004 fix: the TUI sends the
      `ClientToServer::AttachSession` IPC variant — it MUST NOT send `DaemonToHost::Attach`,
      which is a daemon→session-host message that the TUI cannot send). The daemon routes
      `AttachSession` to `SessionManager::attach_session()` which sends `DaemonToHost::Attach`
      to the session-host. NOT re-spawning the session — re-attaching to the existing one.
   d. While waiting for `ScrollbackDumpComplete`, the TUI MUST buffer any `PtyOutput` messages
      received for this session in `pending_pty_bytes[session_id]` (Invariant 6). Live PTY
      bytes continue to arrive during the dump (I3-003 fix: session-host resumes PtyBytes
      immediately after snapshot, does NOT pause for dump transfer).
   e. On `ScrollbackDumpComplete` receipt: validate chunk count, reset parser, reconstruct
      screen from accumulated `SerializedCell` rows, THEN replay all buffered
      `pending_pty_bytes[session_id]` bytes through the now-reset parser in receipt order.
      Clear `pending_pty_bytes[session_id]` after replay. No double-counting: parser was
      reset before dump applied; buffered bytes are post-snapshot live output.
4. `PtyReset` is a rare event — it fires only on an actual PTY byte drop (channel `SendError`,
   OOM, or other extreme condition). Under normal backpressure via `.send().await`, it never
   fires. See SS-session-manager.md v2.6.1 §PTY reader thread §Forced parser-reset protocol.

## Invariants

1. All three variants are `#[non_exhaustive]` per BC-2.02.003 policy and `ADR-0006`.
2. `ScrollbackChunk` and `ScrollbackDumpComplete` are always produced in pairs: the session-host
   MUST send one or more `ScrollbackChunk` messages followed by exactly one
   `ScrollbackDumpComplete`. The daemon fans out each message in order. The TUI MUST NOT
   consider the dump complete until `ScrollbackDumpComplete` is received.
3. The retired `HostToDaemon::ScrollbackDump` single-message form MUST NOT be used. The daemon
   MUST NOT fan out a `ServerToClient::ScrollbackDump` variant. Any session-host sending the
   old `ScrollbackDump` form is treated as protocol-violation; the daemon logs ERROR and
   re-sends `DaemonToHost::Attach` to force a retry.
4. `PtyReset` receipt resets ALL accumulated state for the session: the parser is fresh; any
   partially-accumulated `ScrollbackChunk` messages awaiting `ScrollbackDumpComplete` are
   discarded. The subsequent re-attach produces a fresh scrollback dump.
5. The three variants are dispatched via the same per-client isolated send buffer (capacity 64
   per SS-ipc.md v1.24.0 §TUI IPC Read Loop Pattern) as `PtyOutput` (BC-2.05.009 Invariant 3b).
   A slow TUI client that fills its buffer during a scrollback dump will be disconnected after
   3 consecutive failures (per §5d isolation model).
6. **`dump_in_progress` → buffer live PtyOutput in `pending_pty_bytes`, replay on Complete.**
   When a scrollback dump is in progress (after `ClientToServer::AttachSession` sent; before
   `ScrollbackDumpComplete` received), the TUI MUST buffer any `ServerToClient::PtyOutput`
   messages for the session in `pending_pty_bytes[session_id]` rather than feeding them to
   the parser immediately. On `ScrollbackDumpComplete`: reconstruct screen from dump, THEN
   drain `pending_pty_bytes` through the reconstructed parser in receipt order. This prevents
   live bytes from being applied to a stale parser state during the dump window. Per
   SS-session-manager.md v2.6.1 §Screen-state transfer on Attach step 5e (I3-003 fix).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-285 | `ScrollbackChunk` arrives with non-contiguous `chunk_seq` (e.g., 0, 1, 3 — chunk 2 missing) | TUI logs WARN; discards accumulated chunks for session; sends re-attach trigger; daemon re-sends full dump from session-host |
| EC-286 | `ScrollbackDumpComplete.total_chunks` does not match accumulated chunk count | Same as EC-285 — WARN + re-attach trigger |
| EC-287 | `PtyReset` arrives while a scrollback dump is in progress (incomplete chunk sequence) | TUI discards partial chunks; resets parser; status bar shows `[PTY reset — <id>]` for 5s; re-attach produces fresh dump |
| EC-288 | Per-client send buffer full during scrollback dump (large scrollback, slow TUI) | After 3 consecutive full-buffer failures, broker disconnects client (Invariant 5). TUI reconnects and requests fresh attach |
| EC-289 | `PtyReset` arrives for a session not in `pty_parsers` (e.g., session already GC'd) | TUI ignores the message; no WARN needed (session is gone from TUI state) |

## Canonical Test Vectors

| Scenario | Expected Output | Category |
|----------|----------------|----------|
| 2-chunk scrollback dump | TUI accumulates 2 `ScrollbackChunk` messages; on `ScrollbackDumpComplete{total_chunks:2}` → parser reset + screen reconstruction | happy-path |
| `PtyReset` received while in EmbeddedTerminal | Parser reset; `[PTY reset — <id>]` in status bar for 5s; re-attach triggered | happy-path |
| Chunk count mismatch on `ScrollbackDumpComplete` | WARN logged; re-attach triggered; fresh dump requested | edge-case |
| `ScrollbackChunk` with gap in `chunk_seq` | WARN logged; accumulated chunks discarded; re-attach triggered | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `ScrollbackChunk*` + `ScrollbackDumpComplete` → parser reset + screen reconstruction | integration |
| VP-TBD | Chunk count mismatch → re-attach triggered (no silent corruption) | unit |
| VP-TBD | `PtyReset` → parser reset + 5s status bar indicator + re-attach | unit |
| VP-TBD | Non-contiguous `chunk_seq` → re-attach triggered | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability §SS-05 |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability traceability — the three new ServerToClient variants (ScrollbackChunk, ScrollbackDumpComplete, PtyReset) extend the session/event/prompt push capability with the chunked scrollback dump protocol and PTY reset notification, all transported over the existing shared UDS per ADR-0010 |
| Architecture Module | monocle-ipc (`ServerToClient::ScrollbackChunk`, `ServerToClient::ScrollbackDumpComplete`, `ServerToClient::PtyReset` variants); monocle-runtime (broker fan-out §5b/§5c); monocle-tui (chunk accumulation, parser reset, status bar indicator) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-daemon-wiring-v2-delta.md v1.11.4 §5b (ScrollbackChunk/ScrollbackDumpComplete fan-out; I3-003 resume-after-snapshot); §5c (PtyReset fan-out); SS-session-manager.md v2.6.1 §Screen-state transfer on Attach (step 5d-5e: buffer PtyOutput during dump, replay on Complete); SS-ipc.md v1.24.0 §`ClientToServer::AttachSession` (I3-004 — TUI sends AttachSession not DaemonToHost::Attach); ADR-0010 v1.6.0 §pty-bytes-over-shared-uds-ipc (shared UDS decision + chunked protocol) |
| Cross-Ref | BC-2.05.009 (PtyOutput fan-out; per-client buffer; Invariant 3b — same isolation model); BC-2.08.007 (Attach → triggers ScrollbackChunk* + ScrollbackDumpComplete sequence); BC-2.09.001 (PTY output renders after parser reconstruction completes) |
| Test Name | test_BC_2_05_011_new_server_to_client_scrollback_and_reset_variants |

## Related BCs

- [BC-2.05.009] — composes with: same per-client isolated send buffer (capacity 64, §5d model)
- [BC-2.08.007] — depends on: attach_session() triggers the ScrollbackChunk* + ScrollbackDumpComplete sequence
- [BC-2.09.001] — depends on: PTY output after reconstruction flows through the parser pipeline

## Architecture Anchors

- `architecture/SS-daemon-wiring-v2-delta.md#broker-fan-out-scrollbackchunk-scrollbackdumpcomplete` — §5b
- `architecture/SS-daemon-wiring-v2-delta.md#broker-fan-out-ptyreset` — §5c
- `architecture/SS-session-manager.md#screen-state-transfer-on-attach` — reconstruction protocol
- `architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md` — shared UDS + chunked protocol decision

## Story Anchor

- S-046 — Implement PtyOutput broker fan-out and session-host PTY reader bounded channel
  (owns `ServerToClient::PtyReset` variant definition in monocle-ipc + broker emission in monocle-runtime)
- S-047 — Implement new ClientToServer IPC variants and daemon routing
  (owns `ServerToClient::ScrollbackChunk`, `ServerToClient::ScrollbackDumpComplete` variant definition + broker fan-out, TUI receiver/chunk accumulation/parser reset/reconstruction protocol)

## VP Anchors

VP-TBD — Scrollback dump integration tests and PtyReset unit tests (filled after VP creation)

## §Trace v1.2.2

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-047** (2026-06-15):
- Story Anchor filled from Phase-2 Burst C story decomposition (clusters with BC-2.05.010). No behavioral content changed.

## §Trace v1.2.1

**Arch-source pin v1.9.0→v1.9.1** (2026-06-13 / D-277):
- Arch-source pin: SS-daemon-wiring-v2-delta.md v1.9.0 → v1.9.1 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.2.0

**F-02 closure-audit: Description per-client buffer capacity 256→64** (2026-06-03):
- Description (line 33): "capacity 256" → "capacity 64" — aligns with Invariant 5, ADR-0010,
  SS-daemon-wiring-v2-delta.md §5d, and BC-2.05.009 (all of which already stated 64 since v1.1.0).
- Related BCs (BC-2.05.009 entry): "capacity 256" → "capacity 64" — same correction.
- No functional change; stale prose brought into agreement with canonical 64-slot buffer.

## §Trace v1.1.0

**Adversarial Pass 3 fixes — I3-004 (AttachSession re-attach trigger) + I3-003 (snapshot-then-resume buffering)** (2026-06-03):
- I3-004: PtyReset PC-3c corrected — the TUI sends `ClientToServer::AttachSession { session_id }`
  to the daemon (NOT `DaemonToHost::Attach`, which is a daemon→session-host message the TUI
  cannot send). The daemon routes `AttachSession` to `SessionManager::attach_session()` which
  sends `DaemonToHost::Attach` to the session-host. This replaces the previous incorrect
  "TUI sends a fresh DaemonToHost::Attach via the daemon's attach path" description. Per
  SS-ipc.md v1.13.0 §ClientToServer::AttachSession.
- I3-003 snapshot-then-resume: PC-3d and 3e added — TUI buffers live `PtyOutput` in
  `pending_pty_bytes` during dump transfer (session-host resumes PtyBytes immediately after
  snapshot, does NOT pause); replays buffered bytes after `ScrollbackDumpComplete` +
  parser reconstruction. Invariant 6 added. Per SS-session-manager.md v1.5.0 §Screen-state
  transfer on Attach step 5d-5e. Per-client buffer capacity updated to 64 (from 256) to
  match SS-ipc.md v1.13.0 §TUI IPC Read Loop Pattern canonical pattern.
- Architecture Source updated to SS-daemon-wiring-v2-delta.md v1.3.1, SS-session-manager.md
  v1.4.0, SS-ipc.md v1.13.0, ADR-0010 v1.4.0.

## §Trace v1.0.0

**Initial production — architect-delegated C2-002 BC (adversarial pass 2)** (2026-06-03T23:59:00Z):
- BC-2.05.011 authored as a NEW BC (not an extension of BC-2.05.010) to avoid mixing
  ClientToServer variants (BC-2.05.010 scope) with ServerToClient variants (this BC's scope).
- Covers the three new ServerToClient variants from SS-daemon-wiring-v2-delta.md v1.2.0 §5b/§5c:
  `ScrollbackChunk`, `ScrollbackDumpComplete`, `PtyReset`. These complete the chunked scrollback
  protocol that replaced the retired single-message `HostToDaemon::ScrollbackDump` (C2-002).
- TUI receiver protocol (chunk accumulation, parser reset, reconstruction, mismatch re-attach)
  specified from SS-session-manager.md v1.5.0 §Screen-state transfer on Attach.
- `PtyReset` 5-second status bar indicator specified from SS-session-manager.md v1.5.0 §PTY
  reader thread §TUI-surfaced PTY drop indicator.
- SE-16d PASS: 2026-06-03T23:59:00Z (new artifact).

## §Trace v1.2.5

**F-PASS6-SUG-002 fix — Story Anchor co-ownership: S-047-only → S-046 + S-047** (2026-06-16):
- Story Anchor expanded to list both co-owning stories: S-046 (PtyReset variant definition +
  broker emission) and S-047 (ScrollbackChunk*/ScrollbackDumpComplete + TUI receiver).
- Bidirectionally symmetric with STORY-INDEX BC Coverage (which already listed both stories)
  and with S-046 frontmatter (behavioral_contracts: [BC-2.05.009, BC-2.05.011]).
- No behavioral content changed. Anchor metadata only.

## §Trace v1.2.4

**Phase-2 Pass-1 fix burst — SS-session-manager v2.6.1 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.2.4 timestamp >= v1.2.3. PASS.
