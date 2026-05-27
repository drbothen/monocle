---
scenario_id: HS-EXP-004
title: "SOQ-3 Overlay Clear on Daemon Disconnect — VecDeque Empty Before First Reconnect Attempt"
wave: 6
stories_tested: [S-023, S-026]
source_bcs: [BC-2.05.006, BC-2.05.007]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-004: SOQ-3 Overlay Clear on Daemon Disconnect — VecDeque Empty Before First Reconnect Attempt

**Wave:** 6
**Source BC:** BC-2.05.007 (postconditions PC-1, PC-2, PC-3, PC-4, invariant INV-1), BC-2.05.006 (PC-2)
**Stories Tested:** S-023, S-026

## Setup

A running monocle daemon with one connected TUI client. The TUI has two permission prompts
queued (`prompt_id_1`, `prompt_id_2`) — the TUI is currently in `AppMode::Overlay`. A
synchronization mechanism (tokio Notify or channel) is used to detect the exact instant
the SOQ-3 handler runs and the reconnect loop is scheduled.

## Steps

1. Connect TUI client to daemon via UDS.
2. Daemon emits two `PermissionPromptQueued` messages; TUI `overlay_stack.len() == 2`; `AppMode == Overlay`.
3. Daemon is killed (SIGKILL to force an unexpected disconnect — not graceful shutdown).
4. Observe the TUI's `UdsTransport` receive loop: it detects `UnexpectedEof`.
5. Capture the sequence of events in the TUI event loop:
   a. `TransportEvent::Disconnected` emitted.
   b. SOQ-3 handler: `overlay_stack.clear()` + `AppMode` transition to `Dashboard`.
   c. Reconnect loop: first `reconnect()` attempt with 250ms backoff.
6. Assert on the state of `overlay_stack` and `AppMode` AFTER step 5b and BEFORE step 5c.
7. Assert on the status bar rendering immediately after SOQ-3 (before reconnect completes).

## Expected Outcome

- After SOQ-3 fires (step 5b) and before the reconnect loop starts (step 5c):
  `overlay_stack.len() == 0`; `AppMode == Dashboard`.
- The transition to `AppMode::Dashboard` is visible in the rendered frame BEFORE the first
  reconnect attempt — no frame is rendered with `AppMode::Overlay` and an empty stack.
- Status bar shows `[daemon: reconnecting...]` immediately after SOQ-3.
- SOQ-3 fires for `UnexpectedEof` — confirmed by checking the 3 error variant trigger paths.
- The reconnect loop starts with backoff 250ms (not 0ms or 500ms).

**Ghost-approval impossibility assertion:** Between disconnect detection and the moment the
first reconnect completes (or the 5-second window expires), no `PermissionDecision` message
can be sent by the TUI. The cleared overlay has no entries to approve. This is not a timing
assertion — it is a structural assertion: `overlay_stack.is_empty()` → no `PermissionDecision`
key binding is active in the `Overlay` AppMode (because AppMode is `Dashboard`).

## Satisfaction Criteria

PASS: SOQ-3 fires synchronously on disconnect; `overlay_stack` is empty before any reconnect
attempt; AppMode is Dashboard; status bar shows reconnecting; ghost-approval is structurally
blocked.

FAIL: Any ordering violation: reconnect attempt starts before `overlay_stack` is cleared;
`AppMode::Overlay` with empty stack persists even briefly; SOQ-3 fires asynchronously (in a
separate task) rather than synchronously in the transport receive path; status bar does not
update until after reconnect.

**NOT in any story AC:** S-023 ACs test `VecDeque` clear (AC-002), synchronous ordering (AC-003),
AppMode transition (AC-004), and idempotent clear (AC-015). S-026 AC-011 tests disconnect-triggers-
clear. This holdout tests the observable *sequence of rendered frames and state transitions* across
both stories simultaneously — specifically that no invalid intermediate state is ever rendered and
that the ghost-approval structural impossibility holds end-to-end.
