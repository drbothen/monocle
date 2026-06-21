---
document_type: story
level: L4
story_id: S-042
epic_id: EPIC-09
version: "1.4"
status: draft
producer: vsdd-factory:story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 2
points: 5
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
  - {path: .factory/specs/behavioral-contracts/ss-09/BC-2.09.006.md, version: "1.2.0"}
  - {path: .factory/specs/architecture/SS-embedded-pty.md, version: "1.14.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.2.1"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "1.0.2"}
input-hash: "[pending]"
traces_to: "Implements BC-2.09.006 (resize detection, 50ms debounce, ResizePane IPC, local parser immediate resize)"
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

### AC-012 (traces to BC-2.09.006 edge case EC-239 — degenerate pane area: rows=0 or cols=0)

If `area.rows == 0` or `area.cols == 0`, the TUI does NOT send `ResizePane` — same as
"resize to same size as current" (no-op). Defense-in-depth: the daemon's IPC handler also
clamps zero dimensions to minimum 1 (BC-2.05.010 EC-282 / Invariant 5 — that behavior is
daemon-side, not re-implemented here).

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

### Daemon-side and Session-Host-side Scope Boundary (CRITICAL — read before touching runtime code)

**S-042 owns two things in the runtime layer and nothing else:**

1. **`monocle-session-host` binary — `DaemonToHost::Resize` match arm** (BC-2.09.006 PC-6/7):
   The session-host's event loop already has a `DaemonToHost::Resize { rows, cols }` variant
   defined (per S-033 Ruling E — all `DaemonToHost` variants are present with stubs). S-042
   must replace the stub with: `pty.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;`
   and `parser.set_size(rows, cols);`. This calls `portable-pty`'s resize, which sends SIGWINCH
   to the harness child. Add a corresponding unit test in the session-host test module.

2. **`monocle-ipc/src/lib.rs` — `ClientToServer::ResizePane` variant** (already present from
   prior delivery; verify existence only; do NOT modify the variant definition).

**S-042 MUST NOT touch:**

- `monocle-runtime/src/ipc_server.rs` — The `ClientToServer::ResizePane` dispatch arm (calling
  `handle_resize_pane`) belongs to **S-047** (Wave 8, IPC Handler Arm Ownership table). When S-042
  is dispatched, S-047 may or may not be merged yet. If S-047 is NOT yet merged, the
  `ClientToServer::ResizePane` match arm in `ipc_server.rs` will be ABSENT from the enum
  (the variant was added by S-042 to `monocle-ipc/src/lib.rs` but `ipc_server.rs` uses
  `#[non_exhaustive]` + wildcard `_ =>` arm for forward-compat per BC-2.05.010 AC-011). The
  compiler will NOT panic; the wildcard arm fires, returning `ServerToClient::Error { code:
  "invalid_request", message: "unknown variant" }`. This is the correct production-grade behavior
  for an unrecognized variant arriving at the daemon before its handler is wired — no panic, no
  silent drop, taxonomy-correct error code per the 12-code taxonomy.
- `monocle-runtime/src/session_manager/mod.rs` — The `todo!("S-033 (S-047 scope): implement
  resize_session()")` stub MUST be left untouched. `resize_session()` belongs to **S-047**.

**Wave ordering implication (non-blocking):** S-047 is Wave 8 and MUST deliver before S-042
(Wave 9) due to S-042's `depends_on: [S-039]` (which is Wave 9). The dependency graph does
NOT require a direct S-047→S-042 edge because Wave 8 completes before Wave 9 starts. When
S-042 is dispatched, S-047 will already be merged and the `ResizePane` daemon dispatch arm
will be live. No panic path will exist at delivery time. This ruling records the ownership
split for clarity but does NOT require a new dependency edge.

## Library and Framework Requirements

