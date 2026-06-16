---
document_type: behavioral-contract
level: L3
version: "1.0.5"
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
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.007: Sessions Panel: Enter Transitions to Fullscreen

## Description

Pressing `Enter` when `AppMode` is `Dashboard { focused: FocusSnapshot::Sessions }` transitions
to `AppMode::Fullscreen { panel: PanelId::Sessions, prior: FocusSnapshot::Sessions }`. The
fullscreen view occupies 100% of the main area (excluding the always-visible status bar) and
renders the selected session's detail: token history, cost breakdown, hook event count, and
current phase tag. `Action::ExitFullscreen` returns to `Dashboard { focused: FocusSnapshot::Sessions }`;
the physical `Esc` key is bound to `Action::Esc` (builtin binding, identity in Fullscreen — does NOT
exit). The `Esc`→`Action::ExitFullscreen` per-context key binding is the responsibility of the
Sessions Panel fullscreen-view story, not this BC.

## Preconditions

1. `AppMode` is `Dashboard { focused: FocusSnapshot::Sessions }` when `Enter` is pressed.
2. `app.sessions` contains at least one `EnrichedSession`. (If the session list is empty,
   the `Enter` keypress still dispatches `Action::Enter`, but the Fullscreen renderer
   gracefully shows "No session selected".)
3. The `transition()` function (BC-2.06.001) maps
   `(Dashboard { focused: Sessions }, Action::Enter)` →
   `Fullscreen { panel: PanelId::Sessions, prior: FocusSnapshot::Sessions }`.

## Postconditions

1. **AppMode transitions to Fullscreen:** After `Action::Enter` is dispatched, `app.mode`
   is `AppMode::Fullscreen { panel: PanelId::Sessions, prior: FocusSnapshot::Sessions }`.
2. **Fullscreen occupies 100% of main area:** The `draw_fullscreen()` function renders the
   Sessions Panel content across the full main area (all available rows and columns, minus
   the 2-row status bar at the bottom).
3. **Fullscreen content — session detail view:** The fullscreen view renders the following
   for the focused session (Phase 1):
   - Session header: harness icon, project name, session ID (abbreviated).
   - Phase tag line: `Phase: <phase_tag>` if `Some`; omitted if `None`.
   - Token summary: `Tokens used: <formatted count>` (same human format as the panel column).
   - Cost summary: `Estimated cost: $<N.NN>` if cost is available; omitted if `None`.
   - Uptime: `Session uptime: HH:MM:SS`.
   - Hook event count: `Hook events: <N>` (count of events received for this session).
4. **Fullscreen updates live:** While in Fullscreen mode, the TUI continues receiving IPC
   `SessionListUpdate` and `HookEventReceived` messages. The displayed session data updates
   on each draw tick.
5. **Fullscreen exit — `Action::ExitFullscreen` returns to Dashboard:** When
   `transition(AppMode::Fullscreen { panel: Sessions, prior: Sessions }, Action::ExitFullscreen)` is
   called, the returned `AppMode` is `Dashboard { focused: FocusSnapshot::Sessions }` per BC-2.06.002.
   The physical `Esc` key is bound to `Action::Esc` (identity in Fullscreen — does NOT exit);
   `Action::ExitFullscreen` is wired to a key (per-context Fullscreen binding) by the Sessions Panel
   fullscreen view story. Only `Action::ExitFullscreen` exits fullscreen.
6. **Status bar remains visible:** The 2-row status bar (breadcrumb + keybinding hint) is
   always rendered, even in Fullscreen mode. The breadcrumb shows "Dashboard > Sessions > Fullscreen".
7. **Empty-session guard:** If `app.sessions` is empty when `Enter` is pressed (transition
   still occurs because `transition()` is mode-only, not data-dependent), the fullscreen
   renderer shows "No session selected" instead of crashing or panicking.

## Invariants

1. The `transition()` function's `Enter` arm does not inspect `app.sessions`. It
   transitions to `Fullscreen` based solely on `AppMode` and `Action`. The guard for empty
   sessions is in the renderer, not in the transition function.
