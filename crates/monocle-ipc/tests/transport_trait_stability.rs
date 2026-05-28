//! Transport trait compile-time stability tests for `monocle-ipc` (BC-2.05.008).
// Test function names follow the VSDD `test_BC_S_SS_NNN_xxx` convention (uppercase BC).
// Suppress non_snake_case lint for test files — BC_ prefix is intentional for traceability.
#![allow(non_snake_case)]
//!
//! These tests are compile-time assertions — they do not execute logic at runtime.
//! The test PASSES by compiling successfully. If `UdsClientTransport` does not implement
//! `Transport`, the crate fails to compile.
//!
//! BC-2.05.008 PC-5/PC-6 / AC-015.

use monocle_ipc::transport::Transport;
use monocle_ipc::uds::UdsClientTransport;

/// test_BC_2_05_008_uds_client_transport_implements_transport_trait:
/// `UdsClientTransport` implements the `Transport` trait (compile-time assertion).
///
/// If this function does not compile, `UdsClientTransport` is missing the trait impl.
///
/// BC-2.05.008 PC-5 — `UdsTransport` (and its per-client handle) is the sole Phase 1
/// `Transport` implementor in `monocle-ipc`.
#[allow(dead_code)]
fn test_BC_2_05_008_uds_client_transport_implements_transport_trait() {
    fn assert_impl<T: Transport>() {}
    assert_impl::<UdsClientTransport>();
}

/// test_BC_2_05_008_transport_trait_is_send_sync_static:
/// The `Transport` trait bound includes `Send + Sync + 'static`, required for use in
/// Tokio tasks and `Arc<dyn Transport>` patterns.
///
/// BC-2.05.008 PC-6.
#[allow(dead_code)]
fn test_BC_2_05_008_transport_trait_is_send_sync_static() {
    fn assert_send_sync_static<T: Transport + Send + Sync + 'static>() {}
    assert_send_sync_static::<UdsClientTransport>();
}

/// test_BC_2_05_008_transport_dyn_compatible:
/// `Transport` can be used as `dyn Transport` (object-safety check).
///
/// This verifies that async_trait's desugaring produces an object-safe trait.
/// BC-2.05.008 PC-6 (Phase 4 `ShmTransport` will require `dyn Transport`).
#[allow(dead_code)]
fn test_BC_2_05_008_transport_dyn_compatible() {
    fn _accepts_dyn(_t: &dyn Transport) {}
}
