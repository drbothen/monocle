---
scenario_id: HS-EXP-003
title: "IPC InitialState Captures Pending Prompts for Late-Connecting TUI"
wave: 6
stories_tested: [S-022]
source_bcs: [BC-2.05.002, BC-2.05.005]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-003: IPC InitialState Captures Pending Prompts for Late-Connecting TUI

**Wave:** 6
**Source BC:** BC-2.05.002 (postconditions PC-1, PC-2, invariant INV-3), BC-2.05.005 (PC-1)
**Stories Tested:** S-022

## Setup

A running monocle daemon with zero TUI clients connected. A PreToolUse hook POST is in-flight
(the daemon has registered a `prompt_id` in its pending-decision registry; the hook response
is being held open). Now a new TUI client connects.

## Steps

1. POST `/hooks/pre-tool-use` with `Bash` tool payload. Mock `EngineModule` returns `HookDecision::Defer`.
2. Daemon registers `prompt_id_X` in the pending-decision registry; broadcasts `PermissionPromptQueued`
   (goes nowhere — no clients yet).
3. (While the hook response is still open) Connect a new TUI client via UDS.
4. Read the first framed message from the TUI client's UDS connection.
5. Decode the message as `ServerToClient`.
6. Assert on the message content.
7. Send `ClientToServer::PermissionDecision { prompt_id: prompt_id_X, decision: Accept }`.
8. Verify the hook caller (step 1) receives HTTP 200 `{"decision": "allow"}`.
9. Verify the TUI client receives a subsequent `ServerToClient::PermissionPromptResolved { prompt_id: prompt_id_X }`.

## Expected Outcome

- Step 4: the first message is `ServerToClient::InitialState` (not `PermissionPromptQueued`, not any other variant).
- Step 6: `initial_state.overlay_stack` contains exactly one entry: `PermissionPromptPayload { prompt_id: prompt_id_X, ... }`.
- Step 6: `initial_state.sessions` reflects the current session count (may be 0 or 1 depending on session registry).
- Step 6: `initial_state.drop_counter` is 0.
- Step 7-8: The late-connecting TUI can fully resolve the prompt — the daemon routes the decision to the `oneshot`.
- Step 9: `PermissionPromptResolved` is broadcast to the now-connected TUI (the client receives it).
- No events are dropped between the `InitialState` snapshot and subsequent streaming — the gap-free invariant (INV-3) holds.

## Satisfaction Criteria

PASS: All 6 assertions hold in 5 consecutive runs. The late-connecting TUI can fully resolve any
pending prompt that was registered before it connected.

FAIL: First message is not `InitialState`; `overlay_stack` is empty when a pending prompt exists;
resolution fails because the prompt was not in the initial state; duplicate `PermissionPromptResolved`
sent (indicates the oneshot was resolved twice).

**NOT in any story AC:** S-022 ACs test `InitialState` as the first message (AC-002), `overlay_stack`
contents (AC-015), and the resolution lifecycle (AC-009). This holdout tests the specific case where
a prompt is pending BEFORE the TUI connects — the "catch-up" window — and verifies that a late-
connecting TUI can still resolve a prompt already queued in the daemon. This is the gap-free invariant
(BC-2.05.002 INV-3) in its most adversarial form.
