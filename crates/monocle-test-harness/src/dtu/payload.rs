//! Monocle-canonical payload structs for all 5 hook endpoint POST bodies.
// Stub module: `FixtureScore::compute` body is `todo!()` per TDD Red Gate.
// The `todo` lint is suppressed at file scope; this allow is intentional and
// will be removed when the scoring function is implemented.
#![allow(clippy::todo)]
//!
//! Field definitions sourced from SS-core-types-and-abi.md v1.2.13
//! §Non-Exhaustive Inner Structs. The gene-source fields (from BC-HOOK-007)
//! are retained as struct doc comments for provenance traceability.
//!
//! The DTU clone MUST produce monocle-canonical payloads — gene-source fields
//! alone will fail deserialization at the monocle daemon (dtu-assessment.md
//! §Endpoint matrix column header rationale).
//!
//! BC traces: BC-HOOK-007 (5-endpoint schema), AC-003 (payload structure)

use serde::{Deserialize, Serialize};

/// POST body for `/hooks/pre-tool-use`.
///
/// Monocle-canonical fields per SS-core-types-and-abi.md v1.2.13:
/// `session_id, pid, tool_name, tool_input`
///
/// Gene-source BC-HOOK-007 body fields: `type, pid, tool_name, tool_input`
/// (note: `type` field replaced by `session_id` in monocle-canonical schema).
///
/// BC-HOOK-007 (endpoint schema), BC-HOOK-016 (auth header)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolUsePayload {
    /// Claude Code's own session UUID. Monocle-canonical extension over gene-source.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
    /// Name of the tool about to be invoked (e.g., "Bash", "Edit", "Write").
    pub tool_name: String,
    /// JSON-encoded tool input arguments.
    pub tool_input: serde_json::Value,
}

/// POST body for `/hooks/notification`.
///
/// Monocle-canonical fields per SS-core-types-and-abi.md v1.2.13:
/// `session_id, pid, notification_type, tool_name, tool_input, message`
///
/// Gene-source BC-HOOK-007 body fields: `pid, tool_name, tool_input, message`
/// (note: `session_id` and `notification_type` are monocle extensions).
///
/// BC-HOOK-007 (endpoint schema), BC-HOOK-016 (auth header),
/// BC-HOOK-034 (notification_type filter)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
    /// "permission_prompt" or "assistant_message". Phase 1 always "permission_prompt"
    /// due to client-side filter per SS-core-types-and-abi.md §Non-Exhaustive Inner Structs.
    pub notification_type: String,
    /// Tool name; populated for permission_prompt type. Empty string otherwise.
    pub tool_name: String,
    /// JSON tool input; populated for permission_prompt type.
    pub tool_input: serde_json::Value,
    /// Human-readable notification body.
    pub message: String,
}

/// POST body for `/hooks/stop`.
///
/// Monocle-canonical fields per SS-core-types-and-abi.md v1.2.13:
/// `session_id, pid, stop_reason`
///
/// Gene-source BC-HOOK-007 body fields: `pid, stop_reason, session_id`
///
/// BC-HOOK-007 (endpoint schema), BC-HOOK-016 (auth header)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopPayload {
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
    /// Reason the session ended. Known values: "end_turn" | "max_tokens" |
    /// "tool_use" | "error". Stored as String for forward compatibility.
    pub stop_reason: String,
}

/// POST body for `/hooks/session-start`.
///
/// Monocle-canonical fields per SS-core-types-and-abi.md v1.2.13:
/// `session_id, pid, cwd (EX-2), transcript_path (EX-2)`
///
/// Gene-source BC-HOOK-007 body fields: `pid, session_id`
/// (note: `cwd` and `transcript_path` are EX-2 monocle architect extensions).
///
/// BC-HOOK-007 (endpoint schema), BC-HOOK-016 (auth header)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartPayload {
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
    /// Working directory of the spawned Claude Code session. EX-2 extension.
    pub cwd: String,
    /// Absolute path to Claude Code's session transcript file. EX-2 extension.
    pub transcript_path: String,
}

/// POST body for `/hooks/prompt-submit`.
///
/// Monocle-canonical fields per SS-core-types-and-abi.md v1.2.13:
/// `session_id, pid, prompt (EX-2)`
///
/// Gene-source BC-HOOK-007 body fields: `pid, session_id`
/// (note: `prompt` is an EX-2 monocle architect extension).
///
/// BC-HOOK-007 (endpoint schema), BC-HOOK-016 (auth header)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptSubmitPayload {
    /// Claude Code's own session UUID.
    pub session_id: String,
    /// PID of the Claude Code subprocess.
    pub pid: u32,
    /// The submitted prompt text. EX-2 extension. Bounded by BC-2.01.003 (256 KiB).
    pub prompt: String,
}

/// Fidelity score result for a single fixture comparison.
///
/// BC-HOOK-007 (scoring function), AC-004 (≥0.95 threshold)
#[derive(Debug, Clone)]
pub struct FixtureScore {
    /// Path to the fixture file relative to the fixture corpus root.
    pub fixture_path: String,
    /// Score in [0.0, 1.0]: matched_fields / total_expected_fields.
    pub score: f64,
    /// List of field names that failed the match (present, wrong type, or missing required).
    pub mismatched_fields: Vec<String>,
}

impl FixtureScore {
    /// Construct a FixtureScore by comparing a synthesized payload against a fixture.
    ///
    /// Scoring function per dtu-assessment.md §Scoring Function:
    /// - Missing required fields: 0 per field
    /// - Extra unknown fields: ignored (not penalized)
    /// - Type mismatch on a present field: 0 for that field
    ///
    /// BC-HOOK-007 (scoring), AC-004 (fidelity threshold)
    pub fn compute(
        _fixture_path: String,
        _fixture_json: &serde_json::Value,
        _clone_json: &serde_json::Value,
    ) -> Self {
        todo!("S-DTU-001 implementation pending; BC-HOOK-007 fidelity scoring function")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::expect_used)]
    fn pre_tool_use_payload_serializes_canonical_fields() {
        // Ensures serde derives compile and field names are correct.
        // Real behavioral tests in test_bc_hook_*.rs per Step 3.
        let p = PreToolUsePayload {
            session_id: "s".to_string(),
            pid: 1,
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"command": "ls"}),
        };
        let v = serde_json::to_value(&p).expect("serialize");
        assert!(v.get("session_id").is_some());
        assert!(v.get("pid").is_some());
        assert!(v.get("tool_name").is_some());
        assert!(v.get("tool_input").is_some());
    }
}
