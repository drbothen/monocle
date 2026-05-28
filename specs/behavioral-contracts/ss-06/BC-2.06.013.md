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

# Behavioral Contract BC-2.06.013: Permission Overlay: Reject Keybinding (`n`/`r`)

## Description

In `AppMode::Overlay`, pressing `n` or `r` (both bound to `Action::PermissionReject` in
the `SearchPrompt` binding layer) sends a deny decision to the daemon via
`ClientToServer::PermissionDecision { prompt_id, decision: PermissionDecision::Reject }`
over `App::ipc_tx`. The TUI does NOT immediately pop the `PromptModal`; it waits for
`ServerToClient::PermissionPromptResolved { prompt_id }` before removing the modal (per
BC-2.06.023). The daemon forwards `{"decision": "deny"}` to the stalled Claude Code HTTP
response; Claude Code receives the deny and does not execute the tool, then the daemon
broadcasts `PermissionPromptResolved`. The TUI-side wait-for-resolved and stack removal
behavior is identical to Accept-Once (BC-2.06.011) and Accept-Always (BC-2.06.012),
except the decision value is `Reject`.

## Preconditions

1. `AppMode` is `Overlay { prior }` and `App.overlay_stack.len() >= 1`.
2. The `SearchPrompt` binding layer (highest priority) maps keys `n` and `r` to
   `Action::PermissionReject` when `AppMode` is `Overlay`. These bindings are active
   only in Overlay mode and override any Global or PerContext bindings for these keys.
3. `App.overlay_stack.front()` is the `PromptModal` whose `prompt_id` will be sent in the
   `ClientToServer::PermissionDecision`.
4. The IPC send channel (`App::ipc_tx`) has capacity for at least one additional message.

## Postconditions

1. **IPC send enqueued:** `ClientToServer::PermissionDecision { prompt_id: App.overlay_stack.front().prompt_id, decision: PermissionDecision::Reject }` is enqueued on `App::ipc_tx`. This is non-blocking.
2. **Modal NOT immediately popped:** The TUI does NOT call `App.overlay_stack.pop_front()` or `App.overlay_stack.retain()` upon sending the decision. The `PromptModal` remains in `App.overlay_stack` until `ServerToClient::PermissionPromptResolved { prompt_id }` is received.
3. **PermissionPromptResolved triggers removal:** When `ServerToClient::PermissionPromptResolved { prompt_id }` is received, the TUI calls `App.overlay_stack.retain(|m| m.prompt_id != prompt_id)` to remove the modal (per BC-2.06.023).
4. **Stack-empty collapse after removal:** If `App.overlay_stack.is_empty()` after the `retain()` call and `AppMode` is `Overlay`, the TUI collapses to `AppMode::Dashboard { focused: prior }`.
5. **Stack-non-empty continuation after removal:** If `App.overlay_stack.len() >= 1` after the `retain()` call, `AppMode` remains `Overlay { prior }` with the new front item rendered.
6. **Badge counter decrements on removal:** The overlay badge counter decrements by 1 when the `retain()` removes the modal from `App.overlay_stack`.
7. **`prior` focus preserved:** The `prior: FocusSnapshot` is unchanged throughout.
8. **Deny is not the same as `Esc` hide:** `n`/`r` (Reject) enqueues a deny decision and waits for `PermissionPromptResolved` to remove the modal. `Esc` (BC-2.06.014) does NOT send any IPC message and does NOT remove the modal. This distinction is CRITICAL: `Esc` must never be confused with reject by the implementation.

## Invariants

1. `PermissionDecision::Reject` carries the same `prompt_id` as `PermissionDecision::Accept` and
   `PermissionDecision::AcceptAlways`. The TUI-side wait-for-resolved and stack removal logic
   is identical for all three decision types (BC-2.06.023 handles all three).
2. The `n` and `r` bindings are active ONLY in `AppMode::Overlay`, registered in the
   `SearchPrompt` (highest-priority) layer. In `AppMode::Dashboard` or `AppMode::Filtering`,
   `n` and `r` match no overlay binding.
3. The IPC send channel is bounded. If full, the message is dropped and the drop counter
   increments (BC-2.04.011). The modal remains visible (no pop) — correct recovery UX.
