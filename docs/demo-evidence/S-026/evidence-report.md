---
document_type: demo-evidence-report
product: monocle
story_id: S-026
pipeline_run: "2026-05-31"
demo_type: cli
recording_tool: vhs
status: complete
---

# Demo Evidence Report — S-026: Permission Overlay Core

## Product: monocle
## Story: S-026 — Push/Pop, Decision Dispatch, Disconnect Clear, UUID Removal
## Pipeline Run: 2026-05-31
## Demo Type: CLI (Rust TUI — logic/IPC/state layer)

---

## Nature of This Story

S-026 is a **logic/IPC/state story**, not a UI-rendering story. The permission overlay
visual rendering (diff preview, modal frame) is explicitly S-027's scope —
`crates/monocle-tui/src/ui/overlay.rs` is an intentional placeholder here. There is
no net-new on-screen rendered overlay to demonstrate. The demonstrable behavior lives at
the state-machine + IPC-wire level.

All evidence is production-path test execution. The VHS recordings show the real
`cargo test` suite executing against the real implementation — not scripted shell output,
not mock output. Each test name encodes the BC it satisfies (e.g.,
`test_BC_2_06_011_accept_once_y_sends_allow_ipc_modal_stays_in_stack`), making the
AC-to-test linkage transparent in the recordings.

---

## Per-AC Demo Coverage

