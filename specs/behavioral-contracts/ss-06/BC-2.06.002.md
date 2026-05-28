---
document_type: behavioral-contract
level: L3
version: "1.0.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T00:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "6e22061"
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

# Behavioral Contract BC-2.06.002: FocusSnapshot: Focus Restored After Overlay/Fullscreen Close

## Description

When the TUI transitions from `Dashboard` into `Overlay` or `Fullscreen` mode, the
current `FocusSnapshot` is captured in the `prior` field of the new `AppMode` variant.
When the user exits back to `Dashboard` (via `Escape` from `Fullscreen`, or via decision
from `Overlay`), the `AppMode` restores to `Dashboard { focused: <prior snapshot> }` —
never to a default or hardcoded value. This prevents the NikiforovAll/lazyclaude gap
where modal-close from Sessions loses the Sessions context.

## Preconditions

1. `FocusSnapshot` is defined in `monocle-core/src/app_mode.rs` as an enum with at least
   two variants: `Sessions` and `EventRibbon`. Both are `#[non_exhaustive]` to allow Phase
   2 panel additions.
2. `FocusSnapshot` derives `Clone`, `PartialEq`, `Eq`, and `Debug`.
3. `FocusSnapshot::cycle()` is a pure method that advances focus to the next panel in
   round-robin order.
4. `FocusSnapshot::to_panel_id()` is a pure method that converts a `FocusSnapshot` to the
   corresponding `PanelId`.
5. The `AppMode::Overlay`, `AppMode::Fullscreen`, and `AppMode::Filtering` variants each
   carry a `prior: FocusSnapshot` field.
6. The `transition()` function (BC-2.06.001) is the sole code path for `AppMode` changes
   in response to user actions.

## Postconditions

1. **Fullscreen close restores prior focus:** When `transition(AppMode::Fullscreen { panel, prior }, Action::Escape)`
   is called, the returned `AppMode` is `Dashboard { focused: prior }` where `prior` is
   exactly the `FocusSnapshot` captured when `Fullscreen` was entered. The `panel`
   argument is not used to derive `focused`.
2. **Overlay close restores prior focus (stack emptied):** When `App.overlay_stack`
   becomes empty after a prompt removal (triggered by `PermissionAcceptOnce`,
   `PermissionAcceptAlways`, `PermissionReject`, or a daemon-initiated `PermissionPromptResolved`),
   the TUI collapses `AppMode` to `Dashboard { focused: prior }` where `prior` is the
   `FocusSnapshot` captured in the `Overlay { prior }` variant when the overlay was first
   entered (not when the final item was removed).
3. **Overlay `prior` is not mutated by stack rotation:** `Action::OverlayCycleNext` rotates
   `App.overlay_stack` (front → back) but does NOT change the `prior: FocusSnapshot` field
   of `AppMode::Overlay`. After any number of `OverlayCycleNext` actions, the `prior` field
   in `AppMode::Overlay { prior }` remains identical to the value it held when the `Overlay`
   variant was first constructed.
4. **Filtering close restores prior focus:** When `transition(AppMode::Filtering { prior, .. }, Action::Escape)`
   or `transition(AppMode::Filtering { prior, .. }, Action::FilterClear)` is called, the
   returned `AppMode` is `Dashboard { focused: prior }`.
5. **No code path produces `Dashboard { focused: <hardcoded Sessions> }` as a fallback:**
   Every `Dashboard` construction in `transition()` uses a `prior: FocusSnapshot` variable,
   never a literal `FocusSnapshot::Sessions`. (Exception: the IPC disconnect path in
   BC-2.06.016 resets to `Sessions` explicitly — that is the one documented reset to
   default focus; it is not an oversight.)

## Invariants

1. `FocusSnapshot` is always non-None when the TUI is running. There is no `Option<FocusSnapshot>`
   in the codebase — the enum is always a concrete variant.
2. The `prior` field in `Overlay`, `Fullscreen`, and `Filtering` is set exactly once (when
   the variant is constructed by `transition()`) and never mutated thereafter.
3. Nesting is not permitted: transitioning from `Overlay` to `Fullscreen` directly is
   undefined (no `transition()` arm handles `(Overlay, Enter)`). The identity arm returns
   `Overlay` unchanged, preventing accidental nesting.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-065 | User opens Overlay (Sessions focused), then cycles `App.overlay_stack` 3 times via `OverlayCycleNext`, then decides | `Dashboard { focused: Sessions }` — `prior` in `AppMode::Overlay { prior }` is unchanged by stack cycling |
| EC-066 | User opens Fullscreen from EventRibbon panel | `Fullscreen { panel: EventRibbon, prior: EventRibbon }` entered; `Escape` returns `Dashboard { focused: EventRibbon }` |
| EC-067 | User opens Filtering, types "foo", clears with `FilterClear` | `Dashboard { focused: <prior at filter-open> }` — focus is the panel that was focused before `/` was pressed |
| EC-068 | User presses `Escape` in `Dashboard` mode | Identity transition: `Dashboard { focused: <current> }` — no focus change |
| EC-069 | `FocusSnapshot::cycle()` called when only one panel exists in Phase 1 | Returns the same `FocusSnapshot::Sessions` (single-panel round-robin is idempotent) |

