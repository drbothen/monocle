---
document_type: behavioral-contract
level: L3
version: "1.7.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T23:30:00Z
phase: v1A-prd-delta
inputs: [prd.md, architecture/ARCH-INDEX.md, architecture/SS-embedded-pty.md, architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md]
input-hash: "4caaa43"
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
   **F-S039-004 extension:** This `.send().await` requirement extends to the `AttachSession`
   control message sent in `enter_embedded_terminal()`. `try_send()` on that path is
   forbidden; see SS-embedded-pty.md §Auto-attach §F-S039-004 RULING for the full rollback
   contract (if the channel is closed, do NOT set `dump_in_progress`, do NOT enter
   `EmbeddedTerminal` mode, surface error to status bar).
4. `SCROLLBACK_ROWS` default is 1000 rows (maximum 10000, configurable).
   `vt100::Parser::new(rows, cols, scrollback_rows)` is initialized with the scrollback
   value sourced from `~/.monocle/config.json:pty_scrollback_rows` according to these rules
   (normative; see BC-2.09.007 Invariant 1 + EC-242/EC-243 for the authoritative definition):
   - **Absent (key missing / config falls back to default → `None`):** `scrollback_rows = 1000`.
   - **Present (key exists with a parseable `u32`):** value is clamped to [1, 10000].
     `0 → 1` (clamped to minimum per BC-2.09.007 EC-243); values above 10000 → 10000
     (clamped to maximum per BC-2.09.007 EC-242). Present out-of-range values are NOT
     defaulted to 1000 — they are clamped. Only absence yields the 1000 default.
   Memory per parser: ~16 bytes/cell × cols × (visible_rows + scrollback_rows).
   Default: 16 × 80 × 1024 ≈ 1.3 MB/session; 8 sessions ≈ 10.4 MB. Cap at 10000 rows
   yields ~12.8 MB/session at 80 cols. See SS-embedded-pty.md §O4 for full bound analysis.
   **Default parser dimensions (F-S039-P2-004):** When parsers are created on `SessionListUpdate`
   / `InitialState` arrival (before any attach), the canonical placeholder dimensions are
   `PTY_DEFAULT_ROWS = 24` and `PTY_DEFAULT_COLS = 80` (defined in `monocle-core` per
   SS-embedded-pty.md §Parser initialization). These are ALWAYS replaced by the real PTY
   dimensions from `ScrollbackDumpComplete.pty_rows`/`pty_cols` before first render (the parser
   is reset in the `ScrollbackDumpComplete` handler). No BC content changes — this is an
   implementation constraint on what to pass to `vt100::Parser::new()` for non-attached sessions.
