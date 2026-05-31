//! Tests for BC-2.06.009 (stack rotation via Up/Down → Action::OverlayCycleNext).
//!
//! # Red Gate
//!
//! All tests in this file are expected to FAIL against the S-026 stubs.
//! The test-writer fills in assertions; the implementer makes them green.
//!
//! # Coverage
//!
//! - Rotation with `stack.len() > 1`: `pop_front()` + `push_back()` moves front to back.
//! - Single-item no-op: `stack.len() == 1`, rotation returns same item to front (EC-065).
//! - Mode stays `Overlay { prior }` after rotation — `transition()` is identity for OverlayCycleNext.
//! - `overlay_stack.front()` changes after rotation (new prompt rendered).

// No tests yet — test-writer fills these in per S-026 §Tasks.
