---
document_type: behavioral-contract
level: L3
version: "1.0.5"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-26T04:00:00Z
phase: phase-1-expansion
inputs: [prd-expansion-scope.md, architecture/SS-ipc.md, architecture/ARCH-INDEX.md]
input-hash: "73990b1"
traces_to: prd.md
origin: greenfield
subsystem: SS-05
capability: CAP-005
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

# Behavioral Contract BC-2.05.007: Overlay Stack Cleared on Daemon Disconnect (SOQ-3)

## Description

SOQ-3 is the safety invariant from the product brief (line 145): when the TUI loses its UDS
connection to the daemon unexpectedly, all entries in the `VecDeque<PromptModal>` overlay
stack are cleared before any reconnection attempt begins. The rationale is that Claude Code
subprocesses apply a 300ms timeout to PreToolUse hook responses; if the daemon restarts, the
pending decision channel is destroyed, and any `PromptModal` still visible in the TUI is
stale — approving it would send a decision to a nonexistent channel. SOQ-3 prevents ghost
approvals by clearing stale prompts at the transport-level disconnect signal.

The IPC layer enforces SOQ-3 at the `UdsTransport` level: when `read_framed` returns a
connection-loss error, `UdsTransport` emits `TransportEvent::Disconnected` before the error
propagates. The TUI event loop receives this signal and clears the `VecDeque<PromptModal>`
synchronously — before the reconnect loop begins. The TUI-layer response to this signal is
specified in BC-2.06.016.

## Preconditions

1. A TUI client is connected to the daemon's UDS socket and has at least one entry in its
   local `VecDeque<PromptModal>` overlay stack.
2. The `read_framed` call in the TUI's receive loop returns one of: `UnexpectedEof`,
   `BrokenPipe`, or `ConnectionReset`.
3. The TUI event loop has not yet begun any reconnection attempt.

## Postconditions

1. `UdsTransport` emits `TransportEvent::Disconnected` immediately upon detecting the
   connection-loss error, before returning the error to the caller or entering the reconnect
   loop.
2. The TUI event loop receives `TransportEvent::Disconnected` and calls the SOQ-3 handler,
   which clears all entries from the `VecDeque<PromptModal>`. The Vec is empty after the
   handler returns.
3. The clear operation is synchronous: it completes before the reconnect loop begins. There
   is no window between the disconnect detection and the overlay clear where a stale prompt
   could be interacted with.
4. After the overlay is cleared, `AppMode` is no longer `Overlay` (there are no prompts to
   display). If the TUI was in `AppMode::Overlay`, it transitions to `AppMode::Dashboard`
   as part of the SOQ-3 handler (BC-2.06.016).
5. The cleared prompts are NOT preserved in any intermediate buffer. They are discarded
   permanently. On reconnect, only prompts still pending in the daemon's registry (those
   that have not yet timed out) are re-delivered via `InitialState.overlay_stack`.
6. The `TransportEvent::Disconnected` signal is emitted for ALL unexpected disconnects:
   daemon crash, daemon graceful shutdown while TUI is connected, network interruption.
   It is NOT emitted for TUI-initiated graceful disconnect (TUI process exits normally).

## Invariants

1. **SOQ-3 ordering is unconditional:** `TransportEvent::Disconnected` is always the first
   event emitted on connection loss. The reconnect loop NEVER starts before the Disconnected
   event is handled. This ordering is enforced at the `UdsTransport` level — not in the TUI
   event handler — so it cannot be bypassed by TUI code changes.
2. **Zero ghost-approval window:** At no point after `UdsTransport` detects a connection loss
   can the TUI send a `PermissionDecision` for a stale prompt. The overlay is cleared before
   any reconnect attempt, and the TUI cannot interact with a cleared overlay.
3. **Idempotent clear:** If the TUI's `VecDeque<PromptModal>` is already empty when the
   Disconnected event is received (no prompts were queued), the SOQ-3 handler runs without
   error. An empty-clear is a no-op.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TUI has 3 queued prompts when daemon crashes | All 3 `PromptModal` entries cleared synchronously. AppMode transitions to Dashboard. Reconnect loop begins. On successful reconnect, `InitialState.overlay_stack` re-delivers any of the 3 prompts that are still pending (within their 300ms timeout window). |
