//! TUI state machine types for the monocle TUI plane.
//!
//! `AppMode` is the top-level state; `transition` drives the state machine.
//! Per AC-013, `AppMode` is NOT `#[non_exhaustive]` — exhaustive matching is
//! required in the binary crate so the compiler enforces complete mode coverage.

use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

/// Top-level application mode for the TUI state machine.
///
/// Intentionally NOT `#[non_exhaustive]` (AC-013): the binary crate must
/// exhaustively match all modes so new modes require explicit handling.
///
/// `Clone` is derived so that the event loop can call `transition(app.mode.clone(), action)`
/// — `transition` takes ownership of the mode, so cloning avoids moving out of `App`.
#[derive(Clone)]
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
    ///
    /// The actual stack of `PromptModal` items lives ONLY in `App::overlay_stack`
    /// (the single source of truth — F-S025-ADV2-HIGH-003). The existence of this
    /// mode variant indicates "TUI is in overlay mode"; the stack is accessed via
    /// the App-level field, not embedded here.
    ///
    /// Stack mutations (push, pop, cycle) are performed by App methods that update
    /// `App::overlay_stack` directly and then call `transition()` for mode changes.
    /// `transition()` itself never touches the stack — it remains pure on
    /// `(AppMode, Action) → AppMode`.
    Overlay {
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
/// Stored in `App::overlay_stack` as a `VecDeque<PromptModal>` — the single
/// source of truth per F-S025-ADV2-HIGH-003. Never wrapped in `Option`
/// (forbidden pattern per SS-conventions-anti-patterns.md).
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
    /// Context-sensitive escape key action.
    ///
    /// Per-mode behavior (product-owner ruling):
    /// - `Dashboard` → identity (no-op).
    /// - `Overlay` → explicit no-op arm in `transition()` (AC-008).
    /// - `Fullscreen` → identity; only `Action::ExitFullscreen` exits fullscreen
    ///   (wired by the fullscreen-view story, not S-025 skeleton).
    /// - `Filtering` → identity; `Action::CancelFilter` cancels filtering — not Esc.
    ///
    /// Esc is NOT a quit path.
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
    /// Stub: trace the current permission prompt to its source file.
    ///
    /// // Phase 1: stub sets status_message placeholder
    ///
    /// In Phase 1 this is a no-op that writes `App.status_message` to the
    /// canonical placeholder text (BC-2.06.015 PC-1). The `[t]` binding is
    /// registered in Phase 1 to reserve the key for the Phase 2 Static-plane
    /// implementation — preventing keybinding conflicts when the full
    /// trace-to-source behavior ships.
    ///
    /// `transition(Overlay { prior }, PermissionTraceToSource)` is the identity:
    /// mode stays `Overlay { prior }` and `overlay_stack` is not modified
    /// (BC-2.06.015 PC-2). No IPC message is sent (BC-2.06.015 PC-3).
    PermissionTraceToSource,
    /// Move selection to the next row in the currently focused panel list.
    ///
    /// Does not change `AppMode`; the render loop updates the panel's `ListState`.
    SelectNext,
    /// Move selection to the previous row in the currently focused panel list.
    ///
    /// Does not change `AppMode`; the render loop updates the panel's `ListState`.
    SelectPrev,
    /// Request clean exit from the TUI.
    ///
    /// Only registered in Dashboard mode (via the per-context binding layer) so that
    /// typing `q` in Filtering mode inserts the character rather than quitting.
    /// Introduced in F-S025-ADV2-HIGH-002 to replace the overloaded `Action::Esc`
    /// quit-path that broke Filtering-mode filter entry (MED-004 resolution).
    Quit,
    /// No-op; used by the key resolver when no binding matches.
    Noop,
    /// Scroll the Event Ribbon one row toward older events (down the newest-first list).
    ///
    /// Only dispatched when `AppMode::Dashboard { focused: EventRibbon }` is active.
    /// Sets `EventRibbonState::pinned_top = true` (user left row 0).
    /// At the last row (oldest), the scroll offset is clamped — no panic, no wrap.
    ///
    /// BC-2.06.018 PC-5 / AC-007.
    ScrollDown,
    /// Scroll the Event Ribbon one row toward newer events (up the newest-first list).
    ///
    /// Only dispatched when `AppMode::Dashboard { focused: EventRibbon }` is active.
    /// Clears `EventRibbonState::pinned_top = false` when row 0 is reached
    /// (auto-scroll resumes — BC-2.06.018 AC-008).
    ///
    /// BC-2.06.018 PC-5 / AC-007.
    ScrollUp,
}

