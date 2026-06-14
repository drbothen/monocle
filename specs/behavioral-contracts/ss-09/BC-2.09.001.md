---
document_type: behavioral-contract
level: L3
version: "1.3.2"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md, architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md]
input-hash: "e2da3f4"
traces_to: prd.md
origin: greenfield
subsystem: SS-09
capability: CAP-009
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

# Behavioral Contract BC-2.09.001: PTY Output Renders Within 100ms of Byte Receipt at TUI

## Description

When the TUI's IPC reader receives a `ServerToClient::PtyOutput { session_id, bytes }` message,
it passes the bytes to the corresponding `vt100::Parser` instance and schedules a render tick.
The bytes MUST be visible in the TUI's embedded terminal widget within 100ms of receipt at the
TUI's IPC socket. This timing budget covers: IPC framing decode → `vt100::Parser.process()` →
`terminal.draw()` → `PseudoTerminal::render()`.

## Preconditions

1. `AppMode::EmbeddedTerminal { session_id }` is active.
2. The TUI is connected to the daemon's UDS and receiving IPC messages.
3. `App::pty_parsers[session_id]` is initialized (a `vt100::Parser` exists for this session).
4. The `PtyOutput` bytes form valid terminal escape sequences or printable characters.

## Postconditions

1. `App::on_pty_output(session_id, bytes)` is called by the IPC reader task within one
   mpsc channel cycle of the `PtyOutput` message being received.
2. `App::pty_parsers[session_id].process(&bytes)` is called. The parser updates its internal
   screen model.
3. A render tick is triggered immediately. `terminal.draw()` is called, resulting in
   `render_embedded_terminal(frame, area, &pty_parsers[session_id])` being invoked.
4. The rendered output is visible on the user's terminal within 100ms of the `PtyOutput`
   IPC message being received at the TUI's UDS connection.
5. All sessions receive `PtyOutput` processing even if not currently focused. Only the
   focused session is rendered, but non-focused sessions' parsers are updated (enabling
   instant-rendering on focus switch with no re-fetch needed).
6. **Auto-attach on first entry — persistence-reconnect (I11-001 PRONG A):** When
   `enter_embedded_terminal(session_id)` is invoked for a session that has not yet received a
   `ScrollbackDumpComplete` in the current TUI process lifetime (i.e., `session_id` is absent
   from `App::pty_dump_received`), the TUI MUST send
   `ClientToServer::AttachSession { session_id }` to the daemon before the first render tick of
   `AppMode::EmbeddedTerminal`. The TUI MUST mark the session as dump-in-progress so that any
   `PtyOutput` messages received while awaiting `ScrollbackDumpComplete` are buffered in
   `pending_pty_bytes[session_id]`. The resulting `ScrollbackChunk*` + `ScrollbackDumpComplete`
   dump and screen reconstruction MUST complete before live `PtyOutput` is applied to the
   parser (the buffering-and-replay obligations of BC-2.09.001 Invariant 5 and
   BC-2.05.011 §ScrollbackDumpComplete PC-3 and Invariant 6 already apply).

## Invariants

1. The 100ms budget is for the TOTAL pipeline: IPC decode + parser.process() + draw(). Each
   stage must be fast: IPC decode ≈ 1ms; parser.process() ≈ 1-5ms; draw() ≈ 10-50ms.
   The budget is dominated by the ratatui render cycle.
2. The `vt100::Parser` is updated on EVERY `PtyOutput` for the session, including sessions
   that are not currently focused. This enables O(1) session switching without re-fetching
   scrollback.
3. The TUI's mpsc channel for IPC events is bounded (capacity 64 per SS-embedded-pty.md
   §PTY Widget Pipeline). The IPC reader uses `.send().await` (backpressure), NOT `.try_send()`
   (drop). A slow render loop applies backpressure up to the daemon broker and ultimately
   to the session-host's PTY reader. This is the correct behavior — no PTY bytes are dropped.
4. `SCROLLBACK_ROWS` default is 1000 rows (maximum 10000, configurable).
   `vt100::Parser::new(rows, cols, scrollback_rows)` is initialized with this default unless
   overridden by config. Memory per parser: ~16 bytes/cell × cols × (visible_rows + scrollback_rows).
   Default: 16 × 80 × 1024 ≈ 1.3 MB/session; 8 sessions ≈ 10.4 MB. Cap at 10000 rows
   yields ~12.8 MB/session at 80 cols. See SS-embedded-pty.md §O4 for full bound analysis.