5. **Chunked scrollback receipt — parser reset protocol (C5):** When the TUI receives
   `ServerToClient::ScrollbackDumpComplete` for a session (per BC-2.05.011 §ScrollbackDumpComplete
   PC-3), the TUI MUST:
   **F-S039-P2-002 — idempotency guard (pre-condition for all steps below):**
   The handler MUST first check `dump_in_progress.get(&session_id) == Some(&true)`. If this
   check fails (i.e., `dump_in_progress` is `false`, absent, or the session_id is unknown),
   the handler MUST no-op with a `tracing::trace!` log and return immediately. This guard
   prevents spurious/duplicate `ScrollbackDumpComplete` messages (e.g., daemon re-broadcast,
   post-detach delivery) from destroying a live populated parser. Only when the guard passes
   (a dump window IS active) do the steps below execute.
   a. Reset the parser on `ScrollbackDumpComplete` receipt:
      `pty_parsers[session_id] = vt100::Parser::new(pty_rows, pty_cols, SCROLLBACK_ROWS)`.
      Use `pty_rows` and `pty_cols` from the `ScrollbackDumpComplete` message fields.
   b. **[S-047 scope — styled-cell reconstruction]** Reconstruct the screen from the
      accumulated `Vec<Vec<SerializedCell>>` styled-cell data WITHOUT re-parsing raw PTY bytes
      (see SS-session-manager.md §Screen-state transfer). This step is implemented by S-047
      (which delivers `ScrollbackChunk` accumulation, `total_chunks` validation, `chunk_seq`
      contiguity, and cursor restoration from `cursor_row`/`cursor_col`). S-039 does NOT
      implement this step — the daemon emits empty dumps (`total_chunks: 0`) until S-047
      delivers the daemon-side chunk broadcast (F-S035-AC005-DAEMON-BROADCAST). See
      SS-embedded-pty.md §F-S039-005/006 RULING for the full boundary.
   c. Any `PtyOutput` messages received while waiting for `ScrollbackDumpComplete` are
      buffered in `pending_pty_bytes[session_id]` (per BC-2.05.011 Invariant 6). After
      the parser reset (step a), replay buffered bytes through the reset parser in receipt
      order, then apply all subsequent `PtyOutput` events to the clean parser.
   d. **Remove** `session_id` from `dump_in_progress` (i.e., call `dump_in_progress.remove(&session_id)`,
      NOT `dump_in_progress.insert(session_id, false)`) and insert `session_id` into `pty_dump_received`.
      **Normative note:** the idempotency guard added for F-S039-P2-002 checks
      `dump_in_progress.get(&session_id) == Some(&true)`. An absent entry and a `Some(false)` entry
      are treated identically as "no active dump window" — both cause the guard to no-op. Removing the
      entry (rather than setting it false) avoids accumulating stale map entries across session reconnects,
      which would otherwise grow unboundedly over the TUI process lifetime.
   The retired single-message `ServerToClient::ScrollbackDump` variant MUST NOT be used.
   The old behavior (forwarding raw bytes into an existing live parser) would double-apply
   content already in the parser's screen model — causing visual artifacts.
6. Scroll offsets are per-session (I7). `pty_scroll_offsets[session_id]` is consulted at
   render time, not a shared `pty_scroll_offset` field.
7. **Dump-window buffer cap (F-PASS4-MED-001):** The `pending_pty_bytes[session_id]` buffer is
   bounded. While `dump_in_progress[session_id] == true`, the TUI MUST enforce both of:
   - **Byte cap:** `MAX_PENDING_PTY_BYTES = 512 * 1024` (512 KiB) total across all buffered
     `Vec<u8>` entries for that session.
   - **Message cap:** `MAX_PENDING_PTY_MESSAGES = 4096` entries in the per-session `Vec<Vec<u8>>`.
   When either cap is exceeded on a new `PtyOutput` arrival, the implementation MUST drop the
   OLDEST entry (drop-oldest eviction; NOT drop-newest, NOT clear-all) and increment a per-session
   `pending_pty_drop_count: HashMap<String, u64>` counter (separate from the IPC channel drop
   counter). The drop counter MUST be surfaced in the status bar whenever `dump_in_progress` is
   active for the focused session (e.g., `[dump: 3 drops]`). This bound prevents unbounded heap
   growth when `ScrollbackDumpComplete` is delayed indefinitely (daemon hang, lost message,
   S-047 bug). Constants `MAX_PENDING_PTY_BYTES` and `MAX_PENDING_PTY_MESSAGES` are defined in
   `monocle-core` (pure constants, no I/O). The dump-window timeout in Invariant 8 provides a
   complementary time-based exit; this cap provides a memory-based exit.
   **Single-oversized-entry clarification (EC-208):** When a single `PtyOutput` entry arrives
   whose byte length alone exceeds `MAX_PENDING_PTY_BYTES` and the buffer is otherwise empty,
   drop-oldest eviction removes that sole entry (oldest == only entry), the buffer remains
   empty, and the drop counter is incremented — this is the defined, intended behavior, not a
   violation of the drop-oldest rule.
