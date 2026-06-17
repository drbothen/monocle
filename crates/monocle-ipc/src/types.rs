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
//!
//! # HookEventRecord (relocated from monocle-runtime)
//!
//! [`HookEventRecord`] is defined here because it is the canonical IPC + JSONL ring transport
//! format (architect decision F-S022-ADV2-HIGH-002, ADR-0006). It belongs in `monocle-ipc`
//! as the wire type that crosses the daemon-TUI boundary. `monocle-runtime::ring` re-exports
//! it from here, breaking the potential circular dependency that would arise from
//! `monocle-ipc` importing from `monocle-runtime` (which already depends on `monocle-ipc`).

use monocle_core::engine::{EnrichedSession, SpawnOptions};
// HookType is already non_exhaustive + serde in monocle-core; re-export for downstream consumers.
pub use monocle_core::hook_events::HookType;
use uuid::Uuid;

/// Ring format version constant — FC-01 forward-compatibility contract.
///
/// Relocated from `monocle-runtime::ring` to `monocle-ipc::types` per architect decision
/// F-S022-ADV2-HIGH-002 (ADR-0006): `HookEventRecord` is the canonical IPC + JSONL ring
/// transport format and belongs in `monocle-ipc`.
pub const RING_FORMAT_VERSION: u32 = 1;

/// A single hook event record written to the JSONL ring buffer and transported over IPC.
///
/// This is the canonical IPC + JSONL ring transport format (architect decision
/// F-S022-ADV2-HIGH-002, ADR-0006, BC-2.04.012 PC-1). The RAM ring in `monocle-runtime`
/// stores `HookEventRecord` directly; `InitialState.ring_tail` uses `Vec<HookEventRecord>`
/// so that no lossy reconstruction from ring storage to a richer type is required.
///
/// # Relocation note
///
/// Previously defined in `monocle-runtime::ring`. Moved here to break the circular
/// dependency that would arise from `monocle-ipc` importing from `monocle-runtime`
/// (which already depends on `monocle-ipc`). `monocle-runtime::ring` re-exports
/// `HookEventRecord` from this location for backward compatibility.
///
/// Fields are in canonical declaration order per SS-core-types-and-abi.md §HookEventRecord.
/// `format_version` MUST serialize as the first JSON key (struct field order preservation).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct HookEventRecord {
    /// FC-01 forward-compatibility version stamp; always set to [`RING_FORMAT_VERSION`].
    pub format_version: u32,
    /// Opaque session identifier (UUID string).
    pub session_id: String,
    /// Unix epoch timestamp in microseconds (signed per SS-core-types-and-abi.md §HookEventRecord).
    pub timestamp_micros: i64,
    /// Process ID of the originating harness process.
    pub pid: u32,
    /// Hook type discriminant (e.g. `"PreToolUse"`, `"SessionStart"`).
    pub hook_type: String,
    /// Tool name present only for tool-context hook types (e.g. `"PreToolUse"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool input JSON present only for tool-context hook types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
}

impl HookEventRecord {
    /// Construct a new record. `format_version` is set to [`RING_FORMAT_VERSION`] internally.
    ///
    /// External callers MUST use this constructor — `#[non_exhaustive]` forbids struct-literal
    /// construction outside `monocle-ipc::types` (BC-2.01.007 PC-5, Rust E0639).
    pub fn new(
        session_id: String,
        timestamp_micros: i64,
        pid: u32,
        hook_type: String,
        tool_name: Option<String>,
        tool_input: Option<serde_json::Value>,
    ) -> Self {
        Self {
            format_version: RING_FORMAT_VERSION,
            session_id,
            timestamp_micros,
            pid,
            hook_type,
            tool_name,
            tool_input,
        }
    }
}

// ---------------------------------------------------------------------------
// S-033: SessionState (wire type — authoritative location per architecture compliance rule)
// SessionState lives in monocle-ipc, NOT monocle-runtime, because SessionStateChanged
// { new_state: SessionState } and SessionSnapshot { state: SessionState } are wire types.
// monocle-ipc MUST NOT depend on monocle-runtime; placing SessionState in monocle-runtime
// would create a circular dependency.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// S-033: SessionSidecarV3 — shared session-state.json schema (Ruling B)
// Lives in monocle-ipc so both monocle-runtime and monocle-session-host import the
// SAME type. This is the byte-level schema agreement mechanism: any change to the
// struct will produce a compile-time error in both crates simultaneously (HIGH-009).
// ---------------------------------------------------------------------------