| EC-002 | TUI has 0 queued prompts when daemon crashes | SOQ-3 handler runs; no-op clear. AppMode remains Dashboard. Reconnect loop begins. |
| EC-003 | TUI is in `AppMode::Dashboard` (not Overlay) when daemon crashes | SOQ-3 handler runs; no-op clear; AppMode remains Dashboard. Reconnect loop begins. |
| EC-004 | User attempts to press a key (e.g., Accept keybinding) during the brief window between connection loss and SOQ-3 clear | The crossterm event loop receives the keypress. The `UdsTransport` connection-loss error is processed first (SOQ-3 is synchronous in the event loop). The keypress action is dispatched after the overlay is cleared; since there are no prompts, the action is a no-op. |
| EC-005 | Daemon shuts down gracefully while TUI has a queued prompt | Daemon sends HTTP fail-open/closed response before shutdown (hook timeout will fire). TUI receives EOF on UDS socket. SOQ-3 fires; overlay cleared. This is the correct behavior: the daemon has already handled the pending decision via timeout. |
| EC-006 | `TransportEvent::Disconnected` is received but the TUI event loop is busy processing a render cycle | The Disconnected event is queued in the TUI event channel. The event loop processes it at the next iteration, before the reconnect loop starts. The reconnect loop is also initiated from the event loop, ensuring ordering. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| TUI has 2 queued prompts; daemon UDS connection drops (EOF) | `TransportEvent::Disconnected` emitted; VecDeque cleared (length 0); AppMode = Dashboard; reconnect loop begins | happy-path |
| TUI has 0 queued prompts; daemon UDS connection drops | `TransportEvent::Disconnected` emitted; no-op clear (VecDeque remains empty); reconnect loop begins | happy-path |
| TUI has 1 queued prompt; daemon restarts; TUI reconnects within 5s; prompt still pending (within 300ms) | SOQ-3 fires (clear); reconnect; `InitialState.overlay_stack` re-delivers prompt; TUI shows overlay with 1 prompt | happy-path |
| TUI has 1 queued prompt; daemon restarts; TUI reconnects within 5s; prompt timed out | SOQ-3 fires (clear); reconnect; `InitialState.overlay_stack` is empty (prompt already timed out); TUI shows Dashboard | edge-case |
| User presses Accept key within 1ms of disconnect | SOQ-3 fires synchronously first; keypress handled after clear; no Accept sent to nonexistent channel | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `TransportEvent::Disconnected` emitted before reconnect loop starts | unit (mock UdsTransport) |
| VP-TBD | VecDeque<PromptModal> is empty after SOQ-3 handler runs | unit |
| VP-TBD | SOQ-3 fires on EOF, BrokenPipe, and ConnectionReset (all three error variants) | unit |
| VP-TBD | SOQ-3 does NOT fire on graceful TUI-initiated disconnect | unit |
| VP-TBD | AppMode transitions to Dashboard after overlay cleared (if was Overlay) | unit |
| VP-TBD | Zero ghost-approval window: no PermissionDecision sent after disconnect detected | integration |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability |
| Capability Anchor Justification | CAP-005 ("Internal TUI-to-daemon transport; UDS framing; session/event/prompt push; permission decision routing; SOQ-3 overlay clear") per ARCH-INDEX §Capability Traceability §SS-05 — the capability description explicitly names "SOQ-3 overlay clear" as a responsibility of SS-05; this BC is the canonical specification of that invariant at the IPC transport layer |
| L2 Domain Invariants | DI-001 (hook events written before ACK — SOQ-3 protects the integrity of this invariant by ensuring TUI clients cannot send decisions for timed-out prompts after daemon restart, which would corrupt the decision channel); DI-007 (monocle must not write to harness-owned files — ghost approvals would cause monocle to indirectly influence Claude Code behavior via a stale decision channel; SOQ-3 prevents this) |
| Architecture Module | monocle-ipc (UdsTransport, TransportEvent::Disconnected) per ARCH-INDEX Subsystem Registry SS-05 |
| Architecture Source | SS-ipc.md v1.17.0 §SOQ-3 Overlay Clear on Disconnect; SS-ipc.md v1.17.0 §Reconnection Behavior |
| Cross-Ref | BC-2.05.006 (reconnect loop — SOQ-3 fires before the loop begins); BC-2.06.016 (TUI-layer handler that clears VecDeque<PromptModal> in response to TransportEvent::Disconnected); product-brief.md line 145 (SOQ-3 definition) |
| Test File | `monocle-ipc/tests/soq3_overlay_clear.rs` |
| Test Name | `test_BC_2_05_007_overlay_cleared_on_disconnect` |
| Stories | S-TBD (filled by story-writer) |

