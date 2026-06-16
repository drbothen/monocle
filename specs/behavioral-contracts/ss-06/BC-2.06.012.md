---
document_type: behavioral-contract
level: L3
version: "1.2.1"
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
modified: [F-P1D2-010, F-P1D7-001, ADJ-ADV2-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.012: Permission Overlay: Accept-Always Keybinding (`A`)

## Description

In `AppMode::Overlay`, pressing `A` (uppercase; bound to `Action::PermissionAcceptAlways`
in the `SearchPrompt` binding layer) sends an accept-always decision to the daemon via
`ClientToServer::PermissionDecision { prompt_id, decision: PermissionDecision::AcceptAlways }`
over `App::ipc_tx`. The TUI does NOT immediately pop the `PromptModal`; it waits for
`ServerToClient::PermissionPromptResolved { prompt_id }` before removing the modal from
the `VecDeque` stack (per BC-2.06.023). The daemon forwards `{"decision": "always"}` to
the stalled Claude Code HTTP response, records the tool-pattern for future auto-accept,
then broadcasts `PermissionPromptResolved`. Accept-Always differs from Accept-Once
(BC-2.06.011) solely in the decision value sent; the TUI-side wait-for-resolved and
stack management behavior are identical.

## Preconditions

1. `AppMode` is `Overlay { prior }` and `App.overlay_stack.len() >= 1`.
2. The `SearchPrompt` binding layer (highest priority) maps key `A` (uppercase) to
   `Action::PermissionAcceptAlways` when `AppMode` is `Overlay`. This binding is active
   only in Overlay mode and overrides any Global or PerContext bindings for `A`.
3. `App.overlay_stack.front()` is the `PromptModal` whose `prompt_id` will be sent in the
   `ClientToServer::PermissionDecision`.
4. The IPC send channel (`App::ipc_tx`) has capacity for at least one additional message.

## Postconditions

1. **IPC send enqueued:** `ClientToServer::PermissionDecision { prompt_id: App.overlay_stack.front().prompt_id, decision: PermissionDecision::AcceptAlways }` is enqueued on `App::ipc_tx`. This is non-blocking.
2. **Modal NOT immediately popped:** The TUI does NOT call `App.overlay_stack.pop_front()` or `App.overlay_stack.retain()` upon sending the decision. The `PromptModal` remains in `App.overlay_stack` until `ServerToClient::PermissionPromptResolved { prompt_id }` is received. This ensures the modal stays visible if the IPC send fails (channel drop).
3. **PermissionPromptResolved triggers removal:** When `ServerToClient::PermissionPromptResolved { prompt_id }` is received, the TUI calls `App.overlay_stack.retain(|m| m.prompt_id != prompt_id)` to remove the modal (per BC-2.06.023).
4. **Stack-empty collapse after removal:** If `App.overlay_stack.is_empty()` after the `retain()` call and `AppMode` is `Overlay`, the TUI collapses to `AppMode::Dashboard { focused: prior }`.
5. **Stack-non-empty continuation after removal:** If `App.overlay_stack.len() >= 1` after the `retain()` call, `AppMode` remains `Overlay { prior }` with the new front item rendered.
6. **Badge counter decrements on removal:** The overlay badge counter decrements by 1 when the `retain()` removes the modal from `App.overlay_stack` (not when the decision is sent).
7. **`prior` focus preserved:** The `prior: FocusSnapshot` is unchanged throughout — both during the decision send and during subsequent modal removal.
8. **Pattern recording is daemon-side:** The TUI sends `PermissionDecision::AcceptAlways` in the IPC message. The daemon is solely responsible for recording the pattern (tool + path + session). The TUI does not maintain any pattern state.

## Invariants

1. `PermissionDecision::AcceptAlways` carries the same `prompt_id` as `PermissionDecision::Accept` —
   the distinction lies solely in the `decision` field value. The TUI-side wait-for-resolved
   and stack removal logic is identical for both decisions (BC-2.06.023 handles both).
2. Pattern recording by the daemon is out of scope for this BC. This BC specifies only the
   TUI-side behavior (IPC send + wait for `PermissionPromptResolved` + modal removal).
3. The `A` (uppercase) binding is active ONLY in `AppMode::Overlay` (registered in the
   `SearchPrompt` layer). Pressing `A` in `AppMode::Dashboard` or `AppMode::Filtering`
   matches no overlay binding.
4. The IPC send channel is bounded. If the channel is full, the message is dropped and the
   drop counter increments (BC-2.04.011). The modal remains visible (no pop), which is the
   correct recovery UX.
5. The TUI MUST NOT call `App.overlay_stack.retain()` or `App.overlay_stack.pop_front()`
   upon sending the decision. Modal removal from `App.overlay_stack` is triggered
   exclusively by `ServerToClient::PermissionPromptResolved` (BC-2.06.023).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-082 | `App.overlay_stack` has exactly 1 item when `A` is pressed; `PermissionPromptResolved` arrives | IPC `PermissionDecision::AcceptAlways` enqueued; modal stays in `App.overlay_stack` until `PermissionPromptResolved`; `retain()` removes it; `App.overlay_stack` empties; `AppMode` collapses to `Dashboard { focused: prior }` |
| EC-083 | `App.overlay_stack` has 3 items when `A` pressed; `PermissionPromptResolved` arrives for front item | IPC decision enqueued; modal stays in `App.overlay_stack`; on resolved: `retain()` removes front item; 2 items remain; `AppMode` stays `Overlay { prior }`; badge decrements |
| EC-084 | Daemon receives `PermissionDecision::AcceptAlways` but the pattern-recording logic fails internally | Daemon-side failure; daemon still sends `PermissionPromptResolved` (session unblocked); TUI removes modal normally; no TUI error |
| EC-085 | `A` pressed in `AppMode::Dashboard` | No `SearchPrompt`-layer overlay binding active; no IPC decision sent; no overlay effect |
| EC-086 | Two concurrent prompts for the same tool+path; user accepts-always the first | Daemon records the pattern; second prompt in the TUI stack may or may not auto-resolve before the user acts on it — this is a daemon-side race condition outside TUI scope |

## Canonical Test Vectors

| Input | Action / Event | Expected Output | Category |
|-------|---------------|----------------|----------|
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) | key `A` → `PermissionAcceptAlways` | `PermissionDecision { prompt_id: P1.id, decision: AcceptAlways }` enqueued; `AppMode` stays `Overlay { prior: Sessions }`; App.overlay_stack unchanged (no pop yet) | happy-path step 1 |
| After step 1 above: receive `PermissionPromptResolved { prompt_id: P1.id }` | daemon broadcast | `App.overlay_stack` is empty; `AppMode` collapses to `Dashboard { focused: Sessions }` | happy-path step 2 |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2]) | key `A`; then receive `PermissionPromptResolved { prompt_id: P1.id }` | After decision: App.overlay_stack stays [P1, P2]; after resolved: App.overlay_stack = [P2]; `AppMode` stays `Overlay { prior: Sessions }` | happy-path |
| `Dashboard { focused: Sessions }` | key `A` | No overlay binding active; no IPC send; mode unchanged | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | IPC `ClientToServer::PermissionDecision` with `decision: PermissionDecision::AcceptAlways` and correct `prompt_id` is enqueued before state transition | integration test |
| VP-TBD | Empty stack after accept-always collapses `AppMode` to `Dashboard` | unit test |
| VP-TBD | `prior` FocusSnapshot restored correctly on stack-empty collapse | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the Accept-Always decision path within the "permission overlay stack" component of CAP-006, enabling the user to grant persistent per-tool permission to avoid repeated prompts |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: Accept-Always sends an IPC decision message; pattern recording is daemon-side; TUI writes no files) |
| Architecture Module | monocle-core (transition() PermissionAcceptAlways arm); monocle-tui (App::handle_action enqueues IPC message) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Permission Overlay §Overlay Stack Lifecycle step 3 (Decide); §AppMode State Machine §Transition Function Contract (PermissionAcceptAlways arm) |
| Cross-Ref | BC-2.06.011 (Accept-Once — sibling with identical TUI behavior, different decision value), BC-2.06.013 (Reject — sibling decision), BC-2.06.001 (pure transition function), BC-2.06.008 (overlay push), BC-2.04.011 (bounded event bus drop counter) |
| Test File | `monocle-tui/tests/overlay_decisions.rs` |
| Test Name | `test_BC_2_06_012_accept_always_pops_front_and_sends_ipc` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: empty-stack collapse after `retain()` reuses BC-2.06.001 invariant
- [BC-2.06.011] — composes with: Accept-Once (keys `y`/`Enter`) is the sibling with one-time semantics
- [BC-2.06.013] — composes with: Reject (keys `n`/`r`) is the sibling with deny semantics
- [BC-2.06.008] — depends on: the stack being modified was created by the push behavior in BC-2.06.008
- [BC-2.06.023] — depends on: `PermissionPromptResolved` handling (BC-2.06.023) is the removal trigger; the two BCs implement the decision-then-resolve round-trip together
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
- Reads: SS-tui.md v1.1.0 §Permission Overlay §Overlay Stack Lifecycle step 3,
  §AppMode State Machine §Transition Function Contract; prd-expansion-scope.md §3.3
  BC-2.06.012 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: TUI sends IPC message only; pattern recording is daemon-side.
