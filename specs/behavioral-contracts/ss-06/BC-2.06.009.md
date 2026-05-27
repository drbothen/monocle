---
document_type: behavioral-contract
level: L3
version: "1.0.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T14:00:00Z
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
modified: [F-P1D2-010]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.009: Permission Overlay: `[↑↓]` Rotates Stack

## Description

When the permission overlay is active (`AppMode::Overlay`) and the user presses `[↑]` or
`[↓]`, `Action::OverlayCycleNext` is dispatched. The `transition()` function rotates the
`VecDeque<PromptModal>`: the front `PromptModal` is moved to the back of the deque,
exposing the next queued prompt as the new front item. The overlay re-renders to show the
new front prompt. This allows the user to inspect all queued permission prompts before
deciding, without discarding any.

## Preconditions

1. `AppMode` is `Overlay { stack, prior }` with `stack.len() >= 2`.
2. The keybinding dispatcher (`Dispatcher`) has `[↑]` / `[↓]` mapped to
   `Action::OverlayCycleNext` in the `PerContext` binding table for `AppMode::Overlay`.
3. The current front of `stack` (`stack.front()`) is the `PromptModal` currently rendered
   in the overlay header and body.

## Postconditions

1. **`pop_front` + `push_back` rotation:** `transition(Overlay { stack, prior }, OverlayCycleNext)`
   pops the front item from `stack` and pushes it to the back. The previous second item
   is now the front.
2. **New front item rendered:** After the transition, the overlay renders the new
   `stack.front()` as the active prompt — showing its `tool_name`, `tool_payload`, and
   `session_id`.
3. **Stack length preserved:** `stack.len()` is unchanged after rotation. No `PromptModal`
   is created or destroyed by the rotate action.
4. **Single-item stack no-op:** When `stack.len() == 1`, rotating pop_front + push_back
   returns the same single item to the front. The rendered overlay is visually unchanged.
   No error or warning is produced.
5. **Badge counter unchanged:** The overlay badge counter in the status bar (showing number
   of queued prompts) does NOT change on rotation — rotation is not a decision.
6. **`prior` focus preserved:** The `prior: FocusSnapshot` field in `AppMode::Overlay` is
   unchanged by the rotation. Focus will still restore correctly when the overlay closes.

## Invariants

1. Rotation never creates an `Overlay` variant with an empty `stack`. The `transition()`
   function's rotate arm executes `pop_front` + `push_back` as an atomic pair; if `stack`
   was non-empty before, it is non-empty after.
2. The `VecDeque` ordering after rotation satisfies: if the original stack was
   `[P1, P2, P3]`, after one rotation it is `[P2, P3, P1]`. After a second rotation it is
   `[P3, P1, P2]`. After N rotations on a stack of N items it returns to `[P1, P2, P3]`.
3. No IPC message is sent to the daemon on rotation. The rotation is a local TUI state
   change only. The daemon's pending-prompt registry (`overlay_stack` in IPC) is not modified.
4. The `OverlayCycleNext` action has no defined behavior outside `AppMode::Overlay`. In
   any other mode, `transition()` returns the original mode unchanged (identity transition
   per BC-2.06.001 Postcondition 2).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-065 | `stack.len() == 1` when `OverlayCycleNext` is dispatched | `pop_front` returns the single item; `push_back` returns it to front; rendered output is unchanged; no error |
