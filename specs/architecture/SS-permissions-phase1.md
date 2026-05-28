---
document_type: architecture-section
level: L3
section: "permissions-phase1"
subsystem: cross-cutting
version: "1.5.2"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-17T16:30:00Z
inputs: [product-brief.md, semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md]
input-hash: "1e1d4e7"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# Architecture: Phase 1 Permission Enum

## [Section Content]

## Status

Accepted (architect-produced, human-directed per Q-A-permission-enum Option A).

## Context

Brief v1.3 at spec authoring time introduced `Permission token enum: 17 variants in monocle-core::permissions;
dispatcher no-op until Phase 3 (SOQ-4)`. Those 17 variants were sourced verbatim from
zellij's `PermissionType` enum, which models WASM plugin sandbox capabilities
(`ReadApplicationState`, `OpenFiles`, `RunCommands`, `WebAccess`, etc.).

Adversary fresh-pass finding F-NEW-03 (CRITICAL, commit e2c224b) identifies this as a
Rule 1 violation: monocle is NOT a WASM plugin sandbox host until Phase 3. Shipping
a 17-variant enum with a no-op dispatcher in Phase 1 is dead-code scaffolding for a
future phase. Under the canonical principle (CLAUDE.md §Rule 1), dead scaffolding
is forbidden when the correct Phase 1 surface is derivable in-scope.

The correct Phase 1 permission surface is derivable from Claude Code hook semantics,
specifically the `Notification` hook with `notification_type: 'permission_prompt'`
(BC-HOOK-020) and the `PreToolUse` hook with fail-open semantics (BC-HOOK-018).
monocle's TUI permission overlay intercepts these signals and dispatches one of a
small, closed set of user responses back to the waiting hook.

## Decision

Phase 1 ships a small, purpose-built permission enum derived from Claude Code hook
permission semantics. The zellij-shaped 17-variant enum is reserved for Phase 3
alongside the wasmtime SDK that makes it meaningful.

The Phase 1 permission types reside in `monocle-core::permissions`. They are
**exhaustive by design**; the `#[non_exhaustive]` attribute is forbidden on this
enum. Exhaustiveness is a correctness invariant: the TUI dispatcher must handle
every variant at compile time.

## Phase 1 Permission Enum

### Canonical Definition

```rust
// monocle-core/src/permissions.rs
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Phase 1 permission dispatch surface for Claude Code hook integration.
// This enum models the TUI's response to a Claude Code permission_prompt
// Notification hook event. It is exhaustive — add variants only via ADR.

use std::sync::Arc;
use std::time::Duration;

/// The full set of permission decisions monocle's TUI can produce
/// in response to a Claude Code `permission_prompt` Notification event.
///
/// This enum is EXHAUSTIVE. `#[non_exhaustive]` is forbidden: the
/// match arms in the dispatcher must account for every variant at
/// compile time, and adding a variant is an explicit architectural act
/// (requires an ADR).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase1Permission {
    /// Allow this specific invocation of `tool`, matching the exact
    /// `args_hash`. Does not affect future invocations with different args.
    AllowOnce {
        tool: ClaudeCodeTool,
        args_hash: u64,
    },

    /// Allow all future invocations of `tool` that match `pattern`,
    /// for the duration of the current Claude Code session.
    AllowAlways {
        tool: ClaudeCodeTool,
        pattern: AllowPattern,
    },

    /// Deny this specific invocation of `tool`, matching the exact
    /// `args_hash`. Claude Code receives a non-zero exit from the hook,
    /// which causes it to abort the tool call and surface `reason`
    /// to the user.
    DenyOnce {
        tool: ClaudeCodeTool,
        args_hash: u64,
        reason: DenyReason,
    },

    /// Deny all future invocations of `tool` that match `pattern`,
    /// for the duration of the current Claude Code session.
    DenyAlways {
        tool: ClaudeCodeTool,
        pattern: DenyPattern,
        reason: DenyReason,
    },

    /// Present the permission prompt to the user interactively via the
    /// TUI overlay. The daemon parks the hook response until the user
    /// acts or `timeout` elapses. On timeout, the daemon falls through
    /// to fail-open (echoes stdin unchanged, per BC-HOOK-018).
    ///
    /// `AskUser` is the initial state for every inbound permission_prompt
    /// Notification; the TUI replaces it with AllowOnce/AllowAlways/
    /// DenyOnce/DenyAlways on user action.
    AskUser {
        tool: ClaudeCodeTool,
        args: Arc<HookArgs>,
        timeout: Duration,
    },
}

