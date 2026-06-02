//! Hook endpoint handler modules for S-018.
//!
//! Each sub-module implements one or more of the 5 canonical hook endpoints
//! with full EngineModule dispatch, event bus publish, ring append, and
//! timeout budgets.
//!
//! # Behavioral contracts covered
//!
//! - BC-2.04.007: PreToolUse routing (300ms timeout, Defer support)
//! - BC-2.04.008: Notification routing (2000ms timeout, no Defer)
//! - BC-2.04.009: Stop/SessionStart/PromptSubmit routing (300ms timeout, no Defer)
//! - BC-2.04.011: Bounded event bus (try_send, drop counter)

pub mod notification;
pub mod pre_tool_use;
pub mod stop_session_prompt;

/// The JSON body posted by Claude Code hook scripts (all 5 hook types share this envelope).
///
/// Fields are a superset — each hook type uses a different subset. Unknown fields are
/// silently ignored (`deny_unknown_fields` is NOT set) for forward-compatibility.
///
/// BC-2.04.007 PC-1, BC-2.04.008 PC-1, BC-2.04.009 PC-1: deserialization failure →
/// HTTP 422 `{"error":"invalid_body","message":"<serde error>"}`.
///
/// Required fields (present in all 5 hook types):
/// - `session_id`: Claude Code's own session UUID
/// - `pid`: PID of the Claude Code subprocess
///
/// Optional / hook-type-specific fields:
/// - `tool_name`: present in PreToolUse, Notification (permission_prompt)
/// - `tool_input`: present in PreToolUse, Notification (permission_prompt)
/// - `stop_reason`: present in Stop
/// - `cwd`: present in SessionStart
/// - `transcript_path`: present in SessionStart
/// - `prompt`: present in UserPromptSubmit
/// - `notification_type`: present in Notification
/// - `message`: present in Notification
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HookEnvelope {
    /// Claude Code's own session UUID. Required in all hook types.
    pub session_id: String,
    /// PID of the Claude Code subprocess. Defaults to 0 (unknown) if absent.
    ///
    /// The S-009 `extract_pid()` convention documents 0 as the sentinel for "unknown PID";
    /// callers must not assume a non-zero PID without a validity check.
    #[serde(default)]
    pub pid: u32,
    /// Tool name (PreToolUse, Notification permission_prompt). Optional.
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Tool input JSON (PreToolUse, Notification permission_prompt). Optional.
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    /// Stop reason (Stop hook). Optional.
    #[serde(default)]
    pub stop_reason: Option<String>,
    /// Working directory (SessionStart hook). Optional.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Transcript path (SessionStart hook). Optional.
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Prompt text (UserPromptSubmit hook). Optional.
    #[serde(default)]
    pub prompt: Option<String>,
    /// Notification type string (Notification hook). Optional.
    #[serde(default)]
    pub notification_type: Option<String>,
    /// Human-readable notification body (Notification hook). Optional.
    #[serde(default)]
    pub message: Option<String>,
}

/// Session lifecycle state for the session registry.
///
/// BC-2.04.009 PC-2: Stop handler transitions session to `Stopped`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// Session is active (hook events are being received).
    Active,
    /// Session has ended (Stop hook received).
    Stopped,
}

/// Per-session record tracked in `SessionRegistry`.
#[derive(Debug, Clone)]
pub struct SessionEntry {
    /// Claude Code's session UUID.
    pub session_id: String,
    /// Current lifecycle state.
    pub state: SessionState,
}

/// Registry of active and recently-stopped Claude Code sessions.
///
/// BC-2.04.007 PC-2, BC-2.04.008 PC-2, BC-2.04.009 PC-2: each hook handler
/// looks up or creates a session entry. All mutations are protected by the
/// registry's internal lock.
///
/// The registry is `Arc<SessionRegistry>` so it can be shared across axum handler tasks.
#[derive(Debug, Default)]
pub struct SessionRegistry {
    sessions: std::sync::Mutex<std::collections::HashMap<String, SessionEntry>>,
}