/// Session-state.json sidecar, schema version 3.
///
/// Shared between `monocle-runtime` (daemon writer at spawn — `child_pid: None`) and
/// `monocle-session-host` (writer at startup step 8 — `child_pid: Some(pid)`).
/// Both crates import this from `monocle-ipc`. The serde schema is the byte-level
/// agreement; compile errors propagate to both crates on struct change.
///
/// **Ownership protocol (SS-session-manager.md §Ruling B):**
/// - Daemon writes all fields first with `child_pid: None`.
/// - Session-host overwrites the sidecar with `child_pid: Some(pid)` at startup step 8,
///   after the PTY is open and the harness child is spawned.
/// - Both writes are atomic via `tempfile::persist`.
///
/// **Forward compat:** `#[serde(default)]` on `kill_deadline_unix_ms` allows schema v1/v2
/// sidecars to deserialize into this struct with the field absent → `None`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionSidecarV3 {
    /// Always 3 for this struct version.
    pub schema_version: u32,
    /// Session UUID string.
    pub session_id: String,
    /// OS PID of the session-host process (daemon's initial write).
    pub pid: u32,
    /// Per-session UDS socket path: `<runtime_dir>/session-<uuid>.sock`.
    pub socket_path: String,
    /// Harness child PID. `None` in daemon's initial write; populated by session-host at step 8.
    pub child_pid: Option<u32>,
    /// Lifecycle state. `Launching` in daemon's initial write; updated on transitions.
    pub state: SessionState,
    /// User-selected project directory (wizard Step 2).
    pub project_root: String,
    /// Resolved worktree root (equals `project_root` when no worktree configured).
    pub cwd: String,
    /// Harness identifier (e.g., `"claude-code"`).
    pub harness_id: String,
    /// Profile identifier.
    pub profile_id: String,
    /// ISO-8601 UTC spawn timestamp (e.g., `chrono::Utc::now().to_rfc3339()`).
    pub started_at: String,
    /// Human-readable session display name.
    pub display_name: String,
    /// Initial PTY rows.
    pub pty_rows: u16,
    /// Initial PTY cols.
    pub pty_cols: u16,
    /// Kill deadline as Unix epoch milliseconds.
    /// `null` unless `state == Terminating`. Forward-compat: absent in v1/v2 sidecars → `None`.
    #[serde(default)]
    pub kill_deadline_unix_ms: Option<u64>,
}

/// Session lifecycle state machine.
///
/// The canonical 5 variants. `Created` and `Killed` are RETIRED — do NOT use them.
///
/// `#[non_exhaustive]` per SS-session-manager.md §Session lifecycle state machine.
/// Serialize/Deserialize for session-state.json sidecar and IPC wire transport.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionState {
    /// session-host process spawned; waiting for `StateChanged { new_state: Running }`.
    /// Initial state written to sidecar at spawn time.
    Launching,
    /// session-host alive, daemon attached, PTY streaming to broker.
    Running,
    /// TUI or daemon explicitly detached; session-host still alive.
    Detached,
    /// kill_session() called; awaiting HostToDaemon::StateChanged { Terminated }.
    Terminating,
    /// Harness child exited. Terminal state.
    Terminated,
}

/// Messages sent from the session-host process to the daemon over the per-session UDS socket.
///
/// Serialized as length-prefixed JSON (4-byte LE u32 + JSON body) on the per-session
/// UDS control connection (`<runtime_dir>/session-<uuid>.sock`).
///
/// `#[serde(tag = "type")]` uses the variant name lowercased (e.g., `state_changed`)
/// per SS-session-manager.md §session-host to daemon protocol.
///
/// S-033: minimum viable set for the Launching → Running transition.
/// S-034/S-035: Kill/Detach result variants are added in those stories.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostToDaemon {
    /// Session-host reports a lifecycle state transition.
    ///
    /// Sent at startup (Launching → Running) and on termination (→ Terminated).
    StateChanged {
        /// The new lifecycle state.
        new_state: SessionState,
        /// Optional degraded-env flag. `Some(true)` when HOME or PATH were not set.
        /// `None` in normal operation.
        degraded_env: Option<bool>,
    },
}

