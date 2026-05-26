//! Auth header validation unit tests for `validate_auth_header()` (S-009).
//!
//! Tests the pure dual-accept validation function extracted from `auth_middleware` for
//! unit-testing the dual-accept logic independently of the axum request stack.
//!
//! # Contract
//!
//! Exercises BC-2.01.008 (Auth Token Wire Format FC-06) and BC-2.01.009 (Auth Header
//! Validation ADR-0005). Verifies VP-008 and VP-009.
//!
//! # Red Gate
//!
//! All tests in this file MUST FAIL before S-009 implementation is complete.
//! `validate_auth_header()` currently panics with `todo!()`. Tests will panic
//! (surfaced as test failure) until the implementation replaces the stub.
//!
//! # Coverage Map
//!
//! | Test | BC / AC | VP | Decision matrix row |
//! |------|---------|-----|---------------------|
//! | test_BC_2_01_009_validate_canonical_correct_token | AC-006 | VP-009 | canonical present, valid prefix, token matches → Ok(Canonical) |
//! | test_BC_2_01_009_validate_canonical_wrong_token | AC-006 | VP-009 | canonical present, valid prefix, token mismatch → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_alias_correct_token | AC-005 | VP-009 | canonical absent, alias present, token matches → Ok(Alias) |
//! | test_BC_2_01_009_validate_alias_wrong_token | AC-005 | VP-009 | canonical absent, alias present, token mismatch → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_both_absent | AC-004, AC-009 | VP-009 | both absent → Err(MissingToken) |
//! | test_BC_2_01_009_validate_canonical_wins_when_both_present | AC-007 | VP-009 | both present, canonical correct → Ok(Canonical) |
//! | test_BC_2_01_009_validate_canonical_bad_prefix | AC-006 | VP-009 | canonical present, wrong prefix → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_canonical_empty_token_after_prefix | AC-006, EC-009 | VP-009 | canonical present, prefix-only, no hex suffix → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_alias_non_hex | AC-005 | VP-009 | alias present, non-hex chars → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_alias_wrong_length | AC-005 | VP-009 | alias present, wrong length (32 not 64) → Err(InvalidToken) |
//! | test_BC_2_01_008_vp_008_source_grep_no_eq_on_secret_bytes | VP-008, AC-008 | VP-008 | source audit: no `==` on secret bytes |
//! | test_BC_2_01_009_vp_009_source_grep_constant_time_eq_on_alias_path | VP-009, AC-008 | VP-009 | source audit: constant_time_eq present on alias path |
//! | test_BC_2_01_008_vp_008_length_mismatch_uses_sentinel | BC-2.01.008 INV-7, F-D-01 | VP-008 | length mismatch must still run constant_time_eq against sentinel |
//! | test_BC_2_01_009_validate_canonical_empty_value | EC-007 | VP-009 | empty canonical value → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_both_present_canonical_bad_alias_correct | AC-007, INV-5 | VP-009 | canonical wrong + alias correct → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_alias_returns_alias_variant_not_canonical | AC-005 | VP-009 | alias success → Ok(Alias) not Ok(Canonical) |
//! | test_BC_2_01_009_validate_canonical_returns_canonical_variant_not_alias | AC-006 | VP-009 | canonical success → Ok(Canonical) not Ok(Alias) |
//! | test_BC_2_01_009_validate_canonical_wrong_version_prefix | AC-006 | VP-009 | monocle-v2: prefix → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_canonical_raw_hex_no_prefix | AC-006 | VP-009 | correct token value but missing monocle-v1: prefix → Err(InvalidToken) |
//! | test_BC_2_01_009_invariant_missing_and_invalid_are_distinct | INV-2 | VP-009 | error discriminants must differ |
//! | test_BC_2_01_009_invariant_warn_log_flag_is_alias_path | INV-6 | VP-009 | Ok(Alias) ← caller must emit WARN; Ok(Canonical) ← must not |
//! | test_BC_2_01_009_validate_alias_64_all_zeros_wrong | AC-005 | VP-009 | alias: 64 zeros ≠ non-zero token → Err(InvalidToken) |
//! | test_BC_2_01_009_validate_canonical_64_zeros_prefix_wrong | AC-006 | VP-009 | canonical: monocle-v1: + 64 zeros ≠ token → Err(InvalidToken) |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per the naming convention.
#![allow(non_snake_case)]

use monocle_runtime::auth::{validate_auth_header, AuthError, AuthPath};

// ---------------------------------------------------------------------------
// Test constants
// ---------------------------------------------------------------------------

/// Raw 64-hex-char token used as the stored expected secret across all tests.
/// All lowercase hex, exactly 64 chars, no prefix. Regex: /^[0-9a-f]{64}$/.
const TEST_TOKEN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// A different 64-hex-char token that does not match TEST_TOKEN — used for mismatch cases.
const WRONG_TOKEN: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

/// A 64-hex-char token consisting of all zeros — different from TEST_TOKEN.
const ZERO_TOKEN: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// The `monocle-v1:` prefix required on the canonical header value.
const CANONICAL_PREFIX: &str = "monocle-v1:";

/// Correct canonical header value: prefix + TEST_TOKEN.
const CANONICAL_VALUE_CORRECT: &str =
    "monocle-v1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// Wrong canonical header value: prefix + WRONG_TOKEN.
