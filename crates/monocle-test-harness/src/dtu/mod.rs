//! DTU clone for the Claude Code hook protocol.
//!
//! Implements a behavioral L3 clone of Claude Code's 5-endpoint hook protocol
//! at fidelity ≥0.95 against the 25-fixture corpus at
//! `tests/fixtures/dtu/claude-code-hook-2x/`.
//!
//! Source authority:
//! - dtu-assessment.md v1.7.5 §Clone Development Approach §Packaging Decision
//! - SS-core-types-and-abi.md v1.2.13 §Non-Exhaustive Inner Structs
//! - SS-daemon-lifecycle.md v1.0.33 §Start Sequence (lock file schema)
//! - BC-HOOK-001..BC-HOOK-041

#![forbid(unsafe_code)]

pub mod endpoints;
pub mod handlers;
pub mod lock_reader;
pub mod payload;
pub mod server;