/// The set of Claude Code tools that monocle can receive
/// permission_prompt Notifications for in Phase 1.
///
/// Derived from BC-HOOK-007 (canonical 5-hook matrix) and the Claude Code
/// 2.x tool surface. `Unknown` is a catch-all for tools added by Anthropic
/// after this enum was defined; monocle must not fail on unknown tools.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClaudeCodeTool {
    Bash,
    Read,
    Write,
    Edit,
    MultiEdit,
    Glob,
    Grep,
    LS,
    WebFetch,
    WebSearch,
    TodoRead,
    TodoWrite,
    NotebookRead,
    NotebookEdit,
    Task,
    /// Catch-all for tools introduced after this enum was defined.
    /// monocle renders the tool name as-is in the TUI overlay and
    /// dispatches AskUser with a 300ms timeout (same as PreToolUse budget).
    Unknown(String),
}

/// Describes which future invocations an AllowAlways decision applies to.
///
/// `#[non_exhaustive]` — Phase 2+ may add new pattern types (e.g., regex match,
/// glob path pattern). Consumers must handle unknown patterns gracefully.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowPattern {
    /// Allow any invocation of the tool, regardless of args.
    AnyArgs,
    /// Allow only invocations whose args hash to `args_hash`.
    ExactArgs { args_hash: u64 },
    /// Allow invocations where the first positional arg matches `prefix`.
    /// Use case: `AllowAlways { tool: Bash, pattern: PathPrefix("/tmp") }`
    /// to permit all Bash commands targeting /tmp.
    PathPrefix(String),
}

/// Describes which future invocations a DenyAlways decision applies to.
///
/// `#[non_exhaustive]` — Phase 2+ may add new pattern types (e.g., glob path
/// deny pattern matching a directory subtree). Consumers must handle unknown patterns.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenyPattern {
    /// Deny any invocation of the tool, regardless of args.
    AnyArgs,
    /// Deny only invocations whose args hash to `args_hash`.
    ExactArgs { args_hash: u64 },
}

/// The reason surfaced to Claude Code (and the user) on a Deny decision.
///
/// `#[non_exhaustive]` — Phase 2+ may add new denial reason types (e.g., a
/// machine-learning classifier deny, a compliance-rule deny). Consumers must
/// handle unknown reasons without panicking.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenyReason {
    /// User explicitly chose to deny.
    UserDenied,
    /// The TUI overlay timed out and the policy is deny-on-timeout.
    /// Default policy is fail-open (AllowOnce), but sessions configured
    /// with `permission_timeout_policy = "deny"` produce this reason.
    Timeout,
    /// The tool matches a policy rule in `~/.monocle/config.json` that
    /// auto-denies without prompting the user.
    PolicyRule { rule_id: String },
}

