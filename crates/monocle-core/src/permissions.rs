// monocle-core/src/permissions.rs
//
// SPDX-License-Identifier: Apache-2.0
//
// Phase 1 permission dispatch surface for Claude Code hook integration.
// Declares Phase1Permission, ClaudeCodeTool (exhaustive per ADR-0004) and
// DenyReason, AllowPattern, DenyPattern (non-exhaustive per BC-2.02.003).

use std::sync::Arc;
use std::time::Duration;

use crate::hook_events::NotificationEvent;

/// The full set of permission decisions monocle's TUI can produce
/// in response to a Claude Code `permission_prompt` Notification event.
///
/// This enum is EXHAUSTIVE. `#[non_exhaustive]` is **forbidden** on this type
/// per ADR-0004. The match arms in the TUI dispatcher must account for every
/// variant at compile time; adding a variant is an explicit architectural act
/// requiring a new ADR.
///
/// Five variants per SS-permissions-phase1.md §Canonical Definition.
///
/// # `PartialEq` / `Eq` semantics for `AskUser`
///
/// `AskUser` carries `Arc<HookArgs>` which contains `serde_json::Value`.
/// Because `serde_json::Value` does not implement `Eq` (floating-point JSON
/// numbers lack a total order), `PartialEq` is implemented manually.
/// Two `AskUser` variants compare equal if and only if they hold the same
/// `Arc` pointer (pointer equality), the same `tool`, and the same `timeout`.
/// This matches the TUI's identity semantics: two `AskUser` prompts for the
/// same `Arc` allocation represent the same pending decision.
#[derive(Debug, Clone)]
pub enum Phase1Permission {
    /// Allow this specific invocation of `tool`, matching the exact `args_hash`.
    /// Does not affect future invocations with different args.
    AllowOnce {
        /// The Claude Code tool being permitted.
        tool: ClaudeCodeTool,
        /// FNV-1a hash of the tool's serialised arguments.
        args_hash: u64,
    },

    /// Allow all future invocations of `tool` that match `pattern`, for the
    /// duration of the current Claude Code session.
    AllowAlways {
        /// The Claude Code tool being permitted.
        tool: ClaudeCodeTool,
        /// The invocation pattern that this decision covers.
        pattern: AllowPattern,
    },

    /// Deny this specific invocation of `tool`, matching the exact `args_hash`.
    /// Claude Code receives a non-zero exit from the hook, which causes it to
    /// abort the tool call and surface `reason` to the user.
    DenyOnce {
        /// The Claude Code tool being denied.
        tool: ClaudeCodeTool,
        /// FNV-1a hash of the tool's serialised arguments.
        args_hash: u64,
        /// Denial reason surfaced to Claude Code and the user.
        reason: DenyReason,
    },

    /// Deny all future invocations of `tool` that match `pattern`, for the
    /// duration of the current Claude Code session.
    DenyAlways {
        /// The Claude Code tool being denied.
        tool: ClaudeCodeTool,
        /// The invocation pattern that this decision covers.
        pattern: DenyPattern,
        /// Denial reason surfaced to Claude Code and the user.
        reason: DenyReason,
    },

    /// Present the permission prompt to the user interactively via the TUI
    /// overlay. The daemon parks the hook response until the user acts or
    /// `timeout` elapses. On timeout, the daemon falls through to fail-open
    /// (echoes stdin unchanged, per BC-HOOK-018).
    ///
    /// `AskUser` is the initial state for every inbound `permission_prompt`
    /// Notification; the TUI replaces it with `AllowOnce`/`AllowAlways`/
    /// `DenyOnce`/`DenyAlways` on user action.
    AskUser {
        /// The Claude Code tool awaiting the user's decision.
        tool: ClaudeCodeTool,
        /// The full hook event body, parked until user response.
        args: Arc<HookArgs>,
        /// How long the daemon waits before falling through to fail-open.
        timeout: Duration,
    },
}

impl PartialEq for Phase1Permission {
    fn eq(&self, other: &Self) -> bool {
        use Phase1Permission::*;
        match (self, other) {
            (
                AllowOnce {
                    tool: t1,
                    args_hash: h1,
                },
                AllowOnce {
                    tool: t2,
                    args_hash: h2,
                },
            ) => t1 == t2 && h1 == h2,
            (
                AllowAlways {
                    tool: t1,
                    pattern: p1,
                },
                AllowAlways {
                    tool: t2,
                    pattern: p2,
                },
            ) => t1 == t2 && p1 == p2,
            (
                DenyOnce {
                    tool: t1,
                    args_hash: h1,
                    reason: r1,
                },
                DenyOnce {
                    tool: t2,
                    args_hash: h2,
                    reason: r2,
                },
            ) => t1 == t2 && h1 == h2 && r1 == r2,
            (
                DenyAlways {
                    tool: t1,
                    pattern: p1,
                    reason: r1,
                },
                DenyAlways {
                    tool: t2,
                    pattern: p2,
                    reason: r2,
                },
            ) => t1 == t2 && p1 == p2 && r1 == r2,
            // Two AskUser variants are equal iff they share the same Arc allocation,
            // tool, and timeout. Pointer equality is the correct identity for a parked
            // permission prompt: the same Arc means the same pending hook body.
            (
                AskUser {
                    tool: t1,
                    args: a1,
                    timeout: to1,
                },
                AskUser {
                    tool: t2,
                    args: a2,
                    timeout: to2,
                },
            ) => t1 == t2 && Arc::ptr_eq(a1, a2) && to1 == to2,
            _ => false,
        }
    }
}

