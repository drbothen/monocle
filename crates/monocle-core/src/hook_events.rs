/// Hook event types for the monocle harness plane.
///
/// `HookEvent` is defined in this module (not in `engine`) per BC-2.03.001 invariant 2
/// and SS-core-types-and-abi.md §Non-Exhaustive Inner Structs. The `engine` module
/// references this type; it does not re-declare it.
///
/// # Phase 1 canonical variant set
///
/// The 5 variants match the 5 canonical Claude Code hook endpoints. `PostToolUse` is
/// intentionally absent per JC-2 (explicit decision: `post-tool-use` is not part of the
/// Phase 1 canonical 5-endpoint surface; see `oq-research.md §JC-2`). BC-2.03.004
/// invariant 1 implements this constraint.
///
/// # Wildcard arm requirement
///
/// All `match` sites on `HookEvent` MUST include a wildcard arm (`_ => ...`) for
/// fail-open behaviour on unrecognized Phase 4 variants (EC-031). The compiler enforces
/// this because `HookEvent` carries `#[non_exhaustive]`.
use serde_json::Value;

/// Discriminant enum for the hook endpoint type, used as a key in
/// `ClaudeCodeModule::hook_paths()`.
///
/// `#[non_exhaustive]` is required per SS-core-types-and-abi.md §`HookType`
/// (FC-02 resolution): Phase 4 brief §Scope notes a `PostToolUse` revisit. Adding a
/// variant here must not break external `match` sites. All external `match` blocks
/// MUST include a wildcard arm.
///
/// `Hash + Eq` are required for use as `HashMap` keys in `hook_paths()`.
/// `Copy` is derived because the type is 1-byte with no owned data.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HookType {
    /// Matches the `/hooks/session-start` endpoint.
    SessionStart,
    /// Matches the `/hooks/prompt-submit` endpoint.
    UserPromptSubmit,
    /// Matches the `/hooks/pre-tool-use` endpoint.
    PreToolUse,
    /// Matches the `/hooks/notification` endpoint.
    Notification,
    /// Matches the `/hooks/stop` endpoint.
    Stop,
    // NOTE: PostToolUse is intentionally absent — JC-2 / BC-2.03.004 invariant 1.
}

/// The unified hook event carrying the actual event payload.
///
/// `#[non_exhaustive]` permits adding new variants (new hook types) without breaking
/// match sites in Phase 1 consumers. All external `match` blocks must include a
/// wildcard arm (EC-031).
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HookEvent {
    /// Fired when a Claude Code session begins.
    SessionStart(SessionStartEvent),
    /// Fired when the user submits a prompt.
    UserPromptSubmit(UserPromptSubmitEvent),
    /// Fired before Claude Code executes a tool.
    PreToolUse(PreToolUseEvent),
    /// Fired for permission prompts and assistant messages.
    Notification(NotificationEvent),
    /// Fired when a Claude Code session ends.
    Stop(StopEvent),
    // NOTE: PostToolUse is intentionally absent — JC-2 / BC-2.03.004 invariant 1.
}

/// SessionStart hook — fired when a Claude Code session begins.
///
/// Gene-source body fields: `pid`, `session_id`.
/// EX-2 extension adds: `cwd`, `transcript_path`.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionStartEvent {
    /// Working directory of the spawned Claude Code session.
    pub cwd: String,
    /// Absolute path to Claude Code's session transcript file.
    pub transcript_path: String,
    /// Claude Code's own session UUID (not monocle's internal session ID).
    pub session_id: String,
    /// PID of the Claude Code subprocess (process.ppid in the hook JS).
    pub pid: u32,
}

/// UserPromptSubmit hook — fired when the user submits a prompt.
///
/// Gene-source body fields: `pid`, `session_id`.
/// EX-2 extension adds: `prompt` text.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPromptSubmitEvent {
    /// The submitted prompt text. May be large; bounded by BC-2.01.003 (256 KiB).
    pub prompt: String,
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
}

/// PreToolUse hook — fired before Claude Code executes a tool.
///
/// Gene-source body fields: `type` ('tool_info'), `pid`, `tool_name`, `tool_input`.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreToolUseEvent {
    /// Name of the tool about to be invoked (e.g., `"Bash"`, `"Edit"`, `"Write"`).
    pub tool_name: String,
    /// JSON-encoded tool input arguments. Stored as [`serde_json::Value`] to avoid
    /// double-deserialization; the TUI renders this in the permission overlay.
    pub tool_input: Value,
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
}

/// Notification hook — fired for permission prompts and assistant messages.
///
/// Gene-source applies a client-side filter (`notification_type == 'permission_prompt'`)
/// before POSTing; monocle preserves that filter in hook-install JS.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NotificationEvent {
    /// `"permission_prompt"` or `"assistant_message"`. Always `"permission_prompt"`
    /// for Phase 1 due to the client-side filter. Retained as `String`
    /// (not enum) for Phase 2 forward compatibility when the filter may relax.
    pub notification_type: String,
    /// Tool name; populated for `permission_prompt` type. Empty string otherwise.
    pub tool_name: String,
    /// JSON tool input; populated for `permission_prompt` type.
    pub tool_input: Value,
    /// Human-readable notification body. May be large; bounded by BC-2.01.003.
    pub message: String,
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
}

/// Stop hook — fired when a Claude Code session ends.
///
/// Gene-source body fields: `pid`, `stop_reason`, `session_id`.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StopEvent {
    /// Reason the session ended. Known values: `"end_turn"` | `"max_tokens"` |
    /// `"tool_use"` | `"error"`. Stored as `String` for forward compatibility.
    pub stop_reason: String,
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
}
