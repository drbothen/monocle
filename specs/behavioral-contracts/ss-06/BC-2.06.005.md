---
document_type: behavioral-contract
level: L3
version: "1.0.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "[pending]"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.005: Sessions Panel: Session List Renders from IPC State

## Description

The Sessions Panel in `monocle-tui` renders one row per `EnrichedSession` received from
the daemon via `SessionListUpdate` IPC messages. The panel never reads from disk, process
state, or any source other than the most recently received IPC message. Each row displays
six columns: harness icon, project name, phase tag, token count, cost, and uptime. When
the session list is empty, the panel renders a two-line empty state message. The Sessions
Panel occupies the left 60% of the main area in Dashboard layout.

## Preconditions

1. The TUI has connected to the daemon UDS and received at least one `SessionListUpdate`
   IPC message (or an initial state push per BC-2.05.002 which includes the session list).
2. `EnrichedSession` is the struct carried by `SessionListUpdate`. It includes:
   - `engine_metadata: EngineMetadata` (with `icon: char`, `display_name: String`)
   - `project_name: String` (derived from working directory basename)
   - `phase_tag: Option<String>` (from `FactoryAdapter`; `None` if not a factory project)
   - `token_count: u64`
   - `cost_usd: Option<f64>`
   - `uptime: Duration` (wall clock since `SessionStart` hook)
3. `AppMode` is `Dashboard` or `Filtering` (Sessions Panel is visible in these modes).
4. The draw loop runs at ~60fps (16ms tick rate).

## Postconditions

1. **Row per session:** For each `EnrichedSession` in `app.sessions`, exactly one row is
   rendered in the Sessions Panel. Rows are rendered in the order received from the daemon
   (most recently started session last, unless the daemon orders differently — ordering is
   daemon-determined and the TUI renders it as-is).
2. **Column layout:**
   - Icon column: renders `EngineMetadata::icon` as a single character. For Claude Code in
     Phase 1, this is `●` (U+25CF BLACK CIRCLE).
   - Project column: renders `EnrichedSession::project_name` (truncated with `…` if the
     column is too narrow to display the full name).
   - Phase column: renders `EnrichedSession::phase_tag` if `Some(tag)`; renders blank
     (empty string) if `None`.
   - Tokens column: renders `EnrichedSession::token_count` in human-readable format
     (e.g., `142k` for 142,000; `1.2M` for 1,200,000; raw count for < 1,000).
   - Cost column: renders `EnrichedSession::cost_usd` as `$N.NN` (two decimal places) if
     `Some(cost)`; renders blank if `None`.
   - Uptime column: renders `EnrichedSession::uptime` as `HH:MM:SS` (e.g., `00:03:47`).
3. **Empty state:** When `app.sessions` is empty (zero `EnrichedSession` items), the
   Sessions Panel renders exactly two lines:
   ```
   No sessions detected
   Start Claude Code in any terminal to see it here.
   ```
   No other content is rendered in the panel area.
4. **No direct process reads:** The Sessions Panel does not call `std::fs`, `std::process`,
   or any OS API. All displayed data comes from `app.sessions` which is populated
   exclusively by IPC `SessionListUpdate` messages.
5. **Panel width constraint:** Sessions Panel occupies 60% of the main area width in
   Dashboard layout. In Fullscreen mode (BC-2.06.007), it occupies 100% of the main area.
6. **Selected row highlighting:** The currently-focused row (as determined by the
   `FocusSnapshot::Sessions` context) is rendered with the terminal's selection highlight
   style. In Phase 1, "focused row" is the first row (no row-level cursor navigation within
   the panel — `Enter` immediately fullscreens the focused session).

## Invariants

1. The Sessions Panel never initiates I/O. It is a pure render of `app.sessions`.
2. The token count formatter is deterministic: the same `u64` always produces the same
   string representation.
3. Phase tag renders blank (not "None" or "N/A") when `phase_tag` is `None`. Blank
   is the correct empty state for the Phase column — it signals "not a factory project"
   without cluttering the display.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-082 | Session list updated (new `SessionListUpdate` arrives) while panel is rendering | Next draw tick renders the updated list; no visual tearing (ratatui double-buffer) |