2. `prior: FocusSnapshot` captured on Fullscreen entry is always `FocusSnapshot::Sessions`
   when entered from `Dashboard { focused: Sessions }`. This is a consequence of BC-2.06.002's
   postcondition — `transition()` captures `focused` as `prior`.
3. Only one Fullscreen panel is active at a time (compile-time mutual exclusion enforced
   by BC-2.06.001).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-095 | `Enter` pressed when `app.sessions` is empty | `AppMode` transitions to `Fullscreen { panel: Sessions, prior: Sessions }`; renderer shows "No session selected"; `Escape` returns to `Dashboard` |
| EC-096 | Session ends (removed from `app.sessions`) while user is in Fullscreen for that session | Renderer shows stale data from the last `SessionListUpdate` until the next update arrives; on update, the session is gone — renderer falls back to "No session selected" |
| EC-097 | `Enter` pressed in `Dashboard { focused: EventRibbon }` | Transitions to `Fullscreen { panel: PanelId::EventRibbon, prior: FocusSnapshot::EventRibbon }` — Phase 1 has a separate fullscreen for EventRibbon; this BC covers Sessions only |
| EC-098 | `Enter` pressed while `AppMode` is `Filtering { .. }` | No `transition()` arm handles `(Filtering, Enter)`; identity transition returns `Filtering` unchanged; `Enter` during filter mode does nothing |
| EC-099 | Session token count exceeds u64 max | Not possible in practice; `u64::MAX` ≈ 1.8 × 10^19 tokens; token formatter handles arbitrarily large values without overflow |

## Canonical Test Vectors

| Input (mode, action) | Expected AppMode | Category |
|----------------------|-----------------|----------|
| `Dashboard { focused: Sessions }`, `Enter` | `Fullscreen { panel: Sessions, prior: Sessions }` | happy-path |
| `Fullscreen { panel: Sessions, prior: Sessions }`, `ExitFullscreen` | `Dashboard { focused: Sessions }` | happy-path |
| `Dashboard { focused: EventRibbon }`, `Enter` | `Fullscreen { panel: EventRibbon, prior: EventRibbon }` | edge-case (different panel) |
| `Filtering { panel: Sessions, query: "foo", prior: Sessions }`, `Enter` | `Filtering { panel: Sessions, query: "foo", prior: Sessions }` (no transition — identity) | edge-case |

**Note — `Action::Esc` vs `Action::ExitFullscreen`:** These are distinct variants after the
F-S025-ADV2-HIGH-002 split. `Action::Esc` is a builtin binding that is context-sensitive; in
Fullscreen mode it is an identity transition (no state change). `Action::ExitFullscreen` is a
per-context Fullscreen binding that triggers the actual fullscreen-exit transition to `Dashboard`.
The physical `Esc` key mapping to `Action::ExitFullscreen` is wired by the fullscreen-view story
(outside this BC's scope). Test vectors and PC-5 use `Action::ExitFullscreen` exclusively.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `(Dashboard { focused: Sessions }, Enter)` → `Fullscreen { panel: Sessions, prior: Sessions }` | Unit test (AppMode transition table) |
| VP-TBD | Fullscreen renderer does not panic when `app.sessions` is empty | Unit test (ratatui TestBackend with empty session list) |
| VP-TBD | Status bar breadcrumb shows "Dashboard > Sessions > Fullscreen" in Fullscreen mode | Unit test (ratatui TestBackend) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the fullscreen transition for the "sessions panel" component of CAP-006, enabling the session detail view that is referenced in the Phase 1 delivery contract |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — Fullscreen view is read-only: it renders data received via IPC; no writes) |
| Architecture Module | monocle-tui (draw_fullscreen(), transition() enter arm) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Panel Architecture §Sessions Panel (Fullscreen paragraph); §Rendering Architecture (draw_fullscreen() call in AppMode::Fullscreen match arm) |
| Cross-Ref | BC-2.06.001 (Fullscreen AppMode variant and Enter transition arm), BC-2.06.002 (FocusSnapshot restored on Escape from Fullscreen) |
| Test File | `monocle-tui/tests/sessions_fullscreen.rs` |
| Test Name | `test_BC_2_06_007_sessions_enter_transitions_to_fullscreen` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: Fullscreen AppMode variant and `(Dashboard { focused: Sessions }, Enter)` transition arm
- [BC-2.06.002] — composes with: Escape from Fullscreen restores FocusSnapshot per BC-2.06.002
- [BC-2.06.005] — composes with: the session data rendered in Fullscreen view comes from the same `app.sessions` source used by the Sessions Panel in BC-2.06.005