/// The parsed body of a hook POST, held by AskUser until the user responds.
#[derive(Debug, Clone)]
pub struct HookArgs {
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub message: Option<String>,
}
```

### Variant Justifications

| Variant | Phase 1 use case |
|---------|-----------------|
| `AllowOnce` | User sees the TUI permission overlay and clicks "Allow once." The daemon echoes stdin back to Claude Code (PreToolUse fail-open semantics) for this invocation only. |
| `AllowAlways` | User clicks "Allow always for this tool" or "Allow always for this path." The session-scoped allow-list updates and future Notifications matching the pattern are auto-approved without prompting. |
| `DenyOnce` | User clicks "Deny." The daemon responds to the PreToolUse hook with a non-zero exit, causing Claude Code to abort the tool call and surface the denial. |
| `DenyAlways` | User clicks "Deny always." Matching future Notifications are auto-denied without prompting. Also used by policy-rule auto-deny (`DenyReason::PolicyRule`). |
| `AskUser` | Every inbound `permission_prompt` Notification starts here. The TUI overlay VecDeque (per SS-conventions anti-pattern enforcement) pushes this variant. The user's response replaces it with one of the four above. |

### Exhaustiveness Invariant

The match arms in `monocle-runtime::hook_handler::dispatch_permission` MUST cover
every variant. The compiler enforces this. To add a variant:

1. Produce an ADR recording the new Claude Code tool or new permission decision type.
2. Add the variant to this enum.
3. Add match arms in every `match permission { ... }` site in the codebase.
4. Dispatch a security-reviewer agent: adding a Deny variant path that the user can
   trigger requires review to confirm the Claude Code tool-abort semantics are handled
   correctly.

## What This Enum Is NOT

This is NOT a WASM plugin sandbox permission enum.

The zellij-style permission enum models host-capability grants to untrusted WASM
guest plugins: `ReadApplicationState`, `OpenFiles`, `RunCommands`, `WriteToStdout`,
`ReadApplicationState`, `WebAccess`, `Reconfigure`, `FullHdAccess`, and others.
These variants describe what a plugin binary is allowed to do inside the wasmtime
sandbox.

monocle has no wasmtime sandbox in Phase 1. The 17-variant zellij-shaped enum is
categorically the wrong abstraction for Phase 1's actual problem, which is: "given
that Claude Code's hook protocol has fired a permission_prompt Notification, what
does the user decide?"

## Phase 3 Future Enum

Phase 3 introduces a separate enum `monocle-plugin-sdk::PluginPermission` for WASM
plugin sandbox capabilities. It will be derived from zellij's `PermissionType` (with
monocle-specific naming and any variants required by the wasmtime 44 host ABI).

The Phase 3 enum and the Phase 1 `Phase1Permission` enum are **categorically distinct**
and MUST NOT merge:

- `Phase1Permission`: Claude Code session-permission dispatch (TUI overlay → hook
  response). Lives in `monocle-core`.
- `PluginPermission`: WASM plugin host-capability grants (sandbox boundary). Lives
  in `monocle-plugin-sdk`.

A new artifact `SS-permissions-phase3.md` will be produced during the Phase 3
architecture cycle to define the `PluginPermission` enum surface.

## Consequences

- **Brief v1.4.3 (at spec authoring time):** the permission line references this artifact
  rather than spelling out "17 variants."
- **Phase 1 implementation:** `monocle-core::permissions` has a small, fully-typed,
  exhaustive permission surface with no dead variants. Every variant maps to a
  concrete Phase 1 use case.
- **Phase 3 architecture:** a new artifact `SS-permissions-phase3.md` defines the
  WASM plugin sandbox permission enum separately from this one.
- **Exhaustive enums (two):** `Phase1Permission` (5 canonical variants) and
  `ClaudeCodeTool` (15 named variants + `Unknown(String)`) are exhaustive by design per
  ADR-0004. `#[non_exhaustive]` is FORBIDDEN on these two enums; exhaustiveness is a
  compile-time correctness invariant enforced by the Rust compiler.
- **Non-exhaustive supporting enums (three):** `DenyReason`, `AllowPattern`, and
  `DenyPattern` carry `#[non_exhaustive]` per BC-TYPES-001. Phase 2+ may add new
  pattern or reason variants without breaking Phase 1 consumers. Consumers of these
  enums MUST include a wildcard arm in match statements.

## Re-eval Triggers