8. **Dump-window timeout (F-PASS4-MED-001):** If `dump_in_progress[session_id] == true` and
   `DUMP_WINDOW_TIMEOUT = 10s` elapses without a `ScrollbackDumpComplete` arriving for that
   session, the TUI MUST force-resolve the dump window:
   - Reset `dump_in_progress[session_id]` (call `dump_in_progress.remove(&session_id)`).
   - Clear `pending_pty_bytes[session_id]` (call `pending_pty_bytes[session_id].clear()` or
     remove the entry).
   - Do NOT insert `session_id` into `pty_dump_received` (the dump never completed; a future
     `enter_embedded_terminal` re-run will restart the full attach protocol).
   - Reset the parser to placeholder dims: `pty_parsers[session_id] = vt100::Parser::new(PTY_DEFAULT_ROWS, PTY_DEFAULT_COLS, SCROLLBACK_ROWS)` so the terminal is not blank.
   - Surface a status bar warning: `"[warn] scrollback dump timed out for <session_id>"`.
   - Log at `tracing::warn!` level with `session_id` and elapsed duration.
   This ensures the terminal recovers and new live `PtyOutput` is applied to a clean parser
   rather than held in the buffer forever. `DUMP_WINDOW_TIMEOUT = 10` seconds is defined in
   `monocle-core`. A tokio `timeout` or `sleep` task is used; the timeout is cancelled
   immediately on `ScrollbackDumpComplete` receipt (normal path).
9. **Reconnect dump-state reset (F-PASS4-MED-002):** When the TUI IPC transport fires
   `TransportEvent::Disconnected` (UDS connection to daemon lost), the TUI MUST clear the
   following fields for ALL sessions without exception:
   - `dump_in_progress`: call `dump_in_progress.clear()` — removes all in-flight dump flags.
   - `pending_pty_bytes`: call `pending_pty_bytes.clear()` (or drain per-session) — discards
     all buffered PtyOutput accumulated on the dead connection; these bytes will never be
     followed by a `ScrollbackDumpComplete` from the dead connection.
   - `pty_dump_received`: call `pty_dump_received.clear()` — marks all sessions as needing a
     fresh dump on next `enter_embedded_terminal`. The old connection's `AttachSession` request
     and `ScrollbackDumpComplete` can never arrive on the new connection; the fresh connection
     is a new attach period.
   The `pty_parsers` HashMap MUST NOT be cleared (parsers contain the best-available screen
   content; no-clobber is correct — the user sees stale but non-blank content until the next
   attach protocol completes). Parser content is refreshed when the user next enters
   EmbeddedTerminal mode (triggering `AttachSession` on the new connection, per the auto-attach
   mandate in PC-6).
   This clearing MUST happen in the `on_transport_event(Disconnected)` handler, NOT deferred
   to `on_initial_state`. Rationale: between `Disconnected` and `InitialState`, `PtyOutput`
   events cannot arrive (the IPC reader task is not running), so clearing on `Disconnected`
   is safe and eliminates the window where stale `pending_pty_bytes` would accumulate.
   If `AppMode::EmbeddedTerminal { session_id }` is active at the moment of disconnect, the TUI
   MUST also exit `EmbeddedTerminal` mode (transition to `Dashboard` or `prior` AppMode)
   to ensure the render loop does not attempt to render a stale parser as if the session
   were live. A reconnecting indicator MUST be shown in the status bar; the shared
   `DAEMON_DISCONNECT_STATUS` constant (currently `"[disconnected] reconnecting..."`) satisfies
   this requirement. The requirement is functional — a reconnect indicator MUST be visible —
   not a literal string match.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-200 | `PtyOutput` received for a session not in `pty_parsers` (e.g., race during session creation) | `pty_parsers.get_mut(&session_id)` returns `None`; bytes are silently dropped for this tick; next render shows nothing; parser will be initialized when `SessionListUpdate` adds the session — no panic |
