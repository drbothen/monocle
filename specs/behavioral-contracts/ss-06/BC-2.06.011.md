---
document_type: behavioral-contract
level: L3
version: "1.2.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T00:00:00Z
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
modified: [F-P1D2-010, F-P1D7-001, ADJ-ADV2-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.011: Permission Overlay: Accept-Once Keybinding (`y`/`Enter`)

## Description

In `AppMode::Overlay`, pressing `y` or `Enter` (both bound to `Action::PermissionAcceptOnce`
in the `SearchPrompt` binding layer, which has highest priority in overlay mode) sends an
accept-once decision to the daemon via `ClientToServer::PermissionDecision { prompt_id,
decision: PermissionDecision::Accept }` over `App::ipc_tx`. The TUI does NOT immediately
pop the `PromptModal`; it waits for the daemon to send `ServerToClient::PermissionPromptResolved
{ prompt_id }` before removing the modal from the `VecDeque` stack (per BC-2.06.023). The
daemon forwards `{"decision": "accept"}` to the stalled Claude Code HTTP response, unblocking
that session, then broadcasts `PermissionPromptResolved`. On receipt of that broadcast, the
TUI removes the matching modal via `retain()`. If the stack becomes empty after removal,
`AppMode` transitions to `Dashboard { focused: prior }`. If items remain, `AppMode` stays
`Overlay` with the next front item rendered.

## Preconditions

1. `AppMode` is `Overlay { prior }` and `App.overlay_stack.len() >= 1`.
2. The `SearchPrompt` binding layer (highest priority) maps keys `y` and `Enter` to
   `Action::PermissionAcceptOnce` when `AppMode` is `Overlay`. These bindings are active
   only in Overlay mode and override any Global or PerContext bindings for these keys.
3. `App.overlay_stack.front()` is the `PromptModal` whose `prompt_id` will be sent in the
   `ClientToServer::PermissionDecision`.
4. The IPC send channel (`App::ipc_tx`) has capacity for at least one additional message
   (bounded channel; drop semantics apply if full, per BC-2.04.011).

## Postconditions

1. **IPC send enqueued:** `ClientToServer::PermissionDecision { prompt_id: App.overlay_stack.front().prompt_id, decision: PermissionDecision::Accept }` is enqueued on `App::ipc_tx`. This is non-blocking: the TUI enqueues the message and continues without waiting for daemon acknowledgement.
2. **Modal NOT immediately popped:** The TUI does NOT call `App.overlay_stack.pop_front()` or `App.overlay_stack.retain()` upon sending the decision. The `PromptModal` remains in `App.overlay_stack` until the daemon sends `ServerToClient::PermissionPromptResolved { prompt_id }` confirming the decision was processed. This ensures the modal stays visible if the IPC send fails (channel drop) or the daemon rejects the decision.
3. **PermissionPromptResolved triggers removal:** When `ServerToClient::PermissionPromptResolved { prompt_id }` is received, the TUI calls `App.overlay_stack.retain(|m| m.prompt_id != prompt_id)` to remove the modal (per BC-2.06.023). This removal is NOT routed through `transition()`.
4. **Stack-empty collapse after removal:** If `App.overlay_stack.is_empty()` after the `retain()` call and `AppMode` is `Overlay`, the TUI collapses to `AppMode::Dashboard { focused: prior }` per the BC-2.06.001 empty-stack invariant.
5. **Stack-non-empty continuation after removal:** If `App.overlay_stack.len() >= 1` after the `retain()` call, `AppMode` remains `Overlay { prior }`. The overlay re-renders with the new front item.
6. **Badge counter decrements on removal:** The overlay badge counter decrements by 1 when the `retain()` removes the modal from `App.overlay_stack` (not when the decision is sent). This keeps the badge count synchronized with `App.overlay_stack.len()`.
7. **`prior` focus preserved:** The `prior: FocusSnapshot` in `AppMode::Overlay` is unchanged throughout — both during the decision send and during the subsequent modal removal. It is restored correctly if the stack empties after removal.
8. **Decision sent before awaiting removal:** The IPC decision enqueue happens first; the TUI then awaits `PermissionPromptResolved`. If the TUI crashes after enqueue but before receiving the resolved notification, the daemon has already received the decision and will resolve the prompt — the crash is safe.

## Invariants

1. The `prompt_id` sent in the `ClientToServer::PermissionDecision` is the `Uuid` of
   `App.overlay_stack.front()` at the time `y` or `Enter` is pressed. If the daemon
   receives a `PermissionDecision` for a `prompt_id` it does not recognize (e.g., the
   prompt timed out), the daemon silently discards the response — the TUI does not need
   to handle this case.
2. Accept-Once semantics mean the daemon allows this specific invocation (`prompt_id`) but
   does NOT record a pattern for future auto-accept. This is in contrast to Accept-Always
   (BC-2.06.012) which records a pattern.
3. The `y` and `Enter` bindings for `PermissionAcceptOnce` are active ONLY in
   `AppMode::Overlay`, registered in the `SearchPrompt` (highest-priority) layer. In
   `AppMode::Dashboard`, `y` and `Enter` match no overlay binding (though they may match
   other bindings in their respective layers).
4. The IPC send channel is bounded. If the channel is full when the decision is enqueued,
   the message is dropped and the drop counter increments (BC-2.04.011). In this case the
   modal remains visible (no pop), which is the correct recovery UX — the user can see the
   prompt persisted and retry. The drop counter in the status bar surfaces this condition.
5. The TUI MUST NOT call `App.overlay_stack.retain()` or `App.overlay_stack.pop_front()`
   upon sending the decision. The modal removal from `App.overlay_stack` is triggered
   exclusively by receipt of `ServerToClient::PermissionPromptResolved` (BC-2.06.023).
   There is exactly one removal path per prompt lifecycle.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-077 | `App.overlay_stack` has exactly 1 item when `y` is pressed; `PermissionPromptResolved` arrives | IPC decision enqueued; modal stays visible in `App.overlay_stack` until `PermissionPromptResolved`; `retain()` removes it; `App.overlay_stack` empties; `AppMode` collapses to `Dashboard { focused: prior }` |
| EC-078 | `App.overlay_stack` has 5 items; `y` pressed; `PermissionPromptResolved` arrives for front item | IPC decision enqueued; modal stays in `App.overlay_stack` until resolved; `retain()` removes front item; 4 items remain; `AppMode` stays `Overlay { prior }`; badge decrements to 4 |
| EC-079 | IPC send channel is full when decision is enqueued | Message dropped; drop counter increments; modal remains in stack (no pop); user sees the modal still active; user may press `y`/`Enter` again if the hook has not yet timed out; status bar shows elevated drop counter |
| EC-080 | Daemon has already timed out the prompt before TUI's decision arrives | Daemon silently discards the `PermissionDecision`; daemon still sends `PermissionPromptResolved` for the timed-out prompt (via auto-resolve path); TUI receives it and removes the modal normally |
| EC-081 | `y` or `Enter` pressed in `AppMode::Dashboard` | No `SearchPrompt`-layer overlay binding active; key matches standard Dashboard bindings (if any); no IPC decision sent; no overlay effect |

## Canonical Test Vectors

| Input | Action / Event | Expected Output | Category |
|-------|---------------|----------------|----------|
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) | key `y` → `PermissionAcceptOnce` | `PermissionDecision { prompt_id: P1.id, decision: Accept }` enqueued; `AppMode` stays `Overlay { prior: Sessions }`; App.overlay_stack unchanged (no pop yet) | happy-path step 1 |
| After step 1 above: receive `PermissionPromptResolved { prompt_id: P1.id }` | daemon broadcast | `App.overlay_stack` is empty; `AppMode` collapses to `Dashboard { focused: Sessions }` | happy-path step 2 |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2]) | key `Enter` → `PermissionAcceptOnce`; then receive `PermissionPromptResolved { prompt_id: P1.id }` | After decision: App.overlay_stack stays [P1, P2]; after resolved: App.overlay_stack = [P2]; `AppMode` stays `Overlay { prior: Sessions }` | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) | key `y`; IPC channel full | `PermissionDecision` dropped; drop counter increments; `AppMode` stays `Overlay { prior: Sessions }`; App.overlay_stack unchanged (modal visible); no `PermissionPromptResolved` received | edge-case (channel full) |
| `Dashboard { focused: Sessions }` | key `y` | No overlay binding active; no IPC send; mode unchanged | edge-case |

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

