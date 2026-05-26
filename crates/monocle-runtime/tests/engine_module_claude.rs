//! Integration tests for `ClaudeCodeModule` — VP-020, VP-022 (S-015).
//!
//! Covers:
//! - AC-001 / BC-2.03.002 PC-4 / EC-033: detect() true for `"claude"` basename
//! - AC-001 / BC-2.03.002 PC-4 / EC-034: detect() true for `"claude.js"` basename
//! - AC-001 / BC-2.03.002 PC-4 / EC-035: detect() false for `"claude-squad"` basename
//! - AC-001: detect() false for `"claudio"` basename
//! - AC-002 / BC-2.03.002 PC-5 / EC-032: detect() false when exe_path is None
//! - AC-001: detect() false for `"Claude"` (case-sensitive)
//! - AC-001: detect() false for `"claude-code-runner"` (suffixed look-alike)
//! - AC-004 / BC-2.03.002 PC-3: id() returns "claude-code"
//! - AC-007 / BC-2.03.004 PC-1 / VP-022: hook_paths() len == 5
//! - AC-007: hook_paths() contains all 5 correct path mappings
//! - AC-010 / BC-2.03.001 EC-031: on_hook() returns Allow for all 5 known variants
//! - AC-008 / BC-2.03.004 PC-2 / EC-038: spawn() panics with todo!()
//! - AC-009 / BC-2.03.004 PC-3 / EC-039: preflight() panics with todo!()
//!
//! # Red Gate
//!
//! Tests for hook_paths(), metadata(), enrich(), spawn(), and preflight() MUST FAIL before
//! implementation. Tests for detect(), id(), and on_hook() will pass (already implemented).
//!
//! # Coverage Map
//!
//! | BC Clause | AC | Test function |
//! |-----------|----|---------------|
//! | BC-2.03.002 PC-4, EC-033 | AC-001 | test_BC_2_03_002_detect_true_for_claude_basename |
//! | BC-2.03.002 PC-4, EC-034 | AC-001 | test_BC_2_03_002_detect_true_for_claude_js_basename |
//! | BC-2.03.002 PC-4, EC-035 | AC-001 | test_BC_2_03_002_detect_false_for_claude_squad |
//! | BC-2.03.002 PC-4 | AC-001 | test_BC_2_03_002_detect_false_for_claudio |
//! | BC-2.03.002 PC-5, EC-032 | AC-002 | test_BC_2_03_002_detect_false_for_exe_path_none |
//! | BC-2.03.002 PC-4 | AC-001 | test_BC_2_03_002_detect_false_case_sensitive |
//! | BC-2.03.002 PC-4 | AC-001 | test_BC_2_03_002_detect_false_for_claude_code_runner |
//! | BC-2.03.002 PC-3 | AC-004 | test_BC_2_03_002_id_returns_claude_code |
//! | BC-2.03.004 PC-1, VP-022 | AC-007 | test_BC_2_03_004_hook_paths_returns_exactly_5_entries |
//! | BC-2.03.004 PC-1 | AC-007 | test_BC_2_03_004_hook_paths_contains_correct_paths |
//! | BC-2.03.001 EC-031 | AC-010 | test_BC_2_03_001_on_hook_session_start_returns_allow |
//! | BC-2.03.001 EC-031 | AC-010 | test_BC_2_03_001_on_hook_pre_tool_use_returns_allow |
//! | BC-2.03.001 EC-031 | AC-010 | test_BC_2_03_001_on_hook_notification_returns_allow |
//! | BC-2.03.001 EC-031 | AC-010 | test_BC_2_03_001_on_hook_stop_returns_allow |
//! | BC-2.03.001 EC-031 | AC-010 | test_BC_2_03_001_on_hook_user_prompt_submit_returns_allow |
//! | BC-2.03.004 PC-2, EC-038 | AC-008 | test_BC_2_03_004_spawn_is_todo_stub |
//! | BC-2.03.004 PC-3, EC-039 | AC-009 | test_BC_2_03_004_preflight_is_todo_stub |

// Test files: expect/unwrap are idiomatic assertion amplification, not production code.
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Non-snake-case test names encode BC IDs with dots-as-underscores per the naming convention.
#![allow(non_snake_case)]