| EC-201 | `PtyOutput` bytes contain a partial UTF-8 sequence at end | `vt100::Parser.process()` handles partial sequences correctly (vt100 crate internally buffers until sequence is complete); no corruption |
| EC-202 | High-frequency PTY output (>100 messages/second) | IPC reader and parser keep up; render tick is bounded by terminal refresh rate (typically 60Hz = 16.7ms/frame); frames may be merged (multiple PtyOutputs processed before one draw()); 100ms latency for first-byte-to-pixel is still met |
| EC-203 | `PtyOutput` received while `AppMode::Dashboard` is active (session not focused) | Parser updated for the session; no render of the PTY widget (only Dashboard panels rendered); O(1) switch to EmbeddedTerminal shows current parser state immediately |
| EC-204 | `PtyOutput` arrives during dump window and `pending_pty_bytes` byte cap (`MAX_PENDING_PTY_BYTES = 512 KiB`) is exceeded | Drop oldest entry; increment `pending_pty_drop_count` for this session; if session is focused and in EmbeddedTerminal, status bar shows `[dump: N drops]`; no panic |
| EC-205 | `ScrollbackDumpComplete` never arrives within `DUMP_WINDOW_TIMEOUT = 10s` | Force-resolve dump window: remove `dump_in_progress` entry, clear `pending_pty_bytes`, reset parser to `PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS`, surface warning, do NOT insert into `pty_dump_received`; subsequent `enter_embedded_terminal` re-triggers full attach protocol |
| EC-206 | Transport disconnects while `dump_in_progress[session_id] == true` and session is mid-dump | `on_transport_event(Disconnected)` clears `dump_in_progress`, `pending_pty_bytes`, and `pty_dump_received` for ALL sessions; TUI exits EmbeddedTerminal mode if active; reconnecting indicator shown via `DAEMON_DISCONNECT_STATUS` constant; next `enter_embedded_terminal` after reconnect re-runs full attach protocol |
| EC-207 | Transport reconnects; `InitialState` arrives listing sessions that were mid-dump on the old connection | `dump_in_progress` and `pending_pty_bytes` already cleared at `Disconnected`; `pty_dump_received` already cleared; parsers are preserved with stale content; next `enter_embedded_terminal` call triggers fresh `AttachSession` on the new connection |
| EC-208 | A single `PtyOutput` message arrives whose byte length alone exceeds `MAX_PENDING_PTY_BYTES` and the buffer is otherwise empty | Drop-oldest eviction removes that sole entry (oldest == only entry); buffer remains empty; `pending_pty_drop_count` incremented; this is defined, intended behavior — the oversized message cannot fit and is safely discarded; not a violation of the drop-oldest rule |

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
| Architecture Source | SS-embedded-pty.md §PTY Widget Pipeline; §Parser ownership in TUI; §Parser initialization (PTY_DEFAULT_ROWS/COLS, F-S039-P2-004); §O4 memory bound; §I7 per-session scroll offset; §EmbeddedTerminal ENTRY (auto-attach mandate, I11-001 PRONG A; F-S039-004 rollback ruling; F-PASS4-MED-001 buffer cap + timeout; F-PASS4-MED-002 reconnect dump-state reset; F-S039-005/006 scope boundary); §F-S039-005/006 RULING §S-039 OWNS (F-S039-P2-002 idempotency guard); §state-machine-invariants (F-S039-P2-003 terminated-exit-before-GC); SS-session-manager.md v2.17.1 §Screen-state transfer (C5); ADR-0011 v1.2.1 §Decision |
| Test Name | test_BC_2_09_001_pty_output_renders_within_100ms |

## Related BCs

- [BC-2.08.007] — depends on: attach triggers the `ScrollbackChunk*` + `ScrollbackDumpComplete` chunked sequence that flows through this pipeline for parser reset + screen reconstruction
- [BC-2.09.002] — composes with: keyboard input flows opposite direction (TUI → PTY)
- [BC-2.09.006] — composes with: resize changes parser dimensions before PTY output is processed

## Architecture Anchors

- `architecture/SS-embedded-pty.md#pty-widget-pipeline` — full pipeline diagram and parser ownership
- `architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md` — technology selection rationale

## Story Anchor

S-039 — Implement TUI PTY widget (vt100 parser, PseudoTerminal render, PtyOutput IPC handler)

## VP Anchors

VP-TBD — PTY output render latency tests (filled after VP creation)

## §Trace 1.7.6

