# S-031 Demo Evidence Report

## Story: Profile Picker — Profile Selection Widget and Config Save

**Story ID:** S-031  
**Epic:** EPIC-07  
**BCs:** BC-2.07.004, BC-2.07.005  
**Points:** 5  
**Wave:** 7 (Final)

---

## Recording Medium

**CLI product (Rust TUI):** VHS terminal recordings — `cargo test` runs targeting production
TestBackend render tests and behavioral unit tests. The profile picker requires a live daemon +
App state to reach interactively, so the strongest available evidence is the comprehensive
TestBackend render harness (render_frame_integration_s031.rs, profile_picker.rs,
profile_picker_adv_pass2.rs, profile_picker_adv_pass4.rs, profile_switch.rs). This is the
same approach used for S-025, S-026, S-027, S-028.

**Live-binary capture status:** Not available without a running daemon. All production widget
logic is exercised via TestBackend (ratatui TestBackend renders the actual production widgets
to a terminal buffer). No evidence is fabricated.

---

## Test Suite Summary

| Suite | Tests | Status |
|-------|-------|--------|
| `profile_picker.rs` | 21 | PASS |
| `profile_switch.rs` | 8 | PASS |
| `render_frame_integration_s031.rs` | 19 | PASS |
| `profile_picker_adv_pass2.rs` | 3 | PASS |
| `profile_picker_adv_pass4.rs` | 1 | PASS |
| **Total** | **52** | **PASS** |

Full output: `all-tests.log`

---

## AC Coverage Map

| AC | BC | Recording | Tests Exercised | Live Binary | Notes |
|----|----|-----------|-----------------|----|-------|
| AC-001 | BC-2.07.005 PC-1 | `AC-001-ctrl-p-opens-picker.{gif,webm,tape}` | `ctrl_p_sets_profile_picker_some`, `open_picker_does_not_change_app_mode`, `ctrl_p_idempotent_when_picker_already_open`, `picker_profiles_sorted_alphabetically` | No — TestBackend | Ctrl-P sets `app.profile_picker = Some(..)` without changing AppMode; idempotent on double-open (EC-110) |
| AC-002 | BC-2.07.005 PC-2/PC-3 | `AC-002-modal-render-profile-list.{gif,webm,tape}` | `render_frame_renders_picker_modal_when_picker_is_some`, `render_frame_picker_modal_shows_profile_list`, `render_frame_picker_modal_shows_no_profiles_message`, `active_marker_uses_per_dir_not_first_match` | No — TestBackend | Modal renders "Profile Picker" title + profile IDs; empty-profiles shows exact em-dash BC literal; `*` marker uses per-dir lookup |
| AC-003 | BC-2.07.004 PC-3 | `AC-003-navigation-wrap.{gif,webm,tape}` | `picker_select_next_increments_index`, `picker_select_next_wraps_to_top`, `picker_select_prev_decrements_index`, `picker_select_prev_wraps_to_bottom`, `navigation_noop_on_empty_profiles` | No — TestBackend | j/down increments + wraps to top; k/up decrements + wraps to bottom; no-op on empty list |
| AC-004 | BC-2.07.004 INV-4 / BC-2.07.005 PC-9 | `AC-004-keyboard-isolation.{gif,webm,tape}` | `picker_open_down_arrow_routes_to_picker_not_session_scroll`, `picker_open_tab_key_isolated_does_not_move_focus`, `picker_open_esc_routes_to_close_picker` | No — TestBackend | dispatch_key_event consumes all keys while picker open; Down routes to picker_select_next not session scroll; Tab does not mutate AppMode focus |
| AC-005 | BC-2.07.005 PC-5a/b | `AC-005-enter-commit-atomic-write.{gif,webm,tape}` | `commit_selection_closes_picker`, `commit_selection_writes_project_profiles_entry`, `picker_open_enter_commits_and_closes_picker`, `ec108_select_same_profile_is_idempotent` | No — TestBackend | Enter closes picker + writes `project_profiles[current_dir]`; dispatch_key_event routes Enter to commit; idempotent re-select (EC-108) |
| AC-006 | BC-2.07.005 PC-5c / INV-3 | `AC-006-write-failure-in-memory.{gif,webm,tape}` | `pc5c_write_failure_applies_in_memory_and_sets_status_message`, `commit_production_empty_cwd_does_not_write_empty_dir_key`, `commit_wrapper_err_branch_empty_cwd_does_not_insert_empty_key` | No — TestBackend | write failure: in-memory applied + status_message set + picker closes; empty-CWD guard prevents `project_profiles[""]` write (MAJOR-2 + ADV pass-4 regression) |
| AC-007 | BC-2.07.005 PC-5 / PC-3 | `AC-007-ccr-path-status-bar.{gif,webm,tape}` | `detect_ccr_called_and_ccr_path_updated_after_switch`, `startup_ccr_path_initialized_from_detect_ccr`, `render_frame_status_bar_shows_ccr_path_when_some`, `render_frame_status_bar_shows_ccr_none_when_absent` | No — TestBackend | detect_ccr called after commit and at startup; status bar renders "CCR: \<path\>" or "CCR: none" |
| AC-008 | BC-2.07.004 INV-1 | `AC-008-not-appmode-overlay.{gif,webm,tape}` | `invariant_picker_is_not_app_mode_overlay`, `dispatch_ctrl_p_from_overlay_opens_picker_appmode_unchanged`, `dispatch_ctrl_p_from_filtering_opens_picker` | No — TestBackend | picker is `Option<ProfilePickerState>` not `AppMode::Overlay`; Ctrl-P fires in ALL AppModes (INV-1) without mode guard |
| AC-009 | BC-2.07.005 INV-2 | `AC-009-atomic-write-tempfile.{gif,webm,tape}` | `invariant_atomic_write_via_write_config`, grep for `std::fs::write`, monocle-config test suite | No — TestBackend | `write_config()` uses `tempfile::persist`; no direct `std::fs::write` in TUI profile code; round-trip write+read succeeds |
| AC-010 | BC-2.07.005 PC-1/PC-7/PC-8 | `AC-010-integration-dispatch-render.{gif,webm,tape}` | `dispatch_ctrl_p_from_dashboard_opens_picker`, `render_frame_renders_picker_modal_when_picker_is_some`, `per_directory_preselection_uses_current_dir_not_first_match`, `invariant_write_read_normalization_verbatim_consistent` | No — TestBackend | Full integration: Ctrl-P dispatch → open, render_frame renders picker, per-dir pre-selection (not first-match), normalization round-trip |