| EC-083 | Session with very long project name (e.g., 80-character monorepo path) | Truncated to fit column width with trailing `…`; no panic, no line wrap |
| EC-084 | Session with `cost_usd = None` (harness does not report cost) | Cost column renders blank |
| EC-085 | Session with `token_count = 0` | Tokens column renders `0` (not blank) |
| EC-086 | Session with `uptime` exceeding 99 hours (e.g., 100:00:00) | Uptime column renders `100:00:00` (extended format); no truncation |
| EC-087 | Session list transitions from non-empty to empty (all sessions ended) | Empty state message renders immediately on the next draw tick after `app.sessions` is cleared |
| EC-088 | `icon` character for a future non-Claude harness (Phase 2) | Renders whatever `char` the engine provides; no hardcoded Claude-specific logic in the render path |

## Canonical Test Vectors

| Input | Expected Rendered Content | Category |
|-------|--------------------------|----------|
| `app.sessions = []` | Two-line empty state message | happy-path |
| `app.sessions = [session with project="monocle", phase_tag=Some("phase-3"), tokens=437000, cost=None, uptime=3h47m]` | Row: `● monocle phase-3 437k [blank] 03:47:00` | happy-path |
| `app.sessions = [session with cost=0.83]` | Cost column: `$0.83` | happy-path |
| `app.sessions = [session with tokens=999]` | Tokens column: `999` (no suffix) | edge-case |
| `app.sessions = [session with tokens=1000]` | Tokens column: `1k` | edge-case |
| `app.sessions = [session with tokens=1200000]` | Tokens column: `1.2M` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Empty state message renders when `app.sessions` is empty | Unit test (ratatui TestBackend) |
| VP-TBD | Token formatter: 999 → "999", 1000 → "1k", 142000 → "142k", 1200000 → "1.2M" | Unit test (table-driven) |
| VP-TBD | Cost formatter: None → "", 0.83 → "$0.83" | Unit test (table-driven) |
| VP-TBD | Sessions panel has no `std::fs`, `std::process`, or OS API imports | `cargo check` + `grep` in CI |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the "sessions panel" component of CAP-006: the primary panel the user sees when managing multiple Claude Code sessions |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — the Sessions Panel is read-only: it renders IPC-pushed data and performs no writes) |
| Architecture Module | monocle-tui (draw_dashboard(), draw_sessions_panel()) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.0.0 §Panel Architecture §Sessions Panel (column layout table, empty state, filter mode, fullscreen) |
| Cross-Ref | BC-2.05.003 (SessionListUpdate IPC message — the source of `app.sessions`), BC-2.06.006 (filter mode for Sessions Panel), BC-2.06.007 (Enter → Fullscreen) |
| Test File | `monocle-tui/tests/sessions_panel.rs` |
| Test Name | `test_BC_2_06_005_sessions_panel_renders_from_ipc_state` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.003] — depends on: `SessionListUpdate` IPC message is the sole data source for this panel
- [BC-2.06.006] — composes with: pressing `/` in Sessions Panel enters Filtering mode over this panel
- [BC-2.06.007] — composes with: pressing `Enter` on a focused session row triggers Fullscreen for this panel

## Architecture Anchors

- `architecture/SS-tui.md#panel-architecture` — Sessions Panel column layout table
- `architecture/SS-tui.md#panel-architecture` — Empty state render specification
- `architecture/SS-tui.md#rendering-architecture` — Dashboard layout: Sessions 60% left, Event Ribbon 40% right

## Story Anchor

S-TBD — Implement Sessions Panel renderer with 6-column layout, empty state, token/cost formatters (filled by story-writer)

## VP Anchors

- VP-TBD — ratatui TestBackend snapshot tests for Sessions Panel column layout and empty state

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.005 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.0.0 §Panel Architecture §Sessions Panel (column layout table, empty
  state, filter mode, fullscreen); prd-expansion-scope.md §3.3 BC-2.06.005 description;
  ARCH-INDEX.md §Capability Traceability SS-06.
- Postcondition 6 ("selected row highlighting") specifies Phase 1 behavior: no intra-panel
  cursor navigation. `Enter` immediately fullscreens the single focused session. Phase 2
  will add row-level cursor navigation; this BC will be superseded at that point.
- EC-088 future-proofs the icon render path: the `icon: char` field comes from
  `EngineMetadata`, which is harness-specific. The Sessions Panel render must not hardcode
  Claude-specific logic.
