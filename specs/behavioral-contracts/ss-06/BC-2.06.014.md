---
document_type: behavioral-contract
level: L3
version: "1.0.8"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T14:00:00Z
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

# Behavioral Contract BC-2.06.014: Permission Overlay: `[Esc]` Hides Without Rejecting

## Description

In `AppMode::Overlay`, pressing `[Esc]` (bound to `Action::Escape` in the `PerContext`
binding table for `Overlay`) is a no-op on both `AppMode` and `App.overlay_stack`. The
`transition()` function returns the current mode unchanged: `(Overlay { prior }, Escape)`
→ `Overlay { prior }` (identity). No mutation to `App.overlay_stack` occurs. The overlay
is hidden when the user
presses `Ctrl-\` at the tmux level, which spawns a new `monocle` process on the next open.
Prompts remain queued in the daemon's pending-prompt registry (`overlay_stack`). On the next `Ctrl-\`,
the new TUI process receives the full current overlay stack in the initial state push
(BC-2.05.002) and transitions to `AppMode::Overlay` if any prompts are queued. This
behavior is the SOQ-3 complement: queued prompts survive the hide/show cycle without
being answered or dropped.

## Preconditions

1. `AppMode` is `Overlay { prior }` and `App.overlay_stack.len() >= 1`.
2. The `PerContext` binding table for `AppMode::Overlay` maps `[Esc]` to `Action::Escape`.
3. The daemon's pending-prompt registry (`overlay_stack` in the IPC `InitialState` push) holds the same prompts as the TUI's local
   `App.overlay_stack` (they are synchronized; the daemon is the durable store per BC-2.04 / SS-tui.md
   §Ctrl-\ Integration).

## Postconditions

1. **No stack modification:** `transition(Overlay { prior }, Escape)` returns
   `Overlay { prior }` verbatim. `App.overlay_stack` is not touched; `App.overlay_stack.len()` is
   unchanged. No `PromptModal` is popped, pushed, or modified.
2. **No IPC message sent:** The TUI does NOT send any `ClientToServer::PermissionDecision`, `ClearOverlay`,
   or other IPC message to the daemon on `[Esc]`. The daemon's pending-prompt registry (`overlay_stack`) is
   unchanged.
3. **Overlay popup hidden by tmux layer:** The hide action (`Ctrl-\`) is managed at the
   tmux level, external to the TUI process. When the user presses `Ctrl-\`, tmux closes
   the popup window. The TUI process exits (or is suspended). The `AppMode` at exit is
   `Overlay` — the process terminates with an active overlay, and that is correct.
4. **Prompts survive hide/show via daemon ownership:** The daemon holds the pending-prompt registry (`overlay_stack` in IPC).
   When the user opens `Ctrl-\` again, `tmux display-popup`
   spawns a NEW `monocle` process. The new TUI process receives the current
   `overlay_stack` in the daemon's initial state push (BC-2.05.002), populates
   `App.overlay_stack` via `payload_to_modal()`, and transitions to
   `AppMode::Overlay { prior: FocusSnapshot::Sessions }`.
5. **Overlay badge preserved on next show:** The next TUI instance receives the same
   number of queued prompts from the daemon. The badge counter in the status bar reflects
   the current `stack.len()` on render, which matches the daemon's queue.
6. **In-process behavior of `[Esc]`:** If the user is in `AppMode::Overlay` and presses
   `[Esc]` without subsequently dismissing the popup via `Ctrl-\`, the current process
   remains alive with `AppMode::Overlay` unchanged. The overlay continues to be rendered.
   `[Esc]` in overlay mode does nothing visible — it is specifically a no-op to avoid
   accidental dismiss-without-decision. The keybinding hint line shows `Esc: hide` to
   communicate that `Ctrl-\` is the hide mechanism, not `Esc`.

## Invariants

1. `[Esc]` in `AppMode::Overlay` is strictly a no-op: zero `App.overlay_stack` changes, zero IPC sends,
   zero mode transitions. This is enforced by the `transition()` identity arm:
   `(mode @ AppMode::Overlay { .. }, Action::Escape) => mode`.
2. This invariant is the SOQ-3 complement. SOQ-3 states that overlay prompts must survive
   the `Ctrl-\` hide/show cycle. The mechanism is: (a) daemon owns `overlay_stack` (IPC); (b)
   `[Esc]` sends no decision and does not touch `App.overlay_stack`; (c) each new TUI process loads the queue from the daemon.
3. `[Esc]` in `AppMode::Filtering` has DIFFERENT behavior: it clears the filter and
   returns to `Dashboard { focused: prior }` (per BC-2.06.006). The `[Esc]` no-op behavior
   is specific to `AppMode::Overlay`.
4. `[Esc]` in `AppMode::Fullscreen` returns to `Dashboard { focused: prior }` (per
   BC-2.06.007). Again, the no-op is specific to `AppMode::Overlay`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-092 | User presses `[Esc]` multiple times in `AppMode::Overlay` | Each press is a no-op; stack unchanged; no error; no visual change in the overlay (except the hint line may flash if there is a keypress indicator) |
| EC-093 | User presses `[Esc]` then immediately presses `[n/r]` (Reject) | `[Esc]` is a no-op; `[n/r]` sends Reject IPC; modal remains until `PermissionPromptResolved` arrives from daemon; correct behavior |
| EC-094 | New `PermissionPromptQueued` IPC message arrives while the user is in `Overlay` AppMode (process still alive, not hidden) | The new `PromptModal` is pushed to the back of `stack`; if already `Overlay`, the existing `prior` is preserved; overlay badge increments; no `[Esc]` involvement |
| EC-095 | User hides popup (`Ctrl-\`) then daemon disconnects while popup is hidden | On next `Ctrl-\`, new TUI process connects to daemon; if daemon is down, TUI renders "Daemon disconnected — reconnecting..." (BC-2.06.016 path); no overlay shown because there is no daemon to push queued prompts |
| EC-096 | `stack.len() == 0` when `[Esc]` is dispatched (impossible in valid state) | `transition()` identity arm returns `Overlay { prior }`; the empty-stack invariant is enforced at App-level (`App.overlay_stack` remains empty) — the empty-stack collapse (BC-2.06.001) guarantees this state cannot be reached via normal decision paths; if it occurs in testing, the no-op arm returns the impossible state unchanged |

## Canonical Test Vectors

| Input (mode, action) | Expected Output | Category |
|----------------------|----------------|----------|
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]), `Escape` | `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) — no change | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2]), `Escape` | `Overlay { prior: Sessions }` (App.overlay_stack = [P1, P2]) — no change | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]), `Escape` × 5 | `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) — still no change | edge-case |
| `Filtering { panel: Sessions, query: "foo", prior: Sessions }`, `Escape` | `Dashboard { focused: Sessions }` — Esc clears filter (different behavior than Overlay Esc) | edge-case |
| `Fullscreen { panel: Sessions, prior: Sessions }`, `Escape` | `Dashboard { focused: Sessions }` — Esc exits fullscreen (different behavior than Overlay Esc) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `Action::Escape` in `AppMode::Overlay` never modifies `App.overlay_stack` | unit test (assert `App.overlay_stack.len()` unchanged after Esc) |
| VP-TBD | `Action::Escape` in `AppMode::Overlay` never sends IPC message | unit test (assert `ipc_tx` empty after Esc in Overlay) |
| VP-TBD | Overlay stack is preserved across TUI process restart (daemon-ownership mechanism) | integration test (spawn TUI, open overlay, kill TUI process, spawn new TUI, assert overlay stack received from daemon) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the `[Esc]` hide-without-decision behavior that is the SOQ-3 complement for the "permission overlay stack" and "Ctrl-\ popup integration" components of CAP-006 |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: `[Esc]` sends no IPC message and writes no files) |
| Architecture Module | monocle-core (transition() Overlay+Escape identity arm); monocle-tui (daemon-ownership of pending-prompt registry via IPC initial state push `overlay_stack`) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §AppMode State Machine §Transition Function Contract (Overlay+Escape no-op arm and SOQ-3 comment); §Permission Overlay §Overlay Stack Lifecycle step 4 (Hide); §Ctrl-\ Integration §State Preservation Across Hide/Show |
| Cross-Ref | BC-2.06.013 (Reject — CRITICAL DISTINCTION: `[n/r]` sends deny IPC and waits for `PermissionPromptResolved`; `[Esc]` does neither), BC-2.05.002 (TUI initial state push — mechanism by which overlay survives TUI process restart), BC-2.06.001 (pure transition function — Esc identity arm), BC-2.06.016 (overlay cleared on daemon disconnect — different from hide) |
| Test File | `monocle-core/tests/app_mode_transitions.rs` |
| Test Name | `test_BC_2_06_014_esc_in_overlay_is_noop` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.06.001] — depends on: `[Esc]` no-op is the identity arm in the `transition()` pure function defined in BC-2.06.001
- [BC-2.06.013] — CRITICAL DISTINCTION: Reject (keys `[n/r]`) sends deny IPC and waits for `PermissionPromptResolved`; `[Esc]` is a no-op; these must never be conflated
- [BC-2.05.002] — depends on: TUI initial state push is the mechanism by which queued prompts survive the `Ctrl-\` hide/show cycle
- [BC-2.06.016] — composes with: daemon disconnect clears the overlay (different from hide); the `[Esc]` no-op applies only when the daemon is still connected

## Architecture Anchors

- `architecture/SS-tui.md#appmode-state-machine` — transition function Overlay+Escape no-op arm with SOQ-3 comment
- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle step 4 (Hide) and SOQ-3 rationale
- `architecture/SS-tui.md#ctrl-integration` — state preservation across hide/show via daemon ownership