If Phase 1 implementation reveals a missing variant (e.g., Anthropic adds a new
Claude Code tool that generates permission_prompt Notifications not covered by the
`Unknown(String)` catch-all in a way that requires different dispatch semantics),
the architect adds the variant via ADR before the implementing story ships. The
`Unknown(String)` catch-all handles graceful degradation for unknown tools without
requiring an ADR — only new dispatch semantics require one.

## §Trace

v1.4 changes (round-59.1 F-R59-adv-2 — §Trace heading drift fix + §Trace-Heading-Convention):
- F-R59-adv-2 RESOLVED (MED + process-gap): Heading `## Trace` (missing `§` prefix) renamed
  to `## §Trace`. The pre-v1.27 PG-3-TRACE-NEW-ENTRY self-audit recipe targeted `^## §Trace`
  exactly; the bare `## Trace` heading caused the grep to return zero matches, silently
  bypassing the defense layer. This file was the sole corpus divergence across 7 SS-*.md +
  dtu-assessment.md. Fix: 1-character insertion of `§`. §Trace-Heading-Convention codified
  in SS-conventions-anti-patterns.md v1.27 closes the class.

v1.3 changes (round-58.1 F-R58-1-cons — PG-3-TRACE-NEW-ENTRY self-violation fix):
- §Trace v1.2 entries contained bare L-number pinpoints (§Context entry and §Consequences
  entry). Bare L-numbers in §Trace prose violate PG-3-TRACE-NEW-ENTRY (codified v1.20).
  Both entries rewritten to position-free form: section names only, no L-number pinpoints.
  No substantive content change — the PG-5 fixes described by v1.2 are unchanged.

v1.2 changes (round-57.1 PG-5 sweep — 2 brief version citations):
- §Context opening sentence lacked PG-5 Form 2 qualifier. Fixed: historical-anchor framing
  added so the citation reads as provenance at authoring time, not a navigation pointer.
- §Consequences opening bullet was future-tense provenance now fulfilled. Fixed: tense
  updated to past-accurate and historical qualifier added. The `traces_to` frontmatter
  field is exempt per PG-5 Option B carve-out (SS-conventions-anti-patterns.md v1.25).

v1.1 changes: #[non_exhaustive] on DenyReason/AllowPattern/DenyPattern per F-FC-O005
regression + G-R14-002; §Consequences scope corrected; round-14 adversary N7 resolved.

v1.0 (initial): Resolves F-NEW-03 (CRITICAL adversary finding, commit e2c224b). Human
Q-A-permission-enum Option A. Gene-source: BC-HOOK-007 (canonical 5-hook matrix),
BC-HOOK-018 (fail-open semantics), BC-HOOK-020 (Notification filter), BC-HOOK-022
(timeout matrix).

**§Trace v1.5** (2026-05-17T11:00:00Z) — Template compliance Dispatch 1:
- NORMATIVE: `document_type` corrected from `architecture-permissions` → `architecture-section`
  per audit §11 (SS-permissions-phase1.md L1 verdict: FAIL; wrong document_type).
- NORMATIVE: `section` field corrected from `"permissions"` → `"permissions-phase1"` (full,
  unambiguous section name per template convention).
- NORMATIVE: `subsystem: cross-cutting` added (cross-cutting file per ARCH-INDEX.md §Cross-Cutting
  Files; Phase1Permission enum applies across all subsystems).
- NORMATIVE: `traces_to` corrected to `architecture/ARCH-INDEX.md` (was long trace-history string;
  ARCH-INDEX.md now created in this dispatch).
- NORMATIVE: `timestamp` bumped to 2026-05-17T11:00:00Z (>= chain high-water 2026-05-17T10:30:00Z;
  SE-16d PASS).
- INFORMATIONAL: Version bump 1.4 → 1.5 records structural fix; no content changes.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §11 (SS-permissions-phase1).
- SE-17g classification: all citations above NORMATIVE or INFORMATIONAL as labeled.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:00:00Z >= chain high-water 2026-05-17T10:30:00Z.