5. **Chunked scrollback receipt — parser reset protocol (C5):** When the TUI receives
   `ScrollbackChunk*` + `ServerToClient::ScrollbackDumpComplete` for a session (per
   BC-2.05.011 §ScrollbackDumpComplete PC-3), the TUI MUST:
   a. Reset the parser on `ScrollbackDumpComplete` receipt:
      `pty_parsers[session_id] = vt100::Parser::new(pty_rows, pty_cols, SCROLLBACK_ROWS)`.
   b. Reconstruct the screen from the accumulated `Vec<Vec<SerializedCell>>` styled-cell data
      WITHOUT re-parsing raw PTY bytes (see SS-session-manager.md §Screen-state transfer).
   c. Any `PtyOutput` messages received while waiting for `ScrollbackDumpComplete` are
      buffered in `pending_pty_bytes[session_id]` (per BC-2.05.011 Invariant 6). After
      reconstruction, replay buffered bytes through the reset parser in receipt order, then
      apply all subsequent `PtyOutput` events to the clean parser.
   The retired single-message `ServerToClient::ScrollbackDump` variant MUST NOT be used.
   The old behavior (forwarding raw bytes into an existing live parser) would double-apply
   content already in the parser's screen model — causing visual artifacts.
6. Scroll offsets are per-session (I7). `pty_scroll_offsets[session_id]` is consulted at
   render time, not a shared `pty_scroll_offset` field.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-200 | `PtyOutput` received for a session not in `pty_parsers` (e.g., race during session creation) | `pty_parsers.get_mut(&session_id)` returns `None`; bytes are silently dropped for this tick; next render shows nothing; parser will be initialized when `SessionListUpdate` adds the session — no panic |
| EC-201 | `PtyOutput` bytes contain a partial UTF-8 sequence at end | `vt100::Parser.process()` handles partial sequences correctly (vt100 crate internally buffers until sequence is complete); no corruption |
| EC-202 | High-frequency PTY output (>100 messages/second) | IPC reader and parser keep up; render tick is bounded by terminal refresh rate (typically 60Hz = 16.7ms/frame); frames may be merged (multiple PtyOutputs processed before one draw()); 100ms latency for first-byte-to-pixel is still met |
| EC-203 | `PtyOutput` received while `AppMode::Dashboard` is active (session not focused) | Parser updated for the session; no render of the PTY widget (only Dashboard panels rendered); O(1) switch to EmbeddedTerminal shows current parser state immediately |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `PtyOutput { session_id: "s1", bytes: "Hello\r\n" }` while in EmbeddedTerminal mode | "Hello" appears on parser screen; render tick fires; visible within 100ms | happy-path |
| `PtyOutput` for non-focused session | Parser updated; no PTY widget render | happy-path |
| `PtyOutput` with ANSI color sequence (`\x1b[32mGreen\x1b[0m`) | vt100 parser handles escape; cell attributes carry green color; tui-term renders green text | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `vt100::Parser.process()` called on PtyOutput receipt | unit |
| VP-TBD | PtyOutput for non-focused session updates parser but doesn't trigger PTY render | unit |
| VP-TBD | 100ms latency budget met in integration test with mock session-host PTY fixture | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability §SS-09 |
| Capability Anchor Justification | CAP-009 ("Embedded PTY widget; full-fidelity keyboard forwarding (printable + control + arrows + mouse + Kitty); PTY byte pipeline (IPC → vt100 → tui-term); session creation wizard") per ARCH-INDEX §Capability traceability — this BC defines the PTY byte pipeline performance contract: IPC → vt100 → tui-term within 100ms, which is the core of CAP-009's embedded PTY widget capability |
| Architecture Module | monocle-tui (App::on_pty_output, pty_parsers, PseudoTerminal widget) per ARCH-INDEX Subsystem Registry SS-09 |
| Architecture Source | SS-embedded-pty.md v1.5.2 §PTY Widget Pipeline; §Parser ownership in TUI; §O4 memory bound; §I7 per-session scroll offset; §EmbeddedTerminal ENTRY (auto-attach mandate, I11-001 PRONG A); SS-session-manager.md v2.2.1 §Screen-state transfer (C5); ADR-0011 v1.2.1 §Decision |
| Test Name | test_BC_2_09_001_pty_output_renders_within_100ms |

## Related BCs

- [BC-2.08.007] — depends on: attach triggers the `ScrollbackChunk*` + `ScrollbackDumpComplete` chunked sequence that flows through this pipeline for parser reset + screen reconstruction
- [BC-2.09.002] — composes with: keyboard input flows opposite direction (TUI → PTY)
- [BC-2.09.006] — composes with: resize changes parser dimensions before PTY output is processed

## Architecture Anchors

- `architecture/SS-embedded-pty.md#pty-widget-pipeline` — full pipeline diagram and parser ownership
- `architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md` — technology selection rationale

