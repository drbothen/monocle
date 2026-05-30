//! Route definitions for the 5 DTU clone hook endpoints.
//!
//! Wires the axum router to the 5 handler functions in `handlers.rs`.
//! The path strings match monocle's canonical endpoint paths per
//! dtu-assessment.md §Endpoint matrix (NOT the gene-source paths from
//! any-context; gene-source uses `/notify`, `/stop`, etc. — monocle uses
//! `/hooks/pre-tool-use`, etc.).
//!
//! Source authority:
//! - AC-001 (5 endpoints)
//! - dtu-assessment.md §Endpoint matrix column "Path" (implemented against v1.7.5 at S-DTU-001 authoring time)
//! - BC-HOOK-007 (endpoint schema)

use axum::{routing::post, Router};

use crate::dtu::{handlers, server::CloneState};

/// Build the axum router with all 5 hook endpoints.
///
/// Endpoints per dtu-assessment.md §Endpoint matrix:
/// - `POST /hooks/pre-tool-use`   — BC-HOOK-007 (PreToolUse schema)
/// - `POST /hooks/notification`   — BC-HOOK-007 (Notification schema)
/// - `POST /hooks/stop`           — BC-HOOK-007 (Stop schema)
/// - `POST /hooks/session-start`  — BC-HOOK-007 (SessionStart schema)
/// - `POST /hooks/prompt-submit`  — BC-HOOK-007 (UserPromptSubmit schema)
///
/// AC-001, BC-HOOK-007
pub fn build_router(state: CloneState) -> Router {
    Router::new()
        .route(paths::PRE_TOOL_USE, post(handlers::handle_pre_tool_use))
        .route(paths::NOTIFICATION, post(handlers::handle_notification))
        .route(paths::STOP, post(handlers::handle_stop))
        .route(paths::SESSION_START, post(handlers::handle_session_start))
        .route(paths::PROMPT_SUBMIT, post(handlers::handle_prompt_submit))
        .with_state(state)
}

/// The 5 canonical hook POST paths as string constants.
/// Used by fidelity tests to construct requests without hardcoding strings.
///
/// Verified against dtu-assessment.md §Endpoint matrix column "Path".
pub mod paths {
    /// `POST /hooks/pre-tool-use`
    pub const PRE_TOOL_USE: &str = "/hooks/pre-tool-use";
    /// `POST /hooks/notification`
    pub const NOTIFICATION: &str = "/hooks/notification";
    /// `POST /hooks/stop`
    pub const STOP: &str = "/hooks/stop";
    /// `POST /hooks/session-start`
    pub const SESSION_START: &str = "/hooks/session-start";
    /// `POST /hooks/prompt-submit`
    pub const PROMPT_SUBMIT: &str = "/hooks/prompt-submit";
}

/// The alias auth header name that real Claude Code hook scripts send.
/// Hardcoded in hooks.go per BC-HOOK-016.
/// The DTU clone MUST use this header to exercise the daemon's ADR-0005
/// compatibility alias code path.
pub const AUTH_HEADER_ALIAS: &str = "X-Claude-Code-Ide-Authorization";

#[cfg(test)]
mod tests {
    use super::paths;

    #[test]
    fn endpoint_paths_match_dtu_assessment_matrix() {
        // Canonical path constants are correct per dtu-assessment.md §Endpoint matrix.
        // Real routing tests in test_bc_hook_*.rs per Step 3.
        assert_eq!(paths::PRE_TOOL_USE, "/hooks/pre-tool-use");
        assert_eq!(paths::NOTIFICATION, "/hooks/notification");
        assert_eq!(paths::STOP, "/hooks/stop");
        assert_eq!(paths::SESSION_START, "/hooks/session-start");
        assert_eq!(paths::PROMPT_SUBMIT, "/hooks/prompt-submit");
    }
}