**SS-session-manager arch-source pin cascade v2.17.0→v2.17.1 (F-S042-ADV-MED-001 ownership-drift cleanup)** (2026-06-21):
- Architecture Source pin updated: SS-session-manager.md v2.17.0 → v2.17.1. No behavioral content changed.
- SE-16d monotonicity: 1.7.6 > 1.7.5. PASS.

## §Trace 1.7.5

**SS-session-manager arch-source pin cascade v2.16.0→v2.17.0** (2026-06-21):
- Architecture Source pin updated: SS-session-manager.md v2.16.0 → v2.17.0. No behavioral content changed.
- SE-16d monotonicity: 1.7.5 > 1.7.4. PASS.

## §Trace v1.7.4

**SS-session-manager v2.15.1 → v2.16.0 Architecture Source pin cascade (Ruling A errata)** (2026-06-21):
- Architecture Source pin updated: SS-session-manager.md v2.15.1 → v2.16.0. No behavioral content changed.
- SE-16d monotonicity: v1.7.4 > v1.7.3. PASS.

## §Trace v1.7.3

**Arch-source pin: SS-embedded-pty.md v1.10.0 → v1.11.0** (2026-06-20):
- S-040 delivery flag-set correction bumped SS-embedded-pty to v1.11.0. Architecture Source
  row updated. No behavioral content changed.
- SE-16d monotonicity: v1.7.3 timestamp >= v1.7.2. PASS.

## §Trace v1.7.2

**F-S039-P7-LOW-002 — Invariant 7: single-oversized-entry edge case clarification; EC-208 added** (2026-06-20):

- **Invariant 7 extended (F-S039-P7-LOW-002):** Added normative single-oversized-entry
  clarification: when a single `PtyOutput` entry arrives whose byte length alone exceeds
  `MAX_PENDING_PTY_BYTES` and the buffer is otherwise empty, drop-oldest eviction removes
  that sole entry (oldest == only entry), the buffer remains empty, and
  `pending_pty_drop_count` is incremented. This is defined, intended behavior — the oversized
  message cannot fit and is safely discarded. It is not a violation of the drop-oldest rule
  (the rule holds: oldest entry is removed; it happens to also be the only entry).
- **EC-208 added:** Dedicated edge case row documenting the single-oversized-entry scenario
  and its expected behavior.
- No other behavioral content changed; the implementation already handles this correctly per
  the existing eviction loop — this clarification closes the spec gap so the implementation
  path is explicitly covered.
- Source finding: F-S039-P7-LOW-002 (S-039 adversarial Pass-7, LOW severity).
- SE-16d monotonicity: v1.7.2 timestamp 2026-06-20 >= v1.7.1 timestamp 2026-06-20. PASS.

## §Trace v1.7.1

**F-S039-P6-LOW-001 — Invariant 9 / EC-206 / implementer directive #7: status-bar message softened from exact literal to functional requirement** (2026-06-20):

- **Invariant 9 revised (status-bar wording):** The requirement `A status bar message MUST be
  shown: "[reconnecting...]"` has been replaced with a functional requirement: a reconnecting
  indicator MUST be visible in the status bar; the shared `DAEMON_DISCONNECT_STATUS` constant
  (currently `"[disconnected] reconnecting..."`) satisfies this. The prior exact-literal
  formulation conflicted with the pre-existing cross-story constant `DAEMON_DISCONNECT_STATUS`
  owned by S-023/S-025 (used on all disconnect-related screens). The richer string
  `"[disconnected] reconnecting..."` contains the reconnect signal and is acceptable UX.
  Changing the shared constant to match the old BC literal would have cross-story blast radius.
- **EC-206 updated:** `[reconnecting...]` literal replaced with `reconnecting indicator shown
  via DAEMON_DISCONNECT_STATUS constant` for consistency with the revised invariant.
- **Implementer directive #7 updated:** Hardcoding `"[reconnecting...]"` is now explicitly
  forbidden; directive reads "display the reconnecting indicator via the shared
  `DAEMON_DISCONNECT_STATUS` constant."
- Source finding: F-S039-P6-LOW-001 (S-039 adversarial Pass-6, LOW severity observation).
- SE-16d monotonicity: v1.7.1 timestamp 2026-06-20 >= v1.7.0 timestamp 2026-06-20. PASS.