const CANONICAL_VALUE_WRONG: &str =
    "monocle-v1:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

// ---------------------------------------------------------------------------
// Positive path: canonical header
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-2 / AC-006: `validate_auth_header` with the canonical header present and
/// the correct 64-hex token (after `monocle-v1:` prefix strip) returns `Ok(AuthPath::Canonical)`.
///
/// Decision matrix row: canonical present, valid `monocle-v1:<64-hex>`, token matches → `Ok(Canonical)`.
///
/// Counter-example guarded: `todo!()` stub panics — this test fails until the implementation
/// strips the prefix and performs constant-time comparison successfully.
///
/// Traces to BC-2.01.009 PC-2, AC-006.
#[test]
fn test_BC_2_01_009_validate_canonical_correct_token() {
    let result = validate_auth_header(Some(CANONICAL_VALUE_CORRECT), None, TEST_TOKEN);

    assert_eq!(
        result,
        Ok(AuthPath::Canonical),
        "validate_auth_header with canonical header `monocle-v1:<correct-token>` must return \
        Ok(AuthPath::Canonical); got: {result:?}. \
        Traces to BC-2.01.009 PC-2 / AC-006 / VP-009. \
        Counter-example: todo!() stub panics."
    );
}

/// BC-2.01.009 PC-2 / AC-006: `validate_auth_header` with canonical header present but wrong
/// token value returns `Err(AuthError::InvalidToken)`.
///
/// Decision matrix row: canonical present, valid `monocle-v1:<64-hex>`, token mismatch → `Err(InvalidToken)`.
///
/// Counter-example guarded: `todo!()` stub panics; an incorrect implementation that ignores
/// mismatch and returns Ok would also fail this test.
///
/// Traces to BC-2.01.009 PC-2, AC-006.
#[test]
fn test_BC_2_01_009_validate_canonical_wrong_token() {
    let result = validate_auth_header(Some(CANONICAL_VALUE_WRONG), None, TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with canonical header `monocle-v1:<wrong-token>` must return \
        Err(AuthError::InvalidToken); got: {result:?}. \
        Traces to BC-2.01.009 PC-2 / AC-006 / VP-009. \
        Counter-example: constant_time_eq(wrong_bytes, expected_bytes) returns false."
    );
}

// ---------------------------------------------------------------------------
// Positive path: alias header
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-3 / AC-005: `validate_auth_header` with canonical absent, alias present and
/// containing the correct raw 64-hex token returns `Ok(AuthPath::Alias)`.
///
/// Decision matrix row: canonical absent, alias present, raw token matches → `Ok(Alias)`.
/// Caller is responsible for emitting the ADR-0005 WARN log when `Ok(Alias)` is returned.
///
/// Counter-example guarded: `todo!()` stub panics; an implementation that returns `Ok(Canonical)`
/// on the alias path would be caught by the discriminant check.
///
/// Traces to BC-2.01.009 PC-3, AC-005.
#[test]
fn test_BC_2_01_009_validate_alias_correct_token() {
    // Alias path: raw 64-hex, no `monocle-v1:` prefix.
    let result = validate_auth_header(None, Some(TEST_TOKEN), TEST_TOKEN);

    assert_eq!(
        result,
        Ok(AuthPath::Alias),
        "validate_auth_header with alias header (raw-64-hex correct token, no canonical) must \
        return Ok(AuthPath::Alias); got: {result:?}. \
        Traces to BC-2.01.009 PC-3 / AC-005 / VP-009. \
        Counter-example: returning Ok(Canonical) on alias path would violate the discriminant."
    );
}

/// BC-2.01.009 PC-3 / AC-005: `validate_auth_header` with canonical absent, alias present but
/// wrong token returns `Err(AuthError::InvalidToken)`.
///
/// Decision matrix row: canonical absent, alias present, token mismatch → `Err(InvalidToken)`.
///
/// Traces to BC-2.01.009 PC-3, AC-005.
#[test]
fn test_BC_2_01_009_validate_alias_wrong_token() {
    let result = validate_auth_header(None, Some(WRONG_TOKEN), TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with alias header (wrong raw-64-hex, no canonical) must return \
        Err(AuthError::InvalidToken); got: {result:?}. \
        Traces to BC-2.01.009 PC-3 / AC-005 / VP-009."
    );
}

// ---------------------------------------------------------------------------
// Missing path: both headers absent
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-1 / AC-004, AC-009: `validate_auth_header` with BOTH canonical and alias
/// absent returns `Err(AuthError::MissingToken)` — the dual-absence case, not a value-present
/// failure.
///
/// Decision matrix row: canonical absent, alias absent → `Err(MissingToken)`.
///
/// Counter-example guarded: returning `Err(InvalidToken)` when both are absent would violate
/// INV-2 (missing vs. invalid distinction preserved) and BC-2.01.009 PC-1 (the body must be
/// `{"error":"missing_auth_token"}`, not `{"error":"invalid_auth_token"}`).
///
/// Traces to BC-2.01.009 PC-1, EC-008, AC-004, AC-009.
#[test]
fn test_BC_2_01_009_validate_both_absent() {
    let result = validate_auth_header(None, None, TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::MissingToken),
        "validate_auth_header with both canonical and alias absent must return \
        Err(AuthError::MissingToken), not Err(InvalidToken); got: {result:?}. \
        INV-2: missing vs. invalid distinction is preserved (missing is a client-config error; \
        invalid is an auth-attempt failure). \
        Traces to BC-2.01.009 PC-1 / EC-008 / AC-004 / AC-009."
    );
}

