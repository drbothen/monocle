---
document_type: story
level: L4
story_id: S-028
epic_id: EPIC-06
version: "1.0"
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
behavioral_contracts: [BC-2.06.006, BC-2.06.018]
verification_properties: []
estimated_days: 2
inputs:
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.006.md, version: "1.0.0"}
  - {path: .factory/specs/behavioral-contracts/ss-06/BC-2.06.018.md, version: "1.0.0"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "1.1.17"}
input-hash: "[pending]"
traces_to: "Implements BC-2.06.006 (sessions fuzzy filter), BC-2.06.018 (event ribbon panel)"
---

# S-028: Sessions Filter + Event Ribbon

## Narrative

As a daemon operator with many active sessions, I want to type a filter query to fuzzy-match
session IDs and harness types in the Sessions panel, and view the chronological event ribbon
for a selected session, so that I can quickly locate the session I care about and see its
recent activity.

## Acceptance Criteria

### AC-001 (traces to BC-2.06.006 postcondition PC-1 — filter entry)
From `Dashboard { focused: FocusSnapshot { panel: PanelId::Sessions, .. } }`, pressing `/`
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
`<timestamp> | <event_type> | <summary>`. Events are sourced from the ring buffer
(S-008/S-020) via `ServerToClient::SessionEvents { session_id, events }` IPC messages.

### AC-007 (traces to BC-2.06.018 postcondition PC-2 — event ribbon keyboard navigation)
In `Dashboard { focused: FocusSnapshot { panel: PanelId::EventRibbon, .. } }` mode:
- `j` / `↓` scrolls down one event
- `k` / `↑` scrolls up one event
- `G` jumps to the newest event (bottom)
- `g` (twice: `gg`) jumps to the oldest event (top)
- `Enter` enters `Fullscreen { panel: PanelId::EventRibbon, prior: current_focus }`
These bindings are dispatched via `resolve_binding()` using the `Global` layer.

### AC-008 (traces to BC-2.06.018 postcondition PC-3 — event ribbon auto-scroll)
When a new event arrives for the selected session (via `ServerToClient::SessionEvents`),
the event ribbon auto-scrolls to the bottom UNLESS the user has manually scrolled up
(i.e., the current scroll position is not at the bottom). This is a standard "follow
tail unless pinned" pattern — track whether the user has scrolled away from bottom.

### AC-009 (traces to BC-2.06.018 invariant INV-1 — event ribbon panel scope)
The event ribbon shows events ONLY for the session currently selected in the Sessions
panel. When session selection changes (via `j`/`k` navigation or filter commit), the
event ribbon clears its event list and requests a fresh `SessionEvents` load for the
new `session_id`.

## Token Budget Estimate

| Component | Tokens |
|-----------|--------|
| This story spec | ~1,800 |
| BC-2.06.006.md | ~900 |
| BC-2.06.018.md | ~900 |
| S-025 (TUI skeleton, Sessions panel, layout) | ~700 |
| S-021 (UDS IPC types, SessionEvents) | ~500 |
| nucleo 0.5 API | ~400 |
| Test files | ~900 |
| **Total estimate** | **~6,100** |

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
- [ ] Implement `ServerToClient::SessionEvents` IPC handler: update `App.event_ribbon_events` for selected session
- [ ] Implement auto-scroll logic: auto-follow bottom unless user has scrolled up (bool flag `pinned_top`)
- [ ] Implement event ribbon keyboard navigation: `j`/`k`/`G`/`gg`/`Enter` bindings in Global layer
- [ ] Implement session-change event ribbon reset: on selection change, clear events, request fresh load
- [ ] Unit tests `monocle-tui/tests/filter_sessions.rs` — filter entry/exit, nucleo scoring, empty query
- [ ] Unit tests `monocle-tui/tests/event_ribbon.rs` — auto-scroll, pin/unpin, session-change reset

## Previous Story Intelligence

S-025 (TUI skeleton): `App.sessions: Vec<SessionState>` is the unfiltered session list.
The Sessions panel uses ratatui `ListState` for scroll/selection tracking. The `Filtering`
mode query field is in `AppMode::Filtering { query, .. }` — extract it from `App.mode`
in the render function.

S-021 (UDS IPC types): `ServerToClient::SessionEvents { session_id: String, events: Vec<RingEvent> }`
is the IPC message for event ribbon data. Confirm exact field names in S-021 before
implementing.

nucleo 0.5 API: `Matcher::new(Config::DEFAULT)` creates a matcher. `matcher.fuzzy_match(haystack, needle, false)`
scores a single string. Pattern can be pre-compiled via `Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart)`.
Use `Pattern::score(haystack, &mut matcher)` for repeated scoring.

## Architecture Compliance Rules

From `architecture/SS-tui-core.md` and `architecture/SS-conventions-anti-patterns.md`:
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
| monocle-ipc | workspace path | `ServerToClient::SessionEvents` |

## File Structure Requirements

Files to create:
- `monocle-tui/src/ui/event_ribbon.rs` — EventRibbon panel widget
- `monocle-tui/tests/filter_sessions.rs` — session filter unit tests
- `monocle-tui/tests/event_ribbon.rs` — event ribbon unit tests

Files to modify:
- `monocle-tui/Cargo.toml` — add `nucleo 0.5`
- `monocle-tui/src/app.rs` — add `matcher: nucleo::Matcher` field; add `SessionEvents` handler;
  add event ribbon state (`event_ribbon_events`, `pinned_top`)
- `monocle-tui/src/ui/sessions_panel.rs` (from S-025) — add filter input box rendering; add nucleo scoring
- `monocle-tui/src/ui/mod.rs` — declare `event_ribbon` module

## Downstream Consumer Contract

No new public API produced by this story. The filter and event ribbon features are
internal to `monocle-tui`. No stories block on this one; it is a leaf story in the
dependency graph.
