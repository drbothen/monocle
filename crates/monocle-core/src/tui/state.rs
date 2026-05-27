//! TUI state machine types for the monocle TUI plane.
//!
//! `AppMode` is the top-level state; `transition` drives the state machine.
//! Per AC-013, `AppMode` is NOT `#[non_exhaustive]` — exhaustive matching is
//! required in the binary crate so the compiler enforces complete mode coverage.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

/// Top-level application mode for the TUI state machine.
///
/// Intentionally NOT `#[non_exhaustive]` (AC-013): the binary crate must
/// exhaustively match all modes so new modes require explicit handling.
pub enum AppMode {
    /// Normal dashboard view with a focused panel.
    Dashboard {
        /// Which panel currently holds keyboard focus.
        focused: FocusSnapshot,
    },
    /// Incremental filter input active on the given panel.
    Filtering {
        /// The panel whose list is being filtered.
        panel: PanelId,
        /// The current filter query string as the user types.
        query: String,
        /// The focus state to restore when filtering ends.
        prior: FocusSnapshot,
    },
    /// One or more permission overlays stacked on the view.
    Overlay {
        /// Stack of pending permission modals; never empty while this mode is active.
        stack: VecDeque<PromptModal>,
        /// The focus state to restore when all overlays are dismissed.
        prior: FocusSnapshot,
    },
    /// A single panel occupies the full terminal.
    Fullscreen {
        /// The panel currently occupying the full terminal.
        panel: PanelId,
        /// The focus state to restore when fullscreen is exited.
        prior: FocusSnapshot,
    },
}

/// Which panel currently holds keyboard focus in `AppMode::Dashboard`.
///
/// `#[non_exhaustive]` — new panels may be added in future waves without
/// forcing a semver break in downstream crates.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FocusSnapshot {
    /// Session list panel (left pane).
    Sessions,
    /// Event ribbon panel (right pane).
    EventRibbon,
}

impl FocusSnapshot {
    /// Returns the next focus target in the cyclic tab order.
    pub fn cycle(&self) -> FocusSnapshot {
        todo!()
    }

    /// Maps this focus target to the corresponding `PanelId`.
    pub fn to_panel_id(&self) -> PanelId {
        todo!()
    }
}

/// Identifier for a renderable panel in the TUI layout.
///
/// `#[non_exhaustive]` — additional panels (e.g., log viewer, plugin panel)
/// will be added in future waves.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PanelId {
    /// Session list (Runtime plane).
    Sessions,
    /// Live event ribbon (Runtime plane).
    EventRibbon,
    /// Binding / customization explorer (Static plane).
    StaticExplorer,
    /// Factory STATE.md viewer (Workflow plane).
    WorkflowPanel,
    /// Engine module status (Harness plane).
    HarnessPanel,
}

/// A pending permission prompt awaiting user disposition.
///
/// Stacked in `AppMode::Overlay::stack` as a `VecDeque<PromptModal>` — never
/// wrapped in `Option` (forbidden pattern per SS-conventions-anti-patterns.md).
pub struct PromptModal {
    /// Stable identifier correlating this modal to the originating hook request.
    pub prompt_id: Uuid,
    /// Claude Code session that issued the hook call.
    pub session_id: String,
    /// Tool name as reported by the hook endpoint (e.g., "Write", "Bash").
    pub tool_name: String,
    /// Structured payload for the tool invocation.
    pub tool_payload: ToolPayload,
    /// Wall-clock instant at which the hook request arrived.
    pub received_at: Instant,
}

/// Structured tool payload for a permission prompt.
///
/// Covers the canonical Phase 1 hook endpoint set plus a `Generic` catch-all
/// for future or unknown tools.
pub enum ToolPayload {
    /// File edit — old content replaced with new content at path.
    Edit {
        /// The original file content before the edit.
        old_content: String,
        /// The proposed file content after the edit.
        new_content: String,
        /// Filesystem path of the file being edited.
        path: PathBuf,
    },
    /// Shell command invocation.
    Bash {
        /// The shell command string to be executed.
        command: String,
    },
    /// File read request.
    Read {
        /// Filesystem path of the file being read.
        path: PathBuf,
    },
    /// Any tool not matching the above variants; raw JSON payload preserved.
    Generic {
        /// The tool name as reported by the hook endpoint.
        tool_name: String,
        /// Raw JSON tool input as received from Claude Code.
        tool_input: serde_json::Value,
    },
}

/// Discrete user or system action that drives the `AppMode` state machine.
///
/// `#[non_exhaustive]` — new actions will be added as panels and interaction
/// patterns expand in future waves.
#[non_exhaustive]
pub enum Action {
    /// Begin incremental filtering on the given panel.
    StartFilter {
        /// The panel whose list will be filtered.
        panel: PanelId,
    },
    /// Commit the current filter query and return to Dashboard.
    CommitFilter,
    /// Cancel filtering without applying the query.
    CancelFilter,
    /// Expand a panel to fullscreen.
    EnterFullscreen {
        /// The panel to expand to fullscreen.
        panel: PanelId,
    },
    /// Collapse fullscreen back to Dashboard.
    ExitFullscreen,
    /// Push a new permission modal onto the overlay stack.
    PushOverlay {
        /// The modal to push onto the overlay stack.
        modal: PromptModal,
    },
    /// Dismiss the top-most overlay; returns to Dashboard if the stack empties.
    PopOverlay,
    /// Context-sensitive escape: collapse filter, exit fullscreen, or pop overlay.
    Esc,
    /// Move keyboard focus to the next panel in tab order.
    MoveFocus,
    /// Append a character to the active filter query.
    FilterType(char),
    /// Cycle the visible modal in a multi-overlay stack without dismissing.
    OverlayCycleNext,
    /// Accept the top-most permission prompt for this invocation only.
    PermissionAcceptOnce,
    /// Accept the top-most permission prompt and persist an allow-pattern.
    PermissionAcceptAlways,
    /// Reject the top-most permission prompt.
    PermissionReject,
    /// No-op; used by the key resolver when no binding matches.
    Noop,
}

/// Drive the `AppMode` state machine forward by one step.
///
/// Consumes the current mode and an action; returns the successor mode.
/// The function is pure (no I/O) and must be exercised by unit tests without
/// spawning any async runtime.
pub fn transition(_mode: AppMode, _action: Action) -> AppMode {
    todo!()
}
