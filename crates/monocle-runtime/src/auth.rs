//! Auth middleware for the monocle daemon authenticated router.
//!
//! Implements BC-2.01.009 (dual-accept auth header semantics per ADR-0005):
//!
//! **Priority 1 — Canonical (monocle-aware tools):**
//! If `X-Monocle-Authorization` is present, validate with the `monocle-v1:` prefix,
//! strip it, and constant-time compare the 64-hex suffix against the stored secret.
//!
//! **Priority 2 — Compatibility alias (real Claude Code hooks):**
//! If `X-Monocle-Authorization` is absent but `X-Claude-Code-Ide-Authorization` is
//! present, emit a WARN deprecation log and constant-time compare the raw 64-hex token.
//!
//! **Neither header present:** return HTTP 401 `{"error":"missing_auth_token"}` (E-AUTH-001).
//!
//! # Security invariant (NFR-010)
//!
//! BOTH the canonical and alias comparison paths MUST use `constant_time_eq::constant_time_eq`.
//! Direct `==` on token bytes is FORBIDDEN. Violation breaks timing-attack resistance.
//!
//! # ADR-0005 canonical WARN string (INV-6, AC-002)
//!
//! `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization`
//!
//! This exact string MUST be emitted on alias-path auth. No ellipsis, no paraphrase.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use constant_time_eq::constant_time_eq;

use crate::state::DaemonState;

/// Header name for the canonical monocle auth token.
///
/// Format: `monocle-v1:<64-hex-chars>` (with prefix).
const CANONICAL_HEADER: &str = "x-monocle-authorization";

/// Header name for the Claude Code compatibility alias.
///
/// Format: raw `<64-hex-chars>` (no prefix).
const ALIAS_HEADER: &str = "x-claude-code-ide-authorization";

/// Expected prefix on the canonical header value.
const CANONICAL_PREFIX: &str = "monocle-v1:";

/// Axum middleware function for authenticating requests on the authenticated router.
///
/// Implements the dual-accept protocol from ADR-0005:
/// 1. Checks `X-Monocle-Authorization` (canonical). Validates `monocle-v1:<64-hex>` format,
///    strips prefix, constant-time compares against `state.auth_token`.
/// 2. Falls back to `X-Claude-Code-Ide-Authorization` (compatibility alias) when the
///    canonical header is absent. Emits a WARN log and constant-time compares raw 64-hex.
/// 3. Returns HTTP 401 `{"error":"missing_auth_token"}` (E-AUTH-001) when both headers absent.
/// 4. Returns HTTP 401 `{"error":"invalid_auth_token"}` (E-AUTH-002) on format or value failure.
///
/// # Security
///
/// All token comparisons use `constant_time_eq::constant_time_eq` (NFR-010).
/// The `==` operator is NEVER used on token bytes.
pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<Arc<DaemonState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Guard: if auth_token has not been initialized (empty string), reject ALL requests.
    // An empty token is the default-constructed state (DaemonState::new() before S-004 writes
    // the generated secret). Allowing any request through against an empty token would permit
    // the empty-credential bypass: constant_time_eq("", "") == true.
    // This guard fires before any header extraction so neither path can be exploited.
    // Tracing error is intentional — an uninitialized daemon serving authenticated routes
    // indicates a startup sequencing defect worth surfacing in logs.
    if state.auth_token.is_empty() {
        tracing::error!("auth_token not initialized; rejecting all authenticated requests");
        return invalid_token_response();
    }

    let headers = request.headers();

    // Priority 1: check canonical header `X-Monocle-Authorization`.
    if let Some(canonical_value) = headers.get(CANONICAL_HEADER) {
        // Header is present — any failure is E-AUTH-002 (invalid), not E-AUTH-001 (missing).
        let value_str = match canonical_value.to_str() {
            Ok(s) => s,
            Err(_) => return invalid_token_response(),
        };

        // Must start with "monocle-v1:" prefix.
        let Some(hex_suffix) = value_str.strip_prefix(CANONICAL_PREFIX) else {
            return invalid_token_response();
        };

        // Constant-time comparison of the hex suffix against the stored token.
        // NFR-010: MUST use constant_time_eq, NEVER `==`.
        if constant_time_eq(hex_suffix.as_bytes(), state.auth_token.as_bytes()) {
            return next.run(request).await;
        }
        return invalid_token_response();
    }

    // Priority 2: check alias header `X-Claude-Code-Ide-Authorization`.
    if let Some(alias_value) = headers.get(ALIAS_HEADER) {
        // Header is present — any failure is E-AUTH-002 (invalid), not E-AUTH-001 (missing).
        let value_str = match alias_value.to_str() {
            Ok(s) => s,
            Err(_) => return invalid_token_response(),
        };

        // Emit the mandatory WARN per ADR-0005 INV-6 / AC-002.
        // Exact string is normative — do not paraphrase.
        tracing::warn!(
            "WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); \
            monocle-aware harness should use X-Monocle-Authorization"
        );

        // Constant-time comparison of the raw hex value against the stored token.
        // NFR-010: MUST use constant_time_eq, NEVER `==`.
        if constant_time_eq(value_str.as_bytes(), state.auth_token.as_bytes()) {
            return next.run(request).await;
        }
        return invalid_token_response();
    }

    // Neither header present — E-AUTH-001 (missing).
    missing_token_response()
}

