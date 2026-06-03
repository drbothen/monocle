---
story_id: S-031
story_title: "Profile Picker — Profile Selection Widget and Config Save"
wave: 7
evidence_method: VHS (ratatui TestBackend test-suite capture)
produced_by: vsdd-factory:demo-recorder
date: 2026-06-03
---

# S-031 Demo Evidence Report

## Method

monocle is a TUI application (ratatui). Evidence is captured via VHS recordings
that drive the real compiled binary's test suites against the production ratatui
`TestBackend` render path and the production `dispatch_key_event` / `render_frame`
paths. Integration tests in `render_frame_integration_s031.rs` exercise the full
event-loop wiring (AC-010) — the same path validated by AC-010's integration
requirement. This is the correct evidence vehicle for ratatui TUI products.

## Artifacts

| Artifact | Format | ACs Covered |
|----------|--------|-------------|
| `AC-001-004-008-picker-open-nav.gif` | GIF (embed) | AC-001, AC-002, AC-003, AC-004, AC-008 |
| `AC-001-004-008-picker-open-nav.webm` | WebM (archival) | AC-001, AC-002, AC-003, AC-004, AC-008 |
| `AC-001-004-008-picker-open-nav.tape` | VHS source | — |
| `AC-005-010-switch-config-ccr-integration.gif` | GIF (embed) | AC-005, AC-006, AC-007, AC-009, AC-010 |
| `AC-005-010-switch-config-ccr-integration.webm` | WebM (archival) | AC-005, AC-006, AC-007, AC-009, AC-010 |
| `AC-005-010-switch-config-ccr-integration.tape` | VHS source | — |

## AC Coverage Mapping

### AC-001 (BC-2.07.005 PC-1 — profile picker entry via Ctrl-P)
**Evidence:** `AC-001-004-008-picker-open-nav.gif`
**Tests (profile_picker.rs):**
- `test_BC_2_07_005_ctrl_p_sets_profile_picker_some` — Ctrl-P sets `app.profile_picker = Some(...)`
- `test_BC_2_07_005_open_picker_does_not_change_app_mode` — app.mode unchanged while picker open
- `test_BC_2_07_005_ctrl_p_idempotent_when_picker_already_open` — EC-110 no-op if already open
**Path:** success path
**Status:** PASS (21/21 profile_picker tests)

### AC-002 (BC-2.07.005 PC-2 — profile list rendering)
**Evidence:** `AC-001-004-008-picker-open-nav.gif`
**Tests (profile_picker.rs + render_frame_integration_s031.rs):**
- `test_BC_2_07_004_picker_profiles_sorted_alphabetically` — profiles sorted by display_name
- `test_BC_2_07_005_picker_opens_with_empty_profiles` — empty harness_profiles opens picker
- `test_BC_2_07_004_ec097_first_launch_empty_config` — empty config edge case
**Tests (render_frame_integration_s031.rs):**
- `test_BC_2_07_005_render_frame_renders_picker_modal_when_picker_is_some` — modal in render output
- `test_BC_2_07_005_render_frame_picker_modal_shows_profile_list` — profile display_names in modal
- `test_BC_2_07_005_render_frame_picker_modal_shows_no_profiles_message` — empty-profiles message
- `test_BC_2_07_005_per_directory_preselection_uses_current_dir_not_first_match` — sticky pre-select per dir
- `test_BC_2_07_005_per_directory_preselection_for_dir_a_selects_profile_x` — resolve_profile_for_dir
**Path:** success + error (empty profiles message) paths
**Status:** PASS