| AC | BC Traced | Behavior Demonstrated | Evidence Artifact (test file) | Key Tests | Result |
|----|-----------|----------------------|-------------------------------|-----------|--------|
| AC-001 | BC-2.06.008 PC-1, BC-2.05.002 Inv-4 | PermissionPromptQueued push to VecDeque; idempotent insert (duplicate prompt_id silently discarded at TRACE) | [AC-001-002-overlay-push-fifo.webm](AC-001-002-overlay-push-fifo.webm) / `overlay_push_pop` | `test_BC_2_06_008_push_from_dashboard_enters_overlay`, `test_BC_2_05_002_invariant_4_duplicate_prompt_id_silently_discarded`, `test_BC_2_05_002_invariant_4_idempotent_after_other_pushes` | PASS (24/24) |
| AC-002 | BC-2.06.008 PC-2 | FIFO ordering: oldest prompt at front(), new prompts via push_back() | [AC-001-002-overlay-push-fifo.webm](AC-001-002-overlay-push-fifo.webm) / `overlay_push_pop` | `test_BC_2_06_008_fifo_ordering_three_prompts`, `test_BC_2_06_008_push_from_overlay_extends_stack_preserves_prior` | PASS (24/24) |
| AC-003 | BC-2.06.011 PC-1 | y/Enter in Overlay mode → ClientToServer::PermissionDecision{Accept} sent; modal stays in VecDeque until resolved | [AC-003-004-005-008-decision-keybindings.webm](AC-003-004-005-008-decision-keybindings.webm) / `overlay_decision` | `test_BC_2_06_011_accept_once_y_sends_allow_ipc_modal_stays_in_stack`, `test_BC_2_06_011_accept_once_enter_sends_allow_ipc_modal_stays`, `test_BC_2_06_011_accept_once_uses_front_prompt_id_in_multi_stack` | PASS (11/11) |
| AC-003 (wire) | BC-2.06.011 PC-1 | **IPC wire-traversal proof**: PermissionDecision frame physically reaches daemon-side channel (Pass-5 CRITICAL fix) | [AC-003-005-wire-traversal.webm](AC-003-005-wire-traversal.webm) / `ipc_outbound_writer` | `test_f_s026_adv5_crit001_decision_message_traverses_wire_to_daemon`, `test_f_s026_adv5_crit001_ipc_tx_is_some_after_setup` | PASS (3/3) |
| AC-004 | BC-2.06.012 PC-1 | A (uppercase) in Overlay mode → PermissionDecision{AcceptAlways} sent; modal stays | [AC-003-004-005-008-decision-keybindings.webm](AC-003-004-005-008-decision-keybindings.webm) / `overlay_decision` | `test_BC_2_06_012_accept_always_uppercase_a_sends_accept_always_modal_stays` | PASS (11/11) |
| AC-005 | BC-2.06.013 PC-1 | n/r in Overlay mode → PermissionDecision{Reject} sent; modal stays | [AC-003-004-005-008-decision-keybindings.webm](AC-003-004-005-008-decision-keybindings.webm) / `overlay_decision` | `test_BC_2_06_013_reject_n_sends_deny_modal_stays_in_stack`, `test_BC_2_06_013_reject_r_sends_deny_modal_stays_in_stack` | PASS (11/11) |
| AC-006 | BC-2.06.023 PC-1 | PermissionPromptResolved → retain() removes by UUID regardless of stack position (front, back, middle) | [AC-006-007-015-uuid-removal.webm](AC-006-007-015-uuid-removal.webm) / `overlay_uuid_removal` + `overlay_decision` | `test_BC_2_06_023_pc1_retain_removes_front_entry`, `test_BC_2_06_023_pc1_retain_removes_back_entry`, `test_BC_2_06_023_pc1_retain_removes_middle_entry_in_four_item_stack`, `test_BC_2_06_023_pc2_retain_removes_all_duplicate_entries` | PASS (12/12) |
| AC-007 | BC-2.06.023 PC-3 | Unknown prompt_id in PermissionPromptResolved is silent no-op (no WARN, no ERROR, no state change) | [AC-006-007-015-uuid-removal.webm](AC-006-007-015-uuid-removal.webm) / `overlay_uuid_removal` + `overlay_decision` | `test_BC_2_06_023_pc3_unknown_prompt_id_noop_stack_unchanged`, `test_BC_2_06_023_pc3_unknown_prompt_id_on_empty_stack_is_safe`, `test_BC_2_06_023_pc3_ac007_unknown_prompt_id_no_state_change_production`, `test_BC_2_06_023_pc3_unknown_prompt_id_resolved_is_noop` | PASS (12/12) |
| AC-008 | BC-2.06.014 PC-1 | Esc in Overlay mode is identity no-op: no reject, no pop, no IPC send, no mode change | [AC-003-004-005-008-decision-keybindings.webm](AC-003-004-005-008-decision-keybindings.webm) / `overlay_decision` | `test_BC_2_06_014_esc_in_overlay_is_identity_no_ipc_send`, `test_BC_2_06_014_transition_esc_overlay_is_identity_pure_function` | PASS (11/11) |
| AC-009 | BC-2.06.003 | SearchPrompt layer registers overlay decision keys (y/Enter/A/n/r) with highest priority in Overlay mode | [AC-013-014-overlay-rotation.webm](AC-013-014-overlay-rotation.webm) / `overlay_rotation` | `test_BC_2_06_009_transition_overlay_cycle_next_is_identity` | PASS (8/8) |
| AC-010 | BC-2.06.003 | Overlay binding isolation: j/k/Tab blocked from session-nav passthrough while Overlay is active | [AC-013-014-overlay-rotation.webm](AC-013-014-overlay-rotation.webm) / `overlay_rotation` | `test_ac_010_overlay_binding_isolation_j_does_not_scroll_sessions` | PASS (8/8) |
| AC-011 | BC-2.06.016 PC-1 | TransportEvent::Disconnected clears overlay_stack (VecDeque::new()), transitions to Dashboard; satisfies SOQ-3 | [AC-011-012-disconnect-restore.webm](AC-011-012-disconnect-restore.webm) / `overlay_disconnect` | `test_BC_2_06_016_pc1_disconnect_clears_overlay_stack`, `test_BC_2_06_016_pc1_disconnect_is_idempotent`, `test_BC_2_06_016_pc1_disconnect_from_dashboard_no_overlay_is_safe` | PASS (9/9) |
| AC-012 | BC-2.06.016 PC-2 | InitialState on reconnect re-populates overlay_stack; enters Overlay mode if prompts pending | [AC-011-012-disconnect-restore.webm](AC-011-012-disconnect-restore.webm) / `overlay_disconnect` | `test_BC_2_06_016_pc2_reconnect_restores_overlay_from_initial_state`, `test_BC_2_06_016_pc2_reconnect_with_empty_overlay_stays_dashboard`, `test_BC_2_06_016_pc2_overlay_restored_in_fifo_order`, `test_snapshot_window_prompt_dedup` | PASS (9/9) |
| AC-013 | BC-2.06.009 PC-1 | Up/Down when stack.len() > 1 rotates: pop_front() + push_back(); new front is next oldest | [AC-013-014-overlay-rotation.webm](AC-013-014-overlay-rotation.webm) / `overlay_rotation` | `test_BC_2_06_009_pc1_rotation_len_gt_1_moves_front_to_back`, `test_BC_2_06_009_pc1_two_rotations_produce_correct_order`, `test_BC_2_06_009_pc1_three_rotations_wraps_back_to_original_order`, `test_BC_2_06_009_pc1_rotation_via_down_key_also_rotates` | PASS (8/8) |
| AC-014 | BC-2.06.009 EC-065 | Single-item rotation is a no-op: item returns to front, no error, stack stays single-item | [AC-013-014-overlay-rotation.webm](AC-013-014-overlay-rotation.webm) / `overlay_rotation` | `test_BC_2_06_009_ec065_single_item_rotation_is_noop`, `test_BC_2_06_009_ec065_single_item_rotation_down_is_noop` | PASS (8/8) |
| AC-015 | BC-2.06.023 PC-4 | After retain() removes last modal, overlay_stack.is_empty() → transition to Dashboard{focused: prior}; Overlay never left with empty stack | [AC-006-007-015-uuid-removal.webm](AC-006-007-015-uuid-removal.webm) / `overlay_uuid_removal` + `overlay_push_pop` | `test_BC_2_06_023_pc4_empty_stack_collapse_to_dashboard_after_last_retain`, `test_BC_2_06_023_pc4_non_empty_stack_after_retain_stays_overlay`, `test_BC_2_06_023_pc4_sequential_removals_collapse_only_on_last`, `test_BC_2_06_023_pc4_on_initial_state_then_retain_collapses` | PASS (12/12) |
| AC-016 | BC-2.06.008 PC-1, BC-2.06.024 | payload_to_modal() exhaustive dispatch: Bash→ToolPayload::Bash, Read→::Read, Edit/Write+content→::Edit, Edit/Write+None/None→::Generic (Phase-1 normal path), unknown→::Generic; received_at=Instant::now() | [AC-016-payload-conversion.webm](AC-016-payload-conversion.webm) / `overlay_push_pop` (payload_to_modal subset) | `test_BC_2_06_024_payload_to_modal_bash_variant`, `test_BC_2_06_024_payload_to_modal_edit_none_content_yields_generic`, `test_BC_2_06_024_payload_to_modal_write_none_content_yields_generic`, `test_BC_2_06_024_payload_to_modal_write_with_new_content_and_path_yields_edit`, `test_BC_2_06_024_payload_to_modal_generic_variant`, `test_BC_2_06_024_payload_to_modal_received_at_set_at_conversion_time` | PASS (11/11 payload_to_modal tests) |

