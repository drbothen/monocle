//! Tests for BC-2.06.023 (UUID removal via retain(), duplicate removal, empty-stack collapse).
//!
//! # Red Gate
//!
//! All tests in this file are expected to FAIL against the S-026 stubs.
//! The test-writer fills in assertions; the implementer makes them green.
//!
//! # Coverage
//!
//! - `retain()` removes the matching modal regardless of stack position (not just front).
//! - `retain()` removes ALL entries with matching `prompt_id` (duplicate-from-reconnect race).
//! - Unknown `prompt_id` in `PermissionPromptResolved`: WARN logged, no-op (BC-2.06.023 PC-3).
//! - Empty-stack collapse: after `retain()` removes the last modal, `AppMode` transitions to
//!   `Dashboard { focused: prior }` (BC-2.06.023 PC-4, AC-015).
//! - Removal is NOT routed through `transition()` — direct mutation + post-retain collapse check
//!   (Architecture Compliance Rule: §Forbidden Dependencies / BC-2.06.023).

// No tests yet — test-writer fills these in per S-026 §Tasks.