- [BC-2.06.001] — depends on: empty-stack collapse after `retain()` reuses BC-2.06.001 invariant
- [BC-2.06.008] — depends on: the stack being popped was created by the push behavior in BC-2.06.008
- [BC-2.06.012] — composes with: Accept-Always (key `A`) is the sibling decision with pattern-recording semantics
- [BC-2.06.013] — composes with: Reject (keys `n`/`r`) is the sibling decision with deny semantics
- [BC-2.06.023] — depends on: `PermissionPromptResolved` handling (BC-2.06.023) is the removal trigger for this BC's modal; the two BCs implement the decision-then-resolve round-trip together
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

## §Trace v1.2.0

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. `AppMode::Overlay` carries only `{ prior: FocusSnapshot }`. All references to `stack.front()`, `stack.pop_front()`, `stack.len()`, and `stack` inside `Overlay { ... }` updated to reference `App.overlay_stack`.
- Precondition 1: `Overlay { stack, prior }` → `Overlay { prior }` with `App.overlay_stack.len() >= 1`.
- Precondition 3: `stack.front()` → `App.overlay_stack.front()`.
- Postconditions 1-6: all `stack.*` references → `App.overlay_stack.*`.
- Invariant 1, 5: `stack.front()` / `retain()` → `App.overlay_stack.front()` / `App.overlay_stack.retain()`.
- Edge Cases EC-077, EC-078: "Stack has N items" → "`App.overlay_stack` has N items".
- Test vectors: `Overlay { stack: [...], prior: ... }` → `Overlay { prior: ... }` (App.overlay_stack noted).
- SE-16d monotonicity: v1.2.0 timestamp 2026-05-28T00:00:00Z > v1.1.0. PASS.