use std::path::PathBuf;

use monocle_core::engine::{HookDecision, EngineModule, ProcessSnapshot};
use monocle_core::hook_events::{HookEvent, HookType};
use monocle_runtime::engine::{ClaudeCodeModule, SpawnArgs};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Construct a minimal `ProcessSnapshot` with the given exe_path for detect() tests.
fn proc_with_exe(exe_path: Option<PathBuf>) -> ProcessSnapshot {
    ProcessSnapshot::new(12345, exe_path, vec!["claude".to_string()], 0)
}

/// Construct a `ClaudeCodeModule` with a dummy base URL (URL validation is deferred to preflight()).
fn module() -> ClaudeCodeModule {
    ClaudeCodeModule::new("http://127.0.0.1:7891".to_string())
}

// ---------------------------------------------------------------------------
// detect() tests — AC-001, AC-002 (BC-2.03.002 PC-4 + PC-5)
// ---------------------------------------------------------------------------

/// AC-001 / EC-033: detect() returns true for the exact basename "claude".
/// Canonical path: `/usr/local/bin/claude`.
#[test]
fn test_BC_2_03_002_detect_true_for_claude_basename() {
    let m = module();
    let proc = proc_with_exe(Some(PathBuf::from("/usr/local/bin/claude")));
    assert!(
        m.detect(&proc),
        "detect() must return true for exe_path with basename exactly 'claude'"
    );
}

/// AC-001 / EC-034: detect() returns true for the exact basename "claude.js".
/// Node.js wrapper path used by npm-installed Claude Code.
#[test]
fn test_BC_2_03_002_detect_true_for_claude_js_basename() {
    let m = module();
    let proc = proc_with_exe(Some(PathBuf::from("/usr/local/bin/claude.js")));
    assert!(
        m.detect(&proc),
        "detect() must return true for exe_path with basename exactly 'claude.js'"
    );
}

/// AC-001 / EC-035: detect() returns false for "claude-squad" — look-alike but distinct process.
#[test]
fn test_BC_2_03_002_detect_false_for_claude_squad() {
    let m = module();
    let proc = proc_with_exe(Some(PathBuf::from("/usr/local/bin/claude-squad")));
    assert!(
        !m.detect(&proc),
        "detect() must return false for exe_path with basename 'claude-squad'"
    );
}

/// AC-001: detect() returns false for "claudio" — distinct basename, not claude.
#[test]
fn test_BC_2_03_002_detect_false_for_claudio() {
    let m = module();
    let proc = proc_with_exe(Some(PathBuf::from("/usr/local/bin/claudio")));
    assert!(
        !m.detect(&proc),
        "detect() must return false for exe_path with basename 'claudio'"
    );
}

/// AC-002 / EC-032: detect() returns false when exe_path is None, regardless of cmdline.
/// BC-2.03.002 PC-5: process with cmdline[0] = "claude" but exe_path = None → false.
/// detect() MUST NOT inspect cmdline as primary signal.
#[test]
fn test_BC_2_03_002_detect_false_for_exe_path_none() {
    let m = module();
    // cmdline[0] = "claude" but exe_path is None — must still return false
    let proc = ProcessSnapshot::new(12345, None, vec!["claude".to_string()], 0);
    assert!(
        !m.detect(&proc),
        "detect() must return false when exe_path is None, even if cmdline[0] == 'claude'"
    );
}

/// AC-001: detect() is case-sensitive — "Claude" (capital C) must return false.
#[test]
fn test_BC_2_03_002_detect_false_case_sensitive() {
    let m = module();
    let proc = proc_with_exe(Some(PathBuf::from("/usr/local/bin/Claude")));
    assert!(
        !m.detect(&proc),
        "detect() must return false for 'Claude' (case-sensitive — only lowercase 'claude' is allowed)"
    );
}

/// AC-001: detect() returns false for "claude-code" — distinct basename.
#[test]
fn test_BC_2_03_002_detect_false_for_claude_code_basename() {
    let m = module();
    let proc = proc_with_exe(Some(PathBuf::from("/usr/local/bin/claude-code")));
    assert!(
        !m.detect(&proc),
        "detect() must return false for exe_path with basename 'claude-code'"
    );
}

