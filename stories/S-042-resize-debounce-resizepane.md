---
document_type: story
level: L4
story_id: S-042
epic_id: EPIC-09
version: "1.6"
status: done
producer: vsdd-factory:story-writer
timestamp: 2026-06-22T00:00:00Z
phase: 2
points: 8
wave: 9
tdd_mode: strict
priority: P0
depends_on: [S-039]
blocks: [S-043]
target_module: monocle-tui
subsystems: [SS-09]
behavioral_contracts: [BC-2.09.006]
verification_properties: []
estimated_days: 3
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.006.md, version: "1.3.0"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.14.0"}
  - {path: .factory/specs/architecture/SS-session-manager.md, version: "2.17.1"}
  - {path: .factory/specs/architecture/SS-ipc.md, version: "1.24.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.09.006 (full end-to-end resize: detection, 50ms debounce, ResizePane IPC, local parser immediate resize, daemon routing, resize_session, DaemonToHost::Resize forwarding, session-host pty.resize)"
# NOTE: BC-2.09.006 was ABSENT from the original Burst B dispatch list but EXISTS in the BC file set.
# It covers resize (PTY and parser resized within 2 render ticks; 50ms debounce) and must be covered per
# the story-writer obligation to cover all v1A domain capabilities. This story is the coverage vehicle.
---

# S-042: PTY Resize Detection, 50ms Debounce, and ResizePane IPC

## Narrative

As the monocle TUI in `AppMode::EmbeddedTerminal`, I want the TUI to detect when the Preview
pane area changes, debounce for 50ms, then send `ClientToServer::ResizePane { session_id, rows, cols }`
to the daemon — while immediately resizing the local `vt100::Parser` without debounce — so
that the harness child receives `SIGWINCH` and adjusts its output formatting to the correct
terminal dimensions within 2 render ticks of the area change.

## Acceptance Criteria

### AC-001 (traces to BC-2.09.006 postcondition 1 — size change detection per render cycle)

At each render cycle while `AppMode::EmbeddedTerminal` is active, the TUI checks:
`area.rows != parser.screen().size().0 || area.cols != parser.screen().size().1`.
If the area has changed AND 50ms has elapsed since the first detected change in the current
debounce window, a resize is triggered.

### AC-002 (traces to BC-2.09.006 postcondition 2 — ResizePane IPC sent on debounce expiry)

When the debounce window expires, `ClientToServer::ResizePane { session_id, rows: area.rows, cols: area.cols }`
is sent over IPC within the same render cycle as the size change detection.

### AC-003 (traces to BC-2.09.006 postcondition 3 — local parser resized immediately, not debounced)

`App::pty_parsers[session_id].set_size(area.rows, area.cols)` is called synchronously on EVERY
render cycle where the pane area differs from the parser size — without waiting for the 50ms
debounce. The local render uses the correct new size immediately; the IPC `ResizePane` is
debounced. These two operations are independent.

### AC-004 (traces to BC-2.09.006 postcondition 8 — total resize latency ≤ 100ms)

Total end-to-end resize latency from user terminal resize to PTY resize is ≤ 100ms:
50ms debounce + ~50ms for IPC round-trip and PTY resize in session-host. The 2-render-tick
bound (≈ 33ms at 60fps) for steps 2–6 of the resize sequence is met.

### AC-005 (traces to BC-2.09.006 invariant 1 — 50ms debounce; only final size per window sent)

The debounce timer tracks the FIRST detected size change in a window. A `ResizePane` message is
sent only once per 50ms window, encoding the CURRENT (final/stable) dimensions at timer expiry.
Intermediate sizes within the window are discarded. Rapid resize events (continuous drag) coalesce
into a single `ResizePane` per 50ms window.

### AC-006 (traces to BC-2.09.006 invariant 2 — resize sent only when pending_size != last_sent_size)

`App::last_sent_size: Option<(u16, u16)>` tracks the last-sent `(rows, cols)`. A resize message
is sent ONLY when `pending_size != last_sent_size` AND the 50ms debounce has elapsed. Resizing
to the same dimensions as already sent produces no IPC message.

### AC-007 (traces to BC-2.09.006 invariant 3 — resize sent for focused session only)

`ResizePane` is sent only for the session focused in `AppMode::EmbeddedTerminal { session_id }`.
Resizing the terminal while viewing the sessions panel (Dashboard mode) sends NO `ResizePane` IPC.
Non-focused sessions are NOT resized.

### AC-008 (traces to BC-2.09.006 invariant 4 — local parser resize is synchronous; not debounced)

`pty_parsers[session_id].set_size(new_rows, new_cols)` is called immediately when the pane area
changes — without waiting for the debounce timer. The local rendering is correct at the new size
on the next render tick even if the IPC round-trip has not yet completed.

### AC-009 (traces to BC-2.09.006 edge case EC-235 — continuous drag coalesced)

When the user rapidly resizes the terminal (continuous drag), only one `ResizePane` is sent per
50ms window — encoding the final stable size in that window. Multiple intermediate sizes are
discarded. The local parser is updated on EACH intermediate size change for correct rendering.

### AC-010 (traces to BC-2.09.006 edge case EC-236 — resize in Dashboard mode; no IPC)

When `AppMode::Dashboard` is active (not `EmbeddedTerminal`), terminal resize events do NOT
trigger `ResizePane` IPC. Local parsers are NOT resized. They will be resized when the session
is next entered in `EmbeddedTerminal` mode.

### AC-011 (traces to BC-2.09.006 edge case EC-237 — resize to same size; no-op)

If `area.rows == parser.screen().size().0 && area.cols == parser.screen().size().1`, no IPC
message is sent and `last_sent_size` is unchanged. This is a no-op.

### AC-012 (traces to BC-2.09.006 edge case EC-239 — degenerate pane area: rows=0 or cols=0; TUI side)

If `area.rows == 0` or `area.cols == 0`, the TUI does NOT send `ResizePane` — same as
"resize to same size as current" (no-op). The daemon IPC handler also clamps zero dimensions
as defense-in-depth (AC-014 below).

### AC-013 (traces to BC-2.09.006 postcondition 4 — daemon routes ResizePane → resize_session)

The daemon's `ipc_server.rs` match arm for `ClientToServer::ResizePane { session_id, rows, cols }`
calls `session_manager.resize_session(&session_id, rows.max(1), cols.max(1))`. Zero-dim clamp
(`rows.max(1)` and `cols.max(1)`) is applied BEFORE the call. All transport errors from
`resize_session()` are WARN-dropped — no `ServerToClient::Error` response is sent to the TUI
for resize failures (BC-2.05.010 Invariant 6 ResizePane carve-out). The `resize_session()`
implementation: (a) validates `session_id` is a non-empty string; (b) looks up the session in
the registry — on `SessionNotFound` WARN-drops; (c) sends `DaemonToHost::Resize { rows, cols }`
to the session-host via its `host_conn.writer`; (d) on send failure (SessionHostDead, Io, etc.)
WARN-drops per the ResizePane carve-out. `resize_session()` MUST NOT return a result that maps
to any wire error code — the IPC handler unconditionally discards the `Result`.

### AC-014 (traces to BC-2.09.006 edge case EC-239 — daemon zero-dimension defense-in-depth)

If `ClientToServer::ResizePane { rows: 0, cols: _ }` or `{ rows: _, cols: 0 }` arrives at the
daemon (defense-in-depth against a TUI-side guard failure), the IPC handler clamps: `rows.max(1)`
and `cols.max(1)` before calling `resize_session()`. The clamped values — never 0 — are forwarded
to the session-host. A `tracing::warn!` is emitted on the clamped-from-zero path.

### AC-015 (traces to BC-2.09.006 postcondition 5 — SessionManager sends DaemonToHost::Resize)

`SessionManager::resize_session()` serializes `DaemonToHost::Resize { rows, cols }` and writes
it to the session-host via `monocle_ipc::uds::write_framed_to_stream` on the
`SessionEntry::host_conn.writer`. On `SessionState::Running`, `host_conn` is Some; on
`SessionState::Launching` (host_conn still None), `resize_session()` returns
`Err(SessionError::SessionNotReady)` which the IPC handler WARN-drops per AC-013.
On `SessionState::Detached` or `Terminating`, `resize_session()` also WARN-drops per the
state-disposition table (SS-session-manager.md §Terminated-in-grace dispositions row for resize).

### AC-016 (traces to BC-2.09.006 edge case EC-238 — session-host dies mid-resize)

If `resize_session()` returns `Err(SessionError::SessionHostDead)` or any IO error from the
`DaemonToHost::Resize` write, the IPC handler emits `tracing::warn!` and returns without
sending `ServerToClient::Error` to the TUI. The session transitions to Terminated via the
standard watchdog path — the resize failure itself does not drive the state transition.

## Tasks

- [ ] Add `App::last_sent_size: Option<(u16, u16)>` and `App::resize_debounce_deadline: Option<Instant>` fields to the `App` struct in `crates/monocle-tui/src/app.rs`.
- [ ] Implement resize detection logic in the render loop (called from `terminal.draw()` callback or post-draw): compare `area.rows/cols` with `parser.screen().size()`; if different and `area.rows > 0 && area.cols > 0`:
  - Call `pty_parsers[session_id].set_size(area.rows, area.cols)` immediately (synchronous; not debounced).
  - If `resize_debounce_deadline.is_none()`, set `resize_debounce_deadline = Some(Instant::now() + Duration::from_millis(50))`.
  - Also reset `pty_scroll_offsets[session_id]` to 0 (resize reflows; per BC-2.09.001 Invariant 6 / SS-embedded-pty.md §Scrollback offset invariants).
- [ ] Add a debounce-check tick in the event loop: after each render tick, check if `resize_debounce_deadline.map_or(false, |d| Instant::now() >= d)`:
  - If true AND `last_sent_size != Some((area.rows, area.cols))`: send `ClientToServer::ResizePane { session_id, rows: area.rows, cols: area.cols }` over IPC; update `last_sent_size`; clear `resize_debounce_deadline`.
- [ ] Ensure `resize_debounce_deadline` and `last_sent_size` are cleared on `AppMode` exit from `EmbeddedTerminal`.
- [ ] Write unit test `test_BC_2_09_006_resize_sends_resizepane_after_50ms`: tokio::time::pause; trigger pane resize; advance time 49ms → no IPC; advance 1ms more → `ResizePane` sent with correct dimensions.
- [ ] Write unit test `test_BC_2_09_006_local_parser_resized_immediately`: resize before debounce expiry; assert `pty_parsers[session_id].screen().size()` == new dimensions immediately; assert no IPC yet.
- [ ] Write unit test `test_BC_2_09_006_rapid_resize_coalesced`: three sizes within 50ms; assert only one `ResizePane` for the final size.
- [ ] Write unit test `test_BC_2_09_006_resize_to_same_size_no_op`: resize to current size; assert no IPC.
- [ ] Write unit test `test_BC_2_09_006_dashboard_mode_no_resizepane`: resize in Dashboard mode; assert no `ResizePane` sent.
- [ ] Write unit test `test_BC_2_09_006_zero_dimensions_no_op`: area.rows=0; assert no IPC sent.
- [ ] Add `ClientToServer::ResizePane { session_id: String, rows: u16, cols: u16 }` variant to `crates/monocle-ipc/src/lib.rs` if not already present (S-040 may have added it; verify and add if absent).
- [ ] Add `Ok(ClientToServer::ResizePane { session_id, rows, cols }) =>` match arm to `crates/monocle-runtime/src/ipc_server.rs` calling `handle_resize_pane(session_id, rows, cols, &state).await`. Implement `handle_resize_pane()` in the same file: clamp `rows.max(1)` and `cols.max(1)`, call `session_manager.resize_session(id, clamped_rows, clamped_cols)`, WARN-drop all errors (no `ServerToClient::Error` response). Emit `tracing::warn!` on zero-dim clamp path.
- [ ] Implement `resize_session()` in `crates/monocle-runtime/src/session_manager/mod.rs` (replacing the `todo!("S-033 (S-047 scope): implement resize_session()")` stub): validate `session_id` non-empty; look up session in registry (WARN-drop on SessionNotFound — resize carve-out); on SessionNotReady/Launching (host_conn: None) WARN-drop; on Running: write `DaemonToHost::Resize { rows, cols }` to `host_conn.writer` via `monocle_ipc::uds::write_framed_to_stream`; WARN-drop IO/SessionHostDead errors. Must NOT return a code that maps to any `ServerToClient::Error` wire code.
- [ ] Write unit test `test_BC_2_09_006_daemon_resize_pane_routes_daemon_to_host`: mock SessionManager; send `ClientToServer::ResizePane { rows: 30, cols: 100 }` via IPC; assert `DaemonToHost::Resize { rows: 30, cols: 100 }` received on session-host control channel; no `ServerToClient::Error` emitted.
- [ ] Write unit test `test_BC_2_09_006_daemon_zero_dim_clamp`: send `ResizePane { rows: 0, cols: 80 }`; assert `DaemonToHost::Resize { rows: 1, cols: 80 }` forwarded; WARN emitted; no Error response.
- [ ] Write unit test `test_BC_2_09_006_daemon_session_not_found_warn_drop`: send `ResizePane` for non-existent session_id; assert no `ServerToClient::Error` returned; WARN logged.
- [ ] Write unit test `test_BC_2_09_006_daemon_host_dead_warn_drop` (EC-238): send `ResizePane` while session-host connection is dead; assert WARN emitted; no `ServerToClient::Error`; session transitions to Terminated via standard watchdog path (this is an integration-level assertion — can use mock that returns `Err(SessionHostDead)`).
- [ ] Write integration test `test_BC_2_09_006_run_loop_invokes_resize_detection`: verify that the run loop actually calls `check_resize_debounce()` (or equivalent) on each tick when in EmbeddedTerminal mode — this is an anti-dead-code test. Approach: use a mock IPC sink; trigger a pane area change in the App state; assert `ResizePane` is sent after debounce expires WITHOUT manually calling the resize function from outside the loop. This guards against the class of bug where the function exists but is never wired into the loop.

## Previous Story Intelligence

- **S-039** (PTY output pipeline): `App::pty_parsers` and `App::pty_scroll_offsets` exist. `AppMode::EmbeddedTerminal` variant is defined. The render loop calls `render_embedded_terminal()`. This story hooks into that render loop to add resize detection.
- **S-025** (TUI skeleton): The ratatui `terminal.draw()` callback provides the `Frame` reference and therefore the current pane `Rect`. Confirm how pane area is computed (layout-split in the draw callback) and add resize detection there.
- The scroll offset reset on resize is required by SS-embedded-pty.md §Scrollback offset invariants ("resize reflows content; old offset is meaningless"). Implement it here to avoid a follow-up finding.

## Architecture Compliance Rules

- Resize detection lives in `monocle-tui` (effectful shell) — it requires reading the rendered `Rect` and sending IPC.
- Local `pty_parsers[session_id].set_size()` is synchronous and MUST happen before the debounce timer check. Do NOT debounce the local parser resize.
- `ResizePane` is only sent in `AppMode::EmbeddedTerminal`. No resize in Dashboard or other modes.
- Zero dimensions (`area.rows==0 || area.cols==0`) → no-op; no IPC. The daemon has defense-in-depth but the TUI should not send degenerate sizes.
- `pty_scroll_offsets[session_id]` MUST be reset to 0 on resize (SS-embedded-pty.md §Scrollback offset invariants).
- Forbidden: no `std::thread::sleep` in the event loop. Use `tokio::time::Instant` for debounce tracking.

### Daemon-side and Session-Host-side Scope (CRITICAL — read before touching runtime code)

**S-042 owns the FULL end-to-end resize pipeline per human ruling 2026-06-21:**

1. **`monocle-ipc/src/lib.rs` — `ClientToServer::ResizePane` variant**: ensure it exists
   (`session_id: String, rows: u16, cols: u16`). Add if absent.

2. **`monocle-runtime/src/ipc_server.rs` — `ClientToServer::ResizePane` dispatch arm** (BC-2.09.006 PC-4):
   Add `Ok(ClientToServer::ResizePane { session_id, rows, cols }) =>` arm; implement
   `handle_resize_pane()`: clamp zero dims, call `session_manager.resize_session()`, WARN-drop
   all errors. `ClientToServer` has NO `#[non_exhaustive]` and NO wildcard arm — the match is
   exhaustive; the new arm MUST be added or the code will not compile.

3. **`monocle-runtime/src/session_manager/mod.rs` — `resize_session()` implementation** (BC-2.09.006 PC-5):
   Replace `todo!("S-033 (S-047 scope): implement resize_session()")` with the real body: look
   up session, send `DaemonToHost::Resize { rows, cols }` to session-host via `host_conn.writer`,
   WARN-drop SessionNotFound/SessionNotReady/SessionHostDead/Io errors per the ResizePane carve-out
   (SS-session-manager.md §resize row in state-disposition table — WARN-drop for all states).

4. **`monocle-session-host/src/main.rs` — `DaemonToHost::Resize` match arm** (BC-2.09.006 PC-6/7):
   Replace the stub with: `pty.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;`
   and `parser.set_size(rows, cols);`. This triggers SIGWINCH to the harness child via
   `portable-pty`.

**Why S-042 owns the daemon leg (not S-047):** `ClientToServer` is exhaustive (no `#[non_exhaustive]`,
no wildcard arm). Adding `ResizePane` to the enum without a matching arm in `ipc_server.rs` is a
compile error. S-047 is `status: draft`, Wave 8, with undelivered deps (S-046 ← S-032). Shipping
S-042 with `resize_session()` as `todo!()` would deliver a live panic path (`todo!()` panics on
call). Both facts make a split ownership approach non-production-grade.

**Dependency reachability confirmation (S-042 with existing merged deps):**
- `DaemonToHost::Resize { rows, cols }` — defined in `monocle-ipc/src/types.rs` since S-033 (merged).
- `PtySize` — `portable_pty::PtySize` (SS-deps-pin-manifest.md; used in session-host since S-033).
- `host_conn.writer` — `SessionEntry::host_conn` defined in S-033 (merged); the write path is the
  same as used by `send_key_input()` (S-040, merged). No new `depends_on` edge required.
- All required types are reachable from S-042's existing `depends_on: [S-039]` chain (S-039 ← S-033/S-034/S-035 all merged).

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tokio` | `=1.52` (exact) | `tokio::time::Instant`, `Duration`, IPC send | SS-deps-pin-manifest.md §Exact-pinned |
| `ratatui` | `"0.30"` (caret) | `Rect` (pane area from render frame) | SS-deps-pin-manifest.md |
| `vt100` | `=0.16.2` (exact) | `parser.set_size(rows, cols)` (TUI + session-host) | SS-deps-pin-manifest-v2-delta.md |
| `portable-pty` | `"0.8"` (caret) | `PtySize`, `pty.resize()` in session-host | SS-deps-pin-manifest.md |
| `monocle-ipc` | workspace | `DaemonToHost::Resize`, `ClientToServer::ResizePane`, IPC framing | workspace |
| `tracing` | `"0.1"` | TRACE/WARN logging for debounce, zero-dim clamp, WARN-drop paths | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-tui/src/app.rs` | Add `last_sent_size: Option<(u16, u16)>`, `resize_debounce_deadline: Option<Instant>`; implement resize detection + debounce logic; clear state on EmbeddedTerminal exit |
| `crates/monocle-tui/src/ui/embedded_terminal.rs` | Expose pane `Rect` to the App's post-render logic (return `Rect` from render function or store in `App::last_pty_pane_area` per S-041) |
| `crates/monocle-ipc/src/lib.rs` | Ensure `ClientToServer::ResizePane { session_id: String, rows: u16, cols: u16 }` exists; add if absent |
| `crates/monocle-runtime/src/ipc_server.rs` | Add `Ok(ClientToServer::ResizePane { session_id, rows, cols }) =>` match arm; implement `handle_resize_pane()` with zero-dim clamp (`rows.max(1)`, `cols.max(1)`), call `session_manager.resize_session()`, WARN-drop all errors, no `ServerToClient::Error` response |
| `crates/monocle-runtime/src/session_manager/mod.rs` | Replace `todo!("S-033 (S-047 scope): implement resize_session()")` stub with real implementation: session lookup, `DaemonToHost::Resize` forwarding via `host_conn.writer`, WARN-drop all errors per ResizePane carve-out |
| `crates/monocle-session-host/src/main.rs` | Replace stub `DaemonToHost::Resize { rows, cols }` match arm with: `pty.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;` and `parser.set_size(rows, cols);` |

## Token Budget Estimate

| Source | Estimated Tokens |
|--------|-----------------|
| This story spec | ~4,000 |
| BC-2.09.006 | ~3,000 |
| SS-embedded-pty.md §Pane area and resize; §Scrollback offset invariants | ~3,000 |
| Existing App struct + render loop (from S-039) | ~5,000 |
| Test files to write | ~4,000 |
| **Total estimate** | **~19,000** |

Within the 30% context window bound. No split required.

## Behavioral Contracts

| BC | Title | Version |
|----|-------|---------|
| BC-2.09.006 | Resize — PTY and Parser Resized Within 2 Render Ticks of Pane Area Change; 50ms Debounce | (see inputs: frontmatter) |

## Architecture Mapping

| Component | Module/File | Pure/Effectful |
|-----------|------------|----------------|
| Resize detection + debounce logic | `monocle-tui/src/app.rs` | Effectful shell (timer + IPC send) |
| `pty_parsers[id].set_size()` call | `monocle-tui/src/app.rs` | Effectful shell (parser mutation) |
| `App::last_sent_size` | `monocle-tui/src/app.rs` | Pure core (cached state) |
| `App::resize_debounce_deadline` | `monocle-tui/src/app.rs` | Pure core (timer tracking) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-235 | Continuous drag resize | Single `ResizePane` per 50ms window; local parser resized each tick |
| EC-236 | Resize in Dashboard mode | No IPC; no local resize |
| EC-237 | Resize to same size | No-op; no IPC |
| EC-238 | Session-host dies mid-resize | `resize_session()` returns Err; IPC handler WARN-drops; session transitions to Terminated via watchdog path (AC-016) |
| EC-239 | Pane collapses to 0 rows/cols | TUI no-op (AC-012); daemon zero-dim clamp defense-in-depth (AC-014) |

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because resize detection, debounce, and `ResizePane` IPC are defined in SS-embedded-pty.md §Pane area and resize as part of the embedded-pty subsystem's PTY sizing contract.

**Dependency Anchors:**
- S-042 depends on S-039 because S-039 adds `pty_parsers`, `AppMode::EmbeddedTerminal`, and the render loop infrastructure that resize detection hooks into.
- S-042 BLOCKS S-043 (listed in frontmatter `blocks: [S-043]`). S-042 owns the `pty_scroll_offsets[session_id]=0` reset in the ResizePane handler (Tasks checklist item: "Also reset `pty_scroll_offsets[session_id]` to 0"). S-043 verifies that reset is present (S-043 AC-009 / invariant 3a) and explicitly states "This reset is OWNED BY S-042". S-043 must not be dispatched until S-042 is complete. S-042 may land in parallel with S-040, S-041, and S-044 after S-039.

## Trace

| Version | Change | Pass |
|---------|--------|------|
| v1.5 | Human ruling 2026-06-21: expand S-042 to end-to-end resize. BC-2.09.006 input pin bumped v1.2.0→v1.3.0. SS-session-manager (v2.17.0 at v1.5 authoring time <!-- version-pin-historical -->) and SS-ipc v1.24.0 added to inputs. Points 5→8 (daemon impl + 4 new daemon tests + 1 run-loop wiring test). AC-013..AC-016 added (daemon leg: ResizePane dispatch, zero-dim clamp, resize_session impl, EC-238 WARN-drop). AC-012 clarified (TUI side only; daemon side in AC-014). Architecture Compliance Rules "MUST NOT touch" restriction removed; replaced with full ownership statement. File Structure Requirements: ipc_server.rs, session_manager/mod.rs, monocle-session-host/src/main.rs added. Tasks: 5 daemon tests + 1 run-loop wiring test added. Library table: portable-pty + monocle-ipc rows added. Edge cases: EC-238/EC-239 descriptions updated to cite ACs. Dependency reachability confirmed: no new depends_on edge required. | architect |
| v1.4 | Architect ruling: daemon-side scope boundary clarification. BC-2.09.006 input pin bumped v1.1.5→v1.2.0 (Story Anchor split). Architecture Compliance Rules extended with "Daemon-side and Session-Host-side Scope Boundary" section: S-042 owns the session-host `DaemonToHost::Resize` handler and verifying the `ResizePane` IPC variant exists; S-047 owns the daemon IPC dispatch arm and `resize_session()`. MUST NOT touch `ipc_server.rs` or `session_manager/mod.rs`. No AC changes — behavioral scope unchanged. | architect |
| v1.3 | Input pins updated <!-- version-pin-historical: prior versions BC-2.09.006 v1.1.3, SS-embedded-pty v1.7.0 at S-042 v1.2 authoring time -->: BC-2.09.006 bumped v1.1.3→v1.1.5 (multiple BC revisions since initial authoring), SS-embedded-pty bumped v1.7.0→v1.14.0 (major arch evolution through S-040 delivery cycle). SS-deps-pin-manifest v1.2.1 and SS-deps-pin-manifest-v2-delta v1.0.2 unchanged. No AC or task changes. | story-writer |
| v1.2 | Phase-2 Pass-5 fix burst: S-042→S-043 dep edge added (`blocks: [S-043]`); S-043 AC-009 requires the `ResizePane` handler scroll-offset reset owned by S-042. | story-writer |
| v1.1 | AC ranges and dependency corrections from Phase-2 Pass-5 housekeeping (BC-2.09.006 `AC-001..AC-012` coverage; STORY-INDEX dep column corrected). | story-writer |
| v1.0 | Initial decomposition. | Phase-2 |