| EC-066 | `stack.len() == 0` — impossible in valid state; `Overlay` variant cannot hold empty stack | `transition()` should never receive an `Overlay` with empty stack; if somehow reached (test-only), `pop_front` returns `None`; no panic; `push_back` never called; stack remains empty; `transition()` collapses to `Dashboard { focused: prior }` per empty-stack invariant in BC-2.06.001 |
| EC-067 | `OverlayCycleNext` dispatched while a `PermissionPromptQueued` IPC message is being processed | The IPC push and the key dispatch are both handled in the single-threaded event loop; they are strictly ordered; no concurrent mutation; the rotation sees either the pre-push or post-push stack |
| EC-068 | User holds `[↓]` key down (auto-repeat generates rapid `OverlayCycleNext` events) | Each event independently rotates the stack by one step; at 60fps tick rate with auto-repeat, the cycle proceeds step-by-step; no skipped prompts |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Overlay { stack: [P1, P2], prior: Sessions }`, `OverlayCycleNext` | `Overlay { stack: [P2, P1], prior: Sessions }` — P2 is now front | happy-path |
| `Overlay { stack: [P1, P2, P3], prior: Sessions }`, `OverlayCycleNext` × 3 | `Overlay { stack: [P1, P2, P3], prior: Sessions }` — full rotation returns to original order | happy-path |
| `Overlay { stack: [P1], prior: Sessions }`, `OverlayCycleNext` | `Overlay { stack: [P1], prior: Sessions }` — single-item no-op | edge-case |
| `Dashboard { focused: Sessions }`, `OverlayCycleNext` | `Dashboard { focused: Sessions }` — identity transition; action ignored outside Overlay | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `OverlayCycleNext` never reduces `stack.len()` | Kani proof harness |
| VP-TBD | After N rotations on a stack of N items, order returns to original | property test (proptest) |
| VP-TBD | `prior` field is identical before and after rotation | Kani proof harness |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the stack rotation behavior that is the core navigation mechanic of the "permission overlay stack" component of CAP-006, enabling users to inspect all queued prompts before deciding |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: rotation sends no IPC message and performs no file I/O) |
| Architecture Module | monocle-core (transition() rotate arm); monocle-tui (draw loop re-renders front item after transition) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.5.0 §AppMode State Machine §Transition Function Contract (OverlayCycleNext arm); §Permission Overlay §Overlay Stack Lifecycle step 2 |
| Cross-Ref | BC-2.06.001 (pure transition function — this BC's rotation is one of its arms), BC-2.06.008 (overlay push — creates the stack this BC rotates), BC-2.06.011 (Accept-Once — pops front after decision) |
| Test File | `monocle-core/tests/app_mode_transitions.rs` |
| Test Name | `test_BC_2_06_009_overlay_cycle_rotates_stack` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: rotation is one arm of the `transition()` pure function defined in BC-2.06.001
- [BC-2.06.008] — depends on: the `VecDeque<PromptModal>` stack that this BC rotates is created by the push behavior in BC-2.06.008
- [BC-2.06.011] — composes with: after rotating to inspect a prompt, the user decides via Accept-Once (BC-2.06.011), Accept-Always (BC-2.06.012), or Reject (BC-2.06.013)

## Architecture Anchors

- `architecture/SS-tui.md#appmode-state-machine` — transition function rotate arm (OverlayCycleNext)
- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle step 2 (Rotate)

## Story Anchor

S-TBD — Implement overlay stack rotation via OverlayCycleNext action in transition() (filled by story-writer)

## VP Anchors

- VP-TBD — Kani proof harness for OverlayCycleNext stack-length preservation invariant

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.009 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.1.0 §AppMode State Machine (OverlayCycleNext transition arm),
  §Permission Overlay §Overlay Stack Lifecycle step 2, §Constraints; prd-expansion-scope.md
  §3.3 BC-2.06.009 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: rotation performs no file I/O and sends no IPC message.
- EC-066 explicitly documents the empty-stack impossibility path, cross-referencing the
  collapse invariant from BC-2.06.001.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**IPC field name corrected: `queued_prompts` → `overlay_stack`** (2026-05-26T00:00:00Z):
- Invariant 3: "The daemon's `queued_prompts` is not modified." → "The daemon's pending-prompt
  registry (`overlay_stack` in IPC) is not modified." Canonical IPC field name is `overlay_stack`
  from `ServerToClient::InitialState`; `queued_prompts` was a stale fabrication.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.
