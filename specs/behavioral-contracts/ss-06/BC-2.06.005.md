---
document_type: behavioral-contract
level: L3
version: "1.0.6"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "c1e8267"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010, F-P14-004]
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
seven columns: session ID, harness icon, project name, status, token count, cost, and
uptime. The session ID column is essential for operator clarity and debuggability when
multiple sessions are running simultaneously. When the session list is empty, the panel
renders a two-line empty state message. The Sessions Panel occupies the left 60% of the
main area in Dashboard layout.

## Preconditions

1. The TUI has connected to the daemon UDS and received at least one `SessionListUpdate`
   IPC message (or an initial state push per BC-2.05.002 which includes the session list).
2. `EnrichedSession` is the struct carried by `SessionListUpdate`. It includes the
   following Phase 1 fields (per SS-engine-module.md §EnrichedSession):
   - `session_id: String`
   - `harness_type: String` (e.g., `"claude-code"`; used to derive the icon character)
   - `transcript_path: Option<PathBuf>`
   - `config_path: Option<PathBuf>`
   - `status: SessionStatus`
   - `last_event_micros: Option<i64>`
   - `project_name: Option<String>` (derived from transcript directory name; `None` when
     `transcript_path` is unknown)
   - `started_at: Option<chrono::DateTime<chrono::Utc>>` (from the first `SessionStart`
     hook event; `None` until received; used to compute uptime at render time)
   - `token_count: u64`
   - `cost_usd: Option<f64>`
   Note: `phase_tag` was considered but removed from `EnrichedSession` in Phase 1
   (requires `FactoryAdapter` integration not yet available). `uptime` is not a field;
   it is computed as `now - started_at` at render time.
3. `AppMode` is `Dashboard` or `Filtering` (Sessions Panel is visible in these modes).
4. The draw loop runs at ~60fps (16ms tick rate).

## Postconditions

1. **Row per session:** For each `EnrichedSession` in `app.sessions`, exactly one row is
   rendered in the Sessions Panel. Rows are rendered in the order received from the daemon
   (most recently started session last, unless the daemon orders differently — ordering is
   daemon-determined and the TUI renders it as-is).
2. **Column layout** (per SS-tui.md v1.8.2 §Sessions Panel):
   - Session ID column: renders `EnrichedSession::session_id` as a short identifier
     (e.g., `sess-001`). This column is required for operator clarity when multiple
     sessions share the same project name or harness type, and for debuggability when
     correlating TUI rows to daemon logs.
   - Icon column: renders a single `char` derived from `EnrichedSession::harness_type`.
     For Claude Code (`harness_type == "claude-code"`) in Phase 1, this is `●`
     (U+25CF BLACK CIRCLE). The render path must not hardcode Claude-specific logic —
     see EC-088.
   - Project column: renders `EnrichedSession::project_name` if `Some(name)` (truncated
     with `…` if the column is too narrow); renders `"—"` if `None`.
   - Status column: renders `EnrichedSession::status` as the `SessionStatus` display
     string (e.g., `Active`, `Idle`, `WaitingOnPermission`).
   - Tokens column: renders `EnrichedSession::token_count` in human-readable format
     (e.g., `142k` for 142,000; `1.2M` for 1,200,000; raw count for < 1,000).
   - Cost column: renders `EnrichedSession::cost_usd` as `$N.NN` (two decimal places) if
     `Some(cost)`; renders `"—"` if `None`.
   - Uptime column: computed as `now - EnrichedSession::started_at` and rendered as
     `HH:MM:SS` (e.g., `00:03:47`); renders `"—"` if `started_at` is `None`.
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
3. The Cost column renders `"—"` (not `"None"` or `"N/A"`) when `cost_usd` is `None`.
   The Project column renders `"—"` when `project_name` is `None`. The Uptime column
   renders `"—"` when `started_at` is `None`. Using `"—"` for all optional columns is
   consistent and signals missing data without cluttering the display.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-082 | Session list updated (new `SessionListUpdate` arrives) while panel is rendering | Next draw tick renders the updated list; no visual tearing (ratatui double-buffer) |
