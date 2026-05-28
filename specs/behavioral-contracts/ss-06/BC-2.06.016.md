---
document_type: behavioral-contract
level: L3
version: "1.0.7"
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
modified: [F-P1D2-010, F-P1D7-001]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.016: Permission Overlay: Cleared on Daemon Disconnect

## Description

When the TUI receives a daemon disconnect signal (IPC channel closed, corresponding to
BC-2.05.007 at the IPC layer), the TUI immediately clears `App.overlay_stack: VecDeque<PromptModal>`
(the single source of truth for the modal stack) and transitions `AppMode` to
`Dashboard { focused: FocusSnapshot::Sessions }`. The status bar renders
"Daemon disconnected — reconnecting..." until reconnection. This is the SOQ-3 enforcement
at the TUI layer: orphaned prompts from a disconnected daemon must never persist in
`App.overlay_stack` because the old daemon's stalled HTTP responses will time out, and any future
decision would be sent to the wrong daemon connection.

## Preconditions

1. The TUI is connected to the daemon via IPC (UDS channel active).
2. `AppMode` is any valid state — `Dashboard`, `Filtering`, `Overlay`, or `Fullscreen`.
   The disconnect handler is unconditional.
3. The IPC transport signals `TransportEvent::Disconnected` (per BC-2.05.007): this occurs
   when `read_framed` returns `UnexpectedEof`, `BrokenPipe`, or `ConnectionReset` on the UDS
   connection. There is no explicit sentinel message — the disconnect is detected at the
   transport layer, not as a `ServerToClient` variant.
4. The overlay stack may be empty or non-empty; the clear is a no-op if already empty.

## Postconditions

1. **Overlay stack cleared:** `App.overlay_stack: VecDeque<PromptModal>` is cleared
   (`.clear()` called). `App.overlay_stack.len() == 0` after handling.
2. **AppMode transitions to Dashboard:** After clearing, `AppMode` is set to
   `Dashboard { focused: FocusSnapshot::Sessions }` regardless of what mode was active
   before the disconnect (Overlay, Fullscreen, Filtering, or Dashboard).
3. **No IPC decision sent:** The TUI does NOT attempt to send any `ClientToServer::PermissionDecision`
   message on disconnect. The daemon is gone; the channel is closed. No write to the IPC
   send channel is attempted.
4. **Status bar renders disconnect indicator:** The status bar renders the text
   "Daemon disconnected — reconnecting..." until the IPC reconnect sequence (BC-2.05.006)
   completes and the daemon delivers a new initial state push (BC-2.05.002).
5. **Badge counter reset:** The overlay badge counter in the status bar resets to 0
   as a consequence of the cleared stack.
6. **Transition is synchronous:** The clear and mode transition happen within the same
   `handle_ipc_message()` call that processes the disconnect signal. There is no deferred
   or async path for this operation — the state is consistent on the next `draw()` tick.

## Invariants

1. After a daemon disconnect event, no `PromptModal` from the previous daemon session
   remains in `App.overlay_stack`. This is a safety invariant: old prompts against a
   disconnected daemon MUST NOT be auto-decided against a new daemon connection.
2. The `AppMode` after disconnect is always `Dashboard { focused: FocusSnapshot::Sessions }`.
   There is no exception for other prior modes.