## §Trace v1.7.0

**F-S039-P5-001 — Invariant 4: explicit absent→1000 / present-out-of-range→clamped semantics** (2026-06-20):

- **Invariant 4 clarified (F-S039-P5-001):** Replaced the vague "initialized with this default
  unless overridden by config" language with an explicit two-case normative rule:
  - ABSENT (`pty_scrollback_rows` key missing → `None`) → 1000 default.
  - PRESENT (key exists with a parseable `u32`) → clamped to [1, 10000]; `0 → 1` (EC-243),
    `>10000 → 10000` (EC-242). Present out-of-range values are NOT defaulted to 1000.
  Cross-references to BC-2.09.007 Invariant 1 + EC-242/EC-243 added as normative anchors.
- Source finding: F-S039-P5-001 (S-039 adversarial Pass-5) — the prior Invariant 4 wording
  was compatible with the incorrect reading "invalid/0 → 1000 default" used in S-039 AC-008
  and the `default_scrollback_rows` doc-comment. This fix makes Invariant 4 unambiguous and
  consistent with BC-2.09.007's edge cases (which were already correct).
- SE-16d monotonicity: v1.7.0 timestamp 2026-06-20 >= v1.6.0 timestamp 2026-06-20. PASS.

## §Trace v1.6.0

**F-PASS4-MED-001 + F-PASS4-MED-002 — dump-window buffer cap + timeout; reconnect dump-state reset** (2026-06-20):

- **Invariant 7 (new — F-PASS4-MED-001 — dump-window buffer cap):** `pending_pty_bytes[session_id]`
  is bounded by `MAX_PENDING_PTY_BYTES = 512 KiB` (total bytes) and `MAX_PENDING_PTY_MESSAGES = 4096`
  (entry count). On cap exceeded: drop-oldest eviction, increment `pending_pty_drop_count` counter,
  surface drop count in status bar while dump is active for focused session.
- **Invariant 8 (new — F-PASS4-MED-001 — dump-window timeout):** `DUMP_WINDOW_TIMEOUT = 10s`
  force-resolve: on expiry, remove `dump_in_progress` entry, clear buffer, reset parser to
  `PTY_DEFAULT_ROWS × PTY_DEFAULT_COLS`, surface `tracing::warn!` + status bar message, do NOT
  insert into `pty_dump_received` (next enter re-runs full attach).
- **Invariant 9 (new — F-PASS4-MED-002 — reconnect dump-state reset):** `on_transport_event(Disconnected)`
  MUST call `dump_in_progress.clear()`, `pending_pty_bytes.clear()`, and `pty_dump_received.clear()`
  for ALL sessions. `pty_parsers` MUST NOT be cleared (no-clobber; preserves best-available screen).
  If `AppMode::EmbeddedTerminal` is active at disconnect, exit to prior mode and show
  `"[reconnecting...]"`. Clearing at `Disconnected` (not `InitialState`) eliminates the window
  where stale bytes accumulate between disconnect and reconnect.
- **Edge cases EC-204 through EC-207 added** documenting cap eviction, timeout force-resolve,
  mid-dump disconnect, and reconnect with survivors.
