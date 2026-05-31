//! monocle-test-harness — DTU clone library for Claude Code hook protocol testing.
//!
//! This crate provides the behavioral L3 clone (`dtu-claude-code-hooks-v1`) of the
//! Claude Code 5-endpoint hook protocol. It synthesizes hook POSTs for integration
//! tests, holdout evaluation, and formal hardening without requiring a live Claude
//! Code instance.
//!
//! Source authority:
//! - S-DTU-001 v1.2 (story spec)
//! - dtu-assessment.md §Packaging Decision (implemented against v1.7.5 at S-DTU-001 authoring time)
//! - BC-HOOK-001..BC-HOOK-041

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod dtu;
