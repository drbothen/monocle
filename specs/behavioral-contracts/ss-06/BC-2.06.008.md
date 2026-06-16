---
document_type: behavioral-contract
level: L3
version: "1.1.1"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T00:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "c1e8267"
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
a `PromptModal` from the message payload and pushes it to the back of `App.overlay_stack:
VecDeque<PromptModal>` — the single source of truth for the modal stack. If `AppMode` is
`Dashboard` or `Filtering`, the TUI then transitions `AppMode` to
`AppMode::Overlay { prior: <current_focus> }`. If `AppMode` is already `Overlay`, the
`PromptModal` is appended to `App.overlay_stack` without changing the `AppMode::Overlay::prior`
field. Note: the modal stack lives in `App.overlay_stack` exclusively; the `Overlay` variant
carries only `{ prior: FocusSnapshot }`. This is the entry point for the permission
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
2. **Push to back of `App.overlay_stack`:** The `PromptModal` is pushed to the back of
   `App.overlay_stack` via `VecDeque::push_back`. The front of the queue is always the next
   prompt to be displayed and decided upon.
3. **Transition from `Dashboard` or `Filtering`:** If `AppMode` at message receipt is
   `Dashboard { focused }` or `Filtering { prior, .. }`, the TUI:
   (a) Pushes the new `PromptModal` to `App.overlay_stack`.
   (b) Transitions `AppMode` to `AppMode::Overlay { prior: <focused_or_prior> }`.
   In the `Filtering` case, `prior` is taken from the `Filtering::prior` field (the
   `FocusSnapshot` captured before filter mode was entered).
   The `Overlay` variant carries only `{ prior }` — the stack contents live in `App.overlay_stack`.
4. **Extend existing `App.overlay_stack`:** If `AppMode` at message receipt is already
   `Overlay { prior }`, the new `PromptModal` is pushed to the back of `App.overlay_stack`.
   The `AppMode::Overlay::prior` field is NOT changed — the original focus context (from
   when the overlay was first entered) is preserved through the entire overlay lifetime.
5. **Overlay badge counter increments:** After each push, the status bar overlay badge
   counter (count of items in `App.overlay_stack`) is updated. On the next draw tick, the
   status bar renders `[N prompts]` in the breadcrumb (e.g., "Dashboard > Overlay [2 prompts]").
6. **Rendering on next tick:** The push path does not call `draw()` directly. The updated
   `AppMode` is in place before the next draw tick (≤16ms), which renders the overlay with
   the new front prompt.
7. **Non-blocking IPC drain:** The push is performed in the synchronous `handle_ipc_message()`
   call inside the draw loop's drain phase. No `await` is used in the push path. The IPC
   `ipc_rx` channel is a bounded `mpsc::Receiver`; if the channel is full, the newest IPC
   message is dropped and the drop counter increments (per BC-2.06.019 and BC-2.04.011).

## Invariants

1. **`App.overlay_stack` is never empty while `AppMode` is `Overlay`:** The push path always
   adds at least one item to `App.overlay_stack` when transitioning to `AppMode::Overlay`.
   The App-level `retain()`-based removal collapses `AppMode` to `Dashboard` when
   `App.overlay_stack` empties — so `Overlay` with an empty `App.overlay_stack` is
   unreachable in steady state (per BC-2.06.001 Postcondition 3 as updated).
2. **`prior` is set once per Overlay lifetime:** The `prior: FocusSnapshot` in
   `AppMode::Overlay { prior }` is captured when the `Overlay` variant is first constructed
   (Postcondition 3). It is not updated by subsequent pushes (Postcondition 4). This ensures
   `Escape` from `Overlay` or the final decision always restores to the panel that had focus
   before the first prompt arrived.
3. **`Fullscreen` mode handling:** If `AppMode` is `Fullscreen` when `PermissionPromptQueued`
   arrives, the TUI pushes the prompt to `App.overlay_stack` and transitions to
   `AppMode::Overlay { prior: Fullscreen::prior }`. The Fullscreen view is abandoned in favor
   of the permission overlay. The `prior` field carries the `FocusSnapshot` from the
   Fullscreen's `prior` (which in turn came from the Dashboard focus before Fullscreen was
   entered), ensuring focus is restored correctly.
4. **`PromptModal::received_at` is set at TUI handling time, not at IPC wire time.** This
   means `received_at` measures the time from daemon send to TUI consumption. It is used for
   display purposes (queue age) and is not used for hook timeout decisions (the daemon
   independently tracks hook timeouts).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-100 | Two `PermissionPromptQueued` messages arrive in the same draw tick drain phase | Both are processed sequentially in the drain loop; both prompts pushed to `App.overlay_stack`; `AppMode::Overlay { prior }` (App.overlay_stack = [P1, P2]) after the drain; next draw renders overlay with P1 in front |
