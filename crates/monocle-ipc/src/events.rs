//! Transport-level events for the TUI-to-daemon IPC connection (S-023, BC-2.05.007).
//!
//! [`TransportEvent`] is emitted by [`crate::uds::UdsClientTransport`] on the TUI side when
//! a connection-loss error is detected on the read path. The TUI event loop receives this
//! signal and executes the SOQ-3 overlay-clear handler before the reconnect loop begins.
//!
//! # SOQ-3 Ordering Invariant (BC-2.05.007 Invariant 1)
//!
//! `TransportEvent::Disconnected` is always emitted BEFORE the error is returned to the
//! caller and BEFORE any reconnect attempt. This ordering is enforced at the transport layer
//! (not in the TUI event handler) so it cannot be bypassed by TUI code changes.

/// Events emitted by the IPC transport layer to the TUI event loop (S-023).
///
/// # Phase 1 — Single Variant
///
/// Phase 1 defines exactly one event variant: `Disconnected`. Future phases may add
/// variants such as `Reconnecting`, `Reconnected`, or transport-level diagnostic events.
/// The `#[non_exhaustive]` attribute ensures that TUI match arms must include a wildcard,
/// providing forward compatibility without requiring a BC revision when new variants land.
///
/// # SOQ-3 (BC-2.05.007)
///
/// The TUI event loop reacts to `Disconnected` by synchronously clearing the
/// `VecDeque<PromptModal>` overlay stack before entering the reconnect loop.
/// This prevents ghost approvals for stale prompts (SOQ-3 safety invariant).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportEvent {
    /// The IPC connection to the daemon was lost unexpectedly.
    ///
    /// Emitted by [`crate::uds::UdsClientTransport`] when `read_framed` returns one of:
    /// - `UnexpectedEof`
    /// - `BrokenPipe`
    /// - `ConnectionReset`
    ///
    /// NOT emitted on TUI-initiated graceful disconnect (TUI process exits normally).
    ///
    /// The TUI event loop MUST clear the `VecDeque<PromptModal>` overlay stack upon
    /// receiving this event, before beginning the reconnect loop (SOQ-3, BC-2.05.007).
    Disconnected,
}
