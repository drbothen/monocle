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

# Behavioral Contract BC-2.06.013: Permission Overlay: Reject Keybinding

## Description

In `AppMode::Overlay`, pressing `3` (bound to `Action::PermissionReject` in the `PerContext`
binding table) sends a deny decision to the daemon and pops the front `PromptModal` from
the `VecDeque` stack. The TUI sends `IpcClientMessage::DecisionResponse { prompt_id, decision: Decision::Deny }` to the daemon via `App::ipc_tx`. The daemon forwards `{"decision": "deny"}` to the stalled Claude Code HTTP response; Claude Code receives the deny and does not execute the tool. The TUI-side stack management and mode transition behavior are identical to Accept-Once (BC-2.06.011) and Accept-Always (BC-2.06.012), except the decision value is `Deny`.

## Preconditions

1. `AppMode` is `Overlay { stack, prior }` with `stack.len() >= 1`.
2. The `PerContext` binding table for `AppMode::Overlay` maps key `3` to
   `Action::PermissionReject`.
3. `stack.front()` is the `PromptModal` whose `prompt_id` will be sent in the
   `DecisionResponse`.
4. The IPC send channel (`App::ipc_tx`) has capacity for at least one additional message.

## Postconditions

1. **IPC send enqueued:** `IpcClientMessage::DecisionResponse { prompt_id: stack.front().prompt_id, decision: Decision::Deny }` is enqueued on `App::ipc_tx` before the state transition. This is non-blocking.
2. **Front `PromptModal` popped:** `transition(Overlay { stack, prior }, PermissionReject)` calls `stack.pop_front()`, removing the front item.
3. **Stack-empty collapse:** If `stack.is_empty()` after the pop, the transition returns `AppMode::Dashboard { focused: prior }`.
4. **Stack-non-empty continuation:** If `stack.len() >= 1` after the pop, the transition returns `AppMode::Overlay { stack, prior }` with the new front item rendered.
5. **Badge counter decrements:** The overlay badge counter decrements by 1.
6. **`prior` focus preserved:** The `prior: FocusSnapshot` is unchanged during the transition and is correctly restored if the stack empties.
7. **Deny is not the same as `[Esc]` hide:** `[3]` (Reject) POPS the `PromptModal` and sends a decision to the daemon. `[Esc]` (BC-2.06.014) does NOT pop and does NOT send a decision. This distinction is CRITICAL: `[Esc]` must never be confused with reject by the implementation.

## Invariants

1. `Decision::Deny` carries the same `prompt_id` as `Decision::AcceptOnce` and
   `Decision::AcceptAlways`. The TUI-side stack management is identical for all three
   decision types.
2. The `[3]` binding is active ONLY in `AppMode::Overlay` (PerContext table). Pressing `3`
   in `AppMode::Dashboard` or `AppMode::Filtering` has no effect.
3. The IPC send channel is bounded. If full, the message is dropped and the drop counter
   increments (BC-2.04.011). The state transition still occurs.
4. Deny semantics (whether Claude Code retries the tool, aborts the task, or surfaces the
   rejection to the user) are governed by Claude Code's behavior, not by monocle. The
   monocle TUI sends `{"decision": "deny"}` and is done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-087 | Stack has exactly 1 item when `[3]` is pressed | Pop leaves stack empty; `AppMode` transitions to `Dashboard { focused: prior }`; IPC `Decision::Deny` enqueued |
