---
story_id: S-046
title: PtyOutput Fan-out Broker — Bounded Channel, Backpressure, and Client Lifecycle
version: "1.0"
produced_by: vsdd-factory:demo-recorder
date: 2026-06-22
---

# S-046 Demo Evidence Report

## Story Summary

S-046 implements the PTY output fan-out broker for the monocle daemon (BC-2.05.009 / BC-2.05.011):
a `PtyBroker` struct that owns a bounded INPUT channel (`tokio::mpsc::channel::<Arc<Bytes>>(1024)`)
between the proxy task and the broker event loop; fan-out to all connected TUI clients via
`broadcast_to_subscribers(&shared_subscriber_list, msg)` on the daemon's single `Arc<SubscriberList>`
(the same list used for all daemon-to-TUI fan-out, not a per-broker duplicate registry);
1-strike slow-client disconnect via `broadcast_to_subscribers` semantics (BC-2.05.004 EC-005);
`pty_drop_counter` (`Arc<AtomicU64>`) incremented ONLY in the proxy task on `tx.send().await` `Err`
(not on backpressure, not on graceful close, not on per-client broadcast failure);
`PtyReset` emission on exactly two triggers (session-host `HostToDaemon::PtyReset` or proxy
`tx.send` `Err`) but NOT on graceful `input_rx.recv() == None`; `biased;` `select!` so hook/control
events are never starved by PTY volume; and zero `unbounded_channel` calls in the PTY fan-out path.

The `ServerToClient::PtyReset { session_id: String }` variant (BC-2.05.011) is also added in this
story as the daemon-side emission point; the TUI-side handler (scrollback clear + AttachSession
re-trigger) is S-047 scope.

All 8 acceptance criteria across BC-2.05.009 postconditions, invariants, and source guards are
covered by 8 passing tests.

## Coverage Map

| Recording | Acceptance Criteria | BCs Covered | Tests |
|-----------|--------------------|-----------:|------:|
| AC-001-pty-broker-unit-tests.webm | AC-001..AC-007 (+ AC-004 dual sub-case) | BC-2.05.009 | 8 (full suite) |

**Total: 8 tests passing, 0 failures** (1 test file: `pty_broker.rs`).

## Recording Details

### AC-001 — Full Test Suite: PTY Fan-out Broker Unit Tests

**File:** `AC-001-pty-broker-unit-tests.webm`
**Tape:** `AC-001-pty-broker-unit-tests.tape`

Runs `cargo test -p monocle-runtime --test pty_broker` (full 8-test suite):

**Bounded INPUT channel backpressure (BC-2.05.009 Postcondition 1):**
- `test_BC_2_05_009_bounded_channel_backpressure_blocks_not_drops` — AC-001: fills the 1024-capacity INPUT channel; verifies `try_send` fails with `TrySendError::Full` (channel is bounded, not unbounded); confirms `.send().await` would block, not drop

**Fan-out via shared SubscriberList (BC-2.05.009 Postcondition 1b):**
- `test_BC_2_05_009_fan_out_via_subscriber_list_not_broker_registry` — AC-002: registers two subscribers via `register_subscriber` + `SubscriberList`; sends one PTY frame through the broker; asserts both client channels receive `ServerToClient::PtyOutput` with correct `session_id` and `bytes`

**1-strike disconnect (BC-2.05.009 Postcondition 1b / BC-2.05.004 EC-005):**
- `test_BC_2_05_009_one_strike_disconnect_slow_client` — AC-003: fills one client's per-client channel (capacity 64) then sends one more frame; verifies slow client is removed immediately by `broadcast_to_subscribers`; verifies other client is unaffected

**pty_drop_counter not incremented on graceful close or broadcast failure (BC-2.05.009 Postcondition 2/3):**
- `test_BC_2_05_009_pty_drop_counter_not_incremented_on_graceful_close` — AC-004 sub-case A: drops INPUT channel sender (graceful session exit path `input_rx.recv() == None`); asserts `pty_drop_counter` stays at 0
- `test_BC_2_05_009_pty_drop_counter_not_incremented_on_broadcast_failure` — AC-004 sub-case B: simulates per-client channel full (slow client drop via `broadcast_to_subscribers`); asserts `pty_drop_counter` stays at 0

**PtyReset NOT emitted on graceful input close (BC-2.05.009 Invariant 4):**
- `test_BC_2_05_009_pty_reset_not_emitted_on_graceful_input_close` — AC-005: closes INPUT channel sender (proxy task exits); waits for broker event loop to exit; asserts no `ServerToClient::PtyReset` was sent to registered subscriber

**biased-select source-guard (BC-2.05.009 Invariant 6):**
- `test_BC_2_05_009_biased_select_source_guard` — AC-006: reads `crates/monocle-runtime/src/pty_broker.rs`; asserts `biased;` is present in the `select!` macro body (static source guard — behavioral ordering not observable at unit-test boundary; see AC-006 commentary in test file)

**No unbounded_channel source-guard (BC-2.05.009 Invariant 3):**
- `test_BC_2_05_009_no_unbounded_channel_in_pty_path` — AC-007: reads `crates/monocle-runtime/src/pty_broker.rs`; asserts `unbounded_channel` does not appear anywhere in the file (grep-level compile-time contract)

## Integration Caveat

Live end-to-end PTY streaming (session-host → proxy task → broker INPUT channel → broker event loop →
`broadcast_to_subscribers` → TUI client) is verified at the **EPIC-09 integration gate** after S-047
(the TUI-side `PtyReset` handler) is delivered. S-046's evidence boundary is the broker unit and
contract behavior — the 8 tests above cover every behavioral contract from BC-2.05.009 that is
verifiable at the unit-test boundary. The integration path requires a running daemon, a connected
session-host, and a live PTY writer, which are outside the unit-test scope for this story.

This is the same honest evidence boundary used by S-039, S-040, S-041, S-042, and S-043.

## What Is Not Demonstrated

The full live PTY streaming path and the TUI-side `PtyReset` → scrollback-clear → `AttachSession`
re-trigger round-trip are S-047/S-048 scope. The `ServerToClient::PtyReset` variant is defined in
this story (`monocle-ipc/src/lib.rs`) but the TUI handler that reacts to it is S-047.

## Format Notes

- Output format: WEBM only (no GIF — project demo policy for Wave 9).
- Font: `FiraCode Nerd Font Mono` (detected via `fc-list`).
- VHS version: 0.11.0.
- `Sleep 30s` used for the full suite (8 tests, ~0.06s runtime with precompiled binary).
- `Wait+Line` not used: fast pre-compiled test binaries complete before VHS line-scanner triggers.
