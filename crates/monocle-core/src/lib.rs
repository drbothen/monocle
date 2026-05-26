//! monocle-core — Core types, traits, and ABI for the monocle workspace.
//!
//! Phase 1 module layout per `SS-core-types-and-abi.md` v1.2.13 §Module Layout.
//! Stubs created in S-001; populated by S-011, S-012, S-013, S-014.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// Engine module trait abstraction and supporting types (S-014: EngineModule trait).
pub mod engine;

/// Hook event types for the harness plane (S-014: HookEvent enum).
pub mod hook_events;

/// Factory adapter abstraction (populated by S-012: FactoryAdapter trait).
pub mod factory {}

/// ABI surface for cross-crate stability (populated by S-011: #[non_exhaustive] policy).
pub mod abi;

// Re-export at crate root per BC-2.02.002 postcondition 2 (S-010 canonical owner;
// S-003 references via `monocle_core::MONOCLE_ABI_VERSION`).
pub use abi::MONOCLE_ABI_VERSION;