- Postcondition 7 explicitly scopes pattern recording to the daemon to enforce the
  TUI-is-a-client invariant (SS-tui.md §Architectural Principle: Observe-Only).


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
- `Decision::AcceptAlways` → `PermissionDecision::AcceptAlways`. The variant name is correct
  (`AcceptAlways`) but it belongs to the `PermissionDecision` enum (not a bare `Decision` enum).
- All occurrences updated: Description, Postcondition 1, Postcondition 7, Invariant 1, EC-082,
  EC-084, test vectors, VP table.
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

## §Trace v1.2.1

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): already propagated in §Trace v1.2.0 below.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers Accept-Always decision keybinding only; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
- SE-16d monotonicity: v1.2.1 timestamp 2026-05-29T00:00:00Z > v1.2.0. PASS.

## §Trace v1.2.0

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. Symmetric with BC-2.06.011 v1.2.0 sweep. All `stack.front()` / `stack.pop_front()` / `stack.len()` / `overlay_stack.retain()` references updated to reference `App.overlay_stack`. `Overlay { stack, prior }` → `Overlay { prior }` throughout.
- SE-16d monotonicity: v1.2.0 timestamp 2026-05-28T00:00:00Z > v1.1.0. PASS.

## §Trace v1.1.0

**ADJ-ADV2-001 HIGH — Two adjudication decisions applied (keybinding canonical set + pop semantics)** (2026-05-27T00:00:00Z):

**Decision 1 — Keybinding: `2` → `A` (mnemonic set wins):**
- Symmetric with BC-2.06.011 v1.1.0. The `A` (uppercase, "Accept Always") keybinding is
  the canonical Accept-Always key per S-026 AC-004 and S-027 AC-001 footer. Updated: H1
  title, Description, Precondition 2, Invariant 3, Edge Cases, Test Vectors, Related BCs.
  Binding layer changed from `PerContext` to `SearchPrompt` (highest priority) to match
  S-026 AC-009.

**Decision 2 — Pop semantics: immediate pop → wait-for-PermissionPromptResolved:**
- Symmetric with BC-2.06.011 v1.1.0. Postconditions completely rewritten to specify wait-
  for-resolved semantics. Invariant 5 added (no-pop obligation). Edge Cases updated.
  Test Vectors updated to show two-step round-trip (decision send → resolved arrival).
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-27T00:00:00Z > v1.0.5. PASS.