## Story Anchor

S-TBD — Implement Esc no-op behavior in Overlay mode and verify overlay survives Ctrl-\ cycle (filled by story-writer)

## VP Anchors

- VP-TBD — Integration test: overlay stack preserved across TUI process restart via daemon ownership

## §Trace v1.0.0

**Initial production** (2026-05-26T14:00:00Z):
- BC-2.06.014 created as part of SS-06 TUI behavioral contract burst (BCs 009–015).
- Reads: SS-tui.md v1.1.0 §AppMode State Machine (Overlay+Escape identity arm and SOQ-3
  comment), §Permission Overlay §Overlay Stack Lifecycle step 4, §Ctrl-\ Integration
  §State Preservation Across Hide/Show; prd-expansion-scope.md §3.3 BC-2.06.014 description.
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: `[Esc]` sends no IPC message and writes no files.
- Invariant 3 and 4 explicitly document that `[Esc]` has DIFFERENT semantics in Filtering
  and Fullscreen modes — the no-op is Overlay-specific.
- Postcondition 3 and 4 document the daemon-ownership mechanism: hiding is a tmux-level
  operation; the TUI process exits; prompts survive via daemon's pending-prompt registry (`overlay_stack`).


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**IPC sweep — fabricated `DecisionResponse` in prohibition list replaced** (2026-05-26T14:30:00Z):
- Postcondition 2: "The TUI does NOT send any `DecisionResponse`, `ClearOverlay`..." →
  "The TUI does NOT send any `ClientToServer::PermissionDecision`, `ClearOverlay`...".
  The prohibition list explicitly named the IPC message type; updated to canonical name
  per SS-ipc.md §Client-to-Server Messages.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.5

