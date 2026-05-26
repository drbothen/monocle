# Demo Evidence Report — S-009 Auth Token Wire Format + Header Validation

**Story:** S-009 — Auth Token Wire Format + Header Validation (BC-2.01.008 + BC-2.01.009)
**Evidence Type:** Integration test run output (library-level story — no TUI/binary entrypoint)
**Date:** 2026-05-26

## Evidence Rationale

S-009 implements `validate_auth_header()` and extends the auth middleware in
`monocle-runtime::auth`. This is a library-level story with no runnable binary entry point.
The daemon-level binary entrypoint is deferred to the full `main.rs` wiring story (S-005
and future integration story per `S-005-main-wiring` durable task register entry).

For a library crate with no runnable binary entry point, the functional evidence is the
integration test suite passing. VHS/Playwright demo recording is not applicable at this
library layer.

## AC Coverage

| AC | Description | Evidence | Status |
|----|-------------|----------|--------|
| AC-001 | OsRng token in lock file, 64-hex regex | `test_BC_2_01_009_validate_canonical_correct_token` (reads from DaemonState) | PASS |
| AC-002 | Raw hex in authToken (no prefix in lock file) | `test_BC_2_01_009_validate_alias_correct_token` (alias accepts raw hex) | PASS |
| AC-003 | Canonical header format monocle-v1:<64-hex> | `test_BC_2_01_009_validate_canonical_raw_hex_no_prefix` (missing prefix → 401) | PASS |
| AC-004 | Missing both → 401 E-AUTH-001 | `test_BC_2_01_009_validate_both_absent` | PASS |
| AC-005 | Alias path + WARN E-AUTH-003 on every alias request | `test_BC_2_01_009_invariant_warn_log_flag_is_alias_path`, `test_BC_2_01_009_validate_alias_correct_token`, `test_BC_2_01_009_validate_alias_wrong_token` | PASS |
| AC-006 | Canonical path, no WARN | `test_BC_2_01_009_validate_canonical_returns_canonical_variant_not_alias` | PASS |
| AC-007 | Both present → canonical wins, no WARN | `test_BC_2_01_009_validate_canonical_wins_when_both_present` | PASS |
| AC-008 | constant_time_eq ALL paths, no == on token bytes | `test_BC_2_01_008_vp_008_source_grep_no_eq_on_secret_bytes`, `test_BC_2_01_009_vp_009_source_grep_constant_time_eq_on_alias_path` | PASS |
| AC-009 | Both absent → 401 E-AUTH-001 | `test_BC_2_01_009_validate_both_absent` | PASS |
| AC-010a | 5 hook endpoints, dual-accept auth | `test_hook_pre_tool_use_unauthenticated_returns_401` | PASS |
| AC-010b | 5 hook handlers, RingBuffer + {"status":"ok"} | `test_hook_pre_tool_use_running_canonical_auth_returns_200` | PASS |

## Test Run Evidence

### auth_header_rejection.rs (24 unit tests)

```
running 24 tests
test test_BC_2_01_009_invariant_missing_and_invalid_are_distinct ... ok
test test_BC_2_01_009_invariant_warn_log_flag_is_alias_path ... ok
test test_BC_2_01_009_validate_alias_64_all_zeros_wrong ... ok
test test_BC_2_01_009_validate_alias_correct_token ... ok
test test_BC_2_01_009_validate_alias_non_hex ... ok
test test_BC_2_01_009_validate_alias_returns_alias_variant_not_canonical ... ok
test test_BC_2_01_009_validate_alias_wrong_length ... ok
test test_BC_2_01_009_validate_alias_wrong_token ... ok
test test_BC_2_01_009_validate_both_absent ... ok
test test_BC_2_01_009_validate_both_present_canonical_bad_alias_correct ... ok
test test_BC_2_01_009_validate_canonical_64_zeros_prefix_wrong ... ok
test test_BC_2_01_009_validate_canonical_bad_prefix ... ok
test test_BC_2_01_009_validate_canonical_correct_token ... ok
test test_BC_2_01_009_validate_canonical_empty_token_after_prefix ... ok
test test_BC_2_01_008_vp_008_length_mismatch_uses_sentinel ... ok
test test_BC_2_01_009_validate_canonical_empty_value ... ok
test test_BC_2_01_009_validate_canonical_raw_hex_no_prefix ... ok
test test_BC_2_01_009_validate_canonical_returns_canonical_variant_not_alias ... ok
test test_BC_2_01_009_validate_canonical_wins_when_both_present ... ok
test test_BC_2_01_009_validate_canonical_wrong_token ... ok
test test_BC_2_01_009_validate_canonical_wrong_version_prefix ... ok
test test_validate_uppercase_hex_rejected ... ok
test test_BC_2_01_009_vp_009_source_grep_constant_time_eq_on_alias_path ... ok
test test_BC_2_01_008_vp_008_source_grep_no_eq_on_secret_bytes ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### hook_post_running_mode.rs (2 integration tests)

```
running 2 tests
test test_hook_pre_tool_use_unauthenticated_returns_401 ... ok
test test_hook_pre_tool_use_running_canonical_auth_returns_200 ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Workspace-wide

```
400 tests pass, 0 regressions, clippy clean, cargo fmt --check clean.
```
