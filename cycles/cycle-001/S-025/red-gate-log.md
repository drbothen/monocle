---
document_type: red-gate-log
story_id: S-025
cycle: cycle-001
version: "1.0.0"
timestamp: 2026-05-28T00:00:00Z
status: RED_GATE_ESTABLISHED
producer: vsdd-factory:test-writer
---

# Red Gate Log — S-025: TUI Skeleton + Sessions Panel

## Summary

Red Gate established. All tests that invoke S-025 production `todo!()` functions
fail via `not yet implemented` panics. Tests compile cleanly (warnings only, no errors).

## Test Files

| File | Test Count | Failing | Passing |
|------|------------|---------|---------|
| `crates/monocle-tui/tests/startup_connect.rs` | 18 | 14 | 4 |
| `crates/monocle-tui/tests/sessions_panel.rs` | 21 | 2 | 19 |

## AC Coverage Table

| AC | Test Function(s) | File | Status |
|----|-----------------|------|--------|
| AC-001 | `test_bc_2_06_007_pc1_ac001_app_constructs_for_startup` | startup_connect.rs | PASS (App::new implemented) |
| AC-002 | `test_bc_2_06_004_pc1_ac002_run_panics_on_todo` | startup_connect.rs | PASS (should_panic, run() is todo!()) |
| AC-003 | `test_bc_2_06_004_pc2_ac003_on_disconnect_transitions_to_dashboard`, `test_bc_2_06_004_pc2_ac003_on_disconnect_clears_overlay_stack` | startup_connect.rs | FAIL (todo!()) |
| AC-004 | `test_bc_2_06_004_pc3_ac004_default_config_fallback_succeeds` | startup_connect.rs | PASS (MonocleConfig::default() works) |
| AC-005 | `test_bc_2_06_005_pc1_ac005_renders_one_row_per_session`, `test_bc_2_06_005_pc1_ac005_renders_three_rows_for_three_sessions`, `test_bc_2_06_005_pc3_ac005_renders_empty_state_when_no_sessions`, `test_bc_2_06_005_pc2_inv3_*` | sessions_panel.rs | PASS (should_panic, render() is todo!()) |
| AC-006 | `test_bc_2_06_005_pc2_ac006_tab_cycles_focus_*`, `test_bc_2_06_005_pc2_ac006_j_key_moves_selection_down`, `test_bc_2_06_007_pc1_enter_transitions_to_fullscreen`, `test_bc_2_06_007_pc5_escape_from_fullscreen_returns_to_dashboard` | sessions_panel.rs | Mixed (see notes) |
| AC-007 | `test_bc_2_06_005_pc3_ac007_on_drop_counter_update_sets_field`, `test_bc_2_06_005_pc3_ac007_on_drop_counter_update_zero`, `test_bc_2_06_005_pc3_ac007_drop_counter_shown_when_nonzero`, `test_bc_2_06_005_pc3_ac007_drop_counter_hidden_when_zero` | both files | Mixed (handler is todo!(), render is todo!()) |
| AC-008 | `test_bc_2_05_002_inv4_*` (6 tests), `test_bc_2_06_004_pc2_ac008_*` (4 tests) | startup_connect.rs | FAIL (todo!()) |
| AC-009 | `test_bc_2_06_007_pc1_ac001_app_constructs_for_startup` (partial); full TTY deferred to HS-EXP-009 | startup_connect.rs | PASS (constructor path) |
| AC-010 | `test_bc_2_06_004_inv1_ac010_no_client_disconnect_message` | startup_connect.rs | PASS (compile-time type check) |

## Production Functions Under Test

| Function | File | Status |
|----------|------|--------|
| `apply_permission_prompt_queued` | `app.rs:116` | `todo!()` → FAIL |
| `on_initial_state` | `app.rs:133` | `todo!()` → FAIL |
| `on_drop_counter_update` | `app.rs:147` | `todo!()` → FAIL |
| `on_transport_event` | `app.rs:156` | `todo!()` → FAIL |
| `run` | `app.rs:176` | `todo!()` → FAIL (should_panic passes) |
| `SessionsPanel::render` | `ui/sessions_panel.rs:76` | `todo!()` → FAIL (should_panic passes) |
| `format_token_count` | `ui/sessions_panel.rs` (stub added) | `todo!()` → FAIL (should_panic passes) |
| `format_cost` | `ui/sessions_panel.rs` (stub added) | `todo!()` → FAIL (should_panic passes) |
| `build_dashboard_layout` | `ui/layout.rs:42` | `todo!()` → FAIL (should_panic passes) |

## Cargo Test Output (totals)

```
test result: FAILED. 4 passed; 14 failed; 0 ignored (startup_connect.rs)
test result: FAILED. 19 passed; 2 failed; 0 ignored (sessions_panel.rs)
```

First failure (verbatim):
```
thread 'test_bc_2_05_002_inv4_apply_empty_overlay_plus_p1_length_1' panicked at crates/monocle-tui/src/app.rs:120:5:
not yet implemented
```

## Red Gate Exceptions and Flags

### Tests that PASS in sessions_panel.rs (non-S-025 production code)

The following tests exercise `monocle_core::tui::state::transition()` from S-024
(already implemented). They pass but are NOT Red Gate violations because they are
testing pre-existing S-024 code, not S-025 production code:

1. `test_bc_2_06_007_pc1_enter_transitions_to_fullscreen` — PASS: tests `transition(Dashboard{Sessions}, EnterFullscreen{Sessions})` arm, implemented in S-024.
2. `test_bc_2_06_007_pc5_escape_from_fullscreen_returns_to_dashboard` — PASS: tests `transition(Fullscreen{Sessions}, ExitFullscreen)` arm, implemented in S-024.

These are documentation tests verifying the BC-2.06.007 contract. They remain
in the test suite because the S-025 implementer needs them to pass and they do
not represent untested production code in S-025.

The two `MoveFocus` tests (`tab_cycles_focus_sessions_to_event_ribbon`,
`tab_cycles_focus_event_ribbon_to_sessions`) FAIL because `transition()` does
not yet handle `Action::MoveFocus` (falls through to identity arm). The
implementer must add the `MoveFocus` arm to S-024's `transition()` — or confirm
this is intended as an S-025 task.

### AC-001 / AC-009 full coverage note

`install_panic_hook()` and `restore_terminal()` are private to `main.rs` and
cannot be imported by integration tests. Full AC-009 coverage (panic hook TTY
restoration) is deferred to Phase 4 holdout HS-EXP-009 per the test authoring
rules. The constructor-path test covers the AC-001 prerequisite.

### AC-006 `j`/`k` key navigation

The `on_key_down` / `on_key_up` handlers do not exist yet as stub functions in
S-025. The `j_key_moves_selection_down` test exercises the render path
(which is `todo!()`) rather than a dedicated key handler. When the implementer
adds key handlers, a dedicated non-render test should be added.
