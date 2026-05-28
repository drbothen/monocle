//! IPC message types for monocle daemon-TUI communication.
//!
//! Defines `ServerToClient`, `ClientToServer`, `PermissionPromptPayload`, and the
//! `truncate_to_utf8_boundary` helper used for `HookEventReceived.payload_excerpt`.
//!
//! # Type Reuse
//!
//! [`monocle_core::hook_events::HookType`] is re-exported from `monocle-core` rather than
//! redefined here — the same discriminant enum serves both the hook-ingestion path and the
//! IPC wire format. Callers import `monocle_ipc::types::HookType`.
//!
//! [`monocle_core::engine::EnrichedSession`] is used directly in `SessionListUpdate` and
//! `InitialState`. The struct carries `#[non_exhaustive]` per BC-2.02.003.

use monocle_core::engine::EnrichedSession;
use monocle_core::hook_events::HookEvent;
// HookType is already non_exhaustive + serde in monocle-core; re-export for downstream consumers.
pub use monocle_core::hook_events::HookType;
use uuid::Uuid;

/// Messages sent from the daemon to TUI clients.
///
/// # Serialization
///
/// Each variant is serialized to JSON and framed with the 4-byte LE length-prefix
/// protocol (see [`crate::framing`]).  Variants are tagged with `#[serde(tag = "type")]`
/// for self-describing wire messages.
///
/// # Forward Compatibility
///
/// `#[non_exhaustive]` is intentionally NOT applied here — the daemon controls this enum
/// and adding new variants is a controlled operation (requires BC revision). TUI clients
/// compiled against an older version will encounter deserialization errors for unknown
/// variants, which is the correct Phase 1 behavior (TUI is updated alongside the daemon).
/// Phase 4 may introduce `#[non_exhaustive]` once cross-version compatibility is scoped.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ServerToClient {
    /// Full initial state push sent immediately after a TUI client connects.
    ///
    /// Fields per BC-2.05.002 (InitialState push on connect):
    /// - `sessions`: complete current session roster
    /// - `ring_tail`: last N hook events from the JSONL ring
    /// - `overlay_stack`: current permission prompt queue
    /// - `drop_counter`: cumulative event drops since daemon start
    InitialState {
        /// Complete current session roster at time of connection.
        sessions: Vec<EnrichedSession>,
        /// Last N hook events from the JSONL ring (event ribbon backfill).
        ring_tail: Vec<HookEvent>,
        /// Current permission prompt overlay stack.
        overlay_stack: Vec<PermissionPromptPayload>,
        /// Cumulative event drop count since daemon start.
        drop_counter: u64,
    },

    /// Incremental session roster update.
    ///
    /// Sent whenever the session roster changes (session added, removed, or enriched).
    /// The `sessions` Vec contains the **complete current list** — not a diff.
    /// TUI clients replace their entire local session roster on receipt (BC-2.05.003 PC-2).
    SessionListUpdate {
        /// The complete current session roster.
        sessions: Vec<EnrichedSession>,
    },

    /// A hook event was received by the daemon and passed through the event bus.
    ///
    /// Sent for each of the 5 hook endpoints: PreToolUse, Notification, Stop,
    /// SessionStart, UserPromptSubmit (BC-2.05.004 PC-1).
    HookEventReceived {
        /// The discriminant of the received hook endpoint.
        hook_type: HookType,
        /// Session identifier from the hook POST body.
        session_id: String,
        /// First 256 bytes of the hook POST body JSON, truncated at a valid UTF-8
        /// character boundary. Never exceeds 256 bytes. May be empty for zero-byte bodies.
        payload_excerpt: String,
        /// Wall-clock milliseconds from HTTP POST receipt to HTTP ACK sent to the caller.
        latency_ms: u64,
    },

    /// A new permission prompt has been queued and is awaiting a TUI decision.
    ///
    /// The TUI renders this as an overlay requiring user interaction.
    PermissionPromptQueued {
        /// Full payload for the queued permission prompt.
        payload: PermissionPromptPayload,
    },

    /// A permission prompt has been resolved (approved or denied).
    ///
    /// The TUI removes the matching overlay from its stack.
    PermissionPromptResolved {
        /// The unique identifier of the resolved prompt.
        prompt_id: Uuid,
    },

    /// The daemon's event drop counter has been updated.
    ///
    /// Sent when the bounded event bus drops an event (bus full). The TUI updates
    /// its status bar drop-counter display. This message is sent INSTEAD of
    /// `HookEventReceived` for dropped events (BC-2.05.004 PC-4 / BC-2.04.011).
    DropCounterUpdate {
        /// Cumulative event drops since daemon start.
        drop_counter: u64,
    },
}