| EC-101 | `PermissionPromptQueued` arrives when `AppMode` is `Overlay` with 5 items already | Sixth item pushed to back of `App.overlay_stack`; `AppMode::Overlay { prior }` unchanged; App.overlay_stack now has 6 entries — no upper bound on VecDeque size |
| EC-102 | `PermissionPromptQueued` arrives when `AppMode` is `Fullscreen` | Fullscreen abandoned; new_prompt pushed to `App.overlay_stack`; `AppMode` transitions to `Overlay { prior: Fullscreen::prior }` (see Invariant 3) |
| EC-103 | `PermissionPromptQueued` deserialization fails (malformed IPC message) | TUI logs error via `tracing::error!` and discards the message; no `PromptModal` is pushed; `AppMode` unchanged |
| EC-104 | `PermissionPromptQueued` arrives during Filtering mode | new_prompt pushed to `App.overlay_stack`; Filtering abandoned; `AppMode` transitions to `Overlay { prior: Filtering::prior }` — the FocusSnapshot from before filter mode was entered is used as `prior` |
| EC-105 | `PermissionPromptQueued` arrives after daemon disconnect clear (BC-2.06.016) has cleared the stack | This is impossible during normal operation: after disconnect, the IPC channel is closed and no new messages arrive. A reconnection triggers a fresh initial state push, not individual `PermissionPromptQueued` messages for old prompts |

## Canonical Test Vectors

| Initial AppMode | IPC Message | Expected AppMode | Category |
|-----------------|-------------|-----------------|----------|
| `Dashboard { focused: Sessions }` | `PermissionPromptQueued { prompt_id: P1, tool: Edit, .. }` | App.overlay_stack = [P1]; `AppMode` → `Overlay { prior: Sessions }` | happy-path |
| `Overlay { prior: Sessions }` (App.overlay_stack = [P1]) | `PermissionPromptQueued { prompt_id: P2, tool: Bash, .. }` | App.overlay_stack = [P1, P2]; `AppMode` stays `Overlay { prior: Sessions }` (prior unchanged) | happy-path |
| `Filtering { panel: Sessions, query: "foo", prior: Sessions }` | `PermissionPromptQueued { prompt_id: P1, .. }` | App.overlay_stack = [P1]; `AppMode` → `Overlay { prior: Sessions }` | edge-case |
| `Fullscreen { panel: Sessions, prior: Sessions }` | `PermissionPromptQueued { prompt_id: P1, .. }` | App.overlay_stack = [P1]; `AppMode` → `Overlay { prior: Sessions }` | edge-case |
| `Overlay { prior: EventRibbon }` (App.overlay_stack = [P1, P2]) | `PermissionPromptQueued { prompt_id: P3, .. }` | App.overlay_stack = [P1, P2, P3]; `AppMode` stays `Overlay { prior: EventRibbon }` (prior still EventRibbon) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Push from `Dashboard` populates `App.overlay_stack = [P1]` and transitions `AppMode` to `Overlay { prior: <captured focus> }` | Integration test (mock IPC channel) |
| VP-TBD | Push from `Overlay` extends `App.overlay_stack` without changing `AppMode::Overlay::prior` | Integration test (mock IPC channel) |
| VP-TBD | Push from `Fullscreen` transitions to `Overlay { prior: Fullscreen::prior }` | Integration test (mock IPC channel) |
| VP-TBD | Malformed IPC message causes error log and no panic | Unit test (error injection) |
| VP-TBD | Two simultaneous pushes in one drain cycle produce stack of 2 | Integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability — this BC specifies the "permission overlay stack" component of CAP-006: the VecDeque push on PermissionPromptQueued is the entry point for the permission overlay system and the product's primary competitive differentiator (D-2 in the PRD: simultaneous multi-session permission handling without prompt drop) |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness — overlay push path performs no file writes; it only modifies in-memory `app.mode` and logs via tracing) |
| Architecture Module | monocle-tui (App::handle_ipc_message(), draw loop IPC drain phase) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Permission Overlay §Overlay Stack Lifecycle (Push section, step 1); §PromptModal Type; §Rendering Architecture (draw loop drain phase) |
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

## §Trace v1.1.1

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.5.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: `SS-tui.md v1.5.0` → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): already propagated in §Trace v1.1.0 below.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers Permission Overlay push only; no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
- SE-16d monotonicity: v1.1.1 timestamp 2026-05-29T00:00:00Z > v1.1.0. PASS.

## §Trace v1.1.0

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack: ... }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. This BC is the primary push-path entry point; non-mechanical rewrite required.
- Description: push now goes to `App.overlay_stack: VecDeque<PromptModal>` (single source of truth); `AppMode::Overlay` variant carries only `{ prior: FocusSnapshot }`. The two-step sequence (push to stack, then update AppMode) is explicit.
- Postconditions 2-4: all references to `stack` inside `Overlay { stack, prior }` replaced with `App.overlay_stack`. Postconditions 3-4 now describe the two-step App-level operation (push then AppMode update) rather than constructing `Overlay { stack: VecDeque::from([...]) }`.
- Invariants 1-3: reframed with `App.overlay_stack` as the container; `AppMode::Overlay { prior }` as the mode signal.
- EC-100, 101, 102, 104: all `Overlay { stack: [...] }` shapes replaced.
- Test vectors: all shapes updated.
- VP table: push assertions updated to reference `App.overlay_stack`.
- Note: `transition()` is NOT called for the push path — this BC's operation is an App-level effectful mutation (`App.overlay_stack.push_back` + `app.mode = Overlay { prior }`). This has always been the case (Invariant 4 of BC-2.06.001); the architect decision makes it structurally enforced.
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-28T00:00:00Z > v1.0.4. PASS.