/// Drive the `AppMode` state machine forward by one step.
///
/// Consumes the current mode and an action; returns the successor mode.
/// The function is pure (no I/O) and must be exercised by unit tests without
/// spawning any async runtime.
///
/// Empty-stack collapse invariant (AC-005): `AppMode::Overlay` has shape
/// `Overlay { prior: FocusSnapshot }` — there is no `stack` field on the mode.
/// The overlay stack lives in `App::overlay_stack` (a `VecDeque<PromptModal>`).
///
/// `transition()` is stack-agnostic: it cannot enforce the empty-stack collapse
/// invariant because it has no access to `App::overlay_stack`. The collapse is
/// enforced APP-LEVEL in `App::pop_overlay` (the `Action::PopOverlay` arm of
/// the run-loop key handler): after popping from `overlay_stack`, if the stack
/// is now empty, the run-loop transitions to `Dashboard { focused: prior }`;
/// if the stack is still non-empty, it re-enters `Overlay`. Callers of
/// `transition()` must perform this post-call check themselves.
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
        //
        // F-S025-ADV2-HIGH-003: The modal is NOT stored in AppMode::Overlay anymore.
        // The caller (App-level handler) must push the modal to `App::overlay_stack`
        // BEFORE calling transition(). transition() here only records the prior focus.
        (AppMode::Dashboard { focused }, Action::PushOverlay { .. }) => {
            AppMode::Overlay { prior: focused }
        }

        // --- Overlay push from Filtering ---
        // Same as Dashboard push: record prior focus, stack mutation is App-level.
        (AppMode::Filtering { prior, .. }, Action::PushOverlay { .. }) => {
            AppMode::Overlay { prior }
        }

        // --- Overlay push from existing Overlay (identity mode change — prior preserved) ---
        //
        // App-level handler has already pushed to overlay_stack. Mode stays Overlay.
        // The `modal` field is ignored here — App::overlay_stack is the single source.
        (AppMode::Overlay { prior }, Action::PushOverlay { .. }) => AppMode::Overlay { prior },

        // --- Overlay pop (collapse invariant enforcement is App-level) ---
        //
        // F-S025-ADV2-HIGH-003: transition() always collapses to Dashboard.
        // The App-level handler:
        //   1. Pops from overlay_stack.
        //   2. Calls transition(mode, PopOverlay) → Dashboard { focused: prior }.
        //   3. If overlay_stack is still non-empty, re-enters Overlay: sets
        //      app.mode = AppMode::Overlay { prior: <captured prior> }.
        // This preserves the empty-stack collapse invariant (AC-005) while keeping
        // transition() stack-agnostic.
        (AppMode::Overlay { prior }, Action::PopOverlay) => AppMode::Dashboard { focused: prior },

        // --- MoveFocus: cycle focus in Dashboard via FocusSnapshot::cycle() ---
        // BC-2.06.005 PC-2 / AC-006: Tab cycles focus through the panel tab order.
        // FocusSnapshot::cycle() encodes the canonical two-panel round-robin:
        //   Sessions → EventRibbon → Sessions (per SS-tui.md §FocusSnapshot::cycle).
        (AppMode::Dashboard { focused }, Action::MoveFocus) => AppMode::Dashboard {
            focused: focused.cycle(),
        },

        // --- Esc in Overlay is identity (AC-008) ---
        (AppMode::Overlay { prior }, Action::Esc) => AppMode::Overlay { prior },

        // --- OverlayCycleNext: identity mode change; App-level rotates overlay_stack ---
        //
        // F-S025-ADV2-HIGH-003: the rotation of overlay_stack is an App-level mutation.
        // transition() returns Overlay unchanged — the visual change is from the render
        // loop reading overlay_stack.front() after App rotates it.
        (AppMode::Overlay { prior }, Action::OverlayCycleNext) => AppMode::Overlay { prior },

        // --- PermissionTraceToSource: identity (BC-2.06.015 PC-2) ---
        //
        // Phase 1 stub: App-level handler sets status_message to the canonical placeholder.
        // No mode change, no overlay_stack mutation, no IPC send.
        // The identity arm is needed in all modes (EC-099: Dashboard also returns identity).
        (AppMode::Overlay { prior }, Action::PermissionTraceToSource) => AppMode::Overlay { prior },

        // --- Identity (all other combinations) ---
        // EC-061: unmatched (mode, action) pairs return identity
        (mode, _) => mode,
    }
}