- **Implementer directives:**
  1. Define `MAX_PENDING_PTY_BYTES: usize = 512 * 1024`, `MAX_PENDING_PTY_MESSAGES: usize = 4096`,
     and `DUMP_WINDOW_TIMEOUT: Duration = Duration::from_secs(10)` in `monocle-core`
     (pure constants module, e.g., `monocle-core/src/pty_constants.rs`).
  2. Add `pending_pty_drop_count: HashMap<String, u64>` field to `App` struct.
  3. In `on_pty_output`: when `dump_in_progress[session_id] == Some(&true)`, after appending
     to `pending_pty_bytes[session_id]`, check both caps; if exceeded, remove `pending_pty_bytes[session_id].first()` (drop oldest) and increment `pending_pty_drop_count[session_id]`.
  4. In `render_status_bar` (or equivalent): if `dump_in_progress` is active for focused session
     AND `pending_pty_drop_count[focused_session_id] > 0`, render `[dump: N drops]` segment.
  5. In `enter_embedded_terminal` (after successful `AttachSession` send): spawn a `tokio::time::sleep(DUMP_WINDOW_TIMEOUT)` task that, on firing, performs force-resolve if `dump_in_progress[session_id]` is still `Some(&true)`. Cancel the task on `ScrollbackDumpComplete` receipt by storing a `JoinHandle` or `AbortHandle` in a `dump_timeout_handles: HashMap<String, AbortHandle>` map. Abort and remove on `ScrollbackDumpComplete`.
  6. Add `dump_timeout_handles: HashMap<String, AbortHandle>` field to `App` struct (monocle-tui scope; not pure-core since it holds tokio handles).
  7. In `on_transport_event(Disconnected)`: call `dump_in_progress.clear()`, `pending_pty_bytes.clear()`, `pty_dump_received.clear()`. Abort and clear `dump_timeout_handles`. If `AppMode::EmbeddedTerminal`, call `exit_embedded_terminal()`. Display the reconnecting indicator via the shared `DAEMON_DISCONNECT_STATUS` constant (currently `"[disconnected] reconnecting..."`); do NOT hardcode a different string literal.
- SE-16d monotonicity: v1.6.0 timestamp 2026-06-20 >= v1.5.1 timestamp 2026-06-20. PASS.

## §Trace v1.5.1

**F-S039-REV-003 — Invariant 5 step d: remove dump_in_progress entry instead of setting false** (2026-06-20):

- **Invariant 5 step d revised:** Changed normative text from `dump_in_progress[session_id] = false`
  to `dump_in_progress.remove(&session_id)`. Added normative note clarifying that the F-S039-P2-002
  idempotency guard treats an absent entry and `Some(false)` identically as "no active dump window";
  removal avoids stale-entry accumulation across session reconnects.
- **Implementer note:** change `dump_in_progress.insert(id, false)` to `dump_in_progress.remove(&id)`
  in `on_scrollback_dump_complete`. No other behavioral change.
- SE-16d monotonicity: v1.5.1 timestamp 2026-06-20 >= v1.5.0 timestamp 2026-06-20. PASS.

## §Trace v1.5.0

**F-S039-P2-004 + F-S039-P2-002 + F-S039-P2-003 rulings — parser default dims; idempotency guard; terminated-session exit ordering** (2026-06-20):

- **Invariant 4 extended (F-S039-P2-004):** Added normative note documenting `PTY_DEFAULT_ROWS = 24`
  and `PTY_DEFAULT_COLS = 80` as the canonical placeholder dimensions for parsers created on session
  arrival (before any attach). These are always replaced by real dims from `ScrollbackDumpComplete`
  on first attach. No behavioral change — implementers now have a named constant to use instead of
  hardcoded literals.
- **Invariant 5 revised (F-S039-P2-002):** Added mandatory idempotency guard as a pre-condition
  for all parser-reset steps. Handler MUST check `dump_in_progress.get(&session_id) == Some(&true)`;
  if false/absent, no-op with `tracing::trace!` and return. Prevents spurious/duplicate/post-detach
  `ScrollbackDumpComplete` from destroying live parser state.
- **Architecture Source updated:** SS-embedded-pty.md v1.8.0 → v1.9.0; added §Parser initialization
  (F-S039-P2-004), §F-S039-005/006 RULING idempotency guard (F-S039-P2-002), and
  §state-machine-invariants terminated-exit-before-GC (F-S039-P2-003) as explicit anchor citations.
- SE-16d monotonicity: v1.5.0 timestamp 2026-06-20 >= v1.4.0 timestamp 2026-06-20. PASS.

## §Trace v1.4.0

**F-S039-004 + F-S039-005/006 rulings — async auto-attach contract; S-039 vs S-047 scope boundary** (2026-06-20):

