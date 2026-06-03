---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md, architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md]
input-hash: "36255cf"
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

## Invariants

1. The 100ms budget is for the TOTAL pipeline: IPC decode + parser.process() + draw(). Each
   stage must be fast: IPC decode ≈ 1ms; parser.process() ≈ 1-5ms; draw() ≈ 10-50ms.
   The budget is dominated by the ratatui render cycle.
2. The `vt100::Parser` is updated on EVERY `PtyOutput` for the session, including sessions
   that are not currently focused. This enables O(1) session switching without re-fetching
   scrollback.
3. The TUI's mpsc channel for IPC events is bounded (capacity 64 per SS-embedded-pty.md
   §PTY Widget Pipeline). If the channel is full, the IPC reader blocks briefly. A slow
   render loop that cannot keep up will cause backpressure on the IPC channel. This is
   acceptable because it naturally throttles PTY output (the session-host's PTY reader also
   has a bounded channel).
4. `SCROLLBACK_ROWS` default is 1000 rows. `vt100::Parser::new(rows, cols, scrollback_rows)`
   is initialized with this default unless overridden by config.

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
| Architecture Source | SS-embedded-pty.md v1.0.2 §PTY Widget Pipeline; §Parser ownership in TUI; ADR-0011 §PTY stack selection |
| Test Name | test_BC_2_09_001_pty_output_renders_within_100ms |

## Related BCs

- [BC-2.08.007] — depends on: attach produces ScrollbackDump which flows through this pipeline
- [BC-2.09.002] — composes with: keyboard input flows opposite direction (TUI → PTY)
- [BC-2.09.006] — composes with: resize changes parser dimensions before PTY output is processed

## Architecture Anchors

- `architecture/SS-embedded-pty.md#pty-widget-pipeline` — full pipeline diagram and parser ownership
- `architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md` — technology selection rationale

## Story Anchor

S-TBD — Implement TUI PTY widget (vt100 parser, PseudoTerminal render, PtyOutput IPC handler; filled by story-writer)

## VP Anchors

VP-TBD — PTY output render latency tests (filled after VP creation)

## §Trace v1.0.0

**Initial production — v1A PRD delta** (2026-06-03T23:30:00Z):
- BC-2.09.001 authored for SS-09 (new subsystem) as part of the v1A control-center pivot BC burst.
- 100ms latency bound from SS-embedded-pty.md architect proposal preserved verbatim.
- SE-16d PASS: 2026-06-03T23:30:00Z (new artifact).
