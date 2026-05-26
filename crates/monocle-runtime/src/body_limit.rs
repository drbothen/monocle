//! Body-size enforcement middleware for the authenticated router.
//!
//! `DefaultBodyLimit::max(262144)` tells extractors (like `Bytes`, `Json`) to reject bodies
//! larger than 256 KiB. However, it does NOT intercept requests at the middleware level — if a
//! handler does not read the body, no 413 is produced. This module provides an explicit
//! `from_fn` middleware that checks `Content-Length` before any handler runs and returns a
//! structured JSON 413 when the header indicates a body that exceeds the limit.
//!
//! # Design rationale
//!
//! `tower_http::limit::RequestBodyLimitLayer` provides content-length enforcement but emits a
//! plain-text `"length limit exceeded"` response. The behavioral contract for this daemon
//! (BC-2.01.003 Invariant 2 / SS-daemon-lifecycle.md §Body Size Limit) requires a JSON body:
//! `{"error":"payload_too_large","limit_bytes":262144}`. Using a custom `from_fn` middleware is
//! the minimal-dependency approach that keeps the JSON shape under our control.
//!
//! # Limitation
//!
//! Only `Content-Length`-bearing requests are eagerly rejected. Chunked-transfer requests
//! without `Content-Length` are not rejected here; the downstream extractor will enforce the
//! limit when reading the body via `DefaultBodyLimit`.
//!
//! # Registration
//!
//! Applied only to the authenticated router in `server::build_server`.
//! Not applied to the unauthenticated router (`/healthz`).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// The enforced body size limit in bytes: 256 KiB.
///
/// Per SS-daemon-lifecycle.md v1.0.33 §Body Size Limit.
pub const BODY_LIMIT_BYTES: usize = 262_144;

/// The exact JSON error body returned on 413 Payload Too Large.
///
/// Shape: `{"error":"payload_too_large","limit_bytes":262144}`
///
/// This is the canonical error body for BC-2.01.003 Invariant 2 (body size limit on
/// authenticated router). The shape is fixed and tested by the integration test suite.
#[derive(Debug, serde::Serialize)]
pub struct PayloadTooLargeBody {
    /// Error code identifying the rejection reason.
    pub error: &'static str,
    /// Enforced limit in bytes (matches the `DefaultBodyLimit::max` value in `server.rs`).
    pub limit_bytes: usize,
}

/// Axum `from_fn` middleware that rejects requests with `Content-Length > 262144`.
///
/// When `Content-Length` is present and exceeds [`BODY_LIMIT_BYTES`], this middleware
/// short-circuits the request pipeline and returns:
/// - Status: `413 Payload Too Large`
/// - Content-Type: `application/json`
/// - Body: `{"error":"payload_too_large","limit_bytes":262144}`
///
/// When `Content-Length` is absent or within the limit, the request is passed through
/// unchanged to the next middleware or handler.
///
/// # Registration
///
/// Applied on the authenticated router only via `server::build_server`:
/// ```ignore
/// .layer(axum::middleware::from_fn(body_size_limit_middleware))
/// ```
pub async fn body_size_limit_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let content_length: Option<usize> = request
        .headers()
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());

    if let Some(len) = content_length {
        if len > BODY_LIMIT_BYTES {
            tracing::warn!(
                content_length = len,
                limit_bytes = BODY_LIMIT_BYTES,
                "request body exceeds 256 KiB limit; rejecting with HTTP 413"
            );
            let body = PayloadTooLargeBody {
                error: "payload_too_large",
                limit_bytes: BODY_LIMIT_BYTES,
            };
            let response = (StatusCode::PAYLOAD_TOO_LARGE, Json(body)).into_response();
            return Ok(response);
        }
    }

    Ok(next.run(request).await)
}
