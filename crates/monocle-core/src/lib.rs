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

/// Factory adapter abstraction (BC-2.02.004, BC-2.02.005, S-012).
pub mod factory;

/// ABI surface for cross-crate stability (populated by S-011: #[non_exhaustive] policy).
pub mod abi;

/// Phase 1 permission dispatch surface for Claude Code hook integration.
///
/// Declares `Phase1Permission` and `ClaudeCodeTool` (exhaustive per ADR-0004) and
/// `DenyReason`, `AllowPattern`, `DenyPattern` (non-exhaustive per BC-2.02.003 / S-011).
pub mod permissions;

/// TUI plane — state machine types, key events, and binding resolution (S-024).
///
/// Pure-core: no ratatui, crossterm, or other I/O dependencies.
/// All terminal integration lives in the `monocle` binary crate.
pub mod tui;

/// Pure-core PTY pipeline constants (S-039 adversarial Pass-4, F-PASS4-MED-001).
///
/// Bounds for `pending_pty_bytes` (byte cap + message cap) and the dump-window
/// timeout. All values are pure arithmetic — no I/O.
pub mod pty_constants;
pub use pty_constants::{DUMP_WINDOW_TIMEOUT, MAX_PENDING_PTY_BYTES, MAX_PENDING_PTY_MESSAGES};

// Re-export at crate root per BC-2.02.002 postcondition 2 (S-010 canonical owner;
// S-003 references via `monocle_core::MONOCLE_ABI_VERSION`).
pub use abi::MONOCLE_ABI_VERSION;
