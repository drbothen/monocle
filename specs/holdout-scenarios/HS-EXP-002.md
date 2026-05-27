---
scenario_id: HS-EXP-002
title: "PreToolUse Daemon Timeout Returns Allow (Fail-Open) with Correct HTTP Body"
wave: 5
stories_tested: [S-018]
source_bcs: [BC-2.04.007, BC-2.04.011]
severity: must-pass
visibility: holdout-evaluator-only
producer: vsdd-factory:product-owner
timestamp: 2026-05-27T00:00:00Z
---

# HS-EXP-002: PreToolUse Daemon Timeout Returns Allow (Fail-Open) with Correct HTTP Body

**Wave:** 5
**Source BC:** BC-2.04.007 (postcondition PC-4, fail-open), BC-2.04.011 (PC-3, try_send non-blocking)
**Stories Tested:** S-018

## Setup

A running monocle daemon. No TUI clients connected (so `PermissionPromptQueued` broadcasts
go nowhere — the daemon holds the hook response open waiting for a decision that never arrives).
A slow `EngineModule` mock that returns `HookDecision::Defer` and then does not resolve the
`oneshot` channel within 300ms.

## Steps

1. POST a valid `PreToolUse` hook request to `/hooks/pre-tool-use` with a `Bash` tool payload.
2. The mock `EngineModule::on_hook()` returns `HookDecision::Defer`.
3. The daemon registers a `oneshot` channel and broadcasts `PermissionPromptQueued` (to zero clients).
4. Wait for 310ms without sending any `PermissionDecision`.
5. Record the HTTP response received by the caller.

## Expected Outcome

- HTTP 200 response with body `{"decision": "allow", "reason": "timeout"}`.
- Response arrives within 350ms of the POST (300ms timeout + ≤50ms processing overhead).
- No HTTP 408 / 503 / 504 — the daemon responds 200 regardless of timeout.
- The `drop_counter` in `DaemonState` is NOT incremented by this scenario (the timeout is not
  a dropped event; the event was published via try_send before the Defer path was entered, and
  the timeout path is a normal resolution, not a channel-full drop).

## Satisfaction Criteria

PASS: HTTP 200 `{"decision": "allow", "reason": "timeout"}` received; response latency ≤ 350ms.

FAIL: Any non-200 response; response body missing `"reason": "timeout"`; response latency > 500ms;
or daemon enters a blocked state (no response at all — indicates `.send().await` was used instead
of `tokio::time::timeout`).

**NOT in any story AC:** S-018 AC-011 specifies that the handler uses `tokio::time::timeout(300ms, ...)`
and returns `{"decision": "allow", "reason": "timeout"}`. This holdout tests the actual latency
guarantee end-to-end (from POST arrival to response delivery) and the precise JSON body shape when
no client is connected to resolve the Defer — a scenario where the oneshot is never resolved at all.
The 350ms bound is a tighter observable contract than the AC implies.
