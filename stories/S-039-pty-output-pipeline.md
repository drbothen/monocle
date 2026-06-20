---
document_type: story
level: L4
story_id: S-039
epic_id: EPIC-09
version: "1.6"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-20T00:00:00Z
phase: 2
points: 8
wave: 9
tdd_mode: strict
priority: P0
depends_on: [S-021, S-025, S-035]
blocks: [S-040, S-042, S-043]
target_module: monocle-tui
subsystems: [SS-09]
behavioral_contracts: [BC-2.09.001]
verification_properties: []
estimated_days: 4
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.001.md, version: "1.7.0"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.9.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.09.001 (PTY output renders within 100ms; vt100::Parser pipeline; auto-attach on first entry; scrollback dump buffering and replay)"
# BC status: non-empty; status draft pending Phase-2 adversarial convergence gate (authoritative versions in inputs: frontmatter)
---

# S-039: PTY Output Pipeline — vt100::Parser, PseudoTerminal Render, PtyOutput IPC Handler, Auto-Attach on First Entry

## Narrative

As the monocle TUI, I want all `ServerToClient::PtyOutput { session_id, bytes }` IPC
messages to be fed immediately to the corresponding `vt100::Parser` instance and trigger
a render tick so that the embedded terminal widget displays up-to-date PTY output within
100ms of byte receipt — even for non-focused sessions, and including the initial screen
reconstruction triggered by the auto-attach-on-first-entry mandate.

## Acceptance Criteria

### AC-001 (traces to BC-2.09.001 postcondition 1 — on_pty_output called within one mpsc cycle)

When the TUI's IPC reader task receives `ServerToClient::PtyOutput { session_id, bytes }`,
it calls `App::on_pty_output(session_id, bytes)` within one mpsc channel cycle. The IPC
reader MUST NOT poll or buffer multiple messages before dispatching; each `PtyOutput` is
handled as it arrives.

### AC-002 (traces to BC-2.09.001 postcondition 2 — parser.process() called per message)

`App::on_pty_output` calls `App::pty_parsers.get_mut(&session_id)?.process(&bytes)`.
The parser updates its internal screen model for every `PtyOutput` message received,
regardless of whether the session is currently focused.

### AC-003 (traces to BC-2.09.001 postcondition 3 — render tick triggered; PseudoTerminal renders)

After `parser.process()`, a render tick fires immediately: `terminal.draw()` calls
`render_embedded_terminal(frame, area, &pty_parsers[session_id])` which calls
`PseudoTerminal::new(parser.screen())` and renders it into the preview pane.
The rendered output is visible within 100ms of `PtyOutput` IPC receipt.

### AC-004 (traces to BC-2.09.001 postcondition 5 — non-focused sessions update parsers; O(1) focus switch)

All sessions' `vt100::Parser` instances are updated on `PtyOutput` receipt, not only the
focused one. When the user switches focus to a previously non-focused session, the PTY
widget renders the current parser state immediately (O(1) — no re-fetch).

### AC-005 (traces to BC-2.09.001 postcondition 6 / invariant 5 — auto-attach-on-first-entry with dump-in-progress buffering)

When `App::enter_embedded_terminal(session_id)` is called for a session NOT present in
`App::pty_dump_received`:
- `App::dump_in_progress.insert(session_id.clone(), true)` is called BEFORE sending `AttachSession`.
- `ClientToServer::AttachSession { session_id }` is sent to the daemon via `.send().await`
  (backpressure; full rollback — clear `dump_in_progress[session_id]` and abort entry — on send
  failure per BC-2.09.001 Inv-3 / SS-embedded-pty 1.8.0). `enter_embedded_terminal` is `async`.
- Any `PtyOutput` messages arriving while `dump_in_progress[session_id] == true` are appended
  to `App::pending_pty_bytes[session_id]` (buffered, not fed to parser).
