//! Tests for BC-2.06.008 (overlay push/FIFO), BC-2.06.023 PC-4 (empty-stack collapse),
//! BC-2.06.024 (payload_to_modal() conversion), and BC-2.05.002 Invariant 4 (idempotent insert).
//!
//! # Red Gate
//!
//! All tests in this file are expected to FAIL against the S-026 stubs.
//! The test-writer fills in assertions; the implementer makes them green.
//!
//! # Coverage
//!
//! - FIFO ordering: oldest prompt is `overlay_stack.front()`, new prompts append via `push_back`.
//! - Empty-stack collapse: after the last modal is removed, `AppMode` transitions to Dashboard.
//! - Idempotent insert: duplicate `prompt_id` is silently discarded (BC-2.05.002 Invariant 4).
//! - `payload_to_modal()` conversion: all four `ToolPayload` variants (Edit, Bash, Read, Generic).
//! - `payload_to_modal()` fallback: missing `"command"` key → `ToolPayload::Generic`.
//! - `payload_to_modal()` fallback: missing `"path"` key for Read → `ToolPayload::Generic`.

// No tests yet — test-writer fills these in per S-026 §Tasks.