/// A snapshot of a single session (wire type for SessionListUpdate, InitialState).
///
/// Carries the daemon-observable session metadata for display in the TUI sessions panel.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionSnapshot {
    /// Session UUID string.
    pub session_id: String,
    /// Harness identifier (e.g., `"claude-code"`).
    pub harness_id: String,
    /// Profile identifier.
    pub profile_id: String,
    /// Human-readable session name. Defaults to `"<harness_id> — <project_root_basename>"`.
    pub display_name: String,
    /// Project root directory.
    pub project_root: String,
    /// Working directory (resolved worktree root or project_root).
    pub cwd: String,
    /// Current lifecycle state.
    pub state: SessionState,
    /// ISO-8601 UTC spawn timestamp.
    pub started_at: String,
    /// True when the session-host detected missing critical env vars (HOME, PATH).
    pub degraded: bool,
    /// Human-readable degraded reason, if degraded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
}

impl SessionSnapshot {
    /// Construct a `SessionSnapshot`.
    ///
    /// Required because `SessionSnapshot` is `#[non_exhaustive]` — struct-literal
    /// construction is forbidden outside `monocle-ipc::types` (Rust E0639).
    /// Called from `monocle-runtime::session_manager` to build `session_list()`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: String,
        harness_id: String,
        profile_id: String,
        display_name: String,
        project_root: String,
        cwd: String,
        state: SessionState,
        started_at: String,
        degraded: bool,
        degraded_reason: Option<String>,
    ) -> Self {
        Self {
            session_id,
            harness_id,
            profile_id,
            display_name,
            project_root,
            cwd,
            state,
            started_at,
            degraded,
            degraded_reason,
        }
    }
}

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
    /// Fields per BC-2.05.002 §Postconditions (InitialState push on connect):
    /// - `sessions`: complete current session roster
    /// - `ring_tail`: last N events from the RAM ring as `Vec<HookEventRecord>`
    /// - `overlay_stack`: current permission prompt queue
    /// - `drop_counter`: cumulative event drops since daemon start
    ///
    /// # ring_tail type (architect decision F-S022-ADV2-HIGH-002, BC-2.05.002 PC-2)
    ///
    /// `ring_tail` is `Vec<HookEventRecord>` — the native RAM ring storage type.
    /// Using the ring's storage type directly avoids lossy reconstruction: the RAM ring
    /// stores `HookEventRecord`; reconstructing `HookEvent` variants from records requires
    /// fabricating absent fields (cwd, transcript_path, prompt, stop_reason) with empty-string
    /// defaults, producing silently incorrect data (previous attempt; superseded by
    /// ADR-0006 / BC-2.05.002 §Postconditions PC-2). The TUI (S-025) renders the event ribbon
    /// from `HookEventRecord` fields (`hook_type`, `session_id`, `timestamp_micros`,
    /// `tool_name`) which are sufficient for display.
    InitialState {
        /// Complete current session roster at time of connection.
        sessions: Vec<EnrichedSession>,
        /// Last N events from the RAM ring as native ring storage type (BC-2.05.002 PC-2,
        /// BC-2.04.012 PC-1). No lossy reconstruction — the TUI renders from
        /// `hook_type`, `session_id`, `timestamp_micros`, `tool_name` fields.
        ring_tail: Vec<HookEventRecord>,
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
    ///
    /// # timestamp_micros (SS-ipc / BC-2.05.004 PC-2)
    ///
    /// The daemon captures `SystemTime::now()` ONCE at hook POST receipt time and
    /// writes the same value to BOTH the ring entry (`HookEventRecord::timestamp_micros`)
    /// AND this IPC message. The TUI MUST use this value for the ribbon's Timestamp column
    /// rather than substituting `SystemTime::now()` at message-arrival time — which would
    /// produce a wall-clock skew equal to the IPC round-trip latency.
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
        /// Unix epoch timestamp in microseconds at the moment the daemon received the hook
        /// POST (per BC-2.05.004 PC-2 / SS-ipc). The daemon captures this ONCE and
        /// writes the same value to BOTH this message and the corresponding
        /// `HookEventRecord::timestamp_micros` in the ring (BC-2.05.004 PC-2 equality).
        ///
        /// The TUI MUST propagate this value to `HookEventRow::timestamp_micros` (not
        /// substitute `SystemTime::now()` at TUI receive time) to maintain deterministic
        /// wall-clock display in the Event Ribbon Timestamp column.
        timestamp_micros: i64,
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

    // -----------------------------------------------------------------------
    // S-033 variants (BC-2.08.001, BC-2.08.008)
    // -----------------------------------------------------------------------
    /// Immediate acknowledgement of a `ClientToServer::SpawnSession` request.
    ///
    /// Sent to the REQUESTING client ONLY (not broadcast) BEFORE `spawn_session()`
    /// is called, in the IPC handler. The TUI uses `session_id` to track the
    /// in-progress spawn (F-P41-IMP-001, BC-2.08.001 PC-1).
    SpawnAck {
        /// The UUID v4 string generated by the daemon IPC handler for this spawn.
        session_id: String,
    },

    /// A session's lifecycle state changed.
    ///
    /// Published to ALL TUI clients via the broker. MUST be sent BEFORE
    /// `SessionListUpdate` for the same transition (BC-2.08.008 Invariant 4).
    SessionStateChanged {
        /// The session whose state changed.
        session_id: String,
        /// The new lifecycle state.
        new_state: SessionState,
    },

    /// An IPC operation failed.
    ///
    /// Sent to the requesting client when a lifecycle operation returns a
    /// `SessionError`. The `code` field maps to the v1A IPC error code taxonomy
    /// in SS-session-manager.md §session_error_to_code().
    Error {
        /// Machine-readable error code (e.g., `"binary_not_found"`, `"spawn_failed"`).
        code: String,
        /// Human-readable error message (from `SessionError::to_string()`).
        message: String,
    },
}