4. Deny semantics (whether Claude Code retries the tool, aborts the task, or surfaces the
   rejection to the user) are governed by Claude Code's behavior, not by monocle. The
   monocle TUI sends `{"decision": "deny"}` and is done.
5. The TUI MUST NOT call `App.overlay_stack.retain()` or `App.overlay_stack.pop_front()`
   upon sending the decision. Modal removal from `App.overlay_stack` is triggered
   exclusively by `ServerToClient::PermissionPromptResolved` (BC-2.06.023).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-087 | `App.overlay_stack` has exactly 1 item when `n` pressed; `PermissionPromptResolved` arrives | IPC `PermissionDecision::Reject` enqueued; modal stays in `App.overlay_stack` until `PermissionPromptResolved`; `retain()` removes it; `App.overlay_stack` empties; `AppMode` collapses to `Dashboard { focused: prior }` |
| EC-088 | `App.overlay_stack` has 4 items when `r` pressed; `PermissionPromptResolved` arrives for front item | IPC decision enqueued; modal stays in `App.overlay_stack`; on resolved: `retain()` removes front item; 3 items remain; `AppMode` stays `Overlay { prior }`; badge decrements |
| EC-089 | IPC send channel is full when Reject is enqueued | Message dropped; drop counter increments; modal remains visible (no pop); hook will eventually time out at daemon with fail-open semantics (BC-HOOK-001); user sees the persistent modal and can retry `n`/`r` |
| EC-090 | `n` or `r` pressed in `AppMode::Dashboard` | No `SearchPrompt`-layer overlay binding active; no IPC decision sent; no overlay effect |
| EC-091 | User presses `Esc` intending to hide — `n`/`r` is a distinct key | `n`/`r` sends Reject and waits for `PermissionPromptResolved`; `Esc` only hides (no IPC, no removal); these are distinct keys — terminal-mapping issues are out of scope for this BC |

## Canonical Test Vectors

| Input | Action / Event | Expected Output | Category |
|-------|---------------|----------------|----------|
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) | key `n` → `PermissionReject` | `PermissionDecision { prompt_id: P1.id, decision: Reject }` enqueued; `AppMode` stays `Overlay { prior: Sessions }`; App.overlay_stack unchanged (no pop yet) | happy-path step 1 |
| After step 1 above: receive `PermissionPromptResolved { prompt_id: P1.id }` | daemon broadcast | `App.overlay_stack` is empty; `AppMode` collapses to `Dashboard { focused: Sessions }` | happy-path step 2 |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2]) | key `r`; then receive `PermissionPromptResolved { prompt_id: P1.id }` | After decision: App.overlay_stack stays [P1, P2]; after resolved: App.overlay_stack = [P2]; `AppMode` stays `Overlay { prior: Sessions }` | happy-path |
| `Dashboard { focused: Sessions }` | key `n` | No overlay binding active; no IPC send; mode unchanged | edge-case |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) | key `Esc` | `AppMode` stays `Overlay { prior: Sessions }`; App.overlay_stack unchanged — Esc does NOT send IPC, does NOT remove modal | edge-case (Esc-vs-Reject distinction) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | IPC `ClientToServer::PermissionDecision` with `decision: PermissionDecision::Reject` and correct `prompt_id` is enqueued before state transition | integration test |
| VP-TBD | `Action::Escape` in `Overlay` does NOT call `stack.pop_front()` | unit test (confirm Esc-vs-Reject distinction) |
| VP-TBD | Empty stack after reject collapses `AppMode` to `Dashboard` | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the Reject (deny) decision path within the "permission overlay stack" component of CAP-006, which is the user's mechanism to prevent a Claude Code tool from executing |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: Reject sends an IPC decision message; the TUI writes no files) |
| Architecture Module | monocle-core (transition() PermissionReject arm); monocle-tui (App::handle_action enqueues IPC message) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.5.0 §Permission Overlay §Overlay Stack Lifecycle step 3 (Decide); §AppMode State Machine §Transition Function Contract (PermissionReject arm) |
| Cross-Ref | BC-2.06.011 (Accept-Once — sibling), BC-2.06.012 (Accept-Always — sibling), BC-2.06.014 (Esc Hide — CRITICAL distinction: Esc does NOT pop), BC-2.06.001 (pure transition function), BC-2.04.011 (bounded event bus) |
| Test File | `monocle-tui/tests/overlay_decisions.rs` |
| Test Name | `test_BC_2_06_013_reject_pops_front_and_sends_deny_ipc` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: empty-stack collapse after `retain()` reuses BC-2.06.001 invariant
- [BC-2.06.011] — composes with: Accept-Once (keys `y`/`Enter`) is the sibling with accept semantics
- [BC-2.06.012] — composes with: Accept-Always (key `A`) is the sibling with always-accept semantics
- [BC-2.06.014] — CRITICAL DISTINCTION: Esc (BC-2.06.014) hides without sending IPC or removing modal; Reject sends deny and waits for `PermissionPromptResolved`; these must never be conflated in implementation
- [BC-2.06.008] — depends on: the stack being modified was created by the push behavior in BC-2.06.008
- [BC-2.06.023] — depends on: `PermissionPromptResolved` handling (BC-2.06.023) is the removal trigger; the two BCs implement the decision-then-resolve round-trip together