impl Eq for Phase1Permission {}

/// The set of Claude Code tools that monocle can receive `permission_prompt`
/// Notifications for in Phase 1.
///
/// This enum is EXHAUSTIVE. `#[non_exhaustive]` is **forbidden** on this type
/// per ADR-0004. The named variants are the explicit specification of tools
/// monocle knows about; `Unknown` is the runtime catch-all for tools introduced
/// after this enum was defined. Adding a named variant is a deliberate product
/// decision requiring an ADR.
///
/// Fifteen named variants plus `Unknown(String)` per SS-permissions-phase1.md
/// §Canonical Definition and ADR-0004 §Context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClaudeCodeTool {
    /// The `Bash` shell execution tool.
    Bash,
    /// The `Read` file-read tool.
    Read,
    /// The `Write` file-write tool.
    Write,
    /// The `Edit` single-block file-edit tool.
    Edit,
    /// The `MultiEdit` multi-block file-edit tool.
    MultiEdit,
    /// The `Glob` filesystem glob tool.
    Glob,
    /// The `Grep` file content search tool.
    Grep,
    /// The `LS` directory listing tool.
    LS,
    /// The `WebFetch` URL-fetch tool.
    WebFetch,
    /// The `WebSearch` search query tool.
    WebSearch,
    /// The `TodoRead` todo-list read tool.
    TodoRead,
    /// The `TodoWrite` todo-list write tool.
    TodoWrite,
    /// The `NotebookRead` Jupyter notebook read tool.
    NotebookRead,
    /// The `NotebookEdit` Jupyter notebook edit tool.
    NotebookEdit,
    /// The `Task` sub-agent task tool.
    Task,
    /// Catch-all for tools introduced after this enum was defined.
    ///
    /// monocle renders the tool name as-is in the TUI overlay and dispatches
    /// `AskUser` with a 300 ms timeout (same as the PreToolUse budget per
    /// BC-HOOK-018).
    Unknown(String),
}

/// Describes which future invocations an `AllowAlways` decision applies to.
///
/// `#[non_exhaustive]` — Phase 2+ may add new pattern types (e.g., regex
/// match, glob path pattern). Consumers must handle unknown patterns gracefully.
/// Per SS-permissions-phase1.md lines 160–171.
///
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowPattern {
    /// Allow any invocation of the tool, regardless of args.
    AnyArgs,
    /// Allow only invocations whose args hash to `args_hash`.
    ExactArgs {
        /// FNV-1a hash of the tool's serialised arguments.
        args_hash: u64,
    },
    /// Allow invocations where the first positional arg matches `prefix`.
    ///
    /// Use case: `AllowAlways { tool: Bash, pattern: PathPrefix("/tmp") }`
    /// to permit all Bash commands targeting `/tmp`.
    PathPrefix(String),
}

/// Describes which future invocations a `DenyAlways` decision applies to.
///
/// `#[non_exhaustive]` — Phase 2+ may add new pattern types (e.g., glob path
/// deny pattern matching a directory subtree). Consumers must handle unknown
/// patterns gracefully. Per SS-permissions-phase1.md lines 177–184.
///
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenyPattern {
    /// Deny any invocation of the tool, regardless of args.
    AnyArgs,
    /// Deny only invocations whose args hash to `args_hash`.
    ExactArgs {
        /// FNV-1a hash of the tool's serialised arguments.
        args_hash: u64,
    },
}

/// The reason surfaced to Claude Code (and the user) on a Deny decision.
///
/// `#[non_exhaustive]` — Phase 2+ may add new denial reason types (e.g., a
/// machine-learning classifier deny, a compliance-rule deny). Consumers must
/// handle unknown reasons without panicking. Per SS-permissions-phase1.md
/// lines 191–203.
///
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenyReason {
    /// User explicitly chose to deny.
    UserDenied,
    /// The TUI overlay timed out and the policy is deny-on-timeout.
    ///
    /// Default policy is fail-open (`AllowOnce`), but sessions configured
    /// with `permission_timeout_policy = "deny"` produce this reason.
    Timeout,
    /// The tool matches a policy rule in `~/.monocle/config.json` that
    /// auto-denies without prompting the user.
    PolicyRule {
        /// Identifier of the auto-deny policy rule.
        rule_id: String,
    },
}

/// The parsed body of a hook POST, held by `AskUser` until the user responds.
///
/// Wraps the full `NotificationEvent` for the permission prompt, plus a
/// pre-extracted `args_hash` for the allow/deny variant constructors.
#[derive(Debug, Clone)]
pub struct HookArgs {
    /// The full notification event body, as received from the hook endpoint.
    pub event: NotificationEvent,
    /// Pre-computed FNV-1a hash of `event.tool_input` serialised as canonical
    /// JSON. Used by `AllowOnce`/`DenyOnce` variant constructors to avoid
    /// re-serialising the `serde_json::Value` at dispatch time.
    pub args_hash: u64,
}