## Story Anchor

S-TBD — Implement TUI PTY widget (vt100 parser, PseudoTerminal render, PtyOutput IPC handler; filled by story-writer)

## VP Anchors

VP-TBD — PTY output render latency tests (filled after VP creation)

## §Trace v1.3.2

**Arch-source pin v1.5.1→v1.5.2** (2026-06-13 / D-277):
- Arch-source pin: SS-embedded-pty.md v1.5.1 → v1.5.2 (Architecture Source row).
- No behavioral content changed. Patch bump only.

## §Trace v1.3.1

**I17-001 + S17-002 — Pin-symmetry fix: ADR-0011 in Architecture Source; §-anchor corrected to exact heading** (2026-06-04):
- Architecture Source: `ADR-0011 §PTY stack selection` → `ADR-0011 v1.2.0 §Decision`.
  Fixes I17-001 (unpinned ADR in multi-doc Architecture Source cell — pin-symmetry violation)
  and S17-002 (loose §-anchor label paraphrasing actual heading; ADR-0011's decision section
  heading is `## Decision` per ADR-0011 file line 63).
- No behavioral content changed; version bumped as patch 1.3.0→1.3.1.

## §Trace v1.3.0

**I11-001 PRONG A — PC-6: auto-attach mandate for persistence-reconnect** (2026-06-04):
- PC-6 added: normative postcondition mandating that `enter_embedded_terminal(session_id)` MUST
  send `ClientToServer::AttachSession { session_id }` before the first render tick of
  `AppMode::EmbeddedTerminal` when the session has not yet received a `ScrollbackDumpComplete`
  in the current TUI process lifetime (i.e., `session_id` absent from `App::pty_dump_received`).
  Closes the v1A spec gap identified by architect in Phase-1d Pass-11 finding I11-001 PRONG A.
- Architecture Source updated: `SS-embedded-pty.md v1.3.0` → `v1.4.0`, adding
  §EmbeddedTerminal ENTRY (auto-attach mandate) as an explicit anchor citation.
- BC-2.05.011 §ScrollbackDumpComplete PC-3 + Invariant 6 referenced in PC-6 as the
  buffering-and-replay obligations that govern the dump window (verified: PC-3 is item 3 in
  the §ScrollbackDumpComplete postconditions, covering parser reset + screen reconstruction;
  Invariant 6 specifies `pending_pty_bytes` buffering during dump-in-progress).

## §Trace v1.2.0

**CRIT-001 adversarial pass-4 fix — Invariant 5 + Related-BCs: retired ScrollbackDump → chunked protocol** (2026-06-03):
- Invariant 5: retired `ServerToClient::ScrollbackDump` single-message name replaced throughout.
  The TUI now correctly references `ScrollbackChunk*` + `ServerToClient::ScrollbackDumpComplete`
  (per BC-2.05.011 §ScrollbackDumpComplete PC-3). Parser reset happens on `ScrollbackDumpComplete`
  receipt; live `PtyOutput` arriving during the dump window is buffered in
  `pending_pty_bytes[session_id]` (per BC-2.05.011 Invariant 6) and replayed after
  reconstruction. The protocol invariant that the retired single-message form MUST NOT be
  used is now stated explicitly.
- Related-BCs [BC-2.08.007] entry: "attach produces ScrollbackDump" → "attach triggers
  `ScrollbackChunk*` + `ScrollbackDumpComplete` chunked sequence".

## §Trace v1.1.0

**C5/O4/I7 — ScrollbackDump parser reset, memory bound, per-session scroll offset** (2026-06-03):
- Invariant 3: backpressure model clarified (`.send().await`; no drop on IPC channel).
- Invariant 4: memory bound revised to include per-cell styled-attribute size (~16 bytes/cell).
  10000-row cap yields ~12.8 MB/session. Default yields ~1.3 MB/session.
- Invariant 5 (new): chunked scrollback receipt (`ScrollbackChunk*` + `ScrollbackDumpComplete`) → parser reset protocol. Receiving styled-cell
  data requires resetting the parser before applying; prevents double-counting live state.
  (Note: original trace text said "ScrollbackDump" — corrected to chunked protocol in v1.2.0.)
- Invariant 6 (new): per-session scroll offset (I7 fix from SS-embedded-pty.md).
- Architecture Source updated to SS-session-manager.md v1.5.0 and SS-embedded-pty.md v1.3.0.

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.001 authored for SS-09 (new subsystem) as part of the v1A control-center pivot BC burst.
- 100ms latency bound from SS-embedded-pty.md architect proposal preserved verbatim.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