// ---------------------------------------------------------------------------
// Priority: canonical wins when both headers present
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-4 / AC-007: When BOTH headers are present and the canonical value is correct,
/// `validate_auth_header` returns `Ok(AuthPath::Canonical)` and ignores the alias.
///
/// Decision matrix row: canonical present and valid, alias also present → `Ok(Canonical)`.
/// No WARN log is warranted (canonical path, not alias path).
///
/// Counter-example guarded: an implementation that evaluates the alias when canonical also
/// succeeds would return `Ok(Alias)` — failing the discriminant assertion.
///
/// Traces to BC-2.01.009 PC-4, INV-5, AC-007.
#[test]
fn test_BC_2_01_009_validate_canonical_wins_when_both_present() {
    // Both present; canonical is correct; alias is a different (wrong) value.
    // If canonical priority is honoured, the alias value is irrelevant.
    let result = validate_auth_header(Some(CANONICAL_VALUE_CORRECT), Some(WRONG_TOKEN), TEST_TOKEN);

    assert_eq!(
        result,
        Ok(AuthPath::Canonical),
        "validate_auth_header with both headers present (canonical correct, alias wrong) must \
        return Ok(AuthPath::Canonical); got: {result:?}. \
        INV-5: canonical priority is immutable. \
        Traces to BC-2.01.009 PC-4 / INV-5 / AC-007 / VP-009."
    );
}

// ---------------------------------------------------------------------------
// Canonical format failures
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-2 / AC-006: Canonical header with a wrong version prefix (`monocle-v2:`)
/// returns `Err(AuthError::InvalidToken)`.
///
/// The value is present (header is sent) but the prefix does not match `monocle-v1:`. This is
/// a value-present format failure, NOT a missing-header case. Returns E-AUTH-002 (invalid),
/// not E-AUTH-001 (missing).
///
/// Counter-example guarded: returning `MissingToken` for a wrong-prefix canonical value would
/// violate INV-2 (header is present; it fails validation).
///
/// Traces to BC-2.01.009 PC-2 / AC-006. Canonical test vector row 3.
#[test]
fn test_BC_2_01_009_validate_canonical_bad_prefix() {
    // monocle-v2: prefix — wrong version, not the monocle-v1: prefix.
    let bad_prefix_value = format!("monocle-v2:{TEST_TOKEN}");
    let result = validate_auth_header(Some(&bad_prefix_value), None, TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with canonical header `monocle-v2:<token>` (wrong prefix) must \
        return Err(AuthError::InvalidToken); got: {result:?}. \
        Header is present; the prefix check fails → value-present failure → E-AUTH-002. \
        Counter-example: returning MissingToken would violate INV-2. \
        Traces to BC-2.01.009 PC-2 / AC-006. Test vector row 3."
    );
}

/// BC-2.01.009 EC-009 / AC-006: Canonical header with the correct prefix but no hex suffix
/// (`monocle-v1:` — nothing after the colon) returns `Err(AuthError::InvalidToken)`.
///
/// The prefix check passes, but the empty suffix does not match the 64-char stored secret.
/// Per BC-2.01.009 INV-7 the comparison MUST still run against a fixed-length sentinel
/// (not short-circuit before comparison) to prevent a timing oracle on the empty-suffix branch.
///
/// Counter-example guarded: returning `MissingToken` would violate INV-2 (header IS present).
///
/// Traces to BC-2.01.009 EC-009, INV-7, AC-006. Canonical test vector row 4.
#[test]
fn test_BC_2_01_009_validate_canonical_empty_token_after_prefix() {
    // Prefix present, no hex suffix — empty string after stripping `monocle-v1:`.
    let result = validate_auth_header(Some(CANONICAL_PREFIX), None, TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with canonical header `monocle-v1:` (prefix only, no hex suffix) \
        must return Err(AuthError::InvalidToken); got: {result:?}. \
        EC-009: empty suffix passes prefix check, then fails comparison → value-present failure. \
        INV-7: comparison must run against sentinel even on length mismatch. \
        Traces to BC-2.01.009 EC-009 / INV-7 / AC-006. Test vector row 4."
    );
}

/// BC-2.01.009 EC-007 / AC-006: Canonical header with a completely empty value returns
/// `Err(AuthError::InvalidToken)`.
///
/// An empty string does not begin with `monocle-v1:` — value-present format failure.
/// Header IS present with an empty value — not the dual-absence missing-header case.
///
/// Counter-example guarded: returning `MissingToken` when the header is present but empty
/// would violate INV-2.
///
/// Traces to BC-2.01.009 EC-007, INV-2, AC-006.
#[test]
fn test_BC_2_01_009_validate_canonical_empty_value() {
    // Canonical header present but value is empty string.
    let result = validate_auth_header(Some(""), None, TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with canonical header empty value must return \
        Err(AuthError::InvalidToken), not MissingToken; got: {result:?}. \
        EC-007: header IS present (value-present failure); empty string ≠ `monocle-v1:` prefix. \
        Traces to BC-2.01.009 EC-007 / INV-2 / AC-006."
    );
}

