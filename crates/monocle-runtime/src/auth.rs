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

use crate::state::DaemonState;

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
    axum::extract::State(_state): axum::extract::State<Arc<DaemonState>>,
    _request: Request<Body>,
    _next: Next,
) -> Response {
    unimplemented!(
        "auth_middleware: dual-accept auth per ADR-0005 / BC-2.01.009. \
        Read X-Monocle-Authorization (canonical) then X-Claude-Code-Ide-Authorization \
        (alias, emit WARN). Constant-time compare with state.auth_token. \
        Return 401 E-AUTH-001 (missing) or E-AUTH-002 (invalid) on failure."
    )
}

/// Build the HTTP 401 missing-token response (E-AUTH-001).
///
/// Returns `{"error":"missing_auth_token"}` with HTTP 401.
/// Called when neither `X-Monocle-Authorization` nor `X-Claude-Code-Ide-Authorization` is present.
///
/// WIRING-EXEMPT: this is a single-statement JSON constructor delegated to axum's
/// `IntoResponse` machinery. The body literal is mandated verbatim by BC-2.01.009 PC-1.
#[allow(dead_code)]
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
#[allow(dead_code)]
fn invalid_token_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error": "invalid_auth_token"})),
    )
        .into_response()
}
