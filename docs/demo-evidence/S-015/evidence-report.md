# Demo Evidence Report — S-015 ClaudeCodeModule Implementation

**Story:** S-015 — ClaudeCodeModule Implementation (BC-2.03.001..004)
**Evidence Type:** Integration test run output (library-level story — no TUI/binary entrypoint)
**Date:** 2026-05-26

## Evidence Rationale

S-015 implements `ClaudeCodeModule` in `monocle-runtime` — a library crate with no runnable
binary entry point. The implementation covers strict basename detection, infallible construction,
HomeUnresolvable error handling, hook_paths() returning exactly 5 entries, and todo!() stubs
for spawn/preflight. For a library crate, the functional evidence is the integration test
suite passing. VHS/Playwright demo recording is not applicable at this library layer.

## AC Coverage

| AC | Description | Test Suite | Key Test(s) | Status |
|----|-------------|-----------|-------------|--------|
| AC-001 | Strict basename detect — "claude" and "claude.js" true; look-alikes false | `engine_module_claude` | `detect_true_for_claude_basename`, `detect_true_for_claude_js_basename`, `detect_false_for_claude_squad`, `detect_false_for_claudio`, `detect_false_for_claude_code_basename`, `detect_false_case_sensitive` | PASS |
| AC-002 | exe_path None → false; cmdline not consulted | `engine_module_claude` | `detect_false_for_exe_path_none` | PASS |
| AC-003 | Infallible constructor | `engine_module_surface` | `test_BC_2_03_001_ac003_process_snapshot_with_full_context` (construction chain) | PASS |
| AC-004 | id() returns "claude-code" | `engine_module_claude` | `test_BC_2_03_002_id_returns_claude_code` | PASS |
| AC-005 | HomeUnresolvable error on metadata/enrich when home env vars unset | `engine_module_home_unresolvable` | `test_BC_2_03_003_metadata_home_unresolvable`, `test_BC_2_03_003_enrich_home_unresolvable` | PASS |
| AC-006 | E-ENG-001 log assertion on HomeUnresolvable | `engine_module_home_unresolvable` | `test_BC_2_03_003_metadata_home_unresolvable`, `test_BC_2_03_003_enrich_home_unresolvable` | PASS |
| AC-007 | hook_paths() returns exactly 5 entries with correct paths | `engine_module_claude` | `test_BC_2_03_004_hook_paths_returns_exactly_5_entries`, `test_BC_2_03_004_hook_paths_contains_correct_paths` | PASS |
| AC-008 | spawn() is todo!() stub | `engine_module_claude` | `test_BC_2_03_004_spawn_is_todo_stub` | PASS |
| AC-009 | preflight() is todo!() stub | `engine_module_claude` | `test_BC_2_03_004_preflight_is_todo_stub` | PASS |
| AC-010 | detect() is I/O-free; on_hook() fail-open with Allow | `engine_module_claude`, `engine_module_surface` | `test_BC_2_03_001_ec031_wildcard_match_arm_on_hook_event_produces_allow`, all detect() tests | PASS |
| AC-011 | Per-variant regression guard — all 5 HookEvent variants → Allow | `engine_module_claude` | `on_hook_session_start_returns_allow`, `on_hook_user_prompt_submit_returns_allow`, `on_hook_pre_tool_use_returns_allow`, `on_hook_notification_returns_allow`, `on_hook_stop_returns_allow` | PASS |

## Test Run Evidence

### monocle-core — EngineModule trait surface (38 tests)