/// Build the HTTP 401 missing-token response (E-AUTH-001).
///
/// Returns `{"error":"missing_auth_token"}` with HTTP 401.
/// Called when neither `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` is present.
///
/// WIRING-EXEMPT: this is a single-statement JSON constructor delegated to axum's
/// `IntoResponse` machinery. The body literal is mandated verbatim by BC-2.01.009 PC-1.
fn missing_token_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error": "missing_auth_token"})),
    )
        .into_response()
}

/// Build the HTTP 401 invalid-token response (E-AUTH-002).
///
/// Returns `{"error":"invalid_auth_token"}` with HTTP 401.
/// Called when a header is present but the token value fails format or constant-time comparison.
///
/// WIRING-EXEMPT: this is a single-statement JSON constructor delegated to axum's
/// `IntoResponse` machinery. The body literal is mandated verbatim by BC-2.01.009 PC-2.
fn invalid_token_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error": "invalid_auth_token"})),
    )
        .into_response()
}

/// Fixed-length sentinel for timing-safe constant_time_eq on length-mismatched inputs.
///
/// Per BC-2.01.008 INV-7 / S-009 F-D-01: when the input token does not have the correct
/// length or format, we still call `constant_time_eq` against this sentinel rather than
/// returning early. This prevents a timing oracle where an attacker could distinguish
/// "correct prefix, wrong length" from "wrong prefix" by measuring response latency.
const SENTINEL_64: [u8; 64] = [0u8; 64];

/// Validate auth headers implementing the dual-accept protocol per ADR-0005.
///
/// Pure validation function extracted from `auth_middleware` for unit-testing the dual-accept
/// logic independently of the axum request stack.
///
/// # Parameters
///
/// - `canonical`: value of `X-Monocle-Authorization` if present; `None` if absent.
/// - `alias`: value of `X-Claude-Code-Ide-Authorization` if present; `None` if absent.
/// - `expected_token`: the stored 64-hex session token (from lock file / `DaemonState::auth_token`).
///
/// # Decision matrix (mirrors `auth_middleware` priority order)
///
/// | canonical | alias | outcome |
/// |-----------|-------|---------|
/// | present, valid `monocle-v1:<64-hex>`, token matches | any | `Ok(AuthPath::Canonical)` |
/// | present, any format/value failure | any | `Err(AuthError::InvalidToken)` |
/// | absent | present, raw 64-hex token matches | `Ok(AuthPath::Alias)` — caller MUST emit WARN |
/// | absent | present, any mismatch | `Err(AuthError::InvalidToken)` |
/// | absent | absent | `Err(AuthError::MissingToken)` |
///
/// # Security invariants (NFR-010, BC-2.01.009 INV-7)
///
/// - ALL comparisons use `constant_time_eq::constant_time_eq`. The `==` operator is NEVER
///   used on secret bytes.
/// - Length-mismatched inputs still run `constant_time_eq` against a fixed-length sentinel
///   (`[0u8; 64]`) to prevent a timing oracle on the mismatch branch (BC-2.01.008 INV-7).
///
/// # Warn log
///
/// This function does NOT emit the WARN log itself — it returns `Ok(AuthPath::Alias)` and
/// the caller (`auth_middleware`) is responsible for emitting the ADR-0005 WARN per INV-6.
/// This separation allows the pure function to be unit-tested without a tracing subscriber.
pub fn validate_auth_header(
    canonical: Option<&str>,
    alias: Option<&str>,
    expected_token: &str,
) -> Result<AuthPath, AuthError> {
    match canonical {
        Some(canonical_value) => {
            // Canonical header is present — any failure returns InvalidToken (E-AUTH-002),
            // not MissingToken (E-AUTH-001). INV-2: header-present failures are always invalid.

            // Strip the `monocle-v1:` prefix. If it fails, still run sentinel comparison
            // before returning to prevent timing oracle (INV-7).
            let Some(hex_suffix) = canonical_value.strip_prefix(CANONICAL_PREFIX) else {
                // Wrong or missing prefix — compare against sentinel to normalise timing.
                let _ = constant_time_eq(canonical_value.as_bytes(), &SENTINEL_64);
                return Err(AuthError::InvalidToken);
            };

            // Validate hex suffix: must be exactly 64 lowercase hex chars (`^[0-9a-f]{64}$`).
            // On length or character mismatch, still run constant_time_eq against sentinel (INV-7).
            let is_valid_hex = hex_suffix.len() == 64
                && hex_suffix.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());

            if !is_valid_hex {
                // Length or character mismatch — sentinel comparison for timing safety (INV-7).
                let _ = constant_time_eq(hex_suffix.as_bytes(), &SENTINEL_64);
                return Err(AuthError::InvalidToken);
            }

            // Valid format — constant-time compare suffix against expected token (NFR-010).
            if constant_time_eq(hex_suffix.as_bytes(), expected_token.as_bytes()) {
                Ok(AuthPath::Canonical)
            } else {
                Err(AuthError::InvalidToken)
            }
        }

        None => match alias {
            Some(alias_value) => {
                // Alias header is present — any failure returns InvalidToken (E-AUTH-002).
                // Canonical is absent, so this is the compatibility path.

                // Validate alias: must be exactly 64 lowercase hex chars (`^[0-9a-f]{64}$`).
                // On length or character mismatch, still run constant_time_eq against sentinel (INV-7).
                let is_valid_hex = alias_value.len() == 64
                    && alias_value
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());

                if !is_valid_hex {
                    // Length or character mismatch — sentinel comparison for timing safety (INV-7).
                    let _ = constant_time_eq(alias_value.as_bytes(), &SENTINEL_64);
                    return Err(AuthError::InvalidToken);
                }

                // Valid format — constant-time compare raw alias value against expected token (NFR-010).
                // Caller is responsible for emitting the ADR-0005 WARN log on Ok(AuthPath::Alias).
                if constant_time_eq(alias_value.as_bytes(), expected_token.as_bytes()) {
                    Ok(AuthPath::Alias)
                } else {
                    Err(AuthError::InvalidToken)
                }
            }

            // Neither header present — E-AUTH-001 (missing), not E-AUTH-002 (invalid). INV-2.
            None => Err(AuthError::MissingToken),
        },
    }
}