## Canonical Test Vectors

| Input (mode, action) | Expected Output `focused` | Category |
|----------------------|--------------------------|----------|
| `Fullscreen { panel: Sessions, prior: Sessions }`, `Escape` | `Dashboard { focused: Sessions }` | happy-path |
| `Fullscreen { panel: EventRibbon, prior: EventRibbon }`, `Escape` | `Dashboard { focused: EventRibbon }` | happy-path |
| `Overlay { prior: EventRibbon }` (App.overlay_stack = [P1]), `PermissionAcceptOnce` | `Dashboard { focused: EventRibbon }` after `PermissionPromptResolved` empties `App.overlay_stack` | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2]), `OverlayCycleNext` then `PermissionReject` | `Overlay { prior: Sessions }` (App.overlay_stack = [P1] after resolution; prior unchanged by cycle) | edge-case |
| `Filtering { panel: Sessions, query: "foo", prior: EventRibbon }`, `Escape` | `Dashboard { focused: EventRibbon }` | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | When `App.overlay_stack` empties after prompt removal while `AppMode` is `Overlay`, `AppMode` collapses to `Dashboard { focused: prior }` | Integration test (inject `PermissionPromptResolved` for last prompt; assert AppMode = Dashboard) |
| VP-TBD | For all `(Overlay, OverlayCycleNext)` inputs, `AppMode::Overlay::prior` equals input `prior` (rotation does not mutate `prior`) | Kani proof harness |
| VP-TBD | `FocusSnapshot::cycle()` is total and never panics | Unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the FocusSnapshot capture/restore mechanism that is part of the AppMode state machine component of CAP-006, preventing the gap identified in the NikiforovAll reference implementation |
| L2 Domain Invariants | DI-006 (every EngineModule implementation must be stateless — orthogonally reinforced here: the `FocusSnapshot` design ensures the TUI panel focus is tracked via pure value types, not shared mutable state) |
| Architecture Module | monocle-core (FocusSnapshot enum, cycle(), to_panel_id()) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.5.0 §AppMode State Machine (FocusSnapshot definition, transition function arms for Filtering/Fullscreen/Overlay exit) |
| Cross-Ref | BC-2.06.001 (transition function definition that enforces this contract), BC-2.06.016 (daemon-disconnect focus reset — the one documented exception to pure prior restoration) |
| Test File | `monocle-core/tests/app_mode_transitions.rs` |
| Test Name | `test_BC_2_06_002_focus_snapshot_restored_after_modal_close` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: `transition()` is the function that implements this contract
- [BC-2.06.007] — composes with: Enter → Fullscreen transition must capture FocusSnapshot per this BC
- [BC-2.06.008] — composes with: Overlay push must capture FocusSnapshot per this BC
- [BC-2.06.016] — exception to: daemon-disconnect explicitly resets to `Sessions` rather than restoring `prior`

## Architecture Anchors

- `architecture/SS-tui.md#appmode-state-machine` — FocusSnapshot enum definition and transition arms
- `architecture/SS-tui.md#appmode-state-machine` — "Key invariants enforced by this design" paragraph

## Story Anchor

S-TBD — Implement FocusSnapshot enum with cycle() and to_panel_id() methods; verify prior-restoration in transition() (filled by story-writer)

## VP Anchors

- VP-TBD — Kani proof harnesses verifying FocusSnapshot preservation through all transition arms

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.002 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.1.0 §AppMode State Machine (FocusSnapshot definition, key invariants
  paragraph, transition function arms for Fullscreen, Filtering, and Overlay exit);
  prd-expansion-scope.md §3.3 BC-2.06.002 description; ARCH-INDEX.md §Capability Traceability SS-06.
- EC-069 notes that Phase 1 has only two panel variants (Sessions, EventRibbon); cycle() on
  a single-panel state is idempotent. Phase 2 adds Customizations, Workflow, Preview, which
  will extend the cycle order without changing this BC.
- Postcondition 5 explicitly documents the IPC-disconnect exception (BC-2.06.016) to prevent
  false adversary findings about "hardcoded Sessions" in the disconnect path.


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

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. `App.overlay_stack: VecDeque<PromptModal>` is now the single source of truth for the modal stack. `AppMode::Overlay` carries only `{ prior: FocusSnapshot }`.
- Postcondition 2: "stack becomes empty" reframed as `App.overlay_stack` emptying after `retain()` removal; "returned AppMode" → "TUI collapses AppMode"; collapse is App-level, not a `transition()` return.
- Postcondition 3: rotation operates on `App.overlay_stack`, not on `AppMode::Overlay::stack`; `prior` field references now explicitly cite `AppMode::Overlay { prior }`.
- EC-065: "cycles stack" → "cycles `App.overlay_stack`"; `prior` reference updated.
- Test vectors: `Overlay { stack: [P1], prior: ... }` → `Overlay { prior: ... }` (App.overlay_stack noted).
- VP table: collapse VP updated to integration test rather than Kani (the collapse is App-level effectful).
- SE-16d monotonicity: v1.0.4 timestamp 2026-05-28T00:00:00Z > v1.0.3. PASS.
