//! Auth header validation integration tests (S-009).
//!
//! Placeholder created by the stub architect. The test-writer agent populates
//! this file with the full test suite for BC-2.01.008 and BC-2.01.009.
//!
//! Planned test coverage (see S-009 story spec §Tasks):
//! - `test_BC_2_01_009_canonical_auth_returns_200_no_warn` — AC-006
//! - `test_BC_2_01_009_alias_auth_returns_200_with_warn` — AC-005
//! - `test_BC_2_01_009_both_headers_canonical_wins_no_warn` — AC-007
//! - `test_BC_2_01_009_missing_both_headers_returns_401_e_auth_001` — AC-004, AC-009
//! - `test_BC_2_01_009_wrong_token_canonical_returns_401_e_auth_002` — AC-006
//! - `test_BC_2_01_009_wrong_token_alias_returns_401_e_auth_002_with_warn` — AC-005
//! - `test_BC_2_01_008_vp_008_source_grep_no_eq_on_secret_bytes` — VP-008, AC-008
//! - `test_BC_2_01_009_vp_009_source_grep_constant_time_eq_on_alias_path` — VP-009, AC-008
//! - `test_BC_2_01_008_token_format_64_hex_chars` — AC-001
//! - `test_BC_2_01_009_hook_endpoints_accept_canonical_auth` — AC-010a
//! - `test_BC_2_01_009_hook_endpoints_accept_alias_auth_with_warn` — AC-010a
//! - `test_BC_2_01_009_hook_endpoints_missing_auth_returns_401` — AC-010a
//! - `test_BC_2_01_008_vp_008_length_mismatch_uses_sentinel` — BC-2.01.008 INV-7 / F-D-01