Verifies AC-003 (trait types and construction), AC-006 (log contract via trait surface), AC-010
(I/O-free detect, fail-open on_hook), and the full VP-019 property matrix (5 methods, Send+Sync+
'static supertraits, return types, non-exhaustive enums/structs).

```
running 38 tests
test test_BC_2_03_001_ac003_hook_response_builder_chaining ... ok
test test_BC_2_03_001_ac003_hook_response_with_redirect_sets_field ... ok
test test_BC_2_03_001_ac003_hook_response_with_diagnostic_sets_field ... ok
test test_BC_2_03_001_ac003_process_snapshot_with_full_context ... ok
test test_BC_2_03_001_ac003b_hook_event_has_5_phase1_variants_no_post_tool_use ... ok
test test_BC_2_03_001_ac003b_hook_event_inner_structs_are_non_exhaustive ... ok
test test_BC_2_03_001_ac003_hook_response_redirect_url_is_option_string ... ok
test test_BC_2_03_001_ac003_hook_response_has_canonical_3_fields ... ok
test test_BC_2_03_001_ac005_home_unresolvable_is_usable_in_match ... ok
test test_BC_2_03_001_ac006_engine_module_is_open_trait_implementable_from_outside ... ok
test test_BC_2_03_001_ac001_id_method_signature ... ok
test test_BC_2_03_001_ac003_hook_decision_has_3_canonical_variants ... ok
test test_BC_2_03_001_ac003_hook_response_diagnostic_is_option_string ... ok
test test_BC_2_03_001_ac006_process_snapshot_uses_realistic_epoch ... ok
test test_BC_2_03_001_ac001_on_hook_method_signature ... ok
test test_BC_2_03_001_ac001_detect_method_signature ... ok
test test_BC_2_03_001_ac004_last_event_micros_field_type_verified_by_ast ... ok
test test_BC_2_03_001_ec031_hook_response_allow_is_valid_fail_open_default ... ok
test test_BC_2_03_001_ac007_async_trait_rationale_text_in_engine_module_rustdoc ... ok
test test_BC_2_03_001_ec031_wildcard_match_arm_on_hook_event_produces_allow ... ok
test test_BC_2_03_001_ac003b_hook_event_is_non_exhaustive ... ok
test test_BC_2_03_001_vp019_19f_supporting_types_pub_in_monocle_core_engine ... ok
test test_BC_2_03_001_ac003b_hook_event_not_declared_in_engine_rs ... ok
test test_BC_2_03_001_ac003_engine_metadata_error_is_non_exhaustive ... ok
test test_BC_2_03_001_ac001_metadata_method_is_sync ... ok
test test_BC_2_03_001_ac003_session_status_has_5_canonical_variants ... ok
test test_BC_2_03_001_ac001_enrich_method_is_async ... ok
test test_BC_2_03_001_invariant_non_exhaustive_on_all_supporting_enums ... ok
test test_BC_2_03_001_vp019_19a_exactly_5_methods_with_canonical_names ... ok
test test_BC_2_03_001_vp019_19b_send_sync_static_supertraits_present ... ok
test test_BC_2_03_001_vp019_19d_enrich_return_type_is_result_enriched_session_error ... ok
test test_BC_2_03_001_vp019_19b_no_sealed_supertrait ... ok
test test_BC_2_03_001_vp019_19h_async_trait_attribute_on_engine_module ... ok
test test_BC_2_03_001_invariant_non_exhaustive_on_all_supporting_structs ... ok
test test_BC_2_03_001_vp019_19c_metadata_return_type_is_result_engine_metadata_error ... ok
test test_BC_2_03_001_ac005_engine_metadata_error_home_unresolvable_exists ... ok
test test_BC_2_03_001_vp019_19e_enriched_session_last_event_micros_is_option_i64 ... ok
test test_BC_2_03_001_vp019_19g_sealed_token_absent_from_trait_declaration ... ok

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### monocle-runtime — ClaudeCodeModule detect/id/hooks (18 tests)

Verifies AC-001, AC-002, AC-004, AC-007, AC-008, AC-009, AC-010, AC-011 directly against the
`ClaudeCodeModule` implementation. Includes both success paths (claude, claude.js basenames
return true) and error/rejection paths (claude-squad, claudio, claude-code, Claude, exe_path
None all return false). spawn() and preflight() are confirmed as todo!() via `#[should_panic]`.

```
running 18 tests
test test_BC_2_03_002_id_returns_claude_code ... ok
test test_BC_2_03_004_hook_paths_returns_exactly_5_entries ... ok
test test_BC_2_03_004_hook_paths_contains_correct_paths ... ok
test test_BC_2_03_002_detect_false_for_claude_code_runner ... ok
test test_BC_2_03_002_detect_false_for_exe_path_none ... ok
test test_BC_2_03_002_detect_false_case_sensitive ... ok
test test_BC_2_03_002_detect_false_for_claude_code_basename ... ok
test test_BC_2_03_002_detect_true_for_claude_basename ... ok
test test_BC_2_03_002_detect_false_for_claude_squad ... ok
test test_BC_2_03_002_detect_false_for_claudio ... ok
test test_BC_2_03_002_detect_true_for_claude_js_basename ... ok
test test_BC_2_03_001_on_hook_session_start_returns_allow ... ok
test test_BC_2_03_001_on_hook_user_prompt_submit_returns_allow ... ok
test test_BC_2_03_001_on_hook_pre_tool_use_returns_allow ... ok
test test_BC_2_03_001_on_hook_stop_returns_allow ... ok
test test_BC_2_03_001_on_hook_notification_returns_allow ... ok
test test_BC_2_03_004_spawn_is_todo_stub - should panic ... ok
test test_BC_2_03_004_preflight_is_todo_stub - should panic ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### monocle-runtime — HomeUnresolvable error contract (2 tests)

Verifies AC-005 and AC-006. Uses `temp-env 0.3` `async_with_vars` to atomically unset all four
home env vars (HOME, USERPROFILE, HOMEDRIVE, HOMEPATH) and confirms both `metadata()` and
`enrich()` return `Err(EngineMetadataError::HomeUnresolvable)` with the E-ENG-001 log message
emitted.

```
running 2 tests
test test_BC_2_03_003_enrich_home_unresolvable ... ok
test test_BC_2_03_003_metadata_home_unresolvable ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Workspace Status

440 tests pass across the workspace, 0 failures, clippy clean (`-D warnings`).

S-015 contributes 58 tests: 38 (engine_module_surface) + 18 (engine_module_claude) + 2
(engine_module_home_unresolvable).
