//! Tests for BC-2.06.011 (Accept-Once), BC-2.06.012 (Accept-Always), BC-2.06.013 (Reject),
//! and the wait-for-PermissionPromptResolved round-trip (modal stays in stack after decision send).
//!
//! # Red Gate
//!
//! All tests in this file are expected to FAIL against the S-026 stubs.
//! The test-writer fills in assertions; the implementer makes them green.
//!
//! # Coverage
//!
//! - Accept-Once (`y`/`Enter`): `ClientToServer::PermissionDecision { decision: Allow }` enqueued.
//! - Accept-Always (`A`): `ClientToServer::PermissionDecision { decision: AcceptAlways }` enqueued.
//! - Reject (`n`/`r`): `ClientToServer::PermissionDecision { decision: Deny }` enqueued.
//! - Modal NOT immediately popped after decision send (BC-2.06.011 Invariant 5 / BC-2.06.023).
//! - Modal removed only after `PermissionPromptResolved` arrives.
//! - IPC channel full: message dropped, drop counter increments, modal stays visible (EC-079).
//! - Esc in Overlay: identity no-op, no IPC send, mode unchanged (BC-2.06.014 PC-1).

// No tests yet — test-writer fills these in per S-026 §Tasks.
