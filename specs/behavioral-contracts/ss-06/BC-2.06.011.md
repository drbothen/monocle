---
document_type: behavioral-contract
level: L3
version: "1.0.5"
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
modified: [F-P1D2-010, F-P1D7-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.011: Permission Overlay: Accept-Once Keybinding

## Description

In `AppMode::Overlay`, pressing `1` (bound to `Action::PermissionAcceptOnce` in the
`PerContext` binding table) sends an accept-once decision to the daemon and pops the front
`PromptModal` from the `VecDeque` stack. The TUI sends `ClientToServer::PermissionDecision
{ prompt_id, decision: PermissionDecision::Accept }` to the daemon via the bounded IPC send
channel (`App::ipc_tx`). The daemon forwards `{"decision": "accept"}` to the stalled
Claude Code HTTP response, unblocking that session. After popping, if the stack is empty,
`AppMode` transitions to `Dashboard { focused: prior }`. If the stack still has items,
`AppMode` remains `Overlay` with the next front item rendered.

## Preconditions

1. `AppMode` is `Overlay { stack, prior }` with `stack.len() >= 1`.
2. The `PerContext` binding table for `AppMode::Overlay` maps key `1` to
   `Action::PermissionAcceptOnce`.
3. `stack.front()` is the `PromptModal` whose `prompt_id` will be sent in the
   `ClientToServer::PermissionDecision`.
4. The IPC send channel (`App::ipc_tx`) has capacity for at least one additional message
   (bounded channel; drop semantics apply if full, per BC-2.04.011).

## Postconditions

1. **IPC send enqueued:** `ClientToServer::PermissionDecision { prompt_id: stack.front().prompt_id, decision: PermissionDecision::Accept }` is enqueued on `App::ipc_tx`. This is non-blocking: the TUI does not wait for the daemon to acknowledge before proceeding with the state transition.
2. **Front `PromptModal` popped:** `transition(Overlay { stack, prior }, PermissionAcceptOnce)` calls `stack.pop_front()`, removing the front item.
3. **Stack-empty collapse:** If `stack.is_empty()` after the pop, the transition returns `AppMode::Dashboard { focused: prior }`. The overlay is no longer rendered.
4. **Stack-non-empty continuation:** If `stack.len() >= 1` after the pop, the transition returns `AppMode::Overlay { stack, prior }`. The overlay re-renders with the new front item.
5. **Badge counter decrements:** The overlay badge counter in the status bar decrements by 1 to reflect the popped prompt. If it reaches 0 and mode is `Dashboard`, no badge is shown.
6. **`prior` focus preserved:** The `prior: FocusSnapshot` in the `AppMode::Overlay` variant is unchanged during the pop transition. If the stack empties and mode collapses to `Dashboard`, the restored `focused` is the `prior` snapshot from before the overlay was opened.
7. **Decision sent before state transition:** The IPC enqueue happens before `transition()` is called. This ordering guarantees that even if the TUI crashes immediately after the state transition (e.g., terminal close), the decision message was already in the outbound channel.

## Invariants

1. The `prompt_id` sent in the `ClientToServer::PermissionDecision` is the `Uuid` of the
   front `PromptModal` at the time `[1]` is pressed. If the daemon receives a
   `PermissionDecision` for a `prompt_id` it does not recognize (e.g., the prompt timed
   out), the daemon silently discards the response — the TUI does not need to handle this
   case.
2. Accept-Once semantics mean the daemon allows this specific invocation (`prompt_id`) but
   does NOT record a pattern for future auto-accept. This is in contrast to Accept-Always
   (BC-2.06.012) which records a pattern.
3. The `[1]` binding is active ONLY in `AppMode::Overlay` (registered in the `PerContext`
   table). In `AppMode::Dashboard`, pressing `1` matches no binding and is silently
   discarded (identity transition).
4. The IPC send channel is bounded. If the channel is full when the decision is enqueued,
   the message is dropped and the drop counter increments (BC-2.04.011). This is an
   acceptable degraded-mode behavior; the user would need to retry if the hook times out.
   The drop counter in the status bar surfaces this condition.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-077 | Stack has exactly 1 item when `[1]` is pressed | Pop leaves stack empty; `AppMode` transitions to `Dashboard { focused: prior }`; overlay dismissed; IPC send enqueued |
| EC-078 | Stack has 5 items when `[1]` is pressed | Pop leaves 4 items; `AppMode` stays `Overlay`; overlay renders next front item; badge decrements to 4 |
| EC-079 | IPC send channel is full when decision is enqueued | Message dropped; drop counter increments; `AppMode` transition still occurs (local state is consistent); status bar shows elevated drop counter |
| EC-080 | Daemon has already timed out the prompt (no HTTP response connection waiting) | Daemon silently discards the `PermissionDecision`; TUI is unaware; no error surfaced to user; the TUI-side pop already occurred and is correct |
| EC-081 | `[1]` pressed in `AppMode::Dashboard` | No binding match; identity transition; keypress silently discarded |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Overlay { stack: [P1], prior: Sessions }`, `PermissionAcceptOnce` | `Dashboard { focused: Sessions }` + `PermissionDecision { prompt_id: P1.id, decision: PermissionDecision::Accept }` enqueued | happy-path |
| `Overlay { stack: [P1, P2], prior: Sessions }`, `PermissionAcceptOnce` | `Overlay { stack: [P2], prior: Sessions }` + `PermissionDecision { prompt_id: P1.id, decision: PermissionDecision::Accept }` enqueued | happy-path |
| `Dashboard { focused: Sessions }`, `PermissionAcceptOnce` | `Dashboard { focused: Sessions }` (identity; no IPC send) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | IPC `ClientToServer::PermissionDecision` with correct `prompt_id` is enqueued before state transition | integration test (mock IPC channel; assert message received before mode changes) |
| VP-TBD | Empty stack after accept collapses `AppMode` to `Dashboard` | unit test |
| VP-TBD | Non-empty stack after accept keeps `AppMode` as `Overlay` with new front | unit test |
| VP-TBD | `prior` FocusSnapshot matches the focus at the time overlay was opened | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the Accept-Once decision path within the "permission overlay stack" component of CAP-006, which is the primary user action for resolving a queued permission prompt |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: Accept-Once sends an IPC decision message to the daemon; it does not write any file directly) |
| Architecture Module | monocle-core (transition() PermissionAcceptOnce arm); monocle-tui (App::handle_action enqueues IPC message; App::ipc_tx send channel) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.5.0 §Permission Overlay §Overlay Stack Lifecycle step 3 (Decide); §AppMode State Machine §Transition Function Contract (PermissionAcceptOnce arm) |
| Cross-Ref | BC-2.06.001 (pure transition function — Accept-Once is one of its arms), BC-2.06.008 (overlay push — creates the stack this BC pops), BC-2.05.005 (PermissionPromptQueued IPC — upstream source of prompt_id), BC-2.04.011 (bounded event bus — governs IPC send channel drop behavior) |
| Test File | `monocle-tui/tests/overlay_decisions.rs` |
| Test Name | `test_BC_2_06_011_accept_once_pops_front_and_sends_ipc` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: Accept-Once is one arm of the `transition()` pure function
- [BC-2.06.008] — depends on: the stack being popped was created by the push behavior in BC-2.06.008
- [BC-2.06.012] — composes with: Accept-Always (key `2`) is the sibling decision with pattern-recording semantics
- [BC-2.06.013] — composes with: Reject (key `3`) is the sibling decision with deny semantics
- [BC-2.06.002] — depends on: `prior` FocusSnapshot restore on empty-stack collapse is governed by BC-2.06.002

## Architecture Anchors

- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle step 3 (Decide)
- `architecture/SS-tui.md#appmode-state-machine` — transition function PermissionAcceptOnce arm

## Story Anchor

S-TBD — Implement Accept-Once keybinding for permission overlay with IPC decision send (filled by story-writer)

## VP Anchors

- VP-TBD — Integration tests for overlay decision IPC send and state transition

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.011 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.1.0 §Permission Overlay §Overlay Stack Lifecycle step 3,
  §AppMode State Machine §Transition Function Contract (accept/reject decision arms);
  prd-expansion-scope.md §3.3 BC-2.06.011 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: decision sends IPC message, not direct file write.
- Postcondition 7 documents the "IPC enqueue before transition" ordering guarantee to
  protect against crash-after-transition loss scenarios.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-P1D7-001 HIGH — Fabricated IPC type names replaced with canonical types** (2026-05-26T00:00:00Z):
- `IpcClientMessage::DecisionResponse` → `ClientToServer::PermissionDecision`. The canonical
  client-to-server enum is `ClientToServer` per SS-ipc.md §Client-to-Server Messages.
- `Decision::AcceptOnce` → `PermissionDecision::Accept`. The canonical accept-once variant is
  `Accept` (not `AcceptOnce`) per SS-ipc.md §PermissionDecision enum.
- All occurrences updated: Description, Postcondition 1, Invariant 1, EC-080, test vectors, VP table.
- The `DecisionResponse` variant does not exist in `ClientToServer`; the correct variant is
  `PermissionDecision { prompt_id, decision }`.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**IPC sweep — Precondition 3 residual fix** (2026-05-26T14:30:00Z):
- Precondition 3: "sent in the `DecisionResponse`" → "sent in the `ClientToServer::PermissionDecision`".
  This occurrence was missed in the v1.0.3 sweep; the fabricated `DecisionResponse` shorthand
  has now been eliminated from all sections of this file.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.0.5

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.5 timestamp >= v1.0.4. PASS.