/// BC-2.01.009 PC-2 / AC-006: Canonical header with the raw token value (no prefix) returns
/// `Err(AuthError::InvalidToken)`.
///
/// The canonical header ALWAYS requires the `monocle-v1:` prefix per BC-2.01.008 PC-2. Sending
/// the raw 64-hex without the prefix to `X-Monocle-Authorization` is a format failure. This is
/// true even if the raw hex happens to match the stored secret — the prefix is mandatory.
///
/// Traces to BC-2.01.009 PC-2, BC-2.01.008 PC-2, AC-006. Test vector row 2.
#[test]
fn test_BC_2_01_009_validate_canonical_raw_hex_no_prefix() {
    // Correct token value but no monocle-v1: prefix — sent as if it were the alias format.
    let result = validate_auth_header(Some(TEST_TOKEN), None, TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with canonical header raw-hex (no monocle-v1: prefix) must return \
        Err(AuthError::InvalidToken); got: {result:?}. \
        The canonical header ALWAYS requires the monocle-v1: prefix per BC-2.01.008 PC-2. \
        Traces to BC-2.01.009 PC-2 / BC-2.01.008 PC-2 / AC-006. Test vector row 2."
    );
}

/// BC-2.01.009 PC-2 / AC-006: Canonical header with the `monocle-v1:` prefix followed by a
/// wrong-version (`monocle-v2:`) as the token value.
///
/// This is a distinct case from `test_BC_2_01_009_validate_canonical_bad_prefix` — here the
/// canonical header value begins with `monocle-v1:` but the hex suffix is also wrong.
///
/// Traces to BC-2.01.009 PC-2 / AC-006.
#[test]
fn test_BC_2_01_009_validate_canonical_wrong_version_prefix() {
    // The canonical header value starts with the correct monocle-v1: prefix but the hex suffix
    // is all zeros — does not match TEST_TOKEN (all 'a's).
    let canonical_with_zeros = format!("{CANONICAL_PREFIX}{ZERO_TOKEN}");
    let result = validate_auth_header(Some(&canonical_with_zeros), None, TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with canonical `monocle-v1:<zero-64-hex>` (correct prefix, wrong \
        value) must return Err(AuthError::InvalidToken); got: {result:?}. \
        Traces to BC-2.01.009 PC-2 / AC-006."
    );
}

// ---------------------------------------------------------------------------
// Alias format failures
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-3 / AC-005: Alias header with non-hex characters in the token value returns
/// `Err(AuthError::InvalidToken)`.
///
/// The alias path expects raw 64-hex (`^[0-9a-f]{64}$`). A value containing non-hex characters
/// fails validation. Per BC-2.01.008 INV-7 / S-009 F-D-01, the constant_time_eq comparison
/// MUST still run against a fixed-length sentinel to prevent a timing oracle.
///
/// Traces to BC-2.01.009 PC-3, BC-2.01.008 INV-7, AC-005.
#[test]
fn test_BC_2_01_009_validate_alias_non_hex() {
    // Non-hex characters in the alias value — contains 'g' through 'z'.
    let non_hex_value = "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg";
    let result = validate_auth_header(None, Some(non_hex_value), TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with alias header containing non-hex characters must return \
        Err(AuthError::InvalidToken); got: {result:?}. \
        Alias path expects raw ^[0-9a-f]{{64}}$; non-hex is a value-present failure. \
        INV-7: constant_time_eq must still run against sentinel to prevent timing oracle. \
        Traces to BC-2.01.009 PC-3 / BC-2.01.008 INV-7 / AC-005."
    );
}

/// BC-2.01.009 PC-3 / AC-005: Alias header with a 32-character hex token (half the required
/// length) returns `Err(AuthError::InvalidToken)`.
///
/// The alias path requires exactly 64 hex chars. A length mismatch is a value-present failure.
/// Per BC-2.01.008 INV-7 / S-009 F-D-01, the constant_time_eq comparison MUST still run
/// against a fixed-length sentinel even on a length mismatch to avoid an early-return timing
/// oracle.
///
/// Traces to BC-2.01.009 PC-3, BC-2.01.008 INV-7, AC-005.
#[test]
fn test_BC_2_01_009_validate_alias_wrong_length() {
    // 32 hex chars — exactly half the required 64-char length.
    let short_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // 40 chars for a different wrong length
    assert_ne!(
        short_token.len(),
        64,
        "sanity: this value must not be 64 chars"
    );
    let result = validate_auth_header(None, Some(short_token), TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with alias header of wrong length ({} chars, expected 64) must \
        return Err(AuthError::InvalidToken); got: {result:?}. \
        INV-7: constant_time_eq must run against sentinel even on length mismatch (timing-oracle \
        defense per BC-2.01.008 INV-7 / F-D-01). \
        Traces to BC-2.01.009 PC-3 / BC-2.01.008 INV-7 / AC-005.",
        short_token.len()
    );
}