## Architecture Anchors

- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle step 3 (Decide, Reject arm)
- `architecture/SS-tui.md#appmode-state-machine` — transition function PermissionReject arm
- `architecture/SS-tui.md#status-bar` — keybinding hint line: `[n/r] Reject`

## Story Anchor

S-TBD — Implement Reject keybinding for permission overlay with IPC deny send (filled by story-writer)

## VP Anchors

- VP-TBD — Integration tests for Reject IPC send, state transition, and Esc-vs-Reject distinction

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.013 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.1.0 §Permission Overlay §Overlay Stack Lifecycle step 3,
  §AppMode State Machine §Transition Function Contract; prd-expansion-scope.md §3.3
  BC-2.06.013 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: TUI sends IPC message only; no file writes.
- Postcondition 7 and EC-091 + Related BCs explicitly flag the CRITICAL Esc-vs-Reject
  distinction to prevent implementation conflation.


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
- `Decision::Deny` → `PermissionDecision::Reject`. The canonical reject variant is `Reject`
  (wire value `"deny"`) per SS-ipc.md §PermissionDecision enum. The bare `Decision::Deny`
  enum does not exist.
- All occurrences updated: Description, Postcondition 1, Invariant 1, EC-087, test vectors, VP table.
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
- Resolves F-S025-ADV3-BLOCKER-002. Symmetric with BC-2.06.011 v1.2.0 and BC-2.06.012 v1.2.0 sweeps. All stack references updated to `App.overlay_stack`; `Overlay { stack, prior }` → `Overlay { prior }` throughout.
- SE-16d monotonicity: v1.2.0 timestamp 2026-05-28T00:00:00Z > v1.1.0. PASS.

## §Trace v1.1.0

**ADJ-ADV2-001 HIGH — Two adjudication decisions applied (keybinding canonical set + pop semantics)** (2026-05-27T00:00:00Z):

**Decision 1 — Keybinding: `3` → `n`/`r` (mnemonic set wins):**
- Symmetric with BC-2.06.011 v1.1.0. The `n` (no) and `r` (reject) keybindings are canonical
  per S-026 AC-005 and S-027 AC-001 footer `"[n/r] Reject"`. Updated: H1 title, Description,
  Precondition 2, Invariant 2, Edge Cases, Test Vectors, Related BCs, Architecture Anchors
  (status bar hint updated from `3: reject` to `[n/r] Reject`).
  Binding layer changed from `PerContext` to `SearchPrompt` (highest priority).

**Decision 2 — Pop semantics: immediate pop → wait-for-PermissionPromptResolved:**
- Symmetric with BC-2.06.011 v1.1.0. Postconditions completely rewritten. Invariant 5 added.
  Edge Cases updated. Test Vectors updated.
- The Esc-vs-Reject distinction (Postcondition 8, EC-091) remains CRITICAL and is now framed
  in terms of wait-for-resolved: Esc sends no IPC and does no retain(); Reject sends deny IPC
  and waits for PermissionPromptResolved before retain(). The distinction is sharper than before.
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-27T00:00:00Z > v1.0.5. PASS.