/// Messages sent from TUI clients to the daemon.
///
/// Phase 1 has a single client-to-server message: `PermissionDecision`.
/// S-033 adds `SpawnSession` (BC-2.08.001 PC-1 / SS-session-manager.md §IPC handler pattern).
/// The enum is left open for Phase 2+ additions.
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

    // -----------------------------------------------------------------------
    // S-033 variant (BC-2.08.001 — spawn a new session)
    // -----------------------------------------------------------------------
    /// Request the daemon to spawn a new session.
    ///
    /// The TUI populates `project_root`, `worktree_root`, `harness_id`, `profile_id`,
    /// and `ccr_base_url` in `opts`. The daemon fills `session_id` and
    /// `hooks_settings_path` via `opts.with_daemon_fields()` on receipt, then calls
    /// `spawn_session(opts)` (BC-2.08.001 §IPC handler pattern).
    SpawnSession {
        /// Spawn parameters from the SessionCreation wizard.
        opts: SpawnOptions,
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
///
/// # Variant correspondence to BC-2.06.011/012/013
///
/// | BC variant name          | IPC enum variant | Keybinding | Semantics                          |
/// |--------------------------|------------------|------------|------------------------------------|
/// | `PermissionDecision::Accept`      | `Allow`          | `y`/Enter  | Allow this invocation once         |
/// | `PermissionDecision::AcceptAlways`| `AcceptAlways`   | `A`        | Allow + persist pattern (S-026)    |
/// | `PermissionDecision::Reject`      | `Deny`           | `n`/`r`    | Deny this invocation               |
///
/// `AcceptAlways` was added in S-026 (BC-2.06.012 PC-1). It instructs the daemon to both
/// unblock this invocation AND record a persistent allow-pattern for future auto-accept.
/// The TUI sends this variant verbatim; the daemon is solely responsible for pattern recording.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PermissionDecisionKind {
    /// The user approved the tool invocation for this specific occurrence.
    Allow,
    /// The user approved the tool invocation and requested persistent auto-accept.
    ///
    /// Instructs the daemon to: (1) unblock this invocation and (2) record a
    /// tool+path allow-pattern so future identical invocations bypass the TUI
    /// overlay (BC-2.06.012 PC-1). Pattern recording is entirely daemon-side;
    /// the TUI does not maintain any pattern state.
    AcceptAlways,
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