- On receipt of `ServerToClient::ScrollbackDumpComplete` for this session, the handler lives in
  `app.rs::handle_server_message` (NOT `event_loop.rs`):
  1. Parser is reset: `pty_parsers[session_id] = vt100::Parser::new(pty_rows, pty_cols, SCROLLBACK_ROWS)`.
  2. **[S-047 scope — NOT S-039]** Styled-cell reconstruction from `ScrollbackChunk` data is deferred to S-047. S-039 does NOT read `total_chunks`, `cursor_row`, or `cursor_col` from `ScrollbackDumpComplete`. The parser is already reset in step 1; proceeding to step 3 is correct. Add a `// S-047: styled-cell reconstruction from ScrollbackChunk rows; cursor restore from cursor_row/cursor_col` comment at this position in the handler as the S-047 extension point.
  3. Buffered bytes in `pending_pty_bytes[session_id]` are replayed through the reset parser in receipt order.
  4. `pending_pty_bytes[session_id]` is cleared.
  5. `dump_in_progress.insert(session_id.clone(), false)`.
  6. `pty_dump_received.insert(session_id.clone())`.
- This mandate does NOT apply when entering `EmbeddedTerminal` from `SessionCreation::Launching`
  (new session; no historical state).

**Re-attach after detach:** When the user detaches from a session (exits `EmbeddedTerminal` mode),
`pty_dump_received` REMOVES the session_id entry. On the next call to `enter_embedded_terminal`
for the same session_id, the session is again NOT present in `pty_dump_received`, and the full
auto-attach + buffering + dump protocol runs again from the beginning. `pty_dump_received` marks
"a complete dump was received for this continuous attach period"; it does NOT permanently suppress
re-dumps across detach/re-attach cycles. This is consistent with S-047 AC-006 (re-attach triggers
fresh dump from daemon side).

### AC-006 (traces to BC-2.09.001 invariant 2 — non-focused parsers always updated)