**Stale keybinding + pop-semantics fix in EC-093 and cross-references** (2026-05-27T00:00:00Z):
- EC-093: `[3]` → `[n/r]` (Reject keybinding updated per BC-2.06.013 v1.1.0).
- EC-093: "sends Reject + pops the front item" → "sends Reject IPC; modal remains until `PermissionPromptResolved` arrives from daemon" (per BC-2.06.023 wait-for-resolved semantics).
- Related BCs entry for BC-2.06.013: "Reject (key `3`) pops and sends deny" → "Reject (keys `[n/r]`) sends deny IPC and waits for `PermissionPromptResolved`".
- Traceability Cross-Ref: same correction applied to the BC-2.06.013 parenthetical.
- SE-16d monotonicity: v1.0.5 timestamp >= v1.0.4. PASS.

## §Trace v1.0.7

**F-S025-ADV5-HIGH-002 — EC-096 Overlay shape corrected; prior §Trace v1.0.6 sweep was incomplete** (2026-05-28T14:00:00Z):
- EC-096 Expected Behavior: `Overlay { stack: empty, prior }` → `Overlay { prior }`; added explicit App-level note: "`App.overlay_stack` remains empty" as an adjacent clarification.
- §Trace v1.0.6 claimed "Preconditions, Postconditions, Invariants, and test vectors updated throughout" but EC-096 in the Edge Cases table was not updated in that sweep. This entry honestly documents the missed row.
- No other stale `Overlay { stack` references found in live content (§Trace v1.0.6 historical text at lines 203–204 is historiography and retained verbatim).
- SE-16d monotonicity: v1.0.7 timestamp 2026-05-28T14:00:00Z > v1.0.6 2026-05-28T00:00:00Z. PASS.