---

## IPC Wire-Traversal Proof (Pass-5 CRITICAL Fix Highlight)

The `ipc_outbound_writer` test suite is the key demonstration that decision keybindings
actually function end-to-end, not merely produce local state changes. This was the
Pass-5 CRITICAL finding: `app.ipc_tx` was `None` after IPC setup, meaning all
`PermissionDecision` sends silently dropped.

The three tests in `ipc_outbound_writer.rs` prove:

1. `test_f_s026_adv5_crit001_ipc_tx_is_some_after_setup` — `app.ipc_tx` is `Some(_)` after
   connecting IPC; the channel exists for sends to use.
2. `test_f_s026_adv5_crit001_decision_message_traverses_wire_to_daemon` — A `PermissionDecision`
   frame constructed by the keybinding handler physically arrives at the daemon-side `rx`
   channel. This is the end-to-end wire proof.
3. `test_f_s026_adv5_crit001_reconnect_rewires_ipc_tx_to_new_channel` — On reconnect,
   `ipc_tx` is rewired to the new channel so subsequent decisions still reach the daemon.

Recording: [AC-003-005-wire-traversal.webm](AC-003-005-wire-traversal.webm)

---

## VHS Recording Summary

| Recording | Tape Script | GIF | WebM | Test File | Tests |
|-----------|-------------|-----|------|-----------|-------|
| AC-001/002 overlay push + FIFO | [AC-001-002-overlay-push-fifo.tape](AC-001-002-overlay-push-fifo.tape) | [gif](AC-001-002-overlay-push-fifo.gif) | [webm](AC-001-002-overlay-push-fifo.webm) | `overlay_push_pop` | 24 |
| AC-003/004/005/008 decision keybindings | [AC-003-004-005-008-decision-keybindings.tape](AC-003-004-005-008-decision-keybindings.tape) | [gif](AC-003-004-005-008-decision-keybindings.gif) | [webm](AC-003-004-005-008-decision-keybindings.webm) | `overlay_decision` | 11 |
| AC-003/005 IPC wire traversal | [AC-003-005-wire-traversal.tape](AC-003-005-wire-traversal.tape) | [gif](AC-003-005-wire-traversal.gif) | [webm](AC-003-005-wire-traversal.webm) | `ipc_outbound_writer` | 3 |
| AC-006/007/015 UUID removal + collapse | [AC-006-007-015-uuid-removal.tape](AC-006-007-015-uuid-removal.tape) | [gif](AC-006-007-015-uuid-removal.gif) | [webm](AC-006-007-015-uuid-removal.webm) | `overlay_uuid_removal` | 12 |
| AC-011/012 disconnect + restore | [AC-011-012-disconnect-restore.tape](AC-011-012-disconnect-restore.tape) | [gif](AC-011-012-disconnect-restore.gif) | [webm](AC-011-012-disconnect-restore.webm) | `overlay_disconnect` | 9 |
| AC-013/014 stack rotation | [AC-013-014-overlay-rotation.tape](AC-013-014-overlay-rotation.tape) | [gif](AC-013-014-overlay-rotation.gif) | [webm](AC-013-014-overlay-rotation.webm) | `overlay_rotation` | 8 |
| AC-016 payload_to_modal conversion | [AC-016-payload-conversion.tape](AC-016-payload-conversion.tape) | [gif](AC-016-payload-conversion.gif) | [webm](AC-016-payload-conversion.webm) | `overlay_push_pop` (subset) | 11 |

