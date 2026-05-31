//! Tests for BC-2.06.016 (disconnect clears overlay; reconnect restores via InitialState).
//!
//! # Red Gate
//!
//! All tests in this file are expected to FAIL against the S-026 stubs.
//! The test-writer fills in assertions; the implementer makes them green.
//!
//! # Coverage
//!
//! - Disconnect clears: `TransportEvent::Disconnected` → `overlay_stack.clear()`,
//!   `AppMode` transitions to `Dashboard { focused: Sessions }` (BC-2.06.016 PC-1, SOQ-3).
//! - Status message set to `DAEMON_DISCONNECT_STATUS` on disconnect.
//! - Reconnect restore: `ServerToClient::InitialState { overlay_stack: [P1, P2] }` re-populates
//!   `App::overlay_stack` and transitions to `AppMode::Overlay` if non-empty (BC-2.06.016 PC-2).
//! - Snapshot-window dedup: `InitialState` with prompt X, then streaming `PermissionPromptQueued`
//!   for X → VecDeque contains X exactly once (BC-2.05.002 Invariant 4, `test_snapshot_window_prompt_dedup`).

// No tests yet — test-writer fills these in per S-026 §Tasks.