---

## Error Path Coverage

| AC | Error Path | Test | Outcome |
|----|-----------|------|---------|
| AC-001 | Double Ctrl-P (EC-110) | `ctrl_p_idempotent_when_picker_already_open` | PASS — no-op, state preserved |
| AC-002 | Empty harness_profiles (EC-106) | `render_frame_picker_modal_shows_no_profiles_message` | PASS — exact em-dash literal rendered |
| AC-003 | Navigation on empty list | `navigation_noop_on_empty_profiles` | PASS — no panic, index stays 0 |
| AC-002 | Dangling profile ID (EC-100) | `invariant_dangling_entry_no_panic` | PASS — None returned, no panic |
| AC-005 | Re-selecting same profile (EC-108) | `ec108_select_same_profile_is_idempotent` | PASS — write called, picker closes |
| AC-006 | write_config path failure (PC-5c) | `pc5c_write_failure_applies_in_memory_and_sets_status_message` | PASS — in-memory applied, status_message set |
| AC-006 | Empty CWD commit (MAJOR-2) | `commit_production_empty_cwd_does_not_write_empty_dir_key` | PASS — no `project_profiles[""]` write |
| AC-006 | Wrapper err-branch empty CWD (ADV pass-4) | `commit_wrapper_err_branch_empty_cwd_does_not_insert_empty_key` | PASS — guard hoisted before config_path() |
| AC-003 | Trailing slash mismatch (INV-1) | `invariant_trailing_slash_mismatch_is_a_miss` | PASS — miss returned, normalization consistent |
| AC-002 | Empty string profile ID (EC-105) | `ec105_empty_string_profile_id_returns_none` | PASS — None returned |

---

## ADV Pass Coverage

