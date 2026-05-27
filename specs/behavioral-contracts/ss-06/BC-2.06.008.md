---
document_type: behavioral-contract
level: L3
version: "1.0.4"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T12:00:00Z
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

# Behavioral Contract BC-2.06.008: Permission Overlay: VecDeque Stack Push on PermissionPromptQueued

## Description

When the TUI receives a `PermissionPromptQueued` IPC message from the daemon, it constructs
a `PromptModal` from the message payload and pushes it to the back of the
`VecDeque<PromptModal>` in the current overlay stack. If `AppMode` is `Dashboard` or
`Filtering`, the TUI transitions to `AppMode::Overlay { stack: [new_prompt], prior: <current_focus> }`.
If `AppMode` is already `Overlay`, the new `PromptModal` is appended to the existing stack's
`VecDeque` without changing the `prior` focus. This is the entry point for the permission
overlay system and the TUI-side counterpart of BC-2.05.005.

## Preconditions

1. The TUI's IPC receive channel (`app.ipc_rx`) delivers `ServerToClient::PermissionPromptQueued`
   messages from the daemon.
2. `PromptModal` is defined in `monocle-core/src/prompt_modal.rs` with fields:
   - `prompt_id: Uuid` (stable ID from daemon for decision correlation)
   - `session_id: String`
   - `tool_name: String`
   - `tool_payload: ToolPayload` (Edit, Bash, Read, or Generic variants)
   - `received_at: std::time::Instant`
3. The `handle_ipc_message()` method on `App` is called on each draining of `ipc_rx` in
   the draw loop (before rendering, per the draw loop architecture in SS-tui.md).
4. `AppMode` is one of: `Dashboard`, `Filtering`, or `Overlay` when the message arrives.
   (`Fullscreen` is not listed; see Invariant 3.)

## Postconditions

1. **`PromptModal` constructed from message:** The `PermissionPromptQueued` message is
   deserialized and mapped to a `PromptModal` with:
   - `prompt_id` from message.
   - `session_id` from message.
   - `tool_name` from message.
   - `tool_payload` constructed from message payload fields (Edit fields → `ToolPayload::Edit`,
     Bash command → `ToolPayload::Bash`, Read path → `ToolPayload::Read`, other → `ToolPayload::Generic`).
   - `received_at: Instant::now()` (set at message-handling time, not IPC receive time).
2. **Push to back of VecDeque:** The `PromptModal` is pushed to the back of the overlay
   stack via `VecDeque::push_back`. The front of the queue is always the next prompt to be
   displayed and decided upon.
3. **Transition from `Dashboard` or `Filtering`:** If `AppMode` at message receipt is
   `Dashboard { focused }` or `Filtering { prior, .. }`, the TUI transitions to:
   `AppMode::Overlay { stack: VecDeque::from([new_prompt]), prior: <focused_or_prior> }`.
   In the `Filtering` case, `prior` is taken from the `Filtering::prior` field (the
   `FocusSnapshot` captured before filter mode was entered).
4. **Extend existing `Overlay` stack:** If `AppMode` at message receipt is already
   `Overlay { mut stack, prior }`, the new `PromptModal` is pushed to the back of `stack`.
   The `prior` field is NOT changed — the original focus context (from when the overlay was
   first opened) is preserved through the entire overlay lifetime.
5. **Overlay badge counter increments:** After each push, the status bar overlay badge
   counter (count of items in `stack`) is updated. On the next draw tick, the status bar
   renders `[N prompts]` in the breadcrumb (e.g., "Dashboard > Overlay [2 prompts]").
6. **Rendering on next tick:** The push path does not call `draw()` directly. The updated
   `AppMode` is in place before the next draw tick (≤16ms), which renders the overlay with
   the new front prompt.
7. **Non-blocking IPC drain:** The push is performed in the synchronous `handle_ipc_message()`
   call inside the draw loop's drain phase. No `await` is used in the push path. The IPC
   `ipc_rx` channel is a bounded `mpsc::Receiver`; if the channel is full, the newest IPC
   message is dropped and the drop counter increments (per BC-2.06.019 and BC-2.04.011).

## Invariants

1. **`VecDeque` is never empty in `Overlay` state:** This invariant is guaranteed by
   BC-2.06.001 Postcondition 3. The push path always adds at least one item to the VecDeque
   when constructing `AppMode::Overlay`. The transition path pops the front on decision and
   collapses to `Dashboard` if the result is empty — so a zero-item `Overlay` is unreachable.
