---
story_id: S-022
evidence_type: integration-test-capture
product_type: daemon-ipc (no UI)
recording_tool: cargo test --nocapture
generated: 2026-05-28
---

# S-022 Demo Evidence Report

Story: TUI UDS Connection, InitialState Push, and Permission Prompt IPC
Behavioral contracts: BC-2.05.002 (TUI connect + InitialState push), BC-2.05.005 (PermissionPromptQueued broadcast + PermissionDecision routing)

S-022 is a backend-only IPC story. There is no TUI, no CLI prompt loop, and no browser.
Evidence is integration test output from the production handler stack exercised end-to-end.

## Test Suite Summary

| Crate | Test File | Tests Passing |
|-------|-----------|---------------|
| monocle-ipc | tests/connection_handshake.rs | 8 / 8 |
| monocle-ipc | tests/permission_prompt.rs | 9 / 9 |
| monocle-runtime | tests/hook_defer_race.rs | 3 / 3 |
| monocle-runtime | tests/ipc_broadcast.rs | 2 / 2 |
| **Total** | | **22 / 22** |

All 22 tests pass. Zero failures. Zero ignored.

## AC Coverage Map

| AC | BC Clause | Test(s) | Artifact | Result |
|----|-----------|---------|----------|--------|
| AC-001 | BC-2.05.002 PC-1 | `ac_001_per_client_tokio_task_spawned` | AC-001-per-client-tokio-task-spawned.txt | PASS |
| AC-002 | BC-2.05.002 PC-2 | `ac_002_initial_state_is_first_message` | AC-002-initial-state-is-first-message.txt | PASS |
| AC-003 | BC-2.05.002 PC-3 | `ac_003_four_byte_le_framing` + `test_BC_2_05_002_ring_tail_non_empty_passes_through` | AC-003-four-byte-le-framing.txt | PASS |
| AC-004 | BC-2.05.002 PC-4 | `ac_004_initial_state_too_large_closes_connection` | AC-004-initial-state-too-large-closes-connection.txt | PASS |
| AC-005 | BC-2.05.002 PC-5/PC-6 | `ac_005_push_only_no_polling` | AC-005-push-only-no-polling.txt | PASS |
| AC-006 | BC-2.05.002 invariant 3 | `ac_006_no_gap_window_between_snapshot_and_streaming` | AC-006-no-gap-window-snapshot-to-streaming.txt | PASS |
| AC-007 | BC-2.05.005 PC-1 | `ac_007_permission_prompt_queued_broadcast_on_decision_required` + `test_F_S022_ADV12_MED_001_timeout_arm_broadcasts_once_on_normal_timeout` | AC-007-permission-prompt-queued-broadcast.txt | PASS |
| AC-008 | BC-2.05.005 PC-2 | `ac_008_prompt_id_stable_across_queued_and_resolved` | AC-008-prompt-id-stable-queued-to-resolved.txt | PASS |
| AC-009 | BC-2.05.005 PC-3 | `ac_009_permission_decision_routes_to_oneshot` + `ac_009b_permission_decision_unknown_prompt_id_silently_discarded` | AC-009-permission-decision-routing.txt | PASS |
| AC-010 | BC-2.05.005 PC-4 | `ac_010_timeout_broadcasts_resolved_and_removes_registry` + `test_F_S022_ADV12_MED_001_timeout_arm_broadcasts_once_on_normal_timeout` | AC-010-timeout-fail-open-resolved-broadcast.txt | PASS |
| AC-011 | BC-2.05.005 invariant 2 | `ac_011_at_most_one_resolution_via_oneshot` | AC-011-at-most-one-resolution-via-oneshot.txt | PASS |
| AC-012 | BC-2.05.005 invariant 3 | `ac_012_resolved_requires_prior_queued` | AC-012-resolved-requires-prior-queued.txt | PASS |
| AC-013 | BC-2.05.002 EC-001 | `ac_013_empty_initial_state` | AC-013-empty-initial-state.txt | PASS |
| AC-014 | BC-2.05.005 EC-001 | `ac_014_dual_resolution_race` | AC-014-dual-resolution-race.txt | PASS |
| AC-015 | BC-2.05.005 EC-003 | `ac_015_no_clients_connected_for_queued` | AC-015-no-clients-queued-in-overlay-stack.txt | PASS |

Coverage: 15 / 15 ACs demonstrated. No gaps.

## Notes on Test-to-AC Mapping

**AC-007 and AC-010 share a test**: `test_F_S022_ADV12_MED_001_timeout_arm_broadcasts_once_on_normal_timeout` in `monocle-runtime/tests/hook_defer_race.rs` exercises both the PermissionPromptQueued broadcast (AC-007, queued_count==1 assertion) and the timeout-path PermissionPromptResolved broadcast (AC-010, resolved_count==1 assertion) in a single end-to-end pass through the axum handler. Each AC has its own dedicated test in `monocle-ipc/tests/permission_prompt.rs`; the hook_defer_race test provides supplementary evidence of the production path.

**AC-009 has two tests**: The story spec AC-009 covers both the "found" path (routes to oneshot, broadcasts Resolved) and the "not found" path (silently discarded). These are split into `ac_009_...` and `ac_009b_...` respectively. Both are covered in the AC-009 artifact.

**ring_tail non-empty pass-through**: `test_BC_2_05_002_ring_tail_non_empty_passes_through` (synchronous test in connection_handshake.rs) provides supplementary evidence for AC-002/AC-003 that ring_tail records survive snapshot_initial_state without lossy conversion. Driven by architect directive F-S022-ADV2-HIGH-002.

## Raw Test Invocations for pr-manager

```
cargo test -p monocle-ipc --test connection_handshake -- --nocapture
# running 8 tests; test result: ok. 8 passed

cargo test -p monocle-ipc --test permission_prompt -- --nocapture
# running 9 tests; test result: ok. 9 passed

cargo test -p monocle-runtime --test hook_defer_race -- --nocapture
# running 3 tests; test result: ok. 3 passed

cargo test -p monocle-runtime --test ipc_broadcast -- --nocapture
# running 2 tests; test result: ok. 2 passed
```