| Library | Version | Usage | Source |
|---------|---------|-------|--------|
| `tokio` | `=1.52` (exact) | `tokio::time::Instant`, `Duration`, IPC send | SS-deps-pin-manifest.md §Exact-pinned |
| `ratatui` | `"0.30"` (caret) | `Rect` (pane area from render frame) | SS-deps-pin-manifest.md |
| `vt100` | `=0.16.2` (exact) | `parser.set_size(rows, cols)` | SS-deps-pin-manifest-v2-delta.md |
| `tracing` | `"0.1"` | TRACE logging for debounce events | SS-deps-pin-manifest.md |

## File Structure Requirements

Files to MODIFY:

| File | Change |
|------|--------|
| `crates/monocle-tui/src/app.rs` | Add `last_sent_size: Option<(u16, u16)>`, `resize_debounce_deadline: Option<Instant>`; implement resize detection + debounce logic; clear state on EmbeddedTerminal exit |
| `crates/monocle-tui/src/ui/embedded_terminal.rs` | Expose pane `Rect` to the App's post-render logic (return `Rect` from render function or store in `App::last_pty_pane_area` per S-041) |
| `crates/monocle-ipc/src/lib.rs` | Ensure `ClientToServer::ResizePane { session_id: String, rows: u16, cols: u16 }` exists; add if absent |

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
| EC-238 | Session-host dies mid-resize | IPC error propagated; session transitions to Terminated |
| EC-239 | Pane collapses to 0 rows/cols | TUI no-op; no IPC sent |

## Subsystem Anchor Justifications

**SS-09 owns this story's scope** because resize detection, debounce, and `ResizePane` IPC are defined in SS-embedded-pty.md §Pane area and resize as part of the embedded-pty subsystem's PTY sizing contract.

**Dependency Anchors:**
- S-042 depends on S-039 because S-039 adds `pty_parsers`, `AppMode::EmbeddedTerminal`, and the render loop infrastructure that resize detection hooks into.
- S-042 BLOCKS S-043 (listed in frontmatter `blocks: [S-043]`). S-042 owns the `pty_scroll_offsets[session_id]=0` reset in the ResizePane handler (Tasks checklist item: "Also reset `pty_scroll_offsets[session_id]` to 0"). S-043 verifies that reset is present (S-043 AC-009 / invariant 3a) and explicitly states "This reset is OWNED BY S-042". S-043 must not be dispatched until S-042 is complete. S-042 may land in parallel with S-040, S-041, and S-044 after S-039.

## Trace

| Version | Change | Pass |
|---------|--------|------|
| v1.4 | Architect ruling: daemon-side scope boundary clarification. BC-2.09.006 input pin bumped v1.1.5→v1.2.0 (Story Anchor split). Architecture Compliance Rules extended with "Daemon-side and Session-Host-side Scope Boundary" section: S-042 owns the session-host `DaemonToHost::Resize` handler and verifying the `ResizePane` IPC variant exists; S-047 owns the daemon IPC dispatch arm and `resize_session()`. MUST NOT touch `ipc_server.rs` or `session_manager/mod.rs`. No AC changes — behavioral scope unchanged. | architect |
| v1.3 | Input pins updated <!-- version-pin-historical: prior versions BC-2.09.006 v1.1.3, SS-embedded-pty v1.7.0 at S-042 v1.2 authoring time -->: BC-2.09.006 bumped v1.1.3→v1.1.5 (multiple BC revisions since initial authoring), SS-embedded-pty bumped v1.7.0→v1.14.0 (major arch evolution through S-040 delivery cycle). SS-deps-pin-manifest v1.2.1 and SS-deps-pin-manifest-v2-delta v1.0.2 unchanged. No AC or task changes. | story-writer |
| v1.2 | Phase-2 Pass-5 fix burst: S-042→S-043 dep edge added (`blocks: [S-043]`); S-043 AC-009 requires the `ResizePane` handler scroll-offset reset owned by S-042. | story-writer |
| v1.1 | AC ranges and dependency corrections from Phase-2 Pass-5 housekeeping (BC-2.09.006 `AC-001..AC-012` coverage; STORY-INDEX dep column corrected). | story-writer |
| v1.0 | Initial decomposition. | Phase-2 |
