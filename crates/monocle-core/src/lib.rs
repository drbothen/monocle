//! monocle-core — Core types, traits, and ABI for the monocle workspace.
//!
//! Phase 1 module layout per `SS-core-types-and-abi.md` v1.2.13 §Module Layout.
//! Stubs created in S-001; populated by S-011, S-012, S-013, S-014.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// Engine module trait abstraction (populated by S-014: EngineModule trait).
pub mod engine {}

/// Factory adapter abstraction (populated by S-012: FactoryAdapter trait).
pub mod factory {}

/// ABI surface for cross-crate stability (populated by S-011: #[non_exhaustive] policy).
pub mod abi {
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
}

// Re-export at crate root per BC-2.02.002 postcondition 2 (S-010 canonical owner;
// S-003 references via `monocle_core::MONOCLE_ABI_VERSION`).
pub use abi::MONOCLE_ABI_VERSION;