/// Messages sent from TUI clients to the daemon.
///
/// Phase 1 has a single client-to-server message: `PermissionDecision`.
/// The enum is left open for Phase 2+ additions (e.g., explicit disconnect, ping).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ClientToServer {
    /// The user has made a permission decision for a queued prompt.
    ///
    /// The daemon routes this decision to the `ClaudeCodeModule::on_hook` path.
    PermissionDecision {
        /// The unique identifier of the prompt being decided.
        prompt_id: Uuid,
        /// The user's decision: approve or deny.
        decision: PermissionDecisionKind,
    },
}

/// The kind of permission decision the user made.
///
/// # Naming
///
/// This enum is named `PermissionDecisionKind` rather than `PermissionDecision` to avoid
/// a name collision with the `ClientToServer::PermissionDecision` variant. Using the same
/// name for both the enum and the variant it appears in would require fully-qualified syntax
/// at every use site and create confusion when reading IPC message construction code.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PermissionDecisionKind {
    /// The user approved the tool invocation.
    Allow,
    /// The user denied the tool invocation.
    Deny,
}

/// Payload for a permission prompt overlay.
///
/// Sent in `ServerToClient::PermissionPromptQueued` and stored in
/// `ServerToClient::InitialState::overlay_stack`.
///
/// The `prompt_id` field is always assigned by the registry, never by the caller.
/// Callers build a `PermissionPromptPayload` through
/// [`PendingDecisionRegistry::register_prompt`] which accepts a
/// [`PromptPayloadInputs`] (no `prompt_id`) and returns `(prompt_id, payload)`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionPromptPayload {
    /// Unique identifier for this prompt instance.
    ///
    /// Assigned by [`crate::permissions::PendingDecisionRegistry::register_prompt`];
    /// callers MUST NOT supply this value.
    pub prompt_id: Uuid,
    /// Session that raised this permission prompt.
    pub session_id: String,
    /// The tool being invoked (e.g., `"Bash"`, `"Edit"`).
    pub tool_name: String,
    /// JSON-encoded tool input arguments.
    pub tool_input: serde_json::Value,
    /// Original file content (for Edit/Write tools). `None` for non-file tools.
    pub old_content: Option<String>,
    /// Proposed new file content (for Edit/Write tools). `None` for non-file tools.
    pub new_content: Option<String>,
}

/// Caller-supplied fields for registering a new permission prompt (F-ADV2-MED-001).
///
/// Separates the caller-controlled inputs from the registry-assigned `prompt_id`.
/// Pass this to [`PendingDecisionRegistry::register_prompt`]; the registry generates
/// and assigns the `prompt_id`, returning a complete [`PermissionPromptPayload`].
///
/// # Why a separate type?
///
/// The previous API accepted a `PermissionPromptPayload` whose `prompt_id` field was
/// silently overwritten by `register_prompt`. This was an API leak: callers had to know
/// that the field they set would be discarded. `PromptPayloadInputs` makes the contract
/// explicit — there is no `prompt_id` to set.
#[derive(Debug, Clone)]
pub struct PromptPayloadInputs {
    /// Session that raised this permission prompt.
    pub session_id: String,
    /// The tool being invoked (e.g., `"Bash"`, `"Edit"`).
    pub tool_name: String,
    /// JSON-encoded tool input arguments.
    pub tool_input: serde_json::Value,
    /// Original file content (for Edit/Write tools). `None` for non-file tools.
    pub old_content: Option<String>,
    /// Proposed new file content (for Edit/Write tools). `None` for non-file tools.
    pub new_content: Option<String>,
}