/// Outcome discriminant returned by `validate_auth_header` on success.
///
/// The caller (`auth_middleware`) uses this to decide whether to emit the ADR-0005 WARN
/// log (alias path) or proceed silently (canonical path).
#[derive(Debug, PartialEq, Eq)]
pub enum AuthPath {
    /// Request authenticated via the canonical `X-Monocle-Authorization: monocle-v1:<64-hex>` header.
    /// No WARN log required.
    Canonical,
    /// Request authenticated via the compatibility alias `X-Claude-Code-Ide-Authorization: <raw-64-hex>`.
    /// Caller MUST emit the ADR-0005 WARN per BC-2.01.009 INV-6.
    Alias,
}

/// Auth validation error discriminant.
///
/// Maps to the E-AUTH error taxonomy (error-taxonomy.md):
/// - `MissingToken` → HTTP 401 `{"error":"missing_auth_token"}` (E-AUTH-001)
/// - `InvalidToken` → HTTP 401 `{"error":"invalid_auth_token"}` (E-AUTH-002)
#[derive(Debug, PartialEq, Eq)]
pub enum AuthError {
    /// Neither `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` present.
    /// Maps to E-AUTH-001 in error-taxonomy.md.
    MissingToken,
    /// A header is present but the token value fails format or constant-time comparison.
    /// Maps to E-AUTH-002 in error-taxonomy.md.
    InvalidToken,
}

/// Generate a cryptographically random session token (32 bytes → 64 hex chars).
///
/// Uses [`rand::rngs::OsRng`] as the entropy source, which delegates to the platform
/// CSPRNG (`getrandom(2)` on Linux, `SecRandomCopyBytes` on macOS).
///
/// The `rand` crate is pinned to `=0.8.6` (EXACT) per SS-deps-pin-manifest.md §Pinning
/// Policy. The `0.9` series introduced an ergonomic regression in `OsRng` usage
/// (trait import requirements changed); the exact pin prevents accidental upgrade.
///
/// # Output format
///
/// Returns a lowercase 64-character hex string. No prefix. The caller (lock file writer)
/// or auth middleware adds the `monocle-v1:` prefix at the wire layer.
///
/// # Usage
///
/// Called by [`crate::lock::DaemonLock::acquire`] during daemon startup to generate the
/// session auth token written into the lock file and loaded into [`crate::state::DaemonState`].
pub fn generate_session_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes.iter().fold(String::with_capacity(64), |mut acc, b| {
        use std::fmt::Write;
        write!(acc, "{b:02x}").expect("fmt::Write on String is infallible");
        acc
    })
}