/// AC-001: detect() returns false for "claude-code-runner" — suffixed look-alike that
/// shares the "claude" prefix but is not a Claude Code process.
/// This is a distinct test vector from EC-035 ("claude-squad") and the "claude-code"
/// basename test: it exercises the suffix-rejection path for runner-type wrappers that
/// may appear in CI environments alongside the real claude binary.
#[test]
fn test_BC_2_03_002_detect_false_for_claude_code_runner() {
    let m = module();
    let proc = proc_with_exe(Some(PathBuf::from("/usr/local/bin/claude-code-runner")));
    assert!(
        !m.detect(&proc),
        "detect() must return false for exe_path with basename 'claude-code-runner'"
    );
}

// ---------------------------------------------------------------------------
// id() test — AC-004 (BC-2.03.002 PC-3)
// ---------------------------------------------------------------------------

/// AC-004: id() returns the stable string "claude-code".
#[test]
fn test_BC_2_03_002_id_returns_claude_code() {
    let m = module();
    assert_eq!(
        m.id(),
        "claude-code",
        "id() must return exactly 'claude-code' per BC-2.03.002 PC-3"
    );
}

// ---------------------------------------------------------------------------
// hook_paths() tests — AC-007 / VP-022 (BC-2.03.004 PC-1)
// ---------------------------------------------------------------------------

/// AC-007 / VP-022: hook_paths() returns a HashMap with exactly 5 entries.
/// PostToolUse is intentionally absent (JC-2 / BC-2.03.004 invariant 1).
#[test]
fn test_BC_2_03_004_hook_paths_returns_exactly_5_entries() {
    let m = module();
    let paths = m.hook_paths();
    assert_eq!(
        paths.len(),
        5,
        "hook_paths() must return exactly 5 entries per BC-2.03.004 PC-1 / VP-022; got {}",
        paths.len()
    );
}

/// AC-007: hook_paths() maps each HookType to the correct URL path segment.
/// Canonical 5-entry mapping per BC-2.03.004 PC-1 and SS-engine-module.md §Struct-level inherent.
#[test]
fn test_BC_2_03_004_hook_paths_contains_correct_paths() {
    let m = module();
    let paths = m.hook_paths();

    let expected = [
        (HookType::SessionStart, "/hooks/session-start"),
        (HookType::UserPromptSubmit, "/hooks/prompt-submit"),
        (HookType::PreToolUse, "/hooks/pre-tool-use"),
        (HookType::Notification, "/hooks/notification"),
        (HookType::Stop, "/hooks/stop"),
    ];

    for (hook_type, expected_path) in &expected {
        let actual = paths.get(hook_type).unwrap_or_else(|| {
            panic!(
                "hook_paths() must contain an entry for {:?} per BC-2.03.004 PC-1",
                hook_type
            )
        });
        assert_eq!(
            actual.as_str(),
            *expected_path,
            "hook_paths()[{:?}] must equal '{}' per BC-2.03.004 PC-1; got '{}'",
            hook_type,
            expected_path,
            actual
        );
    }
}

// ---------------------------------------------------------------------------
// on_hook() tests — AC-010, AC-011 (BC-2.03.001 EC-031)
// Phase 1: returns Allow for all 5 known variants and wildcard arm.
// ---------------------------------------------------------------------------

/// AC-010 / AC-011: on_hook(SessionStart) returns HookDecision::Allow.
///
/// HookEvent inner structs are `#[non_exhaustive]` — struct literal construction is
/// forbidden from outside monocle-core (E0639). Canonical construction path is
/// serde_json deserialization per SS-engine-module.md §Cross-Crate Constructor Audit.
#[tokio::test]
async fn test_BC_2_03_001_on_hook_session_start_returns_allow() {
    let m = module();
    let event: HookEvent = serde_json::from_str(r#"{
        "SessionStart": {
            "cwd": "/tmp/project",
            "transcript_path": "/tmp/transcript.json",
            "session_id": "test-session-id",
            "pid": 42
        }
    }"#).expect("SessionStart JSON must deserialize");
    let response = m.on_hook(event).await;
    assert_eq!(
        response.decision,
        HookDecision::Allow,
        "on_hook(SessionStart) must return Allow in Phase 1 per BC-2.03.001 EC-031"
    );
}

