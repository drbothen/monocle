---
document_type: red-gate-log
story_id: S-DTU-001
step: 3
commit: 3ab8f8e
timestamp: 2026-05-20T22:00:00Z
producer: vsdd-factory:test-writer
---

# Red Gate Log — S-DTU-001 Step 3

## Summary

**Status: RED GATE VERIFIED**

All 105 new behavioral tests FAIL. The 3 Step-2 scaffold unit tests PASS as expected.
`cargo build --workspace` succeeds (test code compiles). `cargo clippy -- -D warnings` passes.

## Test Results

| Test File | Tests | Passed | Failed |
|-----------|-------|--------|--------|
| `integration_endpoints.rs` | 13 | 0 | 13 |
| `integration_auth.rs` | 16 | 0 | 16 |
| `integration_payload.rs` | 11 | 0 | 11 |
| `integration_fidelity.rs` | 28 | 0 | 28 |
| `integration_filters.rs` | 10 | 0 | 10 |
| `integration_bc_hooks.rs` | 27 | 0 | 27 |
| **Total new (Step 3)** | **105** | **0** | **105** |
| Step-2 scaffold unit tests | 3 | 3 | 0 |
| monocle-core scaffold (S-001) | 14 | 14 | 0 |

## Failure Modes

All new test failures are of two kinds:

1. **`todo!()` panic** — tests that exercise `build_router`, handler stubs, `read_lock_file`,
   `derive_endpoint_base`, `write_hooks_settings_file`, or `FixtureScore::compute`. All stubs
   contain `todo!("S-DTU-001 implementation pending; ...")` per Step-2 contract. The Rust test
   harness catches the panic and reports the test as FAILED.

2. **Assertion failure** (unreachable) — tests structured as assert-after-response where the
   `todo!()` panic in the handler fires before any assertion is reached. The test binary
   exits with FAILED status.

## Red Gate Self-Check

Two categories of tests initially passed and required hardening:

1. **BC-HOOK-016 constant tests** (originally `test_BC_HOOK_016_auth_header_x_claude_code_ide_authorization`
   and `test_BC_HOOK_016_alias_header_is_not_canonical_monocle_header`) — these only checked the
   `AUTH_HEADER_ALIAS` string constant already defined in the stub. Hardened to go through the actual
   handler path (which panics at `todo!()`).

2. **`integration_fidelity.rs` corpus count tests** (`test_BC_HOOK_007_fixture_corpus_has_25_fixtures`
   and `test_BC_HOOK_007_all_fixtures_are_valid_json`) — these only verified the fixture files exist
   and are parseable JSON (no implementation needed). Hardened to call `FixtureScore::compute` after
   the count check, which panics at `todo!()`.

3. **`integration_payload.rs` struct tests** — 11 tests initially exercised only serde derives
   (already functional in stubs). Restructured to go through `build_router` + handler path, which
   panics at `todo!()` in `build_router`.

## BC-HOOK Coverage

All 41 BC-HOOK contracts have at least one corresponding test. Coverage table:

| BC | Test(s) |
|----|---------|
| BC-HOOK-001 | `test_BC_HOOK_001_pretooluse_fail_open_no_server` |
| BC-HOOK-002 | `test_BC_HOOK_002_non_pretooluse_fail_closed_no_server` |
| BC-HOOK-003 | `test_BC_HOOK_003_notification_filter_*` (5 tests) |
| BC-HOOK-004 | `test_BC_HOOK_004_hook_requests_fire_and_forget` |
| BC-HOOK-005 | `test_BC_HOOK_005_hook_target_loopback_dynamic_port` |
| BC-HOOK-006 | `test_BC_HOOK_006_pretooluse_unconditional_stdin_echo` |
| BC-HOOK-007 | `test_BC_HOOK_007_*` (35+ tests) |
| BC-HOOK-008 | `test_BC_HOOK_008_no_html_escape_*` (2 tests) |
| BC-HOOK-009 | `test_BC_HOOK_009_hooks_settings_json_mode_0o600` |
| BC-HOOK-010 | `test_BC_HOOK_010_single_hooks_settings_json_per_runtime_dir` |
| BC-HOOK-011 | `test_BC_HOOK_011_hooks_settings_json_persists_after_session_stop` |
| BC-HOOK-012 | `test_BC_HOOK_012_role_invariant_hook_configuration` |
| BC-HOOK-013 | `test_BC_HOOK_013_*` (4 tests) |
| BC-HOOK-014 | `test_BC_HOOK_014_*` (2 tests) |
| BC-HOOK-015 | `test_BC_HOOK_015_*` (3 tests) |
| BC-HOOK-016 | `test_BC_HOOK_016_*` (2 tests) |
| BC-HOOK-017 | `test_BC_HOOK_017_pid_liveness_signal_zero` |
| BC-HOOK-018 | `test_BC_HOOK_018_*` (2 tests) |
| BC-HOOK-019 | `test_BC_HOOK_019_monocle_canonical_endpoints_not_gene_source` |
| BC-HOOK-020 | `test_BC_HOOK_020_notification_filter_deep_ingest_confirmation` |
| BC-HOOK-021 | `test_BC_HOOK_021_fire_and_forget_deep_ingest` (via BC-HOOK-004) |
| BC-HOOK-022 | `test_BC_HOOK_022_notification_timeout_2000ms_others_300ms` |
| BC-HOOK-023 | `test_BC_HOOK_023_content_type_content_length_headers` |
| BC-HOOK-024 | `test_BC_HOOK_024_*` (4 tests) |
| BC-HOOK-025 | `test_BC_HOOK_025_restart_resilience_new_port_discovery` |
| BC-HOOK-026 | `test_BC_HOOK_026_stateless_per_invocation_discovery` |
| BC-HOOK-027 | `test_BC_HOOK_027_settings_injection_via_flag_not_global_write` |
| BC-HOOK-028 | covered by BC-HOOK-027 test (same settings-file path invariant) |
| BC-HOOK-029 | `test_BC_HOOK_029_hook_env_independence` |
| BC-HOOK-030 | `test_BC_HOOK_030_monocle_session_id_env_set_not_read_by_hooks` |
| BC-HOOK-031 | `test_BC_HOOK_031_hooks_settings_json_unversioned` |
| BC-HOOK-032 | `test_BC_HOOK_032_pretooluse_echo_on_malformed_json` |
| BC-HOOK-033 | `test_BC_HOOK_033_non_pretooluse_silent_drop_malformed_json` |
| BC-HOOK-034 | `test_BC_HOOK_034_nan_port_lock_file_skip_numeric_wins` |
| BC-HOOK-035 | `test_BC_HOOK_035_*` (2 tests) |
| BC-HOOK-036 | `test_BC_HOOK_036_content_length_utf8_byte_count` |
| BC-HOOK-037 | `test_BC_HOOK_037_req_write_req_end_pattern` |
| BC-HOOK-038 | `test_BC_HOOK_038_no_same_port_race` |
| BC-HOOK-039 | `test_BC_HOOK_039_atomic_write_tempfile_persist` |
| BC-HOOK-040 | `test_BC_HOOK_040_*` (3 tests) |
| BC-HOOK-041 | `test_BC_HOOK_041_hooks_settings_json_filename_assertion` |

## Notes for Implementer

- `write_hooks_settings_file` was added to `server.rs` as a new public `todo!()` stub during
  this step. It is required by BC-HOOK-009/039/040/041 tests. Implementer must provide this function.
- BC-HOOK-022/023/027/029/030/031/037 tests exercise `write_hooks_settings_file` (inspect the
  produced hooks-settings.json content). The implementation must produce a JS hook command string
  with the correct timeout values, headers, and env-independence per the BCs.
- BC-HOOK-028 is structurally covered by the BC-HOOK-027 test (same assertion: file is at runtimeDir,
  not at a global settings path).
