---
document_type: behavioral-contract
level: L3
version: "1.1.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T00:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "e1ed8bb"
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
`[↓]`, `Action::OverlayCycleNext` is dispatched. The `transition()` function returns
`AppMode` unchanged (since `Overlay { prior }` has no stack field to rotate). The actual
rotation is performed by the `App`-level handler: it rotates `App.overlay_stack:
VecDeque<PromptModal>` — the front `PromptModal` is moved to the back of the deque,
exposing the next queued prompt as the new front item. The overlay re-renders to show the
new front prompt. This allows the user to inspect all queued permission prompts before
deciding, without discarding any.

## Preconditions

1. `AppMode` is `Overlay { prior }` and `App.overlay_stack.len() >= 2`.
2. The keybinding dispatcher (`Dispatcher`) has `[↑]` / `[↓]` mapped to
   `Action::OverlayCycleNext` in the `PerContext` binding table for `AppMode::Overlay`.
3. The current front of `App.overlay_stack` (`App.overlay_stack.front()`) is the `PromptModal`
   currently rendered in the overlay header and body.

## Postconditions

1. **`pop_front` + `push_back` rotation on `App.overlay_stack`:** The `App`-level handler
   for `Action::OverlayCycleNext` pops the front item from `App.overlay_stack` and pushes
   it to the back. The previous second item is now the front. `AppMode` is not changed by
   this operation (since `Overlay { prior }` carries no stack field).
2. **New front item rendered:** After the rotation, the overlay renders the new
   `App.overlay_stack.front()` as the active prompt — showing its `tool_name`, `tool_payload`, and
   `session_id`.
3. **Stack length preserved:** `App.overlay_stack.len()` is unchanged after rotation. No
   `PromptModal` is created or destroyed by the rotate action.
4. **Single-item stack no-op:** When `App.overlay_stack.len() == 1`, rotating pop_front +
   push_back returns the same single item to the front. The rendered overlay is visually
   unchanged. No error or warning is produced.
5. **Badge counter unchanged:** The overlay badge counter in the status bar (showing number
   of queued prompts) does NOT change on rotation — rotation is not a decision.
6. **`prior` focus preserved:** The `prior: FocusSnapshot` field in `AppMode::Overlay { prior }`
   is unchanged by the rotation. Focus will still restore correctly when the overlay closes.

## Invariants

1. Rotation never makes `App.overlay_stack` empty. The `pop_front` + `push_back` pair
   executes as an atomic App-level operation; if `App.overlay_stack` was non-empty before,
   it is non-empty after.
2. The `VecDeque` ordering after rotation satisfies: if the original stack was
   `[P1, P2, P3]`, after one rotation it is `[P2, P3, P1]`. After a second rotation it is
   `[P3, P1, P2]`. After N rotations on a stack of N items it returns to `[P1, P2, P3]`.
3. No IPC message is sent to the daemon on rotation. The rotation is a local TUI state
   change only. The daemon's pending-prompt registry (`overlay_stack` in IPC) is not modified.
4. The `OverlayCycleNext` action has no defined behavior outside `AppMode::Overlay`. In
   any other mode, the App-level handler performs no `App.overlay_stack` mutation and
   `transition()` returns the original mode unchanged (identity transition per BC-2.06.001
   Postcondition 2).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-065 | `App.overlay_stack.len() == 1` when `OverlayCycleNext` is dispatched | `pop_front` returns the single item; `push_back` returns it to front; rendered output is unchanged; no error |
| EC-066 | `App.overlay_stack.len() == 0` when `AppMode` is `Overlay` — impossible in valid state | `Overlay` mode with empty `App.overlay_stack` is unreachable (BC-2.06.001). If reached in testing, `pop_front` returns `None`; no panic; rotation is a no-op; the App-level handler should collapse `AppMode` to `Dashboard { focused: prior }` per the empty-stack invariant |
| EC-067 | `OverlayCycleNext` dispatched while a `PermissionPromptQueued` IPC message is being processed | The IPC push and the key dispatch are both handled in the single-threaded event loop; they are strictly ordered; no concurrent mutation; the rotation sees either the pre-push or post-push stack |
| EC-068 | User holds `[↓]` key down (auto-repeat generates rapid `OverlayCycleNext` events) | Each event independently rotates the stack by one step; at 60fps tick rate with auto-repeat, the cycle proceeds step-by-step; no skipped prompts |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2]), `OverlayCycleNext` | `Overlay { prior: Sessions }` (App.overlay_stack = [P2, P1]) — P2 is now front | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2, P3]), `OverlayCycleNext` × 3 | `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2, P3]) — full rotation returns to original order | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]), `OverlayCycleNext` | `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) — single-item no-op | edge-case |
| `Dashboard { focused: Sessions }`, `OverlayCycleNext` | `Dashboard { focused: Sessions }` — no `App.overlay_stack` mutation; action ignored outside Overlay | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `OverlayCycleNext` never changes `App.overlay_stack.len()` | property test (proptest) |
| VP-TBD | After N rotations on a stack of N items, order returns to original | property test (proptest) |
| VP-TBD | `AppMode::Overlay::prior` field is identical before and after rotation | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the stack rotation behavior that is the core navigation mechanic of the "permission overlay stack" component of CAP-006, enabling users to inspect all queued prompts before deciding |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: rotation sends no IPC message and performs no file I/O) |
| Architecture Module | monocle-core (transition() rotate arm); monocle-tui (draw loop re-renders front item after transition) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §AppMode State Machine §Transition Function Contract (OverlayCycleNext arm); §Permission Overlay §Overlay Stack Lifecycle step 2 |
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

## §Trace v1.1.1

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): already propagated in §Trace v1.1.0 below.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers overlay stack rotation only; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
- SE-16d monotonicity: v1.1.1 timestamp 2026-05-29T00:00:00Z > v1.1.0. PASS.

## §Trace v1.1.0

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed; rotation moves to App level** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. Non-mechanical rewrite required: rotation is now an App-level effectful operation on `App.overlay_stack`, not a `transition()` arm operating on an `Overlay::stack` field.
- Description: `transition()` returns `AppMode` unchanged for `OverlayCycleNext`; actual rotation is performed by `App`-level handler on `App.overlay_stack: VecDeque<PromptModal>`.
- Preconditions: `Overlay { stack, prior }` → `Overlay { prior }` and `App.overlay_stack.len() >= 2`.
- Postconditions 1-6: all `stack.len()` / `stack.front()` references → `App.overlay_stack`; "transition() function" → "App-level handler"; AppMode not modified.
- Invariants 1, 4: reframed with `App.overlay_stack`; `transition()` role clarified.
- Edge Cases: `stack.len()` → `App.overlay_stack.len()`.
- Test vectors: `Overlay { stack: [...] }` shapes → `Overlay { prior: ... }` (App.overlay_stack noted).
- VP table: Kani proof harness for stack-length invariant replaced with proptest (App-level effectful operations cannot be exhaustively proved via Kani on pure functions).
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-28T00:00:00Z > v1.0.4. PASS.
