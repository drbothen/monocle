//! Compile-time ABI stability test (BC-2.02.002 PC-3, VP-012).
//!
//! This file-scope assertion fails at compile time if MONOCLE_ABI_VERSION changes
//! without updating the assertion. Any change requires an ADR.

const _: () = assert!(
    monocle_core::MONOCLE_ABI_VERSION == 1,
    "ABI version mismatch: MONOCLE_ABI_VERSION changed without updating this assertion. An ADR is required for ABI version changes."
);

#[test]
fn abi_version_is_1() {
    assert_eq!(monocle_core::MONOCLE_ABI_VERSION, 1);
}
