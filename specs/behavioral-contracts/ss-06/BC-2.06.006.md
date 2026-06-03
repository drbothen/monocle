---
document_type: behavioral-contract
level: L3
version: "1.1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "64a61b4"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.1.0
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.006: Sessions Panel: / Filter with Nucleo Fuzzy Match

## Description

Pressing `/` in the Sessions Panel (when `AppMode` is `Dashboard { focused: Sessions }`)
transitions to `AppMode::Filtering { panel: PanelId::Sessions, query: String::new(), prior: Sessions }`.
Subsequent keystrokes append characters to `query`, which is passed to the nucleo fuzzy
matcher on every keystroke. Only `EnrichedSession` entries whose `project_name` or harness
`display_name` match the query are shown. Matched characters are highlighted via ratatui
`Span` styling. `Esc` clears the filter and transitions back to `Dashboard { focused: Sessions }`.

## Preconditions

1. `AppMode` is `Dashboard { focused: FocusSnapshot::Sessions }` when `/` is pressed.
2. `nucleo 0.5` is listed as a dependency in `monocle-tui/Cargo.toml` per SS-deps-pin-manifest.md.
3. `app.matcher` is a `nucleo::Matcher` instance reused across filter inputs (not
   re-created per keystroke).
4. `app.sessions` contains the current session list from the last `SessionListUpdate` IPC
   message.

## Postconditions

1. **`/` activates Filtering mode:** Pressing `/` in `Dashboard { focused: Sessions }`
   transitions to `AppMode::Filtering { panel: PanelId::Sessions, query: "".to_string(), prior: FocusSnapshot::Sessions }`.
   The filter input is shown as an input widget at the bottom of the Sessions Panel area.
2. **Typed characters append to `query`:** Each `Action::FilterType(char)` (dispatched
   from `SearchPrompt` level per BC-2.06.003) appends `char` to `Filtering::query`. The
   updated query is passed to `app.matcher` on each character. The panel re-renders within
   one draw tick (≤16ms).
3. **Nucleo match criteria:** Sessions are included in the filtered view if the nucleo
   fuzzy matcher returns a match for either `EnrichedSession::project_name` OR
   `EnrichedSession::display_name` against the current `query`. The match is case-insensitive.
   `EnrichedSession::display_name` is the TUI-readable copy of `EngineMetadata::display_name`
   stored on `EnrichedSession` at enrichment time by the daemon (S-028 ADR: display_name
   sourcing decision — the TUI reads `session.display_name` directly from IPC wire data;
   it MUST NOT call `EngineModule::metadata()` which is a daemon-only call path).
4. **Match highlight:** For each displayed session row, matched character positions are
   rendered using ratatui `Span::styled` with a distinct highlight style (e.g., bold or
   foreground color different from the default). Unmatched characters use the default style.
5. **`Esc` clears filter and returns to Dashboard:** `Action::Escape` in `Filtering` mode
   transitions to `AppMode::Dashboard { focused: FocusSnapshot::Sessions }` per the
   `transition()` function (BC-2.06.001). The `query` is discarded. All sessions are
   visible again on the next draw tick.
6. **`Action::FilterClear` behaves identically to `Escape`:** Both actions restore
   `Dashboard` with the prior `FocusSnapshot`. `FilterClear` is a separate action to allow
   binding it to a different key (e.g., `Ctrl-U`) in `Builtin` without conflicting with
   `Escape`.
7. **Sessions Panel continues receiving IPC updates during Filtering:** If a new
   `SessionListUpdate` arrives while in `Filtering` mode, `app.sessions` is updated. The
   matcher is re-run against the updated session list on the next draw tick. Newly arrived
   sessions that match the query appear; sessions that ended are removed.
8. **Zero matches renders empty state variant:** If the nucleo matcher returns no matches
   for the current query, the filtered Sessions Panel renders: "No sessions match filter".
   This is distinct from the base empty state ("No sessions detected").

## Invariants

1. The nucleo `Matcher` instance is shared across the lifetime of the filter session. It is
   not re-created per keystroke. This reuse is required for performance at the P0 60fps
   target.
2. The filter operates on `app.sessions` only — it does not read from the JSONL ring or
   any disk source.
3. Backspace reduces the query by one character (removes the last appended char). The
   matcher is re-run after each backspace.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-089 | User types query that matches no sessions | Panel renders "No sessions match filter" (not the base empty state) |