## Architecture Anchors

- `architecture/SS-tui.md#panel-architecture` — Sessions Panel fullscreen paragraph
- `architecture/SS-tui.md#rendering-architecture` — `draw_fullscreen()` and AppMode::Fullscreen match arm

## Story Anchor

S-TBD — Implement Sessions Panel fullscreen view with session detail (token history, cost, hook event count, phase tag); Escape returns to Dashboard (filled by story-writer)

## VP Anchors

- VP-TBD — ratatui TestBackend snapshot tests for Fullscreen content layout and status bar breadcrumb

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.007 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.1.0 §Panel Architecture §Sessions Panel (Fullscreen paragraph);
  §Rendering Architecture (draw_fullscreen call); prd-expansion-scope.md §3.3 BC-2.06.007
  description; ARCH-INDEX.md §Capability Traceability SS-06.
- Invariant 1 is critical for test-writer: `transition()` does not inspect `app.sessions`.
  The renderer handles the empty-session guard. This means the unit test for the transition
  function does NOT need to set up any session data — it tests pure `(AppMode, Action)` →
  `AppMode` pairs. The renderer unit test DOES need session data (or empty session data for
  the guard test).
- EC-097 notes that Enter on EventRibbon also triggers Fullscreen — this is covered by the
  general `(Dashboard { focused }, Action::Enter)` transition arm in BC-2.06.001, not by
  a separate BC for EventRibbon fullscreen.


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

**F-S025-ADV23-MED-001 Category 8 sweep — Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` (active pointer was stale by 3 minor versions).
- No substantive BC body prose propagation required: §Sessions Panel Fullscreen paragraph and §Rendering Architecture draw_fullscreen() specification are unchanged across v1.5.0→v1.8.2. The AppMode::Overlay shape change (v1.8.0) and daemon-status rendering change (v1.8.2) do not affect this BC's scope (fullscreen entry/exit/rendering).
- SE-16d monotonicity: v1.0.4 timestamp 2026-05-29T00:00:00Z > v1.0.3. PASS.

## §Trace v1.0.5

**Pass-39 adjudication item 6 — Pre-split `Action::Escape` terminology corrected** (2026-05-30T00:00:00Z):
- Defect: PC-5, §Description, and test-vector row used `Action::Escape` — a pre-split term that no longer
  exists as an Action variant after F-S025-ADV2-HIGH-002 split Esc into `Action::Esc` (context-sensitive,
  builtin-bound, identity in Fullscreen) and `Action::ExitFullscreen` (the actual fullscreen-exit action).
- §Description: "Pressing `Escape` returns to Dashboard" sentence updated to split terminology —
  `Action::ExitFullscreen` exits; `Action::Esc` (physical Esc key, builtin binding) is identity in Fullscreen;
  the Esc→ExitFullscreen key binding is the fullscreen-view story's responsibility.
- PC-5: replaced `Action::Escape` with `Action::ExitFullscreen`; corrected text per adjudication ruling
  (full transition call signature, physical key vs action distinction, per-context binding attribution).
- Test-vector row: `Fullscreen{…}, Escape → Dashboard` corrected to `Fullscreen{…}, ExitFullscreen → Dashboard`.
- Clarifying note added (before §Verification Properties) distinguishing `Action::Esc` (builtin, identity in
  Fullscreen) from `Action::ExitFullscreen` (per-context, exits fullscreen).
- EC-095 description ("Escape returns to Dashboard") left intentionally unchanged — that EC tests renderer
  behavior on empty-session entry/exit, and the informal "Escape" reference there is to the physical key
  event, not the Action variant; test-writer to use `Action::ExitFullscreen` in test code.
- SE-16d monotonicity: v1.0.5 timestamp 2026-05-30T00:00:00Z > v1.0.4 timestamp 2026-05-29T00:00:00Z. PASS.