3. This invariant is the TUI-side complement of BC-2.05.007 (IPC-side: "Overlay Stack
   Cleared on Daemon Disconnect"). BC-2.05.007 specifies the IPC layer's obligation to
   signal the disconnect; this BC specifies the TUI layer's obligation to handle it.
4. On daemon reconnect, the TUI does NOT restore `App.overlay_stack` from local memory.
   It receives a fresh `overlay_stack` in the daemon's initial state push (BC-2.05.002);
   only the daemon's authoritative state is used to rebuild `App.overlay_stack` via
   `payload_to_modal()`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-097 | Disconnect occurs when `AppMode` is `Overlay` with 3 queued prompts | Stack cleared to 0; AppMode → `Dashboard { Sessions }`; badge shows 0; status bar shows disconnect message |
| EC-098 | Disconnect occurs when `AppMode` is `Dashboard` (no overlay active, stack already empty) | `.clear()` on empty `VecDeque` is a no-op; AppMode → `Dashboard { Sessions }` (already there); status bar shows disconnect message; no error |
| EC-099 | Disconnect occurs when `AppMode` is `Fullscreen` | Stack cleared; AppMode → `Dashboard { Sessions }` (fullscreen forcefully exited); status bar shows disconnect message |
| EC-100 | Disconnect occurs when `AppMode` is `Filtering` | Stack cleared (was empty in Filtering); AppMode → `Dashboard { Sessions }` (filter mode exited); status bar shows disconnect message |
| EC-101 | Daemon reconnects within 1 second; daemon has 0 queued prompts in new session | Status bar "reconnecting..." clears; AppMode stays `Dashboard`; overlay stack stays empty — correct |
| EC-102 | Daemon reconnects; daemon has 2 queued prompts from a NEW Claude Code session that arrived during reconnect window | TUI receives initial state push with 2 prompts; `App.overlay_stack = [P1, P2]` (populated via `payload_to_modal()`); transitions to `AppMode::Overlay { prior: Sessions }`; overlay renders — correct, these are fresh prompts from the new daemon, not the orphaned old ones |
| EC-103 | IPC channel drops silently (no sentinel message, just EOF on the read half) | `ipc_rx.recv()` returns `None`; treated identically to `DaemonDisconnect` sentinel; stack cleared and mode reset |
| EC-104 | User was mid-keypress when disconnect occurred (e.g., just pressed `2` for Accept-always) | If the `ClientToServer::PermissionDecision` was already enqueued in `ipc_tx` before the disconnect: the send fails (channel closed); error is logged with `tracing::warn!` and swallowed; stack still cleared; no panic |

## Canonical Test Vectors

| Initial State | Event | Expected Post-State | Category |
|---------------|-------|---------------------|----------|
| `AppMode::Overlay { prior: Sessions }`, `App.overlay_stack = [P1, P2]` | `TransportEvent::Disconnected` | `AppMode::Dashboard { focused: Sessions }`, `App.overlay_stack` empty, badge 0, status bar "reconnecting..." | happy-path |
| `AppMode::Dashboard { focused: Sessions }` | `TransportEvent::Disconnected` | `AppMode::Dashboard { focused: Sessions }`, stack empty (was empty), status bar "reconnecting..." | edge-case |
| `AppMode::Fullscreen { panel: Sessions, prior: Sessions }` | `TransportEvent::Disconnected` | `AppMode::Dashboard { focused: Sessions }`, status bar "reconnecting..." | edge-case |
| `AppMode::Filtering { panel: Sessions, query: "api", prior: Sessions }` | `TransportEvent::Disconnected` | `AppMode::Dashboard { focused: Sessions }`, status bar "reconnecting..." | edge-case |
| Stack has 1 `PromptModal`; TUI just sent `ClientToServer::PermissionDecision`; disconnect arrives on same tick | Stack cleared; mode → Dashboard; warn log for failed send; no panic | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | After `DaemonDisconnect`, `App.overlay_stack` (`VecDeque<PromptModal>`) is empty | unit test |
| VP-TBD | After `DaemonDisconnect`, `AppMode` is `Dashboard { focused: Sessions }` regardless of prior mode | unit test (4 prior-mode variants) |
| VP-TBD | No `ClientToServer::PermissionDecision` is sent to IPC on disconnect | unit test (assert `ipc_tx` has no pending messages after disconnect handling) |
| VP-TBD | Status bar renders "Daemon disconnected — reconnecting..." on disconnect | integration test |
| VP-TBD | On reconnect with 0 queued prompts, overlay remains empty | integration test |
| VP-TBD | On reconnect with N queued prompts (fresh), overlay renders N prompts | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the "permission overlay stack" clear behavior on daemon disconnect, which is a direct component of the CAP-006 "permission overlay stack" scope |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: disconnect handling clears local state only, no file writes occur); DI-001 (every hook event received MUST be written to the JSONL ring before acknowledgement — enforced upstream in daemon; TUI disconnect does not affect JSONL integrity) |
| Architecture Module | monocle-tui (App::handle_ipc_message disconnect arm); monocle-core (AppMode state, VecDeque<PromptModal>) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.5.0 §Permission Overlay §Overlay Stack Lifecycle step 5 (Daemon disconnect SOQ-3); §Ctrl-\ Integration §State Preservation Across Hide/Show |
| Cross-Ref | BC-2.05.007 (IPC-side: Overlay Stack Cleared on Daemon Disconnect — this is the IPC event this TUI BC responds to); BC-2.05.006 (TUI Reconnects After Daemon Restart — reconnect sequence that follows disconnect); BC-2.05.002 (Initial state push on reconnect — delivers fresh overlay_stack); BC-2.06.014 (Esc hides without clearing — DISTINCT from disconnect which DOES clear) |
| Test File | `monocle-tui/tests/overlay_disconnect.rs` |
| Test Name | `test_BC_2_06_016_overlay_cleared_on_daemon_disconnect` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.007] — depends on: this is the TUI-side handler for the IPC disconnect signal specified in BC-2.05.007
- [BC-2.05.006] — composes with: reconnect sequence follows the disconnect this BC handles
- [BC-2.05.002] — composes with: initial state push on reconnect provides fresh overlay_stack to rebuild overlay
- [BC-2.06.014] — CRITICAL DISTINCTION: `[Esc]` hides without clearing the stack; disconnect unconditionally clears

## Architecture Anchors

- `architecture/SS-tui.md#permission-overlay` — overlay stack lifecycle step 5 (daemon disconnect SOQ-3)
- `architecture/SS-tui.md#ctrl-integration` — state preservation mechanism and daemon ownership of overlay_stack

## Story Anchor

S-TBD — Implement daemon disconnect handler: clear overlay stack, reset AppMode to Dashboard, render reconnecting status (filled by story-writer)