For a `PtyOutput` message targeting a non-focused session:
- `pty_parsers[session_id].process(&bytes)` is called.
- No PTY widget render is triggered (only the focused session's parser state is rendered).
- `pty_scroll_offsets[session_id]` is unaffected.

### AC-007 (traces to BC-2.09.001 invariant 3 — bounded IPC channel with backpressure)

The IPC reader task uses a bounded `mpsc::channel(64)` for IPC events. The reader sends via
`.send().await` (backpressure), NOT `.try_send()` (drop). A slow render loop propagates
backpressure through the IPC pipeline; no `PtyOutput` bytes are dropped.

### AC-008 (traces to BC-2.09.001 invariant 4 — SCROLLBACK_ROWS default/max; configurable via config.json)

`vt100::Parser::new(rows, cols, scrollback_rows)` is initialized with `scrollback_rows` sourced
from `~/.monocle/config.json:pty_scrollback_rows`. Two distinct cases apply:
- **Absent** (key missing, or config falls back to default → `None`): `scrollback_rows = 1000`.
- **Present** (key exists with a parseable `u32`): value is clamped to [1, 10000].
  `0 → 1` (clamped to minimum; per BC-2.09.007 EC-243); values above 10000 → 10000
  (clamped to maximum; per BC-2.09.007 EC-242). A present out-of-range value is NOT
  defaulted to 1000 — it is clamped. Only absence yields the 1000 default.

A new `vt100::Parser` is created for each new session received via `SessionListUpdate` or
`InitialState`. Parsers are removed on session GC (`SessionState::Terminated` + list removal);
`pty_dump_received` and `pty_scroll_offsets` entries for the session are also removed at that time.

### AC-011 (traces to SS-embedded-pty.md §Parser initialization — PTY_DEFAULT_ROWS/PTY_DEFAULT_COLS; F-S039-P2-004)

When `App::on_session_list_update()` or `App::on_initial_state()` creates a new `vt100::Parser`
for an arriving session, the `rows` and `cols` arguments MUST use the named constants
`PTY_DEFAULT_ROWS = 24` and `PTY_DEFAULT_COLS = 80` (defined in `monocle-core`). No hardcoded
numeric literals `24` or `80` are permitted at parser construction sites for non-attached sessions.
The scrollback dimensions are separately sourced from `App::scrollback_rows` (configured value).

### AC-012 (traces to BC-2.09.001 Invariant 5 — idempotency guard; F-S039-P2-002)

`App::on_scrollback_dump_complete(session_id, pty_rows, pty_cols, ...)` MUST begin with:

```rust
if dump_in_progress.get(&session_id) != Some(&true) {
    tracing::trace!(session_id = %session_id,
        "ScrollbackDumpComplete outside dump window — no-op");
    return;
}
```

A unit test MUST verify: when `dump_in_progress[session_id]` is `false` (or absent) and
`ScrollbackDumpComplete` arrives, the parser is NOT reset and the function returns without
modifying any state.

### AC-013 (traces to SS-embedded-pty.md §state-machine-invariants §F-S039-P2-003 — terminated-session exit-before-GC)

When `ServerToClient::SessionStateChanged { session_id, new_state: Terminated }` is received
and `app.app_mode == AppMode::EmbeddedTerminal { session_id: ref sid, .. }` where `sid == &session_id`:

- The handler calls `app.exit_embedded_terminal(session_id.clone())` (restores `prior` AppMode,
  calls `DisableMouseCapture`) BEFORE any GC (`pty_parsers.remove(...)`, etc.).
- The handler MUST NOT send `ClientToServer::DetachSession { session_id }`.
- All GC operations use `HashMap::remove()` / `HashSet::remove()` — never index access.

A unit test MUST verify: after receiving `Terminated` for the focused session while in
`EmbeddedTerminal` mode, `app.app_mode` is NOT `EmbeddedTerminal` and
`pty_parsers.contains_key(&session_id)` is `false`.

### AC-009 (traces to BC-2.09.001 edge case EC-200 — PtyOutput for unknown session_id)

When `pty_parsers.get_mut(&session_id)` returns `None` (session not yet in parsers map — race
during session creation), the bytes are silently dropped for this tick. No panic. No error log
at WARN or higher (TRACE is acceptable). The parser will be created when `SessionListUpdate`
adds the session.

### AC-010 (traces to BC-2.09.001 edge case EC-202 — high-frequency PtyOutput)

When PTY output arrives faster than the render rate (>100 messages/second), the render cycle
merges frames: multiple `PtyOutput` messages are processed before one `draw()` call. The 100ms
first-byte-to-pixel budget is still met for any single message when the render loop is
processing normally. The `mpsc::channel(64)` provides 64 slots of burst absorption.

## Tasks

- [ ] Add `App::pty_parsers: HashMap<String, vt100::Parser>` field to `App` struct in `crates/monocle-tui/src/app.rs`.
- [ ] Add `App::pty_scroll_offsets: HashMap<String, usize>` field to `App` struct.
- [ ] Add `App::pty_dump_received: HashSet<String>` field to `App` struct.
- [ ] Add `App::dump_in_progress: HashMap<String, bool>` field to `App` struct.
- [ ] Add `App::pending_pty_bytes: HashMap<String, Vec<Vec<u8>>>` field to `App` struct.
- [ ] Implement `App::on_pty_output(&mut self, session_id: String, bytes: Vec<u8>)`: check `dump_in_progress[session_id]` — if true, append to `pending_pty_bytes`; otherwise call `pty_parsers.get_mut(&session_id)?.process(&bytes)`.
- [ ] Wire `ServerToClient::PtyOutput` arm in `app.rs::handle_server_message` to call `on_pty_output` then `request_render()`. (IPC server-message dispatch lives in `app.rs::handle_server_message`, NOT in `event_loop.rs`.)
- [ ] Implement `App::enter_embedded_terminal(session_id: String)` as `async fn`: check `pty_dump_received`, set `dump_in_progress = true` before `AttachSession` send if not already dumped, send `AttachSession` via `.send().await` with rollback on failure (BC-2.09.001 Inv-3), transition `AppMode::EmbeddedTerminal`.
- [ ] Implement `ScrollbackDumpComplete` handler in `app.rs::handle_server_message`: reset parser (step 1); add `// S-047: styled-cell reconstruction from ScrollbackChunk rows; cursor restore from cursor_row/cursor_col` comment as S-047 extension point (S-039 does NOT read `total_chunks`, `cursor_row`, or `cursor_col`); replay `pending_pty_bytes` (step 3); clear buffer (step 4); set `dump_in_progress = false` (step 5); insert into `pty_dump_received` (step 6).
- [ ] Implement `render_embedded_terminal(frame, area, parser)` in `crates/monocle-tui/src/ui/embedded_terminal.rs`: creates `PseudoTerminal::new(parser.screen())` and renders into pane `Rect`.
- [ ] Initialize `vt100::Parser` for each session in `on_session_list_update()` and `on_initial_state()` using configured `scrollback_rows`.
- [ ] Load `pty_scrollback_rows` from `~/.monocle/config.json` at TUI startup; clamp 1–10000; default 1000.
- [ ] Remove parser + scroll offset + dump state on session GC (`SessionState::Terminated`).
- [ ] On `App::exit_embedded_terminal(session_id)` (detach / exit EmbeddedTerminal mode):
      `pty_dump_received.remove(&session_id)` so the next `enter_embedded_terminal` for the
      same session re-runs the full attach + dump protocol (AC-005 re-attach clause).
- [ ] Write unit test `test_BC_2_09_001_pty_output_renders_within_100ms`: tokio::time::pause, send `PtyOutput`, assert render tick, verify 100ms budget via mock.
- [ ] Write unit test `test_BC_2_09_001_non_focused_parser_updated`: two sessions; only s2 focused; send `PtyOutput` for s1; assert s1 parser updated, no render of s1 PTY widget.
- [ ] Write unit test `test_BC_2_09_001_auto_attach_on_first_entry_buffering`: simulate `AttachSession` → `ScrollbackChunk` + `ScrollbackDumpComplete`; assert buffered `PtyOutput` replayed after reset; `pty_dump_received` populated.
- [ ] Write unit test `test_BC_2_09_001_unknown_session_id_drop`: `PtyOutput` for unknown session_id; assert no panic, no WARN log.
- [ ] Define `PTY_DEFAULT_ROWS: u16 = 24` and `PTY_DEFAULT_COLS: u16 = 80` constants in `monocle-core` (e.g., `monocle-core/src/pty_defaults.rs` or a constants submodule). Use these constants in `on_session_list_update()` and `on_initial_state()` parser construction. No hardcoded `24` or `80` at parser-creation sites (AC-011).
- [ ] Add idempotency guard to `on_scrollback_dump_complete`: check `dump_in_progress.get(&session_id) != Some(&true)` and no-op with `tracing::trace!` if false. Write unit test `test_scrollback_dump_complete_idempotency_guard` verifying parser is NOT reset when guard fires (AC-012).
- [ ] In `on_session_state_changed(Terminated)` handler: if `app.app_mode == EmbeddedTerminal { session_id }`, call `exit_embedded_terminal()` BEFORE all GC operations. MUST NOT send `ClientToServer::DetachSession`. All GC via `remove()`. Write unit test `test_terminated_session_exits_embedded_mode_before_gc` (AC-013).

## Previous Story Intelligence

- **S-025** (TUI skeleton sessions): `App` struct exists in `crates/monocle-tui/src/app.rs`; `AppMode` enum exists in `monocle-core/src/app_mode.rs`. Sessions panel renders `SessionListUpdate` data.
- **S-021** (UDS IPC types): `ServerToClient::PtyOutput`, `ClientToServer::AttachSession`, `ServerToClient::ScrollbackDumpComplete` wire types must exist in `monocle-ipc`. Confirm presence; add if absent.
- **S-035** (attach/detach): `ClientToServer::AttachSession` IPC round-trip exists; daemon responds with `ScrollbackChunk*` + `ScrollbackDumpComplete`. The session-host's scrollback serialization protocol is live.
- Per CLAUDE.md conventions: bounded `mpsc::channel(64)` is the canonical pattern; no `unbounded_channel`.
- `AppMode::EmbeddedTerminal` variant is NOT yet present in `monocle-core/src/app_mode.rs` — this story adds it as part of `enter_embedded_terminal()`. The variant body is defined in SS-embedded-pty.md.

## Architecture Compliance Rules

- `vt100::Parser`, `pty_parsers`, and `pty_scroll_offsets` live in `monocle-tui/src/app.rs` (effectful shell; per Module Purity table in SS-embedded-pty.md).
- `pty_dump_received: HashSet<String>`, `dump_in_progress: HashMap<String, bool>`, `pending_pty_bytes: HashMap<String, Vec<Vec<u8>>>` are all pure-core fields (in-memory state; no I/O).
- `dump_in_progress` and `pty_dump_received` serve DIFFERENT purposes and MUST NOT be conflated (SS-embedded-pty.md §Auto-attach mandate): `dump_in_progress` is the in-flight signal; `pty_dump_received` is the completed signal.
- `dump_in_progress` MUST be set to `true` BEFORE `AttachSession` is sent — not on first `ScrollbackChunk` receipt. Live `PtyOutput` may arrive before the first chunk.
- IPC reader channel: `.send().await` (backpressure), never `.try_send()` (drop). Channel capacity 64.
- `SCROLLBACK_ROWS`: absent key → 1000 default; present value → clamped to [1, 10000] (0→1 per BC-2.09.007 EC-243; >10000→10000 per EC-242). Read from `~/.monocle/config.json:pty_scrollback_rows`.
  **S-039 OWNS this config load** — no other story loads `pty_scrollback_rows`. S-043 (scrollback
  navigation) consumes the already-loaded value via `App::scrollback_rows`; it does NOT re-load it.
  S-042 (resize debounce) OWNS the `pty_scroll_offsets[session_id] = 0` reset in the `ResizePane`
  handler. S-039 creates the `pty_scroll_offsets` HashMap; S-042 adds the reset on resize;
  S-043 verifies the reset is present and asserts the S-042-owned behavior.
- `AppMode::EmbeddedTerminal { session_id: String, prior: FocusSnapshot }` — `session_id` is String (UUID as String), not typed `uuid::Uuid`.
- Forbidden dependency: `monocle-tui` MUST NOT depend on `monocle-runtime` internals (only on `monocle-ipc` for wire types and `monocle-core` for `AppMode`).

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `vt100` | `=0.16.2` (exact) | `vt100::Parser`, `vt100::Screen` | SS-deps-pin-manifest-v2-delta.md (ADR-0011) |
| `tui-term` | `=0.3.4` (exact) | `tui_term::widget::PseudoTerminal` | SS-deps-pin-manifest-v2-delta.md (ADR-0011) |
| `ratatui` | `"0.30"` (caret) | `Frame`, `terminal.draw()`, `Rect` | SS-deps-pin-manifest.md |
| `tokio` | `=1.52` (exact) | `mpsc::channel`, `spawn`, `send().await` | SS-deps-pin-manifest.md §Exact-pinned |
| `crossterm` | `"0.29"` (caret) | `Terminal` backend used by ratatui | SS-deps-pin-manifest.md |
| `serde_json` | `=1.0.149` (exact) | `config.json` deserialization for `pty_scrollback_rows` | SS-deps-pin-manifest.md §Exact-pinned |
| `tracing` | `"0.1"` | Structured logging (TRACE for dropped PtyOutput, not WARN) | SS-deps-pin-manifest.md |

**Forbidden libraries:** Do NOT use `portable-pty` in `monocle-tui`. The PTY is owned by `monocle-session-host`. The TUI only receives serialized PTY bytes via IPC.

## File Structure Requirements

Files to CREATE:

| File | Purpose |
|------|---------|
| `crates/monocle-tui/src/ui/embedded_terminal.rs` | `render_embedded_terminal(frame, area, parser)` — creates `PseudoTerminal::new(parser.screen())` and renders into pane `Rect` |

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-tui/src/app.rs` | Add five new `App` fields: `pty_parsers`, `pty_scroll_offsets`, `pty_dump_received`, `dump_in_progress`, `pending_pty_bytes`; implement `on_pty_output()`; implement `enter_embedded_terminal()` (async — sends `AttachSession` via `.send().await` with rollback on failure per BC-2.09.001 Inv-3); implement `ScrollbackDumpComplete` handler in `handle_server_message` (IPC server-message dispatch lives here, NOT in `event_loop.rs`); wire `ServerToClient::PtyOutput` arm in `handle_server_message` to call `on_pty_output` then `request_render()` |
| `crates/monocle-core/src/app_mode.rs` | Add `AppMode::EmbeddedTerminal { session_id: String, prior: FocusSnapshot }` and `AppMode::SessionCreation { step: SessionCreationStep, prior: FocusSnapshot, launching_session_id: Option<String> }` variants; add `SessionCreationStep` enum (ProfilePicker, ProjectPicker, WorktreeConfirm, Launching) |
| `crates/monocle-tui/src/ui/mod.rs` | `pub mod embedded_terminal;` |
| `crates/monocle-tui/Cargo.toml` | Add `vt100 = "=0.16.2"`, `tui-term = "=0.3.4"` to dependencies |
| `crates/monocle-ipc/src/lib.rs` | Add `ServerToClient::PtyOutput { session_id: String, bytes: Vec<u8> }`, `ClientToServer::AttachSession { session_id: String }` variants if absent; reference `ServerToClient::ScrollbackDumpComplete` (full 6-field shape: `{ session_id, total_chunks, cursor_row, cursor_col, pty_rows, pty_cols }` — defined/owned by S-047 per SS-ipc; S-039 CONSUMES this variant, do NOT define or add a partial shape here) |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~4,500 |
| BC-2.09.001 | ~4,000 |
| SS-embedded-pty.md §PTY Widget Pipeline; §Parser ownership in TUI; §EmbeddedTerminal ENTRY; §O4 | ~10,000 |
| SS-deps-pin-manifest-v2-delta.md (vt100/tui-term versions) | ~2,000 |
| Existing App struct + AppMode + event loop | ~8,000 |
| monocle-ipc wire types (PtyOutput, AttachSession, ScrollbackDumpComplete) | ~3,000 |
| Test files to write | ~5,000 |
| **Total estimate** | **~36,500** |

Within the 30% context window bound for a Sonnet-class model (~200k tokens = 60k max per story). No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.09.001 | PTY Output Renders Within 100ms of Byte Receipt at TUI | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| `App::pty_parsers` | `monocle-tui/src/app.rs` | Effectful shell (vt100::Parser.process() is stateful mutation) |
| `App::pty_scroll_offsets` | `monocle-tui/src/app.rs` | Pure core (in-memory usize map) |
| `App::pty_dump_received` | `monocle-tui/src/app.rs` | Pure core (in-memory HashSet) |
| `App::dump_in_progress` | `monocle-tui/src/app.rs` | Pure core (in-memory HashMap<String, bool>) |
| `App::pending_pty_bytes` | `monocle-tui/src/app.rs` | Pure core (in-memory buffer) |
| `App::on_pty_output()` | `monocle-tui/src/app.rs` | Effectful shell (parser mutation + render trigger) |
| `render_embedded_terminal()` | `monocle-tui/src/ui/embedded_terminal.rs` | Effectful shell (ratatui render → terminal I/O) |
| `AppMode::EmbeddedTerminal` | `monocle-core/src/app_mode.rs` | Pure core (state variant; no I/O) |
| `AppMode::SessionCreation` | `monocle-core/src/app_mode.rs` | Pure core (state variant; no I/O) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-200 | `PtyOutput` for unknown session_id | Silently dropped (no panic, TRACE log only) |
| EC-201 | `PtyOutput` with partial UTF-8 sequence | vt100::Parser handles internally; no corruption |
| EC-202 | >100 PtyOutput messages/second | Render merges frames; 100ms budget still met per first byte; channel(64) absorbs bursts |
| EC-203 | `PtyOutput` for non-focused session | Parser updated; no PTY render; O(1) focus switch |

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because the PTY output pipeline (IPC → vt100::Parser → tui-term render) is the core data path of the embedded-pty subsystem, defined in SS-embedded-pty.md §PTY Widget Pipeline and §Parser ownership in TUI.

**Dependency Anchors:**
- S-039 depends on S-021 because `ServerToClient::PtyOutput`, `ClientToServer::AttachSession`, and `ServerToClient::ScrollbackDumpComplete` wire types must exist in `monocle-ipc`.
- S-039 depends on S-025 because the `App` struct and its basic IPC event loop infrastructure already exist; this story adds parser fields to that struct.
- S-039 depends on S-035 because `AttachSession` → `ScrollbackChunk*` + `ScrollbackDumpComplete` is the session-host-side behavior that produces the dump this story consumes; S-035 must be built first.
- S-039 blocks S-040/S-042/S-043 (direct edges; S-041 and S-044 are transitive via S-040/S-041) because all remaining SS-09 stories extend `App` with fields or behaviors that depend on `AppMode::EmbeddedTerminal` being defined (added here) and the parser infrastructure being present.