## §Trace v1.0.6

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. Description: `(Overlay { stack, prior }, Escape)` → `(Overlay { prior }, Escape)` identity; `App.overlay_stack` explicitly noted as unmodified. Preconditions, Postconditions, Invariants, and test vectors updated throughout.
- Postcondition 4: rebuild path now explicitly states populate `App.overlay_stack` via `payload_to_modal()` then transition to `Overlay { prior: FocusSnapshot::Sessions }`.
- VP: "never modifies `stack`" → "never modifies `App.overlay_stack`".
- SE-16d monotonicity: v1.0.6 timestamp 2026-05-28T00:00:00Z > v1.0.5. PASS.

## §Trace v1.0.4

**F-FINAL-001 MEDIUM — Daemon-side `queued_prompts`/`DaemonState::queued_prompts` replaced with canonical IPC field name** (2026-05-26T00:00:00Z):
- Description: `DaemonState::queued_prompts` → `pending-prompt registry (overlay_stack)`.
- Precondition 3: `DaemonState::queued_prompts` → `overlay_stack (IPC InitialState push)`.
- Postcondition 2: `queued_prompts` → `overlay_stack`; clarified TUI builds `VecDeque<PromptModal>` from it.
- Postcondition 4: `DaemonState::queued_prompts` → pending-prompt registry / `overlay_stack`.
- Invariant 2: `daemon owns queued_prompts` → `daemon owns overlay_stack (IPC)`.
- Architecture Module: `queued_prompts` → `overlay_stack`.
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per final bulk pin update.
- §Trace v1.0.0 historical note: `queued_prompts` → `overlay_stack`.
- The TUI-side `VecDeque<PromptModal>` references throughout Postconditions/Invariants are RETAINED — the TUI
  local stack type is `VecDeque<PromptModal>`; only the IPC/daemon naming was wrong.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.0.8

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): already propagated in §Trace v1.0.6/v1.0.7 above.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers Esc hide behavior; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): this BC is about Esc-hides-without-clearing, which is the CONTRAST to BC-2.06.016's disconnect behavior; no disconnect rendering text in scope.
- SE-16d monotonicity: v1.0.8 timestamp 2026-05-29T00:00:00Z > v1.0.7. PASS.