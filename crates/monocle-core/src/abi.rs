//! ABI surface for cross-crate stability.
//!
//! This module owns the canonical `MONOCLE_ABI_VERSION` constant. It is populated
//! as part of S-010; the `#[non_exhaustive]` policy is applied in S-011.

/// Monotonic ABI version of the monocle daemon wire protocol.
///
/// The value `1` identifies Phase 1 (Claude Code integration). This constant is read by
/// `monocle-runtime/src/handlers/status.rs` and exposed in the `/status` response body
/// as the `abi_version` field (BC-2.02.001 PC-1).
///
/// Callers may compile-time assert:
/// ```rust
/// const _: () = assert!(monocle_core::MONOCLE_ABI_VERSION == 1, "ABI version mismatch");
/// ```
///
/// S-010 is the canonical producer of this constant; S-003 exposes it via `/status`.
/// The value is intentionally `1` for Phase 1; it increments on incompatible wire-format
/// changes per the forward-compatibility policy in `SS-forward-compatibility.md`.
pub const MONOCLE_ABI_VERSION: u32 = 1;
