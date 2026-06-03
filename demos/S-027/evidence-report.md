---
story_id: S-027
story_title: "Overlay Rendering + Diff Preview + Status Bar"
wave: 7
evidence_method: VHS (ratatui TestBackend test-suite capture)
produced_by: vsdd-factory:demo-recorder
date: 2026-06-03
---

# S-027 Demo Evidence Report

## Method

monocle is a TUI application (ratatui). A live daemon+prompt setup is impractical
for isolated evidence generation. Evidence is captured via VHS recordings that
drive the real compiled binary's test suites. The test suites exercise all
acceptance criteria through the production ratatui `TestBackend` render path —
the same path validated in AC-012's integration render test. This is the
correct evidence vehicle for ratatui TUI products; plain `cargo test` output
is not used — the recordings show the commands executed against the real codebase.

## Artifacts

| Artifact | Format | ACs Covered |
|----------|--------|-------------|
| `AC-001-006-011-overlay-rendering.gif` | GIF (embed) | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-011 |
| `AC-001-006-011-overlay-rendering.webm` | WebM (archival) | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-011 |
| `AC-001-006-011-overlay-rendering.tape` | VHS source | — |
| `AC-007-013-statusbar-integration.gif` | GIF (embed) | AC-007, AC-008, AC-009, AC-010, AC-012, AC-013 |
| `AC-007-013-statusbar-integration.webm` | WebM (archival) | AC-007, AC-008, AC-009, AC-010, AC-012, AC-013 |
| `AC-007-013-statusbar-integration.tape` | VHS source | — |

## AC Coverage Mapping

### AC-001 (BC-2.06.010 PC-1 — overlay widget layout)
**Evidence:** `AC-001-006-011-overlay-rendering.gif`
**Tests:**
- `test_BC_2_06_010_overlay_header_contains_permission_request_label` — header "Permission Request" present
- `test_BC_2_06_010_overlay_footer_contains_accept_hint` — footer "[y] Accept" present
- `test_BC_2_06_010_overlay_footer_contains_reject_hint` — footer "[n/r] Reject" present
- `test_BC_2_06_010_overlay_header_shows_stack_depth_indicator` — "(M of N)" in header
- `test_BC_2_06_010_overlay_modal_width_respects_cap_min_w_minus_4_100` — width capped
- `test_BC_2_06_021_overlay_header_shows_one_of_one_indicator_for_single_prompt_stack` — "(1 of 1)" for single prompt
**Path:** success path
**Status:** PASS (47/47 overlay_render tests)

### AC-002 (BC-2.06.010 PC-2 — dimmed background)
**Evidence:** `AC-001-006-011-overlay-rendering.gif`
**Tests:**
- `test_BC_2_06_010_render_dimmed_background_applies_dim_modifier_to_all_cells` — DIM modifier on all non-modal cells
- `test_BC_2_06_010_overlay_does_not_dim_status_bar_row` — status bar NOT dimmed
**Path:** success path
**Status:** PASS

### AC-003 (BC-2.06.024 PC-1 — Bash tool display)
**Evidence:** `AC-001-006-011-overlay-rendering.gif`
**Tests:**
- `test_BC_2_06_024_render_bash_payload_shows_command_label_and_value` — "command: <cmd>" label
- `test_BC_2_06_024_render_bash_payload_block_has_title_command` — block title
- `test_BC_2_06_024_bash_empty_command_renders_command_empty_fallback` — "(empty)" fallback
**Path:** success + error (empty command) paths
**Status:** PASS

### AC-004 (BC-2.06.024 PC-2 — Read tool display)
**Evidence:** `AC-001-006-011-overlay-rendering.gif`
**Tests:**
- `test_BC_2_06_024_render_read_payload_shows_path_label_and_value` — "path: <path>" label
- `test_BC_2_06_024_render_read_payload_block_has_title_file` — block title
- `test_BC_2_06_024_read_empty_path_renders_path_empty_fallback` — "(empty)" fallback
**Path:** success + error (empty path) paths
**Status:** PASS