| EC-090 | User types query, a new session arrives that matches the query | The new session appears in the filtered list on the next draw tick |
| EC-091 | Backspace pressed when `query` is empty | Query remains empty (`"".to_string()`); no panic; matcher re-run produces the same full-list result |
| EC-092 | Unicode character typed (e.g., emoji or CJK character) | Character is appended to query via `Action::FilterType(char)`; nucleo receives it; result depends on nucleo's Unicode handling (not overridden by monocle) |
| EC-093 | Filter active when `Ctrl-\` hide/show cycle occurs | Filter state (Filtering AppMode, current query) is NOT preserved across hide/show (transient TUI state; daemon does not store filter query); next popup starts from Dashboard |
| EC-094 | All sessions end while filter is active | `app.sessions` is cleared by `SessionListUpdate`; matcher returns no matches; panel renders "No sessions match filter" (0 sessions matched, not the base empty state) |

## Canonical Test Vectors

| Query | Sessions in `app.sessions` | Expected Shown Sessions | Category |
|-------|-----------------------------|------------------------|----------|
| `"mono"` | `[{project: "monocle"}, {project: "another-project"}]` | `[{project: "monocle"}]` | happy-path |
| `""` (empty) | `[{project: "monocle"}, {project: "another-project"}]` | Both sessions (empty query matches all) | happy-path |
| `"xyz"` | `[{project: "monocle"}]` | `[]` (no match → "No sessions match filter" state) | edge-case |
| `"MONO"` | `[{project: "monocle"}]` | `[{project: "monocle"}]` (case-insensitive match) | edge-case |
| `"cla"` | `[{project: "monocle", display_name: "Claude Code"}]` | `[{project: "monocle"}]` (matched via display_name) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `/` keypress transitions AppMode to Filtering with empty query | Unit test (AppMode state machine) |
| VP-TBD | `Esc` in Filtering transitions back to Dashboard with correct prior FocusSnapshot | Unit test (AppMode state machine) |
| VP-TBD | Nucleo matcher runs on each typed character; panel re-renders within one tick | Integration test (ratatui TestBackend) |
| VP-TBD | "No sessions match filter" renders when query matches nothing | Unit test (ratatui TestBackend) |
| VP-TBD | Filter does not read from disk or process state | `cargo check` + `grep` in CI |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the filter feature of the "sessions panel" component of CAP-006, enabling telescope-style session discovery for users managing many concurrent sessions |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — filter operates purely on in-memory `app.sessions` with no disk access) |
| Architecture Module | monocle-tui (draw_filter_overlay(), nucleo Matcher usage, SearchPrompt keybinding table) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Panel Architecture §Sessions Panel (Filter mode paragraph); §Dependency Graph (nucleo 0.5) |
| Cross-Ref | BC-2.06.001 (Filtering AppMode variant and transition function), BC-2.06.003 (SearchPrompt dispatch level captures printable keys during Filtering), BC-2.06.002 (Esc restores FocusSnapshot) |
| Test File | `monocle-tui/tests/sessions_filter.rs` |
| Test Name | `test_BC_2_06_006_sessions_filter_nucleo_fuzzy_match` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: `Filtering` AppMode variant is defined by BC-2.06.001; `FilterType` and `FilterClear` actions are in the Action enum
- [BC-2.06.002] — composes with: `Esc` from Filtering restores `prior: FocusSnapshot` per BC-2.06.002
- [BC-2.06.003] — depends on: `SearchPrompt` level in the 5-level dispatcher captures printable keys as `FilterType(char)` during Filtering mode

## Architecture Anchors

- `architecture/SS-tui.md#panel-architecture` — Filter mode paragraph in Sessions Panel section
- `architecture/SS-tui.md#rendering-architecture` — `draw_filter_overlay()` call in AppMode::Filtering match arm

## Story Anchor

S-TBD — Implement Sessions Panel filter mode with nucleo 0.5 matcher; match highlights via ratatui Span; SearchPrompt dispatch level (filled by story-writer)

## VP Anchors

- VP-TBD — Integration tests: nucleo fuzzy match against real EnrichedSession data; AppMode Filtering round-trip

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.006 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.1.0 §Panel Architecture §Sessions Panel (filter mode paragraph);
  §Rendering Architecture (draw_filter_overlay call); §Dependency Graph (nucleo 0.5);
  prd-expansion-scope.md §3.3 BC-2.06.006 description; ARCH-INDEX.md §Capability Traceability SS-06.
- Postcondition 8 ("zero matches renders empty state variant") distinguishes the filter-no-match
  state from the base empty state. Both are two-line messages but with different text. The
  test-writer must produce separate test vectors for each.
- EC-093 confirms that filter state is NOT preserved across Ctrl-\ hide/show — intentional
  design per the lazygit philosophy. Transient TUI state (filter query, scroll position) is
  ephemeral. Only daemon-owned state (session list, queued prompts) survives reconnection.


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

## §Trace v1.1.0

**S-028 ADR — display_name sourcing decision: field source corrected to `EnrichedSession::display_name`** (2026-06-01):
- PC-3 previously referenced `EngineMetadata::display_name` as the filter match source. This
  was incorrect: `EngineMetadata` is a daemon-side type returned by `EngineModule::metadata()`.
  The TUI process cannot call `EngineModule::metadata()` — it has no access to the
  `EngineModule` trait object. The correct source is `EnrichedSession::display_name`, a new
  field added to `EnrichedSession` in `monocle-core` in this same S-028 ADR burst.
- **What changed:** At daemon enrichment time, `ClaudeCodeModule::enrich()` calls
  `metadata()?.display_name` and stores the result as `EnrichedSession::display_name`. This
  field is serialized into `SessionListUpdate` / `InitialState.sessions` IPC messages and is
  therefore available to the TUI without any trait object access. The TUI filter reads
  `session.display_name` directly.
- **PC-3 updated:** `EngineMetadata::display_name` → `EnrichedSession::display_name` with
  explanation of the sourcing chain and MUST NOT constraint on `EngineModule::metadata()`.
- SE-16d monotonicity: v1.1.0 timestamp 2026-06-01 > v1.0.4 timestamp 2026-05-29. PASS.

## §Trace v1.0.4

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): this BC has no `Overlay { stack }` references; it covers the Filtering mode only.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC references the Sessions Panel filter mode paragraph only, not the column layout table.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
- SE-16d monotonicity: v1.0.4 timestamp 2026-05-29T00:00:00Z > v1.0.3. PASS.
