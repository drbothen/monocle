// VP-013 Test Vector 13.b: synthetic failure fixture.
//
// This file is NEVER compiled as part of the monocle-core test suite.
// It is parsed directly by `enum_audit.rs` via `syn 2` to exercise the
// failure-detection path: the AST audit must identify that `BadEnum` lacks
// `#[non_exhaustive]` and is not in the ADR-0004 EXEMPT list.
//
// The `#![cfg(test)]` guard below documents intent but is irrelevant:
// the file is not compiled via Cargo; only parsed as raw source text
// by the AST audit harness.
#![cfg(test)]

/// A synthetic `pub enum` that intentionally lacks `#[non_exhaustive]`.
///
/// This type does NOT exist in production monocle-core source. It exists
/// solely to give the AST audit harness a concrete non-exhaustive-policy
/// violation to detect. See VP-013 Probe 13.b and BC-2.02.003 §Canonical
/// Test Vectors row 1.
pub enum BadEnum {
    /// First variant (content irrelevant; the attribute absence is the signal).
    VariantA,
    /// Second variant.
    VariantB,
}