/// BC-2.01.009 PC-3 / AC-005: Alias header with a 64-char all-zeros value (valid hex format,
/// correct length) that does not match the stored token returns `Err(AuthError::InvalidToken)`.
///
/// This tests the happy-path format (correct length, valid hex characters) but wrong value.
/// Ensures constant-time comparison correctly distinguishes a structurally-valid but
/// semantically-wrong alias token.
///
/// Traces to BC-2.01.009 PC-3 / AC-005.
#[test]
fn test_BC_2_01_009_validate_alias_64_all_zeros_wrong() {
    // ZERO_TOKEN is a valid 64-hex string but does not match TEST_TOKEN (all 'a').
    let result = validate_auth_header(None, Some(ZERO_TOKEN), TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with alias header `{ZERO_TOKEN}` (64 zeros, correct format but \
        wrong value) must return Err(AuthError::InvalidToken); got: {result:?}. \
        Traces to BC-2.01.009 PC-3 / AC-005."
    );
}

/// BC-2.01.009 PC-2 / AC-006: Canonical header with a `monocle-v1:` prefix followed by
/// 64 all-zero hex chars (correct format, wrong token value) returns `Err(AuthError::InvalidToken)`.
///
/// Traces to BC-2.01.009 PC-2 / AC-006.
#[test]
fn test_BC_2_01_009_validate_canonical_64_zeros_prefix_wrong() {
    let canonical_with_zeros = format!("{CANONICAL_PREFIX}{ZERO_TOKEN}");
    // Explicitly different from CANONICAL_VALUE_CORRECT (which uses TEST_TOKEN = all 'a').
    assert_ne!(canonical_with_zeros, CANONICAL_VALUE_CORRECT);
    let result = validate_auth_header(Some(&canonical_with_zeros), None, TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with canonical `monocle-v1:<64-zeros>` (valid format, wrong value) \
        must return Err(AuthError::InvalidToken); got: {result:?}. \
        Constant-time comparison must detect the value mismatch. \
        Traces to BC-2.01.009 PC-2 / AC-006."
    );
}

// ---------------------------------------------------------------------------
// Priority corner case: wrong canonical + correct alias → InvalidToken (canonical wins)
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-4 / INV-5 / AC-007: When BOTH headers are present, canonical takes priority
/// even if canonical is wrong and alias would be correct.
///
/// Decision matrix: canonical present (wrong value) → `Err(InvalidToken)`. Alias is not
/// evaluated when canonical is present, regardless of alias value.
///
/// Counter-example guarded: an implementation that falls back to the alias when canonical fails
/// would return `Ok(Alias)` — violating INV-5 (canonical priority is immutable).
///
/// Traces to BC-2.01.009 PC-4 / INV-5 / AC-007.
#[test]
fn test_BC_2_01_009_validate_both_present_canonical_bad_alias_correct() {
    // Wrong canonical (monocle-v1: prefix + wrong value), correct alias.
    // Canonical priority must mean this returns InvalidToken, not Ok(Alias).
    let result = validate_auth_header(Some(CANONICAL_VALUE_WRONG), Some(TEST_TOKEN), TEST_TOKEN);

    assert_eq!(
        result,
        Err(AuthError::InvalidToken),
        "validate_auth_header with wrong canonical + correct alias must return \
        Err(AuthError::InvalidToken); got: {result:?}. \
        INV-5: canonical priority is immutable — alias is NOT evaluated when canonical is present. \
        Counter-example: falling back to alias would return Ok(Alias) — violates INV-5. \
        Traces to BC-2.01.009 PC-4 / INV-5 / AC-007."
    );
}

// ---------------------------------------------------------------------------
// Discriminant integrity: Ok(Alias) ≠ Ok(Canonical)
// ---------------------------------------------------------------------------

/// BC-2.01.009 PC-3 / AC-005: Alias success returns `Ok(AuthPath::Alias)`, NOT `Ok(AuthPath::Canonical)`.
///
/// The `AuthPath` discriminant tells the caller (`auth_middleware`) whether to emit the ADR-0005
/// WARN log. An implementation that returns `Ok(Canonical)` on the alias path would suppress the
/// WARN log, violating INV-6.
///
/// Traces to BC-2.01.009 PC-3 / INV-6 / AC-005.
#[test]
fn test_BC_2_01_009_validate_alias_returns_alias_variant_not_canonical() {
    let result = validate_auth_header(None, Some(TEST_TOKEN), TEST_TOKEN);

    // First assert it's Ok at all.
    assert!(
        result.is_ok(),
        "alias with correct token must succeed; got: {result:?}"
    );

    // Then assert the discriminant is Alias, not Canonical.
    assert_eq!(
        result.as_ref().unwrap(),
        &AuthPath::Alias,
        "alias success must return AuthPath::Alias (not Canonical); got: {result:?}. \
        INV-6: caller must emit WARN when AuthPath::Alias is returned; returning Canonical would \
        suppress the WARN log, violating INV-6. \
        Traces to BC-2.01.009 PC-3 / INV-6 / AC-005."
    );
    assert_ne!(
        result.as_ref().unwrap(),
        &AuthPath::Canonical,
        "alias success must NOT return AuthPath::Canonical"
    );
}