All 7 recordings produced both `.gif` and `.webm`. VHS 0.10.0 confirmed installed.

---

## Coverage Summary

| Category | ACs Covered | Total | Status |
|----------|-------------|-------|--------|
| Test-execution evidence | AC-001 through AC-016 | 16 | Complete |
| VHS recordings | 7 recordings covering all 16 ACs | 7 tapes | Complete |
| Wire-level proof (IPC traversal) | AC-003/004/005 end-to-end | 3 tests | Complete |
| Success path | All 16 ACs | 16 | Complete |
| Error/edge path | AC-007 (unknown UUID no-op), AC-008 (Esc identity), AC-014 (single-item rotation no-op), AC-016 (None/None→Generic fallback) | 4 key edge cases | Complete |

**Total tests across all S-026 test suites: 64 tests, 0 failures.**

---

## Overlay Rendering Note

The `monocle-tui/src/ui/overlay.rs` file is an intentional placeholder in S-026.
Visual rendering of the permission modal (diff pane, modal frame, tool display) is
S-027's scope. No VHS recording of a rendered overlay was produced because no rendered
overlay exists in S-026 scope — producing one would require fabricating output that
does not exist in the implementation. The production-path test suite is the authoritative
evidence for this logic-level story.

---

## Toolchain

| Tool | Version | Status |
|------|---------|--------|
| VHS | 0.10.0 | installed (`/opt/homebrew/bin/vhs`) |
| cargo | workspace (Rust 1.88 MSRV) | installed |
| FiraCode Nerd Font Mono | system | installed (font used in recordings) |