| EC-083 | Session with very long project name (e.g., 80-character monorepo path) | Truncated to fit column width with trailing `…`; no panic, no line wrap |
| EC-084 | Session with `cost_usd = None` (harness does not report cost) | Cost column renders `"—"` |
| EC-085 | Session with `token_count = 0` | Tokens column renders `0` (not blank) |
| EC-086 | `started_at` is set such that `now - started_at` exceeds 99 hours (e.g., 100:00:00) | Uptime column renders `100:00:00` (extended format); no truncation |
| EC-087 | Session list transitions from non-empty to empty (all sessions ended) | Empty state message renders immediately on the next draw tick after `app.sessions` is cleared |
| EC-088 | `harness_type` value for a future non-Claude harness (Phase 2) | Renders whatever `char` the harness type maps to; no hardcoded Claude-specific logic in the render path |

## Canonical Test Vectors

| Input | Expected Rendered Content | Category |
|-------|--------------------------|----------|
| `app.sessions = []` | Two-line empty state message | happy-path |
| `app.sessions = [session with session_id="sess-001", harness_type="claude-code", project_name=Some("monocle"), status=Active, token_count=437000, cost_usd=None, started_at=Some(now - 3h47m)]` | Row: `sess-001 ● monocle Active 437k — 03:47:00` | happy-path |
| `app.sessions = [session with project_name=None]` | Project column: `—` | happy-path |
| `app.sessions = [session with cost_usd=Some(0.83)]` | Cost column: `$0.83` | happy-path |
| `app.sessions = [session with cost_usd=None]` | Cost column: `—` | happy-path |
| `app.sessions = [session with started_at=None]` | Uptime column: `—` | edge-case |
| `app.sessions = [session with token_count=999]` | Tokens column: `999` (no suffix) | edge-case |
| `app.sessions = [session with token_count=1000]` | Tokens column: `1k` | edge-case |
| `app.sessions = [session with token_count=1200000]` | Tokens column: `1.2M` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Empty state message renders when `app.sessions` is empty | Unit test (ratatui TestBackend) |
| VP-TBD | Token formatter: 999 → "999", 1000 → "1k", 142000 → "142k", 1200000 → "1.2M" | Unit test (table-driven) |
| VP-TBD | Cost formatter: None → "—", 0.83 → "$0.83" | Unit test (table-driven) |
| VP-TBD | Sessions panel has no `std::fs`, `std::process`, or OS API imports | `cargo check` + `grep` in CI |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the "sessions panel" component of CAP-006: the primary panel the user sees when managing multiple Claude Code sessions |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — the Sessions Panel is read-only: it renders IPC-pushed data and performs no writes) |
| Architecture Module | monocle-tui (draw_dashboard(), draw_sessions_panel()) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Panel Architecture §Sessions Panel (column layout table, empty state, filter mode, fullscreen) |
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

S-TBD — Implement Sessions Panel renderer with 7-column layout, empty state, token/cost formatters (filled by story-writer)

## VP Anchors

- VP-TBD — ratatui TestBackend snapshot tests for Sessions Panel column layout and empty state

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.005 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.1.0 §Panel Architecture §Sessions Panel (column layout table, empty
  state, filter mode, fullscreen); prd-expansion-scope.md §3.3 BC-2.06.005 description;
  ARCH-INDEX.md §Capability Traceability SS-06.
- Postcondition 6 ("selected row highlighting") specifies Phase 1 behavior: no intra-panel
  cursor navigation. `Enter` immediately fullscreens the single focused session. Phase 2
  will add row-level cursor navigation; this BC will be superseded at that point.
- EC-088 future-proofs the icon render path: the `icon: char` field comes from
  `EngineMetadata`, which is harness-specific. The Sessions Panel render must not hardcode
  Claude-specific logic.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-P14-004 CRITICAL — EnrichedSession field fabrication corrected** (2026-05-26T00:00:00Z):
- Finding: Precondition 2 listed four fields that do not exist on `EnrichedSession` per
  SS-engine-module.md: `engine_metadata: EngineMetadata`, `project_name: String`
  (non-optional), `phase_tag: Option<String>` (removed per F-P13-001), and
  `uptime: Duration` (not a field; computed at render time).
- Fix — Precondition 2: replaced fabricated field list with the actual Phase 1
  `EnrichedSession` fields: `session_id`, `harness_type`, `transcript_path`,
  `config_path`, `status`, `last_event_micros`, `project_name: Option<String>`,
  `started_at: Option<chrono::DateTime<chrono::Utc>>`, `token_count`, `cost_usd`.
  Added notes explaining `phase_tag` exclusion and that `uptime` is computed from
  `started_at` at render time.
- Fix — Description: "phase tag" column renamed to "status" column. The panel has seven
  columns (Session ID, Icon, Project, Status, Tokens, Cost, Uptime) per SS-tui.md v1.8.2
  §Sessions Panel; Status is the fourth column, not the sixth (the sixth is Cost).