/// BC-2.01.009 PC-2 / AC-006: Canonical success returns `Ok(AuthPath::Canonical)`, NOT
/// `Ok(AuthPath::Alias)`.
///
/// An implementation that returns `Ok(Alias)` on the canonical path would cause the caller to
/// emit a spurious WARN log, violating INV-5/INV-6 (WARN is only for alias-path auth).
///
/// Traces to BC-2.01.009 PC-2 / INV-5 / AC-006.
#[test]
fn test_BC_2_01_009_validate_canonical_returns_canonical_variant_not_alias() {
    let result = validate_auth_header(Some(CANONICAL_VALUE_CORRECT), None, TEST_TOKEN);

    // First assert it's Ok at all.
    assert!(
        result.is_ok(),
        "canonical with correct token must succeed; got: {result:?}"
    );

    // Then assert the discriminant is Canonical, not Alias.
    assert_eq!(
        result.as_ref().unwrap(),
        &AuthPath::Canonical,
        "canonical success must return AuthPath::Canonical (not Alias); got: {result:?}. \
        Traces to BC-2.01.009 PC-2 / INV-5 / AC-006."
    );
    assert_ne!(
        result.as_ref().unwrap(),
        &AuthPath::Alias,
        "canonical success must NOT return AuthPath::Alias (no WARN log required)"
    );
}

// ---------------------------------------------------------------------------
// INV-2: missing and invalid are distinct error discriminants
// ---------------------------------------------------------------------------

/// BC-2.01.009 INV-2: `AuthError::MissingToken` and `AuthError::InvalidToken` are distinct
/// values — verifying the discriminant separation at the type level.
///
/// This is the function-level complement to the HTTP integration test in `status_endpoint_auth.rs`
/// that verifies the response bodies are distinct. At the unit level, the `AuthError` enum
/// discriminants must be `!=`.
///
/// Traces to BC-2.01.009 INV-2.
#[test]
fn test_BC_2_01_009_invariant_missing_and_invalid_are_distinct() {
    let missing = validate_auth_header(None, None, TEST_TOKEN);
    let invalid = validate_auth_header(Some(CANONICAL_VALUE_WRONG), None, TEST_TOKEN);

    // Both must be Err variants.
    assert!(
        missing.is_err(),
        "both-absent must return Err; got: {missing:?}"
    );
    assert!(
        invalid.is_err(),
        "wrong-token must return Err; got: {invalid:?}"
    );

    // They must be DIFFERENT Err variants.
    assert_ne!(
        missing, invalid,
        "AuthError::MissingToken and AuthError::InvalidToken must be distinct discriminants; \
        both returned: {missing:?}. \
        INV-2: the missing/invalid distinction provides developer-friendly diagnostics. \
        Traces to BC-2.01.009 INV-2."
    );

    // Explicit discriminant checks.
    assert_eq!(
        missing,
        Err(AuthError::MissingToken),
        "both-absent must be Err(MissingToken), not Err(InvalidToken)"
    );
    assert_eq!(
        invalid,
        Err(AuthError::InvalidToken),
        "wrong-token must be Err(InvalidToken), not Err(MissingToken)"
    );
}

// ---------------------------------------------------------------------------
// INV-6: Ok(Alias) implies caller MUST emit WARN; Ok(Canonical) implies no WARN
// ---------------------------------------------------------------------------

/// BC-2.01.009 INV-6 / AC-005: The `Ok(AuthPath::Alias)` variant is the WARN signal to the
/// caller. This test verifies that the return value of `validate_auth_header` on the alias
/// success path carries the correct discriminant for the caller to dispatch the WARN.
///
/// `validate_auth_header` itself does NOT emit the WARN (by design — pure function). The
/// caller (`auth_middleware`) emits the WARN when `Ok(AuthPath::Alias)` is returned.
/// This test confirms the contract: alias success → `Alias` discriminant, NOT `Canonical`.
///
/// Traces to BC-2.01.009 INV-6 / AC-005.
#[test]
fn test_BC_2_01_009_invariant_warn_log_flag_is_alias_path() {
    // Alias success → must return Alias (caller will emit WARN).
    let alias_result = validate_auth_header(None, Some(TEST_TOKEN), TEST_TOKEN);
    // Canonical success → must return Canonical (caller must NOT emit WARN).
    let canonical_result = validate_auth_header(Some(CANONICAL_VALUE_CORRECT), None, TEST_TOKEN);

    assert_eq!(
        alias_result,
        Ok(AuthPath::Alias),
        "alias-path success must return Ok(AuthPath::Alias) so the caller emits the WARN log; \
        got: {alias_result:?}. \
        INV-6: WARN log emitted once per alias-path attempt regardless of outcome. \
        validate_auth_header is pure — the caller dispatches WARN based on this discriminant."
    );

    assert_eq!(
        canonical_result,
        Ok(AuthPath::Canonical),
        "canonical-path success must return Ok(AuthPath::Canonical) — no WARN log warranted; \
        got: {canonical_result:?}. \
        INV-6: WARN is alias-path only. Returning Alias on canonical path would cause spurious WARN."
    );

    // Confirm they are different discriminants.
    assert_ne!(
        alias_result, canonical_result,
        "alias success and canonical success must return DIFFERENT AuthPath variants; \
        both returned: {alias_result:?}"
    );
}

// ---------------------------------------------------------------------------
// VP-008 source audit: no `==` on secret bytes in auth.rs
// ---------------------------------------------------------------------------