## Related BCs

- [BC-2.05.006] — composes with: SOQ-3 clear is step 1 of the reconnect sequence
- [BC-2.06.016] — composes with: TUI event loop handler that performs the VecDeque clear in response to TransportEvent::Disconnected

## Architecture Anchors

- `architecture/SS-ipc.md#soq-3-overlay-clear-on-disconnect` — SOQ-3 rationale, ordering requirement (clear before reconnect), TransportEvent::Disconnected signal
- `architecture/SS-ipc.md#reconnection-behavior` — step 1: "Overlay clear (SOQ-3)" is the first step in the reconnect sequence

## Story Anchor

S-TBD — Implement SOQ-3 TransportEvent::Disconnected emission and overlay-clear handler (filled by story-writer)

## VP Anchors

VP-TBD — SOQ-3 invariant verification properties (filled after VP creation)

## §Trace v1.0.0

**Initial production** (2026-05-26T04:00:00Z):
- BC-2.05.007 authored for SS-05 IPC subsystem per `prd-expansion-scope.md §3.2` and
  `SS-ipc.md §SOQ-3 Overlay Clear on Disconnect + §Reconnection Behavior step 1`.
- Covers: SOQ-3 safety invariant (ghost-approval prevention), TransportEvent::Disconnected
  emission at UdsTransport layer (before reconnect), synchronous VecDeque<PromptModal> clear,
  AppMode Dashboard transition, ordering guarantee (disconnect → clear → reconnect), idempotent
  clear on empty VecDeque, graceful-disconnect non-triggering.
- 6 edge cases documented (EC-001..EC-006).
- DI-007 cited: SOQ-3 prevents monocle from indirectly influencing Claude Code behavior via
  stale decision channels, upholding the observe-only invariant.
- SE-16d PASS: 2026-05-26T04:00:00Z is the production timestamp for this wave.


## §Trace v1.0.1

**F-P1D2-010 LOW — Architecture Source pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.0.0` → `SS-ipc.md v1.1.0` per F-P1D2-010 bulk update (cosmetic pin refresh).
- SE-16d monotonicity: v1.0.1 timestamp >= v1.0.0. PASS.

## §Trace v1.0.2

**F-P1D4-004 LOW — Architecture Source pin updated from v1.1.0 to v1.3.0** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.1.0` (2 occurrences) → `SS-ipc.md v1.3.0` per F-P1D4-004 bulk update.
- SE-16d monotonicity: v1.0.2 timestamp >= v1.0.1. PASS.

## §Trace v1.0.3

**F-FINAL-003 LOW — Architecture Source version pin updated** (2026-05-26T00:00:00Z):
- Architecture Source: `SS-ipc.md v1.3.0` (2 occurrences) → `SS-ipc.md v1.4.0` per F-FINAL-003 bulk pin update.
- SE-16d monotonicity: v1.0.3 timestamp >= v1.0.2. PASS.

## §Trace v1.0.4

**ADV23-SCOPE-001 — Path B Category 8 scope expansion: SS-ipc.md v1.4.0 → v1.9.0 Architecture Source pin refresh** (2026-05-29T00:00:00Z):
- Architecture Source row: `SS-ipc.md v1.4.0 §SOQ-3 Overlay Clear on Disconnect` → `SS-ipc.md v1.9.0 §SOQ-3 Overlay Clear on Disconnect`; `SS-ipc.md v1.4.0 §Reconnection Behavior` → `SS-ipc.md v1.9.0 §Reconnection Behavior`.
- Plain version-pin refresh. No substantive content propagation required — §SOQ-3 Overlay Clear on Disconnect and §Reconnection Behavior section headings and content anchors are unchanged between v1.4.0 and v1.9.0.
- SE-16d monotonicity: v1.0.4 timestamp >= v1.0.3. PASS.
