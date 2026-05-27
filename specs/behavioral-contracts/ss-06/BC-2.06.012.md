---
document_type: behavioral-contract
level: L3
version: "1.0.0"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.012: Permission Overlay: Accept-Always Keybinding

## Description

In `AppMode::Overlay`, pressing `2` (bound to `Action::PermissionAcceptAlways` in the
`PerContext` binding table) sends an accept-always decision to the daemon and pops the
front `PromptModal` from the `VecDeque` stack. The TUI sends `IpcClientMessage::DecisionResponse
{ prompt_id, decision: Decision::AcceptAlways }` to the daemon via `App::ipc_tx`. The
daemon forwards `{"decision": "always"}` to the stalled Claude Code HTTP response and
records the tool-pattern for future auto-accept, so subsequent identical tool calls from
the same session are automatically allowed without requiring user interaction. Accept-Always
differs from Accept-Once (BC-2.06.011) solely in the decision value sent; the TUI-side
stack management and state transition behavior are identical.

## Preconditions

1. `AppMode` is `Overlay { stack, prior }` with `stack.len() >= 1`.
2. The `PerContext` binding table for `AppMode::Overlay` maps key `2` to
   `Action::PermissionAcceptAlways`.
3. `stack.front()` is the `PromptModal` whose `prompt_id` will be sent in the
   `DecisionResponse`.
4. The IPC send channel (`App::ipc_tx`) has capacity for at least one additional message.

## Postconditions

1. **IPC send enqueued:** `IpcClientMessage::DecisionResponse { prompt_id: stack.front().prompt_id, decision: Decision::AcceptAlways }` is enqueued on `App::ipc_tx` before the state transition. This is non-blocking.
2. **Front `PromptModal` popped:** `transition(Overlay { stack, prior }, PermissionAcceptAlways)` calls `stack.pop_front()`, removing the front item.
3. **Stack-empty collapse:** If `stack.is_empty()` after the pop, the transition returns `AppMode::Dashboard { focused: prior }`.
4. **Stack-non-empty continuation:** If `stack.len() >= 1` after the pop, the transition returns `AppMode::Overlay { stack, prior }` with the new front item rendered.
5. **Badge counter decrements:** The overlay badge counter decrements by 1. If it reaches 0 and mode is `Dashboard`, no badge is shown.
6. **`prior` focus preserved:** The `prior: FocusSnapshot` is unchanged during the transition and is correctly restored if the stack empties.
7. **Pattern recording is daemon-side:** The TUI sends `Decision::AcceptAlways` in the IPC message. The daemon is solely responsible for recording the pattern (tool + path + session). The TUI does not maintain any pattern state; it has no knowledge of which future prompts will be auto-accepted.

## Invariants

1. `Decision::AcceptAlways` carries the same `prompt_id` as `Decision::AcceptOnce` — the
   distinction lies solely in the `decision` field value. The TUI-side stack pop and mode
   transition logic is identical for both decisions.
2. Pattern recording by the daemon is out of scope for this BC. This BC specifies only the
   TUI-side behavior (IPC send + stack pop + mode transition).
3. The `[2]` binding is active ONLY in `AppMode::Overlay` (PerContext table). Pressing `2`
   in `AppMode::Dashboard` or `AppMode::Filtering` has no effect (identity transition per
   BC-2.06.001).
4. The IPC send channel is bounded. If the channel is full, the message is dropped and the
   drop counter increments (BC-2.04.011). The state transition still occurs.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-082 | Stack has exactly 1 item when `[2]` is pressed | Pop leaves stack empty; `AppMode` transitions to `Dashboard { focused: prior }`; IPC `Decision::AcceptAlways` enqueued |
| EC-083 | Stack has 3 items when `[2]` is pressed | Pop leaves 2 items; `AppMode` stays `Overlay`; overlay renders next front item; badge decrements |
| EC-084 | Daemon receives `Decision::AcceptAlways` but the pattern-recording logic fails internally | Daemon-side failure; TUI is unaware; the session is unblocked as with Accept-Once; no TUI error |
| EC-085 | `[2]` pressed in `AppMode::Dashboard` | No binding match; identity transition; keypress silently discarded |
| EC-086 | Two concurrent prompts for the same tool+path; user accepts-always the first | Daemon records the pattern; second prompt in the TUI stack may or may not auto-resolve before the user acts on it — this is a daemon-side race condition outside TUI scope |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Overlay { stack: [P1], prior: Sessions }`, `PermissionAcceptAlways` | `Dashboard { focused: Sessions }` + `DecisionResponse { prompt_id: P1.id, decision: AcceptAlways }` enqueued | happy-path |
| `Overlay { stack: [P1, P2], prior: Sessions }`, `PermissionAcceptAlways` | `Overlay { stack: [P2], prior: Sessions }` + `DecisionResponse { prompt_id: P1.id, decision: AcceptAlways }` enqueued | happy-path |
| `Dashboard { focused: Sessions }`, `PermissionAcceptAlways` | `Dashboard { focused: Sessions }` (identity; no IPC send) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | IPC `DecisionResponse` with `decision: AcceptAlways` and correct `prompt_id` is enqueued before state transition | integration test |
| VP-TBD | Empty stack after accept-always collapses `AppMode` to `Dashboard` | unit test |
| VP-TBD | `prior` FocusSnapshot restored correctly on stack-empty collapse | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the Accept-Always decision path within the "permission overlay stack" component of CAP-006, enabling the user to grant persistent per-tool permission to avoid repeated prompts |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: Accept-Always sends an IPC decision message; pattern recording is daemon-side; TUI writes no files) |
| Architecture Module | monocle-core (transition() PermissionAcceptAlways arm); monocle-tui (App::handle_action enqueues IPC message) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.0.0 §Permission Overlay §Overlay Stack Lifecycle step 3 (Decide); §AppMode State Machine §Transition Function Contract (PermissionAcceptAlways arm) |
| Cross-Ref | BC-2.06.011 (Accept-Once — sibling with identical TUI behavior, different decision value), BC-2.06.013 (Reject — sibling decision), BC-2.06.001 (pure transition function), BC-2.06.008 (overlay push), BC-2.04.011 (bounded event bus drop counter) |
| Test File | `monocle-tui/tests/overlay_decisions.rs` |
| Test Name | `test_BC_2_06_012_accept_always_pops_front_and_sends_ipc` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: Accept-Always is one arm of the `transition()` pure function
- [BC-2.06.011] — composes with: Accept-Once (key `1`) is the sibling with one-time semantics
- [BC-2.06.013] — composes with: Reject (key `3`) is the sibling with deny semantics
- [BC-2.06.008] — depends on: the stack being popped was created by the push behavior in BC-2.06.008
- [BC-2.06.002] — depends on: `prior` FocusSnapshot restore on empty-stack collapse

## Architecture Anchors

- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle step 3 (Decide, AcceptAlways arm)
- `architecture/SS-tui.md#appmode-state-machine` — transition function PermissionAcceptAlways arm

## Story Anchor

S-TBD — Implement Accept-Always keybinding for permission overlay with IPC decision send (filled by story-writer)

## VP Anchors

- VP-TBD — Integration tests for Accept-Always IPC send and state transition

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.012 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.0.0 §Permission Overlay §Overlay Stack Lifecycle step 3,
  §AppMode State Machine §Transition Function Contract; prd-expansion-scope.md §3.3
  BC-2.06.012 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: TUI sends IPC message only; pattern recording is daemon-side.
- Postcondition 7 explicitly scopes pattern recording to the daemon to enforce the
  TUI-is-a-client invariant (SS-tui.md §Architectural Principle: Observe-Only).