### AC-003 (BC-2.07.004 PC-3 — profile picker navigation)
**Evidence:** `AC-001-004-008-picker-open-nav.gif`
**Tests (profile_picker.rs):**
- `test_BC_2_07_004_picker_select_next_increments_index` — j/down increments
- `test_BC_2_07_004_picker_select_next_wraps_to_top` — wrap at bottom
- `test_BC_2_07_004_picker_select_prev_decrements_index` — k/up decrements
- `test_BC_2_07_004_picker_select_prev_wraps_to_bottom` — wrap at top
- `test_BC_2_07_004_esc_closes_picker_without_change` — Esc sets profile_picker = None without write
- `test_BC_2_07_005_esc_does_not_change_app_mode` — AppMode unchanged on Esc
- `test_BC_2_07_004_navigation_noop_on_empty_profiles` — nav safe when no profiles
**Path:** success + error (navigation on empty list) paths
**Status:** PASS

### AC-004 (BC-2.07.004 PC-4 — picker modal keyboard isolation)
**Evidence:** `AC-001-004-008-picker-open-nav.gif`
**Tests (render_frame_integration_s031.rs):**
- `test_BC_2_07_005_picker_open_tab_key_isolated_does_not_move_focus` — Tab does NOT fire session nav
- `test_BC_2_07_005_picker_open_down_arrow_routes_to_picker_not_session_scroll` — down goes to picker
- `test_BC_2_07_005_picker_open_esc_routes_to_close_picker` — Esc routed to picker close
- `test_BC_2_07_005_picker_open_enter_commits_and_closes_picker` — Enter routed to picker commit
**Path:** success + error (isolation: keys that should NOT fire) paths
**Status:** PASS

### AC-005 (BC-2.07.005 PC-5 — profile switch saves config atomically)
**Evidence:** `AC-005-010-switch-config-ccr-integration.gif`
**Tests (profile_switch.rs):**
- `test_BC_2_07_005_commit_selection_writes_project_profiles_entry` — project_profiles[dir] written
- `test_BC_2_07_005_commit_selection_closes_picker` — picker closes on Enter
- `test_BC_2_07_005_active_profile_highlighted_as_default_selection` — sticky selection respected
- `test_BC_2_07_005_ec108_select_same_profile_is_idempotent` — same profile EC-108
- `test_BC_2_07_005_ec110_second_ctrl_p_no_extra_picker` — EC-110 idempotent open
**Tests (render_frame_integration_s031.rs):**
- `test_BC_2_07_005_picker_open_enter_commits_and_closes_picker` — Enter in integration context
- `test_BC_2_07_005_active_profile_highlighted_two_entry_per_dir_not_first_match` — per-dir pre-select
**Path:** success path
**Status:** PASS (8/8 profile_switch; 20/20 render_frame_integration_s031)

### AC-006 (BC-2.07.005 PC-5c — config save error display)
**Evidence:** `AC-005-010-switch-config-ccr-integration.gif`
**Tests (render_frame_integration_s031.rs):**
- `test_BC_2_07_005_pc5c_write_failure_applies_in_memory_and_sets_status_message` — write failure:
  in-memory profile applied, "Config save failed: <error>" in status_message, picker closes
**Tests (profile_switch.rs):**
- `test_BC_2_07_005_ec106_ctrl_p_with_empty_profiles_opens_picker` — empty profiles edge case
**Path:** error path (write failure: in-memory apply + status bar error)
**Status:** PASS

### AC-007 (BC-2.07.005 PC-5 — detect_ccr on switch)
**Evidence:** `AC-005-010-switch-config-ccr-integration.gif`
**Tests (profile_switch.rs):**
- `test_BC_2_07_005_detect_ccr_called_and_ccr_path_updated_after_switch` — detect_ccr called, ccr_path updated
**Tests (render_frame_integration_s031.rs):**
- `test_BC_2_07_005_startup_ccr_path_initialized_from_detect_ccr` — ccr_path set at startup
- `test_BC_2_07_005_render_frame_status_bar_shows_ccr_path_when_some` — "CCR: <path>" in status bar
- `test_BC_2_07_005_render_frame_status_bar_shows_ccr_none_when_absent` — "CCR: none" when absent
**Path:** success + error (ccr not found) paths
**Status:** PASS