/// The maximum payload excerpt size in bytes (BC-2.05.004 PC-1, invariant 1).
///
/// `payload_excerpt` in `HookEventReceived` is truncated to this many bytes,
/// at a valid UTF-8 character boundary.
pub const PAYLOAD_EXCERPT_MAX_BYTES: usize = 256;

/// Truncate a UTF-8 string to at most `max_bytes` bytes, snapping back to the last
/// complete character boundary.
///
/// # Contract (BC-2.05.004 PC-1, invariant 1)
///
/// - The returned slice is always valid UTF-8.
/// - The returned slice is always `<= max_bytes` bytes.
/// - If `s.len() <= max_bytes`, the full string is returned unchanged.
/// - Truncation snaps to the last character boundary before or at `max_bytes`,
///   never splitting a multi-byte UTF-8 sequence.
///
/// # Examples
///
/// ```rust
/// use monocle_ipc::types::truncate_to_utf8_boundary;
///
/// // Short string: returned unchanged.
/// assert_eq!(truncate_to_utf8_boundary("hello", 10), "hello");
///
/// // ASCII at boundary: exactly 5 bytes.
/// assert_eq!(truncate_to_utf8_boundary("hello world", 5), "hello");
///
/// // Multi-byte char: boundary snaps back before the split sequence.
/// let s = "aé"; // "a" (1 byte) + "é" (2 bytes) = 3 bytes total.
/// assert_eq!(truncate_to_utf8_boundary(s, 2), "a");
/// ```
pub fn truncate_to_utf8_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    // Walk backwards from max_bytes to find the last valid UTF-8 char boundary.
    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &s[..boundary]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_empty() {
        assert_eq!(truncate_to_utf8_boundary("", 256), "");
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate_to_utf8_boundary("hello", 256), "hello");
    }

    #[test]
    fn truncate_ascii_at_boundary() {
        assert_eq!(truncate_to_utf8_boundary("hello world", 5), "hello");
    }

    #[test]
    fn truncate_multibyte_snaps_back() {
        // "é" is U+00E9, encoded as 0xC3 0xA9 (2 bytes).
        // "aé" = 3 bytes total; truncating to 2 must return "a" (1 byte),
        // not the invalid partial "é".
        let s = "a\u{00e9}"; // "aé"
        assert_eq!(truncate_to_utf8_boundary(s, 2), "a");
    }

    #[test]
    fn truncate_three_byte_char_snaps_back() {
        // "€" is U+20AC, encoded as 0xE2 0x82 0xAC (3 bytes).
        // Truncate to 2 bytes: must snap back to "" (before the €).
        let s = "\u{20ac}"; // "€"
        assert_eq!(truncate_to_utf8_boundary(s, 2), "");
    }

    #[test]
    fn truncate_exact_boundary_returned() {
        // Truncate to exact length: full string returned.
        let s = "hello";
        assert_eq!(truncate_to_utf8_boundary(s, 5), "hello");
    }

    #[test]
    fn truncate_result_always_valid_utf8() {
        // Construct a string with multi-byte chars, truncate at various points,
        // verify all results are valid UTF-8.
        let s = "abc\u{00e9}def\u{20ac}ghi"; // mix of ASCII and multi-byte
        for max in 0..=s.len() + 1 {
            let result = truncate_to_utf8_boundary(s, max);
            assert!(
                std::str::from_utf8(result.as_bytes()).is_ok(),
                "truncate_to_utf8_boundary({s:?}, {max}) produced invalid UTF-8"
            );
        }
    }
}
