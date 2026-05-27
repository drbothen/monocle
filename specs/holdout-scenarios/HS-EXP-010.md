---
scenario_id: HS-EXP-010
title: "Permission Overlay Lifecycle: Queue → Timeout-Resolved → Clear from Both Paths"
wave: 6
stories_tested: [S-022, S-026]
source_bcs: [BC-2.05.005, BC-2.06.011, BC-2.06.016]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-010: Permission Overlay Lifecycle — Queue → Timeout-Resolved → Clear from Both Paths

**Wave:** 6
**Source BC:** BC-2.05.005 (postcondition PC-4), BC-2.06.011 (PC-1), BC-2.06.016 (PC-1)
**Stories Tested:** S-022, S-026

## Setup

A running monocle daemon with one connected TUI client. The PreToolUse handler's 300ms timeout
is the active resolution mechanism (no human interaction). The TUI has `ratatui::TestBackend`
for frame inspection.

## Steps

### Part A: Timeout-triggered PermissionPromptResolved clears TUI overlay

1. Daemon receives `PreToolUse` for `Bash` tool; `EngineModule` returns `HookDecision::Defer`.
2. Daemon broadcasts `PermissionPromptQueued { prompt_id: P1 }` to TUI.
3. TUI `overlay_stack.push_back(P1)`; `AppMode` → `Overlay`.
4. Wait 310ms (300ms timeout fires).
5. Daemon resolves fail-open: HTTP 200 `{"decision": "allow", "reason": "timeout"}` to Claude Code.
6. Daemon broadcasts `ServerToClient::PermissionPromptResolved { prompt_id: P1 }` to TUI.
7. TUI receives `PermissionPromptResolved { P1 }`.
8. TUI `overlay_stack.retain(|m| m.prompt_id != P1)` → stack is empty.
9. TUI `transition()` collapses to `AppMode::Dashboard`.

### Part B: Disconnect-triggered clear vs. timeout-triggered resolved (race condition)

10. Start a fresh scenario: daemon has `prompt_id: P2` pending; TUI is in `Overlay` with P2 visible.
11. At t=0ms: TUI receives `TransportEvent::Disconnected` (daemon killed).
12. At t=0ms: (concurrent) The 300ms timeout fires for P2, and daemon (now dead) would have sent
    `PermissionPromptResolved { P2 }` — but it cannot because it's dead.
13. Assert: SOQ-3 fires on the TUI side; `overlay_stack.clear()`; `AppMode` → `Dashboard`.
    The TUI does NOT wait for a `PermissionPromptResolved` from the dead daemon.
14. The stale P2 entry is gone. No ghost approval possible.

## Expected Outcome

- Part A: After step 9, `overlay_stack.is_empty()` and `AppMode == Dashboard`. The TUI does NOT
  remain in `Overlay` with an empty stack. The timeout-resolved path clears the overlay correctly.
- Part A TestBackend: Frame after step 9 shows Sessions panel (no overlay content).
- Part B: Disconnect path (SOQ-3) supersedes the timeout-resolved path. The TUI clears its
  overlay on disconnect regardless of whether a `PermissionPromptResolved` would later arrive.
  No race between disconnect and timeout leaves the overlay in an inconsistent state.

## Satisfaction Criteria

PASS: Part A — timeout fires; `PermissionPromptResolved` received; overlay clears; Dashboard restored.
Part B — disconnect fires; SOQ-3 clears overlay synchronously; no inconsistent state.

FAIL: Part A — overlay remains non-empty after `PermissionPromptResolved` for a timeout-triggered
resolution; `AppMode::Overlay` persists with empty stack. Part B — TUI waits for a
`PermissionPromptResolved` that never arrives from a dead daemon, leaving the overlay non-empty.

**NOT in any story AC:** S-022 AC-010 tests timeout-triggered `PermissionPromptResolved` broadcast.
S-026 AC-006 tests `PermissionPromptResolved` pop. S-026 AC-011 tests disconnect clears overlay.
This holdout tests the COMBINATION: does the timeout-triggered `PermissionPromptResolved` path
actually clear the TUI overlay correctly end-to-end? And does the disconnect path correctly
supersede a pending timeout without leaving a stale overlay entry? These two paths interact in
the real implementation and the interaction is not covered by any single AC.
