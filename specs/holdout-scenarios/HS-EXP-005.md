---
scenario_id: HS-EXP-005
title: "IPC State Fully Rebuilds from InitialState After Daemon Restart"
wave: 6
stories_tested: [S-023]
source_bcs: [BC-2.05.006]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-005: IPC State Fully Rebuilds from InitialState After Daemon Restart

**Wave:** 6
**Source BC:** BC-2.05.006 (postconditions PC-3, PC-6, PC-7, PC-8)
**Stories Tested:** S-023

## Setup

A running monocle daemon with one connected TUI client. The TUI has received prior `SessionState`
updates and has a non-empty session list. The daemon is then stopped and restarted (new PID, new
auth token, new lock file). The TUI enters the reconnect loop.

## Steps

1. Start daemon (Daemon-A). TUI connects; receives `InitialState` with 0 sessions.
2. One hook event triggers session creation. TUI receives `SessionState` update for session S1.
3. Kill Daemon-A (SIGKILL). TUI receives `TransportEvent::Disconnected`.
4. SOQ-3 fires: `overlay_stack.clear()`; `AppMode` → `Dashboard`.
5. TUI begins reconnect loop with 250ms backoff.
6. After first retry attempt: TUI re-reads `<runtime_dir>/monocle.lock`. Daemon-A's lock file
   is gone (cleaned up by kill or leftover — holdout tests BOTH cases).
7. Start Daemon-B (new PID, new auth token). Lock file appears at same path with new content.
8. TUI detects new lock file on a subsequent retry read; updates its internal `pid`, `port`,
   `auth_token` fields.
9. TUI connects to Daemon-B.
10. TUI receives fresh `ServerToClient::InitialState` from Daemon-B.

## Expected Outcome

- Step 10: TUI discards all prior local state (session S1 is gone from TUI's session list).
- `InitialState` from Daemon-B has empty sessions (Daemon-B has no sessions yet).
- TUI renders 0 sessions (not 1 stale session from Daemon-A).
- Status bar reverts to normal (no `[daemon: reconnecting...]` indicator).
- If Daemon-A's lock file was stale (PID dead): TUI re-reads the file, detects dead PID,
  waits for file change. This must NOT cause an infinite loop or a TUI panic.
- Drop counter resets to 0 (Daemon-B starts with counter 0; TUI reflects Daemon-B's value).

## Satisfaction Criteria

PASS: After successful reconnect to Daemon-B, TUI shows zero sessions (not stale Daemon-A data);
status bar is normal; no panic or infinite loop on stale lock file.

FAIL: TUI retains Daemon-A session data after reconnect to Daemon-B; drop counter carries over;
TUI panics on stale lock file; TUI enters infinite reconnect loop when Daemon-B has not yet started.

**NOT in any story AC:** S-023 AC-008 (lock file re-read after each retry), AC-011 (InitialState
rebuild), AC-013 (status bar revert). This holdout tests all three properties simultaneously across
a daemon-restart event, plus the stale lock file edge case (AC-008 + AC-010 interaction) and
the explicit assertion that ALL prior session data is purged from TUI state — not just the overlay stack.
