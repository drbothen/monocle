---
scenario_id: HS-EXP-006
title: "Ctrl-\\ Popup: Permission Prompts Survive Hide/Show Cycle Without Corruption"
wave: 6
stories_tested: [S-025, S-026]
source_bcs: [BC-2.06.004, BC-2.06.007, BC-2.06.008]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-006: Ctrl-\\ Popup: Permission Prompts Survive Hide/Show Cycle Without Corruption

**Wave:** 6
**Source BC:** BC-2.06.004 (PC-1, PC-2), BC-2.06.007 (PC-1), BC-2.06.008 (PC-1, INV-1)
**Stories Tested:** S-025, S-026

## Setup

A running monocle daemon with one connected TUI client (`monocle-tui` launched as a tmux popup).
One permission prompt is queued in the daemon (`prompt_id_P1`). The TUI is displaying the prompt
in `AppMode::Overlay`. The user "hides" the popup (presses `Ctrl-\` to dismiss the tmux popup
window) and then "shows" it again (presses `Ctrl-\` a second time to re-open).

The "hide" is simulated by sending SIGTERM to the `monocle-tui` process (the TUI exits). The
"show" is simulated by launching a fresh `monocle-tui` process that reconnects to the same daemon.

## Steps

1. Daemon has `prompt_id_P1` pending (300ms timeout not yet expired).
2. TUI-1 is running; `overlay_stack.len() == 1`; `AppMode::Overlay`.
3. TUI-1 is terminated via SIGTERM (graceful exit — not SIGKILL). Terminal restored.
4. TUI-1 terminates. Note: graceful disconnect does NOT emit `TransportEvent::Disconnected`
   (per BC-2.05.007 postcondition PC-6). The daemon retains `prompt_id_P1` in its registry.
5. TUI-2 launches and connects to the daemon.
6. TUI-2 receives `ServerToClient::InitialState` with `overlay_stack: [PermissionPromptPayload { prompt_id: prompt_id_P1 }]`.
7. TUI-2 immediately enters `AppMode::Overlay` (stack is non-empty on connect per S-025 AC-008).
8. TUI-2 user presses `y` to accept `prompt_id_P1`.

## Expected Outcome

- Step 3: TUI-1 exits cleanly (terminal restored to normal mode — no raw mode leakage).
- Step 4: The daemon does NOT clear `prompt_id_P1` from its registry when TUI-1 disconnects.
  (TUI-initiated graceful disconnect does not trigger SOQ-3 in the daemon; the prompt stays pending.)
- Step 5-6: TUI-2 receives `prompt_id_P1` in `InitialState.overlay_stack`.
- Step 7: TUI-2 is in `AppMode::Overlay` after rendering the first frame (not Dashboard).
- Step 8: TUI-2 sends `PermissionDecision::Accept` for `prompt_id_P1`; daemon resolves the hook
  response with `{"decision": "allow"}`; TUI-2 receives `PermissionPromptResolved { prompt_id: prompt_id_P1 }`;
  `overlay_stack` empties; `AppMode` collapses to `Dashboard`.

## Satisfaction Criteria

PASS: The permission prompt survives the full hide/show cycle. TUI-1 exits cleanly. TUI-2 can
resolve the prompt. No terminal corruption.

FAIL: `prompt_id_P1` is absent from `InitialState` when TUI-2 connects (daemon incorrectly cleared
the registry on TUI-1 graceful disconnect); TUI-2 starts in `AppMode::Dashboard` despite a pending
prompt; terminal raw mode leaks after TUI-1 exits; double resolution of `prompt_id_P1` is attempted.

**NOT in any story AC:** S-025 AC-002 (connection on startup), AC-008 (overlay pre-load). S-026 AC-012
(overlay restored on reconnect). This holdout tests the specific `Ctrl-\` usage pattern — the most
common user interaction with monocle — and the non-emission of `TransportEvent::Disconnected` on
graceful exit (BC-2.05.007 PC-6), which means the daemon retains pending prompts for re-delivery.
The combination is not mechanically stated in any AC.
