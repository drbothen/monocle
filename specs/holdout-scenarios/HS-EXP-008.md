---
scenario_id: HS-EXP-008
title: "Killer Scenario: Dual Prompt Resolved in 6 Keystrokes via ratatui TestBackend"
wave: 7
stories_tested: [S-029, S-026, S-027]
source_bcs: [BC-2.06.022, BC-2.06.009, BC-2.06.011]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-008: Killer Scenario — Dual Prompt Resolved in 6 Keystrokes via ratatui TestBackend

**Wave:** 7
**Source BC:** BC-2.06.022 (postcondition PC-3), BC-2.06.009 (PC-1, PC-2), BC-2.06.011 (PC-1)
**Stories Tested:** S-029, S-026, S-027

## Setup

`ratatui::backend::TestBackend` with a 120×40 terminal. A `MockDaemon` connected to the TUI
via UDS in a `tempfile::TempDir`. Two permission prompts pre-queued in the `MockDaemon` before
the TUI starts (so both arrive in `InitialState.overlay_stack`).

## Steps

1. TUI starts; receives `InitialState` with two prompts:
   - `prompt_id_1`: tool `Bash`, command `"echo hello"`
   - `prompt_id_2`: tool `Bash`, command `"cat /etc/hosts"`
2. TUI is in `AppMode::Overlay`; `overlay_stack.len() == 2`.
3. **Keystroke 1:** `n` (reject front prompt `prompt_id_1`).
4. TUI sends `ClientToServer::PermissionDecision { prompt_id: prompt_id_1, decision: Reject }`.
5. MockDaemon sends `PermissionPromptResolved { prompt_id: prompt_id_1 }`.
6. TUI removes `prompt_id_1` from stack; `overlay_stack.len() == 1`; mode remains `Overlay`.
7. **Keystroke 2:** `y` (accept remaining prompt `prompt_id_2`).
8. TUI sends `PermissionDecision { prompt_id: prompt_id_2, decision: Accept }`.
9. MockDaemon sends `PermissionPromptResolved { prompt_id: prompt_id_2 }`.
10. TUI removes `prompt_id_2`; `overlay_stack` is empty; mode collapses to `Dashboard`.

The keystrokes needed are:
- `Ctrl-\` (launch popup) — counted as keystroke 0 (outside TUI scope)
- `n` (reject first prompt) — **keystroke 1**
- `y` (accept second prompt) — **keystroke 2**

For a **single-prompt acceptance** (the minimum case), 1 keystroke (`y`) resolves it.
For a **dual-prompt rejection-then-acceptance** as above, 2 keystrokes resolve both.
The "6 keystroke" goal in the brief includes: `Ctrl-\` (popup open), [focus panel if needed],
`n`, `y`, `Ctrl-\` (popup close). Even with navigation keystrokes included, ≤6 is required.

**Adversarial probe:** inject `Esc` between keystroke 1 and keystroke 2 to verify it is a no-op
(3 Esc presses). Stack must still be 1 after all 3 Esc presses.

## Expected Outcome

- Steps 1-10 complete without error.
- `App.mode == AppMode::Dashboard { .. }` after keystroke 2.
- `App.overlay_stack.is_empty()` after keystroke 2.
- The TestBackend renders a frame after each keystroke; assertions on rendered content:
  - After step 6 (post-`n`): rendered overlay shows `prompt_id_2` (the second prompt) as
    the active entry. The `prompt_id_1` entry is absent from all rendered cells.
  - After step 10 (post-`y`): rendered frame shows no overlay content; Sessions panel is visible.
- Adversarial Esc probe: after 3 Esc presses, `overlay_stack.len() == 1` and `App.mode` is
  still `AppMode::Overlay { .. }`.
- Total wall-clock time for the 2-keystroke sequence: < 200ms (excluding I/O wait; synchronization
  via `tokio::sync::Notify` is used so no polling delay).

## Satisfaction Criteria

PASS: Both prompts resolved in exactly 2 keystrokes; rendered frames match expectations;
Esc is confirmed as a no-op; total latency < 200ms.

FAIL: More than 2 keystrokes required to resolve both prompts; TestBackend render shows
stale prompt content after resolution; Esc triggers any state change; latency > 500ms.

**NOT in any story AC:** S-029 AC-003 tests the two-prompt multi-prompt stacking scenario but
does not assert on rendered frame content or keystroke-to-resolution latency. This holdout
adds: (1) TestBackend rendered-content assertions verifying the correct prompt is displayed
after the first resolution; (2) the Esc no-op adversarial probe interleaved with the resolution
sequence; (3) a latency bound (< 200ms) for the dual-resolution sequence using Notify-based
synchronization (no sleep). These three properties are not captured in any story AC.
