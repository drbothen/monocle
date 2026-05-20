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
pub mod abi {}
