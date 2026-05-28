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
    ///
    /// Phase 1 has two panels: Sessions → EventRibbon → Sessions (round-robin).
    pub fn cycle(&self) -> FocusSnapshot {
        match self {
            FocusSnapshot::Sessions => FocusSnapshot::EventRibbon,
            FocusSnapshot::EventRibbon => FocusSnapshot::Sessions,
        }
    }

    /// Maps this focus target to the corresponding `PanelId`.
    pub fn to_panel_id(&self) -> PanelId {
        match self {
            FocusSnapshot::Sessions => PanelId::Sessions,
            FocusSnapshot::EventRibbon => PanelId::EventRibbon,
        }
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
#[derive(Clone)]
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
///
/// `#[non_exhaustive]` — additional tool variants will be added as Claude Code
/// expands its tool set in future releases.
#[non_exhaustive]
#[derive(Clone)]
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
///
/// Empty-stack collapse invariant (AC-005): whenever a path would produce
/// `Overlay { stack: empty, .. }`, it collapses to `Dashboard { focused: prior }`
/// instead. This invariant is enforced inside this function — callers need not check.
pub fn transition(mode: AppMode, action: Action) -> AppMode {
    match (mode, action) {
        // --- Filtering entry ---
        (AppMode::Dashboard { focused }, Action::StartFilter { panel }) => AppMode::Filtering {
            panel,
            query: String::new(),
            prior: focused,
        },

        // --- Filtering exit (commit or cancel) ---
        (AppMode::Filtering { prior, .. }, Action::CommitFilter) => {
            AppMode::Dashboard { focused: prior }
        }
        (AppMode::Filtering { prior, .. }, Action::CancelFilter) => {
            AppMode::Dashboard { focused: prior }
        }

        // --- Fullscreen entry ---
        (AppMode::Dashboard { focused }, Action::EnterFullscreen { panel }) => {
            AppMode::Fullscreen {
                panel,
                prior: focused,
            }
        }

        // --- Fullscreen exit ---
        (AppMode::Fullscreen { prior, .. }, Action::ExitFullscreen) => {
            AppMode::Dashboard { focused: prior }
        }

        // --- Overlay push from Dashboard ---
        (AppMode::Dashboard { focused }, Action::PushOverlay { modal }) => AppMode::Overlay {
            stack: VecDeque::from([modal]),
            prior: focused,
        },

        // --- Overlay push from Filtering ---
        (AppMode::Filtering { prior, .. }, Action::PushOverlay { modal }) => AppMode::Overlay {
            stack: VecDeque::from([modal]),
            prior,
        },

        // --- Overlay push from existing Overlay (append to back, preserve prior) ---
        (AppMode::Overlay { mut stack, prior }, Action::PushOverlay { modal }) => {
            stack.push_back(modal);
            AppMode::Overlay { stack, prior }
        }

        // --- Overlay pop ---
        (AppMode::Overlay { mut stack, prior }, Action::PopOverlay) => {
            stack.pop_front();
            // Empty-stack collapse invariant (AC-005)
            if stack.is_empty() {
                AppMode::Dashboard { focused: prior }
            } else {
                AppMode::Overlay { stack, prior }
            }
        }

        // --- MoveFocus: cycle focus in Dashboard via FocusSnapshot::cycle() ---
        // BC-2.06.005 PC-2 / AC-006: Tab cycles focus through the panel tab order.
        // FocusSnapshot::cycle() encodes the canonical two-panel round-robin:
        //   Sessions → EventRibbon → Sessions (per SS-tui.md §FocusSnapshot::cycle).
        (AppMode::Dashboard { focused }, Action::MoveFocus) => AppMode::Dashboard {
            focused: focused.cycle(),
        },

        // --- Esc in Overlay is identity (AC-008) ---
        (AppMode::Overlay { stack, prior }, Action::Esc) => AppMode::Overlay { stack, prior },

        // --- OverlayCycleNext: rotates front to back, preserves prior ---
        (AppMode::Overlay { mut stack, prior }, Action::OverlayCycleNext) => {
            if stack.len() > 1 {
                if let Some(front) = stack.pop_front() {
                    stack.push_back(front);
                }
            }
            AppMode::Overlay { stack, prior }
        }

        // --- Identity (all other combinations) ---
        // EC-061: unmatched (mode, action) pairs return identity
        (mode, _) => mode,
    }
}