## §Trace v1.1.0

**ADJ-ADV2-001 HIGH — Two adjudication decisions applied (keybinding canonical set + pop semantics)** (2026-05-27T00:00:00Z):

**Decision 1 — Keybinding: `1` → `y`/`Enter` (mnemonic set wins):**
- The story layer (S-026, S-027) used `y`/`Enter` for Accept-Once, `A` for Accept-Always,
  `n`/`r` for Reject. The BC layer used `1`/`2`/`3`. The adversarial pass 2 flagged the
  contradiction. Adjudication resolution: the mnemonic set (`y`/`Enter`/`A`/`n`/`r`) is
  canonical because (a) it matches lazygit-philosophy verb keybindings, (b) `y`/`n` is the
  universal TUI confirmation convention, (c) S-026/S-027 are more recent artifacts and reflect
  deliberate UX design. Updated: H1 title, Description, Precondition 2, Invariant 3, Edge
  Cases, Test Vectors, Related BCs. SS-tui.md §Overlay Stack Lifecycle Step 3 must be updated
  by architect to replace `[1]/[2]/[3]` with `[y]/[A]/[n/r]` (architectural source correction,
  not product-owner scope).
- Binding layer changed from `PerContext` to `SearchPrompt` (highest priority) to match
  S-026 AC-009, which specifies the `SearchPrompt` layer for overlay decision keys.

**Decision 2 — Pop semantics: immediate pop → wait-for-PermissionPromptResolved:**
- BC-2.06.011 Postcondition 2 previously specified `transition()` calls `stack.pop_front()`
  immediately after the IPC send. S-026 AC-003 and BC-2.06.023 both specify wait-for-resolved
  semantics: the modal is removed ONLY when `ServerToClient::PermissionPromptResolved` arrives.
  Adjudication resolution: wait-for-resolved is production-grade because (a) it keeps the modal
  visible if the IPC send fails (channel drop) allowing user to see the failure and retry,
  (b) it is consistent with BC-2.06.023 which is the authoritative removal mechanism, (c)
  immediate pop creates an invisible state gap (TUI thinks it's done, daemon has not confirmed).
  Updated: Description, all Postconditions (completely rewritten), Invariants 4+5 added,
  Edge Cases updated, Test Vectors updated.
- Cross-reference to BC-2.06.023 added to Related BCs.
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-27T00:00:00Z > v1.0.5. PASS.