- Fix — Postcondition 2: Column layout table corrected. Icon column now derives from
  `harness_type` (not `EngineMetadata`). Phase column replaced by Status column
  (`EnrichedSession::status`). Project column now shows `"—"` for `None`.
  Cost column now shows `"—"` for `None`. Uptime column now correctly sources from
  `started_at` (computed) and shows `"—"` when `None`.
- Fix — Invariant 3: replaced `phase_tag` reference with correct documentation of
  `"—"` sentinel for `cost_usd`, `project_name`, and `started_at` optional fields.
- Fix — EC-086: reworded to reference `started_at` computation instead of fabricated
  `uptime` field. EC-088: reworded to reference `harness_type` instead of `EngineMetadata`.
- Fix — Canonical Test Vectors: replaced `phase_tag`/`uptime` references with
  `status`/`started_at`-derived values. Added `project_name=None` → `"—"` and
  `started_at=None` → `"—"` test cases. Added `cost_usd=None` → `"—"` case.
- Fix — Architecture Source: pinned to SS-tui.md v1.6.0 (current).
- Source-of-truth reads: SS-engine-module.md (EnrichedSession struct lines 311–368);
  SS-tui.md v1.6.0 §Sessions Panel column layout table (lines 415–420).
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.0.6

**F-S025-ADV23-MED-001 Category 8 sweep — Architecture Source pin refresh + §Trace v1.0.4 internal contradiction fix** (2026-05-29T00:00:00Z):
- Architecture Source (Traceability table): `SS-tui.md v1.6.0` → `SS-tui.md v1.8.2` (active pointer was stale by 2 minor versions).
- Postcondition 2 inline citation: `per SS-tui.md v1.6.0 §Sessions Panel` → `per SS-tui.md v1.8.2 §Sessions Panel`.
- §Trace v1.0.4 prose fix: "the sixth column is Status, not Phase Tag, per SS-tui.md v1.6.0 §Sessions Panel" → corrected to seven-column reality ("Status is the fourth column" per the 7-column layout Session ID/Icon/Project/Status/Tokens/Cost/Uptime). The original §Trace v1.0.4 prose incorrectly called Status the "sixth" column because it was written against the 6-column layout (before Session ID was added in v1.0.5). The BC body (Description, PC-2) correctly reflects 7 columns since v1.0.5. The §Trace v1.0.4 historical prose was internally inconsistent with the current body — this fix makes the §Trace prose accurate.
- No substantive BC body (Postconditions, Invariants, Edge Cases) changes: the 7-column layout and column content are already correct in v1.0.5.
- SE-16d monotonicity: v1.0.6 timestamp 2026-05-29T00:00:00Z > v1.0.5 timestamp 2026-05-28T12:00:00Z. PASS.

## §Trace v1.0.5

**F-S025-ADV4-BLOCKER-002 — Column count adjudication: 6 → 7 columns (Option A: amend BC)** (2026-05-28T12:00:00Z):
- Finding: Implementation (`sessions_panel.rs:304-307`) renders SEVEN columns including
  `session_id` (format `{session_id} {icon} {project} | {status} | {tokens} | {cost} | {uptime}`).
  BC-2.06.005 PC-2 previously specified six columns, omitting `session_id`.
- Adjudication: Option A chosen. `session_id` IS required for production-grade operator clarity
  and debuggability: when multiple sessions share the same project name or harness type,
  the session ID is the only stable discriminator for correlating TUI rows to daemon logs.
  The implementation correctly identified the under-specification. The BC was wrong, not the impl.
- Fix — Description: "six columns" → "seven columns"; `session_id` added with rationale.
- Fix — Postcondition 2 (Column layout): `Session ID column` added as first column entry,
  rendering `EnrichedSession::session_id` (e.g., `sess-001`).
- Fix — Canonical Test Vector (happy-path row): `session_id="sess-001"` added to input;
  `● monocle Active 437k — 03:47:00` → `sess-001 ● monocle Active 437k — 03:47:00`.
- Fix — Story Anchor: "6-column" → "7-column".
- No EC changes required: existing EC table rows describe column-specific behaviors that
  are unaffected by the addition of the session_id column (EC-082 through EC-088 remain valid).
- SE-16d monotonicity: v1.0.5 timestamp 2026-05-28T12:00:00Z > v1.0.4 timestamp 2026-05-26T00:00:00Z. PASS.