| ADV Pass | Finding | Test File | Status |
|----------|---------|-----------|--------|
| Pass-1 (render_frame_integration_s031) | INTEGRATION-1: Ctrl-P dispatch registration | `render_frame_integration_s031.rs` | PASS |
| Pass-1 | INTEGRATION-2: render_frame picker branch | `render_frame_integration_s031.rs` | PASS |
| Pass-1 | INTEGRATION-3: picker-local key routing before resolve_binding | `render_frame_integration_s031.rs` | PASS |
| Pass-1 | INTEGRATION-4/5: ccr_path startup + status bar | `render_frame_integration_s031.rs` | PASS |
| Pass-1 | MAJOR-1: per-dir pre-selection via open_profile_picker_with_dir | `render_frame_integration_s031.rs` | PASS |
| Pass-1 | EC-106: empty-list commit no-op | `render_frame_integration_s031.rs` | PASS |
| Pass-1 | PC-5c: write-failure seam | `render_frame_integration_s031.rs` | PASS |
| Pass-2 | BLOCKER-1: Step-2 or_else fallback removed | `profile_picker_adv_pass2.rs` | PASS |
| Pass-2 | BLOCKER-2: `*` marker uses per-dir current_dir | `profile_picker_adv_pass2.rs` | PASS |
| Pass-2 | MAJOR-2: empty-CWD commit guard | `profile_picker_adv_pass2.rs` | PASS |
| Pass-4 | MAJOR-1 (wrapper): err-branch empty-CWD guard hoisted | `profile_picker_adv_pass4.rs` | PASS |

---

## ACs Lacking Live-Binary Capture — Justification

All 10 ACs lack live-binary capture. Justification: The profile picker (`Option<ProfilePickerState>`)
is an in-process TUI overlay requiring a connected daemon and running event loop. Launching the
full monocle binary to the picker state requires:

1. A running `monocled` daemon (network service, platform-dependent socket path).
2. The TUI event loop responding to key events in a real terminal.
3. Harness profiles pre-configured in `config.json`.

This infrastructure is not available in the recording environment. TestBackend renders use the
**exact same production widget code** (`render_profile_picker`, `dispatch_key_event`,
`open_profile_picker`, `commit_profile_selection_with_path`) against a TestBackend terminal buffer,
providing behavioral verification equivalent to the live binary for all stateful paths. The
S-025/S-026/S-027/S-028 precedent applies: all prior TUI stories use the same TestBackend approach.

---

## File Inventory

| File | Type | AC |
|------|------|----|
| `AC-001-ctrl-p-opens-picker.gif` | VHS recording | AC-001 |
| `AC-001-ctrl-p-opens-picker.webm` | VHS recording | AC-001 |
| `AC-001-ctrl-p-opens-picker.tape` | VHS script | AC-001 |
| `AC-002-modal-render-profile-list.gif` | VHS recording | AC-002 |
| `AC-002-modal-render-profile-list.webm` | VHS recording | AC-002 |
| `AC-002-modal-render-profile-list.tape` | VHS script | AC-002 |
| `AC-003-navigation-wrap.gif` | VHS recording | AC-003 |
| `AC-003-navigation-wrap.webm` | VHS recording | AC-003 |
| `AC-003-navigation-wrap.tape` | VHS script | AC-003 |
| `AC-004-keyboard-isolation.gif` | VHS recording | AC-004 |
| `AC-004-keyboard-isolation.webm` | VHS recording | AC-004 |
| `AC-004-keyboard-isolation.tape` | VHS script | AC-004 |
| `AC-005-enter-commit-atomic-write.gif` | VHS recording | AC-005 |
| `AC-005-enter-commit-atomic-write.webm` | VHS recording | AC-005 |
| `AC-005-enter-commit-atomic-write.tape` | VHS script | AC-005 |
| `AC-006-write-failure-in-memory.gif` | VHS recording | AC-006 |
| `AC-006-write-failure-in-memory.webm` | VHS recording | AC-006 |
| `AC-006-write-failure-in-memory.tape` | VHS script | AC-006 |
| `AC-007-ccr-path-status-bar.gif` | VHS recording | AC-007 |
| `AC-007-ccr-path-status-bar.webm` | VHS recording | AC-007 |
| `AC-007-ccr-path-status-bar.tape` | VHS script | AC-007 |
| `AC-008-not-appmode-overlay.gif` | VHS recording | AC-008 |
| `AC-008-not-appmode-overlay.webm` | VHS recording | AC-008 |
| `AC-008-not-appmode-overlay.tape` | VHS script | AC-008 |
| `AC-009-atomic-write-tempfile.gif` | VHS recording | AC-009 |
| `AC-009-atomic-write-tempfile.webm` | VHS recording | AC-009 |
| `AC-009-atomic-write-tempfile.tape` | VHS script | AC-009 |
| `AC-010-integration-dispatch-render.gif` | VHS recording | AC-010 |
| `AC-010-integration-dispatch-render.webm` | VHS recording | AC-010 |
| `AC-010-integration-dispatch-render.tape` | VHS script | AC-010 |
| `all-tests.log` | Test output log | All ACs |
| `evidence-report.md` | This report | All ACs |