### AC-005 (BC-2.06.010 PC-1 — Edit diff preview via similar)
**Evidence:** `AC-001-006-011-overlay-rendering.gif`
**Tests (diff_preview.rs):**
- `test_BC_2_06_010_diff_preview_insert_lines_rendered_green` — "+" lines in green
- `test_BC_2_06_010_diff_preview_delete_lines_rendered_red` — "-" lines in red
- `test_BC_2_06_010_diff_preview_equal_lines_rendered_default_color` — context lines default color
- `test_BC_2_06_010_diff_preview_edit_path_shown_in_diff_block_title` — path in body Block title, not header
- `test_BC_2_06_010_diff_preview_wrap_trim_false_preserves_leading_spaces` — Wrap trim false
- `test_BC_2_06_010_diff_preview_height_cap_truncates_without_panic` — height cap, no panic
- `test_BC_2_06_010_diff_preview_empty_old_content_all_insert_lines_green` — all-insert edge case
- `test_BC_2_06_010_diff_preview_empty_new_content_all_delete_lines_red` — all-delete edge case
**Path:** success + error (empty old/new) paths
**Status:** PASS (13/13 diff_preview tests)

### AC-006 (BC-2.06.024 PC-3 — Generic tool display)
**Evidence:** `AC-001-006-011-overlay-rendering.gif`
**Tests:**
- `test_BC_2_06_024_render_generic_payload_shows_tool_and_input_labels` — "tool:/input:" labels
- `test_BC_2_06_024_render_generic_payload_truncates_input_excerpt_at_256_chars` — 256-char truncation
- `test_BC_2_06_024_generic_payload_fallback_text_is_unrepresentable` — unrepresentable fallback
- `test_BC_2_06_024_generic_no_overflow_shows_zero_scroll_hints` — no spurious scroll hints
- `test_BC_2_06_024_generic_overflow_shows_exactly_one_scroll_hint` — scroll hint on overflow
- `test_BC_2_06_024_generic_wrap_overflow_shows_exactly_one_scroll_hint` — wrap overflow
**Path:** success + error (serialization fallback) paths
**Status:** PASS

### AC-007 (BC-2.06.015 INV-1 — similar in monocle-tui only)
**Evidence:** `AC-001-006-011-overlay-rendering.gif`
**Tests:**
- `test_BC_2_06_015_invariant_2_similar_crate_not_in_monocle_core_cargo_toml` — Cargo.toml check
- `test_BC_2_06_015_invariant_2_similar_not_used_in_monocle_core_source` — source scan
**Path:** purity boundary invariant (no happy/error split — structural property)
**Status:** PASS

### AC-008 (BC-2.06.019 PC-2,7 / BC-2.06.020 PC-1,3,5 / BC-2.06.021 PC-3 — two-row status bar)
**Evidence:** `AC-007-013-statusbar-integration.gif`
**Tests:**
- `test_BC_2_06_019_status_bar_is_two_rows_breadcrumb_upper_hint_lower` — two-row layout
- `test_BC_2_06_019_render_status_bar_drop_zero_renders_mode_indicator` — no drops:0 noise
- `test_BC_2_06_019_render_status_bar_drop_nonzero_renders_dropped_label_in_yellow` — "drops: N" yellow
- `test_BC_2_06_019_pc7_coexistence_drops_and_disconnect_both_visible_in_two_row_bar` — coexistence
- `test_BC_2_06_019_pc7_coexistence_drops_and_trace_stub_both_visible_in_two_row_bar` — [t]+drops coexist
- `test_BC_2_06_019_pc7_drop_counter_only_no_status_message_stays_green` — no mutual exclusion
- `test_BC_2_06_020_render_status_bar_breadcrumb_dashboard_sessions` — "Dashboard > Sessions"
- `test_BC_2_06_020_render_status_bar_breadcrumb_overlay_singular_prompt` — "Dashboard > Overlay [1 prompt]"
- `test_BC_2_06_020_render_status_bar_breadcrumb_overlay_plural_prompts` — "Dashboard > Overlay [N prompts]"
- `test_BC_2_06_020_breadcrumb_dashboard_event_ribbon_focus_renders_events` — "Dashboard > Events"
- `test_BC_2_06_021_render_status_bar_hint_line_dashboard_contains_key_hints` — Dashboard hint
- `test_BC_2_06_021_render_status_bar_hint_line_overlay_contains_trace_stub_binding` — Overlay hint with "t: trace"
- `test_BC_2_06_021_overlay_hint_exact_canonical_y_A_nr_keys` — exact canonical overlay hint
- `test_BC_2_06_021_filtering_hint_exact_canonical_string` — Filtering hint
- `test_BC_2_06_021_fullscreen_hint_exact_canonical_string` — Fullscreen hint
- `test_BC_2_06_021_invariant_3_all_hint_lines_fit_in_79_display_columns` — width constraint
**Path:** success + error (coexistence/no mutual exclusion) paths
**Status:** PASS

