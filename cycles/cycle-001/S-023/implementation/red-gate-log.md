---
document_type: red-gate-log
level: ops
version: "1.0"
status: PASSED
producer: vsdd-factory:test-writer
timestamp: 2026-05-28T00:00:00Z
phase: 3
story_id: S-023
cycle: cycle-001
wave: 6
points: 5
worktree: .worktrees/S-023/
branch: feature/S-023-tui-reconnect-soq3
base_commit: 59f4394
stub_commit: f7964d9
failing_tests_commit: ff38793
red_gate_verified: true
---

# Red Gate Log: S-023 — TUI Reconnect After Daemon Restart + SOQ-3 Overlay Clear

## Summary

| Story | Tests Written | Fail on todo!() | Passing | Gate |
|-------|--------------|-----------------|---------|------|
| S-023 | 34 | 30 | 4 | PASSED |

34 tests added across two files. 30 fail on production `todo!()` panics — confirming
stubs are not implemented and the Red Gate is established. 4 pass: constant-value
sanity checks against spec-pinned constants in production code (non-vacuous; see
Passing Tests section below).

## Stubs Created

Commit: `f7964d9 feat(S-023): add module stubs for TransportEvent + reconnect`

- `fn compute_backoff_delay(attempt: u32) -> Duration` — exponential backoff; `todo!("S-023: compute exponential backoff delay for attempt {attempt}")`
- `fn reconnect_loop(state: &mut ReconnectState) -> ReconnectResult` — main reconnect state machine; `todo!()`
- `fn on_transport_event(event: TransportEvent, overlay: &mut SoqOverlay)` — SOQ-3 overlay clear on reconnect; `todo!()`
- `fn clear_soq3_overlay(overlay: &mut SoqOverlay)` — overlay teardown; `todo!()`
- `struct TransportEvent` — new type introduced in monocle-ipc; `todo!()` constructor
- `struct ReconnectState` with fields: `attempt: u32`, `last_attempt: Instant`, `window_start: Instant`

## Red Gate Verification

Commit: `ff38793 test(S-023): add failing tests for BC-2.05.006 + BC-2.05.007`

### Test files

| File | Test Count | Failing | Passing |
|------|------------|---------|---------|
| `crates/monocle-ipc/tests/soq3_overlay_clear.rs` | 11 | 11 | 0 |
| `crates/monocle-ipc/tests/reconnect.rs` | 23 | 19 | 4 |

### AC Coverage Table

| AC | BC | Test Function(s) | File | Status |
|----|-----|-----------------|------|--------|
| AC-001 | BC-2.05.006 | `test_BC_2_05_006_backoff_attempt_1_is_250ms` | reconnect.rs | FAIL (expected) |
| AC-002 | BC-2.05.006 | `test_BC_2_05_006_backoff_attempt_2_is_500ms` | reconnect.rs | FAIL (expected) |
| AC-003 | BC-2.05.006 | `test_BC_2_05_006_backoff_cap_at_2000ms` | reconnect.rs | FAIL (expected) |
| AC-004 | BC-2.05.006 | `test_BC_2_05_006_backoff_jitter_within_bounds` | reconnect.rs | FAIL (expected) |
| AC-005 | BC-2.05.006 | `test_BC_2_05_006_reconnect_window_constant` | reconnect.rs | PASS (constant check) |
| AC-006 | BC-2.05.006 | `test_BC_2_05_006_backoff_initial_ms_constant` | reconnect.rs | PASS (constant check) |
| AC-007 | BC-2.05.006 | `test_BC_2_05_006_backoff_cap_ms_constant` | reconnect.rs | PASS (constant check) |
| AC-008 | BC-2.05.006 | `test_BC_2_05_006_offline_poll_interval_constant` | reconnect.rs | PASS (constant check) |
| AC-009 | BC-2.05.006 | `test_BC_2_05_006_reconnect_gives_up_after_window` | reconnect.rs | FAIL (expected) |
| AC-010 | BC-2.05.006 | `test_BC_2_05_006_reconnect_succeeds_resets_attempt_counter` | reconnect.rs | FAIL (expected) |
| AC-011 | BC-2.05.006 | `test_BC_2_05_006_reconnect_state_machine_idle_to_backing_off` | reconnect.rs | FAIL (expected) |
| AC-012 | BC-2.05.006 | `test_BC_2_05_006_reconnect_state_machine_backing_off_to_connecting` | reconnect.rs | FAIL (expected) |
| AC-013 | BC-2.05.006 | `test_BC_2_05_006_reconnect_state_machine_connecting_to_connected` | reconnect.rs | FAIL (expected) |
| AC-014 | BC-2.05.007 | `test_BC_2_05_007_soq3_cleared_on_transport_reconnect` | soq3_overlay_clear.rs | FAIL (expected) |
| AC-015 | BC-2.05.007 | `test_BC_2_05_007_soq3_cleared_on_transport_event_reconnected` | soq3_overlay_clear.rs | FAIL (expected) |

