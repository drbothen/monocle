//! VP-015 integration tests for `VsddFactoryAdapter` (BC-2.02.005, S-012).
//!
//! Self-referential test: runs `VsddFactoryAdapter::detect` against the monocle repo's
//! own `.factory/STATE.md` to verify end-to-end detection + state parsing.
//!
//! Canonical test-function name pinned at BC-2.02.005 line 93 (verbatim).

// Placeholder — test bodies will be filled in by the test-writer agent.

#[test]
#[allow(clippy::todo)]
#[allow(non_snake_case)]
fn test_BC_FACTORY_002_vsdd_adapter_self_referential_detection() {
    // VP-015: detect monocle repo root → Some(FactoryDetection), read_state → Ok(FactoryState),
    // subscribe → empty stream, display_name → "VSDD Factory".
    todo!("S-012 test-writer: VP-015 self-referential integration test")
}
