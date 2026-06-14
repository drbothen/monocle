---
scenario_id: HS-EXP-011
title: "Session Survives Graceful Daemon Restart — PTY Stream Re-Attached, SessionEntry Visible"
wave: 8
stories_tested: [S-TBD-session-manager]
source_bcs: [BC-2.08.002, BC-2.08.004, BC-2.05.006]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T12:00:00Z
---

# HS-EXP-011: Session Survives Graceful Daemon Restart — PTY Stream Re-Attached, SessionEntry Visible

**Wave:** 8
**Source BC:** BC-2.08.002 (postconditions PC-1, PC-3), BC-2.08.004 (PC-1, PC-6 + Invariant 1), BC-2.05.006 (PC-1)
**Stories Tested:** S-TBD-session-manager

## Setup

A running monocle daemon with one connected TUI client. One active session is alive: `session_id = S1`,
associated `monocle-session-host` process is alive (verified via PID liveness check). The session has
a `session-state.json` sidecar at the flat path `<runtime_dir>/session-S1.json` (canonical flat layout
per SS-session-manager.md — NOT nested under `runtime_dir/sessions/S1/`) with `state: "Running"`.
A `ratatui::TestBackend` is connected to the TUI for frame inspection.

## Steps

1. Verify initial state: TUI sessions panel shows S1 as `Running`. Daemon `SessionRegistry` contains
   S1 with alive PID.

2. Send graceful stop signal to the daemon (SIGTERM or `monocle daemon stop`). The daemon initiates
   its 10-second drain window per BC-2.01.004.

3. Observe: the `monocle-session-host` process for S1 is **still alive** during and after daemon
   shutdown. S1's PTY is running independently; it did NOT receive SIGTERM from the daemon.

4. The daemon completes shutdown and exits.

5. Start a new daemon instance. The new daemon must NOT bind its UDS socket until re-discovery
   completes (BC-2.08.004 PC-6 + Invariant 1: UDS bind proceeds only after `rediscover_sessions()`
   returns; ordering is mandatory to prevent TUI from receiving an incomplete session list).

6. The new daemon performs re-discovery: scans `runtime_dir/session-*.json` (flat glob per SS-session-manager.md
   §re-discovery algorithm), finds `session-S1.json`, verifies PID liveness, and adds S1 back to
   `SessionRegistry` with `state: "Running"` (re-discovered sessions resume `Running` state;
   there is no `Reconnected` state in the SessionState enum).

7. Re-discovery must complete within 5 seconds of daemon start (BC-2.08.004 PC-1 timing SLA).

8. The new daemon binds UDS after re-discovery. TUI reconnects (BC-2.05.006) and receives a new
   `InitialState` push that includes S1 in the session list.

9. TUI sessions panel renders S1 as alive with `state: Running`. The panel must
   show S1 without requiring any user action.

## Expected Outcome

- Step 3: the `monocle-session-host` process PID is alive after daemon shutdown. The session host
  is not coupled to daemon lifecycle (ADR-0009 detached process model).
- Step 5: the new daemon does NOT accept IPC connections before re-discovery finishes. If the TUI
  tries to reconnect early, it receives connection-refused or sees the socket does not yet exist.
- Step 7: re-discovery completes within 5 seconds (measured from new daemon process start to UDS bind).
- Step 9: TUI shows S1 as `Running` after reconnect. The sessions panel is not empty.

## Satisfaction Criteria

PASS: S1's session-host process is alive throughout the daemon restart cycle; re-discovery scans the flat
`runtime_dir/session-*.json` glob and adds S1 to the new daemon's registry within 5 seconds with
`state: Running`; UDS bind occurs only after re-discovery; TUI shows S1 as `Running` after reconnect.

FAIL: session-host process dies when daemon receives SIGTERM; re-discovery takes >5 seconds; UDS
socket becomes available before re-discovery completes (race window open); TUI shows an empty sessions
panel after reconnect because S1 was not re-discovered; sidecar state is `Reconnected` (non-existent
state) or uses nested path `runtime_dir/sessions/S1/session-state.json` (wrong path).

**NOT in any story AC:** The story implementing BC-2.08.002 will have ACs verifying the session-host
survives daemon shutdown. The story implementing BC-2.08.004 will have ACs for the re-discovery
algorithm. This holdout tests the **integration** between these two behaviors: the timing property
that a real daemon restart followed by real re-discovery followed by real TUI reconnect all complete
within the combined SLA, with no race window where the TUI can observe an empty session list between
the old and new daemon. The integration ordering constraint (UDS bind blocked until re-discovery)
is not exercised by any single story AC in isolation.