/// AC-010 / AC-011: on_hook(PreToolUse) returns HookDecision::Allow.
#[tokio::test]
async fn test_BC_2_03_001_on_hook_pre_tool_use_returns_allow() {
    let m = module();
    let event: HookEvent = serde_json::from_str(r#"{
        "PreToolUse": {
            "tool_name": "Bash",
            "tool_input": {"command": "ls"},
            "session_id": "test-session-id",
            "pid": 42
        }
    }"#).expect("PreToolUse JSON must deserialize");
    let response = m.on_hook(event).await;
    assert_eq!(
        response.decision,
        HookDecision::Allow,
        "on_hook(PreToolUse) must return Allow in Phase 1 per BC-2.03.001 EC-031"
    );
}

/// AC-010 / AC-011: on_hook(Notification) returns HookDecision::Allow.
#[tokio::test]
async fn test_BC_2_03_001_on_hook_notification_returns_allow() {
    let m = module();
    let event: HookEvent = serde_json::from_str(r#"{
        "Notification": {
            "notification_type": "permission_prompt",
            "tool_name": "Bash",
            "tool_input": {},
            "message": "Allow bash command?",
            "session_id": "test-session-id",
            "pid": 42
        }
    }"#).expect("Notification JSON must deserialize");
    let response = m.on_hook(event).await;
    assert_eq!(
        response.decision,
        HookDecision::Allow,
        "on_hook(Notification) must return Allow in Phase 1 per BC-2.03.001 EC-031"
    );
}

/// AC-010 / AC-011: on_hook(Stop) returns HookDecision::Allow.
#[tokio::test]
async fn test_BC_2_03_001_on_hook_stop_returns_allow() {
    let m = module();
    let event: HookEvent = serde_json::from_str(r#"{
        "Stop": {
            "stop_reason": "end_turn",
            "session_id": "test-session-id",
            "pid": 42
        }
    }"#).expect("Stop JSON must deserialize");
    let response = m.on_hook(event).await;
    assert_eq!(
        response.decision,
        HookDecision::Allow,
        "on_hook(Stop) must return Allow in Phase 1 per BC-2.03.001 EC-031"
    );
}

/// AC-010 / AC-011: on_hook(UserPromptSubmit) returns HookDecision::Allow.
#[tokio::test]
async fn test_BC_2_03_001_on_hook_user_prompt_submit_returns_allow() {
    let m = module();
    let event: HookEvent = serde_json::from_str(r#"{
        "UserPromptSubmit": {
            "prompt": "Write a test for me",
            "session_id": "test-session-id",
            "pid": 42
        }
    }"#).expect("UserPromptSubmit JSON must deserialize");
    let response = m.on_hook(event).await;
    assert_eq!(
        response.decision,
        HookDecision::Allow,
        "on_hook(UserPromptSubmit) must return Allow in Phase 1 per BC-2.03.001 EC-031"
    );
}

// ---------------------------------------------------------------------------
// spawn() and preflight() stub tests — AC-008, AC-009 (EC-038, EC-039)
// These tests MUST FAIL with a panic (todo!()) until Phase 3 implementation.
// ---------------------------------------------------------------------------

/// AC-008 / EC-038: spawn() is a todo!() stub in Phase 1 — calling it panics.
/// This test exercises the binding signature and verifies the Phase 1 panic contract.
#[tokio::test]
#[should_panic]
async fn test_BC_2_03_004_spawn_is_todo_stub() {
    let m = module();
    let args = SpawnArgs::new(PathBuf::from("/tmp/project"));
    // This must panic with todo!() per BC-2.03.004 PC-2 / EC-038
    let _ = m.spawn(args).await;
}

/// AC-009 / EC-039: preflight() is a todo!() stub in Phase 1 — calling it panics.
/// This test exercises the binding signature and verifies the Phase 1 panic contract.
#[tokio::test]
#[should_panic]
async fn test_BC_2_03_004_preflight_is_todo_stub() {
    let m = module();
    // This must panic with todo!() per BC-2.03.004 PC-3 / EC-039
    let _ = m.preflight().await;
}
