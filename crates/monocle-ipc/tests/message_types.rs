//! Message type tests for `monocle-ipc`.
// Test function names follow the VSDD `test_BC_S_SS_NNN_xxx` convention (uppercase BC).
// Suppress non_snake_case lint for test files — BC_ prefix is intentional for traceability.
#![allow(non_snake_case)]
//!
//! Covers serde roundtrip for `ServerToClient` / `ClientToServer` variants,
//! `HookType` non_exhaustive enforcement, and `PermissionPromptPayload` serialization.
//!
//! All test names follow `test_BC_S_SS_NNN_[description]` convention.

use monocle_core::engine::{EnrichedSession, SessionStatus};
use monocle_core::hook_events::{HookEvent, HookType};
use monocle_ipc::types::{
    ClientToServer, PermissionDecisionKind, PermissionPromptPayload, ServerToClient,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// ServerToClient serde roundtrip tests
// ---------------------------------------------------------------------------

/// test_BC_2_05_003_server_to_client_session_list_update_serde_roundtrip:
/// `ServerToClient::SessionListUpdate` survives a JSON roundtrip with field values intact.
#[test]
fn test_BC_2_05_003_server_to_client_session_list_update_serde_roundtrip() {
    let msg = ServerToClient::SessionListUpdate { sessions: vec![] };
    let json = serde_json::to_string(&msg).expect("serialize SessionListUpdate");
    let decoded: ServerToClient =
        serde_json::from_str(&json).expect("deserialize SessionListUpdate");
    match decoded {
        ServerToClient::SessionListUpdate { sessions } => {
            assert!(
                sessions.is_empty(),
                "sessions must be empty after roundtrip"
            );
        }
        other => panic!("expected SessionListUpdate, got {other:?}"),
    }
}

/// test_server_to_client_initial_state_serde_roundtrip:
/// `ServerToClient::InitialState` survives a JSON roundtrip with all fields intact.
///
/// Verifies sessions count, ring_tail count, overlay_stack count, and drop_counter value.
#[test]
fn test_server_to_client_initial_state_serde_roundtrip() {
    let session = EnrichedSession::new(
        "session-init-001".to_string(),
        "claude-code".to_string(),
        None,
        None,
        SessionStatus::Active,
        None,
    );

    // SessionStartEvent is #[non_exhaustive] so it cannot be constructed outside monocle-core.
    // Deserialize from JSON to work around the non_exhaustive restriction in tests.
    let hook_event: HookEvent = serde_json::from_value(serde_json::json!({
        "SessionStart": {
            "cwd": "/home/user",
            "transcript_path": "/tmp/transcript.jsonl",
            "session_id": "session-init-001",
            "pid": 12345
        }
    }))
    .expect("construct HookEvent::SessionStart via serde");

    let prompt_id = Uuid::new_v4();
    let overlay = PermissionPromptPayload {
        prompt_id,
        session_id: "session-init-001".to_string(),
        tool_name: "Bash".to_string(),
        tool_input: serde_json::json!({"command": "ls"}),
        old_content: None,
        new_content: None,
    };

    let msg = ServerToClient::InitialState {
        sessions: vec![session],
        ring_tail: vec![hook_event],
        overlay_stack: vec![overlay],
        drop_counter: 42,
    };

    let json = serde_json::to_string(&msg).expect("serialize InitialState");
    let decoded: ServerToClient = serde_json::from_str(&json).expect("deserialize InitialState");

    match decoded {
        ServerToClient::InitialState {
            sessions,
            ring_tail,
            overlay_stack,
            drop_counter,
        } => {
            assert_eq!(
                sessions.len(),
                1,
                "sessions must have 1 entry after roundtrip"
            );
            assert_eq!(
                sessions[0].session_id, "session-init-001",
                "session_id must survive roundtrip"
            );
            assert_eq!(
                ring_tail.len(),
                1,
                "ring_tail must have 1 entry after roundtrip"
            );
            assert_eq!(
                overlay_stack.len(),
                1,
                "overlay_stack must have 1 entry after roundtrip"
            );
            assert_eq!(
                overlay_stack[0].prompt_id, prompt_id,
                "prompt_id must survive roundtrip"
            );
            assert_eq!(drop_counter, 42, "drop_counter must survive roundtrip");
        }
        other => panic!("expected InitialState, got {other:?}"),
    }
}

/// test_BC_2_05_004_server_to_client_hook_event_received_serde_roundtrip:
/// `ServerToClient::HookEventReceived` survives a JSON roundtrip with all fields intact.
#[test]
fn test_BC_2_05_004_server_to_client_hook_event_received_serde_roundtrip() {
    let msg = ServerToClient::HookEventReceived {
        hook_type: HookType::PreToolUse,
        session_id: "test-session-123".to_string(),
        payload_excerpt: r#"{"tool":"Bash"}"#.to_string(),
        latency_ms: 42,
    };
    let json = serde_json::to_string(&msg).expect("serialize HookEventReceived");
    let decoded: ServerToClient =
        serde_json::from_str(&json).expect("deserialize HookEventReceived");
    match decoded {
        ServerToClient::HookEventReceived {
            hook_type,
            session_id,
            payload_excerpt,
            latency_ms,
        } => {
            assert_eq!(hook_type, HookType::PreToolUse);
            assert_eq!(session_id, "test-session-123");
            assert_eq!(payload_excerpt, r#"{"tool":"Bash"}"#);
            assert_eq!(latency_ms, 42);
        }
        other => panic!("expected HookEventReceived, got {other:?}"),
    }
}

/// test_BC_2_05_004_server_to_client_drop_counter_update_serde_roundtrip:
/// `ServerToClient::DropCounterUpdate` survives a JSON roundtrip with counter value intact.
#[test]
fn test_BC_2_05_004_server_to_client_drop_counter_update_serde_roundtrip() {
    let msg = ServerToClient::DropCounterUpdate {
        drop_counter: 1_234_567,
    };
    let json = serde_json::to_string(&msg).expect("serialize DropCounterUpdate");
    let decoded: ServerToClient =
        serde_json::from_str(&json).expect("deserialize DropCounterUpdate");
    match decoded {
        ServerToClient::DropCounterUpdate { drop_counter } => {
            assert_eq!(drop_counter, 1_234_567);
        }
        other => panic!("expected DropCounterUpdate, got {other:?}"),
    }
}

/// test_BC_2_05_003_server_to_client_permission_prompt_queued_serde_roundtrip:
/// `ServerToClient::PermissionPromptQueued` survives a JSON roundtrip.
#[test]
fn test_BC_2_05_003_server_to_client_permission_prompt_queued_serde_roundtrip() {
    let prompt_id = Uuid::new_v4();
    let payload = PermissionPromptPayload {
        prompt_id,
        session_id: "session-abc".to_string(),
        tool_name: "Bash".to_string(),
        tool_input: serde_json::json!({"command": "ls -la"}),
        old_content: None,
        new_content: None,
    };
    let msg = ServerToClient::PermissionPromptQueued { payload };
    let json = serde_json::to_string(&msg).expect("serialize PermissionPromptQueued");
    let decoded: ServerToClient =
        serde_json::from_str(&json).expect("deserialize PermissionPromptQueued");
    match decoded {
        ServerToClient::PermissionPromptQueued { payload } => {
            assert_eq!(payload.prompt_id, prompt_id);
            assert_eq!(payload.session_id, "session-abc");
            assert_eq!(payload.tool_name, "Bash");
        }
        other => panic!("expected PermissionPromptQueued, got {other:?}"),
    }
}

/// test_BC_2_05_003_server_to_client_permission_prompt_resolved_serde_roundtrip:
/// `ServerToClient::PermissionPromptResolved` survives a JSON roundtrip with UUID intact.
#[test]
fn test_BC_2_05_003_server_to_client_permission_prompt_resolved_serde_roundtrip() {
    let prompt_id = Uuid::new_v4();
    let msg = ServerToClient::PermissionPromptResolved { prompt_id };
    let json = serde_json::to_string(&msg).expect("serialize PermissionPromptResolved");
    let decoded: ServerToClient =
        serde_json::from_str(&json).expect("deserialize PermissionPromptResolved");
    match decoded {
        ServerToClient::PermissionPromptResolved {
            prompt_id: decoded_id,
        } => {
            assert_eq!(decoded_id, prompt_id);
        }
        other => panic!("expected PermissionPromptResolved, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// ClientToServer serde roundtrip tests
// ---------------------------------------------------------------------------

/// test_BC_2_05_008_client_to_server_permission_decision_allow_serde_roundtrip:
/// `ClientToServer::PermissionDecision { Allow }` survives a JSON roundtrip.
#[test]
fn test_BC_2_05_008_client_to_server_permission_decision_allow_serde_roundtrip() {
    let prompt_id = Uuid::new_v4();
    let msg = ClientToServer::PermissionDecision {
        prompt_id,
        decision: PermissionDecisionKind::Allow,
    };
    let json = serde_json::to_string(&msg).expect("serialize PermissionDecision(Allow)");
    let decoded: ClientToServer =
        serde_json::from_str(&json).expect("deserialize PermissionDecision(Allow)");
    match decoded {
        ClientToServer::PermissionDecision {
            prompt_id: decoded_id,
            decision,
        } => {
            assert_eq!(decoded_id, prompt_id);
            assert_eq!(decision, PermissionDecisionKind::Allow);
        }
    }
}

/// test_BC_2_05_008_client_to_server_permission_decision_deny_serde_roundtrip:
/// `ClientToServer::PermissionDecision { Deny }` survives a JSON roundtrip.
#[test]
fn test_BC_2_05_008_client_to_server_permission_decision_deny_serde_roundtrip() {
    let prompt_id = Uuid::new_v4();
    let msg = ClientToServer::PermissionDecision {
        prompt_id,
        decision: PermissionDecisionKind::Deny,
    };
    let json = serde_json::to_string(&msg).expect("serialize PermissionDecision(Deny)");
    let decoded: ClientToServer =
        serde_json::from_str(&json).expect("deserialize PermissionDecision(Deny)");
    match decoded {
        ClientToServer::PermissionDecision {
            prompt_id: decoded_id,
            decision,
        } => {
            assert_eq!(decoded_id, prompt_id);
            assert_eq!(decision, PermissionDecisionKind::Deny);
        }
    }
}

// ---------------------------------------------------------------------------
// HookType non_exhaustive enforcement tests
// ---------------------------------------------------------------------------

/// test_BC_2_05_004_hook_type_non_exhaustive_wildcard_arm_compiles:
/// A match on `HookType` with a wildcard arm compiles correctly.
///
/// `#[non_exhaustive]` requires that external match sites include a wildcard arm.
/// This test also documents the fallback label ("unknown") for future variants.
#[test]
fn test_BC_2_05_004_hook_type_non_exhaustive_wildcard_arm_compiles() {
    let all_variants = [
        HookType::PreToolUse,
        HookType::Notification,
        HookType::Stop,
        HookType::SessionStart,
        HookType::UserPromptSubmit,
    ];
    for variant in &all_variants {
        let label = match variant {
            HookType::PreToolUse => "PreToolUse",
            HookType::Notification => "Notification",
            HookType::Stop => "Stop",
            HookType::SessionStart => "SessionStart",
            HookType::UserPromptSubmit => "UserPromptSubmit",
            // Wildcard arm required by #[non_exhaustive]; renders "unknown" for future variants.
            _ => "unknown",
        };
        assert!(
            !label.is_empty(),
            "label must be non-empty for all known variants"
        );
    }
}

/// test_BC_2_05_004_hook_type_all_variants_serde_roundtrip:
/// All five known `HookType` variants survive a JSON serde roundtrip.
#[test]
fn test_BC_2_05_004_hook_type_all_variants_serde_roundtrip() {
    let variants = [
        HookType::PreToolUse,
        HookType::Notification,
        HookType::Stop,
        HookType::SessionStart,
        HookType::UserPromptSubmit,
    ];
    for variant in &variants {
        let json =
            serde_json::to_string(variant).unwrap_or_else(|e| panic!("serialize {variant:?}: {e}"));
        let decoded: HookType = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("deserialize HookType from {json:?}: {e}"));
        assert_eq!(
            *variant, decoded,
            "HookType::{variant:?} must roundtrip through JSON"
        );
    }
}

// ---------------------------------------------------------------------------
// PermissionPromptPayload field coverage tests
// ---------------------------------------------------------------------------

/// test_BC_2_05_003_permission_prompt_payload_with_content_fields:
/// `PermissionPromptPayload` with `old_content` and `new_content` survives serde roundtrip.
#[test]
fn test_BC_2_05_003_permission_prompt_payload_with_content_fields() {
    let prompt_id = Uuid::new_v4();
    let payload = PermissionPromptPayload {
        prompt_id,
        session_id: "session-xyz".to_string(),
        tool_name: "Edit".to_string(),
        tool_input: serde_json::json!({"file_path": "/tmp/foo.rs"}),
        old_content: Some("old code".to_string()),
        new_content: Some("new code".to_string()),
    };
    let json = serde_json::to_string(&payload).expect("serialize payload");
    let decoded: PermissionPromptPayload =
        serde_json::from_str(&json).expect("deserialize payload");
    assert_eq!(decoded.prompt_id, prompt_id);
    assert_eq!(decoded.old_content, Some("old code".to_string()));
    assert_eq!(decoded.new_content, Some("new code".to_string()));
}

/// test_BC_2_05_003_permission_prompt_payload_none_content_fields:
/// `PermissionPromptPayload` with `None` content fields serializes and deserializes correctly.
#[test]
fn test_BC_2_05_003_permission_prompt_payload_none_content_fields() {
    let payload = PermissionPromptPayload {
        prompt_id: Uuid::new_v4(),
        session_id: "s1".to_string(),
        tool_name: "Bash".to_string(),
        tool_input: serde_json::json!({"cmd": "pwd"}),
        old_content: None,
        new_content: None,
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    let decoded: PermissionPromptPayload = serde_json::from_str(&json).expect("deserialize");
    assert!(decoded.old_content.is_none());
    assert!(decoded.new_content.is_none());
}

// ---------------------------------------------------------------------------
// Wire format tag tests (serde(tag = "type"))
// ---------------------------------------------------------------------------

/// test_BC_2_05_003_server_to_client_wire_has_type_tag:
/// The JSON wire format for `ServerToClient` variants includes a `"type"` discriminant field.
///
/// This is required for self-describing messages on the wire (serde tag = "type").
#[test]
fn test_BC_2_05_003_server_to_client_wire_has_type_tag() {
    let msg = ServerToClient::DropCounterUpdate { drop_counter: 0 };
    let json = serde_json::to_string(&msg).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse JSON");
    assert!(
        value.get("type").is_some(),
        "ServerToClient wire format must include a 'type' discriminant field"
    );
    assert_eq!(
        value["type"], "DropCounterUpdate",
        "type field must be the variant name"
    );
}

/// test_BC_2_05_008_client_to_server_wire_has_type_tag:
/// The JSON wire format for `ClientToServer` variants includes a `"type"` discriminant field.
#[test]
fn test_BC_2_05_008_client_to_server_wire_has_type_tag() {
    let msg = ClientToServer::PermissionDecision {
        prompt_id: Uuid::new_v4(),
        decision: PermissionDecisionKind::Allow,
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse JSON");
    assert!(
        value.get("type").is_some(),
        "ClientToServer wire format must include a 'type' discriminant field"
    );
    assert_eq!(value["type"], "PermissionDecision");
}