impl SessionRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Look up a session by ID, creating it in Active state if absent.
    ///
    /// Returns a clone of the current session entry.
    pub fn get_or_create(&self, session_id: &str) -> SessionEntry {
        let mut guard = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .entry(session_id.to_owned())
            .or_insert_with(|| SessionEntry {
                session_id: session_id.to_owned(),
                state: SessionState::Active,
            })
            .clone()
    }

    /// Transition a session to `Stopped` state, creating the entry if absent.
    ///
    /// BC-2.04.009 PC-2: Stop handler calls this after `EngineModule::on_hook()` returns.
    pub fn mark_stopped(&self, session_id: &str) {
        let mut guard = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let entry = guard
            .entry(session_id.to_owned())
            .or_insert_with(|| SessionEntry {
                session_id: session_id.to_owned(),
                state: SessionState::Active,
            });
        entry.state = SessionState::Stopped;
    }

    /// Returns the current state of a session, or `None` if not found.
    pub fn get_state(&self, session_id: &str) -> Option<SessionState> {
        let guard = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        guard.get(session_id).map(|e| e.state.clone())
    }

    /// Snapshot all sessions as `EnrichedSession` values (BC-2.05.002 AC-002, S-022).
    ///
    /// Maps each `SessionEntry` to an `EnrichedSession` using the following field mapping:
    /// - `session_id`: verbatim.
    /// - `harness_type`: `"claude-code"` (Phase 1 only harness; multi-harness is Phase 4).
    /// - `transcript_path`, `config_path`: `None` (not tracked per session in Phase 1).
    /// - `status`: `SessionState::Active` → `SessionStatus::Active`;
    ///   `SessionState::Stopped` → `SessionStatus::Stopped`.
    /// - `last_event_micros`: `None` (not tracked in `SessionEntry` in Phase 1).
    pub fn snapshot_enriched_sessions(&self) -> Vec<monocle_core::engine::EnrichedSession> {
        // BC-2.06.006 PC-3 / ADV Pass-4 C3: resolve display_name from
        // ClaudeCodeModule::metadata().display_name — NOT a hardcoded literal.
        //
        // This single resolution covers all sessions in the snapshot (Phase 1:
        // every session uses harness_type="claude-code"). When metadata() returns
        // Err (HomeUnresolvable in headless/container CI), fall back to the
        // EngineMetadata canonical constant "Claude Code" so the snapshot is
        // never empty-string. The fallback literal matches the metadata() value —
        // it is a guard against environment failures, not a primary data source.
        //
        // `use monocle_core::engine::EngineModule` brings metadata() into scope
        // as a trait method (it is defined on the EngineModule trait).
        use monocle_core::engine::EngineModule;
        let claude_display_name = crate::engine::ClaudeCodeModule::new(String::new())
            .metadata()
            .map(|m| m.display_name.to_owned())
            .unwrap_or_else(|_| "Claude Code".to_owned());

        let guard = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .values()
            .map(|entry| {
                let status = match entry.state {
                    SessionState::Active => monocle_core::engine::SessionStatus::Active,
                    SessionState::Stopped => monocle_core::engine::SessionStatus::Stopped,
                };
                // BC-2.06.006 PC-3 / ADV Pass-3 MAJOR-2: populate display_name with
                // the harness engine's human-readable name so the IPC wire snapshot
                // carries display_name for the TUI sessions filter.
                // ADV Pass-4 C3: sourced from ClaudeCodeModule::metadata() (above),
                // not from a hardcoded literal that could drift from the canonical value.
                monocle_core::engine::EnrichedSession::new_with_display_name(
                    entry.session_id.clone(),
                    "claude-code".to_owned(),
                    None,
                    None,
                    status,
                    None,
                    None, // project_name: Phase 1 default
                    None, // started_at: Phase 1 default
                    0,    // token_count: Phase 1 default
                    None, // cost_usd: Phase 1 default
                    claude_display_name.clone(),
                )
            })
            .collect()
    }
}