/// VP-008 / AC-008: Source-grep audit on `auth.rs` verifying that `==` is never used
/// directly on secret token bytes — only `constant_time_eq::constant_time_eq` is used.
///
/// This is a static analysis test (does not invoke `validate_auth_header`). It prevents timing
/// oracle attacks by ensuring the implementation does not short-circuit via the `PartialEq`
/// operator on secret bytes.
///
/// Specifically, we check that no non-comment line in `auth.rs` contains the pattern
/// `== state.auth_token` or `== expected_token` or `== token` or `==` adjacent to
/// well-known secret-holding variables.
///
/// The positive assertion (constant_time_eq IS used) is in
/// `test_BC_2_01_009_vp_009_source_grep_constant_time_eq_on_alias_path`.
///
/// Traces to VP-008 / BC-2.01.009 INV-7 / NFR-010 / AC-008.
#[test]
fn test_BC_2_01_008_vp_008_source_grep_no_eq_on_secret_bytes() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let auth_src =
        fs::read_to_string(manifest_dir.join("src/auth.rs")).expect("src/auth.rs must exist");

    // Patterns that indicate direct `==` comparison on secret bytes.
    // These are the specific patterns we guard against:
    //  1. `== state.auth_token` — comparing against DaemonState field directly.
    //  2. `== expected_token` — comparing against function parameter.
    //  3. `hex_suffix ==` or `== hex_suffix` — comparing the stripped canonical suffix.
    //  4. `value_str ==` or `== value_str` — comparing raw alias/canonical value string.
    let forbidden_patterns = [
        "== state.auth_token",
        "state.auth_token ==",
        "== expected_token",
        "expected_token ==",
        "hex_suffix ==",
        "== hex_suffix",
        "value_str ==",
        "== value_str",
    ];

    for pattern in &forbidden_patterns {
        let hits: Vec<(usize, &str)> = auth_src
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                let t = line.trim_start();
                !t.starts_with("//") && line.contains(pattern)
            })
            .map(|(i, line)| (i + 1, line))
            .collect();

        assert!(
            hits.is_empty(),
            "src/auth.rs MUST NOT use `==` directly on secret token bytes (NFR-010 \
            timing-attack resistance; VP-008). Found forbidden pattern `{pattern}` at \
            non-comment lines: {hits:?}. All comparisons must use \
            `constant_time_eq::constant_time_eq`. \
            Traces to VP-008 / BC-2.01.009 INV-7 / NFR-010 / AC-008."
        );
    }
}

// ---------------------------------------------------------------------------
// VP-009 source audit: constant_time_eq present on alias path in auth.rs
// ---------------------------------------------------------------------------

/// VP-009 / AC-008: Source-grep audit on `auth.rs` verifying that `constant_time_eq`
/// (the crate function) is present in non-comment executable code.
///
/// This positive assertion complements `test_BC_2_01_008_vp_008_source_grep_no_eq_on_secret_bytes`
/// (the negative assertion). Both are required to verify VP-008/VP-009: `constant_time_eq`
/// must be used AND `==` must not be used.
///
/// Traces to VP-009 / BC-2.01.009 INV-7 / NFR-010 / AC-008.
#[test]
fn test_BC_2_01_009_vp_009_source_grep_constant_time_eq_on_alias_path() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let auth_src =
        fs::read_to_string(manifest_dir.join("src/auth.rs")).expect("src/auth.rs must exist");

    // Positive assertion: constant_time_eq must appear in non-comment executable code.
    let cts_hits: Vec<(usize, String)> = auth_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            !t.starts_with("//") && line.contains("constant_time_eq")
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        !cts_hits.is_empty(),
        "src/auth.rs must use `constant_time_eq` in executable code for token comparison \
        (NFR-010 timing-attack resistance on BOTH canonical and alias paths). \
        No non-comment `constant_time_eq` usage found in auth.rs. \
        Traces to VP-009 / BC-2.01.009 INV-7 / NFR-010 / AC-008."
    );

    // Must appear at least twice: once for canonical path, once for alias path.
    // This verifies BC-2.01.009 INV-7: "comparison algorithm is identical regardless of
    // which header is used."
    // A single occurrence would mean one path uses == instead of constant_time_eq.
    assert!(
        cts_hits.len() >= 2,
        "src/auth.rs must use `constant_time_eq` at least twice (once per comparison path: \
        canonical AND alias per BC-2.01.009 INV-7). Found only {} occurrence(s) at: {:?}. \
        A single occurrence means one path may be using `==` instead of constant_time_eq. \
        Traces to VP-009 / BC-2.01.009 INV-7 / NFR-010.",
        cts_hits.len(),
        cts_hits
    );
}

// ---------------------------------------------------------------------------
// Uppercase hex rejection (AC-005/AC-006 charset constraint: ^[0-9a-f]{64}$ — lowercase only)
// ---------------------------------------------------------------------------