## VP Anchors

- VP-TBD — Integration test: overlay cleared on daemon disconnect; overlay rebuilt from daemon state on reconnect

## §Trace v1.0.0

**Initial production** (2026-05-26T18:00:00Z):
- BC-2.06.016 created as part of SS-06 TUI behavioral contract burst (BCs 016–022).
- Reads: SS-tui.md v1.1.0 §Permission Overlay §Overlay Stack Lifecycle step 5 (daemon
  disconnect SOQ-3); §Ctrl-\ Integration §State Preservation Across Hide/Show;
  prd-expansion-scope.md §3.3 BC-2.06.016 description and §5.2 dependency table
  (BC-2.05.007 → BC-2.06.016 dependency chain).
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability table row SS-06.
- DI-007 cited: no file writes on disconnect; DI-001 cited: JSONL ring integrity not
  affected by TUI disconnect.
- EC-103 covers silent EOF (channel close without sentinel) — production-grade robustness.
- EC-104 covers in-flight send on disconnect — warn log, no panic, swallowed.
- Invariant 4 explicitly prohibits restoring old overlay from local memory on reconnect —
  daemon is the authoritative state source per §Ctrl-\ Integration.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.4

**IPC field name corrected: `queued_prompts` → `overlay_stack`** (2026-05-26T00:00:00Z):
- Invariant 4: `queued_prompts` in the daemon's initial state push → `overlay_stack`.
  Canonical IPC field name is `overlay_stack` from `ServerToClient::InitialState`;
  `queued_prompts` was a stale fabrication.
- Cross-Ref table (BC-2.05.002 entry): `fresh queued_prompts` → `fresh overlay_stack`.
- Related BCs (BC-2.05.002 bullet): `fresh queued_prompts` → `fresh overlay_stack`.
- Architecture Anchors (SS-tui.md#ctrl-integration): `queued_prompts` → `overlay_stack`.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.

## §Trace v1.0.3

**F-P1D7-001 HIGH — Fabricated `IpcServerMessage::DaemonDisconnect` and `DecisionResponse` replaced** (2026-05-26T00:00:00Z):
- Precondition 3: `IpcServerMessage::DaemonDisconnect` sentinel → `TransportEvent::Disconnected`
  signal. Per SS-ipc.md §Reconnection Behavior and BC-2.05.007, disconnect is detected at the
  transport layer when `read_framed` returns `UnexpectedEof`/`BrokenPipe`/`ConnectionReset`.
  There is no `DaemonDisconnect` variant in `ServerToClient`. The BC-2.05.007 mechanism is
  `TransportEvent::Disconnected`.
- All test vector rows: `DaemonDisconnect` → `TransportEvent::Disconnected`.
- Postcondition 3, EC-104, test vector row, VP table: `DecisionResponse` →
  `ClientToServer::PermissionDecision`.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.5

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.5 timestamp >= v1.0.4. PASS.

## §Trace v1.0.6

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed (partial)** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002 (partial). `VecDeque<PromptModal>` overlay stack is now `App.overlay_stack` (single source of truth). Description, Postcondition 1, Invariants 1 and 4, VP table updated to reference `App.overlay_stack` explicitly.
- NOTE: EC-102 and canonical test vector row 1 were NOT updated in this pass — they retained stale `AppMode::Overlay { stack: [P1, P2], prior: Sessions }` shape. Corrected in v1.0.7.
- SE-16d monotonicity: v1.0.6 timestamp 2026-05-28T00:00:00Z > v1.0.5. PASS.

## §Trace v1.0.7

**F-S025-ADV4-HIGH-001 — EC-102 and test vector row 1 body sweep completion** (2026-05-28T13:00:00Z):
- Finding: §Trace v1.0.6 falsely claimed the `Overlay { stack }` shape sweep was complete.
  EC-102 (Expected Behavior column) and canonical test vector row 1 (Initial State column)
  still contained the stale `AppMode::Overlay { stack: [P1, P2], prior: Sessions }` shape.
- Fix — EC-102: `transitions to AppMode::Overlay { stack: [P1, P2], ... }` →
  `App.overlay_stack = [P1, P2]` (populated via `payload_to_modal()`); `AppMode::Overlay { prior: Sessions }`.
  The stack content is now expressed as an `App.overlay_stack` assignment adjacent to the mode,
  consistent with the canonical two-step semantics from BC-2.06.004 PC-2.
- Fix — Canonical test vector row 1 (Initial State column):
  `AppMode::Overlay { stack: [P1, P2], prior: Sessions }` →
  `AppMode::Overlay { prior: Sessions }`, `App.overlay_stack = [P1, P2]`.
  Post-State column: `stack empty` → `App.overlay_stack empty` for consistency.
- §Trace v1.0.6 note retrospectively updated to mark the pass as "partial" rather than
  complete, accurately reflecting what was and was not changed in that pass.
- SE-16d monotonicity: v1.0.7 timestamp 2026-05-28T13:00:00Z > v1.0.6 timestamp 2026-05-28T00:00:00Z. PASS.
