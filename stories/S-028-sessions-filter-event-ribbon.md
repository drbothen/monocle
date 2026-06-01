---
document_type: story
level: L4
story_id: S-028
epic_id: EPIC-06
version: "1.5"
status: not_started
producer: vsdd-factory:story-writer
timestamp: 2026-05-27T00:00:00Z
phase: 2
points: 5
wave: 7
tdd_mode: strict
priority: P1
depends_on: [S-025, S-021]
blocks: []
target_module: monocle-tui
subsystems: [SS-06]
behavioral_contracts: [BC-2.05.002, BC-2.05.004, BC-2.06.006, BC-2.06.018]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md, version: "1.0.6"}
  - {path: .factory/specs/behavioral-contracts/ss-05/BC-2.05.004.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.006.md, version: "1.1.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.018.md, version: "1.0.5"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "b95151a"
traces_to: "Implements BC-2.05.002 (InitialState ring_tail delivery), BC-2.05.004 (HookEventReceived streaming), BC-2.06.006 (sessions fuzzy filter), BC-2.06.018 (event ribbon panel)"
---

# S-028: Sessions Filter + Event Ribbon

## Narrative

As a daemon operator with many active sessions, I want to type a filter query to fuzzy-match
session IDs and harness types in the Sessions panel, and view the chronological event ribbon
for a selected session, so that I can quickly locate the session I care about and see its
recent activity.

## Acceptance Criteria

### AC-001 (traces to BC-2.06.006 postcondition PC-1 — filter entry)
From `Dashboard { focused: FocusSnapshot::Sessions }`, pressing `/`
or `f` dispatches `Action::StartFilter { panel: PanelId::Sessions }` via `resolve_binding()`,
transitioning to `Filtering { panel: PanelId::Sessions, query: String::new(), prior: focused }`.
A search input box is rendered at the top of the Sessions panel with cursor.

### AC-002 (traces to BC-2.06.006 postcondition PC-2 — fuzzy match via nucleo)
While in `Filtering` mode with `panel: PanelId::Sessions`, each typed character updates
`query` and re-scores all sessions against `query` using `nucleo::Matcher`. Sessions are
ranked by match score (highest first). Non-matching sessions are hidden. The `nucleo::Matcher`
instance is shared (not recreated per keystroke) — it is held in `App` as a field.

### AC-003 (traces to BC-2.06.006 postcondition PC-3 — filter exit)
Pressing `Enter` in `Filtering` mode dispatches `Action::CommitFilter`, returning to
`Dashboard { focused: prior }` with the filtered selection retained as the active session.
Pressing `Esc` dispatches `Action::CancelFilter`, returning to `Dashboard { focused: prior }`
with the full unfiltered session list restored.

### AC-004 (traces to BC-2.06.006 postcondition PC-4 — empty query shows all)
When `query` is empty (`String::new()`), all sessions are shown in their default order
(insertion order from `ServerToClient::SessionState` messages). No nucleo scoring is
applied for empty queries.

### AC-005 (traces to BC-2.06.006 invariant INV-1 — shared Matcher)
`nucleo::Matcher` is instantiated once and stored in `App.matcher: nucleo::Matcher`.
It is NOT recreated per keystroke or per filter entry. Recreating the matcher per
keystroke would reset internal caches and degrade performance.

### AC-006 (traces to BC-2.06.018 postcondition PC-1 — event ribbon panel)
The EventRibbon panel renders a scrollable chronological list of events for the
currently selected session (highlighted row in Sessions panel). Each event row shows:
`<timestamp> | <event_type> | <summary>`. Events are sourced from two paths per
BC-2.05.004 invariant 3 (no filtering at the IPC layer; the TUI filters for display):
- `InitialState.ring_tail` (received on connect, per BC-2.05.002) — backfills historical events
- `HookEventReceived` messages (streaming, per BC-2.05.004) — delivers live events as they arrive
The TUI holds a `VecDeque<HookEventRow>` in `App.event_ribbon_events` and filters it
client-side by `session_id` to display only events for the selected session.

### AC-007 (traces to BC-2.06.018 postcondition PC-2 — event ribbon keyboard navigation)
In `Dashboard { focused: FocusSnapshot::EventRibbon }` mode:
- `j` / `↓` scrolls down one event
- `k` / `↑` scrolls up one event
- `G` jumps to the newest event (bottom)
- `g` (twice: `gg`) jumps to the oldest event (top)
- `Enter` enters `Fullscreen { panel: PanelId::EventRibbon, prior: current_focus }`
These bindings are dispatched via `resolve_binding()` using the `Global` layer.

### AC-008 (traces to BC-2.06.018 postcondition PC-3 — event ribbon auto-scroll)
When a new `HookEventReceived` message arrives (per BC-2.05.004) and the incoming event
matches the selected session (client-side filter by `session_id`), the event ribbon
auto-scrolls to the bottom UNLESS the user has manually scrolled up (i.e., the current
scroll position is not at the bottom). This is a standard "follow tail unless pinned"
pattern — track whether the user has scrolled away from bottom via a `pinned_top: bool` flag.

### AC-009 (traces to BC-2.06.018 invariant INV-1 / postcondition PC-2 — event ribbon panel scope)
The event ribbon shows events ONLY for the session currently selected in the Sessions
panel. When session selection changes (via `j`/`k` navigation or filter commit), the
event ribbon re-filters `App.event_ribbon_events` (the full `VecDeque<HookEventRow>`)
by the new `session_id` and resets scroll to the auto-follow position: the newest event
at row 0 (the top of the newest-first list, per BC-2.06.018 PC-2), clearing any
`pinned_top` state (i.e., `pinned_top = false`). No new IPC request is issued on
selection change — all events across all sessions are already held client-side from
`InitialState.ring_tail` (BC-2.05.002) and ongoing `HookEventReceived` messages
(BC-2.05.004); filtering is purely client-side per BC-2.05.004 invariant 3.

### AC-010 (traces to BC-2.06.006 postcondition PC-2 / BC-2.06.018 postcondition PC-5 — integration: Event Ribbon and Sessions filter wired into render_frame and dispatch_key_event)
The Event Ribbon widget AND the Sessions filter input MUST be rendered by the production
`render_frame` / `draw()` path — not only callable in isolation as unit-tested widgets.
Acceptance:
1. A `TestBackend`-driven `render_frame` test asserts that (a) the 40% right-side area of
   the terminal buffer contains event ribbon row content (timestamp | hook-type | session |
   latency columns) when `App.event_ribbon_events` is non-empty, and (b) when
   `AppMode::Filtering { panel: PanelId::Sessions, .. }` is active, the buffer contains
   the filter input box and scored/filtered session rows (including the "No sessions match
   filter" sentinel for a zero-match query).
2. A `dispatch_key_event` test asserts that pressing `j` / `k` / `↓` / `↑` / `G` / `gg`
   in `Dashboard { focused: FocusSnapshot::EventRibbon }` dispatches
   `Action::ScrollDown` / `Action::ScrollUp` and that the ribbon's scroll offset changes
   accordingly (i.e., the action reaches the ribbon state handler, not just a key table).

A widget that is fully implemented and unit-tested in isolation but never wired into
`render_frame` / `dispatch_key_event` is dead code and would cause the event ribbon and
filter input to be absent or non-interactive at runtime. This AC closes that process gap.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,900 |
| BC-2.05.002.md | ~700 |
| BC-2.05.004.md | ~700 |
| BC-2.06.006.md | ~900 |
| BC-2.06.018.md | ~900 |
| S-025 (TUI skeleton, Sessions panel, layout) | ~700 |
| S-021 (UDS IPC types, HookEventReceived, InitialState) | ~500 |
| nucleo 0.5 API | ~400 |
| Test files | ~900 |
| **Total estimate** | **~7,600** |

## Tasks

- [ ] Add `nucleo 0.5` to `monocle-tui/Cargo.toml`
- [ ] Add `App.matcher: nucleo::Matcher` field in `monocle-tui/src/app.rs` — initialized once at startup
- [ ] Implement filter entry: `/` and `f` key dispatch `Action::StartFilter { panel: Sessions }`
      via `resolve_binding()` with `Global` layer binding
- [ ] Implement filter rendering: search input box at top of Sessions panel with cursor, typed query
- [ ] Implement nucleo scoring in Sessions panel render: when `Filtering` mode, score sessions
      against `query`, sort by score descending, hide score-0 sessions; when query empty, show all
- [ ] Implement `Action::CommitFilter` handler: retain selected filtered session, transition to Dashboard
- [ ] Implement `Action::CancelFilter` handler: restore full session list, transition to Dashboard
- [ ] Create `monocle-tui/src/ui/event_ribbon.rs` — EventRibbon panel widget with scroll state
- [ ] Add `App.event_ribbon_events: VecDeque<HookEventRow>` — holds ALL events from all sessions;
      populated from `InitialState.ring_tail` on connect (BC-2.05.002) and from each `HookEventReceived`
      message (BC-2.05.004); client-side filtered by `session_id` in the render function
- [ ] Implement `HookEventReceived` IPC handler: append new event to `App.event_ribbon_events`
- [ ] Implement `InitialState` handler: pre-populate `App.event_ribbon_events` from `ring_tail` on connect
- [ ] Implement auto-scroll logic: auto-follow bottom unless user has scrolled up (bool flag `pinned_top`)
- [ ] Implement event ribbon keyboard navigation: `j`/`k`/`G`/`gg`/`Enter` bindings in Global layer
- [ ] Implement session-change event ribbon reset: on selection change, re-filter `App.event_ribbon_events`
      by new `session_id` (client-side only — no IPC request issued) and reset scroll to auto-follow
      position (newest at row 0, `pinned_top = false`) per AC-009
- [ ] Unit tests `monocle-tui/tests/filter_sessions.rs` — filter entry/exit, nucleo scoring, empty query
- [ ] Unit tests `monocle-tui/tests/event_ribbon.rs` — auto-scroll, pin/unpin, session-change reset
- [ ] Integration render test `monocle-tui/tests/render_frame_integration_s028.rs` — drive `App::render_frame`
      via `TestBackend`; assert event ribbon content in 40% area and filtered sessions in Filtering mode (AC-010)
- [ ] Integration dispatch test — drive `dispatch_key_event` for `j`/`k`/`G`/`gg` in EventRibbon focus;
      assert `Action::ScrollDown`/`Action::ScrollUp` is dispatched and ribbon scroll offset changes (AC-010)

## Previous Story Intelligence

<!-- structural-claim-historical: Vec<SessionState> → Vec<EnrichedSession> drift deferred per ADR-0008 §Cross-story propagation (BC-5.39.002 PC2); story-writer wave-gate sweep obligation -->
S-025 (TUI skeleton): `App.sessions: Vec<SessionState>` is the unfiltered session list.
The Sessions panel uses ratatui `ListState` for scroll/selection tracking. The `Filtering`
mode query field is in `AppMode::Filtering { query, .. }` — extract it from `App.mode`
in the render function.

S-021 (UDS IPC types): `ServerToClient::HookEventReceived { session_id: String, event: RingEvent }`
delivers individual events as they arrive (BC-2.05.004). There is NO `ServerToClient::SessionEvents`
variant — event ribbon data is sourced from `InitialState.ring_tail` (BC-2.05.002) on connect and
from ongoing `HookEventReceived` messages (BC-2.05.004) while running. Filtering by session_id
is done client-side in the TUI per BC-2.05.004 invariant 3; no session-scoped IPC request exists.
Confirm exact field names for `HookEventReceived` and `InitialState.ring_tail` in S-021 before
implementing.

nucleo 0.5 API: `Matcher::new(Config::DEFAULT)` creates a matcher. `matcher.fuzzy_match(haystack, needle, false)`
scores a single string. Pattern can be pre-compiled via `Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart)`.
Use `Pattern::score(haystack, &mut matcher)` for repeated scoring.

## Architecture Compliance Rules

From `architecture/SS-tui.md` and `architecture/SS-conventions-anti-patterns.md`:
- `nucleo::Matcher` is shared — held in `App` as a field, NOT created per keystroke
- Fuzzy filtering uses `nucleo 0.5` exclusively — no custom string matching
- `Filtering` mode's `query` field in `AppMode::Filtering` is the authoritative query source
  during filter mode; do not maintain a duplicate query field in `App`
- Event ribbon `"gg"` pattern: detect two consecutive `g` keystrokes; implement as a
  pending-key state (`Option<KeyCode>`) in the event handler, not in `transition()`
- `ratatui::StatefulWidget` pattern for event ribbon scrolling; `ListState` for selection

**Forbidden Dependencies:**
- Custom fuzzy matching — use `nucleo 0.5` exclusively
- `nucleo` in `monocle-core` — lives in `monocle-tui` only (purity boundary)

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| nucleo | 0.5 | `Matcher`, `Pattern` for fuzzy session filtering |
| ratatui | workspace pin | `StatefulWidget`, `ListState`, `Paragraph` for filter input |
| monocle-core | workspace path | `AppMode::Filtering`, `Action`, `transition()`, `PanelId` |
| monocle-ipc | workspace path | `ServerToClient::HookEventReceived`, `ServerToClient::InitialState` (ring_tail) |

## File Structure Requirements

Files to create:
- `monocle-tui/src/ui/event_ribbon.rs` — EventRibbon panel widget
- `monocle-tui/tests/filter_sessions.rs` — session filter unit tests
- `monocle-tui/tests/event_ribbon.rs` — event ribbon unit tests
- `monocle-tui/tests/render_frame_integration_s028.rs` — AC-010 integration render test (TestBackend drive of `App::render_frame`; asserts ribbon content and Filtering mode layout)

Files to modify:
- `monocle-tui/Cargo.toml` — add `nucleo 0.5`
- `monocle-tui/src/app.rs` — add `matcher: nucleo::Matcher` field; add `HookEventReceived` handler
  (append to `event_ribbon_events`); add `InitialState` ring_tail pre-population;
  add event ribbon state (`event_ribbon_events: VecDeque<HookEventRow>`, `pinned_top: bool`)
- `monocle-tui/src/ui/sessions_panel.rs` (from S-025) — add filter input box rendering; add nucleo scoring
- `monocle-tui/src/ui/mod.rs` — declare `event_ribbon` module

## Downstream Consumer Contract

No new public API produced by this story. The filter and event ribbon features are
internal to `monocle-tui`. No stories block on this one; it is a leaf story in the
dependency graph.

## §Trace v1.3

**F-S025-ADV22-MED-001 sibling propagation — SS-tui-core.md → SS-tui.md (line 166)** (2026-05-29):
- Architecture Compliance Rules header: `architecture/SS-tui-core.md` → `architecture/SS-tui.md`.
- Systematic EPIC-06 story-writing burst defect; canonical anchor is `SS-tui.md` per BC-2.06.005 §Architecture Source + audit-table.md row 41.
- SE-16d monotonicity: v1.3 timestamp 2026-05-29 >= v1.2 timestamp 2026-05-27. PASS.

## §Trace v1.4

**ADV S-028 Pass-1 — AC-009 reconciled to BC-2.06.018 PC-2 row-0-newest; added integration AC-010; BC version pins refreshed** (2026-06-01):
- AC-009 reconciliation: removed contradictory "resets scroll position to the bottom" phrasing. The ribbon uses newest-first ordering (BC-2.06.018 PC-2): newest event is at row 0 (top). AC-009 now says "resets scroll to the auto-follow position: newest event at row 0 (the top of the newest-first list, per BC-2.06.018 PC-2), clearing `pinned_top = false`." The prior "bottom" wording implied an oldest-first model, which is the opposite of PC-2.
- AC-010 added: integration render + dispatch requirement — `App::render_frame` must wire Event Ribbon widget (40% area) and Sessions filter input (Filtering mode layout); `dispatch_key_event` must handle `j`/`k`/`G`/`gg` scroll in EventRibbon focus via `Action::ScrollDown`/`Action::ScrollUp`. Closes dead-code integration gap (same class as S-027 AC-012).
- BC version pins in `inputs:` frontmatter refreshed to current: BC-2.05.002 1.0.0→1.0.6, BC-2.05.004 1.0.0→1.0.4, BC-2.06.006 1.0.0→1.0.4, BC-2.06.018 1.0.0→1.0.5.
- Tasks: added integration render test task and integration dispatch test task (AC-010).
- File Structure: added `render_frame_integration_s028.rs` (AC-010 integration test).
- SE-16d monotonicity: v1.4 timestamp 2026-06-01 >= v1.3 timestamp 2026-05-29. PASS.