| EC-088 | Stack has 4 items when `[3]` is pressed | Pop leaves 3 items; `AppMode` stays `Overlay`; overlay renders next front item; badge decrements |
| EC-089 | IPC send channel is full when Reject is enqueued | Message dropped; drop counter increments; state transition still occurs; user may not see Claude Code response; the hook will eventually time out at the daemon with fail-open semantics (BC-HOOK-001) |
| EC-090 | `[3]` pressed in `AppMode::Dashboard` | No binding match; identity transition; keypress silently discarded |
| EC-091 | User presses `[Esc]` intending to hide but `[3]` was the key sent (e.g., terminal mapping issue) | `[3]` sends Reject + pops; `[Esc]` only hides; these are distinct keys — terminal-mapping issues are out of scope for this BC |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Overlay { stack: [P1], prior: Sessions }`, `PermissionReject` | `Dashboard { focused: Sessions }` + `DecisionResponse { prompt_id: P1.id, decision: Deny }` enqueued | happy-path |
| `Overlay { stack: [P1, P2], prior: Sessions }`, `PermissionReject` | `Overlay { stack: [P2], prior: Sessions }` + `DecisionResponse { prompt_id: P1.id, decision: Deny }` enqueued | happy-path |
| `Dashboard { focused: Sessions }`, `PermissionReject` | `Dashboard { focused: Sessions }` (identity; no IPC send) | edge-case |
| `Overlay { stack: [P1], prior: Sessions }`, `Escape` | `Overlay { stack: [P1], prior: Sessions }` (Esc does NOT pop; must be distinguished from Reject) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | IPC `DecisionResponse` with `decision: Deny` and correct `prompt_id` is enqueued before state transition | integration test |
| VP-TBD | `Action::Escape` in `Overlay` does NOT call `stack.pop_front()` | unit test (confirm Esc-vs-Reject distinction) |
| VP-TBD | Empty stack after reject collapses `AppMode` to `Dashboard` | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the Reject (deny) decision path within the "permission overlay stack" component of CAP-006, which is the user's mechanism to prevent a Claude Code tool from executing |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: Reject sends an IPC decision message; the TUI writes no files) |
| Architecture Module | monocle-core (transition() PermissionReject arm); monocle-tui (App::handle_action enqueues IPC message) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.0.0 §Permission Overlay §Overlay Stack Lifecycle step 3 (Decide); §AppMode State Machine §Transition Function Contract (PermissionReject arm) |
| Cross-Ref | BC-2.06.011 (Accept-Once — sibling), BC-2.06.012 (Accept-Always — sibling), BC-2.06.014 (Esc Hide — CRITICAL distinction: Esc does NOT pop), BC-2.06.001 (pure transition function), BC-2.04.011 (bounded event bus) |
| Test File | `monocle-tui/tests/overlay_decisions.rs` |
| Test Name | `test_BC_2_06_013_reject_pops_front_and_sends_deny_ipc` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: Reject is one arm of the `transition()` pure function
- [BC-2.06.011] — composes with: Accept-Once (key `1`) is the sibling with accept semantics
- [BC-2.06.012] — composes with: Accept-Always (key `2`) is the sibling with always-accept semantics
- [BC-2.06.014] — CRITICAL DISTINCTION: Esc (BC-2.06.014) hides without popping; Reject pops and sends deny; these must never be conflated in implementation
- [BC-2.06.008] — depends on: the stack being popped was created by the push behavior in BC-2.06.008

## Architecture Anchors

- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle step 3 (Decide, Reject arm)
- `architecture/SS-tui.md#appmode-state-machine` — transition function PermissionReject arm
- `architecture/SS-tui.md#status-bar` — keybinding hint line: `3: reject`

## Story Anchor

S-TBD — Implement Reject keybinding for permission overlay with IPC deny send (filled by story-writer)

## VP Anchors

- VP-TBD — Integration tests for Reject IPC send, state transition, and Esc-vs-Reject distinction

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.013 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.0.0 §Permission Overlay §Overlay Stack Lifecycle step 3,
  §AppMode State Machine §Transition Function Contract; prd-expansion-scope.md §3.3
  BC-2.06.013 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: TUI sends IPC message only; no file writes.
- Postcondition 7 and EC-091 + Related BCs explicitly flag the CRITICAL Esc-vs-Reject
  distinction to prevent implementation conflation.