### AC-008 (BC-2.07.004 INV-1 — picker is not AppMode::Overlay)
**Evidence:** `AC-001-004-008-picker-open-nav.gif`
**Tests (profile_picker.rs):**
- `test_BC_2_07_004_invariant_picker_is_not_app_mode_overlay` — Option<ProfilePickerState> model
- `test_BC_2_07_004_invariant_resolve_is_pure_no_side_effects` — resolve_profile_for_dir is pure
- `test_BC_2_07_004_invariant_dangling_entry_no_panic` — dangling project_profiles entry safe
- `test_BC_2_07_004_invariant_trailing_slash_mismatch_is_a_miss` — trailing slash invariant
- `test_BC_2_07_004_ec105_empty_string_profile_id_returns_none` — empty id edge case
**Path:** invariant (architectural constraint, not happy/error split)
**Status:** PASS

### AC-009 (BC-2.07.005 INV-2 — atomic write required)
**Evidence:** `AC-005-010-switch-config-ccr-integration.gif`
**Tests (profile_switch.rs):**
- `test_BC_2_07_005_invariant_atomic_write_via_write_config` — tempfile::persist invariant; verifies
  write_config() is called (not std::fs::write directly)
**Path:** invariant (atomic write enforcement)
**Status:** PASS

### AC-010 (BC-2.07.005 PC-1,7,8 — integration: key dispatch, render, navigation, dismiss)
**Evidence:** `AC-005-010-switch-config-ccr-integration.gif`
**Tests (render_frame_integration_s031.rs):**
- `test_BC_2_07_005_dispatch_ctrl_p_from_dashboard_opens_picker` — Ctrl-P in Dashboard
- `test_BC_2_07_005_dispatch_ctrl_p_from_overlay_opens_picker_appmode_unchanged` — Ctrl-P in Overlay; mode unchanged
- `test_BC_2_07_005_dispatch_ctrl_p_from_filtering_opens_picker` — Ctrl-P in Filtering
- `test_BC_2_07_005_picker_open_down_arrow_routes_to_picker_not_session_scroll` — isolation routing
- `test_BC_2_07_005_picker_open_esc_routes_to_close_picker` — Esc → close without write_config
- `test_BC_2_07_005_picker_open_enter_commits_and_closes_picker` — Enter → commit + close
- `test_BC_2_07_005_render_frame_renders_picker_modal_when_picker_is_some` — render path wired
- `test_BC_2_07_005_startup_ccr_path_initialized_from_detect_ccr` — ccr_path at startup
- `test_BC_2_07_005_ec106_empty_profiles_commit_does_not_write_project_profiles_entry` — empty EC
- `test_BC_2_07_005_ec106_empty_profiles_esc_closes_without_write` — Esc on empty EC
- `test_BC_2_07_004_invariant_write_read_normalization_verbatim_consistent` — dir key consistency
**Path:** success + error (empty profiles, Esc without write, dispatch from all AppModes) paths
**Status:** PASS (20/20 render_frame_integration_s031 tests)

## Summary

| AC | Status | Evidence Artifact | Path |
|----|--------|-------------------|------|
| AC-001 | PASS | `AC-001-004-008-picker-open-nav.gif` | success |
| AC-002 | PASS | `AC-001-004-008-picker-open-nav.gif` | success + error |
| AC-003 | PASS | `AC-001-004-008-picker-open-nav.gif` | success + error |
| AC-004 | PASS | `AC-001-004-008-picker-open-nav.gif` | success + isolation |
| AC-005 | PASS | `AC-005-010-switch-config-ccr-integration.gif` | success |
| AC-006 | PASS | `AC-005-010-switch-config-ccr-integration.gif` | error (write failure) |
| AC-007 | PASS | `AC-005-010-switch-config-ccr-integration.gif` | success + error |
| AC-008 | PASS | `AC-001-004-008-picker-open-nav.gif` | invariant |
| AC-009 | PASS | `AC-005-010-switch-config-ccr-integration.gif` | invariant |
| AC-010 | PASS | `AC-005-010-switch-config-ccr-integration.gif` | success + error |

All 10 ACs have recorded evidence. No ACs without evidence.
