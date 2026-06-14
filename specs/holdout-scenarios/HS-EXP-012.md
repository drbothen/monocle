---
scenario_id: HS-EXP-012
title: "Re-Discovery Completes Before UDS Bind — No TUI Connection Accepted During Discovery Window"
wave: 8
stories_tested: [S-TBD-session-manager]
source_bcs: [BC-2.08.004]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-06-03T12:00:00Z
---

# HS-EXP-012: Re-Discovery Completes Before UDS Bind — No TUI Connection Accepted During Discovery Window

**Wave:** 8
**Source BC:** BC-2.08.004 (PC-6 + Invariant 1: UDS bind MUST NOT precede re-discovery completion)
**Stories Tested:** S-TBD-session-manager

## Setup

A `tempfile::TempDir` as runtime_dir. Three alive session-host processes pre-existing: `S1`, `S2`,
`S3`. Their `session-state.json` sidecars are at flat paths `<runtime_dir>/session-S1.json`,
`<runtime_dir>/session-S2.json`, `<runtime_dir>/session-S3.json` (canonical flat layout per
SS-session-manager.md — NOT nested under `runtime_dir/sessions/<id>/`) with `state: "Running"`
and valid PIDs. A mock TUI client will attempt to connect to the UDS socket during the daemon
startup sequence to probe whether the socket appears before or after re-discovery finishes.

## Steps

1. Start a new daemon instance. Begin a polling loop on the TUI side: every 5ms, attempt
   `UnixStream::connect(runtime_dir/monocle.sock)`. Record the timestamp of first successful connect
   as `T_socket_available`.

2. Simultaneously, instrument the daemon's re-discovery function to record the timestamp when
   re-discovery completes (all three sessions scanned and registered) as `T_rediscovery_done`.

3. The daemon performs re-discovery by scanning `runtime_dir/session-*.json` (flat glob). During
   re-discovery the daemon MUST NOT bind the UDS socket.

4. After re-discovery finishes, the daemon binds the UDS socket.

5. The TUI client's polling loop eventually succeeds; record `T_socket_available`.

6. The TUI connects and receives `InitialState`. Verify `InitialState.sessions` contains S1, S2, S3.

## Expected Outcome

- `T_socket_available >= T_rediscovery_done`: the TUI cannot connect before re-discovery finishes.
  The socket does not exist or rejects connections during the re-discovery window.
- `InitialState.sessions` received immediately after connect contains all three sessions (S1, S2, S3).
  There is no window where the TUI receives an empty session list and then a `SessionListUpdate`.

## Adversarial Probe

Inject an artificial 100ms sleep inside the re-discovery scan loop (simulating slow disk/PID check)
and verify the TUI's connect attempts all fail with connection-refused (or no-such-file) during
that 100ms window.

## Satisfaction Criteria

PASS: `T_socket_available >= T_rediscovery_done` in all trials; `InitialState` delivered to TUI
with all 3 sessions immediately (each with `state: Running`); adversarial 100ms sleep probe shows
all TUI connect attempts during the sleep fail; sidecars are at flat paths `runtime_dir/session-*.json`.

FAIL: TUI successfully connects before `T_rediscovery_done`; `InitialState` delivered with fewer
than 3 sessions (TUI receives a partial or empty initial state); adversarial probe shows at least
one TUI connect succeeds during the re-discovery window; sidecar paths use nested directory structure;
sidecar field is `status` instead of `state`.

**NOT in any story AC:** The implementing story's AC for BC-2.08.004 will verify that re-discovery
completes within 5 seconds and that the registry contains all alive sessions. This holdout adds a
**temporal ordering proof**: the socket must not be accessible before re-discovery is finished.
This is a race condition property — it requires concurrent probing of both the socket availability
and the re-discovery completion timestamp, which cannot be asserted by a sequential unit or integration
test of either BC-2.08.004 or BC-2.05.001 in isolation.
