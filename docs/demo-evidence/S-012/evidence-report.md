# Demo Evidence Report — S-012 FactoryAdapter Trait + VsddFactoryAdapter

**Story:** S-012 — FactoryAdapter Trait + VsddFactoryAdapter (BC-2.02.004 + BC-2.02.005)
**Evidence Type:** Integration test run output (library-level story — no TUI/binary entrypoint)
**Date:** 2026-05-26

## Evidence Rationale

S-012 implements the `FactoryAdapter` trait and `VsddFactoryAdapter` in `monocle-core::factory`.
This is a library-level story with no runnable binary entry point. The TUI planes that consume
`FactoryAdapter` (Workflow plane, Phase 3 WASM plugin registry) are deferred to later waves and
phases.

For a library crate with no runnable binary entry point, the functional evidence is the
integration test suite passing, including the self-referential detection test that runs against
the monocle project's own `.factory/STATE.md`. VHS/Playwright demo recording is not applicable
at this library layer.

## AC Coverage

| AC | Description | Evidence | Status |
|----|-------------|----------|--------|
| AC-001 | 7 methods exact in FactoryAdapter trait | `test_BC_FACTORY_001_trait_defined_open_no_sealed_bound` (AST count) | PASS |
| AC-002 | No Sealed bound; Send+Sync+'static only | `test_BC_FACTORY_001_sealed_token_absent_from_trait_declaration` | PASS |
| AC-003 | Supporting types co-located in factory module | `test_BC_FACTORY_001_supporting_types_pub_in_monocle_core_factory` | PASS |
| AC-004 | FactoryState exactly 7 fields, no raw_frontmatter | `test_BC_FACTORY_001_factory_state_exactly_7_fields` + `factory_state_custom_fields_uses_serde_yaml_ng_not_json` | PASS |
| AC-005 | detect() frontmatter-only (EC-021 guard) | `test_BC_FACTORY_002_vsdd_detect_negative_body_only` | PASS |
| AC-006 | Self-referential detection on monocle repo | `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection` | PASS |
| AC-007 | subscribe() Phase 1 stub = empty stream | `test_BC_FACTORY_002_vsdd_adapter_subscribe_empty` | PASS |
| AC-008 | 3-state error handling (NotFound/ParseError/Ok) | `test_BC_FACTORY_002_vsdd_adapter_read_state_not_found` + `read_state_parse_error_no_frontmatter` | PASS |
| AC-009 | subscribe() poll returns None immediately | `test_BC_FACTORY_002_vsdd_adapter_subscribe_empty` | PASS |
| AC-010 | display_name() returns exact "VSDD Factory" | `test_BC_FACTORY_002_vsdd_adapter_display_name` | PASS |
| AC-011 | new() infallible, no validation at ctor | `test_BC_FACTORY_002_vsdd_adapter_new_constructor` | PASS |
| AC-012 | Absent fields = None, not "unknown" | `test_BC_FACTORY_002_vsdd_adapter_read_state_cycle_absent` | PASS |
| AC-013 | 4 parser guards + unquote | `test_BC_FACTORY_002_vsdd_parse_guard_*` (7 tests) | PASS |

## Test Run Evidence

```
running 12 tests
test test_BC_FACTORY_001_supporting_types_pub_in_monocle_core_factory ... ok
test test_BC_FACTORY_001_factory_state_custom_fields_uses_serde_yaml_ng_not_json ... ok
test test_BC_FACTORY_001_abi_version_has_default_impl ... ok
test test_BC_FACTORY_001_factory_subscribe_error_is_non_exhaustive ... ok
test test_BC_FACTORY_001_factory_read_error_is_non_exhaustive ... ok
test test_BC_FACTORY_001_blocking_severity_is_non_exhaustive ... ok
test test_BC_FACTORY_001_factory_state_awaiting_is_option_string ... ok
test test_BC_FACTORY_001_factory_detection_3_fields ... ok
test test_BC_FACTORY_001_factory_state_exactly_7_fields ... ok
test test_BC_FACTORY_001_trait_defined_open_no_sealed_bound ... ok
test test_BC_FACTORY_001_detect_method_has_where_self_sized ... ok
test test_BC_FACTORY_001_sealed_token_absent_from_trait_declaration ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 22 tests
test test_BC_FACTORY_002_vsdd_adapter_display_name ... ok
test test_BC_FACTORY_002_vsdd_adapter_new_constructor ... ok
test test_BC_FACTORY_002_matches_self_referential ... ok
test test_BC_FACTORY_002_vsdd_adapter_self_referential_detection ... ok
test test_BC_FACTORY_002_vsdd_adapter_read_state_on_real_state_md ... ok
test test_BC_FACTORY_002_vsdd_adapter_subscribe_empty ... ok
test test_BC_FACTORY_002_matches_returns_false_for_non_factory_dir ... ok
test test_BC_FACTORY_002_vsdd_detect_negative_no_state_file ... ok
test test_BC_FACTORY_002_vsdd_adapter_read_state_not_found ... ok
test test_BC_FACTORY_002_matches_returns_true_for_vsdd_workspace ... ok
test test_BC_FACTORY_002_vsdd_parse_guard_continuation_line_yields_none ... ok
test test_BC_FACTORY_002_vsdd_adapter_read_state_parse_error_no_frontmatter ... ok
test test_BC_FACTORY_002_vsdd_parse_single_quoted_scalar_unquoted ... ok
test test_BC_FACTORY_002_vsdd_adapter_read_state_cycle_absent ... ok
test test_BC_FACTORY_002_vsdd_parse_guard_block_scalar_literal_yields_none ... ok
test test_BC_FACTORY_002_vsdd_parse_guard_empty_quoted_value_yields_none ... ok
test test_BC_FACTORY_002_vsdd_parse_guard_block_scalar_folded_yields_none ... ok
test test_BC_FACTORY_002_vsdd_parse_double_quoted_scalar_unquoted ... ok
test test_BC_FACTORY_002_vsdd_adapter_read_state_success ... ok
test test_BC_FACTORY_002_vsdd_parse_guard_empty_value_yields_none ... ok
test test_BC_FACTORY_002_vsdd_parse_guard_flow_list_yields_none ... ok
test test_BC_FACTORY_002_vsdd_detect_negative_body_only ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Workspace-wide: 340+ tests pass, 0 regressions, clippy clean.