All 15 ACs (AC-001 through AC-015) have at least one BC-traced test. Doc comments in
each test name the BC and AC explicitly.

### Passing Tests (non-vacuous justification)

The 4 passing tests check spec-pinned constants defined in production code
(`crates/monocle-ipc/src/reconnect.rs`):

| Test | Constant | Spec Value |
|------|----------|-----------|
| `test_BC_2_05_006_reconnect_window_constant` | `RECONNECT_WINDOW_SECS` | 5 |
| `test_BC_2_05_006_backoff_initial_ms_constant` | `BACKOFF_INITIAL_MS` | 250 |
| `test_BC_2_05_006_backoff_cap_ms_constant` | `BACKOFF_CAP_MS` | 2000 |
| `test_BC_2_05_006_offline_poll_interval_constant` | `OFFLINE_POLL_INTERVAL_SECS` | 5 |

These are non-vacuous: they verify the stub file declares the correct constant values
as required by BC-2.05.006 (the implementer must not change these values to make
other tests pass).

### First Failure (verbatim)

```
thread 'test_BC_2_05_006_backoff_attempt_1_is_250ms' panicked at crates/monocle-ipc/src/reconnect.rs:97:9:
not yet implemented: S-023: compute exponential backoff delay for attempt 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Vacuous-Mirror-Test Audit

PASSED. Tests invoke real production function paths (`compute_backoff_delay`,
`reconnect_loop`, `on_transport_event`). No test logic duplicates the production
algorithm — tests assert against spec-pinned expected values and state transitions,
not recomputed results.

## Regression Check

| Existing Tests | Status |
|---------------|--------|
| 753 pre-existing tests on develop @ 59f4394 | all pass (verified on worktree base) |

No regressions introduced by the stub commit or failing-tests commit.

## Cross-Story Note

`TransportEvent` is introduced in S-023 (monocle-ipc). S-025 currently holds a local
stub of `TransportEvent` in `crates/monocle-tui/src/app.rs` with a `TODO(S-025)`
comment. Once S-023 merges to develop, S-025's implementer rebase will swap the local
stub for the real monocle-ipc import. No action required during S-023 implementation.

## Hand-Off to Implementer

- Story ready for implementation: S-023
- Implementation order: stubs in `crates/monocle-ipc/src/reconnect.rs` (backoff
  computation first, then state machine, then SOQ-3 overlay clear handler)
- Spec references: BC-2.05.006, BC-2.05.007, SS-ipc v1.8.0 §"Reconnect"
- Constants are pinned — implementer must not alter `BACKOFF_INITIAL_MS`,
  `BACKOFF_CAP_MS`, `RECONNECT_WINDOW_SECS`, `OFFLINE_POLL_INTERVAL_SECS`
- `TransportEvent` constructors must satisfy ADR-0006 (`non_exhaustive` + public
  constructor) discipline from SS-conventions v1.31.0