- **Invariant 3 extended (F-S039-004):** Added normative note extending the `.send().await`
  mandate to the `AttachSession` control message sent in `enter_embedded_terminal()`.
  `try_send()` on that path is explicitly forbidden. Cross-reference to
  SS-embedded-pty.md §Auto-attach §F-S039-004 RULING for full rollback contract.
- **Invariant 5 revised (F-S039-005/006):** Styled-cell reconstruction (step b) annotated
  as [S-047 scope]. S-039 implements steps a (parser reset using `pty_rows`/`pty_cols` from
  the message), c (replay buffered bytes), and d (flag updates). S-047 implements styled-cell
  accumulation, `total_chunks` validation, chunk contiguity, and cursor restoration. Rationale:
  daemon emits empty dumps today (F-S035-AC005-DAEMON-BROADCAST); step b cannot be tested
  end-to-end until S-047 delivers daemon-side chunk broadcast. Step d (flags) made explicit
  in the numbered list (was implied only). Architecture Source updated:
  SS-embedded-pty.md v1.7.0 → v1.8.0.
- SE-16d monotonicity: v1.4.0 timestamp 2026-06-20 >= v1.3.8 timestamp 2026-06-19. PASS.

## §Trace v1.3.8 (retained — originally a duplicate header; now correctly sequenced)

## §Trace v1.3.7

**SS-session-manager v2.14.0 → v2.15.0 Architecture Source pin cascade (F-S038-PASS1-001)** (2026-06-19):
- Architecture Source pin: SS-session-manager.md v2.14.0 → v2.15.0 (single-writer mandate +
  HookEndpointConfig construction). No behavioral content changes to this BC.
- SE-16d monotonicity: v1.3.7 timestamp 2026-06-19 >= v1.3.6 timestamp 2026-06-19. PASS.

## §Trace v1.3.6

**SS-session-manager v2.13.0 → v2.14.0 Architecture Source pin cascade (F-S035-PASS5-MED-001)** (2026-06-19T00:00:00Z):
- Architecture Source pin: SS-session-manager.md v2.13.0 → v2.14.0 (v2.14.0 adds EC-188
  timeout → Terminated subpath in the attach_session Detached cell of the action×state matrix
  — F-S035-PASS5-MED-001). No behavioral content changes to this BC.
- SE-16d monotonicity: v1.3.6 timestamp 2026-06-19 >= v1.3.5 timestamp 2026-06-16. PASS.

## §Trace v1.3.5

**SS-session-manager v2.12.0 → v2.13.0 Architecture Source pin cascade** (2026-06-19T00:00:00Z):
- Architecture Source pin: SS-session-manager.md v2.12.0 → v2.13.0. No behavioral content changes.
- SE-16d monotonicity: v1.3.5 timestamp 2026-06-19 >= v1.3.4 timestamp 2026-06-16. PASS.

## §Trace v1.3.4

**SS-session-manager v2.11.0 → v2.12.0 Architecture Source pin cascade** (2026-06-19T00:00:00Z):
- Architecture Source pin: SS-session-manager.md v2.11.0 → v2.12.0. No behavioral content changes.
- SE-16d monotonicity: v1.3.4 timestamp 2026-06-19 >= v1.3.3 timestamp 2026-06-15. PASS.

## §Trace v1.3.3

**Burst-E D-305 — Story Anchor resolved: S-TBD → S-039** (2026-06-15):
- Story Anchor filled from Phase-2 Burst B story decomposition. No behavioral content changed.

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

## §Trace v1.3.4

**Phase-2 Pass-1 fix burst — SS-session-manager v2.7.3 / SS-daemon-wiring-v2-delta v1.11.4 Architecture Source pin cascade** (2026-06-16T00:00:00Z):
- Architecture Source pin(s) updated for SS-session-manager.md v2.6.0 → v2.6.1 and/or SS-daemon-wiring-v2-delta.md v1.11.3 → v1.11.4. Plain version-pin refresh — both SS spec bumps were SS-ipc Architecture Source cascade patches only; no normative API or invariant changes.
- SE-16d monotonicity: v1.3.4 timestamp >= v1.3.3. PASS.