/// BC-2.01.009 AC-005: The alias path requires raw lowercase hex (`^[0-9a-f]{64}$`).
/// Uppercase hex digits (A-F) are NOT valid — they are valid hex numerals but violate the
/// lowercase constraint. An alias token of 64 uppercase hex chars must be rejected with
/// `Err(AuthError::InvalidToken)`.
///
/// This guards against an implementation that accepts `[0-9a-fA-F]{64}` instead of the
/// strictly lowercase `[0-9a-f]{64}` required by BC-2.01.009 PC-3 and BC-2.01.008 AC-005.
///
/// Counter-example: accepting uppercase would allow a token value that byte-differs from
/// the stored lowercase secret to pass constant-time comparison only if the comparison
/// is case-insensitive — a security defect.
///
/// Traces to BC-2.01.009 AC-005 / BC-2.01.008 charset invariant.
#[test]
fn test_validate_uppercase_hex_rejected() {
    let token = "a".repeat(64);
    // Valid hex digits but uppercase — must be rejected (charset: ^[0-9a-f]{64}$).
    let uppercase = "A".repeat(64);
    let result = validate_auth_header(None, Some(&uppercase), &token);
    assert!(
        matches!(result, Err(AuthError::InvalidToken)),
        "validate_auth_header with alias = 64 uppercase hex chars must return \
        Err(AuthError::InvalidToken); got: {result:?}. \
        The alias charset is strictly ^[0-9a-f]{{64}}$ (lowercase). \
        Uppercase hex digits violate the constraint per BC-2.01.009 AC-005 / BC-2.01.008."
    );
}

// ---------------------------------------------------------------------------
// BC-2.01.008 INV-7 / F-D-01: Length mismatch must use sentinel (timing-oracle defense)
// ---------------------------------------------------------------------------

/// BC-2.01.008 INV-7 / F-D-01: Source-grep audit verifying that `validate_auth_header`
/// uses a fixed-length sentinel for length-mismatched inputs rather than short-circuiting
/// before the `constant_time_eq` call.
///
/// A naive implementation might check `if hex_suffix.len() != 64 { return Err(InvalidToken); }`
/// before calling `constant_time_eq`. This creates a timing oracle: an attacker can measure
/// whether the prefix check passed (long response) vs. length check failed (short response),
/// leaking whether the canonical prefix was correct.
///
/// The production-grade implementation:
/// 1. Validates the format (prefix, length).
/// 2. BUT still calls `constant_time_eq(input, sentinel)` before returning `Err(InvalidToken)`,
///    where `sentinel` is a fixed-length `[0u8; 64]` or equivalent.
///
/// This source-grep test verifies a fixed-length sentinel is defined in auth.rs to support
/// this timing-safe path.
///
/// Traces to BC-2.01.008 INV-7 / BC-2.01.009 INV-7 / F-D-01 (S-009 §Trace v1.8).
#[test]
fn test_BC_2_01_008_vp_008_length_mismatch_uses_sentinel() {
    use std::fs;
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let auth_src =
        fs::read_to_string(manifest_dir.join("src/auth.rs")).expect("src/auth.rs must exist");

    // We check for the presence of the sentinel byte array declaration in non-comment,
    // non-string-literal code. The only production-grade pattern is `[0u8; 64]` used
    // as an actual runtime value (not inside a string literal or doc comment).
    //
    // We look for `[0u8; 64]` in lines that:
    //   1. Are not doc/line comments (do not start with `//` after trim).
    //   2. Are not pure string content — specifically, the line must contain `[0u8; 64]`
    //      outside of a string literal context. We approximate this by requiring the
    //      line to NOT consist solely of a string assignment or panic! / todo! argument.
    //
    // The sentinel detection specifically requires `[0u8; 64]` — the fixed-size byte
    // array type. Variable names alone (e.g., `sentinel`) are insufficient because they
    // could refer to anything; only the concrete byte-array type guarantees the correct
    // sentinel semantics.
    let sentinel_hits: Vec<(usize, String)> = auth_src
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let t = line.trim_start();
            // Exclude doc comments (///) and line comments (//).
            if t.starts_with("//") {
                return false;
            }
            // Must contain the fixed-length sentinel byte array type.
            if !line.contains("[0u8; 64]") {
                return false;
            }
            // Exclude lines where [0u8; 64] appears only inside a string literal.
            // A string literal context is indicated by the pattern being enclosed in
            // double-quotes or being part of a macro string argument. We detect this
            // by checking whether the line contains an actual binding or expression
            // using the type — i.e., `let`, `=`, or direct use as a function argument
            // without being wrapped in a string.
            // If the line contains `"` before `[0u8; 64]`, it's likely in a string.
            let before_sentinel = line.split("[0u8; 64]").next().unwrap_or("");
            let open_quote_count = before_sentinel.chars().filter(|&c| c == '"').count();
            // Odd number of open quotes before the sentinel means we're inside a string literal.
            open_quote_count % 2 == 0
        })
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect();

    assert!(
        !sentinel_hits.is_empty(),
        "src/auth.rs must define a fixed-length sentinel `[0u8; 64]` in executable code (not \
        inside a string literal or comment) for the length-mismatch constant_time_eq call per \
        BC-2.01.008 INV-7 and F-D-01 (S-009 §Trace v1.8). \
        Without a sentinel, length-mismatched inputs can short-circuit before constant_time_eq, \
        creating a timing oracle (an attacker learns whether the canonical prefix was correct). \
        No `[0u8; 64]` sentinel found in non-comment, non-string-literal executable lines. \
        Traces to BC-2.01.008 INV-7 / BC-2.01.009 INV-7 / F-D-01."
    );
}