2. **`prior` is set once per Overlay lifetime:** The `prior: FocusSnapshot` field is
   captured when the `Overlay` variant is first constructed (Postcondition 3). It is not
   updated by subsequent pushes (Postcondition 4). This ensures `Escape` from `Overlay`
   or the final decision always restores to the panel that had focus before the first
   prompt arrived.
3. **`Fullscreen` mode handling:** If `AppMode` is `Fullscreen` when `PermissionPromptQueued`
   arrives, the TUI pushes the prompt and transitions to `AppMode::Overlay { stack: [new_prompt], prior: Fullscreen::prior }`.
   The Fullscreen view is abandoned in favor of the permission overlay. The `prior` field
   carries the `FocusSnapshot` from the Fullscreen's `prior` (which in turn came from the
   Dashboard focus before Fullscreen was entered), ensuring focus is restored correctly.
4. **`PromptModal::received_at` is set at TUI handling time, not at IPC wire time.** This
   means `received_at` measures the time from daemon send to TUI consumption. It is used for
   display purposes (queue age) and is not used for hook timeout decisions (the daemon
   independently tracks hook timeouts).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-100 | Two `PermissionPromptQueued` messages arrive in the same draw tick drain phase | Both are processed sequentially in the drain loop; both prompts are pushed to the VecDeque; `AppMode::Overlay { stack: [P1, P2], .. }` after the drain; next draw renders overlay with P1 in front |
| EC-101 | `PermissionPromptQueued` arrives when `AppMode` is `Overlay` with 5 items already | Sixth item pushed to back; `AppMode::Overlay { stack: [P1..P6], prior }` — no upper bound on VecDeque size (bounded by system memory, not by the TUI logic) |
| EC-102 | `PermissionPromptQueued` arrives when `AppMode` is `Fullscreen` | Fullscreen abandoned; transitions to `Overlay { stack: [new_prompt], prior: Fullscreen::prior }` (see Invariant 3) |
| EC-103 | `PermissionPromptQueued` deserialization fails (malformed IPC message) | TUI logs error via `tracing::error!` and discards the message; no `PromptModal` is pushed; `AppMode` unchanged |
| EC-104 | `PermissionPromptQueued` arrives during Filtering mode | `Overlay { stack: [new_prompt], prior: Filtering::prior }` — Filtering is abandoned; the FocusSnapshot from before filter mode was entered is used as `prior` |
| EC-105 | `PermissionPromptQueued` arrives after daemon disconnect clear (BC-2.06.016) has cleared the stack | This is impossible during normal operation: after disconnect, the IPC channel is closed and no new messages arrive. A reconnection triggers a fresh initial state push, not individual `PermissionPromptQueued` messages for old prompts |

## Canonical Test Vectors

| Initial AppMode | IPC Message | Expected AppMode | Category |
|-----------------|-------------|-----------------|----------|
| `Dashboard { focused: Sessions }` | `PermissionPromptQueued { prompt_id: P1, tool: Edit, .. }` | `Overlay { stack: [P1], prior: Sessions }` | happy-path |
| `Overlay { stack: [P1], prior: Sessions }` | `PermissionPromptQueued { prompt_id: P2, tool: Bash, .. }` | `Overlay { stack: [P1, P2], prior: Sessions }` (prior unchanged) | happy-path |
| `Filtering { panel: Sessions, query: "foo", prior: Sessions }` | `PermissionPromptQueued { prompt_id: P1, .. }` | `Overlay { stack: [P1], prior: Sessions }` | edge-case |
| `Fullscreen { panel: Sessions, prior: Sessions }` | `PermissionPromptQueued { prompt_id: P1, .. }` | `Overlay { stack: [P1], prior: Sessions }` | edge-case |
| `Overlay { stack: [P1, P2], prior: EventRibbon }` | `PermissionPromptQueued { prompt_id: P3, .. }` | `Overlay { stack: [P1, P2, P3], prior: EventRibbon }` (prior still EventRibbon) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Push from `Dashboard` produces `Overlay { stack: [P1], prior: <captured focus> }` | Integration test (mock IPC channel) |
| VP-TBD | Push from `Overlay` extends existing stack without changing `prior` | Integration test (mock IPC channel) |
| VP-TBD | Push from `Fullscreen` transitions to Overlay with Fullscreen's prior | Integration test (mock IPC channel) |
| VP-TBD | Malformed IPC message causes error log and no panic | Unit test (error injection) |
| VP-TBD | Two simultaneous pushes in one drain cycle produce stack of 2 | Integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the "permission overlay stack" component of CAP-006: the VecDeque push on PermissionPromptQueued is the entry point for the permission overlay system and the product's primary competitive differentiator (D-2 in the PRD: simultaneous multi-session permission handling without prompt drop) |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — overlay push path performs no file writes; it only modifies in-memory `app.mode` and logs via tracing) |
| Architecture Module | monocle-tui (App::handle_ipc_message(), draw loop IPC drain phase) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.5.0 §Permission Overlay §Overlay Stack Lifecycle (Push section, step 1); §PromptModal Type; §Rendering Architecture (draw loop drain phase) |
| Cross-Ref | BC-2.05.005 (PermissionPromptQueued IPC message — the daemon-side precondition for this BC), BC-2.06.001 (Overlay AppMode variant and VecDeque non-empty invariant), BC-2.06.009 (stack rotation, the next operation after push), BC-2.06.011..013 (decision actions that pop from the stack pushed here) |
| Test File | `monocle-tui/tests/permission_overlay_push.rs` |
| Test Name | `test_BC_2_06_008_overlay_vecdeque_push_on_permission_prompt_queued` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.005] — depends on: `PermissionPromptQueued` is the IPC message type that triggers this push (daemon side)
- [BC-2.06.001] — composes with: `Overlay` AppMode variant is defined by BC-2.06.001; BC-2.06.001 Invariant 4 documents this push path as the explicit exception to "all AppMode changes go through transition()"
- [BC-2.06.009] — composes with: stack rotation operates on the VecDeque populated by this BC
- [BC-2.06.011] — composes with: PermissionAcceptOnce pops the front item pushed here
- [BC-2.06.012] — composes with: PermissionAcceptAlways pops the front item pushed here
- [BC-2.06.013] — composes with: PermissionReject pops the front item pushed here
- [BC-2.06.016] — contrasts with: daemon-disconnect clears the VecDeque populated here