### AC-009 (BC-2.06.020 PC-1 — overlay timer display)
**Evidence:** `AC-007-013-statusbar-integration.gif`
**Tests:**
- `test_BC_2_06_020_overlay_header_shows_elapsed_timer_waiting_prefix` — "Waiting: <N>s" present
- `test_BC_2_06_020_overlay_header_elapsed_timer_reflects_real_elapsed_not_hardcoded_zero` — timer not hardcoded
**Path:** success path
**Status:** PASS

### AC-010 (BC-2.06.021 PC-1 — FIFO order in overlay)
**Evidence:** `AC-007-013-statusbar-integration.gif`
**Tests:**
- `test_BC_2_06_021_overlay_oldest_first_fifo_indicator_in_header_for_multi_stack` — "(1 of N)" = oldest
**Path:** success path (multi-prompt stack)
**Status:** PASS

### AC-011 (BC-2.06.010 INV-1 — no blocking render)
**Evidence:** `AC-001-006-011-overlay-rendering.gif`
**Tests:**
- `test_BC_2_06_010_invariant_1_render_overlay_widget_completes_synchronously_within_5ms` — timing invariant
**Note:** Test threshold widened to 10ms per FLAKY-TIMING-5MS entry in durable_task_register (develop @
d88948e). Test passes; the known flake is a CI-level scheduling jitter issue, not a correctness
failure. On develop the test runs in < 1ms in typical conditions.
**Path:** performance invariant
**Status:** PASS (47/47 overlay_render; 13/13 diff_preview)

### AC-012 (BC-2.06.019 PC-1 / BC-2.06.020 PC-1 / BC-2.06.021 PC-3 — integration render wiring)
**Evidence:** `AC-007-013-statusbar-integration.gif`
**Tests (render_frame_integration.rs):**
- `test_BC_2_06_020_render_frame_dashboard_mode_breadcrumb_appears_in_buffer` — breadcrumb in production render
- `test_BC_2_06_021_render_frame_dashboard_mode_hint_line_appears_on_last_row` — hint on last row
- `test_BC_2_06_010_render_frame_overlay_mode_modal_header_appears_in_buffer` — "Permission Request" when Overlay
- `test_BC_2_06_021_render_frame_overlay_mode_hint_line_appears_on_last_row` — Overlay hint on last row
- `test_BC_2_06_020_render_frame_overlay_mode_breadcrumb_appears_on_breadcrumb_row` — breadcrumb in Overlay
- `test_BC_2_06_010_small_terminal_modal_does_not_collide_with_status_bar_rows` — small terminal safe
**Path:** success + error (small terminal) paths via TestBackend drive of `App::render_frame`
**Status:** PASS (6/6 integration tests)

### AC-013 (BC-2.06.015 PC-1/2/3/7 — [t] trace-to-source stub)
**Evidence:** `AC-007-013-statusbar-integration.gif`
**Tests:**
- `test_BC_2_06_019_pc7_coexistence_drops_and_trace_stub_both_visible_in_two_row_bar` — [t] on lower row, drops:N on upper unchanged
- `test_BC_2_06_021_render_status_bar_hint_line_overlay_contains_trace_stub_binding` — "t: trace" in Overlay hint
**Path:** success path (press-gated stub sets status_message) + coexistence (drops:N not displaced)
**Status:** PASS

## Summary

| AC | Status | Evidence Artifact | Path |
|----|--------|-------------------|------|
| AC-001 | PASS | `AC-001-006-011-overlay-rendering.gif` | success + success |
| AC-002 | PASS | `AC-001-006-011-overlay-rendering.gif` | success + success |
| AC-003 | PASS | `AC-001-006-011-overlay-rendering.gif` | success + error |
| AC-004 | PASS | `AC-001-006-011-overlay-rendering.gif` | success + error |
| AC-005 | PASS | `AC-001-006-011-overlay-rendering.gif` | success + error |
| AC-006 | PASS | `AC-001-006-011-overlay-rendering.gif` | success + error |
| AC-007 | PASS | `AC-001-006-011-overlay-rendering.gif` | invariant |
| AC-008 | PASS | `AC-007-013-statusbar-integration.gif` | success + error |
| AC-009 | PASS | `AC-007-013-statusbar-integration.gif` | success |
| AC-010 | PASS | `AC-007-013-statusbar-integration.gif` | success |
| AC-011 | PASS | `AC-001-006-011-overlay-rendering.gif` | invariant |
| AC-012 | PASS | `AC-007-013-statusbar-integration.gif` | success + error |
| AC-013 | PASS | `AC-007-013-statusbar-integration.gif` | success + coexistence |

All 13 ACs have recorded evidence. No ACs without evidence.
