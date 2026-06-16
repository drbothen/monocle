---
scenario_id: HS-EXP-013
title: "Permission Badge+Bell While in EmbeddedTerminal — SUG-3 Guarantee: Prompt Never Silently Queued"
wave: 8
stories_tested: [S-044]
source_bcs: [BC-2.09.009, BC-2.06.008]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T12:00:00Z
---

# HS-EXP-013: Permission Badge+Bell While in EmbeddedTerminal — SUG-3 Guarantee: Prompt Never Silently Queued

**Wave:** 8
**Source BC:** BC-2.09.009 (postconditions PC-1, PC-2, PC-3), BC-2.06.008 (PC-1: VecDeque push)
**Stories Tested:** S-044

## Setup

`ratatui::backend::TestBackend` with 120×40 terminal. A monocle TUI in `AppMode::EmbeddedTerminal`
state — the user is actively watching a PTY session for `session_id = S1`. A `MockDaemon` is
connected via UDS.

## Steps

1. Verify initial state: TUI is in `AppMode::EmbeddedTerminal { session_id: S1 }`. Status bar shows
   no badge. The PTY area renders synthetic vt100 output.

2. MockDaemon sends `ServerToClient::PermissionPromptQueued { prompt_id: P1, tool: "Bash", ... }`
   to the TUI.

3. Observe the **next rendered frame** (within one render tick, not after Esc or mode change):
   - The status bar MUST show a permission badge (e.g., `[!1 prompt]` or equivalent visual indicator)
     indicating a queued prompt exists.
   - An audible bell character `\x07` MUST be emitted to the real terminal output.

4. The TUI remains in `AppMode::EmbeddedTerminal`. The mode does NOT automatically switch to
   `AppMode::Overlay`. The user retains PTY focus.

5. Verify the `overlay_stack` has grown by 1 (P1 is in the stack per BC-2.06.008). The stack
   is behind the EmbeddedTerminal view — not visible — but it is populated.

6. Send a second prompt: MockDaemon sends `PermissionPromptQueued { prompt_id: P2, tool: "Write", ... }`.

7. Observe: the badge count increments (e.g., `[!2 prompts]`). A second bell is emitted.

8. Press `Esc` in `AppMode::EmbeddedTerminal` (the canonical exit keybinding per BC-2.09.009
   PC-5a: `Esc → AppMode transitions to prior`). Because `overlay_stack` is non-empty, per
   BC-2.09.009 PC-5b the mode immediately transitions from `prior` to
   `AppMode::Overlay { prior: Dashboard }`. TUI transitions to `AppMode::Overlay`.

9. Verify: `overlay_stack.len() == 2`; P1 and P2 are both present; the overlay renders the
   most recently added prompt (or the front of the stack per BC-2.06.009 rotation semantics).

## Expected Outcome

- Step 3: badge appears in status bar within one render tick of `PermissionPromptQueued` receipt.
  The badge is visible on the **same frame** as the next PTY output render cycle — not deferred.
- Step 3: bell `\x07` is emitted. The bell is audible to the user even without looking at the TUI.
- Step 4: `AppMode` remains `EmbeddedTerminal`. No automatic mode switch occurs.
- Step 7: second badge increment appears within one render tick. Second bell emitted.
- Step 9: after the user explicitly switches, both prompts are in the overlay stack in the correct order.

## Adversarial Probe

Verify that monocle does NOT silently absorb the prompts: if the user never presses the overlay key,
the prompts remain in `overlay_stack` indefinitely (until the timeout fires or the session ends).
The badge count MUST remain visible — it MUST NOT disappear after one render cycle or self-dismiss.

## Satisfaction Criteria

PASS: badge appears within one render tick for each prompt; bell emitted for each prompt; AppMode
stays EmbeddedTerminal; explicit mode switch shows both prompts in overlay; badge persists until
prompts are resolved.

FAIL: badge does not appear while in EmbeddedTerminal (prompts silently queued); badge disappears
before prompts are resolved; no bell emitted; AppMode auto-switches to Overlay without user action;
one or both prompts are missing from overlay_stack after mode switch.

**NOT in any story AC:** The story implementing BC-2.09.009 will have ACs testing badge render and
bell emission. The story implementing BC-2.06.008 will have ACs testing VecDeque push. This holdout
tests the **SUG-3 integration guarantee**: that the badge is surfaced within the SAME render frame
as the queued prompt, even when in EmbeddedTerminal mode where PTY output is competing for render
capacity. The single-render-tick latency bound and the bell emission in a PTY-active state are
timing properties that can only be validated by a combined end-to-end test with a running TUI in
EmbeddedTerminal mode receiving concurrent PTY bytes and permission prompts.