## Architecture Anchors

- `architecture/SS-tui.md#permission-overlay` — PromptModal type definition
- `architecture/SS-tui.md#permission-overlay` — Overlay Stack Lifecycle §Push step 1 (full push algorithm)
- `architecture/SS-tui.md#rendering-architecture` — Draw loop drain phase (where handle_ipc_message is called)

## Story Anchor

S-TBD — Implement App::handle_ipc_message() for PermissionPromptQueued; VecDeque push logic; AppMode transition from Dashboard/Filtering/Fullscreen → Overlay (filled by story-writer)

## VP Anchors

- VP-TBD — Integration tests for all push-path AppMode transitions; mock IPC channel injecting PermissionPromptQueued messages

## §Trace v1.0.0

**Initial production** (2026-05-26T12:00:00Z):
- BC-2.06.008 created as part of SS-06 TUI behavioral contract burst (BCs 001–008).
- Reads: SS-tui.md v1.1.0 §Permission Overlay (PromptModal type, Overlay Stack Lifecycle
  §Push, §Rotate, §Decide, §Hide, §Daemon disconnect); prd-expansion-scope.md §3.3
  BC-2.06.008 description; ARCH-INDEX.md §Capability Traceability SS-06.
- Invariant 3 (Fullscreen handling) is a design decision made here and not explicitly in
  SS-tui.md. SS-tui.md §Overlay Stack Lifecycle §Push step 1 says "If the current AppMode
  is Dashboard or Filtering" transitions to Overlay. Fullscreen is not listed as an
  explicit case. This BC closes that gap: Fullscreen is treated as Dashboard (the overlay
  takes over, and prior carries the Fullscreen's prior FocusSnapshot). This is the
  production-grade behavior — the alternative (silently dropping the prompt if in Fullscreen)
  would violate the "no prompt drop" competitive differentiator.
- EC-104 documents the Filtering → Overlay transition carefully: the Filtering::prior
  (not Filtering::panel) is used as the Overlay::prior. This is correct because Filtering
  carries the pre-filter FocusSnapshot in its own prior field.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.0.0` → `SS-tui.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-005 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.1.0` → `SS-tui.md v1.3.0` per F-P1D4-005 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-P1D7-001 HIGH — Fabricated `IpcServerMessage` replaced with canonical `ServerToClient`** (2026-05-26T00:00:00Z):
- All occurrences of `IpcServerMessage::PermissionPromptQueued` replaced with
  `ServerToClient::PermissionPromptQueued`. The canonical enum in SS-ipc.md §Server-to-Client
  Messages is `ServerToClient` (not `IpcServerMessage`).
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-tui.md v1.3.0` → `SS-tui.md v1.5.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.
